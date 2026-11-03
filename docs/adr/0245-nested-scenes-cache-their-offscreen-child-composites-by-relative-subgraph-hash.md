---
status: accepted
amends: 0092 (`cache clear` removes `<cache_dir>/nest-cache` alongside `probe-cache.json`), 0195 (time-invariant `nest` elements join the bottom static prefix, and `nest` offscreen child composites are cached across frames and runs), 0201 (`nest` offscreen child composites are keyed by relative sub-graph hash and reused across frames and runs)
---

# 245. Nested scenes cache their offscreen child composites by relative sub-graph hash

Date: 2026-11-03

[#1144](https://github.com/MBehtemam/Montagent/issues/1144).

## Context

In multi-scene projects (such as the 6-scene product sizzle), each scene is authored as a `type: "nest"` container ([ADR-0201](0201-nested-sequences-are-bounded-containers-rendering-to-an-isolated-surface-on-an-absolute-timeline.md)). During `render`, `Painter::nest` allocates an isolated `width × height` offscreen RGBA canvas, rasterizes all active child elements in layer order, snapshots the offscreen buffer, and then draws that buffer onto the parent canvas with the `nest` container's own transform, clip, opacity, blend mode, and parent-level `effects`.

Previously:
1. `prefix::invariant` ([ADR-0195](0195-time-invariant-bottom-layers-share-a-byte-identical-prefix-composite.md)) rejected `type: "nest"` outright, so even a completely static `nest` at the bottom of the stack broke the static prefix and re-rasterized its children on every frame.
2. Within a render, a `nest` sitting above moving layers or undergoing a parent transform/transition ([ADR-0243](0243-native-transitions-between-nest-scenes-render-via-offscreen-surfaces.md)) re-rasterized its entire child tree on every frame even across spans where the children themselves did not change.
3. Across re-renders during iterative editing (when an author tweaks one scene and re-runs `montagent render`), every unchanged `nest` scene was rasterized from scratch.

Encoding intermediate video files per `nest` would introduce lossy codec generation loss (or large mezzanine files) and violate byte-identity ([ADR-0195 §6](0195-time-invariant-bottom-layers-share-a-byte-identical-prefix-composite.md)). Caching the `nest`'s offscreen 8-bit premultiplied RGBA snapshot and its child paint records preserves exact byte-identity while skipping child rasterization on cache hits.

## Decision

1. **Relative sub-graph hash for `nest` child composites**:
   - The offscreen child composite of a `nest` depends only on its child `elements`, the offscreen `Extent { width, height }`, the project `fps`, and the files (fonts, images, videos, LUTs) those children read — **not** on the parent `nest`'s `x`, `y`, `scale`, `rotation`, `opacity`, `blend`, `clip`, parent `effects`, or parent transitions.
   - Because child timestamps are written on the project's absolute clock (`ADR-0201 §2`), the sub-graph hash normalizes every child timestamp by subtracting `nest.start` (`child.start - nest.start`, `child.end - nest.start`, keyframe `t - nest.start`, text run `highlight` windows). Consequently, shifting a `nest` on the timeline (`montagent shift`) or moving/fading the `nest` container on the parent canvas keeps its sub-graph hash identical.
   - Every local file referenced by any child (via `source`, `lut`, or the `fonts` entries referenced by child text elements/runs) contributes its canonical path, byte size, `mtime_ns`, and `sidecar::fingerprint` content guard ([ADR-0092 §3](0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md)) to the sub-graph hash; if any referenced file is missing or lacks `mtime_ns`, persistent caching is skipped for that `nest`.

2. **Invariant-span deduplication vs per-frame slot**:
   - Inside a `nest`, the active children at `instant` determine the visual state.
   - If every active child at `instant` is time-invariant (`prefix::invariant` holds recursively on the child), the cache slot key is `Slot::State(active_indices)` — one cached RGBA composite serves every frame with that active child set.
   - Otherwise (a child has keyframes, video, `motion_blur`, `units`, `highlight`, or `grain`), the cache slot key is `Slot::Offset(instant - nest.start)`.

3. **Two-tier storage (in-memory shared across painters + bounded per-user disk cache)**:
   - Painters within a `render` or `preview` share an in-memory map of cached `nest` child composites (`pixels: Raster` + child `prefix::Record`), and read/write through `<cache_dir>/nest-cache/` under the per-user cache directory ([ADR-0069](0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md), `MONTAGENT_CACHE_DIR`).
   - Like `probe-cache.json`, `<cache_dir>/nest-cache/` never lands beside the project file ([ADR-0053](0053-project-file-plus-relative-assets-is-the-movable-unit.md)), writes atomically via PID-suffixed temp files, treats any I/O failure as a silent cache miss, and is bounded by LRU eviction (`64` scene sub-graphs / `256 MiB`).
   - `montagent cache clear` ([ADR-0092](0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md)) removes `<cache_dir>/nest-cache/` (and any `.tmp` files) alongside `probe-cache.json`.

4. **Static-prefix admission and `render.profile` accounting**:
   - `prefix::invariant` ([ADR-0195](0195-time-invariant-bottom-layers-share-a-byte-identical-prefix-composite.md)) admits a `nest` element when the `nest` itself satisfies `prefix::invariant`'s outer checks (normal blend, no keyframes, no `motion_blur`, only `STATIC_EFFECTS`) and all of its children recursively satisfy `prefix::invariant` with constant presence across the `nest`'s `[start, end)`.
   - When a `nest`'s offscreen child composite is restored from the `nest` scene cache, `Painter::nest` replays the child paint records (`painted`, `not_painted`, `painted_partially`, `declined`, `sources`, `fonts`), applies the `nest`'s own parent `transform`, `clip`, and `effects` onto the parent canvas, and records `frames_reused` in `render.profile` ([ADR-0194](0194-render-profile-says-where-the-painters-cpu-went-by-element-type-and-effect.md)) for the `nest` element and `"nest"` type.
   - `MONTAGENT_NEST_CACHE=off` and `force_nest_cache(bool)` provide `#[doc(hidden)]` test/measurement plumbing parallel to `MONTAGENT_PREFIX_CACHE=off` and `force_prefix_cache(bool)`.
