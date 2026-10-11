---
status: accepted
amends: 0144 (§2/§6 by reference, §7's shared painter state), 0194 (decision 6's deferred frames_reused counter now joins the profile)
---

# Time-invariant bottom layers share a byte-identical prefix composite

[#932](https://github.com/MBehtemam/Montagent/issues/932), implementing the owner-approved
verdicts of [#923](https://github.com/MBehtemam/Montagent/issues/923) and
[#924](https://github.com/MBehtemam/Montagent/issues/924).

## Context

The reporter's 1080p book-page reproduction kit spends about three quarters of its paint
time on a static table and a feathered book plate. ADR-0144 §6 rejected caching moving
filtered layers at a 6.7% hit rate. These plates hold throughout their visible span, so a
bottom composite can be reused on every subsequent frame in that interval.

The byte-identity probe on `probe/923-prefix-identity` restored prefixes after 1–10, 12,
14 and 16 layers, including masks and shadows, with every decoded frame hash unchanged.
The CPU surface is 8-bit premultiplied RGBA: each draw already rounds to 8 bits, and an
exact copy gives later layers the same input as drawing the prefix again.

## Decision

1. **Automatic bottom prefix, whole-frame.** Cache the contiguous bottom run of static
   images, text, rectangles, ellipses and paths. No project field, flag or MCP hint chooses
   this. A keyframe anywhere in the derived animatable properties, including effect and
   gradient parameters, ends the prefix. Videos, group members, non-normal blends, motion
   blur, unit staggering and timed text highlights end it. Only known time-invariant
   effects are eligible; grain and unknown effects end it. A running transition also
   excludes either element it bridges.
2. **Presence and surface are part of identity.** The key is the prefix's ordered element
   positions and the canvas size/base scale. An entering or departing element changes the
   key. Restore with `Src`, nearest sampling and the identity device matrix, so preview's
   base scale is not applied twice. Background is included in the composite.
3. **Shared immutable pixels and records.** Each render shares its latest prefix entry
   across its painters. A short mutex guards lookup/publication; painting and copying
   pixels hold no lock. Simultaneous misses may paint the prefix more than once. Keeping
   only the latest entry bounds retained cache storage to one whole-frame composite;
   painters temporarily retain entries they are restoring. Scheduling may change reuse
   counts or cause a miss, never a frame's bytes. `frame`'s one-off pictures do not cache.
4. **Reports retain their meaning (ADR-0144 §7).** Replay the prefix's `painted`,
   `not_painted`, `painted_partially` and `declined` records in order. Cache construction
   isolates source/font provenance and font registration from earlier painter history;
   hits merge only that prefix's files into the painter's first-use lists. Video never
   enters the prefix, and supplier `begin_frame` and requests remain per frame. Internal
   and missing-tool failures are not cached. A deferred paint error still stops the
   deliverable with the same finding, before another frame can restore that failure.
5. **Profile reuse.** `render.profile` adds `frames_reused` to element and type rows and
   the folded element remainder. It counts element turns restored from the prefix,
   independently of `frames_painted`. Effect applications count actual applications;
   restore/copy overhead stays unattributed. Chunk profiles sum both frame counters.
6. **Identity is gated; speed is observed.** No tolerance, new bottleneck class or speed
   target. `PAINT_TARGET` and ADR-0144's moving-content cache verdict stand. The feathered
   mask is cached with its static plate; no separate mask-alpha cache or changed filter
   arithmetic. Static layers above moving content still pay full price.

`crates/montagent-core/tests/prefix_cache.rs` compares encoder-input hashes, MP4 bytes on
targets where ADR-0190 guarantees them, and public answers with the cache on/off, on one
and three painters. Only execution timing, profile/painting disclosure and independent
probe-sidecar history are excluded. Every successful comparison asserts actual reuse.
Coverage includes masked/shadowed plates, opacity keyframes, grain, groups, blends,
transitions, presence changes, video feed bookkeeping, font/source first-use order and
scaled previews. A still lost after preflight checks the deferred-error path.

`force_prefix_cache` and `MONTAGENT_PREFIX_CACHE=off` are hidden test/measurement plumbing,
like ADR-0144's painter override; they are not authoring or tool settings.

## Measurements

See [the reproducible sweep](../research/static-prefix-cache/measure.py) and its recorded
results in [the measurement notes](../research/static-prefix-cache/README.md). The
"unbaked within 10% of baked" aim is an observed number, never a time gate. Baked pixels
are not an identity oracle: changing two authoring layers into a PNG may change bytes.
The identity comparison is the unchanged project with caching enabled and disabled.
