You are a juror on a design court for Montagent, a JSON video-document format written and edited by AI agents (rendered via ffmpeg). Answer from the context below and your own knowledge. Do not use tools, do not edit anything, do not make recommendations to the person who asked; just cast your ballot.

## Context

The format is adding audio effects (EQ, compressor, limiter, loudness normalisation, pan, de-ess, reverb, echo, pitch shift). Each effect is a member of an ordered list `audio_effects: [{"name": ..., <params>}]` on an element; gain and routing controls (`volume`, pan) are flat fields on the element. Standing rules: every value is a literal, the vocabulary is closed, `validate` can check every value (including ranges), and values are editable by exact-string replace. Precedent is looked for first in CapCut and Premiere; departures from precedent must be named.

Already decided:
- A level parameter takes the unit its precedent uses: dB for gains and thresholds, LUFS for loudness targets, dBTP for true-peak ceilings. The one exception is a keyframable gain whose identity must read as "on": `volume`, a linear multiplier (0 = silent, 1 = source level, >1 amplifies). `volume` stays linear with no dB sibling.
- Pan is −1..+1. Frequency is Hz (never kHz). Ratio is a bare n meaning n:1. Mix / dry-wet / feedback are 0..1 fractions. Pitch is semitones (fractional allowed). Effect-internal durations (attack, release, look-ahead, echo delay) are ms and may be fractional; timeline instants (`start`, `end`) stay integer ms.

Facts:
- Today no key in the format carries its unit in its name: `start`/`end` are ms, `rotation` degrees, `stroke_width` px — all bare.
- The format now has two scales for one kind of quantity (level): linear `volume`, dB for the rest. An agent that learned `volume` may write a linear value into a dB field.
- Where validate catches that: thresholds, ceilings and loudness targets have ranges at or below zero, so a linear-minded `0.125` is out of range and fails loudly. Where it doesn't: EQ band gain and compressor make-up gain legally take small positive dB values, so `"gain": 0.5` meant as "half" passes silently as +0.5 dB.
- Premiere labels its controls with units in the UI: "Gain (dB)", "Threshold (dBFS)", "Target Loudness (LUFS)". A limiter ceiling may be sample-peak (dBFS) or true-peak (dBTP), which are different measurements.

## Question

**Q4 — Do audio effect parameter key names carry their unit?**
- (a) Bare names everywhere (`gain`, `threshold`, `makeup`, `ceiling`, `target`), keeping the format's convention; rely on docs and validate ranges.
- (b) Suffix only the logarithmic levels: `gain_db`, `threshold_db`, `makeup_db`, `target_lufs`, `ceiling_dbtp`. Hz, ms, ratio and fractions stay bare because each has one scale format-wide; `volume` stays bare as the linear default. Rule: a suffix appears exactly where the format has two scales for one kind of quantity. The suffix also names the measurement (`_dbtp` true peak vs `_db`).
- (c) Suffix every unit within audio (`frequency_hz`, `attack_ms`, `gain_db`, ...), uniform within audio but unlike every timeline field.
- Or reject the framing.

## Ballot format

Answer with exactly this block and nothing else:

🗳️ **Juror <n>** (<the model backing you>) — **VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <why not the others>
