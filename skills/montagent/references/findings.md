# Findings guide

Each finding explains itself: its text says what is wrong, and an advise-class finding also gives the repair. This guide lists only the findings that are **expected noise**: ones that fire on correct work, so you can skip them. Act on every finding it doesn't list.

## Caption checks on text that isn't a caption

<!-- workaround: #458 · replaced by: caption checks that fire only on captions -->

`R-CAPTION-MIN-DURATION`, `R-CAPTION-NO-AUDIO`, `R-CAPTION-PACE` and `R-CAPTION-REPEAT-DURATION` fire on every text element, because Montagent can't yet tell a caption from other text. On titles, typed letters, kinetic words, labels and HUD text they are noise, and a typing effect can raise dozens of them. On real captions (words that transcribe speech), act on them.

## `R-EASE-INERT` on a deliberate hold

A hold is two equal keyframes in a row: a title that waits before it leaves, or a pose in a rig baked frame by frame. `R-EASE-INERT` fires on every hold, whichever ease you give it, and that is by design: it makes you confirm that the stillness is meant. When it is, skip the finding and don't spend turns rewriting the hold to quiet it. When the hold ends in an instant change rather than a move, write that change as a `step` to the new value (see the starter's title). That needs no hold, so nothing fires.
