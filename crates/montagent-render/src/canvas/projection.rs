//! Projection (ADR-0167, ADR-0168): an element drawn flat, then turned in front of an eye.
//!
//! **The one place the geometry lives.** The painter ([`paint`]) and every tool that asks
//! where a projected element is ([`footprint`], [`eye_bound`], [`reach_box`]) read the same
//! functions, so a picture and a `query --at` answer cannot drift: ADR-0146's lesson about
//! lists copied between a renderer and its checks.
//!
//! **Drawn flat first.** The element, with its effects and mask in list order, is drawn onto
//! a raster surface of its own, at the device's resolution, under a translation and a
//! positive scale only. [`Canvas::through`](super::Canvas) therefore never runs under a
//! perspective matrix, and the bounds hint, the grain plan and the directional-blur crop
//! (which each read the matrix) never see one. The finished surface is then drawn once,
//! through [`Projection::matrix`] about the element's origin point.
//!
//! **The eye sits in front of the origin point** at distance `perspective`. ADR-0167 §5
//! requires `perspective` to exceed [`eye_bound`] at every instant, which keeps every point
//! of the layer in front of the eye, so there is no behind-the-eye clipping path here. A
//! `perspective` that does not exceed it paints nothing, as a stand-in for `validate`'s
//! `E-PROJECTION-EYE`, which is what a document with one is told.

use skia_safe::surfaces;
use skia_safe::{AlphaType, Color, ColorType, ISize, ImageInfo, Matrix, Paint as SkPaint, Rect};

use super::{Effect, Extent, Transform, named, sampling_for, sigma};

/// ADR-0167's three numbers at one instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    /// Degrees about the vertical axis; positive sends the right edge away (CSS `rotateY`).
    pub swivel: f64,
    /// Degrees about the horizontal axis; positive sends the top edge away (CSS `rotateX`).
    pub tilt: f64,
    /// The eye's distance from the element's plane, in px (CSS `perspective`).
    pub perspective: f64,
}

/// Which way a projected plane faces the eye.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Front,
    Away,
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
    /// The facing, decided in degree space so that exactly 90° is edge-on rather than a
    /// sliver from `cos(90°) ≈ 6e-17`. The turned plane's normal points along
    /// `cos(swivel) · cos(tilt)` in depth, whichever order the two turns compose in, so each
    /// angle contributes a sign and the product decides. An angle that is not a number
    /// faces away.
    pub fn facing(&self) -> Facing {
        // +1 front, -1 back, 0 edge, for one angle.
        let side = |degrees: f64| -> Option<i8> {
            if !degrees.is_finite() {
                return None;
            }
            let a = degrees.rem_euclid(360.0);
            Some(if a == 90.0 || a == 270.0 {
                0
            } else if !(90.0..=270.0).contains(&a) {
                1
            } else {
                -1
            })
        };
        match (side(self.swivel), side(self.tilt)) {
            (Some(s), Some(t)) => match s * t {
                0 => Facing::Edge,
                1 => Facing::Front,
                _ => Facing::Away,
            },
            _ => Facing::Away,
        }
    }

    /// The 3×3 projective map of origin-relative element coordinates (z = 0) to
    /// origin-relative picture coordinates: CSS `perspective(d) rotateX(tilt)
    /// rotateY(swivel)` with the z row dropped, so the swivel turns the element first and the
    /// tilt turns it in its already swivelled frame (ADR-0168 §2). Row-major, in f64.
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

    /// One origin-relative point through [`Projection::matrix`], or `None` where it lands
    /// at or behind the eye (the homogeneous weight is not positive).
    fn apply(&self, (x, y): (f64, f64)) -> Option<(f64, f64)> {
        let m = self.matrix();
        let w = m[2][0] * x + m[2][1] * y + m[2][2];
        (w > 0.0).then(|| {
            (
                (m[0][0] * x + m[0][1] * y) / w,
                (m[1][0] * x + m[1][1] * y) / w,
            )
        })
    }
}

