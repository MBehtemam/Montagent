//! The four caption checks (#199).
//!
//! Every project here is built from `text` elements and `audio` elements carrying no
//! `source`: all four checks are pure document reads (ADR-0054 — *"No file is read; this
//! is a pure document-level check"*), so none of these tests needs the `ffprobe` the disk
//! half of `validate` wants.

use montaget_core::report::Report;
use montaget_core::{parse, validate};

mod common;
use common::{canonical, write_project};

fn project(tracks: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"tracks":[{tracks}]}}"##
    ))
}

fn track(name: &str, layer: i64, elements: &str) -> String {
    format!(r##"{{"name":"{name}","layer":{layer},"elements":[{elements}]}}"##)
}

/// A text element carrying one run. `\n` must reach the document as an escape, so the
/// caller writes it as `\\n` in the literal.
fn caption(id: &str, start: i64, end: i64, text: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"text","start":{start},"end":{end},"x":0,"y":0,"width":1000,"height":200,"font":"brand","size":55,"runs":[{{"text":"{text}"}}]}}"##
    )
}

/// Narration under a caption: an audio element with no `source`, which keeps the disk
/// half out of a question that is entirely about declared times.
fn narration(id: &str, start: i64, end: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"audio","start":{start},"end":{end},"source_start":0,"source_end":{}}}"##,
        end - start
    )
}

/// A narration bed covering `0..end`, for the tests that are not about audio backing.
///
/// `R-CAPTION-NO-AUDIO` looks at every text element in the project, so one written to
/// exercise the pace floor would otherwise report a second finding under every element in
/// it.
fn bed(end: i64) -> String {
    track("narration", 0, &narration("bed", 0, end))
}

/// The four caption checks alone, over a project written to a scratch directory.
///
/// Asked directly rather than through `validate` for `tests/time.rs`'s reason and one of
/// its own. The audio elements here carry no `source`, which the schema requires — a real
/// mp3 beside every project would put the disk half's findings in front of a question that
/// is entirely about declared times — and the elements are deliberately placed to exercise
/// the checks rather than to satisfy ADR-0004's overlap rule.
///
/// It is also the fourth acceptance criterion in executable form: **there is no probe
/// session to pass**. `check` takes a document and a report, so "no file is read by any of
/// the four" is a property of the signature rather than a claim a test has to make.
#[track_caller]
fn report_on(tracks: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montaget.json", &project(tracks));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::caption::check(&document, &mut report);
    report
}

/// Every caption finding's code, in report order.
#[track_caller]
fn caption_codes(report: &Report) -> Vec<&str> {
    report
        .findings
        .iter()
        .map(|f| f.code.as_str())
        .filter(|code| code.starts_with("R-CAPTION-"))
        .collect()
}

fn render(report: &Report) -> String {
    montaget_core::text::render(&report.to_json(), montaget_core::text::Options::verbose())
        .expect("the report renders")
}

/// How many occurrences a repeat finding listed.
trait Occurrences {
    fn count_of_occurrences(&self) -> usize;
}

impl Occurrences for montaget_core::finding::Finding {
    fn count_of_occurrences(&self) -> usize {
        self.fields["detail"]
            .as_str()
            .expect("the listing")
            .matches('`')
            .count()
            / 2
    }
}

/// The one finding carrying `code`.
#[track_caller]
fn only<'a>(report: &'a Report, code: &str) -> &'a montaget_core::finding::Finding {
    let mut found = report.findings.iter().filter(|f| f.code == code);
    let one = found
        .next()
        .unwrap_or_else(|| panic!("no {code}: {:?}", report.findings));
    assert!(
        found.next().is_none(),
        "more than one {code}: {:?}",
        report.findings
    );
    one
}

// ---------------------------------------------------------------------------
// `R-CAPTION-PACE` (ADR-0034, fenced by ADR-0061).
// ---------------------------------------------------------------------------

