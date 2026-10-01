---
status: accepted
amends: 0011 (the format is published as more than two resources: an index and pieces beside the whole schema), 0080 (adds a third named, frozen URI, `montagent://schema/index.json`; the schema's `$def` keys are renamed where schemars numbered them, which §3 never froze; the server instructions start an agent at the index)
---

# The schema is also served in pieces, through one index

[#520](https://github.com/MBehtemam/Montagent/issues/520), raised from
[#441](https://github.com/MBehtemam/Montagent/issues/441) on evidence from
[#447](https://github.com/MBehtemam/Montagent/issues/447).

## The gap

`montagent://schema.json` is 73 KB. That is 54 KB minified, so whitespace isn't the problem.
Claude Code's default MCP output limit is 25,000 tokens, checked first as about 50,000
characters, and it warns at 10,000 tokens. In the no-skills baseline
(`docs/research/skills-eval/BASELINE.md`) the schema **overflowed in 12 of 12 runs**. The
client saved it to a file. Each run spent 1–3 calls pulling out the part it needed, and
carried 51–117 K characters of discovery output in context for the rest of the session.

The skills' answer is a **Workaround** (`reading-the-schema.md`, marked `#520`): save the
schema, then use Python to list the element types and print one `$def`. That is an index
plus pieces, built by hand on the agent's side, and only agents with the skill get it.

## Decision

### 1. The fix is in Montagent, and `schema.json` stays whole

`montagent://schema.json` is not shrunk and not turned into an index. It stays the full
generated schema that editors and validators consume, and its `description`s stay beside
their keys. About 36 K of the 73 KB is description text, and none of it is cut or moved:
it is the rule written where the key is, and moving it into `format.md` would create the
hand-written second copy this project has paid for repeatedly. The size problem is solved
by serving **pieces** beside the schema, not by deleting from it.

### 2. The count of resources is not a rule; generation is

ADR-0011 and ADR-0080 published two resources, and the glossary said "exactly two". That
described what shipped. The reason for resources, *"a resource costs no tool slot"*, argues
for more of them, not against. The rule that does bind every resource, new and old, is this:
**every resource is generated from the same types that parse the project, or embedded
verbatim. No fact about the format is written down twice.**

### 3. Shape: one index, then pieces

- **The schema index**, `montagent://schema/index.json`, is generated. It lists:
  - each element `type`, with its required keys and its piece URI;
  - each effect `name`, with its required parameters and its piece URI;
  - every other piece by name, with its URI.
  
  It carries no descriptions. The rules stay in the pieces.
- **A schema piece** is one part of the schema, served as JSON:
  - **`project`**: the schema root's top-level `properties` and `required`. The root isn't
    a `$def`, so without this piece nothing would serve the project-level keys.
  - **One per element `type`**: the matching branch of `Element`'s `oneOf`.
  - **One per effect `name`**: the matching branch of `Effect`'s `oneOf`.
  - **One per other `$def`**, whole. The small unions inside keyframes are not split.

`Element` (14.6 KB) and `Effect` (6.9 KB) would each fit the budget whole. They are split
because an agent writes one element type or one effect at a time, and a per-variant piece
is the unit it actually asks for.

The index writes out every piece's URI. Nothing depends on MCP resource templates, whose
visibility in Claude Code is undocumented. A server may also declare a template, but no
route relies on it. Pieces don't need to be in `resources/list`: the index is how they are
found.

### 4. A piece's `$ref`s are rewritten to piece URIs

A piece cut from the schema still holds `#/$defs/X` refs, and those don't resolve inside the
piece alone. The generator rewrites each one to the URI of the piece for `X`, so the agent
can follow it. `Element` and `Effect` have no piece of their own (they are split by §3), so
refs to them point at their section of the index.

A piece is therefore a derived view, not a byte-identical slice of `schema.json`. That is not
drift: it is regenerated on every read from the same source. Leaving refs verbatim would
break for exactly the two most-read definitions. Inlining referenced definitions would repeat
`Ease` and the keyframe chains across dozens of pieces.

### 5. Only the index URI is frozen

ADR-0080 §3's promise, that a published URI does not move, extends to
`montagent://schema/index.json` (name `montagent-schema-index`, media type
`application/json`). **Piece URIs are not frozen.** They are reached through the index and
may change when a type is renamed. A skill that wants to point at a piece points at the
index, which is also what "a skill never restates a format fact" already asks of it.

### 6. Numbered `$def` names get stable ones

schemars names a generic type's instantiations in the order it meets them: `Keyframe`,
`Keyframe2`, `Keyframe3`, `Keyframe4`, and the same for `Animatable` and `FirstKeyframe`.
Reordering the Rust types renumbers them, and `Keyframe3` says nothing about which value
type it covers. Since agents read these names in the index and in rewritten refs, the
generic types take a stable name built from their type parameter
(`#[schemars(rename = "Keyframe{T}")]`, supported by schemars 1.2).

This changes `$def` keys inside `schema.json`. It doesn't break §1 or ADR-0080: §1 is about
not shrinking the schema, and ADR-0080 §3 froze the resource's URI, name and media type, not
its internal keys. The schema's content and structure are unchanged. The rename may land as
its own change ahead of the pieces.

### 7. Routing: every entry point names the index

Agents read the URIs they are told about: 14 of 15 baseline runs went straight to the named
resources, and one listed first. So the pointer decides whether pieces are used. It goes in
three places, each saying one thing and none stating a format fact:

- **Server instructions:** start at `montagent://schema/index.json`. They stop naming
  `schema.json` as the place to start.
- **`schema.json`'s description:** it is the whole schema, too large to read in one go in most
  clients. Its pieces are listed in the index.
- **The index's description:** what it lists.

The three URIs come from shared constants, and a test checks that every URI named in routing
text is served.

### 8. A 24 KB budget on every resource meant to be read whole

A test fails when the index, any piece, or `format.md` is over **24,576 bytes**.
`schema.json` is exempt. The budget is in bytes, so the test needs no tokenizer. 24 KB keeps a
read under the 10 K-token warning even for dense JSON, which runs about 3 characters per
token, and leaves margin for clients with unknown, lower limits. The failure names the
resource, the overage and the fix: **split it, do not raise the number**. Raising the number
takes an ADR.

**`format.md` is not split here.** It is 21,386 bytes and fits. It also grew about 4 KB in
the week before this ADR, so the budget will catch it soon. The seams for that split are its
existing `##` sections. Splitting it gives it a table of contents of the same kind as the
schema index, and it is its own ticket.

### 9. The workaround retires with the change

The change that ships the index and pieces also deletes the skill's
`reading-the-schema.md`, and points `SKILL.md` at the index. Keeping the note would leave
the skill teaching the opposite of the server's own routing. `schema.json` stays whole and
readable, so it is the fallback, and the skill needs no copy of one. An evaluation run
checks the shipped route afterwards; it does not gate the change.

## Considered and rejected

- **Index only.** Required keys don't say a key's type or rule, so the agent goes back to the
  73 KB schema the moment it writes an element.
- **Pieces only.** Agents rarely list, and template visibility is undocumented.
- **A 48 KB or token-measured budget.** 48 KB treats the hard limit as the target and draws
  the warning. A token budget ties the suite to one tokenizer.
- **Freezing every piece URI.** That is dozens of promises, and every type rename would become
  a breaking change.

## Consequences

- **`the_published_surface_is_the_one_adr_0080_names`** is amended to pin three listed
  resources in this order: the schema, the format docs, the index.
- **The glossary** changes. **Resource** drops "exactly two" and states the generation rule,
  and adds **Schema index** and **Schema piece**.
- **`schema.json`'s `$def` keys** for the generic instantiations change (§6). The committed
  `schema/montagent.schema.json` is regenerated, and `tests/schema.rs` keeps the two in step.
- **[#520](https://github.com/MBehtemam/Montagent/issues/520)'s workaround** retires with
  the change (§9), so the workaround lint has nothing left to flag when the issue closes.
