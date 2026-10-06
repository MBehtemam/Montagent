//! What one unit is (ADR-0151 §3, amended by ADR-0153 §2): the segmentation a `units` block
//! counts in, the joined pieces that move as one under `by: letter`, and the bodies a placed
//! element's glyphs are drawn in.

use std::path::{Path, PathBuf};

use montagent_text::engine::VerticalOrigin;
use montagent_text::units::{By, bodies, pieces, segment};
use montagent_text::{Align, FontFile, Fonts, Run, Spec, place};

/// The text of every unit, in index order.
fn unit_texts(text: &str, by: By) -> Vec<String> {
    let segmentation = segment(text, by);
    (0..segmentation.count)
        .map(|unit| {
            segmentation
                .graphemes
                .iter()
                .filter(|g| g.unit == Some(unit))
                .map(|g| &text[g.range.clone()])
                .collect()
        })
        .collect()
}

#[test]
fn a_letter_is_a_grapheme_that_is_not_whitespace() {
    // `"A B"` is units 0 and 1: whitespace takes no step.
    assert_eq!(unit_texts("A B", By::Letter), ["A", "B"]);
    // A decomposed accent is one grapheme, and so one letter; punctuation is a letter.
    assert_eq!(unit_texts("e\u{301}!", By::Letter), ["e\u{301}", "!"]);
    // A family emoji is one grapheme.
    assert_eq!(unit_texts("👨‍👩‍👧 x", By::Letter), ["👨‍👩‍👧", "x"]);
}

#[test]
fn letters_are_counted_across_runs_and_lines_in_reading_order() {
    assert_eq!(unit_texts("ab\n\ncd", By::Letter), ["a", "b", "c", "d"]);
    // Right-to-left text counts in reading order, not on-screen order.
    assert_eq!(unit_texts("سلم", By::Letter), ["س", "ل", "م"]);
}

#[test]
fn a_word_takes_its_punctuation_from_before_on_its_line_and_otherwise_from_after() {
    assert_eq!(unit_texts("Hello, world!", By::Word), ["Hello,", "world!"]);
    // An opening quote belongs to the word it opens.
    assert_eq!(
        unit_texts("\u{201C}Go\u{201D} now", By::Word),
        ["\u{201C}Go\u{201D}", "now"]
    );
    // A digit or an emoji makes a word; a line opening with a dash gives it to the word after.
    assert_eq!(unit_texts("one\n— 42 🎉", By::Word), ["one", "—42", "🎉"]);
}

#[test]
fn a_line_is_a_line_with_at_least_one_letter() {
    assert_eq!(unit_texts("Top\n\n  \nBottom", By::Line), ["Top", "Bottom"]);
}

#[test]
fn whitespace_has_no_unit_under_any_by() {
    for by in [By::Letter, By::Word, By::Line] {
        let segmentation = segment("a b", by);
        let space = &segmentation.graphemes[1];
        assert_eq!(space.range, 1..2);
        assert_eq!(space.unit, None, "{by:?}");
    }
}

#[test]
fn the_unit_count_is_the_caption_pace_count_less_whitespace() {
    // The two counts share one grapheme rule: a letter stagger's count is the caption's
    // grapheme count with the spaces taken out, whatever the clusters are.
    let text = "e\u{301}👨‍👩‍👧 ok";
    let graphemes = montagent_text::units::graphemes(text).count();
    assert_eq!(graphemes, 5);
    assert_eq!(segment(text, By::Letter).count, 4);
}

// ---------------------------------------------------------------------------
// Joined pieces (ADR-0153 §1).
// ---------------------------------------------------------------------------

/// Each piece of more than one grapheme, as text.
fn piece_texts(text: &str) -> Vec<String> {
    let all: Vec<&str> = montagent_text::units::graphemes(text)
        .map(|(_, g)| g)
        .collect();
    pieces(text)
        .into_iter()
        .map(|range| all[range].concat())
        .collect()
}

#[test]
fn salaam_is_three_pieces_of_which_one_has_more_than_one_grapheme() {
    // `ا` joins nothing after it, so `السلام` is `ا`, `لسلا` and `م`.
    assert_eq!(piece_texts("السلام"), ["لسلا"]);
    assert_eq!(piece_texts("السلام عليكم"), ["لسلا", "عليكم"]);
}

#[test]
fn zwnj_breaks_a_join_and_zwj_and_tatweel_make_one() {
    assert_eq!(piece_texts("ب\u{200C}ب"), Vec::<String>::new());
    assert_eq!(piece_texts("بـب"), ["بـب"]);
    assert_eq!(piece_texts("ا\u{200D}"), Vec::<String>::new());
}

#[test]
fn latin_never_joins() {
    assert!(pieces("Hello").is_empty());
}

// ---------------------------------------------------------------------------
// Bodies: which glyphs move together, and where each one pivots.
// ---------------------------------------------------------------------------

fn workspace(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above this crate")
        .join(relative)
}

fn fonts() -> Fonts {
    let mut fonts = Fonts::new();
    fonts
        .register(
            "k",
            &[
                FontFile {
                    path: workspace("fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf"),
                    index: None,
                },
                FontFile {
                    path: workspace("fixtures/letter-spacing/fonts/NotoNaskhArabic-Regular.ttf"),
                    index: None,
                },
            ],
        )
        .expect("the fixture faces register");
    fonts
}

