//! `origin` as a free point `[px, py]` — prototype #612 of the decision in #596.
//!
//! Integer pixels in the element's own unscaled box, from its top-left, free to lie outside
//! it, static, on `image`, `video`, `rect` and `ellipse`; text keeps the nine keywords.
//! Every project here is built from shapes, so nothing needs an `ffprobe`.

use montagent_core::finding::Finding;
use montagent_core::report::{ExitCode, Report};
use montagent_core::verbs::fmt::{self, Mode};
use montagent_core::verbs::query::{Ask, query};
use montagent_core::{checks, parse, validate};
use serde_json::{Value, json};

mod common;
use common::compare::rendered;
use common::{canonical, write_project};

fn project(width: i64, height: i64, elements: &str) -> String {
    canonical(&format!(
        r##"{{"frame":{{"width":{width},"height":{height}}},"fps":25,"tracks":[{{"name":"t","layer":1,"elements":[{elements}]}}]}}"##
    ))
}

/// A painted rect with `origin` and whatever else a test splices in, in canonical order.
fn rect(id: &str, x: i64, y: i64, origin: &str, width: i64, height: i64, rest: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"rect","start":0,"end":1000,"x":{x},"y":{y},"origin":{origin},"width":{width},"height":{height},"fill":"#FF0000"{rest}}}"##
    )
}

#[track_caller]
fn validated(body: &str) -> Report {
    let dir = common::tempdir(std::panic::Location::caller().line());
    validate(&write_project(&dir, "p.montagent.json", body))
}

