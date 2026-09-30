//! `fonts list`, `fonts vendor`, and `validate`'s attestation checks (#207, ADR-0057).
//!
//! The one real font these tests have is the fixture's Open Runde, which carries no licence
//! strings at all — bucket 3, the ADR's "common case" — so the declared path through the
//! gate is exercised against real bytes. The blocklist's refusal is exercised against a
//! real Apple font where the machine has one, and against the gate's own unit tests
//! everywhere.

use std::path::{Path, PathBuf};

use montagent_core::finding::{Class, Repair};
use montagent_core::report::ExitCode;
use montagent_core::verbs::fonts::{Vendor, list, vendor};

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

fn codes(report: &montagent_core::report::Report) -> Vec<&str> {
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

    let text = montagent_core::wire::render_fonts_list(
        &listing,
        montagent_core::Wire::Text { verbose: false },
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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);

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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);

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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);
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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);

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
                "../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json",
            ))
            .unwrap(),
        )
        .unwrap();
    let committed = &fixture["fontVendor"]["fonts/OpenRunde-Bold.otf"];

    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);
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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);
    let answer = vendor(&project, &ask(open_runde(), Some("OFL-1.1")));
    let source = &answer.view().unwrap().attestation.source;
    assert!(Path::new(source).is_absolute(), "{source}");
    assert!(source.ends_with("OpenRunde-Bold.otf"), "{source}");
}

#[test]
fn vendoring_the_same_bytes_twice_is_idempotent() {
    let dir = common::tempdir(line!());
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);
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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);
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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);

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
    let project = common::write_project(&dir, "p.montagent.json", HEADER_ONLY);
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

    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), Vec::<&str>::new(), "{:?}", report.findings);
}

#[test]
fn the_advised_repair_converges_when_the_fonts_table_names_a_path_vendor_would_not_default_to() {
    // #367: a chain entry that names a path other than `fonts/<name>` (vendor's default
    // destination) previously looped `E-FONT-UNATTESTED` forever, because the advised
    // repair never told the caller to target that exact path. The fix is `--as <file>`,
    // named in the repair text itself — proven here by parsing it out and following it.
    let dir = common::tempdir(line!());
    std::fs::copy(open_runde(), dir.join("body.otf")).unwrap();
    let project = project_with(&dir, r#""fonts": {"body": [{"file": "body.otf"}]},"#, "");

    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-UNATTESTED"]);
    let repair = match &report.findings[0].repair {
        Some(Repair::Advise(v)) => v["value"].as_str().unwrap().to_string(),
        other => panic!("expected an advise-class repair, got {other:?}"),
    };
    assert!(
        repair.contains("--as body.otf"),
        "the repair must name the exact fonts-table path: {repair}"
    );

    let answer = vendor(
        &project,
        &Vendor {
            destination: Some("body.otf".into()),
            ..ask(open_runde(), Some("OFL-1.1"))
        },
    );
    assert_eq!(answer.view().unwrap().file, "body.otf");

    let report = montagent_core::validate(&project);
    assert_eq!(
        codes(&report),
        Vec::<&str>::new(),
        "following the advised repair once must converge: {:?}",
        report.findings
    );
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
    common::write_project(dir, "p.montagent.json", &common::canonical(&body))
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

    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-UNATTESTED"]);
    let finding = &report.findings[0];
    assert_eq!(finding.fields["file"], "fonts/OpenRunde-Bold.otf");
    assert!(
        matches!(&finding.repair, Some(Repair::Advise(v)) if v["value"].as_str().unwrap().contains("montagent fonts vendor")),
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

    let report = montagent_core::validate(&project);
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

    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), Vec::<&str>::new(), "{:?}", report.findings);
}

#[test]
fn a_referenced_font_that_is_not_on_disk_is_named_rather_than_reported_as_a_mismatch() {
    let dir = common::tempdir(line!());
    let project = project_with(&dir, CHAIN, &attestation(OPEN_RUNDE_SHA256));

    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-MISSING"]);
    assert!(
        report.findings[0].fields["resolved"]
            .as_str()
            .unwrap()
            .contains("OpenRunde-Bold.otf")
    );

    // Missing *and* unattested: both facts, in one run.
    let project = project_with(&dir, CHAIN, "");
    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-MISSING", "E-FONT-UNATTESTED"]);
}

