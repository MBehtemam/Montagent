//! The master stage's document-level reviews (ADR-0172 §Checks), decided from the file alone.
//!
//! - **`R-MASTER-NO-CEILING`**: `target_lufs` without `ceiling_dbtp`. A positive gain with no
//!   limiter can clip, and `validate` cannot know the gain's sign before the measurement pass,
//!   so it is a `review` naming `ceiling_dbtp: -1` as the fix, never an error.
//! - **`R-MASTER-TARGET-UNUSUAL`**: a target louder than −9 LUFS or quieter than −31 LUFS.
//! - **`R-MASTER-CEILING-HIGH`**: a ceiling above −1 dBTP on a lossy AAC deliverable.
//! - **`R-MASTER-HEADROOM`**: `ceiling_dbtp − target_lufs` under 6 dB, which predicts heavy
//!   limiting and a delivered loudness below target before anything renders.
//!
//! The last three decide on borrowed numbers, so each is `review` and carries its citation
//! inline (ADR-0061). The ranges themselves are schema errors ([`crate::model::master`]); a
//! `master` that does not parse is the schema check's to report, and this one stays silent.

use serde_json::json;

use crate::finding::{Citation, Finding};
use crate::model::Master;
use crate::permissive::Loose;
use crate::report::Report;

/// Louder than this is brickwalled territory (ADR-0172's court).
pub const LOUDEST_USUAL_LUFS: f64 = -9.0;
/// Quieter than this is below every broadcast norm (ADR-0172's court).
pub const QUIETEST_USUAL_LUFS: f64 = -31.0;
/// Lossy-delivery guidance: a true-peak ceiling of −1 dBTP or lower.
pub const LOSSY_CEILING_DBTP: f64 = -1.0;
/// The least peak-to-loudness headroom mastered material is usually left with.
pub const MIN_HEADROOM_DB: f64 = 6.0;

pub const TARGET_SOURCE: &str = "Loudness delivery practice as ADR-0172's court read it: \
broadcast −23/−24, streaming −14/−16, podcast −16/−19 and cinema down to −31 LUFS; louder than \
−9 LUFS is brickwalled";
pub const CEILING_SOURCE: &str = "Lossy-delivery true-peak guidance: −1 dBTP or lower for an \
AAC deliverable (ADR-0172; Premiere's export loudness normalisation, PRECEDENT.md §1)";
pub const HEADROOM_SOURCE: &str = "Mastered material rarely has a peak-to-loudness ratio below \
6 dB (ADR-0172's court)";

/// The project's `master`, where it is written and parses. `None` for an absent, inert or
/// malformed one: a malformed `master` is the schema check's finding.
pub fn of(document: &Loose) -> Option<Master> {
    let master = document.value().get("master")?;
    serde_json::from_value::<Master>(master.clone())
        .ok()
        .filter(|master| !master.is_inert())
}

pub fn check(document: &Loose, report: &mut Report) {
    let Some(master) = of(document) else {
        return;
    };
    let at = |code: &str| Finding::new(code).at_file(document.path());
    let target = master.target_lufs.map(|t| t.0);
    let ceiling = master.ceiling_dbtp.map(|c| c.0);

    if let (Some(target), None) = (target, ceiling) {
        report.push(at("R-MASTER-NO-CEILING").field("target_lufs", json!(target)));
    }
    if let Some(target) = target
        && !(QUIETEST_USUAL_LUFS..=LOUDEST_USUAL_LUFS).contains(&target)
    {
        let threshold = if target > LOUDEST_USUAL_LUFS {
            LOUDEST_USUAL_LUFS
        } else {
            QUIETEST_USUAL_LUFS
        };
        report.push(
            at("R-MASTER-TARGET-UNUSUAL")
                .field("target_lufs", json!(target))
                .field(
                    "direction",
                    json!(if target > LOUDEST_USUAL_LUFS {
                        "louder"
                    } else {
                        "quieter"
                    }),
                )
                .field("threshold_lufs", json!(threshold))
                .citation(Citation {
                    threshold: json!([QUIETEST_USUAL_LUFS, LOUDEST_USUAL_LUFS]),
                    source: TARGET_SOURCE.into(),
                    adr: "ADR-0172".into(),
                }),
        );
    }
    if let Some(ceiling) = ceiling
        && ceiling > LOSSY_CEILING_DBTP
    {
        report.push(
            at("R-MASTER-CEILING-HIGH")
                .field("ceiling_dbtp", json!(ceiling))
                .field("threshold_dbtp", json!(LOSSY_CEILING_DBTP))
                .citation(Citation {
                    threshold: json!(LOSSY_CEILING_DBTP),
                    source: CEILING_SOURCE.into(),
                    adr: "ADR-0172".into(),
                }),
        );
    }
    if let (Some(target), Some(ceiling)) = (target, ceiling) {
        let headroom = round1(ceiling - target);
        if headroom < MIN_HEADROOM_DB {
            report.push(
                at("R-MASTER-HEADROOM")
                    .field("target_lufs", json!(target))
                    .field("ceiling_dbtp", json!(ceiling))
                    .field("headroom_db", json!(headroom))
                    .field("threshold_db", json!(MIN_HEADROOM_DB))
                    .citation(Citation {
                        threshold: json!(MIN_HEADROOM_DB),
                        source: HEADROOM_SOURCE.into(),
                        adr: "ADR-0172".into(),
                    }),
            );
        }
    }
}

/// One decimal, the precision a dB figure is written and printed at.
pub(crate) fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
