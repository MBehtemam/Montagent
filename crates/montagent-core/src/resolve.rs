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
//! That last rule is why [`Interpolate::Out`] exists. A resolved `x` is not an `i64`: the document
//! states `x` in absolute integer pixels (ADR-0012), but a value interpolating between two
//! of them passes through the numbers in between, and a resolver that handed back an integer
//! would be publishing a rounding rule ADR-0035 says there is not. The type says so.
//!
//! `scale`, `rotation`, `opacity` and `ease` control points are *"continuous-only and
//! explicitly exempt"* from the exact-arithmetic rule (#168, restating ADR-0028's scope), so
//! `f64` here is the specified arithmetic rather than a shortcut past [`crate::exact`].
//! `x`, `y` and `volume` are not on that list and run through `f64` here all the same,
//! which is the same rule arriving by ADR-0035's route rather than ADR-0028's: the exempt
//! list is about the *fields the document stores*, and #168's three named exact sites are
//! all derivations that feed a `ceil`/`floor`/`round`. A resolved value feeds none — it is
//! rounded nowhere, by that ADR's own decision — so there is no rounding step for exact
//! arithmetic to protect.

use crate::model::{Animatable, Colour, Ease, EaseName, Keyframe, Length, Points, Vertex};

/// A value that can be interpolated, and what interpolating it produces.
///
/// Spelled `Interpolate` rather than the shorter word the animation world uses for it:
/// `CONTEXT.md` lists *tween* under `_Avoid_` twice, on **Keyframe** and on **Easing**.
///
/// The associated type is the load-bearing part. `x` and `y` are `Animatable<i64>` in the
/// document and resolve to a continuous quantity; `scale` is a pair and resolves to a pair.
/// Writing that as one trait keeps [`at`] a single function over every animated property the
/// format has, rather than one per value shape.
pub trait Interpolate {
    /// What this property reads as once resolved.
    type Out;

    /// The value `p` of the way from `a` to `b`, where `p` is the *eased* progress and not
    /// the raw fraction of the segment's time.
    fn between(a: &Self, b: &Self, p: f64) -> Self::Out;

    /// This value, unchanged — what clamping at either end, and a `step` segment, produce.
    fn held(value: &Self) -> Self::Out;
}

impl Interpolate for f64 {
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        a + (b - a) * p
    }

    fn held(value: &Self) -> f64 {
        *value
    }
}

impl Interpolate for i64 {
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

impl Interpolate for Length {
    /// Continuous once resolved and never rounded, as `x` is (ADR-0146 §2).
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        i64::between(&a.0, &b.0, p)
    }

    fn held(value: &Self) -> f64 {
        value.0 as f64
    }
}

/// A colour on its way between two keyframes: sRGB-encoded components **premultiplied by
/// alpha**, each in `0..=1`, and not yet clamped (ADR-0146 §2, the CSS rule for hex
/// colours).
///
/// Unclamped on purpose. An overshooting bezier can carry a blend past either end, and two
/// readers need to see that it did: the one resolving function clamps it
/// ([`Blend::settle`]), and `shift` refuses to write a split whose blend is out of range
/// rather than clamping it into a literal that bends both halves of the curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blend(pub [f64; 4]);

impl Blend {
    fn of(colour: &Colour) -> Blend {
        let [r, g, b, a] = colour.bytes().map(|byte| f64::from(byte) / 255.0);
        Blend([r * a, g * a, b * a, a])
    }

    /// Whether every component is inside its range: alpha in `0..=1`, and each colour
    /// component in `0..=alpha`, which is what a premultiplied component's range is.
    pub fn in_range(self) -> bool {
        let [r, g, b, a] = self.0;
        (0.0..=1.0).contains(&a) && [r, g, b].iter().all(|c| (0.0..=a).contains(c))
    }

    /// The colour this blend paints: each component clamped to its range, then the colour
    /// components divided back out of alpha, then each rounded to the nearest byte. At
    /// alpha 0 there is no colour left to divide out, and the answer is `#00000000`.
    pub fn settle(self) -> Colour {
        let [r, g, b, a] = self.0;
        let a = a.clamp(0.0, 1.0);
        let byte = |unit: f64| (unit.clamp(0.0, 1.0) * 255.0).round() as u8;
        if a == 0.0 {
            return Colour::from_bytes([0, 0, 0, 0]);
        }
        let straight = |c: f64| c.clamp(0.0, a) / a;
        Colour::from_bytes([
            byte(straight(r)),
            byte(straight(g)),
            byte(straight(b)),
            byte(a),
        ])
    }
}

impl Interpolate for Colour {
    type Out = Blend;

    fn between(a: &Self, b: &Self, p: f64) -> Blend {
        let (a, b) = (Blend::of(a), Blend::of(b));
        Blend(std::array::from_fn(|i| f64::between(&a.0[i], &b.0[i], p)))
    }

    fn held(value: &Self) -> Blend {
        Blend::of(value)
    }
}

impl Interpolate for [f64; 2] {
    type Out = [f64; 2];

