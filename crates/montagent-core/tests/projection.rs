//! Projection: `swivel`, `tilt` and `perspective` on every visual element (#787, ADR-0167,
//! ADR-0168).

use std::path::PathBuf;

use montagent_core::finding::Class;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

/// The six visual element types, which all take the three fields.
const VISUAL: [&str; 6] = ["image", "video", "text", "rect", "ellipse", "path"];

// ---------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------

/// `base` with `fields` merged in; a `null` removes the key.
fn with(mut element: Value, fields: Value) -> Value {
    for (key, value) in fields.as_object().expect("fields are an object") {
        if value.is_null() {
            element.as_object_mut().unwrap().remove(key);
        } else {
            element[key] = value.clone();
        }
    }
    element
}

/// A white 400×250 card centred on a 1280×720 frame.
fn card(fields: Value) -> Value {
    with(
        json!({"id": "card", "type": "rect", "start": 0, "end": 1000, "x": 640, "y": 360,
            "origin": "center", "width": 400, "height": 250, "fill": "#FFFFFF"}),
        fields,
    )
}

/// A keyframe list from `from` at 0 ms to `to` at 1000 ms, linear.
fn keyed(from: f64, to: f64) -> Value {
    json!([{"t": 0, "v": from}, {"t": 1000, "v": to, "ease": "linear"}])
}

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_787_000 + NEXT.fetch_add(1, Ordering::Relaxed))
}

/// A 1280×720 project at 10 fps over black, one track holding `elements`.
fn project(elements: &[Value]) -> PathBuf {
    write_project(
        &scratch(),
        "p.json",
        &canonical(
            &json!({"frame": {"width": 1280, "height": 720}, "fps": 10,
                    "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": elements}]})
            .to_string(),
        ),
    )
}

/// Every finding `validate` gives `element` at `class`, as its code and its fields.
fn findings(element: Value, class: Class) -> Vec<(String, Value)> {
    montagent_core::validate(&project(&[element]))
        .findings
        .iter()
        .filter(|finding| finding.class == class)
        .map(|finding| {
            let fields: serde_json::Map<String, Value> = finding
                .fields
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect();
            (finding.code.clone(), fields.into())
        })
        .collect()
}

fn errors(element: Value) -> Vec<(String, Value)> {
    findings(element, Class::Error)
}

/// The fields of every `code` finding `validate` gives `element`, at any class.
fn fired(element: Value, code: &str) -> Vec<Value> {
    [Class::Error, Class::Review]
        .into_iter()
        .flat_map(|class| findings(element.clone(), class))
        .filter(|(fired, _)| fired == code)
        .map(|(_, fields)| fields)
        .collect()
}

/// `validate`'s prose for `element`'s project.
fn prose(element: Value) -> String {
    let report = montagent_core::validate(&project(&[element]));
    montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
        .expect("the report renders")
}

/// The codes of every projection finding `validate` gives `element`.
fn projection_codes(element: Value) -> Vec<String> {
    [Class::Error, Class::Review]
        .into_iter()
        .flat_map(|class| findings(element.clone(), class))
        .map(|(code, _)| code)
        .filter(|code| code.contains("PROJECTION"))
        .collect()
}

/// The one schema error `element` gets, whose text contains `says`.
#[track_caller]
fn schema_error(element: Value, says: &str) {
    let errors = errors(element);
    assert!(
        errors
            .iter()
            .any(|(code, fields)| code.starts_with("E-SCHEMA") && fields.to_string().contains(says)),
        "no schema error saying {says:?}: {errors:#?}"
    );
}

// ---------------------------------------------------------------------------
// The schema and the one list (ADR-0167 §1, ADR-0146).
// ---------------------------------------------------------------------------

