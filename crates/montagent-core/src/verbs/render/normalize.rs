//! Per-element loudness normalisation (ADR-0178 §3): one measured, fixed gain per enabled
//! `normalize_loudness` member.
//!
//! 1. **Measure** ([`settle`]), once per invocation and before any span is encoded: for each
//!    enabled member, the element's **placed window** — its whole `start..end`, after `atrim`,
//!    `atempo`, `aloop` and every enabled member ahead of this one in the list — through the
//!    same meter the master stage uses ([`crate::media::loudness::of_graph`]). The graph
//!    measured is the element's own chain, built by the same functions the mix is, cut at the
//!    member's position.
//! 2. **Apply**: the member lowers to `volume=<target − measured>dB`, held for the whole
//!    element, at its own position in the list.
//!
//! **Partial renders and `preview` use the whole window's gain**, because the measurement is
//! always of the element's full window, whatever span is encoded. **Undefined loudness gets
//! no gain**: silence, or a window under one 400 ms block, reads as the meter's −70 floor, so
//! the member writes nothing into the graph and `N-NORMALIZE-UNDEFINED` says so. A defined
//! loudness always gets its full gain, with `R-NORMALIZE-LIFT` above +20 dB (ADR-0172's
//! figure, [`super::master::GAIN_HIGH_DB`]). `loudnorm` is never used.
//!
//! An element with no enabled member is never measured, so its chain is unchanged byte for
//! byte (ADR-0173 §5).

use std::collections::BTreeMap;
use std::path::Path as FilePath;

use serde::Serialize;
use serde_json::{Value, json};

use crate::checks::normalize::{NAME, members};
use crate::finding::{Citation, Finding};
use crate::media::established::{Established, Use};
use crate::permissive::Loose;

/// The gains settled for one element: by index in its `audio_effects`, the gain in dB, or
/// `None` where the loudness was undefined and no gain is applied.
pub(crate) type ElementGains = BTreeMap<usize, Option<f64>>;

/// What the measurement pass settled for one invocation.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Gains {
    by_element: BTreeMap<String, ElementGains>,
    applied: Vec<Normalized>,
}

/// What `render` reports about one member: the gain is in no file, so this is where it stays
/// visible (ADR-0178 Consequences).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Normalized {
    pub element: String,
    /// The member's index in the element's `audio_effects`.
    pub index: usize,
    pub target_lufs: f64,
    /// The placed window's integrated loudness at the member's position, in LUFS; `null`
    /// where it was undefined and so no gain was applied.
    pub measured_lufs: Option<f64>,
    /// The gain applied, in dB: `target_lufs − measured_lufs`, or `0` where none was.
    pub applied_gain_db: f64,
}

impl Gains {
    /// The settled gains of the element the mix names `element`, where it has any.
    pub fn of(&self, element: &str) -> Option<&ElementGains> {
        self.by_element.get(element)
    }

    /// Every enabled member's measurement, in document order.
    pub fn applied(&self) -> Vec<Normalized> {
        self.applied.clone()
    }
}

fn enabled(member: &Value) -> bool {
    member.get("enabled") != Some(&Value::Bool(false))
}

/// Whether any audible element carries an enabled member, so the pass needs an `ffmpeg`.
pub(crate) fn wanted(document: &Loose) -> bool {
    document.elements().any(|element| {
        Use::of(element.get("type").and_then(Value::as_str)).contains(&Use::Mix)
            && members(element).any(|(_, member)| enabled(member))
    })
}

