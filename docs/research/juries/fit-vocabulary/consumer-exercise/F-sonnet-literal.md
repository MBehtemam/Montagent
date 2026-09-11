# Worklog — F-sonnet-literal

Source consulted: `SPEC-literal.md` only. Reference (read, not edited): `fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`.
No files under `docs/adr/` were read.

**Note on the reference project file up front:** every existing image element in it carries a
`"gravity":"top"` field. The spec's "Not fields" section says explicitly: *"There is no `gravity`
field... on images."* I did not add `gravity` to any of my answers, because the spec is the
governing document here — but this is a real discrepancy between shipped data and published docs,
flagged in the Verdict below.

---

## Task 1 — New photo (images/09.png, 1536x2720)

**Wrote:**
```json
{"id":"photo-09","type":"image","group":"item-09","start":30000,"end":36000,"source":"images/09.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover","clip":[0,0,1080,1300]}
```

**Arithmetic:** box = clip w,h = 1080x1300. sw=1536, sh=2720.
`bw*sh = 1080*2720 = 2,937,600`; `bh*sw = 1300*1536 = 1,996,800`. `bw*sh >= bh*sw` → width drives.
width = bw = 1080 (verbatim). slack (height) = `(sh*bw)//sw = (2720*1080)//1536 = 2,937,600//1536 = 1912` (floor).
→ `width:1080, height:1912`.

**First reached for:** `fit` itself — before opening the spec I assumed I'd need to pick between
something like "fill"/"fit"/"crop" as a value; had to check the closed set (`cover`/`contain`/`literal`).
Also briefly reached for `gravity:"top"` by pattern-matching the existing `photo-06` element, before
remembering the spec's "Not fields" list rules it out.

**Hesitated:** none on the arithmetic itself — the worked "same aperture as the other photos" framing
made `cover` the obvious rule, and it happened to reproduce the exact 1080x1912 already used elsewhere.

**Wanted an error message saying:** N/A — no error here.

---

## Task 2 — Re-exported source (images/06.png, now 1536x2200)

**Wrote (changed fields only, rest of `photo-06` unchanged):**
```json
"width":1080,"height":1546,"fit":"cover","clip":[0,0,1080,1300]
```

**Arithmetic:** box = 1080x1300 (clip unchanged). sw=1536, sh=2200.
`bw*sh = 1080*2200 = 2,376,000`; `bh*sw = 1300*1536 = 1,996,800`. `bw*sh >= bh*sw` → width drives,
width = 1080. slack (height) = `(2200*1080)//1536 = 2,376,000//1536 = 1546` (floor, since 1546.875→1546).
Check coverage requirement for `cover`: width(1080) >= box_w(1080) ✓, height(1546) >= box_h(1300) ✓.

