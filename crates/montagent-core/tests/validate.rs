//! Seam 1 — the `montagent-core` verb API (ADR-0011, spec #168's "Testing strategy").
//!
//! Every assertion here goes through `montagent_core::validate`, with a path and a
//! typed argument struct, and reads the report back as values.

use montagent_core::finding::Class;
use montagent_core::report::ExitCode;
use montagent_core::text;
use montagent_core::validate;

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
    let path = write_project(&dir, "clean.montagent.json", HEADER_ONLY);

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
    let path = write_project(&dir, "clean.montagent.json", HEADER_ONLY);

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
    let path = write_project(&dir, "clean.montagent.json", HEADER_ONLY);

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
        "broken.montagent.json",
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
    // ADR-0073 (#224): not about a document — the caret is the only repair there is,
    // and no `repair` field is emitted at all.
    assert_eq!(f.repair, None);

    let json = report.to_json();
    let rendered = text::render(&json, text::Options::default()).unwrap();
    assert!(
        rendered.contains("\"fps\": ,"),
        "offending line verbatim:\n{rendered}"
    );
    assert!(rendered.contains('^'), "caret:\n{rendered}");
    assert!(rendered.contains("line 3"), "{rendered}");
    assert!(rendered.contains("byte 39"), "{rendered}");
    assert!(
        !rendered.contains("refuse-class") && !rendered.contains("advise-class"),
        "ADR-0073: a finding not about a document states no repair class in words:\n{rendered}"
    );
    assert!(
        json["findings"][0].get("repair").is_none(),
        "the field is absent, not `null`: {json}"
    );
}

#[test]
fn a_malformed_file_is_not_partially_processed() {
    // ADR-0011: "Nothing may partially process a malformed file." The thirteen tracks
    // that parsed before the fourteenth failed must not reach the report.
    let dir = tempdir();
    let path = write_project(
        &dir,
        "broken.montagent.json",
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
    let path = dir.join("absent.montagent.json");

    let report = validate(&path);

    assert_eq!(report.exit_code(), ExitCode::Unparseable);
    assert_eq!(report.findings[0].code, "E-READ");
    // ADR-0073 (#224): not about a document — the OS-derived advice is message text,
    // not a repair.
    assert_eq!(report.findings[0].repair, None);

    let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
    assert!(rendered.contains("could not be read"), "{rendered}");
    assert!(rendered.contains("check the path"), "{rendered}");
    assert!(
        !rendered.contains("refuse-class") && !rendered.contains("advise-class"),
        "{rendered}"
    );
}

#[test]
fn e_reads_advice_follows_the_failure_rather_than_being_fixed() {
    // A file that exists and is not UTF-8 is not a path problem, and telling its author
    // to "check the path and its permissions" sends them to the wrong place.
    let dir = tempdir();
    let path = dir.join("binary.montagent.json");
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
        montagent_core::report::Report::bad_invocation("--nope is not a flag of `validate`");

    assert_eq!(report.exit_code(), ExitCode::BadInvocation);
    assert_eq!(report.findings[0].code, "E-INVOCATION");
    // ADR-0073 (#224): not about a document — "fix the command" is a repair to the
    // invocation, not to anything Montagent can write, so no `repair` field.
    assert_eq!(report.findings[0].repair, None);
}

#[test]
fn even_a_report_that_never_reached_a_project_prints_not_checked() {
    // ADR-0006: "the report ends with its own scope, unconditionally." An exception for
    // the reports that opened no file reads as reasonable, and is how the block starts
    // becoming optional.
    for report in [
        montagent_core::report::Report::bad_invocation("--nope is not a flag"),
        montagent_core::report::Report::internal_failure("the font stack failed"),
    ] {
        let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
        assert!(rendered.contains("NOT CHECKED"), "{rendered}");
    }
}

#[test]
fn an_internal_failure_is_exit_70() {
    let report = montagent_core::report::Report::internal_failure("the font stack failed");

    assert_eq!(report.exit_code(), ExitCode::Internal);
    assert_eq!(report.findings[0].code, "E-INTERNAL");
    // ADR-0073 (#224): not about a document — "no flag can lift it" doesn't fit a
    // condition a retry might clear, so no `repair` field.
    assert_eq!(report.findings[0].repair, None);
}

#[test]
fn errors_exit_1_and_no_errors_exit_0() {
    use montagent_core::finding::Finding;

    let clean = montagent_core::report::Report::new("validate", Some("p.json".into()));
    assert_eq!(clean.exit_code(), ExitCode::Ok);

    let mut noted = montagent_core::report::Report::new("validate", Some("p.json".into()));
    noted.push(Finding::new("N-QUANTIZATION").at_file("p.json"));
    assert_eq!(
        noted.exit_code(),
        ExitCode::Ok,
        "exit non-zero only on `error` (ADR-0011)"
    );

    let mut errored = montagent_core::report::Report::new("validate", Some("p.json".into()));
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
        rendered.contains("does not look like a Montagent project file"),
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

// ---------------------------------------------------------------------------
// A remote source is probed, and `render` will not use it (#456, ADR-0131)
// ---------------------------------------------------------------------------

/// One element of each sourced type, all pointing into `base`.
fn sourced_project(dir: &std::path::Path, base: &str) -> std::path::PathBuf {
    write_project(
        dir,
        "remote.montagent.json",
        &common::canonical(&format!(
            r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",
                "duration":1000,"output":"out/remote.mp4",
                "tracks":[{{"name":"voice","layer":0,"elements":[
                  {{"id":"vo","type":"audio","start":0,"end":1000,
                    "source":"{base}/vo.mp3","source_start":0,"source_end":1000}}
                ]}},{{"name":"picture","layer":1,"elements":[
                  {{"id":"still","type":"image","start":0,"end":1000,
                    "source":"{base}/still.png","x":0,"y":0,"width":200,"height":200,
                    "fit":"cover"}}
                ]}}]}}"##
        )),
    )
}

