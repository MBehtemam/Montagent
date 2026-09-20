//! `fonts list`, `fonts vendor`, and `validate`'s attestation checks (#207, ADR-0057).
//!
//! The one real font these tests have is the fixture's Open Runde, which carries no licence
//! strings at all — bucket 3, the ADR's "common case" — so the declared path through the
//! gate is exercised against real bytes. The blocklist's refusal is exercised against a
//! real Apple font where the machine has one, and against the gate's own unit tests
//! everywhere.

use std::path::{Path, PathBuf};

use montaget_core::finding::{Class, Repair};
use montaget_core::report::ExitCode;
use montaget_core::verbs::fonts::{Vendor, list, vendor};

mod common;

const HEADER_ONLY: &str = r#"{
  "frame": {"width": 1080, "height": 1920},
  "fps": 25,
  "tracks": []
}
"#;

const OPEN_RUNDE_SHA256: &str = "995f115d11590c73ed95cd58bf12db0417f8757986d95c9be773faf1583310fd";

fn open_runde() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
        .canonicalize()
        .expect("the vendored fixture font")
}

fn ask(font: PathBuf, licence: Option<&str>) -> Vendor {
    Vendor {
        font,
        licence: licence.map(String::from),
        source: None,
        destination: None,
    }
}

fn codes(report: &montaget_core::report::Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

// ---- `fonts list` ---------------------------------------------------------------------

#[test]
fn list_enumerates_every_face_under_a_root_with_its_index_and_licence_status() {
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("nested")).unwrap();
    std::fs::copy(open_runde(), dir.join("nested/OpenRunde-Bold.otf")).unwrap();
    // A file with a font extension that is not a font is reported, not skipped.
    std::fs::write(dir.join("not-a-font.ttf"), b"hello").unwrap();
    // A file with no font extension is not a candidate at all.
    std::fs::write(dir.join("README.md"), b"# fonts").unwrap();

    let listing = list(&[dir.clone(), dir.join("absent")]);
    let inventory = listing.inventory();

    assert_eq!(
        inventory.roots,
        vec![dir.display().to_string()],
        "a root that does not exist is skipped, not reported"
    );
    assert_eq!(inventory.fonts.len(), 1, "{:?}", inventory.fonts);
    let face = &inventory.fonts[0];
    assert_eq!(face.index, 0);
    assert_eq!(face.family.as_deref(), Some("Open Runde"));
    assert_eq!(face.postscript.as_deref(), Some("OpenRunde-Bold"));
    assert_eq!(
        face.status, "unknown",
        "Open Runde carries no licence strings: bucket 3"
    );
    assert_eq!(face.matched, None);
    assert_eq!(inventory.unreadable.len(), 1);
    assert!(inventory.unreadable[0].path.ends_with("not-a-font.ttf"));
    assert_eq!(listing.report().exit_code(), ExitCode::Ok);
}

#[test]
fn list_is_one_block_on_the_canonical_json_and_the_prose_is_generated_from_it() {
    let dir = common::tempdir(line!());
    std::fs::copy(open_runde(), dir.join("OpenRunde-Bold.otf")).unwrap();

    let listing = list(std::slice::from_ref(&dir));
    let json = listing.to_json();
    assert_eq!(json["tool"], "fonts list");
    assert_eq!(json["fonts"]["fonts"][0]["status"], "unknown");
    assert_eq!(json["fonts"]["fonts"][0]["index"], 0);

    let text = montaget_core::wire::render_fonts_list(
        &listing,
        montaget_core::Wire::Text { verbose: false },
    );
    assert!(text.contains("FONTS  1 face under"), "{text}");
    assert!(text.contains("unknown"), "{text}");
    assert!(
        text.contains("OpenRunde-Bold.otf#0  Open Runde [OpenRunde-Bold]"),
        "{text}"
    );
}

