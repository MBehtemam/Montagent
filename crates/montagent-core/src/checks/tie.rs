//! `E-LAYER-TIE` — ADR-0060: *"a tie is `error`, never `review`, when the two elements'
//! boxes overlap in both time and space."*
//!
//! Two elements can resolve to one integer `layer` — most commonly two tracks both
//! declaring `layer: 30` — and nothing in the format says which of them draws in front.
//! ADR-0060 settled that this is not a gap to be filled with a fallback order but a
//! document that does not say what it means: *"whichever order ships is exactly as
//! defensible as its opposite,"* and two independent tools reading the committed fixture
//! have already invented two different ones. So the render is refused until the author
//! states an order, and **array order is never consulted** — not here, and not anywhere.
//!
//! **A tie whose boxes never meet is not a finding.** By construction nothing on screen
//! depends on their order, so there is nothing to state and nothing to refuse. That is why
//! this check measures geometry rather than counting ties.
//!
//! ## Why this check samples, and what it samples
//!
//! ADR-0060 rejected a static-extent check explicitly, in both directions: *"two same-layer
//! elements can be spatially disjoint at rest and swept into overlap mid-animation by a pan
//! or zoom, or the reverse."* Paired with an `error` verdict, a rest-position check would
//! either miss a real collision or refuse a project whose elements never actually
//! co-occupy space — *"both failure modes unacceptable once the check gates the render."*
//!
//! So the sample set is the ADR's own: **each keyframe boundary plus an interval between
//! them**, across the pair's temporal intersection only. [`crate::checks::box_samples`]
//! builds it from every keyframe either element carries on a property that moves its box —
//! `x`, `y`, `scale`, `rotation` — clamped to the shared window, with the window's own
//! first and last instants at the ends and the midpoint of every consecutive pair between
//! them. Between two boundaries each box travels monotonically along a single eased
//! segment, so one interior sample is what catches a crossing that begins and ends apart;
//! it is a sample set and not a proof, which is the trade ADR-0060 made when it chose
//! sampling over a static check. It is shared with [`crate::checks::canvas`], which asks
//! the same arithmetic a different question — *does this box ever meet the frame* — and a
//! second copy would be a second answer to which instants are worth looking at.
//!
//! **Scope stays narrow** (ADR-0060): only pairs that already share a resolved layer are
//! sampled, and only across the instants they are both on screen — never a pairwise scan of
//! the project.
//!
//! ## What it does not decide
//!
//! *Paint.* ADR-0060: the check *"needs no paint/effect awareness (opacity, fill colour,
//! occlusion) to be correct at `error` level — it is purely geometric. A same-layer pair
//! whose boxes overlap but one is fully transparent is still `error`."* What it does honour
//! is `clip`, because that is geometry and not paint: the static aperture is the frame-space
//! rectangle the element is drawn *through*, and the part of a box outside it never reaches
//! the screen to collide with anything.
//!
//! *Rotation.* A rotated element's footprint is a parallelogram and this check measures
//! rectangles ([`geometry::NotAxisAligned`], the refusal `query --at` already makes for the
//! same reason). Its bounding box would refuse legal projects; silence would pass a tie
//! nobody looked at. `U-LAYER-TIE-ROTATED` is the third answer — ADR-0006's `unchecked`,
//! which is counted and never a verdict. The refusal is per pair and is not narrowed: a
//! disjoint bounding box would *prove* no collision and could stay silent, but computing
//! one is rotated-footprint geometry [`crate::verbs::query::geometry`] declines to own.
//! Both that and the code itself are surface no ADR ratifies — raised as
//! [#282](https://github.com/MBehtemam/Montagent/issues/282).
//!
//! *Resolution.* Which layer an element is on is [`crate::stack`]'s answer, asked here and
//! never re-derived — the same index [`crate::checks::anchor`] and the rasterizer read, so
//! the tie this refuses is the tie the renderer would have had to break.
//!
//! **Named `tie` and not `layer`.** `crate::stack`'s module doc already records the
//! near-miss: a `layer.rs` sits one letter from `layout`, which is a different subject
//! entirely — and in *this* directory `layout.rs` is a live neighbour, the canonical
//! key-order check. The tie is what this module is about; the layer is only where it
//! looks.
//!
//! **Document-only**: every field it needs is in the document, so it needs no session and
//! can sit anywhere in `validate`'s check list.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::permissive::Loose;
use crate::report::Report;
use crate::stack::{Stack, TimelineRange};
use crate::verbs::query::at::frame_dimensions;
use crate::verbs::query::geometry::{self, NotAxisAligned, Rect};