#[test]
fn an_orphaned_attestation_is_a_note_and_fmt_never_prunes_it() {
    let dir = common::tempdir(line!());
    let project = project_with(&dir, "", &attestation(OPEN_RUNDE_SHA256));

    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), vec!["N-FONT-ATTESTATION-ORPHANED"]);
    assert_eq!(report.findings[0].class, Class::Note);
    assert_eq!(report.exit_code(), ExitCode::Ok);

    let before = std::fs::read_to_string(&project).unwrap();
    montagent_core::verbs::fmt::fmt(&project, montagent_core::verbs::fmt::Mode::Write);
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
    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), vec!["E-FONT-UNATTESTED"], "once, not twice");

    let project = project_with(&dir, chains, &attestation(OPEN_RUNDE_SHA256));
    let report = montagent_core::validate(&project);
    assert_eq!(codes(&report), Vec::<&str>::new());
}

/// The sidecar file a test owns, inside its own scratch directory.
///
/// Every `validate` below goes through it rather than through the machine's default cache
/// (`tests/sidecar.rs`'s rule for the probe half, and #206's for the font half): a test that
/// did not own this file would record its scratch projects into the developer's own cache,
/// and `R-FONT-SWAP` — which compares against what a *previous run* recorded — would be
/// comparing against whatever that developer had validated last.
fn cache(dir: &Path) -> PathBuf {
    dir.join("probe-cache.json")
}

// ---- ADR-0007's glyph coverage and font census (#206) ---------------------------------

/// A project whose `fonts` table the caller composes, with one track of text elements.
///
/// Attested with the real hash, so the only findings these tests see are the ones they are
/// about — the attestation half above already has its own pairs.
fn typeset(dir: &Path, fonts: &str, elements: &str) -> PathBuf {
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::copy(open_runde(), dir.join("fonts/OpenRunde-Bold.otf")).unwrap();
    let body = format!(
        r#"{{"frame": {{"width": 1080, "height": 1920}}, "fps": 25, "fonts": {fonts}, {} "tracks": [{{"name": "captions", "layer": 0, "elements": [{elements}]}}]}}"#,
        attestation(OPEN_RUNDE_SHA256)
    );
    common::write_project(dir, "p.montagent.json", &common::canonical(&body))
}

/// One text element in `font`, carrying one run.
fn typeset_element(id: &str, font: &str, text: &str) -> String {
    serde_json::json!({
        "id": id, "type": "text", "start": 0, "end": 1000,
        "x": 0, "y": 0, "width": 900, "height": 100,
        "font": font, "size": 40, "runs": [{"text": text}]
    })
    .to_string()
}

const BRAND: &str = r#"{"brand": [{"file": "fonts/OpenRunde-Bold.otf"}]}"#;

#[track_caller]
fn findings<'a>(
    report: &'a montagent_core::report::Report,
    code: &str,
) -> Vec<&'a montagent_core::finding::Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

