//! `frame` — the first pixels (#212).
//!
//! Asserted at spec #168's **seam 1**: a real project file goes to the core verb and the
//! answer comes back as values. Where a claim is genuinely about pixels — a shape's stroke
//! falling inside its rect, `origin` placing a box, an `opacity` compositing once — the
//! test asks for PNG at full scale and reads the pixels back, because that is the one class
//! of claim no report can reach (seam 2).
//!
//! Two decisions about *how* it asserts, both taken for the same reason:
//!
//! - **Pixels are read with an independent decoder** (`image`), never with the Skia codec
//!   the renderer encoded them with. An encoder and its own decoder agreeing proves they
//!   agree, not that the bytes are a picture.
//! - **Every geometric assertion names a coordinate the document states**, not one derived
//!   by repeating the renderer's arithmetic. `origin: "top-left"` at `x: 100` means the
//!   box's left edge is at 100; that sentence is the whole test, and a test that
//!   recomputed `x - fx * width` would pass against a renderer that had the same bug.

use std::path::{Path, PathBuf};

use montaget_core::report::ExitCode;
use montaget_core::verbs::frame::{Ask, frame};
use serde_json::Value;

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating")
        .canonicalize()
        .expect("the committed fixture")
}

fn fixture_project() -> PathBuf {
    fixture_dir().join("en-halloween-decorating.montaget.json")
}

fn video_project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/video-decode/video-decode.montaget.json")
        .canonicalize()
        .expect("the decode path's own fixture (#212)")
}

fn at(instant: i64) -> Ask {
    Ask {
        at: Some(instant),
        ..Ask::default()
    }
}

/// One frame, refusing to continue if the verb did not answer.
#[track_caller]
fn drawn(project: &Path, ask: &Ask) -> (Value, Vec<u8>) {
    let answer = frame(project, ask);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    let bytes = answer
        .image()
        .expect("an answer with exit 0 carries a picture")
        .bytes
        .clone();
    (answer.to_json(), bytes)
}

/// The decoded picture, through a decoder that is not the one that wrote it.
#[track_caller]
fn pixels(bytes: &[u8]) -> image::RgbaImage {
    image::load_from_memory(bytes)
        .expect("the bytes decode as a picture")
        .to_rgba8()
}

#[track_caller]
fn rgb(picture: &image::RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let p = picture.get_pixel(x, y).0;
    [p[0], p[1], p[2]]
}

/// A project with one track holding the given elements, on a 400×400 black frame.
///
/// Small and black on purpose: every pixel assertion below is "is this pixel the shape's
/// colour or the background's", and a 400×400 frame renders in milliseconds while staying
/// large enough for a stroke to be several pixels wide.
fn one_track(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
            "tracks":[{{"name":"only","layer":0,"elements":[{elements}]}}]}}"##
    ))
}

const RED: [u8; 3] = [0xFF, 0x00, 0x00];
const BLUE: [u8; 3] = [0x00, 0x00, 0xFF];
const BLACK: [u8; 3] = [0x00, 0x00, 0x00];

/// The exact pixels of a small project, at true scale and lossless.
#[track_caller]
fn painted(line: u32, elements: &str) -> image::RgbaImage {
    let dir = tempdir(line);
    let project = write_project(&dir, "p.montaget.json", &one_track(elements));
    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(500)
        },
    );
    pixels(&bytes)
}

// ---------------------------------------------------------------------------
// The answer's shape: encoding, scale, crop, and the caption that never goes away
// ---------------------------------------------------------------------------

#[test]
fn the_default_is_jpeg_at_half_the_projects_frame_size() {
    // ADR-0011's whole reason for the default: a 1080x1920 frame costs 2691 visual tokens
    // and a 540x960 one costs 700, "every time the agent looks". The picture is still
    // *rasterized* at true pixels — ADR-0021 is explicit that `frame` is never
    // proxy-scaled — so the answer reports both numbers and they differ by exactly 2x.
    let (json, bytes) = drawn(&fixture_project(), &at(11000));

    assert_eq!(json["frame"]["encoding"], "jpeg");
    assert_eq!(json["frame"]["scale"], "half");
    assert_eq!(json["frame"]["width"], 540);
    assert_eq!(json["frame"]["height"], 960);
    assert_eq!(json["frame"]["rasterized"]["width"], 1080);
    assert_eq!(json["frame"]["rasterized"]["height"], 1920);

    // JFIF/EXIF both open with the same start-of-image marker, and the independent
    // decoder is what says it is really a JPEG.
    assert_eq!(&bytes[..2], &[0xFF, 0xD8], "a JPEG starts with SOI");
    let picture = pixels(&bytes);
    assert_eq!((picture.width(), picture.height()), (540, 960));
}

#[test]
fn full_scale_and_png_are_each_behind_their_own_flag() {
    let (json, bytes) = drawn(
        &fixture_project(),
        &Ask {
            full: true,
            png: true,
            ..at(11000)
        },
    );

    assert_eq!(json["frame"]["encoding"], "png");
    assert_eq!(json["frame"]["scale"], "full");
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "a PNG starts with its signature"
    );
    let picture = pixels(&bytes);
    assert_eq!(
        (picture.width(), picture.height()),
        (1080, 1920),
        "full scale is the project's true pixels"
    );

    // The two flags are independent: PNG at half scale is a legal, and different, answer.
    let (half, half_bytes) = drawn(
        &fixture_project(),
        &Ask {
            png: true,
            ..at(11000)
        },
    );
    assert_eq!(half["frame"]["encoding"], "png");
    assert_eq!(half["frame"]["scale"], "half");
    assert_eq!(pixels(&half_bytes).width(), 540);
}

