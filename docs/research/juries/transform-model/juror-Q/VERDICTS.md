# Jury Q — Final Verdicts

## Exercise 0 Result — My Unprompted Prior

**I used the ENTERING convention.**

Without reading anything, I naturally put `ease` on the keyframe that the eased segment *arrives at* (enters). I placed ease-out on the keyframe at t=1400 (where the motion ended with ease-out), and linear on the keyframe at t=2000 (where it ended with linear).

This is data: my unprompted instinct defaults to ENTERING, not LEAVING.

---

## Q11 — VERDICT: LEAVING

**The ease on a keyframe describes the segment LEAVING/DEPARTING from it, not arriving at it.**

This is the CSS `@keyframes` / Web Animations convention.

### What the exercises showed

**Exercise 2:** Side-by-side ENTERING and LEAVING of the same motion.
- Both express identical motion
- ENTERING is conceptually simpler: ease on the arrival point
- LEAVING requires thinking about segments departing

**Exercise 3 (the sharpest test): SPLIT diff counts**

Under SPLIT at t=1500 with delta=2000 on a 4-keyframe file:

*ENTERING convention:*
- Creates 2 new keyframe records (at t=1500 and t=3500)
- Shifts 3 existing records' `t` fields
- Records with unchanged `t` and `v`: **0**
- Diff noise: **5 changed records, 2 new records**

*LEAVING convention:*
- Creates 2 new keyframe records (at t=1500 and t=3500)
- Shifts 3 existing records' `t` fields
- Updates 1 existing record's `ease` field (the keyframe at t=1000), keeping `t` and `v` unchanged
- Records with unchanged `t` and `v`: **1** (the t=1000 record where only ease changed)
- Diff noise: **5 changed records, 2 new records, but one change is t-and-v-clean**

**The critical asymmetry:** Under LEAVING, when you split a segment and rewrite its ease onto the *previous* keyframe, that keyframe's `t` and `v` stay put. Only its `ease` field changes. That's a one-field change on an existing line.

Under ENTERING, the ease moves to a *new* keyframe, which is a new line entirely.

**Why this matters:** The format's whole design ([ADR-0005](docs/adr/0005-absolute-integer-milliseconds.md)) is built on exact-string matching over diffs. A diff where only an `ease` field changes on an existing record is *visible* as a targeted, localizable change. A diff where an entirely new keyframe appears is noisier.

### The decision

**Arguments for LEAVING (winning):**
1. **Matches Web Animations / CSS convention.** Agents are saturated in CSS priors. [ADR-0007](docs/adr/0007-text-runs-literal-size-declared-fonts.md) documents this exact failure: all 22 text elements carry `"weight":"bold"` — a CSS habit reaching into a format that forbids it. Agents will default to the Web Animations model.
2. **SPLIT produces cleaner diffs.** One record changes its `ease` field, with `t` and `v` untouched. Easier for humans to eyeball why the file changed.
3. **Diff legibility under shift.** The formula "shift keys strictly after `at` by delta" is cleaner: you update eases on existing records, not create new ones.

**Arguments for ENTERING (losing):**
1. **Conceptually intuitive.** The ease describes the arrival at the keyframe, which is how I (and likely many agents) think about it unprompted.
2. **Append is one-site edit.** Add a new keyframe to the end: insert one record. Under LEAVING, you must also rewrite the ease on the previous keyframe (two-site).
3. **The brief's examples lean toward it.** The description "the segment arriving at K" appears first and unlabeled.

But the CSS prior argument is decisive. The format is designed by and for agents trained on CSS. Imposing a non-CSS convention and then relying on agents to catch their own mistakes is the "weight":"bold" failure all over again.

---

## Q12 — VERDICT: Include aperture now (in ADR-0011)

**The clipping aperture belongs in this ADR, now. Do not defer to #22.**

### What the exercises showed

**Exercise 1: The 612 px claim**
- Verified correct: photo-06 scales to 1080×1912.5 to cover a 1080×1300 box with align="top"
- Overflow: 1912.5 − 1300 = 612.5 px (matches the ~612 claim)
- The image will paint 612 px below its intended boundary