#[test]
fn list_reports_a_real_apple_font_as_blocklisted_where_the_machine_has_one() {
    // The seed list's one confirmed case, against the bytes Apple actually ships. Skipped
    // rather than faked elsewhere: there is no redistributable Apple font to commit.
    let rounded = Path::new("/System/Library/Fonts/SFNSRounded.ttf");
    if !rounded.is_file() {
        eprintln!("skipping: no Apple system font on this machine");
        return;
    }
    let dir = common::tempdir(line!());
    std::fs::copy(rounded, dir.join("SFNSRounded.ttf")).unwrap();

    let listing = list(&[dir]);
    let face = &listing.inventory().fonts[0];
    assert_eq!(face.status, "blocklisted", "{face:?}");
    assert_eq!(face.matched.as_deref(), Some(".SF NS"));
}

// ---- `fonts vendor`: the gate ---------------------------------------------------------

#[test]
fn an_unrecognised_licence_is_refused_until_declared_and_no_bytes_are_copied() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);

    let answer = vendor(&project, &ask(open_runde(), None));

    let report = answer.report();
    assert_eq!(report.exit_code(), ExitCode::Errors, "{:?}", codes(report));
    assert_eq!(codes(report), vec!["E-FONT-LICENCE-UNKNOWN"]);
    let finding = &report.findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.fields["name"], "Open Runde");
    assert!(
        finding.fields["detail"]
            .as_str()
            .unwrap()
            .contains("no licence description (ID 13)"),
        "{finding:?}"
    );
    assert!(
        matches!(finding.repair, Some(Repair::Advise(_))),
        "the repair is the declaration a human makes: {finding:?}"
    );
    assert!(answer.view().is_none());
    // The refusal is before the copy, structurally: nothing landed and the file is as
    // it was.
    assert!(!dir.join("fonts").exists(), "no bytes were copied");
    assert_eq!(std::fs::read_to_string(&project).unwrap(), HEADER_ONLY);
}

#[test]
fn a_blocklisted_font_is_refused_even_with_a_declared_licence_and_nothing_is_copied() {
    let rounded = Path::new("/System/Library/Fonts/SFNSRounded.ttf");
    if !rounded.is_file() {
        eprintln!("skipping: no Apple system font on this machine");
        return;
    }
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);

    // `--licence` is the bucket-3 declaration; bucket 1 ignores it, because "an override
    // affordance is itself the thing that makes the project a knowing party" (ADR-0057).
    let answer = vendor(&project, &ask(rounded.to_path_buf(), Some("OFL-1.1")));

    let report = answer.report();
    assert_eq!(codes(report), vec!["E-FONT-BLOCKLISTED"]);
    let finding = &report.findings[0];
    assert_eq!(
        finding.repair,
        Some(Repair::None),
        "refuse-class: no flag lifts it"
    );
    assert_eq!(finding.fields["matched"], ".SF NS");
    assert!(
        finding.fields["substitutes"]
            .as_str()
            .unwrap()
            .contains("Open Runde"),
        "a refusal may name a substitute, labelled by its basis: {finding:?}"
    );
    assert!(
        finding.fields["substitutes"]
            .as_str()
            .unwrap()
            .contains("no metric compatibility"),
    );
    assert!(!dir.join("fonts").exists());
    assert_eq!(std::fs::read_to_string(&project).unwrap(), HEADER_ONLY);
}

#[test]
fn a_file_that_is_not_a_font_is_a_bad_invocation_not_a_finding_about_the_project() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);
    let not_a_font = dir.join("nope.ttf");
    std::fs::write(&not_a_font, b"hello").unwrap();

    let answer = vendor(&project, &ask(not_a_font, Some("OFL-1.1")));
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert_eq!(codes(answer.report()), vec!["E-INVOCATION"]);

    let answer = vendor(&project, &ask(dir.join("absent.ttf"), Some("OFL-1.1")));
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
}

// ---- `fonts vendor`: the copy and the attestation -------------------------------------

