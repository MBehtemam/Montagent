//! A path mask (#768, ADR-0163): `shape: "path"` with its `points` written inline, measured
//! from the mask's rect, always closed, keeping its interior by the nonzero rule.
//!
//! Asserted at the seams a path mask is used through: the model's parse (what is a schema
//! error), `validate`'s report (the point checks on a mask), the `frame` verb's pixels,
//! `query --at` and `shift`. Byte identity across painters and with the blur bounds hint on
//! and off lives beside the other gating runs, in `tests/painters.rs`.

use std::path::{Path, PathBuf};

use montagent_core::finding::Class;
use montagent_core::model::Effect;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

fn parse(text: &str) -> Result<Effect, serde_json::Error> {
    serde_json::from_str::<Effect>(text)
}

// ---- the schema ----------------------------------------------------------------------

#[test]
fn a_path_mask_parses_and_writes_back_in_the_canonical_key_order() {
    for written in [
        r##"{"name":"mask","shape":"path","points":[{"at":[10,90]},{"at":[50,10]},{"at":[90,90]}]}"##,
        r##"{"name":"mask","shape":"path","x":1,"y":2,"width":30,"height":40,"points":[{"at":[0,0]},{"at":[30,0],"out":[0,5]},{"at":[30,40],"in":[-2,0]}],"invert":true,"feather":6}"##,
    ] {
        let parsed = parse(written).unwrap_or_else(|e| panic!("{written}: {e}"));
        assert_eq!(
            serde_json::to_string(&parsed).expect("it serialises"),
            written
        );
    }
}

/// The parse error `text` gets, as text.
#[track_caller]
fn refused(text: &str) -> String {
    match parse(text) {
        Ok(parsed) => panic!("{text} parsed as {parsed:?}"),
        Err(e) => e.to_string(),
    }
}

const TRIANGLE: &str = r##"[{"at":[10,90]},{"at":[50,10]},{"at":[90,90]}]"##;