#[test]
fn every_visual_element_takes_the_three_fields_on_the_one_list_after_rotation() {
    use montagent_core::animatable::{Kind, of};
    for kind in VISUAL {
        let names: Vec<&str> = of(kind).iter().map(|p| p.name.as_str()).collect();
        let at = names
            .iter()
            .position(|name| *name == "rotation")
            .unwrap_or_else(|| panic!("{kind}: {names:?}"));
        assert_eq!(
            &names[at..at + 4],
            ["rotation", "swivel", "tilt", "perspective"],
            "{kind}"
        );
        for name in ["swivel", "tilt", "perspective"] {
            let property = of(kind).iter().find(|p| p.name == name).unwrap();
            assert_eq!(property.kind, Kind::Number, "{kind}.{name}");
            assert_eq!(property.maximum, None, "{kind}.{name}");
        }
    }
    for kind in ["audio", "transition"] {
        assert!(
            of(kind)
                .iter()
                .all(|p| !matches!(p.name.as_str(), "swivel" | "tilt" | "perspective"))
        );
    }
}

#[test]
fn the_schema_states_css_signs_and_both_names_and_requires_perspective_with_an_angle() {
    let schema = montagent_core::schema::generate();
    for branch in schema["$defs"]["Element"]["oneOf"].as_array().unwrap() {
        let kind = branch["properties"]["type"]["const"].as_str().unwrap();
        if !VISUAL.contains(&kind) {
            assert!(branch["properties"].get("swivel").is_none(), "{kind}");
            continue;
        }
        let description = |key: &str| {
            branch["properties"][key]["description"]
                .as_str()
                .unwrap_or_else(|| panic!("{kind}.{key} has no description"))
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        };
        let swivel = description("swivel");
        for says in ["right edge away", "rotateY", "Swivel"] {
            assert!(swivel.contains(says), "{kind}.swivel: {swivel}");
        }
        let tilt = description("tilt");
        for says in ["top edge away", "rotateX", "Tilt"] {
            assert!(tilt.contains(says), "{kind}.tilt: {tilt}");
        }
        assert!(description("perspective").contains("px"), "{kind}");
        assert_eq!(
            branch["dependentRequired"],
            json!({"swivel": ["perspective"], "tilt": ["perspective"]}),
            "{kind}"
        );
    }
    let perspective = &schema["$defs"]["Perspective"];
    assert_eq!(perspective["type"], "number", "{perspective}");
    assert_eq!(perspective["exclusiveMinimum"], 0.0, "{perspective}");
}

#[test]
fn every_visual_element_validates_with_a_written_projection() {
    let projection = json!({"swivel": 30, "tilt": -10.5, "perspective": 2000});
    let elements = [
        card(projection.clone()),
        card(with(
            projection.clone(),
            json!({"type": "ellipse", "id": "oval", "fill": "#FF0000"}),
        )),
        with(
            json!({"id": "line", "type": "path", "start": 0, "end": 1000, "x": 640, "y": 360,
                "width": 200, "height": 200, "closed": false, "stroke": "#FFFFFF",
                "stroke_width": 4, "points": [{"at": [20, 100]}, {"at": [180, 100]}]}),
            projection.clone(),
        ),
        card(json!({"swivel": keyed(0.0, 180.0), "tilt": keyed(0.0, 0.0),
            "perspective": keyed(1500.0, 3000.0)})),
    ];
    for element in elements {
        assert_eq!(errors(element.clone()), [], "{element}");
    }
}

#[test]
fn perspective_is_a_positive_number_of_px_in_every_keyframe_value() {
    for value in [json!(0), json!(-200), keyed(1000.0, 0.0)] {
        schema_error(
            card(json!({"swivel": 20, "perspective": value})),
            "greater than 0",
        );
    }
    schema_error(
        card(json!({"swivel": 20, "perspective": "far"})),
        "expected f64",
    );
}

// ---------------------------------------------------------------------------
// `validate` (ADR-0167 §5, §8; ADR-0168 §3).
// ---------------------------------------------------------------------------

/// The card's eye bound about `center`: the distance to a corner of 400×250, √(200² + 125²).
const CARD_R: f64 = 235.849;

