//! Animatable properties, slice 1 (#675, ADR-0146): a shape's size and paint and a text
//! element's paint take keyframe lists, resolve through one function, and are carried by
//! every tool that reads a keyframe list.

use std::path::{Path, PathBuf};

use montagent_core::animatable::{self, Resolved};
use montagent_core::finding::Class;
use montagent_core::model::Colour;
use montagent_core::report::Report;
use montagent_core::verbs::render::{Ask, Supplying, paint_span, render};
use serde_json::json;

mod common;
use common::{canonical, fixture_dir, tempdir, write_project};

/// The morph project the prototype was approved on (#668, `e66dc809`), copied into a
/// scratch directory so a render writes nowhere in the checkout.
fn morph(dir: &Path) -> PathBuf {
    let body = std::fs::read_to_string(
        fixture_dir()
            .parent()
            .unwrap()
            .join("animatable-shape/morph.montagent.json"),
    )
    .unwrap();
    write_project(dir, "morph.montagent.json", &canonical(&body))
}

fn colour_of(literal: &str) -> Colour {
    serde_json::from_value(json!(literal)).unwrap()
}

fn errors(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| finding.class == Class::Error)
        .map(|finding| format!("{}: {:?}", finding.code, finding.fields))
        .collect()
}

#[test]
fn the_morph_project_validates() {
    let dir = tempdir(line!());
    let report = montagent_core::validate(&morph(&dir));
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
}

// ---------------------------------------------------------------------------
// The one resolving function.
// ---------------------------------------------------------------------------

#[test]
fn a_fade_to_transparent_black_keeps_its_hue_at_the_midpoint() {
    let element = json!({"type": "rect", "fill": [
        {"t": 0, "v": "#FF0000"},
        {"t": 1000, "v": "#00000000", "ease": "linear"}
    ]});
    let colour = |t| animatable::at(&element, "fill", t).unwrap().unwrap();
    // Premultiplied: half the red at half the alpha is full red at half alpha. Straight
    // interpolation would answer `#80000080`, a dark red going grey.
    assert_eq!(colour(500), Resolved::Colour(colour_of("#FF000080")));
    assert_eq!(colour(0), Resolved::Colour(colour_of("#FF0000")));
    assert_eq!(colour(1000), Resolved::Colour(colour_of("#00000000")));
}

#[test]
fn a_colour_overshoot_clamps_each_component_to_its_range() {
    // `y` far past 1: the blend runs past white and is clamped back to it.
    let element = json!({"type": "rect", "fill": [
        {"t": 0, "v": "#000000"},
        {"t": 1000, "v": "#FFFFFF", "ease": [0.5, 3.0, 0.5, 3.0]}
    ]});
    assert_eq!(
        animatable::at(&element, "fill", 500).unwrap().unwrap(),
        Resolved::Colour(colour_of("#FFFFFF"))
    );
}

#[test]
fn an_integer_property_resolves_to_a_continuous_value() {
    let element = json!({"type": "rect", "width": [
        {"t": 0, "v": 0},
        {"t": 1000, "v": 1, "ease": "linear"}
    ], "height": 10});
    assert_eq!(
        animatable::at(&element, "width", 500).unwrap().unwrap(),
        Resolved::Number(0.5)
    );
}

#[test]
fn a_resolved_radius_clamps_to_half_the_shorter_side() {
    let element = json!({"type": "rect", "width": 100, "height": [
        {"t": 0, "v": 40},
        {"t": 1000, "v": 20, "ease": "linear"}
    ], "radius": 60});
    let radius = |t| animatable::at(&element, "radius", t).unwrap().unwrap();
    assert_eq!(radius(0), Resolved::Number(20.0));
    assert_eq!(radius(1000), Resolved::Number(10.0));
}

#[test]
fn a_resolved_stroke_width_below_zero_clamps_to_zero() {
    // An overshoot below the first value, which is `0`.
    let element = json!({"type": "rect", "stroke": "#FFFFFF", "stroke_width": [
        {"t": 0, "v": 0},
        {"t": 1000, "v": 10, "ease": [0.5, -2.0, 0.5, 1.0]}
    ]});
    assert_eq!(
        animatable::at(&element, "stroke_width", 200)
            .unwrap()
            .unwrap(),
        Resolved::Number(0.0)
    );
}

