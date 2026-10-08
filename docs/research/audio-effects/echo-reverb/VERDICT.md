# Owner's blind listen: echo and reverb

Date: 2026-10-08. Clips rendered on ffmpeg 7.0.2 (see `measurements-ffmpeg7.0.2.json`).
`ab/KEY` sha256: `88dc2f560f7a31efb505e608d0e5e919786505a5cf912c6b18613b94fc8530d6`.
One listener, heard before the KEY was opened.

## The owner's words, verbatim

> echo Y has a echo , reverb hard for me to distinguish , echo-tail-x has echo same as echo-tail-y. reverb effect is hard for me to distinguish, X has some noise effect under it, Y has more higher noise

The last sentence did not say which reverb pair it described, so it is recorded unlabelled.

## Against the KEY

| Clip | Heard | KEY |
|---|---|---|
| `echo-Y` | an echo | echo (correct) |
| `reverb-X` / `reverb-Y` | hard to distinguish | reverb / bypass (not told apart) |
| `echo-tail-X` / `echo-tail-Y` | same echo in both | tail cut / 1500 ms kept (not told apart) |
| unlabelled reverb pair | X has some noise under it, Y has higher noise | not scored |

## Reading
- The echo is audible on narration.
- The cut tail was not heard on this narration at these settings, so on this one fixture a literal `tail_ms` was not shown to be worth admitting.
- The generated reverb at `mix` 0.3 / `decay_ms` 1200 was not told apart from bypass, and the one description given (noise under the voice) matches the README's worry that it reads as noise, not a room.
- This is one listener and one fixture; it settles nothing in the ADR on its own.
