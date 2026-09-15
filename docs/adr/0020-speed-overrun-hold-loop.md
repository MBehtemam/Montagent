---
status: accepted
amends: 0005 (settles the `speed`/`fill` fields it named but left undefined)
---

# `speed` is a rate multiplier, `fill` is renamed `overrun`, and the two compose

[ADR-0005](./0005-absolute-integer-milliseconds.md) required a time-based
element to declare why its timeline range differs from its source range, and
named the mechanism — `speed`, or an explicit `fill` of `hold`/`loop` — without
settling either. The gap stopped being hypothetical once
[#9](https://github.com/MBehtemam/Montaget/issues/9)'s fixture shipped with
`speed: 0.645` on four narration elements (`vo-sentence-05-b` through `08-b`):
every sentence in the published video is spoken twice, once at normal rate and
once slowed for a language learner, and nothing said what `0.645` meant, what
values were legal, or what the field the ADR called `fill` was actually named
— `fill` had since been claimed as the shape-paint field
([ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)).

Settled by a three-juror panel (Opus, Haiku, Fable), each briefed as an agent
authoring real projects and shown the committed fixture rather than an
abstract spec.

## `speed`

A playback-rate **multiplier**: `0.645` plays the source at 0.645× normal
rate, i.e. slower — matching the fixture and every NLE/`ffmpeg`
(`atempo`/`setpts`) convention. The reciprocal reading ("stretch factor",
where `1.55` would mean the same thing) was rejected unanimously: it inverts
the one convention every comparable tool already agrees on, for no offsetting
benefit.

`speed` must be strictly greater than zero. `speed: 0` is a schema error — it
would name an infinite or zero-width hold, which is what `hold` already
expresses, not a rate. Negative `speed` is also a schema error: reverse
playback is a real need for a general-purpose editor
([ADR-0003](./0003-general-video-editor-not-channel-tooling.md)'s guard says
the fixture proving `speed` is needed says nothing about reverse being
unneeded), but it does not belong as a sign bit on `speed` — it would force
`abs()` into the invariant below and overload one scalar with two orthogonal
decisions. Reverse is deferred to a future explicit field (`reverse: true` or
similar); this ADR does not decide its shape and it stays out of scope, not
silently ruled out.

## The invariant, with tolerance

ADR-0005's internal-consistency check, `end - start == source_end -
source_start`, is the `speed == 1` special case. Restated in general:

```
end - start == round((source_end - source_start) / speed)
```

rounding to the nearest integer millisecond (round-half-up). This is forced
by the fixture's own numbers, not a preference: `vo-sentence-05-b` has
`source_start: 0, source_end: 2184, speed: 0.645`, and `2184 / 0.645 =
3386.0465...`, which rounds to `3386` — exactly its committed `end - start`
(`16558 - 13172`). Exact, unrounded equality is impossible under integer
milliseconds and is falsified by the fixture; an epsilon layered *on top of*
the rounding was considered and rejected, because it would let two files
expressing the same intent both validate with different numbers — the kind of
invisible slack this format has rejected every other time it was proposed
(frame-alignment in ADR-0005, `fit` tolerance in ADR-0013). After rounding,
equality is exact.

**Authority**: `start`/`end` and `source_start`/`source_end` are authoritative
and must never move silently to satisfy this check — the timeline span is
inter-element (adjacency, gaps, overlaps with siblings) and the source range
names real bytes in a real file. `speed` is the free variable: when the
invariant fails, `validate`'s error reports the `speed` value that would
satisfy it (`(source_end - source_start) / (end - start)`), and the agent
edits `speed`, never the spans.

## `overrun`, not `fill`

The selector for what a timeline span does past the end of its (possibly
speed-adjusted) source is named **`overrun`** — `fill` is spent
([ADR-0014](./0014-stroke-is-paint-the-text-box-is-required.md)). Legal values
are exactly `"hold"` and `"loop"`. There is no third value and no `"none"`:
the field's presence alone signals the condition, so an explicit "no overrun"
value would be indistinguishable from omission — the same optional-field-that-
can-silently-mean-nothing shape [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)
already rejected once.

**`hold`**: on a video (or any decodable frame source), the last in-range
frame — at or before `source_end` — freezes and repeats for the remainder of
the timeline span. Ordinary freeze-frame; the renderer never decodes past the
declared source range. On **audio**, `hold` is a schema error: there is no
non-arbitrary meaning for holding the last sample (a sustained tone, not
silence), and "then silence" is already free — the element simply ends
earlier and leaves a gap, which ADR-0005 already made legal.

**`loop`**: the source restarts from `source_start` with a hard cut at the
seam — no crossfade. A crossfade would need an unstated duration and curve,
the exact renderer-invented-value pattern this format has rejected repeatedly
(no auto-wrap, no auto-fit-to-box, no implicit anything). `loop` is legal for
both video and audio — a looping ambience bed or music cue under a scene is
at least as ordinary a need as a looping video texture. If the loop doesn't
divide evenly into the remaining span, the last iteration truncates at `end`.

## `speed` and `overrun` compose

They are **not mutually exclusive**. An element may declare both — a clip
slowed to 0.645× that still doesn't fill its timeline slot needs to hold or
loop past the now-stretched duration, and that is an ordinary edit
(slow-motion B-roll under a long narration line), not a hypothetical. Forcing
exclusivity would push an author to fake it by fudging `speed` away from the
rate they actually want, corrupting the one field the invariant above
designates as the repair site.

They compose in one fixed order: `speed` is applied to the source span first,
producing the as-played duration per the invariant above; `overrun`, if
declared, covers whatever timeline span remains past that duration. The
invariant generalizes: without `overrun`, `end - start` must equal
`round(source_span / speed)` exactly; with `overrun` declared, `end - start`
must be strictly *greater than* that value. An `overrun` declared on an
element that doesn't need it — where `end - start` doesn't exceed the played
duration — is itself a `validate` error.

## Consequences

- The schema replaces `fill` with `overrun` wherever ADR-0005 referenced it.
- `validate`'s speed-mismatch error must compute and print the corrective
  `speed`, not just flag the mismatch — the earlier ADRs' pattern of stating
  facts an agent can act on without a second lookup
  ([ADR-0011](./0011-tool-surface-reads-checks-renders.md)).
- Reverse playback is explicitly undecided, not ruled out — a future ticket,
  not silently closed by this one.