fn schema_reasons(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.code.starts_with("E-SCHEMA"))
        .map(|f| f.fields["reason"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[track_caller]
fn one_reason(body: &str) -> String {
    let reasons = schema_reasons(&validated(body));
    assert_eq!(
        reasons.len(),
        1,
        "expected one schema finding, got {reasons:?}"
    );
    reasons.into_iter().next().unwrap()
}

// ---------------------------------------------------------------------------
// Schema.
// ---------------------------------------------------------------------------

#[test]
fn a_point_validates_on_every_type_that_takes_one_and_outside_the_box_too() {
    for origin in ["[20, 30]", "[0, 0]", "[-40, 500]"] {
        let report = validated(&project(
            1080,
            1920,
            &rect("r", 540, 960, origin, 100, 100, ""),
        ));
        assert_eq!(
            report
                .findings
                .iter()
                .map(|f| f.code.as_str())
                .collect::<Vec<_>>(),
            Vec::<&str>::new(),
            "origin {origin} should validate clean — a point outside the box is legal and \
             review noise counts against the feature"
        );
        assert_eq!(report.exit_code(), ExitCode::Ok);
    }
    let ellipse = r##"{"id":"e","type":"ellipse","start":0,"end":1000,"x":540,"y":960,"origin":[50,0],"width":100,"height":100,"fill":"#FF0000"}"##;
    assert!(schema_reasons(&validated(&project(1080, 1920, ellipse))).is_empty());
}

#[test]
fn a_fractional_point_names_the_keywords_and_the_point_form() {
    let reason = one_reason(&project(
        1080,
        1920,
        &rect("r", 540, 960, "[84.5, 10]", 169, 100, ""),
    ));
    assert!(reason.contains("integer"), "{reason}");
    assert!(
        reason.contains("`top-left`") && reason.contains("`bottom-right`"),
        "{reason}"
    );
    assert!(reason.contains("[px, py]"), "{reason}");
}

#[test]
fn a_wrong_length_point_names_the_keywords_and_the_point_form() {
    let reason = one_reason(&project(
        1080,
        1920,
        &rect("r", 540, 960, "[1, 2, 3]", 100, 100, ""),
    ));
    assert!(reason.contains("two coordinates, not 3"), "{reason}");
    assert!(
        reason.contains("`center`") && reason.contains("[px, py]"),
        "{reason}"
    );
}

#[test]
fn an_unknown_keyword_still_names_both_forms() {
    let reason = one_reason(&project(
        1080,
        1920,
        &rect("r", 540, 960, "\"middle\"", 100, 100, ""),
    ));
    assert!(
        reason.contains("\"middle\"") && reason.contains("[px, py]"),
        "{reason}"
    );
}

#[test]
fn a_point_on_text_names_only_the_nine_keywords() {
    let text = r##"{"id":"cap","type":"text","start":0,"end":1000,"x":540,"y":960,"origin":[0,0],"width":600,"height":100,"font":"brand","size":60,"runs":[{"text":"hi"}]}"##;
    let reason = one_reason(&project(1080, 1920, text));
    assert!(
        reason.contains("text takes one of the nine keywords only"),
        "{reason}"
    );
    assert!(reason.contains("`top-left`"), "{reason}");
    assert!(
        !reason.contains("[px, py]"),
        "offering the point form on text sends the agent round the loop: {reason}"
    );
}

// ---------------------------------------------------------------------------
// `query --at`: the rect maths resolves through the point.
// ---------------------------------------------------------------------------

#[track_caller]
fn not_covered(elements: &str) -> Value {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(100, 100, elements));
    let answer = query(
        &path,
        &Ask {
            at: Some(0),
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok);
    answer.to_json()["query"]["not_covered"].clone()
}

#[test]
fn the_drawn_rect_is_placed_through_the_point_and_scaled_about_it() {
    // A 100×40 rect whose point is 60 px above its top edge, placed at (0, 0): the box
    // hangs from y = 60, leaving the top 60 rows uncovered.
    assert_eq!(
        not_covered(&rect("r", 0, 0, "[0, -60]", 100, 40, "")),
        json!([{"x": 0, "y": 0, "width": 100, "height": 60}])
    );
    // Half the size at scale 2, its point half as far: the same footprint, because the
    // point is the scale's fixed point.
    assert_eq!(
        not_covered(&rect("r", 0, 0, "[0, -30]", 50, 20, r#","scale":[2,2]"#)),
        json!([{"x": 0, "y": 0, "width": 100, "height": 60}])
    );
}

// ---------------------------------------------------------------------------
// Render: a point and the keyword it coincides with draw the same pixels.
// ---------------------------------------------------------------------------

#[track_caller]
fn picture(elements: &str) -> image::RgbaImage {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(200, 200, elements));
    rendered(&path, 0, true)
}

#[test]
fn a_point_at_a_keywords_spot_renders_pixel_identical_to_the_keyword() {
    let transform = r#","scale":[1.3,0.7],"rotation":17"#;
    for (keyword, point) in [
        ("\"top-left\"", "[0, 0]"),
        ("\"center\"", "[30, 20]"),
        ("\"bottom-right\"", "[60, 40]"),
    ] {
        let by_keyword = picture(&rect("r", 100, 100, keyword, 60, 40, transform));
        let by_point = picture(&rect("r", 100, 100, point, 60, 40, transform));
        assert!(
            by_keyword == by_point,
            "{keyword} and {point} should draw the same pixels on a 60×40 box"
        );
    }
}

#[test]
fn rotation_pivots_about_a_point_outside_the_box() {
    // Rotating 180° about a point 50 px left of the box's left edge mirrors the box to the
    // other side of the pivot: the same pixels as the box drawn there unrotated.
    let rotated = picture(&rect(
        "r",
        100,
        100,
        "[-50, 10]",
        40,
        20,
        r#","rotation":180"#,
    ));
    let placed = picture(&rect("r", 10, 90, "\"top-left\"", 40, 20, ""));
    assert!(rotated == placed);
}

// ---------------------------------------------------------------------------
// `R-SOURCE-CUT-POP`: compare resolved points, not spellings.
// ---------------------------------------------------------------------------

fn photo(id: &str, start: i64, end: i64, origin: &str) -> String {
    format!(
        r##"{{"id":"{id}","type":"image","start":{start},"end":{end},"source":"images/05.png","x":0,"y":0,"origin":{origin},"width":1080,"height":1912,"fit":"cover"}}"##
    )
}

#[track_caller]
fn pops(elements: &str) -> Vec<Finding> {
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &project(1080, 1920, elements));
    let document = parse::read(&path).expect("the project parses");
    let mut report = Report::new("validate", Some(document.path().to_string()));
    checks::cut::check(&document, &mut report);
    report
        .findings
        .into_iter()
        .filter(|f| f.code == "R-SOURCE-CUT-POP")
        .collect()
}

#[test]
fn a_keyword_and_the_point_it_resolves_to_are_one_reference_frame_across_a_cut() {
    let elements = format!(
        "{},{}",
        photo("a", 0, 1000, "\"top-left\""),
        photo("b", 1000, 2000, "[0, 0]")
    );
    assert!(pops(&elements).is_empty());

    let elements = format!(
        "{},{}",
        photo("a", 0, 1000, "\"center\""),
        photo("b", 1000, 2000, "[540, 956]")
    );
    assert!(pops(&elements).is_empty());
}

#[test]
fn a_point_that_resolves_elsewhere_is_an_origin_trigger() {
    let elements = format!(
        "{},{}",
        photo("a", 0, 1000, "\"center\""),
        photo("b", 1000, 2000, "[0, 0]")
    );
    let found = pops(&elements);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].fields["properties"], json!(["origin"]));
}

// ---------------------------------------------------------------------------
// `fmt` never converts between the two claims.
// ---------------------------------------------------------------------------

#[test]
fn fmt_leaves_both_forms_as_written() {
    let body = project(
        1080,
        1920,
        &format!(
            "{},{}",
            rect("k", 540, 960, "\"top-left\"", 100, 100, ""),
            rect("p", 540, 960, "[0, 0]", 100, 100, "")
        ),
    );
    let dir = common::tempdir(std::panic::Location::caller().line());
    let path = write_project(&dir, "p.montagent.json", &body);
    fmt::fmt(&path, Mode::Write);
    let written: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("readable")).expect("json");
    let elements = &written["tracks"][0]["elements"];
    assert_eq!(elements[0]["origin"], json!("top-left"));
    assert_eq!(elements[1]["origin"], json!([0, 0]));
}

#[test]
fn a_mask_is_measured_from_the_box_top_left_whatever_the_point() {
    // The mask's rect is element-local from the box's top-left (ADR-0084), so moving the
    // pivot and `x`,`y` together leaves the masked picture where it was.
    let mask =
        r##","effects":[{"name":"mask","shape":"circle","x":10,"y":0,"width":40,"height":40}]"##;
    let by_keyword = picture(&rect("r", 40, 60, "\"top-left\"", 60, 40, mask));
    let by_point = picture(&rect("r", 60, 70, "[20, 10]", 60, 40, mask));
    assert!(by_keyword == by_point);
}
