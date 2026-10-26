# The Montagent project format: nested sequences

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the `nest` element that groups child elements into
an isolated offscreen surface on the absolute timeline. Like the rest of the format docs, every
rule here is an accepted decision in the ADR series, cited inline by number, and the ADR is right
where the two disagree.

## Nested sequences (`nest`)

- **A `nest` is a bounded container element rendering to an isolated surface** (ADR-0201):
  it groups child `elements: [...]` to receive a combined transform, opacity, blend mode, or
  effects chain without touching the parent canvas or reading back from it.
- **`width` and `height`** define the exact pixel boundaries of the isolated offscreen buffer.
  Child elements render into this local coordinate space `(0, 0, width, height)`.
- **Absolute timeline clock invariant** (ADR-0005, ADR-0166, ADR-0201 §2):
  children of a `nest` do not have private or relative clocks. Every child element's `start`,
  `end`, and keyframe `t` are stated in **absolute project timeline milliseconds (`ms`)**.
- **Temporal containment** (`E-NEST-TEMPORAL-BOUNDS`):
  all child elements must start at or after the nest's `start` and finish at or before the
  nest's `end` (`child.start >= nest.start && child.end <= nest.end`). Extending beyond the
  bounds is statically refused.
- **Transforms & compositing**:
  a `nest` carries standard 2D transforms (`x`, `y`, `scale`, `rotation`, `origin`, `opacity`,
  keyframes) and blend modes. The offscreen buffer is drawn onto the parent canvas at `(x, y)`
  in a single feed-forward operation, preserving static prefix caching (ADR-0195).
- **Depth and cycle limits**:
  circular nest references are forbidden via DAG cycle detection (`E-NEST-CIRCULAR`), and
  nesting depth is clamped to a maximum of 4 (`E-NEST-DEPTH`). IDs within nests must be unique
  across the entire document (`E-ID-DUPLICATE`, ADR-0019).
- **Transitions between nests (ADR-0243)**:
  `nest` elements are admitted as valid visual targets (`from` / `to`) for transitions
  (`crossfade`, `wipe`, `slide`, `push`). During transition windows, nested scenes render to
  offscreen surfaces and composite as atomic texture quads, eliminating intra-scene bleed,
  coordinate tearing, and manual intermediate ProRes pre-rendering passes.

