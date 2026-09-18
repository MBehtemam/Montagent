---
status: accepted
---

# Slack is invariant by default; `shift` refuses to silently change it, `compare` is the backstop

> **Amended by five later ADRs.** Read them before relying on anything below.
>
> - [ADR-0039](0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md)
>   — compare gains a second drift-shaped hazard, alongside ADR-0036's
> - [ADR-0047](0047-shift-releases-slack-by-boundary-instant-pairs.md) — settles the
>   release mechanism it deferred
> - [ADR-0051](0051-word-alignment-is-external-validate-and-compare-catch-drift.md) —
>   extends validate's and compare's fact vocabularies
> - [ADR-0063](0063-compare-drift-checks-keyframe-instant-relationships.md) — states
>   precisely where slack-drift's coverage of coincidence ends
> - [ADR-0066](0066-boundary-coincidence-cluster-drift-is-its-own-predicate.md) — closes
>   the pairwise gap named but not mechanized there

Four agents were given the same instruction — "item-07's narration was
re-recorded and is 800 ms longer; make the edit so the file stays legal" — and
produced four different repairs. All four pass every existing check
([ADR-0004](./0004-tracks-as-constrained-lanes.md),
[ADR-0005](./0005-absolute-integer-milliseconds.md)): no overlaps, source
ranges consistent, `duration` equal to the max end. They differ by up to
920 ms of trailing silence in the rendered video —
[#64](https://github.com/MBehtemam/Montaget/issues/64).

The file distinguishes elements from each other but says nothing about the
space *between* them. A 520 ms gap between two sentences turned out to be
bit-identical across four analogous items, and three of four agents recovered
it without being told. A "lead-out" — the distance from the last narration's
end to the photo's end, 912–920 ms across the same four items — was recovered
by none of them; readings split between "an intended design pattern" and
"slack, safe to destroy." Nothing in the document says which reading is
right, so the format itself is silent at the exact point where four
independent readers were not.

## Slack

**Slack** is the timeline distance from one element's boundary — its `start`
or `end` — to the nearest thing that follows or precedes it: a neighbouring
element's boundary (in the same track or a different one), or, for the
project's own last boundary, the derived `duration`. [Gap](../../CONTEXT.md)
is the special case where that neighbour is empty track — slack additionally
names the lead-out case a gap can't reach, because the photo track has no
uncovered stretch at all; the space in question is between two *different*
tracks' boundaries.

**Every slack currently in the file is invariant by default.** Its size is
content, the same way a `speed` value or a `start` is content — not a
default the renderer supplies, but a number the file asserts and an edit
must preserve unless it says otherwise.

This inverts the polarity of the `sequence` label
([#20](https://github.com/MBehtemam/Montaget/issues/20)) and the `kind`-field
option ADR-0006 rejected: those failed because an *unmarked* item was
indistinguishable from "not yet decided" and "deliberately free," so an
optional marker manufactured false confidence for the majority case it never
touched. An opt-in invariant marker would reproduce exactly that shape here.
Defaulting to invariant removes the ambiguity in the other direction:
silence about a slack never means "undecided," it always means "protected."
Only an explicit release is a claim, and a claim is what an edit like #64's
actually is — "800 ms belongs to the narration, not to what follows it" is a
decision, not an default that should be free to make.

The default also costs less than it looks like it does. `shift(path, at,
delta, scope)` already moves every time at or after `at` by `delta`, which
preserves every slack downstream of the edit by construction
([ADR-0005](./0005-absolute-integer-milliseconds.md)). #64's four repairs
diverged because agents reached *past* that behaviour — they chose to absorb
the 800 ms into a neighbouring slack rather than propagate it. Naming slack
as invariant does not add a new constraint to the format; it states, for the
one point where `shift` alone doesn't decide the outcome, a rule the tool
already mostly obeys.

## `shift` refuses rather than silently absorbing

When an edit would change the size of a slack — narrowing or widening the
distance between the affected boundary and its neighbour — `shift` refuses,
naming the slack and its two boundary instants, in the same shape as the
existing straddler refusal
([ADR-0005](./0005-absolute-integer-milliseconds.md)): *"1120–2040 after
item-07's new end would change the 913 ms lead-out to photo-07's end;
nearest boundaries are 61402 and 62322."*

This was chosen over the alternative of letting `shift` auto-adjust to
preserve every slack's size, because the write-tool invariant already values
tools whose output is exactly what their arguments describe (`speed` names
itself as the sole free variable in ADR-0020's invariant rather than being
silently re-derived elsewhere; `shift` moves times, not a tool that also
reaches into unrelated elements to compensate). A `shift` that silently
rewrote a neighbour's boundary to hold a slack constant would be doing that
— useful in the common case, and unpredictable in exactly the case an agent
most needs to trust its output. Refusal is louder and cheaper to reason
about: the agent sees the constraint, decides, and the decision is in the
diff.

To make an edit that does change a protected slack, the agent releases it
explicitly — naming the boundary instant being widened or narrowed, in the
same "timestamp, never a field name or id" shape the write-tool invariant
already requires of `shift`'s other arguments. The exact call shape (a
`release` argument on `shift`, versus a distinct explicit-slack write) is not
settled by this ADR and is deferred to a follow-up ticket once the schema
for slack itself needs to exist as more than a computed distance.

## `compare`, not `validate`, is the backstop

This project already treats a raw file edit as a legitimate authoring path
that bypasses every tool, `shift` included
([ADR-0011](./0011-tool-surface-reads-checks-renders.md)). A slack that
silently drifted under a hand edit needs a check that can still see it.

[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) settles
which tool that is, on its own stated organising principle: *"every check
that compares this document to itself, it passes — only checks that compare
it to something outside it find anything."* `validate` compares the document
to itself and to media on disk; it has no access to what the file used to
say, so it cannot tell a slack that was always 600 ms from one that used to
be 913 ms — both are, on their own, internally legal. The tool that compares
against a prior version already exists for exactly this reason
([#10](https://github.com/MBehtemam/Montaget/issues/10),
[ADR-0011](./0011-tool-surface-reads-checks-renders.md)): **`compare`
reports a slack whose size changed between the two versions it was given**,
the same way it reports any other timeline change. `validate` gains nothing
new here — asking it to remember a prior value would be the second
materialised representation ADR-0005 already rejected, this time of the
file's own edit history instead of its time model.

## Consequences

- **`shift` gains a refusal case**: an edit that would change an existing
  slack's size is refused and reported, in addition to the existing
  straddler refusal.
- **The release mechanism's exact shape is deferred** to a follow-up ticket,
  once slack needs to be nameable rather than only computed and reported.
- **`compare` is confirmed as the tool responsible for catching drift a raw
  edit introduces**, not `validate` — a scope clarification for
  [ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) and
  [ADR-0011](./0011-tool-surface-reads-checks-renders.md), neither of which
  had reason to draw this line before slack existed as a concept.
- **`shift`'s straddler-refusal message format is reused** for the slack
  refusal, rather than inventing a second reporting shape.
