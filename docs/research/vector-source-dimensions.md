# Vector sources and `fit`'s input: what the primary sources say

Gathered while resolving [#57](https://github.com/MBehtemam/Montaget/issues/57), which
[ADR-0015](../adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md) parked under
*Not settled here*: "Sources with no intrinsic pixel dimensions (SVG-like) have no rule
input and must use `literal`." This is a research file, not a decision — it answers the
four questions the ticket asked, with citations, and does not recommend a schema change.

All claims below cite an official/primary source (vendor docs, upstream source code, a
W3C spec, or a crate's own published metadata). Anything that could not be pinned to a
primary source is marked **NOT CONFIRMED**. Fetched September 2026.

## 1. Does the renderer stack actually support vector sources?

**`skia-safe` exposes an `svg` feature, but Montaget's pinned build does not enable it.**

- `skia-safe` 0.153.3 (latest on crates.io, published 2026-09-04) lists 31 feature
  flags; `svg` is one of them, enabling the `skia_safe::svg` module (`Dom`, `Canvas`,
  `Node`/`TypedNode`, and SVG element types like `Circle`, `Path`, `G`, `Gradient`,
  `Filter`) plus a `skia-svg-macros` dependency.
  ([docs.rs feature list](https://docs.rs/crate/skia-safe/latest/features),
  [module docs](https://rust-skia.github.io/doc/skia_safe/svg/index.html))
- The binding was requested in
  [rust-skia/rust-skia#545](https://github.com/rust-skia/rust-skia/issues/545) (opened
  2021-07-21, asking for `SkSVGDOM` bindings so SVG could be "parsed and drawn to the
  canvas") and is now closed/implemented, credited to contributors Osei Fortune and
  Savchenko Ivan in the crate's own release history.
- Upstream, `SkSVGDOM` lives in Skia's `modules/svg`
  ([SkSVGDOM.h](https://github.com/google/skia/blob/main/modules/svg/include/SkSVGDOM.h),
  [class reference](https://api.skia.org/classSkSVGDOM.html)) alongside `modules/skottie`
  (Lottie/Bodymovin animation) — a second, separate module, not something `svg` implies.
  Skia's own release notes describe the module as "left experimental in m88" with "many
  improvements since"
  ([skia-python m116 release notes](https://github.com/skia-python/skia-python/blob/main/relnotes/README.m116.md)),
  and the header itself carries no "stable" designation — **NOT CONFIRMED**: whether
  Google Skia has ever promoted `modules/svg` out of experimental status; no primary
  source found says so either way.
- **This project's own ADR pins a feature key that excludes it.** ADR-0010 fixes the
  prebuilt key to **`jpegd-jpege-pdf`** — "CPU-only, no `ganesh`, no `gl`" — and states
  explicitly: *"the feature set is pinned in one place and changed only deliberately;
  `svg` and `skottie` both imply `textlayout` and would move the key"* (ADR-0010,
  "The decision depends on a premise that decays"). `svg`'s own `Cargo.toml` dependency
  graph pulls in `skia-svg-macros`; whether it independently forces `textlayout` was not
  verified against the crate's build script — **NOT CONFIRMED** in detail, but ADR-0010
  asserts it as a fact about the pinned key. Either way, **as currently locked, `skia-safe`
  in this project does not have SVG decoding compiled in**, and enabling it moves off the
  prebuilt-binary path ADR-0010's whole affordability argument rests on (source compile:
  LLVM/Clang, Python, Ninja, 15.8 s → some larger, unmeasured cost).

**`SkSVGDOM` itself needs an explicit size when the SVG has none to give.** Its own API
docs make the exact shape of this ticket's problem visible inside Skia's own code, not
just in Montaget's schema:

> "Specify a 'container size' for the SVG dom. This is used to resolve the initial
> viewport when the root SVG width/height are specified in relative units." … "If the
> client specified a container size via `setContainerSize()`, then the same size is
> returned. When unspecified by clients, this returns the intrinsic size of the root
> element, as defined by its width/height attributes. If either width or height is
> specified in relative units (e.g. '100%'), then the corresponding intrinsic size
> dimension is zero."
> — [`SkSVGDOM` class reference](https://api.skia.org/classSkSVGDOM.html)

So Skia's own SVG DOM distinguishes "intrinsic size from width/height" from "caller-
supplied container size," and reports a **zero** dimension rather than inventing one when
neither is available — it does not, on its own, treat `viewBox` as a substitute intrinsic
size.

**`resvg`/`usvg` (linebender) is the more mature, separately-maintained decode path.**
Both crates are at **0.48.1**, last published 2026-08-02 on crates.io
([resvg](https://crates.io/crates/resvg), [usvg](https://crates.io/crates/usvg)).
`usvg`'s own README states its scope: *"resvg aims to only support the static SVG
subset; i.e. no `a`, `script`, `view` or `cursor` elements, no events and no
animations"* and notes SVG 2 support "is being worked on"
([linebender/resvg README](https://github.com/linebender/resvg)). This is the crate a
Rust host would plausibly pair with `skia-safe` for decode-then-rasterize, the same
division of labour ADR-0010 already uses for text (`parley`/`skrifa` outside the
rasterizer). This project has not adopted it; it is named here only as the path the
primary sources point to, per the ticket's ask.

## 2. Does the CapCut/Premiere reference class imply vector/SVG support?

**No — in neither product's own documentation is SVG a first-class importable asset.**

- **Adobe Premiere Pro.** Adobe's own supported-file-formats help page could not be
  fetched directly (`helpx.adobe.com` returns HTTP 403 to automated fetches in this
  environment) — **NOT CONFIRMED** by direct page text in this research pass. Corroborating
  evidence from Adobe's own community forum is direct, however: on an official Adobe
  Community thread about importing SVG into Premiere, a Community Expert answers *"Alas
  no svg"* and a moderator states *"SVG is not a supported file format,"* pointing to the
  official formats list and suggesting the file be converted first
  ([community.adobe.com thread](https://community.adobe.com/t5/premiere-pro-discussions/cant-import-svg-file-into-adobe-premiere/m-p/9637407)).
  Consistent with this, Adobe maintains a *separate, dedicated* help page titled
  **"Import SVG files"** — but it is filed under **After Effects**, not Premiere Pro
  ([helpx.adobe.com/after-effects/using/import-svg-files.html](https://helpx.adobe.com/after-effects/using/import-svg-files.html)).
  That page describes SVG import as vector artwork or a composition, convertible to
  editable shape layers, with the caveats that *"SVG Text will not be imported"* and
  *"animated and raster SVGs are not supported."* This matters for scope: **Adobe itself
  places SVG import inside the compositing/motion-graphics tool ADR-0003 names as
  explicitly out of scope**, not inside the general-purpose cuts-and-timing editor that
  is Premiere.
- **CapCut.** CapCut's own help article on import failures
  ("[Why Can't I Import MP4 and JPG Files?](https://www.capcut.com/help/can-not-import-mp4-and-jpg-files)")
  states *"CapCut mainly supports the following file formats"* — but the list itself is
  rendered as an embedded image on the page, not as page text, so it could not be read
  verbatim in this research pass (**NOT CONFIRMED** verbatim). No official CapCut
  documentation found in this pass lists SVG among supported formats; every
  officially-sourced enumeration of CapCut's image formats found in this pass (JPEG,
  PNG, GIF) omits it, and no CapCut help page analogous to Adobe's "Import SVG files"
  page was found.

Under ADR-0003's own asymmetry rule — the fixture's lack of a capability is never
evidence against needing it — the inverse also holds here: **neither reference vendor's
own documentation is evidence a general-purpose video editor of this class is expected
to accept SVG as importable footage.** Both vendors that do document SVG import
(Adobe, via After Effects) do so in the tool ADR-0003 already excludes.

## 3. What does the SVG spec itself guarantee as an intrinsic size?

**Nothing is required. `width`, `height` and `viewBox` are all independently optional
on the root `<svg>`, and the spec defines what happens when none are present.**

- SVG 1.1 (Second Edition), §5.1.2, on the root element's `width`/`height`: *"If the
  attribute is not specified, the effect is as if a value of '100%' were specified"* —
  i.e. omission is legal and falls back to a percentage, not an error
  ([w3.org/TR/SVG11/struct.html](https://www.w3.org/TR/SVG11/struct.html)). A
  percentage width/height is not itself a pixel dimension; it needs an outer viewport to
  resolve against.
- `viewBox` is never required at any SVG version; SVG 1.1 only *recommends* it for
  documents meant to be embedded and scaled ("the author will often want to include a
  `viewBox` attribute…"), and does not define a default value for it when omitted
  (same section).
- **When both are percentage/omitted and there's no outer viewport to resolve
  against (the standalone-document case this ticket is about), the CSS replaced-element
  default governs**: SVG 2 delegates sizing to CSS's *Default Sizing Algorithm* for
  replaced elements, which falls back to **300×150** when there is no intrinsic size at
  all: *"when the referenced resource does not have an intrinsic size … it is assumed to
  have a width of 300px and a height of 150px"*
  ([SVG 2, §12.2 Embedded Content](https://svgwg.org/svg2-draft/embedded.html), citing
  the CSS Images Module 3 algorithm). Note this 300×150 fallback is a **UA/CSS-embedding
  convention**, not a value baked into the SVG document itself — it is what a browser
  invents for a `<img>`/background reference, not a number the file asserts about
  itself.
- The SVG Working Group's own wiki names this exact ambiguity as an open interoperability
  problem, not a solved one: percentage or omitted `width`/`height` combined with no
  `viewBox` is flagged as "Issue 1" — content authored this way "won't scale when
  embedded," and authors are told to add a `viewBox` or use absolute units instead
  ([Intrinsic Sizing wiki](https://www.w3.org/Graphics/SVG/WG/wiki/Intrinsic_Sizing)).

**So the spec-guaranteed floor is: nothing.** An `<svg>` with a `viewBox` and no
`width`/`height` has a well-defined **aspect ratio** (from the `viewBox`) but no
spec-mandated pixel magnitude; a `<svg>` with none of the three has neither, and every
consumer (browser, library) is left to invent a concrete size by its own convention.

## 4. Precedent: how other tools report "dimensionless" vector media

Two independent, unrelated implementations converge on the same answer: **invent a
placeholder pixel size — and the placeholder these two primary sources chose is
identical (100×100) — rather than report "no dimensions" as a distinct case.**

- **FFmpeg's own `librsvg` decoder** (`libavcodec/librsvgdec.c`) exposes `width`/`height`
  options documented as *"Width to render to (0 for default)"* / *"Height to render to
  (0 for default)"*. Reading the source: for librsvg ≥ 2.52 it calls
  `rsvg_handle_get_intrinsic_size_in_pixels()`, and **falls back to 100×100 pixels** when
  that call fails or the SVG has no intrinsic size
  ([FFmpeg/FFmpeg librsvgdec.c](https://github.com/FFmpeg/FFmpeg/blob/master/libavcodec/librsvgdec.c)).
  `ffprobe`/`ffmpeg`'s own image2 demuxer separately fails outright with **"unspecified
  size"** when it cannot recover a width/height at all during stream probing (observed on
  the `-f image2pipe` path, where the SVG header is present but not reachable during
  probing) — i.e. FFmpeg has a real, user-visible failure mode for "no size found," and
  a separate, distinct fallback path once a decoder is actually invoked
  ([ffmpeg-user mailing list thread, 2017-11](https://ffmpeg.org/pipermail/ffmpeg-user/2017-November/037798.html)).
  There is **no native FFmpeg SVG demuxer**: SVG input support is entirely conditional on
  building against `librsvg`, an external C library — SVG does not appear in FFmpeg's own
  formats documentation at all
  ([ffmpeg.org/ffmpeg-formats.html](https://ffmpeg.org/ffmpeg-formats.html)).
- **`usvg` (the Rust SVG parser named in §1) makes the same choice, independently, at the
  API level.** Its parser options carry a documented `default_size` field:

  > "Default viewport size to assume if there is no `viewBox` attribute and the `width`
  > or `height` attributes are relative. Default: `(100, 100)`"
  > — [`usvg::Options::default_size`](https://github.com/linebender/resvg/blob/main/crates/usvg/src/parser/options.rs)

  And every parsed document exposes a concrete, non-optional pixel size —
  `Tree::size() -> Size`, documented as *"Image size. Size of an image that should be
  created to fit the SVG. `width` and `height` in SVG"*
  ([`usvg::Tree::size`](https://github.com/linebender/resvg/blob/main/crates/usvg/src/tree/mod.rs)).
  `usvg` never returns "no size" from a successfully parsed tree; it resolves one, using
  the caller-suppliable 100×100 default when the document itself supplies nothing.

**Skia's `SkSVGDOM`** (§1) is the third data point and the outlier in mechanism, not
outcome: rather than picking a number for the caller, it returns a **zero** dimension on
the relevant axis and requires the caller to call `setContainerSize()` — pushing the
"what number do we use" decision outward to whatever embeds it, rather than resolving it
internally. All three tools agree on the same underlying fact: **an SVG's own bytes do
not always carry a decodable pixel magnitude, and every implementation that has to
produce one adds a policy - a fallback constant, an explicit caller-supplied override, or
an outright probe failure - that is not part of the SVG format itself.**

No other image library's SVG-size convention was checked against primary sources in this
pass; the above three (FFmpeg+librsvg, `usvg`, Skia's `SkSVGDOM`) are the ones the ticket
named or that this research surfaced as directly load-bearing.

## Summary of what could not be confirmed from primary sources

- Whether Google Skia's upstream `modules/svg` has ever been promoted out of
  "experimental" status (only a `skia-python` release-note mention of "left experimental
  in m88" was found; no later status change was located).
- Whether `skia-safe`'s `svg` feature independently forces the `textlayout` feature via
  its own `Cargo.toml`/build script — taken here on ADR-0010's own assertion, not
  independently re-derived from `rust-skia`'s build configuration.
- Adobe's official Premiere Pro supported-file-formats page could not be fetched directly
  in this environment (HTTP 403); the finding that SVG is absent from it rests on Adobe's
  own community-forum corroboration, not the primary formats page itself.
- The exact text of CapCut's own supported-formats list, which its help center renders as
  an image rather than page text.
- Whether any raster-image library commonly paired with a Rust renderer (beyond `usvg`)
  has a documented SVG-size convention; only the three tools above were checked.
