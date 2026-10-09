//! The rasterizer: a Skia surface, the flat transform ADR-0012 declares, and the two
//! primitives ADR-0014 names.
//!
//! **This module knows nothing about the document.** It takes numbers that are already
//! resolved — an instant's `x`, `y`, `scale`, `rotation` and `opacity`, a declared box, a
//! colour — and paints them. Every rule about *where those numbers come from* (which
//! keyframe record, which default, which element is present) lives in `montagent-core`,
//! which is the crate that reads the format. The split is the one that keeps
//! `montagent-core` able to depend on this crate at all: a rasterizer that read the
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
//!   [#274](https://github.com/MBehtemam/Montagent/issues/274) rather than left to be
//!   discovered from a stretched, tilted card.
//! - **`opacity` is one layer, not a per-paint alpha.** An element with a fill *and* an
//!   inside stroke overlaps itself, and multiplying alpha into both paints would blend the
//!   overlap twice and darken the stroke's inner edge. A `save_layer_alpha` composites the
//!   whole element once.
//!
//! Glyph painting is [`Canvas::text`] and arrived with
//! [#213](https://github.com/MBehtemam/Montagent/issues/213). It takes outlines rather than
//! text, which is ADR-0010's split made structural: *"the text stack stands beside the
//! rasterizer"*, so this crate does not depend on `montagent-text` and there is no string,
//! no font and no font file anywhere in it.
//!
//! The ordered [`Effect`] list — blur, shadow, mask and the four colour scalars — arrived
//! with [#214](https://github.com/MBehtemam/Montagent/issues/214) and is applied by
//! [`Canvas::through`], in element space, in the order the list is written. ADR-0088's
//! `chroma` joined it with [#342](https://github.com/MBehtemam/Montagent/issues/342) as an
//! eighth member and a fifth paint rule:
//!
//! - **A key is computed in the chroma plane, not in RGB.** [`Effect::Chroma`] converts
//!   both the pixel and the declared screen colour to BT.601 `(U, V)` and measures the
//!   distance there, with the luma axis dropped rather than weighted. That is `ffmpeg`'s
//!   `chromakey`, which is the filter every number in ADR-0088's evidence table was
//!   measured through — so the fixture's plateau is a reading of this code's own tolerance
//!   axis and not of a neighbouring one.
//!
//! ADR-0156's named effects `posterize`, `glow` and `directional_blur` joined it with
//! [#724](https://github.com/MBehtemam/Montagent/issues/724); their formulas are in `named`.
//!
//! What is **not** here, and is the core's business rather than an omission: `crossfade`
//! and a run's `highlight` window. Both are *resolutions*, not paint rules — a crossfade is an
//! opacity the two bridged elements already carry and a highlight is which of a run's two
//! declared styles applies at this instant — so both are settled in `montagent-core` and
//! reach this crate as the numbers every other element's do.

use skia_safe::{
    AlphaType, BlendMode, Color, Color4f, ColorFilter, ColorType, CubicResampler, Data,
    EncodedImageFormat, ISize, Image, ImageFilter, ImageInfo, Matrix, Paint as SkPaint, PaintStyle,
    Path, PathBuilder, PathFillType, RRect, Rect, RuntimeEffect, SamplingOptions, Surface,
    canvas::SaveLayerRec, color_filters, image_filters, images, surfaces,
};

mod gradient;
mod grain;
mod layer_bound;
mod named;
mod projection;

pub use gradient::{Gradient, GradientKind, Ink};
pub use grain::grain_draw;
#[doc(hidden)]
pub use layer_bound::enabled as filter_layers_bounded;
#[doc(hidden)]
pub use layer_bound::set_enabled as bound_filter_layers;
pub use projection::{Facing, Projection, Quad, eye_bound, quad, reach_box};

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
    /// [#274](https://github.com/MBehtemam/Montagent/issues/274) rather than left to be
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
    /// How the finished element composites into what is below it (ADR-0147). Not a
    /// transform property, and static, but carried here beside `opacity` because the two
    /// are applied together by the one layer the element is composited through.
    pub blend: Blend,
    /// The projection (ADR-0167), on an element that writes `swivel` or `tilt`: it is drawn
    /// flat into a layer with its effects and mask, and the layer is drawn through the
    /// projection about `origin`, innermost, before `scale`, `rotation` and `x`/`y`. `None`
    /// on an element with neither field, which paints exactly as before (ADR-0168 §1).
    pub projection: Option<Projection>,
}

/// ADR-0147's five modes, each one Skia mode. The arithmetic runs on the stored sRGB
/// values: the surface carries no colour space, so Skia blends the bytes as they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Blend {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
}

impl Blend {
    fn mode(self) -> BlendMode {
        match self {
            Blend::Normal => BlendMode::SrcOver,
            Blend::Multiply => BlendMode::Multiply,
            Blend::Screen => BlendMode::Screen,
            Blend::Overlay => BlendMode::Overlay,
            Blend::Add => BlendMode::Plus,
        }
    }
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
///
/// Either side may be a gradient ([`Ink`], ADR-0149), measured against the declared box the
/// core places in the drawing's own coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub fill: Option<Ink>,
    pub stroke: Option<Ink>,
    pub stroke_width: f64,
}

/// How a `path`'s stroke turns its corners and ends its open ends (ADR-0158 §2–§3).
///
/// The join and cap are a path's alone: a `rect` or `ellipse` stroke keeps the join the
/// painter has always drawn it with, and a glyph's its round join. The default is
/// ADR-0154's pin, a round join and a butt cap, and no dash, so a path that writes none of
/// them paints the bytes it always has.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StrokeStyle {
    pub join: Join,
    pub cap: Cap,
    /// The dash pattern (ADR-0158 §5), run from `points[0]` in points order.
    pub dash: Option<Dash>,
    /// The part of the outline the stroke draws (ADR-0160), measured from `points[0]` in
    /// points order. `None` draws it whole, as a full window does.
    pub trim: Option<Trim>,
}

/// A trimmed stroke's window (ADR-0160 §4–§6), resolved by the core: what is left once a
/// full window has been told apart, which draws exactly as no trim and so is `None` where a
/// trim is asked for.
///
/// - **Empty** draws no stroke at all, under any cap: Skia would draw a dot for a
///   zero-length segment under a round cap, and a draw-on must not pop one on its first
///   frame.
/// - **A part** runs from `from` to `to`, fractions of the outline's length from its start
///   point in its direction, measured by the same path measure the dashes are laid along.
///   `from > to` crosses the start point, and is drawn as **one contour** through it, so the
///   outline's own join turns there and no cap is drawn.
///
/// **Dashes stay put.** The pattern is laid along the whole outline first, exactly as an
/// untrimmed stroke lays it — the dash that is "on" across a closed outline's start point
/// stays one dash through it — and the window then keeps the part of each dash it covers.
/// A dash's phase never depends on the trim, and a trim end inside a dash takes the cap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trim {
    Empty,
    Part { from: f64, to: f64 },
}

/// A dash pattern on a stroke (ADR-0158 §5): lengths alternating dash, gap, dash, gap,
/// starting with a dash, in element pixels along the outline, and the phase — how far into
/// the pattern the outline's start falls.
///
/// The pattern is drawn as written: never doubled, never stretched to fit the outline. The
/// outline it runs along is built with ADR-0158 §5's start point and direction
/// ([`Canvas::shape`]), never with Skia's default start index for a rect or an oval, so a
/// Skia update cannot move where a pattern starts.
///
/// **At the start point** Skia joins the pattern across a closed outline's start when the
/// pattern is "on" there: the dash left over at the end of the outline and the first dash
/// draw as one, turning a corner with the stroke's join. When the pattern is "off" there, the
/// leftover is a short gap. Where a dash boundary lands exactly on a vertex, the two ends are
/// drawn with caps and no join between them, as in SVG.
#[derive(Debug, Clone, PartialEq)]
pub struct Dash {
    intervals: Vec<f32>,
    phase: f32,
}

impl Dash {
    /// The pattern `lengths` with `offset` wrapped into `[0, total)`, as SVG and Skia read a
    /// dash phase: a larger offset moves the dashes back, toward the outline's start. `None`
    /// for a pattern Skia cannot dash — an odd count or a zero total, which `validate`
    /// refuses.
    pub fn new(lengths: &[i64], offset: f64) -> Option<Dash> {
        let total: i64 = lengths.iter().sum();
        if lengths.len() % 2 == 1 || lengths.is_empty() || total <= 0 {
            return None;
        }
        Some(Dash {
            intervals: lengths.iter().map(|length| *length as f32).collect(),
            phase: offset.rem_euclid(total as f64) as f32,
        })
    }

    fn effect(&self) -> Option<skia_safe::PathEffect> {
        skia_safe::PathEffect::dash(&self.intervals, self.phase)
    }

    /// Every dash along one contour of `length`, as `(start, end)` distances, exactly where
    /// Skia's dash path effect lays them: its phase arithmetic in `f32` and its walk in
    /// `f64` (`SkDashPath::CalcDashParameters` and `InternalFilter`). On a closed contour
    /// that starts inside a dash, the first dash is skipped and laid after the last one; where
    /// the last one runs on to the end of the contour, the two are **one dash** through the
    /// start point, `(start, length + first)`.
    fn along(&self, length: f32, closed: bool) -> Vec<(f64, f64)> {
        let count = self.intervals.len();
        let total: f32 = self.intervals.iter().sum();
        let mut phase = self.phase;
        if phase >= total {
            phase %= total;
        }
        let (mut index, mut first) = (0, self.intervals[0]);
        for (i, &gap) in self.intervals.iter().enumerate() {
            if phase > gap || (phase == gap && gap != 0.0) {
                phase -= gap;
            } else {
                (index, first) = (i, gap - phase);
                break;
            }
        }
        let initial = index;
        let mut dashes = Vec::new();
        let mut skip = closed;
        let mut added = false;
        let mut distance = 0.0_f64;
        let mut dlen = f64::from(first);
        while distance < f64::from(length) {
            added = false;
            if index % 2 == 0 && !skip {
                added = true;
                let end = distance + dlen;
                dashes.push((distance, end.min(f64::from(length))));
            }
            distance += dlen;
            skip = false;
            index = (index + 1) % count;
            dlen = f64::from(self.intervals[index]);
        }
        if closed && initial % 2 == 0 && first >= 0.0 {
            let first = f64::from(first);
            match (added, dashes.last_mut()) {
                (true, Some(last)) => last.1 = f64::from(length) + first,
                _ => dashes.push((0.0, first)),
            }
        }
        dashes
    }
}

/// A stroke's join. A miter corner whose tip would reach past `limit` half-widths from its
/// vertex is drawn beveled — Skia's and SVG's meaning, so under keyed points a corner that
/// sharpens past the limit snaps from a tip to a bevel on one frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Join {
    #[default]
    Round,
    Bevel,
    Miter {
        limit: f32,
    },
}

/// A stroke's cap, drawn at an open path's two ends.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Cap {
    #[default]
    Butt,
    Round,
    Square,
}

impl StrokeStyle {
    /// Set the join, miter limit and cap on a stroke paint. The dash is the caller's: an
    /// untrimmed stroke takes it as Skia's path effect, and a trimmed one is dashed and cut
    /// by [`trimmed`].
    fn apply(&self, paint: &mut SkPaint) {
        match self.join {
            Join::Round => {
                paint.set_stroke_join(skia_safe::PaintJoin::Round);
            }
            Join::Bevel => {
                paint.set_stroke_join(skia_safe::PaintJoin::Bevel);
            }
            Join::Miter { limit } => {
                paint.set_stroke_join(skia_safe::PaintJoin::Miter);
                paint.set_stroke_miter(limit);
            }
        }
        paint.set_stroke_cap(match self.cap {
            Cap::Butt => skia_safe::PaintCap::Butt,
            Cap::Round => skia_safe::PaintCap::Round,
            Cap::Square => skia_safe::PaintCap::Square,
        });
    }
}

/// One segment of a glyph's outline, at the glyph's own origin, y-down.
///
/// **The rasterizer's own spelling of the same shape `montagent-text` hands back**, and the
/// duplication is the seam rather than an oversight — the pair `Rect`/[`Region`] already
/// carries the same one. ADR-0010 puts the text stack *beside* this crate, so nothing here
/// may name a `montagent-text` type; the conversion happens in `montagent-core`, which is
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
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    /// The glyph's origin, in the text block's own unscaled coordinates.
    pub x: f64,
    pub y: f64,
    /// Which of the `outlines` slice to draw.
    pub outline: usize,
    pub paint: Fill,
    /// The pose of the stagger unit this glyph moves with (ADR-0151 §2), where it is not
    /// the rest pose. `None` draws the glyph exactly where it was placed.
    pub unit: Option<UnitDraw>,
}

/// One stagger unit's pose, as the canvas draws it: a body of glyphs that moves, turns,
/// scales and fades as one (ADR-0151 §2, ADR-0153 §2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitDraw {
    /// Which body: the glyphs of one fading body share one layer.
    pub body: usize,
    /// `[sx, kx, tx, ky, sy, ty]`, from the block's own coordinates to the block's own
    /// coordinates.
    pub matrix: [f32; 6],
    /// Below 1, the body's strokes and fills are drawn together into one layer that takes
    /// this opacity.
    pub opacity: f32,
}

