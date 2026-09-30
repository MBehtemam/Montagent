# Skills eval: what the baseline runs show

This is the evidence from the baseline phase of the skills eval. It covers three development
briefs: A (launch spot), B (talking head) and E (logo loop). Each brief was run three times
with Montagent and no skills (`no-skills-1..3`), and once with no Montagent at all
(`reference-1`), where the model built the video however it liked. The write-up records what
an agent using Montagent without skills gets wrong, and what that costs in turns or in
output quality. Its purpose is to decide which skills get written.

**Sources.** Each run's folder, `runs/baseline/<brief>/<arm>-<n>/`, holds three things:

- `manifest.json`: pins, signals, the final `validate` counts and the element census.
- `transcript.jsonl`: every tool call and every result.
- `workspace/`: the project file and any generator script.

The no-skills runs are pinned to Montagent `6f684fc8`, which includes main's ffmpeg fix.
Every run used Claude Opus 5.5 at effort high and Claude Code 2.1.284. Capability grades
come from the capability inventory on branch `research/capability-inventory`
(`docs/research/skills/capability-inventory.md`), called "the inventory" below. A call
number such as `#30` is that run's thirtieth tool call.

## Run signals

- Turns and cost come from the transcript's `result` event.
- "Before create" counts the tool calls made before the project was scaffolded.
- "`frame` calls" counts calls to the `frame` verb over MCP or the CLI. A1's two calls were
  CLI loops that pulled 12 frames each.
- "Final validate" is the harness's own `validate`, run after the agent finished.

**No run hit a single Montagent `error` finding.** Every render succeeded on the first
attempt. The extra renders below were the agents' own choice to re-render.

### A: launch spot (12 s, 16:9, with music)

| Run | Turns | Cost | Wall | Cache read | Before create | Renders | `frame` calls | Elements | Final validate |
|---|---|---|---|---|---|---|---|---|---|
| no-skills-1 | 31 | $1.46 | 247 s | 1.92 M | 12 | 2 | 2 (CLI loops) | 20 | 0 err, 15 review (`R-EASE-INERT` 12, `R-KEYFRAME-UNREACHED` 3) |
| no-skills-2 | 37 | $1.62 | 289 s | 2.07 M | 8 | 1 | 8 | 22 | 0 err, 12 review (`R-EASE-INERT` 12) |
| no-skills-3 | 39 | $1.56 | 294 s | 1.99 M | 9 | 2 | 10 | 22 | 0 err, 16 review (`R-EASE-INERT` 12, `R-KEYFRAME-UNREACHED` 4) |
| reference-1 | 34 | $1.15 | 312 s | 1.23 M | — | — | — | — | — |

### B: talking-head social cut (10 s, 9:16, green screen, captions, music under voice)

| Run | Turns | Cost | Wall | Cache read | Before create | Renders | `frame` calls | Elements | Final validate |
|---|---|---|---|---|---|---|---|---|---|
| no-skills-1 | 57 | $3.04 | 881 s | 5.44 M | 18 | 2 (136 s, 149 s) | 11 | 110 | 0 err, 100 review (`R-CAPTION-*` 91, `R-EASE-INERT` 9) |
| no-skills-2 | 59 | $2.53 | 706 s | 4.12 M | 18 | 2 (118 s, 124 s) | 8 | 23 | 0 err, 9 review (`R-EASE-INERT`) |
| no-skills-3 | 59 | $2.67 | 664 s | 4.54 M | 8 | 1 (138 s) | 12 | 20 | 0 err, 11 review (`R-EASE-INERT`) |
| reference-1 | 44 | $1.67 | 573 s | 1.77 M | — | — | — | — | — |

### E: logo reveal loop (6 s, 1:1, silent)

| Run | Turns | Cost | Wall | Cache read | Before create | Renders | `frame` calls | Elements | Final validate |
|---|---|---|---|---|---|---|---|---|---|
| no-skills-1 | 28 | $1.29 | 251 s | 1.36 M | 9 | 1 | 2 | 23 | 0 err, 21 review (`R-CAPTION-*`) |
| no-skills-2 | 37 | $1.69 | 316 s | 2.27 M | 10 | 2 | 4 | 25 | 0 err, 22 review (`R-CAPTION-*`) |
| no-skills-3 | 43 | $2.09 | 332 s | 3.18 M | 10 | 3 | 3 | 42 | 0 err, 54 review (`R-CAPTION-*`) |
| reference-1 | 34 | $1.33 | 393 s | 1.52 M | — | — | — | — | — |

