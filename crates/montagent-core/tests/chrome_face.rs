//! Montagent's own chrome face (#421): the one face its labels are drawn in, never the
//! project's and never the system's (ADR-0098 §9).
//!
//! Each requirement ADR-0098 puts on the face is asserted here against the embedded bytes
//! themselves — the licence gate, tabular figures, ASCII coverage, a visible replacement
//! glyph — together with the one property the label sizing leans on without saying so: that
//! the unshaped metrics it does its arithmetic in are the widths the shaper actually lays
//! out. Presence is not asserted, because it cannot fail at test time: the face is
//! `include_bytes!`'d, so a build without it does not compile.

use std::path::Path;

use montagent_core::fonts::chrome;
use montagent_core::fonts::licence::{Bucket, bucket};
use montagent_core::fonts::sha256_hex;
use montagent_text::engine::VerticalOrigin;
use montagent_text::{Align, Charmap, Run, Spec, measure, names, place};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above this crate")
}

/// Every printable ASCII character — the label's alphabet, and what an element id is
/// overwhelmingly spelled in.
fn printable_ascii() -> impl Iterator<Item = char> {
    (0x20u8..=0x7e).map(char::from)
}

/// `text` shaped in the chrome face at `size`, as the renderer would lay it out.
fn shaped_width(text: &str, size: i64) -> f64 {
    let runs = [Run {
        text,
        font: None,
        size: None,
        stroke_width: None,
        dir: None,
    }];
    measure(
        &mut chrome::fonts(),
        &Spec {
            runs: &runs,
            font: chrome::KEY,
            size,
            line_height_tenths: 10,
            stroke_width: 0,
            y: 0,
            vertical_origin: VerticalOrigin::Top,
            align: Align::Start,
            letter_spacing: 0.0,
            optional_ligatures_off: false,
        },
    )
    .unwrap_or_else(|e| panic!("shaping {text:?} in the chrome face: {e}"))
    .advance_width
}

#[test]
fn the_embedded_face_is_the_file_the_third_party_notices_attest() {
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("ci/third-party-notices/bundled.json")).unwrap(),
    )
    .unwrap();
    let fonts = manifest["fonts"]
        .as_array()
        .expect("bundled.json lists fonts");
    let entry = fonts
        .iter()
        .find(|f| f["path"].as_str().unwrap().ends_with(chrome::FILE))
        .unwrap_or_else(|| panic!("bundled.json attributes no font at {}", chrome::FILE));

    assert_eq!(
        entry["sha256"].as_str().unwrap(),
        sha256_hex(chrome::BYTES),
        "the bytes in the binary are the bytes the notice was written for"
    );
    let on_disk = std::fs::read(repo_root().join(entry["path"].as_str().unwrap())).unwrap();
    assert_eq!(
        on_disk,
        chrome::BYTES,
        "and the attested path is the file embedded"
    );
}

#[test]
fn the_face_passes_the_vendoring_gate_as_a_recognised_open_licence() {
    // ADR-0057's gate, asked exactly as `fonts vendor` asks it of a project's font. The
    // chrome face is held to the rule a project's fonts are, not to a softer one.
    let faces = names::faces(chrome::BYTES).unwrap();
    assert_eq!(
        bucket(&faces),
        Bucket::Recognised {
            identifier: "OFL-1.1"
        }
    );
}

#[test]
fn every_printable_ascii_character_has_a_glyph() {
    let charmap = Charmap::of(chrome::BYTES, None).unwrap();
    let missing: String = printable_ascii().filter(|&c| !charmap.covers(c)).collect();
    assert!(missing.is_empty(), "no glyph for {missing:?}");
}

#[test]
fn the_figures_are_tabular() {
    // The label's payload is columns of milliseconds (ADR-0098 §9): `42800` must take the
    // room `11111` does, or a column of instants does not line up.
    let metrics = chrome::metrics();
    let one = metrics.advance("0");
    for digit in '0'..='9' {
        assert_eq!(
            metrics.advance(&digit.to_string()),
            one,
            "{digit} advances as far as 0"
        );
    }
}

#[test]
fn the_face_is_monospaced_over_the_label_alphabet_and_its_replacement_glyph() {
    // The stronger property, and the one that makes ADR-0098's measured `295/chars` law
    // exact rather than ±9 %: a label's width is its character count times one advance.
    let metrics = chrome::metrics();
    let advance = metrics.advance("0");
    for c in printable_ascii() {
        assert_eq!(metrics.advance(&c.to_string()), advance, "{c:?}");
    }
    assert_eq!(
        metrics.advance("\u{0E01}"),
        advance,
        "a character outside the face's coverage is drawn as `.notdef` at the same pitch"
    );
    assert_eq!(
        chrome::ADVANCE,
        advance,
        "the constant the sizing reads is the face's"
    );
    assert_eq!(chrome::UNITS_PER_EM, metrics.units_per_em() as u32);
    assert_eq!(chrome::ASCENDER, metrics.ascender());
    assert_eq!(chrome::DESCENDER, metrics.descender());
}

#[test]
fn the_unshaped_metrics_are_the_widths_the_shaper_lays_out() {
    // The sizing does its fit arithmetic in `chrome::metrics()` and the renderer draws with
    // `chrome::fonts()`; if the two disagreed, a label judged to fit would overrun its
    // tile. They cannot, only while the face has no kerning, ligature or contextual
    // alternate — so this asserts it, on strings holding the pairs a ligature font joins.
    let metrics = chrome::metrics();
    let em = metrics.units_per_em() as i64;
    for text in [
        "9 42800ms +37 +word-08-bridge",
        "-> -- == != <= >= www ::",
        "AVATAR To Wa fi fl",
    ] {
        assert_eq!(
            shaped_width(text, em),
            metrics.advance(text) as f64,
            "{text:?} at one unit per pixel"
        );
        let floor = shaped_width(text, 8);
        let expected = metrics.advance(text) as f64 * 8.0 / em as f64;
        assert!(
            (floor - expected).abs() < 1e-3,
            "{text:?} at the 8 px floor: shaped {floor}, arithmetic {expected}"
        );
    }
}

#[test]
fn a_character_outside_the_face_draws_a_visible_replacement_glyph() {
    // ADR-0098 §9: a visible replacement rather than silent omission. Thai is outside the
    // face; what is drawn must be ink, not a blank the reader would take for a space.
    let runs = [Run {
        text: "\u{0E01}",
        font: None,
        size: None,
        stroke_width: None,
        dir: None,
    }];
    let placement = place(
        &mut chrome::fonts(),
        &Spec {
            runs: &runs,
            font: chrome::KEY,
            size: 100,
            line_height_tenths: 10,
            stroke_width: 0,
            y: 0,
            vertical_origin: VerticalOrigin::Top,
            align: Align::Start,
            letter_spacing: 0.0,
            optional_ligatures_off: false,
        },
    )
    .unwrap();
    assert_eq!(placement.glyphs.len(), 1, "one character, one glyph");
    let outline = &placement.outlines[placement.glyphs[0].outline];
    assert!(
        !outline.is_empty(),
        "the replacement glyph has an outline to fill"
    );
}
