//! The `bounds` hint on a `blur` or `shadow` layer
//! ([#652](https://github.com/MBehtemam/Montagent/issues/652)): the one hint that keeps every
//! byte of the unbounded layer's picture.
//!
//! An unhinted filter layer covers the whole frame plus the blur's reach, so a small glowing
//! title blurs a frame's worth of transparent pixels. A hint bounds Skia's layer outright (not
//! an element `clip`, which is frame-space and applied outside the effects), and
//! [#649](https://github.com/MBehtemam/Montagent/issues/649)'s research
//! (`docs/research/filter-bound/FINDINGS.md` on `research/649-filter-bound`) found which
//! hints keep the bytes:
//!
//! - **The layer's top-left corner must not move.** Skia turns that corner into a translation
//!   in the element's matrix, so moving it changes the float coordinates the element is
//!   rasterized at and flips antialiased coverage on a few edge pixels. No four-sided hint is
//!   byte-identical, however wide. So the hint's left and top are pinned beyond the
//!   unbounded layer's, **in layer space**: under a flip the element-space edge that lands
//!   on the layer's left is the right one.
//! - **The right and bottom are Skia's own bounds of what the element draws**, from an
//!   `SkPicture` recorded under the real matrix with a bounding-box hierarchy, through each
//!   filter's `computeFastBounds`. Not the element's box: text overflows its box, and a
//!   box-built bound fails once σ is too small to hide that.
//!
//! Every pixel the hint drops is then exactly zero in the unbounded layer, because Skia's 8888
//! blur is a decal convolution whose window reaches less than `ceil(3σ)`. That holds only
//! under preconditions, and [`Plan::new`] gives no hint at all where one fails:
//!
//! - every effect is a `blur`, a `shadow`, a `mask`, or one of ADR-0156's `grain`, `glow`,
//!   `posterize` and `directional_blur` — ADR-0049's colour filters and `chroma` are outside
//!   the derivation, and an element with one keeps today's unbounded layers throughout. A
//!   `glow` is hinted as a blur is, its σ meeting the cap below. A `grain`'s own layer is
//!   unhinted, as a `mask`'s is, and passes the bound on. `posterize` keeps the bound (it
//!   maps transparent black to transparent black) and a `directional_blur` reaches its crop;
//!   both their own layers stay unhinted and pass the bound on, because a hint on them moved
//!   a level on a few pixels under a rotation or a flip
//!   ([#722](https://github.com/MBehtemam/Montagent/issues/722));
//! - no perspective;
//! - layer-space σ ≤ [`MAX_LAYER_SIGMA`] on both axes, above which Skia downsamples the
//!   layer and the resampling depends on its size. A feathered `mask`'s σ counts, and so
//!   does its reach in the next bound (#698): its soft edge is a blur in a layer of its own;
//! - no layer large enough for Skia's `maxLayerDim` to rescale it, with or without the hint
//!   ([`MAX_OUTSET`]).

use std::cell::Cell;

use skia_safe::{Canvas as SkCanvas, ImageFilter, Matrix, PictureRecorder, Rect};

use super::{Effect, sigma};

/// The largest layer-space σ Skia's raster blur takes without rescaling the layer first
/// (`Raster8888BlurAlgorithm`'s `maxSigma`). Measured: σ 134 is byte-identical with the
/// hint, σ 140–350 are not.
const MAX_LAYER_SIGMA: f32 = 135.0;

/// The most, in layer pixels per axis, the chain of filter layers may reach past the device
/// clip.
///
/// Skia rescales a layer wider or taller than `max(2 × its target, 2048)`. A layer is its
/// target outset by at most this on each side, and `w + 2 × 512 ≤ max(2w, 2048)` for every
/// `w`. So under this bound no layer is rescaled, with the hint shrinking its target or
/// without — and a rescale is the one thing that would make the hint's size matter.
const MAX_OUTSET: f32 = 512.0;

/// How far past the unbounded layer's top-left the pinned edges sit, in layer pixels: far
/// enough that no float rounding brings them back.
const PIN_MARGIN: f32 = 1024.0;

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(true) };
    /// How many layers this thread has hinted, so a test that finds the same bytes both
    /// ways can also say the hint was there to find them.
    #[cfg(test)]
    pub(super) static HINTED: Cell<usize> = const { Cell::new(0) };
}

/// Paint this thread's filter layers with (`true`, the default) or without the hint.
///
/// Not a user setting: the hint has no visible effect, and this exists so that a test can
/// paint one project both ways and require the same bytes. Per thread, so a test that turns
/// it off cannot reach a painter on another test's thread.
#[doc(hidden)]
pub fn set_enabled(on: bool) {
    ENABLED.with(|enabled| enabled.set(on));
}

