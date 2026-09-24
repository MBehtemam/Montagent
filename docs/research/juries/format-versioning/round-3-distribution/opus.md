# VERDICT-R3 — the distribution fact, and what it does to the stale-binary argument

## Headline

**The distribution fact is decided, and it decides *for* the argument's premise.**
The stale-binary population is non-empty, and not as an open guess: an
MCP-hosted-only, always-current Montagent is *excluded* by four accepted ADRs
acting together, not merely unchosen. So the argument's own stated falsifier does
not fire, and the argument does not collapse where it said it would.

**It collapses one step further in, on its own mechanism.** The monotonicity
objection the brief raises is survivable — an old binary needs only its own
compiled-in revision and a promise of monotonicity, and `declared > R_self` is the
one predicate that does *not* degrade as a binary ages. What is not survivable is
the second objection. The number's signal is **agent-maintained**, and fact 6
(8/8 copying a retired field out of an example) says the dominant real population
is files authored at N+1 that *declare N* because the header got copied. On those
files the guard does not fire at all: `N <= R_self`, the binary proceeds, the
schema says "unknown key `fit`", and the future gets deleted exactly as before.
**The guard is a no-op on its own worst case, and fires only on files the agent
already got right.** Worse, a stale binary is the one reader that cannot
cross-check a version claim, so the number is least reliable precisely where its
only consumer lives — and a believed-wrong number is an *authoritative warrant*
to misread, which is strictly worse than no warrant.

And the instruction the argument is bought for is already available for free. An
old binary meeting `fit` holds the only fact that matters — *"I do not recognise
this key"* — which is a fact about the binary, computed by the binary, and never
stale. Pointing repair at the binary is a property of the **error message text**,
not of a field in the document. The number is a proxy for a fact the reader
already has, routed through the least reliable channel available.

**Implication: the stale-binary argument does not carry a version number.** What
it does carry, and what this investigation says is the real deliverable, is C4 —
settle the unknown-key policy, make the schema closed, and legislate the message
to name the *binary* as the suspect. One clause, no document field, and it works
on every stale-binary encounter including the copied-stale-number case.

One honest hedge, stated up front because it is the soft spot: my attack rests on
generalising fact 6 from a *semantic* field (`gravity`) to a *ceremony* field (a
version integer). That generalisation is **unmeasured**. It is also measurable in
an afternoon on this project's existing method. See *Falsifiers*.

---

## 1. What the accepted ADRs already commit to about distribution

No ADR is titled "distribution", and none says "users download a binary" in those
words. But six load-bearing commitments each independently require a locally
installed executable on a user's own machine, and they are quoted here rather
than summarised because the conclusion rests on their conjunction.

**(a) ADR-0009 — the transport is stdio, and that reasoning is load-bearing, not
incidental.**

> "On the stdio binding the client launches one subprocess per session, so
> startup is paid once: never per tool call, never per preview."

