//! PROTOTYPE (#614): an image's `swaps`, through every surface the resolution of #595
//! names — the renderer, `validate`'s disk checks, `R-SOURCE-CUT-POP`, `query --at` and
//! `shift`. The document checks (order, overlap, range) are unit-tested beside them.

use std::path::{Path, PathBuf};

use montagent_core::report::{ExitCode, Report};
use montagent_core::verbs::query::{Ask as QueryAsk, query};
use montagent_core::verbs::shift::{Ask as ShiftAsk, shift};
use serde_json::{Value, json};

mod common;
use common::{canonical, has_ffprobe, write_project};

/// A flat-colour PNG at `size`, written into `dir`.
fn png(dir: &Path, name: &str, size: (u32, u32), rgb: [u8; 3]) {
    image::RgbaImage::from_pixel(size.0, size.1, image::Rgba([rgb[0], rgb[1], rgb[2], 255]))
        .save(dir.join(name))
        .expect("write a png");
}

const RED: [u8; 3] = [255, 0, 0];
const BLUE: [u8; 3] = [0, 0, 255];
const GREEN: [u8; 3] = [0, 255, 0];

/// One image element over `[0, 1000)` filling a 100×100 frame, base `red.png`, with
/// whatever `extra` keys the test adds; the three source images written beside it.
fn project(dir: &Path, extra: Value) -> PathBuf {
    for (name, rgb) in [("red.png", RED), ("blue.png", BLUE), ("green.png", GREEN)] {
        png(dir, name, (100, 100), rgb);
    }
    let mut element = json!({"id":"m","type":"image","start":0,"end":1000,"source":"red.png",
        "x":0,"y":0,"origin":"top-left","width":100,"height":100,"fit":"literal"});
    for (key, value) in extra.as_object().unwrap() {
        element[key] = value.clone();
    }
    let document = json!({"frame":{"width":100,"height":100},"fps":10,"duration":1000,
        "tracks":[{"name":"t","layer":1,"elements":[element]}]});
    write_project(dir, "p.montagent.json", &canonical(&document.to_string()))
}

fn codes(report: &Report) -> Vec<&str> {
    report.findings.iter().map(|f| f.code.as_str()).collect()
}

fn centre(project: &Path, at: i64) -> [u8; 3] {
    let picture = common::compare::rendered(project, at, true);
    let pixel = picture.get_pixel(50, 50).0;
    [pixel[0], pixel[1], pixel[2]]
}

#[test]
fn each_frame_draws_the_swap_holding_it_else_the_base() {
    let dir = common::tempdir(line!());
    let path = project(
        &dir,
        json!({"swaps":[{"start":200,"source":"blue.png"},{"start":500,"end":700,"source":"green.png"}]}),
    );
    assert_eq!(centre(&path, 100), RED);
    assert_eq!(centre(&path, 200), BLUE);
    assert_eq!(
        centre(&path, 400),
        BLUE,
        "an endless swap runs to the next start"
    );
    assert_eq!(centre(&path, 600), GREEN);
    assert_eq!(
        centre(&path, 700),
        RED,
        "after an explicit end the base shows again"
    );
}

#[test]
fn a_missing_swap_file_is_the_missing_source_error_on_its_element() {
    if !has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let path = project(&dir, json!({"swaps":[{"start":200,"source":"absent.png"}]}));
    let report = montagent_core::validate(&path);
    let missing: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-SOURCE-MISSING")
        .collect();
    assert_eq!(missing.len(), 1, "{:?}", codes(&report));
    assert_eq!(missing[0].location.element.as_deref(), Some("m"));
}

#[test]
fn a_swap_file_of_another_size_is_a_fit_deviation_naming_that_file() {
    if !has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let path = project(
        &dir,
        json!({"fit":"cover","clip":[0,0,100,100],
               "swaps":[{"start":200,"source":"tall.png"}]}),
    );
    png(&dir, "tall.png", (100, 200), BLUE);
    let report = montagent_core::validate(&path);
    let deviations: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-FIT-DEVIATION")
        .collect();
    assert_eq!(deviations.len(), 1, "{:?}", codes(&report));
    assert_eq!(deviations[0].fields["source"], "tall.png");
}

#[test]
fn swaps_on_any_other_type_is_an_error_never_ignored() {
    let dir = common::tempdir(line!());
    let document = json!({"frame":{"width":100,"height":100},"fps":10,"tracks":[{"name":"t",
        "layer":1,"elements":[{"id":"r","type":"rect","start":0,"end":1000,"width":10,
        "height":10,"fill":"#FF0000","swaps":[{"start":200,"source":"blue.png"}]}]}]});
    let path = write_project(&dir, "p.montagent.json", &canonical(&document.to_string()));
    let report = montagent_core::validate(&path);
    // One mistake, one finding: the key is refused, and the file it names is not probed
    // as if something drew it.
    assert_eq!(codes(&report), ["E-SCHEMA-UNKNOWN-KEY"]);
}

