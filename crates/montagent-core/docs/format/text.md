# The Montagent project format: text

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the rules for a `text` element. Like the rest of
the format docs, every rule here is an accepted decision in the ADR series, cited inline by
number, and the ADR is right where the two disagree.

## Text

- **Content is always an ordered array of runs,** even when there is only one.
- **A run boundary is style only** (ADR-0008). It never implies a line break. A line break is a `\n`
  character inside a run's text — the agent chooses every break, and the renderer chooses
  none.
- **`align`'s `start` and `end` follow each line's own direction** (ADR-0133). A line's
  direction is the one its characters give: the first strong letter decides, and a line with
  none is left-to-right. `start` is therefore the right edge of a Hebrew or Arabic line.
  **A run's `dir` never changes a line's direction.** It lays that run out as a bidi
  *isolate* (ADR-0007): the run's own text is reordered, so a `!` at its edge takes the
  run's direction, and nothing outside the run moves. Shaping does not cross the run's edge,
  so a kern or an Arabic join across it is lost.
- **Every text element is a caption unless it says `caption: false`** (ADR-0136). The four
  caption checks — `R-CAPTION-PACE`, `R-CAPTION-MIN-DURATION`, `R-CAPTION-NO-AUDIO` and
  `R-CAPTION-REPEAT-DURATION` — run on every `text` element, whatever its track is called.
  Write `caption: false` on a title, a lower-third, a logo or a kinetic word, and none of
  the four reports it; it is also left out of the repeat check's grouping of identical text.
  Omitting the field and writing `caption: true` mean the same thing. It is one switch for
  all four, it changes no pixel and no other check, and nothing infers it for you.
- **Sizes are literal pixels** (ADR-0007), and a text element's `width`/`height` is a container claim
  rather than drawn geometry. Use `measure` to find out what a string actually occupies in
  the font the project declares; do not estimate it.
- **Lines align inside the block they make, and `origin` places that block** (ADR-0135). The
  block is the widest line's advance by the sum of the line slots. The declared `width` takes
  no part in where text is drawn, so a single line sits at the same place under every `align`,
  and `align` moves only lines narrower than the widest. `query --at`'s `ink_box` is placed
  the same way, so it is where the text is drawn.
- **`letter_spacing` is an integer in thousandths of an em** (ADR-0151, ADR-0153). After
  every grapheme of a line, `size × letter_spacing / 1000` pixels are added, where `size` is
  the size of **the run the grapheme sits in**, so a `size` edit keeps the same tracking.
  Spaces count, and so do run boundaries. **Nothing is added after a line's last grapheme**,
  so a centred or end-aligned line stays where `align` puts it. Negative tightens; the default
  is 0. It belongs to the element: a run cannot override it. It is an animatable property —
  keyframe values are integers, and the resolved value is never rounded — and the spaced
  line is what the block, `align`, `origin` and `query --at`'s `ink_box` all measure.
  Lines still break only at `\n`, so spacing never re-breaks a line.
  **No spacing is added between two letters of the same joining script** (Arabic, Syriac,
  N'Ko, Mongolian, Adlam and the others Unicode lists), whether or not they join: it would
  tear the cursive stroke. A tracked mixed line still spreads its spaces, punctuation,
  digits and other scripts. `validate` names an element where this suppresses a pair as
  `R-SPACING-SUPPRESSED`.
- **A non-zero `letter_spacing` switches optional ligatures off for the whole element**
  (ADR-0151, ADR-0153). If any `letter_spacing` value or keyframe on the element is non-zero,
  `liga`, `clig` and `dlig` are off for its whole range, so `fi` is two glyphs at every frame
  — a spacing keyed through 0 included — and glyphs never swap mid-shot. The rule reads the
  file, never the value at an instant. The joining scripts are exempt: their script runs keep
  every ligature, because some fonts file a spelling-required ligature such as lam-alef under
  `liga`. Required ligatures (`rlig`) are never switched off.
- **`R-LINE-INK-COLLISION` measures a spaced element at every `letter_spacing` value the file
  writes** — the static value, or each keyframe record's `v` — and reports the worst seam.
  Between two records the spacing passes only through the values between them; a bezier's
  overshoot past a record is not sampled. `measure` answers a keyed spacing at the largest
  value the list writes, and says which value it used in `asked.letter_spacing`.
- **`line_height` is a multiplier restricted to one decimal digit** (ADR-0028) — `1.0`, `1.1`, `1.2` —
  so it is always exactly `n/10`. A line's height is the largest `size` among its runs times
  `line_height`; the block height is the `ceil` of that over the line count, computed in
  exact integer arithmetic. Defaults to `1.2`.
- **Fonts are declared as an ordered chain of font *files*, never a system family name**
  (ADR-0007, ADR-0057).
  The renderer opens nothing outside that chain, which is what makes a project that renders
  on your machine render on a clean one.
- **Every font file is vendored through `montagent fonts vendor`, and the `fontVendor` table
  is its receipt** (ADR-0057). The table is keyed by file path — one path, one licence, one
  `sha256` — however many `fonts` chains reference the file, and it is written by the tool,
  never by hand. `validate` checks that every chain path has an entry whose hash matches the
  bytes on disk (`error` when it does not) and that every entry is still referenced (a
  `note` when not; nothing prunes it for you). It re-checks integrity, never licence law.
  `fonts vendor` runs its licence check *before* any bytes are copied: a font on the
  blocklist of known non-redistributable fonts is refused with no override, a recognised
  open licence is recorded, and anything else needs a `--licence` you have verified. Use
  `montagent fonts list` to see each installed font's status before you try.
