---
status: accepted
---

# Refuse-class findings: a `repair` field, decided once per check, uniform across instances, and non-bypassable

**Ticket:** [#78](https://github.com/MBehtemam/Montaget/issues/78)
**Amends:** [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) (report format),
[ADR-0016](./0016-no-format-version-the-unknown-key-error-is-the-mechanism.md) (names this
design as required and does not take it)

## Decision

Every `error`-class finding carries a `repair` field, independent of its stable code
and of severity:

- `"repair": {"value": ...}` (or equivalent structured content) when the correct fix
  is fully determined by the document, the media on disk, and published rendering
  semantics — **advise-class**.
- `"repair": "none"` when the fix depends on knowing what the author meant, which
  the document does not and cannot carry — **refuse-class**.

This is a **general property of any check**, decided once, by whoever authors the
check, at the moment the check is written — not a special case of retired-key
handling. Retired keys are its first instance, not its category.

**Granularity is per check, not per instance.** If any instance a check can match is
capable of being load-bearing, the check emits `repair: "none"` for **every**
instance it matches, including ones that look safe. There is no per-element
detection attempting to sort the safe-looking instances from the load-bearing ones.

**A refuse-class finding carries a sibling census** — the same inert-fact-grouping
pattern ADR-0006 already uses ("four of five are 1597, one is 1537") — grouping the
affected elements by an observable, document-derived fact (e.g. shared `clip`/geometry
pattern), without ranking or judging which group is correct.

**`render` refuses on any `error`, and a refuse-class `error` has no override.**
Ordinary `error`-class findings may someday gain a bypass mechanism; a refuse-class
finding's block is a stated guarantee that no future flag, force mode, or MCP write
tool may lift. The absence of an escape hatch is what makes "refuse" mean refuse.

**What the agent is told to do:** stop, do not attempt a repair via ordinary file
edit, and surface the finding verbatim to whoever is operating it — outside
Montaget's software boundary entirely, the same posture ADR-0006's `NOT CHECKED`
block already takes toward what the tool cannot know. No in-file escalation field is
introduced. Severity stays at the three levels ADR-0006 already closed; `repair` is
orthogonal to `error`/`review`/`note`, not a fourth level.

## Why

### The evidence this design is built on

[`docs/research/juries/format-versioning/experiment-gravity-fork/`](../research/juries/format-versioning/experiment-gravity-fork/),
already cited by ADR-0016: six agents repaired a file where `gravity` — a field
retired by [ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md) —
appeared on 8 elements. For 6 of the 8, deleting the field was a geometric no-op
(the declared `x`/`y`/`clip` already matched the old `gravity` value). For 2
(`photo-06`, `photo-07`, both `gravity:"bottom"`), deletion alone silently changes
which part of the source is visible — the correct repair also required translating
`y`, inferable only from reasoning about the field's retired semantics, not
readable off the document. **4 of 6 agents noticed the fork and wrote it down as a
real risk. 3 of those 4 shipped the delete-only repair anyway.** Only 1 of 6 produced
a defensible repair.

A separate, earlier experiment (cited in ADR-0016) tested pairing the same error
with a glossary entry describing the retired field's old meaning. Agents with the
glossary solved an analogous fork **0 of 3**; agents given only the bare error
solved it **2 of 3**. The glossary's authoritative-sounding prose led agents to trust
it over reasoning through the actual geometry, and was measurably harmful.

Both experiments point the same way: **the tool volunteering a semantic account of
what changed is worse than the tool saying nothing and refusing.** Neither prose
explanation nor per-instance triage rescues the agent; only refusal does.

### The decision was cross-examined by a three-model court

Three independent jurors (Opus, Haiku, Fable), given the gravity-fork evidence and
the same five design questions, answered blind to each other:

| | Opus | Fable | Haiku |
|---|---|---|---|
| Scope general vs. retired-key-only | general | general | retired-key-only |
| Declaration: field vs. code prefix vs. 4th severity | field | field | field |
| Granularity: per-check uniform vs. per-table vs. per-instance detected | uniform | uniform | uniform |
| Sibling census | yes | yes | yes |
| Non-bypassable | yes | yes | yes |

Four of five questions were unanimous. The scope question split 2–1; the dissent's
own stated reasoning — "generalize later if evidence accumulates" — concedes the
general form as the eventual destination and disputes only sequencing. Since general
scope costs nothing beyond the same per-check decision every juror already requires
(**every** check's author decides its class at authoring time regardless of scope),
this ADR resolves the split toward general.

### Why a field, not a code prefix or a fourth severity level

A fourth severity level is foreclosed by ADR-0006 itself: a jury found readers stop
reliably distinguishing severities past three, which is why `error`/`review`/`note`
is closed. A code prefix (`X-` instead of `E-`) was considered and rejected: ADR-0006
makes the finding code the stable identity shared between the canonical JSON and the
generated prose, and folding repair-class into the code conflates two things that can
change independently — what is wrong, and whether the fix is nameable. A field keeps
codes stable, keeps severity closed, and is directly machine-checkable: an agent (or
any future tooling) can gate on `repair == "none"` without parsing prose or memorizing
a naming convention.

### Why uniform per-check, not per-instance detection

The gravity evidence is the argument: the fact that separates the 6 safe instances
from the 2 load-bearing ones — the old field's source-pixel-dimension inputs — is
**not in the document**, which is the entire reason the field was retired rather than
mechanically migrated. Any per-instance heuristic would have to infer that missing
fact, which is exactly the guess ADR-0016 forbids `validate` from making on the
document's behalf. Worse, a heuristic correct on 6 of 8 instances would train
confidence that then misfires on the 2 that matter — the identical failure mode the
6-agent experiment already produced in agents, not in code. A declarative
key-to-class table was also considered and rejected: it separates the classification
from the check that emits it, inviting drift, and repeats the shape of the
retired-spelling glossary ADR-0016 already measured as actively harmful (0/3 vs.
2/3). The check's own author — the person who wrote the retiring ADR and therefore
the only person who ever knows whether an instance could be load-bearing — is the
correct and only place to decide, once.

This deliberately accepts false refusals: under `gravity`, all 8 elements refuse,
including the 6 that were geometrically safe deletions. That cost is real and is the
correct trade — the alternative distribution is 6 silent successes and 2 silently
wrong renders, and a silently wrong render is the exact failure ADR-0006's whole
`validate` posture exists to prevent.

### Why the sibling census, not silence and not a verdict

ADR-0006 already establishes that a census reports inert, document-derived facts
without stating a repair ("four of five are 1597, one is 1537"). The same pattern
survives the glossary experiment's lesson precisely because it never describes old
semantics — only present geometry. For `gravity`, a census grouping the 8 elements by
their `clip`/geometry pattern (6 sharing one pattern, 2 sharing another) gives a human
or agent somewhere to look without the tool adjudicating which group is right. The
experiment's failure was not that agents failed to *find* the fork — 4 of 6 found
it — it was that finding it did not stop them from guessing. A census narrows
attention; it must not be worded in a way that implies the larger group is the
correct one.

### Why non-bypassable

A stop-and-surface posture with no further guarantee (Q5 option A) is necessary but
was judged insufficient by all three jurors: it makes refuse-class a convention that
erodes the first time an unrelated `--force` or override flag is added to `render`
for a different reason. The evidence this ADR is built on is specifically about an
**unattended agent under pressure to ship** — the same population three of four
agents in the gravity experiment belonged to. An override that exists "for
emergencies" is the override such an agent finds first. Making the absence of a
bypass a stated guarantee, not an implementation accident, is what gives the `repair`
field real teeth.

### Why no in-file escalation field (Q5 option C rejected)

A schema field an agent writes to flag an element as "pending human decision" reopens
exactly the "a schema-level field declaring intended author-meaning" question this
map has already fogged out twice — the coupling and derived-time-signature entries
under the map's "Not yet specified" section. It also invites the precise failure
already demonstrated: an agent writes the marker *and* still ships the guessed
repair, treating the marker as discharging the obligation rather than replacing it.
Designing a real provenance/intent mechanism is future work, not this ticket's.

## Consequences

- Every `error`-class finding's JSON shape gains a `repair` field: a structured
  value when the fix is fully determined, or the literal string `"none"` when it is
  not. The prose renderer states the class in words on every finding, since the field
  is not visible in a bare code quoted in text.
- **ADR-0016's retired-key mechanism is now fully specified**: an arithmetic
  retirement (its repair script, per ADR-0016's existing rule) is advise-class and
  states the fix; a retirement where any instance could depend on author intent —
  `gravity` among them — is refuse-class, `repair: "none"`, applied uniformly to
  every instance of that key, with a sibling census grouping the affected elements
  by observable geometry.
- **`render`'s refusal on `error` gains a stated exception to any future override**:
  whatever bypass mechanism `render` may someday grow for ordinary errors, it must
  not and cannot lift a refuse-class finding.
- **MCP write tools** (per ADR-0006, which already returns findings as a write
  tool's result) surface refuse-class findings the same way — a write that produces
  one is not a soft warning to route around.
- **Severity stays closed at three levels.** `repair` is an orthogonal axis, not a
  fourth level.
- **No `migrate` verb, no in-file escalation field, no retired-spelling glossary.**
  All three were considered here or inherited from ADR-0016/ADR-0015 as measured or
  reasoned harmful.

## Reopening condition

Exhibit a check where per-instance detection of load-bearing-ness is reliably
computable from the document alone (i.e. the discriminating fact the gravity case
lacked is present for some other retired or refuse-class check), and show agents
using it produce fewer defects than uniform refusal. Alternatively, exhibit a
refuse-class finding that genuinely has no legitimate consumer able to supply the
missing intent even outside Montaget's boundary, which would argue for the in-file
escalation mechanism rejected above.

## Not settled here

- The exact wire shape of a structured `repair` value for advise-class findings
  (e.g. whether it is the same shape as the arithmetic-script pointer ADR-0016
  promises, or a richer object) — a presentation detail, not a decision this ADR's
  conclusions depend on.
- Which existing checks besides the retired-key mechanism should be reclassified as
  refuse-class under this general rule. None are reclassified by this ADR; future
  checks and future ADRs amending existing ones make that call individually.