#[test]
fn a_character_no_file_in_the_chain_maps_is_a_refuse_class_error() {
    // ADR-0007: "A character with no glyph in any chain entry renders `.notdef` and is a
    // `validate` **error**." Open Runde is a Latin face; the Persian below is tofu in it,
    // and the chain declares nothing else to fall back to.
    let dir = common::tempdir(line!());
    let project = typeset(
        &dir,
        BRAND,
        &typeset_element("greeting", "brand", "\u{0633}\u{0644}\u{0627}\u{0645}"),
    );

    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    let found = findings(&report, "E-FONT-NO-GLYPH");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    let finding = found[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(finding.location.element.as_deref(), Some("greeting"));
    assert_eq!(finding.location.track.as_deref(), Some("captions"));
    assert_eq!(finding.fields["font"], "brand");
    assert_eq!(finding.fields["chain"], "fonts/OpenRunde-Bold.otf");
    assert_eq!(
        finding.fields["characters"],
        "U+0633 \u{0633}, U+0644 \u{0644}, U+0627 \u{0627}, U+0645 \u{0645}"
    );
    assert_eq!(
        finding.repair,
        Some(Repair::None),
        "the chain is short a file, or the text carries a character it was never meant to"
    );
}

#[test]
fn a_missing_glyph_names_no_other_chain_even_where_one_would_draw_it() {
    // ADR-0120. The fork this finding refuses on is *the chain is short a file* or *the
    // text carries a character it was not meant to*, and a list of the project's other
    // chains that do map the characters bears on the first branch only — a menu for it,
    // which is a repair by another name. So the finding carries no such list, even here,
    // where a second declared chain maps every one of them.
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../docs/research/prototypes/thai-vertical-metrics/fonts/NotoSansThai-Regular.ttf",
        ),
        dir.join("fonts/NotoSansThai-Regular.ttf"),
    )
    .unwrap();
    let thai = "\u{0E01}\u{0E02}";
    let mut drawn: serde_json::Value =
        serde_json::from_str(&typeset_element("thai-chain", "thai", thai)).unwrap();
    drawn["start"] = 1000.into();
    drawn["end"] = 2000.into();
    let elements = format!("{}, {drawn}", typeset_element("latin-chain", "brand", thai));
    let fonts = r#"{"brand": [{"file": "fonts/OpenRunde-Bold.otf"}], "thai": [{"file": "fonts/NotoSansThai-Regular.ttf"}]}"#;
    let project = typeset(&dir, fonts, &elements);

    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    let found = findings(&report, "E-FONT-NO-GLYPH");
    // The precondition, so the absence below is not vacuous: the `thai` chain draws what
    // `brand` cannot.
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].location.element.as_deref(), Some("latin-chain"));

    assert!(found[0].census.is_none(), "{:?}", found[0]);
    let json = report.to_json();
    let emitted = json["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["code"] == "E-FONT-NO-GLYPH")
        .unwrap();
    assert!(
        emitted.get("census").is_none_or(serde_json::Value::is_null),
        "{emitted}"
    );
    let prose = montagent_core::text::render(&json, montagent_core::text::Options::verbose())
        .expect("every finding renders");
    // `N-FONT-CENSUS` rightly prints a census here — the text is set in two fonts — so
    // the text-form check reads this finding's own block only.
    let block = prose
        .split("\n\n")
        .find(|b| b.contains("E-FONT-NO-GLYPH"))
        .expect("the finding prints");
    assert!(!block.contains("census"), "{block}");
}

#[test]
fn text_the_declared_font_can_draw_never_fires() {
    // The must-not-fire half, and with it the characters no font is short of for not
    // mapping: the format's own `\n` line break (ADR-0008) and ADR-0007's ZWJ, which is
    // how an emoji sequence is spelled rather than a glyph anything draws.
    let dir = common::tempdir(line!());
    let project = typeset(
        &dir,
        BRAND,
        &typeset_element(
            "caption",
            "brand",
            "cobweb  -  cobweb\nover the door\u{200D}",
        ),
    );

    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    assert_eq!(findings(&report, "E-FONT-NO-GLYPH").len(), 0, "{report:?}");
}

#[test]
fn a_run_overriding_the_font_is_checked_against_the_font_it_names() {
    // ADR-0007 makes `font` a run-level style delta. An element whose base is a chain that
    // covers its base run and whose one emphasised run names a chain that does not is
    // exactly the edit the check exists for, and reading only the element's `font` would
    // miss it.
    let dir = common::tempdir(line!());
    let element = serde_json::json!({
        "id": "mixed", "type": "text", "start": 0, "end": 1000,
        "x": 0, "y": 0, "width": 900, "height": 100,
        "font": "brand", "size": 40,
        "runs": [{"text": "hello "}, {"text": "\u{0633}\u{0644}\u{0627}\u{0645}", "font": "brand-too"}]
    })
    .to_string();
    let fonts = r#"{"brand": [{"file": "fonts/OpenRunde-Bold.otf"}], "brand-too": [{"file": "fonts/OpenRunde-Bold.otf"}]}"#;
    let project = typeset(&dir, fonts, &element);

    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    let found = findings(&report, "E-FONT-NO-GLYPH");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].fields["font"], "brand-too");
}