fn served_media() -> std::path::PathBuf {
    let served = common::tempdir(line!());
    std::fs::copy(
        common::fixture_dir().join("audio/05-cobweb.mp3"),
        served.join("vo.mp3"),
    )
    .expect("copied");
    std::fs::copy(
        common::fixture_dir().join("images/05.png"),
        served.join("still.png"),
    )
    .expect("copied");
    served
}

#[test]
fn a_remote_source_that_answers_is_an_error_naming_the_element_the_url_and_the_remedy() {
    if !common::has_ffprobe() {
        return;
    }
    let base = common::serve(&served_media());
    let path = sourced_project(&common::tempdir(line!()), &base);

    let report = validate(&path);
    let remote: Vec<(&str, &str, Class, &str)> = report
        .findings
        .iter()
        .filter(|f| f.code.ends_with("-REMOTE"))
        .map(|f| {
            (
                f.code.as_str(),
                f.location.element.as_deref().unwrap_or("?"),
                f.class,
                f.fields["source"].as_str().unwrap_or("?"),
            )
        })
        .collect();
    let (vo, still) = (format!("{base}/vo.mp3"), format!("{base}/still.png"));
    assert_eq!(
        remote,
        vec![
            ("E-NOT-MIXED-REMOTE", "vo", Class::Error, vo.as_str()),
            (
                "E-NOT-PAINTED-REMOTE",
                "still",
                Class::Error,
                still.as_str()
            ),
        ],
        "{:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);

    let rendered = text::render(&report.to_json(), text::Options::default()).unwrap();
    assert!(
        !rendered.contains("0 errors"),
        "a document `render` refuses is not a clean pass:\n{rendered}"
    );
    for element in ["vo", "still"] {
        assert!(
            rendered.contains(&format!("`{element}` is not")),
            "{rendered}"
        );
    }
    assert_eq!(
        rendered
            .matches("The remedy is a local copy: fetch the file and point `source` at it")
            .count(),
        2,
        "{rendered}"
    );
}

#[test]
fn a_remote_video_is_neither_mixed_nor_painted() {
    if !common::has_ffprobe() {
        return;
    }
    // Nothing is served: whatever the probe says, the refusal is about the verb, so it is
    // stated beside the network finding rather than hidden behind it.
    let path = write_project(
        &common::tempdir(line!()),
        "remote.montagent.json",
        &common::canonical(
            r##"{"frame":{"width":200,"height":200},"fps":25,"background":"#000000",
                "duration":1000,"output":"out/remote.mp4",
                "tracks":[{"name":"only","layer":0,"elements":[
                  {"id":"clip","type":"video","start":0,"end":1000,
                   "source":"http://127.0.0.1:9/clip.mp4","source_start":0,"source_end":1000,
                   "x":0,"y":0,"width":200,"height":200,"fit":"cover"}
                ]}]}"##,
        ),
    );
    let report = validate(&path);
    let codes: Vec<&str> = report
        .findings
        .iter()
        .map(|f| f.code.as_str())
        .filter(|code| code.ends_with("-REMOTE"))
        .collect();
    assert_eq!(codes, ["E-NOT-MIXED-REMOTE", "E-NOT-PAINTED-REMOTE"]);
}

#[test]
fn a_file_url_is_local_and_draws_no_remote_finding() {
    if !common::has_ffprobe() {
        return;
    }
    let served = served_media();
    let path = sourced_project(
        &common::tempdir(line!()),
        &format!("file://{}", served.display()),
    );
    let report = validate(&path);
    assert!(
        report.findings.iter().all(|f| !f.code.ends_with("-REMOTE")),
        "{:?}",
        report.findings
    );
    assert_eq!(report.exit_code(), ExitCode::Ok, "{:?}", report.findings);
}
