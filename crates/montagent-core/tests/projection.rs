//! Projection (#787, ADR-0167 and ADR-0168): `swivel`, `tilt` and `perspective`.
//!
//! Asserted at the seams a projection is used through, each by its public verb:
//!
//! - **`frame`'s pixels**: the quadrilateral a projected card paints, what a face turned away
//!   paints, and the path an element takes when an angle is written.
//! - **`validate`'s report**: the three errors and the two reviews.
//! - **`query --at`**: the resolved fields, `facing`, `corners` and `ink_box`.
//! - **`shift`**: the three fields split as plain animatable numbers.
//!
//! Byte identity across painter counts, with the bounds hint on and off, lives beside the
//! other gating runs in `tests/painters.rs`; the committed goldens in `tests/golden_frames.rs`
//! hold that an element with neither angle is unchanged.
//!
//! The corner oracle is the prototype's hand-checked table (#786's report): five projected
//! quadrilaterals worked out in f64 from CSS's `perspective(d) rotateX(tilt) rotateY(swivel)`,
//! never read back from the code under test.

use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

// ---- the picture ---------------------------------------------------------------------

/// A project of one track holding `elements`, on a black frame.
fn project(width: u32, height: u32, elements: &[Value]) -> Value {
    json!({"frame": {"width": width, "height": height}, "fps": 25, "background": "#000000",
           "duration": 1000,
           "tracks": [{"name": "only", "layer": 0, "elements": elements}]})
}

