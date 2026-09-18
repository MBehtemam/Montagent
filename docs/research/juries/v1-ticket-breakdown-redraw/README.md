# Jury: the v1 ticket breakdown, second draft

The follow-up to [`../v1-ticket-breakdown/`](../v1-ticket-breakdown/README.md), which refuted the
first draft. Three jurors (Opus, Sonnet, Fable), blind to each other, given the redraw plus one
question:

> **Which of the first court's findings did this redraw fail to absorb, and what is newly wrong
> that was not wrong before?**

A single question rather than the first court's four, because the structural questions had
already been answered 3/3 and re-asking them would have burned three agents to reconfirm.

## Verdict: refuted, 3/3

| | absorbed | partial | missed |
| --- | --- | --- | --- |
| Fable's tally of the first court's 33 findings | 19 | 8 | 5 (+1 resolved upstream) |

The graph is materially better than the first draft — `probe` on the spine owning FFmpeg
resolution, `preview` behind `render`, `compare` independent of the resolver, audio mixing and
the `fonts` surface as real tickets, slack and layer resolution owned rather than assumed. None
of that was disputed. What follows is what survived attack anyway.

## The finding all three reached independently

**The redraw was corrected from the court's summaries rather than from the ADRs the court
cited**, and wherever the two differ it followed the summary. That is the same wrong-source error
the first court diagnosed, one level up — and it is worth recording precisely because the author
had just been told about it.

The sharpest instance is `R-BOX-SLACK`. The first court struck the caption-checks-to-text-engine
edge on Fable's reading of ADR-0058 (*"with no I/O"*), and this repository's own README recorded
that ruling as governing. The redraw then pulled `R-BOX-SLACK` into its own ticket and re-blocked
it on the text engine — **the same edge under a new number, reversing a ruling nobody noticed had
been made.**

Its stated justification was wrong twice over, and both halves were checkable in one read:

- ADR-0028's block height is `ceil(size × line_height × line_count)`. **There is no font term in
  it**, and under ADR-0008's no-auto-wrap rule `line_count` is the author's own `\n` count, which
  a font substitution cannot change.
- ADR-0058 *already measured* that **7 of the fixture's text elements fire this check** and
  accepted it. So "it may fire on the fixture" is not a risk; it is the recorded, intended
  behaviour, and #168's *"a check that fires on the fixture is wrong unless an ADR says
  otherwise"* is satisfied — an ADR says otherwise.

Both errors came from one mistake: assuming *text metrics* means *font metrics*. Most of this
format's text arithmetic is deliberately **declared**, not measured. [#186](https://github.com/MBehtemam/Montaget/issues/186)
was filed on the wrong premise and has been corrected rather than closed — the width term *is*
font-dependent (ADR-0014 parks it `UNCHECKED`), and the falsification test genuinely cannot treat
text as golden while the fixture and its reference are in different typefaces.

## The structural defect: a circular dependency

**Fable, and the worst single error across all three rounds.** ADR-0060's layer-tie check
*"samples across the elements' shared time range, not just their rest extents"* and explicitly
rejects a static check, because two same-layer elements can be disjoint at rest and swept into
overlap by a pan or zoom. So it needs the keyframe resolver — which the redraw blocked on the
very ticket containing the layer-tie check. The graph deadlocks.

Fixed by splitting layer/anchor **resolution** (which the resolver needs) from the layer-tie
**check** (which needs the resolver).

## Ceremonial edges, each verified against its ADR

Every one of these was drawn from a plausible-sounding dependency that the source ADR denies:

| edge | what the ADR actually says |
| --- | --- |
| caption checks → time checks | ADR-0054: *"No file is read; this is a pure document-level check"* |
| `R-VISUAL-GAP` → layer resolution | ADR-0018 is a per-group **time-union** of audio against visual. No rect, no layer — and it explicitly *rejected* area-awareness |
| `R-OFF-CANVAS` → fitted extents | ADR-0044 uses the **declared** rect, resolved across keyframes |
| `fonts` surface → text engine | ADR-0057's licence gate and attestation read no font metrics |

