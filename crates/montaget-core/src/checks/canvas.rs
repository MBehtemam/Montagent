//! `R-OFF-CANVAS` — ADR-0044: *"for every visual element, if the element's declared rect,
//! resolved across its entire active timeline range including every keyframe, never
//! intersects the project's current `frame` at any instant, that is a finding."*
//!
//! The evidence is ADR-0012's measurement: retargeting the committed fixture from
//! 1080×1920 to 1920×1080 leaves **32 of 60 elements entirely outside the new frame**, and
//! the 28 survivors are the 20 audio elements (no spatial extent) plus 8 header-chrome
//! elements whose fixed margins happen to still fit. Absolute integer pixels make that
//! defect visible in principle; this check is what makes `validate` say so.
//!
//! ## Standing, not history-triggered
//!
//! ADR-0044 retires the *"frame-change census"* the ticket was named for along with the
//! premise behind it. An off-canvas element is a fact about the **current file** — *"it is
//! exactly as invisible whether a `frame` edit, a hand-edited `x`, or a pasted-in element
//! from another project put it there"* — and scoping the check to "only after a `frame`
//! edit" would require the one thing this format has refused six times over: knowing what
//! just happened to a file.
//!
//! ## Whole-range, not per-instant
//!
//! *"An element that is off-canvas at some instants of its active range — a slide-in
//! starting at `x:-500` and animating to `x:0`, a slide-out doing the reverse — is
//! ordinary, legal animation vocabulary and fires nothing."* The finding fires only when
//! the rect never intersects the frame across the **whole** range. So one sample that
//! meets the frame ends the question for that element, and the finding is what is left
//! when no sample did.
//!
//! **An element whose box never moves is answered exactly**: with no keyframe on `x`, `y`,
//! `scale` or `rotation` the rect is one rectangle for the element's whole life, and
//! ADR-0012's measured defect — 32 of 60 elements left outside a retargeted frame — is
//! entirely that case.
//!
//! An element that *does* move is **sampled**, through [`crate::checks::box_samples`] —
//! every keyframe boundary that moves the box, the range's own ends, and the midpoint of
//! each consecutive pair — shared with [`crate::checks::tie`], which asks the same
//! arithmetic whether two boxes meet each other rather than whether one meets the frame.
//! That is a sample set and not a proof, and *"never"* is the one shape a sample set
//! cannot establish: a box that crosses the frame and leaves again strictly between two
//! samples would be reported as never having met it. ADR-0044 licenses no sampling — it
//! says *"at any instant"* — so this is the module's reading, taken because the
//! alternative (a closed-form intersection over an eased bezier segment) is arithmetic no
//! ADR specifies and the reported union would still be the union of what was measured.
//! Raised as [#285](https://github.com/MBehtemam/Montaget/issues/285) rather than left to
//! be discovered from the behaviour.
//!
//! ## The *declared* rect
//!
//! ADR-0044 says the declared rect, and this module reads exactly that:
//! [`geometry::drawn_rect`] — `x`, `y`, `origin`, the declared `width`×`height`, and the
//! resolved `scale`. **`clip` is deliberately not applied**, though [`crate::checks::tie`]
//! does apply it: there, the question is which pixels one element can paint *onto another*,
//! and an aperture that excludes a box excludes the collision with it. Here the question is
//! whether the author has placed an element outside the canvas, and an element whose
//! declared rect is on-canvas has been placed on it — whatever a `clip` then shows of it.
//! Reading `clip` would also make the check fire on an aperture the author deliberately
//! closed, which is a different defect with a different fix.
//!
//! ## Rotation
//!
//! A rotated element's footprint is a parallelogram and [`geometry::drawn_rect`] refuses
//! rather than approximate one ([`geometry::NotAxisAligned`], the refusal `query --at`
//! already makes for the same reason). What is left is the **unrotated** rect, which
//! neither contains the footprint nor is contained by it — rotation is about the origin,
//! so the footprint reaches outside the box on some corners and inside it on others — so
//! it can neither prove nor disprove that the frame is met. The element is passed over.
//!
//! Silence rather than an `unchecked` finding, and the difference from
//! `U-LAYER-TIE-ROTATED` is what the two checks gate: `E-LAYER-TIE` refuses a render, so
//! a reader is owed an explicit *"I could not look"* where it declined to; this check
//! gates nothing, and ADR-0044's own reasoning is that a finding nobody can act on is the
//! alarm fatigue ADR-0006 calls a safety problem. It does cost coverage — ADR-0044 says
//! *"the finding applies uniformly to every visual element type"* — so it too is
//! [#285](https://github.com/MBehtemam/Montaget/issues/285) and not this module's to
//! settle.
//!
//! **Document-only**: `frame`, element geometry and keyframes are all the check reads —
//! *"no new field"*, and no disk.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::TimelineRange;
use crate::verbs::query::at::frame_dimensions;
use crate::verbs::query::geometry::{self, Rect};

