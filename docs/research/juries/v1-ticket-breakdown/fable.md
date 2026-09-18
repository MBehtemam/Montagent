# Ballot — Fable — the v1 ticket breakdown

Verified against: `gh issue view 168` (body, no comments), `docs/adr/README.md` (67 ADRs + index),
ADR-0006/0007/0010/0011/0021/0032/0040/0041/0043/0055/0057/0058/0059/0063/0067, `CONTEXT.md`,
the committed fixture (`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`,
155 lines, 14 tracks, 60 elements — the brief's counts are right), #36, #143, #178, the
`to-tickets` SKILL.md (the brief quotes its rules accurately), and
`docs/research/prototypes/rust-rasterizer/Cargo.toml`.

Two facts the brief does not state and the draft does not know, both load-bearing below:

- **The committed fixture does not parse under the draft's ticket 3.** `handle-logo` carries a
  bare `"mask": "circle"` key. ADR-0040 defines effects as `effects: [{name, ...params}]` with
  `mask{shape: ...}` as a member, then says in its own Consequences that `mask:"circle"` on
  `handle-logo` "becomes a valid declaration ... no migration needed". Those two sentences
  cannot both be implemented under `deny_unknown_fields`: either the model admits a second
  spelling (the thing `center-center` and `#RRGGBBFF` were retired for) or the fixture must
  migrate. The fixture's last content migrations were "current with ADRs 0001-0012", the
  gravity retirement, and the font re-vendor — nothing after ADR-0012 has touched it.
- **`fonts vendor` / `fonts list` are CLI tools in ADR-0057** ("`fonts vendor <path> [--licence]`
  is the whole surface"), `validate` gains a font-attestation `error` and an orphaned-attestation
  `note` there, and ADR-0007 adds glyph-coverage (`error`), a font census, a grapheme-cluster
  check, an invisible-character census and a mixed-normalization finding. #168 has stories for
  none of these; its own preamble says the ADR wins where they disagree. The breakdown inherits
  the drop.

---

## Q1 — Vertical or horizontal?  **ACCEPT WITH MODIFICATION**

Most tickets survive the test: a `validate` check with a fire/must-not-fire fixture pair is a
complete path (document → engine → report → adapters → test) and is demoable by running the
binary on the pair. The vertical-slice rule does not say "one check per ticket is horizontal";
it says a ticket may not stop at one layer. 8, 9, 13, 14 do not stop at one layer.

**Ticket 1 is a real slice, not a stub — but it claims stories it cannot demonstrate.** Parsing a
header-only project forces the top-level struct (`frame`, `fps`, `background`, `duration`,
`output`, `fonts`, `fontVendor`, `tracks: []`) — that is the first row of the model, not a mock of
it. `E-PARSE` with caret, exit codes 0/2/3/70, JSON-canonical/text-generated, both adapters: all
real and all demoable on an empty `tracks`. What is *not* demoable on a header-only file:
sibling census (no siblings), `repair` (no check that can be refuse-class exists until retired
spellings land in 3), and exit 1 (no `error`-producing check exists yet). Ticket 1 should own
stories 35, 36, 37, 40, 41, 42 and the `Finding` *type*; the acceptance for 38–39 belongs to the
first ticket that emits a refuse-class finding, which is `gravity` in ticket 3 (ADR-0043
Consequences names it as the first instance).

**8, 9, 13, 14 are four vertical cuts, not four horizontal ones — but two of them are cut on the
wrong line.**

- 8 and 9 are fine as pairs of fixtures per story.
- 13 mixes checks with three different data needs: `R-BOX-SLACK` (30) is pure arithmetic on
  `size`/`line_height`/`runs`/`height` (ADR-0058: "with no I/O"); `R-CAPTION-PACE` /
  `-REPEAT-DURATION` / `-NO-AUDIO` / `-MIN-DURATION` (26–28) are "no I/O" per ADR-0034/0054;
  `R-EASE-INERT` (29) is a keyframe-list read. None of them touches the font engine. See Q2.
- 14 mixes `R-VISUAL-GAP` (31 — declared rects and `group`, no motion) with three motion checks
  (25, 32, 33) that genuinely need the resolver. 31 belongs with 9.

**Tickets that are not demoable on their own as written:**

| ticket | why | what makes it one |
| --- | --- | --- |
| 3 | its demo ("the committed fixture round-trips") fails on `handle-logo`'s bare `mask` | resolve the ADR-0040 self-contradiction (an ADR amendment, not a ticket decision), then migrate the fixture to `effects:[{"name":"mask","shape":"circle"}]` inside 3 and make `mask` as a bare key a retired spelling naming `effects` |
| 6 | `probe` wraps `ffprobe`; resolving `ffmpeg`/`ffprobe` from `PATH` and the exit-70 failure (story 82) is owned by 22, which is blocked by 17 | move 80 and 82 into 6 (see Q2) |
| 10 | story 53 ("the renderer opens nothing outside the font chain") is a renderer property, only observable at 15; story 52 (fitted extents) is image/video `fit` arithmetic (ADR-0013), not text | 53 → 15; 52 → 7 |
| 16 | `preview` is a span of video in every preview ADR (ADR-0021 "a 10 s preview"; the result gains a resolution-tier field) — it needs the encode pipeline, which is 17 | blocked by 17, not 15 |
| 18 | a whole-video comparison needs audio to compare — the fixture has 20 audio elements, four at `speed: 0.645` — and no ticket owns audio mixing | see Q4 |

Everything else is demoable at its declared boundary.

---

## Q2 — The blocking edges.  **REFUTE**

The graph was drawn from the verb table, not from the data each verb consumes. Working from
the data yields the following.

### Missing edges (a ticket could not start with its declared blockers done)

1. **6 needs FFmpeg resolution (stories 80, 82), which sits in 22 ← 17 ← 7 ← 6.** That is a
   dependency inversion: `probe` cannot even fail correctly without the mechanism the last
   ticket in the chain owns. Fix: 80 and 82 move to 6 (they are three lines of `which` plus one
   exit-70 finding); 22 keeps 79 and 81.
2. **15 needs 6.** `frame` on an `image` runs ADR-0023's source-dimension pipeline (EXIF
   rotation) and `frame` on a `video` decodes through the FFmpeg subprocess. Neither is in 11.
   The fixture has zero `video` elements, which is exactly why this edge is easy to miss and
   exactly why ADR-0003 says the fixture is never evidence a capability is unneeded.
3. **16 needs 17** (above). As drawn, the proxy ladder is blocked on a tool that produces one
   still.
4. **11 needs 9, or 9's layer/anchor *resolution* must be prefactored out of 9.** Painter's order
   in `query --at` (43) needs anchors resolved to integers (ADR-0019, one hop). 9 owns the
   *checks* on anchors; nobody owns the resolver. Same shape for **11 → 6**: "offset into
   source" under `overrun: hold|loop` needs the probed source duration (ADR-0011's own table
   marks it "hard — needs ffprobe").
