//! The five EQ findings (ADR-0179 §3), decided from the file:
//!
//! - **`E-EQ-RANGE`** (`error`): `frequency_hz`, `gain_db` or `q` outside its range, or a
//!   `slope_db_per_oct` not in {12, 24, 48}. One finding per offending key, repaired with the
//!   nearest bound or the nearest slope. A bypassed member is still checked: `"enabled":
//!   false` skips the render, not the range.
//! - **`E-EQ-STACK-CAP`** (`error`): more than [`CAP`] enabled EQ members on one element.
//!   Refuse-class with a census of the members by name: the fix forks.
//! - **`R-EQ-GAIN-EXTREME`** (`review`): an enabled `shelf` or `bell` with `abs(gain_db)`
//!   above [`GAIN_REVIEW_DB`].
//! - **`R-EQ-BAND-CROSSED`** (`review`): an enabled `highpass` at or above an enabled
//!   `lowpass` on one element, which leaves nothing in the passband.
//! - **`N-EQ-NO-OP`** (`note`): an enabled `shelf` or `bell` at `gain_db` 0.
//!
//! Values are read leniently off the file: a missing or non-numeric key is the schema's to
//! report, so it is skipped here.

use serde_json::{Value, json};

use crate::finding::{Census, Finding};
use crate::permissive::Loose;
use crate::report::Report;

/// The most enabled EQ members one element may carry (Premiere's band count).
pub const CAP: usize = 8;
/// A gain whose magnitude is above this is reviewed.
pub const GAIN_REVIEW_DB: f64 = 12.0;

const NAMES: [&str; 4] = ["highpass", "lowpass", "shelf", "bell"];
const SLOPES: [f64; 3] = [12.0, 24.0, 48.0];
const FREQUENCY: (f64, f64) = (20.0, 20000.0);
const GAIN: (f64, f64) = (-24.0, 24.0);
const Q: (f64, f64) = (0.1, 10.0);

pub fn check(document: &Loose, report: &mut Report) {
    for finding in findings(document) {
        report.push(finding);
    }
}

fn number(member: &Value, key: &str) -> Option<f64> {
    member.get(key).and_then(Value::as_f64)
}

fn enabled(member: &Value) -> bool {
    member.get("enabled") != Some(&Value::Bool(false))
}

fn nearest_slope(value: f64) -> f64 {
    SLOPES
        .iter()
        .copied()
        .min_by(|a, b| (a - value).abs().total_cmp(&(b - value).abs()))
        .unwrap_or(SLOPES[0])
}

pub(crate) fn findings(document: &Loose) -> Vec<Finding> {
    let mut out = Vec::new();
    for (track, element) in document.elements_in_tracks() {
        if !matches!(
            element.get("type").and_then(Value::as_str),
            Some("audio" | "video")
        ) {
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
        let list = element
            .get("audio_effects")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();

        let mut live: Vec<(usize, &str)> = Vec::new();
        let mut highpasses: Vec<(usize, f64)> = Vec::new();
        let mut lowpasses: Vec<(usize, f64)> = Vec::new();

        for (index, member) in list.iter().enumerate() {
            let Some(name) = member
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| NAMES.contains(n))
            else {
                continue;
            };

            let mut range = |key: &str, bounds: (f64, f64)| {
                let Some(value) = number(member, key) else {
                    return;
                };
                if value >= bounds.0 && value <= bounds.1 {
                    return;
                }
                out.push(locate(
                    Finding::new("E-EQ-RANGE")
                        .field("element", json!(subject))
                        .field("index", json!(index))
                        .field("member", json!(name))
                        .field("key", json!(key))
                        .field("value", json!(value))
                        .field("allowed", json!(format!("{} to {}", bounds.0, bounds.1)))
                        .repair_value(json!({ key: value.clamp(bounds.0, bounds.1) })),
                ));
            };
            range("frequency_hz", FREQUENCY);
            if matches!(name, "shelf" | "bell") {
                range("gain_db", GAIN);
            }
            if name == "bell" {
                range("q", Q);
            }
            if matches!(name, "highpass" | "lowpass")
                && let Some(slope) = number(member, "slope_db_per_oct")
                && !SLOPES.contains(&slope)
            {
                out.push(locate(
                    Finding::new("E-EQ-RANGE")
                        .field("element", json!(subject))
                        .field("index", json!(index))
                        .field("member", json!(name))
                        .field("key", json!("slope_db_per_oct"))
                        .field("value", json!(slope))
                        .field("allowed", json!("12, 24 or 48"))
                        .repair_value(json!({"slope_db_per_oct": nearest_slope(slope)})),
                ));
            }

            if !enabled(member) {
                continue;
            }
            live.push((index, name));
            match name {
                "highpass" => highpasses.extend(number(member, "frequency_hz").map(|f| (index, f))),
                "lowpass" => lowpasses.extend(number(member, "frequency_hz").map(|f| (index, f))),
                _ => {
                    let Some(gain) = number(member, "gain_db") else {
                        continue;
                    };
                    if gain == 0.0 {
                        out.push(locate(
                            Finding::new("N-EQ-NO-OP")
                                .field("element", json!(subject))
                                .field("index", json!(index))
                                .field("member", json!(name)),
                        ));
                    } else if gain.abs() > GAIN_REVIEW_DB {
                        out.push(locate(
                            Finding::new("R-EQ-GAIN-EXTREME")
                                .field("element", json!(subject))
                                .field("index", json!(index))
                                .field("member", json!(name))
                                .field("gain_db", json!(gain))
                                .field("threshold_db", json!(GAIN_REVIEW_DB)),
                        ));
                    }
                }
            }
        }

        for &(h, high) in &highpasses {
            for &(l, low) in &lowpasses {
                if high >= low {
                    out.push(locate(
                        Finding::new("R-EQ-BAND-CROSSED")
                            .field("element", json!(subject))
                            .field("highpass", json!(h))
                            .field("highpass_hz", json!(high))
                            .field("lowpass", json!(l))
                            .field("lowpass_hz", json!(low)),
                    ));
                }
            }
        }

        if live.len() > CAP {
            let mut census = Census::on("name");
            for name in NAMES {
                let members: Vec<String> = live
                    .iter()
                    .filter(|(_, n)| *n == name)
                    .map(|(i, _)| format!("audio_effects[{i}]"))
                    .collect();
                if !members.is_empty() {
                    census = census.group(json!(name), members);
                }
            }
            out.push(locate(
                Finding::new("E-EQ-STACK-CAP")
                    .field("element", json!(subject))
                    .field("count", json!(live.len()))
                    .field("cap", json!(CAP))
                    .census(census),
            ));
        }
    }
    out
}
