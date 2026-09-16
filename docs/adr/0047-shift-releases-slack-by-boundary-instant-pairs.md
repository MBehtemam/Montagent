---
status: accepted
amends: 0032 (settles the release mechanism it deferred)
---

# `shift` releases slack through a `release` argument naming boundary-instant pairs, enumerated individually

[ADR-0032](./0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md)
settled that every slack is invariant by default and that `shift` refuses an
edit that would change one, but left the release mechanism's exact shape
undecided — slack didn't exist as a nameable concept until that ADR, so no
prior ticket had reason to specify it. Three candidates were on the table
going in: a `release` argument on `shift` itself, a distinct write specific
to slack, or something neither prior jury had considered
([#102](https://github.com/MBehtemam/Montaget/issues/102)).

## Decision

**`shift` gains a `release` argument: a list of boundary-instant pairs, each
naming one specific slack in full. Every slack the edit would otherwise
change must appear in the list, named individually — no bulk or "release
everything this call touches" form exists.**

```
shift(path, at, delta, scope, release=[[61402, 62322], ...])
```

1. **The release lives on `shift`, not a separate write.** A released slack
   with no edit that consumes it is either meaningless or itself a silent
   invariant violation — nothing else in the tool surface can legally act on
   a "released" slack except an edit touching the same boundary, so a
   standalone `release_slack` write would either mutate no state (breaking
   the mental model that every write changes something) or persist a
   `released` marker that outlives the edit it was reasoned about, directly
   contradicting ADR-0032: slack size is content, not a flag the system may
   later spend on the system's behalf. `release` is part of the single
   complete value describing one edit, in the same shape the write-tool
   invariant already requires of `shift`'s other arguments — not a bare
   field name, not an element id.

2. **Every affected slack is named individually; no bulk release.** A
   `release="all"` (or equivalent) form would compute its own meaning from
   the file's current state rather than state it — two identical calls
   against a drifted project would mean different things, and a reviewer
   reading the diff or call log could not tell which slacks were consented
   to. That is the same silent-absorption failure ADR-0032 rejected for
   `shift`'s default behaviour, relocated into an opaque flag with the
   agent's signature on it instead of removed. The friction lands on the
   machine, not the human: `shift`'s refusal already enumerates every
   threatened slack and its boundary pair, so releasing is transcription,
   not inspection — the agent pastes the reported pairs back into `release`.

3. **A slack is named by its full boundary-instant pair, never by the single
   moving instant.** A bare timestamp is contextually unambiguous only
   within the specific call that produced it — read in isolation, in a
   diff, or reused a session later, it depends on inferring which of the
   (up to two) slacks touching that boundary is meant from `delta`'s sign
   relative to `at`. That inference is exactly what the write-tool invariant
   exists to eliminate. The failure case is not exotic: a boundary that is
   simultaneously the far edge of one slack and the moving edge of an
   adjacent one is the ordinary shape of a tightly packed timeline, not an
   edge case reserved for pathological input. The pair also buys a
   correctness check a single instant cannot: `shift` validates each named
   pair against the file and refuses if it does not currently bound a real,
   protected slack, catching a stale read, a wrong-element target, or an
   off-by-one before the write lands. And the pair costs nothing to look
   up — it is exactly what the refusal message already prints.

4. **`shift`'s straddler-refusal message format is reused** for reporting
   which slacks a proposed edit would change, per ADR-0032's stated
   consequence — `release` consumes precisely the pairs that message
   reports, so a rejected call and its accepted retry share one vocabulary.

## Evidence

Three courts, one per question, three jurors each (Claude Opus, Claude
Sonnet 5, Claude Haiku 4.5 — independent, blind to each other's ballots).

- **Mechanism** (argument on `shift` vs. distinct write): 2/3 for the
  `shift` argument. The dissent (Haiku) raised a real atomicity concern
  about a two-call sequence — but on inspection that concern argues for the
  majority position, not against it: a release with no atomic consuming
  edit is the hazard the majority's reasoning names, not a feature a
  two-call design preserves.
- **Multiplicity** (enumerate vs. bulk release): unanimous 3/3 for
  individual enumeration, explicitly declining to weight the ticket-drafter's
  stated lean toward it as evidence. One juror recommended, as
  implementation guidance rather than a schema change: the refusal should
  list every threatened slack up front (not just the first it finds), and a
  release argument naming an instant-pair the edit would not in fact change
  should itself refuse, so a release list cannot be padded speculatively or
  carried over stale from an earlier attempt.
- **Identification** (single moving instant vs. boundary pair): unanimous
  3/3 for the pair, each juror independently naming the shared-boundary
  ambiguity case as decisive rather than a corner case to discount.

## Consequences

- `shift` gains a fifth argument, `release`, taking a list of
  `[instant, instant]` pairs. It does not gain a dependency on any new
  schema field — a released slack is not persisted; releasing and editing
  happen in the same call.
- `shift`'s refusal message, on encountering a protected slack the edit
  would change, must enumerate every such slack this call would affect (not
  only the first), each as the same boundary-instant pair `release` expects
  back.
- A `release` entry naming a pair that does not currently bound a real,
  protected slack for this edit is itself a refusal — `release` cannot be
  populated speculatively or reused stale across calls.
- No new tool-surface verb. ADR-0011's tool surface stands unchanged in
  count; `shift`'s signature is the only thing that grows.

## Not settled here

- Whether a narrower range-based release (naming a time span rather than
  enumerating every pair inside it) is worth adding later if wide,
  intentional restructurings prove common enough that per-slack enumeration
  becomes the friction point — raised by one juror as a possible future
  escape hatch, not proposed as part of this decision.
