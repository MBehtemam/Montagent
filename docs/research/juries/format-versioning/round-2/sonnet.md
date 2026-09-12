# Verdict, round 2 — sonnet juror, ticket #14

## Headline

The question is not malformed, but it is **two decisions wearing one name**.
A4 shows the identification problem (does a bytes-blob announce itself as a
Montaget project?) has almost no live consumer on this project's actual tool
surface today — every verb in ADR-0011 takes a filesystem path, and the
`.montaget.json` extension already solves it there for free — while the
versioning problem (does a document declare which reading of the format it
trusts) is real and, per A2/A3, splits cleanly into two very different
sub-problems the brief's Q8 already gestures at: changes that are
**arithmetic** (a field deletion with a sufficient remainder, a stale float
recomputed by a published exact rule) and changes that are **authorship** (an
element type retired with no successor spelling, C1's own worked example of
generalizing from a fixture). A2 found the first class needs no tool beyond
`validate` printing a good message. A3 found the second class needs an agent
and cannot be automated by any mechanism, ever, because what's missing is the
author's unrecoverable intent, not a fact. That split, not a yes/no on "does
`montaget migrate` exist," is what I think the ticket should actually decide.

## Part A findings

The single most important result: **A2's diff was byte-identical with zero
guessing**, and **A3's construction was irreparable from any error message in
principle, not just in my attempt.** Together they show Q8 is not one
question. C1 also checked out exactly under independent re-derivation
(`103*(1920/103) = 1919.9999999999998`), and it generalizes beyond the
fixture for a structural reason (the fixture's two geometries are dyadic,
noted in ADR-0013 itself), which is the opposite conclusion from the brief's
original fact 5. Full detail in `WORKLOG-R2.md`.

---

## Q5 — Does the file carry a marker, and is it a version?

**Preferred answer:** P3-and-P2-combined, precisely because A4 makes them
separable and each survives on different grounds. No *version integer* (P3's
case survives fully — see below), but a frozen, content-level marker is
cheap insurance for the one gap A4 could not close: paths without names. I
would ship `"montaget": true` or a `$schema`-style URI-as-sentinel, not
`"montaget": 1`.

**Attack:** This is exactly the "belt and suspenders nobody asked for"
failure the anti-drift rule warns about. A4 also found **zero named
consumers** for content-only identification on this repo's actual tool
surface — I looked at every verb in ADR-0011 and could not construct one
that receives bytes without a path. A marker that solves a problem with no
instantiated consumer is speculative machinery, which the brief explicitly
forbids arguing for on "real formats have it" grounds. Worse: even a
marker with no version number is still a field agents will see in every
example file they read, and fact 6 (agents copy stale fields, 8/8 on
`gravity`) doesn't care whether the field carries a number or a boolean —
copying a marker into a document type it was never meant for is a strictly
smaller failure than copying a wrong integer, but it is not zero.

**Survives?** Barely, and only in the frozen-sentinel form. I cannot justify
a marker that varies (a version number) on identification grounds — a
constant serves identification exactly as well and cannot go stale, which is
the whole of P2's "free rider" argument, now measured rather than asserted:
A4 found the identification case is thin, and a thin case only funds a
field that costs nothing to keep right, i.e., a constant. If the marker must
vary, I no longer think it survives on identification alone.

**Falsifier:** A concrete MCP or CLI workflow, named in ADR-0011 or a future
ADR, that hands a project's bytes across a trust boundary with no
accompanying filename (e.g., an MCP resource served from memory, a paste
into a chat that an agent must classify before writing it to disk). If that
workflow exists today and I missed it, the identification case is not thin
and P2 gets stronger.

