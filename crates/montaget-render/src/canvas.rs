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
//! Glyph painting is [`Canvas::text`] and arrived with
//! [#213](https://github.com/MBehtemam/Montaget/issues/213). It takes outlines rather than
//! text, which is ADR-0010's split made structural: *"the text stack stands beside the
//! rasterizer"*, so this crate does not depend on `montaget-text` and there is no string,
//! no font and no font file anywhere in it.
//!
//! The ordered [`Effect`] list — blur, shadow, mask and the four colour scalars — arrived
//! with [#214](https://github.com/MBehtemam/Montaget/issues/214) and is applied by
//! [`Canvas::through`], in element space, in the order the list is written.
//!
//! What is **not** here, and is the core's business rather than an omission: `crossfade`
//! and a run's `highlight` window. Both are *resolutions*, not paint rules — a crossfade is an
//! opacity the two bridged elements already carry and a highlight is which of a run's two
//! declared styles applies at this instant — so both are settled in `montaget-core` and
//! reach this crate as the numbers every other element's do.

use skia_safe::{
    AlphaType, BlendMode, Color, Color4f, ColorType, Data, EncodedImageFormat, ISize, Image,
    ImageFilter, ImageInfo, Paint as SkPaint, PaintStyle, Path, PathBuilder, PathFillType, Rect,
    SamplingOptions, Surface, canvas::SaveLayerRec, color_filters, image_filters, images, surfaces,
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

/// One thing's paint — a shape's, or one glyph's.
///
/// The same three fields for both, because ADR-0014 makes them one vocabulary: `stroke`
/// is *"a second paint on the same outline, run-addressable, that never enlarges the
/// declared rect"*. What differs between a shape and a glyph is **which side of the
/// outline the stroke falls on**, and that is a rule about painting rather than about the
/// paint — [`Canvas::shape`] insets, [`Canvas::text`] does not.
///
/// `fill` may be absent when `stroke` is present, giving an outlined shape or an outlined
/// letter; a shape with neither is a schema error the core reports, and paints nothing
/// here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    pub fill: Option<Rgba>,
    pub stroke: Option<Rgba>,
    pub stroke_width: f64,
}

/// One segment of a glyph's outline, at the glyph's own origin, y-down.
///
/// **The rasterizer's own spelling of the same shape `montaget-text` hands back**, and the
/// duplication is the seam rather than an oversight — the pair `Rect`/[`Region`] already
/// carries the same one. ADR-0010 puts the text stack *beside* this crate, so nothing here
/// may name a `montaget-text` type; the conversion happens in `montaget-core`, which is
/// the crate that depends on both.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathEl {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

/// One glyph to paint: where it goes, which outline it is, and the paint it wears.
///
/// The paint is per glyph rather than per call because ADR-0014 makes `stroke` — and
/// ADR-0007 makes `color` and `size` — **run-addressable**: *"outline one word"* is the
/// case the ADR names, and a paint argument covering the whole element could not express
/// it. It is [`Fill`] rather than three fields of its own, so a glyph and a shape carry
/// one paint type between them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    /// The glyph's origin, in the text block's own unscaled coordinates.
    pub x: f64,
    pub y: f64,
    /// Which of the `outlines` slice to draw.
    pub outline: usize,
    pub paint: Fill,
}

/// One member of the closed `effects` vocabulary, already parsed.
///
/// The rasterizer's own spelling of what `montaget-core`'s model reads, for the same
/// reason [`PathEl`] and [`Region`] are: this crate names no type of the crate that reads
/// the document. What reaches here is numbers and colours.
///
/// **Two effects of the same name are ordinary** (ADR-0040) — a `&[Effect]` rather than a
/// set, precisely so a second shadow has somewhere to go — and **the order is semantically
/// real**: `[blur, shadow]` casts a shadow from an already-blurred silhouette, `[shadow,
/// blur]` blurs a picture that already has a hard-edged shadow in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    /// Gaussian blur, one parameter (ADR-0040).
    Blur { radius: f64 },
    /// Drop shadow: the element, with a blurred copy of its own silhouette behind it.
    Shadow {
        dx: f64,
        dy: f64,
        radius: f64,
        colour: Rgba,
        opacity: f64,
    },
    /// Keep only what falls inside a shape, param-less (ADR-0068).
    Mask { shape: MaskShape },
    /// Push pixel colour toward `colour` by `amount` (ADR-0049).
    Tint { colour: Rgba, amount: f64 },
    /// `0` is grayscale, `1` unchanged, `>1` oversaturated (ADR-0049).
    Saturation { amount: f64 },
    /// Signed offset from unchanged at `0` (ADR-0049).
    Brightness { amount: f64 },
    /// Signed offset from unchanged at `0` (ADR-0049).
    Contrast { amount: f64 },
}

