//! **The keyframe resolver.** What one animated property's value *is* at one instant.
//!
//! ADR-0011's organising rule for `query` is that it *"returns resolved values, never
//! echoed fields"* — *"echoing `"scale":[[3018,1.0],[18018,1.08]]` back at the agent tells
//! it nothing it did not have; `scale 1.0170` is the entire point."* This module is the one
//! place that 1.0170 comes from.
//!
//! It is owned here rather than by `query`, for the reason [`crate::stack`] is owned where
//! it is: there are two callers. `query --at` asks *what is this value at this instant*, and
//! the rasterizer asks the same question once per sampled frame. A rule with two
//! implementations is the drift ADR-0041 names in its own domain and this project has now
//! found four times — and for *this* rule the drift has already been observed twice, in
//! ADR-0060's own evidence: a Python `query --at` script and a `jq` one-liner disagreeing
//! about the same document.
//!
//! ## Three rules, and where each comes from
//!
//! **Clamping, at both ends** (ADR-0012). Before the first record the value is the first
//! value; after the last it is the last. So *"and then it holds"* costs no syntax, and every
//! one of the fixture's seven trimmed moves — one of which sits 13.8 s past the end of the
//! project — stays well-defined.
//!
//! **`ease` describes the segment *entering* its record** (ADR-0012), so the shape of the
//! travel from record *n* to record *n+1* is read off record *n+1*. That convention was
//! taken against a measured prior (4 of 7 jurors wrote the other one), and it is the whole
//! of why the lookup below reads the *later* record's `ease` rather than the earlier one's.
//!
//! **No rounding rule** (ADR-0035, unanimous 3/3). A resolved value is *"the correct value
//! of a continuous function, evaluated at the frame's own exact sample timestamp"*.
//! Manufacturing a different one — by snapping a declared `t` to the frame grid before
//! interpolating, or by rounding the result — *"would mean a clip's opacity and a clip's
//! visibility could be computed on two different effective clocks"*. So an off-grid `t`
//! needs nothing here: it is an ordinary input to an ordinary function, and there is no
//! branch below that could treat it as anything else. ADR-0012's *"`x` and `y` round to the
//! nearest integer"* is not a contradiction — it scopes itself to SPLIT, which is `shift`
//! **writing** a record back into the document, and this module writes nothing.
//!
//! That last rule is why [`Tween::Out`] exists. A resolved `x` is not an `i64`: the document
//! states `x` in absolute integer pixels (ADR-0012), but a value interpolating between two
//! of them passes through the numbers in between, and a resolver that handed back an integer
//! would be publishing a rounding rule ADR-0035 says there is not. The type says so.
//!
//! `scale`, `rotation`, `opacity` and `ease` control points are *"continuous-only and
//! explicitly exempt"* from the exact-arithmetic rule (#168, restating ADR-0028's scope), so
//! `f64` here is the specified arithmetic rather than a shortcut past
//! [`crate::exact`].

use crate::model::{Animatable, Ease, Keyframe};

/// A value that can be interpolated, and what interpolating it produces.
///
/// The associated type is the load-bearing part. `x` and `y` are `Animatable<i64>` in the
/// document and resolve to a continuous quantity; `scale` is a pair and resolves to a pair.
/// Writing that as one trait keeps [`at`] a single function over every animated property the
/// format has, rather than one per value shape.
pub trait Tween {
    /// What this property reads as once resolved.
    type Out;

    /// The value `p` of the way from `a` to `b`, where `p` is the *eased* progress and not
    /// the raw fraction of the segment's time.
    fn between(a: &Self, b: &Self, p: f64) -> Self::Out;

    /// This value, unchanged — what clamping at either end, and a `step` segment, produce.
    fn held(value: &Self) -> Self::Out;
}

impl Tween for f64 {
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        a + (b - a) * p
    }

    fn held(value: &Self) -> f64 {
        *value
    }
}

