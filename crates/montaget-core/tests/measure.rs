//! `measure` and the text engine (#205), at seam 1.
//!
//! Spec #168 puts `montaget-text` under test *"through `measure` rather than directly,
//! since its output **is** `measure`'s output"*, so every assertion here goes through the
//! verb.
//!
//! **The fixture is not the oracle for an extent, and deliberately is not used as one.**
//! Spec #168 records the reason: the fixture *"is no longer typeset in the font its
//! reference MP4 was rendered in"*, 22 of its elements carry sizes measured against the old
//! metrics, and [#186](https://github.com/MBehtemam/Montaget/issues/186) — still open — is
//! what settles whether they hold. A test asserting that `measure` agrees with a declared
//! `width` would be asserting #186's outcome in advance. The font *file* is used, because
//! it is the real vendored font and the one this build actually shapes in; what the
//! fixture declares about it is not.
//!
//! Where an expected number is not a bare consequence of an ADR's own arithmetic, it comes
//! from `skrifa` reading the font file directly — a different path to the same fact than
//! the `parley` shaping `measure` runs on, so the assertion can disagree with the code
//! rather than restate it.

use std::path::{Path, PathBuf};

use montaget_core::report::ExitCode;
use montaget_core::verbs::measure::{Answer, Ask};
use serde_json::{Value, json};

mod common;
use common::{canonical, write_project};

// ---------------------------------------------------------------------------
// The harness.
// ---------------------------------------------------------------------------

fn font_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf")
        .canonicalize()
        .expect("the vendored font (#143)")
}

/// A project declaring one font chain, and nothing else worth measuring against.
///
/// The chain entry is an absolute path: a relative one resolves against the project file's
/// own directory (ADR-0053), which for a test in a temporary directory would mean copying
/// the font. The resolution rule itself is exercised by the relative case below.
fn project_with(fonts: &str, line: u32) -> PathBuf {
    let dir = common::tempdir(line);
    write_project(
        &dir,
        "p.montaget.json",
        &canonical(&format!(
            r#"{{"frame":{{"width":1080,"height":1920}},"fps":25,"fonts":{fonts},"tracks":[]}}"#
        )),
    )
}

/// The one-font project every test below measures against, unless it is testing the
/// `fonts` table itself.
fn project(line: u32) -> PathBuf {
    project_with(
        &format!(
            r#"{{"brand":[{{"file":{}}}]}}"#,
            serde_json::to_string(&font_file().display().to_string()).expect("a path serialises")
        ),
        line,
    )
}

#[track_caller]
fn measure(project: &Path, element: Value) -> Answer {
    montaget_core::verbs::measure::measure(
        project,
        &Ask {
            element: Some(element),
            at: None,
            elements: None,
            all: false,
        },
    )
}

#[track_caller]
fn measure_elements(project: &Path, elements: Vec<Value>) -> Answer {
    montaget_core::verbs::measure::measure(
        project,
        &Ask {
            element: None,
            at: None,
            elements: Some(elements),
            all: false,
        },
    )
}

#[track_caller]
fn measure_all(project: &Path) -> Answer {
    montaget_core::verbs::measure::measure(
        project,
        &Ask {
            element: None,
            at: None,
            elements: None,
            all: true,
        },
    )
}

/// The `measure` block of the canonical JSON, which is the whole answer.
#[track_caller]
fn view(answer: &Answer) -> Value {
    let json = answer.to_json();
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "measure did not answer: {}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    json["measure"].clone()
}

/// One text element, as the schema shapes one.
fn text(size: i64, runs: Value) -> Value {
    json!({"type": "text", "font": "brand", "size": size, "runs": runs})
}

fn one_run(text_of: &str) -> Value {
    json!([{"text": text_of}])
}

#[track_caller]
fn line_texts(view: &Value) -> Vec<String> {
    view["lines"]
        .as_array()
        .expect("every answer carries its lines")
        .iter()
        .map(|line| line["text"].as_str().unwrap_or_default().to_string())
        .collect()
}

/// One pixel measurement off the answer, as a number rather than a `Value`.
#[track_caller]
fn px(value: &Value) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("{value} is not a number"))
}

/// The font's own ascent and descent at one size, read straight off the file.
///
/// The independent path this file's expected vertical numbers come from. `skrifa` reports
/// descent as a negative offset from the baseline and `measure` reports it as a positive
/// distance, so the sign is normalised here — the one conversion, stated once.
fn font_metrics(size: f32) -> (f64, f64) {
    use skrifa::prelude::*;
    use skrifa::{FontRef, MetadataProvider};

    let bytes = std::fs::read(font_file()).expect("the vendored font");
    let metrics = FontRef::new(&bytes)
        .expect("a parseable font")
        .metrics(Size::new(size), LocationRef::default());
    (f64::from(metrics.ascent), f64::from(-metrics.descent))
}

// ---------------------------------------------------------------------------
// The line partition is Montaget's own UAX #14, not `split('\n')` (ADR-0008).
// ---------------------------------------------------------------------------

#[test]
fn every_mandatory_break_class_ends_a_line_and_lines_are_breaks_plus_one() {
    let project = project(line!());

    // BK, CR, LF and NL — the whole set ADR-0008 names, in one string. Five breaks, so
    // six lines: the invariant the ADR makes part of the renderer contract.
    let answer = measure(
        &project,
        text(
            20,
            one_run("a\nb\u{000B}c\u{000C}d\u{0085}e\u{2028}f\u{2029}g"),
        ),
    );
    let view = view(&answer);

    assert_eq!(view["line_count"], 7);
    assert_eq!(line_texts(&view), ["a", "b", "c", "d", "e", "f", "g"]);
}

