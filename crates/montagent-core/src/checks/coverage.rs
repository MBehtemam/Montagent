//! Cross-track coverage — *"does either side of an audio/visual pairing outrun the
//! other?"* — computed by **group-scoped pairing**, never a frame-wide union.
//!
//! ADR-0018 amends and withdraws ADR-0006's original design outright: a project-wide
//! union of "visual" coverage fires on **zero** of the committed fixture's eleven real
//! visual gaps, because eight always-on header elements (`group:"header"`) span the full
//! runtime and swallow every basis. There is no field this project will add to tell the
//! header apart from content (`kind` on a track was rejected 8/8 for the same reason
//! `gravity` was), so the fix is not a bigger union — it is asking a narrower question.
//!
//! `validate` instead pairs by `group`. For every `group` whose members include at least
//! one audio-bearing element (`audio`, or `video` — ADR-0055's *"one element with
//! intrinsic audio"*) and at least one visual element (`video` counts on this side too),
//! this reports every stretch where one side's time-union is not covered by the other's
//! — **symmetric**: audio active with no visual in the group is reported exactly like
//! visual active with no audio, per ADR-0018's court, which found the mechanism and the
//! symmetry both unanimous. A group missing either side — `header`, which carries no
//! audio member, or `loop-tail`, which carries no `group`d audio at all — is not
//! evaluated: absence of a pairing asserts nothing, which is ADR-0018's own discipline
//! (*"a false 'covered' is worse than nothing asserted"*).
//!
//! **Purely temporal.** No rect, no layer, no opacity: ADR-0018 rejected area-awareness
//! on this exact fixture (peak opaque coverage 79.64%, two chrome rects filled the exact
//! `background` byte value, real occlusion needing painter's order) as *"a much larger
//! check than the one it was meant to replace"*. This check reads only `group`, `type`,
//! `start` and `end`.
//!
//! # Which direction is `review`
//!
//! Not stated by any registered field — decided here, and worth recording why. ADR-0018's
//! two founding defects (#23) are both narration continuing to play with nothing to match
//! it on screen: audio active, no visual. Run against the committed fixture, that
//! direction is empty — the published video has no such stretch in any group — while the
//! opposite direction (a photo held on screen through an ordinary breath between two
//! narration lines) fires 21 times and is exactly the *"legal, ordinary"* gap `CONTEXT.md`
//! already describes. So audio-with-no-visual is `review` — the one direction ADR-0018's
//! own evidence says is a defect — and visual-with-no-audio is `note`, matching
//! `N-TRACK-GAP`'s own severity for the same kind of silence. `tests/fixture.rs` is the
//! check on this: the fixture's `review` count does not move.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::finding::{Class, Finding};
use crate::permissive::Loose;
use crate::report::Report;

/// One element, reduced to what a coverage question needs.
struct Span {
    element: String,
    start: i64,
    end: i64,
}

/// A maximal stretch covered by one or more of one side's spans, and everything that
/// contributed to it — kept rather than discarded, since naming *what* was playing is
/// what makes the finding actionable (`crate::finding`'s *"never 'see `X`'"*).
struct Block {
    start: i64,
    end: i64,
    spans: Vec<Span>,
}

/// Every group-paired coverage gap, both directions.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();

    for (group, sides) in groups(document) {
        if sides.audio.is_empty() || sides.visual.is_empty() {
            continue;
        }
        let audio = blocks(sides.audio);
        let visual = blocks(sides.visual);

        for gap in uncovered(&audio, &visual) {
            report.push(finding(&group, gap, "audio", "visual").at_file(file));
        }
        for gap in uncovered(&visual, &audio) {
            report.push(finding(&group, gap, "visual", "audio").at_file(file));
        }
    }
}

#[derive(Default)]
struct Sides {
    audio: Vec<Span>,
    visual: Vec<Span>,
}

