# Where the glyph baseline sits inside a line's slot — [#59](https://github.com/MBehtemam/Montaget/issues/59)

**Outcome:** half-leading (`baseline_y = slot_centre_y + (ascent − descent) / 2`), read
from the max ascent and max descent across every run on the line — see
[ADR-0029](../../../adr/0029-line-baseline-half-leading.md).

## Method

Twelve jurors, three independent models (Claude Opus 5, Claude Haiku 4.5, Claude
Fable 5.1) each answering all four sub-questions, one juror per (model, question)
pair. Each juror was briefed with the same context block — ADR-0007's slot
definition and worked example, the measured negative-leading case at size 55, and
(for Q1) the existing undocumented prototype convention — and answered blind to
every other juror's ballot. No juror had tool or file access; all reasoning is from
the briefed context alone.

## Q1 — Which convention governs baseline placement within the slot?

(a) CSS half-leading / canvas `textBaseline:"middle"` — `baseline = slot_centre +
(ascent−descent)/2`.
(b) Ascent-from-top — `baseline = slot_top + ascent`.
(c) Descent-from-bottom — `baseline = slot_bottom − descent`.

🗳️ **Juror** (Claude Opus 5, 1M context) — **VOTE: a**

**Reasoning:** The slot is a symmetric abstraction: ADR-0007 defines it purely as
`max_run_size × line_height`, a height with no intrinsic top or bottom bias, and
the worked example places it by its *centre* (y=1537 → 1506.75–1567.25). A baseline
rule should preserve that symmetry, and only half-leading does: `slot_centre +
(ascent−descent)/2` centres the font's own ink box (ascent+descent) on the slot's
centre, so any overflow from negative leading is split evenly above and below
rather than dumped on one edge. That makes the failure mode benign and
predictable — a caption block centred on y stays optically centred on y regardless
of font metrics or line_height — which is exactly the property a spec whose
project file must be "understandable by reading it" needs. It is also the
convention every reader already knows from CSS half-leading and canvas
`textBaseline:"middle"`, so a third renderer can be written from one sentence of
prose and a pointer to well-known prior art, which is the cheapest possible
documentary authority. Finally, it is a tie-break with a free bonus: one of the two
shipped renderers already implements it, so adopting (a) fixes the divergence by
changing one codebase instead of two, while the "no documentary authority"
objection is cured by the very ADR being written here.

**Trade-offs:** (b) ascent-from-top is the simplest to state and matches naive
top-anchored layout, but it discards descent entirely, so with the measured
negative leading every descender hangs below the slot — and the amount it hangs
depends on the font, meaning the same project file at the same size shifts
vertically when the font changes. (c) descent-from-bottom is the mirror image with
the same defect pointed upward. Both are asymmetric rules bolted onto a symmetric
definition, and both make the overflow twice as large on one side as half-leading
makes it on either.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: b**

**Reasoning:** Ascent-from-top is the most predictable and unambiguous formula.
The slot's top edge is an absolute reference point; ascent is a font metric you
can read from the spec; the baseline distance from slot top is always identical.
This prioritizes keeping ascenders — the most visually prominent glyphs — fully
within the slot bounds, which is what viewers notice when text is clipped.

**Trade-offs:** (a) tries to be symmetric but the asymmetry is still there when
overflow occurs, just hidden in the formula, making it harder to predict where
glyphs will actually render without computing it. (c) is the mirror problem, and
top-of-text clipping is more jarring than descenders going slightly low.

🗳️ **Juror** (Claude Fable 5.1) — **VOTE: a**

**Reasoning:** The slot is defined as size × line_height, and line_height is a
ratio *around* the glyph box, not a top- or bottom-anchored offset. When the slot
is smaller than ascent+descent, the only convention that distributes the deficit
symmetrically is half-leading: ascenders and descenders each overflow by the same
~2.5px, so the visual centre of the glyph box coincides with the slot centre —
exactly what an author means by "block centred on y=1537". It also matches the
dominant precedent an agent will already know.

**Trade-offs:** (b) and (c) are simpler but each anchors to one edge and silently
dumps the whole overflow on the opposite side. (b) has one genuine advantage — it
matches "top-of-line = top-of-slot" mental models — but Montaget's runs-on-lines
model treats every line identically, so that advantage does not apply.

**Verdict: 2–1 for (a) half-leading.**

