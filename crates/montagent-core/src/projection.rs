//! **Projection** (ADR-0167, amended by ADR-0168), read off the document: whether an element
//! is projected, its three numbers at an instant, and **the one geometry function** every
//! tool asks for its frame-space footprint.
//!
//! The arithmetic itself is the rasterizer's ([`montagent_render::canvas::quad`]), the same
//! function the painter draws through, so the picture, `validate`, `query --at` and the
//! contact sheet cannot disagree about where a projected element is (ADR-0167 §6). This
//! module only resolves what that function is handed: the transform the painter places the
//! element with, the box, and the effects whose reach widens it, each through the one
//! resolving function (ADR-0146).
//!
//! **A written angle always projects** (ADR-0168 §1): an element carrying `swivel` or `tilt`,
//! at any value and even where both are 0 at an instant, takes the projected path. An element
//! with neither is untouched, whatever `perspective` holds.

use montagent_render::canvas::{Effect, Extent, Projection, Quad, Transform};
use serde_json::Value;

use crate::animatable;
use crate::model::Origin;
use crate::verbs::query::geometry;

/// The three fields, in the schema's order.
pub const FIELDS: [&str; 3] = ["swivel", "tilt", "perspective"];

/// The two angles: an element is projected where it writes either (ADR-0168 §1).
pub const ANGLES: [&str; 2] = ["swivel", "tilt"];

/// Whether `element` writes `swivel` or `tilt`, static or keyed, at any value.
pub fn projects(element: &Value) -> bool {
    ANGLES.iter().any(|angle| element.get(*angle).is_some())
}

/// The projection at `t = (numerator, denominator)` ms, where the element is projected. An
/// angle that is not written is 0; a `perspective` that is not written (`validate`'s
/// `E-PROJECTION-PERSPECTIVE-MISSING`) is infinitely far, so the picture is not
/// foreshortened.
pub fn at(element: &Value, t: (i128, i128)) -> Option<Projection> {
    if !projects(element) {
        return None;
    }
    let number = |key, default| animatable::number_read(element, key, t.0, t.1, default);
    Some(Projection {
        swivel: number("swivel", 0.0),
        tilt: number("tilt", 0.0),
        perspective: number("perspective", f64::INFINITY),
    })
}

/// `origin`'s two fractions, `center` where it is absent or unreadable.
pub(crate) fn origin_of(element: &Value) -> (f64, f64) {
    let origin = match element.get("origin") {
        None | Some(Value::Null) => Origin::Center,
        Some(value) => serde_json::from_value(value.clone()).unwrap_or(Origin::Center),
    };
    geometry::origin_fraction(origin)
}

/// The transform the painter places `element` with at `t`, before anything a transition does
/// to it: ADR-0012's defaults, the declared `opacity`, `blend` `normal`, and the projection.
/// [`crate::verbs::frame`]'s painter starts from this and adds a running transition's offset,
/// fade and the element's blend.
pub(crate) fn placed(element: &Value, t: (i128, i128), frame: (i64, i64)) -> Transform {
    let (frame_width, frame_height) = frame;
    let [sx, sy] = geometry::number_at::<[f64; 2]>(element, "scale", t, [1.0, 1.0]);
    Transform {
        x: geometry::number_at::<i64>(element, "x", t, frame_width as f64 / 2.0),
        y: geometry::number_at::<i64>(element, "y", t, frame_height as f64 / 2.0),
        origin: origin_of(element),
        scale: (sx, sy),
        rotation: geometry::number_at::<f64>(element, "rotation", t, 0.0),
        opacity: geometry::number_at::<f64>(element, "opacity", t, 1.0),
        blend: montagent_render::canvas::Blend::Normal,
        projection: at(element, t),
        nest: crate::nest::composed(element, t).map(|m| m.as_array()),
    }
}

/// The element's `effects` at `t`, as the painter reads them: every member the model reads,
/// in list order, each parameter through the one resolving function. A member the model
/// refuses is skipped, as the painter skips it.
pub(crate) fn effects_at(element: &Value, t: (i128, i128)) -> Vec<Effect> {
    let count = element
        .get("effects")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    (0..count)
        .filter_map(|index| crate::verbs::frame::effect_of(element, index, t, 0))
        .collect()
}

/// **The one geometry function** (ADR-0167 §6, §7): a projected element's facing, the four
/// frame-space corners of its box widened by every effect's reach, and whether it paints at
/// `t`. Their bounds are [`Quad::bounds`].
///
/// The box is the declared `width` × `height`, resolved at `t`. `None` where the element is
/// not projected, or has no box with a positive size at `t` (it paints nothing then, for its
/// own reason). `offset` is a running slide or push's, in whole frame pixels.
pub fn quad_at(
    element: &Value,
    t: (i128, i128),
    frame: (i64, i64),
    offset: (i64, i64),
) -> Option<Quad> {
    let projection = at(element, t)?;
    let (width, height) = animatable::painted_box(element, t.0, t.1)?;
    let mut transform = placed(element, t, frame);
    transform.x += offset.0 as f64;
    transform.y += offset.1 as f64;
    Some(montagent_render::canvas::quad(
        Extent { width, height },
        &transform,
        projection,
        &effects_at(element, t),
    ))
}

/// [`quad_at`] at a whole millisecond, with no transition.
pub fn quad(element: &Value, instant: i64, frame: (i64, i64)) -> Option<Quad> {
    quad_at(element, (i128::from(instant), 1), frame, (0, 0))
}