#[test]
fn a_caption_above_twenty_characters_per_second_is_a_review_finding() {
    // ADR-0034's own worked case: `hook-loop`, 30 grapheme clusters (the `\n` excluded)
    // in 1200 ms, is 25 cps.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &caption("hook-loop", 0, 1200, r"What is this called\nin English?")
        ),
        bed(1200)
    ));

    assert_eq!(caption_codes(&report), ["R-CAPTION-PACE"]);
    let pace = only(&report, "R-CAPTION-PACE");
    assert_eq!(pace.class, montaget_core::finding::Class::Review);
    assert_eq!(pace.fields["measured_cps"], 25.0);
    assert_eq!(pace.fields["characters"], 30);
    assert_eq!(pace.fields["duration"], 1200);
    assert_eq!(pace.fields["threshold_cps"], 20);
}

#[test]
fn a_caption_at_exactly_twenty_characters_per_second_does_not_fire() {
    // ADR-0034's predicate is `cps > 20`, so the threshold itself is legal. The pair
    // matters more than either half: 20 and 20.1 are the two sides of the only number in
    // this check that is not in the document.
    let at = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &caption("at", 0, 1000, "12345678901234567890")
        ),
        bed(1000)
    ));
    assert_eq!(caption_codes(&at), Vec::<&str>::new());

    let over = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &caption("over", 0, 1000, "123456789012345678901")
        ),
        bed(1000)
    ));
    assert_eq!(caption_codes(&over), ["R-CAPTION-PACE"]);
    assert_eq!(only(&over, "R-CAPTION-PACE").fields["measured_cps"], 21.0);
}

#[test]
fn the_pace_metric_counts_grapheme_clusters_and_not_chars() {
    // ADR-0034: "not UTF-16 code units — combining marks and emoji must not double-count".
    // A family emoji is one grapheme cluster, seven `char`s and eleven UTF-16 code units,
    // so three of them in a second is 3 cps counted right and 21 or 33 counted either
    // wrong way — over the line on both.
    let three_families = "👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦";
    let report = report_on(&format!(
        "{},{}",
        track("caption", 10, &caption("emoji", 0, 1000, three_families)),
        bed(1000)
    ));

    assert_eq!(
        caption_codes(&report),
        Vec::<&str>::new(),
        "{:?}",
        report.findings
    );

    // The count it did make, so this test fails loudly rather than silently passing on a
    // caption nothing could flag.
    let fast = report_on(&format!(
        "{},{}",
        track("caption", 10, &caption("emoji", 0, 90, three_families)),
        bed(90)
    ));
    assert_eq!(only(&fast, "R-CAPTION-PACE").fields["characters"], 3);
}

#[test]
fn a_line_break_is_not_a_character_and_a_space_is() {
    // ADR-0034's metric: "spaces included, `\n` excluded". The break is the agent's own
    // (ADR-0008) and nobody reads it; the spaces are content the fixture's
    // `cobweb  -  cobweb` carries deliberately.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{}",
                caption("broken", 0, 1000, r"aaaaaaaaaa\naaaaaaaaaa"),
                caption("spaced", 2000, 3000, "aaaaaaaaaa aaaaaaaaaa")
            )
        ),
        bed(3000)
    ));

    // 20 clusters in 1000 ms is exactly the threshold and does not fire; the space makes
    // 21 and does.
    assert_eq!(caption_codes(&report), ["R-CAPTION-PACE"]);
    let fired = only(&report, "R-CAPTION-PACE");
    assert_eq!(fired.location.element.as_deref(), Some("spaced"));
    assert_eq!(fired.fields["characters"], 21);
}

#[test]
fn the_pace_finding_cites_its_threshold_inline_and_states_the_raw_measurement() {
    // ADR-0061's three conditions, as one assertion each: `review`, the measurement as
    // the substance, and the source cited **in the finding** rather than only in the ADR.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &caption("fast", 0, 1200, r"What is this called\nin English?")
        ),
        bed(1200)
    ));
    let pace = only(&report, "R-CAPTION-PACE");

    assert_eq!(pace.class, montaget_core::finding::Class::Review);
    let citation = pace.citation.as_ref().expect("a citation (ADR-0061)");
    assert_eq!(citation.threshold, serde_json::json!(20));
    assert_eq!(citation.source, "Netflix and BBC timed-text guidance");
    assert_eq!(citation.adr, "ADR-0034");

    let rendered = render(&report);
    for number in ["25", "30", "1200", "20"] {
        assert!(
            rendered.contains(number),
            "{number} is missing:\n{rendered}"
        );
    }
    assert!(
        rendered.contains("Netflix and BBC timed-text guidance"),
        "cited inline (ADR-0061):\n{rendered}"
    );
}

