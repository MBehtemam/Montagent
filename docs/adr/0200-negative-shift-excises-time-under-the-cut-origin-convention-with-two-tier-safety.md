---
status: accepted
amends: 0011 (§3: shift verb supports signed delta), 0012 (§destructive clause), 0032, 0047
---

# Negative shift excises time under the cut-origin convention with two-tier safety

[#979](https://github.com/MBehtemam/Montagent/issues/979), [#986](https://github.com/MBehtemam/Montagent/issues/986), [#988](https://github.com/MBehtemam/Montagent/issues/988).

## Context

`shift` is defined in Montagent as *"the one edit that is arithmetic rather than authorship, and therefore the one Montagent performs instead of the agent"* ([CONTEXT.md](../../CONTEXT.md)). In v0.2.0, `shift` was restricted to strictly positive offsets (`delta > 0`). Non-positive `delta` was deferred because forward shift inserts time monotonically via `SPLIT`, while excising time cannot be reduced to `SPLIT` run backwards: intermediate keyframes inside the cut are dropped, durations can invert, and moving clips earlier can cause track collisions (`E-TRACK-OVERLAP`).

However, forbidding negative `delta` blocks parity with fundamental non-linear editing (NLE) operations: **Ripple Delete**, **Close Gap**, **Extract**, **Shortening Ripple Trims**, and **Ripple Speed Changes** ([`docs/research/premiere-gap/editing.md`](../research/premiere-gap/editing.md)). Introducing a dedicated verb (such as `ripple_delete`) would burn an unnecessary tool slot against the context budget of [ADR-0009](0009-rust-host.md) and [ADR-0011](0011-tool-surface-reads-checks-renders.md).

## Decision

1. **Signed `delta` and Cut-Origin Convention (Convention 2)**:  
   The `shift` MCP tool and CLI verb accept a signed integer `delta: i64`:
   - For `delta > 0`: `at` is the insertion boundary. Time interval $[at, at + \delta)$ is opened. Elements and keyframes at or after $at$ shift right by $\delta$.
   - For `delta < 0`: `at` is the excision origin. Time interval $[at, at + |\delta|)$ is excised from the timeline. Elements and keyframes at or after $at + |\delta|$ shift left by $|\delta|$. Elements and keyframes before $at$ remain untouched.
   - For `delta == 0`: Refused as a no-op (`E-SHIFT-ZERO`).

2. **Two-Tier Safety Contract (`delete: bool`)**:  
   To prevent unconsented data loss, `shift` accepts an explicit opt-in boolean `delete: bool` (default `false`):
   - **Tier 1: Safe Gap Closure (`delete: false`, Default)**:  
     The excised interval $[at, at + |\delta|)$ must be completely empty of elements and keyframes across the affected tracks. If any element or keyframe exists within $[at, at + |\delta|)$, the call refuses immediately with `E-SHIFT-DESTRUCTIVE`. Releasing the closed gap's slack via `release: [[at, at + |delta|]]` is required under [ADR-0047](0047-shift-releases-slack-by-boundary-instant-pairs.md).
   - **Tier 2: Destructive Extraction / Ripple Delete (`delete: true`)**:  
     Elements wholly contained within $[at, at + |\delta|)$ are deleted. Keyframes within $[at, at + |\delta|)$ on time-invariant straddlers are permanently dropped. Under [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md), `shift` generates an explicit receipt enumerating every destroyed element ID and dropped keyframe (property name, original timestamp, and value).

3. **Exhaustive Refusal Matrix**:  
   `shift` validates the entire project pre-flight and refuses before making any modifications if any of the following conditions occur:
   - `E-SHIFT-UNDERFLOW`: Shifting would produce an element or keyframe timestamp $t < 0$.
   - `E-TRACK-OVERLAP`: Moving an element left pulls it into temporal overlap with an earlier element on the same track ([ADR-0004](0004-tracks-as-constrained-lanes.md)).
   - `E-ELEMENT-COLLAPSE`: An edit would cause an element to have non-positive duration ($end \le start$).
   - `E-SHIFT-STRADDLE`: A time-based element (`video` or `audio`) straddles the cut boundaries ($start < at < end$ or $start < at + |\delta| < end$). Time-based cuts require agent authorship to declare source media alignment.
   - `E-SHIFT-DESTRUCTIVE`: Elements or keyframes exist in $[at, at + |\delta|)$ when `delete` is `false`.
   - `E-SHIFT-SLACK`: An unreleased gap or lead-out is altered without being named in `release` ([ADR-0032](0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md), [ADR-0047](0047-shift-releases-slack-by-boundary-instant-pairs.md)).
   - `E-SHIFT-UNITS-WINDOW`: Slicing into a text element's active unit stagger animation window ([ADR-0151](0151-text-stagger-is-a-timed-window-over-units-and-a-keyframe-inside-it-is-refused.md)).
   - `E-LAYER-TIE`: Moving an element earlier creates a temporal and spatial collision with another track sharing the same resolved layer ([ADR-0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md)).

4. **Keyframe Preservation & Boundary Hold Synthesis**:  
   - Keyframes on downstream elements ($start \ge at + |\delta|$) shift isomorphically ($t' = t - |\delta|$), preserving relative intervals, animation speeds, and cubic bezier control points bit-for-bit.
   - For time-invariant straddlers during destructive extraction (`delete: true`), keyframes inside the cut are dropped, a boundary keyframe is synthesized at $t = at$ holding the evaluated curve state at $at$, and entering segments receive a `step` ease (`Ease::Named(EaseName::Step)`), collapsing identical adjacent holds.

## Consequences

- Agents can execute gap closures, ripple deletions, and timeline extractions in a single arithmetic tool call.
- Context budget is preserved: no new MCP verb is added to the 10-tool surface.
- Author data cannot be silently destroyed: deletion requires `delete: true` and produces an itemized receipt.
