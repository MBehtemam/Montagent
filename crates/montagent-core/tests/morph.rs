//! A morph between unlike shapes (#767, ADR-0162): matching vertex lists written by hand,
//! `E-PATH-KEYFRAME-SHAPE` naming that fix, and `R-PATH-SEAM-CAP` on an open path whose
//! coincident ends meet at a corner under a cap that is not `round`.

use std::path::{Path, PathBuf};

use montagent_core::finding::Class;
use montagent_core::report::Report;
use montagent_core::verbs::render::Supplying;
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

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

/// A scratch directory of its own for every call, so tests running in parallel never share
/// one.
fn scratch() -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    tempdir(2_000_000 + NEXT.fetch_add(1, Ordering::Relaxed))
}

fn validated(element: Value) -> Report {
    montagent_core::validate(&project(&scratch(), &[element]))
}

/// The message of every `code` finding, as `validate` prints it, each one's wrapped lines
/// joined with single spaces.
fn messages(report: &Report, code: &str) -> Vec<String> {
    let prose =
        montagent_core::text::render(&report.to_json(), montagent_core::text::Options::verbose())
            .expect("every template's fields are carried");
    let lines: Vec<&str> = prose.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains(code))
        .map(|(at, _)| {
            lines[at + 1..]
                .iter()
                .take_while(|line| line.starts_with("  ") && !line.trim().is_empty())
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// A path element in the 200×200 frame, placed by its top-left corner, with `fields` laid
/// over it; a `null` field is removed.
fn path(fields: Value) -> Value {
    let mut element = json!({"id": "shape", "type": "path", "start": 0, "end": 1000, "x": 0,
        "y": 0, "origin": "top-left", "width": 200, "height": 200, "closed": true,
        "fill": "#FFFFFF", "points": [{"at": [40, 40]}, {"at": [160, 40]}, {"at": [160, 160]}]});
    for (key, value) in fields.as_object().expect("fields are an object") {
        if value.is_null() {
            element.as_object_mut().unwrap().remove(key);
        } else {
            element[key] = value.clone();
        }
    }
    element
}

// ---------------------------------------------------------------------------
// `E-PATH-KEYFRAME-SHAPE` names the fix.
// ---------------------------------------------------------------------------

#[test]
fn the_keyframe_shape_error_ends_by_naming_coincident_vertices_and_zero_handles() {
    let triangle = json!([{"at": [100, 30]}, {"at": [170, 160]}, {"at": [30, 160]}]);
    let square =
        json!([{"at": [40, 40]}, {"at": [160, 40]}, {"at": [160, 160]}, {"at": [40, 160]}]);
    let report = validated(path(json!({"points": [
        {"t": 0, "v": square},
        {"t": 1000, "v": triangle, "ease": "linear"}
    ]})));
    // The registry's own message ends with the fix; the printed finding carries it, ahead
    // of the refuse-class line the renderer adds to every refusal.
    let fix = "To morph between unlike shapes, give the shorter list coincident vertices (the \
same `at` twice) and write `[0, 0]` for a handle one side lacks.";
    let template = montagent_core::registry::spec("E-PATH-KEYFRAME-SHAPE")
        .expect("registered")
        .template;
    assert!(template.ends_with(fix), "{template}");
    let said = messages(&report, "E-PATH-KEYFRAME-SHAPE");
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains(fix), "{}", said[0]);
}

// ---------------------------------------------------------------------------
// `R-PATH-SEAM-CAP`.
// ---------------------------------------------------------------------------

const SEAM: &str = "R-PATH-SEAM-CAP";

/// The fields of every `R-PATH-SEAM-CAP` finding `validate` gives `element`.
fn seams(element: Value) -> Vec<Value> {
    validated(element)
        .findings
        .iter()
        .filter(|finding| finding.code == SEAM)
        .map(|finding| {
            assert_eq!(finding.class, Class::Review, "{SEAM} is a review");
            finding
                .fields
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect::<serde_json::Map<_, _>>()
                .into()
        })
        .collect()
}

/// A square written as one open, stroked path whose last `at` sits on its first: the seam at
/// the top-left corner is a corner, leaving right and arriving upward.
fn open_square() -> Value {
    json!([{"at": [40, 40]}, {"at": [160, 40]}, {"at": [160, 160]}, {"at": [40, 160]},
           {"at": [40, 40]}])
}

/// An open, stroked path with `points`, and `fields` laid over it.
fn open(points: Value, fields: Value) -> Value {
    let mut element = path(json!({"closed": false, "fill": null, "stroke": "#FFFFFF",
        "stroke_width": 4, "points": points}));
    for (key, value) in fields.as_object().expect("fields are an object") {
        if value.is_null() {
            element.as_object_mut().unwrap().remove(key);
        } else {
            element[key] = value.clone();
        }
    }
    element
}

#[test]
fn a_corner_seam_under_the_default_butt_cap_is_named_with_its_cap() {
    let fired = seams(open(open_square(), json!({})));
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["cap"], json!("butt"));
    let said = messages(&validated(open(open_square(), json!({}))), SEAM);
    assert_eq!(
        said,
        [
            "`shape`.points: the path's ends meet at a corner under a `butt` cap, which draws a \
notch at the seam. Write `\"stroke_cap\": \"round\"`, or make the seam smooth."
        ]
    );
}

