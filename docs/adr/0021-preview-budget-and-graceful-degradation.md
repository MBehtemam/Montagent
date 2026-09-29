---
status: accepted
amends: the performance budget stated in the map's Notes (never itself an ADR)
---

# The performance budget splits in two, proxy-resolution previews get a fixed

> **Amended by six later ADRs.** Read them before relying on anything below.
>
> - [ADR-0078](0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)
>   — states how the `<5 s` below is *judged*. **It bounds one attempt, not one
>   invocation**, so a degraded preview may cost up to twice it; a span that runs past its
>   deadline is abandoned where it stands rather than finished and then judged; and the
>   full-resolution escape hatch, being the observational arm, carries no deadline and is
>   never degraded. The mandatory tier disclosure and the ladder's shape are untouched
>
> - [ADR-0046](0046-proxy-preview-target-is-720p-long-edge-capped.md) — states the deferred
>   target resolution and ladder length; defers the floor
> - [ADR-0050](0050-preview-hard-refuses-below-360p.md) — states the deferred floor
> - [ADR-0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md)
> - [ADR-0072](0072-the-render-budget-is-retired-not-replaced.md) — the **render** half,
>   whose replacement this ADR deferred and never wrote. It stays deferred: `render` is
>   observational, has no target, and the measurements taken in its place are recorded
>   rather than enforced. The 60 s-under-2-minutes figure named at the top of this ADR is
>   retired as a ceiling anywhere in the code, and may not be read linearly as a rate
> - [ADR-0077](0077-the-nine-render-readings-are-ratified.md) — states what `render` does with an
>   **odd frame dimension**, which this ADR's *"never silently"* rule forbids one answer to
>   without choosing among the rest: padded to even, right and bottom, in the project's
>   background colour, and disclosed beside the declared frame
> **Amended by [ADR-0093](./0093-renders-world-effects-are-findings-and-an-error-withholds-the-deliverable.md)**,
> which reads this ADR's *"never silently"* at the level of the **deliverable** rather than
> the prose: any `error`-class finding withholds the file, so **a file at the output path is a
> render with zero errors.** `render` can now spend wall clock and produce nothing where it
> used to produce a file. What shipped an eleven-minute silent cut was not an unread report —
> it was a plausible file existing, which is what a human uploads and what an MCP agent
> `stat`s. Proxy degradation is untouched: a preview is not a deliverable and publishes as it
> always has.
>
> **Amended by [ADR-0104](./0104-the-output-path-is-checked-for-a-foreign-deliverable-before-the-encoder-runs.md)**,
> which keeps `render` the only verb that writes the declared `output` and states what happens
> when something else already has: a file another project attested to is refused before the
> encoder runs. `preview`'s own refusal of the previewing project's `output` is unchanged and
> is now joined by a narrower one — a preview may not land on *any* project's attested
> deliverable.
>
> **Amended by [ADR-0095](./0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md)**,
> which states that the **`<500 ms` `frame` budget below does not bind `frame`'s contact-sheet
> range mode**, and adds a third budget to the two this ADR defines. A sheet rasterizes ~18
> frames at ~80 ms each, so one costs ~1.4 s — 2.9× the budget — and inheriting it would put
> the verb in permanent violation. The third number is **observational**, on this ADR's own
> reasoning for full-resolution `preview`: the sheet's tile-width floor already caps the tile
> count, which bounds the time at ~2.4 s without anything enforcing it, and a time miss has no
> remedy left — dropping tiles is forbidden by ADR-0094 and rasterizing smaller saves only
> 6–12%, because decode dominates. **The two-budgets rule below is thereby confirmed rather
> than broken**: this is a third question (*can I see a span at once*), not the same one, and
> the numbers do not move together. `frame`'s `<500 ms` for a single still stands unchanged,
> as does its true-pixel guarantee — sheet tiles are rasterized at true project pixels and
> composited down, so **no proxy ladder enters `frame`.**