/// The exact pixels at `instant` of `project`, at true scale and lossless, decoded by a
/// decoder that is not the one that wrote them.
#[track_caller]
fn painted(project: &Value, instant: i64) -> image::RgbaImage {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&project.to_string()));
    let answer = frame(
        &path,
        &Ask {
            at: Some(instant),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    image::load_from_memory(&answer.image().expect("a picture").bytes)
        .expect("the bytes decode")
        .to_rgba8()
}

/// A white rectangle, `width` by `height`, with `extra` keys merged in.
fn card(width: u32, height: u32, extra: Value) -> Value {
    let mut element = json!({"id": "card", "type": "rect", "start": 0, "end": 1000,
        "width": width, "height": height, "fill": "#FFFFFF"});
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

/// Whether the pixel at `(x, y)` is lit (the card is white on black).
fn lit(picture: &image::RgbaImage, x: f64, y: f64) -> bool {
    let (width, height) = picture.dimensions();
    if x < 0.0 || y < 0.0 || x >= f64::from(width) || y >= f64::from(height) {
        return false;
    }
    picture.get_pixel(x as u32, y as u32).0[0] >= 128
}

/// 3 px inside and 3 px outside `corner`, along the line to the quadrilateral's centre: the
/// point three pixels in must be white and the point three pixels out must be black.
#[track_caller]
fn assert_corners_painted(picture: &image::RgbaImage, corners: [(f64, f64); 4]) {
    let centre = (
        corners.iter().map(|c| c.0).sum::<f64>() / 4.0,
        corners.iter().map(|c| c.1).sum::<f64>() / 4.0,
    );
    for (i, (cx, cy)) in corners.into_iter().enumerate() {
        let (dx, dy) = (centre.0 - cx, centre.1 - cy);
        let length = dx.hypot(dy);
        let (ux, uy) = (dx / length, dy / length);
        assert!(
            lit(picture, cx + 3.0 * ux, cy + 3.0 * uy),
            "corner {i} {:?}: three pixels inside is not painted",
            (cx, cy)
        );
        assert!(
            !lit(picture, cx - 3.0 * ux, cy - 3.0 * uy),
            "corner {i} {:?}: three pixels outside is painted",
            (cx, cy)
        );
    }
}

#[test]
fn a_swivelled_and_tilted_card_paints_the_quadrilateral_css_order_gives() {
    // The prototype's first row: centre origin, swivel 35, tilt 20, perspective 900, on a
    // 400×250 card centred at (640, 360).
    let picture = painted(
        &project(
            1280,
            720,
            &[card(
                400,
                250,
                json!({"swivel": 35, "tilt": 20, "perspective": 900}),
            )],
        ),
        0,
    );
    assert_corners_painted(
        &picture,
        [
            (463.41, 191.10),
            (780.35, 292.98),
            (792.79, 506.14),
            (443.26, 453.94),
        ],
    );
}

#[test]
fn a_tilt_about_the_top_left_widens_the_bottom() {
    // Third row: origin top-left at (300, 150), tilt 50, perspective 800.
    let picture = painted(
        &project(
            1280,
            720,
            &[card(
                400,
                250,
                json!({"origin": "top-left", "x": 300, "y": 150, "tilt": 50, "perspective": 800}),
            )],
        ),
        0,
    );
    assert_corners_painted(
        &picture,
        [
            (300.0, 150.0),
            (700.0, 150.0),
            (825.89, 361.27),
            (300.0, 361.27),
        ],
    );
}

#[test]
fn the_projection_is_carried_through_rotation_and_scale() {
    // Fifth row: swivel 40, tilt -25, perspective 1000, then rotation 30 and scale [1.3, 0.8].
    let picture = painted(
        &project(
            1280,
            720,
            &[card(
                400,
                250,
                json!({"swivel": 40, "tilt": -25, "perspective": 1000,
                       "rotation": 30, "scale": [1.3, 0.8]}),
            )],
        ),
        0,
    );
    assert_corners_painted(
        &picture,
        [
            (460.74, 190.94),
            (865.19, 344.45),
            (767.34, 480.10),
            (384.17, 377.67),
        ],
    );
}

#[test]
fn a_face_turned_away_or_edge_on_paints_nothing() {
    for (swivel, tilt) in [(90, 0), (120, 0), (-100, 0), (0, 90), (0, 200), (100, 100)] {
        let picture = painted(
            &project(
                640,
                360,
                &[card(
                    200,
                    100,
                    json!({"swivel": swivel, "tilt": tilt, "perspective": 1000}),
                )],
            ),
            0,
        );
        // (100, 100) turns both ways, which faces front again: handled below.
        let expected_front = (swivel, tilt) == (100, 100);
        assert_eq!(
            picture.pixels().any(|p| p.0[0] > 0),
            expected_front,
            "swivel {swivel}, tilt {tilt}"
        );
    }
}

// ---- query --at ----------------------------------------------------------------------

/// The stack row of `card` in `query --at instant`.
#[track_caller]
fn viewed(project: &Value, instant: i64) -> Value {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&project.to_string()));
    let answer = montagent_core::verbs::query::query(
        &path,
        &montagent_core::verbs::query::Ask {
            at: Some(instant),
            ..Default::default()
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "query --at did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    answer.to_json()["query"]["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|row| row["id"] == "card")
        .expect("the card is present")
        .clone()
}

fn number(value: &Value) -> f64 {
    value.as_f64().expect("a number")
}

#[test]
fn query_at_prints_the_resolved_angles_the_facing_and_the_corners() {
    let view = viewed(
        &project(
            1280,
            720,
            &[card(
                400,
                250,
                json!({"swivel": 35, "tilt": 20, "perspective": 900}),
            )],
        ),
        0,
    );
    let projection = &view["projection"];
    assert_eq!(projection["swivel"], 35.0);
    assert_eq!(projection["tilt"], 20.0);
    assert_eq!(projection["perspective"], 900.0);
    assert_eq!(projection["facing"], "front");
    let expected = [
        (463.41, 191.10),
        (780.35, 292.98),
        (792.79, 506.14),
        (443.26, 453.94),
    ];
    for (corner, (x, y)) in projection["corners"]
        .as_array()
        .expect("four corners")
        .iter()
        .zip(expected)
    {
        assert!((number(&corner[0]) - x).abs() < 0.01, "{corner} vs {x}");
        assert!((number(&corner[1]) - y).abs() < 0.01, "{corner} vs {y}");
    }
    // `ink_box` holds the bounds of the quadrilateral.
    let ink = &view["ink_box"];
    assert!((number(&ink["x"]) - 443.26).abs() < 0.01, "{ink}");
    assert!((number(&ink["y"]) - 191.10).abs() < 0.01, "{ink}");
    assert!(
        (number(&ink["width"]) - (792.79 - 443.26)).abs() < 0.02,
        "{ink}"
    );
    assert!(
        (number(&ink["height"]) - (506.14 - 191.10)).abs() < 0.02,
        "{ink}"
    );
}

#[test]
fn query_at_says_away_or_edge_prints_the_corners_and_leaves_the_ink_box_empty() {
    for (swivel, facing) in [(120, "away"), (90, "edge"), (-270, "edge"), (-200, "away")] {
        let view = viewed(
            &project(
                640,
                360,
                &[card(
                    200,
                    100,
                    json!({"swivel": swivel, "perspective": 1000}),
                )],
            ),
            0,
        );
        assert_eq!(view["projection"]["facing"], facing, "swivel {swivel}");
        assert_eq!(
            view["projection"]["corners"].as_array().map(Vec::len),
            Some(4),
            "swivel {swivel}"
        );
        assert_eq!(view["ink_box"], Value::Null, "swivel {swivel}");
    }
}

#[test]
fn query_at_prints_no_projection_for_an_element_that_writes_no_angle() {
    let view = viewed(&project(640, 360, &[card(200, 100, json!({}))]), 0);
    assert!(view.get("projection").is_none(), "{view}");
}

// ---- validate ------------------------------------------------------------------------

use montagent_core::finding::Finding;
use montagent_core::report::Report;

#[track_caller]
fn validated(project: &Value) -> Report {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &canonical(&project.to_string()));
    montagent_core::validate(&path)
}