/// The param-less mask shapes, each derived from the element's own rect (ADR-0068).
///
/// Distinct from [`Shape`], which is a *drawn* element with a paint and a `radius`. A mask
/// has neither: ADR-0068 fixed the param-less form as the only spelling there is until the
/// explicit geometry vocabulary lands, so there is nothing here to carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskShape {
    /// The largest circle inscribed in the element's rect — diameter `min(width, height)`,
    /// centred on it.
    Circle,
    /// The element's rect itself.
    Rect,
    /// The ellipse inscribed in the element's rect.
    Ellipse,
}

impl MaskShape {
    /// Everything an element box of `extent` holds **except** the shape — an
    /// inverse-filled path, at `(0, 0)` in element space.
    ///
    /// Inverse rather than the shape itself, because a mask has to erase what it does not
    /// select and a draw call only ever reaches the pixels its own geometry covers.
    /// Painting the shape in `DstIn` looks like the right gesture and does nothing at all:
    /// inside the shape it multiplies by an alpha of 1, and outside it there is no draw.
    /// The complement, painted in `Clear`, is the operation — and antialiased, so the
    /// mask's edge is a coverage ramp rather than a staircase.
    fn outside(self, extent: Extent) -> Path {
        let (width, height) = (extent.width as f32, extent.height as f32);
        let mut path = PathBuilder::new();
        match self {
            MaskShape::Circle => {
                let diameter = width.min(height);
                path.add_oval(
                    Rect::from_xywh(
                        (width - diameter) / 2.0,
                        (height - diameter) / 2.0,
                        diameter,
                        diameter,
                    ),
                    None,
                    None,
                );
            }
            MaskShape::Rect => {
                path.add_rect(Rect::from_xywh(0.0, 0.0, width, height), None, None);
            }
            MaskShape::Ellipse => {
                path.add_oval(Rect::from_xywh(0.0, 0.0, width, height), None, None);
            }
        }
        let mut path = path.detach();
        path.set_fill_type(PathFillType::InverseWinding);
        path
    }
}

/// A blur or shadow `radius` as the Gaussian sigma Skia takes.
///
/// **`sigma = radius / 2`, a recorded reading rather than an ADR's.** ADR-0040 names
/// `blur`'s one parameter `radius` and `shadow`'s blur `radius`, and no accepted document
/// says what a radius *is* in a Gaussian. Two conventions exist and they differ by 2×:
/// CSS's `filter: blur(r)` and `box-shadow`'s blur radius are both `2σ`, while Skia's own
/// API takes σ directly. CSS is the one an agent has seen before and the one the reference
/// class's own numbers are quoted in, so it is the one written here — and it is a function
/// rather than a literal so the day an ADR states otherwise there is one line to change.
/// Raised at [#280](https://github.com/MBehtemam/Montaget/issues/280).
fn sigma(radius: f64) -> f32 {
    (radius.max(0.0) / 2.0) as f32
}

/// Skia's luma weights, which are the ones its own `SkColorMatrix::setSaturation` uses.
/// Stated here rather than borrowed from a header so that the matrix below can be read
/// without one.
const LUMA: [f32; 3] = [0.213, 0.715, 0.072];

