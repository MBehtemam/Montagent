---
status: accepted
amends: 0021 (retires the render half's enforced reading and records the measurements in its place; no preview decision below it changes)
---

# The render budget is retired, not replaced

> **Amended by [ADR-0142](0142-render-has-one-speed-target-the-benchmark-project-in-three-minutes.md)**,
> which gives `render` a target: the benchmark project (6 minutes of 1080p30, three videos
> visible at once) in ≤ 3 minutes median wall clock on the dev's M1 Pro, `RENDER_TARGET` in
> `budget.rs`. It is one absolute number for one project, never a rate, and it is judged by an
> `#[ignore]`d test run on purpose, not by CI. `Budget::Render.limit()` stays `None`, which
> now means *not enforced in CI*, not *no target*. The retired figure stays retired.

[Ticket #217](https://github.com/MBehtemam/Montagent/issues/217), from
[#168](https://github.com/MBehtemam/Montagent/issues/168).
[ADR-0021](0021-preview-budget-and-graceful-degradation.md) opens by naming the
original budget — *"a 60 s render under 2 minutes, a 10 s preview under 5
seconds"* — and says of it: *"written for 1080×1920/30 and never re-derived"*
after [ADR-0003](0003-general-video-editor-not-channel-tooling.md) generalised
the scope to a CapCut/Premiere-class editor where 4K is ordinary. It then
replaces the **preview** half with two numbers and a rule, and leaves the
**render** half's replacement deferred. It states no render budget anywhere.

The code did. This ADR records why it no longer does.

## What happened

The gap ADR-0021 left open was filled three times, by three tickets, none of
them an ADR.

[#189](https://github.com/MBehtemam/Montagent/issues/189) built
`montagent_render::budget` so that *"the verb tickets have somewhere to assert
rather than each inventing a number"*, and gave `render` a rate —
`RENDER_MS_PER_OUTPUT_SECOND = 2_000` — by dividing the retired pair. Its own
doc comment — removed by #217, and readable in the history of
`crates/montagent-render/src/budget.rs` — was honest about what it was doing:
*"This is the one budget stated here as a rate rather than as the pair it was
written as, and that is an assumption worth seeing."*

[#215](https://github.com/MBehtemam/Montagent/issues/215) asserted against it.
The whole committed fixture is 65216 ms, so the rate produced a 130 s ceiling,
and `render_budget.rs` failed the suite if a render missed it.

[#217](https://github.com/MBehtemam/Montagent/issues/217) removed both. The
constant is deleted and nothing judges against it.

**A rate read off a retired point is not a smaller claim than the point — it is
a larger one.** The original was one measurement of one project at one frame
size. Dividing it asserts, additionally, that render cost is linear in output
length and independent of frame size, which is the claim ADR-0003 made
implausible and the reason the point was retired in the first place. It is the
unmeasured-claim-as-settled-fact pattern ADR-0021 refuses by name three
paragraphs below the sentence it was derived from, and the pattern ADR-0005's
frame-alignment instruction had already had to be walked back for.

## The evidence

`render`, the whole committed `en-halloween-decorating` fixture, cold with an
empty probe sidecar ([ADR-0069](0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md)),
release build, `skia-safe` ([ADR-0010](0010-skia-safe-rasterizer-text-beside-it.md)),
M1 Pro. **1080×1920 at 25 fps, 65216 ms of output** — 60 elements, 22 carrying
text, 20 narration elements mixed.

| reading | wall clock | realtime |
| --- | --- | --- |
| #215, the first whole-fixture render | 17.30 s | — |
| #217, slowest of three minutes apart | 19.78 s | 3.36× |
| #217, fastest of three minutes apart | 18.64 s | 3.50× |
| #217, middle of those three | 19.64 s | 3.36× |
| #217, machine compiling at the same time | 22.10 s | 2.98× |

Every render number this project has taken is in that table. Three clean runs
on one machine span **1.06×**; the whole recorded set spans **1.14×**; and one
busy machine turned that into **1.28×** on its own, at conditions no faster
code could have rescued.

## Decision

**`render` is observational. There is no render budget, and this ADR does not
write one.**

`Budget::Render` joins `Budget::FullResolutionPreview` as an arm that records
and never fails — for a different reason. ADR-0021 makes the preview arm
observational because *"the caller explicitly asked for true pixels and accepted
the cost, so there is no promise for a target to encode"*. `render` is
observational because **nobody has measured what the promise should be.** The
asymmetry is deliberate and the two must not be collapsed: one is a decision not
to promise, the other is an open question.

**Measurements are recorded rather than enforced.** `RENDER_REFERENCES` holds
the table above, on the same terms ADR-0021 sets for the preview arm — appended
to, never replaced, since an entry is a record of what was once true on stated
hardware rather than a current expectation. Each entry states the frame size and
rate it was taken at, because ADR-0003 put 4K in ordinary scope and a wall clock
with no frame behind it is comparable to nothing.

**A reference list belongs to one arm and is never read across arms.** A
720p-capped proxy preview ([ADR-0046](0046-proxy-preview-target-is-720p-long-edge-capped.md))
and a render at the declared frame measure different pixel counts of different
things; scoring one against the other's numbers is drift against nothing. This
extends, one axis over, the rule ADR-0021 already implies by keeping the
rasterizer load-bearing on a reference.

**What a replacement would have to carry.** Three things, none of which #217 had
standing to supply, and all three are why this ADR states no target:

1. **Numbers at more than one frame size.** Every reading above is one fixture
   at 1080×1920. A ceiling derived from one frame size reproduces the retired
   figure's exact defect.
2. **A spread worth deriving from.** 1.06× across clean runs and 1.28× with the
   machine busy. A target off a single reading encodes that noise as much as it
   encodes the code.
3. **A statement of what the target is for.** ADR-0021 establishes that `frame`
   and span time answer different questions and that
   [#7](https://github.com/MBehtemam/Montagent/issues/7)'s reviewers found render
   time *"is not a legitimate per-turn gate"*. A render budget that is not the
   agent's loop needs to say whose it is before it can say what it should be.

Until an ADR carries all three, `RENDER_REFERENCES` is the whole of what this
project knows about how long a render takes.

## Consequences

- `RENDER_MS_PER_OUTPUT_SECOND` is gone from the public surface of
  `montagent-render`, along with the 130 s ceiling #215 asserted against.
  `Budget::Render.limit()` is `None` and `is_enforced()` is false.
- `nearest_reference` takes the budget whose references it reads, and
  `Budget::references()` names them. An enforced arm has none: it judges against
  its stated number, and a reference it never consults would be a number with no
  reader.
- **`render` has no regression gate, and nothing added later may become one by
  sitting in the reference list.** `crates/montagent/tests/render_budget.rs`
  asserts that the render succeeded, that it produced exactly the span the
  references were taken over, and that the harness reached `Verdict::Observed`
  rather than a pass against a limit — the last of those being the guard that
  keeps the retired figure from returning as a constant.
- `frame`'s `<500 ms` cold and the scrub preview's `<5 s` are untouched. They
  are the two numbers ADR-0021 actually states, and they remain the two arms
  that are enforced.
- A later ticket that wants a render target should expect to measure across
  frame sizes rather than to re-derive one from this file.

## On the provenance of the numbers above

`docs/agents/domain.md` asks a numeric claim for a *re-executable* check — *"a
script that re-derives the number and exits non-zero the moment it stops
reproducing"*. **The wall clocks in the table cannot meet that bar, and this
section says so rather than letting the reader assume they do.** A measurement
on stated hardware is not reproducible on other hardware; that is what makes it
a record and not a threshold, and it is precisely why nothing here is enforced.
ADR-0021 carries [#34](https://github.com/MBehtemam/Montagent/issues/34)'s
19.04 s the same way.

What *is* committed and re-executable is the part that can rot silently:

- `crates/montagent/tests/render_budget.rs` re-takes the measurement on any
  release run and prints it against the recorded set, so a reader can see
  today's number beside the ones above.
- `the_recorded_spread_is_the_one_the_prose_states`
  (`crates/montagent-render/tests/budget.rs`) re-derives the ends, the 1.14×
  span, and which entry a measurement is actually baselined against, from the
  entries themselves. It fails the moment someone appends a reading and leaves
  a paragraph — here or there — saying the old thing.

The 22.10 s reading is deliberately **not** an entry in `RENDER_REFERENCES`: the
conditions string those entries share says *cold*, and a run competing for cores
did not measure what the other three measured. It is recorded in this ADR
instead, because it is the sharpest evidence in the table for the decision the
ADR makes.
