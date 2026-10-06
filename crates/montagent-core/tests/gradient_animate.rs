//! Gradients, slice 2 (#687, ADR-0149 §3-§6): `angle`, `center`, `radius` and `stops` animate
//! in place, resolved through the one function every tool reads.

use std::path::{Path, PathBuf};

use montagent_core::finding::Class;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

fn two(from: &str, to: &str) -> Value {
    json!([{"offset": 0, "color": from}, {"offset": 1, "color": to}])
}

fn linear(angle: Value, stops: Value) -> Value {
    json!({"gradient": "linear", "angle": angle, "stops": stops})
}

fn radial(center: Value, radius: Value, stops: Value) -> Value {
    json!({"gradient": "radial", "center": center, "radius": radius, "stops": stops})
}

/// Two records, `v0` at `t` 0 and `v1` at `t` 1000, the second eased `ease`.
fn keyed(v0: Value, v1: Value, ease: &str) -> Value {
    keyed_eased(v0, v1, json!(ease))
}

fn keyed_eased(v0: Value, v1: Value, ease: Value) -> Value {
    json!([{"t": 0, "v": v0}, {"t": 1000, "v": v1, "ease": ease}])
}

fn write(dir: &Path, elements: Value) -> PathBuf {
    let tracks: Vec<Value> = elements
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, e)| json!({"name": format!("t{i}"), "layer": i, "elements": [e]}))
        .collect();
    let body = json!({"frame": {"width": 200, "height": 200}, "fps": 10, "background": "#000000",
                      "tracks": tracks});
    write_project(dir, "p.json", &canonical(&body.to_string()))
}

fn card(fill: Value) -> Value {
    json!({"id": "box", "type": "rect", "start": 0, "end": 1000, "x": 100, "y": 100,
           "width": 100, "height": 50, "fill": fill})
}

#[track_caller]
fn project_with(fill: Value) -> PathBuf {
    write(
        &tempdir(std::panic::Location::caller().line()),
        json!([card(fill)]),
    )
}

fn errors_of(path: &Path) -> Vec<String> {
    montagent_core::validate(path)
        .findings
        .iter()
        .filter(|finding| finding.class == Class::Error)
        .map(|finding| format!("{}: {:?}", finding.code, finding.fields))
        .collect()
}

