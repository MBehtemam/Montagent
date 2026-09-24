# Worklog — arm: literal — agent G (fable)

Frame 1080x1920. Spec: SPEC-literal.md only. Fixture read for context:
`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`.

**Immediate observation:** every photo element in the fixture carries `"gravity":"top"`,
but the spec's "Not fields" section says there is no `gravity` field. The published
example corpus contradicts the published reference. I followed the spec and omitted it.

---

## Task 1 — New photo (images/09.png, 1536x2720, cover into [0,0,1080,1300])

**Arithmetic:** box 1080x1300. `bw*sh = 1080*2720 = 2937600 >= bh*sw = 1300*1536 = 1996800`
→ width drives under cover. width = 1080 (box verbatim). height = `(2720*1080)//1536 =
2937600//1536 = 1912` (exact 1912.5, floor). Cover check: 1080>=1080, 1912>=1300. Drawn
rect [0,0,1080,1912] contains clip [0,0,1080,1300]. Matches the sibling photos exactly.

**Wrote:**
```json
{"id":"photo-09","type":"image","start":30000,"end":36000,"source":"images/09.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1912,"fit":"cover",
 "clip":[0,0,1080,1300]}
```

- **First reached for:** copying `"gravity":"top"` from the neighboring fixture elements —
  a field the spec says does not exist. I dropped it only because the spec's "Not fields"
  list called it out explicitly.
- **Hesitated:** whether floor (1912) vs the exact 1912.5 mattered; re-read the validate
  section — floor or ceil of the exact value is accepted, so 1912 or 1913 both pass.
- **Wanted an error message saying:** n/a (no error hit), but if I had shipped `gravity`,
  I'd want: `unknown field "gravity" on image "photo-09" — position with x/y/origin instead`.

## Task 2 — Re-exported source (images/06.png now 1536x2200)

**Arithmetic:** `bw*sh = 1080*2200 = 2376000 >= bh*sw = 1996800` → width still drives.
width = 1080, height = `(2200*1080)//1536 = 2376000//1536 = 1546` (exact 1546.875; floor
1546 or ceil 1547 both legal). Aperture coverage: [0,0,1080,1546] contains [0,0,1080,1300] ✓.
The stale 1912 was the error: slack axis no longer floor/ceil of the exact value.

**Wrote (changed fields only):**
```json
{"height": 1546}
```
(`width` stays 1080, `fit` stays `"cover"`, `clip` unchanged.)

- **First reached for:** exactly this — recompute `height`. No wrong turn.
- **Hesitated:** briefly wondered whether the renderer would rescale automatically and
  only `fit` needed re-declaring; re-read "the source is resampled to exactly this rect" —
  no, the integers I write are what is drawn, so the number itself must change.
- **Wanted an error message saying:** `photo-06: fit "cover" with source 1536x2200 into box
  1080x1300 requires height 1546 or 1547, found 1912`. Anything less than the expected
  number forces the author to redo the arithmetic blind.

## Task 3 — Badge, nothing cropped (brand/badge.png 1200x400 into [840,1700,200,160])

**Arithmetic:** contain. `bw*sh = 200*400 = 80000 <= bh*sw = 160*1200 = 192000` → width
drives. width = 200, height = `(400*200)//1200 = 80000//1200 = 66` (exact 66.67, floor).
Contain check: 200<=200, 66<=160. Vertically centered by hand: y = 1700 + (160-66)//2 = 1747.
Aperture coverage rule (clip contains drawn rect): [840,1747,200,66] inside [840,1700,200,160] ✓.

**Wrote:**
```json
{"id":"badge","type":"image","start":30000,"end":36000,"source":"brand/badge.png",
 "x":840,"y":1747,"origin":"top-left","width":200,"height":66,"fit":"contain",
 "clip":[840,1700,200,160]}
```
(Task gave no times; I picked 30000-36000 as a placeholder.)

- **First reached for:** an `align` / `gravity` value (something like `"align":"center"`)
  to center the letterboxed badge in the aperture — neither exists on images. Centering has
  to be baked into `x`/`y` arithmetic by the author.
- **Hesitated:** whether `clip` coordinates are frame-space or element-relative; re-read —
  "a static rect in frame space", so the 840/1700 repeat in both the position and the clip.
- **Wanted an error message saying:** if I'd left y at 1700 nothing would error (still
  contained), so no message needed — but the absence of any alignment vocabulary is the pain.

## Task 4 — Deliberate distortion (images/07.png 1536x2720 → exactly 1080x1600)

**Arithmetic:** none, by design — that is what `fit:"literal"` is for. 1080x1600 satisfies
neither cover (1600 >= 1300 but the pair matches no integer-derived rect) nor contain, so
declaring either would be a validate error; `literal` switches the checks off. `clip` is
optional under literal but I keep it: the aperture crop is still wanted. The `scale`
keyframes are untouched — spec says fit/width/height do not interact with animation.

