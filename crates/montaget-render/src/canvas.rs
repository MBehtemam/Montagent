//! The rasterizer: a Skia surface, the flat transform ADR-0012 declares, and the two
//! primitives ADR-0014 names.
//!
//! **This module knows nothing about the document.** It takes numbers that are already
//! resolved — an instant's `x`, `y`, `scale`, `rotation` and `opacity`, a declared box, a
//! colour — and paints them. Every rule about *where those numbers come from* (which
//! keyframe record, which default, which element is present) lives in `montaget-core`,
//! which is the crate that reads the format. The split is the one that keeps
//! `montaget-core` able to depend on this crate at all: a rasterizer that read the
//! document would have to depend back on the core, and there would be no bottom to the
//! tree (ADR-0011's four-crate split, spec #168's workspace shape).
//!
//! Four rules are implemented here rather than stated anywhere else, because they are
//! rules about *painting*:
//!
//! - **The transform pivots about `origin`.** ADR-0013 spells the nine keywords as *"the
//!   nine-way point of an element's own box that its `x`,`y` places, and about which
//!   transforms pivot"*, so the canvas is translated to `(x, y)`, rotated, scaled, and
//!   only then moved back by the origin's fraction of the box. Placing the box first and
//!   rotating afterwards would pivot about the box's top-left and put every rotated
//!   element somewhere else.
//! - **`scale` is applied as a coordinate scale, not by pre-multiplying the box.** That is
//!   what makes ADR-0014's *"stroke is in element space and scales with `scale`"* fall out
//!   rather than be arranged: a `stroke_width` of 8 drawn in the scaled space is 8.64 px
//!   at the fixture's 1.08 peak, which is the worked number the ADR ships.
//! - **A shape's stroke falls inside its declared rect** (ADR-0014). Skia centres a stroke
//!   on the path, so the path is inset by half the width — `card-05` with `stroke_width:
//!   8` still occupies exactly 984×169.
//! - **Rotation is applied before scale**, so an element is scaled in its own axes and the
//!   result is turned — the `translate → rotate → scale` order a CSS `transform` list
//!   spells and the one CapCut and Premiere's Motion panel show. It is invisible while
//!   `scale` is isotropic, which is every keyframe in the fixture, and visible the moment
//!   `sx != sy` — which ADR-0012 makes the *only* spelling there is, by requiring the pair.
//!   No ADR states the order; it is recorded here and raised as
//!   [#274](https://github.com/MBehtemam/Montaget/issues/274) rather than left to be
//!   discovered from a stretched, tilted card.
//! - **`opacity` is one layer, not a per-paint alpha.** An element with a fill *and* an
//!   inside stroke overlaps itself, and multiplying alpha into both paints would blend the
//!   overlap twice and darken the stroke's inner edge. A `save_layer_alpha` composites the
//!   whole element once.
//!
//! What is **not** here, and is a later ticket rather than an omission: glyph painting
//! ([#213](https://github.com/MBehtemam/Montaget/issues/213)) and the effect vocabulary,
//! colour filters, transitions and highlight
//! ([#214](https://github.com/MBehtemam/Montaget/issues/214)).

use skia_safe::{
    AlphaType, Color, Color4f, ColorType, Data, EncodedImageFormat, ISize, Image, ImageInfo,
    Paint as SkPaint, PaintStyle, Rect, SamplingOptions, Surface, images, surfaces,
};

/// `#RRGGBBAA`, already parsed. The format's own colour spelling is the core's to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    /// Opaque black — what a project that declares no `background` is painted on.
    ///
    /// **A decision, recorded rather than defaulted into silently.** No ADR states a
    /// background default, and ADR-0030 makes an absent field *"give me whatever the
    /// default is"* rather than a transparent one. Transparent would be the other candidate
    /// and is worse here for a reason specific to this verb: `frame`'s default encoding is
    /// JPEG, which carries no alpha, so a transparent default would reach the agent as
    /// black anyway — while making PNG and JPEG of the same instant disagree. Opaque black
    /// makes the two forms the same picture. Raised as
    /// [#274](https://github.com/MBehtemam/Montaget/issues/274) rather than left to be
    /// discovered from the code.
    pub const BLACK: Rgba = Rgba([0x00, 0x00, 0x00, 0xFF]);

    fn colour(self) -> Color {
        let [r, g, b, a] = self.0;
        Color::from_argb(a, r, g, b)
    }
}

