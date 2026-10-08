//! A track's elements in **time order**, and the stretches between them.
//!
//! A track supplies stacking, never timing — it *"has no start, no duration and no
//! clock"* and *"the order they appear in carries no meaning"* (`CONTEXT.md`, ADR-0004,
//! ADR-0060). So "what does this track look like along the clock" is a derivation, not a
//! read, and this module is where it is derived.
//!
//! It is owned here rather than inside the check that first wanted it, for
//! [`crate::stack`]'s reason and in [`crate::stack`]'s words: there are two callers — the
//! overlap-and-gap check asks *do two of these meet, and is there a stretch with nothing
//! in it*, and the quantization check asks *does a sampled frame fall inside each of
//! them* — and a traversal with two implementations is the drift this project has found
//! four times. [`crate::slack`] is a third quantity over the same boundaries and
//! deliberately does **not** come through here: it is cross-track by definition and needs
//! no grouping at all.

use serde_json::Value;

use crate::permissive::Loose;

/// One element, reduced to what a timing question needs: a name and a half-open range.
#[derive(Debug, Clone)]
pub struct Span {
    pub element: String,
    pub start: i64,
    pub end: i64,
}

/// One track's elements, earliest first.
#[derive(Debug, Clone)]
pub struct Sequence {
    /// The name the document writes, where it writes one.
    pub track: Option<String>,
    pub spans: Vec<Span>,
}

impl Sequence {
    /// What a finding calls this track. Named here rather than at each call site, so the
    /// unnamed case cannot come to mean two things — [`crate::checks::subject_of`]'s
    /// reason, for the other half of a finding's location.
    pub fn name(&self) -> String {
        self.track
            .clone()
            .unwrap_or_else(|| "a track carrying no `name`".to_string())
    }

    /// Every stretch of this track with no element in it.
    pub fn gaps(&self) -> Vec<Gap> {
        let mut gaps = Vec::new();
        let Some(first) = self.spans.first() else {
            return gaps;
        };
        // The furthest instant the track has reached, and what reached it. Tracked rather
        // than read off the previous element, so an element nested inside a longer one
        // cannot manufacture a gap that is not there.
        let mut covered_to = first.end;
        let mut covered_by = first.element.clone();

        for span in &self.spans[1..] {
            if span.start > covered_to {
                gaps.push(Gap {
                    track: self.name(),
                    from: covered_to,
                    to: span.start,
                    after: covered_by.clone(),
                    before: span.element.clone(),
                });
            }
            if span.end > covered_to {
                covered_to = span.end;
                covered_by = span.element.clone();
            }
        }
        gaps
    }
}

/// A stretch of one track with no element in it.
///
/// `CONTEXT.md`'s **Gap**, and its bounds are two elements *of this track*: a track
/// holding one element inside a longer project has not had black frames punched into it.
/// The distance from that element's end to the project's own last boundary is
/// [`crate::slack`], which is a different quantity with a different owner.
#[derive(Debug, Clone)]
pub struct Gap {
    pub track: String,
    pub from: i64,
    pub to: i64,
    /// The element the gap opens after — the one reaching furthest into the track so far,
    /// which is not always the one written last.
    pub after: String,
    pub before: String,
}

impl Gap {
    pub fn size(&self) -> i64 {
        self.to - self.from
    }
}

/// Every track's elements, in time order.
///
/// The two-level walk rather than [`Loose::elements_in_tracks`], because this is the one
/// question that needs the *grouping* and not only the name: two tracks may be written
/// with the same `name`, and elements of one track may not overlap elements of *that
/// track* — grouping by the name would merge two legal tracks into one illegal sequence.
///
/// An element stating no integer `start` and `end`, or a range that is not a positive
/// half-open interval, contributes nothing: those are schema facts and belong to the check
/// that owns the schema, and an overlap that cannot be computed is never reported as an
/// overlap that is not there.
pub fn sequences(document: &Loose) -> Vec<Sequence> {
    let mut out = Vec::new();
    collect(document.value().get("tracks"), &mut out);
    out
}

/// One level of tracks, and then the tracks of every nest in them (#780). A nest is an
/// element of its parent track and counts by its window; its own tracks are tracks.
fn collect(tracks: Option<&Value>, out: &mut Vec<Sequence>) {
    for track in tracks
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let mut spans: Vec<Span> = track
            .get("elements")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .filter_map(|element| {
                let start = element.get("start")?.as_i64()?;
                let end = element.get("end")?.as_i64()?;
                (end > start).then_some(Span {
                    element: crate::checks::subject_of(element.get("id").and_then(Value::as_str)),
                    start,
                    end,
                })
            })
            .collect();
        // Ties broken by `end` and then by the name, so a report is byte-identical
        // across runs — nothing downstream may depend on which check spoke first, and
        // that has to include which instance of one check did.
        spans.sort_by(|a, b| (a.start, a.end, &a.element).cmp(&(b.start, b.end, &b.element)));
        out.push(Sequence {
            track: track
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string),
            spans,
        });
        for element in track
            .get("elements")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            if crate::nest::is_nest(element) {
                collect(element.get("tracks"), out);
            }
        }
    }
}
