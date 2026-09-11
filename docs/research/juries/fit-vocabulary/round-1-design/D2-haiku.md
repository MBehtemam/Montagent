# Jury D2: fit vocabulary and omission — Haiku

## Q1. Is `fit` a derivation claim, or a render instruction?

**Decision:** Both. `fit` is a **render instruction** telling the renderer which scaling rule to apply, AND a **derivation claim** stating how the author computed the declared width/height.

**Why:** ADR-0013 is explicit: "The declared rect is authoritative at render — the source is resampled to exactly the declared rect." The actual consumer is the renderer, which must know whether to use cover (source fills box) or contain (source fits inside box) semantics to resample the source to the declared dimensions. The secondary consumer is validate, which uses fit to check whether the declared rect agrees with what the rule would compute from the source on disk. Q6's entire existence proves fit is a claim the validator checks, not just a render parameter. Fit is a layout mode (it's an assertion about which axis drives) and also an assertion about how the extents were authored.

**Strongest counter:** The renderer could infer fit from the relationship between source size, declared size, and aperture size without it being an explicit field — it could simply resample the source to the declared rect isotropically. This would make fit purely a validate concern, not a render instruction. But that reading loses the **intent** of the scaling mode and makes validation non-falsifiable: if two authors produce different declared rects for the same source and aperture, nothing says which one is right.

---

## Q2. What box does the fit rule fit into?

**Decision:** The **clip aperture** (the width and height of the `clip` rectangle). For photo-06: 1080×1300 (from clip `[0,0,1080,1300]`). For handle-logo: 68×68 (from clip `[478,96,68,68]`).

**Why:** The fit_rounding_scan.py script takes aperture dimensions (bw, bh) directly as parameters to cover_int, and the fixture proves it by example: photo-06 has clip `[0,0,1080,1300]` and when fit:"cover" is applied to source 1536×2720 with aperture 1080×1300, the rule computes (1080, 2720\*1080//1536) = (1080, 1912), which exactly matches the declared width/height. This is not a coincidence — the declared rect is authoritative, so it reflects the aperture constraint. Q6's deviation check (comparing declared rect to rule-computed rect from the source on disk) is meaningful only if the box is the aperture, not the declared rect (which would be circular). When clip is absent (it's optional), there is no aperture constraint, so fit cannot be meaningfully applied.

**Strongest counter:** The box could be the declared width/height rather than the clip. Under that reading, fit would be a *scaling mode* that tells the renderer how to resample (cover vs contain) without forcing the author to compute the exact extent. But then fit is not a derivation claim; it's only a render hint. Moreover, this breaks Q1's validation story — if the declared rect is given rather than computed, there's nothing for validate to check fit against. And it would make the fixture's 1912 value unexplainable, because there's no box to fit into; the value would be magic.

---

## Q3. Is `fit` required on an element, or may it be omitted?

**Decision:** May be omitted. An omitted `fit` means "no claim made about how the declared rect was computed."

**Why:** Following ADR-0014's principle (citing ADR-0006's opt-in rule): "An omitted height is indistinguishable from a decision not to check." Parallel application: an omitted fit is indistinguishable from a decision not to check for fit deviations. Unlike text height — where omission disables a critical overflow check and drifts the file — fit is **not** checking the integrity of the element itself; it's checking whether the author's claimed methodology matches the source on disk. An author who omits fit simply does not make that claim; the rendered video is unaffected (the declared rect is authoritative either way). ADR-0013 explicitly lists fit as open: "ADR-0012's defaults list does not include `fit`, nothing requires it." The ADR-0012 defaults (x, y, origin, scale, rotation, opacity) are all fields the renderer must have. Fit is a validation/intent field, not a render field.

**Strongest counter:** The fixture has fit on all 8 elements, suggesting it should be required. But ADR-0003's asymmetry forbids this inference: the channel is evidence a capability is **needed**, never that one is **unneeded**. All 8 real images having fit proves fit must work, not that it must be obligatory. An omitted fit would be flagged as "does not check fit deviations," same as ADR-0013's UNCHECKED category for unprobeable sources; that's informational, not an error.

---

## Q4. What is the closed value set, and how is "do not fit this, use my rect" spelled?

**Decision:** 
- **V1 set:** `{"cover"}` only.
- **Escape value:** Recommend `"exact"` (meaning: source is resampled to declared rect exactly as given, no fit rule applied; the author is making no claim about derived extents).
- **Do not ship `contain` in v1** (as ADR-0013 deliberately scopes).

For each value, the inequality it preserves:

- **`cover`** (source fills the box): `s*f >= b` on both axes. Rounding: floor on slack axis (floor preserves `(s_slack * b_driving) // s_driving >= b_slack`).
- **`exact`** (escape): no inequality; author accepts the declared rect as-is.
- **Postponed `contain`**: (source fits inside the box): `s*f <= b`. Rounding: ceil on slack axis (ceil preserves `ceil(s_slack * b_driving / s_driving) <= b_slack`).

