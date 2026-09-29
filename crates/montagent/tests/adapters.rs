//! Adapter smoke tests, and deliberately nothing more.
//!
//! Spec #168: *"One test per CLI subcommand asserting argv reaches the right core call
//! and the exit code is right; one test per MCP tool asserting the schema advertised
//! matches the one enforced. No check logic is ever tested through an adapter — if a
//! test needs the CLI to reach a rule, the rule is in the wrong crate."*

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const HEADER_ONLY: &str = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "background": "#FBF3E3",
  "duration": 65216,
  "output": "out/clean.mp4",
  "tracks": []
}
"##;

// ---- CLI ----------------------------------------------------------------------------

#[test]
fn cli_validate_on_a_header_only_project_is_a_clean_report_and_exit_0() {
    let project = scratch("cli-clean", "clean.montagent.json", HEADER_ONLY);
    let out = montagent(&["validate", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(0));
    assert!(out.stdout.starts_with("0 errors"), "{}", out.stdout);
    assert!(out.stdout.contains("NOT CHECKED"), "{}", out.stdout);
}

#[test]
fn cli_validate_json_replaces_the_text_report_and_never_accompanies_it() {
    let project = scratch("cli-json", "clean.montagent.json", HEADER_ONLY);
    let out = montagent(&["validate", project.to_str().unwrap(), "--json"]);

    assert_eq!(out.code, Some(0));
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    assert_eq!(json["summary"]["error"], 0);
    assert_eq!(json["exit_code"], 0);
}

#[test]
fn cli_validate_on_a_malformed_file_is_exit_2() {
    let project = scratch(
        "cli-broken",
        "broken.montagent.json",
        "{\n  \"frame\": {\"width\": 1080},\n  \"fps\": ,\n  \"tracks\": []\n}\n",
    );
    let out = montagent(&["validate", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(2), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("E-PARSE"), "{}", out.stdout);
}

#[test]
fn cli_fmt_check_reaches_the_verb_without_writing() {
    // One test per subcommand, asserting argv reaches the right core call and the exit
    // code is right (#168). The convention itself is tested in the core.
    let pretty = "{\n  \"frame\": {\n    \"width\": 1080,\n    \"height\": 1920\n  },\n  \"fps\": 25,\n  \"tracks\": []\n}\n";
    let project = scratch("cli-fmt-check", "p.montagent.json", pretty);
    let out = montagent(&["fmt", project.to_str().unwrap(), "--check"]);

    // ADR-0011: "Exit non-zero only on `error`", and `LAYOUT` is not one (ADR-0041).
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("1 layout"), "{}", out.stdout);
    assert!(out.stdout.contains("L-LAYOUT"), "{}", out.stdout);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), pretty);
}

#[test]
fn cli_fmt_writes_and_exits_0() {
    let pretty = "{\n  \"frame\": {\n    \"width\": 1080,\n    \"height\": 1920\n  },\n  \"fps\": 25,\n  \"tracks\": []\n}\n";
    let project = scratch("cli-fmt-write", "p.montagent.json", pretty);
    let out = montagent(&["fmt", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert_eq!(
        std::fs::read_to_string(&project).unwrap(),
        "{\n  \"frame\": {\"width\": 1080, \"height\": 1920},\n  \"fps\": 25,\n  \"tracks\": []\n}\n"
    );
}

#[test]
fn cli_create_project_reaches_the_verb_and_writes_the_scaffold() {
    let dir = scratch_dir("cli-create-project");
    let project = dir.join("new.montagent.json");
    let out = montagent(&[
        "create-project",
        project.to_str().unwrap(),
        "--width",
        "1080",
        "--height",
        "1920",
        "--fps",
        "25",
        "--background",
        "#FBF3E3",
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert_eq!(
        std::fs::read_to_string(&project).unwrap(),
        "{\n  \"frame\": {\"width\": 1080, \"height\": 1920},\n  \"fps\": 25,\n  \
         \"background\": \"#FBF3E3\",\n  \"tracks\": []\n}\n"
    );
    // ADR-0011's write-tool invariant: the findings are the result, so what comes back is
    // `validate`'s report on the new file rather than a receipt.
    assert!(out.stdout.starts_with("0 errors"), "{}", out.stdout);
    assert!(out.stdout.contains("NOT CHECKED"), "{}", out.stdout);
}

#[test]
fn cli_create_project_is_spelled_with_a_hyphen_and_answers_to_the_verbs_own_name() {
    // ADR-0080: `create_project` is the only verb whose ADR name is not a single word, so
    // it is the only place the CLI's one-word-per-subcommand habit and ADR-0011's snake
    // case meet. The hyphen is the CLI's spelling and the underscore is a permanent alias,
    // so a session that read the verb's name from an ADR or from the MCP surface and typed
    // it at a shell is not answered with a usage error. Both spellings are asserted, and a
    // bare `montagent create_project --help` is what an agent actually reaches for.
    for spelling in ["create-project", "create_project"] {
        let dir = scratch_dir(&format!("cli-create-project-{spelling}"));
        let project = dir.join("new.montagent.json");
        let out = montagent(&[
            spelling,
            project.to_str().unwrap(),
            "--width",
            "1080",
            "--height",
            "1920",
            "--fps",
            "25",
        ]);

        assert_eq!(
            out.code,
            Some(0),
            "{spelling}: {}{}",
            out.stdout,
            out.stderr
        );
        assert!(project.is_file(), "{spelling} wrote no file");
    }

    // The hyphen is what `--help` advertises: an alias that showed up in the listing would
    // be two names for one verb in the surface an agent reads, which is what ADR-0011's
    // one-thing-to-parse rule refuses.
    let help = montagent(&["--help"]);
    assert!(help.stdout.contains("create-project"), "{}", help.stdout);
    assert!(
        !help.stdout.contains("create_project"),
        "the underscore is an alias, not a second advertised verb: {}",
        help.stdout
    );
}

#[test]
fn cli_create_project_onto_an_existing_file_is_exit_3_and_changes_nothing() {
    let mine = "{\"an afternoon\": \"of work\"}\n";
    let project = scratch("cli-create-project-exists", "mine.montagent.json", mine);
    let out = montagent(&[
        "create-project",
        project.to_str().unwrap(),
        "--width",
        "1080",
        "--height",
        "1920",
        "--fps",
        "25",
    ]);

    // ADR-0080: exit 3, not 1 — the file that is there is intact, so there is no project
    // to fix; the path argument is what needs changing.
    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("E-PROJECT-EXISTS"), "{}", out.stdout);
    // ADR-0073's shape, reached via ADR-0080: the remedy is message text, so no `repair`
    // block is printed for it.
    assert!(
        !out.stdout.to_lowercase().contains("repair"),
        "a NotAboutDocument code prints no repair block: {}",
        out.stdout
    );
    assert!(
        out.stdout.contains("never overwrites"),
        "the remedy lives in the message: {}",
        out.stdout
    );
    assert_eq!(std::fs::read_to_string(&project).unwrap(), mine);
}

#[test]
fn cli_timeline_reaches_the_verb_and_exits_0() {
    let project = scratch("cli-timeline", "clean.montagent.json", HEADER_ONLY);
    let out = montagent(&["timeline", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("TIMELINE"), "{}", out.stdout);
    assert!(out.stdout.contains("1080×1920 at 25 fps"), "{}", out.stdout);
    assert!(
        out.stdout.contains("0 elements, 0 tracks, 0 groups"),
        "{}",
        out.stdout
    );
}

#[test]
fn cli_timeline_json_replaces_the_text_view_and_never_accompanies_it() {
    let project = scratch("cli-timeline-json", "clean.montagent.json", HEADER_ONLY);
    let out = montagent(&["timeline", project.to_str().unwrap(), "--json"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(!out.stdout.contains("TIMELINE"), "{}", out.stdout);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("the JSON form");
    assert_eq!(json["tool"], "timeline");
    assert_eq!(json["timeline"]["duration_ms"], 65216);
}

#[test]
fn cli_timeline_takes_no_flag_that_collapses_the_view() {
    // The view *is* the answer, so there is nothing for a `--verbose` to expand and nothing
    // a default would have collapsed. A flag that did neither would still have to be
    // supported forever, so its absence is asserted rather than left to drift in.
    let out = montagent(&["timeline", "--help"]);

    assert_eq!(out.code, Some(0));
    assert!(out.stdout.contains("--json"), "{}", out.stdout);
    assert!(!out.stdout.contains("--verbose"), "{}", out.stdout);
}

#[test]
fn cli_query_from_to_reaches_the_cut_list_and_exits_0() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "query",
        fixture.to_str().unwrap(),
        "--from",
        "17000",
        "--to",
        "19000",
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(
        out.stdout.contains("QUERY  cut list over [17000, 19000)"),
        "{}",
        out.stdout
    );
    // ADR-0011: the boundary immediately outside the range, on each side, so the caller
    // never has to guess a window.
    assert!(out.stdout.contains("\n  before  "), "{}", out.stdout);
    assert!(out.stdout.contains("\n  after  "), "{}", out.stdout);
}

#[test]
fn cli_query_where_and_census_reach_the_matched_set_and_the_distribution() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "query",
        fixture.to_str().unwrap(),
        "--where",
        "track = sentence-text",
        "--census",
        "y",
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("5 matched elements"), "{}", out.stdout);
    assert!(out.stdout.contains("census y: 5 at 1537"), "{}", out.stdout);
}

#[test]
fn cli_query_json_replaces_the_text_answer_and_never_accompanies_it() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "query",
        fixture.to_str().unwrap(),
        "--where",
        "type = ellipse",
        "--json",
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(!out.stdout.contains("QUERY"), "{}", out.stdout);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).expect("the JSON form");
    assert_eq!(json["tool"], "query");
    assert_eq!(json["query"]["mode"], "matches");
}

#[test]
fn cli_query_with_no_question_is_exit_3_from_the_verb() {
    // The mode rule lives in the verb, not in argv, so that the MCP surface — which has no
    // `clap` to arrange its arguments — is covered by the same one (ADR-0011).
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&["query", fixture.to_str().unwrap()]);

    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("E-INVOCATION"), "{}", out.stdout);
    assert!(out.stdout.contains("needs a question"), "{}", out.stdout);
}

