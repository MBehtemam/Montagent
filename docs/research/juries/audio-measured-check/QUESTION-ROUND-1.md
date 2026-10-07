# Court packet: the audio map's measured check, round 1

Ticket: [What measured check must every audio capability's ADR name, and what runs it?](https://github.com/MBehtemam/Montagent/issues/802),
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)). The packet is
blind: it carries context and options, and no recommendation.

## Context

Montagent is a video-editing format: a JSON project document that an agent writes, `validate`
checks from the file alone, and `render` turns into an MP4 by driving the ffmpeg the user
installed (floor: ffmpeg 7.1). Audio today is one mix graph per project
(`atrim → atempo → aloop → volume → adelay → amix normalize=0`, 48 kHz stereo, AAC 160k).
An effort is now adding audio effects in CapCut's and Premiere's image: loudness normalisation,
limiter, EQ, compressor, gate, noise reduction, de-ess, crossfades, reverb/echo, pitch, voice
changers. Effects attach to elements (an ordered `audio_effects` list) and to one optional
top-level `master: {target_lufs?, ceiling_dbtp?}` stage, reached by one measured gain then
`alimiter`.

A standing rule of the effort: **every capability's ADR names a measured acceptance check**
(integrated LUFS within tolerance, true peak below the ceiling, energy below a cutoff, an
equal-power sum at a crossfade midpoint) next to the owner's ear. The owner's ear decides; the
numbers are recorded. An audio prototype is rendered through the real pipeline as a
loudness-matched A/B against the bypassed source. Where no honest number exists (reverb,
pitch), the check falls back to a smoke check.

Facts:

- **`verify`** (ADR-0117) is an existing verb. It measures the deliverable MP4 with the decoder
  (`ffprobe`, `ffmpeg`) as a witness independent of the engine: it never reads what the engine
  established. Today it runs ffmpeg's `ebur128` filter on the mixed track for a silence gate
  (−70 LUFS absolute gate). Its own docs: *"Energy is per span, not per element. The file
  carries one mixed track"*, so it cannot attribute a number to one element's effect. A
  measurement miss there is a finding classed `error` (guaranteed wrong), `review` (likely
  wrong, borrowed threshold), or `note`.
- **ADR-0172** (the master stage) says `verify` always reports the deliverable's integrated
  loudness and true peak, and raises a `review` (never an error) when they miss `target_lufs` /
  `ceiling_dbtp`. It leaves **the tolerance** to this ticket. Legitimate misses: heavy limiting
  leaves loudness below target (one gain, no compensation; a headroom-under-6-dB review predicts
  it), and AAC encoding adds inter-sample peaks after a correctly applied limiter. A
  `ceiling_dbtp` above −1 is already a review.
- **Determinism** (measured research): every candidate filter is byte-identical within one
  ffmpeg build. Across builds and CPU SIMD paths, `firequalizer`, `superequalizer`, `afftdn`,
  dynamic `loudnorm` and the pitch chain change bytes. Byte hashes hold only per (build, CPU).
- **CI** runs `cargo test` on Linux (pinned ffmpeg 7.1 floor build), macOS (Homebrew latest)
  and Windows (Chocolatey latest).
- **Picture precedent:** golden frames are compared by SSIM at a stated threshold, never byte
  equality.
- **The ADR evidence rule** (repo): a numeric claim in an ADR needs a committed re-executable
  check (a script that re-derives the number and exits non-zero when it stops reproducing),
  committed with its input data. A qualitative claim (a jury verdict, a prototype's observed
  behaviour) needs the artifact itself committed verbatim. The existing ffmpeg research is
  backed by a committed Python script of this kind.
- **Thresholds** borrowed from outside standards are cited and make a finding `review`, not
  `error` (ADR-0061).
- The lead workflow is TTS narration over a music bed.

## Questions

Answer each one.

**Q1 — Where does a capability's measured check run?**
(a) `verify` on the user's deliverable;
(b) a repo integration test that renders a fixed fixture document through the real pipeline and
asserts the number on every CI leg;
(c) both: (b) for every capability, and (a) only for programme-level properties the document
itself declares.
Also: is the build's test a port of the prototype's ADR-evidence script (same metric, same
fixture), or a separate artifact?

**Q2 — What measures?**
(a) ffmpeg's own meters (`ebur128=peak=true` for LUFS/dBTP, `astats` for peak/RMS, a filter plus
`astats` or `aspectralstats` for band energy);
(b) Rust crates inside the test (`ebur128`, an FFT crate);
(c) something else.

**Q3 — `verify`'s tolerance before a master-stage miss becomes a `review`.**
Loudness: ±0.5 LU (EBU R 128 programme tolerance), ±1 LU (R 128 live/short-form allowance,
ATSC A/85), one-sided, or other. True peak: strictly above the ceiling, or above the ceiling
plus a codec allowance (how much?).

**Q4 — How is a prototype's loudness-matched A/B produced and committed?**
The A/B is the processed document versus the same document with the effect bypassed, rendered
through the real pipeline, both brought to the same integrated loudness by one gain, with a
`measurements.json` and assertions that exit non-zero. Decide: commit the audio clips
themselves, or only the script plus inputs so the clips regenerate; labelled or blind (X/Y with
a key); and where the owner's verdict is recorded.

**Q5 — What is a "smoke check" where no honest number exists (reverb, pitch, voice
changers)?** Name the assertions it makes.
