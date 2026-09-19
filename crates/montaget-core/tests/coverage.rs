//! Cross-track coverage — group-scoped pairing (ADR-0018, #200).
//!
//! Every project here carries elements with nothing but `id`, `type`, `group`, `start`
//! and `end`: no `x`/`y`/`width`/`height`, no `layer`, no `source`. That is deliberate —
//! one of this ticket's own acceptance criteria is that the check consults neither
//! geometry nor layer resolution, and a project that omits both entirely and still
//! produces the right findings is the proof, not merely an assertion about the code.

use montaget_core::finding::Class;
use montaget_core::report::Report;
use montaget_core::validate;

mod common;
use common::write_project;

fn project(tracks: &str) -> String {
    format!(r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##)
}

fn track(elements: &str) -> String {
    format!(r##"{{"elements":[{elements}]}}"##)
}

/// The minimal shape this check reads — no geometry, no `layer`, no `source`.
fn element(id: &str, kind: &str, group: &str, start: i64, end: i64) -> String {
    format!(r##"{{"id":"{id}","type":"{kind}","group":"{group}","start":{start},"end":{end}}}"##)
}

fn ungrouped(id: &str, kind: &str, start: i64, end: i64) -> String {
    format!(r##"{{"id":"{id}","type":"{kind}","start":{start},"end":{end}}}"##)
}

#[track_caller]
fn report_on(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(tracks));
    validate(&path)
}

#[track_caller]
fn visual_gaps(report: &Report) -> Vec<&montaget_core::finding::Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-VISUAL-GAP")
        .collect()
}

#[test]
fn audio_outrunning_its_groups_visual_is_a_review() {
    // Narration keeps going 2000 ms after the last visual in its group ends — ADR-0018's
    // own founding defect (#23): "narration outran the last visual".
    let tracks = track(&format!(
        "{},{}",
        element("photo", "image", "item", 0, 3000),
        element("vo", "audio", "item", 0, 5000),
    ));
    let report = report_on(&tracks);
    let gaps = visual_gaps(&report);

    assert_eq!(gaps.len(), 1, "{gaps:#?}");
    let gap = gaps[0];
    assert_eq!(gap.class, Class::Review);
    assert_eq!(gap.fields["group"], "item");
    assert_eq!(gap.fields["from"], 3000);
    assert_eq!(gap.fields["to"], 5000);
    assert_eq!(gap.fields["size"], 2000);
    assert_eq!(gap.fields["active"], "audio");
    assert_eq!(gap.fields["missing"], "visual");
    assert_eq!(gap.fields["elements"], "vo");
}

#[test]
fn visual_outrunning_its_groups_audio_is_a_note() {
    // Symmetric, exactly as ADR-0018 requires: the same shape, the opposite direction —
    // a photo held on screen 2000 ms past where its narration stops.
    let tracks = track(&format!(
        "{},{}",
        element("photo", "image", "item", 0, 5000),
        element("vo", "audio", "item", 0, 3000),
    ));
    let report = report_on(&tracks);
    let gaps = visual_gaps(&report);

    assert_eq!(gaps.len(), 1, "{gaps:#?}");
    let gap = gaps[0];
    assert_eq!(gap.class, Class::Note);
    assert_eq!(gap.fields["from"], 3000);
    assert_eq!(gap.fields["to"], 5000);
    assert_eq!(gap.fields["active"], "visual");
    assert_eq!(gap.fields["missing"], "audio");
    assert_eq!(gap.fields["elements"], "photo");
}

#[test]
fn a_group_with_only_visual_members_is_not_evaluated() {
    // The fixture's own `header`: eight always-on elements and no audio member. Pairing
    // asserts nothing about a side that was never declared — not a false "covered".
    let tracks = track(&format!(
        "{},{}",
        element("chip", "rect", "header", 0, 65216),
        element("logo", "image", "header", 0, 65216),
    ));
    let report = report_on(&tracks);
    assert!(visual_gaps(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_group_with_only_audio_members_is_not_evaluated() {
    let tracks = track(&element("vo", "audio", "voiceover", 0, 5000));
    let report = report_on(&tracks);
    assert!(visual_gaps(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_gap_between_ungrouped_elements_is_out_of_scope() {
    // ADR-0018's own stated scope: "gaps falling outside any audio/visual group pairing
    // are out of scope by design." Two ungrouped elements, one audio and one visual,
    // never pair — there is no `group` to pair them by.
    let tracks = track(&format!(
        "{},{}",
        ungrouped("photo", "image", 0, 3000),
        ungrouped("vo", "audio", 0, 5000),
    ));
    let report = report_on(&tracks);
    assert!(visual_gaps(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_group_covered_on_both_sides_emits_nothing() {
    let tracks = track(&format!(
        "{},{}",
        element("photo", "image", "item", 0, 5000),
        element("vo", "audio", "item", 0, 5000),
    ));
    let report = report_on(&tracks);
    assert!(visual_gaps(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_video_s_own_soundtrack_covers_its_group_s_audio_side() {
    // ADR-0055: "a `video` element is one element with intrinsic audio, not a visual
    // paired with a separate audio element" — the same reading `caption::no_audio`
    // already gives it. A caption alone with a `video` its whole length must not read as
    // "visual with nothing on audio": the video's own soundtrack is the audio.
    let tracks = track(&format!(
        "{},{}",
        element("footage", "video", "item", 0, 5000),
        element("caption", "text", "item", 0, 5000),
    ));
    let report = report_on(&tracks);
    assert!(visual_gaps(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_video_still_counts_as_visual_when_narration_outruns_it() {
    // `video` is both sides at once, not a choice — this is the half a fix that only
    // added it to `audio` would still get backwards. A separate voiceover keeps going
    // 2000 ms after the footage ends: audio's union (the voiceover's own range, unioned
    // with the video's identical contribution) outruns visual's union (the video's own
    // span, its only visual member), the same `review` a standalone `audio` element
    // would produce.
    let tracks = track(&format!(
        "{},{}",
        element("footage", "video", "item", 0, 3000),
        element("vo", "audio", "item", 0, 5000),
    ));
    let report = report_on(&tracks);
    let gaps = visual_gaps(&report);

    assert_eq!(gaps.len(), 1, "{gaps:#?}");
    let gap = gaps[0];
    assert_eq!(gap.class, Class::Review);
    assert_eq!(gap.fields["from"], 3000);
    assert_eq!(gap.fields["to"], 5000);
    assert_eq!(gap.fields["active"], "audio");
    assert_eq!(gap.fields["missing"], "visual");
}
