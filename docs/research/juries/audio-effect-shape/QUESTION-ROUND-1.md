You are a juror. Answer as an AI agent that authors and edits Montagent projects: you read the JSON document, edit it by exact-string replace, run `validate`, and render. Judge each option by how well you, as that agent, would work with it. Do not use any tools. Do not edit anything. Answer only in the ballot format at the end.

## Background (facts, not recommendations)

Montagent is a video-editing format: a JSON document an agent writes, which a Rust renderer turns into video. Its standing rules: every value is literal, the vocabulary is closed, every value is checkable by `validate`, and every value is editable by exact-string replace. Precedent is looked for first in CapCut and Premiere.

- **Visual effects** (ADR-0040) live in `"effects": [{"name": "blur", "radius": 4}, ...]`, an ordered list on the element. Order is meaningful, and two members of the same name are allowed. After the list come `opacity`, then the blend mode (ADR-0147), both flat fields. ADR-0040 says audio effects, if they arrive, are their own vocabulary on audio-bearing elements, not a branch of `effects`.
- **Audio today** (ADR-0055) has one control: `volume`, a flat field on `audio` and `video` elements, peer to `speed` and `overrun`. It is a linear multiplier (0 is silent, 1 is the source level) and is keyframable as `[{"t":..,"v":..,"ease":..}]`. A fade is two keyframes. A `video` element's embedded audio uses that same `volume`; there is no `mute`.
- **Today's per-element mix graph** (ffmpeg): `atrim → atempo (speed) → aloop (overrun: loop) → volume → adelay (placement) → amix normalize=0`, at 48 kHz stereo.
- **The effort under way** adds audio effects in CapCut and Premiere's image: loudness normalisation, EQ (high-pass, shelves, parametric bands), compressor, limiter, gate, noise reduction, de-ess, reverb/echo, pitch shift, voice changers, pan/balance, and channel operations (mono, swap, one channel). Effects attach to the element, and to one master stage on the final mix, which is decided separately. Track-level effects are ruled out. The lead workflow is narrated video (TTS lines) over a music bed. A planned **ducking write tool** will compute and write ordinary `volume` keyframes on the music element under each narration line.
- **Premiere precedent:** a clip has a few fixed, intrinsic audio controls (Volume, Channel Volume, Panner) plus an ordered rack of standard effects, each with typed, keyframable parameters. CapCut's mechanisms are mostly unpublished.
- **ffmpeg facts:** in an audio chain, order changes the result. EQ then compressor differs from compressor then EQ, and a limiter must come last to hold its ceiling. Echo (`aecho`) lengthens the stream with a tail.

## The questions

**Q1. Where does an audio effect sit on an `audio` or `video` element?**
- (a) One ordered list of tagged audio effects, under its own field name. `volume` stays flat as today, and everything else, pan and channel operations included, goes in the list.
- (b) Only flat fields, peers of `volume`, with the renderer fixing the processing order.
- (c) A mix: an ordered list for signal-shaping processing (EQ, dynamics, restoration, creative), and flat fields for gain and routing controls applied at a fixed point (`volume`, and later pan/balance and channel operations).

**Q2. Where do `volume` and the effect chain sit in the mix graph?** There are two sub-choices:
- **Q2.1, chain against `volume`:** (a) pre-fader: `volume`, then the chain; or (b) post-fader: the chain, then `volume`.
- **Q2.2, chain against `speed`/`aloop`:** (a) before `aloop`, so each loop repeat is processed separately and an echo tail is cut at every seam; or (b) after `aloop`, so the chain sees the continuous stream at playback rate.

**Q3. What does a `video` element's embedded audio carry?**
- (a) The identical audio-effect field(s) as `audio`, applied to its embedded track, living alongside the visual `effects` list.
- (b) Audio effects only on `audio` elements. To process a video's sound, the author silences the video and adds a separate `audio` element on the same source.
- (c) Something else (say what).

A juror may reject the framing of any question outright; that is a valid ballot.

## Ballot format

Give one block per question (Q1, Q2 with both sub-votes, Q3), exactly in this form:

🗳️ **Juror <n>** (<the model backing you>) — **Q<k> VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <why not the others, and what your choice costs>
