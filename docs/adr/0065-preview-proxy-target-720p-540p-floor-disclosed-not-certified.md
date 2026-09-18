---
status: accepted
amends: 0021-preview-budget-and-graceful-degradation.md
---

# The proxy target is 720p, the degradation floor is 540p, and the floor is disclosed, not certified

[ADR-0021](0021-preview-budget-and-graceful-degradation.md) adopted proxy-resolution
preview as the mechanism for hitting the render/preview budget above 1080p, but
deliberately left the target resolution, degradation ladder, and floor unstated pending
measurement. [#87](https://github.com/MBehtemam/Montaget/issues/87) ran that measurement
(`docs/research/prototypes/proxy-preview-savings/FINDINGS.md`, on `main`); this ADR
writes the numbers down. Settled by a 3-juror independent court (Opus, Sonnet, Fable),
each given the same draft to attack, blind to the others
(`docs/research/juries/preview-proxy-target/`).

## The target: 720p, on skia-safe

**1080p is not a safe single target.** #87 measured it clearing budget at 4K (4.28s,
14% under) but missing at 8K (5.79s, 16% over) — the same target, the same backend, only
the source got bigger, exactly the failure mode ADR-0021 declined to guess at.

**720p is the smallest single target that clears the `<5s` scrub-preview budget at both
4K and 8K, on `skia-safe`** (2.68s/3.78s) — the load-bearing rasterizer per
[ADR-0009](0009-rust-host.md)/[#34](https://github.com/MBehtemam/Montaget/issues/34). A
size-dependent target (e.g. 1080p below some resolution threshold) was considered and
rejected: it would contradict ADR-0021's fixed-target shape for a quality gain that
1080p's own single-run 8K margin doesn't support keeping around.

**This guarantee is `skia-safe`-specific, and `tiny-skia` is explicitly not certified at
8K/720p.** #87 measured `tiny-skia` at 4.91s against the 5s budget at 8K/720p — a 1.8%
margin, on a single run with no repeats, which `FINDINGS.md` itself states is "too thin
to certify as safe without more samples." All three jurors independently flagged an
earlier draft of this ADR for citing that thin margin as *supporting* evidence when the
source measurement states it as a refusal to certify. `tiny-skia` remains the named exit
if `skia-safe`'s prebuilt story ever breaks (per #7/ADR-0010), never a co-equal target
for this budget; if it is ever load-bearing, this margin needs its own re-measurement
with repeated samples before being trusted.

## The ladder: one degrade step, to 540p, then hard fail

**The ladder is `720p` (default) → `540p` (the one degrade tier on a miss) → hard-fail
below.** No `360p` tier. #87's savings curve shows almost all recoverable time is bought
by the first step (native → first-clearing-target); at both sizes, `720p → 540p → 360p`
combined buys under 1s (4K) / ~1.06s (8K) — measurable, not load-bearing, for a
mechanism whose whole point is a large win.

**One degrade step does not always rescue a miss, and that is by design, not an
oversight.** `720p → 540p` buys 1.05s at 4K but only **0.71s at 8K** — because 8K's
per-frame cost at small targets is decode-bound (decode is driven by *source* size, not
*target* size, and does not shrink with further downscaling the way resample cost does).
A project whose 8K/720p render misses budget by more than ~0.7s cannot be rescued by
this ladder at all; it lands on the hard-fail floor. The floor exists precisely for this
case — a ladder that always claimed to rescue every miss would be asserting a capacity
`FINDINGS.md`'s own decode numbers don't support.

## The floor is disclosed, not certified

**540p is a wall-clock floor. Its legibility is explicitly unmeasured, and this ADR does
not assert it is fine to read.** `FINDINGS.md` states plainly that "the *time* floor and
the *visual* floor are different questions; only the time floor is answered here" and
that the visual floor "is very likely the binding one in practice" — a judgment about
360p in the source text that generalizes to any tier below native, including 540p.

An earlier draft of this ADR claimed 540p's overlay text is "roughly double" 360p's,
reasoning from "540p is 1.5x the linear scale of 360p" — an internal contradiction
independently caught by all three jurors. Corrected: `FINDINGS.md`'s fixture measured
~19–27px overlay text at 360p (from a 57–80px 1080p-equivalent card); at 1.5x linear
scale, **540p yields ~28–40px, not ~38–54px**. Both figures are properties of #87's own
synthetic fixture's overlay sizing, not a general claim about every Montaget project's
caption size — a project with proportionally smaller captions reaches the
`FINDINGS.md`-flagged risky range at a higher tier than this fixture did.

Given that, **540p ships as the floor without a dedicated legibility pass — but only
because ADR-0021 already built the mechanism this uncertainty needs**: `preview`'s
result carries a mandatory disclosure of which tier was actually used, and an explicit
full-resolution escape hatch exists for any caller for whom precision matters. This ADR
is not certifying "540p is legible" as settled fact (the unmeasured-claim-as-fact
mistake ADR-0005 and ADR-0021 both name); it is choosing a floor by wall-clock evidence
and relying on disclosure, not certification, to cover what wasn't measured. **Any tier
below 540p (e.g. a future `360p`) requires an actual human-legibility pass before being
added** — this ADR settles nothing about it either way, deferred exactly as ADR-0021
deferred the whole ladder pending #87.

The court split 2–1 on this question. Two jurors (Opus, Fable) accepted 540p shipping
on this disclosed-not-certified framing. The third (Sonnet) held that no floor should be
certified without at least a lightweight legibility spot-check run first, and that
absent one, the ladder should stop at 720p with a hard fail below it. The majority
framing is adopted because it does not, in fact, certify legibility — it treats 540p as
the last wall-clock-justified tier before hard failure, discloses it every time, and
leaves the caller free to request true pixels. If a future report shows real projects
hitting an illegible 540p preview in practice, that is grounds to lower the floor to
720p by amendment, not evidence this ADR got the current call wrong.

## The composite-stack scope gap gets its own ticket, not a footnote

ADR-0021 names exactly two triggering conditions for `preview` degradation: an 8K
source, and **a heavy composite stack**. #87's harness measured one video clip, one Ken
Burns still, and one overlay block — never multiple simultaneous clips or effects — and
says outright that a heavier stack "would raise the raster (not decode) side of the
budget further" and was out of scope.

An earlier draft of this ADR treated that gap the way ADR-0021 treated GPU
rasterization: real, unmeasured, non-blocking, open for a future ticket if it's ever
shown to matter. Two of three jurors (Opus, Sonnet) rejected that analogy: GPU is an
unmeasured *upside* lever nobody has promised — not measuring it can only mean the
system is faster than advertised. The composite-stack gap is unmeasured *risk* against
an *enforced* budget that ADR-0021 itself already named as a trigger condition; treating
it as equivalent to an optional future lever misstates what's actually unverified.
Majority governs: **[#159](https://github.com/MBehtemam/Montaget/issues/159) is filed
as part of landing this decision**, not deferred until a real project is shown to miss
the floor.

## Consequences

- `preview`'s scrub budget (`<5s`) targets `720p` by default, on `skia-safe`. This
  guarantee does not extend to `tiny-skia` at 8K/720p, which is unmeasured beyond a
  single thin-margin run.
- The degradation ladder is exactly `720p → 540p → hard fail`. A single degrade step
  is not guaranteed to rescue every miss — an 8K miss larger than ~0.7s cannot be
  rescued by this ladder at all.
- `540p` is a wall-clock floor, disclosed like every other degraded tier per ADR-0021's
  existing mandatory-disclosure rule. Its legibility is explicitly unmeasured and not
  asserted as fact. No tier below `540p` may be added without a human-legibility pass.
- [#159](https://github.com/MBehtemam/Montaget/issues/159) is graduated to measure the
  heavy-composite-stack case ADR-0021 named as a trigger and #87's harness did not
  cover — filed now, not deferred.
- GPU rasterization remains, as ADR-0021 stated, an open and unmeasured lever for a
  future ticket; nothing here changes that.
