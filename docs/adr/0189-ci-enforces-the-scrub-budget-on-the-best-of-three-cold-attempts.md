---
status: accepted
amends: 0065 (the `<5 s` scrub budget at the 720p target is unchanged; CI now enforces it on the best of up to three cold attempts of the same preview, not on one)
---

# CI enforces the scrub budget on the best of three cold attempts

> **Amended by [ADR-0191](0191-the-scrub-budget-test-is-not-run-on-x64-windows-until-its-cost-there-is-understood.md).**
> Best of three still stands on four of the five tested legs. On x86_64 Windows it was not
> enough, and the test is ignored there until the preview's cost on that leg is understood.

The owner approved this decision.

[ADR-0021](0021-preview-budget-and-graceful-degradation.md) makes `<5 s` the enforced number for
the common-case scrub preview, and
[ADR-0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md) sets the 720p
target it is stated for. `crates/montagent/tests/preview_budget.rs` enforces it on every CI leg
with one cold `montagent preview` of the first 10 s of the committed fixture.

## The claim is unchanged

`SCRUB_PREVIEW_LIMIT` stays at 5 s. The span is still 10 s, the target is still 720p, and the
attempt must still come back undegraded. What changed is how many attempts CI may take to
see the preview meet that claim. The verb is not changed: `preview` still judges each rung
on its own clock and still degrades to 540p when a rung misses
([ADR-0078](0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)).
This ADR is only about the test.

## The test takes up to three attempts

The test runs the same preview up to three times. Each attempt is a fresh process with its own
empty scratch directory and its own empty cache directory, so each is as cold as the single run
was. It stops at the first attempt that is inside 5 s **and** at the 720p tier **and** not
degraded. It fails only if none of the three is. On a failure it prints every attempt's time,
tier and verdict.

Two checks do not depend on the clock, and they still bind every attempt rather than the best
one: `preview` must answer successfully, and it must report a 10 s span.

## Why

On ADR-0186's code the preview takes about 5.0 s on shared runners, against the 5 s limit. Its
cost did not change between the runs below. The legs still passed on some runs and missed by
about 0.1 s on others. Load on a shared runner can only make a run slower, never faster, so the
fastest of several runs is the best estimate of what the code costs. A single run measures the
code plus whatever else the runner was doing.

The CI runs, on commits that did not touch the preview path:

| Run | `x86_64-unknown-linux-gnu` | `x86_64-pc-windows-msvc` | the three `aarch64` legs |
|---|---|---|---|
| [37926487139](https://github.com/MBehtemam/Montagent/actions/runs/37926487139) | pass | **`preview_budget` failed** (with `letter_spacing`, `painters`) | pass |
| [37933711391](https://github.com/MBehtemam/Montagent/actions/runs/37933711391) | **`preview_budget` failed**, the only failed target | **`preview_budget` failed**, the only failed target | pass |
| [37937482651](https://github.com/MBehtemam/Montagent/actions/runs/37937482651) | pass | `preview_budget` passed (`letter_spacing`, `painters` failed) | pass |
| [37946793784](https://github.com/MBehtemam/Montagent/actions/runs/37946793784) | **`preview_budget` failed**, the only failed target | cancelled | pass |

The failing target on each leg is from the job log's `N targets failed` summary. The same
test on the same leg passed on one run and failed on the next. The arm legs passed in these
four runs, but at about 5.0 s they have the same small margin, and they have missed by about
0.1 s on other runs.

## What a regression must do to be caught

It must make **every one of the three attempts** miss 5 s at 720p, or make every attempt
degrade. A change to the paint path, decode or encode slows every run, so it is still caught.
#552's trilinear read, which took the scrub from about 5.6 s to about 9.0 s, would have failed
all three. What is lost is the ability to catch a slowdown that is smaller than the run-to-run
noise **and** leaves the fastest of three attempts inside the budget. A preview that sits right
at the line can drift up by about that much before CI notices.

The decision itself is tested on synthetic timings in the same file (`mod best_of_three`), so
it runs in every build, debug included:

- `[5.4, 5.3, 5.2]` at 720p fails, and all three attempts are taken.
- `[5.4, 4.7, 5.3]` passes on 4.7 s, and stops after the second attempt.
- A slowdown on every attempt fails, both a large one (`[7.9, 7.6, 8.1]`) and a small one
  (`[5.05, 5.02, 5.08]`).
- A fast attempt that was bought by degrading (`3.1 s at 540p`) is not a pass.
- A pass on the first attempt costs one run, and there is never a fourth attempt.
- Exactly 5.0 s is inside the limit, as `Budget::judge` has always read it.

## Cost

The common case costs one run, as before, because the test stops at the first pass. The worst
case is three runs of about 5 s each, plus process start-up, on a leg that is failing or
nearly failing.

## Not decided here

`x86_64-pc-windows-msvc` is the marginal leg. `preview_budget` failed there in two of the three
runs above that finished, and the leg is also red for other tests (`letter_spacing`,
`painters`; #869 is looking at its MP4 bytes). If its preview is slower than 5 s on every
attempt, best of three will not save it, and the leg stays red. A separate allowance for one platform would be a new owner decision, and it is not
taken here. Intel macOS left the suite under
[ADR-0188](0188-intel-macos-is-shipped-and-built-in-ci-but-is-no-longer-a-tier-1-test-target.md)
for the same kind of miss.