#[test]
fn an_angle_without_perspective_is_perspective_missing_naming_the_angle() {
    for angle in ["swivel", "tilt"] {
        let found = fired(card(json!({angle: 30})), "E-PROJECTION-PERSPECTIVE-MISSING");
        assert_eq!(found.len(), 1, "{angle}: {found:?}");
        let text = prose(card(json!({angle: keyed(0.0, 30.0)})));
        assert!(text.contains("`card`.perspective"), "{text}");
        assert!(text.contains(angle), "{text}");
    }
    // The near miss: the same angle with its `perspective`.
    assert_eq!(
        projection_codes(card(json!({"swivel": 30, "perspective": 2000}))),
        Vec::<String>::new()
    );
}

#[test]
fn perspective_with_no_angle_is_perspective_alone_and_a_zero_angle_counts() {
    let found = fired(
        card(json!({"perspective": 2000})),
        "E-PROJECTION-PERSPECTIVE-ALONE",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    let text = prose(card(json!({"perspective": 2000})));
    assert!(text.contains("`card`.perspective"), "{text}");
    assert!(text.contains("swivel") && text.contains("tilt"), "{text}");
    for angle in [json!(0), keyed(0.0, 0.0)] {
        for key in ["swivel", "tilt"] {
            let element = card(json!({key: angle.clone(), "perspective": 2000}));
            assert_eq!(projection_codes(element), Vec::<String>::new(), "{key}");
        }
    }
}

#[test]
fn a_perspective_at_or_inside_r_is_the_eye_error_naming_the_instant_r_and_the_value_that_passes() {
    let element = card(json!({"swivel": 20, "perspective": 230}));
    let found = fired(element.clone(), "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["instant"], json!(0));
    assert_eq!(found[0]["passing"], json!(236));
    assert!((found[0]["r"].as_f64().unwrap() - CARD_R).abs() < 0.01);
    let text = prose(element);
    for says in ["at 0 ms", "235.85", "236"] {
        assert!(text.contains(says), "{says}: {text}");
    }
    // The near miss: one past r passes the bound (and is soft, which is a review).
    let passing = card(json!({"swivel": 20, "perspective": 236}));
    assert!(fired(passing, "E-PROJECTION-EYE").is_empty());
}

#[test]
fn the_eye_bound_counts_a_shadows_reach_where_the_bare_box_would_pass() {
    // 300 clears the bare card's 235.85, but a shadow 60 px down and right with a 20 px
    // radius (⌈3σ⌉ = 30) pushes the bottom-right corner to (290, 215) from the centre.
    let shadow = json!({"name": "shadow", "dx": 60, "dy": 60, "radius": 20,
        "color": "#000000", "opacity": 0.5});
    let bare = card(json!({"swivel": 20, "perspective": 300}));
    assert!(fired(bare.clone(), "E-PROJECTION-EYE").is_empty());
    let found = fired(with(bare, json!({"effects": [shadow]})), "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{found:?}");
    let r = found[0]["r"].as_f64().unwrap();
    assert!((r - 290f64.hypot(215.0)).abs() < 0.01, "{r}");
    assert_eq!(found[0]["passing"], json!(362));
}

#[test]
fn the_eye_bound_is_evaluated_at_every_key_and_eased_extreme_of_perspective() {
    // A key inside the range.
    let element = card(json!({"swivel": 20, "perspective":
        [{"t": 0, "v": 2000}, {"t": 500, "v": 200, "ease": "linear"}]}));
    let found = fired(element.clone(), "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["instant"], json!(500));
    assert!(prose(element).contains("a key"));
    // Both keys pass, and the overshoot between them reaches 104 at 732.8 ms.
    let element = card(json!({"end": 2000, "swivel": 20, "perspective":
        [{"t": 0, "v": 1000}, {"t": 1000, "v": 300, "ease": [0.3, 0, 0.5, 2.0]}]}));
    let found = fired(element.clone(), "E-PROJECTION-EYE");
    assert_eq!(found.len(), 1, "{found:?}");
    let instant = found[0]["instant"].as_i64().unwrap();
    assert!((732..=733).contains(&instant), "{instant}");
    assert!(prose(element).contains("an eased extreme"));
}

#[test]
fn a_near_edge_magnified_past_two_is_soft_naming_the_magnification_and_the_perspective_for_two() {
    let element = card(json!({"swivel": 20, "perspective": 300}));
    let found = fired(element.clone(), "R-PROJECTION-SOFT");
    assert_eq!(found.len(), 1, "{found:?}");
    // d / (d − r) = 300 / 64.15.
    assert_eq!(found[0]["magnification"], json!(4.68));
    assert_eq!(found[0]["perspective"], json!(472));
    let text = prose(element);
    assert!(text.contains("4.68") && text.contains("472"), "{text}");
    // The near miss: 2r rounded up brings it back to 2×.
    assert!(
        fired(
            card(json!({"swivel": 20, "perspective": 472})),
            "R-PROJECTION-SOFT"
        )
        .is_empty()
    );
    // Inside the eye bound it is the error, and not also the review.
    assert!(
        fired(
            card(json!({"swivel": 20, "perspective": 230})),
            "R-PROJECTION-SOFT"
        )
        .is_empty()
    );
}

#[test]
fn an_element_that_never_faces_front_is_projection_away_and_a_flip_is_not() {
    for (swivel, tilt) in [
        (json!(180), json!(0)),
        (json!(90), json!(0)),
        (json!(0), json!(-120)),
    ] {
        let element = card(json!({"swivel": swivel, "tilt": tilt, "perspective": 2000}));
        let found = fired(element.clone(), "R-PROJECTION-AWAY");
        assert_eq!(found.len(), 1, "{element}: {found:?}");
    }
    for swivel in [
        json!(89),
        keyed(0.0, 180.0),
        keyed(-180.0, 0.0),
        // Both keys face away, and the turn between them passes the front.
        keyed(100.0, 460.0),
    ] {
        let element = card(json!({"swivel": swivel, "perspective": 2000}));
        assert!(
            fired(element.clone(), "R-PROJECTION-AWAY").is_empty(),
            "{element}"
        );
    }
}

// ---------------------------------------------------------------------------
// The checks read the projected bounds (ADR-0167 §6).
// ---------------------------------------------------------------------------

/// Whether `validate` says `element` is off canvas.
fn off_canvas(element: Value) -> bool {
    !fired(element, "R-OFF-CANVAS").is_empty()
}

#[test]
fn a_card_swung_out_of_frame_is_off_canvas_though_its_flat_box_is_on_it() {
    // Pivoting on its bottom-left corner 15 px below the frame, tilted almost edge-on and
    // swivelled so its right side falls: the flat box reaches 250 px up into the frame,
    // the projected card stays below it.
    let flat = card(json!({"origin": "bottom-left", "x": 1000, "y": 735}));
    assert!(!off_canvas(flat.clone()));
    let swung = with(flat, json!({"swivel": 30, "tilt": 88, "perspective": 2000}));
    assert!(off_canvas(swung));
}

#[test]
fn a_card_projected_into_the_frame_is_not_off_canvas_though_its_flat_box_is_off_it() {
    // Pivoting on its top-left corner 5 px below the frame: flat, the whole card is below
    // it; swivelled and tilted, its top-right corner rises 150 px into it.
    let flat = card(json!({"origin": "top-left", "x": 100, "y": 725}));
    assert!(off_canvas(flat.clone()));
    let projected = with(
        flat,
        json!({"swivel": 60, "tilt": -30, "perspective": 2000}),
    );
    assert!(!off_canvas(projected));
}

#[test]
fn a_layer_tie_is_decided_on_the_projected_bounds() {
    let pair = |left: Value, right: Value| {
        let report = montagent_core::validate(&write_project(
            &scratch(),
            "p.json",
            &canonical(
                &json!({"frame": {"width": 1280, "height": 720}, "fps": 10,
                    "background": "#000000",
                    "tracks": [{"name": "a", "layer": 0, "elements": [left]},
                               {"name": "b", "layer": 0, "elements": [right]}]})
                .to_string(),
            ),
        ));
        report.findings.iter().any(|f| f.code == "E-LAYER-TIE")
    };
    // Two cards on one layer whose flat boxes overlap at one card's bottom-right corner, by
    // 20 × 20 px. Swivelled 40° about its top-left corner, that card's right edge recedes to
    // about x 371, clear of the other.
    let a = card(json!({"id": "a", "origin": "top-left", "x": 100, "y": 100}));
    let b = card(json!({"id": "b", "origin": "top-left", "x": 480, "y": 330}));
    assert!(pair(a.clone(), b.clone()));
    let swung = with(a.clone(), json!({"swivel": 40, "perspective": 2000}));
    assert!(!pair(swung, b));
    // The other way: flat, a card 20 px above `a` misses it; projected, `a`'s top-right
    // corner rises 150 px into it.
    let above = card(
        json!({"id": "above", "origin": "top-left", "x": 100, "y": 0,
        "height": 80}),
    );
    let a = with(a, json!({"y": 100}));
    assert!(!pair(a.clone(), above.clone()));
    let lifted = with(a, json!({"swivel": 60, "tilt": -30, "perspective": 2000}));
    assert!(pair(lifted, above));
}

// ---------------------------------------------------------------------------
// `query --at` (ADR-0167 §7).
// ---------------------------------------------------------------------------

/// The `query --at` answer's row for `id` at `instant`.
fn at(element: Value, instant: i64) -> Value {
    use montagent_core::verbs::query::{self, Ask};
    let id = element["id"].clone();
    let answer = query::query(
        &project(&[element]),
        &Ask {
            at: Some(instant),
            ..Ask::default()
        },
    )
    .to_json()["query"]
        .clone();
    answer["stack"]
        .as_array()
        .expect("a stack")
        .iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("{id} is not in {answer}"))
        .clone()
}

#[test]
fn query_at_prints_the_three_numbers_the_facing_and_the_corners_and_the_ink_box_bounds_them() {
    let row = at(
        card(json!({"swivel": 35, "tilt": 20, "perspective": 900})),
        0,
    );
    let projection = &row["projection"];
    assert_eq!(projection["swivel"], json!(35.0));
    assert_eq!(projection["tilt"], json!(20.0));
    assert_eq!(projection["perspective"], json!(900.0));
    assert_eq!(projection["facing"], json!("front"));
    // The prototype's case 1, which its report checked against painted pixels.
    let expected = [
        [463.41, 191.10],
        [780.35, 292.98],
        [792.79, 506.14],
        [443.26, 453.94],
    ];
    let corners = projection["corners"].as_array().expect("four corners");
    for (corner, want) in corners.iter().zip(expected) {
        for axis in 0..2 {
            let got = corner[axis].as_f64().unwrap();
            assert!((got - want[axis]).abs() < 0.006, "{corners:?}");
        }
    }
    let ink = &row["ink_box"];
    assert!((ink["x"].as_f64().unwrap() - 443.26).abs() < 0.006, "{ink}");
    assert!((ink["y"].as_f64().unwrap() - 191.10).abs() < 0.006, "{ink}");
    assert!((ink["width"].as_f64().unwrap() - (792.79 - 443.26)).abs() < 0.01);
    assert!((ink["height"].as_f64().unwrap() - (506.14 - 191.10)).abs() < 0.01);
    // An element that writes neither angle reports no projection.
    assert!(at(card(json!({})), 0).get("projection").is_none());
}

#[test]
fn facing_is_edge_at_exactly_90_and_away_past_it_and_the_ink_box_is_empty_with_corners() {
    let flip = card(json!({"swivel": keyed(0.0, 180.0), "perspective": 2000}));
    let row = at(flip.clone(), 500);
    assert_eq!(row["projection"]["facing"], json!("edge"));
    assert!(row["ink_box"].is_null());
    assert!(
        row["ink_box_unresolved"]
            .as_str()
            .unwrap()
            .contains("edge-on")
    );
    let row = at(flip.clone(), 600);
    assert_eq!(row["projection"]["facing"], json!("away"));
    assert!(row["ink_box"].is_null());
    assert!(
        row["ink_box_unresolved"]
            .as_str()
            .unwrap()
            .contains("faces away")
    );
    assert_eq!(
        row["projection"]["corners"].as_array().map(Vec::len),
        Some(4)
    );
    assert_eq!(at(flip, 400)["projection"]["facing"], json!("front"));
}

// ---------------------------------------------------------------------------
// `shift` (ADR-0167 §9): the three fields split as plain animatable numbers.
// ---------------------------------------------------------------------------

#[test]
fn shift_splits_a_projected_element_with_keyed_angles_and_perspective() {
    let path = project(&[card(json!({"swivel": keyed(0.0, 180.0),
        "tilt": keyed(-20.0, 20.0), "perspective": keyed(1000.0, 3000.0)}))]);
    let answer = montagent_core::verbs::shift::shift(
        &path,
        &montagent_core::verbs::shift::Ask {
            at: 500,
            delta: 100,
            scope: None,
            release: Vec::new(),
        },
    );
    assert_eq!(
        answer.report().exit_code(),
        montagent_core::report::ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let written = common::document(&path);
    let element = &written["tracks"][0]["elements"][0];
    let records = |key: &str| -> Vec<(i64, f64)> {
        element[key]
            .as_array()
            .unwrap_or_else(|| panic!("{key}: {element}"))
            .iter()
            .map(|record| (record["t"].as_i64().unwrap(), record["v"].as_f64().unwrap()))
            .collect()
    };
    assert_eq!(
        records("swivel"),
        [(0, 0.0), (500, 90.0), (600, 90.0), (1100, 180.0)]
    );
    assert_eq!(
        records("tilt"),
        [(0, -20.0), (500, 0.0), (600, 0.0), (1100, 20.0)]
    );
    assert_eq!(
        records("perspective"),
        [(0, 1000.0), (500, 2000.0), (600, 2000.0), (1100, 3000.0)]
    );
    assert_eq!(element["end"], json!(1100));
    assert_eq!(errors(element.clone()), []);
}

// ---------------------------------------------------------------------------
// The painter (ADR-0167 §3, ADR-0168 §1).
// ---------------------------------------------------------------------------

/// The frame `elements` paint at `instant`, as PNG bytes at true scale.
fn png_at(elements: &[Value], instant: i64) -> Vec<u8> {
    use montagent_core::verbs::frame::{Ask, frame};
    let answer = frame(
        &project(elements),
        &Ask {
            at: Some(instant),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    answer.image().expect("a picture").bytes.clone()
}

#[test]
fn a_keyed_swivel_passing_through_zero_takes_the_projected_path_on_that_frame() {
    // A rotated, scaled card, so the projected path's resample shows (ADR-0168 §1).
    let base = card(json!({"rotation": 12, "scale": [1.1, 0.9]}));
    let crossing = with(
        base.clone(),
        json!({"swivel": keyed(-30.0, 30.0), "perspective": 2000}),
    );
    let still_zero = with(base.clone(), json!({"swivel": 0, "perspective": 2000}));
    let at_zero = png_at(&[crossing], 500);
    assert_eq!(
        at_zero,
        png_at(&[still_zero], 500),
        "the same projected path"
    );
    assert_ne!(at_zero, png_at(&[base], 500), "not the unprojected path");
}

// ---------------------------------------------------------------------------
// Conformance (ADR-0167 §6): the renderer, `validate`, `query --at` and the contact sheet
// read one geometry. ADR-0146's lesson about copied lists, applied to a function.
// ---------------------------------------------------------------------------

/// `elements`' project, written beside the trailer's title face so a `text` can name it.
fn project_with_font(elements: &[Value]) -> PathBuf {
    let dir = scratch();
    let font = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/benchmark/spy-trailer/fonts/Cinzel-Bold.ttf");
    std::fs::create_dir_all(dir.join("fonts")).unwrap();
    std::fs::copy(font, dir.join("fonts/Cinzel-Bold.ttf")).unwrap();
    write_project(
        &dir,
        "p.json",
        &canonical(
            &json!({"frame": {"width": 1280, "height": 720}, "fps": 10,
                "background": "#000000",
                "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
                "fontVendor": {"fonts/Cinzel-Bold.ttf": {
                    "licence": "OFL-1.1",
                    "source": "google/fonts ofl/cinzel, instanced wght=700",
                    "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
                "tracks": [{"name": "only", "layer": 0, "elements": elements}]})
            .to_string(),
        ),
    )
}

/// The bounding box of every pixel the frame at `instant` paints off the black background,
/// as `[left, top, right, bottom]` in pixel edges, or `None` where it paints none.
fn painted_box(path: &std::path::Path, instant: i64) -> Option<[f64; 4]> {
    use montagent_core::verbs::frame::{Ask, frame};
    let answer = frame(
        path,
        &Ask {
            at: Some(instant),
            full: true,
            png: true,
            ..Ask::default()
        },
    );
    let picture = image::load_from_memory(&answer.image().expect("a picture").bytes)
        .expect("a PNG")
        .to_rgba8();
    let mut found: Option<[f64; 4]> = None;
    for (x, y, pixel) in picture.enumerate_pixels() {
        if pixel.0[..3] != [0, 0, 0] {
            let (x, y) = (f64::from(x), f64::from(y));
            found = Some(match found {
                None => [x, y, x + 1.0, y + 1.0],
                Some([l, t, r, b]) => [l.min(x), t.min(y), r.max(x + 1.0), b.max(y + 1.0)],
            });
        }
    }
    found
}

#[test]
fn the_renderer_validate_query_and_the_contact_sheet_agree_on_every_projected_element() {
    use montagent_core::verbs::query::{self, Ask};
    let shadow = json!({"name": "shadow", "dx": 12, "dy": 16, "radius": 10,
        "color": "#FFFFFF", "opacity": 0.5});
    let flip = card(
        json!({"id": "flip", "perspective": 1500, "effects": [shadow],
        "swivel": [{"t": 300, "v": 0}, {"t": 700, "v": 180, "ease": "linear"}]}),
    );
    let oval = card(json!({"id": "oval", "type": "ellipse", "fill": "#FF0000",
        "tilt": keyed(-60.0, 60.0), "swivel": 20, "perspective": 1200, "rotation": 15,
        "scale": [1.2, 0.9]}));
    let star = json!({"id": "star", "type": "path", "start": 0, "end": 1000, "x": 300,
        "y": 300, "origin": "top-left", "width": 200, "height": 200, "closed": true,
        "fill": "#FFD60A", "tilt": 40, "perspective": 900,
        "points": [{"at": [100, 10]}, {"at": [190, 190]}, {"at": [10, 190]}]});
    let gone = card(
        json!({"id": "gone", "origin": "bottom-left", "x": 1000, "y": 735,
        "swivel": 30, "tilt": 88, "perspective": 2000}),
    );
    let away = card(json!({"id": "away", "swivel": 180, "perspective": 2000}));
    let title = json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 60,
        "color": "#FFFFFF", "align": "center", "runs": [{"text": "SPY"}], "width": 300,
        "height": 100, "start": 0, "end": 1000, "x": 640, "y": 360, "origin": "center",
        "caption": false, "swivel": keyed(-50.0, 50.0), "perspective": 1000});

    for element in [flip, oval, star, gone, away, title] {
        let id = element["id"].as_str().unwrap().to_string();
        let path = project_with_font(std::slice::from_ref(&element));
        let report = montagent_core::validate(&path);
        let fires = |code: &str| report.findings.iter().any(|finding| finding.code == code);
        let mut fronts = 0;
        let mut on_canvas = false;
        for instant in (0..1000).step_by(100) {
            let answer = query::query(
                &path,
                &Ask {
                    at: Some(instant),
                    ..Ask::default()
                },
            )
            .to_json()["query"]
                .clone();
            let row = &answer["stack"][0];
            let facing = row["projection"]["facing"].as_str().unwrap();
            let painted = painted_box(&path, instant);
            if facing != "front" {
                // Away or edge-on: nothing painted, and the ink box is empty.
                assert_eq!(painted, None, "{id} at {instant}: {facing}");
                assert!(row["ink_box"].is_null(), "{id} at {instant}");
                continue;
            }
            fronts += 1;
            let ink = &row["ink_box"];
            let [x, y, w, h] = ["x", "y", "width", "height"].map(|key| ink[key].as_f64().unwrap());
            let frame = [0.0, 0.0, 1280.0, 720.0];
            on_canvas |= x < frame[2] && x + w > frame[0] && y < frame[3] && y + h > frame[1];
            // Everything the renderer paints lies inside the bounds `query --at` prints.
            if let Some([l, t, r, b]) = painted {
                assert!(
                    l >= x.floor() - 1.0
                        && t >= y.floor() - 1.0
                        && r <= (x + w).ceil() + 1.0
                        && b <= (y + h).ceil() + 1.0,
                    "{id} at {instant}: painted {:?} outside the ink box {ink}",
                    [l, t, r, b]
                );
            } else {
                // Nothing on the frame: the quadrilateral is off it.
                assert!(
                    !(x < frame[2] && x + w > frame[0] && y < frame[3] && y + h > frame[1]),
                    "{id} at {instant}: {ink}"
                );
            }
        }
        // `validate` reads the same facing and the same bounds.
        assert_eq!(fires("R-PROJECTION-AWAY"), fronts == 0, "{id}");
        assert_eq!(fires("R-OFF-CANVAS"), fronts > 0 && !on_canvas, "{id}");
    }

    // The contact sheet counts the flip's two angle keys as change points.
    let path = project_with_font(&[card(json!({"id": "flip", "perspective": 1500,
        "swivel": [{"t": 300, "v": 0}, {"t": 700, "v": 180, "ease": "linear"}]}))]);
    let sheet = montagent_core::verbs::frame::frame(
        &path,
        &montagent_core::verbs::frame::Ask {
            from: Some(0),
            to: Some(1000),
            keyframes: true,
            ..montagent_core::verbs::frame::Ask::default()
        },
    )
    .to_json();
    let points: Vec<String> = sheet["sheet"]["provenance"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|tile| tile["keyframes"].as_array().cloned().unwrap_or_default())
        .filter_map(|point| point.as_str().map(str::to_string))
        .collect();
    for point in ["flip.swivel@300", "flip.swivel@700"] {
        assert!(points.contains(&point.to_string()), "{point}: {sheet}");
    }
}

#[test]
fn nothing_is_painted_while_the_card_faces_away_or_is_edge_on() {
    let empty = png_at(&[], 0);
    let flip = card(json!({"swivel": keyed(0.0, 180.0), "perspective": 2000}));
    assert_eq!(
        png_at(std::slice::from_ref(&flip), 500),
        empty,
        "edge-on at 90°"
    );
    assert_eq!(
        png_at(std::slice::from_ref(&flip), 600),
        empty,
        "away past 90°"
    );
    assert_ne!(png_at(&[flip], 400), empty, "front before 90°");
}