Does a confidently wrong version number outweigh no version number? **Yes**,
and A2/A3 sharpen why: A2 showed that for the arithmetic class, `validate`
already has to inspect actual field shapes and recompute actual rules to
find the defect — it does not and structurally cannot trust a self-reported
number to decide what to check, because the number and the truth can
diverge in either direction (M1: three ADRs, same bytes, and a naive
number-bump policy would have manufactured three false diffs; C1: a file
can be silently wrong regardless of what number it declares). A number that
is sometimes trusted is worse than a `validate` that always looks, because
it gives an agent a reason to skip the look.

## Q6 — Lower bound, or equality? (Conditional on Q5 shipping a number)

I do not ship a number (Q5), so this is conditional. **Preferred answer, if
forced: P5 (lower bound).** M1 is decisive and I re-verified it myself in A1:
three consecutive accepted ADRs left the one real file byte-identical.
Equality (P6) would have obliged three ceremonial rewrites of a *correct*
file for no defect, and per fact 6 ceremony fields are exactly what gets
copied stale — a lower-bound number is at least honestly inert until
`validate` says otherwise, whereas an equality number invites the false
confidence C1 demonstrates: a file honestly declaring itself current can
still be wrong (the stale-float case), so "equals current" was never a
sufficient condition for correctness in the first place, and asserting it is
worse than not asserting anything.

**Attack:** A lower bound with no upper bound is nearly informationless. If
`validate` has to check the actual content regardless (as A2 shows it does),
what does the number buy? Answer: dispatch — it tells a migration mechanism
*where to start* without re-deriving it from content (relevant to Q8/P11),
which a pure equality-or-nothing scheme also provides, just less honestly.

**Survives:** as a weak, dispatch-only signal, not as a correctness claim.

**Falsifier:** if #61 resolves such that `fmt` never materializes defaults
and a wide class of default-only changes become C1's-class silent
divergences, a lower-bound number stops being sufficient dispatch
information (`validate` would need the number *and* a full content scan
regardless), which weakens the number's value further, not stronger — it
would argue for content-scanning tools over version dispatch, not for
equality semantics.

M1 changes my answer versus what I'd have guessed cold (equality "feels"
more honest) — the measurement is what moved me, exactly as the brief
intends.

## Q7 — What event trips the policy?

**Preferred answer: P8, with a correction to its own reasoning.** P8's
"you cannot index a predicate that does not exist" is right about the
mechanism but I want to name the sharper reason: A2 and A3 both required me
to treat "the current ADR set" as *the* legality predicate to check against,
and that predicate has been prose-only and moving under all fifteen ADRs.
Nothing in the repo lets two parties agree on what "revision 7" means as a
checkable fact until there is a schema artifact a machine can diff. ADR-0011
already schedules the schema as an MCP resource — that publication event is
the first moment "revision N" denotes something other than "read these ADRs
in order and hope you agree with the last juror."

**Attack, hardest form:** This lets fifteen real, already-shipped breaking
changes escape numbering retroactively, including two (ADR-0014, ADR-0015)
that already broke the one real file that exists. That looks like moving the
goalposts to avoid ever having paid the cost. And P7/P9 have a real
practical advantage P8 doesn't: "first tagged release" is an event a human
can point to on a calendar without any schema work being done first, so it
does not block on an artifact nobody has scoped or costed.

**Survives:** yes, on the strength of A2/A3 specifically. Both of my Part A
exercises had me acting as `validate` "under the current ADR set," which
only works because a human juror can hold fifteen ADRs of prose in
context and reconcile them. A version number is supposed to let a *tool*
do the equivalent job. A tool cannot check membership in an unwritten
predicate; it can check membership in a JSON Schema. Numbering before the
schema exists numbers a thing with no test for what it names.

**Falsifier:** if `validate` already implements the fifteen ADRs today as
executable checks (I did not find such an implementation in the repo — only
`verify.py`, a fixture-specific regression script, and `fit_vocabulary_scan.py`,
a research artifact) — i.e., if there is already a machine-checkable
predicate for "conforms to revision N" independent of the schema resource —
then P8's premise is false and P7 or P9 should win instead. I looked for
this and did not find it; I flag it as the thing worth checking before
trusting my answer.

