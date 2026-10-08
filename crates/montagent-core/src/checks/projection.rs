//! What the schema cannot state about a projection (ADR-0167 §5, §8; ADR-0168 §3), as three
//! errors and two reviews, each named at the element it is about.
//!
//! - `E-PROJECTION-PERSPECTIVE-MISSING`: `swivel` or `tilt` written with no `perspective`.
//!   The published schema says this too; the code gives the targeted message.
//! - `E-PROJECTION-PERSPECTIVE-ALONE`: `perspective` written with neither angle, a dead value.
//!   An angle written as `0`, static or keyed, counts as written (ADR-0168 §1).
//! - `E-PROJECTION-EYE`: `perspective` at or inside r, the distance from the origin point to
//!   the farthest corner of the box widened by every effect's reach, at a key or an eased
//!   extreme of `perspective`, of the keyed box, or of an effect parameter.
//! - `R-PROJECTION-SOFT`: the near edge magnified by d / (d − r) more than 2 at one of those
//!   instants, where the eye bound holds. Strong foreshortening may be meant.
//! - `R-PROJECTION-AWAY`: no instant of the element's presence draws: it faces away or is
//!   edge-on at every frame `render` paints, every key and every eased extreme of its angles.
//!
//! Every number is read through the one resolving function and the rasterizer's own
//! [`montagent_render::canvas::eye_bound`], so `validate` measures what the painter draws.

use montagent_render::canvas::{Extent, Facing};
use serde_json::{Value, json};

use crate::animatable;
use crate::finding::Finding;
use crate::model::Ease;
use crate::permissive::Loose;
use crate::projection;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    let fps = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0);
    for (track, element) in document.elements_in_tracks() {
        if !crate::verbs::query::geometry::covers_the_frame(
            element.get("type").and_then(Value::as_str),
        ) {
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
        let angles: Vec<&str> = projection::ANGLES
            .into_iter()
            .filter(|angle| element.get(*angle).is_some())
            .collect();
        let perspective = element.get("perspective").is_some();
        match (angles.is_empty(), perspective) {
            (true, true) => push(
                Finding::new("E-PROJECTION-PERSPECTIVE-ALONE")
                    .repair_value(json!({"value": "drop `perspective`"})),
            ),
            (false, false) => push(
                Finding::new("E-PROJECTION-PERSPECTIVE-MISSING")
                    .field("angles", json!(angles.join("` and `"))),
            ),
            _ => {}
        }
        if angles.is_empty() {
            continue;
        }
        let Some(range) = range(element) else {
            continue;
        };
        if perspective {
            eye(element, range).into_iter().for_each(&mut push);
        }
        if never_front(element, range, fps) {
            push(
                Finding::new("R-PROJECTION-AWAY")
                    .field("start", json!(range.0))
                    .field("end", json!(range.1)),
            );
        }
    }
}

/// The element's half-open range, where it states a non-empty one.
fn range(element: &Value) -> Option<(i64, i64)> {
    let start = element.get("start")?.as_i64()?;
    let end = element.get("end")?.as_i64()?;
    (end > start).then_some((start, end))
}

/// What one instant a check reads at is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Why {
    /// A keyframe record's own `t`.
    Key,
    /// Where an eased segment turns ([`crate::resolve::eased_extremes`]).
    Extreme,
    /// The element's first instant.
    Start,
    /// The element's last instant.
    End,
}

impl Why {
    fn words(self) -> &'static str {
        match self {
            Why::Key => "a key",
            Why::Extreme => "an eased extreme",
            Why::Start => "the element's start",
            Why::End => "the element's last instant",
        }
    }
}

/// Every key and eased extreme of `key` on `owner`, inside `[start, end)`: each key's `t`,
/// and the whole milliseconds either side of every point an eased segment turns at.
fn keys_and_extremes(owner: &Value, key: &str, (start, end): (i64, i64)) -> Vec<(i64, Why)> {
    let Some(records) = animatable::records(owner, key) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let t = |record: &Value| record.get("t").and_then(Value::as_i64);
    for record in records {
        if let Some(t) = t(record) {
            out.push((t, Why::Key));
        }
    }
    for pair in records.windows(2) {
        let (Some(from), Some(to)) = (t(&pair[0]), t(&pair[1])) else {
            continue;
        };
        let Some(ease) = pair[1]
            .get("ease")
            .and_then(|ease| serde_json::from_value::<Ease>(ease.clone()).ok())
        else {
            continue;
        };
        for fraction in crate::resolve::eased_extremes(&ease) {
            let at = from as f64 + fraction * (to - from) as f64;
            out.push((at.floor() as i64, Why::Extreme));
            out.push((at.ceil() as i64, Why::Extreme));
        }
    }
    out.retain(|(t, _)| (start..end).contains(t));
    out
}

