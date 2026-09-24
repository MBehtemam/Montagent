# Worklog — arm: declared (Opus)

Source of truth: `SPEC-declared.md` only. Did not open `docs/adr/`.
Frame 1080x1920. Context read: `fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json` (not edited).

Helper used for every computation (integer only, floor division):

```python
def cover(sw,sh,bw,bh):
    return (bw,(sh*bw)//sw) if bw*sh >= bh*sw else ((sw*bh)//sh,bh)
def contain(sw,sh,bw,bh):
    return (bw,(sh*bw)//sw) if bw*sh <= bh*sw else ((sw*bh)//sh,bh)
```

---

## Task 1 — new photo, images/09.png 1536x2720

**Wrote:**

```json
{"id":"photo-09","type":"image","group":"item-09","start":30000,"end":36000,"source":"images/09.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1912,
 "fit":"cover","clip":[0,0,1080,1300]}
```

**Arithmetic.** Box = clip's w,h = 1080x1300. Source 1536x2720.
Driving test for `cover`: `bw*sh = 1080*2720 = 2937600`, `bh*sw = 1300*1536 = 1996800`.
`2937600 >= 1996800` → **width drives**. width = box_w = **1080** (verbatim).
Slack axis: `(s_slack * b_driving) // s_driving = (2720*1080)//1536 = 2937600//1536 = 1912.5 → 1912`.
Coverage check: 1080 >= 1080 and 1912 >= 1300; drawn rect (0,0,1080,1912) contains clip (0,0,1080,1300). OK.

**First reached for:** `"fit":"cover"` with `width`/`height` set to the *frame* box — I started to compute against 1080x1920 before the spec corrected me that the box is **clip's w and h**, not the frame and not the element's own rect. I also, before reading, half-expected `fit:"cover"` to mean the renderer would do the fitting and that `width`/`height` could be omitted — the spec is explicit that they are required and are drawn verbatim, and that `fit` is only a record of the rule.
I also reached for `"gravity":"top"`, because every existing photo element in the shipped fixture carries it. The spec's "Not fields" section says there is no `gravity` field. I left it out. **The published spec and the shipped fixture disagree.**

**Hesitated:** whether the slack value 1912.5 should floor or round. Spec: slack is `//` (floor) when authoring, and validate accepts floor *or* ceil. Floor chosen.

**Wanted an error message saying:** n/a (nothing wrong), but a message of the shape `fit "cover": box is clip w/h (1080x1300), not the frame` would have pre-empted my first instinct.

---

## Task 2 — images/06.png re-exported 1536x2720 → 1536x2200

**Changed fields (on `photo-06`):**

```json
"height": 1546
```

Full element:

```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,"source":"images/06.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1546,
 "fit":"cover","clip":[0,0,1080,1300],
 "scale":[{"t":17472,"v":[1.0,1.0]},{"t":32472,"v":[1.08,1.08],"ease":"linear"}]}
```

**Arithmetic.** Box 1080x1300, source now 1536x2200.
`bw*sh = 1080*2200 = 2376000`, `bh*sw = 1300*1536 = 1996800` → width still drives.
width = **1080**. height = `(2200*1080)//1536 = 2376000//1536 = 1546.875 → 1546`.
Aperture coverage: 1546 >= 1300, so the drawn rect still fully contains clip. Both conditions hold, so `fit:"cover"` stays.

**First reached for:** changing `fit` to `"declared"` to silence the error. That is the wrong fix — it makes validate stop checking rather than restoring the intent, and the task asks for the aperture to stay covered. Recomputing the slack axis is the right fix. Before that, I briefly reached for scaling the old height proportionally (1912 * 2200/2720) — a float path the spec forbids; it happens to land on 1546.7 → same floor, but it is not the stated rule.

**Hesitated:** whether 1547 (ceil) would also pass. Per spec, validate accepts floor or ceil on the slack axis, so 1547 is legal too; I kept the authoring rule (floor) since 1546 still covers 1300.

**Wanted an error message saying:** `photo-06: fit "cover" with clip 1080x1300 and source 1536x2200 requires height 1546 (or 1547); found 1912` — i.e. name the expected number. "Declared rect disagrees with the rule" alone would have left me guessing which axis.

---

## Task 3 — badge, 1200x400, entirely visible in aperture [840,1700,200,160]

**Wrote:**

```json
{"id":"badge","type":"image","start":0,"end":65216,"source":"brand/badge.png",
 "x":840,"y":1747,"origin":"top-left","width":200,"height":66,
 "fit":"contain","clip":[840,1700,200,160]}
```

