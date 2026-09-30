---
status: accepted
amends: 0047 (names who may consent through `release`, and rewords `E-SHIFT-SLACK` so it no longer invites self-consent), 0043 (scopes its stop-and-surface instruction for a refusal whose fork is in the edit request rather than the document)
---

# `release` carries the requester's consent, and surfacing is the default

**Ticket:** [#506](https://github.com/MBehtemam/Montagent/issues/506). The ruling is in
[its resolution](https://github.com/MBehtemam/Montagent/issues/506#issuecomment-5907473566).

## The contradiction

`E-SHIFT-SLACK` is refuse-class. ADR-0043 tells an agent that meets a refuse-class finding to
*stop, and surface the finding verbatim to whoever is operating Montagent*. But the message
itself said *"Release it explicitly with `release: [[{from}, {to}]]` if that is intended"*,
which tells the same agent how to continue on its own. ADR-0123 ruled that `release` is not a
bypass of ADR-0043's no-override guarantee, because it changes the edit and names the exact fact
it consents to. That settled whether consent through `release` is legitimate. It did not settle
**who** may give it.

The record pointed both ways. ADR-0032, where `release` began, says *"the agent sees the
constraint, decides, and the decision is in the diff"*, and ADR-0047 calls releasing
*"transcription, not inspection"*. ADR-0043 was written for the unattended agent under pressure
to ship, who in the gravity experiment noticed the fork and shipped a guess anyway.

## Decision

**The intent `E-SHIFT-SLACK` asks about belongs to whoever requested the edit.** An agent may
type `release` on its own only when the instruction it was given settles that slack's fate,
either explicitly or by unavoidable implication. Otherwise it surfaces the finding. **Surfacing
is the default.**

- **The fork is in the request, not the document.** `gravity`'s refusal was about meaning the
  document had lost, which only its author knew. `E-SHIFT-SLACK` is about a file that is legal
  and unchanged, because `shift` wrote nothing. What is unknown is whether the instruction
  behind this `shift` meant to change the slack. The party who knows is the one who gave that
  instruction, and that is not necessarily the file's author.
- **When the instruction speaks, `release` carries consent given upstream.** *"Make item-07's
  narration 800 ms longer and push everything after it"* consumes the lead-out, and *"close the
  gap after the title"* names the slack. Pasting the pair back is then transcription, as
  ADR-0047 said. The agent is not consenting for itself.
- **When the instruction is silent, the agent stops.** *"The slack looks unimportant"* is not
  the instruction speaking. By this test, #64's prompt (*"make the edit so the file stays
  legal"*) decides nothing about the lead-out, so all four agents in that study would have had
  to surface it. That is the right outcome, because they split on it.
- **The message states the test.** `E-SHIFT-SLACK` now reads: *"…Release it with
  `release: [[{from}, {to}]]` only if the instruction you were given decides this slack's fate;
  otherwise surface this finding verbatim to whoever is operating Montagent."* `shift`'s MCP
  tool description says the same. The old wording (*"if that is intended"*) left it open whose
  intent counted, and an agent reads that as its own.

This reconciles the two ADRs without overruling either. ADR-0032's *"the agent decides"* covers
the decisions the agent was actually handed. ADR-0043's *stop and surface* covers the rest, and
its meaning is unchanged: *the fix depends on intent you do not have.*

## Evidence

A three-juror court (Opus 5.5, Sonnet, Fable) voted unanimously for this option, and each
juror independently added the strict test above: surfacing by default, and the message
rewritten to state the condition. The human ratified it.

- **Rejected: the caller, with no limit.** It makes pasting the pair back free, which is the
  path the gravity experiment says an unattended agent takes first. It is also #64's pattern:
  agents split on whether the lead-out mattered. The call log records the decision, but it
  does not make it the right party's decision.
- **Rejected: the file's author, always.** It asks the wrong party. The author has no special
  knowledge of what a new edit request meant. It also turns ADR-0047's explicit release into
  a permission form, with a human round-trip even for restructurings the operator asked for
  outright.

**Caveat on the court.** All three jurors called the brief's request-versus-document framing
decisive, and that framing was the Judge's. Read the unanimity as *the framing survived three
models*, not as three independent discoveries.

## Consequences

- **The test is guidance, not enforcement.** Montagent cannot see the agent's instruction, so it
  cannot check whether that instruction decides a slack. The diff and the call log are the audit
  trail. All three jurors named this limit and none proposed a mechanism. Any future attempt to
  enforce it needs its own evidence.
- **`E-SHIFT-SLACK` stays refuse-class, and its JSON is unchanged.** Only the text template and
  `shift`'s tool description change. `from` and `to` are the same fields.
- **This is scoped to `release`.** It does not grant a general permission for an agent to act on
  refuse-class findings whenever its instruction seems to cover them. Another refuse-class code
  gets a consent handle only by meeting ADR-0123's definition, and who may use that handle is
  decided per code, as it is here.
