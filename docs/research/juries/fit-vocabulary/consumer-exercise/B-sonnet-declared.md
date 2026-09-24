# Worklog — B-sonnet-declared

Source of truth used: `SPEC-declared.md` only. Did not read `docs/adr/`. Read the existing
fixture `fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json` for context
(read-only, not edited). Used `python3` to verify the integer arithmetic for every task.

---

## Task 1 — New photo (images/09.png, 1536x2720)

**Wrote:**
```json
{"id":"photo-09","type":"image","start":30000,"end":36000,"source":"images/09.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","clip":[0,0,1080,1300]}
```

**Arithmetic:** box = clip w/h = 1080x1300. `bw*sh = 1080*2720 = 2,937,600` vs
`bh*sw = 1300*1536 = 1,996,800` → `bw*sh >= bh*sw` → width drives.
width = bw = 1080 (verbatim). height = `(sh*bw)//sw = (2720*1080)//1536 = 2,937,600//1536
= 1912` (floor of 1912.5). Check cover: width>=bw (1080>=1080 ✓), height>=bh (1912>=1300 ✓).

**First reached for:** I first reached for `gravity:"top"` because every existing image
element in the fixture carries that field alongside `fit:"cover"`. I checked the spec before
writing it and found the "Not fields" section explicitly says there is no `gravity` field on
images — so I dropped it. This is the single most useful finding of the exercise: the fixture
file and the published spec disagree on a field that appears on literally every image element
in production.

**Hesitated:** Confirming the direction of the cover inequality (`bw*sh >= bh*sw` meaning
*width* drives, not height) — it reads backwards from intuition at first glance, so I verified
it against the existing photo-05..08 elements (all of which resolve to width=1080 driving)
before trusting it on new data.

**Wanted an error message saying:** N/A for this task — it validated cleanly by construction.

---

## Task 2 — Re-exported source (images/06.png: 1536x2720 → 1536x2200)

**Wrote (changed fields only):**
```json
{"id":"photo-06", ..., "width":1080,"height":1546,"fit":"cover","clip":[0,0,1080,1300]}
```
(x, y, origin, clip, source all unchanged.)

**Arithmetic:** new source 1536x2200 into box 1080x1300. `bw*sh = 1080*2200 = 2,376,000` vs
`bh*sw = 1300*1536 = 1,996,800` → width still drives. width = 1080 (unchanged, verbatim).
height = `(2200*1080)//1536 = 2,376,000//1536 = 1546` (floor of 1546.875), down from 1912.
Check cover: height (1546) >= bh (1300) ✓ — aperture still fully covered.

**First reached for:** I first reached for just recomputing `height` and leaving `width`
alone, which turned out to be exactly right since the driving axis doesn't change here — but
I only knew that *after* checking which axis drives, not before. My first instinct was to
guess the new height by scaling 1912 by the ratio 2200/2720, which gives ~1546.47 → I'd have
rounded to 1546 by luck, but the spec's actual rule (recompute from box, floor division) is
not "rescale the old value," so I discarded that shortcut and used the real formula.

**Hesitated:** Whether `clip` needed to change too — re-read the spec's "aperture coverage"
error rule to confirm `clip` is independent of the source and stays as authored; only
`width`/`height` (derived from `fit`) needed fixing.

**Wanted an error message saying:** The spec says the error is "the declared rect disagrees
with the rule named by fit" — a maximally useful validator message here would state the
box dims and the two sides of the driving-axis inequality, e.g. `cover: width 1080 must be
>=1080 (ok), height 1912 must be >=1300 but source only yields 1546 — got 1912, expected
1546`. That would let me skip re-deriving the arithmetic by hand.

---

## Task 3 — Badge, must not be cropped (brand/badge.png, 1200x400, aperture [840,1700,200,160])

**Wrote:**
```json
{"id":"badge","type":"image","start":0,"end":65216,"source":"brand/badge.png",
 "x":840,"y":1700,"origin":"top-left","width":200,"height":66,"fit":"contain",
 "clip":[840,1700,200,160]}
```

**Arithmetic:** box = clip w/h = 200x160. `bw*sh = 200*400 = 80,000` vs
`bh*sw = 160*1200 = 192,000` → `bw*sh <= bh*sw` → width drives (contain rule).
width = bw = 200 (verbatim). height = `(400*200)//1200 = 80,000//1200 = 66` (floor of
66.67). Check contain: width<=bw (200<=200 ✓), height<=bh (66<=160 ✓) — fully inside, nothing
cut off.

**First reached for:** I first reached for "center the badge in the leftover vertical
space" — i.e., wanted something like `y: 1700 + (160-66)/2` to vertically center it in the
gutter. The spec's "Not fields" section rules that out explicitly ("no gravity field, no
align on images"), and there's no field to express relative placement inside the box — so
x/y are just the clip's own origin, top-left-anchored, and the leftover 94px of vertical
slack is simply left unfilled at the bottom. I did not find any field for this and stopped
reaching for one once I re-read "Not fields."

