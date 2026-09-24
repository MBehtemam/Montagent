# Worklog: Haiku model authoring Montagent elements

## Task 1: New photo

**Wrote:**
```json
{
  "id":"photo-09",
  "type":"image",
  "start":30000,
  "end":36000,
  "source":"images/09.png",
  "x":0,
  "y":0,
  "origin":"top-left",
  "width":1080,
  "height":1912,
  "fit":"cover",
  "clip":[0,0,1080,1300]
}
```

**Arithmetic:**
- Source: 1536x2720
- Aperture box (from clip): 1080x1300
- For cover fit: width drives when `bw*sh >= bh*sw`
- Check: 1080×2720 ≥ 1300×1536? → 2,937,600 ≥ 1,996,800? → YES
- Width drives: width = 1080
- Height = (2720 × 1080) ÷ 1536 = 1912.5 → floor = 1912

**First reached for:** None - straightforward, matches existing pattern in fixture

**Hesitated:** None

---

## Task 2: Re-exported source — fix validation error

**Wrote:**
```json
{
  "id":"photo-06",
  "type":"image",
  "start":17472,
  "end":30603,
  "source":"images/06.png",
  "x":0,
  "y":0,
  "origin":"top-left",
  "width":1080,
  "height":1547,
  "fit":"cover",
  "clip":[0,0,1080,1300]
}
```

**Arithmetic:**
- Old source: 1536x2720 → new source: 1536x2200
- Aperture box: 1080x1300
- For cover fit: width drives when `bw*sh >= bh*sw`
- Check: 1080×2200 ≥ 1300×1536? → 2,376,000 ≥ 1,996,800? → YES
- Width drives: width = 1080
- Height exact = (2200 × 1080) ÷ 1536 = 1546.875
- floor = 1546, ceil = 1547
- Chose 1547 (ceil) to ensure robust coverage of [0,0,1080,1300]

**First reached for:** "precision" or "rounding mode" — the spec allows both floor and ceil, chose ceil for safety

**Hesitated:** Whether floor or ceil is preferable for coverage. Spec says "must be floor or ceil of the exact value" but doesn't mandate which. Chose ceil to ensure strict coverage.

---

## Task 3: Badge that must not be cropped

**Wrote:**
```json
{
  "id":"badge",
  "type":"image",
  "x":840,
  "y":1700,
  "origin":"top-left",
  "width":200,
  "height":67,
  "fit":"contain",
  "clip":[840,1700,200,160]
}
```

**Arithmetic:**
- Source: 1200x400
- Aperture box (from clip dimensions): 200x160
- For contain fit: width drives when `bw*sh <= bh*sw`
- Check: 200×400 ≤ 160×1200? → 80,000 ≤ 192,000? → YES
- Width drives: width = 200
- Height exact = (400 × 200) ÷ 1200 = 66.667
- floor = 66, ceil = 67
- Chose 67 (ceil) to maximize badge visibility while staying within the 200x160 aperture

**First reached for:** `gravity` or `align` to position within aperture — but spec says "There is no `gravity` field, no `align` on images". Used x/y positioning instead; centered badge within the 200-pixel-wide aperture.

**Hesitated:** Badge positioning—the spec doesn't have explicit positioning rules. Used x:840, y:1700 (aperture origin) to align the drawn rect with the aperture bounds. This places the 200x67 drawn rect at the aperture origin, fully contained within [840,1700,200,160].

---

## Task 4: Deliberate distortion with animation

**Wrote:**
```json
{
  "id":"photo-07",
  "type":"image",
  "start":30603,
  "end":42763,
  "source":"images/07.png",
  "x":0,
  "y":0,
  "origin":"top-left",
  "width":1080,
  "height":1600,
  "fit":"literal",
  "clip":[0,0,1080,1300],
  "scale":[{"t":30603,"v":[1.0,1.0]},{"t":45603,"v":[1.08,1.08],"ease":"linear"}]
}
```