/// One member of the closed `effects` vocabulary, already parsed.
///
/// The rasterizer's own spelling of what `montagent-core`'s model reads, for the same
/// reason [`PathEl`] and [`Region`] are: this crate names no type of the crate that reads
/// the document. What reaches here is numbers and colours.
///
/// **Two effects of the same name are ordinary** (ADR-0040) — a `&[Effect]` rather than a
/// set, precisely so a second shadow has somewhere to go — and **the order is semantically
/// real**: `[blur, shadow]` casts a shadow from an already-blurred silhouette, `[shadow,
/// blur]` blurs a picture that already has a hard-edged shadow in it.
#[derive(Debug, Clone, PartialEq)]
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
    /// Keep only what falls inside a shape cut in the mask rect (ADR-0084).
    ///
    /// `rect` is `None` for the identity value — the element's own rect — which is what
    /// ADR-0068's param-less form resolves to rather than a second case beside it.
    /// `radius` rounds the corners of a `MaskShape::Rect` and is `0` elsewhere; the model
    /// refuses it on the other two shapes before it can reach here.
    Mask {
        shape: MaskShape,
        rect: Option<MaskRect>,
        radius: f64,
        /// Keep what is outside the shape instead of what is inside (ADR-0152 §1).
        invert: bool,
        /// The soft edge's width in element units, resolved at the instant and never
        /// rounded: the hard coverage blurred by σ = `feather` / 2 ([`sigma`]), centred on
        /// the edge (ADR-0152 §2). `0` is the hard edge, through the hard path.
        feather: f64,
    },
    /// Push pixel colour toward `colour` by `amount` (ADR-0049).
    Tint { colour: Rgba, amount: f64 },
    /// `0` is grayscale, `1` unchanged, `>1` oversaturated (ADR-0049).
    Saturation { amount: f64 },
    /// Signed offset from unchanged at `0` (ADR-0049).
    Brightness { amount: f64 },
    /// Signed offset from unchanged at `0` (ADR-0049).
    Contrast { amount: f64 },
    /// Key `colour` out of the element's pixels (ADR-0088).
    ///
    /// `tolerance` is the normalised chroma distance within which a pixel is keyed out and
    /// `softness` the width of the partial-alpha band beyond it; both have their identity
    /// at `0`, which is a member that keys nothing and one that keys hard. `spill`
    /// suppresses the screen colour reflected onto what the matte keeps, identity `0`.
    Chroma {
        colour: Rgba,
        tolerance: f64,
        softness: f64,
        spill: f64,
    },
    /// Film grain (ADR-0156 §4): each `size`×`size` cell of element space, anchored at the
    /// box origin, offsets the non-premultiplied colour by a draw in `[−amount, +amount]`.
    /// The draw is [`grain_draw`] of `seed`, the cell and `frame`, the element's local frame;
    /// with `mono` one draw serves R, G and B. Alpha is never changed.
    Grain {
        seed: u32,
        amount: f64,
        size: u32,
        mono: bool,
        frame: i64,
    },
    /// Quantise each colour channel to `levels` steps (ADR-0156 §4). `levels` is already the
    /// whole number from 2 to 256 the core rounded it to.
    Posterize { levels: f64 },
    /// A threshold bloom added over the element (ADR-0156 §4); `radius` reads as `blur`'s.
    Glow {
        threshold: f64,
        radius: f64,
        intensity: f64,
    },
    /// A centred smear along `angle` degrees (clockwise, element space), `length` element
    /// pixels long in all (ADR-0156 §4).
    DirectionalBlur { angle: f64, length: f64 },
}

/// The figure a mask cuts in its rect (ADR-0084).
///
/// Distinct from [`Shape`], which is a *drawn* element with a paint. The `radius` a mask
/// rect can carry travels beside this on [`Effect::Mask`] rather than inside it, because
/// ADR-0084 gives all three shapes one field set and lets `shape` say only which figure is
/// drawn in it.
#[derive(Debug, Clone, PartialEq)]
pub enum MaskShape {
    /// The largest circle inscribed in the mask rect — diameter `min(width, height)`,
    /// centred on that rect.
    Circle,
    /// The mask rect itself, its corners rounded by the mask's `radius`.
    Rect,
    /// The ellipse inscribed in the mask rect.
    Ellipse,
    /// A closed outline (ADR-0163), already resolved, clamped into its box and offset by
    /// the mask rect's `x` and `y`: in element space, as the other three are drawn. Filled
    /// by the nonzero rule, the rule a `path` element fills with, so the rect plays no part
    /// in drawing it — it bounded the points, and never scales them.
    Path(Vec<PathEl>),
}

