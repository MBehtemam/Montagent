# Jury D1 Verdict: Define the fit vocabulary

## Q1. Is `fit` a derivation claim, or a render instruction?

**Decision:** `fit` is a **derivation claim**, not a render instruction.

**Why:** ADR-0013 settled that "The declared rect is authoritative at render — the source is resampled to exactly that rect." The renderer consumes the declared `width`/`height` directly; it does not compute them from `fit`. Instead, `fit` documents *how the author computed* those declared dimensions from the source. The actual consumer of `fit` is `validate`, which uses it to check whether the declared rect matches the rule value computed from the source on disk. Under render time, `fit` does nothing — it is strictly a validation/documentation field that makes an assertion about the derivation history of the declared extents.

**Strongest counter:** One could argue that `fit` IS a render instruction because it specifies the scaling relationship between source and output. But ADR-0013 explicitly forecloses this reading by making the declared rect authoritative *regardless of source dimensions*. If `fit` were a render instruction, the renderer would use it to scale the source. Instead, the renderer resamples the source to exactly the declared rect, treating `fit` as inert documentary evidence. The rect is final; `fit` only says how it was supposedly derived.

## Q2. What box does the fit rule fit into?

**Decision:** The box dimensions are `bw = clip[2]` and `bh = clip[3]` — **the aperture width and height defined by the `clip` field**, not the declared `width`/`height`.

**Why:** Tracing the fixture's actual numbers: photo-06 has source 1536×2720, declared dimensions 1080×1912, and clip [0,0,1080,1300]. The fit rule `(2720×1080)//1536 = 1912` produces the declared height when `bw=1080` (the clip width, not the declared width). This is correct because `fit:"cover"` means "scale the source to completely cover the aperture." The aperture is defined by the clip rectangle's extent [1080,1300]. Scaling to cover it: width is driven since `1080*2720 >= 1300*1536`, so height = (2720×1080)//1536 = 1912. For handle-logo (the instructive difference): source 800×800, clip [478,96,68,68], declared 68×68. The calculation (800×68)//800 = 68 is exact because the clip's extent is 68×68, matching the source's aspect exactly. Both elements demonstrate that `fit` operates on the clip aperture, not the declared rect.

When `fit` is absent: Per ADR-0006, omitting a field is indistinguishable from a decision not to check. An omitted `fit` means the declared rect is accepted without any fitting assertion or validation. There is no way to know what operation, if any, was used to derive the extents.

## Q3. Is `fit` required on an element, or may it be omitted?

**Decision:** `fit` **must be required** on image and video elements (those with a source).

**Why:** ADR-0006 establishes the principle: "An omitted field is indistinguishable from a decision not to check." The fit-deviation check is currently a `note` but will eventually become an `error` (Q6). If `fit` is optional, omitting it becomes a silent way to opt out of the error check, reintroducing the opt-in-check danger ADR-0006 exists to prevent. ADR-0014 applies this reasoning to text `height`: it rejected making the field optional despite it being redundant with typography inputs, because an omitted field creates ambiguity about whether a stale value is deliberate or forgotten. The same applies to `fit` — once the error check is live, an optional field is an indirection to bypass it. Requiring `fit` means `validate` always checks, and the check is never opt-in.

**Strongest counter:** ADR-0012's defaults list omits `fit`, and `verify.py` doesn't assert it, suggesting it was left optional by design. The current check is only a `note`, so requiring the field now creates burden for future benefit. But ADR-0006's principle applies regardless of the current check severity: a field that *should* be always-checked should be required from the start. Shipping it optional now, then making it required in Q6's follow-up, would force a schema change that breaks existing files.

## Q4. What is the closed value set, and how is "do not fit this, use my rect" spelled?

**Decision:** v1 ship set is **`{cover}`**. The escape hatch is spelled **`none`**.

**Why:** `cover` is the only value appearing in the fixture and the only one with a published rule. ADR-0013 explicitly refuses to legislate a rounding rule for `contain` because doing so would "create a schema value by implication" — the refusal the ADR makes by name. For the escape: an author who declares a rect without fitting (i.e., the rect is exact, not derived from a fit operation) must spell this somehow. The value `none` is clear, unambiguous (the declared rect is final, not a fitted result), and does not collide with CSS `object-fit` terminology (which has no `none`). Alternatives like `exact` or `identity` are equally valid; `none` is chosen because it reads as "do not apply any fit operation."

`contain` should NOT ship in v1. While it is geometrically obvious (scale to fit inside the aperture, unlike cover which scales to cover it), including it requires publishing its rounding rule. ADR-0013 already computed this: `contain` uses `ceil` on the slack axis to preserve the inequality `s*f <= b`. But shipping both rules creates an expectation that the fit vocabulary is complete. Better to ship one value (`cover`) and one escape (`none`), then add `contain` in a follow-up issue once the rest of the fit semantics are proven stable.

Collision check:
- `cover` — means the same as CSS `object-fit: cover`, which is NOT misleading; agents expecting CSS semantics will find what they expect.
- `none` — custom, no CSS collision. It clearly means "no fitting applied."
- `contain` (if added later) — would mean the same as CSS `object-fit: contain`, which is also NOT misleading.

