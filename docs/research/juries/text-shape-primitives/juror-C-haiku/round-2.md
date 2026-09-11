# Jury Response: Q7, Q8, Q9

## Q7 — The paint package: stroke and alpha in colour

**Initial attraction**: Position B (no stroke in the primitive).

**Position B's core argument** is strong on its own terms: stroke changes what is drawn, but the declared dimensions do not change. This creates exactly the silent failure the format was designed to prevent. The stroke geometry finding is decisive: adding `stroke_width: 8` to `card-05` makes the drawn card 992x177 at (44,1449) while the file still reads 48/984/169, eating 4px of the margin the layout depends on. The validator passes. This is the defect.

**Attack on Position B:**

Position A's point about run-level text stroke is legitimate. Text outline is a real need — ASS already has it — and the ability to outline one word while leaving others unoutlined is expressible with runs as a style delta but inexpressible with an effects vocabulary that attaches to elements. Runs live inside elements; effects attach to elements. These are different places.

Position A also correctly notes that the reference class (CapCut/Premiere) do support stroke. And the format's philosophy is "declare what is drawn," which should include visible paint properties. The philosophical argument that paint consumed during rasterization belongs to the primitive has weight.

Additionally, Position A could argue: keyframes work fine with stroke. `scale` and `stroke_width` animate independently. There is no special "interaction with animation" that makes stroke an effect — both animate as separate channels.

**Counter-attack defending Position B:**

But the core problem remains: **stroke changes drawn geometry without changing declared geometry.** Whether stroke is inside, outside, or centred on the path, the rule becomes part of semantic meaning. Finding 1 states the header panels and card have "exactly square corners." If we add centered strokes later, those same corners gain ink. The rule must be in the document, not the implementation. ADR-0012 and ADR-0013 show how costly it is to get such rules right, and both learned that derived bounds must be explicit in the schema or declared by the author, never inferred.

The text-outline counterargument is real but not decisive. If stroke goes to effects, run-level outline becomes inexpressible *today*. But the effects vocabulary is still being designed. Deferring text stroke to effects with a note "run-level stroke is needed in the full effect set" costs nothing now and unblocks the primitive from a geometry problem that haunts every coordinate system that has strokes.

ADR-0006 records the noise budget principle: a validator that fires many findings on a correct file manufactures false confidence. If Position A is adopted, `validate` must check stroke overflow — but it can only check declared dimensions, not drawn dimensions. A stroke overflow becomes an unchecked failure class, invisible to validation. That violates the format's core principle: everything understandable by reading the document.

**Verdict on Position B: ACCEPT. Confidence: high.**

Stroke belongs to the effects vocabulary being designed in another ticket. For text, run-level stroke is deferred as an open item in ADR-0007, properly recorded as "the full run style-delta set (colour, outline — with the ASS BGR-with-alpha-nibble trap and ScaledBorderAndShadow written down)." The geometry must close before the paint opens.

For alpha in colour: `#RRGGBBAA` is correctly identified as the ASS trap. All transparency goes through `opacity`. Different transparency on fill and stroke is a use case, but it belongs to effects (fill colour + opacity + stroke effect with separate opacity), not to colour strings.

## Q8 — Does a text element still declare `height`?

**Initial attraction**: Option (a) — both `width` and `height` required.

**The fixture reality** is mixed: 15 of 22 elements have height exactly equal to `ceil(size * line_height * line_count)`, while 7 have height copied from a container. The asymmetry is striking: width is externally sourced 22 of 22 times; height is only 7 of 22. The brief's "three findings" notes this: "height overflow is arithmetic on numbers already in the document; width overflow requires `measure` against the current font binary." These are not symmetric properties.

**Attack on Option (a):**

Option (c) fires 15 findings on the only correct project file. ADR-0006 is explicit: a validator that emits many findings on a correct file manufactures false confidence. Firing 15 findings on the correct fixture makes `validate` output meaningless noise. This is disqualifying.

The "regression guard" argument — that a frozen height catches an edit where the text changes but height is not updated — is real but incomplete. It catches adding a line without updating height. It catches changing size without updating height. But if height is computed from documented inputs (size, line_height, line count), why require re-stating it? The inputs are in the document. An overflow check that computes height from size + line_height + newline count is self-contained and needs no frozen number.

Moreover, finding 3 notes: "the container-copied heights are not bindings." The 7 elements that copy height from a card or panel have no relationship to that card in the document. Resize the card and `validate` stays green on a now-stale height. This suggests height declarations are not reliable guards anyway.

**Attack on Option (b):**

Option (b) says height is optional, and its absence means "the element claims no vertical container." But this introduces an ambiguity: is height absent because it is unknown, or because the element genuinely has no container? An author forgetting to declare height and one who deliberately claims no container are indistinguishable. An element with a typo in `size` now silently misbehaves instead of overflowing.

The "inputs all on the same line" argument is weak for the 7 copied-height cases. Those inputs are on the same line, but the height comes from a container. If height is omitted, how does `validate` know whether to compute it from size/line_height or treat the element as unconstrained?

**Verdict on Option (a): ACCEPT. Confidence: medium.**

