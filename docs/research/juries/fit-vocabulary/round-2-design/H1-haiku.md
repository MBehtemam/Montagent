# Round 2 Verdict — Haiku

## R2-Q1: The escape value's name

**Decision:** `exact`

**Why:** The vocabulary uses effect-describing names (`cover`, `contain`). The escape value describes the final state of the geometry — declared integers that are exact and not derived from a fit rule. Using `exact` aligns with ADR-0013's language ("exact integer arithmetic") and signals that the declared rect is authoritative, period. It avoids collision with CSS `object-fit:none` (which means use intrinsic size), preventing confusion for agents trained on CSS. The name is not spent elsewhere.

**Strongest counter:** `declared` is clearer about the *cause* (no derivation was applied) and avoids potential ambiguity with "exact" in other contexts. But `exact` is already the language ADR-0013 uses for the arithmetic it mandates, making it more internally consistent. Cause-focused naming (declared) reads well until an agent sees "the declared rect is authoritative" and asks whether every element's rect is therefore "exact" — effect language is cleaner.

## R2-Q2: Does `contain` ship in v1, or only `cover` + escape?

**Decision:** Ship `contain` in v1. Inequality: `floor((s_slack * b_driving) / s_driving)` preserves `s*f <= b` on the slack axis. Rounding direction: floor (same as cover, for consistency and safe implementation).

**Why:** The rounding rule is deterministic. ADR-0013 explicitly states "if `contain` is ever defined with the obvious semantics, only floor is safe there." The rule is ready; deferring gains nothing. On the aperture-coverage error collision: the error exists to catch unintended coverage violations. Under `contain`, the source is shrunk to fit inside the clip — there is no coverage violation risk. Scope the error to check that coverage requirements are satisfied: for `cover`, "declared rect must contain clip"; for `contain`, the check is trivial (source is shrunk, cannot overshoot) or omitted. This is a parameterization of the existing check, not a new mechanism. The fixture's non-use of `contain` is not evidence against it (ADR-0003 asymmetry: fixture is evidence a capability is needed, not that one is unneeded).

**Strongest counter:** Deferring `contain` keeps the spec smaller and forces the question only when a real project needs it. The parameterization of the aperture error, while feasible, adds a conditional to a rule ADR-0013 deliberated carefully. But "only floor is safe for contain" is already settled; shipping it means one code path, one rounding. Deferring means rediscovering the decision later.

## R2-Q3: What does a declared `fit` mean when `clip` is absent?

**Decision:** A declared `fit` without `clip` is a schema error.

**Why:** `fit` is defined as deriving extents by fitting a source into a box. The box is `clip`'s width/height (R1 settled fact). Without a box, the field has no meaning. The tempting fallback — use the element's own declared rect as the box — produces the fixed-point property noted in the brief: every image element tested reproduces its own declared rect. This cannot be discriminated from a correct fit and is algebraically useless. Defaulting to the project frame is incorrect (the fit rule operates on element-local geometry, not frame space). Making the element UNCHECKED violates ADR-0006's stricture against checks that "silently weaken to nothing" — an agent would write `fit` expecting a check and get silence instead. The pattern ADR-0012 established is clear: "a field the renderer cannot honour is worse than no field." `fit` without a box cannot be honoured.

**Strongest counter:** Omission is undefined today; allowing the element to be UNCHECKED keeps the file legal and lets validate report what it found. But that is precisely the trap ADR-0006 identifies: false confidence from a weakened check. The write-read round trip is also hazardous — an agent writes `fit: exact` believing it is checked, then `fmt` stays silent, and the error never fires.

## R2-Q4: Define "the source's dimensions" normatively

**Decision:** The source's dimensions are the decoded pixel dimensions of the raster data itself, without applying EXIF orientation transformations or other presentation-layer conversions. A JPEG with an EXIF orientation flag is still reported as the dimensions of the encoded pixels, not the rotated visual dimensions. For video, the source dimensions are the frame dimensions (width × height) at which the file was encoded; if a video's actual frame dimensions vary (rare, typically forbidden), use the first frame's dimensions and `validate` notes the discrepancy.

`validate` **must print the dimensions it used** in the fit-deviation report.

**Why:** Normative specificity is required to prevent the ADR-0005 `speed` divergence from recurring. Different image libraries handle EXIF differently; some apply it silently, some don't. Without a normative definition, four agents will compute one integer and one another. Printing the dimensions in validate catches discrepancies immediately — if an agent writes `fit: cover` for what it believes is a 1536×2720 image but validate sees 2720×1536 (EXIF rotation applied), the mismatch becomes visible. This follows ADR-0006's rule: findings state facts. For video, frame dimensions are scalar and unambiguous; the first frame is the operand.

**Strongest counter:** The definition is library-specific; no normative sentence will force all decoders to behave identically. But that is a problem for the decoder specification, not for this ADR. What *this* ADR must do is define what *it* assumes. The printing requirement is the real safety net: if implementations diverge, printing reveals it.

## R2-Q5: Does this ADR cover `video` elements, or only `image`?

**Decision:** The rules apply type-generically to both `image` and `video` from v1. Write the rule as defined here; both are raster media with pixel dimensions, and `fit` operates on those dimensions, not on type.

**Why:** The rule is spatial scaling of raster content, indifferent to source type. Both images and videos are placed by `x`, `y`, `origin`, `width`, `height` and `clip` (all settled in ADR-0012). Both can be scaled and rotated. The fit rule has no reference to image-specific properties. ADR-0003 commits Montaget to video; deferring this rule to later invites divergent implementations — someone will define `fit` for video differently in a future ADR, and files with both image and video elements become ambiguous. Better to settle it once.

Wrinkle specific to video: **for video elements, the source dimensions are the encoded frame dimensions (typically constant). If a video has variable frame dimensions (exotic, rare), use the first frame and note the variance.** This is a clarification, not a new rule. The fit rule itself is unchanged.

**Strongest counter:** The fixture contains only images; the rule hasn't been tested on video. Deferring to when video elements exist in a real project avoids speculating. But that is exactly what ADR-0003 forbids: letting the fixture's non-use shape the spec. Video is a declared scope; the spatial scaling rule is mechanical.

## Anything these five missed

No holes found. The questions cover the vocabulary, the omission semantics, the clip absence case, implementation precision, and type coverage. One weak spot not yet addressed: the render-time rounding for videos at high frame rates (e.g., a 6K frame resampled at an exact non-integer factor) — but that is a render implementation detail, not an ADR-level question. The fit rule itself is orthogonal to framerate.
