//! The rest of the paint vocabulary (#214): the ordered `effects` list, the four colour
//! scalars, `crossfade`, and a run's `highlight` window.
//!
//! Asserted at spec #168's **seam 1** — a project file goes to the core verb and the
//! answer comes back — with the pixels read at **seam 2**, through a decoder that is not
//! the one that wrote them. That second rule matters more here than anywhere else in the
//! suite: every claim in this file is a claim about pixels, and an encoder agreeing with
//! its own decoder would prove only that they agree.
//!
//! **No assertion here recomputes the renderer's arithmetic.** An effect order claim is
//! *"these two frames are different pictures"* and *"the shadow falls where the document
//! says"*, never *"this pixel is 0x7A because a sigma of 3 over that kernel says so" —
//! which would pass against a renderer that had the same bug.

use std::path::Path;

use montaget_core::report::ExitCode;
use montaget_core::verbs::frame::{Ask, frame};
use serde_json::Value;

mod common;
use common::{canonical, tempdir, write_project};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

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
fn one_track(elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
            "tracks":[{{"name":"only","layer":0,"elements":[{elements}]}}]}}"##
    ))
}

/// The exact pixels of a small project at true scale and lossless, at 500 ms.
#[track_caller]
fn painted(line: u32, elements: &str) -> image::RgbaImage {
    painted_at(line, elements, 500)
}

#[track_caller]
fn painted_at(line: u32, elements: &str, instant: i64) -> image::RgbaImage {
    let dir = tempdir(line);
    let project = write_project(&dir, "p.montaget.json", &one_track(elements));
    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(instant)
        },
    );
    pixels(&bytes)
}

/// The full answer as well as the pixels, for the tests that read both.
#[track_caller]
fn answered(line: u32, elements: &str, instant: i64) -> (Value, image::RgbaImage) {
    let dir = tempdir(line);
    let project = write_project(&dir, "p.montaget.json", &one_track(elements));
    let (json, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(instant)
        },
    );
    (json, pixels(&bytes))
}

/// A 100×100 white square centred in the 400×400 frame, carrying `effects` verbatim.
fn square_with(effects: &str) -> String {
    format!(
        r##"{{"id":"square","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":100,"height":100,"fill":"#FFFFFF",
            "effects":[{effects}]}}"##
    )
}

/// The frame's own background in every project this file writes.
const BLACK: [u8; 3] = [0x00, 0x00, 0x00];
const RED: [u8; 3] = [0xFF, 0x00, 0x00];
const BLUE: [u8; 3] = [0x00, 0x00, 0xFF];

/// How many pixels of the two pictures differ at all.
fn differing(a: &image::RgbaImage, b: &image::RgbaImage) -> usize {
    a.pixels().zip(b.pixels()).filter(|(a, b)| a != b).count()
}

// ---------------------------------------------------------------------------
// Order is semantically real (ADR-0040)
// ---------------------------------------------------------------------------

#[test]
fn blur_then_shadow_is_a_different_frame_from_shadow_then_blur() {
    // ADR-0040's own sentence, and the reason `effects` is a list rather than a map:
    // "order is semantically real — blur-then-shadow is a different frame from
    // shadow-then-blur".
    //
    // The shadow's own `radius` is 0, which is what makes the two orders legible rather
    // than merely unequal: a Gaussian composes with a Gaussian, so a blur and a *blurred*
    // shadow very nearly commute and the difference between them is a last-bit one no
    // test should be resting on. With a hard-edged shadow the asymmetry is the whole
    // picture — shadowing first tucks a sharp red copy behind a sharp white square, where
    // it is almost entirely hidden before the blur ever reaches it, while blurring first
    // hands the shadow a soft silhouette that reaches out past the square's own edge.
    const BLUR: &str = r##"{"name":"blur","radius":24}"##;
    const SHADOW: &str =
        r##"{"name":"shadow","dx":12,"dy":12,"radius":0,"color":"#FF0000","opacity":1}"##;

    let blur_then_shadow = painted(line!(), &square_with(&format!("{BLUR},{SHADOW}")));
    let shadow_then_blur = painted(line!(), &square_with(&format!("{SHADOW},{BLUR}")));

    assert!(
        differing(&blur_then_shadow, &shadow_then_blur) > 0,
        "the two orders produced the same picture, so the list's order is being ignored"
    );

    // The square spans (150,150)..(250,250) and the shadow is offset by 12, so (260, 200)
    // is past the square's right edge and just past the shadow's — a place only a shadow
    // that was cast from something already soft reaches.
    let (far, near) = (
        rgb(&blur_then_shadow, 260, 200),
        rgb(&shadow_then_blur, 260, 200),
    );
    assert!(
        far[0] > near[0] + 10,
        "blurring first should push more of the red shadow out here, and did not: \
         {far:?} against {near:?}"
    );
    assert_eq!(
        (far[1], far[2]),
        (near[1], near[2]),
        "the two orders differ in more than the shadow, so this is measuring something else"
    );
}