**Verbs.** All nine runs used the same verbs:

- `create_project` (over MCP or the CLI)
- `fonts vendor`
- `measure`
- `fmt`
- `validate`, usually chained after a generator script
- `frame`
- `render`

`query` appears only in the three B runs, and `timeline` only in E2. **No run used
`preview`, `compare` or `shift`.** All nine wrote a Python generator for the project file.

## How the agents discovered the format

All nine runs took the same route. That route is what a skill should front-load.

1. They read `cat README.md` and listed the files. Most also ran `montagent --help` in the
   first call.
2. They ran `ToolSearch` to load the Montagent MCP tools.
3. They read `montagent://format.md` (20 244 chars).
4. They read `montagent://schema.json`. At 77 KB it **overflowed the tool-result limit in
   all nine runs** and was saved to a file instead. Each run then spent **1–3 calls** getting
   the content out with Python, or by reading it back in chunks. E3, for example, used
   `Read` with `offset 1405`.
5. They ran `montagent fonts --help` and `fonts vendor --help`, all nine of them. That is
   where they learned that vendoring is CLI-only and that `vendor` "never edits the `fonts`
   table". All nine then wrote the `fonts` chain by hand.
6. From there the order was: `create_project` → `fonts vendor` → the `fonts` chain →
   `measure` → generator → `fmt` → `validate` → `frame` at the beats → `render` → ffmpeg
   checks on the MP4.

By the first `create_project`, the discovery phase had put **51–108 K chars** of tool
results into context: A 59–80 K, B 64–86 K, E 51–108 K. That text is carried into every
later turn.

## Why the no-skills runs cost more than the reference

| Brief | Cost vs reference | Turns vs reference | Where the extra went |
|---|---|---|---|
| A | +27 to +41 % | −3 to +5 | Mostly context. Cache read was +0.70–0.84 M tokens over a similar number of turns, carrying the schema and help text above. The reference spent 7 calls of its own looking for a Python install with Pillow and numpy, so the call counts come out close. A1 and A3 each re-rendered once after a review of frames pulled from the MP4. |
| B | +51 to +82 % | +13 to +15 | Chroma tuning took 8–13 key- or green-related calls per run, against 7 in the reference. Hand-written green-pixel scans of the output. B1's 90-element flipbook wipe (pattern 3). Second renders of 118–149 s in B1 and B2. Context grew to 4.1–5.4 M cache-read tokens, against 1.8 M. |
| E | −3 to +57 % | −6 to +9 | E1 beat the reference on both turns and cost. E2 and E3 each re-rendered after finding a problem in their own contact sheet (pattern 5). E3 also re-rendered to fix frame-grid rounding (pattern 7). |

On pure motion graphics (E), Montagent without skills can already match the reference. The
gap opens on footage-heavy work (B), where the agent has to tune and verify things that
Montagent does not measure for it.

## Recurring patterns, ranked

Ranked first by the turns and cost they lost, then by their effect on the output.

### 1. Tuning the green-screen key by hand (B, 3 of 3)

