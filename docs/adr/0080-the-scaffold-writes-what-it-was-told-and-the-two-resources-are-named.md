---
status: accepted
amends: 0011 (names the two resource URIs, names and media types it publishes without
  naming; places the format docs inside `montaget-core` rather than under `docs/`; replaces
  `E-PROJECT-EXISTS`'s exit-1 reading with exit 3; fixes the CLI spelling of the one verb
  whose name is not a single word), 0030 (closes the question it leaves open — the scaffold
  writes no defaultable key it was not asked for, and the ticket's five-key enumeration is
  read as the header's shape rather than the scaffold's output), 0073 (settles one of the
  three codes it names as "a separate call for whoever next touches them":
  `E-PROJECT-EXISTS` moves to `NotAboutDocument`; the other two stay open)
---

# The scaffold writes what it was told, and the two resources are named

> **Amended by [ADR-0083](0083-mcp-iserror-tracks-notaboutdocument.md).** Resolves the
> question §2 leaves open: `isError` over MCP tracks `RepairClass::NotAboutDocument`, so
> `create_project`'s MCP handler now sets `isError` for `E-PROJECT-EXISTS`, matching the
> CLI's exit-3 reading this ADR gives it.

**Ticket:** [#246](https://github.com/MBehtemam/Montaget/issues/246), from
[#194](https://github.com/MBehtemam/Montaget/issues/194).

## The gap

[#194](https://github.com/MBehtemam/Montaget/issues/194) built `create_project` and the two
MCP resources. Five things it had to decide had no ADR behind them, and #246 recorded each
at its site — in `crates/montaget-core/src/verbs/create_project.rs`,
`crates/montaget-core/src/resources.rs`, `crates/montaget-core/src/registry.rs` and
`crates/montaget/src/mcp.rs` — rather than leaving them to be discovered from a struct.

ADR-0031 makes the ADR series the specification. A refusal an author can hit, a URI an agent
is told to cite, and an exit code a script branches on are all observable surface, so a
decision about them living only as a source comment is a gap in the spec and not only in the
docs. This ADR closes it.

**Four of the five are ratified as shipped. One is replaced**: `E-PROJECT-EXISTS` exits 3,
not 1.

## Decision

### 1. The scaffold writes `background`, `duration` and `output` only when asked

Amends [ADR-0030](0030-defaultable-field-presence-is-content.md), which is the ADR that
owns the reasoning and which closes by declining to apply it here:

> Whether `create_project`'s scaffold … should recommend one spelling as more idiomatic is
> a style preference, not a schema decision, and **is left open**.

It is now closed, in favour of what shipped: `frame` and `fps` are required arguments;
`background`, `duration` and `output` are optional ones, and each appears in the file
exactly when it was passed.

The argument is ADR-0030's own, carried one step further than ADR-0030 chose to carry it.
Omission and explicit-at-default are **two spellings of different declarations** — omission
says *"give me whatever the default is"*, an explicit value says *"I have pinned this."* A
scaffold that wrote `"background": "#000000"` unasked would therefore author a declaration
nobody made, and under ADR-0030 that declaration is content: it survives every `fmt`
forever, and it diverges from an omission the moment the default is revisited. There is no
later tool that can undo it, because the tool that would — `fmt` — is the one ADR-0030
forbids from touching presence at all.

**This departs from the ticket, and the departure is the reason this section exists.** #194
says `create_project` *"scaffolds a legal project with `frame`, `fps`, `background`,
`duration`, `output` and an empty `tracks` array"*, and story 1 of
[#168](https://github.com/MBehtemam/Montaget/issues/168) says the same; read literally, all
five keys are always present. The implementation's source comment originally claimed
ADR-0031's tiebreak — *the ADR wins where the ticket disagrees* — and #246's own review
correctly withdrew that: an ADR that says in as many words that a question **is left open**
cannot win a disagreement, so #194's sentence was the only instruction actually speaking.

So the ticket is not overridden by an ADR that already existed. It is overruled by this one,
on the merits, and the five-key sentence is re-read as naming **the header's shape** — which
is what story 1 actually asks for, *"so that I never start from a blank file and invent a
shape"* — rather than the scaffold's output. The shape is delivered by the tool schema,
which enumerates all five arguments whether or not they are passed, and by
`montaget://schema.json`, which enumerates every key in the format.

The cost is real and is accepted: an agent that passes `project`, `frame` and `fps` alone
gets a three-key file, and learns about the other three from the tool schema rather than
from the file it is now editing. The alternative — making all five required — buys a
five-key file at the price of forcing the agent to invent a `background` and a `duration`
before it knows either, which is a worse version of the same friction.

`loop` (ADR-0062) is not offered as an argument at all. It is not one of the five #194
enumerates, and the same rule applies to it a fortiori.

### 2. `E-PROJECT-EXISTS` stands, and moves to exit 3

Amends [ADR-0011](0011-tool-surface-reads-checks-renders.md), which gives the write-tool
invariant and the atomic write and settles nothing about clobbering, and
[ADR-0073](0073-process-level-errors-carry-no-repair-field.md), which names this exact code
as unfinished business.

**The refusal is ratified.** `create_project` never overwrites: an existing path is
`E-PROJECT-EXISTS`, nothing is written, and the file on disk is untouched to the byte. A
scaffold that overwrites is a scaffold that deletes a project, and the file it would land on
is the one the agent is mid-edit on and the one git is tracking. Nothing about the call
distinguishes "scaffold here" from "scaffold here, and I know what is already there", so
there is no intent to act on but the safe one.

**The exit code is replaced: 3, not 1.** ADR-0011 distinguishes its exit codes by *what the
caller does next* — exit 1 is *"fix the project"*, exit 3 is *"fix the command"*. Shipping
this at 1 asked the caller to fix a project that is not broken:

- the file that is there is intact, and this call has said nothing about whether it is
  legal;
- the project being scaffolded does not exist, so there is no document to repair;
- both real next moves — scaffold at another path, or stop and edit the file that is already
  there — are repairs to the invocation.

Two independent reviewers of #194 landed on this, and it was kept at 1 only because moving
it meant widening `Report`'s constructors, which #194's closing comment judged *"a larger
change than the ambiguity warrants **from an implementation ticket**."* This is not an
implementation ticket, and #246 asks in as many words for an ADR that *"ratifies or
replaces"* the code. The widening is `Report::refused_invocation`: a small body built on
`Report::new` and `Report::push`, so the finding is still held to ADR-0043's repair
invariant, and asserting for itself that the code it carries is declared
`NotAboutDocument`.

It is now the only non-`E-INVOCATION` inhabitant of exit 3, and that is the shape the
surface should have: `E-INVOCATION` is the general case whose whole answer is the usage
text, and this is the narrow one a verb has its own registered code and template for. The
exit code says what to do; the code says what happened.

**And it becomes `NotAboutDocument`, not `Advise`.** ADR-0073 established that ADR-0043's
refuse/advise question — *is the fix determined by the document?* — does not apply to a
finding whose subject is not a document, and closed by naming three codes that sit on the
same fault line as the four it reclassified:

> `E-NOT-A-PROJECT`, `E-PROJECT-EXISTS`, and `E-SOURCE-MISSING` are advise-class today on
> reasoning identical in shape to `E-READ`'s retired reasoning … Whether the three above
> should also move to `NotAboutDocument` is **a separate call for whoever next touches
> them**.

This ADR touches `E-PROJECT-EXISTS` and makes that call for it. The advise value it carried
(`"scaffold at a path that does not exist yet"`) was a structured repair to nothing — no
document anywhere would be edited to apply it — and the registry template already states the
remedy as the sentence it is: *"Edit it, or scaffold somewhere else."* That is ADR-0073's
prescribed shape exactly, and moving the code to exit 3 makes keeping the old one
incoherent, since a `repair` field beside *"fix the command"* is advice about the wrong
artifact.

**`E-NOT-A-PROJECT` and `E-SOURCE-MISSING` are untouched and stay open.** They are not this
ticket's, they are reached from different verbs, and `E-SOURCE-MISSING` in particular is
about a document — it fires on a `source` the document names. ADR-0073's sentence still
stands over both.

**The check-then-write window is ratified as the narrower risk.** `path.exists()` is asked
before the atomic write, and the write is a `rename`, which replaces unconditionally, so a
file created in between is overwritten. Closing it would mean reserving the destination
first, which trades a race nobody has hit for a failure mode that is strictly worse and
deterministic: an empty file left behind when the write then fails, on a path the agent
believes is free. Recorded so a later reader knows it was weighed rather than missed.

### 3. The two resource URIs, names and media types are named

Amends ADR-0011, which publishes *"the schema"* and *"the format docs"* as resources —
*"discoverability is the schema's job, and a resource costs no tool slot"* — and names
neither. Ratified as shipped, and now fixed:

| URI | Name | Media type |
| --- | --- | --- |
| `montaget://schema.json` | `montaget-schema` | `application/schema+json` |
| `montaget://format.md` | `montaget-format` | `text/markdown` |

Listed in that order: the shape first, then the rules over it.

They look like the files they serve because that is the only thing about them an agent can
guess, and an agent that has read one will cite it to the next — in a commit message, in a
comment, in a prompt. `montaget://` is an invented scheme, which is what a custom URI scheme
always is; MCP resource URIs are opaque identifiers scoped to their server, so the
alternative is not a standard scheme but a less legible invented one.

The point of ratifying these is that they do not move. `the_published_surface_is_the_one_adr_0080_names`
pins all six strings and their order, so a rename is a failing test rather than a silent
break in a published surface — which is the gap that made this worth an ADR at all, since
nothing in the suite asserted any of them before.

### 4. The format docs live inside `montaget-core`

Amends ADR-0011, on the constraint [ADR-0064](0064-packaging-cargo-and-releases-all-six-targets-passive-updates.md)
imposes. `crates/montaget-core/docs/format.md`, `include_str!`d, rather than a path resolved
under the repository root: ADR-0064 ships `cargo install` and six release binaries, and a
resource that resolved to a repo-relative path would be a resource that works only in a
checkout — that is, a resource that fails for every user who did not clone.

The cost is that the repo's prose lives in two places, and this ADR does not pretend
otherwise. What it has instead of a pretence is a guard:
`every_adr_the_format_docs_cite_exists_and_is_still_accepted` re-reads every one of the 27
ADRs the format docs cite inline and fails if one has stopped being `accepted`. It is not a
semantic check and does not claim to be — what it catches is a rule taught there whose
decision has since been retracted, which is the failure that would otherwise be silent
because the prose would go on reading perfectly well. It is `docs/agents/domain.md`'s
re-executable check rather than a screenshot of one, and it caught a real defect on its
first run: two ADR header spellings live in this series, and a check that knew only the YAML
one failed on ADR-0008 for a reason unrelated to its status.

The schema half needs no such guard, because it is generated from
[`crate::model::Project`] rather than written down twice.

### 5. The CLI spells it `create-project`; the MCP tool takes `frame`

Amends ADR-0011, which names the verb `create_project` throughout. Every other subcommand is
a single word, so this is the first and only place the ADR series' snake case and the CLI's
one-word habit meet.

**The CLI subcommand is `create-project`, with `create_project` as a permanent alias.** The
hyphen is what `--help` advertises, because two advertised names for one verb is two things
to parse where ADR-0011 wants one. The alias is not a deprecation path and will not be
removed: an agent that read the verb's name from an ADR or from the MCP surface and typed it
at a shell should not be answered with a usage error over a punctuation mark it had no way
to know about. `cli_create_project_is_spelled_with_a_hyphen_and_answers_to_the_verbs_own_name`
asserts both spellings work and that only one is listed.

**The MCP tool takes `frame` as the nested object, not a flattened `width`/`height` pair.**
This half was decided during #194's review rather than recorded, and it is the one place
that review *changed* the implementation instead of documenting it. The original flattened
`frame` on both adapters on agent-legibility grounds, which is a convenience argument against
the one constraint #194 names as its own point:

> A tool that writes may only take a complete element, as a schema-shaped object. No tool
> takes a field name or an element id.

The invariant is argued for elements and this verb writes a header, but the rule it protects
— that the argument shape and the file shape are one thing an agent learns, not two —
applies unchanged. So `create_project`'s arguments **are** the project header in the shape
the agent has already read at `montaget://schema.json`, and the adapter invents no shape of
its own. `mcp_create_project_advertises_the_schema_it_enforces_and_returns_the_new_states_findings`
checks every advertised header property against `schema::generate()`, so a future flattening
fails a test rather than passing review.

**The CLI takes `--width` and `--height`**, and that is not a contradiction. argv has no
nesting, and the alternative is inventing a `1080x1920` mini-syntax — a third spelling of
the frame, parsed nowhere else, which is a worse answer to the same rule. ADR-0011's
invariant is hereby read as binding **the MCP surface**, where a schema-shaped object is
expressible, and not argv, where it is not.

## Consequences

- **ADR-0011, ADR-0030 and ADR-0073** each gain an "Amended by" banner pointing here.
  ADR-0030's is its first.
- **`E-PROJECT-EXISTS` changes behaviour.** Its registry entry becomes
  `repair: Some(NotAboutDocument)` and its `adr` field points here; the call site drops its
  `repair_value` and returns through the new `Report::refused_invocation`. A caller that branched on
  exit 1 for this condition now sees exit 3 — the one breaking change in this ADR, and the
  reason it is a replacement rather than a ratification.
- **`Report::refused_invocation` is added** as the third exit-3 constructor, beside `bad_invocation`
  (argv's shape) and `rejected` (a verb's own arguments). It asserts the registry
  declaration itself: a code reaching exit 3 this way must be declared `NotAboutDocument`
  or it panics — a second inhabitant cannot be added without making the same decision.
  `Report::push` alone does not carry that weight, which is why the check is stated here
  rather than inherited: `push`'s assert is satisfied by *either* an exempt declaration or
  a repair value, so an advise-class code that states its repair would pass it and still
  arrive at exit 3 carrying the `repair` field ADR-0073 forbids there. A `should_panic`
  test in `tests/registry.rs` holds the guard.
- **The `#246` passages** in `create_project.rs`, `resources.rs`, `registry.rs` and
  `mcp.rs` are replaced with citations to this ADR.
- **Two readings that shipped with no test now have one**, so a later change is a failing
  suite rather than a silent contradiction of this ADR:
  - reading 3, by `the_published_surface_is_the_one_adr_0080_names`, which pins both URIs,
    both names, both media types and their order;
  - reading 5's CLI half, by
    `cli_create_project_is_spelled_with_a_hyphen_and_answers_to_the_verbs_own_name`, which
    exercises both spellings end to end and asserts `--help` lists only the hyphen.
- The rest were already asserted: reading 1 by
  `a_field_the_agent_did_not_state_is_not_written_for_it`, reading 2's refusal by
  `an_existing_file_is_never_overwritten` (now also asserting exit 3 and the absent `repair`
  key) and `cli_create_project_onto_an_existing_file_is_exit_3_and_changes_nothing`,
  reading 4's guard by `every_adr_the_format_docs_cite_exists_and_is_still_accepted`, and
  reading 5's MCP half by
  `mcp_create_project_advertises_the_schema_it_enforces_and_returns_the_new_states_findings`.

## Not settled here

- **The check-then-write window is ratified, not closed.** A ticket that wants it closed
  needs an atomic create — `O_EXCL` on the destination — which is a different write path
  from `crate::write::atomically` and would have to answer what happens to the temp file
  when the exclusive create succeeds and the write then fails.
- **`E-NOT-A-PROJECT` and `E-SOURCE-MISSING`** keep ADR-0073's open question. Nothing here
  touches them, and `E-SOURCE-MISSING` is about a document in a way this code is not.
- **Whether authoring documentation should recommend omission or explicit-at-default** as
  more idiomatic. ADR-0030 left this open alongside the scaffold's behaviour; only the
  scaffold's behaviour is closed above. `format.md` teaches that presence is content and
  recommends neither, which is the right place for the question to stay open.
