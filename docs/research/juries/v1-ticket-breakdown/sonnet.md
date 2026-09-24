# Ballot — Sonnet 5

Verified against: `gh issue view 168` (full text), `docs/adr/README.md`, `CONTEXT.md`,
`docs/agents/domain.md`, `docs/adr/0043-refuse-class-findings-...md`, `gh issue view 36`,
`gh issue view 178`, and the fixture/prototype tree under `docs/research/` and `fixtures/`.
No `/to-tickets` skill file exists in this repo or in `~/.claude/skills`; I treated the
brief's restated rules as the governing text since it is the only copy available.

---

## Q1 — Vertical or horizontal?

**Verdict: ACCEPT WITH MODIFICATION**

Ticket 1's "header-only project" demo is a real, if shallow, tracer bullet — it is the
only ticket that has no prior software to lean on, and it genuinely closes the loop
(parse → Finding type → JSON-canonical report → text-from-JSON → CLI adapter → MCP
adapter) end to end. But it is thinner than it looks in one specific way: to validate
even a header-only project, `Project.tracks` must already be typed as `Vec<Track>`,
which means ticket 1 must define at least a stub `Track`/`Element` shape to make the
struct compile and `deny_unknown_fields` deserialize — work the draft assigns entirely
to ticket 3 ("the full document model"). Either ticket 1 is silently doing some of
ticket 3's prefactoring, or ticket 3 is not truly greenfield when it starts. This
doesn't break verticality, but it means ticket 1's "none" blocking is optimistic and
ticket 3 is not the clean first-touch of the type it's described as (see Q2).

On tickets 8, 9, 13, 14 ("four horizontal cuts through one check engine"): I don't
think this is the right frame for 8, 9 and 14, but it **is** the right frame for 13,
and 13 has a real bug hiding under it.

- Tickets 8, 9 and 14 each land a self-contained rule family with its own committed
  fixture pair (a file where the check fires, a file where it must not — the spec's
  own testing rule) and each is independently demoable via `validate`'s existing report
  surface. That's what a thin vertical increment through an already-built pipe looks
  like once the pipe exists; it is not the same failure mode as "build the whole schema
  layer, then the whole API layer." ACCEPT for 8, 9, 14.
- Ticket 13 ("Caption and text consequence checks") claims stories 26–30. Story 29 is
  **`R-EASE-INERT`** — "a run of held keyframes carrying a semantically inert `ease`."
  That is a keyframe/animation check. It has nothing to do with captions or text, needs
  no grapheme-cluster measurement, and needs no font metrics. It is misfiled. This is
  the concrete reason ticket 13 reads as "one check engine wearing four costumes" where
  8/9/14 don't: it is bundling two genuinely unrelated rule families (caption pacing
  and stray keyframe animation) under one label, not because they share infrastructure
  but because the author ran out of tickets to put them in. See Q2 and Q4 for the
  consequences.

