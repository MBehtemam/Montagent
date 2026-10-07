//! What the schema cannot state about a `path` (ADR-0154 §6), as four errors.
//!
//! - `E-PATH-TOO-FEW-POINTS`: an open path has at least two vertices and a closed path at
//!   least three, in every literal value.
//! - `E-PATH-DANGLING-HANDLE`: on an open path, an `in` on the first vertex or an `out` on
//!   the last shapes no segment, and nothing in the format is silently ignored.
//! - `E-PATH-KEYFRAME-SHAPE`: the values of one `points` keyframe list differ in vertex count
//!   or in which handles a vertex carries, so there is no correspondence to interpolate
//!   along. Named at the first record and the first vertex that differ from the first value.
//! - `E-PATH-OUTSIDE-BOX`: a vertex or absolute handle outside the inset box
//!   `[m, width − m] × [m, height − m]`, where `m = ceil(k × stroke_width / 2)` — from the
//!   largest value a keyed `stroke_width` states — or `0` with no stroke. `k` is the stroke's
//!   reach factor ([`crate::stroke::reach`], ADR-0158 §4): the miter limit for a miter join,
//!   √2 for a square cap that draws, else 1. A cubic Bezier lies inside the convex hull of
//!   its control points, and a centred stroke reaches no further than `k` times half its
//!   width, so this check proves from the file alone that the box contains the ink. The
//!   bound is worst-case, and the finding says so.
//!
//! Every **literal** value of `points` is checked — the static value, or each keyframe's —
//! and never a resolved one: an in-between value is the resolver's, which clamps an
//! overshoot into the same inset box (`crate::animatable`). A value that does not read as a
//! vertex list is the schema check's to report and is passed over here.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::{Points, Vertex};
use crate::permissive::Loose;
use crate::report::Report;

/// One literal value of `points`, and where it was written: `None` for a static value, or
/// the keyframe record's 1-based position and its `t`.
struct Literal {
    record: Option<(usize, Option<i64>)>,
    vertices: Vec<Vertex>,
}

pub fn check(document: &Loose, report: &mut Report) {
    for (track, element) in document.elements_in_tracks() {
        if element.get("type").and_then(Value::as_str) != Some("path") {
            continue;
        }
        let subject = crate::checks::subject_of(element.get("id").and_then(Value::as_str));
        let mut push = |finding: Finding| {
            let finding = finding.at_file(document.path()).at_element(&subject);
            report.push(match track {
                Some(track) => finding.at_track(track),
                None => finding,
            });
        };
        let literals = literals(element);
        let closed = element.get("closed").and_then(Value::as_bool);
        if let Some(closed) = closed {
            for literal in &literals {
                too_few(literal, closed).into_iter().for_each(&mut push);
                if !closed {
                    dangling(literal).into_iter().for_each(&mut push);
                }
            }
        }
        if let Some(finding) = keyframe_shape(&literals) {
            push(finding);
        }
        for literal in &literals {
            outside(element, literal).into_iter().for_each(&mut push);
        }
    }
}

/// Every literal value of the element's `points` that reads as a vertex list.
fn literals(element: &Value) -> Vec<Literal> {
    let read = |value: &Value| {
        serde_json::from_value::<Points>(value.clone())
            .ok()
            .map(|points| points.0)
    };
    match crate::animatable::records(element, "points") {
        Some(records) => records
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                Some(Literal {
                    record: Some((index + 1, record.get("t").and_then(Value::as_i64))),
                    vertices: read(record.get("v")?)?,
                })
            })
            .collect(),
        None => element
            .get("points")
            .and_then(read)
            .map(|vertices| {
                vec![Literal {
                    record: None,
                    vertices,
                }]
            })
            .unwrap_or_default(),
    }
}

/// `finding` with the keyframe record a literal was written in, where it was one.
fn located(finding: Finding, literal: &Literal) -> Finding {
    match literal.record {
        Some((record, t)) => finding.field("record", json!(record)).field("t", json!(t)),
        None => finding,
    }
}

