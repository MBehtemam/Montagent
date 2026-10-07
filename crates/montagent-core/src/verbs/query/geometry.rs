//! The three components of `--at`'s output that reach outside the document: the crop
//! rectangle, the ink box, and `NOT COVERED` (#210, ADR-0011). The fourth — the offset
//! into the source — is [`super::at::source_offset`], which needs none of this module's
//! machinery and lives beside the field it fills.
//!
//! Every function here computes the frame-space rectangle an element actually occupies —
//! `x`, `y`, `origin`, declared `width`/`height`, and the resolved `scale` — because the
//! crop rectangle, the ink box and `NOT COVERED` are three questions about the same
//! geometry, not three separate ones. A text element's declared `width`/`height` is the one
//! exception: the ink box places the block the lines make instead, as the painter does
//! (ADR-0135). **Rotation refuses rather than approximates**: a
//! rotated element's on-screen footprint is not a rectangle, and reporting one anyway
//! would be exactly the plausible-and-wrong number ADR-0011's resolver refuses to invent
//! for a keyframe list with no `ease` (see [`crate::resolve::Unresolvable`]). No fixture
//! element rotates, so this costs nothing today and is recorded rather than silently
//! narrowed.

use serde::Serialize;
use serde_json::Value;

use crate::model::{Animatable, Origin};
use crate::permissive::Loose;
use crate::resolve::{self, Interpolate};
use crate::verbs::measure::{Measurable, align_of, register, runs_of};

/// An axis-aligned frame-space rectangle, in absolute integer pixels — the same unit
/// every other frame-space number in the format is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

impl Rect {
    fn right(self) -> i64 {
        self.x + self.width
    }

    fn bottom(self) -> i64 {
        self.y + self.height
    }

    /// The overlap of two rectangles, or `None` where they do not overlap at all — an
    /// empty intersection is not a zero-size rectangle, because a caller asking "is this
    /// element visible through `clip`" needs the two told apart.
    pub fn intersect(self, other: Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        (right > x && bottom > y).then_some(Rect {
            x,
            y,
            width: right - x,
            height: bottom - y,
        })
    }
}

/// Why this element's on-screen footprint could not be computed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotAxisAligned {
    /// A non-zero resolved `rotation`. Frame-space coverage under a rotation is a
    /// parallelogram, not a rectangle, and this module answers in rectangles only.
    Rotated(f64),
}

/// This element's own drawn rectangle in frame space — `x`/`y`/`origin` placing the
/// declared `width`×`height` box, scaled by the resolved `scale`, both at `instant`.
///
/// **Defaults apply here**, unlike [`super::at::values`]: that view answers *what the
/// document declares*, and this answers *where the element actually is*, which needs the
/// same defaults the renderer would use — `x`/`y` at the frame's centre, `scale` at
/// `[1,1]`, `origin` at `center` (ADR-0012).
///
/// `None` where `width`/`height` cannot be read, or resolve at or below zero at `instant` —
/// not this module's fact to report; `validate`'s schema check already does.
pub fn drawn_rect(
    element: &Value,
    instant: i64,
    frame: (i64, i64),
) -> Option<Result<Rect, NotAxisAligned>> {
    // A shape's box may be keyed (ADR-0146), so it is resolved like every other animatable
    // property rather than read as one integer; at or below zero it occupies nothing.
    let (width, height) = crate::animatable::painted_box(element, i128::from(instant), 1)?;

    let rotation = number::<f64>(element, "rotation", instant, 0.0);
    if rotation != 0.0 {
        return Some(Err(NotAxisAligned::Rotated(rotation)));
    }

    let (frame_width, frame_height) = frame;
    let x = number::<i64>(element, "x", instant, frame_width as f64 / 2.0);
    let y = number::<i64>(element, "y", instant, frame_height as f64 / 2.0);
    let [scale_x, scale_y] = number::<[f64; 2]>(element, "scale", instant, [1.0, 1.0]);

    let origin = match element.get("origin") {
        None | Some(Value::Null) => Origin::Center,
        Some(value) => serde_json::from_value(value.clone()).ok()?,
    };
    let (fx, fy) = origin_fraction(origin);

    let scaled_width = width * scale_x;
    let scaled_height = height * scale_y;
    let left = x - fx * scaled_width;
    let top = y - fy * scaled_height;

    Some(Ok(Rect {
        x: left.round() as i64,
        y: top.round() as i64,
        width: scaled_width.round() as i64,
        height: scaled_height.round() as i64,
    }))
}

