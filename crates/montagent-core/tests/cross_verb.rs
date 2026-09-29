//! ADR-0093 ruling 3's invariant, asserted **across both verbs**.
//!
//! > `validate`'s unchecked set contains every source `render` declines to use.
//!
//! This file exists because of how the MONTAGENT-1 defect was shaped. `validate` reported
//! `0 unchecked` on a file `render` then declined to mix 134 elements of, in the same
//! session, and **neither verb was wrong about its own data** — they answered one question
//! from two data structures. A unit test on either path would have passed throughout. So the
//! property is stated over the pair, and the ruling says so in as many words: *"written as a
//! real cross-verb property, not a unit test on one path, or the divergence returns."*
//!
//! A **containment**, not an equality, on purpose: `render` mixes local sources only, so it
//! declines a remote one `validate` probed perfectly well. That is a fact about the verb, not
//! about the file. The direction that must never hold is the other one — a source `render`
//! cannot use that `validate` called a clean pass.

use std::path::Path;

use montagent_core::finding::Class;
use montagent_core::report::Report;
use montagent_core::validate;
use montagent_core::verbs::render::{Ask, Progress, render};

mod common;
use common::{fixture_dir, has_ffprobe, tempdir, write_project};

/// Every code by which `render` says *"I could not use this source"*.
///
/// The closed set is the point: before ADR-0093 this was a free-text `String` assembled at
/// whichever arm noticed, so a test could not name the condition it was looking for.
const DECLINED_FOR_SOURCE: &[&str] = &[
    "E-NOT-MIXED-UNESTABLISHED",
    "E-NOT-MIXED-UNREADABLE",
    "E-NOT-PAINTED-UNREADABLE",
    "E-NOT-PAINTED-UNDECODABLE",
];

fn project(dir: &Path, source: &str) -> std::path::PathBuf {
    write_project(
        dir,
        "cross.montagent.json",
        &format!(
            r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",
                "duration":1000,"output":"out/cross.mp4",
                "tracks":[{{"name":"only","layer":0,"elements":[
                  {{"id":"vo","type":"audio","start":0,"end":1000,
                    "source":"{source}","source_start":0,"source_end":1000}}
                ]}}]}}"##
        ),
    )
}

/// Every element `render` declined, with the source its element declared.
fn declined_sources(report: &Report, document: &serde_json::Value) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| DECLINED_FOR_SOURCE.contains(&finding.code.as_str()))
        .filter_map(|finding| finding.location.element.clone())
        .filter_map(|id| source_of(document, &id))
        .collect()
}

fn source_of(document: &serde_json::Value, id: &str) -> Option<String> {
    document["tracks"]
        .as_array()?
        .iter()
        .flat_map(|track| track["elements"].as_array().into_iter().flatten())
        .find(|element| element["id"].as_str() == Some(id))
        .and_then(|element| element["source"].as_str())
        .map(str::to_string)
}

/// Does `validate` say anything at all about this source that is not a clean pass?
///
/// `UNCHECKED` **or** `error`: the ruling's set is *"the unchecked set"*, and an `error`
/// about the same source is a strictly louder statement of the same thing. What fails the
/// invariant is silence.
fn validate_spoke_about(report: &Report, source: &str) -> bool {
    report.findings.iter().any(|finding| {
        matches!(finding.class, Class::Unchecked | Class::Error)
            && finding
                .fields
                .values()
                .chain(finding.fields.values())
                .any(|value| value.as_str().is_some_and(|text| text.contains(source)))
    })
}

/// The invariant itself, over one project.
fn assert_containment(path: &Path) {
    let document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("project")).expect("json");

    let checked = validate(path);
    let rendered = render(path, &Ask::default(), &mut |_: Progress| {});

    for source in declined_sources(rendered.report(), &document) {
        assert!(
            validate_spoke_about(&checked, &source),
            "`render` declined `{source}` and `validate` passed it clean — the two verbs \
             disagree about one file in one session (ADR-0093 ruling 3). validate said: {:?}",
            checked.findings
        );
    }
}

#[test]
fn a_source_render_cannot_read_is_never_a_clean_pass_to_validate() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let source = dir.join("unreadable.mp3");
    std::fs::copy(fixture_dir().join("audio/05-cobweb.mp3"), &source).expect("copied");

    // Present and unopenable: ADR-0056's *"something is there, so it is not a confirmed
    // absence, and nothing about its content was learned"*.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    }
    #[cfg(not(unix))]
    {
        eprintln!("skipped: needs POSIX permissions to make a present file unreadable");
        return;
    }

    let path = project(&dir, "unreadable.mp3");

    // The containment is only worth asserting if `render` declines at all, so say so first.
    // Without this the test passes on an empty declined set — which is exactly how it would
    // rot if the code were ever renamed.
    let document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("project")).expect("json");
    let rendered = render(&path, &Ask::default(), &mut |_: Progress| {});
    assert!(
        !declined_sources(rendered.report(), &document).is_empty(),
        "`render` mixed a source it cannot open, so the containment below proves nothing: {:?}",
        rendered.report().findings
    );

    assert_containment(&path);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o644));
    }
}

