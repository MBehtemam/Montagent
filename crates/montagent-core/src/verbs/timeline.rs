//! `timeline` — *"the human's wide view"* (ADR-0011).
//!
//! CLI-only, by the same cost model as `probe` and `fmt`: *"every MCP tool schema occupies
//! the agent's context and degrades tool selection on every turn... A CLI subcommand costs
//! nothing until invoked."* The agent reaches the same facts through `query`'s census and
//! aggregation modes, which it already has.
//!
//! ## Why this view has no time axis
//!
//! ADR-0031 removed the requirement for one and left the choice here, in both directions:
//! *"`timeline` remains free to render spatially if and when it is built, since a human
//! reader is a different consumer with different evidence requirements."* Nothing in this
//! repository measures a human reader, so nothing is owed either way — and that, rather
//! than anything ADR-0031 measured, is the whole of the argument. Its cost list and its
//! 679 ms-per-character resolution finding are *agent*-scoped and the ADR fences them off
//! from this verb by name; quoting them here would be borrowing evidence about one reader
//! to decide about another.
//!
//! What this view is built on instead is the one thing that repository *does* rank:
//! candidate C of `output-100col.txt`, first of four by readers doing real tasks against
//! the fixture, on the claim *"`group` is the beat the author thinks in. 60 elements
//! collapse to 8 blocks."* Round 1's verdict was voided for contamination and its ranking
//! was not — `FINDINGS.md` says so in as many words.
//!
//! Two departures from that prototype, both required by an ADR rather than preferred:
//!
//! - **Every instant printed is absolute.** The prototype printed each element's offset
//!   from its group's own start. ADR-0004 requires that absolute times be unmissable, and
//!   ADR-0031 reads that as *"stated explicitly, not left to be inferred from position or
//!   array order"* — an offset is exactly that inference.
//! - **The word `segment` does not appear.** `CONTEXT.md` lists it among the words to avoid
//!   for **Group**, next to `scene` and `section`.
//!
//! ## What it is not
//!
//! It judges nothing **about the video** and probes nothing. Every number it prints is
//! either written in the document or is `end - start` over numbers that are. Whether a gap
//! is a defect is `validate`'s question (ADR-0006), and what is true at an instant with
//! every animated value resolved is `query`'s — a view that answered either would be a
//! second place for those rules to live.
//!
//! The one thing it does refuse is a file that is not a project at all, on ADR-0042's
//! shared structural predicate. That is a judgement about the *file*, not the document: a
//! document with no `tracks` has no shape to show, and inventing a second predicate for it
//! would give `validate`, `fmt` and this verb three answers to one question. Whether a view
//! should instead print the header it *can* read and an empty body is a real alternative,
//! and refusing is the narrower of the two.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::finding::Finding;
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::Stack;

const TOOL: &str = "timeline";

/// How much of a run's or a source's text one row shows before it is cut short.
///
/// A cap rather than the whole string: one 400-character caption would widen nothing (the
/// column is last) but would wrap in the terminal and put one element across three rows,
/// which is the same misalignment the line-break mark below prevents.
const DETAIL_WIDTH: usize = 56;

/// A line break, as a row can carry it. ADR-0008 makes `\n` inside a run's text the only
/// spelling of a break, so a view that printed it raw would split one element across two
/// rows for a reason the reader cannot see.
const BREAK_MARK: char = '⏎';

/// The mark on an element that moves — the prototype's `~anim`, kept verbatim.
const MOTION_MARK: &str = "~anim";

/// One `timeline` invocation's answer: the view, and the report every verb answers with.
///
/// Shaped like `probe`'s — a `Report` plus the facts only this verb establishes — so there
/// is one wire shape across the surface rather than a verb-shaped variant.
pub struct Answer {
    overview: Option<Overview>,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The canonical JSON: the report's own object, plus the view under `timeline`.
    ///
    /// The key is present and `null` where no view could be built, rather than absent. A
    /// consumer reading `timeline` off a run that failed to parse gets "there is no view"
    /// either way; an absent key additionally makes it indistinguishable from a version of
    /// Montagent that did not have this verb.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "timeline",
            match &self.overview {
                Some(overview) => serde_json::to_value(overview).unwrap_or(Value::Null),
                None => Value::Null,
            },
        )
    }
}