#[test]
fn cli_query_takes_no_flag_that_collapses_the_answer() {
    // The answer *is* the output, so there is nothing for a `--verbose` to expand — the
    // same absence `timeline` asserts, for the same reason.
    let out = montagent(&["query", "--help"]);

    assert_eq!(out.code, Some(0));
    assert!(out.stdout.contains("--json"), "{}", out.stdout);
    assert!(out.stdout.contains("--where"), "{}", out.stdout);
    assert!(!out.stdout.contains("--verbose"), "{}", out.stdout);
    // All three of ADR-0011's modes reach the verb (#208). What `--at` does *not* yet carry
    // is the half that reaches outside the document — the crop rectangle, the ink box, the
    // offset into the source and `NOT COVERED` — which is #210.
    assert!(out.stdout.contains("--at"), "{}", out.stdout);
}

#[test]
fn cli_measure_reaches_the_verb_and_exits_0() {
    // One test per subcommand, asserting argv reaches the right core call and the exit code
    // is right (#168). The arithmetic is tested in the core.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--element",
        r#"{"id":"intro-title","type":"text","y":1470,"origin":"center","font":"brand","size":58,"line_height":1.1,"runs":[{"text":"4 English words for\ndecorating the house"}]}"#,
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(
        out.stdout.contains("MEASURE  `intro-title`"),
        "{}",
        out.stdout
    );
    assert!(out.stdout.contains("BREAK OPPORTUNITIES"), "{}", out.stdout);
}

#[test]
fn cli_measure_json_replaces_the_text_answer_and_never_accompanies_it() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--element",
        r#"{"font":"brand","size":58,"runs":[{"text":"x"}]}"#,
        "--json",
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    assert_eq!(json["measure"]["line_count"], 1);
}

#[test]
fn cli_measure_with_an_element_that_is_not_json_is_exit_3() {
    // The one thing argv must do that the MCP surface does not: an element arrives here as
    // a string. A string that is not JSON is an invocation error, with the same code and
    // the same object as every other one (ADR-0011).
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--element",
        "{not json",
    ]);

    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
    assert!(out.stderr.contains("E-INVOCATION"), "{}", out.stderr);
}

#[test]
fn cli_measure_takes_no_flag_that_collapses_the_answer() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--element",
        r#"{"font":"brand","size":58,"runs":[{"text":"x"}]}"#,
        "--verbose",
    ]);

    // Same rule as `timeline` and `query`: a verb whose output *is* the answer has nothing
    // left to say if the answer is filtered out, so there is no verbosity switch to offer.
    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
}

#[test]
fn cli_measure_at_reaches_the_verb_and_exits_0() {
    // ADR-0035: the second input mode, a time instead of an element. The fixture is 25fps,
    // whose 40ms step makes 3041 resolve to frame 76 at 3040ms — the same numbers the core
    // arithmetic is tested against directly.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&["measure", fixture.to_str().unwrap(), "--at", "3041"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("MEASURE"), "{}", out.stdout);
    assert!(out.stdout.contains("76"), "{}", out.stdout);
    assert!(out.stdout.contains("3040"), "{}", out.stdout);
}

#[test]
fn cli_measure_with_both_element_and_at_is_exit_3() {
    // The two input modes answer different questions, so a call naming both is an
    // invocation error rather than a silent pick of one.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--element",
        r#"{"font":"brand","size":58,"runs":[{"text":"x"}]}"#,
        "--at",
        "3041",
    ]);

    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
    // Unlike `--element` that is not JSON (a CLI-only parsing failure the verb never
    // sees), this is a verb-level rejection: it prints where every other answer does.
    assert!(out.stdout.contains("E-INVOCATION"), "{}", out.stdout);
}

#[test]
fn cli_measure_elements_returns_n_measured_blocks_in_one_call() {
    // #317's primitive: none of these need exist in the project file.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--elements",
        r#"[{"font":"brand","size":20,"runs":[{"text":"one"}]},
            {"font":"brand","size":30,"runs":[{"text":"two"}]}]"#,
        "--json",
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    let results = json["measure"]["results"].as_array().expect("a batch");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["ok"]["asked"]["size"], 20);
    assert_eq!(results[1]["ok"]["asked"]["size"], 30);
}

#[test]
fn cli_measure_elements_with_one_bad_slot_still_answers_the_rest() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--elements",
        r#"[{"font":"brand","size":20,"runs":[{"text":"ok"}]},
            {"font":"not-a-declared-key","size":20,"runs":[{"text":"bad"}]}]"#,
        "--json",
    ]);

    // The call is not refused — a batch with one bad slot is still an answer (#317).
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    let results = json["measure"]["results"].as_array().expect("a batch");
    assert!(results[0]["error"].is_null());
    assert_eq!(results[1]["error"]["code"], "E-INVOCATION");
    assert!(results[1]["ok"].is_null());
}

#[test]
fn cli_measure_elements_that_is_not_a_json_array_is_exit_3_not_a_per_slot_error() {
    // The same split as `--element` that is not JSON: a CLI-only parsing failure the verb
    // never sees, so the refusal prints on stderr like every other invocation error the
    // argv layer itself catches.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--elements",
        "{not an array}",
    ]);

    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
    assert!(out.stderr.contains("E-INVOCATION"), "{}", out.stderr);
}

#[test]
fn cli_measure_all_returns_one_block_per_text_element_already_in_the_project() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&["measure", fixture.to_str().unwrap(), "--all", "--json"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    let results = json["measure"]["results"].as_array().expect("a batch");
    assert!(
        !results.is_empty(),
        "the fixture has text elements: {results:?}"
    );
    assert!(results.iter().all(|slot| slot["error"].is_null()));
}

#[test]
fn cli_measure_all_on_a_project_with_no_elements_is_an_empty_batch_not_an_error() {
    let project = scratch("cli-measure-all-empty", "clean.montagent.json", HEADER_ONLY);
    let out = montagent(&["measure", project.to_str().unwrap(), "--all", "--json"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    assert_eq!(json["measure"]["results"], serde_json::json!([]));
}

#[test]
fn cli_measure_elements_and_all_are_mutually_exclusive_with_each_other_and_element_and_at() {
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");

    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--elements",
        r#"[{"font":"brand","size":20,"runs":[{"text":"x"}]}]"#,
        "--all",
    ]);
    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("E-INVOCATION"), "{}", out.stdout);

    let out = montagent(&[
        "measure",
        fixture.to_str().unwrap(),
        "--element",
        r#"{"font":"brand","size":20,"runs":[{"text":"x"}]}"#,
        "--all",
    ]);
    assert_eq!(out.code, Some(3), "{}{}", out.stdout, out.stderr);
}

#[test]
fn cli_fonts_list_reaches_the_verb_and_exits_0() {
    let dir = scratch_dir("cli-fonts-list");
    std::fs::copy(open_runde(), dir.join("OpenRunde-Bold.otf")).unwrap();
    let out = montagent(&["fonts", "list", "--root", dir.to_str().unwrap()]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("FONTS  1 face under"), "{}", out.stdout);
    assert!(out.stdout.contains("unknown"), "{}", out.stdout);
}

#[test]
fn cli_fonts_list_json_replaces_the_text_listing_and_never_accompanies_it() {
    let dir = scratch_dir("cli-fonts-list-json");
    std::fs::copy(open_runde(), dir.join("OpenRunde-Bold.otf")).unwrap();
    let out = montagent(&["fonts", "list", "--root", dir.to_str().unwrap(), "--json"]);

    assert_eq!(out.code, Some(0));
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    assert_eq!(json["tool"], "fonts list");
    assert_eq!(json["fonts"]["fonts"][0]["status"], "unknown");
}

#[test]
fn cli_fonts_vendor_reaches_the_gate_and_the_copy() {
    let project = scratch("cli-fonts-vendor", "p.montagent.json", HEADER_ONLY);
    let font = open_runde();

    // Bucket 3 without a declaration: exit 1, and nothing copied.
    let out = montagent(&[
        "fonts",
        "vendor",
        project.to_str().unwrap(),
        font.to_str().unwrap(),
    ]);
    assert_eq!(out.code, Some(1), "{}{}", out.stdout, out.stderr);
    assert!(
        out.stdout.contains("E-FONT-LICENCE-UNKNOWN"),
        "{}",
        out.stdout
    );
    assert!(!project.parent().unwrap().join("fonts").exists());

    // With one: the copy lands, the attestation is written, and the report is the new
    // state's — one orphan note, exit 0.
    let out = montagent(&[
        "fonts",
        "vendor",
        project.to_str().unwrap(),
        font.to_str().unwrap(),
        "--licence",
        "OFL-1.1",
        "--json",
    ]);
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    assert_eq!(json["tool"], "fonts vendor");
    assert_eq!(json["vendor"]["file"], "fonts/OpenRunde-Bold.otf");
    assert_eq!(json["findings"][0]["code"], "N-FONT-ATTESTATION-ORPHANED");
    assert!(
        project
            .parent()
            .unwrap()
            .join("fonts/OpenRunde-Bold.otf")
            .is_file()
    );
}

fn open_runde() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
        .canonicalize()
        .expect("the vendored fixture font")
}

