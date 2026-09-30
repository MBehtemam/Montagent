//! *"What does quantization change?"* — never *"what is unaligned?"*
//!
//! ADR-0006 rewrote this check before it was ever implemented, and the fixture is the
//! argument:
//!
//! > ADR-0005 asks `validate` to *"note boundaries that are not frame-aligned."* **That
//! > instruction is wrong as literally written, and the fixture is the proof:** 109 of its
//! > 120 time values — 47 of 48 distinct instants — are off the 40 ms grid at the
//! > project's own 25 fps, the only aligned instant being `0`. A check with a 98% hit rate
//! > on a correct, published project is not a check; it is the alarm fatigue this ADR
//! > already identified as a safety problem.
//!
//! So alignment is not the question. The question is whether the frames the renderer
//! actually samples still show what the document declares, and ADR-0006 names the two ways
//! they might not:
//!
//! 1. **An element shorter than a frame**, which *"rounds out of existence"* — no sampled
//!    instant falls inside its half-open range, so it never appears. *"No element is
//!    shorter than a frame (the shortest is 1000 ms)"* on the fixture.
//! 2. **A gap shorter than a frame**, the other half of *"rounding can never manufacture
//!    an overlap or a gap"* — the black frames a gap declares are not there if no sampled
//!    instant falls in it. The fixture's shortest gap is 450 ms.
//!
//! An **overlap** is the case that cannot arise, and it is worth saying why rather than
//! leaving it untested: *"every cut is an exact adjacency, so both sides quantize to the
//! same frame."* One element ends where the next begins, at one instant, and one instant
//! resolves to one frame — there is no rounding by which the two sides could disagree.
//!
//! The grid is ADR-0035's and is evaluated in exact rational arithmetic
//! ([`crate::exact::holds_a_sampled_frame`]): the step is `1000/fps` ms and *"not
//! necessarily integral"* — at 30 fps it is `100/3` and only multiples of 100 ms are
//! frame-exact, *"the non-obvious fact one agent found only by building a private
//! checker"*.
//!
//! ADR-0105 adds a third condition on ADR-0006's own reason for the first two:
//!
//! 3. **A visual state the grid never paints.** Two boundaries on *different* elements
//!    less than a frame apart bound a span whose visual presence set the document declares
//!    and no frame shows. No element vanishes and no track gains a gap, so neither (1) nor
//!    (2) sees it — *"one fact, arrived at through two elements rather than one"*. The
//!    states are [`cuts::visual_states`] (ADR-0094 §1), and [`unpainted_states`] is the
//!    finding. `frame`'s range mode is to tile the same selection and raise the same
//!    finding for each `no-grid-frame` run (#488); it is not built yet.
//!
//! On a project where quantization changes nothing, this check emits nothing at all.

use serde_json::{Value, json};

use crate::exact;
use crate::finding::{Class, Finding};
use crate::permissive::Loose;
use crate::report::Report;
use crate::track;
use crate::verbs::query::Named;
use crate::verbs::query::cuts::{self, Interval};
use crate::verbs::render::{extent, instant_of};

/// The whole project, against its own frame grid.
///
/// One finding, not one per boundary. ADR-0006's answer on a clean project is *"one line
/// — nothing changes"*, and its complaint about the literal check was volume; a
/// per-boundary finding would reproduce that shape with a smaller constant.
pub fn check(document: &Loose, report: &mut Report) {
    let Some(fps) = document.value().get("fps").and_then(Value::as_i64) else {
        // No grid, so no question. `fps` is required, and the check that says so owns the
        // schema.
        return;
    };

    vanished(document, fps, report);

    // Over the whole clock the render paints: `[0, extent)`, the range `render` with no
    // `--from`/`--to` draws frames for.
    if let Some(end) = extent(document) {
        for finding in unpainted_states(document, fps, 0, end) {
            report.push(finding);
        }
    }
}

/// ADR-0006's two conditions: an element, or a gap in one track, that no frame falls in.
fn vanished(document: &Loose, fps: i64, report: &mut Report) {
    let mut changed: Vec<i64> = Vec::new();
    let mut detail: Vec<String> = Vec::new();

    for sequence in track::sequences(document) {
        for span in &sequence.spans {
            if exact::holds_a_sampled_frame(span.start, span.end, fps) == Some(false) {
                changed.extend([span.start, span.end]);
                detail.push(format!(
                    "`{}` ({}..{} ms) holds no sampled frame",
                    span.element, span.start, span.end
                ));
            }
        }
        for gap in sequence.gaps() {
            if exact::holds_a_sampled_frame(gap.from, gap.to, fps) == Some(false) {
                changed.extend([gap.from, gap.to]);
                detail.push(format!(
                    "the {} ms gap in track `{}` between `{}` and `{}` ({}..{} ms) holds none",
                    gap.size(),
                    gap.track,
                    gap.after,
                    gap.before,
                    gap.from,
                    gap.to
                ));
            }
        }
    }

    if detail.is_empty() {
        return;
    }

    // Distinct instants: one boundary shared by an element that vanishes and the gap after
    // it is one thing the grid changed, not two.
    changed.sort_unstable();
    changed.dedup();

    report.push(
        // ADR-0006: "Escalate to `review` only for the cases in (1) and (2)" — which are
        // the only two cases this check fires on, so every finding it emits is one. The
        // frames the renderer samples do not show what the document declares, and no
        // arithmetic can tell you whether that was meant.
        Finding::at_class("N-QUANTIZATION", Class::Review)
            .at_file(document.path())
            .field("fps", json!(fps))
            .field("changed", json!(changed.len()))
            .field("detail", json!(detail.join("; "))),
    );
}

