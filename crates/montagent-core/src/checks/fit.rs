//! `E-FIT-DEVIATION` — *"does a declared rect actually derive from the `fit` rule the
//! author named?"* (ADR-0013, ADR-0015, ADR-0024, ADR-0026).
//!
//! `fit` never executes (ADR-0015's load-bearing sentence): the declared rect is drawn
//! verbatim, and `fit` is a claim about how the author computed it — a provenance tag on
//! two integers, not a render instruction. `validate` is `fit`'s only consumer, and this
//! is the check.
//!
//! `cover`/`contain` require `clip`, which is the box (ADR-0015: *"cover and contain
//! therefore require `clip`; declaring one without it is a schema error"*) — that schema
//! error is a later ticket's, and until it lands this check simply has nothing to compare
//! an element carrying neither against, and says nothing about it.
//!
//! **The arithmetic is [`crate::exact::fitted_extent`]** — the one place ADR-0013's exact
//! integer rule lives, so `validate` and a future `measure` (#205, ADR-0024) never carry
//! two implementations that could silently disagree. **Source dimensions are
//! [`crate::media::probe::Probe::dimensions`]** — ADR-0023's pipeline, run once by
//! `ffprobe` and shared with `crate::checks::source` through the same probe [`Session`]
//! and its cache, never re-derived here.
//!
//! **Strict equality, both axes** (ADR-0015's erratum over ADR-0013's original `note`):
//! the declared `width`/`height` must equal the rule's value exactly, or the finding
//! fires. Not under `fit: "literal"`, which states no rule at all. Not at an exact-aspect
//! match (ADR-0026) — `cover` and `contain` derive the identical rect there, so whichever
//! the author wrote is already the value this check would otherwise demand.
//!
//! **An unprobeable source suppresses this check.** ADR-0015 names this explicitly as
//! *"the first `error` an unprobeable source can suppress"* — the disk half of the
//! question is unanswerable, not failed, and `crate::checks::source` already reports the
//! corresponding `U-SOURCE-*`/`E-SOURCE-MISSING` finding for the same element.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::exact::{FitRule, fitted_extent};
use crate::finding::Finding;
use crate::media::Source;
use crate::media::probe::Outcome;
use crate::media::session::Session;
use crate::media::tools::Missing;
use crate::permissive::Loose;
use crate::report::Report;

/// Every raster-source element declaring `cover` or `contain`, checked against the
/// source's real dimensions.
///
/// Takes a session rather than opening its own: `crate::checks::source` already probes
/// every referenced source in the same run, and a second, independent session here would
/// either duplicate every `ffprobe` call or — worse — overwrite `report.misses` with a
/// second, partial view of what actually ran. `crate::verbs::validate` opens the one
/// session both checks share.
pub fn check(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    let base = crate::checks::project_dir(document);

    for (track, element) in document.elements_in_tracks() {
        let Some(rule) = fit_rule(element) else {
            continue;
        };
        let Some((box_width, box_height)) = clip_dimensions(element) else {
            // No `clip` (or a malformed one): there is no box to derive against. ADR-0015
            // makes an unclipped `cover`/`contain` a schema error of its own, which is not
            // this check's to raise (see the module doc).
            continue;
        };

        // #614 (prototype): every swap's file is stretched into the same declared box, so
        // each one is held to the rule the base is held to.
        let mut seen: Vec<&str> = Vec::new();
        for source in crate::swaps::sources(element) {
            if seen.contains(&source) {
                continue;
            }
            seen.push(source);
            let outcome = session.probe(&Source::resolve(source, &base))?;
            let Outcome::Probed(probe) = &outcome else {
                // Existence-only, a confirmed miss, or genuinely unprobeable: nothing about
                // the source's real dimensions was established, so the deviation question is
                // unanswerable rather than failed (ADR-0015).
                continue;
            };
            let Some(dimensions) = probe.dimensions else {
                continue;
            };

            let Some(finding) = deviation(
                element,
                rule,
                (i64::from(dimensions.width), i64::from(dimensions.height)),
                (box_width, box_height),
                source,
            ) else {
                continue;
            };

            let mut finding = finding.at_file(document.path());
            if let Some(track) = track {
                finding = finding.at_track(track);
            }
            report.push(finding);
        }
    }
    Ok(())
}

/// This element's own finding, or `None` where the declared rect already agrees with the
/// rule — including the exact-aspect case, where it agrees with *this* rule precisely
/// because `cover` and `contain` compute the same thing there (ADR-0026).
fn deviation(
    element: &Value,
    rule: FitRule,
    source_dimensions: (i64, i64),
    aperture: (i64, i64),
    source: &str,
) -> Option<Finding> {
    let declared_width = element.get("width").and_then(Value::as_i64)?;
    let declared_height = element.get("height").and_then(Value::as_i64)?;

    let extent = fitted_extent(rule, source_dimensions, aperture)?;
    if declared_width == extent.width && declared_height == extent.height {
        return None;
    }

    let subject = subject_of(element.get("id").and_then(Value::as_str));
    let (source_width, source_height) = source_dimensions;
    Some(
        Finding::new("E-FIT-DEVIATION")
            .at_element(subject.clone())
            .field("element", json!(subject))
            .field("fit", json!(fit_spelling(rule)))
            .field("source", json!(source))
            .field("source_width", json!(source_width))
            .field("source_height", json!(source_height))
            .field("declared_width", json!(declared_width))
            .field("declared_height", json!(declared_height))
            .field("rule_width", json!(extent.width))
            .field("rule_height", json!(extent.height))
            .repair_value(json!({"width": extent.width, "height": extent.height})),
    )
}

fn fit_spelling(rule: FitRule) -> &'static str {
    match rule {
        FitRule::Cover => "cover",
        FitRule::Contain => "contain",
    }
}

/// This element's `fit` rule, or `None` where it names no rule at all — `literal`, an
/// unknown spelling (another check's fact to report), or absence.
fn fit_rule(element: &Value) -> Option<FitRule> {
    match element.get("fit").and_then(Value::as_str) {
        Some("cover") => Some(FitRule::Cover),
        Some("contain") => Some(FitRule::Contain),
        _ => None,
    }
}

/// `clip`'s width and height — the box ADR-0015 names as `cover`/`contain`'s aperture.
fn clip_dimensions(element: &Value) -> Option<(i64, i64)> {
    let clip = element.get("clip")?.as_array()?;
    if clip.len() != 4 {
        return None;
    }
    Some((clip[2].as_i64()?, clip[3].as_i64()?))
}
