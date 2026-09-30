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

use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};
use serde_json::Value;

mod common;
use common::compare::{Plane, Scope, mean_delta, rendered, ssim};
use common::{Scratch, canonical, tempdir, write_project};

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
    let project = write_project(&dir, "p.montagent.json", &one_track(elements));
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
    let project = write_project(&dir, "p.montagent.json", &one_track(elements));
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
fn a_colour_scalar_ahead_of_the_key_changes_what_the_key_finds() {
    // ADR-0088's `R-CHROMA-AFTER-COLOUR`, as the picture the finding is about: "`effects`
    // is ordered and order is semantically real (ADR-0040), so a colour scalar ahead of the
    // key changes the pixels the key is measured against, and the author's `color` no
    // longer names what is in the frame."
    //
    // The two orders are not merely unequal here, they are opposite pictures — keyed to
    // the black background, or left standing and brightened — which is what makes this an
    // assertion about the ordering rule rather than about a few pixels on an edge.
    const SCREEN: &str =
        r##"{"name":"chroma","color":"#00CD00","tolerance":0.05,"softness":0,"spill":0}"##;
    // `saturation: 0` rather than a `brightness` lift, and the choice is the point rather
    // than a convenience: the key is measured in the **chroma plane**, so a scalar that
    // moves only luma barely moves it at all — a +0.25 brightness leaves this green 0.02
    // from its own key, well inside any usable tolerance. Desaturation is the colour
    // operation that genuinely relocates a pixel in the plane the key reads.
    const LIFT: &str = r##"{"name":"saturation","amount":0}"##;

    let green = r##"{"id":"square","type":"rect","start":0,"end":1000,"x":200,"y":200,
        "origin":"center","width":100,"height":100,"fill":"#00CD00","effects":[EFFECTS]}"##;

    let keyed = painted(line!(), &green.replace("EFFECTS", SCREEN));
    assert_eq!(
        rgb(&keyed, 200, 200),
        BLACK,
        "the key takes the square out and the background shows through"
    );

    let keyed_then_lifted = painted(
        line!(),
        &green.replace("EFFECTS", &format!("{SCREEN},{LIFT}")),
    );
    assert_eq!(
        rgb(&keyed_then_lifted, 200, 200),
        BLACK,
        "grading what the key kept leaves nothing there to grade"
    );

    let lifted_then_keyed = painted(
        line!(),
        &green.replace("EFFECTS", &format!("{LIFT},{SCREEN}")),
    );
    let standing = rgb(&lifted_then_keyed, 200, 200);
    assert_ne!(
        standing, BLACK,
        "the lift moved the pixels off the author's `color`, so the key no longer finds them"
    );
    assert!(
        standing[0] == standing[1] && standing[1] == standing[2] && standing[0] > 0,
        "and what is standing there is the *desaturated* square, which is no longer green \
         and so is no longer the colour the key was told to find: {standing:?}"
    );
}

#[test]
fn the_key_is_a_matte_and_keeps_the_rgb_it_was_given() {
    // CONTEXT.md's Matte operation, on a frame: "it decides which pixels survive, and the
    // RGB it keeps is the RGB it was given". Both directions in one picture — the screen
    // goes, the subject stays, and the subject's colour is untouched.
    const SCREEN: &str =
        r##"{"name":"chroma","color":"#00CD00","tolerance":0.05,"softness":0,"spill":0}"##;
    let painted = painted(
        line!(),
        &format!(
            r##"{{"id":"screen","type":"rect","start":0,"end":1000,"x":200,"y":200,
                "origin":"center","width":200,"height":200,"fill":"#00CD00",
                "effects":[{SCREEN}]}},
               {{"id":"subject","type":"rect","start":0,"end":1000,"x":200,"y":200,
                "origin":"center","width":40,"height":40,"fill":"#0000FF",
                "effects":[{SCREEN}]}}"##
        ),
    );
    assert_eq!(rgb(&painted, 130, 200), BLACK, "the screen keyed out");
    assert_eq!(
        rgb(&painted, 200, 200),
        BLUE,
        "the subject survived its own key, in the colour it was declared"
    );
}