/// What `query --at` prints for `property` on `id`.
fn printed(path: &Path, id: &str, property: &str, at: i64) -> Value {
    use montagent_core::verbs::query::{Ask, query};
    let answer = query(
        path,
        &Ask {
            at: Some(at),
            ..Ask::default()
        },
    )
    .to_json();
    answer["query"]["stack"]
        .as_array()
        .unwrap()
        .iter()
        .find(|element| element["id"] == id)
        .unwrap()["values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["property"] == property)
        .unwrap_or_else(|| panic!("no `{property}`: {answer}"))
        .clone()
}

#[test]
fn a_keyed_angle_validates_and_resolves_at_an_instant() {
    let path = project_with(linear(
        keyed(json!(0), json!(90), "linear"),
        two("#FFFFFF", "#000000"),
    ));
    assert!(errors_of(&path).is_empty(), "{:?}", errors_of(&path));
    let fill = printed(&path, "box", "fill", 500);
    assert_eq!(fill["animated"], json!(true));
    assert_eq!(fill["value"]["angle"], json!(45));
}

fn findings(path: &Path, code: &str) -> Vec<(Class, Value)> {
    montagent_core::validate(path)
        .findings
        .iter()
        .filter(|finding| finding.code == code)
        .map(|finding| {
            let fields: serde_json::Map<String, Value> = finding
                .fields
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect();
            (finding.class, Value::Object(fields))
        })
        .collect()
}

#[test]
fn a_keyed_center_radius_and_stops_resolve_each_on_its_own_curve() {
    let fill = json!({"gradient": "radial",
        "center": keyed(json!([0.0, 0.0]), json!([1.0, 0.5]), "linear"),
        "radius": keyed(json!(0.5), json!(1.5), "linear"),
        "stops": keyed(two("#FF0000", "#FF0000"), two("#0000FF", "#0000FF"), "linear")});
    let path = project_with(fill);
    assert!(errors_of(&path).is_empty(), "{:?}", errors_of(&path));
    let value = printed(&path, "box", "fill", 500)["value"].clone();
    assert_eq!(value["center"], json!([0.5, 0.25]));
    assert_eq!(value["radius"], json!(1));
    // Half red, half blue, premultiplied and opaque: each channel 127.5, rounded to 128.
    assert_eq!(value["stops"][0]["color"], json!("#800080"));
    assert_eq!(value["stops"][1]["color"], json!("#800080"));
    // Past either end the value holds.
    assert_eq!(
        printed(&path, "box", "fill", 0)["value"]["radius"],
        json!(0.5)
    );
}

#[test]
fn stop_i_blends_with_stop_i_on_offset_and_colour() {
    let from = json!([{"offset": 0, "color": "#FF0000"}, {"offset": 0.2, "color": "#FF0000"},
                      {"offset": 1, "color": "#FF0000"}]);
    let to = json!([{"offset": 0, "color": "#0000FF"}, {"offset": 0.6, "color": "#0000FF"},
                    {"offset": 1, "color": "#0000FF"}]);
    let path = project_with(linear(json!(90), keyed(from, to, "linear")));
    assert!(errors_of(&path).is_empty(), "{:?}", errors_of(&path));
    let stops = printed(&path, "box", "fill", 500)["value"]["stops"].clone();
    assert_eq!(stops[1]["offset"], json!(0.4));
}

/// A stop list whose second stop overshoots past the third at some instants of the bezier.
fn overshooting() -> Value {
    let from = json!([{"offset": 0, "color": "#FF0000"}, {"offset": 0.4, "color": "#00FF00"},
                      {"offset": 0.6, "color": "#0000FF"}, {"offset": 1, "color": "#FFFFFF"}]);
    let to = json!([{"offset": 0, "color": "#FF0000"}, {"offset": 0.6, "color": "#00FF00"},
                    {"offset": 0.6, "color": "#0000FF"}, {"offset": 1, "color": "#FFFFFF"}]);
    linear(
        json!(90),
        keyed_eased(from, to, json!([0.34, 1.56, 0.64, 1])),
    )
}

#[test]
fn a_bezier_that_carries_an_offset_past_its_neighbour_is_a_hard_edge_and_prints_fixed() {
    let path = project_with(overshooting());
    assert!(errors_of(&path).is_empty(), "{:?}", errors_of(&path));
    let mut crossed = None;
    for t in (0..1000).step_by(50) {
        let stops = printed(&path, "box", "fill", t)["value"]["stops"].clone();
        let offsets: Vec<f64> = stops
            .as_array()
            .unwrap()
            .iter()
            .map(|stop| stop["offset"].as_f64().unwrap())
            .collect();
        assert!(
            offsets.windows(2).all(|w| w[0] <= w[1]),
            "t={t}: {offsets:?}"
        );
        if offsets[1] > 0.6 && offsets[2] == offsets[1] {
            // The raised stop keeps its own colour.
            assert_eq!(stops[2]["color"], json!("#0000FF"));
            crossed = Some(t);
        }
    }
    let crossed = crossed.expect("the curve never carried a stop past its neighbour");
    // And it paints.
    let rasters = montagent_core::verbs::render::paint_span(
        &path,
        crossed,
        crossed + 100,
        montagent_core::verbs::render::Supplying::PerFrame,
    )
    .expect("it paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
}

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

/// An instant inside [`overshooting`]'s segment where its offsets are crossed.
fn crossed_instant(path: &Path) -> i64 {
    (1..1000)
        .find(|&t| {
            let stops = printed(path, "box", "fill", t)["value"]["stops"].clone();
            stops[1]["offset"].as_f64().unwrap() > 0.6
        })
        .expect("a crossed instant")
}

#[test]
fn shift_refuses_a_split_inside_the_overshoot_naming_the_path_and_the_instant() {
    let path = project_with(overshooting());
    let at = crossed_instant(&path);
    let before = std::fs::read_to_string(&path).unwrap();
    let answer = shifted(&path, at, 100);
    let refusals: Vec<_> = answer
        .report()
        .findings
        .iter()
        .filter(|finding| finding.code == "E-SHIFT-SPLIT-UNWRITABLE")
        .collect();
    assert_eq!(refusals.len(), 1, "{:?}", answer.report().findings);
    assert_eq!(refusals[0].fields["property"], "fill.stops");
    assert_eq!(refusals[0].fields["at"], at);
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        before,
        "nothing is written"
    );
}

fn painted(path: &Path, t: i64) -> Vec<u8> {
    let rasters = montagent_core::verbs::render::paint_span(
        path,
        t,
        t + 100,
        montagent_core::verbs::render::Supplying::PerFrame,
    )
    .expect("the span paints");
    assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
    rasters.frames.into_iter().next().unwrap()
}

/// Colours split to byte rounding (ADR-0146 §7), so the frames agree to a level or two.
fn near_every_byte(a: &[u8], b: &[u8]) {
    assert_eq!(a.len(), b.len());
    let worst = a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap();
    assert!(worst <= 2, "the frames differ by {worst} levels");
}

#[test]
fn shift_splits_every_nested_list_elsewhere_and_renders_what_it_did_before() {
    let fill = json!({"gradient": "radial",
        "center": keyed(json!([0.2, 0.2]), json!([0.8, 0.6]), "linear"),
        "radius": keyed(json!(0.4), json!(1.2), "linear"),
        "stops": keyed(two("#FF3366", "#3366FF"), two("#FFCC00", "#00CCFF"), "linear")});
    let kept = project_with(fill.clone());
    let split = project_with(fill);
    // Splitting at 500 and carrying what follows 200 later leaves everything before 500 as
    // it was, holds the value from 500 to 700, and plays the rest 200 later.
    let before: Vec<Vec<u8>> = [400, 500, 800].iter().map(|t| painted(&kept, *t)).collect();
    let answer = shifted(&split, 500, 200);
    assert!(
        answer
            .report()
            .findings
            .iter()
            .all(|f| f.class != Class::Error),
        "{:?}",
        answer.report().findings
    );
    near_every_byte(&before[0], &painted(&split, 400));
    near_every_byte(&before[1], &painted(&split, 500));
    near_every_byte(&before[1], &painted(&split, 600));
    near_every_byte(&before[2], &painted(&split, 1000));
}

#[test]
fn a_keyed_kind_a_list_of_gradients_and_a_mixed_list_are_schema_errors() {
    let schema_errors = |fill: Value| -> usize {
        let path = project_with(fill);
        montagent_core::validate(&path)
            .findings
            .iter()
            .filter(|finding| finding.code.starts_with("E-SCHEMA"))
            .count()
    };
    let g = linear(json!(0), two("#FFFFFF", "#000000"));
    let keyed_kind = json!({"gradient": keyed(json!("linear"), json!("radial"), "linear"),
                            "angle": 0, "stops": two("#FFFFFF", "#000000")});
    assert!(schema_errors(keyed_kind) > 0);
    assert!(schema_errors(keyed(g.clone(), g.clone(), "linear")) > 0);
    assert!(schema_errors(keyed(json!("#FFFFFF"), g, "linear")) > 0);
}

#[test]
fn a_stops_record_with_a_different_count_is_an_error_naming_the_record() {
    let three = json!([{"offset": 0, "color": "#FFFFFF"}, {"offset": 0.5, "color": "#888888"},
                       {"offset": 1, "color": "#000000"}]);
    let path = project_with(linear(
        json!(0),
        json!([{"t": 0, "v": two("#FFFFFF", "#000000")},
               {"t": 500, "v": two("#FFFFFF", "#000000"), "ease": "linear"},
               {"t": 1000, "v": three, "ease": "linear"}]),
    ));
    let found = findings(&path, "E-GRADIENT-STOP-COUNT");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, Class::Error);
    assert_eq!(found[0].1["property"], json!("fill.stops"));
    assert_eq!(found[0].1["record"], json!(3));
    assert_eq!(found[0].1["t"], json!(1000));
    assert_eq!(found[0].1["count"], json!(3));
    assert_eq!(found[0].1["expected"], json!(2));
}