/// Every element carrying a `group`, split into its two sides and grouped by name.
///
/// A `BTreeMap` rather than document order: `group` is not a traversal this project
/// orders by anywhere else, and a name-sorted report is one fewer thing "nothing
/// downstream may depend on which check spoke first" (ADR-0006) has to hold by accident.
fn groups(document: &Loose) -> BTreeMap<String, Sides> {
    let mut out: BTreeMap<String, Sides> = BTreeMap::new();
    for track in document
        .flat_value()
        .get("tracks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for element in track
            .get("elements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(group) = element.get("group").and_then(Value::as_str) else {
                continue;
            };
            let Some(start) = element.get("start").and_then(Value::as_i64) else {
                continue;
            };
            let Some(end) = element.get("end").and_then(Value::as_i64) else {
                continue;
            };
            if end <= start {
                continue;
            }
            let subject = subject_of(element.get("id").and_then(Value::as_str));
            let sides = out.entry(group.to_string()).or_default();
            // ADR-0055: *"a `video` element is one element with intrinsic audio, not a
            // visual paired with a separate audio element"* — the same reason
            // `crate::checks::caption::no_audio` matches `"audio" | "video"` rather than
            // `"audio"` alone. So a `video` is both sides at once, not a choice between
            // them: it contributes its own span to `visual` (it is on screen) and a
            // second, identical span to `audio` (its soundtrack plays for exactly as
            // long). `volume` is not consulted, on the same posture ADR-0054 took for
            // captions — *"under-flagging is the acceptable error direction"* — a `video`
            // muted for one edit is not proof its group's coverage question changed.
            match element.get("type").and_then(Value::as_str) {
                Some("audio") => sides.audio.push(Span {
                    element: subject,
                    start,
                    end,
                }),
                Some("video") => {
                    sides.audio.push(Span {
                        element: subject.clone(),
                        start,
                        end,
                    });
                    sides.visual.push(Span {
                        element: subject,
                        start,
                        end,
                    });
                }
                _ => sides.visual.push(Span {
                    element: subject,
                    start,
                    end,
                }),
            }
        }
    }
    out
}

/// One side's spans, merged into the maximal stretches they cover.
///
/// Spans sort by `(start, end, element)` first, the same tie-break `crate::track` uses,
/// so two runs over one document produce byte-identical reports regardless of the order
/// the elements were written in.
fn blocks(mut spans: Vec<Span>) -> Vec<Block> {
    spans.sort_by(|a, b| (a.start, a.end, &a.element).cmp(&(b.start, b.end, &b.element)));
    let mut out: Vec<Block> = Vec::new();
    for span in spans {
        // Half-open and touching counts as covering, the same boundary rule
        // `crate::track::Sequence::gaps` uses: an element ending where the next begins
        // leaves no instant uncovered.
        match out.last_mut() {
            Some(block) if span.start <= block.end => {
                block.end = block.end.max(span.end);
                block.spans.push(span);
            }
            _ => out.push(Block {
                start: span.start,
                end: span.end,
                spans: vec![span],
            }),
        }
    }
    out
}

/// Every stretch of `active`'s coverage that `other` does not reach.
fn uncovered(active: &[Block], other: &[Block]) -> Vec<Gap> {
    let mut out = Vec::new();
    for block in active {
        let mut cursor = block.start;
        for other_block in other {
            if other_block.end <= cursor {
                continue;
            }
            if other_block.start >= block.end {
                break;
            }
            if other_block.start > cursor {
                out.push(Gap {
                    from: cursor,
                    to: other_block.start.min(block.end),
                    elements: elements_in(block, cursor, other_block.start.min(block.end)),
                });
            }
            cursor = cursor.max(other_block.end);
            if cursor >= block.end {
                break;
            }
        }
        if cursor < block.end {
            out.push(Gap {
                from: cursor,
                to: block.end,
                elements: elements_in(block, cursor, block.end),
            });
        }
    }
    out
}

struct Gap {
    from: i64,
    to: i64,
    elements: String,
}

/// The block's own spans that actually reach into `[from, to)`, in time order and
/// comma-joined — a string rather than an array, since `crate::text`'s interpolation
/// prints a string bare and would otherwise fall back to compact JSON for an array.
fn elements_in(block: &Block, from: i64, to: i64) -> String {
    block
        .spans
        .iter()
        .filter(|span| span.start < to && span.end > from)
        .map(|span| span.element.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn finding(group: &str, gap: Gap, active: &str, missing: &str) -> Finding {
    Finding::at_class("R-VISUAL-GAP", class_for(active))
        .field("group", json!(group))
        .field("from", json!(gap.from))
        .field("to", json!(gap.to))
        .field("size", json!(gap.to - gap.from))
        .field("active", json!(active))
        .field("missing", json!(missing))
        .field("elements", json!(gap.elements))
}

/// `review` for audio outrunning its group's visual — ADR-0018's own two founding
/// defects, and the fixture's own zero — `note` for the reverse, which is the ordinary
/// silence `N-TRACK-GAP` already reports as one. See the module doc for the evidence.
fn class_for(active: &str) -> Class {
    match active {
        "audio" => Class::Review,
        _ => Class::Note,
    }
}
