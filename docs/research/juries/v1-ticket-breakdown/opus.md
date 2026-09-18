# Ballot — the proposed v1 ticket breakdown (#168 → 22 tickets)

Juror: Opus 5 (1M). Ruling alone. All claims below were checked against the working tree at
`73c09f1b` and against the GitHub tracker, not against the brief.

**Facts in the brief I checked and that hold:** 67 ADRs in `docs/adr/` (index rows counted);
`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json` is 155 lines, 14
tracks, 60 elements; `reference/en-halloween-decorating.mp4` exists (10.98 MB, 65.259 s);
#36 is open and unlabelled; ADR-0067 does keep two floors as two separate refusals.
**One arithmetic slip:** #168 has **84** stories, not 83 — 1–83 plus `62a`. Minor, but the
coverage walk in Q4 has to be done over 84 rows or `62a` falls through.

---

## Q1 — Vertical or horizontal? — **ACCEPT WITH MODIFICATION**

The four-way split the brief invites me to attack (8, 9, 13, 14) **survives**. Each of the
four ends at a fact an operator can see: a fixture that fires the check and a fixture that
must not, over `montaget validate`, over both adapters, with the finding's code and inline
numbers asserted. That is a complete path — the spine (parse → check → Finding → JSON →
text → exit code → CLI/MCP) is built once in ticket 1, and after that a check ticket is
narrow-but-complete by construction, not a horizontal cut. The `#168` testing section
mandates a firing/non-firing **pair** per check, which is precisely what makes each of the
four independently demoable. I do not refute the split.

What does not survive:

