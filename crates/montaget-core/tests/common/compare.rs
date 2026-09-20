//! Frame comparison for spec #168's **seam 2**: SSIM at a stated threshold, over a stated
//! region, never byte equality.
//!
//! # Why SSIM and not a pixel delta
//!
//! The two things this compares are a frame Montaget rasterized and a frame an H.264
//! decoder produced from a 1.28 Mb/s stream. A mean-absolute-difference over that pair
//! measures the encoder's quantizer as much as it measures the renderer, and the number it
//! gives has no natural threshold: 2/255 is either "identical" or "the card moved a pixel"
//! depending on how much of the frame is flat. SSIM is local and structural — it asks
//! whether the same edges are in the same places with the same contrast — which is the
//! question *"does this render match the published video"* actually is.
//!
//! # The window, and why it is separable
//!
//! Wang et al.'s original: an 11×11 Gaussian, σ = 1.5, stride 1, with `C1 = (0.01·L)²` and
//! `C2 = (0.03·L)²` at `L = 255`. Every statistic SSIM needs is a Gaussian-weighted mean
//! of something — `x`, `y`, `x²`, `y²`, `xy` — so the window is applied as two 1-D passes
//! rather than one 2-D one. That is the same numbers (a Gaussian is separable) at 11
//! multiply-adds per pixel per plane instead of 121, which is the difference between a
//! test that runs in a debug build and one nobody waits for.
//!
//! # Scopeing, and what it costs
//!
//! [`ssim`] takes a [`Scope`] and averages only the windows whose centre it admits. The
//! reason it exists is [#186](https://github.com/MBehtemam/Montaget/issues/186): the
//! fixture renders in Open Runde and the published video was typeset in SF Pro Rounded, so
//! **every text-bearing region differs for a reason that is not a defect**. A whole-frame
//! threshold loose enough to absorb a typeface change is loose enough to absorb a moved
//! card, which is the *"completed, looked plausible, was wrong"* failure ADR-0010 records
//! this project having had twice.
//!
//! The cost is real and is not hidden: a masked comparison cannot see what it masks. That
//! is why the text regions are still *measured* and reported, and why the suite contains a
//! test that breaks the render inside the gated region and requires the comparison to
//! fail.

#![allow(dead_code)]

use std::path::Path;

use image::RgbaImage;
use montaget_core::report::ExitCode;
use montaget_core::verbs::frame::{Ask, frame};

/// One greyscale plane, in 0..=255 as `f64`.
pub struct Plane {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f64>,
}

impl Plane {
    /// Rec. 601 luma, which is the plane an H.264 stream already carries — so the
    /// conversion on the reference side is close to an identity and the comparison is not
    /// measuring a colour-space choice.
    pub fn of(picture: &RgbaImage) -> Plane {
        let values = picture
            .pixels()
            .map(|pixel| {
                let [r, g, b, _] = pixel.0;
                0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b)
            })
            .collect();
        Plane {
            width: picture.width() as usize,
            height: picture.height() as usize,
            values,
        }
    }
}

/// A rectangle in the compared frame's own pixels.
///
/// The same word `montaget_render::canvas::Region` and the `frame` answer's `region` both
/// already use for a frame-space rectangle, because it is the same thing measured in a
/// smaller frame. `CONTEXT.md` puts *crop* on Clip's avoid-list for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

impl Region {
    /// The same rectangle stated in the project's true pixels, brought down to the size
    /// the comparison runs at, and grown by `margin` pixels on every side.
    ///
    /// Grown for two reasons that stack. Antialiasing, a stroke and the decoder's
    /// ringing all put a difference a pixel or two outside the ink itself. And [`ssim`]
    /// admits *window centres*, so a centre nearer than [`RADIUS`] to an excluded box
    /// averages that box's pixels into its own statistics — which is asserted here rather
    /// than left as a comment, because it is the failure that makes a mask silently stop
    /// masking.
    pub fn scaled(self, from: i64, to: i64, margin: i64) -> Region {
        assert!(
            margin.unsigned_abs() as usize >= RADIUS,
            "a margin of {margin} is inside the {RADIUS}-pixel SSIM window, so the region \
             it excludes would still reach the windows just outside it"
        );
        let down = |v: i64| v * to / from;
        let x = down(self.x) - margin;
        let y = down(self.y) - margin;
        Region {
            x,
            y,
            width: down(self.x + self.width) + margin - x,
            height: down(self.y + self.height) + margin - y,
        }
    }