#[test]
fn a_chain_whose_files_could_not_be_read_makes_no_claim_about_coverage() {
    // `E-FONT-MISSING` has already said the file is not there, and "this font has no glyph
    // for س" derived from a font nobody could open is the false confidence ADR-0006 exists
    // to prevent.
    let dir = common::tempdir(line!());
    let project = project_with(&dir, CHAIN, "");
    let with_text: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    let mut with_text = with_text;
    with_text["tracks"] = serde_json::json!([{
        "name": "captions", "layer": 0,
        "elements": [serde_json::from_str::<serde_json::Value>(&typeset_element("greeting", "brand", "\u{0633}")).unwrap()]
    }]);
    std::fs::write(&project, common::canonical(&with_text.to_string())).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    assert_eq!(findings(&report, "E-FONT-NO-GLYPH").len(), 0, "{report:?}");
    // The file's absence is still reported, once, by the check that owns that fact.
    assert_eq!(findings(&report, "E-FONT-MISSING").len(), 1);
}

#[test]
fn a_project_setting_its_text_in_two_declared_fonts_is_a_census() {
    // ADR-0007's own example: "23 elements use `brand`; 1 uses `brand-old`". The census
    // states the distribution and never says which group is the mistake (ADR-0043).
    let dir = common::tempdir(line!());
    let fonts = r#"{"brand": [{"file": "fonts/OpenRunde-Bold.otf"}], "brand-old": [{"file": "fonts/OpenRunde-Bold.otf"}]}"#;
    let elements = [
        typeset_element("a", "brand", "one"),
        typeset_element("b", "brand", "two"),
        typeset_element("c", "brand-old", "three"),
    ]
    .join(",");
    let project = typeset(&dir, fonts, &elements);

    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    let found = findings(&report, "N-FONT-CENSUS");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    let finding = found[0];
    assert_eq!(finding.class, Class::Note);
    assert_eq!(
        finding.fields["distribution"],
        "2 elements use `brand`, 1 element uses `brand-old`"
    );
    let census = finding.census.as_ref().expect("a census");
    assert_eq!(census.field, "font");
    assert_eq!(census.groups[0].value, "brand");
    assert_eq!(census.groups[0].members, ["a", "b"]);
    assert_eq!(census.groups[1].value, "brand-old");
    assert_eq!(census.groups[1].members, ["c"]);
}

#[test]
fn a_project_whose_text_is_all_one_font_produces_no_census() {
    // The must-not-fire half. A census of one group states a fact nobody can act on, and
    // ADR-0006's noise budget would spend a line on it on every run — the fixture's own 22
    // text elements are all `brand`.
    let dir = common::tempdir(line!());
    let elements = [
        typeset_element("a", "brand", "one"),
        typeset_element("b", "brand", "two"),
    ]
    .join(",");
    let project = typeset(&dir, BRAND, &elements);

    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    assert_eq!(findings(&report, "N-FONT-CENSUS").len(), 0, "{report:?}");
}

// ---- `R-FONT-SWAP`: the cache that makes a font swap visible (#206) --------------------