# target with a deferred number, and `preview` degrades gracefully

The original budget — a 60 s render under 2 minutes, a 10 s preview under 5
seconds — was written for 1080×1920/30 and never re-derived after
[ADR-0003](0003-general-video-editor-not-channel-tooling.md) generalised the
scope to a CapCut/Premiere-class editor, where 4K is ordinary.
[#34](https://github.com/MBehtemam/Montagent/issues/34) measured what that
omission costs: at 2160×3840, a 10 s preview with a video clip on the timeline
took **19.0 s (`skia-safe`) / 30.0 s (`tiny-skia`)** — both miss the < 5 s
budget by 4–6×, decode only 3.2–3.8 s of it. Settled across three rounds of
independent jurors (Opus, Haiku, Fable), each answering one question at a
time with no visibility into the others' ballots.

## Two budgets, not one

`frame` (a single still, the agent's per-turn self-check) and render/preview
time (a span of video) are tracked **separately**, never collapsed into one
number. [#7](https://github.com/MBehtemam/Montagent/issues/7)'s reviewers
established that `render`-time is not a legitimate per-turn gate — the
agent's dominant self-check is `frame`, untouched by the 4K blowup (28.76 ms
warm at 2160×3840, 0.11–0.27 s cold at 1080p). The two numbers answer
different questions — *is my edit right* vs. *does it look right in motion*
— and #34's own data shows they don't move together: `frame` held flat while
preview time regressed 4–6×. Collapsing them into one budget would erase
that distinction rather than reflect it.

### `frame`: `<500ms` cold, resolution-independent

A hard number, not a qualitative description — a budget nothing can regress
against is not a budget. `<500ms` sits ~2× above the worst cold value
measured anywhere in this project (0.27 s at 1080p), so it will not fire on
noise, and it is stated **independent of resolution**: `frame`'s cost has
never once tracked project size in any measurement taken, which is the
strongest empirical claim available and the most useful one to protect.

**`frame` always renders at true project resolution** — never
proxy-scaled, with no separate flag needed to get true pixels. It measured
fast enough at native 4K that downscaling it buys nothing, and the agent
needs one tool it can unconditionally trust for pixel-accurate checks
(text legibility, edge alignment, sub-pixel drift, a 1px border) without
first asking whether what it's looking at is a lie. If a future feature
(effects, an 8K source) ever pushes `frame` past this budget, that is a
real regression to fix or a number to revise by ADR — not a case for
silently downscaling the one tool built to be trusted.

### GPU rasterization: deferred, not decided

`skia-safe`'s CPU-only prebuilt key (`jpegd-jpege-pdf`, pinned by
[ADR-0009](0009-rust-host.md)/[ADR-0010](0010-skia-safe-rasterizer-text-beside-it.md))
is what every measurement in this project has run against. A separate
`ganesh`/`metal` prebuilt key exists but has never been measured or
prototyped — no one knows the real speedup, the cost of wiring context/surface
management, or whether it's even available in every environment Montagent
must run in (headless CI, remote agents with no GPU). v1's budget is decided
on the CPU-only numbers that exist; GPU is recorded as a live, open lever
for a future ticket, neither ruled out nor blocking this one. The CPU path
remains the compatibility floor regardless of what a GPU measurement later
shows.

## The render/preview budget: proxy resolution, two numbers

**Proxy-resolution preview is adopted**: `preview` renders at a **fixed
target resolution**, applied whenever project resolution exceeds it, rather
than at the project's true resolution. Final export (`render`) always
renders at full declared resolution — proxying only ever applies to
`preview`. The dominant cost driving the 4–6× miss (bilinear resampling of
scaled images/video, ~93% of a frame in earlier 1080p measurements) scales
with pixel count, so a fixed cap is the only strategy that actually bounds
`preview`'s cost regardless of project size — a *scale factor* relative to
project resolution (half of 16K is still 8K) would fail to bound anything at
the high end.

**The exact target resolution is deliberately not stated here.** Writing a
number now — even the seemingly obvious 1080p the original budget was
written for — would be exactly the unmeasured-claim-as-settled-fact pattern
this project has had to walk back before (ADR-0005's frame-alignment
instruction). The dominant cost may be driven by source-decode size and
scale ratio as much as by output pixel count, so the actual recoverable
savings from downscaling to any given target is unknown. A new
research/prototype ticket is graduated to measure it; until it reports, no
downstream decision may cite a specific proxy resolution as settled.

**An explicit full-resolution escape hatch** exists on `preview` (as it
already does on `frame`) for the caller to request true pixels when a check
is precision-sensitive — the fixed proxy target keeps the *default* fast and
bounded, and the escape hatch keeps the *ceiling* on what the agent can
verify from silently dropping to the proxy's sampling.

Two numbers, not one, describe the render/preview budget itself:

- **`<5s` (enforced)** for the common-case proxy-resolution scrub preview —
  the number every caller hits by default.
- **Observational only (unenforced)** for the on-demand full-resolution
  preview: the spec states measured reference examples (e.g. "19.04 s at
  2160×3840/30 fps, 10 s clip, reference hardware") rather than inventing a
  target. The caller explicitly asked for true pixels and accepted the
  cost, so there is no promise for a target to encode; a reference example
  is also a *better* regression baseline than a guessed ceiling — it flags
  a real 2× drift that a loose invented bound would still pass, and it does
  not false-alarm on legitimately heavier input.

## Graceful degradation on a budget miss

When even the proxy-resolution target can't meet the `<5s` scrub-preview
budget (an 8K source, a heavy composite stack), `preview` **auto-degrades to
a lower resolution tier** and the result **mandatorily discloses** which
tier was actually used and that it is a degradation from the default. This
applies only to `preview`, never to `render` — final export must never
silently change what was rendered.

This does not conflict with [ADR-0006](0006-validate-reports-facts-and-render-enforces.md)'s
"report facts, never repair, never stay silent" rule: that rule governs
`validate`, an inspection tool whose entire contract is to describe project
state it must not mutate. `preview` is not inspection — it is already a
disposable, lossy artifact by design, having already conceded fidelity for
speed via the proxy mechanism above. Auto-degrading one tier further and
disclosing it in the result is the same trade-off continuing to do its job,
not a new class of silent repair.

**The specific degradation ladder and its floor are also deferred** to the
same graduated measurement ticket — they are arithmetic on the still-unknown
proxy target (a ladder of "half of the target, then a quarter" cannot be
stated before the target itself is known), so writing specific tier values
now would repeat the same unmeasured-claim mistake being avoided above. What
is decided here is the **shape**: the ladder is small, finite, and strictly
ordered; every step taken is disclosed; and a floor tier exists below which
`preview` hard-fails outright rather than returning a preview too degraded
to make a decision from.

## Consequences

- The map's informal "<5s preview" budget is now two numbers plus a rule,
  not one: `frame` (`<500ms` cold, always true resolution), scrub `preview`
  (`<5s`, at a to-be-measured fixed proxy resolution), and full-resolution
  `preview` (observational only).
- `preview` gains a required resolution-tier field on its result — the
  resolution actually rendered at, and whether it is the default target,
  an explicit full-resolution request, or a degraded tier.
- A new research/prototype ticket is graduated to measure the
  proxy-resolution savings ratio (source-decode size and scale ratio vs.
  output pixel count) across several downscale targets against 4K/8K
  sources, real wall-clock. Until it reports, no ADR may cite a specific
  proxy target resolution, degradation ladder, or floor as settled — this
  ADR states only that they exist and their shape.
- GPU rasterization (`ganesh`/`metal`) remains an open, unmeasured lever for
  a future ticket; it does not block this one and is not ruled out of scope.