#[test]
fn cli_a_bad_invocation_is_exit_3_on_stderr() {
    let out = montagent(&["validate", "--nope"]);

    assert_eq!(out.code, Some(3));
    assert!(out.stderr.contains("E-INVOCATION"), "{}", out.stderr);
    assert!(
        out.stdout.is_empty(),
        "the report goes to stderr: {}",
        out.stdout
    );
}

#[test]
fn cli_a_source_that_grew_between_two_runs_is_announced_by_the_second() {
    // #231's criterion, at the only seam that can state it: two processes. Before the
    // sidecar (ADR-0069) the second run reported a `First` miss — "not yet in this
    // session's cache" — and the growth, the one defect class ADR-0011 says nothing else
    // can catch, went unsaid.
    let dir = scratch_dir("cli-grew-between-runs");
    let cache = dir.join("cache");
    let media = dir.join("take3.mp3");
    std::fs::copy(fixture_dir().join("audio/05-cobweb.mp3"), &media).expect("copy a real mp3");
    let source = media.to_str().unwrap().to_string();

    let first = montagent_caching(&["probe", &source], &cache);
    if first.code == Some(70) {
        eprintln!("skipping: {}", first.stderr.trim());
        return;
    }
    assert_eq!(first.code, Some(0), "{}{}", first.stdout, first.stderr);
    assert!(
        first.stdout.contains("not yet in the probe cache"),
        "the first run is a `First` miss:\n{}",
        first.stdout
    );

    // The file gains bytes — and, on this filesystem, a later mtime — between the runs.
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut grown = std::fs::read(&media).unwrap();
    grown.extend_from_slice(&std::fs::read(&media).unwrap());
    std::fs::write(&media, &grown).unwrap();

    let second = montagent_caching(&["probe", &source], &cache);
    assert_eq!(second.code, Some(0), "{}{}", second.stdout, second.stderr);
    assert!(
        second.stdout.contains("CHANGED ON DISK"),
        "the growth must be announced across the process boundary:\n{}",
        second.stdout
    );
    // Both halves of ADR-0006's key, both sides of the change.
    assert!(second.stdout.contains("bytes →"), "{}", second.stdout);
    assert!(second.stdout.contains("mtime "), "{}", second.stdout);

    // And a third run, with nothing changed, is a hit: no CACHE block at all.
    let third = montagent_caching(&["probe", &source], &cache);
    assert_eq!(third.code, Some(0), "{}{}", third.stdout, third.stderr);
    assert!(
        !third.stdout.contains("CACHE"),
        "an unchanged file is not a miss:\n{}",
        third.stdout
    );
}

#[test]
fn cli_the_probe_cache_never_lands_beside_the_project() {
    // ADR-0069: the sidecar lives under the per-user cache directory, so there is nothing
    // beside a project to commit by accident and ADR-0053's movable unit stays as it was.
    let dir = scratch_dir("cli-no-sidecar-beside-the-project");
    let cache = dir.join("cache");
    let media = dir.join("take3.mp3");
    std::fs::copy(fixture_dir().join("audio/05-cobweb.mp3"), &media).expect("copy a real mp3");

    let out = montagent_caching(&["probe", media.to_str().unwrap()], &cache);
    if out.code == Some(70) {
        eprintln!("skipping: {}", out.stderr.trim());
        return;
    }

    let beside: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name != "cache")
        .collect();
    assert_eq!(
        beside,
        vec!["take3.mp3".to_string()],
        "the run left something beside the media"
    );
    assert!(
        cache.join("probe-cache.json").exists(),
        "and wrote it here instead"
    );
}

#[test]
fn cli_cache_clear_removes_the_sidecar_and_its_temporaries_and_states_the_path() {
    // ADR-0092 / #385: the recovery step that previously required knowing
    // `~/Library/Caches/montagent/probe-cache.json` by heart.
    let dir = scratch_dir("cli-cache-clear");
    let cache = dir.join("cache");
    let media = dir.join("take3.mp3");
    std::fs::copy(fixture_dir().join("audio/05-cobweb.mp3"), &media).expect("copy a real mp3");

    let out = montagent_caching(&["probe", media.to_str().unwrap()], &cache);
    if out.code == Some(70) {
        eprintln!("skipping: {}", out.stderr.trim());
        return;
    }
    let sidecar = cache.join("probe-cache.json");
    assert!(sidecar.exists(), "there is a cache to clear");

    // A run killed between `write_atomically`'s write and its rename leaves one of these,
    // and a clear that left it would leave the directory it claims to have emptied non-empty.
    let temporary = cache.join("probe-cache.json.99999.tmp");
    std::fs::write(&temporary, b"half a write").unwrap();

    let out = montagent_caching(&["cache", "clear"], &cache);
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(!sidecar.exists(), "the sidecar is gone");
    assert!(!temporary.exists(), "and so is the temporary beside it");
    assert!(
        out.stdout.contains(sidecar.to_str().unwrap()),
        "it states the path it removed: {}",
        out.stdout
    );

    // Clearing an absent cache is the state the caller asked for, so it succeeds.
    let out = montagent_caching(&["cache", "clear"], &cache);
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(
        out.stdout.contains("no probe cache to clear"),
        "and says there was nothing there: {}",
        out.stdout
    );
}

#[test]
fn cli_probe_reaches_the_probe_verb_and_exits_0() {
    let fixture = fixture_dir();
    let out = montagent(&["probe", fixture.join("images/06.png").to_str().unwrap()]);

    if out.code == Some(70) {
        eprintln!("skipping: {}", out.stderr.trim());
        return;
    }
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("1536×2720"), "{}", out.stdout);
    // ADR-0006 puts the cache line at the top of the report, above the facts, and `probe`
    // answers with the same report shape every other verb does.
    let cache = out.stdout.find("CACHE").expect(&out.stdout);
    let media = out.stdout.find("MEDIA").expect(&out.stdout);
    assert!(cache < media, "{}", out.stdout);
}

#[test]
fn cli_probe_json_replaces_the_text_report_and_never_accompanies_it() {
    let fixture = fixture_dir();
    let out = montagent(&[
        "probe",
        fixture.join("images/06.png").to_str().unwrap(),
        "--json",
    ]);

    if out.code == Some(70) {
        eprintln!("skipping: {}", out.stderr.trim());
        return;
    }
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON alone ({e}):\n{}", out.stdout));
    assert_eq!(json["media"][0]["dimensions"]["width"], 1536);
    assert_eq!(json["network_attempts"], 0);
}

#[test]
fn cli_probe_without_an_ffmpeg_on_path_is_exit_70() {
    // ADR-0011's exit 70: "internal failure (ffmpeg died, font stack failed) → retry or
    // report". The `PATH` is emptied for this child alone.
    let empty = scratch_dir("cli-no-ffmpeg");
    let out = Command::new(binary())
        .args(["probe", "anything.mp4"])
        .env("PATH", &empty)
        .output()
        .expect("run montagent");

    assert_eq!(out.status.code(), Some(70));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ffmpeg"), "{stderr}");
    assert!(
        stderr.contains(&empty.display().to_string()),
        "the search is the actionable half: {stderr}"
    );
}

#[cfg(unix)]
#[test]
fn cli_probe_with_an_ffprobe_too_old_for_our_flags_is_exit_70_not_a_media_finding() {
    // ADR-0009 ships "a binary, plus an `ffmpeg` the user supplies", so the supplied one
    // may be older than the flags Montagent passes. Driven through a real executable on a
    // real `PATH` rather than a hand-built value, because the claim under test is that
    // some code path actually reaches exit 70 — not that the type can represent it.
    let dir = scratch_dir("cli-old-ffprobe");
    for program in ["ffprobe", "ffmpeg"] {
        let path = dir.join(program);
        std::fs::write(
            &path,
            "#!/bin/sh\necho \"Unrecognized option 'protocol_whitelist'.\" >&2\nexit 1\n",
        )
        .expect("write the stub");
        let mut mode = std::fs::metadata(&path).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o755);
        std::fs::set_permissions(&path, mode).expect("make the stub executable");
    }

    let fixture = fixture_dir();
    let out = Command::new(binary())
        .args(["probe", fixture.join("images/06.png").to_str().unwrap()])
        .env("PATH", &dir)
        // A cold cache, so the stub is actually reached: a warm sidecar (ADR-0069) would
        // answer from the last run and the broken tool would never be asked anything.
        .env(
            montagent_core::media::sidecar::CACHE_DIR_VAR,
            dir.join("cache"),
        )
        .output()
        .expect("run montagent");

    assert_eq!(
        out.status.code(),
        Some(70),
        "a broken tool is exit 70, never exit 0 with the media blamed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("protocol_whitelist"), "{stderr}");
    assert!(
        stderr.contains(dir.join("ffprobe").to_str().unwrap()),
        "exit 70 names the resolved path, which is the thing to go and look at: {stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("U-SOURCE-UNPROBEABLE"),
        "the media is not the thing that failed"
    );
}

#[test]
fn cli_validate_offers_no_flag_that_narrows_the_disk_checks() {
    // ADR-0006, unanimous 5 of 5: "No fast mode. No `--no-probe`. No scoping of what is
    // checked." The reason is stated as a prediction about people: "the moment a fast path
    // exists it becomes the mode used in the edit loop, so the single highest-value check
    // in the tool surface is the one that gets skipped." A flag is the easiest thing in the
    // world to add later, so the absence is asserted rather than assumed.
    let out = montagent(&["validate", "--help"]);

    assert_eq!(out.code, Some(0));
    for forbidden in [
        "--no-probe",
        "--fast",
        "--skip",
        "--only",
        "--scope",
        "--group",
        "--no-disk",
        "--offline",
    ] {
        assert!(
            !out.stdout.contains(forbidden),
            "`{forbidden}` would be a way to not run the check that exists to catch the \
             defect that changed on disk rather than in the project:\n{}",
            out.stdout
        );
    }
    // The two that do exist are about the *output*, which ADR-0006 explicitly permits
    // scoping — "a checked-but-unprinted finding still exists; an unchecked one silently
    // does not".
    assert!(out.stdout.contains("--json"), "{}", out.stdout);
    assert!(out.stdout.contains("--verbose"), "{}", out.stdout);
}