impl Tween for i64 {
    /// A resolved `x` is not an integer. See the module note: the integer-pixel rule
    /// constrains what the document may *state*, never what interpolation passes through.
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        f64::between(&(*a as f64), &(*b as f64), p)
    }

    fn held(value: &Self) -> f64 {
        *value as f64
    }
}

impl Tween for [f64; 2] {
    type Out = [f64; 2];

    fn between(a: &Self, b: &Self, p: f64) -> [f64; 2] {
        [f64::between(&a[0], &b[0], p), f64::between(&a[1], &b[1], p)]
    }

    fn held(value: &Self) -> [f64; 2] {
        *value
    }
}

/// **One animated property's value at one instant.**
///
/// A static property is that value at every instant, which is not a special case so much as
/// the shortest list: ADR-0012's clamping already makes a one-record list constant
/// everywhere, and a caller must never have to ask which of the two shapes it holds.
///
/// `None` only where the list is empty — a shape no parse can produce, since an empty array
/// deserialises as a static value and fails there. It is returned rather than panicked on
/// because a resolver that aborts is a resolver that turns a malformed document into exit
/// 70 (ADR-0042).
pub fn at<T: Tween>(property: &Animatable<T>, t: i64) -> Option<T::Out> {
    match property {
        Animatable::Static(value) => Some(T::held(value)),
        Animatable::Keyed(records) => keyed(records, t),
    }
}

/// The resolved value of a keyframe list at `t`.
fn keyed<T: Tween>(records: &[Keyframe<T>], t: i64) -> Option<T::Out> {
    // Sorted by `t` rather than read in array order. A keyframe's `t` is the whole of what
    // places it on the clock (ADR-0012), so array order determines nothing here — the same
    // reading ADR-0060 fixed for elements, where *"array order carries no meaning"*. Stable,
    // so two records written at one instant keep the order the file has: SPLIT's rule is
    // *"never two records at one `t`"*, and a file that breaks it is not one this resolver
    // gets to silently reorder.
    let mut order: Vec<&Keyframe<T>> = records.iter().collect();
    order.sort_by_key(|record| record.t);

    let first = *order.first()?;
    let last = *order.last()?;
    // Clamped at both ends (ADR-0012), inclusively: at exactly the first or last `t` the
    // value *is* that record's, and taking the branch here rather than through the segment
    // arithmetic keeps the endpoint exact rather than the result of a division.
    if t <= first.t {
        return Some(T::held(&first.v));
    }
    if t >= last.t {
        return Some(T::held(&last.v));
    }

    // The segment `t` is inside: `[a.t, b.t)`, half-open like every other range in the
    // format (ADR-0005), which is what makes a record's own instant belong to the segment
    // arriving at it and to no other. It also makes the division below safe without a guard:
    // the predicate holds only where `a.t <= t < b.t`, so `b.t - a.t` is never zero — which
    // is what keeps two records written at one instant (SPLIT's *"never two records at one
    // `t`"*, broken) from dividing by zero rather than merely being answered oddly.
    let (a, b) = order
        .windows(2)
        .map(|pair| (pair[0], pair[1]))
        .find(|(a, b)| a.t <= t && t < b.t)?;

    let fraction = (t - a.t) as f64 / (b.t - a.t) as f64;
    // `ease` on `b`, never on `a`: it describes the segment **entering** its record
    // (ADR-0012).
    Some(match progress(b.ease.as_ref(), fraction) {
        Some(p) => T::between(&a.v, &b.v, p),
        // A `step` segment, and the one other case: a non-first record carrying no `ease` at
        // all. That is a schema error (ADR-0038) and no parse of this format can produce it
        // — `Animatable`'s deserializer refuses it by position — so this branch is reachable
        // only from a value built in code. It holds the previous value rather than assuming
        // `linear`, because **no default ease is ever defined** (ADR-0038): *"there is never
        // an absent value to normalize."* Inventing one here would be the published default
        // that ADR rejected, hidden one layer down.
        None => T::held(&a.v),
    })
}

