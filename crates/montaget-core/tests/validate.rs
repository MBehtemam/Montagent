//! Seam 1 — the `montaget-core` verb API (ADR-0011, spec #168's "Testing strategy").
//!
//! Every assertion here goes through `montaget_core::validate`, with a path and a
//! typed argument struct, and reads the report back as values.

use montaget_core::report::ExitCode;
use montaget_core::text;
use montaget_core::validate;

mod common;
use common::write_project;

// `r##` rather than `r#`: the background colour contains `"#`, which would close a
// single-hash raw string.
const HEADER_ONLY: &str = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "background": "#FBF3E3",
  "duration": 65216,
  "output": "out/clean.mp4",
  "tracks": []
}
"##;

#[test]
fn header_only_project_returns_a_clean_report() {
    let dir = tempdir();
    let path = write_project(&dir, "clean.montaget.json", HEADER_ONLY);

    let report = validate(&path);

    assert!(
        report.findings.is_empty(),
        "expected no findings, got {:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Ok);
    assert_eq!(report.summary().error, 0);
}

#[test]
fn a_clean_report_still_prints_the_not_checked_block() {
    let dir = tempdir();
    let path = write_project(&dir, "clean.montaget.json", HEADER_ONLY);

    let rendered = text::render(&validate(&path).to_json(), text::Options::default()).unwrap();

    assert!(
        rendered.contains("NOT CHECKED"),
        "NOT CHECKED is printed on every report, including clean ones:\n{rendered}"
    );
    // The block is wrapped for the terminal, so compare on collapsed whitespace.
    let flowed = rendered.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flowed.contains("it cannot tell you whether it says what you meant it to say"));
}

#[test]
fn a_clean_report_summarises_in_one_line() {
    let dir = tempdir();
    let path = write_project(&dir, "clean.montaget.json", HEADER_ONLY);

    let rendered = text::render(&validate(&path).to_json(), text::Options::default()).unwrap();
    let summary = rendered.lines().next().unwrap();

    assert!(summary.starts_with("0 errors"), "got: {summary}");
    assert!(summary.contains("0 unchecked"), "got: {summary}");
    assert!(summary.contains("0 layout"), "got: {summary}");
}

#[test]
fn a_malformed_file_is_e_parse_with_a_located_caret_and_exit_2() {
    let dir = tempdir();
    // `"fps": ,` — a value is missing, on line 3, so the failure has a real location.
    let path = write_project(
        &dir,
        "broken.montaget.json",
        "{\n  \"frame\": {\"width\": 1080},\n  \"fps\": ,\n  \"tracks\": []\n}\n",
    );

    let report = validate(&path);

    assert_eq!(report.exit_code(), ExitCode::Unparseable);
    assert_eq!(report.findings.len(), 1);
    let f = &report.findings[0];
    assert_eq!(f.code, "E-PARSE");
    assert_eq!(f.location.line, Some(3));
    assert_eq!(f.location.column, Some(10));
    // The byte offset is the one the line/column pair resolves to, counted from 0:
    // 2 bytes of line 1, 28 of line 2, then 9 columns into line 3.
    assert_eq!(f.location.byte_offset, Some(39));

    let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
    assert!(
        rendered.contains("\"fps\": ,"),
        "offending line verbatim:\n{rendered}"
    );
    assert!(rendered.contains('^'), "caret:\n{rendered}");
    assert!(rendered.contains("line 3"), "{rendered}");
    assert!(rendered.contains("byte 39"), "{rendered}");
}

#[test]
fn a_malformed_file_is_not_partially_processed() {
    // ADR-0011: "Nothing may partially process a malformed file." The thirteen tracks
    // that parsed before the fourteenth failed must not reach the report.
    let dir = tempdir();
    let path = write_project(
        &dir,
        "broken.montaget.json",
        r#"{"frame": {"width": 1080, "height": 1920}, "fps": 25, "tracks": [{"name": "a", "layer": 1, "elements": []},"#,
    );

    let report = validate(&path);

    assert_eq!(report.exit_code(), ExitCode::Unparseable);
    assert_eq!(report.findings.len(), 1, "one E-PARSE and nothing else");
    assert_eq!(report.findings[0].code, "E-PARSE");
}

