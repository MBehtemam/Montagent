//! The projection's arithmetic and its paint (ADR-0167, ADR-0168): the facing, the corners in
//! CSS's angle order, and the painted quadrilateral those corners describe.

use montagent_render::canvas::{
    Blend, Canvas, Effect, Extent, Facing, Fill, Ink, Projection, Rgba, Shape, Transform, quad,
};

const CARD: Extent = Extent {
    width: 400.0,
    height: 250.0,
};

fn transform(x: f64, y: f64, origin: (f64, f64), projection: Option<Projection>) -> Transform {
    Transform {
        x,
        y,
        origin,
        scale: (1.0, 1.0),
        rotation: 0.0,
        opacity: 1.0,
        blend: Blend::Normal,
        projection,
        nest: None,
    }
}

fn turned(swivel: f64, tilt: f64, perspective: f64) -> Projection {
    Projection {
        swivel,
        tilt,
        perspective,
    }
}

/// The five cases of the accepted prototype's corner table (#786's report), each with its
/// corners as the report prints them, to two decimals: TL, TR, BR, BL.
/// One case: its name, its transform, and the four corners the report prints.
type Case = (&'static str, Transform, [(f64, f64); 4]);

fn prototype_cases() -> Vec<Case> {
    let mut rotated = transform(640.0, 360.0, (0.5, 0.5), Some(turned(40.0, -25.0, 1000.0)));
    rotated.rotation = 30.0;
    rotated.scale = (1.3, 0.8);
    vec![
        (
            "centre, swivel 35, tilt 20, d 900",
            transform(640.0, 360.0, (0.5, 0.5), Some(turned(35.0, 20.0, 900.0))),
            [
                (463.41, 191.10),
                (780.35, 292.98),
                (792.79, 506.14),
                (443.26, 453.94),
            ],
        ),
        (
            "door at center-left, swivel -60, d 700",
            transform(400.0, 360.0, (0.0, 0.5), Some(turned(-60.0, 0.0, 700.0))),
            [
                (400.00, 235.00),
                (795.94, 112.54),
                (795.94, 607.46),
                (400.00, 485.00),
            ],
        ),
        (
            "top-left, tilt 50, d 800",
            transform(300.0, 150.0, (0.0, 0.0), Some(turned(0.0, 50.0, 800.0))),
            [
                (300.00, 150.00),
                (700.00, 150.00),
                (825.89, 361.27),
                (300.00, 361.27),
            ],
        ),
        (
            "eye bound + 1 (r 235.8, d 236), swivel 40",
            transform(640.0, 360.0, (0.5, 0.5), Some(turned(40.0, 0.0, 236.0))),
            [
                (303.47, 85.43),
                (739.18, 279.08),
                (739.18, 440.92),
                (303.47, 634.57),
            ],
        ),
        (
            "swivel 40, tilt -25, d 1000, then rotation 30 and scale [1.3, 0.8]",
            rotated,
            [
                (460.74, 190.94),
                (865.19, 344.45),
                (767.34, 480.10),
                (384.17, 377.67),
            ],
        ),
    ]
}

#[test]
fn the_corners_match_the_prototypes_table_in_css_angle_order() {
    for (name, transform, expected) in prototype_cases() {
        let projection = transform.projection.unwrap();
        let found = quad(CARD, &transform, projection, &[]);
        assert_eq!(found.facing, Facing::Front, "{name}");
        assert!(found.drawn, "{name}");
        for (corner, (got, want)) in found.corners.iter().zip(expected).enumerate() {
            assert!(
                (got.0 - want.0).abs() < 0.006 && (got.1 - want.1).abs() < 0.006,
                "{name}, corner {corner}: {got:?}, not {want:?}"
            );
        }
    }
}

#[test]
fn tilt_applies_in_the_swivelled_frame_so_the_order_shows_when_both_are_set() {
    // CSS `rotateX(t) rotateY(s)`: the swivel turns the element first. The other order puts
    // the top-left corner of case 1 somewhere else, so the table above pins the order.
    let t = transform(640.0, 360.0, (0.5, 0.5), Some(turned(35.0, 20.0, 900.0)));
    let found = quad(CARD, &t, t.projection.unwrap(), &[]).corners[0];
    // Tilt first, then swivel, computed by hand: R_y(s) · R_x(t) on (−200, −125, 0).
    let (s, tt) = (35f64.to_radians(), 20f64.to_radians());
    let (x, y) = (-200.0, -125.0);
    let (xt, yt, zt) = (x, y * tt.cos(), y * tt.sin());
    let (xs, zs) = (xt * s.cos() + zt * s.sin(), -xt * s.sin() + zt * s.cos());
    let w = 1.0 - zs / 900.0;
    let other = (640.0 + xs / w, 360.0 + yt / w);
    assert!(
        (found.0 - other.0).abs() > 1.0 || (found.1 - other.1).abs() > 1.0,
        "{found:?} vs {other:?}"
    );
}

#[test]
fn the_eye_bound_of_a_full_hd_element_is_the_formats_1102_and_1994() {
    let hd = Extent {
        width: 1920.0,
        height: 1080.0,
    };
    let centre = montagent_render::canvas::eye_bound(hd, (0.5, 0.5), &[]);
    let hinge = montagent_render::canvas::eye_bound(hd, (0.0, 0.5), &[]);
    assert_eq!(centre.ceil(), 1102.0, "{centre}");
    assert_eq!(hinge.ceil(), 1995.0, "{hinge}");
    assert!((hinge - 1994.5).abs() < 0.1, "{hinge}");
}

#[test]
fn the_facing_is_edge_at_exactly_90_and_away_past_it() {
    let facing = |swivel: f64, tilt: f64| turned(swivel, tilt, 1000.0).facing();
    assert_eq!(facing(0.0, 0.0), Facing::Front);
    assert_eq!(facing(89.9, 0.0), Facing::Front);
    assert_eq!(facing(90.0, 0.0), Facing::Edge);
    assert_eq!(facing(-90.0, 0.0), Facing::Edge);
    assert_eq!(facing(0.0, 270.0), Facing::Edge);
    assert_eq!(facing(90.1, 0.0), Facing::Away);
    assert_eq!(facing(180.0, 0.0), Facing::Away);
    assert_eq!(facing(0.0, -135.0), Facing::Away);
    // Two backs make a front: the plane turned over twice.
    assert_eq!(facing(180.0, 180.0), Facing::Front);
    // Never normalised, and 360 is a whole turn.
    assert_eq!(facing(360.0, 0.0), Facing::Front);
    assert_eq!(facing(450.0, 0.0), Facing::Edge);
    assert_eq!(facing(-200.0, 0.0), Facing::Away);
}

fn white() -> Fill {
    Fill {
        fill: Some(Ink::Flat(Rgba([255, 255, 255, 255]))),
        stroke: None,
        stroke_width: 0.0,
    }
}

/// `t`'s white card painted on a black 1280×720 frame, as RGBA bytes.
fn painted(t: &Transform, effects: &[Effect]) -> Vec<u8> {
    let mut canvas = Canvas::new(1280, 720).unwrap();
    canvas.background(Rgba([0, 0, 0, 255]));
    canvas.shape(
        Shape::Rect { radius: 0.0 },
        CARD,
        t,
        &white(),
        None,
        None,
        None,
        effects,
    );
    canvas.rgba().unwrap()
}

#[test]
fn the_corners_are_the_painted_quadrilateral_on_hand_checked_pixels() {
    // 3 px inside each corner, toward the quadrilateral's centroid, is the card; 3 px outside
    // is the background.
    for (name, t, _) in prototype_cases() {
        let rgba = painted(&t, &[]);
        let at = |x: f64, y: f64| {
            let (x, y) = (x.round() as i64, y.round() as i64);
            ((0..1280).contains(&x) && (0..720).contains(&y))
                .then(|| rgba[((y * 1280 + x) * 4) as usize])
        };
        let corners = quad(CARD, &t, t.projection.unwrap(), &[]).corners;
        let (cx, cy) = corners
            .iter()
            .fold((0.0, 0.0), |(a, b), (x, y)| (a + x / 4.0, b + y / 4.0));
        for (x, y) in corners {
            let (dx, dy) = (cx - x, cy - y);
            let length = dx.hypot(dy);
            let (ux, uy) = (dx / length, dy / length);
            assert_eq!(at(x + 3.0 * ux, y + 3.0 * uy), Some(255), "{name}: in");
            assert!(
                matches!(at(x - 3.0 * ux, y - 3.0 * uy), Some(0) | None),
                "{name}: out"
            );
        }
    }
}

#[test]
fn nothing_is_painted_facing_away_or_edge_on() {
    let black = painted(&transform(0.0, 0.0, (0.0, 0.0), None), &[]);
    let black: Vec<u8> = {
        // The same frame with nothing on it.
        let mut canvas = Canvas::new(1280, 720).unwrap();
        canvas.background(Rgba([0, 0, 0, 255]));
        let empty = canvas.rgba().unwrap();
        assert_ne!(black, empty, "the unprojected card paints");
        empty
    };
    for (swivel, tilt) in [(90.0, 0.0), (0.0, -90.0), (120.0, 0.0), (180.0, 10.0)] {
        let t = transform(640.0, 360.0, (0.5, 0.5), Some(turned(swivel, tilt, 1000.0)));
        let found = quad(CARD, &t, t.projection.unwrap(), &[]);
        assert!(!found.drawn, "{swivel}, {tilt}");
        assert_eq!(painted(&t, &[]), black, "{swivel}, {tilt}");
    }
}

#[test]
fn a_projected_card_with_effects_paints_nothing_outside_its_corners_bounds() {
    let effects = [
        Effect::Shadow {
            dx: 30.0,
            dy: 40.0,
            radius: 24.0,
            colour: Rgba([255, 0, 0, 255]),
            opacity: 1.0,
        },
        Effect::Blur { radius: 6.0 },
    ];
    let t = transform(640.0, 360.0, (0.5, 0.5), Some(turned(30.0, 15.0, 1500.0)));
    let found = quad(CARD, &t, t.projection.unwrap(), &effects);
    assert!(found.drawn);
    let [l, top, r, b] = found.bounds();
    let rgba = painted(&t, &effects);
    let mut inked = 0;
    for y in 0..720i64 {
        for x in 0..1280i64 {
            let i = ((y * 1280 + x) * 4) as usize;
            if rgba[i..i + 3] != [0, 0, 0] {
                inked += 1;
                assert!(
                    (x as f64) >= l.floor() - 1.0
                        && (x as f64) <= r.ceil() + 1.0
                        && (y as f64) >= top.floor() - 1.0
                        && (y as f64) <= b.ceil() + 1.0,
                    "ink at ({x}, {y}) outside [{l}, {top}, {r}, {b}]"
                );
            }
        }
    }
    assert!(inked > 10_000);
}
