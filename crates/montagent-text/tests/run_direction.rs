//! A run's `dir` is laid out as an isolate, and as nothing more (ADR-0007, ADR-0133).
//!
//! #457: the field was parsed, validated and dropped, so a document that set it rendered
//! as though it had not. These tests hold the three things the ruling turns on — the
//! override really reorders the run, it reorders *only* the run, and nothing the author did
//! not write reaches a number `measure` publishes.

use std::path::Path;

use montagent_text::engine::VerticalOrigin;
use montagent_text::{Align, Dir, FontFile, Fonts, PathEl, Placement, Run, Spec, measure, place};

const OPEN_RUNDE: &str = "fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf";

fn fonts() -> Fonts {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above this crate")
        .join(OPEN_RUNDE);
    let mut fonts = Fonts::new();
    fonts
        .register("k", &[FontFile { path, index: None }])
        .expect("the fixture's face registers");
    fonts
}

fn spec<'a>(runs: &'a [Run<'a>], align: Align) -> Spec<'a> {
    Spec {
        runs,
        font: "k",
        size: 100,
        line_height_tenths: 12,
        stroke_width: 0,
        y: 0,
        vertical_origin: VerticalOrigin::Top,
        align,
        letter_spacing: 0.0,
        optional_ligatures_off: false,
    }
}

fn run(text: &str, dir: Option<Dir>) -> Run<'_> {
    Run {
        text,
        dir,
        ..Run::default()
    }
}

/// The glyphs that draw something, left to right, as `(run, outline)`.
///
/// The outline itself rather than its index: [`Placement::outlines`] is numbered in the
/// order glyphs are first met, so the same letter has different indices in two placements.
fn visual(placement: &Placement) -> Vec<(usize, Vec<PathEl>)> {
    let mut inked: Vec<_> = placement
        .glyphs
        .iter()
        .filter(|glyph| !placement.outlines[glyph.outline].is_empty())
        .collect();
    inked.sort_by(|a, b| a.x.total_cmp(&b.x));
    inked
        .iter()
        .map(|glyph| (glyph.run, placement.outlines[glyph.outline].clone()))
        .collect()
}

/// Which run each inked glyph came from, left to right.
fn runs_in_order(placement: &Placement) -> Vec<usize> {
    visual(placement).into_iter().map(|(run, _)| run).collect()
}

/// The outlines one run draws, left to right.
fn outlines_of(placement: &Placement, which: usize) -> Vec<Vec<PathEl>> {
    visual(placement)
        .into_iter()
        .filter(|(run, _)| *run == which)
        .map(|(_, outline)| outline)
        .collect()
}

#[test]
fn an_rtl_override_reorders_the_run_as_an_isolate_and_nothing_outside_it() {
    let mut fonts = fonts();
    let plain = [run("x ", None), run("abc!", None), run(" y", None)];
    let isolated = [
        run("x ", None),
        run("abc!", Some(Dir::Rtl)),
        run(" y", None),
    ];
    let plain = place(&mut fonts, &spec(&plain, Align::Start)).unwrap();
    let isolated = place(&mut fonts, &spec(&isolated, Align::Start)).unwrap();

    // Left to right, `a b c !` without the override.
    let written = outlines_of(&plain, 1);
    assert_eq!(written.len(), 4, "a, b, c and ! each draw: {written:?}");
    let (letters, bang) = (&written[..3], written[3].clone());

    // Inside an RTL isolate the letters stay a left-to-right span (UAX #9 I2) and the
    // trailing `!`, a neutral between that span and the isolate's own right-to-left edge,
    // takes the isolate's direction and lands on the far side of it: `!abc`.
    let mut expected = vec![bang];
    expected.extend_from_slice(letters);
    assert_eq!(
        outlines_of(&isolated, 1),
        expected,
        "an RTL isolate draws `abc!` as `!abc`"
    );
    // RLO would have reversed every character. ADR-0007 rules it out by name.
    let mut overridden = written.clone();
    overridden.reverse();
    assert_ne!(outlines_of(&isolated, 1), overridden);

    // The isolate is the whole point: nothing outside it moves, and nothing inside it
    // crosses to the other side of its neighbours.
    assert_eq!(runs_in_order(&isolated), runs_in_order(&plain));
    assert_eq!(runs_in_order(&isolated), vec![0, 1, 1, 1, 1, 2]);
}

