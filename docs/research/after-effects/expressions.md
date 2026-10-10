# Census: After Effects expressions, scripting, data-driven animation and extensibility

Research for [#906](https://github.com/MBehtemam/Montagent/issues/906), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896). Researched 2026-10-10 against
`main` at `51aa4b5`.

**What this is.** Every feature in this area of Adobe's After Effects user guide, scored
against Montagent. It decides nothing. The map's standing rule applies to every row:
[ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)'s four invariants (literal values, closed vocabulary, checkable by
`validate`, editable by exact-string replace) cannot be waived, so an AE *mechanism* that
evaluates code or links one value to another is scored **refused** and the row moves on to
the **literal-value route**: the result an agent can reach by writing ordinary literals,
usually with a script outside the file, the way the duck does ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md)).

**AE version read.** After Effects 26.x. Adobe's fixed-issues page lists the September 2026
release (26.5) as the newest, and the C++ SDK guide's "What's New" names the 26.5 SDK.

**Sources.**

- **Primary, Adobe:** the After Effects user guide on `helpx.adobe.com` (expression basics,
  the expression language reference, the two expression engines, expression examples,
  expression controls, text expressions, data-driven animation, Essential Properties,
  Motion Graphics templates, keyframe assistants, scripts) and the Adobe developer blog's
  UXP timeline post.
- **Secondary, docsforadobe.dev** (community-maintained, began as Adobe's own guides; used
  only where marked **[dfa]**): the expression reference
  (`ae-expressions.docsforadobe.dev`) for the signatures of `sampleImage`,
  `sourceRectAtTime`, `posterizeTime` and `seedRandom`; the scripting guide
  (`ae-scripting.docsforadobe.dev`) for the scripting DOM; and the C++ SDK guide
  (`ae-plugins.docsforadobe.dev`), which is where Adobe's SDK guide is hosted.
- **How they were read.** This environment's egress proxy refuses direct fetches of
  `helpx.adobe.com`, `developer.adobe.com` and `docsforadobe.dev` (HTTP 403 on CONNECT). The
  pages were read through the search engine's retrieval of them, restricted to those domains,
  and every claim below is one that retrieval quoted from the named page. Two things could
  not be confirmed that way and are said so in their rows: the Wiggler's parameters beyond
  frequency, and the Smoother's steps.
- **Montagent:** `CONTEXT.md`, `docs/adr/` (cited by number), `crates/montagent-core/docs/format.md`
  and its pages, `schema/montagent.schema.json`, the router skill `skills/montagent/SKILL.md`
  (its capability map, including "Not by design"), and the job skills and their `scripts/`.

**Statuses.** **covered**: the format or a tool does it. **partial**: part of it is
covered, and the row says which part. **reachable**: a skill script or a written recipe
reaches the result with literals (the row says which, or that none ships yet).
**open map**: an open map owns the decision. **refused (ADR-NNNN)**: an ADR refuses the
mechanism. **gap**: nothing covers it and nothing has refused it. A seventh mark,
**out of scope**, is used only for app features the map excludes in its "Out of scope"
section (panels, app automation): Montagent has no GUI and an agent stands in for the app.

