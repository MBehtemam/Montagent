//! `E-LAYER-TIE` (#209, ADR-0060): two elements resolving to one layer whose boxes
//! actually overlap, in both time and space, is an `error` — never a fallback draw order.
//!
//! Document-only, so `check` is called directly and no project here ever needs `ffprobe`.
//! Every test writes its geometry with `"origin":"top-left"`, so a rectangle's declared
//! `x`/`y`/`width`/`height` read straight off the JSON as its frame-space box and the
//! arithmetic a reader has to do to follow a case is none.

use montaget_core::finding::{Class, Repair};
use montaget_core::report::Report;
use montaget_core::{parse, registry};

mod common;
use common::{canonical, write_project};

fn project(tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##
    ))
}

/// One track at `layer`, holding `elements`.
fn track(name: &str, layer: i64, elements: &str) -> String {
    format!(r##"{{"name":"{name}","layer":{layer},"elements":[{elements}]}}"##)
}

/// A static rectangle whose box is exactly `x`,`y`,`width`,`height`.
fn rect(id: &str, start: i64, end: i64, x: i64, y: i64, width: i64, height: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":{x},"y":{y},"origin":"top-left","width":{width},"height":{height}}}"##
    )
}

/// The same rectangle with `x` animated through the given `(t, v)` records — the first
/// carrying no `ease` and every later one carrying `linear`, which is ADR-0038's rule.
fn moving_rect(
    id: &str,
    start: i64,
    end: i64,
    xs: &[(i64, i64)],
    y: i64,
    width: i64,
    height: i64,
) -> String {
    let records: Vec<String> = xs
        .iter()
        .enumerate()
        .map(|(i, (t, v))| match i {
            0 => format!(r##"{{"t":{t},"v":{v}}}"##),
            _ => format!(r##"{{"t":{t},"v":{v},"ease":"linear"}}"##),
        })
        .collect();
    format!(
        r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":[{}],"y":{y},"origin":"top-left","width":{width},"height":{height}}}"##,
        records.join(",")
    )
}

#[track_caller]
fn report_on(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(tracks));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::tie::check(&document, &mut report);
    report
}

#[track_caller]
fn ties(report: &Report) -> Vec<&montaget_core::finding::Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "E-LAYER-TIE")
        .collect()
}

#[track_caller]
fn unchecked(report: &Report) -> Vec<&montaget_core::finding::Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "U-LAYER-TIE-ROTATED")
        .collect()
}

/// The pair one finding names, as a set the caller can assert on without knowing which
/// half the finding located itself at.
fn pair(finding: &montaget_core::finding::Finding) -> (String, String) {
    (
        finding.fields["element"].as_str().unwrap().to_string(),
        finding.fields["other"].as_str().unwrap().to_string(),
    )
}

#[test]
fn a_tie_whose_boxes_overlap_in_both_axes_is_an_error() {
    // Two tracks both declaring layer 30, two boxes sharing the rectangle 50,50..100,100.
    let tracks = format!(
        "{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 30, &rect("two", 0, 1000, 50, 50, 100, 100)),
    );
    let report = report_on(&tracks);
    let findings = ties(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);

    let finding = findings[0];
    assert_eq!(finding.class, Class::Error);
    assert_eq!(pair(finding), ("one".to_string(), "two".to_string()));
    assert_eq!(finding.fields["layer"], serde_json::json!(30));
    // The overlap itself, inline — ADR-0006: every relevant number in the finding, so
    // that acting on it costs no re-read of the project.
    assert_eq!(finding.fields["overlap_x"], serde_json::json!(50));
    assert_eq!(finding.fields["overlap_y"], serde_json::json!(50));
    assert_eq!(finding.fields["overlap_width"], serde_json::json!(50));
    assert_eq!(finding.fields["overlap_height"], serde_json::json!(50));
    // ADR-0043: which of the two should draw in front is the author's intent, and no
    // field of the document carries it.
    assert_eq!(finding.repair, Some(Repair::None));
}