**1. There is an ownerless prefactor between ticket 1 and the check tickets.** Tickets 7, 8,
9, 13, 14 and 17 all need the same machinery that no ticket claims: a check registry, the
per-check refuse-class declaration ADR-0043 requires ("decided once per check, uniform across
instances"), the `UNCHECKED` accounting story 23 needs, the `LAYOUT` category that must not
gate `render` (story 18, ADR-0041), and the `NOT CHECKED` block's per-check contributions.
Ticket 1 builds the *report*; nothing builds the *engine*. Tickets 7 and 8 can both start the
moment 6 and 3 land, so two parallel agents will each invent it. The skill's own rule — *any
prefactoring should be done first* — is being violated silently.
**Counter-proposal:** move the check registry + refuse-class declaration + UNCHECKED/LAYOUT
accounting into ticket 1 as named deliverables (its demo already needs one check to be real:
make `E-PARSE` plus one trivial header check the registry's first two entries), and add
`1` as a blocker on 7/8/9/13/14 explicitly rather than transitively.

**2. Ticket 18 is not a slice at all.** "Whole-video fixture comparison against the published
MP4" is a test, with no schema, no API and no UI. It is one acceptance criterion of ticket 17
wearing a ticket number. Merge it into 17.

**3. Ticket 2 is honestly horizontal and that is fine.** CI is infrastructure; the rule the
brief quotes is about product slices. It *is* verifiable on its own (the job goes red on a
source-build fallback). Keep it — but see Q4(a), because as drafted it will guard the wrong
thing.

**4. Ticket 1's header-only demo is a real slice, not a stub — with one condition.** It
exercises every layer the project has at that moment and ends in an observable exit code over
two adapters. But it is only a tracer bullet if the demo includes the *negative* arm: a
malformed file producing `E-PARSE` with line/column/byte offset/offending line/caret and
**exit 2**, and a bad invocation producing **exit 3**. Without those it demonstrates the happy
path of a program that has not yet met a document.

---

## Q2 — The blocking edges — **REFUTE**

### Missing edges (a ticket that cannot start with its declared blockers done)

**10 → 6 (measure needs probe). This is the one that will bite first.** ADR-0024 gives
`measure` the fitted-extent output and says so in its own words: the author today must compute
`(s_slack * b_driving) // s_driving` *"against source dimensions they must probe themselves."*
ADR-0024's Consequences: *"raster-source elements additionally return the derived width/height
and the driving axis under fit."* Story 52 is in ticket 10. Ticket 10 is blocked by 3 only.
Ticket 10 therefore cannot deliver story 52 — it has no source dimensions and no ADR-0023
decode/rotate/PAR pipeline. Add `10 → 6`, or move story 52 out of 10 into a ticket after 6.

**11 → 6 (query --at needs probe).** Story 43 requires `query --at` to return *"offset into
source, crop rectangle, ink box and the NOT COVERED region."* The crop rectangle is
source-dimension arithmetic under `fit` (ADR-0013/0015/0023); ADR-0011 makes this explicit with
the `images/06.png` 1536×2720-into-1080×1300 example and says *"none of that is in the
document."* Ticket 11's only path to 6 is nonexistent — 11 → 10 → 3.

**14 → 6 (R-OFF-CANVAS needs the element's rect).** ADR-0044 fires when an element's *rect*
never intersects the frame. For a raster element the rect is the fitted extent, which is
probed. Same for `R-VISUAL-GAP`'s group-scoped coverage (ADR-0018) once a fitted element is
in a group.

**15 → 6/7 (the rasterizer decodes).** Ticket 15 is blocked by 11 only. Nothing in that chain
reaches `probe`, and `frame` cannot draw a video element without the decode path and the
ADR-0023 rounding. Only ticket 17 declares `7`.

The clean fix for all four: **make ticket 6 an early spine ticket.** It is currently blocked by
3, which is itself questionable (see spurious edges) — `probe` takes a media path, not a
project. Re-sequence 6 to sit beside 3 (blocked by 1), and hang 7, 10, 11, 14, 15, 17 off it.

**17 → 22, partially, and the draft has it backwards.** Story 82 (*a missing or unusable
`ffmpeg` reported as exit 70 with the resolved path*) sits in ticket 22, blocked by 17.
But ticket 17 spawns `ffmpeg` and ticket 6 spawns `ffprobe` — both need FFmpeg resolution
before 22 exists. Either split story 82 out of 22 and give FFmpeg resolution to ticket 6
(where the first subprocess is spawned), or accept that 6 and 17 will each build a private
resolver and 22 will be a consolidation. Say which; do not leave it implicit.

**16 → 17 (preview encodes a video).** See the Q4 coverage section — `preview` is a
10-second clip under a 5-second budget (ADR-0021), not a still. It needs the FFmpeg encode
path that ticket 17 builds. Blocking 16 on 15 alone means ticket 16 either builds a second
encoder or ships nothing demoable.

**19 and 20 both need a `slack` primitive that no ticket owns.** ADR-0032 defines slack as
the distance from a boundary to its nearest neighbouring boundary in any track, or to
`duration`. Story 68 (`shift` refuses to change a slack) and story 72 (`compare` reports slack
drift) both consume it; ticket 8's description is overlap/gap/speed/quantization and never
names it. Two independent tickets will implement the same boundary model. Name slack as a
ticket-8 deliverable and let 19 and 20 depend on it — that is what the `19 → 8` and `20 → 8`
edges are actually for, and neither ticket says so.

### Spurious edges

**20 → 11 is false and costs real parallelism.** ADR-0063 is unambiguous: the predicate is
*"two instants that were numerically equal in the caller's supplied ref … if they are no
longer equal in the current file, report it. Exact equality only."* Its three populations are
keyframe `t` against boundaries and against other keyframe `t`s. No resolved value is ever
sampled. ADR-0066's cluster drift is likewise instants-only. Story 75 (text changed, highlight
timing did not) is structural. `compare` needs the document model and the boundary/slack model
— `3` and `8`. Drop `20 → 11`; ticket 20 can then run in parallel with the whole
10/11/12/15 text-and-raster arm.

**6 → 3 is at best weak.** `probe` answers questions about a media file: the quad, dimensions,
alpha, sample rate, channels, in integer milliseconds (ADR-0023, stories 76–78). Stories 20 and
22 are cache policy. None of it reads a project document. `6 → 1` is the honest edge, and
making it so is what lets 6 land early enough to unblock 10/11/14/15.

**5 → 4 is real, but stories 2 and 3 do not need it.** The schema and format-docs MCP
resources are static artifacts available the moment ticket 3 generates the schema. Splitting
them out lets the resources land with 3 and leaves ticket 5 as `create_project` alone.

### 13 → 10 — the author's stated doubt

**The edge survives, but for a reason the author did not give, and the ticket has a
misfiled story.** `13 → 10` is justified by story 30 alone: ADR-0058 makes `R-BOX-SLACK`
height-only at `slack > max(2px, 10%)`, and the computed height is `measure`'s exact-tenths
arithmetic (ADR-0028). That is a hard dependency. `R-CAPTION-PACE`'s grapheme clusters
(ADR-0034) are Unicode segmentation, not text layout, and would not have carried the edge on
their own.

What is wrong is **story 29**. `R-EASE-INERT` (ADR-0052) fires on *"literal exact equality of
author-written `v`, one finding per run of consecutive holds."* It is a keyframe check with no
text in it. It is in ticket 13 (caption and text consequences), whose blockers are 8 and 10.
Move story 29 to ticket 14 (blocked by 11), where every other keyframe-consequence check lives.

### The wide-refactor question

The brief asks whether ticket 3 (document model) or ticket 1 (Finding) should be sequenced
expand–contract. **No — and the framing is wrong.** Expand–contract is for evolving a type
that already has live consumers. Here both types are prefactors with zero consumers, and the
draft already puts them first, which is what the skill asks for. The real risk is different
and the draft does not address it: **both types will be specified against too few consumers
and will then fan out.** Two concrete instances I can already name:

- **ADR-0030 + ADR-0042 make ticket 3's type design non-obvious, and ticket 4 is where it
  explodes.** ADR-0030: a defaultable field's *presence is content*; `fmt` must never add or
  remove one. That forces every defaultable field to be `Option<T>` in the Rust type, not
  `#[serde(default)]` — otherwise omitted and explicit-at-default are indistinguishable after
  a round trip. Worse: ADR-0042 says `fmt` *"proceeds regardless of any error/review/note
  findings on a file that is recognizably a project"* — refusing only on missing
  `tracks`/`fps`/`frame`. An **unknown key is an error finding** (story 13, ADR-0016/0017), so
  `fmt` must reformat a file that the `deny_unknown_fields` model cannot deserialize at all.
  Ticket 4 therefore needs a second, permissive, order-and-presence-preserving parse path — and
  "canonical key order is Rust struct field order" has no answer for a key that is not a field.
  Discovering this at ticket 4 sends an edit back through ticket 3 and every type in it.
  **Counter-proposal:** make "a lossless round-trip of a file containing an unknown key and a
  retired spelling" an explicit acceptance criterion of **ticket 3**, not ticket 4.
- **Ticket 1's `Finding` is being designed with exactly one finding in hand (`E-PARSE`), and
  E-PARSE uses none of its hard fields** — no sibling census (ADR-0058, ADR-0043), no
  `repair` (ADR-0043), no `UNCHECKED` reason (story 23, ADR-0056), no threshold-provenance
  citation (ADR-0061's fenced exception for `R-CAPTION-PACE`, which requires a *binding
  citation* in the finding). Give ticket 1 acceptance criteria drawn from its hardest
  downstream consumers — construct one census-carrying finding, one refuse-class finding and
  one citation-carrying finding as fixtures in ticket 1's own tests — or ticket 13 will
  reshape the type and every ticket between will re-serialize.

---

## Q3 — Granularity — **REFUTE**

### Too large for one fresh context window

**Ticket 3 — the worst offender, and the reason is structural (see the Q4 headline).** "The
full document model" is sized in the brief against two stories (13, 14). The actual surface is
the ADR series: tracks and `layer` (0004), elements and absolute half-open times (0005), the
flat transform with six properties and `{t,v,ease}` keyframes (0012), `clip` (0025), `fit`
vocabulary (0015/0026), `origin`'s nine keywords (0013), styled runs at literal size (0007),
`highlight` windows on the run (0048), the `fonts` table and `fontVendor` attestation
(0002/0057 — the committed fixture carries a top-level `fontVendor` key today), `rect`/
`ellipse` and stroke-as-paint (0014), the ordered `effects` list with `blur`/`shadow`/`mask`
(0040), four scalar colour filters (0049), transitions as their own element type (0059),
keyframable `volume` (0055), project-level `loop` (0062), required unique element `id`
(0019) — **plus** `deny_unknown_fields` everywhere, the generated-vs-committed JSON Schema
drift test, and the retired-spelling error table naming replacements.
**Split into three:** (3a) project header + track + element common fields + time arithmetic +
the schema-generation harness and drift test, demoed on a hand-cut subset of the fixture;
(3b) the visual vocabulary — transform, keyframes, fit/origin/clip, shapes, effects, colour
filters, transitions; (3c) text runs, `highlight`, `fonts`/`fontVendor`, plus the retired-
spelling table and the unknown-key error. The fixture parses clean only at 3c, which is the
right place for that demo.

**Ticket 10** — `parley`/`harfrust`/`icu_segmenter` with `complex-scripts`, `skrifa` scaling,
Montaget's own UAX #14 line partition (ADR-0008 explicitly takes ownership away from
`split('\n')`), half-leading baselines read across *every* run on the line (ADR-0029), exact-
tenths block height (ADR-0028), fitted extents in exact integer arithmetic including the legal
`0` case ADR-0024 forbids clamping, the declared-font-chain-only rule, and the font-swap
census. **Split:** (10a) shaping + per-line metrics + break opportunities + block height
(stories 50, 51, 53); (10b) fitted extents and the driving axis (story 52 — and this half is
the one that needs ticket 6); (10c) font-swap census (story 54).

**Ticket 11** — the keyframe resolver and `query --at` are two things. `--at` alone owes
painter's order, resolved values, source offset, crop rect, ink box and the `NOT COVERED`
region. **Split:** (11a) resolver, demoed through a minimal `query --at` returning resolved
transform values only; (11b) the full `--at` block. Then **merge ticket 12 into 11b** — it is
two flags on a verb that already exists and does not justify a context window of its own.

**Ticket 15 — the largest ticket in the draft by a wide margin, and it is invisible in its
own description.** "The rasterizer, `frame`, and the first falsification test" must draw
*every element type in ticket 3's vocabulary*: image with fit/crop/Ken Burns resampling, video
with decode and seek, text via `skrifa` outlines, `rect`/`ellipse` with stroke-as-paint, the
ordered effects list (`blur`, `shadow`, shape `mask`), four colour filters, `crossfade`
transitions, opacity/rotation/scale compositing — plus JPEG/PNG encode, `--crop`, half-scale
default, the unconditional `query --at` block, and a 500 ms cold budget. **Split by drawing
capability:** (15a) raster + shape + transform compositing and the `frame` surface, demoed
against `reference/frame-intro.png`; (15b) text drawing; (15c) effects, filters and
transitions. The falsification test belongs at the end of 15b, not 15a — the fixture is 22
text elements out of 60.

**Ticket 17** is borderline. `render` is encode + atomic temp-and-rename + `--from --to`
name derivation and the output-collision refusal + the check gate + the post-success review
and `NOT CHECKED` print + the stdout/stderr split + the 60 s-under-2-min budget. Splitting
`--from --to` (stories 56, and the partial-render budget) into its own ticket is defensible;
I would not insist.

**Ticket 1** is also over-scoped as written — four crates, the full `Finding` type, the
JSON-canonical report with the text form generated from it, `NOT CHECKED`, `E-PARSE` with
caret, five exit codes, and *two adapters including an `rmcp` server*. **Split:** (1a)
workspace + CLI adapter + `E-PARSE` + exit codes 0/1/2/3/70; (1b) the `Finding`/report model,
JSON-canonical with generated text form, `NOT CHECKED`, refuse-class `repair`, and the MCP
adapter.

### Too small — merge

- **18 into 17** (it is an acceptance criterion).
- **12 into 11b** (two flags on an existing verb).
- **21** (`timeline`, story 9) is one story and one ADR (0031). Keep it standalone only
  because it is a genuinely independent leaf; it is the one place a small ticket costs nothing.

---

## Q4 — Coverage and the three judgment calls — **REFUTE (coverage); see per-call rulings**

### The story walk (84 rows)

Mechanically, every numbered story including `62a` is claimed by at least one ticket. Union
check: 1–3→5, 4–8→4, 9→21, 10–12→8, 13–14→3, 15–18→9, 19/21/23/24→7, 20/22/76–78→6, 25/31–33
→14, 26–30→13, 34→8, 35–42→1, 43→11, 44–45→12, 46–49→15, 50–54→10, 55–60→17, 61–63+62a→16,
64–69→19, 70→4, 71–75→20, 79–82→22, 83→2. No story is orphaned. So the *story* coverage is
clean, and that is exactly the problem.

**Dropped (not a story, but required by an accepted ADR and owned by no ticket):**

- **`fonts list` / `fonts vendor` (ADR-0057).** A designed tool with a three-bucket licence
  gate, a hard refuse with no override, and path-keyed attestation. The committed fixture
  carries the resulting `fontVendor` key today. It has no story in #168 and no ticket here.
  Either it is out of v1 — in which case ticket 3 still has to model `fontVendor` and the
  breakdown must *record* the exclusion, as #168 instructs for open questions — or it is a
  ticket.
- **The whole unstoried format vocabulary**: effects (0040), colour filters (0049),
  transitions (0059), `volume` (0055), `loop` (0062), `rect`/`ellipse` (0014), `highlight`
  (0048), required unique `id` (0019). Owned in practice by tickets 3 and 15, claimed by
  neither.
- **ADR-0010's two-arm `rust-rasterizer` oracle.** ADR-0010 calls the golden-frame guard
  *"not optional"* and #36's *Related, same shape* section asks for it to be run on every
  rasterizer bump. Ticket 2 claims #36; #36 contains this clause; ticket 2's description does
  not. Assign it explicitly to 2 or to 15.

**Implicitly assumed by a ticket that does not claim it:**

- **Story 70** (every write tool re-emits canonically and writes atomically) is in ticket 4,
  but the write tools are `create_project` (5) and `shift` (19). At ticket 4 it can only be
  demonstrated for `fmt`. It is a shared invariant, and 5 and 19 each need an acceptance
  criterion re-asserting it.
- **The write-tool-returns-findings invariant** — ADR-0011's load-bearing rule, restated in
  #168's Solution — **has no story number at all**. Ticket 5 claims it in prose. Ticket 19
  (`shift`) must satisfy it too, and at ticket 19 only ticket 8's checks exist, so `shift`
  will return a knowingly partial finding set. Say so in the ticket rather than discovering it.
- **Story 53** ("the renderer opens nothing outside the declared font chain") is filed under
  ticket 10 (`measure`). It is a *renderer* property; its enforcement point is ticket 15.
- **Story 42** ("every tool fails identically on a malformed file") is provable at ticket 1
  for `validate` only. Every subsequent verb ticket inherits it as an unstated criterion.
- **Story 81** ("no unsolicited network calls ever") is in ticket 22, but ticket 6 introduces
  the only network code in the product (remote URL probe, ADR-0056). The invariant needs a
  test from ticket 6 onward.
- **Stories 49 and 60** are performance budgets (`frame` < 500 ms cold; 60 s render < 2 min).
  Neither has a CI home. Ticket 2 is the CI ticket and claims only story 83.

**Double-owned:** none strictly, but stories **38 and 39** are split by design — see (c).

### (a) CI at ticket 2 carrying an uncalled `skia-safe` dependency — **ACCEPT WITH MODIFICATION**

The core objection I expected to make does not land: `cargo` builds a declared dependency
whether or not anything calls it, so `skia-safe`'s build script runs and the prebuilt fetch is
genuinely exercised. The canary is real from day one. The author's reasoning survives.

**But as drafted it will guard the wrong key.** ADR-0010: *"`skia-safe` fetches binaries keyed
on the exact sorted feature set … The feature set is pinned in one place and changed only
deliberately; `svg` and `skottie` both imply `textlayout` and would move the key."* #36's
*Done when* says the same. If ticket 2 declares `skia-safe` with whatever features are
convenient and ticket 15c later adds one for `blur`/`shadow`/`mask` (ADR-0040), the canary
spent thirteen tickets guarding a key the product does not use.
**Modifications:** (i) ticket 2 pins the literal key `jpegd-jpege-pdf` — CPU-only, no
`ganesh`, no `gl` — in exactly one place and asserts it, and the acceptance criterion is *the
key*, not "the build succeeded"; (ii) tickets 15a–c carry an explicit criterion that the
feature set does not change, and a deliberate change is its own ticket that re-verifies all
six targets; (iii) **the canary and the test-suite CI are two jobs, not one** — #36 requires
an **empty `CARGO_HOME` and empty target directory** on a **schedule** ("the failure arrives
from upstream, not from a commit"), which a per-PR cached job structurally cannot detect.
Ticket 2's one-line description conflates them.

### (b) Falsification at 15 (frames) rather than 18 (whole video) — **ACCEPT WITH MODIFICATION**, and one hard problem the author has not seen

The reasoning is right and the material is already in the repo:
`reference/frame-intro.png` and `reference/frame-05-at-11s.png` are two frames extracted from
the published MP4. That is the correct oracle, and it is reachable with `frame` alone.

**Two modifications, the second serious:**

1. **The goldens must be those two frames, not frames we generate.** The brief says "golden
   frames by SSIM with a stated threshold." A golden we render ourselves and commit is
   self-confirming — it falsifies regressions, never the format. Only
   `reference/frame-*.png` falsifies. State which is which in the ticket, because the same
   word covers both and the distinction is the whole judgment call.

2. **The fixture no longer renders in the typeface the reference was set in.** The reference
   README states: *"Typography is `SF Pro Rounded`, bold throughout."* ADR-0057 recorded that
   the declared `fonts/SFProRounded-Bold.ttf` did not exist and is not redistributable; #143
   re-vendored a replacement, and the committed fixture today declares
   `fonts/OpenRunde-Bold.otf`. **The falsification test therefore compares a render set in
   Open Runde against a reference set in SF Pro Rounded.** Twenty-two of the fixture's sixty
   elements are text. No SSIM threshold simultaneously passes that text and remains sensitive
   enough to catch a real defect. The test as specified either fails on ticket 15 for a reason
   that is not a bug, or is tuned so loose it falsifies nothing — and tuning it loose is
   exactly the "shipped a render that completed, looked plausible and was wrong" failure
   ADR-0010 says this project has already had twice.
   **Counter-proposal:** ticket 15's falsification arm compares **masked regions** — the
   image card, the Ken Burns move, the drawn flag rectangles, the cream/navy panels, the
   badge — at a tight threshold, and reports text regions as a separate, explicitly
   non-gating measurement with the font substitution named in the ticket. Then add the
   ticket nobody has written: **re-derive the fixture's 22 hand-tuned text sizes, box heights
   and positions under Open Runde** (#168 flags this as unverified and cites #143). Until
   that exists, `R-BOX-SLACK` and the fit-deviation error are likely to fire on the fixture —
   and #168's own rule is that *"a check that fires on it is wrong unless an ADR says
   otherwise."* Ticket 13 will hit this wall with no owner for it.

### (c) Refuse-class folded into ticket 1 — **ACCEPT WITH MODIFICATION**

Folding the *mechanism* into ticket 1 is right: ADR-0043 is explicit that this is *"a general
property of any check, decided once, by whoever authors the check, at the moment the check is
written — not a special case of retired-key handling."* A separate refuse-class ticket would
have to reach into every check ticket to do its job, which is the horizontal cut the skill
forbids. The author's call survives.

**Three modifications:**

1. **At ticket 1 the rule is unfalsifiable** — the only finding in existence is `E-PARSE`,
   which is not refuse-class. Ticket 1 must construct at least one refuse-class finding as a
   test fixture (the `gravity` retired-spelling case from
   `docs/research/juries/format-versioning/experiment-gravity-fork/` is written and committed),
   or the mechanism ships untested to ticket 3.
2. **ADR-0043's uniformity rule needs a completeness test that no ticket owns.** *"If any
   instance a check can match is capable of being load-bearing, the check emits
   `repair: "none"` for every instance."* Add a test that every registered `error`-class code
   has a declared classification and that no check emits both shapes — and give it a home
   (the check registry, i.e. ticket 1 under my Q1 counter-proposal).
3. **Story 38 is being quietly under-delivered.** Its second half — *"with a sibling census
   where one exists"* — is a real output shape (ADR-0058's `R-BOX-SLACK` census, ADR-0043's
   refuse-class census). Ticket 1 builds the field; ticket 13 is the first ticket that fills
   it, and ticket 13 does not claim story 38. Add the census-producing criterion to 13 and 14.

---

## The single most important thing the author has got wrong

**The breakdown is sized against #168's story list, but #168 itself says the ADR series is the
specification and that where the two disagree, the ADR wins.** Every ticket in the draft is
labelled with story numbers, and the story-coverage walk comes out clean — which is precisely
what hides the problem. The format's largest bodies of work have **no story at all**: the
effect model and its ordered `effects` list (ADR-0040), the four colour filters (ADR-0049),
transitions as their own element type (ADR-0059), keyframable `volume` (ADR-0055), `loop`
(ADR-0062), `rect`/`ellipse` and stroke-as-paint (ADR-0014), `highlight` windows (ADR-0048),
required unique `id`s (ADR-0019), and `fonts vendor`'s licence gate (ADR-0057). All of it
lands, unnamed and unestimated, inside tickets 3 and 15 — the two tickets the brief already
suspects are too big, and whose descriptions give no hint of it. Ticket 3 looks like two
stories and is nine ADRs of vocabulary; ticket 15 looks like four stories and is the entire
drawing surface of the product.

That single error explains most of the rest: it is why ticket 3 and ticket 15 cannot fit a
context window, why the `fonts vendor` scope question is neither shipped nor recorded, and why
the media-facts chain (ticket 6) was never wired into the derivation chain — because the
stories that need it (52, 43, 32) read as text and geometry questions, and only the ADRs say
they are probe questions. **Re-derive tickets 3 and 15 from `docs/adr/README.md` rather than
from the story list, and re-check every other ticket's blockers against the ADRs its content
actually cites.**