#[test]
fn crop_returns_the_requested_region() {
    // The region is stated in frame space at true pixels, and the default half scale
    // applies to it like any other extent — so an 800x600 crop comes back 400x300.
    let (json, bytes) = drawn(
        &fixture_project(),
        &Ask {
            crop: Some("100,200,800,600".into()),
            ..at(11000)
        },
    );

    // `region` rather than `crop`: the `query --at` block beside it already spends `crop`
    // on which part of a *source file* survives the aperture, and one word for two
    // quantities in one answer is how a reader takes the wrong number away.
    assert_eq!(json["frame"]["region"]["x"], 100);
    assert_eq!(json["frame"]["region"]["y"], 200);
    assert_eq!(json["frame"]["region"]["width"], 800);
    assert_eq!(json["frame"]["region"]["height"], 600);
    let picture = pixels(&bytes);
    assert_eq!((picture.width(), picture.height()), (400, 300));
}

#[test]
fn a_crop_returns_the_pixels_that_are_actually_there() {
    // The claim a dimension check cannot make: that the region is the *right* region. A
    // 100x100 red square at the top-left of a black frame, cropped to a window that
    // contains only its right half.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &one_track(
            r##"{"id":"square","type":"rect","start":0,"end":1000,"x":0,"y":0,
                "origin":"top-left","width":100,"height":100,"fill":"#FF0000"}"##,
        ),
    );

    let (_, bytes) = drawn(
        &project,
        &Ask {
            crop: Some("50,0,100,100".into()),
            full: true,
            png: true,
            ..at(500)
        },
    );
    let picture = pixels(&bytes);

    assert_eq!((picture.width(), picture.height()), (100, 100));
    assert_eq!(
        rgb(&picture, 10, 50),
        RED,
        "the crop's left half is the square"
    );
    assert_eq!(
        rgb(&picture, 90, 50),
        BLACK,
        "and its right half is past the square's edge"
    );
}

#[test]
fn a_crop_that_reaches_past_the_frame_returns_the_part_that_is_inside_it() {
    let (json, bytes) = drawn(
        &fixture_project(),
        &Ask {
            crop: Some("900,1800,400,400".into()),
            full: true,
            ..at(11000)
        },
    );

    assert_eq!(json["frame"]["region"]["width"], 180, "1080 - 900");
    assert_eq!(json["frame"]["region"]["height"], 120, "1920 - 1800");
    assert_eq!(pixels(&bytes).dimensions(), (180, 120));
}

#[test]
fn a_crop_that_misses_the_frame_entirely_is_a_wrong_command_and_not_a_broken_montaget() {
    // ADR-0011 keeps exit 3 — "the invocation was wrong, fix the command" — apart from
    // exit 70, "Montaget could not run". A region outside the frame is the first of those:
    // nothing is wrong with the project and nothing failed, the caller named a rectangle
    // that is not part of the picture. It cannot be caught before the file is opened,
    // because it is only wrong relative to the frame the document declares.
    let answer = frame(
        &fixture_project(),
        &Ask {
            crop: Some("4000,4000,100,100".into()),
            ..at(11000)
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
    assert_eq!(answer.report().findings[0].code, "E-INVOCATION");
    assert!(answer.image().is_none());
}

#[test]
fn a_frame_that_is_not_a_frame_is_a_defect_in_the_document_and_not_in_montaget() {
    // The project carries a `frame` key — ADR-0042's structural predicate is satisfied —
    // holding something that is not a frame. That is exit 1: a defect in the document, and
    // ADR-0011 reserves 70 for "Montaget could not run" precisely so the two do not blur.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &canonical(r##"{"frame":{"width":"wide","height":1920},"fps":25,"tracks":[]}"##),
    );

    let answer = frame(&project, &at(0));
    assert_eq!(answer.report().exit_code(), ExitCode::Errors);
    assert_eq!(answer.report().findings[0].code, "E-NOT-A-PROJECT");
    assert!(answer.image().is_none());
}

#[test]
fn a_crop_that_is_not_four_whole_pixels_is_exit_3_and_never_a_verdict() {
    // ADR-0011 keeps exit 3 apart from exit 1 so that "fix the command" is never read as
    // "fix the project" — and the invocation is settled before the file is opened, so this
    // holds for a project path that does not exist at all.
    for spelling in ["1,2,3", "1,2,3,4,5", "1,2,0,4", "1,2,-3,4", "a,2,3,4"] {
        let answer = frame(
            Path::new("/nowhere/no-such-project.montaget.json"),
            &Ask {
                crop: Some(spelling.into()),
                ..at(0)
            },
        );
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::BadInvocation,
            "`--crop {spelling}` is a bad command, not a bad project"
        );
        assert_eq!(answer.report().findings[0].code, "E-INVOCATION");
    }
}

#[test]
fn an_instant_is_required_because_no_verb_chooses_what_you_are_looking_at() {
    let answer = frame(&fixture_project(), &Ask::default());
    assert_eq!(answer.report().exit_code(), ExitCode::BadInvocation);
}

