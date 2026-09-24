# Worklog — C (fable, SPEC-declared)

Frame 1080x1920. All arithmetic integer-only per spec. Box for `fit` = `clip`'s w,h.

A note before the tasks: the fixture file's photo elements all carry `"gravity":"top"`,
but the spec's "Not fields" section says there is no `gravity` field. I followed the spec
and omitted it everywhere. This spec/fixture disagreement is the first thing a real consumer
would trip over.

## Task 1 — new photo, 1536x2720 covering [0,0,1080,1300]

**Wrote:**
```json
{"id":"photo-09","type":"image","start":30000,"end":36000,"source":"images/09.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":1912,
 "fit":"cover","clip":[0,0,1080,1300]}
```
Arithmetic: cover, width drives iff `bw*sh >= bh*sw` → `1080*2720 = 2937600 >= 1300*1536 = 1996800` → width drives.
Width = box w = 1080 verbatim. Slack: `(2720*1080)//1536 = 2937600//1536 = 1912` (exact value 1912.5, floor).
Checks: 1080>=1080, 1912>=1300; drawn rect at (0,0) 1080x1912 contains clip. Matches the other photos exactly.

**First reached for:** copying `"gravity":"top"` from the neighboring fixture elements — the spec says
that field does not exist. Dropped it.
**Hesitated:** whether the slack axis must be floor specifically; re-read "What validate reports" —
floor OR ceil is accepted, so 1912 and 1913 both pass. Wrote floor since the arithmetic rule itself says floor.
**Wanted an error message saying:** n/a.

## Task 2 — re-exported source, now 1536x2200

**Wrote (changed field only):**
```json
"height": 1546
```
(`width:1080`, `fit:"cover"`, `clip:[0,0,1080,1300]` unchanged.)
Arithmetic: `1080*2200 = 2376000 >= 1300*1536 = 1996800` → width still drives, width = 1080.
Slack: `(2200*1080)//1536 = 2376000//1536 = 1546` (exact 1546.875, floor; 1547 would also validate).
Aperture coverage: 1546 >= 1300, so clip is still fully covered.

**First reached for:** exactly this — recompute height. No wrong turn.
**Hesitated:** briefly wondered whether the old 1912 would still "work visually" and only the
validator complains — spec is clear the declared rect IS what is drawn (source resampled to it),
so keeping 1912 would stretch the shorter source. The fix is required for correctness, not just lint.
**Wanted an error message saying:** "fit:cover with source 1536x2200 in box 1080x1300 requires height 1546 (or 1547); found 1912."

## Task 3 — badge 1200x400 entirely visible in [840,1700,200,160]

**Wrote:**
```json
{"id":"badge","type":"image","start":30000,"end":36000,"source":"brand/badge.png",
 "x":840,"y":1747,"origin":"top-left","width":200,"height":66,
 "fit":"contain","clip":[840,1700,200,160]}
```
(Times not specified by the task; used the Task-1 window as a placeholder.)
Arithmetic: contain, width drives iff `bw*sh <= bh*sw` → `200*400 = 80000 <= 160*1200 = 192000` → width drives.
Width = 200 verbatim. Slack: `(400*200)//1200 = 80000//1200 = 66` (exact 66.67, floor).
Placement: no gravity/align exists, so vertical centering is done by hand:
`y = 1700 + (160-66)//2 = 1747`. Coverage check: clip [840,1700,200,160] contains drawn rect
(840,1747)-(1040,1813). Nothing cropped.

