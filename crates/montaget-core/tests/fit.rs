//! `E-FIT-DEVIATION` (#204): does a declared rect actually derive from the `fit` rule the
//! author named, computed against the real source on disk?
//!
//! Real media and a real `ffprobe`, on `tests/disk.rs`'s own reasoning: a recorded probe
//! would let this check pass against a source that never existed, and the whole point is
//! that the numbers in the document and the numbers in the file agree.

use std::path::{Path, PathBuf};

use montaget_core::finding::Class;
use montaget_core::media::probe::ProcessRunner;
use montaget_core::media::session::Session;
use montaget_core::media::tools;
use montaget_core::report::{ExitCode, Report};

mod common;
use common::{canonical, has_ffprobe, write_project};

fn validate(path: &Path) -> Report {
    match tools::resolve() {
        Ok(tools) => {
            let mut session = Session::with(tools, Box::new(ProcessRunner));
            montaget_core::verbs::validate::validate_with(path, &mut session)
        }
        Err(_) => montaget_core::validate(path),
    }
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("a string is always serialisable")
}

/// A one-element image project, every geometry field a caller can vary.
fn image_project(source: &str, width: i64, height: i64, fit: &str, clip: [i64; 4]) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"photo","layer":1,"elements":[{{"id":"photo-01","type":"image","start":0,"end":1000,"source":{source},"x":0,"y":0,"origin":"top-left","width":{width},"height":{height},"fit":{fit},"clip":{clip:?}}}]}}]}}"##,
        source = json_string(source),
        fit = json_string(fit),
    ))
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

// ---------------------------------------------------------------------------
// `E-FIT-DEVIATION` fires at strict equality, and only then.
// ---------------------------------------------------------------------------

#[test]
fn a_correctly_derived_rect_validates_clean() {
    if !has_ffprobe() {
        return;
    }
    // ADR-0013's own worked case: `images/06.png` is 1536x2720, `clip` is 1080x1300, and
    // exact cover floors to 1912.
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &image_project(
            &source.display().to_string(),
            1080,
            1912,
            "cover",
            [0, 0, 1080, 1300],
        ),
    );

    let report = validate(&path);
    assert!(
        report.findings.iter().all(|f| f.code != "E-FIT-DEVIATION"),
        "{:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn a_declared_rect_one_pixel_off_the_rule_is_an_error_naming_both_rects() {
    if !has_ffprobe() {
        return;
    }
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        // The rule derives 1912; a hand-typed 1913 is the stale-rounding case ADR-0015
        // was written to refuse rather than accept as a `note`.
        &image_project(
            &source.display().to_string(),
            1080,
            1913,
            "cover",
            [0, 0, 1080, 1300],
        ),
    );

    let report = validate(&path);

    assert_eq!(codes(&report), ["E-FIT-DEVIATION"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.location.element.as_deref(), Some("photo-01"));

    // ADR-0006: every relevant number inline.
    assert_eq!(finding.fields["source_width"], 1536);
    assert_eq!(finding.fields["source_height"], 2720);
    assert_eq!(finding.fields["declared_width"], 1080);
    assert_eq!(finding.fields["declared_height"], 1913);
    assert_eq!(finding.fields["rule_width"], 1080);
    assert_eq!(finding.fields["rule_height"], 1912);
    assert_eq!(finding.fields["fit"], "cover");

    // ADR-0024: `measure`'s eventual repair is exactly what this check already knows —
    // fully determined by the document and the media, so the class is advise.
    assert_eq!(
        finding.repair,
        Some(montaget_core::finding::Repair::Advise(serde_json::json!({
            "width": 1080,
            "height": 1912
        })))
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn the_deviation_finding_renders_both_rects() {
    if !has_ffprobe() {
        return;
    }
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &image_project(
            &source.display().to_string(),
            1080,
            1913,
            "cover",
            [0, 0, 1080, 1300],
        ),
    );

    let rendered = montaget_core::wire::render(
        &validate(&path),
        montaget_core::Wire::Text { verbose: false },
    );

    assert!(rendered.contains("1080x1913"), "{rendered}");
    assert!(rendered.contains("1080x1912"), "{rendered}");
    assert!(rendered.contains("fit:\"cover\""), "{rendered}");
}

// ---------------------------------------------------------------------------
// ADR-0015's zero-extent case: reported exactly, never clamped.
// ---------------------------------------------------------------------------

#[test]
fn a_legal_zero_extent_validates_clean_when_declared_exactly() {
    if !has_ffprobe() {
        return;
    }
    // `contain` of a 1536x2720 source into a 2000x1 box: height drives (verbatim 1), and
    // the width axis floors to `1536*1/2720 = 0`.
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &image_project(
            &source.display().to_string(),
            0,
            1,
            "contain",
            [0, 0, 2000, 1],
        ),
    );

    let report = validate(&path);
    assert!(
        report.findings.iter().all(|f| f.code != "E-FIT-DEVIATION"),
        "a correctly-transcribed zero is not a deviation: {:?}",
        report.findings
    );
}

