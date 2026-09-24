# Font vendoring, licensing and the fonts discovery tool — court ballots

Two rounds, three jurors each (Claude Opus 5, Claude Sonnet 5, Claude Haiku
4.5), independent, blind to each other's ballots, no assigned stance. Verdicts
and reasoning are recorded in
[ADR-0057](../../adr/0057-font-vendoring-licence-gate-and-path-keyed-attestation.md).

## Round 1

### Q1 — What does `fonts vendor` copy, and from where?

Options: (a) system fonts only, local, offline, pure filesystem copy; (b) also
support external sources (a URL, or a named font family fetched from a
service like Google Fonts).

🗳️ **Juror** (Claude Opus 5) — **VOTE: a**

**Reasoning:** The decision ADR-0007 actually made is that project files
reference vendored file paths, never a system family name. Everything `fonts
vendor` must do to uphold that invariant is satisfiable by a local filesystem
copy: enumerate installed faces (paths + face indices), copy the chosen file
into the repo, rewrite the reference. Fetching from a URL or a Google Fonts
family name is a *font acquisition* concern, not a *font vendoring* concern,
and the two compose fine without being the same tool — an author who wants a
font they don't have downloads it by any means they like (curl, browser,
`gh`, a package manager) and points the vendor step at the resulting file.
Supporting a plain local path argument alongside the system-font picker is not
"external sources"; it's still a pure filesystem copy, and it already covers
the download case at zero cost.