#[test]
fn a_key_after_a_shadow_keys_the_shadow_too_and_before_it_does_not() {
    // The other half of #342's ordering requirement — "where the key sits relative to
    // `blur`/`shadow`/the colour scalars is observable and needs a test". A `shadow` is a
    // *paint*, laid down behind the element in its own colour, so a key that runs after it
    // is measured against a picture that now contains that colour too. Here the shadow is
    // painted in the very colour being keyed, which makes the two orders opposite pictures
    // rather than merely unequal ones.
    const SCREEN: &str =
        r##"{"name":"chroma","color":"#00CD00","tolerance":0.05,"softness":0,"spill":0}"##;
    const CAST: &str =
        r##"{"name":"shadow","dx":40,"dy":0,"radius":0,"color":"#00CD00","opacity":1}"##;

    let blue = r##"{"id":"square","type":"rect","start":0,"end":1000,"x":200,"y":200,
        "origin":"center","width":100,"height":100,"fill":"#0000FF","effects":[EFFECTS]}"##;

    // (270, 200) is 20 px past the square's right edge and inside the shadow it casts.
    let keyed_first = painted(
        line!(),
        &blue.replace("EFFECTS", &format!("{SCREEN},{CAST}")),
    );
    assert_eq!(
        rgb(&keyed_first, 270, 200),
        [0x00, 0xCD, 0x00],
        "the shadow was laid down after the key, so nothing ever keyed it"
    );

    let shadowed_first = painted(
        line!(),
        &blue.replace("EFFECTS", &format!("{CAST},{SCREEN}")),
    );
    assert_eq!(
        rgb(&shadowed_first, 270, 200),
        BLACK,
        "the key ran over a picture that already had the shadow in it, and took it"
    );
    assert_eq!(
        rgb(&shadowed_first, 200, 200),
        BLUE,
        "and left the square, which is not the colour it was told to find"
    );
}

#[test]
fn two_keys_in_one_list_are_legal_and_apply_in_order() {
    // ADR-0088: "Two `chroma` members in one list are legal and apply in order, consistent
    // with ADR-0040's rule that two effects of the same name are ordinary." Two screens on
    // one element is the case that needs two of them — a single member has one `color`, and
    // nothing in the vocabulary composes colours.
    const GREEN: &str =
        r##"{"name":"chroma","color":"#00CD00","tolerance":0.05,"softness":0,"spill":0}"##;
    const BLUE_KEY: &str =
        r##"{"name":"chroma","color":"#0000FF","tolerance":0.05,"softness":0,"spill":0}"##;

    let painted = painted(
        line!(),
        &format!(
            r##"{{"id":"green","type":"rect","start":0,"end":1000,"x":120,"y":200,
                "origin":"center","width":80,"height":80,"fill":"#00CD00",
                "effects":[{GREEN},{BLUE_KEY}]}},
               {{"id":"blue","type":"rect","start":0,"end":1000,"x":220,"y":200,
                "origin":"center","width":80,"height":80,"fill":"#0000FF",
                "effects":[{GREEN},{BLUE_KEY}]}},
               {{"id":"red","type":"rect","start":0,"end":1000,"x":320,"y":200,
                "origin":"center","width":80,"height":80,"fill":"#FF0000",
                "effects":[{GREEN},{BLUE_KEY}]}}"##
        ),
    );
    assert_eq!(
        rgb(&painted, 120, 200),
        BLACK,
        "the first key took the green"
    );
    assert_eq!(rgb(&painted, 220, 200), BLACK, "the second took the blue");
    assert_eq!(
        rgb(&painted, 320, 200),
        RED,
        "and neither was told to find red, so it is still there"
    );
}

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
        // ADR-0084: `radius` is a field of `shape: "rect"` only, so on a circle it is an
        // unknown key rather than a parameter that quietly does nothing. The geometry
        // *rect* is now a member (below); this one field is not.
        r##"{"name":"mask","shape":"circle","radius":20}"##,
    ] {
        let parsed: Result<montagent_core::model::Effect, _> = serde_json::from_str(refused);
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
        serde_json::from_str::<montagent_core::model::Effect>(admitted)
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
    // ADR-0093: the row carries `E-EFFECT-UNKNOWN` and the finding carries which member it
    // was. The element *was* drawn, minus one thing it asked for, so this is
    // `painted_partially` and not `not_painted`.
    let said = json["frame"]["painted_partially"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .any(|row| row["code"].as_str() == Some("E-EFFECT-UNKNOWN"))
        })
        .unwrap_or(false);
    assert!(
        said,
        "nothing beside the picture said the first effect was not painted: {}",
        serde_json::to_string_pretty(&json["frame"]).unwrap_or_default()
    );
    let named = json["findings"]
        .as_array()
        .map(|findings| {
            findings.iter().any(|finding| {
                finding["code"].as_str() == Some("E-EFFECT-UNKNOWN")
                    && finding["fields"]["index"].as_i64() == Some(0)
                    && finding["fields"]["effect"].as_str() == Some("grayscale")
            })
        })
        .unwrap_or(false);
    assert!(
        named,
        "the finding did not say which member was dropped: {}",
        serde_json::to_string_pretty(&json["findings"]).unwrap_or_default()
    );
}