/// The whole view, as one object. Field order here is the order the prose prints in.
#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    /// The project header, each field exactly as the document writes it — or `null` where
    /// the document does not write it at all. Held as [`Value`] rather than typed, because
    /// this view is wanted on a half-written document and an `fps` of `"25"` is a fact the
    /// reader needs to see rather than a parse failure (ADR-0042's spirit, and the reason
    /// every check in this crate reads the permissive tree).
    pub frame: Value,
    pub fps: Value,
    pub duration_ms: Value,
    pub background: Value,
    pub output: Value,
    #[serde(rename = "loop")]
    pub looping: Value,
    pub counts: Counts,
    pub tracks: Vec<TrackRow>,
    pub groups: Vec<GroupBlock>,
}

/// The one line that answers *"how big is this project"*.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Counts {
    pub elements: usize,
    pub tracks: usize,
    /// **Named** groups. The block holding the elements that carry no `group` is not one of
    /// them: `group` is optional (`CONTEXT.md`), and counting its absence as a group would
    /// report a structure the author did not write.
    pub groups: usize,
}

/// One track: its name, the place in the stack it supplies, and how much it holds.
#[derive(Debug, Clone, Serialize)]
pub struct TrackRow {
    pub name: Value,
    pub layer: Value,
    pub elements: usize,
}

/// One group's elements, and the stretch of the clock they span between them.
#[derive(Debug, Clone, Serialize)]
pub struct GroupBlock {
    /// `null` for the block holding every element that carries no `group`.
    pub group: Option<String>,
    /// The earliest `start` and the latest `end` among the elements below, or `null` where
    /// none of them states a range. Derived, never stored: `CONTEXT.md` avoids `duration` as
    /// a stored field, and ADR-0005 keeps it out of the document for the same reason.
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub duration_ms: Option<i64>,
    pub elements: Vec<ElementRow>,
}

/// One element, as a row.
#[derive(Debug, Clone, Serialize)]
pub struct ElementRow {
    pub id: Option<String>,
    /// The label that decides which block this row lands in.
    ///
    /// Not serialised: it is the key of the block the row already sits inside, and a
    /// consumer that read it off the row could disagree with the block it was filed under.
    #[serde(skip)]
    pub group: Option<String>,
    pub track: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Where this element sits in the stack, **resolved** — an anchor arrives here as the
    /// integer ADR-0019's one hop makes it, because handing the reader `{"below": "card"}`
    /// would leave them the arithmetic the view exists to have already done. `null` where
    /// the document does not determine one; naming *which* way it fails is
    /// [`crate::checks::anchor`]'s job and not a view's.
    pub layer: Option<i64>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub duration_ms: Option<i64>,
    /// What this element is, in the fewest words that distinguish it from its siblings: a
    /// source path, a text element's own words, a shape's rect and paint. Without it the
    /// view is a list of ids, and `photo-06` says nothing about what is on screen.
    pub detail: String,
}

/// Build the wide view of the project file at `path`.
///
/// Reads the file once and touches nothing else — no probe, no session, no sidecar.
pub fn timeline(path: &Path) -> Answer {
    let project = Some(path.display().to_string());

    let document = match parse::read(path) {
        Ok(document) => document,
        // ADR-0011: nothing may partially process a malformed file. A view assembled from
        // half a parse is a picture of a document that does not exist.
        Err(finding) => {
            return Answer {
                overview: None,
                report: Report::unparseable(TOOL, project, *finding),
            };
        }
    };

    let mut report = Report::new(TOOL, project);

    if let Err(not_a_project) = document.shape() {
        // ADR-0042's shared structural predicate, reused rather than reinvented: with no
        // `tracks` there is nothing to lay out, and the ADR's own finding was that such a
        // file deserves a sentence naming the likely mismatch rather than a raw schema
        // error.
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point timeline at the project file",
        ));
        return Answer {
            overview: None,
            report,
        };
    }

    Answer {
        overview: Some(overview_of(&document)),
        report,
    }
}