#[test]
fn a_declared_licence_copies_the_bytes_and_writes_the_attestation_keyed_by_path() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);

    let answer = vendor(
        &project,
        &Vendor {
            font: open_runde(),
            licence: Some("OFL-1.1".into()),
            source: Some("https://github.com/lauridskern/open-runde".into()),
            destination: None,
        },
    );

    let report = answer.report();
    assert_eq!(report.tool, "fonts vendor");
    // The write-tool invariant: the new state's findings. The one thing `validate` has to
    // say about a font no chain references yet is exactly the orphan note.
    assert_eq!(codes(report), vec!["N-FONT-ATTESTATION-ORPHANED"]);
    assert_eq!(report.exit_code(), ExitCode::Ok);

    let view = answer.view().expect("something was written");
    assert_eq!(view.file, "fonts/OpenRunde-Bold.otf");
    assert_eq!(view.licence_basis, "declared");
    assert!(!view.already_present);
    assert_eq!(
        view.chain_entry,
        serde_json::json!({"file": "fonts/OpenRunde-Bold.otf"})
    );
    assert_eq!(view.attestation.sha256, OPEN_RUNDE_SHA256);

    // The bytes landed, byte for byte.
    assert_eq!(
        std::fs::read(dir.join("fonts/OpenRunde-Bold.otf")).unwrap(),
        std::fs::read(open_runde()).unwrap()
    );

    // The table is keyed by file path and matches the committed fixture's shape and key
    // order, and it lands where the schema puts it: after `fps` (no `fonts` here), before
    // `tracks`, in the canonical convention.
    assert_eq!(
        std::fs::read_to_string(&project).unwrap(),
        format!(
            "{{\n  \"frame\": {{\"width\": 1080, \"height\": 1920}},\n  \"fps\": 25,\n  \
             \"fontVendor\": {{\"fonts/OpenRunde-Bold.otf\": {{\"licence\": \"OFL-1.1\", \
             \"source\": \"https://github.com/lauridskern/open-runde\", \"sha256\": \
             \"{OPEN_RUNDE_SHA256}\"}}}},\n  \"tracks\": []\n}}\n"
        )
    );
}

#[test]
fn the_attestation_matches_the_committed_fixtures_entry_exactly() {
    // The fixture's own `fontVendor` entry is what `fonts vendor` would have written, so
    // the verb and the fixture cannot drift: same key, same three fields, same hash.
    let fixture: serde_json::Value =
        serde_json::from_str(
            &std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json",
            ))
            .unwrap(),
        )
        .unwrap();
    let committed = &fixture["fontVendor"]["fonts/OpenRunde-Bold.otf"];

    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);
    let answer = vendor(
        &project,
        &Vendor {
            font: open_runde(),
            licence: Some(committed["licence"].as_str().unwrap().into()),
            source: Some(committed["source"].as_str().unwrap().into()),
            destination: None,
        },
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    assert_eq!(
        &written["fontVendor"]["fonts/OpenRunde-Bold.otf"],
        committed
    );
    assert_eq!(
        serde_json::to_value(&answer.view().unwrap().attestation).unwrap(),
        *committed
    );
}

#[test]
fn the_source_defaults_to_the_absolute_path_the_bytes_came_from() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);
    let answer = vendor(&project, &ask(open_runde(), Some("OFL-1.1")));
    let source = &answer.view().unwrap().attestation.source;
    assert!(Path::new(source).is_absolute(), "{source}");
    assert!(source.ends_with("OpenRunde-Bold.otf"), "{source}");
}

#[test]
fn vendoring_the_same_bytes_twice_is_idempotent() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);
    vendor(&project, &ask(open_runde(), Some("OFL-1.1")));
    let first = std::fs::read_to_string(&project).unwrap();

    let again = vendor(&project, &ask(open_runde(), Some("OFL-1.1")));
    assert!(again.view().unwrap().already_present);
    assert_eq!(again.report().exit_code(), ExitCode::Ok);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), first);
}

