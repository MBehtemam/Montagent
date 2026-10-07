//! A text element's one line, bent along its own inline `path` (ADR-0161).
//!
//! [`bend`] is the one placement: the painter draws through it and `query --at` and `measure`
//! report through it, so the letters the frame hides are the letters they name by
//! construction.
//!
//! The line is laid out flat exactly as a text with no `path` is, letter spacing and the
//! `units` stagger included (§4). Then each **rigid body** — a letter, a joined piece
//! (ADR-0153) or a ligature cluster, [`montagent_text::units::bodies`] under `by: letter` —
//! is moved and turned whole, never bent (§3):
//!
//! - its posed advance midpoint's flat `x` gives the distance
//!   `d = path_offset × L + (x − x_anchor)`, where `L` is the curve's length and `x_anchor`
//!   the flat `x` of the line's lowest-distance end, middle or highest-distance end for
//!   `align` `start`, `center` or `end`: on a path, `align` does not follow the text's
//!   direction (§5);
//! - its posed baseline `y`, less the line's own baseline, is an offset along the normal at
//!   `d`, the tangent turned 90° clockwise on screen, so negative `y` is left of travel;
//! - its rotation adds to the tangent angle at `d`.
//!
//! As one matrix per body, in the block's own coordinates: `T(P(d)) · R(θ) · T(−x′, −y₀) · S`,
//! where `S` is the stagger's pose, `x′` the posed midpoint's `x` and `y₀` the line's
//! baseline. The canvas already draws a glyph through a unit matrix before moving it to its
//! flat place, so it draws a bent line with no change.
//!
//! **The ends** (§6, ADR-0164 §2). On an open path a body whose posed midpoint's `d` falls
//! outside `[0, L]` is not drawn, SVG's rule. On a closed path `d` wraps around the start
//! point, and a body is drawn only if its whole advance, where the stagger's `x` has moved
//! it, lies within one loop `[0, L)` measured from the line's lowest-distance end, so the
//! seam never overlaps. `path_offset` runs from −1 to 2 (ADR-0164 §1), so one element can
//! slide a line on and off an open curve.
//!
//! The text's declared box is the frame the curve is written in, as a `path` element's is
//! (§7), so a text on a path is pivoted about its declared box rather than its typographic
//! block.

use montagent_render::canvas::{Curve, UnitDraw};
use montagent_text::Placement;
use montagent_text::units::By;
use serde::Serialize;
use serde_json::Value;

use crate::animatable::{self, Resolved};
use crate::permissive::Loose;

/// Whether this element is a text carrying a `path` (ADR-0161 §2).
pub(crate) fn carries_path(element: &Value) -> bool {
    element.get("type").and_then(Value::as_str) == Some("text")
        && element.get("path").is_some_and(Value::is_object)
}

/// ADR-0161 §7's inset: `m = the largest size among the runs + the largest stroke_width
/// among the runs and the element`, from the file alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inset {
    pub m: i64,
    /// The largest `size` a run is set in: its own, or the element's where it states none.
    pub size: i64,
    /// The largest `stroke_width` among the runs and the element, the element's at its
    /// largest keyed value; `0` with none.
    pub stroke: i64,
}

impl Inset {
    pub fn of(element: &Value) -> Inset {
        let own = element.get("size").and_then(Value::as_i64);
        let runs = crate::verbs::measure::runs_array(element);
        let size = runs
            .iter()
            .filter_map(|run| run.get("size").and_then(Value::as_i64).or(own))
            .chain(runs.is_empty().then_some(own).flatten())
            .max()
            .unwrap_or(0);
        let stroke = runs
            .iter()
            .filter_map(|run| run.get("stroke_width").and_then(Value::as_i64))
            .chain(animatable::greatest_length(element, "stroke_width"))
            .max()
            .unwrap_or(0)
            .max(0);
        Inset {
            m: size + stroke,
            size,
            stroke,
        }
    }

    /// How `m` was derived, in `validate`'s words.
    pub fn derivation(&self) -> String {
        format!(
            "{} = {} (the largest `size` among the runs) + {} (the largest `stroke_width` among \
             the runs and the element)",
            self.m, self.size, self.stroke
        )
    }
}

/// The text's curve at `t = (numerator, denominator)` ms, resolved through the one resolving
/// function (which clamps an overshoot into the inset box), as the outline a `path` element
/// with the same points would stroke. `None` where `path.points` does not resolve.
fn curve_at(element: &Value, t: (i128, i128)) -> Option<Curve> {
    let Some(Ok(Resolved::Points(vertices))) = animatable::read(element, "path.points", t.0, t.1)
    else {
        return None;
    };
    Some(Curve::of(&crate::verbs::frame::outline_of(
        &vertices,
        closed(element),
    )))
}