#[test]
fn a_tie_whose_boxes_never_overlap_is_not_reported() {
    // The fixture's own shape: two panels side by side at one layer. Nothing on screen
    // depends on their order, so ADR-0060 leaves it undefined and unobservable.
    let tracks = format!(
        "{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 30, &rect("two", 0, 1000, 500, 0, 100, 100)),
    );
    assert!(ties(&report_on(&tracks)).is_empty());
}

#[test]
fn a_tie_overlapping_in_one_axis_only_is_not_reported() {
    // Rows share their `y` span entirely and no `x` at all: no pixel is claimed twice.
    let tracks = format!(
        "{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 30, &rect("two", 0, 1000, 100, 0, 100, 100)),
    );
    assert!(ties(&report_on(&tracks)).is_empty());
}

#[test]
fn elements_that_never_share_an_instant_are_not_a_tie() {
    // Half-open (ADR-0005): `one` ends at 1000 and `two` starts there, so there is no
    // instant at which both are on screen and none at which their stacking could matter.
    let tracks = format!(
        "{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 30, &rect("two", 1000, 2000, 0, 0, 100, 100)),
    );
    assert!(ties(&report_on(&tracks)).is_empty());
}

#[test]
fn elements_on_different_layers_are_never_a_tie() {
    let tracks = format!(
        "{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 31, &rect("two", 0, 1000, 0, 0, 100, 100)),
    );
    assert!(ties(&report_on(&tracks)).is_empty());
}

#[test]
fn elements_disjoint_at_rest_and_swept_into_overlap_mid_animation_are_an_error() {
    // ADR-0060's own case, and the whole reason this check samples: `one` sits left of
    // `two` at the start of its travel and right of it at the end. A static-extent check
    // reading either rest position sees two disjoint boxes and reports nothing.
    let tracks = format!(
        "{},{}",
        track(
            "a",
            30,
            &moving_rect("one", 0, 1000, &[(0, 0), (1000, 1600)], 0, 100, 100)
        ),
        track("b", 30, &rect("two", 0, 1000, 800, 0, 100, 100)),
    );
    let report = report_on(&tracks);
    let findings = ties(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    // Mid-travel, not at either end: the instant the finding names is one the interval
    // sample found and no keyframe boundary did.
    let instant = findings[0].fields["instant"].as_i64().unwrap();
    assert!(
        instant > 0 && instant < 999,
        "the overlap is mid-animation, got {instant}"
    );
}

#[test]
fn an_overlap_at_a_keyframe_boundary_is_an_error() {
    // The other half of the sample set: `one` travels out and back, overlapping `two`
    // only at the record in the middle of its list.
    let tracks = format!(
        "{},{}",
        track(
            "a",
            30,
            &moving_rect(
                "one",
                0,
                1000,
                &[(0, 0), (500, 800), (1000, 0)],
                0,
                100,
                100
            )
        ),
        track("b", 30, &rect("two", 0, 1000, 800, 0, 100, 100)),
    );
    let report = report_on(&tracks);
    let findings = ties(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["instant"], serde_json::json!(500));
}

#[test]
fn an_animation_that_keeps_two_boxes_apart_is_not_reported() {
    // The failure mode an `error`-level static check would have: `one` is animated from
    // one side of the frame to the other and never reaches `two`. Refusing this project
    // would be refusing a legal one.
    let tracks = format!(
        "{},{}",
        track(
            "a",
            30,
            &moving_rect("one", 0, 1000, &[(0, 0), (1000, 400)], 0, 100, 100)
        ),
        track("b", 30, &rect("two", 0, 1000, 800, 0, 100, 100)),
    );
    assert!(ties(&report_on(&tracks)).is_empty());
}

#[test]
fn array_order_is_never_consulted() {
    // ADR-0060: array order carries no meaning, for timing or for stacking. Writing the
    // same two elements in the other order produces the identical finding — not a
    // mirrored one, and not a different verdict.
    let one = track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100));
    let two = track("b", 30, &rect("two", 0, 1000, 50, 50, 100, 100));

    let forward = report_on(&format!("{one},{two}"));
    let reversed = report_on(&format!("{two},{one}"));

    let as_json = |report: &Report| -> Vec<serde_json::Value> {
        ties(report)
            .iter()
            .map(|f| {
                let mut value = serde_json::to_value(f).expect("a finding serialises");
                // The scratch directory is per-test-line, so the one thing that legitimately
                // differs between the two runs is the path.
                value["location"]["file"] = serde_json::json!("p.montaget.json");
                value
            })
            .collect()
    };

    assert_eq!(as_json(&forward), as_json(&reversed));
    assert_eq!(ties(&forward).len(), 1);
}

