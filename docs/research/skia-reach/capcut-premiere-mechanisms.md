# What CapCut and Premiere do, by mechanism, for each capability on the Skia-reach map

Research for [#664](https://github.com/MBehtemam/Montagent/issues/664), part of the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). It replaces the from-memory table the
map was charted on.

[ADR-0003](../../adr/0003-general-video-editor-not-channel-tooling.md) names CapCut and Premiere
as the reference class. Under the map's standing rule, a capability outside that class may enter
only through the entry test. This file says, for each of the twelve capabilities, which side of
that line it falls on and what the mechanism is.

## How to read this

- **Sources.** Adobe's Premiere and After Effects help pages, and CapCut's own site, all read on
  2026-10-05. Every claim carries a link. The Premiere pages describe Premiere 26.x.
- **yes / partly / no.** "Partly" means the mechanism exists in a narrower form than the map's
  wording, and the detail section says which form. "Not found" means the vendor's pages I
  reached do not document it. It does not mean the feature is absent from the product.
- **"After Effects-only"** means the mechanism is documented for After Effects and I found no
  equivalent in the Premiere or CapCut pages.
- **CapCut's sources are weaker than Adobe's.** CapCut publishes no reference manual. Its site
  has a few help articles, feature pages and many marketing articles. I cite its help and
  feature pages where they exist, and say so where a claim rests on a marketing article.
  Section [Limits of this research](#limits-of-this-research) lists what could not be sourced.

## Summary

| # | Capability | CapCut | Premiere | Mechanism in the reference class | After Effects-only part |
|---|---|---|---|---|---|
| 1 | Keyframable non-transform properties; spring easing | partly | yes | A keyframe list on any effect property. Easing is a closed list of interpolation kinds plus Bezier handles. | Spring as a keyframe easing is in neither. It exists only inside named presets. |
| 2 | Blend modes | yes | yes | One enumerated blend mode per clip, beside opacity. Premiere lists 27. | none |
| 3 | Gradients | partly | yes | Linear or radial fill with colour stops, opacity stops and an angle. Separate gradient effects have animatable points and colours. | none |
| 4 | Letter spacing; per-letter animation | spacing yes, per-letter partly | spacing yes, per-letter partly | Tracking is a static text style value. Per-letter motion comes only as named presets such as Typewriter. | Free per-character animators with range selectors. |
| 5 | Repeat or stagger construct | not found | partly | Premiere's Clone effect duplicates an element "with offset timing". | A parametric repeater: N copies, one transform applied per copy. |
| 6 | Paths, stroke, trim-path, text on a path, morphing | partly | partly | Pen-drawn Bezier shapes with fill and stroke. CapCut curves text along an arc. | Trim-path, dashes, caps and joins, text on an arbitrary path, shape-path morphing. |
| 7 | Animated mattes; wipe, slide, push; animated clip | yes | yes | A track matte, a shape or text layer used as a mask, keyframed masks, and a closed list of named transitions with a few settings. | none |
| 8 | Motion blur | yes | partly | CapCut: a per-clip feature with blur, blend, direction and speed. Premiere: blur effects, and a shutter angle on the Transform effect. | A per-layer switch with composition shutter angle, phase and sample counts. |
| 9 | Shader vocabulary; pre-render as footage | yes (named effects) | yes (named effects) | A closed library of named effects with typed parameters. Motion-graphics work is brought in as footage or as a template. | An open script or shader file in the project is in neither editor. |
| 10 | 2.5D perspective transform | not found | yes | Basic 3D: Swivel, Tilt, Distance to Image. No camera, no depth sort. | Cameras, lights and depth-sorted 3D layers. Out of the map's scope already. |
| 11 | Speed ramps and time remapping | yes | yes | A speed curve over the clip: keyframes with Bezier handles in Premiere, points on a curve plus named presets in CapCut. | none |
| 12 | Vector sources: SVG, Lottie | not found | partly | Premiere imports Illustrator files and rasterizes them. Neither editor lists SVG or Lottie. | SVG import as editable shape layers. Lottie is not in After Effects' format list either. |

**Outside the CapCut/Premiere class, so the ADR-0003 entry test applies:**

- trim-path, stroke dashes, line caps and joins (part of 6)
- text on an arbitrary path (part of 6; CapCut has an arc only)
- shape-to-shape path morphing (part of 6)
- free per-letter animation (part of 4; the class has named presets only)
- a parametric repeat or stagger construct (5; Premiere's Clone effect is the nearest precedent)
- spring as a keyframe easing (part of 1)
- SVG and Lottie as sources (12)
- an open shader or script file (part of 9; already refused by ADR-0017)

**How this differs from the from-memory table.** Trim-path is confirmed outside the class, and
Premiere 26.0 even removed its nearest equivalent, the Write-On effect. The shader ticket splits:
a fixed vocabulary of named effects with typed parameters is exactly what both editors offer, so
it is inside the class. Only the open shader file is outside.

## 1. Keyframable non-transform properties, and spring easing

**Premiere: yes.** Keyframes attach to effect properties in general, not to transform only.
"In Premiere, effect properties can be animated by assigning keyframes to them", and a keyframe
holds "a value, such as spatial position, opacity, or audio volume"
([About keyframes](https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/about-keyframes.html)).
The switch is per property: "select the Toggle Animation icon to activate keyframes for an effect
property"
([Add keyframes](https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/add-keyframes.html)).

Colour and point values animate too. The 4-Color Gradient effect lets you "animate the positions
and colors of the four effect points", and Ramp lets you "animate the start and end positions"
([Generate effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/generate-effects.html)).
A few properties refuse keyframes or restrict them: Vertical Flip takes none
([Transform effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html)),
and Posterize Time's frame rate allows only Hold
([Time effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/time-effects.html)).

For text, shape and path layers, the documented animatable properties are transform ones:
"Position, Anchor Point, Scale, Rotation, or Opacity"
([Animate layers using the Properties panel](https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html)).
Whether a shape layer's fill colour, size or corner radius takes keyframes is **not found** in
the pages I reached.

**CapCut: partly.** The help page describes keyframes for "parameters like position, scale, and
opacity", found in the "Video, Transform, or Adjust" sections, and warns that some elements do
not support them
([Why can't I see keyframes in CapCut on PC?](https://www.capcut.com/help/keyframes-in-capcut-pc)).
The feature page adds "shape, opacity, and color"
([Keyframe animations](https://www.capcut.com/tools/keyframe-animation)).
A CapCut guide lists "Effect keyframes" that "animate specific visual effects applied to your
clips, such as color, brightness, or filters", and audio keyframes for volume
([How to add keyframes in CapCut PC](https://www.capcut.com/resource/how-to-add-keyframes-in-capcut)).
Mask position, size and rotation also take keyframes
([Mask video online](https://www.capcut.com/tools/mask-video-online)).
No page lists every keyframable parameter.

**Easing vocabulary.** Premiere offers a closed list: Linear, Bezier, Auto Bezier, Continuous
Bezier, Hold, Ease In and Ease Out
([Change the keyframe interpolation method](https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html)).
CapCut names "Ease In and Ease Out options"
([How to add keyframes in CapCut PC](https://www.capcut.com/resource/how-to-add-keyframes-in-capcut))
and "advanced keyframes and graphs"
([CapCut desktop](https://www.capcut.com/tools/desktop-video-editor)).

**Spring easing: in neither, as an easing.** No spring or bounce kind is in Premiere's
interpolation list above. Spring appears only inside named presets: the "Spring Motion"
transition, which brings "bouncy spring-like movement to scene changes", and the Slide
transition's "controls for speed, bounce, and direction"
([Modern transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html)).
I found no CapCut page that documents a spring easing.

**After Effects-only:** nothing in this capability except the expression route, which the map
already rules out. An expression is "a small piece of JavaScript code that you can plug into
animated properties"
([Expression basics](https://helpx.adobe.com/after-effects/desktop/work-with-expressions/expression-basics/expression-basics.html)).

## 2. Blend modes

**Premiere: yes.** The blend mode is one menu under the clip's Opacity effect: "select the arrow
next to Opacity ... Select a blend mode from the list of blend modes"
([Combine multiple video clips using Blend modes](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/combine-multiple-videos-or-stil-images-using-blend-modes.html)).
The documented vocabulary is 27 modes
([Blend mode options](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/blend-mode-options.html)):

| Group | Modes |
|---|---|
| Normal | Normal, Dissolve |
| Subtractive | Darken, Multiply, Color Burn, Linear Burn, Darker Color |
| Additive | Lighten, Screen, Color Dodge, Linear Dodge (Add), Lighter Color |
| Complex | Overlay, Soft Light, Hard Light, Vivid Light, Linear Light, Pin Light, Hard Mix |
| Difference | Difference, Exclusion, Subtract, Divide |
| (not in the page's category table) | Hue, Saturation, Color, Luminosity |

The page's category table names the first five groups, 23 modes. Its description table adds the
four colour-component modes in the last row, which makes 27.

**CapCut: yes.** CapCut's articles tell the user to "experiment with modes like 'Overlay,'
'Screen,' or 'Multiply'" and to "adjust opacity and blending settings"
([Overlay video on video](https://www.capcut.com/resource/overlay-video-on-video)).
A second CapCut article describes adjusting "blend mode and opacity" per layer in its multi-track
editor
([Blend modes for creative video and photo effects](https://www.capcut.com/create/blend-modes-creative-video-photo-effects)).
CapCut's full list of modes is **not found** on its own site.

**After Effects-only:** none. This is squarely inside the class.

## 3. Gradients

**Premiere: yes.** A gradient is a kind of paint on a text or shape layer's Fill, Stroke or
Shadow. The vocabulary is Solid, Linear Gradient and Radial Gradient. The parameters are colour
stops, an opacity stop with a percentage, an Angle for linear gradients, and a Location for each
stop or midpoint
([Add gradients](https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html)).
That page does not say whether the stops can be keyframed: **not found**.

Gradients that do animate are effects. 4-Color Gradient animates "the positions and colors of the
four effect points", and Ramp makes "a color gradient, either linear or radial" with animatable
start and end positions
([Generate effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/generate-effects.html)).
Ramp is now classed as Legacy
([List of effects and transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html)),
and a newer Gradient effect "supports both linear and radial gradients" with controls for
"position, scale, orientation, repetition, mirroring, interpolation, feathering, and grain"
([Modern effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html)).

**CapCut: partly.** The only CapCut source is a marketing article. It says "you can choose from
pre-defined color gradients or create your own" under gradient text effects, and also describes a
workaround: "stack text clips and add masking (such as a circle or rectangle)"
([How to create gradient text](https://www.capcut.com/resource/gradient-text)).
A gradient fill on a shape, and its parameters, are **not found**.

**After Effects-only:** none for the static paint. After Effects does document animating a
gradient fill on a shape layer
([Import SVG files](https://helpx.adobe.com/after-effects/desktop/import-files/import-svg-files/import-svg-files.html)),
which Premiere's fill page does not.

## 4. Letter spacing and per-letter animation

**Letter spacing: yes in both, as a static style value.** Premiere's text style parameters
include Tracking and Kerning
([Style parameters when applying from the style browser](https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/style-parameters-when-applying-from-style-browser.html)).
CapCut's text Style panel adjusts "letter spacing, line spacing, alignment"
([How to adjust text spacing and layout in videos](https://www.capcut.com/resource/adjust-text-spacing-and-layout-in-videos)).
Whether either editor keyframes tracking is **not found**.

**Per-letter animation: partly in both, as named presets only.**

- Premiere ships "Typewriter", which reveals "text or visuals with a typing effect, letter by
  letter", and "Text Animator", which animates "titles and text with professional motion presets,
  without the need for complex keyframing"
  ([Modern transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html),
  [Modern animations](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/animations.html)).
  The parameters of these presets are not documented on those pages.
- CapCut has built-in text animations picked from an Animation tab; its article shows selecting
  "the 'Typewriter' effect to make the text appear letter by letter"
  ([Typewriter effects](https://www.capcut.com/resource/typewriter-effects)).

**After Effects-only: the free per-character animator.** After Effects adds *animator groups* to
a text layer. Each group holds animator properties (Position, Scale, Skew, Rotation, Tracking,
Line Spacing, Character Offset, Blur and others) and one or more *selectors* that decide which
characters they touch. A Range selector has Start, End and Offset, and you "set keyframes for
Start or End properties"; there are also Wiggly and Expression selectors
([Animating text](https://helpx.adobe.com/after-effects/desktop/animating-text/text-animation/animating-text.html)).
Tracking itself animates through a "Tracking Amount" animator property
([Examples and resources for text animation](https://helpx.adobe.com/after-effects/desktop/animating-text/text-animation-examples/examples-resources-text-animation.html)).
Nothing like the selector exists in the Premiere or CapCut pages.

## 5. A repeat or stagger construct

**Premiere: partly.** Two effects come close.

- **Clone** (formerly "FI: Clone FX"): "Duplicate and animate visual elements such as logos,
  text, or footage with offset timing"
  ([Modern effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html)).
  This is one construct standing for several offset copies, which is the map's question. Its
  parameters are not documented on that page.
- **Replicate** exists but is now classed as Legacy
  ([List of effects and transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html)).
  Its parameters are **not found** in the current pages.

**CapCut: not found.** I found no CapCut page that documents a repeat, clone or stagger feature.

**After Effects-only: the parametric repeater.** On a shape layer the Repeater "creates multiple
copies of a shape, applying a specified transformation to each copy", and a Wiggle Transform
placed after it can "randomize the transformations of each repeated shape separately"
([Shape attributes, paint operations, and path operations](https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/shapes-and-shape-attributes/shape-attributes-paint-operations-path.html)).
The stagger across text characters is the Range selector of section 4.

## 6. Paths, lines, trim-path, stroke properties, text on a path, morphing

**Paths and lines. Premiere: yes.** The Pen tool draws "custom shapes" from vertex points,
direction lines and Bezier curves
([Overview of Pen tool](https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-with-pen-tool.html)),
and "the simplest path that you can draw with the Pen Tool is a straight line with two vertex
points"
([Draw a straight line](https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-straight-line.html)).
There are also Rectangle, Ellipse and Polygon tools
([Create a shape](https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html)).
**CapCut: not found.** I found no CapCut page documenting a freeform path or pen tool.

**Stroke properties. Premiere: partly.** A shape's Appearance has Fill, Stroke and Shadow; Stroke
can be "Outer, Inner, or Center", there can be several strokes, and the width is adjustable
([Create a shape](https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html),
[Draw a straight line](https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/draw-a-straight-line.html)).
A separate Stroke effect adds "bold, customizable outlines to any element"
([Modern effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html)).
Dashes, line caps and line joins are **not found** in the Premiere pages.

**Trim-path: no in both; After Effects-only.** Premiere's closest tool was the Write-On effect,
and it is listed under "Removed in version 26.0 (will not render)"
([List of effects and transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html)).
After Effects' Trim Paths is a path operation on a shape layer: "Animate the Start, End, and
Offset properties to trim a path", with a choice of trimming several paths "simultaneously" or
"individually". The same page documents dashed strokes (a Dashes group of up to three dashes,
with an animatable Offset), Line Cap and Line Join
([Shape attributes, paint operations, and path operations](https://helpx.adobe.com/after-effects/desktop/drawing-painting-and-paths/shapes-and-shape-attributes/shape-attributes-paint-operations-path.html)).

**Text on a path. CapCut: partly; Premiere: not found.** CapCut has a "Curve" control on text
that can "fine-tune the curve to any degree"
([Create curved text](https://www.capcut.com/resource/create-curved-text)).
That is one parameter bending text along an arc, not text along a drawn path. Nothing in the
Premiere pages I reached documents text on a path. After Effects does it in full: a text layer's
Path Options take a mask as the path, with First Margin, Last Margin, Reverse Path and related
properties, and "animating Path Options properties is an easy way to animate text along a path"
([Animating text](https://helpx.adobe.com/after-effects/desktop/animating-text/text-animation/animating-text.html)).

**Shape-to-shape morphing. Premiere: partly, for masks only.** A mask can be reshaped per frame,
which creates keyframes: "Frame-level adjustments let you reposition or reshape the mask on
specific frames, creating keyframes automatically"
([Adjust masks at the clip or frame level](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-masks-at-the-clip-or-frame-level.html)).
Keyframing the path of a drawn shape layer is **not found**. A "Shape Flow" preset generates
"flowing shape-based animations", with no documented parameters
([Modern transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html)).
**CapCut: not found.** After Effects owns the general mechanism: "set keyframes for the Mask Path
or Path property, set paths at each keyframe, and After Effects will interpolate between these
specified values"
([Managing and animating shape paths and masks](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animate-shape-paths-and-masks/animating-shape-paths-masks.html)).

## 7. Animated mattes, more transition kinds, and the animated clip

**Mattes. Premiere: yes, three mechanisms.**

- **Track Matte Key.** It "reveals one clip (background clip) through another (superimposed
  clip), using a third file as a matte". White is opaque, black is transparent, grey is partial.
  A moving matte is called a traveling matte, and "because you can use a video clip as a matte in
  the Track Matte Key, the matte can change over time"
  ([Keying effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html)).
- **A layer used as a mask.** A text or shape layer has a "Mask with Text or Mask with Shape"
  checkbox, with "Mask Only Fill" and "Invert"
  ([Mask with text or shape](https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/mask-with-text-or-shape-using-properties-panel.html)).
  Animating the layers under it gives a reveal: "the shape layer becomes a window through which
  you see the animated text"
  ([Create reveal animations using masking techniques](https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-reveal-animations-using-masking-techniques.html)).
  This is the nearest thing to the format's animated `clip`.
- **A mask on an effect.** Ellipse, Rectangle and Pen masks limit where an effect applies
  ([Create masks using shapes](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-shapes.html)).
  Their parameters are Feather, Expansion, Opacity and Invert
  ([Adjust mask properties](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-mask-properties.html)),
  and they can be tracked or keyframed
  ([Track masks](https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/track-masks.html)).

**Mattes. CapCut: yes.** A clip takes one mask from a list of shapes, "circles, stars, hearts,
filmstrips, and more". The parameters are "feather, size, position, rotation, and inversion", and
keyframes can "update the mask's position, size, and rotation"
([Mask video online](https://www.capcut.com/tools/mask-video-online)).
A matte taken from another track is **not found** for CapCut.

**Transitions. Premiere: yes, a closed named list.** The classic list has a Slide group (Band
Slide, Center Split, Push, Slide, Split, Whip) and a Wipe group (Band Wipe, Barn Doors, Clock
Wipe, Radial Wipe, Wipe and others)
([List of Video transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-transitions.html)).
The version 26.0 table keeps Push, Slide and Wipe as Legacy and adds newer Push, Linear Wipe,
Clock Wipe, Radial Wipe, Soft Wipe and more
([List of effects and transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html)).
A transition's settings are few: edge selectors for direction, Start and End percentages, Border
Width, Border Color, Reverse, Anti-Aliasing Quality, and Custom for the rare transition with its
own settings
([Change transition settings](https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/change-transition-settings.html)).

Premiere also has wipes as effects, driven by one keyframable number. Linear Wipe "performs a
simple linear wipe of a clip in a specified direction", and Gradient Wipe makes pixels
transparent by the luminance of another track "as Transition Completion increases"
([Transition effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transition-effects.html)).

**Transitions. CapCut: yes, a closed named library.** CapCut's feature page lists "fade, glitch,
slide, zoom, and 3D spin effects", browsed by category, with "speed and duration" as the
controls
([Free video transitions](https://www.capcut.com/tools/video-transition)).
A CapCut article describes wipe and slide-wipe transitions and how to apply them
([Wipe transitions in film](https://www.capcut.com/resource/wipe-transitions-in-film)).
A list of CapCut's transition names, and a transition named "push", are **not found**.

**After Effects-only:** none. After Effects has track mattes too
([Track Mattes and Traveling Mattes](https://helpx.adobe.com/after-effects/desktop/work-with-transparency-and-compositing/work-with-track-mattes-and-traveling-mattes/track-mattes-and-traveling-mattes.html)),
but the capability is inside the class.

## 8. Motion blur

**CapCut: yes.** Motion blur is a feature on a video clip, under "Video". The parameters are
"Blur" and "Blend" degrees, a motion direction ("forward, backward, and both") and a speed
("once/twice/4 times/6 times")
([Add motion blur to videos](https://www.capcut.com/tools/motion-blur)).
The page does not say what the feature samples. It is described for footage, not for keyframed
movement of a layer.

**Premiere: partly.** The documented pieces are these.

- **Directional Blur** "gives a clip the illusion of motion", applied equally either side of each
  pixel
  ([Blur and Sharpen effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/blur-and-sharpen-effects.html)).
  It is a directional smear, not a blur computed from movement.
- **Presets with blur built in.** 3D Spinback uses "motion blur and adjustable easing"
  ([Modern transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/transitions.html)).
- **The Transform effect's Shutter Angle.** This is the usual route to blur on keyframed
  movement, but Adobe's current help page describes Transform only as adjusting "position, size,
  rotation, and opacity"
  ([Transform effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html)).
  The only Adobe-hosted evidence I found is user posts on Adobe's community forum, for example a
  feature request titled "Add Motion Blur (Shutter Angle) To Default Motion Panel"
  ([Adobe Community](https://community.adobe.com/feature-requests-730/add-motion-blur-shutter-angle-to-default-motion-panel-1329594)).
  The title suggests the fixed Motion effect has no such control. I read the thread's title and a
  search summary of it, not the thread itself. Treat the Shutter Angle mechanism as
  **not confirmed from documentation**.

**After Effects-only: sampled motion blur as a layer property.** "You enable motion blur for each
layer individually" with the Motion Blur layer switch, and it applies to motion you animate, not
to "motion that exists within a layer"
([Assorted animation tools](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/assorted-animation-tools/assorted-animation-tools.html)).
The composition holds the parameters: Shutter angle (1° to 720° in the page's examples), Shutter
phase, Samples per frame and Adaptive sample limit
([Composition basics](https://helpx.adobe.com/after-effects/desktop/work-with-compositions/composition-settings/composition-basics.html)).
The cost is stated too: "Motion blur slows rendering".

## 9. A shader vocabulary, and pre-rendering a code-drawn piece as footage

**Named effects with typed parameters: yes in both. This is the reference-class mechanism.**

- Premiere's Effects panel is a fixed, categorised list. The version 26.0 table enumerates it:
  Blur & Sharpen, Distort, Generate, Keying, Lights & Glows, Stylize, Transform and so on, each
  with named effects
  ([List of effects and transitions](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html)).
  The list is extended only by installed plug-ins: "Premiere offers a variety of built-in and
  third-party plug-in effects", stored in a system Plug-ins folder, and a project opened without
  them shows the effects as offline
  ([Types of effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html)).
  A plug-in is installed software, not a file in the project.
- CapCut offers "a diverse collection of video effects and filters" picked from a library
  ([Free video transitions](https://www.capcut.com/tools/video-transition)),
  and its motion-blur page mentions "preset motion blur effects ... in the filters and effects
  panels"
  ([Add motion blur to videos](https://www.capcut.com/tools/motion-blur)).
  A list of CapCut's effects and their parameters is **not found**.

**An open shader or script file in the project: in neither.** I found no Premiere or CapCut page
documenting user-written shader code. This agrees with the map's standing rule that ADR-0017
stays closed. I did not check whether After Effects documents one.

**Pre-rendered or externally built pieces brought in as footage: yes in Premiere.** This is how
Premiere reaches motion-graphics work it cannot author.

- Dynamic Link places "a 'live' After Effects composition in the Premiere timeline", and Render
  and Replace bakes it when performance suffers
  ([Different ways to work with graphics](https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/different-ways-to-work-with-graphics.html)).
- A Motion Graphics template (`.mogrt`) is "packaged as templates with easy-to-use controls
  designed to be customized in Premiere", giving "the power of After Effects motion graphics"
  ([Overview of Motion Graphics templates](https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html)).

Both are precedents for the map's pre-render skill: the animation is built elsewhere and arrives
on the timeline as a clip with, at most, a few exposed parameters. CapCut's equivalent is
**not found**, beyond importing an ordinary video file.

## 10. A 2.5D perspective transform

**Premiere: yes, and it matches the map's scope exactly.** Basic 3D "lets you rotate and move
clips in 3D space, adjusting them around horizontal and vertical axes, and bringing them closer
or further away", with an optional specular highlight
([Perspective effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html)).
It is one effect on one clip. There is no camera and no depth sorting between clips.

The current Premiere page does not name the parameters. Adobe documents the same-named effect for
After Effects with three: **Swivel** ("horizontal rotation (rotation around a vertical axis)"),
**Tilt** ("vertical rotation (rotation around a horizontal axis)") and **Distance To Image**
("the distance from the image to the viewer")
([Obsolete effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/obsolete-effects.html)).
That these are also the names in Premiere is my inference from the shared effect name, not a
quote from a Premiere page.

Premiere also has **3D Rotate**, which applies "full-space 3D rotation to your footage", and
**Corner Pin**, which distorts "by independently adjusting its four corners"
([Modern effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html)).

**CapCut: not found.** CapCut's pages mention 3D only inside presets, such as "3D spin effects"
among transitions
([Free video transitions](https://www.capcut.com/tools/video-transition)).
A per-clip tilt or perspective control is not documented on the pages I reached.

**After Effects-only: the real 3D scene.** A 3D layer gains "Position (z), Anchor Point (z),
Scale (z), Orientation, X Rotation, Y Rotation, Z Rotation", and "only 3D layers interact with
shadows, lights, and cameras"
([3D layers](https://helpx.adobe.com/after-effects/desktop/work-with-layers/3d-layers/3d-layers.html)).
Adobe itself treats Basic 3D as the lesser tool: in After Effects it is obsolete and the advice
is to "use the 3D layer switch"
([Obsolete effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/obsolete-effects.html)).
The map excludes cameras and depth sorting, so the After Effects part is already out of scope.

## 11. Speed ramps and time remapping

**Premiere: yes.** There are three ways to change speed: the Speed/Duration command, the Rate
Stretch tool and Time Remapping
([Different ways to change clip speed and duration](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/different-ways-to-change-clip-speed-and-duration.html)).
Time Remapping is the ramp. What is keyframed is speed along the clip: you "create a speed
keyframe" on the clip's rubber band, "drag the rubber band between keyframes to adjust speed for
specific portions of the clip", and "drag the Bezier handles that appear to adjust the rate of
speed change". Speed reads "as a percentage of the original speed"
([Change clip speed and duration using Time Remapping](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-and-duration-using-time-remapping.html)).
A freeze of part of a clip uses the same feature
([Freeze a frame for a portion of a clip using Time Remapping](https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/freeze-a-frame-for-a-portion-of-a-clip-using-time-remapping.html);
I read this page's title only).

How in-between frames are made is a separate closed choice: Frame Sampling, Frame Blending or
Optical Flow
([Apply Time Interpolation Methods](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html)).

**CapCut: yes.** Speed has a "Curve" tab with named presets ("Montage, Bullet", "Jump Cut, Hero")
or a custom curve
([Speed curve](https://www.capcut.com/tools/speed-ramp)).
The custom editor is a graph of points: dragging a point horizontally sets when, vertically sets
the pace, and "the upper section of the graph speeds up the video up to 10x, and the lower
section slows your video up to 0.1x". A "Smooth slow mo" checkbox is the frame-interpolation
choice
([How to do velocity on CapCut](https://www.capcut.com/resource/how-to-do-velocity-on-capcut)).

**After Effects-only:** none. After Effects has time-remapping as well
([Time-stretching and time-remapping](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/time-stretching-and-time-remapping/time-stretching-time-remapping.html)),
but the capability is inside the class.

## 12. Vector sources: SVG and Lottie

**Premiere: partly. Vector files come in, but as pixels, and SVG is not among them.** The
supported still-image formats include "AI, EPS" for Adobe Illustrator; SVG does not appear in the
list, and neither does Lottie
([Supported file formats](https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html)).
An Illustrator file is rasterized on import: "Premiere converts path-based Illustrator art into
the pixel-based image format used by Premiere, a process known as rasterization"
([Import Photoshop and Illustrator files](https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html)).
Graphics made inside Premiere stay vector under the Vector Motion effect, which transforms them
"without rasterizing them"
([Edit vector graphics using Vector Motion effect](https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/edit-vector-graphics-using-vector-motion-effect.html)).

**CapCut: not found.** CapCut's help page lists supported import formats for iOS and Android
only: BMP, JPEG, PNG, WebP, HEIF and common video containers. SVG and Lottie are not in either
list, and "if your file format is not listed above, it cannot be imported into CapCut"
([Why can't local assets be recognized during import?](https://www.capcut.com/help/can-not-local-assets-be-recognized-during-import)).
A format list for CapCut desktop is **not found**.

**After Effects-only: SVG as an editable source.** After Effects imports "SVG files as native,
editable shape layers", either as footage ("a single, continuously rasterizable vector layer")
or as a composition of shape layers
([Import SVG files](https://helpx.adobe.com/after-effects/desktop/import-files/import-svg-files/import-svg-files.html)).
Its format list marks SVG "currently in beta"
([Supported file formats](https://helpx.adobe.com/after-effects/desktop/get-started/supported-file-formats/supported-file-formats.html)).

**Lottie: in none of the three, by their format lists.** Lottie is absent from both Adobe lists
above and from CapCut's. I did not find an Adobe page documenting Lottie import or export.

## Limits of this research

**Could not be sourced from a primary source:**

- CapCut's full blend-mode list, effect list and transition list.
- Whether CapCut has a repeat or clone feature, a pen or freeform path tool, trim-path, a
  per-clip 3D tilt, or a matte taken from another track.
- CapCut desktop's supported import formats. Only the mobile lists were found.
- Whether Premiere keyframes a shape layer's fill colour, size, corner radius or path, and
  whether it keyframes text tracking or gradient stops.
- Whether Premiere has text on a path.
- Stroke dashes, caps and joins in Premiere.
- The parameters of Premiere's Clone, Replicate, Text Animator, Typewriter and Shape Flow.
- The Premiere Transform effect's Shutter Angle. Adobe's help page does not mention it; only
  Adobe Community posts do, and I read those through a search summary.
- Basic 3D's parameter names for Premiere specifically. They are quoted from the After Effects
  page for the same-named effect.

**Access problems on 2026-10-05:**

- Adobe's older Premiere pages (`helpx.adobe.com/premiere-pro/using/...`) now redirect to the
  newer, shorter pages. The older effect reference carried parameter-level detail that the new
  pages drop, which is why several parameters above are unsourced.
- The Internet Archive's availability API returned HTTP 429 on the first request, so I did not
  use archived copies of the older pages.
- A third-party page reader was unavailable, so CapCut and Adobe pages were read directly.
- I did not run either product. Nothing here is from observation of the applications.

**Weight of the CapCut evidence.** Pages under `capcut.com/help/` and `capcut.com/tools/` are
CapCut describing its own product. Pages under `capcut.com/resource/` and `capcut.com/create/`
are CapCut-published articles written for search traffic. They are first-party but loosely
worded, and one of them describes other vendors' tools beside CapCut's. Where a claim rests only
on such an article, the section says so.
