# Juror: Haiku

Given [QUESTION.md](./QUESTION.md) in isolation — no repository access, no access to the
other jurors' ballots, no access to my own recommendations from the earlier grilling round.

---

🗳️ **Juror** (Claude Haiku 4.5) — **Q1 VOTE: (c)**

**Reasoning:** The incident demonstrates that the convention's critical importance to the exact-string-replace workflow is incompatible with relying on unchecked folklore. A tool accidentally violating the convention silently broke downstream agents' edits, even though it was technically legal. Since the convention is load-bearing for the entire agent-editing model, it must be enforced rather than hoped for.

**Trade-offs:** Enforcement adds tooling complexity and may reject technically-valid files that violate the convention. However, the alternative — another incident — risks project instability. Option (b) is ruled out because documentation-only failed to prevent the incident. Option (a) guarantees future incidents as tools evolve.

🗳️ **Juror** (Claude Haiku 4.5) — **Q2 VOTE: Single global fixed sequence**

**Reasoning:** A single canonical order applied to all elements regardless of type is simpler to specify, implement, and reason about. It reduces cognitive load for tool authors and creates a uniform contract that all elements follow, even if some types don't use all fields.

**Trade-offs:** Per-type ordering could be more ergonomic for elements with very different field sets, but the simplicity gain from a global order outweighs the ergonomic cost. Consistency and predictability matter more in a foundational constraint.

🗳️ **Juror** (Claude Haiku 4.5) — **Q3 VOTE: Both**

**Reasoning:** The compliance check belongs in `validate` because it unconditionally catches violations on every file, preventing the incident pattern from recurring silently. It should also be in `fmt --check` to let developers preview what a format pass would change before applying it. This dual-layer approach catches violations early and gives operators visibility.

**Trade-offs:** Checking only in `fmt --check` is opt-in and will be missed by agents that don't remember to invoke it (the incident proves this). Checking only in `validate` without a preview mode in `fmt` means developers can't see what formatting will do before it lands. Both layers together add redundancy but ensure safety at the cost of complexity.

🗳️ **Juror** (Claude Haiku 4.5) — **Q4 VOTE: Yes, split into check and rewrite modes**

**Reasoning:** The incident happened because a tool's destructive actions (key reorder + pretty-printing) were invisible until landing in production data. Having `fmt --check` show what would change — before any rewrite happens — is standard practice in formatter design and gives operators a chance to review large diffs before they're committed. The dual materialization of defaults plus key reordering described in #61 makes preview mode essential.

**Trade-offs:** Splitting adds one more command for developers to remember. However, the cost of a silent large destructive change is much higher than the cost of requiring a preview step, especially when format operations can interact in unexpected ways.
