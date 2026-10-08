# Audio crossfade: PROTOTYPE (#833). Throwaway.

Not production code. `prototype_ab.py` hand-builds the per-element mix graph of
`crates/montagent-core/src/verbs/render.rs` with [ADR-0176](../../../adr/0176-a-transition-carries-the-audio-across-its-cut-in-one-field-and-an-audio-only-crossfade-is-a-transition-kind.md)'s
stage in the slot it decides (after `volume` and `pan`, before `adelay`), and renders a blind A/B on the
shared fixture. Run: `python3 -I prototype_ab.py` (ffmpeg only).
The curve numbers are checked separately, by `check_audio_crossfade_curves.py`.

## What the clips are

Each clip is 10 s, cut from 5 s to 15 s of a longer timeline, with a 2 s transition window in the
middle (9 s to 11 s of the timeline, so about 4 s in). Every clip is brought to -20 LUFS by one
gain and encoded AAC 160k, so loudness is not a clue.

- **`ab/pair1-X.m4a`, `ab/pair1-Y.m4a`: music into music (two different signals).** The bed
  bridges into a later part of itself. One clip has no `audio` key, which is today's mix: both
  elements sound at full level through the whole window and each simply starts or stops at its own
  edge. The other has `"audio": "constant_power"`.
- **`ab/pair2-X.m4a`, `ab/pair2-Y.m4a`: a split clip (the same signal on both sides).** The
  narration is cut in two with a 2 s overlap on identical samples, which is what a cutaway back to
  the same take sounds like. One clip has `"audio": "constant_power"` and the other
  `"audio": "constant_gain"`.

Which clip is which is in `ab/KEY`, and the per-clip numbers are in `measurements-ab.json`:
**do not open either until you have judged.**

## Listen for

- **Pair 1:** does the join disappear in one of them? Is there a lurch, a doubled loudness, or an
  abrupt start or stop at the window's edges in the other?
- **Pair 2:** does the level stay steady through the overlap in one, and swell or bump in the middle
  of it in the other? ADR-0176 claims `constant_gain` is steady here and `constant_power` bumps
  about +3 dB at the midpoint.
- Is anything worse in either clip (a dip, a gap, a thin or hollow moment)?

## The verdict

The owner's words go in `VERDICT.md`, verbatim, with the date, the build and the `KEY`'s hash
(ADR-0173 §8).
