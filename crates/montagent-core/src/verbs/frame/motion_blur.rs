//! PROTOTYPE #718 — throwaway. ADR-0155's sample instants and its still test.
//!
//! Frame n is painted at `instant(n)` whole ms. Its N sample instants are the exact
//! rationals
//!
//! ```text
//! t_k = instant + shutter/360 × 1000/fps × ((k + ½)/N − ½)
//!     = (instant·720·fps·N + shutter·1000·(2k + 1 − N)) / (720·fps·N)   ms
//! ```
//!
//! reduced by their gcd, and handed to the resolver's rational entry point
//! ([`crate::resolve::at_instant`]) through the painter's widened time.

use serde_json::Value;

use crate::animatable;
use crate::verbs::query::geometry::number_q;

/// The element's sample instants at the frame painted at `instant`, or `None` where it
/// carries no readable `motion_blur`.
pub(super) fn instants(element: &Value, instant: i64, fps: i64) -> Option<Vec<(i128, i128)>> {
    let field = element.get("motion_blur")?;
    let shutter = field.get("shutter")?.as_i64().filter(|s| (1..=360).contains(s))?;
    let samples = field.get("samples")?.as_i64().filter(|n| (2..=32).contains(n))?;
    let (shutter, n, fps) = (i128::from(shutter), i128::from(samples), i128::from(fps.max(1)));
    let den = 720 * fps * n;
    Some(
        (0..n)
            .map(|k| {
                let num = i128::from(instant) * den + shutter * 1000 * (2 * k + 1 - n);
                let g = gcd(num.abs(), den);
                (num / g, den / g)
            })
            .collect(),
    )
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// ADR-0155 §4: still when every value the element resolves is equal at all N instants —
/// every declared animatable property (transform, opacity, sizes, paints, gradient stops,
/// path points, effect parameters) and every `units` pose.
pub(super) fn still(element: &Value, instants: &[(i128, i128)]) -> bool {
    let mut first: Option<String> = None;
    for t in instants {
        let print = fingerprint(element, *t);
        match &first {
            None => first = Some(print),
            Some(f) if *f != print => return false,
            Some(_) => {}
        }
    }
    true
}

/// `MB718_FORCE_SAMPLING=1`: skip the still test, so a still element goes through the
/// accumulation (the "N identical samples" check). Measurement plumbing only.
pub(super) fn forced() -> bool {
    std::env::var_os("MB718_FORCE_SAMPLING").is_some()
}

/// `MB718_IDENTITY=1`: compare the mean with the first sample's bytes, on stderr.
pub(super) fn identity_check() -> bool {
    std::env::var_os("MB718_IDENTITY").is_some()
}

fn fingerprint(element: &Value, t: (i128, i128)) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for declared in animatable::declared(element) {
        let _ = write!(out, "{}={:?};", declared.path, declared.read(t.0, t.1));
    }
    // The transform once more through the painter's own reading, defaults included.
    let _ = write!(
        out,
        "x={:?};y={:?};s={:?};r={:?};o={:?};",
        number_q::<i64>(element, "x", t, 0.0),
        number_q::<i64>(element, "y", t, 0.0),
        number_q::<[f64; 2]>(element, "scale", t, [1.0, 1.0]),
        number_q::<f64>(element, "rotation", t, 0.0),
        number_q::<f64>(element, "opacity", t, 1.0),
    );
    if let Some(plan) = crate::units::Plan::of(element) {
        for unit in 0..plan.units.len() {
            let _ = write!(out, "u{unit}={:?};", plan.pose_q(unit, t));
        }
    }
    out
}
