# Field report: Montagent's first real use

**Status:** qualitative artifact, committed verbatim. Not a source of numbers.

An agent assembled a real short (images, video, text, speech) with Montagent and
wrote the account below afterwards. **The session it came from no longer exists**
— there is no project file, no logs, no rendered output and no repro. This text
is all that survives.

## How this artifact may be cited

Under the evidence rule in [`docs/agents/domain.md`](../../agents/domain.md), this
document is a **qualitative** artifact and nothing more:

- **Citable as written:** the design judgments and the account of the workflow —
  that the `frame` verb printing the resolved element stack beside the image was
  the most useful thing in the tool; that the one-element-per-line convention did
  not match a generate-then-`fmt` workflow; that font vendoring left the author
  without a bold weight; that the tool was slower for a format already encoded in
  a retired script and decisively better for what that script could not express.
- **Not citable:** every number in it. `1.0000141`, `1003.72px`, `20.5`
  characters/second, `886` frames, `19` speech elements and `51` elements cannot
  be re-derived from anything this repository holds. An ADR resting on any of
  them would repeat the failure that
  [ADR-0006](../../adr/0006-validate-reports-facts-and-render-enforces.md) is
  flagged inline for.
- **Not the ground of the audio finding.** The report's central complaint — a
  render that emitted a complete video with every audio element missing, at exit
  0 — is **independently re-derivable from committed source**, and that reading,
  not this document, is what the render-integrity work rests on. This report is
  what prompted the investigation, not what supports its conclusion.

## The report, verbatim

> Where it actually earned its place. Three defects it caught that I would have shipped:
>
> - R-KEYFRAME-UNREACHED on all three Ken Burns pushes. My closing keyframe sat at the element's end, and since ranges are half-open, it was never sampled — the push was landing on 1.0000141 instead of 1.0. Invisible to the eye, and it silently broke the one property the ping-pong exists for (last frame equals first, so the beat loops). That's not a typo it caught, it's a class of mistake, and I'll now write every fade and settle differently.
> - measure told me the hook overflowed its box at 1003.72px into 920 — before any render. And critically, it reports the stroked extent, which mattered because I was faking font weight with stroke.
> - R-CAPTION-PACE flagged the loop tail at 20.5 characters/second against a cited Netflix/BBC threshold of 20. I would never have computed that, and the fix cost 100ms.
>
> The frame tool printing the resolved element stack next to the image is the single best design decision in the thing. I found the parked hook colliding with the @handle row and could immediately see which of twelve elements produced it. Looking at a rendered frame without knowing what made it is exactly how you fix the wrong element.
>
> Where it cost me, and the big one is real. The MCP render wrote a complete, correct-looking 886-frame video with no audio at all — all 19 speech elements dropped — and returned exit 0. The CLI rendered the identical file correctly. That's probably an environment difference in the server process rather than a design flaw, and to its credit the report said none mixed rather than hiding it. But the design point stands: Montagent's whole self-description splits the world into "the file is internally legal" (which it guarantees) and "the file says what you meant" (which it explicitly doesn't). A render that discards every audio element is a failure in the first half. It should refuse, not narrate. On a channel whose entire product is the sound of a word, that's the one bug that must never be a warning.
>
> Two smaller frictions. Font vendoring is right in principle — freeze the bytes so a clean machine renders the same video — but it made this repo's actual brand typeface unusable and left me without a bold weight. And the no-CRUD-API / one-element-per-line convention is built for an agent doing exact-string replaces; I did zero of those. Fifty-one elements with keyframes isn't something you hand-write, so I wrote a generator and let fmt clean up after it. The convention isn't wrong, it just wasn't the workflow the work wanted.
>
> The verdict I'd actually give. For reproducing this repo's existing Shorts format, Montagent was slower than the retired ffmpeg script would have been — that script already encodes the beat sheet, and I had to read the constants out of it and re-derive the layout by hand. Where it won decisively was everything the format doesn't already cover: the five-word wall with a different motion signature per genre is simply not expressible in build_shorts_video.py, and it's the best beat in the piece.
>
> So: a worse tool for building the thing you've already built, a much better one for building the thing you haven't. Given this repo has six channels shipping one format, I'd keep the declarative model — the timeline being a generated, checkable artifact is a genuinely better fit for the "nothing may re-type a timing" invariant than a Python script that computes the same numbers invisibly. But I'd want that audio bug understood before a second piece goes through it, and the ffprobe post-render check stays in the runbook regardless.