#[test]
fn points_is_required_under_path_and_an_unknown_key_under_every_other_shape() {
    let missing = refused(r##"{"name":"mask","shape":"path"}"##);
    assert!(missing.contains("`points`"), "{missing}");
    for shape in ["circle", "rect", "ellipse"] {
        let stray = refused(&format!(
            r##"{{"name":"mask","shape":"{shape}","points":{TRIANGLE}}}"##
        ));
        assert!(stray.contains("unknown field `points`"), "{shape}: {stray}");
    }
}

#[test]
fn radius_is_an_unknown_key_under_path() {
    let stray = refused(&format!(
        r##"{{"name":"mask","shape":"path","radius":4,"points":{TRIANGLE}}}"##
    ));
    assert!(stray.contains("unknown field `radius`"), "{stray}");
}

#[test]
fn closed_under_path_is_refused_with_the_adrs_own_words() {
    for closed in ["true", "false"] {
        let stray = refused(&format!(
            r##"{{"name":"mask","shape":"path","closed":{closed},"points":{TRIANGLE}}}"##
        ));
        assert!(
            stray.contains("a mask path always closes; drop `closed`."),
            "{stray}"
        );
    }
    // Under the other shapes it is a plain unknown key.
    let elsewhere = refused(r##"{"name":"mask","shape":"circle","closed":true}"##);
    assert!(elsewhere.contains("unknown field `closed`"), "{elsewhere}");
}

#[test]
fn a_keyed_width_or_height_under_path_is_refused_and_a_keyed_x_or_y_is_not() {
    let keyed = r##"[{"t":0,"v":100},{"t":500,"v":120,"ease":"linear"}]"##;
    for (width, height) in [(keyed, "100"), ("100", keyed)] {
        let message = refused(&format!(
            r##"{{"name":"mask","shape":"path","x":0,"y":0,"width":{width},"height":{height},"points":{TRIANGLE}}}"##
        ));
        assert!(message.contains("editing its `points`"), "{message}");
    }
    // The same keyed side is legal on the other shapes, and a keyed `x`/`y` under `path`.
    parse(&format!(
        r##"{{"name":"mask","shape":"ellipse","x":0,"y":0,"width":{keyed},"height":100}}"##
    ))
    .expect("a keyed width on an ellipse mask");
    parse(&format!(
        r##"{{"name":"mask","shape":"path","x":{keyed},"y":{keyed},"width":100,"height":100,"points":{TRIANGLE}}}"##
    ))
    .expect("a keyed x and y on a path mask");
}

#[test]
fn the_rect_is_still_all_or_none_under_path() {
    let partial = refused(&format!(
        r##"{{"name":"mask","shape":"path","x":0,"y":0,"points":{TRIANGLE}}}"##
    ));
    assert!(partial.contains("all-or-none"), "{partial}");
}

#[test]
fn the_published_schema_gates_points_on_path_and_holds_its_rect_static() {
    let schema = montagent_core::schema::generate();
    let mask = schema["$defs"]["Effect"]["oneOf"]
        .as_array()
        .expect("the effect union")
        .iter()
        .find(|branch| branch["properties"]["name"]["const"] == "mask")
        .expect("a mask branch")
        .clone();
    assert_eq!(
        schema["$defs"]["MaskShape"]["enum"],
        json!(["circle", "rect", "ellipse", "path"])
    );
    let keys: Vec<&String> = mask["properties"]
        .as_object()
        .expect("properties")
        .keys()
        .collect();
    assert_eq!(
        keys,
        [
            "shape", "x", "y", "width", "height", "radius", "points", "invert", "feather", "name"
        ]
    );
    assert_eq!(
        mask["properties"]["points"]["$ref"],
        json!("#/$defs/AnimatablePoints")
    );
    let gate = mask["allOf"]
        .as_array()
        .expect("the path gate")
        .iter()
        .find(|rule| rule["if"]["properties"]["shape"]["const"] == "path")
        .expect("a conditional on `path`")
        .clone();
    assert_eq!(gate["then"]["required"], json!(["points"]));
    for (index, side) in ["width", "height"].into_iter().enumerate() {
        assert_eq!(
            gate["then"]["allOf"][index],
            json!({"if": {"properties": {side: {"type": "array"}}, "required": [side]},
                   "then": false}),
            "{side} is static under `path`"
        );
    }
    assert_eq!(gate["else"]["not"], json!({"required": ["points"]}));
    // `radius` stays a field of `rect` alone, which already refuses it under `path`.
    assert_eq!(mask["if"]["properties"]["shape"]["const"], "rect");
    assert_eq!(mask["else"]["not"]["required"], json!(["radius"]));
}

// ---- `validate`: the point checks on a mask ------------------------------------------

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_000_000 + NEXT.fetch_add(1, Ordering::Relaxed))
}

/// A 200×200 project at 10 fps over black, one track holding `elements`.
fn project(dir: &Path, elements: &[Value]) -> PathBuf {
    write_project(
        dir,
        "p.json",
        &canonical(
            &json!({"frame": {"width": 200, "height": 200}, "fps": 10, "background": "#000000",
                    "tracks": [{"name": "only", "layer": 0, "elements": elements}]})
            .to_string(),
        ),
    )
}

/// A 100×100 white `rect` centred on the 200×200 frame, carrying `effects`.
fn plate(effects: Value) -> Value {
    json!({"id": "plate", "type": "rect", "start": 0, "end": 1000, "x": 100, "y": 100,
           "origin": "center", "width": 100, "height": 100, "fill": "#FFFFFF",
           "effects": effects})
}

/// A path mask over `points`, with `fields` laid over it.
fn path_mask(points: Value, fields: Value) -> Value {
    let mut mask = json!({"name": "mask", "shape": "path", "points": points});
    for (key, value) in fields.as_object().expect("fields are an object") {
        mask[key] = value.clone();
    }
    mask
}

fn triangle() -> Value {
    json!([{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}])
}

/// Every finding `validate` gives `element`, as `(code, class, fields)`.
fn validated(element: Value) -> Vec<(String, Class, Value)> {
    let report = montagent_core::validate(&project(&scratch(), &[element]));
    report
        .findings
        .iter()
        .map(|finding| {
            (
                finding.code.clone(),
                finding.class,
                serde_json::to_value(&finding.fields).expect("fields serialise"),
            )
        })
        .collect()
}

