//! Projection (ADR-0167, ADR-0168): what the document says about an element's `swivel`,
//! `tilt` and `perspective`, and where that puts the element, read once for every tool.
//!
//! The painter, `validate`, `query --at` and the contact sheet all ask the same questions of
//! an element: is it projected, with what at this instant, which way does it face, and where
//! are its corners. They are answered here, from the same [`transform_of`] the painter places
//! the element with and the same [`montagent_render::canvas::footprint`] the rasterizer's
//! reach and projection are computed by, so a picture and a report are one reading
//! (ADR-0146's lesson about copied lists, and `tests/projection.rs`'s conformance test).

use montagent_render::canvas::{
    Blend, Effect, Extent, Facing, Footprint, Projection, Transform, eye_bound, footprint,
};
use serde_json::Value;

use crate::model::Origin;
use crate::verbs::query::geometry::{Rect, number_at, origin_fraction};

/// The three fields, in the order the format lists them.
pub const FIELDS: [&str; 3] = ["swivel", "tilt", "perspective"];

/// Whether the element writes an angle, at any value (ADR-0168 §1): `swivel` or `tilt` is a
/// key of the element, static or keyed. This is the whole test for taking the projected
/// path, so the path depends on the file and never on a value at an instant.
pub fn is_projected(element: &Value) -> bool {
    ["swivel", "tilt"]
        .iter()
        .any(|key| element.get(key).is_some_and(|value| !value.is_null()))
}

/// Whether the painter draws this element through a projection: it writes an angle and the
/// `perspective` that gives it an eye. A fact about the file, so it holds at every instant
/// (ADR-0168 §1); an angle with no `perspective` is `E-PROJECTION-PERSPECTIVE-MISSING`, and
/// the element paints as if unprojected.
pub fn projects(element: &Value) -> bool {
    is_projected(element)
        && element
            .get("perspective")
            .is_some_and(|value| !value.is_null())
}

/// The projection at `t` milliseconds, `(numerator, denominator)`: an instant that need not
/// be a whole millisecond, as a motion-blur sample's is (ADR-0155 §3).
///
/// `None` for an element that writes no angle, and for one that writes an angle but no
/// `perspective`, which has no eye to turn in front of (`E-PROJECTION-PERSPECTIVE-MISSING`
/// is what a document like that is told; it paints as if unprojected). An angle that is
/// written once and not the other reads as `0`.
pub fn at(element: &Value, t: (i128, i128)) -> Option<Projection> {
    if !projects(element) {
        return None;
    }
    Some(Projection {
        swivel: number_at::<f64>(element, "swivel", t, 0.0),
        tilt: number_at::<f64>(element, "tilt", t, 0.0),
        perspective: number_at::<f64>(element, "perspective", t, f64::INFINITY),
    })
}

/// The element's own 2D transform at `t`, with its projection: where the painter puts it,
/// before any transition moves it, and fully opaque and blended `normal`. The painter adds
/// what a running transition does and the element's `opacity` and `blend` on top of this,
/// so the placement is read in one place.
pub(crate) fn transform_of(element: &Value, t: (i128, i128), frame: (i64, i64)) -> Transform {
    let origin = match element.get("origin") {
        None | Some(Value::Null) => Origin::Center,
        Some(value) => serde_json::from_value(value.clone()).unwrap_or(Origin::Center),
    };
    let [sx, sy] = number_at::<[f64; 2]>(element, "scale", t, [1.0, 1.0]);
    Transform {
        x: number_at::<i64>(element, "x", t, frame.0 as f64 / 2.0),
        y: number_at::<i64>(element, "y", t, frame.1 as f64 / 2.0),
        origin: origin_fraction(origin),
        scale: (sx, sy),
        rotation: number_at::<f64>(element, "rotation", t, 0.0),
        opacity: 1.0,
        blend: Blend::Normal,
        projection: at(element, t),
    }
}

/// The element's `effects`, in the rasterizer's spelling at `t`, in list order. A member the
/// format does not admit is left out, as the painter leaves it out.
pub(crate) fn effects_at(element: &Value, t: (i128, i128)) -> Vec<Effect> {
    let declared = element
        .get("effects")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    (0..declared)
        // `grain`'s local frame is the one thing only its draw reads, and this reads reach.
        .filter_map(|index| crate::verbs::frame::effect_of(element, index, t, 0))
        .collect()
}

/// An instant, in milliseconds, as an exact fraction: an eased extreme need not fall on a whole
/// millisecond, and the resolving function takes `(numerator, denominator)` (ADR-0035).
pub type Instant = (i128, i128);

/// The denominator an eased extreme's instant is rounded to: a millionth of a millisecond.
const MICRO: i128 = 1_000_000;

/// An instant in milliseconds as a float, for a message.
pub fn millis((numerator, denominator): Instant) -> f64 {
    numerator as f64 / denominator as f64
}

