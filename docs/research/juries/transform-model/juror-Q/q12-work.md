# Q12 Exercise Work — Clipping aperture

## Exercise 1: Verify the 612 px claim

From the sample project JSON, `photo-06`:
```json
{"id":"photo-06","type":"image","group":"item-06","start":17472,"end":30603,"source":"images/06.png","box":[0,0,1080,1300],"fit":"cover","align":"top"}
```

- Source: images/06.png, dimensions 1536×2720 (from the brief)
- Box: [0, 0, 1080, 1300] → x=0, y=0, width=1080, height=1300
- Fit: "cover"
- Align: "top"

**Cover scaling:**
The brief states: "Cover scales it to 1080x1912.5"

Let me verify: 
- Source: 1536×2720
- Box: 1080×1300
- Cover means: scale to fill the box, cropping any excess

To cover a 1080×1300 box from a 1536×2720 source:
- Scale factor: max(1080/1536, 1300/2720) = max(0.703125, 0.477941) = 0.703125
- Scaled source: 1536×0.703125=1080, 2720×0.703125=1912.5
- So the source becomes 1080×1912.5

**Alignment "top":**
The source (1080×1912.5) is placed with align="top" in the box (1080×1300):
- X: 1080 source = 1080 box, so centered → x_offset = 0
- Y: 1912.5 source in 1300 box with "top" align → top-aligned
  - The source is 1912.5 tall, the box is 1300 tall
  - Top-aligned means the top of the source aligns with the top of the box
  - The source overflows by: 1912.5 - 1300 = 612.5 px

**The claim: "the photo paints ~612 px over the background that belongs there"**

This is approximately correct. The source image overflows the box by about 612.5 pixels (or ~612 px) below the bottom of the box.

So the box was doing two jobs:
1. Placing the element (x, y position and size)
2. Clipping it (masking/aperture)

Currently, with box and no aperture, the image will render 612 px beyond the box boundary, painting over whatever is supposed to be behind it.

## Exercise 2: Can the aperture be expressed using settled properties?

