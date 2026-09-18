---
status: accepted
amends: 0006 (retracts a commissioned check that violates the noise-budget principle), 0011 (validate gains nothing; compare's scope grows), 0012 (retracts its commissioned check and records why), 0032 (compare gains a second drift-shaped hazard, alongside ADR-0036's), 0036 (the destroyed-coincidence precedent this ADR extends)
---

# The group keyframe-time check is retracted from `validate` and reassigned to `compare`

> **Amended by [ADR-0063](0063-compare-drift-checks-keyframe-instant-relationships.md).**
> Designs the coupled-motion-drift mechanics it deferred.

[Ticket #71](https://github.com/MBehtemam/Montaget/issues/71), from
[#12](https://github.com/MBehtemam/Montaget/issues/12). [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)
commissioned a `validate` check — fire when elements sharing a `group` have transform
keyframe times that disagree — as the mitigation for a real hazard: two elements meant to
move as one (a lower-third's bar and its text), desynced by an edit, with nothing in the file
recording they were ever coupled. ADR-0012 itself called this "the mitigation, not a cure"
and booked the underlying hazard as open. Four agents who tried to implement or use it each
found a different failure.

## The evidence

1. **False-positives on every group in the real fixture.** Five separate groups — a photo
   card, three more like it, an eight-element quiz group — all show keyframe-time
   disagreement. `group` is `CONTEXT.md`'s vocabulary unit — a photo, a card, a caption — not
   a motion unit. The check fires on the overwhelming majority of correct, intentional
   documents: disabled on first contact, exactly the false-confidence failure
   [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) already named.
2. **Its scoping is per-property and nothing said so.** An agent wrote the literal
   whole-group reading and had to fix it against a group mixing a `scale`-animated headshot
   with an `opacity`-only caption on an unrelated schedule.
3. **It punishes deliberate staggers.** A three-element end-card entrance at 35600/36300/
   36700 ms — correct, intentional — is indistinguishable to the check from the coupled-delta
   bug it exists to catch.
4. **It's blind to the case closest to its target.** A cross-property hand-off ("property A
   starts exactly where property B ends") gets no opinion at all — verified to fire
   identically before and after an edit that destroys such a relationship, discriminating
   nothing.

## Diagnosis

All four failures trace to one structural fact. The hazard is *desynchronization* — a
relationship that held and stopped holding. "Held" and "stopped" are temporal predicates; a
single document has no time axis to evaluate them against. Failures 1 and 3 are the same miss
from opposite directions: group membership and staggering are both ordinary, legitimate
reasons for keyframe-time disagreement, so a check firing on disagreement fires on the base
rate, not the hazard. Failure 4 is the tell — the check discriminates nothing across the
exact edit that destroys the relationship it was written to protect, which is the operational
definition of a check not measuring its own target.

**This project has already ruled on this shape once.**
[ADR-0036](./0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md)
established that `validate` can report a coincidence *exists*, never that one was
*destroyed* — destruction is the actual hazard, and detecting it needs a before and an after,
which is `compare`'s job (ADR-0032's drift backstop), not `validate`'s single-document,
no-I/O budget. Same-property keyframe-time agreement across elements *is* a coincidence
between timestamps. Routing it anywhere but `compare` would leave the map holding two
incompatible rulings on the identical question.

Resolved by a jury of three independent models (Opus, Sonnet, Haiku) — **2/3, not
unanimous**, and the original ballots were never committed alongside this ADR. Same pattern
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s history already named:
an ADR's evidence has to be committed in a form that can be re-executed, not merely
described. Once the gap was noticed, the jury was re-run from a brief that withholds this
ADR's own reasoning and conclusion, three fresh isolated models, ballots committed in full at
[`docs/research/juries/group-keyframe-check-to-compare/`](../research/juries/group-keyframe-check-to-compare/README.md).
**Opus and Haiku independently land on this ADR's disposition** — retract from `validate`,
reassign to `compare` — largely re-deriving the reasoning below on their own. **Sonnet
dissents**, arguing for exactly the opt-in "motion unit" shape the next paragraph
pre-rejects, without having seen that rejection; its ballot does not independently arrive at
the counter-argument or rebut it, so it stands as live, unaddressed disagreement rather than
a considered dissent. Separately, Haiku — while agreeing with the disposition — recommends
new opt-in schema syntax for declaring motion coupling, which this ADR explicitly declines
below; Opus argues against ever adding one, on the same "protects the wrong population"
reasoning that answers Sonnet.

**Why no single-document formulation survives.** A stricter "motion unit" narrower than
`group` is an opt-in marker wearing a schema hat: it only protects documents whose author
already knew the coupling mattered and declared it — precisely the population that doesn't
have the bug, while a silently-desynced lower-third was authored by someone who declared
nothing. **This is the argument Sonnet's dissenting ballot never sees**, since the re-run
brief withholds it to keep the jury uncontaminated; Sonnet's motion-unit proposal supplies no
answer to it. A near-match heuristic ("times within N ms but not equal") restates the
noise-budget violation as arithmetic: failure 3's stagger is closely-spaced *by design*, so
any N wide enough to catch a real drift also swallows deliberate staggers. An explicit
coupling syntax (a `sync` unit, shared keyframe tracks) is a document-model proposal, not a
`validate` check — if it ships, enforcing exact agreement inside a declared sync unit becomes
a trivial `error` needing no check of this shape; until it ships, `validate` has nothing
sound to build. **Haiku's re-run ballot proposes exactly this syntax anyway**; it is noted,
not adopted — nobody is proposing it here, for the reason just given.

## Disposition

**The check is retracted from `validate`'s plan entirely.** `validate` ships nothing new for
this hazard. The coupled-motion-drift hazard is reassigned to `compare`, joining the
destroyed-coincidence hazard ADR-0036 already parked there: a future `compare` capability
reports when a cross-element, same-property keyframe-time relationship that held in the prior
version no longer holds in the current one. This is structurally immune to failures 1 and 3 —
a stagger that was always a stagger produces no drift to report, and a group that was never
coupled has no prior relationship to break — and it inherits failure 2's scoping requirement
directly: whichever `compare` capability gets built must be keyed per-property, not per-group,
from the start.

**What this does not solve, stated rather than hidden.** `compare` needs a before and an
after. A lower-third that is desynced from the moment it is *authored* — never coupled
correctly in the first place, with no prior correct version to diff against — is invisible to
both tools. `validate` can't flag it without asserting intent no document carries; `compare`
has no earlier state showing the coupling ever held. This hazard stays explicitly open, not
graduated to fog — there is no unticketed question left to sharpen, only an honest limit of
what these two tools can do without a schema field declaring intended coupling, which nobody
is proposing here.

## Consequences

- **`validate` gains nothing.** ADR-0012's commissioned group-keyframe-time check is
  retracted, not built.
- **`compare`'s scope grows by one hazard**, keyed per-property: report when a cross-element
  keyframe-time relationship that held in a prior version stops holding in the current one.
  Not designed in full here — the shape (per-property scoping) is fixed, the mechanics are
  future work, alongside ADR-0036's destroyed-coincidence capability.
- **The first-authoring blind spot is recorded as open, not fog**: neither `validate` nor
  `compare` can catch a coupling that was never correct to begin with, and closing it would
  require a schema-level coupling declaration this ADR does not propose.
- **No schema change. No renderer change.**
