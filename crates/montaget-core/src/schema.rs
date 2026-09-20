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

use serde_json::{Value, json};

use crate::model::Project;

/// Where the generated copy is committed, relative to the repository root.
pub const PUBLISHED: &str = "schema/montaget.schema.json";

/// The schema, generated from [`Project`].
pub fn generate() -> Value {
    let mut schema = schemars::schema_for!(Project).to_value();
    deny_null(&mut schema);
    publish_positional_ease(&mut schema);
    publish_bezier_bounds(&mut schema);
    header_first(schema)
}

/// Say ADR-0038's positional `ease` rule in the schema, where the types can only say it in
/// their deserializer.
///
/// Named for the publishing rather than for the rule, so that it and
/// `crate::model::keyframe`'s enforcement of the same rule do not read as one function in
/// two places. They are two statements of one rule, in the two artifacts #168 requires to
/// agree.
///
/// *"`ease` is required on every keyframe record except the first, where it remains a schema
/// error. Presence is a pure function of position in the list."* A Rust struct has no way to
/// express that — one `Keyframe` type describes records at both positions — so
/// [`crate::model::Animatable`] enforces it on the way in. JSON Schema **can** express it,
/// with `prefixItems`, and a published schema that left `ease` optional everywhere would
/// admit files the binary refuses: the two-artifact divergence ADR-0041 and #168 both name,
/// arriving through under-statement rather than through drift.
///
/// So this pass derives the first-record shape from the record shape mechanically — the same
/// arrangement, and the same reason, as [`deny_null`] above. There is no second hand-written
/// record definition to keep in step.
fn publish_positional_ease(schema: &mut Value) {
    let Some(Value::Object(defs)) = schema.get_mut("$defs") else {
        return;
    };

    // Recognised by shape rather than by name: a definition carrying exactly `t`, `v` and
    // `ease` is a keyframe record whatever the generator decided to call this instantiation
    // of it (`Keyframe`, `Keyframe2`, …).
    let records: Vec<String> = defs
        .iter()
        .filter(|(_, def)| is_keyframe_record(def))
        .map(|(name, _)| name.clone())
        .collect();

    let mut rebuilt = serde_json::Map::new();
    for (name, mut def) in std::mem::take(defs) {
        if !records.contains(&name) {
            rebuilt.insert(name, def);
            continue;
        }
        rebuilt.insert(first_name(&name), first_record(&def));
        // Every record this definition now describes is a non-first one, because the
        // `prefixItems` written below takes index 0 away from it.
        if let Some(Value::Array(required)) = def.get_mut("required") {
            required.push(Value::String("ease".into()));
        }
        rebuilt.insert(name, def);
    }
    *defs = rebuilt;

    for name in &records {
        with_prefix_item(schema, name);
    }
}

/// Is this definition one `{"t","v","ease"}` record?
fn is_keyframe_record(def: &Value) -> bool {
    let Some(Value::Object(properties)) = def.get("properties") else {
        return false;
    };
    properties.len() == 3
        && ["t", "v", "ease"]
            .iter()
            .all(|key| properties.contains_key(*key))
}

/// What the first record's definition is called, given the record definition's own name.
fn first_name(name: &str) -> String {
    format!("First{name}")
}

/// The same record with no `ease`, and a description saying why.
///
/// `additionalProperties: false` comes with the clone (ADR-0017 closes every object level),
/// and it is what turns the removal into the error ADR-0012 asks for: an `ease` on the first
/// record is an unknown key there, named, rather than a field quietly ignored.
fn first_record(def: &Value) -> Value {
    let mut first = def.clone();
    if let Some(Value::Object(properties)) = first.get_mut("properties") {
        properties.remove("ease");
    }
    if let Value::Object(body) = &mut first {
        body.insert(
            "description".into(),
            Value::String(
                "The first record of a keyframe list, which carries no `ease`: `ease` \
                 describes the segment *arriving at* a record, and nothing arrives at the \
                 first one. Writing one here is a schema error naming the convention, never \
                 an ignored field (ADR-0012, ADR-0038)."
                    .into(),
            ),
        );
    }
    first
}

