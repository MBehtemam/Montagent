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
use crate::media::Source;
use crate::media::probe::Outcome;
use crate::media::session::Session;
use crate::permissive::Loose;
use crate::report::Report;

/// The audio members that ADR-0169 lets appear once among the enabled ones: loudness
/// normalisation (ADR-0178 §1), whose one gain a second copy could only contradict, and
/// `reduce_noise` (ADR-0181 §1), whose calibrated strength does not compose across two passes.
pub const SINGULAR: &[&str] = &["normalize_loudness", "reduce_noise"];

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
    for finding in member_and_routing_findings(document) {
        report.push(finding);
    }
}

/// Disk-backed check for `channels` on a probed 1-channel source (`R-CHANNELS-ON-MONO-SOURCE`,
/// ADR-0184 §5).
pub fn check_disk(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<crate::media::tools::Missing>> {
    let base = crate::checks::project_dir(document);
    for (track, element) in document.elements_in_tracks() {
        if !has_sound(element) {
            continue;
        }
        let Some(channels) = element
            .get("channels")
            .and_then(Value::as_str)
            .filter(|c| matches!(*c, "swap" | "mono" | "left" | "right"))
        else {
            continue;
        };
        let Some(source) = element.get("source").and_then(Value::as_str) else {
            continue;
        };
        let Outcome::Probed(probe) = session.probe(&Source::resolve(source, &base))? else {
            continue;
        };
        if probe.audio.as_ref().and_then(|a| a.channels) == Some(1) {
            let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
            let finding = Finding::new("R-CHANNELS-ON-MONO-SOURCE")
                .at_file(document.path())
                .at_element(&subject)
                .field("element", json!(subject))
                .field("channels", json!(channels))
                .field("source", json!(source));
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        }
    }
    Ok(())
}

fn trim_num(value: f64) -> String {
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Document checks for `pan` and `channels` (ADR-0184), `reduce_noise` (ADR-0181), and `echo`
/// (ADR-0185).
fn member_and_routing_findings(document: &Loose) -> Vec<Finding> {
    let mut out = Vec::new();
    for (track, element) in document.elements_in_tracks() {
        if !has_sound(element) {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let locate = |finding: Finding| {
            let finding = finding.at_file(document.path()).at_element(&subject);
            match track {
                Some(track) => finding.at_track(track),
                None => finding,
            }
        };

        // ADR-0184 §5: `pan` range (-1..=1) and inert default (`pan: 0`).
        if let Some(pan) = element
            .get("pan")
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite())
        {
            if !(-1.0..=1.0).contains(&pan) {
                let clamped = pan.clamp(-1.0, 1.0);
                out.push(locate(
                    Finding::new("E-PAN-RANGE")
                        .field("element", json!(subject))
                        .field("pan", json!(trim_num(pan)))
                        .field("min", json!(-1))
                        .field("max", json!(1))
                        .field("nearest", json!(clamped))
                        .repair_value(json!({"key": "pan", "value": clamped})),
                ));
            } else if pan == 0.0 {
                out.push(locate(
                    Finding::new("R-PAN-INERT").field("element", json!(subject)),
                ));
            }
        }

        // ADR-0184 §5: `channels: "stereo"` inert default.
        if element.get("channels").and_then(Value::as_str) == Some("stereo") {
            out.push(locate(
                Finding::new("R-CHANNELS-INERT").field("element", json!(subject)),
            ));
        }

        let mut first_enabled_compressor: Option<usize> = None;
        let mut echo_peak_product = 1.0_f64;
        let mut has_enabled_echo = false;

        for (index, member) in members(element, "audio_effects").iter().enumerate() {
            let Some(name) = name_of(member) else {
                continue;
            };
            let is_enabled = enabled(member);
            match name {
                "compressor" if is_enabled => {
                    first_enabled_compressor.get_or_insert(index);
                }
                "reduce_noise" => {
                    if let Some(reduction_db) = member
                        .get("reduction_db")
                        .and_then(Value::as_f64)
                        .filter(|v| v.is_finite())
                        && !(5.0..=20.0).contains(&reduction_db)
                    {
                        let clamped = reduction_db.clamp(5.0, 20.0);
                        out.push(locate(
                            Finding::new("E-NOISE-RANGE")
                                .field("element", json!(subject))
                                .field("index", json!(index))
                                .field(
                                    "reduction_db",
                                    json!(format!("{} dB", trim_num(reduction_db))),
                                )
                                .field("min", json!("5 dB"))
                                .field("max", json!("20 dB"))
                                .field("nearest", json!(format!("{} dB", trim_num(clamped))))
                                .repair_value(json!({"key": "reduction_db", "value": clamped})),
                        ));
                    }
                    if is_enabled && let Some(comp_idx) = first_enabled_compressor {
                        out.push(locate(
                            Finding::new("R-NOISE-AFTER-COMPRESSOR")
                                .field("element", json!(subject))
                                .field("index", json!(index))
                                .field("compressor", json!(comp_idx)),
                        ));
                    }
                }
                "echo" => {
                    let mut in_range = true;
                    for (key, min, max, unit) in [
                        ("delay_ms", 1.0, 2000.0, " ms"),
                        ("feedback", 0.0, 0.95, ""),
                        ("mix", 0.0, 1.0, ""),
                    ] {
                        let Some(val) = member
                            .get(key)
                            .and_then(Value::as_f64)
                            .filter(|v| v.is_finite())
                        else {
                            in_range = false;
                            continue;
                        };
                        if !(min..=max).contains(&val) {
                            in_range = false;
                            let clamped = val.clamp(min, max);
                            out.push(locate(
                                Finding::new("E-ECHO-RANGE")
                                    .field("element", json!(subject))
                                    .field("index", json!(index))
                                    .field("key", json!(key))
                                    .field("value", json!(format!("{}{unit}", trim_num(val))))
                                    .field("min", json!(format!("{}{unit}", trim_num(min))))
                                    .field("max", json!(format!("{}{unit}", trim_num(max))))
                                    .field("nearest", json!(format!("{}{unit}", trim_num(clamped))))
                                    .repair_value(json!({"key": key, "value": clamped})),
                            ));
                        }
                    }
                    if is_enabled
                        && in_range
                        && let (Some(feedback), Some(mix)) = (
                            member.get("feedback").and_then(Value::as_f64),
                            member.get("mix").and_then(Value::as_f64),
                        )
                    {
                        if mix < 0.001 {
                            out.push(locate(
                                Finding::new("N-ECHO-NO-OP")
                                    .field("element", json!(subject))
                                    .field("index", json!(index))
                                    .field("mix", json!(trim_num(mix))),
                            ));
                        } else {
                            let mut tap_sum = 0.0_f64;
                            for k in 1..=8_i32 {
                                let g = mix * feedback.powi(k - 1);
                                if g < 0.001 {
                                    break;
                                }
                                tap_sum += g;
                            }
                            echo_peak_product *= 1.0 + tap_sum;
                            has_enabled_echo = true;

                            let eighth = mix * feedback.powi(7);
                            if eighth > 0.01 {
                                out.push(locate(
                                    Finding::new("N-ECHO-TAPS-CAPPED")
                                        .field("element", json!(subject))
                                        .field("index", json!(index))
                                        .field("eighth_tap_gain", json!(trim_num(eighth))),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if has_enabled_echo && echo_peak_product > 3.0 {
            let rounded = (echo_peak_product * 100.0).round() / 100.0;
            out.push(locate(
                Finding::new("R-ECHO-HOT")
                    .field("element", json!(subject))
                    .field("gain", json!(trim_num(rounded))),
            ));
        }
    }
    out
}

pub(crate) fn findings(document: &Loose, vocab: &Vocabulary) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in misplaced(document.value(), vocab) {
        out.push(
            Finding::new("E-AUDIO-EFFECT-WRONG-LIST")
                .at_file(document.path())
                .field("element", json!("project"))
                .field("list", json!(m.list))
                .field("index", json!(m.index))
                .field("member", json!(m.name))
                .field("right", json!(m.right)),
        );
    }
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
        for name in [
            "highpass",
            "lowpass",
            "shelf",
            "bell",
            "compressor",
            "limiter",
            "normalize_loudness",
            "reduce_noise",
            "echo",
        ] {
            assert!(published.audio.iter().any(|a| a == name), "{name}");
        }
        assert!(published.visual.iter().any(|name| name == "blur"));
        assert_eq!(published.singular, ["normalize_loudness", "reduce_noise"]);
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