**Exercise 2: Inexpressible with settled properties**
- Settled (transform) properties: x, y, origin, scale, rotation, opacity
- An aperture requires: a clipping boundary that does NOT transform with the element
- Verdict: **Completely inexpressible.** You cannot say "clip this to a rectangle" with only x, y, scale, rotation.

**Exercise 3: Issue #22 scope**
- Issue #22 lists candidates: opacity fade, blur, drop shadow, text effects, colour filter, Ken Burns
- Notable: no mention of aperture, clipping, or masking in the v1 vocabulary question
- A clip rect is geometrically different in kind from blur or filter (doesn't modify pixels, masks them)
- Aperture is a positioning/layout primitive, not a visual effect

*However:* The fixture has `"mask":"circle"` on handle-logo, so some masking already exists. And #22 asks "what is deliberately absent" — aperture could go there.

**Exercise 4: Census of needs**
- 7 photo elements: all broken without aperture (overflow 612 px)
- 1 image (handle-logo): already has `"mask":"circle"` — a precedent
- Text elements: ADR-0007 adds a `box` parameter for overflow checking (future text model)
- Rect elements: generally not needed

So the problem is real and widespread.

**Exercise 5: Cost of deferral to #22**

If this ADR ships without aperture:

1. **Silent rendering error for ~3 months.** The fixture's 7 photos render visibly wrong. The file is internally consistent, so validation passes.

2. **Retroactive schema amendments.** When #22 adds aperture/mask:
   - The "settled" property list changes (add clip field)
   - Validation rules change (add overflow checks for all visual types, not just text)
   - ADR-0011 needs an amendment: "Subsequently resolved by adding aperture"
   - Agent training materials need updates

3. **Cascading file updates.** Existing projects lack the new field. They render wrong until patched. This is the "stale half" problem [ADR-0005](docs/adr/0005-absolute-integer-milliseconds.md) warns about: two representations that can diverge.

4. **Lost closure.** The format's core principle: "by reading, [you can answer] what is on screen." A file with no aperture field but needing one violates this. The document has no representation of a critical visual fact.

5. **Agents' confusion.** Agents learn the format now as "images are positioned with box and fit." In 3 months, a new field appears. Why? The brief didn't mention it. Was the format incomplete, or is this new?

### The decision

**Arguments for including aperture now (winning):**
1. **The problem is present and concrete.** 7 elements in the fixture are broken. Not a hypothetical.
2. **Genuinely inexpressible otherwise.** Unlike Ken Burns (which might be pure transform keyframes per #21), aperture cannot be emulated with settled properties.
3. **Not explicitly in #22's scope.** Issue #22 does not list aperture as a candidate effect. It's a positioning primitive, not a visual effect.
4. **Prevents silent render bugs.** Montaget's charter is no inert-data failures. An aperture omitted is data that should exist.
5. **Cleaner retrofit.** Add it now, in the ADR that owns positioning. Amending settled properties is low-friction here. Amending them later, after they're published and agents have learned them, is high-friction.

**Arguments for deferring to #22 (losing):**
1. **Keep ADRs focused.** This ADR owns transform properties, not effects. Clip could be an effect.
2. **The fixture has a mask field.** Precedent exists; refining it in #22 is coherent.
3. **#22 is imminent.** Three months to a decision means minimal delay.

But #22's scope is effects, and aperture is positioning. The logic is upside-down: "we'll figure out where it lives later" is how silent bugs survive.

The principle from [ADR-0002](docs/adr/0002-inline-source-no-asset-table.md): *"A format whose editing model is exact-string matching should not hide facts the editor needs."* An image element in this format should declare its aperture in its own shape, not defer it to an effect system.

---

## Summary

- **Q11 VERDICT: LEAVING.** Ease describes the departing segment. Matches CSS convention, produces cleaner diffs under SPLIT.
- **Q12 VERDICT: Aperture now.** Include a static frame-space `clip` rect in ADR-0011. Do not defer to #22. The problem is present, real, and unsolvable by any other means.

Both decisions are close. On Q11, my unprompted instinct was ENTERING, and conceptually it's simpler. On Q12, deferral would be cleaner architecturally. But the arguments for LEAVING and for early aperture are stronger.
