# REPORT — repair of project.json (en-halloween-decorating)

## What I changed

One kind of change, applied to exactly the eight elements the validator flagged:
I deleted the retired `gravity` field. Nothing else — no values, no ordering,
no whitespace — was touched.

| Element | Removed | Everything else |
|---|---|---|
| `photo-05-intro` | `"gravity":"top"` | unchanged |
| `photo-05` | `"gravity":"top"` | unchanged |
| `photo-06` | `"gravity":"bottom"` | unchanged |
| `photo-07` | `"gravity":"bottom"` | unchanged |
| `photo-08` | `"gravity":"top"` | unchanged |
| `photo-05-quiz` | `"gravity":"top"` | unchanged |
| `photo-05-loop` | `"gravity":"top"` | unchanged |
| `handle-logo` | `"gravity":"center"` | unchanged |

**Why deletion, not translation.** The graveyard excerpt states that `gravity`
"meant *which part of a source survives a crop* — a quantity that does not
exist once the declared rect is authoritative at render: the rect's `x`, `y`,
`origin` and the static frame-space `clip` already determine it, with no
freedom left to spend." In other words, even where `gravity` appears alongside
a full placement, it carried no independent information the renderer could
act on — the rect and clip already pin down which pixels survive. Deleting a
field that has no effect cannot change the video the file describes. The
excerpt also notes that when `gravity` was retired, "its eight occurrences
left the fixture in the same change" — i.e. plain removal was the sanctioned
repair for exactly this situation.

I verified after the edit: the file is valid JSON, keeps one element per line,
elements remain sorted by `start` within each track, and key order in every
element (apart from the deleted key) is stable.

## What I was unsure about

This is the honest core of the task, because there *was* a second legal-looking
repair for two of the elements.

1. **`photo-06` and `photo-07`: delete vs. translate `gravity:"bottom"` into a
   negative `y`.** Both elements declare the image at `x:0, y:0,
   origin:"top-left", width:1080, height:1912` with `clip:[0,0,1080,1300]`.
   As written, the top 1300 px of the declared rect is visible. If `gravity`
   had ever been an *executing* crop-anchor (as in ImageMagick/ffmpeg usage),
   `"bottom"` would have meant "show the bottom of the image", and a
   semantics-preserving repair would have been to rewrite `y` to
   `1300 − 1912 = −612` so the bottom 1300 px of the rect falls inside the
   clip. That would produce a visibly different video from plain deletion.

   I chose deletion, for three reasons:
   - The graveyard is explicit that under this format `gravity` had "no
     freedom left to spend" — the declared rect was already authoritative, so
     the field never moved anything. The video the file describes with the
     field present is the one with `y:0` visible at the top; deleting
     preserves that.
   - The `gravity` values in this file don't behave like load-bearing data:
     five of the seven photo elements say `"top"` while sitting at `y:0` with
     a top-anchored clip (redundant with the rect even under the old folk
     semantics), and `handle-logo` says `"center"` with a clip identical to
     its rect (nothing to anchor at all). This is the copy-a-neighbour pattern
     the graveyard describes, not deliberate framing.
   - Translating would have changed rendered output relative to what the
     declared rects say today, which the task forbids ("without changing the
     video it describes").

   Residual doubt: if the *author's mental picture* for photos 06 and 07 was
   the bottom of those images, my repair preserves the file's meaning but not
   that unexpressed wish. Only the source PNGs could settle whether top or
   bottom framing looks right, and I don't have them (see below).

2. **`handle-logo` `gravity:"center"`.** No real ambiguity: its `clip`
   `[478,96,68,68]` coincides exactly with its declared rect, so every pixel
   of the rect is visible and "center" anchoring is vacuous under any
   semantics. Deletion is a no-op. Noted only for completeness.

3. **Scale keyframes ending after the element ends** (e.g. `photo-05-intro`
   ends at 3018 but its second keyframe is at `t:15000`; `photo-05-loop` ends
   at 65216 with a keyframe at `t:79016`). The validator did not flag these,
   times are absolute milliseconds, and a keyframe beyond the element's end
   is a coherent way to say "traverse only the early part of this ramp" —
   here it gives every photo the same 1.0→1.08-over-15s Ken Burns rate
   regardless of its duration. Treated as intentional; not touched. Flagged
   because a stricter format might consider them errors; repairing them was
   neither requested nor safe.

4. **`fit:"cover"` with height 1912 on a 1920-tall frame.** `fit` is a
   derivation claim that never executes, so even if 1912 looks like an odd
   cover result, correcting it is not a validation repair and could only
   change the video. Left alone.

## What I wanted and did not have

- **The source images** (`images/05.png` … `images/08.png`,
  `brand/logo-en.png`), or at least their pixel dimensions — to check whether
  `gravity:"bottom"` on photos 06/07 corresponded to visibly different
  intended framing. This is the one place deletion-vs-translation actually
  diverges visually.
- **The old (pre-retirement) renderer's behaviour for `gravity`** — whether it
  ever executed, and with what precedence against `x`/`y`/`origin`. The
  graveyard says it did not ("no freedom left to spend"), and I relied on
  that; I could not verify it independently without consulting the ADR, which
  the rules forbid.
- **The `montagent validate` binary**, to re-run validation on the repaired
  file. I verified JSON validity, the absence of `gravity`, and the writing
  conventions by hand instead.

## Confidence

**High (~95%) that the repaired file describes the same video the original
file described**, and validates: the only reported errors were the eight
`gravity` occurrences, `gravity` is documented as carrying no renderable
information, and every other byte of the file is unchanged.

**Moderate-to-high (~85%) that this is the video the author *meant*.** The
gap is entirely the `gravity:"bottom"` question on `photo-06` and `photo-07`:
if the author believed `gravity` executed and wanted the bottom of those two
images on screen, that wish was never expressed in the authoritative fields
and my repair does not resurrect it. I could not check this without the
source images. Everything else — timings, text, audio, layout — I did not
alter and had no reason to doubt.