#[test]
fn cli_validate_that_loses_its_ffprobe_keeps_what_it_had_already_learned() {
    // A run reaches the disk half having already read the document. With no `ffprobe` it
    // has learned two things — the retired key, and that it cannot look at the media — and
    // a report carrying only the second would send an agent off to fix its `PATH` and
    // re-run before hearing about the key it could have fixed in the same turn. Exit 70
    // still, because the run did not finish (ADR-0011).
    //
    // Driven as a child process because `PATH` is process-wide: setting it in-process
    // would break every sibling test that runs at the same time.
    let dir = scratch_dir("cli-lost-ffprobe");
    let project = dir.join("p.montagent.json");
    std::fs::write(
        &project,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"photos","layer":1,"elements":[{"id":"photo-06","type":"image","start":0,"end":1000,"source":"images/05.png","x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","gravity":"bottom"}]}]}"##,
    )
    .expect("write project");
    let empty = scratch_dir("cli-lost-ffprobe-path");

    let out = Command::new(binary())
        .args(["validate", project.to_str().unwrap()])
        .env("PATH", &empty)
        .output()
        .expect("run montagent");

    assert_eq!(out.status.code(), Some(70));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("E-RETIRED-KEY"),
        "what the document half learned survives: {stdout}"
    );
    // ADR-0091 (#368): never found on `PATH` is `E-TOOL-MISSING`, not `E-INTERNAL` — an
    // unconfigured environment (ADR-0009), not Montagent breaking.
    assert!(stdout.contains("E-TOOL-MISSING"), "{stdout}");
    assert!(stdout.contains("ffmpeg"), "{stdout}");
}

/// A `PATH` holding the real `ffprobe` and an `ffmpeg` that is present, runs, and rejects
/// the floor's `-/filter_complex` the way ffmpeg 6.1 does — the machine ADR-0115's tool
/// qualification exists for. `None` where this machine has no `ffprobe` to borrow.
#[cfg(unix)]
fn path_with_an_unqualified_ffmpeg(name: &str) -> Option<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let ffprobe = montagent_core::media::tools::resolve_found()
        .ok()?
        .0
        .ffprobe;
    let dir = scratch_dir(name);
    let ffmpeg = dir.join("ffmpeg");
    std::fs::write(
        &ffmpeg,
        "#!/bin/sh\necho \"Unrecognized option '/filter_complex'.\" >&2\n\
         echo \"Error splitting the argument list: Option not found\" >&2\nexit 8\n",
    )
    .expect("write the stand-in ffmpeg");
    std::fs::set_permissions(&ffmpeg, std::fs::Permissions::from_mode(0o755))
        .expect("make it executable");
    let _ = std::fs::remove_file(dir.join("ffprobe"));
    std::os::unix::fs::symlink(&ffprobe, dir.join("ffprobe")).expect("borrow the real ffprobe");
    Some(dir)
}

#[cfg(unix)]
#[test]
fn cli_validate_on_an_ffmpeg_below_the_floor_is_an_error_and_still_reads_the_disk() {
    // ADR-0115 §6: `render` is guaranteed to refuse, so `validate` says so at `error` —
    // exit 70, `E-TOOL-UNSUPPORTED`, naming the floor and the binary — and the disk half,
    // which needs only `ffprobe`, still completes, as does the document half before it.
    let Some(path) = path_with_an_unqualified_ffmpeg("cli-unqualified-validate-path") else {
        return;
    };
    let dir = scratch_dir("cli-unqualified-validate");
    let project = dir.join("p.montagent.json");
    std::fs::write(
        &project,
        r##"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"photos","layer":1,"elements":[{"id":"photo-06","type":"image","start":0,"end":1000,"source":"images/05.png","x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","gravity":"bottom"}]}]}"##,
    )
    .expect("write project");

    let out = Command::new(binary())
        .args(["validate", project.to_str().unwrap()])
        .env("PATH", &path)
        .output()
        .expect("run montagent");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(70), "{stdout}");
    assert!(stdout.contains("E-TOOL-UNSUPPORTED"), "{stdout}");
    assert!(!stdout.contains("E-TOOL-MISSING"), "it was found: {stdout}");
    assert!(
        stdout.contains("ffmpeg 7.1 or newer"),
        "the floor is named: {stdout}"
    );
    assert!(
        stdout.contains("-/filter_complex"),
        "the capability is named: {stdout}"
    );
    assert!(
        stdout.contains(&path.join("ffmpeg").display().to_string()),
        "the resolved path is named: {stdout}"
    );
    assert!(
        stdout.contains("Unrecognized option"),
        "ffmpeg's own words: {stdout}"
    );
    assert!(
        stdout.contains("E-RETIRED-KEY"),
        "the document half survives: {stdout}"
    );
    assert!(
        stdout.contains("E-SOURCE-MISSING"),
        "the disk half ran on ffprobe alone: {stdout}"
    );
}

#[cfg(unix)]
#[test]
fn cli_render_on_an_ffmpeg_below_the_floor_refuses_before_the_encoder() {
    let Some(path) = path_with_an_unqualified_ffmpeg("cli-unqualified-render-path") else {
        return;
    };
    let dir = scratch_dir("cli-unqualified-render");
    let project = dir.join("p.montagent.json");
    let output = dir.join("out.mp4");
    std::fs::write(
        &project,
        format!(
            r##"{{"frame":{{"width":64,"height":64}},"fps":25,"output":{},"tracks":[{{"name":"shapes","layer":1,"elements":[{{"id":"box","type":"rect","start":0,"end":200,"x":0,"y":0,"origin":"top-left","width":64,"height":64,"fill":"#FF0000"}}]}}]}}"##,
            serde_json::to_string(output.to_str().unwrap()).unwrap()
        ),
    )
    .expect("write project");

    let out = Command::new(binary())
        .args(["render", project.to_str().unwrap()])
        .env("PATH", &path)
        .output()
        .expect("run montagent");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(70), "{stdout}");
    assert!(stdout.contains("E-TOOL-UNSUPPORTED"), "{stdout}");
    assert!(!output.exists(), "nothing was written: {stdout}");
}

#[test]
fn cli_frame_reaches_the_verb_and_writes_the_picture() {
    // One test per subcommand, asserting argv reaches the right core call and the exit code
    // is right (#168). Everything the picture *is* — the default encoding, the half scale,
    // the crop — is asserted in the core, at seam 1.
    let dir = scratch_dir("cli-frame");
    let out_file = dir.join("look.jpg");
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let out = montagent(&[
        "frame",
        fixture.to_str().unwrap(),
        "--at",
        "11000",
        "--out",
        out_file.to_str().unwrap(),
    ]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("FRAME  at 11000"), "{}", out.stdout);
    // ADR-0011: the `query --at` block prints alongside the image, unconditionally.
    assert!(
        out.stdout.contains("QUERY  the resolved stack at 11000"),
        "{}",
        out.stdout
    );
    let written = std::fs::read(&out_file).expect("the picture was written");
    assert_eq!(&written[..2], &[0xFF, 0xD8], "JPEG by default");
}

