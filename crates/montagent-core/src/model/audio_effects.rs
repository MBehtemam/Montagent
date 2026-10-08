//! The ordered `audio_effects` list and its closed vocabulary (ADR-0169).
//!
//! An `audio` or `video` element carries `audio_effects`, an ordered list of signal-shaping
//! members discriminated by `name`, the same tagged shape as the visual `effects` list
//! (ADR-0040). The two vocabularies never share a member: `validate` refuses an audio member
//! in `effects` and a visual one here, naming the right list
//! (`crate::checks::audio_effects`).
//!
//! **The union has no members yet.** ADR-0169 fixed the shape and added none; each capability
//! ADR adds one variant (EQ is ADR-0179). An enum with no variants is a real type to serde and
//! to the schema generator: it parses nothing, so any `name` is a schema error, and it
//! publishes as the schema that matches nothing. When the first variant lands the generator
//! publishes a `oneOf` of tagged branches by itself, and nothing here changes shape.
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
    /// Downward compressor (ADR-0180): `acompressor` with RMS detection and a hard knee.
    ///
    /// Every key is required and none defaults. Every level is RMS dBFS, so a full-scale
    /// sine reads -3.01. The ranges are checked by `validate` (`E-DYNAMICS-RANGE`). A
    /// keyframe list on any key is a schema error: these are plain numbers, not animatable
    /// properties (#842).
    Compressor {
        /// RMS dBFS above which the signal is reduced. -60..0.
        threshold_db: f64,
        /// n:1 above the threshold. 1..20.
        ratio: f64,
        /// Envelope attack in ms. 0.1..2000. Not a 63% time.
        attack_ms: f64,
        /// Envelope release in ms. 1..9000.
        release_ms: f64,
        /// Make-up gain in dB. 0..24.
        makeup_db: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
    /// Look-ahead limiter (ADR-0180): `alimiter` with a fixed 5 ms look-ahead.
    ///
    /// Every key is required and none defaults. `ceiling_db` is a sample peak; the delivered
    /// file's true-peak ceiling is the master's.
    Limiter {
        /// Sample-peak ceiling in dBFS. -24..0 (`alimiter`'s own floor).
        ceiling_db: f64,
        /// Release in ms. 1..1000.
        release_ms: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enabled: Option<bool>,
    },
}
