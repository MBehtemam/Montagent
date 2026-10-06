//! The schema index and its pieces (ADR-0137): the schema, served in parts that each fit
//! one tool result, and found through one index.

use montagent_core::resources;
use serde_json::Value;

/// ADR-0137 §8. In bytes, so the suite needs no tokenizer.
const BUDGET: usize = 24_576;

fn index() -> Value {
    let body = resources::read(resources::INDEX.uri).expect("the schema index");
    serde_json::from_str(&body).expect("the index is JSON")
}

fn read_json(uri: &str) -> Value {
    let body = resources::read(uri).unwrap_or_else(|| panic!("{uri} is listed but not served"));
    serde_json::from_str(&body).unwrap_or_else(|e| panic!("{uri} is not JSON: {e}"))
}

/// Every `montagent://` URI the index names, in the order it names them.
fn listed_uris(index: &Value) -> Vec<String> {
    let mut uris = Vec::new();
    strings(index, &mut |s| {
        if s.starts_with("montagent://") {
            uris.push(s.to_string());
        }
    });
    uris
}

fn strings(value: &Value, visit: &mut impl FnMut(&str)) {
    match value {
        Value::String(s) => visit(s),
        Value::Array(items) => items.iter().for_each(|item| strings(item, visit)),
        Value::Object(map) => map.values().for_each(|child| strings(child, visit)),
        _ => {}
    }
}

fn refs(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::Array(items) => items.iter().for_each(|item| refs(item, found)),
        Value::Object(map) => {
            for (key, child) in map {
                match (key.as_str(), child) {
                    ("$ref", Value::String(target)) => found.push(target.clone()),
                    _ => refs(child, found),
                }
            }
        }
        _ => {}
    }
}

fn pieces() -> Vec<(String, Value)> {
    listed_uris(&index())
        .into_iter()
        .map(|uri| {
            let piece = read_json(&uri);
            (uri, piece)
        })
        .collect()
}

#[test]
fn every_uri_the_index_lists_serves_json() {
    // Pieces need not be in `resources/list` (ADR-0137 §3): the index is how they are
    // found, so a URI it lists that reads as nothing is a dead end with no other way round.
    let uris = listed_uris(&index());
    assert!(uris.len() > 30, "the index lists only {uris:?}");
    for (uri, piece) in pieces() {
        assert!(piece.is_object(), "{uri} is not a schema object");
    }
}

#[test]
fn the_index_holds_every_element_type_and_effect_name_with_its_required_keys() {
    let schema = montagent_core::schema::generate();
    let index = index();
    for (union, key, section) in [
        ("Element", "type", "elements"),
        ("Effect", "name", "effects"),
    ] {
        let branches = schema["$defs"][union]["oneOf"].as_array().unwrap();
        let listed = index[section]
            .as_object()
            .unwrap_or_else(|| panic!("index.{section}"));
        assert_eq!(listed.len(), branches.len(), "index.{section}");
        for branch in branches {
            let name = branch["properties"][key]["const"].as_str().unwrap();
            assert_eq!(
                listed[name]["required"], branch["required"],
                "{section}.{name}"
            );
            assert!(listed[name]["uri"].is_string(), "{section}.{name}");
        }
    }
}

#[test]
fn the_index_carries_no_descriptions() {
    // ADR-0137 §3: the rules stay in the pieces, beside their keys.
    let text = resources::read(resources::INDEX.uri).unwrap();
    assert!(!text.contains("\"description\""), "{text}");
}

#[test]
fn every_ref_in_every_piece_resolves_to_a_listed_piece_or_a_section_of_the_index() {
    // ADR-0137 §4. A `#/$defs/X` ref cut out of the schema resolves to nothing inside the
    // piece alone, so each one is rewritten to where X is served.
    let index = index();
    let listed = listed_uris(&index);
    for (uri, piece) in pieces() {
        let mut found = Vec::new();
        refs(&piece, &mut found);
        for target in found {
            let resolves = match target.split_once('#') {
                Some((base, pointer)) if base == resources::INDEX.uri => {
                    index.pointer(pointer).is_some()
                }
                Some(_) => false,
                None => listed.contains(&target),
            };
            assert!(resolves, "{uri} refers to {target}, which nothing serves");
        }
    }
}