// ---------------------------------------------------------------------------
// `R-CAPTION-MIN-DURATION` (ADR-0054, fenced by ADR-0061).
// ---------------------------------------------------------------------------

#[test]
fn a_caption_below_the_display_floor_is_a_review_finding() {
    // ADR-0054's floor is 834 ms — 5/6 second rounded **up**, so 833 is flagged rather
    // than passed by a rounding artifact — and is script-agnostic and independent of
    // `fps`. The pair is the point: 833 fires and 834 does not.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{}",
                caption("flash", 0, 833, "Go!"),
                caption("held", 1000, 1834, "Go!")
            )
        ),
        bed(1834)
    ));

    let short = only(&report, "R-CAPTION-MIN-DURATION");
    assert_eq!(short.class, montaget_core::finding::Class::Review);
    assert_eq!(short.location.element.as_deref(), Some("flash"));
    assert_eq!(short.fields["duration"], 833);
    assert_eq!(short.fields["floor"], 834);
}

#[test]
fn the_floor_is_a_cited_external_number_like_the_pace_threshold() {
    // ADR-0054 puts the floor "in the same register as `R-CAPTION-PACE`'s 20 cps: an
    // externally documented constant about human reading capacity, not a property of the
    // render" — which is exactly the shape ADR-0061 fences, and its citation rule is
    // "binding policy for future checks of this shape, not best-effort".
    let report = report_on(&format!(
        "{},{}",
        track("caption", 10, &caption("flash", 0, 200, "Go!")),
        bed(200)
    ));
    let finding = only(&report, "R-CAPTION-MIN-DURATION");

    let citation = finding.citation.as_ref().expect("a citation (ADR-0061)");
    assert_eq!(citation.threshold, serde_json::json!(834));
    assert!(
        citation.source.contains("Netflix"),
        "the source in words: {}",
        citation.source
    );
    assert_eq!(citation.adr, "ADR-0054");

    let rendered = render(&report);
    assert!(
        rendered.contains("200"),
        "the measured duration:\n{rendered}"
    );
    assert!(rendered.contains("834"), "the floor:\n{rendered}");
}

#[test]
fn the_floor_does_not_move_with_the_project_s_frame_rate() {
    // "5-6 frames" was a downstream, fps-specific approximation of 5/6 second read back
    // as if it were the primitive (ADR-0054). At 25 fps five frames is 200 ms and six is
    // 240; both of those captions are flagged, and so is one at 800 ms, which no frame
    // count reaches.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{},{}",
                caption("five-frames", 0, 200, "five"),
                caption("six-frames", 1000, 1240, "six"),
                caption("just-under", 2000, 2800, "under")
            )
        ),
        bed(2800)
    ));

    assert_eq!(
        caption_codes(&report),
        ["R-CAPTION-MIN-DURATION"; 3],
        "{:?}",
        report.findings
    );
}

// ---------------------------------------------------------------------------
// `R-CAPTION-NO-AUDIO` (ADR-0054).
// ---------------------------------------------------------------------------

/// A video element, which ADR-0055 calls "one element with intrinsic audio".
fn video(id: &str, start: i64, end: i64) -> String {
    format!(
        r##"{{"id":"{id}","type":"video","start":{start},"end":{end},"source_start":0,"source_end":{},"x":0,"y":0,"width":1080,"height":1920,"fit":"cover"}}"##,
        end - start
    )
}

