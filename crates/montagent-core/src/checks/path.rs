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
//! One review joins them (ADR-0162 §4), `R-PATH-SEAM-CAP`: an open, stroked, undashed path
//! under a `butt` or `square` cap whose first and last `at` coincide, where the seam is a
//! corner. Decided in exact integers from the file alone.
//!
//! Every **literal** value of `points` is checked — the static value, or each keyframe's —
//! and never a resolved one: an in-between value is the resolver's, which clamps an
//! overshoot into the same inset box (`crate::animatable`). A value that does not read as a
//! vertex list is the schema check's to report and is passed over here.
//!
//! # Hosts
//!
//! A vertex list has more than one host: a `path` element, a `path` mask (ADR-0163 §6),
//! and a text's inline path (ADR-0161). The checks run on a [`Host`], which says where the
//! list is written, what a finding calls it, whether it is closed, and the box it must lie
//! in. Each host builds its own and calls [`points`]; none keeps a copy of the checks.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::{Points, Vertex};
use crate::permissive::Loose;
use crate::report::Report;
use crate::stroke::Reach;

/// One vertex list to check, wherever it is written.
pub(crate) struct Host<'a> {
    /// The object the `points` key is written in: the element, or the effect member.
    pub owner: &'a Value,
    /// What a finding calls the list, after the element: `points`, `effects[1].points`.
    pub property: String,
    /// Whether the list closes, where that reads. `None` skips the vertex count and the
    /// dangling handles, which turn on it: an unreadable `closed` is the schema check's.
    pub closed: Option<bool>,
    /// The box every vertex and absolute handle must lie in, where it reads.
    pub bounds: Option<Bounds>,
    /// Fields every finding of this host carries beside its own, such as a mask's `index`.
    pub fields: Vec<(&'static str, Value)>,
}

/// The box a vertex list lies in, and the inset it keeps from the box's edges.
pub(crate) enum Bounds {
    /// A `path` element's declared box, inset by its stroke's reach (ADR-0154 §4, ADR-0158
    /// §4).
    Stroked {
        width: i64,
        height: i64,
        reach: Reach,
    },
    /// A box with no stroke to keep inside it, inset `0`: a mask's (ADR-0163 §6). `rect`
    /// says which rect the box is, in the words a finding prints.
    Bare {
        width: i64,
        height: i64,
        rect: &'static str,
    },
}

impl Bounds {
    fn inset(&self) -> i64 {
        match self {
            Bounds::Stroked { reach, .. } => reach.inset,
            Bounds::Bare { .. } => 0,
        }
    }

    fn sides(&self) -> (i64, i64) {
        match *self {
            Bounds::Stroked { width, height, .. } | Bounds::Bare { width, height, .. } => {
                (width, height)
            }
        }
    }

    /// What `E-PATH-OUTSIDE-BOX` says of where the inset came from.
    fn fields(&self, finding: Finding) -> Finding {
        match self {
            Bounds::Stroked { reach, .. } => finding
                .field("k", json!(reach.source.factor()))
                .field("width", json!(reach.width))
                .field("source", json!(reach.source.describe())),
            Bounds::Bare { rect, .. } => finding.field("rect", json!(rect)),
        }
    }
}

/// Every point finding for `host`: the vertex count and the dangling handles where its
/// closing reads, the keyframe shape, and the box where it reads (ADR-0154 §6).
pub(crate) fn points(host: &Host<'_>) -> Vec<Finding> {
    let literals = literals(host.owner);
    let mut out = Vec::new();
    if let Some(closed) = host.closed {
        for literal in &literals {
            out.extend(too_few(literal, closed));
            if !closed {
                out.extend(dangling(literal));
            }
        }
    }
    out.extend(keyframe_shape(&literals));
    if let Some(bounds) = &host.bounds {
        for literal in &literals {
            out.extend(outside(bounds, literal));
        }
    }
    out.into_iter()
        .map(|finding| {
            host.fields.iter().fold(
                finding.field("property", json!(host.property)),
                |finding, (name, value)| finding.field(*name, value.clone()),
            )
        })
        .collect()
}

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
        let side = |key| element.get(key).and_then(Value::as_i64);
        let host = Host {
            owner: element,
            property: "points".to_string(),
            closed: element.get("closed").and_then(Value::as_bool),
            bounds: match (side("width"), side("height")) {
                (Some(width), Some(height)) => Some(Bounds::Stroked {
                    width,
                    height,
                    reach: crate::stroke::reach(element),
                }),
                _ => None,
            },
            fields: Vec::new(),
        };
        points(&host).into_iter().for_each(&mut push);
        // A `path` element's own review, not a host's: a mask path always closes, so it
        // has no seam to cap.
        for literal in &literals(element) {
            seam_cap(element, literal).into_iter().for_each(&mut push);
        }
    }
}

