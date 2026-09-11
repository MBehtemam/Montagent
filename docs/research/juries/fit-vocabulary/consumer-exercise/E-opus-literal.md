# Worklog — arm: literal (Opus)

Sources: `SPEC-literal.md` only, plus `fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`
for context. No `docs/adr/` was read.

Helper used for all arithmetic (integer only, floor division):

```python
def cover(sw,sh,bw,bh):
    if bw*sh >= bh*sw: return bw, (sh*bw)//sw          # width drives
    else:              return (sw*bh)//sh, bh          # height drives
def contain(sw,sh,bw,bh):
    if bw*sh <= bh*sw: return bw, (sh*bw)//sw          # width drives
    else:              return (sw*bh)//sh, bh          # height drives
```

---

## Task 1 — new photo, `images/09.png` 1536x2720, aperture [0,0,1080,1300]

**Wrote:**

```json
{"id":"photo-09","type":"image","group":"item-09","start":30000,"end":36000,
 "source":"images/09.png","x":0,"y":0,"origin":"top-left",
 "width":1080,"height":1912,"fit":"cover","clip":[0,0,1080,1300]}
```

**Arithmetic.** Box = clip w x h = 1080 x 1300. Source 1536 x 2720.
`cover` width-drives test: `bw*sh = 1080*2720 = 2937600` >= `bh*sw = 1300*1536 = 1996800` -> width drives.
Driving axis verbatim: `width = 1080`.
Slack axis: `(sh * bw) // sw = (2720*1080)//1536 = 2937600//1536 = 1912` (exact value 1912.5, floored).
Coverage: 1080 >= 1080 and 1912 >= 1300 -> aperture covered.

**First reached for:** `"fit":"cover"` (correct) **plus `"gravity":"top"`** — I copied the shape of the
existing `photo-06` element, which carries `"gravity":"top"`. The spec's "Not fields" section says
there is no `gravity` field. **The shipped fixture on `main` contains a field the published spec
declares nonexistent**; if I had not read the "Not fields" paragraph last, I would have written it.
Second wrong instinct: computing the slack axis as `round(2720*1080/1536) = 1913` (float + round)
rather than integer floor `1912`.

**Hesitated:** whether the box for `fit` is the *frame* (1080x1920) or the *clip*. The spec states it
once, in one line ("The box the rule works against is **`clip`'s w and h**"). With frame as the box the
answer would have been 1080x1912 *by coincidence here* — this task cannot distinguish the two readings,
which is a trap.

**Wanted an error message saying:** `gravity: unknown field on image elements (removed; use x/y/origin
plus clip to position the aperture)`.

---

## Task 2 — `images/06.png` re-exported 1536x2720 -> 1536x2200

**Changed fields:** `"height": 1912` -> `"height": 1546`. Everything else (`width` 1080, `fit` `cover`,
`clip` [0,0,1080,1300], x/y/origin, the `scale` keyframes) unchanged.

Full element:

```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,
 "source":"images/06.png","x":0,"y":0,"origin":"top-left",
 "width":1080,"height":1546,"fit":"cover","clip":[0,0,1080,1300],
 "scale":[{"t":17472,"v":[1.0,1.0]},{"t":32472,"v":[1.08,1.08],"ease":"linear"}]}
```

**Arithmetic.** Box 1080x1300, source 1536x2200. `1080*2200 = 2376000 >= 1300*1536 = 1996800` -> width
drives. `width = 1080`. `height = (2200*1080)//1536 = 2376000//1536 = 1546` (exact 1546.875, floored).
`1547` (ceil) is also legal by the spec's "floor or ceil" allowance; I wrote the floor because the
stated formula is floor. Coverage: 1546 >= 1300 -> aperture still fully covered.

**First reached for:** `1547` — I computed 1546.875 and rounded to nearest out of habit. Also my first
diagnosis of the error was "the aspect ratio is stale, scale height by 2200/2720" (= 1546.4), which
lands one pixel *off* the spec's answer. The ratio-scaling shortcut is wrong; the rule recomputes from
the box, not from the old rect.

**Hesitated:** "the slack axis must be `floor` or `ceil`" is in the *validate* section, while the
*arithmetic* section says floor flatly. Two numbers pass; only one is "the" rule. I had to re-read to
convince myself 1546 was not under-covering.

**Wanted an error message saying:** `fit "cover": height 1912 does not match source 1536x2200 in box
1080x1300; width drives, expected height 1546 (or 1547).`

---

## Task 3 — `brand/badge.png` 1200x400, fully visible in aperture [840,1700,200,160]

**Wrote:**

