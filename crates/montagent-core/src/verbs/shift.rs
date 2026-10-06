//! `shift` — *"the one edit that is arithmetic rather than authorship"* (#220).
//!
//! `shift(path, at, delta, scope, release)` moves every time at or after `at` by `delta`,
//! project-scoped by default (ADR-0011). Four things keep it from being a blind `sed` over
//! every number in the file:
//!
//! - **A time-based straddler is refused**, never stretched or relocated (ADR-0005):
//!   stretching speech desyncs its source, and moving it whole relocates audio that has
//!   already begun.
//! - **Keyframes are carried by their element**, not dragged by the raw `at`-or-after rule
//!   (ADR-0012). A time-invariant straddler's keyframe lists go through SPLIT, which
//!   inserts a `delta`-long hold at the cut so the picture is identical outside
//!   `[at, at+delta)`. Every list is one the schema types as animatable
//!   ([`crate::animatable`], ADR-0146), so no property is left behind; a split whose value
//!   is no legal literal for its field is refused (`E-SHIFT-SPLIT-UNWRITABLE`).
//! - **Every slack in the file is invariant by default** (ADR-0032). An edit that would
//!   change one is refused, listing every threatened pair; `release` consumes exactly the
//!   pairs a refusal reported (ADR-0047), named individually — there is no bulk form.
//! - **A coincident-instant preamble prints unconditionally** (ADR-0036): before executing
//!   a shift whose `at` lands on an existing element boundary or keyframe `t`, this verb
//!   says what its own rules do to every record sitting there.
//!
//! # What this ticket does not build
//!
//! **`delta` must be positive.** ADR-0012 also specifies a negative, destructive `delta` —
//! dropping every keyframe in the removed span and naming each one — and that mechanism
//! does not reduce to SPLIT run backwards: the two new records SPLIT writes can invert
//! their own order, and a large enough removal can reach past its own segment's earlier
//! endpoint. None of #220's acceptance criteria exercise it, so a negative `delta` is
//! refused here rather than shipped half-reasoned; it is a follow-up ticket's, not a
//! silent gap in this one.

use std::collections::{HashMap, HashSet};
use std::path::Path as FilePath;

use serde::Serialize;
use serde_json::{Value, json};

use crate::animatable::{self, Kind, Property};
use crate::finding::Finding;
use crate::media::sidecar::Sidecar;
use crate::model::{
    Body, Colour, Ease, EaseName, Element, Keyframe, Points, Project, Scale, Vertex,
};
use crate::report::{ExitCode, Report};
use crate::resolve::{self, Blend, Interpolate, VertexAt};
use crate::slack::{self, Edge, Side, Slack};
use crate::write;

const TOOL: &str = "shift";

/// What one `shift` invocation is asking.
#[derive(Debug, Clone, Default)]
pub struct Ask {
    /// The instant to shift at or after, in absolute milliseconds.
    pub at: i64,
    /// The offset, in milliseconds. Must be positive — see the module note.
    pub delta: i64,
    /// A single track's name, narrowing the edit to it. `None` is the whole project
    /// (ADR-0005/ADR-0011).
    pub scope: Option<String>,
    /// Every slack this edit would otherwise change, named as its full boundary-instant
    /// pair (ADR-0047). No bulk form: each pair is checked against this call's own
    /// threatened slacks, and a pair that does not match one is itself a refusal.
    pub release: Vec<(i64, i64)>,
}

/// One record sitting exactly at the shift's `at`, and what `shift`'s own rules do to it
/// (ADR-0036). Printed unconditionally whenever `at` coincides with an existing instant —
/// never gated behind a flag, because the hazard is that the dangerous call and the
/// routine call are indistinguishable from the author's side.
/// `Clone`: the transform pass may refuse *after* the preamble is computed, and the
/// preamble still belongs on that refusal's answer — ADR-0036 calls it unconditional.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CoincidentRecord {
    pub element: String,
    /// `"boundary"` for an element's own `start`/`end`, `"keyframe"` for a transform
    /// property's record.
    pub kind: &'static str,
    /// `"start"`/`"end"` for a boundary; `"first"`/`"last"`/`"interior"` for a keyframe,
    /// naming its position in its own property's list.
    pub role: &'static str,
    /// The animated property this record belongs to, for a keyframe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    /// What this shift's own published rules do to the record.
    pub effect: String,
}

/// One `shift` invocation's answer: the coincident preamble, and the report every write
/// tool answers with — the new state's findings, never `ok` (ADR-0011).
pub struct Answer {
    preamble: Vec<CoincidentRecord>,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The canonical JSON: the report's own object, plus the preamble under `shift`.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "shift",
            json!({
                "coincident": self.preamble,
            }),
        )
    }
}