/// One `N-QUANTIZATION` per visual state in `states` that holds no painted frame
/// `⌊n × 1000 / fps⌋`. `validate` raises it today, and `frame`'s range mode is to raise it
/// through this same function (#488), so the state has one identity whichever verb saw it
/// (ADR-0105 §5). Crate-visible, not private, for that second caller.
///
/// ADR-0118 ratifies this shape. One finding per state, not one for the document as (1) and (2) are: a state is the unit
/// the sheet skips, and `frame` raises one per `no-grid-frame` run. A state that is unpainted
/// *because* an element inside it vanishes, or because it is a gap, is reported here as well
/// as above — the spec's condition is "every visual state", and the two findings are about
/// different subjects, an element and a combination.
///
/// The states are [`cuts::visual_states`] over `[from, to)`. What changes at a state's
/// boundary is read off its neighbours, and a state at either end of the range reads the
/// visual state just outside it. So a state that closes the document still names what leaves
/// at its end, and nothing is named at an edge only where nothing visual changes there.
pub(crate) fn unpainted_states(document: &Loose, fps: i64, from: i64, to: i64) -> Vec<Finding> {
    let states = cuts::visual_states(&cuts::cuts(document, from, to));
    // One millisecond either side is enough: the presence set is constant over an interval
    // and is read at the instant it opens.
    let outside = |start: i64, end: i64| {
        cuts::visual_states(&cuts::cuts(document, start, end))
            .into_iter()
            .next()
    };
    let before_range = outside(from.saturating_sub(1), from);
    let after_range = outside(to, to.saturating_add(1));

    states
        .iter()
        .enumerate()
        .filter(|(_, state)| {
            exact::holds_a_sampled_frame(state.start, state.end, fps) == Some(false)
        })
        .map(|(index, state)| {
            let before = match index.checked_sub(1) {
                Some(previous) => states.get(previous),
                None => before_range.as_ref(),
            };
            let after = states.get(index + 1).or(after_range.as_ref());
            let opening = Change::between(state.start, before, Some(state));
            let closing = Change::between(state.end, Some(state), after);
            // The frames either side: the last one before the state opens and the first one
            // at or after it closes. Both exist for any state that can be unpainted — one
            // opening at 0 holds frame 0.
            let painted = |frame: Option<exact::Sampled>| frame.map(|f| instant_of(f.frame, fps));
            let previous = painted(exact::frame_before(state.start, fps));
            let next = painted(exact::frame_at_or_after(state.end, fps));

            let mut detail = format!(
                "the visual state {}..{} ms {{{}}} holds no painted frame \u{2014} {} and {}",
                state.start,
                state.end,
                state
                    .present
                    .iter()
                    .map(|named| format!("`{}`", name(named)))
                    .collect::<Vec<_>>()
                    .join(", "),
                opening.prose(),
                closing.prose(),
            );
            if let (Some(previous), Some(next)) = (previous, next) {
                detail.push_str(&format!(
                    ", between the frames painted at {previous} and {next} ms"
                ));
            }

            Finding::at_class("N-QUANTIZATION", Class::Review)
                .at_file(document.path())
                .field("fps", json!(fps))
                .field("changed", json!(2))
                .field("from", json!(state.start))
                .field("to", json!(state.end))
                .field(
                    "present",
                    json!(state.present.iter().map(name).collect::<Vec<_>>()),
                )
                .field("boundaries", json!([opening.to_json(), closing.to_json()]))
                .field("detail", json!(detail))
        })
        .collect()
}

/// What the visual presence set does at one boundary of a state.
struct Change<'a> {
    at: i64,
    entering: Vec<&'a Named>,
    leaving: Vec<&'a Named>,
}

impl<'a> Change<'a> {
    /// The difference between the states either side of `at`. A missing side is the edge
    /// of the range, where nothing is named as entering or leaving.
    fn between(at: i64, before: Option<&'a Interval>, after: Option<&'a Interval>) -> Self {
        let (Some(before), Some(after)) = (before, after) else {
            return Change {
                at,
                entering: Vec::new(),
                leaving: Vec::new(),
            };
        };
        let missing_from = |from: &'a Interval, of: &'a Interval| -> Vec<&'a Named> {
            of.present
                .iter()
                .filter(|named| !from.present.contains(named))
                .collect()
        };
        Change {
            at,
            entering: missing_from(before, after),
            leaving: missing_from(after, before),
        }
    }

    fn to_json(&self) -> Value {
        let members = |list: &[&Named]| -> Value {
            list.iter()
                .map(|named| json!({"element": name(named), "track": named.track}))
                .collect()
        };
        json!({
            "at": self.at,
            "entering": members(&self.entering),
            "leaving": members(&self.leaving),
        })
    }

    /// `` `a` (track `a`) leaves at 1010 ms ``, one clause per element.
    fn prose(&self) -> String {
        let clauses: Vec<String> = self
            .leaving
            .iter()
            .map(|named| (named, "leaves"))
            .chain(self.entering.iter().map(|named| (named, "enters")))
            .map(|(named, verb)| match &named.track {
                Some(track) => format!(
                    "`{}` (track `{track}`) {verb} at {} ms",
                    name(named),
                    self.at
                ),
                None => format!("`{}` {verb} at {} ms", name(named), self.at),
            })
            .collect();
        if clauses.is_empty() {
            format!("the range's edge is at {} ms", self.at)
        } else {
            clauses.join(", ")
        }
    }
}

/// An element's id, or `(no id)` for one without. `query`'s own placeholder also carries
/// the element's traversal index, which a state's members do not keep.
fn name(named: &Named) -> String {
    named.id.clone().unwrap_or_else(|| "(no id)".to_string())
}