    fn between(a: &Self, b: &Self, p: f64) -> [f64; 2] {
        [f64::between(&a[0], &b[0], p), f64::between(&a[1], &b[1], p)]
    }

    fn held(value: &Self) -> [f64; 2] {
        *value
    }
}

/// One vertex of a path between two keyframes: the written vertex's numbers, each
/// interpolated separately, so continuous (ADR-0154 §3). A handle is present where either
/// end writes it; one end missing it reads as the zero offset a missing handle means, so a
/// keyframe list `validate` refuses for its handles (`E-PATH-KEYFRAME-SHAPE`) still
/// resolves to something rather than to nothing.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct VertexAt {
    pub at: [f64; 2],
    #[serde(rename = "in", skip_serializing_if = "Option::is_none")]
    pub arriving: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<[f64; 2]>,
}

impl VertexAt {
    fn of(vertex: &Vertex) -> VertexAt {
        let pair = |[x, y]: [i64; 2]| [x as f64, y as f64];
        VertexAt {
            at: pair(vertex.at),
            arriving: vertex.arriving.map(pair),
            out: vertex.out.map(pair),
        }
    }
}

impl Interpolate for Points {
    type Out = Vec<VertexAt>;

    /// Number by number, vertex by vertex. Two lists of different lengths have no
    /// correspondence to interpolate along — `E-PATH-KEYFRAME-SHAPE` — and hold the earlier
    /// one, as a `step` does.
    fn between(a: &Self, b: &Self, p: f64) -> Vec<VertexAt> {
        if a.0.len() != b.0.len() {
            return Points::held(a);
        }
        let handle = |a: Option<[i64; 2]>, b: Option<[i64; 2]>| {
            (a.is_some() || b.is_some()).then(|| {
                let (a, b) = (a.unwrap_or([0, 0]), b.unwrap_or([0, 0]));
                <[f64; 2]>::between(&a.map(|n| n as f64), &b.map(|n| n as f64), p)
            })
        };
        a.0.iter()
            .zip(&b.0)
            .map(|(a, b)| VertexAt {
                at: <[f64; 2]>::between(&a.at.map(|n| n as f64), &b.at.map(|n| n as f64), p),
                arriving: handle(a.arriving, b.arriving),
                out: handle(a.out, b.out),
            })
            .collect()
    }

    fn held(value: &Self) -> Vec<VertexAt> {
        value.0.iter().map(VertexAt::of).collect()
    }
}

/// Why a keyframe list has no value at an instant.
///
/// Both members are documents the format should not admit, and neither is a value: a
/// resolver that answered them with a number would be the one thing ADR-0011 asks this verb
/// never to do — state a resolved value the document does not determine. Returned rather
/// than panicked on, because a resolver that aborts turns a malformed document into exit 70
/// (ADR-0042).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unresolvable {
    /// A keyframe list with no records. The published schema forbids it and so does
    /// [`Animatable`]'s deserializer, which reads an empty array as a static value and
    /// fails there.
    Empty,
    /// The segment arriving at the record at this `t` states no `ease`, so how the value
    /// travels along it is not in the document. ADR-0082 makes this unreachable for a
    /// keyframe list that has passed the schema check — array order and clock order are the
    /// same order by construction — so this variant now only guards a caller that hands
    /// [`at`]/[`at_instant`] an `Animatable` built without going through that check.
    ///
    /// **No default is supplied**, here or anywhere: ADR-0038 weighed publishing `linear`
    /// and rejected it, because *"a default is still a fact every reader must independently
    /// know and correctly apply; the file itself does not display it."* Holding the previous
    /// value would be that default under another name, and would arrive at the caller
    /// indistinguishable from a hold the author wrote.
    NoEase { t: i64 },
}

/// **One animated property's value at one instant.**
///
/// A static property is that value at every instant, which is not a special case so much as
/// the shortest list: ADR-0012's clamping already makes a one-record list constant
/// everywhere, and a caller must never have to ask which of the two shapes it holds.
pub fn at<T: Interpolate>(property: &Animatable<T>, t: i64) -> Result<T::Out, Unresolvable> {
    at_instant(property, i128::from(t), 1)
}

/// **The same, at an instant that need not be a whole millisecond** — `numerator /
/// denominator`, in milliseconds.
///
/// A *sampled frame's* instant is `n × 1000/fps` ms, which ADR-0035 says is *"not
/// necessarily integral"* and must be evaluated *"in exact rational arithmetic (integer
/// numerator/denominator, never float)"*: at 30 fps the grid step is `100/3` ms and only
/// multiples of 100 ms are frame-exact. [`crate::checks::unreached`] asks what a property
/// resolves to at exactly such an instant, and rounding one to a whole millisecond first
/// would be the rounding rule ADR-0035 says there is not.
///
/// It is this entry point rather than a second resolver for the reason the module doc
/// gives: the rule has two callers already, and for *this* rule the drift has been
/// observed twice. [`at`] is this function with a denominator of 1, so the whole-
/// millisecond case cannot come to disagree with the sub-millisecond one.
pub fn at_instant<T: Interpolate>(
    property: &Animatable<T>,
    numerator: i128,
    denominator: i128,
) -> Result<T::Out, Unresolvable> {
    match property {
        Animatable::Static(value) => Ok(T::held(value)),
        Animatable::Keyed(records) => keyed(records, numerator, denominator),
    }
}

