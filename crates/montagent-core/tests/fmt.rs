//! `fmt`: the convention becomes enforceable, and the atomic write every later write tool
//! reuses.

use montagent_core::layout::{self, Published};
use montagent_core::report::ExitCode;
use montagent_core::verbs::fmt::{self, Mode};
use montagent_core::write;
use std::path::{Path, PathBuf};

mod common;

/// The 154-line fixture, pretty-printed — the incident, in miniature. Every key is in
/// canonical order and every value is untouched; only the layout is wrong.
const PRETTY_PRINTED: &str = r##"{
  "frame": {
    "width": 1080,
    "height": 1920
  },
  "fps": 25,
  "duration": 4000,
  "tracks": [
    {
      "name": "photo",
      "layer": 10,
      "elements": [
        {
          "id": "photo-01",
          "type": "image",
          "start": 0,
          "end": 4000,
          "source": "images/01.png",
          "width": 1080,
          "height": 1300,
          "fit": "cover",
          "clip": [0, 0, 1080, 1300]
        }
      ]
    }
  ]
}
"##;

/// The same project written the way a scripted edit leaves it: keys reordered, layout
/// intact-ish. `type` before `id`, `fit` before `width`.
const KEYS_OUT_OF_ORDER: &str = r##"{
  "fps": 25,
  "frame": {"width": 1080, "height": 1920},
  "duration": 4000,
  "tracks": [
    {
      "layer": 10,
      "name": "photo",
      "elements": [
        {"type":"image","id":"photo-01","start":0,"end":4000,"source":"images/01.png","fit":"cover","width":1080,"height":1300,"clip":[0,0,1080,1300]}
      ]
    }
  ]
}
"##;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
}

fn codes(report: &montagent_core::report::Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

// ---- `--check` reports; write mode writes ------------------------------------------

#[test]
fn check_reports_what_a_rewrite_would_change_and_writes_nothing() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", PRETTY_PRINTED);

    let report = fmt::fmt(&path, Mode::Check);

    assert_eq!(codes(&report), ["L-LAYOUT"], "{:?}", report.findings);
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        PRETTY_PRINTED,
        "`--check` is non-destructive (ADR-0041)"
    );
    // ADR-0011: "Exit non-zero only on `error`", and ADR-0041 is equally flat that a
    // key-order violation is not one — the video renders identically either way. So the
    // gate story 5 wants is the counted `layout` line, not the process's exit status.
    assert_eq!(report.exit_code(), ExitCode::Ok);
    assert_eq!(report.summary().layout, 1);
    assert_eq!(report.summary().error, 0);
}

#[test]
fn write_mode_restores_one_element_per_line() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", PRETTY_PRINTED);

    let report = fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert_eq!(
        report.exit_code(),
        ExitCode::Ok,
        "the file is now canonical"
    );
    assert!(
        after.contains(
            r#"{"id":"photo-01","type":"image","start":0,"end":4000,"source":"images/01.png","width":1080,"height":1300,"fit":"cover","clip":[0,0,1080,1300]}"#
        ),
        "the whole element, on one line, tight (ADR-0007):\n{after}"
    );
    assert_eq!(after.lines().count(), 14, "{after}");
}

#[test]
fn write_mode_restores_canonical_key_order() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", KEYS_OUT_OF_ORDER);

    let report = fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert!(
        codes(&report).contains(&"L-KEY-ORDER"),
        "{:?}",
        report.findings
    );
    assert!(
        after.starts_with("{\n  \"frame\":"),
        "the header too:\n{after}"
    );
    assert!(
        after.contains(r#""layer": 10"#) && after.find(r#""name""#) < after.find(r#""layer""#),
        "a track's own order is the schema's:\n{after}"
    );
    assert!(
        after.contains(r#"{"id":"photo-01","type":"image","start":0,"end":4000,"source":"images/01.png","width":1080,"height":1300,"fit":"cover","#),
        "{after}"
    );
}

#[test]
fn a_key_order_finding_names_the_element_its_line_and_the_order_it_expected() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", KEYS_OUT_OF_ORDER);

    let report = fmt::fmt(&path, Mode::Check);
    let finding = report
        .findings
        .iter()
        .find(|f| f.code == "L-KEY-ORDER")
        .expect("a key-order finding");

    // ADR-0041's worked example: "names the element, its line, and the fix".
    assert_eq!(finding.location.element.as_deref(), Some("photo-01"));
    assert_eq!(finding.location.track.as_deref(), Some("photo"));
    assert_eq!(finding.location.line, Some(10));
    assert_eq!(
        finding.fields["expected"],
        "id,type,start,end,source,width,height,fit,clip"
    );
    // ADR-0030: the expected order names only what the element carries. A list that
    // included `x`, `y`, `origin` and `scale` would read as fields to add.
    assert!(
        !finding.fields["expected"]
            .as_str()
            .unwrap()
            .contains("origin")
    );
}

