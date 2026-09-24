---
status: accepted
amends: 0021 (states the deferred target resolution and ladder length; defers the floor)
---

# The proxy-preview target is a single 720p tier, long-edge capped

> **Amended by three later ADRs.** Read them before relying on anything below.
>
> - [ADR-0078](0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)
>   — states two things the *"no proxy applies at all"* sentence below leaves open. **A
>   project between the two caps still degrades**: its first rung is true pixels and its
>   second is a real 960 px proxy, because the ladder is defined on caps rather than on
>   sizes. And **a rung whose cap never engaged is disclosed as `native`** with a null
>   `long_edge_cap` — the required tier field, and the attempt trace and refusal beside it,
>   name the frame that was rasterized, never a tier that did nothing
>
> - [ADR-0050](0050-preview-hard-refuses-below-360p.md) — adopts a floor below the 720p
>   target
> - [ADR-0067](0067-two-floors-a-wall-clock-give-up-point-and-a-legibility-refusal.md) —
>   confirms the long-edge cap survives 0065

[ADR-0021](./0021-preview-budget-and-graceful-degradation.md) committed to a proxy-resolution
mechanism for scrub `preview` above the render budget, deliberately deferring the specific
target resolution, ladder length, and floor to a measurement ticket rather than guessing —
this project has already had to walk back one unmeasured claim stated as fact (ADR-0005's
frame-alignment instruction). [#87](https://github.com/MBehtemam/Montagent/issues/87)
measured it: a harness extending [#34](https://github.com/MBehtemam/Montagent/issues/34)'s
rasterizer benchmark with synthetic 4K (2160×3840) and 8K (4320×7680) portrait sources,
matching the fixture's own 1080×1920 aspect ratio.

## The target: 720p, one tier

720p is the smallest single target that clears the <5s scrub-preview budget at both 4K and
8K sources. 1080p only clears it at 4K (misses 8K at 5.79s, 16% over). Stepping further down
— 720p → 540p → 360p — buys under 2 seconds combined, against a 5-second budget. Decided
unanimously by a three-juror court (Opus, Haiku, Fable): **a single tier**, not a longer
ladder. ADR-0021 asked for "small, finite, strictly ordered" — one rung is the smallest thing
that satisfies that shape, and #87's numbers say a second rung isn't earning its cost (a
selection policy, a second tested code path, a second degraded-output shape every tool must
report). The one number that argues for a fallback rung — a thin 1.8% 8K margin at 720p — was
measured on `tiny-skia`, a backend this project did not choose; the renderer it did choose
(`skia-safe`, [ADR-0010](./0010-skia-safe-rasterizer-text-beside-it.md)) clears 8K at 720p
comfortably. Designing a permanent extra tier around a rejected backend's margin is
speculative generality. If a real workload ever misses the 720p budget on `skia-safe`, the
honest fix is to re-measure and revise this ADR, not to have pre-baked rungs today.

## What "720p" means for an arbitrary aspect ratio

Montagent is a general-purpose editor ([ADR-0003](./0003-general-video-editor-not-channel-tooling.md)),
not vertical-only, but "720p" is a landscape-era shorthand (conventionally 1280×720, height
capped at 720) and the fixture — and #87's harness — is 9:16 portrait. Stated precisely:

> **The proxy-preview target scales the project's longer edge (width or height, whichever
> is larger) to at most 1280px, preserving the native aspect ratio, with both resulting
> dimensions rounded to the nearest even integer.**

For the fixture's 1080×1920 (portrait), this gives 720×1280 — matching #87's measured
numbers exactly, since 9:16 is 16:9 rotated and #87's harness already used this convention.
For a 1280×720-native or smaller project, no proxy applies at all — the cap only engages
above it.

Unanimous 3/3 on long-edge capping over the alternatives considered:

- **Height-capped ("traditional" 720p convention, height always ≤720px)** — rejected. Applied
  to the fixture's 9:16 ratio this gives ~405×720, 44% fewer pixels than the long-edge cap —
  a materially worse legibility point and a different (untested) performance point than what
  #87 actually measured. It makes "720p" mean something different depending on orientation,
  which is the exact ambiguity this ADR exists to close.
- **Total pixel budget** (solve for ~921,600px at the native aspect ratio) — rejected. Nearly
  equivalent arithmetically to the long-edge cap for common ratios, but is the wrong *rule*:
  it yields non-integer, non-even dimensions needing their own rounding decision, and for
  unusual ratios (4:3, 21:9) it is not a clean, checkable predicate the way "long edge ≤
  1280px" is.
- **Scale factor relative to native** — rejected as wording, not arithmetic (identical to the
  long-edge cap for one aspect ratio). "Downscale by a factor" invites reading the target as
  dependent on source resolution rather than a fixed cap, the exact ambiguity ADR-0021 already
  rejected once for a different reason (a scale factor "doesn't bound anything at the high
  end"). State it as a fixed cap, not a ratio.

## The floor: deferred to a `/prototype` ticket

ADR-0021 wants a floor tier below which `preview` hard-refuses rather than return something
"too degraded to make a decision from." That is a visual-legibility judgment — can on-screen
text still be read, can composition and timing still be judged — not something a wall-clock
benchmark like #87's can answer, and not something this ADR decides by argument. Unanimous
3/3: split it off as a `/prototype` ticket that renders real fixture content at a few
candidate floor resolutions and makes the call by looking, the same discipline ADR-0021 used
to defer the target itself to measurement rather than argument. Graduated to
[#117](https://github.com/MBehtemam/Montagent/issues/117).

Until #117 resolves, `preview` has no defined floor: it attempts the 720p target or fails on
budget, with no refuse path below it — stated here explicitly rather than left implicit.

## Consequences

- `preview`'s proxy-resolution mechanism has exactly one tier: 720p (long-edge capped at
  1280px, aspect preserved, dimensions rounded to even). No intermediate degradation levels
  exist.
- The result's required resolution-tier field (ADR-0021) reports one of: native, the 720p
  proxy tier, or — once #117 resolves — a floor-refuse.
- `preview`'s target-resolution rule is stated as a fixed, orientation-independent cap
  (longer edge ≤ 1280px), not a landscape-convention height cap and not a scale factor.
- The floor is explicitly undecided, not silently assumed: graduated to
  [#117](https://github.com/MBehtemam/Montagent/issues/117).
