# Premiere gap: VR, 360° and immersive video

Research for [#909](https://github.com/MBehtemam/Montagent/issues/909), re-sourced for
[#911](https://github.com/MBehtemam/Montagent/issues/911), on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882).

- **Date:** 2026-10-10 (first pass); re-sourced 2026-10-10 against the fetched pages.
- **Premiere side:** the 14 pages of the "Edit VR content" chapter of Adobe's Premiere user
  guide (`helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/`, Premiere 26.5), all
  fetched in full by the census ([#883](https://github.com/MBehtemam/Montagent/issues/883)) and
  read end to end. The census corpus (807 pages) was also searched for "immersive", "VR", "360"
  and "ambisonic". That search added these pages outside the chapter, cited where they say
  something the chapter does not: the Immersive Video effects page, the export-settings
  reference (its "VR Video" section), the monitor display-mode page, the GPU / Mercury Playback
  Engine page, the custom sequence preset page, the effects-and-transitions lists (list of
  effects and transitions, list of video transitions, list of accelerated effects, transitions
  overview, effects removed), and the third-party plug-ins page. The Generative Extend
  known-issues page also says VR Properties cannot be modified on an extended clip, which is a
  limitation rather than a feature and has no row. The three What's-new pages (What's new,
  release notes, beta features) were read and mention no VR, 360°, immersive or ambisonic item.
- **Source marks** (in the Adobe doc URL column):
  - **(F)** fetched page; the row's mechanism is stated on the cited page(s).
  - **unconfirmed**: no fetched page supports the row. It comes from the first pass's
    search excerpts or the agent's own knowledge and is kept, not dropped, so a later check can
    settle it. The URL is where it would be documented.
  - The first pass's (S), (S, #885) and (K) marks are all gone: every such row is now (F) or
    unconfirmed. Rows whose mechanism the full page contradicted say so in a **Corrected:**
    note in the mechanism cell.
- **Montagent side:** tag `v0.2.0` (`51aa4b5` = `origin/main`): `CONTEXT.md`,
  `schema/montagent.schema.json`, `docs/adr/` (in particular 0002, 0003, 0007, 0015, 0021,
  0023, 0040, 0077, 0132, 0145, 0167, 0168, 0172, 0184), `crates/montagent-core/src/media/probe.rs`,
  the MCP tools in `crates/montagent/src/mcp*`, and `skills/`.
- **Rows:** 67. **have** 1 · **have (skill)** 0 · **partial** 12 · **missing** 54 · **refused** 0.
  By source: 63 (F), 4 unconfirmed. (First pass: 57 rows, 2 have · 8 partial · 47 missing;
  32 (S), 19 (S, #885), 6 (K).) This pass added 10 rows, corrected 19, dropped none, and
  changed one status (Stretch to Fill on export, `have` → `partial`).

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
"projected, never placed" thesis would admit by mechanism. So nothing here is `refused`. GPU
acceleration, which Premiere requires for the VR effects, is "deferred, not decided" (ADR-0021).
Third-party VR plug-ins would come in through a plug-in architecture, which ADR-0003 rules out,
but the mechanisms they provide (360° stabilisation, tracking) are not refused, so those rows are
`missing`.

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
| VR editing (360° and 180°, mono and stereo) | Premiere views 360° and 180° video in the monitor panels, detects whether a clip or sequence has VR properties (VR Projection, VR Layout, VR Captured View) and configures the VR viewer to match. Layouts are monoscopic, stereoscopic over/under and side-by-side. | [VR editing in Premiere][OVR], [VR editing in Premiere (alt URL)][OVR2], [VR auto-detection][DET], [VR assignments][ASG] (F) | missing | No spherical model: `$defs/Frame` is `{width, height}`; ADR-0023 source dimensions are dims, PAR and rotation only | A projection, layout and captured-view model on sources and on the project. |
| VR properties assigned from file metadata | VR properties written into a clip's metadata when it was created are assigned to the clip on import. | [VR assignments][ASG], [VR auto-detection][DET] (F) | missing | `probe.rs` reads only `Display Matrix` side data; spherical and stereo-3D side data are ignored | Reading spherical metadata (e.g. MP4 `sv3d`/`st3d`, ffprobe `Spherical Mapping`, `Stereo 3D`). |
| Detect VR by frame aspect | With no VR metadata, the importer reads 1:1 as stereoscopic over/under, 2:1 as monoscopic and 4:1 as stereoscopic side-by-side, but only at frame heights 960, 1920, 2048, 2880, 4096, 5760, 6000 and 8192. **Corrected:** the page is VR assignments, not VR auto-detection, and it limits the rule to those heights. | [VR assignments][ASG] (F) | missing | — (no VR interpretation exists to default) | The aspect-and-height heuristic, and the property it would set. |
| VR properties in the Properties panel | Select the clip in the Project panel → File → Get Properties for → Selection. VR Projection shows whether the clip has VR properties and whether it is 360° or 180°; VR Layout and VR Captured View follow. **Corrected:** the page is VR auto-detection, and the route is Get Properties, not Interpret Footage. | [VR auto-detection][DET] (F) | missing | `query`/`probe` report dims, PAR, rotation, alpha; nothing spherical | Reporting the properties in `probe`/`query`. |
| Interpret VR footage | When a VR clip has no VR properties, or wrong ones: Project panel → right-click → Modify → Interpret Footage → VR Properties → *Use Properties from File* or *Conform to*. | [Interpret VR footage][INT] (F) | missing | ADR-0002 rules out an asset table, so a per-source override would sit on the element; no element carries a VR field | A per-element VR interpretation (projection, layout, H/V captured view). Not refused: ADR-0002 rules out where Premiere stores it, not the field. |
| VR properties on sequences | Sequence → Sequence Settings → VR Properties tab: Projection (e.g. Equirectangular, "a 360-degree image wrapped around a sphere"), Layout (e.g. Monoscopic for 2:1 clips), Horizontal and Vertical Captured View in degrees. **Corrected:** the page does not list "None" or every layout value. | [Assign VR properties to sequences][SEQ] (F) | partial | `$defs/Frame` takes any integer `width`/`height`, so a 2:1 or 1:1 frame can be written | The VR Properties block itself: projection, layout and captured view on the project, read by render and export. |
| VR sequence presets | File → New Sequence → Sequence Presets → Legacy → VR → Monoscopic 29.97 → 2048×2048 – Ambisonics, or Stereoscopic 29.97 → 2048×2048. **Corrected:** the only presets any page names are these Legacy 2048×2048 ones; the 4096×2048 / 4096×4096 presets of the first pass are not on any page. | [Monitor Ambisonics audio][MON] (F) | partial | `create_project` writes a starter at any frame size; `fps` is an integer (`schema` `properties.fps`) | 29.97 (integer fps only, see editing.md), the VR properties, and the Ambisonics (multichannel) variant. |
| Custom sequence preset with VR settings | File → New → Sequence; pick a preset, change the Settings, Tracks or **VR Video** tabs, name the preset and save it. | [Create a custom sequence preset][CSP] (F) | missing | `create_project` writes one starter; there is no preset store, and no VR fields to save | Saved project presets, and the VR settings in them. |
| New sequence from a VR clip | Project panel → right-click a VR clip → New Sequence From Clip; the sequence inherits its VR properties. | unconfirmed ([Assign VR properties to sequences][SEQ]) | missing | `create_project` takes no source to match | Matching a project to a source, and VR properties to inherit. No fetched page states VR inheritance. |
| Mixed resolutions and layouts in one sequence | Equirectangular clips of different resolutions and mono/stereo layouts share a sequence; the editor applies VR Projection to each and picks the layout so it fills the frame. **Corrected:** the conforming is done by the VR Projection effect the editor applies, not automatically by the sequence. | [Three-axis video rotation][ROT] (F) | partial | Mixed sizes are routine: each element declares its own rect (ADR-0007, ADR-0015) | Per-element layout conformance; a mono and a stereo source cannot be told apart or converted except by hand (see "Output Layout" below). |
| 180° VR (VR180) | 180° video is viewed in the monitors, and the Properties panel's VR Projection says whether a clip is 360° or 180°. **Corrected:** no page states a "180° × 180°" captured-view value. | [VR editing in Premiere][OVR], [VR auto-detection][DET] (F) | missing | — | A captured-view value, and half-sphere viewing and export. |

### Viewing VR in the monitors

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Video display in the Source and Program Monitors | Right-click the video frame → VR Video → Enable (and Show Controls), or the Monitor Settings menu; deselect Enable to return to the normal display. VR Video is a display mode beside Composite, RGBA, Multi-Camera and Comparison, and is disabled when the content has no VR properties. | [360-degree panning][PAN], [Choose a display mode][DISP], [VR auto-detection][DET] (F) | missing | There is no live monitor. `frame` (PNG) and `preview` (proxy MP4) show the flat equirectangular picture (ADR-0078) | A viewport into the sphere, live or as a `frame` option. |
| Toggle VR Video Display button | A button that switches the VR display on and off; added from the monitor's button editor (the + icon) and docked with the playback buttons. | [360-degree panning][PAN] (F) | missing | No monitor, so no monitor buttons | A VR display to toggle. |
| Look around interactively | Turn the dial or double-click within the video frame to change the view, during playback or while paused. **Corrected:** the page names the dial and double-click, not dragging. | [360-degree panning][PAN] (F) | missing | No interactive monitor | Interactive view direction. |
| VR view controls (pan/tilt scroll bars, hot text, continual-pan dial) | The VR view shows scroll bars and hot-text controls for pan and tilt, and a dial at the bottom for continuous 360° pan; the current pan and tilt read out as numbers below and to the right of the viewer. Changes the view only. **Corrected:** tilt is a scroll bar with hot text, not a slider, and no page says double-click re-centres. | [Hide VR video view controls][HIDE], [360-degree panning][PAN], [Monitor Ambisonics audio][MON] (F) | missing | — | View-only pan/tilt controls. |
| VR Video settings (monitor field of view, stereoscopic view) | Monitor → VR Video → Settings: monitor view horizontal/vertical FOV, and which eye (left, right, anaglyph) is shown for stereo. | unconfirmed ([VR editing in Premiere][OVR]) | missing | — | Viewer FOV and eye selection. No fetched page describes this dialog. |
| Hide VR video view controls | Right-click either monitor → VR Video → uncheck Show Controls, to maximise the picture; can be mapped to a keyboard shortcut; the state is kept per monitor and saved with the project. | [Hide VR video view controls][HIDE] (F) | missing | — | Nothing to hide: there are no viewer controls. |
| Head-mounted display monitoring | Edit with a supported VR headset after installing its software; the VR display simulates viewing in a headset such as the Oculus Rift. **Corrected:** no page names "Adobe Immersive Environment" or Windows Mixed Reality; the first pass cited a non-desktop URL. | [VR editing in Premiere][OVR], [Choose a display mode][DISP] (F) | missing | — | Headset playback. |
| Simulate viewing platforms | The VR Video display simulates headset viewing, or desktop viewing through YouTube or Facebook. | [Choose a display mode][DISP] (F) | missing | — | Any VR display. |
| GPU acceleration and VRAM for VR | VR needs a GPU with 6 GB VRAM, more for 8K × 8K stereoscopic frames; all 11 immersive effects and 8 immersive transitions are GPU-accelerated, and applying a VR effect without enough VRAM shows "This effect requires GPU Acceleration to work". | [GPU acceleration][GPU], [List of accelerated effects][ACC] (F) | missing | CPU-only render; GPU rasterisation is "deferred, not decided" (ADR-0021, ADR-0010) | A GPU path. Montagent has no VRAM ceiling either, since it has no VR effects to run. |
| Real-time playback of VR rotation | Three-axis rotation plays back in real time when GPU acceleration is on. | [Three-axis video rotation][ROT] (F) | missing | No live playback and no GPU (ADR-0021) | Real-time playback of any kind. |

### Reorienting 360° footage

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Three-axis video rotation (correct the zero angle) | VR Projection effect → Pan (left-to-right), Tilt (bottom-to-top), Roll (counter-clockwise to clockwise); rotate the sphere in the VR viewer until the content sits right, e.g. to fix bad camera placement. **Corrected:** no page says the rotation is keyframeable. | [Three-axis video rotation][ROT], [Restage aligned video and audio][RES] (F) | partial | Yaw is a horizontal wrap of the equirectangular frame: two `video` elements on the same `source`, side by side, with keyed `x` (ADR-0012 keyframes). Exact at whole-pixel `x` | Tilt and roll (a spherical remap), a single control for the rotation, and doing it on one element. Stereo needs one wrap pair per eye with `clip`. |
| Restage correctly aligned video and audio | Put VR Projection on the video and Panner – Ambisonics on the matching audio clip; rotate the video sphere with Pan/Tilt/Roll in the VR viewer, then type the same numbers into the Panner's Pan, Tilt and Roll sliders. | [Restage aligned video and audio][RES] (F) | missing | No ambisonic audio (ADR-0077 §8 stereo bus) | Sound-field rotation, and anything to align it to. |

### The VR Projection effect, by parameter

The effects research has one row for VR Projection (`missing`). The three-axis rotation page
documents its parameters separately, and some of them can be built from flat mechanisms.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Input Auto VR Properties | When on, the effect uses the VR properties of the source clip or nested sequence it sits on. Turn it off to set Input Layout, Input Horizontal FOV and Input Vertical FOV by hand. **Corrected:** the page is Three-axis video rotation, and it includes nested sequences. | [Three-axis video rotation][ROT] (F) | missing | No VR properties to read (see "VR properties assigned from file metadata") | Input layout and FOV, read or written. |
| Output Auto VR Properties | When on, the effect uses the current sequence's VR settings for its output. Turn it off to set Output Layout, Output Horizontal FOV and Output Vertical FOV by hand. | [Three-axis video rotation][ROT] (F) | missing | No project VR settings | Output VR properties, inherited or set. |
| Output Layout | Sets the output stereoscopic or monoscopic layout, usually matching the sequence's layout, so a clip of one layout can be converted to another. | [Three-axis video rotation][ROT] (F) | partial | Halves are rectangles: two `video` elements, each `clip`ped to one eye and moved with `x`/`y`, turn over/under into side-by-side, or take one eye for a mono deliverable (`CONTEXT.md` "Clip") | A layout field; the conversion is hand-built from two or more elements per clip and is not declared anywhere. |
| Output Horizontal / Vertical FOV | Sets the output's horizontal and vertical field of view by hand. | [Three-axis video rotation][ROT] (F) | missing | — | A spherical re-projection. A 360→180 crop of a mono equirect is a `clip` of the middle half, but the vertical axis and stereo cases are not. |
| Disparity Adjustment | When input and output are both stereoscopic, shifts each eye's image by a total number of degrees: positive deepens the stereo effect, negative flattens it. Usually under 1°, range −180° to +180°; ignored if either layout is monoscopic. | [Three-axis video rotation][ROT] (F) | partial | Convergence in equirectangular stereo is a per-eye yaw: one wrap pair per eye (see "Three-axis video rotation"), offset in opposite directions | A field for it; four elements per stereo clip by hand. |
| Stretch To Fill Frame | Scales the clip to fill the frame regardless of aspect ratio or frame size. | [Three-axis video rotation][ROT] (F) | have | The element's declared `width`/`height` is authoritative at render (ADR-0007, ADR-0015); with `fit: literal` and the frame's size it stretches to fill regardless of aspect. **Corrected evidence:** `fit: cover` keeps aspect, so it is not this | — |
| Filtering | A slider for how many extra samples are taken horizontally and vertically, to cut aliasing; 1 (the default) reads 4 samples, like bilinear. Raising it helps most when scaling down. | [Three-axis video rotation][ROT] (F) | partial | Flat images are sampled by what the draw does: bilinear over linear mipmaps when shrunk, bicubic when enlarged, chosen automatically (ADR-0132) | The spherical re-projection it filters, and a user setting for sample count. |
| Auto VR properties on every immersive effect | All immersive video effects and transitions automatically detect a clip's VR properties; the effect descriptions say they work across poles, edges and the whole spherical view. **Corrected:** the page is Immersive video effects and transitions. | [Immersive video effects and transitions][IMT], [Immersive Video effects][IMF] (F) | missing | — | Per-eye, seam-aware effects. |

### Immersive video effects

Video Effects → Immersive Video lists 11 effects
([Immersive video effects and transitions][IMT], [List of effects and transitions][LET]); none
came from Film Impact, and all are GPU-accelerated ([List of accelerated effects][ACC]). See
`effects.md` (#885) for the effects-area view. One status differs from #885: VR Rotate Sphere is
`partial` here because of the yaw wrap.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Blur | Blurs 360° and 180° footage evenly across the whole view, poles and edges included. | [Immersive Video effects][IMF] (F) | missing | `blur` is planar (ADR-0040) | Spherical wrap. |
| VR Chromatic Aberrations | Adds a subtle colour shift at the edges of 360° video, like headset lens flaws. | [Immersive Video effects][IMF] (F) | missing | — | Spherical model, and channel offsets. |
| VR Color Gradients | Gradient fills that blend seamlessly across the spherical view. | [Immersive Video effects][IMF] (F) | missing | Planar gradients only (ADR-0149) | Spherical model. |
| VR De-Noise | Reduces grain and noise in 360° footage. It is also one of the named replacements for the removed Median (Legacy) effect. | [Immersive Video effects][IMF], [Effects removed from Premiere][REM] (F) | missing | — | Denoise, and the spherical model. |
| VR Digital Glitch | Digital-glitch distortion across 360° video. | [Immersive Video effects][IMF] (F) | missing | — | Spherical model. |
| VR Fractal Noise | Generates evolving fractal-noise textures (smoke, clouds, abstract patterns). **Corrected:** the page does not say it maps over the sphere. | [Immersive Video effects][IMF] (F) | missing | — | Fractal noise, and the spherical model. |
| VR Glow | Glow round bright areas of 360° footage. | [Immersive Video effects][IMF] (F) | missing | `glow` is planar (ADR-0156) | Spherical wrap. |
| VR Plane to Sphere | Wraps a flat image, text or logo onto the 360° sphere so it looks undistorted wherever the viewer looks. | [Immersive Video effects][IMF] (F) | missing | ADR-0167 projects a flat element onto a plane, not a sphere | Equirectangular projection of a flat element. ADR-0167's thesis ("projected, never placed") would admit it by mechanism. |
| VR Projection | Described as projecting 2D footage onto a 360° spherical surface; its rotation, layout, FOV, disparity, fill and filtering parameters are rowed above. | [Immersive Video effects][IMF], [Three-axis video rotation][ROT] (F) | missing | — | Spherical model. |
| VR Rotate Sphere | Rotates 360° footage about the X, Y or Z axis, e.g. to level the horizon. | [Immersive Video effects][IMF] (F) | partial | Yaw only, by the two-element horizontal wrap (ADR-0012 keyed `x`). #885 recorded `missing` | X and Z rotation (tilt, roll), and one element doing it. |
| VR Sharpen | Sharpens 360° video evenly across the spherical scene. | [Immersive Video effects][IMF] (F) | missing | — | Sharpen, and the spherical model. |

### Immersive video transitions

Video Transitions → Immersive Video lists 8 transitions ("VR transitions" in the overview). The
pages give names only, no descriptions; the mechanism column is the name's plain reading.

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| VR Chroma Leaks | A chroma-leak transition for VR footage. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | `TransitionKind` is `crossfade`, `wipe`, `slide`, `push`, `audio_crossfade` | Spherical model. |
| VR Gradient Wipe | A gradient wipe for VR footage. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | as above; `wipe` is planar (ADR-0150) | Spherical model. |
| VR Iris Wipe | An iris wipe for VR footage. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | as above | Spherical model. |
| VR Light Leaks | A light-leak transition for VR footage. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | as above | Spherical model. |
| VR Light Rays | A light-rays transition for VR footage. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | as above | Spherical model. |
| VR Mobius Zoom | A Möbius-zoom transition for VR footage. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | as above | Spherical model. |
| VR Random Blocks | A random-blocks transition for VR footage. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | as above | Spherical model. |
| VR Spherical Blur | A blur transition on the sphere. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) | missing | as above | Spherical model. |

### Ambisonic and spatial audio

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Ambisonic (B-format) clips | Ambisonics is full-sphere surround whose channels carry a speaker-independent sound field (B-format) decoded to the listener's speakers. Multichannel mono media is imported as adaptive-track audio. **Corrected:** no page states "first-order" or "4-channel". | [Assembling Ambisonics audio][AMB] (F) | missing | ADR-0077 §8: every input becomes 48 kHz stereo (`aformat=…channel_layouts=stereo`), so B-format is folded down as surround | Keeping a multichannel source intact, and knowing it is B-format. |
| Ambisonics in a separate WAV or inside MP4 | Ambisonic audio may sit in separate files (often uncompressed PCM WAV) or in the MP4 as AAC, assembled with the 360° video. | [Assembling Ambisonics audio][AMB] (F) | partial | A separate WAV is an ordinary `audio` element synced to the video; WAV/PCM and AAC-in-MP4 both decode | Keeping the channels: both arrive folded to stereo (ADR-0077 §8). |
| Ambisonics sequence (multichannel master) | The Legacy VR … Ambisonics preset makes a sequence that holds multichannel Ambisonics audio, with the Binauralizer on its mix track. The Assembling page also says VR presets are found under the Audio Track Mixer or the Audio Effects folder. | [Monitor Ambisonics audio][MON], [Assembling Ambisonics audio][AMB] (F) | missing | One stereo bus (ADR-0077 §8); `master` holds only `target_lufs` and `ceiling_dbtp` (ADR-0172) | A multichannel bus and master. |
| Panner – Ambisonics | An audio effect on the clip that reorients the sound sphere with Pan, Tilt and Roll sliders. **Corrected:** the page is Restage, not Monitor; no page says it is keyframeable. | [Restage aligned video and audio][RES] (F) | missing | ADR-0184 `pan` is a static stereo balance (proposed, not shipped); `audio_effects` has no rotation | Sound-field rotation. |
| Binauralizer – Ambisonics | Put on the mix track in the Audio Track Mixer to hear the ambisonic mix spatially on headphones. Pan, Tilt and Roll (each −180° to +180°) turn the listening direction; the default faces straight ahead. Remove or disable it when monitoring is done. | [Monitor Ambisonics audio][MON] (F) | missing | — | A binaural decode. |
| Match binaural monitoring to the VR viewer | With the viewer's controls shown, turn the Binauralizer's Pan to the number shown below the VR viewer and its Tilt to the number to the right, so sound and picture face the same way. **Corrected:** this is matched by hand; the sound does not follow the viewer automatically. | [Monitor Ambisonics audio][MON] (F) | missing | No VR viewer and no live audio monitor | A viewer, and a binaural decode to align with it. |
| Channel routing for ambisonic channel order | Map the four channels (W, Y, Z, X in AmbiX order) to the track's channels. | unconfirmed ([Assembling Ambisonics audio][AMB]) | missing | ADR-0184 `channels` has five stereo words (`stereo`, `swap`, `mono`, `left`, `right`), proposed, not shipped | Multichannel routing. No fetched page describes channel order. |

### Publishing VR video

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Video is VR (spherical metadata on export) | Export → Settings → Video section → *Video Is VR*, so the output is "tagged properly for media players". | [Publish VR videos][PUB], [Export settings reference][EXP] (F) | missing | `render` writes H.264 + stereo AAC in MP4 and no spherical or stereo-3D metadata (export.md) | Writing spherical/stereo metadata (`sv3d`/`st3d` or the Spherical Video V1 XMP box). |
| VR export in H.264, HEVC or QuickTime | The VR export's encoding is H.264, HEVC or QuickTime. | [Publish VR videos][PUB] (F) | partial | `render` writes H.264 in MP4 (ADR-0077, ADR-0143) | HEVC and QuickTime (see export.md), and the VR flag on any of them. |
| Export frame layout | With Video Is VR checked, Frame Layout is set to match the media, and it converts between monoscopic, stereoscopic over/under and side-by-side. | [Export settings reference][EXP], [Publish VR videos][PUB] (F) | partial | The pixels can be laid out by hand (see "Output Layout") | The flag that tells a player which layout it is, and a conversion that is not hand-built. |
| Export horizontal / vertical field of view | With Video Is VR checked, the Horizontal and Vertical Fields of View are set on export. | [Export settings reference][EXP] (F) | missing | — | Captured view in the metadata. |
| Audio is Ambisonics | Export Settings → H.264 → *Audio is Ambisonics*, then any Ambisonics setting, marks the output audio as Ambisonics. **Corrected:** no page states a 4-channel layout or names spatial-audio metadata. | [Assembling Ambisonics audio][AMB] (F) | missing | Output is fixed stereo AAC (ADR-0077 §8; export.md) | Multichannel output and its Ambisonics flag. |
| Stretch to Fill on export | Adobe recommends Scaling → Stretch to Fill for VR exports. | [Publish VR videos][PUB] (F) | partial | The project frame is the output size, and an element's declared rect fills it (`fit: literal`, ADR-0015). **Corrected status (was `have`):** there is no export-time scaling, so a frame that differs from the deliverable cannot be stretched at export | Export-time scaling to a different output size. |
| Publish to YouTube or Facebook | On the Export tab pick the platform (YouTube, Facebook), then the Publish dialog uploads the video; it may take a while after upload before it plays in VR mode. | [Publish VR videos][PUB], [VR editing in Premiere][OVR] (F) | missing | No publish destinations (export.md, workflow.md) | Platform upload. |
| VR export presets | H.264 presets such as "VR Monoscopic Match Source Ambisonics" set the VR and ambisonic flags. | unconfirmed ([Export settings reference][EXP]) | missing | One fixed encode (ADR-0077, ADR-0143) | VR presets, which presuppose the flags above. No fetched page names them. |

### Third-party VR tools named in the guide

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| 360° stabilisation (CyberLink VR Stabilizer) | A third-party plug-in that stabilises footage shot with a 360° camera. | [Plug-ins][PLG] (F) | missing | No stabilisation of any kind (ai.md, motion.md). A plug-in route is ruled out (ADR-0003: "an open plugin architecture never is"), but the mechanism is not | 360° stabilisation. |
| 360/VR tracking and masking (Boris FX Mocha Pro) | A third-party plug-in whose planar tracking and masking include mono and stereoscopic 360/VR tools. | [Plug-ins][PLG] (F) | missing | No tracking (motion.md); plug-ins ruled out by ADR-0003, the mechanism is not | 360-aware tracking and masking. |

## Notable gaps

1. **No spherical model.** A source has no projection, layout or captured view, and `probe` ignores
   the spherical side data a 360° camera writes. Almost every other row waits on this. No ADR
   refuses it, and ADR-0167's "projected, never placed" leaves room for it.
2. **No VR metadata on output.** Even a correct 2:1 equirectangular render plays as a flat
   rectangle on YouTube or in a headset, because `render` writes no spherical metadata. Premiere
   writes it from H.264, HEVC or QuickTime; Montagent writes H.264 only.
3. **Ambisonics is destroyed on input.** ADR-0077 §8 converts every input to stereo, so B-format
   (in a WAV or inside the MP4) is folded down as surround. There is no multichannel bus, no
   Panner/Binauralizer, and output is stereo AAC.
4. **No VR viewer.** `frame` and `preview` show the flat equirectangular frame. There is no viewport,
   no look-around dial, no pan/tilt readout, no toggle button and no headset playback.
5. **Rotation is yaw-only, by hand.** Two elements with keyed `x` give an exact yaw. Tilt and roll,
   and therefore horizon correction, need a spherical remap.
6. **All 11 immersive effects and 8 immersive transitions** are missing for the same reason, and
   all are GPU-only in Premiere while Montagent's GPU path is deferred (ADR-0021). VR Rotate Sphere
   is partial here (yaw only).
7. **VR Projection's own controls are missing:** Input/Output Auto VR Properties and FOV. Its
   Filtering is partly matched by ADR-0132's automatic sampling, and Stretch To Fill by
   `fit: literal`.
8. **Stereo layouts are hand-built.** `clip` and offsets can take one eye or rearrange halves, but
   nothing declares a layout, and disparity is four elements per clip.

## Pages covered

| Page URL | Rows / "no feature: <reason>" |
|---|---|
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/360-degree-panning.html | VR Video display in the Source and Program Monitors; Toggle VR Video Display button; Look around interactively; VR view controls |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/assembling-ambisonics-audio.html | Ambisonic (B-format) clips; Ambisonics in a separate WAV or inside MP4; Ambisonics sequence; Audio is Ambisonics; Channel routing for ambisonic channel order (unconfirmed) |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/assign-vr-properties-to-sequences.html | VR properties on sequences; New sequence from a VR clip (unconfirmed) |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/hide-vr-video-view-controls.html | Hide VR video view controls; VR view controls |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/immerse-video-effects-and-transitions.html | Auto VR properties on every immersive effect; the 11 immersive video effects; the 8 immersive video transitions |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/interpret-vr-footage.html | Interpret VR footage |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/monitor-ambisonics-audio.html | VR sequence presets; Ambisonics sequence; Binauralizer – Ambisonics; Match binaural monitoring to the VR viewer; VR view controls (numeric readouts) |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/publish-vr-videos.html | Video is VR; VR export in H.264, HEVC or QuickTime; Export frame layout; Stretch to Fill on export; Publish to YouTube or Facebook |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/restage-correctly-aligned-video-and-audio.html | Restage correctly aligned video and audio; Three-axis video rotation; Panner – Ambisonics |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/three-axis-video-rotation.html | Mixed resolutions and layouts in one sequence; Three-axis video rotation; Real-time playback of VR rotation; Input Auto VR Properties; Output Auto VR Properties; Output Layout; Output Horizontal / Vertical FOV; Disparity Adjustment; Stretch To Fill Frame; Filtering; VR Projection |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-assignments.html | VR properties assigned from file metadata; Detect VR by frame aspect; VR editing (layouts) |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-auto-detection.html | VR editing; VR properties assigned from file metadata; VR properties in the Properties panel; 180° VR; VR Video display (disabled without VR properties) |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-editing-in-premiere-pro.html | VR editing; 180° VR; Head-mounted display monitoring; Publish to YouTube or Facebook; VR Video settings (unconfirmed) |
| https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-editing-in-premiere.html | Same text as vr-editing-in-premiere-pro.html (duplicate URL): VR editing; 180° VR; Head-mounted display monitoring; Publish to YouTube or Facebook |

[OVR]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-editing-in-premiere-pro.html
[OVR2]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-editing-in-premiere.html
[DET]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-auto-detection.html
[ASG]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-assignments.html
[INT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/interpret-vr-footage.html
[SEQ]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/assign-vr-properties-to-sequences.html
[PAN]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/360-degree-panning.html
[ROT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/three-axis-video-rotation.html
[HIDE]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/hide-vr-video-view-controls.html
[IMT]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/immerse-video-effects-and-transitions.html
[AMB]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/assembling-ambisonics-audio.html
[MON]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/monitor-ambisonics-audio.html
[RES]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/restage-correctly-aligned-video-and-audio.html
[PUB]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/publish-vr-videos.html
[IMF]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/immersive-video-effects.html
[LET]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html
[LVT]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-video-transitions.html
[ACC]: https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/list-of-accelerated-effects.html
[REM]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/effects-and-transitions-removed-from-adobe-premiere.html
[EXP]: https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html
[DISP]: https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/choose-a-display-mode.html
[GPU]: https://helpx.adobe.com/premiere/desktop/get-started/download-and-install/mercury-playback-engine-gpu-accelerated-in-premiere.html
[CSP]: https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/create-a-custom-sequence-preset.html
[PLG]: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html