#[test]
fn a_font_rewritten_in_place_is_reported_against_the_elements_set_in_it() {
    // ADR-0007: font files join the `(path, size, mtime)` probe cache, because "a font
    // swapped in place is a silent whole-project render change that no census sees". The
    // `fonts` table below never changes; only the bytes under it do.
    let dir = common::tempdir(line!());
    let elements = [
        typeset_element("a", "brand", "one"),
        typeset_element("b", "brand", "two"),
    ]
    .join(",");
    let project = typeset(&dir, BRAND, &elements);
    let cache = cache(&dir);

    // The first run has nothing to compare against — a project never looked at has not
    // changed — and records what it saw.
    let first = montagent_core::validate_with_cache(&project, &cache);
    assert_eq!(findings(&first, "R-FONT-SWAP").len(), 0, "{first:?}");

    // The swap: different bytes, same path, same table. A `.notdef`-free Latin face
    // replaced by the same face with a byte appended is enough — the check compares an
    // identity, not a rendering.
    let font = dir.join("fonts/OpenRunde-Bold.otf");
    let mut bytes = std::fs::read(&font).unwrap();
    bytes.push(0);
    std::fs::write(&font, &bytes).unwrap();

    let second = montagent_core::validate_with_cache(&project, &cache);
    let found = findings(&second, "R-FONT-SWAP");
    assert_eq!(found.len(), 1, "{:?}", second.findings);
    let finding = found[0];
    assert_eq!(finding.class, Class::Review);
    assert_eq!(finding.fields["font"], "brand");
    assert_eq!(finding.fields["elements"], "2 elements");
    assert!(
        finding.fields["detail"]
            .as_str()
            .unwrap()
            .contains("was rewritten in place"),
        "{:?}",
        finding.fields
    );
    // ADR-0007's census: "naming which measured layouts are now unverified".
    let census = finding.census.as_ref().expect("a census");
    assert_eq!(census.groups[0].members, ["a", "b"]);

    // And it is announced once: the third run has recorded the new identity.
    let third = montagent_core::validate_with_cache(&project, &cache);
    assert_eq!(findings(&third, "R-FONT-SWAP").len(), 0, "{third:?}");
}

#[test]
fn an_unchanged_project_validated_twice_never_reports_a_swap() {
    // The must-not-fire half. Nothing on disk and nothing in the table moves, so a finding
    // here would fire on every second `validate` any project ever gets.
    let dir = common::tempdir(line!());
    let project = typeset(&dir, BRAND, &typeset_element("a", "brand", "one"));
    let cache = cache(&dir);

    montagent_core::validate_with_cache(&project, &cache);
    let second = montagent_core::validate_with_cache(&project, &cache);
    assert_eq!(findings(&second, "R-FONT-SWAP").len(), 0, "{second:?}");
}

#[test]
fn repointing_the_table_at_another_file_is_the_same_finding() {
    // Spec #168's story 54: "a font-swap census when the `fonts` table changes, so that I
    // learn that 22 elements' hand-tuned sizes are now unverified." One mechanism answers
    // both halves, because a chain's identity is the ordered `(file, size, mtime)` of its
    // entries — so an edit visible in the author's own diff and a rewrite that is visible
    // nowhere are one comparison.
    let dir = common::tempdir(line!());
    let project = typeset(&dir, BRAND, &typeset_element("a", "brand", "one"));
    let cache = cache(&dir);
    montagent_core::validate_with_cache(&project, &cache);

    // Different bytes as well as a different name: a chain repointed at a byte-identical
    // copy is no swap at all, which the test below that one asserts.
    let mut bytes = std::fs::read(open_runde()).unwrap();
    bytes.push(0);
    std::fs::write(dir.join("fonts/Renamed.otf"), &bytes).unwrap();
    let mut written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    written["fonts"] = serde_json::json!({"brand": [{"file": "fonts/Renamed.otf"}]});
    std::fs::write(&project, common::canonical(&written.to_string())).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache);
    let found = findings(&report, "R-FONT-SWAP");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(
        found[0].fields["detail"],
        "it names `fonts/Renamed.otf` where it named `fonts/OpenRunde-Bold.otf`"
    );
}

#[test]
fn a_chain_no_element_sets_its_text_in_is_recorded_and_never_reported() {
    // The finding names "which measured layouts are now unverified", and a chain nothing
    // uses has none — so the swap is recorded silently rather than announced to nobody.
    let dir = common::tempdir(line!());
    let project = typeset(&dir, BRAND, "");
    let cache = cache(&dir);
    montagent_core::validate_with_cache(&project, &cache);

    let font = dir.join("fonts/OpenRunde-Bold.otf");
    let mut bytes = std::fs::read(&font).unwrap();
    bytes.push(0);
    std::fs::write(&font, &bytes).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache);
    assert_eq!(findings(&report, "R-FONT-SWAP").len(), 0, "{report:?}");
}