#[test]
fn crlf_is_one_break_and_the_cr_never_reaches_the_line() {
    let project = project(line!());

    let answer = measure(&project, text(20, one_run("a\r\nb")));
    let view = view(&answer);

    // Two lines, not three — `\r\n` is one break (ADR-0008). And the line is `"a"`, not
    // `"a\r"`: a naive `split('\n')` leaves the CR inside the line, where it would be
    // shaped and measured into the advance width. ADR-0007's "no tidying pass, ever" means
    // the CR is content the file keeps; it is the *partition* that must not trip over it.
    assert_eq!(view["line_count"], 2);
    assert_eq!(line_texts(&view), ["a", "b"]);
}

#[test]
fn a_bare_cr_is_its_own_break() {
    let project = project(line!());

    let view = view(&measure(&project, text(20, one_run("a\rb"))));

    assert_eq!(line_texts(&view), ["a", "b"]);
}

#[test]
fn a_trailing_break_leaves_an_empty_last_line() {
    let project = project(line!());

    let view = view(&measure(&project, text(20, one_run("a\n"))));

    // One break, so two lines. The second is empty, and its slot is still reserved: an
    // author who wrote a trailing `\n` asked for the space.
    assert_eq!(view["line_count"], 2);
    assert_eq!(line_texts(&view), ["a", ""]);
    assert_eq!(
        view["block_height"], 48,
        "two 20px slots at the default line_height of 1.2"
    );
    // And the empty line advances by nothing — parley lays an empty paragraph out as one
    // synthetic whitespace cluster, whose width is not a width this line has.
    assert_eq!(px(&view["lines"][1]["advance_width"]), 0.0);
}

#[test]
fn a_break_inside_one_run_partitions_it() {
    let project = project(line!());

    // ADR-0007: a run boundary is style only; a line break is a `\n` character inside a
    // run's text. Three of the fixture's four multi-line events break with no style change
    // at all, so this is the normal case rather than the exotic one.
    let view = view(&measure(&project, text(20, one_run("one\ntwo"))));

    assert_eq!(line_texts(&view), ["one", "two"]);
}

#[test]
fn a_break_across_a_run_boundary_partitions_both() {
    let project = project(line!());

    let view = view(&measure(
        &project,
        text(20, json!([{"text": "one\nt"}, {"text": "wo"}])),
    ));

    assert_eq!(line_texts(&view), ["one", "two"]);
}

// ---------------------------------------------------------------------------
// Break opportunities, with the segmenter named (ADR-0008).
// ---------------------------------------------------------------------------

/// ADR-0008's own worked Thai claim, which is the expected value here:
///
/// > The agent placing that break in `ฉันกำลังถือแมงมุมตัวใหญ่ไว้ในมือ` must know that
/// > `แมงมุม|ตัวใหญ่` is a word boundary and `แมงมุ|มตัวใหญ่` is not. Nothing in the string marks it.
#[test]
fn a_thai_word_boundary_is_offered_and_a_mid_word_position_is_not() {
    let project = project(line!());
    const SENTENCE: &str = "ฉันกำลังถือแมงมุมตัวใหญ่ไว้ในมือ";

    let view = view(&measure(&project, text(20, one_run(SENTENCE))));
    let offered: Vec<usize> = view["lines"][0]["break_opportunities"]
        .as_array()
        .expect("a line carries its opportunities")
        .iter()
        .map(|at| at.as_u64().expect("an offset is a number") as usize)
        .collect();

    let boundary = SENTENCE.find("ตัวใหญ่").expect("the word is in the sentence");
    let mid_word = SENTENCE
        .find("มตัวใหญ่")
        .expect("the position is in the sentence");
    assert!(
        offered.contains(&boundary),
        "`แมงมุม|ตัวใหญ่` is a word boundary and must be offered; offered {offered:?}"
    );
    assert!(
        !offered.contains(&mid_word),
        "`แมงมุ|มตัวใหญ่` is not a word boundary and must not be offered; offered {offered:?}"
    );
}

#[test]
fn thai_without_a_dictionary_would_offer_nothing_at_all() {
    let project = project(line!());

    // #27 measured that a stack resolving UAX #14's class `SA` to `AL` sees one unbreakable
    // 67-character word and offers **zero** opportunities — cosmic-text and parley with the
    // flag off, byte-for-byte identical. So a non-empty answer here is the whole
    // discriminator, and a regression to a dictionary-free segmenter shows up as an empty
    // list rather than as a subtly different one.
    let view = view(&measure(
        &project,
        text(
            20,
            one_run("การเดินทางไปประเทศไทยเป็นประสบการณ์ที่ดีมากสำหรับนักท่องเที่ยวทุกคน"),
        ),
    ));

    let offered = view["lines"][0]["break_opportunities"]
        .as_array()
        .expect("a line carries its opportunities");
    assert!(
        offered.len() > 5,
        "a dictionary-backed segmenter offers many; a dictionary-free one offers none. \
         Offered {offered:?}"
    );
}

#[test]
fn opportunities_are_absolute_offsets_and_never_a_break_that_is_already_there() {
    let project = project(line!());

    let view = view(&measure(&project, text(20, one_run("one two\nthree four"))));
    let of = |line: usize| -> Vec<u64> {
        view["lines"][line]["break_opportunities"]
            .as_array()
            .expect("a line carries its opportunities")
            .iter()
            .map(|at| at.as_u64().expect("an offset is a number"))
            .collect()
    };

    // Offsets into the element's **whole** text, not into the line: an agent placing a `\n`
    // is editing one string, and an offset relative to a line would need the line's own
    // start added back — arithmetic the tool is here to have already done.
    assert_eq!(of(0), [4], "after `one `");
    assert_eq!(
        of(1),
        [14],
        "after `three `, counted from the start of the text"
    );

    // Never 0, never the end, and never the position of a mandatory break the author has
    // already written: none of the three is a place a `\n` can go.
    for line in 0..2 {
        assert!(!of(line).contains(&0));
        assert!(!of(line).contains(&8), "8 is the `\\n` the author wrote");
        assert!(!of(line).contains(&18), "18 is the end of the text");
    }
}

