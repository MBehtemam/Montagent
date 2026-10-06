//! `E-NOT-PAINTED-NO-EXTENT`, asked before anything is painted (ADR-0146 §6).
//!
//! An element whose box has no positive size at any frame of its range is never painted, and
//! so is one that a plain `mask` of no positive size hides at every frame its box does not.
//! A box or a mask that reaches zero only at some frames is a real move — collapsing to
//! nothing, or a reveal starting from it — and draws nothing for those frames, as `opacity`
//! 0 does; it is no finding at all. A mask that is non-zero but lies wholly outside the
//! element is not counted.
//!
//! `validate` and `render` decide it through the same function,
//! [`crate::animatable::never_painted`], at the same frame instants, so the two verbs give
//! one answer: `validate` here, and the painter when a `frame` is asked of a document that
//! has not been validated.

use serde_json::{Value, json};

use crate::animatable::Hidden;
use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// The types that draw something: a declared box that is drawn geometry, or a `text`
/// element's lines. A `text` element's box is a container claim the lines are checked
/// against, never what is drawn (ADR-0135), so only a `mask` can hide one
/// ([`crate::animatable::hidden_at`]).
const DRAWN: [&str; 6] = ["rect", "ellipse", "path", "image", "video", "text"];

/// The finding, naming its cause (ADR-0146 §6): the box, a `mask` that hides the element, or
/// each of them at some of the frames.
pub(crate) fn never_painted(causes: &[Hidden]) -> Finding {
    let named: Vec<String> = causes
        .iter()
        .map(|cause| match cause {
            Hidden::Box => "the box \u{2014} its `width` by its `height` \u{2014} has no \
                            positive size"
                .to_string(),
            Hidden::Mask(index) => format!(
                "the `mask` at `effects[{index}]` has no positive `width` by `height`, so it \
                 hides the whole element"
            ),
        })
        .collect();
    let detail = match named.as_slice() {
        [one] => format!("{one} at any frame in its range"),
        _ => format!(
            "at every frame in its range one of these holds: {}",
            named.join("; ")
        ),
    };
    Finding::new("E-NOT-PAINTED-NO-EXTENT").field("detail", json!(detail))
}

/// Every element that draws nothing at any frame of its range.
pub fn check(document: &Loose, report: &mut Report) {
    let Some(fps) = document.value().get("fps").and_then(Value::as_i64) else {
        return;
    };
    for (track, element) in document.elements_in_tracks() {
        let drawn = element
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| DRAWN.contains(&kind));
        if !drawn {
            continue;
        }
        let Some(causes) = crate::animatable::never_painted(element, fps) else {
            continue;
        };
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let finding = never_painted(&causes)
            .at_file(document.path())
            .at_element(&subject);
        report.push(match track {
            Some(track) => finding.at_track(track),
            None => finding,
        });
    }
}