/// The findings of `code`, in the order the report holds them.
fn of<'a>(report: &'a Report, code: &str) -> Vec<&'a Finding> {
    report.findings.iter().filter(|f| f.code == code).collect()
}

/// One `card` on a 640×360 frame, validated.
#[track_caller]
fn card_report(width: u32, height: u32, extra: Value) -> Report {
    validated(&project(640, 360, &[card(width, height, extra)]))
}

#[test]
fn an_angle_without_a_perspective_is_an_error_naming_the_field() {
    for field in ["swivel", "tilt"] {
        let report = card_report(200, 100, json!({field: 20}));
        let found = of(&report, "E-PROJECTION-PERSPECTIVE-MISSING");
        assert_eq!(found.len(), 1, "{field}: {:#?}", report.findings);
        assert_eq!(found[0].fields["field"], field);
    }
    assert!(
        of(
            &card_report(200, 100, json!({"swivel": 20, "perspective": 900})),
            "E-PROJECTION-PERSPECTIVE-MISSING"
        )
        .is_empty()
    );
}

#[test]
fn a_perspective_with_no_angle_key_is_dead_but_an_angle_of_zero_is_an_angle() {
    let alone = card_report(200, 100, json!({"perspective": 900}));
    assert_eq!(of(&alone, "E-PROJECTION-PERSPECTIVE-ALONE").len(), 1);
    // A `0`, static or keyed, counts as present.
    for angle in [
        json!({"swivel": 0}),
        json!({"tilt": 0}),
        json!({"swivel": [{"t": 0, "v": 0}, {"t": 500, "v": 0, "ease": "linear"}]}),
    ] {
        let mut element = json!({"perspective": 900});
        for (key, value) in angle.as_object().unwrap() {
            element[key] = value.clone();
        }
        let report = card_report(200, 100, element);
        assert!(
            of(&report, "E-PROJECTION-PERSPECTIVE-ALONE").is_empty(),
            "{angle}: {:#?}",
            report.findings
        );
    }
}

