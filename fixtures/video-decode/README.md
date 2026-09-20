# `video-decode` — the decode path's own fixture

This project exists because **the committed `en-halloween-decorating` fixture has zero
`video` elements**, and [ADR-0003](../../docs/adr/0003-general-video-editor-not-channel-tooling.md)'s
asymmetry is explicit that a channel's silence is evidence a capability is *needed*, never
that one is *unneeded*. Spec [#168](https://github.com/MBehtemam/Montaget/issues/168) names
the zero-`video` case as *"the one most likely to be missed"*: `frame` and `render` must
decode through ADR-0023's rotation pipeline, and **no ticket will discover that from the
real fixture**.

So the decode path gets a fixture of its own rather than borrowing one, and
[#212](https://github.com/MBehtemam/Montaget/issues/212) requires it by name.

## What it exercises, and why each element is here

It adds **no new bytes to the repository**. Its `source` is
`reference/en-halloween-decorating.mp4` — the published video, already committed as the
real fixture's reference — reached by a relative path, which is also a small test of
ADR-0053's *"a relative path resolves against the project file's own directory"*.

| element | what it is for |
| --- | --- |
| `whole-frame` | The plain case: one decoded frame resampled to the declared `width`×`height`, with no `clip` and no transform. The seek is `source_start` plus elapsed, which is the arithmetic `query --at`'s *offset into source* answers with. |
| `clipped-and-scaled` | `clip` as the static frame-space aperture (ADR-0025) over a keyframed `scale`, so the decoded frame, the transform and the aperture are all live at once — the combination where an order-of-operations mistake is visible rather than plausible. |
| `slowed-and-held` | `speed: 0.5` with `overrun: "hold"` and a static `opacity`. Its source span is 1000 ms played over 4000 ms, so the second half of its range is past the end of the source and every instant there must resolve to `source_end` rather than to a failed seek (ADR-0020). |
| `corner-badge` | An `ellipse` with a fill **and** an inside stroke — ADR-0014's *"the stroke falls inside the declared rect"*, on the one shape where an outside stroke would be obvious. |
| `lower-panel` | A `rect` with `radius`, which no element of the real fixture carries: the fixture is measurably square, and ADR-0014 admits the field anyway. |

## What it is not

**It is not a golden frame and it cannot falsify the format.** Every frame it produces was
produced by the thing under test, so it catches regressions and nothing else (spec #168:
*"a golden frame we render ourselves and commit is self-confirming"*). Only
`../en-halloween-decorating/reference/frame-*.png`, extracted from the published MP4,
falsifies — and that comparison belongs to
[#213](https://github.com/MBehtemam/Montaget/issues/213), where text is drawn and the
frames become comparable.

**It does not test real container/codec disagreement.** ADR-0023 records the same gap for
its own rule: rotation and PAR are *"argued and reasoned about, not measured against real
footage"*. The source here carries no display transform and square pixels, so what this
fixture exercises is the decode, the seek and the resample — not the normalisation of a
portrait phone clip, which still has no fixture anywhere in the repository.