#[test]
fn stop_order_walks_every_keyframe_of_a_stops_list() {
    let crossed = json!([{"offset": 0.8, "color": "#FFFFFF"}, {"offset": 0.2, "color": "#000000"}]);
    let path = project_with(linear(
        json!(0),
        json!([{"t": 0, "v": two("#FFFFFF", "#000000")},
               {"t": 1000, "v": crossed, "ease": "linear"}]),
    ));
    let found = findings(&path, "E-GRADIENT-STOP-ORDER");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].1["property"], json!("fill.stops"));
    assert_eq!(found[0].1["in"], json!("the keyframe at t=1000"));
    assert_eq!(found[0].1["stop"], json!(2));
}

fn one_colour(fill: Value) -> usize {
    findings(&project_with(fill), "R-GRADIENT-ONE-COLOUR").len()
}

#[test]
fn a_flat_to_gradient_animation_is_not_one_colour_but_dead_weight_is() {
    let same = |c: &str| two(c, c);
    // Flat red growing a blue end: the stops differ at some instant.
    assert_eq!(
        one_colour(linear(
            json!(0),
            keyed(same("#FF0000"), two("#FF0000", "#0000FF"), "linear")
        )),
        0
    );
    // Every record shares one colour at both stops: one colour from start to finish.
    assert_eq!(
        one_colour(linear(
            json!(0),
            keyed(same("#FF0000"), same("#0000FF"), "linear")
        )),
        1
    );
    // A keyed radius that never rises above 0.
    let stops = two("#FFFFFF", "#000000");
    let radius = |from: i64, to: i64| {
        radial(
            json!([0.5, 0.5]),
            keyed(json!(from), json!(to), "linear"),
            stops.clone(),
        )
    };
    assert_eq!(one_colour(radius(0, 0)), 1);
    // One that does rise is not.
    assert_eq!(one_colour(radius(0, 1)), 0);
}