## Q2 — Whose metrics feed the baseline on a mixed-size line?

(a) The largest-size run's font metrics only.
(b) Max ascent / max descent independently across all runs on the line.

🗳️ **Juror** (Claude Opus 5, 1M context) — **VOTE: b**

**Reasoning:** The two rules answer different questions, so they do not have to
share an "author" run. Slot height asks "how much vertical space does this line
reserve?" — ADR-0007 answers it with a single privileged run because the answer
must be one number. Baseline placement asks "where must the baseline sit so
nothing on this line is clipped?" — the honest answer is a constraint over every
run, because glyphs from the small run are drawn on the same baseline and can
still overshoot. Ascent and descent are independent constraints (a font with a
tall ascent and shallow descent is common), so taking max ascent and max descent
separately is the only formula that guarantees no run clips. Option (a) can
visibly clip a small-but-tall-ascent run — a real failure mode with emoji, CJK
fallback, or an icon font mixed into body text at a smaller size. "Inert" means no
hidden computation, not minimal computation; a max over N runs is still fully
readable off the page.

**Trade-offs:** (a) is cheaper and yields a tidier "the largest run owns the line"
narrative, but the tidiness is cosmetic while the clipping is real. The
"independent computation" objection is weaker than it looks — slot height already
uses `size × line_height` rather than metrics at all, so (a) does not actually
unify the two formulas, it only makes them share an input.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: a**

**Reasoning:** Consistency with ADR-0007's established principle that the largest
run is the privileged "author" of slot dimensions. Letting that same run's metrics
determine baseline placement creates a cohesive model where one run fully owns the
vertical behavior of the line — predictable and understandable without hidden
calculations.

**Trade-offs:** (b) would be more typographically sophisticated and match how
browsers handle inline mixed-size text, but it introduces a conceptual split: slot
height follows "largest run only," baseline follows "max across all runs" — two
different "authors" of the line's vertical behaviour.

🗳️ **Juror** (Claude Fable 5.1) — **VOTE: a**

**Reasoning:** ADR-0007 already commits to a single privileged run for slot
height; the baseline is a position *inside* that slot, so it should derive from
the same run — otherwise the slot's size and its internal anchor come from
different inputs. In practice the divergence is rare: ascent and descent scale
with size, so a smaller run out-ascending the largest run requires fonts with
markedly different vertical metrics at nearly equal sizes.

**Trade-offs:** (b) is closer to how browsers build a line box, but browsers get
away with it because their line box *height* also grows from the union — here the
slot height is frozen by ADR-0007, so (b) is only half of the browser model: it
shifts the baseline for the small run while the slot itself never does, trading
one clipping risk for another.

**Verdict: 2–1 for (a) largest-run-only**, with Opus dissenting on a named
correctness scenario (a smaller run's taller ascent clips against the slot) that
neither majority juror addressed.

## Q3 — Does this need a schema change?

(a) Zero schema impact — formula only, in an ADR.
(b) Add an authoring-time override field (e.g. `baseline_offset`).

🗳️ Opus, Haiku, and Fable each voted **(a)**, unanimous 3/3. All three independently
invoked the same precedent: the project has already rejected fields (`speed`,
`fit`) whose correct value isn't computable from in-document inputs, and
`baseline_offset` is structurally identical — uncomputable without eyeballing a
render, and it rots silently if the font changes.

## Q4 — Should `measure` report the resolved baseline?

(a) Yes — add `baseline_y` per line to `measure`'s output.
(b) No — leave `measure` unchanged.

🗳️ Opus, Haiku, and Fable each voted **(a)**, unanimous 3/3. All three cited the
same settled precedent: `measure`/`frame` exist because agents can't otherwise
verify text layout, and forcing hand re-derivation of the ADR formula (or
full-frame pixel inspection) is exactly the failure mode that loop was built to
close.

## Resolution

Q3 and Q4 stood unanimous. Q1 and Q2 split 2–1 in opposite directions. Resolved by
the human: half-leading for Q1 (the majority), and **max-across-all-runs (the
minority) for Q2** — the same symmetry argument that decided Q1 requires the
stronger rule in Q2, since a formula that can silently clip an unprivileged run's
glyphs reproduces exactly the asymmetry Q1 was chosen to avoid. See
[ADR-0029](../../../adr/0029-line-baseline-half-leading.md) for the full decision
and consequences.