/// The resolved transform at one instant — every value already interpolated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    /// `origin`'s two fractions of the element's own box, `(horizontal, vertical)` — `0` at
    /// left/top, `0.5` at centre, `1` at right/bottom. The nine keywords are the core's to
    /// spell; what reaches the canvas is the pair they mean.
    ///
    /// There is deliberately no `Default` for this type. ADR-0012 publishes a default for
    /// every one of these properties, but two of them — `x` and `y`, at the frame's centre
    /// — need the frame to state them, and this crate does not know the frame. A `Default`
    /// that filled in four of six and quietly put the other two at the origin would be a
    /// plausible wrong answer, which is the failure ADR-0012 chose centre over top-left to
    /// avoid.
    pub origin: (f64, f64),
    pub scale: (f64, f64),
    /// Degrees clockwise, never normalised into `[0,360)` (ADR-0012).
    pub rotation: f64,
    pub opacity: f64,
}

/// The declared `width`×`height` box, before `scale`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extent {
    pub width: f64,
    pub height: f64,
}

/// An axis-aligned frame-space rectangle in pixels — `clip`, and `--crop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

impl Region {
    fn rect(self) -> Rect {
        Rect::from_xywh(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

/// The two shapes ADR-0014 admits. An ellipse inscribes its declared rect, which is
/// exactly what distinguishes it from the point-list shapes that ADR rejected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// `radius` is a single integer in element space, defaulting to 0 (ADR-0014).
    Rect {
        radius: f64,
    },
    Ellipse,
}

/// A shape's paint. `fill` may be absent when `stroke` is present, giving an outlined
/// shape; a shape with neither is a schema error the core reports, and paints nothing here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    pub fill: Option<Rgba>,
    pub stroke: Option<Rgba>,
    pub stroke_width: f64,
}

/// A decoded raster source, ready to be resampled into an element's declared box.
///
/// It carries no dimensions of its own, and that is the point rather than an omission:
/// ADR-0013 settled that a source is resampled to exactly the declared `width`x`height`, so
/// nothing in the paint path asks how big the file was. What *does* ask — `fit`'s
/// arithmetic — gets its answer from `probe`, which ADR-0011 makes the one authority on a
/// media file's numbers. A second dimension pair on this type would be a second one.
pub struct Raster {
    image: Image,
}

impl Raster {
    /// Decode an encoded still — PNG, JPEG, whatever the linked codecs read — **with its
    /// EXIF orientation already applied**.
    ///
    /// Skia's own codecs rather than a second decoding crate: the rasterizer already links
    /// them, and two decoders for one file is two answers to *"what are this file's
    /// pixels"* — the ambiguity ADR-0011 spent itself removing one field over.
    ///
    /// **The orientation is load-bearing and is not applied here**, which is worth a
    /// sentence because the opposite reading is the obvious one. ADR-0015 defines source
    /// dimensions as *"decoded, **orientation-applied** integer pixel dimensions"*, so
    /// `validate`'s `fit` arithmetic is already done against the oriented frame; a renderer
    /// painting the *stored* pixels would put the format in the state ADR-0023 names as the
    /// thing to avoid — *"the format disagrees with itself"* — with `validate` green on a
    /// frame that is visibly wrong.
    ///
    /// `Image::from_encoded` applies it. That was **measured, not assumed**: an explicit
    /// second application was written here first, and
    /// `a_stills_exif_orientation_is_applied_before_it_is_painted` caught it as a
    /// *double* rotation, landing the stored bottom-right quadrant where the bottom-left
    /// belonged. That test is what holds this — a codec change that stopped applying the
    /// tag fails it, and so does a second application added back.
    pub fn decode(bytes: &[u8]) -> Option<Raster> {
        Some(Raster {
            image: Image::from_encoded(Data::new_copy(bytes))?,
        })
    }

    /// Wrap already-decoded RGBA8 pixels — one video frame, as `ffmpeg` hands it over.
    pub fn from_rgba(rgba: &[u8], width: u32, height: u32) -> Option<Raster> {
        if rgba.len() != (width as usize) * (height as usize) * 4 {
            return None;
        }
        let info = ImageInfo::new(
            ISize::new(width as i32, height as i32),
            ColorType::RGBA8888,
            AlphaType::Unpremul,
            None,
        );
        Some(Raster {
            image: images::raster_from_data(&info, Data::new_copy(rgba), width as usize * 4)?,
        })
    }
}

/// Which of the two encodings one `frame` answers in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// The default. ADR-0011: *"JPEG is still preferred, purely for latency and disk"* —
    /// encoding format is irrelevant to an agent's context cost, which is a pure function
    /// of decoded pixel dimensions.
    Jpeg,
    Png,
}

