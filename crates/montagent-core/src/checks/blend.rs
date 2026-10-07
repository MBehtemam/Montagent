//! `R-BLEND-BACKGROUND-ONLY` — ADR-0147 §3: a non-`normal` element with nothing beneath it.
//!
//! A blend mode reads what is under the element. A light leak with no footage under it
//! blends with the `background` alone, which over the default black draws what `normal`
//! draws (`screen`, `add`) or black (`multiply`). That is the plain mistake this names. A
//! glow that overhangs its subject is ordinary design, so a partial overhang is silent.
//!
//! **Decided at the frame instants `render` paints, from boxes alone.** Every frame of the
//! render that falls inside the element's range, at [`exact::instant_of`]'s floored
//! instant, with keyed values resolved by [`geometry::number`], the reading the painter
//! uses. No frame is painted.
//!
//! - *Beneath* is a visual element lower in the painter's order (ascending resolved layer,
//!   document order within a tie, [`crate::verbs::query::at`]'s order) whose box meets this
//!   element's box, whatever its `opacity` or `blend`.
//! - Boxes are compared as **axis-aligned bounding boxes**, cut to `clip` where the element
//!   carries one. A bounding box contains the real box, so "no bounding boxes meet" proves
//!   "no boxes meet", rotated or not.
//! - **Fires** when, at some frame, no lower bounding box meets the element's. Once per
//!   element, naming its interval and the first such frame's instant.
//! - **Clean** when, at every frame, at least one meeting is between two unrotated boxes,
//!   whose bounding boxes are the boxes themselves.
//! - **Not checked** when it does not fire and at some frame every meeting that keeps it
//!   quiet involves a rotated box: two bounding boxes can meet where the boxes do not. Said
//!   beneath the report's NOT CHECKED sentence, naming the element, because a silent pass
//!   is the failure that hurts an agent trusting `validate`.
//!
//! **Document-only**: it needs no session.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::exact;
use crate::finding::Finding;
use crate::model::{Blend, Origin};
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::{Stack, TimelineRange};
use crate::verbs::query::at::frame_dimensions;
use crate::verbs::query::geometry::{self, clip_rect, covers_the_frame};

const CODE: &str = "R-BLEND-BACKGROUND-ONLY";

/// One visual element with a place in the stack.
struct Placed<'a> {
    id: &'a str,
    track: Option<&'a str>,
    element: &'a Value,
    range: TimelineRange,
    /// The painter's order: resolved layer, then document order.
    order: (i64, usize),
    blend: Blend,
}

/// `R-BLEND-BACKGROUND-ONLY`, over every element whose `blend` is not `normal`.
pub fn check(document: &Loose, report: &mut Report) {
    let Some(frame) = frame_dimensions(document) else {
        return;
    };
    let Some(fps) = document
        .value()
        .get("fps")
        .and_then(Value::as_i64)
        .filter(|fps| *fps > 0)
    else {
        return;
    };
    let Some(extent) = exact::extent(document) else {
        return;
    };
    let stack = Stack::of(document);

    let mut placed: Vec<Placed<'_>> = Vec::new();
    for (index, (track, element)) in document.elements_in_tracks().enumerate() {
        if !covers_the_frame(element.get("type").and_then(Value::as_str)) {
            continue;
        }
        let Some(id) = element.get("id").and_then(Value::as_str) else {
            continue;
        };
        // An element with no resolved layer has no place below or above anything: the
        // anchor and schema checks report it.
        let Ok(layer) = stack.layer_of(id) else {
            continue;
        };
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        let blend = element
            .get("blend")
            .and_then(|value| serde_json::from_value::<Blend>(value.clone()).ok())
            .unwrap_or_default();
        placed.push(Placed {
            id,
            track,
            element,
            range: TimelineRange { start, end },
            order: (layer, index),
            blend,
        });
    }

    for blended in placed.iter().filter(|p| p.blend != Blend::Normal) {
        let lower: Vec<&Placed<'_>> = placed.iter().filter(|p| p.order < blended.order).collect();
        match decide(blended, &lower, frame, fps, extent) {
            Outcome::Clean => {}
            Outcome::Fires(instant) => {
                let mut finding = Finding::new(CODE)
                    .at_file(document.path())
                    .at_element(blended.id)
                    .field("element", json!(subject_of(Some(blended.id))))
                    .field("blend", json!(blended.blend.as_str()))
                    .field("start", json!(blended.range.start))
                    .field("end", json!(blended.range.end))
                    .field("instant", json!(instant));
                if let Some(track) = blended.track {
                    finding = finding.at_track(track);
                }
                report.push(finding);
            }
            Outcome::NotChecked(instant) => report.not_checked(format!(
                "{CODE} on {}: at {instant} ms every box beneath it that meets its bounding \
                 box is rotated, or it is, and bounding boxes alone cannot prove the boxes \
                 meet.",
                blended.id
            )),
        }
    }
}