/// Canonical in every respect but one element's key order.
const ONLY_ELEMENT_KEYS: &str = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "tracks": [
    {
      "name": "photo",
      "layer": 10,
      "elements": [
        {"type":"image","id":"photo-01","start":0,"end":1000,"source":"a.png","width":10,"height":10,"fit":"literal"}
      ]
    }
  ]
}
"##;

/// Canonical in every respect but the header's own key order.
const ONLY_HEADER_KEYS: &str = r##"{
  "fps": 25,
  "frame": {"width": 1080, "height": 1920},
  "tracks": []
}
"##;

#[test]
fn an_element_whose_keys_are_the_only_fault_is_reported_once() {
    // ADR-0006's noise budget. `L-KEY-ORDER` already names the element and states the
    // order; an `L-LAYOUT` beside it saying the file is not canonical would be the same
    // fact in a second voice, and a reader who learns to skip one learns to skip both.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", ONLY_ELEMENT_KEYS);

    let report = fmt::fmt(&path, Mode::Check);

    assert_eq!(codes(&report), ["L-KEY-ORDER"], "{:?}", report.findings);
}

#[test]
fn the_headers_own_key_order_is_reported_and_restored() {
    // `L-KEY-ORDER`'s template names an element and its type (ADR-0041), so the header has
    // no finding of its own — which would leave `fmt` silently rewriting it if `L-LAYOUT`
    // did not cover everything outside an element.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", ONLY_HEADER_KEYS);

    let report = fmt::fmt(&path, Mode::Write);

    assert_eq!(codes(&report), ["L-LAYOUT"], "{:?}", report.findings);
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .starts_with("{\n  \"frame\":"),
    );
}

#[test]
fn formatting_a_formatted_file_changes_nothing_and_reports_nothing() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", PRETTY_PRINTED);

    fmt::fmt(&path, Mode::Write);
    let once = std::fs::read_to_string(&path).unwrap();
    let report = fmt::fmt(&path, Mode::Write);
    let twice = std::fs::read_to_string(&path).unwrap();

    // Idempotence is the property ADR-0007 refused a length threshold to protect: a
    // conditional serializer makes the second run's diff about the first run.
    assert_eq!(once, twice);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert_eq!(fmt::fmt(&path, Mode::Check).exit_code(), ExitCode::Ok);
}

#[test]
fn a_crlf_checkout_is_reported_at_its_first_line_not_its_last() {
    // The convention is a *byte* convention (`.gitattributes`, #189). `str::lines` strips
    // `\r\n` and `\n` alike, so a comparison built on it would call every line of a CRLF
    // file equal and send the reader to the end of a file that is wrong from line 1.
    let dir = common::tempdir(line!());
    let path = common::write_project(
        &dir,
        "p.montagent.json",
        &ONLY_HEADER_KEYS.replace('\n', "\r\n"),
    );

    let report = fmt::fmt(&path, Mode::Check);
    let finding = report
        .findings
        .iter()
        .find(|f| f.code == "L-LAYOUT")
        .expect("a layout finding");

    assert_eq!(finding.location.line, Some(1));
}

/// Two elements on one track, written later-first — which is legal JSON, renders
/// identically (ADR-0060), and is not the canonical convention.
const OUT_OF_SEQUENCE: &str = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "tracks": [
    {
      "name": "photo",
      "layer": 10,
      "elements": [
        {"id":"second","type":"image","start":1000,"end":2000,"source":"b.png","width":10,"height":10,"fit":"literal"},
        {"id":"first","type":"image","start":0,"end":1000,"source":"a.png","width":10,"height":10,"fit":"literal"}
      ]
    }
  ]
}
"##;

