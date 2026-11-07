---
status: accepted
amends: 0006 (retires the unbuilt third sub-frame alignment note in favor of measure --at and removes dead Class::Note from N-QUANTIZATION)
---

# Retire ADR-0006's sub-frame alignment note and narrow `N-QUANTIZATION` to `review`

[#257](https://github.com/MBehtemam/Montagent/issues/257).
[ADR-0006](0006-validate-reports-facts-and-render-enforces.md) originally sketched `N-QUANTIZATION`
with two `review` escalation conditions — an element or a track gap whose `[start, end)` contains
zero sampled frames at the project's `fps` — plus a third `note`-level idea that would report every
element boundary not falling on an integer multiple of `1000 / fps`.
[ADR-0105](0105-the-sheets-refusals-are-invocation-errors-its-blind-spots-are-not-findings-and-an-unpainted-state-is-quantization.md)
later added a third `review` escalation condition (an unpainted visual state where two boundaries on
different elements collapse onto the same frame).

Meanwhile, [ADR-0035](0035-frame-rate-stays-open-and-measure-reports-the-grid.md) established that at
non-divisor frame rates such as `30` fps (`100/3` ms per frame) or `[24000, 1001]` fps, almost no
integer-millisecond boundary is an exact multiple of `1000 / fps`, and assigned on-demand frame-grid
inspection to `measure --at`. As a result, `checks/time.rs` never emitted a `Class::Note` finding for
`N-QUANTIZATION`, leaving `Class::Note` in `N-QUANTIZATION`'s `CheckSpec.classes` slice in
`crates/montagent-core/src/registry.rs` as an unreachable dead entry.

---

## Decision

1. **Retire ADR-0006's sub-frame boundary alignment note:**
   A boundary that sits between two sampled frames without rounding an element, a track gap, or a
   visual state out of existence is not a `validate` finding. On-demand inspection of which frame
   instant a millisecond timestamp lands on belongs to `measure --at` ([ADR-0035](0035-frame-rate-stays-open-and-measure-reports-the-grid.md)).
2. **`N-QUANTIZATION` is `review`-only (`classes: &[Review]`):**
   `Class::Note` is removed from `N-QUANTIZATION`'s `CheckSpec` in
   `crates/montagent-core/src/registry.rs`. `N-QUANTIZATION` fires exclusively at `Class::Review`
   for its three unpainted-span conditions (an element holding no sampled frame, a track gap holding
   no sampled frame, or a visual state holding no sampled frame under ADR-0105). The `N-` code
   prefix stands unchanged for wire stability.

---

## Consequences

- `crates/montagent-core/src/registry.rs` declares `classes: &[Review]` on `N-QUANTIZATION`.
- Every `Class` listed on `N-QUANTIZATION` in `REGISTRY` is reachable in `checks/time.rs`.