This is the sentence that kills the 88× Rust speed win ("an 88× win for Rust that
**cannot be spent**"). The host decision's central argument *depends* on stdio.
You cannot re-host it over HTTP without reopening ADR-0009's "why".

**(b) ADR-0009 — the binary's dependency is supplied by the user.**

> "This costs the distribution claim above, and the cost is stated rather than
> softened. #7 argued for Rust partly on a single static binary with no runtime.
> That is now **'a binary, plus an `ffmpeg` the user supplies.'**"

Decisive on its own. A hosted service supplies its own ffmpeg; only an installed
artifact can have a *user-supplied* dependency. There is a user, they have a
machine, and it has a PATH.

**(c) ADR-0009 — the whole comparison is a comparison of install stories.**

> "**Distribution is the one axis Rust wins outright and can keep.** A compiled
> binary with no interpreter and no `node_modules`. Node's single-executable
> route is `Stability: 1.1` and untested on macOS x64; a Python package needs an
> interpreter that `uvx` supplies and `pipx` deliberately will not."

Every alternative weighed is a *user-side runtime*. Under hosted-only
distribution this entire paragraph is irrelevant — the operator installs whatever
they like once — and it is the axis that decided the ticket.

**(d) ADR-0011 — three of eleven commands are reachable only as a local
executable.** `probe`, `fmt` and `timeline` are CLI-only, and the stated reason
is that they

> "remain one `Bash` call away, since the agent already carries a shell."

That is `montagent probe …` on the agent's PATH. An MCP-hosted-only Montagent
*cannot* deliver `fmt`, `probe` or `timeline` at all. ADR-0011 also fixes the
artifact shape: "**One binary, one core library.** The MCP server wraps the
library, never the CLI — a subprocess per tool call would pay the startup
ADR-0009 says is paid once on the stdio binding."

**(e) ADR-0010 — the build story is a six-desktop-target shipping story.**

> "the required key **`jpegd-jpege-pdf`** … is published for 0.153.2 on **all six
> desktop tier-1 targets**: `{aarch64, x86_64}` × `{apple-darwin,
> unknown-linux-gnu, pc-windows-msvc}`. … **The shipped binary is 11.4 MB (9.6 MB
> stripped).**"

Nobody verifies Windows and macOS, x64 and aarch64 prebuilts to run a service.
Six desktop targets and a stripped-size figure are the signature of an artifact
handed to strangers. "The shipped binary" is the ADR's own phrase.

**(f) The data model is local-filesystem throughout.** ADR-0011: `validate` asks
"does the document agree with itself **and with the disk**". ADR-0002: an
element's `source` is "a relative path today". ADR-0007: fonts are files the
project declares by path — and the fixture's `SF Pro Rounded` "resolves on its
author's machine only because it was hand-installed". A remote server cannot see
the user's media, cannot see their fonts, and cannot resolve a relative path in
their git repo. Note ADR-0002 explicitly contrasts Montagent with the four
surveyed products *because* "All four are HTTP services".

Add the project-level frame from the map: Montagent "will be **open source and
used by people other than its author**".

## 2. The map's "Packaging and distribution" fog entry, and what GPL implies

Quoted in full from issue #2, section *Not yet specified*:

> **Packaging and distribution** — how Montagent is installed and run. **Narrowed
> hard by ADR-0009, and not in the direction #7 expected.** The host is Rust, so
> #15's three shapes collapse to one — a compiled binary, no interpreter, no
> `node_modules`. But the *self-contained* half of that claim is gone: `libx264`
> is GPL and FFmpeg's legal page states it covers all of FFmpeg, so Montagent
> spawns an `ffmpeg` **the user supplies**, and bundling one instead makes this
> project a distributor of GPL software owing source-offer duties under GPL
> §3/§6. **There is no option that is both self-contained and obligation-free** —
> what is left here is choosing between those two, plus the container question
> and the asset-path story.

Read this precisely. The fog entry's own framing is *"how Montagent is **installed**
and run"*, and the residual choice it names is **which of two installed shapes**:
bundle ffmpeg (and owe §3/§6) or require the user to supply one. Neither branch
is a hosted service. The entry has already spent the question the argument's
falsifier needs to be open.

**What GPL §3/§6 implies, worked out.** The source-offer duty of GPL §3/§6
attaches to **conveying** — handing a copy of the covered work to someone else.
Plain GPL-2.0/3.0 has no network-use clause (that is the AGPL, and no document
here mentions it). Therefore:

- If Montagent were hosted and never downloaded, bundling ffmpeg server-side would
  trigger **no** §3/§6 duty whatsoever. The ASP gap covers it completely.
- ADR-0009 and the map both treat the §3/§6 duty as a **live, unavoidable cost**
  that constrains the design ("no option that is both self-contained and
  obligation-free").

Those two are only consistent if Montagent is **conveyed to users as a
downloadable artifact**. The licensing analysis presupposes distribution to
users; under hosted-only it would be moot, and the paragraph would not have been
written. *(Inference, not a quotation — but a tight one: the ADR spends real
design freedom to avoid a duty that only exists on conveyance.)*

A second, sharper implication that nobody seems to have drawn: **the project has
already accepted an uncontrolled-version dependency living on the user's
machine.** "An `ffmpeg` the user supplies" is whatever Homebrew, apt or a 2021
static build gave them. A project that has designed around a stale *ffmpeg* on the
user's box cannot coherently claim its own binary is always current. The
guarantee the falsifier requires is one the design has already declined to make
about a component in the same process tree.

## 3. Is the stale-binary population non-empty? Decided, or open?

**Decided, and non-empty.** Sharply, per the brief's instruction:

**The ADRs decide this.** To get an MCP-hosted-only, guaranteed-current Montagent
you must overturn: ADR-0009's stdio reasoning (which is the reason its own
headline measurement is discardable), ADR-0011's CLI-only verbs and its
"one binary, one core library", ADR-0002's relative source paths, and ADR-0007's
declared font files. Four accepted ADRs, three of them on their load-bearing
clauses. That is not an open question; it is a settled one that nobody bothered
to write down under its own heading.

**What is genuinely open** — and must not be confused with the above: the
*install channel* (Homebrew / cargo / curl-sh / release archive), the **container
question** (named in the fog entry), the asset-path story, and whether there is an
auto-updater at all. None of these is decided. All of them are downstream of
"there is a binary on a user's machine".

**Does hard auto-update rescue the falsifier?** No. Even granting an auto-updater
that does not yet exist, the population stays non-empty for three reasons, only
the first of which is settled fact rather than forecast:

1. **File-as-truth puts the file in git, and git moves files between machines.**
   That is the stated point of it ("git supplies undo, diff and branching for
   free"). A file authored on machine A at revision N+1 reaches machine B by
   clone or pull. Cloning a repo does not update B's binary. The skew window is
   not a bug in the updater; it is the feature. *(Settled: follows from
   file-as-truth plus the map's "used by people other than its author".)*
2. **Open source + six desktop targets means third-party packaging.** Homebrew,
   nixpkgs, apt and friends are routinely weeks-to-months stale, and a formula's
   binary cannot self-update from inside itself — Homebrew actively opposes it.
   *(Speculation: the channel is unchosen. But choosing "no distro packaging
   ever" for an open-source CLI is not a choice a project gets to make.)*
3. **Pinned environments freeze the binary deliberately** — CI images, Docker (the
   fog entry's own "container question"), nix flakes. And `git checkout` of an old
   branch produces the *reverse* skew: an old file under a new binary. *(Partly
   speculation; the container question is open by the map's own admission.)*

Conclusion: the argument's stated falsifier **does not fire**. It survives this
round on distribution.

## 4. Attacking the argument that survived

### 4a. The monotonicity / self-knowledge objection — the argument survives this

The brief asks whether an old binary that does not know the *current* revision can
distinguish "newer than me" from "corrupt". **Yes, and this objection fails.** The
old binary needs exactly two things, both of which it has:

- `R_self`, the highest format revision it implements — a **compile-time
  constant**, not knowledge about the world; and
- a **promise of monotonicity** on the declared number.

Then `declared > R_self` ⇒ newer than me. No knowledge of the current revision is
required. This is why the number must be a bare monotone integer (`"montagent": 4`)
rather than a date, a hash, or a semver with independent components: the predicate
must remain computable by a binary with no network and no clock.

This is the mechanism's one genuine strength, and it deserves to be stated at full
force before I attack the rest: **the comparison does not degrade with age.** A
binary ten revisions stale still computes `47 > 37` correctly. Every
shape-sniffing alternative degrades monotonically — a binary can only recognise
shapes that existed when it was compiled, so an unknown key is indistinguishable
from a typo, which is the exact confusion that makes agents delete the future.

Two costs this imposes, which any ADR adopting it must state explicitly:

- The check must run **before** schema validation. If the schema fires "unknown
  key `fit`" first, the number never gets read and the whole purchase is void.
- It forces the lower-bound reading (P5/M1): error **iff** `declared > R_self`,
  never on `declared < R_self`. Equality-semantics would make M1's three
  byte-identical revisions each oblige a global rewrite of every correct file as
  pure ceremony — and ceremony fields are exactly what agents copy stale, which
  is the next attack.

### 4b. The stale-number objection — this is where it dies

Fact 6: 8/8 agents reached for `gravity`, several copying it out of the fixture
before reading the spec. A version integer is the single most copy-prone thing in
a file: it is a header, it is semantically inert to the task at hand, and every
example the agent has ever seen carries one.

So the realistic population of files in the world is **not** {correct} ∪ {newer}.
It is dominated by files authored at N+1 that **declare N**, because the header
came along with the copied example, the sibling project, or the fixture.

Trace one through the stale binary at `R_self = N`:

1. Binary reads `"montagent": N`. `N > N` is false. **The guard does not fire.**
2. Binary validates the shape. Finds `fit`.
3. Emits "unknown key `fit`".
4. Agent deletes `fit` to make the file pass. The future is deleted.

Identical to the no-number world, except the project now also carries a field, a
policy, a check and an ordering constraint. And note the shape of the coverage:
**the guard fires only when the number is correct and current — that is, only on
the files where the agent was already careful.** A safety device whose coverage is
conditional on the user having already been careful buys close to nothing, because
careless authorship and stale headers are the same event.

This is the load-bearing claim of the stale-binary argument inverted, not dented.

### 4c. The second-order harm: a believed-wrong number is worse than none

This one is specific to the stale-binary population and is, I think, the sharpest
thing in this verdict.

A **current** binary that reads `"montagent": 1` on a file full of revision-4
fields can notice the contradiction — it knows revision 4 and can see the file is
not revision 1. A **stale** binary at `R_self = 1` reading `"montagent": 1` sees no
contradiction available to it. It has been handed an authoritative, in-band
statement that this file is revision 1, and it will act on it with full
confidence.

**The version number's reliability is lowest exactly where its only consumer
lives.** In the no-number world the stale binary at least has no false warrant; it
merely does not know. In the number world it has been told something false by the
document itself. For a *lenient* reader — the argument's own first failure mode —
this actively causes the silent misrender the number was bought to prevent, and
gives it a justification.

### 4d. The instruction is deliverable without the number

The argument's real product is one sentence: *"this file claims a contract newer
than I speak; upgrade the binary, do not edit the file."* Where does that sentence
have to come from?

The old binary meeting `fit` already holds the decisive fact: **"`fit` is not a
key I know."** That fact is (i) computed by the binary, (ii) about the binary,
(iii) available on *every* stale-binary encounter including 4b's copied-header
case, and (iv) never itself stale. Pointing the repair at the binary is a property
of the **message text**, not of a field in the document:

> `E-UNKNOWN-KEY`: unknown key `fit` on element `photo-03`. This is not a key
> this Montagent knows, and not one it has retired. It may belong to a **newer
> format revision than this binary implements** — check your Montagent version
> before removing it. Do not delete the key to make the file validate.

C4 established that **nothing in this project states an unknown-key policy**. That
means this message is *unwritten*, not *unavailable*. The version number is a
proxy for a fact the reader already possesses, routed through the least reliable
channel in the system (agent hand-maintenance), and it degrades to silence exactly
when that channel fails.

### 4e. Half the stated harm is already excluded

The argument names two failure modes. The **lenient reader** is largely already
off the table: ADR-0006 commits to `validate` reporting and `render` enforcing and
refusing on error; ADR-0011 makes every write tool return findings so the check is
structural rather than opt-in. Montagent's readers are not lenient about anything
they understand.

The precise residue — and it is exactly C4's gap — is leniency about keys the
binary does **not** understand. Closing C4 closed (unknown keys are a schema
error) eliminates the lenient-reader mode entirely, again without a number. So
settling C4 does double duty: it kills failure mode one, and it creates the site
where failure mode two's message lives.

### 4f. The C1 class runs the other way, and the number cannot be acted on there

Verified independently: `103 * (1920/103)` is `1919.9999999999998`, floors to
`1919`; ADR-0013's own founding measurement is a **4.466%** disagreement rate over
**31,402,800** (source, box) combinations (`docs/adr/0013-…md`, lines 66–67).

The stale-binary framing is *new file, old binary*. The C1 class is the inverse:
**old file, new binary** — a legal pre-0013 file with `width: 1919`, now an error
under ADR-0015's strict equality. A lower-bound number does nothing for the stale
binary here, and for the current binary it could in principle say "authored under
float rounding — tolerate or repair". That is a **bug-compatibility switch**, and
the project should refuse it: honouring it means keeping a float rounding path
alive forever, a second implementation of the thing ADR-0013 legislated away.

So on this class the number is either unread or read-and-deliberately-ignored. **A
number nobody acts on is not a number.** This is a third, independent line to the
same conclusion.

### 4g. Attacking my own preferred answer, hardest

**(i) The message cannot distinguish future from typo; the number can.** `fit`
versus `ftt` look identical to a stale binary. The message must hedge, and a
hedged instruction is weaker than a definite one.

*Response, partial concession.* True, and it costs something. But the asymmetry of
consequences runs my way: the hedged message's failure mode is that an agent keeps
a typo'd key for one extra round trip, which the next `validate` catches; the
number's failure mode is a deleted future, which nothing catches. Hedging toward
"do not delete" is the correct bias. And the binary can narrow further than a bare
hedge — it knows its *retired* names too (ADR-0015 knows `gravity`), so it can say
"not a key I know, and not one I retired", reducing the causes to two.

**(ii) It is one integer. Even partial cover is cover.** *Response.* It is not
free. It lands on the exact-string-replace surface every agent uses (fact 7), it
interacts with #73 (key order) and #61 (does `fmt` materialise it, and does
rewriting it break the replace), it must be maintained forever, and its wrong
values are **silent**. Against 4c, partial cover is not even the right
description: on the population that matters it is negative cover.

**(iii) The number is not only for the stale binary — it dispatches migration
(P11).** *Response.* Out of this question's scope, and weakened by C2: there is no
N→N+1 migration practice for it to dispatch, only a cumulative regenerator from an
origin file a stranger does not have.

**(iv) — the one that nearly lands.** My whole attack rests on generalising fact 6
from `gravity` — a *semantic* field an agent reaches for because it wants the
behaviour — to a version integer, a *ceremony* field an agent might treat as a
distinct category ("this is a version, I should check it"). **That generalisation
is unmeasured, and if it is false, 4b dissolves** and coverage goes from "only when
already correct" to "usually correct", at which point the number is worth its cost.

I do not think it is false — the copy-the-example mechanism does not care what the
field means — but I cannot demonstrate it from anything in this repo. This is the
single place my answer leans on speculation, and it is *the* place, so I will not
bury it: **the question turns on an unmeasured agent-behaviour fact, and this
project can measure it in an afternoon.** Recommend measuring rather than
deciding. See below.

## Verdict, stated plainly

1. **The distribution fact is settled by the accepted ADRs: Montagent is a binary
   a user installs and runs on their own machine, over stdio, against their own
   disk, spawning an ffmpeg they supply.** MCP-hosted-only is excluded, not
   merely unchosen. Still open, and separate: install channel, container, asset
   paths, auto-update.
2. **The stale-binary population is therefore non-empty**, and stays non-empty
   under any auto-update policy, because file-as-truth moves files between
   machines by design.
3. **The argument survives its own falsifier and then fails on its mechanism.**
   The monotone comparison works; the signal feeding it does not. The guard
   no-ops on copied-stale headers (the dominant population), and a believed-wrong
   number is an authoritative warrant to misread, delivered to the one reader
   that cannot cross-check it.
4. **Recommendation: do not add a version number on the strength of the
   stale-binary argument.** Instead close C4: make the schema **closed** to
   unknown keys, and legislate the error message to name the *binary* as the
   suspect and forbid deletion as a repair. That delivers the argument's entire
   product — the only message in this space pointing repair at the binary — on
   every stale-binary encounter, with no document field and no dependence on
   agent care.
5. **The marker question (P2) is untouched by this and remains separable.** If a
   top-level marker is warranted on identification alone, it also sharpens the
   message in (4) — "this is a Montagent project *and* I do not know this key"
   removes "wrong file type" from the causes. That is an argument for a marker,
   and still not one for a number.

## Falsifiers

- **On the distribution fact.** Falsified by any accepted decision record
  committing Montagent to a hosted-only transport, removing the CLI, or replacing
  relative disk paths with URLs the server fetches. I grepped all fifteen ADRs and
  `CONTEXT.md` for install / distribut / packag / hosted / binary / release /
  docker / auto-update; none does. Also falsified if the project both adopts a
  hard auto-updater and forbids distro packaging and forbids pinned environments —
  and even then only partially, because git still carries files across machines.
- **On the anti-number conclusion — the measurement that should decide this.**
  Three arms, same stale binary, same N+1 file: (A) correct number present, "newer
  contract" message; (B) no number, closed-schema unknown-key message naming the
  binary; (C) bare "unknown key `fit`". My claim is that B ≈ A and both ≫ C. I am
  falsified if A repairs the binary and B still deletes the field.
- **On 4b, the load-bearing generalisation.** Give agents an example file carrying
  `"montagent": 1` and a task requiring a revision-4 field. If they update the
  integer at a high rate, my attack fails and the number earns its place. #48's
  8/8 exercise shows this method is already in this project's hands.
- **On 4c.** Falsified if a stale binary given a wrong-but-plausible number
  behaves no worse than one given none — i.e. if the false warrant does not
  actually license a misread in practice.

## What I could not settle

- **Whether a marker (P2) is warranted on identification alone.** Out of scope
  here; my finding only says the two are separable and the number does not ride
  free on the marker's justification.
- **The exact unknown-key policy** (C4) — closed vs. open vs. warn-and-proceed. I
  argue it must be closed for the message to exist at all, but that is a decision
  this question was not asked to take, and it is load-bearing for Q4's
  "add an optional field" row.
- **Whether `fmt` would rewrite a version integer** (#61's territory). If it
  would, the field collides with fact 7's exact-string-replace editing on every
  binary upgrade. Not settled; relevant only if a number is adopted anyway.
- **The stale *ffmpeg* population.** The design already accepts an
  uncontrolled-version dependency on the user's machine, and no project-file field
  can address it. Nobody has written down what Montagent does when the user's
  ffmpeg is too old. Probably a separate ticket.

## Confidence

| Claim | Confidence |
| --- | --- |
| ADRs commit to a user-installed local binary (§1, §2) | **High.** Six independent commitments, three quoted verbatim from load-bearing clauses. |
| Hosted-only is excluded rather than merely unchosen (§3) | **High.** Overturning it costs four accepted ADRs. |
| GPL §3/§6 reasoning presupposes conveyance to users (§2) | **Medium-high.** Inference from the ADR spending design freedom to avoid a duty that only exists on conveyance; not stated in those words. |
| Population stays non-empty under auto-update (§3) | **Medium-high** for reason 1 (follows from file-as-truth); **medium** for 2–3 (unchosen channels). |
| Monotone comparison works without knowing the current revision (§4a) | **High.** Mechanical. |
| Stale-number no-op is the dominant case (§4b) | **Medium.** The mechanism is sound; the rate is the unmeasured generalisation of fact 6, and it is where I am most attackable. |
| A believed-wrong number is worse than none for a stale reader (§4c) | **Medium-high.** Argued, not measured. |
| The message delivers the instruction without the number (§4d) | **Medium-high**, pending the three-arm measurement above. |
| Overall: do not add the number on this argument | **Medium-high**, and it should be **downgraded to a recommendation-to-measure** if the convener wants better than that — the deciding fact is cheap to obtain and currently absent. |
