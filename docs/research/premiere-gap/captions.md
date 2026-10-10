# Premiere Pro gap: captions, subtitles and transcripts

Research ticket [#888](https://github.com/MBehtemam/Montagent/issues/888) on the map
[#882](https://github.com/MBehtemam/Montagent/issues/882). Area slug: `captions`.

- **Date:** 2026-10-10
- **Premiere side:** Adobe's own Premiere help (helpx.adobe.com: the user guide's
  *Insert captions*, *Text-Based Editing*, *Stylize text*, *Export files* and *Preferences*
  pages, the What's-new page, and the known-issues page). No third-party source is used.
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, = `origin/main`): `CONTEXT.md`,
  `schema/montagent.schema.json`, `docs/adr/`, the MCP tool surface in
  `crates/montagent/src/mcp.rs`, and `skills/`.
- **Status vocabulary (map #882):** `have`, `have (skill)`, `partial`, `missing`, `refused`.
  Read by mechanism, not by name (ADR-0145).

**Counts (50 rows):** have 8 · have (skill) 2 · partial 11 · missing 22 · refused 7.

**How the Premiere pages were read, and what that limits.** Direct fetches of
helpx.adobe.com failed in this session (DNS resolution refused for the fetch tool), so each
row's mechanism comes from Adobe-domain-restricted searches over those pages, which return
the page's own text. Every URL below is an Adobe page that search returned for that feature.
Three mechanisms could not be pinned down past the page's title and summary, and are marked
*(title-level)*: the Create captions dialog's line-length, duration and gap settings, single-word
captions, and transcript export formats. They need a follow-up read before the status is final
(the map's "Not yet specified" names this case). The status of each is still given, because the
Montagent side does not depend on the missing detail.

## The table

### Transcripts and Text-Based Editing

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Speech to Text (auto transcription) | On-device speech recognition (22.2+) turns a clip's or sequence's audio into a timed transcript in the Text panel's Transcript tab | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/auto-transcribe-video-using-speech-to-text.html | refused | ADR-0051 (Montagent does not invoke, ship or shell out to a forced aligner: a model in the tool breaks reproducibility); `CONTEXT.md` **Montagent** "contains no model"; ADR-0054 calls speech classification a constitutional exclusion | — (the agent transcribes outside Montagent and writes literal text) |
| Transcription language selection and language packs | English ships; other language packs download on demand from the Transcript tab; a separate page lists the supported languages | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/download-language-packs-from-within-premiere.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/languages-supported-by-speech-to-text.html | refused | ADR-0051, ADR-0054 (as above): no recognition model, so no recognition languages | — |
| Auto-transcribe on import | Media Analysis & Transcription preference: transcribe all existing and newly imported clips, or only clips in the sequence | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html | refused | ADR-0051 | — |
| Transcribe In-to-Out range, merge into an existing transcript | *Generate static transcript* dialog limits transcription to In/Out and can insert the result into an existing transcript | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/auto-transcribe-video-using-speech-to-text.html | refused | ADR-0051 | — |
| Speaker labelling (diarization) | Transcription preference separates speakers; off treats all dialogue as one speaker | https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/transcription-preferences.html | refused | ADR-0054 (telling voices apart is classification, "permanently out of reach" under no-model); ADR-0051 | — |
| Edit speaker names | Rename the speaker labels in the transcript | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-speaker-names-in-transcription.html | missing | No transcript or speaker concept in the schema; ADR-0003 says pairing meaning (e.g. who says a line) is the agent's, at most a `group` label | A speaker label; today only a free-text `group` or the element `id` can carry it, with no checks |
| Transcript as a project object, editable | The transcript lives with the clip in the Text panel; words are corrected in place | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-transcripts-using-text-based-editing.html | missing | ADR-0051: the word list is an external, unenforced `{word, start, end}` convention in ms; nothing parses it | Any transcript in the document; Montagent only holds the frozen result (text runs, `highlight` windows) |
| Find and replace in the transcript / captions | Search field highlights every match; Replace / Replace All | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/find-and-replace-text-in-transcription.html | partial | `query` with the `where` predicate (ADR-0070) finds elements; every caption string is editable by exact-string replace (ADR-0145 invariant 4, ADR-0007 raw NFC UTF-8 runs) | No transcript to search; no replace verb (by design: write tools take complete elements, ADR-0011, ADR-0051) — replace is the agent's file edit |
| Text-Based Editing: cut the sequence by deleting transcript text | Deleting words in the sequence transcript removes the matching media from the sequence | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/edit-sequences-using-text-based-editing.html · https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html | partial | Edit primitives ship: `video`/`audio` `source_start`/`source_end`, the `shift` tool (ripple, slack-invariant, ADR-0032/0047); an agent with an external words file can compute the cuts | The link from text to source time; any tool that turns a deleted word span into trims. The transcript itself is refused (ADR-0051) |
| Text-Based Editing: add clips from a transcript selection | Select transcript text in a source clip and insert/overwrite it into the sequence | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/add-clips-to-the-timeline-using-text-based-editing.html | partial | Same as above: a `video` element's `source_start`/`source_end` are the in/out the agent writes | Transcript selection as an in/out source |
| Filler-word detection and bulk delete | Detects "uh"/"umm" filler words in any of the STT languages; filter and Delete all | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/overview-of-text-based-editing.html · https://helpx.adobe.com/in/premiere-pro/using/whats-new/2024-1.html | refused | Recognising a filler word is speech recognition: ADR-0051, ADR-0054 | — |
| Pause detection and bulk delete | Transcript filter shows pauses above a minimum duration (Transcript View Options); Delete / Delete all | https://helpx.adobe.com/premiere/desktop/edit-projects/edit-video-using-text-based-editing/detect-and-delete-pauses-in-transcripts.html | partial | `skills/montagent-footage/scripts/captions.py` runs ffmpeg `silencedetect` and reports words inside silences; ADR-0054 names `silencedetect` as a deterministic, non-model technique | A pause list as an editing input, and the bulk cut; the skill only reports and re-times captions, it never removes media |
| Bulk mute or bleep from the transcript | Select transcript words and mute or bleep their audio in bulk | https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/clean-audio-with-bulk-mute-or-bleep.html | partial | Muting a span is `volume` keyframes (ADR-0055; `duck.py` writes such keyframes from spans, ADR-0177); a bleep is an `audio` element of a tone file | Selection by word; a generated tone (every `audio` element names a file) |
| Export transcript | Exports the transcript to a file *(title-level: formats not confirmed)* | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-transcripts.html | missing | No transcript in the document (ADR-0051) | Any transcript output |

### Caption tracks and formats

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Caption track as its own track type | Captions sit on a dedicated caption track above video tracks; the active one is the export default | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/about-captions.html | partial | A **Caption** is any `text` element without `caption: false` (ADR-0136, `CONTEXT.md`); it lives on an ordinary track (ADR-0004: `name`, `layer`, `elements`) | A track kind or format: a Montagent caption is pixels only, so it carries no caption-stream semantics (format, channel, language) |
| Open captions / Subtitle track, rendered into the picture | Subtitle-format track (and the legacy Open Captions type) drawn over the video | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/about-captions.html · https://helpx.adobe.com/uk/premiere-pro/using/old-captions-workflow.html | have | `text` element: `runs`, `font`, `size`, `color`, `align`, `stroke`, box; rendered into every frame | — |
| CEA-608 closed captions | 608 caption track; channels CC1–CC4; .scc/.mcc/.xml/.srt sidecar or embedded | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html | missing | No caption stream anywhere; `render` writes H.264 MP4 only (ADR-0143) | 608 encoding (roll-up/pop-on, channels, its character set) |
| CEA-708 closed captions | 708 caption track (all services); .mcc/.xml/.srt sidecar or embedded | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html | missing | As above | 708 encoding |
| Wide-aspect CEA-708 import (26.0) | Import CEA-708 captions authored for wide aspect correctly | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html | missing | No caption import | — |
| Teletext | Teletext caption track; .xml/.stl/.srt sidecar | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html | missing | As above | Teletext model |
| EBU Subtitle | EBU subtitle track; .xml/.stl/.srt sidecar | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html | missing | As above | EBU subtitle model |
| Australian OP-47 | OP-47 track; .xml/.stl/.srt sidecar, one stream embedded | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html | missing | As above | OP-47 model |
| SubRip (.srt) import | Import the .srt like media, drag to the sequence: a new caption track is made and populated | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/import-caption-file-from-third-party-service.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html | missing | No ingestion tool of any kind (ADR-0051's no-ingestion stance; ADR-0011 surface) | An .srt → `text` elements conversion (a skill script would fit ADR-0051's pattern) |
| Scenarist (.scc) import/export | 608 only | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html | missing | — | Reader and writer |
| MacCaption VANC (.mcc) import/export | 608 and 708, both at once on export | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html | missing | — | Reader and writer |
| W3C TTML/DFXP, SMPTE-TT, EBU-TT (.xml) import/export | One track per export; 608, 708, Teletext, EBU, OP-47 | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html | missing | — | Reader and writer |
| EBU N19 (.stl) import/export | Teletext, EBU, OP-47; one stream per export | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html | missing | — | Reader and writer |
| WebVTT | Not in Adobe's supported caption format list (SCC, MCC, XML, STL, SRT, DFXP) | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html | missing | — | Neither tool has it; listed because the ticket names it |
| Decode embedded captions from MOV / MXF | Captions embedded in MOV, DNxHD MXF OP1a and MXF OP1a are read on import | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/supported-file-formats-for-captions.html | missing | `probe` reports size, rate, length, audio; no caption stream | Caption-stream decoding |

### Styling

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Caption text and appearance (font, size, colour, alignment, stroke, shadow) | Properties panel (25.0+, formerly Essential Graphics) Text and Appearance controls on a caption | https://helpx.adobe.com/mena_en/premiere/desktop/add-text-images/insert-captions/create-styles-for-captions.html · https://helpx.adobe.com/premiere-pro/using/about-properties-panel.html | have | `text`: `font` (declared fonts, ADR-0007), `size`, `color` (paint incl. gradient, ADR-0149), `align`, `stroke`/`stroke_width` (ADR-0014), `shadow` effect (ADR-0040), per-run deltas | — |
| Caption position and box | Captions placed in the frame; alignment and position edited per caption | https://helpx.adobe.com/uk/premiere-pro/using/old-captions-workflow.html | have | `x`, `y`, `origin`, `width`, `height` (required box, ADR-0014); `measure` sizes it (ADR-0024) | — |
| Background box behind captions | Appearance background on the caption | https://helpx.adobe.com/mena_en/premiere/desktop/add-text-images/insert-captions/create-styles-for-captions.html | have (skill) | `captions.py` `pill` writes a `rect` per page on `<track>-bg`, sized from `measure`; core has `rect` with `radius` | A box tied to the text: in core it is a separate element the agent keeps in step |
| Track style (one style for a whole caption track) | Save a named style; choose it in the Track style dropdown; every caption on the track follows it | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/apply-linked-and-track-styles.html | missing | Every value is a literal on the element (ADR-0007, ADR-0145 invariant 1); ADR-0002 argues against id indirection for files, not styles | A shared style; today a script (e.g. `captions.py` spec) writes the same literals onto every element |
| Linked text styles, local styles, reuse across projects | Save to project (linked) or Local Styles; import into other projects | https://helpx.adobe.com/premiere/desktop/add-text-images/stylize-text/create-text-styles.html | missing | As above | — |
| Default subtitle and closed-caption fonts (26.0) | Graphics preference sets the default caption fonts | https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html | missing | Fonts are declared per project in `fonts` (ADR-0007, ADR-0057); no preferences layer | A default font; every text names its font |

### Getting captions in and out

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Burn captions into video | Export Settings → Captions → Burn Captions Into Video | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html | have | `render` paints every `text` element into the MP4 (ADR-0011, ADR-0143) | — |
| Create sidecar file | Captions written as .scc/.mcc/.xml/.stl/.srt next to the video, formats filtered by track type | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html · https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html | missing | `output` is one path, one MP4 (schema `output`; ADR-0104) | Any second output; the text, timing and `caption` flag needed for an .srt are all in the document |
| Embed in output file | Toggleable caption stream inside MOV / MXF OP1a (608, 708 all channels; one OP-47 stream) | https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-caption-tracks.html | missing | `render` writes libx264 MP4 with no subtitle stream (ADR-0143) | A caption stream; MOV/MXF containers |
| Choose which caption track exports; hide/isolate tracks | The active track is the default; eye icon per track; Alt/Option-click isolates one | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html | missing | A track has only `name`, `layer`, `elements` (schema `Track`) | Track visibility; a render-time track selection |

### Languages

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Translate captions | Cloud translation (Google Translate, Microsoft Translator) makes a new caption track per target language | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html · https://helpx.adobe.com/in/premiere-pro/using/translate-captions-faq.html | refused | ADR-0051 and ADR-0054 (no model in Montagent); `CONTEXT.md` **Agent**: Montagent never calls an agent | — (the agent translates and writes text) |
| Several caption languages in one project | One caption track per language, each shown or hidden with its eye icon | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/translate-captions.html | partial | Any number of text tracks (ADR-0004); ADR-0003 refuses a "bilingual subtitle" field: a pairing is the agent's meaning, at most a `group` label | A language tag, and a way to deliver one language at a time (track visibility or one stream per language) |
| Caption text in many scripts | STT and translation cover European, CJK and other languages | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/languages-supported-by-speech-to-text.html | have | Runs are raw NFC UTF-8, any declared font (ADR-0007); bidi isolates and base direction (ADR-0133); UAX #14 breaks (ADR-0008); joining scripts (ADR-0153); Thai metrics (ADR-0087) | `R-CAPTION-PACE`'s 20 cps is Latin-calibrated and under-flags CJK (ADR-0034) |

### Creating and timing captions

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
| --- | --- | --- | --- | --- | --- |
| Create captions from the transcript (auto-captioning) | Create captions dialog: preset, format, style, and line length / duration / gap settings *(title-level for the settings)*; the result is a new caption track | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-captions.html | have (skill) | `captions.py` (montagent-footage): words file → one `text` per page, `lines`, `width`, `gap`, `hold`, `min_word`, frame-snapped; breaks from `measure` | The words file must come from an aligner outside Montagent (ADR-0051); no caption format choice (pixels only) |
| Single-word captions | Captions made one word at a time *(title-level)* | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-single-word-captions.html | partial | The format holds one-word `text` elements; ADR-0048 highlights one word per run | `captions.py` pages by `width`, `lines` and `gap` only: no words-per-page cap |
| Add and delete caption blocks | Add Caption (+) / Delete Caption (−); caption clips on the timeline | https://helpx.adobe.com/uk/premiere-pro/using/old-captions-workflow.html · https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/about-captions.html | have | Write or remove a complete `text` element (ADR-0011's write invariant) | — |
| Caption timing and duration | Trim caption clips on the timeline; edit timing in the panel | https://helpx.adobe.com/uk/premiere-pro/using/old-captions-workflow.html | have | Literal `start`/`end`; `measure --at` gives drawn instants; four caption checks: `R-CAPTION-PACE`, `R-CAPTION-REPEAT-DURATION` (ADR-0034), `R-CAPTION-NO-AUDIO`, `R-CAPTION-MIN-DURATION` (ADR-0054), scoped by `caption` (ADR-0136) | — |
| Keep captions in sync after trims and ripple deletes | Caption clips move with ripple edits on the timeline | https://helpx.adobe.com/uk/premiere-pro/using/old-captions-workflow.html | have | `shift` moves every time at or after an instant, project-wide, and refuses an edit that would change a slack (ADR-0032, ADR-0047); `compare` reports a run whose text changed while its `highlight` did not (ADR-0051) | — |
| Split a caption | Split one caption into two (a duplication bug fixed in 26.2) | https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/known-and-fixed-issues.html · https://helpx.adobe.com/in/premiere-pro/using/split-or-cut-clips.html | partial | The agent writes two complete `text` elements; `shift` splits only transform keyframes of a straddler | A split verb that divides text and time (by design ADR-0011 adds no field-level edit tools) |
| Merge captions | Not documented in the pages found; Premiere's Merge Clips is an audio/video operation | https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-timeline-panel.html | partial | The agent writes one `text` element in place of two | A merge verb; Premiere's own caption merge is unconfirmed |
| Caption timing matches the speech | Captions inherit word times from Speech to Text | https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/create-captions.html | partial | `captions.py` checks words against `silencedetect` and moves late phrase starts; `R-HIGHLIGHT-UNPAINTED` (ADR-0134); `E-HIGHLIGHT-RANGE`/`-OVERLAP` (ADR-0051) | A check of caption or highlight timing against the audio on disk (ADR-0051, ADR-0134 leave it open; #565) |

## Montagent-only mechanisms (no Premiere page found)

Not rows, since the table lists Premiere features, but they bear on any later decision:

- **Per-word (karaoke) highlight** as a timed style window on a run (ADR-0048), with
  `E-HIGHLIGHT-RANGE`, `E-HIGHLIGHT-OVERLAP` (ADR-0051) and `R-HIGHLIGHT-UNPAINTED` (ADR-0134).
  No Adobe page found in this pass documents an equivalent.
- **Caption quality checks:** reading pace, repeat-duration disagreement, no audio backing,
  minimum duration (ADR-0034, ADR-0054, ADR-0071), and the `caption: false` opt-out (ADR-0136).
  No Adobe page found documents caption QC.
- **`N-CAPTION-SETTLES`** gives the readable instant of a staggered caption (ADR-0151).

## Notable gaps

1. **No caption delivery other than burn-in.** No sidecar (.srt, .scc, .mcc, .xml, .stl), no
   embedded 608/708/OP-47 stream, no MOV/MXF. Everything an .srt needs (text, `start`, `end`,
   the `caption` flag) is already in the document. Only the output is missing.
2. **No caption import.** No .srt or broadcast-format reader, no embedded-caption decode. A
   skill script would fit ADR-0051's "the agent converts outside Montagent" pattern.
3. **Caption formats (CEA-608, CEA-708, Teletext, EBU, OP-47) are absent.** A Montagent caption
   is pixels plus a flag. It has no stream semantics.
4. **Transcript-driven work is refused at the recognition step** (Speech to Text, speaker
   labels, filler words, translation: ADR-0051, ADR-0054). Text-Based Editing and pause cuts
   are only partial: the edit primitives ship, but nothing links text to source time.
5. **No shared styles.** No track style, linked style or default caption font; every value is a
   literal per element, repeated by a script.
6. **No language tag or track visibility**, so several caption languages can coexist but
   cannot be delivered one at a time.
7. **Timing against the audio on disk stays unchecked in core** (ADR-0051 and ADR-0134's open
   gap, #565). Only the footage skill listens.
8. **Split and merge** have no verb. They are full-element rewrites by the agent.