impl Encoding {
    pub fn name(self) -> &'static str {
        match self {
            Encoding::Jpeg => "jpeg",
            Encoding::Png => "png",
        }
    }

    fn format(self) -> EncodedImageFormat {
        match self {
            Encoding::Jpeg => EncodedImageFormat::JPEG,
            Encoding::Png => EncodedImageFormat::PNG,
        }
    }
}

/// JPEG quality, stated once.
///
/// Not a measured optimum: ADR-0011 establishes that the encoding does not affect what the
/// picture costs an agent, so quality trades disk and latency against artefacts and nothing
/// else. 85 is the usual place that trade is made, and it is a constant rather than a
/// literal so that the day it is measured there is one line to change.
pub const JPEG_QUALITY: u32 = 85;

/// Whether the answer comes back at the project's true pixels or at half of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// The default. ADR-0011: full scale *"genuinely costs 2691 tokens every time the agent
    /// looks"* against 700 at half — 3.84×, tracking the true 4× pixel area.
    Half,
    /// Behind a flag. The caller asked for true pixels and is paying for them.
    Full,
}

impl Scale {
    pub fn name(self) -> &'static str {
        match self {
            Scale::Half => "half",
            Scale::Full => "full",
        }
    }

    /// The output extent for a region of `pixels` along one axis.
    ///
    /// Rounds up and floors at one: an odd-width crop must not come back a pixel short of
    /// half, and a 1 px crop must not come back zero-sized — an empty image is not a
    /// smaller picture, it is no picture.
    pub fn apply(self, pixels: i64) -> i64 {
        match self {
            Scale::Full => pixels.max(1),
            Scale::Half => ((pixels + 1) / 2).max(1),
        }
    }
}

/// One encoded picture, and the dimensions it decodes to — which is what it costs an agent
/// to look at (ADR-0011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub width: i64,
    pub height: i64,
    pub encoding: Encoding,
    pub scale: Scale,
}

impl Encoded {
    /// The MIME type an MCP image content block carries.
    pub fn mime_type(&self) -> &'static str {
        match self.encoding {
            Encoding::Jpeg => "image/jpeg",
            Encoding::Png => "image/png",
        }
    }
}

/// The surface one frame is painted on, always at the project's **true** pixel dimensions.
///
/// ADR-0021 is explicit that `frame` is never proxy-scaled: the half-scale default is a
/// property of the *answer*, applied once at encode time, not of the raster. Painting at
/// half and reporting a frame would make the picture a different picture from the one
/// `render` produces, which is the whole thing this verb exists to let an agent believe.
pub struct Canvas {
    surface: Surface,
    width: i32,
    height: i32,
}

