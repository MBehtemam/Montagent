//! A path's stroke join, miter limit and cap (ADR-0158 §1–§3), three static fields on `path`
//! only, and the dash pattern every shape takes (§5): a static `stroke_dash` list and an
//! animatable `stroke_dash_offset`, on `path`, `rect` and `ellipse`.
//!
//! The relations between them — a miter needs its limit, a cap needs an end to draw at, a
//! pattern needs an even number of entries and a total above 0, and none of them shapes a
//! stroke that is not there — are `validate`'s (`crate::checks::stroke`), not the schema's.
//! What the schema says is each field's type, vocabulary and range, that each but the offset
//! is static, and which element types take them ([`off_path`]).

use schemars::JsonSchema;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use super::keyframe::is_keyframe_list;

/// The fields ADR-0158 §1 gives to `path` alone.
pub const PATH_ONLY: [&str; 3] = ["stroke_join", "stroke_miter_limit", "stroke_cap"];

/// The fields ADR-0158 §1 gives to `path`, `rect` and `ellipse`, and not to text.
pub const SHAPES_ONLY: [&str; 2] = ["stroke_dash", "stroke_dash_offset"];

/// The one schema error for a stroke field on an element type that does not take it, naming
/// the first such field `rest` carries — rather than the unknown-key error, which would say
/// the key may belong to a newer format and must not be deleted (ADR-0158 §1).
pub fn off_path(kind: Option<&str>, rest: &Map<String, Value>) -> Option<String> {
    if kind == Some("text")
        && let Some(key) = SHAPES_ONLY.iter().find(|key| rest.contains_key(**key))
    {
        return Some(format!(
            "`{key}` belongs to `path`, `rect` and `ellipse`, not to a `text`: a text stroke \
             falls outside the glyph contour and is set per run, and takes no dash pattern. \
             Drop it, or draw the outline as a `path` (ADR-0158)"
        ));
    }
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

/// `stroke_dash`: the lengths a stroke alternates, dash, gap, dash, gap, starting with a
/// dash (ADR-0158 §5). A static list of 2 to 16 integer pixel counts, each at least 0, in
/// element space. The list is drawn as written: an odd number of entries is not doubled as
/// SVG doubles it, and a zero total has no pattern to repeat — both `validate`'s
/// `E-DASH-SHAPE`, not the schema's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DashPattern(pub Vec<i64>);

impl<'de> Deserialize<'de> for DashPattern {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if is_keyframe_list(&value) {
            return Err(D::Error::custom(
                "`stroke_dash` is static: a keyframe list is not a value of it; animate \
                 `stroke_dash_offset` instead (ADR-0158)",
            ));
        }
        let Some(entries) = value.as_array() else {
            return Err(D::Error::custom(format!(
                "`stroke_dash` is {value}: it is a list of 2 to 16 integer pixel lengths, \
                 dash, gap, dash, gap (ADR-0158)"
            )));
        };
        if !(2..=16).contains(&entries.len()) {
            return Err(D::Error::custom(format!(
                "`stroke_dash` has {} entries: it takes 2 to 16, dash, gap, dash, gap \
                 (ADR-0158)",
                entries.len()
            )));
        }
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| match entry.as_i64() {
                Some(length) if length >= 0 => Ok(length),
                _ => Err(D::Error::custom(format!(
                    "`stroke_dash[{index}]` is {entry}: each entry is an integer pixel length \
                     of at least 0 (ADR-0158)"
                ))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(DashPattern)
    }
}

impl JsonSchema for DashPattern {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DashPattern".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "array",
            "items": {"type": "integer", "format": "int64", "minimum": 0},
            "minItems": 2,
            "maxItems": 16,
            "description": "The lengths the stroke alternates, dash, gap, dash, gap, starting \
                            with a dash, in element pixels along the outline the stroke is \
                            drawn on. Static: animate `stroke_dash_offset` instead. An even \
                            number of entries with a total above 0; an odd list is not \
                            doubled (ADR-0158).",
        }))
        .expect("an object literal is a schema")
    }
}

/// `trim_start` and `trim_end` (ADR-0160 §2): a fraction of the outline's length, from `0`
/// to `1` inclusive, in a static value and in every keyframe record.
///
/// The type states the bound in the published schema, so the one derived list of animatable
/// properties reads it as the range a resolved value clamps to and a `shift` split may not
/// leave (ADR-0146 §5, §7).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TrimFraction(pub f64);

impl<'de> Deserialize<'de> for TrimFraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fraction = f64::deserialize(deserializer)?;
        if !(0.0..=1.0).contains(&fraction) {
            return Err(D::Error::custom(format!(
                "a trim value is {fraction}: `trim_start` and `trim_end` are numbers from 0 to \
                 1, fractions of the outline's length, in a static value and in every keyframe \
                 record (ADR-0160)"
            )));
        }
        Ok(TrimFraction(fraction))
    }
}

impl JsonSchema for TrimFraction {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "TrimFraction".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "type": "number",
            "format": "double",
            "minimum": 0.0,
            "maximum": 1.0,
            "description": "A fraction of the outline's length, measured from its start point \
                            along the outline the dashes use: from `0` to `1` inclusive, in a \
                            static value and in every keyframe record (ADR-0160).",
        }))
        .expect("an object literal is a schema")
    }
}

impl crate::resolve::Interpolate for TrimFraction {
    type Out = f64;

    fn between(a: &Self, b: &Self, p: f64) -> f64 {
        f64::between(&a.0, &b.0, p)
    }

    fn held(value: &Self) -> f64 {
        value.0
    }
}
