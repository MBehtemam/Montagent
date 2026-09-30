# Skills eval: what the baseline runs show

This is the evidence from the baseline phase of the skills eval. It covers the four
development briefs: A (launch spot), B (talking head), C (mascot character short) and E
(logo loop). Each brief was run three times with Montagent and no skills, and once with no
Montagent at all (`reference-1`), where the model built the video however it liked. B's
no-skills runs are numbered 1, 3 and 4 (see "Superseded runs"). C was added to the
development set after the other three had run; its runs are pinned to `65308d57`, whose
binary is identical to `6f684fc8` and whose pack adds the owl rig. The write-up records what
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

**No run left a Montagent `error` finding in its final project.** Every render that finished
succeeded on the first attempt, and the extra renders below were the agents' own choice.
Two did not finish: B4's was killed at session end (pattern 10), and C1's CLI render hung
(C-5).

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
| no-skills-3 | 59 | $2.67 | 664 s | 4.54 M | 8 | 1 (138 s) | 12 | 20 | 0 err, 11 review (`R-EASE-INERT`) |
| no-skills-4 | 92 | $4.29 | 1204 s | 8.73 M | 13 | 1 (324 s) + 1 killed at session end | 33 | 41 | 0 err, 13 review (`R-EASE-INERT` 11, `R-CAPTION-PACE` 2) |

