---
status: accepted
amends: 0043 (narrows "every `error`-class finding" to findings about a document; the four process-level codes #188 introduced carry no `repair` field, their remedy stated in the finding's own message text)
---

# Process-level errors carry no `repair` field: ADR-0043's binary is about the document

> **Amended by [ADR-0080](0080-the-scaffold-writes-what-it-was-told-and-the-two-resources-are-named.md).**
> One of the three codes left open under *"Not settled here"* below is settled:
> **`E-PROJECT-EXISTS` moves to `RepairClass::NotAboutDocument`** and to exit 3, its remedy
> stated as message text. `E-NOT-A-PROJECT` and `E-SOURCE-MISSING` are untouched and keep
> that section's open question. Nothing about the four codes this ADR decided changes.
>
> **Amended by [ADR-0081](0081-mcp-iserror-tracks-notaboutdocument.md).** The
> `NotAboutDocument` classification this ADR introduces for the wire's `repair` field is
> extended to the MCP transport's `isError` flag: every code declared `NotAboutDocument`
> here now sets `isError` over MCP, not just an argument that failed to deserialise.

**Ticket:** [#224](https://github.com/MBehtemam/Montaget/issues/224), resolved by a
three-model court (Opus, Haiku, Fable — reproduce with `/court`), unanimous.

## The conflict

[ADR-0043](0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md)
requires every `error`-class finding to carry a `repair` field: the literal string
`"none"` (refuse) when the fix depends on author intent the document does not carry, or
a structured value (advise) when the fix is fully determined by the document, the media
on disk, and published rendering semantics. [#188](https://github.com/MBehtemam/Montaget/issues/188)
registered four `error`-class codes that fire before, or entirely outside, that
examination: `E-PARSE` (the file is not JSON), `E-READ` (the file would not open or
decode), `E-INVOCATION` (the command was wrong), `E-INTERNAL` (Montaget itself broke).

Forcing a document-shaped answer onto them reads wrong on all four, worked in #224:

- `E-PARSE` as refuse prints "no repair is determined by the document" beneath a caret
  pointing at exactly where a value belongs — there is no document to have determined it
  from.
- `E-READ` as advise promises a value shaped like a document edit; "re-save as UTF-8" is
  real advice, but it is not a change to a project file, which is what ADR-0043's
  advise-class shape (`{"value": …}`, applyable) is built around.
- `E-INVOCATION` as advise names a repair to the invocation, not to anything Montaget can
  write.
- `E-INTERNAL` as refuse claims "no flag can lift it" of a condition a retry might clear.

That the same fault line runs through all four — neither label reads correctly on any of
them — is what makes this an ADR question rather than four separate judgement calls.

## Decision

**ADR-0043's binary applies to findings about a document.** `E-PARSE`, `E-READ`,
`E-INVOCATION` and `E-INTERNAL` carry **no `repair` field at all** — not `"none"`, not a
structured value. Their remedy, where the condition names one (`E-READ`'s OS-derived
advice, `E-INVOCATION`'s usage text), is stated in the finding's own message/template —
the same "message-text property" ADR-0068's amendment already distinguishes from a
repair class.

The registry gains a third `RepairClass` arm, `NotAboutDocument`, alongside
`Advise`/`Refuse`: a finding whose subject is the invocation, the raw bytes, or
Montaget's own process rather than the document. The four codes above are reclassified
to it; no other registered code changes.

`Report::push`'s invariant — *"every `error`-class finding carries a `repair` field"* —
is narrowed to an **enumerated exemption**, not a per-instance judgement call: a code is
exempt only if the registry declares it `NotAboutDocument`. Nothing at a call site can
opt a finding out of the requirement; the same "decided once, by whoever authors the
check" discipline ADR-0043 already established for refuse/advise applies to this third
value.

## Why

### The court

[#224](https://github.com/MBehtemam/Montaget/issues/224) recorded the question, unresolved,
and named three shapes an answer could take:

**A.** The binary applies unchanged — cheapest, matches the code #188 shipped.
**B.** A third `repair`-field shape, or an explicit carve-out, costing a schema and
renderer change.
**C.** The binary applies only to findings about a document; the four process-level
codes carry something else, or nothing.

Three independent jurors — Opus, Haiku, Fable — answered blind to each other, given the
question and #224's own worked table. All three voted **C**, unanimous.

All three located the same mechanism: refuse-class's power is specifically the "there is
no document to reason about" case. ADR-0043's refuse sentence is non-bypassable and
deliberately strong precisely because the gravity-fork evidence it is built on showed 3
of 4 agents that *noticed* a fork in required repairs shipped a guessed wrong one
anyway — only refusal, not prose explanation or per-instance triage, stopped them. Two
jurors (Opus, Fable) argued the erosion runs in the direction that matters most: `E-PARSE`
is plausibly the single most frequent error an agent operating Montaget will ever see, so
printing the refuse sentence beneath its caret is not a neutral cost — it teaches the
agent that "no flag can lift it" is sometimes routine, which is exactly the erosion that
then misfires on the check the sentence exists to protect. Opus located the textual hook
directly: ADR-0068's amendment already distinguishes a "message-text property" (a
retired spelling naming its replacement) from a repair class, and the remedy on these
four codes is that shape, not this one.

Two jurors (Opus, Fable) independently rejected option B on the same ground from the
other side: a third `repair`-field shape spends a permanent schema and renderer change
on codes nobody will ever gate `repair == "none"` on, since these codes never reach the
decision the field exists to encode in the first place. Fable noted ADR-0043's own
"granularity is per check" language does not even apply to these four — they are not
checks; no check is being evaluated when the document failed to parse.

Full ballots are preserved in the #224 conversation transcript this ADR's ticket links.

### Why an enumerated registry value, not a looser `Report::push` exemption

`Report::push`'s panic exists so a check author who forgets to declare a repair fails
loudly rather than shipping a finding a consumer cannot classify. A blanket exemption for
"any error-class finding with no repair" would silently swallow that same mistake on a
future document check. Requiring the registry to say `NotAboutDocument` keeps the
exemption a declared, reviewable property of the code, checked at the same place and by
the same completeness test ADR-0043's refuse/advise decision already is — never a
per-instance escape a check author reaches for at a call site.

## Consequences

- `RepairClass` gains `NotAboutDocument`. `E-PARSE`, `E-READ`, `E-INVOCATION`,
  `E-INTERNAL` are reclassified to it. No other registered code changes.
- These four codes' JSON no longer carries a `repair` key at all — the field is already
  `Option<Repair>`, `#[serde(skip_serializing_if = "Option::is_none")]`, so an absent
  field is the correct wire shape without a schema change. The text renderer already
  treated an absent `repair` as "print nothing further"; it needed no code change.
- `E-READ`'s OS-derived advice ("check the path", "re-save the file as UTF-8", "vendor
  the declared font file", …) and `E-INVOCATION`'s usage text move into their templates
  as ordinary message-text fields, not a structured repair.
- `Report::push`'s invariant is narrowed: an `error`-class finding must carry `repair`
  unless its registry entry declares `RepairClass::NotAboutDocument`.
- `Finding::repair_value` continues to panic on anything but a declared `Advise` code —
  a `NotAboutDocument` code may not state a repair value either, the same refusal a
  `Refuse` code already gets.

## Not settled here

`E-NOT-A-PROJECT`, `E-PROJECT-EXISTS`, and `E-SOURCE-MISSING` are advise-class today on
reasoning identical in shape to `E-READ`'s retired reasoning — "there is no document
whose author could have meant X, so the next move follows from the condition itself" —
and the same fault line arguably runs through them. #224 scoped its question to the four
codes #188 introduced, and the court answered only that question; this ADR reclassifies
only those four. Whether the three above should also move to `NotAboutDocument` is a
separate call for whoever next touches them.