#[test]
fn the_query_at_block_prints_alongside_the_image_under_every_flag() {
    // ADR-0011: "`frame` must print the `query --at` block alongside the image,
    // unconditionally." Asserted over every combination of the flags there are, because
    // "unconditionally" is a claim about all of them and a spot check is a claim about one.
    for full in [false, true] {
        for png in [false, true] {
            for crop in [None, Some("0,0,200,200".to_string())] {
                let ask = Ask {
                    full,
                    png,
                    crop: crop.clone(),
                    ..at(11000)
                };
                let (json, _) = drawn(&fixture_project(), &ask);

                assert_eq!(
                    json["query"]["mode"], "at",
                    "the caption is a `query --at` answer, not a second rendering of one"
                );
                assert_eq!(json["query"]["at"], 11000);
                assert!(
                    !json["query"]["stack"]
                        .as_array()
                        .expect("a stack")
                        .is_empty(),
                    "the fixture has 13 elements at 11000 ms"
                );

                // And in the prose form, at both verbosities — the block has no
                // informational findings to collapse, so neither hides it.
                for verbose in [false, true] {
                    let text = montaget_core::wire::render_frame(
                        &frame(&fixture_project(), &ask),
                        montaget_core::Wire::Text { verbose },
                    );
                    assert!(
                        text.contains("\nQUERY  the resolved stack at 11000"),
                        "{text}"
                    );
                    assert!(text.contains("\nFRAME  at 11000"), "{text}");
                }
            }
        }
    }
}

#[test]
fn the_caption_is_the_same_answer_query_at_gives_on_its_own() {
    // Not merely "a block is present": the *same* block. Two renderings of one moment that
    // could drift is exactly the failure the unconditional caption exists to prevent.
    let (json, _) = drawn(&fixture_project(), &at(11000));
    let queried = montaget_core::verbs::query::query(
        &fixture_project(),
        &montaget_core::verbs::query::Ask {
            at: Some(11000),
            ..Default::default()
        },
    );

    assert_eq!(json["query"], queried.to_json()["query"]);
}

#[test]
fn the_picture_is_painted_in_the_captions_own_order() {
    // The caption is painter's order (ADR-0060 via `crate::stack`), and the picture is the
    // same list: a second sort would be a second place draw order is decided, and a defect
    // attributed to the wrong element is the one failure the caption exists to prevent.
    let (json, _) = drawn(&fixture_project(), &at(11000));

    let painted: Vec<&str> = json["frame"]["painted"]
        .as_array()
        .expect("a painted list")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let caption: Vec<&str> = json["query"]["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .filter_map(|element| element["id"].as_str())
        .filter(|id| painted.contains(id))
        .collect();

    assert_eq!(painted, caption);
    assert_eq!(
        painted.first(),
        Some(&"photo-05"),
        "back to front: the photo is behind everything"
    );
}

#[test]
fn an_element_this_build_cannot_draw_is_named_rather_than_silently_missing() {
    // An agent that cannot tell "the element is not there" from "this build does not draw
    // that yet" goes looking for a defect in the document.
    let (json, _) = drawn(&fixture_project(), &at(11000));

    let deferred: Vec<&str> = json["frame"]["not_painted"]
        .as_array()
        .expect("a not_painted list")
        .iter()
        .filter_map(|entry| entry["element"].as_str())
        .collect();
    assert!(deferred.contains(&"sentence-05"), "{deferred:?}");
    assert!(
        json["frame"]["not_painted"]
            .as_array()
            .expect("a list")
            .iter()
            .all(|entry| entry["reason"].as_str().is_some_and(|r| !r.is_empty())),
        "every entry states why"
    );

    // An element that *was* painted, but not in full, is a different report from one that
    // was not painted at all — `handle-logo` carries the fixture's `mask` effect (#214).
    let partial: Vec<&str> = json["frame"]["painted_partially"]
        .as_array()
        .expect("a painted_partially list")
        .iter()
        .filter_map(|entry| entry["element"].as_str())
        .collect();
    assert_eq!(partial, ["handle-logo"]);
    assert!(
        json["frame"]["painted"]
            .as_array()
            .expect("a list")
            .iter()
            .any(|id| id == "handle-logo"),
        "an element painted without its effects is still painted"
    );
}

#[test]
fn the_bytes_are_written_where_the_caller_asked_for_them() {
    let dir = tempdir(line!());
    let out = dir.join("look.jpg");
    let answer = frame(
        &fixture_project(),
        &Ask {
            out: Some(out.clone()),
            ..at(11000)
        },
    );

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    let written = std::fs::read(&out).expect("the picture was written");
    assert_eq!(written, answer.image().expect("a picture").bytes);
    assert_eq!(answer.to_json()["frame"]["bytes"], written.len());
    assert_eq!(answer.to_json()["frame"]["path"], out.display().to_string());
}

// ---------------------------------------------------------------------------
// `fit` never executes (ADR-0015)
// ---------------------------------------------------------------------------

#[test]
fn fit_is_never_read_by_the_renderer() {
    // ADR-0015's load-bearing sentence: the declared rect is authoritative at render, so
    // `fit` "never executes" and no renderer reads it. Asserted by rendering the same
    // element three ways — `cover`, `contain`, and the field absent entirely — and
    // comparing the bytes. A renderer that read the field would have to produce a
    // different picture for at least one of them, since the three make different claims
    // about how the rect was derived.
    let dir = tempdir(line!());
    let source = fixture_dir().join("images/05.png");
    let element = |fit: &str| {
        format!(
            r##"{{"id":"photo","type":"image","start":0,"end":1000,
                "source":"{}","x":0,"y":0,"origin":"top-left","width":300,"height":200{fit},
                "clip":[20,20,200,150]}}"##,
            source.display().to_string().replace('\\', "\\\\")
        )
    };

    let mut rendered = Vec::new();
    for (name, fit) in [
        ("cover", r##","fit":"cover""##),
        ("contain", r##","fit":"contain""##),
        ("absent", ""),
    ] {
        let project = write_project(
            &dir,
            &format!("{name}.montaget.json"),
            &one_track(&element(fit)),
        );
        let (_, bytes) = drawn(
            &project,
            &Ask {
                full: true,
                png: true,
                ..at(500)
            },
        );
        rendered.push((name, bytes));
    }

    assert_eq!(
        rendered[0].1, rendered[2].1,
        "`fit: cover` and no `fit` at all must be the same picture"
    );
    assert_eq!(
        rendered[1].1, rendered[2].1,
        "`fit: contain` and no `fit` at all must be the same picture"
    );
}

#[test]
fn a_source_is_resampled_to_exactly_the_declared_rect() {
    // The other half of the same rule (ADR-0013): the declared `width`x`height` is what the
    // source is resampled to, whatever the source's own dimensions are. A 400x400 source
    // drawn into a 100x300 box fills the box and nothing outside it.
    let dir = tempdir(line!());
    let source = dir.join("square.png");
    image::RgbaImage::from_pixel(400, 400, image::Rgba([0xFF, 0x00, 0x00, 0xFF]))
        .save(&source)
        .expect("a red square source");
    let project = write_project(
        &dir,
        "p.montaget.json",
        &one_track(
            r##"{"id":"photo","type":"image","start":0,"end":1000,"source":"square.png",
                "x":50,"y":25,"origin":"top-left","width":100,"height":300,"fit":"literal"}"##,
        ),
    );
    let _ = &source;

    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(500)
        },
    );
    let picture = pixels(&bytes);

    assert_eq!(rgb(&picture, 51, 26), RED, "the box's top-left corner");
    assert_eq!(rgb(&picture, 148, 322), RED, "and its bottom-right");
    assert_eq!(
        rgb(&picture, 151, 200),
        BLACK,
        "one pixel past the right edge"
    );
    assert_eq!(rgb(&picture, 100, 326), BLACK, "one pixel past the bottom");
}