#[test]
fn query_at_reports_the_source_showing() {
    let dir = common::tempdir(line!());
    let path = project(
        &dir,
        json!({"swaps":[{"start":200,"end":400,"source":"blue.png"}]}),
    );
    let source_at = |instant| {
        let answer = query(
            &path,
            &QueryAsk {
                at: Some(instant),
                ..QueryAsk::default()
            },
        );
        answer.to_json()["query"]["stack"][0]["source"].clone()
    };
    assert_eq!(source_at(100), "red.png");
    assert_eq!(source_at(300), "blue.png");
    assert_eq!(source_at(400), "red.png");
}

fn shifted(extra: Value, at: i64, delta: i64, line: u32) -> Value {
    let dir = common::tempdir(line);
    let path = project(&dir, extra);
    let answer = shift(
        &path,
        &ShiftAsk {
            at,
            delta,
            scope: None,
            release: Vec::new(),
        },
    );
    assert_ne!(
        answer.report().exit_code(),
        ExitCode::Errors,
        "{:?}",
        codes(answer.report())
    );
    let written: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    written["tracks"][0]["elements"][0]["swaps"].clone()
}

#[test]
fn shift_on_a_straddler_moves_later_swaps_and_lengthens_the_one_holding_at() {
    let swaps = shifted(
        json!({"swaps":[
            {"start":100,"end":300,"source":"blue.png"},
            {"start":300,"source":"green.png"},
            {"start":600,"end":800,"source":"blue.png"}
        ]}),
        500,
        100,
        line!(),
    );
    assert_eq!(
        swaps,
        json!([
            {"start":100,"end":300,"source":"blue.png"},
            {"start":300,"source":"green.png"},
            {"start":700,"end":900,"source":"blue.png"}
        ])
    );

    let swaps = shifted(
        json!({"swaps":[{"start":100,"end":600,"source":"blue.png"}]}),
        500,
        100,
        line!(),
    );
    assert_eq!(swaps, json!([{"start":100,"end":700,"source":"blue.png"}]));
}

#[test]
fn shift_holds_a_swap_starting_exactly_at_at_and_leaves_an_end_at_at() {
    let swaps = shifted(
        json!({"swaps":[
            {"start":100,"end":500,"source":"blue.png"},
            {"start":500,"end":600,"source":"green.png"}
        ]}),
        500,
        100,
        line!(),
    );
    assert_eq!(
        swaps,
        json!([
            {"start":100,"end":500,"source":"blue.png"},
            {"start":500,"end":700,"source":"green.png"}
        ])
    );
}

#[test]
fn shift_moves_every_swap_of_an_element_wholly_after_at() {
    let dir = common::tempdir(line!());
    let path = project(
        &dir,
        json!({"start":200,"swaps":[{"start":200,"end":300,"source":"blue.png"}]}),
    );
    let answer = shift(
        &path,
        &ShiftAsk {
            at: 100,
            delta: 50,
            scope: None,
            release: Vec::new(),
        },
    );
    assert_ne!(answer.report().exit_code(), ExitCode::Errors);
    let written: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let element = &written["tracks"][0]["elements"][0];
    assert_eq!(element["start"], 250);
    assert_eq!(
        element["swaps"],
        json!([{"start":250,"end":350,"source":"blue.png"}])
    );
}

/// Two abutting elements on one track; `a` ends on `a_last` and `b` starts on red.
fn cut_codes(a_swaps: Value, line: u32) -> Vec<String> {
    let dir = common::tempdir(line);
    for (name, rgb) in [("red.png", RED), ("blue.png", BLUE)] {
        png(&dir, name, (100, 100), rgb);
    }
    let base = json!({"type":"image","source":"red.png","x":0,"y":0,"origin":"top-left",
        "width":100,"height":100,"fit":"literal"});
    let mut a = base.clone();
    a["id"] = json!("a");
    a["start"] = json!(0);
    a["end"] = json!(500);
    a["scale"] = json!([1.2, 1.2]);
    a["swaps"] = a_swaps;
    let mut b = base;
    b["id"] = json!("b");
    b["start"] = json!(500);
    b["end"] = json!(1000);
    let document = json!({"frame":{"width":100,"height":100},"fps":10,"duration":1000,
        "tracks":[{"name":"t","layer":1,"elements":[a,b]}]});
    let path = write_project(&dir, "p.montagent.json", &canonical(&document.to_string()));
    montagent_core::validate(&path)
        .findings
        .iter()
        .map(|f| f.code.clone())
        .collect()
}

#[test]
fn the_cut_pop_compares_the_file_showing_on_each_side_of_the_seam() {
    // `a` leaves on blue and `b` enters on red: two files, so no continuous shot to pop.
    let codes = cut_codes(json!([{"start":300,"source":"blue.png"}]), line!());
    assert!(!codes.iter().any(|c| c == "R-SOURCE-CUT-POP"), "{codes:?}");

    // `a`'s swap ended, so it leaves on its base, red, as `b` enters: one shot, and the
    // scale jump pops.
    let codes = cut_codes(
        json!([{"start":100,"end":300,"source":"blue.png"}]),
        line!(),
    );
    assert!(codes.iter().any(|c| c == "R-SOURCE-CUT-POP"), "{codes:?}");
}
