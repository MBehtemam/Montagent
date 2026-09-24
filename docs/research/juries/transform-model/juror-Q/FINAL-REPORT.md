# Jury Q Final Report

## VERDICT

**Q11: LEAVING** — The ease on a keyframe describes the segment departing from it, toward the next. This is the CSS @keyframes and Web Animations convention.

**Q12: Aperture now** — Include a static frame-space `clip` rect field in ADR-0011. Do not defer to #22. The problem is present, concrete, and unsolvable by any other means.

---

## Exercise 0 — Prior Instinct

I used the **ENTERING convention** unprompted. I placed ease on the keyframe that the eased segment enters/arrives at. This is my baseline prior, stated before reading anything else.

---

## What the Work Showed

### Q11: The SPLIT Test

The decisive exercise was comparing how ENTERING and LEAVING behave under SPLIT when shifting a keyframe list at an interior point (delta=2000, at=1500).

Under SPLIT, when you split a segment [1000, 2000] with ease-out at the midpoint t=1500, you get two bezier halves. The question is which keyframe each lands on.

**ENTERING results:**
- Inserts a new keyframe at t=1500 with the left-half ease
- Shifts the original keyframe at t=2000 to t=4000 and updates its ease to the right half
- Diff: 5 changed records (t values shifted), 2 new records, **0 records with unchanged t and v**

**LEAVING results:**
- Inserts a new keyframe at t=1500 with the left-half ease  
- Shifts the original keyframe at t=2000 to t=4000 (t changes)
- Updates the keyframe at t=1000 to have the left-half ease (only its ease field changes; t and v stay put)
- Diff: 5 changed records, 2 new records, **1 record with unchanged t and v** (the t=1000 record, ease-only change)

This matters because [ADR-0005](docs/adr/0005-absolute-integer-milliseconds.md) makes the whole format rest on "exact-string matching over diff a human eyeballs." A change that affects only one field on an existing record is far more legible than a new record entirely. LEAVING produces cleaner diffs.

**Why LEAVING still wins despite my ENTERING instinct:**
- Matches CSS / Web Animations convention
- Agents carry CSS priors (evidenced by the "weight":"bold" field appearing on all 22 text elements despite the schema forbidding it, per ADR-0007)
- Diff legibility under shift: SPLIT changes only the ease field on existing keyframes
- The risk of getting this backwards and silent rendering wrong (because agents default to CSS) exceeds the benefit of one-site appends

The decision is close, but the prior argument is decisive.

### Q12: The Aperture Necessity

**The 612px claim:** Verified. photo-06 (1536×2720 source) scaled to cover a 1080×1300 box with align="top" produces an 1080×1912.5 scaled image, overflowing 612.5 px below the box.

**Inexpressibility:** An aperture (static frame-space clip rect) is completely inexpressible with settled transform properties alone (x, y, origin, scale, rotation, opacity). You cannot express "clip to a rectangle" without a clipping boundary.

**Scope:** Issue #22 (effect model) does not list aperture among candidates. Its vocabulary is visual effects (blur, shadow, filter). An aperture is a geometric masking boundary, more aligned with positioning than effects. Precedent: handle-logo has `"mask":"circle"`, showing masking is already in the fixture.

**The cost of deferral:**

If this ADR ships without aperture, three months later when #22 adds one:

1. The fixture's 7 photos render silently wrong for months (612 px overflow)
2. The schema acquires a retroactive field, creating a stale-half problem (projects need amendment)
3. Validation rules change (new overflow checks for all visual types, not just text)
4. Agents' training needs updating (the settled property list expanded mid-learning)
5. ADR-0011 needs an amendment note

The format's core principle is: *by reading the document, you can answer what is on screen.* A file with no aperture field, needing one, violates this. The visual fact has no representation.

**Why including it now wins:**
- Problem is present and concrete (not hypothetical)
- Inexpressible by other means (unlike Ken Burns, which might be pure keyframes)
- Not explicitly in #22's scope
- Prevents silent render bugs (Montagent's core principle)
- Avoids retroactive schema changes

---

## Files in Jury Q Scratch

- `ex0-prior.md` — My Exercise 0 answer (ENTERING, stated first)
- `q11-work.md` — Full working through Exercises 1–3 for Q11
- `q12-work.md` — Full working through Exercises 1–5 for Q12
- `VERDICTS.md` — Detailed verdict with all arguments
- `FINAL-REPORT.md` — This file

---

## Both Decisions Are Close

**Q11:** My unprompted instinct was ENTERING (conceptually simpler, keyframe receives the ease of the arriving segment). But LEAVING's CSS alignment and cleaner diffs under SPLIT outweigh the appeal.

**Q12:** Deferring aperture to #22 would keep this ADR's scope tight (positioning only). But the silent rendering bug, the concrete presence of the problem, and the clarity of having the aperture in the element that needs it outweigh architectural tidiness.

In both cases, the arguments for LEAVING and for early aperture are stronger, but not overwhelming.
