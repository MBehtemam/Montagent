//! ADR-0112 §4's stopped run: `validate` with no `ffprobe` on `PATH` records `["document"]`.
//!
//! **Its own test binary, for the reason `frame_environment.rs` already gives**: this
//! empties the process's `PATH` for the duration of one call, and Cargo runs a binary's
//! tests as threads of one process, so a sibling spawning `ffprobe` in this binary would see
//! the emptied `PATH` too. One test, its own file.

use montagent_core::report::ExitCode;
use montagent_core::text;
use montagent_core::validate_with_cache;

mod common;
use common::{fixture_project, tempdir};

#[test]
fn a_run_with_no_ffprobe_records_the_document_half_and_says_the_disk_half_did_not_run() {
    let cache = tempdir(line!()).join("sidecar.json");

    let saved_path = std::env::var_os("PATH");
    // SAFETY: one test, one variable, read back only through the code under test, and
    // restored before the test returns.
    unsafe { std::env::set_var("PATH", "") };
    let report = validate_with_cache(&fixture_project(), &cache);
    match saved_path {
        Some(path) => unsafe { std::env::set_var("PATH", path) },
        None => unsafe { std::env::remove_var("PATH") },
    }

    assert_eq!(report.exit_code(), ExitCode::Internal);
    let json = report.to_json();
    assert_eq!(json["check_sets"], serde_json::json!(["document"]));
    assert!(
        report.findings.iter().any(|f| f.code == "E-TOOL-MISSING"),
        "{:?}",
        report.findings
    );
    // The document half's findings stand (ADR-0112 §4), so the header counts them after
    // saying which half did not run.
    assert!(json["summary"]["review"].as_u64().unwrap() > 0, "{json}");

    let prose = text::render(&json, text::Options::default()).unwrap();
    let header = prose.lines().next().unwrap();
    assert!(
        header.starts_with("disk checks not run; 1 error, "),
        "{header}"
    );
    assert!(!header.contains("drift"), "{header}");
    let flowed = prose.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flowed.contains("validate's disk checks were not run: the run stopped before them."),
        "{prose}"
    );
}