#[test]
fn two_effects_of_the_same_name_are_ordinary_rather_than_forbidden() {
    // ADR-0040: "two effects of the same name (two shadows) are ordinary rather than
    // forbidden" — the container is a list precisely so a second shadow has somewhere to
    // go. Two shadows at opposite offsets put colour on both sides of the square.
    let two = painted(
        line!(),
        &square_with(
            r##"{"name":"shadow","dx":40,"dy":0,"radius":0,"color":"#FF0000","opacity":1},
                {"name":"shadow","dx":-40,"dy":0,"radius":0,"color":"#0000FF","opacity":1}"##,
        ),
    );

    // The square spans (150,150)..(250,250). The first shadow lands 40 px right of it,
    // the second 40 px left — and the second is cast over the *result* of the first, so
    // both are there.
    assert_eq!(rgb(&two, 270, 200), [0xFF, 0x00, 0x00], "the right shadow");
    assert_eq!(rgb(&two, 130, 200), [0x00, 0x00, 0xFF], "the left shadow");
}

// ---------------------------------------------------------------------------
// The four colour scalars, and the absence of a fifth (ADR-0049)
// ---------------------------------------------------------------------------

/// A 100×100 square of `colour`, centred, carrying `effects`.
fn coloured_square_with(colour: &str, effects: &str) -> String {
    format!(
        r##"{{"id":"square","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":100,"height":100,"fill":"{colour}",
            "effects":[{effects}]}}"##
    )
}

/// The middle of the square, which every colour-scalar claim below is about.
#[track_caller]
fn middle(picture: &image::RgbaImage) -> [u8; 3] {
    rgb(picture, 200, 200)
}