#[test]
fn elements_are_restored_to_their_sorted_order_within_a_track() {
    // ADR-0005's writing convention in full: "Elements are written sorted by `start` within
    // a track, and formatted one element per line." ADR-0041 restates it and says it "does
    // not reopen" it, so the sort is as much the canonical convention as the key order is.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", OUT_OF_SEQUENCE);

    let report = fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert_eq!(codes(&report), ["L-LAYOUT"], "{:?}", report.findings);
    assert!(
        after.find(r#""id":"first""#) < after.find(r#""id":"second""#),
        "{after}"
    );
    // Moving a line is safe in the one way that matters: the element's own text is
    // unchanged, so an exact-string replace written against it still matches.
    assert!(
        after.contains(
            r#"{"id":"second","type":"image","start":1000,"end":2000,"source":"b.png","width":10,"height":10,"fit":"literal"}"#
        ),
        "{after}"
    );
}

#[test]
fn elements_starting_at_one_instant_keep_the_order_the_file_wrote_them_in() {
    // The document says nothing about which of two coincident elements comes first, and a
    // tie broken on any other field would make the sort's output depend on a value the
    // author may edit next. Stable, by `start` alone.
    let dir = common::tempdir(line!());
    let tied = OUT_OF_SEQUENCE.replace(r#""start":1000,"end":2000"#, r#""start":0,"end":2000"#);
    let path = common::write_project(&dir, "p.montagent.json", &tied);

    fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert!(
        after.find(r#""id":"second""#) < after.find(r#""id":"first""#),
        "{after}"
    );
}

#[test]
fn an_element_with_no_start_yet_sorts_last_rather_than_first() {
    // The mid-edit element ADR-0042 insists stays formattable. Hoisting a half-typed
    // element above every complete one would make `fmt` hardest to read exactly on the file
    // it is being run to tidy.
    let dir = common::tempdir(line!());
    let half_typed = OUT_OF_SEQUENCE.replace(
        r#""id":"second","type":"image","start":1000,"#,
        r#""id":"second","type":"image","#,
    );
    let path = common::write_project(&dir, "p.montagent.json", &half_typed);

    fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert!(
        after.find(r#""id":"first""#) < after.find(r#""id":"second""#),
        "{after}"
    );
}

#[test]
fn the_committed_fixture_is_already_canonical() {
    // "Zero bytes change in the committed fixture. The order this ADR requires is the
    // order already on disk" (ADR-0041). A `fmt` that wants to touch a real published
    // video is a defect in `fmt`.
    let report = fmt::fmt(&fixture(), Mode::Check);

    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

// ---- The three things `fmt` must never do ------------------------------------------

/// One element with every defaultable field omitted, one with several written explicitly
/// at their defaults, and a retired spelling and an unknown key besides.
const PRESENCE: &str = r##"{
  "fps": 25,
  "frame": {"width": 1080, "height": 1920},
  "tracks": [
    {"name": "photo", "layer": 10, "elements": [
      {"type":"image","id":"bare","start":0,"end":1000,"source":"a.png","width":10,"height":10,"fit":"literal"},
      {"type":"image","id":"pinned","start":1000,"end":2000,"source":"b.png","x":0,"y":0,"origin":"top-left","width":10,"height":10,"fit":"literal","scale":[1.0,1.0],"rotation":0,"opacity":1.0,"gravity":"top","reviewedBy":"nobody"}
    ]}
  ]
}
"##;

#[test]
fn no_fmt_run_adds_or_removes_a_defaulted_field() {
    // ADR-0030: omission says "give me whatever the default is" and an explicit
    // `opacity: 1` says "I have pinned this to 1", and they diverge the moment the default
    // is revisited. Materialising the omitted ones would insert seven lines into `bare`
    // and invalidate any pending exact-string replace whose window touched the block.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", PRESENCE);

    fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    let bare = after.lines().find(|l| l.contains("\"bare\"")).unwrap();
    for key in [
        "x",
        "y",
        "origin",
        "scale",
        "rotation",
        "opacity",
        "line_height",
    ] {
        assert!(
            !bare.contains(&format!("\"{key}\":")),
            "`{key}` was materialised on `bare`: {bare}"
        );
    }

    let pinned = after.lines().find(|l| l.contains("\"pinned\"")).unwrap();
    for pair in [
        r#""x":0"#,
        r#""y":0"#,
        r#""origin":"top-left""#,
        r#""scale":[1.0,1.0]"#,
        r#""rotation":0"#,
        r#""opacity":1.0"#,
    ] {
        assert!(pinned.contains(pair), "`{pair}` was stripped: {pinned}");
    }
}

/// Two images whose drawn rect is exactly their `clip` box, so the rect both contains and
/// is contained by it — and `cover` and `contain` are both true.
const EXACT_ASPECT: &str = r##"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "tracks": [
    {
      "name": "photo",
      "layer": 10,
      "elements": [
        {"id":"says-cover","type":"image","start":0,"end":1000,"source":"a.png","width":1080,"height":1300,"fit":"cover","clip":[0,0,1080,1300]},
        {"id":"says-contain","type":"image","start":1000,"end":2000,"source":"b.png","width":1080,"height":1300,"fit":"contain","clip":[0,0,1080,1300]}
      ]
    }
  ]
}
"##;

#[test]
fn both_spellings_of_an_exact_aspect_fit_survive_unchanged() {
    // ADR-0026: at an exact match both are true, `validate` accepts either, and `fmt` has
    // no basis to prefer one. An agent's exact-string replace written against `contain`
    // gets zero hits the moment `fmt` has silently rewritten it to `cover`.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", EXACT_ASPECT);

    let report = fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert!(after.contains(r#""fit":"cover""#), "{after}");
    assert!(after.contains(r#""fit":"contain""#), "{after}");
    // Already canonical, so the whole file is a fixed point: `fmt` had every opportunity
    // to canonicalise one spelling into the other and no reason it could state.
    assert_eq!(after, EXACT_ASPECT);
}

#[test]
fn an_unknown_key_and_a_retired_spelling_both_survive_a_rewrite() {
    // ADR-0017 makes an unknown key an `error`; ADR-0042 makes formatting the file around
    // it `fmt`'s job anyway. Both hold at once only if the key survives — the schema
    // declares no position for it, so it follows the declared keys in the order it was
    // written.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", PRESENCE);

    fmt::fmt(&path, Mode::Write);
    let after = std::fs::read_to_string(&path).unwrap();

    assert!(
        after.contains(r#""gravity":"top","reviewedBy":"nobody""#),
        "{after}"
    );
    assert!(
        after.find(r#""opacity""#) < after.find(r#""gravity""#),
        "an undeclared key has no canonical position, so it follows the declared ones:\n{after}"
    );
}

// ---- The precondition: shape, never severity ---------------------------------------

#[test]
fn fmt_proceeds_on_a_file_carrying_error_findings() {
    // ADR-0042: gating on a clean `validate` would make `fmt` unusable exactly when it is
    // most wanted — mid-authoring, with open findings — and would let "does it say what
    // you meant" hold a byte-level rewrite hostage.
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", PRESENCE);

    // The same file, through `validate`, carries the retired spelling as an `error`.
    let errors = montagent_core::validate(&path)
        .findings
        .iter()
        .filter(|f| f.code.starts_with("E-RETIRED"))
        .count();
    assert!(errors > 0, "the fixture for this test must carry an error");

    let report = fmt::fmt(&path, Mode::Write);

    assert!(
        !codes(&report).iter().any(|c| c.starts_with("E-")),
        "`fmt` reports layout, never severity: {:?}",
        report.findings
    );
    assert!(std::fs::read_to_string(&path).unwrap().contains("\"bare\""));
}

#[test]
fn fmt_refuses_a_document_that_is_not_a_project_and_writes_nothing() {
    // The reported hazard is wrong-file destruction: `fmt` pointed at a transcript export
    // will rewrite it (ADR-0042).
    let dir = common::tempdir(line!());
    let transcript = "{\"segments\": [{\"start\": 0.0, \"text\": \"hello\"}]}\n";
    let path = common::write_project(&dir, "transcript.json", transcript);

    let report = fmt::fmt(&path, Mode::Check);

    assert_eq!(codes(&report), ["E-NOT-A-PROJECT"]);
    assert_eq!(
        report.findings[0].fields["missing"],
        "`tracks`/`fps`/`frame`"
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), transcript);
}

#[test]
fn fmt_refuses_in_write_mode_too_rather_than_only_under_check() {
    let dir = common::tempdir(line!());
    let transcript = "{\"segments\": []}\n";
    let path = common::write_project(&dir, "transcript.json", transcript);

    fmt::fmt(&path, Mode::Write);

    assert_eq!(std::fs::read_to_string(&path).unwrap(), transcript);
}

#[test]
fn a_malformed_file_is_exit_2_and_is_not_partially_processed() {
    let dir = common::tempdir(line!());
    let broken = "{\n  \"fps\": ,\n  \"tracks\": []\n}\n";
    let path = common::write_project(&dir, "broken.montagent.json", broken);

    let report = fmt::fmt(&path, Mode::Write);

    assert_eq!(codes(&report), ["E-PARSE"]);
    assert_eq!(report.exit_code(), ExitCode::Unparseable);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), broken);
}

// ---- The predicate is one function -------------------------------------------------

#[test]
fn the_predicate_and_the_rewrite_are_one_implementation() {
    // ADR-0041: "there is exactly one place the rule lives, not two that can disagree."
    // `is_canonical` is defined as "`reorder` would change nothing", so this is not a
    // coincidence between two traversals — it is the same code answering twice. A future
    // hand-written predicate would fail here on the first element it disagreed about.
    let document: serde_json::Value = serde_json::from_str(PRESENCE).unwrap();
    let elements = document["tracks"][0]["elements"].as_array().unwrap();

    for element in elements {
        let object = element.as_object().unwrap();
        let published = Published::of_element(element);
        assert_eq!(
            layout::is_canonical(published, object),
            layout::reorder(published, object).keys().eq(object.keys()),
        );
        // And the rewrite is a fixed point, which is what makes a second `fmt` run a no-op.
        let once = layout::reorder(published, object);
        assert!(layout::is_canonical(published, &once));
    }
}

#[test]
fn every_published_type_has_exactly_one_stated_order() {
    // ADR-0041 ties the order to the schema so there is "exactly one place canonical key
    // order can go stale — the schema itself — rather than two artifacts that can drift
    // apart". `layout::canonical_order` is the only door: `model` used to re-export a
    // second spelling of it, and a second door is where the drift starts.
    for type_name in [
        "image",
        "video",
        "text",
        "rect",
        "ellipse",
        "audio",
        "transition",
    ] {
        let order = layout::canonical_order(Published::Element(type_name))
            .unwrap_or_else(|| panic!("{type_name} is a published type"));
        assert_eq!(
            &order[..5],
            ["id", "type", "group", "start", "end"],
            "{type_name}: ADR-0041's universal prefix comes first"
        );
    }
}

#[test]
fn a_shape_the_schema_does_not_publish_is_left_exactly_as_written() {
    // A mid-edit element with no `type` yet, which is the file ADR-0042 insists stays
    // formattable. Inventing an order for it is how a formatter damages a file it was
    // pointed at to repair.
    let element = serde_json::json!({"end": 10, "id": "x", "start": 0});
    let object = element.as_object().unwrap();

    assert_eq!(
        layout::canonical_order(Published::of_element(&element)),
        None
    );
    assert!(layout::is_canonical(
        Published::of_element(&element),
        object
    ));
    assert!(
        layout::reorder(Published::of_element(&element), object)
            .keys()
            .eq(object.keys())
    );
}

// ---- The atomic write --------------------------------------------------------------

#[test]
fn a_write_that_fails_leaves_the_original_file_intact() {
    // ADR-0011: "read, parse, check, write atomically, or do nothing." The hazard is not
    // theoretical — a half-written project is unparseable, so every later run answers
    // `E-PARSE` and the work is gone.
    let dir = common::tempdir(line!());
    let original = "the bytes that must survive\n";
    let path = common::write_project(&dir, "p.montagent.json", original);

    if !make_read_only(&dir) {
        eprintln!("skipping: this platform or user can write to a read-only directory");
        return;
    }
    let result = write::atomically(&path, "replacement bytes\n");
    restore_writable(&dir);

    assert!(
        result.is_err(),
        "the write must fail for this test to mean anything"
    );
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        original,
        "the destination is never opened for writing"
    );
}

#[test]
fn a_successful_write_leaves_no_temp_file_behind() {
    let dir = common::tempdir(line!());
    let path = common::write_project(&dir, "p.montagent.json", "before\n");

    write::atomically(&path, "after\n").expect("a writable directory");

    assert_eq!(std::fs::read_to_string(&path).unwrap(), "after\n");
    let left: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name != "p.montagent.json")
        .collect();
    assert!(left.is_empty(), "stray files: {left:?}");
}

/// Make `dir` refuse new files, and say whether it took.
///
/// A read-only directory is the one portable way to fail a write *before* the rename
/// without reaching into the filesystem. It does not take under every user — root ignores
/// the mode bits, and Windows does not express "no new entries" this way — so the caller
/// checks rather than assumes, on the same reasoning `common::has_ffprobe` uses: a
/// capability this machine does not have is a legitimate state, not a failure.
fn make_read_only(dir: &Path) -> bool {
    let mut permissions = std::fs::metadata(dir).unwrap().permissions();
    permissions.set_readonly(true);
    if std::fs::set_permissions(dir, permissions).is_err() {
        return false;
    }
    // Ask the filesystem rather than the mode bits.
    let probe = dir.join(".montagent-write-probe");
    match std::fs::write(&probe, "x") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            restore_writable(dir);
            false
        }
        Err(_) => true,
    }
}

fn restore_writable(dir: &Path) {
    let mut permissions = std::fs::metadata(dir).unwrap().permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);
    let _ = std::fs::set_permissions(dir, permissions);
}
