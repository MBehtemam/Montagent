---
status: accepted
amends: 0012 (settles what it left silent: presence of `ease` on non-first records)
---

# `ease` is required on every non-first keyframe record; absent only on the first

> **Amended by [ADR-0082](./0082-a-keyframe-list-must-be-written-in-ascending-t.md)**:
> ascending `t` becomes a schema rule, making this ADR's positional presence rule and
> clock order the same statement by construction — closing the divergence
> [#270](https://github.com/MBehtemam/Montagent/issues/270) found.

> **Amended by [ADR-0052](./0052-review-check-for-inert-ease-on-held-keyframes.md)**,
> which designs the `review`-level lint this ADR named as the acknowledged cost but
> did not design: `R-EASE-INERT`, firing when consecutive keyframe records hold an
> identical `v` (whole-value, exact) while still carrying an `ease`.

[Ticket #70](https://github.com/MBehtemam/Montagent/issues/70), from
[#12](https://github.com/MBehtemam/Montagent/issues/12). [ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md)
legislated the first keyframe record exhaustively — `ease` there is a schema error, naming
the entering-convention rather than an ignored field — and said nothing about absence on any
other record. Four agents hit that silence independently and resolved it three different ways.

## The evidence

- One agent wrote `"ease":"linear"` on every non-first record defensively, including **10
  inert eases on hold segments** — records whose value doesn't change, so nothing is actually
  easing — purely because the rule was unstated.
- One agent **omitted** `ease` on its hold records, then caught it in self-audit and
  reclassified its own correct-by-its-own-logic output as *"likely a schema violation, 10
  occurrences"* — the silence didn't just produce variance, it produced a false positive that
  burned the agent's own attention.
- One agent wrote `"ease":"linear"` on four constant-hold records, explicitly reasoning
  *"purely because omitting it is undefined."*

Same silence, three resolutions, from agents reading the identical ADR. This isn't cosmetic:
SPLIT step 7 collapses runs of three or more records with **identical `ease`**, a rule that
cannot execute correctly while "absent" and `"linear"` aren't known to be the same value.

## The three candidates, and why two die

**Optional, meaning "unspecified" — the renderer's own choice — is rejected outright, not
deferred.** It violates this project's file-as-truth invariant directly: two byte-identical
files could render differently depending on which renderer, or which renderer *version*,
opened them, since nothing in the document pins the behavior down. The schema already treats
`ease`'s meaning as load-bearing at the first record (rejecting it there because there is no
prior segment to describe); declining to define its meaning everywhere else is incoherent on
the same list. **This is a schema error if spelled out explicitly** (e.g. an author or a
future proposal writing a literal `"ease": "unspecified"` or documenting reliance on
implementation-defined behavior), not merely a discouraged style — so the option cannot be
resurrected by silence a second time.

**Optional with a published default (`linear`) is rejected.** It is the elegant-looking
option and was seriously weighed: files get shorter on hold segments, and it matches what one
of the four agents actually assumed. But a default is still a fact every reader — agent,
validator, renderer, `shift`'s SPLIT implementation — must independently know and correctly
apply; the file itself does not display it. Concretely, it means SPLIT step 7's "identical
`ease`" comparison must normalize absent-vs-`"linear"` correctly in every one of those places,
and if any one drifts, two byte-identical files can silently diverge in rendered behavior —
the exact failure file-as-truth exists to prevent. A syntactic rule (is the field present or
not, checkable with zero domain knowledge) is strictly cheaper and safer than a semantic one
(what does absence mean, checkable only by consulting external knowledge every consumer must
carry).

## The decision

**`ease` is required on every keyframe record except the first, where it remains a schema
error.** Presence is a pure function of position in the list: never on record 0, always on
every subsequent record. No default value is ever defined, remembered, or kept in sync with
SPLIT's equality check — there is never an absent value to normalize, because absence outside
the first record is itself a schema error.

Resolved by a jury of three independent models (Opus, Sonnet, Haiku), unanimous 3/3.

**The acknowledged cost.** Hold segments — where `v` doesn't change between two keyframes, so
nothing is actually easing — still carry an explicit `ease`, typically `"linear"`, that
describes no real motion. This is semantically inert ceremony, and the jury's dissent-in-
agreement flagged the honest remedy: not a schema hole, but a future **review-level lint**
that flags an `ease` written on a segment where `v` is unchanged, so an author is told the
value is inert without the schema needing two shapes of correctness. Not designed here;
recorded as fog.

## Consequences

- **Schema**: every non-first keyframe record's `ease` field is required; a missing `ease` on
  a non-first record is a schema error, exactly symmetric with the existing first-record rule.
- **No default value is published or needed anywhere** — not in the schema, not in `validate`,
  not in `shift`'s SPLIT rule, not in the renderer.
- **SPLIT step 7's "identical `ease`" collapse needs no normalization step**: both records
  being compared always carry an explicit value.
- **"Unspecified, renderer's choice" is a schema error if spelled out**, not merely
  undocumented behavior — closing the option off rather than leaving it revivable by silence.
- **One fog entry graduates conceptually but isn't ticketed here**: a future `review`-level
  check for an inert `ease` (one written on a segment whose `v` doesn't change) is named as
  the correct answer to the ceremony cost, not designed in this ADR.
