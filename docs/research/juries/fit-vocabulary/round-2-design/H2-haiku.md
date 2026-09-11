# Jury 2 verdict — Montaget issue #48, the `fit` vocabulary

Juror: H2 (Haiku 4.5)

## R2-Q1: The escape value's name

**Decision:** `declared`

**Why:**
The escape value must name the act of author declaration without derivation. Candidates ranked:

1. **`declared`** — explicitly names the authorial act ("the author declared this rect"); sits outside the `cover`/`contain`/etc. vocabulary; avoids CSS collision (no `declared` in object-fit); describes the *cause* (no rule applied), which is load-bearing for validation logic that checks whether a *derived* rect contradicts the declared one. The noun "declaration" already lives in ADR-0012 and ADR-0013 as "declared rect", so `declared` as a value name is natural extension rather than collision.

2. `exact` — evokes ADR-0013's "exact cover is 1912.5 px"; risks ambiguity with the mathematical concept of exactness in the fit rule itself; no CSS collision; describes the *effect* (the source is sampled exactly at the declared rect, no re-derivation).

3. `none` — CSS object-fit does have `none` (preserve intrinsic size), which is semantically opposite to what we need here; minimal; but the collision with CSS is real enough that an agent familiar with CSS will misread it.

4. `stretch` — CSS object-fit does have `stretch` (deform to fill); actively misleading.

`declared` is permanent and should not collide with future fit values. The entry in the closed vocabulary is: `"fit": "declared"` meaning "do not apply a fit derivation; the width/height are author-asserted."

**Strongest counter:**
`exact` is stronger on grounding in existing ADR prose ("exact cover"). The suffix-free `declared` requires the reader to know the *concept* it opposes (derivation), while `exact` is self-contained. On a fixture where 7 of 8 elements animate scale to 1.08 and already carry anisotropy, the name `exact` could be read as "this rect is exact, go ahead and distort if the source forces it" — a trap the name `declared` does not quite set, because an agent reading "I declared this" hears "I own this geometry."

---

## R2-Q2: Does `contain` ship in v1, or is the set `cover` + escape only?

**Decision:** Defer `contain` to v2. Ship `cover` + `declared` in v1. A schema error on attempted `fit:"contain"` names the unsupported value and directs to an ADR link for planning.

**Why:**
Under settled reading, `contain` rounding is identical to `cover` on the slack axis (floor in integer arithmetic, same as `cover`). The collision is not rounding; it is error scope:

1. **ADR-0013 publishes an aperture-coverage error, hard-coded to the `cover` direction:** "the declared rect must contain `clip`."

2. **A correct `contain` element is *smaller* than `clip`, and would fail this error by construction.** If an element fits inside its aperture, the aperture cannot fit inside the element.

3. **Fixing this requires parameterizing the error:** a `cover` element must contain the clip; a `contain` element must be contained by the clip. One error field, two directions.

4. **The fixture provides no evidence for `contain` semantics in practice.** All 8 image elements use `cover`. ADR-0003 says evidence of *use* licenses a capability, and the corollary holds: absence of use is not evidence against need — but shipped code must not leave a trap. If the error fires on every legal `contain` element in v1, the field is a hazard to agents, and a hazard to the validate check that is supposed to be safe.

5. **Deferral is not a rejection.** The rules for computing the slack axis are in place; the open question is whether `contain` elements should be checked against a different aperture-coverage rule (contain rather than contain). That is resolvable in v2, and the schema error names it.

**Strongest counter:**
The fixture uses only one geometry on 7 of 8 elements — `(1536, 2720) -> (1080, 1300) = 1080x1912` — copied by a script. A second geometry (`800, 800 -> 68, 68`) is exact and needs no rounding. **No geometry in the real project has tested the floor-versus-ceil decision on `contain`.** If `contain` ships with identical rounding (floor), it is also bulletproof, and the only question is what the aperture-coverage error does. Deferral trades "safe to ship with a parameterized error" for "safe to ship with a deferred decision", and the second does not ship the thing. On a general editor, shipping `contain` with a clear note that the coverage check is not yet defined is better than deferring the entire fit vocabulary split.

---

## R2-Q3: What does a declared `fit` mean when `clip` is absent?