/// Every literal value of `owner`'s `points` that reads as a vertex list.
fn literals(owner: &Value) -> Vec<Literal> {
    let read = |value: &Value| {
        serde_json::from_value::<Points>(value.clone())
            .ok()
            .map(|points| points.0)
    };
    match crate::animatable::records(owner, "points") {
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
        None => owner
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

fn outside(bounds: &Bounds, literal: &Literal) -> Vec<Finding> {
    let (width, height) = bounds.sides();
    let m = bounds.inset();
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
                bounds.fields(
                    Finding::new("E-PATH-OUTSIDE-BOX")
                        .field("vertex", json!(index))
                        .field("handle", json!(handle))
                        .field("position", json!(position))
                        .field("inset", json!(m))
                        .field("right", json!(width - m))
                        .field("bottom", json!(height - m)),
                ),
                literal,
            ));
        }
    }
    out
}

/// `R-PATH-SEAM-CAP` (ADR-0162 §4): an open, stroked, undashed path under a `butt` or
/// `square` cap whose first and last `at` coincide, where the direction leaving the first
/// vertex and the one arriving at the last differ. There the two ends meet as two caps, not
/// a join, and leave a notch or a spur. Silent where either end has no direction.
fn seam_cap(element: &Value, literal: &Literal) -> Option<Finding> {
    let open = element.get("closed").and_then(Value::as_bool) == Some(false);
    if !open || element.get("stroke").is_none() || element.get("stroke_dash").is_some() {
        return None;
    }
    let (cap, mark) = match element.get("stroke_cap").and_then(Value::as_str) {
        None | Some("butt") => ("butt", "notch"),
        Some("square") => ("square", "spur"),
        _ => return None,
    };
    let vertices = &literal.vertices;
    if vertices.first()?.at != vertices.last()?.at {
        return None;
    }
    let [lx, ly] = leaving(vertices)?;
    let [ax, ay] = arriving(vertices)?;
    if lx * ay - ly * ax == 0 && lx * ax + ly * ay > 0 {
        return None;
    }
    Some(located(
        Finding::new("R-PATH-SEAM-CAP")
            .field("cap", json!(cap))
            .field("mark", json!(mark)),
        literal,
    ))
}

/// Each segment of an open path, as its four absolute control points: one vertex's `at`,
/// `at + out`, the next vertex's `at + in`, and its `at`. A missing handle is a zero offset.
fn segments(vertices: &[Vertex]) -> impl DoubleEndedIterator<Item = [[i64; 2]; 4]> + '_ {
    let plus = |[x, y]: [i64; 2], offset: Option<[i64; 2]>| {
        let [dx, dy] = offset.unwrap_or([0, 0]);
        [x + dx, y + dy]
    };
    vertices.windows(2).map(move |pair| {
        let (a, b) = (&pair[0], &pair[1]);
        [a.at, plus(a.at, a.out), plus(b.at, b.arriving), b.at]
    })
}

fn minus([x, y]: [i64; 2], [u, v]: [i64; 2]) -> [i64; 2] {
    [x - u, y - v]
}

/// The direction the stroke leaves the first vertex in: the first non-zero of `P1 − P0`,
/// `P2 − P0` and `P3 − P0` on the first segment of non-zero length.
fn leaving(vertices: &[Vertex]) -> Option<[i64; 2]> {
    segments(vertices).find_map(|[p0, p1, p2, p3]| {
        [p1, p2, p3]
            .into_iter()
            .map(|p| minus(p, p0))
            .find(|&d| d != [0, 0])
    })
}

/// The direction the stroke arrives at the last vertex in: the first non-zero of `P3 − P2`,
/// `P3 − P1` and `P3 − P0` on the last segment of non-zero length.
fn arriving(vertices: &[Vertex]) -> Option<[i64; 2]> {
    segments(vertices).rev().find_map(|[p0, p1, p2, p3]| {
        [p2, p1, p0]
            .into_iter()
            .map(|p| minus(p3, p))
            .find(|&d| d != [0, 0])
    })
}