fn spec<'a>(runs: &'a [Run<'a>], letter_spacing: f64) -> Spec<'a> {
    Spec {
        runs,
        font: "k",
        size: 100,
        line_height_tenths: 12,
        stroke_width: 0,
        y: 0,
        vertical_origin: VerticalOrigin::Top,
        align: Align::Start,
        letter_spacing,
        optional_ligatures_off: true,
    }
}

fn placed(text: &str, letter_spacing: f64) -> montagent_text::Placement {
    let runs = [Run {
        text,
        ..Run::default()
    }];
    place(&mut fonts(), &spec(&runs, letter_spacing)).expect("places")
}

#[test]
fn every_glyph_knows_the_text_it_was_shaped_from() {
    let placement = placed("AB", 0.0);
    let texts: Vec<_> = placement
        .glyphs
        .iter()
        .map(|g| placement.clusters[g.cluster].text.clone())
        .collect();
    assert_eq!(texts, [Some(0..1), Some(1..2)]);
}

#[test]
fn a_latin_letter_is_its_own_body_boxed_by_its_advance_without_the_spacing() {
    let plain = placed("HIH", 0.0);
    let spaced = placed("HIH", 300.0);
    let plain_bodies = bodies("HIH", By::Letter, &plain);
    let spaced_bodies = bodies("HIH", By::Letter, &spaced);
    assert_eq!(spaced_bodies.bodies.len(), 3);
    for (a, b) in plain_bodies.bodies.iter().zip(&spaced_bodies.bodies) {
        let [l0, t0, r0, b0] = a.rect.expect("a box");
        let [l1, t1, r1, b1] = b.rect.expect("a box");
        // Same width, same slot: the spacing moves the box and never widens it.
        assert!(((r0 - l0) - (r1 - l1)).abs() < 1e-6);
        assert_eq!((t0, b0), (t1, b1));
        assert_eq!(b0 - t0, 120.0, "the line's slot");
    }
    // The third H moved right by two gaps of 30 px.
    let shift = spaced_bodies.bodies[2].rect.unwrap()[0] - plain_bodies.bodies[2].rect.unwrap()[0];
    assert!((shift - 60.0).abs() < 1e-3, "{shift}");
}

#[test]
fn a_joined_arabic_piece_is_one_body_and_keeps_every_letters_index() {
    let text = "السلام";
    let placement = placed(text, 0.0);
    let found = bodies(text, By::Letter, &placement);
    // Six letters, six units, whatever shaping did.
    assert_eq!(found.segmentation.count, 6);
    let groups: Vec<Vec<usize>> = found.bodies.iter().map(|b| b.units.clone()).collect();
    assert_eq!(groups, [vec![0], vec![1, 2, 3, 4], vec![5]]);
    assert!(found.bodies[1].joined);
    // Every glyph of the piece is drawn in that one body.
    for (glyph, body) in placement.glyphs.iter().zip(&found.glyph_body) {
        let start = placement.clusters[glyph.cluster]
            .text
            .clone()
            .unwrap()
            .start;
        let expected = match start {
            0 => 0,
            _ if start < text.len() - "م".len() => 1,
            _ => 2,
        };
        assert_eq!(*body, Some(expected), "glyph at byte {start}");
    }
}

#[test]
fn a_shaping_merge_is_one_body_timed_by_its_first_letter() {
    // Oswald's `fi`, shaped with its ligatures on: one glyph for two letters.
    let runs = [Run {
        text: "xfi",
        ..Run::default()
    }];
    let mut ligatures_on = spec(&runs, 0.0);
    ligatures_on.optional_ligatures_off = false;
    let placement = place(&mut fonts(), &ligatures_on).expect("places");
    assert_eq!(placement.glyphs.len(), 2, "`fi` is one glyph");
    let found = bodies("xfi", By::Letter, &placement);
    let groups: Vec<Vec<usize>> = found.bodies.iter().map(|b| b.units.clone()).collect();
    assert_eq!(groups, [vec![0], vec![1, 2]]);
    assert!(found.bodies[1].merged && !found.bodies[1].joined);
    assert_eq!(found.glyph_body, [Some(0), Some(1)]);
    assert_eq!(found.body_of_unit, [0, 1, 1]);
}

#[test]
fn a_word_body_runs_from_its_first_graphemes_leading_edge_to_its_lasts_trailing_edge() {
    let text = "AB CD";
    let placement = placed(text, 0.0);
    let found = bodies(text, By::Word, &placement);
    assert_eq!(found.bodies.len(), 2);
    let letters = bodies(text, By::Letter, &placement);
    let a = letters.bodies[0].rect.unwrap();
    let b = letters.bodies[1].rect.unwrap();
    assert_eq!(found.bodies[0].rect.unwrap(), [a[0], a[1], b[2], b[3]]);
}

#[test]
fn under_by_word_a_joined_piece_is_never_a_body_of_its_own() {
    let text = "السلام عليكم";
    let placement = placed(text, 0.0);
    let found = bodies(text, By::Word, &placement);
    let groups: Vec<Vec<usize>> = found.bodies.iter().map(|b| b.units.clone()).collect();
    assert_eq!(groups, [vec![0], vec![1]]);
    assert!(!found.bodies[0].joined && !found.bodies[0].merged);
}
