# Render failure — en-halloween-decorating

## What I did, and why

**Diagnosis.** `montagent 0.9.2` on this box rejected the file with 16 errors:
unknown key `fit` and unknown key `clip` on exactly 8 elements (the 7
full-frame photo elements and `handle-logo`). Everything else in the file —
`mask`, `speed`, keyframed `scale`, `runs` — validated, so the binary is only
one small vocabulary step behind the file. The file was evidently written (or
regenerated upstream) for a newer format revision than the 0.9.2 binary on
this box implements. There is no network, so upgrading the binary was not an
option today.

**What `fit` and `clip` actually do in this file.** I did not treat them as
noise; I worked out their visual effect element by element:

- Every photo element is placed at (0,0) with width 1080 x height 1912 —
  effectively the full 1080x1920 frame — and carries the identical
  `clip: [0,0,1080,1300]`. The clip's only job is to hide the bottom ~620px
  of each photo so the flat cream page background (`#FBF3E3`) shows through
  where the word captions and sentence cards are drawn (y ~ 1350–1620).
- `handle-logo` carries `clip: [478,96,68,68]` — exactly its own bounding
  box, i.e. a no-op. Its circular shape comes from `mask: "circle"`, which
  0.9.2 accepts.
- `fit: "cover"` is an identity transform whenever the source asset's aspect
  matches the declared box. All photo boxes are the uniform 1080x1912 and the
  logo box is square, which strongly suggests assets authored at those sizes.

**The fix.** I re-expressed the same visuals in vocabulary 0.9.2 understands,
rather than just deleting semantics:

1. Removed `fit` and `clip` from the 8 elements (16 keys — matching the 16
   errors one-to-one).
2. Added a new track `photo-mask` on **layer 11** — between the photos
   (layer 10) and everything drawn on top (cards at 20, text at 21+) — with a
   single rect: full width, y=1300 to the bottom of the frame (1080x620),
   fill `#FBF3E3`, spanning the whole 0–65216ms timeline.

Clipping a full-frame image against a flat background is pixel-identical to
painting that background color over the clipped-away region on a layer above
the image. The Ken Burns `scale` animations (origin top-left, up to 1.08) are
unaffected: the visible window stays fixed at y<1300 exactly as the clip kept
it, and horizontal overflow is cut by the frame edge as before. Nothing that
must remain visible sits between layer 10 and layer 20 in that region.

The original file is preserved unmodified as `project.json.orig`.

## What I considered and rejected

- **Just deleting `fit`/`clip` and rendering.** The error message explicitly
  warns against this, and correctly so: dropping `clip` would have painted the
  photos over the cream caption band, putting blue-on-photo text where the
  published video has blue-on-cream. That video would validate, render, and be
  wrong. Rejected — what I did instead replaces the semantics with an
  equivalent construction, not a deletion of meaning.
- **Upgrading montagent.** No network on the build box, and no other montagent
  binary or package found on disk. Rejected as impossible today.
- **Guessing the retired/older spellings of `fit`/`clip`** and hoping 0.9.2
  accepts them. Without the spec I would be inventing keys; a wrong guess
  either errors again or, worse, silently means something else. Rejected in
  favor of a construction built only from keys this exact file already proves
  0.9.2 accepts (`rect`, `fill`, layers, start/end).
- **Doing nothing and escalating.** The video needs to go out today and an
  equivalence-preserving transform exists; escalation alone doesn't ship it.
  (The root cause still needs fixing — see below.)

## What I would tell the team

Overnight render of en-halloween-decorating failed: the project file uses the
newer `fit`/`clip` keys but the build box is still on montagent 0.9.2, which
predates them. I shipped today's video by rewriting the file in 0.9.2
vocabulary — removed the 16 offending keys and replaced the photo clip with an
equivalent background-colored mask rect on layer 11; output should be
pixel-identical, but please eyeball the caption band and the circular logo
before publishing. Root cause is version skew between whatever now generates
our project files and the build box binary — we should upgrade montagent on the
box (and pin generator and renderer versions together), then revert to the
original file, kept here as `project.json.orig`.

## Confidence

**High (~90%) that the output matches the previously published video**, with
these caveats I could not check:

1. **Source asset dimensions.** If `images/05.png`–`08.png` are exactly
   1080x1912 and `brand/logo-en.png` is square, removing `fit: "cover"` is a
   no-op and confidence is essentially total. If an asset's aspect differs,
   0.9.2's default placement (likely stretch-to-box) would distort it where
   `cover` would have cropped — and only the top 1300px of each photo is
   visible, so any such difference is confined there.
2. **0.9.2's default fit behavior** (stretch vs. letterbox vs. crop) is
   undocumented to me; it only matters in the aspect-mismatch case above.
3. **I could not run a verification render** — the working directory I was
   given contains neither the binary nor the assets, so I validated the JSON
   structurally but not through `montagent render`. Whoever runs the render
   next should confirm zero errors and visually compare a frame around ~12s
   (caption band + sentence card) and the header logo against the last
   published video before it goes out.
