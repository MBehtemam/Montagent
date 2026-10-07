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
  or it has a `units` block with `by: letter`, `liga`, `clig` and `dlig` are off for its whole
  range, so `fi` is two glyphs at every frame
  — a spacing keyed through 0 included — and glyphs never swap mid-shot. The rule reads the
  file, never the value at an instant. The joining scripts are exempt: their script runs keep
  every ligature, because some fonts file a spelling-required ligature such as lam-alef under
  `liga`. Required ligatures (`rlig`) are never switched off.
- **`R-LINE-INK-COLLISION` measures a spaced element at every `letter_spacing` value the file
  writes** — the static value, or each keyframe record's `v` — and reports the worst seam.
  Between two records the spacing passes only through the values between them; a bezier's
  overshoot past a record is not sampled. `measure` answers a keyed spacing at the largest
  value the list writes, and says which value it used in `asked.letter_spacing`.
- **A `units` block staggers one text element** (ADR-0151): its letters, words or lines run
  the block's lists one after another. `by` is `letter` (a grapheme that is not whitespace),
  `word` (a word with its punctuation: punctuation goes to the word before it on its line,
  otherwise to the word after it) or `line` (a `\n` line with a letter). Units are counted in
  reading order over the whole text, ignoring runs; whitespace and empty lines take no step.
  The lists `x`, `y`, `rotation`, `scale` and `opacity` are written **for the first unit** on
  the absolute clock; unit *n* runs them late by its **delay**, its position in `order`
  (`forward` or `reverse`) times `every` ms. `x`, `y` and `rotation` add to the unit's pose
  and `scale` and `opacity` multiply it, in element pixels before the element's own
  transform, so the element can still move as a whole. Each unit turns and scales about the
  pivot `origin` picks in its box: its own advance without the spacing (a word's or line's
  first grapheme to its last) by the line's slot. A unit below opacity 1 draws its stroke
  and fill into one layer, between the element's stroke pass and its fill pass, so two
  overlapping fading units stack whole. `effects`, `blend` and `opacity` apply after.
- **Joined letters move as one** (ADR-0153). Under `by: letter`, a joined piece of a joining
  script (`لسلا` in `السلام`) and letters shaping merges into one glyph move as one body, on
  the first one's timing; each still keeps its index and its step, so the next unit waits.
  `R-UNIT-MERGED` names them; `by: word` avoids the wait.
- **To single out a unit, split it into its own run carrying `unit`** (ADR-0151). Its `delay`
  replaces the derived one; a list replaces that property's list for that unit wholesale.
  **A run's lists are never delayed: they run on the absolute clock as written.** A property
  the run does not name keeps the block's list, run at the run's delay. The run must hold
  exactly one unit (`E-UNIT-RUN-NOT-ONE-UNIT`), name only lists the block declares
  (`E-UNIT-RUN-UNDECLARED`), and not single out a joined or merged letter
  (`E-UNIT-RUN-MERGED`). Retiming is one edit, `"every"`.
- **What the tools say about a stagger.** `query --at` gives `stagger` (`by`, `count`, and the
  `window` from the first list's start to the last's end, after delays and overrides) and a
  `units` row for every unit: delay, which lists a run overrides, `merged_with`, and the pose
  it is drawn with. A staggered caption gets `N-CAPTION-SETTLES`, the instant its text is
  readable. `shift` carries every unit list and keeps the delays, and refuses a cut inside the
  window (`E-SHIFT-UNITS-WINDOW`). **Blind spots:** `R-OFF-CANVAS` widens the rect by the unit
  `x` and `y` offsets but ignores unit `scale` and `rotation`, and says so; the contact sheet
  shows change points for the first and last scheduled units and the overrides only, so read
  every other unit in `query --at`.
- **A stagger across elements is written out** (ADR-0148): each card carries literal `t`
  values. To retime it, edit each card's keyframes, and anchor every replace on the card's
  `id`, because a string such as `"t": 80` repeats across cards.
- **A keyed text `stroke_width` is measured at its widest keyframe** (ADR-0146): the stroke
  moves no glyph, so text is laid out once; each frame paints the resolved width.
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

## Text on a path (ADR-0161)

- **A text may bend its one line along a curve of its own**: `path: {closed, points}`, in a
  `path` element's vocabulary (the paths page), measured in integer pixels from **the
  text's** declared box's top-left. `closed` is required and static; `points` is keyed as one
  whole list of unchanging shape. The `path` takes no other key. It is a guide and is never
  painted: to show the guide, add a `path` element with the same points copied in, and keep
  the two copies in step yourself.