#[test]
fn the_segmenter_and_its_data_version_are_named_beside_the_offsets() {
    let project = project(line!());

    let view = view(&measure(&project, text(20, one_run("one two"))));

    // ADR-0008 asks for the segmenter *and the data version* alongside the offsets,
    // because two versions of this data legitimately disagree — a set of offsets that did
    // not say which produced it would be unfalsifiable.
    assert_eq!(view["segmenter"]["name"], "icu_segmenter");
    assert_eq!(view["segmenter"]["version"], "2.3.0");
    assert_eq!(view["segmenter"]["data"], "icu_segmenter_data 2.3.0");
    assert_eq!(
        view["segmenter"]["model"], "dictionary",
        "ADR-0008: dictionary-backed for class SA"
    );
}

// ---------------------------------------------------------------------------
// The block, the slot and the baseline (ADR-0007, ADR-0029).
// ---------------------------------------------------------------------------

#[test]
fn adr_0007s_worked_example_reproduces_exactly() {
    let project = project(line!());

    // ADR-0007, verbatim: "the worked example there (size 55, `line_height` 1.1, centred on
    // 1537 → 1506.75–1567.25) reproduces."
    let mut element = text(55, one_run("I hang cobwebs over the door."));
    element["line_height"] = json!(1.1);
    element["y"] = json!(1537);
    element["origin"] = json!("center");

    let view = view(&measure(&project, element));

    assert_eq!(px(&view["block_top"]), 1506.75);
    assert_eq!(px(&view["block_bottom"]), 1567.25);
    assert_eq!(px(&view["lines"][0]["slot_height"]), 60.5);
}

#[test]
fn the_baseline_is_half_leading_around_the_slots_own_centre() {
    let project = project(line!());

    let mut element = text(55, one_run("I hang cobwebs over the door."));
    element["line_height"] = json!(1.1);
    element["y"] = json!(1537);
    element["origin"] = json!("center");

    let view = view(&measure(&project, element));

    // ADR-0029: `baseline_y = slot_centre_y + (ascent − descent) / 2`. The slot's centre is
    // ADR-0007's own worked example (1506.75..1567.25, so 1537); the ascent and descent
    // come from the font file, read by `skrifa` rather than by the shaping under test.
    let (ascent, descent) = font_metrics(55.0);
    assert_eq!(
        px(&view["lines"][0]["baseline_y"]),
        1537.0 + (ascent - descent) / 2.0
    );

    // And the metrics themselves are the font's, not something derived from `size`.
    assert_eq!(px(&view["ascent"]), ascent);
    assert_eq!(px(&view["descent"]), descent);
}

#[test]
fn ascent_and_descent_exceed_the_slot_which_is_why_the_convention_had_to_be_decided() {
    let project = project(line!());

    let mut element = text(55, one_run("x"));
    element["line_height"] = json!(1.1);
    let view = view(&measure(&project, element));

    // #59's measurement, re-derived rather than quoted: against the fixture's actual font
    // at size 55, ascent + descent **exceeds** the 60.5 px slot. That negative leading is
    // what makes the three candidate conventions disagree by ~5 px on every caption, and
    // why ADR-0029 exists at all. If this ever stops holding, the ADR's motivating case has
    // changed and someone should know.
    assert!(
        px(&view["ascent"]) + px(&view["descent"]) > px(&view["lines"][0]["slot_height"]),
        "the leading is negative in this font at this size"
    );
}

#[test]
fn each_line_sits_in_its_own_slot_below_the_one_before() {
    let project = project(line!());

    let mut element = text(50, one_run("one\ntwo\nthree"));
    element["line_height"] = json!(1.2);
    element["y"] = json!(0);
    element["origin"] = json!("top-left");

    let view = view(&measure(&project, element));
    let slot_top = |i: usize| px(&view["lines"][i]["slot_top"]);

    // `top-*` puts the block's top at `y`, and the slots stack from there — 50 x 1.2 = 60
    // apiece (ADR-0007).
    assert_eq!(px(&view["block_top"]), 0.0);
    assert_eq!([slot_top(0), slot_top(1), slot_top(2)], [0.0, 60.0, 120.0]);
    assert_eq!(px(&view["block_bottom"]), 180.0);

    // The baselines are one slot apart, exactly.
    let baseline = |i: usize| px(&view["lines"][i]["baseline_y"]);
    assert_eq!(baseline(1) - baseline(0), 60.0);
    assert_eq!(baseline(2) - baseline(1), 60.0);
}

#[test]
fn the_nine_origin_keywords_place_the_block_three_ways() {
    let project = project(line!());

    let placed = |origin: &str| {
        let mut element = text(50, one_run("x"));
        element["line_height"] = json!(1.2);
        element["y"] = json!(1000);
        element["origin"] = json!(origin);
        let view = view(&measure(&project, element));
        (px(&view["block_top"]), px(&view["block_bottom"]))
    };

    // A 60 px block, placed at y = 1000 by each keyword's vertical component (ADR-0013's
    // nine, CONTEXT.md's "the point of an element's own box that its `x`,`y` places").
    for origin in ["top-left", "top-center", "top-right"] {
        assert_eq!(placed(origin), (1000.0, 1060.0), "{origin}");
    }
    for origin in ["center-left", "center", "center-right"] {
        assert_eq!(placed(origin), (970.0, 1030.0), "{origin}");
    }
    for origin in ["bottom-left", "bottom-center", "bottom-right"] {
        assert_eq!(placed(origin), (940.0, 1000.0), "{origin}");
    }
}

