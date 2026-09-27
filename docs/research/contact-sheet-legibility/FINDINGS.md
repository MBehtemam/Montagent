# The legibility limit of a contact sheet

Prototype for [#396](https://github.com/MBehtemam/Montagent/issues/396), on the
map [#395](https://github.com/MBehtemam/Montagent/issues/395).

**The question.** How many tiles can one contact sheet carry before it stops
being able to show a defect — and what does that make the tile scale?

**The short answer.** Tile *count* is the wrong knob. The thing that decides
whether a defect survives is the **served tile width in pixels**, and three
different classes of defect die at three very different widths:

| defect class | example in this fixture | dies at | i.e. tiles at one image budget |
| --- | --- | --- | --- |
| fine detail (small type, ~35–49 px in a 1080-wide frame) | the doubled `word-08` caption ([#404](https://github.com/MBehtemam/Montagent/issues/404)) | **~140 px** tile width | **~30** |
| presence/absence of text on a card | `sentence-08` recoloured to its card | survives **92 px** | **72+** |
| large-area substitution | `photo-07` pointed at `images/06.png` | survives **92 px** | **72+** |
| the per-tile label, fitted to tile width | `9 42763ms item-08 word` | survives **92 px** (6.3 px type) | **72+** |

So **~30 tiles is the ceiling and ~18 is the honest working number** — see
*The number to actually use* below for why the recommendation sits below the
measured cliff.

## What was measured, and how

A copy of `fixtures/en-halloween-decorating` (1080×1920, 25 fps, 65.2 s) was
doctored with the map's two planted defects — `photo-07.source` →
`images/06.png`, `sentence-08.color` → `#1E344C` — and re-checked at **0
errors**, as the map claims. The fixture's own third defect, the doubled
`word-08` caption (#404), came for free and turned out to be the most
informative of the three.

The 18 visual-state instants are the same set the sibling ticket
[#397](https://github.com/MBehtemam/Montagent/issues/397) used, so the two
results compose.

**Sheets were composed at the dimensions a model is actually served.** This is
the methodological point that makes the numbers mean anything: an image is
downscaled *by the API* to fit the resolution tier before the model ever sees
it, so a sheet authored at 6480×6393 is looked at as 1107×1092. Every sheet in
`sheets/` was resized to its served size on disk, so the artifact committed here
**is** the artifact that was judged. Measuring legibility on the authored image
would have overstated every threshold by 5–6×.

Budget throughout is **one standard-tier image, ≤1568 visual tokens** — the cap
a single image hits on its own. Every sheet in the sweep spends 1530–1568 of it.

## Grid shape: near-square, and never a strip

A 9:16 frame does tile badly, but not in the way the ticket guessed. The failure
is not aesthetic — it is that the API caps the **long edge at 1568 px** as well
as the token count, so an elongated sheet is throttled on one axis and cannot
spend its budget:

| n=18 layout | served sheet | tokens | served tile | verdict |
| --- | --- | --- | --- | --- |
| **6×3 grid** | 1107×1092 | **1560** | **184×328** | best |
| 9×2 grid | 1568×688 | 1400 | 174×310 | slightly worse |
| 3×6 grid | 397×1568 | 840 | 132×235 | throttled |
| 18×1 horizontal strip | 1568×172 | 392 | 87×155 | `sheets/B-strip-h.jpg` |
| 1×18 vertical strip | 44×1568 | 112 | 44×78 | unusable |

**A strip is strictly dominated**: at the same tile count it gives tiles 2.1×
(horizontal) to 4.2× (vertical) narrower *while leaving three quarters of the
budget unspent*. There is no budget at which a strip is the right shape, so
"one row per group" should not be an option the feature offers.

The rule that falls out: **pick the (cols, rows) whose resulting sheet is
closest to square.** For 9:16 tiles that means roughly twice as many columns as
rows (6×3, 8×4, 12×6), and those layouts land on 1:1.01 sheet aspect and
saturate the budget exactly.

## Where each defect dies

Sheets `A-count-18` → `A-count-28` → `A-count-32`, and `E-long-48` /
`E-long-72`.

- **The wrong photo survives everything tested.** At 72 tiles / 92 px it is
  still unmistakable: tiles captioned `skeleton - skeleton` carry the same
  spider picture as the tiles captioned `spider - spider`. Worth stating plainly
  — **this defect is invisible in any single frame** and only exists as a
  relation between tiles. The sheet does not merely make it cheaper to find; it
  is the only view in which it is a defect at all.
- **The invisible sentence survives everything tested.** At 92 px the dark card
  with no text on it reads instantly against the neighbouring cards that have
  text. It survives because the defect is a *large uniform area*, which is
  exactly what downscaling preserves.
- **The doubled caption is the one that dies, at ~30 tiles.** At 24 tiles
  (161 px) "string of lights / string of lights" reads as two lines of the same
  words. At 28 (148 px) it is marginal — two lines are visible, reading them as
  identical is a guess. At 32 (138 px) the second line is a smudge. Bracketed
  at **28–32 tiles, ~138–148 px**; call it **~30 tiles / ~140 px**.

## Labels: they outlive the pictures, if they are fitted

The first attempt drew the label at a fixed size and it **overflowed every tile
and overprinted its neighbours** — see `sheets/A-count-18.jpg`, where only the
leftmost label of each row is readable. That is worth recording because it is
the obvious implementation and it destroys the sheet.

Sized to fit the tile width instead (`sheets/E-long-*.jpg`), the full label
`9 42763ms item-08 word` stays readable down to **6.3 px of served type at 92 px
tiles / 72 tiles per sheet** — well past the point the pictures give up.

**The label is therefore not the binding constraint, and the budget should not
be set by it.** Two consequences for [#400](https://github.com/MBehtemam/Montagent/issues/400):
the label must be fitted to the tile width rather than set at a fixed size, and
it can afford to carry instant + group + what-produced-it rather than a bare
index.

## Cropping every tile: the highest-value mode, and the most dangerous

`sheets/C-crop-band-18.jpg` crops all 18 tiles to the caption/sentence band
(`0,1300,1080,360`) at the same 1564-token budget.

- The band's 3:1 tile grids far better than 9:16. The same 18 tiles go from
  184 px wide to **429 px wide — a 2.33× linear gain for the same cost.**
- **The defect that dies at 30 whole frames is trivially legible here**, and
  still trivially legible at **36 cropped tiles** (`C-crop-band-36.jpg`, 299 px),
  where the whole-frame sheet lost it long before.
- **And the wrong photo becomes completely undetectable**, because the band
  excludes the photo. The sheet looks clean and complete and is blind.

That last point is the load-bearing one, and it is the map's own "structurally
blind sampler" failure in a new place: a cropped sheet is a *strictly narrower
instrument* that advertises nothing about what it dropped. If a per-tile crop
mode ships, the disclosure is not optional decoration — it is the only thing
standing between the mode and the exact failure this map was built around.

**#402 blocks this mode and reproduces exactly as filed.** `--crop` without
`--full` returns a 1080×360 ask as 540×180. A sheet built on that would
downscale each tile twice — once by `--crop`, once by the grid — and the 2.33×
gain above is precisely what that would throw away. These sheets were built by
cropping `--full --png` output externally, which is the same workaround the
agent in #402 was forced into.

## The high-resolution tier does not change the answer

On the Claude 4.7+ tier the same 6×3 sheet is served at 1932×1906 with 322 px
tiles — but it costs **4761 tokens, 3.05× the standard-tier sheet**. It buys
resolution, not tiles. Any tile-count policy has to hold on the standard tier,
where the sheet is served at ~1.15 MP whatever it was authored at.

## The number to actually use

The measured cliff is ~30 tiles. **The recommendation is ~18**, for two reasons:

1. **The observer was primed.** These verdicts are one model's readings, by a
   session that knew what all three defects were and where. That is an upper
   bound on legibility, not a floor — an agent told only "something looks wrong"
   will do worse, which is the exact scenario the map's trial ran.
2. **18 sits at 184 px, a 31% margin over the 140 px cliff**, and is the same
   count the map's visual-state collapse produces for a 65-second video anyway.

Stated as a policy the feature could implement: **budget in served tile width,
not tile count.** Hold tiles at ≥180 px wide; derive the count from that and the
sheet's near-square grid; disclose the count, the tile scale and anything
dropped. A sheet that has fallen below ~140 px should say so, because past that
point it is silently no longer able to show a whole class of defect.

## Reproducing

```sh
python3 check_contact_sheet_geometry.py                 # geometry + cost claims
MONTAGENT=target/release/montagent PROJECT=<doctored>.montagent.json \
  python3 check_contact_sheet_geometry.py               # also re-checks #402
```

Per `docs/agents/domain.md` the two kinds of claim are tiered differently:

- **Numeric claims** — every served tile size, token cost, the strip-dominance
  result, the near-square rule, the 2.33× crop gain, the #402 reproduction — are
  re-derived by `check_contact_sheet_geometry.py`, which exits non-zero the
  moment any of them stops holding. All 33 pass as committed.
- **Visibility verdicts** cannot be re-derived by a script, so the sheets that
  were judged are committed verbatim in `sheets/`. They are JPEG q95 4:4:4
  rather than PNG for repo weight (6 MB against 14 MB); `make_sheets.py` and
  `label_test.py` regenerate the exact PNGs that were looked at.

`make_sheets.py` and `label_test.py` are the throwaway prototype itself, kept as
the primary source. They expect the 18 rendered frames in `tiles/` and the
doctored project alongside; both are rebuilt by the recipe at the top of
`make_sheets.py`.
