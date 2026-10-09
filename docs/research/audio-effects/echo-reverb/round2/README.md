# Reverb, round 2: blind A/B (prototype, throwaway)

Round 1 ([../README.md](../README.md), [../VERDICT.md](../VERDICT.md)) used a generated-IR reverb at decay 1200 ms, mix 0.3.
The owner could not tell it from bypass and heard only 'some noise under it'. This round asks one question: **can a
generated-reverb candidate sound like a room at all?** Three candidates, much more audible settings, with the likely
causes of the 'noise' impression addressed. What the candidates are, and their numbers, are in the KEY. The owner decides.

## What to listen to

Three pairs, each X vs Y, one of which is the unprocessed source and the other a candidate (order hidden). Same
narration-over-bed fixture as round 1: 48 kHz stereo, AAC 160k, 20 s, every clip loudness-matched to -20 LUFS (decoded
-20.0 to -20.1), so level is no clue. The narration's last word ends at 19.425 s and the effect is cut there, as in round 1;
the bed runs on to 20 s unchanged.

| Pair | Files | Question for the ear |
|---|---|---|
| A | `ab/r2-A-X.m4a`, `ab/r2-A-Y.m4a` | Which one has a space around the voice? Does it sound like a room, or like noise on the voice? |
| B | `ab/r2-B-X.m4a`, `ab/r2-B-Y.m4a` | Same. |
| C | `ab/r2-C-X.m4a`, `ab/r2-C-Y.m4a` | Same. |

For each pair please say: (1) which of X and Y has the effect, or 'cannot tell'; (2) if you can tell: room, noise, echo, metallic,
muddy, something else, in your own words; (3) if you can tell: is it a sound you would want on narration at all, any amount?

**Recommended order: pair C, then pair B, then pair A.** In a pair, X, then Y, then X again, on headphones. The gaps in the
narration (about 5.8, 7.9, 13.5, 15.4 s and after the last word) are where a room would be easiest to hear.

Do not compare file sizes or hash the files: the unprocessed clip is the same audio in all three pairs.

## Sealed until you have judged

- `KEY.md`: which clip is which, the candidate parameters, the objective decay (RT60-like) figures, latency, drift, smoke checks, and the reasoning behind the order above. sha256 `00504e3ca893770fb91bf3e3ff2d1e0aef2f5109404bcc771b22a91a13ac2422`.
- `measurements-round2.json`: the same measurements raw, with the labels.
- `r2_reverb.py`: the builder; it contains the candidate parameters, so it also reveals which is which.

The owner's words go in a `VERDICT.md` here, verbatim, with date, build and the KEY's hash, as in round 1.

## Reproduce

Needs ffmpeg and numpy. `r2_reverb.py` imports (does not edit) `../prototype_echo_reverb.py`. From this folder:
`python3 -I r2_reverb.py calibrate|check|ab|seal --ffmpeg PATH [--others P1,P2]` (`seal` merges the temporary results into
`KEY.md` and the JSON, then deletes them). The committed files were made with ffmpeg 7.0.2 and compared against 6.1.1 and 7.1.5.