/// The eased progress along a segment, given the raw fraction of its time, or `None` where
/// the segment does not travel at all.
fn progress(ease: Option<&Ease>, fraction: f64) -> Option<f64> {
    match ease? {
        // Named `step`, not `hold`, and it has no bezier: it is not a curve, it is the
        // absence of one (ADR-0012). The value stays at the previous record until this
        // record's own instant, which the clamp and the segment bounds above already deliver
        // — so `None` here is the same answer as `step`, arrived at honestly.
        Ease::Named(name) => name.bezier().map(|points| bezier(points, fraction)),
        Ease::Bezier(points) => Some(bezier(*points, fraction)),
    }
}

/// `y` at `x` on the cubic bezier through `(0,0)`, `(x1,y1)`, `(x2,y2)`, `(1,1)` — the one
/// evaluator, which every named ease is sugar over (ADR-0012).
///
/// The curve is parametric, so `x` is not the parameter: the parameter `s` that puts the
/// curve at this `x` has to be solved for first. Newton–Raphson, which converges in a few
/// steps on the ranges this format admits, with bisection as the fallback for the cases it
/// cannot — a control point may put `x'(s)` at zero, and an `x` outside `[0,1]` is a schema
/// error this function is not the place to report.
fn bezier([x1, y1, x2, y2]: [f64; 4], x: f64) -> f64 {
    // A curve whose two coordinates carry identical control values is the identity: its `x`
    // and `y` polynomials are the same polynomial, so `y = x` exactly, and solving for a
    // parameter would answer a question whose exact answer is already in hand. `linear` is
    // the member of the family that matters — every one of the fixture's fourteen keyframes
    // is `linear`, and a resolved `1.0001493333333333` arriving as `1.000149333333278`
    // because it went the long way round is a difference a reader would have to explain.
    //
    // Keyed on the control points and never on the spelling, so a raw `[0, 0, 1, 1]` and the
    // name `linear` take the same path — *"a name is sugar over one evaluator, never a
    // second mechanism"* (ADR-0012).
    if x1 == y1 && x2 == y2 {
        return x;
    }
    let s = solve(x1, x2, x);
    cubic(y1, y2, s)
}

/// One coordinate of the curve at parameter `s`, with the endpoints fixed at 0 and 1.
fn cubic(c1: f64, c2: f64, s: f64) -> f64 {
    let r = 1.0 - s;
    3.0 * r * r * s * c1 + 3.0 * r * s * s * c2 + s * s * s
}

/// The derivative of [`cubic`] with respect to `s`.
fn slope(c1: f64, c2: f64, s: f64) -> f64 {
    let r = 1.0 - s;
    3.0 * r * r * c1 + 6.0 * r * s * (c2 - c1) + 3.0 * s * s * (1.0 - c2)
}

/// The parameter `s` at which the curve's `x` coordinate is `x`.
fn solve(x1: f64, x2: f64, x: f64) -> f64 {
    const EPSILON: f64 = 1e-14;

    let mut s = x;
    for _ in 0..8 {
        let error = cubic(x1, x2, s) - x;
        if error.abs() < EPSILON {
            return s;
        }
        let d = slope(x1, x2, s);
        if d.abs() < EPSILON {
            break;
        }
        s -= error / d;
    }

    // Bisection over the parameter's own domain. It cannot fail to terminate and it cannot
    // leave `[0,1]`, which is what makes it the fallback rather than a second answer: 60
    // halvings take the interval below the spacing of adjacent `f64`s.
    let (mut low, mut high) = (0.0_f64, 1.0_f64);
    let mut s = x.clamp(0.0, 1.0);
    for _ in 0..60 {
        let at = cubic(x1, x2, s);
        if (at - x).abs() < EPSILON {
            return s;
        }
        if at < x {
            low = s;
        } else {
            high = s;
        }
        s = (low + high) / 2.0;
    }
    s
}
