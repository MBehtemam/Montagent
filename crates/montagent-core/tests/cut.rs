//! `R-SOURCE-CUT-POP` (#211, ADR-0033, ADR-0062): two adjacent elements of one track
//! drawing the same source with no gap, whose resolved transform values disagree across
//! the cut — the viewer sees one continuous shot and the frame pops.
//!
//! Document-only, so `check` is called directly and no project here ever needs `ffprobe`.
//! Sources are bare relative paths that need not exist: source *identity* is a comparison
//! of resolved paths, which ADR-0033 calls "a fact derivable from the document", and
//! nothing in this check opens a file.

use montagent_core::finding::{Class, Finding};
use montagent_core::report::Report;
use montagent_core::{checks, parse};

mod common;
use common::{canonical, write_project};

/// A project with no `duration` and no `loop` — the interior-cut cases.
fn project(tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##
    ))
}

/// The same, declaring `duration` and `loop` — the wrap cases (ADR-0062).
fn looping_project(duration: i64, looping: bool, tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"duration":{duration},"loop":{looping},"tracks":[{tracks}]}}"##
    ))
}

fn track(name: &str, elements: &str) -> String {
    format!(r##"{{"name":"{name}","layer":10,"elements":[{elements}]}}"##)
}

/// An image drawing `source`, with `scale` animated through the given `(t, v)` records —
/// the first carrying no `ease` and every later one `linear`, which is ADR-0038's rule.
fn photo(id: &str, start: i64, end: i64, source: &str, scale: &[(i64, f64)]) -> String {
    let records: Vec<String> = scale
        .iter()
        .enumerate()
        .map(|(i, (t, v))| match i {
            0 => format!(r##"{{"t":{t},"v":[{v},{v}]}}"##),
            _ => format!(r##"{{"t":{t},"v":[{v},{v}],"ease":"linear"}}"##),
        })
        .collect();
    format!(
        r##"{{"id":"{id}","type":"image","start":{start},"end":{end},"source":"{source}","x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","scale":[{}]}}"##,
        records.join(",")
    )
}

#[track_caller]
fn report_on(project: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", project);
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::cut::check(&document, &mut report);
    report
}

#[track_caller]
fn pops(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-SOURCE-CUT-POP")
        .collect()
}

fn detail(finding: &Finding) -> &str {
    finding.fields["detail"].as_str().expect("a detail string")
}

#[test]
fn a_same_source_cut_that_resets_an_animated_property_pops() {
    // The fixture's own second cut, reduced: `photo-05-quiz` ends at 64016 having
    // travelled to 1.05419, and `photo-05-loop` begins there back at 1.0. Both draw
    // `images/05.png`; nothing about the picture changes and the scale jumps.
    let tracks = track(
        "photo",
        &format!(
            "{},{}",
            photo(
                "photo-05-quiz",
                53856,
                64016,
                "images/05.png",
                &[(53856, 1.0), (68856, 1.08)]
            ),
            photo(
                "photo-05-loop",
                64016,
                65216,
                "images/05.png",
                &[(64016, 1.0), (79016, 1.08)]
            ),
        ),
    );
    let report = report_on(&project(&tracks));
    let findings = pops(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);

    let finding = findings[0];
    assert_eq!(finding.class, Class::Review);
    assert_eq!(
        finding.fields["element"],
        serde_json::json!("photo-05-quiz")
    );
    assert_eq!(finding.fields["other"], serde_json::json!("photo-05-loop"));
    // ADR-0033 corrected ADR-0006 here: the second same-source cut is at 64016, not 64816.
    assert_eq!(finding.fields["instant"], serde_json::json!(64016));
    assert_eq!(finding.fields["wrap"], serde_json::json!(false));
    assert_eq!(finding.fields["source"], serde_json::json!("images/05.png"));
    // Every number inline (ADR-0006): the out-value, the in-value and the delta.
    assert!(
        detail(finding).contains("1.05419") && detail(finding).contains("1.00000"),
        "{}",
        detail(finding)
    );
    assert!(detail(finding).contains("-0.05419"), "{}", detail(finding));
}

#[test]
fn a_different_source_cut_resetting_the_same_property_is_not_a_pop() {
    // Four of the fixture's six cuts are this, and they are correct: a new image
    // starting its own Ken Burns move. The discrimination is exactly one field wide.
    let tracks = track(
        "photo",
        &format!(
            "{},{}",
            photo(
                "photo-05",
                0,
                17472,
                "images/05.png",
                &[(0, 1.0), (15000, 1.08)]
            ),
            photo(
                "photo-06",
                17472,
                30603,
                "images/06.png",
                &[(17472, 1.0), (32472, 1.08)]
            ),
        ),
    );
    assert!(pops(&report_on(&project(&tracks))).is_empty());
}

#[test]
fn a_gap_between_two_same_source_elements_suppresses_the_check() {
    // ADR-0033: "a gap suppresses the check, since a gap is content, not a seam." The
    // viewer does not see one continuous shot — they see black, and then a new move.
    let tracks = track(
        "photo",
        &format!(
            "{},{}",
            photo(
                "photo-a",
                0,
                10000,
                "images/05.png",
                &[(0, 1.0), (15000, 1.08)]
            ),
            photo(
                "photo-b",
                10450,
                20000,
                "images/05.png",
                &[(10450, 1.0), (25450, 1.08)]
            ),
        ),
    );
    assert!(pops(&report_on(&project(&tracks))).is_empty());
}

#[test]
fn the_same_source_cut_fires_though_the_two_elements_share_no_group() {
    // The whole of ADR-0033's "why `group` is the wrong key": `photo-05-quiz` is in
    // `quiz` and `photo-05-loop` in `loop-tail`, and ADR-0012's group mechanism misses
    // the larger pop entirely. Nothing in this check reads `group`.
    let with_groups = |id: &str, group: &str, start: i64, end: i64, from: f64, to: f64| {
        photo(
            id,
            start,
            end,
            "images/05.png",
            &[(start, from), (end + 15000, to)],
        )
        .replacen(
            &format!(r##""id":"{id}","type":"image""##),
            &format!(r##""id":"{id}","type":"image","group":"{group}""##),
            1,
        )
    };
    let tracks = track(
        "photo",
        &format!(
            "{},{}",
            with_groups("photo-05-quiz", "quiz", 53856, 64016, 1.0, 1.08),
            with_groups("photo-05-loop", "loop-tail", 64016, 65216, 1.0, 1.08),
        ),
    );
    let report = report_on(&project(&tracks));
    assert_eq!(pops(&report).len(), 1, "{:?}", report.findings);
}

#[test]
fn two_elements_of_different_tracks_sharing_a_boundary_are_not_a_cut() {
    // ADR-0033: "two elements on different tracks that happen to share a boundary instant
    // are not a cut — both may be visible and composited simultaneously."
    let tracks = format!(
        "{},{}",
        track(
            "photo",
            &photo(
                "photo-a",
                0,
                10000,
                "images/05.png",
                &[(0, 1.0), (15000, 1.08)]
            )
        ),
        track(
            "overlay",
            &photo(
                "photo-b",
                10000,
                20000,
                "images/05.png",
                &[(10000, 1.0), (25000, 1.08)]
            )
        ),
    );
    assert!(pops(&report_on(&project(&tracks))).is_empty());
}

#[test]
fn a_difference_inside_the_tolerance_is_noise_and_never_fires() {
    // ADR-0033's tolerance table exists to refuse float and round-trip noise, and
    // nothing else: "a real pop is never close to the line." 0.0002 on `scale` is under
    // the 0.0005 column.
    let tracks = track(
        "photo",
        &format!(
            "{},{}",
            photo("photo-a", 0, 10000, "images/05.png", &[(0, 1.0002)]),
            photo("photo-b", 10000, 20000, "images/05.png", &[(10000, 1.0)]),
        ),
    );
    assert!(pops(&report_on(&project(&tracks))).is_empty());
}

#[test]
fn an_unequal_origin_across_a_same_source_cut_is_its_own_trigger() {
    // ADR-0033: "`origin` is not itself compared, but an unequal `origin` across the cut
    // is its own trigger for the same finding, unconditionally" — two elements
    // presenting one continuous shot whose declared reference frames disagree.
    let a = photo("photo-a", 0, 10000, "images/05.png", &[(0, 1.0)]);
    let b = photo("photo-b", 10000, 20000, "images/05.png", &[(10000, 1.0)])
        .replace(r##""origin":"top-left""##, r##""origin":"center""##);
    let report = report_on(&project(&track("photo", &format!("{a},{b}"))));
    let findings = pops(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert!(
        detail(findings[0]).contains("`origin`") && detail(findings[0]).contains("disagree"),
        "{}",
        detail(findings[0])
    );
}

#[test]
fn a_relative_source_and_its_dot_slash_spelling_name_one_file() {
    // ADR-0033: "source identity is the canonicalized path, not a raw string match."
    let tracks = track(
        "photo",
        &format!(
            "{},{}",
            photo(
                "photo-a",
                0,
                10000,
                "images/05.png",
                &[(0, 1.0), (15000, 1.08)]
            ),
            photo(
                "photo-b",
                10000,
                20000,
                "./images/05.png",
                &[(10000, 1.0), (25000, 1.08)]
            ),
        ),
    );
    let report = report_on(&project(&tracks));
    assert_eq!(pops(&report).len(), 1, "{:?}", report.findings);
}

/// The fixture's own wrap, reduced to one track: a still that ends its travel at ~1.0064
/// and a still that begins at 1.0, both `images/05.png`, spanning `0..65216` with no gap.
fn wrapping_track() -> String {
    track(
        "photo",
        &format!(
            "{},{}",
            photo(
                "photo-05-intro",
                0,
                64016,
                "images/05.png",
                &[(0, 1.0), (79016, 1.08)]
            ),
            photo(
                "photo-05-loop",
                64016,
                65216,
                "images/05.png",
                &[(64016, 1.0), (79016, 1.08)]
            ),
        ),
    )
}

#[test]
fn the_wrap_is_an_ordinary_adjacent_pair_when_loop_is_true() {
    // ADR-0062: "`last` and `first` are treated as an ordinary adjacent pair under
    // `R-SOURCE-CUT-POP`" — same code, and the location is what says it is the wrap.
    let report = report_on(&looping_project(65216, true, &wrapping_track()));
    // The interior cut at 64016 pops too; the wrap is the one this test is about.
    let wrap: Vec<_> = pops(&report)
        .into_iter()
        .filter(|f| f.fields["wrap"] == serde_json::json!(true))
        .collect();
    assert_eq!(wrap.len(), 1, "{:?}", report.findings);

    let finding = wrap[0];
    assert_eq!(
        finding.fields["element"],
        serde_json::json!("photo-05-loop")
    );
    assert_eq!(finding.fields["other"], serde_json::json!("photo-05-intro"));
    // ADR-0062: "the location cites both boundary instants."
    let seam = finding.fields["seam"].as_str().unwrap();
    assert!(seam.contains("65216") && seam.contains("0"), "{seam}");
    // ADR-0062: "no new finding code, no new tolerance table, no new severity."
    assert_eq!(finding.code, "R-SOURCE-CUT-POP");
    assert_eq!(finding.class, Class::Review);
}

#[test]
fn the_wrap_fires_nothing_when_the_project_does_not_declare_loop() {
    // ADR-0033 declined the seam precisely because "that comparison requires treating the
    // project as looping, and nothing in the schema declares that."
    let report = report_on(&looping_project(65216, false, &wrapping_track()));
    assert!(
        pops(&report)
            .into_iter()
            .all(|f| f.fields["wrap"] == serde_json::json!(false)),
        "{:?}",
        report.findings
    );
}

#[test]
fn a_gap_at_either_edge_suppresses_the_wrap_on_that_track() {
    // ADR-0062: "if `last.end < duration` or `first.start > 0`, the wrap-adjacency
    // condition simply fails … no new gap rule is needed, because this is the same
    // condition ADR-0033 already requires, evaluated at the wrap."
    let short = wrapping_track().replace(r##""end":65216"##, r##""end":65000"##);
    let report = report_on(&looping_project(65216, true, &short));
    assert!(
        pops(&report)
            .into_iter()
            .all(|f| f.fields["wrap"] == serde_json::json!(false)),
        "{:?}",
        report.findings
    );
}

/// A video clip of one file, with its own source window and a static `scale`.
fn clip(id: &str, start: i64, end: i64, source_start: i64, source_end: i64, scale: f64) -> String {
    format!(
        r##"{{"id":"{id}","type":"video","start":{start},"end":{end},"source":"clips/a.mov","source_start":{source_start},"source_end":{source_end},"x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","scale":[{scale},{scale}]}}"##
    )
}

#[test]
fn two_contiguous_clips_of_one_video_are_one_shot_and_pop() {
    // ADR-0033's time-based term, satisfied: `B.source_start` *is* `A.source_end`, so the
    // picture does not change across the cut and a reset is a pop like any other.
    let tracks = track(
        "clips",
        &format!(
            "{},{}",
            clip("clip-a", 0, 5000, 0, 5000, 1.08),
            clip("clip-b", 5000, 9000, 5000, 9000, 1.0),
        ),
    );
    let report = report_on(&project(&tracks));
    assert_eq!(pops(&report).len(), 1, "{:?}", report.findings);
}

#[test]
fn two_clips_of_one_video_cut_within_the_media_are_not_one_shot() {
    // ADR-0033: clips "whose `source_start`/`source_end` are not contiguous across the cut
    // are a deliberate cut *within* the media … and a reset there is exactly as correct as
    // a different-source cut."
    let tracks = track(
        "clips",
        &format!(
            "{},{}",
            clip("clip-a", 0, 5000, 0, 5000, 1.08),
            clip("clip-b", 5000, 9000, 40000, 44000, 1.0),
        ),
    );
    assert!(pops(&report_on(&project(&tracks))).is_empty());
}

#[test]
fn the_registry_declares_the_check_live_and_review_class() {
    let spec = montagent_core::registry::spec("R-SOURCE-CUT-POP").expect("registered");
    assert_eq!(spec.default_class(), Class::Review);
    assert_eq!(spec.adr, "ADR-0033");
    // ADR-0033: not `error` — "that would be a verdict about intent, which findings may
    // not state" — so there is no repair class to declare (ADR-0043).
    assert_eq!(spec.repair, None);
    // The tolerance table is about this format's own resolved arithmetic, not a constant
    // borrowed from outside it, so ADR-0061's fenced exception does not apply.
    assert_eq!(
        spec.threshold,
        montagent_core::registry::ThresholdProvenance::Internal
    );
}

#[test]
fn both_the_interior_cut_and_the_wrap_render_as_prose() {
    // The template is only exercised at the renderer, and one template serves both seams
    // — so a field only the wrap carries, or only an interior cut does, is a defect
    // nothing else catches.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &looping_project(65216, true, &wrapping_track()),
    );
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::cut::check(&document, &mut report);

    let rendered =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("every template's fields are carried");
    assert!(rendered.contains("at 64016 ms"), "{rendered}");
    assert!(
        rendered.contains("at the loop seam (duration=65216 \u{2192} 0)"),
        "{rendered}"
    );
}

#[test]
fn an_illegal_overlap_in_the_middle_of_a_track_never_hides_a_real_cut() {
    // ADR-0033's trigger is `A.end == B.start`, asked of every pair — not of neighbours in
    // time order. `interloper` overlaps `photo-a` (which is `E-TRACK-OVERLAP`'s finding,
    // not this check's) and sorts between the two elements that really do cut at 10000, so
    // a sliding window over the sorted list would lose that seam entirely.
    let tracks = track(
        "photo",
        &format!(
            "{},{},{}",
            photo(
                "photo-a",
                0,
                10000,
                "images/05.png",
                &[(0, 1.0), (15000, 1.08)]
            ),
            photo("interloper", 5000, 6000, "images/09.png", &[(5000, 1.0)]),
            photo(
                "photo-b",
                10000,
                20000,
                "images/05.png",
                &[(10000, 1.0), (25000, 1.08)]
            ),
        ),
    );
    let report = report_on(&project(&tracks));
    let findings = pops(&report);
    assert_eq!(findings.len(), 1, "{:?}", report.findings);
    assert_eq!(findings[0].fields["instant"], serde_json::json!(10000));
    assert_eq!(findings[0].fields["element"], serde_json::json!("photo-a"));
}
