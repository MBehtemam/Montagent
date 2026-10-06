//! A paint field, resolved at the painter's instant into the rasterizer's [`Ink`]
//! (ADR-0149).
//!
//! The value comes from the one resolving function ([`animatable::at`]), so the painter
//! draws the paint `query --at` prints. A gradient is handed over with the declared box it
//! is measured against, in the drawing's own unscaled coordinates.

use std::sync::Arc;

use montagent_render::canvas::{Gradient as Shader, GradientKind, Ink};
use serde_json::Value;

use crate::animatable::{self, Resolved};
use crate::model::ResolvedGradient as Gradient;

use super::rgba_of;

/// The ink `property` holds on `element` at `t = (numerator, denominator)` ms, or `None`
/// where it states none that reads. `frame` is the declared box, `[x, y, width, height]`, a
/// gradient is measured against.
pub(super) fn at(element: &Value, property: &str, t: (i128, i128), frame: [f32; 4]) -> Option<Ink> {
    match animatable::read(element, property, t.0, t.1)?.ok()? {
        Resolved::Colour(colour) => rgba_of(&colour).map(Ink::Flat),
        Resolved::Gradient(gradient) => Some(Ink::Gradient(Arc::new(shader(&gradient, frame)?))),
        Resolved::Number(_) | Resolved::Pair(_) | Resolved::Points(_) | Resolved::Stops(_) => None,
    }
}

/// The rasterizer's spelling of a resolved, fixed gradient.
fn shader(gradient: &Gradient, frame: [f32; 4]) -> Option<Shader> {
    let kind = match gradient {
        Gradient::Linear { angle, .. } => GradientKind::Linear {
            angle: angle.degrees(),
        },
        Gradient::Radial { center, radius, .. } => GradientKind::Radial {
            center: center.fractions(),
            radius: radius.fraction(),
        },
    };
    let stops = gradient
        .stops()
        .iter()
        .map(|stop| Some((stop.offset.fraction() as f32, rgba_of(&stop.color)?)))
        .collect::<Option<Vec<_>>>()?;
    Some(Shader { kind, stops, frame })
}

/// The declared text box in the typographic block's own coordinates.
///
/// The renderer places the block, never the declared box (`R-BOX-SLACK`: an oversized box
/// has no rendering effect), so the box is placed by the rule the transform already uses:
/// it shares the block's pivot point. With `origin` at `(fx, fy)` of each, the box's
/// top-left sits at `(fx·(block width − W), fy·(block height − H))` — centred on the block
/// for a centre origin, sharing its top-left corner for `top-left`.
pub(super) fn text_box(origin: (f64, f64), block: (f64, f64), declared: (f64, f64)) -> [f32; 4] {
    let (fx, fy) = origin;
    [
        (fx * (block.0 - declared.0)) as f32,
        (fy * (block.1 - declared.1)) as f32,
        declared.0 as f32,
        declared.1 as f32,
    ]
}
