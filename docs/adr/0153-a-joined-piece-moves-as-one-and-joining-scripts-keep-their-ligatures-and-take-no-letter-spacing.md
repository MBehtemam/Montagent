---
status: accepted
amends: 0151 (a joined piece shares timing like a merged glyph and transforms as one body; the ligature rule spares the joining scripts; no letter spacing between two letters of a joining script; new review `R-SPACING-SUPPRESSED`), 0007 (a text element's joining-script letters are shaped with their optional ligatures on and take no letter spacing between them)
---

# A joined piece moves as one, and joining scripts keep their ligatures and take no letter spacing

[#694](https://github.com/MBehtemam/Montagent/issues/694), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). The accepted prototype of
[ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md)
([#683](https://github.com/MBehtemam/Montagent/issues/683)) found two things it does not
settle for cursive scripts. Reading CSS Text 3 surfaced a third.

1. **Joined letters tear apart in motion.** Arabic letters join cursively, but each one is its
   own glyph. Under `by: letter` each steps on its own, so the joins break while the letters
   move (`السلام عليكم`) and come back together at rest. ADR-0151 keeps letters together only
   where shaping *merges* them into one glyph (lam-alef, conjuncts).
2. **The ligature rule misspells Arabic in some fonts.** ADR-0151 switches `liga`, `clig` and
   `dlig` off when spacing is non-zero or `by: letter` is set. Lateef, Scheherazade New and
   Harmattan put lam-alef, which Arabic spelling requires, under `liga`, so they lose it. Cairo
   and Tajawal put it under `rlig` and keep it.
3. **Letter spacing tears every join, even at rest.** ADR-0151 adds spacing after every
   grapheme but a line's last. Between two joined Arabic letters that leaves a gap in the
   stroke.

**Precedent.** CSS Text 3 (current draft) covers both spacing questions. §7.2: when spacing is
non-zero a browser "should not apply optional ligatures, i.e. those that are not defined as
required for fundamentally correct glyph shaping". OpenType *expects* required ligatures under
`rlig`, but the definition is what correct spelling needs, not the tag. §7.2.1: a browser that
cannot stretch cursive text by elongation "must not apply spacing between any pair of that
script's typographic letter units at all". It names two layouts as bad: even gaps between every
letter, and gaps only where letters do not join.

**How it was decided.** One grilling round, put to a court of three jurors (Opus, Sonnet and
Fable) judging as the agent that writes and edits the file. The jurors chose the same option on
all three questions, and the owner ruled with the Judge's read.

## The decision

### 1. Terms

- A **joining script** is a script whose letters carry a Unicode `Joining_Type` of `D` (dual) or
  `R` (right): Arabic, Syriac, N'Ko, Mongolian, Adlam and the others Unicode lists.
- A **joining-script letter** is a grapheme cluster whose base character has General_Category
  `L*` and is in a joining script. A Common or Inherited character counts as the script around
  it, so the tatweel is a letter. Arabic-Indic digits, Arabic punctuation and spaces are not.
- Two adjacent graphemes **join** when the Unicode cursive-joining rules connect them
  (`ArabicShaping.txt`, Core Specification §9.2). Transparent marks are skipped, ZWJ makes a join
  and ZWNJ breaks one.
- A **joined piece** is a maximal run of graphemes in which each adjacent pair joins. `السلام` is
  three pieces: `ا`, `لسلا` and `م`. A grapheme that joins nothing is a piece of its own.

All four come from the text alone. No font is read.

### 2. A joined piece moves as one (amends ADR-0151 §3)

ADR-0151's merged rule widens from "shaping merges" to "shaping merges or joins":

- **Timing.** Under `by: letter`, a joined piece moves on the timing of its first grapheme in
  reading order. Each grapheme in it keeps its index and its step, so the units after it keep
  their schedule. A five-letter piece therefore holds the next piece back for four steps.
- **One body.** The piece transforms as one body. Its box runs from its first grapheme's leading
  edge to its last grapheme's trailing edge, by the line's slot, and the `units` block's `origin`
  picks the pivot in that box. Rotating or scaling each letter about its own pivot would tear the
  joins this rule exists to keep. A merged glyph that is not part of a longer piece already
  behaves this way.
- **Fading.** A piece whose resolved opacity is below 1 is drawn into one layer, as ADR-0151 draws
  one fading unit. Separate layers would double the alpha where neighbours overlap at a join.
- **`by: word` and `by: line`** are unchanged. A piece never crosses a word.

The tools keep their codes and widen their meaning:

| Code or field | Now covers |
| --- | --- |
| `R-UNIT-MERGED` (review) | Graphemes that share a piece, as well as graphemes that share a glyph. The finding says which (`joined` or `merged`), and says that `by: word` is one edit away if the waiting steps are unwanted. |
| `E-UNIT-RUN-MERGED` (error) | A run's `unit` that singles out a grapheme in a piece of more than one grapheme. A one-grapheme piece, such as a lone `ا`, can still be singled out. |
| `merged_with` in `query --at` | The indexes that move with the unit, by a join or a merge. |

### 3. The ligature rule spares the joining scripts (amends ADR-0151 §1)

ADR-0151's trigger is unchanged: any non-zero `letter_spacing` value or keyframe, or a `units`
block with `by: letter`. What it switches off narrows. `liga`, `clig` and `dlig` are switched off
only in the element's script runs that are **not** a joining script. Script runs are found by
Unicode script itemisation of the text, with Common and Inherited characters resolved to the
script around them. Joining-script runs are shaped as today.

- A Latin `fi` in a mixed line still breaks apart.
- A lam-alef under `liga` now survives. Under `by: letter` it is a merged glyph, so §2's rule and
  `R-UNIT-MERGED` apply to it as they do to Cairo's `rlig` one.
- Optional Arabic ligatures (`dlig` and any other `liga` entries) stay on too. That is accepted:
  §4 puts no spacing between those letters, so nothing pulls the ligature apart, and this is the
  CSS rule applied to a spacing that is zero.

`validate` still says from the file alone how every element shapes.

### 4. No letter spacing between two letters of a joining script (amends ADR-0151 §1)

ADR-0151's "after every grapheme cluster of a line, except the line's last" gains one exception.
No spacing is added after a grapheme when it and the next grapheme on the line are **both
joining-script letters of the same script**, whether or not they join. Spacing is still added
after spaces, punctuation, digits and every other script, so a tracked mixed line still spreads
its words apart.

This is CSS's fallback for a renderer that does not elongate. Layout measures the spaced line as
before, with these pairs contributing nothing.

**`R-SPACING-SUPPRESSED`** (review) fires on an element that has a non-zero `letter_spacing` value
or keyframe and at least one pair this rule suppresses. It names the script and the first word
affected, and says that letter spacing has no effect between that script's letters. It does not
fire on an Arabic element whose spacing is zero throughout.

## The four tests

| Invariant | Joined pieces | Ligatures by script | Spacing suppression |
| --- | --- | --- | --- |
| Literal values | No new value. | No new value. | No new value. |
| Closed vocabulary | No new field. Two codes widen. | No new field. | One new review code. |
| Checkable by `validate` | Pieces come from Unicode properties of the text. | Script runs come from the text. | Suppressed pairs come from the text. |
| Exact-string replace | Unchanged. Moving from letters to words is one edit, `"by": "word"`. | Unchanged. | Unchanged. |

## Considered and refused

- **The whole word moves as one under `by: letter`** (CSS's "each word as a single letter
  unit"). Refused: it is `by: word` under another name, and coarser than the script needs.
  `السلام` has three pieces that can move apart without tearing a stroke. CSS wrote that rule
  for spacing, not motion.
- **Leaving the tear, with a review that names it.** Refused: every frame in motion shows broken
  Arabic, and a review that fires on every Arabic stagger trains agents to ignore reviews.
- **Refusing `by: letter` on joining-script text.** Refused, as ADR-0151 refused it: it would bar
  whole scripts, including letters that do not join and stagger cleanly.
- **Sparing a hand-kept list of required ligatures.** Refused: the list is never complete, and
  OpenType cannot turn on one ligature inside a feature, so it would need font-specific shaping
  tricks that cannot be checked from the file.
- **Keeping the rule and having `validate` name the lost ligature.** Refused: it puts fonts into
  `validate` and still renders misspelled text.
- **Spacing only between letters that do not join.** Refused: CSS names it bad. The gaps read as
  false word boundaries.
- **Leaving spacing on joined letters.** Refused: it tears every join at rest.

## Considered and deferred

- **Kashida elongation** turns letter spacing into stretched joins, as CSS allows. Deferred: it
  depends on the font, the position in the word and the language, and the result cannot be
  checked against the file. It can enter later by its own ADR, with a prototype.

## Consequences

- **Slice 1, letter spacing** ([#692](https://github.com/MBehtemam/Montagent/issues/692)), gains
  §3's ligature-by-script rule, §4's suppression and `R-SPACING-SUPPRESSED`. Its spec confirms
  that the shaper can set features on part of a text. Its byte-identity test includes a
  mixed-script element.
- **Slice 2, the units block** ([#693](https://github.com/MBehtemam/Montagent/issues/693)), gains
  §2: joined pieces in the unit model, the piece's box and pivot, one layer per fading piece, and
  the widened codes. Its spec shows the waiting steps in stills of `السلام عليكم`.
- **No new prototype gate.** §2 keeps joins whole rather than adding a new motion, and §3 and §4
  follow CSS.
- `CONTEXT.md` gains **Joined piece**, and **Letter spacing** and **Unit** are reworded to it.
- `format.md` and the feature map state that joining scripts take no letter spacing between their
  letters and keep their ligatures, and that a joined piece moves as one under `by: letter`.
