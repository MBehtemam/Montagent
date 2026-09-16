---
status: accepted
amends: 0011 (`fmt` gains an explicit exception for defaultable-field presence), 0012
  (states that omission and explicit-at-default both remain legal for the six flat transform
  fields), 0007 (states the same for `line_height`)
---

# A defaultable field's presence is content: `fmt` leaves it alone, both spellings stand

[ADR-0012](./0012-flat-transform-keyframes-carried-by-their-element.md) defaults six things
when omitted — `x`/`y` to the frame centre, `origin` to `center`, `scale` to `[1,1]`,
`rotation` to `0`, `opacity` to `1` — and [ADR-0007](./0007-text-runs-literal-size-declared-fonts.md)
defaults `line_height` to `1.2`. For each, an element can express the same effective value
two ways: by omitting the field, or by writing it explicitly at its default. [ADR-0011](./0011-tool-surface-reads-checks-renders.md)
defines `fmt` as rewriting the file "in the canonical convention" but never says whether it
inserts or strips a defaulted field. [#61](https://github.com/MBehtemam/Montaget/issues/61)
asked whether `fmt` should canonicalize toward one spelling, and if not, whether the schema
should ban the other the way `center-center` is banned.

## Decision

**`fmt` never touches a defaultable field's presence or absence. Both omission and
explicit-at-default remain permanently legal; `validate` and the schema never flag either.**

Omission and explicit-at-default are not the `center-center` case. `center-center` banned
two spellings of *the same declaration* — nothing distinguishes them, so one had to die.
Omission and explicit-at-default are two spellings of *different declarations* that happen
to coincide in effective value today: omission says "I have no opinion, give me whatever the
default is"; writing `opacity: 1` says "I have pinned this to 1." They diverge the moment a
future edit changes the field on a sibling element, or the default itself is revisited — the
choice to pin a value against drift, or to leave it floating, is exactly the kind of
authorial intent ADR-0013 and ADR-0014 already protect as content, never a spelling `fmt`
is free to normalize away.

Both alternatives fail on the mechanism [#8](https://github.com/MBehtemam/Montaget/issues/8)
found is how every agent edits: stripping a written default silently deletes a line the
agent just added; materializing an omitted field silently inserts up to seven lines into
every element and invalidates any pending exact-string replace whose context window touched
that block. Either is `fmt` — a tool whose charter is to be semantically inert — deciding
content on the author's behalf, the same failure `center-center` exists to prevent, just
triggered by presence instead of spelling.

Banning explicit-at-default outright (the reading that stays closest to `center-center`'s
form) was considered and rejected: it is a more invasive move than `center-center`, which
cost the author nothing since `center` says everything `center-center` said. Banning
`opacity: 1` forbids an agent from being explicit about a value it wants guaranteed against
a future default change, and it converts the safest possible edit — replace a scalar in
place — into the most fragile one — delete a key and repair the neighbouring line's trailing
comma — on all six fields, forever.

## Evidence

Three-model court (Claude Opus 5, Claude Sonnet 5, Claude Haiku 4.5), each blind to the
others' ballots, put the same two-part question. **Unanimous 3/3 that `fmt` must never touch
presence.** Split 2–1 on whether explicit-at-default should then stay legal or be banned:
Opus and Sonnet held both spellings legal, for the reason above — omission and
explicit-at-default are different declarations, not a redundant duplicate. Haiku held that
consistency with `center-center` demanded picking one canonical spelling via schema,
independent of `fmt`, with omission as canonical. The majority's reading engages directly
with the premise the dissent doesn't: `center-center` had zero semantic daylight between its
two spellings, and this case does. The dissent's ban also reintroduces the exact-string
fragility cost the majority names, which the dissent's own ballot does not address.

## Consequences

- `fmt` (ADR-0011) gains a documented exception, alongside declared extents (ADR-0013) and
  colour spellings (ADR-0014): it must not insert a default for an omitted field, and must
  not strip a field explicitly written at its default value.
- `validate` and the schema treat both spellings as legal for all six fields
  (`x`, `y`, `origin`, `scale`, `rotation`, `opacity`, `line_height`) — no new check is
  added, and none should flag explicit-at-default as redundant.
- Two project files can be textually different (one element omits `opacity`, an otherwise
  identical element writes `opacity: 1`) and render byte-identically. As with ADR-0026,
  diff-based and byte-equality tooling must treat this as meaningful divergence in authored
  intent, not noise.
- `center-center` (ADR-0013's ban) is unaffected and remains the narrower case: it bans a
  redundant compound spelling of one already-explicit value, not the presence/absence
  distinction this ADR settles.

## Not settled here

- Whether `create_project`'s scaffold, or authoring documentation, should recommend one
  spelling as more idiomatic is a style preference, not a schema decision, and is left open.