#[test]
fn origin_defaults_to_center_when_the_element_does_not_write_one() {
    let project = project(line!());

    let mut element = text(50, one_run("x"));
    element["line_height"] = json!(1.2);
    element["y"] = json!(1000);

    let view = view(&measure(&project, element));

    // CONTEXT.md: "Defaults to `center`, so a bare element is conspicuous rather than
    // plausible." The answer echoes the origin it used, so the default is visible rather
    // than silently applied.
    assert_eq!(view["asked"]["origin"], "center");
    assert_eq!(px(&view["block_top"]), 970.0);
}

#[test]
fn a_line_reserves_the_slot_of_its_largest_run_and_not_the_elements_base_size() {
    let project = project(line!());

    // ADR-0007: "A line's height is the largest `size` among the runs on that line x
    // `line_height`." The base size is 20 and one run overrides to 60, so the slot is 60's.
    let mut element = text(20, json!([{"text": "small "}, {"text": "BIG", "size": 60}]));
    element["line_height"] = json!(1.0);

    let view = view(&measure(&project, element));

    assert_eq!(view["lines"][0]["size"], 60);
    assert_eq!(px(&view["lines"][0]["slot_height"]), 60.0);
    assert_eq!(view["block_height"], 60);
}

#[test]
fn a_base_size_every_run_overrides_sets_no_slot_of_its_own() {
    let project = project(line!());

    // The other direction of ADR-0007's rule, and the one that over-reports if the base is
    // folded in unconditionally: base 88, and every run on the line at 40. The slot is
    // 40's. ADR-0007's model is base-plus-deltas and "a delta that is absent is a delta
    // that was not made" — so a base every run has replaced is not still in force, and a
    // block reported at 88's height would put an author's `height` 48 px out.
    let mut element = text(
        88,
        json!([{"text": "small ", "size": 40}, {"text": "also small", "size": 40}]),
    );
    element["line_height"] = json!(1.0);

    let overridden = view(&measure(&project, element));

    assert_eq!(overridden["lines"][0]["size"], 40);
    assert_eq!(px(&overridden["lines"][0]["slot_height"]), 40.0);
    assert_eq!(overridden["block_height"], 40);

    // One run leaving `size` unstated brings the base back: that run *is* at 88.
    let mut mixed = text(
        88,
        json!([{"text": "base "}, {"text": "small", "size": 40}]),
    );
    mixed["line_height"] = json!(1.0);
    assert_eq!(view(&measure(&project, mixed))["block_height"], 88);
}

#[test]
fn a_base_stroke_width_every_run_overrides_sets_no_extent_of_its_own() {
    let project = project(line!());

    // ADR-0014 makes stroke a run-addressable paint, so it resolves on the same rule — and
    // gets the same wrong answer if the base is folded in unconditionally. Every run
    // overrides 10 down to 2, so the extent grows by 4, not 20.
    let mut element = text(
        50,
        json!([{"text": "a", "stroke_width": 2}, {"text": "b", "stroke_width": 2}]),
    );
    element["line_height"] = json!(1.0);
    element["stroke_width"] = json!(10);

    let view = view(&measure(&project, element));

    assert_eq!(view["extent"]["stroke_width"], 2);
    assert_eq!(
        px(&view["extent"]["width"]),
        px(&view["advance_width"]) + 4.0
    );
}

#[test]
fn two_lines_of_different_sizes_each_reserve_their_own_slot() {
    let project = project(line!());

    let mut element = text(
        20,
        json!([{"text": "small\n"}, {"text": "BIG", "size": 60}]),
    );
    element["line_height"] = json!(1.0);
    element["y"] = json!(0);
    element["origin"] = json!("top-left");

    let view = view(&measure(&project, element));

    // 20 then 60, summed — which is the case `ceil(size x line_height x line_count)` cannot
    // express, and the reason `measure` sums the slots itself and hands the total to
    // ADR-0028's one `ceil` rather than calling the three-argument form.
    assert_eq!(px(&view["lines"][0]["slot_height"]), 20.0);
    assert_eq!(px(&view["lines"][1]["slot_height"]), 60.0);
    assert_eq!(view["block_height"], 80);
    assert_eq!(px(&view["lines"][1]["slot_top"]), 20.0);
}

#[test]
fn ascent_is_read_across_every_run_on_the_line() {
    let project = project(line!());

    // ADR-0029's max-across-all-runs. Two runs on one line at 20 and 58; the answer's
    // ascent is 58's, not the base size's.
    //
    // **What this test cannot distinguish, stated rather than implied.** ADR-0029's rule
    // differs from "the largest run's metrics" only when a *smaller* run out-ascends the
    // larger one — a mixed-script fallback, an icon glyph, a different family. This
    // repository vendors exactly one font (#143), in which ascent is proportional to size,
    // so the max-ascent run and the max-size run are always the same run and the two rules
    // cannot be told apart here. ADR-0029 records the same gap from the other side:
    // "Untested by the fixture."
    let element = text(20, json!([{"text": "a"}, {"text": "b", "size": 58}]));
    let view = view(&measure(&project, element));

    let (ascent, descent) = font_metrics(58.0);
    assert_eq!(px(&view["lines"][0]["ascent"]), ascent);
    assert_eq!(px(&view["lines"][0]["descent"]), descent);
}

// ---------------------------------------------------------------------------
// The block height is ADR-0028's arithmetic, and there is one of it.
// ---------------------------------------------------------------------------

#[test]
fn the_block_height_is_the_one_exact_tenths_ceil_validate_also_uses() {
    let project = project(line!());

    // The uniform-size case, which is what `crate::exact::text_block_height` expresses and
    // what `R-BOX-SLACK` computes a declared `height` against. The two must agree on every
    // input or an author filling `height` from `measure` would fail `validate` — so the
    // expected value here is that function, called directly.
    for (size, tenths, count) in [(55, 11, 1), (58, 11, 2), (35, 11, 1), (20, 11, 3)] {
        let mut element = text(size, one_run(&"x\n".repeat(count as usize - 1)));
        element["line_height"] = json!(
            format!("{}.{}", tenths / 10, tenths % 10)
                .parse::<f64>()
                .expect("a literal")
        );

        let view = view(&measure(&project, element));
        assert_eq!(
            view["block_height"],
            json!(
                montaget_core::exact::text_block_height(size, tenths, count)
                    .expect("positive inputs")
            ),
            "size {size}, line_height tenths {tenths}, {count} lines"
        );
    }
}