/// Whether this thread paints filter layers with the hint, so a render that paints on more
/// than one thread can hand each painter the switch of the thread that asked (#653).
#[doc(hidden)]
pub fn enabled() -> bool {
    ENABLED.with(Cell::get)
}

/// One hint per effect, aligned with `effects` and `filters`, in element space. `None` is
/// an unbounded layer, exactly as an unhinted one always was.
pub(super) fn hints(
    canvas: &SkCanvas,
    effects: &[Effect],
    filters: &[Option<ImageFilter>],
    draw: &dyn Fn(&SkCanvas),
) -> Vec<Option<Rect>> {
    let mut hints = vec![None; effects.len()];
    let Some(plan) = Plan::new(canvas, effects) else {
        return hints;
    };
    let Some(mut content) = plan.recorded(draw) else {
        return hints;
    };
    // Innermost first: effect `i`'s output is what effect `i + 1`'s layer holds.
    for ((effect, filter), hint) in effects.iter().zip(filters).zip(&mut hints) {
        // A `mask` only erases, and a `grain` and a `posterize` keep transparent black
        // transparent, so what each holds bounds what it passes on (ADR-0156 §4). Any member
        // Skia declined to build is a plain layer, which changes nothing.
        let Some(filter) = filter else {
            continue;
        };
        match effect {
            // A `glow`'s reach is its blur's, so its layer takes the hint as a blur's does.
            Effect::Blur { .. } | Effect::Shadow { .. } | Effect::Glow { .. } => {
                let output = filter.compute_fast_bounds(content);
                *hint = plan.pinned(output);
                #[cfg(test)]
                HINTED.with(|hinted| hinted.set(hinted.get() + usize::from(hint.is_some())));
                content = output;
            }
            // Its own layer stays unhinted and passes the bound on (#722): a hint on it
            // moved a level on a few pixels under a flip. Its filter is already cropped to
            // its reach, so Skia's bounds of it are that crop.
            Effect::DirectionalBlur { .. } => content = filter.compute_fast_bounds(content),
            // Unhinted for the same reason under a rotation, and its output is its input's.
            _ => {}
        }
    }
    hints
}

/// Skia's bounds of what `draw` paints, in element space: the element recorded once more
/// under the canvas's matrix, so every matrix-dependent choice the drawing makes is the same
/// one. Only commands are recorded; nothing is rasterized. `None` for a drawing with nothing
/// in it, or a matrix with no inverse.
pub(super) fn drawn(canvas: &SkCanvas, draw: &dyn Fn(&SkCanvas)) -> Option<Rect> {
    let to_element = canvas.local_to_device_as_3x3().invert()?;
    let mut recorder = PictureRecorder::new();
    let everywhere = Rect::new(-1.0e7, -1.0e7, 1.0e7, 1.0e7);
    let recording = recorder.begin_recording(everywhere, true);
    recording.set_matrix(&canvas.local_to_device());
    draw(recording);
    let device = recorder.finish_recording_as_picture(None)?.cull_rect();
    if device.is_empty() {
        return None;
    }
    Some(to_element.map_rect(device).0)
}

/// Everything about the element's matrix that the hint is built from, or the reason there is
/// no hint.
struct Plan<'a> {
    canvas: &'a SkCanvas,
    /// The pinned corner, in element space, and whether each is the low edge there.
    pin: (f32, f32),
    low: (bool, bool),
}