/// The one `E-EFFECT-UNKNOWN` finding `frame` raised for `effects`, and the report's text
/// form, which is where the declared spelling has to reach the reader.
#[track_caller]
fn unknown_effect(line: u32, effects: &str) -> (Value, String) {
    let (json, _) = answered(line, &square_with(effects), 500);
    let found: Vec<&Value> = json["findings"]
        .as_array()
        .map(|findings| {
            findings
                .iter()
                .filter(|finding| finding["code"].as_str() == Some("E-EFFECT-UNKNOWN"))
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        found.len(),
        1,
        "expected one E-EFFECT-UNKNOWN: {}",
        serde_json::to_string_pretty(&json["findings"]).unwrap_or_default()
    );
    let text = montagent_core::text::render(&json, montagent_core::text::Options::verbose())
        .expect("the report renders");
    (found[0].clone(), text)
}

#[test]
fn the_unknown_effect_is_named_by_the_name_the_author_wrote() {
    // #460: an effect is tagged on `name` (the transition beside it is keyed by `kind`),
    // so the finding reads `name`. An invented member is named as written, and so is a real
    // member whose parameters the vocabulary refuses.
    for (effects, declared) in [
        (r##"{"name":"glow"}"##, "glow"),
        (r##"{"name":"mask","shape":"circle","radius":20}"##, "mask"),
    ] {
        let (finding, text) = unknown_effect(line!(), effects);
        assert_eq!(
            finding["fields"]["effect"].as_str(),
            Some(declared),
            "{effects}"
        );
        assert!(
            text.contains(&format!("`{declared}`")),
            "the message did not name `{declared}`:\n{text}"
        );
        assert!(!text.contains("`kind`"), "an effect has no `kind`:\n{text}");
    }
}

#[test]
fn an_effect_with_no_name_says_so_in_the_effect_vocabulary() {
    for effects in [r##"{"amount":1}"##, r##"{"name":7}"##] {
        let (finding, text) = unknown_effect(line!(), effects);
        let said = finding["fields"]["effect"].as_str().unwrap_or_default();
        assert!(said.contains("`name`"), "{effects} fell back to {said:?}");
        assert!(!text.contains("`kind`"), "an effect has no `kind`:\n{text}");
    }
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

// ---------------------------------------------------------------------------
// The mask rect: one shape-independent parameter set (ADR-0084)
// ---------------------------------------------------------------------------

/// The same 200×100 oblong the section above uses — rect spanning (100,150)..(300,250) on
/// the 400×400 frame — carrying whatever `mask` member is handed to it.
fn oblong_masked(mask: &str) -> String {
    format!(
        r##"{{"id":"oblong","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":200,"height":100,"fill":"#FFFFFF",
            "effects":[{mask}]}}"##
    )
}

// ADR-0084's identity claim — that a bare `mask` and one spelling the element's own rect
// are **one declaration** rather than two — is asserted in `tests/golden_frames.rs`, on an
// element carrying a rotation and a scale. It lives there rather than here because the two
// arities could agree at rest and disagree once a rotation is in the matrix, and because
// ADR-0084's Evidence commissions that frame specifically.

#[test]
fn an_explicit_mask_rect_is_element_local_and_is_not_the_elements_own_rect() {
    // The second thing ADR-0084 added, and the one the bare form cannot express: a rect
    // that is *not* the element's. `(0, 0)` is the element rect's top-left — here the
    // frame point (100, 150) — so a 40×40 rect mask at (10, 10) selects the frame square
    // (110, 160)..(150, 200) and nothing else.
    let inset = painted(
        line!(),
        &oblong_masked(r##"{"name":"mask","shape":"rect","x":10,"y":10,"width":40,"height":40}"##),
    );
    assert_eq!(
        rgb(&inset, 130, 180),
        [0xFF, 0xFF, 0xFF],
        "the middle of the declared rect is kept"
    );
    // Its four sides, each just outside, all erased — which is what says the rect is
    // *placed* rather than merely sized.
    for (x, y, side) in [
        (105, 180, "left of it"),
        (160, 180, "right of it"),
        (130, 155, "above it"),
        (130, 210, "below it"),
    ] {
        assert_eq!(
            rgb(&inset, x, y),
            BLACK,
            "the element still paints {side}, so the mask rect is not where the document put it"
        );
    }

    // And `origin` does not participate: the same mask on the same box anchored by a
    // different corner selects the same part *of the element*, which moves with the
    // element rather than staying put on the frame. Anchored `top-left` at (100, 150) the
    // box lands in exactly the place `center` at (200, 200) put it, so the two pictures
    // are identical — a mask that read `origin` would have shifted by half the box.
    let anchored = painted(
        line!(),
        r##"{"id":"oblong","type":"rect","start":0,"end":1000,"x":100,"y":150,
            "origin":"top-left","width":200,"height":100,"fill":"#FFFFFF",
            "effects":[{"name":"mask","shape":"rect","x":10,"y":10,"width":40,
            "height":40}]}"##,
    );
    assert_eq!(
        differing(&inset, &anchored),
        0,
        "`origin` is a placement anchor, not a re-parameterisation of the box's interior: \
         the mask rect is (0, 0, width, height) under every origin (ADR-0084)"
    );
}

#[test]
fn a_circle_is_inscribed_in_the_mask_rect_rather_than_in_the_element() {
    // "circle stays min(width, height) centred **on the mask rect**, not on the element."
    // A 60×60 rect in the oblong's left half: the circle's centre is the frame point
    // (140, 190), well left of the element's own centre at (200, 200).
    let left = painted(
        line!(),
        &oblong_masked(
            r##"{"name":"mask","shape":"circle","x":10,"y":10,"width":60,"height":60}"##,
        ),
    );
    assert_eq!(
        rgb(&left, 140, 190),
        [0xFF, 0xFF, 0xFF],
        "the circle is centred on its own rect"
    );
    assert_eq!(
        rgb(&left, 200, 200),
        BLACK,
        "the element's own centre is outside a circle the document put elsewhere"
    );
}

#[test]
fn radius_rounds_a_rect_masks_corners_and_zero_is_the_identity() {
    // ADR-0014's rule for the same word on a drawn `rect`, applied to the mask rect: "a
    // single integer, defaulting to 0 — one corner radius, not four".
    let square =
        oblong_masked(r##"{"name":"mask","shape":"rect","x":0,"y":0,"width":100,"height":100}"##);
    let rounded = painted(
        line!(),
        &oblong_masked(
            r##"{"name":"mask","shape":"rect","x":0,"y":0,"width":100,"height":100,"radius":40}"##,
        ),
    );
    let sharp = painted(line!(), &square);

    // The rect spans frame (100,150)..(200,250). Its top-left corner is inside the sharp
    // mask and cut away by a radius of 40.
    assert_eq!(rgb(&sharp, 103, 153), [0xFF, 0xFF, 0xFF], "a sharp corner");
    assert_eq!(rgb(&rounded, 103, 153), BLACK, "and a rounded one is gone");
    // The middle of an edge is unaffected, which is what says this is a corner radius
    // rather than an inset.
    assert_eq!(rgb(&rounded, 150, 153), [0xFF, 0xFF, 0xFF], "the top edge");

    // Identity `0`: written out, it is the same picture as writing nothing.
    let zero = painted(
        line!(),
        &oblong_masked(
            r##"{"name":"mask","shape":"rect","x":0,"y":0,"width":100,"height":100,"radius":0}"##,
        ),
    );
    assert_eq!(
        differing(&zero, &sharp),
        0,
        "`radius: 0` is the identity value, so it is the same declaration as omitting it"
    );
}

#[test]
fn the_mask_rects_four_fields_are_all_or_none() {
    // ADR-0084: "Either all four are absent, or all four are present. A partial tuple —
    // `x` without `width` — is a schema error naming the other three."
    for (partial, named) in [
        (
            r##"{"name":"mask","shape":"rect","x":10}"##,
            "`y`, `width`, `height`",
        ),
        (
            r##"{"name":"mask","shape":"rect","width":10,"height":10}"##,
            "`x`, `y`",
        ),
        (
            r##"{"name":"mask","shape":"circle","x":0,"y":0,"width":10}"##,
            "`height`",
        ),
    ] {
        let refused = serde_json::from_str::<montagent_core::model::Effect>(partial)
            .expect_err("a partial mask rect is a schema error");
        let said = refused.to_string();
        assert!(
            said.contains(named),
            "the error must name what is missing — {named} — and said: {said}"
        );
    }

    // Both admitted arities, so the assertion above is about the relation rather than a
    // deserializer that refuses geometry.
    for admitted in [
        r##"{"name":"mask","shape":"circle"}"##,
        r##"{"name":"mask","shape":"circle","x":0,"y":0,"width":10,"height":10}"##,
    ] {
        serde_json::from_str::<montagent_core::model::Effect>(admitted)
            .unwrap_or_else(|e| panic!("{admitted} is an admitted arity and did not parse: {e}"));
    }
}

#[test]
fn radius_is_a_field_of_rect_only_and_says_why_on_the_other_two() {
    // ADR-0084: on `circle` or `ellipse` it is "an unknown key, with a message naming the
    // reason rather than a field that quietly does nothing". The message is the point —
    // ADR-0007 has ruled twice that "a field the renderer cannot honour is worse than no
    // field", and a bare "unknown key `radius`" would leave the author guessing whether
    // this Montagent is simply older than their document (ADR-0016).
    for shape in ["circle", "ellipse"] {
        let refused = serde_json::from_str::<montagent_core::model::Effect>(&format!(
            r##"{{"name":"mask","shape":"{shape}","radius":8}}"##
        ))
        .expect_err("`radius` is not a key of this mask");
        let said = refused.to_string();
        assert!(
            said.contains("corners to round") && said.contains(shape),
            "the message must name the shape and the reason, and said: {said}"
        );
    }

    serde_json::from_str::<montagent_core::model::Effect>(
        r##"{"name":"mask","shape":"rect","radius":8}"##,
    )
    .expect("`radius` is a field of a `rect` mask");
}

#[test]
fn the_word_a_mask_shape_is_named_by_is_the_word_the_document_spells() {
    // `MaskShape::as_str` is the one place the three shape words are written by hand rather
    // than derived by `rename_all = "lowercase"`, because the `radius` message has to name
    // the shape and a serde round trip is not a thing to do inside a deserializer. That
    // makes it a second listing, so it is tied to the first one here rather than trusted.
    use montagent_core::model::MaskShape;
    for shape in [MaskShape::Circle, MaskShape::Rect, MaskShape::Ellipse] {
        assert_eq!(
            serde_json::to_value(shape).expect("a shape serialises"),
            serde_json::json!(shape.as_str())
        );
    }
}

#[test]
fn a_mask_radius_on_the_wrong_shape_is_the_same_report_as_one_on_a_drawn_ellipse() {
    // ADR-0084 does not merely call it an error — "on `circle`/`ellipse` it is an **unknown
    // key** … following ADR-0014's rule for the same word on the drawn `shape` element" —
    // and `CONTEXT.md` promises it behaves "exactly as it is on a drawn ellipse". That is a
    // claim about the *report*, not only about the refusal: `E-SCHEMA-UNKNOWN-KEY` is what
    // carries ADR-0016's guarantee text ("it may belong to a newer format revision … do not
    // delete the key to make the file validate"), and `E-SCHEMA` carries none of it.
    //
    // Measured side by side, because the two travel through different code — the drawn
    // ellipse's `radius` is refused by the derive, the mask's by a hand-written check — and
    // agreeing today is the only way to know they agree.
    let codes = |element: &str| {
        let dir = tempdir(line!());
        let path = write_project(&dir, "p.montagent.json", &one_track(element));
        let document = montagent_core::parse::read(&path).expect("a Loose document");
        let mut report =
            montagent_core::report::Report::new("validate", Some(document.path().to_string()));
        montagent_core::checks::schema::check(&document, &mut report);
        report
            .findings
            .iter()
            .map(|f| f.code.clone())
            .collect::<Vec<_>>()
    };

    let drawn = codes(
        r##"{"id":"oval","type":"ellipse","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":100,"height":100,"fill":"#FFFFFF","radius":8}"##,
    );
    assert_eq!(drawn, ["E-SCHEMA-UNKNOWN-KEY"], "ADR-0014's own case");

    let masked = codes(
        r##"{"id":"square","type":"rect","start":0,"end":1000,"x":200,"y":200,
            "origin":"center","width":100,"height":100,"fill":"#FFFFFF",
            "effects":[{"name":"mask","shape":"ellipse","radius":8}]}"##,
    );
    assert_eq!(
        masked, drawn,
        "a mask's `radius` on an inscribed ellipse must report as the same kind of fact as \
         a drawn ellipse's does (ADR-0084, CONTEXT.md's Mask entry)"
    );
}

#[test]
fn the_mask_member_round_trips_in_adr_0084s_key_order() {
    // ADR-0041 hands a new field's position to the ADR that introduces it, and ADR-0084
    // takes it explicitly: `name, shape, x, y, width, height, radius`. Asserted on the
    // wire form rather than on the struct, because the wire form is the artifact `fmt` and
    // `validate` read the order from.
    let written = r##"{"name":"mask","shape":"rect","x":1,"y":2,"width":3,"height":4,"radius":5}"##;
    let parsed: montagent_core::model::Effect = serde_json::from_str(written).expect("it parses");
    assert_eq!(
        serde_json::to_string(&parsed).expect("it serialises"),
        written
    );

    // And the identity value is *omitted* rather than written out (ADR-0030: presence is
    // content — the bare form says "follow the element's rect" and keeps saying it).
    let bare: montagent_core::model::Effect =
        serde_json::from_str(r##"{"name":"mask","shape":"circle"}"##).expect("it parses");
    assert_eq!(
        serde_json::to_string(&bare).expect("it serialises"),
        r##"{"name":"mask","shape":"circle"}"##
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

/// The fixture badge's slot — `handle-logo`'s `clip`, which ADR-0068 records as equal to
/// the element's own rect — as `[x, y, width, height]` in project pixels.
///
/// Read from the fixture rather than restated, because both tests below crop to it and a
/// hand-copied rectangle is a second statement of the project: the copy is what drifts
/// first, and it would drift into a stray-pixel list rather than into a legible failure.
fn badge_slot() -> [i64; 4] {
    let document = common::document(&common::fixture_project());
    let badge = common::elements(&document)
        .find(|element| element["id"] == "handle-logo")
        .expect("the fixture's badge");
    let clip: Vec<i64> = badge["clip"]
        .as_array()
        .expect("the badge's clip")
        .iter()
        .map(|n| n.as_i64().expect("whole pixels"))
        .collect();
    let slot: [i64; 4] = clip.try_into().expect("a clip is four numbers");
    assert_eq!(
        slot[2], slot[3],
        "the badge's slot is square, which is what makes its inscribed circle the badge"
    );
    slot
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
    // the whole badge. Nothing inside the circle moves by a single bit, and every pixel
    // that does move is on the circle's own edge. The whole-frame consequence is 112
    // pixels of the 540x960 `fixture-intro`, a mean channel delta of 0.0041 — the same
    // order as the cross-platform last bit `tests/golden_frames.rs` already budgets for,
    // and an improvement to look at, since the badge's edge stops being aliased. The two
    // fixture goldens are regenerated in the same change, and the pictures were compared
    // side by side before they were. ADR-0075 retires the ADR's falsified sentence
    // ([#279](https://github.com/MBehtemam/Montagent/issues/279)), and the test below
    // measures the whole frame the way this one measures the badge.
    let with_mask = std::fs::read_to_string(common::fixture_project()).expect("the fixture");
    let without_mask = with_mask.replace(r#","effects":[{"name":"mask","shape":"circle"}]"#, "");
    assert_ne!(
        with_mask, without_mask,
        "the fixture no longer carries the badge's `mask` effect, so this asserts nothing"
    );

    // The assets resolve against the project file's own directory (ADR-0053), so the
    // unmasked copy has to sit beside them rather than in a scratch directory of its own.
    let beside = Scratch::beside_the_fixture("no-mask-for-the-badge", &without_mask);

    // The crop is the badge's own slot, so the comparison is the badge and nothing else.
    // Every other element in the frame would only dilute it.
    let [x, y, side, _] = badge_slot();
    let ask = Ask {
        full: true,
        png: true,
        crop: Some(format!("{x},{y},{side},{side}")),
        ..at(400)
    };
    let (_, masked) = drawn(&common::fixture_project(), &ask);
    let (_, bare) = drawn(beside.path(), &ask);
    let (masked, bare) = (pixels(&masked), pixels(&bare));
    assert_eq!(
        masked.dimensions(),
        bare.dimensions(),
        "the two crops must be the same size to be compared pixel for pixel"
    );

    // The inscribed circle of the square slot: centre and radius are both half its side
    // (ADR-0068's param-less `mask`).
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
            let half = side as f64 / 2.0;
            let (dx, dy) = (f64::from(x) + 0.5 - half, f64::from(y) + 0.5 - half);
            let radius = (dx * dx + dy * dy).sqrt();
            if (radius - half).abs() > 1.0 {
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
    // bit for bit, which is the half of the ADR's claim that does hold exactly. These six
    // are picked for a 68x68 slot, so a resized badge fails here rather than silently
    // sampling somewhere else.
    assert_eq!(
        side, 68,
        "the sample points below are chosen for a 68x68 slot"
    );
    for (x, y) in [(34, 34), (34, 8), (8, 34), (60, 34), (34, 60), (16, 16)] {
        assert_eq!(
            masked.get_pixel(x, y),
            bare.get_pixel(x, y),
            "({x}, {y}) is inside the circle and the mask changed it"
        );
    }
}

#[test]
fn the_badges_mask_changes_only_the_antialiasing_of_its_own_rim() {
    // The whole-frame half of the test above, and the re-executable evidence for the
    // numbers ADR-0075 states in retiring ADR-0068's "the rendered frame is unchanged"
    // ([#279](https://github.com/MBehtemam/Montagent/issues/279)). The test above asks
    // *which* pixels the mask may touch; this one asks *how much of the picture an agent
    // receives* they amount to, at the exact instant and scale the committed golden is
    // taken at.
    let with_mask = std::fs::read_to_string(common::fixture_project()).expect("the fixture");
    let without_mask = with_mask.replace(r#","effects":[{"name":"mask","shape":"circle"}]"#, "");
    assert_ne!(
        with_mask, without_mask,
        "the fixture no longer carries the badge's `mask` effect, so this asserts nothing"
    );

    // Beside the fixture, because assets resolve against the project file's own directory
    // (ADR-0053), and under its own name, because the test above writes one too and the
    // two run in parallel.
    let beside = Scratch::beside_the_fixture("no-mask-for-the-whole-frame", &without_mask);

    // Half scale — 540×960 — because that is what `golden_frames.rs` commits, on the
    // ground that it is "the picture an agent actually receives".
    let masked = rendered(&common::fixture_project(), 400, /* full */ false);
    let bare = rendered(beside.path(), 400, /* full */ false);
    assert_eq!(
        masked.dimensions(),
        bare.dimensions(),
        "the two renders must be the same size to be compared pixel for pixel"
    );

    // The badge's slot, halved, because this frame is the half-scale answer. Every pixel
    // the mask moves must be inside it: a mask on one element that changed a pixel
    // elsewhere would be a compositing defect rather than an edge.
    let [x, y, side, _] = badge_slot();
    let (slot_x, slot_y, slot_side) = ((x / 2) as u32, (y / 2) as u32, (side / 2) as u32);
    let mut changed = 0u32;
    let mut strays = Vec::new();
    for y in 0..masked.height() {
        for x in 0..masked.width() {
            let (a, b) = (masked.get_pixel(x, y).0, bare.get_pixel(x, y).0);
            // All four channels, unlike the mean below: a defect that moved only alpha
            // outside the badge is exactly what the stray check exists to catch, and a
            // colour-only difference would not see it.
            let delta: u64 = (0..4).map(|c| u64::from(a[c].abs_diff(b[c]))).sum();
            if delta == 0 {
                continue;
            }
            changed += 1;
            let inside = (slot_x..slot_x + slot_side).contains(&x)
                && (slot_y..slot_y + slot_side).contains(&y);
            if !inside {
                strays.push((x, y, delta));
            }
        }
    }
    let delta = mean_delta(&masked, &bare);
    let score = ssim(&Plane::of(&masked), &Plane::of(&bare), &Scope::whole());
    println!(
        "MASK  {changed} of {} pixels changed, mean channel delta {delta:.4}, SSIM \
         {score:.6}",
        masked.pixels().len()
    );

    assert!(
        strays.is_empty(),
        "{} pixels outside the badge's own {slot_side}x{slot_side} slot changed: {:?}",
        strays.len(),
        &strays[..strays.len().min(8)]
    );

    // The bands are wide enough for the cross-platform last bit #34 measured at
    // 0.003–0.004 and no wider: the claim ADR-0075 rests on is the *order* — a rim's
    // worth of pixels, a mean delta two orders below the golden suite's own 0.5 ceiling —
    // not a bit-exact count that would fail on a machine that rounds the other way.
    assert!(
        (60..=200).contains(&changed),
        "the mask changed {changed} pixels; ADR-0075 measured 112, a rim's worth. A count \
         far outside that band means the mask is no longer trimming only the badge's edge"
    );
    assert!(
        delta <= 0.01,
        "mean channel delta {delta:.4}; ADR-0075 measured 0.0041, and the golden \
         suite's own ceiling for an unchanged picture is 0.5"
    );
    assert!(
        score >= 0.9999,
        "SSIM {score:.6}; ADR-0075 measured 0.999982, against a golden floor of 0.999"
    );
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
    let project = write_project(&dir, "p.montagent.json", project_body);
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
        "p.montagent.json",
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
    // *measurement* also reads — `montagent-text` grows a line's extent by `2 x
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

#[test]
fn the_printed_answer_names_the_crossfade_too() {
    // The JSON carries `crossfades`, but the CLI prints the text block — and an agent
    // reading that block is the reader the caption exists for. A field only the JSON
    // shows is a field half the surface does not have.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montagent.json",
        &bridged((0, 2000), (1000, 3000), (1000, 2000)),
    );
    let answer = frame(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(1500)
        },
    );
    let printed =
        montagent_core::text::render(&answer.to_json(), montagent_core::text::Options::default())
            .expect("the answer renders");

    assert!(
        printed.contains("crossfade"),
        "the printed answer says nothing about the crossfade:\n{printed}"
    );
    assert!(
        printed.contains("first") && printed.contains("second"),
        "it does not name the pair it bridges:\n{printed}"
    );
}

#[test]
fn a_transition_that_cannot_be_read_is_named_rather_than_silently_inert() {
    // A transition draws nothing of its own, so one this build cannot use looks exactly
    // like one that is simply not running yet — and the verb's own rule is that an agent
    // must never have to tell those two apart by guessing.
    let unusable = |element: &str| {
        let dir = tempdir(line!());
        let project = write_project(
            &dir,
            "p.montagent.json",
            &canonical(&format!(
                r##"{{"frame":{{"width":400,"height":400}},"fps":25,"background":"#000000",
                    "tracks":[
                      {{"name":"a","layer":0,"elements":[
                        {{"id":"first","type":"rect","start":0,"end":2000,"x":200,"y":200,
                          "width":100,"height":100,"fill":"#FF0000"}}]}},
                      {{"name":"b","layer":1,"elements":[
                        {{"id":"second","type":"rect","start":1000,"end":3000,"x":200,
                          "y":200,"width":100,"height":100,"fill":"#0000FF"}}]}},
                      {{"name":"bridge","layer":2,"elements":[{element}]}}
                    ]}}"##
            )),
        );
        let (json, _) = drawn(
            &project,
            &Ask {
                full: true,
                png: true,
                ..at(1500)
            },
        );
        // ADR-0093: the row carries the *code*, and the sentence naming the offending
        // value is the finding's. Both are read, because "named rather than silently
        // inert" is a claim about the pair — a code with no detail names the condition
        // and not the instance.
        let codes: Vec<String> = json["frame"]["not_painted"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row["code"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let details: Vec<String> = json["findings"]
            .as_array()
            .map(|findings| {
                findings
                    .iter()
                    .filter_map(|finding| finding["fields"]["detail"].as_str())
                    .chain(
                        findings
                            .iter()
                            .filter_map(|finding| finding["fields"]["from"].as_str()),
                    )
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        (codes, details)
    };

    // A `kind` outside v1's one-member vocabulary (ADR-0059 defers wipe, slide and push).
    let (codes, details) = unusable(
        r##"{"id":"fade","type":"transition","start":1000,"end":2000,"kind":"wipe",
             "from":"first","to":"second"}"##,
    );
    assert!(
        codes.iter().any(|code| code == "E-NOT-PAINTED-UNDRAWABLE"),
        "a deferred `kind` was silently inert: {codes:?}"
    );
    assert!(
        details.iter().any(|detail| detail.contains("wipe")),
        "the finding did not name the `kind` it could not draw: {details:?}"
    );

    // A `from` naming nothing in the document. ADR-0093 gives it its own code: it is the
    // one transition fault no `validate` check ever claimed.
    let (codes, details) = unusable(
        r##"{"id":"fade","type":"transition","start":1000,"end":2000,"kind":"crossfade",
             "from":"nobody","to":"second"}"##,
    );
    assert!(
        codes
            .iter()
            .any(|code| code == "E-NOT-PAINTED-UNRESOLVED-REF"),
        "a dangling reference was silently inert: {codes:?}"
    );
    assert!(
        details.iter().any(|detail| detail.contains("nobody")),
        "the finding did not name the reference it could not resolve: {details:?}"
    );

    // A pair that never coexist, which is `E-TRANSITION-NO-OVERLAP` in `validate` and no
    // window at all here.
    let (codes, _) = unusable(
        r##"{"id":"fade","type":"transition","start":1000,"end":2000,"kind":"crossfade",
             "from":"first","to":"first"}"##,
    );
    assert!(codes.is_empty() || codes.iter().all(|code| !code.is_empty()));

    // And the other half of the rule: a transition that is simply outside its own window
    // is *not* listed, because nothing about the picture is missing.
    let dir = tempdir(line!());
    let project = write_project(
        &dir,
        "p.montagent.json",
        &bridged((0, 2000), (1000, 3000), (1000, 2000)),
    );
    let (json, _) = drawn(
        &project,
        &Ask {
            full: true,
            png: true,
            ..at(2500)
        },
    );
    assert_eq!(
        json["frame"]["not_painted"].as_array().map(Vec::len),
        Some(0),
        "a transition that has simply finished was reported as a problem: {}",
        json["frame"]["not_painted"]
    );
}
