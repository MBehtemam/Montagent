//! A path's stroke join, miter limit and cap (ADR-0158 §1–§3): three static fields, on
//! `path` only.
//!
//! The relations between them — a miter needs its limit, a cap needs an end to draw at, and
//! none of them shapes a stroke that is not there — are `validate`'s
//! (`crate::checks::stroke`), not the schema's. What the schema says is each field's type,
//! vocabulary and range, that each is static, and that a `rect`, `ellipse` or `text` does
//! not take them ([`off_path`]).

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use super::keyframe::is_keyframe_list;

/// The fields ADR-0158 §1 gives to `path` alone.
pub const PATH_ONLY: [&str; 3] = ["stroke_join", "stroke_miter_limit", "stroke_cap"];

/// The one schema error for a path-only stroke field on another element type, naming the
/// first such field `rest` carries — rather than the unknown-key error, which would say the
/// key may belong to a newer format and must not be deleted (ADR-0158 §1).
pub fn off_path(kind: Option<&str>, rest: &Map<String, Value>) -> Option<String> {
    if !matches!(kind, Some("rect" | "ellipse" | "text")) {
        return None;
    }
    let key = PATH_ONLY.iter().find(|key| rest.contains_key(**key))?;
    let kind = kind.unwrap_or_default();
    let why = if kind == "text" {
        "a text stroke falls outside the glyph contour with round joins, and takes no join, \
         limit or cap"
    } else {
        "a `rect`'s corners are square or `radius`-round and an `ellipse` has none, so their \
         join and cap stay as drawn"
    };
    Some(format!(
        "`{key}` belongs to `path`, not to a `{kind}`: {why}. Drop it, or draw the outline as \
         a `path` (ADR-0158)"
    ))
}

/// Refuse a keyframe list on a static stroke field, with the field's name.
fn refuse_keyed<E: serde::de::Error>(key: &str, value: &Value) -> Result<(), E> {
    if is_keyframe_list(value) {
        return Err(E::custom(format!(
            "`{key}` is static: a keyframe list is not a value of it (ADR-0158)"
        )));
    }
    Ok(())
}

/// `stroke_join`: how two segments of a path meet. `"round"` when omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase", remote = "Self")]
pub enum StrokeJoin {
    Round,
    Bevel,
    // Requires `stroke_miter_limit`; a corner whose tip would reach past it is beveled.
    Miter,
}

/// `stroke_cap`: how an open path's two ends are drawn. `"butt"` when omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase", remote = "Self")]
pub enum StrokeCap {
    Butt,
    Round,
    // Reaches √2 times half the width at its corners, which widens the box's inset.
    Square,
}

// The two halves of `remote = "Self"`: the derived reading, after the static check.
macro_rules! static_enum {
    ($type:ident, $key:literal) => {
        impl Serialize for $type {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                $type::serialize(self, serializer)
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = Value::deserialize(deserializer)?;
                refuse_keyed::<D::Error>($key, &value)?;
                $type::deserialize(value).map_err(D::Error::custom)
            }
        }
    };
}

static_enum!(StrokeJoin, "stroke_join");
static_enum!(StrokeCap, "stroke_cap");

/// `stroke_miter_limit`: the greatest miter tip distance from its vertex, in multiples of
/// half the stroke width. A static integer from 1 to 10, required with `"miter"` and refused
/// with any other join, because the box's inset is computed from it (ADR-0158 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct MiterLimit(pub i64);

impl<'de> Deserialize<'de> for MiterLimit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        refuse_keyed::<D::Error>("stroke_miter_limit", &value)?;
        match value.as_i64() {
            Some(limit) if (1..=10).contains(&limit) => Ok(MiterLimit(limit)),
            _ => Err(D::Error::custom(format!(
                "`stroke_miter_limit` is {value}: it is a static integer from 1 to 10, the \
                 miter tip's greatest distance from its vertex in multiples of half the stroke \
                 width (ADR-0158)"
            ))),
        }
    }
}

impl JsonSchema for MiterLimit {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "MiterLimit".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "integer",
            "format": "int64",
            "minimum": 1,
            "maximum": 10,
            "description": "The greatest miter tip distance from its vertex, in multiples of \
                            half the stroke width; a sharper corner is beveled. Static. \
                            Required with `\"stroke_join\": \"miter\"`, and refused with any \
                            other join (ADR-0158).",
        }))
        .expect("an object literal is a schema")
    }
}