/// The resolved value of a keyframe list at `numerator / denominator` ms.
///
/// Every comparison against a record's own `t` is scaled by the denominator rather than
/// the instant being divided down to it, so an instant between two whole milliseconds
/// lands in the segment that actually contains it.
fn keyed<T: Interpolate>(
    records: &[Keyframe<T>],
    numerator: i128,
    denominator: i128,
) -> Result<T::Out, Unresolvable> {
    // Read in array order rather than sorted by `t`. ADR-0082 guarantees the two agree for
    // any list that has passed the schema check — array order *is* clock order, by
    // construction, closing the gap [#270](https://github.com/MBehtemam/Montagent/issues/270)
    // found — so a per-call sort would only ever reproduce the order already here, paid on
    // every sampled frame the rasterizer resolves. The `debug_assert` below is the defensive
    // check ADR-0082 leaves to the implementer's discretion, for an `Animatable` this module
    // is ever handed without having gone through that check.
    debug_assert!(
        records.windows(2).all(|pair| pair[0].t < pair[1].t),
        "keyed() was handed a keyframe list not in ascending `t` order; the schema check \
         (ADR-0082) should have refused it before it reached the resolver"
    );

    let first = records.first().ok_or(Unresolvable::Empty)?;
    let last = records.last().ok_or(Unresolvable::Empty)?;
    // Clamped at both ends (ADR-0012), inclusively: at exactly the first or last `t` the
    // value *is* that record's, and taking the branch here rather than through the segment
    // arithmetic keeps the endpoint exact rather than the result of a division.
    let scaled = |record: &Keyframe<T>| i128::from(record.t) * denominator;
    if numerator <= scaled(first) {
        return Ok(T::held(&first.v));
    }
    if numerator >= scaled(last) {
        return Ok(T::held(&last.v));
    }

    // The segment `t` is inside: `[a.t, b.t)`, half-open like every other range in the
    // format (ADR-0005), which is what makes a record's own instant belong to the segment
    // arriving at it and to no other. It also makes the division below safe without a guard:
    // the predicate holds only where `a.t <= t < b.t`, so `b.t - a.t` is never zero — which
    // is what keeps two records written at one instant (SPLIT's *"never two records at one
    // `t`"*, broken) from dividing by zero rather than merely being answered oddly.
    // Sorted, and `t` lies strictly between the first and last records, so some window
    // contains it. `Empty` is the honest answer if that ever stops being true, rather than a
    // panic in a view.
    let (a, b) = records
        .windows(2)
        .map(|pair| (&pair[0], &pair[1]))
        .find(|(a, b)| scaled(a) <= numerator && numerator < scaled(b))
        .ok_or(Unresolvable::Empty)?;

    let fraction = (numerator - scaled(a)) as f64 / (scaled(b) - scaled(a)) as f64;
    // `ease` on `b`, never on `a`: it describes the segment **entering** its record
    // (ADR-0012).
    match &b.ease {
        // `step` is the one ease with no bezier — *"it is not a curve"* (ADR-0012) — so the
        // value stays at the previous record until this record's own instant, which the
        // clamp and the segment bounds above already deliver.
        Some(Ease::Named(EaseName::Step)) => Ok(T::held(&a.v)),
        Some(ease) => Ok(T::between(&a.v, &b.v, progress(ease, fraction))),
        None => Err(Unresolvable::NoEase { t: b.t }),
    }
}

/// The eased progress across a transition's window (ADR-0150): the keyframe vocabulary,
/// through the same evaluator keyframes use. `step` holds at `0` until the fraction reaches
/// `1`, so a stepped transition is a cut at the end of its window, as on a keyframe.
pub(crate) fn eased(ease: &Ease, fraction: f64) -> f64 {
    match ease {
        Ease::Named(EaseName::Step) => match fraction >= 1.0 {
            true => 1.0,
            false => 0.0,
        },
        ease => progress(ease, fraction),
    }
}

/// The eased progress along a segment, given the raw fraction of its time.
///
/// Every ease but `step` is a bezier, and `step` is handled by its caller rather than here —
/// it is *"not a curve"* (ADR-0012), and an evaluator with a not-a-curve branch in it is the
/// second mechanism that ADR forbids.
fn progress(ease: &Ease, fraction: f64) -> f64 {
    match ease {
        Ease::Named(name) => match name.bezier() {
            Some(points) => bezier(points, fraction),
            // `step`, which its caller took. Unreachable, and answered rather than panicked:
            // holding is what `step` means.
            None => 0.0,
        },
        Ease::Bezier(points) => bezier(*points, fraction),
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
///
/// `pub(crate)` rather than private: `crate::verbs::shift`'s SPLIT rule (ADR-0012) needs
/// the same curve parameter this module solves for, to subdivide a bezier at the instant
/// it cuts — not to re-evaluate `y` at it, which [`at`] already does.
pub(crate) fn solve(x1: f64, x2: f64, x: f64) -> f64 {
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