fn overview_of(document: &Loose) -> Overview {
    let root = document.value();
    let stack = Stack::of(document);

    let mut tracks = Vec::new();
    let mut rows = Vec::new();
    for track in root
        .get("tracks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = track.get("name").and_then(Value::as_str);
        let elements = track
            .get("elements")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        tracks.push(TrackRow {
            name: stated(track.get("name")),
            layer: stated(track.get("layer")),
            elements: elements.len(),
        });
        for element in elements {
            rows.push(row_of(element, name, track.get("layer"), &stack));
        }
    }

    // By the stack, back to front — the order a reader asking "what is in front of what"
    // needs, and the one thing a track is for (ADR-0004). Ties keep document order, which
    // is a traversal and never a ranking (ADR-0060).
    tracks.sort_by_key(|track| (track.layer.as_i64().is_none(), track.layer.as_i64()));

    let counts = Counts {
        elements: rows.len(),
        tracks: tracks.len(),
        groups: named_groups(&rows),
    };

    Overview {
        frame: stated(root.get("frame")),
        fps: stated(root.get("fps")),
        duration_ms: stated(root.get("duration")),
        background: stated(root.get("background")),
        output: stated(root.get("output")),
        looping: stated(root.get("loop")),
        counts,
        tracks,
        groups: blocks_of(rows),
    }
}

