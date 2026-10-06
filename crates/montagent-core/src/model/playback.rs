//! The three fields that say how a time-based element *plays* — `speed`, `overrun` and
//! `volume` — and the bounds each of them carries.
//!
//! All three bounds are stated in the ADR series and in `CONTEXT.md` as **schema errors**,
//! which in this codebase is a claim about where they are enforced, not about how badly
//! they are meant: [`crate::checks::schema`] reports a schema fact by parsing the document
//! through [`crate::model`], so a bound that is not in a `Deserialize` impl is a bound that
//! `validate` passes and the renderer meets. They are here for the same reason ADR-0038's
//! positional `ease` rule and ADR-0012's bezier `x` bound are in
//! [`crate::model::keyframe`] rather than in a check.
//!
//! The messages are written the way those two are: naming the value, the rule, and the
//! reason — because the reader is an agent that wrote the value on purpose, and *"invalid
//! value"* tells it only that it must guess again.

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

/// A playback-rate multiplier, strictly greater than zero (ADR-0020).
///
/// Named for the field rather than for the multiplier it holds: `CONTEXT.md`'s **Speed**
/// entry avoids *rate*, *stretch factor* and *tempo* as names for this concept, and the
/// type is published into the schema as a `$def` an agent reads.
///
/// `0` is refused because it leaves ADR-0020's own invariant — `end - start` equals
/// `source range / speed` — undefined, and because no length of timeline can hold a source
/// that advances by nothing. Negative is refused because reverse playback *"is not
/// `speed`'s job"*: a sign bit smuggled onto a magnitude would decide it silently, and
/// decide it badly. ADR-0157 has since settled it for video as a falling `source_time`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Speed(pub f64);

impl<'de> Deserialize<'de> for Speed {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // `<= 0.0`, not `!(> 0.0)`: JSON has no `NaN` literal, so there is no third case
        // to spell — and the negated comparison would only read as if there were.
        let speed = f64::deserialize(deserializer)?;
        if speed <= 0.0 {
            return Err(D::Error::custom(format!(
                "`speed` is {speed}: a speed is strictly greater than zero — `0` plays \
                 nothing for any length of time, and reverse playback is a falling \
                 `source_time` curve on a video rather than a negative speed (ADR-0020, \
                 ADR-0157)"
            )));
        }
        Ok(Speed(speed))
    }
}

impl JsonSchema for Speed {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Speed".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        number_schema(
            "A playback-rate multiplier, strictly greater than zero: `0.645` plays the \
             source slower. `0` and negative values are schema errors — reverse, like a \
             ramp or a freeze, is a `source_time` curve on a video, never a sign bit on \
             this (ADR-0020, ADR-0157).",
            "exclusiveMinimum",
        )
    }
}

/// Which moment of a `video`'s file is on screen, in integer **source** milliseconds
/// (ADR-0157): the value of `source_time`, in a literal and in every keyframe record.
///
/// Negative is refused: there is no moment of a file before its first. A fractional value
/// is refused by the integer it is read as, as every millisecond in the format is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SourceTime(pub i64);

impl<'de> Deserialize<'de> for SourceTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ms = i64::deserialize(deserializer)?;
        if ms < 0 {
            return Err(D::Error::custom(format!(
                "`source_time` is {ms}: a source time is a moment of the file in integer \
                 milliseconds from `0`, and there is none before the first (ADR-0157)"
            )));
        }
        Ok(SourceTime(ms))
    }
}

impl JsonSchema for SourceTime {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SourceTime".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        Schema::try_from(serde_json::json!({
            "type": "integer",
            "format": "int64",
            "description": "A moment of the video's file in integer source milliseconds: \
                            which frame is on screen. A literal is a freeze frame; a \
                            keyframe list is a time-remap curve whose slope is the rate, \
                            flat to freeze and falling to reverse. Negative is a schema \
                            error (ADR-0157).",
            "minimum": 0,
        }))
        .expect("an object literal is a schema")
    }
}

