# #11 — what `montaget timeline` prints

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

## What this does and does not settle

**Settled:** an overview earns its place — it halves the work at no cost to
correctness, and agents do not over-trust it.

**Not settled:** whether the overview must be *spatial*. The aggregation may be
doing all the work. Deciding that needs more runs per arm than this.

**Consequence for the ticket:** the question "what does `montaget timeline`
print" is the wrong question. The evidence supports *an overview*, is silent on
*an axis*, and ADR-0011 already argues the agent reaches this through `query`.