/// The reach-widened box an element's flat layer covers, in element units: the declared box
/// `(0, 0, width, height)` widened by each effect's reach, **summed in list order**.
///
/// - `blur` and `glow` reach `⌈3σ⌉` on every side, with σ = radius / 2;
/// - a `shadow` reaches where its blurred copy falls: its offset and `⌈3σ⌉` past the box on
///   the side it is cast to, and `⌈3σ⌉` less the offset on the other (never less than
///   nothing);
/// - a `directional_blur` reaches [`named::directional_reach`] per axis;
/// - `mask`, `grain`, colour members and `chroma` reach nothing.
///
/// This is also what the bounds hint and the directional crop measure, so the surface the
/// element is drawn flat onto holds everything its effect chain can paint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReachBox {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// See [`ReachBox`].
pub fn reach_box(extent: Extent, effects: &[Effect]) -> ReachBox {
    let mut b = ReachBox {
        left: 0.0,
        top: 0.0,
        right: extent.width,
        bottom: extent.height,
    };
    for effect in effects {
        match *effect {
            Effect::Blur { radius } | Effect::Glow { radius, .. } => {
                let k = (3.0 * f64::from(sigma(radius))).ceil();
                b = ReachBox {
                    left: b.left - k,
                    top: b.top - k,
                    right: b.right + k,
                    bottom: b.bottom + k,
                };
            }
            Effect::Shadow { dx, dy, radius, .. } => {
                let k = (3.0 * f64::from(sigma(radius))).ceil();
                b = ReachBox {
                    left: b.left.min(b.left + dx - k),
                    top: b.top.min(b.top + dy - k),
                    right: b.right.max(b.right + dx + k),
                    bottom: b.bottom.max(b.bottom + dy + k),
                };
            }
            Effect::DirectionalBlur { angle, length } => {
                let (rx, ry) = named::directional_reach(angle, length);
                let (rx, ry) = (f64::from(rx), f64::from(ry));
                b = ReachBox {
                    left: b.left - rx,
                    top: b.top - ry,
                    right: b.right + rx,
                    bottom: b.bottom + ry,
                };
            }
            _ => {}
        }
    }
    b
}

impl ReachBox {
    /// The four corners in the box's order: top-left, top-right, bottom-right, bottom-left.
    fn corners(&self) -> [(f64, f64); 4] {
        [
            (self.left, self.top),
            (self.right, self.top),
            (self.right, self.bottom),
            (self.left, self.bottom),
        ]
    }
}

/// ADR-0167 §5's r: the distance from the origin point to the farthest corner of the
/// reach-widened box, in element units. `perspective` must exceed it at every instant.
pub fn eye_bound(extent: Extent, origin: (f64, f64), effects: &[Effect]) -> f64 {
    let (ox, oy) = (origin.0 * extent.width, origin.1 * extent.height);
    reach_box(extent, effects)
        .corners()
        .iter()
        .map(|(x, y)| (x - ox).hypot(y - oy))
        .fold(0.0, f64::max)
}

/// Where a projected element is at one instant: which way it faces and the frame-space
/// quadrilateral of its reach-widened box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Footprint {
    pub facing: Facing,
    /// The four corners in the box's order: top-left, top-right, bottom-right, bottom-left.
    /// Projected about `origin`, then carried through `scale`, `rotation` and `x`/`y`, in f64.
    pub corners: [(f64, f64); 4],
}

