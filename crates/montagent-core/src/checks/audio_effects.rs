//! The `audio_effects` list's own rules (ADR-0169), decided from the file:
//!
//! - **`E-AUDIO-EFFECT-WRONG-LIST`** (`error`): an audio member in `effects`, or a visual one
//!   in `audio_effects`. ADR-0040 keeps the two vocabularies apart, and the message names the
//!   list the member belongs in. This check speaks for the member, so
//!   [`speaks_for`] lets `E-SCHEMA` stay silent about the same unknown `name`.
//! - **`E-AUDIO-EFFECT-SINGULAR`** (`error`): a second enabled copy of a member whose own ADR
//!   declares it singular ([`SINGULAR`]). Disabled copies do not count.
//! - **`R-AUDIO-EFFECT-DISABLED`** (`review`): a member left at `"enabled": false`. In a
//!   finished project a bypassed member is a leftover; it is a routine step while listening
//!   against the source, which is why it is a review and not an error.
//!
//! The two vocabularies are read off the published schema, so there is no list of member
//! names here to drift from the types. [`SINGULAR`] is the one declaration this module owns:
//! a capability ADR that makes its member singular adds the name.

use std::sync::OnceLock;

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// The audio members that ADR-0169 lets appear once among the enabled ones. Empty: no
/// member exists yet. Loudness normalisation is the expected first entry.
pub const SINGULAR: &[&str] = &[];

/// What counts as an audio member, a visual one, and a singular one.
pub(crate) struct Vocabulary {
    audio: Vec<String>,
    visual: Vec<String>,
    singular: Vec<String>,
}

impl Vocabulary {
    /// The vocabulary the published schema declares, once per process.
    fn published() -> &'static Vocabulary {
        static PUBLISHED: OnceLock<Vocabulary> = OnceLock::new();
        PUBLISHED.get_or_init(|| {
            let schema = crate::schema::generate();
            let names = |def: &str| -> Vec<String> {
                schema["$defs"][def]["oneOf"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|branch| branch["properties"]["name"]["const"].as_str())
                    .map(str::to_string)
                    .collect()
            };
            Vocabulary {
                audio: names("AudioEffect"),
                visual: names("Effect"),
                singular: SINGULAR.iter().map(|name| name.to_string()).collect(),
            }
        })
    }
}

/// One member sitting in the wrong list.
#[derive(Debug, PartialEq)]
pub(crate) struct Misplaced {
    /// The list it was written in.
    pub list: &'static str,
    /// The list it belongs in.
    pub right: &'static str,
    pub index: usize,
    pub name: String,
}

fn members<'a>(element: &'a Value, list: &str) -> &'a [Value] {
    element
        .get(list)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn name_of(member: &Value) -> Option<&str> {
    member.get("name").and_then(Value::as_str)
}

fn has_sound(element: &Value) -> bool {
    matches!(
        element.get("type").and_then(Value::as_str),
        Some("audio" | "video")
    )
}

/// Every member of `element` written in the other vocabulary's list.
pub(crate) fn misplaced(element: &Value, vocab: &Vocabulary) -> Vec<Misplaced> {
    let mut out = Vec::new();
    for (index, member) in members(element, "effects").iter().enumerate() {
        if let Some(name) = name_of(member).filter(|n| vocab.audio.iter().any(|a| a == n)) {
            out.push(Misplaced {
                list: "effects",
                right: "audio_effects",
                index,
                name: name.to_string(),
            });
        }
    }
    if has_sound(element) {
        for (index, member) in members(element, "audio_effects").iter().enumerate() {
            if let Some(name) = name_of(member).filter(|n| vocab.visual.iter().any(|v| v == n)) {
                out.push(Misplaced {
                    list: "audio_effects",
                    right: "effects",
                    index,
                    name: name.to_string(),
                });
            }
        }
    }
    out
}

/// Whether this check already reports the fault `reason` (a `serde` message about
/// `element`): an unknown variant that is a member of the other list.
pub(crate) fn speaks_for(element: &Value, reason: &str) -> bool {
    misplaced(element, Vocabulary::published())
        .iter()
        .any(|m| reason.contains(&format!("unknown variant `{}`", m.name)))
}

fn enabled(member: &Value) -> bool {
    member.get("enabled") != Some(&Value::Bool(false))
}

pub fn check(document: &Loose, report: &mut Report) {
    for finding in findings(document, Vocabulary::published()) {
        report.push(finding);
    }
}