fn closed(element: &Value) -> bool {
    element
        .get("path")
        .and_then(|path| path.get("closed"))
        .and_then(Value::as_bool)
        == Some(true)
}

/// A text's line, bent, at one instant.
pub(crate) struct Bent {
    /// The curve's length, by the painter's own path measure.
    pub length: f64,
    /// Per placed glyph, the matrix it is drawn through, or `None` where it is not drawn: its
    /// body is past an end, or it is whitespace, which has no ink and no place on a curve.
    pub glyphs: Vec<Option<UnitDraw>>,
    /// Every letter not drawn, by its ADR-0151 letter index, ascending.
    pub hidden: Vec<usize>,
}

/// `a · b`, both `[sx, kx, tx, ky, sy, ty]`.
fn times(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[1] * b[3],
        a[0] * b[1] + a[1] * b[4],
        a[0] * b[2] + a[1] * b[5] + a[2],
        a[3] * b[0] + a[4] * b[3],
        a[3] * b[1] + a[4] * b[4],
        a[3] * b[2] + a[4] * b[5] + a[5],
    ]
}

const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];

/// Place `placement`'s line on the element's curve at `t = (numerator, denominator)` ms. `None`
/// where the element carries no `path`, or its points do not resolve.
pub(crate) fn bend(element: &Value, placement: &Placement, t: (i128, i128)) -> Option<Bent> {
    if !carries_path(element) {
        return None;
    }
    let curve = curve_at(element, t)?;
    let length = curve.length;
    let closed = closed(element);
    let offset = animatable::number_read(element, "path_offset", t.0, t.1, 0.0);

    // The line's extent in flat space is its advance, letter spacing included: one line is
    // the whole block, so it runs from 0 to the block's width (§5).
    let anchor = match crate::verbs::measure::align_of(element) {
        montagent_text::Align::Start => 0.0,
        montagent_text::Align::Center => placement.width / 2.0,
        montagent_text::Align::End => placement.width,
    };
    let baseline = placement.measurement.lines.first().map_or(0.0, |line| {
        line.baseline_y - placement.measurement.block_top
    });

    let text: String = crate::verbs::measure::runs_array(element)
        .iter()
        .filter_map(|run| run.get("text").and_then(Value::as_str))
        .collect();
    let rigid = montagent_text::units::bodies(&text, By::Letter, placement);
    let poses = crate::verbs::frame::unit_draws(element, placement, t);

    let mut hidden = Vec::new();
    let draws: Vec<Option<UnitDraw>> = rigid
        .bodies
        .iter()
        .enumerate()
        .map(|(index, body)| {
            let [left, _, right, _] = body.rect?;
            // The stagger's pose of the body's first glyph: a rigid body never spans two
            // stagger bodies.
            let pose = rigid
                .glyph_body
                .iter()
                .position(|glyph| *glyph == Some(index))
                .and_then(|glyph| poses.get(glyph).copied().flatten());
            let stagger = pose.map_or(IDENTITY, |pose| pose.matrix.map(f64::from));
            let mid = (left + right) / 2.0;
            let posed = stagger[0] * mid + stagger[1] * baseline + stagger[2];
            let d = offset * length + (posed - anchor);
            let drawn = if closed {
                // ADR-0164 §2: the body's whole advance, where the stagger has moved it,
                // within one loop `[0, L)` from the line's lowest-distance end (flat `x` 0).
                let half = (right - left) / 2.0;
                posed - half >= 0.0 && posed + half < length
            } else {
                (0.0..=length).contains(&d)
            };
            let at = if closed && length > 0.0 {
                d.rem_euclid(length)
            } else {
                d
            };
            let Some((x, y, angle)) = drawn.then(|| curve.at(at)).flatten() else {
                hidden.extend(body.units.iter().copied());
                return None;
            };
            let (sin, cos) = angle.sin_cos();
            let matrix = times(
                times(
                    [cos, -sin, x, sin, cos, y],
                    [1.0, 0.0, -posed, 0.0, 1.0, -baseline],
                ),
                stagger,
            );
            Some(UnitDraw {
                body: pose.map_or(usize::MAX - index, |pose| pose.body),
                matrix: matrix.map(|v| v as f32),
                opacity: pose.map_or(1.0, |pose| pose.opacity),
            })
        })
        .collect();
    hidden.sort_unstable();
    let glyphs = rigid
        .glyph_body
        .iter()
        .map(|body| body.and_then(|body| draws.get(body).copied().flatten()))
        .collect();
    Some(Bent {
        length,
        glyphs,
        hidden,
    })
}

