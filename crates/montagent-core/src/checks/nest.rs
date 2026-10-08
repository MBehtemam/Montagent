//! The nest checks (prototype #780): `E-NEST-OUTSIDE-WINDOW`, `E-NEST-EMPTY`,
//! `E-NEST-TOO-DEEP`, `E-NEST-TARGET` and the review `R-CLIP-IN-MOVING-NEST`.
//!
//! Walks the written tree through [`crate::nest::nests`] and the flattened leaves through
//! [`crate::nest::composed`] — the same resolver the painter and `query --at` ask.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::nest;
use crate::permissive::Loose;
use crate::report::Report;

/// The depth cap. Raising it later breaks nothing.
pub const CAP: usize = 4;

fn id_of(element: &Value) -> String {
    crate::checks::subject_of(element.get("id").and_then(Value::as_str))
}

fn interval(element: &Value) -> Option<(i64, i64)> {
    Some((element.get("start")?.as_i64()?, element.get("end")?.as_i64()?))
}

/// Every element directly or transitively below `nest`.
fn below(nest_element: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    for track in nest_element["tracks"].as_array().map(Vec::as_slice).unwrap_or_default() {
        for element in track["elements"].as_array().map(Vec::as_slice).unwrap_or_default() {
            out.push(element);
            if nest::is_nest(element) {
                out.extend(below(element));
            }
        }
    }
    out
}

pub fn check(document: &Loose, report: &mut Report) {
    let root = document.value();
    let all = nest::nests(root);
    if all.is_empty() {
        return;
    }
    let mut reported_empty: Vec<String> = Vec::new();
    for (chain, nest_element) in &all {
        let subject = id_of(nest_element);
        let at = |finding: Finding| finding.at_file(document.path()).at_element(&subject);

        // Depth: the chain above plus itself.
        let depth = chain.len() + 1;
        if depth > CAP {
            report.push(at(Finding::new("E-NEST-TOO-DEEP")
                .field("nest", json!(subject))
                .field("depth", json!(depth))
                .field("cap", json!(CAP))));
        }

        // Empty — named at the outermost empty nest only.
        let children = below(nest_element);
        if !children.iter().any(|child| !nest::is_nest(child))
            && !chain.iter().any(|outer| reported_empty.contains(outer))
        {
            reported_empty.push(subject.clone());
            report.push(at(Finding::new("E-NEST-EMPTY").field("nest", json!(subject))));
        }

        // Window: every direct child, by its own timeline interval.
        if let Some((start, end)) = interval(nest_element) {
            for track in nest_element["tracks"].as_array().map(Vec::as_slice).unwrap_or_default() {
                for child in track["elements"].as_array().map(Vec::as_slice).unwrap_or_default() {
                    let Some((cs, ce)) = interval(child) else { continue };
                    if cs < start || ce > end {
                        report.push(at(Finding::new("E-NEST-OUTSIDE-WINDOW")
                            .field("nest", json!(subject))
                            .field("start", json!(start))
                            .field("end", json!(end))
                            .field("child", json!(id_of(child)))
                            .field("child_start", json!(cs))
                            .field("child_end", json!(ce))));
                    }
                }
            }
        }
    }

    // A nest named as an anchor or a transition end.
    let nest_ids: Vec<String> = all.iter().map(|(_, n)| id_of(n)).collect();
    for (_, element) in nest::all_elements(root) {
        let subject = id_of(element);
        let mut named: Vec<(&str, &str)> = Vec::new();
        if let Some(layer) = element.get("layer").and_then(Value::as_object) {
            for (key, role) in [("below", "an anchor (`below`)"), ("above", "an anchor (`above`)")] {
                if let Some(target) = layer.get(key).and_then(Value::as_str) {
                    named.push((target, role));
                }
            }
        }
        for (key, role) in [("from", "a transition's `from`"), ("to", "a transition's `to`")] {
            if let Some(target) = element.get(key).and_then(Value::as_str) {
                named.push((target, role));
            }
        }
        for (target, role) in named {
            if nest_ids.iter().any(|id| id == target) {
                report.push(
                    Finding::new("E-NEST-TARGET")
                        .field("element", json!(subject))
                        .field("nest", json!(target))
                        .field("role", json!(role))
                        .at_file(document.path())
                        .at_element(&subject),
                );
            }
        }
    }

    // R-CLIP-IN-MOVING-NEST: a clipped leaf whose chain is not the identity at some instant.
    for (track, element) in document.elements_in_tracks() {
        let Some(clip) = element.get("clip") else {
            continue;
        };
        if nest::chain(element).is_empty() || clip_spans_the_frame(clip, root) {
            continue;
        }
        let Some((start, end)) = interval(element) else { continue };
        let instant = moving_instant(element, start, end);
        if let Some(instant) = instant {
            let subject = id_of(element);
            let finding = Finding::new("R-CLIP-IN-MOVING-NEST")
                .field("element", json!(subject))
                .field("instant", json!(instant))
                .at_file(document.path())
                .at_element(&subject);
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        }
    }
}

/// The first instant in `[start, end)` at which the element's chain is not the identity,
/// sampled at every chain record instant and at eight even steps.
fn moving_instant(element: &Value, start: i64, end: i64) -> Option<i64> {
    let mut instants: Vec<i64> = (0..8).map(|j| start + (end - start) * j / 8).collect();
    for n in nest::chain(element) {
        for declared in crate::animatable::declared(n) {
            for record in declared.records().into_iter().flatten() {
                if let Some(t) = record.get("t").and_then(Value::as_i64) {
                    instants.push(t.clamp(start, end.max(start + 1) - 1));
                }
            }
        }
    }
    instants.sort_unstable();
    instants.dedup();
    instants
        .into_iter()
        .find(|t| nest::composed(element, (i128::from(*t), 1)).is_some_and(|m| !m.is_identity()))
}

/// A `clip` that contains the whole frame cuts nothing, so it has nothing to leave behind
/// when its child moves. (The phone mock-up writes `[0, 0, 1920, 1080]` on every screen.)
fn clip_spans_the_frame(clip: &Value, root: &Value) -> bool {
    let (Some(clip), Some(frame)) = (clip.as_array(), root.get("frame")) else {
        return false;
    };
    let number = |value: Option<&Value>| value.and_then(Value::as_i64);
    let (Some(x), Some(y), Some(w), Some(h)) = (
        number(clip.first()),
        number(clip.get(1)),
        number(clip.get(2)),
        number(clip.get(3)),
    ) else {
        return false;
    };
    let (Some(fw), Some(fh)) = (number(frame.get("width")), number(frame.get("height"))) else {
        return false;
    };
    x <= 0 && y <= 0 && x + w >= fw && y + h >= fh
}
