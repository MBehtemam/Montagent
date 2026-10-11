# Findings guide

Each finding explains itself: its text says what is wrong, and an advise-class finding also gives the repair. This guide lists only the findings that fire on correct work, and what to do about each. Act on every finding it doesn't list.

## Caption checks on text that isn't a caption

`R-CAPTION-MIN-DURATION`, `R-CAPTION-NO-AUDIO`, `R-CAPTION-PACE` and `R-CAPTION-REPEAT-DURATION` fire on every text element that doesn't say `caption: false`. Write `caption: false` on titles, typed letters, kinetic words, labels and HUD text, and the four checks skip it. On real captions (words that transcribe speech), leave the field out and act on the findings.

## A mass `R-KEYFRAME-UNREACHED` from a script

When a script fades every mark the same way, `R-KEYFRAME-UNREACHED` fires once per mark, and each is true: a fade that ends exactly on the element's `end` never reaches its last value on a sampled frame, so the mark pops off. The count means one pattern in the generator, not many separate mistakes. Fix it once there, ending the fade on the last sampled frame (`measure` names the nearest one), and the whole count goes. The report prints the code as one counted line, so a distinct finding beside it is not buried.

## `R-EASE-INERT` on a deliberate hold

A hold is two equal keyframes in a row: a title that waits before it leaves, or a pose in a rig baked frame by frame. `R-EASE-INERT` fires on every hold, whichever ease you give it, and that is by design: it makes you confirm that the stillness is meant. When it is, skip the finding and don't spend turns rewriting the hold to quiet it. When the hold ends in an instant change rather than a move, write that change as a `step` to the new value (see the starter's title). That needs no hold, so nothing fires.