Settled properties (from brief's "Settled" section):
- x, y, origin, scale, rotation, opacity
- No skew
- rotation unwrapped degrees
- keyframes: absolute times

The settled properties do NOT include:
- fit
- align
- box
- clip/crop

So fit, align, and box are NOT in the settled list. But they're described in issue #21.

The question: can an aperture (static frame-space clip rect) be expressed using only settled properties?

**Answer: No.**

An aperture (static frame-space clip rect) requires:
- A clipping/masking boundary that does NOT rotate or scale with the element
- The element's scale/rotation move the picture behind it
- The aperture stays put

The settled properties include transform properties (x, y, scale, rotation, origin) but NOT a clipping/masking boundary. There is no way to say "clip this element to a rectangle" using only x, y, scale, rotation, opacity.

Therefore, an aperture is genuinely inexpressible with settled properties alone. It requires a new field.

## Exercise 3: Read issue 22, does aperture fall naturally inside the effect model?

Issue #22: "Define the effect model and its closed vocabulary"

The issue asks:
1. How does an effect attach to an element?
2. Whether order matters
3. The v1 vocabulary itself
4. Text effects specifically
5. Parameter validation
6. What is deliberately absent

Candidates from issue #22: "opacity fade in/out, blur, drop shadow, text outline/stroke, text background box, colour filter, Ken Burns (which may instead be pure transform keyframes — decide whether it is an effect at all, since #21 may cover it entirely)."

**Does a static frame-space clip rect fall naturally inside this scope?**

Quote from issue #22:
> Effects are a **closed, named, parameterised vocabulary published in the schema** — not an ordered effect *stack* in the After Effects sense

And:
> The reference class named in #19 — CapCut/Premiere — makes effects and *text effects* first-class, and the requester named them explicitly as required.

The candidates listed are mostly visual effects: fade, blur, drop shadow, stroke, background box, colour filter.

A static aperture/clip rect is:
- NOT a visual effect in the traditional sense (it doesn't add something, it removes/masks)
- Differs in kind from other candidates (blur affects all pixels; a clip rect is geometric masking)
- Is more aligned with layout/positioning than with effects

**However**, it could reasonably be implemented as an effect (many systems do treat clipping as a "clip effect"). But issue #22 does NOT list it as a candidate, and it's being decided now for ADR-0011's transform/positioning model.

So aperture is NOT explicitly in #22's scope, but could fit. More importantly:
- **ADR-0011 (transform properties) is being finalized now**
- **Issue #22 (effects) is separate and later**
- **#12 in the brief says the aperture question is: "in the transform ADR now, or wait for #22?"**

The aperture is arguably both:
- Part of the **transform model** (how an element is positioned and sized, where the box currently does both)
- Part of the **effects model** (a mask/clip is a kind of visual effect)

But given that photo-06 is currently BROKEN without an aperture, and the transform ADR needs a solution, the question is timing: fix now or later?

## Exercise 4: Census of clipping needs in the fixture

Scanning the fixture JSON for elements that might need clipping:

**Visual elements with box:**
```
photo-05-intro: box=[0,0,1080,1300], fit=cover, align=top → overflows 612px
photo-05: box=[0,0,1080,1300], fit=cover, align=top → overflows 612px
photo-06: box=[0,0,1080,1300], fit=cover, align=top → overflows 612px
photo-07: box=[0,0,1080,1300], fit=cover, align=top → overflows 612px
photo-08: box=[0,0,1080,1300], fit=cover, align=top → overflows 612px
photo-05-quiz: box=[0,0,1080,1300], fit=cover, align=top → overflows 612px
photo-05-loop: box=[0,0,1080,1300], fit=cover, align=top → overflows 612px
```

All 7 photo elements have the same box, fit, align, and same source dimensions. **All 7 are clipped the same way.**

**Rect elements:**
```
card-05 through card-quiz: box=[48,1453,984,169], fill=... → no overflow, no clipping needed
chip-panel, flag-field, flag-bar-h, flag-bar-v: box with exact fit → no clipping
handle-panel, handle-logo: box with fit=cover → let me check
```

Looking at handle-logo:
```json
{"id":"handle-logo","type":"image","group":"header","start":0,"end":65216,"source":"brand/logo-en.png","box":[478,96,68,68],"fit":"cover","align":"center","mask":"circle"}
```

Wait, this has `"mask":"circle"`! That's a clipping/masking primitive. So the fixture DOES have masking for logo.

**Text elements:**
No text elements carry box with fit/align. Text elements have:
- size, line_height, x, y, align (horizontal alignment, not positional alignment)
- No box parameter in the fixture

**Summary:**
- 7 photo elements NEED clipping (all with the same cover+top setup, overflowing 612px)
- 1 image element (handle-logo) HAS a mask primitive ("circle")
- Text/rect elements: no clipping in fixture
- But ADR-0007 says text elements should have a "box" parameter for overflow checking

So the answer: **Photo elements need clipping. Text elements (per ADR-0007) might need it once the text model is settled. Rect elements generally don't.**

## Exercise 5: What breaks if ADR-0011 ships with no aperture, then #22 adds one later?

**If transform ADR ships now with no aperture and no way to express clipping:**

1. The fixture's 7 photo elements (and any similar projects) will silently render WRONG — images paint 612 px over background
2. Validation can compute the overflow (per ADR-0006's text overflow check) but cannot express it in the schema
3. Agents authoring projects will have no way to clip images, and no signal it's needed (the file is internally consistent)

**When #22 adds an aperture effect 3 months later:**

Files affected:
- The original fixture project and any copies will need retroactive `clip` or `mask` fields added
- Agents' existing workflows that worked around "no clipping" (placing photos carefully, using rects as masks) will suddenly have a new primitive

Decisions revisited:
- ADR-0011 might need an amendment: "Added aperture field to resolve photo clipping"
- The "settled" properties list might change: add clip/mask to positioning
- Validation rules change: add a "uncovered overflow" check for all elements, not just text
- ADR-0007's text overflow check gains a sibling: image/shape overflow check

**Concrete example:**
Currently in the fixture, `photo-06` paints 612px below its box. When aperture is added in #22:
```json
// before (now):
{"id":"photo-06","box":[0,0,1080,1300],"fit":"cover",...}
// renders wrong (612px overflow)

// after #22 adds aperture (3 months later):
{"id":"photo-06","box":[0,0,1080,1300],"fit":"cover","clip":[0,0,1080,1300],...}
// renders correct
// But old project files are invalid until patched
```

The cost is:
1. **Broken rendering** for months (silently wrong output)
2. **Retroactive schema changes** that force existing projects to update
3. **Lost opportunity** to validate and check for the problem now
4. **Agents' confusion** about why the new field exists (it wasn't in the requirements when they learned the format)

This is exactly the kind of split-decision that [ADR-0002](docs/adr/0002-inline-source-no-asset-table.md) talks about: *"a format whose editing model is exact-string matching against what you can see"* should not hide silent render bugs.