#[test]
fn cli_frame_offers_no_flag_that_suppresses_the_caption() {
    // ADR-0011 makes the block unconditional, so the assertion is about the *surface*: an
    // agent that could ask for the picture alone would be able to attribute a defect to the
    // wrong element, and the way to make that impossible is for there to be no such flag.
    let out = montagent(&["frame", "--help"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    for absent in [
        "--no-query",
        "--quiet",
        "--describe",
        "--image-only",
        "--verbose",
    ] {
        assert!(
            !out.stdout.contains(absent),
            "`frame` advertises `{absent}`: {}",
            out.stdout
        );
    }
    for present in ["--at", "--out", "--crop", "--full", "--png", "--json"] {
        assert!(
            out.stdout.contains(present),
            "`frame` does not advertise `{present}`: {}",
            out.stdout
        );
    }
}

#[test]
fn cli_render_puts_the_result_on_stdout_and_progress_on_stderr() {
    // One test per subcommand, asserting argv reaches the right core call and the exit
    // code is right (#168). Everything the render *is* — the refusal, the atomic write,
    // the audio — is asserted in the core, at seam 1. What is the adapter's alone is
    // ADR-0011's split: "the machine-readable result on stdout; progress on stderr".
    if montagent_core::media::tools::resolve().is_err() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }
    let dir = scratch_dir("cli-render");
    let project = dir.join("p.montagent.json");
    std::fs::write(
        &project,
        "{\n  \"frame\": {\"width\": 200, \"height\": 200},\n  \"fps\": 25,\n  \
         \"duration\": 200,\n  \"output\": \"out/p.mp4\",\n  \"tracks\": [\n    {\"name\": \
         \"only\", \"layer\": 0, \"elements\": [\n      {\"id\": \"card\", \"type\": \
         \"rect\", \"start\": 0, \"end\": 200, \"x\": 100, \"y\": 100, \"width\": 100, \
         \"height\": 100, \"fill\": \"#FF0000\"}\n    ]}\n  ]\n}\n",
    )
    .unwrap();
    let out = montagent(&["render", project.to_str().unwrap(), "--json"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not the JSON result alone ({e}):\n{}", out.stdout));
    assert_eq!(json["render"]["frames"], 5);
    assert_eq!(json["render"]["duration_ms"], 200);
    assert!(json["render"]["realtime"].as_f64().is_some());
    assert!(dir.join("out/p.mp4").is_file());
    // Progress is stderr's, coarse, and never on stdout.
    assert!(out.stderr.contains("render  0/5 frames"), "{}", out.stderr);
    assert!(out.stderr.contains("render  5/5 frames"), "{}", out.stderr);
    assert!(!out.stdout.contains("frames  "), "{}", out.stdout);

    // The text form: the block, then the findings, then the footer.
    let out = montagent(&["render", project.to_str().unwrap()]);
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("RENDER  "), "{}", out.stdout);
    assert!(out.stdout.contains("NOT CHECKED"), "{}", out.stdout);
}

#[test]
fn cli_render_on_a_project_with_an_error_is_exit_1_and_writes_nothing() {
    let dir = scratch_dir("cli-render-refused");
    let project = dir.join("p.montagent.json");
    std::fs::write(
        &project,
        "{\n  \"frame\": {\"width\": 200, \"height\": 200},\n  \"fps\": 25,\n  \
         \"duration\": 200,\n  \"output\": \"out/p.mp4\",\n  \"invented\": true,\n  \
         \"tracks\": []\n}\n",
    )
    .unwrap();
    let out = montagent(&["render", project.to_str().unwrap()]);

    assert_eq!(out.code, Some(1), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.starts_with("1 error"), "{}", out.stdout);
    assert!(!dir.join("out").exists());
}

#[test]
fn cli_render_offers_no_flag_that_skips_the_checks_or_scales_the_output() {
    // ADR-0006 makes the check engine the thing `render` runs before it draws anything,
    // and ADR-0021 makes the deliverable the one output that is never downsampled. The
    // assertion is about the *surface*: there is no such flag to pass.
    let out = montagent(&["render", "--help"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    for absent in [
        "--no-validate",
        "--skip-checks",
        "--force",
        "--proxy",
        "--scale",
        "--quiet",
    ] {
        assert!(
            !out.stdout.contains(absent),
            "`render` advertises `{absent}`: {}",
            out.stdout
        );
    }
    for present in ["--from", "--to", "--output", "--json", "--verbose"] {
        assert!(
            out.stdout.contains(present),
            "`render` does not advertise `{present}`: {}",
            out.stdout
        );
    }
}

#[test]
fn cli_preview_discloses_its_tier_and_keeps_adr_0011s_split() {
    // The adapter's half, as `render`'s test above: argv reaches the right core call, the
    // result is stdout's and progress is stderr's. What the ladder *does* is asserted in
    // the core, at seam 1 (`montagent-core/tests/preview.rs`).
    if montagent_core::media::tools::resolve().is_err() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }
    let dir = scratch_dir("cli-preview");
    let project = dir.join("p.montagent.json");
    std::fs::write(
        &project,
        "{\n  \"frame\": {\"width\": 2000, \"height\": 1000},\n  \"fps\": 25,\n  \
         \"duration\": 200,\n  \"output\": \"out/p.mp4\",\n  \"tracks\": [\n    {\"name\": \
         \"only\", \"layer\": 0, \"elements\": [\n      {\"id\": \"card\", \"type\": \
         \"rect\", \"start\": 0, \"end\": 200, \"x\": 1000, \"y\": 500, \"width\": 100, \
         \"height\": 100, \"fill\": \"#FF0000\"}\n    ]}\n  ]\n}\n",
    )
    .unwrap();
    let out = montagent(&["preview", project.to_str().unwrap(), "--json"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    let json: serde_json::Value = serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not the JSON result alone ({e}):\n{}", out.stdout));
    // The 720p target, long edge capped at 1280 (ADR-0046), and the disclosure with it.
    assert_eq!(json["preview"]["width"], 1280);
    assert_eq!(json["preview"]["height"], 640);
    assert_eq!(json["preview"]["tier"]["name"], "720p");
    assert_eq!(json["preview"]["tier"]["degraded"], false);
    // The deliverable is untouched: a preview is never it.
    assert!(!dir.join("out/p.mp4").exists());
    assert!(dir.join("out/p.preview.0-200.mp4").is_file());

    assert!(out.stderr.contains("preview  0/5 frames"), "{}", out.stderr);
    assert!(!out.stdout.contains("frames  "), "{}", out.stdout);

    // The text form states the tier above the file's own numbers.
    let out = montagent(&["preview", project.to_str().unwrap()]);
    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    assert!(out.stdout.contains("PREVIEW  "), "{}", out.stdout);
    assert!(out.stdout.contains("tier        720p"), "{}", out.stdout);
    assert!(out.stdout.contains("NOT CHECKED"), "{}", out.stdout);
}

#[test]
fn cli_preview_offers_the_escape_hatch_and_no_flag_that_names_a_tier() {
    // The target, the ladder and the budget are the ADRs' numbers. What the caller gets to
    // say is whether it wants the proxy at all (`--full`, ADR-0021's escape hatch); a flag
    // naming a resolution would be a caller-specified proxy resolution, which ADR-0067
    // records as *not* admitted today.
    //
    // `--clock` is in the list for the same reason: `Clock::Stated` is a core-library test
    // seam and not adapter surface, both adapters pass `Clock::Scrub`, and admitting a
    // caller-chosen budget is a decision ADR-0078 explicitly does not take.
    let out = montagent(&["preview", "--help"]);

    assert_eq!(out.code, Some(0), "{}{}", out.stdout, out.stderr);
    for absent in [
        "--tier",
        "--resolution",
        "--proxy",
        "--scale",
        "--height",
        "--budget",
        "--clock",
        "--degrade",
    ] {
        assert!(
            !out.stdout.contains(absent),
            "`preview` advertises `{absent}`: {}",
            out.stdout
        );
    }
    for present in [
        "--from",
        "--to",
        "--output",
        "--full",
        "--json",
        "--verbose",
    ] {
        assert!(
            out.stdout.contains(present),
            "`preview` does not advertise `{present}`: {}",
            out.stdout
        );
    }
}

#[test]
fn cli_help_is_not_a_failure() {
    let out = montagent(&["--help"]);
    assert_eq!(out.code, Some(0));
    assert!(out.stdout.contains("validate"), "{}", out.stdout);
}

// ---- MCP ----------------------------------------------------------------------------

#[test]
fn mcp_validate_on_a_header_only_project_is_a_clean_report() {
    let project = scratch("mcp-clean", "clean.montagent.json", HEADER_ONLY);
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap()}
            }),
        ),
    ]);

    let call = session.get(&3).expect("a result for tools/call");
    let text = call["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.starts_with("0 errors"), "{text}");
    assert!(text.contains("NOT CHECKED"), "{text}");
    assert_eq!(call["result"]["isError"], false);
}

#[test]
fn mcp_does_not_advertise_probe() {
    // ADR-0011: `probe` is CLI-only, and the asymmetry is the point. "Every MCP tool
    // schema occupies the agent's context and degrades tool selection on every turn,
    // including turns with nothing to do with video. A CLI subcommand costs nothing until
    // invoked." Asserted here because the cost is paid on turns this suite cannot see.
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
    ]);

    let tools = session.get(&2).expect("a result for tools/list")["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or("?").to_string())
        .collect::<Vec<_>>();

    assert!(tools.iter().any(|name| name == "validate"), "{tools:?}");
    assert!(!tools.iter().any(|name| name == "probe"), "{tools:?}");
}

#[test]
fn the_mcp_surface_is_exactly_nine_tools_and_preview_is_the_ninth() {
    // ADR-0011's table listed eight and spec #168's title says nine; ADR-0078 (#295) settles
    // it by giving the table a `preview` row. The count is asserted against the router
    // rather than restated in prose, because ADR-0011's own opening sentence ("nine verbs
    // and two resources", against a table of eleven) is what a prose count is worth.
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
    ]);

    let mut tools = session.get(&2).expect("a result for tools/list")["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or("?").to_string())
        .collect::<Vec<_>>();
    tools.sort();

    assert_eq!(
        tools,
        vec![
            "compare",
            "create_project",
            "frame",
            "measure",
            "preview",
            "query",
            "render",
            "shift",
            "validate",
        ],
        "the MCP surface changed without an ADR"
    );
}

#[test]
fn mcp_does_not_advertise_timeline() {
    // ADR-0011's unequal split, and #195's second acceptance criterion. `timeline` is the
    // *human's* wide view; ADR-0031 measured that an agent's overview need not be spatial
    // and noted the agent already reaches these facts through `query`. Advertising it would
    // spend the agent's context on every turn to duplicate a verb it has.
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
    ]);

    let tools = session.get(&2).expect("a result for tools/list")["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or("?").to_string())
        .collect::<Vec<_>>();

    assert!(!tools.iter().any(|name| name == "timeline"), "{tools:?}");
}

#[test]
fn mcp_validate_advertises_the_schema_it_enforces() {
    let project = scratch("mcp-schema", "clean.montagent.json", HEADER_ONLY);
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        // The schema says `project` is required and the other two are optional; calling
        // with `project` alone must therefore be accepted.
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap()}
            }),
        ),
        // …and a call missing the one required property must be rejected rather than
        // silently validating something else.
        request(
            4,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {}
            }),
        ),
    ]);

    let tools = &session[&2]["result"]["tools"];
    let validate = tools
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "validate")
        .expect("a `validate` tool");

    let schema = &validate["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["required"], serde_json::json!(["project"]));
    for property in ["project", "json", "verbose"] {
        assert!(
            !schema["properties"][property].is_null(),
            "the schema advertises `{property}`"
        );
    }

    assert_eq!(session[&3]["result"]["isError"], false);
    let rejected = &session[&4];
    assert!(
        rejected.get("error").is_some() || rejected["result"]["isError"] == true,
        "a call missing `project` must be rejected: {rejected}"
    );
}

