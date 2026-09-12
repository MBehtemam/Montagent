# Verdict — ticket #14, format versioning and migration

## Headline

The question is not malformed, but it is **prematurely unified**: it asks for "a version" as
if one field could answer both "what does this document contain" and "under which rules
should it be interpreted," and A4 proves those are different questions with different
answers on an already-accepted ADR (0013). My preferred design is: **no version field today**,
a **pre-1.0 clause** that binds nothing until a stated release event, and when that event
arrives, **two separate mechanisms** — a single top-level shape integer (`"montaget": N`)
that a schema-diff can verify mechanically, plus an ordinary changelog/ADR trail (already
half-built as the accepted ADRs) that is the only honest home for semantics-only revisions
like ADR-0013. A tool can enforce the first; nothing can mechanically enforce the second,
and pretending otherwise is worse than admitting it.

## Part A findings

The reverse migration (A1) worked cleanly and reproduced ADR-0015's own claimed byte-diff
exactly (8 insertions, nothing else) — a good sign the ADR's self-description is accurate
and that this exercise is grounded in a real, checkable pair of files rather than a
hypothetical.

The genuine surprise was **A3**. I expected to have to work to construct an ambiguous file.
I didn't — ADR-0013's own measurement (`gravity` "inert on 8 of 8 elements") hands you the
adversarial case for free: because the field did nothing even under the old rules, a
pre-revision file that simply never bothered to write it is byte-identical to a
post-revision file. This falsifies, for me, any design that leans on shape-sniffing as the
*general* mechanism (candidate (c) in Q1) — it works only in the required-field-addition
direction and silently fails in the retired-optional-field direction, which is exactly the
direction ADR-0015 is an instance of. Before doing A3 I'd have said "shape-sniffing is a
weaker but workable fallback"; after it, I think it's workable only for a strict subset of
breaking changes and must not be relied on as the sole detector.

A4 was the second surprise, and it reframes Q1 and Q4 for me: I went into this brief
assuming "version" meant one thing. It doesn't. ADR-0013 is proof, already accepted, sitting
in `main`, that a revision can be real (it changes what `validate` must accept as correct)
while being completely invisible to any mechanism that reads the document's shape. That
single fact is why I did not converge on a single-field answer to Q1 — see below.

## Q1 — Is there a version in the file at all, and what is versioned?

**Preferred answer:** yes, but *only one thing*: a single top-level integer,
`"montaget": N`, naming a revision of the **document's shape** — the ADR-0014/0015 kind of
change (fields added/required/removed/renamed). It is explicitly **not** asked to capture
ADR-0013-kind changes (interpretation-only, zero bytes). Per-object OTIO-style versions (b)
are rejected: the brief's own health warning is correct to flag this as unratified, and A2
shows the whole document already has to satisfy one consistent generation's rules at once
(nothing here is composed from files of different vintages the way an OTIO archive can be),
so per-object granularity buys nothing an editor here would ever exploit. Shape-sniffing
alone (c) is rejected on A3: it has a real, non-contrived blind spot.