In all three B runs the first key made the hair and collar see-through: B1 at #30, B2 at
#27, B3 at #17. The runs then swept the key parameters through `frame --png`, and each
wrote its own green-pixel counter: B1 `keytest.sh`, B2 `green.py`, B3 `greencheck.sh`. All
three reached similar settings: tolerance 0.2–0.25, softness 0.05–0.1, spill 0.8–1.0. B1
found a residual cast only after its first render (#38), swept again and rendered a second
time (#41–#50). B2 and B3 then checked for green across the whole output, frame by frame.
Each run spent **8–13 calls** on this, the largest single cost in B. Starting values and a
way to check them are skill-teachable. The rest is a **product-gap candidate**: Montagent
has no spill or fringe measure. `measure` reports alpha coverage only, and the agents did
not use it to choose settings.

### 2. Discovery overhead: schema overflow and CLI help (9 of 9)

See the discovery section above. Every run lost 1–3 calls to a 77 KB schema that did not
fit in a tool result. `--help` output added another 4–22 K chars per run. Nothing failed,
but this is the context that makes every later turn more expensive. Against the reference,
cache read ran +0.7–0.8 M tokens in A and +2.4–3.7 M in B. A compact format card is
skill-teachable and would remove most of this. It is also a **product-gap candidate**: there
is no compact or per-type schema resource.

### 3. A frame-by-frame flipbook instead of a wipe (B1)

A mask cannot be animated, so B1 built the name-bar wipe as one element per frame, each
with a static mask slightly wider than the last (`lt-bar-on-01`, 1000..1033 ms, mask width
172; `lt-bar-on-02`, 1033..1066, width 315; and so on). That came to **90 elements across 4
tracks**, and B1 said so itself: "Montagent has no wipe transition, so I built it frame by
frame". Every slice is a text element, so `R-CAPTION-*` fired 91 times in the final
`validate`. B2 and B3 used the inventory's idiom instead: an occluder rect scaled about its
edge (`wipe-on-cover` with `origin: top-left` and `scale [0,1]→[1,1]`, then a reveal rect
scaled back to zero). That took 8 name-bar elements. The superseded runs showed the same
split: two flipbooks, one occluder. Render time did not follow the flipbook here: B3 took
138 s against B1's 136–149 s. The recipe is skill-teachable. The flipbook is also evidence
for inventory flag 8, no animatable `clip` or mask, which is a **product gap**.

### 4. Supplied word timings copied without a check: the 20 ms "Now" (B, 3 of 3)

`take-1.words.json` gives "Now" as 2420–2440 ms. All three B runs copied that window into a
`highlight`, which lights the word for at most one frame. B1 noticed it ("The word 'Now' is
only 20 ms long in the timing file, so it is highlighted for a single frame") and kept it
anyway. `validate` raised nothing. This probably fails the checklist line "each caption
word is highlighted while it is being spoken". A habit of sanity-checking supplied timings
against the audio is skill-teachable. A `review` for any highlight window shorter than a
frame is a **product-gap candidate**.

### 5. Reviewing by home-made contact sheets and re-renders (9 of 9)

`frame` takes a single instant, so every run built its own contact sheet by tiling frames
with ffmpeg, either from `frame` output or from the rendered MP4. Every transcript has
`tile=` or `xstack`/`hstack`. A1 looped 12 CLI `frame` calls (#19). Some of these home-made
tools failed:

- A1 (#20) and B2 (#39) lost a call each to `No such filter: 'drawtext'`.
- E3 lost one to `Unrecognized option 'vsync'` (#24) and one to a `select` parse error
  (#32).

Problems that a range view would have caught before rendering were caught only afterwards:

- E2's spinning tile clipped the circle, found from the post-render contact sheet (#29–#33).
- E3's cursor landed a frame late (#37).

Both led to re-renders. No run used `preview`. This confirms inventory flag 1: range mode
is specified but not shipped, a **product gap**.

### 6. Review noise the agents had to reason away (9 of 9)

- **`R-CAPTION-*` on text that isn't a caption.** It fired on typed letters: 21–54 reviews
  per E run. E1: "caption heuristics (short, silent text), which is expected for a
  typewriter effect". It also fired on B1's flipbook slices, 91 times.
- **`R-EASE-INERT` on deliberate holds**, where two equal keyframes are joined by `linear`.
  All three A runs got 12 each; the B runs got 9–11. A2: "The remaining reviews are
  intentional holds".
- **`R-KEYFRAME-UNREACHED` on push-ins that end at the element's `end`.** It is left in the
  final A1 (3) and A3 (4).

The cost in calls is small. The risk is quality: the noise teaches the agent to ignore
`review`. Both caption noise and hold noise are **product-gap candidates** (inventory
flag 5). A skill can teach which reviews to act on.

### 7. Timing on the frame grid (E, 2 of 3)

- E2 (#18): "Fade ends between frames (last visible frame at 0.9% opacity)". It moved the
  end keyframe onto frame 164.
- E3 (#37–#38): "rounding element boundaries to the nearest ms delays some cuts by one
  frame". It switched to `floor(n*1000/30)` and re-rendered.

The inventory graded half-open time and `measure --at` as "findable". Both runs learned it
from findings or from the render, not up front. Skill-teachable.

### 8. The same beat, built three different ways (A)

The brief asks for "read." to turn the accent colour at 2.0 s. The runs split:

- A3 used a run `highlight` from 2000 ms.
- A2 stacked a second, orange "read." and crossfaded to it over 0.2 s (`hook-read-accent`).
- A1 set the element itself to orange (`color: #FF5A36`), so nothing turns at all.

In the superseded runs all three used `highlight`. Finding the feature is not reliable
without guidance. For the pop, all three A runs spelled the overshoot as explicit keyframes
(0.55 → 1.12–1.14 → 1.0). None used the overshoot bezier (`y > 1`), which the B runs used
for the PiP. For the cursor blink, all three E runs used separate on and off elements
rather than a `step` ease. The output is equivalent, but the project is harder to edit. All
of this is skill-teachable.

### 9. Harness and environment problems (not skills, not product)

- **Runs running at the same time share `$TMPDIR` (`/tmp/claude-501`).** At 16:54:34, E2
  wrote `$TMPDIR/gen.py`. At 16:54:54, B2 ran `python3 $TMPDIR/gen.py`, expecting its own
  generator, and got E2's: `FileNotFoundError … 'logo-loop.json'` (#48). B2 gave up on its
  generator and hand-edited the project for the rest of the run (#49–#52). Other temp paths
  were reused across runs too (`f/`, `sheet.png`, `els.json`, `build/gen.py`), but those
  reuses did not overlap in time. The harness should give each run its own `TMPDIR`
  before the skills arm runs. **This is an eval-validity issue.**
- **Missing Python packages.** B1 had no `numpy` (#16); B2 (#9) and B3 (#13) had no `PIL`.
  Each lost one call.
- **Wrong temp path.** A2, B1, B2 and E2 each ran `Read` on a guessed temp path that did not
  exist, then called `echo $TMPDIR`. This is a Claude Code quirk, not Montagent's.

### Not reproduced in the fresh runs

These superseded-run patterns do not appear in the fresh runs:

- **The centre-`origin` trap.** No run remarks on it or repairs it.
- **The `E-SOURCE-OVERRUN` / `E-OVERRUN-UNNEEDED` detour.** All three B runs wrote
  `source_end: 9760` with `overrun: "hold"` first time.
- **The ffmpeg crash.** It is fixed.

### Capabilities found without help (not a failure)

- `highlight` for karaoke captions (B, 3 of 3, 25 highlighted runs each)
- `chroma` (B, 3 of 3)
- a circle `mask` for the PiP, with an overshoot bezier pop (B, 3 of 3)
- a `mask` rect with `radius` plus `shadow` for the rounded frame (A, 3 of 3)
- the `transition` crossfade, with no `E-TRANSITION-*` errors (A, 3 of 3)
- `origin: bottom-center` bars growing from the baseline (A, 3 of 3)
- `loop: true` (E, 3 of 3)
- `rotation` 1080 (E, 3 of 3)
- keyframed `volume` for ducking and fades (A and B, 6 of 6)
- `measure --elements` for layout (9 of 9)

What was missing was the **craft**: the recipes and values that make these features right
the first time. There was also one miss in finding a feature (`highlight` in A).

## Brief deviations (as far as the files show)

- **A.** All three runs match the structural checklist:
  - hook words start at 0/500/1000/1500/2000;
  - the crossfade is a `transition`;
  - the frame is a `mask` with radius plus `shadow`;
  - the bars grow from `bottom-center`;
  - an occluder wipe clears by 10 450–10 500 ms;
  - the music reaches 0 by 11 900–11 950.

  "read." never visibly *turns* in A1: it arrives orange. In A2 it crossfades over
  0.2 s rather than switching at 2.0 s. In A3 it carries the accent from its own arrival.
  The brief ("turns") and the checklist ("at 2.0 s, not before") pull in different
  directions here. A1 revealed `lockup-on-dark.png` (mark and name) rather than the
  wordmark. Visual quality cannot be judged from the files.
- **B.** All three runs:
  - put a circle `mask` PiP in with an overshoot pop;
  - duck the music to 0.15–0.2 under speech;
  - wipe the name bar on at 1.0 s and off at 6.0 s.

  The "Now" highlight lasts at most one frame in all three (pattern 4). The files cannot
  show whether green is left, or whether the captions cover the face. Each run's own scan
  reported no green outside compression noise.
- **E.** All three runs:
  - use a Paper ground with three Ink tiles and a Signal circle at top right, matching the
    checklist;
  - bring four separate shapes in, staggered;
  - turn one square exactly 1080°;
  - type per letter, with the cursor moving ahead;
  - blink with a hard on/off.

  The first and last encoded frames differ by encoder noise only: at most 1 grey level in
  E2, at most 4 in E3 ("differ by at most 4 of 255 levels in about 0.06% of pixel values").
  E3 reports its own deviations: the cursor appears at about 2.3 s, and the clear runs to
  5.8 s.

## Superseded runs (set aside)

The first nine no-skills runs are in `runs/baseline-superseded/` (see its README). They were
pinned to `92a45d16`, which predates the ffmpeg 7.1+ fix. With the eval machine's ffmpeg 9,
every run with audio (all of A and B) failed its first render with `E-INTERNAL …
Unrecognized option 'filter_complex_script'`, and lost 2–4 calls working around it. That
cost belongs to a product bug that is now fixed, so the runs were repeated. Once the crash
was gone, cost and turns moved like this (mean of three runs, old → new):

| Brief | Turns | Cost |
|---|---|---|
| A | 40.0 → 35.7 | $1.77 → $1.55 |
| B | 63.7 → 58.3 | $3.13 → $2.75 |
| E (never crashed) | 31.0 → 36.0 | $1.65 → $1.69 |

E's shift is run-to-run variance, which is roughly the size of the A and B gains.

## What a skill can teach, and what it cannot

### (a) Skill-teachable

1. **A compact format card**, so the agent never has to parse the 77 KB schema. It should
   cover:
   - the element types and their required fields;
   - the keyframe and `ease` rules, including the overshoot bezier with its peak values and
     `step` for hard switches;
   - `highlight`: one window per run, and at least a frame long;
   - `mask`, which is static and rides the transform;
   - the `transition` window;
   - `loop`;
   - `volume` and the unnormalised mix.
2. **The setup sequence.** `create_project` → `fonts vendor` (CLI) → write the `fonts` chain
   by hand → `measure`. Say plainly that the CLI is required.
3. **Recipes:**
   - an edge-scaled occluder wipe, never a per-frame flipbook;
   - a colour change with `highlight`;
   - a pop;
   - a `step` blink;
   - per-letter typing with a cursor;
   - a seamless loop;
   - a music bed under voice (bed about 0.15–0.2 under speech, about 0.5–0.6 in gaps,
     fade landed on a drawn frame);
   - a green-screen key: start at tolerance about 0.2–0.25, softness about 0.05, spill
     about 0.9, then check a lossless `frame --png` crop at the hair.
4. **The frame grid.** Boundaries sit at `floor(n*1000/fps)`, and a final keyframe belongs
   on a drawn frame (`measure --at`).
5. **The check loop.** Run `frame` or `query --at` at every beat, and check a lossless
   `frame` for keying before rendering. Render once. Use `preview` for motion.
6. **Reading findings.** `R-CAPTION-*` on titles and typed letters, and `R-EASE-INERT` on
   holds, are noise. `R-KEYFRAME-UNREACHED` on a fade to zero is real.
7. **Sanity-check supplied timings** before trusting them. The 20 ms "Now" is the example.

### (b) Product gaps: a skill cannot fix these

The ffmpeg 9 `E-INTERNAL` crash from the superseded runs is fixed on main and is no longer
listed.

1. **No animatable mask or clip** (inventory flag 8). This forced B1's 90-element flipbook
   wipe and the caption noise that came with it.
2. **`frame` range mode (a contact sheet) is specified but not shipped** (inventory flag 1).
   All nine runs rebuilt it with ffmpeg, and two problems were found only after rendering.
3. **Caption checks fire on every text element** (inventory flag 5). That meant 21–54
   reviews per E run and 91 in B1. **`R-EASE-INERT` fires on deliberate holds**: 12 per A
   run.
4. **The schema resource is too big to arrive in one tool result** (77 KB). All nine runs
   spilled it to disk and parsed it.
5. **Keying offers no spill or fringe measure.** All three B runs wrote their own
   green-pixel counters and swept by hand.
6. **Nothing flags a `highlight` window shorter than a frame.** All three B runs shipped
   the 20 ms "Now". This belongs in `review`.
7. **The MCP server instructions do not say a shell is needed** (inventory flag 2). The
   eval prompt told the agents the CLI was on `PATH`, so it cost nothing here. It would cost
   a real MCP-only user.

**Eval-harness issue, not a product gap.** Runs executing at the same time share
`$TMPDIR`, which contaminated B2 (pattern 9). Fix it before the skills arm runs.

## What the skills must teach (short list)

1. A one-page format card that front-loads what the agents had to dig for: `highlight`, the
   overshoot bezier, `step`, static `mask`, the frame grid and half-open ends, and the
   unnormalised mix. It replaces the schema parse.
2. The setup sequence, including the CLI-only font step.
3. Motion recipes: an edge-scaled occluder wipe, a colour change via `highlight`, a pop, a
   `step` blink, per-letter typing, and a seamless loop.
4. Footage recipes: key starting values and a lossless check; ducking levels; how to check
   word timings.
5. The check loop: `frame` or `query` at every beat, one render, and which findings to act
   on.
