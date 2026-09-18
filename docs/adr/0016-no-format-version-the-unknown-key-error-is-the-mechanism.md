# ADR-0016 — The project file carries no version number; the unknown-key error is the migration mechanism

> **Amended by three later ADRs.** Read them before relying on anything below.
>
> - [ADR-0017](0017-closed-schema-no-escape-hatch.md) — settles the unknown-key policy
>   ADR-0016 requires and does not take
> - [ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) —
>   closes the "published key order" entry left unsettled there
> - [ADR-0042](0042-montaget-json-is-a-convention-fmt-gets-a-shape-check.md) — discharges
>   its file-identification deferral

**Status:** accepted
**Ticket:** [#14](https://github.com/MBehtemam/Montaget/issues/14)
**Amends:** [ADR-0011](./0011-tool-surface-reads-checks-renders.md) (the tool surface gains no `migrate` verb)
**Requires:** an unknown-key policy, which no accepted document states — see *Consequences*

## Decision

A Montaget project file **carries no version number**. There is no
`"montaget": N`, no `$schema` revision, no per-object version.

A file authored against a newer revision than the binary reading it is caught by
the **unknown-key error**, which names the binary as the suspect and forbids
deletion as the repair:

```
error  photo-05: unknown key `fit` — not a key this Montaget knows, and not one
       it has retired. It may belong to a newer format revision than this binary
       implements. Check your Montaget version before removing it.
       Do not delete the key to make the file validate.
```

There is **no `montaget migrate`**. A retirement whose repair is arithmetic ships
as a script beside the ADR that caused it. A retirement whose repair requires
knowing what the author meant is **refused**, not guessed.

## Why not a version number

The question was worked for twenty-seven agent sessions across four rounds, and
the answer reversed twice as evidence arrived. Round 1 was 4–1 *for* a number.
Round 2 went 3–2 *against* once a convener error was corrected. The deciding
evidence is measured, not argued, and is in
[`docs/research/juries/format-versioning/`](../research/juries/format-versioning/).

### The number's only exclusive job has no instance

The case for a number rested on one claim: a semantics-only revision changes what
legal bytes *mean* while changing no bytes, so only a declared number can catch
it. That claim is false in general, and the ticket's own framing of it was wrong.

A legal pre-ADR-0013 file written by a float implementation may declare
`width: 1919` where exact integer arithmetic gives `1920` — `103 * (1920/103)` is
`1919.9999999999998`. Under [ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md)'s
strict equality that is now an **error with the repair value printed**. The
semantics-only class is largely absorbed *into checkable shape* by this format's
own trajectory, because `fit` is a derivation claim and a derivation claim can be
re-derived. Where a genuine residue survives, no document field helps: **no field
placement lets a document self-describe an interpretation rule its author never
knew existed.**

The committed fixture escaped byte-free only because `1080/1536 = 45/64` is
dyadic. ADR-0013's own founding measurement puts the float/exact disagreement at
**4.466% of 31,402,800 combinations**.

### A bump must not oblige a correct file to be rewritten

The fixture is byte-identical at ADR-0012, ADR-0013 **and** ADR-0014 — three
consecutive accepted revisions, one blob (`cd8e7971…`) — not because it is small
but because it is **correct**, and correct files are exactly what a tightening
leaves alone. Under equality semantics those three revisions would each have
obliged a rewrite of every correct file in the world as pure ceremony. So a
number, if one existed, could only ever be a **lower bound** — which makes it
weak enough to lose its own argument.

### Measured: the number is a one-character switch for its own guard

Nine agents met a stale binary (`montaget 0.9.2`) holding the current fixture,
under deadline pressure, no network, nobody to ask. Three arms, varying only the
number and the message.

| arm | outcome |
| --- | --- |
| **A** — number + "your file is newer, upgrade the binary" | 2 preserved and escalated; **1 changed `"montaget": 4` to `2`** and shipped |
| **B** — no number, unknown-key error naming the binary | 1 preserved and escalated; 2 compensated correctly |
| **C** — no number, bare `unknown key` list | 2 compensated correctly; 1 squashed the photos |

**Nobody deleted the unknown keys. 0 of 9** — the failure a number is bought to
prevent did not occur even in the bare control. Every agent identified `clip` as
load-bearing before touching it, and four independently invented the *same*
repair: an opaque cream rect between the photo track and the cards.

The one agent that defeated a guard was in arm A, and it did so by lowering the
integer — reasoning, not unreasonably, that *"the revision declaration is a
compatibility claim, not content."* It then listed the silent failures it was
accepting and shipped anyway. **Arms B and C offered no comparable cheap defeat:**
beating them meant editing sixteen keys across sixty elements, and nobody did.

A version number does not merely fail to help. It is the only mechanism tested
that can be switched off by editing one character, and it was.

### The stale-binary population is real, and it still does not carry a number

The last argument standing for a number was the **stale binary** — an old
Montaget meeting a newer file. That population is non-empty by decisions already
taken: ADR-0009 binds the stdio transport and *"a binary, plus an `ffmpeg` the
user supplies"*; ADR-0011 puts `probe`, `fmt` and `timeline` on the CLI only;
ADR-0010 says Montaget *"does not need to be a resident server"*. Hosted-only is
**excluded**, not unchosen. And file-as-truth generates the population directly:
the project file lives in git and travels to other machines without a binary.

It still fails on its mechanism. The signal is agent-maintained, and a file
authored at N+1 that **declares N** — the copied-header case, which fact 6 of the
round-1 brief makes the likely one — satisfies `N <= R_self`, so the guard never
fires and the unknown key is met exactly as before. **The guard fires only when
the number is already right, which is to say only on files whose author was
already careful.** Meanwhile a stale binary is the one reader that cannot
cross-check a version claim, so a wrong number is an authoritative warrant to
misread, delivered to the only consumer.

The instruction the argument wants is available without it. The old binary
already holds the decisive fact — *"this is not a key I know"* — which is about
the binary, computed by the binary, never itself stale, and true on every
encounter including the copied-header one. **Pointing the repair at the binary is
a property of the message text, not of a field in the document.**

## Why no `migrate`, and why no retired-spelling table

`CONTEXT.md` already justifies `shift` as *"the one edit that is arithmetic
rather than authorship, and therefore the one Montaget performs instead of the
agent."* Migrations divide on the same line, and both halves were measured.

**The arithmetic half is reliable.** Six agents repairing the real pre-ADR-0015
file removed all eight `gravity` keys and left the six inert elements untouched,
6/6, preserving formatting. Nobody needed a tool.

**The authorship half is beyond every mechanism, including a tool.** Where
`gravity` was *not* inert, the correct repair moves the rect to `y = -612`, and
the same error message fires either way. Four of six agents **found** the fork,
wrote it down, and three of those four shipped the defect regardless — because
nothing in the file, the message or the glossary carries the author's intent. A
better sentence does not fix this: they already knew. Only the author knows.

So a retirement whose intent the document does not encode must be **refused with
the element ids named**, which ADR-0006 already makes `validate`'s job and
`render`'s unconditional duty. A `migrate` verb would be a second printer of the
same list — and for the hardest case, removing an element type, a tool cannot act
at all: `CONTEXT.md` already names that replacement (*"Commit an SVG or a PNG
instead"*) and authoring an asset is something an agent can do and a program
categorically cannot.

**A retired-spelling table is rejected, against three independent proposals for
one, because it was measured and it made things worse.** Agents given
`CONTEXT.md`'s `Gravity` entry alongside the error solved the fork **0 of 3**;
agents given only the error solved it **2 of 3**. The entry describes a
retirement where the field happened to be inert, and reads as a general rule that
the value never mattered. One model computed `-612` correctly and then rejected
it on the entry's authority. This is ADR-0006's manufactured-confidence failure
arriving through the document meant to prevent it.

## What actually carries a migration

Both experiments point the same way. Where the document's own geometry encoded
the author's intent, agents recovered it unaided — the `clip` line at y=1300
against the topmost caption at y=1324, *"a 24 px gutter, too tidy to be
accidental"*, found independently by four agents. Where it did not, no metadata,
message or glossary recovered it.

**Internal evidence decides a migration; metadata does not.** That is file-as-truth
doing the work it was chosen for.

## Consequences

- **The schema must be closed to unknown keys**, or this ADR's mechanism does not
  exist. No accepted document states any unknown-key policy — zero occurrences of
  `additionalProperties`, "unknown key/field" or "unrecognised" across fifteen
  ADRs and `CONTEXT.md` — which also makes **ADR-0015 unimplementable as written**,
  since *"`gravity` is a schema error"* presupposes the rejection. Settling it is
  [#75](https://github.com/MBehtemam/Montaget/issues/75); it blocks publication of
  the schema resource, not this ADR.
- **`validate` and `render` must distinguish two unknown-key cases in the message
  text**: a key that may be from a newer revision (name the binary, forbid
  deletion) and a key this binary has retired (name the replacement). The retired
  case must additionally **refuse rather than advise** when the repair depends on
  intent the document does not carry. Designing both is
  [#78](https://github.com/MBehtemam/Montaget/issues/78), which amends ADR-0006's
  report format.
- **The tool surface does not grow.** No `migrate` verb; ADR-0011's eleven stand.
- **A retirement ships its migration as a script beside its ADR** where the repair
  is arithmetic. Note this has never actually been done: `migrate.py` is a
  cumulative regenerator from a fixed origin, amended in place at each ADR, not
  an N→N+1 chain. It does regenerate the fixture byte-identically from `main`
  alone; only its docstring is stale
  ([#79](https://github.com/MBehtemam/Montaget/issues/79)).
- **`CONTEXT.md`'s `Gravity` entry must be corrected** — it is measurably a defect
  generator in its current form ([#76](https://github.com/MBehtemam/Montaget/issues/76)).
- **If a marker is ever wanted for file identification**, that is a separate
  decision from versioning and must not be an integer, which every agent will
  read as a revision. The `.montaget.json` convention is used by both project
  files on `main` and specified nowhere — settled by
  [ADR-0042](./0042-montaget-json-is-a-convention-fmt-gets-a-shape-check.md)
  ([#77](https://github.com/MBehtemam/Montaget/issues/77)): documented
  convention only, no marker.

## Reopening condition

Falsifiable, and named by the jury rather than invented here: exhibit a change
that is (i) mechanical, (ii) not expressible as a replacement-naming error
message, and (iii) needed by files outside this repository. Three revisions of
history have produced zero candidates. Alternatively, measure agents authoring
against a fixture that carries a version integer and show they keep it current at
a high rate — the whole anti-number case turns on their not doing so, and that
rate is unmeasured.

## Not settled here

- The unknown-key policy itself (#75), which this ADR requires and does not take.
- Whether `fmt` materialises defaulted fields (#61). It sets the severity class of
  every future default change, but this ADR contains no sentence whose truth
  depends on the answer — deliberately, so it does not block.
- Published key order (#73). Three of nine three-arm agents pretty-printed the
  fixture (154 lines → 1146, 1165, 1243), destroying the one-element-per-line
  convention, and nothing forbids it.
- What a `refuse rather than advise` finding looks like in the report format
  ADR-0006 defines (#78).
