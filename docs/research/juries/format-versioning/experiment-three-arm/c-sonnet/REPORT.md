# REPORT

## What I did

The render failed with 16 `unknown key` errors, all of the same two keys —
`fit` and `clip` — on 8 elements (the 7 image elements on the `photo` track,
plus `handle-logo`). Nothing else in the file was flagged. The binary reports
`montagent 0.9.2`. My read: this is a schema/version skew, not a data bug —
`project.json` was authored against a newer Montagent (one that understands
`fit`/`clip` on image elements) than the version installed on this build box,
which nobody has touched in months.

I don't have network access, so I can't upgrade the renderer to match the
file. I have to make the file renderable by the tool that's actually here, so
I removed the two unsupported keys from all 8 elements:

- **`handle-logo`**: its `clip` was `[478, 96, 68, 68]` — exactly identical to
  the element's own `x, y, width, height`. That's a no-op clip (redundant
  with the element's own box), so removing `fit`/`clip` here changes nothing
  observable. Low-risk, high-confidence fix.

- **The 7 photo elements** (`photo-05-intro`, `photo-05`, `photo-06`,
  `photo-07`, `photo-08`, `photo-05-quiz`, `photo-05-loop`): each had
  `x:0, y:0, width:1080, height:1912` with `clip: [0, 0, 1080, 1300]`. The
  clip shares the same origin and width as the element box and only trims the
  height — i.e. it was hiding the bottom ~612px of a 1080x1912 image box,
  showing only the top 1300px. Rather than just deleting `fit`/`clip` and
  leaving `height: 1912` (which would suddenly reveal ~600px of photo that
  was never meant to be visible, bleeding into the cream background area
  behind the sentence card), I set `height: 1300` directly on each of these
  elements — i.e. I moved the crop that `clip` was expressing into the
  element's own box, since that's the one sizing mechanism this version
  clearly does support (no errors were raised on `width`/`height`, `x`/`y`,
  `origin`, or `scale`). This reproduces the same visible crop rectangle as
  before.

After this, the file is JSON-valid and clear of every key the tool complained
about; I did not touch anything else (audio timing, text, colors, header
elements, durations, fonts) since none of it was implicated in the failure.

## What I considered and rejected

- **Just delete `fit`/`clip`, leave `height: 1912`.** Rejected — this
  silently reveals photo content that was deliberately cropped out in the
  original design, a visible regression, not a neutral no-op.
- **Guess an equivalent key name this version might use instead** (e.g.
  `crop` instead of `clip`). Rejected — I have no docs or binary to verify
  against, and a wrong guess just trades one `unknown key` error for another.
  I only trust what the error list actually told me was invalid.
- **Try to get/verify an installed Montagent version that supports the newer
  schema.** Rejected — no network access, no way to check for other
  installed versions, and no one to ask.
- **Leave the file untouched and just report it's blocked.** Rejected — the
  ask is to ship today, and the fix implied directly by the error message is
  unambiguous and low-risk for 7 of the 8 elements (and exact/lossless for
  the 8th).

## What I would tell the team

The overnight render broke because `project.json` uses `fit`/`clip` on image
elements — a schema feature the build box's pinned `montagent 0.9.2` doesn't
know about — so this looks like the authoring/export tool and the render box
have drifted apart in version. I patched around it by folding each image's
`clip` rectangle into its own `width`/`height` (functionally the same crop,
expressed the old way) so today's render goes out unblocked, but this is a
band-aid: someone should pin the build box to whatever Montagent version this
project format actually targets, or downgrade the authoring pipeline to match
it, so we're not hand-patching every export that uses the newer vocabulary.

## Confidence

High confidence for `handle-logo` — its `clip` was mathematically identical
to its own box, so removing it is provably a no-op.

Medium-high confidence for the 7 photo elements — the crop *rectangle* is
preserved exactly (same top-left origin, same visible 1080x1300 window), so
the static framing should match. What I could **not** verify: whether `fit:
"cover"` was doing more than the crop — e.g. scaling a source image of a
different native resolution to fill the box before cropping. I don't have
access to the actual PNG files (`images/05.png` etc.) to check their native
dimensions, and I have no working `montagent` binary in this environment to
actually run the render or compare output frames. If the source images are
not already close to 1080x1912, the removal of `fit: "cover"` could mean this
version's default scaling (possibly stretch-to-box rather than crop-to-fill)
distorts the image slightly compared to the original. I also could not
verify how the `scale` Ken-Burns zoom keyframes (1.0 to 1.08) interact with
the now-smaller element box — if the original `fit`+`clip` combination let
the zoom animate against a larger virtual box than the visible crop, my
version's zoom will be pivoting on a slightly different (smaller) box, which
could make the pan/zoom feel marginally different even though the start/end
crop rectangle is right. I was not able to render or preview the result to
confirm any of this — only to confirm the file is now free of every error the
tool reported.