/// The instants ADR-0167 §5 is evaluated at: the element's two ends, and every key and eased
/// extreme of `perspective`, of the box and of every effect parameter, in time order, each
/// once, with the first reason it was asked at.
fn eye_instants(element: &Value, range: (i64, i64)) -> Vec<(i64, Why)> {
    let mut instants = vec![(range.0, Why::Start), (range.1 - 1, Why::End)];
    for key in ["perspective", "width", "height"] {
        instants.extend(keys_and_extremes(element, key, range));
    }
    for declared in animatable::declared(element) {
        if declared.effect.is_some() {
            instants.extend(keys_and_extremes(declared.owner, declared.key(), range));
        }
    }
    instants.sort();
    instants.dedup_by_key(|(t, _)| *t);
    instants
}

/// `perspective` and r at one instant, where the element has a box then.
fn reading(element: &Value, instant: i64) -> Option<(f64, f64)> {
    let t = (i128::from(instant), 1);
    let (width, height) = animatable::painted_box(element, t.0, t.1)?;
    let d = animatable::number_read(element, "perspective", t.0, t.1, f64::INFINITY);
    let r = montagent_render::canvas::eye_bound(
        Extent { width, height },
        projection::origin_of(element),
        &projection::effects_at(element, t),
    );
    Some((d, r))
}

/// `E-PROJECTION-EYE` at the instant r most exceeds `perspective` (the first, on a tie), or else
/// `R-PROJECTION-SOFT` at the instant the near edge is magnified most.
fn eye(element: &Value, range: (i64, i64)) -> Option<Finding> {
    let mut worst_eye: Option<(i64, Why, f64, f64)> = None;
    let mut worst_soft: Option<(i64, f64, f64)> = None;
    for (instant, why) in eye_instants(element, range) {
        let Some((d, r)) = reading(element, instant) else {
            continue;
        };
        if d <= r {
            if worst_eye.is_none_or(|(_, _, seen_d, seen_r)| r - d > seen_r - seen_d) {
                worst_eye = Some((instant, why, d, r));
            }
            continue;
        }
        let magnification = d / (d - r);
        if magnification > 2.0 && worst_soft.is_none_or(|(_, seen, _)| magnification > seen) {
            worst_soft = Some((instant, magnification, r));
        }
    }
    if let Some((instant, why, d, r)) = worst_eye {
        let passing = r.floor() as i64 + 1;
        return Some(
            Finding::new("E-PROJECTION-EYE")
                .field("instant", json!(instant))
                .field("why", json!(why.words()))
                .field("perspective", json!(hundredths(d)))
                .field("r", json!(hundredths(r)))
                .field("passing", json!(passing))
                .repair_value(
                    json!({"value": format!("raise `perspective` to {passing} at {instant} ms")}),
                ),
        );
    }
    let (instant, magnification, r) = worst_soft?;
    Some(
        Finding::new("R-PROJECTION-SOFT")
            .field("instant", json!(instant))
            .field("magnification", json!(hundredths(magnification)))
            .field("perspective", json!((2.0 * r).ceil() as i64)),
    )
}

/// Whether the element faces away or is edge-on at every frame `render` paints inside its
/// range, every key and eased extreme of its angles, and its two ends.
fn never_front(element: &Value, range: (i64, i64), fps: Option<i64>) -> bool {
    let mut instants: Vec<i64> = vec![range.0, range.1 - 1];
    for angle in projection::ANGLES {
        instants.extend(
            keys_and_extremes(element, angle, range)
                .into_iter()
                .map(|(t, _)| t),
        );
    }
    if let Some(fps) = fps
        && let (Some(first), Some(last)) = (
            crate::exact::frame_at_or_after(range.0, fps),
            crate::exact::frame_before(range.1, fps),
        )
    {
        instants.extend((first.frame..=last.frame).map(|n| crate::exact::instant_of(n, fps)));
    }
    instants.into_iter().all(|instant| {
        projection::at(element, (i128::from(instant), 1))
            .is_some_and(|projection| projection.facing() != Facing::Front)
    })
}

/// `value` to two decimals, as the report prints it. Presentational only: every comparison
/// above was made on the unrounded value.
fn hundredths(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