/// One element, as this check reads it.
struct Tied<'a> {
    id: &'a str,
    /// The name of the track it sits in, where the track has one. Carried because the
    /// layer usually comes *from* the track — two tracks both declaring `layer: 30` is
    /// ADR-0060's own most common case — so the two names are often where the author's
    /// cheapest fix is, and a finding that named only the elements would send them
    /// looking for it.
    track: Option<&'a str>,
    /// Which track it is in, by position in the `tracks` array. Identity, not order: two
    /// tracks may share a `name` mid-edit, or carry none at all, and *"are these two
    /// elements in one track"* has to be answerable on a document where neither is
    /// nameable.
    track_index: usize,
    element: &'a Value,
    range: TimelineRange,
}

/// `E-LAYER-TIE`/`U-LAYER-TIE-ROTATED`, over every pair of elements sharing a resolved
/// layer.
pub fn check(document: &Loose, report: &mut Report) {
    // Without a legal `frame` there is no frame space to place a box in — `x`/`y` default
    // to its centre (ADR-0012), and a check that invented a size would be measuring a
    // collision in a coordinate system the document does not have. The schema check owns
    // saying so.
    let Some(frame) = frame_dimensions(document) else {
        return;
    };
    let stack = Stack::of(document);

    // Grouped by the layer each element resolves to, never by the order they are written
    // in (ADR-0060). A `BTreeMap` keyed on the layer and members sorted by `id` inside it
    // make the whole traversal — and so the order findings reach the report — a function
    // of the document's content and not of its array order.
    let mut by_layer: BTreeMap<i64, Vec<Tied<'_>>> = BTreeMap::new();
    for (track_index, track) in document
        .value()
        .get("tracks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let name = track.get("name").and_then(Value::as_str);
        for element in track
            .get("elements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(id) = element.get("id").and_then(Value::as_str) else {
                continue;
            };
            if !geometry::covers_the_frame(element.get("type").and_then(Value::as_str)) {
                continue;
            }
            let Ok(layer) = stack.layer_of(id) else {
                // A layer that does not resolve is the anchor check's finding and the
                // schema check's, already reported by whichever owns it. An element with
                // no place in the stack is in no tie.
                continue;
            };
            let Some(range) = stack.placement(id).and_then(|placement| placement.range) else {
                continue;
            };
            by_layer.entry(layer).or_default().push(Tied {
                id,
                track: name,
                track_index,
                element,
                range,
            });
        }
    }

    for (layer, mut tied) in by_layer {
        tied.sort_by_key(|t| t.id);
        for (i, a) in tied.iter().enumerate() {
            for b in &tied[i + 1..] {
                // Two elements of one track sharing an instant is `E-TRACK-OVERLAP` —
                // the rule tracks exist to enforce (ADR-0004), and already an `error`.
                // Their tie is a consequence of that overlap and not a second defect: the
                // fix is to move one of them to another track, never to state a stacking
                // order between two elements that may not coexist at all. Reported here
                // too, it would be two errors about one line, which is the noise budget
                // ADR-0006 calls a safety property. On the same reasoning
                // `crate::checks::anchor` leaves the schema's facts to the schema check.
                // ADR-0060 states no such exception; the silence is this check's own
                // reading and is raised as #282.
                if a.track_index == b.track_index {
                    continue;
                }
                if let Some(finding) = collision(a, b, layer, frame) {
                    let finding = finding.at_file(document.path());
                    // No `at_track`: every pair that reaches here spans two tracks, and
                    // a finding located at either one would be making the arbitrary choice
                    // between them that the whole check exists to refuse. ADR-0006
                    // contemplates no finding about a *pair*, so this too is #282.
                    report.push(finding);
                }
            }
        }
    }
}

