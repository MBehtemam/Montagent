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
//! **Document-only**: every field this needs — `start`, `end`, `from`, `to`, and the two
//! bridged elements' own ranges — is already in the document, so this needs no session and
//! can sit anywhere in `validate`'s check list. The two bridged elements' ranges are read
//! through [`crate::stack::Stack`] — the same index [`crate::checks::anchor`] already
//! builds for exactly this id-to-range lookup — rather than a second, independent one kept
//! in step with it by hand.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::Finding;
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

    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("transition") {
            continue;
        }
        let findings = match references(element, &visual, &sounding) {
            Some(findings) => findings,
            None => candidate(element, &stack)
                .into_iter()
                .chain(slide_under(element, &stack))
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
