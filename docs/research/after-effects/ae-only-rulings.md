# Which recorded rulings refuse a capability for being After Effects-only, and which on an invariant?

Research for [#910](https://github.com/MBehtemam/Montagent/issues/910), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896). It is the fact base for
[#897](https://github.com/MBehtemam/Montagent/issues/897), which asks whether After Effects
counts as precedent under ADR-0145 and which earlier rulings would then be re-read. This
document records what each ruling says and where. It does not argue what should change.

## Sources and method

Everything here is read from the repository at `main` `51aa4b5`, or from its GitHub issues.
Nothing was run, and no web source was consulted.

- `docs/adr/` (206 files), `CONTEXT.md`, `skills/montagent/SKILL.md` (the "Not by design" and
  "What it can't do yet" lists), the other `skills/*`, and `crates/montagent-core/docs/format.md`.
  These were grepped for: After Effects, `AE`, reference class, precedent, ADR-0003, ADR-0145,
  expression, parent, parenting, IK, rig, camera, precomp, adjustment layer, track matte,
  repeater, spring, Lottie, script, skew, corner pin and wiggle. Then each hit was read in
  context. `format.md` names no After Effects ruling.
- Issues: the maps [#2](https://github.com/MBehtemam/Montagent/issues/2) (its "Out of scope"),
  [#591](https://github.com/MBehtemam/Montagent/issues/591) (still open; its "Out of scope" and
  decisions index) and [#663](https://github.com/MBehtemam/Montagent/issues/663) (closed), and
  the resolution comments on [#594](https://github.com/MBehtemam/Montagent/issues/594) and
  [#632](https://github.com/MBehtemam/Montagent/issues/632).

**What counts as a ruling here.** A recorded refusal or deferral whose text appeals to After
Effects, to "the After Effects class/side/layer", or to the reference class (CapCut and
Premiere), or one that refuses a mechanism the text itself attributes to After Effects. A
capability that *entered* is listed only where a part of it was refused. Refusals that cite
neither After Effects nor the reference class (for example ADR-0070's `where` predicate as "a
step towards an expression language", or ADR-0152's mask sources) are left out.

**Ground tags.**

- **RC (reference class).** The ground is that CapCut and Premiere lack the mechanism, that it
  is After Effects-only, or that it needs the entry test because it lacks precedent. This ground
  would fall if After Effects became precedent.
- **INV (invariant).** The ground stands regardless of where precedent comes from. The tag
  names which invariant: **LIT** (literal values), **VOC** (closed vocabulary), **VAL**
  (checkable by `validate`), **ESR** (exact-string replace), or another standing rule, such as
  **ADR-0017** (closed schema, no escape hatch), **BYTE** (byte-identical painting, a standing
  rule on #663), or a named ADR (ADR-0001, ADR-0005, ADR-0010, ADR-0077, ADR-0150, ADR-0154).
- **other.** The ground is neither: a cost, no case asking yet, a design preference or the
  map's destination. It is recorded, so that a reader can see it does not rest on the reference
  class, but it is not a standing invariant either.

**Revisit.** Does the ruling text name what reopens it? "None named" means it does not.

## The census

| # | Ruling and location | Capability refused or deferred | Grounds | Revisit condition | Deciding sentence |
| --- | --- | --- | --- | --- | --- |
| 1 | **Transform-only keyframes.** ADR-0003 *Consequences*; #2 *Out of scope* ("keyframes on *arbitrary* properties"). **Discharged** by ADR-0146 (#669). | Keyframes on any property other than the transform | **RC**: "After Effects' model". **INV (LIT)**: the inert-data principle, "reading the file would stop being enough to know what is on screen". | Named: ADR-0145 *What this does not decide* left it "open to decision" until #669. ADR-0146 then replaced it with a closed list of typed animatable properties. | ADR-0145: "ADR-0003's reason was After Effects' model. That reason no longer rules anything out by itself." |
| 2 | **Expressions.** ADR-0003 *Consequences* (originally); ADR-0145 *What this does not decide* (current ground); #2 and #663 *Out of scope*; #591 *Out of scope*; SKILL.md *Not by design*; CONTEXT.md **Recorded intent** (*Avoid*: "formula, expression"); ADR-0171 §2 (Lottie expressions). | Keyframe expressions, links between values | Originally **RC** ("Deferring After Effects defers that fight"). Now **INV (LIT)** only. | Changing an invariant "is its own ADR, argued for the whole format" (ADR-0145 rule 2). Nothing else is named. | ADR-0145: "**Expressions:** still refused, now because they fail the literal-values invariant." |
| 3 | **Nesting (precomps).** ADR-0003 *Consequences* "Nesting stays rejected"; #2 *Out of scope*; ADR-0145 *What this does not decide*; **erratum** ADR-0166 §1–2; #632 resolution (a provisional space-only `nest`). | A composition inside a composition, a container | Originally **RC**: "With After Effects out of scope… CapCut and Premiere are flat-timeline tools". ADR-0166 finds that premise "wrong by mechanism", so nesting is inside the class. What stands is **INV (ADR-0001)**: no local clock. | Named: a container "has to answer ADR-0001's inference… The answer has to be a rule that `validate` can enforce" (ADR-0166 §2). #632 gates the `nest` on its prototype and A/B score. | ADR-0166: "So nesting is not outside the reference class. Whether a nest enters the format is a separate decision with its own evidence." |
| 4 | **Effect stack or plugin architecture.** ADR-0003 *Consequences*; ADR-0040 ("never a stack in the After Effects sense"); #2 *Out of scope*; ADR-0145 (unchanged); ADR-0017 and ADR-0156 (open shader or script file); SKILL.md *Not by design*. | An open or stackable effect system, plugins, open shader or script files | **INV (VOC, ADR-0017)**: "A closed list published in the schema is discoverable by an agent from the schema alone". **RC** framing: "This follows from CapCut/Premiere rather than After Effects." | Named on #663: reopening ADR-0017 "needs evidence the named vocabulary proved too narrow". New effects join one ADR at a time (ADR-0156). | ADR-0145: "[ADR-0017]'s refusal of any open script, shader file or escape hatch stands." |
| 5 | **Skew and corner pin.** ADR-0012 *Decisions*; ADR-0040 *Deliberately absent* ("Rejected on model grounds"); ADR-0167 *Considered* (still refused). | Skew in the transform, corner pin | **RC**: "it lives in Corner Pin, which is the After Effects side of the line ADR-0003 drew" (ADR-0012); "free deformations with no reference-class precedent" (ADR-0167). **other**: model grounds in ADR-0040: skew belongs with the transform, not with effects, and it changes an element's bounds. | Partly. ADR-0012: "It arrives later as an entry in #22's closed vocabulary if it arrives at all." ADR-0040: "No evidence names a forcing case." ADR-0167 names none. | ADR-0167: "Skew and corner pin are free deformations with no reference-class precedent." |
| 6 | **Camera and depth sorting.** ADR-0167 *Considered*, front-matter and four tests (VOC row "No camera, no depth"); ADR-0040's "3D / perspective distortion" refusal, now amended to a per-element projection; #591 *Out of scope*; CONTEXT.md **Projection** (*Avoid*: camera, depth). | A camera entity (viewport, depth, parallax), depth-sorted placement | **RC**: "a camera and depth sorting are After Effects-only". **other**: "outside the map's destination" (ADR-0167); "A camera move is only ever a group transform over everything" (#591). | None named. A camera pan is handled as a test case for the group transform (#591 *Notes*). | ADR-0167: "a camera and depth sorting are After Effects-only and outside the map's destination." |
| 7 | **Parent link between siblings (parenting).** #594 resolution; #591 *Out of scope*; ADR-0166 §1 (the fact); ADR-0145 *Consequences*; #663 *Notes*. SKILL.md's "What it can't do yet" lists parenting as a gap with a workaround (#499), not as a refusal. | One element's transform following another's (After Effects' parent) | **RC** only. Explicitly *not* an ADR-0086 proof-bar failure. | Named, in two places. #594: "Reopening it would take a new ADR, out in the open." ADR-0145: "It stands, and that map may re-read it." #663 says it "should be re-read there". | #594: "Neither editor ships one. It is After Effects' parenting. It is **excluded by the reference class (ADR-0003)**. It did *not* fail ADR-0086's proof bar, so measuring it can't bring it back." |
| 8 | **IK and rig controllers.** #591 *Out of scope*. | Inverse kinematics, rig controls | **RC** only (cites ADR-0003). | None named. | #591: "**The After Effects-only layer:** expressions, IK and rig controllers (ADR-0003)." |
| 9 | **Adjustment-layer shape for a group transform.** #632 resolution, *Shape*. #594 found the mechanism in Premiere, so it is not After Effects-only. | A transform that applies to everything stacked below it | **INV (VAL via ADR-0150)**: "applied to pixels it is ADR-0150's dependency". **other**: "once scoped it reduces to a container". | None named. | #632: "A transform that 'applies to everything below it' was refused: once scoped it reduces to a container, and applied to pixels it is ADR-0150's dependency." |
| 10 | **Matte from another element (track matte).** ADR-0150 §4; CONTEXT.md **Transition** (*Avoid*: "track matte (refused…)"); SKILL.md *Not by design*. Premiere has a Track Matte Key (ADR-0150 *Precedent*). | One element's alpha or luma deciding where another shows | **INV (VAL)**: "the first edit `validate` cannot follow". **other**: the matte would be an element in a track that does not paint, so every tool would special-case it. **RC**: "The reference class is split. CapCut has no matte taken from another track." | Named: "A matte from another element may return only with an answer to both reasons above." | ADR-0150: "It is refused for what the reference does… The dependency lives in rendered pixels, not in the JSON." |
| 11 | **Spring easing.** ADR-0146 §8; SKILL.md *Not by design*. | Spring as a keyframe easing | **INV (ESR, via `shift`)**: "A split spring is not a spring, so `shift` would need a second mechanism or would change the motion." **other**: a bezier with overshoot plus extra keyframes covers it. **RC**: "no precedent in the reference class and no accepted prototype". | Named: "an owner-accepted prototype showing a look that beziers plus extra keyframes cannot reach, and an answer for the split." | ADR-0146: "It has no precedent in the reference class and no accepted prototype, so it could not enter under ADR-0145 today." |
| 12 | **Repeat or repeater.** ADR-0148 §1 (and §3, a stagger across elements); CONTEXT.md **Repeat, repeater, clone**; SKILL.md *Not by design*. | One element drawn as N copies; a timing link between elements | **INV (ESR, VAL)**: "A repeat fails the fourth test, whichever way it is shaped", and the checks iterate only the elements written in the file. The reference class only set the bar: "A parametric repeater is outside the reference class, so ADR-0145 asks for an owner-accepted prototype." The refusal does not rest on that. | Partly. "Premiere's Clone effect is named here, so it does not reopen the question." For a stagger across elements: "a per-`id` option on `shift` is the cheaper path to weigh… recorded, not decided." | ADR-0148: "A repeat draws things the file does not name." |
| 13 | **Render-time resampling morph.** ADR-0162 §1 and *Considered*; SKILL.md *Not by design*; CONTEXT.md **Morph**. | Automatic vertex matching between unlike paths, as After Effects and flubber do | **INV (LIT, ESR, VAL)**: "geometry the file never states, the correspondence is implicit and cannot be edited… `E-PATH-KEYFRAME-SHAPE` would weaken". **RC**: outside the class, "the entry test would need a prototype". **other**: "no new expressible end state". | Named: "under the entry test, if evidence shows hand-padding is a real source of agent errors." | ADR-0162: "Refused: the in-between shapes are geometry the file never states, the correspondence is implicit and cannot be edited." |
| 14 | **Trimming the fill.** ADR-0160 §3 and *Considered*. | Trimming a path's fill as After Effects and Lottie do | **INV (ADR-0154 §5)**: no implicit closing; the format does not draw what the file does not say. | None named. | ADR-0160: "After Effects and Lottie trim the path and fill the open piece as a chord, which is the implicit closing ADR-0154 §5 refuses." |
| 15 | **`trim_offset` on an open path.** ADR-0160 §4 and *Considered*. | After Effects' wrap of an open line into two pieces | **INV**: the wrapped result is "a picture no field states". | None named. | ADR-0160: "After Effects' wrap into two pieces, each with its own caps, is a picture no field states." |
| 16 | **Dashes that travel with the trim.** ADR-0160 §6. A departure that `format.md` records, not a refused field. | After Effects' dashing of the trimmed path | **INV (LIT)**: it cannot be written exactly. | None named. | ADR-0160: "Dashes that travel with the window cannot be written exactly, since that needs the outline's length in pixels." |
| 17 | **Text on a path: several lines on offset curves.** ADR-0161 §3 (`E-TEXT-PATH-BREAK`). Text on an arbitrary path itself is After Effects-only and *entered* through the entry test (the prototype on #764 was accepted). | Multi-line text on a path | **INV**: the format would promise a picture it cannot define. | None named. "A second line on the same curve is a second text element." | ADR-0161: "Several lines stacked on offset curves (After Effects) fold back inside any tight bend, so the format would promise a picture it cannot define." |
| 18 | **Text on a path: a path borrowed from another element.** ADR-0161 §2. In After Effects, Path Options takes "a mask as the path". | `"on_path": "<id>"` | **INV (ADR-0150's two reasons)**: a guide element that does not paint; the text would inherit another element's transform, box and keys. | None named. | ADR-0161: "An `"on_path": "<id>"` naming a `path` element was refused on ADR-0150's two reasons." |
| 19 | **Text on a path: Reverse Path.** ADR-0161 §4 and *Considered* ("Refused for now"). Reverse Path is one of After Effects' Path Options (§*Precedent*). | A `path_reverse` flag | **other**: no evidence yet that reversing the list by hand is error-prone. | Named: "may join later if the prototype shows that reversing a list by hand… is error-prone enough to earn a field." The #764 prototype found that reversing a closed list by hand "fails three ways" (#663 index), recorded as evidence for the open flag. | ADR-0161: "There is no reverse flag: to read along the bottom of a circle, or from inside it, write the points the other way." |
| 20 | **Speed ramp: audio that varispeeds.** ADR-0157 §4 and *Considered*; #663 *Out of scope* (ramped audio). The time-remap spelling itself entered, on equivalence to Premiere's speed graph. | Audio following a ramp, pitch moving with the rate, as After Effects does | **INV (ADR-0077)**: `atempo` takes one fixed rate, and the mix is built on exact sample counts at 48 kHz. **other**: "it needs a continuously varying resampler Montagent does not have." | Named: "a separate capability with its own precedent question"; #663 says ramped audio returns "as a fresh audio effort". | ADR-0157: "After Effects varispeeds the audio instead, and that is the alternative left for later." |
| 21 | **Motion blur: a project-wide shutter.** ADR-0155 §2 and *Considered*. The *Precedent* paragraph calls "a per-layer switch with a composition-wide shutter" After Effects-only. | A composition- or project-level shutter setting | **other**: "so the cost and the look stay visible where the agent edits." | Named: "A default that element fields override may follow if a case asks." | ADR-0155: "There is no project-wide setting. The field is written on each element that should blur." |
| 22 | **SVG as a source.** ADR-0027 (v1); ADR-0171 §1 (current, amends ADR-0027); SKILL.md *Not by design*; `montagent-prerender`. | A native SVG source element | **RC**: ADR-0027 says "Adobe's dedicated 'Import SVG files' help page exists only under **After Effects** — the tool ADR-0003 already excludes"; ADR-0171 §1 says neither Premiere nor CapCut imports SVG, "so the entry test would need an owner-accepted prototype". **INV (ADR-0010)**: `textlayout` is a second shaper. **INV (BYTE)**: `resvg` would be a parallel raster backend whose identity "cannot be shown to hold". **other**: no agreed size (Skia 0×0 against `usvg` 100×100). | Named: "Reopens only if a reference-class tool gains native import, or the `textlayout` ban is lifted… A player fixing its seek or text gaps does not reopen it." An SVG-to-`path` skill reopens at three or more projects hand-copying points. | ADR-0171: "Under ADR-0145 that is no precedent, so the entry test would need an owner-accepted prototype, and every route to one costs one of the two grounds above." |
| 23 | **Lottie as a source.** ADR-0171 §2; SKILL.md *Not by design*; `montagent-prerender`, which refuses a Lottie that uses expressions. | A native Lottie source element | **INV (ADR-0010)**: the `skottie` pin. **INV (BYTE)**: no safe player, since ThorVG's seeks depend on history. **INV (ADR-0005)**: its own clock. **INV (LIT)**: text "defined by what After Effects exports", and "expressions are literal-value violations by ADR-0145". **RC**: "Premiere does not import Lottie JSON." | Named: the same triggers as row 22. | ADR-0171: "Decided separately, on grounds that are not SVG's." |
| 24 | **A path as an SVG `d` string.** ADR-0014 (with `line`/`polygon`/`path`); ADR-0154 (a point-list `path` enters and "the `d`-string refusal stands"); CONTEXT.md **Line, polygon**. | A path written as a mini-language string | **INV (ESR)**: "unreadable by reading, unmatchable by exact-string replace". **RC** in ADR-0014 only: "the shape half of the After Effects class ADR-0003 ruled out". ADR-0154 and CONTEXT.md restate only the invariant ground. | ADR-0014 named a condition for the *point list* ("Admitting one is a new ADR about placement"), which ADR-0154 met. None is named for the `d` string. | CONTEXT.md: "A `path` written as an SVG `d` string is still refused: a mini-language inside a JSON string is unreadable by reading and unmatchable by exact-string replace." |
| 25 | **Camera shake and wiggle.** ADR-0156 *Considered*. | Procedural shake and wiggle motion | **INV (LIT)**: procedural motion the file would not show. | None named. "Use keyed position or rotation instead." | ADR-0156: "Refused: they are procedural motion the file would not show." |
| 26 | **Sweep and two-point conical gradients.** ADR-0149 §1. | Gradient kinds beyond `linear` and `radial` | **RC** only. | Named: "Each could enter later through its own prototype under ADR-0145, as one more enum value." | ADR-0149: "Sweep and two-point conical have no precedent in the reference class." |

### Counts by ground

26 rulings in all. Count by which tags appear among a ruling's grounds:

| Grounds | Count | Rows |
| --- | --- | --- |
| RC with no invariant (RC alone, or RC + other) | 5 | 5, 6, 7, 8, 26 |
| Invariant with no RC (INV alone, or INV + other) | 10 | 2, 9, 12, 14, 15, 16, 17, 18, 20, 25 |
| Both RC and an invariant | 9 | 1, 3, 4, 10, 11, 13, 22, 23, 24 |
| Neither (other only) | 2 | 19, 21 |

Row 2 is counted by its current ground. Its original ground, under ADR-0003, was RC. Row 12 is
counted as invariant, because the reference class set only its entry bar. Row 1 is already
discharged and row 3's RC premise is already corrected (ADR-0166). They are counted by the
grounds their records gave.

Of the 26, 16 name a revisit condition (rows 1, 2, 3, 4, 7, 10, 11, 13, 19, 20, 21, 22, 23,
26, plus the partial conditions in 5 and 12). Ten name none (rows 6, 8, 9, 14, 15, 16, 17,
18, 24 for the `d` string, and 25).

## Summary

**Rows that rest on the reference class with no invariant beside them** would be re-read if
After Effects became precedent:

- **Parent link** (row 7). It is the only one whose record says outright that the reference
  class alone excludes it. ADR-0145 and #663 both name it for re-reading, and #594 says
  reopening "would take a new ADR".
- **IK and rig controllers** (row 8). A one-line exclusion on #591 citing ADR-0003, with no
  invariant argument recorded.
- **Camera and depth sorting** (row 6). It is After Effects-only, plus "outside the map's
  destination". The second ground is a scope statement about #663, not an invariant.
- **Skew and corner pin** (row 5). Reference class in ADR-0012 and ADR-0167. ADR-0040 adds
  model grounds (transform/effect boundary, bounds), which belong to no invariant test.
- **Sweep and conical gradients** (row 26). The reference class only, with an entry route
  already named.

**Rows with both kinds of ground** keep an invariant ground if the reference class ground
falls. What would change is the procedural bar or a revisit trigger:

- Spring easing (11) and the resampling morph (13) would no longer need the entry test's
  prototype for lack of precedent. Each still has its invariant ground (the `shift` split; the
  file not stating the geometry).
- For SVG and Lottie (22, 23), one named reopen trigger is "a reference-class tool gains native
  import". ADR-0027 itself records that Adobe's SVG import page sits under After Effects. The
  ADR-0010, byte-identity, ADR-0005 and literal-value grounds are separate.
- The matte (10), the effect vocabulary (4) and the `d` string (24) keep VAL, VOC/ADR-0017 and
  ESR respectively.
- Transform-only keyframes (1) is already discharged by ADR-0146. Nesting (3) already had its
  reference-class premise corrected by ADR-0166, and what stands is ADR-0001's clock refusal.

**Rows that stand regardless**, on invariants alone: expressions (2, now LIT by ADR-0145's
own words), the adjustment-layer shape (9), the repeat (12), the three trim departures from
After Effects (14–16), the two text-on-path refusals (17, 18), audio varispeed (20, on ADR-0077
plus a missing resampler) and camera shake or wiggle (25). Rows 19 (Reverse Path) and 21 (a
project-wide shutter) rest on neither. Each waits on a case or on evidence, and each names
what would bring it in.
