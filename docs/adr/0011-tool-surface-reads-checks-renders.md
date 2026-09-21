---
status: accepted
---

# The tool surface reads, checks and renders — the agent brings its own editor

> **Amended by twenty later ADRs.** Read them before relying on anything below.
>
> - [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md) — the crop
>   rectangle becomes computable, and `shift` is unblocked
> - [ADR-0016](0016-no-format-version-the-unknown-key-error-is-the-mechanism.md) — the tool
>   surface gains no `migrate` verb
> - [ADR-0019](0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md) — clarifies
>   the write-tool invariant's scope
> - [ADR-0024](0024-measure-writes-the-fit-repair-not-the-verdict.md) — `measure` gains a
>   fitted-extent output
> - [ADR-0026](0026-exact-aspect-fit-both-spellings-stand.md) — `fmt` gains an explicit
>   exception
> - [ADR-0029](0029-line-baseline-half-leading.md)
> - [ADR-0030](0030-defaultable-field-presence-is-content.md) — `fmt` gains an explicit
>   exception for defaultable-field presence
> - [ADR-0035](0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md) — adds
>   a `measure` output
> - [ADR-0036](0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md) —
>   "informative preamble on legal inserts" is specified
> - [ADR-0037](0037-derived-time-signature-is-a-provenance-gap-not-a-tool.md) — no tenth
>   verb; the nine-tool surface holds
> - [ADR-0039](0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md)
>   — validate gains nothing; compare's scope grows
> - [ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) —
>   `fmt` gains a non-destructive `--check` mode; the write-tool invariant is restated for
>   key order
> - [ADR-0042](0042-montaget-json-is-a-convention-fmt-gets-a-shape-check.md) — adds a
>   precondition to `fmt`
> - [ADR-0051](0051-word-alignment-is-external-validate-and-compare-catch-drift.md) —
>   confirms the tool surface is unchanged
> - [ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md) — records the
>   fallback-order question this ADR closes as no longer open
> - [ADR-0070](0070-the-where-predicate-is-a-conjunction-of-whole-value-terms.md) —
>   specifies the `--where` predicate grammar left undefined here, and bounds *"returns
>   resolved values, never echoed fields"* to `--at`
> - [ADR-0074](0074-the-cut-lists-presence-set-is-every-element.md) — **the cut list's
>   presence set is every element, audio included.** The word *"on-screen"* in the
>   `query --from --to` line below is amended to *"the presence set"*; `unplaced` is
>   specified
> - [ADR-0077](0077-the-nine-render-readings-are-ratified.md) — ratifies `render`'s four
>   command-surface readings: a project with no `duration` renders to its last boundary, a
>   project with no `output` and no `--output` is exit 3, `--to` past the project's end is
>   legal (`--from` before 0 is not), and `<name>` in the derived partial name is the
>   declared `output`'s stem — or the project file's own stem where it declares none, so a
>   partial render still has a name on a project a *full* render would be refused for
> - [ADR-0078](0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)
>   — **the verb table below gains a `preview` row, and its counts become nine MCP tools,
>   twelve CLI commands, two resources.** `preview` is spec
>   [#168](https://github.com/MBehtemam/Montaget/issues/168)'s ninth MCP verb, built by
>   [#218](https://github.com/MBehtemam/Montaget/issues/218) and missing from the table
>   below, which `CONTEXT.md` calls authoritative. *"Eight MCP tools, eleven CLI commands"*
>   no longer holds. The exit-code table's row 3 is also what a `preview` budget hard-fail
>   returns — the document is legal and the invocation is what could not be satisfied
> - [ADR-0079](0079-fmt-check-stays-exit-0-l-layout-is-ratified.md) — the five-code ladder
>   stays five: `LAYOUT` never gates `fmt --check`'s exit code, the identical rule ADR-0041
>   already states for `validate` and `render`. A caller that wants a hard gate on canonical
>   form reads `--json`'s counted `summary.layout` field

> **Extended by [ADR-0030](./0030-defaultable-field-presence-is-content.md)**: `fmt`
> must never insert a default for an omitted field or strip one written explicitly at its
> default — presence/absence of `x`, `y`, `origin`, `scale`, `rotation`, `opacity` and
> `line_height` is content, exactly like declared extents (ADR-0013) and colour spellings
> (ADR-0014). Both spellings stay permanently legal.

> **Clarified by [ADR-0031](./0031-timeline-overview-is-not-required-to-be-spatial.md)**:
> whatever agent-facing overview is built on top of `query`'s aggregation modes is not
> required to render a spatial axis — a flat listing of the same facts measured
> indistinguishably on both cost and correctness. `timeline`'s "human's wide view" role in
> the table below is unaffected; that decision was never tested and stays open.

> **Extended by [ADR-0029](./0029-line-baseline-half-leading.md)**: `measure`'s
> per-line output gains `baseline_y`, the absolute y-coordinate the line-baseline
> formula resolves to, so an agent can verify text placement without re-deriving
> the arithmetic or rendering a frame.

> **Clarified by [ADR-0019](./0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)**:
> the write-tool invariant below is about tool call arguments, not about what an element's
> own fields may contain. An anchor's `{"below": "<id>"}` is data on an element, never an
> argument to a write tool, so it does not violate this ADR — a reading that came up twice
> independently and is recorded there so it is not rediscovered a third time.

> **Resolved by [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)**, on
> the three things this ADR parked. **`shift` is now adopted in full** — its keyframe rule is
> ADR-0012's case table, and the two legal-but-different files this ADR records are both
> wrong. **`query`'s expensive half is unblocked**: with `width`/`height` declared on the
> element the crop rectangle is computable by reading. **How `fit`, `align` and `scale`
> interact** is settled — `align` on images became `gravity`, and the aperture became `clip`.

> **Extended by [ADR-0042](./0042-montaget-json-is-a-convention-fmt-gets-a-shape-check.md)**:
> `fmt` gains a precondition — it refuses to act on a document missing the required top-level
> keys (`tracks`/`fps`/`frame`), sharing the same structural predicate `validate`'s schema
> layer uses. It stays unconditional on `error`/`review`/`note` findings otherwise; the eight
> MCP / eleven CLI counts below are unchanged.


Montaget exposes **nine verbs and two resources**. The surface's job is not to
provide editing verbs; it is to make **reading, checking, comparing and
rendering** cheap, and to let the agent edit the file with the tools it is
already strongest with.

| verb | MCP | CLI | what it is for |
| --- | --- | --- | --- |
| `validate` | ✅ | ✅ | does the document agree with itself and with the disk |
| `query` | ✅ | ✅ | what is true at an instant, over a range, or across a predicate |
| `frame` | ✅ | ✅ | what it looks like |
| `measure` | ✅ | ✅ | what text actually occupies |
| `compare` | ✅ | ✅ | what changed between two versions of the timeline |
| `render` | ✅ | ✅ | files in, video out |
| `shift` | ✅ | ✅ | move every time at or after an instant |
| `create_project` | ✅ | ✅ | scaffold a legal file so the agent never starts blank |
| `probe` | ❌ | ✅ | the one authority on what a media file's numbers are |
| `fmt` | ❌ | ✅ | rewrite the file in the canonical convention |
| `timeline` | ❌ | ✅ | the human's wide view |
| the schema | resource | — | discoverability is the schema's job |
| the format docs | resource | — | ditto |

**Eight MCP tools, eleven CLI commands, two resources.** The asymmetry is the
point and is argued below.

## How this was decided

Ten agent sessions. Five **consumers** (Opus, Sonnet, Haiku, Fable, and a Sonnet
briefed hostile, told to kill tools) each performed four real tasks against the
committed prototype project file and the fixture media before being allowed to
opine: read the stack at 6200 ms and 31000 ms, insert a 2000 ms pause at
30000 ms, hunt an 800 ms drift on disk, and move every sentence caption down
60 px. Then five **adversarial verifiers**, each on a model different from the
claim's author and each instructed to default to *refuted*, attacked the five
load-bearing claims the consumers produced.

**Declared contamination.** The five consumers were given a shared `/tmp` scratch
path. Two of them — Sonnet and Opus — logged another agent overwriting their
working copy mid-session. Both recorded it rather than tidying it away. No
measurement below rests on their timings or their friction reports; the
measurements that survive were re-derived. This is the author's setup error and
is recorded in the same spirit as [#9](https://github.com/MBehtemam/Montaget/issues/9)'s
declared contamination.

Three claims died under verification, and the corrections are load-bearing:

- **`census` does not dissolve `compare`** (see [`compare`](#compare)).
- **`jq` does not replace `query`, but it gets further than the rebuttal claimed**
  (see [`query`](#query)).
- **Frames are not cheap because something downsamples them** (see [`frame`](#frame)).

## The write-tool invariant holds, for the fourth and fifth time

> A tool that writes may only take a complete element, as a schema-shaped object.
> No tool takes a field name or an element id.

[#8](https://github.com/MBehtemam/Montaget/issues/8) found none of three agents
wanted an `update_element`. [#9](https://github.com/MBehtemam/Montaget/issues/9)'s
jury said unanimously that a CRUD API would be *actively negative*. This
exercise reproduces it twice more, now priced:

> *"`update_element` would have made Task D five round-trips instead of one
> command, **and would have cost me the `diff` that proved 'and nothing
> else.'**"*

That second clause is new and is the sharpest form of the argument. The CRUD API
does not merely cost round-trips; it destroys the artifact that makes the edit
verifiable.

One consumer proposed `edit --id sentence-05 --y 1597` while asserting the
invariant "is correct" in the same paragraph. That is a field name and an element
id — the two things the invariant forbids. It is recorded because it shows the
pull is real even in an agent that has read the rule.

## `validate`

Inherits [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) whole,
subject to the amendments in the last section. It is **first in the roster** by
unanimous consent: all five consumers hand-wrote a validator before trusting
their own edits, as all five of #9's jury did before them. This is now the
seventh independent reproduction of *"the auditor and the defendant are the same
person."*

## `query`

**Three modes, one verb.** A separate `boundaries` verb was wanted by three
consumers and refused by a fourth; folding it in satisfies both.

- **`query --at <t>`** — the resolved stack at an instant.
- **`query --from <a> --to <b>`** — the **cut list**: the intervals over which the
  set of on-screen elements is constant. Not sampled instants. It **must always
  name the boundary immediately outside the range on each side**, which folds in
  the `boundaries` want without a fourth verb and without the caller guessing a
  window.
- **`query --where <predicate> [--census <field>]`** — the matched set and the
  distribution.

### `query` returns resolved values, never echoed fields

This is the organising rule. Echoing `"scale":[[3018,1.0],[18018,1.08]]` back at
the agent tells it nothing it did not have; `scale 1.0170` is the entire point.
The output carries the interpolated animation value, the offset into the source,
the crop rectangle, the ink box, the painter's order, and the **`NOT COVERED`**
line — the region of the canvas no element covers, which no per-element listing
can produce and which a picture gives away for free.

**Text by default, `--json` instead, never both** — ADR-0006's wire decision
transplanted whole, for its own stated reason: this is an *explanation* of one
moment, and prose is the denser encoding of an explanation.

### Why `jq` does not kill it — and where the rebuttal was wrong

The hostile consumer killed `query --at` with a one-line `jq` filter. A verifier
ran all eight components of the disputed output:

| component | shell |
| --- | --- |
| presence list | **done** — the one-liner is correct |
| painter's order | **done**, but silently invents an order at layer ties |
| resolved keyframe value | **done** — a custom `jq` function reaches 1.0170 |
| offset into source | **hard** — needs `ffprobe` for the source duration |
| image crop rectangle | **impossible** |
| text ink box | **impossible** |
| `NOT COVERED` | **impossible** |
| previous / next boundary | **done** |

**The rebuttal to the kill was itself half wrong**: the resolved keyframe value
was cited as proof `jq` could not compete, and `jq` computes it. The kill fails
on the other three.

**And the reason those three are impossible is a fact about the format, not about
`jq`.** The crop rectangle is uncomputable because **how `fit`, `align` and
`scale` interact is specified nowhere**. The ink box needs the font binary and a
shaper. A Rust implementation faces the identical wall. So `query`'s expensive
half — the half that justifies the verb — is **blocked on
[#21](https://github.com/MBehtemam/Montaget/issues/21) and on `measure`**, not
merely expensive to build.

The hostile consumer's own caveat is upheld and recorded: this fixture contains
**no layer anchors**, so the one thing `jq` provably cannot do — resolve an
anchor chain — was never exercised by anyone.

## `frame`

**JPEG, at half the project's frame size by default. Full scale and PNG behind
flags. `--crop x,y,w,h` from the start.**

The reasoning that reached this conclusion in an earlier draft was wrong in both
directions and is corrected here, because the corrected version changes what the
default *protects against*.

Claude costs an image as `⌈width / 28⌉ × ⌈height / 28⌉` visual tokens — a pure
function of **decoded pixel dimensions**. Therefore:

- **Encoding format is irrelevant to context cost.** A 1.5 MB PNG and a 229 KB
  JPEG of the same frame cost **identical** tokens. The claim that a large PNG is
  "a large fraction of a turn's budget", made by two consumers, is **false**.
  JPEG is still preferred, purely for latency and disk.
- **Nothing downsamples the frame for you.** The ~1.19 MP ceiling that motivated
  "full resolution buys nothing" applies only to models before Claude 4.7. On the
  models that would drive Montaget the ceiling is ~3.75 MP, and a 1080×1920 frame
  passes through untouched.

| | tokens |
| --- | --- |
| 1080×1920 | **2691** |
| 540×960 | **700** |

**3.84×, tracking the true 4× pixel area.** So half scale is the default not
because full scale is redundant but because **full scale genuinely costs 2691
tokens every time the agent looks**, and no client-side ceiling will save the
caller from it.

**`frame` must print the `query --at` block alongside the image, unconditionally.**
Looking at a picture without knowing which elements produced it is how a defect
gets attributed to the wrong element. The `--describe` text form asked for in
[#26](https://github.com/MBehtemam/Montaget/issues/26) is not an alternative to
the picture; it is the picture's caption.

**And `frame` is demoted from a job it keeps being assigned.** It is how an agent
*believes* a layout; it is not how one *measures* a layout. An agent asked to
settle a 26 px overflow from the rendered frame could not, and fell back to
pixel-scanning raw RGB — where its first scan was wrong because the text colour
`#FFF8E8` and the background `#FBF3E3` differ by only (4, 5, 5).

## `measure`

Already mandated by [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md)
and [ADR-0008](./0008-line-breaks-belong-to-the-agent.md); this ADR supplies the
number that makes it non-optional.

**Nominal `size × line_height` overstates real rendered ink by 1.25×–1.48×.**
Measured by pixel-scanning the published reference frames, and independently
reproduced by a second agent using a different method (confining the scan to the
card interior, where only two colours are physically possible, rather than
thresholding against the near-identical pair above):

| element | size | lines | ink | nominal | ratio |
| --- | --- | --- | --- | --- | --- |
| sentence-05 | 55 | 1 | 41 px | 60.5 | 1.48× |
| sentence-06 | 57 | 2 | 100 px | 125.4 | 1.25× |
| sentence-07 | 57 | 2 | 99 px | 125.4 | 1.27× |
| sentence-08 | 55 | 1 | 42 px | 60.5 | 1.44× |
| sentence-quiz | 55 | 1 | 41 px | 60.5 | 1.48× |

The consequence, computed against the card at y 1453–1622 **after** the +60 px
restyle the exercise performed:

- Nominal metrics flag `sentence-05`, `-08` and `-quiz` as overflowing by
  **+5.25 px**. Real ink clears all three by **4.0–4.5 px**.
- Nominal metrics report `sentence-06` and `-07` at +37.7 px. Real ink puts them
  at **+24.5–25 px** — genuinely outside the card.

**Three false positives out of five, and two true positives.** So ADR-0006's
text-overflow check is a false-positive generator until `measure` exists, and a
check that cries wolf on 60% of a legal restyle is the alarm fatigue ADR-0006
itself identifies as a safety problem.

`measure` returns the per-line ink box and advance width, the block extent, and —
per ADR-0008 — the break opportunities with the segmenter and data version named.
`metrics` and `text_size` are worse names: neither lets a reader guess that break
opportunities come back in the same call. Per [ADR-0029](./0029-line-baseline-half-leading.md),
each line also carries its resolved `baseline_y`.

## `compare`

**Build it.** [#26](https://github.com/MBehtemam/Montaget/issues/26) handed this
verb to the tool-surface question after ten agent sessions asked for it
unprompted. An argument was made here that it should instead be **ruled out of
scope**, dissolved by a census against the current file — *"zero elements remain
at y=1537, five at 1597"* — needing no ref, no "last known-good" definition, and
no tool touching git.

**That argument was tested and it fails.** Two counter-examples, both built and
demonstrated:

**1. The census does not merely miss a defect; it endorses one.** The five
sentence sizes are `[55, 57, 57, 55, 55]` — the two 57s are the two-line
sentences, deliberate variance. Apply the over-broad replace that ADR-0005
documents as the normal editing mode, setting all five to 55, then run the
prescribed census:

```
CENSUS size: [55, 55, 55, 55, 55] → 5 of 5 at 55, zero outliers
```

The census reports the file as **healthier than before the edit**. Census
epistemology treats uniformity as correctness, so **any defect that destroys
deliberate variance is invisible in principle** — the old values exist nowhere in
the current document. The fixture carries the inverse tripwire too:
`word-08-target` (y 1352, size 49, beside siblings at 1373) is a census outlier
that is **correct**, because it pairs with `word-08-bridge` on the overflow
track. A census-guided agent is invited to "fix" it.

**2. On the time axis the census is not defined.** Shift everything at or after
30603 by +800 ms and skip one element whose start sits after a gap. Result:
`validate` reports **zero errors** (the preceding gap absorbs the skip), `git
diff` shows the defective element only as a **context line** — an absent edit
produces no hunk — and there is no census to run, because the 60 elements carry
26 distinct `start` values with no meaningful mode. The census statement one
would need is *"zero elements remain at start=61116"*, and **knowing 61116 is the
old value is version knowledge**. The ref-free census smuggles the ref in through
the agent's context window.

This matters because the ten sessions that asked for the verb asked after doing
**time-axis** edits — the axis where 0 of 3 agents survived in #8, and the axis
where the census has nothing to say.

**Shape.** `compare` takes an **explicit caller-supplied ref** — a path or a git
ref, never an inferred "last known-good", which turns #26's undefined-baseline
objection into the caller's stated choice. It reports **grouped deltas as facts**:

```
19 of 20 elements at/after 30603 moved +800 ms; vo-quiz-answer did not.
2 elements changed `size`; no instruction in this comparison touched size.
```

Grouping by observed delta needs **no selector concept**, which partly dissolves
the first of #26's three open sub-questions. `diff` is the wrong name: the tool's
entire point is that an absent edit produces no hunk, and the name must not
promise hunks.

**`compare` does not read git by default.** It takes a path; a git ref is sugar
resolved to a path. ADR-0006 settled that `validate` does not read git, and
nothing here licenses the surface to acquire a working-tree opinion.

## `probe` — CLI only, and it returns a quad

`probe` does **not** earn its place as an authoring convenience. ADR-0006 already
makes `validate` probe everything and cache on `(path, size, mtime)`, and no
consumer wanted a speculative probe while authoring. Standing alone as an
MCP verb it would be exactly the opt-in check
[ADR-0004](./0004-tracks-as-constrained-lanes.md) warns about.

It earns its place because **there must be exactly one authority on what a media
file's numbers are, and that turns out to be genuinely ambiguous.** On the
fixture's own reference MP4:

| field | value |
| --- | --- |
| container duration | 65.258667 s |
| video stream duration | 65.216016 s |
| `r_frame_rate` | 50/1 |
| `avg_frame_rate` | 24.9785 |
| `start_time` | 0.042031 s |

The project declares `duration: 65216` and `fps: 25` — matching the **video
stream**, and matching **neither** frame rate. An agent that computes
`frames = duration × fps` from the wrong pairing is out by 2×.

All eleven fixture mp3s show **no such split** (`format == stream`, `start_time`
0), so this is an artifact of how a container was muxed, not a general property
of media — which is precisely why the tool, and not the agent, must own it.

**`probe` therefore returns the quad and forces the caller to pick**: video
duration, container duration, `start_time`, and both frame rates — plus pixel
dimensions, alpha, sample rate and channels. **A single scalar named `duration`
is the failure mode**, because it invites arithmetic against the wrong axis. It
returns **integer milliseconds**; the float-seconds conversion is not the agent's
to fumble.

Image dimensions are part of this: `images/06.png` is 1536×2720 drawn into a
1080×1300 box under `fit: cover`, so ~47% of the source height is cropped, and
**none of that is in the document** — the objection that killed implicit `fill`
in ADR-0005.

## `render`

- **Full render writes the project's declared `output`. A partial render must
  never be able to land there.** `--from`/`--to` derives
  `out/<name>.<from>-<to>.mp4`, and an explicit `--output` equal to the project's
  `output` is refused while a range is set. Times go **in the name**: three
  previews in one turn named `preview-1/2/3` cannot be told apart, and the agent
  re-renders to find out.
- **`--from`/`--to` are half-open**, inheriting ADR-0005, and the tool says so in
  its output.
- **One asymmetry is documented, not left to be half-learned**: `render --from`
  mid-element is normal — a preview seeks into elements — while `shift --at`
  mid-element refuses.
- **Progress on stderr, coarse; the machine-readable result on stdout** — path,
  duration, frame count, wall time, realtime factor. A spinner is worth nothing
  to an agent; the realtime factor lets it budget the next call.
- **The failure mode that matters is not a crash — it is success.** After the
  restyle above, the project has **zero errors** and 25 px of text hanging off two
  cards, and `render` produces it happily and exits 0. So **`render` prints the
  `review` findings it did not refuse on, and ADR-0006's `NOT CHECKED` footer,
  after a successful render.** Otherwise exit 0 reads as *"the video is right"* —
  ADR-0004's `sequence`-label failure arriving through the one command that
  cannot be skipped.
- **Write to a temp path and rename atomically.** A truncated MP4 at the
  deliverable path reads as finished.

## `shift` — adopted, and **not fully specifiable yet**

ADR-0005's `shift(path, at, delta, scope)` is confirmed by every consumer that
attempted the edit. Three corrections, and one blocking dependency.

**The refusal must fire on ambiguity, not only on straddling.** Three consumers
independently discovered that the exercise's own instant, 30000 ms, sits **603 ms
before a cut on which five tracks agree** (30603), inside a 912 ms silence.
Nothing time-based straddles 30000, so `shift` as specified executes it
obediently, and the result **validates clean** and is almost certainly not what
was wanted. The nearest-boundary message ADR-0005 designs for the refusal case is
needed as an **informative preamble on legal inserts too**.

**`at` on an exact boundary is the normal case and an off-by-one there is
silent.** Half-open times mean an element ending exactly at `at` is not on screen
at `at`, yet ADR-0005's phrase *"every time at or after `at`"* would stretch it.
The spec sentence and the interval convention disagree; the schema must say which
wins.

**A write tool must re-emit the whole file in the canonical convention.** One
consumer's `json.dump(indent=2)` silently destroyed ADR-0005's one-element-per-
line, stable-key-order convention while performing a correct shift. A verifier
measured the blast radius: a **semantically empty** round-trip through `jq .` or
`json.dumps` produces a **~900-line diff of pure formatting noise**. Since agents
stop hand-editing and start scripting within ~90 seconds, the diff-legibility the
whole surface leans on survives only if every writer honours a convention no
standard tool implements. Hence `fmt`, and hence this as a hard requirement on
every write tool.

**And `shift` cannot ship until [#21](https://github.com/MBehtemam/Montaget/issues/21)
answers.** Two agents independently built both readings of a stretched element's
keyframes. The naive reading stretched `photo-06`'s zoom ramp from 15000 ms to
17000 ms — a ~0.9% scale error, roughly 10 px of framing. The careful reading
leaves the ramp topping out 131 ms before the element ends. **Both files are
legal, both validate clean, and they are different videos.** No check anyone has
proposed catches the difference. This ADR adopts `shift`'s shape and explicitly
does not settle its keyframe rule.

## Failure convention

Five exit codes, distinguished by **what the caller does next** — the only
distinction that pays for itself:

| code | meaning | next move |
| --- | --- | --- |
| 0 | no `error` findings | proceed |
| 1 | `error` findings | fix the project |
| 2 | the file could not be read or parsed | fix the *file* — the JSON is broken |
| 3 | the invocation was wrong | fix the command |
| 70 | internal failure (ffmpeg died, font stack failed) | retry or report |

**1 and 2 must not collapse.** Agents edit with `sed`; a mis-escaped pattern
producing invalid JSON means something completely different from "your timings
overlap."

**Exit non-zero only on `error`.** A `review`-level failure would break every
routine run in the edit loop, which is ADR-0006's alarm fatigue by another route.

**Every tool fails identically on a malformed file**, with line, column, byte
offset, the offending line verbatim, and a caret. **Nothing may partially process
a malformed file**: read, parse, check, write atomically, or do nothing. A `shift`
that parsed thirteen of fourteen tracks and wrote back would silently lose a track
behind a plausible-looking diff.

**An error is a finding.** Same objects and same stable codes as ADR-0006
(`E-SOURCE-OVERRUN`, `E-PARSE`, `R-VISUAL-GAP`, `N-QUANTIZATION`), including for
invocation errors, so there is exactly one thing to parse across the surface.

**Every write tool returns the new state's findings, never `ok`** — ADR-0006's
second structural mechanism, and a constraint on every write tool this ADR
defines, not a property of `validate`. *"Then I don't run `validate`; `validate`
runs me."*

## MCP and CLI: one implementation, deliberately unequal surfaces

**Both. One binary, one core library. The MCP server wraps the library, never the
CLI** — a subprocess per tool call would pay the startup
[ADR-0009](./0009-rust-host.md) says is paid once on the stdio binding.

**The CLI carries more, and that is deliberate.** ADR-0009's cost model is
per-schema and permanent: every MCP tool schema occupies the agent's context and
degrades tool selection on every turn, including turns with nothing to do with
video. A CLI subcommand costs nothing until invoked. So `probe`, `fmt` and
`timeline` are CLI-only — used once, once and never respectively across four real
tasks by five agents — and remain one `Bash` call away, since the agent already
carries a shell.

`timeline` is cut from the MCP surface on its own merits, unanimously among
consumers: over 65216 ms with 14 tracks and 60 elements, an ASCII chart is either
illegible or larger than the file. Every time an agent wanted something
timeline-shaped, the thing it actually wanted was narrower — an instant, a
boundary list, or a census. The map justifies `timeline` as serving *"the human
and the agent from one artifact"*; that is a good reason for it to exist and a bad
reason for it to occupy a schema slot.

**Three places the MCP server is more than a wrapper:**

1. **The schema and format docs as resources.** Discoverability is the schema's
   job, and a resource costs no tool slot. This is what makes *"how does the agent
   know how to edit `project.json`"* answerable at all — the same way it edits
   `package.json`. It has no CLI analogue that helps in the same way.
2. **Write tools returning findings.** Over the CLI, `shift` writes and the agent
   must remember to validate. Over MCP the return value **is** the findings, which
   is what converts an opt-in check into a structural one.
3. **A warm probe cache.** A long-lived server holds `(path, size, mtime) →`
   duration in memory across calls.

## Amendments to ADR-0006

[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) is accepted and
its design survives contact. Three things in it are presented as verified fact and
do not hold. They are corrected here rather than deferred, because ADR-0006 closes
by asserting that *"every numeric claim repeated in this ADR was verified by
script against the file before being written down."*

**1. The severity mechanism does not fire.** ADR-0006 computes *"the union of
visual coverage; a gap whose interval is uncovered in that union is `review`"*.
On the committed prototype the union is **100% covered at every instant** — not
only by the eight always-on header elements, but by the `photo` and `caption`
tracks, each of which is contiguous end to end. Enumerating **all 8192 subsets of
the 13 visual tracks** as the union basis, the number of uncovered gaps is only
ever **0, 9 or 11 — never 1**. Excluding the header alone gives zero. The rule as
written would keep reporting green with every photo, card and caption deleted.

**2. "One visual gap out of eleven" is unreproducible.** The **denominator is
exactly right** — 11 visual gaps, five each on `sentence-card` and
`sentence-text`, one on `caption-overflow`. The numerator is unreachable, and the
interval cited (30603–31403) is not a gap in this file; `31403` appears nowhere
in it. The reason is that ADR-0006's judges were shown a *different, defective*
file from #9's second exercise, and **that file is not committed to this
repository** — so the ADR's headline number cannot be checked by anyone.

**3. The exemplar finding contradicts the rule.** *"No element on any visual track
for 800 ms; the frame is background plus the header"* cannot both be true under a
union whose membership is derived from element `type`, as ADR-0006 explicitly and
deliberately requires so that no track can be excluded. Either the sentence or the
rule must change.

**Area-aware coverage is not the fix**, and the fixture shows why. The `photo` box
is 67.71% of the frame; maximum opaque coverage anywhere in the project is
**79.64%**, so a strict area rule fires at every instant and needs an unprincipled
threshold. The two header panels are filled `#FBF3E3`, **byte-identical to the
project background** — they paint background on background and an area rule counts
it. `chip-panel` at layer 30 opaquely occludes the photo at layer 10, so
correctness needs z-order and opacity. And **22 of the 40 visual elements have no
computable extent**, being text under an unsettled text model. Area-awareness is
opacity plus colour plus z-order plus a settled text model plus a threshold —
"a much larger check than it appears", in ADR-0006's own words about a smaller
check.

**The real defect is that the union has no notion of which elements are
*content*,** and `type` cannot express it. The one signal already in the file and
unused is `group: "header"`. What to do about it is
[#23](https://github.com/MBehtemam/Montaget/issues/23)'s, per ADR-0006's own
escape hatch: *"if #23 settles it differently, the severity rule follows it."*

**4. A miscitation.** ADR-0006 sends the geometry-aware same-layer check to
[#24](https://github.com/MBehtemam/Montaget/issues/24), but #24 is *"Repair the
layer anchor…"*. **The geometry check has no home in the map.**

**5. Drift detection rests on the cache alone.** The document records **no source
duration**, so `max(source_end)` is only a *lower bound*. A source that **shrank**
is caught with certainty; a source that **grew** is caught only by the accident
that this fixture consumes every source to its last millisecond. A file that
gained 800 ms while the project used its first two seconds is invisible to every
check constructible from the document plus the disk. ADR-0006's
`(path, size, mtime)` cache-miss line is therefore **not a performance
optimisation with a pleasant side effect — it is the sole mechanism** catching
that defect class, and must be specified as load-bearing.

## Consequences

- **`shift` is adopted in shape and not in full**, pending #21's keyframe rule.
- **`query`'s expensive half is blocked** on #21 and on the text model; the
  presence half is buildable today and is worth strictly less than the whole.
- **ADR-0006 requires the corrections above.** Its design stands; three of its
  stated facts do not.
- **Every write tool must round-trip the canonical serialization**, or the
  diff-legibility the surface depends on evaporates on first use.
- **`fmt` exists because the convention is normative and nothing enforced it.**
  An agent reverse-engineered the serializer to make one legal edit and got it
  99% right; the miss was `"frame": { "width": ... }`, which the file writes with
  inner spaces and `json.dumps` does not.
- **The surface count is ~11, not 8.** #10's original "roughly 4–8 tools, not 40"
  was already withdrawn by #4; this ADR confirms the replacement invariant is what
  bounds the surface, and that the number lands where #4 predicted.
- **Not settled here**: how `fit`, `align` and `scale` interact (#21); what counts
  as content coverage (#23); draw order at a layer tie, which currently has no
  owner (**settled by [ADR-0060](./0060-layer-tie-is-an-error-array-order-stays-meaningless.md)**:
  a geometry-overlapping tie is `validate` `error`, never a fallback order; array
  order stays meaningless); and font vendoring, which makes every `fonts` table
  unverifiable today.

## Naming

The vocabulary is a UI for the agent, so three names are recorded with what a
worse one would have cost:

- **`shift`, not `insert_pause`.** The friendlier name implies the tool understood
  that a pause between shots was wanted; it would have executed 30000 ms with the
  same silent 603 ms error while sounding as though it had checked. A good name
  here is one that does not imply comprehension the tool lacks.
- **`compare`, not `diff`.** The tool exists because an absent edit produces no
  hunk. A name that promises hunks defeats it.
- **`validate`, not `lint`.** `lint` reads as style suggestions and trains a
  reader to skip the output — for the one check that compares the document to
  something outside itself.

And one naming failure that is not Montaget's, recorded because it generalises:
the fixture's track named `caption` does **not** hold the sentence captions; the
track named `sentence-text` does. An agent asked to restyle "every sentence
caption" was pointed at the wrong track by name, and resolved it only by finding
three independent selectors that agreed on the same five elements. That is the
strongest argument for `query --where` **echoing the matched set rather than a
count**: naming will go wrong, and the surface's job is to make it go wrong
*before* the write.
