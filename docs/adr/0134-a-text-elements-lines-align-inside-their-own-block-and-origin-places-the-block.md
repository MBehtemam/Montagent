---
status: accepted
amends: 0011 (`query --at`'s ink box places a text element's block by `origin` over the block's own width, and aligns each line inside that block with the painter's function; the declared `width` is not read), 0133 (§6's recorded residual is settled: the ink box and the painter now agree on the box as well as the direction)
---

# A text element's lines align inside their own block, and `origin` places the block

[#551](https://github.com/MBehtemam/Montagent/issues/551), which closes §1 of
[#277](https://github.com/MBehtemam/Montagent/issues/277). Reproduced on `74f64832`: one text
element, `"cobweb"` at size 60 in the fixture's `OpenRunde-Bold.otf`, `x:540`,
`origin:"top-center"`, `width:600`, `align:"start"`.

```
$ montagent query p.montagent.json --at 0 --json    # ink_box.x 240.0, width 233.5
$ montagent frame p.montagent.json --at 0 --full --png
                                                     # ink from x 425 to x 655
```

`query`'s ink box ran from 240 to 473.5, flush with the declared box's left edge. The painter
drew the word from 425 to 655, centred on 540. The two were 185 px apart. An agent using the
ink box to check overlap or margins was checking a rectangle where no text is drawn.

The painter aligned the lines inside **the block the lines make**. The ink box aligned them
inside **the declared `width`**, placed by `origin`, with an unclamped
`free = width − advance`. The two agree only when a line's advance equals the declared
`width`. #277 §1 recorded the painter's reading as a choice made to finish #213 and not yet
ratified. ADR-0133 §6 settled the direction half of alignment and left this half open.

## The decision

### 1. The lines align inside the block they make

A text element's **block** is the widest line's advance (before the stroke, ADR-0014) by the
sum of the line slots (ADR-0028). Each line is aligned **inside that block**. `start` and `end`
resolve against the line's own base direction (ADR-0133 §3), and `center` ignores direction.
A line as wide as the block starts at the block's edge whatever `align` says. So a
single-line element is placed the same way under all three values.

This is the reading two existing ADRs already give. ADR-0007 defines `align` as *"how lines
align to each other"*, not how they sit in a container. ADR-0014 makes a text element's
`width`/`height` *"a container claim, not painted geometry"*. It is also how the vertical axis
already works: `measure` places the block by `origin` over the block's own height and never
reads the declared `height` (ADR-0024). Aligning horizontally inside the declared `width` while
placing vertically inside the block would make one element's two axes answer to two different
boxes.

The rejected alternative is aligning inside the declared `width`, as a CSS text block does.
Adopting it would turn a container claim into drawn geometry, which ADR-0014 rules out. It
would also move every committed frame whose text is narrower than its declared box.

### 2. `origin` places the block, not the declared box

`origin`'s horizontal component pivots about the block's own width:
`block_left = x − fx × block_width`. This is the transform ADR-0012 applies to every element,
given the block's extent as the text element's extent. The declared `width` takes no part in
where text is drawn.

### 3. `query --at`'s ink box uses the painter's placement and alignment

The ink box places the block by rule 2 and offsets each line into it with
`montagent_text::place::offset`. That is the painter's own function, clamp included, made
public for this. There is no second copy of the arithmetic, so the two cannot drift apart
again. The ink box no longer reads the declared `width`. An element without an integer
`width` no longer stops it from answering, because nothing it computes depends on one.

`crates/montagent-core/tests/query_at_geometry.rs` holds the ink box's horizontal extent
against the ink in `frame`'s own full-scale PNG. It covers a single line, a block of mixed
line widths, and two blocks whose narrow line carries a stroke that shows which edge `align`
sent it to: one left-to-right, one right-to-left. Each is tested under `start`, `center` and
`end`, at `origin`s that pivot on the left, the centre and the right. The box is advance-based
and the picture is ink, so the ink sits inside the box by the edge glyphs' side bearings,
which is at most 2 px for the Latin glyphs here.

## Consequences

- **Nothing that is drawn changes.** The painter already worked this way. No golden frame or
  fixture moves.
- **`query --at`'s `ink_box` moves horizontally** for every text element whose widest line is
  narrower or wider than its declared `width`, to where the text is drawn. Its vertical extent
  is unchanged.
- **Under the block reading, `align` can move only lines narrower than the block.** The ink
  box is the union of the lines, so `align` changes its horizontal extent only through a
  narrow line's stroke reaching past the block edge it is sent to.
- `crates/montagent-core/docs/format.md` (`montagent://format.md`) states rules 1 and 2 under
  *Text*.

## Not decided here

- The ink box's **vertical** extent, which uses line ascent and descent rather than glyph ink
  (`y` 99.7 → 172.3 against painted ink 114 → 159 in the reproduction).
- #277 §2's recorded residual (trailing whitespace is not hung), and §3, §4 and §5.