#[test]
fn a_property_the_element_does_not_declare_resolves_to_nothing() {
    let element = json!({"type": "rect", "width": 10, "height": 10});
    assert!(animatable::at(&element, "fill", 0).is_none());
}

// ---------------------------------------------------------------------------
// The painter draws what the resolver says.
// ---------------------------------------------------------------------------

/// A 200×200 project at 10 fps over black, one track holding `element`.
fn one_element(dir: &Path, element: &str) -> PathBuf {
    write_project(
        dir,
        "p.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":200,"height":200}},"fps":10,"background":"#000000",
                "tracks":[{{"name":"only","layer":0,"elements":[{element}]}}]}}"##
        )),
    )
}

/// The frame painted at `t`, as RGB.
fn painted_at(path: &Path, t: i64) -> Vec<u8> {
    let rasters = paint_span(path, t, t + 100, Supplying::PerFrame).expect("the span paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    rasters.frames.into_iter().next().unwrap()
}

fn pixel(rgb: &[u8], x: usize, y: usize) -> [u8; 3] {
    let at = (y * 200 + x) * 3;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
}

#[test]
fn a_keyed_width_is_painted_at_its_resolved_size() {
    let dir = tempdir(line!());
    let path = one_element(
        &dir,
        r##"{"id":"bar","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "width":[{"t":0,"v":0},{"t":1000,"v":200,"ease":"linear"}],"height":200,
            "fill":"#FF0000"}"##,
    );
    // At 500 ms the bar is 100 wide, centred: x 50..150.
    let frame = painted_at(&path, 500);
    assert_eq!(pixel(&frame, 100, 100), [255, 0, 0]);
    assert_eq!(pixel(&frame, 60, 100), [255, 0, 0]);
    assert_eq!(pixel(&frame, 40, 100), [0, 0, 0]);
}

#[test]
fn a_keyed_fill_fades_to_transparent_black_without_going_grey() {
    let dir = tempdir(line!());
    let path = one_element(
        &dir,
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "width":200,"height":200,
            "fill":[{"t":0,"v":"#FF0000"},{"t":1000,"v":"#00000000","ease":"linear"}]}"##,
    );
    // Full red at half alpha over black is half red. Straight alpha would paint a quarter.
    let [r, g, b] = pixel(&painted_at(&path, 500), 100, 100);
    assert!((127..=129).contains(&r), "{r}");
    assert_eq!([g, b], [0, 0]);
}

#[test]
fn a_size_that_reaches_zero_at_some_frames_renders_and_draws_nothing_there() {
    let dir = tempdir(line!());
    let path = one_element(
        &dir,
        r##"{"id":"pop","type":"ellipse","start":0,"end":1000,"x":100,"y":100,
            "width":[{"t":0,"v":0},{"t":500,"v":200,"ease":"ease-out"}],
            "height":[{"t":0,"v":0},{"t":500,"v":200,"ease":"ease-out"}],
            "fill":"#FFFFFF"}"##,
    );
    assert!(errors(&montagent_core::validate(&path)).is_empty());
    assert_eq!(pixel(&painted_at(&path, 0), 100, 100), [0, 0, 0]);
    assert_eq!(pixel(&painted_at(&path, 600), 100, 100), [255, 255, 255]);
}

fn no_extent(findings: impl IntoIterator<Item = (String, String)>) -> Vec<(String, String)> {
    findings
        .into_iter()
        .filter(|(code, _)| code == "E-NOT-PAINTED-NO-EXTENT")
        .collect()
}

fn coded(report: &Report) -> Vec<(String, String)> {
    report
        .findings
        .iter()
        .map(|finding| {
            let detail = finding
                .fields
                .iter()
                .find(|(name, _)| *name == "detail")
                .map(|(_, value)| value.to_string())
                .unwrap_or_default();
            (finding.code.clone(), detail)
        })
        .collect()
}