**Decision:** A declared `fit` without `clip` is a schema error, naming `clip` as required when `fit` is declared.

**Why:**

1. **`fit` is a derivation rule that requires a box.** Under ADR-0013, settled reading, the fit rule is: *compute the extent by fitting source into box.* No box, no rule. The rule answers "source of size S goes into box of size B; what is the derived extent?" If B is undefined, the question is unanswerable.

2. **The fixed-point trap.** If the box fell back to the element's own declared rect, the fit rule becomes `fit(S, rect) -> rect`, which is a mathematical fixed point: no matter what fit value is written, the declared rect is reproduced. Under ADR-0013, this makes the check meaningless. The check is: "does the declared rect match the derivation from the rule?" If the rule *is* the declared rect, the check always passes, validate fires zero times, and the field becomes inert—exactly the failure mode ADR-0006 forbids: *"a check that silently weakens to nothing."*

3. **The project-frame fallback is also inert.** For `photo-06`, the element is 1080x1912 and the frame is 1080x1920. Fitting 1536x2720 source into 1080x1920 gives 1080x1910, not 1912. Using the project frame as a fallback box would require the element to *contradict* its own declared rect to avoid validation, which is a worse trap than the fixed point.

4. **Schema errors naming requirements are the stated pattern.** ADR-0012: *"a schema error naming the replacement."* If a `fit` field appears on an element with no `clip`, the error message is: `fit declared without clip: clip is required when fit is used`. This is loud and catches the mistake before rendering.

**Strongest counter:**
`clip` is optional because not all elements need an aperture — text and shapes have no source, and a rect is its own box (ADR-0012). Making `clip` required-when-`fit`-is-declared creates a coupling that might have been avoided by letting the element's own rect serve as the box: the field is optional everywhere, and `validate` simply does not check fit-deviation on elements with no clip (UNCHECKED). Under this reading, omitting `clip` on an image with `fit` is not wrong; it is a signal that the element is outside the scope of the fit-deviation check. The element still renders because the declared rect is authoritative (ADR-0013), so no media is needed, and the file is legal. A schema error is stricter than the requirement actually is.

---

## R2-Q4: Define "the source's dimensions" normatively

**Decision:** 

The source's dimensions are **the width and height as displayed by a conforming media player, with any EXIF orientation transformation applied and pixel aspect ratio normalized to 1:1.**

Specifically:
- For still images: read the image file; if EXIF Orientation tag is present, apply the rotation/transpose it specifies before measuring; report the resulting width and height in pixels.
- For video streams: read the stream; report the frame width and height as would appear in playback, with pixel aspect ratio normalized to 1.0 (square pixels).
- `validate` MUST print both the dimensions it used and the source file path/identity.

**Why:**

1. **Precision requirement from ADR-0005.** The `speed` divergence (3368/0.645 → 5222 vs 5220) showed that rounding rules diverge when input dimensions are ambiguous. The integer-arithmetic fit rule's entire purpose is that independent implementers land on the same integer. Ambiguity in input dimensions defeats that goal completely.

2. **Visual dimensions are the reference.** A user looks at their image in a media player or file explorer; that is the dimensions they author for. EXIF Orientation is transparent: a portrait image with a 90° rotation tag appears as portrait in the player and should be treated as portrait in Montaget. Ignoring EXIF creates a silent mismatch where the author's declared rect contradicts the visual appearance of the source.

3. **Pixel aspect ratio normalization is standard.** Video streams can carry non-square PAR (e.g., 1.2:1). But that is a display-time correction, not a geometry of the source. The fit rule works on pixel dimensions, not display dimensions. Normalizing PAR to 1:1 means independent implementers agree on the raw pixel count, not the display scaling.

4. **Precedent from ADR-0005 on source duration.** That ADR requires `validate` to *print* the source's actual duration when checking time ranges. The same obligation applies here: print the dimensions used. This is not optional; it is required for the check to be actionable by an agent.

**Computation example (JPEG with EXIF Orientation):**
- File `images/06.png` on disk: 1536×2720 pixels
- EXIF Orientation: 1 (no rotation)
- Reported dimensions: 1536×2720

If a JPEG carried EXIF 8 (rotate 270°), a 1536×2720 image would visually appear as 2720×1536:
- File dimensions: 1536×2720 (raw)
- EXIF Orientation: 8 (270° rotation)
- Reported dimensions: 2720×1536 (visual)

