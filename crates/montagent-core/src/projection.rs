//! Projection (ADR-0167, ADR-0168): what the document says about an element's `swivel`,
//! `tilt` and `perspective`, read once for every tool.
//!
//! The painter, `validate`, `query --at` and the contact sheet all ask the same two
//! questions of an element: is it projected, and with what at this instant. They are answered
//! here, and the geometry that follows (facing, corners) is
//! [`montagent_render::canvas::footprint`], so a picture and a report are one reading
//! (ADR-0146's lesson about copied lists).

use montagent_render::canvas::Projection;
use serde_json::Value;

use crate::verbs::query::geometry::number_at;

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

/// The projection at `t` milliseconds, `(numerator, denominator)`: an instant that need not
/// be a whole millisecond, as a motion-blur sample's is (ADR-0155 §3).
///
/// `None` for an element that writes no angle, and for one that writes an angle but no
/// `perspective`, which has no eye to turn in front of (`E-PROJECTION-PERSPECTIVE-MISSING`
/// is what a document like that is told; it paints as if unprojected). An angle that is
/// written once and not the other reads as `0`.
pub fn at(element: &Value, t: (i128, i128)) -> Option<Projection> {
    if !is_projected(element) {
        return None;
    }
    element
        .get("perspective")
        .filter(|value| !value.is_null())?;
    Some(Projection {
        swivel: number_at::<f64>(element, "swivel", t, 0.0),
        tilt: number_at::<f64>(element, "tilt", t, 0.0),
        perspective: number_at::<f64>(element, "perspective", t, f64::INFINITY),
    })
}