impl<'a> Plan<'a> {
    fn new(canvas: &'a SkCanvas, effects: &[Effect]) -> Option<Plan<'a>> {
        if !ENABLED.with(Cell::get) {
            return None;
        }
        let blurs = || {
            effects.iter().filter_map(|effect| match *effect {
                Effect::Blur { radius } | Effect::Glow { radius, .. } => Some((radius, 0.0, 0.0)),
                Effect::Shadow { dx, dy, radius, .. } => Some((radius, dx, dy)),
                _ => None,
            })
        };
        // ADR-0156's members are inside the derivation (#722 measured all three, hint on and
        // off): `glow` is a blur, `posterize` keeps the bound, and `directional_blur` reaches
        // its cropped reach. ADR-0049's four colour filters and `chroma` stay outside it.
        if blurs().next().is_none()
            || !effects.iter().all(|effect| {
                matches!(
                    effect,
                    Effect::Blur { .. }
                        | Effect::Shadow { .. }
                        | Effect::Mask { .. }
                        | Effect::Grain { .. }
                        | Effect::Glow { .. }
                        | Effect::Posterize { .. }
                        | Effect::DirectionalBlur { .. }
                )
            })
        {
            return None;
        }
        let ctm = canvas.local_to_device_as_3x3();
        if ctm.has_perspective() {
            return None;
        }
        let to_element = ctm.invert()?;
        // Skia's own split (`Mapping::decomposeCTM`, for a blur's scale+translate
        // capability): a scale+translate matrix is the layer's matrix whole, so a negative
        // scale flips layer space; anything else leaves the layer a positive scale by the
        // matrix's column lengths (`SkMatrix::decomposeScale`) and the rest to the composite.
        let layer = if ctm.is_scale_translate() {
            ctm
        } else {
            let x = ctm.scale_x().hypot(ctm.skew_y());
            let y = ctm.skew_x().hypot(ctm.scale_y());
            // `decomposeScale` gives up on a nearly-zero axis, and so does the hint.
            if x <= 1.0 / 4096.0 || y <= 1.0 / 4096.0 {
                return None;
            }
            Matrix::scale((x, y))
        };
        let scale = (layer.scale_x().abs(), layer.scale_y().abs());

        // The chain's reach past the device clip, with the layer's one pixel of padding and
        // one more for rounding on each layer. A feathered mask's blur of its own coverage
        // counts as a blur here (#698): its σ meets the same cap, and its reach joins the sum.
        let feathers = effects.iter().filter_map(|effect| match *effect {
            Effect::Mask { feather, .. } if feather > 0.0 => Some((feather, 0.0, 0.0)),
            _ => None,
        });
        let mut outset = (0.0f32, 0.0f32);
        for (radius, dx, dy) in blurs().chain(feathers) {
            let sigma = (sigma(radius) * scale.0, sigma(radius) * scale.1);
            if !(sigma.0 <= MAX_LAYER_SIGMA && sigma.1 <= MAX_LAYER_SIGMA) {
                return None;
            }
            outset.0 += (3.0 * sigma.0).ceil() + (dx as f32 * scale.0).abs() + 2.0;
            outset.1 += (3.0 * sigma.1).ceil() + (dy as f32 * scale.1).abs() + 2.0;
        }
        // A directional blur reaches its crop: its declared reach, rounded up, and a pixel.
        for effect in effects {
            if let Effect::DirectionalBlur { angle, length } = *effect {
                let (x, y) = super::named::directional_reach(angle, length);
                outset.0 += (x * scale.0).ceil() + 2.0;
                outset.1 += (y * scale.1).ceil() + 2.0;
            }
        }
        if !(outset.0 <= MAX_OUTSET && outset.1 <= MAX_OUTSET) {
            return None;
        }

        // The device clip in layer space. For scale+translate it is the clip itself; under a
        // rotation it is the clip's rotated bounding box, which can outgrow `maxLayerDim`
        // where the clip does not, so that is checked rather than argued.
        let clip = Rect::from(canvas.device_clip_bounds()?);
        let target = clip.width().max(clip.height());
        let ceiling = (2.0 * target).max(2048.0);
        let (clip_in_layer, _) = Matrix::concat(&layer, &to_element).map_rect(clip);
        if !(clip_in_layer.width() + 2.0 * outset.0 <= ceiling
            && clip_in_layer.height() + 2.0 * outset.1 <= ceiling)
        {
            return None;
        }

        // A layer-space corner beyond everything the unbounded layer reaches to the left and
        // above, taken back to element space one axis at a time: `layer` is scale+translate.
        let corner = (
            clip_in_layer.left - outset.0 - PIN_MARGIN,
            clip_in_layer.top - outset.1 - PIN_MARGIN,
        );
        let pin = (
            (corner.0 - layer.translate_x()) / layer.scale_x(),
            (corner.1 - layer.translate_y()) / layer.scale_y(),
        );
        if !(pin.0.is_finite() && pin.1.is_finite()) {
            return None;
        }
        Some(Plan {
            canvas,
            pin,
            low: (layer.scale_x() > 0.0, layer.scale_y() > 0.0),
        })
    }

    /// Skia's bounds of what `draw` paints, in element space: the element recorded once more
    /// under the same matrix, so every matrix-dependent choice the drawing makes is the same
    /// one. Only commands are recorded; nothing is rasterized. `None` for a drawing with
    /// nothing in it.
    fn recorded(&self, draw: &dyn Fn(&SkCanvas)) -> Option<Rect> {
        drawn(self.canvas, draw)
    }

    /// `output` with its layer-space left and top moved out to the pin. `None` if that leaves
    /// no rect, which only an element drawn wholly beyond the pin could do.
    fn pinned(&self, output: Rect) -> Option<Rect> {
        let mut hint = output;
        if self.low.0 {
            hint.left = self.pin.0;
        } else {
            hint.right = self.pin.0;
        }
        if self.low.1 {
            hint.top = self.pin.1;
        } else {
            hint.bottom = self.pin.1;
        }
        (hint.left < hint.right && hint.top < hint.bottom).then_some(hint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skia_safe::{ImageInfo, Paint, surfaces};

    /// The hints for one 100×40 rect drawn at `(200, 100)` on a 640×360 raster, through
    /// `effects`, under `scale`.
    fn hints_for(effects: &[Effect], scale: (f32, f32)) -> Vec<Option<Rect>> {
        let mut surface =
            surfaces::raster(&ImageInfo::new_n32_premul((640, 360), None), None, None)
                .expect("a surface");
        let canvas = surface.canvas();
        canvas.translate((200.0, 100.0));
        canvas.scale(scale);
        let filters: Vec<_> = effects.iter().map(|e| e.filter()).collect();
        hints(canvas, effects, &filters, &|canvas| {
            canvas.draw_rect(Rect::from_xywh(0.0, 0.0, 100.0, 40.0), &Paint::default());
        })
    }

    #[test]
    fn the_right_and_bottom_are_what_was_drawn_through_the_filter_and_the_top_left_is_far() {
        // σ 5: Skia's fast bound is 3σ = 15 past the rect on every side.
        let [Some(hint)] = hints_for(&[Effect::Blur { radius: 10.0 }], (1.0, 1.0))[..] else {
            panic!("one hinted layer");
        };
        assert_eq!((hint.right, hint.bottom), (115.0, 55.0));
        // Past the unbounded layer: the frame's top-left less 3σ, its padding and the
        // margin, in element space.
        assert!(hint.left < -200.0 - 15.0 - PIN_MARGIN + 1.0, "{hint:?}");
        assert!(hint.top < -100.0 - 15.0 - PIN_MARGIN + 1.0, "{hint:?}");
    }

    #[test]
    fn under_a_flip_the_pinned_edges_are_the_element_s_right_and_bottom() {
        let [Some(hint)] = hints_for(&[Effect::Blur { radius: 10.0 }], (-1.0, -1.0))[..] else {
            panic!("one hinted layer");
        };
        // The element's left and top land on the layer's right and bottom, so they keep
        // the drawn bound; the right and bottom go out past the layer's left and top.
        assert_eq!((hint.left, hint.top), (-15.0, -15.0));
        assert!(hint.right > 200.0 + PIN_MARGIN && hint.bottom > 100.0 + PIN_MARGIN);
    }

    fn feathered(feather: f64) -> Effect {
        Effect::Mask {
            shape: crate::canvas::MaskShape::Ellipse,
            rect: None,
            radius: 0.0,
            invert: false,
            feather,
        }
    }

    #[test]
    fn a_feather_s_sigma_and_reach_count_in_the_preconditions() {
        // A feather is a blur of the mask's coverage, so a σ past the cap withholds the
        // hint from the whole chain as a blur's would (#698)...
        assert_eq!(
            hints_for(
                &[feathered(300.0), Effect::Blur { radius: 10.0 }],
                (1.0, 1.0)
            ),
            [None, None]
        );
        // ...and so does a reach that, summed with the blur's, passes the outset bound:
        // σ 60 reaches 180 + 2, three of them 546, past 512.
        assert_eq!(
            hints_for(
                &[
                    feathered(120.0),
                    feathered(120.0),
                    Effect::Blur { radius: 120.0 }
                ],
                (1.0, 1.0)
            ),
            [None, None, None]
        );
        // A modest feather leaves the blur hinted.
        assert!(
            hints_for(
                &[feathered(30.0), Effect::Blur { radius: 10.0 }],
                (1.0, 1.0)
            )[1]
            .is_some()
        );
    }

    #[test]
    fn a_grain_keeps_the_bound_and_passes_it_on_unhinted_itself() {
        let grain = Effect::Grain {
            seed: 7,
            amount: 0.3,
            size: 2,
            mono: true,
            frame: 0,
        };
        let hints = hints_for(&[grain, Effect::Blur { radius: 10.0 }], (1.0, 1.0));
        let [None, Some(blur)] = hints[..] else {
            panic!("the grain's layer plain, the blur's hinted: {hints:?}");
        };
        assert_eq!((blur.right, blur.bottom), (115.0, 55.0));
    }

    #[test]
    fn a_shadow_chain_carries_each_layer_s_output_to_the_next() {
        let effects = [
            Effect::Blur { radius: 10.0 },
            Effect::Shadow {
                dx: 20.0,
                dy: 0.0,
                radius: 4.0,
                colour: crate::canvas::Rgba([0, 0, 0, 0xFF]),
                opacity: 1.0,
            },
        ];
        let hints = hints_for(&effects, (1.0, 1.0));
        let (Some(inner), Some(outer)) = (hints[0], hints[1]) else {
            panic!("both hinted: {hints:?}");
        };
        assert_eq!(inner.right, 115.0);
        // The blur's output, shifted 20 right and outset by the shadow's 3σ = 6.
        assert_eq!((outer.right, outer.bottom), (141.0, 61.0));
    }
}
