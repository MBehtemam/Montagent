//! **Slack**: the distance from a boundary to its nearest neighbouring boundary in any
//! track, or to `duration` — derived once, here, for the two verbs that both consume it.
//!
//! `CONTEXT.md` defines it, and ADR-0032 is why it has a name at all: four agents given
//! the same instruction produced four different repairs, all internally legal, differing
//! by up to 920 ms of trailing silence, because *"the file distinguishes elements from
//! each other but says nothing about the space between them"*. Naming that space makes
//! it content: **every slack in the file is invariant by default**, `shift` refuses an
//! edit that would change one, and `compare` reports one that changed under a raw edit.
//!
//! It is owned here rather than by either of those verbs because there are two of them.
//! Spec #168 names the hazard directly — slack is *"a derived quantity that stories 68
//! and 72 both consume and neither defines"* — and two definitions of a distance is the
//! same drift [`crate::stack`] and [`crate::layout`] are each written against.
//!
//! # A slack is not a gap
//!
//! A **gap** is a stretch of one track with no element in it. A slack is the distance
//! between two *boundary instants*, wherever they sit: *"every gap is slack; slack
//! additionally names the cross-track case a gap can't reach, such as the distance from
//! the last narration's end to a still photo's end"* (`CONTEXT.md`). On the committed
//! fixture the smallest slack is **4 ms**, between `56112` and `56116` — two boundaries
//! in two different tracks, which no gap in any track reaches. The gap check
//! ([`crate::checks::track`]) answers the other question and shares nothing with this.
//!
//! # The shape, and why it is a pair
//!
//! ADR-0047 settles that a slack is named by its **full boundary-instant pair**, never by
//! the single moving instant: a bare timestamp *"is contextually unambiguous only within
//! the specific call that produced it"*, and a boundary that is simultaneously the far
//! edge of one slack and the moving edge of the next is *"the ordinary shape of a tightly
//! packed timeline"*. So the pair is the identity, and `shift --release` takes back
//! exactly the pairs a refusal printed.
//!
//! Which means the derivation is the one below and not a per-boundary search: sort every
//! distinct boundary instant the document states, and each adjacent pair is one slack.
//! A boundary's nearest neighbour forwards is the next distinct instant and backwards is
//! the previous one, so the adjacent-pair list *is* the set of distances the definition
//! describes, with each stated once instead of twice.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::permissive::Loose;

/// The distance between two neighbouring boundary instants, and what sits on each end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slack<'a> {
    /// The earlier of the two boundary instants.
    pub from: i64,
    /// The later. `from`/`to` together are ADR-0047's identifying pair.
    pub to: i64,
    /// Every element boundary sitting at `from`, in document order.
    ///
    /// Named for the instant and not for a direction: a boundary at `from` is as often an
    /// element's `end` as its `start`, so `opens`/`closes` would claim a direction the
    /// [`Edge::side`] beside it contradicts.
    pub from_edges: Vec<Edge<'a>>,
    /// Every element boundary sitting at `to`, in document order.
    pub to_edges: Vec<Edge<'a>>,
    /// Whether `to` is the project's own last boundary — its `duration` — rather than an
    /// element's. `CONTEXT.md` names that case explicitly, and it is the one slack with
    /// nothing on its far end to point at.
    pub to_duration: bool,
}

impl Slack<'_> {
    /// How much slack there is, in milliseconds. Always positive: the instants are
    /// distinct and ordered by construction.
    pub fn size(&self) -> i64 {
        self.to - self.from
    }
}

/// One element's boundary, as a slack's end names it.
///
/// It carries the element and its track because `shift`'s refusal says both — ADR-0032's
/// worked example is *"1120–2040 after item-07's new end would change the 913 ms lead-out
/// to photo-07's end"* — while the slack's *identity* is the instant pair alone
/// (ADR-0047). The names are how a reader finds the boundary; the pair is how a call names
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge<'a> {
    pub element: &'a str,
    pub track: Option<&'a str>,
    /// Which of the element's two boundaries this is.
    pub side: Side,
}

/// Which boundary of an element an [`Edge`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Start,
    End,
}

/// **The single exported derivation.** Every slack in the document, earliest first.
///
/// `shift` (#220) and `compare` (#221) both call this rather than each computing a
/// distance, which is the whole reason it is a function and not a paragraph in two
/// tickets.
///
/// An element that does not state **both** an integer `start` and an integer `end`
/// contributes no boundary at all: a document mid-edit is exactly what this runs on, and a
/// slack measured to a boundary that is not there is a number nothing measured (ADR-0006).
pub fn of<'a>(document: &'a Loose) -> Vec<Slack<'a>> {
    // Keyed by instant, so two elements cutting at the same number are one boundary and
    // not two — a cut *"names a single instant, not an overlap and not a gap"*
    // (`CONTEXT.md`), and a zero-width slack between an end and the start it meets is not
    // a distance anything could release.
    let mut boundaries: BTreeMap<i64, Vec<Edge<'a>>> = BTreeMap::new();

    // The shared traversal, because slack is cross-track by definition and needs no
    // grouping: which track a boundary came from is a name it carries, never a bucket it
    // sits in. (`crate::track` is the other walk, and it needs the grouping — an overlap
    // is a fact about one track's own children.)
    for (name, element) in document.elements_in_tracks() {
        let Some(id) = element.get("id").and_then(Value::as_str) else {
            continue;
        };
        // **Both or neither.** An element mid-edit carrying a `start` and no `end` states
        // no range, and one boundary of a range that is not there would manufacture a
        // slack out of a half-written element — the drift spec #168 names this derivation
        // to prevent, arriving from inside it. `crate::track` holds the same rule for the
        // same reason.
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            continue;
        };
        for (side, at) in [(Side::Start, start), (Side::End, end)] {
            boundaries.entry(at).or_default().push(Edge {
                element: id,
                track: name,
                side,
            });
        }
    }

    // The project's own last boundary. `CONTEXT.md` calls it *"the derived `duration`"*,
    // and where the document states one it is the number the render ends at — so the
    // stated one is used and the derived maximum end is already in the set beside it.
    // Where the two disagree, both boundaries are real and the slack between them is the
    // trailing black frames; whether the disagreement is itself a finding belongs to the
    // check that owns `duration`, not to a derivation of distances.
    let duration = document
        .value()
        .get("duration")
        .and_then(Value::as_i64)
        .or_else(|| boundaries.keys().next_back().copied());
    if let Some(duration) = duration {
        boundaries.entry(duration).or_default();
    }

    let instants: Vec<i64> = boundaries.keys().copied().collect();
    instants
        .windows(2)
        .map(|pair| Slack {
            from: pair[0],
            to: pair[1],
            from_edges: boundaries[&pair[0]].clone(),
            to_edges: boundaries[&pair[1]].clone(),
            to_duration: Some(pair[1]) == duration,
        })
        .collect()
}

/// The slack a pair of instants names, if the document has one.
///
/// A lookup by identity, and it lives here because the identity does: ADR-0047 settles
/// that a slack is named by its *"full boundary-instant pair, never by the single moving
/// instant"*, so asking whether a pair names a slack is a question about this derivation.
/// What to *do* when the answer is `None` is the caller's — `shift` refuses a `release`
/// entry *"naming a pair that does not currently bound a real, protected slack"* (#220),
/// and that refusal is #220's, not this module's.
pub fn at<'a>(slacks: &'a [Slack<'a>], from: i64, to: i64) -> Option<&'a Slack<'a>> {
    slacks
        .iter()
        .find(|slack| slack.from == from && slack.to == to)
}