#[test]
fn the_block_height_is_exact_where_ieee_double_diverges() {
    let project = project(line!());

    // ADR-0028 measured `size x line_height x line_count` disagreeing with its exact value
    // on 6.35% of sampled cases at `line_height = 1.1`. This is one of them, chosen so the
    // two answers differ by the whole point: 20 x 1.1 is exactly 22, so `ceil` is 22 — but
    // `20.0 * 1.1` in IEEE double is 22.000000000000004, and `ceil` of that is 23.
    let mut element = text(20, one_run("x"));
    element["line_height"] = json!(1.1);

    let view = view(&measure(&project, element));

    assert_eq!(
        view["block_height"], 22,
        "a `f64` product would reach 23 here — ADR-0028's whole subject"
    );
    // And the exact tenths are what the `ceil` was taken of: 22.0 px, not 22.000000000000004.
    assert_eq!(px(&view["lines"][0]["slot_height"]), 22.0);
}

#[test]
fn line_height_defaults_to_one_point_two_when_the_element_does_not_write_one() {
    let project = project(line!());

    let view = view(&measure(&project, text(50, one_run("x"))));

    // ADR-0007: "`line_height` defaults to 1.2 when omitted." The answer states the tenths
    // it used rather than leaving the reader to assume.
    assert_eq!(view["asked"]["line_height_tenths"], 12);
    assert_eq!(view["block_height"], 60);
}

#[test]
fn a_line_height_outside_the_tenths_domain_is_refused() {
    let project = project(line!());

    let mut element = text(50, one_run("x"));
    element["line_height"] = json!(1.15);

    let answer = measure(&project, element);

    // ADR-0028 restricts `line_height` to multiples of 0.1. Rounding it here would be this
    // verb quietly answering a question the format does not admit.
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert!(
        answer.to_json().to_string().contains("ADR-0028"),
        "the refusal names the rule"
    );
}

// ---------------------------------------------------------------------------
// The extent is the stroked one (ADR-0014).
// ---------------------------------------------------------------------------

#[test]
fn the_extent_is_the_stroked_one_and_the_advance_beside_it_is_not() {
    let project = project(line!());

    let bare = view(&measure(&project, {
        let mut element = text(50, one_run("hello"));
        element["line_height"] = json!(1.0);
        element
    }));
    let stroked = view(&measure(&project, {
        let mut element = text(50, one_run("hello"));
        element["line_height"] = json!(1.0);
        element["stroke_width"] = json!(8);
        element
    }));

    // ADR-0014: on text the stroke falls **outside** the glyph contour, so "the overflow
    // extent gains `2 x stroke_width` on both axes". If `measure` returned stroke-naive
    // numbers "every author adds `2 x stroke_width` by hand and they diverge" — ADR-0005's
    // `speed` failure exactly.
    assert_eq!(
        px(&stroked["extent"]["width"]),
        px(&bare["extent"]["width"]) + 16.0
    );
    assert_eq!(
        px(&stroked["extent"]["height"]),
        px(&bare["extent"]["height"]) + 16.0
    );
    assert_eq!(stroked["extent"]["stroke_width"], 8);

    // The typographic advance is unchanged and is reported separately, so the derivation
    // stays inspectable rather than mysterious. A stroke is paint, and paint moves no glyph.
    assert_eq!(
        px(&stroked["advance_width"]),
        px(&bare["advance_width"]),
        "a stroke is a second paint on the same outline, not a re-layout"
    );
    // And it moves no baseline: the block's own height is the typography's.
    assert_eq!(
        px(&stroked["lines"][0]["baseline_y"]),
        px(&bare["lines"][0]["baseline_y"])
    );
    assert_eq!(stroked["block_height"], bare["block_height"]);
}

#[test]
fn a_runs_own_stroke_width_reaches_the_extent() {
    let project = project(line!());

    let mut element = text(50, json!([{"text": "a"}, {"text": "b", "stroke_width": 4}]));
    element["line_height"] = json!(1.0);

    let view = view(&measure(&project, element));

    // ADR-0014 makes stroke a **run-addressable** paint — "under the alternative, *outline
    // one word* is not expressible at all" — so the extent has to see a run's own.
    assert_eq!(view["extent"]["stroke_width"], 4);
    assert_eq!(view["lines"][0]["stroke_width"], 4);
    assert_eq!(
        px(&view["extent"]["width"]),
        px(&view["advance_width"]) + 8.0
    );
}

#[test]
fn no_stroke_means_the_extent_and_the_advance_agree() {
    let project = project(line!());

    let view = view(&measure(&project, text(50, one_run("hello"))));

    assert_eq!(view["extent"]["stroke_width"], 0);
    assert_eq!(px(&view["extent"]["width"]), px(&view["advance_width"]));
}

// ---------------------------------------------------------------------------
// It derives and never judges (ADR-0024).
// ---------------------------------------------------------------------------

#[test]
fn the_declared_box_changes_nothing_about_the_answer() {
    let project = project(line!());

    let without = view(&measure(&project, text(50, one_run("hello"))));
    let with_absurd_box = view(&measure(&project, {
        let mut element = text(50, one_run("hello"));
        element["width"] = json!(1);
        element["height"] = json!(1);
        element
    }));

    // ADR-0024: "`measure` returns the bare derived extent ... no verdict, no diff against
    // the declared value", because ADR-0006 gives `validate` sole authority to compare the
    // document against itself. A 1x1 box around a 50px line is as wrong as a box gets, and
    // the answer is byte-identical.
    assert_eq!(without, with_absurd_box);
}