**Hesitated:** Whether `start`/`end` (65216 = full-file duration, guessed by pattern-matching
the header/chip elements which run the whole video) was even in scope — task didn't specify
timing, so I used the full-project duration as the least-surprising default, matching how the
`chip-panel`/`flag-field`/etc. elements in the fixture span 0 to full duration.

**Wanted an error message saying:** None needed — validated cleanly. But I'd have liked the
spec itself, not just the validator, to state directly "there is no anchor/align field for
positioning inside a box under contain" — I had to infer this negatively from the "Not
fields" list rather than find it stated positively near the `contain` explanation.

---

## Task 4 — Deliberate distortion (images/07.png, 1536x2720, squash to 1080x1600)

**Wrote (changed fields only, scale animation preserved verbatim):**
```json
{"id":"photo-07","group":"item-07","type":"image","start":30603,"end":42763,
 "source":"images/07.png","x":0,"y":0,"origin":"top-left",
 "width":1080,"height":1600,"fit":"declared","clip":[0,0,1080,1300],
 "scale":[{"t":30603,"v":[1.0,1.0]},{"t":45603,"v":[1.08,1.08],"ease":"linear"}]}
```

**Arithmetic:** None — `declared` is explicitly "no rule; the integers are yours." 1080x1600
is neither `cover` (1600 < box's... actually 1600 >= bh 1300 but width 1080=bw exactly, so it
*could* pass cover's inequality-only check) nor a clean `contain`. Since the brief is
explicitly "the wrong aspect ratio, on purpose," `fit` must be `declared` regardless of
whether the numbers happen to also satisfy `cover`'s inequalities — `declared` is the value
that means "I intend this, don't check it against a rule."

**First reached for:** I first reached for `fit:"cover"` on autopilot, since every element I'd
seen so far in this exercise and in the fixture used `cover`. Re-reading the `fit` table
stopped me: `cover`'s whole point is a rule-derived rect, and the task explicitly wants the
*wrong* aspect ratio chosen on purpose — using `cover` here would either fail validation (if
the numbers don't satisfy the inequality) or, worse, silently look "validated" while implying
a derivation that isn't true. `declared` is the value built for exactly this case.

**Hesitated:** Whether the existing `scale` keyframe array needed any change since the drawn
rect's aspect ratio changed. The spec doesn't mention any interaction between `scale`
keyframes and `width`/`height`/`fit` — `scale` isn't documented in this spec file at all — so
per the task instruction ("must stay readable and untouched") I left it byte-for-byte as in
the fixture and made no changes to it.

**Wanted an error message saying:** If I *had* mistakenly kept `fit:"cover"` here, I'd want:
"fit:cover requires width>=box_w and height>=box_h AND the driving axis must equal the box
dimension exactly — width 1080 = box 1080 (ok) but this pins height on the box's own ratio;
1600 does not match the source-derived 1912. If this rect is intentionally not derived from
the source, use fit:\"declared\" instead." That last sentence is the one that would have
saved me the double-take.

---

## Task 5 — Rotated phone photo (images/10.jpg, stored 3024x4032, EXIF orientation 6)

**Wrote:**
```json
{"id":"photo-10","type":"image","start":0,"end":0,"source":"images/10.jpg",
 "x":0,"y":0,"origin":"top-left","width":1733,"height":1300,"fit":"cover","clip":[0,0,1080,1300]}
```
(start/end left as placeholders — not specified by the task.)

**Answer: width=1733, height=1300.**

**Arithmetic:** Spec: "Source dimensions are the image's orientation-applied pixel
dimensions — if a JPEG carries an EXIF orientation flag that transposes the image, use the
transposed dimensions." Orientation 6 = rotate 90° CW = a transposing orientation, so
orientation-applied dims = 4032 wide x 3024 tall (swapped from the stored 3024x4032).
Box = clip = 1080x1300. `bw*sh = 1080*3024 = 3,265,920` vs `bh*sw = 1300*4032 = 5,241,600` →
`bw*sh < bh*sw` → height drives. height = bh = 1300 (verbatim). width =
`(sw*bh)//sh = (4032*1300)//3024 = 5,241,600//3024 = 1733` (floor of 1733.33...). Check
cover: width (1733) >= bw (1080) ✓, height (1300) >= bh (1300) ✓.

**First reached for:** I first reached for the *stored* dimensions (3024x4032) straight into
the cover formula, before rereading the "Source dimensions are... orientation-applied" line.
Using the stored (untransposed) 3024x4032 instead of the transposed 4032x3024 would have
given a completely different (and wrong-for-the-display-orientation) answer — width would
drive instead of height, producing roughly width=1080, height=1440 — a plausible-looking but
incorrect result that validate would presumably accept as internally consistent (since fit's
error check only compares the declared rect against source+box, and if I'd fed it the wrong
source dims consistently, it'd validate against my own mistake). This is the one place in the
exercise where getting it wrong wouldn't have been caught by validate at all — it would only
show up as a rotated/cropped-wrong image at render time. That's the second most valuable
finding of the exercise.

**Hesitated:** Confirming which of the two numbers is "width" after transposition — orientation
6 rotates 90°, so the display width is the stored *height* value (4032) and vice versa. I
double-checked this by reasoning it through rather than trusting my first instinct.

**Wanted an error message saying:** There's no way for a validator to catch a wrong-EXIF-
assumption automatically (garbage in, garbage out) — but the spec itself could be clearer by
giving a worked example (an orientation-6 case) next to the cover/contain arithmetic, the way
it gives a worked JSON example for the plain element up top. I had to apply the
"orientation-applied" rule myself with no example to check against.

---

## Task 6 — Crop to top third of a tall photo (images/11.png, 1080x3000, aperture [0,0,1080,1000])

**Wrote:**
```json
{"id":"photo-11","type":"image","start":0,"end":0,"source":"images/11.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":3000,"fit":"cover","clip":[0,0,1080,1000]}
```

**Arithmetic:** First checked `declared` (since "top third" sounded like a manual crop, not
a derivable fit rule), planning width=1080, height=3000 (native size) with clip doing the
cropping. Then re-read the `fit` table and ran the `cover` inequality anyway: box = clip =
1080x1000, source = 1080x3000. `bw*sh = 1080*3000 = 3,240,000` vs `bh*sw = 1000*1080 =
1,080,000` → width drives. width = bw = 1080 (verbatim). height =
`(3000*1080)//1080 = 3000` — i.e. the *undistorted native size*, because the source width
already equals the box width exactly. Check cover: width (1080)>=bw(1080) ✓,
height(3000)>=bh(1000) ✓. So `cover` legitimately produces the exact "native size, let clip
do the cropping" rect here — no need for `declared`. Since `clip` is `[0,0,1080,1000]` and
the element itself is drawn from y=0, only the top 1000px of the 3000px-tall drawn rect falls
inside `clip` — i.e. exactly the top third.

**First reached for:** I first reached for `fit:"declared"`, reasoning "a manual top-third
crop isn't really a cover/contain derivation, it's just clip doing the work." Running the
actual cover inequality showed `cover` derives the identical numbers here (because the
source's width already matches the box's width 1:1, the "slack axis" formula degenerates to
the unscaled source height). So `declared` would have been defensible but not the tightest
answer — `cover` is provably correct here and lets `validate` actually check the rect instead
of taking it on faith. I switched to `cover` after doing the arithmetic and seeing it matched
exactly.

**Hesitated:** Whether `clip`'s job (crop-only, "does not move or scale with the element") was
enough on its own to produce "top third visible" without needing `y` to shift, or whether I
needed to offset `y` negatively to slide the image up first. Re-read the `clip` section:
clip is a rect in *frame space*, independent of the element's own position — so with the
element drawn at y=0 with height 3000 and clip fixed at y=0..1000, the top 1000px (= top
third of 3000) is what's visible without touching x/y at all.

**Wanted an error message saying:** Nothing wrong here, but I would have liked the spec to
state explicitly, near the `fit` table, that `cover`/`contain` degenerate to "native size" or
"box size" when a source dimension exactly equals a box dimension on the driving axis — I had
to discover that by running the arithmetic rather than being told it's a named/expected case.

---

## Verdict

**Usable as published:** Mostly yes — the arithmetic is fully specified, closed, and
integer-only, so two implementers working only from this spec should get bit-identical
results, which is the main thing that matters for a validated format.

**Single worst thing:** The spec and the actual fixture file disagree on vocabulary — every
existing image element in `en-halloween-decorating.montagent.json` carries a `"gravity":"top"`
field that this spec explicitly lists under "Not fields" as not existing. If I'd used the
fixture as my primary reference instead of cross-checking every field against the spec text,
I would have propagated a field that (per this document) isn't real. A published spec that
contradicts its own reference project is worse than no reference project.

**One concrete change:** Add a worked example for a non-transposed vs. transposed EXIF case
directly under the "Source dimensions are... orientation-applied" sentence (Task 5's whole
category of risk). It's the one rule in the document whose violation `validate` cannot catch
— get the orientation wrong and you get a self-consistent, fully "valid" element that's
simply wrong at render time — so it's the rule most worth over-specifying with an example,
not stating once in prose.
