# `probe` reading a source's real frame timestamps

This project does not add a frame-timestamp reading to `probe` (a source's
frame grid, or the list of its frame start times). The idea would be to let
the seek clamp work by arithmetic instead of an `ffmpeg` filter.

## Why this is out of scope

[ADR-0096](../docs/adr/0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md)
sends *"which frame is this source showing at t?"* to `ffmpeg` as a
`-copyts` + `select='lte(t,…)'` filter, because neither frame rate in
[ADR-0011](../docs/adr/0011-tool-surface-reads-checks-renders.md)'s quad is the
source's grid. The filter is already **exact**, on constant, fractional and
variable frame rates and on dropped frames alike. A timestamp reading would
make the clamp faster, not more correct:

- **The saving is small.** §2 measured the bounded filter at ~143 ms per seek
  against ~116 ms for the plain seek it replaced: about 20%, and ADR-0096 judged
  that affordable.
- **It would be `probe`'s first expensive reading.** `ffprobe -show_frames`
  reads every packet of the file. The ADR-0069 sidecar would cache it once per
  source, but `probe` is built on cheap readings.
- **A variable-frame-rate source has no grid.** The reading would have to be the
  timestamp list, which has no size limit and has no rule for being stored in a
  cached sidecar, or an "off grid" answer, which gives the clamp nothing to use.
- **The one gain in correctness is narrow.** A source whose probed stream
  duration overstates its real last frame is caught only mid-render, at
  `E-NOT-PAINTED-UNDECODABLE` (#413's closing note). That is already an `error`,
  and ADR-0093 withholds the deliverable.

**The per-frame cost is elsewhere.** `render` paints each frame of a video
element with one `ffmpeg` process per frame (`decode::frame_at`). `decode::frames_from`
measured that at ~13 s against ~0.4 s for one process per run (140 frames at
1080p, about 30×), and ADR-0127 makes a streamed run exact **without** any
timestamp reading. If render speed on video sources matters, streaming decode
in `render` is the change to make, not this reading. That is tracked as #532.

## When to revisit

Only with evidence the ~20% seek cost matters once `render` streams, or if the
narrow case above turns up in practice and needs a pre-render refusal.

## Prior requests

- #415 — "Should probe read a source's real frame timestamps?"