#[test]
fn a_validate_that_could_not_parse_the_file_is_an_mcp_tool_failure() {
    // ADR-0083: `E-PARSE` is `NotAboutDocument` (ADR-0073) — the bytes never became a
    // document for `validate` to have an opinion about — so `isError` is set here, the
    // same way it is for a malformed call. This is different from a validate that *did*
    // run and found document-level `error` findings (see the `render` MCP test's exit-1
    // case), which stays `success` because the findings are still an answer about the
    // project.
    let project = scratch(
        "mcp-errors",
        "broken.montagent.json",
        "{\n  \"fps\": ,\n}\n",
    );
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap()}
            }),
        ),
    ]);

    let call = &session[&2];
    assert_eq!(call["result"]["isError"], true, "{call}");
    assert!(
        call["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("E-PARSE")
    );
}

#[test]
fn mcp_rejects_a_bad_call_with_a_finding_like_every_other_surface() {
    // ADR-0011: "An error is a finding. Same objects and same stable codes as ADR-0006,
    // including for invocation errors, so there is exactly one thing to parse across the
    // surface." Asserting only that a bad call is *rejected* is what let the SDK's own
    // raw deserialisation message through here while the CLI answered with a finding.
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": 123}
            }),
        ),
    ]);

    let call = &session[&2];
    let text = call["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered report, not a bare SDK error");
    assert!(text.contains("E-INVOCATION"), "{text}");
    assert!(text.contains("NOT CHECKED"), "{text}");
    assert_eq!(
        call["result"]["isError"], true,
        "the client is also told the call did not run: {call}"
    );
}

#[test]
fn mcp_query_advertises_the_schema_it_enforces_and_answers_all_three_modes() {
    // ADR-0011 puts `query` on both surfaces: it is what an agent calls in the loop, and
    // the cost argument that keeps `probe`, `fmt` and `timeline` off MCP does not apply to
    // a verb the agent needs on every edit.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let project = fixture.to_str().unwrap();
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "query",
                "arguments": {"project": project, "from": 17000, "to": 19000}
            }),
        ),
        request(
            4,
            "tools/call",
            serde_json::json!({
                "name": "query",
                "arguments": {"project": project, "where": "track = sentence-text", "census": "y"}
            }),
        ),
        // The one required property missing — rejected rather than answered about nothing.
        request(
            5,
            "tools/call",
            serde_json::json!({"name": "query", "arguments": {"from": 0, "to": 1}}),
        ),
        request(
            6,
            "tools/call",
            serde_json::json!({
                "name": "query",
                "arguments": {"project": project, "at": 6205}
            }),
        ),
    ]);

    let query = session[&2]["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .find(|tool| tool["name"] == "query")
        .expect("a `query` tool")
        .clone();
    let schema = &query["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["required"], serde_json::json!(["project"]));
    for property in ["project", "at", "from", "to", "where", "census", "json"] {
        assert!(
            !schema["properties"][property].is_null(),
            "the schema advertises `{property}`: {schema}"
        );
    }
    // `verbose` would cost the agent context on every turn and change nothing: `query` has
    // no informational findings to expand.
    assert!(schema["properties"]["verbose"].is_null(), "{schema}");

    let cuts = session[&3]["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered answer");
    assert!(
        cuts.contains("QUERY  cut list over [17000, 19000)"),
        "{cuts}"
    );
    let matches = session[&4]["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered answer");
    assert!(matches.contains("census y: 5 at 1537"), "{matches}");

    let resolved = session[&6]["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered answer");
    assert!(
        resolved.contains("QUERY  the resolved stack at 6205"),
        "{resolved}"
    );
    // ADR-0011's own worked number, through the surface an agent actually calls.
    assert!(
        resolved.contains("scale [1.016997, 1.016997]"),
        "{resolved}"
    );

    let rejected = &session[&5];
    let text = rejected["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered report, not a bare SDK error");
    assert!(text.contains("E-INVOCATION"), "{text}");
    assert_eq!(rejected["result"]["isError"], true, "{rejected}");
}

#[test]
fn mcp_measure_advertises_the_schema_it_enforces_and_answers() {
    // ADR-0011 puts `measure` on both surfaces: ADR-0007 makes it *mandatory in the text
    // authoring loop*, which is the agent's loop and not a human's.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let project = fixture.to_str().unwrap();
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "measure",
                "arguments": {
                    "project": project,
                    "element": {
                        "type": "text", "font": "brand", "size": 55, "line_height": 1.1,
                        "y": 1537, "origin": "center",
                        "runs": [{"text": "I hang cobwebs over the door."}]
                    },
                    "json": true
                }
            }),
        ),
        // The element missing — rejected rather than answered about nothing.
        request(
            4,
            "tools/call",
            serde_json::json!({"name": "measure", "arguments": {"project": project}}),
        ),
        // ADR-0035's second mode: a time instead of an element.
        request(
            5,
            "tools/call",
            serde_json::json!({
                "name": "measure",
                "arguments": {"project": project, "at": 3041, "json": true}
            }),
        ),
    ]);

    let schema = session[&2]["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .find(|tool| tool["name"] == "measure")
        .expect("a `measure` tool")["inputSchema"]
        .clone();
    assert_eq!(schema["type"], "object");
    // `element` and `at` are each optional — ADR-0035 makes `at` a second, mutually
    // exclusive input mode, so neither can be the one required argument.
    assert_eq!(
        schema["required"],
        serde_json::json!(["project"]),
        "{schema}"
    );
    // The argument is the element itself, not a field name and not an id — ADR-0024
    // requires `measure` to work for an element being authored for the first time.
    assert!(!schema["properties"]["element"].is_null(), "{schema}");
    assert!(!schema["properties"]["at"].is_null(), "{schema}");
    for absent in ["id", "text", "font", "size", "verbose"] {
        assert!(
            schema["properties"][absent].is_null(),
            "the schema advertises no `{absent}`: {schema}"
        );
    }

    let answered: serde_json::Value = serde_json::from_str(
        session[&3]["result"]["content"][0]["text"]
            .as_str()
            .expect("a rendered answer"),
    )
    .expect("`json: true` returns the canonical JSON");
    // ADR-0007's own worked example, reaching the agent through the surface it calls.
    assert_eq!(answered["measure"]["block_top"], 1506.75);
    assert_eq!(answered["measure"]["block_bottom"], 1567.25);

    // Neither input mode named — a verb-level rejection, `query`'s missing-mode pattern:
    // the call reached the verb, which answered `E-INVOCATION`. ADR-0083: `isError`
    // tracks `NotAboutDocument` regardless of which layer produced the finding, and
    // `E-INVOCATION` is `NotAboutDocument` (ADR-0073) whether it came from a malformed
    // call or, as here, a verb that ran and found its own arguments incoherent.
    let rejected = &session[&4];
    let text = rejected["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered report, not a bare SDK error");
    assert!(text.contains("E-INVOCATION"), "{text}");
    assert_eq!(rejected["result"]["isError"], true, "{rejected}");

    // The fixture is 25fps: 3041ms's nearest sampled instant at-or-before it is frame 76,
    // at 3040ms — the same numbers the core arithmetic and the CLI test both check.
    let by_instant: serde_json::Value = serde_json::from_str(
        session[&5]["result"]["content"][0]["text"]
            .as_str()
            .expect("a rendered answer"),
    )
    .expect("`json: true` returns the canonical JSON");
    assert_eq!(by_instant["measure"]["mode"], "at");
    assert_eq!(by_instant["measure"]["frame"], 76);
    assert_eq!(by_instant["measure"]["nearest"], 3040.0);
}

#[test]
fn mcp_measure_batch_advertises_and_answers_the_same_way_the_cli_does() {
    // #317: the same batch capability, the same semantics, on both adapters — exercised
    // here against the identical fixture the CLI batch tests use, so the two surfaces are
    // checked against one shared ground truth rather than two independently plausible ones.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let project = fixture.to_str().unwrap();
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "measure",
                "arguments": {
                    "project": project,
                    "elements": [
                        {"font": "brand", "size": 20, "runs": [{"text": "ok"}]},
                        {"font": "not-a-declared-key", "size": 20, "runs": [{"text": "bad"}]}
                    ],
                    "json": true
                }
            }),
        ),
        request(
            4,
            "tools/call",
            serde_json::json!({
                "name": "measure",
                "arguments": {"project": project, "all": true, "json": true}
            }),
        ),
    ]);

    let schema = session[&2]["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .find(|tool| tool["name"] == "measure")
        .expect("a `measure` tool")["inputSchema"]
        .clone();
    assert!(!schema["properties"]["elements"].is_null(), "{schema}");
    assert!(!schema["properties"]["all"].is_null(), "{schema}");

    // A batch with one bad slot answers `success`, not `isError` — the whole call is not a
    // refusal (ADR-0083's rule extended to a batch's own slots).
    let batch = &session[&3];
    assert_ne!(
        batch["result"]["isError"],
        serde_json::json!(true),
        "{batch}"
    );
    let answered: serde_json::Value = serde_json::from_str(
        batch["result"]["content"][0]["text"]
            .as_str()
            .expect("a rendered answer"),
    )
    .expect("`json: true` returns the canonical JSON");
    let results = answered["measure"]["results"].as_array().expect("a batch");
    assert_eq!(results.len(), 2);
    assert!(results[0]["error"].is_null());
    assert_eq!(results[1]["error"]["code"], "E-INVOCATION");

    let all = &session[&4];
    let answered_all: serde_json::Value = serde_json::from_str(
        all["result"]["content"][0]["text"]
            .as_str()
            .expect("a rendered answer"),
    )
    .expect("`json: true` returns the canonical JSON");
    let all_results = answered_all["measure"]["results"]
        .as_array()
        .expect("a batch");
    assert!(
        !all_results.is_empty(),
        "the fixture has text elements: {all_results:?}"
    );
}

