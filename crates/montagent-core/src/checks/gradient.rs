//! The gradient checks the schema cannot state (ADR-0149 §5).
//!
//! - **`E-GRADIENT-STOP-ORDER`** (`error`): in every literal stop list — a static one, or one
//!   keyframe's `v` — the offsets never decrease. The finding names the property path and the
//!   first pair out of order. Equal offsets are legal: they are a hard edge.
//! - **`E-GRADIENT-STOP-COUNT`** (`error`): every `v` in one `stops` keyframe list holds the
//!   same number of stops, because stop *i* blends with stop *i*. The finding names the path
//!   and the first record whose count differs.
//! - **`R-GRADIENT-ONE-COLOUR`** (`review`): a gradient whose resolved paint is a single
//!   colour at **every frame instant** `render` paints in the element's range — its stops
//!   share one colour, or it is a radial whose `radius` is at or below 0. Decided through the
//!   one resolving function ([`crate::animatable::read`]), so it reads the paint the painter
//!   draws, and a flat-to-gradient animation, whose stops differ at some instant, never fires
//!   it.
//!
//! The paint fields are the schema's: every property [`crate::animatable`] types as a
//! [`Kind::Paint`], so a field that gains a paint later joins without an edit here.

use serde_json::{Value, json};

use crate::animatable::{self, Kind, Resolved};
use crate::finding::Finding;
use crate::model::is_keyframe_list;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    let fps = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    for (track, element) in document.elements_in_tracks() {
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        for property in animatable::of_element(element)
            .iter()
            .filter(|property| property.kind == Kind::Paint)
        {
            let name = property.name.as_str();
            let Some(written) = element.get(name).filter(|value| value.is_object()) else {
                continue;
            };
            let mut found = Vec::new();
            found.extend(stop_order(name, written));
            found.extend(stop_count(name, written));
            found.extend(one_colour(element, name, written, fps));
            for finding in found {
                let finding = finding.at_file(document.path()).at_element(&subject);
                report.push(match track {
                    Some(track) => finding.at_track(track),
                    None => finding,
                });
            }
        }
    }
}

/// `R-GRADIENT-ONE-COLOUR`, decided on the resolved paint at every instant `render` paints
/// the element at.
fn one_colour(element: &Value, name: &str, written: &Value, fps: i64) -> Option<Finding> {
    let mut first = None;
    for instant in instants(element, written, fps) {
        let Some(Ok(Resolved::Gradient(gradient))) = animatable::at(element, name, instant) else {
            return None;
        };
        if !gradient.one_colour() {
            return None;
        }
        first.get_or_insert(gradient);
    }
    let gradient = first?;
    let colour = gradient
        .stops()
        .last()
        .map(|stop| stop.color.as_str().to_string())
        .unwrap_or_default();
    let cause = match &gradient {
        crate::model::ResolvedGradient::Radial { radius, .. } if radius.fraction() <= 0.0 => {
            "its `radius` is 0, which paints the last stop's colour over the box"
        }
        _ => "every stop has that colour",
    };
    Some(
        Finding::new("R-GRADIENT-ONE-COLOUR")
            .field("property", json!(name))
            .field("colour", json!(colour))
            .field("cause", json!(cause)),
    )
}

/// The instants to read a gradient at: the one reading a static gradient needs, else every
/// frame `render` paints in the element's `[start, end)` — as [`animatable::never_painted`]
/// takes them, whole milliseconds `⌊n × 1000 / fps⌋` — or its `start` where the range holds
/// none.
fn instants(element: &Value, gradient: &Value, fps: i64) -> Vec<i64> {
    let start = element.get("start").and_then(Value::as_i64).unwrap_or(0);
    let keyed = ["angle", "center", "radius", "stops"]
        .iter()
        .any(|key| gradient.get(key).is_some_and(is_keyframe_list));
    let end = element.get("end").and_then(Value::as_i64);
    let (true, true, Some(end)) = (keyed, fps > 0, end) else {
        return vec![start];
    };
    let (Some(first), Some(last)) = (
        crate::exact::frame_at_or_after(start, fps),
        crate::exact::frame_before(end, fps),
    ) else {
        return vec![start];
    };
    if first.frame > last.frame {
        return vec![start];
    }
    (first.frame..=last.frame)
        .map(|n| crate::exact::instant_of(n, fps))
        .collect()
}

/// The first pair of stops whose offsets decrease, in a static list or in the first keyframe
/// that has one — read off the literal, so it speaks whether or not the rest of the gradient
/// parses.
fn stop_order(property: &str, gradient: &Value) -> Option<Finding> {
    let stops = gradient.get("stops")?;
    let lists: Vec<(Option<i64>, &Value)> = match animatable::records(gradient, "stops") {
        Some(records) => records
            .iter()
            .map(|record| (record.get("t").and_then(Value::as_i64), &record["v"]))
            .collect(),
        None => vec![(None, stops)],
    };
    lists.into_iter().find_map(|(t, list)| {
        let offsets: Vec<Option<f64>> = list
            .as_array()?
            .iter()
            .map(|stop| stop.get("offset").and_then(Value::as_f64))
            .collect();
        let (index, previous, offset) = offsets.windows(2).enumerate().find_map(|(i, pair)| {
            let (previous, offset) = (pair[0]?, pair[1]?);
            (offset < previous).then_some((i + 1, previous, offset))
        })?;
        Some(
            Finding::new("E-GRADIENT-STOP-ORDER")
                .field("property", json!(format!("{property}.stops")))
                .field(
                    "in",
                    json!(match t {
                        Some(t) => format!("the keyframe at t={t}"),
                        None => "the stop list".to_string(),
                    }),
                )
                // Counted from 1, as a reader counts them.
                .field("stop", json!(index + 1))
                .field("offset", json!(offset))
                .field("previous", json!(index))
                .field("previous_offset", json!(previous)),
        )
    })
}

/// The first keyframe of a `stops` list whose number of stops differs from the first
/// keyframe's.
fn stop_count(property: &str, gradient: &Value) -> Option<Finding> {
    let records = animatable::records(gradient, "stops")?;
    let count = |record: &Value| record.get("v").and_then(Value::as_array).map(Vec::len);
    let expected = count(records.first()?)?;
    let (index, record, found) = records.iter().enumerate().find_map(|(i, record)| {
        let found = count(record)?;
        (found != expected).then_some((i + 1, record, found))
    })?;
    Some(
        Finding::new("E-GRADIENT-STOP-COUNT")
            .field("property", json!(format!("{property}.stops")))
            .field("record", json!(index))
            .field("t", record.get("t").cloned().unwrap_or(Value::Null))
            .field("count", json!(found))
            .field("expected", json!(expected)),
    )
}
