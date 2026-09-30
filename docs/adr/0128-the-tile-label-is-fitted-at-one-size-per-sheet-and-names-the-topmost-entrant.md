---
status: accepted
amends: 0098 (section 8's identifying field names an entrant, and a departure only where nothing entered; its tie-break is the greatest element id; "whole-document span" and "the run's own boundary" are spelled; section 4's fit is one type size per sheet, with the em inside the strip and a character's space after the label; section 2's placeholder is `=`, and after section 5's elision the slot is absent; section 6's sigils and mark are chosen), 0106 (decision 7's blank slot is reconciled with ADR-0098 §2's placeholder), 0114 (the READER CHECK prints before the `SHEET` block, and an answer with no picture has none), 0125 (the record gains `label`, `reader_check` and the label fit; the strip is no longer blank; `E-SHEET-OVERFLOW` raises `type-floor`)
---

# The tile label is fitted at one size per sheet, and names the topmost entrant

[#490](https://github.com/MBehtemam/Montagent/issues/490), a ticket under spec
[#486](https://github.com/MBehtemam/Montagent/issues/486). It draws every tile's label in the
strip beneath it, in the chrome face ADR-0122 embedded, and gives the answer its READER CHECK.
ADR-0098 fixed what the label carries and ADR-0114 what the check says. #486's Further Notes 3
and 9 list what they left unspelled: the sigils, the class mark, what fills an empty id slot,
the tie-break's direction and what "whole-document span" means. Under ADR-0031 each is a spec
gap until an ADR ratifies it. This one ratifies what the build chose, and three things the build
found that no note anticipated.

## Decision

### 1. The id names an entrant, and a departure only where nothing entered

ADR-0098 §8 says the field names what changed at the run's boundary, `+` for entered and `-` for
departed, on the highest layer. Read literally, a departure on a higher layer than every entrant
would win. On the main fixture that happens on four tiles: tile 3 would read `-hook-05` rather
than `+word-05`, because the departing caption sits one layer above the arriving one.

**Entrants are the candidates, and departures are candidates only where nothing entered.** This
is the rule [`docs/research/tile-label/label_field_scan.py`](../research/tile-label/label_field_scan.py)
applied (`entered or departed`), and so the rule every number in ADR-0098 §8 was measured with,
including both planted defects' tiles. It is also the reading §8's own argument makes: *"a
departure-only run names an element that is not on screen in that tile"*. An entrant is on screen
in the tile it labels, so it is the better pointer whenever there is one. The fixture's 18 labels
are that script's table, character for character.

### 2. The tie-break is the greatest element id, and the rest is spelled

- **Highest layer**, as `crate::stack` resolves it, anchors included. An element whose layer
  cannot be resolved ranks below every element whose layer can. Where it sits is `validate`'s
  finding, not a height.
- **Then the greatest element id**, in byte order. This is the script's `max(…, key=(layer, id))`,
  so one key order decides both steps. The fixture never ties among candidates, so no label on it
  depends on the direction. `tests/frame_range.rs` ties three elements whose greatest id is
  neither first nor last in array order.
- **Whole-document span** means `start ≤ 0` and `end ≥` the document's extent, which is its
  declared `duration`, or the last `end` any element states. The requested range plays no part,
  so a label never depends on the range asked for.
- **The boundary is the document's.** A range opening inside a visual state names the change
  that state opened with, as the whole document's visual states record it. The offset is
  measured from the run on the provenance line, which starts at `from`. `frame --from 1510` on a
  state that opened at 1000 labels its first tile `1 1520ms +10 +b`.
- **An element with no id is not a candidate.** The label would have nothing to call it, and a
  missing id is already the schema's finding.

### 3. Empty slots: `=` where nothing changed, blank where nothing could, absent when elided

ADR-0098 §2 says a field with no value prints a placeholder. ADR-0106 D7 says a keyframe or
infill tile leaves the slot blank, never a placeholder. They are about different slots, and each
stands:

- **A run tile whose boundary has no candidate prints `=`.** This happens when every change is a
  whole-document element or has no id, for example the first tile of a document whose only
  element at 0 is its background. It is the script's own placeholder. It cannot be read as an
  element's name, because every real id prints behind `+` or `-`.
- **A keyframe or infill tile's slot is blank**: its label ends after the offset (ADR-0106 D7).
  There is no boundary change to name.
- **After sheet-wide elision the slot is absent on every tile.** It is not filled with a
  placeholder, because a sheet of `=` would read as "nothing changed anywhere". Every label on the
  sheet has the same four fields, and the answer says why: `picture.ids` is `elided`, and the
  `labels` line of the text says the longest label does not fit at the floor with its id.

### 4. One type size per sheet, fitted to the strip and a space

The sizing function decides the fit (#486), in the chrome face's font units:

- **One size for every label on the sheet**: the largest whole pixel size at which the longest
  label fits. A reader calibrates once, as ADR-0098 §5 wanted of sheet-wide elision.
- **Width**: the label's advance **plus one character's** fits the tile's width less 2 px at each
  end. The first build fitted the bare label, and its 30-tile golden showed `+0` and the next
  tile's `11 2000ms` running together as `+011 2000ms`. The extra advance keeps neighbouring
  labels at least a word space apart.
- **Height**: the em fits the strip. The label is centred on the strip and clipped to it, so no
  glyph reaches a tile's pixels. Where the strip is what binds, a descender may lose a pixel.
- **The floor is 8 px** (ADR-0098 §4). If the longest label with its id falls under it, the ids
  elide. If the longest numeric core still falls under it, the sheet is refused as
  `E-SHEET-OVERFLOW` with `limit: type-floor` and `limit_px: 8` (ADR-0126 §3). `fits` and
  `sub_ranges` count the sheets whose tiles clear 140 px **and** whose cores clear 8 px. A
  sub-range's labels are never longer than these, since its instants and offsets are the same and
  its index starts at 1 again.

**A frame too short for the floor is refused at every range.** The strip is 11% of the tile's
height (ADR-0098 §1), so a tile served under about 73 px tall has a strip under 8 px, and no
label fits it. A 100 × 60 project is refused on `type-floor` with `fits: 0` and no sub-range,
exactly as ADR-0126 §3's frame too tall for one tile is refused on `tile-width`. ADR-0125 §2's cap
keeps a small project's tiles at true pixels; it cannot make its strip taller than 11% of them.
Growing the strip for such frames would change ADR-0095's measured geometry, and is not done here.

On the fixture, `9 42800ms +37 +word-08-bridge` is 29 characters. With its space that is 18,000
units, 144 px at 8 px, so a tile carries ids from **148 px** up. At 18 states the tiles serve at
184 px and the labels draw at 10 px, ids carried. Padded to 30 states they serve at 141 px, the ids
elide, and the cores draw at 15 px. A 1920 × 200 project of 30 states serves 588 px tiles over
7 px strips, which is refused on the type floor while the tile width has room to spare.

### 5. The sigils are `K` and `I`, and the mark is an inverted strip

Neither is drawn yet, because no keyframe or infill tile exists before #491 and #492. They are
fixed here so that those tickets build to a decision:

- **Sigils**: `K` for a keyframe tile and `I` for an infill tile, in ADR-0098 §2's second place:
  `12 K 42840ms +27`. They are ASCII, since the chrome face draws a box for anything else, and
  JetBrains Mono separates `I` from `1` and `l`. A run tile has no sigil, and its label reads
  `9 42800ms +37 +word-08-bridge`, one space between fields.
- **Mark**: a keyframe or infill tile's **strip is inverted**, `#1A1A1A` ink on a `#E6E6E6`
  ground, where a run tile's is `#F2F2F2` on `#1A1A1A`. It lives in the strip, so the tile's
  pixels stay `frame --at`'s (ADR-0098 §1). It is a change of luminance, not hue, so it survives
  a greyscale reproduction, which Juror 1 of ADR-0098 §6 said a tint would not.

### 6. The record and the text

- Every provenance entry carries **`label`**, the exact string drawn beneath its tile. The drawing
  reads that same string, so the two cannot differ.
- `picture` gains **`label_px`** (the one type size), **`label_floor_px`** (8) and **`ids`**
  (`carried` or `elided`).
- The sheet gains **`reader_check: {tile, label, sentence}`** after `picture`, in ADR-0114 §3's
  words, around tile 1's label. **It is `null` on an answer with no picture** (ADR-0125 §5), where
  there is no tile 1 to quote. That is the one range answer without a READER CHECK, and it is not a
  refusal.
- The text prints **`READER CHECK`** directly under the header line, before the `SHEET` block, on
  one line so that the text and the JSON hold one string. A reader that stops early meets it
  first. Each `PROVENANCE` line carries ``label `…` `` before the presence set, and the `SHEET` block
  gains a `labels` line.
- Labels are drawn in `#F2F2F2`, left-aligned, 2 px in.
- `E-SHEET-OVERFLOW`'s template names the limit in a clause true of both: *"before it passes its
  {limit} limit of {limit_px} px served, below which a tile or its label is not legible."*

Nothing printed names a model (ADR-0114 §4). A test searches the text and the JSON for model names.

## Consequences

- `sizing::size` takes the longest label's two widths in font units and returns the type size
  and the elision. `Limit` gains `TypeFloor`.
- `frame`'s range mode has a `label` module: the label strings, the id selector, and the drawing.
- `CONTEXT.md`'s **Tile label** gains the candidate rule and the one-size fit.
- The tool description's reader-check sentence stays with the surfaces (#493).

## Evidence

- `crates/montagent-core/src/verbs/frame/sizing.rs`: the elision edge at 148 px across every
  portrait count to 30; 10 px at 184 and 15 px elided at 141; the strip cap; the type-floor
  refusal with sub-ranges that each fit, and the 100 × 60 frame refused with none.
- The drawn string's identity with `provenance[].label` is by construction: the strip is drawn
  from that very string. The SSIM goldens show what is drawn; they cannot compare text.
- `tests/frame_range.rs`: the fixture's 18 labels against the research table, and its 30-state
  padding elided on every tile. Also the tie, the departure, `=`, a range opening mid-state, the
  READER CHECK's placement and byte identity, no model name, and the type-floor refusal.
- `tests/cross_verb.rs`: every tile still equals `frame --at` byte for byte with its label drawn,
  every strip has ink, and the quoted label is `provenance[0].label`.
- `tests/golden_frames.rs`: `sheet-labelled` and `sheet-elided`, looked at before committing.