#[test]
fn a_different_font_under_the_same_path_is_refused_rather_than_swapped_in_place() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::write(dir.join("fonts/OpenRunde-Bold.otf"), b"some other font").unwrap();

    let answer = vendor(&project, &ask(open_runde(), Some("OFL-1.1")));

    // ADR-0007: a font swapped in place silently invalidates every measured size and
    // break. The command is what changes — another name, or remove the old file.
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert_eq!(
        std::fs::read(dir.join("fonts/OpenRunde-Bold.otf")).unwrap(),
        b"some other font"
    );
    assert_eq!(std::fs::read_to_string(&project).unwrap(), HEADER_ONLY);
}

#[test]
fn as_names_the_path_inside_the_project_and_may_not_escape_it() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);

    let answer = vendor(
        &project,
        &Vendor {
            destination: Some("brand/rounded-bold.otf".into()),
            ..ask(open_runde(), Some("OFL-1.1"))
        },
    );
    assert_eq!(answer.view().unwrap().file, "brand/rounded-bold.otf");
    assert!(dir.join("brand/rounded-bold.otf").is_file());

    for escape in ["../elsewhere.otf", "/tmp/elsewhere.otf", "a/../../b.otf"] {
        let answer = vendor(
            &project,
            &Vendor {
                destination: Some(escape.into()),
                ..ask(open_runde(), Some("OFL-1.1"))
            },
        );
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "{escape}"
        );
    }
}

#[test]
fn vendor_never_edits_the_fonts_table_and_a_referenced_font_then_validates_clean() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montaget.json", HEADER_ONLY);
    vendor(&project, &ask(open_runde(), Some("OFL-1.1")));

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    assert!(
        written.get("fonts").is_none(),
        "which chain the font joins is the author's declared change"
    );

    // The author writes the chain entry the answer echoed; the orphan note goes away and
    // the attestation check passes.
    let mut with_chain = written.clone();
    with_chain.as_object_mut().unwrap().insert(
        "fonts".into(),
        serde_json::json!({"brand": [{"file": "fonts/OpenRunde-Bold.otf"}]}),
    );
    std::fs::write(&project, common::canonical(&with_chain.to_string())).unwrap();

    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), Vec::<&str>::new(), "{:?}", report.findings);
}

#[test]
fn vendor_refuses_a_file_that_is_not_a_project_and_a_malformed_one() {
    let dir = common::tempdir(line!());
    let not_a_project = common::write_project(&dir, "t.json", "{\"segments\": []}\n");
    let answer = vendor(&not_a_project, &ask(open_runde(), Some("OFL-1.1")));
    assert_eq!(codes(answer.report()), vec!["E-NOT-A-PROJECT"]);
    assert!(!dir.join("fonts").exists());

    let broken = common::write_project(&dir, "b.json", "{\"fps\": ,}\n");
    let answer = vendor(&broken, &ask(open_runde(), Some("OFL-1.1")));
    assert_eq!(answer.report().exit_code(), ExitCode::Unparseable);
    assert!(!dir.join("fonts").exists());
}

// ---- `validate`: the attestation checks ---------------------------------------------

fn project_with(dir: &Path, fonts: &str, font_vendor: &str) -> PathBuf {
    let body = format!(
        r#"{{"frame": {{"width": 1080, "height": 1920}}, "fps": 25, {fonts} {font_vendor} "tracks": []}}"#
    );
    common::write_project(dir, "p.montaget.json", &common::canonical(&body))
}

fn attestation(sha256: &str) -> String {
    format!(
        r#""fontVendor": {{"fonts/OpenRunde-Bold.otf": {{"licence": "OFL-1.1", "source": "x", "sha256": "{sha256}"}}}},"#
    )
}