```json
{"id":"badge","type":"image","start":0,"end":65216,"source":"brand/badge.png",
 "x":840,"y":1747,"origin":"top-left",
 "width":200,"height":66,"fit":"contain","clip":[840,1700,200,160]}
```

**Arithmetic.** "Entirely visible, nothing cut off" -> `contain`. Box 200x160, source 1200x400.
`contain` width-drives test: `bw*sh = 200*400 = 80000` <= `bh*sw = 160*1200 = 192000` -> width drives.
`width = 200`. `height = (400*200)//1200 = 80000//1200 = 66` (exact 66.67, floored).
Check: 200 <= 200 and 66 <= 160. Coverage rule: clip must contain the drawn rect —
rect [840,1747,200,66] lies inside [840,1700,200,160]. Vertical centring: `1700 + (160-66)//2 = 1747`.

**First reached for:** `"fit":"contain"` — correct, existed. But I first reached for
**`"align":"center"`** to centre the letterboxed badge in its aperture, and the spec's "Not fields"
says there is no `align` on images. I had to centre it by hand in `y`. (`align` *does* exist on text
elements in the fixture, which is exactly why I reached for it.)

**Hesitated:** whether `x`/`y` are the drawn rect's position or the aperture's. The spec never says
they must coincide; I concluded they are independent (clip is "a static rect in frame space"), so I
offset `y` by 47px while `clip` stays at 1700. This is the part I am least sure a validator agrees with.

**Wanted an error message saying:** `align: not a field on image elements; position the drawn rect with
x/y/origin (its size under "contain" is 200x66, aperture is 200x160).`

---

## Task 4 — deliberate distortion, `images/07.png` 1536x2720 squashed to 1080x1600

**Changed fields:** `"height": 1912` -> `"height": 1600`, `"fit": "cover"` -> `"fit": "literal"`.
`scale` keyframes untouched.

```json
{"id":"photo-07","type":"image","group":"item-07","start":30603,"end":42763,
 "source":"images/07.png","x":0,"y":0,"origin":"top-left",
 "width":1080,"height":1600,"fit":"literal","clip":[0,0,1080,1300],
 "scale":[{"t":30603,"v":[1.0,1.0]},{"t":45603,"v":[1.08,1.08],"ease":"linear"}]}
```

**Arithmetic.** None — that is the point. `literal` means "no rule; the integers are yours", the box is
not used, and validate does not report the rect/rule disagreement or the aperture-coverage error under
`literal`. The correct-aspect `cover` rect would be 1080x1912; 1600 is deliberately 312px short of it,
so any fit other than `literal` would be an error. `clip` is optional under `literal`; I kept it so the
element still reads through the same aperture as its neighbours. `scale` is a separate animated
property and is unrelated to `width`/`height`, so it stays byte-for-byte as it was.

**First reached for:** `"fit":"none"` — that value does not exist; the closed set is
`cover` / `contain` / `literal`. `none` is the CSS `object-fit` spelling and it was my first thought.
I also briefly considered expressing the squash as a non-uniform `scale` of `[1.0, 0.837]` instead of
touching `width`/`height` — which would have silently broken the required zoom animation.

**Hesitated:** whether dropping `clip` was mandatory under `literal` ("the box is not used"). It is
optional, not forbidden — re-read to confirm.

**Wanted an error message saying:** `fit: "none" is not a value. Legal: cover, contain, literal. Use
"literal" for an intentionally off-aspect rect.`

---

## Task 5 — `images/10.jpg`, stored 3024x4032, EXIF orientation 6, cover [0,0,1080,1300]

**Wrote:** `"width": 1733, "height": 1300`.

```json
{"id":"photo-10","type":"image","start":0,"end":6000,"source":"images/10.jpg",
 "x":0,"y":0,"origin":"top-left",
 "width":1733,"height":1300,"fit":"cover","clip":[0,0,1080,1300]}
```

**Arithmetic.** EXIF 6 transposes, so the orientation-applied source is **4032 x 3024** (landscape),
not 3024x4032. Box 1080x1300.
Width-drives test: `bw*sh = 1080*3024 = 3265920` >= `bh*sw = 1300*4032 = 5241600`? **No** -> height drives.
Driving axis verbatim: `height = 1300`.
Slack axis: `(sw * bh) // sh = (4032*1300)//3024 = 5241600//3024 = 1733` (exact 1733.33, floored).
Coverage: 1733 >= 1080 and 1300 >= 1300 -> covered (the width overflows the aperture and is clipped).

