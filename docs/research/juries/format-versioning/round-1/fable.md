# VERDICT — jury #14: format versioning and migration (juror on Fable)

## Headline

One top-level integer, `"montagent": <n>`, naming a revision of the **contract**
(shape *and* interpretation, so ADR-0013-class changes bump it), introduced from
ADR-0016 onward under an explicit pre-1.0 clause: pre-release, bumps are free
and the only obligation is the committed re-runnable script the repo already
imposes; the *promise* — a shipped, idempotent `montagent migrate` built from the
chain of those scripts — begins at the first tagged release and covers released
revisions only. Per-object OTIO-style versions are rejected on this project's
own invariants, and no-field shape inference is refuted by Part A: the committed
fixture is itself byte-for-byte legal under two different revisions, and the
ADR-0013 class has no shape signal even in principle. The question is not
malformed, but Q4 exposes that it is really *two* questions wearing one coat —
"what is the version a statement about?" is prior to "where does it live" — and
answering the prior question (the renderer's reading, not the file's shape) is
what makes the other three answerable.

## Part A findings

Full detail in `WORKLOG.md`; the four results that drive Part B:

1. **The reverse migration was mechanical forward, lossy backward.** My
   reconstruction of the pre-#48 file is byte-identical to the real commit
   (`diff` empty against `git show '3b795255^:...'`) — but only because the git
   diff told me which 8 elements had carried `gravity` and with what values.
   The N+1 file alone does not contain that information. Removal migrations are
   one-way functions.
2. **Shape inference cannot say "this file is current."** Element-key union of
   N minus N+1 is exactly `{gravity}`; N+1 minus N is empty. So the only shape
   signal is one-directional, and — the finding that surprised me — **the
   committed fixture at HEAD is itself the ambiguous file** for A3: it carries
   zero keys unknown at N (gravity was optional; `fit` was already on all 8
   raster elements pre-0015), so it is a legal file of both revisions and
   nothing in it says which. I expected to have to construct an adversarial
   case; the repo had already committed one.
3. **"Needs migrating" and "is wrong" are the same observable.** Given only
   vN.json and the ADRs, the tool's evidence is a schema error on `gravity` —
   the identical output a *fresh* file gets when an agent copies `gravity` out
   of an old example, which fact 6 says is the most common way the key appears.
   To know my file was an old *valid* file I had to read the commit history,
   which a stranger's repo does not carry.
4. **ADR-0013's zero-byte-ness is a coincidence of the fixture's numbers, not
   a property of the revision.** Worked in the log: a legal pre-0013 file may
   declare `width: 1919` (the float implementation's own output for sw=103,
   bw=1920); the post-0013/0015 rule makes 1919 a strict-equality *error* and
   the migration must rewrite it to 1920. The fixture crossed unbytechanged
   only because 45/64 is dyadic. This contradicted my prior — I had read fact 5
   as "semantic revisions never touch bytes"; in general they do, for files the
   project cannot see.

## Q1 — Is there a version in the file, and what is versioned?

**Preferred answer:** (a) one top-level integer, `"montagent": <n>`, versioning
the whole-format **contract** — the pair (schema shape, normative reading). One
line, first key of the file, written by `create_project`, checked by
`validate`.

**Strongest attack:** fact 6, taken seriously. Agents hand-maintain this field
and agents demonstrably copy stale fields — 8 of 8 reached for `gravity`,
several straight from the fixture before reading the spec. A stale
`"montagent": 3` in a world at 4 is worse than silence precisely in the one case
the field exists for: when 3→4 was semantic, the shape is consistent with
either number, `validate` cannot catch the lie, and a migration tool will now
*confidently* apply (or skip) a semantic rewrite. The field converts "unknown
revision, proceed with caution" into "known revision, proceed wrongly." A
second attack: the field is pure ceremony for a format whose every file
currently lives in one repo the author controls.

**Why the answer survives:** three reasons, in descending strength.

- *The alternative is refuted by construction, not by preference.* Part A
  showed shape inference fails in both available ways: the ambiguous file
  exists (it is the committed fixture), and the ADR-0013 class has no shape
  signal even in principle. Option (c) is not a lighter-weight competitor; it
  is a non-answer for exactly the revision class that is hardest to survive.
