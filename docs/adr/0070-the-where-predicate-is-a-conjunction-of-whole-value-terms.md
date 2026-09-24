---
status: accepted
amends: 0011 (specifies the `--where` predicate grammar it named and left undefined, and bounds its "resolved values, never echoed fields" rule to `--at`, which is the mode that rule was written about)
---

# `--where`'s predicate is a conjunction of whole-value terms over what the document writes

[ADR-0011](./0011-tool-surface-reads-checks-renders.md) specifies
`query --where <predicate> [--census <field>]` and stops there. It says what the mode is
for — *"naming will go wrong, and the surface's job is to make it go wrong before the
write"* — and nothing about what a predicate looks like. The question the mode answers is
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md)'s sibling census, one
call rather than a script: *"Four of five are 1597, one is 1537 is inert data about the
file: it carries the fix without proposing it."*
[#196](https://github.com/MBehtemam/Montagent/issues/196) built the mode, so a grammar had
to exist, and the one it shipped was argued in
`crates/montagent-core/src/verbs/query/predicate.rs`.
[#249](https://github.com/MBehtemam/Montagent/issues/249) raised that as the wrong place
for it, in the same spirit as [#241](https://github.com/MBehtemam/Montagent/issues/241):
every decision in it is what an agent has to *learn*, and it is as good as permanent once
an agent's prompts contain it. This ADR is where the argument is had.

**The grammar is ratified as shipped.** Nothing below overturns #196; what changes is that
these are now decisions of the series rather than of a source file, and that the reasons
are written where a reader who disagrees can find them.

```text
predicate := term (`and` term)*
term      := path op literal | path `exists` | path `missing`
op        := `=` | `!=` | `<` | `<=` | `>` | `>=`
path      := segment (`.` segment)*
segment   := a key, an array index, or `*` for every member of an array
```

plus one reserved first segment, `track`, meaning the containing track's `name`.

## `and`, and no `or`

`and` is kept because the sibling census needs it. ADR-0006's *"the other four elements of
this track are at y = 1597"* is a census over a set narrowed twice — this track, and this
kind of element — and a census is only readable over a set that was narrowed to the
siblings first. ADR-0011's own recorded naming failure is the same shape from the other
side: the agent that resolved it did so by *"finding three independent selectors that
agreed on the same five elements"*, and a selector with one term is not three.

`or` is refused. Two calls answer a disjunction and the union of their matched sets is the
answer, so what a disjunction buys is one turn — against which a disjunction is the first
step towards an expression language. The step after `or` is precedence, and after
precedence parentheses, and at that point the grammar has to be published as an MCP
resource and versioned. ADR-0011's `jq` section is what makes that cost load-bearing
rather than aesthetic: it concedes that *"`jq` does not replace `query`, but it gets
further than the rebuttal claimed"* — five of its eight components are a one-liner — so
this mode competes with a tool the agent already knows. A predicate an agent has to look
up is a predicate it will write a `jq` filter instead of. A language a reader can hold in
their head after one example is the asset here, and `and`-only is the largest such
language.

The cost is stated: a question of the form *"every element that is text **or** audio"* is
two calls and a merge the caller performs. **The refusal names the rule**, alongside the
text it could not read, because an agent told only `` `or type = image` is left over ``
will next guess `||`, then `OR`, and spend the turn on spelling.

## Whole values only: no substring matching and no regular expressions

`=` compares whole values, and there is no `~`, no `contains`, and no pattern of any kind.

The reason is `--census`, which is the argument for the whole mode. A census groups the
matched set by the values at one path, and every group it names is a value some element
actually states. Under a half-matching predicate the set beneath it is a set of elements
that agreed *partly*, and the distribution stops being a distribution over anything —
*"four of five are 1597, one is 1537"* becomes four of five whose fonts share a prefix,
which is not a fact about the document. Whole-value matching is also what makes the two
halves of the answer commute: the matched set is a set the census could have been taken
over directly.

The cost is stated: *"every element whose text mentions Halloween"* is not askable, and
the mode does not pretend otherwise — there is no operator that half-answers it.

## `exists` and `missing` are required, not extra

[ADR-0030](./0030-defaultable-field-presence-is-content.md) makes a defaultable field's
*presence* content: writing `"fit": "cover"` and omitting `fit` are different documents,
and `fmt` may not add the key. A format in which presence is content and a query surface
in which presence is unaskable would be inconsistent on its face.

#249 notes that ADR-0011's worked example does not need them. That is true and is not the
test: the test is whether absence is reachable at all, and it is not otherwise. A path
that reaches nothing satisfies neither `=` nor `!=` — `!=` is *"has a value that is not"*,
never *"has no value that is"* — so without `missing` the only way to ask *"which elements
declare no `fit`"* is to fetch every element and subtract, which is the `jq` script this
verb exists to replace.

Keeping `!=` as *"has a value that is not"* is the same decision seen from the other side:
an operator that silently also meant *absent* would make the two askable only together,
and ADR-0030 is precisely the ADR that says they are different.

## `*`, and array indices

`*` matches every member of an array. It is ratified because
[ADR-0007](./0007-text-runs-literal-size-declared-fonts.md)'s font question has exactly
that shape: a run may override its element's font, so *"which fonts does this element
use"* is a question about `runs.*.font` and not about `runs.0.font`. Without it the
font census ADR-0007 motivates cannot be asked.

An all-digit segment reads as an **array index**. Nothing asks for `runs.1.font`; the
segment exists because a digit segment has to mean *something*, and the alternative —
reading it as an object key — makes `runs.1.font` reach nothing, silently, on every array
the format has. A path that silently matches nothing is the worse of the two answers,
since an empty matched set is also what a *correct* predicate returns when the document
disagrees with the caller.

This is only safe while no property the schema *declares* is spelled in digits alone,
which `predicate_reserved_names_scan.py` re-derives and fails on. Author-chosen keys are
outside the question: the `fonts` table is the format's one open map, and a predicate path
starts at an element, which never reaches it.

## A comparison holds if **any** value at the path satisfies it

For every path without a `*` this is ordinary comparison, since such a path reaches at
most one value. For `runs.*.font` it reads as *"has a run whose font is …"*, which is the
question an element-set query is asking: the answer is a set of elements, so every term
has to be a predicate about an element.

Two consequences follow and are ratified with it. An element reaching one value twice is
one member of its census group, not two — the answer is a set. And an ordering comparison
between two values of different kinds (`<` against a string on one element and a number on
another) is **no match** rather than an error. The reason is that a query is not read
through the strict model: it reads the permissive tree, which exists because
[ADR-0042](./0042-montagent-json-is-a-convention-fmt-gets-a-shape-check.md) refuses a
document only when it is *"missing the required top-level keys that make it recognizable as
a project at all"*, since a `note`- or `review`-level finding *"does not make a file any
less a legitimate, safely-formattable Montagent project"* — while
[ADR-0017](./0017-closed-schema-no-escape-hatch.md) makes an unknown key an error. So a
query routinely runs over a document holding an `fps` of `"25"`, and a verb that aborted on
the first such element would answer nothing about the other fifty-nine.

## Literals: a bare word is the JSON scalar it spells, and quoting forces a string

`loop = true` reaches the boolean; `id = "true"` reaches the element whose id is the word.
Numbers compare **numerically**, so `opacity = 1` reaches a document writing `1.0` —
`serde_json` holds those as different numbers and the format's own arithmetic does not.

Without the quoted spelling one of those two questions is unaskable, and the one that
would be lost is the one about a *document the agent did not write*, which is the whole
population this verb serves.

## `track` is reserved, unconditionally

`track` as a path's only segment is the containing track's `name`. An element does not
carry the name of the track it sits in — the nesting is the statement — and filtering by
track is the most ordinary question there is; ADR-0011's own recorded naming failure (*the
track named `caption` does not hold the captions*) is a question about tracks.

The reservation is unconditional rather than a fallback for elements writing no `track`
key, because a rule with an exception is a rule an agent has to test before trusting.
[ADR-0017](./0017-closed-schema-no-escape-hatch.md)'s closed schema gives no element type
a `track` field, so the reservation shadows nothing — checked, not assumed, by
`predicate_reserved_names_scan.py`.

**The cost, stated plainly: the format may never give an element a `track` field.** It is
a name spent. That is acceptable because the schema is closed and adding one would be a
format change this series would have to argue anyway — at which point this ADR is amended
rather than silently violated. The scan is what turns "silently" into a failing check.

## Document-literal matching — and what ADR-0011's organising rule actually governs

`--where 'y = 1597'` does **not** match an element whose `y` is a keyframe list passing
through 1597. A path names what the document *writes*: nothing is interpolated, no anchor
is resolved, and `layer` compares against `{"below": "card"}` rather than against the
integer [ADR-0019](./0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md)'s one
hop would produce.

#249 is right that this reads oddly beside ADR-0011's organising rule that *"`query`
returns resolved values, never echoed fields"*. The resolution is that the rule is about
the **output of `--at`**, and its argument says so: *"echoing
`"scale":[[3018,1.0],[18018,1.08]]` back at the agent tells it nothing it did not have;
`scale 1.0170` is the entire point"* — an explanation of one moment, which is what `--at`
produces. `--where` has no instant. There is no *t* at which `y = 1597` could be asked,
so a resolving `--where` would have to mean *"passes through 1597 at some instant"*, which
is a different question wearing the same spelling, and one whose answer changes with
easing.

Read together, the two modes divide cleanly and the division is the decision:

| | question | over |
| --- | --- | --- |
| `--at <t>` | what is true **at this instant** | resolved values |
| `--where` | which elements the **document says this about** | written values |

This also keeps #196's acceptance criterion true by construction — the mode works with no
resolver present — and keeps `--where`'s answer stable under a change to the easing
implementation, which matters for a predicate an agent writes into a prompt.

The gap this leaves is real and is left open deliberately: *"which elements are at y=1597
at 4 s"* is `--at`'s question, and `--at` is the third mode, still blocked by ADR-0011 on
the crop rectangle and the ink box. Until it ships, that question is asked as
`--where 'y exists'` plus a look at the element.

## Not settled here

- **`--at`'s own predicate.** Whether the third mode ever accepts a `--where` — a
  predicate over *resolved* values at an instant — is a question for whoever ships it.
  Nothing here forecloses it; if it arrives, the two readings are two modes and not one
  operator that changes meaning.
- **Publishing the grammar as an MCP resource.** It is in the CLI help and in the `query`
  tool's input schema today, which is where an agent reads it. A separate resource
  (ADR-0011 makes the schema and the format docs resources) buys a canonical URL and costs
  a second place to keep in agreement; nobody has asked yet.
- **Negation of a whole term** (`not <term>`), and grouping. Both are the `or` argument
  again and both are refused for now by the same reasoning rather than decided against
  forever.

## Consequences

- **The grammar is spec.** A change to it amends this ADR, the way a change to the format
  amends the ADR that owns the field.
- **`or` is refused with a sentence that names the rule** and points at the two calls that
  answer it, in place of the generic leftover-text error.
- **The published help states the whole language**, including the quoting rule and the
  any-value reading — an agent that has to read this ADR to write a predicate has already
  paid the cost ADR-0011's `jq` argument was avoiding.
- **`CONTEXT.md`'s *Predicate* entry loses its "not ratified" note** and cites this ADR.
- **`schema/montagent.schema.json` acquires two standing constraints**: no element property
  named `track`, and no declared property spelled in digits alone. Both are re-derivable by
  one command, which is what makes them constraints rather than a note — but **nothing runs
  it for you today**: GitHub Actions is disabled for this repository, so the step registered
  in `ci.yml` beside the amendment-banner check is a standing arrangement rather than a
  gate. Whoever adds a property to the schema runs
  `python3 docs/adr/predicate_reserved_names_scan.py`, the way
  `docs/agents/domain.md`'s evidence rule is a checklist step and not a mechanical one.

## Evidence

The decisions are argued from committed spec text — ADR-0011's `--at` argument, its `jq`
section and its recorded naming failure, ADR-0006's sibling census, ADR-0007's font
question, ADR-0017's closed schema, ADR-0030's presence rule, ADR-0042's refusal
precondition — and the two claims about the *schema* rather than about the argument are
re-executable:

- `docs/adr/predicate_reserved_names_scan.py` — a check a reader runs, not one a robot
  runs for them (see the Consequences). It re-derives both against the committed
  `schema/montagent.schema.json`, and exits non-zero the moment either stops holding: no
  element variant declares `track` (across all 7 variants), and no object property
  anywhere in the schema is spelled in digits alone. Each claim is checked and reported
  independently, so a schema that grows one of them is told which.
- `crates/montagent-core/tests/query.rs` — the behaviour, at the verb's seam. #196 already
  pinned most of what is ratified here: the reserved `track` path, the wildcard and its
  census, `exists`/`missing` with the `!=`-does-not-mean-absent pin, numbers compared
  numerically and quoting forcing a string, and — the document-literal decision, at the
  seam it is decided at — an `x` written as a keyframe list passing through 100, which
  `x = 100` does not match and `x exists` does. Ratifying added the three that were
  decided but unpinned: an index segment reaching one member, whole-value matching that
  does not match a prefix, and the refusal of `or` (in each spelling a caller would try)
  beside the pin that a value merely beginning with those letters is a value.

No jury was convened. Every question #249 raised is settled by a requirement already
written down in this series, and the one that was genuinely in tension — document-literal
matching against *"resolved values, never echoed fields"* — is resolved by reading what
that rule's own argument is about, not by weighing a preference.
