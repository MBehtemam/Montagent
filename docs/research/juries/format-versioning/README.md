# Format versioning and migration — evidence for #14

Everything [ADR-0016](../../../adr/0016-no-format-version-the-unknown-key-error-is-the-mechanism.md)
rests on. `versioning_scan.py` re-derives every number the ADR claims and exits
non-zero if any stops reproducing:

```
python3 docs/research/juries/format-versioning/versioning_scan.py
```

Twenty-seven agent sessions across four rounds: three rounds of opinion, then two
controlled experiments when opinion stopped moving.

| | what | who |
| --- | --- | --- |
| `round-1/` | first jury — four models plus an adversarial seat | 5 |
| `round-2/` | splits returned unattributed, each juror attacking its own answer first | 5 |
| `round-3-distribution/` | is the stale-binary population non-empty? settled from the ADRs | 2 |
| `experiment-gravity-fork/` | does a retired-spelling glossary entry help an agent migrate? | 6 |
| `experiment-three-arm/` | does a version number beat a good error message? | 9 |

`briefs/` holds what the jurors were given. Read `round-2-brief.md`'s corrections
section first: it lists four things round 1 got wrong, three of them errors by
the convener, and round 2's verdicts move because of them.

## What the experiments measured

**The gravity fork.** Six agents repaired a stale project file given only
`montagent validate` output. The file is the real pre-ADR-0015 fixture with two of
its eight `gravity` values flipped `top` -> `bottom`, which is what makes the
trap detectable: on the six inert elements deleting the key is correct, and on
the two non-inert ones the rect must move to `y = 1300 - 1912 = -612` or the
video silently shows the wrong 1300 px. The same error message fires on all
eight. Half the agents also got `CONTEXT.md`'s `Gravity` glossary entry.

Result: **2/3 correct without the glossary entry, 0/3 with it.** The controlled
pairs are clean — the same model solved it without the entry and failed with it,
one of them computing `-612` explicitly before rejecting it on the entry's
"copied, not authored" framing. The entry describes a retirement where the field
*happened* to be inert and reads as a general rule that the value never mattered.

**The three arms.** Nine agents, a stale binary (`montagent 0.9.2`) meeting the
current fixture, under deadline pressure with no network and nobody to ask. Arm A
carried a version number and a "your file is newer, upgrade the binary" refusal;
arm B had no number and an unknown-key error naming the binary as the suspect;
arm C had no number and a bare `unknown key` list. Only the number and the
message text varied.

Result: **nobody deleted the unknown keys** — 0/9, not even in the bare control.
Four agents independently invented the same repair (an opaque cream rect between
the photo track and the cards). The only two damaged files came from the two
paths offering a single-token escape: lowering the version integer (arm A) and
retyping one `height` (arm C). Arms B and C were indistinguishable.

## Setup flaws, recorded rather than glossed

- The round-2 brief's C1 cited "~5.8% of source widths" from a formula the
  convener mis-specified; a juror reproduced it from the brief rather than from
  the ADR, so the apparent independent confirmation was an echo. **Withdrawn** —
  the project's re-derivable figure is ADR-0013's 4.466% of 31,402,800
  combinations, and the scan checks that instead.
- The first gravity-fork scorer read `y` and ignored `origin`, mis-scoring an
  agent that wrote `y:1300, origin:"bottom-left"` — the same rect by another
  route. The convener then built a "blank frame" reading on the mis-score.
  Both retracted; the scan now computes the visible source band.
- One round-1 juror reported fabricated blob hashes while correctly reproducing
  the *shape* of the finding, and misattributed the `gravity` retirement to
  ADR-0013. Its Part A is treated as unrun. This is why the scan exists.
- Two agents' harnesses blocked them from writing `REPORT.md`; those reports were
  returned as final messages and are transcribed with a provenance note. The
  shipped `.json` beside each is what the agent actually left on disk, so no
  finding rests on a transcription.

## Found along the way, outside this ticket

**3 of 9 three-arm agents pretty-printed the whole file** (154 lines -> 1146,
1165, 1243), destroying the one-element-per-line convention ADR-0005 makes
load-bearing for exact-string replace. Nothing forbids it. That is
[#73](https://github.com/MBehtemam/Montagent/issues/73)'s premise, previously a
single incident in #12, now independently reproduced three times.