/// Shift the project at `path`.
pub fn shift(path: &FilePath, ask: &Ask) -> Answer {
    let project_name = Some(path.display().to_string());
    let refused = |report| Answer {
        preamble: Vec::new(),
        report,
    };

    if ask.delta <= 0 {
        return refused(Report::rejected(
            TOOL,
            project_name,
            "`delta` must be a positive number of milliseconds; a non-positive `delta` is \
             refused rather than shipped half-reasoned (see the module note on negative \
             `delta`)",
        ));
    }

    // The identical check engine `render` gates on (ADR-0006/ADR-0011): a document that
    // does not already validate clean has no well-defined "nearest legal boundary" or
    // "current slack size" for this verb's own arithmetic to trust.
    let (document, mut report) =
        match crate::verbs::validate::checked(TOOL, path, None, Sidecar::default_path()) {
            Ok(checked) => checked,
            Err(report) => return refused(*report),
        };
    if report.exit_code() != ExitCode::Ok {
        report.tool = TOOL.to_string();
        return refused(report);
    }

    let mut project: Project = match document.strict() {
        Ok(project) => project,
        Err(e) => {
            report.fail_internally(format!(
                "the project passed every check and still does not fit the model: {e}"
            ));
            return refused(report);
        }
    };

    if let Some(scope) = &ask.scope
        && !project.tracks.iter().any(|track| &track.name == scope)
    {
        return refused(Report::rejected(
            TOOL,
            project_name,
            format!("`scope` names `{scope}`, which is not a track in this project"),
        ));
    }

    let preamble = coincident_preamble(&project, ask.at);

    // ---- A cut inside a stagger window is refused (ADR-0151 §5), naming the window: a
    // split of the unit lists at `at` would be right for one unit only, and writing a split
    // for every unit would turn one block into one run per unit. Only an element the shift
    // would split — one in scope that straddles `at` — can be cut. --------------------------
    let mut cut: Vec<Finding> = Vec::new();
    for track in &project.tracks {
        if ask
            .scope
            .as_deref()
            .is_some_and(|scope| scope != track.name)
        {
            continue;
        }
        for element in &track.elements {
            if !(element.start < ask.at && ask.at < element.end) {
                continue;
            }
            let Some((from, to)) = stagger_window(element) else {
                continue;
            };
            if from < ask.at && ask.at < to {
                cut.push(
                    Finding::new("E-SHIFT-UNITS-WINDOW")
                        .at_file(document.path())
                        .at_element(element.id.clone())
                        .at_track(track.name.clone())
                        .field("at", json!(ask.at))
                        .field("from", json!(from))
                        .field("to", json!(to)),
                );
            }
        }
    }
    if !cut.is_empty() {
        for finding in cut {
            report.push(finding);
        }
        return Answer { preamble, report };
    }

    // ---- The transform, over a clone: a refused edit must leave the file untouched. ----
    let old_slacks = slack::of(&document);
    let mut new_instants: HashMap<(String, Side), i64> = HashMap::new();
    let mut straddlers: Vec<Straddle> = Vec::new();
    let mut unwritable: Vec<(String, String, Unwritable)> = Vec::new();

    for track in &mut project.tracks {
        let in_scope = ask.scope.as_deref().is_none_or(|scope| scope == track.name);
        for element in &mut track.elements {
            match transform_element(element, ask.at, ask.delta, in_scope) {
                Outcome::Straddle => {
                    straddlers.push(Straddle {
                        id: element.id.clone(),
                        track: track.name.clone(),
                        start: element.start,
                        end: element.end,
                    });
                }
                Outcome::Unwritable(refusal) => {
                    unwritable.push((element.id.clone(), track.name.clone(), refusal));
                }
                Outcome::Untouched | Outcome::Moved => {}
            }
            new_instants.insert((element.id.clone(), Side::Start), element.start);
            new_instants.insert((element.id.clone(), Side::End), element.end);
        }
    }

    if !straddlers.is_empty() {
        for straddle in &straddlers {
            report.push(
                Finding::new("E-SHIFT-STRADDLE")
                    .at_file(document.path())
                    .at_element(straddle.id.clone())
                    .at_track(straddle.track.clone())
                    .field("at", json!(ask.at))
                    .field("delta", json!(ask.delta))
                    .field("start", json!(straddle.start))
                    .field("end", json!(straddle.end)),
            );
        }
        return Answer { preamble, report };
    }

    // ADR-0146 §7: a split writes a new keyframe, and its value must be a legal literal.
    // Clamping it instead would bend both halves of the curve without saying so.
    if !unwritable.is_empty() {
        for (element, track, refusal) in &unwritable {
            report.push(
                Finding::new("E-SHIFT-SPLIT-UNWRITABLE")
                    .at_file(document.path())
                    .at_element(element.clone())
                    .at_track(track.clone())
                    .field("property", json!(refusal.property))
                    .field("at", json!(ask.at))
                    .field("value", json!(refusal.value)),
            );
        }
        return Answer { preamble, report };
    }

    // ---- Slack (ADR-0032, ADR-0047). ---------------------------------------------------
    let duration_value = document.value().get("duration").and_then(Value::as_i64);
    let mut used_release: HashSet<(i64, i64)> = HashSet::new();
    let mut changed: Vec<(&Slack, i64)> = Vec::new();
    for old in &old_slacks {
        // A slack whose both ends are the *same* element's own `start` and `end`, with no
        // other boundary coinciding at either, is not space *between* elements at all —
        // it is the element's own on-screen span, and stretching a time-invariant
        // straddler's `end` (ADR-0005) is exactly what widens it. Exempt it, or `shift`
        // would refuse the one thing it exists to do.
        if is_self_span(old) {
            continue;
        }
        let new_from = mapped_from(old.from, &old.from_edges, duration_value, &new_instants);
        let new_to = mapped_to(old.to, &old.to_edges, duration_value, &new_instants);
        let new_size = new_to - new_from;
        if new_size != old.size() {
            if ask.release.contains(&(old.from, old.to)) {
                used_release.insert((old.from, old.to));
            } else {
                changed.push((old, new_size));
            }
        }
    }

    if !changed.is_empty() {
        for (slack, new_size) in &changed {
            report.push(slack_finding(slack, *new_size, document.path()));
        }
        return Answer { preamble, report };
    }

    let stale: Vec<(i64, i64)> = ask
        .release
        .iter()
        .copied()
        .filter(|pair| !used_release.contains(pair))
        .collect();
    if !stale.is_empty() {
        for (from, to) in stale {
            report.push(
                Finding::new("E-SHIFT-RELEASE-INVALID")
                    .at_file(document.path())
                    .field("from", json!(from))
                    .field("to", json!(to)),
            );
        }
        return Answer { preamble, report };
    }

    // ---- The write. `shift` re-emits the whole file in the canonical convention, or not
    // at all (ADR-0011), reusing ticket 6's atomic-write machinery. --------------------
    let value = serde_json::to_value(&project).unwrap_or(Value::Null);
    let canonical = write::canonical(&value);
    if let Err(e) = write::atomically(path, &canonical) {
        report.could_not_write(document.path(), &e);
        return refused(report);
    }

    // The write-tool invariant (ADR-0011): every write tool returns the new state's
    // findings, never `ok` — knowingly partial at this point, since only tickets 5 and
    // 10-15's checks exist when #220 was scoped. Reusing `validate` (ticket 6's engine) is
    // what keeps that partial-ness a property of the check set rather than of this verb.
    let mut report = crate::verbs::validate::validate(path);
    report.tool = TOOL.to_string();
    Answer { preamble, report }
}