The cost of (b) is disproportionate and lands in the wrong place. Network
access in a core project-mutation tool drags in: license/provenance questions
(Google Fonts ships OFL, other URLs ship whatever — and vendored binaries get
committed into someone's repo), integrity verification, TLS/proxy/offline-CI
failure modes, a service dependency whose API shape Montagent doesn't control,
and the reproducibility problem that the same command run twice can yield
different bytes. For an agent-first tool this last point matters most: an
agent invoking a networked `fonts vendor` introduces a nondeterministic,
credential-adjacent, remotely-influenced write into the project repo. A tool
that only reads the local font directories and copies bytes is trivially
auditable and trivially testable with fixtures. Note also that ADR-0056 has
already established session-scoped, no-persistent-cache handling for remote
source probing — which is the *right* home for any future network-font story,
and a reason not to grow a second, differently-shaped remote path inside
`fonts vendor`.

**Trade-offs:** (a) costs one extra manual step for an author who wants a font
not yet installed — they must obtain the file before vendoring. That is a
small, well-understood, one-time friction, and it keeps the license decision
in a human's hands rather than implied by a family-name string. It also means
Montagent has no built-in font discovery ("what fonts can I use?" is answered
only by what's installed), which is a real but acceptable gap. The honest
risk in voting (a) is that `fonts vendor <path-or-url>` looks like a one-line
addition and will be requested repeatedly; the answer should be that a URL is
a separate, explicitly-opted-in command (e.g. `fonts fetch`) that lands a
file on disk and stops — never an implicit network call inside the copy step.
If that separate command is later built, (a) loses nothing: `fonts vendor`
still does exactly one thing.

🗳️ **Juror** (Claude Sonnet 5) — **VOTE: a**

**Reasoning:** ADR-0007's own description — "lists system fonts with paths
and face indices and vendors a chosen one into the repo" — describes a
discovery-then-copy pipeline over a local font catalog, not a fetch pipeline.
Keeping `fonts vendor` a pure local-filesystem copy preserves a strong,
easily-auditable invariant: the tool reads a file that already exists at a
known path with a known face index and writes those exact bytes into the
repo. That is also what makes vendoring meaningful for an agent-first editor
— the whole point of vendoring is to freeze a specific, already-verified
binary (license terms known, glyph coverage already inspected, hinting
already tested) into the project rather than trusting a fetch to reproduce it
identically later.

**Trade-offs:** This costs convenience — an author without the exact font
installed must install it first (via the OS's normal font-installation flow,
package manager, or manual download) before vendoring, rather than the tool
doing that in one step. It also means Google Fonts-style "fetch by family
name" has to live in a separate, clearly-labeled tool (e.g. `fonts fetch` or
similar) with its own network, licensing-verification, and version-pinning
story — conflating "fetch from a service" with "vendor a local file" in one
command hides a trust boundary (network access, unpinned upstream content,
license file provenance) inside what should be a deterministic, offline,
reviewable operation. If network-sourced fonts are wanted later, it should be
an explicit additive tool, not a mode of `fonts vendor`.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: a**

**Reasoning:** Montagent's architecture—agent-first, vendoring fonts as
explicit file paths per ADR-0007—suggests the design prioritizes local
control and offline operation. The ADR language ("lists system fonts with
paths and face indices") indicates system fonts were the intended scope. For
an agent-first tool, a pure local-filesystem copy keeps operations
deterministic, offline-capable, and free from service dependencies or
network failures. The vendoring philosophy itself implies: bring what's
available locally into the repo under version control.

**Trade-offs:** Option (a) limits users to pre-installed fonts—less flexible
than supporting Google Fonts or URLs, and worse UX for teams with
heterogeneous systems. But gaining offline-first operation, zero network
complexity, and alignment with the agent-first architecture's need for
determinism outweighs the convenience loss. If a user wants a font, they
install it first; then vendor it. Clear workflow, no hidden state.

**Verdict: unanimous 3/3 for (a).**

### Q2 — How hard does the licence check block?

Options: (a) `fonts vendor` refuses outright, no file lands in the repo; (b)
copies but prints a loud warning the author must acknowledge; (c) always
copies, licence-checking is a separate subcommand or a later `validate`
finding.

🗳️ **Juror** (Claude Opus 5) — **VOTE: a, with one amendment**

**Reasoning:** The thing being prevented is an act of copying, so the check
belongs at the moment of copying — that is the only point where "nothing bad
has happened yet" is still true. Once the file is in the repo it is in git
history, and in a project whose files are meant to be committed and shared,
the violation is already complete and expensive to reverse (history rewrite,
not `rm`). (c) puts the gate after the irreversible step and is therefore not
a gate at all; it converts a preventable illegal act into a post-hoc report
of one.

(b) fails specifically because Montagent is agent-first. "A loud warning the
author must acknowledge" assumes a human reading a terminal. The actual
caller is usually an agent, and agents acknowledge warnings — that is the one
thing they are reliably good at. Any consent gate whose only cost is emitting
another token is not a consent gate. Warnings are the right instrument for
"this might not be what you meant"; they are the wrong instrument for "this
is illegal."

The amendment: refusal must be tri-state on *detection confidence*, not
binary on *licence status*. Three buckets: (1) known non-redistributable →
hard refuse, no flag, no override; the tool should not offer a mechanism for
doing the illegal thing, and a `--yes-i-know` flag is exactly the affordance
that makes the project a party to it. (2) known redistributable (OFL,
Apache, embedded permissive metadata) → copy silently, record the licence
identifier alongside the vendored file so `validate` and any later
distribution step can re-assert it cheaply. (3) unknown or unparseable —
which will be the common case, since font licence metadata is inconsistent
and often absent → refuse *until declared*, i.e. require the caller to pass
the licence identifier explicitly, which vendors it with an attributed,
auditable claim rather than a shrug. Bucket 3 is what keeps (a) from being
brittle: a heuristic that can only say yes/no will produce false refusals on
legitimate fonts and quietly wave through obscure proprietary ones, whereas
"declare it" turns detector ignorance into recorded human assertion.

**Trade-offs:** The real cost of (a) is friction on the unknown-licence path,
which is most fonts — every vendor of an unrecognised file now needs an extra
argument, and some users will type `OFL-1.1` reflexively to make the error
go away. That is an accepted loss: a false declaration is the declarer's
liability and is recorded, whereas a silent copy is the project's liability
and is not. The secondary cost is that a hard refusal with no override will
occasionally be wrong (a user genuinely holds a redistribution licence for a
font the detector blacklists); the answer there is that bucket 1 should be a
small, high-confidence list of known-non-redistributable families rather
than an aggressive classifier — narrow the blacklist rather than widen the
override. Against the alternatives: (b) buys a marginal convenience for
interactive humans at the price of being ineffective against the primary
caller, and (c) buys nothing at all, since a `validate` finding on a
committed binary tells you about a problem precisely when it has become
hardest to fix.

🗳️ **Juror** (Claude Sonnet 5) — **VOTE: a**

**Reasoning:** For an agent-first tool operating on legally distinct
third-party assets (font files), the licence check is a precondition of the
copy action, not a downstream annotation. Once a non-redistributable font
file lands inside the project repo, it is committed, diffed, pushed, and
potentially distributed to every collaborator or CI runner that clones the
repo — the illegal copy has already propagated before any warning or later
`validate` pass gets a chance to catch it. An agent operating
semi-autonomously is also far less likely than a human to notice or heed a
"loud warning" it can programmatically dismiss; treating the warning as a
checkbox to click past defeats its purpose. The only point where the check
can be enforced with certainty is before the write syscall that puts the font
bytes into the repo tree.

**Trade-offs:** This costs some flexibility — an agent or user who has a
valid personal/enterprise licence for a font Montagent's checker can't verify
(e.g. no machine-readable licence metadata, ambiguous EULA) is blocked even
when copying would be legitimate, so the tool needs an explicit
override/allowlist escape hatch for verified-licensed cases rather than a
dumb blanket ban. Option (b) is rejected because a warning is not a control —
it relies on the same author who wants the font to also be the one who
polices themselves, and in an agent-first workflow there may be no human in
the loop to read it. Option (c) is rejected because deferring the check to
`validate` means the violation is already baked into repo history (and
possibly already shared/pushed) by the time it's caught — undoing a
committed, distributed illegal copy is much harder than never writing it, and
"already committed" is precisely the failure mode a redistribution-licence
check exists to prevent.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: a**

**Reasoning:** The `fonts vendor` tool has one job: safely add fonts to the
project repository. License compliance is a hard constraint on that job, not
a nice-to-have. A font that cannot be legally distributed should not land in
a repo that will be shared, versioned, or deployed. Refusing outright keeps
the tool's contract clean: it either successfully vendors a redistributable
font, or it fails with a clear reason. Users can still obtain and use
fonts—they just can't vendor non-redistributable ones via this tool.

**Trade-offs:** Option (a) is more friction than (b) or (c), but that
friction exists for a reason. (b) shifts responsibility to the author to read
and act on a warning—feasible in theory, but in practice warnings are
dismissed, and the file is already committed. (c) defers the problem to a
separate validation step that runs after the damage is done; by then the
file is in the repo history, and undoing it is messy. For a tool whose job is
to prevent exactly this problem, blocking at the point of action is the right
place to enforce it.

**Verdict: unanimous 3/3 for (a)**, with Opus's tri-state-by-confidence
amendment adopted (made necessary, not optional, by the fsType research —
see ADR-0057).

### Q3 — Substitute-font policy on refusal

Options: (a) hard failure only, no automatic alternative; (b) hard failure
plus the tool suggests known metric-compatible or same-style open
substitutes; (c) hard failure plus the tool offers to vendor a different,
explicitly-open font family under the same fonts-table key.

🗳️ **Juror** (Claude Opus 5) — **VOTE: b**

**Reasoning:** The failure here is a *licence* failure, not a layout failure,
and the tool's job is to keep the author unblocked without silently making a
typographic decision on their behalf. (a) is honest but leaves the author to
rediscover, unaided, a fact the tool already knows — that Helvetica has a
metric-compatible open twin, that Gotham has near-neighbours. That knowledge
is cheap to surface and expensive to look up. (b) surfaces it and stops
there: the suggestion is advisory output, the author still makes the call,
and the fonts-table edit that follows is *their* declared change, which is
exactly what ADR-0007's census hangs off. The provenance stays clean — every
entry in the fonts table was put there by a human decision, and the
document's history shows a font swap as a font swap.

(c) crosses the line that matters. Writing a different family under the
*same* key makes the project file claim continuity where none exists: the
key still reads `heading`, the vendored bytes are a different typeface, and
every consumer downstream of that key now has to distinguish "the font we
chose" from "the font the tool settled for". Vendoring is also the one step
where the licence check is the whole point — having the tool resolve a
licence failure by vendoring something else turns a hard gate into a
negotiation. And "leaving layout to be re-verified" understates it: an
offered-and-accepted substitution invites the author to accept and move on,
precisely the path where an unreviewed census gets waved through, whereas a
substitution the author had to type themselves keeps their attention on the
thing ADR-0007 says is invalidated.

**Trade-offs:** (b) costs a curated substitute table that has to be
maintained and, more importantly, *honest*. "Metric-compatible" is a
falsifiable claim — Liberation Sans for Arial, yes; "same vibe" for a display
face, no. The tool must label each suggestion with its actual basis
(metric-compatible / same classification / visually similar) and its
licence, cite the source, and never present a style-similar match in
language that implies metric safety. A suggestion dressed up as equivalence
is worse than no suggestion, because it lets the author skip the census
they'd otherwise do carefully. Get that labelling wrong and (b) degrades into
(c) with extra steps. The other cost is scope creep — the substitute list is
a small piece of curated font-world knowledge living in a build tool — but
it's bounded, it's static data, and it never executes a decision.

🗳️ **Juror** (Claude Sonnet 5) — **VOTE: c**

**Reasoning:** ADR-0007 already normalizes the idea that a font swap is a
routine, expected event carrying a mandatory re-verification cost — the
census/finding machinery exists precisely to make swaps safe to perform, not
just safe to detect. Given that, refusing to act (a) wastes the tool's own
knowledge of the licence landscape, and merely suggesting substitutes (b)
still leaves the author to manually re-run the vendoring step by hand for a
font the tool already knows is open and available. Since the fonts table is
keyed and machine-checked (vendor + licence check), the tool is the right
place to perform the swap under the same key and immediately hand off to the
existing census/finding workflow — that's the workflow ADR-0007 already
assumes exists, so wiring the failure path into it is the coherent choice
rather than a new escape hatch.

**Trade-offs:** (c) risks being presumptuous — an automatic substitution
could surprise an author who wanted to pick deliberately, and
"explicitly-open" substitutes may not be stylistically close enough to be a
reasonable default. It costs more implementation complexity (the tool needs
a curated open-font catalog with real licence guarantees, not just metric
guesses) than (a), and more than (b) since it must actually perform a
vendoring action, not just print names. I reject (a) because silent-into-
nothing failure is exactly what the question asks to avoid, and it wastes a
mandatory-census system that's built to absorb this exact churn. I reject
(b) as a half-measure: naming alternatives without offering to act on them
forces a manual round-trip through the same tool for information the tool
already has.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: c**

**Reasoning:** For an agent-first video editor, option (c) respects
ADR-0007's mandatory font-swap census requirement while unblocking
workflows. The tool explicitly surfaces the tradeoff (open font available,
but layout must be re-verified), making it transparent and automatable —
agents can then systematically re-verify affected documents. Hard failure
(a) would halt agent workflows entirely; suggesting untested
metric-compatible alternatives (b) risks layout validity without enforcing
the re-verification mandate.

**Trade-offs:**
- **(a)** forces disciplined manual choice but grinds agent-driven workflows
  to a halt on licensing blockers
- **(b)** sounds helpful but treats metric compatibility as settled fact
  rather than verified claim, potentially sneaking invalid fonts past the
  census gate
- **(c)** requires downstream re-verification work, but makes that
  requirement explicit and leaves room for agents to automate it

**Verdict: split 2–1 for (c)** (Sonnet, Haiku) over (b) (Opus); resolved by
the map author for **(b)**, because research done alongside this ticket
found no font claiming formal metric compatibility with SF Pro Rounded —
only style-alikes (Open Runde, Roboto-Round) that explicitly disclaim
metric-compatibility — making (c)'s auto-vendor-under-same-key materially
riskier than the majority's framing assumed. See ADR-0057.

### Q4 — Does the licence check ever get re-verified?

Options: (a) `fonts vendor`-time only, no re-check ever; (b) `validate`/
`render` re-verify licence status on every run.

🗳️ **Juror** (Claude Opus 5) — **VOTE: a** (with one cheap refinement:
vendor-time attestation recorded in the file, which `validate` may *read*
but never re-adjudicate)

**Reasoning:** Licence status is not a property of the bytes on disk — it is
a property of a human/legal act performed at acquisition time, against a
source whose terms can change independently of the file. Nothing `validate`
or `render` could compute locally decides it. Option (b) therefore has only
two possible implementations, and both are bad. Either it re-derives licence
status from a hash-to-known-font table, which is a whitelist that fails
closed on every legitimately licensed font not in it (commercial foundry
purchases, corporate brand fonts, the user's own) — a false-negative machine
that blocks renders for correct projects; or it phones a licence oracle at
render time, which breaks offline rendering, makes `render` non-deterministic
across runs, and lets a third party's data change turn a previously-working
project file into a failing one. Both violate the premise that the project
file in git is the source of truth: a committed repo would no longer render
the same way tomorrow as today.

There is also a consistency argument the Question itself supplies. Montagent
already accepts that an agent with ordinary file tools bypasses `validate`
and `render`. Re-verification at those points is thus security theatre
against the exact adversary it purports to stop — anyone willing to drop an
unlicensed .ttf into `fonts/` and hand-edit the path is not slowed down by a
check running in the same process they can skip. It buys no enforcement
while charging every honest user determinism and offline capability.

What `fonts vendor` should do instead is *write down what it learned*: the
licence identifier, source, and file hash, into the `fonts` entry at vendor
time. That keeps the evidence in the file-as-truth artifact where git
history makes it auditable and reviewable, which is the accountability
mechanism this model actually has. `validate` may then check the mundane,
local, deterministic things — the declared path exists, the hash matches the
recorded one, the licence field is present — and report a *warning*, not an
error, when a font entry carries no attestation (hand-added). That is
integrity checking, not licence re-adjudication, and it stays offline and
reproducible.

**Trade-offs:** The real cost of (a) is that a licence revoked or misjudged
after vendoring is never caught by tooling; a project can keep rendering
with a font it is no longer entitled to use. I accept that, because (b)
would not reliably catch it either — it would catch a narrow, arbitrary
subset while breaking the common case. Licence drift is a compliance concern
belonging to an explicit, opt-in audit command (`fonts audit`, run when the
user wants a network-backed re-check) and to code review of the commit that
introduced the font, not to the hot path of every build. The refinement I
attach costs one schema addition and a hash comparison; it is the smallest
thing that makes the vendor-time decision legible later without pretending
the renderer is a lawyer.

🗳️ **Juror** (Claude Sonnet 5) — **VOTE: b**

**Reasoning:** Montagent already accepts that `validate`/`render` can't stop
an agent from hand-editing the project file, but that's a different threat
than licence drift on a *vendored asset*. The vendor-time check only proves
the file was licensed at the moment it was copied in. After that, the file
sits in git indefinitely and can be replaced, renamed onto, re-pointed, or
copied from another project by ordinary file tools — none of which touch
`fonts vendor` at all. A one-time check that's never revisited means the
declarative project file (the supposed source of truth) can drift into
shipping an unlicensed font while `validate` reports green forever. Given the
project's own principle that render/validate are the last line of defense
against bypass via file tools, licence status is exactly the kind of fact
that needs re-grounding against the actual bytes on disk, not trusted from a
historical event. A cheap re-check (hash/fingerprint the committed font file
against the known-licensed set, or re-verify metadata) at `validate` time
keeps the guarantee meaningful without requiring network calls on every
`render`.

**Trade-offs:** This costs implementation complexity — you need a stable way
to fingerprint "known-licensed" fonts (hash registry, embedded licence
metadata, or a call to a licensing service) and decide what "unknown
licence" should do (hard fail vs. warn). It also risks false positives if
font files legitimately get modified (subsetting, hinting) after vendoring,
so the check must be tolerant of that or done at vendor time plus recorded,
not literally re-derived from scratch. Rejecting re-verification (option a)
is cheaper and matches "file-as-truth, agent can bypass anyway," but it
makes the licence check theater — it protects nothing after the moment it
runs, which is a worse trade for a guarantee users likely care about (legal
exposure), unlike e.g. font kerning correctness where staleness is low-stakes.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: a**

**Reasoning:** The "file-as-truth" model makes the project file itself
authoritative. Once `fonts vendor` validates and commits a font, that asset
becomes a blessed entry in the source of truth. Re-validating on every
`validate`/`render` run contradicts this principle—you're asking the runtime
to distrust what's already in git. The prompt explicitly states that
`validate`/`render` already accept that agents can bypass them with ordinary
file tools, so attempting re-checks there provides no additional safety
while adding runtime cost.

**Trade-offs:** Option (a) relies entirely on tool discipline—agents must use
`fonts vendor`, not manually edit the project or commit fonts directly.
Option (b) would catch workarounds and manual edits, but wouldn't
meaningfully prevent a determined agent (you've already conceded they can
bypass validate/render), and would tax every render with checks that are
performant only if cached externally. The "file-as-truth" principle suggests
one validation gate (the tool) at commit time is cleaner than distributed
enforcement.

**Verdict: split 2–1 for (a)** (Opus, Haiku) over (b) (Sonnet); resolved by
the map author for **(a) as refined by Opus** — a vendor-time attestation
(licence, source, hash) that `validate` checks for integrity (hash match,
presence) without re-adjudicating licence law. This answers Sonnet's drift
concern with the deterministic, local half of "re-verify" that's actually
achievable.

## Round 2

### Q5 — What's on the blocklist (bucket 1), and how is it maintained?

Open-ended; no bounded option set.

🗳️ **Juror** (Claude Opus 5) — **VOTE:** Ship a small, curated, in-repo
blocklist maintained by Montagent itself — seeded with the Apple system font
families (SF Pro, SF Pro Text/Display, SF Pro Rounded, SF Compact, SF Mono,
New York, and the private `.SF NS *` / `.AppleSystemUIFont` internal names) —
versioned with the code, extended only by PR, and append-only from the
user's side: a project/user config may ADD entries (org policy), never
remove or disable shipped ones. No external database, no community
registry, no runtime fetch.

**Reasoning:** The research already establishes there is no machine-readable
redistribution signal, so bucket 1 cannot be derived — it can only be
asserted. An asserted list must live somewhere with accountability, and the
only party with that accountability here is Montagent. Every alternative
fails on a specific point: a user-editable list is not a gate at all (the
one caller most likely to delete the SF Pro entry is precisely the caller
the entry exists for); an external database creates a network dependency and
a supply-chain input on a code path whose whole job is to refuse, and gives
the refusal a third-party failure mode ("offline, so it copied"); a
community registry inherits the legal-claim problem without the legal
review — an incorrect community entry either wrongly blocks a legitimate
font or, worse, wrongly permits one, and Montagent still eats the
reputational hit.

The list is also not load-bearing in the way a security allowlist is, which
is what makes "small and shipped" sufficient. Bucket 3 is the real
workhorse: everything unknown already refuses. Bucket 1 exists to convert
the handful of cases where refusal-with-declaration is wrong — where the
correct answer is "no, and there is no declaration you can make" — because a
well-meaning agent faced with "declare a licence" for SF Pro will cheerfully
declare something. So the list needs to cover fonts that are (a) widely
present on developer machines, (b) attractive defaults, and (c) clearly
non-redistributable. Apple's system fonts are the canonical instance;
Microsoft's core web fonts (Segoe UI, Calibri) and the Adobe/Monotype
desktop-licence families are the obvious next entries by the same test.
Staying small is a feature: a short list can actually be reviewed against
the licences it claims to encode.

Append-only user extension is worth having because it costs almost nothing
and serves a real case — a studio that has an internal "never vendor these"
list — while keeping the gate's floor fixed. Match on family name plus
PostScript name, case- and whitespace-insensitive, and treat the match as
advisory-strong rather than cryptographic: a renamed file will slip through,
and that is fine, because the threat model is an agent doing the wrong thing
by default, not a user determined to launder a font.

**Trade-offs:** The list will be incomplete and will go stale; users will
hit bucket 3 for fonts that morally belong in bucket 1, and the system will
offer them the declaration path for a font they cannot legally declare. That
is the correct failure direction but it means the hard gate carries less
weight than its name suggests — worth saying plainly in the docs so nobody
treats an empty bucket-1 result as clearance. Second, false positives have
no escape hatch by construction: a user with an actual Apple licence
covering their use, or someone who legitimately owns a differently-licensed
font that shares a blocked name, is simply stuck. This is survivable only if
the refusal message is genuinely helpful — it must name the specific reason
and point at the alternative (reference the font by system path at render
time rather than vendoring it into the repo), otherwise the gate reads as a
bug and someone patches it out. Third, Montagent takes on a standing,
unbounded-in-principle maintenance duty and an implicit legal-assertion
posture: publishing "font X may not be redistributed" is a claim about
someone else's licence, and a wrong entry is a small but real liability. Cap
that by keeping entries to cases with a citable licence clause, recording the
citation next to each entry, and refusing to grow the list on vibes.

🗳️ **Juror** (Claude Sonnet 5) — **VOTE:** Ship a small, hardcoded,
non-user-editable blocklist maintained by Montagent itself, starting with the
Apple system families (SF Pro, SF Pro Rounded, SF Compact, New York, and the
private `.SF NS *` names), expanded only by Montagent maintainers via normal
code review/release — not sourced from any external database or community
list.

**Reasoning:** Bucket 1 exists specifically to be a hard, non-overridable
gate — its entire value is that a caller *cannot* talk their way past it.
That requirement is incompatible with user-editability (a user-editable
"hard block" is just bucket 3 with extra steps) and with automatic external
sourcing (an unreviewed feed becomes a supply-chain vector: poison the feed,
silently unblock a font, or — worse — silently block a legitimate one and
break unrelated projects). A community-maintained list has the same trust
problem plus coordination overhead Montagent doesn't need yet. Since research
already established there's no reliable in-file signal for redistributability,
this list can only ever be a curated set of known-bad names/identifiers —
which is exactly what a small, versioned, code-reviewed constant is for.
Starting with only the one concrete case in hand (Apple's system fonts)
avoids speculative scope creep; the tri-state design already routes every
other unknown font through bucket 3 (safe-by-default refusal), so bucket 1
doesn't need to be comprehensive to be sound — it only needs to be correct
for what it does contain.

**Trade-offs:** This means Montagent takes on an ongoing, manual curation
burden — every new known-non-redistributable font family requires a code
change and release, so the list will always lag reality and can never be
crowdsourced or auto-updated for fast coverage. It also concentrates
liability/judgment calls on Montagent's maintainers (what counts as "known
non-redistributable" enough to hard-block, versus merely unknown) with no
formal external authority backing the list, which could invite disputes if a
blocked/unblocked call is contested. Practically low-risk here because
bucket 3's default-refuse already catches everything not explicitly known
either way — bucket 1 is a strict-improvement fast path for egregious,
well-documented cases, not the primary safety mechanism.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE:** Small, project-maintained
blocklist, not user-editable, starting with Apple system fonts and variants.

**Reasoning:** A hard-refuse gate must be unambiguous and tamper-proof.
User-editability defeats the purpose of a hard block. Community or external
maintenance introduces misaligned incentives and dependency risk. The
concrete case (Apple fonts) and research findings (no machine-readable
signal in font files) support a minimal, curated approach where project
maintainers explicitly document legal justification for each entry, starting
with the well-documented Apple platform licensing restrictions.

**Trade-offs:** Maintaining the list requires periodic review as new fonts
and legal clarity emerge. Users cannot override hard-blocked fonts even with
written justification. If the list is incomplete, problematic fonts fall
through to bucket 3 (explicit declaration required). If overly conservative,
legitimate use cases hit the hard block unnecessarily. Success depends on
maintainer diligence and transparency about criteria.

**Verdict: unanimous 3/3**, converging independently on the same seed list
(Apple system fonts) and the same non-user-editable shape. Opus's
append-only user-extension detail adopted in ADR-0057.

### Q6 — Where does the attestation (licence identifier, source, file hash) live in the schema?

Options: (a) per file — each `{"file": ...}` entry in the chain gets its own
attestation fields; (b) per key — one attestation for the whole fallback
chain; (c) something else.

🗳️ **Juror** (Claude Opus 5) — **VOTE: c — per *file*, but recorded once per
file path in a separate vendor table, not inline on each chain entry**

**Reasoning:** Licence, source, and hash are properties of a byte sequence,
not of a style role. `brand+fa` chains Inter and Vazirmatn — SIL OFL from
Google Fonts and an independent Vazirmatn release — vendored by different
`fonts vendor` invocations, with no single licence identifier that is
truthful for both. Per-key (b) forces either a lie or an array that silently
re-invents per-file granularity anyway, and it breaks the moment a chain
gains a third file. So the granularity must be per file; that part is
settled against (b).

The reason I don't stop at plain (a) is that the fonts table is a
*reference* structure, not a storage structure: `fonts/Inter-SemiBold.ttf`
already appears under both `brand` and `brand+fa` in the example, and in
real projects a base Latin face will be reused across most keys. Inlining
the attestation on each `{"file": ...}` entry duplicates the same hash and
licence N times, and duplicated facts drift — a re-vendor updates one copy,
`validate` then sees two attestations for one path that disagree, and now it
has to arbitrate, which is exactly the adjudication the design just decided
to avoid. Keying attestations by file path makes the invariant structural:
one path, one hash, one licence, and `validate`'s cheap check becomes "every
path referenced by any chain resolves to an attestation whose hash matches
the bytes on disk." Chain entries stay what they are — ordered fallback
references — and `fonts vendor` writes into the table once per fetched file
regardless of how many keys later point at it.

**Trade-offs:** It costs one more top-level construct in the project file
and an indirection an agent must follow to see a chain entry's licence; a
naive reader of a single chain entry no longer sees everything inline. It
also needs a defined rule for orphaned entries when the last chain
referencing a file is deleted (my preference: `validate` warns, a prune step
removes — don't make deletion of a key silently discard an attestation the
user may still need). Against (a): (a) is right about granularity and wrong
only about placement, so the gap is narrow — if the team judges the
duplication tolerable and wants everything visible at the point of use, (a)
is an acceptable fallback, provided the schema states that identical paths
must carry identical attestations and `validate` enforces that as a hard
error rather than resolving the conflict itself. Against (b): it cannot
represent a mixed-provenance chain at all, which is the common bilingual
case Montagent explicitly supports, so it is not a viable option here.

🗳️ **Juror** (Claude Sonnet 5) — **VOTE: a — per-file attestation**

**Reasoning:** The `fonts` table key is a fallback chain, not a licensing
unit — it's an editing/rendering convenience that groups otherwise-
independent font files (as `brand+fa` demonstrates by design: an Inter file
and a Vazirmatn file, plausibly from different foundries under different
licences, vendored at different times). Licence identifier, source, and file
hash are all properties of a specific artifact — the file's bytes — not of
the chain it happens to be listed in. A hash is definitionally per-file;
splitting the attestation so the hash is per-file but the licence/source are
per-key would be an incoherent schema, forcing an array-vs-scalar mismatch
and defeating the stated goal of a cheap hash-match check. `validate` needs
to walk each chain entry and confirm *that file's* hash matches *that file's*
attestation — a per-key attestation can't answer "does file 2 of this chain
still match what we vendored" without inventing a second layer of file-level
metadata anyway, at which point per-key is just per-file with extra
indirection.

**Trade-offs:** Per-file costs a little repetition when a chain's files
genuinely share one licence/source (you write the same identifier twice),
and vendor-time tooling must attach metadata per entry rather than once for
the key. But per-key is unworkable the moment a chain mixes licences — which
the example explicitly anticipates — and would require re-deriving per-file
identity from filenames or a parallel structure at validate time, exactly
the re-adjudication the design is trying to avoid. Option (c) (e.g., a
separate top-level manifest keyed by file path, referenced from the chain)
is defensible for dedup but adds a layer of indirection with no benefit over
inlining, since files are rarely shared across many keys in a font table;
not worth the complexity here.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: a**

**Reasoning:** The design documentation itself makes the case: fallback
chains mix independent files that are "potentially vendored separately with
a different licence." Attestation (licence ID, source, file hash) is
inherently a property of the file artifact, not of the grouping key. In
`brand+fa`, Inter-SemiBold and Vazirmatn-SemiBold have distinct legal
provenance, distinct sources, distinct hashes, and distinct vendor records.
A per-key attestation cannot represent this accurately.

**Trade-offs:** Per-file means more schema verbosity and redundancy when all
files in a chain share source/licence (e.g., multiple weights of the same
typeface). But per-key fails the mixed-source case the design explicitly
calls out, breaks attribution fidelity, and makes validation ambiguous about
which file failed attestation. The cost of verbosity is outweighed by
correctness and legal clarity.

**Verdict: split 2–1 for (a)** (Sonnet, Haiku) over (c) (Opus); all three
agreed on per-file granularity, disagreeing only on placement. Resolved by
the map author for **Opus's (c)** — the majority's own worked example
(`Inter-SemiBold.ttf` referenced by both `brand` and `brand+fa`) already
demonstrates the duplication/drift problem plain per-entry inlining would
ship with. See ADR-0057.

### Q7 — Does resolving this ticket also fix the live fixture defect?

Options: (a) fix the fixture as part of this ticket; (b) separate follow-up
`task` ticket.

🗳️ **Juror** (Claude Opus 5) — **VOTE: b**

**Reasoning:** The map's ticket-type taxonomy is the project's own encoding
of "decide" vs. "do," and the whole value of a typed tracker is that the
types mean something. A `grilling` ticket resolves by talking a design to a
conclusion; the fixture fix is mechanical execution of that conclusion —
re-vendor a real font, update the committed project file — which is
precisely what `task` exists to name. Option (a)'s "the design isn't proven
until it's exercised" is the strongest counter, but it proves too much:
under that rule no grilling ticket could ever close without an
implementation, and the project deliberately separated `prototype` (the type
that *does* exist for exercising a design before committing to it) from
`grilling`. If the fonts design genuinely needed empirical validation, the
correct move was a `prototype`, not smuggling implementation into a design
ticket. Crucially, the defect is pre-existing and independently
discoverable — it is a live bug in the fixture that would need fixing
whether or not this design ticket had ever been opened. Attaching it to the
grilling ticket buries a shipped fix inside a decision record, and makes the
design ticket's closure ambiguous: did it close because the design settled,
or because the font got vendored? A follow-up `task` created at close time,
explicitly unblocked by this decision, is exactly the "decision unblocks
work" case the task type is described as covering, and it leaves a clean
audit trail: the ADR says what the policy is, the task says what was done to
comply with it.

**Trade-offs:** The cost is real: a known-broken fixture stays broken for
the interval between ticket close and task execution, and there is a genuine
risk the follow-up is never picked up — the classic "we'll do it next
sprint" decay. It also forgoes the cheap validation of dogfooding the new
`fonts vendor` flow on a real case immediately, which might surface a design
flaw while the design is still warm. I accept those because the mitigation
is cheap and reliable: create the `task` ticket in the same act as closing
the grilling ticket, link it bidirectionally, and note that the fixture is
unrenderable until it lands — a tracked, blocking task is not the same as an
untracked intention. And the alternative's cost is worse and permanent: once
one grilling ticket ships code, the type boundary is advisory rather than
real, and every future grilling ticket inherits an argument about how much
implementation "counts as" resolution.

🗳️ **Juror** (Claude Sonnet 5) — **VOTE: b**

**Reasoning:** The map's ticket taxonomy exists precisely to separate
deciding from doing: `grilling` tickets settle a design question through
conversation, `task` tickets carry out mechanical/manual work the decision
unblocks. Re-vendoring a font into the fixture is exactly the kind of
concrete, checklist-executable work — find a redistributable font, run
`fonts vendor`, update the fixture's project file, confirm it renders — that
the `task` type exists for. It requires zero further design conversation
once this ticket lands; it's an application of the freshly-settled policy,
not a continuation of the settling. Folding it into the grilling ticket
blurs the type boundary the map deliberately draws, and makes the grilling
ticket's "done" criterion depend on execution work that can slip for reasons
(license research, asset selection) unrelated to whether the design itself
is sound.

**Trade-offs:** This costs a moment of the fixture staying broken between
this ticket's close and the follow-up task's completion — a real but
bounded gap, and one the wayfinder map is built to track explicitly (the
defect doesn't vanish, it becomes a visible, typed, sequenced ticket). The
alternative (a) buys immediate proof-by-exercise of the design, but at the
cost of conflating a decision record with an execution record, making the
grilling ticket's scope open-ended (blocked on font sourcing/licensing
legwork) and harder to close cleanly. Given the map already models "decision
unblocks task" as a first-class relationship, the defect fix is better
tracked as its own node than smuggled into the design ticket's closure.

🗳️ **Juror** (Claude Haiku 4.5) — **VOTE: b**

**Reasoning:** The ticket type taxonomy is explicit and semantically
load-bearing in this project's wayfinder system. A `grilling` ticket's job
is to settle a design question (in this case: how font vendoring, licensing
checks, and substitute policies *should* work). That design is now
settled—the question is answered and documented. A `task` ticket's job is to
execute concrete work (in this case: apply the newly-designed policy to a
specific broken fixture). These are different responsibilities. The design
question did not ask "how do we make the halloween fixture renderable"—it
asked "how should font vendoring work." Fixing the fixture is applying that
answer, not proving the design itself. Separating them keeps the tracker
coherent and lets the design ticket close cleanly.

**Trade-offs:** The downside is that the design sits unvalidated until the
task ticket runs (you don't know if the tools actually work until someone
uses them). But that's precisely what the task ticket is for—to exercise and
validate the design in practice. Bundling execution into the design ticket
would blur the tracker's semantic categories and make it harder to track
which questions have been *decided* versus which decisions have been
*executed*.

**Verdict: unanimous 3/3 for (b).**
