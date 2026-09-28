---
status: accepted
amends: 0011 (the verb table's `frame` row gains range arguments and the counts are unchanged;
  the caption obligation is restated as an **attribution** obligation, discharged by the
  `query --at` block when the image is one frame and by the fitted per-tile label plus the
  range-level provenance list when it is a sheet; its published full-scale visual-token figure
  is corrected to a two-tier fact, see this ADR's section 8), 0094 (its unconditional structured disclosure is specified to carry its
  full content in the plain-text form, not only behind `--json`, and the per-tile provenance it
  mandates is named as what discharges ADR-0011's amended obligation)
---

# The range is `--from`/`--to` on both surfaces, and `frame`'s caption obligation becomes an attribution obligation

> **Amended by [ADR-0098](0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md).** Section 7 left the attribution guarantee discharged
> *jointly* by a fitted label and the range-level provenance list, and warned that *"a
> guarantee with two owners is weaker than one with a single owner, and #400 must be read as
> binding."* ADR-0098 completes the label half and **makes the guarantee width-proof rather
> than width-dependent**: the label's type size is **floored at 8 px served**, and content
> gives way instead of type, sheet-wide. So the label is legible at every width by
> construction, and what degrades is how much identity it carries. Section 7's prediction that
> a fitted label *"cannot name the presence set"* at the 140 px floor is confirmed and
> strengthened by measurement — the set does not fit at the **180 px target** either
> (2.67 px typical, 1.83 px busiest), so the routing this ADR chose on a prediction now rests
> on a number.

[#401](https://github.com/MBehtemam/Montagent/issues/401), on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved by a jury of three
independent models (Opus 5, Sonnet 5, Fable 5.1) put to seven sub-questions; ballots verbatim
in [`docs/research/juries/contact-sheet-tool-surface/`](../research/juries/contact-sheet-tool-surface/README.md).

[ADR-0094](0094-the-sheets-instants-are-visual-states-sampled-at-the-first-painted-frame.md)
fixed which instants the sheet shows.
[ADR-0095](0095-the-sheets-budget-is-served-tile-width-and-overflow-refuses.md) fixed its
budget and its refusal. Both explicitly deferred the flag spelling and the ADR-0011
amendment to this ticket, and ADR-0095 added that **#401 carries more weight than its title
suggests**, because the refusal is the common case and its ergonomics decide whether the
feature reads as working.

This ADR fixes **how the range mode is spelled, which surfaces carry it, what each existing
flag does when a range is present, and what `frame` owes the agent about attribution.** It
does not decide the two flag names ADR-0094 mandated — see *What this ADR does not decide*.

## Decision

1. **The range is `--from`/`--to`, half-open `[from, to)`, and both are required.** The same
   spelling, the same reading and the same wording as `query`, `preview` and `render`. There
   is no defaulted `--to` meaning "to the end of the project".
2. **The presence of the pair *is* range mode.** There is no `--sheet` flag. `--at` is
   mutually exclusive with `--from`/`--to`, and the overlap is refused **in the verb**, not in
   `clap`, so both surfaces inherit the rule.
3. **`frame` without a range is untouched**, in every respect. `--at` alone behaves exactly as
   it does today.
4. **Both surfaces carry the range mode.** MCP hands back the caption text block and the sheet
   as one image block, and MCP's `at` becomes optional — the one non-additive change to the
   existing surface. The CLI writes one file to its already-required `--out`.
5. **`--full` with a range is refused**, as an invocation error that names why. Never silently
   ignored, never given a new meaning.
6. **`--png` carries over unchanged. `--json` carries over unchanged in meaning** — canonical
   JSON *instead of* the text caption, sheet returned either way — **and the plain-text form
   carries the full disclosure content**, not a summary of it.
7. **`--crop` composed with a range is the existing `--crop`, never a new flag name — and it
   is refused today as not-yet-legal**, pending [#406](https://github.com/MBehtemam/Montagent/issues/406)
   and its blocker [#402](https://github.com/MBehtemam/Montagent/issues/402).
8. **ADR-0011's caption obligation is restated as an attribution obligation**: *the agent must
   be able to attribute what it sees to what produced it, without a second call.* At
   single-frame scale that is the `query --at` block, exactly as before. At sheet scale it is
   ADR-0094's fitted per-tile label **together with** a range-level provenance list — one line
   per tile carrying the sampled instant, the run boundary and the presence set as element ids.
9. **The verb table's `frame` row gains its range arguments; the counts are unchanged.** Nine
   MCP tools, twelve CLI commands, two resources. This is not a tenth verb in
   [ADR-0037](0037-derived-time-signature-is-a-provenance-gap-not-a-tool.md)'s sense.

## Why

### 1. A fourth range grammar would teach a second way to say one thing

Three verbs already spell a span `--from`/`--to`, half-open, in nearly identical sentences.
All three jurors chose that spelling, and all three rejected the trial agents' own
`--per-state` / `--each-cut` on a reason **the brief did not supply**: those names describe a
*selection rule*, and ADR-0094 has already fixed that rule as the only one the verb has. A
flag named after the rule advertises a sibling rule to switch to, and there is none — ADR-0094
closed `--visual` and made the keyframe question a separate additive flag. As Juror 3 put it,
*"a flag whose sole value is the default is a flag with nothing to say."*

The trial agents' instinct is real and is answered elsewhere: what they wanted named is what
the sheet *does*, and that belongs in the disclosure sentence and the tool description, not in
an argument.

**The absent default is the load-bearing half.** Two jurors independently reached ADR-0095's
own risk note to argue it: the refusal is the *common case* past roughly 65 seconds of the
fixture's density, so an implied "to the end" would make the first call on any real project a
refusal on a range **the caller never typed**. Requiring both arguments keeps the refusal
attributable to something the agent chose, which is the difference between a tool that
refuses and a tool that appears broken. This is the first of three places where ADR-0095's
warning about refusal ergonomics decides a question.

### 2. The pair is the mode, because there is nothing else a range could return

All three jurors reached this and all three cited `measure`, whose `element` / `at` /
`elements` / `all` groups are mutually exclusive and refused in the verb. Juror 3 added the
argument that makes a `--sheet` flag indefensible rather than merely redundant: motion is out
of scope, the tile count is derived and not a caller's knob, so **the sheet is the only thing
a range can return from `frame`**. A flag that must always accompany the pair and can never
be omitted with it carries no information.

Against it, Juror 1 named the real cost — discoverability. Nothing in the flag list announces
that a sheet mode exists, whereas a `--sheet` field would advertise itself in the MCP schema
an agent reads every turn. That cost is accepted on ADR-0011's own standing cost model: an
MCP schema field is permanent context rent on every call, and the mode is discoverable from
the tool description instead, which is where `frame` already explains `--crop` and `--full`.

The refusal placement is not a style preference. The existing CLI comment already requires
it — *"whether the flags ask for a frame at all... is the verb's rule and not argv's: the MCP
surface takes the same arguments with no `clap` to arrange them, and a `clap` requirement here
would leave that surface uncovered"* — and a range mode is exactly the case that comment was
written for.

### 3. Both surfaces, and the strongest reason is a correctness cliff on MCP

The one piece of trial evidence that looks like an argument for CLI-only is not one. An agent
stayed on the CLI *purely* because looping 18 renders in one bash call beat 18 MCP
round-trips, then paid an extra file read per frame, and **said explicitly that both options
were bad**. The sheet dissolves the reason it chose: one call, not eighteen, with the image in
the result and no read.

Two jurors then found the argument that settles the direction: the **>20-image-block clamp is
an MCP-side defect** — more than twenty image blocks in one request silently clamps every
image in it, earlier turns included — so MCP is the surface where the corruption happens and
the last one to drop. The CLI keeps the mode for the human and for scripted use, and `--out`
stays required there for the reason already on record: no ADR names a filename, and a read
must not write into somebody's project directory uninvited.

MCP's `at` becoming optional is the only change this ADR makes to an argument that already
exists. The core verb's `Ask.at` is already `Option<i64>` and the CLI's `--at` is already
optional, so the MCP adapter is the only place the requirement was ever expressed.

### 4. `--full` has no referent in range mode, and silence is the failure this map exists to prevent

`--full`'s entire documented content is *"you are choosing to spend the difference."*
ADR-0095 measured that in range mode **there is no difference**: a sheet spends 1518–1568 of
the standard tier's 1568 tokens at every tile count from 4 to 48. And ADR-0095 already
requires tiles to be rasterized at true project pixels and composited down, so the fidelity
`--full` would ask for is what the sheet already does internally. Above the served long edge
the extra pixels are discarded by the API before anyone sees them.

All three jurors refused it rather than ignoring it, and all three reached the same reason:
this project's characteristic failure is a silence that reads as coverage. The trial agent
that missed the pixel-only defect had a structurally blind sampler and said nothing about it.
A flag that accepts a request for full fidelity and quietly does not honour it is that same
failure in miniature — Juror 3: *"an agent that passes `--full` believing it bought detail will
read a 140 px sheet as full-res evidence."*

Re-purposing it was considered and rejected: any new meaning (say, the 140 px floor) would
smuggle a legibility knob in under a resolution name and collide with ADR-0095's rule that the
count is derived and never set by the caller.

**The refusal message is the teaching surface.** It states the two facts that make the range
mode's currency intelligible in one line — the token cost saturates at every tile count, and
tiles are already true pixels — which is the second place refusal ergonomics decide a design
question.

**This refusal encodes a tier fact, and that is recorded as a revisit condition rather than
designed away.** The 1568-token saturation is a property of the standard tier. If a future
tier serves materially more pixels, this refusal is the thing to reopen — deliberately, as
Juror 1 asked, rather than by letting a silent flag quietly become meaningful again.

### 5. Unconditional cannot mean behind a flag

`--png` is orthogonal: #397 measured 700 tokens for both a 69,681-byte JPEG and a
527,379-byte PNG of the same frame, so encoding is token-irrelevant on a sheet exactly as it
is on a frame. Juror 3 noted it may matter *more* here, since JPEG ringing at 140–180 px is
noise in precisely the fine-detail class that dies first.

`--json` keeps its contract as a serialization choice. **It does not become the gate on the
disclosure.** This was the panel's clearest split, and it turns on a reading of ADR-0094 §6.
That clause requires *"an unconditional structured disclosure... plus one sentence of prose
restating it."* Juror 2 read that as settled in favour of prose-in-text and structure behind
`--json`, and called the question closed — but that reading turns **plus** into **or**, and it
is the one reading §6's own word *unconditional* forbids.

Jurors 1 and 3 supplied the consequence. The default path — no `--json` — is the one an agent
reaching for pixels actually takes, so hiding the disclosure there makes **the default answer
the untrustworthy one**, and the defence this whole map rests on would protect only callers
who happened to pass a flag. Juror 3: *"a prose summary on the default path would rebuild that
failure by design."*

The cost is bounded and was measured before it was argued: a whole sheet answer is **1,413
characters**, against 28,410 for the eighteen `query --at` blocks it replaces. The text form
therefore carries the rule, `skipped` with reasons, the dropped audio-only boundary count, the
untiled keyframe count, the served tile width and which rung produced it, and the fixed
`blind_to` enumeration. `--json` renders the same facts canonically instead of as prose.

One consequence for [#412](https://github.com/MBehtemam/Montagent/issues/412), which owns the
codes: this constrains them to be **renderable as prose**, not only as enum values.

### 6. `--crop` plus a range is reachable today, so the surface cannot be left open

[#406](https://github.com/MBehtemam/Montagent/issues/406) owns whether per-tile crop ships and
what a cropped sheet must disclose, and it is blocked by
[#402](https://github.com/MBehtemam/Montagent/issues/402). Juror 2 argued from that to saying
nothing here. The reason that fails is Juror 3's: the combination is **invocable right now**,
and there is no defined answer — an agent gets either a whole-frame sheet with the crop
silently dropped, or, via #402's live half-scaling defect, a sheet at the wrong scale. Leaving
an undefined live invocation on a surface whose entire argument is that it never lies is worse
than either shipping or refusing.

So the spelling is fixed — the existing `--crop`, composed with the existing range, never a
new flag — and the composition is refused today with a named not-yet-legal code, whose class
is #412's. Fixing the name costs #406 nothing substantive; ADR-0095 already derives a cropped
sheet's tile count from the same crop rectangle, so it presupposes these semantics. What #406
keeps is everything that matters: whether the mode ships at all, and what it must disclose
about the band it cannot see — which is not a small question, because a cropped sheet **looked
clean** while the wrong-photo defect had vanished from it entirely.

### 7. The obligation ADR-0011 was protecting is attribution, and a fitted label cannot carry it alone

ADR-0011 is emphatic: *"`frame` must print the `query --at` block alongside the image,
unconditionally."* Both doc comments and the MCP tool description say there is deliberately no
flag to suppress it. The range mode cannot honour that as written — #397 measured eighteen
such blocks at **28,410 characters against 1,413**, a 20.1× multiplier and the larger,
previously uncosted half of the sheet's cost. Paying it would keep the cost the feature claims
to remove while pretending to remove it.

ADR-0011's stated *reason* is narrow and is what survives: *"looking at a picture without
knowing which elements produced it is how a defect gets attributed to the wrong element."* The
harm is **misattribution**, and Juror 1 established that it is a **per-tile** hazard — a
range-level block naming which elements appear somewhere in the span licenses exactly the
inference that pins tile 7's bad pixels on an element that was not on screen at tile 7. So a
single range-scoped block does not discharge the obligation, and option (b) fails.

Two jurors concluded that the per-tile label discharges it. **Juror 3 alone found why that is
not enough, and it is the finding this panel was worth**: ADR-0094's label is *fitted* to tile
width, so at the 140 px floor it can carry an instant and little else — **it cannot name the
presence set**, at exactly the width where a degraded sheet is hardest to read and where the
refusal is about to fire. A guarantee that holds at the target and lapses at the floor is not
a guarantee. Juror 1 half-saw this from the other side, noting that a fitted label *"cannot
carry a full resolved stack"*, then accepted a second `frame --at` call as the price instead of
following it through.

The fix costs almost nothing, because the content already exists twice over: ADR-0094 already
mandates per-tile provenance in the disclosure, and the re-merged visual cut list is what the
feature computes to choose its instants. Printing it once — one line per tile: sampled
instant, run boundary, presence set as element ids — is the sheet-scale equivalent of the
`query --at` block. Juror 1 had already said it would accept that as *"a friendly amendment
rather than an alternative"*, so the panel agrees on substance once the option is on the table.

What is given up is real and is the right price: the **resolved** stack — geometry, resolved
keyframe values — stays one call away. ADR-0094's refusal of the midpoint is what makes that
call possible: because every tile's instant is one the document itself produces, an agent can
re-call `frame --at <that instant>` or `query --at <that instant>` and reproduce the tile
exactly. And ADR-0011 already demotes `frame` from measuring, so sending an agent to `query`
for resolved geometry is the division of labour it argues for.

**This transfers load onto [#400](https://github.com/MBehtemam/Montagent/issues/400), and the
transfer is stated rather than left in the gap between two tickets**: a label that is
unreadable at the served width, or that drops its identifying field, is no longer a cosmetic
defect but a violation of ADR-0011 as amended here.

### 8. The `2691` figure is corrected here because the refusal's reasoning contradicts it

ADR-0011 publishes `1080x1920 -> 2691` visual tokens as flat fact, and so do
`crates/montagent-core/src/verbs/frame.rs`, `crates/montagent/src/cli.rs` and
`crates/montagent/src/mcp.rs`. #397 measured that it holds **only on the high-resolution tier**
(Claude 4.7+, capped at 2576 px / 4784 tokens); the standard tier caps at 1568 px and serves a
1080×1920 frame as 819×1456 = **1560 tokens**. So `--full` is a 3.84× multiplier on one tier
and 2.23× on the other.

This is not folded in for tidiness. §4's refusal rests on the sentence *"the sheet saturates
the tier's 1568 tokens at every tile count"*, which cannot coexist with a table stating 2691
as unconditional. An amendment that inherited the old figure would ship a contradiction.

[#405](https://github.com/MBehtemam/Montagent/issues/405) had already reached this conclusion
and stated the ordering this ADR follows — *"ADR-0011's verb table is due an amendment for
`frame`'s range mode, and **an amendment would inherit these numbers. Correct them first.**"*
Juror 3 proposed the same fold-in independently, without having been told #405 exists. **This
ADR corrects the figure in prose; #405 remains open and owns the four code sites**, including
the `--full` help text that overstates the penalty.

The figures, as they stand:

| authored | standard tier (1568 px cap) | high-res tier (2576 px cap) |
| --- | --- | --- |
| 540×960 | served as-is, **700** | served as-is, **700** |
| 1080×1920 | downscaled to 819×1456, **1560** | **2691** |
| a sheet, any tile count 4–48 | **1518–1568** | n/a — composed to the served edge |

## The verb table row

ADR-0011's table keeps its original words per `docs/agents/domain.md`; this ADR is where the
amended `frame` row lives, and ADR-0011's banner is where a reader of the table is sent.

| verb | MCP | CLI | what it is for | arguments |
| --- | --- | --- | --- | --- |
| `frame` | ✅ | ✅ | what it looks like, at an instant or over a span | `--at` **or** `--from`/`--to`; `--crop`, `--full`, `--png`, `--json`; `--out` on the CLI |

**The counts do not move: nine MCP tools, twelve CLI commands, two resources**, exactly as
ADR-0078 left them. `frame` was already on both surfaces, and a range is arguments rather than
a verb — the distinction ADR-0037 drew when it refused a tenth verb for a *new capability*,
and the one the court and the trial both applied when they chose to extend `frame` rather than
add a sheet verb, on the ground that an MCP schema costs context on every turn while the
question — *what does it look like* — is the one `frame` already answers.

## What this ADR does not decide

- **The names of the two flags ADR-0094 mandated** — the keyframe opt-in and the uniform-infill
  gap ceiling. These are genuine new coinages, they were **not put to this panel**, and this
  ADR deliberately does not invent them. They are the remaining tool-surface work on the map.
  Note that [#407](https://github.com/MBehtemam/Montagent/issues/407) may flip the keyframe
  default before its flag is named, which is an argument for deciding them together and after
  it.
- **The codes and classes** the refusal, the `skipped` entries, the `blind_to` enumeration and
  the not-yet-legal `--crop` refusal carry — [#412](https://github.com/MBehtemam/Montagent/issues/412),
  now additionally constrained by §5 to codes that render as prose.
- **What the per-tile label carries** — [#400](https://github.com/MBehtemam/Montagent/issues/400),
  now load-bearing for this ADR's §7 rather than cosmetic.
- **Whether per-tile crop ships at all, and what it discloses** —
  [#406](https://github.com/MBehtemam/Montagent/issues/406), blocked by
  [#402](https://github.com/MBehtemam/Montagent/issues/402).
- **The four code sites publishing the wrong token figure** —
  [#405](https://github.com/MBehtemam/Montagent/issues/405). §8 corrects the spec; the code is
  still wrong until #405 lands.

## Trade-offs and risks

- **The mode is under-advertised, by choice.** Nothing in the argument list says "there is a
  sheet here". An agent that has not read the tool description will not discover it, and the
  trial agents' own words (`--per-state`) will produce an unknown-argument error on first
  contact. Juror 1's mitigation is adopted: **the error text is where that is fixed, not the
  flag name.** An unknown-argument refusal on `frame` should name the range arguments.
- **Refusing `--full` hard-codes a tier fact.** Recorded in §4 as a revisit condition. The
  alternative — letting it degrade to a no-op — trades a loud, correctable error for a quiet,
  uncorrectable false belief.
- **§7 splits one obligation across two tickets.** The attribution guarantee is now discharged
  jointly by this ADR's provenance list and #400's label. Stated explicitly, but a guarantee
  with two owners is weaker than one with a single owner, and #400 must be read as binding.
- **The panel answered seven questions in one ballot.** Recorded in the jury README as a
  methodological difference from ADR-0094's single-question panel: it permits cross-question
  trading (Juror 1 did so on Q7, explicitly) and buys less depth per question. The four
  unanimous answers are the ones this ADR leans on hardest, and the two contested ones were
  resolved on arguments quoted rather than tallied.
- **Exit 3 for the overflow refusal is inherited, not tested.** ADR-0078's reading 8 settled
  that a `preview` budget hard fail is exit 3 under ADR-0011's own exit-code table, whose
  distinguishing question is *"what the caller does next"*: the document is legal (not 1), the
  file parsed (not 2), Montagent did not break (not 70), and the lever available is a shorter
  range. The sheet's refusal is the same shape and takes the same code. **This was not put to
  the panel** and is recorded as an inheritance so a later reader can challenge it as one.

## Consequences

- `frame` gains `--from`/`--to` on both surfaces, half-open, both required, mutually exclusive
  with `--at`, with the exclusivity enforced in the core verb.
- MCP's `FrameParams.at` becomes `Option<i64>`, matching the core `Ask` and the CLI.
- `--full` with a range is an invocation error; `--crop` with a range is a not-yet-legal
  refusal naming #406; `--png` and `--json` compose unchanged.
- The plain-text answer carries the full ADR-0094 §6 disclosure, plus ADR-0095's served tile
  width and rung, plus this ADR's per-tile provenance list. `--json` carries the same facts
  canonically.
- ADR-0011's caption rule is amended to an attribution rule, and its `2691` figure to a
  two-tier table. Its banner gains this ADR; its own words stand.
- `CONTEXT.md`'s **Verb** entry records that the counts survive this amendment, and gains the
  attribution obligation in the vocabulary.
- #400 becomes load-bearing for ADR-0011 compliance; #412 is constrained to prose-renderable
  codes; #405's ordering instruction is discharged for the spec and still open for the code.
