//! `compare` — *"what changed between two versions of the timeline, including drift no
//! single document can see"* (#221).
//!
//! Four predicates, all from the ADR series, and none of them a keyframe resolver:
//!
//! - **Slack drift** (ADR-0032), over [`crate::slack::of`]: a boundary-to-boundary
//!   distance that changed size between the two documents.
//! - **Keyframe-instant relationship drift** (ADR-0063): two instants that were
//!   numerically *equal* in the reference document and no longer are in the current
//!   one. Exact equality only — ADR-0063 is explicit that a fixed-offset relationship
//!   "stays deferred", and this is the check ADR-0039 retracted from `validate` entirely
//!   on the grounds that desynchronization is a temporal predicate one document has no
//!   time axis to evaluate.
//! - **Boundary-coincidence-cluster drift** (ADR-0066): a sibling of the above, not an
//!   extension — this one needs no keyframe on either side, and reports a destroyed
//!   coincidence cluster as a moved-set versus a stayed-set, never pairwise (pairwise
//!   would be up to 55× the facts on the real fixture for no more information, per the
//!   ADR's own measurement).
//! - **Highlight text-drift** (ADR-0051): a run whose `text` changed while its
//!   `highlight` object stayed byte/value-identical — karaoke drift gets an owner.
//!
//! Every fact here carries **no severity**: `compare` describes what changed and judges
//! none of it, so its findings are [`crate::finding::Class::Drift`] rather than one of
//! ADR-0006's three severities, and `render` never consults them (see that class's own
//! doc comment for why a fifth report category was chosen over a fourth severity).
//!
//! # Reading the documents: permissive, not typed — and why
//!
//! `shift` (#220) reads its one document as `crate::model::Project`, because a `shift`
//! call already requires the document to validate clean (it runs the check engine
//! first) and wants typed, mutable access to rewrite it. `compare` has neither
//! property: it is handed two **independently arbitrary** documents — either one may be
//! mid-edit, carrying an unknown key or a schema violation the other does not — and a
//! strict parse of either would abort the whole comparison over a field this ticket has
//! nothing to do with. So every predicate here reads the permissive [`Loose`] tree, the
//! way [`crate::slack`] and the checks do: `element.get("id")`, `Value::as_i64`, the
//! same shape test [`crate::model::keyframe::Animatable`]'s own deserializer applies (an
//! array whose first item is an object is a keyframe list, everything else is a static
//! value). JSON is also the right equality for ADR-0063/ADR-0066's "exact equality" and
//! ADR-0051's "byte/value-identical": `serde_json::Value`'s `PartialEq` is exactly that,
//! with no tolerance and no interpolation to accidentally reach for.
//!
//! # Identity across two documents
//!
//! Nothing here diffs the two documents structurally. An element is the same element in
//! both if it carries the same `id` (CONTEXT.md: an id's "only job is to be a target"),
//! and a run has no id at all — ADR-0051/#221 settle that a run is correlated by its
//! **index within its element's own `runs` array**: an element whose run count differs
//! between the two documents has no run identity to diff against and is skipped for the
//! highlight check. Predicate 2 makes the same call for keyframes, which likewise carry
//! no id: a keyframe is named by `(element id, property name, index in that property's
//! own list)`, and an index missing on either side simply drops that candidate rather
//! than guessing a correspondence.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

use crate::finding::{Census, Finding};
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;
use crate::slack::{self, Edge, Side};

const TOOL: &str = "compare";

/// Every transform property that may carry keyframes (`crate::model`'s `Animatable`
/// fields, across every element type that has one).
const ANIMATABLE: &[&str] = &["x", "y", "scale", "rotation", "opacity", "volume"];

