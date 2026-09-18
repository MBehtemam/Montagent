//! ADR-0023's one type-generic source-dimensions pipeline: **decode, resolve rotation,
//! apply PAR as an exact rational, round once.**
//!
//! *"Images are the degenerate case — PAR is `1:1` and there is exactly one rotation
//! signal (EXIF) — so ADR-0015's rule is not replaced, it is the general rule with the
//! video-only stages collapsed to no-ops."* Two rules dispatched by source type were
//! rejected 3–0, so there is one function here and no branch on media type.
//!
//! Three properties are load-bearing and each has a test below:
//!
//! - **Rotation is the container's track-level display transform** (an MP4 `tkhd`
//!   matrix), never codec-level SEI/VUI orientation. A portrait phone clip is the
//!   motivating case: every mainstream player reads the container transform to show it
//!   upright, so that is the dimension an author was looking at.
//! - **Rotation resolves before PAR.** *"Applying PAR along the wrong (pre-rotation) axis
//!   stretches the wrong dimension."*
//! - **One rounding point.** The corrected dimensions stay an exact rational all the way
//!   through and round exactly once, by ADR-0013's floor. *"Two implementations that
//!   rounded at different pipeline stages would diverge by a pixel under `validate`'s
//!   strict-equality check; this ADR forbids that by naming the single rounding point."*

use serde::{Deserialize, Serialize};

use super::Rational;

/// The container's track-level display transform, normalised to a quarter turn.
///
/// Anything that is not a multiple of 90° cannot be expressed as an integer dimension
/// pair at all, so it resolves to the nearest quarter turn that can be — and
/// [`Rotation::exact`] records whether that lost anything, so a finding can say so rather
/// than quietly present a rounded frame as a measured one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rotation {
    /// Clockwise degrees, always one of 0, 90, 180, 270.
    pub degrees: u16,
    /// Which signal in the file this came from. ADR-0023 extends ADR-0015's
    /// print-what-you-used requirement: *"`validate` must print both the dimensions it
    /// used **and which rotation source it applied**"* — so the answer travels with the
    /// number rather than being reconstructed by whoever reports it.
    pub source: RotationSource,
    /// Whether the file's own signal was already a quarter turn.
    pub exact: bool,
}

/// Where a rotation came from. Codec-level orientation (SEI / VUI) is deliberately not a
/// member: ADR-0023 says it *"is not consulted for this purpose"*, so it is not a source
/// this type can report having used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationSource {
    /// The file carries no display transform at all.
    None,
    /// The container's track-level display transform — an MP4 `tkhd` matrix, surfaced by
    /// `ffprobe` as Display Matrix side data. The signal ADR-0023 names.
    DisplayMatrix,
    /// The same `tkhd` value as older builds surface it: a `rotate` tag.
    RotateTag,
}

impl RotationSource {
    pub fn as_str(self) -> &'static str {
        match self {
            RotationSource::None => "none",
            RotationSource::DisplayMatrix => "display matrix",
            RotationSource::RotateTag => "rotate tag",
        }
    }
}

impl Rotation {
    pub const NONE: Rotation = Rotation {
        degrees: 0,
        source: RotationSource::None,
        exact: true,
    };

    /// Normalise a rotation as `ffprobe` reports it — signed, counter-clockwise-negative,
    /// and occasionally not a quarter turn at all.
    pub fn from_degrees(degrees: f64, source: RotationSource) -> Rotation {
        // Quarter turns, rounded half away from zero, then wrapped into [0, 4).
        let quarters = (degrees / 90.0).round();
        // A tolerance rather than `f64::EPSILON`, which is an ulp at 1.0 and so would call
        // 270.0000001 inexact for a reason no author could act on. Anything that is not a
        // quarter turn to within a millidegree is one an integer dimension pair cannot
        // express, which is the distinction this flag is for.
        let exact = (quarters * 90.0 - degrees).abs() < 1e-3;
        let quarters = ((quarters as i64 % 4) + 4) % 4;
        Rotation {
            degrees: (quarters * 90) as u16,
            source,
            exact,
        }
    }

    /// Whether this rotation exchanges the width and height axes.
    pub fn swaps_axes(self) -> bool {
        self.degrees == 90 || self.degrees == 270
    }
}

/// What the file says, before any of it is resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decoded {
    pub width: u32,
    pub height: u32,
}

/// The dimensions `fit`'s `cover`/`contain` arithmetic consumes, and everything that went
/// into them.
///
/// ADR-0023 extends ADR-0015's print-what-you-used requirement to video: *"`validate`
/// must print both the dimensions it used and which rotation source it applied"*. The
/// inputs travel with the result so a finding can state them without re-deriving
/// anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDimensions {
    /// What the decoder reported, before rotation or PAR.
    pub decoded: Decoded,
    pub rotation: Rotation,
    /// The pixel aspect ratio applied. `1:1` for images and for square-pixel video.
    pub par: Rational,
    /// The displayed width as an exact rational — the value `cover`/`contain` consume,
    /// so that the one rounding point stays at the final rect.
    pub exact_width: Rational,
    /// The displayed height as an exact rational. PAR never touches this axis.
    pub exact_height: Rational,
    /// The displayed dimensions as integers, floored exactly once (ADR-0013).
    pub width: u32,
    pub height: u32,
}