5. **19 and 20 both need slack (ADR-0032) and nobody computes it.** 8 is "overlap, gap, speed,
   quantization" — slack is not a gap (CONTEXT.md: slack "additionally names the cross-track
   case a gap can't reach"). Either 8 owns it explicitly or 19 and 20 implement it twice.
6. **4 and 9 implement one predicate twice.** ADR-0041: canonical order "is a requirement
   `validate` and `fmt --check` both verify." 4 owns `fmt --check` (5), 9 owns `LAYOUT` (18),
   neither blocks the other. Prefactor: the order predicate lands once, in 3 (it is struct field
   order), and both consume it.

### Spurious edges (serialization that costs parallelism for nothing)

1. **13 → 10.** The author's least-confident edge, and it is wrong. ADR-0058 states
   `R-BOX-SLACK` is computed "with no I/O"; ADR-0034/0054 say the same of all four caption
   checks; ADR-0028's block height is `(size × line_height×10 × lines + 9) // 10` — integer
   arithmetic on the document. The only text check that needs the font engine is the *width*
   overflow term, which ADR-0014 parks as `UNCHECKED` pending `measure` and which no story in
   13 claims. **13 → 3 only.** (13 → 8 is also unneeded: `R-CAPTION-NO-AUDIO` is a time-range
   intersection, not the overlap check.)
