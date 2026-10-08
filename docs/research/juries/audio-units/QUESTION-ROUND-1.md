You are a juror on a design court for Montagent, a JSON video-document format written and edited by AI agents (rendered via ffmpeg). Answer from the context below and your own knowledge. Do not use tools, do not edit anything, do not make recommendations to the person who asked; just cast your ballot.

## Context

The format is adding audio effects (EQ, compressor, limiter, loudness normalisation, pan, de-ess, reverb, echo, pitch shift). Each effect is a member of an ordered list `audio_effects: [{"name": ..., <params>}]` on an element; gain and routing controls (`volume`, pan) are flat fields on the element. Standing rules: every value is a literal, the vocabulary is closed, `validate` can check every value, and values are editable by exact-string replace. Precedent is looked for first in CapCut and Premiere; departures from precedent must be named.

Facts:
- No key in the format carries its unit in its name. Times are `start`/`end` in integer milliseconds (ADR-0005: time is absolute integer ms; that ADR governs instants on the timeline, which `shift` moves and which are sampled on a frame grid). `rotation` is degrees, `stroke_width` px.
- Linear values today: `volume` (a keyframable linear multiplier: 0 = silent, 1 = source level, >1 amplifies), `opacity` 0..1, `saturation{amount}` (1 = identity), `speed` (rate multiplier).
- ADR-0055 rejected dB for `volume`: the scale is logarithmic and its identity value would be `0`, which means "off" in every other field in the format; a reader cannot tell the level at a keyframe without log arithmetic; "the vocabulary is a UI for the agent, not an audio engineer". It also rejected a 0–100 percentage for `volume` as "the same freedom with a different decimal point, clashing with every other scalar in the format". Fades and (planned) a ducking write tool are expressed as `volume` keyframes.
- Premiere precedent: EQ gain dB, frequency Hz, width as Q; compressor/gate threshold dBFS, ratio n:1, attack/release ms, make-up dB; Hard Limiter ceiling dB, look-ahead ms; loudness targets in LUFS, true peak in dBTP; pan/balance −100..+100 unitless (negative = left); mix, dry/wet and feedback in %; pitch in semitones, cents −100..+100 and a ratio 0.5–2; auto-ducking "Duck Amount" in dB. CapCut documents almost nothing.
- ffmpeg is mixed: `equalizer` gain in dB, `loudnorm` in LUFS/dBTP, but `acompressor` threshold and make-up, and `alimiter` limit, are linear. The renderer converts either way.
- Real compressor/limiter attack times are often below 1 ms (fast attacks 0.1–1 ms).

## Questions — answer all three

**Q1 — What one rule decides the unit of a level parameter** (EQ band gain, compressor threshold and make-up gain, limiter ceiling, loudness target)?
- (a) Everything linear, consistent with `volume`.
- (b) dB for every new level (LUFS for loudness targets, dBTP for true-peak ceilings); `volume` stays the one linear level, as a documented exception, with no dB sibling field.
- (c) Reopen `volume` to dB too, so the whole format has one level scale.
- Or reject the framing.

**Q2 — Pan scale.**
- (a) Premiere's −100..+100.
- (b) −1..+1, 0 = centre, negative = left.
- Or reject the framing.

**Q3 — Units of the non-level parameters.** Proposed package: frequency in Hz as a plain number (never kHz); effect-internal durations (attack, release, hold, look-ahead, echo delay) in ms; ratio as a bare number n meaning n:1, n ≥ 1; mix / dry-wet / feedback as a 0..1 fraction rather than %; pitch in semitones with fractional values allowed instead of a separate cents field. Sub-question: must effect-internal durations be integer ms like timeline times (ADR-0005), or may they be fractional ms?
Vote on the package (accept / amend — say which part and how) and answer the sub-question (integer / fractional).

## Ballot format

Answer with exactly one block per question, in this shape, and nothing else:

🗳️ **Juror <n> — Q<k>** (<the model backing you>) — **VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <what it costs, or why not the others>
