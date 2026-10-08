You are a juror on a court. Answer the Question below and nothing else: do not use any tools, do not edit anything, do not make recommendations to whoever convened you. Judge it from the standpoint of how AI agents actually work with Montagent: they author and edit the JSON document by hand (often by exact-string replace), run `validate`, `render`, partial renders and `preview`, read `verify` findings, and iterate.

## Context (facts, not opinions)

Montagent is a video document format plus renderer. An AI agent writes a JSON project file; the renderer turns it into video with ffmpeg. Standing invariants every capability must keep (ADR-0145): literal values (every value is written in the file, never an expression or reference), closed vocabulary (every key/enum in the schema), checkable by `validate` from the file and probed assets alone without painting a frame, and editable by exact-string replace (every authored choice has its own string).

Project top level today: `frame`, `fps`, `background?`, `duration?`, `loop?`, `output?` (a plain path string), `fonts?`, `fontVendor?`, `tracks`. Elements live only in `tracks[].elements`. Tracks are constrained lanes that own nothing (no timing, no audio fields); track-level audio effects are ruled out of scope.

Audio today: `audio` and `video` elements have `volume`, a keyframeable linear multiplier (>1 allowed; clipping past the sum is documented renderer behaviour). Mix graph per element: `aformat 48k stereo → atrim → atempo → aloop → [audio_effects list, decided] → volume → [pan, decided] → adelay`, then `amix normalize=0 → apad/atrim → AAC 160k`. Nothing after amix: no limiter, no normalisation. Sample rate, stereo and codec are hard-coded. New levels are in dB, loudness targets in LUFS, true-peak ceilings in dBTP, with unit suffixes like `target_lufs`, `ceiling_dbtp` (ADR-0170). A previous court ruled audio effects attach to the element and to ONE master/output stage on the final mix, never tracks, and that the master stage must be a small closed set, not a general bus with an effect chain. Lead workflow: narrated (mostly TTS) video over a music bed.

`verify` today measures R128 momentary loudness only to flag silent spans below −70 LUFS; it has no integrated-loudness or true-peak check. Partial render (`render --from/--to`) and `preview` (proxy resolution, same audio path) mix only the elements inside the span `[from, to)`.

Precedent: Premiere normalises the whole programme at export to BS.1770 / EBU R128 (−23) / ATSC A/85 with an optional true-peak limiter, and its Mix track takes effects; it also has clip Auto-Match (writes ordinary gain). CapCut documents no master/mix processing, only a per-clip "normalize loudness" checkbox (−23 LUFS fixed).

ffmpeg facts: two-pass `loudnorm` in linear mode hit −16.0 LUFS exactly, but silently falls back to dynamic mode when its linear conditions fail, and dynamic-mode output bytes vary with CPU SIMD features. A plain measured gain (`volume=xdB`) followed by `alimiter` is deterministic on any CPU; `alimiter` with `latency=1` adds no delay. Pass 1 (measurement) means rendering the whole mix once (audio only).

## The Question (four parts; answer each)

**Q1 — How does loudness normalisation work?**
(a) A target the renderer reaches: the document says `target_lufs: -16`; the renderer measures the mix in a first pass and applies one fixed gain.
(b) A gain the agent writes: the master stage holds `gain_db`; the agent runs `verify`, reads the measured LUFS and writes the correction.
(c) A write tool measures and writes `gain_db` (as (b), but tool-written).

**Q2 — Where does the master stage live, and what is the key called?**
(a) New optional top-level key `master: {…}`.
(b) New optional top-level key `mix: {…}`.
(c) Turn `output` into an object, e.g. `{path, audio: {…}}` (breaks every existing document).

**Q3 — What does it hold (closed set)? Choose any of:**
- `target_lufs` (integrated loudness target)
- `ceiling_dbtp` (true-peak limiter ceiling)
- output channel layout (stereo/mono)
- an `audio_effects` list (EQ/compression on the whole mix)

**Q4 — What happens when the master stage is absent?**
(a) Today's behaviour: no normalisation, no limiter; existing documents render byte-identically.
(b) A default ceiling (e.g. −1 dBTP) always applies so nothing clips.

You may reject any framing if you think the options are wrong.

## Answer format — exactly this block, nothing else

🗳️ **Juror <n>** (<the model backing you>) — **VOTE: Q1 <choice>; Q2 <choice>; Q3 <choice>; Q4 <choice>**

**Reasoning:** <why, per part>
**Trade-offs:** <what it costs, and why not the other options, per part>
