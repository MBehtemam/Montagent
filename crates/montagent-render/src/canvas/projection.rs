//! **Projection** (ADR-0167, amended by ADR-0168): one flat element drawn as a plane turned in
//! front of an eye. The arithmetic every tool shares lives here, beside the painter that
//! draws through it, so the renderer, `validate`, `query --at` and the contact sheet read one
//! answer (ADR-0167 §6):
//!
//! - [`Projection::matrix`]: CSS's `perspective(d) rotateX(tilt) rotateY(swivel)` with the z
//!   row dropped, built in f64. The swivel turns the element first, then the tilt (ADR-0168
//!   §2).
//! - [`Projection::facing`]: whether the plane's front is toward the eye, decided in degrees
//!   so that exactly 90° is edge-on rather than a sliver.
//! - [`reach_box`] and [`eye_bound`]: the declared box widened by every effect's reach, and
//!   the distance r from the origin point to its farthest corner (ADR-0167 §5).
//! - [`quad`]: the four frame-space corners of that box, projected about `origin` and then
//!   carried through `scale`, `rotation` and `x`/`y`.

use super::{Effect, Extent, Transform, named, sigma};

/// ADR-0167's three numbers at one instant, already resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    /// Degrees about the vertical axis; positive sends the right edge away (CSS `rotateY`).
    pub swivel: f64,
    /// Degrees about the horizontal axis; positive sends the top edge away (CSS `rotateX`).
    pub tilt: f64,
    /// The eye's distance from the plane, in element units (CSS `perspective`). Infinite
    /// where the file writes an angle and no `perspective`, which `validate` refuses: the
    /// picture is then the plane seen from infinitely far, with no foreshortening.
    pub perspective: f64,
}

/// Which way a projected plane faces the eye at one instant (ADR-0167 §4, §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    /// Its front is toward the eye: it draws.
    Front,
    /// Its back is toward the eye: it draws nothing.
    Away,
    /// Exactly edge-on: it draws nothing.
    Edge,
}

impl Facing {
    /// The word `query --at` prints.
    pub fn as_str(self) -> &'static str {
        match self {
            Facing::Front => "front",
            Facing::Away => "away",
            Facing::Edge => "edge",
        }
    }
}

impl Projection {
    /// The facing: the turned plane's normal points toward the eye where
    /// `cos(swivel) · cos(tilt) > 0`. The eye is in front of the origin point and the plane
    /// passes through it, so this is the whole question, whatever `perspective` is and
    /// whichever order the angles compose in (ADR-0168 §2).
    ///
    /// Decided per angle in degrees: an angle ≡ 90° or 270° (mod 360°) is edge-on, exactly.
    pub fn facing(&self) -> Facing {
        fn side(degrees: f64) -> i8 {
            let a = degrees.rem_euclid(360.0);
            if a == 90.0 || a == 270.0 {
                0
            } else if (90.0..270.0).contains(&a) {
                -1
            } else {
                1
            }
        }
        match side(self.swivel) * side(self.tilt) {
            0 => Facing::Edge,
            1 => Facing::Front,
            _ => Facing::Away,
        }
    }

    /// The projective map of origin-relative element coordinates (on the plane, z = 0) to
    /// origin-relative picture coordinates, row-major, for column vectors `(x, y, 1)`:
    ///
    /// `[[cos s, 0, 0], [sin s·sin t, cos t, 0], [sin s·cos t/d, −sin t/d, 1]]`.
    ///
    /// That is CSS `perspective(d) rotateX(t) rotateY(s)` with the z row and column dropped:
    /// the swivel acts first (ADR-0168 §2).
    pub fn matrix(&self) -> [[f64; 3]; 3] {
        let (ss, cs) = self.swivel.to_radians().sin_cos();
        let (st, ct) = self.tilt.to_radians().sin_cos();
        let d = self.perspective;
        [
            [cs, 0.0, 0.0],
            [ss * st, ct, 0.0],
            [ss * ct / d, -st / d, 1.0],
        ]
    }

    /// One origin-relative point through [`Projection::matrix`]. `None` where it reaches or
    /// passes the eye (w ≤ 0), which ADR-0167 §5's bound keeps from happening.
    pub fn apply(&self, (x, y): (f64, f64)) -> Option<(f64, f64)> {
        let m = self.matrix();
        let w = m[2][0] * x + m[2][1] * y + m[2][2];
        (w > 0.0).then(|| {
            (
                (m[0][0] * x + m[0][1] * y + m[0][2]) / w,
                (m[1][0] * x + m[1][1] * y + m[1][2]) / w,
            )
        })
    }
}

