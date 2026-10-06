//! `E-NOT-PAINTED-NO-EXTENT`, asked before anything is painted (ADR-0146 §6).
//!
//! An element whose box has no positive size at any frame of its range is never painted.
//! A box that reaches zero only at some frames is a real move — collapsing to nothing — and
//! draws nothing for those frames, as `opacity` 0 does; it is no finding at all.
//!
//! `validate` and `render` decide it through the same function,
//! [`crate::animatable::never_painted`], at the same frame instants, so the two verbs give
//! one answer: `validate` here, and the painter when a `frame` is asked of a document that
//! has not been validated.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// The types whose declared box is drawn geometry. A `text` element's box is a container
/// claim the lines are checked against, never what is drawn (ADR-0135).
const DRAWN_BOX: [&str; 5] = ["rect", "ellipse", "path", "image", "video"];

/// The finding, naming its cause. The cause is the box: a `mask` hiding every frame joins
/// it when effect parameters animate.
pub(crate) fn never_painted() -> Finding {
    Finding::new("E-NOT-PAINTED-NO-EXTENT").field(
        "detail",
        json!(
            "the box \u{2014} its `width` by its `height` \u{2014} has no positive size at any \
             frame in its range"
        ),
    )
}

/// Every element that states a box and never paints it.
pub fn check(document: &Loose, report: &mut Report) {
    let Some(fps) = document.value().get("fps").and_then(Value::as_i64) else {
        return;
    };
    for (track, element) in document.elements_in_tracks() {
        let drawn = element
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| DRAWN_BOX.contains(&kind));
        if !drawn || !crate::animatable::never_painted(element, fps) {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let finding = never_painted()
            .at_file(document.path())
            .at_element(&subject);
        report.push(match track {
            Some(track) => finding.at_track(track),
            None => finding,
        });
    }
}
