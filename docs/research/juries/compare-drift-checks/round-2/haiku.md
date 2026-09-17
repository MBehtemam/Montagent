🗳️ **Juror** (Haiku) — **VOTE: R2-Q1: YES (narrow to keyframe-only), R2-Q2: NO (exclude self-keyframe-on-boundary), R2-Q3: Option B (design with unmeasured noise caveat)**

**Reasoning:**

**R2-Q1:** Slack-drift already ships the predicate and output shape for boundary-coincidence destruction (coincidence is precisely slack of size 0; destroyed-coincidence is slack changing from 0 to N). The prior court didn't object to this overlap, and this ticket's design cost is highest where it's novel. Keyframe-involving relationships are the residue slack doesn't mechanically cover—focus the design work there. The team should document that boundary-coincidence is subsumed by slack-drift, not re-invent it.

**R2-Q2:** ADR-0036 already judged self-keyframe-on-boundary "structurally benign" in validate's census. Benignness is precisely the logic for excluding it from destroyed-coincidence hazard tracking. A keyframe on its own element's boundary is not a hazard condition; it's definitionally correct authoring. The fact that 7/7 real elements have this pattern suggests it's noise, not signal—compare should avoid surfacing it.

**R2-Q3:** Zero real examples of case (c) is exactly the data point that justifies Option B: the project's documented pattern (frame-alignment, sequence-validation labels) shows theory-only designs routinely prove wrong on real files. Design the predicate fully now (per-property, cross-element, delta-0-in-ref-nonzero-in-current), settle the output format, but explicitly note in the ADR that false-positive/noise behavior is unmeasured. Ship the mechanism disciplined by that caveat, measure against real coupled-motion projects, correct if needed.

**Trade-offs:**

- **Q1:** Loses explicit "destroyed-coincidence" naming for boundary cases (developers read "slack changed" instead), but avoids re-designing slack-drift and sharpens focus. Requires clear cross-reference in docs.
- **Q2:** Trades tracking of self-keyframe position changes for consistency with ADR-0036's benignness judgment and signal-to-noise discipline. If this pattern legitimately moves in real authoring, validate will catch the error; compare doesn't need to chase it.
- **Q3:** Defers measurement risk to a follow-up ticket, but prevents shipping a noisy check that teams disable. Consistent with observed project culture of testing theory first.