**Strongest counter:**
EXIF Orientation is metadata that some tools ignore. Requiring its interpretation could create divergence *because* of the requirement: a naive implementation that ignores EXIF will still diverge, and the validate error ("I saw 1536×2720, but the rule computed a different value") will be harder to debug. Normalizing to raw pixel count avoids the EXIF branch entirely and lets implementers converge on the simpler rule. The counter argument is especially strong for video, where PAR is often absent, and forcing its presence would require defaulting to 1:1 anyway.

---

## R2-Q5: Does this ADR cover `video` elements, or only `image`?

**Decision:** The ADR covers both `image` and `video` elements. The fit rule (driving/slack axis selection, floor rounding) applies generically to both.

The normative definition of "source dimensions" (R2-Q4) must clarify: for video, source dimensions are **frame dimensions** (width and height of one decoded frame), not duration.

**Why:**

1. **The rule is about spatial fitting, not temporal properties.** `fit` describes how source spatial dimensions map to a declared rect. A video's frame dimensions fit into a clip the same way an image's pixel dimensions do. The source-range (start/end of playback) is a separate concern, governed by ADR-0005, not by the fit rule.

2. **ADR-0003 commits Montaget to general video editing.** The architecture must accept video clips, not just images. If the fit rule is scoped to image only, video elements cannot use `clip` for aperture, and the scaling/Ken Burns geometry breaks. That is unacceptable for v1.

3. **The fixture doesn't test video, but the absence is not evidence.** All 8 elements are images; zero are video. By ADR-0003, this is evidence that images must work; it is not evidence that video cannot or should not.

4. **No wrinkle is introduced that images do not already face.** Video has an internal playback rate (`speed`/playback), but that is orthogonal to spatial fitting. A 1920×1080@30fps video fits into a clip the same way: read frame dimensions (1920×1080), apply fit rule, get declared rect. The playback rate affects what portion of the timeline is occupied, not the spatial derivation.

5. **The schema already treats video like image structurally.** Both have `source`, both can have `scale`/`clip` for aperture (ADR-0012). Neither `image` nor `video` carry a `source_range` by default—only time-based elements do—so the fit rule has no temporal input either way.

**Strongest counter:**
Video source dimensions are not always stable: a stream might report dimensions in a header, but the actual decoded frame size could differ (e.g., due to scaling in encoding). For images, a PNG or JPEG is its pixel dimensions; for video, the query "what are the source dimensions" is ambiguous until decoding starts. A safe interpretation would scope the rule to `image` only in v1 and defer video to #22 or a later ticket, ensuring the fit rule is only applied to sources whose dimensions are deterministic by inspection. The fixture's single-geometry data set does not generate confidence in video generalization.

---

## Anything these five missed

**Cache the computed fit values in the project file.**

Once the fit rule is settled and locked (which happens after #48 closes), a future concern: `validate` runs the fit rule over every image and video element and reports any deviation. But the rule is deterministic and expensive (pixel-perfect integer arithmetic on every element, possibly with media I/O). On a long project with hundreds of elements and a validate run after every edit, this scales poorly.

The schema could carry an optional `fit_computed` field (or similar name) recording the rule's output the last time it was checked. `validate` would re-compute, compare, and only report if they diverge—a signal that the source changed. This pattern is already used for the media-probe cache (ADR-0006). It would be a v1.1 optimization, not a blocker, but it should be named now before the schema closes, so the field name is reserved and the decision is visible.

---

**`gravity` decoupling from `fit` creates a blind spot.**

Under declared-rect-authoritative, `gravity` is inert (ADR-0013). But the check that proves it is a *measurement*, not a law: it checks 8 image elements in one fixture. If an agent later writes an element with `clip` but no `gravity`, the aperture position (and thus which part of the source survives) is indeterminate. The check fires zero times because all 8 committed elements carry `gravity: "top"`.

This is not a bug in round 2's decision—`gravity` is correctly deferred—but it is a bug factory for v2: when `gravity` is eventually settled, the check will need to detect elements with no `gravity` specified and decide whether it is an error, a note, or a default-able field. That decision should name the edge case explicitly, so it does not land during `gravity` review as a surprise.

