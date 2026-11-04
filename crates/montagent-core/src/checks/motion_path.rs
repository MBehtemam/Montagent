//! Semantic validation checks for `motion_path` (ADR-0218 §4).
//!
//! - `E-MOTION-PATH-XY`: an element carries both `motion_path` and `x` or `y`.
//! - `E-PATH-TOO-FEW-POINTS`: `motion_path.points` has fewer than 2 vertices (open or closed).
//! - `E-PATH-DANGLING-HANDLE`: on an open `motion_path` (`closed: false`), vertex `0` carries
//!   `in` or the final vertex carries `out`.
//! - `E-MOTION-PATH-ZERO-LENGTH`: `motion_path.points` has `>= 2` vertices and total arc length
//!   `L == 0`.
//! - `E-MOTION-PATH-OFFSET-RANGE`: on an open `motion_path` (`closed: false`), a static or
//!   keyframe `motion_path.offset` value lies in `[-1.0, 0.0)` or `(1.0, 2.0]` (outside
//!   `[0.0, 1.0]`).

use montagent_render::canvas::Curve;
use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::Points;
use crate::permissive::Loose;
use crate::report::Report;
use crate::verbs::query::geometry;

pub fn check(document: &Loose, report: &mut Report) {
    walk_tracks(&document.value()["tracks"], &mut |track, element| {
        check_element(document, track, element, report);
    });
}

fn walk_tracks<'a>(tracks: &'a Value, f: &mut dyn FnMut(Option<&'a str>, &'a Value)) {
    let Some(tracks) = tracks.as_array() else {
        return;
    };
    for track in tracks {
        let name = track.get("name").and_then(Value::as_str);
        let Some(elements) = track.get("elements").and_then(Value::as_array) else {
            continue;
        };
        for element in elements {
            walk_element(name, element, f);
        }
    }
}

fn walk_element<'a>(
    track: Option<&'a str>,
    element: &'a Value,
    f: &mut dyn FnMut(Option<&'a str>, &'a Value),
) {
    f(track, element);
    if element.get("type").and_then(Value::as_str) == Some("nest")
        && let Some(children) = element.get("elements").and_then(Value::as_array)
    {
        for child in children {
            walk_element(track, child, f);
        }
    }
}

fn check_element(document: &Loose, track: Option<&str>, element: &Value, report: &mut Report) {
    if !geometry::covers_the_frame(element.get("type").and_then(Value::as_str)) {
        return;
    }
    let Some(mp) = element.get("motion_path").filter(|v| v.is_object()) else {
        return;
    };
    let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
    let mut push = |finding: Finding| {
        let finding = finding.at_file(document.path()).at_element(&subject);
        report.push(match track {
            Some(track) => finding.at_track(track),
            None => finding,
        });
    };

    let has_x = element.get("x").is_some();
    let has_y = element.get("y").is_some();
    if has_x || has_y {
        let fields = match (has_x, has_y) {
            (true, true) => "`x` and `y`",
            (true, false) => "`x`",
            (false, true) => "`y`",
            (false, false) => unreachable!(),
        };
        push(Finding::new("E-MOTION-PATH-XY").field("fields", json!(fields)));
    }

    let closed = mp.get("closed").and_then(Value::as_bool);
    let points = mp
        .get("points")
        .and_then(|v| serde_json::from_value::<Points>(v.clone()).ok())
        .map(|p| p.0);

    if let (Some(closed), Some(vertices)) = (closed, points) {
        if vertices.len() < 2 {
            push(
                Finding::new("E-PATH-TOO-FEW-POINTS")
                    .field("property", json!("motion_path.points"))
                    .field("count", json!(vertices.len()))
                    .field("least", json!(2))
                    .field("shape", json!(if closed { "closed" } else { "open" })),
            );
        } else {
            let mut has_dangling = false;
            if !closed {
                if vertices
                    .first()
                    .is_some_and(|first| first.arriving.is_some())
                {
                    has_dangling = true;
                    push(
                        Finding::new("E-PATH-DANGLING-HANDLE")
                            .field("property", json!("motion_path.points"))
                            .field("vertex", json!(0))
                            .field("handle", json!("in"))
                            .repair_value(json!({
                                "value": "drop the `in` from vertex 0"
                            })),
                    );
                }
                let last = vertices.len() - 1;
                if vertices.last().is_some_and(|last_v| last_v.out.is_some()) {
                    has_dangling = true;
                    push(
                        Finding::new("E-PATH-DANGLING-HANDLE")
                            .field("property", json!("motion_path.points"))
                            .field("vertex", json!(last))
                            .field("handle", json!("out"))
                            .repair_value(json!({
                                "value": format!("drop the `out` from vertex {last}")
                            })),
                    );
                }
            }
            if !has_dangling && let Some((_, outline)) = crate::motion_path::outline_of(mp) {
                let curve = Curve::of(&outline);
                if curve.length <= 0.0 || curve.length.is_nan() {
                    push(Finding::new("E-MOTION-PATH-ZERO-LENGTH"));
                }
            }
        }
    }

    if closed == Some(false) {
        if let Some(records) = crate::animatable::records(mp, "offset") {
            for (index, record) in records.iter().enumerate() {
                if let Some(v) = record.get("v").and_then(Value::as_f64)
                    && (-1.0..=2.0).contains(&v)
                    && !(0.0..=1.0).contains(&v)
                {
                    push(
                        Finding::new("E-MOTION-PATH-OFFSET-RANGE")
                            .field("record", json!(index + 1))
                            .field("t", json!(record.get("t").and_then(Value::as_i64)))
                            .field("value", json!(v)),
                    );
                }
            }
        } else if let Some(v) = mp.get("offset").and_then(Value::as_f64)
            && (-1.0..=2.0).contains(&v)
            && !(0.0..=1.0).contains(&v)
        {
            push(Finding::new("E-MOTION-PATH-OFFSET-RANGE").field("value", json!(v)));
        }
    }
}