#[test]
fn nothing_in_the_answer_is_a_finding() {
    let project = project(line!());

    let answer = measure(&project, {
        let mut element = text(50, one_run("hello"));
        element["width"] = json!(1);
        element["height"] = json!(1);
        element
    });

    assert!(
        answer.report().findings.is_empty(),
        "measure reaches no verdict, so it has nothing to report as a finding: {:?}",
        answer.report().findings
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
}

#[test]
fn an_element_that_is_not_in_the_file_and_has_no_id_measures_identically() {
    let project = project(line!());

    // ADR-0024: `measure` "must work identically for an element being authored for the
    // first time". The project this measures against declares fonts and no elements at all.
    let anonymous = view(&measure(&project, text(50, one_run("hello"))));
    let named = view(&measure(&project, {
        let mut element = text(50, one_run("hello"));
        element["id"] = json!("caption-01");
        element
    }));

    assert_eq!(anonymous["asked"]["id"], Value::Null);
    assert_eq!(named["asked"]["id"], "caption-01");
    assert_eq!(anonymous["lines"], named["lines"]);
    assert_eq!(anonymous["extent"], named["extent"]);
}

#[test]
fn the_answer_states_the_style_it_was_taken_in() {
    let project = project(line!());

    let mut element = text(55, one_run("x"));
    element["line_height"] = json!(1.1);
    element["y"] = json!(1537);
    element["origin"] = json!("bottom-center");
    element["stroke_width"] = json!(3);

    let view = view(&measure(&project, element));

    // Two calls in a transcript are told apart by reading them, and every default the verb
    // applied is visible rather than assumed.
    assert_eq!(
        view["asked"],
        json!({
            "id": null,
            "font": "brand",
            "size": 55,
            "line_height_tenths": 11,
            "stroke_width": 3,
            "y": 1537,
            "origin": "bottom-center",
        })
    );
}

// ---------------------------------------------------------------------------
// The fonts are the declared ones and nothing else (ADR-0007).
// ---------------------------------------------------------------------------

#[test]
fn a_font_key_the_project_does_not_declare_is_an_invocation_error() {
    let project = project(line!());

    let mut element = text(50, one_run("x"));
    element["font"] = json!("headline");
    let answer = measure(&project, element);

    // Exit 3, not 1: the *command* named a font the document does not carry, and ADR-0011
    // keeps "fix the command" apart from "fix the project" precisely so the next move is
    // never guessed.
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    let json = answer.to_json().to_string();
    assert!(json.contains("headline"), "the refusal names the key");
    assert!(json.contains("fonts"), "and the table it is a key into");
}

#[test]
fn a_system_family_name_resolves_to_nothing_rather_than_to_a_system_font() {
    let project = project(line!());

    let mut element = text(50, one_run("x"));
    element["font"] = json!("Helvetica");
    let answer = measure(&project, element);

    // ADR-0007: "Always a file path, relative to the project. Never a system family name."
    // The fixture's own `SF Pro Rounded` resolved on its author's machine only because it
    // had been hand-installed, which is the failure this refusal exists to make impossible.
    // Nothing in this build can open a system font — `fontique`'s discovery is not compiled
    // in — so the only way to be wrong here is to *silently* fall back to a default face.
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert_eq!(answer.to_json()["measure"], Value::Null);
}

#[test]
fn a_declared_font_file_that_is_not_there_is_reported_against_the_font() {
    let project = project_with(r#"{"brand":[{"file":"fonts/Missing.otf"}]}"#, line!());

    let answer = measure(&project, text(50, one_run("x")));
    let json = answer.to_json();

    // A fact about the project and its disk, so a finding rather than exit 3 — and named
    // against the *font*, so the sentence points at the file to go and vendor rather than
    // at the project file that merely mentions it.
    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    assert_eq!(answer.report().findings[0].code, "E-READ");
    assert!(
        json.to_string().contains("Missing.otf"),
        "the finding names the font file: {json}"
    );
    assert_eq!(json["measure"], Value::Null);
}

#[test]
fn a_relative_font_path_resolves_against_the_project_file() {
    // ADR-0053: a relative path in the document resolves against the project file's own
    // directory, and a font is no exception. The font is copied next to the project so the
    // only thing that can make this pass is the resolution rule.
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).expect("a writable temporary directory");
    std::fs::copy(font_file(), dir.join("fonts/Brand.otf")).expect("the vendored font");
    let project = write_project(
        &dir,
        "p.montaget.json",
        &canonical(
            r#"{"frame":{"width":1080,"height":1920},"fps":25,"fonts":{"brand":[{"file":"fonts/Brand.otf"}]},"tracks":[]}"#,
        ),
    );

    let view = view(&measure(&project, text(50, one_run("hello"))));

    assert!(px(&view["advance_width"]) > 0.0);
}

#[test]
fn a_run_may_override_the_elements_font_with_another_declared_key() {
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).expect("a writable temporary directory");
    std::fs::copy(font_file(), dir.join("fonts/Brand.otf")).expect("the vendored font");
    let project = write_project(
        &dir,
        "p.montaget.json",
        &canonical(
            r#"{"frame":{"width":1080,"height":1920},"fps":25,"fonts":{"brand":[{"file":"fonts/Brand.otf"}],"accent":[{"file":"fonts/Brand.otf"}]},"tracks":[]}"#,
        ),
    );

    // Both keys resolve; a run naming the second measures rather than failing. Two keys
    // over one file is the normal case ADR-0057 describes — the `fonts` table is a
    // *reference* structure, and one file may appear under several keys.
    let view = view(&measure(
        &project,
        text(50, json!([{"text": "a"}, {"text": "b", "font": "accent"}])),
    ));
    assert_eq!(view["line_count"], 1);

    let answer = measure(
        &project,
        text(50, json!([{"text": "a"}, {"text": "b", "font": "nope"}])),
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::BadInvocation,
        "a run's own key is resolved on the same rule as the element's"
    );
}

