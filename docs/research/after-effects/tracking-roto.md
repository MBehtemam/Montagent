# After Effects census: tracking, roto, keying and deformation

Part of the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)), for
[#903](https://github.com/MBehtemam/Montagent/issues/903). It lists each feature After Effects
documents in this area and where Montagent stands on it at `main` `51aa4b5`.

## Sources and method

- **After Effects version:** 26.5 (September 2026), which Adobe's
  [release notes](https://helpx.adobe.com/after-effects/desktop/what-s-new/release-notes-after-effects.html)
  name as current. Adobe's [What's new](https://helpx.adobe.com/after-effects/desktop/what-s-new/whats-new.html)
  dates AI-powered Object Matte and the Unmult effect to 26.2 (April 2026), the faster mask
  tracker to 26.3 (June 2026), and Object Matte's disk cache to 26.5.
- **Adobe pages:** the After Effects user guide on helpx.adobe.com. Each row links the page it
  comes from. **Verified against the live pages on 2026-10-10.** Every link was fetched in full;
  links that redirected now carry the canonical URL, and each row's description was checked
  against its page. The live table of contents for this area (Track motion, Track faces, Track
  mask, Keying, Keying effects, Roto Brush and Refine Matte, Content-Aware Fill, Puppet tools,
  Paint tools, Matte, Distort and Cycore effects) was walked and missed features were added.
  One row still rests partly on a non-Adobe page: **Keylight 1.2**. Adobe's guide says only that
  Keylight ships and defers its controls to the plug-in's own documentation, so the controls in
  that row are taken from Foundry's Keylight reference (the Nuke edition of the same keyer), not
  from an Adobe page. Refine Hard Matte is named in Adobe's effect list but described on no
  Adobe page; its row says so.
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
| Point tracker: one-point (Track Motion, Position) | Follows one feature region across frames (Track Type: Transform) and writes Position keyframes to a target layer. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | gap | Nothing in the repo computes motion from footage. **Arrives as** literal `x`/`y` keyframes on the target element, one per drawn frame. A skill script would write them, as `duck.py` writes `volume` keyframes ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md)). CapCut precedent: camera/object tracking. |
| Point tracker: two-point (Rotation, Scale) | Two track points; the change of angle and of distance between their attach points gives Rotation and Scale keyframes. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | gap | **Arrives as** literal `rotation` and `scale` keyframes (with `x`/`y`). Every target property is already an **Animatable property** ([ADR-0146](../../adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)). |
| Point tracker: parallel corner pin | Three track points (the fourth is derived) drive a Corner Pin effect with skew and rotation but no perspective. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | refused by ADR-0167 | [ADR-0167](../../adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md): "Skew, corner pin, a camera, depth sorting. Still refused." Skew is also refused by ADR-0012 and ADR-0040. |
| Point tracker: perspective corner pin | Four track points follow skew, rotation and perspective, and write keyframes to the four corners of a Corner Pin effect added to the target. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | refused by ADR-0167 | Corner pin is refused as a free deformation (ADR-0167 §"Skew, corner pin…"). When the tracked quad is the projection of a rectangle with the element's aspect, a solve onto the **Projection** (`swivel`, `tilt`, `perspective`, plus `x`, `y`, `scale`, `rotation`) could arrive as literal keyframes. That route is untested. |
| Track applied to an effect control point | Edit Target sends track data to an effect's point (a light centre, say) instead of a layer's Position. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | gap | **Arrives as** literal keyframes on an effect's numbers: a `mask`'s `x`/`y`, a gradient's `center`, a `shadow`'s `dx`/`dy`. The capability map says "Keyframe any effect's numbers and colours". |
| Raw track (Attach Point data only) | Track Type Raw tracks position only and applies nothing; the Attach Point keyframes are copied by hand (to a paint stroke's Position, say) or linked with expressions. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | gap | **Arrives as** literal `x`/`y` keyframes pasted wherever they are needed, as the one-point row. The expression link has no counterpart: no expressions or value links ([ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)). |
| Track data on a null, with layers parented to it | The track is applied to one null object; other layers follow it through parenting. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | owned by an open map (#591) | A group transform is the `nest` on [#591](https://github.com/MBehtemam/Montagent/issues/591) (provisional, [#632](https://github.com/MBehtemam/Montagent/issues/632)). #591 puts the sibling parent link out of scope as After Effects-only ([#594](https://github.com/MBehtemam/Montagent/issues/594)). Today the workaround is to copy the keyframes onto each element (capability map, "Parenting, groups and cameras", workaround #499). |
| Stabilize Motion (point tracker) | Applies the opposite of the tracked motion to the tracked layer itself: Anchor Point keyframes for position, plus Rotation and Scale keyframes. | [Tracking and stabilizing motion](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | gap | **Arrives as** inverse `x`/`y`/`rotation`/`scale` keyframes on the `video` element: AE keys the Anchor Point, but Montagent's `origin` is one of nine keywords, so the inverse position goes on `x`/`y`, which gives the same picture. Its moving edges can be hidden by making the element larger than the frame (`fit: cover`, as the footage skill's "Place the footage" recipe does). CapCut ships stabilise. |
| Warp Stabilizer VFX | Analyses in the background, then smooths or locks the camera (Result: Smooth Motion or No Motion). Methods: Position; Position, Scale and Rotation; Perspective (the whole frame corner-pinned); Subspace Warp (default; warps parts of the frame). Framing crops, auto-scales or synthesises edges from nearby frames; it also reduces rolling-shutter ripple, and Reversible/Reverse Stabilization lets an effect be applied to a steadied region and the shake put back. | [Tracking and stabilizing motion: Warp Stabilizer VFX](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html) | gap | Premiere ships Warp Stabilizer, so it has reference-class precedent. **Arrives as** literal transform keyframes for the Position and Position, Scale and Rotation methods. The Perspective method is a whole-frame corner pin, which ADR-0167 refuses. Subspace Warp and synthesised edges can only arrive as footage written by a skill (**Pre-rendered footage**, [ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7). Reverse Stabilization (putting the shake back) is the inverse keyframes applied to an inserted element. |
| 3D Camera Tracker | Solves camera motion and a 3D point cloud from footage. It creates a 3D camera and text, solids, nulls or a shadow catcher on tracked points and a ground plane, and can export the camera to 3D apps. | [Tracking 3D camera movement](https://helpx.adobe.com/after-effects/desktop/work-with-3d-composition/3d-camera-tracker-effect/tracking-3d-camera-movement.html) | refused by ADR-0167 | "An element may be projected, never placed": no camera and no depth ([ADR-0167](../../adr/0167-an-element-may-be-projected-never-placed-swivel-tilt-and-perspective-join-the-transform.md); **Projection** in `CONTEXT.md`). #591 also puts "a camera as its own entity" out of scope. AE's 3D, the camera included, is censused in #902. |
| Mask Tracker | Tracks a drawn mask on a footage or precomp layer, forward, backward or a frame at a time, from the Tracker panel or the Timeline (26.3, up to 5× faster). Methods: Position; Position & Rotation; Position, Scale & Rotation (default); …& Skew; Perspective (a planar corner-pin solve); Face Tracking (Outline Only or Detailed Features). It writes a Mask Path keyframe on each frame it analyses. | [Mask tracking](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-mask/rigid-mask-tracking.html) | gap | Premiere ships mask tracking forward and backward, so it has precedent. **Arrives as** keyframes on a path `mask`'s `points`, since a whole vertex list is keyable ([ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md)). A Position-only track can be keyframes on the mask's `x`/`y`, which move the whole outline. The Skew and Perspective methods only reshape the vertex list, which a path mask allows, so no refusal applies. |
| Face Tracking (Outline Only) | Finds a face inside a loose mask and writes a mask path that follows the face's outline. | [Face Tracking](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-faces/facetracking.html) | gap | **Arrives as** keyframes on a path `mask`'s `points`, as for the Mask Tracker. |
| Face Tracking (Detailed Features) and face measurements | Tracks eye, brow, pupil, nose and mouth points into a Face Track Points effect. Extract & Copy Face Measurements adds a Face Measurements effect (face offset and orientation, eyelid openness, gaze, mouth scale) relative to a rest pose, and copies it for Character Animator. | [Face Tracking](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-faces/facetracking.html) | gap | **Arrives as** literal `x`/`y` keyframes for elements pinned to feature points. Measurements as values have nowhere to land: there are no expressions or value links ([ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)). `montagent-character` drives the mouth from visemes, not face measurements. |
| Mocha AE (Boris FX) planar tracking | A GPU planar tracker bundled with After Effects for shots with blur or objects leaving frame. It exports Corner Pin or Transform data; its splines become AE masks (Create AE Masks) or a matte (Apply Matte, the Mocha shape effect). | [Tracking and stabilizing motion: Mocha AE](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-motion/tracking-stabilizing-motion-cs5.html); [effect list](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html) | refused by ADR-0167 | Its main output, Corner Pin, is refused (ADR-0167). Its Transform export is the same gap as the two-point row, and its spline export the same gap as the Mask Tracker row. It is also a third-party plug-in, and **Effect** in `CONTEXT.md` says "never an open plugin architecture". |
| Object Matte (AI, 26.2+) | Hover-and-click (or marquee) AI selection of a subject, built on Premiere's Object Mask technology. Quick Selection (classic Roto Brush reimagined), Selection Brush and Refine Edge tools refine it, AE propagates it across frames, and Freeze locks it. Since 26.5 propagated frames persist in the disk cache. | [AI-powered Object Matte](https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/object-matte.html) | gap | Premiere ships Object Mask and CapCut ships "Auto removal", so it has precedent. **Arrives as** footage with alpha written by a skill: [ADR-0088](../../adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md) keeps the pre-keyed alpha path ("A source that arrives already keyed still composites"). Or, if vectorised, it arrives as path `mask` `points` keyframes. |

### Rotoscoping and clean-up

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Roto Brush (Next-Gen / Roto Brush 3) | You paint foreground and background strokes on a base frame. An AI model (version 3.0; 1.0 and 2.0 remain selectable) propagates the segmentation to other frames, you correct strokes where it fails, and Freeze caches the result. | [Roto Brush and Refine Matte](https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/roto-brush-refine-matte.html) | gap | **Arrives as** footage with alpha written by a skill (ADR-0088's pre-keyed path; **Pre-rendered footage**, ADR-0156). No skill names footage segmentation, and an image-sourced `mask` is deferred ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)). |
| Refine Edge tool and Refine Edge Matte | Refines hair and soft edges. Its controls are Smooth, Feather, Contrast, Shift Edges, Reduce Chatter, motion blur and edge-colour decontamination; the Refine Matte effect also works on mattes from other sources. | [Roto Brush and Refine Matte](https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/roto-brush-refine-matte.html) | gap | A `mask`'s `feather` softens a shape's edge evenly and nothing more (`format/compositing.md`, ADR-0152). It has no shift, contrast, chatter or decontamination control. **Arrives as** part of the same alpha footage as Roto Brush. |
| Rotoscoping with animated masks | Mask Path keyframes, set by hand frame by frame, isolate a subject. AE interpolates between them. | [Animating shape paths and masks](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-shape-paths-and-masks/animating-shape-paths-masks.html) | covered | `"shape": "path"` `mask` with keyframed `points`, `feather` and `invert` ([ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md), [ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md)). Capability map: "Mask an element through any closed outline drawn point by point, and animate the outline". #901 scores variable-width feather, mask modes and the interpolation assistant. |
| Content-Aware Fill | Fills a transparent (masked-out) area by synthesising pixels from other frames into a new fill layer. Fill methods are Object, Surface and Edge Blend, with Alpha Expansion, Lighting Correction (Subtle, Moderate, Strong) and reference frames painted in Photoshop. | [Content-Aware Fill](https://helpx.adobe.com/after-effects/desktop/remove-objects-from-your-videos/content-aware-fill.html) | gap | **Arrives as** footage written by a skill (a clean plate or a fill layer) laid over the source as a `video`. Inpainting has no format key and no skill route. |
| Mask Reference | An effect's Compositing Options pick one or more masks on the same layer (tracked or not) so that the effect composites only inside them; the mask's feather and opacity shape and scale the effect. | [Mask Reference](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/track-mask/mask-reference.html) | partial | A `mask` in `effects` cuts the whole element, in list order (ADR-0040, `format/compositing.md`); no effect takes a mask of its own. The same picture is written as a second copy of the element, above the first, carrying the effect and a path `mask` ([ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md)) with `feather`. #901 scores mask modes. |
| Brush and Eraser strokes | Paint strokes on a layer in the Layer panel, on the RGB or alpha channel, for one frame, a custom span or the rest of the layer; strokes are animatable, Eraser strokes remove paint and layer colour, and a stroke can be sketched over time. | [Paint tools](https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/paint-tools-and-paint-strokes/paint-tools-brush-clone-stamp.html) | partial | A stroked `path` element over the footage stands for an RGB brush stroke, with keyed `points` and `trim_start`/`trim_end` for a write-on (capability map, "Elements and placement"). Painting the alpha channel or erasing pixels has no counterpart beyond a path `mask` with `invert` (ADR-0152). No pressure-sensitive brush tip. |
| Clone Stamp and paint clean-up | Clone strokes sample another place or frame of a layer and paint it over rigs, wires, scratches and blemishes, with clone presets, a source overlay and blending modes. | [Paint tools](https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/paint-tools-and-paint-strokes/paint-tools-brush-clone-stamp.html) | gap | **Arrives as** footage written by a skill. There is no paint stroke element. A static patch can be a masked `video` or `image` copy placed over the blemish, using a path `mask` and keyed `x`/`y`. |

### Keying

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Keylight 1.2 | The bundled screen-colour keyer, chained with Key Cleaner and Advanced Spill Suppressor by an animation preset. Adobe's guide defers its controls to the plug-in's own documentation; Foundry's Keylight reference lists Screen Colour, Screen Gain and Balance, Clip Black and White, view modes (Screen Matte, Status, Combined Matte), despill bias, and inside (hold-out) and outside (garbage) masks. | [Keying effects: Keylight](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html), [Keying: garbage and hold-out mattes](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/keying/keying.html); secondary: [Foundry Keylight reference](https://learn.foundry.com/nuke/content/reference_guide/keyer_nodes/keylight.html) | partial | `chroma{color, tolerance, softness, spill}`, all keyable ([ADR-0088](../../adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md); **Chroma** in `CONTEXT.md`; footage skill's "Green-screen key"). It has no clip black/white, screen gain/balance or matte view, and no garbage or core matte input. A garbage matte can be a separate `mask` in the same `effects` list. |
| Key Cleaner | Recovers alpha detail lost by a keyer, including detail lost to compression, before spill suppression. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | gap | No member restores matte detail. **Arrives as** keyed footage with alpha written by a skill, or as a new `chroma` parameter by ADR. |
| Advanced Spill Suppressor | Removes screen-colour spill from the foreground, in Standard mode or Ultra mode (Premiere's Ultra Key algorithm). | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | partial | `chroma.spill` suppresses screen colour on what the matte keeps (ADR-0088, `format/compositing.md`). It has no standalone spill effect without a key, and no choice of method. |
| Color Difference Key | Builds two partial mattes (A from a second colour, B from the key colour) and merges them into one alpha with black, white and gamma controls. It suits well-lit screens and semi-transparent smoke, shadows or glass. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | partial | The same job as `chroma`, by a different algorithm. Its separate partial-matte controls have no counterpart. |
| Color Range | Keys out a range of colours picked with plus and minus eyedroppers in Lab, YUV or RGB, with Fuzziness and per-component Min/Max. It suits multi-coloured or unevenly lit screens. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | partial | `chroma` keys one literal `#RRGGBB` with a `tolerance`. Several `chroma` entries in one ordered `effects` list key several colours. There is no colour-space choice. |
| Linear Color Key | A transparency ramp by closeness to a key colour in RGB, hue or chroma, with matching tolerance and softness. A second instance in Keep Colors mode restores a colour an earlier key removed. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | partial | `chroma`'s `softness` is the partial-alpha band above `tolerance`. Keep Colors (keying back in) has no counterpart. |
| Extract | Keys by brightness from a histogram of one channel (usually luminance), with black and white points and softness. It suits black, white or dark multi-coloured backgrounds. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | gap | No luminance matte exists. `blend: screen` or `multiply` composites dark or light grounds without a matte, which is a partial workaround for black- or white-ground footage. **Arrives as** footage with alpha written by a skill, or as a new matte operation by ADR (Premiere has precedent: its Luma Key, and Ultra Key's matte controls). |
| Unmult | Makes a solid black or white background transparent by brightness (Background Color, Black or White Level) while keeping the foreground's colours; suits smoke, sparks and paint splashes shot on black. New in 26.2. | [Keying effects: Unmult](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | gap | No luminance or unpremultiply matte. `blend: screen` (black ground) or `multiply` (white ground) composites the same footage without a matte ([ADR-0147](../../adr/0147-a-blend-mode-is-a-flat-static-field-of-five-values-and-the-finished-element-blends-last.md)), the same partial workaround as Extract. **Arrives as** footage with alpha written by a skill, or a new matte operation by ADR. |
| Luma Key (obsolete) | Keys by brightness with threshold, tolerance, edge thin and feather; in the Obsolete category. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | gap | As Extract. |
| Difference Matte | Compares the layer with a difference layer (a clean background frame) and removes what matches in position and colour, with matching tolerance, softness and blur before difference. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | refused by ADR-0150 | Its matte comes from another element ([ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md); capability map "Not by design"). An image source inside an effect is also deferred ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md) §"An image or luma source"). A skill could still write the result as footage with alpha. |
| Inner/Outer Key | Pulls a soft-edged subject (hair) from an inner (foreground) and an outer (background) mask, with extra and cleanup masks, edge thin, feather and threshold, and decontaminates the edge colour. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | gap | A `mask`'s `feather` gives an even soft edge (ADR-0152). The key from the band between two masks has no counterpart. **Arrives as** footage with alpha written by a skill. |
| Color Key (obsolete) | Removes one colour within a tolerance, with edge thin and edge feather; in the Obsolete category. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | covered | `chroma{color, tolerance, softness}` (ADR-0088). |
| Spill Suppressor (obsolete) | Removes the key colour from edges after a key; moved to Obsolete in 2014 in favour of Advanced Spill Suppressor. | [Keying effects](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/work-with-keying-effects/keying-effects.html) | covered | `chroma.spill` (ADR-0088). |
| CC Simple Wire Removal | Removes a thin wire or line between Point A and Point B, with a thickness and a removal style (such as Fade) that blends from the surrounding area; listed under Keying. | [Cycore plugins: Keying](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/cycore-plugins.html); [effect list](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html) | gap | **Arrives as** footage written by a skill. Its two endpoints could also be tracked as literal keyframes. |

### Matte refinement

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Refine Soft Matte | Refines a keyed matte's soft edges: Calculate Edge Details with an edge radius, Smooth, Feather, Contrast, Shift Edge, Chatter Reduction, More Motion Blur and edge-colour decontamination. | [Matte effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html) | gap | `chroma`'s `softness` and a `mask`'s `feather` are the only edge controls. **Arrives as** keyed footage with alpha written by a skill, or as `chroma` parameters by ADR. #905 also scores this. |
| Refine Hard Matte | The hard-edged counterpart of Refine Soft Matte. The effect list names it in the Matte category; the Matte effects page documents only Refine Soft Matte, Matte Choker and Simple Choker. | [Effect list (Matte)](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/effects-and-animation-presets/effect-list.html); [Matte effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html) | gap | As Refine Soft Matte. |
| Simple Choker | Shrinks or spreads a matte's edge by a small amount (Choke Matte), with a Matte view. | [Matte effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html) | gap | No choke exists. Changing `tolerance` moves where the key falls, not the matte's edge. Premiere Ultra Key's Choke is precedent for one parameter. |
| Matte Choker | Two-stage spread-then-choke passes, repeated by Iterations, that fill holes in a matte; at Gray Level Softness 100% it blurs only the alpha. | [Matte effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/matte-effects.html) | gap | As Simple Choker. The footage skill's fix for holes is to lower `tolerance` ("Green-screen key", step 3). |

### Deformation

| Feature | What it does | Adobe doc | Montagent status | Evidence |
|---|---|---|---|---|
| Puppet Position pin (Advanced engine; Deform pin in Legacy) | A pin that controls only position on a triangle mesh laid over the layer. | [Puppet tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-with-puppet-tools/animating-puppet-tools.html) | reachable by a skill | `montagent-character` rigs rigid parts with pivots and bakes their motion (`bake_rig.py`, workaround #499), so a jointed limb moves without a mesh. A bent, continuous mesh is reachable as a distortion drawn in code (`montagent-prerender`). |
| Puppet Advanced pin | A pin that controls position, scale and rotation. The mesh shears unless all three are animated. | [Puppet tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-with-puppet-tools/animating-puppet-tools.html) | reachable by a skill | A distortion drawn in code (`montagent-prerender`: "distortions"; **Pre-rendered footage**, ADR-0156 §7). The project names only the footage, so changing the pose means re-running the recipe. |
| Puppet Bend pin | Its position comes from the pins around it, and you control its scale and rotation (breathing, a wagging tail). | [Puppet tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-with-puppet-tools/animating-puppet-tools.html) | reachable by a skill | `montagent-prerender`, as for the Advanced pin. |
| Puppet Starch pin | Stiffens part of the mesh so it distorts less. | [Puppet tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-with-puppet-tools/animating-puppet-tools.html) | reachable by a skill | `montagent-prerender`, as for the Advanced pin. |
| Puppet Overlap pin | Sets which parts of the mesh draw in front where the mesh folds over itself (In Front, Extent). | [Puppet tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-with-puppet-tools/animating-puppet-tools.html) | reachable by a skill | For rigid parts, draw order is track `layer` order, which the rig sets (`montagent-character`, "the order the parts draw in"). In a mesh, it is part of the prerendered distortion. |
| Puppet mesh (Density, Expansion, Legacy Triangles) | The triangle mesh the pins deform, traced from the alpha, a closed path or a mask. Density sets triangle layout, Expansion grows the mesh past the outline, Mesh Rotation Refinement sets how far pins rotate; Legacy uses a Triangles count. | [Puppet tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-with-puppet-tools/animating-puppet-tools.html) | reachable by a skill | `montagent-prerender`. No format key deforms an element's pixels: the **Transform** is one flat matrix plus the **Projection** (ADR-0012, ADR-0167). |
| Puppet Sketch (recorded puppet animation) | Ctrl/Cmd-drag pins in real time, optionally to audio, with Speed and Smoothing record options; AE records the drag as keyframes. | [Puppet tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-with-puppet-tools/animating-puppet-tools.html) | gap | Montagent has no GUI. **Arrives as** literal keyframes written by a skill from a recorded or computed path, here or through the prerender recipe. #899 scores recorded motion sketches in general. |
| Liquify | Brush-based warp, turbulence, twirl, pucker, bloat, shift pixels and reflection tools on a layer, with a Freeze Area Mask and Reconstruction modes. | [Distort effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html) | reachable by a skill | A distortion drawn in code (`montagent-prerender`). ADR-0156 §2's tests decide whether a member could join the `effects` list. |
| Mesh Warp | A grid of Bezier patches (up to 31 rows and 31 columns) whose vertices and tangents reshape the layer, with elasticity. It is often used to morph between two images. | [Distort effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html) | reachable by a skill | `montagent-prerender`. A crossfade between the two warped images is a `transition` ([ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)). |
| Reshape | Morphs one mask shape into another on the same layer, using a source, a destination and a boundary mask, dragging the image with it; Elasticity controls how the pixels follow. | [Distort effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html) | reachable by a skill | Warping the pixels is a `montagent-prerender` distortion. Morphing a *vector* shape is covered by matching `points` lists on a `path` ([ADR-0162](../../adr/0162-a-morph-between-unlike-shapes-is-written-as-matching-vertex-lists-and-no-rule-resamples-them.md)). |

### Counts

50 rows: **covered 3**, **partial 7**, **reachable by a skill 9**, **owned by an open map 1**
(#591), **refused 5** (ADR-0167 ×4, ADR-0150 ×1), **gap 25**.

Verification on 2026-10-10 added four rows: Raw track and Unmult (gap), Mask Reference and
Brush and Eraser strokes (partial). No existing row changed status.

## Gaps worth a decision ticket

### 1. Tracking data as literal keyframes, written by a skill script

The rows: one-point and two-point tracks, a Raw track, a track applied to an effect's numbers, Stabilize
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

The rows: parallel and perspective corner pin, Mocha AE, and the 3D Camera Tracker, plus
Warp Stabilizer's Perspective method, which corner-pins the whole frame. (The Mask Tracker's
Perspective method only reshapes a mask's vertex list and stays in gap 1.) Corner pin and a camera are refused by ADR-0167. Two decisions are open:

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

The rows: Key Cleaner, Simple Choker and Matte Choker, Refine Soft and Hard Matte, Extract,
Unmult and Luma Key, Inner/Outer Key, multi-range Color Range, and Linear Color Key's Keep Colors mode.
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