/// One property, resolved at `instant` with the renderer's own default where the element
/// does not declare it — the reading [`drawn_rect`]'s doc argues for.
///
/// `pub(crate)` because the renderer needs exactly this reading: `frame` paints *where the
/// element actually is*, which is the same question this module answers in rectangles. A
/// second copy in the render path would be a picture and a caption that could disagree
/// about what an element's resolved `x` is.
pub(crate) fn number<T>(element: &Value, key: &str, instant: i64, default: T::Out) -> T::Out
where
    T: serde::de::DeserializeOwned + Interpolate,
    T::Out: Copy,
{
    number_at::<T>(element, key, (i128::from(instant), 1), default)
}

/// [`number`] at `t = (numerator, denominator)` ms, an instant that need not be a whole
/// millisecond: a motion-blur sample's (ADR-0155 §3). [`number`] is this at `(instant, 1)`.
pub(crate) fn number_at<T>(element: &Value, key: &str, t: (i128, i128), default: T::Out) -> T::Out
where
    T: serde::de::DeserializeOwned + Interpolate,
    T::Out: Copy,
{
    let Some(written) = element.get(key) else {
        return default;
    };
    let Ok(animatable) = serde_json::from_value::<Animatable<T>>(written.clone()) else {
        return default;
    };
    resolve::at_instant(&animatable, t.0, t.1).unwrap_or(default)
}

/// `origin`'s two-way point, as `(horizontal, vertical)` fractions of the box — `0` at
/// left/top, `0.5` at centre, `1` at right/bottom.
///
/// `pub(crate)` for the renderer's sake: the nine keywords are spelled once (ADR-0013), and
/// the frame the agent looks at must be placed by the same nine numbers the `query --at`
/// block beside it was computed from.
pub(crate) fn origin_fraction(origin: Origin) -> (f64, f64) {
    let (h, v) = match origin {
        Origin::TopLeft => (0.0, 0.0),
        Origin::TopCenter => (0.5, 0.0),
        Origin::TopRight => (1.0, 0.0),
        Origin::CenterLeft => (0.0, 0.5),
        Origin::Center => (0.5, 0.5),
        Origin::CenterRight => (1.0, 0.5),
        Origin::BottomLeft => (0.0, 1.0),
        Origin::BottomCenter => (0.5, 1.0),
        Origin::BottomRight => (1.0, 1.0),
    };
    (h, v)
}

/// Whether this element type has a frame-space footprint at all — audio and `transition`
/// do not, and are excluded from both `NOT COVERED`'s union and its refusal: a rotated
/// `rect` blocks the computation, but an audio element playing underneath never could.
///
/// `pub(crate)` for the same reason the rest of this module is: the layer-tie check
/// (ADR-0060) asks the identical question about the identical set of types, and a second
/// list would be a second answer to *"does this element claim pixels"* — the drift this
/// module exists to prevent.
pub(crate) fn covers_the_frame(kind: Option<&str>) -> bool {
    matches!(
        kind,
        Some("image" | "video" | "text" | "rect" | "ellipse" | "path")
    )
}

/// `clip`, as a [`Rect`] — only `image` and `video` carry the field.
pub(crate) fn clip_rect(element: &Value) -> Option<Rect> {
    let clip = element.get("clip")?.as_array()?;
    if clip.len() != 4 {
        return None;
    }
    Some(Rect {
        x: clip[0].as_i64()?,
        y: clip[1].as_i64()?,
        width: clip[2].as_i64()?,
        height: clip[3].as_i64()?,
    })
}

/// **The visible rectangle**: [`drawn_rect`] cut down by `clip` where the element carries
/// one — the frame-space pixels this element can actually paint at `instant`.
///
/// `None` where the element has no rectangle to speak of (no readable `width`/`height`)
/// or where `clip` excludes it entirely; `Err` where it rotates, which this module
/// answers in rectangles only.
pub(crate) fn visible_rect(
    element: &Value,
    instant: i64,
    frame: (i64, i64),
) -> Option<Result<Rect, NotAxisAligned>> {
    bridged_visible_rect(
        element,
        instant,
        frame,
        &crate::transition::Bridge::default(),
    )
}

