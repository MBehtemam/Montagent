//! prototype(#764): a text element bent along its own inline `path` (ADR-0161).
//!
//! One function, [`bend`], places every rigid body of a laid-out line on the curve at an
//! instant. The painter and `query --at` both call it, so the letters the frame hides are the
//! letters `query` names by construction, not by agreement.
//!
//! The flat line is laid out exactly as today (ADR-0161 §4); then each rigid body — a letter,
//! a joined piece, a ligature cluster ([`montagent_text::units::bodies`] under `by: letter`) —
//! is placed at its advance midpoint:
//!
//! - its posed flat midpoint `x` (after the stagger's pose) becomes a distance
//!   `d = path_offset × L + (x − x_anchor)` along the curve;
//! - its posed flat baseline `y`, less the line's rest baseline, becomes an offset along the
//!   normal (the tangent turned 90° clockwise on screen);
//! - its rotation adds to the tangent angle.
//!
//! As one matrix per body: `T(P(d)) · R(θ) · T(−x', −y₀) · S`, where `S` is the stagger's
//! flat matrix and `(x', ·)` its posed midpoint. The canvas already concatenates a unit matrix
//! before moving a glyph to its flat place, so nothing in the canvas changes.

use montagent_render::canvas::{Curve, UnitDraw};
use montagent_text::units::By;
use serde::Serialize;
use serde_json::Value;

use crate::animatable::{self, Resolved};

/// Whether this element is a text carrying a `path`.
pub(crate) fn has_path(element: &Value) -> bool {
    element.get("type").and_then(Value::as_str) == Some("text")
        && element.get("path").is_some_and(Value::is_object)
}

/// ADR-0161 §7's inset: the largest `size` among the runs (the element's where a run states
/// none) plus the largest `stroke_width` among the runs and the element (its largest key).
pub fn inset(element: &Value) -> i64 {
    let base_size = element.get("size").and_then(Value::as_i64).unwrap_or(0);
    let base_stroke = animatable::greatest_length(element, "stroke_width")
        .unwrap_or(0)
        .max(0);
    let runs = element
        .get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let size = runs
        .iter()
        .filter_map(|run| run.get("size").and_then(Value::as_i64))
        .chain(std::iter::once(base_size))
        .max()
        .unwrap_or(0);
    let stroke = runs
        .iter()
        .filter_map(|run| run.get("stroke_width").and_then(Value::as_i64))
        .chain(std::iter::once(base_stroke))
        .max()
        .unwrap_or(0);
    size + stroke
}

/// How `m` was derived, in `validate`'s words.
pub fn derivation(element: &Value) -> String {
    let base_size = element.get("size").and_then(Value::as_i64).unwrap_or(0);
    let runs = element
        .get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let size = runs
        .iter()
        .filter_map(|run| run.get("size").and_then(Value::as_i64))
        .chain(std::iter::once(base_size))
        .max()
        .unwrap_or(0);
    let m = inset(element);
    format!(
        "{m} = {size} (the largest `size` among the runs) + {} (the largest `stroke_width` \
         among the runs and the element)",
        m - size
    )
}

