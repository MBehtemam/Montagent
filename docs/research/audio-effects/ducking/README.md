# Duck: PROTOTYPE (#837). Throwaway.

Not production code, and not `duck.py`. `prototype_ab.py` reproduces what
[ADR-0177](../../../adr/0177-ducking-is-a-skill-script-that-writes-ordinary-volume-keyframes-and-the-format-stays-untouched.md)'s
script would write, on the shared narration-over-bed fixture, so the owner can judge how the duck
sounds. Run: `python3 -I prototype_ab.py` (ffmpeg only). The level arithmetic is checked separately by
`check_duck_levels.py`.

## What the clips are

`ab/X.m4a` and `ab/Y.m4a`, 20 s each: the narration over the bed, in two versions of the bed. One has
the bed as it is; the other has it ducked by ADR-0177's defaults (`under_db` -15, `over_db` -6,
`end_db` -1.5, `ramp_ms` 200, `lead_ms` 100, `join_ms` 600, no fade-out). Both are brought to
-20 LUFS by one gain and encoded AAC 160k, so loudness is not a clue.

- **The bed is raised 12 dB in both.** The fixture's own bed sits 15 LU under the voice, where a duck is
  nearly inaudible. Raised, it sits 3 LU under, about where a library track arrives against a voice, and
  that is the case a duck exists for.
- **The ramps use the renderer's `ease-in-out`** (`cubic-bezier(0.42, 0, 0.58, 1)`); held stretches are flat.
- **The speech spans come from `silencedetect`** on the narration, as `captions.py`'s `listen` does,
  because the fixture has no words file.

## What the clips cannot show

This narration has only short pauses, 382 to 442 ms, all under `join_ms` (600). By the ADR's rule a short
pause stays down so the music does not pump, so the duck goes down about 0.5 s in and stays down until
after the last word, then comes back up. **The recovery in a long pause is not exercised by this fixture.**
The keyframes the prototype wrote are in `measurements-ab.json`.

Which clip is which is in `ab/KEY`: **do not open it, or `measurements-ab.json`, until you have judged.**

## Listen for

- Is the narration clearer in one of them? Can you follow every word over the music?
- Does the bed duck smoothly at the start, or can you hear the music being turned down?
- After the last word, does the music come back up naturally?
- Is anything worse (the music vanishing, pumping, a thin or hollow moment)?

## The verdict

The owner's words go in `VERDICT.md`, verbatim, with the date, the build and the `KEY`'s hash
(ADR-0173 §8).