## Q8 — Does `montaget migrate` ever exist?

**This is where A2 and A3 change my answer most, and I want to be explicit
that they point in different directions on the same question, which is
itself the finding.**

**Preferred answer: neither P10 nor P11 as stated — the arithmetic/authorship
line drawn in A2 vs. A3 is the actual boundary, and it does not track
"before/after release" (P11's frame) or "always/never" (P10's frame).**

- A2's `gravity` deletion: zero guessing, byte-identical repair, fully
  mechanical. `validate`'s own message *is* the migration.
- A3's `ellipse` retirement: irreparable from any message, in principle —
  the missing information is unrecoverable authorial intent, not a fact a
  smarter tool or a richer error message could supply.
- C1's stale-float rewrite: zero guessing (the published rule computes the
  one legal replacement, deterministically, from data already in the
  document plus the source file on disk) — arithmetic, despite changing
  bytes and despite crossing a semantics-only revision.

So the honest answer is: **a script can exist for the arithmetic subclass,
should not exist (and cannot honestly exist) for the authorship subclass,
and the dividing line is not visible from the ADR number** — ADR-0015
contains one deletion (arithmetic) and one strict-equality promotion
(arithmetic, per C1) in the same accepted change; a hypothetical ADR-0016
element-type retirement would be pure authorship. `montaget migrate` "exists"
in the narrow sense of a `validate`-driven, per-change, arithmetic-only
repair helper — closer to P11's "chain of per-change scripts" than to a
general tool, but the thing it must never attempt is the authorship
subclass, which P10's core argument (Montaget is agent-first; the repairing
entity is always an LLM; a tool that writes authored content fabricates
intent) is exactly right about *for that subclass only*.

**Attack:** This "it depends on the change" answer is unsatisfying as a
ticket resolution — it tells #14 to decide case by case forever, which is
not a decision, it's a deferral dressed as nuance. And C2's correction
matters here: the "two successful migrations" precedent both P10 and P11
lean on is not what it looked like — `migrate.py` is a *cumulative
regenerator from a fixed #9 origin*, not a chain of N→N+1 scripts, and it
works only because the author owns `old.json`. Neither position has a
real precedent for the thing they're each claiming precedent for.

**Survives:** yes, as a classification rule rather than a single verdict,
because A2 and A3 are both real, both reproducible, and they disagree with
each other in a way that a single "yes" or "no" cannot honestly resolve.
What I'd tell the ticket to adopt: **`validate`'s error message itself
must state, structurally, which class a given failure is** — either it
names a fully-determined replacement (arithmetic; safe to script, safe to
automate, per A2/C1) or it names only the defect with no replacement
(authorship; `validate` refuses and stops, per A3, and no tool should try).
That is a testable property of every future ADR's error message, not a
policy about `migrate` as a binary.

**Falsifier:** a real future ADR whose repair is genuinely intermediate —
partially determined, partially requiring a choice — would break this clean
split. I could not construct one from this repo's fifteen ADRs (every
breaking change I found decomposes into the two classes), but I would not
be surprised if one exists; I did not exhaustively classify all fifteen.

## Q9 — One field or two mechanisms?