/// Compare two versions of a project, reporting drift as facts with no severity.
///
/// `current_path` is the report's `project` (the file an agent is looking at); the
/// reference path travels in every finding's own `ref_project` field, on this ticket's
/// own rule for a two-input verb (ADR-0006: every relevant fact travels inline, so a
/// finding never asks the reader to remember which call produced it).
///
/// Both files must independently parse and independently look like a project — this
/// runs no predicate over a file it cannot at least identify as one, the same
/// precondition `validate`'s `checked` holds for a single file (ADR-0042).
pub fn compare(ref_path: &Path, current_path: &Path) -> Report {
    let project = Some(current_path.display().to_string());
    let ref_project = ref_path.display().to_string();

    let ref_doc = match parse::read(ref_path) {
        Ok(document) => document,
        Err(finding) => return Report::unparseable(TOOL, project, *finding),
    };
    let current_doc = match parse::read(current_path) {
        Ok(document) => document,
        Err(finding) => return Report::unparseable(TOOL, project, *finding),
    };

    let mut report = Report::new(TOOL, project);

    for document in [&ref_doc, &current_doc] {
        if let Err(not_a_project) = document.shape() {
            report.push(Finding::not_a_project(
                document.path(),
                &not_a_project,
                &format!("point {TOOL} at two project files"),
            ));
        }
    }
    if !report.findings.is_empty() {
        return report;
    }

    let current_file = current_doc.path().to_string();

    slack_drift(
        &ref_doc,
        &current_doc,
        &ref_project,
        &current_file,
        &mut report,
    );
    keyframe_instant_drift(
        &ref_doc,
        &current_doc,
        &ref_project,
        &current_file,
        &mut report,
    );
    boundary_cluster_drift(
        &ref_doc,
        &current_doc,
        &ref_project,
        &current_file,
        &mut report,
    );
    highlight_text_drift(
        &ref_doc,
        &current_doc,
        &ref_project,
        &current_file,
        &mut report,
    );

    report
}

/// Which side of an element's range a boundary is, in words.
fn side_word(side: Side) -> &'static str {
    match side {
        Side::Start => "start",
        Side::End => "end",
    }
}

/// One element's `start`/`end` (by id and side), read directly from a document — the
/// permissive equivalent of `crate::slack`'s `Edge`, for looking a specific boundary up
/// in the *other* document rather than deriving every boundary in this one.
fn boundary_of(document: &Loose, id: &str, side: Side) -> Option<i64> {
    let key = match side {
        Side::Start => "start",
        Side::End => "end",
    };
    document.elements_in_tracks().find_map(|(_, element)| {
        if element.get("id").and_then(Value::as_str) != Some(id) {
            return None;
        }
        element.get(key).and_then(Value::as_i64)
    })
}

// ---- Predicate 1: slack drift (ADR-0032) ---------------------------------------------

/// A slack whose two ends are each exactly one element's boundary — the ordinary,
/// uncoincided case this predicate answers for. A slack whose `from` or `to` has *two or
/// more* edges is a boundary-coincidence cluster in the making, which predicate 3 owns
/// exclusively (see its own doc comment) — so this reads only the singly-bounded case
/// and leaves anything else alone rather than picking one of several coincident edges
/// arbitrarily.
fn single_edge<'a>(edges: &[Edge<'a>]) -> Option<(&'a str, Side)> {
    match edges {
        [edge] => Some((edge.element, edge.side)),
        _ => None,
    }
}

/// For every slack the reference document states between two singly-bounded elements,
/// resolve the same two boundaries directly in the current document (by `id` and side,
/// not by re-deriving the current document's own adjacency) and compare distances.
///
/// Resolving directly, rather than matching against `slack::of(current_doc)`'s own
/// output, is deliberate: this predicate is answering "did the distance between *these
/// two specific boundaries* change", and an edit that inserted a third element between
/// them would make `slack::of` stop treating the pair as adjacent even though the
/// distance between them is still a fact worth reporting.
fn slack_drift(
    ref_doc: &Loose,
    current_doc: &Loose,
    ref_project: &str,
    current_file: &str,
    report: &mut Report,
) {
    for slack in slack::of(ref_doc) {
        // The project's own `duration` has no element on the far side to look up in the
        // other document, and this predicate is about the distance between elements
        // (ADR-0032) rather than about `duration` itself.
        if slack.to_duration {
            continue;
        }
        let Some((from_id, from_side)) = single_edge(&slack.from_edges) else {
            continue;
        };
        let Some((to_id, to_side)) = single_edge(&slack.to_edges) else {
            continue;
        };
        let (Some(current_from), Some(current_to)) = (
            boundary_of(current_doc, from_id, from_side),
            boundary_of(current_doc, to_id, to_side),
        ) else {
            // One of the two boundaries no longer exists in the current document (the
            // element was removed or renamed) — a much larger structural change than
            // this predicate is written to describe, so it is left to other checks.
            continue;
        };
        let current_size = current_to - current_from;
        if current_size == slack.size() {
            continue;
        }
        if current_size == 0 {
            // ADR-0066's exception: the two boundaries are now coincident, which is
            // `D-BOUNDARY-CLUSTER-DRIFT`'s fact (a cluster forming) and never this
            // code's — reporting both would double-count one edit.
            continue;
        }
        if current_size < 0 {
            // The two boundaries inverted order: this is an overlap, not a slack —
            // a structurally different condition `E-TRACK-OVERLAP` names, and "is -100
            // ms" is not a distance this predicate's own wording can state honestly.
            continue;
        }
        report.push(
            Finding::new("D-SLACK-DRIFT")
                .at_file(current_file)
                .field("from", json!(slack.from))
                .field("to", json!(slack.to))
                .field("ref_size", json!(slack.size()))
                .field("current_size", json!(current_size))
                .field("from_edges", json!(describe_edges(&slack.from_edges)))
                .field("to_edges", json!(describe_edges(&slack.to_edges)))
                .field("ref_project", json!(ref_project)),
        );
    }
}

