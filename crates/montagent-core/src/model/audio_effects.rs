//! The ordered `audio_effects` list and its closed vocabulary (ADR-0169).
//!
//! An `audio` or `video` element carries `audio_effects`, an ordered list of signal-shaping
//! members discriminated by `name`, the same tagged shape as the visual `effects` list
//! (ADR-0040). The two vocabularies never share a member: `validate` refuses an audio member
//! in `effects` and a visual one here, naming the right list
//! (`crate::checks::audio_effects`).
//!
//! ADR-0169 fixed the shape and added no member; each capability ADR adds variants. EQ's four
//! (ADR-0179) are the first: `highpass`, `lowpass`, `shelf` and `bell`. The generator
//! publishes a `oneOf` of tagged branches by itself.
//!
//! **Ranges are `validate`'s, not the type's.** Every EQ number is a plain `f64`, so a value
//! outside its range, or a slope outside 12|24|48, parses and is reported as `E-EQ-RANGE`
//! with the nearest bound as the repair (ADR-0179 §3) instead of as a schema error with no
//! repair. A keyframe list is not a number, so it is a schema error (#842).
//!
//! # What every future member must declare
//!
//! - `enabled: Option<bool>`, declared `#[serde(default, skip_serializing_if =
//!   "Option::is_none")]` and written last in the member. `"enabled": false` bypasses the
//!   member (it is still validated, and does not render); `fmt` drops `"enabled": true`.
//!   `name` and `enabled` are reserved, so no parameter may use either.
//! - Whether it is singular: add its `name` to
//!   `crate::checks::audio_effects::SINGULAR`.
//! - How it lowers to ffmpeg: a case in `crate::verbs::render`'s `lower_audio_member`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One member of the closed audio-effect vocabulary, discriminated by `name`.
///
/// Two members of the same name are ordinary unless the member's own ADR declares it singular
/// (ADR-0169).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "name", rename_all = "snake_case", deny_unknown_fields)]
pub enum AudioEffect {
    /// A high-pass filter (ADR-0179). Every key is required; there is no default.
    ///
    /// `frequency_hz` 20..20000 is the cutoff, where the response reads -3.01 dB at every
    /// slope. `slope_db_per_oct` is 12, 24 or 48: one, two or four Butterworth sections.
    Highpass {
        frequency_hz: f64,
        slope_db_per_oct: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
    /// A low-pass filter (ADR-0179); the same keys and ranges as `highpass`.
    Lowpass {
        frequency_hz: f64,
        slope_db_per_oct: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
    /// A shelf (ADR-0179). `frequency_hz` 20..20000 is the corner, where the shelf has half
    /// its gain; `gain_db` is -24..24. The shelf's Q is fixed at 0.707107.
    Shelf {
        side: ShelfSide,
        frequency_hz: f64,
        gain_db: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
    /// A bell (peaking) band (ADR-0179). `frequency_hz` 20..20000 is the centre, `gain_db`
    /// -24..24 the height there, `q` 0.1..10 the narrowness.
    Bell {
        frequency_hz: f64,
        gain_db: f64,
        q: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
}

/// Which end of the spectrum a `shelf` lifts or cuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShelfSide {
    /// Everything below the corner (`lowshelf`).
    Low,
    /// Everything above the corner (`highshelf`).
    High,
}