**Preferred answer: P13, and A2/A3 sharpen rather than undercut it, contra
the brief's suggestion that C1 might weaken it.** The brief asks: since C1
shows the semantics-only class is not byte-invisible in general (only at the
fixed point), does that change the two-mechanisms case? I think it
strengthens P13, not weakens it: if a semantics-only revision (ADR-0013)
*can* in fact change bytes on files off the fixed point, then a
shape-only schema-diff genuinely cannot distinguish "this file needs
rewriting because its shape changed" from "this file needs rewriting because
the same shape now means a different number" — both show up as the same
kind of validate error (a value that fails a check), but they need different
tool behavior downstream: the first is a `fmt`-shaped rewrite of structure,
the second requires re-deriving numbers from source media (A2's
recomputation, C1's `1919`→`1920`). A single integer conflates "did the
shape change" with "did the meaning of unchanged shape change," and A2/A3
both needed me to know *which* kind of check I was running before I could
decide whether a mechanical repair was even possible — that is P13's
"non-substitutable" claim, demonstrated rather than argued.

**Attack:** Two mechanisms is real cost for a project with one fixture and
zero users (fact 8): a schema-diff tool to build, an ADR/changelog
convention to maintain, and P12's elimination argument is clean where P13's
is not — "the version's exclusive job is the thing a schema can't check" is
a sharper design principle than "two things because two failure modes,"
which risks turning into "one field per concern" sprawl the project's other
ADRs (e.g., ADR-0012 on `box`/`clip`'s two-meanings defect) explicitly reject.

**Survives:** yes, but weakly — I think P13 is right about the *mechanism*
question (schema-diff and prose-changelog are genuinely different checks)
but I am not confident it needs to be **two schema fields**. It could be one
field (a revision number, Q6/Q7-permitting) plus the ADR trail as the
existing, already-adopted second mechanism (fact: the project already writes
an ADR per change; that *is* the changelog P13 wants, and it costs nothing
additional to keep being that). So P13's mechanisms both already exist
independent of what ships in the JSON; the open question is only whether the
JSON needs a *third* thing (a field) on top of them, and my A2/A3 results
don't settle that — they only establish that content and meaning are
genuinely different axes, which the ADR trail already tracks.

**Falsifier:** if the ADR trail is judged insufficient as a machine-readable
semantics changelog (e.g., a future tool needs to programmatically ask "did
revision N change what my file's bytes mean" without an LLM reading prose),
that would push toward an actual second field, strengthening P13's strong
form.

## Q10a — Does #61 block #14?

**Preferred answer: yes, partially and specifically — not as a general
dependency, but because Q8's classification rule needs it.** My A2/A3
finding depends on being able to tell arithmetic changes from authorship
changes by whether the replacement is fully determined by document content.
A default-field change is exactly the swing case: if `fmt` materializes
defaults, a stale file after a default change has the *old* value written
literally, so `validate` sees a concrete (now-wrong) number and the
situation degrades gracefully to something diffable — arithmetic. If `fmt`
does not materialize defaults, a stale file's *absence* of the field means
"whatever the default was when written," which is unrecoverable after the
default changes — the exact structure of A3's authorship problem (a fact
about which binary/revision produced the file, not present in the document,
P3's own argument about C1's class recurring here). So #61 doesn't just
affect severity classification in the abstract (as four of five round-1
jurors said) — it determines, concretely, which of Q8's two buckets every
future default-change ADR falls into.

**Attack:** "Blocks" is too strong a word for a ticket that (per Q7) may not
even bind until the schema resource ships. #61 could resolve *after* #14 as
long as #14's policy is written to name the classification test ("is the
replacement fully determined by the document plus the schema, yes/no") 
rather than hand-classifying default changes today. That defers #61's answer
without blocking on it.

**Survives:** as "sequencing, not blocking" — #14 should state the test
(Q8's classification rule) in a way that is agnostic to #61's answer, and
#61 then slots a concrete future default-change ADR into the bucket the test
already defines. #14 does not need #61 resolved to ship; it needs to not
contradict whichever way #61 goes.

**Falsifier:** if #61 is trivial to resolve immediately (a quick ADR), just
resolving it first removes the whole question and is simpler than writing a
classification test designed to survive either answer.

## Q10b — Where does the unknown-key policy belong?

**Preferred answer: a clause inside this ticket's ADR, not a new ticket, and
not neither.** C4 is right that no policy exists today, and it is load-
bearing exactly where the brief says: it decides whether "add an optional
field" is breaking. But A2 supplies a concrete reason it belongs *here*
rather than in its own ticket: my A2 exercise only worked — I only knew that
"unknown field `gravity`" was legitimately an error rather than a forward-
compat no-op — because ADR-0015 explicitly named `gravity` a schema error
by name, not because a general "unknown keys are errors" policy exists. A
version/migration ADR that doesn't also settle the unknown-key default
cannot tell an agent, in the general case, whether a field it doesn't
recognize is stale-and-safe-to-ignore or stale-and-load-bearing. Versioning
and unknown-key policy are answering the same underlying question — "what
do I do with a document written against a different picture of the format
than the one I hold" — from two directions, and splitting them into two
tickets risks exactly the kind of two-meanings-for-one-concern defect
ADR-0012 killed for `box`.

**Attack:** This conflates two genuinely separable concerns and risks
bloating #14 past the point of reviewability — C4 is explicitly about
*reading forward* (an old reader meeting a new file with an unrecognized
optional field) while #14 as scoped is about *reading backward* (a new
reader meeting an old file). ADR-0015's `gravity` case is reading-backward
and settled it field-by-field; it is not evidence for how reading-forward
should default.

**Survives:** the attack is right that they're formally separable, but I
still think #14 must at least *state* which of the two directions it is
deciding and explicitly punt the other by name, because a version/marker
proposal that is silent on this will be read by an implementer as having
implicitly picked "strict, additionalProperties: false" or "permissive,"
and C4 shows neither is currently true — leaving it silent is worse than
naming the gap and deferring it, even if the deferral is to a new ticket.

**Falsifier:** if #14's scope, once drafted, turns out to only need the
reading-backward direction (which A2's actual test case needed), then the
reading-forward half (true unknown-key policy) can cleanly go to its own
ticket without weakening #14's answer.

## What I could not settle

- Whether a concrete workflow exists (now or planned) where an MCP client
  hands Montaget project bytes with no path — the one thing that would make
  Q5's marker case strong rather than weak. I searched ADR-0011's tool
  surface and found none, but "not designed yet" is not "will never exist."
- Whether any of the fifteen ADRs contains a breaking change that is neither
  cleanly arithmetic nor cleanly authorship (Q8's falsifier) — I did not
  exhaustively re-derive all fifteen against Q8's test, only ADR-0014,
  0015, and my constructed ADR-0016.
- Whether `fit_rounding_scan.py`'s 31,402,800-combination, 4.466% figure
  independently reproduces — I confirmed the mechanism and the single
  worked example by hand but did not re-run the script.
- The actual cost of building a schema-diff tool (P13's mechanism #1) —
  nobody has scoped it, and "the schema doesn't exist yet" (fact 2, still
  true) means this is closer to a measurement gap than a disagreement.

## Confidence

- Q5: medium. The frozen-marker-for-identification case is real but thin
  (A4); I am confident no varying version number survives on identification
  grounds alone.
- Q6: low-medium, and explicitly conditional on Q5 going the other way from
  my preference. If forced to a number, lower-bound is well-supported by M1.
- Q7: medium-high. P8's mechanism (no schema, no predicate to index) is the
  strongest argument in the whole brief, and A2/A3 gave me independent,
  first-hand confirmation that "the current ADR set" is presently a
  human-reconciled prose object, not a machine-checkable one.
- Q8: medium-high on the classification rule, low on any single-word
  answer. I am confident A2 and A3 are both real and both correctly
  classified; I am much less confident that "always classify by
  determinacy of the replacement" scales cleanly to ADRs I haven't tested.
- Q9: medium. Confident the two mechanisms are conceptually distinct;
  unconfident whether that requires a second *field* versus relying on the
  ADR trail that already exists.
- Q10a: medium-high on "sequence, don't block."
- Q10b: medium on "clause inside this ADR, scoped to reading-backward only,
  reading-forward explicitly punted by name."