#[test]
fn a_caption_with_no_audio_under_it_is_a_review_finding() {
    // ADR-0054's own case: the fixture's `hook-loop` runs 64016–65216 and the last
    // narration element ends at 63300. Nothing declared overlaps it, which is answerable
    // from declared times alone.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{}",
                caption("covered", 0, 2000, "spoken"),
                caption("bare", 3000, 5000, "unspoken")
            )
        ),
        track("narration", 0, &narration("vo", 0, 2000))
    ));

    let bare = only(&report, "R-CAPTION-NO-AUDIO");
    assert_eq!(bare.class, montaget_core::finding::Class::Review);
    assert_eq!(bare.location.element.as_deref(), Some("bare"));
    assert_eq!(bare.fields["start"], 3000);
    assert_eq!(bare.fields["end"], 5000);
}

#[test]
fn any_overlap_at_all_is_enough_and_the_boundary_instant_belongs_to_one_element() {
    // ADR-0054 is presence, not coverage: "a text element sitting entirely under a
    // continuous music bed, with no spoken line anywhere near it, passes this check."
    // A single overlapping millisecond is presence. The half-open convention (ADR-0005)
    // then decides the two elements that merely touch: an audio element ending at 3000
    // is not under a caption starting at 3000.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{}",
                caption("touching", 3000, 5000, "a"),
                caption("grazed", 6000, 8000, "a")
            )
        ),
        track(
            "narration",
            0,
            &format!(
                "{},{}",
                narration("before", 0, 3000),
                narration("graze", 5999, 6001)
            )
        )
    ));

    assert_eq!(caption_codes(&report), ["R-CAPTION-NO-AUDIO"]);
    assert_eq!(
        only(&report, "R-CAPTION-NO-AUDIO")
            .location
            .element
            .as_deref(),
        Some("touching")
    );
}

#[test]
fn a_video_element_is_audio_backing_and_a_shape_is_not() {
    // ADR-0055: "a `video` element is one element with intrinsic audio, not a visual
    // paired with a separate audio element" — so it is a time-based audio source and
    // silences this check, which is the under-flagging direction ADR-0054 chose
    // deliberately. A `rect` under the same caption is not.
    let under_video = report_on(&format!(
        "{},{}",
        track("caption", 10, &caption("over-footage", 0, 2000, "a")),
        track("footage", 0, &video("clip", 0, 2000))
    ));
    assert_eq!(caption_codes(&under_video), Vec::<&str>::new());

    let under_rect = report_on(&format!(
        "{},{}",
        track("caption", 10, &caption("over-a-shape", 0, 2000, "a")),
        track(
            "backdrop",
            0,
            r##"{"id":"card","type":"rect","start":0,"end":2000,"x":0,"y":0,"width":100,"height":100}"##
        )
    ));
    assert_eq!(caption_codes(&under_rect), ["R-CAPTION-NO-AUDIO"]);
}

#[test]
fn the_audio_query_is_project_wide_and_never_reads_a_track_name() {
    // ADR-0054: "a project-wide interval query, no track-name involvement". An author may
    // name the track `subs` or `music` or nothing recognisable at all, and a track name is
    // a free-text label with no semantics elsewhere in this domain model.
    let report = report_on(&format!(
        "{},{}",
        track("lower-third", 10, &caption("subtitle", 0, 2000, "a")),
        track("bed", 0, &narration("music", 0, 2000))
    ));

    assert_eq!(caption_codes(&report), Vec::<&str>::new());
}

// ---------------------------------------------------------------------------
// `R-CAPTION-REPEAT-DURATION` (ADR-0034).
// ---------------------------------------------------------------------------

/// Two showings of one line, the second `second` ms long, over a narration bed.
fn repeat(first: i64, second: i64) -> String {
    format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{}",
                caption("original", 0, first, "What is this called"),
                caption("callback", 10_000, 10_000 + second, "What is this called")
            )
        ),
        bed(30_000)
    )
}

#[test]
fn a_repeated_line_whose_durations_disagree_is_one_review_finding() {
    // ADR-0034's fixture case: `hook-05` 2298 ms and `hook-loop` 1200 ms, one line.
    let report = report_on(&repeat(2298, 1200));

    let finding = only(&report, "R-CAPTION-REPEAT-DURATION");
    assert_eq!(finding.class, montaget_core::finding::Class::Review);
    assert_eq!(finding.fields["text"], "What is this called");
    assert_eq!(finding.fields["shortest"], 1200);
    assert_eq!(finding.fields["longest"], 2298);
    assert_eq!(finding.fields["spread"], 1098);
}