/// `R-OFF-CANVAS`, over every visual element in the document.
pub fn check(document: &Loose, report: &mut Report) {
    // Without a legal `frame` there is no canvas for anything to be off. The schema check
    // owns saying so.
    let Some((frame_width, frame_height)) = frame_dimensions(document) else {
        return;
    };
    let canvas = Rect {
        x: 0,
        y: 0,
        width: frame_width,
        height: frame_height,
    };

    for (track, element) in document.elements_in_tracks() {
        // Audio has no spatial extent and a `transition` paints nothing of its own
        // (ADR-0059), so neither can be off a canvas it never reaches.
        if !geometry::covers_the_frame(element.get("type").and_then(Value::as_str)) {
            continue;
        }
        let Some(range) = active_range(element) else {
            // A malformed or empty range is the schema check's fact, and an element with
            // no instants has none at which it is off canvas.
            continue;
        };
        let Some(union) =
            union_if_it_never_meets(element, range, (frame_width, frame_height), canvas)
        else {
            continue;
        };

        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let finding = Finding::new("R-OFF-CANVAS")
            .at_file(document.path())
            .at_element(&subject)
            .field("element", json!(subject))
            .field("start", json!(range.start))
            .field("end", json!(range.end))
            // Every number inline (ADR-0006): the rect the element actually occupies over
            // its whole range, against the frame it never reaches.
            .field("x", json!(union.x))
            .field("y", json!(union.y))
            .field("width", json!(union.width))
            .field("height", json!(union.height))
            .field("frame_width", json!(frame_width))
            .field("frame_height", json!(frame_height));
        report.push(match track {
            Some(track) => finding.at_track(track),
            None => finding,
        });
    }
}

/// The element's own half-open timeline range, where it states one in integer
/// milliseconds.
fn active_range(element: &Value) -> Option<TimelineRange> {
    let start = element.get("start")?.as_i64()?;
    let end = element.get("end")?.as_i64()?;
    (end > start).then_some(TimelineRange { start, end })
}

/// **The union of this element's sampled rects, if none of them meets the frame** —
/// `None` where one does, or where the question could not be asked at all.
///
/// `None` on the first sample that intersects, because one instant on screen is the whole
/// of what ADR-0044's whole-range trigger requires to stay silent. `None` too where the
/// element rotates (a parallelogram this module does not measure) or states no readable
/// `width`/`height` (the schema check's finding, not this one's) — in both, nothing was
/// measured, and a union of nothing is not a rect to report.
fn union_if_it_never_meets(
    element: &Value,
    range: TimelineRange,
    frame: (i64, i64),
    canvas: Rect,
) -> Option<Rect> {
    let mut union: Option<Rect> = None;
    for instant in crate::checks::box_samples(&[element], range) {
        let rect = geometry::drawn_rect(element, instant, frame)?.ok()?;
        if rect.intersect(canvas).is_some() {
            return None;
        }
        union = Some(match union {
            None => rect,
            Some(so_far) => bounding(so_far, rect),
        });
    }
    union
}

/// The smallest rectangle containing both.
fn bounding(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let right = (a.x + a.width).max(b.x + b.width);
    let bottom = (a.y + a.height).max(b.y + b.height);
    Rect {
        x,
        y,
        width: right - x,
        height: bottom - y,
    }
}
