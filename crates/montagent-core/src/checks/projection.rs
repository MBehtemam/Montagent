//! Projection's checks (ADR-0167 §5, §8; ADR-0168 §3): the three errors that say a projection is
//! not well formed, and the two reviews that say it is well formed and probably not what was
//! meant.
//!
//! - **`E-PROJECTION-PERSPECTIVE-MISSING`**: `swivel` or `tilt` is written and `perspective`
//!   is not. The schema says so too; this is the targeted message.
//! - **`E-PROJECTION-PERSPECTIVE-ALONE`**: `perspective` is written with no `swivel` or `tilt`
//!   key at all, a dead value by `E-DASH-OFFSET-ALONE`'s precedent. An angle of `0`, static or
//!   keyed, counts as present (ADR-0168 §1).
//! - **`E-PROJECTION-EYE`**: `perspective` does not exceed r, the farthest corner of the
//!   reach-widened box from the origin point, at some instant. The bound does not depend on
//!   the angles, so it holds whatever they are, and the painter never needs the part of CSS's
//!   perspective that clips what falls behind the eye.
//! - **`R-PROJECTION-AWAY`**: no instant of the element's presence faces the eye.
//! - **`R-PROJECTION-SOFT`**: the near edge is magnified more than 2×, `d / (d - r)`, at some
//!   instant, which is `d < 2r`, stated as the magnification (ADR-0168 §3).
//!
//! **The instants** are [`crate::projection::instants`]: the element's first instant and its
//! last, every key of the fields that decide the question, and every eased extreme between
//! keys. `r` is [`montagent_render::canvas::eye_bound`], the rasterizer's own, over the same
//! effect reach the painter draws, so what is checked is what is painted.
//!
//! **Document-only**: every number is a literal in the file, resolved by the one resolving
//! function, and nothing is painted.

use montagent_render::canvas::clears_the_eye;
use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::projection::{self, Instant, millis};
use crate::report::Report;
use crate::verbs::query::geometry::covers_the_frame;

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if !covers_the_frame(element.get("type").and_then(Value::as_str)) {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let mut push = |finding: Finding| {
            let finding = finding.at_file(document.path()).at_element(&subject);
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        };
        let written = |key: &str| element.get(key).is_some_and(|value| !value.is_null());
        let angles: Vec<&str> = ["swivel", "tilt"]
            .into_iter()
            .filter(|key| written(key))
            .collect();
        match (angles.is_empty(), written("perspective")) {
            (false, false) => push(
                Finding::new("E-PROJECTION-PERSPECTIVE-MISSING")
                    .field("field", json!(angles.join("` and `"))),
            ),
            (true, true) => push(
                Finding::new("E-PROJECTION-PERSPECTIVE-ALONE")
                    .repair_value(json!({"value": "drop `perspective`"})),
            ),
            _ => {}
        }
        if !projection::projects(element) {
            continue;
        }
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        if end <= start {
            continue;
        }
        let instants = projection::instants(element, start, end, false);
        eye(element, &instants).into_iter().for_each(&mut push);
        away(element, start, end).into_iter().for_each(&mut push);
        soft(element, &instants).into_iter().for_each(&mut push);
    }
}

/// An instant for a finding's field: a whole millisecond as the integer it is, an eased
/// extreme to the thousandth.
fn instant_json(at: Instant) -> Value {
    let ms = millis(at);
    if ms == ms.round() {
        json!(ms as i64)
    } else {
        json!((ms * 1000.0).round() / 1000.0)
    }
}

/// A distance to the hundredth of a pixel, for a finding's field.
fn hundredths(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// `E-PROJECTION-EYE`: the instant at which the eye is closest to a corner, once per element.
fn eye(element: &Value, instants: &[Instant]) -> Option<Finding> {
    // Reversed so that, of instants equally bad, the earliest is the one named.
    let (at, perspective, r) = instants
        .iter()
        .rev()
        .filter_map(|at| {
            let (projection, r) = projection::eye_at(element, *at)?;
            (!clears_the_eye(projection.perspective, r)).then_some((*at, projection.perspective, r))
        })
        // The worst: the corner that gets furthest past the eye.
        .max_by(|a, b| (a.2 - a.1).total_cmp(&(b.2 - b.1)))?;
    // The smallest whole `perspective` that clears r: r is rarely whole, and when it is, the
    // bound is strict.
    let minimum = r.floor() as i64 + 1;
    Some(
        Finding::new("E-PROJECTION-EYE")
            .field("at", instant_json(at))
            .field("perspective", json!(hundredths(perspective)))
            .field("r", json!(hundredths(r)))
            .field("minimum", json!(minimum))
            .repair_value(json!({
                "value": format!("`perspective` of at least {minimum} at {} ms", millis(at))
            })),
    )
}

/// `R-PROJECTION-AWAY`: no instant draws, and the element is present for some.
fn away(element: &Value, start: i64, end: i64) -> Option<Finding> {
    use montagent_render::canvas::Facing;
    let instants = projection::instants(element, start, end, true);
    let mut facing = Vec::new();
    for at in &instants {
        // An instant with no box draws nothing either way; the facing is what it would be.
        let projection = projection::at(element, *at)?;
        facing.push(projection.facing());
    }
    if facing.is_empty() || facing.contains(&Facing::Front) {
        return None;
    }
    let edge = facing.iter().all(|facing| *facing == Facing::Edge);
    Some(Finding::new("R-PROJECTION-AWAY").field("edge", json!(edge)))
}

/// `R-PROJECTION-SOFT`: the instant of the greatest magnification of the near edge, once per
/// element, where it exceeds 2×. An instant at or inside the eye bound is `E-PROJECTION-EYE`'s.
fn soft(element: &Value, instants: &[Instant]) -> Option<Finding> {
    let (at, perspective, r, magnification) = instants
        .iter()
        .rev()
        .filter_map(|at| {
            let (projection, r) = projection::eye_at(element, *at)?;
            let d = projection.perspective;
            (d > r).then(|| (*at, d, r, d / (d - r)))
        })
        .filter(|(_, _, _, magnification)| *magnification > 2.0)
        .max_by(|a, b| a.3.total_cmp(&b.3))?;
    Some(
        Finding::new("R-PROJECTION-SOFT")
            .field("at", instant_json(at))
            .field("perspective", json!(hundredths(perspective)))
            .field("r", json!(hundredths(r)))
            .field(
                "magnification",
                json!((magnification * 10.0).round() / 10.0),
            )
            // d / (d - r) = 2 at d = 2r.
            .field("minimum", json!((2.0 * r).ceil() as i64)),
    )
}