// ---------------------------------------------------------------------------
// The verb's own boundaries (ADR-0011).
// ---------------------------------------------------------------------------

#[test]
fn a_malformed_project_is_never_partially_processed() {
    let dir = common::tempdir(line!());
    let path = dir.join("p.montaget.json");
    std::fs::write(&path, "{\"frame\":").expect("a writable temporary directory");

    let answer = measure(&path, text(50, one_run("x")));

    assert_eq!(answer.report().exit_code(), ExitCode::Unparseable);
    assert_eq!(answer.report().findings[0].code, "E-PARSE");
    assert_eq!(answer.to_json()["measure"], Value::Null);
}

#[test]
fn an_element_that_is_not_text_is_refused_by_name() {
    let project = project(line!());

    let answer = measure(
        &project,
        json!({"type": "image", "source": "a.png", "width": 10, "height": 10}),
    );

    // The fitted-extent half of `measure` (ADR-0024) is a later ticket's. Refusing by name
    // is the difference between "not built" and an image measured as if it were text.
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert!(answer.to_json().to_string().contains("ADR-0024"));
}

#[test]
fn an_element_missing_a_field_measurement_needs_is_refused_naming_it() {
    let project = project(line!());

    for (element, named) in [
        (json!({"font": "brand", "size": 50}), "runs"),
        (json!({"size": 50, "runs": []}), "font"),
        (json!({"font": "brand", "runs": []}), "size"),
    ] {
        let answer = measure(&project, element.clone());
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "{element}"
        );
        assert!(
            answer.to_json().to_string().contains(named),
            "the refusal names `{named}`: {element}"
        );
    }
}

#[test]
fn a_field_measurement_cannot_see_is_ignored_rather_than_refused() {
    let project = project(line!());

    // An element mid-authorship is exactly the element this verb exists for (ADR-0024), so
    // everything measurement has no use for — colour, alignment, a timed highlight window,
    // a keyframed transform — passes straight through.
    let mut element = text(
        50,
        json!([{"text": "x", "color": "#FFFFFF", "highlight": {"start": 0, "end": 1}}]),
    );
    element["color"] = json!("#245C8C");
    element["align"] = json!("center");
    element["opacity"] = json!([{"t": 0, "v": 0}, {"t": 100, "v": 1, "ease": "linear"}]);

    let answer = measure(&project, element);
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
}

#[test]
fn a_non_positive_size_is_refused_rather_than_measured() {
    let project = project(line!());

    for size in [0, -55] {
        let answer = measure(&project, text(size, one_run("x")));
        // A `size` is a length in the project's frame space (ADR-0012), and no text has a
        // non-positive one. Measuring it would report a block of zero or negative height as
        // if it were a measurement.
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "size {size}"
        );
        assert!(
            answer.to_json().to_string().contains("positive"),
            "size {size}"
        );
    }
}

#[test]
fn an_absurd_size_is_answered_without_wrapping_or_panicking() {
    let project = project(line!());

    // `size` is read off a document and carries no bound of its own, so the block
    // arithmetic runs in `i128` — the same widening `R-BOX-SLACK` already needed, for the
    // same reason. An `i64` product of three such numbers panics in a debug build and, far
    // worse, wraps to a plausible small number in a release one; a wrapped height an author
    // transcribes into `height` is exactly the plausible-and-wrong failure this verb exists
    // to remove.
    let mut element = text(i64::MAX, one_run("x"));
    element["line_height"] = json!(1.1);

    let view = view(&measure(&project, element));

    assert_eq!(
        view["block_height"],
        Value::Null,
        "no integer height fits, and `null` says so rather than a wrapped number"
    );
    // The prose says the same thing rather than printing a number of its own.
    let prose = montaget_core::wire::render_measure(
        &measure(&project, {
            let mut element = text(i64::MAX, one_run("x"));
            element["line_height"] = json!(1.1);
            element
        }),
        montaget_core::Wire::Text { verbose: false },
    );
    assert!(prose.contains("block height      ?"), "{prose}");
}

#[test]
fn no_element_at_all_is_an_invocation_error() {
    let project = project(line!());

    let answer = montaget_core::verbs::measure::measure(&project, &Ask::default());

    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert_eq!(answer.to_json()["measure"], Value::Null);
}

#[test]
fn an_element_with_no_runs_at_all_measures_as_one_empty_line() {
    let project = project(line!());

    let mut element = text(50, json!([]));
    element["line_height"] = json!(1.2);
    let view = view(&measure(&project, element));

    // `partition` never returns nothing, so neither does this: one line, no advance, and a
    // slot the base size reserved. Refusing would refuse the element an author has just
    // begun — and reporting parley's synthetic-whitespace advance would report a width for
    // an element carrying no character.
    assert_eq!(view["line_count"], 1);
    assert_eq!(px(&view["advance_width"]), 0.0);
    assert_eq!(view["block_height"], 60);
}

// ---------------------------------------------------------------------------
// Batch mode: `elements` and `all` (#317).
// ---------------------------------------------------------------------------

#[test]
fn elements_returns_n_measured_blocks_in_input_order_for_elements_absent_from_the_project() {
    let project = project(line!());

    let answer = measure_elements(
        &project,
        vec![
            text(20, one_run("one")),
            text(30, one_run("two")),
            text(40, one_run("three")),
        ],
    );

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let results = answer.to_json()["measure"]["results"]
        .as_array()
        .expect("a batch carries its results")
        .clone();
    assert_eq!(results.len(), 3);
    for (i, size) in [20, 30, 40].into_iter().enumerate() {
        assert_eq!(results[i]["index"], i);
        assert_eq!(results[i]["error"], Value::Null);
        assert_eq!(results[i]["ok"]["asked"]["size"], size);
    }
}