- *The stale-copy mechanism is element-scoped; this field is not.* Fact 6's
  measured behaviour is copying **elements** — neighbouring lines inside
  `tracks`. A single top-level scaffold key is copied when a whole file
  skeleton is copied, and the format already has a tool whose stated purpose
  is that agents never start blank (`create_project`, ADR-0011), which stamps
  it correctly. The residual risk — whole-file copying from stale external
  examples — is real but is *strictly narrower* than the attack assumes, and
  the fixture-teaches-by-copy mechanism cuts both ways: a fixture that carries
  the field teaches agents to carry it.
- *A wrong claim is more diagnosable than no claim.* The format already has a
  field that is purely an assertion checked by `validate` — `fit`, "a
  derivation claim, not a layout mode" (ADR-0015). `"montagent"` is the same
  species: a claim about which reading the author worked under. When the claim
  contradicts the shape (`gravity` present in a file stamped 4), `validate`
  can say so *and say which side is probably lying* — an error class that
  cannot exist without the field. When the claim is wrong and the shape is
  consistent, we are exactly where option (c) is *always*.

(b), OTIO per-object versions, is rejected on the repo's own invariants rather
than on taste: the survey's own next paragraph after "steal but simplify"
records that OTIO's "versioning machinery needs a library" and that this format
must be writable "by reading the schema alone"; per-object suffixes put a
hand-maintained version on all 60 elements (multiplying fact 6's surface by
60); and a bump would touch every line of a file deliberately formatted so that
edits produce minimal diffs — destroying the "diff that proved 'and nothing
else'" property ADR-0011 measured agents relying on. Montagent has one document
type evolving in lockstep under one spec; per-object versioning solves a
federation problem this project does not have.

**Directly answering the brief's planted question** — is a confidently wrong
version worse than no version? For the *shape-changing* revision class: no,
because the shape contradicts the stamp and validate catches it. For the
*semantic* class: a wrong stamp and no stamp are equally blind, but only the
stamp gives correct files (the overwhelming majority) a correct answer. Wrong
beats absent on expectation; the attack holds only if mis-stamping is common
AND uncatchable, which is measurable (below).

**What would falsify it:** a consumer exercise in the project's own style —
agents author and edit files against a stamped fixture across a staged revision
bump — showing stale/wrong stamps at gravity-like rates (≥ half of agents) *in
the semantically-consistent case validate cannot catch*. That would show the
field is confidently-wrong machinery and (c)+per-change scripts should win.

## Q2 — Fire now, or after 1.0?

**Preferred answer:** split the mechanics from the promise, and say so in the
spec. **Mechanics bind from ADR-0016**: the field exists, an accepted ADR that
changes the contract bumps it, and the migration is a committed re-runnable
script beside the ADR — which is only a one-integer extension of the rule
`docs/agents/domain.md` already imposes and #42/#48 already obeyed. **The
promise binds at the first tagged release**: from that event, `montagent
migrate` ships and released revisions stay migratable. The trip event is the
release ADR — the same instrument every other decision here uses — and the
maintainer decides, because pre-release there is no one else affected to
consult.

**Strongest attack:** this is ceremony with zero beneficiaries. Fact 8: never
shipped, no renderer, one file, all fifteen changes cost nothing. Binding at
ADR-0016 taxes every future pre-release ADR with a bump-and-script obligation
while the set of files needing migration is {the fixture}, which the domain
rule already covers without any integer. Worse, the integer will read ~20 by
release, or the project will be tempted to reset it at 1.0 — and a reset makes
the same field mean different things in different eras, a two-spellings defect
this project has retired twice (`center-center`, `#RRGGBBFF`).

