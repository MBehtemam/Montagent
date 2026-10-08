# Court packet: the master stage's true-peak ceiling (map ticket "How does the master stage hold a true-peak ceiling on the AAC delivery, and what is the allowance?")

You are a juror. Answer from this packet alone. Do not use tools, edit anything, or address the Judge. Reply in exactly this block for EACH of the three questions, one block per question:

🗳️ **Juror <n>** (<the model backing you>) — **VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <what it costs; for multiple choice, why not the others>

You may reject a question's framing if every option is bad; that is a valid ballot.

## Context
Montagent renders a declarative video project (a JSON file edited by LLM agents with ordinary file tools: exact-string replace). Its format has four invariants: literal values, a closed vocabulary, every field checkable by a `validate` tool, and editable by exact-string replace. An agent writes the project, then calls `validate`, `render`, and `verify`/`review` tools that report findings. Audio is mixed by ffmpeg (48 kHz stereo, native `aac` encoder at 160k; ffmpeg is supplied by the user, floor build 7.1, CI also runs Homebrew and Chocolatey builds).

An accepted decision (ADR-0172) adds an optional top-level `master: {target_lufs?, ceiling_dbtp?}`. The renderer reaches `target_lufs` with one gain measured on the whole programme, then, if `ceiling_dbtp` is set, `alimiter` (limit = the ceiling converted from dBTP to linear, `latency=1`) follows. `verify` always reports integrated loudness and true peak; a miss against a declared key is a `review` finding, never an error. An agent reading `ceiling_dbtp: -1` takes it to mean the delivered file will not exceed -1 dBTP. ADR-0173 (proposed) sets `verify`'s tolerance for `ceiling_dbtp` at "true peak of the decoded AAC > ceiling + 0.5 dB", provisional until measured. A borrowed external figure never gates a test; it may set a `verify`/`review` threshold.

## Measured facts (CI, Linux floor n7.1.5, macOS Homebrew 9.0.1, Windows Chocolatey 9.0.2)
Signals driven 3, 6 and 10 dB over the -1 dBTP ceiling, through gain, the limiter, native aac 160k, decode; true peak by ebur128=peak=true. Overshoot past the ceiling, decoded AAC:
- Stage as ADR-0172 words it (alimiter, latency=1; also `level` defaults ON, which auto-levels the output back up, so `level=0` is required either way): worst +3.8 / +3.3 / +3.3 dB. Cause: alimiter limits SAMPLE peaks, so an fs/4 sine at 45 degrees gets +3 dB of inter-sample peak before any encoder.
- Same limiter run at 4x the sample rate (aresample=192000, alimiter level=0 latency=1, aresample=48000): pre-encoder PCM overshoot <= +0.2 dB; no added delay (impulse stays at its sample), same length, byte-identical across runs on one build. After AAC: worst +1.8 dB on all three legs (clipped-sine bursts, seeded pink noise at +10 dB over); fs/4 sine <= +0.3; the real narration-over-bed fixture (TTS voice over a music bed) <= +0.5 (0.3, 0.3, 0.5 across legs).
- Encoder drift across builds exists: on the plain stage macOS gave +2.4 on the fixture at 10 dB over where Linux and Windows gave +0.9.
- Not yet measured: oversampling factors other than 4x; whether the resampler is byte-identical across builds; a fixed limit margin below the ceiling without oversampling.

## Question 1 (multiple choice): What does `ceiling_dbtp` promise?
(a) The delivered file: the decoded AAC stays at or under the ceiling plus an allowance. (This is how `verify` already reads it.)
(b) The limiter's own output: the mix before the encoder stays at or under the ceiling; the AAC is outside the promise.
(c) Something else (say what).

## Question 2 (multiple choice): Which stage does ADR-0172 specify for the closing limiter?
(i) `level=0` only (the minimum fix).
(ii) `level=0` plus 4x oversampling around the limiter.
(iii) `level=0`, with the limit set a fixed margin below the ceiling (e.g. 1 dB), no oversampling.
(iv) Something else (say what).

## Question 3 (multiple choice): What sets the allowance in `verify`'s `ceiling_dbtp` review (currently +0.5 dB)?
(alpha) The worst adversarial case plus margin (about +1.9 dB under the oversampled stage): one loose tolerance.
(beta) The shared narration-over-bed fixture plus margin (+0.5 dB under the oversampled stage), stated plainly as not covering clipped bursts or heavy noise on AAC.
(gamma) A split: `verify` reviews above +0.5 dB; the ADR records +1.8 dB as the known adversarial worst case and states which signals reach it.
(delta) Something else (say what).

Consider each as the agent who authors a project, reads `verify` findings, and acts on them with exact-string edits would live with it: what it must be told, what it can act on, what misleads it.