#[test]
fn an_unreadable_file_is_e_read_and_exit_2() {
    // Exit 2 is "the file could not be read *or* parsed" (ADR-0011). A file that was
    // never opened has no line, column or offending line, so it gets its own code
    // rather than an `E-PARSE` carrying three zeros.
    let dir = tempdir();
    let path = dir.join("absent.montaget.json");

    let report = validate(&path);

    assert_eq!(report.exit_code(), ExitCode::Unparseable);
    assert_eq!(report.findings[0].code, "E-READ");

    let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
    assert!(rendered.contains("could not be read"), "{rendered}");
    assert!(rendered.contains("check the path"), "{rendered}");
}

#[test]
fn e_reads_advice_follows_the_failure_rather_than_being_fixed() {
    // A file that exists and is not UTF-8 is not a path problem, and telling its author
    // to "check the path and its permissions" sends them to the wrong place.
    let dir = tempdir();
    let path = dir.join("binary.montaget.json");
    std::fs::write(&path, [0xff, 0xfe, 0x00, 0x01]).unwrap();

    let report = validate(&path);
    let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();

    assert_eq!(report.exit_code(), ExitCode::Unparseable);
    assert!(rendered.contains("valid UTF-8"), "{rendered}");
    assert!(rendered.contains("re-save the file as UTF-8"), "{rendered}");
    assert!(
        !rendered.contains("check the path"),
        "the advice must not name a path problem:\n{rendered}"
    );
}

#[test]
fn a_bad_invocation_is_exit_3() {
    let report =
        montaget_core::report::Report::bad_invocation("--nope is not a flag of `validate`");

    assert_eq!(report.exit_code(), ExitCode::BadInvocation);
    assert_eq!(report.findings[0].code, "E-INVOCATION");
}

#[test]
fn even_a_report_that_never_reached_a_project_prints_not_checked() {
    // ADR-0006: "the report ends with its own scope, unconditionally." An exception for
    // the reports that opened no file reads as reasonable, and is how the block starts
    // becoming optional.
    for report in [
        montaget_core::report::Report::bad_invocation("--nope is not a flag"),
        montaget_core::report::Report::internal_failure("the font stack failed"),
    ] {
        let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
        assert!(rendered.contains("NOT CHECKED"), "{rendered}");
    }
}

#[test]
fn an_internal_failure_is_exit_70() {
    let report = montaget_core::report::Report::internal_failure("the font stack failed");

    assert_eq!(report.exit_code(), ExitCode::Internal);
    assert_eq!(report.findings[0].code, "E-INTERNAL");
}

#[test]
fn errors_exit_1_and_no_errors_exit_0() {
    use montaget_core::finding::Finding;

    let clean = montaget_core::report::Report::new("validate", Some("p.json".into()));
    assert_eq!(clean.exit_code(), ExitCode::Ok);

    let mut noted = montaget_core::report::Report::new("validate", Some("p.json".into()));
    noted.push(Finding::new("N-QUANTIZATION").at_file("p.json"));
    assert_eq!(
        noted.exit_code(),
        ExitCode::Ok,
        "exit non-zero only on `error` (ADR-0011)"
    );

    let mut errored = montaget_core::report::Report::new("validate", Some("p.json".into()));
    errored.push(Finding::new("E-SOURCE-OVERRUN").at_file("p.json"));
    assert_eq!(errored.exit_code(), ExitCode::Errors);
}

#[test]
fn a_file_that_is_not_a_project_is_named_as_one_not_dumped_as_a_schema_error() {
    // ADR-0042's precondition, which `validate` now holds alongside `fmt`, `timeline` and
    // `query`. The gap the ADR found was message quality — a file missing its required keys
    // wholesale should "name the likely mismatch, not dump a raw schema error" — and since
    // #244 there is a schema check standing ready to do exactly that.
    let dir = tempdir();
    let path = write_project(
        &dir,
        "transcript.json",
        r#"{"segments": [{"start": 0.0, "text": "hello"}]}
"#,
    );

    let report = validate(&path);

    assert_eq!(
        report.findings.len(),
        1,
        "one finding about the file, and nothing about a project that is not there: {:?}",
        report.findings
    );
    assert_eq!(report.findings[0].code, "E-NOT-A-PROJECT");
    assert_eq!(report.exit_code(), ExitCode::Errors);

    let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
    assert!(
        rendered.contains("does not look like a Montaget project file"),
        "{rendered}"
    );
    assert!(
        !rendered.contains("published schema"),
        "the raw schema error is the thing ADR-0042 asked not to be dumped:\n{rendered}"
    );
}

/// A scratch directory of this test's own, keyed on the line that called for it.
#[track_caller]
fn tempdir() -> std::path::PathBuf {
    common::tempdir(std::panic::Location::caller().line())
}
