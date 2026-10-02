# The second verdict: closed as inconclusive

There is no `verdict.json` for `verdict-2`, and no pass or fail. The decisive input under
`RUBRIC-v2.md` is the human's ballots on 45 blind pairs, and neither judging pass produced
ballots that could be judgements of the videos:

| Pass | Ballots | Span | Median gap between ballots |
|---|---|---|---|
| First (`ballots/human-void.json`, see `human-void.md`) | 45 | 3 min 59 s | 3 s |
| Second (`ballots/human.json`) | 45 | 1 min 49 s | 0 s |

Each pair is two clips of roughly 15–60 s. The human chose not to judge a third time, and
the two passes are kept as they were cast. Neither was tallied, and `key.json` and
`sample.json` were not opened before this decision, so a reader who judges the pairs later
starts blind.

## What follows

The rubric's reading of a second lift fail applies: **the lift claim is dropped.** The skills
ship on parity alone, from the first verdict
([Verdict runs: every arm on the held-out briefs, judged blind](https://github.com/MBehtemam/Montagent/issues/549)),
and are not described as making better videos than Montagent without them. There is no
third lift verdict on this map.

## What stays usable

- The 33 runs in `runs/verdict-2/`, all at `88024191`, all delivered, none with an isolation
  problem ($79.92). One run voided by a session limit is in `runs/verdict-2-void/`.
- The pairing (`pairs.json`, the cyclic sample, `key.json`) and `sittings.json`.
- The court's ballots and stills (Opus 5.5, Sonnet 5.5, Haiku 4.5; recorded, never decisive).

Anyone can judge the pairs later with `harness/judge.py --phase verdict-2` after moving the
second pass aside, then run `verdict.py --phase verdict-2 --write`. Until then,
`verdict.py --phase verdict-2` exits non-zero because there is no committed verdict, by design.
