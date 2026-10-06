//! A gradient paint as the rasterizer draws it (ADR-0149 §2, §7).
//!
//! One Skia gradient shader per paint per frame, built from values the core has already
//! resolved and fixed: `linear` or `radial`, clamp tile mode, premultiplied interpolation,
//! with dithering left off. The geometry is measured against the element's **declared box**,
//! which the core hands over as [`Gradient::frame`] in the drawing's own unscaled
//! coordinates — `[0, 0, W, H]` for a shape, the declared text box placed in the typographic
//! block for text.

use std::sync::Arc;

use skia_safe::gradient::interpolation::{ColorSpace, HueMethod, InPremul};
use skia_safe::gradient::{self, Colors, Interpolation, shaders};
use skia_safe::{Color4f, Matrix, Paint as SkPaint, Point, Shader, TileMode};

use super::Rgba;

/// One side of a [`super::Fill`]: a flat colour, or a gradient.
#[derive(Debug, Clone, PartialEq)]
pub enum Ink {
    Flat(Rgba),
    /// Shared rather than copied: a text element's gradient is the same one on every glyph.
    Gradient(Arc<Gradient>),
}

impl From<Rgba> for Ink {
    fn from(colour: Rgba) -> Ink {
        Ink::Flat(colour)
    }
}

/// One resolved gradient paint.
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    /// `(offset, colour)`, offsets in `0..=1` and never decreasing (ADR-0149 §4's fix, which
    /// the core has applied). At least two.
    pub stops: Vec<(f32, Rgba)>,
    /// The declared box the geometry is measured against, `[x, y, width, height]`, in the
    /// drawing's own unscaled coordinates.
    pub frame: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientKind {
    /// Degrees, CSS's convention: `0` runs bottom to top, `90` left to right, clockwise.
    Linear { angle: f64 },
    /// `center` in box fractions; `radius` a fraction of the distance from the centre to
    /// the farthest corner, in box fractions.
    Radial { center: [f64; 2], radius: f64 },
}

impl Ink {
    /// The Skia paint for this ink, anti-aliased. `offset` is where the drawing's origin has
    /// been moved to before the draw — a glyph is drawn after translating to its own origin,
    /// so its gradient is moved back by the same amount: one gradient across the box, not
    /// one per letter.
    pub(super) fn paint(&self, offset: (f32, f32)) -> SkPaint {
        let mut paint = match self {
            Ink::Flat(colour) => SkPaint::new(Color4f::from(colour.colour()), None),
            Ink::Gradient(gradient) => gradient.paint(offset),
        };
        paint.set_anti_alias(true);
        paint
    }
}

impl Gradient {
    fn paint(&self, (dx, dy): (f32, f32)) -> SkPaint {
        let mut paint = SkPaint::default();
        match self.shader() {
            Some(shader) => {
                let shader = if (dx, dy) == (0.0, 0.0) {
                    shader
                } else {
                    shader.with_local_matrix(&Matrix::translate((-dx, -dy)))
                };
                paint.set_shader(shader);
            }
            // A radius at or below 0 paints the last stop's colour over the whole box, as
            // CSS does (ADR-0149 §4) — and so does a shader Skia declines to build, which
            // only a degenerate stop list reaches.
            None => {
                let last = self.stops.last().map(|(_, c)| *c).unwrap_or(Rgba::BLACK);
                paint.set_color4f(Color4f::from(last.colour()), None);
            }
        }
        // Dithering stays off (ADR-0149 §7): it would add noise that differs by position,
        // and the painter never asks for it.
        paint.set_dither(false);
        paint
    }

    /// The shader, or `None` where the gradient paints one colour over the box.
    fn shader(&self) -> Option<Shader> {
        let colours: Vec<Color4f> = self
            .stops
            .iter()
            .map(|(_, colour)| Color4f::from(colour.colour()))
            .collect();
        let offsets: Vec<f32> = self.stops.iter().map(|(offset, _)| *offset).collect();
        // Premultiplied, in the destination's space: the surface carries no colour space, so
        // that is the document's own sRGB values — the rule ADR-0146 uses across time. Clamp
        // at both ends: the first and last colours extend (ADR-0149 §2).
        let interpolation = Interpolation {
            in_premul: InPremul::Yes,
            color_space: ColorSpace::Destination,
            hue_method: HueMethod::Shorter,
        };
        let spec = gradient::Gradient::new(
            Colors::new(&colours, Some(&offsets), TileMode::Clamp, None),
            interpolation,
        );
        let [x, y, w, h] = self.frame.map(f64::from);
        match self.kind {
            GradientKind::Linear { angle } => {
                // Through the box's centre in direction (sin a, −cos a), y down, with length
                // L = |W·sin a| + |H·cos a|, so the corners land on offsets 0 and 1.
                let a = angle.to_radians();
                let (sin, cos) = a.sin_cos();
                let half = ((w * sin).abs() + (h * cos).abs()) / 2.0;
                let (cx, cy) = (x + w / 2.0, y + h / 2.0);
                let from = Point::new((cx - sin * half) as f32, (cy + cos * half) as f32);
                let to = Point::new((cx + sin * half) as f32, (cy - cos * half) as f32);
                shaders::linear_gradient((from, to), &spec, None)
            }
            GradientKind::Radial { radius, .. } if radius <= 0.0 => None,
            GradientKind::Radial {
                center: [fx, fy],
                radius,
            } => {
                let farthest = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
                    .iter()
                    .map(|(cx, cy): &(f64, f64)| (cx - fx).hypot(cy - fy))
                    .fold(0.0, f64::max);
                // A circle in box fractions, drawn through a matrix that scales by W and H.
                let local =
                    Matrix::translate((x as f32, y as f32)) * Matrix::scale((w as f32, h as f32));
                shaders::radial_gradient(
                    (Point::new(fx as f32, fy as f32), (radius * farthest) as f32),
                    &spec,
                    &local,
                )
            }
        }
    }
}
