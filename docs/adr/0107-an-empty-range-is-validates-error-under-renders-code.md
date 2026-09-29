---
status: accepted
amends: 0093 (the empty range stops being a `render`-only finding: `validate` now states `E-EMPTY-RANGE`, so the check engine refuses before the mix and the two arms ADR-0093 gave the code become `E-INTERNAL` by that ADR's own rule for an arm the check engine closes)
---

# An empty range is `validate`'s `error`, under `render`'s code

[#410](https://github.com/MBehtemam/Montagent/issues/410), a gap ADR-0093 found and
deliberately left open rather than smuggle a new `validate` check into a reporting fix. Part
of [#383](https://github.com/MBehtemam/Montagent/issues/383).

## What was there

`"start": 0, "end": 0` — an element occupying no instant of ADR-0005's half-open clock —
validated at **zero errors**. It was not that nobody looked: five clock checks meet the case
and each steps over it on a sound local argument — `quantization` (`holds_a_sampled_frame` is
`None` on an empty span), `speed` (the invariant has no domain), `coverage`, `unreached` and
`canvas` (no instants, so nothing to say about instants). `canvas.rs` even claimed the fact
belonged to *"the schema check"*, which cannot express a cross-field comparison and does not
state it. Five correct refusals added up to a silence.

`render` then refused the same document with `E-EMPTY-RANGE` (ADR-0093), so the two verbs
disagreed about one document in exactly the shape ADR-0093 ruling 3 spent its length closing
for sources: `render` refuses, `validate` passes it clean.

## Decision

### 1. `validate` emits `E-EMPTY-RANGE` — the registered code, not a parallel one

One new check, `checks::range`, over every element: `start`..`end` and, where both are
integers, `source_start`..`source_end`. Either not advancing is one finding, with `render`'s
existing field set (`field`, `from`, `to`) plus `element`.

Reuse is admissible and a second code is not:

- **ADR-0043 fixes repair form per code**, and the form is `Refuse` on both sides — which of
  `start`, `end` or the source span the author meant to move is not in the document
  (ADR-0020). Nothing about the condition differs by verb.
- **A second code would move the disagreement rather than end it.** The two verbs would agree
  the fact exists and disagree about its name, and every consumer keying on codes
  (`registry.rs`: *"a code is the handle an author suppresses and `compare` diffs on"*)
  would see two facts. #388's test applies: this is one document fact, so it is one code.

The registry entry's `adr` moves from ADR-0093 to this ADR, because the entry's *reason for
existing* changed: it is no longer a render-reachable arm but a document check.

### 2. `error`, and the `review` reading is dismissed explicitly

The argument for `review` is real: an element with no instants draws nothing and mixes
nothing, so the output is not *missing* anything in the way a dropped element's output is.
It fails on the definition. `review` is *"legal, renders, and you must look at a frame"*
(ADR-0006), and **there is no frame at which this element could be looked at** — the
instruction the class carries is unfollowable. What the document declares is an element; what
any render of it produces is no trace of that element at any instant. That is *"guaranteed
wrong"*, which is `error`.

It is also the class of the nearest precedent: `E-TRANSITION-NO-OVERLAP` (ADR-0059) is a
transition whose window is empty, and it is refuse-class `error` for the same reason.

And `review` would reopen the ruling-3 split at a different seam: `validate` would say
*"look"* while `render` refused.

### 3. The cross-verb invariant extends — over codes, not only sources

`tests/cross_verb.rs` asserted a containment about **sources**: every source `render`
declines, `validate` does not call clean. The empty range is the same shape about a
**document fact**, so the file gains the same containment over the code: for each of the two
ranges, `validate` emits `E-EMPTY-RANGE` at `error`, `render` refuses under the same code and
publishes nothing, **and no `E-INTERNAL` appears** — the last clause is what proves the refusal
came from the check engine and not from the mix, which is the property ADR-0093 ruling 3
actually wants.

### 4. `render`'s two arms become `E-INTERNAL`

ADR-0006: `render` runs the identical check engine and refuses on any `error`. With the check
in place `render` never reaches the mix on an empty range, and ADR-0093 is explicit about arms
the check engine closes — they are `E-INTERNAL`, the *"two halves of `render` disagree about
this document"* invariant channel, not findings. Both arms now say so through
`Declined::internal`. Reaching either would be a bug in the check, which is the right thing
for it to read as.

**What this does not cover.** A `transition`'s range is derived from the two elements it
bridges (ADR-0059), so an empty one is already `E-TRANSITION-RANGE` or
`E-TRANSITION-NO-OVERLAP`; `checks::range` skips transitions so one fact is not reported under
two codes. A `highlight` window is not an element and has `E-HIGHLIGHT-RANGE`.

## Consequences

- A project carrying an empty range that used to `validate` clean now reports an `error` and
  exits accordingly. It already could not `render` since ADR-0093, so no project that
  produced a deliverable changes behaviour.
- `render`'s report on such a project is unchanged in code and fields; only where it comes
  from moves (the check engine rather than the mix), and the `audio` block no longer lists the
  element as not mixed, because the mix never ran.
- Verified: `crates/montagent-core/tests/range.rs` (the check) and the new case in
  `tests/cross_verb.rs` (both verbs, both ranges).
