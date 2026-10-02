# Skills eval: the second verdict's rubric and decision rule

Pre-registered: committed before any second-verdict run, and `verdict.py` refuses a verdict
whose runs started before this file was first committed. The phase is `verdict-2`: runs in
`runs/verdict-2/`, judging in `judging/verdict-2/`. Changing anything below after the first
`verdict-2` run voids that verdict.

The first verdict ([Verdict runs: every arm on the held-out briefs, judged blind](https://github.com/MBehtemam/Montagent/issues/549))
was run under `RUBRIC.md`, which stays frozen as its record. It passed parity and failed lift,
and [The verdict failed lift: what does the skills map do now?](https://github.com/MBehtemam/Montagent/issues/577)
read that as a null and set three changes: the end-card ducking fixed first, new held-out
briefs that leave choices open, and 5 runs per Montagent arm from the start. This rubric
records those and the judging decided in
[Second verdict rubric: 5 runs per arm, judging that stays bearable, and what a second fail means](https://github.com/MBehtemam/Montagent/issues/580).

## What changed from `RUBRIC.md`

| | First verdict | Second verdict |
|---|---|---|
| Runs per Montagent arm | 3, up to 5 after a top-up | 5 from the start; no top-up |
| Briefs | `briefs/held-out.sha256` | new briefs sealed in `briefs/held-out-2.sha256`, leaving structure, timing and craft open |
| Lift pairs | every with-skills run against every no-skills run (9 per brief) | a balanced sample fixed in advance: 10 of the 25 per brief |
| Sittings | one, all briefs mixed | one per brief |
| Reported, never decisive | — | per-brief and per-run tallies |
| A second lift fail | — | the lift claim is dropped; the skills ship on parity alone |
| The court | recorded | recorded, and a juror under 60% agreement with the human is retired |
| Pins | `harness/pins.json` | `harness/pins-v2.json`, on a commit that includes the end-card fix |

## The question

Does Opus 5.5 using Montagent **with** the end-user skills make motion graphics

- **comparable** to what it makes today **without Montagent** (parity), and
- **better** than Montagent **without the skills** (lift)?

Both must hold, so that the parity comes from the skills rather than from the product alone.

## Arms

Every arm gets the same brief, the same asset pack and the same prompt wrapper, on the same
model, client, caps and machine.

| Arm | Montagent | Skills | Runs per brief |
|---|---|---|---|
| `reference` | no (not on PATH, no MCP server) | no | 1 |
| `no-skills` | MCP server connected, CLI on PATH | no | 5 |
| `with-skills` | MCP server connected, CLI on PATH | the pinned commit's `skills/`, installed in `.claude/skills/` | 5 |

Installing the skills is the only difference between `no-skills` and `with-skills`. The run
count is fixed: there is no top-up and no further runs, whatever the ballots show.

## Pins and isolation

`harness/pins-v2.json` is part of this rubric. It fixes:

- the model (`claude-opus-5-5`), the effort level, and the Claude Code version (the harness
  refuses to run under any other)
- the caps: wall-clock and spend (Claude Code's `--max-budget-usd`), identical in every arm
- the prompt wrapper, verbatim, including the one line that differs between the reference
  and the Montagent arms (which tool to use)
- the network: WebFetch and WebSearch are removed; Bash runs in Claude Code's sandbox, which
  reaches only npm, PyPI and Playwright's download hosts, so the reference can install a
  renderer the way it would today, and nothing can look anything up
- the built-in skills a fresh Claude Code session carries, which every arm has
- the run count, the lift sample, the court's jurors, how many stills they see, and the
  agreement below which a juror is retired

One Montagent commit pins the binary (built from that commit), the skills (that commit's
`skills/`) and the asset pack (that commit's `assets/`). Every `verdict-2` run uses the same
commit, and it must contain `560333d2`
([Fix the end-card music level in montagent-footage](https://github.com/MBehtemam/Montagent/issues/578)),
the fix to the defect the first verdict was confounded by.

Each run starts in a fresh directory outside the repo that holds only the pack. Claude Code
loads only that directory's settings and only the Montagent MCP server: no user skills,
plugins, settings or MCP servers, and auto-memory starts empty because it is keyed to the new
directory. The harness refuses to run while a global `~/.claude/CLAUDE.md` exists. The run's
own transcript records what the session loaded, and any deviation from its arm's setup is an
**isolation problem**. Runs never overlap: Claude Code's sandbox gives every session the same
per-user temp directory, so the harness holds a lock for the length of a run and moves
whatever the run left in that directory into the run's own scratch afterwards.
`harness/run_arm.py check` proves the sandbox (web blocked, registries reachable, home not
writable) and is run before the verdict runs.

## Briefs

Three new **held-out** briefs, one for each job the first verdict covered:

- an ad for Montagent
- a talking-head cut on a presenter take
- a short for the owl character

Each brief states outcomes and a per-beat acceptance checklist, but leaves structure, timing
and craft choices open, so that runs can differ in ways a viewer can see. A fresh agent
writes them from the committed pack, and the session that seals them never reads them. Their
SHA-256 hashes are committed in `briefs/held-out-2.sha256` before any `verdict-2` run, and
the harness refuses a `verdict-2` run on a brief whose hash is not there. The human keeps the
briefs outside the repo until the runs. The first verdict's held-out briefs are unblinded
and are not used.

## What is judged

Each run's deliverable, `deliverable.mp4`, re-encoded by the harness to 720p (short side) as
`render.mp4`. The re-encode is identical for every arm. Nothing else is judged: not the
source, not the transcript, not the medium.

**The human (decisive)** judges blind pairs on the judging page (`harness/judge.py`).

- **Parity pairs:** the reference against each with-skills run of the same brief, 5 per
  brief.
- **Lift pairs: a balanced sample.** Per brief, once all its runs are recorded, the pairing
  script puts the with-skills runs in a random order W₀…W₄ and, independently, the
  no-skills runs in a random order N₀…N₄ (both drawn with the system's secure random
  source). Wᵢ is paired with Nᵢ and N₍ᵢ₊₁₎ mod 5. That makes 10 lift pairs per brief, and
  every Montagent run appears in exactly 2 of them. The other 15 pairs are never judged.
- Every run is shown under a random name, and each pair in random left/right order.
- **One sitting per brief.** The three briefs are judged in a random order, one sitting
  each, with that brief's 15 pairs in random order. The key stays unopened until all three
  sittings are done.
- Per pair: **left better**, **right better**, or **equal**.
- The question: *which is the better finished piece for this brief?* Judge the whole video:
  first whether it delivers the brief's beats and acceptance checklist, then craft (timing,
  motion, type, composition, polish) and visible defects. Choose **equal** when you cannot
  honestly prefer one.

With 3 briefs that is 3 × (5 + 10) = 45 pairs, in three sittings of 15.

**The court (recorded, never decisive):** each juror in `pins-v2.json` judges the same pairs,
independently, from a sheet of stills at fixed instants, `(k + 0.5) / 8` of the brief's
declared length for k = 0…7, left clip above right, with the same three choices and the same
question. This is unchanged from the first verdict, so that the court's agreement with the
human can be compared across the two. Its ballots are committed verbatim. Where it disagrees
with the human, the disagreement is reported, not resolved.

**Retiring a juror.** The verdict reports each juror's agreement with the human: the share of
pairs the human judged on which the juror cast the same ballot. A juror whose agreement is
**under 60%** is retired, and later verdicts do not convene it. If every juror is retired,
the court is not convened again.

## Edge cases, decided now

- **No deliverable.** A run that leaves no decodable `deliverable.mp4` loses every one of its
  pairs, automatically, without being shown. Two such runs in one pair are **equal**. This
  applies to the sampled pairs only.
- **A cap hit.** A run stopped by the wall-clock or spend cap is judged on whatever it
  delivered, and scored as above if it delivered nothing.
- **Skills that do not trigger.** A with-skills run whose transcript shows no skill triggered
  still counts as a with-skills run: failing to trigger is the skills' failure. It is
  reported.
- **An isolation problem** is the harness's failure, not the agent's. The run is voided: its
  directory moves to `runs/verdict-2-void/` with a note of the problem, and the run is
  repeated before pairing. An API error (a session or rate limit) that stops a run is an
  isolation problem, and the batch stops at the first one. `verdict.py` reports any isolation
  problem among the counted runs, and such a verdict is `invalid`.
- **A ballot that is not left, right or equal** does not count; the pair stays unjudged.
- **A sitting cut short.** It may be finished later. Ballots already cast stand, and the key
  stays unopened until every pair is judged.

## Decision rule

Over the human's ballots and the automatic decisions:

- **Parity passes** when the reference is picked as better in **at most 40%** of parity pairs.
- **Lift passes** when with-skills **wins more** lift pairs than no-skills wins. Equal votes
  count for neither side.

The verdict **passes** only if both pass. There is no top-up and no margin: the rule is
applied once, as it falls.

**If lift fails again**, the lift claim is dropped: the skills ship on parity alone, and they
are not described as making better videos than Montagent without them. There is no third
lift verdict on this map.

## Reported, never decisive

- **Per brief:** parity (reference better / with-skills better / equal) and lift (with-skills
  wins / no-skills wins / equal), so a reader can see whether the result rests on one brief.
- **Per run:** each Montagent run's wins, losses and equal votes across its lift pairs, and
  each with-skills run's parity ballot, so a reader can see whether it rests on one run.
- **The court:** each juror's tally and its agreement with the human.

For every run, `verdict.py` re-derives from the committed files and reports:

- **Reach:** `query --census type` over every element of the final project.
- **Checks:** `validate` counts by class, and the finding codes.
- **Process:** Montagent verbs used, `frame`/`preview` loops, and whether the run looked at
  its own pictures (a look verb, or reading an image file) or delivered one-shot.
- **Cost:** turns, wall-clock, tokens and spend. Cost is recorded only: it is not part of the
  parity or the lift claim.
- **Delivery:** whether a video was delivered, and its length and size. Also whether it may
  not be the agent's last word: background tasks killed when the session ended (from the
  transcript), and the run-time `delivery_warnings` (a render's partial file left behind,
  or a project written after the deliverable). These are recorded and never decisive: the
  video judged is the one delivered.
- Whether the with-skills run's skills triggered.

## Evidence

Everything a verdict rests on is committed under `docs/research/skills-eval/`: the runs
(`runs/verdict-2/`), committed before judging starts; the pairs, the sample and the key; both
sets of ballots; the court's stills; and `judging/verdict-2/verdict.json`. `verdict.py`
re-derives the signals and the tally from them and exits non-zero if the committed verdict no
longer follows.
