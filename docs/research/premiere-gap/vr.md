# Premiere gap: VR, 360° and immersive video

Research for [#909](https://github.com/MBehtemam/Montagent/issues/909), on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882).

- **Date:** 2026-10-10.
- **Premiere side:** the "Edit VR content" chapter of Adobe's Premiere user guide
  (`helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/`, Premiere 26.x), plus the
  export-settings reference and the monitor display-mode page. helpx.adobe.com could not be
  fetched (DNS refused by this environment's network policy). Every chapter row therefore comes
  from WebSearch excerpts restricted to helpx.adobe.com, or from the agent's own knowledge.
  The 11 immersive effects and 8 immersive transitions are carried over from the effects research
  ([#885](https://github.com/MBehtemam/Montagent/issues/885),
  `docs/research/premiere-gap/effects.md` on `research/premiere-gap-effects`), not researched again.
- **Source marks** (in the Adobe doc URL column):
  - **(F)** fetched page. None this time.
  - **(S)** search excerpt from helpx.adobe.com.
  - **(S, #885)** carried over from the effects research, which was itself built from search excerpts.
  - **(K)** the agent's own knowledge. The URL is the page where the feature should be documented, and the excerpts did not confirm it.
- **Montagent side:** tag `v0.2.0` (`51aa4b5` = `origin/main`): `CONTEXT.md`,
  `schema/montagent.schema.json`, `docs/adr/` (in particular 0002, 0023, 0040, 0077, 0145,
  0167, 0168, 0172, 0184), `crates/montagent-core/src/media/probe.rs`, the ten MCP tools in
  `crates/montagent/src/mcp.rs`, and `skills/`.
- **Rows:** 57. **have** 2 · **have (skill)** 0 · **partial** 8 · **missing** 47 · **refused** 0.
  By source: 0 (F), 32 (S), 19 (S, #885), 6 (K).

## The one fact behind most rows

Montagent v0.2.0 has **no spherical model**. A source is a flat raster: its dimensions, its PAR and
its container rotation (ADR-0023). A frame is flat `width × height` (`$defs/Frame`). Projection
(ADR-0167/0168) turns a flat element as a plane in front of an eye; it does not map onto a sphere.
`probe` reads only the `Display Matrix` side data
(`crates/montagent-core/src/media/probe.rs`, `side_data_list` at ~L701) and ignores spherical
(`Spherical Mapping`) and stereo-3D side data. `render` writes H.264 + stereo AAC in MP4 with no
spherical, stereo or ambisonic metadata. The mix bus is 48 kHz stereo, and every input is
converted to it first (ADR-0077 §8), so a four-channel ambisonic track is folded down as if it
were ordinary surround.

No ADR refuses a spherical model, VR metadata or ambisonics. ADR-0167 refuses a **camera** and
**depth sorting** ("After Effects-only"), but no Premiere VR feature needs a scene camera: the VR
viewer is a monitor feature, and VR Plane to Sphere *projects* a flat layer, which ADR-0167's
"projected, never placed" thesis would admit by mechanism. So nothing here is `refused`.

A few things can be built by hand from flat mechanisms, and these are the `partial` rows:

- **Yaw (horizontal re-centring) of an equirectangular frame** is exactly a horizontal wrap. Two
  `video` elements on the same source, side by side, with keyed `x`, rotate the sphere about its
  vertical axis. This is exact, and seamless when `x` is whole pixels. Tilt and roll are not a wrap:
  they need a spherical remap, and nothing does that.
- **Stereo layouts are rectangles.** An over/under or side-by-side frame is two halves. `clip`
  (the static frame-space window, `CONTEXT.md` "Clip") and `x`/`y` can take one eye, or rearrange
  the halves from over/under to side-by-side. That is not a layout *declaration*, and it does not
  change field of view.

## Inventory

### VR footage and sequences

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR editing (360° and 180°, mono and stereo) | Clips and sequences carry VR properties: projection (equirectangular), layout (monoscopic, stereoscopic over/under, side-by-side) and captured view (H × V degrees). Monitors, effects and export read them. | [VR editing in Premiere Pro][OVR] (S) | missing | No spherical model: `$defs/Frame` is `{width, height}`; ADR-0023 source dimensions are dims, PAR and rotation only | A projection, layout and captured-view model on sources and on the project. |
| Detect VR properties from file metadata | On import, Premiere reads spherical metadata (projection, layout, captured view) from the file and sets the clip's VR properties. | [Detect VR properties][DET] (S) | missing | `probe.rs` reads only `Display Matrix` side data; spherical and stereo-3D side data are ignored | Reading spherical metadata (e.g. MP4 `sv3d`/`st3d`, ffprobe `Spherical Mapping`, `Stereo 3D`). |
| Detect VR by frame aspect | With no metadata, at certain frame heights: 1:1 is read as stereo over/under, 2:1 as mono, 4:1 as stereo side-by-side. | [Detect VR properties][DET] (S) | missing | — (no VR interpretation exists to default) | The aspect heuristic, and the property it would set. |
| Interpret VR footage | Project panel → Modify → Interpret Footage → VR Properties: *Use Properties from File* or *Conform to* (Projection, Layout, Captured View). Used when metadata is absent or wrong. | [Interpret VR footage][INT] (S) | missing | ADR-0002 rules out an asset table, so a per-source override would sit on the element; no element carries a VR field | A per-element VR interpretation (projection, layout, H/V captured view). Not refused: ADR-0002 rules out where Premiere stores it, not the field. |
| Clip VR properties shown in Properties panel | Clips with VR metadata list VR Projection, VR Layout, VR Captured View. | [Interpret VR footage][INT] (S) | missing | `query`/`probe` report dims, PAR, rotation, alpha; nothing spherical | Reporting the properties in `probe`/`query`. |
| VR properties on sequences | Sequence Settings → VR Properties: Projection (None / Equirectangular), Layout (Monoscopic, Stereoscopic over/under, side-by-side), Horizontal and Vertical Captured View. | [Assign VR properties to sequences][SEQ] (S) | partial | `$defs/Frame` takes any integer `width`/`height`, so a 4096×2048 (2:1) or 4096×4096 frame can be written | The VR Properties block itself: projection, layout and captured view on the project, read by render and export. |
| VR sequence presets | File → New Sequence (or New Project → View All Presets) → VR: Monoscopic 29.97 (4096×2048), Stereoscopic 29.97 (4096×4096), each with an Ambisonics variant; older Legacy → VR presets at 2048×2048. | [Assign VR properties to sequences][SEQ], [Monitor Ambisonics audio][MON] (S) | partial | `create_project` writes a starter at any frame size; `fps` is an integer (`schema` `properties.fps`) | 29.97 (integer fps only, see editing.md), the VR properties, and the Ambisonics (multichannel) variant. |
| New sequence from a VR clip | Project panel → right-click a VR clip → New Sequence From Clip; the sequence inherits its VR properties. | [Assign VR properties to sequences][SEQ] (S) | missing | `create_project` takes no source to match | Matching a project to a source, and VR properties to inherit. |
| Mixed resolutions and layouts in one sequence | Clips of different sizes and mono/stereo layouts share a sequence; VR Projection conforms each. | [Assign VR properties to sequences][SEQ] (S) | partial | Mixed sizes are routine: `fit` (`cover`/`contain`/`literal`) scales each source to its box (ADR-0015, ADR-0023) | Per-element layout conformance; a mono and a stereo source cannot be told apart or converted except by hand (see "Output layout" below). |
| 180° VR (VR180) | Captured view of 180° × 180°, mono or stereo; viewable in the monitors. | [VR editing in Premiere Pro][OVR] (S) | missing | — | A captured-view value and half-sphere viewing and export. |

### Viewing VR in the monitors

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Video display in Source and Program Monitors | Monitor → VR Video → Enable, or the Toggle VR Video Display button. The monitor shows a rectilinear view into the sphere; disabled when content has no VR properties. | [VR editing in Premiere Pro][OVR], [Choose a display mode][DISP] (S) | missing | There is no live monitor. `frame` (PNG) and `preview` (proxy MP4) show the flat equirectangular picture (ADR-0078) | A viewport into the sphere, live or as a `frame` option. |
| Look around interactively | Drag in the monitor, during playback or while paused, to change the view direction. | [VR editing in Premiere Pro][OVR] (S) | missing | No interactive monitor | Interactive view direction. |
| 360-degree panning controls | Pan (horizontal) dial and tilt slider beside the monitor; double-click the frame to re-centre. Changes the view only, not the footage. | [Use 360-degree panning][PAN] (S) | missing | — | View-only pan/tilt and re-centring. |
| VR Video settings (monitor field of view, stereoscopic view) | Monitor → VR Video → Settings: monitor view horizontal/vertical FOV, which eye (left, right, anaglyph) is shown for stereo. | [VR editing in Premiere Pro][OVR] (K) | missing | — | Viewer FOV and eye selection. The excerpts did not confirm the anaglyph option. |
| Hide VR video view controls | Right-click a monitor → VR Video → uncheck Show Controls; the state is kept per monitor and saved with the project. | [Hide VR video view controls][HIDE] (S) | missing | — | Nothing to hide: there are no viewer controls. |
| Head-mounted display monitoring | Play the sequence to a supported headset (Oculus Rift named) while editing, with Adobe Immersive Environment. | [Set up a head-mounted display][HMD] (S) | missing | — | Headset playback. The excerpts did not confirm Windows Mixed Reality support. |
| Simulate viewing platforms | The VR display simulates a headset or desktop YouTube/Facebook 360 viewing. | [Choose a display mode][DISP] (S) | missing | — | Any VR display. |

### Reorienting 360° footage

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Three-axis video rotation (correct the zero angle) | VR Projection effect → Pan (yaw), Tilt (pitch), Roll; rotate the sphere in the VR viewer until the content is upright. Keyframeable. | [Perform three-axis video rotation][ROT] (S) | partial | Yaw is a horizontal wrap of the equirectangular frame: two `video` elements on the same `source`, side by side, with keyed `x` (ADR-0012 keyframes). Exact at whole-pixel `x` | Tilt and roll (a spherical remap), a single control for the rotation, and doing it on one element. Stereo needs one wrap pair per eye with `clip`. |
| Restage correctly aligned video and audio | After rotating the video, copy the same Pan/Tilt/Roll into Panner – Ambisonics so sound and picture line up. | [Restage aligned video and audio][RES] (S) | missing | No ambisonic audio (ADR-0077 §8 stereo bus) | Sound-field rotation, and anything to align it to. |

### The VR Projection effect, by parameter

The effects research has one row for VR Projection (`missing`). The chapter documents its
parameters separately, and some of them can be built from flat mechanisms.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Input Auto VR Properties | On by default: the effect reads the clip's VR properties. Turn off to set Input Layout, Input Horizontal FOV and Input Vertical FOV by hand. | [Immersive video effects and transitions][IMT] (S) | missing | No VR properties to read (see "Detect VR properties") | Input layout and FOV, read or written. |
| Output Layout | Converts between monoscopic, stereoscopic over/under and side-by-side. | [Immersive video effects and transitions][IMT] (S) | partial | Halves are rectangles: two `video` elements, each `clip`ped to one eye and moved with `x`/`y`, turn over/under into side-by-side, or take one eye for a mono deliverable (`CONTEXT.md` "Clip") | A layout field; the conversion is hand-built from two or more elements per clip and is not declared anywhere. |
| Output Horizontal / Vertical FOV | Re-projects to a different captured view (e.g. 360° → 180°). | [Immersive video effects and transitions][IMT] (S) | missing | — | A spherical re-projection. A 360→180 crop of a mono equirect is a `clip` of the middle half, but the vertical axis and stereo cases are not. |
| Disparity Adjustment | Shifts the eyes against each other to set stereo convergence; only when input and output are both stereoscopic. | [Immersive video effects and transitions][IMT] (S) | partial | Convergence in equirectangular stereo is a per-eye yaw: one wrap pair per eye (see "Three-axis video rotation"), offset in opposite directions | A field for it; four elements per stereo clip by hand. |
| Stretch to Fill Frame | Scales the projected result to fill the sequence frame. | [Immersive video effects and transitions][IMT] (K) | have | `fit: cover`/`literal` with the frame as the box (ADR-0015) | — |
| Auto VR properties and frame layout on every immersive effect | Each immersive effect reads the clip's layout so it treats each eye separately and wraps seams. | [Immersive video effects][IMF] (K) | missing | — | Per-eye, seam-aware effects. |

### Immersive video effects (from #885)

Carried over from `effects.md` §"Immersive Video"; see that file for its evidence. One status
differs: VR Rotate Sphere is `partial` here because of the yaw wrap described above.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Blur | Blur that wraps seamlessly across a 360° frame. | [Immersive video effects][IMF] (S, #885) | missing | `blur` is planar (ADR-0040) | Spherical wrap. |
| VR Chromatic Aberrations | Colour fringing at the edges of a 360° frame. | [Immersive video effects][IMF] (S, #885) | missing | — | Spherical model, and channel offsets. |
| VR Color Gradients | Gradient fills that blend across the sphere. | [Immersive video effects][IMF] (S, #885) | missing | Planar gradients only (ADR-0149) | Spherical model. |
| VR De-Noise | Noise reduction for 360° footage. | [Immersive video effects][IMF] (S, #885) | missing | — | Denoise, and the spherical model. |
| VR Digital Glitch | Digital-glitch distortion across the sphere. | [Immersive video effects][IMF] (S, #885) | missing | — | Spherical model. |
| VR Fractal Noise | Fractal-noise textures over the sphere. | [Immersive video effects][IMF] (S, #885) | missing | — | Spherical model and fractal noise. |
| VR Glow | Glow round bright areas, seamless across the sphere. | [Immersive video effects][IMF] (S, #885) | missing | `glow` is planar (ADR-0156) | Spherical wrap. |
| VR Plane to Sphere | Projects a flat layer (logo, title) onto equirectangular footage, with the distortion that keeps it looking flat in a headset. | [Immersive video effects][IMF] (S, #885) | missing | ADR-0167 projects a flat element onto a plane, not a sphere | Equirectangular projection of a flat element. ADR-0167's thesis ("projected, never placed") would admit it by mechanism. |
| VR Projection | Re-projects and rotates 360° footage (parameters split out above). | [Immersive video effects][IMF] (S, #885) | missing | — | Spherical model. Its layout, disparity and fill parameters are rowed above. |
| VR Rotate Sphere | Rotates 360° footage about X, Y and Z. | [Immersive video effects][IMF] (S, #885) | partial | Yaw only, by the two-element horizontal wrap (ADR-0012 keyed `x`). #885 recorded `missing` | X (tilt) and Z (roll) rotation, and one element doing it. |
| VR Sharpen | Sharpening for 360° footage. | [Immersive video effects][IMF] (S, #885) | missing | — | Sharpen, and the spherical model. |

### Immersive video transitions (from #885)

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Chroma Leaks | A chroma leak, seamless across the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | `TransitionKind` is `crossfade`, `wipe`, `slide`, `push`, `audio_crossfade` | Spherical model. |
| VR Gradient Wipe | A gradient-driven wipe across the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | as above; `wipe` is planar (ADR-0150) | Spherical model. |
| VR Iris Wipe | An iris wipe on the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | as above | Spherical model. |
| VR Light Leaks | Light leaks across the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | as above | Spherical model. |
| VR Light Rays | Light rays across the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | as above | Spherical model. |
| VR Mobius Zoom | A Möbius zoom on the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | as above | Spherical model. |
| VR Random Blocks | Random blocks on the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | as above | Spherical model. |
| VR Spherical Blur | A blur transition on the sphere. | [Immersive effects and transitions][IMT] (S, #885) | missing | as above | Spherical model. |

### Ambisonic and spatial audio

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Ambisonic clips | First-order ambisonic (4-channel) audio is interpreted as multichannel and placed on an Adaptive (multichannel) track before ambisonic effects apply. | [Assemble Ambisonics audio][AMB] (S) | missing | ADR-0077 §8: every input becomes 48 kHz stereo (`aformat=…channel_layouts=stereo`), so B-format is folded down as surround | Keeping a 4-channel source intact, and knowing it is B-format. |
| Ambisonics sequence (multichannel master) | VR … Ambisonics presets set a multichannel master and adaptive tracks. | [Monitor Ambisonics audio][MON] (S) | missing | One stereo bus (ADR-0077 §8); `master` holds only `target_lufs` and `ceiling_dbtp` (ADR-0172) | A multichannel bus and master. |
| Panner – Ambisonics | Rotates the sound field with Pan, Tilt and Roll so it matches the picture; keyframeable. | [Monitor Ambisonics audio][MON], [Restage aligned video and audio][RES] (S) | missing | ADR-0184 `pan` is a static stereo balance (proposed, not shipped); `audio_effects` has no rotation | Sound-field rotation. |
| Binauralizer – Ambisonics | Renders the ambisonic mix binaurally to headphones for monitoring. | [Monitor Ambisonics audio][MON] (S) | missing | — | A binaural decode. |
| Monitor ambisonics while looking around | With the Binauralizer on, the sound follows the VR viewer's direction. | [Monitor Ambisonics audio][MON] (S) | missing | No VR viewer and no live audio monitor | Head- or view-tracked monitoring. |
| Channel routing for ambisonic channel order | Map the four channels (W, Y, Z, X in AmbiX order) to the track's channels. | [Assemble Ambisonics audio][AMB] (K) | missing | ADR-0184 `channels` has five stereo words (`stereo`, `swap`, `mono`, `left`, `right`), proposed, not shipped | Multichannel routing. |

### Publishing VR video

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Video is VR (spatial metadata injection) | Export Settings → Video → *Video is VR*, with Frame Layout; Premiere writes the spherical metadata that YouTube, Facebook and players read. | [Export settings reference][EXP] (S) | missing | `render` writes H.264 + stereo AAC in MP4 and no spherical or stereo-3D metadata (export.md) | Writing spherical/stereo metadata (`sv3d`/`st3d` or the Spherical Video V1 XMP box). |
| Export frame layout | Frame Layout on export: monoscopic, stereoscopic over/under, side-by-side. | [Export settings reference][EXP] (S) | partial | The pixels can be laid out by hand (see "Output Layout") | The flag that tells a player which layout it is. |
| Export horizontal / vertical field of view | Captured view written on export (360° × 180°, 180° × 180°). | [Export settings reference][EXP] (S) | missing | — | Captured view in the metadata. |
| Audio is Ambisonics | H.264 export: *Audio is Ambisonics* writes 4-channel audio with spatial-audio metadata. | [Export settings reference][EXP] (S) | missing | Output is fixed stereo AAC (ADR-0077 §8; export.md) | 4-channel output and its spatial-audio metadata. |
| Stretch to Fill on export | Adobe recommends the Stretch to Fill scaling setting for VR export. | [Export settings reference][EXP] (S) | have | The project frame is the output size; `fit` fills it (ADR-0015) | — |
| Publish to 360 platforms | Publish or upload straight to YouTube / Facebook with VR flags. | [Choose a display mode][DISP] (K) | missing | No publish destinations (export.md, workflow.md) | Platform upload. |
| VR export presets | H.264 presets such as "VR Monoscopic Match Source Ambisonics" set the VR and ambisonic flags. | [Export settings reference][EXP] (K) | missing | One fixed encode (ADR-0077) | VR presets, which presuppose the flags above. |

## Notable gaps

1. **No spherical model.** A source has no projection, layout or captured view, and `probe` ignores
   the spherical side data a 360° camera writes. Almost every other row waits on this. No ADR
   refuses it, and ADR-0167's "projected, never placed" leaves room for it.
2. **No VR metadata on output.** Even a correct 2:1 equirectangular render plays as a flat
   rectangle on YouTube or in a headset, because `render` writes no spherical metadata.
3. **Ambisonics is destroyed on input.** ADR-0077 §8 converts every input to stereo, so 4-channel
   B-format is folded down as surround. There is no multichannel bus, no Panner/Binauralizer, and
   output is stereo AAC.
4. **No VR viewer.** `frame` and `preview` show the flat equirectangular frame. There is no viewport,
   no look-around, no field-of-view setting and no headset playback.
5. **Rotation is yaw-only, by hand.** Two elements with keyed `x` give an exact yaw. Tilt and roll,
   and therefore horizon correction, need a spherical remap.
6. **All 11 immersive effects and 8 immersive transitions** are missing for the same reason (#885);
   VR Rotate Sphere is partial here (yaw only).
7. **Stereo layouts are hand-built.** `clip` and offsets can take one eye or rearrange halves, but
   nothing declares a layout, and disparity is four elements per clip.

[OVR]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-editing-in-premiere-pro.html
[DET]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-auto-detection.html
[INT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/interpret-vr-footage.html
[SEQ]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/assign-vr-properties-to-sequences.html
[PAN]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/360-degree-panning.html
[ROT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/three-axis-video-rotation.html
[HIDE]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/hide-vr-video-view-controls.html
[IMT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/immerse-video-effects-and-transitions.html
[IMF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/immersive-video-effects.html
[AMB]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/assembling-ambisonics-audio.html
[MON]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/monitor-ambisonics-audio.html
[RES]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/restage-correctly-aligned-video-and-audio.html
[EXP]: https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html
[DISP]: https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/choose-a-display-mode.html
[HMD]: https://helpx.adobe.com/ie/premiere-pro/using/immersive-video.html
