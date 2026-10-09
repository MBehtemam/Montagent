//! Loudness normalisation's own rules (ADR-0178 §4), as `validate` decides them:
//!
//! - **`E-NORMALIZE-TARGET-RANGE`** (`error`, advise): `target_lufs` outside -40..-6. The
//!   repair is the nearest bound. A disabled member is still validated.
//! - **`R-NORMALIZE-ABOVE-MASTER`** (`review`): `master.target_lufs` is written and an
//!   enabled member's `target_lufs` is above it, so the master's one gain pulls the element
//!   down again and the element was normalised to a level the deliverable never has.
//! - **`E-NORMALIZE-NO-AUDIO`** (`error`, refuse): the element's source carries no audio
//!   stream, so there is nothing to measure. The fix forks (drop the member, or point the
//!   element at a source with sound), so the repair is `"none"`, with a census of which
//!   normalised elements carry sound (ADR-0120, as `E-TRANSITION-AUDIO-NO-STREAM`). It needs
//!   the probe, so it is [`on_disk`]'s.
//!
//! The two render-time findings (`R-NORMALIZE-LIFT`, `N-NORMALIZE-UNDEFINED`) depend on the
//! measurement pass and are `render`'s and `preview`'s
//! (`crate::verbs::render::normalize`). A second enabled copy is ADR-0169's singular error
//! (`crate::checks::audio_effects::SINGULAR`).

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::{Census, Finding};
use crate::media::Source;
use crate::media::session::Session;
use crate::media::tools::Missing;
use crate::permissive::Loose;
use crate::report::Report;

/// The member's `name`.
pub const NAME: &str = "normalize_loudness";

/// The legal `target_lufs`, inclusive (ADR-0178 §1).
pub const QUIETEST_LUFS: f64 = -40.0;
pub const LOUDEST_LUFS: f64 = -6.0;

fn enabled(member: &Value) -> bool {
    member.get("enabled") != Some(&Value::Bool(false))
}

fn sounds(element: &Value) -> bool {
    matches!(
        element.get("type").and_then(Value::as_str),
        Some("audio" | "video")
    )
}

/// Every `normalize_loudness` member of `element`, with its index in `audio_effects`.
pub(crate) fn members(element: &Value) -> impl Iterator<Item = (usize, &Value)> {
    element
        .get("audio_effects")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .filter(|(_, member)| member.get("name").and_then(Value::as_str) == Some(NAME))
}

/// `master.target_lufs`, where the document writes one.
fn master_target(document: &Loose) -> Option<f64> {
    document
        .value()
        .get("master")?
        .get("target_lufs")?
        .as_f64()
        .filter(|v| v.is_finite())
}

pub fn check(document: &Loose, report: &mut Report) {
    for finding in findings(document) {
        report.push(finding);
    }
}

pub(crate) fn findings(document: &Loose) -> Vec<Finding> {
    let master = master_target(document);
    let mut out = Vec::new();
    for (track, element) in document.elements_in_tracks() {
        if !sounds(element) {
            continue;
        }
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        let locate = |finding: Finding| {
            let finding = finding
                .field("element", json!(subject))
                .at_file(document.path())
                .at_element(&subject);
            match track {
                Some(track) => finding.at_track(track),
                None => finding,
            }
        };
        for (index, member) in members(element) {
            // A non-number is the schema check's finding.
            let Some(value) = member.get("target_lufs").and_then(Value::as_f64) else {
                continue;
            };
            if !value.is_finite() || !(QUIETEST_LUFS..=LOUDEST_LUFS).contains(&value) {
                let nearest = if value.is_nan() {
                    LOUDEST_LUFS
                } else {
                    value.clamp(QUIETEST_LUFS, LOUDEST_LUFS)
                };
                out.push(locate(
                    Finding::new("E-NORMALIZE-TARGET-RANGE")
                        .field("index", json!(index))
                        .field("target_lufs", json!(value))
                        .field("min", json!(QUIETEST_LUFS))
                        .field("max", json!(LOUDEST_LUFS))
                        .field("nearest", json!(nearest))
                        .repair_value(json!({
                            "value": format!("set `target_lufs` to {nearest}")
                        })),
                ));
                continue;
            }
            if enabled(member)
                && let Some(master) = master
                && value > master
            {
                out.push(locate(
                    Finding::new("R-NORMALIZE-ABOVE-MASTER")
                        .field("index", json!(index))
                        .field("target_lufs", json!(value))
                        .field("master_lufs", json!(master)),
                ));
            }
        }
    }
    out
}

