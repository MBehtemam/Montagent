# Verdict: the audio crossfade (#833)

Owner, 2026-10-08, blind listen to `ab/pair1-X.m4a`, `ab/pair1-Y.m4a`, `ab/pair2-X.m4a` and
`ab/pair2-Y.m4a` (10 s each, AAC 160k, each brought to -20 LUFS by one gain), verbatim.

**Pair 1 (music into music):**

> pair 1 both x and y seems theres diff but its hard for me to say what . pair x suddenly  raise  the audio , and the y is same , not sure seems y is slighter higer than x

**Pair 2 (the same speech on both sides):**

> maybe p2 x at first is a little lowe , but overall both are almost same to me

`ab/KEY` (sha256 `e99bc52bd57c469a0f63f27a2240c4ff5fffcf955118840aea5d6132c3828163`), opened after both
verdicts:

| clip | what it was |
| --- | --- |
| `pair1-X` | no `audio` key: today's mix |
| `pair1-Y` | `"audio": "constant_power"` |
| `pair2-X` | `"audio": "constant_power"` |
| `pair2-Y` | `"audio": "constant_gain"` |

## What it shows

- **Pair 1: heard, and in the direction the ADR predicts.** The owner picked out the second clip
  arriving "suddenly" in `X`, which is today's mix, where both elements sound at full level through
  the window and each starts or stops at its own edge. `Y`, the crossfade, was heard as the smooth
  one. "Y slightly higher" fits the numbers: matching `Y` took +12.9 dB against `X`'s +12.1 dB,
  because `X`'s doubled overlap raised its measured loudness.
- **Pair 2: a small difference, not named.** `constant_gain` against `constant_power` on the same
  speech was "almost the same". "X a little lower at first" fits the numbers: `X` (`constant_power`)
  carries the +3 dB bump at the midpoint, so matching its whole-clip loudness took -3.3 dB against
  `Y`'s -2.6 dB, which leaves the rest of `X` 0.7 dB lower. The bump itself was not picked out.
- **So:** a crossfade against none is clearly audible on music. The choice between the two curves
  on identical speech, over a 2 s window, was second-order to this ear. The ADR's measured claim
  (the 3 dB midpoint bump, `check_audio_crossfade_curves.py`) stands; only its audibility here is
  small.

## What it does not show

- It does not show the curve guidance is wrong, or that it is right: a 2 s window on one voice is
  one case. Longer windows, music on both sides at the same level, or louder material may make the
  bump easier to hear.
- It is one listener, on one fixture, on a loudness-matched pair (the match removes the level cue
  that a doubled overlap or a bump would add).
- Not listened to: `exponential` (not in the ADR), `audio_crossfade` with a `video` side, and a
  crossfade over a window with a volume keyframe inside it.

Build: ffmpeg 6.1.1-3ubuntu5 (Linux). Numbers: `measurements-ab.json`. The prototype
(`prototype_ab.py`) is throwaway.
