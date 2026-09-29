# Brief, as every juror received it

Sent verbatim and identical to all three jurors. The only difference between the three
dispatches was the juror number in the output block (`Juror 1` / `Juror 2` / `Juror 3`).
Each was told to answer from its own judgement, to use no tools, and to emit the ballot block
and nothing else.

---

**Context.** Montagent is a declarative video-editing tool driven by an AI agent: a JSON project document describes elements (text, photos, audio) placed on tracks over time, and verbs render or inspect it. The verb `frame --at <ms>` rasterizes one instant and returns the picture, so an agent can see what the video looks like.

A feature effort is adding a **range mode** to `frame`: `frame --from A --to B` returns ONE labelled **contact sheet** — a grid of tiles, each tile a still frame from one "visual state" of the document, so the agent sees a whole span in one call instead of N calls. Its stated destination: *"an agent asks `frame` for a range and gets back a single tiled image it can actually look at, each tile labelled with its instant and what produced it, with the selection rule, the tile scale and anything skipped disclosed in the answer — fast enough to use after every edit, and honest enough that 'I checked the whole thing' is an artifact rather than an argument."*

**The founding experiment.** A real 65-second project fixture was doctored with three defects that the `validate` verb passes at **0 errors**, and three AI agents on three different models were handed the tool and told only "something looks wrong":
- `photo-07.source` was repointed to the wrong image file (visible in the JSON, and on screen as a wrong photo).
- `sentence-08.color` was set to `#1E344C`, identical to its background card — so the text is **invisible in pixels only**; nothing in the JSON looks wrong.
- (A third, `word-08`, a caption accidentally doubled/overprinted — a fine-detail text defect.)

The load-bearing result: the agent that MISSED the pixel-only defect hand-rolled a sampler over the document every 200 ms, checked text bounding-box geometry, and claimed **full visual coverage**. Bounding-box geometry cannot see colour — invisible text has a perfectly correct bounding box. **Its sampler was structurally blind and its silence read as coverage.** The whole effort is organised around the principle that any selection rule can fail this way, and that **disclosure is the only defence**.

**Already decided by this effort (ratified, mostly by earlier juries):**
- A tile is one **visual state**, sampled at the first frame actually painted in it. Every answer carries an **unconditional structured disclosure**: the selection rule, per-tile provenance, what was skipped and why, and a fixed **`blind_to`** enumeration of what this rule cannot see — emitted on every answer including perfect ones, because *"a caveat that appears only when something went wrong teaches the reader that its absence is an all-clear."* A sharp taxonomy was drawn: **`blind_to` describes the rule, `skipped` describes the document.** A blind spot earns no "finding" code, because a finding says something about *this document* while blindness is a property of the *rule*.
- The sheet's budget is **served tile width**: a **180 px target**, a **140 px floor**, and a **refusal** past it. Tile count is *derived* from the width, never set by the caller. Overflow **degrades once then refuses** — it may never thin, split, or reshape the sheet. (Splitting is forbidden partly because >20 image blocks in one API request silently clamps every image in the request.)
- Tokens turn out to be a useless currency here: a sheet spends 1518–1568 of the model's 1568-token cap at *every* tile count from 4 to 48, so a token budget never fires. Legibility, not cost, is the whole game.

**The open question — a per-tile crop.** `frame` already has a `--crop x,y,w,h` flag for single instants. Should the range mode crop **every tile** to the same region?

**The measured case FOR.** At the same ~1564-token cost, cropping all 18 tiles to a caption band gives **429 px tiles against 184 px — 2.33× the linear scale**. The fine-detail `word-08` defect, which becomes illegible at ~30 whole frames, is still *trivially* legible at **36 cropped tiles** (299 px). A 3:1 band also grids far closer to square than a 9:16 frame, wasting less of the sheet. The research called it *"the highest-value mode in the feature."* Roughly 5× as many tiles fit in a band as in whole frames at the same target width.