/// This pair's finding: the first sampled instant at which their boxes overlap, or — if
/// no sample proved one — the first at which one of them rotates and the question stops
/// being answerable in rectangles. `None` where they are never both on screen, or where
/// every sample found them apart with neither rotating.
///
/// **The overlap wins over the refusal, whichever came first on the clock.** A rotation at
/// one instant says nothing about the next: an element spun at the start of the window and
/// axis-aligned on top of its neighbour by the end is an `error` the document really does
/// contain, and returning `unchecked` at the first rotated sample would hide it behind an
/// *"I could not look"* about a different instant. So every sample is taken either way, and
/// the refusal is what is left when none of them found a collision. ADR-0060: *"sample both
/// boxes across their temporal intersection …; flag if any sample overlaps."*
///
/// One finding per pair, not per sample. The tie is a single fact about the two elements
/// — the author states an order once and every instant of it is fixed — and a finding per
/// overlapping instant would be the noise ADR-0006 calls a safety property.
fn collision(a: &Tied<'_>, b: &Tied<'_>, layer: i64, frame: (i64, i64)) -> Option<Finding> {
    let window = shared(a.range, b.range)?;
    let mut refused: Option<Finding> = None;

    for instant in crate::checks::box_samples(&[a.element, b.element], window) {
        let (one, other) = (
            geometry::visible_rect(a.element, instant, frame),
            geometry::visible_rect(b.element, instant, frame),
        );
        for (tied, rect) in [(a, one), (b, other)] {
            if let Some(Err(NotAxisAligned::Rotated(degrees))) = rect {
                // The earliest rotated instant, kept rather than returned: it is the
                // answer only if nothing later turns out to be a collision.
                refused.get_or_insert_with(|| rotated(a, b, layer, tied.id, degrees, instant));
            }
        }
        let (Some(Ok(one)), Some(Ok(other))) = (one, other) else {
            // One of them has no rectangle at this instant: it rotates (above), carries no
            // readable `width`/`height` (the schema check's finding, not this one's), or
            // has a `clip` that excludes it entirely. In the last two it claims no pixels
            // here and collides with nothing.
            continue;
        };
        if let Some(overlap) = one.intersect(other) {
            return Some(tie(a, b, layer, instant, overlap));
        }
    }
    refused
}

/// The instants both elements are on screen, as a range this check can sample — `None`
/// where they share none.
///
/// Half-open (ADR-0005), so an element ending at 7500 and one starting there share no
/// instant, and an empty or inverted range shares none with anything.
fn shared(a: TimelineRange, b: TimelineRange) -> Option<TimelineRange> {
    if !a.overlaps(b) {
        return None;
    }
    Some(TimelineRange {
        start: a.start.max(b.start),
        end: a.end.min(b.end),
    })
}

/// `E-LAYER-TIE`: the two boxes overlap, and the document does not say which draws in
/// front.
///
/// Every number is inline (ADR-0006): the layer they share, the instant the overlap was
/// found at, and the overlapping rectangle itself, so that judging the finding costs no
/// re-read of the project.
fn tie(a: &Tied<'_>, b: &Tied<'_>, layer: i64, instant: i64, overlap: Rect) -> Finding {
    tracks_of(Finding::new("E-LAYER-TIE"), a, b)
        // Located at one of the two, because a finding is located at one place — and at
        // the lexicographically first, which is the pair sorted by content rather than by
        // the array order ADR-0060 keeps meaningless. Both are named in the fields.
        .at_element(a.id)
        .field("element", json!(a.id))
        .field("other", json!(b.id))
        .field("layer", json!(layer))
        .field("instant", json!(instant))
        .field("overlap_x", json!(overlap.x))
        .field("overlap_y", json!(overlap.y))
        .field("overlap_width", json!(overlap.width))
        .field("overlap_height", json!(overlap.height))
}

/// `U-LAYER-TIE-ROTATED`: one of the pair rotates, so whether the boxes overlap is a
/// question about parallelograms and this check measures rectangles.
///
/// `unchecked` rather than `error` or silence — ADR-0006's category for *"the check could
/// not run"*, counted in the summary and never a verdict.
fn rotated(
    a: &Tied<'_>,
    b: &Tied<'_>,
    layer: i64,
    rotated_id: &str,
    degrees: f64,
    instant: i64,
) -> Finding {
    tracks_of(Finding::new("U-LAYER-TIE-ROTATED"), a, b)
        .at_element(a.id)
        .field("element", json!(a.id))
        .field("other", json!(b.id))
        .field("layer", json!(layer))
        .field("rotated", json!(rotated_id))
        .field("degrees", json!(degrees))
        .field("instant", json!(instant))
}

/// The two tracks the pair sits in, as fields, where each has a name.
///
/// Absent rather than spelled with a placeholder when a track carries no `name`: a
/// finding states measured facts (ADR-0006), and "an unnamed track" is a second spelling
/// of one that would then have to be kept in step with every other check's.
fn tracks_of<'a>(finding: Finding, a: &Tied<'a>, b: &Tied<'a>) -> Finding {
    [("track", a.track), ("other_track", b.track)]
        .into_iter()
        .fold(finding, |finding, (key, name)| match name {
            Some(name) => finding.field(key, json!(name)),
            None => finding,
        })
}