#[test]
fn a_stills_exif_orientation_is_applied_before_it_is_painted() {
    // ADR-0015 defines source dimensions as "decoded, **orientation-applied** integer pixel
    // dimensions" and ADR-0023 generalises that to one pipeline, so `validate`'s `fit`
    // arithmetic is already done against the oriented frame. A renderer that painted the
    // stored pixels instead would put the format in the state ADR-0023 names as the thing
    // to avoid — "the format disagrees with itself" — with `validate` green on a frame that
    // is visibly wrong.
    //
    // The fixture is all-PNG and carries no orientation tag anywhere, which is exactly why
    // this needs a written one: the committed project cannot exercise it (ADR-0003).
    let dir = tempdir(line!());
    let quadrants = quadrant_source();
    std::fs::write(dir.join("upright.jpg"), &quadrants).expect("a source with no EXIF");
    std::fs::write(dir.join("turned.jpg"), with_exif_orientation(&quadrants, 6))
        .expect("the same pixels, tagged to be shown rotated 90 degrees clockwise");

    // The same element twice, differing only in which file it names.
    let corner = |file: &str| {
        let project = write_project(
            &dir,
            &format!("{file}.montaget.json"),
            &one_track(&format!(
                r##"{{"id":"photo","type":"image","start":0,"end":1000,"source":"{file}.jpg",
                    "x":0,"y":0,"origin":"top-left","width":200,"height":200,
                    "fit":"literal"}}"##
            )),
        );
        let (_, bytes) = drawn(
            &project,
            &Ask {
                full: true,
                png: true,
                ..at(500)
            },
        );
        rgb(&pixels(&bytes), 50, 50)
    };

    // Channel dominance rather than an exact colour: the source is JPEG, and what is being
    // asserted is *which quadrant landed here*, not what the encoder did to it.
    let upright = corner("upright");
    assert!(
        upright[0] > upright[2],
        "untagged, the stored top-left quadrant is red: {upright:?}"
    );

    // EXIF orientation 6 means "rotate 90 degrees clockwise to display", so the stored
    // bottom-left quadrant — the blue one — is what a viewer sees at the top left.
    let turned = corner("turned");
    assert!(
        turned[2] > turned[0],
        "tagged `Orientation: 6`, the stored bottom-left quadrant is: {turned:?}"
    );
}

/// A 64x64 JPEG in four saturated quadrants: red, green, blue, white, clockwise from the
/// top left.
fn quadrant_source() -> Vec<u8> {
    let mut source = image::RgbaImage::new(64, 64);
    for (x, y, pixel) in source.enumerate_pixels_mut() {
        *pixel = match (x < 32, y < 32) {
            (true, true) => image::Rgba([0xFF, 0x00, 0x00, 0xFF]),
            (false, true) => image::Rgba([0x00, 0xFF, 0x00, 0xFF]),
            (true, false) => image::Rgba([0x00, 0x00, 0xFF, 0xFF]),
            (false, false) => image::Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
        };
    }
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(source)
        .to_rgb8()
        .write_to(&mut bytes, image::ImageFormat::Jpeg)
        .expect("encode a JPEG");
    bytes.into_inner()
}

/// The same JPEG with an EXIF `Orientation` tag spliced in after its start-of-image marker.
///
/// Written by hand rather than through a crate, because the tag is 26 bytes of TIFF and a
/// dependency whose whole job was to write them would be a second thing to keep pinned. The
/// layout: `FFE1`, the segment length, `Exif  `, then a little-endian TIFF header whose
/// only IFD entry is tag `0x0112` (`Orientation`), type SHORT, one value.
fn with_exif_orientation(jpeg: &[u8], orientation: u16) -> Vec<u8> {
    assert_eq!(&jpeg[..2], &[0xFF, 0xD8], "a JPEG starts with SOI");

    let mut tiff: Vec<u8> = Vec::new();
    tiff.extend_from_slice(b"II"); // little-endian
    tiff.extend_from_slice(&42u16.to_le_bytes()); // the TIFF magic
    tiff.extend_from_slice(&8u32.to_le_bytes()); // offset of IFD0, from here
    tiff.extend_from_slice(&1u16.to_le_bytes()); // one entry
    tiff.extend_from_slice(&0x0112u16.to_le_bytes()); // Orientation
    tiff.extend_from_slice(&3u16.to_le_bytes()); // SHORT
    tiff.extend_from_slice(&1u32.to_le_bytes()); // one value
    tiff.extend_from_slice(&orientation.to_le_bytes());
    tiff.extend_from_slice(&[0, 0]); // the value field is four bytes wide
    tiff.extend_from_slice(&0u32.to_le_bytes()); // no next IFD

    let mut segment: Vec<u8> = b"Exif\0\0".to_vec();
    segment.extend_from_slice(&tiff);
    // The length field counts itself, which is the part that is easy to get wrong.
    let length = (segment.len() + 2) as u16;

    let mut out: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE1];
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(&segment);
    out.extend_from_slice(&jpeg[2..]);
    out
}

