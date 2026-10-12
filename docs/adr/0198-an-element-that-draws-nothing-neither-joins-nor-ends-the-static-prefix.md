---
status: accepted
amends: 0195 (§1: elements that draw nothing are passed over by the prefix and kept out of its key)
---

# An element that draws nothing neither joins nor ends the static prefix

[#974](https://github.com/MBehtemam/Montagent/issues/974).

## Context

ADR-0195 §1 caches "the contiguous bottom run of static images, text, rectangles, ellipses
and paths". The run was computed over the whole resolved stack, which also holds `audio` and
`transition` elements. Neither type is on that list, so one of them on a layer below static
plates ended the run, and every plate above it was painted on every frame. In the
reporter's 684.9 s project, a narration track on layer 0 made a 6 s slice cost 4.7× its
paint time. Moving that track to a higher layer gave back byte-identical frames. Nothing told the author why.

Neither type has a frame-space footprint. The painter draws nothing for them and records
nothing for them, so their place in the stack cannot change a prefix's pixels or its
replayed paint records.

## Decision

1. **Passed over, not counted.** The static prefix (CONTEXT.md) is computed over the stack
   with `audio` and `transition` elements skipped. The run ends at its last drawn element,
   and a run of only such elements is no prefix.
2. **Not in the key.** The composite's key holds the drawn elements only. An audio clip
   starting or ending mid-span does not start a new prefix, unlike a drawn element entering
   (ADR-0195 §2).
3. **Still visited.** These elements are visited outside the composite on every frame, as
   before, so the answer and the profile list them the same way with the cache on or off.
4. **One list.** The types that draw nothing are one list in the painter. The paint
   dispatch and the prefix both read it, so a new silent type cannot be drawn as nothing
   and still end the prefix.

The elements a running transition bridges stay excluded while it runs (ADR-0195 §1). Only
the transition element itself is passed over.

Rejected: documenting "put audio tracks above the visuals". The layer of an element that
draws nothing has no visual meaning, and the advice would be a workaround for a defect.

`crates/montagent-core/tests/prefix_cache.rs` covers three cases under the ADR-0195 oracle
(cache on/off, 0, 1 and 3 painters, actual reuse asserted): audio below the plates for the
whole span, audio entering and leaving mid-span, and a transition element below the plates.
