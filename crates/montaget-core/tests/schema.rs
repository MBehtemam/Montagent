//! The published schema, and the drift it exists to prevent.

use montaget_core::model::canonical_key_order;
use montaget_core::schema;

const COMMITTED: &str = "../../schema/montaget.schema.json";

#[test]
fn the_committed_schema_and_the_generated_one_are_identical() {
    let committed = std::fs::read_to_string(COMMITTED).expect("the committed schema");

    assert_eq!(
        committed,
        schema::generated_bytes(),
        "the committed schema has drifted from the types.\n\
         Two artifacts is the drift this project has already found three times — in jury \
         evidence twice and in `validate`'s own measured facts once.\n\
         Regenerate with: cargo run -p montaget-core --example schema > {}",
        schema::PUBLISHED
    );
}

#[test]
fn the_schema_is_closed_at_every_object_level() {
    // ADR-0017, scope: closed uniformly, every object shape — not only the levels most
    // exposed to revision skew. `run` and `keyframe` are authored in the highest
    // multiplicity per file, so if anything they are *more* exposed to copy-paste typos
    // than the project header is.
    let schema = schema::generate();
    let mut open = Vec::new();
    find_open_objects(&schema, String::from("#"), &mut open);
    assert!(
        open.is_empty(),
        "these object schemas admit unknown keys: {open:?}"
    );
}

#[test]
fn no_optional_field_may_be_written_as_null() {
    // ADR-0030 makes omitted-versus-present content. A third spelling that means the same
    // as omitted would be the redundant duplicate this format retires everywhere else.
    let schema = serde_json::to_string(&schema::generate()).unwrap();
    assert!(
        !schema.contains("\"null\""),
        "an optional field is omitted or carries a value; it is never written as null"
    );
}

#[test]
fn canonical_key_order_is_the_schemas_property_order() {
    // ADR-0041's measured table, which cost zero bytes against the committed fixture
    // because the four types it uses were already internally consistent.
    let prefix = ["id", "type", "group", "start", "end"];

    let image = canonical_key_order("image").expect("image is a published type");
    assert_eq!(&image[..5], &prefix);
    assert_eq!(
        &image[5..14],
        &["source", "x", "y", "origin", "width", "height", "fit", "clip", "scale"]
    );
    // ADR-0068: `effects` appends after the type's existing fields.
    assert_eq!(image.last().unwrap(), "effects");

    let text = canonical_key_order("text").expect("text is a published type");
    assert_eq!(
        &text[5..16],
        &[
            "x",
            "y",
            "origin",
            "width",
            "height",
            "font",
            "size",
            "line_height",
            "color",
            "align",
            "runs"
        ]
    );

    let rect = canonical_key_order("rect").expect("rect is a published type");
    assert_eq!(
        &rect[5..11],
        &["x", "y", "origin", "width", "height", "fill"]
    );

    let audio = canonical_key_order("audio").expect("audio is a published type");
    assert_eq!(&audio[5..8], &["source", "source_start", "source_end"]);

    // `ellipse` inherits `rect`'s shape, which ADR-0041 says fixes its order too.
    assert_eq!(canonical_key_order("ellipse"), canonical_key_order("rect"));
}

#[test]
fn a_type_the_schema_does_not_publish_has_no_key_order() {
    assert_eq!(canonical_key_order("scene"), None);
}

/// Every object schema that does not forbid unknown keys, by JSON pointer.
fn find_open_objects(schema: &serde_json::Value, path: String, open: &mut Vec<String>) {
    if let Some(body) = schema.as_object() {
        // A schema with `properties` is describing an object shape, and that is exactly the
        // kind of schema that must be closed. A branch of a discriminated union counts:
        // ADR-0017 closes "every object shape the schema defines … per discriminated type".
        if body.contains_key("properties")
            && body.get("additionalProperties") != Some(&serde_json::Value::Bool(false))
            && body.get("unevaluatedProperties") != Some(&serde_json::Value::Bool(false))
        {
            open.push(path.clone());
        }
        for (key, value) in body {
            // `properties` holds field schemas keyed by field name; their own nested shapes
            // are reached through `$defs`, so walking into them would report the same open
            // shape under several names.
            if key != "properties" {
                find_open_objects(value, format!("{path}/{key}"), open);
            } else if let Some(fields) = value.as_object() {
                for (name, field) in fields {
                    find_open_objects(field, format!("{path}/properties/{name}"), open);
                }
            }
        }
    } else if let Some(items) = schema.as_array() {
        for (i, item) in items.iter().enumerate() {
            find_open_objects(item, format!("{path}/{i}"), open);
        }
    }
}
