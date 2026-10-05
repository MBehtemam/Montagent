---
status: accepted
---

# A repeat does not enter, and a stagger enters only across the units of one text element

> **Amended by [ADR-0151](0151-letter-spacing-is-an-animatable-element-field-and-a-stagger-is-a-units-block-on-one-text-element.md).** Its three binding requirements are
> discharged: the stagger is a `units` block on the text element, a unit is singled out by a run
> carrying `unit` whose values replace the derived ones, a letter is a non-whitespace grapheme
> cluster, and `query --at` lists every unit.

[#673](https://github.com/MBehtemam/Montagent/issues/673), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663). Motion work asks for many copies of
one thing, or one animation started many times. Two shapes were weighed: a **repeat**, one
element that draws as N offset copies, and a **stagger**, one animation whose start is offset
per unit. Per-letter animation waits on this ruling.

**Precedent.** Row 5 of the precedent research
([#664](https://github.com/MBehtemam/Montagent/issues/664)): CapCut has none that could be
found. Premiere's Clone effect duplicates an element "with offset timing", which is a partial
match. A parametric repeater is outside the reference class, so
[ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)
asks for an owner-accepted prototype. ADR-0145 also settled where the argument sits: a construct
such as `{"count": 5, "offset_ms": 80}` passes the literal-values test, because its copies come
from fixed arithmetic, so the argument is made under the exact-string-replace test.

## The cases

The ruling is judged against three scenes:

- **(a)** The letters of a title come in one after another, 40 ms apart, each fading in and
  rising 20 px.
- **(b)** Five cards in a UI list enter 80 ms apart, each sliding in. This is the shape of
  [#463](https://github.com/MBehtemam/Montagent/issues/463).
- **(c)** A 6×4 grid of dots, each copy offset in position and perhaps in time.

Case (a) cannot be written correctly today. One element per letter breaks shaping, kerning and
line-breaking, and no amount of agent effort repairs that. Cases (b) and (c) can already be
written correctly as separate elements. The cost is file length and a few edits per change,
not correctness.

## The decision

### 1. A repeat does not enter

No element draws as more than one copy. Cases (b) and (c) are **served by written-out
elements**: one element per card or dot, each with its own `id`, its own transform and its own
keyframes. Changing the grid's spacing is 24 exact-string edits, each to a unique element.

### 2. A stagger enters only across the units of one text element

A stagger enters as part of per-letter animation: one text element's keyframes run once per
unit, each unit started a fixed offset after the one before. The per-letter ticket specifies
it in full, and it inherits three requirements from this ruling:

- **Replace, not add.** A unit is singled out the way ADR-0145 already shows for a letter: by
  splitting it into its own run. A split run's own offset or keyframes *replace* the derived
  stagger for that run. They are never added on top of it, because an added offset would make
  the run's value relative to another value.
- **The unit is named exactly.** The ticket says what one letter is (a grapheme cluster or a
  character), and what a word and a line are.
- **The tools report per unit.** `query --at` and the checks report timing per unit, so what
  `validate` sees reaches inside the element.

### 3. A stagger across elements does not enter

Case (b) stays written out with literal `t` values on each card. To retime it, the agent edits
each card's keyframes. `format.md` says to anchor each replace on the card's `id`, because a
string such as `"t": 80` repeats across cards.

## The four tests

| Invariant | Written-out copies (cases b, c) | A stagger inside one text element (case a) |
| --- | --- | --- |
| Literal values | Every value is written. | The offset is a literal, and each unit's start is fixed arithmetic on it. |
| Closed vocabulary | Nothing new. | The per-letter ticket names the units. |
| Checkable by `validate` | Every copy is an element the checks already iterate. | Required: `query --at` and the checks report per unit. |
| Exact-string replace | Every copy has its own strings. | A unit is singled out by splitting its run, and its own values replace the derived ones. |

## Why

**What is in the file is what is on the timeline.** The checks (`R-OFF-CANVAS`, `query --at`,
the contact sheet's keyframe tiles, anchors) iterate the elements written in the file, and an
agent trusts them for exactly that reason. A repeat draws things the file does not name.

**A repeat fails the fourth test, whichever way it is shaped.**

- *With a per-index override list*, copy 3 is changed by position, not through a string of its
  own. Lowering `count` from 7 to 5 leaves an override for index 6 pointing at nothing. The
  agent made that orphan with a correct-looking one-literal edit, and only a new check could
  catch it.
- *As a stated expansion*, where the schema defines the N elements a repeat stands for, the
  orphan goes away, but every tool must expand the same way before it reads. Copies get
  addresses such as `dot#3` that appear in no string the agent can search for. A tool that
  expands differently, or not at all, makes what the tools check differ from what renders.

Either way, the cost is paid in front of every check, for case (c) alone.

**The stagger inside a text element costs none of that.** Its units are already in the file as
text, the format already shows how to single one out, and it adds no link between elements. A
stagger across elements would be the first element whose timing derives from another's. The
format refuses that everywhere else: no element's transform is relative to another's.

**Agreement.** A court of three jurors (Opus, Sonnet and Fable), each judging as the agent that
writes and edits the file, agreed with every part of this ruling. They added the replace-not-add
rule, the per-unit reporting requirement and the `id`-anchored retime note. The owner ruled with
the Judge's read.

## Considered and refused

- **A repeat with a per-index override list.** Refused: overrides orphan silently when `count`
  drops.
- **A repeat as a stated expansion.** Refused: an expansion step in front of every check, and
  copies with no `id` in the file, for one rare case.
- **A stagger construct across elements.** Refused: it is an element-to-element timing link,
  and it saves the agent only a few edits.
- **A write-time expander**, a `shift`-like verb that writes literal staggered times into the
  file. Refused: `shift` earns its place by protecting an invariant (slack keeps its size), and
  a stagger expansion protects none. It is authorship, and the tool surface of
  [ADR-0011](0011-tool-surface-reads-checks-renders.md) (reads, checks, renders) stands. If
  staggers across elements later prove common, a per-`id` option on `shift` is the cheaper path
  to weigh, not a construct in the file. This is recorded, not decided.

## Consequences

- `CONTEXT.md` gains **Stagger**, and **Repeat** joins the rejected terms.
- Per-letter animation leaves the map's fog as a grilling ticket carrying the three
  requirements in §2, followed by a prototype ticket for the entry test. That prototype replaces
  this ticket's own follow-on prototype, because a stagger with no per-letter animation has
  nothing to render.
- `format.md` and the feature map gain the `id`-anchored retime note for staggers across
  elements, through the per-letter hand-off spec.
- Premiere's Clone effect is named here, so it does not reopen the question.