/// The 320×180 card about `center`: r is the half-diagonal, 183.58.
#[test]
fn the_eye_bound_names_the_instant_r_and_the_smallest_perspective_that_passes() {
    let at = |perspective: Value| {
        card_report(320, 180, json!({"swivel": 10, "perspective": perspective}))
    };
    let report = at(json!(183));
    let found = of(&report, "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].fields["at"], 0);
    assert_eq!(found[0].fields["r"], 183.58);
    assert_eq!(found[0].fields["minimum"], 184);
    // 184 clears r = 183.58.
    assert!(of(&at(json!(184)), "E-PROJECTION-EYE").is_empty());
    // A keyed perspective is looked at where it is keyed.
    let keyed = at(json!([{"t": 0, "v": 400}, {"t": 600, "v": 150, "ease": "linear"}]));
    let found = of(&keyed, "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{:#?}", keyed.findings);
    assert_eq!(found[0].fields["at"], 600);
}

#[test]
fn the_eye_bound_reads_the_reach_of_every_effect_not_the_bare_box() {
    // 200 clears the bare box (183.58) but a shadow cast 30 px down and right, blurred by
    // ⌈3σ⌉ = 15 more, puts the far corner 245.5 px from the centre.
    let shadowed = json!({"swivel": 10, "perspective": 200, "effects": [
        {"name": "shadow", "dx": 30, "dy": 30, "radius": 10, "color": "#000000", "opacity": 0.5}]});
    let report = card_report(320, 180, shadowed);
    let found = of(&report, "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].fields["r"], 245.46);
    assert_eq!(found[0].fields["minimum"], 246);
    assert!(
        of(
            &card_report(320, 180, json!({"swivel": 10, "perspective": 200})),
            "E-PROJECTION-EYE"
        )
        .is_empty()
    );
}

#[test]
fn the_eye_bound_reads_the_eased_extreme_of_a_keyed_perspective() {
    // Both keys clear r (183.58), but the bezier overshoots past 1, so between them the
    // value dips to about 106: an eased extreme, not a key.
    let overshoot = |ease: Value| {
        card_report(
            320,
            180,
            json!({"swivel": 10, "perspective": [
                {"t": 0, "v": 300}, {"t": 1000, "v": 250, "ease": ease}]}),
        )
    };
    let report = overshoot(json!([0.25, 5.0, 0.75, 5.0]));
    let found = of(&report, "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    let at = found[0].fields["at"].as_f64().expect("an instant");
    assert!(
        at > 0.0 && at < 1000.0,
        "the extreme is inside the segment: {at}"
    );
    assert!(of(&overshoot(json!("linear")), "E-PROJECTION-EYE").is_empty());
}

#[test]
fn an_element_that_never_faces_the_eye_is_a_review_and_one_that_does_for_a_moment_is_not() {
    let away =
        |swivel: Value| card_report(200, 100, json!({"swivel": swivel, "perspective": 1000}));
    for turned in [json!(120), json!(90), json!(-270)] {
        let report = away(turned.clone());
        assert_eq!(
            of(&report, "R-PROJECTION-AWAY").len(),
            1,
            "{turned}: {:#?}",
            report.findings
        );
    }
    assert!(of(&away(json!(89)), "R-PROJECTION-AWAY").is_empty());
    // A flip that starts facing the eye draws for part of its life.
    let flip = away(json!([{"t": 0, "v": 0}, {"t": 800, "v": 180, "ease": "linear"}]));
    assert!(
        of(&flip, "R-PROJECTION-AWAY").is_empty(),
        "{:#?}",
        flip.findings
    );
    // Keys that are both away, with a front-facing stretch between them.
    let spin = away(json!([{"t": 0, "v": 100}, {"t": 800, "v": 460, "ease": "linear"}]));
    assert!(
        of(&spin, "R-PROJECTION-AWAY").is_empty(),
        "{:#?}",
        spin.findings
    );
    let back = away(json!([{"t": 0, "v": 100}, {"t": 800, "v": 260, "ease": "linear"}]));
    assert_eq!(of(&back, "R-PROJECTION-AWAY").len(), 1);
}

#[test]
fn strong_foreshortening_is_a_review_naming_the_magnification_and_the_value_for_two() {
    // r = 183.58, so 2r is 367.17 and d = 300 magnifies the near edge 300 / 116.42 = 2.58.
    let soft =
        |perspective: u32| card_report(320, 180, json!({"swivel": 10, "perspective": perspective}));
    let report = soft(300);
    let found = of(&report, "R-PROJECTION-SOFT");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].fields["magnification"], 2.6);
    assert_eq!(found[0].fields["minimum"], 368);
    assert!(of(&soft(400), "R-PROJECTION-SOFT").is_empty());
    // At or inside the eye bound the error speaks, and the review does not.
    assert!(of(&soft(183), "R-PROJECTION-SOFT").is_empty());
}

#[test]
fn off_canvas_reads_the_projected_bounds_not_the_flat_box() {
    let on_frame = |x: i64, swivel: i64, perspective: u32| {
        card_report(
            200,
            100,
            json!({"x": x, "y": 180, "swivel": swivel, "perspective": perspective}),
        )
    };
    // The flat box [600, 800] meets the frame, but swivelled -80 about its centre the card's
    // near edge swings out to 684..720, past the frame's right edge at 640.
    assert_eq!(of(&on_frame(700, -80, 1000), "R-OFF-CANVAS").len(), 1);
    assert!(of(&on_frame(700, 0, 1000), "R-OFF-CANVAS").is_empty());
    // The flat box [-200, 0] never meets the frame, but swivelled -30 under a close eye the
    // near right edge swings in to about x = 48.
    assert_eq!(of(&on_frame(-100, 0, 120), "R-OFF-CANVAS").len(), 1);
    assert!(of(&on_frame(-100, -30, 120), "R-OFF-CANVAS").is_empty());
}

#[test]
fn the_layer_tie_reads_the_projected_bounds_not_the_flat_box() {
    // Two elements on one layer. The card's flat box [600, 800] meets the plate's [550, 650],
    // but swivelled -80 its quadrilateral is 684..720 and clear of it.
    let tied = |swivel: Option<i64>| {
        let mut moving = card(
            200,
            100,
            json!({"id": "moving", "x": 700, "y": 180, "perspective": 1000}),
        );
        if let Some(swivel) = swivel {
            moving["swivel"] = json!(swivel);
        }
        let plate = json!({"id": "plate", "type": "rect", "start": 0, "end": 1000,
            "x": 600, "y": 180, "width": 100, "height": 100, "fill": "#FF0000"});
        let mut document = project(1280, 360, &[]);
        document["tracks"] = json!([
            {"name": "a", "layer": 5, "elements": [moving]},
            {"name": "b", "layer": 5, "elements": [plate]},
        ]);
        validated(&document)
    };
    assert_eq!(of(&tied(Some(-80)), "E-LAYER-TIE").len(), 0);
    // Written `swivel: 0` is the flat box exactly, and the two do overlap.
    assert_eq!(of(&tied(Some(0)), "E-LAYER-TIE").len(), 1);
    // And a projected element is measured where a rotated one is refused.
    assert!(of(&tied(Some(-80)), "U-LAYER-TIE-ROTATED").is_empty());
}

#[test]
fn a_keyed_angle_is_motion_so_motion_blur_on_it_is_not_still() {
    let blurred = |swivel: Value| {
        card_report(
            200,
            100,
            json!({"swivel": swivel, "perspective": 1000,
                   "motion_blur": {"shutter": 180, "samples": 8}}),
        )
    };
    assert_eq!(of(&blurred(json!(10)), "R-MOTION-BLUR-STILL").len(), 1);
    let keyed = blurred(json!([{"t": 0, "v": 0}, {"t": 500, "v": 40, "ease": "linear"}]));
    assert!(
        of(&keyed, "R-MOTION-BLUR-STILL").is_empty(),
        "{:#?}",
        keyed.findings
    );
}

#[test]
fn a_motion_blur_sample_that_faces_away_is_transparent_and_the_divisor_stays_full() {
    // One 40 ms frame holds a turn through 90 degrees: about half of a 360-degree
    // shutter's samples face the eye. Each draws the white card; the rest draw nothing, and
    // the mean over all of them is a white card at about half strength, never a full one.
    let element = card(
        200,
        100,
        json!({"perspective": 2000, "swivel": [
                   {"t": 0, "v": 0}, {"t": 80, "v": 180, "ease": "linear"}],
               "motion_blur": {"shutter": 360, "samples": 16}}),
    );
    // The frame at 40 ms spans 20 to 60 ms of a turn that is at 45 degrees and at 135.
    let picture = painted(&project(640, 360, &[element]), 40);
    let centre = picture.get_pixel(320, 180).0[0];
    assert!(
        (60..=200).contains(&centre),
        "the centre is neither fully painted nor empty: {centre}"
    );
}

// ---- the renderer, validate, query and the contact sheet agree ------------------------

/// The pixel bounds of everything not black: `(left, top, right, bottom)`, right and bottom
/// exclusive.
fn lit_bounds(picture: &image::RgbaImage, threshold: u8) -> (u32, u32, u32, u32) {
    let (mut l, mut t, mut r, mut b) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, pixel) in picture.enumerate_pixels() {
        if pixel.0[0] > threshold {
            (l, t, r, b) = (l.min(x), t.min(y), r.max(x + 1), b.max(y + 1));
        }
    }
    (l, t, r, b)
}