**First reached for:** I first reached for just recomputing `height` alone and leaving `width:1080`
untouched (since the box width didn't change and 1536/2200 is still portrait) — that happened to be
correct here, but only because width was already the driving axis both before and after the re-export.
I did not assume that in general — I redid the `bw*sh` vs `bh*sw` comparison from scratch to confirm
the driving axis hadn't flipped.

**Hesitated:** re-read the "error" bullet under "What `validate` reports" to confirm which condition
was tripping — the driving axis must equal the box dimension *exactly*, so I checked width==1080 was
still satisfied post-edit as well as re-deriving height.

**Wanted an error message saying:** something like `"cover: height (1912) does not match floor(sh*bw/sw) = 1546 for current source dimensions 1536x2200"` — i.e. name the *expected* value, not just "disagrees with the rule," so I wouldn't have had to redo the whole derivation to find the target number.

---

## Task 3 — Badge that must not be cropped (brand/badge.png, 1200x400, aperture [840,1700,200,160])

**Wrote:**
```json
{"id":"badge","type":"image","start":30000,"end":36000,"source":"brand/badge.png",
 "x":840,"y":1700,"origin":"top-left","width":200,"height":66,"fit":"contain","clip":[840,1700,200,160]}
```

**Arithmetic:** box = clip w,h = 200x160. sw=1200, sh=400.
`bw*sh = 200*400 = 80,000`; `bh*sw = 160*1200 = 192,000`. `contain`: width drives when `bw*sh <= bh*sw`
→ 80,000 <= 192,000 → true, width drives. width = bw = 200 (verbatim).
slack (height) = `(sh*bw)//sw = (400*200)//1200 = 80,000//1200 = 66` (floor, 66.67→66).
Check `contain` coverage: width(200) <= box_w(200) ✓, height(66) <= box_h(160) ✓ — clip contains the drawn rect.

**First reached for:** I first reached for centering the badge vertically inside the aperture (some kind
of `align`/`gravity`/`anchor: center` field), since 66 < 160 leaves 94px of slack. The spec has no such
field ("no `gravity` field, no `align` on images"), so I placed `x,y` at the clip's own origin
(`840,1700`) with `origin:"top-left"` — the drawn rect is then guaranteed to sit inside the clip by
construction, satisfying "must not be cropped" without needing any alignment concept the spec doesn't have.

**Hesitated:** whether `x,y` even needed to be specified explicitly given `clip` already encodes a
rectangle — yes, `x,y,origin` position the *drawn rect*, `clip` is a separate, static aperture; they're
independent per the spec, so both had to be set deliberately.

**Wanted an error message saying:** the spec never states a default/expected relationship between an
element's `x,y` and its `clip` — an error (or a documented convention) like `"contain: drawn rect not
covering any of the clip; did you mean to set x,y to the clip's origin?"` would have saved the double-take.

---

## Task 4 — Deliberate distortion (images/07.png, 1536x2720 → squashed to 1080x1600)

**Wrote (changed fields only; scale keyframes carried over untouched from the existing `photo-07` element):**
```json
{"id":"photo-07","type":"image","group":"item-07","start":30603,"end":42763,"source":"images/07.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1600,"fit":"literal","clip":[0,0,1080,1300],
 "scale":[{"t":30603,"v":[1.0,1.0]},{"t":45603,"v":[1.08,1.08],"ease":"linear"}]}
```

**Arithmetic:** none required — "wrong aspect ratio, on purpose" is exactly `literal`'s definition
("no rule; the integers are yours"), so `width:1080,height:1600` are written directly, no cover/contain
check applies, and `validate` won't report a mismatch under `literal`.

**First reached for:** I first wondered whether `fit:"literal"` would also suppress or interact with the
`scale` animation keyframes (since both `fit`-driven width/height and `scale` affect final drawn size) —
the spec is silent on `scale` entirely (it's not mentioned anywhere in this reference), so I treated it as
an orthogonal field outside this spec's scope and left it byte-for-byte as in the source project file,
per the task's explicit instruction not to touch it.

**Hesitated:** confirming `clip` is "optional" under `literal` per spec, but chose to keep it anyway since
the task didn't say to remove the aperture, just to change the drawn size — `clip` still crops the 1600px-tall
distorted image down to the 1300px-tall aperture, which seems to be the intended visual result.

**Wanted an error message saying:** N/A — `literal` accepts any integers by design, nothing to validate.

---

## Task 5 — Rotated phone photo (images/10.jpg, stored 3024x4032, EXIF orientation 6)

**Wrote:**
```json
{"id":"photo-10","type":"image","start":30000,"end":36000,"source":"images/10.jpg",
 "x":0,"y":0,"origin":"top-left","width":1733,"height":1300,"fit":"cover","clip":[0,0,1080,1300]}
```

**Arithmetic:** Spec: "use the orientation-applied pixel dimensions... if EXIF flag transposes the image."
Orientation 6 (rotate 90° CW for display) transposes stored 3024(w)x4032(h) to displayed sw=4032, sh=3024.
box = clip w,h = 1080x1300.
`bw*sh = 1080*3024 = 3,265,920`; `bh*sw = 1300*4032 = 5,241,600`. `bw*sh >= bh*sw`? 3,265,920 >= 5,241,600
is **false** → height drives (not width). height = bh = 1300 (verbatim).
slack (width) = `(sw*bh)//sh = (4032*1300)//3024 = 5,241,600//3024 = 1733` (floor, 1733.33→1733).
Check `cover`: width(1733) >= box_w(1080) ✓, height(1300) >= box_h(1300) ✓.

**First reached for:** I first reached for the *stored* dimensions (3024x4032) straight off the task
description before noticing the EXIF-orientation sentence in the spec ("Source dimensions are the
image's orientation-applied pixel dimensions") — using the stored (untransposed) values would have
flipped which axis drives the `cover` calc and produced a wrong width/height pair entirely.

**Hesitated:** genuinely had to stop and reason through "orientation 6 = rotate 90° CW to display" →
which of stored-w/stored-h becomes displayed-w/displayed-h. Landed on: displayed_w = stored_h,
displayed_h = stored_w, i.e. sw=4032, sh=3024. The spec states the *rule* (use transposed dims when the
flag transposes) but does not itself spell out the CW/CCW-to-swap mapping for each EXIF value — that
part I had to reason about outside the spec's text.

**Wanted an error message saying:** the spec would be strengthened by literally stating "orientation 6
→ swap stored width and height" (or a short table for 3/6/8) rather than leaving the EXIF-to-transposition
mapping to be inferred.

---

## Task 6 — Crop to the top third of a tall photo (images/11.png, 1080x3000, aperture [0,0,1080,1000])

**Wrote:**
```json
{"id":"photo-11","type":"image","start":30000,"end":36000,"source":"images/11.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":3000,"fit":"cover","clip":[0,0,1080,1000]}
```

**Arithmetic:** sw=1080, sh=3000 (source already matches box width, no scaling needed to fill horizontally).
box = clip w,h = 1080x1000.
`bw*sh = 1080*3000 = 3,240,000`; `bh*sw = 1000*1080 = 1,080,000`. `bw*sh >= bh*sw` → width drives,
width = bw = 1080 (verbatim). slack (height) = `(sh*bw)//sw = (3000*1080)//1080 = 3000` exactly.
So the image is drawn at its full native 1080x3000 (no scaling at all — width already equals the box
width), and `clip:[0,0,1080,1000]` is what reveals only the top 1000px of the 3000px-tall drawn image,
i.e. the top third. `cover` check: width(1080)>=box_w(1080) ✓, height(3000)>=box_h(1000) ✓.

**First reached for:** I first reached for a `crop`/`offset`-style field — something that would let me
say "start the visible window 0px from the top" directly — before remembering the spec's explicit "Not
fields" line: "no `crop`... and no `box`." Realized the *only* mechanism for "which part is visible" is
the static `clip` rect combined with where the drawn rect is positioned (`x,y,origin`), not any per-element
crop-offset concept.

**Hesitated:** whether this counts as `cover` or `literal`. Since 1080x3000 satisfies the `cover` formula
exactly (width already equals box width with zero scaling), I went with `cover` rather than `literal` —
`literal` felt like it should be reserved for when the numbers *don't* follow a formula. This is a case
where two `fit` values could describe the same on-screen result, and the spec doesn't say which is
preferred when a `literal`-looking placement also happens to satisfy `cover`'s arithmetic.

**Wanted an error message saying:** N/A — nothing invalid here — but a spec line saying "if width/height
happen to already satisfy `cover`/`contain`'s arithmetic, prefer that value over `literal`" would resolve
the ambiguity I hit above.

---

## Verdict

**Usable as published, with one serious gap: the vocabulary for "which part of a taller-than-needed
drawn image is visible" is under-specified relative to how people actually think about cropping.**
`clip` is a *static frame-space* rectangle, not tied to the element's position — so to "show the top
third" of an image (Task 6) or "no cropping at all" (Task 3), the author has to hand-derive a
relationship between `x,y,origin`, `width/height`, and `clip` that the spec never states as a named
concept. Three separate times (Tasks 1, 3, 6) my first instinct reached for a field the spec explicitly
says doesn't exist (`gravity`, `align`, `crop`) — that's a strong signal that the *mental model* authors
bring to "framing a photo" doesn't match this vocabulary's actual primitives, even though the primitives
themselves (once understood) are precise and computable.

**Single worst thing:** there is no way to express "align the drawn rect to a corner/edge of its clip"
short of manually computing `x`/`y`/`width`/`height` by hand — every one of my 6 tasks needed me to
independently reconstruct that "draw the aperture-aligned rect starting at the clip's own x,y" was the
intended pattern, because the spec never names or requires that relationship.

**One concrete change:** add a worked, named convention (even just one sentence) for the common case —
e.g. "when an element's visible region should exactly equal `clip`, set the element's `x,y,origin` to
match `clip`'s top-left" — or, better, promote it into the schema as an explicit default so `x,y` need
not be redundantly restated whenever they equal `clip`'s origin.
