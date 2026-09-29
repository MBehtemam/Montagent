# Capability inventory: what Montagent can do, and what an agent will not find

Research for [#445](https://github.com/MBehtemam/Montagent/issues/445), on the map
[#441](https://github.com/MBehtemam/Montagent/issues/441) (end-user agent skills).

**Question.** What can Montagent do, and which of those capabilities is an end-user agent
**unlikely to discover or use well** from `montagent://schema.json`, `montagent://format.md`
and the MCP tool descriptions alone?

**Sources.** All local, at `b1fa1e28` (`main`):

- The published schema: `schema/montagent.schema.json` (what `montagent://schema.json` serves,
  generated from the same types, `crates/montagent-core/src/resources.rs`).
- `crates/montagent-core/docs/format.md` (served as `montagent://format.md`).
- The MCP server instructions and the nine tool descriptions and parameter docs:
  `crates/montagent/src/mcp.rs`. CLI help: `crates/montagent/src/cli.rs` and `montagent --help`.
- `CONTEXT.md`, the ADRs cited inline (banners checked for amendments), the finding
  registry `crates/montagent-core/src/registry.rs`, `examples/hello-text/`,
  `docs/research/first-real-use/FIELD-REPORT.md`, and the open issue list.
- **A live session** with the `main` debug build: `create-project` → `fonts vendor` → a
  lower-third plus two kinetic words, then `fmt`, `validate`, `frame`, `query` (all three
  modes), `measure --all`, `measure --at`, and an invented effect. Observations from it are
  marked **(observed)**.

## How discoverability is graded

An agent connected over MCP sees exactly three things before it starts: the one-paragraph
server instructions (`mcp.rs` `get_info`), the nine tool descriptions with their parameter
docs, and the two resources *if it chooses to read them*. Grades are relative to that
surface only:

| Grade | Meaning |
|---|---|
| **obvious** | Stated in `format.md` or a tool description, in words an agent reads on the way to a first render. |
| **findable** | Present only as schema shape (a key, an enum, a `$defs` description), or taught by a finding's message the first time the agent gets it wrong. An agent that reads the whole schema finds it; one that skims does not. |
| **buried** | Only in `CONTEXT.md`, an ADR, CLI help, or code — none of which an MCP client is shown — or it needs two facts combined that no one surface puts together. |

The finding registry counts as a teaching surface (ADR-0006, ADR-0043): 87 `Live` codes, most
carrying the numbers they rest on, several carrying a verbatim repair. Much of the format is
learned by being told off, which is fine for rules and useless for capabilities: **nothing
reports a capability you never tried.**

## 1. Content capabilities

### Document structure

| Capability | Where it is | Grade | Notes |
|---|---|---|---|
| Header: `frame`, `fps`, `background`, `duration`, `output` | schema; `create_project` params | obvious | `create_project` writes only what it is told (ADR-0030, ADR-0080). |
| `loop: true` (asserts `duration` wraps to 0) | schema description only | findable | "Affects nothing but one `validate` check" (ADR-0062) — the check is `R-SOURCE-CUT-POP` at the wrap. No container metadata is written. |
| Tracks: stacking only, never timing; no overlap inside a track; gaps legal | format.md | obvious | ADR-0004, ADR-0006, ADR-0060. |
| `layer` as integer or anchor `{"above"/"below": id}`, one hop | format.md, schema | obvious | ADR-0019. Layer tie is an error only when boxes meet in time and space (ADR-0060). |
| `group` (render-inert label) | format.md | obvious | Its real value is as a `query --where` handle — **that pairing is buried** (only the `query` param example `group = item-05` hints at it). |
| Seven element types: `image`, `video`, `text`, `rect`, `ellipse`, `audio`, `transition` | schema `oneOf` | obvious | No path/line/polygon (ADR-0014), no SVG (ADR-0027), no gradient fill. |

### Transform and animation

| Capability | Where it is | Grade | Notes |
|---|---|---|---|
| Flat transform: `x`, `y`, `origin` (9 keywords), `scale` `[sx,sy]`, `rotation`, `opacity` | format.md | obvious | ADR-0012, ADR-0013. `rotation` is never normalised: `1080` is three turns (schema). |
| Keyframes: any transform property (and `volume`) is a scalar **or** a list of `{t, v, ease}` | schema `Animatable*` unions | findable | format.md explains `t` semantics but never says in one sentence "every transform property is keyframable". `x`/`y` values are **integers**; `scale` values are `[sx,sy]` arrays; `rotation`/`opacity` are numbers. |
| Clamping at both ends ("and then it holds" is free) | schema `Animatable` description | findable | ADR-0012. |
| Keyframes outside the element's range (a trimmed move) | format.md | obvious | ADR-0012. |
| Ascending `t` required | schema description | findable | ADR-0082; an error names it. |
| `ease` required on every record but the first, forbidden on the first | format.md | obvious | ADR-0038; the most common first-attempt error, and a finding repairs it. |
| Named eases (`linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`, `step`) and raw cubic bezier `[x1,y1,x2,y2]` | schema `Ease`/`EaseName` | findable | **Overshoot is legal** — `y` outside `[0,1]` — which is how a back-out / pop / bounce-ish settle is spelled (schema description). No agent will guess this unless it reads `$defs.Ease`. |
| `step` ease (hard switch at the keyframe) | schema | findable | Useful for blink/flash beats with no extra elements. |
| `t_from` recorded intent (`element-start`, `after-previous`) | format.md | obvious (to read), buried (why use it) | ADR-0086. Only `validate` reads it (`R-DERIVED-T`). Worth teaching for generated timelines whose keyframes are derived from element starts. |
| Width/height, colour, effect parameters, `clip`, mask rect: **not animatable** | format.md ("no effect parameter is keyframable"; `clip` "never animates") | findable (as absences) | ADR-0025, ADR-0040. Motion that looks like a size or colour change must be built from `scale`, `opacity`, stacked elements, or a `highlight` step. |

### Geometry, paint and media

| Capability | Where it is | Grade | Notes |
|---|---|---|---|
| `clip`: static frame-space aperture; Ken Burns = `scale` behind a still `clip` | format.md | obvious | ADR-0025. |
| `fit` (`cover`/`contain`/`literal`) as a claim checked against real source dimensions | format.md, schema | obvious | ADR-0015, ADR-0026. **How to learn the source's dimensions over MCP is buried**: `probe` is CLI-only; the MCP route is to write `fit: "cover"` with a guess and read `E-FIT-DEVIATION`, which states `source_width x source_height` and the exact rect to write. |
| `rect` (`radius`, single corner radius), `ellipse`; `fill` and/or `stroke` (outlined shapes); stroke falls inside | format.md, schema | obvious | ADR-0014. |
| Translucent paint via `#RRGGBBAA` (static), `opacity` for animation | format.md | obvious | ADR-0014. |
| Effects list, order is semantic: `blur`, `shadow` | schema `Effect` | findable | format.md mentions both in passing; their parameters (`shadow{dx,dy,radius,color,opacity}`) are schema-only. |
| Colour filters: `tint`, `saturation`, `brightness`, `contrast` | schema | findable | ADR-0049. **Recipes are buried** in `CONTEXT.md` only: grayscale = `saturation 0`, sepia = that + warm `tint`. |
| `mask` (`circle`/`rect`/`ellipse`, element-local rect, rides the transform) | format.md (long section) | obvious | ADR-0068, ADR-0084. `R-MASK-CIRCLE-NON-SQUARE` at review. A circular avatar crop is one line. |
| `chroma` key (`color`, `tolerance`, `softness`, `spill`), placed before colour scalars | format.md, `measure` description | obvious | ADR-0088; `measure` reports per-frame alpha coverage (drift finder). |
| Video: `source_start`/`source_end`, `speed` (exact `end-start = span/speed`), `overrun` `hold`/`loop` | format.md, schema | obvious | ADR-0020, ADR-0045. `E-SPEED-MISMATCH` states the numbers. No reverse. |
| Audio: `volume` scalar or keyframed (fades), `speed`, `overrun: "loop"` only | schema | findable | ADR-0055. **format.md says nothing about audio at all.** Ducking is hand-authored keyframes (CONTEXT, buried). **The mix sums at `normalize=0`** (ADR-0055 as amended by ADR-0077), so a music bed at `1.0` under narration at `1.0` is two full-level signals — buried, and it is a craft default every ad needs. |
| Video's embedded audio uses the same `volume`; no `mute` | schema description | findable | ADR-0055. |
| Remote `source` URLs | format.md says "a path or a URL" | obvious — **and wrong in practice** | `validate` probes URLs (ADR-0056), but `render` draws and mixes local sources only (`E-NOT-PAINTED-REMOTE`, `E-NOT-MIXED-REMOTE`, ADR-0093). See product-gap flag 3. |
| `transition` element, `kind: "crossfade"` only, `from`/`to` ids | schema | findable | ADR-0059. **Its load-bearing rule — `start`/`end` must exactly equal the intersection of the two bridged ranges, so the two elements must overlap on different tracks — is in neither format.md nor the schema description.** It is taught only by `E-TRANSITION-RANGE` / `E-TRANSITION-NO-OVERLAP`. Wipe/slide/push are deferred. |

### Text

| Capability | Where it is | Grade | Notes |
|---|---|---|---|
| `runs` array; per-run `font`, `size`, `color`, `stroke`, `stroke_width` | format.md, schema | obvious | ADR-0007. A run boundary is style only. |
| Line breaks are `\n` the agent places; the renderer never wraps | format.md, `measure` description | obvious | ADR-0008. `measure` returns break opportunities (byte offsets). |
| `width`/`height` is a container claim; `height` = `measure`'s `block_height` | format.md, `measure` description | obvious | ADR-0024, ADR-0028. `R-BOX-SLACK` reports the slack. |
| `line_height` one decimal, default 1.2 | format.md | obvious | ADR-0028. |
| `align` `start`/`center`/`end` (lines to each other) vs `origin` (box placement) | schema `Align` | findable | ADR-0007. |
| `highlight` on a run: a timed style window (`color`, `stroke`, `stroke_width`) | schema `Run`/`Highlight` | findable | ADR-0048. **Not in format.md.** Only colour/stroke change — no scale or position pop. The window must sit inside the element's range (`E-HIGHLIGHT-RANGE`); one window per run, so every highlighted word is its own run. The karaoke/active-word recipe is buried. |
| Per-run `dir` (RTL isolate) | schema | findable — **and not honoured** | The painter draws it as if absent and raises `E-FIELD-UNHONOURED` (`frame.rs`). See product-gap flag 4. |
| Fonts: `fonts` table of ordered file chains; `fontVendor` attestation written by `montagent fonts vendor` | format.md | obvious (that it exists), **buried (how)** | ADR-0007, ADR-0057. `fonts list` / `fonts vendor` are **CLI-only**. `fonts vendor` writes `fontVendor` but **not** the `fonts` chain — the agent adds `"fonts": {"body": [{"file": "fonts/X.otf"}]}` by hand, otherwise `measure` refuses with "declares no `body`" **(observed)**. |
| Bold / weight = a different font file | CONTEXT (`Weight / bold`) | buried | The schema rejects `weight` with a named replacement. Field report: vendoring left the author without a bold weight (#351). |
| `.ttc` collections via `index` | schema | findable | ADR-0102. |

## 2. Verbs and their modes

Nine MCP tools; twelve CLI commands plus `fonts` and `mcp` (CONTEXT `Verb`, ADR-0011 as amended
by ADR-0078 and ADR-0097).

| Verb | Modes / arguments | Surface | Grade | What an agent misses |
|---|---|---|---|---|
| `create_project` | header fields | MCP + CLI | obvious | Writes no `fonts`; text needs the CLI step next. |
| `validate` | `json`, `verbose` | MCP + CLI | obvious | That `review` is not a failure and `LAYOUT` is not a severity (format.md does say this). That every text element is treated as a caption (see flag 5). |
| `query --at` | resolved stack, anchors resolved, animated values interpolated | MCP + CLI | obvious | Reports declared, not effective, values: a crossfading element reads `opacity 1`, and an active `highlight` is not shown **(observed)**. |
| `query --from/--to` | cut list with outside boundaries named | MCP + CLI | obvious | That this is the "where are my beats" map for a whole piece. |
| `query --where` + `--census` | conjunction of whole-value terms, dotted paths, `runs.*.x`, reserved `track`; distribution | MCP + CLI | findable | The grammar is in the parameter doc only (ADR-0070). No `or`, no substring; values match as written, nothing resolved. The consistency-audit use ("4 of 5 siblings agree") is stated but no example pairs it with `group`. |
| `frame` | `at`; `crop x,y,w,h` (true scale); `full`; `png` | MCP (image inline) + CLI (`--out` required) | obvious | Token costs are in the description. **Range mode / contact sheet (`from`/`to`) is decided (ADR-0094–0098, 0103, 0105) and not shipped on either surface** — `FrameParams` and the CLI take only `at`; see flag 1. CLI writes JPEG bytes to a `.png` path unless `--png` is passed **(observed)**. |
| `measure` `element` | advance, ascent/descent, lines, baselines, `block_height`, stroked extent, break offsets | MCP + CLI | obvious | |
| `measure` `elements` / `all` | batch | MCP + CLI | obvious | |
| `measure` `at` | nearest sampled instant at-or-before a time on the frame grid | MCP + CLI | findable | The trick it serves — landing a fade on a frame that is actually drawn, and "last drawn frame" = `at: end - 1` because ranges are half-open — is only half stated. The field report's biggest catch was `R-KEYFRAME-UNREACHED` on exactly this. |
| `measure` chroma | per-frame keyed-alpha coverage | MCP + CLI | obvious | |
| `shift` | `at`, `delta` (positive only), `scope` (track), `release` pairs | MCP + CLI | findable | **Cannot remove time**: a negative `delta` is refused (`shift.rs` module note). Straddling audio/video is refused. Slack is invariant; `release` must name exact pairs from the refusal. |
| `compare` | `reference` vs `current` | MCP + CLI | findable | **Needs a prior copy of the file that nothing keeps for you** — the agent must snapshot before editing. That workflow is buried. |
| `preview` | proxy 720p (540p degrade, refusal floor), `from`/`to`, `full`, `output` | MCP + CLI | obvious | That it is for the human and for motion; an agent cannot watch an MP4, it can only read the result. |
| `render` | whole or `from`/`to` partial to `out/<name>.<from>-<to>.mp4`, `output` | MCP + CLI | obvious | Runs `validate` first and refuses on `error`; prints `review`s after. CLI-only `--no-clobber` (ADR-0104). |
| `fmt`, `fmt --check` | canonical convention | **CLI only** | findable (format.md names it) | An MCP-only client cannot run it (flag 2). |
| `probe` | media numbers | **CLI only** | buried | See `fit` row for the MCP workaround. |
| `timeline` | the human's wide view | **CLI only** | buried | |
| `fonts list`, `fonts vendor` | licence gate, attestation | **CLI only** | findable (format.md names it) | Required before any text renders (flag 2). |
| `cache` | probe sidecar maintenance | CLI only | buried | Rarely needed. |

## 3. Typical combinations

None of these is written down anywhere an agent is shown. Each is expressible today; the
lower-third and the kinetic words were built and rendered in the live session **(observed)**.

**Lower-third.** A `rect` bar (`radius`, `#RRGGBBAA` fill, `shadow`) and a `text` on a higher
layer (or `layer: {"above": "lt-bar"}`), both in one `group`. The bar "wipes on" by keyframing
`scale` from `[0, 1]` to `[1, 1]` with `origin: "top-left"` (or `center-left`) — width is not
animatable, so scale about the left edge *is* the wipe. The text slides with an `x` keyframe
pair and fades with `opacity`, `ease-out`. Out-animation is the mirror, landing on a sampled
frame (`measure at`). Checks: `measure` for the text's `height`, `query --at` mid-animation,
`frame` + `crop` on the bar.

**Kinetic typography.** One `text` element **per word or phrase**, each with its own `start`
(the stagger is the start offsets), `scale`/`rotation`/`opacity`/`x`/`y` keyframes, and an
overshoot bezier (`[0.3, 1.6, 0.6, 1.0]`) for the pop. Words that share screen time go on
different tracks; words that replace each other can share a track. No per-glyph animation
exists, so letter-by-letter means one element per letter. A colour flip on one word is either
a second element or a `highlight` window. A generator script plus `fmt` is the realistic
workflow at this density (field report; #352).

**Karaoke / active-word captions.** One `text` element per line, one run per word, each word's
run carrying a `highlight` window with literal times from external alignment (ADR-0048,
ADR-0051). `compare` reports `D-HIGHLIGHT-TEXT-DRIFT` after a text edit.

**Photo montage with Ken Burns.** `image` with `fit: "cover"`, a `clip` equal to the frame,
`scale` keyframes behind it, keyframes allowed to overrun the element's `end` (ADR-0012), and
`transition` crossfades between consecutive photos on alternating tracks.

**Picture-in-picture / avatar bubble.** `video` or `image` with `mask {shape: "circle"}` on a
square rect, `shadow` after the mask (order is semantic), `scale` keyframes to pop it in.

**Green-screen presenter.** `video` + `chroma` first in `effects`, then any colour scalars;
`measure` the element for coverage drift; cut the element where coverage steps.

**Music bed under voice.** `audio` bed with `overrun: "loop"` and a `volume` well under 1
(the mix does not normalise), keyframed down under each narration line (manual ducking) and
faded out at the end; narration elements at `1`. `R-CAPTION-NO-AUDIO` goes quiet once
something audible overlaps each text.

**Grade.** `saturation`, `contrast`, `brightness`, `tint` on each footage element (effects are
per-element — there is no adjustment layer), in a consistent order checked with
`query --where` + `--census effects.0.name`.

## 4. Workflow knowledge the tools assume but do not teach

1. **The edit loop.** Read both resources → `create_project` → edit with file tools →
   `validate` (or read the write verb's findings) → `frame`/`query --at` at the beats →
   `preview` for the human → `render`. The server instructions say "edit with your file tools"
   and name the resources; the order and the checkpoints are nowhere.
2. **The CLI is part of the product even for MCP clients.** Fonts must be vendored and `fmt`
   run from a shell. The server instructions never mention the CLI.
3. **Fonts setup is two steps**: `fonts vendor <project> <file> [--licence X]`, then write the
   `fonts` chain by hand. A different weight is a different file.
4. **Write text boxes from `measure`, never estimates**: `height = block_height`, width ≥ the
   stroked extent; place `\n` yourself from the break offsets.
5. **Half-open time is a trap for motion**: a keyframe at an element's `end` is never drawn.
   Land final keyframes on `measure at: end - 1` (issue #319 is the open enhancement).
6. **Snapshot before an edit you want to `compare`.** `compare` has no memory.
7. **`shift` only inserts time.** Removing time is a hand edit, then `validate`.
8. **Read finding classes as facts, not verdicts**: `review` never blocks; `R-CAPTION-*`
   fires on every text element, including titles in a music-only ad.
9. **Generate, then `fmt`**, for anything beyond a handful of elements; keep one element per
   line so later exact-string edits hit one element (format.md's convention, #352).
10. **Learn media numbers from findings** over MCP: `E-FIT-DEVIATION` gives image and video
    dimensions; `E-SOURCE-OVERRUN` gives a source's probed duration.
11. **Attribute before you fix**: read `frame`'s stack caption, then `crop` + `png` for
    detail. `frame` believes a layout; `measure` measures it.
12. **Use `group` + `query --where/--census` as the consistency audit** across repeated units
    (every lower-third, every word card).
13. **Keep sources local.** A URL validates but does not render (flag 3).

## Product-gap flags

Per the map's rule these are raised for separate issues and are **not** worked here. Where an
issue already exists it is named.

1. **`frame`'s range mode (contact sheet) is specified but not shipped.** ADR-0094–0098, 0103
   and 0105 are accepted, CONTEXT's `Verb` entry says "the nine and the twelve survive `frame`'s
   range mode", and the registry carries `E-SHEET-OVERFLOW` — but `FrameParams` (`mcp.rs`) and
   the CLI `Frame` command take only `at`. A skill must not teach it yet. **Existing:** map
   [#395](https://github.com/MBehtemam/Montagent/issues/395) (open; #418 names the remaining
   flags).
2. **An MCP-only client cannot render text.** `fonts vendor` (and `fmt`, `probe`) are CLI-only
   by ADR-0011's cost model, and no text renders without a vendored font. The server
   instructions do not say a shell is required. A skill can say it; only the product can make
   it true or state it. **Related:** #351, #352. **New issue needed** for the instructions /
   route question.
3. **`format.md` teaches URL sources that `render` will not draw.** "A `source` is … a path or
   a URL" and "moving a project to a remote store means rewriting each `source` to a URL" —
   while ADR-0093 makes `render` decline remote sources (`E-NOT-PAINTED-REMOTE`,
   `E-NOT-MIXED-REMOTE`). **New issue needed** (format.md).
4. **`runs[].dir` is published, validated, and not honoured.** The painter raises
   `E-FIELD-UNHONOURED` for it (`frame.rs`), so `render` refuses any project using it; the
   schema advertises the field with no caveat. **New issue needed.**
5. **Caption checks apply to every text element.** `R-CAPTION-NO-AUDIO`, `-PACE`,
   `-MIN-DURATION` fire on titles, lower-thirds and kinetic words **(observed: three findings on
   a three-text motion piece)**. ADR-0054 scopes them to any element with `runs`; #135 closed
   without a discriminator. For the ad eval this is persistent `review` noise a skill can only
   teach the agent to ignore. **Flag for the human; probably a new issue.**
6. **`format.md` is silent on transitions, highlights, audio/volume and `loop`**, although it
   claims to hold "every rule a schema has no way to say". The transition exact-window rule and
   the highlight-inside-range rule are cross-field rules taught only by errors; the
   `normalize=0` mix is a render reading (ADR-0077) no resource mentions. **New issue needed**
   (format.md coverage).
7. **`E-EFFECT-UNKNOWN` names the wrong key.** It reads `kind` from the declared effect, but
   effects are named by `name`, so the message prints `` `(no `kind`)` `` instead of the
   invented member **(observed** with `{"name":"glow"}`; `frame.rs` `effects_of`). Small bug.
   **New issue needed.**
8. **Motion-graphics expressiveness ceiling (deliberate v1 deferrals, not bugs).** No animatable
   `clip`, mask rect or effect parameter (no track-matte reveals, no animated blur), crossfade
   is the only transition, no gradients, no paths/lines, no colour animation beyond a
   `highlight` step, no per-glyph animation, no reverse playback, no negative `shift`. Skills can
   supply workarounds (scale-about-origin wipes, per-word elements, occluder rects on flat
   backgrounds). Recorded so the ad eval's brief is judged against what the format can say;
   **no issue proposed** unless the eval shows the ceiling costs the ad.

## What this does not answer

- Whether agents *actually* miss these in practice — that is the baseline run's job
  ([#447](https://github.com/MBehtemam/Montagent/issues/447)). The grades here predict it from
  the surface; they are not measurements.
- Which skill owns which row — [#449](https://github.com/MBehtemam/Montagent/issues/449).