// ---------------------------------------------------------------------------
// The two shapes (ADR-0014)
// ---------------------------------------------------------------------------

#[test]
fn a_rect_fills_exactly_its_declared_rect() {
    let picture = painted(
        line!(),
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":48,"y":100,
            "origin":"top-left","width":200,"height":80,"fill":"#FF0000"}"##,
    );

    assert_eq!(
        rgb(&picture, 48, 100),
        RED,
        "the declared corner is painted"
    );
    assert_eq!(rgb(&picture, 247, 179), RED, "and the last pixel inside it");
    assert_eq!(rgb(&picture, 47, 100), BLACK, "one pixel left of it is not");
    assert_eq!(rgb(&picture, 248, 100), BLACK, "nor one pixel right of it");
}

#[test]
fn a_shapes_stroke_falls_inside_its_declared_rect() {
    // ADR-0014's worked number: `card-05` with `stroke_width: 8` still occupies exactly
    // 984x169 at (48,1453) — "centred or outside, it would occupy 992x177 at (44,1449),
    // eating 4 px of the 48 px margin the layout rests on".
    let picture = painted(
        line!(),
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "origin":"top-left","width":200,"height":100,"fill":"#FF0000",
            "stroke":"#0000FF","stroke_width":8}"##,
    );

    assert_eq!(
        rgb(&picture, 99, 150),
        BLACK,
        "nothing is painted outside the rect"
    );
    assert_eq!(
        rgb(&picture, 101, 150),
        BLUE,
        "the stroke starts at the declared edge"
    );
    assert_eq!(rgb(&picture, 106, 150), BLUE, "and is 8 px of it");
    assert_eq!(
        rgb(&picture, 112, 150),
        RED,
        "past which the fill takes over"
    );
    assert_eq!(
        rgb(&picture, 298, 150),
        BLUE,
        "and the far edge is stroke again"
    );
    assert_eq!(
        rgb(&picture, 300, 150),
        BLACK,
        "up to the declared edge and no further"
    );
}

#[test]
fn a_stroke_scales_with_scale_because_it_is_in_element_space() {
    // ADR-0014 ships the worked number with the rule: `stroke_width: 8` under the fixture's
    // Ken Burns ramp is 8.64 px at the 1.08 peak. Here the factor is a round 2, so a
    // stroke that is 8 px in element space is 16 px on the frame — and the assertion is
    // that the pixel at 8 px in from the edge, which is stroke at scale 1, is now still
    // stroke rather than fill.
    let stroked = |scale: &str| {
        format!(
            r##"{{"id":"card","type":"rect","start":0,"end":1000,"x":0,"y":0,
                "origin":"top-left","width":100,"height":100,"fill":"#FF0000",
                "stroke":"#0000FF","stroke_width":8{scale}}}"##
        )
    };

    let unscaled = painted(line!(), &stroked(""));
    assert_eq!(rgb(&unscaled, 50, 4), BLUE);
    assert_eq!(rgb(&unscaled, 50, 12), RED, "8 px in is already fill");

    let scaled = painted(line!(), &stroked(r##","scale":[2.0,2.0]"##));
    assert_eq!(rgb(&scaled, 50, 12), BLUE, "at 2x the same stroke is 16 px");
    assert_eq!(rgb(&scaled, 50, 20), RED, "and the fill starts at 16");
}

#[test]
fn a_rects_radius_rounds_its_corners_and_nothing_else() {
    let square = painted(
        line!(),
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "origin":"top-left","width":200,"height":200,"fill":"#FF0000"}"##,
    );
    let rounded = painted(
        line!(),
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "origin":"top-left","width":200,"height":200,"fill":"#FF0000","radius":40}"##,
    );

    assert_eq!(
        rgb(&square, 101, 101),
        RED,
        "a square rect reaches its corner"
    );
    assert_eq!(rgb(&rounded, 101, 101), BLACK, "a rounded one does not");
    // The pair, not just the positive: a radius that ate the straight edges too would be a
    // different shape from the one the document declares.
    assert_eq!(
        rgb(&rounded, 200, 101),
        RED,
        "the top edge is still straight"
    );
    assert_eq!(rgb(&rounded, 101, 200), RED, "and so is the left edge");
}

#[test]
fn an_ellipse_inscribes_its_declared_rect() {
    let picture = painted(
        line!(),
        r##"{"id":"badge","type":"ellipse","start":0,"end":1000,"x":100,"y":100,
            "origin":"top-left","width":200,"height":100,"fill":"#FF0000"}"##,
    );

    assert_eq!(rgb(&picture, 200, 150), RED, "the centre");
    assert_eq!(
        rgb(&picture, 102, 150),
        RED,
        "the left extreme of the inscribed oval"
    );
    assert_eq!(rgb(&picture, 200, 102), RED, "and its top extreme");
    assert_eq!(rgb(&picture, 105, 105), BLACK, "but not the rect's corner");
    assert_eq!(rgb(&picture, 295, 195), BLACK, "nor the opposite one");
}