pub(crate) fn findings(document: &Loose, vocab: &Vocabulary) -> Vec<Finding> {
    let mut out = Vec::new();
    for (track, element) in document.elements_in_tracks() {
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let locate = |finding: Finding| {
            let finding = finding.at_file(document.path()).at_element(&subject);
            match track {
                Some(track) => finding.at_track(track),
                None => finding,
            }
        };

        for m in misplaced(element, vocab) {
            out.push(locate(
                Finding::new("E-AUDIO-EFFECT-WRONG-LIST")
                    .field("element", json!(subject))
                    .field("list", json!(m.list))
                    .field("index", json!(m.index))
                    .field("member", json!(m.name))
                    .field("right", json!(m.right)),
            ));
        }

        if !has_sound(element) {
            continue;
        }
        let mut first_enabled: Vec<(&str, usize)> = Vec::new();
        for (index, member) in members(element, "audio_effects").iter().enumerate() {
            let Some(name) = name_of(member).filter(|n| vocab.audio.iter().any(|a| a == n)) else {
                continue;
            };
            if !enabled(member) {
                out.push(locate(
                    Finding::new("R-AUDIO-EFFECT-DISABLED")
                        .field("element", json!(subject))
                        .field("index", json!(index))
                        .field("member", json!(name)),
                ));
                continue;
            }
            if !vocab.singular.iter().any(|s| s == name) {
                continue;
            }
            match first_enabled.iter().find(|(seen, _)| *seen == name) {
                Some((_, first)) => out.push(locate(
                    Finding::new("E-AUDIO-EFFECT-SINGULAR")
                        .field("element", json!(subject))
                        .field("member", json!(name))
                        .field("first", json!(first))
                        .field("index", json!(index)),
                )),
                None => first_enabled.push((name, index)),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A vocabulary with members, which the real one does not have yet: `gate` and `limit`
    /// are audio, `limit` is singular, `blur` is visual.
    fn vocab() -> Vocabulary {
        Vocabulary {
            audio: vec!["gate".into(), "limit".into()],
            visual: vec!["blur".into()],
            singular: vec!["limit".into()],
        }
    }

    fn project(element: Value) -> Loose {
        Loose::new(
            "p.montagent.json",
            json!({"tracks": [{"name": "t", "layer": 0, "elements": [element]}]}),
        )
    }

    fn audio(list: Value) -> Value {
        json!({"id": "bed", "type": "audio", "audio_effects": list})
    }

    fn codes(element: Value) -> Vec<(String, Value)> {
        findings(&project(element), &vocab())
            .into_iter()
            .map(|f| (f.code.clone(), json!(f.fields)))
            .collect()
    }

    #[test]
    fn the_real_vocabularies_are_the_schemas() {
        let published = Vocabulary::published();
        for name in ["compressor", "limiter"] {
            assert!(published.audio.iter().any(|a| a == name), "{name}");
        }
        assert!(published.visual.iter().any(|name| name == "blur"));
        assert!(published.singular.is_empty());
    }

    #[test]
    fn a_member_left_disabled_is_one_review_and_an_enabled_one_is_none() {
        let found = codes(audio(json!([
            {"name": "gate"},
            {"name": "gate", "enabled": false},
            {"name": "gate", "enabled": true},
        ])));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "R-AUDIO-EFFECT-DISABLED");
        assert_eq!(found[0].1["index"], 1);
        assert_eq!(found[0].1["member"], "gate");
    }

    #[test]
    fn a_disabled_member_is_reviewed_on_a_video_too() {
        let found = codes(json!({"id": "v", "type": "video",
                                 "audio_effects": [{"name": "gate", "enabled": false}]}));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "R-AUDIO-EFFECT-DISABLED");
    }

    #[test]
    fn a_second_enabled_singular_member_is_an_error_naming_both_positions() {
        let found = codes(audio(json!([
            {"name": "limit"}, {"name": "gate"}, {"name": "limit"}
        ])));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "E-AUDIO-EFFECT-SINGULAR");
        assert_eq!(found[0].1["first"], 0);
        assert_eq!(found[0].1["index"], 2);
    }

    #[test]
    fn singular_counts_enabled_members_only() {
        let found = codes(audio(json!([
            {"name": "limit", "enabled": false}, {"name": "limit"}
        ])));
        assert_eq!(
            found.iter().map(|f| f.0.as_str()).collect::<Vec<_>>(),
            ["R-AUDIO-EFFECT-DISABLED"],
            "{found:?}"
        );
    }

    #[test]
    fn duplicates_of_a_member_that_is_not_singular_are_ordinary() {
        assert!(codes(audio(json!([{"name": "gate"}, {"name": "gate"}]))).is_empty());
    }

    #[test]
    fn an_audio_member_in_effects_names_audio_effects() {
        let found = codes(json!({"id": "v", "type": "video",
                                 "effects": [{"name": "blur", "radius": 1}, {"name": "gate"}]}));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "E-AUDIO-EFFECT-WRONG-LIST");
        assert_eq!(found[0].1["list"], "effects");
        assert_eq!(found[0].1["right"], "audio_effects");
        assert_eq!(found[0].1["index"], 1);
        assert_eq!(found[0].1["member"], "gate");
    }

    #[test]
    fn a_visual_member_in_audio_effects_names_effects() {
        let found = codes(audio(json!([{"name": "blur", "radius": 1}])));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "E-AUDIO-EFFECT-WRONG-LIST");
        assert_eq!(found[0].1["list"], "audio_effects");
        assert_eq!(found[0].1["right"], "effects");
    }

    #[test]
    fn a_disabled_member_in_the_wrong_list_is_still_in_the_wrong_list() {
        let found = codes(audio(
            json!([{"name": "blur", "radius": 1, "enabled": false}]),
        ));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "E-AUDIO-EFFECT-WRONG-LIST");
    }

    #[test]
    fn a_name_in_neither_vocabulary_is_the_schemas_to_report() {
        assert!(codes(audio(json!([{"name": "nonsense"}]))).is_empty());
        assert!(
            codes(json!({"id": "r", "type": "rect", "effects": [{"name": "nonsense"}]})).is_empty()
        );
    }

    #[test]
    fn a_type_with_no_sound_has_no_audio_effects_to_check() {
        assert!(
            codes(json!({"id": "r", "type": "rect",
                         "audio_effects": [{"name": "gate", "enabled": false}]}))
            .is_empty()
        );
    }
}