struct Straddle {
    id: String,
    track: String,
    start: i64,
    end: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
    Untouched,
    Moved,
    Straddle,
    /// A split would write a value no literal of the property can hold (ADR-0146 §7).
    Unwritable(Unwritable),
}

/// The property a split could not write, and the value it would have written.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Unwritable {
    property: String,
    value: String,
}

/// ADR-0005's table, amended by ADR-0012 for keyframes: what one element does under a
/// shift at `at` by `delta`, given whether it is in `scope`.
fn transform_element(element: &mut Element, at: i64, delta: i64, in_scope: bool) -> Outcome {
    if !in_scope || element.end <= at {
        return Outcome::Untouched;
    }
    if element.start >= at {
        element.start += delta;
        element.end += delta;
        if let Err(refusal) = edit_lists(&mut element.body, |_, list| {
            carry(list, delta);
            Ok(())
        }) {
            return Outcome::Unwritable(refusal);
        }
        carry_unit_lists(&mut element.body, delta);
        return Outcome::Moved;
    }
    // `start < at < end`: a straddler.
    if is_time_based(&element.body) {
        return Outcome::Straddle;
    }
    element.end += delta;
    // The stagger's lists are never split: a split at `at` would be right for one unit only,
    // so a cut inside the window was refused before this (ADR-0151 §5). A cut at or before
    // the window carries every unit list whole; one at or after its end leaves them be.
    let carries_units = stagger_window(element).is_some_and(|(start, _)| start >= at);
    match edit_lists(&mut element.body, |property, list| {
        split_list(property, list, at, delta)
    }) {
        Ok(()) => {
            if carries_units {
                carry_unit_lists(&mut element.body, delta);
            }
            Outcome::Moved
        }
        Err(refusal) => Outcome::Unwritable(refusal),
    }
}