/// **The instants at which an element's projection is looked at** (ADR-0167 §5, §8): its first
/// instant and its last, `end - 1`; every key of the fields that decide it, inside that
/// range; and every eased extreme of those fields, where an ease's `y` overshoots between two
/// keys. The fields are the three, the box (`width`, `height`) and the reach parameters of the
/// effects that widen it, so a check that reads `d` against `r` sees each of them at its own
/// turning points.
///
/// `across_angles` adds a quarter, a half and three quarters of every segment of `swivel` and
/// `tilt`, for the one question the turning points cannot answer: whether a key-to-key turn of
/// more than half a revolution faces the eye in between (`R-PROJECTION-AWAY`).
pub(crate) fn instants(element: &Value, start: i64, end: i64, across_angles: bool) -> Vec<Instant> {
    use crate::animatable::declared;
    use crate::model::Ease;

    let last = end - 1;
    let mut out: Vec<Instant> = vec![(i128::from(start), 1), (i128::from(last), 1)];
    let inside = |t: f64| t >= start as f64 && t <= last as f64;
    let mut push = |t: f64| {
        if inside(t) {
            out.push(((t * MICRO as f64).round() as i128, MICRO));
        }
    };
    let effect_reach = |member: &str, key: &str| {
        matches!(member, "blur" | "glow" | "shadow" | "directional_blur")
            && matches!(key, "radius" | "dx" | "dy" | "angle" | "length")
    };
    for property in declared(element) {
        let key = property.key();
        let relevant = match property.effect {
            None => matches!(key, "swivel" | "tilt" | "perspective" | "width" | "height"),
            Some((_, member)) => effect_reach(member, key),
        };
        let Some(records) = property.records().filter(|_| relevant) else {
            continue;
        };
        let turns = matches!(key, "swivel" | "tilt") && across_angles;
        for record in records {
            if let Some(t) = record.get("t").and_then(Value::as_f64) {
                push(t);
            }
        }
        for pair in records.windows(2) {
            let (Some(from), Some(to)) = (
                pair[0].get("t").and_then(Value::as_f64),
                pair[1].get("t").and_then(Value::as_f64),
            ) else {
                continue;
            };
            let ease = pair[1]
                .get("ease")
                .and_then(|ease| serde_json::from_value::<Ease>(ease.clone()).ok());
            for fraction in ease.iter().flat_map(crate::resolve::overshoots) {
                push(from + fraction * (to - from));
            }
            if turns {
                for fraction in [0.25, 0.5, 0.75] {
                    push(from + fraction * (to - from));
                }
            }
        }
    }
    out.sort_unstable_by(|a, b| millis(*a).total_cmp(&millis(*b)));
    out.dedup();
    out
}

/// The projection and the eye bound r at `t`, or `None` where the element is not projected or
/// has no box at that instant (so draws nothing and has no bound to clear). Reads no frame:
/// neither depends on where the element is placed.
pub(crate) fn eye_at(element: &Value, t: Instant) -> Option<(Projection, f64)> {
    let (width, height) = crate::animatable::painted_box(element, t.0, t.1)?;
    let extent = Extent { width, height };
    let projection = at(element, t)?;
    let origin = transform_of(element, t, (0, 0)).origin;
    Some((
        projection,
        eye_bound(extent, origin, &effects_at(element, t)),
    ))
}

/// Where a projected element is at one instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub projection: Projection,
    pub footprint: Footprint,
    /// ADR-0167 §5's r: the farthest corner of the reach-widened box from the origin point,
    /// in element units. `projection.perspective` must exceed it.
    pub bound: f64,
}

/// A projected element's placement at `t`, or `None` where it is not projected, has no box at
/// this instant, or has a `perspective` that does not exceed its eye bound, which
/// `E-PROJECTION-EYE` reports and the painter draws nothing for.
///
/// The footprint is the element's own: a running transition's offset is the caller's to add,
/// as it is to any other box.
pub fn placed(element: &Value, t: (i128, i128), frame: (i64, i64)) -> Option<Placed> {
    let (width, height) = crate::animatable::painted_box(element, t.0, t.1)?;
    let extent = Extent { width, height };
    let transform = transform_of(element, t, frame);
    let projection = transform.projection?;
    let effects = effects_at(element, t);
    let bound = eye_bound(extent, transform.origin, &effects);
    if !(projection.perspective > bound) {
        return None;
    }
    Some(Placed {
        projection,
        footprint: footprint(extent, &transform, &effects)?,
        bound,
    })
}

impl Placed {
    pub fn facing(&self) -> Facing {
        self.footprint.facing
    }

    /// The footprint's axis-aligned bounds as whole pixels, rounded **out** (the left and top
    /// down, the right and bottom up) so they contain everything the element paints.
    pub fn bounds(&self) -> Rect {
        let (left, top, right, bottom) = self.footprint.bounds();
        let (x, y) = (left.floor() as i64, top.floor() as i64);
        Rect {
            x,
            y,
            width: right.ceil() as i64 - x,
            height: bottom.ceil() as i64 - y,
        }
    }
}