/// The fields of every `code` finding `validate` gives `element`, each an error naming the
/// element.
#[track_caller]
fn findings(element: Value, code: &str) -> Vec<Value> {
    validated(element)
        .into_iter()
        .filter(|(found, _, _)| found == code)
        .map(|(_, class, fields)| {
            assert_eq!(class, Class::Error, "{code} is an error");
            fields
        })
        .collect()
}

#[test]
fn a_valid_path_mask_validates_clean() {
    let errors: Vec<_> = validated(plate(json!([path_mask(triangle(), json!({}))])))
        .into_iter()
        .filter(|(_, class, _)| *class == Class::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn too_few_points_fires_at_two_vertices_and_is_silent_at_three() {
    let two = findings(
        plate(json!([
            {"name": "blur", "radius": 2},
            path_mask(json!([{"at": [10, 10]}, {"at": [90, 90]}]), json!({}))
        ])),
        "E-PATH-TOO-FEW-POINTS",
    );
    assert_eq!(two.len(), 1, "{two:?}");
    assert_eq!(two[0]["least"], json!(3));
    assert_eq!(two[0]["count"], json!(2));
    assert_eq!(two[0]["index"], json!(1), "the mask's index in `effects`");
    assert_eq!(two[0]["property"], json!("effects[1].points"));
    assert_eq!(
        findings(
            plate(json!([path_mask(triangle(), json!({}))])),
            "E-PATH-TOO-FEW-POINTS"
        ),
        Vec::<Value>::new()
    );
}

#[test]
fn a_keyed_points_whose_counts_differ_is_a_keyframe_shape_error_naming_its_record() {
    let keyed = json!([
        {"t": 0, "v": triangle()},
        {"t": 500, "v": [{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}, {"at": [10, 50]}],
         "ease": "linear"}
    ]);
    let fired = findings(
        plate(json!([path_mask(keyed, json!({}))])),
        "E-PATH-KEYFRAME-SHAPE",
    );
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["record"], json!(2));
    assert_eq!(fired[0]["t"], json!(500));
    assert_eq!(fired[0]["vertex"], json!(3));
    assert_eq!(fired[0]["index"], json!(0));
}

#[test]
fn outside_the_box_fires_against_a_written_rect_with_an_inset_of_zero() {
    // The rect is 60×40 at (20, 30): the box is [0, 60] × [0, 40] in the rect's own pixels,
    // so a vertex at x 60 is on its edge and one at 61 is outside.
    let at = |x: i64| {
        plate(json!([path_mask(
            json!([{"at": [0, 0]}, {"at": [x, 20]}, {"at": [0, 40]}]),
            json!({"x": 20, "y": 30, "width": 60, "height": 40})
        )]))
    };
    let fired = findings(at(61), "E-PATH-OUTSIDE-BOX");
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["position"], json!([61, 20]));
    assert_eq!(fired[0]["inset"], json!(0));
    assert_eq!(
        (fired[0]["right"].clone(), fired[0]["bottom"].clone()),
        (json!(60), json!(40))
    );
    assert_eq!(fired[0]["index"], json!(0));
    assert_eq!(findings(at(60), "E-PATH-OUTSIDE-BOX"), Vec::<Value>::new());
}

#[test]
fn outside_the_box_fires_against_the_elements_own_rect_when_the_rect_is_omitted() {
    // The plate is 100×100; a handle reaching to 110 is outside, though its vertex is in.
    let fired = findings(
        plate(json!([path_mask(
            json!([{"at": [10, 90]}, {"at": [50, 10], "out": [60, 0]}, {"at": [90, 90]}]),
            json!({})
        )])),
        "E-PATH-OUTSIDE-BOX",
    );
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["handle"], json!("out"));
    assert_eq!(fired[0]["position"], json!([110, 10]));
    assert_eq!(
        (fired[0]["right"].clone(), fired[0]["bottom"].clone()),
        (json!(100), json!(100))
    );
}

