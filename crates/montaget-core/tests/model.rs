//! The document model, against the one real project file.
//!
//! Every assertion here goes through the committed fixture rather than a hand-written
//! sample. ADR-0003's asymmetry says the fixture is evidence a capability is *needed* and
//! never that one is unneeded — so it is the right thing to pin and the wrong thing to
//! treat as the whole format.

use montaget_core::model::{Body, Project};
use montaget_core::write;

const FIXTURE: &str =
    "../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json";

fn fixture() -> String {
    std::fs::read_to_string(FIXTURE).expect("the committed fixture")
}

/// The committed fixture is LF, and says so before the round-trip test has to
/// discover it the hard way.
///
/// `write::canonical` emits `\n`, so a checkout that rewrote the committed bytes
/// to CRLF would fail the round-trip with a diff that is invisible in a terminal.
/// This fails first and names the cause.
#[test]
fn the_committed_fixture_is_checked_out_with_lf_line_endings() {
    let source = fixture();
    assert!(
        !source.contains('\r'),
        "the fixture on disk contains CR, so this checkout rewrote its line endings. \
         ADR-0041 makes the canonical convention a byte convention and \
         `write::canonical` emits LF, so `.gitattributes` pins `eol=lf` for every \
         platform. If this fires, that file is missing or is not being applied."
    );
}

#[test]
fn the_committed_fixture_round_trips_byte_for_byte() {
    let source = fixture();

    let project: Project = serde_json::from_str(&source).expect("the fixture parses");
    let written = write::canonical(&serde_json::to_value(&project).unwrap());

    // Compared as bytes, not as values: the whole point of the canonical convention is
    // that an exact-string replace an agent wrote yesterday still matches today, and a
    // value comparison would pass while the bytes moved underneath it (ADR-0005).
    if written != source {
        let (line, expected, actual) = first_difference(&source, &written);
        panic!(
            "the fixture does not round-trip, first difference at line {line}\n\
             expected: {expected}\n\
             actual:   {actual}"
        );
    }
}

#[test]
fn the_fixture_is_read_as_the_types_it_is_written_in() {
    let project: Project = serde_json::from_str(&fixture()).unwrap();

    assert_eq!(project.frame.width, 1080);
    assert_eq!(project.fps, 25);
    assert_eq!(project.duration, Some(65216));
    assert_eq!(project.tracks.len(), 14);

    let photo = &project.tracks[0];
    assert_eq!(photo.name, "photo");
    assert_eq!(photo.layer, 10);
    assert!(matches!(photo.elements[0].body, Body::Image(_)));
    assert_eq!(photo.elements[0].group.as_deref(), Some("item-05"));

    // The narration track is the only one carrying time-based elements, and the fixture's
    // slowed retakes are what pinned `speed` as a rate multiplier rather than its
    // reciprocal (ADR-0020).
    let narration = project
        .tracks
        .iter()
        .find(|t| t.name == "narration")
        .unwrap();
    let slowed = narration
        .elements
        .iter()
        .find(|e| e.id == "vo-sentence-05-b")
        .unwrap();
    match &slowed.body {
        Body::Audio(audio) => assert_eq!(audio.speed, Some(0.645)),
        other => panic!("vo-sentence-05-b is audio, not {}", other.type_name()),
    }
}

#[test]
fn an_element_carries_no_duration_and_its_range_is_half_open() {
    let project: Project = serde_json::from_str(&fixture()).unwrap();
    let photo = &project.tracks[0];

    // ADR-0005: `end` is the instant the element is no longer on screen, and its neighbour
    // starting there is. Adjacency is exact equality, which is the predicate the overlap
    // and gap rules are built on — and the reason the times are integers.
    for pair in photo.elements.windows(2) {
        assert_eq!(
            pair[0].end, pair[1].start,
            "{} and {} are adjacent in the fixture",
            pair[0].id, pair[1].id
        );
    }

    // There is no `duration` field to assert the absence of — the type does not have one.
    // What can be asserted is that the JSON never grows one.
    let written = serde_json::to_value(&project).unwrap();
    let element = &written["tracks"][0]["elements"][0];
    assert!(
        element.get("duration").is_none(),
        "an element states `start` and `end`, never a duration"
    );
}

/// The first line where two documents disagree, for a failure message that names the place
/// rather than printing four thousand characters twice.
/// Split on `\n` rather than with `lines()`, which strips `\r\n` and `\n` alike.
///
/// That blindness is not hypothetical: when a Windows checkout rewrote the
/// fixture's line endings, every line compared equal, the walk ran to the end of
/// both files, and the failure reported `expected: <end of file>` against
/// `actual: <end of file>` — a difference this function could not see and a
/// message that named nothing (#189). Keeping the `\r` on the line makes it a
/// difference like any other, and `escape_debug` makes it visible in the panic.
fn first_difference(expected: &str, actual: &str) -> (usize, String, String) {
    fn render(line: Option<&str>) -> String {
        match line {
            Some(text) => format!("\"{}\"", text.escape_debug()),
            None => "<end of file>".into(),
        }
    }

    let mut expected_lines = expected.split('\n');
    let mut actual_lines = actual.split('\n');
    let mut line = 0;
    loop {
        line += 1;
        match (expected_lines.next(), actual_lines.next()) {
            (None, None) => return (line, "<end of file>".into(), "<end of file>".into()),
            (a, b) if a != b => return (line, render(a), render(b)),
            _ => {}
        }
    }
}