impl Effect {
    /// This effect as an image filter over whatever was painted before it, or `None` for
    /// [`Effect::Mask`] — which is a geometric restriction rather than a filter, and is
    /// applied by [`Canvas::in_element_space`] as a `DstIn` draw over its own layer.
    fn filter(self) -> Option<ImageFilter> {
        match self {
            Effect::Mask { .. } => None,
            Effect::Blur { radius } => {
                image_filters::blur((sigma(radius), sigma(radius)), None, None, None)
            }
            Effect::Shadow {
                dx,
                dy,
                radius,
                colour,
                opacity,
            } => {
                // `opacity` multiplies the shadow colour's own alpha rather than
                // replacing it: ADR-0040 gives `shadow` both a `color` and an `opacity`
                // and does not say what the pair means, and multiplying is both what
                // every editor offering the two controls does and what keeps
                // `opacity: 1` the identity. Raised at
                // [#280](https://github.com/MBehtemam/Montaget/issues/280).
                let [r, g, b, a] = colour.0;
                let alpha = f32::from(a) / 255.0 * opacity.clamp(0.0, 1.0) as f32;
                let colour = Color4f::new(
                    f32::from(r) / 255.0,
                    f32::from(g) / 255.0,
                    f32::from(b) / 255.0,
                    alpha,
                );
                image_filters::drop_shadow(
                    (dx as f32, dy as f32),
                    (sigma(radius), sigma(radius)),
                    colour,
                    None,
                    None,
                    None,
                )
            }
            // The four colour scalars, named rather than caught by a wildcard: it is what
            // makes `matrix`'s own exhaustive match a compiler-checked claim about which
            // members reach it, instead of a comment asserting it.
            Effect::Tint { .. }
            | Effect::Saturation { .. }
            | Effect::Brightness { .. }
            | Effect::Contrast { .. } => image_filters::color_filter(
                color_filters::matrix_row_major(&self.matrix(), None),
                None,
                None,
            ),
        }
    }