#[test]
fn one_bad_slot_in_elements_still_returns_the_others_with_its_own_error() {
    let project = project(line!());

    let mut good_first = text(20, one_run("ok"));
    good_first["id"] = json!("first");
    let mut bad = text(20, one_run("bad font"));
    bad["id"] = json!("second");
    bad["font"] = json!("not-declared");
    let mut good_last = text(20, one_run("also ok"));
    good_last["id"] = json!("third");

    let answer = measure_elements(&project, vec![good_first, bad, good_last]);

    // The call itself is not a refusal — a batch with one bad slot is still an answer.
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let results = answer.to_json()["measure"]["results"].clone();

    assert_eq!(results[0]["id"], "first");
    assert!(results[0]["error"].is_null());
    assert!(!results[0]["ok"].is_null());

    assert_eq!(results[1]["id"], "second");
    assert!(
        results[1]["ok"].is_null(),
        "the bad slot carries no measurement"
    );
    assert_eq!(results[1]["error"]["code"], "E-INVOCATION");
    assert!(
        results[1]["error"]["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("not-declared"),
        "{}",
        results[1]["error"]
    );

    assert_eq!(results[2]["id"], "third");
    assert!(
        !results[2]["ok"].is_null(),
        "the slot after the bad one is unaffected"
    );
}

#[test]
fn all_returns_one_measured_block_per_text_element_already_in_the_project() {
    let dir = common::tempdir(line!());
    std::fs::create_dir_all(dir.join("fonts")).expect("a writable temporary directory");
    std::fs::copy(font_file(), dir.join("fonts/Brand.otf")).expect("the vendored font");
    let project = write_project(
        &dir,
        "p.montaget.json",
        &canonical(
            r#"{"frame":{"width":1080,"height":1920},"fps":25,
                "fonts":{"brand":[{"file":"fonts/Brand.otf"}]},
                "tracks":[{"name":"captions","elements":[
                    {"id":"a","type":"text","font":"brand","size":20,
                     "runs":[{"text":"one"}]},
                    {"id":"b","type":"text","font":"brand","size":30,
                     "runs":[{"text":"two"}]},
                    {"id":"c","type":"image","source":"a.png","width":10,"height":10}
                ]}]}"#,
        ),
    );

    let answer = measure_all(&project);

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let results = answer.to_json()["measure"]["results"]
        .as_array()
        .expect("a batch carries its results")
        .clone();
    // Only the two text elements — the image is not `--all`'s business.
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["id"], "a");
    assert_eq!(results[1]["id"], "b");
    assert!(results[0]["error"].is_null());
    assert!(results[1]["error"].is_null());
}

#[test]
fn all_on_a_project_with_zero_text_elements_is_an_empty_batch_not_an_error() {
    let project = project(line!());

    let answer = measure_all(&project);

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    assert_eq!(
        answer.to_json()["measure"]["results"],
        json!([]),
        "no text elements is an empty answer, not a refusal"
    );
}

#[test]
fn elements_and_all_and_element_and_at_are_pairwise_mutually_exclusive() {
    let project = project(line!());
    let one = vec![text(20, one_run("x"))];

    let combos = [
        Ask {
            element: Some(text(20, one_run("x"))),
            at: Some(0),
            elements: None,
            all: false,
        },
        Ask {
            element: Some(text(20, one_run("x"))),
            at: None,
            elements: Some(one.clone()),
            all: false,
        },
        Ask {
            element: Some(text(20, one_run("x"))),
            at: None,
            elements: None,
            all: true,
        },
        Ask {
            element: None,
            at: Some(0),
            elements: Some(one.clone()),
            all: false,
        },
        Ask {
            element: None,
            at: Some(0),
            elements: None,
            all: true,
        },
        Ask {
            element: None,
            at: None,
            elements: Some(one),
            all: true,
        },
    ];

    for ask in combos {
        let answer = montaget_core::verbs::measure::measure(&project, &ask);
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "{ask:?}"
        );
        assert_eq!(answer.to_json()["measure"], Value::Null);
    }
}

// A malformed JSON array is not a shape the verb ever sees at all: `Ask::elements` is
// already a `Vec<Value>`, so parsing `--elements`'s string is the CLI adapter's own
// transport step, exercised in `montaget/tests/adapters.rs`
// (`cli_measure_elements_that_is_not_a_json_array_is_exit_3_not_a_per_slot_error`) — the
// same split `--element`'s own not-JSON case already draws.

// ---------------------------------------------------------------------------
// The wire (ADR-0006).
// ---------------------------------------------------------------------------

#[test]
fn the_prose_is_generated_from_the_canonical_json_and_carries_the_same_numbers() {
    use montaget_core::Wire;

    let project = project(line!());
    let mut element = text(55, one_run("I hang cobwebs over the door."));
    element["line_height"] = json!(1.1);
    element["y"] = json!(1537);
    element["id"] = json!("sentence-05");

    let answer = measure(&project, element);
    let prose = montaget_core::wire::render_measure(&answer, Wire::Text { verbose: false });
    let view = view(&answer);

    assert!(prose.contains("sentence-05"));
    assert!(prose.contains("line_height 1.1"), "{prose}");
    assert!(
        prose.contains(&view["block_height"].to_string()),
        "the block height an author types into `height`: {prose}"
    );
    assert!(prose.contains("icu_segmenter"), "{prose}");
    // ADR-0006: the report ends with its own scope, unconditionally.
    assert!(prose.contains("NOT CHECKED"));

    // And the other form is the JSON alone, never both in one invocation.
    let json = montaget_core::wire::render_measure(&answer, Wire::Json);
    assert!(!json.contains("NOT CHECKED\n"));
    assert_eq!(
        serde_json::from_str::<Value>(&json).expect("the canonical form is JSON")["measure"],
        view
    );
}
