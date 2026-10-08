# Verdict: the duck (#837)

Owner, 2026-10-08, blind listen to `ab/X.m4a` and `ab/Y.m4a` (20 s each, AAC 160k, each brought to
-20 LUFS by one gain), verbatim:

> the most distinguishable diff between x and y is that music in X is higher that Y , in Y the music is moslty like a background.

`ab/KEY` (sha256 `a3a8888324d77f8aca6377a8f21162bb99b17f68235e9eb4a846105b2a1c919e`), opened after the
verdict:

| clip | what it was |
| --- | --- |
| `X` | the bed unducked (raised 12 dB, as in both) |
| `Y` | the bed ducked by ADR-0177's defaults |

## What it shows

- **The difference was heard, and read the right way round.** The owner picked out the louder music in
  `X`, the unducked clip, and described `Y`, the ducked one, as the music "mostly like a background",
  which is what a duck is for.
- **The numbers agree.** Before the match to -20 LUFS, `X` measured -16.4 LUFS and `Y` measured
  -18.1 LUFS, which is the narration alone (-18.1 LUFS): under speech the ducked bed adds nothing
  measurable to the mix, while the unducked bed adds 1.7 LU. Matching therefore took `X` down 3.6 dB and
  `Y` down 1.9 dB.

## What it does not show

- **The owner did not remark** on the narration's clarity, on the duck sounding turned down at the start,
  on pumping, or on the music coming back after the last word. Nothing was reported wrong, which is not
  the same as having judged each.
- **The recovery in a long pause was not exercised.** This narration's pauses are 382 to 442 ms, all
  under `join_ms` (600), so by the ADR's own rule the duck went down 0.5 s in and stayed down until after
  the last word. The `over_db` level and the return in a pause are untested by ear.
- One listener, one fixture, a bed raised 12 dB to make the case audible, and a loudness-matched pair.

Build: ffmpeg 6.1.1-3ubuntu5 (Linux). Numbers and the keyframes the prototype wrote:
`measurements-ab.json`. The prototype (`prototype_ab.py`) is throwaway.