#[test]
fn a_shape_with_neither_fill_nor_stroke_is_named_rather_than_invisible() {
    // ADR-0014: a shape with neither is a schema error naming both, "because an element
    // that deliberately renders nothing and an element that forgot its paint must not look
    // alike". `frame` runs no checks, so it says the same thing in the one place it can.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &one_track(
            r##"{"id":"ghost","type":"rect","start":0,"end":1000,"x":0,"y":0,
                "origin":"top-left","width":100,"height":100}"##,
        ),
    );
    let (json, _) = drawn(&project, &at(500));

    assert!(
        json["frame"]["painted"]
            .as_array()
            .expect("a list")
            .is_empty()
    );
    let reason = json["frame"]["not_painted"][0]["reason"]
        .as_str()
        .expect("a reason");
    assert!(reason.contains("fill"), "{reason}");
    assert!(reason.contains("stroke"), "{reason}");
}

// ---------------------------------------------------------------------------
// The transform (ADR-0012, ADR-0013)
// ---------------------------------------------------------------------------

#[test]
fn all_nine_origin_keywords_place_the_box_about_their_own_point() {
    // ADR-0013 spelled the nine because six of them had no spelling at all. Each is
    // asserted by the coordinate the document states: a 100x100 box placed at (200,200)
    // has its named point there, so the box's top-left is 200 minus the origin's fraction
    // of 100 — a number the test states rather than recomputes from the renderer's rule.
    for (origin, left, top) in [
        ("top-left", 200, 200),
        ("top-center", 150, 200),
        ("top-right", 100, 200),
        ("center-left", 200, 150),
        ("center", 150, 150),
        ("center-right", 100, 150),
        ("bottom-left", 200, 100),
        ("bottom-center", 150, 100),
        ("bottom-right", 100, 100),
    ] {
        let picture = painted(
            line!(),
            &format!(
                r##"{{"id":"box","type":"rect","start":0,"end":1000,"x":200,"y":200,
                    "origin":"{origin}","width":100,"height":100,"fill":"#FF0000"}}"##
            ),
        );

        assert_eq!(
            rgb(&picture, left as u32 + 1, top as u32 + 1),
            RED,
            "`{origin}` puts the box's top-left at ({left},{top})"
        );
        assert_eq!(
            rgb(&picture, left as u32 - 1, top as u32 - 1),
            BLACK,
            "`{origin}`: and nothing is painted outside it"
        );
        assert_eq!(
            rgb(&picture, left as u32 + 99, top as u32 + 99),
            RED,
            "`{origin}`: the box is still 100x100"
        );
    }
}

#[test]
fn x_and_y_default_to_the_frames_centre() {
    // ADR-0012 chose centre over top-left deliberately: "an element that lands at (0,0)
    // under the header chrome looks *intentional*, and a plausible default camouflages a
    // missing field."
    let picture = painted(
        line!(),
        r##"{"id":"box","type":"rect","start":0,"end":1000,
            "width":100,"height":100,"fill":"#FF0000"}"##,
    );

    assert_eq!(rgb(&picture, 200, 200), RED, "the frame's centre");
    assert_eq!(rgb(&picture, 151, 151), RED, "the box's top-left corner");
    assert_eq!(rgb(&picture, 149, 149), BLACK, "one pixel outside it");
}

#[test]
fn scale_grows_the_box_about_its_origin_rather_than_about_the_frame() {
    let picture = painted(
        line!(),
        r##"{"id":"box","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"top-left","width":100,"height":100,"fill":"#FF0000",
            "scale":[2.0,1.0]}"##,
    );

    // `top-left` pins (200,200): the box grows right, not both ways, and only on x.
    assert_eq!(rgb(&picture, 201, 201), RED);
    assert_eq!(rgb(&picture, 398, 250), RED, "200 px wide now");
    assert_eq!(rgb(&picture, 250, 298), RED, "and still 100 px tall");
    assert_eq!(rgb(&picture, 250, 302), BLACK);
    assert_eq!(rgb(&picture, 198, 250), BLACK, "nothing grew leftward");
}

#[test]
fn rotation_pivots_about_the_origin_and_is_never_normalised() {
    // ADR-0013: `origin` is the point "about which transforms pivot". A 100x100 box at
    // (200,200) with `origin: "center"` rotated 45 degrees keeps its centre and reaches
    // further along the axes than its unrotated corners did.
    let upright = painted(
        line!(),
        r##"{"id":"box","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":100,"height":100,"fill":"#FF0000"}"##,
    );
    let turned = painted(
        line!(),
        r##"{"id":"box","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":100,"height":100,"fill":"#FF0000","rotation":45.0}"##,
    );

    assert_eq!(
        rgb(&turned, 200, 200),
        RED,
        "the pivot is still inside the shape"
    );
    assert_eq!(
        rgb(&upright, 155, 155),
        RED,
        "upright, the corner region is filled"
    );
    assert_eq!(
        rgb(&turned, 155, 155),
        BLACK,
        "turned 45 degrees, it is not"
    );
    assert_eq!(
        rgb(&turned, 200, 132),
        RED,
        "and the diagonal now reaches past where the edge was"
    );

    // "1080 is three turns, and a writer that wraps it silently renders one third of the
    // motion" — 405 degrees is 45 degrees plus a turn, and must paint the same picture.
    let wrapped = painted(
        line!(),
        r##"{"id":"box","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":100,"height":100,"fill":"#FF0000","rotation":405.0}"##,
    );
    assert_eq!(wrapped.into_raw(), turned.into_raw());
}

