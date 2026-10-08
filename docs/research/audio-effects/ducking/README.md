# Ducking: PROTOTYPE (#837). Throwaway.

Not production code. `prototype_ab.py` builds the narration-over-bed fixture's mix the way
`crates/montagent-core/src/verbs/render.rs` does (aformat, atrim, asetpts, `volume`, adelay,
`amix normalize=0`, 48 kHz stereo, AAC 160k) and applies the duck that
[ADR-0177](https://github.com/MBehtemam/Montagent/pull/836) decides, to the bed's `volume`. Run:
`python3 -I prototype_ab.py` (ffmpeg only). It is `captions.py`'s `ducked()` with the levels spelled in dB.

The spans are `silencedetect` on the narration (−40 dB, 0.15 s), written out as explicit spans in
`measurements-ab.json`. The fixture has no words file, and this prototype is not the script.

## What the clips are

Each clip is the full 20 s of the fixture. Every clip is brought to −20 LUFS by one gain and encoded
AAC 160k, so loudness is not a clue. In each pair, which of X and Y is which is in `ab/KEY`, and the
numbers are in `measurements-ab.json`: **do not open either until you have judged.**

- **Pair 1, as the ticket asks.** The fixture as it is. One clip is the bed unducked, the other the bed
  ducked by ADR-0177's defaults (`under_db` −15, `over_db` −6, `end_db` −1.5, `ramp_ms` 200,
  `lead_ms` 100, `join_ms` 600).
- **Pair 2, with a bed that competes.** The fixture's bed sits 15 LU under the narration, so a duck on it
  is nearly inaudible (pair 1 may be hard to tell apart for that reason). Here the bed is raised until it
  sits 6 LU under the narration, which is where a duck earns its place. Unducked against the defaults.
- **Pair 3, the pump.** On that raised bed: the defaults against `join_ms` 300. The fixture's pauses
  are 340 to 440 ms, shorter than 2 × (`lead_ms` + `ramp_ms`) = 600, so at the defaults every pause is
  merged away and the bed stays down from the first word to the last (one flat dip). At `join_ms` 300
  the bed starts to rise in each pause and is pushed down again before it gets there.

## Listen for

- **Pairs 1 and 2:** is the narration clearer in one of them? Does the bed come back naturally after the
  last word (the end of the clip), or does it jump? Does anything sound worse in either (a dip, a gap, a
  thin or hollow moment)?
- **Pair 3:** does the bed pump in the pauses in one of them, a swell and a dip between sentences? Is
  that the one the ADR's `join_ms` default is meant to prevent?

## Honest approximations

- `volume` is applied as a per-sample gain envelope (`amultiply`), not through the engine's timed
  commands. ADR-0175 says the engine is sample-exact on a keyframe's instant; so is this envelope, but
  the engine is not exercised.
- `ease-in-out` is taken to be CSS's `cubic-bezier(0.42, 0, 0.58, 1)`. The engine's curve was not read.
- The `ffmpeg` on this machine is 6.1.1, below the 7.1 floor of ADR-0115. It is recorded in
  `measurements-ab.json` and goes into the verdict.
- One listener, one fixture, 20 s clips.

## The verdict

The owner's words go in `VERDICT.md`, verbatim, with the date, the build and the `KEY`'s hash
(ADR-0173 §8). It is written after the listen.