/// Lay a text element out flat as the painter does at `t`, with every font its chain and
/// runs name registered from the document. `Err` says why it could not be placed. What
/// `query --at` and `measure` read a text's glyphs from, bent or staggered.
pub(crate) fn place(
    document: &Loose,
    element: &Value,
    t: (i128, i128),
) -> Result<Placement, String> {
    use crate::verbs::measure::{
        Measurable, align_of, letter_spacing_read, optional_ligatures_off, register, runs_of,
    };
    let style = Measurable::of(element)?;
    let mut fonts = montagent_text::Fonts::new();
    for key in std::iter::once(style.asked.font.clone()).chain(Measurable::keys(element)) {
        register(document, &key, &mut fonts).map_err(|e| e.to_string())?;
    }
    let runs = runs_of(element);
    montagent_text::place(
        &mut fonts,
        &montagent_text::Spec {
            runs: &runs,
            font: &style.asked.font,
            size: style.asked.size,
            line_height_tenths: style.asked.line_height_tenths,
            stroke_width: style.asked.stroke_width,
            y: 0,
            vertical_origin: montagent_text::VerticalOrigin::Top,
            align: align_of(element),
            letter_spacing: letter_spacing_read(element, t),
            optional_ligatures_off: optional_ligatures_off(element),
        },
    )
    .map_err(|e| e.to_string())
}

/// What `query --at` and `measure` report of a text on a path (ADR-0161 §8).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reading {
    /// `path_offset` at the instant through the one resolving function, unrounded: `0`
    /// where the text writes none.
    pub path_offset: f64,
    /// The curve's length by the painter's own path measure, to 0.01 px. Informative, not a
    /// contract: it is a measure of a curve, not a number in the file.
    pub length: f64,
    pub length_note: &'static str,
    /// The letters not drawn at the instant, by their ADR-0151 letter index, or `"none"`. A
    /// hidden joined piece or ligature lists all its letters.
    pub hidden: Value,
    /// The bent line's ink, `[left, top, right, bottom]` in pixels from the declared box's
    /// top-left, stroke included; `null` where nothing is drawn. Where it passes the box,
    /// the line overflows it (ADR-0161 §7).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ink: Option<Option<[f64; 4]>>,
    /// Why the line could not be placed, where it could not: a font, or points that do not
    /// resolve. The other fields are then empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved: Option<String>,
}

impl Reading {
    /// `query --at`'s reading at `instant`, or `None` on a text with no `path`.
    pub(crate) fn at(document: &Loose, element: &Value, instant: i64) -> Option<Reading> {
        Reading::of(document, element, instant, false)
    }

    /// `measure`'s: the same reading at the element's first frame, with the bent ink.
    pub(crate) fn first_frame(document: &Loose, element: &Value) -> Option<Reading> {
        let start = element.get("start").and_then(Value::as_i64).unwrap_or(0);
        let first = document
            .value()
            .get("fps")
            .and_then(Value::as_i64)
            .and_then(|fps| {
                crate::exact::frame_at_or_after(start, fps)
                    .map(|frame| crate::exact::instant_of(frame.frame, fps))
            })
            .unwrap_or(start);
        Reading::of(document, element, first, true)
    }

    fn of(document: &Loose, element: &Value, instant: i64, ink: bool) -> Option<Reading> {
        if !carries_path(element) {
            return None;
        }
        let path_offset = animatable::number_at(element, "path_offset", instant, 0.0);
        let t = (i128::from(instant), 1);
        let unresolved = |reason: String| Reading {
            path_offset,
            length: 0.0,
            length_note: LENGTH_NOTE,
            hidden: Value::Null,
            ink: None,
            unresolved: Some(reason),
        };
        let placement = match place(document, element, t) {
            Ok(placement) => placement,
            Err(reason) => return Some(unresolved(reason)),
        };
        let Some(bent) = bend(element, &placement, t) else {
            return Some(unresolved(
                "its `path.points` do not resolve at the instant".to_string(),
            ));
        };
        Some(Reading {
            path_offset,
            length: (bent.length * 100.0).round() / 100.0,
            length_note: LENGTH_NOTE,
            hidden: if bent.hidden.is_empty() {
                Value::String("none".to_string())
            } else {
                serde_json::json!(bent.hidden)
            },
            ink: ink.then(|| ink_of(element, &placement, &bent, instant)),
            unresolved: None,
        })
    }
}

const LENGTH_NOTE: &str = "informative, not a contract: the painter's own path measure";