2. **12 → 11.** The cut list (44) is the set of intervals over which the *presence* set is
   constant — element boundaries only. `--where --census` (45) is a document query. Neither
   resolves a keyframe. **12 → 3.** This is a big parallelism win: 12 is the `jq`-replacement
   half of `query` and can ship while 10/11 are still in flight.
3. **20 → 11.** ADR-0063's predicate is exact equality of keyframe *`t` values* between two
   files; ADR-0066 is cluster membership of boundary instants; 72 is slack drift; 75 is a
   `runs[].text` vs `highlight` comparison. None of it interpolates. **20 → 3 (+ whichever
   ticket owns slack).**
4. **6 → 3.** `probe <file>` reads media, not a project. **6 → 1.** (Story 78's `par` is read
   off the element — that half is `validate`'s and belongs in 7.)
5. **2 → 1 is defensible but not necessary.** `docs/research/prototypes/rust-rasterizer/Cargo.toml`
   already depends on `skia-safe = "0.153.2"`; a cold-cache matrix build of that crate is a
   working canary today with no blocker. The cost of keeping 2 → 1 is that #36's "pinned in
   exactly one place" is honoured from the start. I would keep the edge and note the option.

### Corrected graph (only changed rows)

```
2  ← 1            (unchanged; alternative: none, targeting the prototype crate)
6  ← 1            (was 3)   + owns 80, 82
7  ← 3, 6         (unchanged) + owns 52, 78
11 ← 9, 10, 6     (was 10)
12 ← 3            (was 11)
13 ← 3            (was 8, 10)
14 ← 11           (unchanged; 31 moves to 9)
15 ← 11, 6        (was 11)   + owns 53
16 ← 17           (was 15)
17 ← 7, 8, 9, 15  (unchanged; see Q3 for the split)
19 ← 4, 8         (unchanged, with 8 owning slack)
20 ← 3, 8         (was 8, 11)
22 ← 17           (unchanged; loses 80, 82)
```

### The wide-refactor question

**The exception does not apply, and invoking it would be a mistake.** Expand–contract exists
for a mechanical change with thousands of existing call sites; on day one there are zero. What
the author is actually worried about is a different risk — *the type is wrong and every later
ticket pays* — and expand–contract does not address it; it only makes a wrong type survive
longer beside a right one.

Two different answers for the two types:

- **`Finding` (ticket 1) is specified, not discovered.** ADR-0006 (code, severity, location,
  every number inline, census, JSON canonical), ADR-0043 (`repair` on `error` only, orthogonal to
  severity), ADR-0041 (`LAYOUT` as a fourth category), ADR-0011 (`E-*` for invocation errors,
  `UNCHECKED` with structured reason). That is enough to get the type right in 1. The genuine
  risk is per-code *field sets* — ADR-0006 says each code "declare[s] its fields and render[s]
  through a template". If 1 ships `Finding` with a free-form `numbers: Map<String, Value>` bag,
  every later ticket is fine; if it ships a closed enum of codes, every check ticket edits ticket
  1's file. The mitigation is a design constraint on 1, not a sequencing rule.
- **The document model (ticket 3) is the one that fans out — through `serde` field order, not
  through call sites.** ADR-0041 makes Rust struct field order the canonical key order, so
  *reordering a field is a format change* visible in `fmt` and `LAYOUT` on every file. That is
  a reason to land the field order once and treat it as frozen, which argues for 3 being
  complete on the *shape* of every type before 4 starts — and against splitting 3 in a way that
  adds fields to existing structs later. See Q3 for a split that respects this.

---

## Q3 — Granularity.  **REFUTE** on 3, 10, 15, 17; **ACCEPT** on 11 with one split; merges below.

**Ticket 3 cannot fit one context window and its demo is currently false.** Inventory of what
"the full document model" means once the ADRs are read: project header + `fonts` + `fontVendor`
(ADR-0057) + `loop` (ADR-0062); tracks; five element types plus `transition` (ADR-0059);
transform fields, nine `origin` keywords, `clip`, `fit`/`par`, source ranges, `speed`, `overrun`,
`volume` (keyframable, ADR-0055); keyframes with named and bezier `ease` and the position rule
(ADR-0038); `runs` with per-run stroke and `highlight` (ADR-0048); `effects` as a discriminated
union of seven members (ADR-0040/0049); colour grammar; `radius`; nine retired spellings each
naming its replacement *and* each classified advise/refuse with a geometry-grouped census for
`gravity` (ADR-0043); schema generation; the committed-vs-generated drift test; the fixture
round-trip; and the migration the fixture actually needs. Split on this line:

- **3a — the shape.** Every struct, every field, field order frozen, `deny_unknown_fields`,
  `i64`/half-open, schema generation + drift test, unknown key as `E-*` finding (13). Demo: the
  fixture — *after* its `mask` migration, which is inside this ticket — round-trips byte-for-byte.
  This is the ticket that must be complete on shape before 4.
- **3b — retired spellings (14) as checks**, blocked by 3a. Nine spellings, each a fire/no-fire
  pair, each with its `repair` class; `gravity` refuse-class with census. This is where 39's
  acceptance lives.

**Ticket 10 is two engines.** Text metrics via `parley`/`skrifa` with `complex-scripts`, UAX #14
line partition, break opportunities with segmenter version named, per-line `baseline_y`
(ADR-0029, across every run), advance width, exact block height — that is one ticket (50, 51).
Fitted extents (52) is `fit` arithmetic on source dimensions (ADR-0013) with no font in it and
belongs to 7 alongside `E-SOURCE-OVERRUN` and the ADR-0023 pipeline. 53 goes to 15. 54
(font-swap census) has no clean home: ADR-0007 puts it in `validate` keyed off the font-file
probe cache — so it is a 7 story, not a 10 story.

**Ticket 11 fits, once 12 leaves it (Q2) — but split the resolver from the expensive half.**
11a: resolver + `query --at` with presence, painter's order, resolved values (blocked by 3, 9).
11b: offset into source, crop rectangle, ink box, `NOT COVERED` (blocked by 11a, 10, 6). 15 then
blocks on 11b (it prints the block, story 48) but its rasterizer work can begin against 11a.

**Ticket 15 cannot fit.** As written it is: image resample through `clip`/`fit`; `rect`/`ellipse`
with fill, inside-stroke, `radius`; text as `skrifa` outlines with outside-stroke and `highlight`
windows; `opacity`/`rotation`/`origin`; seven effects in order; `crossfade` transitions; `frame`'s
JPEG/PNG/half-scale/`--crop`; the `query --at` caption; the 500 ms budget test; SSIM golden
frames against the two-arm harness; and the reference-MP4 falsification (which itself needs
FFmpeg to extract frames — another reason 82 must precede 15). Split:

- **15a — `frame` on what the fixture uses** (image, rect, text, `mask`, keyframed `scale`),
  46–49, golden frames, budget test, and the falsification comparison against
  `reference/frame-intro.png` / `frame-05-at-11s.png` and frames pulled from the MP4.
- **15b — the rest of the paint vocabulary**: blur, shadow, the four colour scalars, `ellipse`,
  strokes, `highlight`, `crossfade`, `rotation`/`opacity`. Each demoable via `frame` on a small
  fixture that uses it; none has a story in #168 (see Q4).

**Ticket 17 cannot fit.** FFmpeg encode, the frame loop, atomic rename (55), partial-render
naming and refusal (56), the check gate (57), surviving-`review` + `NOT CHECKED` footer (58),
stdout/stderr contract (59), the 2-minute budget (60) — *plus* audio: 20 audio elements, four
with `speed: 0.645` (`atempo`), `volume` keyframes (ADR-0055), `overrun: loop` on audio,
`R-SOURCE-CUT-POP`-style wrap under `loop`. Split:

- **17a — video-only `render`** of the fixture: 55, 57, 58, 59; blocked by 7, 8, 9, 15a.
- **17b — audio mix**: `speed`, `volume` keyframes, `overrun`, `loop`, mixdown; blocked by 17a.
- **17c — `--from --to`** naming and refusal (56), and the 60 s budget (60); blocked by 17a.
- 18 blocks on 17b; 16 on 17c; 22 on 17a.

**Merges.**

- **5 into 4.** `create_project` (1) is the serializer from 4 applied to a constant; the MCP
  resources (2, 3) are two `rmcp` resource registrations over the schema 3a already generates.
  "First write tool; proves findings-as-result" is a one-assertion test, not a ticket.
- **21 stays separate** (it has a prototype at `docs/research/prototypes/timeline-output/` and
  no dependents) but it is a candidate to absorb the `query --from --to` text projection from
  12 if 12 turns out small.
- Do **not** merge 8 and 9 — they are already at the size where one context window holds the
  ADRs they cite (0004, 0005, 0020, 0045, 0006 for 8; 0019, 0060, 0038, 0041 for 9).

---

## Q4 — Coverage and the three judgment calls.  **ACCEPT WITH MODIFICATION** on coverage; (a) **ACCEPT**; (b) **ACCEPT WITH MODIFICATION**; (c) **ACCEPT WITH MODIFICATION**.

### The story walk

Every numbered story (1–83, plus 62a) is claimed by exactly one ticket. **Nothing is dropped by
number.** The defects are in what the numbers hide.

**Misowned** (claimed by a ticket that cannot verify it):

| story | claimed by | verifiable only at | note |
| --- | --- | --- | --- |
| 39 | 1 | 3b | first refuse-class check is `gravity` |
| 18 (second half: "never gates `render`") | 9 | 17a | `LAYOUT` gating is a `render` property |
| 52 | 10 | 7 | `fit` arithmetic, not text |
| 53 | 10 | 15 | renderer property |
| 54 | 10 | 7 | ADR-0007 places it in `validate` via the font-file probe cache |
| 78 | 6 | 7 | `par` is read off the element |
| 80, 82 | 22 | 6 | needed by the first tool that spawns a subprocess |
| 31 | 14 | 9 | no motion in it |

**Implicitly assumed** — work a ticket must do that no story and no ticket names:

1. **Audio mixing** (ADR-0055, ADR-0020's `atempo` convention, `loop` per ADR-0062). Stories 55–60
   never say the audio is rendered. 18 cannot pass without it. Assumed by 17.
2. **Effects, transitions, `highlight`, strokes, `ellipse`, `rotation`/`opacity` rendering**
   (ADR-0040, 0049, 0059, 0048, 0014). Assumed by 15. The Out-of-Scope section of #168 lists what
   is *excluded* from these vocabularies, which implies the rest is in scope — and then writes no
   story for it.
3. **`fonts vendor` / `fonts list`** (ADR-0057) — an entire CLI verb pair. Not in #168's "three CLI
   verbs", not in any ticket. Also **`validate`'s font-attestation `error` and orphaned-attestation
   `note`** (ADR-0057), and ADR-0007's glyph-coverage `error`, font census, grapheme-cluster,
   invisible-character and mixed-normalization checks. The fixture carries a `fontVendor` table
   whose `sha256` nobody will ever check. Proposed: a **ticket 23 — `fonts list`/`fonts vendor`
   and the attestation checks**, blocked by 3a and 7; and the ADR-0007 text checks folded into 10
   (they are the one set of checks that genuinely need the font binary).
4. **Anchor/layer resolution** as a function — needed by 9 (checks), 11 (painter's order), 15
   (draw order). Assumed by all three; owned by none.
5. **Slack computation** — assumed by 19 and 20; owned by none (Q2).
6. **The fixture migration** — assumed by 3's demo; owned by none.
7. **Frame extraction from the reference MP4** for 15's falsification test — needs FFmpeg
   resolution before 15.

**Double-owned:** `fmt --check` (5) and `LAYOUT` (18) are one predicate in two tickets; slack
would become double-implemented if left as is.

### (a) CI at ticket 2 carrying `skia-safe` that nothing calls — ACCEPT

The author's argument is right and I checked its premise: ADR-0010 says the C++ dependency is
"affordable because the prebuilt matches" and calls the premise one that "decays"; #36 wants the
canary *scheduled* because "the failure arrives from upstream, not from a commit". A canary that
goes live at ticket 15 has zero coverage during the months in which every other ticket is built
on the affordability assumption. The cost — ~16 s and ~200 MB per cold CI build (ADR-0010's own
numbers) for a crate with one unused dependency — is trivial. The one thing to add: pin the
feature set *and* verify the resolved key is `jpegd-jpege-pdf`, not just "no source build
happened", because the prototype's `Cargo.toml` pins no features at all and a default-feature
drift would pass a naive "did it compile from source" check while changing the key.

### (b) Falsification at 15 (frames) rather than 18 (whole video) — ACCEPT WITH MODIFICATION

The reasoning is sound: #168 wants the comparison "early", a whole-video comparison is by
construction the last thing possible, and `frame` at fixed timestamps is the earliest point at
which the format can be falsified. Two modifications:

1. **It is not a golden test and must not be sold as one.** The reference MP4 was rendered by a
   different pipeline (ASS subtitles, FFmpeg `zoompan` per CONTEXT.md and ADR-0010) *in a font
   that is no longer in the repo* — #143 swapped SF Pro Rounded for Open Runde, and ADR-0057
   records that "no font claims formal metric compatibility". SSIM at any threshold tight enough
   to catch a real defect will fail on every text-bearing frame for a reason that is not a
   defect. The comparison is *diagnostic* (a human looks at the pair) with a loose,
   region-masked SSIM at best; the committed `reference/frame-intro.png` and
   `frame-05-at-11s.png` are the honest inputs. State this in the ticket, or the first
   implementer will "fix" the renderer to match a dead font.
2. **18 is not redundant and must not be quietly demoted.** Frames cannot falsify duration
   (65.216 s against the stream, not the container — ADR-0011's `probe` table), frame count,
   audio placement, or `speed: 0.645` on four narration elements. 18 is the only test of 17b.

Also: "early" is relative. Under the draft, 15 sits behind 3 → 10 → 11; under the Q3 split it
sits behind 3a → 10 → 11a, with 15a able to begin against the resolver alone. That is as early as
the data dependencies allow.

### (c) Refuse-class folded into ticket 1's finding type, each check classifying its own — ACCEPT WITH MODIFICATION

This is what ADR-0043 literally prescribes: "decided once, by whoever authors the check, at the
moment the check is written — not a special case of retired-key handling", with a
"declarative key-to-class table" explicitly rejected. A separate refuse-class ticket would be
that table by another name. So the *shape* — `repair` on the type in 1, classification per
check — is correct.

The modification: ticket 1 cannot exercise it, and the draft's wording ("the refuse-class rule
from 38–39") lets 1 close with `repair` untested. The acceptance criteria for 38 (census present
where one exists) and 39 (`repair: "none"`, uniform per check, no override) belong to 3b, where
`gravity` provides a refuse-class instance with a real geometry census
(`docs/research/juries/format-versioning/experiment-gravity-fork/`), and the non-bypassability
half of 39 belongs to 17a, since "no future flag lifts it" is a property of `render`. Ticket 1
keeps the type and the JSON shape only.

---

## The single most important thing the author has got wrong

**The blocking graph was drawn from the verb table, and the verb table is not the dependency
structure.** Every missing edge above has the same cause: a ticket was placed by which verb it
*ships* rather than by which data it *consumes*. The sharpest instance is FFmpeg resolution
(stories 80, 82) sitting in ticket 22, blocked by 17, when `probe` at ticket 6 cannot spawn
`ffprobe` without it — a dependency inversion four tickets long. The same error puts `preview`
behind `frame` instead of behind `render`, blocks the caption checks on a font engine none of
them reads, and blocks the cut list on a keyframe resolver that never runs. Redraw the graph
from inputs (document, media facts, resolved layers, resolved keyframes, laid-out text, encoded
video) and the three spurious edges disappear, the five missing edges appear, and the frontier
at day one becomes {1} → {2, 3a, 6} → {3b, 4, 7, 8, 9, 10, 12, 13, 21} instead of the near-linear
chain the draft implies.
