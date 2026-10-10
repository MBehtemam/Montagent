# Premiere gap: AI and automated features

Research for [#893](https://github.com/MBehtemam/Montagent/issues/893), part of the map
[#882](https://github.com/MBehtemam/Montagent/issues/882). Area slug: `ai`.

- **Date:** 2026-10-10.
- **Premiere side:** Adobe's own help pages on helpx.adobe.com only: the Premiere desktop user
  guide, [What's new][wn], the [release notes][rn] (26.0, January 2026, through 26.5, September
  2026) and the [beta features][beta] page. The agent proxy refused direct fetches of
  helpx.adobe.com (CONNECT 403), so every page was read through a web search restricted to
  `helpx.adobe.com`. Each row cites the page its mechanism comes from. Where Adobe pages
  disagree on dates, the release notes win.
- **Montagent side:** tag `v0.2.0` (`51aa4b5` = `origin/main` at the time of writing):
  `CONTEXT.md`, `schema/montagent.schema.json`, `docs/adr/`, the MCP tool surface in
  `crates/montagent/src/mcp.rs`, and every file under `skills/`.
- **How a status is read.** Montagent contains no model (`CONTEXT.md`, **Montagent**), so no AI
  feature can be `have` inside the binary. The map's vocabulary applies by mechanism
  ([ADR-0145](../../adr/0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)):
  - `have`: an equivalent mechanism ships in the v0.2.0 binary or format.
  - `have (skill)`: a bundled skill or skill script produces the same result as ordinary file
    contents.
  - `partial`: the format can hold the result (or part of the job ships), but the analysis or
    generation step is missing. The row says which part.
  - `missing`: nothing equivalent ships, and no ADR refuses it. Many generated or analysed
    results could still be *placed* as an ordinary `video`, `audio` or keyframe list once an
    outside tool made them. That alone does not lift a row out of `missing`.
  - `refused`: an ADR rules the mechanism out; the ADR is cited.

## Counts

| Status | Rows |
|---|---|
| have | 2 |
| have (skill) | 2 |
| partial | 17 |
| missing | 15 |
| refused | 1 |
| **Total** | **37** |

## Inventory

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| **Generative AI** | | | | | |
| Generative Extend (video) | A Firefly model generates new frames at the head or tail of a clip (one end, not both). Adds up to 2 s of video. Since 26.5 it accepts 360p to 4K UHD, any aspect ratio, and interlaced or non-square-pixel footage. | [Generative Extend overview][genext], [FAQ][genext-faq], [What's new][wn] | missing | Nearest: `overrun: "hold"` freezes the last frame ([ADR-0020](../../adr/0020-speed-overrun-hold-loop.md)), and a literal `source_time` freezes any frame ([ADR-0157](../../adr/0157-a-speed-ramp-is-a-time-remap-curve-of-source-times-on-a-video-element.md)). A clip extended by an outside model would enter as an ordinary `video` source. | Any frame generation. No skill calls an outside video model, and no recipe convention records a generated extension the way pre-rendered footage records its recipe ([ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7). |
| Generative Extend (audio) | Generates up to 10 s more room tone, sound effects or background audio. It will not create dialogue, and clips containing music are not eligible. Extended stretches cannot be transcribed. | [Generative Extend FAQ][genext-faq], [known issues][genext-ki] | missing | Nearest: `overrun: "loop"` restarts an audio source with a hard cut ([ADR-0020](../../adr/0020-speed-overrun-hold-loop.md)). A loop repeats the tone; it does not generate any. | Audio generation. A looped bed with an `audio_crossfade` ([ADR-0176](../../adr/0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)) is the only hand-written stand-in, and no skill teaches it. |
| Generative Media Tool: text to video (26.5) | Generates video clips in the timeline from a text prompt and optional reference frames, using Firefly or a chosen partner model, and places them as editable clips. Uses generative credits. | [Generative Media Tool overview][gmt], [generate media][gmt-gen], [FAQ][gmt-faq] | missing | A generated file would be an ordinary `video` element ([ADR-0002](../../adr/0002-inline-source-no-asset-table.md)). | Generation, model choice, and a skill that calls an outside generator and records its prompt, model and seed as a recipe beside the footage. |
| Generative Media Tool: sound effects (26.5) | Generates sound effects from a text prompt, optionally guided by the editor's voice for timing and intensity. | [Generative Media Tool overview][gmt] | missing | A generated file would be an ordinary `audio` element. | Generation, and voice-guided timing. |
| Firefly integration (send from Firefly, Firefly Boards import, 26.3) | Media generated in Firefly or Firefly Boards is sent or imported into a Premiere project. | [What's new][wn], [beta features][beta] | missing | Sources are plain paths or URLs on the element ([ADR-0053](../../adr/0053-asset-path-resolution-no-assetroot.md)); a URL is probed but must be local to render ([ADR-0131](../../adr/0131-render-and-frame-use-local-sources-only-and-validate-says-so.md)). | Any import from a generative service. Only a downloaded file can be used. |
| Content Credentials with generative-AI disclosure (26.2) | Export Settings can attach Content Credentials (C2PA), including whether generative AI was used. | [Release notes][rn] | missing | Related but different: `verify` attests that the MP4 was rendered from this version of the project ([ADR-0117](../../adr/0117-verify-measures-the-deliverable-with-the-decoder-and-a-stale-file-is-one-error.md)). Pre-rendered footage carries a recipe outside the project ([ADR-0156](../../adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7). | A provenance manifest embedded in the deliverable, and any record of which sources were AI-generated. |
| **Masking and tracking** | | | | | |
| Object Masking: select a person or object (26.0; edge modes 26.2; more objects, smoothness and Mask Expansion 26.5) | One click on the subject; a model segments it into a mask. Add or subtract regions, Sharp or Smooth edges, and expand or contract. | [Object Masking][objmask], [What's new][wn] | partial | The mask a selection produces can be held: a `mask` effect member takes a closed path written point by point, keyable, with `feather` and `invert` ([ADR-0152](../../adr/0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md), [ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md)). `frame --crop` gives the agent a true-scale look ([ADR-0101](../../adr/0101-a-crop-is-served-at-true-scale.md)). | Segmentation: nothing finds the subject's outline, and no skill does. A path is a polygon, not a per-pixel matte. A matte taken from another element is refused ([ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)), so a segmentation could not enter as a matte video either. No expand or contract parameter. |
| Mask tracking (Object Mask tracking, faster with restore in 26.3; shape-mask Track forward/backward) | The mask is analysed and moved frame by frame to follow the subject, then reviewed and adjusted. | [Mask tracking tools][masktrack], [Track masks][masktrack2], [Object Masking][objmask] | partial | A tracked mask's result can be held as keyframes on the mask path's `points` and rect, which are animatable ([ADR-0146](../../adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md), [ADR-0163](../../adr/0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md)). | The tracker. No point or planar tracker, and no skill that writes per-frame mask keyframes from footage. |
| **Audio** | | | | | |
| Enhance Speech | A model rebuilds dialogue to remove noise and room and improve the voice. It runs in the background from Essential Sound; a Mix Amount slider blends enhanced and original. It is a render pass and is not keyframable. | [Enhance speech][enhance], [Essential Sound panel][esp] | partial | Conventional repair ships: `highpass`, `lowpass`, `shelf`, `bell` EQ ([ADR-0179](../../adr/0179-eq-is-four-stackable-audio-effects-built-from-butterworth-biquad-sections.md)) and `compressor`/`limiter` ([ADR-0180](../../adr/0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md)) in `audio_effects`. Mix Amount can be built from two elements (the original and an externally enhanced file) at chosen `volume`s. | Model-based speech enhancement, and a skill to run one outside and place its output. `reduce_noise` ([ADR-0181](../../adr/0181-reduce-noise-is-a-singular-audio-effect-over-anlmdn-whose-dial-holds-on-reference-noise-and-nowhere-else-measured.md)) and the noise gate ([ADR-0183](../../adr/0183-the-noise-gate-is-one-stackable-audio-effect-with-a-range-no-hold-and-an-rms-threshold.md)) are `proposed` and absent from the v0.2.0 schema. No de-reverb, de-hum or de-ess. |
| Audio category auto-tagging | A model classifies each clip as Dialogue, Music, SFX or Ambience and badges it; the type then drives Essential Sound's controls, including ducking. | [Auto tag audio categories][autotag], [older page][autotag-old] | missing | Audio is an ordinary element with no role field ([CONTEXT.md](../../../CONTEXT.md), **Element**). A free-text `group` or the track's `name` can carry a label the agent chooses; neither is read by anything. | The classifier, and any typed role that a tool reads. The duck takes its `--bed` and `--voice` ids from the agent ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) §3). |
| Auto Ducking | Essential Sound > Ducking: choose what to Duck Against, set Sensitivity, Duck Amount (dB) and Fade Duration, then Generate Keyframes. The result is ordinary keyframes (on an Amplify effect's gain) on the ducked clip. | [Automatically duck audio][duck], [how-to][duck-howto] | have (skill) | `skills/montagent-footage/scripts/duck.py` writes ordinary `volume` keyframes from word timings or spans, in dB, with ramp, lead, join and fade ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md), [ADR-0055](../../adr/0055-audio-mixing-model-volume-fades-ducking-deferred.md)). `--check` finds a stale duck. | Detection of the voice from the signal, which the next row covers. Several beds need several calls. |
| Auto Ducking: trigger detected from the audio (Sensitivity) | The dips are placed where Premiere detects the duck-against signal, at a Sensitivity threshold. | [Automatically duck audio][duck] | refused | [ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) §2: "No silence detection. It would put a signal-analysis judgement, with thresholds the file never records, inside the write path." The script refuses to guess from the voice element when it has no words or spans. | By design. The spans come from an aligner's words file or are written in the call. |
| Auto-Match loudness | Essential Sound > Loudness > Auto-Match measures the clip and sets its gain to a LUFS target; the clip's measured LUFS is shown. | [Auto-match audio loudness][automatch] | have | `normalize_loudness` with one authored `target_lufs`, measured and applied as one fixed gain per clip ([ADR-0178](../../adr/0178-normalize-loudness-is-a-singular-audio-effect-with-one-authored-target-lufs-and-one-fixed-gain.md)); a programme-level `master.target_lufs` and `ceiling_dbtp` ([ADR-0172](../../adr/0172-the-master-stage-is-a-top-level-loudness-target-and-true-peak-ceiling-reached-by-one-measured-gain.md)); `verify` measures the result ([ADR-0173](../../adr/0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md)). | No per-type default targets: the target is always written by the author. ADR-0178 is still `proposed` pending its measured table. |
| Remix | Retimes a music clip to a target duration, keeping the start and end and rearranging the middle at analysed segment boundaries, usually to within a second. Segment and variation controls; zigzag marks show the edits. | [Remix in Premiere][remix], [Remix audio using AI][remix-ai] | partial | The result can be held: several `audio` elements over source ranges of one file, joined by `audio_crossfade` transitions ([ADR-0176](../../adr/0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)). `skills/montagent-craft/scripts/beat_grid.py` finds tempo and downbeats and lays the track in. | The analysis that picks matching segments and cut points, and a script that writes the rearranged elements to a target length. Beat finding is itself a workaround for #547. |
| Bulk mute or bleep censored words (26.2.2) | After transcription, a Censored word list flags words; one click applies a Bleep or Mute tone to one or all instances. | [Clean audio with bulk mute or bleep][bleep], [release notes][rn] | partial | Silencing a span is ordinary `volume` keyframes ([ADR-0055](../../adr/0055-audio-mixing-model-volume-fades-ducking-deferred.md)); `duck.py --spans` writes dips over given spans. A tone file can sit on its own `audio` element. Word times follow the `{word, start, end}` convention ([ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md)). | The transcription, a word-list matcher, a true mute (the duck's depth and ramps are tuned for music beds), and a bundled bleep tone. No skill does it. |
| Sound-effect search (media intelligence, 26.2.2) | Media intelligence finds sound effects in the project by AI analysis. | [Release notes][rn] | missing | None. | Audio content analysis and search. |
| **Colour** | | | | | |
| Auto Color | An Adobe Sensei model sets Lumetri Basic Correction (white balance, tone, saturation) from the frame; an Intensity slider scales it. Works on LOG and LUT-applied footage. | [Use Auto Color][autocolor] | partial | Four colour filters `brightness`, `contrast`, `saturation`, `tint`, each keyable ([ADR-0049](../../adr/0049-v1-colour-filter-vocabulary-four-scalar-members.md), [ADR-0146](../../adr/0146-an-animatable-property-is-one-the-schema-types-so-colour-blends-premultiplied-and-spring-easing-is-refused.md)). The agent can look with `frame` and choose values. | The analysis, and no skill for it. White balance (temperature), exposure, highlights and shadows, curves, wheels and LUTs are absent; LUT and curve are on the colour filter's avoid list and need their own ADR under ADR-0049's four clauses. |
| Color Match | Comparison View: pick a reference frame, Apply Match; Lumetri colour wheels and saturation are set to match, weighting skin tones when Face Detection is on. Removed in the 27.0 beta's Color mode along with Clip Match. | [Match color between shots][colormatch], [how-to][colormatch-howto], [Lumetri retirement][lumetri27] | partial | Same four filters as Auto Color. `frame` can show two instants for the agent to compare. | Measured matching (no histogram or scope reading is served), face weighting, colour wheels. No skill. |
| **Editing automation** | | | | | |
| Morph Cut | A Dissolve-group transition that uses face tracking and optical-flow interpolation to hide a jump cut in a talking head; typically 10 to 20 frames. | [Apply Morph Cut][morphcut] | missing | `transition` kinds are `crossfade`, `wipe`, `slide`, `push` and `audio_crossfade` ([ADR-0059](../../adr/0059-transitions-element-type-crossfade-only-exact-window.md), [ADR-0150](../../adr/0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)). The talking-head reference says to cut on a pause, not mid-word. | Optical-flow or face-aware interpolation between two frames. A crossfade is the only stand-in. |
| Auto Reframe | Analyses the action in each shot and keyframes position and scale to keep it in a new aspect ratio (Slower, Default, Faster Motion). On a sequence, it makes a duplicate sequence at the new size. | [Auto Reframe overview][reframe], [on a clip][reframe-clip], [on a sequence][reframe-seq] | partial | The result can be held: `fit: "cover"` sizes the footage ([ADR-0015](../../adr/0015-fit-is-a-derivation-claim-and-gravity-retires.md)), and keyframed `x`, `y`, `scale` move it ([ADR-0012](../../adr/0012-flat-transform-keyframes-carried-by-their-element.md)). The footage skill's "wide footage in a tall frame" recipe slides the element to keep the speaker centred. | Subject detection and the motion path from it. No skill, and the recipe is a single static slide. Making a new-aspect copy of a project is a hand edit. |
| Scene Edit Detection | Finds the cuts inside one video file and either cuts the clip there, makes subclips in a bin, or adds markers. | [Scene Edit Detection][sed], [older page][sed-old] | missing | Nearest: the footage skill scans a long clip with `frame --from --to --infill-ceiling` in a scratch project ([ADR-0130](../../adr/0130-infill-fills-the-rung-not-the-grid-at-the-latest-frame-within-the-ceiling.md)), and the agent reads the tiles. `query --from --to` lists the project's own cuts ([ADR-0074](../../adr/0074-the-cut-lists-presence-set-is-every-element.md)), not a source's. | Shot-boundary detection in a source file. No markers, bins or subclips to write the result to; the agent would write split `video` elements by source range. |
| Automate to Sequence | Places selected bin clips in order (sort or selection order) at the playhead or at unnumbered markers, with the default video and audio transition between each, and sets overlap. | [Set and apply default transitions][automate] | partial | The agent writes the elements, and the router skill says to generate a project with a script "when the structure repeats". Transitions are explicit elements ([ADR-0059](../../adr/0059-transitions-element-type-crossfade-only-exact-window.md)). | A bundled script that lays N clips back to back with transitions. There are no markers, no bins and no default transition. |
| Synchronize clips by audio (Synchronize, multicam source sequence) | Aligns clips by matching their audio waveforms. | [Synchronize clips in the Timeline][sync], [multi-camera source sequence][multicam] | missing | None. Element times are absolute literals ([ADR-0005](../../adr/0005-absolute-integer-milliseconds.md)). | Waveform cross-correlation, and a skill that writes the resulting `start` or `source_start` offsets. |
| Warp Stabilizer | Analyses camera motion and warps the frames to cancel shake (Position to Subspace Warp). | [Warp Stabilizer][warp] | missing | None in the effects list ([ADR-0040](../../adr/0040-effect-model-attachment-and-v1-vocabulary.md)). A clip stabilised outside could be placed as an ordinary `video`. | Motion analysis and warping, and a pre-render recipe for an outside stabiliser. |
| **Transcription and text** | | | | | |
| Speech to Text (transcription) | An Adobe Sensei model transcribes dialogue in the listed languages, on import or on demand, into a word-timed transcript. | [Auto transcribe with Speech to Text][stt], [languages][stt-lang], [preferences][stt-prefs] | partial | Word timing is an outside step by decision ([ADR-0051](../../adr/0051-word-alignment-is-external-validate-and-compare-catch-drift.md)): the agent normalises any aligner's output to `{word, start, end}` ms; `validate` checks highlight containment and overlap, and `compare` flags text changed under unchanged timing. | A transcriber. Running one inside Montagent is refused by ADR-0051 ("does not invoke, ship, or shell out to a forced aligner"); no skill runs one outside either, and every skill assumes a words file is supplied. |
| Speaker labels | Transcription separates speakers; speakers can be renamed or added. | [Edit speaker names][speakers], [preferences][stt-prefs] | missing | The talking-head reference covers two speakers' styling, not detection. | Diarisation. The words convention has no speaker key. |
| Voice Activity Detection (26.5) | A new model decides where speech occurs before transcription. | [Release notes][rn], [What's new][wn] | partial | `captions.py` runs ffmpeg `silencedetect` (a non-model detector) to move a late phrase start back and report words inside a silence. | Use of the result anywhere else: `duck.py` refuses silence detection ([ADR-0177](../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md) §2), and there is no model-based detector. |
| Create captions from a transcript (styles; single-word captions 26.3) | Captions are generated from the transcript with a preset and style, line length and line count, and can be one word at a time. | [Create captions][captions], [What's new][wn] | have (skill) | `skills/montagent-footage/scripts/captions.py` pages word timings into one or two lines with `measure`, lights each spoken word with a `highlight` window ([ADR-0048](../../adr/0048-per-word-highlighting-is-a-timed-window-on-the-run.md)), and repairs bad timings. Caption checks: pace, minimum duration, no audio, repeat ([ADR-0034](../../adr/0034-caption-pace-and-repeat-duration-checks.md), [ADR-0054](../../adr/0054-caption-audio-backing-and-minimum-duration-checks.md), [ADR-0136](../../adr/0136-a-text-element-opts-out-of-the-caption-checks-with-caption-false.md)). | A one-word-per-page mode is not a documented spec option. No sidecar caption export (SRT, VTT); captions are burned in. |
| Translate captions | Translates a caption track into other languages, each as its own caption track. | [Translate captions][translate] | partial | The agent is a language model and can translate text; translated captions are ordinary `text` elements. A target and translation pair is meaning a `group` may label, not a field ([ADR-0003](../../adr/0003-general-video-editor-not-channel-tooling.md)). | Timing for the translation: `captions.py` needs per-word times in the caption's language, so a translated page must be timed by hand to the source page. No skill. |
| Text-Based Editing | Cut, copy, paste and rearrange words in the sequence transcript; the timeline follows. Three-point edits from the source transcript. | [Text-Based Editing overview][tbe], [edit sequences][tbe-seq], [edit transcripts][tbe-edit] | partial | The agent reads the words file and rewrites `source_start`/`source_end`. `shift` moves every time at or after an instant to close or open time, and refuses rather than stretch a straddling clip ([ADR-0032](../../adr/0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md), [ADR-0047](../../adr/0047-shift-releases-slack-by-boundary-instant-pairs.md)). | A script that turns a transcript selection into source ranges and a ripple. The transcript itself is outside (row above). |
| Filler-word detection and bulk delete | The transcript marks filler words (uh, um); one or all are deleted from the timeline at once. | [Detect and delete pauses and fillers][pauses], [TBE overview][tbe] | partial | Same pieces as Text-Based Editing. The talking-head reference says cuts inside a take go on a pause of 150 ms or more. | Filler tagging (many aligners drop fillers), and a cut-and-ripple script. |
| Pause detection and bulk delete | The transcript marks pauses; one or all are deleted. | [Detect and delete pauses][pauses] | partial | `captions.py`'s `silencedetect` finds pauses for its own check; `shift` closes time. | A script that cuts at the detected pauses and ripples. |
| Paper Edit (26.5) | Builds a new sequence from selected transcript passages. | [Release notes][rn], [TBE overview][tbe] | partial | The agent can write a new project from chosen source ranges; `create_project` scaffolds one ([ADR-0080](../../adr/0080-the-scaffold-writes-what-it-was-told-and-the-two-resources-are-named.md)). | Transcription, and a script from passage selection to elements. |
| Remove all of one speaker | Deletes every passage the transcript assigns to one speaker, for example an off-camera interviewer. | [Remove speaker dialogue][rmspeaker] | missing | None. | Diarisation, then the same cut-and-ripple script as Text-Based Editing. |
| **Search and organisation** | | | | | |
| Media intelligence search (26.0) | Footage is analysed locally on import; the Search panel finds shots by what is visible, by transcript and by metadata across thousands of shots. | [Search media using AI][mi], [FAQ][mi-faq], [What's new][wn] | partial | The agent can look: one `frame --from --to` contact sheet per clip shows labelled tiles across it (footage skill, [ADR-0094](../../adr/0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md)), and `probe` reports size, rate, length and sound. | An index or a search across media. Each clip is looked at one sheet at a time, and there is no project bin to search. |
| Search Similar Frames and alternate takes (26.0, 26.2.2) | Right-click a frame in the Program Monitor; the Search panel lists visually similar sections of other clips with In and Out set. | [Find similar visuals][similar], [release notes][rn] | missing | None beyond looking at contact sheets. | Visual embedding and similarity search. |
| Premiere AI Assistant (beta) | Window > Assistant takes plain-language requests: bins, renaming and colour labels, transcripts, slate detection, markers, stringouts and rough cuts. Every action is in Undo and History. Some tools run in the cloud. | [AI Assistant overview][assistant], [FAQ][assistant-faq] | have | Montagent's premise: an outside agent edits the project file and calls Montagent's verbs over MCP ([CONTEXT.md](../../../CONTEXT.md), **Agent**; [ADR-0011](../../adr/0011-tool-surface-reads-checks-renders.md)); the router skill holds the loop and the capability map; the file in git is the undo history. | Nothing in the mechanism. Some of the Assistant's tasks have no target here: bins, colour labels and markers do not exist, and slate detection needs the agent to look at sheets. |

## Checked and left out

- **Object removal or addition by generative fill in video.** Named by the ticket. No Premiere
  desktop help page describes it; Adobe documents Content-Aware Fill in
  [After Effects][ae-caf] only. Not a Premiere feature, so no row.
- **Premiere on iPhone only:** Beat Detection, Speed Curves, AI speech cleanup
  ([iPhone what's new][iphone]). Desktop is the map's subject. For the record, beat finding
  exists as `beat_grid.py` in `montagent-craft`.
- **Media intelligence in Media Encoder** ([page][ame-mi]): a Media Encoder feature.
- **Essential Sound Repair and Clarity sliders, DeNoise, DeReverb, DeHummer, Adaptive Noise
  Reduction:** effects with sliders, not analysis. They belong to the audio area and are covered
  in [`docs/research/audio-effects/PRECEDENT.md`](../audio-effects/PRECEDENT.md).
- **Color mode (27.0 beta)** replacing the Lumetri panel: a colour-management workflow with no
  documented AI step ([Color mode][colormode]).

## Notable gaps

1. **No transcription anywhere in the bundle.** ADR-0051 refuses one inside Montagent, but no
   skill runs one outside either. Six rows (Text-Based Editing, filler and pause deletion, Paper
   Edit, bleep, translation) depend on a words file the user must already have.
2. **Transcript-driven cutting.** The format and `shift` can hold a transcript edit, but no
   script turns "delete these words" or "delete pauses over 400 ms" into source ranges and a
   ripple.
3. **Subject detection and tracking.** Object Masking, mask tracking and Auto Reframe all
   produce keyframes Montagent can already hold (mask `points`, `x`/`scale`). What is missing is
   the tracker, and no skill fills the gap.
4. **Generative media.** Generative Extend and the Generative Media Tool have no route, not even
   a recipe convention for recording the prompt, model and seed of a generated source the way
   ADR-0156 records pre-rendered footage.
5. **Colour analysis.** Auto Color and Color Match have only four colour filters to write into,
   with no white balance, wheels, curves or LUTs.
6. **Media search.** Media intelligence and Search Similar Frames have no index; the agent looks
   at one contact sheet per clip.
7. **Speech enhancement.** Model-based Enhance Speech has no equivalent, and the conventional
   `reduce_noise` and noise gate are still `proposed` and absent from the v0.2.0 schema.
8. **Interpolation and analysis effects.** Morph Cut, Warp Stabilizer and audio sync of
   multicam angles are absent, and the pre-render skill has no recipe for outside tools that do
   them.

<!-- Adobe -->
[wn]: https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html
[rn]: https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html
[beta]: https://helpx.adobe.com/premiere/desktop/whats-new/beta-features.html
[genext]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-extend-overview.html
[genext-faq]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-extend-faq.html
[genext-ki]: https://helpx.adobe.com/premiere-pro/using/generative-extend-known-issues.html
[gmt]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-media-tool-overview.html
[gmt-gen]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generate-media-with-generative-media-tool.html
[gmt-faq]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-media-tool-faq.html
[objmask]: https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/object-masking.html
[masktrack]: https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/mask-tracking-tools.html
[masktrack2]: https://helpx.adobe.com/premiere/desktop/add-video-effects/work-with-masks/track-masks.html
[enhance]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/enhance-speech.html
[esp]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-editing-with-essential-sound-panel.html
[autotag]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-category-tagging.html
[autotag-old]: https://helpx.adobe.com/mena_en/premiere-pro/using/audio-auto-tagging.html
[duck]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/automatically-duck-audio.html
[duck-howto]: https://helpx.adobe.com/ph_fil/premiere-pro/how-to/automatic-audio-ducking.html
[automatch]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/auto-match-audio-loudness.html
[remix]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-in-premiere.html
[remix-ai]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-audio-in-premiere.html
[bleep]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/clean-audio-with-bulk-mute-or-bleep.html
[autocolor]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/use-auto-color.html
[colormatch]: https://helpx.adobe.com/premiere/desktop/correct-color/add-color-effects/match-color-between-shots.html
[colormatch-howto]: https://helpx.adobe.com/ph_fil/premiere-pro/how-to/automatically-match-color.html
[lumetri27]: https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/lumetri-color.html
[colormode]: https://helpx.adobe.com/premiere/desktop/correct-color/color-mode-fundamentals/color-mode-basics.html
[morphcut]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/apply-morph-cut-to-smoothen-jump-cuts.html
[reframe]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/auto-reframe-overview.html
[reframe-clip]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-clip.html
[reframe-seq]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/add-auto-reframe-effect-to-a-sequence.html
[sed]: https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/detect-edit-points-using-scene-edit-detection.html
[sed-old]: https://helpx.adobe.com/be_en/premiere-pro/using/scene-edit-detection.html
[automate]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-transitions/set-and-apply-default-transitions.html
[sync]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/synchronize-clips-in-the-timeline-panel.html
[multicam]: https://helpx.adobe.com/premiere/desktop/edit-projects/set-up-multi-camera-sequences-for-editing/create-a-multi-camera-source-sequence.html
[warp]: https://helpx.adobe.com/premiere/desktop/add-video-effects/commonly-used-effects/stabilize-shaky-footage-using-warp-stabilizer.html
[stt]: https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/auto-transcribe-video-using-speech-to-text.html
[stt-lang]: https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/languages-supported-by-speech-to-text.html
[stt-prefs]: https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html
[speakers]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-speaker-names-in-transcription.html
[captions]: https://helpx.adobe.com/ca/premiere-pro/using/working-with-captions.html
[translate]: https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html
[tbe]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html
[tbe-seq]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-sequences-using-text-based-editing.html
[tbe-edit]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-transcripts-using-text-based-editing.html
[pauses]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/detect-and-delete-pauses-in-transcripts.html
[rmspeaker]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/remove-all-instances-of-one-speaker-in-transcript.html
[mi]: https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-for-media-using-ai-powered-media-intelligence.html
[mi-faq]: https://helpx.adobe.com/premiere-pro/using/media-intelligence-and-search-panel-faq.html
[similar]: https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-for-similar-visuals-using-media-intelligence.html
[assistant]: https://helpx.adobe.com/premiere/desktop/premiere-ai-assistant/overview.html
[assistant-faq]: https://helpx.adobe.com/premiere/desktop/premiere-ai-assistant/assistant-faq.html
[ae-caf]: https://helpx.adobe.com/after-effects/using/content-aware-fill.html
[iphone]: https://helpx.adobe.com/premiere/mobile/whats-new/whats-new.html
[ame-mi]: https://helpx.adobe.com/media-encoder/using/media-intelligence-in-media-encoder.html