    /// The four colour scalars as one row-major colour matrix (ADR-0049).
    ///
    /// Skia applies the matrix to **unpremultiplied** colour and premultiplies afterwards,
    /// so the alpha row is identity in every one of them and a transparent pixel stays
    /// transparent however hard it is tinted. The translation column is in unit rather
    /// than byte range, which is why `brightness` writes `amount` and not `amount × 255`.
    fn matrix(self) -> [f32; 20] {
        match self {
            Effect::Saturation { amount } => {
                // The identity is `amount: 1`, so `0` is the grayscale case ADR-0049
                // folded the `grayscale` member into and `>1` oversaturates.
                let a = amount.max(0.0) as f32;
                let [lr, lg, lb] = LUMA;
                let (d, o) = (|w: f32| w + a * (1.0 - w), |w: f32| w * (1.0 - a));
                [
                    d(lr),
                    o(lg),
                    o(lb),
                    0.0,
                    0.0, //
                    o(lr),
                    d(lg),
                    o(lb),
                    0.0,
                    0.0, //
                    o(lr),
                    o(lg),
                    d(lb),
                    0.0,
                    0.0, //
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    0.0,
                ]
            }
            Effect::Brightness { amount } => {
                // A signed offset, unchanged at 0 (ADR-0049) — added to each channel
                // rather than multiplied into it, which is what makes `0` the identity
                // and `-1` black rather than a scale that can never reach either end.
                let b = amount as f32;
                [
                    1.0, 0.0, 0.0, 0.0, b, //
                    0.0, 1.0, 0.0, 0.0, b, //
                    0.0, 0.0, 1.0, 0.0, b, //
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ]
            }
            Effect::Contrast { amount } => {
                // Pivoted on mid-grey, so `0` is the identity and the two directions are
                // symmetric: `c' = (c - 0.5) × (1 + amount) + 0.5`.
                let s = (1.0 + amount as f32).max(0.0);
                let t = 0.5 - 0.5 * s;
                [
                    s, 0.0, 0.0, 0.0, t, //
                    0.0, s, 0.0, 0.0, t, //
                    0.0, 0.0, s, 0.0, t, //
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ]
            }
            Effect::Tint { colour, amount } => {
                // A straight lerp toward the tint: `c' = c × (1 - a) + tint × a`, which is
                // ADR-0049's "pushes pixel colour toward `color` by `amount`" and nothing
                // more. `amount: 0` is the documented identity that makes the non-scalar
                // `color` admissible at all.
                let a = (amount.clamp(0.0, 1.0)) as f32;
                let [r, g, b, _] = colour.0;
                let (tr, tg, tb) = (
                    f32::from(r) / 255.0 * a,
                    f32::from(g) / 255.0 * a,
                    f32::from(b) / 255.0 * a,
                );
                let k = 1.0 - a;
                [
                    k, 0.0, 0.0, 0.0, tr, //
                    0.0, k, 0.0, 0.0, tg, //
                    0.0, 0.0, k, 0.0, tb, //
                    0.0, 0.0, 0.0, 1.0, 0.0,
                ]
            }
            // Never reached: [`Effect::filter`] names the four colour members explicitly
            // and sends nothing else here. The identity is what a new member would get if
            // that ever stopped being true, and adding one to the enum breaks *this* match
            // first, which is the point of spelling the three out rather than `_`.
            Effect::Blur { .. } | Effect::Shadow { .. } | Effect::Mask { .. } => [
                1.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, 0.0,
            ],
        }
    }
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
        effects: &[Effect],
    ) {
        if paint.fill.is_none() && paint.stroke.is_none() {
            return;
        }
        self.in_element_space(extent, transform, clip, effects, |canvas| {
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
        effects: &[Effect],
    ) {
        self.in_element_space(extent, transform, clip, effects, |canvas| {
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

    /// Paint one text element's glyphs (ADR-0010: the rasterizer fills the paths).
    ///
    /// `extent` is the **typographic block** — the widest line's advance by the sum of the
    /// slots — and every glyph's `x`/`y` is relative to that block's top-left, so text
    /// goes through exactly the same transform, pivot and `clip` as every other element
    /// type ([`Canvas::in_element_space`]). Nothing about text is placed here.
    ///
    /// **Two passes, strokes first.** ADR-0014 puts a text stroke *outside* the glyph
    /// contour, which means a wide one reaches over its neighbours — the ASS `\bord`
    /// model, where the whole border layer is drawn and the letterforms are laid on top.
    /// One pass per glyph would let each glyph's border cut into the letter before it, so
    /// the border of `ll` would show a seam that the same text in one paint does not.
    ///
    /// The stroke is drawn at **twice** the declared width, because Skia centres a stroke
    /// on the path: half of `2 × stroke_width` outside the contour is the
    /// `stroke_width` ADR-0014 asks for, and the inner half is covered by the fill that
    /// follows. That is also what makes the ADR's other clause fall out rather than be
    /// arranged — the stroke is in element space, so it scales with `scale`.
    ///
    /// A glyph with a stroke and **no** fill has nothing to cover that inner half, so
    /// there the stroke is clipped to the outside of its own contour and the letterform
    /// stays open. No document reaches that today — the core gives every run a colour,
    /// defaulting to black — but this is a public surface and an outlined-and-hollow
    /// letter is what `fill: None` reads as.
    pub fn text(
        &mut self,
        glyphs: &[Glyph],
        outlines: &[Vec<PathEl>],
        extent: Extent,
        transform: &Transform,
        clip: Option<Region>,
        effects: &[Effect],
    ) {
        if glyphs.is_empty() {
            return;
        }
        // Built once per element and keyed by the same index `montaget-text` deduplicated
        // on, so a repeated letter is one path however many times it appears.
        let paths: Vec<Path> = outlines.iter().map(|outline| path_of(outline)).collect();

        self.in_element_space(extent, transform, clip, effects, |canvas| {
            for pass in [Pass::Stroke, Pass::Fill] {
                for glyph in glyphs {
                    let Some(path) = paths.get(glyph.outline) else {
                        continue;
                    };
                    let paint = match pass {
                        Pass::Stroke => {
                            let (Some(colour), true) =
                                (glyph.paint.stroke, glyph.paint.stroke_width > 0.0)
                            else {
                                continue;
                            };
                            let mut paint = SkPaint::new(Color4f::from(colour.colour()), None);
                            paint.set_anti_alias(true);
                            paint.set_style(PaintStyle::Stroke);
                            paint.set_stroke_width(glyph.paint.stroke_width as f32 * 2.0);
                            // Round joins rather than mitres: a mitre on a sharp interior
                            // angle spikes out to an arbitrary length, which on a serif or
                            // a comma is a visible whisker rather than a border.
                            paint.set_stroke_join(skia_safe::PaintJoin::Round);
                            paint
                        }
                        Pass::Fill => {
                            let Some(colour) = glyph.paint.fill else {
                                continue;
                            };
                            let mut paint = SkPaint::new(Color4f::from(colour.colour()), None);
                            paint.set_anti_alias(true);
                            paint.set_style(PaintStyle::Fill);
                            paint
                        }
                    };
                    canvas.save();
                    canvas.translate((glyph.x as f32, glyph.y as f32));
                    // A stroke with no fill behind it has to be clipped to the outside of
                    // the contour itself, or the inner half of the doubled width — the
                    // half a fill would normally cover — paints over the letterform and
                    // an outlined word comes out solid. Only in that case: with a fill
                    // present the clip would put an antialiased seam along every contour,
                    // and the fill already does the job.
                    if matches!(pass, Pass::Stroke) && glyph.paint.fill.is_none() {
                        canvas.clip_path(path, skia_safe::ClipOp::Difference, true);
                    }
                    canvas.draw_path(path, &paint);
                    canvas.restore();
                }
            }
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
        effects: &[Effect],
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
        //
        // **Outside the effects**, so `opacity` fades the finished element — its shadow
        // included. Inside them, a half-transparent element would cast a full-strength
        // shadow, which is the one thing every editor in the reference class agrees it
        // does not do. Neither ADR-0012 nor ADR-0040 says which wraps which; raised at
        // [#280](https://github.com/MBehtemam/Montaget/issues/280).
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
        Canvas::through(canvas, extent, effects, draw);
        if layered {
            canvas.restore();
        }
        canvas.restore();
    }

    /// Run `draw` inside the ordered `effects` list, in element space.
    ///
    /// **One layer per effect, the last one outermost.** A layer's paint filters its
    /// contents at the moment it is composited into its parent, so nesting them in reverse
    /// is what makes `effects[0]` see the bare element and `effects[n-1]` see everything
    /// before it — ADR-0040's *"order is semantically real"*, implemented as the order the
    /// layers unwind in rather than arranged by a second sort.
    ///
    /// **The filters run in element space**, because that is the coordinate space the
    /// canvas is already in when the layers are opened and Skia maps an image filter
    /// through the current matrix. So a `blur` of radius 8 on an element at `scale: 2` is
    /// 16 frame pixels wide, exactly as ADR-0014's `stroke_width` is — the same rule, for
    /// the same reason, and not a second one written down here.
    ///
    /// **A `mask` is not a filter and is not a clip.** It is [`MaskShape::outside`] —
    /// everything the shape does not cover — painted over its own layer in `Clear`, which
    /// erases the layer everywhere the mask does not select. A clip would have done the
    /// same thing more cheaply and been wrong for a specific reason: a clip established
    /// *before* an enclosing layer restricts that layer's bounds, so `[mask, blur]` would
    /// hand the blur an input already cut to the mask and the blur would lose every
    /// contribution from just outside it. Painting the complement keeps each effect's
    /// input the full element.
    fn through(
        canvas: &skia_safe::Canvas,
        extent: Extent,
        effects: &[Effect],
        draw: impl FnOnce(&skia_safe::Canvas),
    ) {
        for effect in effects.iter().rev() {
            match effect.filter() {
                Some(filter) => {
                    let mut paint = SkPaint::default();
                    paint.set_image_filter(filter);
                    canvas.save_layer(&SaveLayerRec::default().paint(&paint));
                }
                // A `mask`, and also any filter Skia declined to build — an effect that
                // cannot be made is a layer that changes nothing rather than a missing
                // layer, so the unwinding below stays in step with the list whatever
                // happens here.
                None => {
                    canvas.save_layer(&SaveLayerRec::default());
                }
            };
        }
        draw(canvas);
        for effect in effects {
            if let Effect::Mask { shape } = effect {
                let mut paint = SkPaint::default();
                paint.set_anti_alias(true);
                paint.set_blend_mode(BlendMode::Clear);
                canvas.draw_path(&shape.outside(extent), &paint);
            }
            canvas.restore();
        }
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

/// Which of [`Canvas::text`]'s two passes is being painted.
enum Pass {
    Stroke,
    Fill,
}

/// One outline as a Skia path, at the glyph's own origin.
fn path_of(outline: &[PathEl]) -> Path {
    let mut path = PathBuilder::new();
    for element in outline {
        match *element {
            PathEl::Move(x, y) => {
                path.move_to((x, y));
            }
            PathEl::Line(x, y) => {
                path.line_to((x, y));
            }
            PathEl::Quad(cx, cy, x, y) => {
                path.quad_to((cx, cy), (x, y));
            }
            PathEl::Cubic(ax, ay, bx, by, x, y) => {
                path.cubic_to((ax, ay), (bx, by), (x, y));
            }
            PathEl::Close => {
                path.close();
            }
        }
    }
    path.detach()
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

    /// A 40x40 square at the glyph origin, as an outline — the simplest shape whose
    /// interior and whose border are both easy to point at.
    fn square() -> Vec<PathEl> {
        vec![
            PathEl::Move(0.0, 0.0),
            PathEl::Line(40.0, 0.0),
            PathEl::Line(40.0, 40.0),
            PathEl::Line(0.0, 40.0),
            PathEl::Close,
        ]
    }

    fn one_glyph(paint: Fill) -> Vec<u8> {
        let mut canvas = Canvas::new(100, 100).expect("a surface");
        canvas.background(Rgba([0xFF, 0xFF, 0xFF, 0xFF]));
        canvas.text(
            &[Glyph {
                x: 0.0,
                y: 0.0,
                outline: 0,
                paint,
            }],
            &[square()],
            Extent {
                width: 40.0,
                height: 40.0,
            },
            &Transform {
                x: 30.0,
                y: 30.0,
                origin: (0.0, 0.0),
                scale: (1.0, 1.0),
                rotation: 0.0,
                opacity: 1.0,
            },
            None,
            &[],
        );
        canvas
            .encode(None, Scale::Full, Encoding::Png)
            .expect("an encoded frame")
            .bytes
    }

    /// One pixel of a PNG this module just wrote, read back through Skia's own decoder.
    ///
    /// The independent-decoder rule `tests/frame.rs` follows does not apply here: this is
    /// a unit test of one paint rule, not a claim that the bytes are a picture, and the
    /// crate links no second codec.
    fn pixel_at(png: &[u8], x: i32, y: i32) -> [u8; 4] {
        let image = Image::from_encoded(Data::new_copy(png)).expect("it decodes");
        let info = ImageInfo::new(
            ISize::new(1, 1),
            ColorType::RGBA8888,
            AlphaType::Unpremul,
            None,
        );
        let mut out = [0u8; 4];
        assert!(
            image.read_pixels(
                &info,
                &mut out,
                4,
                (x, y),
                skia_safe::image::CachingHint::Allow
            ),
            "reading ({x}, {y})"
        );
        out
    }

    #[test]
    fn a_glyph_stroke_with_no_fill_behind_it_leaves_the_letterform_open() {
        // ADR-0014 puts a text stroke *outside* the contour. The stroke is painted at
        // twice the declared width and normally relies on the fill to cover the inner
        // half — so with no fill, an unclipped stroke would flood the interior and an
        // outlined word would come out solid. The interior must stay the background.
        let hollow = one_glyph(Fill {
            fill: None,
            stroke: Some(Rgba([0x00, 0x00, 0xFF, 0xFF])),
            stroke_width: 6.0,
        });
        // The square spans (30, 30)..(70, 70); its middle is nowhere near either edge.
        assert_eq!(
            pixel_at(&hollow, 50, 50),
            [0xFF, 0xFF, 0xFF, 0xFF],
            "the interior was painted over"
        );
        // And the border is there, six pixels outside the contour and not inside it.
        assert_eq!(pixel_at(&hollow, 27, 50), [0x00, 0x00, 0xFF, 0xFF]);
        assert_eq!(
            pixel_at(&hollow, 33, 50),
            [0xFF, 0xFF, 0xFF, 0xFF],
            "the stroke reached inside the contour"
        );

        // With a fill, the same call paints a solid square — the clip is not applied, so
        // no antialiased seam is introduced along the contour of ordinary text.
        let solid = one_glyph(Fill {
            fill: Some(Rgba([0xFF, 0x00, 0x00, 0xFF])),
            stroke: Some(Rgba([0x00, 0x00, 0xFF, 0xFF])),
            stroke_width: 6.0,
        });
        assert_eq!(pixel_at(&solid, 50, 50), [0xFF, 0x00, 0x00, 0xFF]);
        assert_eq!(pixel_at(&solid, 33, 50), [0xFF, 0x00, 0x00, 0xFF]);
        assert_eq!(pixel_at(&solid, 27, 50), [0x00, 0x00, 0xFF, 0xFF]);
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