    fn holds(&self, x: usize, y: usize) -> bool {
        let (x, y) = (x as i64, y as i64);
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Which pixels a comparison is allowed to look at.
///
/// **Not `Mask`**, which `CONTEXT.md` has already spent on an `effects` vocabulary member
/// — *"a closed shape … that clips an element's rendered pixels"* (ADR-0040) — and which
/// the fixture really carries, on `handle-logo`. One word for two things, in a repository
/// where one of them is a document field, is how a reader takes the wrong meaning away.
///
/// **Layers, last one wins**, rather than a single in-or-out rectangle list. One frame's
/// gated region is genuinely nested: the photograph is excluded, the two cream header
/// panels drawn *over* it are back in, and the chip and handle text drawn over *those* is
/// out again. A flat list cannot say that, and the alternative — hand-splitting each panel
/// into the four rectangles around its text — puts arithmetic into the one place in the
/// suite whose whole job is to be read and believed.
#[derive(Debug, Clone)]
pub struct Scope {
    /// What a pixel no layer names is: `true` for a mask that starts from the whole frame
    /// and cuts regions out, `false` for one that starts from nothing.
    pub default: bool,
    pub layers: Vec<(Region, bool)>,
}

impl Scope {
    /// The whole frame.
    pub fn whole() -> Scope {
        Scope {
            default: true,
            layers: Vec::new(),
        }
    }

    /// Everything except these rectangles.
    pub fn outside(boxes: Vec<Region>) -> Scope {
        Scope::whole().less(boxes)
    }

    /// Only these rectangles.
    pub fn inside(boxes: Vec<Region>) -> Scope {
        Scope {
            default: false,
            layers: Vec::new(),
        }
        .plus(boxes)
    }

    /// Cut these rectangles out of whatever is admitted so far.
    pub fn less(mut self, boxes: Vec<Region>) -> Scope {
        self.layers.extend(boxes.into_iter().map(|b| (b, false)));
        self
    }

    /// Put these rectangles back in.
    pub fn plus(mut self, boxes: Vec<Region>) -> Scope {
        self.layers.extend(boxes.into_iter().map(|b| (b, true)));
        self
    }

    fn admits(&self, x: usize, y: usize) -> bool {
        self.layers
            .iter()
            .rev()
            .find(|(area, _)| area.holds(x, y))
            .map(|(_, admitted)| *admitted)
            .unwrap_or(self.default)
    }

    /// How many of a frame's pixels this mask admits, as a fraction — printed beside every
    /// number the suite reports, because an SSIM over 2% of a frame is not the same claim
    /// as one over 80% of it and the two must not read alike.
    pub fn coverage(&self, width: usize, height: usize) -> f64 {
        let admitted = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .filter(|&(x, y)| self.admits(x, y))
            .count();
        admitted as f64 / (width * height) as f64
    }
}

/// The window: 11 taps of a Gaussian at σ = 1.5, normalised.
const SIGMA: f64 = 1.5;

/// Half the window, in pixels.
///
/// `pub` because it is a **lower bound on any exclusion margin**, and the caller is the
/// one that sets those. [`ssim`] admits or rejects a *window centre*, and every window
/// reaches [`RADIUS`] pixels in each direction — so a centre admitted 3 px outside an
/// excluded box still averages that box's pixels into its own `mu` and covariance, and
/// the exclusion leaks back into the number it was meant to keep out. A margin below this
/// is a mask that does not mask.
pub const RADIUS: usize = 5;

/// `(0.01 · 255)²` and `(0.03 · 255)²` — the stabilising constants, at 8-bit dynamic
/// range.
const C1: f64 = 6.5025;
const C2: f64 = 58.5225;

/// Mean SSIM over the windows the mask admits, and the fraction of the frame that was.
///
/// Panics if the two planes are different sizes: comparing a 1080×1920 render against a
/// 270×480 reference by silently sampling one of them is exactly the kind of quiet
/// accommodation that makes a green suite meaningless. The caller resizes, deliberately.
pub fn ssim(a: &Plane, b: &Plane, mask: &Scope) -> f64 {
    assert_eq!(
        (a.width, a.height),
        (b.width, b.height),
        "SSIM compares two frames of the same size"
    );
    let (w, h) = (a.width, a.height);

    let mu_a = blur(&a.values, w, h);
    let mu_b = blur(&b.values, w, h);
    let aa = blur(&product(&a.values, &a.values), w, h);
    let bb = blur(&product(&b.values, &b.values), w, h);
    let ab = blur(&product(&a.values, &b.values), w, h);

    let mut total = 0.0;
    let mut counted = 0usize;
    for y in 0..h {
        for x in 0..w {
            if !mask.admits(x, y) {
                continue;
            }
            let i = y * w + x;
            let (ma, mb) = (mu_a[i], mu_b[i]);
            let va = aa[i] - ma * ma;
            let vb = bb[i] - mb * mb;
            let cov = ab[i] - ma * mb;
            let numerator = (2.0 * ma * mb + C1) * (2.0 * cov + C2);
            let denominator = (ma * ma + mb * mb + C1) * (va + vb + C2);
            total += numerator / denominator;
            counted += 1;
        }
    }
    assert!(counted > 0, "the mask admitted no pixel at all");
    total / counted as f64
}

fn product(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b).map(|(a, b)| a * b).collect()
}

/// The separable Gaussian, edge-clamped.
///
/// Clamped rather than zero-padded: a zero border invents a black frame around both
/// pictures, and the windows near the edge would then agree with each other about
/// something neither picture contains.
fn blur(values: &[f64], width: usize, height: usize) -> Vec<f64> {
    let taps = taps();
    let mut horizontal = vec![0.0; values.len()];
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for (i, tap) in taps.iter().enumerate() {
                let at = (x + i).saturating_sub(RADIUS).min(width - 1);
                sum += tap * values[y * width + at];
            }
            horizontal[y * width + x] = sum;
        }
    }
    let mut out = vec![0.0; values.len()];
    for y in 0..height {
        for x in 0..width {
            let mut sum = 0.0;
            for (i, tap) in taps.iter().enumerate() {
                let at = (y + i).saturating_sub(RADIUS).min(height - 1);
                sum += tap * horizontal[at * width + x];
            }
            out[y * width + x] = sum;
        }
    }
    out
}

