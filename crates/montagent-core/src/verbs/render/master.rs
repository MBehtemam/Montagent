//! The master stage on the mix bus (ADR-0172, ADR-0174).
//!
//! It runs after `amix` and before the closing `apad`/`atrim`:
//!
//! 1. **Pass 1, measure** (only when `target_lufs` is set): the whole programme's mix, audio
//!    only, through [`crate::media::loudness`], with no encoder and no master stage.
//! 2. **Pass 2, apply**: one fixed `volume=<target − measured>dB`, then, where `ceiling_dbtp`
//!    is set, the closing limiter at four times the rate with `level=0`:
//!    `aresample=192000,alimiter=limit=<ceiling as linear>:latency=1:level=0,aresample=48000`.
//!
//! **Partial renders and `preview` use the whole programme's gain**, because [`settle`]
//! always measures `[0, extent)`, whatever span is being encoded: a span sounds as it does in
//! the deliverable. **One gain, no compensation**: the gain is computed before the limiter and
//! never re-measured. **Undefined loudness gets no gain** and a finding; any defined loudness
//! gets its full gain, however large, with a finding above +20 dB. `loudnorm` is never used:
//! it falls back to a dynamic mode whose output varies with the CPU.
//!
//! With no `master`, or `master: {}`, [`Stage::bus`] is empty and the graph is today's,
//! byte for byte.

use std::path::Path as FilePath;

use serde::Serialize;
use serde_json::json;

use crate::finding::{Citation, Finding};
use crate::media::established::Established;
use crate::permissive::Loose;

/// The rate the closing limiter runs at: four times the mix bus's (ADR-0174 §2).
pub const LIMITER_RATE: i64 = 4 * super::MIX_RATE;

/// Above this applied gain the noise floor rises with it (ADR-0172's court).
pub const GAIN_HIGH_DB: f64 = 20.0;

pub const GAIN_SOURCE: &str = "A gain above +20 dB raises the noise floor audibly (ADR-0172's \
court; +12 dB would fire on ordinary quiet TTS mixes)";

/// What the master stage does to this render's bus, settled before any span is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Stage {
    pub target_lufs: Option<f64>,
    pub ceiling_dbtp: Option<f64>,
    /// Pass 1's integrated loudness of the whole programme, where it is defined.
    pub measured_lufs: Option<f64>,
    /// The one gain pass 2 applies, where there is one.
    pub gain_db: Option<f64>,
}

/// What `render` reports about the gain, which the file does not carry (ADR-0172): present
/// whenever `target_lufs` is set.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Applied {
    /// The whole programme's integrated loudness before the gain, in LUFS; `null` where every
    /// block was gated out and so no gain was applied.
    pub measured_lufs: Option<f64>,
    /// The gain applied, in dB: `target_lufs − measured_lufs`, or `0` where none was.
    pub applied_gain_db: f64,
}

impl Stage {
    /// The filters between the summed mix and the closing `apad`, each followed by `,`; empty
    /// where the stage does nothing.
    pub fn bus(&self) -> String {
        let mut bus = String::new();
        if let Some(gain) = self.gain_db {
            bus.push_str(&format!("volume={gain}dB,"));
        }
        if let Some(ceiling) = self.ceiling_dbtp {
            bus.push_str(&format!(
                "aresample={LIMITER_RATE},alimiter=limit={:.8}:latency=1:level=0,aresample={},",
                linear(ceiling),
                super::MIX_RATE
            ));
        }
        bus
    }

    pub fn applied(&self) -> Option<Applied> {
        self.target_lufs.map(|_| Applied {
            measured_lufs: self.measured_lufs,
            applied_gain_db: self.gain_db.unwrap_or(0.0),
        })
    }
}

/// A dB figure as the linear amplitude `alimiter`'s `limit` takes (ADR-0170: the document's
/// unit never follows a filter's).
pub fn linear(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}

/// Settle the master stage for one invocation: read `master`, and where it sets a target,
/// measure the whole programme. The findings are the measurement's, for the caller's report.
pub(crate) fn settle(
    document: &Loose,
    project_dir: &FilePath,
    established: &Established,
    fps: i64,
    ffmpeg: &FilePath,
) -> Result<(Stage, Vec<Finding>), String> {
    let Some(master) = crate::checks::master::of(document) else {
        return Ok((Stage::default(), Vec::new()));
    };
    let mut stage = Stage {
        target_lufs: master.target_lufs.map(|t| t.0),
        ceiling_dbtp: master.ceiling_dbtp.map(|c| c.0),
        ..Stage::default()
    };
    let Some(target) = stage.target_lufs else {
        return Ok((stage, Vec::new()));
    };
    let at = |code: &str| Finding::new(code).at_file(document.path());

    let measured = match crate::exact::extent(document) {
        Some(end) => {
            let mix = super::Mix::of(document, project_dir, established, fps, 0, end, "");
            match mix.audio {
                Some(audio) => {
                    crate::media::loudness::of_graph(ffmpeg, &audio.inputs, &audio.graph)?
                        .integrated
                }
                None => None,
            }
        }
        None => None,
    };
    let mut findings = Vec::new();
    match measured {
        None => {
            findings.push(at("R-MASTER-LOUDNESS-UNDEFINED").field("target_lufs", json!(target)))
        }
        Some(measured) => {
            let gain = round2(target - measured);
            stage.measured_lufs = Some(measured);
            stage.gain_db = Some(gain);
            if gain > GAIN_HIGH_DB {
                findings.push(
                    at("R-MASTER-GAIN-HIGH")
                        .field("target_lufs", json!(target))
                        .field("measured_lufs", json!(measured))
                        .field("applied_gain_db", json!(gain))
                        .field("threshold_db", json!(GAIN_HIGH_DB))
                        .citation(Citation {
                            threshold: json!(GAIN_HIGH_DB),
                            source: GAIN_SOURCE.into(),
                            adr: "ADR-0172".into(),
                        }),
                );
            }
        }
    }
    Ok((stage, findings))
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_master_writes_no_bus() {
        assert_eq!(Stage::default().bus(), "");
        assert_eq!(Stage::default().applied(), None);
    }

    /// ADR-0174 §2's recipe, as a committed golden.
    #[test]
    fn the_bus_is_one_gain_then_the_4x_limiter_with_level_off() {
        let stage = Stage {
            target_lufs: Some(-16.0),
            ceiling_dbtp: Some(-1.0),
            measured_lufs: Some(-23.4),
            gain_db: Some(7.4),
        };
        assert_eq!(
            stage.bus(),
            "volume=7.4dB,aresample=192000,alimiter=limit=0.89125094:latency=1:level=0,\
             aresample=48000,"
        );
        assert_eq!(
            stage.applied(),
            Some(Applied {
                measured_lufs: Some(-23.4),
                applied_gain_db: 7.4
            })
        );
    }

    #[test]
    fn a_ceiling_alone_is_the_limiter_alone_and_an_undefined_target_no_gain() {
        let ceiling = Stage {
            ceiling_dbtp: Some(-2.0),
            ..Stage::default()
        };
        assert_eq!(
            ceiling.bus(),
            "aresample=192000,alimiter=limit=0.79432823:latency=1:level=0,aresample=48000,"
        );
        assert_eq!(ceiling.applied(), None);
        let undefined = Stage {
            target_lufs: Some(-16.0),
            ..Stage::default()
        };
        assert_eq!(undefined.bus(), "");
        assert_eq!(
            undefined.applied(),
            Some(Applied {
                measured_lufs: None,
                applied_gain_db: 0.0
            })
        );
    }
}
