Retarget 1080x1920 -> 1920x1080, by eye over the #9 file:
- "frame": one line. Then: 7 photo boxes [0,0,1080,1300] (1300 > new frame height —
  invalid composition, not just ugly); 5 card rects [48,1453,984,169] (y=1453 is
  off-screen at height 1080); 22 text x/y (x=540 was center of 1080 — is 960 now;
  y=1537/1470/1373/1352/1398 all off-screen); 8 header chrome rects/positions;
  every hand-fitted size (88/80/73/55/49/35) is fitted to a 984-wide card that no
  longer exists at that width. That is ~50 of 60 elements.
- No scale factor exists: 1920/1080 horizontally x 1080/1920 vertically is a
  non-uniform squash that destroys type and circles. Fractions (0..1) would make
  the file "valid" after one edit and WRONG everywhere — a portrait stack of
  photo-over-card-over-caption is not a landscape layout at any coordinates.
  Fractions convert a loud re-layout into a silent one.
- So: inherently a re-layout, owned by the agent. What the format can do:
  (a) pixels + literal frame, so the mismatch is at least computable and validate
  can flag out-of-frame geometry; (b) group/query selectors so the re-layout is
  per-slot not per-element; (c) frame/measure so the new fitted sizes are cheap.
