//! `E-TRANSITION-RANGE` and `E-TRANSITION-NO-OVERLAP` — ADR-0059: *"a transition's
//! `start`/`end` must exactly equal the intersection of the two elements it bridges."*
//! Outside that window only one of the two bridged elements exists, so a wider or
//! narrower declared range is a field the renderer cannot honour.
//!
//! The check ADR-0059 itself names: *"`validate` can therefore check transition bounds as
//! a closed-form function of the two referenced elements' own ranges — drift on either
//! side surfaces immediately as an error."* Because the correct value is exactly that
//! closed-form function of fields already in the document, drift is advise-class: the
//! repair is the recomputed intersection, not a request for the author's intent.
//!
//! **That closed-form function has no domain when `from` and `to` never coexist.**
//! `max(start)..min(end)` on two ranges that do not overlap collapses to an empty or
//! inverted pair, and a transition that happens to declare exactly that pair would
//! otherwise validate clean despite bridging nothing a crossfade could ever show. Reported
//! separately as `E-TRANSITION-NO-OVERLAP`, refuse-class — the document does not say
//! whether the transition should move, one of the bridged elements should, or the
//! transition should not exist at all, so there is no single recomputed value to advise,
//! on [`crate::checks::track`]'s `E-TRACK-OVERLAP` reasoning. Its own code rather than a
//! branch inside `E-TRANSITION-RANGE`: ADR-0043's uniformity rule fixes one repair class
//! per code, and the two outcomes here are determined differently.
//!
//! **The references themselves** (ADR-0150). A `from`/`to` naming no visual element is
//! `E-TRANSITION-REF-MISSING`, and one naming the same element twice is
//! `E-TRANSITION-REF-SELF` — the anchor's `E-ANCHOR-MISSING` and `E-ANCHOR-SELF`, one
//! reference over. Until ADR-0150 only `frame` and `render` caught a dangling reference, as
//! `E-NOT-PAINTED-UNRESOLVED-REF`, which stays for `frame`, since `frame` runs without
//! `validate`. Where either fires, the range checks above stay quiet: one fact, one code.
//!
//! **A slide's stacking** (ADR-0150). Only `to` moves in a `slide`, over a still `from`, so
//! `to` must paint above `from`. `E-TRANSITION-SLIDE-UNDER` fires where `to`'s resolved
//! layer is below `from`'s. Refuse-class: raising `to`, lowering `from`, or swapping which
//! element each track holds all fix it, and the document does not say which was meant. Two
//! elements at one layer are the layer-tie check's question, not this one's
//! ([`crate::checks::tie`]). A `wipe` or `push` works in either order, and never fires.
//!
//! **The sound** (ADR-0176). An `audio_crossfade` names `audio` or `video` elements rather
//! than visual ones, and `E-TRANSITION-REF-MISSING`'s text says which. `R-TRANSITION-VOLUME-STACK`
//! and `R-TRANSITION-AUDIO-SILENT` read the document alone; `E-TRANSITION-AUDIO-NO-STREAM` and
//! `R-TRANSITION-AUDIO-UNSET` need to know whether a source carries an audio stream, so they run
//! with the disk checks ([`on_disk`]). Where a reference error fires they stay quiet.
//!
//! **Document-only**: every field this needs — `start`, `end`, `from`, `to`, and the two
//! bridged elements' own ranges — is already in the document, so this needs no session and
//! can sit anywhere in `validate`'s check list. The two bridged elements' ranges are read
//! through [`crate::stack::Stack`] — the same index [`crate::checks::anchor`] already
//! builds for exactly this id-to-range lookup — rather than a second, independent one kept
//! in step with it by hand.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
use crate::media::Source;
use crate::media::session::Session;
use crate::media::tools::Missing;
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::{Stack, TimelineRange};

/// Every transition check, over every `transition` element in the document.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();
    let stack = Stack::of(document);
    let visual: Vec<&str> = document
        .elements_in_tracks()
        .filter(|(_, element)| {
            crate::verbs::query::geometry::covers_the_frame(
                element.get("type").and_then(Value::as_str),
            )
        })
        .filter_map(|(_, element)| element.get("id").and_then(Value::as_str))
        .collect();

    // ADR-0176: an `audio_crossfade` crossfades sound, so it names an `audio` or a `video`
    // (whose embedded audio is crossfaded); the picture kinds keep refusing an `audio`.
    let sounding: Vec<&str> = document
        .elements_in_tracks()
        .filter(|(_, element)| {
            matches!(
                element.get("type").and_then(Value::as_str),
                Some("audio" | "video")
            )
        })
        .filter_map(|(_, element)| element.get("id").and_then(Value::as_str))
        .collect();

    let by_id = elements_by_id(document);

    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("transition") {
            continue;
        }
        let findings = match references(element, &visual, &sounding) {
            Some(findings) => findings,
            None => candidate(element, &stack)
                .into_iter()
                .chain(slide_under(element, &stack))
                .chain(sound_reviews(element, &by_id))
                .collect(),
        };

        for finding in findings {
            let mut finding = finding.at_file(file);
            if let Some(track) = track {
                finding = finding.at_track(track);
            }
            report.push(finding);
        }
    }
}

