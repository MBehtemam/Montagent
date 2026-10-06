//! **Motion blur** (ADR-0155): a visual element carrying `motion_blur` is painted at N
//! instants spread across a shutter centred on the frame instant, and the N paints are
//! averaged.
//!
//! This module holds the arithmetic the painter, `validate` and `query --at` share: the
//! field as the file writes it, the exact rational sample instants, and whether an element
//! is **still** — every value its own keyframes resolve equal at every instant asked. The
//! values are read through [`crate::animatable::declared`], the one list of animatable
//! properties, and a `units` stagger's poses, so a property added to the schema is sampled
//! without a second list to keep in step.

use serde_json::Value;

use crate::animatable;
use crate::model::MotionBlur;

/// The sample instants of frame `n`, as exact `(numerator, denominator)` milliseconds in
/// lowest terms (ADR-0155 §3):
///
/// `t_k = instant(n) + shutter/360 × (1000/fps) × ((k + ½)/N − ½)`, `k = 0 … N−1`,
///
/// with `instant(n) = ⌊n × 1000 / fps⌋`.
pub fn sample_instants(n: i64, fps: i64, shutter: i64, samples: i64) -> Vec<(i128, i128)> {
    around(
        i128::from(crate::exact::instant_of(n, fps.max(1))),
        1,
        fps,
        shutter,
        samples,
    )
}

/// The frame whose instant is the last at or before `instant` ms: the frame `instant` lies
/// in, as `render` paints the timeline.
pub(crate) fn frame_containing(instant: i64, fps: i64) -> i64 {
    let fps = fps.max(1);
    let mut n = (i128::from(instant) * i128::from(fps)).div_euclid(1000) as i64;
    while crate::exact::instant_of(n + 1, fps) <= instant {
        n += 1;
    }
    while n > 0 && crate::exact::instant_of(n, fps) > instant {
        n -= 1;
    }
    n
}

/// The sample instants centred on `centre_n / centre_d` ms.
fn around(
    centre_n: i128,
    centre_d: i128,
    fps: i64,
    shutter: i64,
    samples: i64,
) -> Vec<(i128, i128)> {
    let (fps, shutter, samples) = (
        i128::from(fps.max(1)),
        i128::from(shutter),
        i128::from(samples.max(1)),
    );
    // shutter·1000·(2k + 1 − N) / (720·fps·N), added to the centre.
    let denominator = 720 * fps * samples * centre_d;
    (0..samples)
        .map(|k| {
            let numerator =
                centre_n * 720 * fps * samples + shutter * 1000 * (2 * k + 1 - samples) * centre_d;
            reduced(numerator, denominator)
        })
        .collect()
}

/// `numerator / denominator` in lowest terms, the denominator positive.
fn reduced(numerator: i128, denominator: i128) -> (i128, i128) {
    let divisor = gcd(numerator.abs(), denominator.abs()).max(1);
    let sign = if denominator < 0 { -1 } else { 1 };
    (sign * numerator / divisor, sign * denominator / divisor)
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// The element's `motion_blur`, where it writes one the model reads. One it does not read is
/// the schema check's error, and the painter paints the element sharp.
pub(crate) fn of(element: &Value) -> Option<MotionBlur> {
    serde_json::from_value(element.get("motion_blur")?.clone()).ok()
}

/// The sample instants of the shutter centred on `instant` ms: a frame instant, or the
/// instant `frame --at` was asked for.
pub(crate) fn samples_at(instant: i64, fps: i64, blur: MotionBlur) -> Vec<(i128, i128)> {
    around(i128::from(instant), 1, fps, blur.shutter, blur.samples)
}

/// Whether no value the element's own keyframes resolve differs between any two instants
/// inside `[start, end)`, a `units` stagger's poses included (ADR-0155 §5): what
/// `R-MOTION-BLUR-STILL` asks, from the file alone. `None` where the element states no
/// readable range.
///
/// A keyed value changes only between two of its records, so the range is cut at every
/// record instant inside it — every animatable property's, every effect and gradient
/// parameter's, and every unit list's after its delay — and each piece is read at its start
/// and at seven more evenly spaced instants. A piece between two records with different
/// values is never constant over a stretch of time, so it differs at one of them.
pub(crate) fn still_throughout(element: &Value) -> Option<bool> {
    let start = element.get("start")?.as_i64()?;
    let end = element.get("end")?.as_i64()?;
    if end <= start {
        return Some(true);
    }
    let mut cuts: Vec<i64> = vec![start, end];
    let mut cut = |records: Option<&Vec<Value>>, late: i64| {
        for record in records.into_iter().flatten() {
            if let Some(t) = record.get("t").and_then(Value::as_i64) {
                cuts.push(t + late);
            }
        }
    };
    for declared in animatable::declared(element) {
        cut(declared.records(), 0);
    }
    let plan = crate::units::Plan::of(element);
    if let Some(plan) = &plan {
        for unit in 0..plan.units.len() {
            for property in crate::units::LISTS {
                let (from, late) = plan.source(unit, property);
                cut(animatable::records(from, property), late);
            }
        }
    }
    cuts.retain(|t| (start..=end).contains(t));
    cuts.sort_unstable();
    cuts.dedup();
    let instants: Vec<(i128, i128)> = cuts
        .windows(2)
        .flat_map(|piece| {
            let (from, to) = (i128::from(piece[0]), i128::from(piece[1]));
            (0..8).map(move |j| reduced(from * 8 + (to - from) * j, 8))
        })
        .collect();
    Some(still(element, &instants))
}

/// **Still** at these instants (ADR-0155 §4): every value the element's own keyframes
/// resolve is equal at every one of them — each animatable property it writes (effect
/// parameters, gradient parameters and `points` included), read through the one list, and
/// each `units` pose.
///
/// `volume` is left out: it is heard, not painted, so it moves nothing a sample would show.
pub(crate) fn still(element: &Value, instants: &[(i128, i128)]) -> bool {
    let Some((&first, rest)) = instants.split_first() else {
        return true;
    };
    let unchanged = animatable::declared(element)
        .into_iter()
        .filter(|declared| !(declared.effect.is_none() && declared.key() == "volume"))
        .all(|declared| {
            let at_first = declared.read(first.0, first.1);
            rest.iter().all(|t| declared.read(t.0, t.1) == at_first)
        });
    if !unchanged {
        return false;
    }
    let Some(plan) = crate::units::Plan::of(element) else {
        return true;
    };
    (0..plan.units.len()).all(|unit| {
        let at_first = plan.pose_at(unit, first);
        rest.iter().all(|t| plan.pose_at(unit, *t) == at_first)
    })
}