**First reached for:** `"gravity":"center"` (it's all over the fixture) — does not exist per spec.
Then briefly `align` — also listed as not a field. Centering must be baked into `y` manually.
**Hesitated:** whether `x`/`y` are the element position independent of clip (yes — clip is a static
rect in frame space that does not move with the element), so I had to place the drawn rect inside
the clip myself.
**Wanted an error message saying:** if I'd left y:1700 it would validate fine; no error teaches you
that centering is your own arithmetic. A doc line "there is no alignment; compute x/y yourself" exists
only implicitly via "Not fields".

## Task 4 — deliberate 1080x1600 squash of images/07.png

**Wrote (changed fields only, on `photo-07`):**
```json
"width": 1080, "height": 1600, "fit": "declared"
```
Everything else — including the `scale` keyframes — untouched. `clip:[0,0,1080,1300]` may stay
(it is optional under `declared`, not forbidden) and keeping it preserves the aperture.
Arithmetic: none — that is the point of `declared`. 1080x1600 is not a legal `cover` result
(cover would demand height 1912) and validate's rect-vs-rule and aperture-coverage errors are
"not reported under declared", so the wrong-on-purpose integers pass.

**First reached for:** `fit:"fill"` (CSS object-fit muscle memory) — not in the closed value set.
`declared` is the escape hatch.
**Hesitated:** whether `declared` also switches off the aperture-coverage error (the squashed rect
1080x1600 does still contain the clip here, but I checked) — spec says coverage errors are tied to
cover/contain, so yes, off.
**Wanted an error message saying:** if I'd written fit:cover with 1080x1600: "cover requires 1080x1912;
if the distortion is intentional use fit:declared."

## Task 5 — EXIF-rotated phone photo, 3024x4032 stored, orientation 6

**Wrote:** `"width": 1733, "height": 1300` (with `fit:"cover"`, `clip:[0,0,1080,1300]`).
Arithmetic: orientation 6 transposes → source dimensions for the rule are **4032x3024**.
Cover: width drives iff `1080*3024 = 3265920 >= 1300*4032 = 5241600` — false → height drives.
Height = box h = 1300 verbatim. Slack: `(4032*1300)//3024 = 5241600//3024 = 1733` (exact 1733.33, floor).
Checks: 1733>=1080, 1300>=1300; at (0,0) the drawn rect contains the clip.

**First reached for:** the stored dimensions 3024x4032 (which would give the Task-1-style 1080-wide
answer, width drives, 1080x1440 — wrong). The spec's orientation-applied rule caught it before I wrote it.
**Hesitated:** none after re-reading the EXIF paragraph; it is explicit.
**Wanted an error message saying:** "source is EXIF-rotated: rule dimensions are 4032x3024, not 3024x4032."
Without validate reading the file (UNCHECKED case), this mistake would silently validate as a plausible
cover of a portrait source — the scariest failure mode in the whole spec.

## Task 6 — top third of a 1080x3000 photo in [0,0,1080,1000]

**Wrote:**
```json
{"id":"photo-11","type":"image","start":30000,"end":36000,"source":"images/11.png",
 "x":0,"y":0,"origin":"top-left","width":1080,"height":3000,
 "fit":"cover","clip":[0,0,1080,1000]}
```
Draw the source at natural size (1080x3000) anchored at y:0; the clip's 1000-px-tall aperture shows
rows 0-999 — the top third of 3000. Arithmetic: this even validates as `cover`:
width drives (`1080*3000 = 3240000 >= 1000*1080 = 1080000`), width = 1080 = box w,
slack = `(3000*1080)//1080 = 3000` exactly. Coverage: 1080x3000 contains [0,0,1080,1000].
(`fit:"declared"` with the same integers would be equally legal; `cover` documents that the
aperture is guaranteed covered, so I kept it.)

**First reached for:** a `crop` field to take the source's top third — spec's "Not fields" says no
`crop`. The idiom is: draw big, position, let `clip` do the cropping.
**Hesitated:** whether cover's arithmetic *forces* a rect that exactly fills the box (making 3000 illegal
because cover's minimal fit of 1080x3000 into 1080x1000... ) — no: cover's formula for this source/box
literally yields 1080x3000, since the source is already box-width. Coincidence, but a happy one; had the
source been 1200x3000 I would have needed `declared` (or accepted cover's `1080x2700`, which shows the
top 37%, not the top third).
**Wanted an error message saying:** n/a — but note the "top third" intent is unrepresentable as a rule;
it only survives as unexplained integers plus a clip.

## Verdict

Usable, yes — the closed value set, integer-only arithmetic, and floor/ceil tolerance make every task
mechanically checkable, and `declared` is an honest escape hatch. The single worst thing: **the spec and
the shipped fixture contradict each other** — every photo element in
`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json` carries `"gravity":"top"`,
which the spec flatly says is not a field. A consumer copying the nearest working example (the normal
way anyone authors these files) immediately writes an illegal field, and there is no stated behavior for
unknown fields (rejected? ignored?). One concrete change: make `validate` reject (or at least warn on)
unknown fields, and scrub `gravity` from the fixture — or, if gravity is real, document it; either way
the example file and the reference must agree.

Runner-up: positioning-relative-to-aperture is entirely manual (no alignment vocabulary), which is fine
minimalism, but the spec should say so in one affirmative sentence instead of only via "Not fields".