enum Outcome {
    Clean,
    /// The first painted instant with nothing beneath.
    Fires(i64),
    /// The first painted instant whose only meetings involve a rotated box.
    NotChecked(i64),
}

/// The outcome over every frame `render` paints inside the element's range.
fn decide(
    blended: &Placed<'_>,
    lower: &[&Placed<'_>],
    frame: (i64, i64),
    fps: i64,
    extent: i64,
) -> Outcome {
    let from = blended.range.start.max(0);
    let to = blended.range.end.min(extent);
    let (Some(first), Some(last)) = (
        exact::frame_at_or_after(from, fps),
        exact::frame_before(to, fps),
    ) else {
        return Outcome::Clean;
    };
    let mut unproved: Option<i64> = None;
    for n in first.frame..=last.frame {
        let instant = exact::instant_of(n, fps);
        let Some(own) = bounds(blended.element, instant, frame) else {
            // No readable box, or a `clip` that excludes it: it paints nothing here.
            continue;
        };
        let mut met = false;
        let mut proved = false;
        for other in lower {
            if !(other.range.start <= instant && instant < other.range.end) {
                continue;
            }
            let Some(theirs) = bounds(other.element, instant, frame) else {
                continue;
            };
            if own.meets(&theirs) {
                met = true;
                if !own.rotated && !theirs.rotated {
                    proved = true;
                    break;
                }
            }
        }
        if !met {
            return Outcome::Fires(instant);
        }
        if !proved {
            unproved.get_or_insert(instant);
        }
    }
    match unproved {
        Some(instant) => Outcome::NotChecked(instant),
        None => Outcome::Clean,
    }
}

/// A frame-space axis-aligned bounding box, and whether the box it bounds is rotated.
struct Bounds {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    rotated: bool,
}

impl Bounds {
    /// Whether the two overlap with a positive area: boxes that only touch share no pixel.
    fn meets(&self, other: &Bounds) -> bool {
        self.left.max(other.left) < self.right.min(other.right)
            && self.top.max(other.top) < self.bottom.min(other.bottom)
    }
}

/// The element's bounding box at `instant`: its declared box placed by `x`, `y`, `origin`,
/// `scale` and `rotation` as the painter places it (translate, rotate, scale, about the
/// origin), then cut to `clip`. `None` where it has no readable box or `clip` leaves none.
fn bounds(element: &Value, instant: i64, frame: (i64, i64)) -> Option<Bounds> {
    // A projected element's box is the bounds of its projected quadrilateral (ADR-0167 §6),
    // the one reading every check of a frame-space box shares.
    if crate::projection::projects(element) {
        let placed = crate::projection::placed(element, (i128::from(instant), 1), frame)?;
        let (mut left, mut top, mut right, mut bottom) = placed.footprint.bounds();
        if let Some(clip) = clip_rect(element) {
            left = left.max(clip.x as f64);
            top = top.max(clip.y as f64);
            right = right.min((clip.x + clip.width) as f64);
            bottom = bottom.min((clip.y + clip.height) as f64);
        }
        return (left < right && top < bottom).then_some(Bounds {
            left,
            top,
            right,
            bottom,
            // A quadrilateral is not its bounds, so as for a rotated box, two bounds that
            // meet are no proof that the boxes do.
            rotated: true,
        });
    }
    let width = element.get("width").and_then(Value::as_i64)? as f64;
    let height = element.get("height").and_then(Value::as_i64)? as f64;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let (frame_width, frame_height) = frame;
    let x = geometry::number::<i64>(element, "x", instant, frame_width as f64 / 2.0);
    let y = geometry::number::<i64>(element, "y", instant, frame_height as f64 / 2.0);
    let [sx, sy] = geometry::number::<[f64; 2]>(element, "scale", instant, [1.0, 1.0]);
    let rotation = geometry::number::<f64>(element, "rotation", instant, 0.0);
    let origin = match element.get("origin") {
        None => Origin::Center,
        Some(value) => serde_json::from_value(value.clone()).unwrap_or(Origin::Center),
    };
    let (fx, fy) = geometry::origin_fraction(origin);
    let (sin, cos) = rotation.to_radians().sin_cos();

    let (mut left, mut top, mut right, mut bottom) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (u, v) in [(0.0, 0.0), (width, 0.0), (0.0, height), (width, height)] {
        let (lx, ly) = ((u - fx * width) * sx, (v - fy * height) * sy);
        let (px, py) = (x + lx * cos - ly * sin, y + lx * sin + ly * cos);
        left = left.min(px);
        right = right.max(px);
        top = top.min(py);
        bottom = bottom.max(py);
    }
    if let Some(clip) = clip_rect(element) {
        left = left.max(clip.x as f64);
        top = top.max(clip.y as f64);
        right = right.min((clip.x + clip.width) as f64);
        bottom = bottom.min((clip.y + clip.height) as f64);
    }
    (left < right && top < bottom).then_some(Bounds {
        left,
        top,
        right,
        bottom,
        rotated: rotation != 0.0,
    })
}