/// `E-TRANSITION-REF-MISSING` and `E-TRANSITION-REF-SELF`: `Some` where the two references
/// do not name two distinct visual elements, carrying what to say about it.
fn references(element: &Value, visual: &[&str], sounding: &[&str]) -> Option<Vec<Finding>> {
    let audio_only = element.get("kind").and_then(Value::as_str) == Some("audio_crossfade");
    let (accepted, accepts) = match audio_only {
        true => (sounding, "an audio or video element"),
        false => (
            visual,
            "a visual element (image, video, text, rect, ellipse or path)",
        ),
    };
    let subject = subject_of(element.get("id").and_then(Value::as_str));
    let from = element.get("from").and_then(Value::as_str);
    let to = element.get("to").and_then(Value::as_str);

    let missing: Vec<Finding> = [("from", from), ("to", to)]
        .into_iter()
        .filter_map(|(side, target)| {
            let target = target?;
            (!accepted.contains(&target)).then(|| {
                Finding::new("E-TRANSITION-REF-MISSING")
                    .at_element(subject.clone())
                    .field("element", json!(subject))
                    .field("side", json!(side))
                    .field("target", json!(target))
                    .field("accepts", json!(accepts))
                    .repair_value(json!({
                        "value": format!(
                            "set `{side}` to the id of {accepts} in the project"
                        )
                    }))
            })
        })
        .collect();
    if !missing.is_empty() {
        return Some(missing);
    }

    match (from, to) {
        (Some(from), Some(to)) if from == to => Some(vec![
            Finding::new("E-TRANSITION-REF-SELF")
                .at_element(subject.clone())
                .field("element", json!(subject))
                .field("target", json!(to))
                .repair_value(json!({
                    "value": "set `to` to the element the transition hands over to, which \
                              is not the one it leaves"
                })),
        ]),
        _ => None,
    }
}

/// `E-TRANSITION-SLIDE-UNDER`: a `slide` whose `to` resolves to a lower layer than its
/// `from`. `None` wherever either layer does not resolve — the anchor checks' to report.
fn slide_under(element: &Value, stack: &Stack<'_>) -> Option<Finding> {
    if element.get("kind").and_then(Value::as_str) != Some("slide") {
        return None;
    }
    let from = element.get("from").and_then(Value::as_str)?;
    let to = element.get("to").and_then(Value::as_str)?;
    let from_layer = stack.layer_of(from).ok()?;
    let to_layer = stack.layer_of(to).ok()?;
    if to_layer >= from_layer {
        return None;
    }
    let subject = subject_of(element.get("id").and_then(Value::as_str));
    Some(
        Finding::new("E-TRANSITION-SLIDE-UNDER")
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("from", json!(from))
            .field("to", json!(to))
            .field("from_layer", json!(from_layer))
            .field("to_layer", json!(to_layer)),
    )
}

/// This transition's finding, if its declared range has drifted from the intersection of
/// its two bridged elements, or if those two elements never overlap at all — `None` if
/// any field this check needs is absent or unresolvable (another check's business to
/// name) or if a real window exists and the declared range matches it.
fn candidate(element: &Value, stack: &Stack<'_>) -> Option<Finding> {
    let own_start = element.get("start").and_then(Value::as_i64)?;
    let own_end = element.get("end").and_then(Value::as_i64)?;
    let from = element.get("from").and_then(Value::as_str)?;
    let to = element.get("to").and_then(Value::as_str)?;

    let from_range = stack.placement(from)?.range?;
    let to_range = stack.placement(to)?.range?;

    if !from_range.overlaps(to_range) {
        return Some(no_overlap(
            element.get("id").and_then(Value::as_str),
            own_start,
            own_end,
            from,
            to,
            from_range,
            to_range,
        ));
    }

    let derived_start = from_range.start.max(to_range.start);
    let derived_end = from_range.end.min(to_range.end);

    if own_start == derived_start && own_end == derived_end {
        return None;
    }

    let subject = subject_of(element.get("id").and_then(Value::as_str));
    Some(
        Finding::new("E-TRANSITION-RANGE")
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("start", json!(own_start))
            .field("end", json!(own_end))
            .field("from", json!(from))
            .field("to", json!(to))
            .field("derived_start", json!(derived_start))
            .field("derived_end", json!(derived_end))
            .repair_value(json!({
                "value": format!(
                    "set `start` to {derived_start} and `end` to {derived_end} — the \
                     intersection of `{from}` and `{to}`"
                )
            })),
    )
}

