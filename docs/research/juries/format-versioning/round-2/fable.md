# VERDICT-R2 — juror "fable", round 2, ticket #14

## Headline

One top-level integer, `"montagent": N`, lower bound, value `1` until first
release, whose **sole designed consumer is an older binary refusing to
mis-repair a newer file** — and whose defense against fact 6 (agents copy
fields stale) is that validate can check it in both directions, which no other
position in the round provides. `montagent migrate` never exists; its real
content is a **retired-vocabulary table inside validate** (retired key/type →
retiring ADR → successor recipe or explicit "no successor"), which my A2 run
shows is sufficient for the only real migration this project has ever performed
(byte-identical repair from error messages), and my A3 run shows is *equally
powerful* to a migrate tool on the hard case, because the hard case is
authorship and no tool may perform authorship. #61 does not block #14; the
unknown-key policy is a new ticket.

## Part A findings

**A1.** M1 re-verified by `git rev-parse`: three consecutive accepted revisions
(afc12d24, ed3db37c, 2f8e9405) share blob `cd8e7971…`; 3b795255 moves to
`a2e64f7c…`. C1's arithmetic confirmed by running it: `103 * (1920/103)` =
1919.9999999999998; my own scan against box dims (1080, 1920) reproduces the
divergence on **5.74%** of source widths 1..4096 — the brief's ~5.8%. So a
legal pre-0013 float-written file can carry `width: 1919`, now an error under
0015's strict equality, and migrating it rewrites bytes. C2 also confirmed
first-hand: `migrate.py` reads `old.json` (the #9 prototype) and has been
edited in place by each ADR (three commits total, per `git log`); it is a
cumulative regenerator from an origin only the author holds, not an N→N+1
chain. C4 confirmed: the grep returns nothing.

**A2 — the core result.** I took the real pre-0015 file, emitted the eight
`validate` errors ADR-0015 specifies ("unknown key `gravity` — retired;
position is x/y/origin, visible region is clip; remove it"), then repaired
using only those messages and the ADRs, by exact-string replace. **The repair
is byte-identical to the real post-0015 file** (`cmp` clean).

What I had to guess — the honest inventory:

1. **Nothing about retired values.** M2 (only git witnesses `gravity:"center"`)
   is about *reconstructing the old file from the spec*, which no migration
   ever needs to do. Forward repair holds the old file; the value is in the
   bytes in front of you. M2 is true and irrelevant to Q8.
2. **One judgment call**: that *deletion* is the repair, rather than
   re-expressing gravity's value through `clip`. The error string alone does
   not settle this; ADR-0015's structural argument ("no degree of freedom left
   for gravity to spend") does. Note carefully: a migration tool would embody
   the *same* judgment — this is not information a tool has and an agent lacks.
3. I re-ran the cover arithmetic (1080x1912 from 1536x2720 into 1080x1300;
   68x68 from 800x800) to confirm the surviving extents pass strict equality.
   They do. An agent that skipped this check would still have produced the
   right file here, but only by luck; the check is cheap and the ADR tells you
   how to run it.

**A3.** Retired `ellipse` (real vocabulary: CONTEXT.md defines `rect` and
`ellipse` as separate element types; an ellipse inscribes its declared rect; a
rect may declare `radius`). The validate message can carry the whole repair
for the **circle sub-case** (rect + radius = width/2 is the same shape —
mechanical). The **non-circular sub-case** is unresolvable by *any* mechanism,
migrate included, because what is missing is a decision the format can no
longer express — approximate, pre-render to an image, or delete is a choice
about what the author meant. The best any mechanism can do is refuse loudly,
name element ids, and enumerate legal options — and the error message above
does exactly that, in situ. **M3 therefore does not discriminate between
validate-errors and migrate**; it discriminates between mechanisms that carry
a retired-vocabulary table and mechanisms that don't.

**A4.** The fixture directory itself contains a JSON that is not a project
file (`transcript.json`), so "which JSON is the project?" exists even at home.
Identification today is the `.montagent.json` extension (defeated by
rename/pipe) or sniffing the key conjunction {frame, fps, background,
duration, output, fonts, tracks} — a heuristic. Identification **alone**
justifies a marker only weakly: the failure prevented ("not a Montagent
project" instead of a 60-error spray an eager agent might "repair") is real in
kind but unmeasured, and the extension covers the common path. **The marker
and the version are separable decisions** — and on my Q5 answer the direction
of P2's free ride reverses: the number is independently justified, and
identification rides on it.

## Q5 — marker at all, and is it a version?

**Preferred answer: P1, sharpened.** One top-level integer `"montagent": N`
naming a revision of the whole contract, with two non-negotiable riders:

- **Its designed consumer is the stale binary.** The failure it prevents is
  specific to agent-first and nobody in round 1 named it cleanly: an old
  binary meeting a newer file reports either nothing (lenient unknown-key →
  silent misrender) or "unknown key `fit`" (strict) — and fact 6's population
  of eager repairers responds to "unknown key" by **deleting the future to
  make the file pass**. `"montagent": 4` read by a binary that speaks 2
  produces "this file claims a newer contract than I speak; upgrade montagent,
  do not edit the file" — the only error message in this space that points the
  repair at the *binary* instead of the *file*.
- **It must be checkable, or it is ceremony.** Validate (current binary)
  enforces two directions: declared ≤ current revision (a claim about a future
  contract is an authoring error), and declared ≥ the minimal revision the
  file's own vocabulary requires (each vocabulary item carries its introducing
  revision in the published schema, so the minimum is computable from
  content). Fact 6's gravity went stale because *nothing checked it* pre-0015;
  once checked, staleness is caught at authoring time. A checked field is not
  a ceremony field.

**Strongest attack (P3's, plus the population attack).** (a) Pre-release there
are no stale binaries, and Montagent may be consumed mostly via MCP where the
server is current — the number's sole consumer may be a near-empty population;
this is unmeasured and I label it speculation. (b) P3's structural point is
correct as far as it goes: C1's class is a fact about which binary you run,
and no field can encode "my meaning depends on code outside me." (c) The
min-required check needs revision tags on schema vocabulary — machinery bought
before any failure has occurred.

**Survival.** (a) is why the number is frozen at 1 until release (see Q7) —
until then it costs one inert, correct, copyable token. (b) concedes P3's
point and routes around it: the number is not trying to encode meaning-shift
for the *current* binary (validate's strict-equality checks increasingly catch
that class — C1's own example is now a checkable error); it exists for the
*old* binary, the one case where P3's "validate catches everything" premise
fails, because the old validate doesn't know what it doesn't know. (c) the tag
is one integer per vocabulary introduction, maintained by the same ADR process
that already writes replacement-naming error messages.

**Confidently wrong vs absent:** wrong-low is caught by the min-required
check; wrong-high is caught at authoring time by declared ≤ current; the
residue (file authored without ever running validate) degrades to today's
behavior, no worse. An *unchecked* wrong number would indeed be worse than
none — which is the argument against P2's frozen-at-1-forever marker
masquerading as a version, and against any design that adds the field without
the checks.

**On P4:** I concur with round 1's rejection. Per-object versions solve skew
between objects migrated at different times; under file-as-truth the file
migrates atomically in one git commit, so the skew being managed cannot occur.

**Falsifier:** a decided distribution model that guarantees binary currency
(MCP-hosted only, hard auto-update). Then the stale-binary population is
empty by construction, the number's only consumer vanishes, and P3 is right.

## Q6 — lower bound or equality?

**Preferred answer: P5, lower bound** — the revision authored against; a file
declaring 1 in a world at 4 is not an error. M1 forces it: under P6, three
consecutive revisions would each have obliged rewriting every correct file in
the world as pure ceremony, and (fact 6) ceremony writes are exactly what
agents then copy stale. M1 was measured after positions formed; it settles
this question for me — it is a *measured* instance of the equality policy's
cost falling on files that did nothing wrong.

**Strongest attack (P6's).** A lower bound means the number cannot tell you
what the file conforms to *now*, so "what revision is this file?" has no
answer from the field — isn't it then useless? **Survival:** yes, and that is
the design: conformance-now is validate's job against the current schema; the
field answers only the one question validate cannot — asked *by an older
binary* about a newer file. The one-sided claim is exactly the one-sided
question. **Falsifier:** a demonstrated consumer that needs equality
semantics; none was named in either round.

## Q7 — what trips the policy?

**Preferred answer: P9, with P8 supplying the floor.** P8's core insight is
right and, as far as I can tell, unrebutted: a version names a legality
predicate, and through ADR-0013 ("whether `fit` may be omitted" left
undefined) there was no fact of the matter about legality — you cannot index a
predicate that does not exist. So all fifteen ADRs collapse into revision 1,
and the *mechanics* (the field exists, value 1; the schema carries
introduced-at tags from its first publication as an ADR-0011 resource) bind
when the schema resource is published. The *promise* — bumps, retired-key
table rows, the stranger-facing compatibility story — binds at first tagged
release, decided by the maintainer as a recorded event, because a promise to
strangers starts when strangers can plausibly arrive.

**Strongest attack.** The split invites drift: pre-release ADRs keep breaking
freely under a field that says 1, training agents (and the author) that the
number never moves — the first real bump becomes a novel event nobody
exercises until it matters. **Survival, partially:** this is a real cost; the
mitigation is that the *checking* machinery (min-required, ≤ current) is live
from schema publication, so the field is exercised even while its value is
constant. I cannot fully dismiss the attack and say so. **Falsifier:** if
ADR-0016 turns out to need a bump for a real external consumer before any
release, P8/P9's collapse was wrong and P7's trigger was too late.

## Q8 — does `montagent migrate` ever exist?

**Preferred answer: P10 — never**, and my Part A results, not my prior, decide
it: A2 produced a byte-identical repair from error messages plus ADRs, and A3
showed the hard case is beyond *every* mechanism equally, so it cannot count
for migrate. The deliverable hiding inside this question is the
**retired-vocabulary table in validate**: every retirement adds a row (retired
key/type → retiring ADR → successor recipe, or explicit "no successor" with
the enumeration of legal options). Errors are stated against the *current*
contract, which dissolves P11's dispatch-and-ordering argument: a stranger
several revisions stale never needs to find and sequence N scripts, because
the current validate names the current replacement directly — there is no
N→N+1 chain to order. M3's "refuse loudly and name the affected ids" lands in
validate's message, where A3 shows it fits.

**Strongest attack (P11's best version).** Bulk mechanics: a stranger with 400
files and 8 deletions each wants one idempotent command, not 3,200 exact-string
replaces; and a *composition* of revisions (rename at N+1, value-split at N+3)
might defeat a current-contract message. Also C2 removed the "two successful
migrations" precedent — but note it cuts P11 harder than P10: the project has
*never actually performed* an N→N+1 script migration, so the tool P11 proposes
has no ancestor in practice at all.

**Survival.** Bulk deletions and renames are, by CONTEXT.md's own line
(`shift` is "the one edit that is arithmetic rather than authorship"),
arithmetic — and P10 itself permits them as scripts committed beside the ADR
(now understood, per C2, as a practice to *start*, not one to continue). The
composition case: the retired-vocabulary table is cumulative, so the rename
row at N+1 and the split row at N+3 are both in the current table; the message
for the old spelling can name the whole path. I could not construct a
composition the table cannot express, but I did not search exhaustively —
labeled as an open edge. **Falsifier:** a real future ADR whose repair is
simultaneously (i) mechanical, (ii) not expressible as a replacement-naming
message, and (iii) needed by files outside the repo. Three revisions of
history have produced zero candidates; C1's class comes closest and fails
(iii)→(ii): strict-equality validate already names the rule value, and the
element's own `fit` claim ("cover") makes writing 1920 the *author's own*
arithmetic, not fabricated intent.

## Q9 — one field or two mechanisms?

**Preferred answer: P12 — one field**, covering the whole contract (shape and
normative reading), because by elimination a shape-only version is redundant
with the schema, and because the field's consumer (the stale binary) needs one
answer to one question — "is this file from my future?" — which does not
decompose along the shape/semantics line. P13's mechanical-checkability worry
is answered without a second field: shape-conformance stays checkable because
the *schema* checks it, not the number; semantics-only revisions bump the same
integer and are recorded in the ADR/changelog trail, which exists regardless
and needs no second in-file field to be machine-readable.

**C1's sharpening, addressed explicitly: it strengthens P12.** The
semantics-only class is not byte-invisible in general — and the project's own
trajectory shows the class being *absorbed into checkable shape*: ADR-0013's
rounding rule became ADR-0015's strict-equality error, at which point the
float-written 1919 file stopped being a semantics ghost and became an ordinary
validate error naming its own fix. A format whose fields are derivation claims
(`fit`) keeps converting "meaning changed" into "check fails", which is
exactly the regime where one contract-revision integer suffices and a
dedicated semantics channel has shrinking work to do.

**Strongest attack (P13's core).** A file cannot self-describe an
interpretation rule its author never knew existed — true, structurally. If a
future semantics revision is *not* absorbable into a check (no later
tightening makes it visible), the single integer records that the contract
moved but old files carry no trace, and only the changelog knows. **Survival:
yes, because the alternative fails identically.** P13's second mechanism is
the ADR trail — which my design also has; the disagreement reduces to whether
the *in-file* field must additionally distinguish shape from reading, and no
consumer needing that distinction was produced in either round. **Falsifier:**
a concrete future revision of C1's class that validate structurally cannot
convert into a check, plus a consumer that must distinguish it from a shape
change in-file.

## Q10a — should #61 block #14?

**No — argued, not just dissented from the 4/5.** The four-juror position is
right about the *classification*: if `fmt` does not materialise defaults, a
default change silently changes the meaning of every file that omitted the
field — C1's class cheaper. But #14 does not need to classify default changes
to be written; it needs a *rule*, and the rule can be stated invariantly:
**any revision that changes the picture some legal file renders to bumps N.**
Whether a given default change trips that rule is then decided by #61 — the
rule doesn't move, only which side of it default-changes land on. Blocking #14
on #61 confuses "the policy must decide every future case now" with "the
policy must be decidable on every future case", and only the second is owed.
**Strongest attack:** if #61 lands on non-materialising, default changes
become routine silent bumps and the pressure to *never change defaults* — or
to materialise after all — will be discovered late; deciding #61 first would
surface that cost before the versioning ADR entrenches the rule. That is a
real sequencing benefit, and I weight it below the cost of blocking: #14's
rule is invariant either way, and the ADR should carry one sentence noting
that #61 determines the default-change classification. **Survives, narrowly.**

## Q10b — the unknown-key policy

**A new ticket — not a clause in #14's ADR, not silence.** It is load-bearing
three times over: it decides whether adding an optional field is breaking (the
Q4 row round 1 split on); it decides the failure mode of old-binary-meets-new-
file (silent misrender vs misattributed error), which my Q5 answer builds on;
and it interacts with #73 (published key order) and with `fmt`. Deciding it
inside #14 would create a schema policy by implication — the move ADR-0013
explicitly refused for `contain`, and this project's discipline on that point
is worth keeping. The project's temperament points hard at strict
(`additionalProperties: false`-shaped): "the spelling is content"
(ADR-0013/0014), retired keys are schema errors (0015), and a lenient reader
turns the typo `gravty` into a silently ignored no-op — the exact silent-
divergence class this project keeps legislating against. But that is the new
ticket's argument to have. #14 can proceed: under strict, the version number's
stale-binary story is "error points at the file, number redirects to the
binary"; under lenient, it is "number is the *only* thing standing between the
old binary and a silent misrender" — the number is justified under both
outcomes, so #14 does not block on the new ticket either.

## What I could not settle

- **Whether the stale-binary population will exist.** The entire weight of my
  Q5 answer rests on Montagent shipping as an installable CLI/MCP server that
  goes stale in the wild. That is a distribution-model fact nobody has
  decided. It should be *decided or measured*, not assumed — and my verdict
  flips to P3 if it resolves to guaranteed-current.
- **Where the field sits and how it survives `fmt`** — interacts with open
  #73 (no published key order) and #61; first key by convention is the obvious
  answer but is exactly the kind of thing this project publishes rather than
  assumes.
- **Whether every future semantics-only revision is absorbable into a check**
  (my Q9 survival leans on the observed trajectory, n=1: 0013 absorbed by
  0015). A second instance either way would be strong evidence.
- **The composition edge in Q8**: I could not construct a multi-revision
  composition the cumulative retired-vocabulary table cannot express, but my
  search was not exhaustive and I flag it rather than claim it closed.

## Confidence

- **Q5:** medium. The mechanism is coherent and checkable; the consumer
  population is unmeasured, and I have said what flips it.
- **Q6:** high. M1 is measured, the counter-position named no consumer.
- **Q7:** medium. P8's predicate argument is solid; the split's training-drift
  cost is real and only partially mitigated.
- **Q8:** medium-high. Grounded in my own A2 byte-identical result and A3's
  symmetric-impossibility finding; the composition edge keeps it from high.
- **Q9:** medium. Elimination argument is clean; leans on an n=1 trajectory.
- **Q10a:** medium-high. The invariance argument is crisp; the sequencing
  counter-cost is real but smaller.
- **Q10b:** high on "new ticket"; the non-blocking claim inherits Q5's medium.