/// [`visible_rect`] under what the running transitions do to the element (ADR-0150): its
/// drawn rectangle moved by a slide or push's offset, and its `clip`, moved with it, cut by
/// a wipe. The same [`crate::transition::Bridge`] `frame`'s painter paints through.
pub(crate) fn bridged_visible_rect(
    element: &Value,
    instant: i64,
    frame: (i64, i64),
    bridge: &crate::transition::Bridge,
) -> Option<Result<Rect, NotAxisAligned>> {
    match drawn_rect(element, instant, frame)? {
        Err(not_axis_aligned) => Some(Err(not_axis_aligned)),
        Ok(rect) => {
            let moved = Rect {
                x: rect.x + bridge.offset.0,
                y: rect.y + bridge.offset.1,
                ..rect
            };
            match bridge.aperture(clip_rect(element)) {
                Some(aperture) => moved.intersect(aperture).map(Ok),
                None => Some(Ok(moved)),
            }
        }
    }
}

/// **The crop rectangle**: which part of the *source file's own pixels* survive onto the
/// screen, in source pixel space.
///
/// ADR-0013 settled that the source is resampled to exactly the declared `width`×`height`
/// — never cropped in the sense `gravity` meant — so every source pixel maps linearly onto
/// that box before any transform or `clip` is applied. What crops the source is `clip`:
/// the static frame-space aperture can show less of the drawn rectangle than the full box,
/// and the part that never reaches the screen is real bytes of the file the agent will
/// never see rendered. This function names exactly that rectangle, mapped back through the
/// same linear resampling, in source pixels.
///
/// Takes the *local* box (`width`×`height`, before `scale`) and the drawn rectangle
/// [`drawn_rect`] computed, because the resampling happened at the local box's size and
/// the transform that placed it in frame space is what this function undoes to get back
/// there.
///
/// `None` where `clip` excludes the element entirely — nothing of the source reaches the
/// screen, which is a fact worth a name of its own rather than a rectangle of zero size
/// that reads like a measurement.
pub fn crop_rectangle(
    local: (i64, i64),
    drawn: Rect,
    clip: Rect,
    source: (i64, i64),
) -> Option<Rect> {
    let visible = drawn.intersect(clip)?;
    let (local_width, local_height) = local;
    let (source_width, source_height) = source;
    if local_width <= 0 || local_height <= 0 || drawn.width <= 0 || drawn.height <= 0 {
        return None;
    }

    // Undo the transform that produced `drawn` from the local box: `drawn` is the local
    // box scaled by `drawn.width / local_width` (and the height axis independently), with
    // no rotation (the only case this module's caller reaches this function for).
    let scale_x = drawn.width as f64 / local_width as f64;
    let scale_y = drawn.height as f64 / local_height as f64;
    let local_left = (visible.x - drawn.x) as f64 / scale_x;
    let local_top = (visible.y - drawn.y) as f64 / scale_y;
    let local_right = local_left + visible.width as f64 / scale_x;
    let local_bottom = local_top + visible.height as f64 / scale_y;

    // The local box *is* the resampled source, so the map into source pixels is the same
    // proportion on each axis independently — the driving axis of ADR-0013's rule maps
    // 1:1 by construction, and the slack axis maps by the same ratio the fitted-extent
    // arithmetic used to derive it, floored to a whole pixel here as everywhere else the
    // format rounds a derived extent.
    let sx = source_width as f64 / local_width as f64;
    let sy = source_height as f64 / local_height as f64;
    let left = (local_left * sx).floor() as i64;
    let top = (local_top * sy).floor() as i64;
    let right = (local_right * sx).ceil() as i64;
    let bottom = (local_bottom * sy).ceil() as i64;

    Some(Rect {
        x: left.clamp(0, source_width),
        y: top.clamp(0, source_height),
        width: (right - left).clamp(0, source_width),
        height: (bottom - top).clamp(0, source_height),
    })
}