#[test]
fn a_corner_seam_under_a_square_cap_is_named_as_a_spur() {
    let square = open(open_square(), json!({"stroke_cap": "square"}));
    let fired = seams(square.clone());
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(fired[0]["cap"], json!("square"));
    let said = messages(&validated(square), SEAM);
    assert!(said[0].contains("draws a spur at the seam"), "{said:?}");
    // A written `"butt"` is the default, and fires the same.
    assert_eq!(
        seams(open(open_square(), json!({"stroke_cap": "butt"}))).len(),
        1
    );
}

#[test]
fn a_round_cap_hides_every_seam() {
    assert_eq!(
        seams(open(open_square(), json!({"stroke_cap": "round"}))),
        Vec::<Value>::new()
    );
}

/// The same square, starting from the middle of its top edge, so the seam is a straight run
/// through that point: leaving right and arriving right.
fn square_from_its_top_edge() -> Value {
    json!([{"at": [100, 40]}, {"at": [160, 40]}, {"at": [160, 160]}, {"at": [40, 160]},
           {"at": [40, 40]}, {"at": [100, 40]}])
}

#[test]
fn a_smooth_seam_is_silent_under_butt_and_square() {
    for cap in [Value::Null, json!("square")] {
        assert_eq!(
            seams(open(square_from_its_top_edge(), json!({"stroke_cap": cap}))),
            Vec::<Value>::new(),
            "{cap}"
        );
    }
    // A circle written open from its top: it leaves along the first `out` and arrives along
    // the negated last `in`, both rightward.
    let circle = json!([
        {"at": [100, 40], "out": [33, 0]},
        {"at": [160, 100], "in": [0, -33], "out": [0, 33]},
        {"at": [100, 160], "in": [33, 0], "out": [-33, 0]},
        {"at": [40, 100], "in": [0, 33], "out": [0, -33]},
        {"at": [100, 40], "in": [-33, 0]}
    ]);
    assert_eq!(seams(open(circle, json!({}))), Vec::<Value>::new());
}

#[test]
fn ends_meeting_head_on_are_a_corner() {
    // Out and back along one line: the cross product is 0 and the dot product below it.
    let spike = json!([{"at": [40, 100]}, {"at": [160, 100]}, {"at": [40, 100]}]);
    assert_eq!(seams(open(spike, json!({}))).len(), 1);
}