impl Footprint {
    /// The axis-aligned bounds of [`Footprint::corners`], as `(left, top, right, bottom)`.
    /// The quadrilateral is convex (every corner is in front of the eye), so these contain
    /// everything the element paints.
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let xs = self.corners.map(|c| c.0);
        let ys = self.corners.map(|c| c.1);
        (
            xs.into_iter().fold(f64::INFINITY, f64::min),
            ys.into_iter().fold(f64::INFINITY, f64::min),
            xs.into_iter().fold(f64::NEG_INFINITY, f64::max),
            ys.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
}

/// The footprint of a projected element, or `None` for one with no projection or whose
/// `perspective` leaves a corner at or behind the eye (`E-PROJECTION-EYE`'s case).
pub fn footprint(extent: Extent, transform: &Transform, effects: &[Effect]) -> Option<Footprint> {
    let projection = transform.projection?;
    let (ox, oy) = (
        transform.origin.0 * extent.width,
        transform.origin.1 * extent.height,
    );
    let (sin, cos) = transform.rotation.to_radians().sin_cos();
    let mut corners = [(0.0, 0.0); 4];
    for (slot, (x, y)) in corners.iter_mut().zip(reach_box(extent, effects).corners()) {
        let (px, py) = projection.apply((x - ox, y - oy))?;
        let (sx, sy) = (px * transform.scale.0, py * transform.scale.1);
        *slot = (
            transform.x + sx * cos - sy * sin,
            transform.y + sx * sin + sy * cos,
        );
    }
    Some(Footprint {
        facing: projection.facing(),
        corners,
    })
}

/// Transparent layer pixels kept around the flat layer, so its edge is a mipmapped fade into
/// transparent rather than the image's clamped border.
const FLAT_PAD: f64 = 2.0;

/// The most layer pixels the flat surface spans on one side. Past it the layer is drawn at a
/// proportionally lower resolution rather than not at all.
const MAX_SIDE: f64 = 16384.0;

/// Draw one element through its projection. `canvas` already holds the base scale,
/// translate, rotate and scale (the element's 2D transform) and `base` is the base scale.
///
/// 1. Facing away or edge-on draws nothing (ADR-0167 §4). Neither does a `perspective` that
///    does not exceed [`eye_bound`].
/// 2. The element is drawn **flat**, effects and mask in list order, on a surface covering
///    [`reach_box`] plus [`FLAT_PAD`], clipped to the reach box (ink outside it, such as
///    overflowing text, is cut, so [`footprint`] bounds everything painted), at `|scale|`
///    layer pixels per element unit.
/// 3. That surface is drawn once through PROJECT · origin offset · 1 / `|scale|`, sampled by
///    [`sampling_for`] (ADR-0132) and antialiased at its edge.
pub(super) fn paint(
    canvas: &skia_safe::Canvas,
    extent: Extent,
    transform: &Transform,
    projection: Projection,
    base: (f32, f32),
    effects: &[Effect],
    draw: &dyn Fn(&skia_safe::Canvas),
) {
    if projection.facing() != Facing::Front
        || !(projection.perspective > eye_bound(extent, transform.origin, effects))
    {
        return;
    }
    let (mut kx, mut ky) = (
        transform.scale.0.abs() * f64::from(base.0),
        transform.scale.1.abs() * f64::from(base.1),
    );
    if !(kx > 0.0 && ky > 0.0 && kx.is_finite() && ky.is_finite()) {
        return;
    }
    let reach = reach_box(extent, effects);
    let side = |lo: f64, hi: f64, k: f64| (hi - lo) * k + 2.0 * (FLAT_PAD + 1.0);
    if side(reach.left, reach.right, kx) > MAX_SIDE {
        kx *= MAX_SIDE / side(reach.left, reach.right, kx);
    }
    if side(reach.top, reach.bottom, ky) > MAX_SIDE {
        ky *= MAX_SIDE / side(reach.top, reach.bottom, ky);
    }
    // Whole layer pixels on every side, so the box's top-left lands at a whole-pixel
    // translation of the flat surface.
    let pad_left = (-reach.left * kx).ceil() + FLAT_PAD;
    let pad_top = (-reach.top * ky).ceil() + FLAT_PAD;
    let width = (pad_left + (reach.right * kx).ceil() + FLAT_PAD) as i32;
    let height = (pad_top + (reach.bottom * ky).ceil() + FLAT_PAD) as i32;
    let info = ImageInfo::new(
        ISize::new(width, height),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let Some(mut flat) = surfaces::raster(&info, None, None) else {
        return;
    };
    {
        let layer = flat.canvas();
        layer.clear(Color::TRANSPARENT);
        layer.translate((pad_left as f32, pad_top as f32));
        layer.scale((kx as f32, ky as f32));
        layer.clip_rect(
            Rect::from_ltrb(
                reach.left as f32,
                reach.top as f32,
                reach.right as f32,
                reach.bottom as f32,
            ),
            None,
            Some(true),
        );
        super::Canvas::through(layer, extent, effects, draw);
    }
    let image = flat.image_snapshot();

    let m = projection.matrix();
    let project = Matrix::new_all(
        m[0][0] as f32,
        m[0][1] as f32,
        m[0][2] as f32,
        m[1][0] as f32,
        m[1][1] as f32,
        m[1][2] as f32,
        m[2][0] as f32,
        m[2][1] as f32,
        m[2][2] as f32,
    );
    canvas.save();
    canvas.concat(&project);
    canvas.translate((
        (-transform.origin.0 * extent.width) as f32,
        (-transform.origin.1 * extent.height) as f32,
    ));
    canvas.scale(((1.0 / kx) as f32, (1.0 / ky) as f32));
    canvas.translate((-pad_left as f32, -pad_top as f32));
    let to_device = canvas.local_to_device_as_3x3();
    let mut paint = SkPaint::default();
    paint.set_anti_alias(true);
    canvas.draw_image_with_sampling_options(&image, (0, 0), sampling_for(&to_device), Some(&paint));
    canvas.restore();
}