/// How many distinct `group` labels the document writes.
///
/// Counted from the elements rather than from the blocks below, so that the number stays
/// what it says it is — the count of labels the *author* wrote — whatever the view does
/// with the elements that carry none.
fn named_groups(rows: &[ElementRow]) -> usize {
    rows.iter()
        .filter_map(|row| row.group.as_deref())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// Gather the rows into one block per group, then the residue.
fn blocks_of(rows: Vec<ElementRow>) -> Vec<GroupBlock> {
    // Insertion-ordered by first appearance, which is the tie-break the sort below falls
    // through to. `BTreeMap` would order by label, making `item-10` precede `item-5`.
    let mut order: Vec<Option<String>> = Vec::new();
    let mut by_group: BTreeMap<usize, Vec<ElementRow>> = BTreeMap::new();
    for row in rows {
        let group = row.group.clone();
        let index = match order.iter().position(|seen| *seen == group) {
            Some(index) => index,
            None => {
                order.push(group);
                order.len() - 1
            }
        };
        by_group.entry(index).or_default().push(row);
    }

    let mut blocks: Vec<GroupBlock> = by_group
        .into_iter()
        .map(|(index, mut elements)| {
            // Chronological, then by the instant it lets go, then document order. An
            // element with no range sorts last: there is no instant at which to place it.
            elements.sort_by_key(|row| (row.start.is_none(), row.start, row.end));
            let start = elements.iter().filter_map(|row| row.start).min();
            let end = elements.iter().filter_map(|row| row.end).max();
            GroupBlock {
                group: order[index].clone(),
                start,
                end,
                duration_ms: length_of(start, end),
                elements,
            }
        })
        .collect();

    // The ungrouped block goes last, whatever it spans: it is a residue rather than a beat,
    // and sorting it in among the groups would give the reader a heading that means
    // "everything else" in a position that means "next".
    blocks.sort_by(|a, b| {
        let key = |block: &GroupBlock| {
            (
                block.group.is_none(),
                block.start.is_none(),
                block.start,
                block.end,
            )
        };
        key(a).cmp(&key(b))
    });
    blocks
}

fn row_of(
    element: &Value,
    track: Option<&str>,
    track_layer: Option<&Value>,
    stack: &Stack<'_>,
) -> ElementRow {
    let id = element.get("id").and_then(Value::as_str);
    let start = element.get("start").and_then(Value::as_i64);
    let end = element.get("end").and_then(Value::as_i64);

    ElementRow {
        id: id.map(str::to_string),
        group: element
            .get("group")
            .and_then(Value::as_str)
            .map(str::to_string),
        track: track.map(str::to_string),
        kind: element
            .get("type")
            .and_then(Value::as_str)
            .map(str::to_string),
        // One hop through the one resolver (ADR-0019). An element the stack cannot name —
        // it carries no `id` — still sits in its track, so the track's own layer is the
        // honest answer rather than nothing.
        layer: id
            .and_then(|id| stack.layer_of(id).ok())
            .or_else(|| track_layer.and_then(Value::as_i64)),
        start,
        end,
        duration_ms: length_of(start, end),
        detail: detail_of(element),
    }
}

/// What this element is, in one column.
///
/// Ordered by what distinguishes an element from its siblings rather than by the format's
/// key order: on a track of photos the source is the whole difference, and on a track of
/// captions the words are.
fn detail_of(element: &Value) -> String {
    if let Some(source) = element.get("source").and_then(Value::as_str) {
        return with_motion(element, elide(source));
    }
    if let Some(runs) = element.get("runs").and_then(Value::as_array) {
        let text: String = runs
            .iter()
            .filter_map(|run| run.get("text").and_then(Value::as_str))
            .collect();
        return with_motion(element, elide(&text.replace('\n', &BREAK_MARK.to_string())));
    }
    if let (Some(from), Some(to)) = (
        element.get("from").and_then(Value::as_str),
        element.get("to").and_then(Value::as_str),
    ) {
        return elide(&format!("{from} → {to}"));
    }
    let paint = element
        .get("fill")
        .or_else(|| element.get("stroke"))
        .and_then(Value::as_str);
    match (
        element.get("width").and_then(Value::as_i64),
        element.get("height").and_then(Value::as_i64),
        paint,
    ) {
        (Some(width), Some(height), Some(paint)) => {
            with_motion(element, format!("{width}×{height} {paint}"))
        }
        (Some(width), Some(height), None) => with_motion(element, format!("{width}×{height}")),
        _ => String::new(),
    }
}

/// Mark an element that moves.
///
/// The prototype this view is built on carried it (`05.png ~anim`), and without it a Ken
/// Burns push and a still photograph are the same row. It is a *presence* claim and nothing
/// more — what the motion is, and what it resolves to at an instant, is `query`'s question.
fn with_motion(element: &Value, detail: String) -> String {
    if !animated(element) {
        return detail;
    }
    match detail.is_empty() {
        true => MOTION_MARK.to_string(),
        false => format!("{detail} {MOTION_MARK}"),
    }
}

/// Does any **animatable** property of this element carry a keyframe list rather than one
/// value?
///
/// Restricted to the closed set ADR-0012 names, and the restriction is load-bearing rather
/// than tidy: the shape test alone — an array whose first item is an object — is also true
/// of `runs`, and a bare walk over every key marks all 22 of the fixture's text elements as
/// moving when none of them does. The set is the animatable properties themselves
/// (ADR-0012), plus `volume`, which ADR-0055 gives *"the same scalar-or-keyframe-array
/// polymorphism every other animatable property already has."*
///
/// Read off the permissive tree rather than the typed model, because this view is wanted on
/// documents the types cannot hold.
fn animated(element: &Value) -> bool {
    const ANIMATABLE: [&str; 7] = [
        "x",
        "y",
        "letter_spacing",
        "scale",
        "rotation",
        "opacity",
        "volume",
    ];

    ANIMATABLE.iter().any(|property| {
        // ADR-0012's own shape test, as `Animatable` applies it on the way in: a keyframe
        // record is an object, so an array **of objects** is a keyframe list and every other
        // array — `scale`'s own `[sx, sy]` — is a static value.
        element
            .get(property)
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .is_some_and(Value::is_object)
    })
}

/// `end - start`, where the document states both and the subtraction is representable.
///
/// `checked_sub` rather than bare arithmetic: a hand-written `start` of `i64::MIN` makes
/// the difference overflow, and a *view* that aborted on a number it was only reporting
/// would be the least useful moment for Montagent to stop. A length that cannot be
/// represented is not a length, and the view already has a spelling for that.
fn length_of(start: Option<i64>, end: Option<i64>) -> Option<i64> {
    end?.checked_sub(start?)
}

/// `DETAIL_WIDTH` characters at most, with an ellipsis where something was cut.
///
/// Not `clip`: that word is a live field of the format — the static frame-space aperture of
/// ADR-0025 — and `CONTEXT.md` additionally lists it among the rejected terms. A helper
/// called `clip` beside a document that has `clip` reads as clipping media.
fn elide(text: &str) -> String {
    if text.chars().count() <= DETAIL_WIDTH {
        return text.to_string();
    }
    text.chars().take(DETAIL_WIDTH - 1).chain(['…']).collect()
}

/// A key as the document writes it, or `null` where it does not write it at all.
fn stated(value: Option<&Value>) -> Value {
    value.cloned().unwrap_or(Value::Null)
}