const CHAIN: &str = r#""fonts": {"brand": [{"file": "fonts/OpenRunde-Bold.otf"}]},"#;

#[test]
fn a_referenced_font_with_no_attestation_is_an_advise_class_error() {
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::copy(open_runde(), dir.join("fonts/OpenRunde-Bold.otf")).unwrap();
    let project = project_with(&dir, CHAIN, "");

    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-UNATTESTED"]);
    let finding = &report.findings[0];
    assert_eq!(finding.fields["file"], "fonts/OpenRunde-Bold.otf");
    assert!(
        matches!(&finding.repair, Some(Repair::Advise(v)) if v["value"].as_str().unwrap().contains("montaget fonts vendor")),
        "{finding:?}"
    );
    assert_eq!(report.exit_code(), ExitCode::Errors);
}

#[test]
fn a_hash_that_does_not_match_the_file_on_disk_is_a_refuse_class_error() {
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::copy(open_runde(), dir.join("fonts/OpenRunde-Bold.otf")).unwrap();
    let stale = "0000000000000000000000000000000000000000000000000000000000000000";
    let project = project_with(&dir, CHAIN, &attestation(stale));

    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-HASH-MISMATCH"]);
    let finding = &report.findings[0];
    assert_eq!(finding.repair, Some(Repair::None));
    assert_eq!(finding.fields["recorded"], stale);
    assert_eq!(finding.fields["actual"], OPEN_RUNDE_SHA256);
}

#[test]
fn a_matching_hash_is_silent() {
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::copy(open_runde(), dir.join("fonts/OpenRunde-Bold.otf")).unwrap();
    let project = project_with(&dir, CHAIN, &attestation(OPEN_RUNDE_SHA256));

    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), Vec::<&str>::new(), "{:?}", report.findings);
}

#[test]
fn a_referenced_font_that_is_not_on_disk_is_named_rather_than_reported_as_a_mismatch() {
    let dir = common::tempdir(line!());
    let project = project_with(&dir, CHAIN, &attestation(OPEN_RUNDE_SHA256));

    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-MISSING"]);
    assert!(
        report.findings[0].fields["resolved"]
            .as_str()
            .unwrap()
            .contains("OpenRunde-Bold.otf")
    );

    // Missing *and* unattested: both facts, in one run.
    let project = project_with(&dir, CHAIN, "");
    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-MISSING", "E-FONT-UNATTESTED"]);
}

#[test]
fn an_orphaned_attestation_is_a_note_and_fmt_never_prunes_it() {
    let dir = common::tempdir(line!());
    let project = project_with(&dir, "", &attestation(OPEN_RUNDE_SHA256));

    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), vec!["N-FONT-ATTESTATION-ORPHANED"]);
    assert_eq!(report.findings[0].class, Class::Note);
    assert_eq!(report.exit_code(), ExitCode::Ok);

    let before = std::fs::read_to_string(&project).unwrap();
    montaget_core::verbs::fmt::fmt(&project, montaget_core::verbs::fmt::Mode::Write);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
    assert!(before.contains("\"fontVendor\""));
}

#[test]
fn one_file_under_two_keys_is_checked_once_and_attested_once() {
    // ADR-0057's whole reason for a path-keyed table: `Inter-SemiBold.ttf` under `brand`
    // and again under `brand+fa` is one file, one hash, one licence.
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::copy(open_runde(), dir.join("fonts/OpenRunde-Bold.otf")).unwrap();
    let chains = r#""fonts": {"brand": [{"file": "fonts/OpenRunde-Bold.otf"}], "brand+fa": [{"file": "fonts/OpenRunde-Bold.otf"}]},"#;

    let project = project_with(&dir, chains, "");
    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-UNATTESTED"], "once, not twice");

    let project = project_with(&dir, chains, &attestation(OPEN_RUNDE_SHA256));
    let report = montaget_core::validate(&project);
    assert_eq!(codes(&report), Vec::<&str>::new());
}