#[test]
fn a_keyed_size_at_zero_on_every_frame_gets_one_finding_from_validate_and_render() {
    let dir = tempdir(line!());
    let path = one_element(
        &dir,
        r##"{"id":"flat","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "width":[{"t":0,"v":0},{"t":2000,"v":100,"ease":"step"}],"height":50,
            "fill":"#FF0000"}"##,
    );
    let validated = no_extent(coded(&montagent_core::validate(&path)));
    assert_eq!(validated.len(), 1, "{validated:?}");
    assert!(
        validated[0].1.contains("box"),
        "the finding names its cause"
    );

    // The painter, asked without `validate` in front of it, decides the same.
    let rasters = paint_span(&path, 0, 1000, Supplying::PerFrame).unwrap();
    let painted: Vec<(String, String)> = rasters
        .declined
        .iter()
        .map(|line| {
            let (code, detail) = line.split_once(": ").unwrap();
            (code.to_string(), detail.to_string())
        })
        .collect();
    assert_eq!(no_extent(painted), validated);

    if common::has_ffprobe() {
        let ask = Ask {
            output: Some(dir.join("out.mp4")),
            ..Ask::default()
        };
        let answer = render(&path, &ask, &mut |_| {});
        assert_eq!(no_extent(coded(answer.report())), validated);
        assert!(answer.video().is_none());
    }
}

#[test]
fn the_morph_project_renders() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = morph(&dir);
    let answer = render(&path, &Ask::default(), &mut |_| {});
    assert!(
        errors(answer.report()).is_empty(),
        "{:#?}",
        errors(answer.report())
    );
    assert!(answer.video().is_some());
}

/// A paint chunk starts by painting what `--from` would, so every frame of a keyed shape
/// must be a function of its instant alone: painted inside a long span or on its own, it
/// is the same bytes (ADR-0144).
#[test]
fn a_keyed_frame_is_the_same_bytes_painted_alone_or_in_a_span() {
    let dir = tempdir(line!());
    let path = morph(&dir);
    let span = paint_span(&path, 900, 2000, Supplying::PerFrame).unwrap();
    for (index, t) in [(0usize, 900i64), (7, 1133), (20, 1566), (32, 1966)] {
        let alone = paint_span(&path, t, t + 1, Supplying::PerFrame).unwrap();
        assert_eq!(alone.frames.len(), 1);
        assert!(
            alone.frames[0] == span.frames[index],
            "frame {index} at {t} ms differs"
        );
    }
}

/// Every frame of a keyed shape, with an overshoot and a collapse to nothing, is the same
/// bytes at the encoder's input on three painters over two-frame chunks as on one (ADR-0144).
#[test]
fn a_keyed_shape_renders_byte_identically_on_parallel_painters() {
    use montagent_core::verbs::render::{Forced, force_painting, tap_frames};

    if !common::has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = write_project(
        &dir,
        "p.json",
        &canonical(
            r##"{"frame":{"width":200,"height":200},"fps":10,"background":"#000000","output":"out.mp4",
                "tracks":[{"name":"only","layer":0,"elements":[
                {"id":"morph","type":"rect","start":0,"end":2000,"x":100,"y":100,
                 "width":[{"t":0,"v":0},{"t":800,"v":160,"ease":[0.34,1.56,0.64,1]},
                          {"t":2000,"v":40,"ease":"ease-in-out"}],
                 "height":[{"t":0,"v":0},{"t":800,"v":120,"ease":[0.34,1.56,0.64,1]}],
                 "radius":[{"t":0,"v":0},{"t":1500,"v":60,"ease":"linear"}],
                 "fill":[{"t":0,"v":"#3B82F6"},{"t":2000,"v":"#22C55E00","ease":"linear"}],
                 "stroke":"#FFFFFF",
                 "stroke_width":[{"t":0,"v":0},{"t":1000,"v":6,"ease":"ease-out"}]}]}]}"##,
        ),
    );
    let hashes = |forced: Forced| {
        let _forced = force_painting(forced);
        let tap = tap_frames();
        let answer = render(&path, &Ask::default(), &mut |_| {});
        assert!(
            errors(answer.report()).is_empty(),
            "{:?}",
            errors(answer.report())
        );
        tap.hashes()
    };
    let one = hashes(Forced::Chunks {
        painters: 1,
        chunk: 2,
        window_bytes: None,
    });
    let three = hashes(Forced::Chunks {
        painters: 3,
        chunk: 2,
        window_bytes: None,
    });
    assert_eq!(one.len(), 20);
    assert_eq!(one, three);
}

// ---------------------------------------------------------------------------
// The one list, derived from the schema.
// ---------------------------------------------------------------------------

