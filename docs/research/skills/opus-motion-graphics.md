# What motion graphics people make with Opus 5.5, and what Montagent can express

Research for [#462](https://github.com/MBehtemam/Montagent/issues/462), on the map
[#441](https://github.com/MBehtemam/Montagent/issues/441) (end-user agent skills). It feeds
the eval design: the eval's briefs should be the kind of animation people already make with
Opus 5.5.

**Question.** What video and motion graphics are people publicly making with Claude Opus 5.5
(and recent Claude models), in which medium, what kinds of pieces, what makes the admired ones
good, and what workflow the agent used? Which of those pieces could Montagent express today,
which only with taught craft, and which not at all?

**Method, and how far to trust it.** Gathered on 2026-09-29, one week after Opus 5.5 shipped
(2026-09-22, [Anthropic](https://www.anthropic.com/claude-opus-5-5)).

- Almost all primary evidence is **posts on X**. I read post text through the public
  `api.fxtwitter.com` mirror of each post, and linked the original `x.com` URL. I did **not
  watch the videos**. Visual claims come from (a) the single poster frame that
  [yihui-dev/awesome-opus5-5-videos](https://github.com/yihui-dev/awesome-opus5-5-videos)
  publishes for each entry, which I looked at for four pieces, and (b) the nine-frame
  observations that [athemeroy/awesome-opus-5-5-videos](https://github.com/athemeroy/awesome-opus-5-5-videos)
  records per case in [`data/cases.csv`](https://github.com/athemeroy/awesome-opus-5-5-videos/blob/main/data/cases.csv).
- **The model version is almost always self-reported.** No post links a transcript. Where a
  creator names "Opus 5.5" I say so. Where the creator only says "Claude", I say that too.
  athemeroy excluded one viral piece for exactly this reason (its
  [README](https://github.com/athemeroy/awesome-opus-5-5-videos#readme)).
- "One prompt" claims are often wrong in spirit. Thariq, who works on Claude Code, put it this
  way: "the post: 'Claude one-shot this' the prompt: 10k characters with good takes plus skills,
  examples and API keys" ([post](https://x.com/trq212/status/2102870353781641416)). Several
  pieces below rely on skills, codebases or supplied assets that you can't see in the post.
- Both "awesome" lists are community curation that started within days of launch. Their counts
  describe what the curators collected, not what everyone on X made.
- **Evidence for earlier Claude models is thin.** The one strong earlier data point is
  Remotion's January 2026 launch of its Agent Skills ("This animation was created just by
  prompting", 18.9M views, [post](https://x.com/Remotion/status/2013626968386765291)). That
  post names Claude Code but not the model. I found nothing comparable for Opus 5 or Fable 5.1
  beyond creators' comparisons in passing (for example
  [@bridgemindai](https://x.com/bridgemindai/status/2102462889160286423): "Better than anything
  I have gotten from Fable 5.1 or GPT 6 Astra").

## 1. The answer in brief

- **A new genre took off in the week after launch.** Within days, a curated list had 389
  Opus 5.5 videos with their prompts
  ([yihui-dev](https://github.com/yihui-dev/awesome-opus5-5-videos)). A second project
  collected 1,511 candidate posts and reviewed 168 of them by hand
  ([athemeroy](https://github.com/athemeroy/awesome-opus-5-5-videos)).
- **The medium is mostly a hand-rolled browser renderer, not a framework.** Opus writes one
  HTML file with a `seek(t)` or `draw(t)` function. Playwright screenshots every frame and
  ffmpeg encodes the result. Remotion and HyperFrames are used when the user asks for them.
- **What people make.** The largest group is **ads and launches in a motion-graphics / UI
  style** (154 of 1,119 classified files). After that come the "motion designer showreel",
  explainers (education, process, recipes), AI-about-AI films, and code-drawn cartoon shorts.
  3D and game captures form a separate large group.
- **The admired pieces share a recognisable craft.** They use real product UI and real data,
  one idea per shot, a beat grid with every cut on a beat, springs with only a little
  overshoot, masked or morphing transitions instead of fades, and restrained type and colour.
  They also keep generic "AI chrome" off the frame.
- **The workflow that works is render and look.** The agent plans a beat sheet or storyboard,
  renders stills or a contact sheet (one frame per beat), fixes what it sees, and only then
  renders the whole piece. Human notes are phrased the way a director would give them.

## 2. Mediums

Tag counts over the 389 entries in yihui-dev's
[`data/videos.json`](https://github.com/yihui-dev/awesome-opus5-5-videos/blob/main/data/videos.json).
The curator assigned the tags; the list doesn't say how. An entry can carry several tags.

| Tag | Entries | | Named in the prompt text | Entries |
|---|---:|---|---|---:|
| `canvas` | 288 | | "ffmpeg" | 18 |
| `threejs` | 130 | | "playwright" | 16 |
| `svg` | 125 | | "remotion" | 3 |
| `shader` | 95 | | "hyperframes" | 2 |
| `gsap` | 59 | | "manim" | 1 |
| `css` | 39 | | "lottie" | 0 |
| `audio` | 36 | | "after effects" | 4 (mostly "After Effects style") |

**Default route.** Tommy Rossi inspected the one-shot output and posted the code
([post](https://x.com/__morse/status/2103485566570369333),
[gist](https://gist.github.com/remorses/3d467b50a0519ef7859823046dc9c427)). He found the model
"put all the code in a single index.html file and rendered it using playwright frame by frame
in a headless window, then ffmpeg to generate the mp4. it used a seek function and eval". He
also found it "seems to prefer doing everything with zero dependencies from scratch instead of
using tools like remotion, egaki, or hyperframes". It analysed the audio with Python and pasted
the beat timestamps into the page as magic numbers. A second creator reported the same thing
second-hand: "neither [Remotion nor HyperFrames] - it just built its own renderer instead"
([@fabianstelzer](https://x.com/fabianstelzer/status/2103466607804862796)).

**Framework routes.** Some creators did name a framework:

- **Remotion**: [@bridgemindai merch ad](https://x.com/bridgemindai/status/2102462889160286423),
  and prompts that say "using Remotion" in yihui's list.
- **HyperFrames (HTML+GSAP)**: [@Miguel07Code, Shotbase launch](https://x.com/Miguel07Code/status/2102441708395041170).
- **Both**: @leodev tried Remotion first and moved to HyperFrames, noting that "there is no
  single tool that wins each times"
  ([post](https://x.com/leodev/status/2102897952587133299)).
- **Manim**: a derivative lesson with edge-tts narration
  ([@LinearUncle](https://x.com/LinearUncle/status/2103128559174971663)).
- **Driving other tools**: in one piece Opus drove **After Effects** to beat-cut and add
  effects to footage from another video model
  ([@aicreataro](https://x.com/aicreataro/status/2102656273112326609), as summarised in
  athemeroy's `cases.csv`). Other pieces drive Blender, Runway or Seedance
  ([athemeroy's seven production paths](https://github.com/athemeroy/awesome-opus-5-5-videos#seven-production-paths-with-frames)).

**Audio.** The soundtrack is either synthesised with the Web Audio API in the same page
([iArt](https://www.iart.ai/blog/ai-javascript-animation)) or taken from a licensed song
(Mixkit in the [gist README](https://gist.github.com/remorses/3d467b50a0519ef7859823046dc9c427)).
Either way, the song is analysed with numpy for a beat grid, and each sound effect is placed so
that its measured peak lands on the event
([@twoclipping Hooklab prompt](https://x.com/twoclipping/status/2102554209166000267)).

## 3. Kinds of pieces

athemeroy used a vision classifier to label 1,119 files (confidence `yes` or `likely`); the
labels are not verified authorship. The ten largest cells in its
[statistics](https://github.com/athemeroy/awesome-opus-5-5-videos/blob/main/docs/statistics.md):

| Domain × style | Files |
|---|---:|
| Games / interactive × 3D render | 161 |
| **Ads / launches × motion graphics / UI** | **154** |
| AI about AI × motion graphics / UI | 62 |
| AI about AI × 3D render | 60 |
| Education / science × motion graphics / UI | 55 |
| Short story × flat cartoon | 37 |

The largest style overall is motion graphics / UI (350 files), then 3D render (324). In
yihui's list, the category counts are `motion` 223, `interactive` 68, `3d` 51 and
`explainer` 47.

**One prompt dominates.** 56 of yihui's 389 entries open with the same sentence (matched on
their first 120 characters, case-insensitive), "make a dynamic
15-second motion graphics video that shows what an incredible motion designer you are, like
it's your showreel for a résumé. go all out." The most-liked piece in athemeroy's sample uses
it: 16k likes and 1.6M views at the time of their refresh
([@stephanlivera](https://x.com/stephanlivera/status/2103315922098470926), "Opus 5.5 on Max
effort"). Movez calls the result "brief contagion": many near-identical reels
([article](https://x.com/0xMovez/article/2104216919033192746)).

**By kind, for the kinds the ticket names:**

- **Product ads and launch videos.** This is the biggest practical use. Many prompts send the
  agent to a product's own URL to fetch its screenshots and logo. Examples:
  [@deedydas](https://x.com/deedydas/status/2102787937482252537) ("1min for ~$2", self-reported),
  [@moritzkremb](https://x.com/moritzkremb/status/2103066071838466494),
  [@Miguel07Code](https://x.com/Miguel07Code/status/2102441708395041170),
  [@leodev](https://x.com/leodev/status/2102897952587133299) and
  [@bridgemindai](https://x.com/bridgemindai/status/2102462889160286423).
- **UI demos.** The standout form is the "one shape, never cut" morph: a single UI element
  morphs through 8 to 12 states, driven by a cursor. Examples:
  [@twoclipping](https://x.com/twoclipping/status/2103273003555402193) (quoted in full by
  [@__morse](https://x.com/__morse/status/2103485566570369333)) and
  [@verbove](https://x.com/verbove/status/2103483957266268381).
- **Kinetic type.** Kinetic type is inside almost every ad. Examples: the Muda film's
  full-width claim cards (poster frame, @deedydas), and "the hook lands word by word on the
  beats" (Hooklab).
- **Logo reveals.** Usually these are the last beat of an ad. One prompt asks for a reveal on
  its own: wireframe logo → low-poly shards → icon → typed brand name
  ([@Mounnna](https://x.com/Mounnna/status/2103802871934497266)).
- **Explainers.**
  - "How browsers work" in 40 s: [@addyosmani](https://x.com/addyosmani/status/2103009037164110327).
  - A Negroni recipe, with a pour-to-measurement-line and a step timeline:
    [@Ror_Fly](https://x.com/Ror_Fly/status/2102853258582880547).
  - Recursion shown in nine styles: [@emollick](https://x.com/emollick/status/2103688362960019567).
  - Long Manim and Transformer lessons. Educational explainers had the longest median preview
    in athemeroy's cases, at 102 s.
- **Data viz.** Examples include a neural-network training visual
  ([@DotCSV](https://x.com/DotCSV/status/2102737776219168939)) and token-analytics videos with
  "checked data slots" ([@arambarnett](https://x.com/arambarnett/status/2104011150471917838)).
  Data viz is less common as a piece of its own than as charts inside ads.
- **Social shorts.** Most are 9:16 versions of ads. One is a talking-head clip "cut into a
  punchy, fun edit with subtitles, graphics and music", which took about 1 h 51 min, about
  $23 of tokens and $49 of fal ([@sab8a](https://x.com/sab8a/status/2103144778481475686)).

**Length.** The showreel prompt fixes 15 s. Ad prompts ask for 15 to 30 s. Across the 1,401
classified files, the median preview is 39 s (athemeroy
[statistics](https://github.com/athemeroy/awesome-opus-5-5-videos/blob/main/docs/statistics.md)).

## 4. What makes the admired ones good

These are the creators' own stated rules, from the most-copied prompt specs, plus what I could
see in the poster frames. None of it is measured.

1. **Real material, never placeholders.**
   - "Real UI. Real data. No placeholders." ([@verbove](https://x.com/verbove/status/2103483957266268381))
   - "Real footage only, never placeholder cards." ([@twoclipping, Hooklab](https://x.com/twoclipping/status/2102554209166000267))
   - "without it, opus draws your product UI from scratch and it looks off. wrong spacing,
     placeholder boxes, fake-looking buttons" ([@rexan_wong](https://x.com/rexan_wong/status/2103707054108299437))
   - @leodev's list adds "Have a good landing page" and "good branding"
     ([post](https://x.com/leodev/status/2102897952587133299)).

   In the MakerMap poster frame, the "matches" card shows real names and avatars.
2. **A reference, not an adjective.** @rexan_wong: "without a reference, opus falls back to
   its default look: centered text, gradient background, everything fading in … naming a style
   works way better than describing one". Movez repeats this
   ([article](https://x.com/0xMovez/article/2104216919033192746)).
3. **Timing on a beat grid.** "120 BPM grid. Something happens on every beat" (@verbove).
   "Every cut sits on a downbeat, every UI hit on a beat" (Hooklab). The iArt write-up says
   the films that feel on the beat put "the tempo, the shot list and a list of events … in one
   data structure that the drawing code and the music code both read"
   ([iArt](https://www.iart.ai/blog/ai-javascript-animation)).
4. **Springs with little overshoot, and continuity instead of cuts.**
   - "Closed-form springs everywhere with only a tiny overshoot" (@verbove).
   - "One shape, never cut" (@twoclipping UI morph).
   - "Content enters after its container starts morphing and leaves before the next morph so
     text never overlaps" (@verbove).
   - "Never fade black directly into the accent color. Move an accent element between states
     instead" (@verbove).
5. **Restraint, and a list of what not to do.**
   - "One idea per shot, lots of empty space, one accent color, one clean sans … Masked type
     reveals, match cuts, one smooth camera language."
   - "Banned: shockwave rings, particle bursts, RGB split, camera shake, lens flares, neon
     glows, grid floors, flashing backgrounds, bouncy easing." (Hooklab)
6. **No "AI chrome".** Several prompts ban the corner labels that the model otherwise adds:
   "Avoid the frames and texts on the corners which are typical ai made giveaways!"
   ([@souravbhar871](https://x.com/souravbhar871/status/2103711849703477361)), and "avoid the
   border text or frames!" ([@1littlecoder](https://x.com/1littlecoder/status/2103746736980378066)).
   The default is visible in the two most-liked one-liners. The Muda frame has
   `MUDA / FILM 01`, `T+18.300s` and `1920×1080 · 30 FPS · 120 BPM` in its corners. The
   showreel frame has `CLAUDE MOTION REEL — 2026`, `00:00:09:03 60 FPS` and `128 BPM BAR 5/8`
   (the yihui list's poster frames for
   [@deedydas](https://x.com/deedydas/status/2102787937482252537) and
   [@stephanlivera](https://x.com/stephanlivera/status/2103315922098470926); one frame each).
7. **Motion blur.** Hooklab and MakerMap both specify sub-frame sampling (3 to 6 subframes,
   blended with ffmpeg `tmix`) for real motion blur at 60 fps.

## 5. Workflow the agents used

Creators converge on the same loop. In rough order of how often it's cited:

1. **Plan before code.** Ask for 3 storyboard variants
   ([@rexan_wong](https://x.com/rexan_wong/status/2103707054108299437)), "a storyboard with
   every timing on the beat grid before you write any code" (Hooklab), or "a beat sheet and 4
   keyframes only; write the file after I approve"
   ([HF blog](https://huggingface.co/blog/karmen-beatapi/how-to-make-videos-with-claude-opus-5-5)).
2. **Render stills before motion.** "ask for one still frame per scene before anything moves.
   fixing a storyboard is way cheaper than fixing a render" (@rexan_wong).
3. **Look at a contact sheet.** "First render one frame per beat as a contact sheet. Fix
   anything cramped or broken" (@verbove). "Probe 20 or more frames before the full render"
   (Hooklab). The [gist](https://gist.github.com/remorses/3d467b50a0519ef7859823046dc9c427) has a
   `render.ts beats` mode that writes `out/beats.png`. A small controlled test
   ([samuellawrentz/profile#132](https://github.com/samuellawrentz/profile/pull/132)) compared a
   one-shot with a contact-sheet critique loop. The loop caught text drawn at y≈726 on a 720 px
   canvas, which the one-shot had shipped. It cost $1.02 against $0.32. The author notes that
   the critique prompt also contained a cliché checklist, so the two effects can't be
   separated.
4. **Iterate with director's notes.** "slow every zoom to 0.7x", "hard cut here", "push in on
   the button". "the first render is usually 80% there" (@rexan_wong). One creator's second
   prompt: "it is a bit too fast and bumpy"
   ([@sachaarbonel](https://x.com/sachaarbonel/status/2103614797501673648)).
5. **Give it context and components.** Build inside the product's codebase and use an
   animation component library such as remocn or 21st.dev
   ([@leodev](https://x.com/leodev/status/2102897952587133299), @rexan_wong). Use high effort:
   Movez reports every viral one-shot ran on xhigh or max.
6. **Long autonomous runs are real, but they are not one prompt.** One music video took
   "11 subagents, 23.5 hours, 3k tool calls, … ~82 notes from me"
   ([@dhvanil via @fabianstelzer](https://x.com/fabianstelzer/status/2103466607804862796)).

**Montagent implications.**

- The contact-sheet step is the most-cited quality lever, and Montagent's range mode for
  `frame` is decided but not shipped (inventory flag 1, [#395](https://github.com/MBehtemam/Montagent/issues/395)).
  Until it ships, a skill has to teach looking at single frames, one per beat, with
  `frame --at`.
- The beat grid comes from outside Montagent: the agent analyses the song and writes literal
  times.
- The "−14 LUFS loudnorm" step has no counterpart. Montagent mixes at `normalize=0`
  (inventory, Audio row).

## 6. Representative pieces mapped to Montagent

Grades are measured against the
[capability inventory](https://github.com/MBehtemam/Montagent/blob/research/capability-inventory/docs/research/skills/capability-inventory.md):

- **Today**: the obvious or findable surface says it; a competent agent gets there unaided.
- **Craft**: expressible, but only through buried combinations or workarounds a skill would
  teach.
- **Not**: a missing capability blocks the piece as made. "Part" marks a piece where most of it
  is Craft and one named element is Not.

The source of each piece is linked in its row.

| # | Piece (kind, medium) | Grade | What carries it in Montagent / what blocks it |
|---|---|---|---|
| 1 | [@deedydas Muda inference launch](https://x.com/deedydas/status/2102787937482252537): 26 s kinetic type, charts, logo, UI cards (HTML/GSAP per tags) | **Craft** | Per-phrase `text` elements with staggered starts and an overshoot bezier. Bar charts as `rect`s scaled about `bottom-*` origin. Cards as radius `rect`s with `shadow`. Logo `image`. A *line* chart would be Not (no paths). |
| 2 | [@Miguel07Code Shotbase launch](https://x.com/Miguel07Code/status/2102441708395041170): 58 s screenshots, controls, typography (HyperFrames) | **Craft** | Screenshots as `image`s inside a `mask` rect with `radius`, pushed in by `scale` keyframes (Ken Burns). `transition` crossfades. Product mark reveal. Recreating live DOM UI is out of scope; screenshots stand in. |
| 3 | [@bridgemindai merch ad](https://x.com/bridgemindai/status/2102462889160286423): 30 s logo, hoodie stills, price, shop UI (Remotion) | **Craft** | Product-photo Ken Burns (`clip` + `scale`, keyframes overrunning `end`). Price pop with overshoot. Colour `highlight` on the price run. Music bed with a keyframed fade. |
| 4 | [@twoclipping Hooklab ad](https://x.com/twoclipping/status/2102554209166000267): 20 s, 10 bars at 120 BPM, clip wall, masked type, 3D carousel, logo (HTML+Playwright+ffmpeg) | **Part** | Craft: clip wall (`video` × N with `clip`), a circle "opening out of the button" (`ellipse` scaled up), stats on push cuts, logo reveal, SFX as `audio` at beat times. Not: **3D carousel with floor reflections** (no 3D or perspective), **motion blur**. Masked type reveals only as an occluder rect on a flat background (no animatable `clip`/mask). |
| 5 | [@twoclipping UI morph](https://x.com/twoclipping/status/2103273003555402193) / [@verbove MakerMap](https://x.com/verbove/status/2103483957266268381): "one shape, never cut", 8–12 states, springs, cursor | **Not** | The morph *is* animated width, height, corner radius and colour (visible in the [gist](https://gist.github.com/remorses/3d467b50a0519ef7859823046dc9c427)). Montagent can't animate `width`/`height`, `radius` or colour (inventory: "not animatable"). `scale` distorts corners and content. Also blocked: short transition blur (no animated effect parameter) and motion blur. The cursor and closed-form springs (many keyframes) are expressible. |
| 6 | [@stephanlivera showreel](https://x.com/stephanlivera/status/2103315922098470926): 15 s generative Truchet tiles, waves, 128 BPM (canvas) | **Not** | Generative arc tiling needs arcs or paths and per-frame procedural motion over hundreds of elements. There are no paths (ADR-0014) and no generative primitive. |
| 7 | [@ajith_io showreel](https://x.com/ajith_io/status/2103449416325890146): 15 s red dot, wordmark, abstract shapes (canvas) | **Craft** | Per athemeroy's frames: `ellipse`/`rect` choreography, kinetic wordmark, end card. Overshoot eases, `step` flashes, `rotation` turns. Any generative passages would be Not. |
| 8 | [@Ror_Fly Negroni recipe](https://x.com/Ror_Fly/status/2102853258582880547): 30 s pour to measurement lines, six-step timeline (JS canvas) | **Craft** | Glass as a supplied `image`. Liquid as a `rect` scaled about `bottom-center` between the glass layers. Step dots as `ellipse` + check `image`, popping with overshoot. Progress bar as a `scale` wipe about `center-left`. Step card text swapped by `step`. Hand-drawn line wobble is Not. |
| 9 | [@addyosmani "how browsers work"](https://x.com/addyosmani/status/2103009037164110327): 40 s diagram explainer (JS) | **Craft** | Boxes as `rect`s. Straight connectors as thin `rect`s with `rotation`, "drawn on" by `scale` about one end. Labels measured with `measure`. Curved arrows and DOM-tree edges at arbitrary angles work only as rotated straight bars. |
| 10 | [@sab8a talking-head edit](https://x.com/sab8a/status/2103144778481475686): subtitles, graphics, music (OpenEdit + fal) | **Craft** | `video` cuts with `source_start`, karaoke captions (one run per word, `highlight` windows), a lower-third, a music bed ducked by keyframed `volume`. The inventory grades all of these buried. A green-screen presenter over graphics is `chroma`. fal-generated inserts are out of scope. |
| 11 | [@LinearUncle Manim derivative lesson](https://x.com/LinearUncle/status/2103128559174971663): graphs, tangents, formulas, TTS | **Not** | Function plots need polylines or paths. Formulas need math typesetting. A tangent line alone would be a rotated `rect`, and TTS audio would be an `audio` element. |
| 12 | [@aicreataro AE effects](https://x.com/aicreataro/status/2102656273112326609): beat cuts plus particles, kaleidoscope, datamosh on generated footage | **Part** | Beat-cut `video` segments are Today. Particle, light, kaleidoscope and datamosh effects are Not (the effect set is `blur`, `shadow`, colour filters, `chroma`, `mask`). |
| 13 | [@Aurelien_Gz Clearwater](https://x.com/Aurelien_Gz/status/2102786378282987591) (WebGL2 water) and other 3D and demoscene pieces | **Not** | Shaders, 3D and real-time simulation. Outside Montagent's model, and deliberately so. |

**Pattern.** Montagent can express the **ad / launch / explainer / social-edit** core of the
trend (rows 1–4, 7–10) once the craft is taught. That core is where the "ads × motion
graphics / UI" cluster sits. What it can't express falls into five gaps:

1. **Morphing properties**: animated size, radius or colour.
2. **Paths**: line charts, curves, arcs, plots.
3. **Generative, shader and 3D work.**
4. **Blur over time**: motion blur and animated blur.
5. **Animated mattes**: masked reveals.

All five are already recorded as deliberate v1 deferrals in inventory flag 8 ("no issue
proposed unless the eval shows the ceiling costs the ad"), so this research raises no new
product-gap issue. It does note that gap 1 blocks the single most-copied *prompt pattern* of
the week (the UI morph), and gap 4 blocks a technique the top prompt specs ask for by name.

## 7. Candidate eval briefs

Each brief is drawn from a piece people actually made, runs 5 to 15 s, and is expressible
today or with taught craft. Together they are meant to touch most of the inventory's
**buried** or **findable-only** capabilities. The last line of each brief lists what it
exercises.

**A. Startup launch spot (12 s, 16:9, 120 BPM).** *From rows 1–3 (@deedydas, @Miguel07Code,
@bridgemindai).*

- Five hook words land one per beat with a small overshoot.
- The last word turns accent-coloured on the downbeat.
- A product screenshot enters in a rounded frame with a soft shadow and pushes in slowly.
- It crossfades to a second screenshot.
- A three-bar chart grows from the baseline.
- A wordmark is revealed by a bar wiping across, then holds.
- A music bed fades out over the last second.
- No corner labels or frame chrome.

*Exercises:* every transform keyframable; bezier overshoot; `highlight` colour flip;
`mask` rect `radius` with `shadow` after the mask (order); Ken Burns with keyframes past
`end`; the `transition` exact-window rule; scale-about-origin wipes; keyframed `volume` and
the `normalize=0` mix; fonts vendoring, with bold as a separate file; landing the last
keyframe on `measure at: end - 1`.

**B. Talking-head social cut (10 s, 9:16).** *From row 10 (@sab8a) and the 9:16 ads in
yihui's list.*

- A green-screen presenter is keyed over a brand-coloured background.
- Captions highlight the active word in time with the speech.
- A lower-third name bar wipes on at 1 s and off at 6 s.
- A circular picture-in-picture of the product pops in at the right.
- A music bed sits under the voice, dips while the presenter speaks and fades out at the end.

*Exercises:* `chroma` before the colour filters, and `measure` for coverage drift; karaoke
(one run per word, `highlight` windows inside the element's range); the lower-third recipe;
`mask` circle on a square rect; manual ducking; `R-CAPTION-*` review noise that the agent has
to read as facts, not verdicts; learning source dimensions from `E-FIT-DEVIATION`.

**C. Recipe / process explainer (15 s, 1:1).** *From row 8 (@Ror_Fly) and @alex_prompter's
five-scene explainer prompt in yihui's list.*

- A glass fills to three marked levels, one ingredient per beat.
- A six-step timeline along the bottom checks off each step with a pop.
- A progress bar fills under the current step card.
- The step card's number and title change on the beat.
- All six step units look identical except their label.

*Exercises:* `group` with `query --where`/`--census` as a consistency audit across repeated
units; `t_from: after-previous` for derived timelines; the `step` ease for swaps; scale about
`bottom-*`/`center-left` origins for the fill and the bar; `measure` for text boxes and `\n`
placement; generating the file and then running `fmt` at this element count.

**D. Product-photo ad with a flashback grade (10 s, 16:9).** *From row 3 (@bridgemindai) and
the "before / after" launch pieces.*

- Four product photos crossfade with slow Ken Burns moves.
- The first two are graded as a faded sepia "before".
- The last two are in full colour.
- A price tag pops in with an overshoot.
- The piece loops seamlessly: the last frame matches the first.

*Exercises:* colour-filter recipes (grayscale = `saturation 0`, sepia = that plus warm
`tint`); `fit: cover` checked against real dimensions; alternating tracks for crossfades;
`loop: true` and `R-SOURCE-CUT-POP` at the wrap; `compare` after a snapshot when revising the
grade.

**E. Logo reveal loop (6 s, 1:1).** *From @Mounnna's logo-reveal prompt and the "last frame
equals first" rule in the UI-morph specs.*

- A mark is assembled from four shapes that fly in and settle with a stagger.
- One shape spins three full turns on its way in.
- A cursor blinks after the brand name.
- The brand name then types on letter by letter.
- The loop closes back to an empty frame.

*Exercises:* per-letter elements (there is no per-glyph animation); `rotation` not normalised
(`1080`); the `step` ease for the blink; `shift` to insert a beat, which can only insert
time; `loop: true`.

**Coverage check against the inventory's buried or findable-only rows:**

| Capability | Covered by |
|---|---|
| Keyframes on any transform property | A–E |
| Overshoot bezier | A, C, D, E |
| `step` ease | C, E |
| Keyframes past `end` | A, D |
| `t_from` | C |
| Effects order | A, B |
| Colour-filter recipes | D |
| `mask` circle and rect with radius | A, B |
| `chroma` | B |
| Audio and ducking, `normalize=0` | A, B |
| `transition` window rule | A, D |
| `highlight` / karaoke | A, B |
| Fonts vendoring, bold as a file | A |
| `measure at` for half-open time | A |
| `group` + `query` census | C |
| `compare` after a snapshot | D |
| `shift` inserts only | E |
| `loop` | D, E |
| Learning media numbers from findings | B, D |

Not covered: remote sources and `runs[].dir`. Both are product-gap flags, not craft.

## Sources not used, and gaps

- **X articles.** I read the Movez "full course" through the fxtwitter mirror and cite it
  only for its reporting of other posts, which I checked where I could. Two items could not be
  read: the NeilXbt course (X article) returned an error, and the pasqualepillitteri.it
  summary did not load.
- **Video.** No video was watched. Motion quality, audio quality and continuity are
  unverified beyond the creators' own statements and athemeroy's nine-frame notes.
- **Engagement.** Like and view counts are dated observations (athemeroy, 2026-09-27; fxtwitter,
  2026-09-29). They are not quality scores.