/// **The conformance test** (ADR-0146's lesson about copied lists): where the renderer paints
/// a projected card, where `query --at` says its box is, and where `validate` says it never
/// is are one place, each read through its own verb and none through the others' code.
#[test]
fn the_renderer_query_and_validate_agree_on_where_a_projected_card_is() {
    let turned = |extra: Value| {
        card(220, 120, {
            let mut element = json!({"x": 300, "y": 200, "swivel": 50, "tilt": -30,
                    "perspective": 700, "rotation": 15, "scale": [1.2, 0.9]});
            for (key, value) in extra.as_object().expect("an object") {
                element[key] = value.clone();
            }
            element
        })
    };

    // Tight, with no effects: the painted bounds are the bounds query prints, to a pixel.
    let plain = project(640, 400, &[turned(json!({}))]);
    let ink = &viewed(&plain, 0)["ink_box"];
    let (x, y) = (number(&ink["x"]), number(&ink["y"]));
    let (w, h) = (number(&ink["width"]), number(&ink["height"]));
    let (l, t, r, b) = lit_bounds(&painted(&plain, 0), 0);
    assert!((f64::from(l) - x.floor()).abs() <= 1.0, "left {l} vs {x}");
    assert!((f64::from(t) - y.floor()).abs() <= 1.0, "top {t} vs {y}");
    assert!(
        (f64::from(r) - (x + w).ceil()).abs() <= 1.0,
        "right {r} vs {}",
        x + w
    );
    assert!(
        (f64::from(b) - (y + h).ceil()).abs() <= 1.0,
        "bottom {b} vs {}",
        y + h
    );

    // With effects: the box is widened by their reach, and nothing the renderer paints,
    // down to the faintest pixel of a blurred shadow, falls outside it.
    let shadowed = project(
        640,
        400,
        &[turned(json!({"effects": [
            {"name": "shadow", "dx": 24, "dy": 16, "radius": 12, "color": "#FFFFFF", "opacity": 1},
            {"name": "blur", "radius": 4}]}))],
    );
    let ink = &viewed(&shadowed, 0)["ink_box"];
    let (x, y) = (number(&ink["x"]), number(&ink["y"]));
    let (w, h) = (number(&ink["width"]), number(&ink["height"]));
    let (l, t, r, b) = lit_bounds(&painted(&shadowed, 0), 0);
    assert!(f64::from(l) >= x.floor() - 1.0 && f64::from(t) >= y.floor() - 1.0);
    assert!(f64::from(r) <= (x + w).ceil() + 1.0 && f64::from(b) <= (y + h).ceil() + 1.0);
    // And the effects did widen it: the shadowed box is larger than the bare one.
    assert!(w > number(&viewed(&plain, 0)["ink_box"]["width"]) + 20.0);

    // `validate`'s R-OFF-CANVAS reports the same rectangle for the same card moved off the
    // frame: a frame too small to meet it, so the finding names the box it computed.
    let aside = project(100, 100, &[turned(json!({"x": 700, "y": 700}))]);
    let off = validated(&aside);
    let found = of(&off, "R-OFF-CANVAS");
    assert_eq!(found.len(), 1, "{:#?}", off.findings);
    let moved = viewed(&aside, 0);
    let ink = &moved["ink_box"];
    let reported = |field: &str| found[0].fields[field].as_i64().expect("whole pixels");
    assert_eq!(reported("x"), number(&ink["x"]).floor() as i64);
    assert_eq!(reported("y"), number(&ink["y"]).floor() as i64);
    assert_eq!(
        reported("x") + reported("width"),
        (number(&ink["x"]) + number(&ink["width"])).ceil() as i64
    );
}