/// **`NOT COVERED`**: the region of the frame no visible element's rectangle reaches, as a
/// set of rectangles whose union is exactly that region.
///
/// Not necessarily the *fewest* rectangles that could express it — a coordinate-compressed
/// grid, marked cell by cell and merged one row at a time, then merged again down columns
/// of identical width. That is a real partition of the uncovered area, computed exactly,
/// rather than a minimal cover this function does not need to claim.
///
/// `rects` is every element's own drawn (and `clip`-intersected) rectangle. **An `ellipse`
/// contributes its full bounding rectangle here**, not the inscribed ellipse it actually
/// paints — recorded as this function's one over-approximation, on the ADR-0018 reading
/// that a false "covered" at a shape's four corners is the safer error than a spurious gap
/// report on every rounded panel the fixture draws.
pub fn not_covered(frame: (i64, i64), rects: &[Rect]) -> Vec<Rect> {
    let (frame_width, frame_height) = frame;
    if frame_width <= 0 || frame_height <= 0 {
        return Vec::new();
    }

    let mut xs: Vec<i64> = vec![0, frame_width];
    let mut ys: Vec<i64> = vec![0, frame_height];
    for rect in rects {
        xs.push(rect.x.clamp(0, frame_width));
        xs.push(rect.right().clamp(0, frame_width));
        ys.push(rect.y.clamp(0, frame_height));
        ys.push(rect.bottom().clamp(0, frame_height));
    }
    xs.sort_unstable();
    xs.dedup();
    ys.sort_unstable();
    ys.dedup();

    let covered = |cx0: i64, cx1: i64, cy0: i64, cy1: i64| {
        rects.iter().any(|rect| {
            rect.x <= cx0 && cx1 <= rect.right() && rect.y <= cy0 && cy1 <= rect.bottom()
        })
    };

    // One rectangle per row of cells first — adjacent uncovered cells in a row merge into
    // one interval — then rows with the same `(x, width)` merge vertically. The second
    // pass is what keeps a tall uncovered strip from coming back as one rectangle per row
    // of the grid.
    let mut row_rects: Vec<Rect> = Vec::new();
    for wy in ys.windows(2) {
        let (y0, y1) = (wy[0], wy[1]);
        let mut run_start: Option<i64> = None;
        for wx in xs.windows(2) {
            let (x0, x1) = (wx[0], wx[1]);
            if covered(x0, x1, y0, y1) {
                if let Some(start) = run_start.take() {
                    row_rects.push(Rect {
                        x: start,
                        y: y0,
                        width: x0 - start,
                        height: y1 - y0,
                    });
                }
            } else if run_start.is_none() {
                run_start = Some(x0);
            }
        }
        if let Some(start) = run_start {
            row_rects.push(Rect {
                x: start,
                y: y0,
                width: xs[xs.len() - 1] - start,
                height: y1 - y0,
            });
        }
    }

    let mut out: Vec<Rect> = Vec::new();
    'rows: for rect in row_rects {
        for existing in out.iter_mut() {
            if existing.x == rect.x && existing.width == rect.width && existing.bottom() == rect.y {
                existing.height += rect.height;
                continue 'rows;
            }
        }
        out.push(rect);
    }
    out
}