/// The box an element's flat layer covers, in element units, as `(left, top, right, bottom)`
/// around the declared box `(0, 0, width, height)` (ADR-0167 §5): each effect's declared
/// reach, summed in list order.
///
/// - `blur` and `glow` reach ⌈3σ⌉, σ = `radius` / 2, on every side;
/// - a `shadow` is a copy of what came before it, moved by its offset and blurred, so the box
///   grows to hold that copy: the offset plus ⌈3σ⌉ on the side it falls to;
/// - a `directional_blur` reaches its crop's own reach on each axis;
/// - a `mask`, `grain`, the colour members, `posterize` and `chroma` reach nothing.
pub fn reach_box(extent: Extent, effects: &[Effect]) -> (f64, f64, f64, f64) {
    let (mut l, mut t, mut r, mut b) = (0.0_f64, 0.0_f64, extent.width, extent.height);
    let spread = |radius: f64| (3.0 * f64::from(sigma(radius))).ceil();
    for effect in effects {
        match *effect {
            Effect::Blur { radius } | Effect::Glow { radius, .. } => {
                let k = spread(radius);
                (l, t, r, b) = (l - k, t - k, r + k, b + k);
            }
            Effect::Shadow { dx, dy, radius, .. } => {
                let k = spread(radius);
                (l, t, r, b) = (
                    l.min(l + dx - k),
                    t.min(t + dy - k),
                    r.max(r + dx + k),
                    b.max(b + dy + k),
                );
            }
            Effect::DirectionalBlur { angle, length } => {
                let (rx, ry) = named::directional_reach(angle, length);
                let (rx, ry) = (f64::from(rx), f64::from(ry));
                (l, t, r, b) = (l - rx, t - ry, r + rx, b + ry);
            }
            _ => {}
        }
    }
    (l, t, r, b)
}

/// The four corners of [`reach_box`], in the box's order: top-left, top-right, bottom-right,
/// bottom-left.
fn reach_corners(extent: Extent, effects: &[Effect]) -> [(f64, f64); 4] {
    let (l, t, r, b) = reach_box(extent, effects);
    [(l, t), (r, t), (r, b), (l, b)]
}

/// **r** (ADR-0167 §5): the distance, in element units, from the origin point to the farthest
/// corner of the reach-widened box. `perspective` must exceed it, so no point the layer holds
/// reaches the eye.
pub fn eye_bound(extent: Extent, origin: (f64, f64), effects: &[Effect]) -> f64 {
    let (ox, oy) = (origin.0 * extent.width, origin.1 * extent.height);
    reach_corners(extent, effects)
        .iter()
        .map(|(x, y)| (x - ox).hypot(y - oy))
        .fold(0.0, f64::max)
}

/// One projected element's footprint at one instant (ADR-0167 §6, §7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub facing: Facing,
    /// The reach-widened box's four frame-space corners, top-left, top-right, bottom-right,
    /// bottom-left. Printed whatever the facing.
    pub corners: [(f64, f64); 4],
    /// Whether the element paints at this instant: it faces front, has a box, and its
    /// `perspective` exceeds r. Where it does not, it paints nothing, as under `opacity: 0`.
    pub drawn: bool,
}

impl Quad {
    /// The corners' axis-aligned bounds, `[left, top, right, bottom]`: what every check that
    /// reads a frame-space box reads (ADR-0167 §6). Whatever the facing, as a box is still
    /// somewhere under `opacity: 0`; `query --at`'s ink box is these only where
    /// [`Quad::drawn`].
    pub fn bounds(&self) -> [f64; 4] {
        let mut out = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
        for (x, y) in self.corners {
            out = [out[0].min(x), out[1].min(y), out[2].max(x), out[3].max(y)];
        }
        out
    }
}

/// **The projected quadrilateral**: the reach-widened box's corners projected about `origin`,
/// then scaled, rotated and placed at `x`/`y`, all in f64 (ADR-0167 §6). The painter draws
/// through the same matrix, so these are the corners of what it paints.
///
/// A corner at or behind the eye (only where `perspective` ≤ r, which `validate` refuses) is
/// left where the unprojected plane has it, and the quad is not drawn.
pub fn quad(
    extent: Extent,
    transform: &Transform,
    projection: Projection,
    effects: &[Effect],
) -> Quad {
    let (ox, oy) = (
        transform.origin.0 * extent.width,
        transform.origin.1 * extent.height,
    );
    let (sin, cos) = transform.rotation.to_radians().sin_cos();
    let facing = projection.facing();
    let mut in_front = projection.perspective > eye_bound(extent, transform.origin, effects);
    let corners = reach_corners(extent, effects).map(|(x, y)| {
        let local = (x - ox, y - oy);
        let (px, py) = projection.apply(local).unwrap_or_else(|| {
            in_front = false;
            local
        });
        let (sx, sy) = (px * transform.scale.0, py * transform.scale.1);
        (
            transform.x + sx * cos - sy * sin,
            transform.y + sx * sin + sy * cos,
        )
    });
    Quad {
        facing,
        corners,
        drawn: facing == Facing::Front
            && in_front
            && extent.width > 0.0
            && extent.height > 0.0
            && transform.scale.0 != 0.0
            && transform.scale.1 != 0.0,
    }
}
