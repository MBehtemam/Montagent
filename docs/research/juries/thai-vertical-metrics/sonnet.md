# Ballot — Sonnet

## Verdict

Candidate 1 is dead on arrival: for both OFL faces this repo would plausibly vendor,
`hhea` and `OS/2` sTypo* are numerically identical and `fsSelection` bit 7
(`USE_TYPO_METRICS`) is already set, so the stack (`skrifa::metrics::Metrics::new`,
called from `parley::layout::data::push_run`) is *already* on the OS/2-typo branch —
reading the table candidate 1 names would change zero rendered pixels on this
evidence. Candidate 2's own floor is not a script constant: Noto clears at
`line_height=1.3`, Sarabun needs `1.6` — a 0.3 spread between two faces of the same
script at the same size — so a schema floor keyed on `script: "thai"` would either
under-protect Sarabun or over-constrain Noto, and the mechanism that would fix that
(font-derived, not script-derived) is the one thing ADR-0007 forbids putting in the
schema. Candidate 3 (push it to font selection) is defensible only as a stopgap,
because it asks a document author to solve, per font and per size, a problem the
*tooling* can compute exactly and cheaply from data it already opens (the same font
bytes ADR-0007 already requires by path) — refusing to compute it and refusing to
report it are two different postures, and the ticket conflates them. The sharpest
finding is outside all three candidates: a **single line** of full-stack Thai text —
no second line to collide with — already pokes ink above its own declared box at
`line_height=1.1` (Noto +8.45px, Sarabun +15.51px, both MEASURED) and still does at
`1.3` (Noto +2.95px) and `1.4` (Noto +0.20px). This is not an inter-line collision at
all; it is the single-run case ADR-0029 already named ("ascent + descent exceeds the
slot") going unchecked because `R-BOX-SLACK` is arithmetic-only (declared `height` vs.
computed slot height, never real ink) and no check in `checks/mod.rs` other than the
one I was told not to read appears to touch ink against a box. If that forbidden file
already covers this, my finding is redundant with it; if it does not, this is a gap
in #325's framing, not just in its candidate list.

## Q1 — Candidate 1: which table, and does it change a pixel?

**READ**, `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/skrifa-0.46.2/src/metrics.rs`,
`Metrics::new` (starts line 106): the function implements FreeType's own fallback
chain verbatim (comment cites `sfobjs.c`) — *use OS/2 sTypo* if the table exists and
`fsSelection` bit 7 (`USE_TYPO_METRICS`) is set; otherwise `hhea`; otherwise, if
`hhea` is all-zero, `OS/2` sTypo if non-zero else `usWin`*. So the table actually read
is **conditionally OS/2 typo, never unconditionally `hhea`** — candidate 1's premise
("whatever `hhea`-derived value skrifa/parley currently reports") is not what the
code does.

**READ**, `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/parley-0.11.1/src/layout/data.rs`
line 414, inside `push_run`: `skrifa::metrics::Metrics::new(&font_ref,
skrifa::prelude::Size::new(font_size), coords)` — parley calls exactly this function
and adds no metric-source logic of its own (matches the probe's own comment at
`docs/research/prototypes/thai-vertical-metrics/probe/src/main.rs`, which I also read
and agree with independently).