**The measured case AGAINST.** On the same cropped sheet, the planted `photo-07` defect became **completely undetectable**, because the band excludes the photo. The research wrote: *"The sheet looks clean and complete and is blind… a cropped sheet is a strictly narrower instrument that advertises nothing about what it dropped."* And separately, about why the sheet exists at all: the wrong-photo defect *"is invisible in any single frame and only exists as a relation between tiles. The sheet does not merely make it cheaper to find; it is the only view in which it is a defect at all."* A crop removes precisely that capability. Unlike the whole-frame sheet's blindness, this blindness is **chosen by the caller** and invisible in the result.

**Further facts that bear on it, established since the question was filed:**
1. **The evidence base is one eyeballed rectangle.** The band was a hardcoded constant `(0, 1300, 1080, 360)` over the caption area, written by hand, never computed from any element's geometry. **No alternative region was ever measured** — no photo band, no centre crop, no auto-derived region. And the tally on the cropped sheets is: `word-08` confirmed legible, `photo-07` confirmed annihilated, **`sentence-08` never re-tested on a cropped sheet at all**. (Whole-frame, `sentence-08` survived down to 92 px — the smallest width ever tested — because it is a large uniform area, which downscaling preserves.)
2. **A single-instant crop is now lossless and already available.** A just-landed decision makes `frame --crop` return the region at **true scale** (it previously half-scaled it silently). So the two-step workflow "whole-frame sheet to find the suspicious instant, then `frame --crop --at <that instant>` to look closely" works today with no new specification.
3. **Disclosure was just measured failing on this very flag.** When `frame --crop` was half-scaling, the failure was *not* silent — the answer printed `half scale, 492x170 from a 1080x1920 frame` plus the region line. A reporting agent read past both and filed it as silent, because the tool's *documentation* promised true scale. The finding was written up as: **"A promise in the tool description outranked a fact in the answer."**
4. **A caller-chosen band does not fit the existing disclosure taxonomy.** `blind_to` describes the rule; `skipped` describes the document. A region the caller typed is a property of *neither* — it is a property of this call's arguments. Shipping it requires a third disclosure category.
5. **The document model has no region vocabulary to lean on.** There is no safe-area concept, no named regions, no normalized rects, and no mapping from a track to a rectangle. There ARE seven document-derived rectangles that could in principle compose into a band (authored clip and mask rects, derived drawn and visible rects, an uncovered-area partition, text ink boxes, and an off-canvas union) — but a text ink box **refuses** to compute on non-zero rotation, on non-unit scale, and on any right-to-left text run, so a derived band would inherit a refusal surface. No derived band has ever been built or measured.
6. **The fine-detail class has other channels.** `word-08` is a *text* defect; the tool separately exposes text measurement and text ink-box extents from the document alone, without pixels, and a factual `validate` check for it is plausible. The pixel-only colour defect (`sentence-08`) has no non-pixel channel at all — it is the reason the visual channel exists.
7. The flag's spelling is already fixed by an earlier decision: if it ships it is the existing `--crop` composed with the existing range, never a new flag name. Today the combination is **invocable with no defined answer**, which is why it must be either shipped or explicitly refused — leaving it undefined is considered worse than either.

**Your Question — choose one:**

- **(A) Ship the per-tile crop as a caller-typed region**, with a mandatory disclosure naming the region kept and stating that everything outside it was not looked at.
- **(B) Ship a per-tile crop, but only as a region *derived from the document*** (e.g. the union of the rects of every element that changed), so it cannot exclude a changed element by construction — never a caller-typed rectangle.
- **(C) Refuse `--crop` composed with a range**, permanently for this effort, and rule the per-tile crop out of scope. The agent keeps the two-step loop: whole-frame sheet to locate the instant, single-instant `frame --crop` to look closely.
- **(D) Something else** — including rejecting this framing if you think all three options are wrong.

If you vote A or B, your ballot must also say **what the disclosure has to be**: where it lives (the structured field, the prose, or both), whether it needs a machine-readable code, and how it avoids the failure in fact 3 above — an accurate disclosure that the reading agent skipped because the documentation had promised otherwise.
