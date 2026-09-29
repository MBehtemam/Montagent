You are a juror. Answer the four questions below from the facts given. Do not use tools, do not edit anything, do not make recommendations to the person who convened you beyond your ballot. Reply ONLY with the ballot block specified at the end.

# Background

Montagent is a tool for agents that edit a declarative video project (a JSON file of tracks and timed elements: images, text, rects, audio). A transform property (scale, x, y, opacity…) may be animated by a list of **keyframes**, each `{t, v, ease}`: at timeline millisecond `t` the property's value is exactly `v`, and between two keyframes the value is interpolated by the later one's `ease` (`linear`, `step`, and curves). A keyframe may legally sit outside its element's lifetime (a "trimmed move"). Verbs include `query --at <MS>` (the resolved stack at an instant) and `frame --at <MS>` (the still the renderer would emit there). The project has a frame grid at `fps`; frame `n` is painted at millisecond `⌊n × 1000 / fps⌋` and shows the document evaluated at that millisecond. A millisecond between two painted ones is never on screen.

`frame` is gaining a range mode (`--from`/`--to`) that returns ONE contact sheet. Already ratified:

- **Run tiles (ADR-0094 §1–2).** The range is cut into *visual states* (runs over which the set of visible elements is constant). Each run `[s, e)` gets one tile, sampled at **the first frame the grid paints inside it**: least `n` with `⌊n·1000/fps⌋ ∈ [s, e)`. Not the midpoint — "a midpoint is synthetic, so no other verb can reproduce the tile"; every tile's instant must be one `frame --at` / `query --at` reproduces exactly. A run with no painted frame gets no tile and is listed in `skipped[]` as `no-grid-frame`, and raises a finding (`N-QUANTIZATION` at `review`) because "the document declares it and the rendered video never shows it". Easing midpoints get no tile, 3–0: "an easing midpoint is derived from the curve, not stated by the document. Inventing instants is the guessing this feature exists to replace."
- **Keyframe tiles (just decided).** An opt-in `--keyframes` flag adds tiles at keyframe change points. The population is **keyframes interior to a run, on an element visible at that keyframe**: keyframes on a run boundary are already covered by the run tile, and keyframes outside their element's lifetime are not visible change. Every answer reports two counts, `tiled` and `untiled`, flag or no flag, so the zero is asserted. Keyframe tiles are document-derived and count toward the width budget; too many refuses the call (`E-SHEET-OVERFLOW`) — they are never silently dropped.
- **Tile classes and label (ADR-0098).** Three classes: document-derived run tiles (unmarked), `keyframe`, `infill` (uniform samples, opt-in). Keyframe and infill tiles are marked visually and by a class token on their provenance line. The label is `<index> <sigil> <instant>ms <±offset> <±id>`; the identifying field names what changed at the run's own boundary, and keyframe/infill tiles (which have no boundary) carry none — their provenance line names e.g. `photo-05.scale`. The provenance list is "the census"; the label is "a pointer".
- **Blind spots are disclosed as a fixed rule-level list**, never findings; one is `between-keyframes`: "Keyframed values are shown only where a tile falls, and keyframe instants are untiled unless asked for; a wrong easing curve shows only if its endpoints are wrong." ADR-0103: a blind spot is safe to disclose when it is *constant and learnable*.
- **Measured (#407).** On a constructed project, keyframe tiles at 138 px caught a 1.85× scale **overshoot** peaking at a mid-run keyframe (face cropped), and a text block that drifts off canvas **only between** its endpoints. They did not catch an 8 % `step` snap (1.0 → 1.08): under two pixels at tile size. "The tile sees amplitude, not shape." On the repo's one real project (25 fps) there are 14 declared keyframe change points and 0 in the population.

## What nobody has decided

ADR-0094 §2 fixed which frame a *run* is sampled at. Nothing fixes which frame a *keyframe change point* is sampled at. A keyframe's `t` is an authored millisecond and is usually off-grid: at 25 fps frames paint at …, 40, 80, 120, …, so a keyframe at `t = 1013` is never itself on screen; the nearest painted frames are 1000 (before) and 1040 (after). At 1000 the value is still approaching `v`; at 1040 it has already moved `27 ms` into the next segment toward the following keyframe. For a `step` ease, 1000 shows the old value and 1040 the new one.

# The questions

**Q1 — Which painted frame is a keyframe change point at `t`, inside run `[s, e)`, sampled at?**
(a) The first painted frame at or after `t`: least `n` with `⌊n·1000/fps⌋ ≥ t`, provided it is `< e` — the same shape as the run rule.
(b) The painted frame nearest to `t`, ties to the earlier (or later — say which).
(c) The last painted frame at or before `t`.
(d) Something else, or reject the premise.

**Q2 — Coincidence.** Several change points can map to one painted frame: two properties keyframed at the same `t`, two elements keyframed a few ms apart, or a keyframe a few ms after `s` whose sample frame *is* the run tile's frame.
(a) One tile per distinct painted frame. Its provenance line lists every change point that mapped to it; all count as `tiled`. If the frame is a run tile's, the tile keeps the run-tile class (unmarked, with its boundary field) and the change points are listed on its provenance line, adding no tile.
(b) As (a), but a run tile that absorbs a keyframe change point takes the `keyframe` mark.
(c) Something else.
Also: does the population definition become grid-aware — i.e. a change point whose sample frame is the run tile's frame is *not* counted as `untiled` even without the flag, because the sheet already shows it — or does "interior" stay a raw-millisecond test (`s < t < e`)?

**Q3 — A change point with no painted frame to sample** under your Q1 answer (e.g. under Q1(a), `t` lies in the run's last partial frame period, so the first painted frame ≥ `t` belongs to the next run).
(a) It is counted `untiled` with a reason (e.g. `no-grid-frame`), disclosure only, no finding — the value was never meant to be held on screen, and off-grid keyframes are ordinary.
(b) Fall back to the nearest painted frame inside the run on the other side.
(c) It counts as `tiled` by the next run's tile if the element is still visible there, else as (a).
(d) It raises a finding, as an unpainted run does.
(e) Something else.

**Q4 — Is this sample instant, and not an easing midpoint or a peak, sufficient for what the flag is for?** The overshoot defect #407 caught peaked *at* a keyframe; the off-canvas drift was visible at its keyframe. Should the rule stay "change points only", or does anything here reopen ADR-0094's 3–0 refusal of curve-derived instants (e.g. the extremum of an overshooting curve)? Answer: stays / reopens, and why.

# Ballot

Reply with exactly this block and nothing else. You may reject a question's framing outright; that is a valid answer.

```
🗳️ **Juror <n>** (<the model backing you>) — **VOTE: Q1 … ; Q2 … ; Q3 … ; Q4 …**

**Reasoning:** <why, per question>
**Trade-offs:** <what each choice costs, or why not the other options>
```