#[test]
fn saturation_zero_is_the_grayscale_case_and_one_changes_nothing() {
    // ADR-0049 folded `grayscale` into this member's zero endpoint rather than shipping
    // it: "0 = fully desaturated (equivalent to a bare `grayscale` effect), 1 = unchanged
    // (identity value)".
    const ORANGE: &str = "#FF8A00";

    let grey = middle(&painted(
        line!(),
        &coloured_square_with(ORANGE, r##"{"name":"saturation","amount":0}"##),
    ));
    assert_eq!(
        (grey[0], grey[1]),
        (grey[1], grey[2]),
        "`saturation: 0` left colour in the picture: {grey:?}"
    );

    // The identity value does exactly nothing — which is clause (c) of ADR-0049's own
    // admissibility rule, asserted rather than assumed.
    let untouched = painted(
        line!(),
        &coloured_square_with(ORANGE, r##"{"name":"saturation","amount":1}"##),
    );
    let bare = painted(line!(), &coloured_square_with(ORANGE, ""));
    assert_eq!(
        differing(&untouched, &bare),
        0,
        "`saturation: 1` is meant to be the identity and changed the picture"
    );

    // And above 1 it pushes the other way, which is what makes this one axis rather than
    // a `grayscale` switch.
    let over = middle(&painted(
        line!(),
        &coloured_square_with(ORANGE, r##"{"name":"saturation","amount":2}"##),
    ));
    let plain = middle(&bare);
    assert!(
        over[1] < plain[1],
        "oversaturating an orange should drive its green channel further from the \
         luminance it shares with the grey above: {over:?} against {plain:?}"
    );
}

#[test]
fn brightness_and_contrast_are_signed_offsets_from_unchanged_at_zero() {
    // ADR-0049: both are "signed offset from unchanged at 0". Two claims per member —
    // that zero is the identity, and that the two signs go in opposite directions.
    const GREY: &str = "#808080";

    for member in ["brightness", "contrast"] {
        let identity = painted(
            line!(),
            &coloured_square_with(GREY, &format!(r##"{{"name":"{member}","amount":0}}"##)),
        );
        assert_eq!(
            differing(
                &identity,
                &painted(line!(), &coloured_square_with(GREY, ""))
            ),
            0,
            "`{member}: 0` is meant to be the identity and changed the picture"
        );
    }

    let lighter = middle(&painted(
        line!(),
        &coloured_square_with(GREY, r##"{"name":"brightness","amount":0.25}"##),
    ));
    let darker = middle(&painted(
        line!(),
        &coloured_square_with(GREY, r##"{"name":"brightness","amount":-0.25}"##),
    ));
    let plain = middle(&painted(line!(), &coloured_square_with(GREY, "")));
    assert!(
        lighter[0] > plain[0] && darker[0] < plain[0],
        "brightness did not move both ways: {lighter:?}, {plain:?}, {darker:?}"
    );

    // Contrast pivots on mid-grey, so the member is shown on a pair either side of it:
    // more contrast drives the dark one darker and the light one lighter at once.
    const DARK: &str = "#404040";
    const LIGHT: &str = "#C0C0C0";
    let harder = |colour| {
        middle(&painted(
            line!(),
            &coloured_square_with(colour, r##"{"name":"contrast","amount":0.5}"##),
        ))[0]
    };
    let plain_of = |colour| middle(&painted(line!(), &coloured_square_with(colour, "")))[0];
    assert!(
        harder(DARK) < plain_of(DARK),
        "contrast should drive a dark tone darker"
    );
    assert!(
        harder(LIGHT) > plain_of(LIGHT),
        "contrast should drive a light tone lighter"
    );
}

#[test]
fn tint_pushes_colour_toward_its_own_and_is_the_identity_at_zero() {
    // ADR-0049: "pushes pixel colour toward `color` by `amount` (0-1)". `amount: 0` is the
    // documented identity that makes `color` — the vocabulary's sole non-scalar parameter
    // — admissible at all, so it is the half of this that carries the ADR's argument.
    const WHITE: &str = "#FFFFFF";

    let untouched = painted(
        line!(),
        &coloured_square_with(WHITE, r##"{"name":"tint","color":"#FF0000","amount":0}"##),
    );
    assert_eq!(
        differing(
            &untouched,
            &painted(line!(), &coloured_square_with(WHITE, ""))
        ),
        0,
        "`tint` at `amount: 0` is meant to be the identity and changed the picture"
    );

    let all_the_way = middle(&painted(
        line!(),
        &coloured_square_with(WHITE, r##"{"name":"tint","color":"#FF0000","amount":1}"##),
    ));
    assert_eq!(
        all_the_way,
        [0xFF, 0x00, 0x00],
        "at `amount: 1` the pixel is the tint colour"
    );

    // And halfway is halfway — the claim that this is a push toward the colour rather
    // than a switch onto it.
    let half = middle(&painted(
        line!(),
        &coloured_square_with(WHITE, r##"{"name":"tint","color":"#FF0000","amount":0.5}"##),
    ));
    assert_eq!(half[0], 0xFF);
    assert!(
        (100..=155).contains(&half[1]) && half[1] == half[2],
        "half a tint from white toward red should leave the other two channels near the \
         middle: {half:?}"
    );
}

#[test]
fn there_is_no_fifth_colour_member() {
    // ADR-0049 is explicit: "No `grayscale` or `sepia` member exists or will be added
    // under this rule without a new ADR". The vocabulary is closed at the model, which is
    // the one authority the renderer reads it through — so a document naming a fifth
    // member does not quietly paint as one of the four.
    for refused in [
        r##"{"name":"grayscale","amount":0}"##,
        r##"{"name":"sepia","amount":1}"##,
        r##"{"name":"invert"}"##,
        // ADR-0068: the param-less form is the only `mask` spelling until the explicit
        // geometry vocabulary lands (#185), so geometry parameters are not a member
        // either.
        r##"{"name":"mask","shape":"circle","radius":20}"##,
    ] {
        let parsed: Result<montaget_core::model::Effect, _> = serde_json::from_str(refused);
        assert!(
            parsed.is_err(),
            "the effect vocabulary admitted {refused}, which no ADR does"
        );
    }

    // All seven that do exist parse, so the assertion above is about the vocabulary rather
    // than about a deserializer that refuses everything.
    for admitted in [
        r##"{"name":"blur","radius":4}"##,
        r##"{"name":"shadow","dx":1,"dy":1,"radius":2,"color":"#000000","opacity":0.5}"##,
        r##"{"name":"mask","shape":"circle"}"##,
        r##"{"name":"tint","color":"#FF8A00","amount":0.4}"##,
        r##"{"name":"saturation","amount":0}"##,
        r##"{"name":"brightness","amount":0.15}"##,
        r##"{"name":"contrast","amount":0.2}"##,
    ] {
        serde_json::from_str::<montaget_core::model::Effect>(admitted)
            .unwrap_or_else(|e| panic!("{admitted} is a v1 member and did not parse: {e}"));
    }
}

#[test]
fn an_effect_the_vocabulary_does_not_admit_is_named_beside_the_picture() {
    // `frame` runs no checks (ADR-0006 gives `render` the enforcement), so a document
    // carrying a member the format does not have still gets a picture. What it must not
    // get is silence: an agent that cannot tell "the blur is subtle" from "the blur was
    // never applied" chases the wrong defect.
    let (json, picture) = answered(
        line!(),
        &square_with(
            r##"{"name":"grayscale","amount":0},{"name":"tint","color":"#FF0000","amount":1}"##,
        ),
        500,
    );

    assert_eq!(
        middle(&picture),
        [0xFF, 0x00, 0x00],
        "the members that *are* admitted still painted"
    );
    let said = json["frame"]["painted_partially"]
        .as_array()
        .map(|rows| {
            rows.iter().any(|row| {
                row["reason"]
                    .as_str()
                    .is_some_and(|r| r.contains("effects[0]"))
            })
        })
        .unwrap_or(false);
    assert!(
        said,
        "nothing beside the picture said the first effect was not painted: {}",
        serde_json::to_string_pretty(&json["frame"]).unwrap_or_default()
    );
}

// ---------------------------------------------------------------------------
// `mask`, param-less, the inscribed shape of the element's own rect (ADR-0068)
// ---------------------------------------------------------------------------

#[test]
fn a_param_less_mask_is_the_inscribed_shape_of_the_elements_own_rect() {
    // ADR-0068 states the geometry ADR-0040 left as an ellipsis: `{"name": "mask",
    // "shape": "circle"}` with no geometry parameters is "the largest circle inscribed in
    // the element's own rect — diameter `min(width, height)`, centred on that rect", and
    // `rect`/`ellipse` "take the element's rect itself under the same rule".
    //
    // The element is deliberately *not* square — 200 wide by 100 tall, centred, so its rect
    // spans (100,150)..(300,250) — because that is the only shape on which `circle` and
    // `ellipse` are different answers, and a square fixture would let one stand in for the
    // other.
    let oblong = |effects: &str| {
        format!(
            r##"{{"id":"oblong","type":"rect","start":0,"end":1000,"x":200,"y":200,
                "origin":"center","width":200,"height":100,"fill":"#FFFFFF",
                "effects":[{effects}]}}"##
        )
    };

    let circle = painted(line!(), &oblong(r##"{"name":"mask","shape":"circle"}"##));
    // Diameter 100, centred: the circle spans x 150..250, y 150..250.
    assert_eq!(rgb(&circle, 200, 200), [0xFF, 0xFF, 0xFF], "its middle");
    assert_eq!(
        rgb(&circle, 120, 200),
        BLACK,
        "a circle's diameter is min(width, height), so the oblong's own ends are outside it"
    );
    assert_eq!(rgb(&circle, 280, 200), BLACK, "and so is the other end");
    assert_eq!(
        rgb(&circle, 200, 155),
        [0xFF, 0xFF, 0xFF],
        "the circle reaches the short axis's own edge"
    );

    let ellipse = painted(line!(), &oblong(r##"{"name":"mask","shape":"ellipse"}"##));
    // The ellipse is inscribed in the *rect*, so it reaches both ends the circle did not.
    assert_eq!(rgb(&ellipse, 200, 200), [0xFF, 0xFF, 0xFF], "its middle");
    assert_eq!(
        rgb(&ellipse, 120, 200),
        [0xFF, 0xFF, 0xFF],
        "an ellipse inscribes the rect, so it reaches the long axis's ends"
    );
    // And it is still an ellipse rather than the rect: its corners are cut.
    assert_eq!(
        rgb(&ellipse, 105, 155),
        BLACK,
        "the rect's corner is not in it"
    );

    let rect = painted(line!(), &oblong(r##"{"name":"mask","shape":"rect"}"##));
    assert_eq!(
        differing(&rect, &painted(line!(), &oblong(""))),
        0,
        "a `rect` mask is the element's own rect, so it selects everything the element \
         already painted"
    );
}

#[test]
fn a_mask_cuts_what_the_effects_before_it_produced_and_not_what_comes_after() {
    // The order rule again, on the one member that is a shape rather than a filter. A
    // blur before the mask is cut off at the mask's edge; the same blur after it softens
    // the mask's own edge and reaches beyond it.
    let inside_first = painted(
        line!(),
        &square_with(r##"{"name":"blur","radius":24},{"name":"mask","shape":"circle"}"##),
    );
    let outside_first = painted(
        line!(),
        &square_with(r##"{"name":"mask","shape":"circle"},{"name":"blur","radius":24}"##),
    );

    // The square spans (150,150)..(250,250), so its inscribed circle has radius 50. A
    // point 8 px outside that circle is reached only by a blur applied after the mask.
    assert_eq!(
        rgb(&inside_first, 258, 200),
        BLACK,
        "the mask was applied after the blur, so nothing may be outside it"
    );
    assert!(
        rgb(&outside_first, 258, 200)[0] > 0,
        "the blur was applied after the mask and should have reached past its edge"
    );
}

#[test]
fn the_fixtures_badge_keeps_every_pixel_the_mask_selects() {
    // ADR-0068 landed the fixture's migration on this claim: "The change is pixel-inert.
    // The asset's own alpha already produces the circle, `clip` equals the element rect,
    // and the slot is square, so the mask selects every pixel the asset already shows."
    //
    // **That claim is very nearly, but not exactly, true, and this test is where the
    // difference is recorded.** The ADR's evidence is `decode_logo_alpha.py`, which
    // measures the 800x800 asset and finds "zero opaque pixels outside the inscribed
    // circle". The renderer does not paint the asset at 800x800: `handle-logo` resamples
    // it into a 68x68 slot, and a bilinear minification by 11.8x carries a little of each
    // boundary texel into the destination pixels just outside the circle. So the *drawn*
    // badge reaches marginally past the circle the *stored* badge does not, and a real
    // mask trims that.
    //
    // What survives, and is what the ADR's sentence is actually about: the mask selects
    // the whole badge. Nothing inside the circle moves by a single bit, every pixel that
    // does move is on the circle's own edge, and the whole-frame consequence sits inside
    // the same band `tests/golden_frames.rs` budgets for a platform's last bit — its
    // committed goldens still pass unchanged, which is the check that would have caught a
    // real change.
    let with_mask = std::fs::read_to_string(common::fixture_project()).expect("the fixture");
    let without_mask = with_mask.replace(r#","effects":[{"name":"mask","shape":"circle"}]"#, "");
    assert_ne!(
        with_mask, without_mask,
        "the fixture no longer carries the badge's `mask` effect, so this asserts nothing"
    );

    // The assets resolve against the project file's own directory (ADR-0053), so the
    // unmasked copy has to sit beside them rather than in a scratch directory of its own.
    let beside = common::fixture_dir().join("no-mask-for-a-test.montaget.json");
    std::fs::write(&beside, &without_mask).expect("a copy beside the fixture's assets");

    // `handle-logo` is a 68x68 slot at (478, 96), which is exactly the crop below — so
    // the comparison is the badge and nothing else. Every other element in the frame
    // would only dilute it.
    let ask = Ask {
        full: true,
        png: true,
        crop: Some("478,96,68,68".into()),
        ..at(400)
    };
    let (_, masked) = drawn(&common::fixture_project(), &ask);
    let (_, bare) = drawn(&beside, &ask);
    let _ = std::fs::remove_file(&beside);
    let (masked, bare) = (pixels(&masked), pixels(&bare));

    // The inscribed circle of the 68x68 slot: centre (34, 34), radius 34.
    //
    // Two bands, because two different things happen in them. Inside the circle the only
    // difference a mask may make is the rounding of one extra premultiplied round trip —
    // the effect pipeline composites the element through a layer of its own — and that is
    // a bit or two, never a visible change. On the circle's own edge the mask's coverage
    // ramp meets the resampled badge's, and the product of two ramps is a real, if
    // sub-pixel, difference.
    const ROUNDING: i32 = 2;
    let mut eaten = Vec::new();
    for y in 0..masked.height() {
        for x in 0..masked.width() {
            let (a, b) = (masked.get_pixel(x, y).0, bare.get_pixel(x, y).0);
            let delta = (0..4)
                .map(|c| i32::from(a[c]).abs_diff(i32::from(b[c])) as i32)
                .max()
                .unwrap_or(0);
            if delta <= ROUNDING {
                continue;
            }
            let (dx, dy) = (f64::from(x) + 0.5 - 34.0, f64::from(y) + 0.5 - 34.0);
            let radius = (dx * dx + dy * dy).sqrt();
            if (radius - 34.0).abs() > 1.0 {
                eaten.push((x, y, radius, delta));
            }
        }
    }
    assert!(
        eaten.is_empty(),
        "the mask changed {} pixels that are not on the circle's own edge, so it is not \
         selecting everything the badge shows: {:?}",
        eaten.len(),
        &eaten[..eaten.len().min(8)]
    );

    // And the badge's interior — well clear of the edge in every direction — is identical
    // bit for bit, which is the half of the ADR's claim that does hold exactly.
    for (x, y) in [(34, 34), (34, 8), (8, 34), (60, 34), (34, 60), (16, 16)] {
        assert_eq!(
            masked.get_pixel(x, y),
            bare.get_pixel(x, y),
            "({x}, {y}) is inside the circle and the mask changed it"
        );
    }
}

// ---------------------------------------------------------------------------
// `crossfade`, over the exact derived window (ADR-0059)
// ---------------------------------------------------------------------------

/// Two full-frame squares on two tracks, bridged by a `crossfade` over their overlap.
///
/// Two tracks rather than one, because ADR-0059 leaves ADR-0004's per-track non-overlap
/// rule untouched: "the two bridged elements must already live on separate tracks, exactly
/// as anything else that needs simultaneous visibility already requires."
fn bridged(from_range: (i64, i64), to_range: (i64, i64), window: (i64, i64)) -> String {
    let square = |id: &str, colour: &str, (start, end): (i64, i64)| {
        format!(
            r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":200,"y":200,
                "origin":"center","width":400,"height":400,"fill":"{colour}"}}"##
        )
    };
    canonical(&format!(
        r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
            "tracks":[
              {{"name":"outgoing","layer":0,"elements":[{}]}},
              {{"name":"incoming","layer":1,"elements":[{}]}},
              {{"name":"bridge","layer":2,"elements":[
                {{"id":"fade","type":"transition","start":{},"end":{},
                  "kind":"crossfade","from":"first","to":"second"}}]}}
            ]}}"##,
        square("first", "#FF0000", from_range),
        square("second", "#0000FF", to_range),
        window.0,
        window.1,
    ))
}

#[track_caller]
fn frame_of(line: u32, project_body: &str, instant: i64) -> image::RgbaImage {
    let dir = tempdir(line);
    let project = write_project(&dir, "p.montaget.json", project_body);
    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(instant)
        },
    );
    pixels(&bytes)
}

#[test]
fn a_crossfade_runs_from_one_element_to_the_other_across_its_window() {
    // ADR-0059: crossfade is the whole of v1's `kind` vocabulary, and its window "must
    // exactly equal the intersection of the two elements it bridges" — here 1000..2000.
    // The outgoing element is red and the incoming one blue, so the mix at any instant is
    // readable straight off one pixel.
    let project = bridged((0, 2000), (1000, 3000), (1000, 2000));

    // Before the window: the outgoing element alone, at full strength.
    assert_eq!(rgb(&frame_of(line!(), &project, 500), 200, 200), RED);
    // After it: the incoming element alone. (Both are present between 1000 and 2000 only.)
    assert_eq!(rgb(&frame_of(line!(), &project, 2500), 200, 200), BLUE);

    // At the window's start the incoming element has not arrived at all, and at its end
    // the outgoing one has gone — the two endpoints that make this a *cross* fade rather
    // than a dissolve to and from the background.
    assert_eq!(rgb(&frame_of(line!(), &project, 1000), 200, 200), RED);

    // And in the middle, both. Halfway through, each carries about half its own colour —
    // the blue is painted over the red on a higher layer, so what comes back is
    // `red x (1 - p)` seen through `blue x p`.
    let middle = rgb(&frame_of(line!(), &project, 1500), 200, 200);
    assert!(
        middle[0] > 20 && middle[2] > 20,
        "halfway through a crossfade both elements should be in the picture: {middle:?}"
    );

    // Monotone across the window: the outgoing element only ever gets weaker and the
    // incoming one only ever stronger. This is the claim a single midpoint cannot make.
    let mut previous = rgb(&frame_of(line!(), &project, 1000), 200, 200);
    for instant in [1200, 1400, 1600, 1800, 1999] {
        let now = rgb(&frame_of(line!(), &project, instant), 200, 200);
        assert!(
            now[0] <= previous[0] && now[2] >= previous[2],
            "the fade went backwards at {instant}: {now:?} after {previous:?}"
        );
        previous = now;
    }
}

#[test]
fn a_crossfade_is_the_window_the_two_elements_derive_rather_than_the_one_declared() {
    // ADR-0059 makes the declared range a closed-form function of the two bridged
    // elements' own ranges, and ADR-0007's rule decides what a renderer does when a
    // document states one it cannot honour: "a wider declared range is a field the
    // renderer cannot honour — worse than no field". Outside the intersection only one of
    // the two elements exists, so the picture is drawn over the intersection and
    // `validate` is what reports the drift.
    let honest = bridged((0, 2000), (1000, 3000), (1000, 2000));
    let drifted = bridged((0, 2000), (1000, 3000), (0, 3000));

    for instant in [500, 1000, 1500, 1900, 2500] {
        assert_eq!(
            rgb(&frame_of(line!(), &honest, instant), 200, 200),
            rgb(&frame_of(line!(), &drifted, instant), 200, 200),
            "a drifted declared window changed the picture at {instant}"
        );
    }
}

#[test]
fn a_crossfade_is_named_beside_the_picture() {
    // The same rule the rest of this verb follows: an element that changes what the
    // picture shows without drawing anything itself is still owed a sentence, or an agent
    // reading a half-faded element off the caption's `opacity: 1` will go looking for a
    // defect in the wrong place.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montaget.json",
        &bridged((0, 2000), (1000, 3000), (1000, 2000)),
    );
    let (json, _) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(1500)
        },
    );

    let crossfades = json["frame"]["crossfades"]
        .as_array()
        .expect("the answer names the crossfades in effect")
        .clone();
    assert_eq!(crossfades.len(), 1, "one transition, one row");
    let row = &crossfades[0];
    assert_eq!(row["element"], "fade");
    assert_eq!(row["from"], "first");
    assert_eq!(row["to"], "second");
    assert_eq!(row["start"], 1000, "the derived window's start");
    assert_eq!(row["end"], 2000, "and its end");

    // And no row at all outside the window: a transition that is not running is not a
    // thing the picture is doing.
    let (outside, _) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(2500)
        },
    );
    assert_eq!(
        outside["frame"]["crossfades"].as_array().map(Vec::len),
        Some(0)
    );
}

// ---------------------------------------------------------------------------
// `highlight` — a timed window on the run (ADR-0048)
// ---------------------------------------------------------------------------

/// How many pixels of the picture are exactly this colour.
fn count(picture: &image::RgbaImage, colour: [u8; 3]) -> usize {
    picture
        .pixels()
        .filter(|p| [p.0[0], p.0[1], p.0[2]] == colour)
        .count()
}

/// One text element whose second word carries a `highlight` window over `[start, end)`.
///
/// Three runs rather than one, because ADR-0048 makes every highlighted word its own run:
/// "letting one run span several words would need sub-run text offsets, which a later text
/// edit invalidates silently". The words either side are what says the window is addressed
/// at the run rather than at the element.
fn karaoke(start: i64, end: i64) -> String {
    let font = common::with_forward_slashes(
        &common::fixture_dir()
            .join("fonts/OpenRunde-Bold.otf")
            .display()
            .to_string(),
    );
    canonical(&format!(
        r##"{{"frame":{{"width":600,"height":200}},"fps":25,"background":"#000000",
            "fonts":{{"brand":[{{"file":"{font}"}}]}},
            "tracks":[{{"name":"only","layer":0,"elements":[
              {{"id":"line","type":"text","start":0,"end":3000,"x":300,"y":100,
                "origin":"center","width":560,"height":80,"font":"brand","size":56,
                "color":"#FFFFFF","align":"center","runs":[
                  {{"text":"one "}},
                  {{"text":"two","highlight":{{"start":{start},"end":{end},
                    "color":"#FFD34D"}}}},
                  {{"text":" three"}}
                ]}}
            ]}}]}}"##
    ))
}

#[test]
fn a_runs_highlight_paints_its_own_style_inside_the_window_and_the_runs_outside_it() {
    // ADR-0048: "a run gains an optional `highlight` object: a start/end time window
    // (within the element's own range) plus the style delta that applies during it,
    // falling back to the run's unconditional style outside the window."
    const WHITE: [u8; 3] = [0xFF, 0xFF, 0xFF];
    const AMBER: [u8; 3] = [0xFF, 0xD3, 0x4D];

    let project = karaoke(1000, 2000);

    let before = frame_of(line!(), &project, 500);
    assert_eq!(
        count(&before, AMBER),
        0,
        "the highlight painted before its window opened"
    );
    assert!(count(&before, WHITE) > 0, "the line is there at all");

    let during = frame_of(line!(), &project, 1500);
    assert!(
        count(&during, AMBER) > 0,
        "the highlight did not paint inside its own window"
    );
    assert!(
        count(&during, WHITE) > 0,
        "the words either side of the highlighted one lost their own style, so the window \
         is addressed at the element rather than at the run"
    );

    // The window is half-open, exactly like every other range in the format (ADR-0005):
    // the instant it opens is inside it and the instant it closes is not.
    assert!(count(&frame_of(line!(), &project, 1000), AMBER) > 0);
    assert_eq!(count(&frame_of(line!(), &project, 2000), AMBER), 0);

    // And the highlighted word is genuinely restyled rather than added to: the same
    // number of pixels are painted either way, they are just a different colour.
    let after = frame_of(line!(), &project, 2500);
    assert_eq!(
        count(&before, WHITE),
        count(&after, WHITE),
        "the line is not the same line before and after the window"
    );
    assert_eq!(
        count(&during, WHITE) + count(&during, AMBER),
        count(&before, WHITE),
        "the highlighted word moved or changed size, which a paint delta must not do"
    );
}

#[test]
fn a_highlight_moves_no_glyph() {
    // ADR-0048's delta may carry `stroke_width`, which is the one field in it that the
    // *measurement* also reads — `montaget-text` grows a line's extent by `2 x
    // stroke_width` (ADR-0014). If a highlight's stroke_width reached the layout, a word
    // would jump sideways as it lit up, and `measure`'s answer — taken once, at authoring
    // time — would stop describing the frame.
    //
    // So a highlight is paint and only paint: the element is laid out from the runs'
    // unconditional style, and the window changes what is painted into the slots, never
    // where the slots are.
    let font = common::with_forward_slashes(
        &common::fixture_dir()
            .join("fonts/OpenRunde-Bold.otf")
            .display()
            .to_string(),
    );
    let with_stroke = |highlight: &str| {
        canonical(&format!(
            r##"{{"frame":{{"width":600,"height":200}},"fps":25,"background":"#000000",
                "fonts":{{"brand":[{{"file":"{font}"}}]}},
                "tracks":[{{"name":"only","layer":0,"elements":[
                  {{"id":"line","type":"text","start":0,"end":3000,"x":300,"y":100,
                    "origin":"center","width":560,"height":80,"font":"brand","size":56,
                    "color":"#FFFFFF","align":"center","runs":[
                      {{"text":"one "}},
                      {{"text":"two"{highlight}}},
                      {{"text":" three"}}
                    ]}}
                ]}}]}}"##
        ))
    };

    let quiet = with_stroke(
        r##","highlight":{"start":1000,"end":2000,"stroke":"#FF0000","stroke_width":6}"##,
    );
    // The same document with the window shut, so the only difference between the two
    // frames is whether the highlight is in force.
    let plain = with_stroke("");

    let outside = frame_of(line!(), &quiet, 500);
    assert_eq!(
        differing(&outside, &frame_of(line!(), &plain, 500)),
        0,
        "a highlight outside its own window changed the picture"
    );

    // Inside the window the stroke is painted — and the white letterforms are still in
    // exactly the places they were, which is the claim.
    let inside = frame_of(line!(), &quiet, 1500);
    assert!(
        count(&inside, [0xFF, 0x00, 0x00]) > 0,
        "the highlight's stroke did not paint"
    );
    let unstroked = frame_of(line!(), &plain, 1500);
    for (x, y) in [(300u32, 100u32), (200, 100), (400, 100)] {
        let (a, b) = (inside.get_pixel(x, y).0, unstroked.get_pixel(x, y).0);
        if a == b {
            continue;
        }
        // Where they differ it must be the stroke's own colour arriving, never a letter
        // having moved out from under the sample.
        assert_eq!(
            [a[0], a[1], a[2]],
            [0xFF, 0x00, 0x00],
            "({x}, {y}) changed to something that is not the highlight's stroke"
        );
    }
}
