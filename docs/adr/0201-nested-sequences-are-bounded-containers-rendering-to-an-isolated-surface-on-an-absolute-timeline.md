---
status: accepted
amends: 0001 (rejection of local clock stands; shared spatial container admitted), 0166 (§2: nest container design), 0195
---

# Nested sequences are bounded containers rendering to an isolated surface on an absolute timeline

[#981](https://github.com/MBehtemam/Montagent/issues/981), [#986](https://github.com/MBehtemam/Montagent/issues/986), [#989](https://github.com/MBehtemam/Montagent/issues/989).

## Context

[ADR-0166](0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md) clarified that Adobe Premiere Pro nested sequences and CapCut compound clips share a spatial coordinate transform, and that [ADR-0001](0001-flat-element-list.md)'s rejection of `scene` applied exclusively to private/relative local clocks, not shared spatial transforms. Furthermore, research in [#981](../research/premiere-parity/multi-element-adjustment.md) showed that free-floating adjustment layers must be rejected because they demand mid-stack canvas read-backs (`surface.image_snapshot()`), stalling GPU memory bandwidth (~33 MB/frame at 4K) and invalidating the static prefix cache ([ADR-0195](0195-the-static-prefix-cache-holds-invariant-layers-at-the-bottom-of-the-stack.md)).

Instead, grouping multiple elements to receive a combined transform, blend mode, or filter effect requires a bounded container element that renders strictly feed-forward without touching the parent canvas.

## Decision

1. **Declarative Schema (`type: "nest"`)**:  
   A new element kind `nest` enters `schema/montagent.schema.json`:
   - Encloses child `elements: [...]`.
   - Declares explicit bounding dimensions `width` and `height` in integer pixels.
   - Carries standard 2D affine transforms (`x`, `y`, `scale`, `rotation`, `origin`, `opacity`, keyframes).
   - Carries an optional `effects: [...]` array and `blend_mode` applied to the composite of its children.
   ```json
   {
     "type": "nest",
     "id": "intro-card-group",
     "track": "graphics",
     "layer": 10,
     "start": 1000,
     "end": 6000,
     "width": 1280,
     "height": 720,
     "x": 320,
     "y": 180,
     "effects": [
       {"name": "blur", "radius": 10}
     ],
     "elements": [
       { "type": "video", "id": "card-bg", "start": 1000, "end": 6000, ... },
       { "type": "text", "id": "card-label", "start": 1500, "end": 5500, ... }
     ]
   }
   ```

2. **Absolute Timeline Clock Invariant ([ADR-0001](0001-flat-element-list.md), [ADR-0166](0166-capcut-and-premiere-nest-with-a-transform-and-adr-0001-refuses-the-clock-not-the-shared-space.md))**:  
   Children of a `nest` do **not** have private, relative clocks. Every child element's `start`, `end`, and keyframe `t` are stated in **absolute project timeline milliseconds (`ms`)**.
   - `validate` statically enforces that all child elements are temporally contained within the parent nest:
     $$\forall c \in \text{children},\quad c.\text{start} \ge \text{nest}.\text{start} \quad \land \quad c.\text{end} \le \text{nest}.\text{end}$$
   - Any child extending outside the parent's temporal boundaries is statically rejected (`E-NEST-TEMPORAL-BOUNDS`).
   - Reading remains free ([ADR-0005](0005-absolute-integer-milliseconds.md)): an agent or author inspecting any child element knows immediately when it is active without traversing parent container hierarchies.

3. **Feed-Forward Offscreen Isolation**:  
   A `nest` renders strictly feed-forward:
   - When a `nest` is active at frame time $T$, an isolated offscreen Skia surface (`Canvas::layer()`) of size $\text{width} \times \text{height}$ is instantiated with a transparent clear (`[0, 0, 0, 0]`).
   - Children active at $T$ are rasterized into this buffer in their respective layer order.
   - If the nest specifies `effects`, the filter chain is evaluated on the offscreen surface.
   - The finished buffer is drawn onto the parent canvas at position $(x, y)$ using the nest's transform, opacity, and blend mode in a single operation.
   - **Zero Canvas Read-Back**: The parent canvas is never read back, maintaining feed-forward streaming and preventing 4K memory stalls.
   - **Static Prefix Preservation**: Layers below the `nest` remain cached under ADR-0195 without invalidation.

4. **Static Validation Rules**:  
   `montagent validate` enforces:
   - **Cycle Prevention**: Circular nest references (e.g. nest $A$ containing nest $B$ containing nest $A$) are forbidden via directed acyclic graph (DAG) cycle detection (`E-NEST-CIRCULAR`). Maximum nesting depth is clamped to 4 (`E-NEST-DEPTH`).
   - **ID Uniqueness**: Element IDs inside nests must be unique across the entire project document ([ADR-0019](0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)).
   - **Track Containment**: Children inside a nest cannot specify a `track` property; track allocation is held by the parent `nest` element on the sequence timeline.

## Consequences

- Group transforms, combined blurs, and compound layouts enter Montagent without violating the four invariants of ADR-0145.
- Relative clock hazards that broke earlier models are completely prevented by enforcing absolute timeline timestamps.
- Memory bandwidth remains bounded to the declared container dimensions rather than whole-screen 4K framebuffers.
