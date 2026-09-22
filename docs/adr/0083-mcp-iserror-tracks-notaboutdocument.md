---
status: accepted
amends: 0011 (the write-tool invariant — "the return value is the findings" — is
  restated as scoped to findings about the document; MCP's `isError` now carries the
  distinction ADR-0011's exit-code table already draws), 0073 (extends the
  `NotAboutDocument` classification's reach from the wire's `repair` field to the MCP
  transport's `isError` flag), 0080 (resolves the tension its own text left open: whether
  `isError` tracks the exit-code class or the write-tool invariant for
  `E-PROJECT-EXISTS`)
---

# MCP's `isError` tracks `RepairClass::NotAboutDocument`, not "did the args deserialise"

**Ticket:** [#313](https://github.com/MBehtemam/Montaget/issues/313), grilled and then
put to three three-juror courts (Opus, Sonnet, Haiku — reproduce with `/court`):
unanimous on the rule below and on this ADR's placement, 2–1 on scope, judge-broken
toward the wider reading recorded here.

## The gap

ADR-0080 moved `E-PROJECT-EXISTS` from exit 1 to exit 3 and from `Advise` to
`NotAboutDocument`: a `create_project` call onto an existing path is not a document that
needs fixing, it is a command that needs fixing. The CLI carries that. The MCP arm did
not — `create_project`'s handler returned `CallToolResult::success` unconditionally,
under the write-tool invariant's own words: *"the return value is the findings, which is
what converts an opt-in check into a structural one."*

Both readings are correct as far as they go, and #313 is that they disagree.
`E-PROJECT-EXISTS` is simultaneously "a verb that ran and found something" (the
write-tool invariant's trigger) and "the invocation was wrong" (`rejected`'s own stated
trigger for `isError`, quoted from `crates/montaget/src/mcp.rs`: *"`isError` is what
tells its client the call did not run at all"*). ADR-0080's own text supplied a possible
scope argument — *"a caller that branched on exit 1"*, read as an argv caller — but that
line is ADR-0080 §5, about the `frame`-nesting argument shape, not §2, about the exit
code; it settles nothing here.

Investigating #313 surfaced that the disagreement is not unique to `E-PROJECT-EXISTS`.
Every MCP write-tool handler returns `success` unconditionally regardless of what the
core `Report` contains — `E-PARSE`, `E-READ` and `E-INTERNAL` (all `NotAboutDocument`,
ADR-0073) are indistinguishable from a clean run over MCP today, and so is a verb-level
`Report::rejected` (`E-INVOCATION`) reached *after* successful argument deserialisation,
such as `measure` called with neither `element` nor `at`. `isError` is set to `true` in
exactly one place: a local `rejected()` helper that fires only when the incoming JSON
fails to deserialise into the tool's parameter struct — a narrower condition than
anything the core computes.

## Decision

**`isError` is true exactly when the report is `RepairClass::NotAboutDocument`-classed
— equivalently, when its exit code is 2, 3 or 70 — and `success` otherwise (exit 0 or
1).** This is not a new partition: it is the one ADR-0073 already drew for the wire's
`repair` field, and ADR-0011's exit-code table already draws it for the CLI. The
write-tool invariant's *"return value is the findings"* was reasoned about findings
*about a document* — `Advise`/`Refuse`, exit 0/1 — and stays exactly that scoped:
findings about the document keep answering `success`, findings whose subject is the
invocation, the raw bytes, or Montaget's own process now set `isError`.

Applies uniformly to every code the registry declares `NotAboutDocument` today:
`E-PARSE`, `E-READ`, `E-INVOCATION`, `E-INTERNAL`, `E-PROJECT-EXISTS` — regardless of
which layer produced the finding. A schema-deserialisation failure, a verb-level
`Report::rejected`, and `Report::refused_invocation` all reach exit 3 by the same
`E-INVOCATION`-or-declared-`NotAboutDocument` route and now answer `isError` alike; the
distinction between "the args never parsed" and "the verb ran and refused" was never
load-bearing and this ADR retires it.

Implemented as one predicate, `Report::is_not_about_document`, and one call site per MCP
tool routed through a shared `respond(report, content)` helper in `crates/montaget/src/mcp.rs`
— so a future `NotAboutDocument` code inherits the rule rather than requiring a ninth
call site to remember it.

## Why

**The rule is not invented — it is the one the codebase already has.** `RepairClass` was
carved into three arms by ADR-0073 for exactly this reason: `Advise`/`Refuse` findings
are verdicts on a document, `NotAboutDocument` findings say the run never got far enough
to reach one. `isError` is a transport-level "did the call succeed" signal, which is a
claim about the second thing, not the first. Mapping `isError ⟺ NotAboutDocument`
costs no new field, no new table, and no per-tool judgment call — correctness follows
from a declaration the registry already requires ADR-0073-wide.

**It composes with the write-tool invariant instead of contradicting it.** The
invariant's *"then I don't run `validate`; `validate` runs me"* is about an agent not
being able to skip past an error finding by treating the call as bare success — a claim
about document-shaped answers. A `NotAboutDocument` finding was never that kind of
answer: nothing was produced to skip past. Setting `isError` there does not erode the
invariant; it completes it, by making the one case the invariant never covered
distinguishable from the case it does.

**The partition and the exit-code ladder are the same partition, not two.** "Mirror the
whole exit-code ladder" was considered and rejected: that would flip exit-1 `error`
findings to `isError` too, which is the erosion the invariant exists to prevent. The
actual candidate rule is narrower and turns out to be identical to `NotAboutDocument` —
codes at exit 2, 3 and 70 are precisely the ones the registry already marks
`NotAboutDocument`; codes at exit 0 and 1 are precisely the ones it does not. Two
vocabularies, one boundary.

**Scope: all five codes, not `E-PROJECT-EXISTS` alone.** The panel split on this
(2–1): the narrow reading argued for a smaller, more reviewable diff, landing #313 as
filed and opening a follow-up for the rest. The majority argued that a partial fix of a
uniform bug makes `isError` *inconsistently* meaningful — worse for a client than the
current uniformly-absent signal, since "isError is false" can no longer be trusted to
mean "this is a document-shaped answer" while three of five `NotAboutDocument` codes
still lie about it. Judged in favour of the majority: this ADR states the general rule
regardless of scope, so implementing only a fifth of it would immediately raise "why was
`E-PROJECT-EXISTS` fixed and `E-PARSE` was not," and a deferred follow-up would inflict
the same breaking change on the same clients a second time.

## Consequences

- **Breaking change for MCP clients.** Any client that branched on `isError` and
  assumed it was always `false` for `E-PARSE`, `E-READ`, `E-INVOCATION`, `E-INTERNAL`,
  or `E-PROJECT-EXISTS` sees a behaviour change. The change is additive at the wire
  level — the same content, the same codes, the same JSON shape — only the protocol
  flag moves.
- **`Report::is_not_about_document`** is added to `montaget-core`'s `Report`, backed by
  the existing private `terminal` field — no new state, a name for a distinction the
  type already drew for `exit_code()`.
- **`crates/montaget/src/mcp.rs` gains a `respond` helper** that every write-tool
  handler (`validate`, `create_project`, `query`, `frame`, `render`, `preview`,
  `measure`, `shift`, `compare`) now routes its result through, replacing nine
  independent `CallToolResult::success(...)` call sites.
- **`rejected()`'s own doc comment is corrected**: it is now one instance of this ADR's
  general rule, not a special case reasoned about separately.
- Tests: `mcp_create_project_onto_an_existing_file_sets_is_error_and_changes_nothing`
  covers the code #313 named; the pre-existing `measure` "neither input mode named" test
  and the `validate`-on-malformed-JSON test are updated from asserting `isError: false`
  to asserting `isError: true`, since both were asserting the gap this ADR closes.