**Why:** ADR-0013 is explicit in refusing to legislate contain: *"it would create a schema value by implication."* Defining contain's rounding rule now would lock both values into permanent coupling, making future migration hard. The escape value is critical: without it, Q6's error check cannot ship (per ADR-0013's "Not settled here"). The issue notes that `"none"` collides with CSS `object-fit` semantics. `"exact"` is unambiguous, non-CSS, and means "author asserts the declared rect without a fit derivation." The fixture's current workaround (rect at rule value + distortion in scale) fails on 7 of 8 elements because scale animates; an escape spelling lets those elements declare: `"fit":"exact", "scale":[1.0, 1.236740]` instead of baking distortion into keyframes.

**Strongest counter:** Ship `contain` now to avoid later migration. The semantics are obvious (inside instead of fill) and the rounding rule is defensible (ceil to preserve the inequality). However, ADR-0013's refusal is load-bearing: the ADR explicitly chose floor for cover as a tiebreak between two equally geometrically sound options, not as a logical consequence. Extending that tiebreak to contain would re-derive the same choice for a different value, which is exactly the "creation by implication" the ADR forbids. Shipping both requires a decision ADR-0013 punted.

---

## Q5. What happens to `gravity`?

**Decision:** Retire `gravity` on images. Make it a **schema error** on images, with the message: "gravity is inert; the clip aperture specifies which part of the source survives. Remove this field."

**Why:** ADR-0013 measured it inert on **8 of 8** image elements in the only real project file. All seven photos declare `"gravity":"top"`, yet all produce identical clipped output `[0,0,1080,1300]` regardless. The handle-logo declares `"gravity":"center"` but the clip `[478,96,68,68]` already specifies the frame-space position exactly. The aperture is static and frame-space (per ADR-0012); it does not move with scale or rotation. Gravity's definition is *which part of the source survives a crop*, but clip **is** that specification, stated in frame coordinates instead of source coordinates. Keeping gravity creates a second, redundant way to state the same constraint, which ADR-0012 rejected for `box` (a field that meant two things). The field is not just unused; it is misleading — an agent reading it would believe it controls cropping when it does not.

**Strongest counter:** Keep gravity as an optional **informational field** (deprecated). It documents author intent even though it is not rendered, and deferring the retirement avoids a breaking schema change. However, ADR-0014 precedent cuts the other way: it explicitly makes gravity on **text or shape** a schema error (not optional, not silently ignored). The principle is consistent: a field that claims to do something but does not is worse than no field. Recorded residual from ADR-0012: "Placement drift is not fixed here and needs its own ticket." If gravity were ever given a *new* job (e.g., default clip position when clip is absent), that job belongs to a new field with a new name, not to a resurrected gravity.

---

## Q6. The severity and shape of the fit-deviation check.

**Decision:**
- **Severity:** `error` (blocking, once fit vocabulary is defined).
- **Shape:** Predicate B: **"membership in {floor(exact), ceil(exact)} on the slack axis, with zero tolerance on the driving axis."**
  - Formal: For the slack axis, declared value must equal either floor(exact) or ceil(exact). For the driving axis (the one assigned the box dimension verbatim), declared value must equal the box dimension exactly (zero deviation).
- **Breaks on fixture:** **0 of 8** elements.

Verification on committed fixture:
- photo-06 (and all 7 photos): exact = 1912.5, slack axis (height) declared = 1912 = floor(1912.5) ✓, driving axis (width) declared = 1080 = box = 1080 ✓ → PASS
- handle-logo: exact = 68.0, slack axis (height) declared = 68 = floor(68.0) ✓, driving axis (width) declared = 68 = box = 68 ✓ → PASS

**Why:** Shape A (deviation > 1 step) is not scale-free. The script proves it: the same 2-step deviation is 0.105% on photo-06's 1912 and 2.941% on handle-logo's 68 — a **28x spread**. A threshold expressed in step count is arbitrary under different source/box ratios. Shape B (membership in {floor, ceil}) is scale-invariant: it simply checks that the declared value is one of the two rounding directions the rule could have chosen, with no intermediate choices allowed. This is geometrically tight: it admits only the values that preserve the fit inequality (cover's `s*f >= b`). The driving-axis check (zero tolerance) is correct because the driving axis is assigned the box dimension verbatim by construction and never computed; any deviation there is an author error, not a rounding choice. ADR-0013's "note" is correct under both futures (shape A or B), so it ships now at `note` severity. Once fit vocabulary is defined and the escape value lands, the check promotes to `error`, and shape B should be chosen because it is scale-free.

**Strongest counter:** Shape A (step count > 1) is simpler to explain and is the one ADR-0013 named first. However, simplicity of explanation is orthogonal to the check's correctness, and the 28x scale spread is disqualifying. A user on the handle-logo geometry would see a 1-step deviation flagged as an error, while a user on photo-06's geometry would see 2 steps waved through — same code, opposite answers, purely from geometry. Shape B is harder to explain but it is precise and fair across all (source, box) ratios.

---

## Anything the questions missed

**The absence of clip.** Fit is meaningful only when clip defines the aperture (the box to fit into). The schema should require that if fit is present, clip must be present, or clip should have a default value. ADR-0012 says clip is static and frame-space; nothing says what happens to fit when clip is absent. This creates an ambiguity: does fit without clip become an unchecked (ignored) claim, or is it an error? Recommend: if fit is present, clip is required.

**Forward compatibility of the escape value.** Choosing the escape value's name now locks in what "do not fit" means in the entire future of the format. The name should not collide with CSS `object-fit` (rules out "none"), should not reuse a committed meaning (rules out "cover", "contain"), and should be greppable/typeable. The fitness of candidates: `"exact"` (clear, not taken), `"literal"` (weaker, still available), `"declare"` (verb, confusing), `"skip"` (too vague). Recommend `"exact"`.