**Arithmetic.** "Entirely visible, nothing cut off" → `contain`. Box 200x160, source 1200x400.
Driving test for `contain`: `bw*sh = 200*400 = 80000`, `bh*sw = 160*1200 = 192000`.
`80000 <= 192000` → **width drives**. width = **200**. height = `(400*200)//1200 = 80000//1200 = 66.67 → 66`.
Check `contain` inequality: 200 <= 200 and 66 <= 160. OK.
Aperture coverage under `contain`: clip must contain the drawn rect. Drawn rect is (840, y, 200, 66). Vertically centred in the 160-tall aperture: `y = 1700 + (160-66)//2 = 1700 + 47 = 1747`. Rect spans y 1747..1813, inside 1700..1860. OK.

**First reached for:** `"fit":"contain"` plus an alignment field to centre it in the taller aperture — I wanted `"gravity":"center"`, and failing that `"align":"center"`. Neither exists (`align` exists on text elements in the fixture but the spec says there is no `align` on images). The only way to centre is to compute `y` by hand. That arithmetic — `1700 + (160-66)//2` — is exactly the kind of thing an author gets wrong silently, and nothing validates it.

**Hesitated:** whether `x`/`y` are independent of `clip` at all. The spec says `clip` is a static rect in frame space that does not move with the element, which implies I must position the element into the aperture myself; if I had left `x:0,y:0` the badge would be clipped away entirely and (I believe) `validate` would fire the coverage error. That relationship is inferable but never stated directly.

**Wanted an error message saying:** `fit "contain": drawn rect (840,1700,200,66) is not inside clip (840,1700,200,160)` — with both rects printed, so the offset is obvious.

---

## Task 4 — deliberate distortion, images/07.png 1536x2720 → exactly 1080x1600

**Changed fields (on `photo-07`):**

```json
"height": 1600,
"fit": "declared"
```

Full element (animation untouched):

```json
{"id":"photo-07","type":"image","group":"item-07","start":30603,"end":42763,"source":"images/07.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1600,
 "fit":"declared","clip":[0,0,1080,1300],
 "scale":[{"t":30603,"v":[1.0,1.0]},{"t":45603,"v":[1.08,1.08],"ease":"linear"}]}
```

**Arithmetic.** None — that is the point. `declared` means "no rule; the integers are yours", and the rule check is not reported under `declared`. The correct aspect for this source into this box would be 1080x1912 (Task 1); 1600 is deliberately not that, so no rule-bearing `fit` value can express it. `width`/`height` are drawn verbatim, so 1080x1600 is exactly what is resampled.
The `scale` keyframe array is a separate animated property and is copied through byte-for-byte; changing `height` does not touch it. (The 1.0→1.08 Ken Burns still reads, because it multiplies whatever rect is drawn.)

**First reached for:** `"fit":"none"` — the CSS-ish value my hands typed first. Not legal; the closed set is `cover` / `contain` / `declared`. I also expected `"fill"` (the CSS `object-fit` value that means exactly "stretch to the box, ignore aspect") to be the intended spelling. `declared` is the right value here but it is the least guessable name of the three, because it describes the *author's provenance* rather than the *visual result*.

**Hesitated:** two things. (a) Whether aperture coverage is still checked under `declared` — the spec lists it as a separate error bullet from the rule check and only names `cover`/`contain`, so I read it as unchecked; 1080x1600 contains the 1080x1300 clip regardless, so it is moot here, but I could not settle it from the text. (b) Whether `clip` should be dropped since it is "optional" under `declared` — I kept it, because dropping it would unclip the element and paint the full 1600px into the frame. "Optional" is misleading: omitting it changes the picture.

**Wanted an error message saying:** on typing `"fit":"fill"` — `unknown fit "fill"; legal values are cover, contain, declared (use "declared" for a deliberate off-aspect rect)`. The parenthetical is what would have saved me.

---

## Task 5 — rotated phone photo, images/10.jpg 3024x4032 stored, EXIF orientation 6

**Wrote:**

```json
{"id":"photo-10","type":"image","start":0,"end":6000,"source":"images/10.jpg",
 "x":0,"y":0,"origin":"top-left","width":1733,"height":1300,
 "fit":"cover","clip":[0,0,1080,1300]}
```

**Answer: `width: 1733`, `height: 1300`.**

**Arithmetic.** EXIF orientation 6 transposes the image, so the orientation-applied dimensions are **4032x3024** (landscape), not the stored 3024x4032. Box 1080x1300.
`bw*sh = 1080*3024 = 3265920`, `bh*sw = 1300*4032 = 5241600`. `3265920 >= 5241600` is **false** → **height drives**. height = box_h = **1300** verbatim.
Slack axis: `(sw * bh)//sh = (4032*1300)//3024 = 5241600//3024 = 1733.33 → 1733`.
Cover check: 1733 >= 1080 and 1300 >= 1300. Drawn rect (0,0,1733,1300) contains clip (0,0,1080,1300). OK — the 653px of horizontal overflow is clipped, which is what `cover` means.