impl Canvas {
    /// A CPU raster surface at true pixel dimensions, or `None` for a frame no surface can
    /// be made at — a zero or negative dimension, or one large enough to overflow the
    /// allocation. Which of those a project's `frame` is, is `validate`'s fact to report.
    pub fn new(width: i64, height: i64) -> Option<Canvas> {
        if width <= 0 || height <= 0 || width > i32::MAX as i64 || height > i32::MAX as i64 {
            return None;
        }
        let (width, height) = (width as i32, height as i32);
        let info = ImageInfo::new(
            ISize::new(width, height),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        Some(Canvas {
            surface: surfaces::raster(&info, None, None)?,
            width,
            height,
        })
    }

    /// Paint the whole frame one colour — the project's `background`.
    pub fn background(&mut self, colour: Rgba) {
        self.surface.canvas().clear(colour.colour());
    }

    /// Draw one `rect` or `ellipse` (ADR-0014).
    pub fn shape(
        &mut self,
        shape: Shape,
        extent: Extent,
        transform: &Transform,
        paint: &Fill,
        clip: Option<Region>,
    ) {
        if paint.fill.is_none() && paint.stroke.is_none() {
            return;
        }
        self.in_element_space(extent, transform, clip, |canvas| {
            let box_rect = Rect::from_xywh(0.0, 0.0, extent.width as f32, extent.height as f32);

            if let Some(colour) = paint.fill {
                let mut fill = SkPaint::new(Color4f::from(colour.colour()), None);
                fill.set_anti_alias(true);
                fill.set_style(PaintStyle::Fill);
                match shape {
                    Shape::Rect { radius } if radius > 0.0 => {
                        canvas.draw_round_rect(box_rect, radius as f32, radius as f32, &fill);
                    }
                    Shape::Rect { .. } => {
                        canvas.draw_rect(box_rect, &fill);
                    }
                    Shape::Ellipse => {
                        canvas.draw_oval(box_rect, &fill);
                    }
                }
            }

            let (Some(colour), true) = (paint.stroke, paint.stroke_width > 0.0) else {
                return;
            };
            // ADR-0014: on a shape the stroke falls **inside** the declared rect. Skia
            // centres a stroke on the path, so the path is the box inset by half the
            // width — "centred or outside, it would occupy 992x177 at (44,1449), eating
            // 4 px of the 48 px margin the layout rests on".
            let inset = paint.stroke_width as f32 / 2.0;
            let mut stroke = SkPaint::new(Color4f::from(colour.colour()), None);
            stroke.set_anti_alias(true);
            stroke.set_style(PaintStyle::Stroke);
            stroke.set_stroke_width(paint.stroke_width as f32);
            let path = Rect::from_ltrb(
                box_rect.left + inset,
                box_rect.top + inset,
                box_rect.right - inset,
                box_rect.bottom - inset,
            );
            if path.right <= path.left || path.bottom <= path.top {
                // A stroke wider than the box it must fall inside has no inset path to
                // draw. Filling the box would be a different picture from the one the
                // document declares, so nothing is drawn and the core says so.
                return;
            }
            match shape {
                Shape::Rect { radius } if radius > 0.0 => {
                    // The inset path's corners are the declared radius less the inset, so
                    // the stroke's *outer* edge keeps the radius that was written.
                    let radius = (radius as f32 - inset).max(0.0);
                    canvas.draw_round_rect(path, radius, radius, &stroke);
                }
                Shape::Rect { .. } => {
                    canvas.draw_rect(path, &stroke);
                }
                Shape::Ellipse => {
                    canvas.draw_oval(path, &stroke);
                }
            }
        });
    }

    /// Draw a decoded raster source resampled to exactly the declared box.
    ///
    /// **Never cropped, and `fit` is never consulted** — ADR-0015 is explicit that the
    /// declared rect is authoritative at render and that `fit` *"never executes"*, and
    /// ADR-0013 settled that the source is resampled to exactly `width`×`height`. What
    /// crops is `clip`, in frame space, which is applied around the transform rather than
    /// inside it.
    pub fn raster(
        &mut self,
        source: &Raster,
        extent: Extent,
        transform: &Transform,
        clip: Option<Region>,
    ) {
        self.in_element_space(extent, transform, clip, |canvas| {
            let destination = Rect::from_xywh(0.0, 0.0, extent.width as f32, extent.height as f32);
            let paint = SkPaint::default();
            canvas.draw_image_rect_with_sampling_options(
                &source.image,
                None,
                destination,
                sampling(),
                &paint,
            );
        });
    }

    /// Establish one element's own coordinate space and run `draw` inside it.
    ///
    /// The order is the whole of ADR-0012's placement rule: `clip` is frame-space and so
    /// is clipped first, outside everything; then the pivot moves to `(x, y)`; then the
    /// rotation and the scale, both about that pivot; and only then is the box moved back
    /// by the origin's fraction of itself. Inside `draw`, the element's box is
    /// `(0, 0, width, height)` in unscaled units — which is why a `stroke_width` written
    /// in the document scales with `scale` without anything arranging it.
    fn in_element_space(
        &mut self,
        extent: Extent,
        transform: &Transform,
        clip: Option<Region>,
        draw: impl FnOnce(&skia_safe::Canvas),
    ) {
        if extent.width <= 0.0 || extent.height <= 0.0 || transform.opacity <= 0.0 {
            return;
        }
        let canvas = self.surface.canvas();
        canvas.save();
        if let Some(clip) = clip {
            canvas.clip_rect(clip.rect(), None, Some(true));
        }
        // One layer for the whole element rather than alpha on each paint: a fill and an
        // inside stroke overlap, and two alphas would blend the overlap twice.
        let layered = transform.opacity < 1.0;
        if layered {
            canvas.save_layer_alpha_f(None, transform.opacity as f32);
        }
        canvas.translate((transform.x as f32, transform.y as f32));
        if transform.rotation != 0.0 {
            canvas.rotate(transform.rotation as f32, None);
        }
        canvas.scale((transform.scale.0 as f32, transform.scale.1 as f32));
        canvas.translate((
            (-transform.origin.0 * extent.width) as f32,
            (-transform.origin.1 * extent.height) as f32,
        ));
        draw(canvas);
        if layered {
            canvas.restore();
        }
        canvas.restore();
    }

    /// Crop, scale and encode — the three things that happen to the finished frame, in the
    /// one place they happen, so a `--crop` and a `--full` cannot disagree about order.
    ///
    /// `crop` is in **frame space**, at true pixels, and is intersected with the frame: a
    /// region that reaches outside the canvas comes back as the part that is inside it,
    /// rather than as a picture with an invented border. `None` where the crop misses the
    /// frame entirely, or where the encoder refused.
    pub fn encode(
        &mut self,
        crop: Option<Region>,
        scale: Scale,
        encoding: Encoding,
    ) -> Option<Encoded> {
        let whole = Region {
            x: 0,
            y: 0,
            width: self.width as i64,
            height: self.height as i64,
        };
        let region = match crop {
            Some(crop) => intersect(crop, whole)?,
            None => whole,
        };

        let out_width = scale.apply(region.width);
        let out_height = scale.apply(region.height);
        let info = ImageInfo::new(
            ISize::new(out_width as i32, out_height as i32),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let mut out = surfaces::raster(&info, None, None)?;
        // Cleared opaque black before the frame is drawn over it, so a translucent frame
        // encodes to the same picture in both formats: JPEG carries no alpha and would
        // otherwise flatten against whatever the encoder chose.
        out.canvas().clear(Rgba::BLACK.colour());
        let snapshot = self.surface.image_snapshot();
        let paint = SkPaint::default();
        out.canvas().draw_image_rect_with_sampling_options(
            &snapshot,
            Some((&region.rect(), skia_safe::canvas::SrcRectConstraint::Strict)),
            Rect::from_xywh(0.0, 0.0, out_width as f32, out_height as f32),
            sampling(),
            &paint,
        );

        let quality = match encoding {
            Encoding::Jpeg => Some(JPEG_QUALITY),
            Encoding::Png => None,
        };
        let bytes = out
            .image_snapshot()
            .encode(None, encoding.format(), quality)?
            .as_bytes()
            .to_vec();
        Some(Encoded {
            bytes,
            width: out_width,
            height: out_height,
            encoding,
            scale,
        })
    }
}

/// Bilinear, no mipmaps — the prototype's sampling, kept because the golden frames the
/// oracle guards were measured with it (ADR-0010).
fn sampling() -> SamplingOptions {
    SamplingOptions::new(skia_safe::FilterMode::Linear, skia_safe::MipmapMode::None)
}

/// The overlap of two regions, or `None` where they do not overlap at all.
///
/// Saturating, like every other edge in this crate that a caller's arithmetic can reach: an
/// edge past `i64::MAX` means past everything, which is what the caller wrote, rather than
/// a panic in one build and a wrapped negative edge in the other.
fn intersect(a: Region, b: Region) -> Option<Region> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = a.x.saturating_add(a.width).min(b.x.saturating_add(b.width));
    let bottom =
        a.y.saturating_add(a.height)
            .min(b.y.saturating_add(b.height));
    (right > x && bottom > y).then_some(Region {
        x,
        y,
        width: right - x,
        height: bottom - y,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_scale_rounds_up_and_never_reaches_zero() {
        assert_eq!(Scale::Half.apply(1080), 540);
        assert_eq!(Scale::Half.apply(1920), 960);
        // An odd extent must not come back a pixel short of half.
        assert_eq!(Scale::Half.apply(101), 51);
        assert_eq!(Scale::Half.apply(1), 1);
        assert_eq!(Scale::Full.apply(1), 1);
    }

    #[test]
    fn a_crop_that_reaches_outside_the_frame_comes_back_as_the_part_inside_it() {
        let whole = Region {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
        let asked = Region {
            x: 80,
            y: 80,
            width: 40,
            height: 40,
        };
        assert_eq!(
            intersect(asked, whole),
            Some(Region {
                x: 80,
                y: 80,
                width: 20,
                height: 20
            })
        );
        assert_eq!(
            intersect(
                Region {
                    x: 200,
                    y: 0,
                    width: 10,
                    height: 10
                },
                whole
            ),
            None
        );
    }
}
