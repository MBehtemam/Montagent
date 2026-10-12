# Premiere Pro titles, text, graphics and Motion Graphics templates, set against Montagent v0.2.0

Research for [#887](https://github.com/MBehtemam/Montagent/issues/887), re-sourced for
[#911](https://github.com/MBehtemam/Montagent/issues/911), part of the map
[#882](https://github.com/MBehtemam/Montagent/issues/882). Area slug: **titles**.

- **Date:** 2026-10-10. Nothing was built or run. **This note decides nothing.** It lists what
  Premiere has and what Montagent v0.2.0 has for each item.
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, = `origin/main`). Sources: `CONTEXT.md`,
  `schema/montagent.schema.json`, the ADRs cited per row, the MCP tool surface
  (`crates/montagent/src/mcp.rs`: `where`, `validate`, `create_project`, `query`, `frame`,
  `render`, `verify`, `preview`, `measure`, `shift`, `compare`), the CLI's `fonts list` and
  `fonts vendor`, and the skills `montagent`, `montagent-motion` (with
  `scripts/type_on.py`), `montagent-craft`, `montagent-footage`, `montagent-prerender` and
  `montagent-character` (with `scripts/bake_rig.py`).
- **Premiere side:** Adobe's own Premiere user guide at `helpx.adobe.com/premiere/desktop/`
  (Premiere 26.5), every page read in full. No third-party source is used.
- **Counts (one table, 111 rows):** have **37** · have (skill) **0** · partial **38** ·
  missing **30** · refused **6**. 103 rows are sourced (F). **8 rows are marked unconfirmed**:
  no page in Adobe's guide states the Premiere feature (auto-sizing text box, justify,
  strikethrough, vertical text, variable fonts, font favourites, per-corner radius, crawl).
  They are kept and counted, not dropped. 3 more rows are (F) but name details no page
  confirms (text background and shadow parameters, keyframing a shape layer's path).
- **Before #911:** 74 rows (29 have · 23 partial · 17 missing · 5 refused), with 11 recalled
  items listed apart and not counted. All 11 are now counted rows; 26 more rows come from
  features the pages describe that the first pass lacked.

## How this was read

Every Premiere cell was re-sourced for #911 from the census download of the guide (#883):
807 pages fetched in full from `helpx.adobe.com/premiere/desktop/`, Premiere 26.5. This area's
48 pages, and the three What's-new pages, were read in full. The rest of the guide was searched
for features that might be documented elsewhere; where a row cites a page outside this area
(Graphics preferences, Transitions, Technical requirements, the masking pages), that is why.
Legacy URLs the first pass cited were opened live: the old fill/stroke/background/shadow page
and the old edit-text page now redirect to `create-text-styles.html` and `create-titles.html`,
and the Essential Graphics page now redirects off helpx to `adobe.com/learn`.

Each Premiere mechanism is tagged:

- **(F)** the cited page was fetched and read in full, and states the mechanism as written.
- **unconfirmed** no page in Adobe's guide states it. The mechanism is the earlier pass's
  recall or search excerpt, kept so the row is not lost. Re-check it before relying on it.

Statuses follow the map: **have** (the mechanism ships in v0.2.0), **have (skill)** (a bundled
skill provides it, not the format), **partial** (some of it ships; the last column says what does
not), **missing** (nothing equivalent, and no ADR refuses it), **refused** (an ADR or ADR-0145's
four invariants rule it out; cited). Read by mechanism, not by name (ADR-0145): a Premiere
"graphic clip" is compared with Montagent elements, not with a Montagent type of that name.

## The table

### Creating and editing text

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Create a title (point text) | Type tool, type in the Program Monitor; right-click > Edit Properties opens the Properties panel (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-titles.html | have | `text` element with `runs`, `font`, `size`, `width`, `height` (ADR-0007, ADR-0014); `create_project` MCP tool scaffolds the file | — (the agent writes JSON; the MCP surface reads, checks and renders, ADR-0011) |
| Paragraph text in a text box | With the Type tool, "draw a text box to create a paragraph text" (F). How text flows inside the box is not stated on any page | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-color-fonts.html | refused | ADR-0007 ("No fit-to-box. No automatic wrapping."), ADR-0008 (line breaks belong to the agent); `measure` returns the extent | — (route: the agent writes `\n` and checks with `measure`) |
| Auto-sizing text box | A text box that grows or shrinks to its text. **unconfirmed**: no page states it | — | refused | ADR-0007 ("No fit-to-box") | — |
| Style part of a text layer | Select part of the text, set font, colour and size, or pick a style thumbnail; only the selection changes (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-style.html | have | `runs`: each run carries style deltas `font`, `size`, `color`, `stroke`, `stroke_width`, `dir` (ADR-0007, ADR-0014) | — |
| Edit text in the Graphics tab | The Text panel's Graphics tab lists all text in the sequence, from Premiere graphics and After Effects templates, with a thumbnail, timecode range and video track; double-click to edit; filter by track (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/edit-text-in-graphics-tab.html | have | Every element is one line, so its text is a unique exact-string edit (ADR-0005, ADR-0007); `query --where "type = text"` lists every text with its track and times (ADR-0070) | — (no thumbnail list; `frame` or `preview` renders one on request) |
| Find and replace text | Graphics tab Search field with Previous and Next; Replace or Replace All (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/find-and-replace-text-using-the-graphics-tab.html | have | The project is plain JSON text, one element per line (ADR-0005), so a search and replace over it is an ordinary edit; `query` lists every text element (ADR-0070) | — (no replace tool on the MCP surface; the agent edits the file) |
| Spell check | Edit > Spelling > Check Spelling underlines misspelled words in red, offers suggestions, and learns or forgets words in an OS-wide dictionary; language from the OS (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/check-spelling-using-the-graphics-tab.html | missing | `validate` checks glyph coverage (`.notdef`), not spelling (ADR-0007) | A spelling check over text elements. |
| Text animation presets (Text Animator, Typewriter) | Text Animator: "Animate titles and text with professional motion presets, without the need for complex keyframing"; Typewriter Impacts reveals text "letter by letter" (F); 26.0 added drag-and-drop animation tools (What's new) | https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html | have | `units` stagger by `letter`, `word` or `line`, with per-unit `unit` overrides (ADR-0148, ADR-0151, ADR-0153); `type_on.py` in `montagent-motion` for typing with a cursor | — (no named preset list; the move is written as keyframes) |
| Emojis | Edit > Emoji & Symbols inserts an emoji into a text layer; macOS draws Apple Color Emoji, Windows' Segoe UI Emoji renders black and white; a default emoji font can be set (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-emojis.html | partial | An emoji is a character in a run, drawn from whatever face the `fonts` chain declares (ADR-0007); a pictograph counts as a word unit (CONTEXT, ADR-0153) | Colour emoji, which waits on colour-glyph support (see Colour fonts, unverified). A default emoji font: the chain is written per font key. |
| Smart quotes | Graphics preferences > Text: "Enable smart quotes in your text" (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | have | A run's `text` holds the exact typographic character the agent writes (ADR-0007) | — (no input-time conversion is needed) |

### Character and paragraph properties

The Text section's parameters, as Adobe's two style-parameter pages list them (F): Font, Font
Style, Font Size, Paragraph Alignment, Tracking, Kerning, Leading, Baseline Shift, Tab Width,
Faux Bold, Faux Italics, All Caps, Small Caps, Superscript, Subscript, Underline, Tsume.
Kerning, Baseline Shift and Tab Width apply to text layers only, not captions.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Font | Font (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | `font` is a key into the top-level `fonts` table, an ordered fallback chain of project-relative files (ADR-0007, ADR-0057, schema `FontFile`) | — (by design, never an installed family name) |
| Font Style (weight, italic face) | Font Style (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | A face is its own file, chosen per run by `font`; `.ttc` faces by `index` (ADR-0007, ADR-0102); `weight`/`bold` are rejected naming the chain (ADR-0014) | — |
| Font Size | Font Size (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | Literal integer `size` on the element and on each run (ADR-0007) | — (`size` is not keyable; ADR-0146 §3 admits text paint only, and `scale` animates drawn size) |
| Paragraph alignment | Paragraph Alignment (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | `align`: `start`, `center`, `end`, resolved per line's own direction (ADR-0007, ADR-0133, ADR-0135) | — |
| Justified alignment | Justify options among the paragraph alignments. **unconfirmed**: the pages name only "Paragraph Alignment" | — | missing | `Align` has three values (schema) | Justified lines (space distributed to the block width). No ADR refuses it. |
| Tracking | Tracking (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/style-parameters-when-applying-from-style-browser.html | have | `letter_spacing` in thousandths of an em, keyable (ADR-0151); joining scripts take none (ADR-0153) | — (element-level only, not per run) |
| Kerning | Kerning, text layers only (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | The font's own kerning is applied by the shaper (ADR-0010; `type_on.py` places units "kerning included") | Manual kerning of one letter pair. `letter_spacing` cannot be written on a run (schema). |
| Leading | Leading (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | have | `line_height`, a multiplier in exact tenths, half-leading baseline (ADR-0007, ADR-0028, ADR-0029) | — (no finer step than 0.1 and no per-line value, both by ADR-0028's exact arithmetic) |
| Baseline Shift | Baseline Shift, text layers only (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | A `units` block plus a run's `unit` override can offset one letter or word in `y` (ADR-0151 §4) | A plain per-run baseline offset. Every run on a line sits on one shared baseline (ADR-0029). |
| Tab Width | Tab Width, text layers only (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | missing | No tab field | Tab stops and tab width. Columns are separate elements placed by the agent. |
| Faux Bold | Faux Bold (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | refused | ADR-0007 ("No `weight`, no `bold`": synthetic emboldening is renderer-specific; the schema must reject it); ADR-0014 | — (route: vendor a bold file) |
| Faux Italics | Faux Italics (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | missing | A true italic face works through a run's `font` (ADR-0007) | Synthetic slant. ADR-0007's refusal names bold and weight only, so this is not refused by name. |
| All Caps | All Caps (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | Capitals written literally in `runs.text` | A case transform that keeps the source casing in the file. |
| Small Caps | Small Caps (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | Faux small caps as capital runs at a smaller `size` (ADR-0007) | The font's real small caps (`smcp`), and any OpenType feature switch: only ligature handling is fixed, by rule (ADR-0151, ADR-0153). |
| Superscript and Subscript | Superscript, Subscript (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | A smaller run `size`; a vertical offset only through `units`/`unit` (ADR-0151 §4) | A raised or lowered run, and the `sups`/`subs` features. |
| Underline | Underline (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | A separate `rect` or open `path` under the text; `montagent-motion` recipes "Grow from an edge" and "Draw a line on" draw and animate one; `query --at` gives the ink box | An underline that is a text or run attribute and follows the glyphs when the text changes. |
| Strikethrough | A line through the text. **unconfirmed**: absent from Adobe's own list of Text parameters | — | missing | — (a `rect` or `path` drawn over the text, as for Underline) | A strikethrough attribute. |
| Tsume | Tsume (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | missing | — | Tsume. |
| Ligatures on or off | Graphics preferences > Text: "Enable ligatures in your text" (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | Ligatures are fixed by rule: optional ligatures are off when any spacing is non-zero, and joining scripts keep theirs (ADR-0151 §1, ADR-0153 §3) | A switch to turn optional ligatures on or off. |
| Hindi digits | Graphics preferences > Text: "Enable support for Hindi numerals in your text" (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | Devanagari digits are written literally as characters in `runs.text` (ADR-0007) | A display switch that keeps the source digits in the file. |
| Default paragraph direction | Graphics preferences > Text: left to right or right to left (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | A line's base direction is the one its own characters give; a run's `dir` is an isolate and never sets it (ADR-0133) | A settable base direction, for a line whose first strong character disagrees with the paragraph. |
| Middle Eastern and Indic text (RTL, Indic shaping) | "Premiere supports text composition in Middle Eastern and Indic scripts using the South Asian and Middle Eastern text engines" (F); 26.x fixed glyph-boundary selection in Arabic, Sinhala and Tamil (release notes) | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/adobe-premiere-pro-technical-requirements.html | have | `dir` runs as isolates, line direction from its own characters (ADR-0007, ADR-0133); joining scripts (ADR-0153); Thai breaks placed by the agent (ADR-0008) | — |
| Vertical text | Vertical writing mode. **unconfirmed**: no page states it | — | missing | ADR-0007 defers vertical writing modes ("deferred but not precluded") | Vertical text. |
| Variable fonts | Axis sliders for a variable font. **unconfirmed**: no page states it | — | missing | ADR-0007 records variable axes as "deferred, not forgotten" ("No variable-axis fields in v1") | Variable-axis values. |

### Fonts

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Colour fonts | Pick a colour font in the Properties panel's font picker; switch between its styles in the font-style picker (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-color-fonts.html | missing | No ADR, schema key or test names COLR/CPAL, sbix, CBDT or SVG-in-OpenType | Colour glyphs. Whether the painter draws them today is undetermined; it needs a measured test before it can count. |
| Replace fonts in project | Graphics and Titles > Replace Fonts in Project lists the fonts used; choose a Replacement Font for each (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html | have | One edit to a `fonts` chain re-points every element using its key (ADR-0007); the font-swap census reports what the swap moved (ADR-0085) | — |
| Adobe Fonts | A new font must be "installed on your computer or activated through Adobe Fonts" (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html | refused | ADR-0007 (a font is a file the project declares, never a system family; the renderer never consults system fonts); ADR-0057 (vendoring behind a licence gate) | — (route: `fonts vendor` a licensed file) |
| Missing fonts | A template shows offline until loaded; "If the template requires fonts that have not been installed, you can resolve the fonts that are missing" (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/add-motion-graphic-templates-to-a-sequence.html | have | `validate`: a missing font file, or a character with no glyph in any chain entry (`.notdef`), is an error (ADR-0007); `fontVendor` hashes are checked (ADR-0057) | — |
| Missing font replacement | Graphics preferences > Text: a custom replacement font, used when a template's fonts cannot be synced (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | Each `fonts` key is an ordered fallback chain, so a later entry draws a glyph an earlier face lacks (ADR-0007) | Substitution for a missing font file: `validate` makes that an error instead (ADR-0007). |
| Font discovery on the machine | Fonts installed on the computer or activated through Adobe Fonts appear in the font pickers (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html | have | `montagent fonts list` walks the machine's fonts with licence status; `fonts vendor` copies one in (ADR-0057). CLI only, not an MCP tool | — |
| Font favourites and filtering | Favourites and filters in the font picker. **unconfirmed**: no page states it (favourites exist for templates, not fonts) | — | missing | — (no font picker; the agent reads `fonts list`) | Font favourites and filters (GUI). |

### Appearance: fill, stroke, background, shadow, gradients

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Solid fill | Appearance > Fill; Solid in the colour picker (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | have | `color` on text, `fill` on shapes, `#RRGGBBAA`; keyable on the element (ADR-0146 §3) | — (a run's colour is static; timed run colour is a `highlight`, ADR-0048) |
| Linear and radial gradient | Fill, Stroke or Shadow colour picker: Solid, Linear Gradient or Radial Gradient; colour stops added and removed; opacity stops with an Opacity %; Angle; a Location for a stop or midpoint (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | partial | `Gradient` paint: `linear` (`angle`) or `radial` (`center`, `radius`), `stops` with `offset` and `#RRGGBBAA` colour, all keyable (ADR-0149) | Midpoint controls between stops, and opacity stops placed apart from colour stops (route for both: extra stops). A gradient on one run or highlight is refused (ADR-0149, ADR-0146). |
| Gradient stroke | Gradient chosen from the Stroke colour picker (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | have | `stroke` is a `Paint` on text and shapes (ADR-0149) | — |
| Gradient shadow | Gradient chosen from the Shadow colour picker (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | missing | ADR-0149: "A gradient on `shadow.color`. Deferred, not refused: Premiere has it" | A gradient `shadow.color`. |
| Outer stroke | Stroke: "Add Outer, Inner, or Center strokes" (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | On text the stroke falls outside the glyph contour and grows into the declared box (ADR-0014) | — |
| Inner and Centre stroke | Stroke position Inner or Center, on text and shapes (F). How each sits against the fill is not stated | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | partial | The position is fixed by element type: text outside, `rect`/`ellipse` inside the declared rect (ADR-0014), `path` centred (ADR-0154 §4) | A choosable stroke position on one element. |
| Multiple strokes | "add multiple strokes" (F); 26.x fixed adding a second stroke while typing (release notes) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | partial | One `stroke` per element; a second outline is a second element written out (ADR-0148 §1: copies are written out) | Stacked strokes on one element. |
| Stroke width | Stroke under Appearance, increase the width (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-straight-line.html | have | `stroke_width`, keyable on text and shapes (ADR-0146 §3); per run, static | — |
| Stroke line join and miter limit | Miter join with a Miter limit, Round join or Bevel join, set separately for text and shapes (F); a style carries "Stroke including Stroke Styles" | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | `stroke_join` (`round`, `bevel`, `miter`) and `stroke_miter_limit` on `path` (ADR-0158) | A join choice on text strokes (round, fixed: ADR-0158 §1) and on `rect`/`ellipse` (a schema error there, ADR-0158 §1). |
| Stroke line cap | Butt, Round or Square cap, set separately for text and shapes (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | `stroke_cap` (`butt`, `round`, `square`) on `path` (ADR-0158) | A cap on `rect`/`ellipse` dashes (fixed butt) and on text (takes none), both by ADR-0158 §1. |
| Text background | A style carries "Background including Background Styles" (F). The parameters (opacity, size, corner radius 0–100) are **unconfirmed**: the page that gave them now redirects | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | partial | A `rect` with `radius` and an alpha `fill` on the layer below (ADR-0014 and ADR-0040 relocate the background box to a `rect`); `captions.py` in `montagent-footage` writes a caption `pill` with `pad` | A background that follows the text's extent by itself. The agent sizes the `rect` from `measure` or `query`. |
| Background per line or whole block | Graphics preferences > Text > Background fill: All lines, or Per-line (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | partial | One `rect` for the block, or one per line, each sized by the agent from `measure` (ADR-0007, ADR-0014) | A background mode on the text element. |
| Shadow | "Add shadows or multiple shadows" (F). The parameters (opacity, angle, distance, size, blur) are **unconfirmed**: the page that gave them now redirects | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | partial | `shadow{dx, dy, radius, color, opacity}` on the whole element, every parameter keyable (ADR-0040, ADR-0146 §3) | Shadow size (spread), if Premiere has it (unconfirmed). Angle and distance are written as `dx`/`dy`. |
| Multiple shadows | "Add shadows or multiple shadows" (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | have | Two `effects` members of the same name are ordinary, applied in list order (CONTEXT, effects) | — |

### Masking with text and shapes

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Mask with Text | Appearance > Mask with Text: the text layer reveals the video or graphics beneath only within its letters; Mask Only Fill leaves strokes and shadows out of the mask; Invert (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/mask-with-text-or-shape-using-properties-panel.html | refused | ADR-0152 (a text or shape source is refused inside `mask`); ADR-0150 (a matte from another element is refused) | — |
| Mask with Shape | Appearance > Mask with Shape: a shape layer becomes a window onto the layers beneath; animating a layer under it gives a reveal (F) | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-reveal-animations-using-masking-techniques.html | partial | Each element's own `mask` effect: `circle`, `rect`, `ellipse` or a closed `path`, keyable, with `invert` and `feather` (ADR-0084, ADR-0152, ADR-0163) | One shape masking several layers at once; ADR-0150 refuses that by taking no matte from another element. Route: the same `mask` written on each element. |
| Invert the mask | Invert: everything outside the layer visible, everything inside transparent (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/mask-with-text-or-shape-using-properties-panel.html | have | `mask.invert` (ADR-0152, ADR-0165) | — |
| Pen mask that limits an effect | Draw a shape with the Pen tool, then apply effects "to the area outside or inside the object boundaries" (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-free-form-shapes-for-masking.html | partial | A `mask` with a closed `path`, and `invert`, keeps the element inside or outside the shape (ADR-0152) | An effect limited to the masked area (an effect-scoped mask; #885 counts it). |

### Shapes

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Rectangle | Rectangle tool, drag on the Program Monitor (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | `rect`, with `width`, `height`, `radius` keyable (ADR-0146 §3) | — |
| Ellipse | Ellipse tool (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | `ellipse` | — |
| Polygon | Polygon tool (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | A closed `path` with its vertices written in integer pixels (ADR-0154) | — (no side-count parameter; the vertices are written out) |
| Per-corner rectangle radius | A different radius on each corner. **unconfirmed**: the 26.0 rounded-corner control is for mask tools, not shapes | — | partial | One `radius` for all four corners (schema) | A radius per corner (route: a closed `path`). |
| Pen tool and Bezier paths | Vertex points, direction lines and Bezier curves; drag direction lines for C- and S-curves; Alt/Option-click turns a vertex into a Bezier point (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-with-pen-tool.html | have | `path` with `points` of `at`, `in`, `out` handles; open or `closed` (ADR-0154) | — |
| Straight line | Pen tool, two vertex points; keep clicking for connected straight segments (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-straight-line.html | have | An open `path` of two vertices, with `stroke_cap` (ADR-0154, ADR-0158) | — |
| Constrained angles while drawing | Shift constrains new points, moved points and Bezier handles to 0°, 45° and 90°; Shift+Alt breaks a handle into two curves (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-straight-line.html | have | Vertices and handles are literal integers (ADR-0154), so any angle is written exactly; `in` and `out` are independent handles | — |
| Shape appearance | Fill, Stroke (Outer, Inner, Center, multiple), Shadow on a shape layer (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | have | `fill`, `stroke`, `stroke_width`, `effects` `shadow` on `rect`, `ellipse`, `path` | — (stroke position and count: see the Appearance rows) |
| Reshape a drawn shape over time | Keyframing a shape layer's own path: **unconfirmed**, no page states it. Masks are reshaped per frame, creating keyframes (F) | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-masks-at-the-clip-or-frame-level.html | have | Keyed `points` as one whole list, morph by matching vertex lists (ADR-0154 §3, ADR-0162) | — |

### Layers, transform and animation of graphics

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Transform a layer | Align and Transform: Position, Anchor Point, Scale, Rotation, Opacity; the Selection tool changes width, height, rotation and anchor point (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html | partial | `x`, `y`, `scale` (two components), `rotation`, `opacity`; the pivot is `origin` (ADR-0012, ADR-0013); plus `swivel`, `tilt`, `perspective` (ADR-0167) | An arbitrary anchor point: the pivot is one of nine `origin` keywords (ADR-0013). |
| Uniform scale lock | Set Scale Lock in the Properties panel (F, named in a 26.x fix) | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html | have | `scale` has two components; uniform scale writes them equal (ADR-0012) | — |
| Keyframe layer properties | Enable animation for a property, move the playhead, set values; Stopwatch and Add/Remove Keyframe in the Effect Controls panel; Show Clip Keyframes in the timeline (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html | have | Keyframe lists on every transform field, with required `ease` (ADR-0012, ADR-0038, ADR-0146); `query` lists them | — |
| Keyframe interpolation | Right-click a keyframe for Bezier, Ease In or Ease Out (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-the-effect-controls-panel.html | have | `ease`: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`, `step`, or a cubic-bezier array (ADR-0012, schema `Ease`) | — |
| Align objects | By top, vertical centre, bottom, left, horizontal centre or right edge; Align to Video Frame, Align to Video Frame as Group, or Align to Selection (a key object) (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/align-and-distribute-objects/align-objects.html | partial | `origin` keywords put an edge or centre at `x`,`y` (ADR-0013); `query --at` gives each ink box; `montagent-craft` gives the safe-area boxes in pixels | An align command, to the frame, as a group or to a key object. The agent computes each literal. |
| Distribute objects | Space selected text or shapes equally, vertically or horizontally (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/align-and-distribute-objects/distribute-objects.html | partial | Positions are literal integers the agent computes; `query --at` gives the boxes to space | A distribute command. |
| Edit several layers at once | Select multiple clips; a shared value shows, differing values show a dash for relative adjustment (F); 26.x fixed moving several selected text layers together (release notes) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/about-properties-panel.html | partial | `query --where` finds every element sharing a value (ADR-0070); `shift` moves times relatively (CONTEXT "Shift") | A relative edit of a spatial or style value across several elements: the agent writes each new literal. |
| Guides and snapping | Add Guide and Edit Guide in the Program Monitor; snap one element to another (F, named in 26.x fixes) | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html | partial | `montagent-craft` gives safe-area boxes in pixels; `query --at` gives every element's box | Snapping to another element's edge. The agent computes it from `query`. |
| A graphic holding several layers | Still images and video added "as layers within your graphic" alongside text and shapes (F); a template's duration is trimmed as one clip (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-a-clip-layer.html | partial | A flat list of elements on tracks (ADR-0001, ADR-0004) | One clip that trims, moves and fades as a unit. A nest with its own transform has precedent (ADR-0166) and is not built; "parenting, groups and cameras" is listed as not yet possible (`montagent` skill, #499). |
| Group layers | Create Group keeps the Properties panel uncluttered, and "is useful when you want to create masking effects"; the group gets a default name (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/group-text-and-graphic-layers.html | partial | The render-inert `group` string, searchable with `query --where "group = …"` (ADR-0001, ADR-0070) | A group as the scope of a mask: a matte from another element is refused (ADR-0150). |
| Rename layers | Shape and clip layers can be renamed; a text layer's name is its text (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/rename-layers.html | have | Every element carries a required unique `id` (CONTEXT "Id", ADR-0019) | — |
| Clip layers | New Layer > From File adds a still or video as a layer in the graphic (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-a-clip-layer.html | have | `image` and `video` elements on any layer (ADR-0001) | — |
| Copy and paste a graphic | Copy a design element on the timeline, paste it elsewhere, modify the new instance (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/different-ways-to-work-with-graphics.html | have | An element is one JSON object on one line (ADR-0005); a copy is written with a new `id` (ADR-0148 §1: copies are written out) | — |
| After Effects compositions | Dynamic Link places a "live" After Effects composition; Render and Replace bakes it while keeping the original (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/different-ways-to-work-with-graphics.html | partial | A composition rendered to a file is a `video` element; a code-drawn piece is pre-rendered with alpha through `montagent-prerender` (ADR-0156 §7) | A live link that re-renders on change. |
| Render and Replace a graphic | Render & Replace a heavy graphic to a format with alpha so it moves freely; Restore Unrendered to edit it again (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/tips-for-authoring-motion-graphics-templates.html | partial | `montagent-prerender` bakes an outside piece to footage with alpha and keeps its source (ADR-0156 §7) | Baking a Montagent graphic itself to alpha footage: output is H.264 MP4 without alpha (#892). |
| Markers in the Effect Controls panel | Add Marker on the timeline ruler; markers show in the Effect Controls panel and mark comments, transitions and effect cue points (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-markers-in-the-effect-controls-panel.html | missing | No marker in the schema or CONTEXT (#884 counts timeline markers) | Markers. |

### Responsive Design and rolling credits

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Responsive Design: pin to video frame | A graphic adapts to changes in the frame's aspect ratio (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html | refused | ADR-0012 refused fractions for placement: placement is integer pixels, so a retarget is a re-layout, not a stretch | — |
| Responsive Design: pin to another layer | Pin To parents the selected layer to another; a diagram picks the pinned edges (top, bottom, left, right, or all); the child then follows the parent's position, length/width and scale; blue pins show the pinned edges (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html | missing | No reference between elements' geometry. ADR-0012 retired `box:"<id>"`; ADR-0145's literal-values test fails "a reference to another value"; parenting is on #499. `montagent-character`'s `bake_rig.py` bakes parent position and rotation into keyframes for image rigs only | Pinning or parenting, and any size that follows another element. Any design has to pass ADR-0145's literal-values test. |
| Responsive Design: time (intro and outro) | Intro and Outro durations, set in the Properties panel or by handles in the Effect Controls panel; keyframes inside them are kept when the graphic is trimmed, and keyframes between are stretched or compressed to fit (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/preserve-intro-outro-animations-while-creating-responsive-design-graphics.html | partial | `shift` (MCP) moves every time at or after an instant, so inserting time in the middle carries the outro's keyframes (CONTEXT "Shift", ADR-0047) | Keyframes that follow an element's `end` at render, and middle keyframes rescaled to fit. Times are absolute integer milliseconds (ADR-0005). |
| Rolling credits | Roll checkbox: title or credit rolls that move vertically; a scroll bar gives a Document View of the roll (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html | partial | A tall `text` element (or several) with linear `y` keyframes (ADR-0012); `montagent-craft`'s "read while moving" speed limit | Speed derived from the duration; no credits recipe in `montagent-motion`. |
| Roll options | Start or end offscreen; Preroll, Postroll, Ease in and Ease out as timecodes (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html | partial | First and last keyframe values and instants, `ease` on each keyframe (ADR-0012, ADR-0038) | Named options. The agent computes the off-screen positions and the eased segments. |
| Soften a roll | Soften checkbox "to smoothen the motion of your rolling text" (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html | partial | `motion_blur{shutter, samples}` on the element averages paints across the shutter (ADR-0155) | A soften option. The page does not say what Soften does, so whether motion blur matches it is unknown. |
| Crawl (horizontal scroll) | A horizontal title crawl. **unconfirmed** as a Premiere feature: the guide names title crawls only in Manifesto, a third-party plug-in | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html | partial | Linear `x` keyframes (ADR-0012); `montagent-craft` gives the speed limit | Named crawl options. |

### Text styles

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Create a text style | Only the Text and Appearance settings are a style; + icon > Create style; Save to Project links the layer and adds a project item (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | missing | The `fonts` table's semantic keys are the only shared, named value (ADR-0007) | Named style objects. ADR-0002's case against indirection in the file bears on any design. |
| Apply a style to many layers | Apply the same style "across multiple layers in different graphics" (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | partial | `query --where` finds every element sharing a written value, e.g. `runs.*.font = brand` (ADR-0070); the agent edits each | A single edit that restyles them all. |
| Linked styles (titles) and track styles (captions) | A linked style for titles and a track style for caption tracks keep them consistent (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-linked-and-track-styles.html | missing | — | Linked instances. Track styles for captions are #888's. |
| Apply linked and track styles | Pick a style in the Linked style dropdown; usage indicators count and jump to the graphics or caption tracks using it (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-linked-and-track-styles.html | missing | — (`query --where` finds equal values, but nothing records a link) | Applying a linked style, and a usage count. |
| Redefine a style | Redefine Style overwrites the style and updates every linked text layer or caption at once (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/redefine-linked-and-track-styles.html | missing | — | Propagation of a style edit. |
| Save a style for other projects | Save a style to a local folder; reuse it in later projects through the style browser (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-linked-and-track-styles.html | missing | — | A style library shared across projects. |
| Style browser parameters | A style applied from the Style Browser carries most Text parameters, all of Appearance, and Align and Transform for captions only (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/style-parameters-when-applying-from-style-browser.html | missing | — | A style browser. |
| User defaults for new graphics | Graphics preferences, applied to the next text or shape layer: default text, subtitle and caption fonts, stroke join and cap, background fill, emoji font (F) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html | missing | Defaults are fixed by the schema (`line_height` 1.2, `stroke_join` round, `stroke_cap` butt); an omitted field and an explicit default are different documents (ADR-0030) | User-set defaults. |

### Motion Graphics templates and legacy titles

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Use a Motion Graphics template (`.mogrt`) | A `.mogrt` file made in Premiere or After Effects, "packaged as templates with easy-to-use controls designed to be customized in Premiere"; drag from My Templates onto a track; trim to change its duration (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html | missing | No template source type. CONTEXT lists "mogrt" as a term to avoid for pre-rendered footage; `montagent-prerender` bakes an outside piece to footage (ADR-0156 §7, ADR-0171) | A live template on the timeline. An After Effects-authored template carries an After Effects project and possibly expressions, which ADR-0145's closed-vocabulary test calls programs. |
| Template editable properties | Colour, source text and animation options from choices the author provides; fonts, size, faux styles; sliders that pick a colour scheme or an icon (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/customize-motion-graphics-templates.html | partial | Spec-driven generator scripts in skills write a piece from parameters: `type_on.py` (`montagent-motion`), `captions.py` (`montagent-footage`), `beat_grid.py` (`montagent-craft`) | Parameters inside the file. A control that drives values elsewhere is "a reference to another value" under ADR-0145. |
| Media Replacement in templates | Swap a template's placeholder images, background videos or logos for your own while its animation stays (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/media-replacement-in-motion-graphics-templates.html | missing | — (a `source` is a literal path the agent can change, but nothing marks a placeholder) | Replaceable media slots in a template. |
| Data-driven templates (CSV/TSV) | Link a CSV or TSV to a template; a slider picks the row; columns are Text, Color or Numbers, typed by the author in After Effects; mapped data shows as disabled labels (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/use-data-driven-motion-graphics-templates.html | missing | — (a skill script could write elements from a table, but none ships) | Data-driven templates. |
| Install templates | Install Motion Graphics template icon, or drag several `.mogrt` files into My Templates; Creative Cloud Library templates need no install (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/install-motion-graphics-templates.html | missing | — | Template installation. |
| Browse, search, sort, favourite and tag templates | Graphics Templates panel: search, filters by location, sort by Recently Used or Title, hover-scrub previews, a Favorites filter, rename and tag in Info view (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/browse-and-sort-motion-graphics-templates.html | missing | Skill recipes and references (`launch-spot.md`, `logo-loop.md`, `trailer.md`) are written instructions, not assets | A template library and browser. |
| Organise and share templates | Copy to a Creative Cloud Library, shared with a team; move or copy between local folders; add local, cloud or network folders to My Templates (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/organize-motion-graphics-templates.html | missing | — | Template libraries and shared folders. |
| Templates from Adobe Stock | Search Adobe Stock in the Graphics Templates panel; license by dragging to the sequence or Buy License and Download (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/use-motion-graphics-templates-from-adobe-stock.html | missing | — | A template marketplace. |
| Export a graphic as a Motion Graphics template | Graphics and Titles > Export Motion Graphics Template packs a Premiere graphic, with its text and shape layers, effects and keyframes; not for After Effects templates or several graphics at once (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/export-graphic-as-a-motion-graphics-template.html | missing | — | A packaged, reusable graphic. |
| Legacy titles, upgraded to Source Graphics | Graphics and Titles > Upgrade to Source Graphic turns a Legacy Titler graphic into a Source Graphic project item (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-source-graphics.html | missing | — | Reading a Legacy Title or a Premiere graphic. Interchange is #892's. |
| Source Graphic instances | Every graphic made from a Source Graphic is an exact duplicate; a change to text, style or contents in one instance shows in all (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-source-graphics.html | missing | No shared source between elements: ADR-0002 (inline source, no asset table); ADR-0148 §1 (copies are written out) | Linked instances. Any design has to pass ADR-0145's literal-values test. |
| The Properties panel as the editing surface | From 25.0 the Properties panel holds what the Essential Graphics panel held; it shows the controls for what is selected, with quick actions such as opening Graphics Templates (F) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/about-properties-panel.html | have | The project file is the editing surface; the `montagent` skill's capability map and `montagent://format.md` say which key spells each property | — (no GUI by design: ADR-0011's tool surface) |

## Montagent mechanisms Premiere's pages do not show

For the assembly ticket's reference, not counted: text along an arbitrary inline path and
sliding along it (`path`, `path_offset`, ADR-0161, ADR-0164); keyable text paint, stroke width
and gradients (ADR-0146, ADR-0149); per-letter, per-word and per-line staggers with one unit
singled out (ADR-0151); trim paths and dash patterns on every shape (ADR-0158, ADR-0160); shape
morphs (ADR-0162); a timed per-word highlight (ADR-0048).

## Notable gaps

- **Text styles are missing:** no linked style, no Redefine Style, no style library across
  projects, and no user defaults. Restyling many titles is one edit per element, found with
  `query --where`.
- **Motion Graphics templates are missing** as live, editable templates, and so is everything
  around them: install, browse, libraries, Adobe Stock, Media Replacement, data-driven (CSV)
  templates, and export. The only routes are baking to footage (`montagent-prerender`) and
  spec-driven generator scripts in skills. In-file template parameters would have to pass
  ADR-0145's literal-values test.
- **Source Graphic instances are missing:** Premiere keeps every instance of a source graphic
  identical. Montagent writes copies out (ADR-0148 §1).
- **Responsive Design is mostly out.** Pin to frame is refused (ADR-0012). Pin to layer is
  parenting with edge pins, and is missing (#499). Intro and outro time is partial, through
  `shift` at authoring time; nothing rescales the middle keyframes.
- **Spell check is missing.** Find and replace is the agent's own edit of the file.
- **Text background is partial:** a `rect` that the agent sizes from `measure`, per line or for
  the block, not a background that follows the text.
- **Character controls:** manual kerning, baseline shift, super/subscript, small caps, tab
  stops, Tsume, justification, faux italic, a ligature switch and a run-level underline are
  partial or missing. Faux bold is refused (ADR-0007).
- **Stroke and shadow:** one stroke per element in a fixed position, joins and caps on `path`
  only, no shadow spread, and no gradient shadow (deferred by ADR-0149).
- **Masking:** Mask with Text is refused (ADR-0150, ADR-0152). Mask with Shape works only as
  each element's own mask, and no mask limits an effect to an area.
- **Colour fonts and colour emoji** are unverified. Variable fonts and vertical text are
  deferred by ADR-0007 (both rows unconfirmed on Premiere's side).
- **Credits:** a roll is a hand-keyed `y` move, with no recipe in `montagent-motion`.
- **Layout aids:** no align, distribute or snapping command. The agent computes every literal.

## Pages covered

Every page assigned to this area by the census, plus the three What's-new pages. Each URL is
copied from the first line of the fetched file.

| Page URL | Rows / "no feature: <reason>" |
|---|---|
| https://helpx.adobe.com/premiere/desktop/add-text-images/align-and-distribute-objects/align-objects.html | Align objects |
| https://helpx.adobe.com/premiere/desktop/add-text-images/align-and-distribute-objects/distribute-objects.html | Distribute objects |
| https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html | Rectangle; Ellipse; Polygon; Shape appearance; Outer stroke; Shadow; Transform a layer; Responsive Design: pin to video frame |
| https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-free-form-shapes-for-masking.html | Pen mask that limits an effect; Pen tool and Bezier paths |
| https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-curve.html | Pen tool and Bezier paths |
| https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-straight-line.html | Straight line; Constrained angles while drawing; Stroke width |
| https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-with-pen-tool.html | Pen tool and Bezier paths |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html | Solid fill; Linear and radial gradient; Gradient stroke; Gradient shadow |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html | Transform a layer; Keyframe layer properties |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-the-effect-controls-panel.html | Keyframe layer properties; Keyframe interpolation |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/check-spelling-using-the-graphics-tab.html | Spell check |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-a-clip-layer.html | Clip layers; A graphic holding several layers |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html | Rolling credits; Roll options; Soften a roll |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-markers-in-the-effect-controls-panel.html | Markers in the Effect Controls panel |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html | Responsive Design: pin to video frame; Responsive Design: pin to another layer |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-source-graphics.html | Legacy titles, upgraded to Source Graphics; Source Graphic instances |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/different-ways-to-work-with-graphics.html | Copy and paste a graphic; After Effects compositions; Use a Motion Graphics template; Organise and share templates |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/edit-text-in-graphics-tab.html | Edit text in the Graphics tab |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/export-graphic-as-a-motion-graphics-template.html | Export a graphic as a Motion Graphics template |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/find-and-replace-text-using-the-graphics-tab.html | Find and replace text |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/group-text-and-graphic-layers.html | Group layers |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/mask-with-text-or-shape-using-properties-panel.html | Mask with Text; Mask with Shape; Invert the mask |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/preserve-intro-outro-animations-while-creating-responsive-design-graphics.html | Responsive Design: time (intro and outro) |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/rename-layers.html | Rename layers |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/tips-for-authoring-motion-graphics-templates.html | Render and Replace a graphic; Data-driven templates (CSV/TSV); Organise and share templates; Browse, search, sort, favourite and tag templates (keywords). The rest is authoring advice: no feature |
| https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/working-with-these-motion-graphics-templates-in-premiere.html | Template editable properties; Data-driven templates (CSV/TSV). Otherwise example-project downloads: no feature |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/about-properties-panel.html | The Properties panel as the editing surface; Edit several layers at once |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-linked-and-track-styles.html | Apply linked and track styles |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-style.html | Style part of a text layer |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-linked-and-track-styles.html | Linked styles (titles) and track styles (captions); Save a style for other projects |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | Create a text style; Apply a style to many layers; Inner and Centre stroke; Multiple strokes; Multiple shadows |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-titles.html | Create a title (point text) |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html | Font; Font Style; Font Size; Paragraph alignment; Kerning; Leading; Baseline Shift; Tab Width; Faux Bold; Faux Italics; All Caps; Small Caps; Superscript and Subscript; Underline; Tsume; Text background |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/redefine-linked-and-track-styles.html | Redefine a style |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html | Replace fonts in project; Adobe Fonts; Font discovery on the machine |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/style-parameters-when-applying-from-style-browser.html | Style browser parameters; Tracking |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-color-fonts.html | Colour fonts; Paragraph text in a text box |
| https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-emojis.html | Emojis |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html | Use a Motion Graphics template (`.mogrt`) |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/add-motion-graphic-templates-to-a-sequence.html | Use a Motion Graphics template (`.mogrt`); Missing fonts |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/browse-and-sort-motion-graphics-templates.html | Browse, search, sort, favourite and tag templates |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/customize-motion-graphics-templates.html | Template editable properties; Use a Motion Graphics template (`.mogrt`) |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/install-motion-graphics-templates.html | Install templates |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/media-replacement-in-motion-graphics-templates.html | Media Replacement in templates |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/organize-motion-graphics-templates.html | Organise and share templates |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/use-data-driven-motion-graphics-templates.html | Data-driven templates (CSV/TSV) |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/use-motion-graphics-templates-from-adobe-stock.html | Templates from Adobe Stock |
| https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/where-do-i-get-motion-graphics-templates.html | Use a Motion Graphics template (`.mogrt`); Templates from Adobe Stock; Export a graphic as a Motion Graphics template; The Properties panel as the editing surface (Graphics Templates panel replaced Essential Graphics in 25.0) |
| https://helpx.adobe.com/premiere/desktop/whats-new/beta-features.html | no feature: its two beta items (AI Assistant, Color mode) belong to other areas; nothing on titles or graphics |
| https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html | Uniform scale lock; Guides and snapping; Edit several layers at once; Multiple strokes; Middle Eastern and Indic text (RTL, Indic shaping); The Properties panel as the editing surface ("Edit graphics using the Properties panel"); User defaults for new graphics ("New preferences for Graphics"). The rest are bug fixes or other areas |
| https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html | Text animation presets (Text Animator, Typewriter) ("New animation tools", 26.0); User defaults for new graphics ("New preferences for Graphics", 26.0); The Properties panel as the editing surface (26.0). Other items belong to other areas |