fn too_few(literal: &Literal, closed: bool) -> Option<Finding> {
    let least = if closed { 3 } else { 2 };
    (literal.vertices.len() < least).then(|| {
        located(
            Finding::new("E-PATH-TOO-FEW-POINTS")
                .field("count", json!(literal.vertices.len()))
                .field("least", json!(least))
                .field("shape", json!(if closed { "closed" } else { "open" })),
            literal,
        )
    })
}

fn dangling(literal: &Literal) -> Vec<Finding> {
    let mut out = Vec::new();
    let last = literal.vertices.len().saturating_sub(1);
    let mut name = |vertex: usize, handle: &str| {
        out.push(located(
            Finding::new("E-PATH-DANGLING-HANDLE")
                .field("vertex", json!(vertex))
                .field("handle", json!(handle))
                .repair_value(json!({
                    "value": format!("drop the `{handle}` from vertex {vertex}")
                })),
            literal,
        ));
    };
    if literal
        .vertices
        .first()
        .is_some_and(|first| first.arriving.is_some())
    {
        name(0, "in");
    }
    if literal
        .vertices
        .last()
        .is_some_and(|last| last.out.is_some())
    {
        name(last, "out");
    }
    out
}

/// Which handles a vertex carries: the part of its shape a keyframe list must keep.
fn handles(vertex: &Vertex) -> (bool, bool) {
    (vertex.arriving.is_some(), vertex.out.is_some())
}

fn keyframe_shape(literals: &[Literal]) -> Option<Finding> {
    let (first, rest) = literals.split_first()?;
    first.record?;
    for literal in rest {
        let differs = (0..first.vertices.len().max(literal.vertices.len())).find(|&index| {
            match (first.vertices.get(index), literal.vertices.get(index)) {
                (Some(a), Some(b)) => handles(a) != handles(b),
                _ => true,
            }
        });
        if let Some(vertex) = differs {
            let detail = if first.vertices.len() != literal.vertices.len()
                && vertex >= first.vertices.len().min(literal.vertices.len())
            {
                format!(
                    "it has {} vertices, and the first has {}",
                    literal.vertices.len(),
                    first.vertices.len()
                )
            } else {
                "it carries different handles (`in`, `out`, both or neither)".to_string()
            };
            return Some(located(
                Finding::new("E-PATH-KEYFRAME-SHAPE")
                    .field("vertex", json!(vertex))
                    .field("detail", json!(detail)),
                literal,
            ));
        }
    }
    None
}

fn outside(element: &Value, literal: &Literal) -> Vec<Finding> {
    let side = |key| element.get(key).and_then(Value::as_i64);
    let (Some(width), Some(height)) = (side("width"), side("height")) else {
        return Vec::new();
    };
    let reach = crate::stroke::reach(element);
    let m = reach.inset;
    let inside = |[x, y]: [i64; 2]| (m..=width - m).contains(&x) && (m..=height - m).contains(&y);
    let mut out = Vec::new();
    for (index, vertex) in literal.vertices.iter().enumerate() {
        let [x, y] = vertex.at;
        let points = [
            ("at", Some([0, 0])),
            ("in", vertex.arriving),
            ("out", vertex.out),
        ];
        for (handle, offset) in points {
            let Some([dx, dy]) = offset else {
                continue;
            };
            let position = [x + dx, y + dy];
            if inside(position) {
                continue;
            }
            out.push(located(
                Finding::new("E-PATH-OUTSIDE-BOX")
                    .field("vertex", json!(index))
                    .field("handle", json!(handle))
                    .field("position", json!(position))
                    .field("inset", json!(m))
                    .field("right", json!(width - m))
                    .field("bottom", json!(height - m))
                    .field("k", json!(reach.source.factor()))
                    .field("width", json!(reach.width))
                    .field("source", json!(reach.source.describe())),
                literal,
            ));
        }
    }
    out
}
