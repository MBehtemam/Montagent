# Brief: the proposed v1 ticket breakdown

The author has broken [#168](https://github.com/MBehtemam/Montaget/issues/168) — the Montaget
v1 implementation spec, 83 user stories — into 22 tickets. **This draft has not been attacked.
Your job is to attack it.**

## Context you must establish yourself

- Read #168 in full: `gh issue view 168` in `/Users/mohammedehtemam/projects/github/Montaget`.
- `docs/adr/README.md` is the index of all 67 ADRs, with an *Amended by* column. Read it before
  reading any single ADR — roughly a third of the series amends another.
- `CONTEXT.md` is the domain glossary. `docs/agents/domain.md` carries the ADR conventions.
- There is **no source tree**. This is a greenfield Rust build. Nothing has ever been parsed,
  checked or rendered by the tool the ADRs specify.

## The rules the breakdown is supposed to satisfy

From the `/to-tickets` skill:

- Each slice cuts a **narrow but COMPLETE path through every layer** (schema, API, UI, tests) —
  vertical, NOT a horizontal slice of one layer.
- A completed slice is **demoable or verifiable on its own**.
- Each slice is sized to fit in **a single fresh context window**.
- Any prefactoring should be done first.
- Each ticket declares its **blocking edges** — the tickets that must complete before it starts.
- **Wide refactors are the exception**: a mechanical change whose blast radius fans across the
  codebase is sequenced expand–contract, not forced into a tracer bullet.

## The draft breakdown

**1. Workspace skeleton, findings, and `validate` on a header-only project** — *blocked by: none*
The four crates (`montaget-core`, `montaget-text`, `montaget-render`, `montaget`), the `Finding`
type (code, severity, location, inline numbers, sibling census, `repair`), JSON-canonical report
with the text form generated from it, `NOT CHECKED` block, `E-PARSE` with line/column/byte
offset/offending line/caret, exit codes 0/1/2/3/70, and both adapters (CLI + MCP) wired thin.
Demo: `montaget validate` on a hand-written header-only project, over CLI and MCP.
Stories 35–37, 40–42, and the refuse-class rule from 38–39.

**2. CI on six tier-1 targets with the Skia prebuilt canary** — *blocked by: 1*
Story 83 / #36. Author's judgment call: this lands second, before any rasterizing, with
`montaget-render` carrying the `skia-safe` dependency from day one even though nothing calls it —
on the argument that the canary is worthless if it only goes live at ticket 15, and ADR-0010's
affordability argument is load-bearing for everything after.

**3. The full document model, and the committed fixture parses clean** — *blocked by: 1*
Track/element/run/keyframe, `deny_unknown_fields` at every level, `i64` milliseconds and
half-open ranges, JSON Schema generated from the Rust types with a committed-vs-generated drift
test, unknown key as a named error, retired spellings naming their replacements. Demo: the
60-element committed fixture round-trips. Stories 13, 14.

**4. `fmt`, the canonical convention, and atomic whole-file write** — *blocked by: 3* — stories 4–8, 70.

**5. `create_project` and the MCP resources** — *blocked by: 4* — stories 1–3. First write tool;
proves the findings-as-result rule.

**6. `probe` and the media-facts authority** — *blocked by: 3* — stories 76–78, 20, 22.

**7. `validate` against the disk** — *blocked by: 6* — stories 19, 21, 23, 24, `E-SOURCE-OVERRUN`.

**8. Structural time checks** — *blocked by: 3* — overlap, gap, the exact `speed` invariant,
`N-QUANTIZATION`. Stories 10–12, 34.

**9. Structural document checks** — *blocked by: 3* — layer ties, anchors, keyframe `ease`
position, `LAYOUT` key order. Stories 15–18.

**10. `measure` and the text engine** — *blocked by: 3* — stories 50–54, plus exact text block
height and fitted extents.

**11. The keyframe resolver and `query --at`** — *blocked by: 10* — story 43.

**12. `query --from --to` and `--where --census`** — *blocked by: 11* — stories 44, 45.

**13. Caption and text consequence checks** — *blocked by: 8, 10* — stories 26–30.

**14. Geometry and motion consequence checks** — *blocked by: 11* — stories 25, 31–33.

**15. The rasterizer, `frame`, and the first falsification test** — *blocked by: 11*
Stories 46–49, golden frames by SSIM with a stated threshold. Author's judgment call: #168 says
the fixture-versus-published-MP4 comparison "should happen early, not at the end", but it reads
as a `render` test, which cannot be early. Comparing *frames* at fixed timestamps against the
reference MP4 needs only `frame`, so the falsifying comparison lands here rather than at 18.

**16. The `preview` proxy ladder** — *blocked by: 15* — stories 61, 62, 62a, 63; the two floors
kept as two constants per ADR-0067.

**17. `render` end-to-end** — *blocked by: 7, 8, 9, 15* — stories 55–60. Blocked by all three
check tickets because "render runs the identical checks" is the enforcement.

**18. Whole-video fixture comparison against the published MP4** — *blocked by: 17*.

**19. `shift`** — *blocked by: 4, 8* — stories 64–69.

**20. `compare`** — *blocked by: 8, 11* — stories 71–75.

**21. `timeline`** — *blocked by: 3* — story 9.

**22. Packaging, releases, and FFmpeg resolution** — *blocked by: 17* — stories 79–82.

## The questions put to you

**Q1 — Vertical or horizontal?** Is each ticket a genuine tracer bullet, or are some of them
horizontal layer slices wearing a vertical costume? Name every ticket that is not demoable on its
own, and say what it would take to make it one. Consider in particular whether ticket 1's
"header-only project" demo is a real slice or a stub, and whether tickets 8, 9, 13 and 14 are
four horizontal cuts through one check engine.

**Q2 — The blocking edges.** For each edge: is it real? Name every **missing** edge (a ticket that
could not actually start with its declared blockers done) and every **spurious** edge (a false
serialization that costs parallelism for nothing). The author is least confident about 13 → 10.
Also consider whether the wide-refactor exception applies anywhere here — in particular whether
the document model (ticket 3) or the finding type (ticket 1) is a change whose later evolution
will fan out across every other ticket, and if so whether expand–contract should be sequenced
rather than assuming the type is right the first time.

**Q3 — Granularity.** Which tickets cannot fit in a single fresh context window? Which should be
split, and on what line? Which should merge? Be concrete about 3, 10, 11, 15 and 17.

**Q4 — Coverage and the three judgment calls.** Walk #168's 83 stories against the 22 tickets:
name any story that is **dropped**, **double-owned**, or **implicitly assumed** by a ticket that
does not claim it. Then rule on each of the author's three judgment calls:
  (a) CI at ticket 2 carrying a `skia-safe` dependency nothing calls yet;
  (b) the falsification test at 15 (frames) rather than 18 (whole video);
  (c) refuse-class findings (stories 38–39) folded into ticket 1's finding type rather than
      given their own ticket, with each check ticket classifying its own checks.

## How to answer

**Default to "refuted".** Attack the draft rather than assume it. A juror who agrees with
everything has not done the exercise — but do not manufacture disagreement either; if a thing is
right, say why it survives attack.

Verify claims against the repository rather than against the brief. The brief may contain errors;
finding them is the exercise.

Structure your ballot as: a verdict line per question (`ACCEPT` / `ACCEPT WITH MODIFICATION` /
`REFUTE`), then the reasoning, then a concrete counter-proposal for anything you refute. Be
specific — ticket numbers, story numbers, ADR numbers. End with the single most important thing
the author has got wrong.
