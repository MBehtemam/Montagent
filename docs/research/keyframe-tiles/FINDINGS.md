# Does a keyframe tile catch anything the run-start tile misses?

Prototype for [#407](https://github.com/MBehtemam/Montagent/issues/407), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). It tests the one thing
[ADR-0094](../../adr/0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md)
decided on an absence of evidence and said so: **keyframes are excluded from the sheet's
default instant set and put behind a flag.**

**The verdict: the default does not flip.** But three of the things this measured are not
what either side of the jury's split was arguing about, and two of them are corrections to
clauses already ratified.

---

## 1. The fixture the ticket names has no keyframe population at all

The ticket says *"its photos each carry a linear zoom, so it has the keyframe population
this question is about."* That is false in the way that matters, and it is checkable:

| `fixtures/en-halloween-decorating` | count |
| --- | --- |
| keyframe change points declared | **14** |
| …sitting exactly on a run boundary the run-start rule **already tiles** | **7** |
| …lying **outside the lifetime of the element that declares them** | **7** |
| …**interior to a run, on a visible element** — the only class a keyframe tile can add | **0** |

The seven trimmed ones are not an accident and not a fixture bug. Every photo carries a
15 000 ms Ken Burns ramp that its own segment cuts short, the second keyframe landing past
`end`; the schema documents that spelling in `Keyframe.t` itself — *"Legal outside the
element's own range — that is how a trimmed move is spelled, and seven of the fixture's
photo elements carry one"* — and the fixture README records the ramp's amplitude and timing
as verified against the published video ([#276](https://github.com/MBehtemam/Montagent/issues/276)).

So on the one real project in this repo, **`--keyframes` adds zero tiles and costs zero
pixels**, and the two sheets the ticket asks for are byte-for-byte the same sheet. The jury
argued the trade on a fixture where the trade does not exist.

`classify_keyframes.py` re-derives all four numbers and exits non-zero if any moves.

### The correction this forces on ADR-0094 §3

> *"The count of untiled keyframe change points is named in every answer regardless."*

Read literally against the document's keyframe list, that clause prints **14** on this
fixture, where the honest count of omissions is **0**. Seven of those fourteen are on the
sheet already and seven describe motion that is never painted.

That is this map's founding failure running in the opposite direction. The trial agent's
sampler was silent where it was blind, and silence read as coverage. A disclosure that
reports fourteen omissions where there are none is **noise read as a gap** — and a number
that is routinely wrong by its whole magnitude is a number the reader learns to skip, which
costs the disclosure the credibility that ADR-0094 §6 spends unconditionally to buy.

**The counted set must be keyframes interior to a run on an element visible at that
keyframe** — not keyframes declared.

---

## 2. What a keyframe tile actually catches

Since the fixture has no interior population, one was authored. `make_kf_project.py` writes
`doctored/`, which carries four constructed keyframe defects, the map's own two
presence-level defects (as a loss control), and seven decorative interior keyframes on the
photos that host nothing — so the tile-count cost is a real animated project's cost, not the
cost of the defects alone. It re-checks at **0 errors** (one `layout`, formatting only).

| | defect | run-start tile | keyframe tile at 138 px |
| --- | --- | --- | --- |
| **D1** | `photo-06` scale overshoots to 1.85 mid-run and returns — crops the face | clean | **caught, unmistakable** |
| **D2** | `sentence-07` drifts to `x: 2000` mid-run and returns — off canvas | clean | **caught** |
| **D3** | `photo-05` scale snaps 1.0 → 1.08 on an `ease: "step"` | clean | **not caught** |
| **D4** | `photo-08` honest linear ramp 1.0 → 1.08 — *the control, not a defect* | clean | **nothing**, correctly |
| D5 | `photo-07.source` → `images/06.png` (the map's) | caught | kept |
| D6 | `sentence-08.color` → its card (the map's) | caught | kept |
| #404 | the fixture's doubled `word-08` caption | caught at 184 px | **lost at 138 px** |

**So the answer to the ticket's question is yes, for two of four constructed classes** —
and both survivors are the same shape: a **large-area geometric** change (a 1.85× crop, a
text block vacating its card). Both are the classes [#396](https://github.com/MBehtemam/Montagent/issues/396)
measured as surviving to 92 px. Neither needs fine detail to read.

**D3 is the negative result and it is the informative one.** An 8 % scale step is a
*discontinuity in the motion* and a tile cannot show it, because a tile has no neighbour in
time — the keyframe tile and the run-start tile differ by 8 % of linear scale and at 138 px
that is under two pixels. **Juror 3's claim is confirmed, and it generalises further than he
put it**: a keyframe tile is blind not only to a *linear* tween but to any keyframe whose
value change is small, whatever its easing. What it sees is amplitude, not shape.

### D2 is the class nothing else in the tool reaches

`validate` reports **0 errors** on the doctored file, and that is not an oversight.
[ADR-0044](../../adr/0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md)
decision 2 fires `R-OFF-CANVAS` only when an element's rect **never** intersects the frame
across its whole range, and says why:

> An element that is off-canvas at *some* instants of its active range — a slide-in starting
> at `x:-500` and animating to `x:0`, a slide-out doing the reverse — is ordinary, legal
> animation vocabulary and fires nothing.

An element that is off canvas *only between its endpoints* is therefore, by an accepted
decision, invisible to `validate` **by design**. The keyframe tile is the only view in this
tool that reaches it. That is a real argument for the flag existing. It is not an argument
for the default, because of what section 3 costs.

---

## 3. The cost, and the part nobody costed: the flag can make the call refuse

Three sheets, same document, same range, same one-image budget, composed at served size
(#396's method — the file in `sheets/` **is** the file that was judged):

| sheet | tiles | grid | served | tokens | tile width | smallest label type |
| --- | --- | --- | --- | --- | --- | --- |
| `S1-run-start-18.png` | 18 | 6×3 | 1107×1092 | 1560 | **184 px** | 11.3 px |
| `S2-keyframe-22.png` | 22 | 8×3 | 1287×952 | 1564 | 161 px | 9.8 px |
| `S3-keyframe-31.png` | 31 | 8×4 | 1107×1092 | 1560 | **138 px** | 7.9 px |

S2 is a diagnostic, not a policy — it adds only the four keyframes that host a defect, which
is the best case a keyframe tile could ever have and which no implementation can select,
because the tool cannot know which keyframes host defects. **S3 is what the flag gives you.**

And S3 is **not a worse sheet. It is not a sheet at all**:

- 138.4 px is **below [ADR-0095](../../adr/0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md)'s
  140 px floor**, so the range mode **refuses**.
- 7.9 px of served type is **below ADR-0098 decision 4's 8 px floor**, so the label refuses
  independently.

The width sweep reproduces ADR-0095's own cliff exactly — **30 tiles at 141.9 px is the last
admissible count on a 9:16 frame**. So:

> **On an 18-visual-state document, the keyframe budget is 12 keyframe tiles.** The
> thirteenth turns a working call into a refusal.

That is a stronger cost than the jury's *"roughly doubles tile count"*. Doubling is a
legibility trade the caller can weigh. Crossing the floor is a **failure with no degraded
mode**, because ADR-0095 already decided overflow may not thin, split or reshape. A caller
who passes `--keyframes` on an animated project gets nothing back.

### What the flag trades, stated plainly

At 184 px → 138 px the sheet **gains** two large-area geometric classes and **loses** the
fine-detail class (#404's doubled caption, which #396 bracketed at 138–148 px and which dies
here exactly as predicted). It is not a strict gain in either direction.

---

## 4. A spec gap: the keyframe tile has no identifying field

ADR-0098 decision 8 defines the label's identifying field as *"what changed at this run's own
boundary"*. **A keyframe tile has no boundary change** — that is the whole point of it — so
the clause does not define its label. This prototype used `element.property`
(`photo-06.scale`), which is the natural analogue and is what the sigil'd labels in
`sheets/` carry.

It is materially longer than a boundary field: `+word-08-bridge` is 15 characters,
`photo-05-loop.scale` is 19, and on S3 the keyframe labels are the ones that overrun
(`6 ◆ 12000ms +1532 photo-05.sc`, `29 ◆ 62000ms +884 photo-05-c`). Because ADR-0098
decision 5 makes elision **sheet-wide**, a keyframe tile whose id will not fit at the floor
strips the identifying field from **every** tile on the sheet. So keyframe tiles push a sheet
into elision earlier than tile count alone predicts, and that belongs to
[#418](https://github.com/MBehtemam/Montagent/issues/418) along with the flag's spelling.

---

## 5. Verdict

**ADR-0094's default does not flip, and its "decided on an absence of evidence" caveat can be
retired** — it is now decided on evidence, and the evidence points the same way for reasons
the jury did not have:

1. On the repo's one real project the flag adds **zero** tiles, so the default costs nothing
   and buys nothing there. Juror 1's *"goes silent across the fixture's zooming photos"* is
   not true of this fixture: there is nothing at those keyframes to go silent about.
2. Where an interior population does exist, the flag catches **large-area geometric** defects
   and only those, loses the fine-detail class, and **past 12 keyframe tiles on an 18-state
   document it refuses outright**. A default that can turn a working call into a refusal is
   not a default.
3. Juror 3's linear-tween argument is confirmed and is broader than he stated: the tile sees
   **amplitude, not shape**. Easing defects are out of reach of this rule entirely.

Two corrections fall out, and both are for [#418](https://github.com/MBehtemam/Montagent/issues/418):

- **The disclosed count is interior-on-visible keyframes, not declared keyframes** (§1). On
  the committed fixture that is 0, not 14.
- **The flag needs the width floor checked before it is honoured** (§3), and the answer
  should say how many keyframe tiles would fit — ADR-0095's *"naming sub-ranges that would
  fit"* has an exact analogue here, and a refusal that says *"12 of 13 keyframe tiles fit"*
  is worth more than one that says the sheet is too small.

**One thing this does not decide**, and it is a real option the measurement suggests rather
than settles: a **width-conditional default** — include keyframes whenever the resulting
sheet still clears 140 px, exclude them otherwise. It would make the flag free on the
fixture and on any lightly-animated project, and never reach the refusal. It is a new shape
for the knob rather than a choice between the two the jury debated, so it goes to #418 as an
option, not as this ticket's verdict.

---

## Rebuilding

`tiles/` (31 frames, 63 MB) and `sheets/` inputs are derived and `tiles/` is not committed.

```sh
cargo build --release
cd docs/research/keyframe-tiles
python3 make_kf_project.py          # writes doctored/
python3 classify_keyframes.py       # the three numeric claims; writes instants.json
for t in $(python3 -c "import json;print(' '.join(map(str,json.load(open('instants.json'))['union'])))"); do
    ../../../target/release/montagent frame doctored/en-halloween-decorating.montagent.json \
        --at $t --full --png --out tiles/$t.png
done
python3 make_sheets.py              # writes sheets/
```

`classify_keyframes.py` is the re-executable check `docs/agents/domain.md` requires for the
numeric claims (§1's four counts, §3's tile counts and the 140 px cliff). The qualitative
claims — which defect each sheet shows — rest on the three sheets in `sheets/`, committed
verbatim at the size they were judged at, and on one observer who knew the defects, the same
standing caveat #396 and ADR-0095 carry.