fn taps() -> [f64; RADIUS * 2 + 1] {
    let mut taps = [0.0; RADIUS * 2 + 1];
    let mut total = 0.0;
    for (i, tap) in taps.iter_mut().enumerate() {
        let d = i as f64 - RADIUS as f64;
        *tap = (-(d * d) / (2.0 * SIGMA * SIGMA)).exp();
        total += *tap;
    }
    for tap in &mut taps {
        *tap /= total;
    }
    taps
}

/// One picture at another's size, through Lanczos3.
///
/// The render is rasterized at the project's true 1080×1920 (ADR-0021: `frame` is never
/// proxy-scaled) and the committed reference frames are 270×480, so something has to
/// resample. Down rather than up, because upsampling the reference would invent detail
/// and then ask the render to match it.
pub fn resized(picture: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    if picture.width() == width && picture.height() == height {
        return picture.clone();
    }
    image::imageops::resize(
        picture,
        width,
        height,
        image::imageops::FilterType::Lanczos3,
    )
}

/// One frame of a project, lossless, decoded with a decoder that is not the one that
/// wrote it.
///
/// PNG always: a picture compared through a lossy encoder is a comparison of the
/// encoder's quantizer. `full` picks between ADR-0011's two scales — the falsification
/// suite rasterizes at true pixels and resamples down to the reference's own size, while
/// the golden suite keeps the half-scale answer an agent actually receives.
///
/// Shared by `reference_frames.rs` and `golden_frames.rs` rather than written twice: the
/// two differ in one flag, and two copies of "did the verb answer, and what are its
/// pixels" is two places for that question to get a different answer.
#[track_caller]
pub fn rendered(project: &Path, at: i64, full: bool) -> RgbaImage {
    let answer = frame(
        project,
        &Ask {
            at: Some(at),
            full,
            png: true,
            ..Ask::default()
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer at {at} for {}",
        project.display()
    );
    image::load_from_memory(&answer.image().expect("a picture").bytes)
        .expect("the bytes decode as a picture")
        .to_rgba8()
}
