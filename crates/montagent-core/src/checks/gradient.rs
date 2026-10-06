//! The two gradient checks the schema cannot state (ADR-0149 §5).
//!
//! - **`E-GRADIENT-STOP-ORDER`** (`error`): in a literal stop list the offsets never
//!   decrease. The finding names the property path and the first pair out of order. Equal
//!   offsets are legal: they are a hard edge.
//! - **`R-GRADIENT-ONE-COLOUR`** (`review`): a gradient whose resolved paint is a single
//!   colour — its stops share one colour, or it is a radial whose `radius` is at or below 0.
//!   Decided through the one resolving function ([`crate::animatable::read`]), so it reads
//!   the paint the painter draws.
//!
//! The paint fields are the schema's: every property [`crate::animatable`] types as a
//! [`Kind::Paint`], so a field that gains a paint later joins without an edit here. In this
//! slice a gradient is static, so one reading answers for every frame; the keyed cases are
//! the next slice's.

use serde_json::{Value, json};

use crate::animatable::{self, Kind, Resolved};
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let start = element.get("start").and_then(Value::as_i64).unwrap_or(0);
        for property in animatable::of_element(element)
            .iter()
            .filter(|property| property.kind == Kind::Paint)
        {
            let name = property.name.as_str();
            let Some(written) = element.get(name).filter(|value| value.is_object()) else {
                continue;
            };
            let mut found = Vec::new();
            if let Some(finding) = stop_order(name, written) {
                found.push(finding);
            }
            if let Some(Ok(Resolved::Gradient(gradient))) = animatable::at(element, name, start)
                && gradient.one_colour()
            {
                let colour = gradient
                    .stops()
                    .last()
                    .map(|stop| stop.color.as_str().to_string())
                    .unwrap_or_default();
                let cause = match &gradient {
                    crate::model::Gradient::Radial { radius, .. } if radius.fraction() <= 0.0 => {
                        "its `radius` is 0, which paints the last stop's colour over the box"
                    }
                    _ => "every stop has that colour",
                };
                found.push(
                    Finding::new("R-GRADIENT-ONE-COLOUR")
                        .field("property", json!(name))
                        .field("colour", json!(colour))
                        .field("cause", json!(cause)),
                );
            }
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

/// The first pair of stops whose offsets decrease, as a finding — read off the literal list,
/// so it speaks whether or not the rest of the gradient parses.
fn stop_order(property: &str, gradient: &Value) -> Option<Finding> {
    let stops = gradient.get("stops")?.as_array()?;
    let offsets: Vec<Option<f64>> = stops
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
            // Counted from 1, as a reader counts them.
            .field("stop", json!(index + 1))
            .field("offset", json!(offset))
            .field("previous", json!(index))
            .field("previous_offset", json!(previous)),
    )
}
