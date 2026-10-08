//! `Element`'s three hand-written impls, and the one reason they are hand-written.
//!
//! An element's canonical key order is `id, type, group, start, end, layer, …` (ADR-0041,
//! and #243 for `layer`'s position) — the discriminator sits *second*, between two prefix
//! fields. No derive produces that: `#[serde(tag = "type")]` writes the tag first, and
//! `#[serde(flatten)]` would put the prefix first but silently drops `deny_unknown_fields`,
//! which ADR-0017 requires at every object level. So the prefix is assembled by hand, in
//! all three directions — the wire form, the parse, and the published schema — and each of
//! the three is the same six names in the same order.

use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::{Error as _, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use super::{Body, Element, Layer};

/// The universal prefix, in canonical order. `type` is the body's own discriminator and is
/// spliced in at position 1 rather than carried as a field.
const PREFIX: [&str; 6] = ["id", "type", "group", "start", "end", "layer"];

impl Serialize for Element {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let body = serde_json::to_value(&self.body).map_err(serde::ser::Error::custom)?;
        let Value::Object(body) = body else {
            return Err(serde::ser::Error::custom(
                "an element body is always an object",
            ));
        };

        // `group` is omitted entirely when absent rather than written as `null`, which is
        // what every element in the committed fixture already does.
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("type", self.body.type_name())?;
        if let Some(group) = &self.group {
            map.serialize_entry("group", group)?;
        }
        map.serialize_entry("start", &self.start)?;
        map.serialize_entry("end", &self.end)?;
        // Omitted when absent, like `group`: the track supplies stacking unless this
        // element overrides it, and writing the track's value here would turn a
        // declaration ("wherever my track sits") into a pinned number (ADR-0030).
        if let Some(layer) = &self.layer {
            map.serialize_entry("layer", layer)?;
        }
        for (key, value) in &body {
            if key != "type" {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Element {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ElementVisitor)
    }
}

struct ElementVisitor;

impl<'de> Visitor<'de> for ElementVisitor {
    type Value = Element;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an element object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Element, A::Error> {
        // Read the whole object first, then split it: the prefix is read here and
        // everything else is handed to the body, whose own `deny_unknown_fields` is what
        // rejects an unknown key. Splitting it any other way would need this function to
        // know every type's field set — a second enumeration of the schema, in the one
        // place a drift between two enumerations would go unnoticed.
        let mut rest = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            if rest.insert(key.clone(), value).is_some() {
                return Err(A::Error::custom(format!(
                    "`{key}` appears twice on one element"
                )));
            }
        }

        let id = take_string(&mut rest, "id")?;
        let group = match rest.remove("group") {
            Some(Value::String(group)) => Some(group),
            // Written as `null`, which the canonical form never emits: the field is either
            // a group name or absent.
            Some(Value::Null) => {
                return Err(A::Error::custom(format!(
                    "`{id}`: `group` is written as null; omit the key instead"
                )));
            }
            Some(other) => {
                return Err(A::Error::custom(format!(
                    "`{id}`: `group` is {other}, not a name"
                )));
            }
            None => None,
        };
        let start = take_i64(&mut rest, "start", &id)?;
        let end = take_i64(&mut rest, "end", &id)?;
        let layer = match rest.remove("layer") {
            // The two forms the format writes, and no third: an integer, or an anchor.
            // Anything else — a bare string, `{"below": 3}`, `null` — is a schema error
            // here rather than a value some later stage has to second-guess.
            Some(value) => Some(serde_json::from_value::<Layer>(value.clone()).map_err(|_| {
                A::Error::custom(format!(
                    "`{id}`: `layer` is {value}, not an integer or an anchor such as \
{{\"below\": \"title\"}}"
                ))
            })?),
            None => None,
        };

        // Before the body's own `deny_unknown_fields`, whose unknown-key reading would say a
        // path-only stroke field may belong to a newer format (ADR-0158 §1).
        if let Some(reason) =
            super::stroke::off_path(rest.get("type").and_then(Value::as_str), &rest)
        {
            return Err(A::Error::custom(format!("`{id}`: {reason}")));
        }
        // A nest is a pure matrix (#632): the error says why, instead of the generic
        // unknown-key reading, which would suggest the field belongs to a newer format.
        if rest.get("type").and_then(Value::as_str) == Some("nest") {
            if layer.is_some() {
                return Err(A::Error::custom(format!(
                    "`{id}`: a nest has no `layer`; layers stay global, so give the layer to the \
children inside it"
                )));
            }
            for key in ["opacity", "blend", "effects", "mask", "clip", "motion_blur"] {
                if rest.contains_key(key) {
                    return Err(A::Error::custom(format!(
                        "`{id}`: a nest carries no `{key}`: group compositing is not part of a \
nest, which is a pure matrix"
                    )));
                }
            }
        }
        let body: Body = serde_json::from_value(Value::Object(rest))
            .map_err(|e| A::Error::custom(format!("`{id}`: {e}")))?;

        Ok(Element {
            id,
            group,
            start,
            end,
            layer,
            body,
        })
    }
}

fn take_string<E: serde::de::Error>(map: &mut Map<String, Value>, key: &str) -> Result<String, E> {
    match map.remove(key) {
        Some(Value::String(text)) => Ok(text),
        Some(other) => Err(E::custom(format!("`{key}` is {other}, not a string"))),
        None => Err(E::custom(format!("an element carries no `{key}`"))),
    }
}

fn take_i64<E: serde::de::Error>(
    map: &mut Map<String, Value>,
    key: &str,
    id: &str,
) -> Result<i64, E> {
    match map.remove(key) {
        Some(Value::Number(n)) => n
            .as_i64()
            .ok_or_else(|| E::custom(format!("`{id}`: `{key}` is {n}, not integer milliseconds"))),
        Some(other) => Err(E::custom(format!(
            "`{id}`: `{key}` is {other}, not integer milliseconds"
        ))),
        None => Err(E::custom(format!("`{id}` carries no `{key}`"))),
    }
}

impl JsonSchema for Element {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Element".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        // The body's schema is a `oneOf` over the types, each already carrying its own
        // `type` const and its own property order. Rebuilding each branch's `properties`
        // with the prefix in front is what publishes canonical key order — the schema is
        // where that order lives, and `fmt` and `validate`'s `LAYOUT` check read it from
        // here rather than from a second hand-maintained list.
        let mut schema = Body::json_schema(generator).to_value();
        let branches = schema
            .get_mut("oneOf")
            .and_then(Value::as_array_mut)
            .expect("the element body is a discriminated union");

        for branch in branches.iter_mut() {
            let Some(properties) = branch.get("properties").and_then(Value::as_object) else {
                continue;
            };
            let mut ordered = Map::new();
            for name in PREFIX {
                match name {
                    "type" => {
                        if let Some(tag) = properties.get("type") {
                            ordered.insert("type".into(), tag.clone());
                        }
                    }
                    name => {
                        ordered.insert(name.into(), prefix_property(generator, name));
                    }
                }
            }
            for (name, value) in properties {
                if name != "type" {
                    ordered.insert(name.clone(), value.clone());
                }
            }

            let required = branch
                .get("required")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut all_required: Vec<Value> =
                vec!["id".into(), "type".into(), "start".into(), "end".into()];
            for name in required {
                if !all_required.contains(&name) {
                    all_required.push(name);
                }
            }

            branch["properties"] = Value::Object(ordered);
            branch["required"] = Value::Array(all_required);
        }

        Schema::try_from(schema).expect("a rebuilt object schema is still a schema")
    }
}

fn prefix_property(generator: &mut SchemaGenerator, name: &str) -> Value {
    match name {
        "id" | "group" => generator.subschema_for::<String>().to_value(),
        "layer" => generator.subschema_for::<Layer>().to_value(),
        _ => generator.subschema_for::<i64>().to_value(),
    }
}