#[test]
fn a_clamped_one_instead_of_the_legal_zero_is_a_deviation() {
    if !has_ffprobe() {
        return;
    }
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &image_project(
            &source.display().to_string(),
            1,
            1,
            "contain",
            [0, 0, 2000, 1],
        ),
    );

    let report = validate(&path);
    assert_eq!(codes(&report), ["E-FIT-DEVIATION"]);
    assert_eq!(
        report.findings[0].fields["rule_width"], 0,
        "the rule's own zero, stated rather than clamped to 1"
    );
}

// ---------------------------------------------------------------------------
// ADR-0026: exact-aspect match — both spellings stand.
// ---------------------------------------------------------------------------

#[test]
fn an_exact_aspect_match_validates_clean_under_either_spelling() {
    if !has_ffprobe() {
        return;
    }
    // The fixture's own `handle-logo`: an 800x800 source into a 68x68 aperture. `cover`
    // and `contain` derive the identical rect, so both must validate clean.
    let source = fixture_dir().join("brand/logo-en.png");
    for fit in ["cover", "contain"] {
        let dir = common::tempdir(std::panic::Location::caller().line());
        let path = write_project(
            &dir,
            "p.montaget.json",
            &image_project(&source.display().to_string(), 68, 68, fit, [0, 0, 68, 68]),
        );

        let report = validate(&path);
        assert!(
            report.findings.iter().all(|f| f.code != "E-FIT-DEVIATION"),
            "fit:{fit:?}: {:?}",
            report.findings
        );
    }
}

// ---------------------------------------------------------------------------
// `literal` states no rule at all.
// ---------------------------------------------------------------------------

#[test]
fn literal_never_fires_however_far_the_declared_rect_is_from_the_rule() {
    if !has_ffprobe() {
        return;
    }
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &image_project(
            &source.display().to_string(),
            9999,
            1,
            "literal",
            [0, 0, 1080, 1300],
        ),
    );

    let report = validate(&path);
    assert!(
        report.findings.iter().all(|f| f.code != "E-FIT-DEVIATION"),
        "{:?}",
        report.findings
    );
}

// ---------------------------------------------------------------------------
// An unprobeable source suppresses the check (ADR-0015), never fabricates a finding.
// ---------------------------------------------------------------------------

#[test]
fn a_missing_source_reports_its_own_finding_and_never_a_fit_deviation() {
    if !has_ffprobe() {
        return;
    }
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &image_project(
            "does-not-exist.png",
            1080,
            1912,
            "cover",
            [0, 0, 1080, 1300],
        ),
    );

    let report = validate(&path);
    assert_eq!(codes(&report), ["E-SOURCE-MISSING"]);
}

// ---------------------------------------------------------------------------
// No `clip` — nothing this check derives against; left to the schema check.
// ---------------------------------------------------------------------------

#[test]
fn cover_with_no_clip_reports_nothing_from_this_check() {
    if !has_ffprobe() {
        return;
    }
    let source = fixture_dir().join("images/06.png");
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{{"name":"photo","layer":1,"elements":[{{"id":"photo-01","type":"image","start":0,"end":1000,"source":{source},"x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover"}}]}}]}}"##,
            source = json_string(&source.display().to_string()),
        )),
    );

    let report = validate(&path);
    assert!(
        report.findings.iter().all(|f| f.code != "E-FIT-DEVIATION"),
        "{:?}",
        report.findings
    );
}