- **One line only.** A `\n` in any run of a text carrying `path` is `E-TEXT-PATH-BREAK`. A
  second line on the same curve is a second text element with its own `path`.
- **`path_offset` places the line** (ADR-0164): a number from `-1` to `2`, keyframe values
  included, the fraction of the curve's length (by the painter's own path measure) where the
  point `align` names sits. Default `0`. Below `0` or above `1` that point is off the curve,
  one curve length past an end at most. Animate it to slide the line along the curve; an
  ease that overshoots holds at `-1` or `2`. On a text with no `path` it is
  `E-TEXT-PATH-OFFSET-ORPHAN`.
- **Slide one element on and off an open curve** with `align: start` and `path_offset` keyed
  from `-1` to `1`: at `-1` the line ends at or before the curve's start, at `1` it begins at
  its end. A line longer than its curve cannot fully enter or fully leave in one element: use
  a stagger's `x`, or a cut to a second element.
- **On a path, `align` names an end of the curve, not of the reading direction.** `start` is
  the line's lowest-distance end, its visual left; `center` its middle; `end` its
  highest-distance end, its visual right. This holds for every script: an Arabic or Hebrew
  line with the defaults (`start`, offset `0`) starts at the curve's first point and draws.
  On flat text `start` is a right-to-left line's right edge (above); on a path it is not.
- **Direction.** The line's visual left-to-right runs in `points` order, for every script. A
  circle drawn clockwise on screen puts the letters on its outside, reading clockwise. To read
  along the bottom of a circle, or from inside it, write the points the other way: reverse the
  list and swap every vertex's `in` and `out`. Reversing a closed list also moves its first
  vertex, which is where `path_offset: 0` is, so rotate the list to keep the start where it was.
- **The curve bends the finished flat line.** Layout, `letter_spacing` and the `units` stagger
  run flat, as above. Then each **rigid body** (a letter, an Arabic joined piece, a ligature
  such as `fi`) is moved and turned whole at its advance midpoint, never bent: its flat `x`
  becomes a distance along the curve, its baseline sits on the curve, and its rotation adds
  to the curve's tangent. So a stagger's `y` lifts a letter along the curve's normal (negative
  `y` is to the left of travel, above a line read left to right), its `x` slides the letter
  along the curve, and its `rotation`, `scale` and `opacity` act as on flat text. On a bend
  tighter than a body is wide, the body's ends lift off the curve: that is the rule, not a
  defect. Space characters are not drawn on a curve; the gap they leave is.
- **The box is the frame the curve is written in**, as on a `path` element, so a text on a
  path pivots `x`, `y`, `origin`, `scale` and `rotation` about its declared box, not the
  block its line makes. `R-BOX-SLACK` does not run on it.
- **Past the ends.** On an open path, a body whose distance falls before `0` or past the
  curve's length is not drawn, so a line slides off an end body by body; a stagger's `x` can
  slide a letter off too. On a closed path the distance wraps around the start point (so
  `-0.25` and `0.75` draw the same), and a body is drawn only if its whole advance, where a
  stagger's `x` has moved it, lies within one loop measured from the line's start
  (ADR-0164). A body whose middle fits but whose end would reach round onto the first letter
  is hidden, so no two letters are laid on top of each other; a loop slightly too long for
  its line can show a gap of up to one letter at the seam. Nothing in `validate` checks that
  a line fits its curve: that needs the font. Instead **`query --at`** prints, on a text
  carrying `path` only, the resolved `path_offset`, the curve's length (informative, like a
  dash outline's), and `hidden:`, the letters not drawn at that instant by their letter
  index (whitespace not counted), or `none`. A hidden joined piece or ligature lists all its
  letters. **`measure`** adds the same `hidden:` and the bent line's ink, in box pixels, at
  the element's first frame.
- **The inset `m` is a frame convention, not a guarantee.** Every vertex and absolute handle,
  in every `points` keyframe, must lie inside the box inset by `m = the largest size among
  the runs + the largest stroke_width among the runs and the element`, or `E-PATH-OUTSIDE-BOX`
  fires and states `m` and how it was derived. `m` keeps a plain letter sitting on the curve
  inside the box. A wide body on a tight bend, or a letter a stagger lifts off the curve, can
  still pass it: `measure`'s bent ink is where that shows. `E-PATH-TOO-FEW-POINTS`,
  `E-PATH-DANGLING-HANDLE` and `E-PATH-KEYFRAME-SHAPE` fire on the text's `path` as on a
  `path` element, each naming `path.points` on a `text`.
- **`shift`** treats `path.points` as a `path`'s points (a cut inside a keyed window it cannot
  write as integer literals is refused) and splits `path_offset` like any number.