#[test]
fn an_ltr_override_on_ltr_text_draws_and_measures_exactly_what_no_override_does() {
    let mut fonts = fonts();
    let plain = [run("Hello, ", None), run("world", None)];
    let isolated = [run("Hello, ", None), run("world", Some(Dir::Ltr))];
    let plain = place(&mut fonts, &spec(&plain, Align::Center)).unwrap();
    let isolated = place(&mut fonts, &spec(&isolated, Align::Center)).unwrap();

    assert_eq!(isolated.measurement, plain.measurement);
    assert_eq!(isolated.width, plain.width);
    // Every glyph that draws is where it was. The isolate's own marks shape as nothing.
    let inked = |p: &Placement| -> Vec<(i64, usize)> {
        p.glyphs
            .iter()
            .filter(|g| !p.outlines[g.outline].is_empty())
            .map(|g| ((g.x * 1000.0).round() as i64, g.run))
            .collect()
    };
    assert_eq!(inked(&isolated), inked(&plain));
}

#[test]
fn nothing_the_author_did_not_write_reaches_what_measure_publishes() {
    let mut fonts = fonts();
    let plain = [run("x ", None), run("abc!\nde", None), run(" y", None)];
    let isolated = [
        run("x ", None),
        run("abc!\nde", Some(Dir::Rtl)),
        run(" y", None),
    ];
    let plain = measure(&mut fonts, &spec(&plain, Align::Start)).unwrap();
    let isolated = measure(&mut fonts, &spec(&isolated, Align::Start)).unwrap();

    assert_eq!(isolated.line_count, 2);
    for (got, want) in isolated.lines.iter().zip(&plain.lines) {
        // The line's text, where it starts, and where it may break are all offsets into
        // what the author wrote — never into the laid-out string the marks went into.
        assert_eq!(got.text, want.text);
        assert_eq!(got.start, want.start);
        assert_eq!(got.break_opportunities, want.break_opportunities);
        // The marks advance by nothing and carry no metrics of their own.
        assert!(
            (got.advance_width - want.advance_width).abs() < 1e-6,
            "line {}: {} against {}",
            got.index,
            got.advance_width,
            want.advance_width
        );
        assert_eq!(got.ascent, want.ascent);
        assert_eq!(got.descent, want.descent);
        assert_eq!(got.baseline_y, want.baseline_y);
    }
    assert_eq!(isolated.block_height_tenths, plain.block_height_tenths);
}

#[test]
fn measure_publishes_what_the_painter_lays_out_under_an_override() {
    let mut fonts = fonts();
    let runs = [
        run("x ", None),
        run("abc!\nlonger line", Some(Dir::Rtl)),
        run(" y", None),
    ];
    for align in [Align::Start, Align::Center, Align::End] {
        let measured = measure(&mut fonts, &spec(&runs, align)).unwrap();
        let placed = place(&mut fonts, &spec(&runs, align)).unwrap();
        assert_eq!(placed.measurement, measured);
        assert_eq!(placed.width, measured.advance_width);
        // Every glyph sits inside the advance `measure` published for the block.
        for glyph in &placed.glyphs {
            assert!(
                glyph.x >= -1e-6 && glyph.x <= measured.advance_width + 1e-6,
                "{align:?}: a glyph at {} falls outside 0..{}",
                glyph.x,
                measured.advance_width
            );
        }
    }
}

#[test]
fn an_override_never_changes_the_base_direction_start_and_end_resolve_against() {
    // A line that is all Hebrew is a right-to-left paragraph. UAX #9 P2 skips an
    // isolate's contents when it looks for the first strong character, so wrapping the
    // whole line in an isolate would make it left-to-right and send `start` to the other
    // edge. ADR-0133: the base direction is the one the author's characters give, with or
    // without `dir`. The face has no Hebrew; bidi classes are the characters', not the
    // font's, and the replacement glyphs still draw.
    let mut fonts = fonts();
    let plain = [run("שלום", None), run("\nabcdefghij", None)];
    let isolated = [run("שלום", Some(Dir::Ltr)), run("\nabcdefghij", None)];
    let plain_m = measure(&mut fonts, &spec(&plain, Align::Start)).unwrap();
    let isolated_m = measure(&mut fonts, &spec(&isolated, Align::Start)).unwrap();
    assert!(plain_m.lines[0].rtl, "an all-Hebrew line is right-to-left");
    assert!(isolated_m.lines[0].rtl, "and `dir` does not change that");
    assert!(!isolated_m.lines[1].rtl);

    let first_line_left = |p: &Placement| {
        p.glyphs
            .iter()
            .filter(|g| g.run == 0 && !p.outlines[g.outline].is_empty())
            .map(|g| g.x)
            .fold(f64::INFINITY, f64::min)
    };
    let plain_p = place(&mut fonts, &spec(&plain, Align::Start)).unwrap();
    let isolated_p = place(&mut fonts, &spec(&isolated, Align::Start)).unwrap();
    assert!(
        first_line_left(&plain_p) > 0.0,
        "`start` on an RTL line is the right edge"
    );
    assert!(
        (first_line_left(&isolated_p) - first_line_left(&plain_p)).abs() < 1e-6,
        "the override moved the line from {} to {}",
        first_line_left(&plain_p),
        first_line_left(&isolated_p)
    );
}