/// `E-TRANSITION-NO-OVERLAP`: the bridged pair share no instant, so no declared range —
/// including whatever this element happens to state — is ever correct. Fires
/// unconditionally, unlike `E-TRANSITION-RANGE`, because there is no matching value it
/// could accept.
fn no_overlap(
    id: Option<&str>,
    own_start: i64,
    own_end: i64,
    from: &str,
    to: &str,
    from_range: TimelineRange,
    to_range: TimelineRange,
) -> Finding {
    let subject = subject_of(id);
    Finding::new("E-TRANSITION-NO-OVERLAP")
        .at_element(subject.clone())
        .field("element", json!(subject))
        .field("start", json!(own_start))
        .field("end", json!(own_end))
        .field("from", json!(from))
        .field("to", json!(to))
        .field("from_start", json!(from_range.start))
        .field("from_end", json!(from_range.end))
        .field("to_start", json!(to_range.start))
        .field("to_end", json!(to_range.end))
}

/// Every element with an `id`, by that id.
fn elements_by_id(document: &Loose) -> std::collections::HashMap<&str, &Value> {
    document
        .elements_in_tracks()
        .filter_map(|(_, element)| Some((element.get("id").and_then(Value::as_str)?, element)))
        .collect()
}

fn is_sounding(element: &Value) -> bool {
    matches!(
        element.get("type").and_then(Value::as_str),
        Some("audio" | "video")
    )
}

