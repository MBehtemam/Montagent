You are a juror on a technical design court. Answer the Question below and nothing else — no
editing, no tools, no recommendations to the judge, no follow-up questions. You have all the
facts you are getting; where a fact is missing, say what you would need and answer anyway on
the balance of what is here.

Output EXACTLY this block, once for the whole ballot, naming the model that backs you:

🗳️ **Juror <n>** (<the model backing you>) — **VOTE: <one line per sub-question, Q1..Q7, each naming your chosen option or a one-line answer>**

**Reasoning:** <why, per sub-question>
**Trade-offs:** <what each choice costs — for the multiple-choice ones, why not the others>

You may reject the framing of any sub-question outright: if the options are all bad or the
question is malformed, saying so is a valid ballot, not a failed one.

=====================================================================
# BACKGROUND

**Montagent** is a declarative video editor for AI agents. A project is a JSON file the agent
edits with ordinary file tools; a Rust host exposes verbs over both an **MCP server** and a
**CLI**. It is written in Rust. Architectural decisions are recorded as numbered ADRs, which
are **amended, never rewritten** — a later ADR that corrects an earlier one says so, and the
earlier one gets a banner naming the amendment.

## ADR-0011 — the tool surface

Nine verbs and two resources. "The surface's job is not to provide editing verbs; it is to
make reading, checking, comparing and rendering cheap." The verb table (since amended by
twenty-seven later ADRs):

| verb | MCP | CLI | what it is for |
| --- | --- | --- | --- |
| `validate` | yes | yes | does the document agree with itself and with the disk |
| `query` | yes | yes | what is true at an instant, over a range, or across a predicate |
| `frame` | yes | yes | what it looks like |
| `measure` | yes | yes | what text actually occupies |
| `compare` | yes | yes | what changed between two versions of the timeline |
| `render` | yes | yes | files in, video out |
| `shift` | yes | yes | move every time at or after an instant |
| `create_project` | yes | yes | scaffold a legal file so the agent never starts blank |
| `preview` | yes | yes | what a span looks like in motion, fast enough to scrub |
| `probe` | CLI only | yes | the one authority on what a media file's numbers are |
| `fmt` | CLI only | yes | rewrite the file in the canonical convention |
| `timeline` | CLI only | yes | the human's wide view |

ADR-0011's `frame` section, verbatim in the parts that matter:

> **JPEG, at half the project's frame size by default. Full scale and PNG behind flags.
> `--crop x,y,w,h` from the start.**
>
> Claude costs an image as `ceil(width/28) x ceil(height/28)` visual tokens — a pure function
> of decoded pixel dimensions. Therefore: **encoding format is irrelevant to context cost**
> (a 1.5 MB PNG and a 229 KB JPEG of the same frame cost identical tokens; JPEG is still
> preferred, purely for latency and disk). And **nothing downsamples the frame for you**.
>
> | | tokens | 1080x1920 | **2691** | 540x960 | **700** |
>
> So half scale is the default not because full scale is redundant but because **full scale
> genuinely costs 2691 tokens every time the agent looks**.
>
> **`frame` must print the `query --at` block alongside the image, unconditionally.** Looking
> at a picture without knowing which elements produced it is how a defect gets attributed to
> the wrong element. The `--describe` text form is not an alternative to the picture; it is
> the picture's caption.
>
> **And `frame` is demoted from a job it keeps being assigned.** It is how an agent
> *believes* a layout; it is not how one *measures* a layout.

## The live `frame` surface, as built

**MCP** (`FrameParams`), with the doc comments verbatim:
- `project: String` — path to the project file.
- `at: i64` — **REQUIRED, not an Option**. "The instant to draw, in absolute milliseconds on
  the project's one clock."
- `crop: Option<String>` — "Return just this region, as `x,y,w,h` in whole frame-space pixels
  at true scale — so you can look closely at one card without paying for the whole canvas."
- `full: bool` — "Return the frame at the project's true pixel dimensions instead of half of
  them. A 1080x1920 frame costs 2691 visual tokens at full scale and 700 at half; this flag
  is you choosing to spend the difference."
- `png: bool` — "Return PNG instead of JPEG. It costs the same tokens ... and buys lossless
  pixels for more latency."
- `json: bool` — "Return the canonical JSON *instead of* the text caption, never alongside
  it. The image comes back either way."

The struct's own doc comment: *"There is deliberately **no argument that suppresses the
caption**. ADR-0011: 'frame must print the query --at block alongside the image,
unconditionally' — an agent looking at a picture without knowing which elements produced it
attributes the defect to the wrong element, so the block is the picture's caption rather than
an alternative to it, and there is nothing to turn off. There is also no `out`: the CLI writes
a file because a terminal cannot show a picture, and this surface hands the image back in the
result."*