**First reached for:** the stored dimensions, 3024x4032, straight off the file. That gives width-drives, 1080x1440 — wrong, and it would have rendered a rotated photo squeezed into a portrait rect. The spec's orientation paragraph is the only thing that stopped me, and I only re-read it because the task said "rotated". **If the task had not flagged it, I would have shipped 1080x1440.** This is the single most dangerous field in the format: the number I need is not the number the file reports, and nothing in the element records which one I used.

**Hesitated:** whether orientation 6 is 90 CW or CCW. It does not matter for sizing — either way the dimensions transpose to 4032x3024 — but I had to reason that out rather than read it.

**Wanted an error message saying:** `images/10.jpg: EXIF orientation 6 — using orientation-applied size 4032x3024 (stored 3024x4032); fit "cover" requires 1733x1300, found 1080x1440`. Printing *both* sizes is the fix; an error that printed only "expected 1733x1300" would have left me thinking validate was broken.

---

## Task 6 — top third of images/11.png 1080x3000 in aperture [0,0,1080,1000]

**Wrote:**

```json
{"id":"photo-11","type":"image","start":0,"end":6000,"source":"images/11.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":3000,
 "fit":"cover","clip":[0,0,1080,1000]}
```

**Arithmetic.** The aperture is 1000 tall and the source is 3000 tall; the source width 1080 already equals the aperture width, so drawing at natural size 1080x3000 with the element's top-left at the aperture's top-left means the aperture reveals source rows 0..999 — exactly the **top third**. The crop-to-top is achieved by *placement*, not by any crop field: `clip` is static in frame space, the element is positioned so its top aligns with it, and everything below y=1000 is not painted.
`fit` check — box 1080x1000, source 1080x3000: `bw*sh = 1080*3000 = 3240000`, `bh*sw = 1000*1080 = 1080000` → width drives, width = **1080**, slack height = `(3000*1080)//1080 = 3000`. So `cover` reproduces 1080x3000 exactly; it is the honest value and validate passes.
Coverage: 1080 >= 1080, 3000 >= 1000, drawn rect contains clip. OK.

**First reached for:** a source-space crop. My first thought was literally `"crop":[0,0,1080,1000]`, and when that did not exist, `"clip"` interpreted as a source-space rect. Both wrong: the spec says explicitly there is no `crop`, and `clip` is **frame** space. My third reach was `"origin":"top-center"` plus `"gravity":"top"` to pin the visible band — again, `gravity` does not exist.
Getting from "show the top third" to "draw the whole thing at natural size and let a frame-space aperture hide the rest" took three wrong turns. For a non-square source it would also have required computing the offset `y` by hand.

**Hesitated:** whether `declared` was more honest here than `cover`, since my intent was "natural size", not "cover the box". They happen to produce identical integers. I chose `cover` because it is the rule that actually justifies the numbers and it keeps validate checking them if the source is ever re-exported (cf. Task 2) — `declared` would have silently accepted a stale rect.

**Wanted an error message saying:** `there is no "crop" field — to show part of a source, position the element and use "clip" (frame-space aperture)`. An unknown-field error that names the replacement concept, not just the typo.

---

## Verdict

**Is this vocabulary usable as published? Barely — yes for `cover`/`contain` on a full-bleed photo, no for anything that needs placement.** All six tasks are expressible and the integer arithmetic is unambiguous and reproducible, which is a real strength: two authors following this spec get the same numbers, and the floor/ceil tolerance on the slack axis is a sensible bit of give. But three of the six tasks required arithmetic the format does not help with and does not check.

**The single worst thing:** `fit` is documentation, not behaviour — "it does not change what the renderer draws" — while carrying the names (`cover`, `contain`) of the CSS property where it *is* behaviour. Every instinct I brought from `object-fit` was wrong in the same direction: I expected to declare intent and have the renderer size the rect, when in fact I must compute the rect myself and then separately assert which rule I used. The two can silently disagree the moment a source is re-exported (Task 2 is precisely that failure), and `declared` exists as a one-word opt-out of the only check that catches it. Close second: the spec's "Not fields" list denies `gravity`, but every image element in the shipped fixture carries `"gravity":"top"` — so the published documentation contradicts the shipped file, and an author reading the fixture for examples learns a field that does not exist.

**One concrete change:** let `width`/`height` be **omitted** when `fit` is `cover` or `contain`, and have the renderer compute them by the published integer rule. Keep them writable for `declared` (and for anyone who wants them pinned), but stop requiring the author to hand-carry a derived number that the tool can derive exactly. That single change deletes Task 2's entire class of bug — a re-export just works — and turns `fit` from an assertion that can rot into an instruction that cannot. If they must stay required, then the second-best change is to make the error message print the expected integers and, for any source with an EXIF orientation flag, print both the stored and orientation-applied dimensions (Task 5).

**Fields/values I reached for that do not exist:** `gravity` (twice — and it is in the fixture), `align` on an image, `crop`, `fit:"none"`, `fit:"fill"`, `clip` as a source-space rect, and omitting `width`/`height` under `cover`.
