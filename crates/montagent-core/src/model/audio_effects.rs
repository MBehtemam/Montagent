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
pub enum AudioEffect {}
