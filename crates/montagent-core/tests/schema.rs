//! The published schema, and the drift it exists to prevent.

use montagent_core::layout::{Published, canonical_order};
use montagent_core::schema;

const COMMITTED: &str = "../../schema/montagent.schema.json";

#[test]
fn the_committed_schema_and_the_generated_one_are_identical() {
    let committed = std::fs::read_to_string(COMMITTED).expect("the committed schema");

    assert_eq!(
        committed,
        schema::generated_bytes(),
        "the committed schema has drifted from the types.\n\
         Two artifacts is the drift this project has already found three times — in jury \
         evidence twice and in `validate`'s own measured facts once.\n\
         Regenerate with: cargo run -p montagent-core --example schema > {}",
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
    // ADR-0041 enumerated five; `layer` is the sixth, placed at the tail of the prefix
    // because it is a field of every type rather than of one (#243).
    let prefix = ["id", "type", "group", "start", "end", "layer"];

    let image = canonical_order(Published::Element("image")).expect("image is a published type");
    assert_eq!(&image[..6], &prefix);
    assert_eq!(
        &image[6..15],
        &[
            "source", "x", "y", "origin", "width", "height", "fit", "clip", "scale"
        ]
    );
    // ADR-0068: `effects` appends after the type's existing fields.
    assert_eq!(image.last().unwrap(), "effects");

    let text = canonical_order(Published::Element("text")).expect("text is a published type");
    assert_eq!(
        &text[6..17],
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

    let rect = canonical_order(Published::Element("rect")).expect("rect is a published type");
    assert_eq!(
        &rect[6..12],
        &["x", "y", "origin", "width", "height", "fill"]
    );

    let audio = canonical_order(Published::Element("audio")).expect("audio is a published type");
    assert_eq!(&audio[6..9], &["source", "source_start", "source_end"]);

    // ADR-0014 adds three fields to a shape — `stroke`, `stroke_width` and `radius` — in
    // the order that ADR's own headings introduce them, after ADR-0041's measured six.
    assert_eq!(
        &rect[12..15],
        &["stroke", "stroke_width", "radius"],
        "the three fields ADR-0014 adds, in its own order (#212)"
    );

    // `ellipse` is `rect`'s order **minus `radius`**, and the difference is exactly one
    // key. ADR-0014's heading is *"`radius` is a field on `rect`"*, and an ellipse
    // inscribing its rect has no corners to round — so `radius` on one is an unknown key
    // rather than a field that quietly does nothing (ADR-0007: "a field the renderer
    // cannot honour is worse than no field").
    let ellipse = canonical_order(Published::Element("ellipse")).expect("a published type");
    assert_eq!(
        ellipse,
        rect.iter()
            .filter(|key| *key != "radius")
            .cloned()
            .collect::<Vec<_>>()
    );
}

/// The `mask` branch of the published `Effect` union.
fn mask_member() -> serde_json::Value {
    schema::generate()["$defs"]["Effect"]["oneOf"]
        .as_array()
        .expect("the effect vocabulary is a union")
        .iter()
        .find(|branch| branch.pointer("/properties/name/const") == Some(&serde_json::json!("mask")))
        .expect("`mask` is a member of it")
        .clone()
}

#[test]
fn the_mask_members_key_order_is_the_one_adr_0084_takes() {
    // ADR-0041 hands a new field's position to the ADR that introduces it, and ADR-0084
    // takes it explicitly rather than leaving it to be read off a struct: `name, shape, x,
    // y, width, height, radius`, and ADR-0152 appends `invert` then `feather`. The rect fields follow `shape` in the order ADR-0012 fixed
    // for every other rect in the format, and `radius` trails them exactly as it trails the
    // drawn `shape` element's own fields under ADR-0014.
    //
    // `name` is the union's own tag and the generator appends it, as it does for all seven
    // members — so the order asserted here is the branch's declared fields, and `name`
    // leads the member on the wire because serde writes the tag first.
    let mask = mask_member();
    let declared: Vec<&str> = mask["properties"]
        .as_object()
        .expect("an object shape")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        declared,
        [
            "shape", "x", "y", "width", "height", "radius", "invert", "feather", "name"
        ]
    );
}

#[test]
fn the_schema_states_the_two_mask_rules_the_types_can_only_enforce() {
    // #168's two-artifact rule: the published schema must not admit files the binary
    // refuses. Both of ADR-0084's relational rules are enforced in
    // `crate::model::effects`, and both are said again here — the all-or-none rect as
    // `dependentRequired`, and `radius`-on-`rect`-only as the one conditional.
    let mask = mask_member();

    for field in ["x", "y", "width", "height"] {
        let siblings = mask["dependentRequired"][field]
            .as_array()
            .unwrap_or_else(|| panic!("`{field}` requires its siblings"));
        assert_eq!(siblings.len(), 3, "`{field}` names the other three");
        assert!(!siblings.contains(&serde_json::json!(field)));
    }

    assert_eq!(mask["if"]["properties"]["shape"]["const"], "rect");
    assert_eq!(
        mask["else"]["not"]["required"],
        serde_json::json!(["radius"])
    );
    assert!(
        mask["else"]["description"]
            .as_str()
            .is_some_and(|said| said.contains("corners to round")),
        "the prohibition names its reason, as the deserializer's own message does"
    );
}

/// The `chroma` branch of the published `Effect` union.
fn chroma_member() -> serde_json::Value {
    schema::generate()["$defs"]["Effect"]["oneOf"]
        .as_array()
        .expect("the effect vocabulary is a union")
        .iter()
        .find(|branch| {
            branch.pointer("/properties/name/const") == Some(&serde_json::json!("chroma"))
        })
        .expect("`chroma` is a member of it")
        .clone()
}

#[test]
fn the_schema_states_chromas_bounds_the_types_can_only_enforce() {
    // #168's two-artifact rule again, on ADR-0088's four narrowings: three scalars bounded
    // to `[0, 1]`, and a `color` narrowed to `#RRGGBB` with no alpha. All four are
    // enforced in `crate::model::effects`, and a schema silent about them would admit
    // files the binary refuses.
    //
    // They are also what makes the member *learnable by reading* (ADR-0017): the identity
    // value is one end of a stated range, and `{"type": "number"}` does not have an end.
    let chroma = chroma_member();

    for field in ["tolerance", "softness", "spill"] {
        assert_eq!(
            chroma["properties"][field]["minimum"],
            serde_json::json!(0.0),
            "`{field}` states its lower bound"
        );
        assert_eq!(
            chroma["properties"][field]["maximum"],
            serde_json::json!(1.0),
            "`{field}` states its upper bound"
        );
    }

    let color = &chroma["properties"]["color"];
    assert_eq!(color["pattern"], "^#[0-9A-F]{6}$");
    assert_eq!(
        color["allOf"][0]["$ref"], "#/$defs/Colour",
        "narrowed, not replaced: the value is still the format's one colour"
    );
}

#[test]
fn the_chroma_members_key_order_is_the_one_adr_0088_takes() {
    // ADR-0041 hands a new field's position to the ADR that introduces it, and ADR-0088
    // spells the member `chroma{color, tolerance, softness, spill}` in its own decision
    // section and in its one worked example. `name` is the union's own tag, which the
    // generator appends and serde writes first, exactly as it does for the other seven.
    let chroma = chroma_member();
    let declared: Vec<&str> = chroma["properties"]
        .as_object()
        .expect("an object shape")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        declared,
        ["color", "tolerance", "softness", "spill", "name"]
    );
}

