# Brief: which instants go on the contact sheet

Sent **verbatim and identical** to all three jurors, with only the juror number and
the model name changed in the ballot template at the foot.

---

You are Juror <n> on a design court. Answer the question below and nothing else. Do NOT use tools, do NOT read or edit files, do NOT make recommendations to whoever relays this. Reply with ONLY the ballot block specified at the end. The model backing you is <model> — name it in your ballot.

## Background

Montagent is a CLI/MCP tool that reads, checks and renders a declarative video project (a JSON document describing tracks of elements with start/end times in absolute milliseconds). An AI agent edits the JSON with ordinary file tools and calls Montagent's verbs to see what it did.

Relevant verbs:
- `frame <project> --at <ms>` — rasterizes ONE instant, returns the image plus a `query --at` view of what is present. JPEG at half the project's frame size by default; `--full` and `--png` behind flags; `--crop x,y,w,h` returns a region.
- `query --at <ms>` — the resolved stack at one instant.
- `query --from <a> --to <b>` — the "cut list": the intervals over which the presence set is constant.
- `preview` — renders motion video, for a human.
- `validate` — reports facts, never verdicts.

**The feature being designed** (wayfinder map #395): `frame` gains a **range mode** returning ONE labelled contact sheet — a single tiled image — whose instants are taken from the document's own visual-state boundaries, so an agent can see a span of video in one call instead of N. Done means: an agent asks for a range and gets a single tiled image, each tile labelled with its instant and what produced it, with the selection rule, the tile scale and anything skipped disclosed in the answer. Motion stays with the human via `preview` (explicitly out of scope). Any verdict layer is out of scope — no scoring, no "tile 7 looks wrong".

**Why this exists — the experiment that motivated it.** A real fixture (65.2 s, 1080x1920, 25 fps language-teaching video) was doctored with two defects that `validate` passes at 0 errors: a photo element pointed at the wrong image file, and a text element recoloured to exactly its background card's colour (invisible text — readable in the JSON, but only detectable in pixels). Three agents on three models were given the tool and told only "something looks wrong".
- All three found the JSON-readable defect by *reading the file*, before rendering anything.
- The two that found everything both asked for this range feature unprompted, and both described the same manual ritual it deletes: take `query --from --to`'s **46 intervals**, hand-collapse the audio-only ones to **~14–18 visual states**, loop `frame`, read the images back.
- **The load-bearing failure:** the agent that missed the pixel-only defect hand-rolled a sampler over `query --at` every 200 ms, checking ink-box overlap and frame bounds, then claimed full visual coverage of all 65216 ms. Ink-box geometry cannot see colour: invisible text has a perfectly correct ink box. **Its sampler was structurally blind and its silence read as coverage.** Any selection rule this feature adopts can fail the same way, and disclosure is the only defence.

**Measured findings from sibling tickets — treat these as established fact:**
- Visual token cost is ceil(w/28) x ceil(h/28) on *served* dimensions. An image is downscaled by the API to fit a resolution tier (standard: 1568 px max long edge AND <=1568 tokens) before the model sees it.
- **More than 20 image blocks in one request silently clamps every image in that request**, including earlier turns'. The status-quo N-frame-calls path crosses this on a 65 s video. This is a correctness argument for the sheet, not a budget one.
- Tiling per se saves nothing; shrinking does. 16 loose tiles cost 720 tokens vs a sheet's 700. The real saving (12,600 -> 700) is a 20x resolution trade, so any budget policy is a legibility policy.
- The caption text printed beside images is a second, uncosted budget: 18 per-frame captions = 28,410 characters vs 1,413 for one sheet answer (20.1x).
- **Legibility, measured by building sheets and looking at them:** what decides whether a defect survives is *served tile width*, not tile count. Fine detail (small type) dies at ~140 px tile width (~30 tiles per image budget). Large-area substitution (the wrong photo) and absence-of-text-on-a-card both survive 92 px (72+ tiles). A per-tile label fitted to the tile width survives 72 tiles at 6.3 px of type — the label is NOT the binding constraint. Recommended working point: **~18 tiles at ~180 px**, because the observer was primed and the measured cliff is an upper bound on legibility, not a floor.
- The wrong-photo defect is **invisible in any single frame** — it exists only as a relation between tiles.

**Accepted decisions (ADRs) you must reason against, not around:**
- **ADR-0074** — "The cut list's presence set is **every** element, audio included." It explicitly *rejected* a `--visual` flag or second mode: "Each member states its `type`, so a caller that wants the visual cut list **filters one field of an answer it already has**. No flag, no second mode, and no `--visual`: the narrowing is a filter over the returned members, not a different question to ask." It also established that `presence set` is one defined term meaning the same thing in both `query` modes, and that elements the document does not place on the clock are *named* in the answer, never silently dropped.
- **ADR-0035** — a keyframe's `t` is NOT constrained to the frame grid; off-grid keyframe times are ordinary, meaningful data. "Alignment is never itself the check, only its structural consequences are." A bare note on every off-grid keyframe was rejected as alarm fatigue. ADR-0077 amends it: the whole millisecond frame n is painted at is floor(n x 1000 / fps).
- **ADR-0006** — `validate` reports facts; `render` enforces. No verdicts anywhere.
- **ADR-0011** — the tool surface; `frame`'s defaults (half scale, JPEG) and `--crop`/`--full`/`--png`.
- **ADR-0021 / 0046 / 0065 / 0067 / 0078** — the `preview` proxy ladder: a budget-and-disclose policy that degrades resolution under a budget, discloses the degradation in prose, and carries refusal codes. This is the closest existing precedent in the codebase.

## The Question

**Which instants go on the contact sheet? Pin down the selection rule.**

The destination says "the document's own visual-state boundaries", which is a direction, not a rule. Settle these six, each genuinely open:

1. **The collapse rule.** `query`'s presence set is deliberately every element, audio included (ADR-0074). The sheet needs a visual-only view. Is that a filter on `type` applied inside `frame`, a new mode on `query`, or something else? ADR-0074 settled the analogous question the other way for the cut list, and its reasoning has to be faced, not bypassed.
2. **Where in an interval the instant falls.** Midpoint, first frame, or first frame on the project's own grid (ADR-0035/0077)? A boundary instant and a settled instant answer different questions.
3. **Keyframes.** Presence can be constant while a transform animates — the fixture's photos each carry a linear zoom. Does a keyframe instant earn a tile? An earlier juror argued tween start/end and easing midpoints are also document-stated change points.
4. **Intervals too short to see.** The fixture has a 4 ms interval at 56112–56116 — a tenth of a frame at 25 fps, incapable of producing a frame of its own. What happens to it?
5. **Uniform infill.** Both successful trial agents and two earlier jurors wanted an `--n`-style flag for sampling *inside* a constant interval, for defects the document cannot predict (a source clip's own cut, motion mid-shot). It was agreed this must be a flag, never the default — uniform sampling is the blind guessing this feature exists to replace. Confirm or reject, and specify it.
6. **The blindness, stated out loud.** Any rule here is blind to some defect class. The trial proved an agent will read a sampler's silence as coverage. What exactly does the answer disclose so that does not happen — and in what form (prose, finding codes, a structured field)?

You may reject the framing outright if you think it is wrong — that is a valid ballot, not a failed one.

## Your ballot — reply with ONLY this block

🗳️ **Juror <n>** (<model>) — **VOTE: <your one-line answer: the selection rule in a sentence>**

**Reasoning:** <why — cover all six sub-questions explicitly, numbered>
**Trade-offs:** <what your rule costs; which defect classes it is blind to>
