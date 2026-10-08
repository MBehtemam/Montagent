//! `master`: the one processing stage on the final mix (ADR-0172, ADR-0174).
//!
//! A closed set of two optional, independent, non-animatable keys, named under ADR-0170's
//! suffix rule: `target_lufs`, the integrated loudness the deliverable's mix is brought to by
//! one measured gain, and `ceiling_dbtp`, the true-peak ceiling of the limiter that closes the
//! stage. An empty `master: {}` behaves exactly as an absent one: the mix is left as summed.
//!
//! The two ranges are `validate` errors from the file alone (ADR-0172 §Checks), so each is a
//! type whose deserializer refuses a value outside it and whose schema states the bound, as
//! [`super::Length`] and [`super::Perspective`] do: a schema admitting numbers this binary
//! refuses is the two-artifact divergence.

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

/// The project's master stage. Field order is canonical key order (ADR-0041).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Master {
    /// The integrated loudness (ITU-R BS.1770, gated) the deliverable's mix is brought to, in
    /// LUFS, from −40 to −5. The renderer measures the whole programme's mix and applies one
    /// fixed gain of `target_lufs − measured`; `render` reports both numbers. Not animatable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_lufs: Option<TargetLufs>,
    /// The true-peak ceiling of the limiter that closes the mix, in dBTP, from −12 to 0. It
    /// promises the delivered file: the decoded AAC stays at or under the ceiling plus 1.0 dB
    /// (ADR-0174), not the limiter's own output. Clipped or noise-like material driven far
    /// over it can exceed that by up to +1.8 dB; lower the ceiling by the reported overshoot.
    /// Not animatable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceiling_dbtp: Option<CeilingDbtp>,
}

impl Master {
    /// Whether the stage does anything: `master: {}` is the summed mix, byte for byte.
    pub fn is_inert(&self) -> bool {
        self.target_lufs.is_none() && self.ceiling_dbtp.is_none()
    }
}

/// A bounded `f64` with its range stated once, for the deserializer and the schema alike.
macro_rules! bounded {
    ($name:ident, $key:literal, $unit:literal, $min:expr, $max:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub f64);

        impl $name {
            pub const MIN: f64 = $min;
            pub const MAX: f64 = $max;
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = f64::deserialize(deserializer)?;
                if !(value.is_finite() && ($min..=$max).contains(&value)) {
                    return Err(D::Error::custom(format!(
                        concat!(
                            "`",
                            $key,
                            "` is {}: it is a number of ",
                            $unit,
                            " from {} to {} (ADR-0172)"
                        ),
                        value, $min, $max
                    )));
                }
                Ok($name(value))
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::Schema::try_from(serde_json::json!({
                    "type": "number",
                    "format": "double",
                    "minimum": $min,
                    "maximum": $max,
                    "description": $doc,
                }))
                .expect("an object literal is a schema")
            }
        }
    };
}

bounded!(
    TargetLufs,
    "target_lufs",
    "LUFS",
    -40.0,
    -5.0,
    "An integrated loudness in LUFS (ITU-R BS.1770, gated), from −40 to −5 (ADR-0172)."
);

bounded!(
    CeilingDbtp,
    "ceiling_dbtp",
    "dBTP",
    -12.0,
    0.0,
    "A true-peak ceiling in dBTP, from −12 to 0. It promises the decoded AAC within +1.0 dB \
     (ADR-0172, ADR-0174)."
);
