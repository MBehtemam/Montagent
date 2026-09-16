---
status: accepted
amends: 0005 (delivers the nearest-boundary message this ADR commissioned, for the coincident case), 0006 (declines a coincidence census, and states why), 0011 ("informative preamble on legal inserts" is specified), 0012 (the preamble covers keyframe records, not just element boundaries)
---

# `shift` prints what it will do to every record at a coincident `at`; `validate` gets no coincidence census

[Ticket #69](https://github.com/MBehtemam/Montaget/issues/69), graduated from
[#12](https://github.com/MBehtemam/Montaget/issues/12). This is what remained after
*"`shift` silently desyncs a dependent animation"* was measured and substantially
dissolved: the relationship between an event that ends and a move that starts survives
`shift` at every value of `at` except the exact shared instant, where the ambiguity is
the edit's actual meaning — *"insert 1000ms here"* legitimately means either "the gap
goes before the move" or "the move stays glued through it," and both are legal, both
differently shaped. `shift` already picks one reading, per its own published rules, and
says nothing.

Resolved by a jury of three independent models (Opus, Sonnet, Haiku) on three
sub-questions, unanimous 9/9.

## `shift` prints a coincident-instant preamble, unconditionally

Adopted as the ticket proposed it, with no flag and no separate call. Before executing
a shift whose `at` coincides with one or more existing instants — an element boundary or
a keyframe `t`, per [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)
— `shift` enumerates every record sitting exactly at `at`, its role (first / last /
interior), and what its own published rules (this ADR's amendment to
[ADR-0005](./0005-absolute-integer-milliseconds.md), and the SPLIT rule in
[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)) will do to it —
e.g. *"23122 is the final opacity record of `sentence-06` (stays) and the first scale
record of `photo-06` (moves); after this shift the ramp begins at 24122 and the fade
still completes at 23122."*

**Why unconditional, not gated.** The hazard this closes — from
[ADR-0011](./0011-tool-surface-reads-checks-renders.md), *"the nearest-boundary message
is needed as an informative preamble on legal inserts too"* and *"`at` on an exact
boundary is the normal case and an off-by-one there is silent"* — is precisely that the
dangerous call and the routine call are indistinguishable **from the author's side**.
Any gate (a flag, a `--preview` mode, a separate tool call) is only reachable by an
author who already suspects the ambiguity the preamble exists to reveal — a circular
precondition, and the same silence ADR-0011 already named. Folding it into `shift`'s own
output also holds the line [ADR-0011](./0011-tool-surface-reads-checks-renders.md)
already drew: new outputs fold into existing tools; this is not a tenth verb.

**Why this doesn't reopen the noise question.** The trigger is not "every shift" — it is
already conditional on `at` landing on a coincident instant, and the content is pure
arithmetic over the document plus already-published semantics: no intent inferred,
nothing that can be wrong in the way a heuristic filter could be. A narrower trigger —
firing only when the coincidence is "meaningfully" ambiguous — was considered and
rejected: deciding which coincidences are meaningful requires asserting intent, exactly
the line the ticket declines to cross, and "coincident" already **is** the ambiguity
predicate measured in the ticket (identical outside that one instant, at every other
value of `at` tested). Measured cost: mean 5.21 lines at a coincident instant, max 12 (at
the project start, where the most records share an instant), 1 line at a typical
within-shot `at` — a receipt roughly the size of the edit it describes, not a flood.

## `validate` gets no coincidence census

Rejected outright, not deferred. The ticket's own measurement shows why: 120 boundary
instants (element-only) or 134 (with keyframes), of which 92 / 99 coincide — 76.7–73.9%.
Partitioning all 305 coincident pairs: 117 same-track cuts, 136 same-group co-boundaries,
7 self-keyframe-on-own-boundary, and 45 cross-element pairs, every one of the 45
derivative of those same 7. The shape a census would exist to catch — an independent,
non-derivative cross-element coincidence not explained by track or group structure —
occurs **zero times**. A census would report 0/305 as interesting on the one real
project measured: coincidence is the ordinary shape of a cut or a group boundary, not a
signal.

**The deeper reason, not just the rate.** *"A stateless census can report that a
coincidence exists and can never report that one was destroyed"* — and destruction, not
presence, is the actual hazard this ticket is about. Detecting it requires a before and
an after; `validate` has neither, running once against one document, the same no-I/O
budget every check in [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)
holds to. `compare` (ADR-0032's drift backstop) has both states already — a coincidence
that held in one version and doesn't in the next is a natural, rare, actionable `compare`
finding, not a `validate` one. Building a census into `validate` anyway would repeat the
exact failure [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) already
rejected once for frame-alignment reporting: a near-100%-harmless-fire check manufactures
the false confidence the ADR names as the real cost of noise, and does so while missing
the hazard entirely.

A narrower census — surfacing only the 7 self-keyframe root cases instead of all 305
pairs — was considered and rejected: those 7 are the *most* structurally benign category
(a keyframe on the boundary of its own element is the definition of correct authoring),
so narrowing inverts the filter rather than sharpening it, and it is still stateless and
still blind to destruction.

**Disposition of the destruction hazard.** Not solved here. It is a `compare` feature —
detecting that a coincidence present in one version of a project is absent in the next —
and is out of this ticket's scope.

## What remains unresolved, and where it goes

**"Inert provenance"** — a renderer-ignored derivation-claim field beside a literal
timestamp, structurally [ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md)'s
`fit` transplanted to the time axis, considered as a possible way to make an author's
intended timestamp relationship legible to `validate` without a live, executable
reference — was raised and left open. It has no sharp question yet: the design's own
shape is undetermined (it can name "same instant as this element's edge" but not
open-ended relationships like "the end of the fade" without asserting intent it doesn't
have, and it records a start instant but not whether the intended relationship is
same-rate or same-endpoint, so it is half a binding even in its own terms), and the
authoring shape it would protect occurs zero times in the one fixture measured — priced,
per this map's standing rule, as *evidence for pricing the need, never evidence of
unneed*. That is squarely the wayfinding fog criterion: in-scope, not yet phraseable as
an answerable ticket. It graduates to the map's **Not yet specified**, named by the
**hazard** — authoring-time drift between two timestamps an author intends to stay
related, with nothing in the document recording that intent — rather than by this
rejected candidate mechanism, so a future session starts from the problem instead of
re-deriving the same half-binding objection against one already-weak shape. A live,
persisted time-anchor reference stays rejected (3/3, recorded in the ticket, not
reopened here): it breaks read-by-reading, degrades to a stale literal the first time
anything is shifted, and gives one element's rendered geometry a silent second author.

## Consequences

- `shift` gains an unconditional preamble on any invocation where `at` coincides with an
  existing element boundary or keyframe `t`. No new tool, no new flag.
- No `validate` check for coincident instants, typed or otherwise.
- The destroyed-coincidence hazard is out of scope here; a future `compare` capability is
  the correct home, not ticketed by this ADR.
- The map's **Not yet specified** gains one fog entry: unrecorded intended-drift between
  related timestamps, blocked on frequency evidence from a wider fixture corpus.
- No schema change. No renderer change.
