---
status: accepted
amends: 0015 (discharges its "sources with no intrinsic pixel dimensions" deferral)
---

# Vector sources are out of scope for v1

[ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md) parked, without
deciding: *"sources with no intrinsic pixel dimensions (SVG-like) have no rule input and
must use `literal`."* [#57](https://github.com/MBehtemam/Montagent/issues/57) asked whether
Montagent accepts vector sources at all, and if so what `fit` should do about a source with
no decodable pixel size. Research findings, cited to primary sources, are in
[`docs/research/vector-source-dimensions.md`](https://github.com/MBehtemam/Montagent/blob/research/vector-source-dimensions/docs/research/vector-source-dimensions.md)
(branch `research/vector-source-dimensions`).

## Decision

**Vector sources (SVG-like) are out of scope for v1. `fit`'s schema needs no vector case,
because no element type accepts a vector source to begin with.**

Two independent findings converge:

**The renderer stack, as currently pinned, cannot decode one.** `skia-safe` exposes an
`svg` feature wrapping upstream Skia's `modules/svg` (`SkSVGDOM`), but
[ADR-0010](./0010-skia-safe-rasterizer-text-beside-it.md) pins the prebuilt feature key to
`jpegd-jpege-pdf` specifically *because* `svg` (and `skottie`) would move the key off the
free-prebuilt path its whole affordability argument rests on. Enabling vector decode today
means trading a 15.8-second prebuilt fetch for an unmeasured source compile — a cost ADR-0010
was written to avoid, not a cost this ticket has grounds to reintroduce. Separately, Skia's
own `SkSVGDOM` API documents that it returns a **zero** dimension, not an invented one, when
an SVG's root has no absolute width/height — meaning even a fully-wired vector path would
still need external policy to produce a fit-rule input, not get one for free.

**The reference class doesn't ask for it.** Neither Adobe Premiere Pro nor CapCut treats SVG
as first-class importable footage, per their own documentation: Adobe's own community forum
states "SVG is not a supported file format" for Premiere, and Adobe's dedicated "Import SVG
files" help page exists only under **After Effects** — the tool ADR-0003 already excludes
from this project's reference class. Under ADR-0003's asymmetry rule, absence of evidence for
a need is not evidence against one — but here it cuts the other way too: neither reference
vendor's documentation is evidence the CapCut/Premiere class *expects* vector import either.
There is no standing requirement pulling this capability in.

Given both a real, named cost (ADR-0010's pinned key) and no pull from the reference class,
the format admits no vector-source element in v1. This is a scope decision, not a
because-the-fixture-lacks-one decision — it is grounded in the renderer's actual cost
structure and the reference class's own documented behaviour, exactly the kind of evidence
ADR-0003 requires.

**The SVG spec itself would not have made this easy even with a wired decoder.** `width`,
`height` and `viewBox` are all independently optional on a root `<svg>`; when none resolve
to a concrete size, the only guaranteed fallback is CSS's 300×150 replaced-element default —
a UA embedding convention, not something the file asserts about itself. FFmpeg's `librsvg`
decoder and the `usvg` crate both independently converge on inventing a 100×100 placeholder
in that case. Every implementation checked adds its own policy on top of the format; none of
them derive a magnitude from the SVG bytes alone. Had this project accepted vector sources,
it would have needed to pick and document its own such policy — a second decision this ADR
avoids by not needing one.

## Consequences

- No schema change: no vector-source element type is added, so `fit`'s value set (`cover` /
  `contain` / `literal`) and its required-on-raster-source rule (ADR-0015) are untouched.
  ADR-0015's `Not settled here` entry is discharged with "not applicable — out of scope."
- `probe` need not define a "dimensionless source" report category; every source Montagent
  accepts has decodable pixel dimensions.
- This is a v1 scope boundary, not a permanent rejection. If vector-source support is
  proposed later, it reopens as a fresh scope question weighing the same two costs named
  here (the renderer's prebuilt-key cost, and whatever reference-class evidence exists at
  that time) — not a resumption of this ticket.

## Not settled here

- What policy Montagent would adopt for a vector source's fit-rule input, should vector
  sources ever be accepted. Every primary-source precedent found (Skia's zero-dimension,
  FFmpeg/librsvg's and usvg's 100×100 fallback) is a *different* policy; none is adopted or
  preferred here, since none is needed while vector sources remain unsupported.