#[test]
fn opacity_composites_the_whole_element_once() {
    // A fill under an inside stroke overlaps itself. Multiplying alpha into both paints
    // would blend the overlap twice and darken the stroke's inner edge, so the element is
    // composited as one layer — asserted where the two paints meet.
    let picture = painted(
        line!(),
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "origin":"top-left","width":200,"height":100,"fill":"#FF0000",
            "stroke":"#0000FF","stroke_width":8,"opacity":0.5}"##,
    );

    let stroke = rgb(&picture, 103, 150);
    let fill = rgb(&picture, 200, 150);
    // Half of #FF0000 over black, allowing for the encoder's rounding.
    assert!(
        (fill[0] as i32 - 128).abs() <= 2 && fill[1] == 0 && fill[2] == 0,
        "the fill is half-opaque red: {fill:?}"
    );
    assert!(
        (stroke[2] as i32 - 128).abs() <= 2 && stroke[0] == 0,
        "and the stroke is half-opaque blue, not a double-blended one: {stroke:?}"
    );
}

#[test]
fn an_element_at_zero_opacity_paints_nothing() {
    let picture = painted(
        line!(),
        r##"{"id":"box","type":"rect","start":0,"end":1000,"x":0,"y":0,
            "origin":"top-left","width":400,"height":400,"fill":"#FF0000","opacity":0.0}"##,
    );
    assert_eq!(rgb(&picture, 200, 200), BLACK);
}

#[test]
fn an_animated_property_is_resolved_at_the_instant_asked_for() {
    // The same resolver the caption reports through (`crate::resolve`): at the midpoint of
    // a linear ramp from 0 to 200, the box's left edge is at 100.
    let element = r##"{"id":"box","type":"rect","start":0,"end":1000,
        "x":[{"t":0,"v":0},{"t":1000,"v":200,"ease":"linear"}],"y":0,
        "origin":"top-left","width":50,"height":50,"fill":"#FF0000"}"##;
    let dir = tempdir(line!());
    let project = write_project(&dir, "p.montaget.json", &one_track(element));

    let (json, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(500)
        },
    );
    let picture = pixels(&bytes);

    assert_eq!(rgb(&picture, 101, 10), RED, "half way along the ramp");
    assert_eq!(rgb(&picture, 99, 10), BLACK);
    // And the caption states the same number the picture was drawn from.
    let resolved = &json["query"]["stack"][0]["values"];
    assert_eq!(resolved[0]["property"], "x");
    // The resolver interpolates a declared-integer property as a continuous value and the
    // view reports what it resolved, so the caption reads `100.0` where the picture put the
    // edge at 100 — the same number, not two.
    assert_eq!(resolved[0]["value"], 100.0);
}

// ---------------------------------------------------------------------------
// `clip` (ADR-0025)
// ---------------------------------------------------------------------------

#[test]
fn clip_is_a_frame_space_aperture_and_does_not_move_with_the_element() {
    // ADR-0025 settled that `clip` never animates: it is "simultaneously the aperture a
    // source is drawn through and the fixed denominator ADR-0015's fit check compares a
    // declared extent against". Here the element is scaled 2x and the aperture stays put.
    let dir = tempdir(line!());
    let source = dir.join("red.png");
    image::RgbaImage::from_pixel(64, 64, image::Rgba([0xFF, 0x00, 0x00, 0xFF]))
        .save(&source)
        .expect("a red source");
    let project = write_project(
        &dir,
        "p.montaget.json",
        &one_track(
            r##"{"id":"photo","type":"image","start":0,"end":1000,"source":"red.png",
                "x":0,"y":0,"origin":"top-left","width":200,"height":200,"fit":"literal",
                "clip":[50,50,100,100],"scale":[2.0,2.0]}"##,
        ),
    );

    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(500)
        },
    );
    let picture = pixels(&bytes);

    assert_eq!(rgb(&picture, 100, 100), RED, "inside the aperture");
    assert_eq!(
        rgb(&picture, 49, 100),
        BLACK,
        "left of it, though the element reaches there"
    );
    assert_eq!(rgb(&picture, 151, 100), BLACK, "and right of it");
    assert_eq!(rgb(&picture, 100, 151), BLACK, "and below it");
}

// ---------------------------------------------------------------------------
// The font chain (ADR-0007)
//
// The decoy-font claim, and "a project of stills alone needs no ffmpeg", are in
// `frame_environment.rs`: both change a process-wide environment variable, and the tests
// in one binary are threads of one process.
// ---------------------------------------------------------------------------

#[test]
fn a_font_chain_that_does_not_resolve_is_named_beside_the_picture() {
    // The other half of the pair: the chain is genuinely consulted, so a project naming a
    // file that is not there says so rather than rendering as though nothing was asked.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &canonical(
            r##"{"frame":{"width":400,"height":400},"fps":25,"background":"#000000",
                "fonts":{"brand":[{"file":"fonts/not-here.otf"}]},
                "tracks":[{"name":"only","layer":0,"elements":[
                  {"id":"title","type":"text","start":0,"end":1000,"x":10,"y":10,
                   "origin":"top-left","width":300,"height":80,"font":"brand","size":40,
                   "runs":[{"text":"hello"}]}]}]}"##,
        ),
    );

    let (json, _) = drawn(&project, &at(500));

    assert!(
        json["frame"]["fonts"]
            .as_array()
            .expect("a list")
            .is_empty()
    );
    let reasons: Vec<&str> = json["frame"]["not_painted"]
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(|entry| entry["reason"].as_str())
        .collect();
    assert!(
        reasons.iter().any(|reason| reason.contains("not-here.otf")),
        "{reasons:?}"
    );
}

// ---------------------------------------------------------------------------
// The decode path, on its own fixture (#212, ADR-0003)
// ---------------------------------------------------------------------------

