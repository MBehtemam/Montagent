//! Letter spacing (ADR-0151 §1, amended by ADR-0153 §3–§4).
//!
//! `size × letter_spacing / 1000` pixels after every grapheme of a line but the last, against
//! the size of the run the grapheme sits in; none between two letters of one joining script;
//! and optional ligatures off outside the joining scripts when the file asks for it.

use std::path::{Path, PathBuf};

use montagent_text::engine::VerticalOrigin;
use montagent_text::{Align, FontFile, Fonts, Placement, Run, Spec, measure, place};

const OPEN_RUNDE: &str = "fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf";

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
            &[FontFile {
                path: workspace(OPEN_RUNDE),
                index: None,
            }],
        )
        .expect("the fixture's face registers");
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
        optional_ligatures_off: letter_spacing != 0.0,
    }
}

fn run(text: &str) -> Run<'_> {
    Run {
        text,
        ..Run::default()
    }
}

fn advance(runs: &[Run<'_>], letter_spacing: f64) -> f64 {
    measure(&mut fonts(), &spec(runs, letter_spacing))
        .expect("measures")
        .advance_width
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn spacing_is_added_after_every_grapheme_but_the_lines_last() {
    // `HIH` has three graphemes and so two gaps: 100 × 200 / 1000 = 20 px each.
    let runs = [run("HIH")];
    let plain = advance(&runs, 0.0);
    let spaced = advance(&runs, 200.0);
    assert!(close(spaced - plain, 40.0), "{plain} → {spaced}");
}

#[test]
fn each_grapheme_is_spaced_by_its_own_runs_size_and_run_boundaries_count() {
    // `AB` at 100 and `CD` at 50, spacing 200: `A` 20, `B` 20 (the boundary counts, at
    // `B`'s own size), `C` 10, and nothing after the line's last, `D`.
    let runs = [
        run("AB"),
        Run {
            text: "CD",
            size: Some(50),
            ..Run::default()
        },
    ];
    let plain = advance(&runs, 0.0);
    let spaced = advance(&runs, 200.0);
    assert!(close(spaced - plain, 50.0), "{plain} → {spaced}");
}

#[test]
fn spaces_take_spacing_like_any_grapheme() {
    // `A B`: a gap after `A` and after the space, none after `B`.
    let runs = [run("A B")];
    assert!(close(advance(&runs, 100.0) - advance(&runs, 0.0), 20.0));
}

#[test]
fn negative_spacing_tightens_the_line() {
    let runs = [run("HIH")];
    assert!(close(advance(&runs, -50.0) - advance(&runs, 0.0), -10.0));
}

#[test]
fn every_line_of_a_block_is_spaced_and_none_past_its_own_last_grapheme() {
    let runs = [run("HI\nHIH")];
    let plain = measure(&mut fonts(), &spec(&runs, 0.0)).expect("measures");
    let spaced = measure(&mut fonts(), &spec(&runs, 200.0)).expect("measures");
    assert!(close(
        spaced.lines[0].advance_width - plain.lines[0].advance_width,
        20.0
    ));
    assert!(close(
        spaced.lines[1].advance_width - plain.lines[1].advance_width,
        40.0
    ));
}

/// The inked glyphs' `x`, left to right.
fn inked_x(placement: &Placement) -> Vec<f64> {
    let mut xs: Vec<f64> = placement
        .glyphs
        .iter()
        .filter(|glyph| !placement.outlines[glyph.outline].is_empty())
        .map(|glyph| glyph.x)
        .collect();
    xs.sort_by(f64::total_cmp);
    xs
}

#[test]
fn the_drawn_glyphs_move_by_the_gaps_before_them() {
    let runs = [run("HIH")];
    let plain = inked_x(&place(&mut fonts(), &spec(&runs, 0.0)).expect("places"));
    let spaced = inked_x(&place(&mut fonts(), &spec(&runs, 200.0)).expect("places"));
    let moved: Vec<f64> = spaced.iter().zip(&plain).map(|(s, p)| s - p).collect();
    assert!(
        moved.iter().zip([0.0, 20.0, 40.0]).all(|(m, e)| close(*m, e)),
        "{moved:?}"
    );
}

/// Oswald (which carries an `fi` ligature under `liga`) with Noto Naskh Arabic behind it, so
/// a mixed line shapes its Latin in one face and its Arabic in the other.
const OSWALD: &str = "fixtures/benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf";
const NASKH: &str = "fixtures/letter-spacing/fonts/NotoNaskhArabic-Regular.ttf";

fn mixed_fonts() -> Fonts {
    let mut fonts = Fonts::new();
    fonts
        .register(
            "k",
            &[
                FontFile {
                    path: workspace(OSWALD),
                    index: None,
                },
                FontFile {
                    path: workspace(NASKH),
                    index: None,
                },
            ],
        )
        .expect("the fixture faces register");
    fonts
}

fn glyph_count(text: &str, letter_spacing: f64, ligatures_off: bool) -> usize {
    let runs = [run(text)];
    let mut s = spec(&runs, letter_spacing);
    s.optional_ligatures_off = ligatures_off;
    place(&mut mixed_fonts(), &s)
        .expect("places")
        .glyphs
        .len()
}

#[test]
fn optional_ligatures_go_off_when_the_file_says_so_whatever_the_spacing_now() {
    // The rule is read from the file, never from the instant: an element whose spacing is
    // keyed through 0 is shaped with `fi` as two glyphs at 0 too.
    assert_eq!(glyph_count("fi", 0.0, false), 1, "Oswald's `fi` ligature");
    assert_eq!(glyph_count("fi", 0.0, true), 2);
    assert_eq!(glyph_count("fi", 200.0, true), 2);
}

#[test]
fn a_joining_scripts_letters_take_no_spacing_between_them() {
    // ADR-0153 §4: `سلام` is four Arabic letters, so no pair of them is spaced, and the
    // line's last takes none either.
    let runs = [run("سلام")];
    let measure_at = |ls: f64| {
        measure(&mut mixed_fonts(), &spec(&runs, ls))
            .expect("measures")
            .advance_width
    };
    assert!(close(measure_at(200.0), measure_at(0.0)));
}

#[test]
fn a_mixed_line_spreads_everything_but_its_joining_letter_pairs() {
    // `سلام fi`: no gap inside `سلام`; gaps after `م` (the next grapheme is a space), after
    // the space and after `f` — three, of 20 px each.
    let runs = [run("سلام fi")];
    // Ligatures off at both values, so only the spacing differs.
    let measure_at = |ls: f64| {
        let mut s = spec(&runs, ls);
        s.optional_ligatures_off = true;
        measure(&mut mixed_fonts(), &s)
            .expect("measures")
            .advance_width
    };
    assert!(
        close(measure_at(200.0) - measure_at(0.0), 60.0),
        "{} → {}",
        measure_at(0.0),
        measure_at(200.0)
    );
}

#[test]
fn ligatures_go_off_only_outside_the_joining_scripts() {
    // The Latin `fi` in a mixed line breaks apart; the Arabic run is shaped as today.
    assert_eq!(
        glyph_count("سلام fi", 0.0, true),
        glyph_count("سلام", 0.0, false) + 1 + 2
    );
    assert_eq!(
        glyph_count("سلام", 0.0, true),
        glyph_count("سلام", 0.0, false)
    );
}

#[test]
fn the_first_suppressed_pair_names_its_script_and_word() {
    let found = montagent_text::first_suppressed("Hello سلام عليكم").expect("suppressed");
    assert_eq!(found.script, "Arabic");
    assert_eq!(found.word, "سلام");
    assert_eq!(montagent_text::first_suppressed("Hello world"), None);
    // A lone Arabic letter beside Latin is no pair.
    assert_eq!(montagent_text::first_suppressed("a ب c"), None);
}

#[test]
fn the_ink_is_measured_on_the_spaced_line() {
    // Two lines whose glyphs share no horizontal space unspaced can be brought over each
    // other by spacing; the seam is measured where the glyphs are drawn.
    let runs = [run("HIH")];
    let plain = measure(&mut fonts(), &spec(&runs, 0.0)).expect("measures");
    let spaced = measure(&mut fonts(), &spec(&runs, 200.0)).expect("measures");
    assert_eq!(plain.ink_top, spaced.ink_top);
    assert!(close(
        spaced.extent.width - plain.extent.width,
        40.0
    ));
}
