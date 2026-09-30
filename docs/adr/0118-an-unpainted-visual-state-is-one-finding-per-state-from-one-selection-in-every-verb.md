---
status: accepted
amends: 0105 (the `N-QUANTIZATION` a visual state raises is specified — one finding per state, its field names, and that it fires for every unpainted state, including one an element or a gap already explains), 0094 (section 1's "filters members to visual types" is read as dropping `type: "audio"`, the one element type that is not visual)
---

# An unpainted visual state is one finding per state, from one selection, in every verb

[#437](https://github.com/MBehtemam/Montagent/issues/437), a ticket under spec
[#486](https://github.com/MBehtemam/Montagent/issues/486). ADR-0105 §5 gave `N-QUANTIZATION` a
third condition, a visual state the grid never paints, and left `validate` silent on it. #437
teaches `validate` the same fact and lands the selection of visual states that `frame`'s range
mode ([#488](https://github.com/MBehtemam/Montagent/issues/488)) will reuse.

Building it made four observable choices that no ADR spells. Under ADR-0031 each is a spec gap
until an ADR ratifies it, and #486 says so directly: *"The build must not invent silently."*
The finding is about to have a second emitter, and `compare` diffs on a finding's identity
(ADR-0006), so these are ratified now rather than when `frame` ships.

## Decision

1. **One selection.** The visual states are `query`'s cut list with members of
   `type: "audio"` dropped and equal neighbours re-merged, computed by one function,
   `query::cuts::visual_states`. It reuses the cut list's own merge. `validate` calls it, and
   `frame`'s range mode is to call it. Neither verb filters for itself.
2. **"Visual" means "not audio".** ADR-0094 §1 says *"filters members to visual types"*. The
   element types are `audio`, `ellipse`, `image`, `rect`, `text`, `transition` and `video`, so
   dropping `audio` is exactly that filter today. `video` carries sound but is visual.
   `transition` is visual too, since it changes the pixels over its window, and it adds no cut
   of its own: ADR-0059 makes its range the intersection of the two elements it bridges. An
   element with no `type` or an unknown one is kept. A new non-visual type must change this
   line.
3. **One finding per unpainted state**, built by one function,
   `checks::quantization::unpainted_states`, over a range: `[0, extent)` for `validate`, the
   requested range for `frame`. It carries the registered template's three fields and four
   more:

   | field | value |
   | --- | --- |
   | `fps` | the project's rate |
   | `changed` | always `2`, the state's own two boundaries |
   | `detail` | the prose: interval, presence set, what enters and leaves at each boundary, and the painted frames either side |
   | `from`, `to` | the state's half-open interval, in ms |
   | `present` | the visual presence set, as element ids in document order |
   | `boundaries` | two entries, opening then closing: `at`, with `entering` and `leaving` as lists of `{element, track}` |

   What changes at a boundary is read off the neighbouring state. At the edge of the range it
   is read off the visual state just outside, so a state that closes the document still names
   what leaves there.
4. **It fires for every unpainted visual state, even one the first two conditions already
   explain.** An element no frame falls in, or a gap in one track that no frame falls in, is
   also a visual state no frame paints. That document gets two findings: ADR-0006's element or
   gap finding, and this state finding.

## Why

### One selection, because one fact needs one identity

ADR-0105 §4 chose a shared code over a `frame`-only one, so the fact has one identity *"the day
`validate` learns to see it"*. A shared code is not enough by itself. If two verbs cut the clock
at different places, they report different states under the same code, and `compare` sees a
change that never happened. So the selection is one function, the finding is one function, and
a cross-verb test in `tests/cross_verb.rs` pins both to `query --from --to` filtered by hand.

### "Not audio", because a wrong merge is invisible

A `type` allow-list would have to change for every new visual type. If it didn't, that type's
elements would drop out of the presence set, and states that differ only in them would merge
away. A merged state gets no tile and no finding, so nothing shows that it was lost. Keeping
unknown types errs toward a split, and a split state is visible on a sheet. An unknown `type` is
a schema error in `validate` anyway.

### One per state, because the sheet skips per state

ADR-0105 §4: *"Only `no-grid-frame` entries also raise a finding"*, one entry per skipped run.
If `validate` folded every state into one finding, as ADR-0006's first two conditions are
folded, the same state would be one finding from `frame` and a clause inside another finding
from `validate`. ADR-0099 bounds a report with many of them, so per-state findings do not flood
a report.

### Every state, because a partial rule re-splits the identity

ADR-0105 §4 describes the new case as two boundaries *"on different elements"*. It would have
been possible to raise the state finding only when neither earlier condition fires. **This was
rejected**, for the reason decision 1 exists. The sheet has no such exception: it skips every
unpainted run and raises a finding for each. A `validate` that stayed silent on some of those
states would disagree with `frame` about them, which is the split ADR-0105 was written to
prevent. The two findings name different subjects, an element or a gap on one side and a
combination never on screen on the other, so neither is a copy of the other.

## Costs, recorded honestly

- **A vanished element or a sub-frame gap now reads as two `review` findings.** Three existing
  tests in `tests/time.rs` were changed to expect both. The real fixture has zero instances, so
  the cost falls only on documents that already have a quantization defect.
- **`changed` is a constant on this path.** It stays because the registered template reads it.
  Dropping it would need a second template for one code.
- **`checks::quantization` now depends on two verbs' helpers**, `render::extent` and
  `render::instant_of`. Moving them into `crate::exact` is left to #488, which needs them too.
- **ADR-0105's evidence script changed.** Its ninth claim checked that `validate` was silent,
  and it now checks the closing of that gap. The claim ADR-0105 made was true when it was
  accepted; the script checks what is true now.

## What this ADR does not decide

- Anything about the sheet itself: its tiles, labels, `skipped[]` shape, or how `frame` renders
  more than three of these findings. That is #488 and ADR-0099.
- Whether ADR-0006's element and gap findings should become per-element. They stay one
  aggregate finding.