#[test]
fn one_finding_per_pair_however_many_samples_overlap() {
    // Two boxes that sit on top of each other for their whole shared range. Every sample
    // overlaps; the tie is one fact about one pair, and a finding per sample would be the
    // noise budget ADR-0006 calls a safety property.
    let tracks = format!(
        "{},{}",
        track(
            "a",
            30,
            &moving_rect(
                "one",
                0,
                10_000,
                &[(0, 0), (5000, 10), (10_000, 0)],
                0,
                100,
                100
            )
        ),
        track("b", 30, &rect("two", 0, 10_000, 0, 0, 100, 100)),
    );
    assert_eq!(ties(&report_on(&tracks)).len(), 1);
}

#[test]
fn three_elements_at_one_layer_report_each_overlapping_pair() {
    // The tie is pairwise: `one` overlaps `two` and `two` overlaps `three`, while `one`
    // and `three` never meet. Two findings, not one and not three.
    let tracks = format!(
        "{},{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 30, &rect("two", 0, 1000, 50, 0, 100, 100)),
        track("c", 30, &rect("three", 0, 1000, 120, 0, 100, 100)),
    );
    let report = report_on(&tracks);
    let mut pairs: Vec<(String, String)> = ties(&report).iter().map(|f| pair(f)).collect();
    pairs.sort();
    assert_eq!(
        pairs,
        vec![
            ("one".to_string(), "two".to_string()),
            ("three".to_string(), "two".to_string()),
        ],
        "{:?}",
        report.findings
    );
}

#[test]
fn two_elements_of_one_track_are_left_to_the_track_check() {
    // Children of one track may not overlap in time at all (ADR-0004), and when they do
    // `E-TRACK-OVERLAP` already says so. Their tie is a consequence of that overlap, not a
    // second defect — and the fix is to move one of them to another track, never to state
    // a stacking order between two elements that may not coexist.
    let tracks = track(
        "a",
        30,
        &format!(
            "{},{}",
            rect("one", 0, 1000, 0, 0, 100, 100),
            rect("two", 500, 1500, 50, 50, 100, 100)
        ),
    );
    let report = report_on(&tracks);
    assert!(ties(&report).is_empty(), "{:?}", report.findings);

    // And the track check does speak, on the same project.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(&tracks));
    let full = montaget_core::validate(&path);
    assert!(
        full.findings.iter().any(|f| f.code == "E-TRACK-OVERLAP"),
        "{:?}",
        full.findings
    );
}