**Arithmetic:**
- Source: 1536x2720
- Forced to: 1080x1600 (art director's explicit request)
- fit: "literal" (no rule applied; the declared dimensions are authored as-is)
- Kept scale animation untouched

**First reached for:** Whether fit:literal is a valid value — spec explicitly lists "cover", "contain", and "literal" as the closed value set. ✓

**Hesitated:** None—spec is clear that literal is for "no rule; the integers are yours"

---

## Task 5: Rotated phone photo with EXIF orientation

**Wrote:**
```json
{
  "id":"photo-10",
  "type":"image",
  "source":"images/10.jpg",
  "x":0,
  "y":0,
  "origin":"top-left",
  "width":1733,
  "height":1300,
  "fit":"cover",
  "clip":[0,0,1080,1300]
}
```

**Arithmetic:**
- Stored pixel dimensions: 3024x4032
- EXIF orientation 6 (rotate 90° clockwise for display) swaps dimensions
- **Display dimensions after orientation: 4032x3024** (swapped)
- Aperture box: 1080x1300
- For cover fit: width drives when `bw*sh >= bh*sw`
- Check: 1080×3024 ≥ 1300×4032? → 3,265,920 ≥ 5,241,600? → NO
- Height drives: height = 1300
- Width exact = (4032 × 1300) ÷ 3024 = 1733.333
- floor = 1733, ceil = 1734
- Chose 1733 (floor) per cover rule (drawn rect covers the box)

**First reached for:** Whether to use stored or display dimensions for EXIF. Spec says "Source dimensions are the image's **orientation-applied** pixel dimensions — if a JPEG carries an EXIF orientation flag that transposes the image, use the transposed dimensions." ✓

**Hesitated:** Confirmation that orientation 6 = rotate 90° CW (transposing). Verified against EXIF standard: orientation 6 is indeed rotate 90° clockwise, so swap w↔h.

---

## Task 6: Crop to top third of tall photo

**Wrote:**
```json
{
  "id":"photo-11",
  "type":"image",
  "source":"images/11.png",
  "x":0,
  "y":0,
  "origin":"top-left",
  "width":1080,
  "height":3000,
  "fit":"cover",
  "clip":[0,0,1080,1000]
}
```

**Arithmetic:**
- Source: 1080x3000
- Aperture box (from clip dimensions): 1080x1000
- For cover fit: width drives when `bw*sh >= bh*sw`
- Check: 1080×3000 ≥ 1000×1080? → 3,240,000 ≥ 1,080,000? → YES
- Width drives: width = 1080
- Height = (3000 × 1080) ÷ 1080 = 3000
- Clip [0,0,1080,1000] masks to show only the top 1000 pixels
- This preserves the top third (1/3 of 3000 = 1000) without scaling/resampling the full image

**First reached for:** `crop` field — but spec explicitly says "There is no `gravity` field, no `align` on images, no `crop`, and no `box`." Used clip instead to achieve the same effect.

**Hesitated:** Whether to resample to 1080x1000 (which would squash the full image) vs. keeping full height and clipping. Chose full height + clip approach to show the top portion at natural scale, not squashed.

---

## Verdict

**Is this vocabulary usable as published?**

**Yes, with one caveat:** The vocabulary is publishable and mostly clear. However, there are edge cases in sizing calculations.

**Single worst thing:** The lack of explicit guidance on **floor vs. ceil ambiguity in cover/contain calculations**. When the exact driving-axis height/width lands on a .5 boundary (e.g., 1546.875), the spec says "floor or ceil" are both valid per validation, but provides no guidance on which to prefer. This creates silent ambiguity in production—different systems might choose differently without either being "wrong."

**Concrete change I would make:** Add a tiebreaker rule to the spec:
- For `cover` fits: prefer **ceil** (ensures complete coverage even at sub-pixel offsets)
- For `contain` fits: prefer **floor** (ensures strict containment without overflow)

This would make authored dimensions deterministic across implementations.

---

## Summary of reaches

- ✓ All fit values (cover, contain, literal) are real and in spec
- ✗ `gravity` and `align` on images are NOT in spec (attempted mental model before reading)
- ✗ `crop` field does NOT exist (attempted mental model before reading)
- ✓ EXIF orientation handling follows standard (orientation 6 = rotate 90° CW)
- ⚠ floor/ceil choice in cover/contain is ambiguous when exact value is non-integer