/// `E-NORMALIZE-NO-AUDIO` (ADR-0178 §4): off the probes the source check has already cached.
pub fn on_disk(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    let base = crate::checks::project_dir(document);
    // Every normalised element, whether its source is known to carry sound (`None` where
    // nothing was established, which the source check reports on its own).
    let mut normalised = Vec::new();
    for (track, element) in document.elements_in_tracks() {
        if !sounds(element) {
            continue;
        }
        let Some(index) = members(element).map(|(index, _)| index).next() else {
            continue;
        };
        let Some(source) = element.get("source").and_then(Value::as_str) else {
            continue;
        };
        let outcome = session.probe(&Source::resolve(source, &base))?;
        let carries = outcome.probe().map(|probe| probe.audio.is_some());
        let subject = subject_of(element.get("id").and_then(Value::as_str));
        normalised.push((track, subject, index, source.to_string(), carries));
    }

    let mut census = Census::on("audio_stream");
    for carries in [false, true] {
        let members: Vec<&str> = normalised
            .iter()
            .filter(|(.., known)| *known == Some(carries))
            .map(|(_, subject, ..)| subject.as_str())
            .collect();
        if !members.is_empty() {
            census = census.group(json!(carries), members);
        }
    }

    for (track, subject, index, source, carries) in &normalised {
        if *carries != Some(false) {
            continue;
        }
        let mut finding = Finding::new("E-NORMALIZE-NO-AUDIO")
            .at_file(document.path())
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("index", json!(index))
            .field("source", json!(source))
            .census(census.clone());
        if let Some(track) = track {
            finding = finding.at_track(*track);
        }
        report.push(finding);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(element: Value, master: Option<Value>) -> Vec<(String, Value)> {
        let mut body = json!({"tracks": [{"name": "t", "layer": 0, "elements": [element]}]});
        if let Some(master) = master {
            body["master"] = master;
        }
        findings(&Loose::new("p.montagent.json", body))
            .into_iter()
            .map(|f| (f.code.clone(), json!(f.fields)))
            .collect()
    }

    fn audio(list: Value) -> Value {
        json!({"id": "bed", "type": "audio", "audio_effects": list})
    }

    #[test]
    fn a_non_finite_or_missing_target_is_left_to_the_schema() {
        assert!(codes(audio(json!([{"name": NAME}])), None).is_empty());
        assert!(codes(audio(json!([{"name": NAME, "target_lufs": "loud"}])), None).is_empty());
    }

    #[test]
    fn an_element_with_no_sound_is_not_this_checks() {
        let rect = json!({"id": "r", "type": "rect",
                          "audio_effects": [{"name": NAME, "target_lufs": 0}]});
        assert!(codes(rect, None).is_empty());
    }

    #[test]
    fn an_out_of_range_target_is_not_also_compared_with_the_master() {
        let found = codes(
            audio(json!([{"name": NAME, "target_lufs": -2}])),
            Some(json!({"target_lufs": -16})),
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "E-NORMALIZE-TARGET-RANGE");
    }

    #[test]
    fn a_disabled_member_is_not_compared_with_the_master() {
        let found = codes(
            audio(json!([{"name": NAME, "target_lufs": -10, "enabled": false}])),
            Some(json!({"target_lufs": -16})),
        );
        assert!(found.is_empty(), "{found:?}");
    }
}