/// Every stagger list moves by `delta`: the `units` block's and every run override's
/// (ADR-0151 §5). They are nested inside the element, so they are their own walk beside the
/// element's own animatable properties. A delay is relative to the lists, so it is carried
/// unchanged.
fn carry_unit_lists(body: &mut Body, delta: i64) {
    let Body::Text(_) = body else {
        return;
    };
    let Ok(mut value) = serde_json::to_value(&*body) else {
        return;
    };
    if let Some(block) = value.get_mut("units") {
        for property in crate::units::LISTS {
            if let Some(list) = block.get_mut(property) {
                carry(list, delta);
            }
        }
    }
    for run in value
        .get_mut("runs")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        if let Some(unit) = run.get_mut("unit") {
            for property in crate::units::LISTS {
                if let Some(list) = unit.get_mut(property) {
                    carry(list, delta);
                }
            }
        }
    }
    if let Ok(edited) = serde_json::from_value(value) {
        *body = edited;
    }
}

/// A text element's stagger window (ADR-0151 §5), read through the same plan the painter
/// and `query --at` read.
fn stagger_window(element: &Element) -> Option<(i64, i64)> {
    let value = serde_json::to_value(element).ok()?;
    crate::units::Plan::of(&value)?.window()
}

/// Does this element's source carry a clock? `video` and `audio` do (ADR-0005); every
/// other type has no internal clock and may be stretched instead of refused.
fn is_time_based(body: &Body) -> bool {
    matches!(body, Body::Video(_) | Body::Audio(_))
}

/// A boundary's own new instant, per the one edge that names it — see [`mapped_from`] and
/// [`mapped_to`], the two ways a slack reads a set of them.
fn new_instant_of(edge: &Edge<'_>, new_instants: &HashMap<(String, Side), i64>) -> i64 {
    *new_instants
        .get(&(edge.element.to_string(), edge.side))
        .expect("every element's new instant was recorded during the transform pass")
}