**Wrote (changed fields only, on photo-07):**
```json
{"width": 1080, "height": 1600, "fit": "literal"}
```
Everything else — including `"scale":[{"t":30603,"v":[1.0,1.0]},{"t":45603,"v":[1.08,1.08],"ease":"linear"}]` — stays exactly as it is.

- **First reached for:** a `stretch` or `fill` value for `fit` (CSS `object-fit: fill`
  muscle memory). Neither is legal; the closed set is cover/contain/literal, and
  `literal` is the escape hatch.
- **Hesitated:** whether dropping `fit` entirely was allowed for "no rule" — no, `fit` is
  required on every raster element; "no rule" is spelled `"literal"`. Also re-read whether
  removing the cover-check would somehow affect the scale animation — it does not; `fit`
  never changes what the renderer draws.
- **Wanted an error message saying:** n/a.

## Task 5 — Rotated phone photo (images/10.jpg, stored 3024x4032, EXIF orientation 6)

**Arithmetic:** orientation 6 transposes → effective source 4032x3024 (landscape). Cover
into 1080x1300: `bw*sh = 1080*3024 = 3265920 < bh*sw = 1300*4032 = 5241600` → height
drives. height = 1300, width = `(4032*1300)//3024 = 5241600//3024 = 1733` (exact 1733.33;
floor 1733 or ceil 1734 legal). Cover check: 1733>=1080, 1300>=1300 ✓.

**Answer:** `"width":1733,"height":1300` (with `"fit":"cover","clip":[0,0,1080,1300]`).

- **First reached for:** the stored dimensions 3024x4032 — I started to compute with the
  portrait numbers before the spec's orientation-applied sentence stopped me. With stored
  dims the answer would have been 1080x1440, silently wrong.
- **Hesitated:** none after that; the spec is explicit that EXIF-transposed dimensions
  are the source dimensions.
- **Wanted an error message saying:** validate reads the source, so it should catch this:
  `photo-10: source is 4032x3024 after EXIF orientation (stored 3024x4032); declared rect
  matches the un-oriented dimensions`. Naming both dimension pairs is what would fix an
  author fastest.

## Task 6 — Top third of a tall photo (images/11.png 1080x3000 into [0,0,1080,1000])

**Arithmetic:** cover into box 1080x1000: `bw*sh = 1080*3000 = 3240000 >= bh*sw =
1000*1080 = 1080000` → width drives. width = 1080, height = `(3000*1080)//1080 = 3000`
(exact). Drawn at 1:1, anchored top-left at y=0, the aperture [0,0,1080,1000] shows source
rows 0-999 of 3000 — exactly the top third. Cover coverage check: [0,0,1080,3000] contains
[0,0,1080,1000] ✓.

**Wrote:**
```json
{"id":"photo-11","type":"image","start":36000,"end":42000,"source":"images/11.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":3000,"fit":"cover",
 "clip":[0,0,1080,1000]}
```
(Times were unspecified; placeholder.)

- **First reached for:** a `crop` field — `"crop":[0,0,1080,1000]` in source pixels — to
  name the top third directly. It does not exist ("no `crop`"). The crop is instead an
  emergent property of drawn-rect size + position + aperture, which only works out this
  cleanly because the source width equals the box width. Had the author wanted the *middle*
  third, the recipe is a negative y (`y:-1000`), which the spec never demonstrates.
- **Hesitated:** whether `fit:"cover"` was even the right declaration since my real intent
  was a source-space crop; it validates, but the declared `fit` records a derivation, not
  the intent.
- **Wanted an error message saying:** n/a (validates), but nothing in the file records
  "top third" — a future editor sees only 1080x3000 and must reverse-engineer the intent.

---

## Verdict

Usable, but only just — it is a calculator's format, not an author's. Every element is
correct and validate-clean, yet three of six tasks required me to hand-derive integers
(1912, 1546, 1733) that the renderer/validator could compute itself, and the fixture that
ships alongside the spec contradicts it by using `gravity` on every photo.

**Single worst thing:** `width`/`height` are simultaneously *derived output* (from
fit+source+clip, checked by validate) and *authoritative input* (the renderer draws
exactly these integers). Authors must run the tool's own arithmetic by hand, in integer
math, with a floor/ceil ambiguity, and re-run it every time a source is re-exported
(Task 2 is exactly that failure mode).

**One concrete change:** make `width`/`height` optional when `fit` is `cover`/`contain` —
the rule plus source plus clip fully determines them; let validate/render fill them in
(or a `montagent fmt` write them back). Keep them required only under `literal`, where the
integers genuinely are the author's. That deletes the whole class of stale-integer errors
and the EXIF trap in one move. (Runner-up: reconcile the spec with the fixture on
`gravity`, and give images an alignment vocabulary so Task 3's centering isn't manual
pixel arithmetic.)