#[test]
fn mcp_frame_advertises_the_schema_it_enforces_and_hands_back_an_image() {
    // ADR-0011 puts `frame` on both surfaces, and this one is where it matters: a picture
    // that came back as a path would be a picture an agent cannot see.
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let project = fixture.to_str().unwrap();
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "frame",
                "arguments": {"project": project, "at": 11000, "json": true}
            }),
        ),
        // No instant — rejected rather than answered about an instant nobody named.
        request(
            4,
            "tools/call",
            serde_json::json!({"name": "frame", "arguments": {"project": project}}),
        ),
    ]);

    let schema = session[&2]["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .find(|tool| tool["name"] == "frame")
        .expect("a `frame` tool")["inputSchema"]
        .clone();
    assert_eq!(schema["type"], "object");
    assert_eq!(
        schema["required"],
        serde_json::json!(["project", "at"]),
        "{schema}"
    );
    for present in ["crop", "full", "png", "json"] {
        assert!(
            !schema["properties"][present].is_null(),
            "the schema advertises no `{present}`: {schema}"
        );
    }
    // There is no argument that suppresses the caption, and none that writes a file: the
    // first is ADR-0011's "unconditionally", the second is the CLI's alone.
    for absent in ["out", "verbose", "describe", "quiet"] {
        assert!(
            schema["properties"][absent].is_null(),
            "the schema advertises `{absent}`: {schema}"
        );
    }

    let content = session[&3]["result"]["content"]
        .as_array()
        .expect("content blocks");
    let answered: serde_json::Value =
        serde_json::from_str(content[0]["text"].as_str().expect("a rendered answer"))
            .expect("`json: true` returns the canonical JSON");
    assert_eq!(answered["frame"]["encoding"], "jpeg");
    assert_eq!(
        answered["query"]["mode"], "at",
        "the caption travels with it"
    );

    let image = &content[1];
    assert_eq!(image["type"], "image", "{image}");
    assert_eq!(image["mimeType"], "image/jpeg");
    let data = image["data"].as_str().expect("base64 bytes");
    assert_eq!(
        data.len(),
        answered["frame"]["bytes"]
            .as_u64()
            .expect("a byte count")
            .div_ceil(3) as usize
            * 4,
        "the block carries the whole picture, base64-encoded"
    );

    let rejected = &session[&4];
    let text = rejected["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered report, not a bare SDK error");
    assert!(text.contains("E-INVOCATION"), "{text}");
    assert_eq!(rejected["result"]["isError"], true, "{rejected}");
}

#[test]
fn mcp_preview_advertises_the_escape_hatch_and_no_tier_argument() {
    if montagent_core::media::tools::resolve().is_err() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }
    let dir = scratch_dir("mcp-preview");
    let project = dir.join("p.montagent.json");
    std::fs::write(
        &project,
        "{\n  \"frame\": {\"width\": 2000, \"height\": 1000},\n  \"fps\": 25,\n  \
         \"duration\": 200,\n  \"output\": \"out/p.mp4\",\n  \"tracks\": [\n    {\"name\": \
         \"only\", \"layer\": 0, \"elements\": [\n      {\"id\": \"card\", \"type\": \
         \"rect\", \"start\": 0, \"end\": 200, \"x\": 1000, \"y\": 500, \"width\": 100, \
         \"height\": 100, \"fill\": \"#FF0000\"}\n    ]}\n  ]\n}\n",
    )
    .unwrap();
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "preview",
                "arguments": {"project": project.to_str().unwrap(), "json": true}
            }),
        ),
    ]);

    let schema = session[&2]["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .find(|tool| tool["name"] == "preview")
        .expect("a `preview` tool")["inputSchema"]
        .clone();
    assert_eq!(
        schema["required"],
        serde_json::json!(["project"]),
        "{schema}"
    );
    for present in ["from", "to", "output", "full", "json", "verbose"] {
        assert!(
            !schema["properties"][present].is_null(),
            "no `{present}`: {schema}"
        );
    }
    // The target, the ladder and the budget are the ADRs' numbers, not the caller's —
    // `clock` included: `Clock::Stated` is a core-library test seam, never adapter surface,
    // and this schema is where that would leak first (ADR-0078).
    for absent in [
        "tier",
        "resolution",
        "proxy",
        "scale",
        "budget",
        "clock",
        "degrade",
    ] {
        assert!(
            schema["properties"][absent].is_null(),
            "advertises `{absent}`: {schema}"
        );
    }

    // And the answer carries the disclosure ADR-0021 makes mandatory.
    let result = &session[&3]["result"];
    assert_ne!(result["isError"], true, "{result}");
    let answered: serde_json::Value =
        serde_json::from_str(result["content"][0]["text"].as_str().expect("the result"))
            .expect("`json: true` returns the canonical JSON");
    assert_eq!(answered["tool"], "preview");
    assert_eq!(answered["preview"]["tier"]["name"], "720p");
    assert_eq!(answered["preview"]["width"], 1280);
    assert!(!dir.join("out/p.mp4").exists(), "never the deliverable");
}

#[test]
fn mcp_render_advertises_the_schema_it_enforces_and_answers_with_the_findings() {
    // ADR-0011 puts `render` on both surfaces. What this test asserts is the adapter's:
    // the schema advertised is the one enforced, a refused render is an answer rather
    // than a protocol error, and there is no argument that skips the checks.
    let dir = scratch_dir("mcp-render");
    let project = dir.join("p.montagent.json");
    std::fs::write(
        &project,
        "{\n  \"frame\": {\"width\": 200, \"height\": 200},\n  \"fps\": 25,\n  \
         \"duration\": 200,\n  \"output\": \"out/p.mp4\",\n  \"invented\": true,\n  \
         \"tracks\": []\n}\n",
    )
    .unwrap();
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "render",
                "arguments": {"project": project.to_str().unwrap(), "json": true}
            }),
        ),
        request(
            4,
            "tools/call",
            serde_json::json!({"name": "render", "arguments": {"from": 0}}),
        ),
    ]);

    let schema = session[&2]["result"]["tools"]
        .as_array()
        .expect("a tool list")
        .iter()
        .find(|tool| tool["name"] == "render")
        .expect("a `render` tool")["inputSchema"]
        .clone();
    assert_eq!(
        schema["required"],
        serde_json::json!(["project"]),
        "{schema}"
    );
    for present in ["from", "to", "output", "json", "verbose"] {
        assert!(
            !schema["properties"][present].is_null(),
            "no `{present}`: {schema}"
        );
    }
    for absent in ["skip_checks", "force", "proxy", "scale"] {
        assert!(
            schema["properties"][absent].is_null(),
            "advertises `{absent}`: {schema}"
        );
    }

    // Refused on the schema error, as an answer: the findings are the result (ADR-0006).
    let refused = &session[&3]["result"];
    assert_ne!(refused["isError"], true, "{refused}");
    let answered: serde_json::Value =
        serde_json::from_str(refused["content"][0]["text"].as_str().expect("the result"))
            .expect("`json: true` returns the canonical JSON");
    assert_eq!(answered["tool"], "render");
    assert_eq!(answered["exit_code"], 1);
    assert!(answered["render"].is_null());
    assert!(!dir.join("out").exists());

    let rejected = &session[&4];
    let text = rejected["result"]["content"][0]["text"]
        .as_str()
        .expect("a rendered report, not a bare SDK error");
    assert!(text.contains("E-INVOCATION"), "{text}");
    assert_eq!(rejected["result"]["isError"], true, "{rejected}");
}

#[test]
fn mcp_query_with_no_question_answers_with_a_finding_rather_than_a_protocol_error() {
    // The verb ran and rejected its arguments, which is an answer about the call: same
    // code, same object, same thing to parse as every other surface (ADR-0011).
    let fixture = fixture_dir().join("en-halloween-decorating.montagent.json");
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "query",
                "arguments": {"project": fixture.to_str().unwrap()}
            }),
        ),
    ]);

    let call = &session[&2];
    let text = call["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("E-INVOCATION"), "{text}");
    assert!(text.contains("needs a question"), "{text}");
    assert!(text.contains("NOT CHECKED"), "{text}");
}

#[test]
fn mcp_validate_json_returns_the_canonical_json_instead_of_the_text() {
    let project = scratch("mcp-json", "clean.montagent.json", HEADER_ONLY);
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "validate",
                "arguments": {"project": project.to_str().unwrap(), "json": true}
            }),
        ),
    ]);

    let text = session[&2]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(text).expect("the canonical JSON alone");
    assert_eq!(json["exit_code"], 0);
    assert!(
        !text.contains("NOT CHECKED\n  This file"),
        "not both forms: {text}"
    );
}