**Strongest attack on my own answer:** fact 6 — agents copy stale fields out of the fixture
before reading the spec (8/8 in #48's exercise). A hand-maintained integer is exactly the
kind of field an agent will copy verbatim from an example project and never update, which
means the field can be **confidently wrong** — worse than useless, because a stale-but-present
version number invites a validator to trust it and skip the real check, whereas no field at
all forces the validator to actually re-derive conformance from content. A confidently wrong
integer is worse than an absent one precisely because absence is legible as "unknown, go
check," while a wrong number is legible as "known, and false."

**Why it survives anyway:** the fix is not to drop the field but to make it advisory, never
authoritative — `validate` must always independently re-check the document against the
*current* schema regardless of what `montaget: N` claims, and must **flag** (not merely
correct) a mismatch between the declared number and the shape actually observed, the same
way a `Content-Length` header that lies about body size is a bug report, not instructions.
Under that design, a stale copied version number is caught by the same validation pass that
would have caught the underlying stale field anyway (fact 6's `gravity` case: `validate`
already flags `gravity` on its own merits, version number or not — see A2). So the field
adds a fast path and a piece of provenance for humans and tools, and loses none of its
value even when an agent gets it wrong, because nothing downstream trusts it unchecked.

**What would falsify this:** if it turns out `validate` cannot, in practice, cheaply
re-derive full shape-conformance without the version number (e.g., because two revisions
are shape-*compatible* but must be treated differently for some other reason I haven't
found), then the number stops being purely advisory and the agent-copies-stale-fields risk
becomes real and uncontained. I don't have a concrete case where this happens today, so this
is speculation flagged as such.

## Q2 — Does the policy fire now, or only after a stated 1.0?

**Preferred answer:** a pre-1.0 clause, stated in the spec: before a named release event,
the format breaks freely; the only binding obligation is the one `docs/agents/domain.md`
already imposes generally — a migration, when the fixture is affected, ships as a committed,
re-runnable script beside the ADR that caused it (exactly #42 and #48's pattern, already
proven twice). The versioning machinery itself (Q1's integer, Q3's tooling) is specified now,
in the same ADR that would ratify this ticket, but its *obligations* — "every ADR after this
one must bump N and ship a migration" — do not bind until the trip event. The trip event I'd
propose: **the first published release Montaget makes available for a consumer that is not
this repo's own fixture** (a tagged release, a package registry publish, or the first
external user's project file existing) — because that is the first moment a file can exist
that the ADR authors cannot simply re-migrate themselves. Who decides: whoever cuts that
release, since it's an operational act (a `git tag` / publish), not a design judgment call.

**Strongest attack:** "wait for a trigger" is exactly the kind of decision that never gets
revisited under its own steam — pre-1.0 policies have a well-known failure mode of becoming
permanent by default, because nobody's job is to notice the trigger fired. Fifteen ADRs have
already landed with zero version discipline; an ADR-0016 with "this doesn't apply yet" has
no natural forcing function to become "this applies now."

**Why it survives:** fact 8 is unambiguous — no renderer exists, no users, one project file,
which the maintainer controls completely. Under file-as-truth (a standing principle, not
reopenable here), Montaget never holds a database of files it must sweep; the *only* file at
risk today is the one committed fixture, which the domain doc's existing rule already
protects. Building the machinery to protect zero real files, before the moment a real
stranger's file could exist, spends effort on a threat that provably (fact 8) doesn't exist
yet, and the two precedents (#42, #48) show that when a real breaking change lands, the
project already does the right minimal thing without a Q1-style version field ever being
consulted. I don't have a strong rebuttal to "the trigger might get missed" beyond: make the
trigger a literal checklist item in the release-cut process (a real forcing function, not a
vibe), which is a process fix, not evidence the policy itself is wrong.

**What would falsify this:** a second real consumer's project file appearing in this repo
(or referenced by it) *before* any stated release — that would prove files-in-the-wild can
predate "release" as an event, and the trigger needs to be earlier (e.g., "first MCP server
publish," not "first release").

## Q3 — Whose files must survive N+1?

**Preferred answer: (b), files within reach.** A migration is a script committed beside the
ADR that caused it, exactly what #42 and #48 already did — and no, `montaget migrate` as a
standalone shipped verb does not need to exist yet. Under file-as-truth, Montaget genuinely
never holds a database it can sweep (a standing principle, undisputed) — so "(a) files
Montaget never wrote and cannot see" describes a class of files that, today, is empty by
fact 8, and a general migration tool built for it would be validated against nothing.

What a general migration tool buys over a per-change script, honestly assessed: **nothing
today**, and something real *later* — specifically, the ability for a stranger who has
`fmt`'d their file into some `N` and hasn't touched it since to run one command and land on
`N+current` without hand-collecting every intervening per-ADR script and running them in
order. A per-change script commits the *transformation*; a general tool commits the
*composition and sequencing* of all of them, which is exactly the part that gets harder,
not easier, the more ADRs accumulate. That composition problem does not exist yet (there is
one migrated file and it has been migrated exactly twice, both times by the project's own
author, both times by hand-invoking the right script). So: build per-change scripts now (Q2
already requires this once the pre-1.0 clause is discharged); revisit whether a general
`montaget migrate` verb is worth shipping once there are enough accumulated per-change
scripts that a stranger would plausibly need to chain three or more of them — that's a
measurable trigger ("N accumulated migrations"), not a vibe.

**Strongest attack:** this is exactly the "ship it when you need it" argument that starves
infrastructure until the pain is already expensive — by the time a stranger needs to chain
five migrations, Montaget has shipped and lost the ability to make this decision for free.
Also, ADR-0011's tool surface already treats the CLI as the home for exactly this kind of
batch/convenience operation (`fmt`, `timeline`, `probe` are all CLI-only), so there's a
live precedent for adding low-stakes CLI verbs cheaply, and `migrate` composing already-
committed per-ADR scripts is not expensive to build once two or three exist.

**Why it survives:** the attack argues timing, not substance — it doesn't dispute that a
general tool buys nothing *today*, only that waiting has a cost *later*. I accept the cost
is real but think it is cheap and reversible (a CLI verb that chains already-existing
scripts is a thin wrapper, not a redesign), whereas building the general tool now means
guessing at a composition/chaining interface with a sample size of two migrations, which is
exactly the kind of premature generalization CONTEXT.md's own "inert data, no evaluation"
and "general-purpose but not After-Effects-class" postures argue against elsewhere in this
project.

**What would falsify this:** if the two existing migrations (#42, #48) turn out **not** to
compose cleanly — i.e., running `migrate_0015.py` on `migrate_0042.py`'s output requires
manual reconciliation rather than clean sequential application — that would mean the
chaining problem is already hard with only two data points, which would argue for solving
the composition interface now rather than waiting for a third.

## Q4 — What counts as a breaking change?

**Preferred classification**, against the shape-integer from Q1:

| change | bumps N? | why |
|---|---|---|
| adding an optional field | no | old files remain legal; a reader ignoring an unknown-to-it optional field is not this format's problem (schema-catches-malformed still requires readers reject *truly* unknown keys, so this is about a field the *current* schema already names as optional — its addition to the vocabulary is the ADR event, and old files simply don't use it yet) |
| adding a required field | **yes** | ADR-0014, ADR-0015: an old file is now invalid until edited. This is the clean case A3 showed shape-sniffing *can* detect. |
| renaming a field | **yes** | old and new names look identical to "unknown field" handling; without a bump, a file frozen at the old name is silently wrong rather than loudly invalid |
| removing a field | **yes** | ADR-0015's `gravity`: the field becomes an error. Old files carrying it break. |
| changing a default | **yes, with a caveat** | if ADR-0012's six defaulted fields change what they default to, a file that relied on the old default now renders differently while remaining schema-legal — this is a real breaking change that shape-checking *cannot* see (structurally identical to ADR-0013, below), so it belongs to the *semantics* track, not the shape-integer, unless the spec chooses to make defaults part of "shape" by requiring them stated explicitly at some N (which is one legitimate way to resolve #61) |
| changing meaning with no spelling change (ADR-0013) | **no, on the shape integer — and that is the finding, not a gap I'm papering over** | A4 shows this concretely: zero bytes change, so nothing for a shape integer to react to. This is not a defect in my design to be patched; it is a demonstration that "the file's shape" and "the renderer's reading" are **different axes**, and Q1's integer only ever answers the first. ADR-0013-class changes are real and must be recorded, but in the ADR trail / changelog, not in a per-file field — because there is no candidate field placement that would make a *file* self-describe an interpretation rule its author didn't know existed yet. |
| removing an element type | **yes** | strictly harder than removing a field: every element of that type in every existing file is now unparseable, not just missing one key. Same bucket as required-field addition, more severe. |

**Strongest attack on this whole scheme:** the "changing a default" row is a fudge — I've
put it on both sides depending on how #61 resolves, which is exactly the kind of hedging
this brief tells jurors not to manufacture. A harder critic would say: if a default change
can silently alter rendered output for a file that never mentions the field (exactly
ADR-0012's six defaulted fields, exactly what #61 is open about), then it is *structurally
identical* to ADR-0013 and belongs in the same "shape-integer can't see it" bucket, full
stop — no caveat, no escape hatch through "make it explicit." Making a previously-defaulted
field explicit-and-required at some N *is* itself a required-field addition (my "yes" row
above already covers that), so the "yes, with a caveat" row is actually two different things
wearing one label: changing what an *omitted* field defaults to (semantics-only, ADR-0013
bucket, no bump) versus making a field newly mandatory (shape bucket, bump). I conflated
them.

**Why the corrected version survives:** splitting that row makes the table internally
consistent and sharpens the answer to the brief's last sub-question directly: **a version
that changes what a legal file renders to while changing zero bytes is a statement about the
renderer's reading, and it is a different field (or no field at all — an ADR/changelog
entry) than the one Q1 proposes for shape.** Trying to force both into one integer means
either (a) bumping it for changes that leave every existing file's shape-conformance
untouched, which breaks the property Q1's whole design depends on (that shape-conformance is
mechanically checkable against the number), or (b) leaving semantics-only revisions
unrecorded anywhere machine-readable, which is honest about what's checkable but means
nobody can ask "was this file's `1912` computed under floor or under ties-away-from-zero"
except by reading fifteen ADRs by hand — which, per fact 2, is the *current* state of the
world regardless of what this ticket decides.

**What would falsify this:** a demonstrated case where a consumer needs to mechanically
distinguish "rendered under ADR-0013's floor rule" from "rendered under whatever came
before" for a file that doesn't otherwise fail validation — i.e., a real cost from *not*
having a semantics-version field. I don't have one; today `validate`'s strict-equality check
(ADR-0015) makes the pre-ADR-0013 ambiguity moot by simply re-deriving and comparing, which
is cheaper than versioning it. That is itself a pattern worth naming: several of this
format's "what does this mean" questions (ADR-0013's rounding, ADR-0015's `fit`) turn out to
be **fully re-derivable from the document plus the current rule**, which is precisely what
"inert data, no evaluation" promises and is why this format may need less semantic
versioning than a format without that constraint.

## What I could not settle

- Whether "adding an optional field" ever needs a bump for a *different* reason than the one
  I gave: if `fmt` is defined to reject unknown keys strictly (I did not find this stated
  either way), then even optional additions become breaking for old *tooling* reading new
  files, which is the mirror image of the problem this ticket is about (old files vs new
  tooling) and isn't asked by Q4 as posed.
- #61 and #73 are both open and I was told not to assume either way; my Q4 "changing a
  default" split assumes #61 could go either way and tried to make the classification
  survive both outcomes, but I have not verified it actually does for every plausible
  resolution of #61 — that needs checking against #61's actual resolution once it lands,
  not decided here.
- Whether the "advisory, never authoritative" version number I proposed for Q1 has any real
  cost in implementation complexity (a second, independent shape-check path) versus just
  living without the field and always fully re-deriving. I asserted this is cheap; I did not
  measure it, because there is no `validate` implementation yet (fact 2) to measure against.
  This needs measuring, not deciding, once a schema exists.
- I did not find anywhere in the ADRs a stated position on whether the shape-integer, if
  adopted, is validated *against* the JSON Schema resource ADR-0011 says ships as an MCP
  resource, or is free-floating prose in a doc. If the schema doesn't exist yet (fact 2),
  "the version means what the schema at revision N says" is aspirational until that artifact
  exists — this ticket may be partly blocked on that artifact rather than being independently
  resolvable now.

## Confidence

- **Q1** (single shape-integer, advisory not authoritative, no per-object versioning):
  **medium-high**. The per-object rejection is high-confidence (A2's own document has no
  compositional structure across files that would exploit it). The "advisory not
  authoritative" design is the part I'm least sure survives contact with a real
  implementation — it depends on `validate` being cheap enough to always run in full, which
  I have not measured (no `validate` exists yet).
- **Q2** (pre-1.0 clause, trigger = first external-facing release): **medium**. The
  substance (don't build machinery for zero real files) is high-confidence given fact 8; the
  specific trigger event is a judgment call I'm not fully confident in — "first release" vs
  "first MCP server publish" is a real fork I flagged as a falsifier and haven't resolved.
- **Q3** ((b), per-change scripts, no general tool yet, revisit at N≈3 chained migrations):
  **medium-high**. The precedent (#42, #48) is concrete and strong; the specific numeric
  trigger ("three or more") is a guess, not measured, and I say so.
  **Q3 in bullet form: this is genuinely an "attack survives" case, not a manufactured one**
  — I think the composition-cost argument is real and I could not fully rebut it, only argue
  the cost is deferrable.
- **Q4** (table above, with the corrected split on defaults): **medium**. The required/
  removed/renamed rows are high-confidence and mirror the brief's own worked facts directly.
  The "changing a default" split and the "removing an element type" row are the parts I'm
  least sure about — the latter because no ADR has yet removed an element type, so it is
  argued by analogy rather than evidence, and I flag that as a straightforward weakness: this
  row is speculation, not a measured case like the others.
