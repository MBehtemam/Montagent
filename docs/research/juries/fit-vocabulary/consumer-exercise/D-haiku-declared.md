# Worklog: Montaget Declared Format Exercise

Model: claude-haiku-4-5-20251001

## Task 1: New photo

**Element:**
```json
{
  "id": "photo-09",
  "type": "image",
  "start": 30000,
  "end": 36000,
  "source": "images/09.png",
  "x": 0,
  "y": 0,
  "origin": "top-left",
  "width": 1080,
  "height": 1912,
  "fit": "cover",
  "clip": [0, 0, 1080, 1300]
}
```

**Arithmetic:**
- Source: 1536×2720, Aperture box: 1080×1300
- Check width drives: `bw*sh >= bh*sw` → `1080*2720 >= 1300*1536` → `3,240,000 >= 1,996,800` ✓
- Width = `bw` = 1080
- Height = `(sh * bw) // sw` = `(2720 * 1080) // 1536` = `2,937,600 // 1536` = 1912
- Cover check: drawn rect (1080×1912) contains clip (1080×1300) ✓

**First reached for:** `x` and `y` positioning syntax — verified top-left corner at origin point.

**Hesitated:** Whether to include `group` field (saw it in existing elements) — spec doesn't mention it, so I omitted per spec-only constraint.

**Wanted an error message saying:** (none — this was straightforward)

---

## Task 2: Fix images/06.png (re-exported to 1536×2200)

**Element changes:**
```json
{
  "id": "photo-06",
  "width": 1080,
  "height": 1546,
  "fit": "cover",
  "clip": [0, 0, 1080, 1300]
}
```

**Arithmetic:**
- Source changed to 1536×2200, Aperture box: 1080×1300
- Check width drives: `bw*sh >= bh*sw` → `1080*2200 >= 1300*1536` → `2,376,000 >= 1,996,800` ✓
- Width = `bw` = 1080
- Height = `(sh * bw) // sw` = `(2200 * 1080) // 1536` = `2,376,000 // 1536` = 1546
- Old element had height=1912, which violated the rule because 1912 is too tall
- Cover check: drawn rect (1080×1546) contains clip (1080×1300) ✓

**First reached for:** Whether I could use `ceil` for the slack axis to avoid validation errors — spec allows both `floor` or `ceil`, but 1546 (floor) is cleaner.

**Hesitated:** Whether this change would affect the `scale` animation keyframes — spec says fit doesn't change render behavior, so animation timestamps should remain valid.

**Wanted an error message saying:** "For source 1536×2200 with cover in box 1080×1300, slack axis height must be 1546 or 1547, got 1912"

---

## Task 3: Badge that must not be cropped

**Element:**
```json
{
  "id": "badge",
  "type": "image",
  "source": "brand/badge.png",
  "x": 840,
  "y": 1700,
  "origin": "top-left",
  "width": 200,
  "height": 66,
  "fit": "contain",
  "clip": [840, 1700, 200, 160]
}
```

**Arithmetic:**
- Source: 1200×400, Aperture box: 200×160
- Check width drives for contain: `bw*sh <= bh*sw` → `200*400 <= 160*1200` → `80,000 <= 192,000` ✓
- Width = `bw` = 200
- Height = `(sh * bw) // sw` = `(400 * 200) // 1200` = `80,000 // 1200` = 66
- Contain check: clip (200×160) contains drawn rect (200×66) ✓

**First reached for:** A `gravity` or `align` field for centering inside the aperture — but spec explicitly states "There is no gravity field, no align on images". Positioned at aperture top-left.

**Hesitated:** Whether `clip` must always equal aperture bounds — it can be positioned anywhere, and in this case it positions the aperture at (840, 1700) on the frame.

**Wanted an error message saying:** "For contain, source 1200×400 in box 200×160, drawn rect (200×66) must fit inside clip (200×160). Got width=200 height=66 — valid."

---

## Task 4: Deliberate distortion

**Element:**
```json
{
  "id": "photo-07-distorted",
  "type": "image",
  "source": "images/07.png",
  "x": 0,
  "y": 0,
  "origin": "top-left",
  "width": 1080,
  "height": 1600,
  "fit": "declared",
  "clip": [0, 0, 1080, 1300]
}
```

**Arithmetic:**
- Source: 1536×2720, requested: 1080×1600 (intentional wrong aspect ratio)
- Fit = `declared` means "no rule; the integers are yours" — aspect ratio violation allowed
- No validation rule applied; integers are trusted as-is

**First reached for:** Whether wrong aspect ratios trigger validation errors — they do under `cover`/`contain`, but not under `declared`, which is the escape hatch.