/// ADR-0023's pipeline, whole. There is no image branch and no video branch.
pub fn resolve(decoded: Decoded, rotation: Rotation, par: Rational) -> SourceDimensions {
    // 1. Rotation first. An image with no EXIF orientation and a video with no container
    //    display transform both arrive here as `Rotation::NONE`, and this is a no-op for
    //    both — the degenerate case ADR-0023 describes, not a skipped stage.
    let (rotated_width, rotated_height) = if rotation.swaps_axes() {
        (decoded.height, decoded.width)
    } else {
        (decoded.width, decoded.height)
    };

    // 2. Then PAR, along the width axis of the *displayed* orientation, as an exact
    //    rational. `par` is `1:1` for every image, which is again a no-op rather than a
    //    branch.
    let exact_width = Rational::new(i64::from(rotated_width) * par.num, par.den);
    let exact_height = Rational::new(i64::from(rotated_height), 1);

    // 3. One rounding point, ADR-0013's floor. Nothing above this line rounded.
    SourceDimensions {
        decoded,
        rotation,
        par,
        exact_width,
        exact_height,
        width: floor(exact_width),
        height: floor(exact_height),
    }
}

/// ADR-0013's floor, in integer arithmetic. *"`math.floor(sw * f)` with `f` computed in
/// floating point is not the same function"* — so this never constructs a float.
fn floor(value: Rational) -> u32 {
    let floored = value.num.div_euclid(value.den);
    floored.clamp(0, i64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoded(width: u32, height: u32) -> Decoded {
        Decoded { width, height }
    }

    #[test]
    fn an_image_is_the_degenerate_case_of_the_same_pipeline() {
        // The fixture's own `images/06.png`, which ADR-0011 cites at 1536×2720.
        let resolved = resolve(decoded(1536, 2720), Rotation::NONE, Rational::ONE);

        assert_eq!((resolved.width, resolved.height), (1536, 2720));
        assert_eq!(resolved.par, Rational::ONE);
        assert!(!resolved.rotation.swaps_axes());
    }

    #[test]
    fn a_portrait_phone_clip_resolves_through_the_container_transform() {
        // ADR-0023's motivating case: 1920×1080 coded, shown upright by every player.
        let resolved = resolve(
            decoded(1920, 1080),
            Rotation::from_degrees(-90.0, RotationSource::DisplayMatrix),
            Rational::ONE,
        );

        assert_eq!(resolved.rotation.degrees, 270);
        assert_eq!((resolved.width, resolved.height), (1080, 1920));
    }

    #[test]
    fn par_is_applied_rather_than_ignored() {
        // ADR-0023 measured PAR divergence at 33.4% on a 4:3 worked case — three orders
        // of magnitude past the 0.038% that got `speed` legislated.
        let resolved = resolve(decoded(720, 576), Rotation::NONE, Rational::new(16, 15));

        assert_eq!(resolved.width, 768, "720 × 16/15");
        assert_eq!(resolved.height, 576, "PAR never touches the height axis");
    }

    #[test]
    fn rotation_resolves_before_par_is_applied() {
        // The whole point of the ordering: applying 16/15 to the pre-rotation width would
        // stretch 720 → 768 and then swap, giving 576×768. The ADR's order gives 614×720.
        let resolved = resolve(
            decoded(720, 576),
            Rotation::from_degrees(90.0, RotationSource::DisplayMatrix),
            Rational::new(16, 15),
        );

        assert_eq!(
            (resolved.width, resolved.height),
            (614, 720),
            "576 × 16/15 = 614.4, floored once at the end"
        );
    }

    #[test]
    fn the_exact_rational_survives_to_the_caller_and_rounds_exactly_once() {
        let resolved = resolve(decoded(720, 576), Rotation::NONE, Rational::new(16, 15));

        // 11520/15 is 768 exactly; the point is that the caller is handed the ratio and
        // not a pre-rounded integer it would have to round again at the final rect.
        assert_eq!(resolved.exact_width, Rational::new(11520, 15));
        assert_eq!(resolved.exact_height, Rational::new(576, 1));
    }

    #[test]
    fn the_single_rounding_point_floors() {
        // 101 × 3/2 = 151.5. Floor, per ADR-0013, and never 152.
        let resolved = resolve(decoded(101, 50), Rotation::NONE, Rational::new(3, 2));
        assert_eq!(resolved.width, 151);
    }

    #[test]
    fn a_rotation_that_is_not_a_quarter_turn_says_so() {
        let rotation = Rotation::from_degrees(17.0, RotationSource::DisplayMatrix);
        assert!(
            !rotation.exact,
            "an integer dimension pair cannot express it, and the report must not pretend otherwise"
        );
        assert_eq!(rotation.degrees, 0);

        assert!(Rotation::from_degrees(180.0, RotationSource::DisplayMatrix).exact);
        assert_eq!(Rotation::from_degrees(-270.0, RotationSource::RotateTag).degrees, 90);
        assert_eq!(Rotation::from_degrees(360.0, RotationSource::DisplayMatrix).degrees, 0);
    }
}
