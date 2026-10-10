# Premiere Pro titles, text, graphics and Motion Graphics templates, set against Montagent v0.2.0

Research for [#887](https://github.com/MBehtemam/Montagent/issues/887), part of the map
[#882](https://github.com/MBehtemam/Montagent/issues/882). Area slug: **titles**.

- **Date:** 2026-10-10. Nothing was built or run. **This note decides nothing.** It lists what
  Premiere has and what Montagent v0.2.0 has for each item.
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, = `origin/main`). Sources: `CONTEXT.md`,
  `schema/montagent.schema.json`, the ADRs cited per row, the MCP tool surface
  (`crates/montagent/src/mcp.rs`: `where`, `validate`, `create_project`, `query`, `frame`,
  `render`, `verify`, `preview`, `measure`, `shift`, `compare`), the CLI's `fonts list` and
  `fonts vendor`, and the skills `montagent`, `montagent-motion` (with
  `scripts/type_on.py`), `montagent-craft`, `montagent-footage` and `montagent-prerender`.
- **Premiere side:** Adobe's own helpx.adobe.com pages for Premiere (desktop 25.x/26.x, the
  Properties panel that replaced the Essential Graphics panel in 25.0). No third-party source
  is used.
- **Counts (main table, 74 rows):** have **29** · have (skill) **0** · partial **23** ·
  missing **17** · refused **5**. A further 11 Premiere items that could not be confirmed from an
  Adobe page are listed separately under [Not confirmed](#premiere-items-not-confirmed-from-an-adobe-page)
  and are not counted.

## How this was read, and what that does to the confidence

The sandbox this was written in **could not open helpx.adobe.com**: the egress proxy refuses the
host (`CONNECT tunnel failed, response 403`), as it did for the audio precedent study
(`docs/research/audio-effects/PRECEDENT.md`). The session's web-search budget also ran out part
way through. Each Premiere cell is therefore tagged:

- **[C]** an Adobe page was located, and the search engine's extract of *that page* states the
  mechanism. The URL is that page.
- **[P]** the mechanism is quoted from the earlier precedent study on branch
  `research/664-capcut-premiere-mechanisms` (`docs/research/skia-reach/capcut-premiere-mechanisms.md`,
  read 2026-10-05), which cites the same Adobe URL.
- **[T]** the Adobe page was located by its title and URL, but no extract stated the detail.
  The mechanism in the cell is **recalled** from the product and is a lead, not a citation.

Re-open every URL before quoting it in an ADR.

Statuses follow the map: **have** (the mechanism ships in v0.2.0), **have (skill)** (a bundled
skill provides it, not the format), **partial** (some of it ships; the last column says what does
not), **missing** (nothing equivalent, and no ADR refuses it), **refused** (an ADR or ADR-0145's
four invariants rule it out; cited). Read by mechanism, not by name (ADR-0145): a Premiere
"graphic clip" is compared with Montagent elements, not with a Montagent type of that name.

## The table

### Creating and editing text

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Create a title (point text) | Type tool, type in the Program Monitor; right-click > Edit Properties opens the Properties panel [C] | https://helpx.adobe.com/premiere-pro/using/edit-text-using-the-properties-panel.html | have | `text` element with `runs`, `font`, `size`, `width`, `height` (ADR-0007, ADR-0014); `create_project` MCP tool scaffolds the file | — (the agent writes JSON; the MCP surface reads, checks and renders, ADR-0011) |
| Paragraph (area) text that wraps to a box | Drag a text box with the Type tool; text wraps inside it [T] | https://helpx.adobe.com/premiere-pro/using/edit-text-using-the-properties-panel.html | refused | ADR-0007 ("No fit-to-box. No automatic wrapping."), ADR-0008 (line breaks belong to the agent); `measure` returns the extent | — (route: the agent writes `\n` and checks with `measure`) |
| Style part of a text layer | Apply attributes to a selection (one word) in the Properties panel [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-style.html | have | `runs`: each run carries style deltas `font`, `size`, `color`, `stroke`, `stroke_width`, `dir` (ADR-0007, ADR-0014) | — |
| Edit text in the Graphics tab | Edit graphic text from a Graphics tab [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/edit-text-in-graphics-tab.html | have | Every element is one line, so its text is a unique exact-string edit (ADR-0005, ADR-0007); `query --where "type = text"` lists every text (ADR-0070) | — |
| Text animation presets (Typewriter, Text Animator) | Named presets: Typewriter reveals "letter by letter"; Text Animator applies "motion presets" [P] | https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/animations.html | have | `units` stagger by `letter`, `word` or `line`, with per-unit `unit` overrides (ADR-0148, ADR-0151, ADR-0153); `type_on.py` in `montagent-motion` for typing with a cursor | — (no named preset list; the move is written as keyframes) |

### Character and paragraph properties

The Text section's parameter list, as Adobe's style-parameters page gives it [C]: Font, Font
Style, Font Size, Paragraph Alignment, Tracking, Kerning, Leading, Baseline Shift, Tab Width,
Faux Bold, Faux Italics, All Caps, Small Caps, Superscript, Subscript, Underline, Tsume.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Font | Font picker [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | `font` is a key into the top-level `fonts` table, an ordered fallback chain of project-relative files (ADR-0007, ADR-0057, schema `FontFile`) | — (by design, never an installed family name) |
| Font Style (weight, italic face) | Font Style menu [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | A face is its own file, chosen per run by `font`; `.ttc` faces by `index` (ADR-0007, ADR-0102); `weight`/`bold` are rejected naming the chain (ADR-0014) | — |
| Font Size | Font Size [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | Literal integer `size` on the element and on each run (ADR-0007) | — (`size` is not keyable; ADR-0146 §3 admits text paint only, and `scale` animates drawn size) |
| Paragraph alignment (left, centre, right) | Paragraph Alignment [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | `align`: `start`, `center`, `end`, resolved per line's own direction (ADR-0007, ADR-0133, ADR-0135) | — |
| Justified alignment | Justify options among the paragraph alignments [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | missing | `Align` has three values (schema) | Justified lines (space distributed to the block width). No ADR refuses it. |
| Tracking | Tracking, a static style value [C][P] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/style-parameters-when-applying-from-style-browser.html | have | `letter_spacing` in thousandths of an em, keyable (ADR-0151); joining scripts take none (ADR-0153) | — (element-level only; Premiere applies tracking to a selection [T]) |
| Kerning | Kerning (pair spacing at the cursor) [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | The font's own kerning is applied by the shaper (ADR-0010; `type_on.py` places units "kerning included") | Manual kerning of one letter pair. `letter_spacing` cannot be written on a run (schema). |
| Leading | Leading [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | `line_height`, a multiplier in exact tenths, half-leading baseline (ADR-0007, ADR-0028, ADR-0029) | — (no finer step than 0.1 and no per-line value, both by ADR-0028's exact arithmetic) |
| Baseline Shift | Raise or lower selected text from the baseline [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | A `units` block plus a run's `unit` override can offset one letter or word in `y` (ADR-0151 §4) | A plain per-run baseline offset. Every run on a line sits on one shared baseline (ADR-0029). |
| Tab Width | Tab Width [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | missing | No tab field | Tab stops and tab width. Columns are separate elements placed by the agent. |
| Faux Bold | Synthetic bold for a font with no bold face [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | refused | ADR-0007 ("No `weight`, no `bold`": synthetic emboldening is renderer-specific; the schema must reject it); ADR-0014 | — (route: vendor a bold file) |
| Faux Italics | Synthetic oblique [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | missing | A true italic face works through a run's `font` (ADR-0007) | Synthetic slant. ADR-0007's refusal names bold and weight only, so this is not refused by name. |
| All Caps | Display text in capitals [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | Capitals written literally in `runs.text` | A case transform that keeps the source casing in the file. |
| Small Caps | Small capitals (faux when the font lacks them) [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | Faux small caps as capital runs at a smaller `size` (ADR-0007) | The font's real small caps (`smcp`), and any OpenType feature switch: only ligature handling is fixed, by rule (ADR-0151, ADR-0153). |
| Superscript and Subscript | Superscript, Subscript (with baseline shift) [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | A smaller run `size`; a vertical offset only through `units`/`unit` (ADR-0151 §4) | A raised or lowered run, and the `sups`/`subs` features. |
| Underline | Underline [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | A separate `rect` or open `path` under the text; `montagent-motion` recipes "Grow from an edge" and "Draw a line on" draw and animate one; `query --at` gives the ink box | An underline that is a text or run attribute and follows the glyphs when the text changes. |
| Tsume | Tsume (CJK proportional squeeze) [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | missing | — | Tsume. |

### Fonts

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Colour fonts | Pick a colour font in the Properties panel's font picker for multicolour letterforms [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-color-fonts.html | missing | No ADR, schema key or test names COLR/CPAL, sbix, CBDT or SVG-in-OpenType | Colour glyphs. Whether the painter draws them today is undetermined; it needs a measured test before it can count. |
| Replace fonts in project | Graphics and Titles > Replace Fonts in Project updates every font at once [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html | have | One edit to a `fonts` chain re-points every element using its key (ADR-0007); the font-swap census reports what the swap moved (ADR-0085) | — |
| Adobe Fonts | A new font must be installed or activated through Adobe Fonts [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html | refused | ADR-0007 (a font is a file the project declares, never a system family; the renderer never consults system fonts); ADR-0057 (vendoring behind a licence gate) | — (route: `fonts vendor` a licensed file) |
| Missing fonts | Install or activate the missing font, or replace it [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html | have | `validate`: a missing font file, or a character with no glyph in any chain entry (`.notdef`), is an error (ADR-0007); `fontVendor` hashes are checked (ADR-0057) | — |
| Font discovery on the machine | Font picker lists installed and activated fonts [T] | https://helpx.adobe.com/premiere-pro/using/edit-text-using-the-properties-panel.html | have | `montagent fonts list` walks the machine's fonts with licence status; `fonts vendor` copies one in (ADR-0057). CLI only, not an MCP tool | — |

### Appearance: fill, stroke, background, shadow, gradients

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Solid fill | Appearance > Fill colour [C] | https://helpx.adobe.com/ca/premiere-pro/using/work-with-fill-stroke-background-shadow.html | have | `color` on text, `fill` on shapes, `#RRGGBBAA`; keyable on the element (ADR-0146 §3) | — (a run's colour is static; timed run colour is a `highlight`, ADR-0048) |
| Linear and radial gradient fill | Fill type Solid, Linear Gradient or Radial Gradient; colour stops, opacity stops, Angle, a Location per stop or midpoint [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | partial | `Gradient` paint: `linear` (`angle`) or `radial` (`center`, `radius`), `stops` with `offset` and `#RRGGBBAA` colour, all keyable (ADR-0149) | Midpoint controls between stops (route: an extra stop). A gradient on one run or highlight is refused (ADR-0149, ADR-0146). |
| Gradient stroke | Gradient as stroke paint [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | have | `stroke` is a `Paint` on text and shapes (ADR-0149) | — |
| Gradient shadow | Gradient as shadow paint [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | missing | ADR-0149: "A gradient on `shadow.color`. Deferred, not refused: Premiere has it" | A gradient `shadow.color`. |
| Outer stroke | Stroke position Outer, drawn behind the fill [C] | https://helpx.adobe.com/ca/premiere-pro/using/work-with-fill-stroke-background-shadow.html | have | On text the stroke falls outside the glyph contour and grows into the declared box (ADR-0014) | — |
| Inner and Centre stroke | Stroke position Inner (over the fill) or Center (over the fill on shapes, under it on text) [C] | https://helpx.adobe.com/ca/premiere-pro/using/work-with-fill-stroke-background-shadow.html | partial | The position is fixed by element type: text outside, `rect`/`ellipse` inside the declared rect (ADR-0014), `path` centred (ADR-0154 §4) | A choosable stroke position on one element. |
| Multiple strokes | + icon in Appearance adds strokes [C] | https://helpx.adobe.com/ca/premiere-pro/using/work-with-fill-stroke-background-shadow.html | partial | One `stroke` per element; a second outline is a second element written out (ADR-0148 §1: copies are written out) | Stacked strokes on one element. |
| Stroke width | Stroke width [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | `stroke_width`, keyable on text and shapes (ADR-0146 §3); per run, static | — |
| Stroke line join and miter limit | Stroke styles (wrench icon) include line join; Graphics preferences: Miter join with a Miter limit, or Round join [C] | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | `stroke_join` (`round`, `bevel`, `miter`) and `stroke_miter_limit` on `path` (ADR-0158) | A join choice on text strokes. On `rect`/`ellipse` it is refused (ADR-0158, Considered options). |
| Text background | Background checkbox: fill colour, Opacity, Size, Corner Radius (0–100) [C] | https://helpx.adobe.com/ca/premiere-pro/using/work-with-fill-stroke-background-shadow.html | partial | A `rect` with `radius` and an alpha `fill` on the layer below (ADR-0014 and ADR-0040 relocate the background box to a `rect`); `captions.py` in `montagent-footage` writes a caption `pill` with `pad` | A background that follows the text's extent by itself. The agent sizes the `rect` from `measure` or `query`. |
| Shadow | Shadow checkbox: opacity, angle, distance, size, blur; per layer, not per character [C] | https://helpx.adobe.com/ca/premiere-pro/using/work-with-fill-stroke-background-shadow.html | partial | `shadow{dx, dy, radius, color, opacity}` on the whole element, every parameter keyable (ADR-0040, ADR-0146 §3) | Shadow `size` (spread). Angle and distance are written as `dx`/`dy`. |
| Multiple shadows | + icon adds shadows [C] | https://helpx.adobe.com/ca/premiere-pro/using/work-with-fill-stroke-background-shadow.html | have | Two `effects` members of the same name are ordinary, applied in list order (CONTEXT, effects) | — |

### Masking with text and shapes

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Mask with Text | Appearance > Mask with Text: the text layer becomes a window onto the layers beneath it; Mask Only Fill; Invert [C][P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/mask-with-text-or-shape-using-properties-panel.html | refused | ADR-0152 (a text or shape source is refused inside `mask`); ADR-0150 (a matte from another element is refused) | — |
| Mask with Shape | Appearance > Mask with Shape; animating layers under it gives a reveal [C][P] | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-reveal-animations-using-masking-techniques.html | partial | Each element's own `mask` effect: `circle`, `rect`, `ellipse` or a closed `path`, keyable, with `invert` and `feather` (ADR-0084, ADR-0152, ADR-0163) | One shape masking several layers at once; ADR-0150 refuses that by taking no matte from another element. Route: the same `mask` written on each element. |
| Invert the mask | Invert [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/mask-with-text-or-shape-using-properties-panel.html | have | `mask.invert` (ADR-0152, ADR-0165) | — |

### Shapes

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Rectangle | Rectangle tool [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | `rect`, with `width`, `height`, `radius` keyable (ADR-0146 §3) | — (one radius for all four corners, schema `radius`) |
| Ellipse | Ellipse tool [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | `ellipse` | — |
| Polygon | Polygon tool [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | A closed `path` with its vertices written in integer pixels (ADR-0154) | — (no side-count parameter; the vertices are written out) |
| Pen tool and Bezier paths | Pen tool draws custom shapes from vertex points, direction lines and Bezier curves [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-with-pen-tool.html | have | `path` with `points` of `at`, `in`, `out` handles; open or `closed` (ADR-0154) | — |
| Straight line | Pen tool, two vertex points [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-straight-line.html | have | An open `path` of two vertices, with `stroke_cap` (ADR-0154, ADR-0158) | — |
| Shape appearance | Fill, Stroke and Shadow on a shape layer [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | `fill`, `stroke`, `stroke_width`, `effects` `shadow` on `rect`, `ellipse`, `path` | — |
| Reshape a drawn shape over time | Keyframing a shape layer's path: not found in Premiere's pages; masks reshape per frame [P] | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-masks-at-the-clip-or-frame-level.html | have | Keyed `points` as one whole list, morph by matching vertex lists (ADR-0154 §3, ADR-0162) | — |

### Layers, transform and animation of graphics

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Transform a layer | Align and Transform: Position, Anchor Point, Scale, Rotation, Opacity [C][P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html | partial | `x`, `y`, `scale` (two components), `rotation`, `opacity`; the pivot is `origin` (ADR-0012, ADR-0013); plus `swivel`, `tilt`, `perspective` (ADR-0167) | An arbitrary anchor point: the pivot is one of nine `origin` keywords (ADR-0013). |
| Keyframe layer properties | Keyframes on a graphic layer's Position, Anchor Point, Scale, Rotation, Opacity [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html | have | Keyframe lists on every transform field, with required `ease` (ADR-0012, ADR-0038, ADR-0146) | — |
| Align and distribute layers | Align controls in Align and Transform [T] | https://helpx.adobe.com/premiere-pro/using/edit-text-using-the-properties-panel.html | partial | `origin` keywords put an edge or centre at `x`,`y` (ADR-0013); `montagent-craft` gives the safe-area boxes in pixels | An align or distribute command. The agent computes each literal. |
| A graphic holding several layers | One graphic clip in the timeline holds text, shape and clip layers [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/different-ways-to-work-with-graphics.html | partial | A flat list of elements on tracks (ADR-0001, ADR-0004) | One clip that trims, moves and fades as a unit. A nest with its own transform has precedent (ADR-0166) and is not built; "parenting, groups and cameras" is listed as not yet possible (`montagent` skill, #499). |
| Group layers | Group text and graphic layers [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/group-text-and-graphic-layers.html | partial | The render-inert `group` string, searchable with `query --where "group = …"` (ADR-0001, ADR-0070) | A group transform or opacity, and collapsing a group. |
| Clip layers | Still images and video added as layers inside a graphic [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-a-clip-layer.html | have | `image` and `video` elements on any layer (ADR-0001) | — |
| After Effects compositions | Dynamic Link places a "live" After Effects composition; Render and Replace bakes it [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/different-ways-to-work-with-graphics.html | partial | A composition rendered to a file is a `video` element; a code-drawn piece is pre-rendered with alpha through `montagent-prerender` (ADR-0156 §7) | A live link that re-renders on change. |

### Responsive Design and rolling credits

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Responsive Design: pin to video frame | A layer adapts to changes in the frame's aspect ratio [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html | refused | ADR-0012 refused fractions for placement: placement is integer pixels, so a retarget is a re-layout, not a stretch | — |
| Responsive Design: pin to another layer | A layer follows the position or scale of another graphic layer, e.g. a box that grows with its text [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html | missing | No reference between elements' geometry. ADR-0012 retired `box:"<id>"`; ADR-0145's literal-values test fails "a reference to another value"; parenting is on #499 | Pinning or parenting. Any design has to pass ADR-0145's literal-values test. |
| Responsive Design: time (intro and outro) | Protected intro and outro regions keep their keyframes when the clip is lengthened [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html | partial | `shift` (MCP) moves every time at or after an instant, so inserting time in the middle carries the outro's keyframes (CONTEXT "Shift", ADR-0047) | Keyframes that follow an element's `end` at render. Times are absolute integer milliseconds (ADR-0005). |
| Rolling credits | Roll option in the Properties panel for vertically scrolling titles [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html | partial | A tall `text` element (or several) with linear `y` keyframes (ADR-0012); `montagent-craft`'s "read while moving" speed limit | Speed derived from the duration; no credits recipe in `montagent-motion`. |
| Roll options | Start off screen, end off screen, preroll, ease-in, ease-out, postroll [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html | partial | First and last keyframe values and instants, `ease` on each keyframe (ADR-0012, ADR-0038) | Named options. The agent computes the off-screen positions and the eased segments. |

### Text styles

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Create a text style | Save font, colour, size and other Text and Appearance settings as a style in the project [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | missing | The `fonts` table's semantic keys are the only shared, named value (ADR-0007) | Named style objects. ADR-0002's case against indirection in the file bears on any design. |
| Apply a style to many layers | One style applied across several text layers [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | partial | `query --where` finds every element sharing a written value, e.g. `runs.*.font = brand` (ADR-0070); the agent edits each | A single edit that restyles them all. |
| Linked styles (titles) and track styles (captions) | Keep titles and captions consistent across a project [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-linked-and-track-styles.html | missing | — | Linked instances. Track styles for captions are #888's. |
| Redefine a style | Redefine Style pushes changes to all linked instances [C] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/redefine-linked-and-track-styles.html | missing | — | Propagation of a style edit. |
| Apply linked and track styles | Apply a linked style to a title or a track style to a caption track [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-linked-and-track-styles.html | missing | — | As above. |
| Style browser parameters | Which parameters a style applied from the style browser carries (Tracking, Kerning and others) [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/style-parameters-when-applying-from-style-browser.html | missing | — | A style browser. |
| Graphics preferences | Defaults for new graphics, e.g. Miter or Round join and the Miter limit [C] | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | missing | Defaults are fixed by the schema (`line_height` 1.2, `stroke_join` round, `stroke_cap` butt); an omitted field and an explicit default are different documents (ADR-0030) | User-set defaults. |

### Motion Graphics templates and legacy titles

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Use a Motion Graphics template (`.mogrt`) | After Effects motion graphics "packaged as templates with easy-to-use controls designed to be customized in Premiere" [P] | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html | missing | No template source type. CONTEXT lists "mogrt" as a term to avoid for pre-rendered footage; `montagent-prerender` bakes an outside piece to footage (ADR-0156 §7, ADR-0171) | A live template on the timeline. An After Effects-authored template carries an After Effects project and possibly expressions, which ADR-0145's closed-vocabulary test calls programs. |
| Template editable properties | Text, font, colour and media controls exposed in the Properties panel; the options depend on the template [C] | https://helpx.adobe.com/premiere-pro/how-to/essential-graphics-panel.html | partial | Spec-driven generator scripts in skills write a piece from parameters: `type_on.py` (`montagent-motion`), `captions.py` (`montagent-footage`), `beat_grid.py` (`montagent-craft`) | Parameters inside the file. A control that drives values elsewhere is "a reference to another value" under ADR-0145. |
| Browse and install templates | Graphics templates browser in the Essential Graphics panel; Adobe Stock templates [T] | https://helpx.adobe.com/premiere-pro/how-to/essential-graphics-panel.html | missing | Skill recipes and references (`launch-spot.md`, `logo-loop.md`, `trailer.md`) are written instructions, not assets | A template library and browser. |
| Export a graphic as a Motion Graphics template | Export a graphic built in Premiere as a `.mogrt` [T] | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html | missing | — | A packaged, reusable graphic. |
| Legacy titles, upgraded to source graphics | Legacy Titler text is upgraded with Graphics and Titles > Upgrade to Source Graphic [C] | https://helpx.adobe.com/be_en/premiere/desktop/add-text-images/insert-images-and-graphics/create-source-graphics.html | missing | — | Reading a Legacy Title or a Premiere graphic. Interchange is #892's. |
| The Properties panel as the editing surface | The Properties panel replaced the Essential Graphics panel from 25.0 [C] | https://helpx.adobe.com/lu_en/premiere-pro/using/about-properties-panel.html | have | The project file is the editing surface; the `montagent` skill's capability map and `montagent://format.md` say which key spells each property | — (no GUI by design: ADR-0011's tool surface) |

## Premiere items not confirmed from an Adobe page

Premiere features related to this area that no Adobe page reached here confirmed. Each is
**RECALLED** and needs an Adobe source before it becomes a row. Montagent's side is given so it
is not lost.

| Recalled Premiere feature | Montagent v0.2.0 |
|---|---|
| Variable fonts (axis sliders) | missing: ADR-0007 records variable axes as "deferred, not forgotten" |
| South Asian and Middle Eastern text engine (RTL, Indic shaping) | have: `dir` runs as isolates, line direction from its own characters (ADR-0007, ADR-0133); joining scripts (ADR-0153); Thai breaks by the agent (ADR-0008) |
| Vertical text | missing: ADR-0007 defers vertical writing modes ("deferred but not precluded") |
| Strikethrough | missing |
| Spell check on graphic text | missing |
| Crawl (horizontal scroll), a Legacy Titler option | partial: linear `x` keyframes; `montagent-craft` gives the speed limit |
| Data-driven Motion Graphics templates (spreadsheet) | missing |
| Font favourites and filtering in the font picker | missing (GUI) |
| Line cap on a stroke | have on `path` (`stroke_cap`, ADR-0158) |
| Per-corner rectangle radius | partial: one `radius` for four corners (schema) |
| Auto-sizing text box | refused: ADR-0007 (no fit-to-box) |

## Montagent mechanisms Premiere's pages do not show

For the assembly ticket's reference, not counted: text along an arbitrary inline path and
sliding along it (`path`, `path_offset`, ADR-0161, ADR-0164); keyable text paint, stroke width
and gradients (ADR-0146, ADR-0149); per-letter, per-word and per-line staggers with one unit
singled out (ADR-0151); trim paths and dash patterns on every shape (ADR-0158, ADR-0160); shape
morphs (ADR-0162); a timed per-word highlight (ADR-0048).

## Notable gaps

- **Text styles, linked styles and Redefine Style are missing.** Restyling many titles is one
  edit per element, found with `query --where`.
- **Motion Graphics templates are missing** as live, editable templates. The only routes are
  baking to footage (`montagent-prerender`) and spec-driven generator scripts in skills.
  In-file template parameters would have to pass ADR-0145's literal-values test.
- **Responsive Design is mostly out.** Pin to frame is refused (ADR-0012). Pin to layer is
  missing and waits on parenting (#499). Time-based intro and outro is partial, through `shift`
  at authoring time.
- **The text background is partial:** a `rect` that the agent sizes from `measure`, not a
  background that follows the text.
- **Character controls:** manual kerning, baseline shift, super/subscript, small caps, tab
  stops, Tsume, justification, faux italic and a run-level underline are partial or missing.
  Faux bold is refused (ADR-0007).
- **Stroke and shadow:** one stroke per element in a fixed position, no shadow spread, and no
  gradient shadow (deferred by ADR-0149).
- **Masking:** Mask with Text is refused (ADR-0150, ADR-0152). Mask with Shape exists only as
  each element's own mask.
- **Colour fonts and variable fonts** are not supported or not verified.
- **Credits:** a roll is a hand-keyed `y` move, with no recipe in `montagent-motion`.