/// The guide at an instant, through the one resolving function (which clamps an overshoot
/// into the §7 inset box), as the outline a `path` element would stroke.
pub(crate) fn curve_at(element: &Value, t: (i128, i128)) -> Option<Curve> {
    let Some(Ok(Resolved::Points(vertices))) = animatable::read(element, "path.points", t.0, t.1)
    else {
        return None;
    };
    let closed = element
        .get("path")
        .and_then(|path| path.get("closed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Some(Curve::of(&crate::verbs::frame::outline_of(&vertices, closed)))
}

/// One rigid body on the curve at an instant.
#[derive(Debug, Clone, Serialize)]
pub struct BodyPlace {
    /// Its ADR-0151 letter indices.
    pub letters: Vec<usize>,
    /// Its flat advance width, letter spacing excluded.
    pub advance: f64,
    /// Its distance along the curve, before a closed curve's wrap.
    pub d: f64,
    pub drawn: bool,
    /// Where its midpoint sits on the curve and the tangent there, when drawn.
    #[serde(skip)]
    pub on_curve: Option<(f64, f64, f64)>,
}

/// A text's line, bent, at one instant.
pub struct Bent {
    pub length: f64,
    pub offset: f64,
    pub closed: bool,
    /// Per placed glyph: the matrix it is drawn through, or `None` when it is not drawn.
    pub glyph: Vec<Option<UnitDraw>>,
    /// Per placed glyph: the letters of the rigid body it belongs to (empty for whitespace).
    pub glyph_letters: Vec<Vec<usize>>,
    pub bodies: Vec<BodyPlace>,
    pub curve: Curve,
}

impl Bent {
    /// Every letter not drawn, ascending.
    pub fn hidden(&self) -> Vec<usize> {
        let mut out: Vec<usize> = self
            .bodies
            .iter()
            .filter(|body| !body.drawn)
            .flat_map(|body| body.letters.iter().copied())
            .collect();
        out.sort_unstable();
        out
    }
}

/// `a · b`, both `[sx, kx, tx, ky, sy, ty]`.
fn mul(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[1] * b[3],
        a[0] * b[1] + a[1] * b[4],
        a[0] * b[2] + a[1] * b[5] + a[2],
        a[3] * b[0] + a[4] * b[3],
        a[3] * b[1] + a[4] * b[4],
        a[3] * b[2] + a[4] * b[5] + a[5],
    ]
}

fn apply(m: [f64; 6], (x, y): (f64, f64)) -> (f64, f64) {
    (m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5])
}

const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];

/// Place `placement`'s line on the element's curve at `t`. `None` where the element carries
/// no readable `path`.
pub(crate) fn bend(
    element: &Value,
    placement: &montagent_text::Placement,
    t: (i128, i128),
) -> Option<Bent> {
    if !has_path(element) {
        return None;
    }
    let curve = curve_at(element, t)?;
    let length = curve.length;
    let closed = curve.closed;
    let offset = animatable::number_read(element, "path_offset", t.0, t.1, 0.0).clamp(0.0, 1.0);

    // The line's flat advance extent, letter spacing included: from the leftmost cluster's
    // left edge to the rightmost cluster's trailing edge (no spacing follows the last).
    let (mut left, mut right) = (f64::INFINITY, f64::NEG_INFINITY);
    for cluster in placement.clusters.iter().filter(|c| c.text.is_some()) {
        left = left.min(cluster.x);
        right = right.max(cluster.x + cluster.advance);
    }
    if !left.is_finite() {
        (left, right) = (0.0, 0.0);
    }
    let anchor = match element.get("align").and_then(Value::as_str) {
        Some("center") => (left + right) / 2.0,
        Some("end") => right,
        _ => left,
    };
    let baseline = placement
        .measurement
        .lines
        .first()
        .map(|line| line.baseline_y - placement.measurement.block_top)
        .unwrap_or(0.0);

    let text: String = crate::verbs::measure::runs_array(element)
        .iter()
        .map(|run| run.get("text").and_then(Value::as_str).unwrap_or(""))
        .collect();
    let rigid = montagent_text::units::bodies(&text, By::Letter, placement);
    let stagger = crate::verbs::frame::unit_draws(element, placement, t);

    let mut bodies = Vec::with_capacity(rigid.bodies.len());
    let mut matrices: Vec<Option<(UnitDraw, bool)>> = Vec::with_capacity(rigid.bodies.len());
    for (index, body) in rigid.bodies.iter().enumerate() {
        let letters = body.units.clone();
        let Some([l, _, r, _]) = body.rect else {
            bodies.push(BodyPlace {
                letters,
                advance: 0.0,
                d: 0.0,
                drawn: false,
                on_curve: None,
            });
            matrices.push(None);
            continue;
        };
        let mid = (l + r) / 2.0;
        // The stagger pose of the body's first glyph: every glyph of one rigid body is in
        // one stagger body.
        let pose = rigid
            .glyph_body
            .iter()
            .position(|b| *b == Some(index))
            .and_then(|g| stagger.get(g).copied().flatten());
        let s = pose.map_or(IDENTITY, |p| p.matrix.map(f64::from));
        let (mx, _) = apply(s, (mid, baseline));
        let d = offset * length + (mx - anchor);
        let drawn = if closed {
            // Up to one full loop from the line's lowest-distance end, judged on the rest
            // (flat, unposed) midpoint.
            mid - left < length
        } else {
            (0.0..=length).contains(&d)
        };
        let at = if closed { d.rem_euclid(length) } else { d };
        let on_curve = drawn.then(|| curve.at(at)).flatten();
        let drawn = on_curve.is_some();
        bodies.push(BodyPlace {
            letters,
            advance: r - l,
            d,
            drawn,
            on_curve,
        });
        matrices.push(on_curve.map(|(px, py, theta)| {
            let (sin, cos) = theta.sin_cos();
            let place = [cos, -sin, px, sin, cos, py];
            let m = mul(mul(place, [1.0, 0.0, -mx, 0.0, 1.0, -baseline]), s);
            (
                UnitDraw {
                    body: pose.map_or(1_000_000 + index, |p| p.body),
                    matrix: m.map(|v| v as f32),
                    opacity: pose.map_or(1.0, |p| p.opacity),
                },
                true,
            )
        }));
    }
    let glyph = rigid
        .glyph_body
        .iter()
        .map(|body| {
            body.and_then(|b| matrices.get(b).cloned().flatten())
                .map(|(draw, _)| draw)
        })
        .collect();
    let glyph_letters = rigid
        .glyph_body
        .iter()
        .map(|body| body.map(|b| rigid.bodies[b].units.clone()).unwrap_or_default())
        .collect();
    Some(Bent {
        glyph_letters,
        length,
        offset,
        closed,
        glyph,
        bodies,
        curve,
    })
}

