---
status: accepted
amends: 0017 (confirms the closed project schema holds against templates, presets and expression controls; a skill script paired with its standalone `<spec>` JSON sidecar is the complete answer, with no in-document keys and no project-level sidecar manifest), 0177 (codifies the general skill-script contract in `skills/README.md` and extends it to `skills/montagent-motion/scripts/keyframes.py` for motion bakes and keyframe assistants), 0215 (disposes of the motion-bake, keyframe-assistant, measure-then-write auto-colour, and template/MOGRT/Essential-Properties rows from the After Effects censuses on map #896)
---

# Motion bakes and keyframe assistants enter in one `keyframes.py` script, auto-colour is refused, and a skill script with its `<spec>` is a template

> **Amended by [ADR-0226](./0226-stroke-taper-boolean-paths-and-path-modifiers-are-write-time-scripts-and-trim-is-single-contour.md)**: Stroke taper, boolean paths, and path modifiers are write-time skill scripts, and trim remains single-contour.

> **Amended by [ADR-0219](0219-smooth-pass-through-easing-enters-as-an-auto-bezier-pass-and-recipe-and-the-runtime-evaluator-stays-local.md)**: extends `skills/montagent-motion/scripts/keyframes.py` with an `auto_bezier` pass for monotonic cubic spline easing, alongside a slope-matching recipe in `skills/montagent-motion/SKILL.md` ([#1061](https://github.com/MBehtemam/Montagent/issues/1061)).

[Which motion-bake, keyframe-assistant and measure-then-write skill scripts enter, and how do templates work?](https://github.com/MBehtemam/Montagent/issues/1059)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).
[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md)
settled that After Effects is precedent that a rendered result is wanted, while every capability
keeps the four invariants ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md):
literal values, closed vocabulary, checkable by `validate`, and exact-string replace), zero live
references ([ADR-0086](0086-a-keyframe-is-an-instant-a-value-and-an-ease-and-nothing-else.md)), and
no element-level repeat or write-time stagger expander ([ADR-0148](0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)).
Where an After Effects capability is an expression, a keyframe assistant, a pixel-measurement button,
or a parametric template, it cannot enter the format as a runtime formula or live link. Its candidate
home in Montagent is a **skill script** ([ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md))
or a written recipe in a job skill.

Across four of map [#896](https://github.com/MBehtemam/Montagent/issues/896)'s area censuses —
[`expressions-and-presets.md`](../research/after-effects/expressions-and-presets.md)
([#905](https://github.com/MBehtemam/Montagent/issues/905)),
[`animation-and-keyframes.md`](../research/after-effects/animation-and-keyframes.md)
([#899](https://github.com/MBehtemam/Montagent/issues/899)),
[`effects-colour-and-stylize.md`](../research/after-effects/effects-colour-and-stylize.md)
([#902](https://github.com/MBehtemam/Montagent/issues/902)), and
[`shapes-and-paths.md`](../research/after-effects/shapes-and-paths.md)
([#900](https://github.com/MBehtemam/Montagent/issues/900)) — four groups of capabilities were routed to
[#1059](https://github.com/MBehtemam/Montagent/issues/1059):

1. **Motion and expression bakes:** seeded `wiggle(freq, amp)` / *The Wiggler* / shape *Wiggle Transform*
   ([ADR-0216](0216-shape-trim-joins-the-shape-types-and-the-effect-bound-list-and-shape-operators-are-refused.md) §3)
   and post-keyframe decaying inertial bounce (*Overshoot*); keyframe loops (`loopIn` / `loopOut` —
   `cycle`, `pingpong`, `offset`, `continue`); and delayed followers (`valueAtTime(t - d)`),
   *Sequence Layers*, and keyframe time-stretching / reversing.
2. **Keyframe assistants:** *Convert Audio to Keyframes*, keyframe *Smoother* / tolerance reducer,
   and *Exponential Scale*.
3. **Measure-then-write colour helpers:** *Auto Levels*, *Auto Contrast*, and *Auto Color*.
4. **Templates, MOGRTs, Essential Properties, and Expression Controls:** Motion Graphics Templates
   (`.mogrt`), Essential Graphics, Master/Essential Properties on nested compositions, Expression
   Controls (`Slider`, `Angle`, `Checkbox`, `Color`, `Point`, `3D Point`, `Layer`, `Dropdown Menu`),
   and `.ffx` animation presets.

This ADR decides which of these enter as shipped skill scripts, which stay written recipes in a job
skill, which are refused, how the admitted motion scripts are packaged, and how `<spec>` sidecar files
answer templates and presets.

## Decisions

### 1. Motion and expression bakes: `wiggle`, `bounce`, and `loop` (`cycle`, `pingpong`, `offset`) enter as a skill script; `loopOut("continue")` and `Time-Reverse Keyframes` are skill recipes; delayed copies, *Sequence Layers*, and time-stretching are refused

1. **Seeded `wiggle(freq, amp)` and decaying inertial bounce (`bounce`) enter as script passes.**
   [`skills/montagent-motion/references/trailer.md`](../../skills/montagent-motion/references/trailer.md)
   already tells an agent never to hand-write a shake or jitter table and to generate it with a small
   script instead, because hand-typing dense frame-by-frame keyframes in `.montagent.json` bloats
   context, drifts off drawn-frame boundaries (`⌊n × 1000 / fps⌋`), and leaves un-thinned keyframe
   runs that make subsequent exact-string replacements brittle. Shipping both passes in
   `skills/montagent-motion/scripts/keyframes.py` gives every agent deterministic seeded smooth or
   stepped noise (`wiggle`, covering AE's `wiggle(freq, amp)`, *The Wiggler*, and shape
   *Wiggle Transform*) and post-keyframe damped harmonic settle (`bounce`, covering AE's inertial
   *Overshoot* / bounce expression) sampled on drawn frames and thinned by Ramer–Douglas–Peucker
   `simplify(samples, tol)`.
2. **Keyframe `loop` (`cycle`, `pingpong`, `offset`) enters as a script pass; `loopOut("continue")`
   and `Time-Reverse Keyframes` are written recipes in `skills/montagent-motion/SKILL.md`.**
   Repeating an authored keyframe slice across `[start, end]` in `cycle`, `pingpong`, or `offset`
   mode is mechanical, and `pingpong` has an invariant trap that agents routinely miss when editing
   JSON by hand: because Montagent attaches `ease` as the *incoming* ease on the destination keyframe
   ([ADR-0038](0038-ease-is-required-on-every-non-first-keyframe-record.md)), reversing a multi-segment
   keyframe slice requires moving each segment's ease to the opposite end of that segment and
   mirroring raw cubic beziers (`[x1, y1, x2, y2]` $\to$ `[1 - x2, 1 - y2, 1 - x1, 1 - y1]`,
   swapping `ease-in` $\leftrightarrow$ `ease-out`). By contrast, `loopOut("continue")` is a single
   `linear` keyframe at `end` with value $v_{\text{last}} + \frac{v_{\text{last}} - v_{\text{prev}}}{t_{\text{last}} - t_{\text{prev}}}(t_{\text{end}} - t_{\text{last}})$,
   and reversing a two-keyframe transition on its own is a one-line formula once documented. Both are
   written down as recipes in `skills/montagent-motion/SKILL.md`.
3. **Delayed followers (`valueAtTime(t - d)`), *Sequence Layers*, and keyframe time-stretching are
   refused as shipped scripts.**
   [ADR-0148](0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md) §3
   already refused a write-time stagger/repeat expander because an agent editing JSON directly (or
   running a 3-line Python loop when the count is large) does not benefit from learning another tool's
   flag or spec grammar for `t + i * delay` arithmetic. The same ground governs `valueAtTime(t - d)`
   echoes/trails, *Sequence Layers* (`start_i = end_{i-1} - overlap`), and linear keyframe
   time-stretching (`t' = t0 + round((t - t0) * factor)`).

### 2. All three keyframe assistants enter, packaged with §1's motion bakes in one `skills/montagent-motion/scripts/keyframes.py <project> <spec>` script

All three After Effects keyframe assistants enter and share a single script with §1's motion bakes:
`skills/montagent-motion/scripts/keyframes.py <project> <spec>`.

1. **Admitted keyframe-assistant passes:**
   - **`audio` (*Convert Audio to Keyframes*):** Decodes an audio source (by `source` path or by
     referencing an existing `audio` or `video` element's `id` and placed window) via `ffmpeg` into
     mono PCM float samples over `[start, end]`, computes per-drawn-frame RMS amplitude in dBFS over a
     configurable window (default one frame period), maps `[floor_db, peak_db]` linearly onto
     `[min_v, max_v]` (clamped), thins the resulting curve via `simplify(samples, tol)`, and writes
     ordinary keyframes onto the target property. Supports `--check` (exiting `1` if the keyframes in
     `<project>` do not match what the spec and audio source currently produce, matching `duck.py`'s
     [ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) §5
     contract).
   - **`smooth` (Keyframe *Smoother* / reducer):** Evaluates an existing dense keyframe slice (or raw
     sampled points) on drawn frames over `[start, end]`, applies an optional rolling-window box filter
     of radius `window_ms`, and thins the curve with Ramer–Douglas–Peucker `simplify(samples, tol)`.
   - **`exp_scale` (*Exponential Scale*):** Samples the optical constant-rate zoom curve
     $s(t) = s_0 (s_1 / s_0)^{(t - t_0)/(t_1 - t_0)}$ (for $s_0, s_1 > 0$) on drawn frames across
     `[start, end]` and thins it with `simplify(samples, tol)` so deep zooms ($2\times$ and above) move
     at constant visual speed without adding an exponential ease to [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md)'s
     closed ease vocabulary.
2. **One unified script (`skills/montagent-motion/scripts/keyframes.py <project> <spec>`):**
   - Every pass (`wiggle`, `bounce`, `loop`, `smooth`, `exp_scale`, and `audio`) targets an
     `(element_id, property, [start, end])` slice — where `property` names any numeric transform field,
     pair component, shape field, or numeric effect parameter admitted by
     [ADR-0146](0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md) —
     quantizes sample times to drawn frames (`⌊n × 1000 / fps⌋`), runs `simplify(samples, tol)`,
     replaces the keyframe slice in `[start, end]` idempotently, and validates the output with
     `montagent validate` before printing the updated project JSON to stdout.
   - Packaging all six passes in `keyframes.py` means an agent learns one entrypoint and one `<spec>`
     format, and one `<spec>` sidecar can own all generated motion keyframes on an element (for
     example, an `audio`-driven `scale` pulse alongside a `wiggle` on `x`/`y` or an `exp_scale` zoom)
     without two scripts or two sidecar files clobbering each other's keyframes. `ffmpeg` is invoked
     lazily only when the `<spec>` includes an `"audio"` pass, so pure-math specs have zero media
     dependency.

### 3. A measure-then-write auto-colour script (*Auto Levels* / *Auto Contrast* / *Auto Color*) is refused

No script or `ffprobe`/`signalstats` recipe is added for *Auto Levels*, *Auto Contrast*, or
*Auto Color*.

- Measurement scripts enter Montagent only when the authoring agent has a sensory or metric blind spot
  that it cannot read from the rendered frame: `duck.py` ([ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md))
  and `beat_grid.py` decode audio because an LLM cannot hear an audio file without `ffmpeg`, and
  `montagent measure` computes exact font ink bounds because text layout cannot be eyeballed to the
  pixel before drawing.
- By contrast, a multimodal agent sees rendered video and image frames directly through
  `montagent frame`. Blind percentile histogram stretching (`signalstats` `YLOW`/`YHIGH` or RGB
  percentiles) is routinely fooled by letterbox bars, alpha overlays, specular highlights, and
  intentionally high-key or low-key cinematography, writing `brightness`, `contrast`, or `cdl` numbers
  into `<project>` that the agent immediately has to inspect with `frame` and undo via exact-string
  replace. Colour grading stays visual in `skills/montagent-footage/SKILL.md`.

### 4. A skill script paired with its standalone `<spec>` sidecar is Montagent's complete answer to templates, MOGRTs, Essential Properties, Expression Controls, and `.ffx` presets

1. **Nothing enters `<project>`, and no project-level sidecar manifest is added.**
   Amending [ADR-0017](0017-closed-schema-no-escape-hatch.md) to settle its open sidecar question for
   parametric authoring: Montagent does not add template definitions, preset references, exposed
   *Essential Properties*, or inert *Expression Control* fields (`Slider Control`, `Checkbox Control`,
   `Color Control`, `Layer Control`, `Dropdown Menu Control`) to `.montagent.json`, nor does it add a
   project-level sidecar registry or manifest linking `<spec>` files to element `id`s.
2. **A skill script plus its `<spec>` JSON file is the template.**
   In After Effects, a `.mogrt`, *Essential Graphics* panel, or *Expression Control* effect exists so a
   human in a GUI has sliders, checkboxes, and dropdowns that drive underlying layers via expressions.
   In Montagent, the author is an agent editing JSON:
   - The `<spec>` JSON file (such as `rig.spec.json`, `type_on.spec.json`, `captions.spec.json`, or
     `keyframes.spec.json`) is the exposed parameter sheet (the *Essential Properties* / *Expression
     Controls*).
   - The skill script (`bake_rig.py`, `type_on.py`, `captions.py`, `duck.py`, `keyframes.py`) is the
     parametric template or preset that expands those parameters into ordinary literal elements and
     keyframes in `<project>`.
   - To re-parameterize an animation, the agent edits `<spec>` and re-runs the script, which replaces
     its target elements or keyframe slices by `id` idempotently — the workflow 6 of 6 baseline agents
     already chose naturally in [#591](https://github.com/MBehtemam/Montagent/issues/591).
3. **Standard skill-script contract in `skills/README.md`:**
   `skills/README.md` codifies the permanent skill-script contract established across
   [ADR-0177](0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md)
   and the shipped scripts:
   - **Location and runtime:** `skills/<skill>/scripts/<name>.py`, executable Python $\ge 3.9$,
     standard library only, plus `ffmpeg` on `PATH` when a script or pass decodes media.
   - **Invocation and output:** `<script>.py <project> <spec>` (or `<project>` with CLI flags for
     single-purpose media tools such as `duck.py`), writing the updated project JSON to **stdout** and
     errors to **stderr** without mutating `<project>` in place unless explicitly designed around
     `montagent measure --write`.
   - **Idempotent re-run:** Re-running a script with an edited `<spec>` replaces the target elements
     (matched by `id`) or target property keyframe slices (or via `--replace` on flag-driven scripts)
     cleanly, so the `<spec>` sidecar remains the repeatable source of truth.
   - **Staleness check (`--check`):** Scripts or passes that derive keyframes from external media or
     timing files (`duck.py`, `keyframes.py`'s `audio` pass) support `--check`, exiting `0` when
     `<project>` matches the current inputs and `1` when stale.
   - **Validation and CI guard:** Every script validates its output by running `montagent validate` on
     a temporary file before printing to stdout, and carries a `# drift-guard:` docstring header tested
     by `crates/montagent/tests/skills.rs`.

### 5. `CONTEXT.md` gains **Skill script** under `## Language` and **Template, preset** under `## Rejected terms`

- Under `## Language` in [`CONTEXT.md`](../../CONTEXT.md), add **Skill script** beside **Job skill**
  and **Workaround** to distinguish a permanent helper script (`duck.py`, `captions.py`, `type_on.py`,
  `bake_rig.py`, `keyframes.py`) from a temporary **Workaround** (`beat_grid.py`).
- Under `## Rejected terms` in [`CONTEXT.md`](../../CONTEXT.md), add **Template, preset** (with
  `_Avoid_: MOGRT, Essential Properties, master properties, expression control, animation preset`)
  explaining that a **Skill script** paired with its standalone `<spec>` JSON sidecar is how parametric
  motion and reusable rigs work in Montagent, while `.montagent.json` holds only the resulting literal
  elements and keyframes.

## Evidence: three-juror court

A three-juror court (`inherit`, `pro`, `flash`) was convened under `.agents/skills/court/SKILL.md` to
evaluate all four questions from the perspective of an authoring agent working with `.montagent.json`,
skill scripts, and `<spec>` sidecars. The ballots are recorded verbatim below:

```
🗳️ **Juror 1** (inherit) — **VOTE: Q1: Option A | Q2: Option A | Q3: Option A | Q4: Option A**

**Reasoning:**
- **Q1 (Motion and expression bakes):** **Option A** matches the exact boundary where an LLM agent's native JSON-editing workflow breaks down. Emitting dense frame-by-frame keyframes for seeded `wiggle(freq, amp)` or decaying post-keyframe bounce directly via exact-string replace bloats context, drifts off drawn-frame boundaries, and leaves dozens of un-thinned keyframes in `.montagent.json` that make subsequent edits brittle. Likewise, `loopIn`/`loopOut` (`cycle`, `pingpong`, `offset`) looks deceptively simple until the agent hits `pingpong`: because Montagent attaches `ease` as the *incoming* ease on the destination keyframe, reversing a slice requires shifting eases across segment boundaries and mirroring cubic beziers (`[x1, y1, x2, y2] -> [1 - x2, 1 - y2, 1 - x1, 1 - y1]`), which agents routinely botch by hand. By contrast, `loopOut("continue")` is a single linear extrapolation keyframe, reversing a 2-keyframe transition is a one-line formula once documented in `SKILL.md`, and delayed followers (`valueAtTime`), *Sequence Layers*, and time-stretching are 2–5 literal elements/keyframes that an agent edits faster in JSON (or with a 3-line loop for large `N`, per `ADR-0148`) than by authoring a `<spec>`.
- **Q2 (Keyframe assistants and script packaging):** **Option A** admits all three assistants (*Convert Audio to Keyframes*, *Smoother*/reducer, and *Exponential Scale*) and packages them with Q1's bakes in a single `skills/montagent-motion/scripts/keyframes.py <project> <spec>`. From an agent workflow standpoint, all five operations (`wiggle`, `bounce`, `loop`, `smooth`/`simplify`, `exp_scale`, and `audio`) do the exact same job: compute or transform a keyframe curve on a target `(element_id, property, [start, end])`, quantize to drawn frames, thin via Ramer–Douglas–Peucker `simplify(samples, tol)`, replace the slice idempotently, and run `montagent validate`. A unified script means the agent learns one CLI entrypoint and one `<spec>` schema, and can combine an audio-reactive `scale` pulse with a `wiggle` on `x`/`y` or an `exp_scale` zoom in a single `<spec>` sidecar without two scripts clobbering each other's keyframes.
- **Q3 (Measure-then-write colour helper):** **Option A** refuses an auto-colour script because multimodal LLM agents already inspect rendered pixels directly via `montagent frame`. Audio needed `duck.py` and `beat_grid.py` only because an agent cannot "hear" media files without `ffmpeg` decoding; visual grading has no such sensory blind spot. In practice, blind `signalstats` percentile stretching (*Auto Levels* / *Auto Contrast*) gets tricked by letterbox bars, specular highlights, alpha graphics, and intentionally moody high/low-key shots, writing noisy `brightness`/`contrast`/`cdl` numbers into `.montagent.json` that the agent immediately has to inspect with `frame` and overwrite via exact-string replace anyway.
- **Q4 (Templates, MOGRTs, and sidecar convention):** **Option A** codifies what agents already do naturally (`#591` showed 6 of 6 agents editing `<spec>` and re-running the script rather than hand-editing baked output). A standalone `<spec>` JSON file paired with an idempotent skill script (`<script>.py <project> <spec>`, replacing targets by `id`, supporting `--check` where media can drift, and validating before output) *is* Montagent's parametric template and Expression Controls equivalent. Codifying this contract in `skills/README.md` and adding **Skill script** (`## Language`) and **Template, preset** (`## Rejected terms`) in `CONTEXT.md` keeps script interfaces uniform across skills without polluting `.montagent.json`.

**Trade-offs:**
- **Q1:** Choosing **Option A** over **Option B** means an agent doing a 15-element staggered follower trail (`valueAtTime`) or rescaling a 20-keyframe curve still writes a 3-line Python snippet instead of calling a shipped verb; choosing it over **Option C/D** adds maintenance and `drift-guard` test surface for `loop` (`cycle`, `pingpong`, `offset`), which is well worth paying to eliminate `pingpong` incoming-ease/bezier-mirror bugs.
- **Q2:** Choosing **Option A** over **Option B** couples `ffmpeg` (needed only when a `<spec>` uses `audio`-to-keyframes) into the same script as pure-math keyframe bakes, making `keyframes.py` larger (~250–350 lines) than split scripts; however, lazy-calling `ffmpeg` only on `audio` specs avoids runtime cost while sparing the agent from juggling two separate keyframe-writing scripts and two sidecar files on the same element. Choosing **Option A** over **Option C/D** ships *Exponential Scale* and *Smoother* even though their core math is short, because the real boilerplate is drawn-frame sampling + RDP thinning + JSON path dispatch.
- **Q3:** Choosing **Option A** over **Option B/C** means an agent cannot get an instant numeric luma-percentile readout before its first `montagent frame` call on flat/log footage, trading away a mechanical starting guess to avoid misleading histogram auto-grades and unnecessary `ffmpeg` script maintenance.
- **Q4:** Choosing **Option A** over **Option B** means there is no machine-readable manifest linking element `id`s in `<project>` back to the `<spec>` file that generated them (so `montagent validate` cannot automatically warn if an agent hand-edits a baked element without updating its `<spec>`), but it avoids a brittle bookkeeping file that agents frequently forget to update when renaming or deleting elements. Choosing **Option A** over **Option C** requires bringing CLI-flag-only scripts into alignment with the documented skill-script contract.
```

```
🗳️ **Juror 2** (pro) — **VOTE: Q1: Option A | Q2: Option B | Q3: Option A | Q4: Option A**

**Reasoning:**
- **Q1 (Motion and expression bakes):** LLMs struggle with complex generative math (Perlin noise, damped sine) and bezier handle geometry mirroring (`pingpong` ease swaps), so `wiggle`, `bounce`, and `loop` heavily benefit from a dedicated script. However, calculating time offsets (`valueAtTime`, Sequence Layers, time-stretching) is trivial arithmetic. As per `ADR-0148`, the agent is perfectly capable of writing a throwaway Python one-liner or doing exact-string replacements in JSON to shift elements around.
- **Q2 (Keyframe assistants and script packaging):** Splitting media-dependent scripts from pure-math scripts makes the toolset cleaner for an agent to reason about. Audio-to-keyframes requires invoking `ffmpeg` to decode media (fitting perfectly in `montagent-footage` with `duck.py`), whereas pure-JSON keyframe math (`smooth`, `exp_scale`, `wiggle`) belongs in a unified `montagent-motion` script. 
- **Q3 (Measure-then-write colour helper):** Multimodal LLMs can "see." An agent can run `montagent frame`, inspect the rendered visual output, and iteratively adjust the literal colour values. Blind auto-levels/contrast scripts frequently destroy intentional aesthetic choices (like silhouettes or low-key lighting). Visual grading leverages the agent's actual capabilities rather than relying on blunt statistical heuristics.
- **Q4 (Templates, MOGRTs, and sidecar convention):** The standalone `<spec>` sidecar file plus a Python script is already proven to be highly effective for LLMs (6/6 success rate). It perfectly fulfills the role of MOGRTs/expressions while preserving the invariant of a strictly literal, closed-schema project file. Codifying this pattern in the docs gives agents a clear, standard contract without requiring complex, over-engineered manifests.

**Trade-offs:**
- **Q1:** Costs the agent a bit of manual effort (or writing a short ad-hoc script) when sequencing layers or time-stretching, but avoids bloating the core skill scripts with trivial offset logic that duplicates native JSON editing.
- **Q2:** The agent must remember to use two different scripts depending on whether the keyframe source is media (audio) or math, but this cleanly segregates `ffmpeg` dependencies from pure JSON operations.
- **Q3:** The agent spends extra inference turns using `frame` to dial in the grade visually rather than getting a mathematically "normalized" starting point, but this completely avoids the tedious process of undoing bad auto-grades on stylised shots.
- **Q4:** We lack a formalized native link between the `<spec>` file and the `<project>` file, meaning the agent is solely responsible for maintaining both and keeping `id`s aligned. However, this is an acceptable cost that agents already handle gracefully, and it keeps the core schema pure.
```

```
🗳️ **Juror 3** (flash) — **VOTE: Q1: Option A | Q2: Option B | Q3: Option A | Q4: Option A**

**Reasoning:**
- **Q1 (Motion and expression bakes):** Option A scripts complex noise/decay math (wiggle/bounce) and tricky bezier handle inversion (pingpong loops) that LLMs struggle to compute accurately in text generation, while refusing simple element timing/staggers (`valueAtTime`, Sequence Layers) which agents handle trivially via direct JSON edits or Python one-liners per ADR-0148.
- **Q2 (Keyframe assistants and script packaging):** Option B separates media-decoding audio scripts (requiring `ffmpeg`) from pure JSON keyframe math operations, keeping tool dependencies clear, CLI interfaces clean, and execution deterministic for authoring LLMs.
- **Q3 (Measure-then-write colour helper):** Option A relies on visual feedback via `montagent frame` rather than blind histogram stretching; LLMs can inspect rendered frames, whereas automated histogram scripts frequently destroy intentional high/low-key lighting and specular highlights.
- **Q4 (Templates, MOGRTs, and sidecar convention):** Option A maintains strictly literal and closed project schema invariants (ADR-0017, ADR-0145) while formalizing the `<script>.py <project> <spec>` workflow in `skills/README.md` and `CONTEXT.md` as the complete solution for parameterized components.

**Trade-offs:**
- **Q1:** Agents must handle simple element delays and sequence timing in JSON, but avoids CLI wrapper sprawl for basic timing arithmetic.
- **Q2:** Requires maintaining two distinct scripts across `montagent-footage` and `montagent-motion`, but prevents mixing `ffmpeg` runtime requirements into pure JSON transforms.
- **Q3:** Agents lack a one-shot command for auto-contrast adjustment, but avoids introducing un-artistic histogram clip distortions into project files.
- **Q4:** Parameterized components require external script execution and sidecar spec files rather than inline macro properties, prioritizing project purity over inline template syntax.
```

### Tally and ruling

- **Q1 (`Option A`, 3–0):** Unanimous.
- **Q2 (Admitting all three assistants, 3–0; packaging in one `keyframes.py` vs splitting audio, 1–2):**
  All three jurors voted to admit *Convert Audio to Keyframes*, *Smoother* / reducer, and *Exponential
  Scale*. On packaging, Jurors 2 and 3 favoured splitting audio-to-keyframes into a separate script so
  pure-math passes do not share a file with an `ffmpeg` call, while Juror 1 and the Judge favoured one
  unified `skills/montagent-motion/scripts/keyframes.py <project> <spec>` that invokes `ffmpeg` lazily
  only when an `"audio"` pass is present, avoiding duplicated drawn-frame quantization, property path
  dispatch, and `simplify(samples, tol)` code and allowing a single `<spec>` sidecar to own all motion
  keyframes on an element without two scripts contending over the same element. The owner ruled with
  the Judge (**Option A**).
- **Q3 (`Option A`, 3–0):** Unanimous.
- **Q4 (`Option A`, 3–0):** Unanimous.
