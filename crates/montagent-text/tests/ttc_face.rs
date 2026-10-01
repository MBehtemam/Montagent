//! A `.ttc` chain entry shapes in the face its `index` names.
//!
//! ADR-0007 gives a chain entry an `index` *"for `.ttc` collections, defaulting to 0"*,
//! counting *"49 of the fonts in a stock macOS `/System/Library/Fonts`"* as collections to
//! say this is the normal case. It was parsed, validated and dropped: `fontique` registers
//! every face of a collection into one family and `parley` picks between them by a
//! width/style/weight query the format does not carry, so the face that won was a property
//! of the file's other faces. Measured on a stock `Avenir Next.ttc`, all twelve declared
//! indices shaped `Handgloves`/100 to 546.90 — face 7 (Regular), never the declared one.
//!
//! **The collection under test is built here, from committed faces.** ADR-0007 exists
//! because a test that reads `/System/Library/Fonts` passes on its author's machine and
//! asserts nothing anywhere else — and five of ADR-0064's six tier-1 targets have no
//! `Avenir Next.ttc` at all, which is exactly the shape of failure this ticket is about.
//! So [`collection_of`] assembles the bytes from fonts already in this repository, and the
//! assertion is **differential**: each face of the built collection must measure as the
//! file it was built from, and those files must measure differently from each other.

use std::path::{Path, PathBuf};

use montagent_text::engine::VerticalOrigin;
use montagent_text::{Align, FontFile, Fonts, Run, Spec, measure};

/// Two faces with visibly different advance widths, both committed and both OFL, with
/// their licences beside them: the pair ADR-0087's Thai measurements were taken on. Both
/// are Regular, and that is load-bearing — see [`OPEN_RUNDE`].
const NOTO: &str = "docs/research/prototypes/thai-vertical-metrics/fonts/NotoSansThai-Regular.ttf";
const SARABUN: &str = "docs/research/prototypes/thai-vertical-metrics/fonts/Sarabun-Regular.ttf";
/// **Face 0 of the built collection, and Bold.** Two reasons, both about what the fixture
/// can catch. It is CFF where the two above are `glyf`, so the collection is not
/// homogeneous and the extraction cannot be quietly relying on a table set that happens to
/// be the same in every face. And putting the only non-Regular face *first* is what makes
/// `an_omitted_index_is_face_zero_and_not_whatever_an_attribute_query_wins` mean anything:
/// a default attribute query asks for weight 400 and would reach past it, so a fixture
/// whose face 0 were Regular would pass that test with the index still ignored. This is
/// the stock `Avenir Next.ttc` arrangement — face 0 Bold, the Regular at 7.
const OPEN_RUNDE: &str = "fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above this crate")
        .to_path_buf()
}

fn read(relative: &str) -> Vec<u8> {
    let path = repo_root().join(relative);
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {path:?}: {e}"))
}

/// Assemble a font collection whose faces are the given single-face files, in order.
///
/// A `.ttc` is a header of offsets into one file, each pointing at an ordinary table
/// directory — so a collection of N faces is the N directories rewritten to point at one
/// pooled copy of their tables. Nothing here is shared between faces: a real collection
/// usually shares, and the extraction must not care either way, so the simple form is the
/// honest fixture.
fn collection_of(faces: &[&[u8]]) -> Vec<u8> {
    let header = 12 + faces.len() * 4;
    let directories: Vec<usize> = faces
        .iter()
        .scan(header, |at, face| {
            let start = *at;
            *at += 12 + table_count(face) * 16;
            Some(start)
        })
        .collect();

    let mut out = vec![
        0u8;
        directories.last().copied().unwrap_or(header)
            + 12
            + table_count(faces.last().unwrap()) * 16
    ];
    out[0..4].copy_from_slice(b"ttcf");
    out[4..6].copy_from_slice(&1u16.to_be_bytes());
    out[6..8].copy_from_slice(&0u16.to_be_bytes());
    out[8..12].copy_from_slice(&(faces.len() as u32).to_be_bytes());
    for (position, directory) in directories.iter().enumerate() {
        let at = 12 + position * 4;
        out[at..at + 4].copy_from_slice(&(*directory as u32).to_be_bytes());
    }

    for (face, directory) in faces.iter().zip(&directories) {
        let count = table_count(face);
        out[*directory..*directory + 12].copy_from_slice(&face[0..12]);
        for position in 0..count {
            let record = 12 + position * 16;
            let from =
                u32::from_be_bytes(face[record + 8..record + 12].try_into().unwrap()) as usize;
            let length =
                u32::from_be_bytes(face[record + 12..record + 16].try_into().unwrap()) as usize;

            while !out.len().is_multiple_of(4) {
                out.push(0);
            }
            let at = out.len() as u32;
            out.extend_from_slice(&face[from..from + length]);

            let slot = directory + 12 + position * 16;
            out[slot..slot + 8].copy_from_slice(&face[record..record + 8]);
            out[slot + 8..slot + 12].copy_from_slice(&at.to_be_bytes());
            out[slot + 12..slot + 16].copy_from_slice(&(length as u32).to_be_bytes());
        }
    }
    out
}