#[test]
fn a_handle_decides_the_direction_ahead_of_the_vertices() {
    // The vertices alone make a straight run through the seam, but the first `out` leaves
    // downward, so the seam is a corner.
    let mut points = square_from_its_top_edge();
    points[0]["out"] = json!([0, 20]);
    assert_eq!(seams(open(points, json!({}))).len(), 1);
    // A last `in` that points the arrival back along the leaving direction keeps it smooth.
    let mut points = square_from_its_top_edge();
    points[5]["in"] = json!([-20, 0]);
    assert_eq!(seams(open(points, json!({}))), Vec::<Value>::new());
}

#[test]
fn the_review_is_silent_on_a_closed_path_with_a_dash_and_without_a_stroke() {
    let closed = open(open_square(), json!({"closed": true}));
    assert_eq!(seams(closed), Vec::<Value>::new(), "closed");
    let dashed = open(open_square(), json!({"stroke_dash": [6, 4]}));
    assert_eq!(seams(dashed), Vec::<Value>::new(), "dashed");
    let unstroked = open(open_square(), json!({"stroke": null, "stroke_width": null}));
    assert_eq!(seams(unstroked), Vec::<Value>::new(), "no stroke");
    // Ends that do not meet have no seam.
    let mut apart = open_square();
    apart[4]["at"] = json!([40, 60]);
    assert_eq!(seams(open(apart, json!({}))), Vec::<Value>::new(), "apart");
}

#[test]
fn an_end_with_no_direction_is_silent() {
    // Every segment is zero-length, so neither end has a direction to compare.
    let dot = json!([{"at": [50, 50]}, {"at": [50, 50], "in": [0, 0]}, {"at": [50, 50]}]);
    assert_eq!(seams(open(dot, json!({}))), Vec::<Value>::new());
}

#[test]
fn zero_length_padding_at_either_end_is_passed_over() {
    // The square, padded with a coincident vertex (the same `at` twice) at each end, `[0, 0]`
    // handles on the first: each end's direction comes from the segment past the padding.
    let padded = json!([
        {"at": [40, 40], "out": [0, 0]}, {"at": [40, 40], "in": [0, 0]}, {"at": [160, 40]},
        {"at": [160, 160]}, {"at": [40, 160]}, {"at": [40, 40]}, {"at": [40, 40]}
    ]);
    assert_eq!(seams(open(padded, json!({}))).len(), 1);
    // The smooth square, padded the same way, stays smooth.
    let mut smooth = square_from_its_top_edge().as_array().unwrap().clone();
    smooth.insert(0, json!({"at": [100, 40]}));
    smooth.push(json!({"at": [100, 40]}));
    assert_eq!(seams(open(json!(smooth), json!({}))), Vec::<Value>::new());
}

#[test]
fn each_keyframe_value_is_judged_on_its_own_and_named_by_record_and_t() {
    // A corner seam at the top-left, and the same six vertices turned into a smooth one.
    let corner = json!([{"at": [40, 40]}, {"at": [100, 40]}, {"at": [160, 40]},
                        {"at": [160, 160]}, {"at": [40, 160]}, {"at": [40, 40]}]);
    let keyed = |first: &Value, second: &Value| {
        open(
            json!({}),
            json!({"points": [{"t": 0, "v": first},
                              {"t": 1000, "v": second, "ease": "linear"}]}),
        )
    };
    let fired = seams(keyed(&corner, &square_from_its_top_edge()));
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(
        (fired[0]["record"].clone(), fired[0]["t"].clone()),
        (json!(1), json!(0))
    );
    let element = keyed(&square_from_its_top_edge(), &corner);
    let fired = seams(element.clone());
    assert_eq!(fired.len(), 1, "{fired:?}");
    assert_eq!(
        (fired[0]["record"].clone(), fired[0]["t"].clone()),
        (json!(2), json!(1000))
    );
    let said = messages(&validated(element), SEAM);
    assert!(
        said[0].starts_with("`shape`.points in keyframe record 2 (`t` 1000): "),
        "{said:?}"
    );
    assert_eq!(seams(keyed(&corner, &corner)).len(), 2);
}

