//! Motion along an arc-length parameterised path (ADR-0218).
//!
//! [`at`] is the single evaluator for `motion_path`: the painter, `query --at`, geometry queries,
//! and static checks (`R-OFF-CANVAS`, `R-MOTION-BLUR-STILL`, `R-BLEND-BACKGROUND-ONLY`,
//! `R-SOURCE-CUT-POP`) all read position and tangent orientation through it.

use montagent_render::canvas::{Curve, PathEl};
use serde_json::Value;

use crate::animatable;
use crate::model::Points;
use crate::resolve::VertexAt;
use crate::verbs::frame::outline_of as frame_outline_of;
use crate::verbs::query::geometry;

/// A resolved sample on an element's `motion_path` at one instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Sample {
    /// The resolved `motion_path.offset` value (after clamping to the legal range).
    pub offset: f64,
    /// The normalised arc-length parameter `u` in `[0, 1]` (open) or `[0, 1)` (closed).
    pub u: f64,
    /// Total arc length `L` of the motion path in pixels.
    pub length: f64,
    /// Target distance `d = u * L` along the motion path in pixels.
    pub distance: f64,
    /// Resolved pivot `x` coordinate in canvas pixels.
    pub x: f64,
    /// Resolved pivot `y` coordinate in canvas pixels.
    pub y: f64,
    /// Forward tangent angle `atan2(dy, dx)` in degrees (`[-180, 180]`).
    pub tangent_deg: f64,
    /// Whether `motion_path.orient` is `true`.
    pub orient: bool,
}

impl Sample {
    /// Effective rotation in degrees: `base_rotation + tangent_deg` when `orient` is `true`,
    /// otherwise `base_rotation`.
    pub(crate) fn effective_rotation(&self, base_rotation: f64) -> f64 {
        if self.orient {
            base_rotation + self.tangent_deg
        } else {
            base_rotation
        }
    }
}

/// Whether `element` is a visual element carrying a `motion_path` object (ADR-0218 §1).
pub(crate) fn carries_motion_path(element: &Value) -> bool {
    geometry::covers_the_frame(element.get("type").and_then(Value::as_str))
        && element.get("motion_path").is_some_and(Value::is_object)
}

/// Build the [`PathEl`] outline of a `motion_path` object, if its `closed` flag and
/// `points` list are structurally valid (`>= 2` vertices and no dangling endpoint handles on an
/// open path).
pub(crate) fn outline_of(mp: &Value) -> Option<(bool, Vec<PathEl>)> {
    let closed = mp.get("closed")?.as_bool()?;
    let points: Points = serde_json::from_value(mp.get("points")?.clone()).ok()?;
    if points.0.len() < 2 {
        return None;
    }
    if !closed
        && (points.0.first().is_some_and(|v| v.arriving.is_some())
            || points.0.last().is_some_and(|v| v.out.is_some()))
    {
        return None;
    }
    let vertices: Vec<VertexAt> = points
        .0
        .iter()
        .map(|v| VertexAt {
            at: [v.at[0] as f64, v.at[1] as f64],
            arriving: v.arriving.map(|[dx, dy]| [dx as f64, dy as f64]),
            out: v.out.map(|[dx, dy]| [dx as f64, dy as f64]),
        })
        .collect();
    Some((closed, frame_outline_of(&vertices, closed)))
}

/// Evaluate `element`'s `motion_path` at rational millisecond instant `t = (num, den)`.
///
/// Returns `None` if `element` does not carry a valid positive-length `motion_path`.
pub(crate) fn at(element: &Value, t: (i128, i128)) -> Option<Sample> {
    if !carries_motion_path(element) {
        return None;
    }
    let mp = element.get("motion_path")?;
    let (closed, outline) = outline_of(mp)?;
    let curve = Curve::of(&outline);
    if curve.length <= 0.0 || curve.length.is_nan() {
        return None;
    }
    let offset = animatable::number_read(element, "motion_path.offset", t.0, t.1, 0.0);
    let u = if closed {
        offset.clamp(-1.0, 2.0).rem_euclid(1.0)
    } else {
        offset.clamp(0.0, 1.0)
    };
    let distance = u * curve.length;
    let (x, y, theta_rad) = curve.at(distance)?;
    let orient = mp.get("orient").and_then(Value::as_bool).unwrap_or(false);
    Some(Sample {
        offset,
        u,
        length: curve.length,
        distance,
        x,
        y,
        tangent_deg: theta_rad.to_degrees(),
        orient,
    })
}