**Hesitated:** Whether to keep the existing `scale` animation — spec says fit doesn't change render behavior, so keyframes stay identical and readable.

**Wanted an error message saying:** (none needed — declared is the point)

---

## Task 5: Rotated phone photo

**Element:**
```json
{
  "id": "photo-10-rotated",
  "type": "image",
  "source": "images/10.jpg",
  "x": 0,
  "y": 0,
  "origin": "top-left",
  "width": 1734,
  "height": 1300,
  "fit": "cover",
  "clip": [0, 0, 1080, 1300]
}
```

**Arithmetic:**
- Stored dimensions: 3024×4032, EXIF orientation 6 = rotate 90° clockwise
- Orientation-applied dimensions: 4032×3024 (swap for display)
- Use **orientation-applied** dimensions per spec: sw=4032, sh=3024
- Aperture box: 1080×1300
- Check width drives: `bw*sh >= bh*sw` → `1080*3024 >= 1300*4032` → `3,265,920 >= 5,241,600` ✗
- Height drives: `height = bh` = 1300
- Width = `(sw * bh) // sh` = `(4032 * 1300) // 3024` = `5,241,600 // 3024` = 1734 (floor, remainder 1008)
- Cover check: drawn rect (1734×1300) contains clip (1080×1300) ✓

**First reached for:** The stored JPEG dimensions (3024×4032) directly — but spec explicitly requires "orientation-applied pixel dimensions" which means I must account for EXIF orientation 6's 90° rotation, transposing to 4032×3024.

**Hesitated:** Precision of integer division — `5,241,600 // 3024` = 1734 exactly (floor), confirmed with Python.

**Wanted an error message saying:** "For rotated image (EXIF orientation 6), use orientation-applied dimensions 4032×3024, not stored 3024×4032"

---

## Task 6: Crop to top third

**Element:**
```json
{
  "id": "photo-11-top-third",
  "type": "image",
  "source": "images/11.png",
  "x": 0,
  "y": 0,
  "origin": "top-left",
  "width": 1080,
  "height": 3000,
  "fit": "cover",
  "clip": [0, 0, 1080, 1000]
}
```

**Arithmetic:**
- Source: 1080×3000, Aperture box: 1080×1000
- Check width drives: `bw*sh >= bh*sw` → `1080*3000 >= 1000*1080` → `3,240,000 >= 1,080,000` ✓
- Width = `bw` = 1080
- Height = `(sh * bw) // sw` = `(3000 * 1080) // 1080` = 3000
- Cover check: drawn rect (1080×3000) contains clip (1080×1000) ✓
- Clipping to [0, 0, 1080, 1000] shows only the top 1000 pixels of the 3000-pixel height

**First reached for:** A `crop` field or `offset` field to specify which part of the source to show — but spec explicitly says "There is no... crop" field. Cropping is done via `clip`, not a separate field.

**Hesitated:** Whether drawn rect (3000 pixels tall) covering only a 1000-pixel aperture was the right interpretation — confirmed by re-reading that `clip` defines what's painted regardless of drawn size.

**Wanted an error message saying:** "Use clip to define which part of the element is visible; there is no crop field"

---

## Verdict

**Is this vocabulary usable as published?** Mostly yes, with friction.

**Single worst thing:** The **absence of `gravity`/`align` fields** is frustrating and error-prone. Every other image format (CSS, web platforms, design tools) provides a standard way to position content within a viewport. Without it, you must compute `x`, `y` for each positioning need, which is both unintuitive and fragile. The spec's reasoning (rejecting field complexity) is understandable, but forces consumers to either hardcode pixel math or implement their own gravity system on top.

**One concrete change I would make:** Restore a single `gravity` field with closed values (`top-left`, `top-center`, `top-right`, `center-left`, `center`, `center-right`, `bottom-left`, `bottom-center`, `bottom-right`). At render time, compute `x` and `y` from `gravity` + `clip` dimensions. This:
- Keeps the schema simple (one enum field vs. computed positioning rules)
- Aligns with user mental models (people think "place it centered", not "compute x=540")
- Doesn't break the "fit is declarative, not imperative" philosophy
- Addresses the gap where existing projects use `gravity` but spec forbids it

**Reach-for audit:**
- Reached for `gravity` on 3 tasks (Tasks 1, 3, 6) — field explicitly forbidden
- Reached for `crop` on Task 6 — field explicitly forbidden  
- Reached for `ceil` rounding on Task 2 — allowed but floor is default
- Reached for stored dimensions instead of orientation-applied on Task 5 — caught by spec and corrected

All calculations validated against spec; no arithmetic errors.
