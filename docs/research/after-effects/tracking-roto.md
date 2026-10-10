# After Effects census: tracking, roto, keying and deformation

Part of the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)), for
[#903](https://github.com/MBehtemam/Montagent/issues/903). It lists each feature After Effects
documents in this area and where Montagent stands on it at `main` `51aa4b5`.

## Sources and method

- **After Effects version:** 26.5 (September 2026), which Adobe's
  [release notes](https://helpx.adobe.com/after-effects/desktop/what-s-new/release-notes-after-effects.html)
  name as current. AI-powered Object Matte arrived in 26.2 (April 2026), and 26.3 (June 2026)
  made the mask tracker faster.
- **Adobe pages:** the After Effects user guide on helpx.adobe.com. Each row links the page it
  comes from. This environment's proxy refused direct fetches of helpx.adobe.com (CONNECT 403),
  so the pages were read through a search index of those same URLs. Check one-line
  descriptions against the live page before you quote them in an ADR.
- **Reference-class pages**, for precedent under ADR-0145: Premiere's
  [mask tracking](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/track-masks.html),
  [Object Mask](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html),
  [Warp Stabilizer](https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html)
  and [Ultra Key](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html).
  CapCut's [green screen](https://www.capcut.com/resource/how-to-use-green-screen-on-capcut),
  [background removal](https://www.capcut.com/resource/how-to-remove-background-in-capcut) and
  [stabilise](https://www.capcut.com/resource/stabilize-video-online) pages are marketing
  material, so treat them as weaker evidence.
- **Montagent sources:** `CONTEXT.md`, `docs/adr/`, `crates/montagent-core/docs/format.md` and
  `format/compositing.md`, `skills/montagent/SKILL.md` (the capability map and "Not by
  design"), `skills/montagent-footage`, `skills/montagent-prerender`, `skills/montagent-character`,
  `schema/montagent.schema.json` and `docs/research/chroma-key`. The repo was grepped for
  tracker, stabilise, warp stabilizer, puppet, liquify, mesh warp, content-aware, roto, face
  track, planar, mocha, keylight, spill suppress, luma key, difference matte and colour range.
  None of these terms appears in a skill, the schema, `format.md` or an ADR, except where the
  table cites one.

### What the statuses mean here

- **covered**: a format key produces the result.
- **partial**: a format key produces part of the result, and the row says which part is missing.
- **reachable by a skill**: a shipped skill names a route to the result. Distortions are named
  by `montagent-prerender` ("distortions, mosaic, lens flare…"), so a mesh deformation drawn in
  code and placed as **Pre-rendered footage** counts. Footage *analysis* (tracking, roto,
  inpainting, stabilising) is named by no skill, so those rows are gaps. For each one, the
  **arrives as** note says whether its output could come in as literal keyframes, a path
  `mask`'s `points`, or footage written by a skill.
- **owned by an open map**: the transforms map, [#591](https://github.com/MBehtemam/Montagent/issues/591)
  (nest, free pivot, swaps), or the audio map, [#795](https://github.com/MBehtemam/Montagent/issues/795).
- **refused by ADR-NNNN**: an accepted ADR refuses the mechanism.
- **gap**: none of the above. Each gap row names its arrival route.

Neighbouring rows that other census tickets score are left out here: the Corner Pin, Bezier
Warp and Rolling Shutter Repair effects (#905), Camera-Shake Deblur (#904), track mattes
(#898), and mask modes, variable-width feather, RotoBezier and the mask-interpolation
assistant (#901).

## The census

### Motion tracking and stabilising

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Point tracker: one-point (Track Motion, Position) | Follows one feature region across frames and writes Position keyframes to a target layer. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html) | gap | Nothing in the repo computes motion from footage. **Arrives as** literal `x`/`y` keyframes on the target element, one per drawn frame. A skill script would write them, as `duck.py` writes `volume` keyframes ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md)). CapCut precedent: camera/object tracking. |
| Point tracker: two-point (Rotation, Scale) | Two track points; the angle and the distance between them give Rotation and Scale keyframes. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html) | gap | **Arrives as** literal `rotation` and `scale` keyframes (with `x`/`y`). Every target property is already an **Animatable property** ([ADR-0146](../../adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)). |
| Point tracker: parallel corner pin | Three track points (the fourth is derived) drive a Corner Pin effect with skew and rotation but no perspective. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html) | refused by ADR-0167 | [ADR-0167](../../adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md): "Skew, corner pin, a camera, depth sorting. Still refused." Skew is also refused by ADR-0012 and ADR-0040. |
| Point tracker: perspective corner pin | Four coplanar track points write keyframes to the four corners of a Corner Pin effect on the target. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html) | refused by ADR-0167 | Corner pin is refused as a free deformation (ADR-0167 §"Skew, corner pin…"). When the tracked quad is the projection of a rectangle with the element's aspect, a solve onto the **Projection** (`swivel`, `tilt`, `perspective`, plus `x`, `y`, `scale`, `rotation`) could arrive as literal keyframes. That route is untested. |
| Track applied to an effect control point | Edit Target sends track data to an effect's point (a light centre, say) instead of a layer's Position. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html) | gap | **Arrives as** literal keyframes on an effect's numbers: a `mask`'s `x`/`y`, a gradient's `center`, a `shadow`'s `dx`/`dy`. The capability map says "Keyframe any effect's numbers and colours". |
| Track data on a null, with layers parented to it | The track goes to one null; other layers follow it through parenting. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html) | owned by an open map (#591) | A group transform is the `nest` on [#591](https://github.com/MBehtemam/Montagent/issues/591) (provisional, [#632](https://github.com/MBehtemam/Montagent/issues/632)). #591 puts the sibling parent link out of scope as After Effects-only ([#594](https://github.com/MBehtemam/Montagent/issues/594)). Today the workaround is to copy the keyframes onto each element (capability map, "Parenting, groups and cameras", workaround #499). |
| Stabilize Motion (point tracker) | Applies the inverse of the tracked Position, Rotation and Scale to the tracked layer itself. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html) | gap | **Arrives as** inverse `x`/`y`/`rotation`/`scale` keyframes on the `video` element. Its moving edges can be hidden by making the element larger than the frame (`fit: cover`, as the footage skill's "Place the footage" recipe does). CapCut ships stabilise. |
| Warp Stabilizer VFX | Analyses in the background, then smooths or locks the camera (Smooth Motion or No Motion). Its methods run from Position up to Subspace Warp, which warps parts of the frame. Framing can synthesise edges, and Reversible/Reverse Stabilization puts the shake back. | [Stabilize footage](https://helpx.adobe.com/after-effects/how-to/stabilize-footage.html) | gap | Premiere ships Warp Stabilizer, so it has reference-class precedent. **Arrives as** literal transform keyframes for the Position, Scale and Rotation methods. Subspace Warp and synthesised edges can only arrive as footage written by a skill (**Pre-rendered footage**, [ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7). Reverse Stabilization (putting the shake back) is the inverse keyframes applied to an inserted element. |
| 3D Camera Tracker | Solves camera motion and a 3D point cloud from footage. It creates a 3D camera and text, solids or nulls on a ground plane. | [Tracking 3D camera movement](https://helpx.adobe.com/after-effects/using/tracking-3d-camera-movement.html) | refused by ADR-0167 | "An element may be projected, never placed": no camera and no depth ([ADR-0167](../../adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md); **Projection** in `CONTEXT.md`). #591 also puts "a camera as its own entity" out of scope. AE's 3D, the camera included, is censused in #902. |
| Mask Tracker | Tracks a drawn mask with one of these methods: Position; Position, Scale & Rotation; …& Skew; Perspective; or Face. It writes a Mask Path keyframe on each frame it analyses. | [Mask tracking](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-mask/rigid-mask-tracking.html) | gap | Premiere ships mask tracking forward and backward, so it has precedent. **Arrives as** keyframes on a path `mask`'s `points`, since a whole vertex list is keyable ([ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md)). A Position-only track can be keyframes on the mask's `x`/`y`, which move the whole outline. The Skew and Perspective methods only reshape the vertex list, which a path mask allows, so no refusal applies. |
| Face Tracking (Outline Only) | Finds a face inside a loose mask and writes a mask path that follows the face's outline. | [Face Tracking](https://helpx.adobe.com/after-effects/using/facetracking.html) | gap | **Arrives as** keyframes on a path `mask`'s `points`, as for the Mask Tracker. |
| Face Tracking (Detailed Features) and face measurements | Tracks eye, brow, nose and mouth points into a Face Track Points effect. It can extract measurements relative to a rest pose, and these feed Character Animator. | [Face Tracking](https://helpx.adobe.com/after-effects/using/facetracking.html) | gap | **Arrives as** literal `x`/`y` keyframes for elements pinned to feature points. Measurements as values have nowhere to land: there are no expressions or value links ([ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)). `montagent-character` drives the mouth from visemes, not face measurements. |
| Mocha AE (Boris FX) planar tracking | A planar tracker for shots with blur or occlusion. It exports Corner Pin or Transform data, and its splines convert to AE masks for roto. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/using/tracking-stabilizing-motion-cs5.html); [effect list](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html) | refused by ADR-0167 | Its main output, Corner Pin, is refused (ADR-0167). Its Transform export is the same gap as the two-point row, and its spline export the same gap as the Mask Tracker row. It is also a third-party plug-in, and **Effect** in `CONTEXT.md` says "never an open plugin architecture". |
| Object Matte (AI, 26.2+) | A hover-and-click selection of a subject. Object Matte, Quick Selection, Selection Brush and Refine Edge tools refine it, and AE propagates it across frames. Since 26.5 the result persists in the disk cache. | [AI-powered Object Matte](https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/object-matte.html) | gap | Premiere ships Object Mask and CapCut ships "Auto removal", so it has precedent. **Arrives as** footage with alpha written by a skill: [ADR-0088](../../adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md) keeps the pre-keyed alpha path ("A source that arrives already keyed still composites"). Or, if vectorised, it arrives as path `mask` `points` keyframes. |

### Rotoscoping and clean-up

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Roto Brush (Next-Gen / Roto Brush 3) | You paint foreground and background strokes on a base frame. An AI model propagates the segmentation to other frames, and you correct strokes where it fails. | [Roto Brush and Refine Matte](https://helpx.adobe.com/after-effects/using/roto-brush-refine-matte.html) | gap | **Arrives as** footage with alpha written by a skill (ADR-0088's pre-keyed path; **Pre-rendered footage**, ADR-0156). No skill names footage segmentation, and an image-sourced `mask` is deferred ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)). |
| Refine Edge tool and Refine Edge Matte | Refines hair and soft edges. Its controls are Feather, Contrast, Shift Edges, Reduce Chatter, motion-blur compensation and edge-colour decontamination. | [Roto Brush and Refine Matte](https://helpx.adobe.com/after-effects/using/roto-brush-refine-matte.html) | gap | A `mask`'s `feather` softens a shape's edge evenly and nothing more (`format/compositing.md`, ADR-0152). It has no shift, contrast, chatter or decontamination control. **Arrives as** part of the same alpha footage as Roto Brush. |
| Rotoscoping with animated masks | Mask Path keyframes, set by hand frame by frame, isolate a subject. AE interpolates between them. | [Animating shape paths and masks](https://helpx.adobe.com/after-effects/using/animating-shape-paths-masks.html) | covered | `"shape": "path"` `mask` with keyframed `points`, `feather` and `invert` ([ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md), [ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)). Capability map: "Mask an element through any closed outline drawn point by point, and animate the outline". #901 scores variable-width feather, mask modes and the interpolation assistant. |
| Content-Aware Fill | Removes an object inside a transparent (masked-out) area by synthesising fill from other frames. Its methods are Object, Surface and Edge Blend, with lighting correction and Photoshop reference frames. | [Content-Aware Fill](https://helpx.adobe.com/after-effects/using/content-aware-fill.html) | gap | **Arrives as** footage written by a skill (a clean plate or a fill layer) laid over the source as a `video`. Inpainting has no format key and no skill route. |
| Clone Stamp and paint clean-up | Paint strokes on a layer, cloned from another frame or place, remove rigs, wires and blemishes during roto clean-up. | [Paint tools](https://helpx.adobe.com/after-effects/using/paint-tools-brush-clone-stamp.html) | gap | **Arrives as** footage written by a skill. There is no paint stroke element. A static patch can be a masked `video` or `image` copy placed over the blemish, using a path `mask` and keyed `x`/`y`. |

### Keying

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Keylight 1.2 | A screen-colour keyer: sample Screen Colour, then screen gain/balance, clip black/white, screen matte views, built-in despill, and inside/outside masks. | [Keying](https://helpx.adobe.com/after-effects/using/keying.html) | partial | `chroma{color, tolerance, softness, spill}`, all keyable ([ADR-0088](../../adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md); **Chroma** in `CONTEXT.md`; footage skill's "Green-screen key"). It has no clip black/white, screen gain/balance or matte view, and no garbage or core matte input. A garbage matte can be a separate `mask` in the same `effects` list. |
| Key Cleaner | Recovers alpha detail lost by a keyer or by compression, before spill suppression. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | gap | No member restores matte detail. **Arrives as** keyed footage with alpha written by a skill, or as a new `chroma` parameter by ADR. |
| Advanced Spill Suppressor | Removes screen-colour spill from the foreground, in Standard mode or Ultra mode (Premiere's Ultra Key algorithm). | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | partial | `chroma.spill` suppresses screen colour on what the matte keeps (ADR-0088, `format/compositing.md`). It has no standalone spill effect without a key, and no choice of method. |
| Color Difference Key | Builds two partial mattes and merges them into one alpha. It suits well-lit screens and semi-transparent smoke or glass. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | partial | The same job as `chroma`, by a different algorithm. Its separate partial-matte controls have no counterpart. |
| Color Range | Keys out a range of colours in Lab, YUV or RGB. It suits multi-coloured or unevenly lit screens. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | partial | `chroma` keys one literal `#RRGGBB` with a `tolerance`. Several `chroma` entries in one ordered `effects` list key several colours. There is no colour-space choice. |
| Linear Color Key | A transparency ramp by closeness to a key colour. Keep Colors mode restores a colour an earlier key removed. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | partial | `chroma`'s `softness` is the partial-alpha band above `tolerance`. Keep Colors (keying back in) has no counterpart. |
| Extract | Keys by brightness, from a histogram of one channel (usually luminance). It suits dark areas and shadows. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | gap | No luminance matte exists. `blend: screen` or `multiply` composites dark or light grounds without a matte, which is a partial workaround for black- or white-ground footage. **Arrives as** footage with alpha written by a skill, or as a new matte operation by ADR (Premiere has precedent: its Luma Key, and Ultra Key's matte controls). |
| Luma Key (obsolete) | Keys by brightness (superseded by Extract). | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | gap | As Extract. |
| Difference Matte | Compares the layer with a clean background layer or frame and removes what matches. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | refused by ADR-0150 | Its matte comes from another element ([ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md); capability map "Not by design"). An image source inside an effect is also deferred ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md) §"An image or luma source"). A skill could still write the result as footage with alpha. |
| Inner/Outer Key | Pulls a soft-edged subject (hair) from an inner and an outer mask, and decontaminates the edge colour. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | gap | A `mask`'s `feather` gives an even soft edge (ADR-0152). The key from the band between two masks has no counterpart. **Arrives as** footage with alpha written by a skill. |
| Color Key (obsolete) | Removes one colour within a tolerance, with edge thin and feather. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | covered | `chroma{color, tolerance, softness}` (ADR-0088). |
| Spill Suppressor (obsolete) | Removes the key colour from edges after a key. | [Keying effects](https://helpx.adobe.com/after-effects/using/keying-effects.html) | covered | `chroma.spill` (ADR-0088). |
| CC Simple Wire Removal | Removes a thin wire between two points by displacing or replicating neighbouring pixels. | [Effect list](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html) | gap | **Arrives as** footage written by a skill. Its two endpoints could also be tracked as literal keyframes. |

### Matte refinement

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Refine Soft Matte | Refines a keyed matte's soft edges: smooth, feather, contrast, shift edge, chatter reduction, motion blur and edge decontamination. | [Matte effects](https://helpx.adobe.com/after-effects/using/matte-effects.html) | gap | `chroma`'s `softness` and a `mask`'s `feather` are the only edge controls. **Arrives as** keyed footage with alpha written by a skill, or as `chroma` parameters by ADR. #905 also scores this. |
| Refine Hard Matte | The same refinement for hard-edged mattes. | [Matte effects](https://helpx.adobe.com/after-effects/using/matte-effects.html) | gap | As Refine Soft Matte. |
| Simple Choker | Shrinks or spreads a matte's edge by a small amount. | [Matte effects](https://helpx.adobe.com/after-effects/using/matte-effects.html) | gap | No choke exists. Changing `tolerance` moves where the key falls, not the matte's edge. Premiere Ultra Key's Choke is precedent for one parameter. |
| Matte Choker | Repeated spread-then-choke passes that fill holes in a matte. | [Matte effects](https://helpx.adobe.com/after-effects/using/matte-effects.html) | gap | As Simple Choker. The footage skill's fix for holes is to lower `tolerance` ("Green-screen key", step 3). |

### Deformation

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Puppet Position pin (Advanced engine; Deform pin in Legacy) | A pin that controls only position on a triangle mesh laid over the layer. | [Puppet tools](https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html) | reachable by a skill | `montagent-character` rigs rigid parts with pivots and bakes their motion (`bake_rig.py`, workaround #499), so a jointed limb moves without a mesh. A bent, continuous mesh is reachable as a distortion drawn in code (`montagent-prerender`). |
| Puppet Advanced pin | A pin that controls position, scale and rotation. The mesh shears unless all three are animated. | [Puppet tools](https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html) | reachable by a skill | A distortion drawn in code (`montagent-prerender`: "distortions"; **Pre-rendered footage**, ADR-0156 §7). The project names only the footage, so changing the pose means re-running the recipe. |
| Puppet Bend pin | Its position comes from the pins around it, and you control its scale and rotation (breathing, a wagging tail). | [Puppet tools](https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html) | reachable by a skill | `montagent-prerender`, as for the Advanced pin. |
| Puppet Starch pin | Stiffens part of the mesh so it distorts less. | [Puppet tools](https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html) | reachable by a skill | `montagent-prerender`, as for the Advanced pin. |
| Puppet Overlap pin | Sets which parts of the mesh draw in front where the mesh folds over itself (In Front, Extent). | [Puppet tools](https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html) | reachable by a skill | For rigid parts, draw order is track `layer` order, which the rig sets (`montagent-character`, "the order the parts draw in"). In a mesh, it is part of the prerendered distortion. |
| Puppet mesh (Density, Expansion, Legacy Triangles) | The triangle mesh the pins deform. Density sets triangle layout and Expansion grows the mesh past the outline. | [Puppet tools](https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html) | reachable by a skill | `montagent-prerender`. No format key deforms an element's pixels: the **Transform** is one flat matrix plus the **Projection** (ADR-0012, ADR-0167). |
| Puppet Sketch (recorded puppet animation) | Drag pins in real time, optionally to audio, and AE records the drag as keyframes. | [Puppet tools](https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html) | gap | Montagent has no GUI. **Arrives as** literal keyframes written by a skill from a recorded or computed path, here or through the prerender recipe. #899 scores recorded motion sketches in general. |
| Liquify | Brush-based push, pull, twirl, bloat and pucker on a layer, with a Freeze Area Mask and Reconstruction mode. | [Distort effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html) | reachable by a skill | A distortion drawn in code (`montagent-prerender`). ADR-0156 §2's tests decide whether a member could join the `effects` list. |
| Mesh Warp | A grid of Bezier patches (up to 31×31) whose vertices and tangents reshape the layer. It is often used to morph between two images. | [Distort effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html) | reachable by a skill | `montagent-prerender`. A crossfade between the two warped images is a `transition` ([ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)). |
| Reshape | Morphs one mask shape into another on the same layer, using a source, a destination and an optional boundary mask. Elasticity controls how the pixels follow. | [Distort effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html) | reachable by a skill | Warping the pixels is a `montagent-prerender` distortion. Morphing a *vector* shape is covered by matching `points` lists on a `path` ([ADR-0162](../../adr/0162-a-morph-between-unlike-shapes-is-written-as-matching-vertex-lists-and-no-rule-resamples-them.md)). |

### Counts

46 rows: **covered 3**, **partial 5**, **reachable by a skill 9**, **owned by an open map 1**
(#591), **refused 5** (ADR-0167 ×4, ADR-0150 ×1), **gap 23**.

## Gaps worth a decision ticket

### 1. Tracking data as literal keyframes, written by a skill script

The rows: one-point and two-point tracks, a track applied to an effect's numbers, Stabilize
Motion, Warp Stabilizer's Position/Scale/Rotation methods, the Mask Tracker, the Face Tracking
outline and feature points, and CC Simple Wire Removal's endpoints. All of these
**arrive as literal keyframes**: on `x`, `y`, `scale` and `rotation`, on effect numbers, or on
a path `mask`'s `points`. The format already accepts every one of them. The question is the
script, by the ducking precedent (ADR-0177):

- the tracker it uses, and how its output stays reproducible (its recipe, pinned version,
  `--check`);
- one keyframe per drawn frame against fitted keyframes, and what that does to `R-EASE-INERT`
  and file size;
- a tracked target inside a `nest`, once #591 lands it.

The reference class has precedent: Premiere has mask tracking and Warp Stabilizer, and CapCut
has stabilise and tracking. So this needs no entry-test prototype under ADR-0145, only the
four invariants.

### 2. Planar and corner-pin placement

The rows: parallel and perspective corner pin, Mocha AE, and the 3D Camera Tracker. Corner pin
and a camera are refused by ADR-0167. Two decisions are open:

- Does solving four tracked corners onto the **Projection**'s `swivel`, `tilt` and
  `perspective`, which arrives as literal keyframes, count as reaching the result for
  screen-replacement shots? It holds only when the quad is the image of a rectangle with
  the element's aspect.
- If [#897](https://github.com/MBehtemam/Montagent/issues/897) makes After Effects precedent,
  does ADR-0167's corner-pin refusal get re-read? (#910 is the ticket for listing AE-only
  refusals.)

### 3. Mattes and clean plates from footage analysis

The rows: Roto Brush, Refine Edge, Object Matte, Content-Aware Fill, Clone Stamp clean-up,
and the matte from Difference Matte. All of these **arrive as footage with alpha (or a clean
plate) written by a skill**. ADR-0088 keeps the pre-keyed path open, and ADR-0156 §7 defines
**Pre-rendered footage**. No shipped skill names footage segmentation or inpainting, though.
`montagent-prerender` is scoped to "looks" drawn in code. The decision is whether a skill route
(and its recipe and decode check) covers *processing* recorded footage, and how ADR-0088's
document-authority point is answered: changing the matte leaves the project byte-identical.
The reference class has precedent: Premiere's Object Mask and CapCut's Auto removal.

### 4. Keyer and matte depth

The rows: Key Cleaner, Simple Choker and Matte Choker, Refine Soft and Hard Matte, Extract and
Luma Key, Inner/Outer Key, multi-range Color Range, and Linear Color Key's Keep Colors mode.
Each would enter as a `chroma` parameter or a new **Matte operation** by its own ADR. Premiere
Ultra Key's matte controls (choke, soften, contrast) and Premiere's luma key are the
precedent. Until then, a skill can write each result as keyed footage with alpha. The footage
skill's edge-floor note shows today's ceiling (`montagent-footage`, "Green-screen key").

### 5. Deformation in the format

The rows: the Puppet pins and mesh, Liquify, Mesh Warp and Reshape. Today all of them are
reachable only as **Pre-rendered footage** (or as rigid parts through `montagent-character`).
A deformation in the format would be a new kind of geometry beyond ADR-0012's flat transform
and ADR-0167's projection. No reference-class precedent turned up in this census, so it would
need the entry test. Puppet Sketch's recorded drags **arrive as literal keyframes** and belong
with #899's motion-sketch question.