#[test]
fn a_type_the_schema_does_not_publish_has_no_key_order() {
    assert_eq!(canonical_order(Published::Element("scene")), None);
}

/// The one applicator keyword whose subschema constrains an instance some *enclosing*
/// schema has already given a shape to, rather than declaring a shape of its own.
///
/// ADR-0017 closes every object shape the schema defines. An `if` is not one: ADR-0084's
/// `radius` conditional tests `{"shape": {"const": "rect"}}` against the whole mask member,
/// which also carries `name` and the rect fields — so closing it would not tighten the
/// schema, it would stop the condition ever matching and quietly withdraw the rule. The
/// shape those instances are held to is the branch's own `additionalProperties: false`,
/// which this walk still requires.
///
/// `then`, `else` and `not` are deliberately **not** here. The same argument would extend
/// to them, and nothing in this schema needs it yet — an exemption written before a schema
/// needs it is a place a future open object hides. The walk below also keeps descending
/// through an `if`, so a definition nested under one is still held to the rule; only the
/// condition object itself is excused.
const CONDITION: &str = "if";

/// Is this pointer the `if` *keyword*, rather than a field that happens to be called one?
///
/// The walk spells a field's path `…/properties/<name>`, so the two are distinguishable and
/// a format that one day publishes an `if` key does not silently lose its closure check.
fn is_condition(path: &str) -> bool {
    path.ends_with(&format!("/{CONDITION}")) && !path.ends_with(&format!("/properties/{CONDITION}"))
}

/// Every object schema that does not forbid unknown keys, by JSON pointer.
fn find_open_objects(schema: &serde_json::Value, path: String, open: &mut Vec<String>) {
    if let Some(body) = schema.as_object() {
        // A schema with `properties` is describing an object shape, and that is exactly the
        // kind of schema that must be closed. A branch of a discriminated union counts:
        // ADR-0017 closes "every object shape the schema defines … per discriminated type".
        //
        // A condition is the exception, and only the condition object itself: see
        // `CONDITION` above. The walk still descends through it.
        if body.contains_key("properties")
            && !is_condition(&path)
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

#[test]
fn no_def_name_is_a_number_the_generator_assigned() {
    // ADR-0137 §6. schemars numbers a generic type's instantiations in the order it meets
    // them (`Keyframe2`, `Keyframe3`, …), so reordering the Rust types renumbers them and
    // the name says nothing about the value type. Agents read these names in the schema
    // index and in every rewritten ref, so each one is built from its type parameter.
    //
    // A generator-assigned number is recognised by its shape: another definition's name
    // with digits after it. A trailing digit alone is not one — `Keyframeint64` names its
    // parameter, `int64`.
    let schema = schema::generate();
    let defs = schema["$defs"].as_object().expect("the schema's $defs");
    let numbered: Vec<&String> = defs
        .keys()
        .filter(|name| {
            let base = name.trim_end_matches(|c: char| c.is_ascii_digit());
            base.len() < name.len() && defs.contains_key(base)
        })
        .collect();
    assert!(numbered.is_empty(), "numbered $def names: {numbered:?}");
}