/// A slack's `from`, mapped to what it becomes.
///
/// A `from` with no edge at all can only be the project's own `duration` (`slack::of`
/// inserts it whether or not any element also sits there) — and `shift` never rewrites
/// `duration` directly. Consuming the lead-out it bounds is what an explicit `release` is
/// for, not an implicit side effect of moving whatever precedes it.
///
/// **Multiple edges at one instant are not interchangeable.** ADR-0047: *"a boundary that
/// is simultaneously the far edge of one slack and the moving edge of an adjacent one is
/// the ordinary shape of a tightly packed timeline"* — `a1` ending exactly where `a2`
/// starts is the normal case, and the two can move differently under one `shift` call
/// (`a1` untouched, `a2` dragged downstream). Picking an arbitrary edge as *the* answer
/// would make whether an unrelated slack is flagged depend on element declaration order
/// rather than on anything that changed. The correct reading, for `from`, is the
/// **latest** of the edges' new instants: `from` is the point up to which everything
/// *before* this slack now extends, and that is decided by whichever of the coincident
/// edges moved furthest forward.
fn mapped_from(
    original: i64,
    edges: &[Edge<'_>],
    duration_value: Option<i64>,
    new_instants: &HashMap<(String, Side), i64>,
) -> i64 {
    // An explicit `duration` field is a literal the file states and `shift` never
    // rewrites, so a boundary that happens to sit at it stays there even if an edge
    // coinciding with it moves — the two are no longer coincident, which is a fact for
    // `compare` (ADR-0036), not a licence for this slack to silently track the edge.
    if Some(original) == duration_value {
        return original;
    }
    edges
        .iter()
        .map(|edge| new_instant_of(edge, new_instants))
        .max()
        .unwrap_or(original)
}

/// A slack's `to`, mapped to what it becomes — the mirror of [`mapped_from`]: the
/// **earliest** of the coincident edges' new instants, since `to` is the point before
/// which everything *after* this slack now begins.
fn mapped_to(
    original: i64,
    edges: &[Edge<'_>],
    duration_value: Option<i64>,
    new_instants: &HashMap<(String, Side), i64>,
) -> i64 {
    if Some(original) == duration_value {
        return original;
    }
    edges
        .iter()
        .map(|edge| new_instant_of(edge, new_instants))
        .min()
        .unwrap_or(original)
}

/// Whether a slack's two ends are one element's own `start` and `end`, with nothing else
/// coinciding at either — see the exemption at its one call site.
fn is_self_span(slack: &Slack) -> bool {
    let [from] = slack.from_edges.as_slice() else {
        return false;
    };
    let [to] = slack.to_edges.as_slice() else {
        return false;
    };
    from.element == to.element && from.side == Side::Start && to.side == Side::End
}

fn slack_finding(slack: &Slack, new_size: i64, file: &str) -> Finding {
    Finding::new("E-SHIFT-SLACK")
        .at_file(file)
        .field("from", json!(slack.from))
        .field("to", json!(slack.to))
        .field("size", json!(slack.size()))
        .field("new_size", json!(new_size))
        .field(
            "from_edges",
            json!(describe_edges(&slack.from_edges, false)),
        )
        .field(
            "to_edges",
            json!(describe_edges(&slack.to_edges, slack.to_duration)),
        )
}

fn describe_edges(edges: &[Edge<'_>], to_duration: bool) -> String {
    if to_duration {
        return "the project's `duration`".to_string();
    }
    edges
        .iter()
        .map(|edge| {
            format!(
                "`{}`.{}",
                edge.element,
                match edge.side {
                    Side::Start => "start",
                    Side::End => "end",
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The coincident-instant preamble (ADR-0036), computed against the document before
/// anything moves. Unconditional: printed whenever `at` lands on an existing element
/// boundary or keyframe `t`, never gated behind a flag.
fn coincident_preamble(project: &Project, at: i64) -> Vec<CoincidentRecord> {
    let mut out = Vec::new();
    for track in &project.tracks {
        for element in &track.elements {
            if element.start == at {
                out.push(CoincidentRecord {
                    element: element.id.clone(),
                    kind: "boundary",
                    role: "start",
                    property: None,
                    effect: "at or after `at`: moves with the element".to_string(),
                });
            }
            if element.end == at {
                out.push(CoincidentRecord {
                    element: element.id.clone(),
                    kind: "boundary",
                    role: "end",
                    property: None,
                    effect: "before `at` (half-open): stays".to_string(),
                });
            }
            for (property, times) in animatable_lists(&element.body) {
                if times.is_empty() {
                    continue;
                }
                for (index, t) in times.iter().enumerate() {
                    if *t != at {
                        continue;
                    }
                    let role = if index == 0 {
                        "first"
                    } else if index + 1 == times.len() {
                        "last"
                    } else {
                        "interior"
                    };
                    out.push(CoincidentRecord {
                        element: element.id.clone(),
                        kind: "keyframe",
                        role,
                        property: Some(property.clone()),
                        effect: "carried with its element, not moved by the at-or-after \
                                 rule (ADR-0012)"
                            .to_string(),
                    });
                }
            }
        }
    }
    out
}

/// Every animated property's keyframe `t` list, for the coincidence preamble — read off the
/// one list the schema types (ADR-0146), as the plain shift and SPLIT read it.
fn animatable_lists(body: &Body) -> Vec<(String, Vec<i64>)> {
    let Ok(value) = serde_json::to_value(body) else {
        return Vec::new();
    };
    let times = |records: &Vec<Value>| -> Vec<i64> {
        records
            .iter()
            .filter_map(|record| record.get("t")?.as_i64())
            .collect()
    };
    animatable::of(body.type_name())
        .iter()
        .filter_map(|property| {
            let records = animatable::records(&value, &property.name)?;
            Some((property.name.clone(), times(records)))
        })
        .chain(keyed_feathers(&value).into_iter().map(|index| {
            let records = animatable::records(&value["effects"][index], "feather");
            (feather(index).name, records.map(times).unwrap_or_default())
        }))
        .collect()
}

/// The positions in `effects` of every `mask` whose `feather` is keyed (ADR-0152 §2).
///
/// By hand, because the derived list does not yet walk the nested paths inside `effects`:
/// #676 brings every effect parameter into it, and this and [`feather`] go with it.
fn keyed_feathers(body: &Value) -> Vec<usize> {
    body.get("effects")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .filter(|(_, effect)| {
            effect.get("name").and_then(Value::as_str) == Some("mask")
                && animatable::records(effect, "feather").is_some()
        })
        .map(|(index, _)| index)
        .collect()
}

/// The `feather` of the mask at `index`, as the one list would describe it: an integer
/// length, bounded below by `0`, named by its position as ADR-0146 §4 names an effect
/// parameter.
fn feather(index: usize) -> Property {
    Property {
        name: format!("effects[{index}].feather (mask)"),
        kind: Kind::Integer,
        minimum: Some(0.0),
    }
}

/// Edit every keyframe list `body` carries, through the one list of animatable properties
/// the schema types (ADR-0146) — so a property joins `shift` by being typed animatable, and
/// no list here can leave one behind.
///
/// The body is read as the document writes it, each list edited in place, and the whole read
/// back as the model, so the edit meets the same reader the file will. A refusal leaves the
/// body as it was.
fn edit_lists(
    body: &mut Body,
    mut edit: impl FnMut(&Property, &mut Value) -> Result<(), Unwritable>,
) -> Result<(), Unwritable> {
    let Ok(mut value) = serde_json::to_value(&*body) else {
        return Ok(());
    };
    let mut touched = false;
    for property in animatable::of(body.type_name()) {
        if animatable::records(&value, &property.name).is_none() {
            continue;
        }
        edit(property, &mut value[property.name.as_str()])?;
        touched = true;
    }
    for index in keyed_feathers(&value) {
        edit(&feather(index), &mut value["effects"][index]["feather"])?;
        touched = true;
    }
    if !touched {
        return Ok(());
    }
    match serde_json::from_value(value) {
        Ok(edited) => {
            *body = edited;
            Ok(())
        }
        // SPLIT and the plain shift write only what the model reads; reaching this is this
        // verb contradicting itself, and is refused rather than written.
        Err(e) => Err(Unwritable {
            property: "(the element)".to_string(),
            value: format!("a keyframe list the model does not read back: {e}"),
        }),
    }
}

/// The "entirely after" row: every keyframe moves by `delta`, including those before `at`
/// (ADR-0012's amendment to ADR-0005).
fn carry(list: &mut Value, delta: i64) {
    for record in list.as_array_mut().into_iter().flatten() {
        if let Some(t) = record.get("t").and_then(Value::as_i64) {
            record["t"] = json!(t + delta);
        }
    }
}

/// SPLIT (ADR-0012) on one property's list, written as the property's [`Kind`] says:
/// an integer to the nearest integer, ties away from zero; a number to six places; a pair
/// component by component; a colour to bytes (ADR-0146 §7).
fn split_list(
    property: &Property,
    list: &mut Value,
    at: i64,
    delta: i64,
) -> Result<(), Unwritable> {
    let refuse = |value: String| Unwritable {
        property: property.name.clone(),
        value,
    };
    // A bound the schema states on every value is a bound the written split must keep.
    let bounded = |raw: f64| match property.minimum {
        Some(minimum) if raw < minimum => Err(refuse(format!("{}", round6(raw)))),
        _ => Ok(raw),
    };
    match property.kind {
        Kind::Integer => split_typed::<i64>(list, at, delta, |raw| bounded(raw).map(round_i64)),
        Kind::Number => split_typed::<f64>(list, at, delta, |raw| bounded(raw).map(round6)),
        Kind::Pair => split_typed::<Scale>(list, at, delta, |raw| Ok(round_scale(raw))),
        Kind::Colour | Kind::Paint => split_typed::<Colour>(list, at, delta, |blend: Blend| {
            if blend.in_range() {
                Ok(blend.settle())
            } else {
                let [r, g, b, a] = blend.0.map(round6);
                Err(refuse(format!(
                    "a colour outside its range (premultiplied red {r}, green {g}, blue {b}, \
                     alpha {a})"
                )))
            }
        }),
        // A vertex list is integer pixels, and rounding one coordinate would bend the drawing
        // without saying so: a split is written only where every number already is an
        // integer (ADR-0154 §3, ADR-0146 §7).
        Kind::Points => split_typed::<Points>(list, at, delta, |vertices: Vec<VertexAt>| {
            integer_points(&vertices).ok_or_else(|| {
                refuse(format!(
                    "a vertex list with a fractional coordinate ({})",
                    serde_json::to_string(&vertices).unwrap_or_default()
                ))
            })
        }),
    }
}

/// The vertex list `vertices` spells, where every coordinate is an integer.
fn integer_points(vertices: &[VertexAt]) -> Option<Points> {
    let whole = |value: f64| {
        let rounded = value.round();
        ((value - rounded).abs() < 1e-9).then_some(rounded as i64)
    };
    let pair = |[x, y]: [f64; 2]| Some([whole(x)?, whole(y)?]);
    vertices
        .iter()
        .map(|vertex| {
            Some(Vertex {
                at: pair(vertex.at)?,
                arriving: match vertex.arriving {
                    Some(offset) => Some(pair(offset)?),
                    None => None,
                },
                out: match vertex.out {
                    Some(offset) => Some(pair(offset)?),
                    None => None,
                },
            })
        })
        .collect::<Option<Vec<_>>>()
        .map(Points)
}

fn split_typed<T>(
    list: &mut Value,
    at: i64,
    delta: i64,
    write: impl Fn(<T as Interpolate>::Out) -> Result<T, Unwritable>,
) -> Result<(), Unwritable>
where
    T: Interpolate + Clone + PartialEq + Serialize + serde::de::DeserializeOwned,
{
    // A list that does not read as the property's type is the schema check's to report, and
    // `shift` runs only on a document that validated clean.
    let Ok(mut records) = serde_json::from_value::<Vec<Keyframe<T>>>(list.clone()) else {
        return Ok(());
    };
    split_keyed(&mut records, at, delta, &write)?;
    if let Ok(written) = serde_json::to_value(&records) {
        *list = written;
    }
    Ok(())
}

fn round_i64(v: f64) -> i64 {
    // "Ties away from zero" (ADR-0012), which is exactly `f64::round`'s own rule.
    v.round() as i64
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

fn round_scale(v: [f64; 2]) -> [f64; 2] {
    [round6(v[0]), round6(v[1])]
}

/// SPLIT's seven steps, over one property's records.
fn split_keyed<T>(
    records: &mut Vec<Keyframe<T>>,
    at: i64,
    delta: i64,
    write: &impl Fn(<T as Interpolate>::Out) -> Result<T, Unwritable>,
) -> Result<(), Unwritable>
where
    T: Interpolate + Clone + PartialEq,
{
    records.sort_by_key(|record| record.t);
    let (Some(first_t), Some(last_t)) = (
        records.first().map(|record| record.t),
        records.last().map(|record| record.t),
    ) else {
        return Ok(());
    };

    // Steps 2-4, 6: computed against the *original* list, before step 5 touches anything,
    // so the segment indices below stay valid through the shift.
    let mut pending: Option<(usize, Keyframe<T>, Keyframe<T>)> = None;
    let mut remove: Option<usize> = None;

    if at > first_t && at < last_t {
        if let Some(index) = records.iter().position(|record| record.t == at) {
            // Step 3: a record already sits at `at`. A copy stays; the original moves to
            // `at + delta` and becomes the hold segment's own arrival.
            let existing = records[index].clone();
            pending = Some((
                index,
                Keyframe {
                    t: at,
                    // The same instant as the record that was there, so it keeps that
                    // record's own claim about where the instant came from (ADR-0086).
                    t_from: existing.t_from,
                    v: existing.v.clone(),
                    ease: existing.ease.clone(),
                },
                Keyframe {
                    t: at + delta,
                    // A record this verb synthesised, at an instant no author wrote. An
                    // absent `t_from` means *no claim*, which is the honest thing for
                    // `shift` to say about its own arithmetic (ADR-0086).
                    t_from: None,
                    v: existing.v,
                    ease: Some(Ease::Named(EaseName::Step)),
                },
            ));
            remove = Some(index);
        } else if let Some(segment) = (0..records.len().saturating_sub(1))
            .find(|&i| records[i].t < at && at < records[i + 1].t)
        {
            // Step 4/6: cut an existing segment. `b`'s ease (the segment entering it) is
            // what describes the cut; `a` is untouched.
            //
            // **`b`'s own `t_from` is left exactly as written, and where it declares
            // `after-previous` this cut makes it stale** — two records are spliced in ahead
            // of it, so "the previous record" is now one of them and no longer `a`. That is
            // reported rather than repaired: `validate` answers with `R-DERIVED-T` and the
            // integer the rule now derives (ADR-0086), and every write tool hands back the
            // new state's findings (ADR-0011), so the agent is told. Silently rewriting the
            // `ms` would make `shift` the second author of a claim only the author can make,
            // and silently dropping the key would discard that claim without saying so —
            // ADR-0086 makes an absent declaration mean *no claim*, which is not what
            // happened here.
            let a = records[segment].clone();
            let b_ease = records[segment + 1]
                .ease
                .clone()
                .expect("ADR-0038: every non-first record carries an ease");
            let fraction = (at - a.t) as f64 / (records[segment + 1].t - a.t) as f64;
            let (left, right, progress) = subdivide(&b_ease, fraction);
            // The value is written before anything in the list moves, so a refusal leaves
            // the list as it was.
            let v = write(T::between(&a.v, &records[segment + 1].v, progress))?;
            records[segment + 1].ease = Some(right);
            pending = Some((
                segment + 1,
                Keyframe {
                    t: at,
                    // Both records are this verb's own, at instants no author wrote, so
                    // neither carries a claim (ADR-0086).
                    t_from: None,
                    v: v.clone(),
                    ease: Some(left),
                },
                Keyframe {
                    t: at + delta,
                    t_from: None,
                    v,
                    ease: Some(Ease::Named(EaseName::Step)),
                },
            ));
        }
    }

    // Step 5: every keyframe with an *original* `t > at` shifts by `delta`. The records
    // `pending` will insert are already at their final `at`/`at + delta` positions and are
    // not part of this loop.
    for record in records.iter_mut() {
        if record.t > at {
            record.t += delta;
        }
    }

    if let Some(index) = remove {
        records.remove(index);
    }
    if let Some((index, left, right)) = pending {
        records.splice(index..index, [left, right]);
    }

    // Step 7: collapse any run of three or more consecutive records sharing one `v` and
    // one `ease` to its two endpoints.
    collapse_holds(records);
    Ok(())
}

/// The eased progress `y` at `x = fraction` on `ease`'s curve, and `ease` split there into
/// its entering-left and entering-right halves — De Casteljau subdivision of the cubic,
/// rescaled back into `[0,1]x[0,1]` (ADR-0012: the raw form is forced, never a named one).
fn subdivide(ease: &Ease, fraction: f64) -> (Ease, Ease, f64) {
    if matches!(ease, Ease::Named(EaseName::Step)) {
        // Not a curve (ADR-0012): the value holds through the whole segment, so both
        // halves hold too, and the value at any interior instant is the earlier endpoint's.
        return (
            Ease::Named(EaseName::Step),
            Ease::Named(EaseName::Step),
            0.0,
        );
    }
    let [x1, y1, x2, y2] = match ease {
        Ease::Named(name) => name
            .bezier()
            .expect("`step` is handled above; every other named ease has control points"),
        Ease::Bezier(points) => *points,
    };
    if x1 == y1 && x2 == y2 {
        // The identity curve — `linear`, or any raw spelling of it — is closed under
        // subdivision: each half is the identity again (ADR-0012).
        return (
            Ease::Named(EaseName::Linear),
            Ease::Named(EaseName::Linear),
            fraction,
        );
    }

    let s = resolve::solve(x1, x2, fraction);
    let p0 = (0.0, 0.0);
    let p1 = (x1, y1);
    let p2 = (x2, y2);
    let p3 = (1.0, 1.0);
    let a = lerp(p0, p1, s);
    let b = lerp(p1, p2, s);
    let c = lerp(p2, p3, s);
    let d = lerp(a, b, s);
    let e = lerp(b, c, s);
    let q = lerp(d, e, s);

    (
        Ease::Bezier(rescale(p0, a, d, q)),
        Ease::Bezier(rescale(q, e, c, p3)),
        q.1,
    )
}

fn lerp(a: (f64, f64), b: (f64, f64), t: f64) -> (f64, f64) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

/// Rescale a bezier sub-curve running `start`..`end` (through `c1`, `c2`) back into the
/// `[0,1]x[0,1]` frame every [`Ease::Bezier`] is written in.
fn rescale(start: (f64, f64), c1: (f64, f64), c2: (f64, f64), end: (f64, f64)) -> [f64; 4] {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let scale_x = |v: f64| {
        if dx == 0.0 {
            0.0
        } else {
            ((v - start.0) / dx).clamp(0.0, 1.0)
        }
    };
    let scale_y = |v: f64| if dy == 0.0 { 0.0 } else { (v - start.1) / dy };
    [scale_x(c1.0), scale_y(c1.1), scale_x(c2.0), scale_y(c2.1)]
}

/// Step 7: collapse a run of 3+ consecutive records sharing one `v` and one `ease` to its
/// two endpoints. Without this, repeated shifts through one segment grow a 2-record list
/// without bound (ADR-0012).
fn collapse_holds<T: PartialEq>(records: &mut Vec<Keyframe<T>>) {
    let n = records.len();
    if n < 3 {
        return;
    }
    let mut keep = vec![true; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n
            && records[j + 1].v == records[i].v
            && (j + 1 == i + 1 || records[j + 1].ease == records[i + 1].ease)
        {
            j += 1;
        }
        if j - i + 1 >= 3 {
            for slot in keep.iter_mut().take(j).skip(i + 1) {
                *slot = false;
            }
        }
        i = j + 1;
    }
    let mut index = 0;
    records.retain(|_| {
        let keep_this = keep[index];
        index += 1;
        keep_this
    });
}
