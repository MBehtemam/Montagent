//! The compressor's and the limiter's own rules (ADR-0180 section 3), decided from the file:
//!
//! - **`E-DYNAMICS-RANGE`** (`error`): a key outside its range. The ranges sit inside
//!   `acompressor`'s and `alimiter`'s own limits, so no accepted document fails at render.
//!   A disabled member is still validated.
//! - **`E-LIMITER-ABOVE-MASTER`** (`error`): `master.ceiling_dbtp` is written and a limiter's
//!   `ceiling_db` is above `ceiling_dbtp - 1`. Fires only with a master that states a ceiling.
//! - **`R-DYNAMICS-ORDER`** (`review`): an enabled compressor after an enabled limiter.
//! - **`R-COMPRESSOR-MAKEUP-CLIP`** (`review`): `makeup_db` above the static headroom of a
//!   full-scale sine, with no enabled limiter after it and no master ceiling.
//! - **`N-COMPRESSOR-RATIO-1`** (`note`): a compressor that is a plain gain.
//! - **`N-LIMITER-STACKED`** (`note`): more than one enabled limiter on one element.
//!
//! Bypassed members (`"enabled": false`) take no part in the ordering, clip, ratio and
//! stacking rules.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;

/// A numeric key and its inclusive range.
type Bound = (&'static str, f64, f64);

const COMPRESSOR: &[Bound] = &[
    ("threshold_db", -60.0, 0.0),
    ("ratio", 1.0, 20.0),
    ("attack_ms", 0.1, 2000.0),
    ("release_ms", 1.0, 9000.0),
    ("makeup_db", 0.0, 24.0),
];

const LIMITER: &[Bound] = &[("ceiling_db", -24.0, 0.0), ("release_ms", 1.0, 1000.0)];

/// The gap ADR-0174 allows between a sample-peak ceiling and the master's true-peak one.
const MASTER_GAP_DB: f64 = 1.0;

pub fn check(document: &Loose, report: &mut Report) {
    for finding in findings(document) {
        report.push(finding);
    }
}

fn enabled(member: &Value) -> bool {
    member.get("enabled") != Some(&Value::Bool(false))
}

fn name_of(member: &Value) -> Option<&str> {
    member.get("name").and_then(Value::as_str)
}

fn number(member: &Value, key: &str) -> Option<f64> {
    member
        .get(key)
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite())
}

/// `master.ceiling_dbtp`, where the document states one.
fn master_ceiling(document: &Loose) -> Option<f64> {
    document
        .value()
        .get("master")?
        .get("ceiling_dbtp")?
        .as_f64()
        .filter(|v| v.is_finite())
}

