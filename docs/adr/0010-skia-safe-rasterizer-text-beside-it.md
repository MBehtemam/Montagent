---
status: accepted
---

# The renderer is `skia-safe`, with text layout beside it and FFmpeg outside it

> **Amended by [ADR-0040](0040-effect-model-attachment-and-v1-vocabulary.md).** Names its
> "obvious members" — blur, drop shadow — as the accepted v1 set.

Montaget rasterizes with **`skia-safe`** (Rust bindings to Google's C++ Skia).
Text is shaped and positioned by **`parley`** and scaled by **`skrifa`** *outside*
the rasterizer, which only fills paths. **FFmpeg is a separate subprocess** for
decode and encode.

This ADR records the **renderer**. [ADR-0009](0009-rust-host.md) records the
host — Rust and `rmcp` — and explicitly declined to choose the rasterizer. Where
this ADR depends on a claim ADR-0009 established, it cites rather than re-argues,
so reversing one does not silently reverse the other.

## Why — and why not for the reason the measurement led with

**Not for speed.** [#34](https://github.com/MBehtemam/Montaget/issues/34)
measured `skia-safe` at **6.7 ms/frame against `tiny-skia`'s 13.5 ms** at
1080×1920, and framed the < 5 s preview budget as *"the whole decision."*
**That framing is rejected here; the measurements are kept.** Two reasons.

First, **the gap is one function.** Bilinear resampling of a single Ken Burns
still is ~93 % of the frame in both arms and holds *all* of the difference
(6.5 ms vs 12.3 ms). Glyph-path filling is 0.4 ms vs 1.1 ms — 5 % of the frame.
On flat fills **`tiny-skia` is 2.5× faster.** The honest statement is not
"`tiny-skia` is slow"; it is *"`tiny-skia`'s image resampler is about half the
speed of Skia's, and that scene is an image-resampler benchmark."* A reason that
one upstream patch could erase is not a reason to write down.

Second, **the budget it was measured against is not a gate.** The < 5 s preview
number was written for 1080×1920/30 and has never been re-derived since
[ADR-0003](0003-general-video-editor-not-channel-tooling.md) generalised the
scope. `tiny-skia` sits at 4.92–5.03 s *without* video — on the line — and only
fails once a clip is on the timeline. Reading 5.03 against 5.00 as a verdict is
reading noise as a decision. And the agent's dominant self-check is not `render`
at all: it is **`frame`, at 0.11–0.27 s cold, where the two arms do not
discriminate** (0.12 vs 0.13 s).

**The durable reason is a capability ceiling.** `tiny-skia` 0.12.0 puts **GPU
rendering, image filters and blur out of scope** — ADR-0009 already named these
as "the real objections to it — not text." These are missing subsystems, not
missing optimisations: no patch adds them without `tiny-skia` becoming a second
Skia. Montaget's effect vocabulary is a closed named list still to be designed
([#22](https://github.com/MBehtemam/Montaget/issues/22)) and blur is an obvious
member. Choosing `skia-safe` buys headroom for a feature category already
anticipated. **That is a capability bet, not a speed bet** — and if someone
optimises `tiny-skia`'s resampler, nothing here changes.

**The C++ dependency is affordable because the prebuilt matches.** `skia-safe`
fetches binaries keyed on the *exact sorted feature set*, and a miss means
compiling Skia from source (LLVM/Clang, Python, Ninja). #34 verified the hit on
one target; it generalises: the required key **`jpegd-jpege-pdf`** — CPU-only,
no `ganesh`, no `gl` — is published for 0.153.2 on **all six desktop tier-1
targets**: `{aarch64, x86_64}` × `{apple-darwin, unknown-linux-gnu,
pc-windows-msvc}`. The cost is 15.8 s from an empty `CARGO_HOME` and ~200 MB of
build artifacts, against `tiny-skia`'s 4.7 s and 17 MB. The shipped binary is
11.4 MB (9.6 MB stripped).

**Maintenance is not an argument, and was corrected.** `tiny-skia`'s two-year
release gap (0.11.4, Feb 2024 → 0.12.0, Feb 2026) is real but **ended seven
months ago**. Describing it as unmaintained today is not supported, and this ADR
does not rely on it.

## `tiny-skia` is the exit route, not a second backend

`tiny-skia` is **not** a supported parallel backend. A backend kept switchable
constrains every rendering feature to the *intersection* of both — which is
precisely the no-filters, no-GPU ceiling `skia-safe` was chosen to escape. It
would either be abandoned the first time blur ships, or it would hold and negate
the reason for choosing Skia at all.

It is recorded instead as the **named exit** if the C++ prebuilt story breaks.
What keeps that exit cheap is the artifact #34 already built: **the two-arm
shared-ops-list harness stays in-tree.** One ops list, two rasterizers, arms
agreeing to antialiasing noise (mean Δ 0.05–0.07 of 255). It serves twice — as
the correctness oracle that makes the exit real rather than aspirational, and as
a **golden-frame guard**, since a `skia-safe` bump can quietly shift resampling
and antialiasing with nothing erroring. This project has shipped a render that
completed, looked plausible and was wrong **twice** (the FFmpeg `zoompan` trap;
the headless-Chrome pipeline blank behind its text). That guard is not optional.

**The revisit trigger, stated so it is not re-litigated from scratch:** reopen
if `tiny-skia` grows image filters and a GPU backend **and** the `skia-safe`
prebuilt story breaks. **Not** if someone merely optimises the resampler.

## Text layout stands beside the rasterizer

Unanimous across three independent reviewers in
[#7](https://github.com/MBehtemam/Montaget/issues/7) and carried by ADR-0009,
recorded here because it is a *renderer* decision and has no other home.

**`SkParagraph` is excluded, and the exclusion is load-bearing.**
[ADR-0008](0008-line-breaks-belong-to-the-agent.md) requires `measure` to report
**break opportunities** and to name its segmenter and data version.
`SkParagraph` is a wrapping-policy engine with ICU sealed inside and exposes
neither. Taking it would ship **two shapers**, and the one the agent measures
with would not be the one that draws.

`parley` shapes (with `harfrust`) and breaks (with `icu_segmenter`); **`skrifa`
scales the outlines** and the rasterizer fills the paths. `swash` is not used —
it would duplicate a shaper the stack already has. Per
[ADR-0007](0007-text-runs-literal-size-declared-fonts.md) there is **no
automatic wrapping**: the agent places every break itself.

**The coupling, stated explicitly** so that reversing one decision does not
silently reverse another: adopting `SkParagraph` later would re-seal ICU and
break the agent's break-opportunity contract, which is the whole basis of
ADR-0008 — *and* it would remove the reason "Skia has better text" was
unavailable as an argument for `skia-safe` in the first place. Swapping to
`tiny-skia` forfeits image filters and GPU headroom.

## FFmpeg is a subprocess

Cited to ADR-0009, not re-argued: `libx264` is GPL, and linking `libavcodec`
would make the shipped binary a GPL combined work. Decode and encode therefore
run in a spawned `ffmpeg` the user supplies.

One consequence belongs here rather than there. Because both arms in #34 used
the **identical** FFmpeg subprocess, decode was identical in both (0.63–0.68 s
of a 10 s preview, ~16 %) and **cancels out of the comparison entirely** — which
is what makes the 1.8–2.0× rasterizer ratio mean anything. Decode is not a
rasterizer axis and must not be read as one.

Also settled by #34, and asked for by ADR-0009: **#6's position pathology does
not reappear.** Time-to-first-frame after an input seek is flat across the file
(0.10–0.18 s at every position tested). Cost proportional to where a preview
window *starts* was an artifact of a synthesised FFmpeg filtergraph having no
decoder to seek. Give the backend a decoder and it goes away. `--from`/`--to`
partial render is therefore ordinary here, not architectural.

## Consequences

### Montaget does not need to be a resident server

ADR-0009 asked the renderer ADR to state which mode it assumes. **It assumes
none.** A fully cold process — launch, load the project, resolve fonts, decode
the still, seek the clip, rasterize, encode PNG — costs **0.11–0.27 s**, against
`skia-canvas`'s 0.50 s and FFmpeg's 0.93 s. The question dissolves rather than
being answered, and **it does not discriminate between rasterizers.**

### The decision depends on a premise that decays

Accepting a C++ prebuilt was justified on prebuilt coverage **verified today**.
That premise must be continuously verified: a sustained prebuilt miss on a
tier-1 target is a trigger to revisit toward the `tiny-skia` exit. The
*mechanism* — a cold-cache CI job across the six targets that fails if a Skia
source compile is triggered — is implementation and is ticketed separately
([#36](https://github.com/MBehtemam/Montaget/issues/36)). The feature set is
pinned in one place and changed only deliberately; `svg` and `skottie` both
imply `textlayout` and would move the key.

### GPU is not enabled, and 4K is a budget problem

The chosen key is **CPU-only**. At 2160×3840 a 10 s preview with video costs
**19.0 s (`skia-safe`) and 30.0 s (`tiny-skia`)** — *both* miss the < 5 s budget
by 4–6×, so 4K discriminates nothing about this choice. It is a fact about the
**budget**, ticketed as
[#35](https://github.com/MBehtemam/Montaget/issues/35); a proxy-resolution
preview strategy is the likely shape, and enabling a `ganesh`/`metal` prebuilt
(published, a different key) is the other lever. `tiny-skia` has no such lever.

### Not decided here

- **Image filters and blur are untested in either arm.** Belongs to the effect
  model ([#22](https://github.com/MBehtemam/Montaget/issues/22)).
- **The glyph atlas.** Neither arm used one, so the measured gap **understates**
  `skia-safe`'s lead rather than overstating it. Skia has one; `tiny-skia` has
  no equivalent.
- **What `frame` returns and at what scale.** PNG encode is 10–12 ms under this
  host — ~9 % of a cold frame, not the third of one #6 implied. Whatever decides
  it, it will not be encode latency.
- **"Rust is slower than Node" must not be read out of #34.** `skia-canvas`
  renders the scene in 13.07 s where this harness takes 20.72 s, but the harness
  is single-threaded and fills glyph outlines per glyph. Only arm-against-arm on
  one shared ops list is sound.
