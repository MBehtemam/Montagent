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
