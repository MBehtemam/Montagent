# Premiere Pro vs Montagent v0.2.0: the feature inventory

This is the destination of [Map: every Premiere Pro feature, set against Montagent v0.2.0](https://github.com/MBehtemam/Montagent/issues/882). It sets **every feature in the current Adobe Premiere user guide** (Premiere 26.5, `helpx.adobe.com/premiere/desktop/`, 807 pages, fetched 2026-10-10) against **Montagent v0.2.0** (tag `v0.2.0` = `51aa4b5`).

It decides nothing about what to build. It records, for each feature, what Montagent has today.

## How to read it

- **Status.** `have`: an equivalent mechanism ships in v0.2.0. `have (skill)`: a bundled skill under `skills/` provides it, not the format. `partial`: some of the mechanism ships, and the row says what's missing. `missing`: nothing equivalent ships and no ADR refuses it. `refused`: an ADR, or the four invariants of ADR-0145, rules it out, and the row cites which.
- **Read by mechanism, not by name** (ADR-0145). A GUI-only Premiere feature is `missing`, not `refused`, unless an ADR refuses it, because no ADR refuses a GUI.
- **The area files hold the detail.** For each row they give Premiere's mechanism, the exact Adobe page, and the Montagent evidence (ADR, schema key, MCP tool or skill). This file gives the totals, every `partial` and `missing` row (the gap list), every `refused` row, and the reconciliations. `have` rows are counted here and listed in the area files.
- **Sources.** Every Premiere row cites a page read in full (F), apart from a handful each area file labels *unconfirmed*: kept, not dropped, because no guide page states them. The first pass ran while helpx.adobe.com was blocked. Every row was then re-sourced against the full pages in [Do the area inventories hold up against the full Adobe pages, now that helpx is reachable?](https://github.com/MBehtemam/Montagent/issues/911).
- **Overlap.** Some features belong to two areas (for example Auto Reframe in editing, effects, motion and AI). Each area keeps its own row, so a feature can be counted more than once across areas. Where statuses disagreed, the reconciliation below settles them.

## Totals

| Area | Rows | have | have (skill) | partial | missing | refused | File |
|---|---|---|---|---|---|---|---|
| [Editing and timeline](https://github.com/MBehtemam/Montagent/issues/884) | 145 | 54 | 0 | 38 | 44 | 9 | [`editing.md`](editing.md) |
| [Video effects and transitions](https://github.com/MBehtemam/Montagent/issues/885) | 367 | 104 | 16 | 73 | 153 | 21 | [`effects.md`](effects.md) |
| [Motion, keyframing and compositing](https://github.com/MBehtemam/Montagent/issues/890) | 183 | 60 | 3 | 23 | 62 | 35 | [`motion.md`](motion.md) |
| [Colour](https://github.com/MBehtemam/Montagent/issues/886) | 193 | 13 | 0 | 42 | 113 | 25 | [`colour.md`](colour.md) |
| [Titles, graphics and text](https://github.com/MBehtemam/Montagent/issues/887) | 111 | 37 | 0 | 38 | 30 | 6 | [`titles.md`](titles.md) |
| [Captions, subtitles and transcripts](https://github.com/MBehtemam/Montagent/issues/888) | 79 | 12 | 3 | 15 | 32 | 17 | [`captions.md`](captions.md) |
| [Audio](https://github.com/MBehtemam/Montagent/issues/889) | 184 | 28 | 2 | 39 | 87 | 28 | [`audio.md`](audio.md) |
| [Import, formats and media handling](https://github.com/MBehtemam/Montagent/issues/891) | 235 | 50 | 3 | 40 | 121 | 21 | [`import.md`](import.md) |
| [Export, delivery and interchange](https://github.com/MBehtemam/Montagent/issues/892) | 135 | 20 | 1 | 13 | 90 | 11 | [`export.md`](export.md) |
| [AI and automation](https://github.com/MBehtemam/Montagent/issues/893) | 57 | 2 | 2 | 20 | 29 | 4 | [`ai.md`](ai.md) |
| [VR / 360 and immersive](https://github.com/MBehtemam/Montagent/issues/909) | 67 | 1 | 0 | 12 | 54 | 0 | [`vr.md`](vr.md) |
| [Workflow: interface, monitors, preferences, render](https://github.com/MBehtemam/Montagent/issues/894) | 244 | 63 | 3 | 42 | 121 | 15 | [`workflow.md`](workflow.md) |
| [Workflow: collaboration and other apps](https://github.com/MBehtemam/Montagent/issues/911) | 87 | 14 | 0 | 25 | 36 | 12 | [`workflow-collab.md`](workflow-collab.md) |
| **All areas** | **2087** | **458** | **33** | **420** | **972** | **204** | |

Coverage: all 757 census pages that describe a feature appear in some area file's *Pages covered* table. The other 50 are troubleshooting pages, listed under *Pages that describe no feature* below. `python3 census_coverage_check.py` re-derives this and exits non-zero if any page is unaccounted for.

## Headline gaps

The gap list below has 1,392 rows. These are the themes it falls into, roughly ordered by how many rows each one touches. Each bullet is a pointer into the area files, not a recommendation.

- **Delivery is one file shape.** `render` writes H.264 + AAC stereo in MP4 only. Missing:
  - ProRes, DNxHR, HEVC, AV1, GIF and image sequences;
  - alpha export, audio-only export and 5.1;
  - HDR10 and colour-space tags;
  - sidecar or embedded captions;
  - SRT streaming and publish destinations.

  ([export](export.md))
- **No interchange at all.** FCP XML, EDL, AAF, OMF, OTIO and ALE are missing in both directions. So are project trimming and archiving. ([export](export.md), [import](import.md))
- **No colour pipeline.** Missing:
  - colour management, log/HDR/raw handling and tone mapping;
  - a pipeline above 8 bits, and colour metadata on output;
  - scopes;
  - Lumetri's tonal tools: wheels, highlights/shadows, HSL secondary, and Color mode's zones and shifts.

  Curves and LUTs are refused (ADR-0040, ADR-0049). ([colour](colour.md))
- **Nothing acts above one element.** Missing:
  - adjustment layers, and sequence- or group-level grades and effects;
  - masks that scope a single effect;
  - mask union (masks only intersect).

  ([effects](effects.md), [motion](motion.md), [colour](colour.md))
- **Nothing analyses the picture or sound.** Missing:
  - motion and mask tracking, Warp Stabilizer and Auto Reframe;
  - Scene Edit Detection and Morph Cut;
  - audio-based sync;
  - frame blending and optical flow;
  - waveform reading.

  None is refused. Each could be an outside tool that writes literal keyframes. ([motion](motion.md), [ai](ai.md), [editing](editing.md))
- **Built-in AI is refused; the outside-tool route is open.** Speech-to-text, and everything downstream of it, needs an external words file: Text-Based Editing, filler removal, caption translation, Paper Edit. Enhance Speech is refused as a built-in. Generative Extend, the Generative Media Tool, Generate Music and Separate Crosstalk have no route, and there is no record of how a generated source was made. ([ai](ai.md), [captions](captions.md))
- **Timeline model gaps.** Missing:
  - `shift` can't move things earlier, which blocks ripple delete and close-gap;
  - markers of any kind;
  - multicam;
  - non-integer `fps` (no 29.97);
  - track lock, mute and enable;
  - linked audio and video;
  - deinterlacing.

  Nested sequences with their own clock are refused (ADR-0001). ([editing](editing.md))
- **Audio is a fixed stereo bus.** Everything is mixed at 48 kHz stereo. `pan`, `channels`, `echo`, `reduce_noise` and the noise gate are proposed but not shipped in v0.2.0. Effect parameters can't be animated. Also missing: reverb, pitch shift, peak normalising, readable waveforms and multichannel output. The track mixer, buses and sends are refused. ([audio](audio.md))
- **Import breadth.** Missing:
  - camera raw formats;
  - stills beyond PNG, JPEG and GIF;
  - per-source metadata and timecode;
  - proxy files, ingest with copy-and-verify, offline clips, and automatic relink.

  Bins and the media catalogue are refused (ADR-0002). ([import](import.md))
- **Text and captions.** Missing:
  - text and track styles;
  - Motion Graphics templates (.mogrt);
  - Responsive Design's pin to another layer;
  - more than one stroke, and spell check;
  - caption formats (SRT, SCC, 608/708, Teletext) and caption import.

  Captions exist only as burned-in text. ([titles](titles.md), [captions](captions.md))
- **VR has no model.** Missing: spherical projection, VR and ambisonic metadata, the VR viewer, and ambisonic audio (folded to stereo). ([vr](vr.md))
- **Blend modes.** 5 of Premiere's 27 ship. The other 22 are refused "for now" by ADR-0147, until each has a forcing case. ([motion](motion.md))
- **Workflow surfaces.** Missing:
  - GPU rendering, a preview-file cache and smart rendering;
  - a live, scrubbable monitor and playback devices;
  - default-value preferences.

  Collaboration is git only: no locking, presence, review comments or cross-project references. The remote instance (ADR-0159) is accepted but isn't in v0.2.0, which ships stdio only. Third-party plug-ins are refused. ([workflow](workflow.md), [workflow-collab](workflow-collab.md))

## Reconciliations

Where two area files rated the same feature differently, the status below is the one this inventory uses. The area files keep their original wording, and the gap list marks each reconciled row with its key.

- **R1** (5 row(s) changed). **Dynamic Link (After Effects compositions).** `motion.md` had *partial* and *missing*, `import.md` had *missing*, and `workflow-collab.md` had *refused*. The full pages show After Effects draws a linked composition's frames live, every time Premiere asks for one. A `source` that names a program to run fails ADR-0145's closed-vocabulary test (*a path may name media, never a program*), so the live link is **refused**. Pre-rendering a composition and placing the file still works (`montagent-prerender`, ADR-0156). Rows reconciled: import, create and replace a composition, in `motion.md`, `import.md` and `colour.md`.
- **R2** (1 row(s) changed). **VR Rotate Sphere.** `effects.md` had *missing* and `vr.md` *partial*. The VR ticket's finer analysis stands: yaw can be built exactly with two wrapped `video` elements, while tilt and roll need a spherical remap. So it is **partial**.
- **R3** (1 row(s) changed). **Adjustment layers.** `editing.md` had *partial*, because a blended `rect` can tint what lies below it. `effects.md` and `colour.md` had *missing*. Premiere's mechanism applies any effect to everything below, and no Montagent effect can filter other elements' pixels. So it is **missing**, and the blended `rect` workaround is noted. No ADR refuses it.
- **R4** (2 row(s) changed). **Auto Reframe.** `ai.md` had *partial*, because `x` and `scale` keyframes can hold a reframe. `editing.md`, `effects.md` and `motion.md` had *missing*. The feature is the subject-tracking analysis, which Montagent doesn't do, so it is **missing**. The keyframes can still hold a reframe that an outside tool computes.
- **R5** (1 row(s) changed). **Enhance Speech.** `ai.md` had *partial*, because EQ and dynamics ship, and `audio.md` had *refused*. Enhance Speech is a trained model, and a model inside the renderer breaks Montagent's defining rule that it *contains no model* (`CONTEXT.md`; ADR-0051 and ADR-0054 apply the same rule to speech-to-text). So it is **refused** as a built-in. Running a speech enhancer outside Montagent and placing its output stays open.

## The gap list: every `partial` and `missing` row

### Editing and timeline: 38 partial, 44 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Create sequences; many per project | partial | More than one timeline per project. Separate project files stand in for separate sequences, but nothing can refer from one to another (see the nesting rows). | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/create-a-sequence.html (F) |
| Open several sequences: tabs and separate Timeline panels | partial | Several timelines open side by side in one view. (GUI; #894.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/navigate-sequences-in-the-timeline.html (F) |
| Sequence presets and custom presets | partial | A library of named presets, and a tracks layout in the scaffold. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/create-a-custom-sequence-preset.html (F) |
| Sequence timebase / frame rate | partial | Fractional and NTSC rates: 24000/1001, 30000/1001 and 60000/1001 cannot be written. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Sequence pixel aspect ratio | missing | A non-square output PAR. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Field order / interlaced sequences | missing | Interlaced output and field order. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Sequence audio settings | partial | A choice of sample rate and channel format (mono, 5.1, multichannel). Audio samples as a time display unit. (#889 owns the rest.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Editing Mode | missing | A preview format separate from the deliverable. (#894.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Video preview settings | missing | A per-project preview codec and preview size. (#894 owns performance.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Maximum Bit Depth | missing | High-bit-depth processing. (#886.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Maximum Render Quality | partial | A choice between a draft and a high-quality scaler. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Working colour space and Auto Tone Map Media | missing | A working colour space and tone mapping. (#886.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Composite in linear colour | missing | A linear-light compositing option. (#886.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Scale motion effects proportionally when changing frame size | partial | The arithmetic: nothing rescales the elements when `frame` changes, so the agent does it by hand. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| Match sequence to a clip | partial | A one-step "settings from this clip", and a mismatch warning. `probe` is CLI-only (skill `montagent`, #455). | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/create-a-sequence.html (F) |
| Paper Edit: sequence from transcript lines (26.5) | missing | Building a cut from transcript text. (#888 and #893.) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/create-a-sequence-with-paper-edit.html (F) |
| Auto Reframe Sequence | missing | Subject-tracked reframing and a reframed copy of the project. (#893 owns the ML side.) | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html (F) |
| Audio track types | partial | 5.1 and multichannel layouts, channel-typed tracks, and submix tracks. (#889.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/add-tracks.html (F) |
| Sync lock | partial | Shifting an arbitrary **subset** of tracks: `scope` takes one name or none. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sync-lock-to-prevent-changes.html (F) |
| Track lock | missing | A lock that `shift` and `validate` respect. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/track-lock-to-prevent-changes.html (F) |
| Track output toggle, mute and solo | missing | Turning a whole track off or soloing it without deleting or editing its elements. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/edit-track-appearance.html (F) |
| Track appearance and Timeline Display Settings | partial | A spatial timeline view and an audio waveform view. (GUI chrome; #894.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/edit-track-appearance.html (F) |
| Timeline zoom, scroll and time ruler | partial | A zoomable spatial view. (GUI; #894.) | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/navigation-controls-in-the-timeline.html (F) |
| Sequence label colour in sequence tabs (25.x) | missing | A project-level label. (GUI organisation; #894.) | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Sequence Index panel (26.2) | partial | A flash-frame check: no finding flags a one- or two-frame sliver. A CSV export of the listing. | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/navigate-timelines-with-sequence-index.html (F) |
| Edit from a sequence loaded into the Source Monitor | partial | A live link: the rendered file goes stale and nothing records which project made it. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-presets-and-settings.html (F) |
| Create multi-camera source sequence | missing | The whole construct: a synced angle set. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/create-a-multi-camera-source-sequence.html (F) |
| Multicam sync by timecode (Ignore Hours, LTC, Sound, Aux) | missing | Reading any source timecode, and syncing on it. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/create-a-multi-camera-source-sequence.html (F) |
| Multicam sync by audio | missing | Waveform cross-correlation. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/create-a-multi-camera-source-sequence.html (F) |
| Multicam audio offset | missing | Depends on multicam. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/create-a-multi-camera-source-sequence.html (F) |
| Mark clips for synchronization | missing | Named sync marks and a sync step that reads them. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/mark-clips-for-synchronization.html (F) |
| Multi-camera target sequence and live angle switching | missing | Switchable angles. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/create-and-edit-a-multi-camera-target-sequence.html (F) |
| Edit cameras: reorder or disable angles | missing | Depends on multicam. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/change-the-order-of-cameras-in-source-and-program-monitors.html (F) |
| Multicam audio: Camera 1, All Cameras, Switch Audio | missing | Depends on multicam. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/create-a-multi-camera-source-sequence.html (F) |
| Multi-Camera View overlays and camera names | missing | GUI on top of multicam. | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/display-overlays-in-multi-camera-view.html (F) |
| Multicam thumbnails and poster frames | missing | Depends on multicam. | https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/manage-multi-camera-thumbnails-in-the-project-panel.html (F) |
| Flatten multicam | missing | Depends on multicam. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/simplify-sequence.html (F) |
| Linked Selection and selecting one half of a linked clip | partial | A `video`'s embedded sound cannot be trimmed or moved apart from its picture: only `volume` acts on it. A separate `audio` element with the same `source` is needed. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/select-clips.html (F) |
| Overwrite edit | partial | Arithmetic that trims or splits what is overwritten. The agent does it by hand. | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/add-media-to-the-timeline-using-source-patching.html (F) |
| Extract | partial | A negative `shift`. It is specified in [ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md) but refused in v0.2.0, so closing time is a hand edit of every later time. | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html (F) |
| Find a clip's uses across sequences | partial | A search across several project files. Reveal In Project has no target (no bins, [ADR-0002](../../adr/0002-inline-source-no-asset-table.md)). | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/find-clips-in-a-sequence.html (F) |
| Duplicate frame markers | partial | An overlap check of source ranges that flags reused frames. | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| Ripple edit tool / ripple trim | partial | Shortening needs a negative `shift`, which v0.2.0 refuses. | https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/perform-ripple-edits.html (F) |
| Ripple Sequence Markers | missing | Depends on markers. | https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/perform-ripple-edits.html (F) |
| Ripple trim previous/next edit to playhead | partial | The ripple form needs a negative `shift` to shorten. | https://helpx.adobe.com/premiere-pro/how-to/dynamic-trimming.html (unconfirmed: no fetched page or shortcut table lists Q/W) |
| Asymmetrical trimming | partial | Shifting earlier needs a negative `shift`. No arithmetic applies one trim across tracks. | https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/asymmetrical-trimming.html (F) |
| Ripple delete, close gaps | partial | A negative `shift` to close the time. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/remove-clips-from-a-sequence.html (F) |
| Trim mode, dynamic trimming and trim preferences | partial | An interactive trim monitor with looped playback. (GUI; #894.) | https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/about-trim-mode.html (F) |
| Sequence (timeline) markers | missing | A timed, named, render-inert point on the timeline. | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/add-a-marker-to-a-clip.html (F) |
| Clip markers | missing | A marker in source time, carried by its element. | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/add-a-marker-to-a-clip.html (F) |
| Marker types: comment, chapter, segmentation, web link, Flash cue point | missing | Typed markers, and chapter and segment metadata in the MP4. (#892 owns export.) | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/overview-of-markers.html (F) |
| Duration (spanned) markers | missing | Range markers. | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| Markers panel, marker properties and marker search | missing | Depends on markers. | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/overview-of-markers.html (F) |
| Copy, paste, navigate and clear markers | missing | Depends on markers. | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/copy-and-paste-sequence-markers.html (F) |
| Effect Controls markers (cue points) | missing | Cue-point markers. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-markers-in-the-effect-controls-panel.html (F) |
| Frame.io comments as markers | missing | Markers, then review integration. (#894.) | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/import-comments-as-markers.html (F) |
| Scene Edit Detection | missing | Shot-boundary detection, in the tool or in a skill script. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/detect-edit-points-using-scene-edit-detection.html (F) |
| Link and unlink audio/video | partial | A link binding a separate `audio` element (double-system sound) to a `video`, so a trim or slip of one is checked against the other. | https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/link-audio-and-video-clips.html (F) |
| Merge clips | partial | A merged-clip entity. It would also meet [ADR-0002](../../adr/0002-inline-source-no-asset-table.md)'s asset-table refusal if it were a named bin item. | https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-project-panel.html (F) |
| Synchronize clips | missing | Sync by audio waveform and by timecode. | https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/synchronize-clips-in-the-timeline-panel.html (F) |
| Out-of-sync indicators | partial | A per-pair source-offset check between a picture and its separate sound. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Enable or disable a clip | missing | An element bypass that keeps its values. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/enable-or-disable-a-clip.html (F) |
| Match frame | partial | A direct view of the source at that offset. Today it needs a scratch project (`montagent-footage`). | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/match-a-frame-with-its-source.html (F) |
| Reverse match frame | partial | Mapping a source instant back to the timeline instant. It is the agent's arithmetic today. | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html (F) |
| Source Monitor: open, list, navigate and clear clips | partial | A source viewer with waveform and a list of open sources. (GUI; #894.) | https://helpx.adobe.com/premiere/desktop/edit-projects/compare-edits-to-source-clips/open-or-clear-a-clip-in-the-source-monitor.html (F) |
| Comparison View | partial | A split or before/after view in one image. A before/after of effects needs a scratch copy without them. | https://helpx.adobe.com/premiere/desktop/edit-projects/compare-edits-to-source-clips/compare-edits-to-clips-in-the-program-monitor.html (F) |
| Automatic pixel-aspect assignment | partial | A frame-size rule for files whose metadata carries no PAR. | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/aspect-ratio-preservation.html (F) |
| Offline and online editing with low-resolution copies | partial | A step that makes the copies, a proxy link (#891), and EDL export (#892). | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/creating-clips-for-offline-editing.html (F) |
| Adobe Stock panel | missing | A stock library integration. (#894.) | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/use-built-in-adobe-stock.html (F) |
| Speed/Duration "Ripple Edit, Shifting Trailing Clips" | partial | Shorter needs a negative `shift`. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-using-the-speedduration-option.html (F) |
| Time interpolation: Frame Blending | missing | A per-element interpolation choice and a blending sampler. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html (F) |
| Time interpolation: Optical Flow | missing | Motion-compensated frame synthesis. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html (F) |
| Interpret Footage: frame rate | partial | Interpreting a file once for every use. There is no asset table ([ADR-0002](../../adr/0002-inline-source-no-asset-table.md)), so it is per element. Frame-exact conform depends on the integer `fps`. | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/change-the-frame-rate-of-a-clip.html (F) |
| Interpret Footage: pixel aspect ratio, alpha | partial | An override when the file's own flag is wrong. ADR-0023 names a declared `par`, but the v0.2.0 schema has no such field. | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/modifying-clip-properties-with-interpret-footage.html (F) |
| Interpret Footage: field order | missing | Field-order interpretation. | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/change-the-field-order-of-a-clip.html (F) |
| Field Options: deinterlace, flicker removal, reverse dominance | missing | A deinterlacer and a flicker filter. (#885 for the effect side.) | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/create-interlaced-or-non-interlaced-clips.html (F) |
| Source timecode | missing | Reading source timecode, which sync by timecode needs. | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/choose-timecode-display-format.html (F) |
| History panel | partial | An action list inside the tool. Undo is a git checkout, outside Montagent. | https://helpx.adobe.com/premiere/desktop/edit-projects/correct-mistakes/view-or-make-changes-in-the-history-panel.html (F) |
| Adjustment layers | missing (R3) | Effects that filter the backdrop: `effects` filter only the element's own pixels ([ADR-0040](../../adr/0040-effect-model-attachment-and-v1-vocabulary.md), CONTEXT *Effect*). No ADR refuses it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/create-adjustment-layers.html (F) |
| Render and Replace / Restore Unrendered | partial | Flattening part of the project with alpha and handles, and a recorded way back to the unrendered elements. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-and-replace-media-in-a-sequence.html (F) |
| Simplify Sequence | missing | A clean-up verb. Its vertical-gap move is a hand rewrite of track membership. Its other inputs (multicam, markers) do not exist in Montagent. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/simplify-sequence.html (F) |
| Text-Based Editing: delete text, fillers, pauses and speakers | missing | Turning external word timings into cuts, by a script or a verb. It would also need a negative `shift`. (#888 and #893.) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/detect-and-delete-pauses-in-transcripts.html (F) |

### Video effects and transitions: 73 partial, 153 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Moving a geometry step into the stack (Transform in place of Motion, Alpha Adjust in place of Opacity) | partial | No way to put a transform or opacity step *before* an effect (e.g. blur after scaling). The only route is nesting, which v0.2.0 lacks (ADR-0166; `montagent` skill "can't do yet", #499). | [TYP][TYP], [KEY][KEY] (F) |
| Effect on/off toggle | missing | A visual effect can only be disabled by deleting it, which loses its parameters. | [PERF][PERF] (F) |
| Global FX Mute | missing | A render switch that skips effects. | [PERF][PERF] (F) |
| Effect Controls panel views: timeline view, Value and Velocity graphs, Pin to Clip, filter properties, looped audio preview | missing | *GUI-only.* | [ECP][ECP] (F) |
| Eyedropper and colour picker for colour controls | missing | Sampling a colour from pixels. An agent can read a rendered `frame` and write the hex itself. | [ADJC][ADJC], [UKA][UKA] (F) |
| Effect masks (a mask limits one effect to a region) | partial | A mask cuts the **whole element's** alpha. It cannot scope a single effect. A regional blur needs a second, masked copy of the element. Mask expansion and per-mask opacity are absent. | [MSK][MSK], [VM][VM] (F) |
| Mask tracking | missing | No tracker. An agent would have to measure motion and write keyframes itself. | [MTK][MTK] (F) |
| Effect presets: built-in and custom | partial | No preset store in the format and no apply-by-name. Indirection is avoided on purpose (ADR-0002's no-asset-table reasoning). | [EP][EP], [CPR][CPR], [APR][APR] (F) |
| Preset keyframe retiming: Scale, Anchor to In Point, Anchor to Out Point | missing | Retiming pasted keyframes to a new element's length or ends. | [CPR][CPR] (F) |
| FX badges | missing | *GUI-only.* | [FX][FX] (F) |
| Adjustment layers | missing | An effect over the composite below. ADR-0167 §10 calls this "group compositing" and leaves it in the map's fog, gated on nests. | [AL][AL], [MCO][MCO] (F) |
| Resize an adjustment layer | missing | As adjustment layers. | [RAL][RAL] (F) |
| Effects on a nested sequence | missing | A container, and group compositing for its effects. | [TYP][TYP], [WS][WS] (F) |
| Markers in the Effect Controls time ruler | missing | Markers as authored data. | [ECM][ECM] (F) |
| GPU-accelerated effects | missing | GPU rendering. A performance item; the workflow area owns it. | [ACC][ACC], [TYP][TYP], [MF][MF] (F) |
| 32-bit color (high-bit-depth) effects | missing | A high-bit-depth pipeline. | [B32][B32], [TYP][TYP] (F) |
| YUV effects | missing | Not meaningful without a YUV path. Listed for completeness. | [YUV][YUV], [TYP][TYP] (F) |
| Surprise Me | missing | A randomise operation. An agent can write varied values itself. | [MF][MF], [RN][RN] (F) |
| Film Impact Dashboard | missing | *GUI-only.* | [RN][RN] (F) |
| Bars and Tone | partial | A tone generator. Audio comes only from files. | [BT][BT] (F) |
| Universal Counting Leader | partial | The cue blips need an audio file. No one-step leader. | [CL][CL] (F) |
| Lumetri vignette | partial | Roundness toward a rounded rectangle. The radial gradient is circular or elliptical only. | [LV][LV], [CC][CC] (F) |
| Mask-based vignette | partial | The mask scopes the whole copy, not one effect, and nothing tracks it. | [VM][VM] (F) |
| Extract | missing | A luminance-range threshold. | [ADJ][ADJ], [L][L] (F) |
| Levels | partial | Gamma, invert and per-channel balance. Check against ADR-0049's four-clause rule before any proposal. | [ADJ][ADJ], [L][L] (F) |
| Lighting Effects | missing | Lights with direction, falloff and surface relief. A bump map from another track would be refused (ADR-0150). | [ADJ][ADJ], [L][L], [RN][RN] (F) |
| ProcAmp | partial | Hue rotation. The split-screen compare is *GUI-only*. | [ADJ][ADJ], [L][L], [B32][B32] (F) |
| Gaussian Blur (modern; was "FI: Blur FX") | partial | Horizontal-only or vertical-only blur, and repeat edge pixels. A one-axis blur is approximated by `directional_blur` at 0° or 90° (ADR-0156). | [BS][BS], [L][L], [MF][MF], [RN][RN] (F) |
| Bokeh Blur (was "FI: Bokeh Blur FX") | missing | Shaped bokeh kernel and highlight bloom. | [L][L], [MF][MF] (F) |
| Focus Blur (was "FI: Focus Blur FX") | partial | One member. Today it takes two elements. | [L][L] (F) |
| Channel Blur (modern, 26.2.2) | missing | Per-channel blur. | [MF][MF], [WN][WN], [RN][RN] (F) |
| Reduce Interlace Flicker | missing | Not applicable to progressive output. Listed for completeness. | [BS][BS], [L][L] (F) |
| Sharpen | missing | A sharpen member. #703: Skia `matrix_convolution`. | [BS][BS], [L][L] (F) |
| Unsharp Mask | missing | As Sharpen, plus a threshold. | [BS][BS], [L][L] (F) |
| Camera Blur (Legacy; Windows only) | partial | A lens kernel. `blur` is Gaussian. | [BS][BS], [L][L], [RM][RM] (F) |
| ASC CDL | missing | Per-channel slope, offset and power. Test against ADR-0049's rule. | [CC][CC], [L][L] (F) |
| Lumetri Color | partial | White balance, exposure, the tone-range controls, vibrance, colour wheels and HSL secondaries are missing. The colour area covers Lumetri in depth. | [CC][CC], [LV][LV] (F) |
| Tint | partial | Premiere's two-colour map is a duotone, which ADR-0049 excludes. The one-colour wash ships. | [CC][CC], [L][L] (F) |
| Video Limiter | missing | A legal-range clamp. | [CC][CC], [L][L] (F) |
| Lens Distortion | missing | A lens warp. Workaround: `montagent-prerender`. | [DIS][DIS], [MF][MF], [L][L] (F) |
| Magnify (modern, 26.3) | partial | A loupe that magnifies the element's own pixels in place, without a second element. | [DIS][DIS], [MF][MF], [WN][WN], [RN][RN] (F) |
| Mirror | partial | A mirror line at an arbitrary angle within one element. | [DIS][DIS], [MF][MF], [L][L] (F) |
| Spherize | missing | A bulge warp. Workaround: `montagent-prerender`. | [DIS][DIS], [L][L] (F) |
| Turbulent Displace | missing | A seeded noise displacement. ADR-0156 §3 has the seed and time rules it would need. | [DIS][DIS], [MF][MF], [L][L] (F) |
| Twirl (rebuilt in 26.5) | missing | A twirl warp. Workaround: `montagent-prerender`. | [DIS][DIS], [MF][MF], [L][L], [RN][RN], [WN][WN] (F) |
| Wave Warp | missing | A wave member. ADR-0156 §3 says a wave takes a keyed phase rather than a time input. | [DIS][DIS], [L][L] (F) |
| Warp Stabilizer | missing | Motion analysis and stabilisation. | [DIS][DIS], [WS][WS], [WSS][WSS], [L][L] (F) |
| Warp Stabilizer border framing | missing | Auto-scale from analysis, and edge synthesis from other frames. | [WSS][WSS] (F) |
| Warp Stabilizer Rolling Shutter Ripple | missing | Motion analysis. | [WSS][WSS] (F) |
| Rolling Shutter Repair (Obsolete) | missing | Motion analysis. | [DIS][DIS], [L][L] (F) |
| 4-Color Gradient | partial | A four-point gradient. | [GEN][GEN], [L][L] (F) |
| Gradient (modern, 26.2.2; replaces Ramp) | partial | Repetition and mirroring, an interpolation choice, and the texture layer. | [MF][MF], [WN][WN], [RN][RN] (F) |
| Ramp (Legacy) | partial | Ramp Scatter (dithering). | [GEN][GEN], [L][L] (F) |
| Color Pass | missing | A colour-keyed desaturation. | [IC][IC], [L][L] (F) |
| Color Replace | missing | Colour-range replacement. | [IC][IC], [L][L] (F) |
| Gamma Correction | missing | A gamma member. | [IC][IC], [L][L] (F) |
| Invert (moved here in 26.0) | missing | An invert member. #703: one colour-matrix call. | [CH][CH], [L][L], [RO][RO] (F) |
| VR Blur | missing | Spherical wrap. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Chromatic Aberrations | missing | Spherical model, and channel offsets. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Color Gradients | missing | Spherical model. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR De-Noise | missing | Denoise, and the spherical model. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Digital Glitch | missing | Spherical model. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Fractal Noise | missing | Spherical model and fractal noise. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Glow | missing | Spherical wrap. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Plane to Sphere | missing | Equirectangular projection. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Projection | missing | Spherical model. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Rotate Sphere | partial (R2) | Spherical model. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| VR Sharpen | missing | Sharpen, and the spherical model. | [IMF][IMF], [IMT][IMT], [L][L] (F) |
| Ultra Key | partial | Choke, Pedestal, Highlight and Shadow, matte Contrast and Mid Point, spill Range and Luma, foreground hue, and an alpha-channel view. Soften maps only roughly to `softness`. | [KEY][KEY], [UK][UK], [UKA][UKA], [L][L] (F) |
| Luma Key | missing | A luminance matte. ADR-0152 defers luma-sourced masks. | [KEY][KEY], [L][L] (F) |
| Alpha Adjust | partial | Opacity at a chosen point in the effect order. Ignore Alpha, Invert Alpha and Mask Only, from the earlier reading, are not on the fetched pages (unconfirmed). | [KEY][KEY], [TYP][TYP], [L][L] (F) |
| Logo Cutout (was "FI: Alpha FX") | partial | Mechanism described in one line; re-check. | [L][L], [MF][MF] (F) |
| Echo Glow (was "FI: Echo Glow FX") | missing | A glow that reads earlier frames. | [L][L], [MF][MF] (F) |
| Edge Glow (was "FI: Edge Glow FX") | missing | Edge-detected glow. | [L][L], [MF][MF] (F) |
| Glint (was "FI: Glint FX") | missing | A seeded, time-reading sparkle. | [L][L], [MF][MF] (F) |
| Volumetric Rays (was "FI: Volumetric Rays FX") | missing | A radial ray shader. | [L][L], [MF][MF] (F) |
| Wonder Glow (was "FI: Wonder Glow FX") | partial | A glow colour. ADR-0156 left `color` out on purpose: the glow takes the element's own colours. | [L][L], [MF][MF] (F) |
| Focus Glow FX | partial | One member. | [MF][MF] (F) |
| Noise (modern, 26.2.2) | partial | Tonal-range weighting, and a blend mode inside the effect. A grain `rect` with `blend: screen`/`add`/`overlay` covers part of the latter; Soft Light is not a blend mode (ADR-0147). | [NG][NG], [MF][MF], [L][L], [RN][RN] (F) |
| Basic 3D | partial | The specular highlight. | [PER][PER], [L][L], [ACC][ACC] (F) |
| Drop Shadow | partial | Shadow Only. | [PER][PER], [L][L] (F) |
| Long Shadow (was "FI: Long Shadow FX") | missing | A long cast shadow. #703: a runtime shader. | [L][L], [MF][MF] (F) |
| Alpha Glow (Legacy) | partial | A two-colour, fading glow. | [STY][STY], [L][L] (F) |
| Brush Strokes | missing | Seeded brush strokes. | [STY][STY], [L][L] (F) |
| Color Emboss | missing | A relief kernel. | [STY][STY], [L][L] (F) |
| Find Edges | missing | A convolution kernel. | [STY][STY], [L][L] (F) |
| Mosaic (modern; was "FI: Mosaic FX") | missing | A mosaic member. `montagent-prerender` names mosaic as its job, but only for footage processed outside Montagent. | [STY][STY], [MF][MF], [L][L], [RN][RN], [WN][WN] (F) |
| Roughen Edges | missing | Seeded edge displacement. | [STY][STY], [L][L] (F) |
| Strobe Light | partial | The arithmetic operations (invert and the like); there is no invert member. | [STY][STY], [L][L] (F) |
| Echo (Obsolete) | missing | Frame echoes. | [TIME][TIME], [L][L] (F) |
| Posterize Time | partial | A single frame-rate field per element. | [TIME][TIME], [L][L] (F) |
| Auto Reframe (clip) | missing | Subject tracking. | [TR][TR], [AR][AR], [ARC][ARC], [L][L] (F) |
| Auto Reframe Sequence | missing | Subject tracking, and a whole-project reframe. | [ARS][ARS], [AR][AR], [RN][RN] (F) |
| Offset (moved here in 26.0) | missing | Wrap-around tiling. #703: an image shader with repeat tiling. | [DIS][DIS], [L][L], [RO][RO] (F) |
| Spacer (was "FI: Spacer FX") | missing | Mechanism described in one line; re-check. | [L][L], [MF][MF] (F) |
| Transform (effect) | partial | There is no way to place the transform inside the effect order (see Effect controls). | [TR][TR], [TYP][TYP], [L][L], [RN][RN] (F) |
| Block Dissolve (Legacy) | missing | Seeded block pattern. | [TE][TE], [L][L] (F) |
| Auto Align (was "FI: Auto Align FX") | partial | Automatic placement. Montagent places from literals. | [L][L], [MF][MF] (F) |
| Cineon Converter | missing | Log and colour-space conversion. The colour area owns it. | [UT][UT], [L][L] (F) |
| Metadata & Timecode Burn-in | missing | Text generated from metadata, and a running timecode. | [VF][VF], [L][L], [RM][RM], [RN][RN] (F) |
| Stroke (was "FI: Stroke FX") | partial | A stroke round the alpha of an image or video. | [L][L], [MF][MF] (F) |
| SDR Conform (Obsolete in the 26.0 table) | missing | HDR handling. The colour area owns it. | [VF][VF], [L][L] (F) |
| Arithmetic | missing | Per-channel arithmetic. ADR-0049 excludes per-channel colour matrices, so any proposal starts there. | [L][L], [RM][RM] (F) |
| Auto Color | missing | A colour correction computed from source pixels. | [L][L], [RM][RM] (F) |
| Auto Contrast | missing | The automatic analysis. | [L][L], [RM][RM] (F) |
| Auto Levels | missing | Per-channel levels, automatic or manual. | [L][L], [RM][RM] (F) |
| Bevel Alpha | missing | Alpha-edge bevel. | [L][L], [RM][RM] (F) |
| Bevel Edges | missing | Edge bevel. | [L][L], [RM][RM] (F) |
| Blend | partial | Naming a specific other layer is refused (ADR-0150). Colour-only, tint-only, darken and lighten modes were refused for now (ADR-0147 "Considered and refused"). | [L][L], [RM][RM] (F) |
| Change Color | missing | Hue-selective correction. | [L][L], [RM][RM] (F) |
| Change to Color | missing | Hue-selective replacement. | [L][L], [RM][RM] (F) |
| Clip Name | partial | Automatic text from clip metadata. | [L][L], [RM][RM] (F) |
| Color Balance | missing | Independent per-channel gain. | [L][L], [RM][RM] (F) |
| Color Balance (HLS) | partial | Hue rotation. | unconfirmed (no fetched page names an HLS variant) |
| Convolution Kernel | missing | An arbitrary kernel. #703 maps it to Skia `matrix_convolution`. | [L][L], [RM][RM] (F) |
| Dust & Scratches | missing | As Median. | [L][L], [RM][RM] (F) |
| Emboss | missing | As Color Emboss. | [L][L], [RM][RM] (F) |
| Equalize | missing | Histogram equalisation. | [L][L], [RM][RM] (F) |
| Eyedropper Fill | missing | A colour sampled from pixels. That would fail ADR-0145's literal-values test if done at render time. | [L][L], [RM][RM] (F) |
| Fast Color Corrector | missing | Colour wheels and levels. | [L][L], [RM][RM] (F) |
| Ghosting | missing | Frame trails. | [L][L], [RM][RM] (F) |
| Horizontal Hold | missing | Mechanism undocumented on the fetched pages; re-check. | [L][L], [RM][RM] (F) |
| Image Matte Key | missing | An image-sourced mask. | [L][L], [RM][RM] (F) |
| Leave Color | missing | A colour-keyed desaturation. | [L][L], [RM][RM] (F) |
| Luma Corrector | missing | Tonal-range correction. | [L][L], [RM][RM] (F) |
| Median (Legacy) | missing | A median filter. | [L][L], [RM][RM] (F) |
| Noise Alpha | missing | Alpha noise. | [L][L], [RM][RM] (F) |
| Noise HLS | partial | Separate hue, lightness and saturation amounts. | [L][L], [RM][RM] (F) |
| Noise HLS Auto | partial | As Noise HLS. | [L][L], [RM][RM] (F) |
| Non Red Key | partial | Mechanism undocumented on the fetched pages; re-check. | [L][L], [RM][RM] (F) |
| Paint Bucket | missing | A pixel-dependent fill. | [L][L], [RM][RM] (F) |
| Radial Shadow | missing | A point-light projection. | [L][L], [RM][RM] (F) |
| Radial Wipe (effect) | partial | A pie mask with one angle parameter. Today it is many keyed vertices. | [L][L] (F) |
| Remove Matte | missing | Unpremultiply against a matte colour. | [L][L], [RM][RM] (F) |
| RGB Color Corrector | missing | Per-channel tonal correction. | [L][L], [RM][RM] (F) |
| Shadow/Highlight | missing | Tone-range-dependent lift and compression. | [L][L], [RM][RM] (F) |
| Solarize | missing | A solarize curve. | [L][L], [RM][RM] (F) |
| Three-Way Color Corrector | missing | Tonal-range colour wheels. | [L][L], [RM][RM] (F) |
| Threshold | missing | A luminance threshold at a chosen level. | [L][L], [RM][RM] (F) |
| Timecode | missing | A generated, running timecode. | [L][L], [RM][RM] (F) |
| Venetian Blinds (effect) | missing | A union of strips. | [L][L] (F) |
| Vertical Hold | missing | Mechanism undocumented on the fetched pages; re-check. | [L][L], [RM][RM] (F) |
| Video Limiter (Legacy) | missing | As Video Limiter. | [L][L], [RM][RM] (F) |
| Default transition duration | missing | A project-level default. | [TD][TD], [DT][DT], [MCA][MCA] (F) |
| Single-sided transition | partial | A single-sided transition element. ADR-0059 requires two bridged elements. | [SS][SS], [TO][TO] (F) |
| Transition settings: Start and End %, border width and colour, Reverse, Anti-Aliasing Quality, Custom | missing | The options. They are deferred, not refused. | [CTS][CTS] (F) |
| Show Actual Sources | missing | *GUI-only.* | [CTS][CTS] (F) |
| Default transition | missing | A project-level default. | [DT][DT], [RO][RO] (F) |
| Apply default transitions to a selection; Automate to Sequence | missing | A batch apply of a default. | [DT][DT] (F) |
| Clip handles and insufficient media | partial | No hold before a source's first frame, for an incoming element without a head handle. | [HAND][HAND], [TO][TO] (F) |
| Video transition handles (26.0); show or hide them (26.5) | missing | *GUI-only.* | [VTH][VTH], [WN][WN], [RN][RN] (F) |
| Block Motion | partial | Blocks as one parameter. Today it is N copies. | [L][L], [AN][AN], [MT][MT] (F) |
| Fold Motion | partial | Several folds take many copies; no depth sorting (ADR-0167). | [L][L], [AN][AN], [MT][MT] (F) |
| Burn Alpha | missing | A burn-edge matte. | [L][L], [MT][MT] (F) |
| Film Dissolve | missing | A linear-light crossfade. ADR-0147's refusal is about `blend`, not transitions, so this is `missing`. | [VD][VD], [L][L] (F) |
| Luma Fade | missing | A luma-ordered reveal (ADR-0152 defers luma-sourced masks). | [L][L], [MT][MT] (F) |
| Morph Cut | missing | Optical flow and face tracking. | [VD][VD], [MCO][MCO], [MCA][MCA], [MC][MC], [L][L] (F) |
| Mosaic (transition; was "FI: Mosaic Impacts") | missing | A mosaic member. | [L][L], [MT][MT] (F) |
| Chaos | missing | A seeded distortion. | [L][L], [MT][MT] (F) |
| Glass | missing | A refraction warp. | [L][L] (F) |
| Grunge | partial | Scratches and dirt. Pre-rendered texture via `montagent-prerender`. | [L][L] (F) |
| Kaleidoscope | missing | A kaleidoscope warp. | [L][L] (F) |
| Liquid Distortion | missing | A displacement warp. | [L][L] (F) |
| VHS Damage | partial | Tracking-line displacement. | [L][L] (F) |
| Burn Chroma | missing | A burn matte with channel split. | [L][L], [MT][MT] (F) |
| Cross Zoom (Adobe lists it under Zoom on the transitions list) | partial | A radial zoom blur. | [VT][VT], [L][L] (F) |
| Lens Blur | partial | A lens (bokeh) kernel. | [L][L] (F) |
| Phosphor | missing | Light trails, which read earlier frames. | [L][L], [MT][MT] (F) |
| Radial Blur | missing | A rotational blur. | [L][L], [MT][MT] (F) |
| Ray | missing | A ray shader. | [L][L], [MT][MT] (F) |
| Solarize (transition) | missing | A solarize curve and an invert. | [L][L], [MT][MT] (F) |
| Zoom Blur | missing | A radial zoom blur. | [L][L], [MT][MT] (F) |
| Film Roll | partial | Frame borders and sprocket texture, and a motion smear (`motion_blur` covers that). | [L][L], [MT][MT] (F) |
| Roll | partial | Mechanism only described in one line; re-check. | [L][L], [MT][MT] (F) |
| Split | partial | Sections as one parameter. Today it is N copies (ADR-0148). | [VT][VT], [L][L], [MT][MT] (F) |
| Stretch | partial | The warp. Scale is uniform across the element. | [L][L], [MT][MT] (F) |
| Shape Dissolve (26.3) | missing | A pattern of many shapes revealing at once. | [MT][MT], [WN][WN], [RN][RN] (F) |
| Shape Flow | partial | The generator. Each shape is written by hand. | [L][L], [MT][MT] (F) |
| Frame | partial | Mechanism described in one line; re-check. | [L][L], [MT][MT] (F) |
| Louver | missing | A union of strips, as Venetian Blinds. | [L][L], [MT][MT] (F) |
| Mirror (transition) | partial | Mechanism only described in one line; re-check. | [L][L], [MT][MT] (F) |
| Page Peel (modern) | missing | A page-curl warp. | [L][L], [MT][MT] (F) |
| Slice | partial | Slices as one parameter. Today it is N copies (ADR-0148). | [L][L], [MT][MT] (F) |
| Wave | missing | A wave warp. | [L][L], [MT][MT] (F) |
| Clock Wipe (modern) | partial | A one-angle pie mask. | [L][L], [MT][MT] (F) |
| Neon Wipe | partial | A lit edge as part of the wipe. | [L][L], [MT][MT] (F) |
| Panel Wipe | partial | Panels as one parameter. | [L][L], [MT][MT] (F) |
| Plateau Wipe | missing | A fold warp with depth. | [L][L], [MT][MT] (F) |
| Radial Wipe (modern) | partial | As Clock Wipe. | [VT][VT], [L][L] (F) |
| Soft Wipe | partial | A lens (bokeh) kernel. | [L][L], [MT][MT] (F) |
| Stretch Wipe | missing | A warp at the edge. | [L][L], [MT][MT] (F) |
| Center Split (Legacy) | partial | As Split. | [VT][VT], [L][L] (F) |
| Band Slide (Obsolete) | partial | Bands as one parameter. Today it is N copies (ADR-0148). | [VT][VT], [L][L], [RM][RM] (F) |
| Band Wipe (Obsolete) | missing | A union of band masks. | [VT][VT], [L][L], [RM][RM] (F) |
| Checker Wipe (Obsolete) | missing | A union of many masks. | [VT][VT], [L][L], [RM][RM] (F) |
| CheckerBoard (Obsolete) | missing | As Checker Wipe. | [VT][VT], [L][L], [RM][RM] (F) |
| Page Peel (Obsolete) | missing | A page-curl warp. | [VT][VT], [L][L] (F) |
| Page Turn (Obsolete) | missing | As Page Peel. | [VT][VT], [L][L] (F) |
| Paint Splatter (Obsolete) | missing | A splatter matte. | [VT][VT], [L][L], [RM][RM] (F) |
| Pinwheel (Obsolete) | missing | A union of blades. | [VT][VT], [L][L], [RM][RM] (F) |
| Random Blocks (Obsolete) | missing | Seeded blocks. | [VT][VT], [L][L], [RM][RM] (F) |
| Random Wipe (Obsolete) | missing | Seeded blocks. | [VT][VT], [L][L], [RM][RM] (F) |
| Spiral Boxes (Obsolete) | missing | A union of boxes. | [VT][VT], [L][L], [RM][RM] (F) |
| Venetian Blinds (Obsolete transition) | missing | A union of strips. | [VT][VT], [L][L], [RM][RM], [SS][SS] (F) |
| Wedge Wipe (Obsolete) | partial | A one-angle wedge. | [VT][VT], [L][L], [RM][RM] (F) |
| Zig-Zag Blocks (Obsolete) | missing | A union of blocks. | [VT][VT], [L][L], [RM][RM] (F) |
| Cube Spin (removed 26.0) | partial | No depth sorting (refused, ADR-0167), so the two faces' stacking is set by layer and may need a swap mid-turn. | [L][L], [RM][RM] (F) |
| Gradient Wipe (transition; removed 26.0) | missing | A gradient-driven reveal. If the gradient is another element, ADR-0150 §4 refuses it. | [L][L], [RM][RM] (F) |
| VR Chroma Leaks | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| VR Gradient Wipe | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| VR Iris Wipe | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| VR Light Leaks | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| VR Light Rays | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| VR Mobius Zoom | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| VR Random Blocks | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| VR Spherical Blur | missing | Spherical model. | [IMT][IMT], [VT][VT], [L][L] (F) |
| Gaussian Blur (Legacy) | partial | One-axis blur and repeat edge pixels. | [L][L] (F) |
| Magnify (Legacy) | partial | As Magnify. | [L][L] (F) |
| Mosaic (Legacy) | missing | A mosaic member. | [L][L] (F) |
| Twirl (Legacy) | missing | A twirl warp. | [RN][RN] (F) |
| Noise (Legacy) | partial | As Noise. | [RN][RN] (F) |
| Clock Wipe (Legacy) | partial | A one-angle pie mask. | [L][L], [CTS][CTS] (F) |
| Cross Zoom (Legacy) | partial | A radial zoom blur. | [L][L] (F) |
| Film Dissolve (Legacy) | missing | A linear-light crossfade. | [L][L] (F) |
| Radial Wipe (Legacy) | partial | A one-angle pie mask. | [L][L] (F) |
| Split (Legacy) | partial | Sections as one parameter. | [L][L] (F) |

### Motion, keyframing and compositing: 23 partial, 62 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Motion: Anchor Point | partial | An arbitrary pivot point (any x/y inside or outside the box) and a keyed pivot. Only the nine snap points ship, and they are static. | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/edit-video-using-the-properties-panel.html (F); keyed: https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html (F) |
| Motion: Anti-flicker Filter | missing | The whole mechanism. | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/eliminate-flicker.html (F) |
| Opacity render order | partial | Moving opacity earlier or later in the chain. The order is one fixed rule. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/compositing-overview.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html (F) |
| Alpha Adjust effect | missing | An opacity step placed in the effect chain. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html (F) |
| Blend Mode menu (in Opacity) | partial | 22 of the 27 modes (rows in section 6). | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/combine-multiple-videos-or-stil-images-using-blend-modes.html (F) |
| Straight vs premultiplied alpha | partial | No stated rule for which alpha a source is read as, and no per-source override. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/compositing-overview.html (F) |
| Alpha-carrying file formats | partial | PSD, TGA, TIFF, EPS, PDF and AI sources. The import area owns the format list. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/compositing-overview.html (F) |
| Channel effects for transparency | missing | Any channel-to-alpha operation. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-composites/compositing-overview.html (F) |
| Time interpolation: Frame Blending | missing | The whole mechanism. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html (F) |
| Time interpolation: Optical Flow | missing | The whole mechanism. A skill could pre-render it the way ADR-0171 does for Lottie. | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/apply-time-interpolation-methods-to-adjust-clip-speed.html (F) |
| Keyframe navigation (go to previous/next keyframe) | partial | A "next keyframe" step across all lists. An agent reads the `t` values directly. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/move-current-time-indicator-to-keyframe.html (F) |
| Track-level keyframes | missing | Any property or keyframe on a track. The audio area owns the audio side. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/view-keyframes-and-properties-in-the-timeline-panel.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/copy-and-paste-keyframes.html (F) |
| Filter properties in Effect Controls | partial | A filtered view or a query that lists only keyed or only edited properties. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/filter-properties-in-the-effect-controls-panel.html (F) |
| Summary keyframes | partial | A per-effect roll-up of key times. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/view-keyframes-in-the-effect-controls-panel.html (F) |
| Effect Controls timeline view | missing | The GUI. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/view-keyframes-in-the-effect-controls-panel.html (F) |
| View keyframes in the Timeline (one property per clip, tooltip) | partial | A visual timeline of keys. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/view-keyframes-and-properties-in-the-timeline-panel.html (F) |
| Interpolation: Auto Bezier | partial | Automatic tangents. The author must compute handles that make the rate continuous through each key. No ease name does it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html (F) |
| Interpolation: Continuous Bezier | partial | A constraint tying one segment's outgoing handle to the next segment's incoming handle. Continuity is the author's job, and `validate` does not check it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html (F) |
| Spatial interpolation (motion path) | missing | A spatial curve between position keys with its own handles. A curved move today is two decoupled per-axis eases or many keys. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/control-effect-changes-using-keyframe-interpolation.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/control-effect-changes-using-bezier-keyframe-interpolation.html (F) |
| Value graph | partial | An interactive graph view. Montagent has no GUI. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/edit-keyframes-graphs.html (F) |
| Velocity graph / influence | partial | A velocity view and velocity in units per second. An agent derives speed from the bezier itself. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/adjust-effect-speed.html (F) |
| Timeline rubber-band keyframing | missing | The GUI. | https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/keyframes-and-graphs-in-panels.html (F) |
| Mask bounding-box mode | partial | Rotating a mask apart from its element, and scaling a `path` mask's outline (its rect bounds the points and never scales them, ADR-0163; a scaling viewBox is refused there). | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/rectangle-ellipse-and-pen-masks.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/modifiers-and-conditions-for-masking.html (F) |
| Mask Transform (Position, Scale, Rotation, Anchor) | partial | Mask scale, rotation and anchor, and a transform layered on top of the path keys. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-masks-at-the-clip-or-frame-level.html (F) |
| Clip-level vs frame-level mask adjust | partial | Clip mode's offset over keyed motion. A whole-clip change to a keyed mask means rewriting every key, since there is no second layer to add. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-masks-at-the-clip-or-frame-level.html (F) |
| Mask combine: Add (union) | missing | A union of two shapes on one element. The workaround is two copies of the element. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html (F) |
| Mask scoped to an effect | missing | Scoping a blur, colour or other effect to a region. The workaround is a masked duplicate element. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-shapes.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-masks-using-shapes.html (F) |
| Unassigned masks | missing | Masks as reusable objects. Copies are written per element (ADR-0163 §1). | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/apply-effects-to-existing-unassigned-masks.html (F) |
| Move masks between effects | missing | Moving a mask between effect scopes, which needs effect-scoped masks first. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/copy-and-paste-masks.html (F) |
| Mask overlay | partial | An overlay or alpha-only view of the mask itself. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html (F) |
| Direct manipulation toggle | missing | The GUI. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html (F) |
| Blend arithmetic at the project's bit depth | partial | Higher bit depth and unclamped float results. | (above, F) |
| Mask tracking: forward / backward / bi-directional / one frame | missing | An analysis tool that writes the mask keyframes. No ADR refuses it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/track-masks.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html (F) |
| Mask tracker option: Position | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html (F) |
| Mask tracker option: Scale | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html (F) |
| Mask tracker option: Rotation | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html (F) |
| Mask tracker option: Perspective | missing | As above, plus a per-vertex result; a tracked `path` could carry it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html (F) |
| Vector mask tracking engine | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-masks-at-the-clip-or-frame-level.html (F) |
| Warp Stabilizer (effect) | missing | The whole mechanism. Render-time analysis would break ADR-0146 criterion 2 (nothing carried between frames), so it fits a pre-render skill or a tool that writes keyframes. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/stabilize-shaky-footage-using-warp-stabilizer.html (F) |
| Warp Stabilizer: Analyze / Cancel | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Result: Smooth Motion + Smoothness | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Result: No Motion | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Method: Position; Position, Scale, Rotation; Perspective; Subspace Warp | missing | As above. The first two could land as `x`/`y`/`scale`/`rotation` keyframes; Perspective and Subspace Warp could not (corner pin refused, ADR-0167). | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer: Preserve Scale | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Framing: Stabilize Only; Stabilize, Crop; Stabilize, Crop, Auto-scale; Stabilize, Synthesize Edges | missing | The analysis, and edge synthesis from neighbouring frames. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Auto-scale: Maximum Scale, Action-Safe Margin | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer: Additional Scale | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Advanced: Detailed Analysis, Fast Analysis | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Advanced: Rolling Shutter Ripple | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Advanced: Crop Less ↔ Smooth More | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Advanced: Synthesis Input Range, Edge Feather, Edge Cropping | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Warp Stabilizer Advanced: Hide Warning Banner | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/warp-stabilizer-settings.html (F) |
| Auto Reframe (sequence) | missing | Subject tracking and a new-aspect copy of the project. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html (F) |
| Auto Reframe sequence: custom Resolution | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html (F) |
| Auto Reframe sequence: Clip Nesting | missing | As above (and nesting, section 11). | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html (F) |
| Auto Reframe (clip effect) | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-clip.html (F) |
| Auto Reframe Motion Tracking presets: Slower / Default / Faster | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html (F) |
| Auto Reframe adjustments: Overwrite Generated Path, Adjust Position, Reframe Offset, Scale, Rotation | missing | The generated path to adjust. | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-clip.html (F) |
| Ultra Key: Key Color + Matte Generation | partial | Highlight, Shadow, Pedestal and a separate Transparency. Only one distance threshold and one soft band ship. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/apply-and-customize-chromakey-using-the-ultra-key-effect.html (F) |
| Ultra Key: Matte Cleanup | missing | Choke, soften and contrast on the matte apart from the key. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html (F) |
| Ultra Key: Spill Suppression | partial | Desaturate, Range and Luma restore. One spill amount ships. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html (F) |
| Ultra Key: Color Correction | partial | Hue rotation; no effect member rotates hue. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/ultra-key-effect-parameters.html (F) |
| Ultra Key: Output views | partial | A rendered alpha-only (matte) view and a status view. Coverage is reported as numbers. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/apply-and-customize-chromakey-using-the-ultra-key-effect.html (F) |
| Luma Key | missing | The whole mechanism. No ADR refuses it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html (F) |
| Image Matte Key | missing | An asset-referenced matte. Premiere itself no longer ships it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html (F) |
| Difference Matte | missing | The whole mechanism (it needs a second source inside an effect, the same block ADR-0152 §3 names). Premiere itself no longer ships it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html (F) |
| Edge Feather (Legacy in 26.0) | partial | A border that darkens to black. Montagent's feather fades to the backdrop, which is black only when nothing lies below and the `background` is black. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html (F) |
| Basic 3D: Specular Highlight | missing | The whole mechanism. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html (F) |
| Nested sequence with its own Motion transform | missing | A space-only container (group transform). Its ruling (#632) is still open. | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-nested-sequences/about-nested-sequences.html (F); https://helpx.adobe.com/premiere/desktop/edit-projects/edit-nested-sequences/nest-a-sequence-in-another-sequence.html (F) |
| Dynamic Link: share assets between AE and Premiere | missing | As above. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/share-assets-between-after-effects-and-premiere-using-dynamic-link.html (F) |
| Dynamic Link: relink offline compositions | missing | As above (no link to relink). | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/relink-a-dynamically-linked-composition.html (F) |
| Dynamic Link: make composition offline | missing | As above. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/make-a-dynamically-linked-composition-offline.html (F) |
| Copy layers from After Effects to Premiere | missing | The whole mechanism. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/copy-from-after-effects-to-premiere.html (F) |
| Advanced mask tracking in After Effects | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/track-masks.html (F) |
| Object Masking (AI) | missing | Segmentation, and the per-frame alpha-matte source it would need. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F); https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Object mask selection by Rectangle or Lasso | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/modifiers-and-conditions-for-masking.html (F) |
| Object mask refinement (+ / −) | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F) |
| Object mask Sharp / Smooth modes | missing | The whole mechanism (depends on Object Masking). | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/apply-sharp-and-smooth-modes-in-object-masking.html (F) |
| Object mask Sharpness (Smooth-mode edge refinement, 26.5) | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F); https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| Object detection coverage (26.5) | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F) |
| Object mask data storage | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F) |
| Locate / restore offline object masks | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F); https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Object mask quality feedback | missing | As above. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F) |
| HSL Mask Tool | missing | A colour-range mask source. No ADR names it. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-hsl-and-luminance.html (F) |
| Luminance Mask Tool | missing | A luminance-range mask source. | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-hsl-and-luminance.html (F) |

### Colour: 42 partial, 113 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| White balance eyedropper | missing | Sampling a frame to a white-balance correction. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F) |
| Temperature | missing | A luminance-preserving per-channel gain on a temperature axis. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F) |
| Tint (white balance) | missing | A green to magenta balance axis. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F) |
| Exposure | partial | A multiplicative gain with black pinned (exposure in stops) and a shadow roll-off. An offset lifts blacks, and exposure does not. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F) |
| Contrast pivot (Color mode) | partial | A pivot parameter, and the shadow roll-off. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F) |
| Highlights | missing | A luminance-range-weighted adjustment. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F) |
| Shadows | missing | As above. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F) |
| Whites | partial | An independent white point with its own soft roll-off. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F) |
| Blacks | partial | An independent black point with its own soft roll-off. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F) |
| Black (Color mode) | partial | A floor that bounds the other controls, and a lift that leaves the highlights in place. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F) |
| Auto (tone) | missing | Footage analysis that writes literal `brightness`/`contrast` values. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F) |
| Auto Color (Sensei ML) | missing | Analysis that writes literal `effects`. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/use-auto-color.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-correction-workflow.html (F) |
| Auto Color Intensity | missing | Falls with Auto Color. A per-stack mix is also missing. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/use-auto-color.html (F) |
| Lumetri Presets (film stock and camera looks) | missing | A named library of saved `effects` lists. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/lumetri-enhancements.html (F) |
| Faded Film | missing | A fade that leaves highlights alone. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F) |
| Sharpen (Creative and HSL Secondary) | missing | A `sharpen` member. It is deferred, not refused. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html (F) |
| Vibrance | missing | Saturation weighted by each pixel's own saturation. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F) |
| Shadow Tint / Highlight Tint wheels | missing | Tone-range-weighted tinting. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F) |
| Tint Balance | missing | A green to magenta balance axis (as for Tint above). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F) |
| Shadows / Midtones / Highlights colour wheels | missing | Lift/gamma/gain by tonal range, with a hue offset per range. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-color-wheel.html (F) |
| Color Match (auto) | missing | A frame-to-frame colour statistic and a tool that writes literal effect values from it. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html (F) |
| Face Detection (in Color Match) | missing | As above, plus face detection. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html (F) |
| Apply Match / fine-tune after match | missing | The wheels it writes into, and the match. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html (F) |
| Key: Set colour eyedropper, plus and minus eyedroppers | partial | Sampling, and growing or shrinking the key with more samples. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html (F) |
| Key: H, S, L range sliders (range and softness triangles) | partial | Independent H, S and L ranges and softnesses. A luminance-only qualifier. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html (F) |
| Color/Gray, Color/Black, White/Black mask preview | missing | A matte view from `frame`. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html (F) |
| Refine: Denoise, Blur | partial | A matte blur and a matte denoise. ADR-0088 says edge quality is "specified but untested". | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html (F) |
| Invert key (HSL Secondary) | missing | An inverted colour key. | unconfirmed: the HSL Secondary page does not describe an invert. (The HSL Mask Tool can be inverted, see *HSL mask* below.) |
| Correction on the selection | partial | Correcting **inside** the key, which needs an inverted key or a key that gates a correction. Two elements decode the same source. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html (F) |
| Isolate one colour (Color Pass effect) | partial | A single-element mechanism keyed by hue range. Today it takes a duplicated element and draws a review finding. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/image-control-effects.html (F) |
| Adjustment layer (grade a whole track or sequence) | missing | An adjustment element, or a track- or project-level `effects`. Today a grade is repeated on every element. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hsl-secondary-controls.html (F) |
| Colour effects across several clips by nesting | missing | A nest, or an adjustment element. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/about-color-correction-effects.html (F) |
| Lumetri Color workspace | missing | A grading surface. It is GUI-only and outside the format's reach, but no ADR refuses it. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/about-color-grading.html (F) |
| Masks on a colour operation (Object Mask, Ellipse, Rectangle, Pen) | partial | A mask on the *correction* rather than on a duplicate element. Object (AI) masks. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html (F) |
| Mask tracking and Frame or Clip keyframe modes | partial | Tracking. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html (F) |
| Mask blend modes (Add, Subtract, Intersect) | partial | Add (union) and Subtract. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html (F), https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html (F) |
| HSL mask | partial | Sampling, H/S/L ranges, an inverted key, and using it as a mask for a correction (see the HSL Secondary rows). | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-hsl-and-luminance.html (F) |
| Luminance mask | missing | A self-luma matte (deferred, not refused). | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/create-masks-using-hsl-and-luminance.html (F) |
| Color mode workspace (Color tab, Color monitor, Color Controls, Clip Grid, per-mode workspaces) | missing | A grading surface. It is GUI-only and outside the format's reach, but no ADR refuses it. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/open-a-sequence-in-color-mode.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/work-in-color-mode.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/lumetri-color.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-workflow.html (F) |
| Edit-mode quick colour (Properties panel Color section) | partial | The controls listed in their own rows (temperature, pivot, black, texture, sharpness). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/applying-clip-operations.html (F) |
| Default Adjust operation (Adjust 1 on every clip) | partial | The Adjust controls listed in their own rows. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/default-adjust-operation.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F) |
| Operations as ordered layers (Clip, then Custom Group, then Sequence) | partial | The group and sequence levels (rows below). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-tools-panel-overview.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-the-clip-grid.html (F) |
| Multiple operations: add, rename, duplicate, reorder, reset, delete, move between levels | partial | A label to rename, and levels to move between. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/managing-multiple-color-operations.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-tools-panel-overview.html (F) |
| Toggle an operation or effect off without deleting it (A/B versions) | missing | A bypass flag on an effect. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/managing-multiple-color-operations.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-style.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html (F) |
| Sequence operations (grade the composited sequence, "show look") | missing | A project-level or adjustment `effects` that runs on the composite. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/applying-operations-to-the-sequence.html (F) |
| Exclude top tracks from sequence operations | missing | A track boundary for a composite-level grade. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/applying-operations-to-the-sequence.html (F) |
| Custom groups (shared operations across member clips) | missing | Effects shared by reference across elements. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/using-custom-groups.html (F) |
| Group membership editing | missing | — | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/using-custom-groups.html (F) |
| Copy and paste a whole grade | partial | Pasting into a shared group (falls with groups). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/copy-and-paste-for-grade-management.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-the-clip-grid.html (F) |
| Copy and paste a single control, one axis of a 2D control, a zone or a swatch | partial | Zones and swatches (fall with those controls). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/copy-and-paste-for-grade-management.html (F) |
| Paste keyframed operations with keyframes offset to the target clip | partial | Pasting keyframes relative to the target element's start. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/copy-and-paste-for-grade-management.html (F) |
| Setup chosen from the first clip | missing | Falls with colour management. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/work-in-color-mode.html (F) |
| Balance (polar colour balance) | partial | Chromatic adaptation, a cast-cancelling direction, and the shadow roll-off. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/default-adjust-operation.html (F) |
| Tonal zones (Exposure, Temperature, Balance, Saturation per zone) | missing | Luminance-thresholded zones with a blend. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/default-adjust-operation.html (F) |
| Automatic zone analysis and threshold eyedroppers | missing | Analysis that writes literal thresholds. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-the-clip-grid.html (F) |
| Sat Shift | missing | A hue-ranged saturation member. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Color (Hue) Shift | missing | A hue-ranged hue rotation. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Lum Shift | missing | A hue-ranged lightness change. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Hue swatches (auto-extracted, sampled, Full Spectrum, Memory Color) | missing | Falls with the Shift controls, plus analysis. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Texture (one control or five bands) | partial | Sharpening, and band-limited (frequency-split) control. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F) |
| Sharpness (amount, threshold; Amount, Radius, Threshold sliders) | missing | A `sharpen` member. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-adjust.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-adjustment-in-edit-mode.html (F) |
| Color Controls panel (latched 2D controls, sensitivity, nudge, dead zone, keyboard latching) | missing | GUI-only, and no ADR refuses it. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-tools-panel-overview.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-the-clip-grid.html (F) |
| Heads-up displays (HUDs) | missing | A per-control colour reading. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/understanding-heads-up-displays.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/default-adjust-operation.html (F) |
| Style operation (an ordered stack of interchangeable modules) | partial | The modules (rows below), and mute (see the toggle row). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-style.html (F) |
| Style presets (browse, preview, apply, save, install) | missing | A named library of saved `effects` lists. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/grading-clips-using-style.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-workflow.html (F) |
| Color & Contrast module | partial | Pivot, gain exposure, black floor, Temperature/Tint and Balance. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Contrast Kit: Compression (S-curve with pivot) | missing | An S-curve contrast member. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Contrast Kit: Range | partial | — (only the S curve within the range, see above). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Contrast Kit: Highlights (HDR range) | missing | Falls with HDR. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Detail module | partial | Sharpening and band control (see Texture and Sharpness). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Film Color (negative and print stock emulation) | missing | Film-stock emulation. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Flare (black point with threshold and slope) | missing | A thresholded black lift. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/available-style-modules.html (F) |
| Color monitor Sequence and Clip modes, loop and navigation | partial | A live, looping, clip-scoped viewer (GUI). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-monitor-overview.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-the-clip-grid.html (F) |
| Solo mode | missing | An element-isolation option on `frame`. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-monitor-overview.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-the-clip-grid.html (F) |
| RGBA channel view | missing | A channel option on `frame`. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-monitor-overview.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-the-clip-grid.html (F) |
| Global FX Mute (quick before and after) | missing | An effects-bypass flag on `frame`. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-monitor-overview.html (F) |
| Toggle Proxies (Color monitor) | partial | Proxy media. The proxy rows belong to the import area. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-monitor-overview.html (F) |
| Clip Grid (colour-accurate thumbnail of every clip) | partial | One tile per element, rather than one per visual state. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-the-clip-grid.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-the-clip-grid.html (F) |
| Clip Grid filter and sort | partial | Media metadata the document does not hold (codec, frame rate, resolution, date, scene). | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-the-clip-grid.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/using-custom-groups.html (F) |
| Clip Grid metadata (including each clip's colour space) | missing | A source's colour space, read and reported. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-the-clip-grid.html (F) |
| Comparison View: reference and current clip | partial | A single picture with reference and current composed together. A pinned reference. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-comparison-view.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-workflow.html (F) |
| Comparison layouts: horizontal or vertical split, swap, draggable split line | missing | A split output from `frame`. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-comparison-view.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html (F) |
| Graded against ungraded original | partial | An effects-bypass flag on `frame`, so the agent need not edit the file to see "before". | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-comparison-view.html (F) |
| Color Reference Markers | missing | Markers, and a reference type. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-comparison-view.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-monitor-overview.html (F) |
| Apply Inverse Tone Mapping and Gamut Compression (beta) | missing | Inverse tone mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/using-color-mode-with-color-management.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html (F) |
| Out-of-range values preserved through the grade | missing | An unclamped (float) intermediate. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html (F) |
| Export the grade as a LUT (`.cube`) | missing | A tool that bakes an element's colour effects to a `.cube` file. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/save-or-export-look-up-tables.html (F) |
| Lumetri Scopes panel (choose scopes, presets, resizable) | missing | A per-frame colour reading. No ADR refuses one, and ADR-0024's "derivation, no verdict" shape fits it. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/display-lumetri-scopes.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/about-color-grading.html (F) |
| Vectorscope YUV | missing | A per-frame chroma distribution reading. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html (F) |
| Vectorscope HLS | missing | As above. | unconfirmed: neither scopes page lists it (they list Vectorscope YUV only). |
| Histogram | missing | A per-frame histogram reading. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html (F) |
| Parade (RGB, YUV, RGB-White, YUV-White) | missing | A per-channel level reading. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html (F) |
| Waveform (RGB, Luma IRE −20 to 120, YC, YC no chroma) | missing | A luma and chroma level reading, for example min, max and clip counts. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-lumetri-scopes.html (F) |
| Scope bit depth (8-bit, 10-bit, float, HDR 0–10000 nits) | missing | Any scope, and any depth above 8 bits. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/display-lumetri-scopes.html (F) |
| Clamp Signal | missing | Falls with the scopes. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/display-lumetri-scopes.html (F) |
| Video Scopes panel: Vectorscope, Waveform Monitor, RGB Parade, RGB Overlay | missing | As in the Lumetri Scopes rows. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-video-scopes.html (F) |
| Scope graticules (10-bit, Percentage, HDR nits) and layout | missing | Falls with the scopes. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/working-with-video-scopes.html (F) |
| Colour management on or off (Disable Color Management) | partial | A *managed* mode. Also, nothing records or declares the pass-through, so the conversions are whatever the `ffmpeg` build defaults to. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/disabling-color-management.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| Working colour space (Rec.709, Rec.2100 PQ, Rec.2100 HLG, ACEScct) | partial | A choice of working space, and any wide-gamut, HDR or log working space. A linear working space would have to amend ADR-0147. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/customize-color-setup-presets-for-new-or-existing-sequences.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F) |
| Output colour space (Rec.709, Rec.2020, Rec.2100 PQ, Rec.2100 HLG) | missing | Any output space other than the implicit SDR one. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configure-sequence-output-color-space.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| Output Peak Luminance | missing | Falls with HDR. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configuring-sequence-color-management.html (F) |
| Several deliverables from one grade | missing | Falls with the output space. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configuring-sequence-color-management.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| Colour setup preset: Direct 709 (SDR), the default | partial | Rec.709 is never declared or tagged (no colour tags in `encode.rs`), decode conversion is not chosen by Montagent, and nothing is tone-mapped. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configuring-sequence-color-management.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F) |
| Colour setup preset: Direct HDR (HLG) | missing | HLG decode, working space and encode. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configuring-sequence-color-management.html (F) |
| Colour setup preset: Direct PQ (HDR) | missing | PQ decode, working space and encode. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configuring-sequence-color-management.html (F) |
| Colour setup presets: Wide Gamut (Tone Mapped, Minimal Tone Mapping, No Tone Mapping) | missing | A wide-gamut working space and tone mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configuring-sequence-color-management.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F) |
| Custom colour setup (Advanced controls) | missing | Falls with the settings. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/customize-color-setup-presets-for-new-or-existing-sequences.html (F) |
| Sequence colour management settings | missing | A project-level colour declaration. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/lumetri-color.html (F) |
| Log footage handling (camera log input spaces) | missing | A log-to-display transform, either as a named, closed set of camera transforms or as a source field. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| Color Manage Auto Detected Log and Raw Media | missing | Falls with log handling. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| Raw camera media colour handling | missing | Raw debayer and colour transform. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| Raw Source Settings colour space | missing | Raw decode settings. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-using-clip-modify.html (F) |
| Standard input colour spaces (Rec.601, Rec.709, Rec.2020, Rec.2100, P3-D65, DCDM X'Y'Z', sRGB) | missing | A closed list of source spaces. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F) |
| Colour metadata read from containers, and manual tagging | missing | Reading primaries, transfer, matrix and range. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F), https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Data range handling (super-whites and sub-blacks) | missing | Choosing full or limited range per source. | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Override media colour space (per clip) | missing | Overriding a source's primaries, transfer, matrix and range. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-using-clip-modify.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Preserve RGB (per clip) | partial | The per-clip choice (it only means something once conversion exists). | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-using-clip-modify.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Input tone mapping (per clip) | missing | Per-source tone mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F) |
| Exposure before input tone mapping | missing | — | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Output tone mapping (sequence) | missing | Output tone mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F) |
| Automatic tone mapping of HDR and log media in Rec.709 sequences | missing | Falls with input tone mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/tone-mapping-in-premiere.html (F) |
| Tone-mapping algorithms: By Channel, Hue Preservation, Max RGB | missing | A closed enum of algorithms (it would fit ADR-0145's closed-vocabulary invariant). | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F) |
| Tone-mapping parameters: Highlight Saturation and Knee | missing | Falls with tone mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Gamut compression (input and output) | missing | Gamut mapping. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/customize-color-setup-presets-for-new-or-existing-sequences.html (F) |
| Gamut compression methods: Luminance Preserving, Saturation Preserving | missing | Falls with gamut compression. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F) |
| HDR editing (Rec.2100 PQ and HLG timelines, HDR scopes) | missing | Decode above 8 bits, an HDR working space, HDR encode and metadata. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/customize-color-setup-presets-for-new-or-existing-sequences.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/using-color-mode-with-color-management.html (F) |
| High-bit-depth stills for HDR finishing (16-bit PNG) | missing | Decode above 8 bits. | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Colour-space-aware effects (project and sequence setting) | missing | Space-aware colour filters. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Lumetri and colour management (Lumetri enhancements) | missing | As above. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/lumetri-enhancements.html (F) |
| Graphics white (project default, sequence override, export) | missing | — | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/customize-color-setup-presets-for-new-or-existing-sequences.html (F), https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html (F) |
| Display colour management | missing | Untagged output means the viewer guesses. See the export rows. | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/get-started/set-up-directx-hdr-for-windows/set-up-premiere-for-directx-hdr.html (F) |
| Extended Dynamic Range Monitoring | missing | — | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/work-in-color-mode.html (F) |
| Viewer gamma (2.4 Broadcast, 2.2 Web, 1.96 QuickTime) | missing | — | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F), https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/premiere-exports-look-washed-out.html (F) |
| Source Monitor colour management | missing | — | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/manage-source-media-colors-in-program-monitor.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| Transmit to an external display in the output space | missing | — | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/configure-sequence-output-color-space.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Windows DirectX HDR monitoring | missing | — | https://helpx.adobe.com/premiere/desktop/get-started/set-up-directx-hdr-for-windows/set-up-premiere-for-directx-hdr.html (F), https://helpx.adobe.com/premiere/desktop/get-started/set-up-directx-hdr-for-windows/set-up-your-system-for-directx-hdr.html (F), https://helpx.adobe.com/premiere/desktop/get-started/set-up-directx-hdr-for-windows/supported-and-unsupported-features-with-directx-hdr-and-premiere.html (F), https://helpx.adobe.com/premiere/desktop/get-started/set-up-directx-hdr-for-windows/supported-operating-systems-gpus-and-displays-for-directx-hdr-support.html (F) |
| Legacy projects keep their colour | missing | Falls with colour management. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-options.html (F) |
| Lumetri Color (as an effect) | partial | Every missing or refused row above. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/lumetri-color.html (F) |
| Video Limiter (Clip Level, Compression before clipping) | missing | Legal-range clamping and a declared range. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/about-color-correction-effects.html (F) |
| Video Limiter Gamut Warning | missing | An out-of-range reading or overlay. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html (F) |
| Color Replace | partial | Recolouring that keeps the original shading. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/image-control-effects.html (F) |
| Gamma Correction | missing | A `gamma` member (it would need its own ADR). | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/image-control-effects.html (F) |
| Levels | partial | Gamma, colour balance and invert. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/adjust-effects.html (F) |
| ProcAmp | partial | Hue rotation. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/adjust-effects.html (F) |
| Extract | missing | A luminance window. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/adjust-effects.html (F) |
| SDR Conform | missing | — | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/video-effects.html (F) |
| Ultra Key (colour-based keying) | partial | Matte choke and contrast, a matte view, and a colour-correction subsection. The keying area is owned by the effects ticket, and this row is a cross-reference. | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/apply-and-customize-chromakey-using-the-ultra-key-effect.html (F) |
| High-bit-depth (32-bit float) effect processing | missing | Painting above 8 bits. | https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html (F), https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |
| YUV effects | missing | — | https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html (F) |
| Export colour space | missing | Any choice. Even the one fixed output is not declared. | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html (F), https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html (F) |
| Export colour space assigned automatically | missing | Falls with the output space. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/how-color-management-works.html (F) |
| Colour metadata in the file (CICP) | missing | Tagging the output, which is what lets players show the render as painted. | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Render at Maximum Depth / Maximum Bit Depth | missing | Higher-depth paint and encode. | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html (F), https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html (F) |
| HDR export metadata (HDR10, mastering display, content light levels) | missing | — | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html (F), https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html (F) |
| Tone mapping on export | missing | — | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html (F) |

### Titles, graphics and text: 38 partial, 30 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Spell check | missing | A spelling check over text elements. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/check-spelling-using-the-graphics-tab.html |
| Emojis | partial | Colour emoji, which waits on colour-glyph support (see Colour fonts, unverified). A default emoji font: the chain is written per font key. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-emojis.html |
| Justified alignment | missing | Justified lines (space distributed to the block width). No ADR refuses it. | — |
| Kerning | partial | Manual kerning of one letter pair. `letter_spacing` cannot be written on a run (schema). | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Baseline Shift | partial | A plain per-run baseline offset. Every run on a line sits on one shared baseline (ADR-0029). | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Tab Width | missing | Tab stops and tab width. Columns are separate elements placed by the agent. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Faux Italics | missing | Synthetic slant. ADR-0007's refusal names bold and weight only, so this is not refused by name. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| All Caps | partial | A case transform that keeps the source casing in the file. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Small Caps | partial | The font's real small caps (`smcp`), and any OpenType feature switch: only ligature handling is fixed, by rule (ADR-0151, ADR-0153). | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Superscript and Subscript | partial | A raised or lowered run, and the `sups`/`subs` features. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Underline | partial | An underline that is a text or run attribute and follows the glyphs when the text changes. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Strikethrough | missing | A strikethrough attribute. | — |
| Tsume | missing | Tsume. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Ligatures on or off | partial | A switch to turn optional ligatures on or off. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Hindi digits | partial | A display switch that keeps the source digits in the file. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Default paragraph direction | partial | A settable base direction, for a line whose first strong character disagrees with the paragraph. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Vertical text | missing | Vertical text. | — |
| Variable fonts | missing | Variable-axis values. | — |
| Colour fonts | missing | Colour glyphs. Whether the painter draws them today is undetermined; it needs a measured test before it can count. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-color-fonts.html |
| Missing font replacement | partial | Substitution for a missing font file: `validate` makes that an error instead (ADR-0007). | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Font favourites and filtering | missing | Font favourites and filters (GUI). | — |
| Linear and radial gradient | partial | Midpoint controls between stops, and opacity stops placed apart from colour stops (route for both: extra stops). A gradient on one run or highlight is refused (ADR-0149, ADR-0146). | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html |
| Gradient shadow | missing | A gradient `shadow.color`. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/add-gradients.html |
| Inner and Centre stroke | partial | A choosable stroke position on one element. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html |
| Multiple strokes | partial | Stacked strokes on one element. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html |
| Stroke line join and miter limit | partial | A join choice on text strokes (round, fixed: ADR-0158 §1) and on `rect`/`ellipse` (a schema error there, ADR-0158 §1). | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Stroke line cap | partial | A cap on `rect`/`ellipse` dashes (fixed butt) and on text (takes none), both by ADR-0158 §1. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Text background | partial | A background that follows the text's extent by itself. The agent sizes the `rect` from `measure` or `query`. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Background per line or whole block | partial | A background mode on the text element. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Shadow | partial | Shadow size (spread), if Premiere has it (unconfirmed). Angle and distance are written as `dx`/`dy`. | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-a-shape.html |
| Mask with Shape | partial | One shape masking several layers at once; ADR-0150 refuses that by taking no matte from another element. Route: the same `mask` written on each element. | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-reveal-animations-using-masking-techniques.html |
| Pen mask that limits an effect | partial | An effect limited to the masked area (an effect-scoped mask; #885 counts it). | https://helpx.adobe.com/premiere/desktop/add-text-images/draw-objects/create-free-form-shapes-for-masking.html |
| Per-corner rectangle radius | partial | A radius per corner (route: a closed `path`). | — |
| Transform a layer | partial | An arbitrary anchor point: the pivot is one of nine `origin` keywords (ADR-0013). | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/animate-layers-using-properties-panel.html |
| Align objects | partial | An align command, to the frame, as a group or to a key object. The agent computes each literal. | https://helpx.adobe.com/premiere/desktop/add-text-images/align-and-distribute-objects/align-objects.html |
| Distribute objects | partial | A distribute command. | https://helpx.adobe.com/premiere/desktop/add-text-images/align-and-distribute-objects/distribute-objects.html |
| Edit several layers at once | partial | A relative edit of a spatial or style value across several elements: the agent writes each new literal. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/about-properties-panel.html |
| Guides and snapping | partial | Snapping to another element's edge. The agent computes it from `query`. | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html |
| A graphic holding several layers | partial | One clip that trims, moves and fades as a unit. A nest with its own transform has precedent (ADR-0166) and is not built; "parenting, groups and cameras" is listed as not yet possible (`montagent` skill, #499). | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-a-clip-layer.html |
| Group layers | partial | A group as the scope of a mask: a matte from another element is refused (ADR-0150). | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/group-text-and-graphic-layers.html |
| After Effects compositions | partial | A live link that re-renders on change. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/different-ways-to-work-with-graphics.html |
| Render and Replace a graphic | partial | Baking a Montagent graphic itself to alpha footage: output is H.264 MP4 without alpha (#892). | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/tips-for-authoring-motion-graphics-templates.html |
| Markers in the Effect Controls panel | missing | Markers. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-markers-in-the-effect-controls-panel.html |
| Responsive Design: pin to another layer | missing | Pinning or parenting, and any size that follows another element. Any design has to pass ADR-0145's literal-values test. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html |
| Responsive Design: time (intro and outro) | partial | Keyframes that follow an element's `end` at render, and middle keyframes rescaled to fit. Times are absolute integer milliseconds (ADR-0005). | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/preserve-intro-outro-animations-while-creating-responsive-design-graphics.html |
| Rolling credits | partial | Speed derived from the duration; no credits recipe in `montagent-motion`. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html |
| Roll options | partial | Named options. The agent computes the off-screen positions and the eased segments. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html |
| Soften a roll | partial | A soften option. The page does not say what Soften does, so whether motion blur matches it is unknown. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-credit-rolls.html |
| Crawl (horizontal scroll) | partial | Named crawl options. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html |
| Create a text style | missing | Named style objects. ADR-0002's case against indirection in the file bears on any design. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html |
| Apply a style to many layers | partial | A single edit that restyles them all. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html |
| Linked styles (titles) and track styles (captions) | missing | Linked instances. Track styles for captions are #888's. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-linked-and-track-styles.html |
| Apply linked and track styles | missing | Applying a linked style, and a usage count. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-linked-and-track-styles.html |
| Redefine a style | missing | Propagation of a style edit. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/redefine-linked-and-track-styles.html |
| Save a style for other projects | missing | A style library shared across projects. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-linked-and-track-styles.html |
| Style browser parameters | missing | A style browser. | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/style-parameters-when-applying-from-style-browser.html |
| User defaults for new graphics | missing | User-set defaults. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html |
| Use a Motion Graphics template (`.mogrt`) | missing | A live template on the timeline. An After Effects-authored template carries an After Effects project and possibly expressions, which ADR-0145's closed-vocabulary test calls programs. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html |
| Template editable properties | partial | Parameters inside the file. A control that drives values elsewhere is "a reference to another value" under ADR-0145. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/customize-motion-graphics-templates.html |
| Media Replacement in templates | missing | Replaceable media slots in a template. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/media-replacement-in-motion-graphics-templates.html |
| Data-driven templates (CSV/TSV) | missing | Data-driven templates. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/use-data-driven-motion-graphics-templates.html |
| Install templates | missing | Template installation. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/install-motion-graphics-templates.html |
| Browse, search, sort, favourite and tag templates | missing | A template library and browser. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/browse-and-sort-motion-graphics-templates.html |
| Organise and share templates | missing | Template libraries and shared folders. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/organize-motion-graphics-templates.html |
| Templates from Adobe Stock | missing | A template marketplace. | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/use-motion-graphics-templates-from-adobe-stock.html |
| Export a graphic as a Motion Graphics template | missing | A packaged, reusable graphic. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/export-graphic-as-a-motion-graphics-template.html |
| Legacy titles, upgraded to Source Graphics | missing | Reading a Legacy Title or a Premiere graphic. Interchange is #892's. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-source-graphics.html |
| Source Graphic instances | missing | Linked instances. Any design has to pass ADR-0145's literal-values test. | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-source-graphics.html |

### Captions, subtitles and transcripts: 15 partial, 32 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Edit speaker names | missing | A speaker label; today only a free-text `group` or the element `id` can carry it, with no checks | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-speaker-names-in-transcription.html (F) |
| Transcript as a project object, correctable | missing | Any transcript in the document; Montagent only holds the frozen result (text runs, `highlight` windows) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-transcripts-using-text-based-editing.html (F) |
| Sequence transcript kept in step with the timeline | missing | A transcript derived from the timeline | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html · https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-sequences-using-text-based-editing.html (F) |
| Spell check in the transcript | missing | A spelling check (on transcripts or caption text) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-transcripts-using-text-based-editing.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/check-spelling-using-the-graphics-tab.html (F) |
| Find and replace in the transcript / captions | partial | No transcript to search; no replace verb (by design: write tools take complete elements, ADR-0011, ADR-0051) — replace is the agent's file edit | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/find-and-replace-text-in-transcription.html (F) |
| Search transcripts across the project | missing | A transcript to search | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/media-intelligence-and-search-panel.html (F) |
| Text-Based Editing: rearrange and cut the sequence through its transcript | partial | The link from text to source time; any tool that turns a deleted or moved word span into trims. The transcript itself is refused (ADR-0051) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-sequences-using-text-based-editing.html · https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html (F) |
| Text-Based Editing: add clips from a transcript selection | partial | Transcript selection as an in/out source | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/add-clips-to-the-timeline-using-text-based-editing.html (F) |
| Paper Edit (26.5) | partial | Selections made in a transcript; the transcript itself (ADR-0051) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/create-a-sequence-with-paper-edit.html · https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Bulk delete matching text | partial | Text-to-time link; a bulk cut | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/detect-and-delete-pauses-in-transcripts.html (F) |
| Pause detection and bulk delete | partial | A pause list as an editing input, and the bulk cut; the skill only reports and re-times captions, it never removes media | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/detect-and-delete-pauses-in-transcripts.html (F) |
| Bulk mute or bleep | partial | A word list matched against a transcript; a generated tone (every `audio` element names a file) | https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/clean-audio-with-bulk-mute-or-bleep.html · https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Markers in the Transcript tab | missing | Markers; a transcript | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| Export transcript | missing | Any transcript output | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-transcripts.html (F) |
| Text-Based Editing in 18 interface languages | missing | A localised surface (see `workflow.md`) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html (F) |
| Caption track as its own track type | partial | A track kind or format: a Montagent caption is pixels only, so it carries no caption-stream semantics (format, stream, language) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/about-captions.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| CEA-608 closed captions | missing | 608 encoding (streams, display type, its character set) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html · https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/known-and-fixed-issues.html (F) |
| CEA-708 closed captions | missing | 708 encoding | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| Wide-aspect CEA-708 import (26.0) | missing | — | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html · https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Teletext | missing | Teletext model | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-captions.html (F) |
| EBU Subtitle | missing | EBU subtitle model | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| Australian OP-47 | missing | OP-47 model | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| SubRip (.srt) import | missing | An .srt → `text` elements conversion (a skill script would fit ADR-0051's pattern) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/import-caption-file-from-third-party-service.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html · https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| Scenarist (.scc) import/export | missing | Reader and writer | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| MacCaption VANC (.mcc) import/export | missing | Reader and writer | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| W3C TTML/DFXP, SMPTE-TT, EBU-TT (.xml) import/export | missing | Reader and writer | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| EBU N19 (.stl) import/export | missing | Reader and writer | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html (F) |
| WebVTT | missing | Neither tool has it; listed because the ticket names it | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html (F) |
| Decode embedded captions from MOV / MXF | missing | Caption-stream decoding | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html · https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Captions and Graphics workspace | missing | Any interactive panel (see `workflow.md`) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/about-captions.html (F) |
| Inner, centre and multiple strokes | partial | Inner and centre placement; more than one stroke | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-styles-for-captions.html (F) |
| Caption zone | partial | A named frame region a caption is placed in; the mechanism needs a follow-up read | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html (F) |
| Responsive Design – Position | missing | Pinning to the frame or another element (see `titles.md`) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-styles-for-captions.html (F) |
| Track style (one style for a whole caption track) | missing | A shared style; today a script (e.g. `captions.py` spec) writes the same literals onto every element | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-styles-for-captions.html · https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-linked-and-track-styles.html · https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html (F) |
| Redefine a track style | missing | A one-step change to every caption of a track | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/redefine-linked-and-track-styles.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html (F) |
| Style usage indicators | missing | — | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-linked-and-track-styles.html (F) |
| Linked text styles, local styles, reuse across projects | missing | — | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-styles-for-captions.html · https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-linked-and-track-styles.html (F) |
| Default subtitle and closed-caption fonts (26.0) | missing | A default font; every text names its font | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html · https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Create sidecar file | missing | Any second output; the text, timing and `caption` flag needed for an .srt are all in the document | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html (F) |
| Export selected captions from the Text panel | missing | A caption-only text output | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-captions.html (F) |
| Embed in output file | missing | A caption stream; MOV/MXF containers | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/about-captions.html (F) |
| Choose which caption track exports; hide/isolate tracks | missing | Track visibility; a render-time track selection | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html (F) |
| Several caption languages in one project | partial | A language tag, and a way to deliver one language at a time (track visibility or one stream per language) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html (F) |
| Single-word captions (26.3) | partial | `captions.py` pages by `width`, `lines` and `gap` only: no words-per-page cap | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-single-word-captions.html · https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| Split a caption | partial | A split verb that divides text and time (by design ADR-0011 adds no field-level edit tools) | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html · https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/known-and-fixed-issues.html (F) |
| Merge captions | partial | A merge verb | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| Caption timing matches the speech | partial | A check of caption or highlight timing against the audio on disk (ADR-0051, ADR-0134 leave it open; #565) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-captions.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-single-word-captions.html (F) |

### Audio: 39 partial, 87 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Amplify | partial | A dB gain, per-channel gain, and a position among the effects: `volume` is linear, whole-element, and runs after `audio_effects`. | [lib][lib] (F) |
| Channel Mixer | missing | Any channel-mix matrix; per-channel polarity. | [lib][lib] (F) |
| Channel Volume | missing | The member. | [lib][lib] (F) |
| DeEsser | missing | A band-limited (side-chain filtered) compressor or a `de_ess` member. | [lib][lib] (F) |
| Dynamics | partial | The gate (`noise_gate`, ADR-0183, proposed, not in S) and the downward expander (no ADR). | [lib][lib] (F) |
| Dynamics Processing | partial | A drawn multi-point transfer curve, peak detection (ADR-0180 refuses `detection: peak` for its member), look-ahead, band limiting. | [lib][lib] (F) |
| Hard Limiter | partial | Input boost (place a `volume` or `makeup_db` before it instead), a look-ahead parameter, channel-link choice. | [lib][lib] (F) |
| Multiband Compressor | missing | The member, and band splitting. | [lib][lib] (F) |
| Tube-modeled Compressor | partial | The tube/saturation colouring. | [lib][lib] (F) |
| Delay ‡ | missing | The built member. | [lib][lib] (F) |
| Analog Delay | missing | Built echo, plus saturation and spread. | [lib][lib] (F) |
| Multitap Delay | missing | Parallel taps. | [lib][lib] (F) |
| Bandpass ‡ | partial | A single band member; slopes are only 12/24/48 dB/oct. | [lib][lib] (F) |
| FFT Filter | missing | An arbitrary drawn response. | [lib][lib] (F) |
| Graphic Equalizer (10, 20, 30 bands) | partial | 10/20/30 bands: `E-EQ-STACK-CAP` refuses more than 8 enabled EQ members on one element. | [lib][lib] (F) |
| Notch Filter | partial | Very narrow notches: `q` is capped at 10 and `gain_db` at −24; no notch member. | [lib][lib] (F) |
| Notch Filter presets (added) | missing | Named presets. | [lib][lib] (F) |
| Parametric Equalizer ‡ | partial | Nine sections against a cap of 8; Q only 0.1..10 (the page's narrow notches need about 100); no constant-width mode. | [lib][lib] (F) |
| Scientific Filter | partial | The other three families, arbitrary order, and band-stop as one filter. | [lib][lib] (F) |
| Simple Notch Filter | partial | Narrow notches beyond `q` 10. | unconfirmed: not in the fetched library or any census page ([lib][lib] lists no such effect) |
| Chorus/Flanger | missing | The member. | [lib][lib] (F) |
| Flanger | missing | The member. | [lib][lib] (F) |
| Phaser | missing | The member. | [lib][lib] (F) |
| Automatic Click Remover | missing | The member. | [lib][lib] (F) |
| DeHummer | partial | A harmonic-series member; narrow enough notches (`q` ≤ 10); the 8-member cap limits harmonics. | [lib][lib] (F) |
| DeNoise | missing | The built member. | [lib][lib] (F) |
| Studio Reverb | missing | The member and its ADR. | [lib][lib] (F) |
| Balance (effect) | missing | The built field. | unconfirmed: no such effect on [lib][lib] |
| Binauralizer – Ambisonics | missing | Ambisonic channel formats and the decode. | [ambi][ambi] (F) |
| Distortion | missing | The member. | [lib][lib] (F) |
| Fill Left with Right ‡ | missing | The built field. | [lib][lib] (F) |
| Fill Right with Left ‡ | missing | The built field. | [lib][lib] (F) |
| Guitar Suite | missing | Distortion and amp/cabinet modelling. | [lib][lib] (F) |
| Invert | missing | The member. | [lib][lib] (F) |
| Loudness Meter | partial | Momentary, short-term and range readings over time, per-track metering, platform presets (a target is a literal number). | [lmeter][lmeter], [lm-apply][lm-apply], [lm-measure][lm-measure] (F) |
| Loudness Meter settings (added) | partial | A noise-floor threshold, display scale and units, named presets. | [lm-settings][lm-settings] (F) |
| Loudness Radar ‡ | partial | A per-clip or per-track loudness reading. | [lib][lib] (F) |
| Mastering | partial | Exciter, widener, reverb; and any effect on the mix itself, which ADR-0172 refuses (see section K). | [lib][lib] (F) |
| Panner – Ambisonics | missing | Ambisonics. | [ambi-restage][ambi-restage] (F) |
| Pitch Shifter ‡ | missing | The member and its ADR. | [lib][lib] (F) |
| Stereo Expander ‡ | missing | The member. | [lib][lib] (F) |
| Swap Channels | missing | The built field. | [lib][lib] (F) |
| Vocal Enhancer | missing | The member (and de-ess, plosive handling, mic modelling). | [lib][lib] (F) |
| Voice-changer presets | missing | Pitch Shifter first. | unconfirmed: no voice-changer preset on [lib][lib] or any census page |
| Panner (fixed effect) | missing | The built field and keyframes. | [props][props], [acm-auto][acm-auto], [pan-track][pan-track] (F) |
| Effect channel layout (Channelize effects) (added) | missing | Channel maps on effects. | [channelize][channelize] (F) |
| Obsolete audio effects ‡ | missing | A replace-on-open path (not a format feature). | [lib][lib] (F) |
| Fade handles | missing | The interactive editor (Montagent's tools read, check and render; ADR-0011). | [fade][fade], [xfade-handles][xfade-handles], [relnotes][relnotes] (F) |
| Default audio transition and its duration | missing | An apply-default command (an editor convenience, not a format feature). | [trans-default][trans-default], [tl-prefs][tl-prefs], [shortcuts][shortcuts] (F) |
| Multi Transitions (26.0) | missing | A batch command or skill script. | [multitrans][multitrans], [whatsnew][whatsnew] (F) |
| Repair > Reduce Noise | missing | The built member. | [repair][repair] (F) |
| Repair > DeHum | partial | As DeHummer, section E. | [repair][repair] (F) |
| Repair > DeEss | missing | A de-ess member. | [repair][repair] (F) |
| Clarity > Dynamics | partial | A one-dial macro; every compressor key must be written. | [clarity][clarity] (F) |
| Clarity > EQ | partial | Named presets and an amount dial. | [clarity][clarity] (F) |
| Clarity > Vocal Enhancer ‡ | missing | The member. | [clarity][clarity] (F) |
| Creative > Reverb ‡ | missing | The member. | [reverb][reverb] (F) |
| Music/SFX > Stereo Width | missing | The member. | unconfirmed: no census page names this control ([esp][esp] lists no Stereo Width) |
| SFX > Pan | missing | The built field. | unconfirmed: no census page names this control |
| Music > Duration (Remix) | missing | A skill script that analyses the track and writes cuts and crossfades, as `beat_grid.py` writes a placement. | [remix][remix], [remix-about][remix-about], [remix-tips][remix-tips] (F) |
| Audio Gain: Set Gain to / Adjust Gain by ‡ | partial | A clip gain in dB separate from `volume`. | [gain][gain] (F) |
| Peak Amplitude readout (added) | missing | A per-source or per-element peak reading. | [gain][gain] (F) |
| Normalize Max Peak to ‡ | missing | A peak target. | [gain][gain] (F) |
| Normalize All Peaks to ‡ | missing | A peak target. | [gain][gain] (F) |
| Normalize Mix Track | partial | A peak (not loudness) target on the mix. | unconfirmed: no census page has it |
| Clip effect-parameter keyframes | missing | Keyframable effect parameters. | [trackvol][trackvol], [ecp][ecp] (F) |
| Clip pan keyframes | missing | Built and animatable `pan`. | [props][props], [acm-auto][acm-auto] (F) |
| Automation keyframe thinning | missing | (Only meaningful with live automation recording.) | [kfopt][kfopt], [prefs][prefs] (F) |
| Large Volume Adjustment ‡ | missing | The interactive editor. | [prefs][prefs] (F) |
| Speed with Maintain Audio Pitch | partial | The "off" state: a speed change that also shifts pitch. | [speed][speed] (F) |
| 5.1 Panner tray (added) | missing | A 5.1 bus. | [panctl][panctl] (F) |
| Pre-fader / post-fader effect placement ‡ | partial | A post-fader member position (ADR-0169 chose post-fader `volume` deliberately). | [prepost][prepost] (F) |
| Audio Clip Mixer | partial | Per-element `pan` (ADR-0184 not built); the live mixing surface. | [concepts][concepts], [atm][atm] (F) |
| Audio Clip Mixer: automation (Latch, Touch) | missing | Live recording of keyframes (no playback surface). | [acm-auto][acm-auto] (F) |
| Global Mute (26.2) (added) | missing | A monitoring control (not a format feature). | [gmute][gmute], [whatsnew][whatsnew] (F) |
| Mix (master) track volume and loudness | partial | A static or keyframed mix gain that is not a loudness target. | [atm][atm] (F) |
| Pan a nested sequence as one element (added) | missing | Pan, and pan on a nested unit. | [panctl][panctl] (F) |
| Audio track types: Standard, Mono, Adaptive, 5.1 | missing | Channel formats (the per-track part would also meet the track-level refusal). | [concepts][concepts] (F) |
| Direct output / track output channel assignment | missing | Multichannel master and routing. | [trackout][trackout] (F) |
| Multiple stereo assignments to a multichannel master | missing | Multichannel master. | [multistereo][multistereo] (F) |
| Modify > Audio Channels (source channel mapping) | missing | Built field; mapping of more than two source channels. | [chmap][chmap], [chmap-concept][chmap-concept] (F) |
| Default Audio Tracks on import (added) | missing | Channel formats. | [map-import][map-import], [tl-prefs][tl-prefs] (F) |
| Hide audio channel labels (added) | missing | The interactive editor. | [chmap][chmap], [relnotes][relnotes] (F) |
| Multichannel master and 5.1 mixing ‡ | missing | Any non-stereo bus or output. | [trackout][trackout], [panctl][panctl] (F) |
| 5.1 Mixdown Type preference ‡ | missing | A 5.1 bus. | [prefs][prefs] (F) |
| Breakout to Mono ‡ | missing | The built field; per-channel access beyond stereo. | [breakout][breakout] (F) |
| Sequence sample rate | partial | A chosen rate (44.1, 96 kHz …); sample-unit time display. | [seqset][seqset] (F) |
| 32-bit float processing (added) | partial | A stated sample format for the rest of the graph and a chosen rate. | [sr][sr], [conform][conform] (F) |
| Match Source audio (26.5) | missing | A source-matched bus. | [whatsnew][whatsnew], [relnotes][relnotes] (F) |
| Synchronize clips (added) | partial | Source timecode and clip markers (see the editing area); a sync command. | [sync-tl][sync-tl] (F) |
| Synchronize by audio waveform | missing | A skill script. | [sync-tl][sync-tl], [merge-limits][merge-limits] (F) |
| Merge Clips | partial | A linked unit that moves as one (no linking concept, by ADR-0001). | [merge][merge], [merge-tl][merge-tl], [sync][sync], [merge-limits][merge-limits] (F) |
| Trim merged clips (added) | missing | A linked trim. | [merge-edit][merge-edit] (F) |
| Merged clip metadata (added) | missing | Metadata. | [merge-meta][merge-meta] (F) |
| Audio timecode from a primary clip; Remove Audio from AV Clip (added) | partial | Source timecode. | [merge-tc][merge-tc] (F) |
| Link / unlink audio and video ‡ | partial | Linking of two elements (a `group` shares no timing; ADR-0001). | [link][link] (F) |
| Edit a multi-clip link in the Source Monitor (added) | missing | Linking. | [mclink][mclink] (F) |
| Bulk mute or bleep (added) | partial | Word detection (transcription is model-driven; see the captions area), a censored-word list, and any tone source for a bleep. | [bleep][bleep], [whatsnew][whatsnew] (F) |
| Timeline audio waveforms (peak files) | missing | A waveform reading (image or numbers). | [conform][conform], [src-mon][src-mon], [prefs][prefs] (F) |
| Dynamic Audio Waveforms (26.0) | missing | A post-volume waveform reading. | [dynwave][dynwave], [whatsnew][whatsnew] (F) |
| Live waveform editing (26.0) (added) | missing | The interactive editor. | [livewave][livewave], [whatsnew][whatsnew] (F) |
| Live track resize (added) | missing | The interactive editor. | [livewave][livewave], [relnotes][relnotes] (F) |
| Rectified waveforms | missing | — | unconfirmed: no census page has it |
| Source Monitor audio and video waveforms (26.2) | missing | — | [src-wave][src-wave], [whatsnew][whatsnew] (F) |
| Source Monitor audio inspection (added) | missing | A waveform or level reading per source. | [src-mon][src-mon] (F) |
| Sum multichannel outputs to mono in Source Monitor (added) | missing | — | [prefs][prefs] (F) |
| Audio scrubbing | partial | Interactive scrubbing and shuttling. | [prefs][prefs], [src-mon][src-mon] (F) |
| Audio Meters ‡ | partial | A level-over-time reading per element or track. | [meters][meters] (F) |
| Audio hardware preferences | missing | — | [hw][hw] (F) |
| Map output to hardware channels (added) | missing | — | [hwmap][hwmap], [hw][hw] (F) |
| Monitor Ambisonics audio | missing | Ambisonics. | [ambi][ambi] (F) |
| Properties panel: audio controls ‡ | partial | Pan (ADR-0184 not built). | [props][props] (F) |
| Properties panel: multi-clip editing (added) | missing | A batch command or skill script. | [props][props] (F) |
| Voice-over Record button on a track ‡ | missing | Capture (outside a file-in, file-out editor). | [vo][vo] (F) |
| Record through the Audio Track Mixer ‡ | missing | As above. | [rec-atm][rec-atm] (F) |
| Recording setup (input device, mono/stereo jacks) | missing | As above. | [rec][rec], [hw][hw] (F) |
| Mute input during timeline recording (added) | missing | — | [mute-input][mute-input], [prefs][prefs] (F) |
| Audio codec choice | partial | Any other codec. | [export-ref][export-ref] (F) |
| Audio bitrate | partial | A chosen bitrate. | [export-ref][export-ref] (F) |
| Export sample rate | partial | A chosen rate. | [export-ref][export-ref], [export][export] (F) |
| Export channels and downmix | partial | Mono and 5.1 output. | [export-ref][export-ref] (F) |
| Audio-only export | missing | An audio-only deliverable. | [export][export] (F) |
| Separate audio tracks / stems in export | missing | Stems or multitrack audio output. | unconfirmed for Premiere's own export |
| Edit in Adobe Audition | missing | A round-trip command. | [audition][audition] (F) |
| Remix in Audition to match duration (added) | missing | A round trip, or the Remix skill. | [remix-au][remix-au] (F) |
| Adobe Stock audio in the Essential Sound panel (added) | missing | A stock browser (outside the format). | [stock][stock], [props][props] (F) |

### Import, formats and media handling: 40 partial, 121 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| **Formats ›** AV1 | partial | ADR-0115's floor (ffmpeg 7.1 + libx264) does not require a software AV1 decoder (libdav1d/libaom), so it depends on the user's build; no repo test | unconfirmed (nearest: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html, which omits it) |
| **Formats ›** Panasonic P2 (AVC-Intra, AVC Ultra, AVC-LongG, DVCPRO in MXF OP1b / OP-Atom) | partial | Nothing joins OP-Atom picture and audio files into one clip: the author must place one `video` plus one `audio` element per file and align them by hand | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Formats ›** Animated GIF | partial | `Raster::decode` keeps the first frame only; a GIF as a `video` source would go through ffmpeg's `gif` decoder, but no doc, skill or test establishes that route | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-images-as-image-sequence.html (F) |
| **Formats ›** Phantom Cine | partial | Unverified: no test; raw-Bayer Cine depends on ffmpeg's own debayer and has no raw controls | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Formats ›** Adobe Sound Document (ASND) | missing | Any ASND decode (export WAV from Audition) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Formats ›** Maximum frame sizes | partial | No stated or checked maximum source or canvas size | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Formats ›** Growing files (MXF AVC-Intra, IMX, XDCAM HD RDD9, OP1b) | partial | No refresh while a verb runs; elements state their own source ranges, so nothing extends with the file | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Formats ›** Data range and CICP colour metadata (DNxHR/DNxHD MXF, PNG) | missing | Range and colour-tag handling (overlaps #886) | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **Camera raw ›** RED R3D (incl. R3D NE) | missing | Any R3D decode; must be transcoded outside Montagent | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Camera raw ›** ARRIRAW (MXF, ALEXA 35, ALEXA LF, ARRIRAW HDE) | missing | Any ARRIRAW decode (ALEXA 35 ProRes files are covered by the ProRes row) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Camera raw ›** ARRICORE (ALEXA 35 Xtreme) | missing | Any ARRICORE decode | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **Camera raw ›** Blackmagic RAW (.braw) | missing | Any BRAW decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/blackmagic-raw-support.html (F) |
| **Camera raw ›** Apple ProRes RAW | missing | Any ProRes RAW decode and its raw controls | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Camera raw ›** Canon Cinema RAW Light (.crm) / Canon RAW | missing | Any CRM decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Camera raw ›** Sony RAW / X-OCN (F65, F55, F5, FS700, VENICE, BURANO, X-OCN LT) | missing | Any Sony RAW decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Camera raw ›** Nikon N-RAW | missing | Any N-RAW decode | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **Camera raw ›** CinemaDNG | missing | Any CinemaDNG decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Camera raw ›** Raw Source Settings (colour space, gamma curve, debayer, metadata parameters) | missing | The whole raw-parameter surface | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Camera raw ›** Camera acquisition metadata (Panasonic: ISO, white balance, gamma, gamut; Sony camera metadata) | missing | Reading camera metadata | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Audio ›** Conformed-file reuse | missing | (Only meaningful with a conform cache) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/audio-conforming-specifications.html (F) |
| **Audio ›** Waveform peak files (PEK) | missing | Peak data | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/audio-sample-rates-in-premiere.html (F) |
| **Audio ›** Audio channel interpretation (Modify > Audio Channels matrix) | partial | No channel choice or remap per source; ADR-0184's `channels` routing field is `proposed` and absent from the v0.2.0 schema; no 5.1 bus | https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/change-source-audio-channel-mapping.html (F) |
| **Audio ›** Default channel mapping on import | missing | Any per-format default | https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/map-source-audio-channels-on-import.html (F) |
| **Audio ›** Multiple audio streams in one file | partial | No key selects a stream by index; `probe`'s `Audio` reports one stream | unconfirmed (nearest: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/change-source-audio-channel-mapping.html) |
| **Audio ›** Match Source audio (channels and sample rate) | missing | Matching, and a warning when a layout is lost | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **Stills ›** 16-bit PNG | partial | Unverified: no test establishes that more than 8 bits survive to the composite | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| **Stills ›** BMP / DIB / RLE / ICO | partial | Unverified: no test or doc establishes that the pinned prebuilt decodes BMP/ICO | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Stills ›** TIFF | missing | Decode; `validate` passes it (ffprobe reads it) and `render` refuses at `E-NOT-PAINTED-UNDECODABLE` | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Stills ›** Targa (TGA, ICB, VDA, VST) | missing | Decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Stills ›** Cineon / DPX | missing | Decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Stills ›** HEIF / HEIC | missing | Decode | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Stills ›** WebP | missing | Decode | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| **Stills ›** OpenEXR (.exr, .mxr, .sxr) | missing | Decode, float pixels, channel selection | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Stills ›** Premiere legacy titles (PTL, PRTL) | missing | Title import (overlaps #887) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Layered & vector ›** Photoshop PSD, merged layers | missing | Decode; workaround is exporting PNG | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html (F) |
| **Layered & vector ›** Photoshop PSD, individual layers | missing | Layer extraction (the bin half is refused with the catalogue) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html (F) |
| **Layered & vector ›** Photoshop PSD as a sequence | missing | PSD layer decode and layout into elements | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html (F) |
| **Layered & vector ›** Photoshop timeline animations and video | missing | Animated PSD import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html (F) |
| **Layered & vector ›** Edit Original | missing | A launch-the-source command | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html (F) |
| **Project & interchange files ›** Premiere project (PRPROJ), whole or selected sequences | partial | No import verb; source paths are relative to each file's own folder (ADR-0053), so copied elements can break; no rate conversion | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-selected-premiere-sequences.html (F) |
| **Project & interchange files ›** Project shortcuts | missing | Linked project items | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-projects-as-project-shortcuts.html (F) |
| **Project & interchange files ›** Final Cut Pro XML (FCP 7 XML) | missing | XML import (overlaps #892) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/migrate-from-final-cut-pro-x.html (F) |
| **Project & interchange files ›** Final Cut Pro X translation rules | missing | All of it | https://helpx.adobe.com/premiere/desktop/organize-media/transfer-files/supported-elements-for-final-cut-pro-x.html (F) |
| **Project & interchange files ›** AAF (Avid Media Composer) | missing | AAF import and a translation report | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-avid-media-composer-project-files.html (F) |
| **Project & interchange files ›** EDL (CMX3600) | missing | EDL import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Project & interchange files ›** OpenTimelineIO (OTIO) | missing | OTIO import | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| **Project & interchange files ›** Batch lists (CSV, PBL, TXT, TAB) | missing | Batch-list import | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Project & interchange files ›** Premiere Elements project (PREL) | missing | PREL conversion | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-a-premiere-elements-project.html (F) |
| **Project & interchange files ›** Character Animator project (CHPROJ) | missing | CHPROJ import; nearest route is a rendered MOV | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Project & interchange files ›** Premiere on iPhone projects | missing | Mobile-project import | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **Project & interchange files ›** Preset import and export | missing | Presets (overlaps #892) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-and-export-presets.html (F) |
| **Captions ›** SRT import | missing | SRT parsing (overlaps #888) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/import-caption-file-from-third-party-service.html (F) |
| **Captions ›** Broadcast caption files (SCC, MCC, EBU STL) | missing | Parsing (overlaps #888) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Captions ›** Timed-text XML (DFXP, W3C/SMPTE/EBU Timed Text) | missing | Parsing (overlaps #888) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Source properties ›** HDR sources (HLG, PQ) | missing | HDR detection, tone mapping, HDR working space (overlaps #886) | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/tone-mapping-in-premiere.html (F) |
| **Source properties ›** High bit depth (10/12-bit, 16-bit, float) | partial | Precision is lost: every frame is RGBA8 (`-pix_fmt rgba`) | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **Source properties ›** Log and raw colour-space detection | missing | Detection and conversion (overlaps #886) | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| **Source properties ›** Source timecode | missing | Timecode reading, display or addressing | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/choose-timecode-display-format.html (F) |
| **Source properties ›** Embedded captions (608/708) | missing | Extraction (overlaps captions ticket #888) | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| **Interpret footage ›** Frame rate ("Assume this frame rate") | partial | No per-source declared rate: a conform is spelled as a `speed` ratio on every element using the file; project `fps` is an integer (schema), so 23.976/29.97 projects cannot be stated | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/change-the-frame-rate-of-a-clip.html (F) |
| **Interpret footage ›** Pixel aspect ratio | partial | ADR-0023's `par` override key is accepted but absent from the v0.2.0 schema; `decode.rs` does not apply PAR (open as #274) | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/pixel-aspect-ratio.html (F) |
| **Interpret footage ›** Field order / deinterlace | missing | Field detection, override and deinterlacing | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/change-the-field-order-of-a-clip.html (F) |
| **Interpret footage ›** Remove pulldown (3:2) | missing | Pulldown removal | unconfirmed (nearest: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html) |
| **Interpret footage ›** Alpha channel and matte options | partial | No key to ignore, invert or declare premultiplied alpha on a source | https://helpx.adobe.com/premiere/desktop/edit-projects/modify-clip-properties/modifying-clip-properties-with-interpret-footage.html (F) |
| **Interpret footage ›** Override media colour space, Preserve RGB | missing | Override (overlaps #886) | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-using-clip-modify.html (F) |
| **Interpret footage ›** Modify > Timecode | missing | Timecode override | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/set-clip-timecode-manually.html (F) |
| **Import & ingest ›** Import mode (New Project Import desktop) | missing | The import workspace; its copy, sequence and transcription halves are their own rows | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/create-new-project.html (F) |
| **Import & ingest ›** New sequence from clip (match settings) | partial | No derive-from-source; integer `fps` cannot match a 29.97 source | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/create-new-project.html (F) |
| **Import & ingest ›** Automatic transcription on import | missing | (See #888 and AI #893) | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/create-new-project.html (F) |
| **Import & ingest ›** Ingest settings: Copy | missing | Copy-and-relink | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/configure-ingest-settings.html (F) |
| **Import & ingest ›** Copy with MD5 verification | missing | Verified copy | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/migrate-existing-prelude-workflows-to-premiere.html (F) |
| **Import & ingest ›** Ingest settings: Transcode | missing | Source transcoding | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/configure-ingest-settings.html (F) |
| **Import & ingest ›** Ingest settings: Copy and Create Proxies | missing | Both halves | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/configure-ingest-settings.html (F) |
| **Import & ingest ›** Media Browser | missing | Card-structure reading and spanned-clip joining | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/migrate-existing-prelude-workflows-to-premiere.html (F) |
| **Import & ingest ›** Spanned clips | missing | Joining; today the author abuts elements by hand | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Import & ingest ›** Prelude workspace and Broadcast Template Project | missing | The workspace and the template (rough cut and logging halves are the editing and marker rows) | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/migrate-existing-prelude-workflows-to-premiere.html (F) |
| **Import & ingest ›** Hardware-accelerated decoding | missing | GPU decode (overlaps performance ticket #894) | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/enable-hardware-accelerated-decoding-support.html (F) |
| **Import & ingest ›** Remote / cloud-hosted media (Frame.io V4 panel, Cloud Media Downloads) | partial | `render`/`frame` use local sources only (ADR-0131); ADR-0159's remote instance with download cache is accepted but v0.2.0 is stdio-only | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **Import & ingest ›** Import from Adobe Firefly | missing | The handoff (a downloaded file can be named as a `source`) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-media-from-adobe-firefly.html (F) |
| **Import & ingest ›** Import from Firefly Boards | missing | The handoff | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-media-from-firefly-boards.html (F) |
| **Import & ingest ›** Firefly Boards: create a sequence on import | missing | The automatic assembly | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-media-from-firefly-boards.html (F) |
| **Import & ingest ›** Creative Cloud Libraries | missing | All of it | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/about-creative-cloud-libraries.html (F) |
| **Import & ingest ›** Adobe Stock panel | missing | All of it | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/use-built-in-adobe-stock.html (F) |
| **Import & ingest ›** Motion Graphics templates (.mogrt) import | missing | All of it (overlaps #887) | https://helpx.adobe.com/premiere/desktop/add-text-images/use-motion-graphics-templates/about-motion-graphics-templates.html (F) |
| **Proxies ›** Create proxies | missing | Proxy media files of any kind | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/create-proxies.html (F) |
| **Proxies ›** Proxy watermark | missing | A picture watermark | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/create-proxies.html (F) |
| **Proxies ›** Attach proxies | missing | Attach | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/attach-proxies-to-full-resolution-media.html (F) |
| **Proxies ›** Reconnect full-resolution media | missing | Reconnect | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/reconnect-full-resolution-media-to-proxies.html (F) |
| **Proxies ›** Detach proxies | missing | Detach | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/detach-proxies.html (F) |
| **Proxies ›** Toggle proxies for playback | partial | Decode still reads full-size sources (ADR-0065: 8K is decode-bound), so the decode saving a proxy file buys is absent; no author toggle beyond `--full` | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| **Offline & relink ›** Link Media / relink | partial | No search for a moved file by name or attributes, and no relink-all; ADR-0002 leaves replace-all to tooling that does not exist | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/locate-and-link-offline-files.html (F) |
| **Offline & relink ›** Relink to a different file | partial | Replace-all is a find-and-replace the agent does itself | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/relink-offline-clips.html (F) |
| **Offline & relink ›** Automatic relink (historical paths, two-pass search, cross-platform) | partial | No path history, no search, no cross-OS path matching; status was `have` before this pass | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/relink-offline-media-automatically.html (F) |
| **Offline & relink ›** Open a project with missing files (Find, Skip, Skip All, Skip Previews, Offline, Offline All) | partial | No persistent offline mark: every run re-reports, and `render` refuses (ADR-0093) | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/open-projects.html (F) |
| **Offline & relink ›** Create offline clips | missing | A placeholder that carries the logged facts | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/create-offline-clips.html (F) |
| **Offline & relink ›** Edit offline clips | missing | All of it | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/edit-offline-clips.html (F) |
| **Offline & relink ›** Make offline | missing | A deliberate offline state | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/convert-online-clips-to-offline-clips.html (F) |
| **Offline & relink ›** Merge clips (sync separate audio and video) | partial | No merged item; no sync by timecode or marker (neither exists); the agent computes the offset | https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-project-panel.html (F) |
| **Media cache ›** Media cache (CFA conform, PEK peaks, cache database) | partial | Caches probe facts only: no conformed audio, waveform peaks or decode indexes | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/audio-conforming-specifications.html (F) |
| **Media cache ›** Media cache management | partial | No location, age or size setting; not an MCP tool | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/automatically-manage-your-media-cache-files.html (F) |
| **Project Manager ›** Collect Files and Copy to New Location | missing | Collect and copy | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/copy-project.html (F) |
| **Project Manager ›** Consolidate and Transcode | missing | Trim, handles, transcode | https://helpx.adobe.com/premiere/desktop/organize-media/transfer-files/copy-transcode-or-archive-project.html (F) |
| **Project Manager ›** Render and replace | missing | Flattening an element or range back to a source | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-and-replace-media-in-a-sequence.html (F) |
| **Projects ›** Create a project (name, location, template, settings) | partial | No templates, scratch disks or ingest settings | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/create-new-project.html (F) |
| **Projects ›** Project templates | partial | Nothing ships or applies templates; bins and search bins are refused (ADR-0002); no adjustment layer or submix (see #885, #889) | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/create-your-own-project-templates.html (F) |
| **Projects ›** Open several projects and drag between them | partial | No cross-project verb; relative `source` paths can break when copied (ADR-0053) | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/edit-multiple-open-projects.html (F) |
| **Projects ›** Project locking | missing | Locking | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/project-locking-with-multiple-open-projects.html (F) |
| **Projects ›** Save a workspace per project | missing | *GUI-only* (workflow #894) | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/save-workspace-when-editing-multiple-projects-simultaneously.html (F) |
| **Projects ›** Backward compatibility | partial | No converter: a file using a newer key fails on an older binary, with no downgrade route | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/backward-compatibility-of-projects.html (F) |
| **Projects ›** Move a project to another computer | partial | No collect step (see Collect Files row) | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/move-project-to-another-computer.html (F) |
| **Projects ›** Productions | missing | Cross-project references, shared cache, locking | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/about-productions.html (F) |
| **Organising ›** Labels (colour labels and presets) | partial | No colour; nothing labels a source | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/labels-preferences.html (F) |
| **Organising ›** Label colours in sequence tabs | missing | *GUI-only* | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/label-colors-in-sequence-tabs.html (F) |
| **Organising ›** Search box and columns (Project panel, Media Browser) | partial | No source catalogue to search; whole-value matches only, no partial-name search; no metadata | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-options-in-premiere.html (F) |
| **Organising ›** Find dialog | partial | Visual, audio and transcript results (see Media Intelligence rows) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/find-assets-using-the-find-dialog.html (F) |
| **Organising ›** Search bins (saved searches) | missing | Saved queries | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/create-your-own-project-templates.html (F) |
| **Organising ›** Sequence Index CSV export | partial | No CSV and no column choice | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/navigate-timelines-with-sequence-index.html (F) |
| **Organising ›** Effects-panel custom and preset bins | missing | Effect presets and their bins (overlaps #885) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/manage-bins.html (F) |
| **Media Intelligence ›** Visual search | missing | Content search (overlaps AI ticket #893) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-for-media-using-ai-powered-media-intelligence.html (F) |
| **Media Intelligence ›** Search for audio | missing | Audio search (#893) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-for-audio-using-media-intelligence.html (F) |
| **Media Intelligence ›** Search similar frames | missing | Similarity search (#893) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-for-similar-visuals-using-media-intelligence.html (F) |
| **Media Intelligence ›** Search panel advanced filters | partial | No exclude terms; probe facts (frame rate, file type) are not queryable fields | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-for-media-using-ai-powered-media-intelligence.html (F) |
| **Media Intelligence ›** Analysis cache (Media Cache, `.prmi` sidecar, none) | missing | (Depends on analysis) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/manage-media-intelligence-metadata.html (F) |
| **Media Intelligence ›** Analysis behaviour (background, local, opt-in per import) | missing | (Depends on analysis) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/media-intelligence-and-search-panel.html (F) |
| **Metadata ›** Metadata panel / XMP | partial | No descriptive or XMP metadata read or write | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/metadata-in-premiere.html (F) |
| **Metadata ›** Edit XMP metadata | missing | Writing into sources | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/edit-xmp-metadata.html (F) |
| **Metadata ›** Custom metadata schemas and properties | missing | User-defined fields | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/create-metadata-schemas-and-properties.html (F) |
| **Metadata ›** Link clip data to XMP | missing | All of it | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/link-clip-data-to-xmp-metadata.html (F) |
| **Metadata ›** Clip data (Premiere Project Metadata schema) | partial | No log note, description, scene, shot or tape name | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/view-clip-data-in-the-metadata-panel.html (F) |
| **Metadata ›** Show, hide and save metadata sets | missing | *GUI-only* | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/save-switch-or-delete-metadata-sets.html (F) |
| **Metadata ›** Search in the Metadata panel | missing | (Depends on metadata) | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/search-xmp-metadata.html (F) |
| **Metadata ›** XMP on import (write XMP ID, write clip markers to XMP, clip/XMP linking) | missing | All of it | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| **Overlays ›** Monitor overlay settings | partial | No source timecode, marker or sound-roll fields (no such facts exist); workflow #894 rates the generic overlay row `have` | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/create-overlay-settings.html (F) |
| **Overlays ›** Overlay placement and reference | missing | *GUI-only* | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/overlay-settings-reference.html (F) |
| **Overlays ›** Edit point and end-of-media indicators | partial | Nothing is drawn on the picture | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/display-metadata-as-overlays.html (F) |
| **Overlays ›** Safe margins (action, title, 4:3) | missing | Safe-area guides | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/overlay-settings-reference.html (F) |
| **Overlays ›** Overlay presets (`.olp`) | missing | *GUI-only* | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/import-export-or-delete-overlay-settings.html (F) |
| **Overlays ›** Multi-Camera View overlays | missing | Multicam first | https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/display-overlays-in-multi-camera-view.html (F) |
| **Overlays ›** Source timecode shown during trim, slip and slide | missing | Source timecode | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/view-source-timecode-in-the-program-monitor.html (F) |
| **Markers ›** Marker types (comment, chapter, segmentation, web link, Flash cue point) | missing | Typed markers | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/overview-of-markers.html (F) |
| **Markers ›** Clip markers (Source Monitor) | missing | A marker in source time, carried by its element | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/add-a-marker-to-a-clip.html (F) |
| **Markers ›** Sequence markers | missing | A timed, named, render-inert point | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/add-a-marker-to-a-clip.html (F) |
| **Markers ›** Effect Controls markers | missing | Cue markers | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/add-a-marker-to-a-clip.html (F) |
| **Markers ›** Marker properties dialog and Markers panel | missing | Markers first | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/view-marker-comments.html (F) |
| **Markers ›** Show or hide markers by colour | missing | Markers first | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/show-or-hide-markers-by-color.html (F) |
| **Markers ›** Default marker colours | missing | Markers first | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/set-default-marker-colors.html (F) |
| **Markers ›** Copy and paste sequence markers | missing | Markers first | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/copy-and-paste-sequence-markers.html (F) |
| **Markers ›** Find, navigate, move and delete markers | missing | Markers first | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/find-move-and-delete-markers.html (F) |
| **Markers ›** Share markers with After Effects | missing | Both | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/share-markers-with-after-effects.html (F) |
| **Timecode ›** Unbroken timecode (logging and sync) | missing | Timecode reading | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/about-timecode.html (F) |
| **Timecode ›** Audio time units (samples, milliseconds) | partial | No sample-unit addressing | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/change-timecode-display-format.html (F) |
| **Recording ›** Record audio (Audio Track Mixer) | missing | Recording | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-audio-using-the-audio-track-mixer.html (F) |
| **Recording ›** Voice-over record from the timeline | missing | Voice-over recording | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-a-voice-over-on-an-audio-track-from-the-timeline.html (F) |
| **Recording ›** Input device setup | missing | (Depends on recording) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-audio.html (F) |
| **Recording ›** Mute input during timeline recording | missing | (Depends on recording) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/mute-input-during-recording.html (F) |
| **Immersive ›** 360° / 180° VR interpretation | missing | VR interpretation | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/interpret-vr-footage.html (F) |
| **Immersive ›** VR auto-detection by frame shape | missing | Detection and stereo handling | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-assignments.html (F) |
| **Immersive ›** VR video viewing (monitor VR mode, headset) | missing | VR viewing | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/vr-editing-in-premiere.html (F) |
| **Immersive ›** VR metadata on export | missing | Spherical metadata (overlaps #892) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/publish-vr-videos.html (F) |
| **Immersive ›** Ambisonics audio | missing | Ambisonic channel handling | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/assembling-ambisonics-audio.html (F) |
| **Immersive ›** Stereoscopic 3D (non-VR) | missing | Stereo pairs | unconfirmed (nearest: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html) |

### Export, delivery and interchange: 13 partial, 90 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| H.264 profiles and 10-bit (High10) | missing | Any bit depth or chroma format other than 8-bit 4:2:0 | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| HEVC (H.265) | missing | An HEVC encode path (encoder, container, licence posture under ADR-0009's spawn-not-link rule) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| AV1 (**unconfirmed**) | missing | AV1 encode, if Premiere offers it at all | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| Apple ProRes | missing | A ProRes (mezzanine) deliverable | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html |
| Avid DNxHD / DNxHR | missing | Any DNx encode, any MXF wrapper | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| GoPro CineForm | missing | CineForm encode | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| QuickTime (.mov) container | missing | A MOV deliverable; a `.mov` `output` still gets MP4 bytes | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video-and-audio-files.html |
| Uncompressed / lossless video | missing | A lossless deliverable. (ADR-0143's VMAF reference used `-qp 0` through a test wrapper only, never a shipped option) | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| AVI (Windows only) | missing | AVI container and its codecs | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| DV / DVCPRO | missing | DV-family encode | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| XAVC | missing | XAVC encode and MXF wrapping | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| MXF OP1a (generic) | missing | MXF container | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| AVC-Intra / XDCAM HD / P2 Movie | missing | Each of these encodes | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| AS-10 / AS-11 | missing | Broadcast-shim delivery | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| JPEG 2000 MXF OP1a | missing | JPEG 2000 encode | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| Wraptor DCP | missing | DCP packaging | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| MPEG-2 / DVD / Blu-ray | missing | MPEG-2 and Blu-ray-compliant streams | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| MPEG-4 | missing | MPEG-4 Part 2 | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| Windows Media (Windows only) | missing | WMV encode | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/file-formats-supported-export.html |
| Animated GIF | missing | A palette-quantised GIF deliverable | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Match Source (Rewrap) presets | missing | Any rewrap or pass-through | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Export Frame (one still) | partial | Only JPEG (default) and PNG; default is half scale (`--full` for true pixels); 8-bit only; no timecode in the name | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-a-still-frame.html |
| Image sequence export | missing | A numbered per-frame file sequence | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-a-still-image.html |
| DPX | missing | DPX writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html |
| TIFF | missing | TIFF writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html |
| Targa (TGA) | missing | TGA writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html |
| BMP (Windows) | missing | BMP writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-export-file-formats.html |
| OpenEXR | missing | EXR writer and a float pipeline | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Export with an alpha channel | missing | A transparent deliverable (a codec with alpha, and a background that can be absent) | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html |
| GIF transparency | missing | GIF with transparency | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Render Alpha Channel Only | missing | A matte (alpha-as-luma) deliverable | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Audio-only export (WAV, AIFF, MP3, AAC) | missing | An audio-only deliverable in any format | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Export video without audio | partial | No switch: a project with audible elements always carries its mix | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Audio codec, sample rate, bitrate, sample size | missing | Any choice of audio codec, rate, bit depth or bitrate. Hard-coded, but no ADR refuses it | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Audio channels and downmix | missing | Mono, surround or multichannel output | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Match Source audio | missing | Following the sources' channel count or rate | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html |
| QuickTime audio channel configuration | missing | Multiple audio tracks or channel maps in one deliverable | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Smart Rendering | missing | Any pass-through of unchanged source frames | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/smart-rendering-supported-formats.html |
| Use Previews | missing | Re-using preview renders in the deliverable | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Optimize Stills | missing | Variable-duration frames for stills | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Frame size on export | missing | An output size other than the project frame | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Source scaling | missing | Fit modes for a differently sized output | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Export crop | missing | Cropping the deliverable | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Source rotation | missing | An output-wide rotation | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Frame rate conversion and time interpolation | partial | Frame blending and optical flow; an export rate different from `fps` | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Field order (interlaced) | missing | Interlaced output | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Pixel aspect ratio | missing | Non-square output pixels | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Render at Maximum Depth | missing | Higher-than-8-bit compositing | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Multiplexer and stream compatibility | partial | Any other mux, elementary streams, or a compatibility choice | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Estimated file size / export summary | partial | Any estimate before encoding | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Set Start Timecode | missing | A start-timecode track | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Send to Media Encoder / background encoding | partial | No separate encoder application or persisted job list | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-directly-to-adobe-media-encoder.html |
| Encode queue | partial | A queue that outlives the MCP process or can be reordered or inspected as a list | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/overview-media-encoder-user-interface.html |
| Multiple outputs from one source | partial | Every render is the same shape (ADR-0143), so outputs can differ only in range and path | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-directly-to-adobe-media-encoder.html |
| Watch folders | missing | A folder-triggered encode | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/overview-media-encoder-user-interface.html |
| Publish to social destinations | missing | Platform authentication and upload | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-videos-for-social-media-channels.html |
| Social-platform presets and templates | missing | Named platform specs and safe-zone guides | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-videos-for-social-media-channels.html |
| AME Publish tab (FTP, Behance, Facebook, X, Vimeo, YouTube) | missing | Post-encode upload | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/publish-settings.html |
| Frame.io upload / share for review | missing | Any review-service upload | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/upload-media-to-frameio.html |
| HDR export (Rec. 2100 PQ / HLG) | missing | 10-bit output and a PQ/HLG transfer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/create-and-export-hdr-videos-for-youtube-on-mac.html |
| Export colour space | missing | A declared output colour space and its container tags | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| HDR Graphics White | missing | Graphics-white mapping | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| HDR10 metadata | missing | HDR10 static metadata | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| Export tone mapping (HDR to SDR) | missing | Tone mapping | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| SDR Conform | missing | SDR conform | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Captions as a sidecar file | missing | A sidecar writer (the words and times are in the file, so it is derivable) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html |
| Captions embedded in the stream | missing | Closed-caption data in the deliverable | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Export selected captions (TXT, CSV, SRT) | partial | A TXT, CSV or SRT writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-captions.html |
| Export transcripts | missing | A transcript and its export | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-transcripts.html |
| Export text from Motion Graphics templates | partial | A TXT/CSV writer (and Montagent has no .mogrt templates) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-text-from-motion-graphics.html |
| XMP metadata export | partial | Title/author/copyright/description fields, XMP, sidecar | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Markers in exported metadata | missing | Markers, and writing them into the file | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Content Credentials | missing | C2PA-style signed provenance | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-videos-with-content-credentials.html |
| VR video export | missing | VR metadata on export | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Import the export back into the project | missing | A one-step "deliverable becomes an element" | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Loop flag for looping formats (**unconfirmed**) | missing | Any container loop flag (moot until GIF exists) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/basic-video-settings.html |
| Text (name) overlay | partial | No derived text: the source or output file name is not filled in automatically | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/add-a-visual-watermark-during-export.html |
| Timecode overlay | missing | A per-frame derived timecode text (or a skill that writes one element per frame) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/add-a-visual-watermark-during-export.html |
| Source-timecode burn-in (Metadata & Timecode Burn-in) | missing | Per-frame source timecode text | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/add-a-visual-watermark-during-export.html |
| Video Limiter on export | missing | Luma/chroma legalisation | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Time Tuner | missing | Automatic imperceptible re-timing | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Timecode effect (timeline) | missing | Same as the timecode overlay | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/effects-and-transitions-removed-from-adobe-premiere.html |
| Final Cut Pro XML export | missing | An XML writer (timeline mapping, translation report) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-a-project-as-a-final-cut-pro-xml-file.html |
| EDL (CMX 3600) export | missing | EDL writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-a-project-as-an-edl-file.html |
| AAF export | missing | AAF writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-aaf-files.html |
| AAF: Mixdown video | missing | Mixdown-plus-AAF | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-aaf-files.html |
| AAF: audio rendering options | missing | Per-clip audio stems with handles | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-aaf-files.html |
| OMF export (Pro Tools) | missing | OMF writer | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-omf-files-for-pro-tools.html |
| OMF: what is carried | missing | The OMF mapping | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/supported-features-for-exporting-omf-files-for-pro-tools.html |
| OpenTimelineIO (OTIO) export | missing | OTIO writer | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html |
| Scripted export (UXP API) | partial | Interchange writers to script, and presets (refused, ADR-0143 §5) | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html |
| ALE export | missing | ALE writer | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html |
| AAF from Avid Media Composer into Premiere | missing | AAF reader | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-from-avid-media-composer-to-premiere.html |
| Export Frame into the project | missing | A one-step "still becomes an `image` element" | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-a-still-frame.html |
| Frame.io comments from markers | missing | Markers and a review-service bridge | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/upload-media-to-frameio.html |
| Export marker lists | missing | Markers and their export | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html |
| Trim a project for archive | missing | Consolidate and trim with handles | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-options-in-premiere.html |
| Export a selection as a project | missing | Extracting part of a timeline into a new project file | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html |
| SRT stream output | missing | Any live output stream | https://helpx.adobe.com/premiere/desktop/render-and-export/stream-video/enable-secure-reliable-transport-stream.html |
| SRT reliability and encryption | missing | Encrypted network delivery | https://helpx.adobe.com/premiere/desktop/render-and-export/stream-video/overview-of-secure-reliable-transport.html |
| SRT connection modes | missing | SRT handshake modes | https://helpx.adobe.com/premiere/desktop/render-and-export/stream-video/secure-reliable-transport-modes.html |
| SRT settings | missing | SRT connection settings | https://helpx.adobe.com/premiere/desktop/render-and-export/stream-video/srt-settings.html |
| Streaming indicator | missing | A live-stream status | https://helpx.adobe.com/premiere/desktop/render-and-export/stream-video/srt-settings.html |

### AI and automation: 20 partial, 29 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Generative Extend (video) | missing | Any frame generation. No skill calls an outside video model. | [Generative Extend overview][genext] (F), [Add frames][genext-add] (F), [FAQ][genext-faq] (F), [known issues][genext-ki] (F), [What's new][wn] (F) |
| Generative Extend (audio) | missing | Audio generation. A looped bed with an `audio_crossfade` ([ADR-0176](../../adr/0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)) is the only hand-written stand-in, and no skill teaches it. | [Add frames][genext-add] (F), [FAQ][genext-faq] (F), [overview][genext] (F), [known issues][genext-ki] (F) |
| Generated-media record: AI label, Regenerate, Revert to Original, Generation history | missing | A record of a generated source's prompt, model, seed and reference frames, a label that marks a source as generated, and a way to regenerate from that record. | [Add frames][genext-add] (F), [Generative Extend FAQ][genext-faq] (F), [Generative Media Tool FAQ][gmt-faq] (F), [Generate Music FAQ][music-faq] (F), [model output feedback][feedback] (F) |
| Generative Media Tool: text and reference frames to video (26.5) | missing | Generation, model choice, settings matched to the project, and a skill that calls an outside generator and records its prompt, model and seed as a recipe beside the footage. | [Generative Media Tool overview][gmt] (F), [Generate media][gmt-gen] (F), [FAQ][gmt-faq] (F), [What's new][wn] (F) |
| Generative Media Tool: generated transition from first and last frames (26.5) | missing | Frame-conditioned generation, and a skill that extracts the two frames and places the result. | [Generate media][gmt-gen] (F) |
| Generative Media Tool: sound effects (26.5) | missing | Generation, picture-synced timing, and voice-guided timing. | [Add generated music][music-add] (F), [Generative Media Tool overview][gmt] (F), [FAQ][gmt-faq] (F) |
| Generate Music (beta) | missing | Music generation, and a set of variations to choose from. | [Generate Music overview][music] (F), [Add generated music][music-add] (F), [FAQ][music-faq] (F) |
| Generate soundscape (beta) | missing | Picture-to-sound generation. | [Add generated music][music-add] (F) |
| Send media from Firefly on the web (26.0) | missing | Any hand-off from a generative service. Only a file already downloaded can be used. | [Import media from Firefly][ff-web] (F), [What's new][wn] (F) |
| Import from Firefly Boards, with automatic sequence (26.0; sequence 26.3) | missing | The hand-off from Boards, and its ordering. | [Import media from Firefly Boards][ff-boards] (F), [What's new][wn] (F), [release notes][rn] (F) |
| Content Credentials on export (26.2) | missing | A provenance manifest embedded in the deliverable, an identity, a training preference, and any record of which sources were AI-generated. | [Export with Content Credentials][c2pa] (F), [Generative Media Tool FAQ][gmt-faq] (F), [What's new][wn] (F), [release notes][rn] (F) |
| Scan for Content Credentials on import | missing | Reading a C2PA manifest from a source. | [Export with Content Credentials][c2pa] (F) |
| Object Mask: select a person or object (26.0; more objects 26.5) | partial | Segmentation: nothing finds the subject's outline, and no skill does. A path is a polygon, not a per-pixel matte (next row). | [Object masking][objmask] (F), [What's new][wn] (F), [release notes][rn] (F) |
| Object Mask edge quality: Sharp and Smooth modes, Sharpness (26.2; Sharpness 26.5) | partial | Smooth: a soft per-pixel matte. A matte from another element is refused ([ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)) and an image-sourced mask is deferred ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md) §3), so a model's matte has no way in today. No Sharpness curve. | [Object masking][objmask] (F), [Sharp and Smooth modes][objmask-modes] (F), [What's new][wn] (F) |
| Object Mask data storage and Restore (26.3) | partial | No recorded selection steps to replay: nothing produced the path, so nothing can regenerate it. | [Object masking][objmask] (F), [What's new][wn] (F), [release notes][rn] (F) |
| Object Mask tracking (26.0; faster 26.3) | partial | The tracker, and a skill that writes per-frame mask keyframes from footage. | [Object masking][objmask] (F), [Mask tracking tools][masktrack] (F), [Track masks][masktrack2] (F), [What's new][wn] (F) |
| Shape-mask tracking (Rectangle, Ellipse, Pen) | partial | A point, planar or perspective tracker, and a skill that runs one outside and writes the keyframes. | [Mask tracking tools][masktrack] (F), [Track masks][masktrack2] (F) |
| Separate Crosstalk (cloud AI voice feature) | missing | Source separation, and a skill to run it and place the stems. | [Separate overlapping speakers][crosstalk] (F), [About cloud-based AI voice features][voice-ai] (F) |
| Audio category auto-tagging | missing | The classifier, and any typed role that a tool reads. The duck takes its `--bed` and `--voice` ids from the agent ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) §3). | [Automatically tag audio][autotag] (F), [Essential Sound panel][esp] (F) |
| Remix | partial | The beat-similarity analysis that picks matching cut points, and a script that writes the rearranged elements to a target length. Beat finding is itself a workaround for #547. | [Remix audio][remix-ai] (F), [Remix in Premiere][remix] (F), [Remix tool considerations][remix-tips] (F) |
| Bulk mute or bleep censored words (26.0) | partial | The transcription, a word-list matcher, a true mute (the duck's depth and ramps are tuned for music beds), and a bundled bleep tone. No skill does it. | [Clean audio with bulk mute or bleep][bleep] (F), [What's new][wn] (F), [release notes][rn] (F) |
| Sound search (media intelligence, 26.0) | missing | Audio content analysis and search. | [Search for audio][mi-audio] (F), [What's new][wn] (F), [release notes][rn] (F) |
| Auto Color | partial | The analysis, and no skill for it. White balance (temperature) and exposure are absent; LUT and curve are on the colour filter's avoid list and need their own ADR under ADR-0049's four clauses. | [Use Auto Color][autocolor] (F), [colour correction workflow][colorwf] (F) |
| Color Match (Clip Match) | partial | Measured matching (no histogram or scope reading is served), face weighting, colour wheels. No skill. | [Match color between shots][colormatch] (F), [Color mode basics][colormode] (F) |
| Color mode: automatically analysed tonal zones (27.0 beta) | missing | Tonal analysis, and any zone- or luma-qualified adjustment. | [Default Adjust operation][adjust] (F), [beta features][beta] (F) |
| Color mode: automatically sampled colour patches (27.0 beta) | missing | Hue analysis, and hue-qualified adjustment. | [Default Adjust operation][adjust] (F) |
| Morph Cut | missing | Optical-flow or face-aware interpolation between two frames. A crossfade is the only stand-in. | [Morph Cut overview][morphcut-ov] (F), [Apply Morph Cut][morphcut] (F), [Morph Cut options][morphcut-opt] (F) |
| Auto Reframe (effect on a clip) | missing (R4) | Subject detection and the motion path from it. No skill, and the recipe is a single static slide. | [Auto Reframe overview][reframe] (F), [on a clip][reframe-clip] (F) |
| Auto Reframe Sequence (custom resolution 26.2.2) | missing (R4) | A tool or script that makes the new-aspect copy and reframes every element, and the analysis behind it. | [Auto Reframe overview][reframe] (F), [on a sequence][reframe-seq] (F), [release notes][rn] (F) |
| Scene Edit Detection | missing | Shot-boundary detection in a source file. No markers, bins or subclips to write the result to; the agent would write split `video` elements by source range. | [Scene Edit Detection][sed] (F) |
| Automate to Sequence | partial | A bundled script that lays N clips back to back with transitions. There is no default transition. | [Set and apply default transitions][automate] (F), [Project panel options][projpanel] (F) |
| Synchronize clips (and multicam source sequences) by audio | missing | Audio alignment, and a skill that writes the resulting `start` or `source_start` offsets. | [Synchronize clips in the Timeline][sync] (F), [multi-camera source sequence][multicam] (F) |
| Warp Stabilizer | missing | Motion analysis and warping, and a pre-render recipe for an outside stabiliser. | [Warp Stabilizer][warp] (F), [settings][warp-set] (F) |
| Optical Flow time interpolation | missing | Motion-compensated frame synthesis. | [Time interpolation methods][interp] (F) |
| Speech to Text (transcription) | partial | A transcriber. Running one inside Montagent is refused by ADR-0051 ("does not invoke, ship, or shell out to a forced aligner"); no skill runs one outside either, and every skill assumes a words file is supplied. | [Auto transcribe][stt] (F), [languages][stt-lang] (F), [language packs][stt-packs] (F), [preferences][stt-prefs] (F), [multichannel][tbe-multi] (F), [TBE overview][tbe] (F) |
| Spoken-language auto-detection | missing | Language identification, and a place to record a source's language. | [preferences][stt-prefs] (F) |
| Speaker labels | missing | Diarisation. The words convention has no speaker key. | [Edit speaker names][speakers] (F), [preferences][stt-prefs] (F), [Transcribe individual files][tbe-files] (F), [release notes][rn] (F) |
| Voice Activity Detection (26.5) | partial | Use of the result anywhere else: `duck.py` refuses silence detection ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) §2), and there is no model-based detector. | [Release notes][rn] (F) |
| Single-word captions (26.3) | partial | A one-word-per-page setting: the spec's `lines` and `width` only approximate it. | [Create single word captions][captions-1w] (F), [What's new][wn] (F) |
| Translate captions | partial | Timing for the translation: `captions.py` needs per-word times in the caption's language, so a translated page must be timed by hand to the source page. No skill. | [Translate captions][translate] (F), [What's new][wn] (F) |
| Text-Based Editing | partial | A script that turns a transcript selection into source ranges and a ripple. The transcript itself is outside (Speech to Text row). | [TBE overview][tbe] (F), [edit sequences][tbe-seq] (F), [add clips][tbe-add] (F) |
| Filler-word detection and bulk delete | partial | Filler tagging (many aligners drop fillers), and a cut-and-ripple script. | [Detect and delete pauses][pauses] (F), [TBE overview][tbe] (F) |
| Pause detection and bulk delete | partial | A script that cuts at the detected pauses and ripples. | [Detect and delete pauses][pauses] (F) |
| Paper Edit (26.5) | partial | Transcription, and a script from passage selection to elements. | [Create a sequence with Paper Edit][paperedit] (F), [What's new][wn] (F), [release notes][rn] (F) |
| Remove all of one speaker | missing | Diarisation, then the same cut-and-ripple script as Text-Based Editing. | [Remove speaker dialogue][rmspeaker] (F), [Detect and delete pauses][pauses] (F) |
| Import a corrected transcript | partial | Mapping a plain-text correction back onto the timings. | [Edit transcripts][tbe-edit] (F) |
| Media intelligence search (26.0) | partial | An index or a search across media. Each clip is looked at one sheet at a time, and there is no project bin to search. | [Search media using AI][mi] (F), [Media intelligence and Search panel][mi-faq] (F), [preferences][stt-prefs] (F), [What's new][wn] (F) |
| Media analysis cache and project index | missing | Content analysis to cache, and a shareable sidecar for it. | [Manage media intelligence metadata][mi-meta] (F), [Media intelligence and Search panel][mi-faq] (F), [preferences][stt-prefs] (F) |
| Search Similar Frames and alternate takes (26.0) | missing | Visual embedding and similarity search. | [Find similar visuals][similar] (F), [What's new][wn] (F), [release notes][rn] (F) |

### VR / 360 and immersive: 12 partial, 54 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| VR editing (360° and 180°, mono and stereo) | missing | A projection, layout and captured-view model on sources and on the project. | [VR editing in Premiere][OVR], [VR editing in Premiere (alt URL)][OVR2], [VR auto-detection][DET], [VR assignments][ASG] (F) |
| VR properties assigned from file metadata | missing | Reading spherical metadata (e.g. MP4 `sv3d`/`st3d`, ffprobe `Spherical Mapping`, `Stereo 3D`). | [VR assignments][ASG], [VR auto-detection][DET] (F) |
| Detect VR by frame aspect | missing | The aspect-and-height heuristic, and the property it would set. | [VR assignments][ASG] (F) |
| VR properties in the Properties panel | missing | Reporting the properties in `probe`/`query`. | [VR auto-detection][DET] (F) |
| Interpret VR footage | missing | A per-element VR interpretation (projection, layout, H/V captured view). Not refused: ADR-0002 rules out where Premiere stores it, not the field. | [Interpret VR footage][INT] (F) |
| VR properties on sequences | partial | The VR Properties block itself: projection, layout and captured view on the project, read by render and export. | [Assign VR properties to sequences][SEQ] (F) |
| VR sequence presets | partial | 29.97 (integer fps only, see editing.md), the VR properties, and the Ambisonics (multichannel) variant. | [Monitor Ambisonics audio][MON] (F) |
| Custom sequence preset with VR settings | missing | Saved project presets, and the VR settings in them. | [Create a custom sequence preset][CSP] (F) |
| New sequence from a VR clip | missing | Matching a project to a source, and VR properties to inherit. No fetched page states VR inheritance. | unconfirmed ([Assign VR properties to sequences][SEQ]) |
| Mixed resolutions and layouts in one sequence | partial | Per-element layout conformance; a mono and a stereo source cannot be told apart or converted except by hand (see "Output Layout" below). | [Three-axis video rotation][ROT] (F) |
| 180° VR (VR180) | missing | A captured-view value, and half-sphere viewing and export. | [VR editing in Premiere][OVR], [VR auto-detection][DET] (F) |
| VR Video display in the Source and Program Monitors | missing | A viewport into the sphere, live or as a `frame` option. | [360-degree panning][PAN], [Choose a display mode][DISP], [VR auto-detection][DET] (F) |
| Toggle VR Video Display button | missing | A VR display to toggle. | [360-degree panning][PAN] (F) |
| Look around interactively | missing | Interactive view direction. | [360-degree panning][PAN] (F) |
| VR view controls (pan/tilt scroll bars, hot text, continual-pan dial) | missing | View-only pan/tilt controls. | [Hide VR video view controls][HIDE], [360-degree panning][PAN], [Monitor Ambisonics audio][MON] (F) |
| VR Video settings (monitor field of view, stereoscopic view) | missing | Viewer FOV and eye selection. No fetched page describes this dialog. | unconfirmed ([VR editing in Premiere][OVR]) |
| Hide VR video view controls | missing | Nothing to hide: there are no viewer controls. | [Hide VR video view controls][HIDE] (F) |
| Head-mounted display monitoring | missing | Headset playback. | [VR editing in Premiere][OVR], [Choose a display mode][DISP] (F) |
| Simulate viewing platforms | missing | Any VR display. | [Choose a display mode][DISP] (F) |
| GPU acceleration and VRAM for VR | missing | A GPU path. Montagent has no VRAM ceiling either, since it has no VR effects to run. | [GPU acceleration][GPU], [List of accelerated effects][ACC] (F) |
| Real-time playback of VR rotation | missing | Real-time playback of any kind. | [Three-axis video rotation][ROT] (F) |
| Three-axis video rotation (correct the zero angle) | partial | Tilt and roll (a spherical remap), a single control for the rotation, and doing it on one element. Stereo needs one wrap pair per eye with `clip`. | [Three-axis video rotation][ROT], [Restage aligned video and audio][RES] (F) |
| Restage correctly aligned video and audio | missing | Sound-field rotation, and anything to align it to. | [Restage aligned video and audio][RES] (F) |
| Input Auto VR Properties | missing | Input layout and FOV, read or written. | [Three-axis video rotation][ROT] (F) |
| Output Auto VR Properties | missing | Output VR properties, inherited or set. | [Three-axis video rotation][ROT] (F) |
| Output Layout | partial | A layout field; the conversion is hand-built from two or more elements per clip and is not declared anywhere. | [Three-axis video rotation][ROT] (F) |
| Output Horizontal / Vertical FOV | missing | A spherical re-projection. A 360→180 crop of a mono equirect is a `clip` of the middle half, but the vertical axis and stereo cases are not. | [Three-axis video rotation][ROT] (F) |
| Disparity Adjustment | partial | A field for it; four elements per stereo clip by hand. | [Three-axis video rotation][ROT] (F) |
| Filtering | partial | The spherical re-projection it filters, and a user setting for sample count. | [Three-axis video rotation][ROT] (F) |
| Auto VR properties on every immersive effect | missing | Per-eye, seam-aware effects. | [Immersive video effects and transitions][IMT], [Immersive Video effects][IMF] (F) |
| VR Blur | missing | Spherical wrap. | [Immersive Video effects][IMF] (F) |
| VR Chromatic Aberrations | missing | Spherical model, and channel offsets. | [Immersive Video effects][IMF] (F) |
| VR Color Gradients | missing | Spherical model. | [Immersive Video effects][IMF] (F) |
| VR De-Noise | missing | Denoise, and the spherical model. | [Immersive Video effects][IMF], [Effects removed from Premiere][REM] (F) |
| VR Digital Glitch | missing | Spherical model. | [Immersive Video effects][IMF] (F) |
| VR Fractal Noise | missing | Fractal noise, and the spherical model. | [Immersive Video effects][IMF] (F) |
| VR Glow | missing | Spherical wrap. | [Immersive Video effects][IMF] (F) |
| VR Plane to Sphere | missing | Equirectangular projection of a flat element. ADR-0167's thesis ("projected, never placed") would admit it by mechanism. | [Immersive Video effects][IMF] (F) |
| VR Projection | missing | Spherical model. | [Immersive Video effects][IMF], [Three-axis video rotation][ROT] (F) |
| VR Rotate Sphere | partial | X and Z rotation (tilt, roll), and one element doing it. | [Immersive Video effects][IMF] (F) |
| VR Sharpen | missing | Sharpen, and the spherical model. | [Immersive Video effects][IMF] (F) |
| VR Chroma Leaks | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| VR Gradient Wipe | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| VR Iris Wipe | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| VR Light Leaks | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| VR Light Rays | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| VR Mobius Zoom | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| VR Random Blocks | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| VR Spherical Blur | missing | Spherical model. | [Immersive video effects and transitions][IMT], [List of video transitions][LVT] (F) |
| Ambisonic (B-format) clips | missing | Keeping a multichannel source intact, and knowing it is B-format. | [Assembling Ambisonics audio][AMB] (F) |
| Ambisonics in a separate WAV or inside MP4 | partial | Keeping the channels: both arrive folded to stereo (ADR-0077 §8). | [Assembling Ambisonics audio][AMB] (F) |
| Ambisonics sequence (multichannel master) | missing | A multichannel bus and master. | [Monitor Ambisonics audio][MON], [Assembling Ambisonics audio][AMB] (F) |
| Panner – Ambisonics | missing | Sound-field rotation. | [Restage aligned video and audio][RES] (F) |
| Binauralizer – Ambisonics | missing | A binaural decode. | [Monitor Ambisonics audio][MON] (F) |
| Match binaural monitoring to the VR viewer | missing | A viewer, and a binaural decode to align with it. | [Monitor Ambisonics audio][MON] (F) |
| Channel routing for ambisonic channel order | missing | Multichannel routing. No fetched page describes channel order. | unconfirmed ([Assembling Ambisonics audio][AMB]) |
| Video is VR (spherical metadata on export) | missing | Writing spherical/stereo metadata (`sv3d`/`st3d` or the Spherical Video V1 XMP box). | [Publish VR videos][PUB], [Export settings reference][EXP] (F) |
| VR export in H.264, HEVC or QuickTime | partial | HEVC and QuickTime (see export.md), and the VR flag on any of them. | [Publish VR videos][PUB] (F) |
| Export frame layout | partial | The flag that tells a player which layout it is, and a conversion that is not hand-built. | [Export settings reference][EXP], [Publish VR videos][PUB] (F) |
| Export horizontal / vertical field of view | missing | Captured view in the metadata. | [Export settings reference][EXP] (F) |
| Audio is Ambisonics | missing | Multichannel output and its Ambisonics flag. | [Assembling Ambisonics audio][AMB] (F) |
| Stretch to Fill on export | partial | Export-time scaling to a different output size. | [Publish VR videos][PUB] (F) |
| Publish to YouTube or Facebook | missing | Platform upload. | [Publish VR videos][PUB], [VR editing in Premiere][OVR] (F) |
| VR export presets | missing | VR presets, which presuppose the flags above. No fetched page names them. | unconfirmed ([Export settings reference][EXP]) |
| 360° stabilisation (CyberLink VR Stabilizer) | missing | 360° stabilisation. | [Plug-ins][PLG] (F) |
| 360/VR tracking and masking (Boris FX Mocha Pro) | missing | 360-aware tracking and masking. | [Plug-ins][PLG] (F) |

### Workflow: interface, monitors, preferences, render: 42 partial, 121 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| Workspaces | missing | *GUI-only.* No arrangeable views. | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/change-create-reset-workspaces.html (F) |
| Manage workspaces | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/manage-workspaces.html (F) |
| Import workspace from projects | missing | *GUI-only.* No per-project view state. | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/import-workspaces.html (F) |
| Panels: open, close, resize, scroll tabs | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/customize-panels.html (F) |
| Dock, group, undock panels | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/dock-group-undock-panels.html (F) |
| Display any panel full-screen | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/display-any-panel-full-screen.html (F) |
| Panel menus and context menus | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/display-panel-options-and-menu.html (F) |
| Navigate the panels | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/navigate-the-panels.html (F) |
| Project panel position | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-the-position-of-the-project-panel.html (F) |
| Home screen | partial | No recent-projects list. Montagent keeps no record of projects across runs. | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/what-are-workspaces.html (F) |
| Info panel | partial | `probe` is CLI-only. No tape name. | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/clip-details-in-the-info-panel.html (F) |
| HiDPI display support | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/adobe-premiere-faq.html (F) |
| Tools panel | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/tools-panel-and-options-panel.html (F) |
| Ripple Edit tool | partial | `shift` moves times only later, so a ripple that shortens (moves later edits earlier) cannot be done in one step (see `editing.md`). | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/tools-panel-and-options-panel.html (F) |
| Hand tool | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/tools-panel-and-options-panel.html (F) |
| Generative Extend tool | missing | No generated frames or audio. | https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/tools-panel-and-options-panel.html (F) |
| Touch gestures on Windows | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/use-touch-and-gesture-controls/control-premiere-through-touch-and-gesture-in-microsoft-surface-pro-and-windows.html (F) |
| Touch buttons and drop-zones | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/use-touch-and-gesture-controls/learn-how-touch-and-gesture-actions-work-with-buttons.html (F) |
| Thumbnail controls for all pointing devices | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/use-touch-and-gesture-controls/use-thumbnail-controls-in-the-project-panel.html (F) |
| List View and Metadata Display | partial | No list of unused media (ADR-0002). No column choice, no thumbnail. `probe` is CLI-only. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-list-view-in-project-panel.html (F) |
| Icon View thumbnails and zoom | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-icon-view-in-project-panel.html (F) |
| Poster frame | missing | *GUI-only.* No per-file representative frame. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-icon-view-in-project-panel.html (F) |
| Sort icons | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-icon-view-in-project-panel.html (F) |
| Hover Scrub | partial | No live scrub, and a scratch project is needed. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-icon-view-in-project-panel.html (F) |
| Freeform View arrangement | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-freeform-view-in-project-panel.html (F) |
| Freeform Align and Reset to Grid | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/align-and-reset-clips-to-grid-in-freeform-view-in-project-panel.html (F) |
| Freeform layouts | missing | *GUI-only.* ADR-0017 would put view state in a sidecar, not the project. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/create-layouts-in-freeform-view-in-project-panel.html (F) |
| Freeform metadata display | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-metadata-in-freeform-view-in-project-panel.html (F) |
| Default keyboard shortcuts | missing | *GUI-only.* No key bindings. | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html (F) |
| Custom shortcuts and preset sets | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/customize-keyboard-shortcuts.html (F) |
| Visual keyboard layout | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/about-keyboard-shortcuts.html (F) |
| Shortcut conflict resolution | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/about-keyboard-shortcuts.html (F) |
| Drag-and-drop assignment | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/about-keyboard-shortcuts.html (F) |
| Several shortcuts per command | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/assign-multiple-keyboard-shortcuts-to-a-command.html (F) |
| Copy shortcuts between computers | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/copy-keyboard-shortcuts-from-one-computer-to-another.html (F) |
| Remove shortcuts and sets | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/remove-keyboard-shortcuts.html (F) |
| Preferences store | missing | No preferences file. No ADR refuses one. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/preferences-overview.html (F) |
| At Startup | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/general-preferences.html (F) |
| When Opening a Project | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/general-preferences.html (F) |
| Projects double-click behaviour | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/general-preferences.html (F) |
| Show Workspace Reset Warning dialog | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/general-preferences.html (F) |
| GPU accelerated UI rendering | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/general-preferences.html (F) |
| Appearance: themes and contrast | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/appearance-preferences.html (F) |
| Display Color Management | missing | No display transform. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/color-preferences.html (F) |
| Extended Dynamic Range Monitoring | missing | No HDR or EDR preview. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/color-preferences.html (F) |
| AI Models preferences | missing | No model to describe. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/ai-models-preferences.html (F) |
| Audio device class and driver | missing | No audio device I/O. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-hardware-preferences.html (F) |
| Default input and output devices | missing | No recording, no monitoring. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-hardware-preferences.html (F) |
| Clock, clock source and sample rate | missing | No hardware clock. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-hardware-preferences.html (F) |
| Latency and I/O buffer size | missing | No real-time path. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-hardware-preferences.html (F) |
| Map Output | missing | No output mapping. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-hardware-preferences.html (F) |
| Automatch Time | missing | No automation recording. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| 5.1 Mixdown Type | missing | No 5.1 track or output. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Sum multichannel to mono in Source Monitor | missing | No monitoring mode. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Play audio while scrubbing | partial | No scrubbing. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Maintain pitch while shuttling | missing | No variable-speed monitoring. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Mute input during timeline recording | missing | No recording. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Generate waveforms on import | missing | No waveform. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Auto-tag audio types | missing | No classification. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Automation keyframe thinning | missing | No thinning. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Render Edit in Audition files location | missing | No Audition hand-off. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html (F) |
| Auto Save | partial | No automatic snapshot. A version exists only when the agent (or the user) commits. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/auto-save-preferences.html (F) |
| Auto Save also saves the current project | missing | No automatic save. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/auto-save-preferences.html (F) |
| Emergency backup project | missing | No automatic backup. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/auto-save-preferences.html (F) |
| Save backup project to Creative Cloud | partial | Nothing pushes automatically. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/auto-save-preferences.html (F) |
| Unshared Team Project changes reminder | missing | No reminder. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/collaboration-preferences.html (F) |
| Project locking preference | missing | No project lock. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/collaboration-preferences.html (F) |
| Hardware control surfaces | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/control-surface-preferences.html (F) |
| Ligatures | partial | No author switch for ligatures. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html (F) |
| Default stroke join and cap (text and shapes) | missing | No default. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html (F) |
| Default background fill | missing | No default, no field. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html (F) |
| Default text, subtitle and closed-caption fonts | missing | No default. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html (F) |
| Import CEA-708 captions with wide aspect | missing | No caption import. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html (F) |
| Indeterminate Media Timebase | missing | No image sequences. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Timecode on import | partial | Embedded source timecode is never read. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Default Media Scaling | missing | No default. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Write XMP ID to files on import | missing | No file tagging. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Write clip markers to XMP | missing | No markers. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Clip and XMP metadata linking | missing | No metadata. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Include captions on import | missing | No caption import. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Enable proxies | missing | No proxy media. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Project import options | missing | No project import. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Growing files | partial | No periodic refresh, no live playback. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Hardware decoding preference | missing | No hardware decode. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| RAM reserved for other applications | missing | No memory cap. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/memory-preferences.html (F) |
| Installed and available RAM display | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/memory-preferences.html (F) |
| Pause Media Encoder queue during playback | missing | No queue. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/playback-preferences.html (F) |
| Audio playback offset | missing | No device offset. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/playback-preferences.html (F) |
| Primary audio device | missing | No playback. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/playback-preferences.html (F) |
| Mercury Transmit and transmit devices | missing | No external video output. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/playback-preferences.html (F) |
| Default transition and still durations | missing | No user-set defaults that Montagent applies on insert, because Montagent never inserts. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Timeline playback auto-scrolling | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Timeline mouse scrolling | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Default audio tracks | missing | No track channel type. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Mono, stereo, 5.1 and multichannel media interpretation | partial | No choice of mapping, no surround, no adaptive tracks. ADR-0184's `channels` is proposed, not shipped. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Set focus on the Timeline after Insert/Overwrite | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Return to beginning at playback end | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Out-of-sync indicators for unlinked clips | partial | No check that an audio element and the video element from one file stay in step. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Play after rendering previews | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Restore open sequences when opening projects | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html (F) |
| Roll and Ripple without a modifier | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/trim-preferences.html (F) |
| Shift overlapping clips during ripple trim | partial | `shift` moves later only (see `editing.md`). | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/trim-preferences.html (F) |
| Ripple trim adds edits to keep sync | partial | No one-step version, and only later shifts. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/trim-preferences.html (F) |
| Playhead determines trim loop | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/trim-preferences.html (F) |
| Dynamic timeline updates during ripple trim | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| Cache analysis results | missing | No analysis cache. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html (F) |
| Analyze imported media for visual search | missing | No visual search. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html (F) |
| Screen reader and magnifier support | partial | No stated accessibility commitment, and no testing with assistive technology. The picture itself has no text alternative beyond the attribution. | https://helpx.adobe.com/premiere/desktop/get-started/set-up-accessibility-features/screen-reader-screen-magnifier-and-operating-system-accessibility-support.html (F) |
| Audio descriptions at export | missing | No second audio track. | https://helpx.adobe.com/premiere/desktop/get-started/set-up-accessibility-features/accessibility-overview.html (F) |
| Program Monitor | partial | No live, interactive playback or scrubbing. Each look is a call that returns a still, a sheet or a file. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/play-active-sequence-in-program-monitor.html (F) |
| Source Monitor and source In/Out | partial | `frame` and `preview` take a project, not a media file, so viewing a source means writing a project that holds it. `probe` is CLI-only. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-in-and-out-points-in-the-source-monitor.html (F) |
| Play clips in the Source Monitor | partial | No transport and no variable speed. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/play-clips-in-source-monitor.html (F) |
| J-K-L shuttle | missing | No shuttle speeds or reverse play. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/play-active-sequence-in-program-monitor.html (F) |
| Open or clear clips in the Source Monitor | missing | *GUI-only.* No source viewer. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/open-or-clear-a-clip-in-the-source-monitor.html (F) |
| Navigate clips in the Source menu | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/navigate-clips-in-the-source-menu-in-the-source-monitor.html (F) |
| Monitor edit buttons | partial | Extract needs a ripple earlier, which `shift` cannot do (see `editing.md`). | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/about-source-monitor-and-program-monitor.html (F) |
| Playback and paused resolution | partial | The caller cannot choose the fraction. The ladder is fixed, and a caller-specified proxy resolution is only anticipated (`CONTEXT.md` *legibility floor*). | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-display-quality-for-the-source-and-program-monitors.html (F) |
| Display modes | partial | No alpha, channel or matte view as an image. `frame` returns a composited RGB still only. Multi-camera and VR are in `editing.md` and `vr.md`. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/choose-a-display-mode.html (F) |
| Magnification | partial | No zoom above true scale, and no picture of the pasteboard. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/adjust-the-magnification-for-the-source-and-program-monitors.html (F) |
| Rulers and unit of measurement | partial | Pixels only; no percent unit and no ruler drawn. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/change-the-unit-of-measurement.html (F) |
| Guides | partial | No guide drawn on a `frame` image and no saved guide lines. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/add-or-remove-guides.html (F) |
| Guide templates | missing | No guide files. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/save-export-and-import-custom-guides.html (F) |
| Snap in Program Monitor | partial | No guides to snap to. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/snap-objects-to-guides.html (F) |
| Overlay: marker comments | missing | No markers. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-overlay-settings.html (F) |
| Overlay: sound timecode and sound roll | missing | No sound timecode. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-overlay-settings.html (F) |
| Overlay settings and presets | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-overlay-settings.html (F) |
| Comparison View (Program Monitor) | partial | No single split or paired image, and no before/after: video `effects` have no bypass flag. Audio effects do have `enabled`. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/compare-edits-to-clips-in-the-program-monitor.html (F) |
| Comparison View (Color mode) | partial | No split image of graded against ungraded, and no reference marker. | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/about-comparison-view.html (F) |
| Trim mode two-up display | partial | No single two-up image of one cut. | https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/about-trim-mode.html (F) |
| Gang Source and Program | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/gang-source-and-program-monitor.html (F) |
| Button Editor | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/display-controls-in-the-source-and-program-monitors.html (F) |
| Interlaced display | missing | No field handling. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/set-display-options-for-interlaced-footage.html (F) |
| Time ruler | partial | No ruler on a monitor. | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/time-controls-in-the-source-and-program-monitors.html (F) |
| Zoom scroll bars | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/source-and-program-monitor-adjustments/time-controls-in-the-source-and-program-monitors.html (F) |
| Loop playback | missing | No loop, no live adjustment. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/play-back-in-a-continuous-loop.html (F) |
| Video on a second monitor | missing | No external video output. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/display-video-on-a-second-monitor.html (F) |
| Preview on a television monitor | missing | No external device, no export to tape, no 24p pulldown. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/previewing-on-a-television-monitor.html (F) |
| Interface on a second monitor | missing | *GUI-only.* | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/display-premiere-interface-on-a-second-monitor.html (F) |
| HDR display (DirectX HDR on Windows) | missing | No HDR viewing. | https://helpx.adobe.com/premiere/desktop/get-started/set-up-directx-hdr-for-windows/set-up-premiere-for-directx-hdr.html (F) |
| Render bar status | missing | No per-range report of what would play in real time, and none of what is already rendered. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-a-section-of-a-sequence.html (F) |
| Render Effects In to Out, Render In to Out | partial | A rendered span is a separate file. Nothing tracks it as a cached segment of the timeline. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-a-section-of-a-sequence.html (F) |
| Render Audio | partial | No audio-only preview file. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-a-section-of-a-sequence.html (F) |
| Preview files and Use Previews at export | missing | No persistent preview-segment cache, and no reuse at export. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/use-preview-files-when-rendering.html (F) |
| Delete preview files | missing | No preview cache to manage. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/delete-preview-files.html (F) |
| Smart rendering | missing | No pass-through of unchanged source. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/smart-rendering-supported-formats.html (F) |
| Render and Replace | partial | The flattened file is always H.264 (ADR-0143): lossy, with no alpha and no mezzanine format. | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/render-and-replace-media-in-a-sequence.html (F) |
| Mercury Playback Engine GPU acceleration | missing | No GPU rasterization. It is open, not refused. | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/enable-mercury-playback-engine-gpu-accelerated-renderer.html (F) |
| Multi-GPU rendering and export | missing | No GPU use at all (see above). | https://helpx.adobe.com/premiere/desktop/get-started/download-and-install/mercury-playback-engine-gpu-accelerated-in-premiere.html (F) |
| Hardware-accelerated decoding | missing | No hardware decode. No ADR refuses it. | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/enable-hardware-accelerated-decoding-support.html (F) |
| Media cache management | partial | No age or size limits, no automatic cleanup, no configurable location. | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/automatically-manage-your-media-cache-files.html (F) |
| Clear media cache | partial | No option to delete only entries whose source is gone. | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/clear-media-cache-using-preferences.html (F) |
| History panel | partial | No per-action state list within a session. Granularity is whatever the agent commits. | https://helpx.adobe.com/premiere/desktop/edit-projects/correct-mistakes/view-or-make-changes-in-the-history-panel.html (F) |
| Team Projects | partial | No JSON-aware merge driver: a merge is checked only after the fact. No presence, and no sequence lock. The remote instance (ADR-0159) is accepted but **not shipped in v0.2.0**: `rmcp` is built with `transport-io` only (`crates/montagent/Cargo.toml`), so there is no HTTP door or workspace. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/about-team-projects.html (F) |
| Team Projects auto-saves and cache location | partial | No per-edit save. Workspaces are not shipped in v0.2.0. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/specify-auto-save-cache-location.html (F) |
| Frame.io review and comments | missing | No review-link hosting. No timecoded comments on the video. No import of comments as markers. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/about-frameio.html (F) |
| Productions | partial | No project locking. No reference from one project into another: ADR-0001 refuses a nested clock, but ADR-0166 notes it does not refuse a shared space, so this is open. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/about-productions.html (F) |
| Creative Cloud Libraries | missing | No synced library. ADR-0002 rules out an *in-project* asset table, but not an external library. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/about-creative-cloud-libraries.html (F) |
| Plug-in panels and extensions | missing | *GUI-only.* No in-app UI to extend. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/install-plug-ins-and-extensions.html (F) |
| Codec and file-format plug-ins | partial | No plug-in interface. Output is fixed at H.264 via `libx264` (ADR-0143). | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| Intel macOS | partial | Its tests are not run (ADR-0188). | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/adobe-premiere-pro-technical-requirements.html (F) |
| Localised UI languages | missing | No localisation. | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/adobe-premiere-pro-technical-requirements.html (F) |
| System compatibility report | partial | Only the one external tool is checked; no system report to export. | https://helpx.adobe.com/premiere/desktop/get-started/download-and-install/check-if-your-system-is-compatible-with-premiere.html (F) |
| Long-term-support releases | missing | No LTS branch. | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |

### Workflow: collaboration and other apps: 25 partial, 36 missing

| Premiere feature | Status | What's missing in Montagent | Adobe page |
|---|---|---|---|
| **L1** Creative Cloud Libraries | missing | A named, synced asset library and a way to browse it. ADR-0002 rules out an *in-project* asset table, not an external library. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/about-creative-cloud-libraries.html (F) |
| **L2** Create a library | missing | Named collections of assets. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/add-a-creative-cloud-library.html (F) |
| **L3** Share a library, join a shared library | missing | Invitations and shared libraries. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/collaborate-with-other-creative-cloud-users.html (F) |
| **L4** Place a library graphic | partial | The library to place from (L1). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/use-assets-stored-in-creative-cloud-libraries.html (F) |
| **L5** Save a grade as a Look in a library | partial | A named, stored look, and a library to keep it in. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/add-looks-to-a-creative-cloud-library.html (F) |
| **P1** Productions | partial | No production object: every tool takes one project, and nothing knows which projects belong together. No panel. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/about-productions.html (F) |
| **P2** Build a production | partial | No duplicate check. Nothing rewrites relative paths when a project moves. No lock to respect (P4). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/add-projects-to-a-production.html (F) |
| **P3** Configure Production settings | missing | Settings shared by a group of projects. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/configure-productions-settings.html (F) |
| **P4** Project locking | missing | Any lock, and any record of who has a project open. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/change-project-lock-status-in-production.html (F) |
| **P6** Know a project changed under you | partial | No notification of another editor's change. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/change-project-lock-status-in-production.html (F) |
| **P8** Shared scratch disks and preview files | missing | Any cache a team can share. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/about-productions.html (F) |
| **P9** Cross-project clip referencing | partial | The source clip itself. A *master clip* other projects point at is the media catalogue ADR-0002 refuses (`import.md`). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/how-clips-markers-and-labels-work-in-a-production.html (F) |
| **P12** Reassociate source clips | partial | Nothing searches for the moved file or reconnects it. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/find-and-edit-source-clips-in-a-production.html (F) |
| **P13** Copy and move clips and sequences between projects | partial | A relative `source` resolves against the *destination* project's folder (ADR-0053), so a copy between folders needs its paths rewritten. Nothing does that. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/copy-clips-across-projects-in-a-production.html (F) |
| **P14** Markers shared between source clip and sequence | missing | Markers, and their sync. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/how-clips-markers-and-labels-work-in-a-production.html (F) |
| **P15** Label colours in a Production | partial | No label colour. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/how-clips-markers-and-labels-work-in-a-production.html (F) |
| **P16** Multicam source sequences in a Production | missing | Multicam. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/how-clips-markers-and-labels-work-in-a-production.html (F) |
| **P17** Link one project into another | missing | Any reference from one project to another. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/productions-faq.html (F) |
| **P19** Cross-platform paths | partial | An absolute path is never translated: on another machine it is a missing-source error (ADR-0053). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/general-best-practices-for-using-productions-on-shared-storage.html (F) |
| **T1** Team Projects | partial | No hosted service. The remote instance (ADR-0159) is accepted but not shipped in v0.2.0: `rmcp` is built with `transport-io` only (`crates/montagent/Cargo.toml`). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/about-team-projects.html (F) |
| **T2** Create or convert a Team Project | partial | No name, description or hosting in Montagent. Those belong to the git host. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/create-team-projects.html (F) |
| **T3** Invite and remove collaborators | missing | Invitations and membership. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/invite-individual-collaborators-or-groups-to-team-projects.html (F) |
| **T4** See who is online | missing | Presence. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/view-online-collaborators.html (F) |
| **T5** Sequence Locking | missing | Any lock, at project or sequence level. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/sequence-locking.html (F) |
| **T7** Visual cues during collaboration | missing | *GUI-only.* No in-tool indication. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/visual-cues-during-collaboration.html (F) |
| **T9** Offline conflicts | partial | No JSON-aware merge driver: the merge is checked only after it is done (`workflow.md` *Team Projects*). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/sequence-locking-for-offline-editing.html (F) |
| **T10** Auto save on every edit | partial | No automatic snapshot. A version exists only when someone commits (`workflow.md` *Auto Save*). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/specify-auto-save-cache-location.html (F) |
| **T11** Browse and restore auto saves | partial | No snapshots between commits (T10). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/view-auto-saves.html (F) |
| **T14** Media for a Team Project | partial | Nothing moves the media: the repo (or LFS, or a synced folder) is the team's choice, outside Montagent. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/add-media.html (F) |
| **T16** Archive, restore and delete Team Projects | missing | — (outside Montagent) | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/archive-team-projects.html (F) |
| **T17** Search the list of Team Projects | missing | A project list. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/search-and-filter-team-projects.html (F) |
| **F1** Frame.io panel | missing | A review service and its panel. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/about-frameio.html (F) |
| **F2** Bring media in from Frame.io | partial | Fetching the file: the agent downloads it and writes the local path. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/about-frameio.html (F) |
| **F3** Share for review | missing | Upload, hosting and a review link. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/share-for-review-with-frame-io.html (F) |
| **F4** Upload the active sequence | partial | The upload, the version stack and marker export. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/upload-media-to-frameio.html (F) |
| **F5** Version stacks | partial | No stack of uploaded versions. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/upload-media-to-frameio.html (F) |
| **F6** Frame-accurate review comments | missing | Timecoded comments and drawings on the video. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/add-comments-on-shared-projects.html (F) |
| **F7** Import comments as markers | missing | Markers, and a comment importer. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/import-comments-as-markers.html (F) |
| **F8** Export markers as comments | missing | Markers (F7). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/upload-media-to-frameio.html (F) |
| **F9** Frame.io collaborators | missing | — (a review service, F1) | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/add-collaborators-using-the-frameio-panel.html (F) |
| **F10** Upload bins and Camera to Cloud | missing | — (a review service, F1) | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaboration-in-premiere.html (F) |
| **F11** Branded presentations | missing | — (a review service, F1) | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaboration-in-premiere.html (F) |
| **F12** Workfront integration | missing | A project-management integration. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/share-for-review-using-frame-io/integrate-adobe-workfront-with-frameio.html (F) |
| **B1** Edit on a cloud machine | partial | The remote instance. No live video stream out (`workflow.md` *Multiple monitors*). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/collaborate-with-other-teams-and-team-members.html (F) |
| **B4** QC report as timeline markers | missing | Markers and a report importer. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/export-for-viewing-platforms.html (F) |
| **B5** Automated QC of the deliverable | partial | No broadcast-legal checks (levels, gamut) and no third-party QC report. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/export-for-viewing-platforms.html (F) |
| **X4** Codec and file-format plug-ins | partial | No plug-in interface. Output is fixed at H.264 via `libx264` (ADR-0143). | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| **X6** Workflow tools that read or write the edit | partial | No interchange format (FCP XML, AAF, EDL, OTIO; `export.md`). | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| **X7** Extension panels | missing | *GUI-only.* No in-app UI to extend. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| **A1** Copy clips from Premiere to After Effects | missing | No compositing app to send to, and no converter. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/copy-clips-from-premiere-to-after-effects.html (F) |
| **A2** Copy After Effects layers to Premiere | missing | A converter to editable elements. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/copy-from-after-effects-to-premiere.html (F) |
| **A9** Disable a clip | missing | An element-level bypass. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/make-a-dynamically-linked-composition-offline0.html (F) |
| **U1** Edit a clip in Audition | partial | No audio-only render of one element (`export.md`: no audio-only output), and no launch of an editor. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/edit-premiere-audio-clips-in-audition.html (F) |
| **U2** Send a sequence to Audition | missing | Any multitrack session export. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/edit-premiere-sequences-in-audition.html (F) |
| **U3** Transfer effects, pan and volume to Audition | missing | A mapping to a DAW's effects. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/transfer-of-information-from-premiere-to-audition.html (F) |
| **S1** Create a Photoshop file in a project | missing | Launching an image editor with a matching canvas. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/create-a-photoshop-file-in-a-project.html (F) |
| **S3** Import a Photoshop file, flattened or as layers | missing | PSD decode and layer extraction. The route is to export PNG. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/import-photoshop-files-into-premiere.html (F) |
| **K1** Search and license Adobe Stock in the app | missing | Search, licensing and download. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/add-assets-from-adobe-stock-to-a-creative-cloud-library.html (F) |
| **K2** Preview now, license later | partial | Licensing, and knowing which files are previews. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/add-assets-from-adobe-stock-to-a-creative-cloud-library.html (F) |
| **O1** Media from Firefly and Firefly Boards | missing | Receiving media from another app. | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |
| **O2** Premiere on iPhone to desktop | missing | A mobile app. | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |

## Refused: every row an ADR rules out

### Editing and timeline: 9 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Nest clips / nested sequences | Container clock refused ([ADR-0001](../../adr/0001-flat-element-list.md)), restated by [ADR-0166](../../adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md). Instance propagation is a copy construct ([ADR-0148](../../adr/0148-a-repeat-does-not-enter-and-a-stagger-enters-only-across-the-units-of-one-text-element.md)). | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-nested-sequences/about-nested-sequences.html (F) |
| Open a nest, reveal a clip in its source sequence | Depends on the nest refused above ([ADR-0001](../../adr/0001-flat-element-list.md), [ADR-0166](../../adr/0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md)). | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-nested-sequences/reveal-a-clip-in-a-nested-source-sequence.html (F) |
| Subclips | A named media entry is the asset table refused by [ADR-0002](../../adr/0002-inline-source-no-asset-table.md). The per-use range ships: `source_start`/`source_end` on each element (CONTEXT *Source range*). | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/create-a-subclip-from-the-project-panel.html (F) |
| Edit Subclip (media start and end) | Depends on the subclip refused by [ADR-0002](../../adr/0002-inline-source-no-asset-table.md). Each element's own range is edited directly. | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/adjust-media-start-and-end-times-of-subclip.html (F) |
| Convert subclip to master clip | Depends on the Project-panel asset refused by [ADR-0002](../../adr/0002-inline-source-no-asset-table.md). | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/convert-subclip-to-master-clip.html (F) |
| Linked audio under time remapping | [ADR-0157](../../adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md) refuses the desync: a remapped `video` must carry `volume: 0` (`E-REMAP-AUDIBLE`). | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-and-duration-using-time-remapping.html (unconfirmed: the fetched page does not mention audio) |
| Generative Extend | Montagent "contains no model" (CONTEXT *Montagent*), and a model inside the tool is refused ([ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md)). Footage generated elsewhere enters as an ordinary `video`. | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-extend-overview.html (F) |
| Generative Media Tool (26.5) | Same as Generative Extend ([ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md)). | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-media-tool-overview.html (F) |
| Timecode display formats | Time is absolute integer milliseconds with no second representation ([ADR-0005](../../adr/0005-absolute-integer-milliseconds.md)). CONTEXT *Timeline range* lists "timecode" under avoid. | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/change-timecode-display-format.html (F) |

### Video effects and transitions: 21 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Source clip effects | ADR-0002: no asset table, and each element names its file inline | [SRC][SRC], [ASRC][ASRC], [FX][FX] (F) |
| Source-clip effects shown on Project panel thumbnails | ADR-0002 (no media catalogue, so no thumbnails to decorate) | [THB][THB] (F) |
| Third-party effect and transition plug-ins | ADR-0040 (vocabulary is "never a plugin architecture"), ADR-0017 (closed schema), ADR-0156 §8 (an open shader or script stays refused) | [TYP][TYP], [PLG][PLG], [RN][RN] (F) |
| Effect Manager | ADR-0040, ADR-0017: no plug-ins to manage; the vocabulary is closed | [MGR][MGR] (F) |
| Compound Blur (was "FI: Compound Blur FX") | ADR-0150 §4: a matte or map taken from another element is refused | [L][L], [MF][MF] (F) |
| Look-up tables (Lumetri Input LUT and Look) | ADR-0040 ("semantics live entirely in an external `.cube` file … a plugin in all but name"), ADR-0049 | [LUT][LUT] (F) |
| Corner Pin | ADR-0167: "a camera, depth sorting, skew and corner pin stay refused"; swivel/tilt cover rigid turns | [DIS][DIS], [MF][MF], [L][L] (F) |
| Channel Mix (was "FI: Channel Mix FX"; moved here from Tools) | ADR-0049 excludes per-channel colour matrices | [L][L], [MF][MF] (F) |
| Track Matte Key | ADR-0150 §4 refuses a matte from another element; the element's own `mask` is the documented substitute | [KEY][KEY], [L][L] (F) |
| Gradient Wipe (effect; current) | ADR-0150 §4: a matte from another element is refused | [TE][TE], [L][L], [ACC][ACC] (F) |
| Calculations | ADR-0150 §4: reading another element's pixels | [L][L], [RM][RM] (F) |
| Camera View | ADR-0167: "a camera … stay[s] refused" | [L][L], [RM][RM] (F) |
| Channel Mixer | ADR-0049 excludes per-channel colour matrices | [L][L], [RM][RM] (F) |
| Compound Arithmetic | ADR-0150 §4 | [L][L], [RM][RM] (F) |
| Difference Matte | ADR-0150 §4: a matte computed from another element's pixels | [L][L], [RM][RM] (F) |
| Luma Curves | ADR-0049: "Curves, LUTs, per-channel colour matrices, and duotone-style two-colour maps are categorically excluded" | [L][L], [RM][RM] (F) |
| RGB Curves | ADR-0049 (curves excluded) | [L][L], [RM][RM], [RGBC][RGBC] (F) |
| Hue Saturation Curves (Lumetri) | ADR-0049 (curves excluded) | [HSC][HSC], [CC][CC] (F) |
| Set Matte | ADR-0150 §4: a matte from another element is refused | [L][L], [RM][RM] (F) |
| Texturize | ADR-0150 §4: reads another element's pixels | [L][L], [RM][RM] (F) |
| Spring Motion | ADR-0146 refuses spring easing (a cubic bezier with overshoot is the documented substitute) | [L][L], [AN][AN], [MT][MT] (F) |

### Motion, keyframing and compositing: 35 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Mask Expansion | ADR-0152 §3: "a second knob for one edge" | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-mask-properties.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html (F) |
| Mask Opacity | ADR-0152 §3: "Not until a case asks" | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/adjust-mask-properties.html (F) |
| Mask with Shape (graphic as matte) | ADR-0152 §3 (text and shape sources refused inside `mask`); ADR-0150 §4 (matte from another element) | https://helpx.adobe.com/premiere/desktop/add-video-effects/create-masks-and-composites/create-reveal-animations-using-masking-techniques.html (F) |
| Dissolve | ADR-0147 (wider vocabulary refused for now) | (above, F) |
| Darken | ADR-0147 (`darken` named) | (above, F) |
| Color Burn | ADR-0147 (`color_burn` named) | (above, F) |
| Linear Burn | ADR-0147 | (above, F) |
| Darker Color | ADR-0147 | (above, F) |
| Lighten | ADR-0147 (`lighten` named) | (above, F) |
| Color Dodge | ADR-0147 (`color_dodge` named) | (above, F) |
| Lighter Color | ADR-0147 | (above, F) |
| Soft Light | ADR-0147 (`soft_light` named) | (above, F) |
| Hard Light | ADR-0147 (`hard_light` named) | (above, F) |
| Vivid Light | ADR-0147 | (above, F) |
| Linear Light | ADR-0147 | (above, F) |
| Pin Light | ADR-0147 | (above, F) |
| Hard Mix | ADR-0147 | (above, F) |
| Difference | ADR-0147 (`difference` named) | (above, F) |
| Exclusion | ADR-0147 | (above, F) |
| Subtract | ADR-0147 | (above, F) |
| Divide | ADR-0147 | (above, F) |
| Hue | ADR-0147 | (above, F) |
| Saturation | ADR-0147 | (above, F) |
| Color | ADR-0147 | (above, F) |
| Luminosity | ADR-0147 | (above, F) |
| Track Matte Key | ADR-0150 §4: "A matte from another element is refused" | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/keying-effects.html (F) |
| Basic 3D: Distance to Image | ADR-0167 "Considered": a second size control; `perspective` keeps CSS's meaning | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/perspective-effects.html (F) |
| Corner Pin | ADR-0167 "Skew, corner pin, a camera, depth sorting. Still refused." | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/distort-effects.html (F) |
| Transform effect: Skew / Skew Axis | ADR-0012, ADR-0040 ("Skew — rejected on model grounds"), ADR-0167 | unconfirmed (nearest: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html (F)) |
| Transform effect (a second transform in the effect stack) | One transform per element, "never nested, never composed" (CONTEXT.md **Transform**, ADR-0012) | https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/transform-effects.html (F); https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/types-of-effects.html (F) |
| Nested sequence's local clock | ADR-0001 clock refusal, restated by ADR-0166 §2 | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-nested-sequences/about-nested-sequences.html (F) |
| Dynamic Link: create new After Effects composition (R1) | No external-app source; ADR-0145 bars AE as a reason to enter | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/create-a-dynamically-linked-composition.html (F) |
| Dynamic Link: replace clips with an After Effects composition (R1) | — | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/replace-clips-with-a-dynamically-linked-after-effects-composition.html (F) |
| Dynamic Link: import an After Effects composition (R1) | `skills/montagent-prerender`: a Lottie export of an AE comp is pre-rendered to footage and placed as a `video` (ADR-0171) | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/import-an-after-effects-composition.html (F) |
| Object mask expansion | ADR-0152 §3 refuses Expansion on `mask` ("a second knob for one edge") | https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html (F) |

### Colour: 25 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Input LUT (Basic Correction) | ADR-0040 *Out of scope*: "LUTs — semantics live entirely in an external `.cube` file, which is a plugin in all but name". ADR-0049: "Curves, LUTs, per-channel colour matrices … are categorically excluded". | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/basic-color-correction-options.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/add-look-up-tables.html (F) |
| Look (preset) | A Look is a LUT (the Creative folder of *Install custom LUT files* populates it). ADR-0040 and ADR-0049 refuse LUTs. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F) |
| Look Intensity | Falls with the look (ADR-0040, ADR-0049). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F) |
| RGB Curves: master (luma) curve | ADR-0049: "a curve or a LUT is not a fixed parameter set, it is an arbitrary function", so curves are "categorically excluded". `CONTEXT.md` **Colour filter** *Avoid*: "curve". | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-rgb-curves.html (F) |
| RGB Curves: per-channel R, G, B | ADR-0049 (curves, and per-channel colour matrices). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-rgb-curves.html (F) |
| RGB Curves effect (Effect Controls) | ADR-0049 (curves). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-rgb-curves.html (F) |
| Hue vs Sat curve | ADR-0049 (curves). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html (F) |
| Hue vs Hue curve | ADR-0049 (curves). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html (F) |
| Hue vs Luma curve | ADR-0049 (curves). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html (F) |
| Luma vs Sat curve | ADR-0049 (curves). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html (F) |
| Sat vs Sat curve | ADR-0049 (curves). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html (F) |
| Curve eyedropper (three control points from a sample) | Falls with the curves (ADR-0049). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/correct-color-using-hue-and-saturation-curves.html (F) |
| Mask expansion | ADR-0152 §3 *What does not enter*: "Expansion. It moves the same edge the explicit mask rect already moves with literal numbers." | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html (F), https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html (F) |
| Mask opacity | ADR-0152 §3: "Mask opacity … No case asks for it, and the element's `opacity` with a second element covers what is near. Not until a case asks." | https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/isolated-adjustments-using-masks.html (F), https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/refining-and-combining-masks.html (F) |
| Creative LUT / custom look file | ADR-0040 (external `.cube` is "a plugin in all but name"). ADR-0049. ADR-0017 (closed schema, no escape hatch). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/apply-lumetri-looks.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/install-custom-look-up-table-files.html (F) |
| Install custom LUT files (Input, Output, Creative, Technical folders) | Falls with LUTs (ADR-0040, ADR-0049). | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/install-custom-look-up-table-files.html (F) |
| Source-clip Input LUT in colour management (Modify Clip > Color) | ADR-0040, ADR-0049. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-using-clip-modify.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Look LUT colour-space declaration | Falls with LUTs (ADR-0040, ADR-0049). | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/lumetri-enhancements.html (F) |
| Lumetri adjustments applied at export (Export > Effects) | The export-time Lumetri adjustment is a look or LUT applied to the output. ADR-0040, ADR-0049, and ADR-0143 §5 exposes no export options. | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html (F) |
| 3D LUT interpolation preference | No LUTs (ADR-0040, ADR-0049). | https://helpx.adobe.com/premiere/desktop/correct-color/color-correction-fundamentals/color-settings.html (F) |
| Colour management through Dynamic Link to After Effects (R1) | No After Effects interchange. It belongs to the workflow-collab area. | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/premiere-after-effects-color-management-compatibility.html (F), https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/about-color-management.html (F) |
| ASC CDL | Read against ADR-0049: per-channel slope and offset form a per-channel colour matrix, which is excluded by name. Master slope and offset already compose from `contrast`+`brightness`. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html (F) |
| Tint (effect: map black to a colour and white to a colour) | ADR-0049: "duotone-style two-colour maps are categorically excluded". Montagent's `tint` is the one-colour lerp. | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/available-color-correction-effects.html (F) |
| Channel Mix FX (Channel Mixer) | ADR-0049: "per-channel colour matrices … are categorically excluded". | https://helpx.adobe.com/premiere/desktop/add-video-effects/types-of-effects/effects.html (F) |
| Composite in linear colour | ADR-0147 *Considered and refused*: "Linear-light arithmetic. Refused: it differs from the reference editors' default and from what an agent expects 'screen' to do." | https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html (F) |

### Titles, graphics and text: 6 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Paragraph text in a text box | ADR-0007 ("No fit-to-box. No automatic wrapping."), ADR-0008 (line breaks belong to the agent); `measure` returns the extent | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/use-color-fonts.html |
| Auto-sizing text box | ADR-0007 ("No fit-to-box") | — |
| Faux Bold | ADR-0007 ("No `weight`, no `bold`": synthetic emboldening is renderer-specific; the schema must reject it); ADR-0014 | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/parameters-when-applying-as-linked-style-or-track-style.html |
| Adobe Fonts | ADR-0007 (a font is a file the project declares, never a system family; the renderer never consults system fonts); ADR-0057 (vendoring behind a licence gate) | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/replace-fonts.html |
| Mask with Text | ADR-0152 (a text or shape source is refused inside `mask`); ADR-0150 (a matte from another element is refused) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/mask-with-text-or-shape-using-properties-panel.html |
| Responsive Design: pin to video frame | ADR-0012 refused fractions for placement: placement is integer pixels, so a retarget is a re-layout, not a stretch | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-images-and-graphics/create-responsive-graphics.html |

### Captions, subtitles and transcripts: 17 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Speech to Text (auto transcription) | ADR-0051 (Montagent does not invoke, ship or shell out to a forced aligner: a model in the tool breaks reproducibility); `CONTEXT.md` **Montagent** "contains no model"; ADR-0054 calls speech classification a constitutional exclusion | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/auto-transcribe-video-using-speech-to-text.html · https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html (F) |
| Transcribe only some audio (audio analysis) | ADR-0051, ADR-0054: there is no recognition step whose input could be chosen | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/auto-transcribe-video-using-speech-to-text.html (F) |
| Multichannel audio in transcription | ADR-0051, ADR-0054 (as above) | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/multichannel-audio-support-in-text-based-editing.html (F) |
| Transcribe individual source files | ADR-0051, ADR-0054 | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/transcribe-individual-source-files.html (F) |
| Transcription language selection and language packs | ADR-0051, ADR-0054: no recognition model, so no recognition languages | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/download-language-packs-from-within-premiere.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/languages-supported-by-speech-to-text.html (F) |
| Language packs from the Creative Cloud app | ADR-0051, ADR-0054 (as above) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/download-language-packs-from-creative-cloud.html (F) |
| Language packs deployed for Teams and Enterprise | ADR-0051, ADR-0054 (as above) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/language-packs-for-teams-and-enterprise-users.html (F) |
| Language auto-detection and default language | ADR-0051, ADR-0054: telling languages apart is classification | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html (F) |
| Auto-transcribe on import | ADR-0051 | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/transcribe-video.html · https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-captions.html (F) |
| Transcription cache | ADR-0051: nothing is transcribed, so nothing is cached | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html (F) |
| Voice Activity Detection for speech to text (26.5) | ADR-0051, ADR-0054 for the model. ADR-0054 names ffmpeg `silencedetect` as the deterministic, non-model route, which `skills/montagent-footage/scripts/captions.py` uses | https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html (F) |
| Transcribe In-to-Out range, merge into an existing transcript | ADR-0051 | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/auto-transcribe-video-using-speech-to-text.html (F) |
| Speaker labelling (diarization) | ADR-0054 (telling voices apart is classification, "permanently out of reach" under no-model); ADR-0051 | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html · https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/transcribe-individual-source-files.html (F) |
| Filler-word detection and bulk delete | Recognising a filler word is speech recognition: ADR-0051, ADR-0054 | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html · https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/detect-and-delete-pauses-in-transcripts.html (F) |
| Remove all instances of one speaker | Selection depends on speaker labels, which are diarization: ADR-0054, ADR-0051 | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/remove-all-instances-of-one-speaker-in-transcript.html (F) |
| Mask with Text | ADR-0152 refuses a text source inside `mask`; ADR-0150 refuses a matte from another element | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-styles-for-captions.html (F) |
| Translate captions | ADR-0051 and ADR-0054 (no model in Montagent); `CONTEXT.md` **Agent**: Montagent never calls an agent | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html · https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html (F) |

### Audio: 28 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Adaptive Noise Reduction ‡ | Adaptive (detector-driven) processing: ADR-0145 literal values, per #795 Out of scope. | [obsolete-ts][obsolete-ts] (F) |
| DeReverb | Model/detector-driven: #795 Out of scope, ADR-0145 literal values. | [lib][lib] (F) |
| Convolution Reverb | #795 Out of scope: "an external reference, not a literal parameter set" (ADR-0145 literal values). | [lib][lib] (F) |
| Surround Reverb ‡ | Impulse-file convolution: ADR-0145 literal values, as Convolution Reverb (#795 Out of scope). The 5.1 bus is also missing (ADR-0077). | [lib][lib] (F) |
| Third-party audio plug-ins (VST, VST3, AU) | ADR-0145 closed-vocabulary and checkable-by-`validate` invariants; `audio_effects` is a closed union keyed on `name` (ADR-0169). | [plugins][plugins], [prefs][prefs] (F) |
| Exponential Fade | ADR-0176 §1, Rejected: "A third curve, `exponential` … no rule an agent can pick by"; the door is kept open. | [xfade][xfade] (F) |
| Audio types: Dialogue, Music, SFX, Ambience | ADR-0178 §2 rejects "a `role` or `"bed"` token … a second vocabulary and a hidden default". | [esp][esp], [concepts][concepts] (F) |
| Audio Category Tagging (Auto Tag) | No type vocabulary (ADR-0178 §2); a model-driven classifier fails ADR-0145 literal values (#795 Out of scope). | [tag][tag] (F) |
| Essential Sound presets | ADR-0178 §2: "per-type presets, which #816 already refused"; every member parameter is required, nothing defaults (ADR-0179, 0180). | [presets][presets], [esp][esp] (F) |
| Repair > Reduce Reverb | As DeReverb, section E (#795, ADR-0145). | [repair][repair] (F) |
| Clarity > Enhance Speech | "AI enhance voice" is #795 Out of scope (ADR-0145 literal values). | [enh][enh], [enh-req][enh-req] (F) |
| Track keyframes (Volume, Mute, Panner, track-effect parameters) | Track-level audio: #795 court, ADR-0172, ADR-0004 (tracks own nothing). | [trackvol][trackvol], [trackfx-tl][trackfx-tl] (F) |
| Reverse speed with audio ‡ | Reverse is only a `source_time` remap, and a remapped `video` must carry `volume: 0` (`E-REMAP-AUDIBLE`, ADR-0157). | [speed][speed] (F) |
| Audio Track Mixer: track fader | Track-level audio: #795 court; ADR-0172; C **Master stage** ("not a track and not a bus"). | [atm][atm] (F) |
| Audio Track Mixer: track pan/balance | Track-level audio: #795 court; ADR-0172; ADR-0004. | [pan][pan], [panctl][panctl] (F) |
| Audio Track Mixer: effect slots (inserts) ‡ | "Track-level audio effects" are #795 Out of scope; ADR-0172. | [atm][atm], [atm-fx][atm-fx] (F) |
| Copy and move track effects (added) | Track-level audio effects (#795; ADR-0172). | [atm-copy][atm-copy] (F) |
| Copy and paste track-effect keyframes (added) | Track keyframes are track-level audio (#795; ADR-0172). | [fx-paste][fx-paste] (F) |
| Audio Track Mixer: automation modes | Track keyframes are track-level audio (#795; ADR-0172). | [atm-auto][atm-auto], [atm][atm], [rec-mix][rec-mix] (F) |
| Automatch Time (added) | It times track automation, which is track-level audio (#795; ADR-0172). | [automatch-time][automatch-time], [prefs][prefs] (F) |
| Mute / Solo track buttons ‡ | A track carries no audio state (ADR-0004; #795; ADR-0172) and the format has no `mute` field (ADR-0055). | [rec-atm][rec-atm], [relnotes][relnotes] (F) |
| Submix tracks | ADR-0172: the master is "not a general bus with an effect chain"; C **Master stage**: "not a track and not a bus"; track-level audio refused (#795). | [submix][submix], [atm][atm] (F) |
| Sends (pre/post fader) to submixes | As submixes: ADR-0172 (master "not a general bus"); #795 track-level audio. | [atm][atm], [prepost][prepost], [trackout][trackout] (F) |
| Mix (master) track effects | ADR-0172 Rejected: "An `audio_effects` list on the master … the stage must be a closed set". | [lm-apply][lm-apply], [ambi][ambi] (F) |
| Record audio mix changes | Track automation is track-level audio: #795; ADR-0172; ADR-0004. | [rec-mix][rec-mix] (F) |
| Separate Crosstalk (added) | Model-driven audio: ADR-0145 literal values (#795 Out of scope). | [crosstalk][crosstalk], [crosstalk-ai][crosstalk-ai] (F) |
| Generative Extend (audio) ‡ | Model-driven audio: ADR-0145 literal values (#795 Out of scope). | [genext][genext] (F) |
| Generative Media Tool: sound effects (26.5) (added) | Model-driven audio: ADR-0145 literal values (#795 Out of scope). A file generated elsewhere can still be a `source`. | [whatsnew][whatsnew], [relnotes][relnotes] (F) |

### Import, formats and media handling: 21 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| **Camera raw ›** ARRI AMIRA colour LUTs on import | LUTs are deliberately absent from the effect vocabulary (ADR-0040, ADR-0049), as in the colour file (#886) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html (F) |
| **Layered & vector ›** Illustrator AI / EPS | ADR-0027: vector sources out of scope (amended by ADR-0171) | https://helpx.adobe.com/premiere/desktop/organize-media/import-files/import-photoshop-and-illustrator-files.html (F) |
| **Layered & vector ›** Vector Motion (continuous rasterisation of vector layers) | ADR-0027 / ADR-0171: no vector source enters | https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/edit-vector-graphics-using-vector-motion-effect.html (F) |
| **Layered & vector ›** SVG | ADR-0171 §1 refuses SVG as a source; `montagent-prerender` SVG route rasterises once to a PNG `image` | unconfirmed (nearest: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/supported-file-formats.html, which omits it) |
| **Interpret footage ›** Input LUT per clip | LUTs are deliberately absent from the effect vocabulary (ADR-0040, ADR-0049), as in #886 | https://helpx.adobe.com/premiere/desktop/correct-color/set-up-color-management/color-management-using-clip-modify.html (F) |
| **Import & ingest ›** Import After Effects compositions (Dynamic Link) (R1) | `montagent-prerender` takes Lottie or code, not AE projects | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/import-after-effects-compositions.html (F) |
| **Proxies ›** Use proxies on export | ADR-0186: the deliverable (`render`, `frame`, `preview --full`) always paints true pixels | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/export-proxies.html (F) |
| **Offline & relink ›** Media Offline placeholder in playback and export | ADR-0093: an `error` withholds the deliverable; ADR-0053: missing source is an `error` | https://helpx.adobe.com/premiere/desktop/organize-media/ingest-proxy-workflow/convert-online-clips-to-offline-clips.html (F) |
| **Organising ›** Project panel (catalogue of master clips) | ADR-0002: no asset table; ADR-0053: no `assetRoot` | https://helpx.adobe.com/premiere/desktop/organize-media/create-projects/start-projects.html (F) |
| **Organising ›** Bins and nested bins | ADR-0002 (no catalogue to hold bins) | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/bins-overview.html (F) |
| **Organising ›** Add and delete bins | ADR-0002 | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/add-and-delete-bins.html (F) |
| **Organising ›** Open and close bins | ADR-0002 | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/open-and-close-bins.html (F) |
| **Organising ›** Bin behaviours | ADR-0002 | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/change-bin-behaviors.html (F) |
| **Organising ›** Sort and view bins (List, Icon, Freeform) | ADR-0002 | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/organize-bins.html (F) |
| **Organising ›** Freeform view (storyboarding in the Project panel) | ADR-0002 (no catalogue items to arrange) | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-freeform-view-in-project-panel.html (F) |
| **Organising ›** Consolidate duplicates | ADR-0002: no catalogue, so no duplicate items | https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/consolidate-duplicate-folders.html (F) |
| **Organising ›** Subclips | ADR-0002: no named indirection; each element carries its own `source_start`/`source_end` | https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/create-a-subclip-from-the-project-panel.html (F) |
| **Timecode ›** Timecode display formats | Time is absolute integer milliseconds with no second representation (ADR-0005); CONTEXT *Timeline range* lists "timecode" under avoid | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/change-timecode-display-format.html (F) |
| **Timecode ›** Frame count (start at 0, at 1, or timecode conversion) | ADR-0005: milliseconds only; frame numbers are derived, not stored | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/choose-timecode-display-format.html (F) |
| **Timecode ›** Timecode panel and display options | ADR-0005 (no timecode representation); there is no playhead | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/timecode-display-options.html (F) |
| **Timecode ›** View sequence timecode | ADR-0005 | https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/view-sequence-timecode.html (F) |

### Export, delivery and interchange: 11 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Hardware encoding | ADR-0143 §2: VideoToolbox "is not a default and not an option"; any future hardware-encode option needs its own ADR with quality evidence | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| Use Proxies | ADR-0067 ("Nothing about `render` changes. Proxy degradation has never applied to it"); ADR-0186 (the true-pixel sampling rule "belongs to the deliverable") | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Bitrate encoding (CBR / VBR 1-pass / 2-pass), target and maximum bitrate | ADR-0143 §5: no project field, CLI or MCP option for encoder, preset, CRF or threads | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html |
| Profile, level, key-frame distance | ADR-0143 §5 (same) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/video-encoding-settings.html |
| Maximum File Size | ADR-0143 §5: rate control is an encoder setting with no public knob | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Save a custom export preset | ADR-0143 §5: an encoder setting "is not a property of the composition", and a public knob is refused | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/create-and-save-custom-presets.html |
| Preset Manager; import/export presets | ADR-0143 §5 (as above: there is no preset to share) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/preset-manager-overview.html |
| Manage presets (sort, favourite, search, delete) | ADR-0143 §5 (as above) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/manage-presets.html |
| Parallel encoding | ADR-0108 §3: one encode slot; raising the bound above one requires a per-output-path lock it chose not to build | https://helpx.adobe.com/media-encoder/desktop/encoding-quick-start-and-basics/preferences.html |
| Lumetri Look / LUT on export | ADR-0040 ("LUTs: semantics live entirely in an external `.cube` file, which is a plugin in all but name"); ADR-0049's stopping rule excludes curves and LUTs | https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/export-settings-reference.html |
| Save / export a LUT from a grade | ADR-0040, ADR-0049 (same refusal; no LUT concept to write) | https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/save-or-export-look-up-tables.html |

### AI and automation: 4 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Mask Expansion (26.5) | [ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md) §3, "What does not enter": "Expansion. It moves the same edge the explicit mask rect already moves with literal numbers." | [Object masking][objmask] (F), [Sharp and Smooth modes][objmask-modes] (F) |
| Enhance Speech (R5) | Conventional repair ships: `highpass`, `lowpass`, `shelf`, `bell` EQ ([ADR-0179](../../adr/0179-eq-is-four-stackable-audio-effects-built-from-butterworth-biquad-sections.md)) and `compressor`/`limiter` ([ADR-0180](../../adr/0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md)) in `audio_effects`. Mix Amount can be built from two elements (the original and an externally enhanced file) at chosen `volume`s. | [Apply Enhance Speech][enhance] (F), [technical requirements][enhance-req] (F), [running slow][enhance-slow] (F), [Essential Sound panel][esp] (F) |
| Auto Ducking: trigger detected from the audio (Sensitivity) | [ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) §2: "No silence detection. It would put a signal-analysis judgement, with thresholds the file never records, inside the write path." The script refuses to guess from the voice element when it has no words or spans. | [Automatically duck audio][duck] (F) |
| Automatic transcription on import | [ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md): Montagent "does not invoke, ship, or shell out to a forced aligner". There is also no import step: a source is a path on an element ([ADR-0002](../../adr/0002-inline-source-no-asset-table.md)). | [preferences][stt-prefs] (F), [Transcribe video][tbe-transcribe] (F), [Create captions][captions] (F) |

### Workflow: interface, monitors, preferences, render: 15 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| Project panel as a media catalogue | ADR-0002: elements name their files inline and there is no asset table. A file is "in" the project only where an element uses it. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customization-options-for-the-project-panel.html (F) |
| Custom metadata properties | ADR-0017: an undefined key is an error, and third-party annotation belongs in a sidecar file in the repo, never in the project. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/customize-list-view-in-project-panel.html (F) |
| Column: Name (project-only name) | ADR-0002: no asset record to carry a name. An element's `id` names the element, not the file. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/list-view-columns.html (F) |
| Column: Label | ADR-0017: annotation that is not input to rendering lives in a sidecar. The free `group` string groups elements, not files. | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/list-view-columns.html (F) |
| Columns: logging fields | ADR-0017 (sidecar, not project). | https://helpx.adobe.com/premiere/desktop/get-started/customize-the-project-panel/list-view-columns.html (F) |
| Bins double-click behaviour | ADR-0002: no asset table, so no bins. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/general-preferences.html (F) |
| Label colours and label defaults | ADR-0017: organising annotation lives in a sidecar file, never in the project. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/labels-preferences.html (F) |
| Default paragraph direction | ADR-0133 §2: a line's base direction is its own characters', and a paragraph- or element-level direction field is not added. A run's `dir` is an isolate only. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/graphics-preferences.html (F) |
| ProRes hardware encoding preference | ADR-0143 §2: no hardware encoder is a default or an option; any future one needs its own ADR with quality evidence. Output is H.264 only. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/media-preferences.html (F) |
| Automatic transcription | ADR-0051: word timing is produced by a step the agent runs outside Montagent, which does not invoke, ship or shell out to an aligner. ADR-0048 keeps model output out of the render path. | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html (F) |
| Reveal Nested Sequence | ADR-0001: one flat element list with no nested clock (ADR-0166 keeps that refusal for nesting). | https://helpx.adobe.com/premiere/desktop/render-and-export/render-sequences-for-playback/match-a-frame-with-its-source.html (F) |
| Hardware encoding | ADR-0143 §2: *"`h264_videotoolbox` is not a default and not an option"* (no CRF, VMAF loss, macOS-only, determinism unproven). *"Any future hardware-encode option needs its own ADR with quality evidence."* | https://helpx.adobe.com/premiere/desktop/get-started/technical-requirements/enable-hardware-encoding-support.html (F) |
| Third-party video effect and transition plug-ins | ADR-0003: effects are *"a closed, named vocabulary — not an effect stack or a plugin system"*. ADR-0017 (no open shader, script or escape hatch), reaffirmed by ADR-0156. ADR-0145 *closed vocabulary*: *"A path may name media to decode, never a program to run."* A code-drawn look comes in as pre-rendered footage (`montagent-prerender`). | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| Audio plug-ins (VST, VST3, Audio Units) | Audio effects are a closed list in the schema (ADR-0169, ADR-0179–0185). Plug-ins are refused under ADR-0003, ADR-0017 and ADR-0145's closed vocabulary. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| Automatic updates | ADR-0064: *"Update policy: passive"*. *"No update-check code, no telemetry, no hardcoded 'latest version' endpoint, ever, in v1."* | https://helpx.adobe.com/premiere/desktop/get-started/download-and-install/best-practices-for-updating-premiere.html (F) |

### Workflow: collaboration and other apps: 12 refused

| Premiere feature | Refused by (Montagent evidence) | Adobe page |
|---|---|---|
| **L6** Apply a Look from a library | ADR-0040: *"LUTs — semantics live entirely in an external `.cube` file, which is a plugin in all but name"*. ADR-0049 refuses curves and LUTs from the colour vocabulary. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-creative-cloud-libraries/use-assets-stored-in-creative-cloud-libraries.html (F) |
| **P10** Source-clip effects | ADR-0002: no asset table, so there is no shared object to hold an effect. Each element carries its own literal `effects` (ADR-0040), and copying a list is an exact-string edit (ADR-0145). | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-productions/find-and-edit-source-clips-in-a-production.html (F) |
| **T15** Local-only media mapping | ADR-0053 rejects `assetRoot`: anything that changes how a `source` resolves, away from the element, is *"the asset-table indirection ADR-0002 rejected"*. A per-machine mapping is that same indirection, kept outside the file. | https://helpx.adobe.com/premiere/desktop/collaborate-with-others/collaborate-using-team-projects/manage-media.html (F) |
| **X1** Install plug-ins from the Marketplace | ADR-0003: effects are *"a closed, named vocabulary — not an effect stack or a plugin system"*. ADR-0017 (no open shader, script or escape hatch), reaffirmed by ADR-0156. ADR-0145 *closed vocabulary*: *"A path may name media to decode, never a program to run."* | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/install-plug-ins-and-extensions.html (F) |
| **X2** Third-party video effect and transition plug-ins | As X1. A look outside the `effects` list comes in as pre-rendered footage: drawn in code, encoded once, placed as a `video` (ADR-0156 §7, `montagent-prerender`). | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| **X3** Audio plug-ins: VST, VST3, Audio Units | Audio effects are a closed list in the schema (`highpass`, `lowpass`, `shelf`, `bell`, `compressor`, `normalize_loudness`, `limiter`; ADR-0169). Plug-ins are refused under ADR-0003, ADR-0017 and ADR-0145. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| **X5** Title and caption plug-ins | As X1. Text is the `text` element's closed vocabulary (ADR-0007). Caption files are `captions.md`'s rows. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html (F) |
| **A3** Import an After Effects composition (Dynamic Link) | ADR-0145 *closed vocabulary*: *"A path may name media to decode, never a program to run."* A linked composition is drawn by After Effects at Premiere's request. ADR-0171 refuses SVG and Lottie as sources, and pre-renders them through the skill instead. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/import-after-effects-compositions.html (F) |
| **A4** New linked composition | As A3. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/create-a-dynamically-linked-composition.html (F) |
| **A5** Replace clips with a linked composition | As A3. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/replace-clips-with-a-dynamically-linked-after-effects-composition.html (F) |
| **A6** Edit Original | As A3. Editing a *media* file outside and having it picked up is `have` (S2). | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/modify-a-dynamically-linked-composition-in-after-effects.html (F) |
| **A7** Take offline, relink or delete a linked composition | As A3: there is no linked composition to manage. For ordinary media, a moved file is an `E-SOURCE-MISSING` error (ADR-0053), relinking is a path edit, and deleting is removing the element. | https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/make-a-dynamically-linked-composition-offline.html (F) |

## Pages that describe no feature

These census pages are troubleshooting, crash, known-issue and preference-reset pages. Each one names a defect or a recovery step, not a capability, so none gets an inventory row.

| Page | URL |
|---|---|
| Enhance Speech is running slow | https://helpx.adobe.com/premiere/desktop/troubleshooting/audio-issues/enhance-speech-is-running-slow.html |
| Playback issues due to missing sound from all clips | https://helpx.adobe.com/premiere/desktop/troubleshooting/audio-issues/playback-issues-due-to-missing-sound-from-all-clips.html |
| Playback issues due to missing sound from some of the clips | https://helpx.adobe.com/premiere/desktop/troubleshooting/audio-issues/playback-issues-due-to-missing-sound-from-some-of-the-clips.html |
| Search for clips without sound | https://helpx.adobe.com/premiere/desktop/troubleshooting/audio-issues/search-for-clips-without-sound.html |
| Fix outdated Windows Libraries | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/fix-outdated-windows-libraries.html |
| Premiere is crashing | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/fix-premiere-crash-issues.html |
| Premiere is crashing | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/fix-premiere-pro-crash-issues.html |
| No splash screen appears on app launch | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/no-splash-screen-appears-on-app-launch.html |
| Premiere crashes when you open a specific project | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/premiere-crashes-when-you-open-a-specific-project.html |
| Premiere freezes on the splash screen | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/premiere-freezes-on-the-splash-screen.html |
| Adobe Premiere hangs on launch on macOS | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/premiere-hangs-on-launch-on-macos.html |
| Recover projects after a crash | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/recover-projects-after-a-crash.html |
| Try Adobe Creative Cloud Diagnostics | https://helpx.adobe.com/premiere/desktop/troubleshooting/crash-issues/try-adobe-creative-cloud-diagnostics.html |
| Export is very slow | https://helpx.adobe.com/premiere/desktop/troubleshooting/export-issues/export-is-very-slow.html |
| Unable to find the cause of render or export issue | https://helpx.adobe.com/premiere/desktop/troubleshooting/export-issues/unable-to-find-the-cause-of-render-or-export-issue.html |
| Unable to render or export a specific project | https://helpx.adobe.com/premiere/desktop/troubleshooting/export-issues/unable-to-render-or-export-a-specific-project.html |
| Unable to render or export Premiere project | https://helpx.adobe.com/premiere/desktop/troubleshooting/export-issues/unable-to-render-or-export-premiere-project.html |
| Projects saved in v25.6.0 cannot be opened in other v25.x versions | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/cannot-open-25-6-projects-in-25-x.html |
| Common issues in Premiere on Apple silicon Macs | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/common-issues-in-premiere-on-apple-silicon-macs.html |
| Creative Cloud Synced files End of life for Premiere | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/creative-cloud-synced-files-end-of-life-for-premiere.html |
| Files conformed each time project opens | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/files-conformed-each-time-project-opens.html |
| Generative Extend known issues | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/generative-extend-known-issues.html |
| iPhone HDR clips are unsupported for Generative Extend | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/iphone-hdr-clips-are-unsupported.html |
| Known and fixed issues in Adobe Premiere | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/known-and-fixed-issues.html |
| Networks and removable media with Digital Video | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/networks-and-removable-media-with-digital-video.html |
| Software rendering update | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/software-rendering-update.html |
| UI errors when using obsolete effects in on macOS Dark Mode | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/ui-errors-when-using-obsolete-effects-in-on-macos-dark-mode.html |
| Unable to send media from Firefly to Premiere | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/unable-to-send-media-from-firefly-to-premiere.html |
| Automatically manage your Media Cache files | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/automatically-manage-your-media-cache-files.html |
| Clear media cache on launch | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/clear-media-cache-on-launch.html |
| Clear media cache using preferences | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/clear-media-cache-using-preferences.html |
| Delete media cache files manually | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/delete-media-cache-files-manually.html |
| Eliminate flicker | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/eliminate-flicker.html |
| Manage media cache | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/manage-media-cache.html |
| Video assets from Creative Cloud Libraries broken between Premiere and After Effects | https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/video-assets-from-creative-cloud-libraries-broken-between-after-effects-and-premiere.html |
| Audio playback keep getting stuck | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/audio-playback-keep-getting-stuck.html |
| Choppy playback and poor performance issue | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/choppy-playback-and-poor-performance-issue.html |
| Hardware setup not optimal for playback performance | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/hardware-setup-not-optimal-for-playback-performance.html |
| Imports only audio in project bin | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/imports-only-audio-in-project-bin.html |
| Not able to optimize playback performance | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/not-able-to-optimize-playback-performance.html |
| Optimize playback performance for H.264 and H.265 media | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/optimize-playback-performance-for-h264-h265-media.html |
| Premiere exports look washed out | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/premiere-exports-look-washed-out.html |
| Slow rendering and playback | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/slow-rendering-and-playback.html |
| Timeline doesn’t show video preview | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/timeline-doesnt-show-video-preview.html |
| Troubleshoot sequence and file-interpretation settings | https://helpx.adobe.com/premiere/desktop/troubleshooting/playback-issues/troubleshoot-sequence-and-file-interpretation-settings.html |
| Reset preferences at launch | https://helpx.adobe.com/premiere/desktop/troubleshooting/preferences-and-settings-issues/reset-preferences-at-launch.html |
| Reset preferences using the preferences folder | https://helpx.adobe.com/premiere/desktop/troubleshooting/preferences-and-settings-issues/reset-preferences-using-the-preferences-folder.html |
| Restore custom preferences | https://helpx.adobe.com/premiere/desktop/troubleshooting/preferences-and-settings-issues/restore-preferences.html |
| Unable to migrate settings after updating Premiere | https://helpx.adobe.com/premiere/desktop/troubleshooting/preferences-and-settings-issues/unable-to-migrate-settings-after-updating-premiere.html |
| Why reset preferences | https://helpx.adobe.com/premiere/desktop/troubleshooting/preferences-and-settings-issues/why-reset-preferences.html |