fn names_of(element_type: &str) -> Vec<&'static str> {
    animatable::of(element_type)
        .iter()
        .map(|property| property.name.as_str())
        .collect()
}

#[test]
fn the_list_is_what_the_schema_types_as_animatable() {
    assert_eq!(
        names_of("rect"),
        [
            "x",
            "y",
            "width",
            "height",
            "fill",
            "fill.angle",
            "fill.stops",
            "fill.center",
            "fill.radius",
            "stroke",
            "stroke.angle",
            "stroke.stops",
            "stroke.center",
            "stroke.radius",
            "stroke_width",
            "radius",
            "scale",
            "rotation",
            "opacity"
        ]
    );
    assert_eq!(
        names_of("ellipse"),
        [
            "x",
            "y",
            "width",
            "height",
            "fill",
            "fill.angle",
            "fill.stops",
            "fill.center",
            "fill.radius",
            "stroke",
            "stroke.angle",
            "stroke.stops",
            "stroke.center",
            "stroke.radius",
            "stroke_width",
            "scale",
            "rotation",
            "opacity"
        ]
    );
    assert_eq!(
        names_of("text"),
        [
            "x",
            "y",
            "color",
            "color.angle",
            "color.stops",
            "color.center",
            "color.radius",
            "stroke",
            "stroke.angle",
            "stroke.stops",
            "stroke.center",
            "stroke.radius",
            "stroke_width",
            "letter_spacing",
            "scale",
            "rotation",
            "opacity"
        ]
    );
    // `letter_spacing` (ADR-0151) is integer-typed, so a `shift` split rounds it as it rounds
    // `x`, and unbounded: a negative value tightens.
    let spacing = animatable::property("letter_spacing").expect("letter_spacing is animatable");
    assert_eq!(spacing.kind, animatable::Kind::Integer);
    assert_eq!(spacing.minimum, None);
    assert_eq!(
        names_of("image"),
        ["x", "y", "scale", "rotation", "opacity"]
    );
    assert_eq!(names_of("audio"), ["volume"]);
    assert!(names_of("transition").is_empty());
}

#[test]
fn every_numeric_and_colour_effect_parameter_is_in_the_list() {
    // ADR-0146 §3, slice 2: every member's numeric and colour parameters; `mask.shape` and
    // `invert` stay static.
    let members: Vec<(&str, Vec<&str>)> = animatable::effect_members()
        .map(|(member, parameters)| (member, parameters.iter().map(|p| p.name.as_str()).collect()))
        .collect();
    assert_eq!(
        members,
        [
            ("blur", vec!["radius"]),
            ("shadow", vec!["dx", "dy", "radius", "color", "opacity"]),
            (
                "mask",
                vec!["x", "y", "width", "height", "radius", "feather"]
            ),
            ("tint", vec!["color", "amount"]),
            ("saturation", vec!["amount"]),
            ("brightness", vec!["amount"]),
            ("contrast", vec!["amount"]),
            ("chroma", vec!["color", "tolerance", "softness", "spill"]),
        ]
    );
    let (_, chroma) = animatable::effect_members()
        .find(|(member, _)| *member == "chroma")
        .unwrap();
    let tolerance = chroma.iter().find(|p| p.name == "tolerance").unwrap();
    assert_eq!(
        (tolerance.kind, tolerance.minimum, tolerance.maximum),
        (animatable::Kind::Number, Some(0.0), Some(1.0))
    );
    assert_eq!(chroma[0].kind, animatable::Kind::Colour);
}

#[test]
fn the_box_moving_properties_are_animatable_ones() {
    for property in montagent_core::checks::MOVES_THE_BOX {
        assert!(
            animatable::names().iter().any(|name| name == property),
            "`{property}`"
        );
    }
}

// ---------------------------------------------------------------------------
// Schema: a negative length is an error, `0` is legal.
// ---------------------------------------------------------------------------

fn schema_reasons(path: &Path) -> Vec<String> {
    montagent_core::validate(path)
        .findings
        .iter()
        .filter(|finding| finding.code.starts_with("E-SCHEMA"))
        .map(|finding| finding.fields["reason"].to_string())
        .collect()
}

