---
status: accepted
amends: 0012 (confirms single-segment entering-ease locality and the closed ease vocabulary hold; neighbour-dependent continuous and auto eases are refused in the runtime evaluator and format schema), 0038 (confirms ease describes strictly the segment entering its keyframe; smooth pass-through across keyframes is expressed through literal slope-matched cubic Bezier handles), 0052 (confirms validate leaves monotonic multi-segment decelerations untouched rather than adding a lint check; intermediate pauses are authorial choice, with the pause trap documented in the motion skill), 0215 (disposes of the Continuous Bezier and Auto Bezier rows from the After Effects keyframe census on map #896), 0217 (extends skills/montagent-motion/scripts/keyframes.py with an auto_bezier pass and documents the slope-matching recipe in skills/montagent-motion/SKILL.md)
---

# Smooth pass-through easing enters as an `auto_bezier` pass and recipe, and the runtime evaluator stays local

[How should smooth pass-through easing across keyframes (continuous and auto Bezier) work?](https://github.com/MBehtemam/Montagent/issues/1061)
on the After Effects map ([#896](https://github.com/MBehtemam/Montagent/issues/896)).

[ADR-0215](0215-after-effects-is-precedent-for-rendered-results-and-capcut-and-premiere-stay-first-for-shape.md)
settled that After Effects is precedent for rendered motion-graphics results while Montagent keeps its
four core invariants ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md):
literal values, closed vocabulary, checkable by `validate`, and exact-string-replace locality).
Under [ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md) and
[ADR-0038](0038-ease-is-required-on-every-non-first-keyframe-record.md), `ease` describes strictly the segment
*entering* a keyframe record from the preceding one. Easing is either one of the closed names (`linear`, `ease`,
`ease-in`, `ease-out`, `ease-in-out`, `step`) or a raw cubic Bezier array `[x1, y1, x2, y2]` where $x \in [0, 1]$
and $y \in \mathbb{R}$.

In After Effects ([#899](https://github.com/MBehtemam/Montagent/issues/899), rows 3 and 4), **Continuous Bezier**
and **Auto Bezier** adjust keyframe direction handles so velocity passes smoothly through an intermediate
keyframe $k_i$ ($v_{i-1} < v_i < v_{i+1}$) without dropping to zero or kinking. In Montagent today, if an author
naively assigns `"ease": "ease-in-out"` to both segments $[k_{i-1}, k_i]$ and $[k_i, k_{i+1}]$, the property
decelerates to a complete stop ($v' = 0$) at $k_i$ before accelerating again.

This ADR decides:
1. Whether smooth pass-through easing enters the runtime evaluator and schema or stays in authoring tools.
2. How the auto-Bezier algorithm is packaged and parameterized.
3. Whether `validate` emits a review check or note on monotonic multi-segment decelerations.

## Decisions

### 1. The runtime evaluator stays strictly single-segment and local; neighbour-dependent "continuous" ease keywords are refused

A neighbour-dependent keyword such as `"ease": "continuous"` or `"ease": "auto"` is **refused** in the format
and runtime evaluator:

1. **Breaks single-segment locality ([ADR-0012](0012-flat-transform-keyframes-carried-by-their-element.md), [ADR-0038](0038-ease-is-required-on-every-non-first-keyframe-record.md)).**
   Evaluating a segment $[k_{i-1}, k_i]$ would require inspecting $k_{i+1}$ (and potentially $k_{i-2}$), turning
   independent segment evaluation into a multi-segment lookahead solver.
2. **Breaks exact-string-replace locality ([ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)).**
   If a segment's interpolation derived dynamically from adjacent records, retiming or editing $k_{i+1}$ would
   silently alter the rendered curve of $[k_{i-1}, k_i]$ without touching any string in that segment.
3. **Breaks `shift` subdivision.**
   `shift`'s SPLIT rule divides a segment using closed cubic Bezier subdivision (`subdivide(ease, fraction)`).
   A segment whose curve depends on external neighbours cannot be split in isolation.
4. **Already solvable with raw cubic Beziers.**
   Because Montagent's raw cubic Bezier `[x1, y1, x2, y2]` admits any $y \in \mathbb{R}$, continuous velocities
   across adjacent segments can be represented exactly as literal Bezier control points:
   $$v'_{\text{in}}(k_i) = \frac{1 - y_{2, i}}{1 - x_{2, i}} \cdot \frac{\Delta v_i}{\Delta t_i} = \frac{y_{1, i+1}}{x_{1, i+1}} \cdot \frac{\Delta v_{i+1}}{\Delta t_{i+1}} = v'_{\text{out}}(k_i)$$

Smooth pass-through easing is therefore purely an authoring task: computing slope-matched cubic Bezier handles
and writing them as literal `[x1, y1, x2, y2]` arrays.

### 2. Auto-Bezier smoothing enters as an `auto_bezier` pass in `keyframes.py` alongside a written recipe in `montagent-motion`

Following [ADR-0217](0217-motion-bakes-and-keyframe-assistants-enter-in-one-keyframes-script-auto-colour-is-refused-and-a-skill-script-with-its-spec-is-a-template.md)'s
unified script architecture, smooth pass-through easing enters in two forms:

1. **`auto_bezier` pass in `skills/montagent-motion/scripts/keyframes.py <project> <spec>`:**
   - Targets an `(element_id, property, [start, end])` slice of existing keyframes.
   - Computes physical slopes $m_i = dv/dt$ at each interior keyframe using standard monotonic cubic spline
     formulation (Fritsch–Carlson / Steffen):
     - If $k_i$ is a local extremum ($(v_i - v_{i-1})(v_{i+1} - v_i) \le 0$), slope $m_i = 0$ (prevents overshoot/ringing).
     - Otherwise, $m_i$ is initialized from the weighted harmonic mean of adjacent secants and clamped to ensure monotonicity.
   - Converts $m_i$ into normalized cubic Bezier control points using the standard $1/3$ time handle length:
     - Incoming segment $i$: $x_2 = 1 - 1/3 \approx 0.666667$, $y_2 = 1 - \frac{m_i \Delta t_i}{3 \Delta v_i}$.
     - Outgoing segment $i+1$: $x_1 = 1/3 \approx 0.333333$, $y_1 = \frac{m_i \Delta t_{i+1}}{3 \Delta v_{i+1}}$.
   - Boundary keyframes: default to zero velocity ($m_0 = m_n = 0$, equivalent to `ease-out` / `ease-in` handles)
     unless specified otherwise in `<spec>`.
   - Replaces the segment eases with raw `[x1, y1, x2, y2]` arrays (rounded to 6 decimal places per ADR-0012)
     and validates with `montagent validate`.
2. **Written slope-matching recipe in `skills/montagent-motion/SKILL.md`:**
   - Explains the physical slope relationship and provides the formulas so an agent working on a 2-segment
     or 3-keyframe transition can calculate matching `[x1, y1, x2, y2]` handles directly without invoking
     an external script.

### 3. `validate` leaves monotonic multi-segment decelerations untouched

No `review` or `note` lint finding is added to `validate` for monotonic sequences with decelerating eases
(e.g. `ease-in-out` into `ease-in-out`):

- **Legitimate artistic staging.** Pausing or decelerating to rest at a waypoint before continuing is a standard
  motion-design choice (e.g. staged reveals, UI pauses at intermediate docking thresholds).
- **Distinction from `R-EASE-INERT` ([ADR-0052](0052-review-check-for-inert-ease-on-held-keyframes.md)).**
  `R-EASE-INERT` fires only when $v$ is identical across keyframes, meaning the ease does literally zero work.
  A monotonic sequence with `ease-in-out` produces real, rendered velocity changes; flagging it would second-guess
  artistic intent and produce noisy false positives.
- **Guidance lives in skill documentation.** The "accidental midpoint pause" pitfall and its solution via
  `auto_bezier` or slope-matched handles are documented in `skills/montagent-motion/SKILL.md`.

## Consequences

- The format schema, Rust keyframe model (`Ease`, `EaseName`), and runtime evaluator (`resolve.rs`) remain
  completely untouched.
- `skills/montagent-motion/scripts/keyframes.py` gains the `auto_bezier` pass for automated tangent smoothing.
- `skills/montagent-motion/SKILL.md` gains the smooth pass-through slope-matching recipe and midpoint pause guidance.
- Implementation graduates to a task ticket on map #896.