/// Measure every enabled member's placed window and settle its gain. The findings are the
/// measurement's, for the caller's report.
pub(crate) fn settle(
    document: &Loose,
    project_dir: &FilePath,
    established: &Established,
    ffmpeg: &FilePath,
) -> Result<(Gains, Vec<Finding>), String> {
    let mut gains = Gains::default();
    let mut findings = Vec::new();
    for (position, element) in document.elements().enumerate() {
        let kind = element.get("type").and_then(Value::as_str);
        if !Use::of(kind).contains(&Use::Mix) {
            continue;
        }
        let wanted: Vec<(usize, f64)> = members(element)
            .filter(|(_, member)| enabled(member))
            .filter_map(|(index, member)| {
                Some((index, member.get("target_lufs").and_then(Value::as_f64)?))
            })
            .collect();
        if wanted.is_empty() {
            continue;
        }
        let name = super::element_name(element, position);
        // The element's whole placed window. Anything that keeps it out of the mix (a remap,
        // a source the check engine did not establish) is the mix's own finding, and leaves
        // nothing here to measure.
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        if end <= start || crate::remap::is_remapped(element) {
            continue;
        }
        let Ok((path, placed)) = super::placed(
            element,
            &name,
            kind,
            project_dir,
            established,
            1,
            start,
            end,
        ) else {
            continue;
        };
        let mut settled = ElementGains::new();
        for (index, target) in wanted {
            let Ok(stage) = super::audio_effects_stage_with(
                element,
                &name,
                &super::lower_audio_member,
                Some(&settled),
                Some(index),
            ) else {
                break;
            };
            let graph = format!("{placed}{stage}[mix]");
            let measured =
                crate::media::loudness::of_graph(ffmpeg, std::slice::from_ref(&path), &graph)?
                    .integrated;
            let at = |code: &str| {
                Finding::new(code)
                    .at_file(document.path())
                    .at_element(name.clone())
                    .field("element", json!(name))
                    .field("index", json!(index))
                    .field("target_lufs", json!(target))
            };
            let gain = measured.map(|measured| super::master::round2(target - measured));
            match (measured, gain) {
                (Some(measured), Some(gain)) if gain > super::master::GAIN_HIGH_DB => findings
                    .push(
                        at("R-NORMALIZE-LIFT")
                            .field("measured_lufs", json!(measured))
                            .field("applied_gain_db", json!(gain))
                            .field("threshold_db", json!(super::master::GAIN_HIGH_DB))
                            .citation(Citation {
                                threshold: json!(super::master::GAIN_HIGH_DB),
                                source: super::master::GAIN_SOURCE.into(),
                                adr: "ADR-0172".into(),
                            }),
                    ),
                (None, _) => findings.push(at("N-NORMALIZE-UNDEFINED")),
                _ => {}
            }
            settled.insert(index, gain);
            gains.applied.push(Normalized {
                element: name.clone(),
                index,
                target_lufs: target,
                measured_lufs: measured,
                applied_gain_db: gain.unwrap_or(0.0),
            });
        }
        gains.by_element.insert(name, settled);
    }
    Ok((gains, findings))
}

/// How a settled member is spelled in the chain: `volume=<gain>dB`, or nothing at all where
/// the loudness was undefined. `Err` where the member was never measured, which is the
/// measurement pass and the mix disagreeing.
pub(crate) fn lowered(gains: Option<&ElementGains>, index: usize) -> Result<Option<String>, ()> {
    match gains.and_then(|gains| gains.get(&index)) {
        Some(Some(gain)) => Ok(Some(format!("volume={gain}dB"))),
        Some(None) => Ok(None),
        None => Err(()),
    }
}

/// Whether `name` is this member's.
pub(crate) fn is_member(name: &str) -> bool {
    name == NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_settled_gain_is_one_volume_in_db_and_an_undefined_one_is_nothing() {
        let gains = ElementGains::from([(0, Some(-4.35)), (2, None)]);
        assert_eq!(
            lowered(Some(&gains), 0),
            Ok(Some("volume=-4.35dB".to_string()))
        );
        assert_eq!(lowered(Some(&gains), 2), Ok(None));
        assert_eq!(lowered(Some(&gains), 1), Err(()));
        assert_eq!(lowered(None, 0), Err(()));
    }
}