No ticket in the set is a true horizontal layer-slice in the bad sense ("all the
schema, nothing else" or "all the CLI wiring, nothing else") — the closest thing is
ticket 3, which I address as a wide-refactor candidate in Q2 rather than a verticality
failure.

---

## Q2 — The blocking edges

**Verdict: ACCEPT WITH MODIFICATION**

**13 → 10 (the author's stated low-confidence edge): mostly REFUTE.** Walking the five
stories ticket 13 claims:
- Story 26 (`R-CAPTION-PACE`, cps over grapheme clusters) — needs Unicode grapheme
  segmentation, not font layout. No dependency on `measure`/`montagent-text`.
- Story 27 (`R-CAPTION-REPEAT-DURATION`) — string equality of run text + duration
  comparison. No text engine.
- Story 28 (`R-CAPTION-NO-AUDIO`, `R-CAPTION-MIN-DURATION`) — pure temporal/track
  overlap logic, ticket 8's domain.
- Story 29 (`R-EASE-INERT`) — doesn't belong in this ticket at all (Q1). What it
  actually needs is the keyframe resolver, ticket 11, not `measure`.
- Story 30 (`R-BOX-SLACK`, "declared text box exceeds its computed height") — this is
  the **only** story in the bundle that genuinely needs ticket 10's exact-arithmetic
  block-height derivation (ADR-0028).

So the 13→10 edge is real for exactly one of the five stories bundled under it, and
spurious for the rest — meaning as currently scoped, ticket 13 needlessly serializes
four independent, cheaper checks behind the text-engine ticket. Counter-proposal below.

**Missing/spurious edges elsewhere:**
- **Ticket 6 (`probe`) → 3 is likely spurious.** `probe` wraps `ffprobe` and returns
  facts about a file path (stories 76–78 are explicitly framed "as a human author,"
  operating on a path, not a project). Nothing in its scope reads `Track`/`Element`.
  The only thing it borrows from ticket 3 is the convention "times are `i64`
  milliseconds," which is a trivial, stable type that doesn't need the full document
  model finished. I'd cut this edge to `probe: blocked by 1` and let it run in
  parallel with tickets 3–5.
- **Ticket 2 (CI canary) → 1 is stricter than necessary.** The canary only needs a
  `Cargo.toml` workspace with `montagent-render` depending on `skia-safe` — a few lines
  of ticket 1's scope, not the Finding type, the adapters, or the report format. Low
  cost to leave as-is, but worth naming: CI could start the moment the workspace
  skeleton exists, days before ticket 1 as a whole is "done."
- **Ticket 5 (`create_project`) → 4: real, correctly kept.** Every write tool must
  re-emit the file in canonical convention (story 70), so `create_project`'s first
  write already needs ticket 4's canonical-writer machinery. Confirmed, not spurious.

**Wide-refactor exception — ACCEPT that it applies, and the draft under-uses it.**
The brief is right to flag ticket 3. The document model (`Track`/`Element`/`Run`/
`Keyframe`, `deny_unknown_fields` at every level, the Rust-struct-order-is-canonical-
order decision, the generated-vs-committed JSON Schema drift test) is exactly the kind
of type every other ticket (4 through 22) imports transitively and pattern-matches
against. Nothing in this repository has ever parsed a project file with a real
renderer, prober, or text engine attached to it — issue #168 says so explicitly
("None of it exists as software... never parsed, never checked, never rendered"). Bet
that this type is right on the first pass, and any correction discovered once ticket 6
(probe) or ticket 10 (measure) actually exercises it against real media/fonts fans out
across every already-built check ticket that reads its fields, exactly the blast-radius
the wide-refactor rule is for. I'd split ticket 3 into an **expand** phase (minimal
document model sufficient for tickets 4, 8, 9 — the structural checks that only need
shape, not media or text) and a **contract** phase (closed-schema retired-spelling
errors + generated-schema-with-drift-test), sequenced so the contract lands *after*
probe (6) and measure (10) have both touched real files once, rather than before either.
This is the same shape the brief already applies to ticket 1's Finding type implicitly
(the `repair` field exists in ticket 1 but isn't populated by any real check until much
later) — I'd apply it explicitly to ticket 3 as well.

---

## Q3 — Granularity

**Verdict: ACCEPT WITH MODIFICATION**

- **Ticket 3 — too large, should split.** It currently bundles: multiple element type
  variants (image/video/audio/text/shape × rect+ellipse/transition — at least 6 element
  kinds per ADR-0007/0014/0059), `deny_unknown_fields` at every nested level, 8 named
  retired-spelling error mappings (`gravity`, `box`, `align`-on-non-text, `anchor`-as-
  string, `center-center`, `#RRGGBBFF`, `bold`, `none`/`fill` as `fit` — enumerated in
  `CONTEXT.md`'s "Rejected terms"), and schema-generation-with-drift-test tooling. Split
  on: **3a** — core `Project`/`Track`/element-envelope shape + `deny_unknown_fields` +
  minimal image/video/audio/text elements, unblocking 4/8/9 early; **3b** — shapes
  (`rect`/`ellipse`, stroke) + transitions; **3c** — the 8 retired-spelling error
  mappings + JSON-Schema-generation-with-drift-test (naturally the "contract" half of
  the expand–contract sequencing argued in Q2).
- **Ticket 10 — too large, should split.** Bundles standing up an entirely new crate
  (`montagent-text` over `parley`/`skrifa`) with a real UAX #14 line-partition algorithm
  (story 51 — Montagent, not the renderer, owns line breaking per ADR-0008, which is
  substantially more than a wrapper), exact-arithmetic block-height derivation (ADR-0028),
  fitted-extent-plus-driving-axis (story 52), closed-font-chain enforcement (story 53),
  and font-swap census (story 54). Split on: **10a** — crate integration + advance/
  ascent/descent/line-count/baseline_y/block-height (stories 50, 53); **10b** — UAX #14
  break opportunities (story 51, a genuinely separate algorithm); **10c** — fitted
  extent/driving axis + font-swap census (52, 54).
- **Ticket 11 — roughly right-sized, but under-attributed.** It claims only "story 43,"
  but story 43's own text packs in painter's order, resolved keyframe values, source
  offset, crop rectangle, **ink box**, and the `NOT COVERED` region — that's most of the
  runtime query engine in one bullet. I'd leave it as one ticket (its `blocked by: 10`
  is correctly justified — ink box for text elements needs `measure`'s metrics) but flag
  that its story attribution undersells its actual weight; the draft should say so
  explicitly rather than let "story 43" read as a one-liner.
- **Ticket 15 — badly oversized; the worst offender in the set.** Nothing before this
  ticket has rasterized a single pixel. As scoped it must deliver, in one context
  window: the full transform pipeline (x/y/origin/scale/rotation/opacity/clip), effects
  (`blur`/`shadow`/`mask` — ADR-0040/0049), shape rendering with stroke/fill, text
  rendering via `montagent-text`'s output, PAR/rotation-corrected video frame decode
  (ADR-0023), the `frame` CLI/MCP surface with crop and the 500 ms budget, *and* the
  SSIM golden-frame harness with a stated threshold. Split: **15a** — core transform
  pipeline for image/shape elements only (first pixel on screen, no text/video/effects);
  **15b** — text rendering integration; **15c** — video frame decode + PAR/rotation;
  **15d** — effects; **15e** — `frame` surface + 500 ms budget; **15f** — golden-frame
  SSIM harness + the falsification test against the fixture. Six tickets' worth of work
  is currently one.
- **Ticket 17 — right-sized only if 15 is split as above.** If ticket 15 delivers a
  working per-frame renderer, "sequence frames through FFmpeg, refuse on `error`, write
  atomically" is a reasonable single ticket. But see Q4: neither 15 nor 17 names audio
  mixing anywhere, and `render` cannot be correct without it.
- **Merge candidate: ticket 21.** "`timeline`" is one CLI-only story (9) with a trivial
  dependency (the parsed document model, ticket 3) and no interaction with any other
  ticket's output. Giving it a full ticket's worth of tracer-bullet ceremony (its own
  demo, its own PR, its own "single fresh context window" framing) is wasted structure
  for a read-only pretty-printer. Fold it into ticket 4 (the other CLI-only, read-mostly
  ticket) or ticket 9.

---

## Q4 — Coverage and the three judgment calls

**Verdict: ACCEPT WITH MODIFICATION**

**Coverage walk.** Every numbered story 1–83 (and 62a) is claimed by the numeric
ranges printed against some ticket — I checked the full union against the story list
in #168 and found no story number entirely absent from the ranges as printed. But two
things the numeric coverage hides:

- **Double-owned / misplaced: story 29 (`R-EASE-INERT`).** Claimed by ticket 13
  ("caption and text consequence checks") where it does not belong (Q1, Q2). It is not
  double-claimed by any other ticket, so the net effect is not duplication but
  **misattribution** — the story is "covered" on paper but assigned the wrong blocking
  edge (8, 10 instead of 11), which would send whoever picks up ticket 13 looking for
  font metrics to implement a keyframe check.
- **Implicitly assumed, claimed by no ticket: audio mixing into `render`'s output.**
  Volume is a keyframable, flat field on `audio`/`video` elements (ADR-0055) and the
  fixture is a published video with narration and captions checked against audio
  presence (`R-CAPTION-NO-AUDIO`, story 28). But no numbered story among the 83 says
  "render mixes/encodes the project's audio elements into the output," and neither
  ticket 15 ("the rasterizer, `frame`") nor ticket 17 ("`render` end-to-end," stories
  55–60, all about write-atomicity, naming, refusal, progress reporting) names audio
  encoding as in-scope work. `frame` is legitimately silent on audio — it's a still
  image — but `render` cannot be, and nothing currently owns it. This is a real gap in
  #168 itself, not only in the ticket breakdown, but the breakdown compounds it by not
  flagging the gap the way it flags the three explicit judgment calls.
- I did **not** find a story dropped outright (claimed by zero tickets and not
  implicitly required by a claimed one), beyond the audio case above.

**Judgment call (a) — CI at ticket 2 with an uncalled `skia-safe` dependency:
ACCEPT.** Issue #36 and `gh issue view 36` confirm this was independently named "the
single most likely regret path by four independent reviewers on four different
models" in #7, with a stated revisit trigger toward `tiny-skia`. Landing the canary
before any rasterizing code exists, rather than at ticket 15 where the draft itself
notes it would be "worthless," is well-grounded. Minor: ticket 2's `blocked by: 1` is
stricter than needed (Q2) — it only needs the workspace skeleton, not the whole of
ticket 1 — but that costs nothing since ticket 1 is presumably `git init`-adjacent
early work either way.

**Judgment call (b) — falsification test at 15 (frames) rather than 18 (whole video):
ACCEPT.** Issue #168 closes on exactly this argument ("that comparison should happen
early, not at the end... the only test in this spec capable of falsifying the format
itself"), and comparing rasterized frames at fixed timestamps against frames pulled
from the reference MP4 via `ffprobe`/`ffmpeg` genuinely needs only `frame`, not the
video encoder. The reasoning survives attack. It does mean that once ticket 15 is
split (Q3), the SSIM harness becomes its own sub-ticket (15f) rather than an
afterthought bolted onto a rasterizer ticket — but the placement decision itself holds.

**Judgment call (c) — refuse-class findings (38–39) folded into ticket 1, each check
ticket self-classifies: ACCEPT.** ADR-0043 states the decision is "a general property
of any check, decided once, by whoever authors the check, at the moment the check is
written — not a special case." That is a direct textual argument *against* a dedicated
refuse-class ticket: centralizing the classification decision would contradict the ADR
that defines it. Ticket 1 correctly owns only the mechanism (the `repair` field's two
shapes and the sibling-census field on `Finding`), and policy is correctly left
distributed. No defect found here.

---

## Most important thing the author got wrong

**Story 29 (`R-EASE-INERT`) is misfiled into ticket 13 as a "caption and text"
check, when it is purely a held-keyframe/animation check with a real dependency on the
keyframe resolver (ticket 11), not the text engine (ticket 10).** This single error is
the reason the author's own flagged uncertainty about 13→10 is justified — but the
author framed it as "am I unsure whether caption checks need measure," when the actual
finding is sharper: *four of the five stories in ticket 13 don't need ticket 10 at all,
and the fifth story shouldn't be in ticket 13 in the first place.* It's a small, single-
story bug, but it's the kind that doesn't show up in a numeric coverage check (29 *is*
claimed by a ticket, so a naive "are all 83 stories covered" audit passes clean) and
would only surface once someone actually opened ticket 13 looking for font-metrics work
and found a keyframe-holding-pattern check waiting for them instead. If the same
grep-level care that assembled the story ranges was applied without reading what each
story actually says, the other 21 tickets deserve a second pass for the same failure
mode before this breakdown is trusted.
