# Jury brief: the group keyframe-time check

You are one of three independent jurors (different models) answering the same question in
isolation. You will not see the other jurors' answers, and they will not see yours. Answer
from the evidence given here — do not search the internet or assume context you were not
given.

## Project context

Montaget is a declarative, agent-authored video editor. A project file is a flat list of
`elements` (image, video, audio, text, rect, ellipse) placed on `tracks` (named lanes with an
integer `layer`; children of one track may not overlap in time). Every visual element carries
a flat transform — `x`, `y`, `origin`, `scale`, `rotation`, `opacity` — in absolute integer
pixels on the project's one clock. Any transform property may be a literal value or a list of
keyframe records `{"t","v","ease"}`, `t` in absolute timeline milliseconds.

Elements may share an optional `group` string. `group` is **purely a vocabulary/organizational
label** — "this photo, its caption card, and its caption text are conceptually one item" — and
is **render-inert**: nothing about layout, z-order, or timing derives from group membership.

`montaget validate` answers exactly one question: **is this project file internally legal, and
does it agree with the media on disk?** It runs unconditionally (no fast mode, nothing
scoped out), with **no I/O beyond probing referenced media**, on a single document with no
prior version to compare against. Its own governing principle, established for this project:
**an opt-in check is worth nothing, and a noisy check is worse than no check** — a validator
that fires on the overwhelming majority of correct files gets disabled on first contact and
teaches false confidence (a clean run reads as "the file is right," which is a claim `validate`
never makes). Severity is `error` / `review` / `note`, and findings must never assert intent
the document doesn't carry.

`montaget compare` is a separate, already-accepted tool: it takes **two versions of a project**
(a before and an after — e.g. via git) and reports what changed between them. It is the
project's designated place for anything that requires a *before/after*, i.e. detecting that a
relationship **held and then stopped holding** — as opposed to `validate`, which only ever sees
one document and can report that something *is* true of it, never that something *used to be*
true and no longer is.

## The commissioned check

A separate accepted decision (about the transform/keyframe model) commissioned a `validate`
check for a specific real hazard: **an animated multi-element entrance is expensive to author
correctly.** Canonical example — a "lower third" graphic made of a colored bar element and a
text element, both meant to slide in together as one visual unit. They are two separate
elements (there's no "compound element" concept), conventionally sharing a `group` string for
organization. If an edit changes one element's animation keyframes but not the other's, the
two pieces desync — the bar slides in on one timing, the text on another — and **nothing in the
file says these two were ever supposed to move together**, so there's no way for a reader (or
`validate`) to know a coupling was even intended, let alone broken.

The check as commissioned: **fire when elements sharing a `group` have transform keyframe
`t` values that disagree.** It was explicitly described as "the mitigation, not a cure" for
this hazard — a mitigation, not a full fix.

## Evidence: four agents tried to implement or use this check, and each found a different failure

1. **False-positives on every group in the real fixture.** Measured against a real, correct,
   already-shipped project file: five separate groups — a photo card, three more like it, and
   an eight-element quiz group — **all** show keyframe-time disagreement among their members.
   `group` is a *vocabulary* unit (a photo, a card, a caption, a set of counters) — not a
   *motion* unit. The check fires on the overwhelming majority of correct, intentional
   documents.
2. **Its scoping is per-property and nothing says so.** An agent wrote the literal
   "all keyframe `t`s in a group must match" reading and had to fix it against a group mixing
   a `scale`-animated headshot with an `opacity`-only caption on an unrelated schedule — two
   channels that have no reason to agree and were never coupled.
3. **It punishes deliberate staggers.** A three-element end-card entrance at
   35600 / 36300 / 36700 ms — correct and intentional, a staggered entrance — is
   indistinguishable to the check from the coupled-delta bug it exists to catch.
4. **It's blind to the case closest to its actual target.** A cross-property hand-off
   ("property A's animation starts exactly where property B's animation ends," a real coupling
   pattern) gets no opinion at all from a same-property, same-group check — verified to fire
   identically before and after an edit that destroys such a relationship, discriminating
   nothing about the exact case it should catch.

## What to decide

1. **What unit should the check range over?** (`group` as-is; a narrower opt-in "motion unit";
   something else?)
2. **Should it be scoped per-property, and if so how is that expressed/discovered by an
   author?**
3. **How would a deliberate stagger be expressed so it does not fire as a false positive** —
   without requiring every author who wants a stagger to already know about and opt out of
   the check?
4. **Is the cross-property hand-off case in scope for this check, or does it belong to a
   different mechanism entirely?**

Then give your **overall verdict**: should some version of this check ship in `validate`
(describe its exact final shape, addressing all four evidence points above), or should it be
retracted from `validate` and handled elsewhere (say where, and how), or something else
entirely? You may conclude any of these — argue from the evidence and the project's stated
principles (noise budget, no-intent-assertion, validate's single-document/no-before-after
scope, compare's before/after scope), not from precedent you assume exists elsewhere.

## Output format

Answer each of the four sub-questions in turn, then state your verdict as one bolded sentence,
then your reasoning. Be concrete: if you propose a check, state its exact trigger condition
and what it would report. If you propose retracting it, say precisely what (if anything)
replaces it and where that lives.
