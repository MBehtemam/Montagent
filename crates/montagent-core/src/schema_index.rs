//! The schema index and the schema pieces (ADR-0137): the published schema, served in parts
//! that each fit one tool result, and found through one index.
//!
//! `montagent://schema.json` is about 73 KB, and a client's tool result holds about 50,000
//! characters; in the no-skills baseline it overflowed in 12 of 12 runs. The whole schema
//! stays whole (ADR-0137 §1). This module serves it a second way:
//!
//! - **the index** names each element `type` and each effect `name` with its required keys,
//!   and gives the URI of every piece. It carries no descriptions; the rules stay in the
//!   pieces, beside their keys.
//! - **a piece** is one part of the schema: the project's own keys, one branch of
//!   `Element`'s or `Effect`'s `oneOf`, or one other `$def` whole.
//!
//! **Both are cut from the schema [`crate::schema::generate`] returns, on every read** — the
//! same value `schema.json` serialises, so neither can drift from it (ADR-0137 §2). A piece is
//! a derived view rather than a byte slice: each `#/$defs/X` ref in it is rewritten to the
//! URI that serves `X`, because the fragment resolves to nothing once it is cut out (§4).
//!
//! Only the index URI is a published promise (§5). Piece URIs are reached through the index
//! and may move when a type is renamed.

use serde_json::{Map, Value, json};

use crate::resources;

/// Where every piece is served, under one prefix the index's own URI shares.
const PIECES: &str = "montagent://schema/";

/// The piece for the project's own keys: the schema root has no `$def` to serve it by.
const PROJECT: &str = "project";

/// A `$def` that is a `oneOf` of tagged branches, served one branch per piece because an
/// agent writes one element type or one effect at a time (ADR-0137 §3).
struct Union {
    /// Its `$def` name.
    def: &'static str,
    /// The key whose `const` names each branch.
    tag: &'static str,
    /// Its section of the index, which is where a ref to the whole union points.
    section: &'static str,
    /// The path segment its pieces are served under.
    segment: &'static str,
}

const UNIONS: [Union; 3] = [
    Union {
        def: "Element",
        tag: "type",
        section: "elements",
        segment: "element",
    },
    Union {
        def: "Effect",
        tag: "name",
        section: "effects",
        segment: "effect",
    },
    // ADR-0169. It has no branch until a capability ADR adds a member, so its section is
    // empty; naming it here is what stops the first member being served as a whole `$def`.
    Union {
        def: "AudioEffect",
        tag: "name",
        section: "audio_effects",
        segment: "audio_effect",
    },
];

/// The `$def`s served whole, under this segment.
const DEFINITION: &str = "def";

fn piece_uri(path: &str) -> String {
    format!("{PIECES}{path}.json")
}

fn union_named(def: &str) -> Option<&'static Union> {
    UNIONS.iter().find(|union| union.def == def)
}

/// One union's branches, each with the name its tag gives it.
fn branches<'a>(schema: &'a Value, union: &Union) -> impl Iterator<Item = (&'a str, &'a Value)> {
    schema["$defs"][union.def]["oneOf"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(move |branch| {
            let name = branch["properties"][union.tag]["const"].as_str()?;
            Some((name, branch))
        })
}

/// Every `$def` served whole: all of them but the unions, which are split.
fn definitions(schema: &Value) -> impl Iterator<Item = (&String, &Value)> {
    schema["$defs"]
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(name, _)| union_named(name).is_none())
}

/// The index: what an agent reads first to find the part of the schema it needs.
pub fn index(schema: &Value) -> Value {
    let mut index = Map::new();
    index.insert(
        PROJECT.into(),
        json!({"required": schema["required"], "uri": piece_uri(PROJECT)}),
    );
    for union in &UNIONS {
        let section: Map<String, Value> = branches(schema, union)
            .map(|(name, branch)| {
                let uri = piece_uri(&format!("{}/{name}", union.segment));
                (
                    name.to_string(),
                    json!({"required": branch["required"], "uri": uri}),
                )
            })
            .collect();
        index.insert(union.section.into(), Value::Object(section));
    }
    let defs: Map<String, Value> = definitions(schema)
        .map(|(name, _)| {
            let uri = piece_uri(&format!("{DEFINITION}/{name}"));
            (name.clone(), Value::String(uri))
        })
        .collect();
    index.insert("definitions".into(), Value::Object(defs));
    Value::Object(index)
}

/// The piece served at `uri`, or `None` if no piece is.
pub fn piece(schema: &Value, uri: &str) -> Option<Value> {
    let path = uri.strip_prefix(PIECES)?.strip_suffix(".json")?;
    let mut piece = if path == PROJECT {
        let mut root = schema.as_object()?.clone();
        root.remove("$schema");
        root.remove("$defs");
        Value::Object(root)
    } else {
        let (segment, name) = path.split_once('/')?;
        if segment == DEFINITION {
            definitions(schema)
                .find(|(def, _)| def.as_str() == name)
                .map(|(_, def)| def.clone())?
        } else {
            let union = UNIONS.iter().find(|union| union.segment == segment)?;
            branches(schema, union)
                .find(|(branch, _)| *branch == name)
                .map(|(_, branch)| branch.clone())?
        }
    };
    rewrite_refs(&mut piece);
    Some(piece)
}

/// Point every `#/$defs/X` ref at the URI that serves `X`: a union's section of the index,
/// or `X`'s own piece.
fn rewrite_refs(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                match (key.as_str(), child) {
                    ("$ref", Value::String(target)) => {
                        if let Some(def) = target.strip_prefix("#/$defs/") {
                            *target = match union_named(def) {
                                Some(union) => {
                                    format!("{}#/{}", resources::INDEX.uri, union.section)
                                }
                                None => piece_uri(&format!("{DEFINITION}/{def}")),
                            };
                        }
                    }
                    (_, child) => rewrite_refs(child),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(rewrite_refs),
        _ => {}
    }
}

/// The index as served: pretty-printed, as `schema.json` is.
pub fn index_bytes() -> String {
    crate::schema::pretty(&index(&crate::schema::generate()))
}

/// The piece at `uri` as served, or `None` if no piece is.
pub fn piece_bytes(uri: &str) -> Option<String> {
    // Checked before generating, so a URI outside the pieces costs no schema generation.
    if !uri.starts_with(PIECES) {
        return None;
    }
    piece(&crate::schema::generate(), uri).map(|piece| crate::schema::pretty(&piece))
}