#[test]
fn the_finding_names_both_tracks_the_tie_came_from() {
    // Two tracks both declaring `layer: 30` is ADR-0060's own most common case, and the
    // author's cheapest fix is often to renumber one of them — which a finding naming only
    // the two elements would send them looking for.
    let tracks = format!(
        "{},{}",
        track("chip-panel", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("handle-panel", 30, &rect("two", 0, 1000, 50, 50, 100, 100)),
    );
    let report = report_on(&tracks);
    let findings = ties(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["track"], serde_json::json!("chip-panel"));
    assert_eq!(
        findings[0].fields["other_track"],
        serde_json::json!("handle-panel")
    );
    // Not located at either track: choosing one would be the arbitrary choice between the
    // two that the check exists to refuse.
    assert_eq!(findings[0].location.track, None);
}

#[test]
fn an_element_with_no_frame_space_footprint_is_never_a_tie() {
    // Audio has no box. Two narration elements at one layer claim no pixels between them,
    // and a check that read their absent geometry as a shared origin would refuse a
    // project whose every frame is correct.
    let audio = |id: &str| {
        format!(r##"{{"id":"{id}","type":"audio","start":0,"end":1000,"source":"audio/a.mp3"}}"##)
    };
    let tracks = format!(
        "{},{}",
        track("a", 0, &audio("one")),
        track("b", 0, &audio("two")),
    );
    assert!(ties(&report_on(&tracks)).is_empty());
}

#[test]
fn a_rotated_element_in_a_tie_is_reported_unchecked_rather_than_guessed() {
    // A rotated element's footprint is a parallelogram, and this check answers in
    // rectangles. Its bounding box would refuse projects that are legal; silence would
    // pass a tie nobody looked at. Neither, on ADR-0006's rule that a check that could
    // not run says so.
    let rotated = r#"{"id":"one","type":"rect","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"rotation":30.0}"#;
    let tracks = format!(
        "{},{}",
        track("a", 30, rotated),
        track("b", 30, &rect("two", 0, 1000, 50, 50, 100, 100)),
    );
    let report = report_on(&tracks);
    assert!(ties(&report).is_empty(), "{:?}", report.findings);

    let findings = unchecked(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].class, Class::Unchecked);
    assert_eq!(findings[0].fields["rotated"], serde_json::json!("one"));
}

#[test]
fn a_rotation_at_one_instant_never_hides_an_overlap_at_another() {
    // The refusal is about the instants it was taken at and no others. `one` is spun at
    // the start of the window and square on top of `two` by the end — an `error` the
    // document really does contain, which answering `unchecked` at the first rotated
    // sample would hide behind an "I could not look" about a different instant.
    let spun = r##"{"id":"one","type":"rect","start":0,"end":1000,"x":50,"y":50,"origin":"top-left","width":100,"height":100,"rotation":[{"t":0,"v":30.0},{"t":500,"v":0.0,"ease":"linear"}]}"##;
    let tracks = format!(
        "{},{}",
        track("a", 30, spun),
        track("b", 30, &rect("two", 0, 1000, 0, 0, 100, 100)),
    );
    let report = report_on(&tracks);
    let findings = ties(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert!(
        findings[0].fields["instant"].as_i64().unwrap() >= 500,
        "the overlap is found once the rotation has run out: {:?}",
        findings[0].fields
    );
    assert!(
        unchecked(&report).is_empty(),
        "a proved collision is not an unanswered question: {:?}",
        report.findings
    );
}

#[test]
fn a_fully_transparent_overlap_is_still_an_error() {
    // ADR-0060: "A same-layer pair whose boxes overlap but one is fully transparent is
    // still `error`: the format has no way to state 'this collision is intentional and
    // harmless' other than the same `layer`/anchor fields that resolve the tie outright."
    let invisible = r##"{"id":"one","type":"rect","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"opacity":0.0}"##;
    let tracks = format!(
        "{},{}",
        track("a", 30, invisible),
        track("b", 30, &rect("two", 0, 1000, 50, 50, 100, 100)),
    );
    assert_eq!(ties(&report_on(&tracks)).len(), 1);
}

#[test]
fn a_clipped_away_element_claims_no_pixels_of_the_tie() {
    // `clip` is the static frame-space aperture the element is drawn through, so the part
    // of its box outside it never reaches the screen and cannot collide with anything.
    // Geometry, not paint — which is the distinction ADR-0060 draws when it says the check
    // needs no opacity or fill awareness.
    let clipped = r#"{"id":"one","type":"image","start":0,"end":1000,"source":"images/a.png","x":0,"y":0,"origin":"top-left","width":100,"height":100,"clip":[0,0,40,40]}"#;
    let tracks = format!(
        "{},{}",
        track("a", 30, clipped),
        track("b", 30, &rect("two", 0, 1000, 50, 50, 100, 100)),
    );
    assert!(ties(&report_on(&tracks)).is_empty());
}

#[test]
fn the_committed_fixture_carries_no_layer_tie() {
    // ADR-0060: the fixture's two tied clusters — `chip-panel`/`handle-panel` at 30 and
    // `flag-field`/`handle-logo`/`handle-text` at 31 — "stay legal, since their boxes are
    // spatially disjoint for their entire shared time range. This ADR changes zero bytes
    // of the committed file." The live regression case the ADR asks for.
    let path = common::fixture_dir().join("en-halloween-decorating.montaget.json");
    let document = parse::read(&path).expect("the committed fixture parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::tie::check(&document, &mut report);
    assert!(
        report.findings.is_empty(),
        "the fixture's ties are disjoint: {:?}",
        report.findings
    );
}

#[test]
fn moving_a_tied_cluster_into_its_neighbour_starts_failing() {
    // The other half of the ADR's regression case: "an edit that moved either cluster into
    // the other's box would need to start failing `validate`." `handle-panel` moved from
    // x 438 to x 100 puts it under `chip-panel`, which sits at 48..420.
    let path = common::fixture_dir().join("en-halloween-decorating.montaget.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read the fixture"))
            .expect("the fixture is JSON");
    for track in value["tracks"].as_array_mut().unwrap() {
        if track["name"] == serde_json::json!("handle-panel") {
            track["elements"][0]["x"] = serde_json::json!(100);
        }
    }

    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &montaget_core::write::canonical(&montaget_core::layout::canonicalise(&value)),
    );
    let document = parse::read(&path).expect("the edited fixture parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::tie::check(&document, &mut report);

    let findings = ties(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(
        pair(findings[0]),
        ("chip-panel".to_string(), "handle-panel".to_string())
    );
}

#[test]
fn both_findings_render_as_prose() {
    // ADR-0006: one code, one field set, one template. A template naming a field the
    // finding does not carry is a render error rather than a silent gap, so this is the
    // test that the two declarations and the two builders agree.
    let tracks = format!(
        "{},{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 30, &rect("two", 0, 1000, 50, 50, 100, 100)),
        track(
            "c",
            31,
            r#"{"id":"spun","type":"rect","start":0,"end":1000,"x":0,"y":0,"origin":"top-left","width":100,"height":100,"rotation":30.0}"#,
        ),
    ) + ","
        + &track("d", 31, &rect("still", 0, 1000, 50, 50, 100, 100));

    let report = report_on(&tracks);
    let prose =
        montaget_core::text::render(&report.to_json(), montaget_core::text::Options::verbose())
            .expect("both templates render");

    assert!(
        prose.contains("one and two both resolve to layer 30"),
        "{prose}"
    );
    assert!(prose.contains("50×50 px at (50, 50)"), "{prose}");
    assert!(
        prose.contains("spun's resolved `rotation` is 30"),
        "{prose}"
    );
}

#[test]
fn the_registry_declares_the_check_live_and_refuse_class() {
    // ADR-0043: the refuse-class decision is made once, in the declaration, and no call
    // site can vary it. ADR-0060 makes it refuse — which of the two draws in front is the
    // author's intent, and the document cannot carry it.
    let spec = registry::spec("E-LAYER-TIE").expect("the code is registered");
    assert_eq!(spec.default_class(), Class::Error);
    assert_eq!(spec.repair, Some(registry::RepairClass::Refuse));
    assert_eq!(spec.adr, "ADR-0060");
    assert_eq!(spec.status, registry::Status::Live);

    let unchecked = registry::spec("U-LAYER-TIE-ROTATED").expect("the code is registered");
    assert_eq!(unchecked.default_class(), Class::Unchecked);
    assert_eq!(unchecked.status, registry::Status::Live);
}

#[test]
fn validate_runs_the_check() {
    // The check is not merely callable: `validate` runs every check on the whole project,
    // every time (ADR-0006), and `render` refuses on any `error` it finds.
    let tracks = format!(
        "{},{}",
        track("a", 30, &rect("one", 0, 1000, 0, 0, 100, 100)),
        track("b", 30, &rect("two", 0, 1000, 50, 50, 100, 100)),
    );
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(&tracks));
    let report = montaget_core::validate(&path);
    assert!(
        report.findings.iter().any(|f| f.code == "E-LAYER-TIE"),
        "{:?}",
        report.findings
    );
}