#[test]
fn a_confirmed_absence_is_an_error_in_both_verbs_and_reaches_no_painter() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    let path = project(&dir, "absent.mp3");

    // The containment holds trivially here and that is the point worth pinning: a confirmed
    // absence is `E-SOURCE-MISSING`, an `error`, so `render` refuses on the check engine's
    // own report (ADR-0006) and never reaches the mix. The declined set is empty because
    // nothing was attempted, not because something was passed over in silence.
    assert_containment(&path);

    let checked = validate(&path);
    assert!(
        checked
            .findings
            .iter()
            .any(|finding| finding.code == "E-SOURCE-MISSING"),
        "{:?}",
        checked.findings
    );
    let rendered = render(&path, &Ask::default(), &mut |_: Progress| {});
    assert!(
        rendered.video().is_none(),
        "a project whose source is not there produced a file"
    );
}

#[test]
fn a_source_both_verbs_can_use_declines_nothing() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    let dir = tempdir(line!());
    std::fs::copy(
        fixture_dir().join("audio/05-cobweb.mp3"),
        dir.join("vo.mp3"),
    )
    .expect("copied");
    let path = project(&dir, "vo.mp3");

    let document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("project")).expect("json");
    let rendered = render(&path, &Ask::default(), &mut |_: Progress| {});

    // The other direction, so the test above cannot pass by `render` declining nothing at
    // all: a source both verbs can use is mixed, and the containment is vacuous rather than
    // untested.
    assert!(
        declined_sources(rendered.report(), &document).is_empty(),
        "a readable source was declined: {:?}",
        rendered.report().findings
    );
}

// ---------------------------------------------------------------------------
// The same containment for a document fact rather than a source (#410, ADR-0107)
// ---------------------------------------------------------------------------

/// An empty range is the other shape ruling 3 closes: `render` refused it as
/// `E-EMPTY-RANGE` while `validate` passed the document at zero errors. The containment is
/// stated over **codes** here rather than sources — every document-fact refusal `render`
/// makes, `validate` makes as an `error` — and it is also required that the refusal come
/// from the check engine rather than from the mix, which is what makes `render`'s own two
/// arms `E-INTERNAL`.
#[test]
fn a_range_render_refuses_is_an_error_to_validate_under_the_same_code() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe on PATH (ADR-0009)");
        return;
    }
    for (label, element) in [
        (
            "`start`..`end`",
            r#""start":500,"end":500,"source_start":0,"source_end":1000"#,
        ),
        (
            "`source_start`..`source_end`",
            r#""start":0,"end":1000,"source_start":400,"source_end":400"#,
        ),
    ] {
        let dir = tempdir(line!());
        std::fs::copy(
            fixture_dir().join("audio/05-cobweb.mp3"),
            dir.join("vo.mp3"),
        )
        .expect("copied");
        let path = write_project(
            &dir,
            "cross.montagent.json",
            &format!(
                r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",
                    "duration":1000,"output":"out/cross.mp4",
                    "tracks":[{{"name":"only","layer":0,"elements":[
                      {{"id":"vo","type":"audio","source":"vo.mp3",{element}}}
                    ]}}]}}"##
            ),
        );

        let empty = |report: &Report| {
            report
                .findings
                .iter()
                .filter(|f| f.code == "E-EMPTY-RANGE" && f.fields["field"] == label)
                .map(|f| f.class)
                .collect::<Vec<_>>()
        };

        let checked = validate(&path);
        assert_eq!(
            empty(&checked),
            vec![Class::Error],
            "{label}: `validate` must state the fact `render` refuses on: {:?}",
            checked.findings
        );

        let rendered = render(&path, &Ask::default(), &mut |_: Progress| {});
        assert_eq!(empty(rendered.report()), vec![Class::Error], "{label}");
        assert!(
            rendered
                .report()
                .findings
                .iter()
                .all(|f| f.code != "E-INTERNAL"),
            "{label}: the mix was reached, so the check engine did not refuse first: {:?}",
            rendered.report().findings
        );
        assert!(rendered.video().is_none(), "{label}: a file was produced");
    }
}