The MCP handler returns the **caption text block first and the image block second**, "so a
client that renders content blocks in order shows the picture under the list of what is in
it — which is the reading order ADR-0011 argues for, the block being the picture's caption."

**CLI** (`montagent frame`): same flags, plus `--at` is `Option<i64>` (already optional), plus
`--out PATH` which is **required and deliberately not defaulted**: *"no ADR names a filename
for this, and a verb that invented one would be writing a file into somebody's project
directory on a read."* The CLI handler carries this comment: *"Whether the flags ask for a
frame at all — a missing `--at`, an unparseable `--crop` — is the verb's rule and not argv's:
the MCP surface takes the same arguments with no clap to arrange them, and a clap requirement
here would leave that surface uncovered."* The core verb's `Ask.at` is already `Option<i64>`.

`frame` offers **no `--verbose`**, for the same reason `query` and `measure` offer none: "the
answer itself is the output and is never collapsed."

## Range precedents on other verbs

Three verbs already take a range, all spelled `--from`/`--to`, all half-open `[from, to)`,
all saying so in nearly the same words:
- **`query --from --to`** returns the **cut list** — "the intervals over which the presence
  set is constant". Both are `Option`; the pair is what puts `query` in range mode.
- **`render --from --to`** — "Render only from this instant... Asked for with `--to`. A partial
  render is written to `out/<name>.<from>-<to>.mp4`." "The end of the partial range, exclusive:
  the range is half-open `[from, to)`."
- **`preview --from --to`** — "Preview only from this instant... Asked for with `--to`." Same
  half-open sentence. Writes `out/<name>.preview.<from>-<to>.mp4`.

`measure` is the precedent for a verb with **mutually exclusive argument groups**: `element` /
`at` / `elements` / `all` are each exclusive with the others, and the overlap is refused **in
the verb**, not in `clap`, so both surfaces inherit the rule.

## The effort this ticket belongs to

A map: **"`frame` gains a range mode that returns ONE labelled contact sheet"**, whose instants
are taken from the document's own visual-state boundaries, so an agent can see a span of the
video in one call instead of N. Done means: an agent asks `frame` for a range and gets back a
single tiled image, each tile labelled with its instant and what produced it, with the
selection rule, the tile scale and anything skipped disclosed in the answer. **Motion stays
with the human via `preview` and is explicitly out of scope** (an agent reading a GIF sees one
decoded frame). **Any verdict layer is out of scope** — no scoring, no "tile 7 looks wrong".

The evidence is an experiment, not an opinion. A fixture was doctored with two defects that the
`validate` checker passes at 0 errors: a wrong photo source (readable in the JSON) and a caption
recoloured to exactly its card colour (**visible only in pixels** — an ink-box geometry check
cannot see colour, since invisible text has a perfectly correct ink box). Three agents on three
models were told only that "something looks wrong". Two found everything and **both asked for
this feature unprompted, in nearly the same words**, both describing the same manual ritual it
deletes: take `query --from --to`'s 46 intervals, hand-collapse the audio-only ones to ~14–18
visual states, loop `frame`, read the images back. The third missed the pixel-only defect: it
hand-rolled a sampler over `query --at` every 200 ms checking ink-box overlap, then **claimed
full visual coverage**. Its sampler was structurally blind and its silence read as coverage.
**Any selection rule this feature adopts can fail the same way, and the disclosure is the only
defence.**

Both successful trial agents spelled their ask as a `frame` flag — `frame --per-state`,
`frame --each-cut`. A court and a trial both landed on **extending `frame` with a range rather
than adding a tenth verb**: an MCP schema costs context on every turn, while the question is
the one `frame` already answers — "what does it look like" — now asked of a span.

## Measurements the decisions rest on