#[test]
fn the_repeat_finding_is_symmetric_a_repeat_that_grew_reads_the_same_as_one_that_shrank() {
    // ADR-0034 rejects the ticket's own "shorter on repeat" framing: it "bakes in a claim
    // about what a repeat is *for*", and the opposite convention — a recognized line
    // clipped short because the viewer already read it once — is equally ordinary. So the
    // finding states that the durations disagree and claims nothing about which is right,
    // and the two directions produce the same numbers.
    let shrank = only(&report_on(&repeat(2298, 1200)), "R-CAPTION-REPEAT-DURATION").clone();
    let grew = only(&report_on(&repeat(1200, 2298)), "R-CAPTION-REPEAT-DURATION").clone();

    assert_eq!(shrank.fields["shortest"], grew.fields["shortest"]);
    assert_eq!(shrank.fields["longest"], grew.fields["longest"]);
    assert_eq!(shrank.fields["spread"], grew.fields["spread"]);

    // Neither wording nor field names may name an expected direction: "first", "original",
    // "shorter on repeat" and "should" are all claims about intent (ADR-0006).
    for direction in [
        &report_on(&repeat(2298, 1200)),
        &report_on(&repeat(1200, 2298)),
    ] {
        let rendered = render(direction);
        let lowered = rendered.to_lowercase();
        for claim in ["shorter", "longer", "should", "expected"] {
            assert!(
                !lowered.contains(claim),
                "{claim} is a claim about intent:\n{rendered}"
            );
        }
    }
}

#[test]
fn durations_within_one_frame_of_each_other_are_not_a_disagreement() {
    // ADR-0034: the tolerance "is required, not optional" — two elements independently
    // snapped to frame boundaries can differ by a few milliseconds with no author decision
    // behind it. One frame at the project's 25 fps is 40 ms, and the predicate is *more
    // than* one frame, so 40 passes and 41 does not.
    assert_eq!(
        caption_codes(&report_on(&repeat(2000, 2040))),
        Vec::<&str>::new()
    );
    assert_eq!(
        caption_codes(&report_on(&repeat(2000, 2041))),
        ["R-CAPTION-REPEAT-DURATION"]
    );
}

#[test]
fn a_group_of_five_is_one_finding_listing_five_and_not_ten_pairwise_comparisons() {
    // ADR-0034: "one finding per distinct-text group, not pairwise" — the same "scope the
    // output, never the analysis" discipline ADR-0006 applies elsewhere. Each occurrence
    // is listed with its id, start and duration.
    let five: Vec<String> = (0..5)
        .map(|i| {
            caption(
                &format!("say-{i}"),
                i * 10_000,
                i * 10_000 + 1000 + i * 100,
                "again",
            )
        })
        .collect();
    let report = report_on(&format!(
        "{},{}",
        track("caption", 10, &five.join(",")),
        bed(60_000)
    ));

    let finding = only(&report, "R-CAPTION-REPEAT-DURATION");
    assert_eq!(finding.count_of_occurrences(), 5);
    let detail = finding.fields["detail"].as_str().expect("the listing");
    assert_eq!(finding.fields["count"], 5);

    // Every occurrence's id, start and duration, and all of it in the rendered report
    // rather than only in the JSON — a reader who cannot see `start` cannot go and look at
    // the moment the finding is about.
    for (id, start, duration) in [
        ("say-0", 0, 1000),
        ("say-1", 10_000, 1100),
        ("say-2", 20_000, 1200),
        ("say-3", 30_000, 1300),
        ("say-4", 40_000, 1400),
    ] {
        assert!(
            detail.contains(&format!(
                "`{id}` {start}..{} ms ({duration} ms)",
                start + duration
            )),
            "{id} is missing from: {detail}"
        );
    }
    assert!(render(&report).contains("say-4"), "{}", render(&report));
}

