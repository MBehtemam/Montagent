---
status: accepted
amends: 0086 (adds last-drawn-frame with optional integer offset to t_from's closed rule set on keyframe records)
---

# `t_from` gains the `last-drawn-frame` provenance rule

[#319](https://github.com/MBehtemam/Montagent/issues/319).
[ADR-0086](0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md) introduced
`t_from` on keyframe records with two rules (`element-start` and `after-previous`) and stated that a
third rule would enter only when a concrete class of keyframes could not be expressed by those two
and drifted under ordinary project edits.

Because element time ranges `[start, end)` are half-open ([ADR-0005](0005-single-timeline-integer-ms-half-open-ranges.md))
and the frame grid samples at `⌊n × 1000 / fps⌋` ms ([ADR-0035](0035-frame-rate-stays-open-and-measure-reports-the-grid.md)),
a keyframe placed at `t == end` is never sampled (`R-KEYFRAME-UNREACHED`,
[ADR-0071](0071-an-unreached-keyframe-at-or-past-end-is-a-review-finding.md)). Authors therefore
place terminal keyframes on the last sampled frame strictly before `end` (for example, `t = 966` for
`end = 1000` at `30` fps, `t = 960` at `25` fps, or `t = 959` at `[24000, 1001]` fps). Neither
`element-start` nor `after-previous` can express that derivation: when an element's `end` is trimmed
or the project's `fps` changes, a terminal keyframe stamped with `after-previous` either stays frozen
at the old duration or lands off the new last drawn frame.

---

## Decision

1. **Third rule in `t_from`'s closed set (`last-drawn-frame`):**
   `t_from` on any keyframe record (including the first record of a list, inside `FirstDerivation`
   as well as `Derivation`) accepts:
   ```json
   {"rule": "last-drawn-frame"}
   ```
   or, with an optional integer millisecond offset (default `0`):
   ```json
   {"rule": "last-drawn-frame", "offset": -33}
   ```
2. **Exact integer derivation:**
   Given the enclosing element's `end` and the project's `fps`, let `last` be the last sampled
   timeline frame strictly before `end` (`exact::frame_before(end, fps)`), painted at
   `base = exact::instant_of(last.frame, fps) = ⌊last.frame × 1000 × den / num⌋`. The expected keyframe
   instant is `base + offset.unwrap_or(0)`.
3. **Validation (`R-DERIVED-T`) and schema rules:**
   - Writing `"ms"` on `"last-drawn-frame"` is refused with `E-SCHEMA` naming `"offset"` (`ms` belongs
     only to `after-previous`).
   - Writing a non-integer or `null` `"offset"` is refused with `E-SCHEMA`.
   - When a keyframe's `t` disagrees with `base + offset.unwrap_or(0)`, `validate` emits `R-DERIVED-T`
     (`error`, `Advise`) stating the element's `end`, the project's `fps`, and the re-derived integer
     `t`.
4. **Motion keyframe generator support (`skills/montagent-motion/scripts/keyframes.py`):**
   Operations in `keyframes.py` accept `"end": "last-drawn-frame"` or
   `"end": {"rule": "last-drawn-frame", "offset": <int>}`, resolve the terminal instant in exact
   integer rational arithmetic (`last_drawn_frame_ms`), and stamp `"t_from"` on the terminal
   keyframe record.

---

## Consequences

- `crates/montagent-core/src/model/keyframe.rs` adds `LAST_DRAWN_FRAME` and
  `Derivation::LastDrawnFrame { offset: Option<i64> }`.
- `crates/montagent-core/src/checks/derived.rs` checks `Derivation::LastDrawnFrame` against
  `exact::frame_before(end, fps)` and `exact::instant_of`.
- `schema/montagent.schema.json` publishes the three-branch `Derivation` union and two-branch
  `FirstDerivation` union (`element-start` and `last-drawn-frame`).
