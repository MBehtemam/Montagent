# Both perceptual floors, re-measured cold

Prototype for [#422](https://github.com/MBehtemam/Montagent/issues/422), on the
map [#395](https://github.com/MBehtemam/Montagent/issues/395).

**The question.** ADR-0095's 140 px served tile width floor and ADR-0098's 8 px
served type floor both rest on #396's single observer, and that observer knew
all three defects before looking. Do the floors survive observers who do not?

**The short answer.** Both floors survive, and neither moves up. The danger the
ticket named (true floor above 140 px, so sheets between the two look complete
while unreadable) does not appear for Sonnet or Opus. The result that matters
more is one nobody asked about: **the floors are conditional on the reader.**
Haiku 4.5 found none of the three defects at any width, *including the 184 px
target*. It invented defects that are not there, and it never found the label
strip at all.

## Method

- **42 cold readings.** Every reading was a fresh subagent that saw exactly one
  image. The prompt named no defect and gave no defect list. It said only that
  "the video's author says something about it looks wrong". The observers were
  told to read that one file and nothing else. Each condition was read by three
  models: Haiku 4.5, Sonnet and Opus.
- **Picture sheets isolate width.** #396's sheets past 18 tiles repeat
  instants (`(INSTANTS*2)[:n]`), so a cold observer sees every tile twice and
  could fairly call that the defect. Here **one** 18-tile 6×3 sheet of the
  doctored fixture (the map's two planted defects, plus #404's doubled caption)
  was downscaled so its tiles are served at 184, 160, 148, 140, 130, 120 and
  92 px. Content is identical at every width, so only width varies.
- **Control.** The same sheet built from the *undoctored* fixture at 184 px. It
  has no wrong photo and no invisible sentence, and still has #404's doubled
  caption. It measures false alarms.
- **Label sheets use transcription, not recognition.** 18 random but realistic
  ADR-0098 labels (`<index> <instant>ms <±offset> <±id>`, with ids drawn from the
  fixture) sit under 140 px tiles at fixed served type sizes of 5, 6, 7, 8 and
  10 px. The observer transcribes all 18 without knowing the strings. The
  score is exact-match against `manifest.json`, so recognising a known string
  cannot inflate it.

## Picture floor: 140 px holds

Strict scoring. The **wrong photo** is a hit if tiles 7–8 are named as the
skeleton word over the spider picture. The **invisible sentence** is a hit if
tile 10's empty bar is named. The **doubled caption** is a hit only if the
observer says tile 9's second line repeats the first. Noticing a smaller
second line without reading it is a partial.

| served tile | Sonnet (photo / sentence / caption) | Opus | Haiku |
| --- | --- | --- | --- |
| 184 px (target) | hit / hit / hit | hit / hit / partial | miss / miss / miss |
| 160 | hit / hit / hit | hit / hit / partial | miss / miss / miss |
| 148 | hit / hit / hit | hit / hit / hit | miss / miss / miss |
| **140 (floor)** | hit / hit / hit | hit / hit / hit | miss / miss / miss |
| 130 | hit / hit / hit | hit / hit / hit | miss / miss / miss |
| 120 | hit / hit / hit | hit / hit / hit | partial / miss / miss |
| 92 | **miss** / hit / miss | hit / hit / hit | miss / miss / miss |
| control, 184 | no false photo or sentence; caption hit | no false alarm; saw the real skeleton | reported only invented problems |

- **Fine detail, the class that set 140, does not die above 140 cold.** Sonnet
  read the doubled caption at every width down to 120. Opus read it down to 92.
  #396's primed observer put the cliff at ~138–148 px. The cold readers did
  *at least as well*, so **"primed means an upper bound" was not borne out
  here.** The primed number was the conservative one. A likely confound is that
  #396's sheets past 18 tiles contained duplicate tiles, so its narrower sheets
  held different content, not only smaller tiles.
- **The first Sonnet/Opus loss is the wrong photo at 92 px** (Sonnet). That is
  the class #396 said survives *everything*. One reading is not a cliff, but it
  shows the relational defect is not immune to width.
- **Two Opus partials sit at the *wider* widths (184, 160).** At those widths
  Opus described two stacked lines without saying they matched. That is noise
  at n = 1 per cell, not a trend. It argues against reading any single cell as
  a threshold.
- **The control shows the hits are signal.** On the clean sheet neither Sonnet
  nor Opus reported a wrong photo or an empty bar, and Opus correctly saw a
  skeleton in tile 8.

**Verdict.** No evidence that the floor should rise. There is weak evidence it
could fall to ~120 px for Sonnet-class readers or better. **It should not
fall**: the evidence is one reading per cell, and 140 px is also what bounds
wall clock (ADR-0095).

## Type floor: 8 px holds with ~2 px of margin

| served type | Sonnet exact | Opus exact | Haiku |
| --- | --- | --- | --- |
| 10 px | 18/18 | 18/18 | transcribed the in-frame captions, not the labels |
| **8 px (floor)** | 18/18 | 18/18 | same |
| 7 px | 17/18 (unsure) | 18/18 | same |
| 6 px | 18/18 | 18/18 (unsure) | same |
| 5 px | 17/18, **confident** | 12/18 (unsure) | same |

- **The cold cliff is at ~5 px.** #396's primed 6.3 px reading was about right.
  8 px leaves ~2–3 px of margin and has no errors from either capable reader.
- **What goes first at 5 px is the sign.** Five of Opus's six errors are `-`
  read as `+`. That is the field ADR-0098 §3/§8 uses to say *which side of the
  boundary* an id changed on. The label's least redundant character is its
  most fragile one.
- **Sonnet's 5 px error was reported with full confidence** (`32200` read as
  `32220`). Below the floor, failures are silent: silence-as-coverage again, in
  the label.

**Verdict.** The floor survives. No change.

## The finding nobody asked for: the floors are reader-conditional

Haiku 4.5 scored 0 of 21 on picture sheets, with one mis-indexed partial. At
the 184 px *target* it invented a duplicate "5", an umbrella scene and a
dreamcatcher. On every label sheet, including 10 px, it transcribed the
captions inside the frames and never found the gutter labels. So for this
reader no width ADR-0095 admits, and no type size ADR-0098 floors, makes the
sheet usable. It also fails confidently, which is the map's founding failure
(silence read as coverage) moved into the reader.

Montagent cannot know which model is reading. A width floor measured on a
capable reader is therefore a necessary condition, not a sufficient one, and
nothing in the sheet's disclosure currently says so.

## Limits

- One reading per model per cell. The table is a direction, not a psychometric
  curve.
- Width was isolated by downscaling one 18-tile sheet, not by adding tiles. A
  real sheet at 120 px holds ~36 *different* tiles, and more neighbours may
  cost attention in ways this did not measure.
- The prompt still says "something looks wrong", as the map's trial did. A
  reader told nothing at all is a colder condition not tested here.
- Committed sheets are JPEG q95 4:4:4 (5 MB against 11 MB PNG), as #396 did.
  The observers judged the PNGs; `build_sheets.py` regenerates them.

## Reproducing

`build_sheets.py` expects `tiles/` (doctored) and `clean/` (undoctored)
frames, rendered with `frame --at <t> --full --png` at #396's 18 instants
(recipe in `../contact-sheet-legibility/make_sheets.py`). It writes the sheets
under random names, so the observers see no hint, plus `manifest.json`, which
holds the name mapping and every label's true string.

```sh
python3 score_labels.py    # re-derives the label table from transcripts/
```

`picture_scores.csv` holds each picture verdict with a note from the report
that earned it. The picture verdicts are judgements and cannot be re-derived
by a script. The four Haiku label transcripts past 7 px are the same caption
list as `7-haiku.txt`, differing only in `[unsure]` flags.