**First reached for:** the **stored** dimensions 3024x4032, which give `width 1080, height 1440` —
a portrait rect. Wrong. The orientation rule is one sentence at the bottom of the arithmetic section and
I only caught it on a second pass. This is the single easiest mistake in the whole vocabulary, and it is
a *silent* one: 1080x1440 is a perfectly well-formed `cover` rect for the wrong source, so validate can
only catch it if it reads EXIF itself.

**Hesitated:** "rotate 90 degrees clockwise for display" — does the *drawn rect* describe the source
before or after rotation? I took it as after (the rect is what is painted on screen), consistent with
"the source is resampled to exactly this rect".

**Wanted an error message saying:** `fit "cover": images/10.jpg has EXIF orientation 6; effective source
is 4032x3024 (not 3024x4032). Expected 1733x1300, got 1080x1440.`

---

## Task 6 — top third of `images/11.png` 1080x3000, aperture [0,0,1080,1000]

**Wrote:**

```json
{"id":"photo-11","type":"image","start":0,"end":6000,"source":"images/11.png",
 "x":0,"y":0,"origin":"top-left",
 "width":1080,"height":3000,"fit":"cover","clip":[0,0,1080,1000]}
```

**Arithmetic.** The source is 1080 wide — exactly the aperture width — and 3000 tall, exactly 3x the
aperture height. So "top third" needs no cropping field at all: draw the photo at natural size, anchored
top-left at the aperture's top-left, and let `clip` show the top 1000px.
Check it is legal `cover`: box 1080x1000, source 1080x3000. `1080*3000 = 3240000 >= 1000*1080 = 1080000`
-> width drives, `width = 1080` (box dim verbatim), `height = (3000*1080)//1080 = 3000` (exact, no
rounding). Coverage: 1080 >= 1080, 3000 >= 1000 -> rect contains clip. So `fit:"cover"` validates, and
the visible third is chosen purely by geometry: the rect spans y 0..3000, the clip admits y 0..1000.

**First reached for:** **`"crop":[0,0,1080,1000]`** — does not exist ("There is no ... `crop`").
Immediately after that, **`"gravity":"top"`** — also does not exist, though it is sitting in the shipped
fixture on every photo element. Both instincts were "name the part of the source I want"; the format has
no such concept, and you steer the visible region entirely through `x`/`y` against a static `clip`.
I also considered `fit:"literal"` as an escape hatch before noticing `cover` computes 3000 exactly.

**Hesitated:** the most, here. Nothing in the spec says "to choose which part of the source is visible,
offset x/y against the clip" — I had to infer it from "`clip` ... does not move or scale with the
element". Had the aperture been, say, `[0,200,1080,1000]` I would have had to set `y:200` to keep the
top third, and I am not confident I would have got that right without an example.

**Wanted an error message saying:** `crop: not a field. The visible region is (element rect) ∩ (clip);
move the element with x/y to change which part shows.`

---

## Verdict

**Usable as published? Marginally — yes for `cover`/`contain`/`literal` arithmetic, no for
"which part of the image shows".**

The arithmetic half is genuinely good: the driving-axis test is stated as an integer inequality, the
slack formula is explicit floor division, and the "driving axis takes the box dimension verbatim" rule
removes the usual off-by-one ambiguity. I got tasks 1, 2, 3, 4 right on the first pass with it.

**The single worst thing:** the spec defines `fit` as *documentation of arithmetic you already did*
("It does not change what the renderer draws") while giving you no vocabulary at all for the thing
authors actually want — *which part of the source survives the aperture*. Cropping is emergent from
`x`/`y` vs a static `clip`, and that inference is never spelled out. The proof that this is a real
failure and not my inattention: **the shipped fixture contains `"gravity":"top"` on all seven photo
elements**, a field the spec says does not exist. Somebody — author or older renderer — needed a word
for "show the top", could not find one, and invented it. I reached for `crop` and `gravity` myself in
task 6 before finding neither. A field that is documented only by its absence, in a "Not fields"
footnote, while live data still carries it, is a vocabulary that has not finished shipping.

Runner-up: `fit` is required and validated, yet changes nothing about the render. It is a checksum
wearing the name of a layout property, which is exactly why `fit:"none"` and `object-fit` habits leak in.

**One concrete change I would make:** add a worked "choosing the visible region" example to the `clip`
section — one tall source, one aperture *not* at the origin, showing top / centre / bottom framing as
three different `y` values against the same `clip`, with the arithmetic (`y = clip_y - (drawn_h - clip_h)
* k`). Two lines of prose and a three-row table would have eliminated every hesitation I recorded in
tasks 3 and 6, and would have told whoever wrote `gravity:"top"` what to write instead.
