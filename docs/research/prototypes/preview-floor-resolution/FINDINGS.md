# Preview floor resolution — findings

Prototype for [#117](https://github.com/MBehtemam/Montagent/issues/117), graduated from
[ADR-0046](../../../adr/0046-proxy-preview-target-is-720p-long-edge-capped.md), which fixed
the proxy-preview target at 720p and deferred the hard-refuse floor to this ticket.

## Method

No production Montagent renderer exists yet, so this prototype uses the committed fixture's
own published reference render (`fixtures/en-halloween-decorating/reference/en-halloween-decorating.mp4`,
1080×1920, 25fps) as ground truth for the exact project timeline — same frame, same text,
same timing a real Montagent render would produce. Four timestamps were chosen from the
fixture's own declared element sizes (`en-halloween-decorating.montagent.json`) to stress its
smallest and densest text:

| label | t | element(s) | size |
|---|---|---|---|
| intro | 0.5s | handle-text + chip-text | 34px / 52px, on screen the full 65s runtime |
| word-08 | 45.0s | word-08-bridge + word-08-target | 35px / 49px — the smallest caption run in the file |
| sentence-05 | 11.5s | sentence-05 | 55px — the fixture's ordinary caption size |
| quiz | 62.0s | sentence-quiz | 55px — same card geometry, near the tail |

Each native frame was downscaled bilinear to four candidate tiers (long-edge capped per
ADR-0046's rule: 720p 720×1280, 540p 540×960, 360p 360×640, 240p 240×426), then scaled back
to native size with the same filter — simulating how a preview player displays a proxy frame
in a fixed-size viewport, rather than judging raw small bitmaps at incomparable sizes. Regions
around each stress element were cropped and zoomed 3× (nearest-neighbor) into five-panel
comparison strips (`crops/*-strip.png`), tier order native → 720p → 540p → 360p → 240p.
`run.sh` regenerates everything from the committed fixture.

## Judgment

Presented as an interactive [artifact](https://claude.ai/artifact/TYt6SeJCLxPBKQ99JFgQ98) for
live discussion, then put to a three-juror court (Opus, Haiku, Fable), each shown the same
four strips with no visibility into the others' ballots. **Unanimous 3/3** on both questions.

## Result

**360p is the hard-refuse floor.** The 55px standard subtitle cards (`sentence-05`, `quiz`)
stay legible through every tier tested, including 240p — they don't force the floor. What
does: the handle/chip badge and the smallest caption line (34–35px) read cleanly at 360p and
degrade at 240p from soft-but-distinguishable to glyphs losing their contour, readable only
by already knowing the string — the exact position an agent checking its own unfamiliar
render is not in.

**Content-dependence is a finding, not the mechanism.** Legibility tracks the smallest text
on screen at a given instant, not resolution alone — but this fixture's small text (the
handle/chip) persists through nearly the whole runtime, so a per-instant floor would buy no
real headroom here, only a threshold the agent can't predict before calling `preview`. A
fixed number from the worst case actually observed is conservative and shippable now.

**The refusal must name the floor and the reason** — a bare failure below 360p is
indistinguishable from a render error or bad input, and the caller's only lever is the
resolution it requested.

Recorded in [ADR-0050](../../../adr/0050-preview-hard-refuses-below-360p.md).