#[test]
fn the_run_override_review_covers_a_keyed_gradient() {
    let dir = tempdir(line!());
    let gradient = linear(
        keyed(json!(0), json!(90), "linear"),
        two("#FFFFFF", "#000000"),
    );
    let path = write(
        &dir,
        json!([{"id": "title", "type": "text", "start": 0, "end": 1000, "x": 100, "y": 100,
                "width": 180, "height": 60, "font": "brand", "size": 20, "color": gradient,
                "runs": [{"text": "Hi", "color": "#FFFFFF"}]}]),
    );
    let found = findings(&path, "R-TEXT-PAINT-OVERRIDDEN");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].1["property"], json!("color"));
}

#[test]
fn a_nested_list_is_held_to_the_ascending_t_and_ease_rules() {
    let schema_errors = |angle: Value| -> usize {
        let path = project_with(linear(angle, two("#FFFFFF", "#000000")));
        montagent_core::validate(&path)
            .findings
            .iter()
            .filter(|finding| finding.code.starts_with("E-SCHEMA"))
            .count()
    };
    assert_eq!(
        schema_errors(json!([{"t": 500, "v": 0}, {"t": 100, "v": 90, "ease": "linear"}])),
        1
    );
    assert_eq!(
        schema_errors(json!([{"t": 0, "v": 0}, {"t": 100, "v": 90}])),
        1
    );
}