#[test]
fn a_repeat_is_recognised_by_its_text_and_never_by_its_track_or_group() {
    // ADR-0034 groups "project-wide — no dependency on `track` or `group`", the same
    // reasoning ADR-0033 applied to `source`: a caption is recognized by its text, not by
    // where it sits.
    let report = report_on(&format!(
        "{},{},{}",
        track("caption", 10, &caption("here", 0, 2000, "one line")),
        track(
            "caption-overflow",
            11,
            &caption("there", 5000, 8000, "one line")
        ),
        bed(30_000)
    ));

    let finding = only(&report, "R-CAPTION-REPEAT-DURATION");
    assert_eq!(finding.count_of_occurrences(), 2);
}

#[test]
fn two_spellings_of_one_string_group_together_after_nfc_normalization() {
    // ADR-0034 groups on text that is byte-identical "after NFC normalization". The two
    // captions below are one line on screen — a composed é and a decomposed one — and
    // would otherwise each be a group of one, which never fires.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{}",
                caption("composed", 0, 2000, "caf\u{e9}"),
                caption("decomposed", 5000, 8000, "cafe\u{301}")
            )
        ),
        bed(30_000)
    ));

    let finding = only(&report, "R-CAPTION-REPEAT-DURATION");
    assert_eq!(finding.fields["text"], "caf\u{e9}");
    assert_eq!(finding.count_of_occurrences(), 2);
}

#[test]
fn elements_carrying_no_text_are_not_repeats_of_each_other() {
    // The empty string is the one value shared by elements that have nothing in common.
    // ADR-0034 recognises a caption "by its text", and two blank elements of unequal length
    // are not one line shown two ways — reporting them would be the check asserting a
    // repeat that nobody wrote.
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{}",
                caption("blank-long", 0, 5000, ""),
                caption("blank-short", 10_000, 11_000, "")
            )
        ),
        bed(30_000)
    ));

    assert_eq!(
        caption_codes(&report),
        Vec::<&str>::new(),
        "{:?}",
        report.findings
    );
}

#[test]
fn two_different_lines_are_two_groups_and_neither_sees_the_other() {
    let report = report_on(&format!(
        "{},{}",
        track(
            "caption",
            10,
            &format!(
                "{},{},{}",
                caption("a-1", 0, 2000, "first line"),
                caption("a-2", 5000, 8000, "first line"),
                caption("b-1", 10000, 12000, "second line")
            )
        ),
        bed(30_000)
    ));

    let finding = only(&report, "R-CAPTION-REPEAT-DURATION");
    assert_eq!(finding.fields["text"], "first line");
    assert_eq!(finding.count_of_occurrences(), 2);
}

// ---------------------------------------------------------------------------
// The committed fixture — the pairs every one of the four is fired and not fired against.
// ---------------------------------------------------------------------------

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
}

/// The four checks over the committed fixture, with **no probe session in existence**.
///
/// This is the ticket's fourth acceptance criterion, and it is a criterion about the
/// signature as much as the run: there is no session to pass, no `ffprobe` to resolve and
/// no media path opened, on a project whose 60 elements reference real files on disk.
fn fixture_report() -> Report {
    let document = parse::read(&fixture()).expect("the fixture parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montaget_core::checks::caption::check(&document, &mut report);
    report
}

#[track_caller]
fn fired(report: &Report, code: &str) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code == code)
        .map(|f| {
            f.location
                .element
                .clone()
                .unwrap_or_else(|| f.fields["text"].as_str().unwrap_or("?").to_string())
        })
        .collect()
}

#[test]
fn the_fixture_s_one_caption_that_outruns_the_pace_floor_is_the_loop_out() {
    // ADR-0034 names `hook-loop` — 1200 ms for a line `hook-05` gives 2298 — and the other
    // 21 text elements of the fixture do not fire, which is the must-not-fire half.
    let report = fixture_report();

    assert_eq!(fired(&report, "R-CAPTION-PACE"), ["hook-loop"]);
    let pace = only(&report, "R-CAPTION-PACE");
    assert_eq!(pace.fields["characters"], 30);
    assert_eq!(pace.fields["measured_cps"], 25.0);

    // **Against ADR-0034's own worked table, and deliberately.** That table counts
    // `quiz-question` at 46 characters for 20.4 cps and calls it a second firing; 46 is the
    // count *including* the `\n`, which the ADR's own metric paragraph excludes two
    // paragraphs earlier ("spaces included, `\n` excluded"). Counted the way the metric
    // states, it is 45 characters in 2260 ms — 19.9 cps, under the line. The normative
    // sentence wins over the illustrative table (#259).
    let quiz = report
        .findings
        .iter()
        .find(|f| f.location.element.as_deref() == Some("quiz-question"));
    assert!(quiz.is_none(), "{quiz:?}");
}