fn table_count(face: &[u8]) -> usize {
    u16::from_be_bytes(face[4..6].try_into().unwrap()) as usize
}

/// The advance width of one string shaped through a one-entry chain over this file.
///
/// A width rather than a name: ADR-0007 makes the face's *metrics* what the document
/// depends on, and a name matching while the metrics do not is the failure mode a name
/// assertion would miss.
fn width(path: &Path, index: Option<u32>) -> f64 {
    let mut fonts = Fonts::new();
    fonts
        .register(
            "k",
            &[FontFile {
                path: path.to_path_buf(),
                index,
            }],
        )
        .unwrap_or_else(|e| panic!("registering {path:?} index {index:?}: {e}"));
    measure(
        &mut fonts,
        &Spec {
            runs: &[Run {
                text: "Handgloves",
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
    .unwrap_or_else(|e| panic!("measuring in {path:?} index {index:?}: {e}"))
    .extent
    .width
}

struct Built {
    collection: PathBuf,
    faces: Vec<PathBuf>,
}

/// The built collection and its three source faces, on disk.
///
/// Written out rather than held in memory because [`Fonts::register`] takes a path: the
/// registry reads from the filesystem and records what it opened (ADR-0007), and a test
/// that bypassed that would not be exercising the code a project goes through.
fn build(test: &str) -> Built {
    let dir = scratch(test);
    let sources = [OPEN_RUNDE, NOTO, SARABUN];
    let bytes: Vec<Vec<u8>> = sources.iter().map(|s| read(s)).collect();

    let faces = sources
        .iter()
        .zip(&bytes)
        .map(|(source, face)| {
            let path = dir.join(Path::new(source).file_name().unwrap());
            std::fs::write(&path, face).unwrap();
            path
        })
        .collect();

    let collection = dir.join("built.ttc");
    let slices: Vec<&[u8]> = bytes.iter().map(Vec::as_slice).collect();
    std::fs::write(&collection, collection_of(&slices)).unwrap();

    Built { collection, faces }
}

/// A fresh directory under the system temporary directory, per the pattern
/// `tests/sidecar.rs` established.
///
/// Named for its test and wiped on entry. Cargo runs these in parallel threads, and a
/// directory shared between them is one test deleting another's fixture mid-read — which
/// this file caught by reporting a three-face collection as not a font.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("montagent-ttc-face-tests")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("creating {dir:?}: {e}"));
    dir
}

#[test]
fn the_fixture_collection_has_faces_worth_telling_apart() {
    let built = build("faces-differ");

    let widths: Vec<f64> = built.faces.iter().map(|f| width(f, None)).collect();
    for (a, first) in widths.iter().enumerate() {
        for (b, second) in widths.iter().enumerate().skip(a + 1) {
            assert_ne!(
                first, second,
                "faces {a} and {b} of the fixture collection measure the same, so every \
                 assertion below would hold with the index ignored — which is the defect"
            );
        }
    }

    assert_eq!(
        montagent_text::sfnt::faces(&std::fs::read(&built.collection).unwrap()).unwrap(),
        3,
        "the built file is a three-face collection"
    );
}

#[test]
fn each_declared_index_shapes_in_the_face_it_names() {
    let built = build("each-index");

    for (index, face) in built.faces.iter().enumerate() {
        assert_eq!(
            width(&built.collection, Some(index as u32)),
            width(face, None),
            "`index` {index} of the collection did not shape as the file it was built \
             from. ADR-0007 declares the field and ADR-0007 also rules that a field the \
             renderer cannot honour is worse than no field."
        );
    }
}

#[test]
fn an_omitted_index_is_face_zero_and_not_whatever_an_attribute_query_wins() {
    let built = build("omitted-index");

    // The regression this catches is not a non-zero index: it is the *default*. ADR-0007
    // says `index` defaults to 0, and leaving the collection whole resolved it by
    // parley's default width/style/weight query instead — which on a stock
    // `Avenir Next.ttc` lands on face 7, not face 0.
    assert_eq!(
        width(&built.collection, None),
        width(&built.faces[0], None),
        "an omitted `index` must be face 0"
    );
    assert_eq!(
        width(&built.collection, None),
        width(&built.collection, Some(0))
    );
}

#[test]
fn an_index_past_the_end_names_no_face_and_says_how_many_there_are() {
    let built = build("past-the-end");

    let error = Fonts::new()
        .register(
            "k",
            &[FontFile {
                path: built.collection.clone(),
                index: Some(3),
            }],
        )
        .expect_err("face 3 of a three-face collection does not exist");
    assert_eq!(
        error.reason, "it has no face at `index` 3 (it carries 3)",
        "the refusal names the index and the count, so the author can correct it without \
         running `fonts list`"
    );
}

#[test]
fn a_single_face_file_takes_index_zero_and_refuses_any_other() {
    let built = build("single-face");

    assert!(
        width(&built.faces[0], Some(0)).is_finite(),
        "`index: 0` on a single-face file is the same request as omitting it"
    );
    assert_eq!(
        width(&built.faces[0], Some(0)),
        width(&built.faces[0], None)
    );

    let error = Fonts::new()
        .register(
            "k",
            &[FontFile {
                path: built.faces[0].clone(),
                index: Some(1),
            }],
        )
        .expect_err("a single-face file has no face 1");
    assert_eq!(error.reason, "it has no face at `index` 1 (it carries 1)");
}

#[test]
fn a_file_that_is_not_a_font_is_refused_as_one() {
    let path = scratch("not-a-font").join("not-a-font.ttf");
    std::fs::write(&path, b"this is not an sfnt").unwrap();

    let error = Fonts::new()
        .register(
            "k",
            &[FontFile {
                path: path.clone(),
                index: None,
            }],
        )
        .expect_err("prose is not a font");
    assert!(
        error.reason.contains("not a font"),
        "a non-font must still read as one, not as a missing face: {}",
        error.reason
    );
}

#[test]
fn validates_coverage_face_and_the_renderers_shaping_face_are_the_same_face() {
    // The sharpest symptom of the defect was never that `index` was ignored — it was that
    // it was ignored *on one side*. `checks/fonts.rs` asks `Charmap::of(bytes, index)`
    // about the declared face, so ADR-0007's glyph-coverage `error` was computed against
    // face N while the renderer drew whichever face a default attribute query won. A
    // project could validate clean on a face it never rendered, and render tofu on a face
    // no check ever read.
    //
    // The two now agree, but they agree through two different call sites — `Charmap::of`
    // reaches the face with `FontRef::from_index`, the registry cuts it out with
    // `sfnt::face` — so the agreement is asserted rather than assumed.
    let built = build("cross-verb");
    let collection = std::fs::read(&built.collection).unwrap();

    // A character each face disagrees about: Thai is in the two Thai faces and not in
    // Open Runde, which is face 0.
    const THAI: char = 'ก';

    let covers = |index: u32| {
        montagent_text::Charmap::of(&collection, Some(index))
            .unwrap()
            .covers(THAI)
    };
    assert!(!covers(0), "face 0 is Open Runde, which has no Thai");
    assert!(covers(1), "face 1 is Noto Sans Thai");

    // …and the renderer, asked for the same face, shapes in a font whose metrics match
    // that face's file. Coverage and metrics are different readings of one question —
    // which face is this — so if the two sides ever pick differently, one of these two
    // assertions moves without the other.
    for (index, face) in built.faces.iter().enumerate() {
        let index = index as u32;
        assert_eq!(
            covers(index),
            montagent_text::Charmap::of(&std::fs::read(face).unwrap(), None)
                .unwrap()
                .covers(THAI),
            "`validate` read face {index}'s coverage out of a face the renderer does not \
             shape in"
        );
        assert_eq!(width(&built.collection, Some(index)), width(face, None));
    }
}