**From the token-cost ticket (#397):**
- The `ceil(w/28) x ceil(h/28)` formula **holds, but on _served_ dimensions** — what the API
  actually delivers after its own downscale, not what you authored. `540x960 -> 700` is right
  everywhere; `1080x1920 -> 2691` is right only on the newest tier, because the standard tier
  downscales to `819x1456 = 1560`. So `frame`'s own doc string is half wrong.
- Encoding is irrelevant as claimed: 700 tokens for both JPEG and PNG.
- **"16 for the price of one" is true but tiling itself saves nothing — shrinking does.**
  16 loose tiles cost 720 against a sheet's 700, and at 18 the sheet actually *loses*. The real
  saving (12,600 -> 700) is a **20x resolution trade**, so any budget policy is a legibility
  policy.
- **The caption is a second, uncosted budget at 20.1x**: 28,410 characters across 18 states
  (i.e. 18 full `query --at` blocks) against 1,413 characters for one sheet answer.
- **A correctness cliff**: more than 20 image blocks in one request **silently clamps every
  image in that request**, earlier turns' included. The status-quo N-calls path crosses this on
  a 65-second video. This is a correctness argument for the sheet, not a budget one.

**From the legibility ticket (#396):** tile count is the wrong knob; **served tile width** is.
Three defect classes die at three widths: fine detail at ~140 px / ~30 tiles; large-area
substitution and absence-of-text both survive 92 px / 72 tiles. Sheets must be composed at the
size the API serves (1568 px long edge); measuring on the authored image **overstates every
threshold by 5–6x**. A **fitted** label outlives the pictures (readable at 6.3 px type) so it
must not set the budget, but a **fixed-size** label overflows and overprints its neighbours.
**Cropping every tile buys 2.33x scale and total blindness outside the band** — the wrong-photo
defect vanished entirely from a cropped sheet that looked clean.

## ADR-0094 (accepted) — which instants go on the sheet

**A tile is one visual state, sampled at the first frame the project's grid actually paints
inside it.**
1. The collapse is a **`type` filter inside `frame`, followed by a re-merge** of adjacent
   intervals whose filtered presence set is equal. No `--visual` flag, no second `query` mode.
   The filter alone changes nothing — **the re-merge is the feature** (46 intervals -> ~18).
2. The instant is the least frame index `n` whose painted millisecond `floor(n*1000/fps)` falls
   inside the run. Not the midpoint — midpoint was rejected 3–0 because it is *synthetic*, so no
   other verb could reproduce the tile.
3. **Keyframes are excluded by default and available behind a flag.** The count of untiled
   keyframe change points is named in every answer regardless.
4. A run containing no painted frame **gets no tile and is named** in `skipped` with reason
   `no-grid-frame`.
5. **Uniform infill is a flag, never a default, and is a gap ceiling (in ms), not a count.**
   Strictly additive, never displaces a document-derived tile, first evicted under budget,
   labelled in a different register.
6. **Every answer carries an unconditional structured disclosure** — the rule, the per-tile
   provenance, what was skipped and why, the audio-only boundaries dropped, and a fixed
   `blind_to` enumeration — **plus one sentence of prose restating it.**

ADR-0094 explicitly says: *"This ADR fixes which instants the sheet shows and what it must say
about the ones it does not. It does not fix the flag spelling or the verb table — that is
#401's, and the amendment to ADR-0011 lands there."*

The load-bearing support was a passage no juror was shown: an earlier ADR had already measured
47 boundaries / 19 visual / **28 audio-only** and already called this sheet's own operation a
failure — *"it reports an interval it believes is constant, and is wrong about, with nothing in
the output to suggest otherwise"* — so the sheet **must name the audio-only boundaries it
dropped**.

## ADR-0095 (accepted) — the tile budget and what overflow does

**The budget is served tile width: a 180 px target, a 140 px floor, and a refusal past it.**
- Both candidate currencies fail on measurement. **Tokens are flat**: a sheet spends 1518–1568
  of the tier's 1568 tokens at *every* tile count from 4 to 48 — it saturates the cap
  everywhere, so a token budget never fires. **Tile count is aspect-dependent**: a cropped 3:1
  band admits 112 tiles at 180 px where whole 9:16 frames admit 18.
- **Count is therefore derived, never set by the caller.** `frame`'s range mode takes **no
  tile-count argument**, and "there is no flag for 'give me 40 tiles' to have to refuse."
- The two constants give exact cliffs: **180 px => 18 tiles**, **140 px => 30 tiles**.
- State the cost as an equivalence, not a budget: **18 visual states cost exactly one
  `frame --full`** (1560 tokens each on the standard tier).
- **Overflow degrades once, then refuses, naming sub-ranges that would fit.** It may not thin,
  may not split (a split loses the wrong-photo defect *entirely*, and >20 image blocks clamps
  the whole request), and may not reshape into a strip (392 of 1568 tokens for 87 px tiles).
- **Tiles are rasterized at true project pixels and composited down. No proxy ladder enters
  `frame`** — this is stated as a decision rather than an omission, because it is the obvious
  optimisation and it buys ~10% at the cost of the one guarantee `frame` was built around:
  *"one tool it can unconditionally trust for pixel-accurate checks... without first asking
  whether what it's looking at is a lie."* Half-scale rasterization saves only 6–12% anyway;
  decode dominates.
- **Every range answer discloses the served tile width and which rung produced it**, degraded
  or not.
- A 500 ms wall-clock budget that binds single-frame `frame` is stated as **not** binding the
  range mode (a sheet is 2.9x that); the width floor already bounds time at ~2.4 s.

ADR-0095's "what this does not decide" and its risks, on this ticket:
> **Flag spelling and the verb table — #401.** This ADR fixes that a refusal exists and what it
> must name, not what the refusal's code is called or how the range arguments are spelled.
> **#401 now carries more weight than its title suggests** — see the risk below.
>
> **The refusal is the common case for anything longer than the fixture, not the edge case.**
> The fixture's ~18 visual states land on the 180 px target exactly, which is luck rather than
> design: 65 seconds of this density fits, and roughly 110 seconds does not fit even at the
> floor. Callers will hit the refusal routinely. This is accepted — the alternative is a sheet
> that lies — but it makes **the refusal's ergonomics load-bearing**, and that is #401's, which
> is why the pointer above is emphatic. **A refusal that does not make narrowing obvious will
> read as the feature being broken.**

## Sibling tickets, still open — NOT yours to decide

- **#400 — what each tile is labelled with.** Open. The label must be *fitted* to tile width.
- **#406 — whether the range mode crops every tile, and what a cropped sheet must discloses.**
  Open, and **blocked** by #402 — a live defect where `--crop` silently half-scales its output.
  ADR-0095 supplies the constant a cropped sheet's count would be derived from, not whether the
  mode ships.
- **#412 — the codes and classes the sheet's refusal, skipped runs and `blind_to` carry**, under
  the project's findings model (which gives one verb facts and no verdicts, and gives
  refuse-class findings a mandatory repair field).
- **#407 — measure whether a keyframe tile catches anything the run-start tile misses.** May flip
  ADR-0094's keyframes-off default.

=====================================================================
# THE QUESTION

Decide how `frame`'s range mode appears on the tool surface. Seven sub-questions. Answer all
seven.

**Q1 — The argument names and the half-open reading.** Does `frame` take `--from`/`--to` with
the same half-open `[from, to)` reading as `query`, `preview` and `render`, or something else
(e.g. the `--per-state` / `--each-cut` spellings the trial agents actually reached for)? And
does it inherit the same *defaulting* as its neighbours, where omitting `--to` is an error
rather than meaning "to the end of the project"?

**Q2 — What puts `frame` into range mode, and is `--at` exclusive with it.** Either:
(a) the presence of `--from`/`--to` *is* the mode, with `--at` plus `--from` refused as a
malformed invocation; or
(b) a separate `--sheet`-style flag turns the mode on, with the range as its arguments.

**Q3 — MCP, CLI, or both.** `frame` is on both surfaces today. On MCP the range mode would
return the sheet inline as one image block; on the CLI it would write one file to `--out`.
Relevant: one trial agent stayed on the CLI *purely* because looping 18 renders in one bash
call beat 18 MCP round-trips — then paid an extra file-read per frame, and **explicitly called
both options bad**. Ship the range mode on both, MCP only, or CLI only?

**Q4 — `--full`.** On single-frame `frame` this flag means "true project pixels, 2691 tokens
instead of 700, and that is the whole of what this flag does." On a sheet the currency is
different: ADR-0095 fixed that a sheet spends 1518–1568 of the tier's 1568 tokens at *every*
tile count, so there is no difference left to spend; and ADR-0095 already ruled tiles are
rasterized at true project pixels and composited down. Options: refuse `--full` with a range as
an invocation error naming why; silently ignore it; or give it a new meaning with a range (and
if so, what).

**Q5 — `--png` and `--json`.** Does each carry over to the range mode unchanged? `--png` is
about encoding, which #397 confirmed is token-irrelevant. `--json` currently means "canonical
JSON *instead of* the text caption, never alongside it; the image comes back either way" — and
ADR-0094 mandates an unconditional *structured* disclosure. Does the **plain-text** form have to
carry the full `rule` / `skipped` / `coverage` / `blind_to` content too, or may the structured
disclosure live only behind `--json` with prose carrying a summary?

**Q6 — `--crop`.** #406 (blocked by #402) owns whether per-tile crop ships and what a cropped
sheet discloses. Does this ticket settle anything about `--crop` now — for instance, that if the
mode ships it is the existing `--crop` composed with the existing range and never a new flag
name, with `--crop` plus range recorded as *not yet legal* until #406 — or does this ticket say
nothing about `--crop`, leaving even the spelling to #406?

**Q7 — The `query --at` caption, which the range mode cannot honour as written.** ADR-0011
requires the resolved stack alongside the image **unconditionally**, and both the MCP schema and
the CLI doc comment state there is deliberately no flag to suppress it. #397 measured the
sheet's version of that obligation at 28,410 characters across 18 states against 1,413 for one
sheet answer — 20.1x, and the larger *uncosted* half of the cost. So the range mode must do
something ADR-0011's words forbid. Which:
(a) the **per-tile label** discharges the caption obligation — the label *is* the caption at
sheet scale, and ADR-0011 is amended to say the obligation is "the agent must be able to
attribute what it sees", satisfied per-tile by #400's label rather than by 18 full stacks;
(b) the sheet prints **one** full `query --at`-style block for the range as a whole (a cut list,
not 18 stacks);
(c) the sheet prints **all 18 blocks** and pays the 20.1x;
(d) something else.
