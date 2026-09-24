# #11 — what `montagent timeline` prints

Two rounds. **The first was contaminated and its headline result is void.** It is
kept here because the contamination is the finding most worth not repeating.

## Round 1 — void

Five agents (Opus, Sonnet, Haiku, Fable, a briefed-hostile seat) each did four
real tasks against the fixture, then judged four candidate renderings. They
concluded 5/5 that a wide time axis is "a human affordance being assumed onto
agents."

**That verdict does not stand.** The packet handed to every juror included
ADR-0011's surface table, whose line 32 reads:

    | `timeline` | ❌ | ✅ | the human's wide view |

The conclusion under test was in the jurors' own reading list, marked `accepted`.
Part 2's question 4 then asked whether the view was "a human affordance being
assumed onto you" — a loaded disjunction — and offered "do not build it" as a
first-class option with no symmetric reassurance that "yes, build it" was equally
respectable. Two of the four tasks were also purpose-built for `query` and
`shift`, verbs that already exist.

What survives from round 1 is only what came out of the *work*, not the opinions:
the format findings below, and the candidate quality ranking (C > B > A > D).

## Round 2 — the measurement

No ADRs in the packet, no mention of `timeline`, no mention that a tool was under
evaluation, no self-report. Nine runs, one model (Sonnet), three conditions,
five questions with ground truth computed *before* the questions were written
(`rerun/GROUND-TRUTH.txt`):

- **X** — `project.json` and ordinary tools only.
- **Y1** — same, plus a *spatial* overview: axis, bars, ruler.
- **Y2** — same, plus the *identical facts* as flat text.

Y1 and Y2 are emitted by one script (`conditions.py`) from the same variables, so
the only difference is whether the facts sit on an axis. That is the ablation.

### Result

| condition | tool calls | mean |
| --- | --- | --- |
| X — raw file | 10, 10, 13 | 11.0 |
| Y1 — spatial | 4, 7, 7 | **6.0** |
| Y2 — flat | 7, 8, 11 | 8.7 |

**Correctness was 5/5 in all nine runs.** The overview never changed an answer.

- **X vs Y1 — disjoint ranges.** An overview roughly halves the work.
- **Y1 vs Y2 — overlapping, inconclusive.** Whether the *bar* beats the same
  numbers printed flat is **not established**. n=3 per arm; Y2c took 11 calls,
  more than two of the three controls.

### The over-trust hypothesis is dead

Q5 was built to punish the spatial view: it asks for the narration track's gap
count and smallest gap, and **six of those gaps are smaller than Y1's resolution
of 679 ms per character** — physically unrepresentable in the bars, explicit in
Y2's list. All nine runs answered 19 gaps / 450 ms correctly. Nobody trusted the
picture where the picture could not resolve the answer.

### Self-reported effort is unreliable

Runs were asked to report their own tool-call count. Three misreported: Y2a said
6 (actual 8), Y2c said 11 (actual 11, after visibly recounting), X3 said 8
(actual 13). All scoring here uses harness-recorded counts. This is the same
failure mode as round 1's self-reported preferences, and is the reason round 2
measures behaviour instead of asking for it.

## What round 2 does and does not settle

**Settled:** an overview earns its place — it halves the work at no cost to
correctness, and agents do not over-trust it.

**Not settled:** whether the overview must be *spatial*. The aggregation may be
doing all the work. Deciding that needs more runs per arm than this.

**Consequence for the ticket:** the question "what does `montagent timeline`
print" is the wrong question. The evidence supports *an overview*, is silent on
*an axis*, and ADR-0011 already argues the agent reaches this through `query`.

## Round 3 — raising n per arm ([#63](https://github.com/MBehtemam/Montagent/issues/63))

Three more runs per arm, same model (Sonnet), same blinding (no ADRs, no
mention of `timeline` or an evaluation), same materials — `view-Y1.txt` and
`view-Y2.txt` unchanged, one shared `project.json`. Tool counts are
harness-recorded (`<usage>` block), never self-reported, per round 2's finding
that self-report is unreliable.

| condition | round 2 | round 3 | combined n=6 | mean |
| --- | --- | --- | --- | --- |
| Y1 — spatial | 4, 7, 7 | 6, 10, 12 | 4, 6, 7, 7, 10, 12 | 7.7 |
| Y2 — flat | 7, 8, 11 | 8, 11, 11 | 7, 8, 8, 11, 11, 11 | 9.3 |

**Correctness stayed 5/5 across all 18 answers in the six new runs** (30/30
including round 2) — the fourteenth and fifteenth consecutive perfect scores on
this task, whatever the view. The over-trust hypothesis stays dead.

**The ranges did not separate. They converged.** Y1's three new runs (6, 10,
12) are all *higher* than its round-2 mean, and its new maximum (12) now
exceeds Y2's maximum (11) outright — the arm predicted to be cheaper produced
this round's single most expensive run. Mann-Whitney U on the combined n=6 per
arm gives U=10 (`rerun/score.py`), against a critical value of 5 at α=.05,
two-tailed, for n=6,6 — not significant, and the direction of travel is toward
the null, not away from it. Doubling the sample did not sharpen the round-2
trend; it erased it.

**Settled: the effect is ruled out, not merely unproven.** Two independent
signals point the same way and neither existed at n=3: the ranges now overlap
by more than half their span, and the arm carrying the spatial axis produced
the outlier on the wrong side. The most defensible reading is round 1's
original suspicion, corrected of its contamination: **the aggregation is doing
the work; the axis is doing nothing measurable.** [ADR-0031](../../adr/0031-timeline-overview-is-not-required-to-be-spatial.md)
records the consequence.