#[test]
fn a_negative_keyframe_value_for_a_length_is_a_schema_error() {
    for (property, records) in [
        (
            "width",
            r#"[{"t":0,"v":100},{"t":500,"v":-5,"ease":"linear"}]"#,
        ),
        ("height", r#"[{"t":0,"v":-1}]"#),
        (
            "radius",
            r#"[{"t":0,"v":0},{"t":500,"v":-2,"ease":"linear"}]"#,
        ),
        (
            "stroke_width",
            r#"[{"t":0,"v":4},{"t":500,"v":-4,"ease":"linear"}]"#,
        ),
    ] {
        let dir = tempdir(line!());
        let mut element: serde_json::Value = serde_json::from_str(
            r##"{"id":"card","type":"rect","start":0,"end":1000,"x":100,"y":100,
                "width":100,"height":100,"fill":"#FF0000","stroke":"#FFFFFF","stroke_width":2}"##,
        )
        .unwrap();
        element[property] = serde_json::from_str(records).unwrap();
        let path = one_element(&dir, &element.to_string());
        let reasons = schema_reasons(&path);
        assert!(
            reasons.iter().any(|reason| reason.contains("non-negative")),
            "`{property}`: {reasons:?}"
        );
    }
}

#[test]
fn a_zero_keyframe_value_for_a_length_is_legal() {
    let dir = tempdir(line!());
    let path = one_element(
        &dir,
        r##"{"id":"card","type":"rect","start":0,"end":1000,"x":100,"y":100,
            "width":[{"t":0,"v":0},{"t":500,"v":100,"ease":"linear"}],"height":100,
            "radius":[{"t":0,"v":0},{"t":500,"v":10,"ease":"linear"}],
            "fill":"#FF0000","stroke":"#FFFFFF",
            "stroke_width":[{"t":0,"v":0},{"t":500,"v":4,"ease":"linear"}]}"##,
    );
    assert!(schema_reasons(&path).is_empty());
}

// ---------------------------------------------------------------------------
// The off-canvas check resolves a keyed box over the whole range.
// ---------------------------------------------------------------------------

fn off_canvas(path: &Path) -> usize {
    montagent_core::validate(path)
        .findings
        .iter()
        .filter(|finding| finding.code == "R-OFF-CANVAS")
        .count()
}

#[test]
fn a_keyed_box_that_never_meets_the_frame_is_off_canvas() {
    let dir = tempdir(line!());
    // Left of the frame for its whole life, collapsing from nothing and growing to 300.
    let path = one_element(
        &dir,
        r##"{"id":"far","type":"rect","start":0,"end":1000,"x":-400,"y":100,
            "width":[{"t":0,"v":0},{"t":1000,"v":300,"ease":"linear"}],"height":50,
            "fill":"#FF0000"}"##,
    );
    assert_eq!(off_canvas(&path), 1);
}

#[test]
fn a_keyed_box_that_grows_onto_the_frame_is_not_off_canvas() {
    let dir = tempdir(line!());
    // Centred at -400: on screen only once it is wider than 800, which it is from 500 ms.
    let path = one_element(
        &dir,
        r##"{"id":"reach","type":"rect","start":0,"end":1000,"x":-400,"y":100,
            "width":[{"t":0,"v":10},{"t":500,"v":1000,"ease":"linear"},
                     {"t":1000,"v":10,"ease":"linear"}],"height":50,
            "fill":"#FF0000"}"##,
    );
    assert_eq!(off_canvas(&path), 0);
}

// ---------------------------------------------------------------------------
// A keyed text paint every run overrides changes nothing.
// ---------------------------------------------------------------------------