#[test]
fn outside_the_box_uses_the_smallest_keyed_size_of_a_rect_element() {
    // The plate grows from 80×100 to 120×70: the box is its smallest of each, 80×70.
    let mut element = plate(json!([path_mask(
        json!([{"at": [10, 60]}, {"at": [75, 10]}, {"at": [80, 70]}]),
        json!({})
    )]));
    element["width"] = json!([{"t": 0, "v": 80}, {"t": 1000, "v": 120, "ease": "linear"}]);
    element["height"] = json!([{"t": 0, "v": 100}, {"t": 1000, "v": 70, "ease": "linear"}]);
    assert_eq!(
        findings(element.clone(), "E-PATH-OUTSIDE-BOX"),
        Vec::<Value>::new()
    );
    element["effects"][0]["points"][2]["at"] = json!([81, 71]);
    let fired = findings(element, "E-PATH-OUTSIDE-BOX");
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["position"], json!([81, 71]));
    assert_eq!(
        (fired[0]["right"].clone(), fired[0]["bottom"].clone()),
        (json!(80), json!(70))
    );
}

#[test]
fn outside_the_box_checks_every_keyframe_value_and_names_its_record() {
    let keyed = json!([
        {"t": 0, "v": triangle()},
        {"t": 400, "v": [{"at": [10, 90]}, {"at": [50, 101]}, {"at": [90, 90]}], "ease": "linear"}
    ]);
    let fired = findings(
        plate(json!([path_mask(keyed, json!({}))])),
        "E-PATH-OUTSIDE-BOX",
    );
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(
        (fired[0]["record"].clone(), fired[0]["t"].clone()),
        (json!(2), json!(400))
    );
}

#[test]
fn a_mask_path_never_dangles_a_handle() {
    // Both handles shape the closing segment of a closed path.
    let element = plate(json!([path_mask(
        json!([{"at": [10, 90], "in": [5, 0]}, {"at": [50, 10]}, {"at": [90, 90], "out": [-5, 0]}]),
        json!({})
    )]));
    assert_eq!(
        findings(element, "E-PATH-DANGLING-HANDLE"),
        Vec::<Value>::new()
    );
}

#[test]
fn the_rect_mask_reviews_stay_silent_on_a_path_mask() {
    // An inverted path mask tracing the whole plate keeps nothing, and it is deliberate.
    let covering = json!([{"at": [0, 0]}, {"at": [100, 0]}, {"at": [100, 100]}, {"at": [0, 100]}]);
    let mut element = plate(json!([path_mask(
        covering.clone(),
        json!({"invert": true})
    )]));
    element["width"] = json!(160);
    element["effects"][0]["points"] =
        json!([{"at": [0, 0]}, {"at": [160, 0]}, {"at": [160, 100]}, {"at": [0, 100]}]);
    let codes: Vec<String> = validated(element)
        .into_iter()
        .map(|(code, _, _)| code)
        .collect();
    assert!(
        !codes.iter().any(|code| code.starts_with("R-MASK-")),
        "{codes:?}"
    );
}

#[test]
fn a_mask_path_whose_ends_meet_has_no_seam_to_cap() {
    // `R-PATH-SEAM-CAP` is a stroked open `path` element's: a mask path always closes, and
    // its first and last `at` meeting is only a doubled vertex.
    let element = plate(json!([path_mask(
        json!([{"at": [10, 90]}, {"at": [50, 10]}, {"at": [90, 90]}, {"at": [10, 90]}]),
        json!({})
    )]));
    let codes: Vec<String> = validated(element)
        .into_iter()
        .map(|(code, _, _)| code)
        .collect();
    assert!(
        !codes.iter().any(|code| code == "R-PATH-SEAM-CAP"),
        "{codes:?}"
    );
}

#[test]
fn a_point_finding_reads_as_a_sentence_naming_the_mask() {
    let report = montagent_core::validate(&project(
        &scratch(),
        &[plate(json!([path_mask(
            json!([{"at": [10, 90]}, {"at": [50, 120]}, {"at": [90, 90]}]),
            json!({})
        )]))],
    ));
    let text =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::default())
            .expect("the report renders");
    assert!(
        text.contains("`plate`.effects[0].points: vertex 1's `at` sits at [50,120]"),
        "{text}"
    );
    assert!(
        text.contains("outside the mask's box [0, 100] × [0, 100], the element's own rect"),
        "{text}"
    );
    assert!(
        !text.contains("keeps the stroke"),
        "a mask has no stroke: {text}"
    );
}