#[test]
fn the_contact_sheet_marks_the_three_fields_where_they_change() {
    use montagent_core::verbs::frame::{Ask as FrameAsk, frame};
    let element = card(
        200,
        100,
        json!({"swivel": [{"t": 0, "v": 0}, {"t": 400, "v": 30, "ease": "linear"},
                          {"t": 800, "v": 0, "ease": "linear"}],
               "tilt": [{"t": 0, "v": 0}, {"t": 400, "v": 10, "ease": "linear"},
                        {"t": 800, "v": 0, "ease": "linear"}],
               "perspective": [{"t": 0, "v": 900}, {"t": 400, "v": 1200, "ease": "linear"},
                               {"t": 800, "v": 900, "ease": "linear"}]}),
    );
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&project(320, 180, &[element]).to_string()),
    );
    let sheet = frame(
        &path,
        &FrameAsk {
            from: Some(0),
            to: Some(800),
            keyframes: true,
            ..FrameAsk::default()
        },
    )
    .to_json();
    let points: Vec<String> = sheet["sheet"]["provenance"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|tile| tile["keyframes"].as_array().into_iter().flatten())
        .filter_map(|point| point.as_str().map(String::from))
        .collect();
    for field in ["swivel", "tilt", "perspective"] {
        assert!(
            points
                .iter()
                .any(|p| p.starts_with(&format!("card.{field}@"))),
            "{field} has no change point on the sheet: {points:?}"
        );
    }
}