/// The rect a mask shape is inscribed in — **element-local, in unscaled element units**,
/// with `(0, 0)` at the element rect's top-left whatever the element's `origin` keyword is
/// (ADR-0084).
///
/// Held as `f64` rather than the document's integers because this crate names no type of
/// the crate that reads the document, and everything that reaches the rasterizer is
/// numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaskRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl MaskRect {
    /// The identity value: the element's own rect, `(0, 0, width, height)`.
    ///
    /// ADR-0084 makes this an *arithmetic* default rather than a second case — a
    /// param-less `mask` and one spelling the element's rect out are the same declaration
    /// here, which is what the identity golden asserts on the frame.
    fn of(extent: Extent) -> Self {
        MaskRect {
            x: 0.0,
            y: 0.0,
            width: extent.width,
            height: extent.height,
        }
    }

    fn rect(self) -> Rect {
        Rect::from_xywh(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

impl MaskShape {
    /// Everything an element box of `extent` holds **except** the shape cut in `rect` — an
    /// inverse-filled path, in element space.
    ///
    /// `rect` is the mask rect, defaulting to the element's own; `radius` rounds a
    /// `MaskShape::Rect`'s corners and is ignored by the other two, which the model has
    /// already refused it on.
    ///
    /// Inverse rather than the shape itself, because a mask has to erase what it does not
    /// select and a draw call only ever reaches the pixels its own geometry covers.
    /// Painting the shape in `DstIn` looks like the right gesture and does nothing at all:
    /// inside the shape it multiplies by an alpha of 1, and outside it there is no draw.
    /// The complement, painted in `Clear`, is the operation — and antialiased, so the
    /// mask's edge is a coverage ramp rather than a staircase.
    ///
    /// The path is built in **element space**, which is what makes the mask ride the
    /// transform (ADR-0084): [`Canvas::in_element_space`] has already translated, rotated
    /// and scaled the canvas by the time this is drawn, so a `rect` mask on a rotated
    /// element paints a rotated rectangle.
    ///
    /// With `invert` the eraser is the shape itself: the mask keeps the outside and
    /// erases the inside, through the same `Clear` draw (ADR-0152 §1).
    fn eraser(&self, extent: Extent, rect: Option<MaskRect>, radius: f64, invert: bool) -> Path {
        let mut path = self.figure(extent, rect, radius);
        if !invert {
            path.set_fill_type(PathFillType::InverseWinding);
        }
        path
    }

    /// The shape itself, cut in `rect` (the element's own where `None`), in element space.
    fn figure(&self, extent: Extent, rect: Option<MaskRect>, radius: f64) -> Path {
        let rect = rect.unwrap_or_else(|| MaskRect::of(extent));
        let mut path = PathBuilder::new();
        match self {
            MaskShape::Circle => {
                let diameter = rect.width.min(rect.height) as f32;
                path.add_oval(
                    Rect::from_xywh(
                        rect.x as f32 + (rect.width as f32 - diameter) / 2.0,
                        rect.y as f32 + (rect.height as f32 - diameter) / 2.0,
                        diameter,
                        diameter,
                    ),
                    None,
                    None,
                );
            }
            MaskShape::Rect => {
                // One integer radius, both axes — and the `radius > 0` guard is the drawn
                // `rect`'s own, a few hundred lines below, rather than a second rule about
                // radii written here (ADR-0014).
                if radius > 0.0 {
                    let radius = radius as f32;
                    path.add_rrect(RRect::new_rect_xy(rect.rect(), radius, radius), None, None);
                } else {
                    path.add_rect(rect.rect(), None, None);
                }
            }
            MaskShape::Ellipse => {
                path.add_oval(rect.rect(), None, None);
            }
            // `path_of` is the `path` element's own outline builder; its default fill
            // type is the nonzero winding rule.
            MaskShape::Path(outline) => return path_of(outline),
        }
        path.detach()
    }

    /// Erase what this mask does not keep from the layer `canvas` is drawing into, in
    /// element space (ADR-0084, ADR-0152).
    ///
    /// **`feather` at or below `0` is the hard edge**: [`MaskShape::eraser`] painted in
    /// `Clear`, antialiased, exactly as before `feather` existed, so a written `0` paints
    /// the bytes an omitted one does.
    ///
    /// **A feather blurs the shape's coverage, not the picture** (ADR-0152 §2). The shape
    /// is drawn opaque into a layer whose paint carries `blur`'s own image filter at
    /// σ = [`sigma`]`(feather)`, and that layer is composited `DstIn` (keep the blurred
    /// inside) or, inverted, `DstOut` (keep 1 − it). So the softness goes through the same
    /// matrix decomposition a `blur` does: it rides the transform and is anisotropic under
    /// a non-uniform scale, and an inverted mask keeps the exact complement of the plain
    /// one. Measured by the prototype ([#696](https://github.com/MBehtemam/Montagent/issues/696)),
    /// byte-identical across painters and with the blur bounds hint on and off.
    ///
    /// Two choices of this mechanism's own, neither visible (at most 2 levels from the
    /// unbounded form, and both fixed per frame so no painter can differ):
    ///
    /// - the layer is bounded to the shape's bounds outset by 2 × ⌈3σ⌉, so the blur runs
    ///   over the shape and its reach rather than over the whole frame (~15 ms per erase at
    ///   1080p against ~55 ms);
    /// - a plain mask first clears, hard, everything beyond one reach of the shape, where
    ///   the blurred shape is already 0. `DstIn` reaches only as far as the layer, so
    ///   without it what lies beyond the layer would be kept.
    fn erase(
        &self,
        canvas: &skia_safe::Canvas,
        extent: Extent,
        rect: Option<MaskRect>,
        radius: f64,
        invert: bool,
        feather: f64,
    ) {
        let mut eraser = SkPaint::default();
        eraser.set_anti_alias(true);
        eraser.set_blend_mode(BlendMode::Clear);
        if feather <= 0.0 {
            canvas.draw_path(&self.eraser(extent, rect, radius, invert), &eraser);
            return;
        }

        let sigma = sigma(feather);
        let reach = (3.0 * sigma).ceil();
        let figure = self.figure(extent, rect, radius);
        let bounds = *figure.bounds();
        if !invert {
            let mut beyond = Path::rect(bounds.with_outset((reach, reach)), None);
            beyond.set_fill_type(PathFillType::InverseWinding);
            eraser.set_anti_alias(false);
            canvas.draw_path(&beyond, &eraser);
        }
        let mut soft = SkPaint::default();
        soft.set_image_filter(image_filters::blur((sigma, sigma), None, None, None));
        soft.set_blend_mode(if invert {
            BlendMode::DstOut
        } else {
            BlendMode::DstIn
        });
        let layer_bounds = bounds.with_outset((2.0 * reach, 2.0 * reach));
        canvas.save_layer(&SaveLayerRec::default().bounds(&layer_bounds).paint(&soft));
        let mut opaque = SkPaint::default();
        opaque.set_anti_alias(true);
        canvas.draw_path(&figure, &opaque);
        canvas.restore();
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
/// Raised at [#280](https://github.com/MBehtemam/Montagent/issues/280).
fn sigma(radius: f64) -> f32 {
    (radius.max(0.0) / 2.0) as f32
}

/// Skia's luma weights, which are the ones its own `SkColorMatrix::setSaturation` uses.
/// Stated here rather than borrowed from a header so that the matrix below can be read
/// without one.
const LUMA: [f32; 3] = [0.213, 0.715, 0.072];

/// ADR-0088's keyer, as one SkSL colour filter.
///
/// **A runtime effect rather than a colour matrix**, because a key is not a linear function
/// of the pixel: the output alpha is a *distance* thresholded against two numbers, and no
/// 4x5 matrix has a threshold in it. This is the one effect in the vocabulary that cannot
/// be spelled as [`Effect::matrix`].
///
/// # The three readings this shader makes, none of which an ADR fixes
///
/// ADR-0088 specifies the parameters, their bounds and their identity values, and says
/// `tolerance` is a *"normalised distance in the chroma plane"*. It does not say which
/// chroma plane, how the distance is normalised, or what `spill` does arithmetically. Three
/// readings are made here, and they are recorded rather than assumed — the same treatment
/// [`sigma`] gives `blur`'s radius.
///
/// - **BT.601 chroma, and the distance normalised by `sqrt(2)`.** This is `ffmpeg`'s
///   `chromakey`, which is what every number in ADR-0088's evidence table was measured
///   through: the fixture's plateau of 0.05-0.30 and its silent 0.01 are readings of *that*
///   filter's tolerance axis, and a keyer whose axis was scaled differently would reproduce
///   none of them. Matching it is what makes `chroma_key_scan.sh`'s numbers an acceptance
///   test of this code rather than of `ffmpeg` alone.
/// - **`softness` is the blend band above `tolerance`**, so alpha ramps linearly from 0 at
///   `tolerance` to 1 at `tolerance + softness`. At `softness: 0` the comparison is the
///   hard one, which keeps ADR-0088's *"identity `0`: a hard, binary matte"* exact rather
///   than approached.
/// - **`spill` removes the key's own chroma component from what the matte keeps.** The
///   pixel's chroma is projected onto the key's chroma direction and `spill` x that
///   projection is subtracted, luma untouched — so `spill: 1` leaves a retained pixel with
///   no screen colour in it at all, and `spill: 0` provably touches nothing. Conditioning on
///   the projection is what ADR-0088 names as the reason the member is not reproducible by
///   composing `tint`/`saturation`: those reach every pixel, and this reaches the ones
///   carrying screen colour.
///
/// Raised at [#342](https://github.com/MBehtemam/Montagent/issues/342) alongside ADR-0088's
/// own unmeasured edges, since a reading is not a decision.
///
/// # Premultiplied in, premultiplied out
///
/// Skia hands a runtime colour filter **premultiplied** colour, and the key is a question
/// about the pixel's own colour rather than about its colour already faded — so the shader
/// unpremultiplies, keys, and premultiplies the result against the alpha it computed. A
/// keyer that read premultiplied channels would find a half-transparent green pixel a
/// different colour from an opaque one, and `[mask, chroma]` would key a different set of
/// pixels from `[chroma, mask]` for a reason nothing in ADR-0040's ordering rule predicts.
/// `a_half_transparent_screen_pixel_keys_like_an_opaque_one` is what holds it.
const KEYER: &str = r"
uniform float keyR;
uniform float keyG;
uniform float keyB;
uniform float tolerance;
uniform float softness;
uniform float spill;

// BT.601 chroma, centred on zero: the (U, V) `ffmpeg`'s `chromakey` measures in.
float2 chroma_of(float3 rgb) {
    return float2(
        -0.168736 * rgb.r - 0.331264 * rgb.g + 0.500000 * rgb.b,
         0.500000 * rgb.r - 0.418688 * rgb.g - 0.081312 * rgb.b);
}

half4 main(half4 color) {
    // ADR-0088's stated identity, and it is a guard rather than a value the arithmetic
    // below happens to produce. At `tolerance: 0` the blend band still opens when
    // `softness` is set, and a pixel sitting exactly on the key would be keyed by it --
    // so chroma{tolerance: 0, softness: 0.3} would key a green screen outright while
    // ADR-0088 says the whole member reduces to a no-op at tolerance 0, and while
    // N-CHROMA-INERT tells its author it keys nothing. The identity wins.
    if (tolerance <= 0.0) { return color; }
    float a = float(color.a);
    if (a <= 0.0) { return color; }
    float3 rgb = float3(color.rgb) / a;

    float2 key = chroma_of(float3(keyR, keyG, keyB));
    float2 uv = chroma_of(rgb);
    // Normalised by the longest distance the plane holds, so `tolerance` runs 0-1.
    float distance = length(uv - key) / sqrt(2.0);

    float keep = softness > 0.0
        ? clamp((distance - tolerance) / softness, 0.0, 1.0)
        : (distance > tolerance ? 1.0 : 0.0);

    // Despill, on what the matte keeps. `length(key)` is zero only for a grey screen
    // colour, which has no chroma direction to suppress along and no spill to remove.
    float weight = dot(key, key);
    if (spill > 0.0 && weight > 0.0) {
        float projection = dot(uv, key) / weight;
        if (projection > 0.0) {
            uv -= spill * projection * key;
            float luma = 0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b;
            rgb = clamp(
                float3(luma + 1.402000 * uv.y,
                       luma - 0.344136 * uv.x - 0.714136 * uv.y,
                       luma + 1.772000 * uv.x),
                0.0, 1.0);
        }
    }

    float out_a = a * keep;
    return half4(half3(rgb * out_a), half(out_a));
}
";

impl Effect {
    /// This effect as an image filter over whatever was painted before it, or `None` for
    /// [`Effect::Mask`] — which is a geometric restriction rather than a filter, and is
    /// applied by [`Canvas::in_element_space`] as a `DstIn` draw over its own layer.
    fn filter(&self) -> Option<ImageFilter> {
        match *self {
            // A `grain` is drawn over its own layer by [`Canvas::through`], as a mask is.
            Effect::Mask { .. } | Effect::Grain { .. } => None,
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
                // [#280](https://github.com/MBehtemam/Montagent/issues/280).
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
            // ADR-0156's three, each a runtime effect of Montagent's own ([`named`]). An
            // identity value is no filter at all, so it paints a plain layer: the same bytes
            // as no member even under a rotation, where a filter layer resamples.
            Effect::Posterize { levels } => named::posterize(levels),
            Effect::Glow {
                threshold,
                radius,
                intensity,
            } => named::glow(threshold, sigma(radius), intensity),
            Effect::DirectionalBlur { angle, length } => named::directional_blur(angle, length),
            // The one member with a threshold in it, so the one that is a runtime effect
            // rather than a matrix ([`KEYER`]).
            Effect::Chroma {
                colour,
                tolerance,
                softness,
                spill,
            } => {
                let [r, g, b, _] = colour.0;
                image_filters::color_filter(
                    Effect::keyer(&[
                        f32::from(r) / 255.0,
                        f32::from(g) / 255.0,
                        f32::from(b) / 255.0,
                        tolerance as f32,
                        softness as f32,
                        spill as f32,
                    ])?,
                    None,
                    None,
                )
            }
        }
    }

    /// [`KEYER`] bound to one set of uniforms, or `None` if this Skia declined to build it.
    ///
    /// Compiled on every call rather than cached: `filter` already builds a fresh
    /// `ImageFilter` per effect per element per frame, and a cache keyed on six floats would
    /// be a second lifetime to reason about for a compile Skia itself memoises.
    ///
    /// The uniforms are six bare `float`s in declaration order — never a `float3` — so the
    /// bytes below are the packing SkSL asks for without a layout rule having to be
    /// remembered here.
    fn keyer(uniforms: &[f32; 6]) -> Option<ColorFilter> {
        let bytes: Vec<u8> = uniforms.iter().flat_map(|v| v.to_ne_bytes()).collect();
        RuntimeEffect::make_for_color_filter(KEYER, None)
            .ok()?
            .make_color_filter(Data::new_copy(&bytes), None)
    }

    /// The four colour scalars as one row-major colour matrix (ADR-0049).
    ///
    /// Skia applies the matrix to **unpremultiplied** colour and premultiplies afterwards,
    /// so the alpha row is identity in every one of them and a transparent pixel stays
    /// transparent however hard it is tinted. The translation column is in unit rather
    /// than byte range, which is why `brightness` writes `amount` and not `amount × 255`.
    fn matrix(&self) -> [f32; 20] {
        match *self {
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
            // first, which is the point of spelling the four out rather than `_`.
            Effect::Blur { .. }
            | Effect::Shadow { .. }
            | Effect::Mask { .. }
            | Effect::Chroma { .. }
            | Effect::Grain { .. }
            | Effect::Posterize { .. }
            | Effect::Glow { .. }
            | Effect::DirectionalBlur { .. } => [
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
///
/// `Clone` is a reference-count bump on the decoded image, not a copy of the pixels — which
/// is what lets `render` decode a still once and paint it on every frame it is present in.
#[derive(Clone)]
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
    ///
    /// **The pixels are decoded here, once.** `from_encoded` alone returns a lazy image
    /// that decodes into Skia's resource cache, whose default 32 MiB limit is smaller than
    /// one project's stills; each eviction meant a fresh decode, 10.7% of a paint-heavy
    /// render's samples (#648). `make_raster_image` makes the `Raster` own its pixels, so the
    /// one `Painter::stills` keeps for the render is decoded once per painter.
    /// `a_decoded_still_holds_its_pixels_rather_than_a_lazy_generator` holds this.
    pub fn decode(bytes: &[u8]) -> Option<Raster> {
        Some(Raster {
            image: Image::from_encoded(Data::new_copy(bytes))?
                .make_raster_image(None, skia_safe::image::CachingHint::Disallow)?,
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

/// The running sum of N motion-blur samples, byte by byte (ADR-0155 §4).
///
/// **Integer, in fixed order, correctly rounded.** Each byte of the mean is
/// `(Σ + ⌊N/2⌋) / N`: the sum is exact in `u32` for up to 16 843 009 samples, and the
/// quotient rounds half up. So N identical samples give back exactly the source bytes, and
/// the mean of premultiplied samples is premultiplied, since a colour byte at or below its
/// alpha in every sample stays at or below it in the sum and in the rounded quotient.
///
/// ADR-0155 says "correctly rounded" and not which way a tie goes; half up is the
/// prototype's (#718) and is recorded in `compositing.md`.
#[derive(Debug, Clone)]
pub struct Accumulation {
    sums: Vec<u32>,
    samples: u32,
}

impl Accumulation {
    /// An empty sum over `len` bytes.
    pub fn new(len: usize) -> Accumulation {
        Accumulation {
            sums: vec![0; len],
            samples: 0,
        }
    }

    /// Add one sample's bytes. A sample of another length is a caller's bug; its bytes past
    /// the shorter of the two are ignored.
    pub fn add(&mut self, bytes: &[u8]) {
        for (sum, byte) in self.sums.iter_mut().zip(bytes) {
            *sum += u32::from(*byte);
        }
        self.samples += 1;
    }

    /// The rounded mean of every sample added, or all zeros where none was.
    pub fn mean(&self) -> Vec<u8> {
        if self.samples == 0 {
            return vec![0; self.sums.len()];
        }
        let (n, half) = (self.samples, self.samples / 2);
        self.sums
            .iter()
            .map(|sum| ((sum + half) / n) as u8)
            .collect()
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

/// The surface one frame is painted on — at the project's **true** pixel dimensions
/// through [`Canvas::new`], and at a proxy tier's through [`Canvas::scaled`], which is
/// `preview`'s alone.
///
/// ADR-0021 is explicit that `frame` is never proxy-scaled: the half-scale default is a
/// property of the *answer*, applied once at encode time, not of the raster. Painting at
/// half and reporting a frame would make the picture a different picture from the one
/// `render` produces, which is the whole thing this verb exists to let an agent believe.
/// `preview` is the one caller entitled to a smaller surface, and it discloses the tier
/// every time (ADR-0021); `render` is entitled to none at all (ADR-0067).
pub struct Canvas {
    surface: Surface,
    width: i32,
    height: i32,
    /// The base scale [`Canvas::scaled`] sets before any draw, `(1, 1)` at true pixels. A
    /// [`Canvas::layer`] takes the same, so a sample painted on it lands where it would on
    /// this canvas.
    base: (f32, f32),
    /// Whether this canvas is a disclosed proxy or true pixels, which decides how a raster
    /// shrunk onto it is read ([`sampling_for`], ADR-0186). Set by the constructor and
    /// never by the scale: [`Canvas::scaled`] is a proxy, [`Canvas::new`] is not.
    fidelity: Fidelity,
}

/// What a canvas's pixels are for, which is the one thing besides the draw's own matrix
/// that decides how a raster is sampled onto it (ADR-0132 as scoped by ADR-0186).
///
/// ADR-0132's sampling rule is the **deliverable's**: `render`, `frame` and
/// `preview --full` paint true pixels, the caller asked for that quality, and they get
/// trilinear minification and Catmull-Rom magnification. A proxy tier is already disclosed
/// as not true pixels (ADR-0065) and runs against an enforced wall clock (ADR-0021), so it
/// may read a shrunk raster through one mip level instead of blending two — the trilinear
/// read is what #552 measured taking the fixture's scrub preview from about 5.6 s to 9 s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fidelity {
    /// True pixels: the deliverable's sampling, ADR-0132's rule unchanged.
    True,
    /// A `preview` proxy tier (720p or 540p): minification reads the nearest mip level.
    Proxy,
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
            base: (1.0, 1.0),
            fidelity: Fidelity::True,
        })
    }

    /// A surface smaller than the project's frame, painted in the project's own
    /// coordinates — `preview`'s proxy tier ([`crate::proxy`]), and nothing else.
    ///
    /// The scale is a base matrix set once, before any draw, and every element's own
    /// `save`/`restore` nests inside it: so **nothing above this line knows it is painting
    /// a proxy**. The painter resolves the same numbers on the same document and paints
    /// the same pass in the same order; only the device it lands on is smaller. That is
    /// what makes a proxy frame the render's picture rather than a second picture that has
    /// to be kept in agreement with it — and it is also why the saving is real, since the
    /// resample that dominates a frame's cost is done once into the small surface rather
    /// than at full size and thrown away (ADR-0021).
    ///
    /// `frame` and `render` never call this: ADR-0021 keeps `frame` at true pixels so the
    /// agent has one tool it can trust for pixel-accurate checks, and forbids the
    /// deliverable being quietly downsampled.
    ///
    /// **It is a proxy, and samples like one** ([`Fidelity::Proxy`], ADR-0186): a raster
    /// shrunk onto it reads the nearest mip level rather than blending two, because the
    /// scrub budget is enforced here and true-pixel quality is not promised.
    pub fn scaled(width: i64, height: i64, scale: (f64, f64)) -> Option<Canvas> {
        if !(scale.0.is_finite() && scale.1.is_finite()) || scale.0 <= 0.0 || scale.1 <= 0.0 {
            return None;
        }
        let mut canvas = Canvas::new(width, height)?;
        canvas.fidelity = Fidelity::Proxy;
        canvas.base = (scale.0 as f32, scale.1 as f32);
        canvas.surface.canvas().scale(canvas.base);
        Some(canvas)
    }

    /// A transparent surface of this canvas's size and base scale: one motion-blur sample
    /// is painted on it (ADR-0155 §4), read back with [`Canvas::rgba`], and summed into an
    /// [`Accumulation`].
    ///
    /// **Unbounded** (ADR-0144 §9): the layer is the whole frame. A bound would have to be
    /// measured byte-identical first, and the prototype that measured this field (#718)
    /// measured none.
    pub fn layer(&self) -> Option<Canvas> {
        let mut layer = Canvas::new(i64::from(self.width), i64::from(self.height))?;
        layer.base = self.base;
        layer.fidelity = self.fidelity;
        layer.surface.canvas().clear(Color::TRANSPARENT);
        layer.surface.canvas().scale(layer.base);
        Some(layer)
    }

    /// Composite a whole-frame layer of premultiplied RGBA8 bytes — an [`Accumulation`]'s
    /// mean — over this canvas once, in `blend` (ADR-0155 §4: the samples are averaged, then
    /// blended). Drawn at device pixels, one source pixel on one destination pixel, so the
    /// average is not resampled. `false` where `rgba` is not this canvas's size.
    pub fn composite_layer(&mut self, rgba: &[u8], blend: Blend) -> bool {
        let info = ImageInfo::new(
            ISize::new(self.width, self.height),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        if rgba.len() != self.width as usize * self.height as usize * 4 {
            return false;
        }
        let Some(image) =
            images::raster_from_data(&info, Data::new_copy(rgba), self.width as usize * 4)
        else {
            return false;
        };
        let canvas = self.surface.canvas();
        canvas.save();
        canvas.reset_matrix();
        let mut paint = SkPaint::default();
        paint.set_blend_mode(blend.mode());
        canvas.draw_image(&image, (0, 0), Some(&paint));
        canvas.restore();
        true
    }

    /// Paint the whole frame one colour — the project's `background`.
    pub fn background(&mut self, colour: Rgba) {
        self.surface.canvas().clear(colour.colour());
    }

    /// The frame's pixels as packed RGB8, row-major, composited over opaque black.
    ///
    /// What `render` hands to the encoder, one frame at a time. Over black for the same
    /// reason [`Canvas::encode`] clears its output to black first: the deliverable carries no
    /// alpha, and a translucent `background` must encode to the same picture the `frame`
    /// verb shows for it. The surface is premultiplied, and a premultiplied colour *is* the
    /// colour over black — so dropping the alpha byte is the composite, not an
    /// approximation of one.
    ///
    /// Three bytes rather than four because every one of them crosses a pipe to a
    /// subprocess: at 1080x1920 that is 2 MB per frame saved, 1631 times on the committed
    /// fixture.
    pub fn rgb(&mut self) -> Option<Vec<u8>> {
        Some(
            self.rgba()?
                .chunks_exact(4)
                .flat_map(|px| [px[0], px[1], px[2]])
                .collect(),
        )
    }

    /// The frame's pixels as packed **premultiplied** RGBA8, row-major — the surface's own
    /// bytes, alpha included.
    ///
    /// [`Canvas::rgb`] is this composited over black, and is what the encoder is fed. This
    /// is what a caller asking about *transparency* needs: `measure`'s keyed-alpha coverage
    /// reading (ADR-0088) counts these alpha bytes, and it is the same surface `render`
    /// paints, through the same effects, rather than a second keyer written to answer the
    /// question.
    pub fn rgba(&mut self) -> Option<Vec<u8>> {
        let info = ImageInfo::new(
            ISize::new(self.width, self.height),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let row = self.width as usize * 4;
        let mut rgba = vec![0u8; row * self.height as usize];
        if !self.surface.image_snapshot().read_pixels(
            &info,
            &mut rgba,
            row,
            (0, 0),
            skia_safe::image::CachingHint::Allow,
        ) {
            return None;
        }
        Some(rgba)
    }

    /// The frame as it stands, as a still that can be drawn somewhere else.
    ///
    /// A reference to the surface's pixels at this moment, not a live view: painting the next
    /// frame on this canvas does not change a snapshot already taken. `frame`'s range mode
    /// takes one per tile and [`Canvas::composite`]s it onto the sheet (ADR-0095).
    pub fn snapshot(&mut self) -> Raster {
        Raster {
            image: self.surface.image_snapshot(),
        }
    }

    /// Draw `source` into `into`, resampled to exactly that rectangle — a contact sheet's
    /// tile, composited down from a frame painted at true pixels (ADR-0095 §5).
    ///
    /// **Mipmapped, always** — it is always a reduction, and it predates the per-draw rule
    /// ([`sampling_for`], ADR-0132) that now reaches the same answer for an element shrunk
    /// the same way. A tile is a frame reduced
    /// five- or sixfold, and a bilinear read of that samples four source pixels out of
    /// thirty-odd: a one-pixel stroke either vanishes or survives by where it happens to
    /// fall. Reading from the mip level nearest the reduction averages every pixel, so what
    /// a tile shows depends on the picture and not on the grid.
    pub fn composite(&mut self, source: &Raster, into: Region) {
        let paint = SkPaint::default();
        self.surface.canvas().draw_image_rect_with_sampling_options(
            &source.image,
            None,
            into.rect(),
            SamplingOptions::new(skia_safe::FilterMode::Linear, skia_safe::MipmapMode::Linear),
            &paint,
        );
    }

    /// Draw one `rect` or `ellipse` (ADR-0014), its stroke dashed by `dash` (ADR-0158 §5).
    ///
    /// An undashed stroke is Skia's own rect, rounded rect or oval on the inset box. A dashed
    /// one is drawn on [`inset_outline`]'s explicit outline instead, which starts and runs as
    /// ADR-0158 §5's table says. Its dash ends are butt, and a rect's corners inside a dash
    /// keep the miter join every rect stroke has.
    ///
    /// A trimmed stroke (ADR-0160) is cut from that same explicit outline, so the window is
    /// measured from where the dashes start; its ends are butt. `trim` is `None` with no
    /// trim and for a full window, both of which draw exactly as before trim existed.
    #[allow(clippy::too_many_arguments)]
    pub fn shape(
        &mut self,
        shape: Shape,
        extent: Extent,
        transform: &Transform,
        paint: &Fill,
        dash: Option<&Dash>,
        trim: Option<Trim>,
        clip: Option<Region>,
        effects: &[Effect],
    ) {
        if paint.fill.is_none() && paint.stroke.is_none() {
            return;
        }
        self.in_element_space(extent, transform, clip, effects, |canvas| {
            let box_rect = Rect::from_xywh(0.0, 0.0, extent.width as f32, extent.height as f32);

            if let Some(ink) = &paint.fill {
                let mut fill = ink.paint((0.0, 0.0));
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

            let (Some(ink), true) = (&paint.stroke, paint.stroke_width > 0.0) else {
                return;
            };
            // ADR-0014: on a shape the stroke falls **inside** the declared rect. Skia
            // centres a stroke on the path, so the path is the box inset by half the
            // width — "centred or outside, it would occupy 992x177 at (44,1449), eating
            // 4 px of the 48 px margin the layout rests on".
            let inset = paint.stroke_width as f32 / 2.0;
            // The same box as the fill (ADR-0149 §2): the stroke's gradient is measured
            // against the declared rect, not the inset path.
            let mut stroke = ink.paint((0.0, 0.0));
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
            match trim {
                Some(Trim::Empty) => return,
                Some(Trim::Part { from, to }) => {
                    let Some(outline) = inset_outline(shape, extent, paint.stroke_width) else {
                        return;
                    };
                    let cut = trimmed(canvas, &outline, (from, to), dash);
                    canvas.draw_path(&cut, &stroke);
                    return;
                }
                None => {}
            }
            if let Some(effect) = dash.and_then(Dash::effect) {
                let Some(outline) = inset_outline(shape, extent, paint.stroke_width) else {
                    return;
                };
                stroke.set_path_effect(effect);
                canvas.draw_path(&outline, &stroke);
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

    /// Draw one `path` (ADR-0154): `outline` in element space, where the declared box is
    /// `(0, 0, width, height)`.
    ///
    /// The fill uses the nonzero winding rule, so a self-intersecting outline fills its
    /// overlap. The stroke is **centred** on the outline, unlike a `rect`'s, with `style`'s
    /// join and cap: those reach no further than the reach factor times half the stroke's
    /// width from the outline (ADR-0158 §4), which is what lets `validate` prove from the
    /// points alone that the box contains the ink. Nothing is clipped to the box.
    #[allow(clippy::too_many_arguments)]
    pub fn path(
        &mut self,
        outline: &[PathEl],
        extent: Extent,
        transform: &Transform,
        paint: &Fill,
        style: StrokeStyle,
        clip: Option<Region>,
        effects: &[Effect],
    ) {
        if paint.fill.is_none() && paint.stroke.is_none() {
            return;
        }
        let mut path = path_of(outline);
        path.set_fill_type(PathFillType::Winding);
        self.in_element_space(extent, transform, clip, effects, |canvas| {
            // A gradient on either is measured against the declared box (ADR-0149 §2), which
            // is the element space this runs in.
            if let Some(ink) = &paint.fill {
                let mut fill = ink.paint((0.0, 0.0));
                fill.set_style(PaintStyle::Fill);
                canvas.draw_path(&path, &fill);
            }
            let (Some(ink), true) = (&paint.stroke, paint.stroke_width > 0.0) else {
                return;
            };
            let mut stroke = ink.paint((0.0, 0.0));
            stroke.set_style(PaintStyle::Stroke);
            stroke.set_stroke_width(paint.stroke_width as f32);
            style.apply(&mut stroke);
            match style.trim {
                // The fill is never trimmed (ADR-0160 §3); an empty window draws no stroke.
                Some(Trim::Empty) => {}
                Some(Trim::Part { from, to }) => {
                    let cut = trimmed(canvas, &path, (from, to), style.dash.as_ref());
                    canvas.draw_path(&cut, &stroke);
                }
                None => {
                    if let Some(effect) = style.dash.as_ref().and_then(Dash::effect) {
                        stroke.set_path_effect(effect);
                    }
                    canvas.draw_path(&path, &stroke);
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
        let fidelity = self.fidelity;
        self.in_element_space(extent, transform, clip, effects, |canvas| {
            let destination = Rect::from_xywh(0.0, 0.0, extent.width as f32, extent.height as f32);
            let to_device = canvas.local_to_device_as_3x3()
                * Matrix::rect_2_rect(Rect::from(source.image.bounds()), destination, None)
                    .unwrap_or_default();
            let paint = SkPaint::default();
            canvas.draw_image_rect_with_sampling_options(
                &source.image,
                None,
                destination,
                sampling_for(&to_device, fidelity),
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
        // Built once per element and keyed by the same index `montagent-text` deduplicated
        // on, so a repeated letter is one path however many times it appears.
        let paths: Vec<Path> = outlines.iter().map(|outline| path_of(outline)).collect();

        self.in_element_space(extent, transform, clip, effects, |canvas| {
            // **A fading stagger unit is one layer, between the stroke pass and the fill
            // pass** (ADR-0151 §2, as the accepted prototype #683 measured it). Every other
            // glyph keeps the two passes: all strokes, then all fills. A body below opacity 1
            // draws its own strokes then its own fills into one layer that takes its opacity,
            // so its stroke never shows through its own fill; its fill still covers its
            // neighbours' strokes, and its neighbours' fills still cover its stroke. Two
            // overlapping fading bodies stack whole, the later over the earlier. With no
            // fading body this is exactly the two passes over every glyph, so an idle stagger
            // draws the same bytes as no stagger.
            let fading = |glyph: &Glyph| glyph.unit.is_some_and(|unit| unit.opacity < 1.0);
            let solid: Vec<&Glyph> = glyphs.iter().filter(|glyph| !fading(glyph)).collect();
            draw_pass(canvas, &paths, &solid, Pass::Stroke);
            let mut bodies: Vec<(usize, f32)> = Vec::new();
            for unit in glyphs.iter().filter(|g| fading(g)).filter_map(|g| g.unit) {
                if !bodies.iter().any(|(body, _)| *body == unit.body) {
                    bodies.push((unit.body, unit.opacity));
                }
            }
            for (body, opacity) in bodies {
                // A body at opacity 0 draws nothing at all.
                if opacity <= 0.0 {
                    continue;
                }
                let members: Vec<&Glyph> = glyphs
                    .iter()
                    .filter(|glyph| glyph.unit.is_some_and(|unit| unit.body == body))
                    .collect();
                // Bounded to the body's own ink: a full-canvas layer per fading letter costs
                // about 1.4 ms a frame at 1080p, a bounded one about 8 µs (#683). The bound is
                // a function of the glyphs alone, so it is the same on every painter.
                canvas.save_layer_alpha_f(unit_bounds(&paths, &members), opacity);
                draw_pass(canvas, &paths, &members, Pass::Stroke);
                draw_pass(canvas, &paths, &members, Pass::Fill);
                canvas.restore();
            }
            draw_pass(canvas, &paths, &solid, Pass::Fill);
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
        draw: impl Fn(&skia_safe::Canvas),
    ) {
        if extent.width <= 0.0 || extent.height <= 0.0 || transform.opacity <= 0.0 {
            return;
        }
        // A projected element facing away, edge-on, or reaching the eye paints nothing at
        // this instant (ADR-0167 §4, §5): the same answer `query --at` and the checks read.
        if let Some(projection) = transform.projection
            && !projection::quad(extent, transform, projection, effects).drawn
        {
            return;
        }
        let device = Device {
            base: self.base,
            fidelity: self.fidelity,
        };
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
        // [#280](https://github.com/MBehtemam/Montagent/issues/280).
        //
        // **The blend is the same layer's paint** (ADR-0147): effects, then `opacity`, then
        // the mode, so the finished element, shadow included, is composited once inside
        // `clip`. No bounds hint: the layer is the element's whole paint, and a hint here
        // would be a second place for its edge to be decided.
        let layered = transform.opacity < 1.0 || transform.blend != Blend::Normal;
        if transform.blend == Blend::Normal {
            if layered {
                canvas.save_layer_alpha_f(None, transform.opacity as f32);
            }
        } else {
            let mut paint = SkPaint::default();
            paint.set_alpha_f(transform.opacity.min(1.0) as f32);
            paint.set_blend_mode(transform.blend.mode());
            canvas.save_layer(&SaveLayerRec::default().paint(&paint));
        }
        canvas.translate((transform.x as f32, transform.y as f32));
        if transform.rotation != 0.0 {
            canvas.rotate(transform.rotation as f32, None);
        }
        canvas.scale((transform.scale.0 as f32, transform.scale.1 as f32));
        match transform.projection {
            None => {
                canvas.translate((
                    (-transform.origin.0 * extent.width) as f32,
                    (-transform.origin.1 * extent.height) as f32,
                ));
                Canvas::through(canvas, extent, effects, draw);
            }
            Some(projection) => {
                projected(
                    canvas, extent, transform, projection, device, effects, &draw,
                );
            }
        }
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
    /// **A `mask` is not a filter and is not a clip.** It is [`MaskShape::eraser`] —
    /// everything the shape does not cover — painted over its own layer in `Clear`, which
    /// erases the layer everywhere the mask does not select. A clip would have done the
    /// same thing more cheaply and been wrong for a specific reason: a clip established
    /// *before* an enclosing layer restricts that layer's bounds, so `[mask, blur]` would
    /// hand the blur an input already cut to the mask and the blur would lose every
    /// contribution from just outside it. Painting the complement keeps each effect's
    /// input the full element.
    ///
    /// **A `blur` or `shadow` layer carries a bounds hint** where [`layer_bound`]'s
    /// preconditions hold, which stops it blurring a frame's worth of transparent pixels
    /// and paints the same bytes as the unbounded layer. That is why `draw` is `Fn`: the
    /// hint's right and bottom come from recording the element once before painting it.
    fn through(
        canvas: &skia_safe::Canvas,
        extent: Extent,
        effects: &[Effect],
        draw: impl Fn(&skia_safe::Canvas),
    ) {
        // No effect, bounds hint, grain plan or directional crop ever runs under a
        // perspective matrix: a projected element runs this on its flat layer (ADR-0167 §3).
        // A hard assert, on in release, because each of those reads the matrix.
        assert!(
            !canvas.local_to_device_as_3x3().has_perspective(),
            "an effect chain under a perspective matrix (ADR-0167 §3)"
        );
        // A `grain` at `amount: 0` is no member at all, rather than a layer that changes
        // nothing: the identity paints the bytes of the list without it (ADR-0156).
        let effects: Vec<Effect> = effects
            .iter()
            .filter(|effect| !matches!(effect, Effect::Grain { amount, .. } if *amount <= 0.0))
            .cloned()
            .collect();
        let effects = effects.as_slice();
        let mut filters: Vec<Option<ImageFilter>> = effects.iter().map(|e| e.filter()).collect();
        Canvas::crop_directional(canvas, effects, &mut filters, &draw);
        let bounds = layer_bound::hints(canvas, effects, &filters, &draw);
        let grains = grain::plan(canvas, effects, &filters, &draw);
        for (filter, bound) in filters.into_iter().zip(&bounds).rev() {
            match filter {
                Some(filter) => {
                    let mut paint = SkPaint::default();
                    paint.set_image_filter(filter);
                    let mut layer = SaveLayerRec::default().paint(&paint);
                    if let Some(bound) = bound {
                        layer = layer.bounds(bound);
                    }
                    canvas.save_layer(&layer);
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
        for (effect, grain) in effects.iter().zip(&grains) {
            if let Some(grain) = grain {
                grain.apply(canvas);
            }
            if let Effect::Mask {
                shape,
                rect,
                radius,
                invert,
                feather,
            } = effect
            {
                shape.erase(canvas, extent, *rect, *radius, *invert, *feather);
            }
            canvas.restore();
        }
    }

    /// Crop every `directional_blur` to its reach (ADR-0156 §4), **always**, whether or not
    /// the bounds hint is on.
    ///
    /// Skia cannot bound a runtime-shader filter, so uncropped it is evaluated over the whole
    /// layer. The crop is the element-space bounds of what the filter is given, walked down
    /// the chain from what `draw` paints, grown by [`named::directional_reach`]. It changes a
    /// few edge pixels by a level against no crop (#722), which is harmless only because it
    /// is never off: hint on and hint off paint the same cropped filter.
    fn crop_directional(
        canvas: &skia_safe::Canvas,
        effects: &[Effect],
        filters: &mut [Option<ImageFilter>],
        draw: &dyn Fn(&skia_safe::Canvas),
    ) {
        let directional = |effect: &Effect| matches!(effect, Effect::DirectionalBlur { .. });
        if !effects
            .iter()
            .zip(filters.iter())
            .any(|(effect, filter)| directional(effect) && filter.is_some())
        {
            return;
        }
        // Nothing drawn crops to nothing, so the shader still never runs over the layer.
        let mut content = layer_bound::drawn(canvas, draw).unwrap_or_else(Rect::new_empty);
        for (effect, filter) in effects.iter().zip(filters.iter_mut()) {
            let Some(built) = filter.take() else {
                continue;
            };
            match *effect {
                Effect::DirectionalBlur { angle, length } => {
                    let (cropped, bound) = named::crop_directional(built, content, angle, length);
                    *filter = cropped;
                    content = bound;
                }
                _ => {
                    content = built.compute_fast_bounds(content);
                    *filter = Some(built);
                }
            }
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
        let destination = Rect::from_xywh(0.0, 0.0, out_width as f32, out_height as f32);
        let to_device = Matrix::rect_2_rect(region.rect(), destination, None).unwrap_or_default();
        out.canvas().draw_image_rect_with_sampling_options(
            &snapshot,
            Some((&region.rect(), skia_safe::canvas::SrcRectConstraint::Strict)),
            destination,
            sampling_for(&to_device, self.fidelity),
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

/// What [`projected`] needs of the canvas it lands on: the base scale its flat layer is
/// sized by, and the fidelity its one draw is sampled at.
#[derive(Clone, Copy)]
struct Device {
    base: (f32, f32),
    fidelity: Fidelity,
}

/// Transparent layer pixels kept around a projected element's flat layer, so its edge is
/// sampled against transparency rather than against the image's clamped border.
const FLAT_PAD: f64 = 2.0;

/// Draw one element through its projection (ADR-0167 §3, as the accepted prototype #786
/// measured it). `canvas` already holds translate · rotate · scale, and the element faces
/// front with every corner in front of the eye ([`projection::quad`]).
///
/// 1. **Flat.** The element is drawn on a raster surface of its own, with its effects and
///    mask in list order, through the normal [`Canvas::through`]. The surface covers the
///    reach-widened box ([`reach_box`]) plus [`FLAT_PAD`], rounded out to whole pixels, and
///    holds `|scale|` layer pixels per element unit (times the canvas's base scale), under a
///    matrix that is a translation and a positive scale only. What the element paints
///    outside the reach box, such as text overflowing its block, is cut.
/// 2. **Projected.** That surface is drawn once through PROJECT · origin offset ·
///    1/|scale|, sampled by [`sampling_for`] (ADR-0132) and antialiased at its edge. This is
///    the only draw under a perspective matrix; `opacity`, `blend` and `clip` wrap it as they
///    wrap every element.
fn projected(
    canvas: &skia_safe::Canvas,
    extent: Extent,
    transform: &Transform,
    projection: Projection,
    device: Device,
    effects: &[Effect],
    draw: &dyn Fn(&skia_safe::Canvas),
) {
    let (kx, ky) = (
        transform.scale.0.abs() * f64::from(device.base.0),
        transform.scale.1.abs() * f64::from(device.base.1),
    );
    if !(kx > 0.0 && ky > 0.0 && kx.is_finite() && ky.is_finite()) {
        return;
    }
    let (l, t, r, b) = reach_box(extent, effects);
    // Whole layer pixels on every side, so the reach box's top-left lands at a whole-pixel
    // translation of the flat surface.
    let left = (-l * kx).ceil() + FLAT_PAD;
    let top = (-t * ky).ceil() + FLAT_PAD;
    let width = left + (r * kx).ceil() + FLAT_PAD;
    let height = top + (b * ky).ceil() + FLAT_PAD;
    if !(width.is_finite() && height.is_finite()) || width > f64::from(i32::MAX) {
        return;
    }
    let info = ImageInfo::new(
        ISize::new(width as i32, height as i32),
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
        layer.translate((left as f32, top as f32));
        layer.scale((kx as f32, ky as f32));
        layer.clip_rect(
            Rect::from_ltrb(l as f32, t as f32, r as f32, b as f32),
            None,
            Some(true),
        );
        Canvas::through(layer, extent, effects, draw);
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
    canvas.translate((-left as f32, -top as f32));
    let to_device = canvas.local_to_device_as_3x3();
    let mut paint = SkPaint::default();
    paint.set_anti_alias(true);
    canvas.draw_image_with_sampling_options(
        &image,
        (0, 0),
        sampling_for(&to_device, device.fidelity),
        Some(&paint),
    );
    canvas.restore();
}

/// Which of [`Canvas::text`]'s two passes is being painted.
enum Pass {
    Stroke,
    Fill,
}

/// The matrix a stagger unit's pose is drawn through.
fn unit_matrix(unit: &UnitDraw) -> skia_safe::Matrix {
    let [a, b, c, d, e, f] = unit.matrix;
    skia_safe::Matrix::new_all(a, b, c, d, e, f, 0.0, 0.0, 1.0)
}

/// One of [`Canvas::text`]'s passes over some glyphs.
fn draw_pass(canvas: &skia_safe::Canvas, paths: &[Path], glyphs: &[&Glyph], pass: Pass) {
    for glyph in glyphs {
        let Some(path) = paths.get(glyph.outline) else {
            continue;
        };
        let paint = match pass {
            Pass::Stroke => {
                let (Some(ink), true) = (&glyph.paint.stroke, glyph.paint.stroke_width > 0.0)
                else {
                    continue;
                };
                // A gradient is moved back by the glyph's own offset, so it is one gradient
                // across the declared box rather than one per letter (ADR-0149).
                let mut paint = ink.paint((glyph.x as f32, glyph.y as f32));
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_width(glyph.paint.stroke_width as f32 * 2.0);
                // Round joins rather than mitres: a mitre on a sharp interior angle spikes
                // out to an arbitrary length, which on a serif or a comma is a visible
                // whisker rather than a border.
                paint.set_stroke_join(skia_safe::PaintJoin::Round);
                paint
            }
            Pass::Fill => {
                let Some(ink) = &glyph.paint.fill else {
                    continue;
                };
                let mut paint = ink.paint((glyph.x as f32, glyph.y as f32));
                paint.set_style(PaintStyle::Fill);
                paint
            }
        };
        canvas.save();
        // A stagger unit's pose acts in the block's own coordinates, before the glyph is
        // moved to its place in the block.
        if let Some(unit) = &glyph.unit {
            canvas.concat(&unit_matrix(unit));
        }
        canvas.translate((glyph.x as f32, glyph.y as f32));
        // A stroke with no fill behind it has to be clipped to the outside of the contour
        // itself, or the inner half of the doubled width — the half a fill would normally
        // cover — paints over the letterform and an outlined word comes out solid. Only in
        // that case: with a fill present the clip would put an antialiased seam along every
        // contour, and the fill already does the job.
        if matches!(pass, Pass::Stroke) && glyph.paint.fill.is_none() {
            canvas.clip_path(path, skia_safe::ClipOp::Difference, true);
        }
        canvas.draw_path(path, &paint);
        canvas.restore();
    }
}

/// One fading body's ink in the block's own coordinates: every member glyph's path bounds,
/// outset by its doubled stroke and one pixel, through its unit's matrix, rounded out.
fn unit_bounds(paths: &[Path], glyphs: &[&Glyph]) -> Option<Rect> {
    let mut all: Option<Rect> = None;
    for glyph in glyphs {
        let path = paths.get(glyph.outline)?;
        let pad = glyph.paint.stroke_width as f32 * 2.0 + 1.0;
        let local = path
            .bounds()
            .with_offset((glyph.x as f32, glyph.y as f32))
            .with_outset((pad, pad));
        let mapped = match &glyph.unit {
            Some(unit) => unit_matrix(unit).map_rect(local).0,
            None => local,
        };
        all = Some(match all {
            None => mapped,
            Some(r) => Rect::join2(r, mapped),
        });
    }
    all.map(|r| <Rect as skia_safe::RoundOut<Rect>>::round_out(&r))
}

/// The inset outline a `rect`'s or `ellipse`'s stroke is drawn on, built with ADR-0158 §5's
/// start point and direction rather than Skia's default start index, so a dash pattern runs
/// from where the format says it does. `None` where a stroke wider than the box leaves no
/// inset outline to draw.
///
/// All three run clockwise on screen (y down):
///
/// - a rect with no `radius`: from the inset outline's top-left corner, along the top edge
///   first;
/// - a rect with a `radius`: from where the top-left arc meets the top edge. The inset radius
///   is the declared one less the inset, clamped to half the inset box's shorter side, as
///   Skia clamps a rounded rect's;
/// - an ellipse: from 3 o'clock, toward 6 o'clock first.
///
/// Each quarter arc is a conic of weight √2/2, Skia's own exact quarter circle.
fn inset_outline(shape: Shape, extent: Extent, stroke_width: f64) -> Option<Path> {
    let inset = stroke_width as f32 / 2.0;
    let (l, t) = (inset, inset);
    let (r, b) = (extent.width as f32 - inset, extent.height as f32 - inset);
    if r <= l || b <= t {
        return None;
    }
    let weight = std::f32::consts::FRAC_1_SQRT_2;
    let mut path = PathBuilder::new();
    match shape {
        Shape::Rect { radius } if radius > 0.0 => {
            let k = (radius as f32 - inset)
                .max(0.0)
                .min((r - l) / 2.0)
                .min((b - t) / 2.0);
            path.move_to((l + k, t));
            path.line_to((r - k, t));
            path.conic_to((r, t), (r, t + k), weight);
            path.line_to((r, b - k));
            path.conic_to((r, b), (r - k, b), weight);
            path.line_to((l + k, b));
            path.conic_to((l, b), (l, b - k), weight);
            path.line_to((l, t + k));
            path.conic_to((l, t), (l + k, t), weight);
        }
        Shape::Rect { .. } => {
            path.move_to((l, t));
            path.line_to((r, t));
            path.line_to((r, b));
            path.line_to((l, b));
        }
        Shape::Ellipse => {
            let (cx, cy) = ((l + r) / 2.0, (t + b) / 2.0);
            path.move_to((r, cy));
            path.conic_to((r, b), (cx, b), weight);
            path.conic_to((l, b), (l, cy), weight);
            path.conic_to((l, t), (cx, t), weight);
            path.conic_to((r, t), (r, cy), weight);
        }
    }
    path.close();
    Some(path.detach())
}

/// The part of `outline` a trimmed stroke draws (ADR-0160): the window `(from, to)`, in
/// fractions of the outline's length, cut from it with Skia's own path measure, after
/// `dash` has been laid along the whole outline.
///
/// The measure takes the resolution scale Skia's stroker and dasher take from the canvas
/// matrix, so a dash kept whole by the window is cut at the very distances the untrimmed
/// dash is. Each piece starts with a move; a piece that crosses a closed outline's start
/// point continues through it with no move, so the stroker joins it there with the paint's
/// own join and draws no cap.
fn trimmed(
    canvas: &skia_safe::Canvas,
    outline: &Path,
    window: (f64, f64),
    dash: Option<&Dash>,
) -> Path {
    let mut builder = PathBuilder::new();
    let Some(contour) = skia_safe::ContourMeasureIter::new(
        outline,
        false,
        res_scale(&canvas.local_to_device_as_3x3()),
    )
    .next() else {
        return builder.detach();
    };
    let length = contour.length();
    let whole = f64::from(length);
    for (lo, hi) in window_pieces(whole, contour.is_closed(), window, dash) {
        // Distances as Skia's dasher hands them to the measure: `f64` narrowed to `f32`.
        let mut segment = |lo: f64, hi: f64, moved: bool| {
            // `false` for a zero-length piece, which still appends a dot's zero-length line
            // for the stroker to cap, as it does for the dasher.
            let _ = contour.get_segment(lo as f32, hi as f32, &mut builder, moved);
        };
        if hi <= whole {
            segment(lo, hi, true);
        } else if lo >= whole {
            segment(lo - whole, hi - whole, true);
        } else {
            segment(lo, whole, true);
            segment(0.0, hi - whole, false);
        }
    }
    builder.detach()
}

/// The pieces a window keeps of one contour of `length`, as distances `(lo, hi)` with
/// `0 ≤ lo < length` and `hi ≤ lo + length`; a piece whose `hi` passes `length` runs on
/// through the start point.
///
/// Undashed, that is the window itself. Dashed, it is every dash ([`Dash::along`])
/// intersected with the window around the closed outline: each pair is compared at the
/// window's own place and one turn either side, so a dash and a window that each cross the
/// start point meet once, and a window that covers both ends of a dash keeps both. A
/// zero-length dash — a dot — is kept where it lies inside the window, ends included.
fn window_pieces(
    length: f64,
    closed: bool,
    (from, to): (f64, f64),
    dash: Option<&Dash>,
) -> Vec<(f64, f64)> {
    let start = from * length;
    let end = if from > to {
        (to + 1.0) * length
    } else {
        to * length
    };
    let Some(dash) = dash else {
        return vec![(start, end)];
    };
    let turns: &[f64] = if closed { &[-1.0, 0.0, 1.0] } else { &[0.0] };
    let mut pieces = Vec::new();
    for (on, off) in dash.along(length as f32, closed) {
        for turn in turns {
            let lo = on.max(start + turn * length);
            let hi = off.min(end + turn * length);
            if lo < hi || (on == off && lo == hi) {
                pieces.push((lo, hi));
            }
        }
    }
    pieces
}

/// The resolution scale Skia's stroker and dasher measure a path at under `matrix`
/// (`SkDraw::ComputeResScaleForStroking`): the larger of the matrix's two column lengths,
/// or 1 where that is not a positive finite number.
fn res_scale(matrix: &Matrix) -> f32 {
    let sx = (matrix.scale_x() * matrix.scale_x() + matrix.skew_y() * matrix.skew_y()).sqrt();
    let sy = (matrix.skew_x() * matrix.skew_x() + matrix.scale_y() * matrix.scale_y()).sqrt();
    let scale = sx.max(sy);
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

/// The length of the outline a `rect`'s or `ellipse`'s dash pattern runs along, by Skia's
/// own path measure over the very outline [`Canvas::shape`] dashes — informative, for
/// `query --at` (ADR-0158 §7), and never a second computation that could disagree with the
/// drawing. `None` where there is no inset outline.
pub fn shape_outline_length(shape: Shape, extent: Extent, stroke_width: f64) -> Option<f64> {
    inset_outline(shape, extent, stroke_width).map(|path| measured(&path))
}

/// The length of a `path`'s outline, by the same path measure [`Canvas::path`]'s dash runs
/// along.
pub fn path_outline_length(outline: &[PathEl]) -> f64 {
    measured(&path_of(outline))
}

/// How much finer than 1:1 a text's curve is measured: [`Curve`].
const CURVE_RESOLUTION: f32 = 64.0;

/// A text's curve (ADR-0161), measured once: its length, and the point and tangent at any
/// distance along it, by the same path measure a `path`'s dash runs along (ADR-0158), over
/// the very outline a `path` element strokes. A text's curve is one figure, so only its
/// first contour is read.
///
/// Measured at [`CURVE_RESOLUTION`] times the dash's resolution: at 1:1 Skia's measure puts a
/// point up to a tenth of a pixel from its distance along a curve (worst on a straight
/// segment, whose cubic has its handles on its ends), which a letter would show as uneven
/// spacing; at 64 it is under a hundredth.
pub struct Curve {
    measure: Option<skia_safe::ContourMeasure>,
    /// The curve's length in box pixels. `0` where the outline has no length.
    pub length: f64,
}

impl Curve {
    pub fn of(outline: &[PathEl]) -> Curve {
        let measure =
            skia_safe::ContourMeasureIter::new(&path_of(outline), false, CURVE_RESOLUTION).next();
        let length = measure
            .as_ref()
            .map_or(0.0, |measure| f64::from(measure.length()));
        Curve { measure, length }
    }

    /// The point at distance `d` along the curve and the tangent's angle there, in radians
    /// in y-down screen space. `None` where the curve has no length.
    pub fn at(&self, d: f64) -> Option<(f64, f64, f64)> {
        let (point, tangent) = self.measure.as_ref()?.pos_tan(d as f32)?;
        Some((
            f64::from(point.x),
            f64::from(point.y),
            f64::from(tangent.y).atan2(f64::from(tangent.x)),
        ))
    }
}

/// The tight box of `glyphs`' ink in the text block's own coordinates, through each glyph's
/// unit matrix and outset by its stroke, as `[left, top, right, bottom]`: what
/// [`Canvas::text`] would draw, before the element's transform. `None` where no glyph has
/// ink.
pub fn glyph_ink(glyphs: &[Glyph], outlines: &[Vec<PathEl>]) -> Option<[f64; 4]> {
    let mut all: Option<Rect> = None;
    for glyph in glyphs {
        let Some(outline) = outlines.get(glyph.outline) else {
            continue;
        };
        let mut matrix = glyph
            .unit
            .as_ref()
            .map_or_else(skia_safe::Matrix::default, unit_matrix);
        matrix.pre_translate((glyph.x as f32, glyph.y as f32));
        let bounds = path_of(outline)
            .with_transform(&matrix)
            .compute_tight_bounds();
        if bounds.is_empty() {
            continue;
        }
        let reach = if glyph.paint.stroke.is_some() {
            glyph.paint.stroke_width as f32
        } else {
            0.0
        };
        let bounds = bounds.with_outset((reach, reach));
        all = Some(match all {
            None => bounds,
            Some(r) => Rect::join2(r, bounds),
        });
    }
    all.map(|r| [r.left, r.top, r.right, r.bottom].map(f64::from))
}

/// Every contour's length, summed, by the measure Skia's dash lays a pattern along at 1:1.
///
/// Skia measures a curve by chords within a tolerance, so a curve's figure falls a little
/// short of its exact length (about 0.16% on a 98-px circle). Under an element `scale` the
/// dash is laid along a finer measure of the same outline, which is one more reason the
/// figure is informative.
fn measured(path: &Path) -> f64 {
    skia_safe::ContourMeasureIter::new(path, false, None)
        .map(|contour| f64::from(contour.length()))
        .sum()
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

/// How a source is read, chosen from `to_device` — the matrix taking **source pixels** to
/// **device pixels** for this one draw, so the declared rect, the element's transform and
/// a proxy canvas's base scale are all already in it (ADR-0132).
///
/// Three branches, in this order:
///
/// - **Identity** — no scale, no skew, no perspective, and a whole-pixel translation:
///   nearest, so the source lands pixel for pixel. Stated here rather than left to Skia's
///   pass-through, which is an implementation detail a `skia-safe` bump could move, and
///   which a cubic does not take at all. Defined on the composed matrix, never on
///   `scale`: `scale: 1` over a declared rect that is not the source's own size is a real
///   resample.
/// - **Minification** — the smaller singular value under 1, either axis: bilinear over
///   linear mipmaps. Skia's cubics ignore mipmaps and alias here worse than bilinear did
///   (#500 measured 29 dB against 38 at ×0.3), which is why one filter cannot serve both
///   directions. A perspective matrix, which only a projected element's flat layer is drawn
///   through (ADR-0167 §3), lands here too: it is the branch that cannot alias.
/// - **Magnification** — everything else: Catmull-Rom, which #500 measured at the PIL
///   bicubic reference (40.2 dB against 40.0 at ×2.3) where bilinear stair-steps a hard
///   edge (32.3).
///
/// **On a proxy canvas the minification branch reads one mip level, not two** (ADR-0186):
/// bilinear within the nearest level, which keeps most of what mipmaps buy against
/// aliasing and drops the second level's read and blend, the cost #552 measured taking the
/// fixture's scrub preview past its budget. Identity and magnification are the same on
/// both, and so is everything a true-pixel canvas paints.
fn sampling_for(to_device: &Matrix, fidelity: Fidelity) -> SamplingOptions {
    /// How far from exact a matrix may be and still be the identity. f32 composition of
    /// translate · scale(1) · translate leaves error near 1e-7; a scale wrong by 1e-6
    /// moves the far edge of a 4K source by under a hundredth of a pixel.
    const SCALE_EPSILON: f32 = 1e-6;
    /// And how far from a whole pixel a translation may sit.
    const TRANSLATE_EPSILON: f32 = 1e-3;

    let near = |value: f32, target: f32| (value - target).abs() <= SCALE_EPSILON;
    let whole = |value: f32| (value - value.round()).abs() <= TRANSLATE_EPSILON;
    if !to_device.has_perspective()
        && near(to_device.scale_x(), 1.0)
        && near(to_device.scale_y(), 1.0)
        && near(to_device.skew_x(), 0.0)
        && near(to_device.skew_y(), 0.0)
        && whole(to_device.translate_x())
        && whole(to_device.translate_y())
    {
        return SamplingOptions::new(skia_safe::FilterMode::Nearest, skia_safe::MipmapMode::None);
    }
    // `min_scale` is -1 for a perspective matrix, which therefore minifies.
    if to_device.min_scale() < 1.0 {
        let levels = match fidelity {
            Fidelity::True => skia_safe::MipmapMode::Linear,
            Fidelity::Proxy => skia_safe::MipmapMode::Nearest,
        };
        return SamplingOptions::new(skia_safe::FilterMode::Linear, levels);
    }
    SamplingOptions::from(CubicResampler::catmull_rom())
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
    use std::cell::Cell;

    // -----------------------------------------------------------------------
    // ADR-0088's keyer, at the one seam where it can be asked about a pixel
    // -----------------------------------------------------------------------
    //
    // Every assertion below reads **alpha**, because a matte operation's whole output is
    // alpha, and asserts **both directions** — the colour that must key and the colour
    // that must not — which is the rule `chroma_key_scan.sh` follows for the same reason:
    // a keyer that keys everything passes a one-sided test.

    /// The screen colour ADR-0088's forcing case measures: `(0, 205, 0)` at every one of
    /// its 25 sample points.
    const SCREEN: Rgba = Rgba([0x00, 0xCD, 0x00, 0xFF]);

    /// One 2x1 element — `left` then `right` — painted through `effects`, as premultiplied
    /// RGBA.
    ///
    /// A raster rather than a shape, because that is what a keyed element is: the pixels
    /// come from a source and the keyer reads them, rather than being a paint this crate
    /// chose.
    fn keyed(left: [u8; 4], right: [u8; 4], effects: &[Effect]) -> Vec<u8> {
        let mut pixels = left.to_vec();
        pixels.extend_from_slice(&right);
        let source = Raster::from_rgba(&pixels, 2, 1).expect("a 2x1 source");
        let mut canvas = Canvas::new(2, 1).expect("a surface");
        canvas.raster(
            &source,
            Extent {
                width: 2.0,
                height: 1.0,
            },
            &Transform {
                x: 0.0,
                y: 0.0,
                scale: (1.0, 1.0),
                rotation: 0.0,
                opacity: 1.0,
                blend: Blend::Normal,
                projection: None,
                origin: (0.0, 0.0),
            },
            None,
            effects,
        );
        canvas.rgba().expect("the surface reads back")
    }

    fn chroma(tolerance: f64, softness: f64, spill: f64) -> Effect {
        Effect::Chroma {
            colour: SCREEN,
            tolerance,
            softness,
            spill,
        }
    }

    #[test]
    fn the_keyer_keys_the_screen_colour_and_keeps_what_is_not_it() {
        // Both directions in one frame: the screen goes, the subject stays. A keyer that
        // erased the whole element would satisfy only the first half.
        let out = keyed(
            SCREEN.0,
            [0xC8, 0x64, 0x32, 0xFF],
            &[chroma(0.10, 0.0, 0.0)],
        );
        assert_eq!(out[3], 0, "the screen pixel is keyed out: {out:?}");
        assert_eq!(out[7], 0xFF, "the subject pixel is untouched: {out:?}");
        assert_eq!(
            [out[4], out[5], out[6]],
            [0xC8, 0x64, 0x32],
            "and keeps the RGB it was given — a matte operation, not a colour one"
        );
    }

    #[test]
    fn tolerance_zero_is_the_identity_whatever_the_other_two_parameters_say() {
        // ADR-0088's documented identity, which is what makes the member a no-op rather
        // than a member with no off switch. Asserted on the screen colour itself, the one
        // pixel any non-identity tolerance would take.
        let out = keyed(SCREEN.0, SCREEN.0, &[chroma(0.0, 0.0, 0.0)]);
        assert_eq!([out[3], out[7]], [0xFF, 0xFF], "{out:?}");

        // And with the other two parameters set, which is where the band arithmetic alone
        // would have keyed it: `(distance - 0) / softness` is 0 at the key, and 0 is
        // transparent. ADR-0088 says "the whole member reducing to a no-op at
        // `tolerance: 0`", and `N-CHROMA-INERT` tells its author it "keys nothing" — both
        // are false for this document unless the identity outranks the formula.
        let out = keyed(SCREEN.0, SCREEN.0, &[chroma(0.0, 0.3, 1.0)]);
        assert_eq!([out[3], out[7]], [0xFF, 0xFF], "{out:?}");
        assert_eq!(
            [out[0], out[1], out[2]],
            [SCREEN.0[0], SCREEN.0[1], SCREEN.0[2]],
            "and `spill` did not touch the RGB either: {out:?}"
        );
    }

    #[test]
    fn softness_zero_is_a_hard_matte_and_a_band_above_it_is_partial() {
        // A colour sitting inside the blend band: keyed softly, keyed hard, kept entirely
        // — three answers about one pixel, decided by `softness` alone.
        let edge = [0x40, 0xA0, 0x40, 0xFF];
        let hard = keyed(edge, edge, &[chroma(0.02, 0.0, 0.0)]);
        assert_eq!(hard[3], 0xFF, "outside a hard key, nothing is partial");

        let soft = keyed(edge, edge, &[chroma(0.02, 0.20, 0.0)]);
        assert!(
            (1..0xFF).contains(&soft[3]),
            "inside the blend band the alpha ramps: {}",
            soft[3]
        );
    }

    #[test]
    fn spill_zero_provably_leaves_rgb_untouched_and_spill_one_removes_the_screen_cast() {
        // ADR-0088 admits `spill` as a colour operation on ADR-0049's clauses, one of
        // which is a documented identity value. This is that clause, measured — and its
        // other end, so the parameter is not merely inert.
        let cast = [0x50, 0xB0, 0x50, 0xFF];
        let none = keyed(cast, cast, &[chroma(0.01, 0.0, 0.0)]);
        assert_eq!([none[0], none[1], none[2]], [0x50, 0xB0, 0x50]);

        let despilled = keyed(cast, cast, &[chroma(0.01, 0.0, 1.0)]);
        assert!(
            despilled[1] < none[1],
            "the green cast is suppressed: {} vs {}",
            despilled[1],
            none[1]
        );
        assert_eq!(despilled[3], 0xFF, "and the pixel is still opaque");
    }

    #[test]
    fn a_half_transparent_screen_pixel_keys_like_an_opaque_one() {
        // The premultiplication reading in [`KEYER`]'s own doc, as a test. A shader reading
        // premultiplied channels would see `(0, 103, 0)` here and measure it against the
        // key as a different colour, so `[mask, chroma]` and `[chroma, mask]` would key
        // different pixels for a reason ADR-0040's ordering rule does not predict.
        let half = [0x00, 0xCD, 0x00, 0x80];
        let out = keyed(half, half, &[chroma(0.10, 0.0, 0.0)]);
        assert_eq!(out[3], 0, "{out:?}");
    }

    #[test]
    fn the_tolerance_plateau_adr_0088_measured_is_flat_here_too() {
        // ADR-0088's forcing case: 0.05 through 0.30 all key identically, which is the
        // measurement that makes one static `tolerance` usable at all. Asserted against a
        // *noisy* screen pixel — the fixture is h264 and its screen arrives a digit or two
        // off `(0, 205, 0)` — and against a subject colour that must survive every one of
        // them, so a wider key fails here as loudly as a broken one.
        let noisy = [0x04, 0xC9, 0x06, 0xFF];
        let subject = [0xC8, 0x64, 0x32, 0xFF];
        for tolerance in [0.05, 0.10, 0.20, 0.30] {
            let out = keyed(noisy, subject, &[chroma(tolerance, 0.0, 0.0)]);
            assert_eq!(out[3], 0, "the screen keys at {tolerance}: {out:?}");
            assert_eq!(out[7], 0xFF, "the subject survives {tolerance}: {out:?}");
        }
        // And the cliff below it, which is what tells the plateau from a keyer that keys
        // whatever it is given: ADR-0088 measured `tolerance: 0.01` keying nothing.
        let out = keyed(noisy, subject, &[chroma(0.01, 0.0, 0.0)]);
        assert_eq!([out[3], out[7]], [0xFF, 0xFF], "{out:?}");
    }

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
                unit: None,
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
                blend: Blend::Normal,
                projection: None,
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
    fn a_composited_tile_averages_its_source_rather_than_sampling_four_pixels_of_it() {
        // One-pixel black and white columns, reduced fivefold. A bilinear read lands on one
        // column's centre and comes back black or white, by where it fell; a mipmapped one
        // averages the columns to grey.
        let mut stripes = vec![0u8; 100 * 100 * 4];
        for (i, px) in stripes.chunks_exact_mut(4).enumerate() {
            let level = if (i % 100) % 2 == 0 { 0 } else { 255 };
            px.copy_from_slice(&[level, level, level, 255]);
        }
        let source = Raster::from_rgba(&stripes, 100, 100).expect("100x100 RGBA");
        let mut sheet = Canvas::new(20, 20).expect("a surface");
        sheet.background(Rgba::BLACK);
        let whole = Region {
            x: 0,
            y: 0,
            width: 20,
            height: 20,
        };
        sheet.composite(&source, whole);
        let png = sheet
            .encode(None, Scale::Full, Encoding::Png)
            .expect("encodes");
        for x in [9, 10] {
            let [r, g, b, _] = pixel_at(&png.bytes, x, 10);
            assert!(
                (80..=175).contains(&r),
                "({r}, {g}, {b}) at x={x} is not grey"
            );
        }
    }

    #[test]
    fn a_snapshot_keeps_the_frame_it_was_taken_of() {
        let red = Rgba([0xFF, 0x00, 0x00, 0xFF]);
        let mut frame = Canvas::new(4, 4).expect("a surface");
        frame.background(red);
        let first = frame.snapshot();
        frame.background(Rgba::BLACK);

        let mut sheet = Canvas::new(4, 4).expect("a surface");
        sheet.composite(
            &first,
            Region {
                x: 0,
                y: 0,
                width: 4,
                height: 4,
            },
        );
        let png = sheet
            .encode(None, Scale::Full, Encoding::Png)
            .expect("encodes");
        assert_eq!(pixel_at(&png.bytes, 2, 2), [0xFF, 0x00, 0x00, 0xFF]);
    }

    #[test]
    fn a_glyph_stroke_with_no_fill_behind_it_leaves_the_letterform_open() {
        // ADR-0014 puts a text stroke *outside* the contour. The stroke is painted at
        // twice the declared width and normally relies on the fill to cover the inner
        // half — so with no fill, an unclipped stroke would flood the interior and an
        // outlined word would come out solid. The interior must stay the background.
        let hollow = one_glyph(Fill {
            fill: None,
            stroke: Some(Rgba([0x00, 0x00, 0xFF, 0xFF]).into()),
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
            fill: Some(Rgba([0xFF, 0x00, 0x00, 0xFF]).into()),
            stroke: Some(Rgba([0x00, 0x00, 0xFF, 0xFF]).into()),
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

    // -----------------------------------------------------------------------
    // ADR-0132's sampling rule: the matrix decides, never the `scale` field
    // -----------------------------------------------------------------------

    fn nearest() -> SamplingOptions {
        SamplingOptions::new(skia_safe::FilterMode::Nearest, skia_safe::MipmapMode::None)
    }

    fn mipmapped() -> SamplingOptions {
        SamplingOptions::new(skia_safe::FilterMode::Linear, skia_safe::MipmapMode::Linear)
    }

    fn cubic() -> SamplingOptions {
        SamplingOptions::from(CubicResampler::catmull_rom())
    }

    /// How far a proxy's shrunk read may sit from true pixels' on `rings` at ×0.3, as a
    /// mean delta over all four channels (ADR-0186). Measured when committed on skia-safe
    /// 0.153.2: 3.58 through one mip level, 11.64 with no mipmaps. The ceiling leaves room
    /// for a `skia-safe` bump and none for losing the mipmaps.
    const PROXY_MEAN_DELTA: f64 = 4.5;

    /// Bilinear within the nearest mip level: a proxy's minifier (ADR-0186).
    fn one_level() -> SamplingOptions {
        SamplingOptions::new(
            skia_safe::FilterMode::Linear,
            skia_safe::MipmapMode::Nearest,
        )
    }

    /// ADR-0132's rule as the deliverable reads it.
    fn true_pixels(to_device: &Matrix) -> SamplingOptions {
        sampling_for(to_device, Fidelity::True)
    }

    /// A source of `native` pixels drawn into a `declared` box under `transform`, as the
    /// one matrix [`Canvas::raster`] hands the rule.
    fn drawn(native: (f32, f32), declared: (f32, f32), transform: Matrix) -> Matrix {
        transform
            * Matrix::rect_2_rect(
                Rect::from_wh(native.0, native.1),
                Rect::from_wh(declared.0, declared.1),
                None,
            )
            .expect("a non-empty source")
    }

    #[test]
    fn an_identity_draw_at_a_whole_pixel_offset_is_nearest() {
        for (x, y) in [(0.0, 0.0), (37.0, -12.0), (1919.0, 1079.0)] {
            assert_eq!(
                true_pixels(&Matrix::translate((x, y))),
                nearest(),
                "at ({x}, {y})"
            );
        }
        // And within float drift of one, which is what f32 composition leaves.
        let drifted = Matrix::new_all(
            1.0 + 1e-7,
            0.0,
            40.0004,
            0.0,
            1.0 - 1e-7,
            7.9998,
            0.0,
            0.0,
            1.0,
        );
        assert_eq!(true_pixels(&drifted), nearest());
    }

    #[test]
    fn a_fractional_offset_at_scale_one_is_not_the_identity() {
        // Nearest here would move every pixel by half of one; it is a resample, and at
        // scale 1 it is the magnification branch.
        assert_eq!(true_pixels(&Matrix::translate((10.5, 0.0))), cubic());
        assert_eq!(true_pixels(&Matrix::translate((0.0, 3.25))), cubic());
    }

    #[test]
    fn scale_one_over_a_declared_rect_that_is_not_the_native_size_is_a_resample() {
        // The case that makes the rule a fact about the matrix and not about `scale`.
        assert_eq!(
            true_pixels(&drawn(
                (100.0, 100.0),
                (120.0, 120.0),
                Matrix::new_identity()
            )),
            cubic()
        );
        assert_eq!(
            true_pixels(&drawn((100.0, 100.0), (80.0, 80.0), Matrix::new_identity())),
            mipmapped()
        );
        // And the native size declared is the identity again.
        assert_eq!(
            true_pixels(&drawn(
                (100.0, 100.0),
                (100.0, 100.0),
                Matrix::translate((4.0, 4.0))
            )),
            nearest()
        );
    }

    #[test]
    fn uniform_scales_fall_either_side_of_one() {
        for (scale, expected) in [
            (0.3, mipmapped()),
            (0.999, mipmapped()),
            (1.0, nearest()),
            (1.001, cubic()),
            (2.3, cubic()),
        ] {
            assert_eq!(
                true_pixels(&Matrix::scale((scale, scale))),
                expected,
                "at ×{scale}"
            );
        }
    }

    #[test]
    fn a_rotation_is_never_the_identity_and_its_scale_decides() {
        let turned = |degrees: f32, scale: f32| {
            let mut m = Matrix::new_identity();
            m.pre_rotate(degrees, None).pre_scale((scale, scale), None);
            m
        };
        assert_eq!(true_pixels(&turned(30.0, 1.0)), cubic());
        assert_eq!(true_pixels(&turned(5.0, 2.0)), cubic());
        assert_eq!(true_pixels(&turned(30.0, 0.58)), mipmapped());
    }

    #[test]
    fn one_axis_under_one_is_minification_whatever_the_other_does() {
        // The smaller singular value, not the mean: a source stretched 2× wide and
        // squeezed to half height aliases along its height.
        assert_eq!(true_pixels(&Matrix::scale((2.0, 0.5))), mipmapped());
        assert_eq!(true_pixels(&Matrix::scale((0.9, 3.0))), mipmapped());
    }

    #[test]
    fn a_perspective_matrix_takes_the_branch_that_cannot_alias() {
        let m = Matrix::new_all(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.001, 0.0, 1.0);
        assert_eq!(true_pixels(&m), mipmapped());
    }

    // -----------------------------------------------------------------------
    // ADR-0186: the rule is the deliverable's, and a proxy trades one branch of it
    // -----------------------------------------------------------------------

    #[test]
    fn a_proxy_reads_a_shrunk_raster_through_one_mip_level_and_nothing_else_moves() {
        let mut turned = Matrix::new_identity();
        turned.pre_rotate(30.0, None).pre_scale((0.58, 0.58), None);
        let minified = [
            Matrix::scale((0.3, 0.3)),
            // The fixture's 1536×2720 stills on the 720p proxy of its 1080×1920 frame.
            Matrix::scale((0.47, 0.47)),
            Matrix::scale((0.999, 0.999)),
            Matrix::scale((2.0, 0.5)),
            turned,
            Matrix::new_all(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.001, 0.0, 1.0),
        ];
        for m in minified {
            assert_eq!(true_pixels(&m), mipmapped(), "true pixels at {m:?}");
            assert_eq!(
                sampling_for(&m, Fidelity::Proxy),
                one_level(),
                "a proxy at {m:?}"
            );
        }
        let unchanged = [
            Matrix::translate((37.0, -12.0)),
            Matrix::translate((10.5, 0.0)),
            Matrix::scale((1.001, 1.001)),
            Matrix::scale((2.3, 2.3)),
        ];
        for m in unchanged {
            assert_eq!(
                sampling_for(&m, Fidelity::Proxy),
                true_pixels(&m),
                "at {m:?}"
            );
        }
    }

    #[test]
    fn only_a_scaled_canvas_and_its_layers_are_proxies() {
        // The fidelity is the constructor's, never read off the scale: a true-pixel canvas
        // is true pixels whatever its elements' `scale`, and a proxy's motion-blur layer
        // samples like the proxy it is composited onto.
        let true_pixels = Canvas::new(64, 36).expect("a surface");
        assert_eq!(true_pixels.fidelity, Fidelity::True);
        assert_eq!(
            true_pixels.layer().expect("a layer").fidelity,
            Fidelity::True
        );
        let proxy = Canvas::scaled(64, 36, (0.5, 0.5)).expect("a surface");
        assert_eq!(proxy.fidelity, Fidelity::Proxy);
        assert_eq!(proxy.layer().expect("a layer").fidelity, Fidelity::Proxy);
    }

    /// `sampling-shrunk`'s source (ADR-0132's golden): rings and hatching a few source
    /// pixels wide, on transparency, supersampled 4×4 so the source itself is not aliased.
    fn rings(size: u32) -> Raster {
        const SUB: u32 = 4;
        let c = f64::from(size) / 2.0;
        let mut rgba = Vec::with_capacity((size * size * 4) as usize);
        for py in 0..size {
            for px in 0..size {
                let (mut sum, mut covered) = ([0u32; 3], 0u32);
                for sy in 0..SUB {
                    for sx in 0..SUB {
                        let x = f64::from(px) + (f64::from(sx) + 0.5) / f64::from(SUB);
                        let y = f64::from(py) + (f64::from(sy) + 0.5) / f64::from(SUB);
                        let r = ((x - c).powi(2) + (y - c).powi(2)).sqrt();
                        if r > c * 0.95 {
                            continue;
                        }
                        let ring = (r % 12.0) < 3.0;
                        let hatch = ((x + y) % 10.0) < 2.0 && x < c;
                        let rgb: [u8; 3] = if ring || hatch {
                            [0x1E, 0x34, 0x4C]
                        } else {
                            [0xFB, 0xF3, 0xE3]
                        };
                        covered += 1;
                        for k in 0..3 {
                            sum[k] += u32::from(rgb[k]);
                        }
                    }
                }
                // Straight alpha, as `Raster::from_rgba` takes it.
                for k in sum {
                    rgba.push(k.checked_div(covered).unwrap_or(0) as u8);
                }
                rgba.push((covered * 255 / (SUB * SUB)) as u8);
            }
        }
        Raster::from_rgba(&rgba, size, size).expect("a source")
    }

    /// The mean absolute channel difference between two same-sized RGBA buffers.
    fn mean_delta(a: &[u8], b: &[u8]) -> f64 {
        assert_eq!(a.len(), b.len());
        let sum: u64 = a
            .iter()
            .zip(b)
            .map(|(x, y)| u64::from(x.abs_diff(*y)))
            .sum();
        sum as f64 / a.len() as f64
    }

    #[test]
    fn a_raster_shrunk_onto_a_proxy_reads_one_mip_level_and_stays_near_true_pixels() {
        // The same 400 px source landing at ×0.3 on a 120 px device two ways: through an
        // element `scale` on a true-pixel canvas, and through a proxy's base scale. The
        // device matrix is the same, so only the canvas's fidelity can tell them apart.
        const SIZE: u32 = 400;
        const OUT: i64 = 120;
        let source = rings(SIZE);
        let extent = Extent {
            width: f64::from(SIZE),
            height: f64::from(SIZE),
        };
        let paint = |mut canvas: Canvas, at: Transform| {
            canvas.background(Rgba([0xF2, 0xA6, 0x5A, 0xFF]));
            canvas.raster(&source, extent, &at, None, &[]);
            canvas.rgba().expect("the surface reads back")
        };
        let half = f64::from(SIZE) / 2.0;
        let true_pixels = paint(
            Canvas::new(OUT, OUT).expect("a surface"),
            at(OUT as f64 / 2.0, OUT as f64 / 2.0, (0.3, 0.3), 0.0),
        );
        let proxy = paint(
            Canvas::scaled(OUT, OUT, (0.3, 0.3)).expect("a surface"),
            at(half, half, (1.0, 1.0), 0.0),
        );

        // The reference reads: the same draw straight onto a surface, under each sampler.
        let direct = |sampling: SamplingOptions| {
            let info = ImageInfo::new(
                ISize::new(OUT as i32, OUT as i32),
                ColorType::RGBA8888,
                AlphaType::Premul,
                None,
            );
            let mut surface = surfaces::raster(&info, None, None).expect("a surface");
            let canvas = surface.canvas();
            canvas.clear(Rgba([0xF2, 0xA6, 0x5A, 0xFF]).colour());
            canvas.draw_image_rect_with_sampling_options(
                &source.image,
                None,
                Rect::from_wh(OUT as f32, OUT as f32),
                sampling,
                &SkPaint::default(),
            );
            let mut out = Canvas {
                surface,
                width: OUT as i32,
                height: OUT as i32,
                base: (1.0, 1.0),
                fidelity: Fidelity::True,
            };
            out.rgba().expect("the surface reads back")
        };
        let bilinear = direct(SamplingOptions::new(
            skia_safe::FilterMode::Linear,
            skia_safe::MipmapMode::None,
        ));

        // Each canvas reads with its own sampler, and the proxy's is the cheaper one.
        assert!(mean_delta(&true_pixels, &direct(mipmapped())) < 0.05);
        assert!(mean_delta(&proxy, &direct(one_level())) < 0.05);
        assert_ne!(
            proxy, true_pixels,
            "the proxy read the same mip levels as true pixels"
        );

        // And the trade is bounded: one mip level stays far nearer the deliverable's read
        // than no mipmaps at all, the sampling ADR-0132 retired.
        let proxy_delta = mean_delta(&proxy, &true_pixels);
        let bilinear_delta = mean_delta(&bilinear, &true_pixels);
        eprintln!("one mip level: {proxy_delta:.3}; no mipmaps: {bilinear_delta:.3}");
        assert!(
            proxy_delta <= PROXY_MEAN_DELTA,
            "a proxy's shrunk raster is {proxy_delta:.3} from true pixels, over \
             {PROXY_MEAN_DELTA} (ADR-0186)"
        );
        assert!(
            proxy_delta * 2.0 < bilinear_delta,
            "one mip level ({proxy_delta:.3}) is no longer clearly better than none \
             ({bilinear_delta:.3})"
        );
    }

    #[test]
    fn an_identity_draw_paints_the_source_byte_for_byte() {
        // The identity branch's promise, read off the surface: every byte of an opaque
        // source comes back where it was, at a whole-pixel offset, through the real
        // element path. A filter applied at 1:1 — Mitchell's blur, or a bilinear read at a
        // half-pixel phase — fails it.
        let (w, h) = (7u32, 5u32);
        let source: Vec<u8> = (0..w * h)
            .flat_map(|i| {
                [
                    (i * 37 % 251) as u8,
                    (i * 91 % 241) as u8,
                    (i * 13 % 239) as u8,
                    0xFF,
                ]
            })
            .collect();
        let raster = Raster::from_rgba(&source, w, h).expect("a source");
        let mut canvas = Canvas::new(12, 9).expect("a surface");
        canvas.raster(
            &raster,
            Extent {
                width: w as f64,
                height: h as f64,
            },
            &Transform {
                x: 3.0,
                y: 2.0,
                scale: (1.0, 1.0),
                rotation: 0.0,
                opacity: 1.0,
                blend: Blend::Normal,
                projection: None,
                origin: (0.0, 0.0),
            },
            None,
            &[],
        );
        let out = canvas.rgba().expect("the surface reads back");
        for row in 0..h as usize {
            let at = ((row + 2) * 12 + 3) * 4;
            let from = row * w as usize * 4;
            assert_eq!(
                &out[at..at + w as usize * 4],
                &source[from..from + w as usize * 4],
                "row {row}"
            );
        }
    }

    #[test]
    fn a_decoded_still_holds_its_pixels_rather_than_a_lazy_generator() {
        // Why a lazy one is wrong is on `Raster::decode`.
        let mut canvas = Canvas::new(4, 4).expect("a surface");
        canvas.background(Rgba([0x20, 0x40, 0x60, 0xFF]));
        let png = canvas
            .encode(None, Scale::Full, Encoding::Png)
            .expect("an encoded still")
            .bytes;
        let still = Raster::decode(&png).expect("it decodes");
        assert!(!still.image.is_lazy_generated());
    }

    // -----------------------------------------------------------------------
    // #652's filter-layer hint: the same bytes with it as without it
    // -----------------------------------------------------------------------
    //
    // Every case paints one element twice on the same thread, hint on and hint off, and
    // compares every byte. A case the hint should apply to also asserts that it did, since
    // a hint that never fires paints the same bytes for nothing.

    const FRAME: (i64, i64) = (640, 360);

    fn at(x: f64, y: f64, scale: (f64, f64), rotation: f64) -> Transform {
        Transform {
            x,
            y,
            origin: (0.5, 0.5),
            scale,
            rotation,
            opacity: 1.0,
            blend: Blend::Normal,
            projection: None,
        }
    }

    fn blur(radius: f64) -> Effect {
        Effect::Blur { radius }
    }

    fn shadow(dx: f64, dy: f64, radius: f64) -> Effect {
        Effect::Shadow {
            dx,
            dy,
            radius,
            colour: Rgba([0xFF, 0x9F, 0x2E, 0xFF]),
            opacity: 0.8,
        }
    }

    /// One stroked ellipse through `effects`, on `canvas()`, with the hint `on` or off; the
    /// pixels and how many layers were hinted.
    fn painted_once(
        canvas: &dyn Fn() -> Canvas,
        transform: &Transform,
        effects: &[Effect],
        on: bool,
    ) -> (Vec<u8>, usize) {
        let mut canvas = canvas();
        canvas.background(Rgba([0x10, 0x14, 0x18, 0xFF]));
        bound_filter_layers(on);
        layer_bound::HINTED.with(|hinted| hinted.set(0));
        canvas.shape(
            Shape::Ellipse,
            Extent {
                width: 120.0,
                height: 70.0,
            },
            transform,
            &Fill {
                fill: Some(Rgba([0xF2, 0xF2, 0xF2, 0xFF]).into()),
                stroke: Some(Rgba([0xFF, 0x3B, 0x30, 0xFF]).into()),
                stroke_width: 9.0,
            },
            None,
            None,
            None,
            effects,
        );
        bound_filter_layers(true);
        let hinted = layer_bound::HINTED.with(Cell::get);
        (canvas.rgba().expect("the surface reads back"), hinted)
    }

    /// The bytes are the same both ways; how many layers the hinted paint hinted.
    #[track_caller]
    fn same_bytes(canvas: &dyn Fn() -> Canvas, transform: &Transform, effects: &[Effect]) -> usize {
        let (bounded, hinted) = painted_once(canvas, transform, effects, true);
        let (unbounded, off) = painted_once(canvas, transform, effects, false);
        assert_eq!(off, 0, "off hints nothing");
        let differ = bounded
            .chunks(4)
            .zip(unbounded.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            differ, 0,
            "{differ} pixels differ with the hint, at {transform:?} through {effects:?}"
        );
        hinted
    }

    fn frame() -> Canvas {
        Canvas::new(FRAME.0, FRAME.1).expect("a surface")
    }

    #[test]
    fn a_hinted_blur_or_shadow_paints_the_bytes_the_unbounded_layer_does() {
        let chains: [&[Effect]; 9] = [
            &[blur(14.0)],
            &[shadow(0.0, 0.0, 28.0)],
            &[shadow(7.5, -3.25, 20.0)],
            &[blur(2.0)],
            &[blur(10.0), shadow(20.0, 20.0, 30.0)],
            &[
                Effect::Mask {
                    shape: MaskShape::Ellipse,
                    rect: None,
                    radius: 0.0,
                    invert: false,
                    feather: 0.0,
                },
                blur(20.0),
            ],
            &[
                Effect::Mask {
                    shape: MaskShape::Ellipse,
                    rect: None,
                    radius: 0.0,
                    invert: true,
                    feather: 0.0,
                },
                blur(20.0),
            ],
            // Feathered (#698), plain and inverted, a fractional feather among them.
            &[
                Effect::Mask {
                    shape: MaskShape::Ellipse,
                    rect: None,
                    radius: 0.0,
                    invert: false,
                    feather: 18.0,
                },
                blur(20.0),
            ],
            &[
                blur(6.0),
                Effect::Mask {
                    shape: MaskShape::Rect,
                    rect: Some(MaskRect {
                        x: 20.0,
                        y: 10.0,
                        width: 80.0,
                        height: 50.0,
                    }),
                    radius: 12.0,
                    invert: true,
                    feather: 13.37,
                },
                shadow(7.5, -3.25, 20.0),
            ],
        ];
        // The frame's four edges, scales either side of 1, and each drifting by fractions
        // of a pixel: one position alone can miss a hint that moves the layer's origin,
        // which flips coverage on only some edge pixels at only some offsets.
        let places: Vec<Transform> = [
            (320.37, 180.5, (1.0, 1.0)),
            (-20.3, 40.6, (1.4, 1.4)),
            (630.6, 350.25, (0.4, 0.4)),
            (200.1, 300.9, (2.0, 0.5)),
        ]
        .into_iter()
        .flat_map(|(x, y, scale)| {
            (0..4).map(move |k| at(x + 0.37 * f64::from(k), y + 0.21 * f64::from(k), scale, 0.0))
        })
        .collect();
        for effects in chains {
            for place in &places {
                let hinted = same_bytes(&frame, place, effects);
                let layers = effects.iter().filter(|e| e.filter().is_some()).count();
                assert_eq!(hinted, layers, "every filter layer hinted: {effects:?}");
            }
        }
    }

    #[test]
    fn a_flipped_element_pins_the_edge_that_lands_on_the_layer_s_left_and_top() {
        // A negative scale keeps the matrix scale+translate, so it is the layer's own and
        // flips layer space: the element's right edge is the layer's left.
        for scale in [(-1.0, 1.0), (1.0, -1.3), (-2.0, -0.6)] {
            for effects in [&[blur(14.0)][..], &[shadow(24.0, 18.0, 24.0)]] {
                let place = at(300.4, 170.7, scale, 0.0);
                assert_eq!(same_bytes(&frame, &place, effects), 1, "{scale:?}");
            }
        }
        // Rotated as well: the layer is a positive scale again and the flip goes to the
        // composite.
        let place = at(300.4, 170.7, (-1.5, 1.0), 30.0);
        assert_eq!(same_bytes(&frame, &place, &[blur(24.0)]), 1);
    }

    #[test]
    fn a_rotated_or_skewed_element_is_hinted_in_its_layer_s_positive_scale() {
        for rotation in [30.0, 90.0, 180.0, 233.0] {
            let place = at(320.4, 180.2, (1.6, 0.7), rotation);
            assert_eq!(same_bytes(&frame, &place, &[shadow(10.0, 10.0, 20.0)]), 1);
        }
        // `preview`'s proxy surface scales the frame unevenly, which makes a rotated
        // element's matrix a skew.
        let proxy = || Canvas::scaled(FRAME.0, FRAME.1, (0.5, 0.31)).expect("a surface");
        let place = at(640.4, 360.2, (1.2, 1.2), 30.0);
        assert_eq!(same_bytes(&proxy, &place, &[blur(16.0)]), 1);
    }

    #[test]
    fn a_layer_sigma_past_skia_s_rescale_takes_the_unbounded_path() {
        // σ = radius / 2 × scale: 150 and 150, both over 135.
        for (radius, scale) in [(300.0, 1.0), (100.0, 3.0)] {
            let place = at(320.4, 180.2, (scale, scale), 0.0);
            assert_eq!(same_bytes(&frame, &place, &[blur(radius)]), 0);
        }
        // 134 is under it.
        let place = at(320.4, 180.2, (1.0, 1.0), 0.0);
        assert_eq!(same_bytes(&frame, &place, &[blur(268.0)]), 1);
    }

    #[test]
    fn a_colour_filter_anywhere_in_the_list_leaves_every_layer_unbounded() {
        let tint = || Effect::Tint {
            colour: Rgba([0x30, 0x60, 0xFF, 0xFF]),
            amount: 0.5,
        };
        let place = at(320.4, 180.2, (1.0, 1.0), 0.0);
        assert_eq!(same_bytes(&frame, &place, &[blur(14.0), tint()]), 0);
        assert_eq!(
            same_bytes(&frame, &place, &[tint(), shadow(0.0, 0.0, 20.0)]),
            0
        );
    }

    #[test]
    fn posterize_and_a_directional_blur_pass_the_bound_on_and_a_glow_takes_it() {
        // ADR-0156 §4: `posterize` keeps the bound. #722: its own layer and a directional
        // blur's stay unhinted, and every blur, shadow or glow layer around them keeps its
        // hint. Flat, rotated and flipped.
        let posterize = || Effect::Posterize { levels: 4.0 };
        let smear = || Effect::DirectionalBlur {
            angle: 30.0,
            length: 24.0,
        };
        let glow = || Effect::Glow {
            threshold: 0.4,
            radius: 12.0,
            intensity: 1.5,
        };
        let chains: [(&[Effect], usize); 6] = [
            (&[blur(14.0), posterize()], 1),
            (&[posterize(), blur(14.0)], 1),
            (&[glow()], 1),
            (&[blur(6.0), smear(), shadow(7.5, -3.25, 20.0)], 2),
            (&[smear(), glow(), posterize(), blur(4.0)], 2),
            (&[posterize(), smear()], 0),
        ];
        for place in [
            at(320.37, 180.5, (1.0, 1.0), 0.0),
            at(300.4, 170.7, (1.3, 0.8), 27.0),
            at(300.4, 170.7, (-1.2, 1.0), 0.0),
        ] {
            for (effects, layers) in chains {
                assert_eq!(
                    same_bytes(&frame, &place, effects),
                    layers,
                    "{effects:?} at {place:?}"
                );
            }
        }
    }
}