#[test]
fn no_resource_meant_to_be_read_whole_is_over_budget() {
    // ADR-0137 §8. `schema.json` is exempt: it is the whole, and the pieces are its answer.
    let mut read_whole: Vec<String> = vec![
        resources::INDEX.uri.to_string(),
        resources::FORMAT.uri.to_string(),
    ];
    read_whole.extend(
        resources::FORMAT_PAGES
            .iter()
            .map(|page| page.uri.to_string()),
    );
    read_whole.extend(listed_uris(&index()));
    for uri in read_whole {
        let size = resources::read(&uri).unwrap().len();
        assert!(
            size <= BUDGET,
            "{uri} is {size} bytes, {} over the {BUDGET}-byte budget. Split it; do not raise \
             the number (ADR-0137 §8 — raising it takes an ADR)",
            size - BUDGET
        );
    }
}

/// `value` with every `$ref` dropped, so a piece and its branch compare on everything but
/// where their refs point.
fn without_refs(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(without_refs).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| key.as_str() != "$ref")
                .map(|(key, child)| (key.clone(), without_refs(child)))
                .collect(),
        ),
        other => other.clone(),
    }
}

#[test]
fn each_piece_has_the_properties_and_required_of_its_part_of_the_schema() {
    let schema = montagent_core::schema::generate();
    let index = index();

    let mut expected: Vec<(String, &Value)> = vec![(
        index["project"]["uri"].as_str().unwrap().to_string(),
        &schema,
    )];
    for (union, key, section) in [
        ("Element", "type", "elements"),
        ("Effect", "name", "effects"),
    ] {
        for branch in schema["$defs"][union]["oneOf"].as_array().unwrap() {
            let name = branch["properties"][key]["const"].as_str().unwrap();
            let uri = index[section][name]["uri"].as_str().unwrap().to_string();
            expected.push((uri, branch));
        }
    }

    for (uri, part) in expected {
        let piece = read_json(&uri);
        for field in ["properties", "required"] {
            assert_eq!(
                without_refs(&piece[field]),
                without_refs(&part[field]),
                "{uri}'s {field}"
            );
        }
    }
}

#[test]
fn every_other_def_is_served_whole() {
    let schema = montagent_core::schema::generate();
    let index = index();
    for (name, def) in schema["$defs"].as_object().unwrap() {
        if name == "Element" || name == "Effect" {
            continue;
        }
        let uri = index["definitions"][name]
            .as_str()
            .unwrap_or_else(|| panic!("the index lists no piece for {name}"));
        assert_eq!(without_refs(&read_json(uri)), without_refs(def), "{name}");
    }
}

#[test]
fn every_uri_the_routing_text_names_is_served() {
    // ADR-0137 §7: the pointer decides whether pieces are used, so a pointer to nothing is
    // the one failure this routing cannot afford.
    let mut texts = vec![resources::SERVER_INSTRUCTIONS];
    texts.extend(resources::all().iter().map(|r| r.description));
    for text in texts {
        for (at, _) in text.match_indices("montagent://") {
            let uri: String = text[at..]
                .chars()
                .take_while(|c| !c.is_whitespace() && !matches!(c, '`' | ')' | ','))
                .collect();
            let uri = uri.trim_end_matches('.');
            assert!(
                resources::read(uri).is_some(),
                "{uri} is named but not served"
            );
        }
    }
    assert!(
        resources::SERVER_INSTRUCTIONS.contains(resources::INDEX.uri),
        "the server instructions start an agent at the index"
    );
}

#[test]
fn a_uri_beside_a_piece_serves_nothing() {
    assert_eq!(
        resources::read("montagent://schema/def/NoSuchDef.json"),
        None
    );
    assert_eq!(
        resources::read("montagent://schema/element/sprite.json"),
        None
    );
    assert_eq!(resources::read("montagent://schema/effect/glow.json"), None);
    assert_eq!(resources::read("montagent://schema/def/Element.json"), None);
}
