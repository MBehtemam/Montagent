---
status: accepted
amends: 0094 (adds a caller-selected multi-instant mode alongside visual-state range sheets without reopening count-based `--n` sampling), 0097 (`--at` on the CLI and `at` on MCP accept two or more strictly ascending instants to draw a contact sheet), 0098 (tiles on a caller-selected multi-instant sheet carry `<index> <instant>ms` with no boundary offset or entrant id), 0125 (`sheet` carries `instants: [i64]` in place of `from`/`to` and `rule.name: "caller-selected-instants"` when drawn from multiple `--at` instants)
---

# 244. Multi-instant `--at` lists draw caller-selected contact sheets

Date: 2026-11-03

[#1143](https://github.com/MBehtemam/Montagent/issues/1143).

## Context

`frame --from <MS> --to <MS>` ([ADR-0094](0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md), [ADR-0097](0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md)) draws one contact sheet of a time range by tiling every visual state at its first painted frame, and refuses with `E-SHEET-OVERFLOW` once the range exceeds the 140 px tile-width refusal (30 portrait tiles, [ADR-0095](0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md)).

On a long multi-scene project (such as the 41-second, 6-scene sizzle video), the full timeline holds far more than 30 visual states, so `frame --from 0 --to 41000` is refused by design. When an agent wants a single visual overview across specific moments in the project (for example, one representative instant per scene, or four key moments across a transition), it previously had to make $N$ separate `frame --at <MS>` calls instead of receiving one composited contact sheet. Meanwhile, ADR-0094 §5 and [ADR-0106](0106-the-sheets-opt-ins-are-keyframes-and-infill-ceiling-and-a-keyframe-tile-is-sampled-where-its-change-first-paints.md) §3 explicitly rejected a uniform count-based `--n` sampler because a count spends the same tiles on a 200 ms interval and a 20 s one while disguising a sparse sample as coverage of a range.

## Decision

1. **Multiple `--at` instants draw a contact sheet on both surfaces**:
   - On the CLI, `--at <MS>` is repeatable (`--at 1000 --at 5000`) and accepts comma-delimited values (`--at 1000,5000`).
   - On MCP, `frame`'s `at` parameter accepts either a single integer (`"at": 1000`) or an array of integers (`"at": [1000, 5000]`).
   - Passing **one** instant runs single-frame mode (`frame` + `query --at` blocks at half scale by default) exactly as before. Passing **two or more** instants draws a contact sheet of those exact instants.

2. **Strictly ascending, non-negative instants (`E-INVOCATION`)**:
   - A multi-instant `--at` list must satisfy $0 \le t_1 < t_2 < \dots < t_k$.
   - Any negative instant, duplicate instant, or out-of-order instant is refused with `E-INVOCATION` (exit 3) before the project file is opened, matching [ADR-0082](0082-a-keyframe-list-must-be-written-in-ascending-t.md)'s ascending-time rule and ADR-0094's rule against silently reordering or dropping what the caller asked for.

3. **Each tile paints the exact requested millisecond `t`**:
   - Unlike range sheets (which snap visual-state boundaries to the project's `fps` grid), a caller-selected instant `t` is rasterized at exact millisecond `t` through `Painter` and `query::at::answer(&document, t)`, then resampled with mipmaps into its tile cell.
   - Consequently, tile $i$ is a pure function of `frame --at <t_i>` and its presence set is `query --at <t_i>`'s stack.

4. **Two-field tile labels and the `caller-selected-instants` `sheet` record**:
   - Because caller-selected instants are not triggered by a visual-state boundary, each tile's gutter label carries two fields: `<index> <instant>ms` (e.g., `1 1000ms`), drawn in `#F2F2F2` on `#1A1A1A` at one sheet-wide type size floored at 8 px ([ADR-0098](0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md), [ADR-0128](0128-the-tile-label-is-fitted-at-one-size-per-sheet-and-names-the-topmost-entrant.md)).
   - The answer emits `frame: null`, `query: null`, and a `sheet` object carrying:
     - `instants: [i64]` (in place of `from` and `to`),
     - `rule: { name: "caller-selected-instants", version: 1, sentence: "one tile per caller-requested instant, painted at the exact millisecond asked for" }`,
     - `picture`, `reader_check` (quoting tile 1's label), `rasterized`, `sources`, `fonts`, and `blind_to`,
     - `provenance[]` entries with `{ index, instant_ms, label, why: "requested", class: "instant", present, not_painted, painted_partially }`,
     - `classes: { run: 0, keyframe: 0, infill: 0, instant: k }`.
   - Range-only fields (`from`, `to`, `skipped`, `audio_boundaries_dropped`, `coverage`, `keyframes`, `infill`) are omitted.

5. **Flag composition and `E-SHEET-OVERFLOW`**:
   - Combining multiple `--at` instants with `--from`/`--to`, `--keyframes`, `--infill-ceiling`, `--crop`, or `--full` is refused with `E-INVOCATION` in the precedence order of [ADR-0125 §4](0125-the-sheets-first-build-spells-its-record-and-its-refusals.md).
   - `--png` and `--json` behave identically to range sheets.
   - Grid sizing uses [`sizing::size_tiles`](../../crates/montagent-core/src/verbs/frame/sizing.rs) (180 px tile-width target, 140 px tile-width refusal, 8 px served type floor). Passing more instants than fit on one sheet is refused with `E-SHEET-OVERFLOW` (exit 3), reporting `states` (the number of requested instants) and `fits` so the caller can split its list into batches of at most `fits`.