#[track_caller]
fn overridden(runs: &str) -> Vec<(String, String)> {
    let dir = tempdir(std::panic::Location::caller().line());
    let path = one_element(
        &dir,
        &format!(
            r##"{{"id":"title","type":"text","start":0,"end":1000,"x":100,"y":100,
                "width":180,"height":60,"font":"brand","size":20,
                "color":[{{"t":0,"v":"#FFFFFF"}},{{"t":500,"v":"#FF0000","ease":"linear"}}],
                "stroke":"#000000",
                "stroke_width":[{{"t":0,"v":0}},{{"t":500,"v":2,"ease":"linear"}}],
                "runs":[{runs}]}}"##
        ),
    );
    let document = montagent_core::parse::read(&path).unwrap();
    let mut report = Report::new("validate", Some(document.path().to_string()));
    montagent_core::checks::overridden::check(&document, &mut report);
    report
        .findings
        .iter()
        .map(|finding| {
            assert_eq!(finding.class, Class::Review);
            assert_eq!(finding.code, "R-TEXT-PAINT-OVERRIDDEN");
            (
                finding.location.element.clone().unwrap(),
                finding.fields["property"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn a_keyed_text_paint_that_every_run_overrides_is_reported() {
    assert_eq!(
        overridden(
            r##"{"text":"Hello ","color":"#00FF00"},{"text":"world","color":"#0000FF","stroke_width":1}"##
        ),
        [("title".to_string(), "color".to_string())]
    );
}

#[test]
fn a_keyed_text_paint_one_run_leaves_alone_is_not_reported() {
    assert!(overridden(r##"{"text":"Hello ","color":"#00FF00"},{"text":"world"}"##).is_empty());
}

// ---------------------------------------------------------------------------
// A text element's keyed paint.
// ---------------------------------------------------------------------------

#[test]
fn a_text_elements_keyed_stroke_is_painted_at_its_resolved_width_and_colour() {
    let fixture = common::document(&common::fixture_project());
    let project = canonical(
        &json!({
            "frame": {"width": 200, "height": 200}, "fps": 10, "background": "#000000",
            "fonts": fixture["fonts"], "fontVendor": fixture["fontVendor"],
            "tracks": [{"name": "only", "layer": 0, "elements": [
                {"id": "word", "type": "text", "start": 0, "end": 1000, "x": 100, "y": 100,
                 "width": 180, "height": 120, "font": "brand", "size": 80,
                 "color": [{"t": 0, "v": "#FFFFFF"}, {"t": 1000, "v": "#0000FF", "ease": "linear"}],
                 "stroke": "#FF0000",
                 "stroke_width": [{"t": 0, "v": 0}, {"t": 1000, "v": 8, "ease": "linear"}],
                 "runs": [{"text": "Hi"}], "caption": false}
            ]}]
        })
        .to_string(),
    );
    let scratch = common::Scratch::beside_the_fixture("animatable-text-paint", &project);
    let reddish = |t: i64| {
        let rasters = paint_span(scratch.path(), t, t + 100, Supplying::PerFrame).unwrap();
        assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
        rasters.frames[0]
            .chunks(3)
            .filter(|rgb| rgb[0] > 200 && rgb[1] < 60 && rgb[2] < 60)
            .count()
    };
    assert_eq!(reddish(0), 0, "no stroke at width 0");
    assert!(reddish(900) > 0, "a red stroke once it has width");
}

/// The stroke never moves a glyph, so a keyed `stroke_width` lays text out once: measured
/// with the widest stroke it ever reaches, which is the extent a declared box is checked
/// against.
#[test]
fn a_keyed_text_stroke_width_is_measured_at_its_widest() {
    let fixture = common::document(&common::fixture_project());
    let ink = |stroke_width: serde_json::Value, name: &str| {
        let project = canonical(
            &json!({
                "frame": {"width": 400, "height": 400}, "fps": 10,
                "fonts": fixture["fonts"], "fontVendor": fixture["fontVendor"],
                "tracks": [{"name": "only", "layer": 0, "elements": [
                    {"id": "word", "type": "text", "start": 0, "end": 1000, "x": 200,
                     "y": 200, "width": 380, "height": 200, "font": "brand", "size": 80,
                     "stroke": "#FF0000", "stroke_width": stroke_width,
                     "runs": [{"text": "Hi"}], "caption": false}
                ]}]
            })
            .to_string(),
        );
        let scratch = common::Scratch::beside_the_fixture(name, &project);
        let document = montagent_core::parse::read(scratch.path()).unwrap();
        let stack = montagent_core::verbs::query::at::at(&document, 0, None);
        serde_json::to_value(stack.stack[0].ink_box).unwrap()
    };
    let keyed = ink(
        json!([{"t": 0, "v": 0}, {"t": 500, "v": 8, "ease": "linear"}, {"t": 1000, "v": 2, "ease": "linear"}]),
        "animatable-text-measure-keyed",
    );
    assert!(!keyed.is_null());
    assert_eq!(keyed, ink(json!(8), "animatable-text-measure-static"));
}

// ---------------------------------------------------------------------------
// `shift` carries and splits every list.
// ---------------------------------------------------------------------------

fn shifted(path: &Path, at: i64, delta: i64) -> montagent_core::verbs::shift::Answer {
    montagent_core::verbs::shift::shift(
        path,
        &montagent_core::verbs::shift::Ask {
            at,
            delta,
            scope: None,
            release: Vec::new(),
        },
    )
}

#[test]
fn shift_splits_a_keyed_width_and_fill_with_the_elements_x() {
    let dir = tempdir(line!());
    let path = one_element(
        &dir,
        r##"{"id":"card","type":"rect","start":0,"end":2000,
            "x":[{"t":0,"v":50},{"t":1000,"v":150,"ease":"linear"}],"y":100,
            "width":[{"t":0,"v":10},{"t":1000,"v":30,"ease":"linear"}],"height":20,
            "fill":[{"t":0,"v":"#FF0000"},{"t":1000,"v":"#0000FF","ease":"linear"}]}"##,
    );
    let answer = shifted(&path, 500, 200);
    assert!(
        errors(answer.report()).is_empty(),
        "{:?}",
        errors(answer.report())
    );

    let element = &common::document(&path)["tracks"][0]["elements"][0];
    let list = |property: &str| -> Vec<(i64, serde_json::Value)> {
        element[property]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| (record["t"].as_i64().unwrap(), record["v"].clone()))
            .collect()
    };
    assert_eq!(
        list("x"),
        [
            (0, json!(50)),
            (500, json!(100)),
            (700, json!(100)),
            (1200, json!(150))
        ]
    );
    assert_eq!(
        list("width"),
        [
            (0, json!(10)),
            (500, json!(20)),
            (700, json!(20)),
            (1200, json!(30))
        ]
    );
    // Half red, half blue, premultiplied and opaque: each channel 127.5, rounded to 128.
    assert_eq!(
        list("fill"),
        [
            (0, json!("#FF0000")),
            (500, json!("#800080")),
            (700, json!("#800080")),
            (1200, json!("#0000FF"))
        ]
    );
}

#[test]
fn shift_refuses_a_split_inside_an_overshoot_that_dips_below_zero() {
    let dir = tempdir(line!());
    let body = r##"{"id":"pop","type":"rect","start":0,"end":2000,"x":100,"y":100,
            "width":[{"t":0,"v":100},{"t":1000,"v":0,"ease":[0.34,1.56,0.64,1]}],"height":50,
            "fill":"#FF0000"}"##;
    let path = one_element(&dir, body);
    let element: serde_json::Value = serde_json::from_str(body).unwrap();
    let at = 600;
    let raw = animatable::at(&element, "width", at).unwrap().unwrap();
    assert!(
        raw.number().unwrap() < 0.0,
        "the curve dips below zero at {at}: {raw:?}"
    );

    let before = std::fs::read_to_string(&path).unwrap();
    let answer = shifted(&path, at, 100);
    let refusals: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refusals[0].fields["property"], "width");
    assert_eq!(refusals[0].fields["at"], at);
    assert_eq!(refusals[0].location.element.as_deref(), Some("pop"));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing is written"
    );
}

// ---------------------------------------------------------------------------
// `query --at` prints every animated property's resolved value.
// ---------------------------------------------------------------------------

#[test]
fn query_at_prints_a_resolved_colour_as_a_literal_that_pastes_back() {
    let dir = tempdir(line!());
    let path = one_element(
        &dir,
        r##"{"id":"card","type":"rect","start":0,"end":2000,"x":100,"y":100,
            "width":[{"t":0,"v":100},{"t":1000,"v":101,"ease":"linear"}],"height":100,
            "fill":[{"t":0,"v":"#FF0000"},{"t":1000,"v":"#00000000","ease":"linear"}]}"##,
    );
    let document = montagent_core::parse::read(&path).unwrap();
    let value = |t: i64, property: &str| {
        let stack = montagent_core::verbs::query::at::at(&document, t, None);
        stack.stack[0]
            .values
            .iter()
            .find(|value| value.property == property)
            .and_then(|value| value.value.clone())
            .unwrap()
    };
    assert_eq!(value(0, "fill"), json!("#FF0000"));
    assert_eq!(value(500, "fill"), json!("#FF000080"));
    assert_eq!(value(1500, "fill"), json!("#00000000"));
    assert_eq!(value(500, "width"), json!(100.5));
}
