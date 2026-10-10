# After Effects census: shape layers and masks

[#901](https://github.com/MBehtemam/Montagent/issues/901), on the map
[#896](https://github.com/MBehtemam/Montagent/issues/896). It lists every shape-layer and mask
feature Adobe documents in the current After Effects user guide, and scores each one against
Montagent at `main` `51aa4b5`.

**Sources.** Adobe's After Effects user guide on helpx.adobe.com, the version current at
After Effects **26.5** (September 2026, per Adobe's [release notes][rn]). The pages read are
listed under [Adobe pages](#adobe-pages) below. The first draft was written from search-engine
excerpts of those pages while `helpx.adobe.com` was blocked. On 2026-10-10 every row was verified
against the live pages, fetched whole: each link is the canonical URL Adobe redirects to, each
feature is on the page it links, and the user guide's "Drawing, painting, and paths" and masking
sections were walked for features the draft missed. No secondary write-up was used. One row stays
unverified: Skew inside a shape group, because no page of the current guide lists the properties
of a group's transform (the overview says only that each group has its own blend mode and
transform), so the row rests on the app rather than on Adobe's documentation. The
Montagent side comes from `CONTEXT.md`, `docs/adr/`, `crates/montagent-core/docs/format.md`
and its pages, `skills/*` and the router skill's capability map. Each "gap" below was first
searched for in the repo and nothing was found.

**Statuses.** **covered**: Montagent spells the feature's result. **partial**: some of it.
**reachable**: an agent reaches the result with existing keys, by hand or with a skill.
**owned by #591**: the open transforms map decides it. **refused by ADR-NNNN**: a decision
keeps it out. **gap**: nothing spells it and nothing has decided it.

Out of this census's area: extruding shapes in 3D (the 3D census), Roto Brush and Object Matte
(the roto census), the tracker beyond the mask tracker (the tracking census), and the paint tools
(Brush, Clone Stamp and Eraser), which share the guide's "Drawing, painting, and paths" section
but are not shape or mask features.

## The census

### Drawing shapes

| Feature | What it does | Adobe doc | Montagent status | Evidence |
| --- | --- | --- | --- | --- |
| Rectangle | A parametric rectangle path in a shape group. | [creating shapes][b] | covered | `rect` (CONTEXT **Shape**; ADR-0014). |
| Rounded rectangle | A rectangle with one Roundness value. | [creating shapes][b] | covered | `rect` `radius`, one integer corner radius, keyable (format.md; ADR-0014, ADR-0146). |
| Ellipse | A parametric ellipse path. | [creating shapes][b] | covered | `ellipse`, which inscribes its declared box (CONTEXT **Shape**). |
| Polygon | A polystar with Points, Outer Radius, Outer Roundness and Rotation, but no inner radius. | [creating shapes][b] | covered | A closed `path` of corners. A `polygon` type is refused as a second spelling of one drawing (ADR-0154; CONTEXT **Line, polygon**). The parametric knobs are not in the format: the figure is written out vertex by vertex. |
| Star | A polystar with inner and outer radius and inner and outer roundness. | [creating shapes][b] | covered | A closed `path`. Roundness is written as handles, and keyed radii are keyed `points` lists with the same vertex count (ADR-0154 §3). A star whose point count changes is padded by hand (ADR-0162). |
| Pen paths | Bezier paths drawn with the Pen tool, with corner and smooth vertices. | [overview][a], [creating shapes][b] | covered | `path` with `points`, `at`, `in`, `out` and `closed`. A vertex without handles is a corner (ADR-0154). |
| Convert a parametric shape to a Bezier path | Turns a rectangle, ellipse or polystar into editable vertices. | [animating paths][f] | reachable | `rect` and `ellipse` stay their own types; the same figure is rewritten as a `path` with explicit handles (ADR-0154). |
| Reverse Path Direction | Flips a path's direction, which decides holes under the nonzero rule and where trims and dashes run. | [shape attributes][c] | reachable | Reverse the `points` list and swap each vertex's `in` and `out`. No flag (ADR-0154). |
| Animating a shape path | Keyframes on the Path property. | [animating paths][f] | covered | `points` animates as one whole list (ADR-0154 §3), and a morph between unlike shapes uses matched lists (ADR-0162). |
| Expression access to path points | Expressions read and build mask, shape and brush paths (`points()`, `tangents`, `createPath`). | [animating paths][f] | refused by ADR-0145 | Expressions and value links are refused (ADR-0145; capability map *Not by design*). |

### Groups

| Feature | What it does | Adobe doc | Montagent status | Evidence |
| --- | --- | --- | --- | --- |
| Groups inside a shape layer | Shapes, paints and operations nested in groups. Each group has its own blend mode and transform. | [overview][a] | owned by #591 | The nest (provisional `"type": "nest"`, [#632](https://github.com/MBehtemam/Montagent/issues/632)). Today the capability map lists "Parenting, groups and cameras" under *can't do yet* (#499). CONTEXT **Group** is a label that does not affect rendering. |
| Group transform (anchor, position, scale, rotation) | Transforms every shape in the group, "the same as layer parenting". | [overview][a] | owned by #591 | The nest is a pure matrix with `x`, `y`, `rotation`, `scale` and `pivot` (#632). The free pivot is ADR-0174, written on its branch and not yet on `main` (#631, #822). |
| Group blend mode and opacity | Each group has its own blend mode; its transform carries the group's opacity. | [overview][a] | owned by #591 | #632 keeps group opacity and group compositing out of the nest. #591 lists them under *Not yet specified*. |
| Group skew and skew axis (unverified) | Skew in the group's transform. No page of the current guide names it. | none | refused by ADR-0012 | "Skew is out": neither CapCut nor Premiere's Motion has it (ADR-0012). |

### Fill and stroke (paint operations)

| Feature | What it does | Adobe doc | Montagent status | Evidence |
| --- | --- | --- | --- | --- |
| Solid fill and fill opacity | Paints the inside of the paths above it in the group. | [shape attributes][c] | covered | `fill` with `#RRGGBBAA`, keyable (CONTEXT **Fill**, **Colour**; ADR-0146). |
| Fill rule | Nonzero winding (the default) or even-odd. | [shape attributes][c] | partial | Nonzero only (ADR-0154 §5). A path is one outline, since subpaths are refused (ADR-0154), so even-odd would change only a self-crossing outline. |
| Compound paths and holes | Several paths in one group filled as one, so an inner path cuts a hole. | [shape attributes][c] | reachable | ADR-0154 refuses subpaths. A hole is a second, inverted path `mask` (ADR-0163; CONTEXT **Mask**). |
| Several fills and strokes, and Composite | One path carries many paints, stacked, and Composite reorders them ("Above Previous In Same Group"). | [shape attributes][c] | reachable | Each element draws exactly one thing, so each paint is its own element stacked by `layer` (ADR-0148; CONTEXT **Repeat, repeater, clone**). |
| A blend mode per fill or stroke | Each paint blends with the other paints in its group. | [shape attributes][c] | reachable | `blend` on the separate element (ADR-0147). It takes five values; the full list of AE blend modes is scored in the compositing census. |
| Stroke colour, width and opacity | Strokes the outline. | [shape attributes][c] | covered | `stroke` and `stroke_width`, keyable. The stroke falls inside the box on a `rect` or `ellipse` and is centred on a `path` (CONTEXT **Stroke**; ADR-0014, ADR-0154). |
| Gradient fill | A linear or radial gradient with colour and opacity stops. | [shape attributes][c] | covered | `gradient` `linear` or `radial` as a `fill`, with keyable `stops` (ADR-0149; CONTEXT **Gradient**). |
| Gradient stroke | The same gradient used as the stroke paint. | [shape attributes][c] | covered | A gradient is a paint, so `stroke` takes one (ADR-0149; CONTEXT **Paint**). |
| Gradient Start and End points | Set the gradient's direction and scale. | [shape attributes][c] | covered | `angle` for linear, and `center` and `radius` for radial, as fractions of the box (ADR-0149). A two-point spelling is refused as four numbers that must move together (ADR-0149, considered options). |
| Highlight Length and Highlight Angle | Move a radial gradient's highlight off centre (a focal point). | [shape attributes][c] | refused by ADR-0149 | "Sweep or two-point conical now. Refused: no precedent, so each needs its own prototype" (ADR-0149, considered options). |
| Line join and miter limit | Miter, round or bevel corners, where Miter Limit sets when a miter becomes a bevel. | [shape attributes][c], [offset paths][e] | covered | `stroke_join` and `stroke_miter_limit` on `path`. A `rect` or `ellipse` keeps a fixed join (ADR-0158 §1). |
| Line cap | Butt, round or projecting ends. | [shape attributes][c] | covered | `stroke_cap` `butt`, `round` or `square` on `path`. A `rect` or `ellipse` has butt dash ends (ADR-0158). |
| Dashes and dash offset | Up to three dash and gap pairs that repeat along the path, with an animatable Offset. | [shape attributes][c] | covered | `stroke_dash` (any even list) and keyable `stroke_dash_offset` on `path`, `rect` and `ellipse`, including marching ants (ADR-0158; CONTEXT **Dash pattern**). |
| Taper | Thins the stroke toward its start, its end or both, with a length, width and ease for each; Length Units are Percentage of the path or Pixels. Native strokes only. | [taper][d] | gap | No ADR, key or skill. The stroke width is uniform along the outline (CONTEXT **Stroke**). |
| Wave | Ripples the stroke's width along the path: Amount, Units (pixels or cycles), Wavelength and Phase. | [taper][d] | gap | No ADR, key or skill. |

### Path operations

| Feature | What it does | Adobe doc | Montagent status | Evidence |
| --- | --- | --- | --- | --- |
| Merge Paths | Combines the paths above it into one compound path: Merge, Add (union), Subtract, Intersect or Exclude Intersections. | [shape attributes][c] | partial | On what an element keeps: Intersect is two masks in one list, and Subtract is a mask plus an inverted mask (ADR-0152, ADR-0163). A union fill can be drawn as overlapping opaque elements. Exclude, and a stroke around the merged outline, have no spelling. |
| Offset Paths | Grows or shrinks a path by an Amount, with a line join and miter limit, and makes Copies. | [offset paths][e] | gap | No outline offset exists. A `rect` or `ellipse` is offset by resizing its box, but a curved `path` has no exact integer offset. Copies are refused by ADR-0148. |
| Pucker & Bloat | Pulls vertices out while the segments curve in, or the reverse. | [shape attributes][c] | reachable | The result is a literal vertex list an agent computes, keyed as matched lists. ADR-0162 already argues that any end state a rule could reach is expressible in exact integers. No skill script does this. |
| Round Corners | Rounds a path's corners by a Radius. | [shape attributes][c] | reachable | Covered on `rect` by `radius`. On a `path`, a corner is rounded by writing handles (ADR-0154). |
| Twist | Rotates a path more at its centre than at its edges. | [shape attributes][c] | reachable | Literal vertices computed by the agent, after splitting segments (the method in ADR-0162 §1). No skill. |
| Zig Zag | Turns a path into regular peaks and valleys (size, ridges, corner or smooth). | [shape attributes][c] | reachable | Literal vertices, corner or with handles (ADR-0154). No skill. |
| Wiggle Paths | Randomises a path into irregular peaks that animate by themselves, with no keyframes. | [shape attributes][c] | gap | Expressions are refused (ADR-0145), and only an effect takes a `seed` (ADR-0156). Per-frame keyed `points` written by a skill script would follow the ducking pattern (ADR-0177), but no such script exists. |
| Wiggle Transform | Randomises position, anchor, scale and rotation, per copy when placed after a Repeater, with a Correlation control. | [shape attributes][c] | gap | Same as Wiggle Paths: no skill writes random literal transform keyframes. Per copy, it also needs the refused repeat (ADR-0148). |
| Repeater | Draws virtual copies of everything above it, each transformed a step further, with Copies, Offset, and Start and End Opacity. | [shape attributes][c] | refused by ADR-0148 | "A repeat does not enter": each copy is its own element (ADR-0148; capability map *Not by design*). |
| Trim Paths on one path | Draws a window from Start to End, turned by Offset. | [shape attributes][c] | covered | `trim_start`, `trim_end` and `trim_offset` on `path`, `rect` and `ellipse` (ADR-0160; CONTEXT **Trim**). |
| Trim Paths "Simultaneously" | Trims every path above it with the same window. | [shape attributes][c] | reachable | Each outline is its own element carrying the same trim keyframes (ADR-0148, ADR-0160). |
| Trim Paths "Individually" | Treats the paths as one compound path and runs the window through them in sequence. | [shape attributes][c] | gap | One window across several elements has no spelling. Splitting it by hand needs outline lengths, which ADR-0158 and ADR-0160 record are not exact numbers in the file. |

### Converting to shapes

| Feature | What it does | Adobe doc | Montagent status | Evidence |
| --- | --- | --- | --- | --- |
| Create Shapes from Vector Layer, importing SVG as a composition (26.2), and pasting SVG or Illustrator art as shape layers (26.3) | Turns Illustrator, EPS, PDF or SVG artwork into editable shape layers, keeping gradient fills, gradient strokes and transparency. | [creating shapes][b], [SVG import][j] | partial | No element takes an SVG (ADR-0171). The `montagent-prerender` SVG route brings the artwork in as one PNG `image`. No tool converts artwork into editable `path` elements, and ADR-0171 does not rule on one. |
| Copy a path from Illustrator or Photoshop | Pastes a copied path in as a mask path or a shape path. | [creating shapes][b] | gap | No importer turns a foreign path into `points`, and ADR-0171 does not rule on one. An agent can only transcribe the vertices by hand. |
| Create a shape or mask from a motion path | Pastes Position or Anchor Point keyframes into a Path or Mask Path property. | [creating shapes][b], [create masks][i] | reachable | `x` and `y` are keyed apart, so the motion is a straight-segment sequence; an agent writes those positions as a `path` or path-mask `points` list (ADR-0154, ADR-0163). |
| Create Shapes from Text | Puts each character's outline in its own shape group, with matching fill and stroke. | [creating shapes][b] | gap | Nothing exports glyph outlines as `path` elements. A text element bent along a curve is a different capability (ADR-0161). |
| Create Masks from Text | Makes a mask from each character's outline on a new solid. | [creating shapes][b] | refused by ADR-0152 | A text source in a `mask` is refused. Footage seen through letters belongs to paint (ADR-0152 §3). |

### Masks

| Feature | What it does | Adobe doc | Montagent status | Evidence |
| --- | --- | --- | --- | --- |
| Masks drawn with the shape tools | Rectangle, rounded rectangle, ellipse, polygon and star masks. | [create masks][i] | covered | `mask` with `shape` `rect`, `ellipse` or `circle`, and `radius` on `rect` (ADR-0084); a polygon or star is a `path` mask (ADR-0163). |
| Bezier mask paths | Any closed outline drawn with the Pen tool. | [create masks][i] | covered | `shape: "path"` with inline `points`, always closed (ADR-0163). |
| Animated mask path | Keyframes on Mask Path, as in a reveal. | [animating paths][f] | covered | The rect, `feather` and path `points` are keyable, and a reveal is a keyed rect from 0 (ADR-0146, ADR-0163; format/compositing.md). |
| Mode: Add | Adds each mask's area to the masks above it (a union). The default mode. | [masks and mattes][g] | gap | Masks in one list only intersect (ADR-0152). Nothing spells the union of two masks. |
| Mode: Subtract | Cuts this mask's area out of the masks above it. | [masks and mattes][g] | covered | A plain mask plus an inverted mask in one list, which is how a ring is made (ADR-0152; CONTEXT **Mask**). |
| Mode: Intersect | Keeps only where the masks overlap. | [masks and mattes][g] | covered | "Masks in one list intersect" (ADR-0152). |
| Mode: Lighten and Darken | Where masks overlap, keep the higher or the lower transparency. | [masks and mattes][g] | partial | Montagent has no mask opacity, so Darken is Intersect (covered) and Lighten is Add (a gap). |
| Mode: Difference | Keeps where the masks do not overlap. | [masks and mattes][g] | gap | No spelling. |
| Mode: None | The mask does not change alpha, and its path only feeds effects such as Stroke or Fill. | [masks and mattes][g] | reachable | A separate `path` element draws the outline. A mask never names another element's outline, and no effect reads one (ADR-0150, ADR-0163 §1). |
| Inverted | Swaps the mask's inside and outside. | [masks and mattes][g] | covered | `invert` (ADR-0152). It is the exact complement only off the edge (ADR-0165). |
| Mask Feather | A uniform soft edge that straddles the mask path, with a Gaussian falloff. | [masks and mattes][g] | covered | `feather`, centred on the edge and keyable (ADR-0152). AE's feather has separate horizontal and vertical values; Montagent's is one number. |
| Variable-width feather | The Mask Feather tool sets feather points with their own widths along the mask, with a smooth or linear falloff. | [masks and mattes][g] | gap | No ADR or key. |
| Mask Opacity | How strongly the mask applies. | [masks and mattes][g] | refused by ADR-0152 | "Not until a case asks" (ADR-0152 §3). |
| Mask Expansion | Moves the mask's effect in or out without moving its path. | [masks and mattes][g] | refused by ADR-0152 | It would be a second knob for the edge the rect already moves (ADR-0152 §3). |
| RotoBezier paths | Handles and curvature are computed from the vertices automatically. | [animating paths][f] | reachable | The format has no derived handles: an agent writes the same curve as explicit `in` and `out` handles (ADR-0154). |
| First vertex and Set First Vertex | Picks the vertex the path's numbering starts from, which drives interpolation. | [animating paths][f] | covered | `points[0]` is the start, and the list is reordered to move it. Matching start vertices is part of a hand-written morph (ADR-0160 §1, ADR-0162 §1). |
| Smart Mask Interpolation | Generates in-between mask keyframes: keyframe rate, bending resistance, quality, added vertices, matching method. | [animating paths][f] | refused by ADR-0162 | "No resampling rule; the lists are matched by hand" (ADR-0162; capability map *Not by design*). |
| Free Transform Points | Scales and rotates a whole mask, or some of its vertices. | [animating paths][f] | reachable | A keyed `x` or `y` moves a path mask as a whole. Any other change is a rewrite of `points`. The mask rides the element's transform (ADR-0084, ADR-0163). |
| Move a mask, or pan the layer behind it | Drag the mask alone, or move the layer while its mask stays put (the Pan Behind tool). | [animating paths][f] | reachable | Moving the mask is its keyed `x` and `y`. A mask rides the element's transform, so panning behind it is the element's motion with the opposite keys on the mask (ADR-0084; format/compositing.md). |
| Mask path as a motion path | Pastes a mask, shape, paint or Illustrator path into Position or Anchor Point; the pasted keyframes rove so the layer moves at a constant speed. | [animation tools][n] | gap | `x` and `y` are keyed separately, with no spatial curve. Only text follows a curve, with `path_offset` (ADR-0161). No skill bakes a path into position keyframes. |
| Mask motion blur | A mask blurs with its own movement, per mask: Same As Layer, On or Off. | [animation tools][n] | partial | Same As Layer is covered: every keyed value, a mask's included, is read at each motion-blur sample (ADR-0155; format/compositing.md). A per-mask On or Off has no field. |
| Mask Tracker | Tracks a mask to footage with seven methods, from Position to Perspective and two face-tracking modes. | [mask tracking][k] | gap | No tracker exists. This belongs in the tracking census. |
| Auto-trace | Makes Bezier masks from a layer's alpha, red, green, blue or luminance channel at a threshold, on one frame or across the work area. | [create masks][i] | gap | No tracing tool. |
| Mask Reference | Limits an effect to the area of one or more of the layer's masks. | [mask reference][h] | reachable | `effects` is an ordered list that holds the mask (ADR-0040; format/compositing.md), so a mask cannot fence one effect off from the rest. A copy of the element carrying the effect and then the mask, stacked over the original, gives the result. |
| Track mattes | One layer's alpha or luma is another layer's matte. | [track mattes][m] | refused by ADR-0150 | A matte taken from another element is refused (ADR-0150; capability map *Not by design*). |
| Create Nulls from Paths | Nulls that drive path points, follow them, or trace the path. | [nulls from paths][l], [animating paths][f] | refused by ADR-0145 | Links between values are refused (ADR-0145; capability map *Not by design*). Parenting itself is #591's. |

### Counts

| Status | Rows |
| --- | --- |
| covered | 24 |
| partial | 5 |
| reachable | 16 |
| owned by #591 | 3 |
| refused by an ADR | 10 (ADR-0012, ADR-0145 ×2, ADR-0148, ADR-0149, ADR-0150, ADR-0152 ×3, ADR-0162) |
| gap | 14 |
| **total** | **72** |

One row, Group skew, is unverified against the guide (see the intro).

## Gaps worth a decision ticket

**Stroke width along the outline.** Taper and Wave both vary the stroke's width along its
length. They are one question: does a stroke get a width profile, written as literal
values in fractions of the outline, the way a trim is? They share ADR-0160's unit and its
start points. Neither CapCut nor Premiere was checked as precedent, so ADR-0145's entry test
probably applies.

**Path operations as geometry the agent writes.** Pucker & Bloat, Twist, Zig Zag, Round
Corners on a path, and Offset Paths all produce a vertex list. The first four are already
reachable as literal `points`, but the arithmetic is error-prone by hand. One ticket can ask
whether a `montagent-motion` script should write them, as `type_on.py` does. Offset Paths on a
curved outline has no exact integer answer, so it needs its own ruling: an offset helper, or
a recorded "no".

**Randomness written as literal keyframes.** Wiggle Paths and Wiggle Transform (and wiggle in
general, which #896 already notes) could follow ADR-0177: a skill script writes seeded literal
keyframes and the format stays untouched. One ticket for both.

**Union and difference of shapes.** Mask modes Add, Lighten and Difference, and Merge Paths'
Add and Exclude, have no spelling. Masks only intersect (ADR-0152), and a path has no subpaths
(ADR-0154). The question is whether a mask list gets a combine mode, or a `path` gets more than
one outline, or neither. It interacts with the fill rule (nonzero only).

**Following a path.** Using a mask or shape path as a motion path, at the constant speed AE's
roving keyframes give it, exists only for text (ADR-0161). Whether a visual element's position may follow a curve (a spatial
curve on `x`/`y`, a `path_offset`-like field, or a skill that bakes keyframes) is one ticket.
It may sit near #591.

**One trim across several outlines.** Trim Paths "Individually" needs a trim window that runs
through several elements in sequence. It is a small, specific question for ADR-0160's owner.

**Smaller or deferred.** These are worth a ticket only if a case asks:
- Variable-width feather.
- Creating editable `path` elements from SVG or Illustrator artwork or a copied Illustrator or Photoshop path (a converter skill, which ADR-0171 leaves unruled), or from text outlines.
- A per-mask motion-blur switch.
- Auto-trace.
- The mask tracker, which belongs in the tracking census.
- The two-point conical gradient (Highlight Length and Angle), which needs ADR-0149's prototype.

## Adobe pages

- [a] [Overview of shape layers, paths, and vector graphics][a]
- [b] [Creating shapes][b]
- [c] [Shape attributes, paint operations, and path operations for shape layers][c]
- [d] [How to taper shape strokes][d] (taper and wave)
- [e] [Use Offset Paths shape effect to alter shapes][e]
- [f] [Managing and animating shape paths and masks][f]
- [g] [Alpha channels and masks][g] (mask modes, opacity, expansion and feather)
- [h] [Mask Reference][h] (limiting an effect to masks)
- [i] [Create masks][i]
- [j] [Import SVG files][j]
- [k] [Mask Tracking][k]
- [l] [Create Nulls for Positional Properties and Paths][l]
- [m] [Track Mattes and Traveling Mattes][m]
- [n] [Assorted animation tools][n] (motion paths from paths, mask motion blur)
- [rn] [After Effects release notes][rn]

[a]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/vector-graphics-and-raster-images/overview-shape-layers-paths-vector.html
[b]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/shapes-and-shape-attributes/creating-shapes-masks.html
[c]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/shapes-and-shape-attributes/shape-attributes-paint-operations-path.html
[d]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/taper-shape-strokes/taper-shape-strokes.html
[e]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/shapes-and-shape-attributes/use-offset-paths.html
[f]: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-shape-paths-and-masks/animating-shape-paths-masks.html
[g]: https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/work-with-alpha-channels-and-masks/alpha-channels-masks-mattes.html
[h]: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-mask/mask-reference.html
[i]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/work-with-masks/create-masks.html
[j]: https://helpx.adobe.com/after-effects/desktop/import-files/import-svg-files/import-svg-files.html
[k]: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-mask/rigid-mask-tracking.html
[l]: https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/null-controllers/create-nulls-for-positional-properties-and-paths.html
[m]: https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/work-with-track-mattes-and-traveling-mattes/track-mattes-and-traveling-mattes.html
[n]: https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/assorted-animation-tools/assorted-animation-tools.html
[rn]: https://helpx.adobe.com/after-effects/desktop/what-s-new/release-notes-after-effects.html
