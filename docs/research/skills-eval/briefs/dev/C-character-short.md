# Brief C: mascot character short

**8 seconds · 1920×1080 · 30 fps · 16:9.** Deliver one MP4 with sound.

Hoot, Montagent's owl mascot, introduces the product in one line, standing in a study. It
is a cartoon, so it lives or dies on the character feeling alive: the beak moves with the
words, the owl blinks and gestures, and it never comes apart at the joints.

Everything you need is in the asset pack; its `README.md` lists it. The owl is a cut-out
rig in `character/`: its parts, their pivots and parents, and its mouths are described in
`character/rig.json`. The line is `character/voice/line-1.wav`, with its words, visemes and
their timings in `character/voice/line-1.json`. The set is `character/study.png`. The
product pictures are in `stills/` and the brand is in `brand/`.

## Beat by beat

| When | What happens |
|---|---|
| 0–1 s | The study. The owl hops in from the side and lands on the floor, arms at its sides. |
| 1 s | The line starts: "Hi, I'm Hoot! Your agent writes the video as a file, and Montagent turns it into a movie." |
| "Hi… Hoot" | The owl waves: one arm raised, its forearm swinging back and forth from the elbow. |
| "…as a file" | A card with `stills/session-01.png` on it pops in beside the owl, and the owl gestures towards it with the arm on that side. |
| "movie" | The card's picture changes to `stills/session-02.png`, the video that session made. |
| ~7 s | The brand lockup lands above the owl, and the owl lifts both arms in a small cheer. |
| throughout | The beak moves with the words, and the owl blinks now and then. |

## Acceptance checklist

- [ ] The owl is built from the rig's parts and holds together: no gap or loose part at the neck, shoulders or elbows on any frame.
- [ ] The owl stands on the study's floor and is the frame's main subject, at least half the frame tall.
- [ ] The beak shapes follow the line's visemes: it moves while the voice speaks and is at rest in the pauses and after the line ends.
- [ ] The wave swings the forearm from the elbow at least twice while the upper arm stays raised.
- [ ] The owl blinks at least twice, each blink lasting only a few frames.
- [ ] The card pops in with a small overshoot, and the owl's gesture points at it.
- [ ] The card's picture changes on "movie", not before.
- [ ] The lockup is legible and on screen from about 7 s to the end.
- [ ] The voice is clear and starts at 1 s; nothing is clipped at the end.