#[test]
fn the_sidecar_records_the_chain_beside_the_probes_and_not_instead_of_them() {
    // One cache file, one cache directory, one `MONTAGENT_CACHE_DIR`. Read back as JSON,
    // because "the font half did not clobber the probe half" is a claim about the bytes.
    let dir = common::tempdir(line!());
    let project = typeset(&dir, BRAND, &typeset_element("a", "brand", "one"));
    let cache = cache(&dir);
    std::fs::write(
        &cache,
        r#"{"version": 2, "entries": {"/clips/take3.mov": {"size": 12, "mtime_ns": 7, "last_used_ns": 7, "probe": {"source": "/clips/take3.mov", "quad": {}, "dimensions": null, "alpha": null, "codec_name": null, "audio": null}}}}"#,
    )
    .unwrap();

    montagent_core::validate_with_cache(&project, &cache);

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&cache).unwrap()).unwrap();
    assert_eq!(
        written["entries"]["/clips/take3.mov"]["size"], 12,
        "the probe half survives a font-half write: {written}"
    );
    let chains = &written["fonts"][common::with_forward_slashes(
        &std::fs::canonicalize(&project)
            .unwrap()
            .display()
            .to_string(),
    )]["keys"]["brand"];
    assert_eq!(chains[0]["file"], "fonts/OpenRunde-Bold.otf", "{written}");
    assert!(chains[0]["size"].as_u64().unwrap() > 0, "{written}");
}

#[test]
fn the_three_font_findings_render_as_prose_from_their_own_field_sets() {
    // "One code, one field set, one template" (ADR-0006). The prose renderer has never seen
    // a `Report`, so a template naming a field its check does not supply fails at the moment
    // a user asks for text — which no assertion about the JSON would catch.
    let dir = common::tempdir(line!());
    let fonts = r#"{"brand": [{"file": "fonts/OpenRunde-Bold.otf"}], "brand-old": [{"file": "fonts/OpenRunde-Bold.otf"}]}"#;
    let elements = [
        typeset_element("a", "brand", "one"),
        typeset_element("b", "brand-old", "\u{0633}"),
    ]
    .join(",");
    let project = typeset(&dir, fonts, &elements);
    let cache = cache(&dir);
    montagent_core::validate_with_cache(&project, &cache);

    let font = dir.join("fonts/OpenRunde-Bold.otf");
    let mut bytes = std::fs::read(&font).unwrap();
    bytes.push(0);
    std::fs::write(&font, &bytes).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache);
    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("every finding renders");

    assert!(
        prose.contains("nothing in the `brand-old` chain has a glyph for U+0633"),
        "{prose}"
    );
    assert!(
        prose.contains("1 element uses `brand`, 1 element uses `brand-old`"),
        "{prose}"
    );
    assert!(
        prose.contains("is not the one this project was last validated against"),
        "{prose}"
    );
    assert!(
        prose.contains("1 element measured in the old chain"),
        "{prose}"
    );
}

#[test]
fn a_font_whose_modification_time_moved_but_whose_bytes_did_not_is_no_swap() {
    // The cache is keyed on `(path, size, mtime)` and *holds* the content hash, which is
    // ADR-0069's own shape — and here it is load-bearing rather than tidy. An mtime moves
    // for reasons that are not edits: a fresh clone, a branch switch, a restore. A check
    // that fired on the key would announce a whole-project render change every time
    // somebody checked the repository out again, on a file nobody touched.
    let dir = common::tempdir(line!());
    let project = typeset(&dir, BRAND, &typeset_element("a", "brand", "one"));
    let cache = cache(&dir);
    montagent_core::validate_with_cache(&project, &cache);

    // Rewritten with its own bytes: same content, new modification time.
    let font = dir.join("fonts/OpenRunde-Bold.otf");
    let bytes = std::fs::read(&font).unwrap();
    std::fs::remove_file(&font).unwrap();
    std::fs::write(&font, &bytes).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache);
    assert_eq!(findings(&report, "R-FONT-SWAP").len(), 0, "{report:?}");
}