/// **The ink box**: a text element's rendered extent, in absolute frame-space pixels —
/// the tight rectangle the glyphs actually occupy, stroke included, not the nominal
/// `size × line_height` box ADR-0011 measured overstating real ink by 1.25×–1.48×.
///
/// Built on [`montagent_text::measure`] — the same engine and the same call `measure` the
/// verb makes (#205) — so this is the second *caller* of that arithmetic, never a second
/// *implementation* of it. What this function adds is the half `measure` does not build:
/// the block's horizontal placement (`x`, `origin`'s horizontal component, `align`), which
/// turns a line's typographic advance into an absolute rectangle.
///
/// **The lines align inside the block they make, and `origin` places that block**
/// (ADR-0135): the widest line's advance, exactly as the painter places it. The declared
/// `width` is a container claim (ADR-0014) and is not read here, as the declared `height`
/// is not read by `measure`. Each line's offset into the block is
/// [`montagent_text::place::offset`] — the painter's own function, not a copy of it.
///
/// **`start` and `end` resolve against each line's base direction** (ADR-0133): the one
/// the engine laid the line out in, read off its measurement rather than guessed here from
/// the characters. A run's `dir` is an isolate and never changes it.
///
/// **Scope, stated rather than left to be discovered**: refuses on a non-zero resolved
/// `rotation` or a resolved `scale` other than `[1,1]` (the ink box is not derived at a
/// transformed size). None of the fixture's 22 text elements trips either refusal.
pub fn ink_box(
    document: &Loose,
    element: &Value,
    instant: i64,
    frame: (i64, i64),
) -> Result<InkBox, String> {
    let rotation = number::<f64>(element, "rotation", instant, 0.0);
    if rotation != 0.0 {
        return Err(format!(
            "its resolved `rotation` is {rotation}°; the ink box is not derived for a \
             rotated text element"
        ));
    }
    let scale = number::<[f64; 2]>(element, "scale", instant, [1.0, 1.0]);
    if scale != [1.0, 1.0] {
        return Err(format!(
            "its resolved `scale` is {scale:?}; the ink box is not derived at a scaled size"
        ));
    }
    // A text on a path draws its bent line in its declared box (ADR-0161 §7): the ink box is
    // that line's ink, moved to where the box sits.
    if crate::text_path::carries_path(element) {
        let ink = crate::text_path::ink_at(document, element, instant)?
            .ok_or_else(|| "no letter is drawn on its curve at this instant".to_string())?;
        let origin = match element.get("origin") {
            None | Some(Value::Null) => Origin::Center,
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|_| format!("`origin` is not one of the nine keywords: {value}"))?,
        };
        let (fx, fy) = origin_fraction(origin);
        let side = |key| element.get(key).and_then(Value::as_f64).unwrap_or(0.0);
        let left = number::<i64>(element, "x", instant, frame.0 as f64 / 2.0) - fx * side("width");
        let top = number::<i64>(element, "y", instant, frame.1 as f64 / 2.0) - fy * side("height");
        let [l, t, r, b] = ink;
        return Ok(InkBox {
            x: left + l,
            y: top + t,
            width: r - l,
            height: b - t,
        });
    }
    let spec = Measurable::of(element)?;

    let mut fonts = montagent_text::Fonts::new();
    for key in std::iter::once(spec.asked.font.clone()).chain(Measurable::keys(element)) {
        register(document, &key, &mut fonts).map_err(|e| e.to_string())?;
    }

    let resolved_y = number::<i64>(element, "y", instant, frame.1 as f64 / 2.0).round() as i64;
    let runs = runs_of(element);
    // A malformed `align` refuses rather than answer for `start`, as a malformed `origin`
    // does below: the ink box is an answer about the document as written.
    if let Some(value) = element.get("align").filter(|value| !value.is_null())
        && !matches!(value.as_str(), Some("start" | "center" | "end"))
    {
        return Err(format!(
            "`align` is not `start`, `center` or `end`: {value}"
        ));
    }
    let align = align_of(element);
    let measured = montagent_text::measure(
        &mut fonts,
        &montagent_text::Spec {
            runs: &runs,
            font: &spec.asked.font,
            size: spec.asked.size,
            line_height_tenths: spec.asked.line_height_tenths,
            stroke_width: spec.asked.stroke_width,
            y: resolved_y,
            vertical_origin: spec.vertical_origin,
            // Read back below, with each line's direction, through the painter's own
            // `offset` — so the ink box and the picture align every line the same way.
            align,
            // The spaced line at this instant (ADR-0151): the ink box measures it.
            letter_spacing: crate::verbs::measure::letter_spacing_at(element, instant),
            optional_ligatures_off: crate::verbs::measure::optional_ligatures_off(element),
        },
    )
    .map_err(|e| e.to_string())?;

    let resolved_x = number::<i64>(element, "x", instant, frame.0 as f64 / 2.0);
    let origin = match element.get("origin") {
        None | Some(Value::Null) => Origin::Center,
        Some(value) => serde_json::from_value(value.clone())
            .map_err(|_| format!("`origin` is not one of the nine keywords: {value}"))?,
    };
    let (fx, _) = origin_fraction(origin);
    // The painter's block: the widest line's advance, before the stroke (ADR-0014), which
    // is the extent `origin` pivots about in `frame`'s transform.
    let block_width = measured.advance_width;
    let block_left = resolved_x - fx * block_width;

    let mut ink_left = f64::INFINITY;
    let mut ink_right = f64::NEG_INFINITY;
    let mut ink_top = f64::INFINITY;
    let mut ink_bottom = f64::NEG_INFINITY;
    for line in &measured.lines {
        let offset =
            montagent_text::place::offset(align, line.rtl, block_width, line.advance_width);
        let stroke = line.stroke_width as f64;
        let left = block_left + offset - stroke;
        let right = left + line.extent_width;
        ink_left = ink_left.min(left);
        ink_right = ink_right.max(right);
        ink_top = ink_top.min(line.baseline_y - line.ascent - stroke);
        ink_bottom = ink_bottom.max(line.baseline_y + line.descent + stroke);
    }

    Ok(InkBox {
        x: ink_left,
        y: ink_top,
        width: ink_right - ink_left,
        height: ink_bottom - ink_top,
    })
}

/// A text element's rendered extent, in absolute frame-space pixels — stroke included
/// (ADR-0014), never the nominal box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct InkBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