**MEASURED**, I ran `docs/research/prototypes/thai-vertical-metrics/run.sh` myself
(fonts already present, sha256-pinned, verified by the script; a warm and later a
clean `cargo build` both succeeded) and it reproduced every one of its own 13
asserted headline numbers (`ok` on all). I then independently cross-checked the four
raw-table numbers with a second, unrelated toolchain (`fontTools.ttLib`, Python, in a
throwaway venv, not part of this repo's dependency graph) against the two committed
font files:

| face | unitsPerEm | hhea asc/desc/gap | OS/2 sTypo asc/desc/gap | OS/2 usWin asc/desc | fsSelection bit7 |
|---|---|---|---|---|---|
| Noto Sans Thai Regular 2.002 | 1000 | 1061 / −450 / 0 | 1061 / −450 / 0 | 1061 / 450 | **true** |
| Sarabun Regular 1.000 | 1000 | 1068 / −232 / 0 | 1068 / −232 / 0 | 1286 / 567 | **true** |

Both tools agree exactly. On both faces `hhea == OS/2 sTypo` field-for-field and bit
7 is set, so `Metrics::new` is already resolving to the OS/2 typo values — there is
no divergence between what candidate 1 proposes and what the code already computes.
**Candidate 1 would change zero rendered pixels on these two faces.** I cannot rule
out a font where `hhea != OS/2 sTypo` and bit 7 is unset (that font would genuinely
hit the `hhea` branch and candidate 1 would matter) — I did not go looking for one,
and doing so was out of scope for a two-face brief; flagging it because "candidate 1
is a no-op" is a claim about *these two faces*, not about the table-read logic in the
abstract.

`usWin` is the one number that does diverge (Sarabun 1286/567 vs. sTypo 1068/232,
Noto's own usWin coincidentally equals its sTypo) — irrelevant to candidate 1 as
literally stated (it names `OS/2 typo`, not `usWin`), but worth naming because it is
the fallback #3 in the FreeType chain above, and a reader could confuse "OS/2 has
bigger numbers" with "OS/2 typo has bigger numbers." It doesn't.

## Q2 — Candidate 2: is the floor a script property or a face property?

**MEASURED** (re-ran `run.sh` myself; numbers below reproduce `results.txt`, sweep is
ADR-0028's tenths 1.0–2.5, size 55, "ink" = glyph outline bbox, not advance box):

- Noto Sans Thai, full-stack 2-line Thai: first collision-free tenth is **1.3**
  (seam at 1.1 is +9.58px overlap; at 1.3 it clears by −1.42px).
- Sarabun, full-stack 2-line Thai: first collision-free tenth is **1.6** (seam at 1.1
  is +26.52px; even at 1.3, still +12.77px overlap — candidate 1's own generous
  reading, an sTypo-derived 1.3 slot, still collides on Sarabun).
- Latin control, same faces: clears at **1.0** on both.

That is a **0.3-tenths spread between two faces of the identical script** (1.3 vs.
1.6), against a spread of essentially zero between the two faces on Latin (both clear
at 1.0). **This is a face property, not a script property** — the floor tracks each
font's own ascent/descent geometry (Sarabun's descent alone is 232 units vs. Noto's
450, but its glyphs draw lower relative to its own metrics; I did not decompose why,
only that the two faces disagree by an amount too large to attribute to measurement
noise: 0.3 line-height tenths is 16.5px at size 55).

**What this implies about a schema floor, INFERRED**: a floor keyed on
`script: "thai"` alone has no single correct value — pick 1.3 and Sarabun still
collides; pick 1.6 and every Noto-set project pays for headroom it doesn't need
(Noto's slot at 1.6 has −17.92px of *negative* seam, i.e. ~18px of dead vertical
space per line-gap it didn't need). A script floor is only sound if defined as
"the max over every font a project could vendor for that script," which is not a
fixed number the schema can pin — it would have to be re-derived every time a new
face enters the licence-gated vendor set (ADR-0057), at which point it is no longer
a schema constant, it's a computation over the font, which is candidate-1-shaped
data the schema still isn't storing. A **per-face-and-size floor, computed at
`measure`/`validate` time from the actual vendored font bytes**, is the only version
of "a floor" that is both correct and requires no schema change — which is a
different shape of answer than "the format has no way to express this," because the
gap is not expressiveness, it's that nothing currently computes and reports the
number.

## Q3 — Candidate 3: if the format declines to repair, what should the tools do?

**READ**, `docs/adr/0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md`
and `docs/adr/0006-validate-reports-facts-and-render-enforces.md`: this repo already
has the exact vocabulary this question needs. Severity is closed to three levels —
`error` ("the render is refused or is guaranteed wrong"), `review` ("legal, renders,
you must look at a frame to know if it was meant"), `note` ("a fact you may want and
will not act on today") — and every `error`-class finding carries a `repair` field
that is either a computed value (**advise-class**) or the literal string `"none"`
(**refuse-class**, no bypass, ever).

**My answer, INFERRED from that architecture, not read off any decided ticket**:
this is squarely a **refuse-class `error`**, not `review` and not `note`. It fails
`review`/`note` because the ADR-0043 test for `error` is "guaranteed wrong," and
measured ink-to-ink overlap at a declared `line_height` is exactly that — not a
judgment call about intent, a geometric fact. It is refuse-class rather than
advise-class because the *correct number* to repair to depends on which font the
project will end up vendoring and at what size, which the check can name but not
choose on the author's behalf (the same shape of problem ADR-0043's `gravity`
precedent walks through: computable existence of a problem, non-computable
single correct fix). Concretely: a check (name aside) that, for every text
element, opens the declared font(s) via the same path ADR-0007 already requires,
computes real ink for the line(s) as this probe does, and emits `error`/refuse when
ink-to-ink overlap is positive at the *declared* `line_height` — with a sibling
census grouping by the same declared `line_height` value the way `R-BOX-SLACK`
groups by declared `height` (ADR-0043's pattern: state the fact, not the fix).
`render` refuses on it per ADR-0006's existing rule; no new bypass. This needs no
schema change (Q2 already established the number isn't a legitimate field) — it is
a `measure`/`validate`-time computation over data the tools already have to open to
draw a single glyph, the identical argument ADR-0029 already used to keep
`baseline_y` out of the schema.

I want to be explicit that I could not confirm or deny whether this already exists:
this repo's checks directory lists a module named `ink` (`crates/montagent-core/src/checks/ink.rs`,
plus `crates/montagent-text/src/ink.rs` and a test file), and the current branch is
named `impl/325-thai-ink-seam` — both facts I necessarily saw as filenames while
listing the source tree and could not avoid, but I did not open any of the three
forbidden files and my Q3 answer was derived independently, before I noticed the
branch name. If a working check with this exact shape already ships, my answer here
should be read as "I'd have proposed the same repair," not as new information.

**If the tools decline to do even this**: I do not think "nothing" is the right
answer once the ink-vs-box math is this cheap and this exact (no rasterization, no
sampling — closed-form from the glyph outline the renderer already opens). "Nothing"
would only be defensible if computing real ink were expensive, approximate, or
required information the renderer doesn't already have at render time, none of
which is true here.

## Q4 — the question none of the three asks

**MEASURED**, by modifying the probe myself (a one-line-only sample of the same
worst-case Thai stack `THAI_STACK`, added locally, run, then reverted — the repo is
back to its committed state, verified via `git status`/`git diff` showing no changes
under `docs/research/prototypes/thai-vertical-metrics/`): a text element with
**exactly one line** — nothing to collide with — still has its ink escape the
element's own vertical box at every `line_height` #325's worked example would
plausibly use:

| face | line_height=1.1 (ADR-0007's worked value) | 1.3 (candidate-1-implied, Noto) | 1.4 |
|---|---|---|---|
| Noto Sans Thai, 1 line | top escape **+8.45px** | top escape **+2.95px** | top escape **+0.20px** |
| Sarabun, 1 line | top **+15.51px**, bottom **+10.51px** | top **+10.01px**, bottom **+5.01px** | top **+7.26px**, bottom **+2.76px** |

("escape" = ink extending past the block's own top/bottom edge as derived from the
declared `line_height` and size — same metric the committed probe already computes
for the *block* as a whole, I only fed it a one-line input instead of two.)

This is the single-line defect ADR-0029 already flagged in prose ("against the
fixture's actual font at size 55, ascent + descent **exceeds** the ... slot") but
which #325's three candidates all implicitly assume away by framing the problem as
"two consecutive lines" — a single Thai caption, one line, no second line in sight,
already has ink poking out of its own declared box at a `line_height` (1.1) the
existing worked example in ADR-0007 uses. None of the three candidates fix this:

- **Candidate 1** is irrelevant here for the same reason it's irrelevant to Q1 — the
  slot these faces compute is already OS/2-typo-derived, and the escape persists at
  every tested tenth up to 1.4–1.6 depending on face.
- **Candidate 2**, a script-aware `line_height` *floor*, is aimed at inter-line
  spacing; it happens to also raise the single-line slot height as a side effect
  (a bigger slot contains ink better), so it would incidentally help here too, but
  the ticket frames it entirely around "two consecutive lines... do not overlap,"
  never naming the single-line box-escape case as a thing to solve for. A floor
  tuned to stop inter-line collision is not guaranteed to be tall enough to contain
  one line's own ink inside its own box — those are two different inequalities that
  happen to be satisfied by similar-looking numbers on this data, not because they're
  the same constraint.
- **Candidate 3** (font selection) doesn't help unless the selected font's
  ascent+descent happens to fit inside `size × line_height` for whatever
  `line_height` the author picked — the same problem, pushed to a human who has no
  tool telling them where the threshold is.

I also checked whether an existing check catches this and could not find one that
does: `R-BOX-SLACK` (`crates/montagent-core/src/checks/box_slack.rs`) is explicitly
"height only... no font, no I/O" and compares the *declared* box height against a
*computed-from-declared-fields* expected height — it cannot see real ink because it
deliberately never opens a font. Whatever check (if any) does open a font to check
ink is one of the three files I was told not to read, so I cannot confirm from this
side of the wall whether this exact single-line case is covered. My best-effort
search of every other `checks/*.rs` module found nothing else that measures glyph
outline ink against a box.

## What blocked me

Nothing blocked the measurements — Rust toolchain, network-independent (fonts were
already fetched and hash-verified), and a second independent tool (`fontTools`) were
all available. The one thing I could not do inside this brief's scope was search for
a font where `hhea != OS/2 sTypo` or bit 7 is unset, to give candidate 1 a case where
it would actually matter — flagged above as a scope limit, not a blocker.

I did modify `docs/research/prototypes/thai-vertical-metrics/probe/src/main.rs` and
re-ran `run.sh` twice, once with a one-line sample added (for Q4) and once to revert;
`git status`/`git diff` on that directory confirm no residual changes.