**Why the answer survives:** the tax is one integer per breaking ADR and one
line in scripts that must exist anyway — measurably near-zero against the 15
changes behind us (#42 and #48 both already wrote the script; neither would
have been lengthened by ten minutes). What it buys is not compatibility — the
attack is right that nobody needs that yet — it is **rehearsal**: the first
exercise of bump-plus-migration happens while every file is in reach and every
mistake is free, instead of at first contact with strangers' files, which is
the most expensive possible moment to discover the process has a hole. Part A
found such a hole already (removal migrations are lossy backward; semantic
migrations may need media, since #42's script needed `ffprobe` dimensions) —
findable only by doing. On the counter about the ugly integer: ship 1.0 at
whatever the counter reads; the number is a revision counter, not marketing,
and continuity is worth more than aesthetics. No reset.

**What would falsify it:** two or three pre-release bumps executed with no
process learning — no script hole found, no ADR friction — would show the
rehearsal bought nothing and the pure post-1.0 clause was equally safe.

## Q3 — Whose files must survive N+1?

**Preferred answer:** (b) now, hardening into a narrow (a) at release:
per-change scripts committed beside the ADR remain the unit of migration
forever; at 1.0, `montagent migrate` ships as **the chain of those scripts made
idempotent by the version stamp** (apply steps > stamp, in order, restamp) and
covers released revisions only. Files in strangers' repos are the design
target *from the release boundary forward*, not retroactively.

**Strongest attack:** `montagent migrate` need not exist at all, even post-1.0
— and the attack is native to this project, not generic. Every Montagent user
has an agent by definition; ADR-0015's schema errors are deliberately
pedagogical ("a schema error naming `x`/`y`/`origin` and `clip`"); so the
migration story could simply be: `validate` names what is wrong and what
replaces it, the agent edits, git shows the diff. Shipping a migrator adds a
second writer of the file beside the agent, and A3's construction shows it
misfiring: an auto-migrator that drops `gravity` also fires on a *new* file
where the agent copied it expecting behaviour, silently erasing exactly the
teaching moment the schema error was designed to produce.

**Why the answer survives — what the tool buys, named in failures prevented:**

1. *Semantic bumps are beyond error-message pedagogy.* A validate error can
   name a replacement key; it cannot re-derive `1919 → 1920` across an
   ADR-0013-class change (A4). That rewrite is exact integer arithmetic with a
   right answer — which is this project's own published criterion for what
   Montagent does instead of the agent: CONTEXT.md defines `shift` as "the one
   edit that is arithmetic rather than authorship, and therefore the one
   Montagent performs instead of the agent." Migration is the second such edit.
   Leaving it to the agent reintroduces the exact failure ADR-0015 documented
   killing the loose rounding rule: eight agents, three tasks, two different
   legal files.
2. *Ordering and double-application.* A stranger's file may be several
   revisions old or half-migrated. N scripts run by hand can run out of order
   or twice; a chained migrator keyed on the stamp cannot, and idempotence
   falls out of the stamp for free (which is also the concrete answer to why
   Q1's field and this tool are one design, not two).
3. *The failed migration is a result.* Removing an element type (Q4's last
   case) has no mechanical target; a shipped migrator can refuse loudly and
   name the element ids. Hand-run scripts scattered in `docs/research/` of a
   repo the user has never cloned cannot refuse anything — they simply aren't
   found.

The misfire in the attack is answered by scope, not by softening: `migrate` is
CLI-only, never invoked implicitly by `validate` or `render`, and refuses to
run on a file whose stamp already equals current — so the new-file-with-stale-
key case still gets the pedagogical schema error, untouched.

**What would falsify it:** if, at the first real post-release breaking change,
agent-driven migration guided only by validate errors reproduces files
byte-identical to the script's output across models (a rerun of the #48-style
consumer exercise), the deterministic migrator bought nothing and (b)-forever
wins.

## Q4 — What counts as a breaking change?

**Preferred answer:** the version bumps on **any accepted change to the
normative reading of a legal file** — equivalently: bump iff there exists a
byte-stream whose validity or whose rendered meaning differs across the change.
Classification of the brief's seven cases:

| change | bumps? | migration |
| --- | --- | --- |
| add optional field | **yes** | identity |
| add required field (0014, 0015 `fit`) | yes | synthesis — may need media (#42 needed ffprobe) |
| rename field | yes | mechanical rewrite |
| remove field (0015 `gravity`) | yes | drop key; **lossy backward** (A1) |
| change a default (ADR-0012's six; #61 open) | conditional — see below | rewrite omissions to old literal value |
| meaning change, zero spelling change (0013) | **yes** | verify-and-restamp; rewrites bytes where the file isn't at the fixed point (A4) |
| remove element type | yes | none mechanical — migrate refuses and names ids |

Two lines need defending. *Add optional bumps* even though old files stay
valid, because this schema rejects unknown keys (`gravity` is a schema error;
"Schema catches malformed"), so a **new** file breaks an **old** reader — the
bump is what turns that reader's "unknown field" into the true diagnosis "file
newer than me." *Change-a-default* is the nastiest case and is honestly
conditional on open #61: if `fmt` materialises defaults, no legal file omits
the field and the change is spec-text only; if defaults stay omittable, it is
a full semantic break — ADR-0013 in a cheaper costume — and must bump with a
migration that writes the old literal into every omitting file. I flag, as
this ticket's contribution to #61 rather than a verdict on it: versioning
pressure is a previously uncounted argument on the materialise side, because
materialised defaults delete an entire class of zero-byte semantic breaks.

**Why this line and not one field over:** because the number's one consumer is
a reader asking "may I apply my semantics to these bytes?", and Part A showed
shape answers that question only for shape changes. Any narrower line — "bump
only on shape breaks" — makes the field lie exactly in the ADR-0013 case, the
only case where it is irreplaceable. The brief's direct question: the version
is a statement about the **renderer's reading**, not the file's shape — A4 is
the proof, since 0013 changed the reading of unchanged bytes — and it is the
**same field** as Q1's. Two fields (shape-version + semantics-version) would
give agents two hand-maintained numbers that can disagree, with no consumer
that ever wants one without the other: no tool exists that should accept the
right shape under the wrong reading.

**Strongest attack:** the everything-bumps line makes the integer churn, and
churn is where fact 6 bites — the more often the current number changes, the
more often a copied example is stale. A minor/major split (SemVer-ish) would
let additive changes ride without invalidating agents' memorised number.

**Why the answer survives, partially:** the churn is real but priced wrong by
the attack. A stale number that is *older* than current is the designed-for
case — migrate handles it by construction, and for additive hops the migration
is the identity, so the stale copy costs a restamp and nothing else. A
two-part number, by contrast, doubles the hand-maintained surface and imports
a compatibility algebra ("which minors may I read?") whose every consumer here
is an agent that should just run `migrate`. I note this is my least attacked
conviction: I cannot construct a *serious* failure of "single integer, bump on
everything, identity migrations are cheap" beyond aesthetic churn — which, per
the brief's instruction, I flag as a possible sign the thinking isn't finished
rather than manufacture a weak attack.

**What would falsify it:** a demonstrated failure mode in which identity-bump
churn causes agents to mis-author at rates that a frozen-integer-plus-shape-
inference world would have avoided; or #61 resolving to materialised defaults
*and* an audit showing all remaining semantic-break classes are similarly
removable, which would shrink the semantic class enough to reopen
shape-versioning as a cheaper answer.

## What I could not settle

- **#61 decides how big the semantic-break class is.** Materialised defaults
  delete the change-a-default break entirely. This ticket's design should
  state its default-change rule conditionally rather than pre-empting #61.
- **Where the current number lives for agents.** The schema resource
  (ADR-0011) must state the current integer, or agents will learn it only from
  the fixture — re-creating fact 6's copy-from-example channel as the primary
  channel. Needs a decision, not a measurement; but it belongs to the schema
  ticket, not this one.
- **Whether migrations may require media.** #42's script needed ffprobe
  dimensions; a stranger's repo may lack the media (`UNCHECKED` sources,
  ADR-0015). Whether `migrate` may stamp a file it could not fully verify —
  and how it says so — is a real open design point this brief does not ask.
- **Needs measuring, not deciding:** the Q1 falsifier — mis-stamp rates in a
  staged consumer exercise across a revision bump. The project has run exactly
  this experiment shape twice (#48, #9) and it changed outcomes both times.
- **Backward migration / downgrade** (OTIO has it; A1 showed removals are
  lossy backward). I believe the answer is "out of scope — git is the
  downgrade," which is file-as-truth doing its job, but no one has said it in
  writing.

## Confidence

- **Q1: high** on "a version exists in the file" and on rejecting (b) and (c)
  — (c) is refuted by construction, not judgment. Medium on the residual
  stale-stamp risk; that is the falsifier to run.
- **Q2: medium.** The mechanics/promise split is reasoned, not measured; the
  pure post-1.0 clause is a defensible neighbour and the cost difference is
  small. This is the question where a convener could most reasonably overrule
  me.
- **Q3: high** on per-change scripts remaining the unit and on the
  shift-precedent argument for a deterministic migrator; medium on whether
  `migrate` must exist *at 1.0* versus when the first post-release break
  actually lands (shipping it lazily at that moment loses little).
- **Q4: high** on bump-on-any-normative-change and on the version being a
  statement about the reading; **low** on my dismissal of the two-part number,
  flagged above as the one place I could not build a serious attack on my own
  answer.
