//! `measure`'s keyed-alpha coverage path with no `ffmpeg`/`ffprobe` on `PATH` (#377).
//!
//! **Its own test binary, for the reason `frame_environment.rs` already gives**: this
//! empties the process's `PATH` for the duration of one call, and Cargo runs a binary's
//! tests as threads of one process, so a sibling spawning `ffmpeg` in this binary would see
//! the emptied `PATH` too. One test, its own file.

use montagent_core::report::ExitCode;
use montagent_core::verbs::measure::{Ask, measure};

mod common;
use common::{canonical, tempdir, write_project};

#[test]
fn a_missing_ffmpeg_on_the_keyed_coverage_path_is_e_tool_missing_at_exit_70_not_e_invocation() {
    // Before #377: `keyed::coverage` collapsed `tools::resolve()`'s `Missing` into the same
    // bare `String` an element-shaped refusal uses, and `measure` reported it as
    // `E-INVOCATION`/exit 3 — "fix the command" for a machine that has no `ffmpeg` at all.
    // ADR-0091 already drew the right line for `render`/`preview`/`validate`/`probe`; this
    // element never reaches an `ffmpeg` that could resolve, so `Source::resolve`'s target
    // need not exist on disk.
    let element = r##"{"id":"trex","type":"video","start":0,"end":1000,
        "source":"nowhere/does-not-exist.mp4","x":0,"y":0,"origin":"top-left",
        "width":100,"height":100,
        "effects":[{"name":"chroma","color":"#00CD00","tolerance":0.05,"softness":0.0,"spill":0.0}]}"##;
    let body = canonical(&format!(
        r##"{{"frame":{{"width":100,"height":100}},"fps":25,
            "tracks":[{{"name":"t","layer":0,"elements":[{element}]}}]}}"##
    ));
    let dir = tempdir(line!());
    let path = write_project(&dir, "p.montagent.json", &body);

    let saved_path = std::env::var_os("PATH");
    // SAFETY: one test, one variable, read back only through the code under test, and
    // restored before the test returns.
    unsafe { std::env::set_var("PATH", "") };
    let answer = measure(
        &path,
        &Ask {
            element: Some(serde_json::from_str(element).expect("the element is JSON")),
            ..Default::default()
        },
    );
    match saved_path {
        Some(path) => unsafe { std::env::set_var("PATH", path) },
        None => unsafe { std::env::remove_var("PATH") },
    }

    let report = answer.report();
    assert_eq!(
        report.exit_code(),
        ExitCode::Internal,
        "ADR-0011: exit 70 is for Montagent breaking, not a bad invocation: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    let json = answer.to_json();
    let codes: Vec<&str> = json["findings"]
        .as_array()
        .expect("a findings array")
        .iter()
        .filter_map(|f| f["code"].as_str())
        .collect();
    assert_eq!(
        codes,
        vec!["E-TOOL-MISSING"],
        "ADR-0091: an unresolvable `ffmpeg` is the designed-for gap, not an internal \
         failure and not a bad invocation: {}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
}