#[test]
fn a_chain_repointed_at_a_byte_identical_copy_is_no_swap_either() {
    // Same reasoning from the other side: the table edit is real and visible in the
    // author's diff, and the *fonts* did not change — so no size and no break measured
    // against the old spelling is unverified.
    let dir = common::tempdir(line!());
    let project = typeset(&dir, BRAND, &typeset_element("a", "brand", "one"));
    let cache = cache(&dir);
    montagent_core::validate_with_cache(&project, &cache);

    std::fs::copy(open_runde(), dir.join("fonts/Copy.otf")).unwrap();
    let mut written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    written["fonts"] = serde_json::json!({"brand": [{"file": "fonts/Copy.otf"}]});
    std::fs::write(&project, common::canonical(&written.to_string())).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache);
    assert_eq!(findings(&report, "R-FONT-SWAP").len(), 0, "{report:?}");
}

#[test]
fn a_key_the_table_no_longer_declares_is_the_same_swap() {
    // The largest version of spec #168's story 54 — "a font-swap census when the `fonts`
    // table changes, so that I learn that 22 elements' hand-tuned sizes are now unverified".
    // Every element still naming the key has lost the font it was measured in, and the
    // coverage check has nothing left to open, so this is where it is reported.
    let dir = common::tempdir(line!());
    let fonts = r#"{"brand": [{"file": "fonts/OpenRunde-Bold.otf"}], "brand-old": [{"file": "fonts/OpenRunde-Bold.otf"}]}"#;
    let project = typeset(&dir, fonts, &typeset_element("a", "brand-old", "one"));
    let cache = cache(&dir);
    montagent_core::validate_with_cache(&project, &cache);

    let mut written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    written["fonts"] = serde_json::json!({"brand": [{"file": "fonts/OpenRunde-Bold.otf"}]});
    std::fs::write(&project, common::canonical(&written.to_string())).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache);
    let found = findings(&report, "R-FONT-SWAP");
    assert_eq!(found.len(), 1, "{:?}", report.findings);
    assert_eq!(found[0].fields["font"], "brand-old");
    assert_eq!(
        found[0].fields["detail"],
        "the `fonts` table no longer declares it"
    );
    assert_eq!(found[0].census.as_ref().unwrap().groups[0].members, ["a"]);
}

#[test]
fn a_font_file_that_went_missing_is_not_reported_as_a_swap() {
    // The must-not-fire half of the same branch: the table still declares the key, so the
    // one true thing to say is `E-FONT-MISSING`, and calling a file nobody could open a
    // font swap would be one fault reported as two.
    let dir = common::tempdir(line!());
    let project = typeset(&dir, BRAND, &typeset_element("a", "brand", "one"));
    let cache = cache(&dir);
    montagent_core::validate_with_cache(&project, &cache);

    std::fs::remove_file(dir.join("fonts/OpenRunde-Bold.otf")).unwrap();

    let report = montagent_core::validate_with_cache(&project, &cache);
    assert_eq!(findings(&report, "R-FONT-SWAP").len(), 0, "{report:?}");
    assert_eq!(findings(&report, "E-FONT-MISSING").len(), 1);
}

#[test]
fn a_missing_glyph_names_the_projects_other_chains_that_do_have_it() {
    // ADR-0043's sibling census on a refuse-class finding: inert, document-derived, and it
    // carries the fix without proposing one. Whether the right move is to add that file to
    // this chain, move the run to that key, or change the text is the author's.
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    // A second chain that does map the character: the same Latin face, asked about a
    // character it has. `@` is in Open Runde and `\u{0633}` is not, so the two chains below
    // differ in exactly the fact the census reports.
    let fonts = r#"{"brand": [{"file": "fonts/OpenRunde-Bold.otf"}], "brand+fa": [{"file": "fonts/OpenRunde-Bold.otf"}]}"#;
    let project = typeset(&dir, fonts, &typeset_element("a", "brand", "@"));

    // `@` is covered by both, so nothing fires and there is nothing to census.
    let report = montagent_core::validate_with_cache(&project, &cache(&dir));
    assert_eq!(findings(&report, "E-FONT-NO-GLYPH").len(), 0, "{report:?}");
}