/// The bent line's ink at `instant`, in box pixels: `query --at`'s ink box on a text on a
/// path. `Ok(None)` where no letter is drawn; `Err` says why the line could not be placed.
pub(crate) fn ink_at(
    document: &Loose,
    element: &Value,
    instant: i64,
) -> Result<Option<[f64; 4]>, String> {
    let t = (i128::from(instant), 1);
    let placement = place(document, element, t)?;
    let bent = bend(element, &placement, t)
        .ok_or_else(|| "its `path.points` do not resolve at the instant".to_string())?;
    Ok(ink_of(element, &placement, &bent, instant))
}

/// The bent line's ink in box pixels: every drawn glyph's outline through its matrix, with
/// its stroke.
fn ink_of(element: &Value, placement: &Placement, bent: &Bent, instant: i64) -> Option<[f64; 4]> {
    use crate::verbs::frame::{glyphs_of, outlines_of, text_paints};
    // Where a gradient sits changes no glyph's ink, so its box is not worked out here.
    let paints = text_paints(element, (i128::from(instant), 1), [0.0; 4]);
    let glyphs = glyphs_of(placement, &bent.glyphs, &paints, true);
    montagent_render::canvas::glyph_ink(&glyphs, &outlines_of(placement))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{bend, place};
    use crate::permissive::Loose;

    /// A document beside the trailer's fonts, so `fonts/Oswald-SemiBold.ttf` resolves.
    fn document() -> Loose {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/benchmark/spy-trailer");
        Loose::new(
            dir.join("t.montagent.json").display().to_string(),
            json!({"fonts": {"title": [{"file": "fonts/Oswald-SemiBold.ttf"}]}}),
        )
    }

    fn text(fields: Value) -> Value {
        let mut element = json!({"type": "text", "start": 0, "end": 1000, "width": 900,
            "height": 200, "font": "title", "size": 64, "letter_spacing": 40,
            "runs": [{"text": "Wavy fi Text"}]});
        for (key, value) in fields.as_object().unwrap() {
            element[key] = value.clone();
        }
        element
    }

    /// Where a matrix puts the glyph origin `(x, y)`.
    fn mapped(matrix: [f32; 6], (x, y): (f64, f64)) -> (f64, f64) {
        let m = matrix.map(f64::from);
        (m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5])
    }

    #[test]
    fn a_straight_left_to_right_curve_places_the_line_as_flat_text_would() {
        // The curve starts at (100, 120) and runs right: with `align: start` and offset 0
        // the line's left end sits at the start and its baseline on the curve, so every
        // glyph is the flat glyph moved by (100, 120 − baseline), within float tolerance.
        let element = text(json!({"path": {"closed": false, "points": [
            {"at": [100, 120]}, {"at": [880, 120]}]}}));
        let placement = place(&document(), &element, (0, 1)).unwrap();
        let bent = bend(&element, &placement, (0, 1)).unwrap();
        let baseline = placement.measurement.lines[0].baseline_y - placement.measurement.block_top;
        let mut drawn = 0;
        for (glyph, draw) in placement.glyphs.iter().zip(&bent.glyphs) {
            let Some(draw) = draw else { continue };
            drawn += 1;
            let (x, y) = mapped(draw.matrix, (glyph.x, glyph.y));
            // The path measure places a point within a hundredth of a pixel of its distance.
            assert!(
                (x - (glyph.x + 100.0)).abs() < 0.01,
                "{x} vs {}",
                glyph.x + 100.0
            );
            assert!((y - (glyph.y - baseline + 120.0)).abs() < 0.01, "{y}");
            let [a, b, _, d, e, _] = draw.matrix;
            assert_eq!(
                [a, b, d, e],
                [1.0, 0.0, 0.0, 1.0],
                "no turn on a straight curve"
            );
        }
        // Every letter, `f` and `i` apart (the spacing turns the ligature off), and no
        // whitespace glyph.
        assert_eq!(drawn, "WavyfiText".len());
        assert!(bent.hidden.is_empty());
    }

    #[test]
    fn a_downward_curve_turns_every_body_a_quarter_turn_clockwise() {
        let element = text(json!({"path": {"closed": false, "points": [
            {"at": [450, 10]}, {"at": [450, 190]}]}, "runs": [{"text": "AB"}]}));
        let placement = place(&document(), &element, (0, 1)).unwrap();
        let bent = bend(&element, &placement, (0, 1)).unwrap();
        assert_eq!(bent.glyphs.iter().flatten().count(), 2);
        for draw in bent.glyphs.iter().flatten() {
            let [a, b, _, d, e, _] = draw.matrix.map(f64::from);
            assert!(
                a.abs() < 1e-6 && (b + 1.0).abs() < 1e-6,
                "{:?}",
                draw.matrix
            );
            assert!(
                (d - 1.0).abs() < 1e-6 && e.abs() < 1e-6,
                "{:?}",
                draw.matrix
            );
        }
    }
}