#[test]
fn mcp_create_project_advertises_the_schema_it_enforces_and_returns_the_new_states_findings() {
    let dir = scratch_dir("mcp-create-project");
    let project = dir.join("new.montagent.json");
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "tools/list", serde_json::json!({})),
        request(
            3,
            "tools/call",
            serde_json::json!({
                "name": "create_project",
                "arguments": {
                    "project": project.to_str().unwrap(),
                    "frame": {"width": 1080, "height": 1920},
                    "fps": 25,
                    "background": "#FBF3E3",
                    "duration": 65216,
                    "output": "out/clean.mp4"
                }
            }),
        ),
        // Every required property missing but `project` — rejected rather than scaffolded
        // against invented numbers.
        request(
            4,
            "tools/call",
            serde_json::json!({
                "name": "create_project",
                "arguments": {"project": dir.join("other.montagent.json").to_str().unwrap()}
            }),
        ),
    ]);

    let create = session[&2]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "create_project")
        .expect("a `create_project` tool")
        .clone();
    let schema = &create["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(
        schema["required"],
        serde_json::json!(["project", "frame", "fps"]),
        "the two the format requires, plus where to write it"
    );
    for property in ["background", "duration", "output", "json", "verbose"] {
        assert!(
            !schema["properties"][property].is_null(),
            "the schema advertises `{property}`"
        );
    }

    // ADR-0011's write-tool invariant is that a write tool takes "a schema-shaped object",
    // never a list of field names. The header fields this tool advertises must therefore be
    // the header fields the *published* schema defines, spelled the same way and nested the
    // same way — an adapter that flattened `frame` into `width`/`height` would be a second
    // shape for the agent to learn, and nothing but this assertion would notice.
    let published = montagent_core::schema::generate();
    let header = published["properties"]
        .as_object()
        .expect("the project header's properties");
    let advertised: Vec<&str> = schema["properties"]
        .as_object()
        .expect("the tool's properties")
        .keys()
        .map(String::as_str)
        // `project`, `json` and `verbose` are the call's own — where to write, and which
        // wire form to answer in — and are not fields of the document.
        .filter(|name| !["project", "json", "verbose"].contains(name))
        .collect();
    for name in &advertised {
        assert!(
            header.contains_key(*name),
            "`{name}` is not a field of the project header: {:?}",
            header.keys().collect::<Vec<_>>()
        );
    }
    assert_eq!(
        schema["properties"]["frame"]["$ref"]
            .as_str()
            .map(|r| r.rsplit('/').next().unwrap_or(r)),
        Some("FrameParam"),
        "`frame` is the nested object the format defines, not a flattened pair: {}",
        schema["properties"]["frame"]
    );

    let call = &session[&3];
    assert_eq!(call["result"]["isError"], false, "{call}");
    let text = call["result"]["content"][0]["text"].as_str().unwrap();
    // ADR-0011's write-tool invariant, at the surface it was argued for: "every write tool
    // returns the new state's findings, never `ok`". Not a receipt, not a path echoed back.
    assert!(text.starts_with("0 errors"), "{text}");
    assert!(text.contains("NOT CHECKED"), "{text}");
    assert_eq!(
        std::fs::read_to_string(&project).unwrap(),
        "{\n  \"frame\": {\"width\": 1080, \"height\": 1920},\n  \"fps\": 25,\n  \
         \"background\": \"#FBF3E3\",\n  \"duration\": 65216,\n  \"output\": \
         \"out/clean.mp4\",\n  \"tracks\": []\n}\n"
    );

    let rejected = &session[&4];
    assert!(
        rejected.get("error").is_some() || rejected["result"]["isError"] == true,
        "a call missing `width`/`height`/`fps` must be rejected: {rejected}"
    );
    assert!(
        !dir.join("other.montagent.json").exists(),
        "and must write nothing"
    );
}

#[test]
fn mcp_create_project_onto_an_existing_file_sets_is_error_and_changes_nothing() {
    // #313, resolved by ADR-0083: `E-PROJECT-EXISTS` is `NotAboutDocument` (ADR-0073,
    // ADR-0080) — the project being scaffolded does not exist, so there is no document
    // to have an opinion about — and `isError` now tracks that classification on every
    // MCP tool, not just a bad-invocation deserialisation failure.
    let dir = scratch_dir("mcp-create-project-exists");
    let project = dir.join("mine.montagent.json");
    let mine = "{\"an afternoon\": \"of work\"}\n";
    std::fs::write(&project, mine).unwrap();

    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(
            2,
            "tools/call",
            serde_json::json!({
                "name": "create_project",
                "arguments": {
                    "project": project.to_str().unwrap(),
                    "frame": {"width": 1080, "height": 1920},
                    "fps": 25
                }
            }),
        ),
    ]);

    let call = &session[&2];
    assert_eq!(call["result"]["isError"], true, "{call}");
    let text = call["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("E-PROJECT-EXISTS"), "{text}");
    assert!(
        !text.to_lowercase().contains("repair"),
        "a NotAboutDocument code prints no repair block: {text}"
    );
    assert_eq!(
        std::fs::read_to_string(&project).unwrap(),
        mine,
        "the file is untouched, to the byte"
    );
}

#[test]
fn mcp_publishes_the_schema_and_the_format_docs_as_resources() {
    // ADR-0011: "the schema and format docs as resources. Discoverability is the schema's
    // job, and a resource costs no tool slot. This is what makes 'how does the agent know
    // how to edit project.json' answerable at all." Asserted at the protocol, because the
    // cost this buys is paid on turns this suite cannot see.
    let session = mcp_session(&[
        handshake(1),
        notification("notifications/initialized"),
        request(2, "resources/list", serde_json::json!({})),
        request(
            3,
            "resources/read",
            serde_json::json!({"uri": "montagent://schema.json"}),
        ),
        request(
            4,
            "resources/read",
            serde_json::json!({"uri": "montagent://format.md"}),
        ),
        request(
            5,
            "resources/read",
            serde_json::json!({"uri": "montagent://nothing-here"}),
        ),
        request(6, "tools/list", serde_json::json!({})),
    ]);

    let listed: Vec<String> = session[&2]["result"]["resources"]
        .as_array()
        .expect("a resource list")
        .iter()
        .map(|resource| resource["uri"].as_str().unwrap_or("?").to_string())
        .collect();
    assert_eq!(
        listed,
        vec![
            "montagent://schema.json".to_string(),
            "montagent://format.md".to_string()
        ]
    );

    // The schema resource is the *generated* artifact, not the committed copy of it: the
    // two are compared in the core's own suite, and what is served here is the function.
    let served = session[&3]["result"]["contents"][0]["text"]
        .as_str()
        .expect("the schema's bytes");
    assert_eq!(served, montagent_core::schema::generated_bytes());
    assert_eq!(
        session[&3]["result"]["contents"][0]["mimeType"],
        "application/schema+json"
    );

    let docs = session[&4]["result"]["contents"][0]["text"]
        .as_str()
        .expect("the format docs");
    assert!(docs.contains("Presence is content"), "{docs}");
    assert_eq!(
        session[&4]["result"]["contents"][0]["mimeType"],
        "text/markdown"
    );

    assert!(
        session[&5].get("error").is_some(),
        "an unpublished URI is an error, not an empty document: {}",
        session[&5]
    );

    // And neither of them cost a tool slot, which is the whole reason they are resources.
    let tools: Vec<String> = session[&6]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or("?").to_string())
        .collect();
    assert!(
        !tools
            .iter()
            .any(|name| name.contains("schema") || name.contains("docs")),
        "{tools:?}"
    );
}

// ---- harness ------------------------------------------------------------------------

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn binary() -> PathBuf {
    // `CARGO_BIN_EXE_<name>` is set by cargo for integration tests of a binary crate.
    PathBuf::from(env!("CARGO_BIN_EXE_montagent"))
}

/// One `montagent` run, against a probe cache no other run shares.
///
/// Cold by construction: the sidecar (ADR-0069) is real, and a test that asserted a `CACHE`
/// block would otherwise pass once and then never again on the same machine.
fn montagent(args: &[&str]) -> Output {
    static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    montagent_caching(
        args,
        &scratch_dir(&format!("cache/{}-{n}", std::process::id())),
    )
}

/// The same, against a cache directory the caller owns — which is how two runs share one,
/// and the only way to ask whether anything survived between them.
fn montagent_caching(args: &[&str], cache: &Path) -> Output {
    let out = Command::new(binary())
        .args(args)
        .env(montagent_core::media::sidecar::CACHE_DIR_VAR, cache)
        .output()
        .expect("run montagent");
    Output {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// The `initialize` every session opens with.
fn handshake(id: u64) -> String {
    request(
        id,
        "initialize",
        serde_json::json!({
            "protocolVersion": "2026-07-28",
            "capabilities": {},
            "clientInfo": {"name": "montagent-tests", "version": "0"}
        }),
    )
}

fn request(id: u64, method: &str, params: serde_json::Value) -> String {
    serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

fn notification(method: &str) -> String {
    serde_json::json!({"jsonrpc": "2.0", "method": method}).to_string()
}

/// Drive the stdio server through one session, returning every response by its id.
fn mcp_session(messages: &[String]) -> std::collections::BTreeMap<u64, serde_json::Value> {
    let expected = messages.iter().filter(|m| m.contains("\"id\":")).count();

    let mut child = Command::new(binary())
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn the MCP server");

    let mut stdin = child.stdin.take().expect("stdin");
    for message in messages {
        writeln!(stdin, "{message}").expect("write a request");
    }
    stdin.flush().expect("flush");

    let mut responses = std::collections::BTreeMap::new();
    let reader = BufReader::new(child.stdout.take().expect("stdout"));
    for line in reader.lines() {
        let line = line.expect("read a response");
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("response is not JSON ({e}): {line}"));
        if let Some(id) = value["id"].as_u64() {
            responses.insert(id, value);
        }
        if responses.len() == expected {
            break;
        }
    }

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();

    assert_eq!(responses.len(), expected, "every request got a response");
    responses
}

/// The committed fixture's own directory — the only media this suite probes.
fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montagent-adapter-tests/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn scratch(name: &str, file: &str, body: &str) -> PathBuf {
    let path = scratch_dir(name).join(file);
    std::fs::write(&path, body).expect("write project");
    assert!(Path::new(&path).exists());
    path
}