#[test]
fn a_video_element_decodes_seeks_and_resamples_into_its_declared_rect() {
    // The committed fixture has zero `video` elements and ADR-0003 forbids reading that
    // silence as evidence the path is unneeded, so the decode path has a fixture of its own
    // (`fixtures/video-decode/`).
    if !has_ffprobe() {
        return;
    }
    let (json, bytes) = drawn(
        &video_project(),
        &Ask {
            full: true,
            png: true,
            ..at(1000)
        },
    );

    let painted: Vec<&str> = json["frame"]["painted"]
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(painted.contains(&"whole-frame"), "{painted:?}");
    assert!(
        json["frame"]["sources"]
            .as_array()
            .expect("a list")
            .iter()
            .any(|source| source.as_str().is_some_and(|s| s.ends_with(".mp4"))),
        "the decoded source is reported like any other file opened"
    );

    // A decoded frame, not a blank one: the source is a real video and no 1080x1920 region
    // of it is one flat colour.
    let picture = pixels(&bytes);
    assert_eq!(picture.dimensions(), (1080, 1920));
    let first = rgb(&picture, 0, 0);
    assert!(
        (0..1080)
            .step_by(37)
            .any(|x| rgb(&picture, x, 900) != first),
        "the frame is a decoded picture rather than one flat colour"
    );
}

#[test]
fn the_seek_lands_on_a_different_frame_at_a_different_instant() {
    // The claim a single decode cannot make: that the offset into the source is actually
    // used. Two instants 2000 ms apart in the same element must be two different pictures.
    if !has_ffprobe() {
        return;
    }
    let ask = |instant| Ask {
        crop: Some("0,0,1080,1080".into()),
        full: true,
        png: true,
        ..at(instant)
    };
    let (_, early) = drawn(&video_project(), &ask(500));
    let (_, late) = drawn(&video_project(), &ask(2500));

    assert_ne!(early, late, "the seek moved with the instant");
}

#[test]
fn a_seek_past_the_end_of_the_source_holds_the_last_frame() {
    // `slowed-and-held` plays a 1000 ms source span over 4000 ms at `speed: 0.5`, so it
    // runs out half way through and `overrun: "hold"` must resolve every later instant to
    // `source_end` (ADR-0020) rather than to a failed seek.
    if !has_ffprobe() {
        return;
    }
    for instant in [10_500, 11_900] {
        let (json, _) = drawn(&video_project(), &at(instant));
        let painted: Vec<&str> = json["frame"]["painted"]
            .as_array()
            .expect("a list")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert!(
            painted.contains(&"slowed-and-held"),
            "at {instant} ms the held frame must still paint: {json}"
        );
    }
}

// ---------------------------------------------------------------------------
// The document, when it is wrong
// ---------------------------------------------------------------------------

#[test]
fn a_project_that_does_not_parse_is_exit_2_and_draws_nothing() {
    let dir = tempdir(line!());
    let project = write_project(&dir, "p.montaget.json", "{not json");
    let answer = frame(&project, &at(0));

    assert_eq!(answer.report().exit_code(), ExitCode::Unparseable);
    assert_eq!(answer.report().findings[0].code, "E-PARSE");
    assert!(answer.image().is_none());
}

#[test]
fn a_document_full_of_errors_still_draws_what_it_can() {
    // `frame` is the verb an agent reaches for precisely when the document is wrong, so it
    // runs no checks and refuses on no finding: a project carrying an element `validate`
    // would reject still comes back as a picture plus a caption naming what did not paint.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &one_track(
            r##"{"id":"good","type":"rect","start":0,"end":1000,"x":0,"y":0,
                "origin":"top-left","width":100,"height":100,"fill":"#FF0000"},
               {"id":"bad","type":"rect","start":0,"end":1000,"x":0,"y":0,
                "origin":"top-left","width":0,"height":0,"fill":"#00FF00"}"##,
        ),
    );

    let (json, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(500)
        },
    );

    assert_eq!(
        json["frame"]["painted"].as_array().expect("a list"),
        &vec![Value::String("good".into())]
    );
    assert_eq!(json["frame"]["not_painted"][0]["element"], "bad");
    assert_eq!(rgb(&pixels(&bytes), 50, 50), RED);
}

#[test]
fn a_project_with_no_background_is_painted_on_opaque_black() {
    // No ADR states a background default. Opaque black is this ticket's choice and the
    // reason is that `frame`'s default encoding carries no alpha: a transparent default
    // would reach the agent as black anyway, while making the PNG and the JPEG of one
    // instant two different pictures. Asserted so the choice cannot drift silently.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &canonical(
            r##"{"frame":{"width":100,"height":100},"fps":25,
                "tracks":[{"name":"only","layer":0,"elements":[]}]}"##,
        ),
    );

    for png in [false, true] {
        let (_, bytes) = drawn(
            &project,
            &Ask {
                full: true,
                png,
                ..at(0)
            },
        );
        let picture = pixels(&bytes);
        assert_eq!(rgb(&picture, 50, 50), BLACK);
        assert_eq!(picture.get_pixel(50, 50).0[3], 0xFF, "and it is opaque");
    }
}

#[test]
fn an_instant_with_nothing_on_it_is_the_background_and_an_empty_caption() {
    let (json, _) = drawn(&fixture_project(), &at(-1));

    assert!(
        json["frame"]["painted"]
            .as_array()
            .expect("a list")
            .is_empty()
    );
    assert!(
        json["query"]["stack"]
            .as_array()
            .expect("a stack")
            .is_empty()
    );
    assert_eq!(
        json["exit_code"], 0,
        "an empty instant is a fact, not a failure"
    );
}
