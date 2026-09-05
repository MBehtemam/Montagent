# Exercise 4 — box/align retirement, rewritten for real

## photo-06 (was: "box":[0,0,1080,1300],"fit":"cover","align":"top")
{"id":"photo-06","type":"image","start":17472,"end":30603,"source":"images/06.png","x":540,"y":650,"origin":"center","width":1080,"height":1300,"fit":"cover","gravity":"top","scale":[{"t":17472,"v":1.0},{"t":32472,"v":1.08}]}
- x,y,origin replace box's position; width,height replace its size. I chose center origin
  deliberately: the Ken Burns zoom must pivot at the slot's centre, and the box form left
  that implicit (round-1 evidence: "a scale with no origin is unreadable"). BETTER, not worse.
- fit stays; align -> gravity. Retiring fit entirely would force:
  cover scale = max(1080/1536, 1300/2720) = 0.703125; visible source height = 1300/0.703125
  = 1848.888... px -> a fractional source-crop rect, computed from dimensions (1536x2720)
  that are NOT in the document. Authoring "put this photo in this slot" becomes probe+math.
  fit:cover still violates ships-its-own-inputs (#21 comment 2), but the crop alternative
  needs the same missing numbers AND makes the agent do the arithmetic. Keep fit; make
  validate report the probed dimensions and the resolved crop.

## card-05 (was: "box":[48,1453,984,169])
{"id":"card-05","type":"rect","start":10468,"end":17472,"x":48,"y":1453,"origin":"top-left","width":984,"height":169,"fill":"#1E344C"}
- Pure mechanical expansion, 1 field -> 5 flat fields, ~26 chars longer. Nothing lost;
  x/y become keyframable like every other element, one grammar for all types.

## text with ADR-0007's "box":"card-05"
was: {"id":"word-05",...,"x":540,"y":1373,...,"box":"card-05",...}
now: {"id":"word-05","type":"text","start":5316,"end":17472,"x":540,"y":1537,"origin":"center","font":"brand","size":55,"line_height":1.1,"color":"#FFF8E8","width":984,"align":"center","runs":[{"text":"I hang cobwebs over the door."}]}
- The id-reference becomes a literal width (the overflow-check term ADR-0007 says the check
  gains). Cost, stated loudly: the anti-drift binding is LOST — resize card-05 and the text's
  width:984 silently desyncs, exactly the drift the layer-anchor exists to prevent. But the
  binding was a second positioning model (a size by reference) inside the one round 1 just
  unified, and the fixture already carries x/y literally beside the box reference, so the
  desync surface already existed. Group + compare is the mitigation, not a hidden reference.

## align
Same treatment as box: one word, two meanings, split it.
- image gravity -> "gravity" (top/bottom/left/right/center...), legal only beside fit:cover.
- text line alignment -> "align" stays, now single-meaning, text-only (left/center/right).
Nothing became inexpressible; photo-06 got better (explicit pivot); the one real loss is the
box:"card-05" binding.

# Exercise 5 — bare elements

## Bare:
{"id":"sticker","type":"image","start":5000,"end":8000,"source":"images/pumpkin.png"}
Where should it render? My answer while authoring it, before consulting anything: centred in
the frame at natural pixel size, fully opaque. I typed it expecting a preview, not a layout.
- opacity 1, scale 1, rotation 0: no other sane answer.
- position: x=frame.width/2, y=frame.height/2, origin="center" -> centred.
- size: the source's natural pixels (with the ships-its-own-inputs caveat: natural size is
  discoverable only via probe/validate, same status as source duration).

## Deliberately hard to find:
{"id":"badge","type":"image","start":0,"end":65216,"source":"brand/logo-en.png"}
logo-en.png is 68x68 as used in the fixture. Under a top-left default it renders at (0,0) —
UNDER the header chrome: chip-panel (layer 30) covers x 48-420, y 88-172 and sits above the
photo track; a small image at the corner is boxed in by cream panels and near-invisible in a
1080x1920 frame. I would hunt it with frame() for a while. Under a centred default it lands
at (540,960), dead centre of the safe area — impossible to miss.
=> The default that saves the author is CENTRED. Also the fixture votes for it: 19 of 22
text elements sit at x:540 (frame centre), and every Ken Burns needs a centre pivot.
