# Juror O — working notes

Ex0: LEAVING, unprompted (ease on every record incl. a dead filler on the last).

Q11 measurements:
- SPLIT shift (delta 2000, at=1500 inside eased segment): ENTERING 2 changed + 2 inserted,
  0 changed records with unchanged t/v. LEAVING 3 changed + 2 inserted, 1 changed record
  with unchanged t/v — at t=1000 < at, ease-only rewrite, invisible-looking in a diff.
- Settled SPLIT wording ("insert at at/at+delta, shift keys strictly after at") is complete
  under ENTERING; under LEAVING it needs an extra clause to reach a record before at.
- Append: ENTERING 1 site, LEAVING 2. Prepend mirror. Delete-middle symmetric (~1-2 both).
- Tripwire: ENTERING can make first-record ease a schema error; CSS-prior authors (me, Ex0)
  write one, so the wrong prior bounces loudly = weight:"bold" pattern. Not guaranteed
  (a LEAVING author using linear defaults may omit it), so prior cost is reduced, not zero.

Q12 numbers: cover factor 0.703125; drawn 1080x1912.5; overflow 612.5 px (brief's ~612 ✓).
At scale 1.08: 765.5 px vertical, 86.4 px horizontal (frame-clipped). Visible wrong bands
on frame: 153 px (1300-1453) + 290.5 px (1622-1912.5).
- Settled-fields composition: only a background-coloured rect over the spill (solid bg hack);
  lies to the reader, fails over any non-solid ground. Genuinely inexpressible in general.
- Issue 22 quote: effects are a "closed, named, parameterised vocabulary"; candidates list has
  no mask/crop/clip; and "Ken Burns (which may instead be pure transform keyframes — decide
  whether it is an effect at all, since #21 may cover it entirely)".
- Census: 7 photos + handle-logo (8 image elements, all fit:cover) need the aperture; zero
  text, zero rect. handle-logo also carries invented mask:"circle" (FINDINGS.md: "Probably #22")
  — the decorative shape mask can stay in #22; the rectangular frame-space aperture cannot.
- Ship-without: fixture unportable to settled model / renders wrong; query --at stays blocked
  (ADR-0011 blocks it on "how fit, align and scale interact"); #22 would later have to amend
  the transform ADR to say the clip does not rotate/scale — a transform-semantics decision.

VERDICTS: Q11 ENTERING. Q12 aperture decided NOW in this ADR (rect frame-space clip);
decorative masks stay #22.
