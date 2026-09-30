# Skills eval

Evidence for the eval designed in
[Eval design: measuring a Montagent ad built with and without skills](https://github.com/MBehtemam/Montagent/issues/446).
It asks whether Opus 5.5 using Montagent with the end-user skills makes motion graphics
comparable to what it makes without Montagent, and better than Montagent without the skills.

| Path | What it holds |
|---|---|
| `briefs/dev/` | The four open development briefs: A (Montagent launch spot), B (talking-head social cut), C (mascot character short), E (logo reveal loop). The baseline runs these, and the skills may be written with them in view. |
| `assets/` | The fixed asset pack. Every run starts from a copy of this directory and nothing else. Its `README.md` is the manifest, and is part of the pack. |
| `pack-src/` | The scripts that made the pack's generated files (brand, music bed, voiceover, presenter word timings, the screen-recording tape), and in `character/` the owl's FLUX drawings and the scripts that cut its rig, voiced its lines and prepared its set. They are not part of the pack. |
| `RECORDING.md` | How the pack's footage was made: the presenter takes and their scripts, and the screen recording. |
| `RUBRIC.md` | The pre-registered rubric and decision rule. `harness/pins.json` is part of it. |
| `harness/` | The scripts that run the arms, pair the runs blind, serve the judging page and convene the court. |
| `runs/<phase>/<brief>/<arm>-<n>/` | One recorded run: manifest, transcript, workspace, 720p render. `<phase>` is `baseline` (development briefs, exploratory) or `verdict` (held-out briefs). |
| `judging/<phase>/` | Pairs, key, ballots, the court's stills, and `verdict.json`. |
| `verdict.py` | Re-derives the signals and the tally from the committed files, and exits non-zero if `judging/<phase>/verdict.json` no longer follows. |

The held-out briefs are not here. Only their SHA-256 hashes will be, once they are written
([Held-out eval briefs: written blind and sealed by hash](https://github.com/MBehtemam/Montagent/issues/466)).

## Running it

Every script is a `uv run` script with its dependencies inline; run them from the repo root.
They need `claude` at the version `harness/pins.json` pins, logged in as usual, plus
`ffmpeg` and `cargo`. Before a batch of runs, `uv run $H/run_arm.py check` proves the
sandbox with one cheap Haiku session.

```sh
H=docs/research/skills-eval/harness
# 1. One run of one arm. --commit pins binary, skills and pack (default HEAD).
uv run $H/run_arm.py --brief <brief.md> --arm reference|no-skills|with-skills [--phase verdict]
# 2. Once every arm has its runs: pair them blind (re-run after a top-up).
uv run $H/pair.py --phase verdict
# 3. The human judges on a local page; ballots are saved as they are cast.
uv run $H/judge.py --phase verdict --briefs <held-out briefs>
# 4. The court, over stills (recorded, not decisive).
uv run $H/court.py --phase verdict --briefs <held-out briefs>
# 5. Tally, write, commit; afterwards, the check.
uv run docs/research/skills-eval/verdict.py --phase verdict --write
uv run docs/research/skills-eval/verdict.py --phase verdict
```

`run_arm.py probe` prints what an isolated session loads (one tiny Haiku call); use it to
refresh `builtin_skills` when Claude Code's pin moves. `--dry-run` sets a run up
and prints the exact command instead of running it.