// ---- shift ---------------------------------------------------------------------------

#[test]
fn shift_moves_a_projected_elements_keyed_angles_and_perspective() {
    use montagent_core::verbs::shift::{Ask as ShiftAsk, shift};
    let element = card(
        200,
        100,
        json!({"swivel": [{"t": 100, "v": 0}, {"t": 600, "v": 40, "ease": "linear"}],
               "tilt": [{"t": 100, "v": 0}, {"t": 600, "v": 10, "ease": "linear"}],
               "perspective": [{"t": 100, "v": 1000}, {"t": 600, "v": 1400, "ease": "linear"}]}),
    );
    let dir = tempdir(std::panic::Location::caller().line());
    let path = write_project(
        &dir,
        "p.montagent.json",
        &canonical(&project(640, 360, &[element]).to_string()),
    );
    let answer = shift(
        &path,
        &ShiftAsk {
            at: 300,
            delta: 200,
            scope: None,
            release: Vec::new(),
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let text = std::fs::read_to_string(&path).expect("the file");
    let written: Value = serde_json::from_str(&text).expect("a project");
    let card = &written["tracks"][0]["elements"][0];
    // Every one of the three was split at 300 and shifted, as `x` and `opacity` are.
    for field in ["swivel", "tilt", "perspective"] {
        let ts: Vec<i64> = card[field]
            .as_array()
            .unwrap_or_else(|| panic!("{field} stays a keyframe list"))
            .iter()
            .map(|record| record["t"].as_i64().expect("a t"))
            .collect();
        assert!(ts.contains(&100), "{field}: {ts:?}");
        assert!(
            ts.contains(&300),
            "{field} is split where the shift cuts: {ts:?}"
        );
        assert!(
            ts.contains(&500),
            "{field} after the cut moves by 200: {ts:?}"
        );
        assert!(ts.contains(&800), "{field}: {ts:?}");
    }
}

// ---- the path an element takes -------------------------------------------------------

/// A white card whose rotation and scale make the resample show: an axis-aligned card at a
/// whole pixel resamples to itself, byte for byte.
fn turned_card(extra: Value) -> Value {
    let mut element = card(160, 90, json!({"rotation": 12, "scale": [1.1, 0.9]}));
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

#[track_caller]
fn bytes_of(element: Value, instant: i64) -> Vec<u8> {
    painted(&project(320, 200, &[element]), instant).into_raw()
}

#[test]
fn a_written_angle_projects_even_at_zero_so_a_keyed_swivel_crossing_zero_does_not_switch_path() {
    // `swivel` keyed from 0, so it is exactly 0 at instant 0 and 20 at instant 500.
    let keyed = bytes_of(
        turned_card(json!({"perspective": 900, "swivel": [
            {"t": 0, "v": 0}, {"t": 500, "v": 20, "ease": "linear"}]})),
        0,
    );
    let written_zero = bytes_of(turned_card(json!({"perspective": 900, "swivel": 0})), 0);
    let written_tilt_zero = bytes_of(turned_card(json!({"perspective": 900, "tilt": 0})), 0);
    let no_angle = bytes_of(turned_card(json!({})), 0);
    assert_eq!(
        keyed, written_zero,
        "a keyed swivel at exactly 0 paints what a static `swivel: 0` paints"
    );
    assert_eq!(written_zero, written_tilt_zero, "either angle projects");
    assert_ne!(
        keyed, no_angle,
        "a written angle differs from none only by the resample, and it does differ"
    );
}

#[test]
fn an_element_with_no_angle_paints_the_same_whatever_perspective_it_is_given() {
    // `perspective` alone is a dead value (`E-PROJECTION-PERSPECTIVE-ALONE` is what the
    // document is told); it must not move a byte.
    assert_eq!(
        bytes_of(turned_card(json!({})), 0),
        bytes_of(turned_card(json!({"perspective": 900})), 0)
    );
}