/// A playback level, as a linear multiplier: `0` silent, `1` the source's own level, above
/// `1` amplified (ADR-0055).
///
/// Negative is the only value refused. It is not a quieter sound — it is the waveform
/// inverted at full level, which nobody has ever meant by writing it, and which sums
/// against a sibling element as cancellation rather than as a mix. The upper end is
/// deliberately open: *"`>1` is permitted rather than capped; clipping past that point is
/// the renderer's documented behaviour, not a schema-enforced ceiling"*.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Volume(pub f64);

impl<'de> Deserialize<'de> for Volume {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let level = f64::deserialize(deserializer)?;
        if level < 0.0 {
            return Err(D::Error::custom(format!(
                "`volume` is {level}: a level is a linear multiplier from `0` (silent) \
                 upwards, and a negative one inverts the waveform at full level rather \
                 than quietening it. Silence is `0` (ADR-0055)"
            )));
        }
        Ok(Volume(level))
    }
}

impl JsonSchema for Volume {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Volume".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        number_schema(
            "A linear level multiplier: `0` is silent, `1` (the default) is the source's \
             own level, and values above `1` amplify — clipping past that is the \
             renderer's documented behaviour rather than a ceiling. Negative is a schema \
             error (ADR-0055).",
            "minimum",
        )
    }
}

/// `{"type": "number", "format": "double"}` with one bound and a description — the same
/// shape `f64`'s own derived schema has, so a bounded number stays recognisable as a
/// number to every reader of the published schema.
fn number_schema(description: &str, bound: &str) -> Schema {
    Schema::try_from(serde_json::json!({
        "type": "number",
        "format": "double",
        "description": description,
        bound: 0.0,
    }))
    .expect("an object literal is a schema")
}

/// What an `audio` element does past the end of its (possibly speed-adjusted) source.
///
/// A `video` element's `overrun` minus `hold`. ADR-0020: there is no non-arbitrary
/// meaning for holding the last *sample* — a frame held is the picture that was already
/// there, and a sample held is a DC offset or a click — and "then silence" is already
/// free, and already legible on the timeline, as a shorter element and a gap.
///
/// Its own type rather than a check on the shared one because it is a different field:
/// an `audio` element publishes one legal value here and the schema says so, which is what
/// an agent reading the schema for the answer gets to read.
// The shared one is `super::Overrun`, named here rather than in the doc comment above:
// this type's doc comment is what `schemars` publishes as its schema `description`, and a
// Rust path is not something a reader of the schema can follow. (`Speed` and `Volume`
// escape the same trap differently — their descriptions are written out in
// `number_schema`, because a hand-written `JsonSchema` impl publishes no doc comment.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AudioOverrun {
    /// The source restarts from `source_start` with a hard cut — no crossfade, which would
    /// need an unstated duration and curve.
    Loop,
}

impl<'de> Deserialize<'de> for AudioOverrun {
    /// Hand-written for the message alone. `serde`'s own would be *"unknown variant
    /// `hold`, expected `loop`"*, which is the bare schema-mismatch dump ADR-0042 asked
    /// implementations not to produce — and `hold` is the one wrong value an author
    /// actually writes here, having just written it correctly on a video.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match String::deserialize(deserializer)?.as_str() {
            "loop" => Ok(AudioOverrun::Loop),
            "hold" => Err(D::Error::custom(
                "`overrun` is \"hold\" on an audio element: `hold` freezes a video's last \
                 frame, and there is no non-arbitrary meaning for holding the last sample. \
                 On audio the field's one value is \"loop\"; a silenced tail is a shorter \
                 element and a gap (ADR-0020)",
            )),
            other => Err(D::Error::custom(format!(
                "`overrun` is \"{other}\": an audio element's one value is \"loop\", and \
                 the field is absent where it is not needed — there is no \"none\" \
                 (ADR-0020)"
            ))),
        }
    }
}