// ---------------------------------------------------------------------------
// The recipe.
// ---------------------------------------------------------------------------

/// The one JSON project in `montagent-motion`'s "Morph one shape into another" recipe.
fn recipe_example() -> String {
    let skill = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/montagent-motion/SKILL.md"),
    )
    .unwrap();
    let start = skill
        .find("### Morph one shape into another")
        .expect("the recipe is in the skill");
    let recipe = &skill[start..];
    let recipe = &recipe[..recipe[4..]
        .find("\n### ")
        .map_or(recipe.len(), |end| end + 4)];
    let body = &recipe[recipe
        .find("```json\n")
        .expect("the recipe has a JSON project")
        + 8..];
    body[..body.find("```").unwrap()].to_string()
}

#[test]
fn the_recipes_square_to_triangle_validates_with_no_findings_and_paints_both_shapes() {
    let dir = scratch();
    let path = write_project(&dir, "morph.montagent.json", &canonical(&recipe_example()));
    let report = montagent_core::validate(&path);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert_eq!(codes, Vec::<&str>::new());
    // The square's top-left corner, 20 px in from its box's corner, is filled at the start
    // and outside the triangle at the end: the triangle's left side crosses that row 10 px
    // left of the apex.
    let painted = |t| {
        let rasters =
            montagent_core::verbs::render::paint_span(&path, t, t + 1, Supplying::PerFrame)
                .expect("the span paints");
        assert!(rasters.declined.is_empty(), "{:?}", rasters.declined);
        rasters.frames.into_iter().next().unwrap()
    };
    let pixel = |rgb: &[u8], x: usize, y: usize| {
        let at = (y * 1920 + x) * 3;
        [rgb[at], rgb[at + 1], rgb[at + 2]]
    };
    let (start, end) = (painted(0), painted(1900));
    assert_eq!(pixel(&start, 820, 400), [0xFF, 0x5A, 0x36], "the square");
    assert_eq!(
        pixel(&end, 820, 400),
        [0x10, 0x14, 0x18],
        "outside the triangle"
    );
    assert_eq!(
        pixel(&end, 960, 500),
        [0xFF, 0x5A, 0x36],
        "inside the triangle"
    );
}

#[test]
fn a_circle_to_star_written_by_the_recipe_validates_clean() {
    // A circle of radius 60 as four cubics with handles of 0.5523 × 60, rounded to 33, each
    // corner padded with coincident vertices (the same `at` twice) to the star's ten; the
    // star writes `[0, 0]` for every handle. Both start at the top and run clockwise.
    let corner = |at: [i64; 2], arriving: [i64; 2], leaving: [i64; 2], copies: usize| {
        (0..copies).map(move |copy| {
            let first = if copy == 0 { arriving } else { [0, 0] };
            let last = if copy + 1 == copies { leaving } else { [0, 0] };
            json!({"at": at, "in": first, "out": last})
        })
    };
    let circle: Vec<Value> = corner([100, 40], [-33, 0], [33, 0], 3)
        .chain(corner([160, 100], [0, -33], [0, 33], 2))
        .chain(corner([100, 160], [33, 0], [-33, 0], 2))
        .chain(corner([40, 100], [0, 33], [0, -33], 3))
        .collect();
    let star: Vec<Value> = [
        [100, 30],
        [118, 76],
        [167, 78],
        [129, 109],
        [141, 157],
        [100, 130],
        [59, 157],
        [71, 109],
        [33, 78],
        [82, 76],
    ]
    .into_iter()
    .map(|at| json!({"at": at, "in": [0, 0], "out": [0, 0]}))
    .collect();
    let report = validated(path(json!({"points": [
        {"t": 0, "v": circle},
        {"t": 900, "v": star, "ease": [0.65, 0, 0.35, 1]}
    ]})));
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert_eq!(codes, Vec::<&str>::new());
}
