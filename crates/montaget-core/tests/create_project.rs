//! `create_project`: the scaffold, and the first place the write-tool invariant is
//! observable.

use montaget_core::report::ExitCode;
use montaget_core::verbs::create_project::{Scaffold, create_project};

mod common;

/// The scaffold story 1 of spec #168 describes: every optional header field stated, so the
/// file the agent gets back has the shape it would otherwise have had to invent.
fn full() -> Scaffold {
    Scaffold {
        width: 1080,
        height: 1920,
        fps: 25,
        background: Some("#FBF3E3".to_string()),
        duration: Some(65216),
        output: Some("out/halloween.mp4".to_string()),
    }
}

/// The two fields the format requires, and nothing else.
fn bare() -> Scaffold {
    Scaffold {
        width: 1080,
        height: 1920,
        fps: 25,
        background: None,
        duration: None,
        output: None,
    }
}

#[test]
fn the_scaffold_is_a_legal_project_in_the_canonical_convention() {
    let dir = common::tempdir(line!());
    let path = dir.join("new.montaget.json");

    let report = create_project(&path, &full());

    assert_eq!(
        report.exit_code(),
        ExitCode::Ok,
        "{:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
    // Canonical to the byte, including the key order, the one-line frame, the empty
    // `tracks` array on its own line and the trailing newline.
    assert_eq!(
        std::fs::read_to_string(&path).expect("the scaffolded file"),
        "{\n  \"frame\": {\"width\": 1080, \"height\": 1920},\n  \"fps\": 25,\n  \
         \"background\": \"#FBF3E3\",\n  \"duration\": 65216,\n  \"output\": \
         \"out/halloween.mp4\",\n  \"tracks\": []\n}\n"
    );
}

#[test]
fn the_scaffold_validates_clean() {
    let dir = common::tempdir(line!());
    let path = dir.join("new.montaget.json");

    create_project(&path, &full());

    // Asked a second time, through the verb whose answer is the standard — a scaffold that
    // needed `create_project`'s own report to look clean would not be a legal project.
    let report = montaget_core::validate(&path);
    let summary = report.summary();
    assert_eq!(
        (summary.error, summary.review, summary.note, summary.layout),
        (0, 0, 0, 0),
        "{:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}

#[test]
fn it_returns_the_new_states_findings_rather_than_ok() {
    // ADR-0011's write-tool invariant: "Every write tool returns the new state's findings,
    // never `ok`." The observable form of that is not a flag on the report — it is that the
    // report is the same object `validate` produces, about the file that now exists, with
    // the same boundary printed on it.
    let dir = common::tempdir(line!());
    let path = dir.join("new.montaget.json");

    let report = create_project(&path, &full());

    assert_eq!(report.tool, "create_project");
    assert_eq!(
        report.project.as_deref(),
        Some(path.display().to_string()).as_deref()
    );

    let rendered =
        montaget_core::wire::render(&report, montaget_core::Wire::Text { verbose: true });
    assert!(
        rendered.contains("NOT CHECKED") && rendered.contains("it cannot tell you"),
        "the scaffold's report carries `validate`'s own boundary, because it *is* \
         `validate`'s answer:\n{rendered}"
    );
    assert!(
        rendered.starts_with("0 errors"),
        "and its summary, rather than a receipt for the write:\n{rendered}"
    );

    // And the canonical JSON is a findings document like every other verb's.
    let json = montaget_core::wire::render(&report, montaget_core::Wire::Json);
    let json: serde_json::Value = serde_json::from_str(&json).expect("canonical JSON");
    assert_eq!(json["tool"], "create_project");
    assert!(json["findings"].is_array(), "{json}");
}

#[test]
fn a_field_the_agent_did_not_state_is_not_written_for_it() {
    // ADR-0030: omission and explicit-at-default are two spellings of *different*
    // declarations. A scaffold that helpfully wrote `"background": "#000000"` would be
    // authoring a declaration nobody made, and the author would then have to notice it.
    let dir = common::tempdir(line!());
    let path = dir.join("bare.montaget.json");

    let report = create_project(&path, &bare());
    assert_eq!(report.exit_code(), ExitCode::Ok);

    assert_eq!(
        std::fs::read_to_string(&path).expect("the scaffolded file"),
        "{\n  \"frame\": {\"width\": 1080, \"height\": 1920},\n  \"fps\": 25,\n  \
         \"tracks\": []\n}\n"
    );
}

#[test]
fn an_existing_file_is_never_overwritten() {
    let dir = common::tempdir(line!());
    let path = dir.join("mine.montaget.json");
    let mine = "{\"this\": \"is a whole afternoon of work\"}\n";
    std::fs::write(&path, mine).expect("write the file that is already there");

    let report = create_project(&path, &full());

    assert_eq!(report.exit_code(), ExitCode::Errors);
    assert_eq!(
        report
            .findings
            .iter()
            .map(|f| f.code.as_str())
            .collect::<Vec<_>>(),
        vec!["E-PROJECT-EXISTS"]
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("still there"),
        mine,
        "the file is untouched, to the byte"
    );
}

#[test]
fn a_colour_the_format_does_not_admit_is_a_bad_invocation_and_writes_nothing() {
    let dir = common::tempdir(line!());
    let path = dir.join("bad-colour.montaget.json");

    let report = create_project(
        &path,
        &Scaffold {
            // Lowercase, and a CSS-shaped three-digit shorthand would fail the same way.
            background: Some("#fbf3e3".to_string()),
            ..full()
        },
    );

    assert_eq!(report.exit_code(), ExitCode::BadInvocation);
    assert_eq!(report.findings[0].code, "E-INVOCATION");
    assert!(!path.exists(), "nothing is written on a bad invocation");
}

#[test]
fn a_parent_directory_that_does_not_exist_is_a_bad_invocation_not_an_internal_failure() {
    // Exit 70's next move is "retry or report", which is the wrong instruction for a
    // mistyped path. Exit 3's is "fix the command", which is exactly right.
    let dir = common::tempdir(line!());
    let path = dir.join("no/such/directory/new.montaget.json");

    let report = create_project(&path, &full());

    assert_eq!(report.exit_code(), ExitCode::BadInvocation);
    assert_eq!(report.findings[0].code, "E-INVOCATION");
}

#[test]
fn the_scaffold_leaves_no_temp_file_beside_itself() {
    // ADR-0011's atomic write is ticket 6's machinery, reused rather than reimplemented —
    // and its temp file is a sibling, so a run that forgot to rename would be visible here.
    let dir = common::tempdir(line!());
    let path = dir.join("new.montaget.json");

    create_project(&path, &full());

    let beside: Vec<String> = std::fs::read_dir(&dir)
        .expect("read the scratch directory")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(beside, vec!["new.montaget.json".to_string()]);
}

#[test]
fn the_scaffolds_key_order_comes_from_the_schema_rather_than_from_this_verb() {
    // ADR-0041: there is exactly one place the rule lives. The scaffold's order is the
    // published schema's property order for the project header, and asserting it against
    // that predicate rather than against a literal is what keeps the two from drifting.
    let dir = common::tempdir(line!());
    let path = dir.join("new.montaget.json");
    create_project(&path, &full());

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("the scaffolded file"))
            .expect("valid JSON");
    let written: Vec<&str> = written
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();

    let schema_order =
        montaget_core::layout::canonical_order(montaget_core::layout::Published::Project)
            .expect("the project header is a published type");
    let expected: Vec<&str> = schema_order
        .iter()
        .map(String::as_str)
        .filter(|key| written.contains(key))
        .collect();

    assert_eq!(written, expected);
}
