# Skills eval: rubric and decision rule

Pre-registered: committed before any verdict run, and `verdict.py` refuses a verdict whose
runs started before this file was first committed. It implements the eval designed in
[Eval design: measuring a Montagent ad built with and without skills](https://github.com/MBehtemam/Montagent/issues/446).
Changing anything below after the first verdict run voids that verdict.

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
| `no-skills` | MCP server connected, CLI on PATH | no | 3 (5 after a top-up) |
| `with-skills` | MCP server connected, CLI on PATH | the pinned commit's `skills/`, installed in `.claude/skills/` | 3 (5 after a top-up) |

Installing the skills is the only difference between `no-skills` and `with-skills`.

## Pins and isolation

`harness/pins.json` is part of this rubric. It fixes:

- the model (`claude-opus-5-5`), the effort level, and the Claude Code version (the harness
  refuses to run under any other)
- the caps: wall-clock and spend (Claude Code's `--max-budget-usd`), identical in every arm
- the prompt wrapper, verbatim, including the one line that differs between the reference
  and the Montagent arms (which tool to use)
- the network: WebFetch and WebSearch are removed; Bash runs in Claude Code's sandbox, which
  reaches only npm, PyPI and Playwright's download hosts, so the reference can install a
  renderer the way it would today, and nothing can look anything up
- the built-in skills a fresh Claude Code session carries, which every arm has
- the court's jurors and how many stills they see

One Montagent commit pins the binary (built from that commit), the skills (that commit's
`skills/`) and the asset pack (that commit's `assets/`). Every verdict run uses the same
commit.

Each run starts in a fresh directory outside the repo that holds only the pack. Claude Code
loads only that directory's settings and only the Montagent MCP server: no user skills,
plugins, settings or MCP servers, and auto-memory starts empty because it is keyed to the new
directory. The harness refuses to run while a global `~/.claude/CLAUDE.md` exists. The run's
own transcript records what the session loaded, and any deviation from its arm's setup is an
**isolation problem**. Runs never overlap: Claude Code's sandbox gives every session the same
per-user temp directory, so the harness holds a lock for the length of a run and moves
whatever the run left in that directory into the run's own scratch afterwards. `harness/run_arm.py check` proves the sandbox (web blocked, registries
reachable, home not writable) and is run before the verdict runs.

## Briefs

The verdict runs on the three **held-out** briefs only. Their SHA-256 hashes are committed in
`briefs/held-out.sha256` before any verdict run, and the harness refuses a verdict run on a
brief whose hash is not there. The development briefs (`briefs/dev/`) are for the baseline,
which is exploratory and never decides the verdict.

## What is judged

Each run's deliverable, `deliverable.mp4`, re-encoded by the harness to 720p (short side) as
`render.mp4`. The re-encode is identical for every arm. Nothing else is judged: not the
source, not the transcript, not the medium.

**The human (decisive)** judges blind pairs on the judging page (`harness/judge.py`):

- **Parity pairs:** the reference against each with-skills run of the same brief.
- **Lift pairs:** every with-skills run against every no-skills run of the same brief.
- Every run is shown under a random name, each pair in random left/right order, and all
  pairs mixed across kinds and briefs in random order. The key stays unopened until every
  pair is judged.
- Per pair: **left better**, **right better**, or **equal**.
- The question: *which is the better finished piece for this brief?* Judge the whole video:
  first whether it delivers the brief's beats and acceptance checklist, then craft (timing,
  motion, type, composition, polish) and visible defects. Choose **equal** when you cannot
  honestly prefer one.

With 3 held-out briefs and 3 runs per Montagent arm, that is 3 × (3 + 9) = 36 pairs.

**The court (recorded, never decisive):** each juror in `pins.json` judges the same pairs,
independently, from a sheet of stills at fixed instants, `(k + 0.5) / 8` of the brief's
declared length for k = 0…7, left clip above right, with the same three choices and the same
question. Its ballots are committed verbatim. Where it disagrees with the human, the
disagreement is reported, not resolved.

## Edge cases, decided now

- **No deliverable.** A run that leaves no decodable `deliverable.mp4` loses every one of its
  pairs, automatically, without being shown. Two such runs in one pair are **equal**.
- **A cap hit.** A run stopped by the wall-clock or spend cap is judged on whatever it
  delivered, and scored as above if it delivered nothing.
- **Skills that do not trigger.** A with-skills run whose transcript shows no skill triggered
  still counts as a with-skills run: failing to trigger is the skills' failure. It is
  reported.
- **An isolation problem** is the harness's failure, not the agent's. The run is voided: its
  directory moves to `runs/<phase>-void/` with a note of the problem, and the run is repeated
  before pairing. `verdict.py` reports any isolation problem among the counted runs, and
  such a verdict is `invalid`.
- **A ballot that is not left, right or equal** does not count; the pair stays unjudged.

## Decision rule

Over the human's ballots and the automatic decisions:

- **Parity passes** when the reference is picked as better in **at most 40%** of parity pairs.
- **Lift passes** when with-skills **wins more** lift pairs than no-skills wins. Equal votes
  count for neither side.

The verdict **passes** only if both pass.

**Top-up.** If a single changed human ballot would flip either result, the result is too
close to call at 3 runs: run each Montagent arm up to **5 runs per brief** (the reference
stays at one build), pair the new runs (the existing pairs and ballots stand), judge the new
pairs, and apply the rule again to everything. After the top-up the rule is applied as it
falls; there is no second top-up.

## Recorded, never decisive

For every run, `verdict.py` re-derives from the committed files and reports:

- **Reach:** `query --census type` over every element of the final project.
- **Checks:** `validate` counts by class, and the finding codes.
- **Process:** Montagent verbs used, `frame`/`preview` loops, and whether the run looked at
  its own pictures (a look verb, or reading an image file) or delivered one-shot.
- **Cost:** turns, wall-clock, tokens and spend.
- **Delivery:** whether a video was delivered, and its length and size.
- Whether the with-skills run's skills triggered.

`R-CAPTION-*` findings fire on every text element (see
[Flag: caption checks fire on every text element](https://github.com/MBehtemam/Montagent/issues/458)),
so finding counts are noisy for motion graphics. That is one reason they are never decisive.

## Evidence

Everything a verdict rests on is committed under `docs/research/skills-eval/`: the runs
(`runs/verdict/`), the pairs and key, both sets of ballots, the court's stills, and
`judging/verdict/verdict.json`. `verdict.py` re-derives the signals and the tally from them
and exits non-zero if the committed verdict no longer follows.