B has no `no-skills-2`: that run read another run's temp files and was set aside (see
"Superseded runs"). `no-skills-4` repeats it. **B4's `render.mp4` is not its final project.**
After its one finished render it moved the PiP off the presenter's neck and raised the bed
(#84), started the re-render in the background (#87), and ended its turn. Headless, that ends
the session, and the render was killed. Its final `validate` and census describe a file
that was never rendered; `render.mp4` shows the defect the agent had already found (pattern
10).
| reference-1 | 44 | $1.67 | 573 s | 1.77 M | — | — | — | — | — |

### C: mascot character short (8 s, 16:9, voice line, pre-cut owl rig)

| Run | Turns | Cost | Wall | Cache read | Before create | Renders | `frame` calls | Elements | Final validate |
|---|---|---|---|---|---|---|---|---|---|
| no-skills-1 | 81 | $4.12 | 1330 s | 7.98 M | 10 (no scaffold; generator at #11) | 4 (15–17 s, 12.5 s) + 1 hung CLI render (#62) | 9 | 72 | 0 err, 698 review (`R-EASE-INERT` 625, `R-KEYFRAME-UNREACHED` 73) |
| no-skills-2 | 50 | $2.66 | 496 s | 3.54 M | 13 (no scaffold; generator at #14) | 2 (33.5 s, 21.2 s); `preview` refused | 17 | 65 | 0 err, 655 review (`R-EASE-INERT` 632, `R-KEYFRAME-UNREACHED` 23) |
| no-skills-3 | 84 | $4.19 | 749 s | 8.00 M | 13 | 1 (23.8 s) | 20 | 69 | 0 err, 681 review (`R-EASE-INERT` 681) |
| reference-1 | 86 | $3.95 | 909 s | 6.75 M | — | — | — | — | — |

C1 and C2 never called `create_project`: their generators wrote `hoot.json` directly. **The
torso art is defective**, and it drove most of C's cost (see "Brief C" below).

### E: logo reveal loop (6 s, 1:1, silent)

| Run | Turns | Cost | Wall | Cache read | Before create | Renders | `frame` calls | Elements | Final validate |
|---|---|---|---|---|---|---|---|---|---|
| no-skills-1 | 28 | $1.29 | 251 s | 1.36 M | 9 | 1 | 2 | 23 | 0 err, 21 review (`R-CAPTION-*`) |
| no-skills-2 | 37 | $1.69 | 316 s | 2.27 M | 10 | 2 | 4 | 25 | 0 err, 22 review (`R-CAPTION-*`) |
| no-skills-3 | 43 | $2.09 | 332 s | 3.18 M | 10 | 3 | 3 | 42 | 0 err, 54 review (`R-CAPTION-*`) |
| reference-1 | 34 | $1.33 | 393 s | 1.52 M | — | — | — | — | — |

**Verbs.** The nine A, B and E runs used the same verbs:

- `create_project` (over MCP or the CLI)
- `fonts vendor`
- `measure`
- `fmt`
- `validate`, usually chained after a generator script
- `frame`
- `render`

`query` appears only in B1, B3, C2 and C3, and `timeline` only in E2. B4 scaffolded with the
CLI's `create-project` rather than the MCP tool. C has no text, so no C run touched `fonts`,
and C1 and C2 never scaffolded at all: their generators wrote the project from scratch.
**Only C2 used `preview`, and it was refused (C-6). No run used `compare` or `shift`.** All
twelve wrote a Python generator for the project file.

## How the agents discovered the format

All twelve runs took the same route. That route is what a skill should front-load.

1. They read `cat README.md` and listed the files. Most also ran `montagent --help` in the
   first call.
2. They ran `ToolSearch` to load the Montagent MCP tools.
3. They read `montagent://format.md` (20 244 chars).
4. They read `montagent://schema.json`. At 77 KB it **overflowed the tool-result limit in
   all twelve runs** and was saved to a file instead. Each run then spent **1–3 calls**
   getting the content out with Python, or by reading it back in chunks. E3, for example,
   used `Read` with `offset 1405`.
5. They ran `montagent fonts --help` and `fonts vendor --help`: all nine A, B and E runs,
   whose briefs have text. That is where they learned that vendoring is CLI-only and that
   `vendor` "never edits the `fonts` table". All nine then wrote the `fonts` chain by hand.
6. From there the order was: `create_project` → `fonts vendor` → the `fonts` chain →
   `measure` → generator → `fmt` → `validate` → `frame` at the beats → `render` → ffmpeg
   checks on the MP4.

By the first `create_project` (or, in C1 and C2, the first generator), the discovery phase
had put **51–117 K chars** of tool results into context: A 59–80 K, B 64–117 K, C 65–71 K,
E 51–108 K. B4's 117 K is mostly one 58 K read of the whole schema. That text is carried
into every later turn.

## Why the no-skills runs cost more than the reference

| Brief | Cost vs reference | Turns vs reference | Where the extra went |
|---|---|---|---|
| A | +27 to +41 % | −3 to +5 | Mostly context. Cache read was +0.70–0.84 M tokens over a similar number of turns, carrying the schema and help text above. The reference spent 7 calls of its own looking for a Python install with Pillow and numpy, so the call counts come out close. A1 and A3 each re-rendered once after a review of frames pulled from the MP4. |
| B | +60 to +157 % | +13 to +48 | Chroma tuning took 8–13 key- or green-related calls in B1 and B3, and about 45 of B4's 92, against 7 in the reference. Hand-written green-pixel scans of the output. Flipbook wipes in B1 (90 elements) and B4 (17 image slices) (pattern 3). A second render of 149 s in B1, and a 324 s render in B4 from its five keyed copies of the presenter. Context grew to 4.5–8.7 M cache-read tokens, against 1.8 M. |
| C | −33 to +6 % | −36 to −2 | Nothing: here the reference cost as much as the Montagent runs. It hit the same torso defect (C-1) and repaired the art with a script, redrawing both upper arms and the body's shoulder outline (`build/repair.py`). Then it wrote its own renderer (`build/render.py`: PIL frames piped to ffmpeg). C2, which confined the arms rather than fixing the torso, was the cheapest run of the four. Cache read was 3.5–8.0 M in the Montagent runs, against 6.8 M. |
| E | −3 to +57 % | −6 to +9 | E1 beat the reference on both turns and cost. E2 and E3 each re-rendered after finding a problem in their own contact sheet (pattern 5). E3 also re-rendered to fix frame-grid rounding (pattern 7). |

On pure motion graphics (E), Montagent without skills can already match the reference. The
gap opens on footage-heavy work (B), where the agent has to tune and verify things that
Montagent does not measure for it. C matches the reference too, but only because the
defective torso cost both arms alike. Its numbers say little until C is re-run on fixed art
([#516](https://github.com/MBehtemam/Montagent/issues/516)).

## Recurring patterns, ranked

Ranked first by the turns and cost they lost, then by their effect on the output.

### 1. Tuning the green-screen key by hand (B, 3 of 3)

In all three B runs the first key made the hair and collar see-through: B1 at #30, B3 at
#17, B4 at #22 (tolerance 0.3 turned the black hair to `43211b`). The runs then swept the
key parameters through `frame --png`, and each wrote its own green-pixel counter: B1
`keytest.sh`, B3 `greencheck.sh`, B4 `greencheck.py` and `greencheck2.py`. B1 and B3 reached
tolerance 0.2–0.25, softness 0.05–0.1, spill 0.8–1.0; B4 settled at 0.15 / 0.1 / 1.0 by #30
("hair stays solid, zero green pixels"). B1 found a residual cast only after its first
render (#38), swept again and rendered a second time (#41–#50). B3 then checked for green
across the whole output, frame by frame. B1 and B3 spent **8–13 calls** on this, the
largest single cost in B.

B4 went much further. It sampled each setting as a 1×1 or one-row `frame --crop --png` read
through `xxd`, rewriting and re-`fmt`-ing the project every time: 28 `fmt` calls, and about
145 `frame` executions counting its shell loops. Then it chased a 1–2 px olive fringe on
the sweater edge (#49–#74): "a key strong enough to clear the 1-2 px band … also eats the
black hair, the white collar and skin". It ended with **five keyed copies of the
presenter**: a full-frame copy that carries the voice, and four silent copies at
0.28 / 0.04 / 1.0 clipped to head, collar, hands and trousers. That is about **45 of its 92
calls**, and it more than doubled its render time (324 s against 136–149 s).

Starting values, a way to check them, and a rule for when a 1–2 px fringe is good enough
are skill-teachable. The rest is a **product-gap candidate**: Montagent has no spill or
fringe measure, no edge choke, and no way to read a pixel value short of cropping a PNG.
`measure` reports alpha coverage only; B4 read it once (#17) and did not use it again.

### 2. Discovery overhead: schema overflow and CLI help (12 of 12)

See the discovery section above. Every run lost 1–3 calls to a 77 KB schema that did not
fit in a tool result. `--help` output added another 4–22 K chars per run. Nothing failed,
but this is the context that makes every later turn more expensive. Against the reference,
cache read ran +0.7–0.8 M tokens in A and +2.8–7.0 M in B. A compact format card is
skill-teachable and would remove most of this. It is also a **product-gap candidate**: there
is no compact or per-type schema resource.

### 3. A frame-by-frame flipbook instead of a wipe (B, 2 of 3)

A mask cannot be animated, so B1 built the name-bar wipe as one element per frame, each
with a static mask slightly wider than the last (`lt-bar-on-01`, 1000..1033 ms, mask width
172; `lt-bar-on-02`, 1033..1066, width 315; and so on). That came to **90 elements across 4
tracks**, and B1 said so itself: "Montagent has no wipe transition, so I built it frame by
frame". Every slice is a text element, so `R-CAPTION-*` fired 91 times in the final
`validate`.

B4 built a second kind of flipbook. It made a helper Montagent project just to rasterize the
bar to `gfx/namebar.png` (#36–#41, including two `E-INVOCATION` errors from `fonts vendor
--as ../fonts/…`), then placed **17 image slices** on one track, each a frame long with a
static `clip` a little wider than the last: "a true wipe, one frame-long slice per frame
through a growing clip". Rasterizing avoided the caption noise. The bar sits over the
presenter's hands, so a plain occluder would also have covered footage, but B4 never says
whether that was its reason.

B3 used the inventory's idiom instead: an occluder rect scaled about its edge
(`wipe-on-cover` with `origin: top-left` and `scale [0,1]→[1,1]`, then a reveal rect scaled
back to zero). That took 8 name-bar elements. The superseded runs showed the same split: two
flipbooks, one occluder. Render time did not follow the flipbook: B3 took 138 s against
B1's 136–149 s. The recipe is skill-teachable. Both flipbooks are also evidence for
inventory flag 8, no animatable `clip` or mask, which is a **product gap**.

### 4. Supplied word timings copied without a check: the 20 ms "Now" (B, 3 of 3)

`take-1.words.json` gives "Now" as 2420–2440 ms. All three B runs copied that window into a
`highlight`, which lights the word for at most one frame. B1 noticed it ("The word 'Now' is
only 20 ms long in the timing file, so it is highlighted for a single frame") and kept it
anyway. B4 merged phrases when `R-CAPTION-MIN-DURATION` fired on the one starting with "Now"
(#45–#47), but never looked at the highlight itself. `validate` raised nothing about it.
This probably fails the checklist line "each caption word is highlighted while it is being
spoken". A habit of sanity-checking supplied timings
against the audio is skill-teachable. A `review` for any highlight window shorter than a
frame is a **product-gap candidate**.

### 5. Reviewing by home-made contact sheets and re-renders (12 of 12)

`frame` takes a single instant, so every run built its own contact sheet by tiling frames
with ffmpeg, either from `frame` output or from the rendered MP4. Every transcript has
`tile=` or `xstack`/`hstack`. A1 looped 12 CLI `frame` calls (#19). Some of these home-made
tools failed:

- A1 (#20), C1 (#25), C2 (#41) and C3 (#67) each lost a call to `No such filter:
  'drawtext'`.
- B4 lost one to a 1×1 `crop` that ffmpeg rejects ("Invalid too big or non positive
  size", #21).
- E3 lost one to `Unrecognized option 'vsync'` (#24) and one to a `select` parse error
  (#32).

Problems that a range view would have caught before rendering were caught only afterwards:

- E2's spinning tile clipped the circle, found from the post-render contact sheet (#29–#33).
- E3's cursor landed a frame late (#37).
- B4's PiP sat over the presenter's chin and neck, found from four `tile=6x2` sheets of the
  MP4 (#78–#82).

All three led to re-renders (B4's was killed, pattern 10). The C runs went further and built
whole throwaway projects just to look at arm poses (C-4). Only C2 tried `preview`, and it was
refused for the full 8 s (C-6). This confirms inventory flag 1: range mode is specified but
not shipped, a **product gap**.

### 6. Review noise the agents had to reason away (12 of 12)

- **`R-CAPTION-*` on text that isn't a caption.** It fired on typed letters: 21–54 reviews
  per E run. E1: "caption heuristics (short, silent text), which is expected for a
  typewriter effect". It also fired on B1's flipbook slices, 91 times.
- **`R-EASE-INERT` on deliberate holds**, where two equal keyframes are joined by `linear`.
  All three A runs got 12 each; the B runs got 9–11. A2: "The remaining reviews are
  intentional holds". B4 switched its holds from `linear` to `step` to silence them (#47);
  they still fire (`ease="step" describes no motion`). No way of writing a hold is quiet.
  The C runs got **625–681 each**, from the per-frame bake of the rig (C-2). C1: "cosmetic
  notes about redundant keyframes". C3 spent #20–#23 and #64–#66 tidying them, with `step`
  again, and they still fire.
- **`R-KEYFRAME-UNREACHED` on push-ins that end at the element's `end`.** It is left in the
  final A1 (3) and A3 (4). It is also left in C1 (73) and C2 (23), on padding keyframes past
  a part's end; C3 trimmed them, from 19 to 0.

The cost in calls is small. The risk is quality: the noise teaches the agent to ignore
`review`. Both caption noise and hold noise are **product-gap candidates** (inventory
flag 5). A skill can teach which reviews to act on.

### 7. Timing on the frame grid (E 2 of 3; planned for in B4, C1 and C3)

- E2 (#18): "Fade ends between frames (last visible frame at 0.9% opacity)". It moved the
  end keyframe onto frame 164.
- E3 (#37–#38): "rounding element boundaries to the nearest ms delays some cuts by one
  frame". It switched to `floor(n*1000/30)` and re-rendered.

The inventory graded half-open time and `measure --at` as "findable". Both runs learned it
from findings or from the render, not up front. Three runs planned for it: B4 with
`frame_ms(k) = (k*1000)//FPS` for its wipe slices; C1 with the same floor in its first
generator; C3 by checking with `measure --at` (#16–#17: "Frames sample at exact n·1000/30
ms"). C2 used `round(n*1000/30)`, harmless here. A per-frame bake makes the grid
unavoidable, so the character brief teaches it by force. Skill-teachable.

### 8. The same beat, built three different ways (A)

The brief asks for "read." to turn the accent colour at 2.0 s. The runs split:

- A3 used a run `highlight` from 2000 ms.
- A2 stacked a second, orange "read." and crossfaded to it over 0.2 s (`hook-read-accent`).
- A1 set the element itself to orange (`color: #FF5A36`), so nothing turns at all.

In the superseded runs all three used `highlight`. Finding the feature is not reliable
without guidance. For the pop, all three A runs spelled the overshoot as explicit keyframes
(0.55 → 1.12–1.14 → 1.0). None used the overshoot bezier (`y > 1`), which the B runs used
for the PiP. For the cursor blink, all three E runs used separate on and off elements
rather than a `step` ease. The output is equivalent, but the project is harder to edit. C's
card pop split three ways too: the overshoot bezier (C1), a back-ease baked per frame (C2),
and explicit keys 0.02 → 1.1 → 0.96 → 1.0 (C3). All of this is skill-teachable.

### 9. Harness and environment problems (not skills, not product)

- **Runs running at the same time share `$TMPDIR` (`/tmp/claude-501`).** At 16:54:34, E2
  wrote `$TMPDIR/gen.py`. At 16:54:54, the first B2 ran `python3 $TMPDIR/gen.py`, expecting
  its own generator, and got E2's: `FileNotFoundError … 'logo-loop.json'` (#48). It gave up
  on its generator and hand-edited the project for the rest of the run (#49–#52). Other temp
  paths were reused across runs too (`f/`, `sheet.png`, `els.json`, `build/gen.py`), but
  those reuses did not overlap in time. **This was an eval-validity issue.** The harness now
  runs one arm at a time and sweeps the shared directory after each run; that run is set
  aside and B4 repeats it. B4 overlapped nothing, and 7 leftovers were swept.
- **Missing Python packages.** B1 had no `numpy` (#16); B3 (#13) had no `PIL`; B4 had
  neither (#8, #23); C1 and C2 had no `PIL` (#7), and C3 no `numpy` (#11). Each lost a call.
  B4's pure-Python pixel loops made its full-output green scan too slow to finish, and C3
  wrote its own PNG decoder (#13).
- **Wrong temp path.** A2, B1 and E2 each ran `Read` on a guessed temp path that did not
  exist, then called `echo $TMPDIR`. This is a Claude Code quirk, not Montagent's.
- **Shell quirks.** B4 lost two calls to zsh not word-splitting `$s` (#20–#21), and hit
  sandbox refusals of `nice` (#83) and `pgrep` (#86). C1 hit refusals of `sleep` (#65) and
  `ps` (#68), and found no `timeout` (#70).

### 10. Ending the session with the render still running (B4)

B4's last fix — the PiP moved off the neck, the captions nudged down, the bed raised from
0.14 to 0.2 under speech — went into the project at #84. It started the render in the
background (#87), waited on it once (#89), called `ScheduleWakeup` (#90) and ended its
turn: "The final render … and a frame-by-frame green scan are both running in the
background." In a headless session that is the end, and both background tasks were killed.
The delivered MP4 is the earlier render, with the defect it had found; `workspace/` holds a
48-byte `.montagent-partial` file. The 15 calls from #76 were wasted.

A skill can say: render in the foreground, and never end a turn with work pending. The
harness recorded this run as a clean success, and the final `validate` and census describe
a project that was never rendered. That was an eval-validity gap. The harness now flags it
(see "Eval-harness issues" below).

C1 also ended with background tasks killed, after its CLI render hung (C-5). Its deliverable
is not stale: it had rendered the unchanged project over MCP first.

### Not reproduced in the fresh runs

These superseded-run patterns do not appear in the fresh A, B and E runs:

- **The centre-`origin` trap.** Only B4 hit it, once, in its helper project: "Default origin
  is `center`; setting `top-left` explicitly" (after #38). It is back in all three C runs,
  on the backdrop image (C-4).
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
- keyframed `volume` for ducking and fades (A, B and C, 9 of 9)
- `measure --elements` for layout (the nine runs with text)
- a per-frame forward-kinematics bake of the rig, exact to 1.2 px (C, 3 of 3)
- viseme → mouth overlays riding the head's transform, and blink elements (C, 3 of 3)
- a `mask` on image parts to split the torso (C1, C3), and a range render with `from`/`to`
  (C1)
- `query --at` to check a beat (C2, C3), and `measure --at` for the grid (C3)

What was missing was the **craft**: the recipes and values that make these features right
the first time. There was also one miss in finding a feature (`highlight` in A).

## Brief C: what the character short adds

C asks for a cut-out character built from a pre-cut rig, so it tests what the format has no
word for: parenting, face overlays, lip sync. Ranked by cost:

### C-1. The torso's shoulder cut shows once an arm is raised (3 of 3, an asset-pack defect)

`character/parts/torso.png` keeps flat, notched sleeve stubs at the shoulders. They show
once an upper arm leaves about −22° to +2° of rest, which any wave, point or cheer does.
That contradicts the pack README ("a part turned about its pivot never shows a gap").

- **C1**, about 40 calls (#19–#62). It mapped the torso's alpha channel as ASCII art (#30,
  #33, #55), then drew the torso as masked copies of itself, each sleeve stub shown only
  while its arm is down. It rendered three extra times.
- **C2**, about 17 calls (#21–#38). It confined the upper arms to −22…+2° and let the
  forearms "do the big moves" (#38). Its wave and cheer barely lift the arm.
- **C3**, about 30 calls (#33–#63). It added ink "shoulder plates" behind the torso and two
  masked torso copies. After rendering it saw "torso sides under the shoulders look slightly
  ragged" (#74), and left it.

Frame pulls still show a notch or jagged torso edge at the armpit in C1 and C3, and a faint
spur in C2. The reference hit it too, and said so: "Its upper arms are really just a shoulder
disc and an elbow cap. The sleeve between them is painted onto the body … Once the owl
lowers or raises an arm, a gap opens at the shoulder". It repaired the art by script before
animating. This is neither a skill nor a product gap: it is our art, raised as
[Asset pack: the owl's torso shows its shoulder cut once an arm is raised](https://github.com/MBehtemam/Montagent/issues/516).
These runs are kept as they are, and C is re-run on the fixed pack there. Until then, C's
cost mostly measures the workaround.

### C-2. The scene graph, flattened by a script (3 of 3, right first time)

All three read the schema, saw that transforms are flat, and baked the rig's forward
kinematics into per-frame keyframes on their first try. C3 (#7): "the transforms are flat
with no parenting, so I'll compute the owl's joint hierarchy myself". Every part has `x`,
`y` and `rotation` keyed at all 240 frames. Over all 240 frames, the joint error is at most
1.2 px at every joint in every run, all of it from rounding `x` and `y` to whole pixels.
The cost is not turns. It is a 138–191 KB project, and 625–681 `R-EASE-INERT` reviews on the
baked holds (pattern 6). This is
[Flag: no parenting, group or camera transform — a cut-out character needs a flattening script](https://github.com/MBehtemam/Montagent/issues/499).

### C-3. Lip sync and blinks (3 of 3, correct)

All three mapped viseme → mouth through `rig.json`'s table and merged adjacent runs of the
same mouth. They dropped the zero-length viseme at 4175 ms and placed each mouth with the
head's own per-frame transform. The beak rests in the pauses. There are no overlapping mouths
and no one-frame flicker. C1 and C3 rounded mouth boundaries to the nearest frame, so 21 of
240 frames show the next shape a frame early; C2 kept the raw milliseconds. Blinks are
separate `eyes_closed` elements, 3–4 frames long. Only C3 checked the sync after rendering
(`query --at`, #80).

Because an element's `source` cannot change, each run needed **47 mouth elements**, each
repeating the head's keyframes. That is a new product gap,
[Flag: an element cannot change its image over time, so a mouth is 47 elements](https://github.com/MBehtemam/Montagent/issues/518),
alongside #499.

### C-4. Smaller craft misses

- **The centre-`origin` trap, 3 of 3.** Each placed the study backdrop off-frame first and
  lost one look and one fix: C1 #15–#16; C2 #17–#18 ("The set image defaulted to a centre
  origin, so it landed off-frame"); C3 #24–#25.
- **A raised hand hidden behind the head.** The head draws over the arms. C2 (#20) tilted
  the head away. C3 (#29) swung the wave out to the side: "the first version hid the hand
  behind the head".
- **A wave too short.** C1 re-rendered only to lengthen it (#45–#49): "only swings about
  twice".
- **Rig test projects.** Each built a throwaway project to try arm angles: C1 `rigtest.json`;
  C2 `sheet.json` (9 `frame` calls); C3 `shoulder.json`, `zoom.json` and `zoom2.json`. This
  is also the contact-sheet habit (pattern 5).
- **A music bed the brief does not mention, 3 of 3,** at volume 0.1 under the voice. All three said so in
  their summaries.
- **No run trusted `line-1.json`'s `duration_ms: 6000`.** All three probed the WAV and used
  its 6048 ms. That is the timing check pattern 4 asks for.

### C-5. A CLI render that hung (C1)

C1's CLI `montagent render` at #62 ran inside one compound Bash call. It passed the tool's
120 s timeout and was moved to the background. It had still not finished nine minutes
later ("The render seems stuck: the project file was written 9 minutes ago but no new
video came out"). The same, unchanged project then rendered over MCP in **12.5 s** (#74).
That cost about 10 calls and 9 minutes. It left a 56 MB `.montagent-partial` behind, and the
deliverable is not stale. Whether Montagent, the sandbox or the backgrounding is at fault is
open:
[A CLI render of the owl short hung for 9 minutes; the same project rendered in 12.5 s over MCP](https://github.com/MBehtemam/Montagent/issues/517).

### C-6. `preview` refused (C2)

C2 is the first baseline run to call `preview` (#39). It was refused: "720p … then 540p ran
to 5.0 s — past the 5.0 s scrub budget … Preview a shorter range than 240 frames". C2 fell
back to a full render rather than a shorter range. The refusal names its repair, so this is
skill-teachable: preview a range around a beat, not the whole piece.

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
  - key the presenter over the Signal accent;
  - put a circle `mask` PiP in with an overshoot pop;
  - duck the music under speech: 0.15–0.2 in B1 and B3, 0.14 in B4 as delivered;
  - wipe the name bar on at 1.0 s and off at 6.0 s.

  The "Now" highlight lasts at most one frame in all three (pattern 4). In B4's delivered
  render the PiP overlaps the presenter's chin and neck; its fix exists only in the
  unrendered project (pattern 10). The files cannot
  show whether green is left, or whether the captions cover the face. Each run's own scan
  reported no green outside compression noise.
- **C.** All three runs:
  - build the owl from the rig, and it holds together (joint error at most 1.2 px), but the
    torso's shoulder notch shows in C1 and C3, and faintly in C2 (C-1): a borderline fail on
    "no gap or loose part at the shoulders";
  - stand it on the floor, 65–67 % of the frame tall;
  - move the beak with the visemes, and blink;
  - pop the card with an overshoot (peak 1.04–1.1) as "as" is said, with the right arm
    pointing at it;
  - change the picture on "movie", at exactly 6200 ms;
  - hold the lockup from 6850–6950 ms to the end;
  - play the whole 6048 ms line from 1000 ms.

  The wave passes in C1 (upper arm at 45°, about 3 swings) and C3 (26°, 3 swings). In C2 the
  upper arm stays 23° below horizontal, with about 1.8 cycles of forearm swing, so it barely
  reads as a wave; its cheer keeps the upper arms down too. All three added a music bed the
  brief does not mention (it asks only for "sound").
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
| B | 63.7 → 69.3 | $3.13 → $3.33 |
| E (never crashed) | 31.0 → 36.0 | $1.65 → $1.69 |

The new B mean is B1, B3 and B4. B4 (92 turns, $4.29) is an outlier driven by its key
chase (pattern 1); without it B is 58.0 turns and $2.86. The shifts are mostly run-to-run
variance, which is at least the size of the crash's cost.

`B-talking-head/no-skills-2-shared-tmp`, from the repeated batch, is set aside there too
(pattern 9).

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
   - a green-screen key: start at tolerance about 0.15–0.25, softness about 0.05–0.1, spill
     about 0.9–1.0, then check a lossless `frame --png` crop at the hair; accept a 1–2 px
     edge fringe rather than splitting the key by region;
   - a cut-out character: bake the rig's forward kinematics to one keyframe per drawn frame
     on the floor grid, keep positions unrounded, place each mouth overlay with the head's
     transform, make blinks 3–4 frames, and keep a raised hand clear of the head;
   - a background image: set `origin` (the default is the centre).
4. **The frame grid.** Boundaries sit at `floor(n*1000/fps)`, and a final keyframe belongs
   on a drawn frame (`measure --at`).
5. **The check loop.** Run `frame` or `query --at` at every beat, and check a lossless
   `frame` for keying before rendering. Render once, in the foreground, and never end the
   turn while a render is running. Use `preview` for motion, over a range around the beat
   rather than the whole piece. Render over MCP rather than a long compound shell call.
6. **Reading findings.** `R-CAPTION-*` on titles and typed letters, and `R-EASE-INERT` on
   holds and baked rigs, are noise. `R-KEYFRAME-UNREACHED` on a fade to zero is real.
7. **Sanity-check supplied timings** before trusting them. The 20 ms "Now" is the example.

### (b) Product gaps: a skill cannot fix these

The ffmpeg 9 `E-INTERNAL` crash from the superseded runs is fixed on main and is no longer
listed.

1. **No animatable mask or clip** (inventory flag 8). This forced B1's 90-element flipbook
   wipe and the caption noise that came with it, and B4's helper project and 17-slice image
   flipbook.
2. **`frame` range mode (a contact sheet) is specified but not shipped** (inventory flag 1).
   All twelve runs rebuilt it with ffmpeg or throwaway projects, and three problems were
   found only after rendering.
3. **Caption checks fire on every text element** (inventory flag 5). That meant 21–54
   reviews per E run and 91 in B1. **`R-EASE-INERT` fires on deliberate holds**: 12 per A
   run, whether the hold is written with `linear` or `step` (B4), and 625–681 per C run.
4. **The schema resource is too big to arrive in one tool result** (77 KB). All twelve runs
   spilled it to disk and parsed it.
5. **Keying offers no spill or fringe measure, and no edge choke.** All three B runs wrote
   their own green-pixel counters and swept by hand. With no way to read a pixel short of
   a cropped PNG through `xxd`, B4 rewrote the project for every setting (28 `fmt` calls),
   and with no choke it split the key over five copies of the presenter.
6. **Nothing flags a `highlight` window shorter than a frame.** All three B runs shipped
   the 20 ms "Now". This belongs in `review`.
7. **The MCP server instructions do not say a shell is needed** (inventory flag 2). The
   eval prompt told the agents the CLI was on `PATH`, so it cost nothing here. It would cost
   a real MCP-only user.
8. **No parenting, group or camera transform**
   ([#499](https://github.com/MBehtemam/Montagent/issues/499)). Every C run baked the rig to
   240 keyframes per part, which made a 138–191 KB project.
9. **An element cannot change its image over time**
   ([#518](https://github.com/MBehtemam/Montagent/issues/518)), so each C run needed 47
   mouth elements.
10. **A CLI render hung for 9 minutes; the same project rendered in 12.5 s over MCP**
    ([#517](https://github.com/MBehtemam/Montagent/issues/517), a candidate, not yet
    diagnosed).

**Asset-pack issue.** The owl's torso shows its shoulder cut once an arm is raised
([#516](https://github.com/MBehtemam/Montagent/issues/516)). It cost the C runs 17–40 calls
each. Fix it, and re-run C's baseline, before any skills or verdict run uses the rig.

**Eval-harness issues, not product gaps.**

- Runs executing at the same time shared `$TMPDIR`, which contaminated the first B2
  (pattern 9). Fixed: runs no longer overlap, and each run's leftovers are swept.
- A run that ended with its render killed was recorded as a clean success, and its final
  `validate` describes a project that was never rendered (pattern 10). Fixed: the
  transcript signals now count `background_tasks_killed` (B4 and C1 have 2 each; every other
  run has 0), and `run_arm.py` records `delivery_warnings` from the live directory: a
  render's partial file left behind, or a project written after the deliverable. The runs
  here predate that record. Both are reported, never decisive (`RUBRIC.md`).

## What the skills must teach (short list)

1. A one-page format card that front-loads what the agents had to dig for: `highlight`, the
   overshoot bezier, `step`, static `mask`, the frame grid and half-open ends, and the
   unnormalised mix. It replaces the schema parse.
2. The setup sequence, including the CLI-only font step.
3. Motion recipes: an edge-scaled occluder wipe, a colour change via `highlight`, a pop, a
   `step` blink, per-letter typing, and a seamless loop.
4. Footage recipes: key starting values and a lossless check; ducking levels; how to check
   word timings.
5. The character recipe: the forward-kinematics bake on the frame grid, viseme overlays that
   ride the head, blinks, and a pose check before rendering.
6. The check loop: `frame` or `query` at every beat, `preview` over a range, one render in
   the foreground, and which findings to act on.
