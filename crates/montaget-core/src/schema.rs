//! The published JSON Schema, generated from the Rust types.
//!
//! ADR-0041 ties canonical key order to the schema's property-declaration order precisely
//! so there is **one** place that order can go stale — *"not a second, hand-maintained
//! list"*, after the same two-artifact drift had already been found twice in jury evidence
//! and once in `validate`'s own measured facts. Generating the schema from the types is
//! what makes that one place the types themselves.
//!
//! The file is committed as well as generated, because the schema is a resource the MCP
//! surface publishes (ADR-0011) and *"discoverability is the schema's job"*. A test
//! compares the two, so the committed copy cannot quietly fall behind the types — the
//! drift this arrangement exists to prevent would otherwise just reappear one level up.

use serde_json::Value;

use crate::model::Project;

/// Where the generated copy is committed, relative to the repository root.
pub const PUBLISHED: &str = "schema/montaget.schema.json";

/// The schema, generated from [`Project`].
pub fn generate() -> Value {
    let mut schema = schemars::schema_for!(Project).to_value();
    deny_null(&mut schema);
    header_first(schema)
}

/// Move `$schema`, `title` and `description` to the front.
///
/// The generator appends them, which is correct JSON and reads backwards: the first thing
/// in a published schema should say what it is. Everything after the header keeps the order
/// it was generated in, because that order **is** canonical key order (ADR-0041) and this
/// pass must not disturb it.
fn header_first(schema: Value) -> Value {
    let Value::Object(mut body) = schema else {
        return schema;
    };
    let mut ordered = serde_json::Map::new();
    for key in ["$schema", "title", "description"] {
        if let Some(value) = body.remove(key) {
            ordered.insert(key.to_string(), value);
        }
    }
    ordered.extend(body);
    Value::Object(ordered)
}

/// Strip the `null` alternative an optional field's schema is generated with.
///
/// `Option<T>` means *omitted or a value*, never *written as null* — ADR-0030 makes the
/// omitted/present distinction content, and a third spelling that means the same as omitted
/// would be exactly the redundant second spelling this format retires everywhere it finds
/// one. The generator cannot know that, since `Option<T>` in Rust genuinely does have a
/// `None` it could serialise; the types express the difference with
/// `skip_serializing_if`, and this pass says the same thing in the schema.
fn deny_null(schema: &mut Value) {
    match schema {
        Value::Object(body) => {
            if let Some(Value::Object(properties)) = body.get_mut("properties") {
                for value in properties.values_mut() {
                    drop_null_alternative(value);
                }
            }
            for value in body.values_mut() {
                deny_null(value);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(deny_null),
        _ => {}
    }
}

fn drop_null_alternative(property: &mut Value) {
    let Value::Object(body) = property else {
        return;
    };

    // `"type": ["integer", "null"]` — the shape a nullable scalar is generated as.
    if let Some(Value::Array(types)) = body.get_mut("type") {
        types.retain(|t| t != "null");
        if types.len() == 1 {
            let only = types[0].clone();
            body.insert("type".into(), only);
        }
    }

    // `"anyOf": [{"$ref": …}, {"type": "null"}]` — the shape a nullable named type is
    // generated as. With the null branch gone the union has one member, and a one-member
    // union is the member.
    if let Some(Value::Array(branches)) = body.get_mut("anyOf") {
        branches.retain(|branch| branch.get("type").and_then(Value::as_str) != Some("null"));
        if branches.len() == 1 {
            let only = branches[0].clone();
            body.remove("anyOf");
            if let Value::Object(only) = only {
                for (key, value) in only {
                    body.entry(key).or_insert(value);
                }
            }
        }
    }
}

/// The schema as bytes, in the form the committed file carries: pretty-printed, two-space
/// indent, one trailing newline. Stated here rather than at the test so the generator and
/// the comparison cannot disagree about what "identical" means.
pub fn generated_bytes() -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(&generate()).expect("a schema serialises")
    )
}
