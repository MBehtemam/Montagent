//! A face registered from bytes shapes exactly as the same face registered from its file.
//!
//! #421 gives Montagent a face of its own for its chrome, embedded in the binary rather than
//! read off a disk a clean machine may not have — so the registry has to take bytes as well
//! as paths. What the bytes path must *not* do is behave differently: a face is its
//! outlines and metrics (ADR-0007), and where they were read from is not one of them.

use std::path::{Path, PathBuf};

use montagent_text::engine::VerticalOrigin;
use montagent_text::{Align, FaceMetrics, FontFile, Fonts, Run, Spec, measure};

const OPEN_RUNDE: &str = "fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf";

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above this crate")
        .join(relative)
}

fn read(relative: &str) -> Vec<u8> {
    let path = repo_path(relative);
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {path:?}: {e}"))
}

fn width(fonts: &mut Fonts, text: &str) -> f64 {
    measure(
        fonts,
        &Spec {
            runs: &[Run {
                text,
                font: None,
                size: None,
                stroke_width: None,
                dir: None,
            }],
            font: "k",
            size: 100,
            line_height_tenths: 12,
            stroke_width: 0,
            y: 0,
            vertical_origin: VerticalOrigin::Top,
            align: Align::Start,
        },
    )
    .unwrap_or_else(|e| panic!("measuring {text:?}: {e}"))
    .advance_width
}

#[test]
fn a_face_from_bytes_measures_as_the_same_face_from_its_file() {
    let mut from_file = Fonts::new();
    from_file
        .register(
            "k",
            &[FontFile {
                path: repo_path(OPEN_RUNDE),
                index: None,
            }],
        )
        .unwrap();

    let mut from_bytes = Fonts::new();
    from_bytes
        .register_bytes("k", &read(OPEN_RUNDE), None)
        .unwrap();

    for text in ["Handgloves", "9 42800ms +37 +word-08-bridge"] {
        assert_eq!(
            width(&mut from_bytes, text),
            width(&mut from_file, text),
            "{text:?} shapes to one width whichever way the face was registered"
        );
    }
}

#[test]
fn a_face_from_bytes_opens_no_file() {
    let mut fonts = Fonts::new();
    fonts.register_bytes("k", &read(OPEN_RUNDE), None).unwrap();
    width(&mut fonts, "Handgloves");
    assert!(
        fonts.opened().is_empty(),
        "`opened` is every path the registry read, and bytes are not a path: {:?}",
        fonts.opened()
    );
}

#[test]
fn bytes_that_are_not_a_font_are_refused_without_naming_a_path() {
    let error = Fonts::new()
        .register_bytes("k", b"definitely not an sfnt", None)
        .expect_err("text is not a font");
    assert_eq!(error.path, None, "no file was involved, so none is named");
    assert!(
        error.reason.contains("not a font"),
        "the reason says what was wrong: {}",
        error.reason
    );
}

#[test]
fn a_face_from_bytes_is_one_key_like_any_other() {
    let mut fonts = Fonts::new();
    fonts.register_bytes("k", &read(OPEN_RUNDE), None).unwrap();
    assert!(fonts.declares("k"));
    assert!(!fonts.declares("other"));
}

#[test]
fn face_metrics_are_the_files_own_integers() {
    let bytes = read(OPEN_RUNDE);
    let metrics = FaceMetrics::read(&bytes, None).unwrap();
    assert!(metrics.units_per_em() > 0);
    assert!(metrics.ascender() > 0, "the ascender is above the baseline");
    assert!(metrics.descender() < 0, "the descender is below it, y-up");
    assert_eq!(metrics.advance(""), 0);
    assert_eq!(
        metrics.advance("Hand"),
        metrics.advance("Ha") + metrics.advance("nd"),
        "an unshaped advance is the sum of its characters'"
    );
}

#[test]
fn a_character_the_face_does_not_map_advances_by_its_replacement_glyph() {
    // OpenRunde is Latin; U+0E01 is Thai. What is drawn in its place is `.notdef`, and
    // `.notdef` takes room — a width that counted it as nothing would under-fit a label.
    let bytes = read(OPEN_RUNDE);
    let metrics = FaceMetrics::read(&bytes, None).unwrap();
    assert!(metrics.advance("\u{0E01}") > 0);
}

#[test]
fn metrics_of_bytes_that_are_not_a_font_are_refused() {
    assert!(FaceMetrics::read(b"definitely not an sfnt", None).is_err());
}