`R-VISUAL-GAP` is the one worth dwelling on. The redraw described it as *"a check about `group`
and declared rects"* — language appearing in **no ADR**, apparently inherited from a juror's
shorthand about a different, now-dissolved ticket bundle. The author then drew a dependency on
the layer resolver *from his own wrong description*. Sonnet's framing: the sharpest case of the
redraw reproducing its own diagnosed failure mode.

## Misdescribed ADRs

Five, each claimed as implemented by a ticket that misstates it:

- **ADR-0015** — the rasterizer ticket says it resamples "through `clip`/`fit`". ADR-0015: `fit`
  *"never executes. No renderer reads it."* Its only consumer is `validate`.
- **ADR-0062** — the audio ticket puts the `loop` wrap in `render`. ADR-0062: *"purely
  `validate`-facing … `render` and every other tool are unaffected."*
- **ADR-0021** — the partial-render ticket cites a 60 s-under-2-minutes budget. ADR-0021
  describes that exact figure as *"written for 1080×1920/30 and never re-derived"* and defers its
  replacement. **#168's story 60 inherits the same superseded number.**
- **ADR-0018** and **ADR-0058**, above.

## Still missed from the first court

- `fmt --check` and `LAYOUT` remain one predicate in two tickets with no edge. ADR-0041: *"there
  is exactly one place the rule lives, not two."* Unchanged from the first draft.
- Story 53 (*"the renderer opens nothing outside the declared font chain"*) is still filed under
  `measure`, where it cannot be demonstrated. It is a renderer property.
- ADR-0010's two-arm `rust-rasterizer` golden-frame oracle — which ADR-0010 calls *"not
  optional"* — still has no owning ticket.
- The `frame` and render budgets still have no CI home.
- ADR-0051's two `validate`-side `highlight` checks (out-of-parent-range, sibling-window overlap)
  are claimed by no ticket; only its `compare` half landed.

## The encode arm

**Opus.** The whole-video falsification — the only test capable of falsifying the format, and the
only test of the audio ticket — was blocked on a chain containing neither text drawing nor
effects. It would have rendered the fixture's 22 text elements and its one `mask` as *absent* and
compared that to the published MP4. Meanwhile `render` claimed to run *"the identical check
engine"* while blocked on three of the roughly ten tickets that produce `error`-class findings —
first-draft language carried forward unexamined.

## Sizing

All three held that the document-model ticket, left whole at nineteen ADRs while the text engine
split into four and the rasterizer into three, was defended by a rationalisation rather than an
argument: ADR-0041's field-order freeze constrains **insertion order**, not delivery. Fable named
the thing that should actually come out of it — the permissive, presence-preserving parse path
ADR-0042 forces, which the redraw had double-owned between two tickets anyway.

## Coverage

Four index rows have no ticket — ADR-0003 (scope), ADR-0022 (a relabelling that changes no
decision), ADR-0027 (vector sources out of scope), ADR-0037 (*no* authoring tool ships). All four
are legitimately non-implementing, but the redraw claimed a complete walk of the index and did not
record them as deliberate. Recording an exclusion is what distinguishes a decision from an
oversight.

## Outcome

Twenty fixes, each verified against the cited ADR text rather than against a ballot, applied to
produce the published ticket set. **No fourth court was convened**, on the reasoning that an ADR
is expensive to unwind and near-permanent while a ticket is cheap to edit and gets corrected the
moment an agent picks it up and finds a blocker missing — and that the durable findings from
three rounds (two ADR contradictions, thirty missing stories, the typeface divergence, and
committed evidence that had silently stopped reproducing) are already on `main`.

The risk that accepts is recorded rather than hidden: the author was refuted three times running,
twice for the same reason, and is now applying the third round's corrections unattended. If the
published graph turns out to carry another ceremonial edge or another misdescribed ADR, this note
is where to start.
