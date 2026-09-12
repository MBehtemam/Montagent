# Halloween decorating render — what I found and what I changed

## Headline

The 16 errors are 8 image elements × 2 keys: `fit` and `clip`. montaget 0.9.2
does not know them. Deleting them makes the render pass — but `clip` on the
photos is **load-bearing layout**, not decoration, so deleting it alone would
have shipped a video that renders green and looks wrong. I removed the keys and
rebuilt the one effect that mattered out of primitives this build box accepts.

**I could not run the render.** `montaget` is not on the PATH in the environment
I was given, and `images/`, `audio/`, `fonts/` and `brand/` are not present
either — the directory holds only `project.json` and `render-output.txt`. So
everything below is reasoned from the project file, and nobody has yet seen a
frame. Read the confidence section before publishing.

## What I did

**1. Removed `"fit":"cover"` and `"clip":[…]` from all 8 image elements.**
Required: the validator refuses on unknown keys, so nothing renders while they
are there. There is no version of "get the render working" on a box with no
network that keeps them.

**2. Added a `photo-crop` track (layer 15): one `rect`, `#FBF3E3`, x0 y1300
1080×620, spanning the whole 65216 ms.**
This restores what the deleted `clip` was doing. Evidence that it mattered:

- Every photo element is `0,0 → 1080×1912` in a 1080×1920 frame, and every one
  carried the identical `clip:[0,0,1080,1300]`. So the photo was drawn only in
  the top 1300 px.
- Every card and every caption in the file starts at y ≥ 1324. The lowest is
  `word-05`/`word-quiz` at 1324..1422; the cards sit at 1453..1622. The clip
  line at 1300 is exactly the top edge of the text zone. That is not a
  coincidence — it is the design: photo on top, cream lower third, navy
  (`#245C8C`) text on cream.
- Without the clip the photo fills to 1912 and every caption and sentence card
  lands on top of the photograph instead of on cream. Navy text on an
  un-dimmed photo, for the full runtime.

The rect is an exact substitute rather than an approximation, for three
reasons: layers 11–19 are empty, so nothing is hidden by painting over that
band; `background` is `#FBF3E3`, the same cream the chip panels use, so the
covered region is pixel-identical to bare background; and 1912..1920 was
already background anyway. It also correctly clips the 1.0→1.08 Ken Burns zoom,
which grows the photo downward.

It uses only keys already proven to work in this file (`rect` with
`x/y/origin/width/height/fill`), so it cannot introduce a new unknown-key error.
I reused the existing `group:"header"` — the only group already spanning the
full timeline — rather than invent a group name, purely to avoid any new value
in a validator I cannot test against. Rename it if the author prefers.

The original is preserved at `project.json.orig`. A diff confirms the file is
otherwise byte-for-byte unchanged: same tracks, same 60 elements, same timings.

## What I considered and rejected

**Just delete `fit` and `clip` and ship.** Rejected. It renders, which is the
trap — 16 errors become 0 and the mp4 appears. But it silently changes the
composition for 100% of the runtime. "The render works" is not the job; the job
is the video we published before.

**Change the image `height` from 1912 to 1300.** The obvious way to crop
without a crop key. Rejected: with `fit` gone I do not know the default scaling
rule, and if it stretches source to the element box this squashes every photo
vertically. It also changes the box the `scale` keyframes act on. It trades a
known-good result for an unknown one.

**Reach for `mask`.** `mask:"circle"` on the logo did not error, so masks exist
in 0.9.2 — but the only value I have evidence for is `"circle"`. Guessing a
rect mask syntax against a validator I cannot run is a coin flip, and a wrong
guess is another refused render.

**Upgrade montaget.** The shape of this bug is a stale build box: the file was
almost certainly authored against a newer montaget where `fit` and `clip` are
real keys. `fit`+`clip` appear together on all 8 images, and on the logo `clip`
is exactly the element box — a no-op default — which reads like keys emitted
mechanically by a newer writer. Upgrading is the real fix. No network, so not
today. Worth filing.

**Wait for the author.** Legitimate, and I would choose it over shipping a
guess. I did not need to: the crop reconstruction is derived from the file's own
geometry, not guessed.

## What I would tell the team

> Overnight render died on 16 validation errors: the project file uses `fit`
> and `clip` on image elements and the build box is on montaget 0.9.2, which
> doesn't know those keys. Looks like the file was written against a newer
> montaget than the box has — nobody's updated the box in months, and we have no
> network today. I stripped both keys and rebuilt the part that mattered: the
> `clip` was holding the photos out of the bottom third so the captions sit on
> cream, so I put a cream rect on layer 15 over y1300–1920, which is
> pixel-equivalent. Nothing else in the file changed; original saved as
> `project.json.orig`. **Caveat: I couldn't actually run the render or see the
> assets from where I was working, so this hasn't been eyeballed.** Someone
> please scrub the output — especially the first second and the 42–54 s
> "string of lights" section — before it goes out. Real fix is upgrading
> montaget on the build box; filing that now.

## Confidence

**That the crop reconstruction matches what we published: high.** The 1300 line
against the 1324 caption top is decisive, layers 11–19 are empty, and the fill
is the declared background colour. I would defend this one.

**That the overall video matches: medium, and gated on one thing I could not
check — `fit:"cover"`.** If the source PNGs are already 1080×1912 (very likely:
the element boxes match the frame width exactly and are 8 px off frame height,
which is what pre-sized assets look like), `cover` was a no-op and the output is
identical. If they are not, then whatever 0.9.2 does by default — stretch,
contain, or natural size — now applies instead of a centre-crop-to-fill, and
the framing of every photo shifts. I have no images on disk to measure and no
renderer to test with, so I cannot close this. It is the single thing to look
at first in the output.

**Could not check at all:**
- Whether the file renders. montaget is not installed where I worked. Errors
  beyond the 16 reported may exist further down the pipeline — the validator
  stops at validation, so missing assets or font problems would not have
  surfaced in this log yet.
- Source image and logo dimensions (see `fit` above).
- The logo. Its `clip:[478,96,68,68]` equalled its own element box, so removing
  it is a no-op; `mask:"circle"` is untouched. But its `fit:"cover"` carried the
  same uncertainty — if `brand/logo-en.png` is not square it may now sit
  differently inside the 68 px circle.
- Audio. Untouched, and nothing in the log complained about it, but no
  timing/levels check was possible.

One oddity I noticed and deliberately left alone: every photo's `scale`
keyframe ramps over 15000 ms regardless of how long that photo is on screen
(e.g. `photo-05-loop` is 1200 ms long with a keyframe at t=79016). That is a
consistent constant-rate Ken Burns pattern across all 7 photos, so I read it as
intentional authoring, not damage. It is unrelated to this failure.