/// What `query --at` reports on a text carrying `path` (ADR-0161 §8).
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// `path_offset` at the instant, raw (clamped to [0, 1] only by an overshooting ease).
    pub path_offset: f64,
    /// The curve's length by the painter's own path measure, to 0.01 px. Informative.
    pub length: f64,
    pub length_note: &'static str,
    /// The letters not drawn at the instant, by ADR-0151 letter index, or `"none"`.
    pub hidden: Value,
}

/// `query --at`'s reading: lays the line out with the same spec the painter does.
pub(crate) fn report(
    document: &crate::permissive::Loose,
    element: &Value,
    instant: i64,
) -> Option<Result<Report, String>> {
    if !has_path(element) {
        return None;
    }
    Some(placement_for(document, element, instant).and_then(|placement| {
        let bent = bend(element, &placement, (i128::from(instant), 1))
            .ok_or_else(|| "its `path.points` do not resolve".to_string())?;
        let hidden = bent.hidden();
        Ok(Report {
            path_offset: animatable::number_at(element, "path_offset", instant, 0.0),
            length: (bent.length * 100.0).round() / 100.0,
            length_note: "informative, not a contract: the painter's own path measure",
            hidden: if hidden.is_empty() {
                Value::String("none".into())
            } else {
                serde_json::json!(hidden)
            },
        })
    }))
}

/// The flat placement the painter makes for this element at `instant`.
pub(crate) fn placement_for(
    document: &crate::permissive::Loose,
    element: &Value,
    instant: i64,
) -> Result<montagent_text::Placement, String> {
    use crate::verbs::measure::{Measurable, register};
    let style = Measurable::of(element)?;
    let mut fonts = montagent_text::Fonts::new();
    for key in std::iter::once(style.asked.font.clone()).chain(Measurable::keys(element)) {
        register(document, &key, &mut fonts).map_err(|e| e.to_string())?;
    }
    let runs = crate::verbs::measure::runs_of(element);
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
            align: crate::verbs::measure::align_of(element),
            letter_spacing: crate::verbs::measure::letter_spacing_read(
                element,
                (i128::from(instant), 1),
            ),
            optional_ligatures_off: crate::verbs::measure::optional_ligatures_off(element),
        },
    )
    .map_err(|e| e.to_string())
}
