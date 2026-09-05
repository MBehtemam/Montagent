# Juror P — verdict

Q11: LEAVING. Q12: decide the aperture NOW, in the transform ADR (static frame-space clip rect).

Ex0 prior: LEAVING, unprompted (see ex0-prior.md, incl. contamination note).

Key measurements:
- SPLIT diff (shift delta=2000 at t=1500 inside eased segment):
  ENTERING: 4 changed records, 0 with unchanged t&v; all touched records t>=at.
  LEAVING: 5 changed records, 1 with unchanged t&v (the pre-at record t=1000 whose
  named ease-out is rewritten to cubic-bezier(0,0,0.454208,0.571917)).
- Append: ENTERING 1 site / LEAVING 2. Prepend: mirror. Delete-middle: 2 sites both,
  and both conventions produce an ease-only change on an unchanged-t,v neighbor.
- Middle-keyframe ease misplacement is schema-invisible under both conventions;
  only first/last-slot misuse is rejectable. So the wrong prior cannot be rejected,
  it can only be avoided by matching it. Repo's weight:"bold" precedent applies only
  to rejectable habits.
- Q12: cover factor 0.703125, scaled 1080x1912.5, overflow 612.5 px (765.5 at zoom 1.08).
  8 of 9 image placements need the aperture (7 photos + possibly logo); 0 text, 0 rect.
  FINDINGS.md §G already files box/fit under #21 and mask:"circle" under #22.
  Issue #22's scope is a "closed, named, parameterised vocabulary" of effects
  (fade, blur, shadow, outline, colour filter); no mask/clip/crop word appears in it.