fn describe_edges(edges: &[Edge<'_>]) -> String {
    edges
        .iter()
        .map(|edge| format!("`{}`.{}", edge.element, side_word(edge.side)))
        .collect::<Vec<_>>()
        .join(", ")
}

// ---- Predicate 2: keyframe-instant relationship drift (ADR-0063) ---------------------

/// Every keyframe `t` in `document`, as `(element id, property name, index within that
/// property's own list, t)`.
///
/// The shape test mirrors `Animatable`'s own deserializer (`crate::model::keyframe`): an
/// array whose first item is an object is a keyframe list; everything else — including
/// `scale`'s own `[sx, sy]` — is a static value and contributes nothing here.
fn keyframes_of(document: &Loose) -> Vec<(String, &'static str, usize, i64)> {
    let mut out = Vec::new();
    for (_, element) in document.elements_in_tracks() {
        let Some(id) = element.get("id").and_then(Value::as_str) else {
            continue;
        };
        for &property in ANIMATABLE {
            let Some(records) = element.get(property).and_then(Value::as_array) else {
                continue;
            };
            if !records.first().is_some_and(Value::is_object) {
                continue;
            }
            for (index, record) in records.iter().enumerate() {
                if let Some(t) = record.get("t").and_then(Value::as_i64) {
                    out.push((id.to_string(), property, index, t));
                }
            }
        }
    }
    out
}

/// Every element's own `(start, end)`, for the boundary half of candidate populations
/// (a) and (b).
fn boundaries_of(document: &Loose) -> Vec<(String, i64, i64)> {
    document
        .elements_in_tracks()
        .filter_map(|(_, element)| {
            let id = element.get("id").and_then(Value::as_str)?;
            let start = element.get("start").and_then(Value::as_i64)?;
            let end = element.get("end").and_then(Value::as_i64)?;
            Some((id.to_string(), start, end))
        })
        .collect()
}

/// The `t` a specific `(element id, property, index)` keyframe carries in `document`, if
/// that record still exists there at all.
fn keyframe_at(document: &Loose, id: &str, property: &str, index: usize) -> Option<i64> {
    document.elements_in_tracks().find_map(|(_, element)| {
        if element.get("id").and_then(Value::as_str) != Some(id) {
            return None;
        }
        element
            .get(property)
            .and_then(Value::as_array)?
            .get(index)?
            .get("t")
            .and_then(Value::as_i64)
    })
}

/// One rule, over three candidate populations (ADR-0063): a keyframe against its own
/// element's boundary (`kind` `a`), a keyframe against a *different* element's boundary
/// (`kind` `b`), or two different elements' same-property keyframe times, keyed
/// per-property and never per-`group` (`kind` `c` — the ADR-0039 lesson: a shared
/// `group` is not evidence of a temporal relationship, only a shared authorial unit).
///
/// Case (c) has, per the ADR, zero real-fixture coverage today — it is still
/// implemented, and covered here by a synthetic fixture, rather than left unimplemented
/// on the strength of that absence.
fn keyframe_instant_drift(
    ref_doc: &Loose,
    current_doc: &Loose,
    ref_project: &str,
    current_file: &str,
    report: &mut Report,
) {
    let keyframes = keyframes_of(ref_doc);
    let boundaries = boundaries_of(ref_doc);

    // (a) and (b): a keyframe against a boundary, own or another element's.
    for (element_id, property, index, ref_t) in &keyframes {
        for (boundary_id, start, end) in &boundaries {
            for (side, boundary_t) in [(Side::Start, *start), (Side::End, *end)] {
                if *ref_t != boundary_t {
                    continue;
                }
                let kind = if element_id == boundary_id { "a" } else { "b" };
                let left = format!("{element_id}'s {property} keyframe");
                let right = format!("{boundary_id}'s own {}", side_word(side));
                let (Some(current_left), Some(current_right)) = (
                    keyframe_at(current_doc, element_id, property, *index),
                    boundary_of(current_doc, boundary_id, side),
                ) else {
                    continue;
                };
                if current_left != current_right {
                    report.push(instant_drift_finding(
                        kind,
                        &left,
                        &right,
                        element_id,
                        boundary_id,
                        *ref_t,
                        current_left,
                        current_right,
                        current_file,
                        ref_project,
                    ));
                }
            }
        }
    }

    // (c): two different elements' same-property keyframe times.
    for i in 0..keyframes.len() {
        let (id1, prop1, index1, t1) = &keyframes[i];
        for (id2, prop2, index2, t2) in &keyframes[i + 1..] {
            if id1 == id2 || prop1 != prop2 || t1 != t2 {
                continue;
            }
            let left = format!("{id1}'s {prop1} keyframe");
            let right = format!("{id2}'s {prop2} keyframe");
            let (Some(current_left), Some(current_right)) = (
                keyframe_at(current_doc, id1, prop1, *index1),
                keyframe_at(current_doc, id2, prop2, *index2),
            ) else {
                continue;
            };
            if current_left != current_right {
                report.push(instant_drift_finding(
                    "c",
                    &left,
                    &right,
                    id1,
                    id2,
                    *t1,
                    current_left,
                    current_right,
                    current_file,
                    ref_project,
                ));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn instant_drift_finding(
    kind: &str,
    left: &str,
    right: &str,
    left_element: &str,
    right_element: &str,
    ref_at: i64,
    current_left: i64,
    current_right: i64,
    current_file: &str,
    ref_project: &str,
) -> Finding {
    Finding::new("D-KEYFRAME-INSTANT-DRIFT")
        .at_file(current_file)
        .field("kind", json!(kind))
        .field("left", json!(left))
        .field("right", json!(right))
        .field("left_element", json!(left_element))
        .field("right_element", json!(right_element))
        .field("ref_at", json!(ref_at))
        .field("current_left", json!(current_left))
        .field("current_right", json!(current_right))
        .field("ref_project", json!(ref_project))
}

// ---- Predicate 3: boundary-coincidence-cluster drift (ADR-0066) ----------------------

/// Every instant in `document` where two or more element boundaries coincide, with the
/// `(element id, side)` at each — scoped to all N≥2 clusters, never just N≥3 (ADR-0066).
///
/// A standalone traversal rather than a read of `crate::slack::of`'s output: that
/// function only ever exposes *adjacent* pairs, one per gap between distinct instants,
/// which is exactly the information a full coincidence cluster is not. The walk itself —
/// `elements_in_tracks`, requiring both `start` and `end` as `i64`, `Loose`'s permissive
/// reads — is `crate::slack::of`'s own, kept for the same reason it is there: a document
/// mid-edit is exactly what this runs on, and a cluster measured to a boundary that
/// is not fully stated would manufacture a coincidence out of a half-written element.
fn boundary_map(document: &Loose) -> BTreeMap<i64, Vec<(String, Side)>> {
    let mut map: BTreeMap<i64, Vec<(String, Side)>> = BTreeMap::new();
    for (_, element) in document.elements_in_tracks() {
        let Some(id) = element.get("id").and_then(Value::as_str) else {
            continue;
        };
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        map.entry(start)
            .or_default()
            .push((id.to_string(), Side::Start));
        map.entry(end)
            .or_default()
            .push((id.to_string(), Side::End));
    }
    map
}

/// A sibling predicate to keyframe-instant drift, not an extension of it: this one needs
/// no keyframe on either side, and answers a purely boundary-only question.
///
/// Run in **both directions** — every ref-side cluster checked against the current
/// document, and every current-side cluster checked against the reference one — because
/// the two directions catch different edits. A cluster that held in the reference
/// document and split apart in the current one is what ADR-0066 states directly. But
/// ADR-0032's own suppression (predicate 1's zero-distance exception) needs the mirror:
/// two boundaries that were at a *nonzero* distance in the reference document and closed
/// to an exact coincidence in the current one never appear as a cluster on the reference
/// side at all (they were not coincident there), so only the current-side pass finds
/// them. Both passes apply the identical "does every member agree on the other side too"
/// test, so a cluster that simply moved together, unchanged relative to itself, produces
/// no fact from either direction — and a genuine split or a genuine formation is
/// reported exactly once, from the side that actually holds the cluster.
fn boundary_cluster_drift(
    ref_doc: &Loose,
    current_doc: &Loose,
    ref_project: &str,
    current_file: &str,
    report: &mut Report,
) {
    cluster_direction(
        ref_doc,
        current_doc,
        "reference",
        "current",
        ref_project,
        current_file,
        report,
    );
    cluster_direction(
        current_doc,
        ref_doc,
        "current",
        "reference",
        ref_project,
        current_file,
        report,
    );
}

#[allow(clippy::too_many_arguments)]
fn cluster_direction(
    anchor_doc: &Loose,
    other_doc: &Loose,
    which: &str,
    other: &str,
    ref_project: &str,
    current_file: &str,
    report: &mut Report,
) {
    for (at, members) in boundary_map(anchor_doc)
        .iter()
        .filter(|(_, m)| m.len() >= 2)
    {
        let mut groups: BTreeMap<Option<i64>, Vec<String>> = BTreeMap::new();
        for (id, side) in members {
            let other_at = boundary_of(other_doc, id, *side);
            groups.entry(other_at).or_default().push(id.clone());
        }
        if groups.len() <= 1 {
            // Every member agrees with every other on the far side too, whether that is
            // this same instant (nothing moved) or one shared new instant (the cluster
            // moved together) — either way it is intact, not destroyed.
            continue;
        }

        let stayed = groups.get(&Some(*at)).cloned().unwrap_or_default();
        let moved: Vec<String> = groups
            .iter()
            .filter(|(key, _)| **key != Some(*at))
            .flat_map(|(key, ids)| {
                let phrase = match key {
                    Some(t) => format!("sits at {t} in the {other} version"),
                    None => "is no longer present in that version".to_string(),
                };
                ids.iter().map(move |id| format!("{id} {phrase}"))
            })
            .collect();

        let mut census = Census::on("boundary_instant");
        for (key, ids) in &groups {
            let value = match key {
                Some(t) => json!(t),
                None => json!("removed"),
            };
            census = census.group(value, ids.clone());
        }

        let all_members: Vec<String> = members.iter().map(|(id, _)| id.clone()).collect();
        report.push(
            Finding::new("D-BOUNDARY-CLUSTER-DRIFT")
                .at_file(current_file)
                .field("at", json!(at))
                .field("which", json!(which))
                .field("other", json!(other))
                .field("members", json!(all_members.join(", ")))
                .field(
                    "stayed",
                    json!(if stayed.is_empty() {
                        "none".to_string()
                    } else {
                        format!("{} still coincide", stayed.join(", "))
                    }),
                )
                .field(
                    "moved",
                    json!(if moved.is_empty() {
                        "none".to_string()
                    } else {
                        moved.join("; ")
                    }),
                )
                .field("ref_project", json!(ref_project))
                .census(census),
        );
    }
}

// ---- Predicate 4: highlight text-drift (ADR-0051) ------------------------------------

/// A run whose `text` differs between the two documents while its `highlight` object is
/// byte/value-identical — karaoke drift gets an owner.
///
/// **Requires a `highlight` present and equal on both sides**, not merely equal
/// (including both absent): the predicate is about a run whose *karaoke window* survived
/// an edit to its words, and a plain, unhighlighted run whose text changed is ordinary
/// authorship this check has nothing to say about. Runs are correlated by index within
/// an element matched by `id`; an element whose run count differs between the two
/// documents has no run identity to diff against and is skipped entirely for this check
/// (see the module doc comment) — a fuzzy match was rejected rather than guessed at.
fn highlight_text_drift(
    ref_doc: &Loose,
    current_doc: &Loose,
    ref_project: &str,
    current_file: &str,
    report: &mut Report,
) {
    for (_, current_element) in current_doc.elements_in_tracks() {
        let Some(id) = current_element.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some(current_runs) = current_element.get("runs").and_then(Value::as_array) else {
            continue;
        };
        let Some(ref_element) = ref_doc
            .elements_in_tracks()
            .find(|(_, element)| element.get("id").and_then(Value::as_str) == Some(id))
            .map(|(_, element)| element)
        else {
            continue;
        };
        let Some(ref_runs) = ref_element.get("runs").and_then(Value::as_array) else {
            continue;
        };
        if ref_runs.len() != current_runs.len() {
            continue;
        }
        for (index, (ref_run, current_run)) in ref_runs.iter().zip(current_runs).enumerate() {
            let (Some(ref_text), Some(current_text)) = (
                ref_run.get("text").and_then(Value::as_str),
                current_run.get("text").and_then(Value::as_str),
            ) else {
                continue;
            };
            if ref_text == current_text {
                continue;
            }
            let highlight_unchanged = matches!(
                (ref_run.get("highlight"), current_run.get("highlight")),
                (Some(a), Some(b)) if !a.is_null() && a == b
            );
            if !highlight_unchanged {
                continue;
            }
            report.push(
                Finding::new("D-HIGHLIGHT-TEXT-DRIFT")
                    .at_file(current_file)
                    .at_element(id)
                    .field("run_index", json!(index))
                    .field("ref_text", json!(ref_text))
                    .field("current_text", json!(current_text))
                    .field("ref_project", json!(ref_project)),
            );
        }
    }
}