**Overlaps with sibling tickets.** The Wiggler, the Smoother, Motion Sketch and Convert Audio
to Keyframes are keyframe assistants, and their own rows belong to
[#899](https://github.com/MBehtemam/Montagent/issues/899); they appear here only as AE's own
precedent for baking. Text animators belong to [#900](https://github.com/MBehtemam/Montagent/issues/900);
only the expression selector is scored here. `aerender` and render queues belong to
[#907](https://github.com/MBehtemam/Montagent/issues/907). Whether AE counts as precedent at
all is [#897](https://github.com/MBehtemam/Montagent/issues/897).

## The census

### 1. The expression mechanism and its engines

| Feature | What it does | Doc | Montagent status | Evidence | Literal-value route |
|---|---|---|---|---|---|
| Expressions | A JavaScript snippet on a property, evaluated every frame, that computes its value | [Expression basics](https://helpx.adobe.com/after-effects/using/expression-basics.html) | **refused (ADR-0145)** | ADR-0145: "Expressions: still refused, now because they fail the literal-values invariant". [ADR-0003](../../adr/0003-general-video-editor-not-channel-tooling.md): "A keyframe list is still data you can read. An expression is not." Capability map, Not by design: "Keyframe expressions, or links between values. Write literal keyframes." CONTEXT **Recorded intent** puts *formula, expression* on the avoid list | Literal keyframes. AE's own **Convert Expression to Keyframes** is the precedent for the route (row 1.6). No general bake skill ships; per-job scripts do (`bake_rig.py`, `duck.py`, `captions.py`) |
| JavaScript expression engine | The default engine, based on ECMAScript 2018; up to 5x faster than Legacy | [Syntax differences between expression engines](https://helpx.adobe.com/after-effects/using/legacy-and-extend-script-engine.html) | **refused (ADR-0145)** | As above; [ADR-0017](../../adr/0017-closed-schema-no-escape-hatch.md): no open script in the project, at any level | None needed: an agent writes the values the engine would compute |
| Legacy ExtendScript expression engine | The older engine (ECMAScript 3), the default for projects saved in earlier versions; snake_case names, `.jsxbin` | [Syntax differences between expression engines](https://helpx.adobe.com/after-effects/using/legacy-and-extend-script-engine.html) | **refused (ADR-0145)** | As above | As above |
| Property link (pick whip) | Writes an expression that makes one property follow another, on the same or another layer | [Expression basics](https://helpx.adobe.com/after-effects/using/expression-basics.html) | **refused (ADR-0086)** | [ADR-0086](../../adr/0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md): **zero live references**; CONTEXT **Live versus recorded reference**; [ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md) retired the last element-to-element reference | Copy the literal (or the whole keyframe list) to the second element. Within one keyframe list, `t_from` `after-previous` records an offset that `validate` re-checks (`R-DERIVED-T`); between elements nothing records it (ADR-0086 grants the reference permission to no axis) |
| `.jsx` expression function libraries and `eval()` | A `.jsx` footage item holding functions that expressions call; `eval()` of a string | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0017)** | ADR-0017 (no escape hatch); ADR-0145 closed vocabulary: "A path may name media to decode, never a program to run" | The library's logic lives in a skill script outside the file, which writes literals |
| Convert Expression to Keyframes | Evaluates an expression, writes a keyframe on every frame, and turns the expression off (kept, disabled) | [Setting, selecting and deleting keyframes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/setting-selecting-deleting-keyframes.html); [Expression controls](https://helpx.adobe.com/after-effects/using/expression-controls.html) | **reachable** | The pattern is [ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md): a skill script computes and writes ordinary keyframes, keeps no provenance record in the file, has `--check` for staleness, and ends with `validate`. Router skill loop, step 3: "generate it with a script when the structure repeats (… a rig baked frame by frame …)" | This *is* the route. No general bake helper ships; each job skill bakes its own job. See "Results", group A |

### 2. The common expression methods

| Feature | What it does | Doc | Montagent status | Evidence | Literal-value route |
|---|---|---|---|---|---|
| `time` | The current composition time in seconds, used to drive a value (`time*360`) | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **covered** | A value linear in time is two keyframes with `"ease": "linear"` ([ADR-0146](../../adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md) §2 interpolation). `format.md`, Time: "A keyframe outside its element's range is legal and ordinary", so a ramp can run past `end` at a fixed rate | Two keyframes, linear, the second placed past `end` if the rate must hold to the cut |
| `index` | The layer's index, used to offset copies (`delay = index*k`) | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)**; partial inside one text element | ADR-0145's own failing example is `"delay": "index * 80"`. [ADR-0148](../../adr/0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md): no repeat, no stagger across elements; copies are written out. [ADR-0151](../../adr/0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md): a `units` block staggers the letters, words or lines of one text element | Write each copy as its own element with its own literal `t` values (ADR-0148 §3). For letters of one text, `units` with `every` and `order` |
| `wiggle(freq, amp, octaves, amp_mult, t)` | Smooth random deviation of a property around its value | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)** | ADR-0145 literal values. Capability map, Not by design (expressions). `montagent-motion/references/trailer.md`: shake and jitter are "sampled per frame … so write them with a generator script from the beat sheet" | Per-frame (or per-N-frame) `x`/`y`/`rotation`/`scale` keyframes from a seeded generator, `linear` between them. **No script ships**: `trailer.md` tells the agent to write its own; `bake_rig.py`'s `swings` adds a sine (not noise) to a rig joint only. AE's own Wiggler panel bakes wiggle to keyframes ([Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html); its rows are #899's) |
| `seedRandom(offset, timeless)`, `random()`, `gaussRandom()` | Seeded random numbers; by default the seed hashes layer, property and time; `timeless` holds one value | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html); [dfa] [Random numbers](https://ae-expressions.docsforadobe.dev/general/random-numbers/) | **refused (ADR-0145)**; partial for grain | [ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §3: a member random by nature takes a required literal `seed` and re-rolls every output frame by a fixed hash of seed and local instant; `grain` is the one such member. `R-GRAIN-SEED-SHARED` | Any other random value is drawn by a script and written as literals; the seed belongs in the script's spec or recipe (as `montagent-prerender` records its seed), never in the file |
| `loopOut("cycle")` / `loopIn("cycle")` | Repeats the last (or first) keyframe segments to the layer's end (or from its start) | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)** | ADR-0145 literal values; ADR-0148 (no construct that draws a thing again from fixed arithmetic without a per-copy string). Project `loop` ([ADR-0062](../../adr/0062-loop-declares-a-boolean-wrap-r-source-cut-pop-extends-mechanically.md)) is a different thing: it says the *video* wraps, affects only one `validate` check, and repeats no keyframes | Write the cycle's keyframes out to the element's `end`. **No script ships.** `montagent-motion/references/logo-loop.md` teaches planning one cycle by hand |
| `loopOut("pingpong")` | Repeats the segment alternately forward and backward | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)** | As above | As above, with each odd cycle's values reversed. Beziers must be mirrored, which by hand is error-prone. No script |
| `loopOut("offset")` | Repeats the segment, each cycle offset by the segment's start-to-end change times the loop count | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)** | As above | As above, with each cycle's values shifted by n times the delta. No script |
| `loopOut("continue")` | Keeps animating at the velocity of the last keyframe (no repeat) | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **reachable** | `format.md`, Time: a keyframe past the element's `end` is legal; `linear` is a named ease | One extra keyframe at or past `end`, extrapolated linearly from the last segment's rate. Hand arithmetic; no script |
| `loopOutDuration` / `loopInDuration` | The same loops, measured by a duration in seconds rather than a keyframe count | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)** | As `loopOut` | As `loopOut` |
| `valueAtTime(t)` | Reads a property's value at another time, for delays, echoes and trails, often from another layer | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0086)** | ADR-0086 zero live references; ADR-0145 literal values. `query --at` reads a resolved value for the agent, never for the renderer (`format.md`, Time) | Copy the keyframe list with every `t` shifted by the delay. Inter-element offsets have no recorded-intent field: ADR-0086 measured the 8-of-8 repeat-reading gaps and left them unsolved. No script |
| `linear()`, `ease()`, `easeIn()`, `easeOut()` | Remap an input range to an output range, linearly or with eased ends | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **partial** | Over time: covered by keyframe interpolation, six named eases or a cubic bezier (ADR-0146 §8; [ADR-0038](../../adr/0038-ease-is-required-on-every-non-first-keyframe-record.md)). As a mapping from another property's value: refused by ADR-0145 and ADR-0086 | Over time, two keyframes with the matching ease. A map from another value: a script computes the outputs and writes them |
| `posterizeTime(fps)` | Lets a property update fewer times per second than the composition | [dfa] [Global](https://ae-expressions.docsforadobe.dev/general/global/) | **partial** | `"ease": "step"` holds a value to the next keyframe (`trailer.md` glitch recipe; `type_on.py` cursor). No per-element update rate exists. `grain` always re-rolls every output frame (ADR-0156 §3) | Sample the motion at N per second and write `step` keyframes, each on a drawn instant (`measure --at`). For grain at a lower rate there is no route short of pre-rendering it (`montagent-prerender`) |
| `sampleImage(point, radius, postEffect, t)` | Reads the average colour and alpha of a layer's pixels around a point | [dfa] [Layer General](https://ae-expressions.docsforadobe.dev/layer/general/) | **refused (ADR-0145)** | ADR-0145 checkable test fails "A parameter whose legal range depends on rendered pixels"; ADR-0146 criterion 2: a property resolves "from the file and the instant alone" | The agent renders `frame --at`, reads the pixel with its own code, and writes the colour as a literal. No skill. Low demand; no ticket proposed |
| `sourceRectAtTime(t, includeExtents)` | Returns a layer's bounding box (top, left, width, height), used to fit a plate behind text | [dfa] [Layer General](https://ae-expressions.docsforadobe.dev/layer/general/) | **partial** | `measure` sizes text boxes (router skill loop, step 3; [ADR-0024](../../adr/0024-measure-writes-the-fit-repair-not-the-verdict.md): `measure` writes the fit repair); `query --at` prints `ink_box` (`format/text.md`). A box that follows the text live is refused (ADR-0086; `fit` is a derivation claim, [ADR-0015](../../adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md)) | `measure` the text, then write the plate's `width`/`height` as literals. Re-measure after a text edit; nothing flags a stale plate |
| Keyframe and marker access (`key()`, `nearestKey()`, `numKeys`, `marker`) | Reads a property's keyframes or a layer's markers from an expression | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)** | ADR-0145; the format has no markers (`format.md` has none) | A script reads the keyframes from the JSON and writes what it derives |
| `velocity`, `speed`, `smooth()` | Read a property's rate of change, or smooth it over a window | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **refused (ADR-0145)** | ADR-0145 | A script differentiates or smooths the keyframes and writes the result. The Smoother keyframe assistant is #899's row |
| Layer space transforms (`toComp`, `fromComp`, `toWorld`) | Convert a point between a layer's space and the composition's | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **open map** ([#591](https://github.com/MBehtemam/Montagent/issues/591)) | No composed space exists today; the router skill's "can't do yet": parenting, groups and cameras (#499). #591's provisional `nest` reports composed values "through one shared resolver" | Today `bake_rig.py` composes parent transforms frame by frame and writes literal `x`, `y`, `rotation` |
| Time conversion (`timeToFrames`, `framesToTime`, `timeToTimecode`) | Convert between seconds, frames and timecode strings | [Expression language reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) | **covered** | Times are absolute integer ms ([ADR-0005](../../adr/0005-absolute-integer-milliseconds.md)); which frame an instant draws is one rule ([ADR-0096](../../adr/0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md)), and `measure --at` confirms a drawn instant (router skill loop, step 3) | Not needed |
| Text: `sourceText` driven by an expression | Sets a text layer's string from code or another layer | [Text expressions](https://helpx.adobe.com/after-effects/using/expressions-text-properties.html) | **refused (ADR-0145)** | ADR-0145; text content is literal runs ([ADR-0007](../../adr/0007-text-runs-literal-size-declared-fonts.md)) | Literal runs. `captions.py` writes text elements from a words file, the data-to-literals route |
| Text style expressions (`style.setFontSize`, `setFauxBold`, `setText`) | Read and set font, size, colour and other style on character ranges from code | [Text expressions](https://helpx.adobe.com/after-effects/using/expressions-text-properties.html) | **refused (ADR-0145)** | ADR-0145; ADR-0007: runs carry their own literal size and colour | Split the text into runs, each with its own literal style |
| Expression selector (`textIndex`, `textTotal`, `selectorValue`) | Computes a text animator's amount per character, once per character | [Text expressions](https://helpx.adobe.com/after-effects/using/expressions-text-properties.html) | **partial** | ADR-0151: a `units` block runs one set of keyframes per letter, word or line, each late by `every` ms in `forward` or `reverse` order; a run carrying `unit` replaces the derived values. An arbitrary per-unit function is refused (ADR-0148 "replace, not add"; ADR-0145) | Stagger with `units`; single a unit out with a run carrying `unit`. Selector shapes beyond a linear stagger are #900's |

### 3. Expression controls

| Feature | What it does | Doc | Montagent status | Evidence | Literal-value route |
|---|---|---|---|---|---|
| Slider, Angle, Point, 3D Point, Color and Checkbox Controls | Effects that hold one value for expressions to read, often on a null driving many layers | [Expression controls](https://helpx.adobe.com/after-effects/using/expression-controls.html) | **refused (ADR-0086)** | ADR-0086 zero live references; ADR-0017 no unknown keys and no escape hatch; ADR-0145 literal values | **reachable**: the controls live in a skill script's spec and the script writes literals. Every shipped job script works this way: `bake_rig.py`'s spec (poses, swings, hops), `duck.py`'s `--under-db` and `--ramp-ms`, `captions.py`'s spec, `type_on.py`. #591's baseline found agents edit the spec and re-bake rather than edit the project (6 of 6 runs) |
| Layer Control | An effect that names a layer for expressions to read | [Expression controls](https://helpx.adobe.com/after-effects/using/expression-controls.html) | **refused (ADR-0086)** | ADR-0086; ADR-0012 | A script argument naming an element id (as `duck.py --bed --voice` does; ADR-0177 §1 says a script, unlike an MCP tool, may take ids) |
| Dropdown Menu Control | An effect offering named choices whose index an expression switches on | [Drop-down lists in Motion Graphics templates](https://helpx.adobe.com/after-effects/using/create_dropdowns_using_expressions.html) | **refused (ADR-0086)** | As Slider Control | An enum in the script's spec |

### 4. Essential Properties and Motion Graphics templates

| Feature | What it does | Doc | Montagent status | Evidence | Literal-value route |
|---|---|---|---|---|---|
| Essential Properties (formerly Master Properties) | Exposes a nested composition's properties on the instance in its parent, so it can be changed without opening it | [Work with Essential Properties](https://helpx.adobe.com/after-effects/using/essential-properties.html) | **open map** ([#591](https://github.com/MBehtemam/Montagent/issues/591)) for the nest; the exposed-value link is **refused (ADR-0086)** | ADR-0003 and [ADR-0001](../../adr/0001-flat-element-list.md) reject nesting with a clock; [ADR-0166](../../adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md) refuses the clock, not the shared space; #591's provisional `nest` is a pure matrix with no overrides | One written-out copy per instance with its own literals; or a script that emits each instance from a spec |
| Per-instance override of values and keyframes | Each instance of one composition keeps its own values for the exposed properties | [Work with Essential Properties](https://helpx.adobe.com/after-effects/using/essential-properties.html) | **refused (ADR-0148)** | ADR-0148: no element draws as more than one copy; each copy is written out and edited by its own string | Written-out copies; a generator script owns the shared part |
| Push and pull to source | Pushes an instance's override back to the source composition, updating every linked instance | [Work with Essential Properties](https://helpx.adobe.com/after-effects/using/essential-properties.html) | **refused (ADR-0148)** | Nothing is shared between copies (ADR-0148; ADR-0086) | Edit the spec and re-run the script (the #591 baseline's observed workflow) |
| Essential Graphics panel and `.mogrt` export | Packages a composition with only its exposed controls as a template for Premiere | [Create Motion Graphics templates](https://helpx.adobe.com/after-effects/using/creating-motion-graphics-templates.html) | **reachable** | Montagent's parameterised artifact is a job skill script plus its spec (`type_on.py`, `captions.py`, `bake_rig.py`, `duck.py`): the spec is the exposed controls, the project is generated literals. CONTEXT **Pre-rendered footage** puts *mogrt* on the avoid list. ADR-0017 left a sidecar's format open | A script and a spec per template. No convention for a user-authored template exists. See "Results", group C |
| Replaceable media in a template | Marks a layer so the template's user swaps in their own image or video | [Replaceable media](https://helpx.adobe.com/after-effects/desktop/motion-graphics/work-with-motion-graphics-templates/replace-media-motion-graphics-templates.html) | **covered** | A `source` is written inline on the element ([ADR-0002](../../adr/0002-inline-source-no-asset-table.md), [ADR-0053](../../adr/0053-asset-path-resolution-no-assetroot.md)) and changed by exact-string replace (`format.md`, How you edit a project); `validate` re-probes and checks `fit` | Replace the `source` string |
| Data-driven templates in Premiere | Premiere fills a template's fields from a spreadsheet, one graphic per row | [Data-driven Motion Graphics templates](https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/use-data-driven-motion-graphics-templates.html) | **reachable** | As data-driven animation (section 5) | A script reads the rows and writes one element, or one project, per row. None ships |

### 5. Data-driven animation

| Feature | What it does | Doc | Montagent status | Evidence | Literal-value route |
|---|---|---|---|---|---|
| JSON imported as footage, read by `sourceData` | A JSON file in the project; expressions read its values | [Data-driven animation](https://helpx.adobe.com/after-effects/using/data-driven-animations.html) | **refused (ADR-0017)** | ADR-0017; ADR-0145 closed vocabulary (a path names media to decode); ADR-0086 (a value the renderer resolves from elsewhere is a live reference) | **reachable**: a script reads the JSON and writes literals. Shipped instances: `captions.py` (a words file to text and highlights), `bake_rig.py` (a viseme file to mouth elements), `duck.py` (a words file to `volume` keyframes) |
| CSV and TSV read by `dataValue([row, col])` | Tabular data imported as footage, read by zero-based row and column | [Data-driven animation](https://helpx.adobe.com/after-effects/using/data-driven-animations.html) | **refused (ADR-0017)** | As JSON | A script reads the table and writes literals. No CSV script ships. See "Results", group B |
| mgJSON time-varying data streams | The one data format whose values change over time, read by `dataValue` as streams | [Data-driven animation](https://helpx.adobe.com/after-effects/using/data-driven-animations.html) | **refused (ADR-0017)** | As JSON | Time series written as keyframes by a script, as `duck.py` and `bake_rig.py` do with their own time series |
| Create Keyframes from Data | Converts mgJSON streams to keyframes; the result is no longer live | [Data-driven animation](https://helpx.adobe.com/after-effects/using/data-driven-animations.html); [Setting, selecting and deleting keyframes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/setting-selecting-deleting-keyframes.html) | **reachable** | ADR-0177's pattern; ADR-0055's door for "a write tool that computes and writes the keyframes" was answered as a skill script | Per job, as above. No general "series to keyframes" script |
| Versions from a CSV | Links text layers to CSV columns and replaces the file to regenerate every graphic | [Create versions with CSV files](https://helpx.adobe.com/after-effects/how-to/multiple-graphics-text-csv.html) | **reachable** | Router skill loop: the CLI and MCP verbs check and render any number of projects | A script writes one project per row, then renders each. None ships |
| Animated charts and graphs | Bars, lines and pies driven by data through expressions | [Create animated charts and graphs](https://helpx.adobe.com/after-effects/how-to/create-data-driven-animations.html) | **reachable** | Keyable `width`/`height` on `rect`, `points` and `trim_end` on `path` (capability map, Motion and Elements); `montagent-motion/references/launch-spot.md` mentions "one small chart … growing in on beats" | Hand-written or script-written literal bars and paths. No chart script |
| Convert Audio to Keyframes | Writes the audio's amplitude as slider keyframes on a new layer, for expressions to read | [Assorted animation tools](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html) | **gap** (the amplitude curve); beat times are **reachable** | `montagent-craft/scripts/beat_grid.py` prints a beat grid on drawn frames (workaround for #547); nothing writes an amplitude curve. The tool's own row is #899's | A script could decode the audio and write, say, `scale` keyframes from its envelope. None ships |

### 6. Scripting and extensibility

| Feature | What it does | Doc | Montagent status | Evidence | Literal-value route |
|---|---|---|---|---|---|
| ExtendScript scripts (`.jsx`, `.jsxbin`, File > Scripts) | Automates the app: builds compositions, layers and keyframes | [Scripts](https://helpx.adobe.com/after-effects/using/scripts.html) | **covered** | [ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md): an agent edits the project with ordinary file tools, and the tools check, measure and render; router skill loop, step 3 ("generate it with a script when the structure repeats") | Any script in any language that writes the JSON, then `fmt` and `validate` |
| The scripting DOM's keyframe API (`Property.setValueAtTime`, `app.project.items.addComp`) | Sets keyframes and creates items from a script | [dfa] [Property object](https://ae-scripting.docsforadobe.dev/property/property/) | **covered** | A keyframe is a `{t, t_from, v, ease}` record written in the file (ADR-0146 §1); ADR-0011 | Write the record |
| Running a script from the command line (`afterfx -r`) | Runs a script in an open instance | [Scripts](https://helpx.adobe.com/after-effects/using/scripts.html) | **covered** | Every verb is on the CLI, and the MCP server shares them (router skill, Environment) | Not needed |
| "Allow scripts to write files and access network" | A preference that lets scripts touch the file system and network | [Scripts](https://helpx.adobe.com/after-effects/using/scripts.html) | **out of scope** | An app security setting; the agent's own environment governs this | — |
| ScriptUI panels | Scripts that dock as panels in the Window menu | [Scripts](https://helpx.adobe.com/after-effects/using/scripts.html) | **out of scope** | #896 Out of scope: panels; Montagent has no GUI | — |
| CEP extensions | HTML/JS panels running Chromium beside ExtendScript; new submissions stop in December 2028, retired at the end of 2029 | [Adobe developer blog: UXP comes to the flagship apps](https://blog.developer.adobe.com/en/publish/2026/09/investing-in-the-future-of-creative-cloud-extensibility-uxp-comes-to-our-flagship-applications) | **out of scope** | As ScriptUI panels | — |
| UXP plugins and scripts | CEP's successor: panels and commands calling host APIs from one JavaScript context; AE public beta by November 2026; ExtendScript unaffected | [Adobe developer blog](https://blog.developer.adobe.com/en/publish/2026/09/investing-in-the-future-of-creative-cloud-extensibility-uxp-comes-to-our-flagship-applications) | **out of scope** | As ScriptUI panels | — |
| C++ plug-in SDK: effect plug-ins | Third-party effects compiled against the SDK, described by a PiPL, listed in the Effects menu | [dfa] [After Effects C++ SDK Guide](https://ae-plugins.docsforadobe.dev/) | **refused (ADR-0003)** | ADR-0003: "Effects are a closed, named vocabulary — not an effect stack or a plugin system"; ADR-0017; ADR-0156: the list grows one member per ADR, and a code-drawn look enters as pre-rendered footage | **reachable**: `montagent-prerender` draws the look in any code, encodes lossless footage with alpha, places it as a `video`, and keeps a recipe that rebuilds and hash-checks it |
| C++ plug-in SDK: AEGPs | Plug-ins that read and modify the project and preferences, and add menus and panels | [dfa] [AEGP overview](https://ae-plugins.docsforadobe.dev/aegps/overview/) | **out of scope** | App automation (#896 Out of scope); its project-editing half is the scripting row above | — |
| C++ plug-in SDK: Artisans | Plug-ins that take over 3D layer rendering | [dfa] [Artisan data types](https://ae-plugins.docsforadobe.dev/artisans/artisan-data-types/) | **refused (ADR-0003)** | ADR-0003 (no plugin system); 3D itself is [#902](https://github.com/MBehtemam/Montagent/issues/902)'s | — |
| AE expressions inside an exported Lottie | A Lottie exported from AE that still carries expressions (`x` beside `k`) | — (Lottie, not Adobe) | **refused (ADR-0171)** | [ADR-0171](../../adr/0171-svg-and-lottie-do-not-enter-as-sources-and-each-is-pre-rendered-through-the-skill.md): a Lottie whose expressions the player skips is refused; `prerender.py` names each expression and fails the build | Convert the expressions to keyframes in AE before export, then pre-render the Lottie as footage |

## Counts

| Status | Rows |
|---|---|
| covered | 6 |
| partial | 4 |
| reachable | 7 |
| open map (#591) | 2 |
| refused | 29 |
| gap | 1 |
| out of scope | 5 |
| **total** | **54** |

A row with two statuses (for example "refused; partial inside one text element") is counted
once, under its first. Every refused row except Artisans names a literal-value route. For the
data, control, plug-in and Lottie rows a shipped script already takes it (`captions.py`,
`duck.py`, `bake_rig.py`, `prerender.py`). For the motion rows (`wiggle`, the loops,
`valueAtTime`, `random`) no script does, and the agent writes its own generator each time.

## Results worth a decision ticket

### A. Bake scripts: the expression results agents ask for most, written as keyframes

The route ADR-0177 opened (a skill script writes ordinary keyframes, keeps no record in the
file, has `--check`, ends with `validate`) reaches every motion expression in section 2. AE
itself ships the same route as **Convert Expression to Keyframes** and the **Wiggler**, so
the mechanism has an AE-side precedent; whether that counts is #897's call. Today agents
must write their own generator each time (`trailer.md` says so for shake and jitter).

1. **A wiggle/noise bake.** Seeded, smooth noise on `x`, `y`, `rotation`, `scale` or
   `opacity`, with frequency, amplitude, octaves and a seed in the spec, written on drawn
   instants with `linear` ease, optionally held N times per second with `step` (the
   `posterizeTime` case). It would replace `trailer.md`'s "write your own generator", and
   could take over `bake_rig.py`'s `swings`.
2. **A loop bake.** `cycle`, `pingpong`, `offset` and `continue` over the last (or first)
   N keyframe segments, written out to the element's `end`. Pingpong must mirror bezier
   eases, which is the error-prone part by hand. It needs a rule for the instant where a
   cycle ends exactly on `end`, and should say how it relates to project `loop` (ADR-0062).
3. **An offset copy** (`valueAtTime` trails and echoes): copy one element's keyframe lists
   to N others, each shifted by k ms. It overlaps the cross-element stagger ADR-0148 refused
   as a construct; as a script it adds no construct.

Questions each ticket must answer: one script with subcommands or several; which job skill
owns it (`montagent-motion`, most likely); keyframe density against file size and the edit
test (#591's baseline: 709 baked keyframes, and agents edit the spec, not the project);
`R-EASE-INERT` on held values (ADR-0177 §5 already documents it as expected on a hold);
and `--check` for staleness.

### B. Data to literals

4. **A table-to-elements script** for charts and per-row versions (CSV, TSV or JSON in;
   literal elements or one project per row out), generalising what `captions.py` already
   does for words. AE's CSV versioning and Premiere's data-driven templates are the
   precedent rows.
5. **An audio-envelope-to-keyframes script** (Convert Audio to Keyframes' result). This
   probably belongs to the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795))
   or #899, beside `beat_grid.py`.

### C. Templates: is "a script plus its spec" Montagent's MOGRT?

6. Every shipped job script already behaves like a template: the spec holds the exposed
   controls (the expression-control rows), and the project is generated output. #591's
   baseline shows agents already treat the spec as the source of truth. A decision ticket
   would settle whether that is the template story, and if so the convention for a spec
   file beside a project (ADR-0017 left the sidecar's format open), including what happens
   when someone edits the generated project by hand. Essential Properties' nesting half
   waits on #591's `nest` either way.

### D. Recorded intent across elements

7. The `valueAtTime`, `index` and Property-link rows all end in the same place: an offset
   between two elements' values that nothing records. ADR-0086 grants the recorded-reference
   permission to no axis and asks for a measurement that inference fails; its own 8-of-8
   repeat-reading gaps are the strongest candidate. If the bake scripts in group A ship, the
   offsets they write become measurable, which could supply that census. Low priority until
   then.

### Smaller, and not proposed as tickets

- **Grain at a lower update rate** (`posterizeTime` on noise is common in AE): `grain`
  re-rolls every output frame (ADR-0156 §3). A future rate parameter would go through
  ADR-0156's growth path. No demand recorded.
- **`sampleImage`:** refused on the checkable invariant, and the manual route (`frame --at`,
  read the pixel, write a literal) is enough.
- **Capability map wording:** once a bake script exists, the "Not by design" line on
  expressions ("Write literal keyframes") can name it, as the spring line already names
  "keyframes baked out".