/// Give every array of `name` records a `prefixItems` naming the first-record shape.
///
/// Under 2020-12 `prefixItems` claims index 0 and `items` applies from index 1 on, so the
/// two together are exactly the positional rule: no `ease` on the first record, a required
/// one on every other.
fn with_prefix_item(schema: &mut Value, name: &str) {
    let reference = format!("#/$defs/{name}");
    let mut visit = |body: &mut serde_json::Map<String, Value>| {
        if body.get("type").and_then(Value::as_str) != Some("array") {
            return;
        }
        if body.get("items").and_then(|items| items.get("$ref")) != Some(&json!(reference)) {
            return;
        }
        // Rebuilt rather than inserted, so `prefixItems` reads before the `items` it takes
        // the first index away from.
        let mut ordered = serde_json::Map::new();
        for (key, value) in std::mem::take(body) {
            if key == "items" {
                ordered.insert(
                    "prefixItems".into(),
                    json!([{"$ref": format!("#/$defs/{}", first_name(name))}]),
                );
            }
            ordered.insert(key, value);
        }
        // `prefixItems` constrains index 0 and does not require it to exist, so an empty
        // array would satisfy the schema while `Animatable`'s deserializer refuses it — it
        // reads `[]` as a static value and fails there. A list of no records is not a
        // property that never changes; it is a property with nothing said about it.
        ordered.insert("minItems".into(), json!(1));
        *body = ordered;
    };
    walk_objects(schema, &mut visit);
}

/// Every object in the schema, handed to `visit` — including the ones inside `$defs`.
fn walk_objects(schema: &mut Value, visit: &mut impl FnMut(&mut serde_json::Map<String, Value>)) {
    match schema {
        Value::Object(body) => {
            visit(body);
            for value in body.values_mut() {
                walk_objects(value, visit);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| walk_objects(item, visit)),
        _ => {}
    }
}

/// Say ADR-0012's bound on a raw ease's `x` control points in the schema.
///
/// *"`x1`/`x2` outside `[0,1]` is a schema error; `y` outside is legal, because overshoot is
/// a real need beziers give away free."* A Rust `[f64; 4]` cannot say that — the bound holds
/// for two of the four positions — so [`crate::model::Ease`] enforces it in its
/// deserializer, and this says the same thing in the published schema, by position, the way
/// [`publish_positional_ease`] does for `ease`.
fn publish_bezier_bounds(schema: &mut Value) {
    let Some(Value::Object(ease)) = schema.pointer_mut("/$defs/Ease") else {
        return;
    };
    let Some(Value::Array(branches)) = ease.get_mut("anyOf") else {
        return;
    };
    for branch in branches {
        let Value::Object(body) = branch else {
            continue;
        };
        // The raw form is the one branch of the union that is an array of four numbers; the
        // other is the published name.
        if body.get("type").and_then(Value::as_str) != Some("array") {
            continue;
        }
        let bound = |axis: &str, index: usize| {
            json!({
                "type": "number",
                "format": "double",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": format!(
                    "`{axis}{index}` — the curve's own time, which runs from 0 to 1. Outside \
                     that is a schema error (ADR-0012)."
                ),
            })
        };
        let free = |axis: &str, index: usize| {
            json!({
                "type": "number",
                "format": "double",
                "description": format!(
                    "`{axis}{index}` — outside `[0, 1]` is legal, because overshoot is a real \
                     need beziers give away free (ADR-0012)."
                ),
            })
        };
        let mut ordered = serde_json::Map::new();
        for (key, value) in std::mem::take(body) {
            if key == "items" {
                ordered.insert(
                    "prefixItems".into(),
                    json!([bound("x", 1), free("y", 1), bound("x", 2), free("y", 2)]),
                );
            }
            ordered.insert(key, value);
        }
        *body = ordered;
    }
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