#[test]
fn the_fixture_s_repeated_lines_are_three_groups_and_its_two_bridged_copies_are_not_one() {
    // ADR-0034 reports one group because it was reading the `caption` track; the check is
    // project-wide (ADR-0054), and the fixture's `sentence-text` track repeats its quiz
    // line too. The must-not-fire half is in the same fixture: `word-08-target` and
    // `word-08-bridge` carry one string across two tracks at identical durations, and a
    // check that grouped on anything but text, or that had no tolerance, would report them.
    let report = fixture_report();

    let mut groups = fired(&report, "R-CAPTION-REPEAT-DURATION");
    groups.sort();
    assert_eq!(
        groups,
        [
            "I hang cobwebs over the door.",
            "What is this called\nin English?",
            "cobweb  -  cobweb",
        ]
    );

    let hook = report
        .findings
        .iter()
        .find(|f| {
            f.fields.get("text").and_then(|t| t.as_str())
                == Some("What is this called\nin English?")
        })
        .expect("the hook group");
    assert_eq!(hook.fields["shortest"], 1200);
    assert_eq!(hook.fields["longest"], 2298);
    assert_eq!(hook.fields["count"], 2);
}

#[test]
fn the_fixture_s_captions_with_nothing_spoken_under_them_are_the_countdown_and_the_loop_out() {
    // ADR-0054's own case is `hook-loop`: it runs 64016–65216 and `narration`'s last
    // element ends at 63300. The five countdown cards are the same fact — `vo-quiz` ends at
    // 56112 and `vo-quiz-answer` starts at 61116 — and ADR-0054 accepts exactly this
    // exposure when it scopes the check to every text element rather than to a track name.
    // The must-not-fire half is the other 16, `chip-text` and `handle-text` included.
    let report = fixture_report();

    assert_eq!(
        fired(&report, "R-CAPTION-NO-AUDIO"),
        [
            "count-5",
            "count-4",
            "count-3",
            "count-2",
            "count-1",
            "hook-loop"
        ]
    );
}

#[test]
fn no_caption_in_the_fixture_is_below_the_display_floor() {
    // The whole must-not-fire half for this one: the fixture's shortest text element is a
    // 1000 ms countdown card, 166 ms clear of ADR-0054's floor. A check that had read
    // "5-6 frames" as the primitive would be silent here too — but so would one that had
    // taken the floor from `fps` — which is why `the_floor_does_not_move_with_the_project_s_frame_rate`
    // carries that half rather than this test.
    assert_eq!(
        fired(&fixture_report(), "R-CAPTION-MIN-DURATION"),
        Vec::<String>::new()
    );
}

#[test]
fn validate_runs_all_four_on_a_project_that_references_no_media_at_all() {
    // The wiring, end to end, on the one kind of project that needs no `ffprobe`: a single
    // text element referencing nothing on disk. `validate` "always runs every check on the
    // whole project" (ADR-0006), and three of the four have something to say here — the
    // fourth, `R-CAPTION-REPEAT-DURATION`, needs a second showing to have one.
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montaget.json",
        &project(&track(
            "caption",
            10,
            &caption("flash", 0, 500, "far too much text here"),
        )),
    );

    let report = validate(&path);
    let mut codes = caption_codes(&report);
    codes.sort();

    assert_eq!(
        codes,
        [
            "R-CAPTION-MIN-DURATION",
            "R-CAPTION-NO-AUDIO",
            "R-CAPTION-PACE"
        ]
    );
    assert_eq!(
        report.exit_code(),
        montaget_core::report::ExitCode::Ok,
        "`review` never gates a render (ADR-0006): {:?}",
        report.findings
    );
}