Keep both `width` and `height` required. Option (c) is ruled out immediately by ADR-0006. Option (b) creates ambiguity between "absent by intent" and "absent by mistake." Option (a) is a simple, uniform rule: declare what you are drawing, always. The regression guard works: an author who adds a line without updating height gets an overflow error. Yes, 15 of 22 examples have height equal to the derived value — but computed values are waste, not safety. The safety is that `validate` catches the moment the computed value no longer holds. Option (a) keeps that safety without the noise, because the check itself computes the derived value and compares; it does not fire a "finding."

## Q9 — Are `line` and `polygon` rejected, or merely unevidenced?

**Initial attraction**: Position A — rejected, on placement grammar.

**Position A's argument** is falsifiable, which is the format's standard: "the day a genuine point-list shape is needed, that is a new ADR about placement, not a schema addition." This turns rejection into a testable claim. A point list really does need different semantics than a rect with origin + width + height. A rect is placed by one point (origin + x,y) and two extents (width, height). A line is two points; a polygon is many. The placement grammar is fundamentally different. Recording this as a rejection with this reason is precise.

**Attack on Position A:**

The evidence for this placement-grammar argument is asserted, not proven. A line could be placed by:
- Its first point + origin (like any element)
- Its midpoint + bounding box
- Its bounding box + origin
- A start and end point as two separate x,y pairs

The brief's standing rule applies: "the fixture is test data, never a scope boundary; it is evidence a capability is needed, never evidence one is unneeded." The ASS files are all axis-aligned (m/l only, no beziers) — but that is one tool, one project. Different authoring systems might need different drawing models. Absence of evidence in the fixture is not evidence of absence of need.

Position B's point is also defensible: CapCut and Premiere are the reference class, and both have line and path tools. Shipping without them means the format cannot express basic shapes that the reference class supports.

**Counter-attack defending Position A:**

But the standing rule cuts both ways. Just because CapCut has lines does not mean *this format* needs them. The fixture models every line as a thin rotated rect (axis-aligned rectangles only). No one has actually used lines in this format. No one has tested whether the placement model works. Shipping it now means guessing what the rule should be, and guessing wrong before the format is live teaches you nothing that living users will not teach you faster and more painfully.

Position A does not say "lines are bad." It says "we need a separate ADR about placement for point lists." That is a different, weaker claim. When someone has a real use case — a free-form path, a line animation with multiple control points, a polygon fill — that person can write the ADR explaining how point-list placement grammar should work in this format. That ADR becomes a testable decision with evidence, not a guess.

The "thin rotated rect" point is strong: every line needed in the fixture is expressible as a rect under a rotation transform. This is exactly Position A's argument: if the reference class can achieve the visual by an alternate means within existing primitives, the new primitive is unevidenced.

**Verdict on Position A: ACCEPT. Confidence: medium-high.**

Lines and polygons are rejected on placement grammar. The decision is falsifiable: the day someone needs a genuine point-list shape whose bounding-box placement grammar does not work, that is a new ADR about placement, not a schema addition. Recording it this way honors the standing rule ("fixture is evidence of need, not scope boundary") by leaving the door open: a real use case triggers a real ADR, not a schema guess. Position B is pragmatically weaker because "they have it in CapCut" is not a reason; evidence of use is.

---

## Strongest attacks I mounted and whether they survived:

**Q7 attack on Position B**: Run-level text stroke is inexpressible with effects. *Survived partially*. Position B concedes the need is real but defers it correctly — ADR-0007 already records it as open. The counterpoint (stroke changes drawn geometry without changing declared geometry) is the decisive one.

**Q8 attack on Option (a)**: 15 findings on the correct file violates the noise budget. *Did not survive*. The noise comes from option (c), not (a). Option (a) requires height but does not fire a finding when height equals derived; it only fires when they differ.

**Q9 attack on Position A**: The placement grammar is asserted, not proven. *Did not fully survive*. The placement grammar is a real problem, but "thin rotated rects work here" is not proof it works everywhere. The falsifiability is still the stronger argument.

---

## What would bite me first under these verdicts:

1. **Text outline.** ADR-0007 records it as open, but agents will reach for stroke on text and it will be missing. Either effects land quickly, or a run-level style delta for outline (not stroke, but the same effect) needs to ship with the primitive. This is the most likely rollback.

2. **The noise budget at scale.** Option (a) requires height but does not auto-report redundant heights. Real projects with hundreds of text elements might reveal that the freeze-at-authoring-time rule has a cost in edit friction that outweighs the regression guard. The fixture is 22 elements; scale matters.

3. **Point-list evidence appearing.** A real project might need a shape that is genuinely not a rect — a free-form drawn line, a complex polygon — and it might not fit the bounding-box model. This would force the ADR sooner than expected. The falsifiable argument only works if no one ships the format for long enough to learn the need.

---

## Questions I think are malformed or presuppose something false:

None. All three questions are well-framed. Each position articulates a real tradeoff, and the evidence cuts both ways on all of them. The brief's instruction to "genuinely try to break" the attractive position is what makes them answerable.