**Strongest counter:** The opposite-operation pair (cover/contain) is so obviously needed that shipping only cover forces workarounds or fictional intermediate rects. But ADR-0013's refusal to rule in `contain` without settling its rounding is load-bearing: a half-specified value is worse than an absent one. Adding `contain` later in a follow-up PR breaks zero existing files because `fit: "contain"` is not valid in today's schema anyway.

## Q5. What happens to `gravity`?

**Decision:** **Retire `gravity` on image elements**. Make it a schema error with a message explaining the field is inert and naming the two alternatives: adjust `clip` coordinates, or adjust `width`/`height` to select the desired source region.

**Why:** ADR-0013 measured that `gravity` is **inert on 8 of 8 image elements** in the only real project file. Under the settled model (explicit `width`/`height` and `clip`), the declared rect and aperture together fully determine which part of the source is visible. The gravity parameter has no effect on output. ADR-0014 explicitly declined to decide this, saying "Deciding it here would create a schema value by implication" — the refusal to legislate a value without full specification. But now the measurement is in: it is genuinely inert, not merely unmeasured. Retiring it (making it a schema error) is the honest documentation of this fact.

Cost to fixture: All 8 image elements currently carry `"gravity": "top"` or `"gravity": "center"`. Removing these fields is a breaking schema change, but it changes **zero bytes** of the committed output video (since gravity is inert). The fixture's 7 photos show gravity is dead weight; the script confirms it.

**Strongest counter:** Keeping the field costs nothing — it's already there — and might become useful if the model evolves. For instance, if a future fit mode allows variable-aspect-scaling, gravity could determine which part of the source becomes the primary axis. But ADR-0012's rejected-optional-field principle applies: a persisted inert field creates false confidence that it might do something. Better to retire it cleanly, documented as a schema error, and add it back later if a use case emerges.

## Q6. The severity and shape of the fit-deviation check

**Decision:** The check is an **`error`**. The predicate is: **membership in `{floor(exact), ceil(exact)}` on the slack axis with zero grace on the driving axis**.

Exact formulation: For each image element with `fit:"cover"`, compute the exact fitted dimensions using ADR-0013's rule (integer arithmetic, floor). The declared `width` and `height` must satisfy:
- On the driving axis: exact match to the box dimension (driving axis is exact by construction).
- On the slack axis: the declared value must be exactly `floor(exact)` or `ceil(exact)`.

**Why:** ADR-0013 identifies two candidate shapes. The chosen shape (membership in `{floor, ceil}`) is better than "deviation > 1 step" because step count is not scale-free: a 2-step deviation on 1912 is 0.105%, but the same 2-step deviation on 68 is 2.941% (28x spread). The membership rule is scale-invariant and catches meaningful deviations (stale sources causing 23.6% stretch) while rejecting no correct files. The rule says: the declared extent is either the floor or ceil of the exact value, nothing else. This catches the reproduced stale-source case (1912 declared vs. 1546 computed gives 1912 ≠ {1546, 1547}, error) while passing all committed elements.

**Checking against the fixture:**
- photo-05-intro: source 1536×2720, box 1080×1300, declared 1080×1912
  - Exact: (2720×1080)//1536 = 1912 (floor)
  - Driving axis: width 1080 (exact) ✓
  - Slack axis: 1912 = floor(1912) ✓
  - Status: PASS

- photo-05, photo-06, photo-07, photo-08, photo-05-quiz, photo-05-loop: identical geometry, all PASS

- handle-logo: source 800×800, box 68×68, declared 68×68
  - Exact: (800×68)//800 = 68
  - Driving axis: width 68 (exact) ✓
  - Slack axis: 68 = floor(68) = ceil(68) ✓
  - Status: PASS

**Total broken: 0 of 8 elements.**

**Strongest counter:** Making this an error now is premature because the escape hatch doesn't exist yet. Issue #48 must provide `none` first (Q4), so authors have a way to declare a non-fitted rect. ADR-0012's principle says "a schema error naming the replacement" — but we're providing the replacement (none) in the same ticket, so this objection is answered. However, if we're unsure about shipping the `none` escape in this PR, the error check could be deferred to a follow-up PR to #48 that only ships after `none` is validated. The measurement (0 of 8 broken) means tightening later breaks nothing in the committed corpus.

## Anything the questions missed

**Whether `fit` should be keyframable:** The current fixture doesn't animate `fit`, and animating it would be strange (changing the fitting operation mid-element doesn't make sense under the "declared rect is authoritative" model). But no ADR has explicitly rejected it, and the fixture's lack of use is not evidence (ADR-0003). However, this is cleanly deferred: `fit` is not a transform property, so it doesn't participate in keyframe lists. If a use case emerges (e.g., transitioning between two fitting modes), that's a separate ticket.

**Whether the closing condition for `fit` is settled:** The issue says the escape hatch blocks the error check. We've spelled the escape (`none`), so the condition is met. But the actual error-check implementation should be tested against real files beyond the fixture before shipping. The scale-invariant predicate is sound in theory; the scan proves it on one geometry. Recommend a broader test when the check is implemented.