pub(crate) fn findings(document: &Loose) -> Vec<Finding> {
    let master = master_ceiling(document);
    let mut out = Vec::new();
    for (track, element) in document.elements_in_tracks() {
        if !matches!(
            element.get("type").and_then(Value::as_str),
            Some("audio" | "video")
        ) {
            continue;
        }
        let Some(members) = element.get("audio_effects").and_then(Value::as_array) else {
            continue;
        };
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
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

        let mut first_limiter: Option<usize> = None;
        let mut limiters: Vec<usize> = Vec::new();
        for (index, member) in members.iter().enumerate() {
            let Some(name) = name_of(member) else {
                continue;
            };
            let bounds = match name {
                "compressor" => COMPRESSOR,
                "limiter" => LIMITER,
                _ => continue,
            };
            for &(key, min, max) in bounds {
                let Some(value) = number(member, key).filter(|v| *v < min || *v > max) else {
                    continue;
                };
                let nearest = value.clamp(min, max);
                out.push(locate(
                    Finding::new("E-DYNAMICS-RANGE")
                        .field("index", json!(index))
                        .field("member", json!(name))
                        .field("key", json!(key))
                        .field("value", json!(value))
                        .field("min", json!(min))
                        .field("max", json!(max))
                        .field("nearest", json!(nearest))
                        .repair_value(json!({"value": format!("set `{key}` to {nearest}")})),
                ));
            }
            if name == "limiter"
                && let (Some(ceiling_dbtp), Some(ceiling)) = (master, number(member, "ceiling_db"))
                && ceiling > ceiling_dbtp - MASTER_GAP_DB
            {
                let limit = ceiling_dbtp - MASTER_GAP_DB;
                out.push(locate(
                    Finding::new("E-LIMITER-ABOVE-MASTER")
                        .field("index", json!(index))
                        .field("ceiling_db", json!(ceiling))
                        .field("ceiling_dbtp", json!(ceiling_dbtp))
                        .field("limit", json!(limit))
                        .repair_value(json!({"value": format!("set `ceiling_db` to {limit}")})),
                ));
            }
            if !enabled(member) {
                continue;
            }
            match name {
                "limiter" => {
                    first_limiter.get_or_insert(index);
                    limiters.push(index);
                }
                "compressor" => {
                    if let Some(limiter) = first_limiter {
                        out.push(locate(
                            Finding::new("R-DYNAMICS-ORDER")
                                .field("index", json!(index))
                                .field("limiter", json!(limiter)),
                        ));
                    }
                    if number(member, "ratio") == Some(1.0) {
                        out.push(locate(
                            Finding::new("N-COMPRESSOR-RATIO-1")
                                .field("index", json!(index))
                                .field("makeup_db", json!(number(member, "makeup_db"))),
                        ));
                    }
                    if master.is_none() {
                        let limited_after = members[index + 1..]
                            .iter()
                            .any(|m| name_of(m) == Some("limiter") && enabled(m));
                        if let (false, Some(makeup), Some(headroom)) =
                            (limited_after, number(member, "makeup_db"), headroom(member))
                            && makeup > headroom + 1e-9
                        {
                            out.push(locate(
                                Finding::new("R-COMPRESSOR-MAKEUP-CLIP")
                                    .field("index", json!(index))
                                    .field("makeup_db", json!(makeup))
                                    .field("headroom", json!(round3(headroom)))
                                    .field("threshold_db", json!(number(member, "threshold_db")))
                                    .field("ratio", json!(number(member, "ratio"))),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        if limiters.len() > 1 {
            out.push(locate(
                Finding::new("N-LIMITER-STACKED")
                    .field("count", json!(limiters.len()))
                    .field(
                        "indexes",
                        json!(
                            limiters
                                .iter()
                                .map(usize::to_string)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    ),
            ));
        }
    }
    out
}

/// What a full-scale sine can be lifted by before the compressed signal reaches 0 dBFS, in
/// RMS terms: `max(0, (-3 - threshold_db) * (1 - 1/ratio))` (ADR-0180 section 3). `None`
/// where a key is missing or `ratio` is not positive, which the schema and the range rule
/// report.
fn headroom(member: &Value) -> Option<f64> {
    let threshold = number(member, "threshold_db")?;
    let ratio = number(member, "ratio").filter(|r| *r >= 1.0)?;
    Some(((-3.0 - threshold) * (1.0 - 1.0 / ratio)).max(0.0))
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(master: Option<Value>, list: Value) -> Loose {
        let mut doc = json!({"tracks": [{"name": "t", "layer": 0, "elements": [
            {"id": "bed", "type": "audio", "audio_effects": list}]}]});
        if let Some(master) = master {
            doc["master"] = master;
        }
        Loose::new("p.montagent.json", doc)
    }

    fn limiter(ceiling: f64) -> Value {
        json!({"name": "limiter", "ceiling_db": ceiling, "release_ms": 50})
    }

    fn codes(doc: &Loose) -> Vec<String> {
        findings(doc).into_iter().map(|f| f.code).collect()
    }

    #[test]
    fn a_limiter_above_the_masters_ceiling_less_one_db_is_an_error_naming_that_value() {
        let doc = project(Some(json!({"ceiling_dbtp": -1.5})), json!([limiter(-2.0)]));
        let found = findings(&doc);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].code, "E-LIMITER-ABOVE-MASTER");
        assert_eq!(found[0].fields["limit"], -2.5);
    }

    #[test]
    fn a_limiter_at_or_under_the_limit_is_fine() {
        let doc = project(Some(json!({"ceiling_dbtp": -1.0})), json!([limiter(-2.0)]));
        assert!(codes(&doc).is_empty());
    }

    #[test]
    fn with_no_master_or_no_ceiling_it_cannot_be_wrong() {
        assert!(codes(&project(None, json!([limiter(0.0)]))).is_empty());
        let doc = project(Some(json!({"target_lufs": -16})), json!([limiter(0.0)]));
        assert!(codes(&doc).is_empty());
    }

    #[test]
    fn a_master_ceiling_silences_the_makeup_clip_review() {
        let loud = json!({"name": "compressor", "threshold_db": -24, "ratio": 3, "attack_ms": 20,
                          "release_ms": 250, "makeup_db": 20});
        let doc = project(None, json!([loud.clone()]));
        assert_eq!(codes(&doc), ["R-COMPRESSOR-MAKEUP-CLIP"]);
        let doc = project(Some(json!({"ceiling_dbtp": -1})), json!([loud]));
        assert!(codes(&doc).is_empty());
    }
}