// ---- the picture ---------------------------------------------------------------------

/// The exact pixels at `instant` of the 200×200 black frame holding `elements`, at true
/// scale and lossless, decoded by a decoder that is not the one that wrote them.
#[track_caller]
fn painted_at(elements: &[Value], instant: i64) -> image::RgbaImage {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::frame::{Ask, frame};
    let answer = frame(
        &project(&scratch(), elements),
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

#[track_caller]
fn painted(element: Value) -> image::RgbaImage {
    painted_at(&[element], 500)
}

/// White on black, so the red channel is the kept coverage.
fn red(picture: &image::RgbaImage, x: u32, y: u32) -> i32 {
    i32::from(picture.get_pixel(x, y).0[0])
}

/// The plate spans 50..150 on both axes of the frame; `(x, y)` in its own pixels.
fn kept(picture: &image::RgbaImage, x: u32, y: u32) -> i32 {
    red(picture, 50 + x, 50 + y)
}

#[test]
fn a_triangle_mask_keeps_exactly_its_interior() {
    let masked = painted(plate(json!([path_mask(triangle(), json!({}))])));
    // The triangle (10, 90), (50, 10), (90, 90) as three half-planes, each `a·x + b·y ≤ c`
    // inside, with `(a, b)` of unit length: the geometry, stated apart from any painter.
    let norm = |a: f64, b: f64, c: f64| {
        let length = a.hypot(b);
        (a / length, b / length, c / length)
    };
    let edges = [
        norm(-80.0, -40.0, -4400.0),
        norm(80.0, -40.0, 3600.0),
        (0.0, 1.0, 90.0),
    ];
    // Whether every corner of the pixel lies `margin` px inside (or, negated, outside) one
    // edge.
    let corners = |x: u32, y: u32| {
        [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
            .map(|(dx, dy)| (f64::from(x) + dx, f64::from(y) + dy))
    };
    let within = |x, y, margin: f64| {
        edges.iter().all(|&(a, b, c)| {
            corners(x, y)
                .iter()
                .all(|&(px, py)| a * px + b * py <= c - margin)
        })
    };
    let beyond = |x, y, margin: f64| {
        edges.iter().any(|&(a, b, c)| {
            corners(x, y)
                .iter()
                .all(|&(px, py)| a * px + b * py >= c + margin)
        })
    };
    let (mut rim, mut deep) = (0, 0);
    for y in 0..100u32 {
        for x in 0..100u32 {
            let value = kept(&masked, x, y);
            if within(x, y, 1.0) {
                assert_eq!(value, 255, "({x}, {y}) lies a pixel or more inside");
                deep += 1;
            } else if within(x, y, 0.0) {
                // The eraser's antialiasing is supersampled at 1/16 of a pixel, so a pixel
                // just inside the edge may lose up to that much, as on every mask shape.
                assert!(value >= 255 - 16, "({x}, {y}) lies wholly inside: {value}");
            } else if beyond(x, y, 1.0) {
                assert_eq!(value, 0, "({x}, {y}) lies a pixel or more outside");
            } else if beyond(x, y, 0.0) {
                assert!(value <= 16, "({x}, {y}) lies wholly outside: {value}");
            } else {
                rim += i32::from(value > 0 && value < 255);
            }
        }
    }
    assert!(deep > 2500, "{deep} interior pixels");
    assert!(rim > 50, "the edge is antialiased ({rim} partial pixels)");
    // Beyond the plate there is nothing to keep.
    assert_eq!(red(&masked, 20, 20), 0);
}

#[test]
fn a_self_crossing_outline_keeps_its_overlap_by_the_nonzero_rule() {
    // A five-pointed star drawn in one stroke: its centre pentagon is wound twice, which
    // the nonzero rule keeps and even-odd would erase.
    let star = json!([{"at": [50, 5]}, {"at": [78, 95]}, {"at": [5, 38]}, {"at": [95, 38]},
                      {"at": [22, 95]}]);
    let picture = painted(plate(json!([path_mask(star, json!({}))])));
    assert_eq!(kept(&picture, 50, 52), 255, "the overlap is kept");
    assert_eq!(kept(&picture, 50, 20), 255, "a point is kept");
    assert_eq!(kept(&picture, 50, 97), 0, "between two points is erased");
}

/// ADR-0165 §1, over every mask shape: a plain mask and its inverted twin are exact
/// complements wherever either keeps a pixel whole or erases it whole; on a hard edge's partly
/// covered pixels the shape and its inverse are antialiased apart, so the edge is only bounded
/// thin; and a feathered pair, one blurred coverage kept `DstIn` and `DstOut`, sums to the
/// plate within one level of 255 on every pixel.
#[test]
fn a_mask_and_its_inverted_twin_are_complements_off_the_edge_and_within_one_level_when_feathered() {
    // Each shape's rect is placed so its edge partly covers pixels: a curve for `circle`,
    // `ellipse` and `path`, rounded corners for `rect`. Mask rects are whole numbers, so an
    // axis-aligned `rect` with no `radius` lies on whole pixels and has no partly covered
    // pixel at all; it is kept as the fifth case, where the hard pair is exact everywhere.
    // The rim bound is about two pixels along each outline's length; the radius-less rect
    // has no partly covered pixel, so its bound is none and its hard pair is the complement
    // on every pixel.
    let shapes = [
        (
            "rect",
            json!({"shape": "rect", "x": 15, "y": 20, "width": 70, "height": 60, "radius": 18}),
            true,
            200,
        ),
        (
            "rect without radius",
            json!({"shape": "rect", "x": 15, "y": 20, "width": 70, "height": 60}),
            false,
            1,
        ),
        (
            "circle",
            json!({"shape": "circle", "x": 10, "y": 10, "width": 80, "height": 80}),
            true,
            500,
        ),
        (
            "ellipse",
            json!({"shape": "ellipse", "x": 5, "y": 20, "width": 90, "height": 60}),
            true,
            500,
        ),
        (
            "path",
            json!({"shape": "path", "points": [{"at": [10, 90]}, {"at": [50, 10], "out": [30, 0]},
                                                     {"at": [90, 90], "in": [0, -30]}]}),
            true,
            500,
        ),
    ];
    for (shape, geometry, has_edge, rim_bound) in shapes {
        for feather in [None, Some(12)] {
            let mask = |invert: bool| {
                let mut mask = json!({"name": "mask"});
                for (key, value) in geometry.as_object().expect("geometry is an object") {
                    mask[key] = value.clone();
                }
                mask["invert"] = json!(invert);
                if let Some(feather) = feather {
                    mask["feather"] = json!(feather);
                }
                mask
            };
            let plain = painted(plate(json!([mask(false)])));
            let inverted = painted(plate(json!([mask(true)])));
            let (mut partial, mut rim) = (0, 0);
            for y in 0..100 {
                for x in 0..100 {
                    let (p, i) = (kept(&plain, x, y), kept(&inverted, x, y));
                    let whole = |value: i32| value == 0 || value == 255;
                    if feather.is_some() {
                        assert!(
                            (p + i - 255).abs() <= 1,
                            "{shape}, feather {feather:?} at ({x}, {y}): plain {p}, inverted {i}"
                        );
                    } else if whole(p) && whole(i) {
                        assert_eq!(p + i, 255, "{shape} at ({x}, {y}), off the edge");
                    } else {
                        rim += 1;
                    }
                    partial += i32::from(!whole(p));
                }
            }
            if has_edge || feather.is_some() {
                assert!(
                    partial > 0,
                    "{shape}, feather {feather:?}: an edge was drawn"
                );
            }
            assert!(rim < rim_bound, "{shape}: {rim} edge pixels");
            // Outside the element there is nothing to keep either way.
            assert_eq!(
                (red(&plain, 20, 20), red(&inverted, 20, 20)),
                (0, 0),
                "{shape}"
            );
        }
    }
}

#[test]
fn a_written_feather_zero_paints_the_bytes_of_none() {
    let bare = painted(plate(json!([path_mask(triangle(), json!({}))])));
    let zero = painted(plate(json!([path_mask(triangle(), json!({"feather": 0}))])));
    assert!(bare == zero);
    let soft = painted(plate(json!([path_mask(triangle(), json!({"feather": 8}))])));
    assert!(bare != soft, "a feather softens the edge");
}

#[test]
fn a_path_tracing_the_rect_paints_the_bytes_of_a_rect_mask() {
    let rect = json!({"x": 20, "y": 30, "width": 60, "height": 40});
    let corners = json!([{"at": [0, 0]}, {"at": [60, 0]}, {"at": [60, 40]}, {"at": [0, 40]}]);
    let mut square = json!({"name": "mask", "shape": "rect"});
    for (key, value) in rect.as_object().unwrap() {
        square[key] = value.clone();
    }
    for invert in [false, true] {
        let mut fields = rect.clone();
        fields["invert"] = json!(invert);
        square["invert"] = json!(invert);
        let traced = painted(plate(json!([path_mask(corners.clone(), fields)])));
        let boxed = painted(plate(json!([square.clone()])));
        assert!(traced == boxed, "invert {invert}");
    }
    // The bare form: corners of the element's own rect, against a bare `rect` mask.
    let whole = json!([{"at": [0, 0]}, {"at": [100, 0]}, {"at": [100, 100]}, {"at": [0, 100]}]);
    let traced = painted(plate(json!([
        path_mask(whole, json!({})),
        {"name": "blur", "radius": 3}
    ])));
    let boxed = painted(plate(json!([
        {"name": "mask", "shape": "rect"},
        {"name": "blur", "radius": 3}
    ])));
    assert!(traced == boxed, "the bare form");
}

#[test]
fn a_keyed_x_translates_the_whole_outline() {
    // `x` from 0 to 20 over the element's life is 10 at its midpoint: the same picture as
    // the outline with every vertex 10 px to the right.
    let outline = |dx: i64| {
        json!([{"at": [10 + dx, 90]}, {"at": [40 + dx, 10], "out": [20, 0]},
               {"at": [70 + dx, 90], "in": [0, -20]}])
    };
    let keyed = painted(plate(json!([path_mask(
        outline(0),
        json!({"x": [{"t": 0, "v": 0}, {"t": 1000, "v": 20, "ease": "linear"}],
               "y": 0, "width": 80, "height": 100})
    )])));
    let shifted = painted(plate(json!([path_mask(
        outline(10),
        json!({"x": 0, "y": 0, "width": 100, "height": 100})
    )])));
    assert!(keyed == shifted);
    let still = painted(plate(json!([path_mask(
        outline(0),
        json!({"x": 0, "y": 0, "width": 80, "height": 100})
    )])));
    assert!(keyed != still, "the keyed `x` moved the mask");
}

#[test]
fn a_keyed_outline_paints_its_in_between_value_and_clamps_an_overshoot_into_the_box() {
    // The apex sinks from y 10 to y 50: at the midpoint it is the static outline at y 30.
    let at = |y: i64| json!([{"at": [10, 90]}, {"at": [50, y]}, {"at": [90, 90]}]);
    let keyed = |ease: Value| {
        path_mask(
            json!([{"t": 0, "v": at(10)}, {"t": 1000, "v": at(50), "ease": ease}]),
            json!({}),
        )
    };
    assert!(
        painted(plate(json!([keyed(json!("linear"))])))
            == painted(plate(json!([path_mask(at(30), json!({}))])))
    );
    // An ease that overshoots below 10 takes the apex above the plate's top: it is held on
    // the box's edge, y 0, rather than drawn outside it.
    let overshoot = painted_at(
        &[plate(json!([path_mask(
            json!([{"t": 0, "v": at(50)}, {"t": 1000, "v": at(10), "ease": [0.3, 2.5, 0.6, 1.0]}]),
            json!({})
        )]))],
        300,
    );
    assert!(overshoot == painted(plate(json!([path_mask(at(0), json!({}))]))));
}

// ---- `query --at` and `shift` --------------------------------------------------------

/// A plate under a blur and a path mask whose apex sinks from `[50, 10]` to `[50, 50]` over
/// the element's life, its handle growing from 10 to 30 px.
fn sinking() -> Value {
    let at = |y: i64, reach: i64| json!([{"at": [10, 90]}, {"at": [50, y], "out": [reach, 0]}, {"at": [90, 90]}]);
    plate(json!([
        {"name": "blur", "radius": 2},
        path_mask(
            json!([{"t": 0, "v": at(10, 10)}, {"t": 1000, "v": at(50, 30), "ease": "linear"}]),
            json!({})
        )
    ]))
}

#[test]
fn query_at_prints_a_path_masks_absolute_control_points_in_box_pixels() {
    use montagent_core::verbs::query::at;
    let path = project(&scratch(), &[sinking()]);
    let document = montagent_core::parse::read(&path).expect("it parses");
    let answer = serde_json::to_value(at::at(&document, 500, None)).expect("it serialises");
    let present = &answer["stack"][0];
    assert_eq!(
        present["masks"],
        json!([{"index": 1, "path": [
            {"at": [10.0, 90.0]},
            {"at": [50.0, 30.0], "out": [70.0, 30.0]},
            {"at": [90.0, 90.0]}
        ]}]),
        "{present}"
    );
    // The relative form is still the one resolved value the one list prints.
    let row = present["values"]
        .as_array()
        .expect("values")
        .iter()
        .find(|row| row["property"] == "effects[1].points (mask)")
        .unwrap_or_else(|| panic!("no points row: {present}"))
        .clone();
    assert_eq!(row["animated"], json!(true));
    assert_eq!(
        row["value"][1],
        json!({"at": [50.0, 30.0], "out": [20.0, 0.0]})
    );
}

fn shifted(path: &Path, at: i64) -> montagent_core::verbs::shift::Answer {
    montagent_core::verbs::shift::shift(
        path,
        &montagent_core::verbs::shift::Ask {
            at,
            delta: 100,
            scope: None,
            release: Vec::new(),
        },
    )
}

#[test]
fn shift_cuts_a_keyed_mask_outline_where_it_resolves_to_integers_and_refuses_elsewhere() {
    use montagent_core::report::ExitCode;
    // At 250 ms the apex is at box y 20 and its handle at 15: whole numbers.
    let path = project(&scratch(), &[sinking()]);
    let answer = shifted(&path, 250);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "{:?}",
        answer.report().findings
    );
    let written = common::document(&path);
    let apex: Vec<(i64, Value)> = written["tracks"][0]["elements"][0]["effects"][1]["points"]
        .as_array()
        .expect("still keyed")
        .iter()
        .map(|record| (record["t"].as_i64().unwrap(), record["v"][1].clone()))
        .collect();
    assert_eq!(
        apex,
        [
            (0, json!({"at": [50, 10], "out": [10, 0]})),
            (250, json!({"at": [50, 20], "out": [15, 0]})),
            (350, json!({"at": [50, 20], "out": [15, 0]})),
            (1100, json!({"at": [50, 50], "out": [30, 0]})),
        ]
    );

    // At 333 ms the apex is at box y 23.32: the cut is refused and nothing is written.
    let path = project(&scratch(), &[sinking()]);
    let before = std::fs::read_to_string(&path).unwrap();
    let answer = shifted(&path, 333);
    let refusals: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refusals[0].fields["property"], "effects[1].points (mask)");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing is written"
    );
}

#[test]
fn fmt_carries_a_path_mask_through_unchanged_and_canonical() {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::fmt::{Mode, fmt};
    // A member's own keys are in the order its schema declares them, which the model's
    // own writer follows (the first test above); `fmt` lays the element out around it.
    let element = sinking();
    let raw = json!({"frame": {"width": 200, "height": 200}, "fps": 10, "background": "#000000",
        "tracks": [{"name": "only", "layer": 0, "elements": [element]}]});
    let path = write_project(&scratch(), "p.json", &raw.to_string());
    let before = common::document(&path);
    assert_eq!(fmt(&path, Mode::Write).exit_code(), ExitCode::Ok);
    assert_eq!(common::document(&path), before, "the same document");
    assert!(
        fmt(&path, Mode::Check).findings.is_empty(),
        "canonical after one rewrite"
    );
    let effects = &common::document(&path)["tracks"][0]["elements"][0]["effects"][1];
    let keys: Vec<&String> = effects.as_object().expect("a member").keys().collect();
    assert_eq!(keys, ["name", "shape", "points"]);
}