/// Both bridged elements, where each names a distinct element that can carry sound. The
/// reference errors above are the answer for anything else: one fact, one code.
fn sounding_sides<'a>(
    element: &'a Value,
    by_id: &std::collections::HashMap<&str, &'a Value>,
) -> Option<[(&'static str, &'a str, &'a Value); 2]> {
    let from = element.get("from").and_then(Value::as_str)?;
    let to = element.get("to").and_then(Value::as_str)?;
    if from == to {
        return None;
    }
    let (from_element, to_element) = (*by_id.get(from)?, *by_id.get(to)?);
    // A picture kind names visual elements, so of the elements that sound only a `video`
    // is one; an `audio` there is `E-TRANSITION-REF-MISSING`'s, and this stays quiet.
    let audio_only = element.get("kind").and_then(Value::as_str) == Some("audio_crossfade");
    let accepted = |side: &Value| {
        is_sounding(side)
            && (audio_only || side.get("type").and_then(Value::as_str) == Some("video"))
    };
    (accepted(from_element) && accepted(to_element))
        .then_some([("from", from, from_element), ("to", to, to_element)])
}

/// Is the element's `volume` the literal `0` for its whole length? A keyframe list of only
/// zeros is the same silence. A volume that is not a number or a list is another check's.
fn constant_zero_volume(element: &Value) -> bool {
    match element.get("volume") {
        Some(Value::Array(keyframes)) => {
            !keyframes.is_empty()
                && keyframes
                    .iter()
                    .all(|keyframe| keyframe.get("v").and_then(Value::as_f64) == Some(0.0))
        }
        Some(number) => number.as_f64() == Some(0.0),
        None => false,
    }
}

/// `R-TRANSITION-VOLUME-STACK` and `R-TRANSITION-AUDIO-SILENT` (ADR-0176 §5): the two
/// reviews the document alone can state.
fn sound_reviews(element: &Value, by_id: &std::collections::HashMap<&str, &Value>) -> Vec<Finding> {
    let Some(sides) = sounding_sides(element, by_id) else {
        return Vec::new();
    };
    let subject = subject_of(element.get("id").and_then(Value::as_str));
    let mut findings = Vec::new();
    for (side, target, bridged) in sides {
        let silent = if constant_zero_volume(bridged) {
            Some("a constant `volume: 0`")
        } else if crate::remap::is_remapped(bridged) {
            Some("a speed-ramped `video`, which carries `volume: 0`")
        } else {
            None
        };
        if let Some(reason) = silent {
            findings.push(
                Finding::new("R-TRANSITION-AUDIO-SILENT")
                    .at_element(subject.clone())
                    .field("element", json!(subject))
                    .field("side", json!(side))
                    .field("target", json!(target))
                    .field("reason", json!(reason)),
            );
        }
        if let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) {
            let times = level_changes_inside(bridged, start, end);
            if !times.is_empty() {
                let list = times
                    .iter()
                    .map(|t| format!("t={t}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                findings.push(
                    Finding::new("R-TRANSITION-VOLUME-STACK")
                        .at_element(subject.clone())
                        .field("element", json!(subject))
                        .field("side", json!(side))
                        .field("target", json!(target))
                        .field("start", json!(start))
                        .field("end", json!(end))
                        .field("keyframe_times", json!(times))
                        .field("keyframes", json!(format!("keyframes at {list}"))),
                );
            }
        }
    }
    findings
}

/// The times of the keyframes that make `volume` change level inside `(start, end)`: a
/// keyframe strictly inside, or the two either side of a segment that straddles the window
/// with differing `v`. A flat volume across the window is not stacking.
fn level_changes_inside(element: &Value, start: i64, end: i64) -> Vec<i64> {
    let Some(Value::Array(keyframes)) = element.get("volume") else {
        return Vec::new();
    };
    let mut points: Vec<(i64, f64)> = keyframes
        .iter()
        .filter_map(|keyframe| {
            Some((
                keyframe.get("t").and_then(Value::as_i64)?,
                keyframe.get("v").and_then(Value::as_f64)?,
            ))
        })
        .collect();
    points.sort_by_key(|(t, _)| *t);
    let mut times: Vec<i64> = points
        .iter()
        .filter(|(t, _)| *t > start && *t < end)
        .map(|(t, _)| *t)
        .collect();
    for pair in points.windows(2) {
        let ((t0, v0), (t1, v1)) = (pair[0], pair[1]);
        if t0 <= start && t1 >= end && v0 != v1 {
            times.extend([t0, t1]);
        }
    }
    times.sort_unstable();
    times.dedup();
    times
}

/// `E-TRANSITION-AUDIO-NO-STREAM` and `R-TRANSITION-AUDIO-UNSET` (ADR-0176 §4, §5): the two
/// findings that need to know whether a source carries an audio stream, off a probe the
/// source check has already cached.
pub fn on_disk(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    let file = document.path().to_string();
    let base = crate::checks::project_dir(document);
    let by_id = elements_by_id(document);

    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("transition") {
            continue;
        }
        let Some(sides) = sounding_sides(element, &by_id) else {
            continue;
        };
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        let kind = element.get("kind").and_then(Value::as_str).unwrap_or("");

        // Whether each side's source is known to carry an audio stream: `None` where nothing
        // was established, which the source check reports as its own finding.
        let mut carries = [None, None];
        for (k, (_, _, side)) in sides.iter().enumerate() {
            let Some(source) = side.get("source").and_then(Value::as_str) else {
                continue;
            };
            let outcome = session.probe(&Source::resolve(source, &base))?;
            carries[k] = outcome.probe().map(|probe| probe.audio.is_some());
        }

        let mut findings = Vec::new();
        if kind == "audio_crossfade" {
            for (k, (side, target, bridged)) in sides.iter().enumerate() {
                if carries[k] == Some(false) {
                    findings.push(
                        Finding::new("E-TRANSITION-AUDIO-NO-STREAM")
                            .at_element(subject.clone())
                            .field("element", json!(subject))
                            .field("side", json!(side))
                            .field("target", json!(target))
                            .field(
                                "source",
                                json!(bridged.get("source").and_then(Value::as_str)),
                            ),
                    );
                }
            }
        } else if element.get("audio").is_none()
            && carries == [Some(true), Some(true)]
            && !sides.iter().any(|(_, _, side)| constant_zero_volume(side))
        {
            findings.push(
                Finding::new("R-TRANSITION-AUDIO-UNSET")
                    .at_element(subject.clone())
                    .field("element", json!(subject))
                    .field("kind", json!(kind))
                    .field("from", json!(sides[0].1))
                    .field("to", json!(sides[1].1)),
            );
        }

        for finding in findings {
            let mut finding = finding.at_file(&file);
            if let Some(track) = track {
                finding = finding.at_track(track);
            }
            report.push(finding);
        }
    }
    Ok(())
}
