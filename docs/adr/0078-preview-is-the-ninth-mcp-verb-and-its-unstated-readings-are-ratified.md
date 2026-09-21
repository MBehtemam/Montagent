---
status: accepted
amends: 0011 (the verb table gains a `preview` row and the counts become nine MCP tools /
  twelve CLI commands; the exit-code table's row 3 is what a budget hard-fail returns),
  0006 (its "`render` runs the identical checks and is what enforces them" rule extends to
  `preview`), 0021 (the enforced `<5 s` bounds one attempt rather than the invocation; a
  span past its deadline is abandoned; the observational arm carries no deadline and is
  never degraded), 0046 (a project between the two caps degrades from true pixels to a real
  proxy, and a rung whose cap never engaged is disclosed as `native`), 0050 (its "any
  resolution request below 360x640-equivalent is a refusal" bullet is bounded to a *proxy*
  resolution, never the author's declared frame)
---

# `preview` is the ninth MCP verb, and the eleven readings #218 had to pick are ratified — one with a correction

**Ticket:** [#295](https://github.com/MBehtemam/Montaget/issues/295).
[#218](https://github.com/MBehtemam/Montaget/issues/218) built `preview` and the proxy
ladder. The ladder is fully ADR-stated — [ADR-0021](0021-preview-budget-and-graceful-degradation.md),
[ADR-0046](0046-proxy-preview-target-is-720p-long-edge-capped.md),
[ADR-0050](0050-preview-hard-refuses-below-360p.md),
[ADR-0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md),
[ADR-0067](0067-two-floors-a-wall-clock-give-up-point-and-a-legibility-refusal.md) — and
**nothing here touches a rung, a cap or a floor**. What #218 had to pick was everything
*around* the ladder: the verb's place on the surface, its refusals, its clock and the shape
of what it discloses. Eleven such readings were recorded in the source and raised rather
than left to be found. Ten are ratified as written. One is ratified with a correction to the
code, below.

**Evidence:** `crates/montaget-core/tests/preview.rs`, which asserts each reading against a
real project file through the core verb, and `crates/montaget-render/src/proxy.rs`'s own unit
tests for the arithmetic. Each numbered reading below names the test that fails the moment it
stops holding. The two readings that had no test — the between-caps degrade and the
per-attempt clock — gained one in this change, which is how the correction was found.

## The surface

**1. `preview` is the ninth MCP verb. ADR-0011's table gains a row.**

Spec [#168](https://github.com/MBehtemam/Montaget/issues/168)'s title says *"nine MCP verbs,
three CLI verbs"* and its stories 61–63 are `preview`'s, so the ninth is this verb.
ADR-0011's own table — the one `CONTEXT.md`'s **Verb** entry calls authoritative — lists
eight MCP tools and eleven CLI commands, and `preview` is in neither count.

**The table is what was wrong, not the spec.** The row:

| verb | MCP | CLI | what it is for |
| --- | --- | --- | --- |
| `preview` | ✅ | ✅ | what a span looks like in motion, fast enough to scrub |

and ADR-0011's *"Eight MCP tools, eleven CLI commands, two resources"* reads, from here on,
**nine MCP tools, twelve CLI commands, two resources**. ADR-0011 keeps its original words
per `docs/agents/domain.md`; this ADR is where the row lives and its banner is where a
reader of the table is sent.

The MCP count is asserted rather than restated:
`crates/montaget/tests/adapters.rs::the_mcp_surface_is_exactly_nine_tools_and_preview_is_the_ninth`
reads `tools/list` off the running server and names all nine. ADR-0011's own opening
sentence — *"nine verbs and two resources"*, above a table of eleven — is the standing
demonstration of what a count in prose is worth once the thing it counts has moved.
`CONTEXT.md`'s **Verb** entry is updated to state the
resolved counts and drop the *"no count here should be quoted as settled"* hedge #295 put
there.

This is not a tenth verb in [ADR-0037](0037-derived-time-signature-is-a-provenance-gap-not-a-tool.md)'s
sense. ADR-0037 refused a *new* capability; `preview` is spec #168's own verb, built, and
missing only from a table.

**2. `preview` runs the identical check engine and refuses on any `error`.**
([ADR-0006](0006-validate-reports-facts-and-render-enforces.md) states that rule for
`render` and not for this verb.) Ratified. A document the checks refuse is one no painter
can be handed, and a preview that rendered what `render` refuses would be the one artefact
in the product showing an illegal project — the agent would then be judging a picture of a
file it cannot export. Both verbs call `validate::checked` and answer the same document with
the same exit code: `preview_runs_the_same_checks_render_runs_and_refuses_on_an_error`.

**8. A hard fail on the budget is exit 3.** Ratified, under ADR-0011's own exit-code table,
whose distinguishing question is *"what the caller does next"*: the document is legal
(not 1), the file parsed (not 2), and Montaget did not break (not 70). What could not be
satisfied is the invocation, and row 3's next move — *fix the command* — is exactly the
lever available: a shorter range, or the full-resolution escape hatch. Asserted in
`a_540p_miss_hard_fails_and_there_is_no_third_tier` and
`a_project_with_nothing_left_to_degrade_to_fails_rather_than_re_render_the_same_frame`.

## The artefact

**3. A preview never lands on the deliverable, with or without a range.** Ratified.
`render`'s story-56 rule is about *partial* renders; this is stronger, and carries an
additional reason: every frame here may be proxy-scaled, so a preview on the deliverable's
path is not merely incomplete, it is the wrong size. The derived name is
`out/<name>.preview.<from>-<to>.mp4`, and an explicit `--output` naming the project's own
`output` is refused whatever the range: `a_preview_may_never_land_on_the_deliverable`.

## The clock

**4. The budget is judged per attempt, on that attempt's own clock**, while the answer's
`wall_ms` is the whole invocation. Ratified, and **the cost is recorded rather than
smoothed over: a degraded preview can cost up to twice the budget.**

ADR-0021 enforces `<5 s` for *"the common-case proxy-resolution scrub preview — the number
every caller hits by default"*, which is one rung, and ADR-0065's 2.68 s / 3.78 s are what
*one* render of a span cost. So a rung is what the number describes and a rung is what it is
scored against. The two alternatives are worse:

- **A whole-invocation clock** leaves the second rung whatever the first miss did not spend
  — usually nothing, since the first rung missed by running out. ADR-0021's *required*
  degrade step would then be a step that could never land, which is not a stricter budget but
  a broken ladder.
- **Predicting the miss rather than measuring it** is not available. The cost of a render is
  what the render takes; a ladder that responds to a miss has spent the miss by definition.

`the_budget_is_judged_per_attempt_and_wall_ms_is_the_whole_invocation` asserts both halves:
the disclosed `budget_ms` is the landing rung's own number, and `wall_ms` covers every
attempt, because that is what the caller actually waited.

**5. A span that runs past its deadline is abandoned where it stands**, rather than finished
and then judged. Ratified. A ladder that ran every rung to completion would cost the sum of
its rungs, which is the opposite of what a wall-clock budget is for, and the partial file is
deleted rather than left behind — the same test asserts the missed rung encoded fewer frames
than the span holds, and `a_540p_miss_hard_fails_and_there_is_no_third_tier` asserts an
abandoned rung leaves no file on disk.

**6. The full-resolution escape hatch is never degraded and carries no deadline.** Ratified,
and it is ADR-0021 read straight rather than a new decision: that arm is *"observational only
(unenforced)"* because *"the caller explicitly asked for true pixels and accepted the cost,
so there is no promise for a target to encode"*. A promise there is nothing to encode for is
also nothing to degrade against. It discloses `native` with a null `budget_ms`:
`the_full_resolution_escape_hatch_is_true_pixels_and_carries_no_budget`.

**9. `Clock::Stated` is a core-library seam, not adapter surface.** Ratified. Neither
adapter offers it; both pass `Clock::Scrub`, which is ADR-0021's `<5 s`. It exists because
the only other way to assert a degrade or the hard fail is to find a machine slow enough,
which tests the hardware rather than the ladder — the same objection ADR-0022 made to an
unmeasured example and ADR-0065 made to a single thin-margin run.

**The risk it carries is stated rather than hidden.** It is a public item on a library
crate, so a future adapter *could* expose it, and a caller-chosen budget is a different
decision from this one — it would let a caller buy a degrade it did not measure, and the
disclosure would then report a tier chosen against a number no ADR wrote. Nothing here
admits that; if it is ever wanted it is its own ticket. The guard is that both adapters
construct `Ask` with `Clock::Scrub` literally, and that neither advertises an argument for
it: `cli_preview_offers_the_escape_hatch_and_no_flag_that_names_a_tier` and
`mcp_preview_advertises_the_escape_hatch_and_no_tier_argument` both list `clock` among the
arguments that must be absent, so the leak fails a test rather than a review.

## The floor, the caps and what gets disclosed

**7. The legibility floor governs a proxy resolution, never the author's declared frame.**
Ratified, and it **bounds ADR-0050's Consequences bullet**, which reads *"any resolution
request below 360×640-equivalent (long-edge capped, aspect preserved) is a refusal"* and, on
its own, reads the other way.

A project declaring a 320×180 frame is previewed at its own pixels. Nothing downscaled it,
so there is no proxy for the floor to judge, and `preview` is not the verb that tells an
author their project is too small — a verb that refused here would be `validate`'s job done
in the wrong place, and would refuse a project `render` exports happily. What settles it is
ADR-0067, which is later and governs: it records the 360p threshold as *"unreachable
today"* and *"a guard on the ladder's future, not a live branch"*, and a declared frame that
fired it would make it reachable today, contradicting ADR-0067's own statement of where it
binds. ADR-0050's measurement, its number and its refusal text are untouched; what is
bounded is the word *request*.
Asserted: `the_legibility_floor_does_not_govern_a_project_that_declares_a_small_frame`.

**10. A project between the two caps is degraded to 540p even though the 720p cap never
engaged for it.** Ratified. Its first rung is true pixels — ADR-0046: *"for a
1280×720-native or smaller project, no proxy applies at all"* — and its second rung is a
real proxy at 960 px. This follows from the ladder being defined on **caps** rather than on
sizes, and neither ADR states it either way. The alternative — no degrade step for such a
project, because its first rung was not a proxy — would delete ADR-0021's required degrade
for exactly the band of projects where it is cheapest to take.

**11. A rung whose cap never engaged discloses `native`, not the rung's name.** Ratified.
ADR-0046 makes the field report *"native, the 720p proxy tier, or — once #117 resolves — a
floor-refuse"*, so a 200×200 project comes back as `native` with `long_edge_cap: null`, and
the prose says *which* native it is — inside the cap, versus the escape hatch. The
alternative, `name: "720p", proxied: false`, puts a number in the field that is not the
frame in front of the caller, which is the one thing the field exists to prevent.

### The correction: the same rule had not reached the trace or the prose

Readings 10 and 11 are individually correct and, taken together, falsified what the code did
in the case where they meet. `Preview.attempts` named each rung with the tier's own name
unconditionally, and the degraded disclosure hardcoded the sentence *"one tier below the 720p
target, because 720p ran to N s"*. For a project between the two caps — a 1200×600 frame,
inside the 1280 px cap and outside the 960 px one — that produced an answer disclosing
`native` in `tier.name` and `720p` in `attempts[0].tier` **for the same frame**, and a
caller-facing sentence asserting that a 720p proxy ran when true pixels did.

**Reading 11's rule is not about one field; it is about not putting a number in front of a
caller that is not the frame that was drawn.** So it now governs all three places a rung is
named — the disclosure, the attempt trace and the hard-fail refusal — through one function,
`rung_name`, and the degraded sentence names the rung above from the trace rather than
assuming it. The rung above is still the 720p *rung*; what it is no longer called is a 720p
*frame*.

This is the one code change in #295, and it is the reason the ticket's own instruction —
*"ratification or correction, not a code change first"* — was worth following: the defect
was invisible while the two readings were prose in a module header and became a failing
assertion the moment they were written as one test.
`a_project_between_the_two_caps_degrades_from_true_pixels_to_a_real_proxy` is that test.

## Consequences

- **ADR-0011's verb table gains a `preview` row; its counts read nine MCP tools, twelve CLI
  commands, two resources.** ADR-0011 keeps its original words; this ADR and ADR-0011's
  banner are where the row is found. `CONTEXT.md`'s **Verb** entry drops the *"no count here
  should be quoted as settled"* hedge and states the resolved counts.
- **ADR-0006's enforcement rule covers `preview`.** `validate` reports, `render` and
  `preview` refuse, on the identical check engine and the same `error` threshold.
- **ADR-0021's enforced `<5 s` bounds one attempt, not one invocation.** A degraded preview
  may cost up to twice it. A rung past its deadline is abandoned mid-span and leaves no file.
  The observational full-resolution arm has no deadline and is never degraded.
- **ADR-0050's refusal is bounded to a proxy resolution.** A project whose *declared* frame
  is below 360p is previewed at its own pixels. The threshold stays where ADR-0067 put it:
  unreachable today, a guard on the ladder's future.
- **A rung is named for the frame it rasterized, everywhere it is named** — disclosure,
  attempt trace and refusal. A project between the two caps degrades from `native` to
  `540p`, and no answer names one frame two ways.
- **`Clock::Stated` stays a core-library test seam.** Admitting a caller-chosen budget on
  either adapter is a separate decision and is not taken here.
- Nothing about the ladder moves: the rungs, both caps, both floors, the mandatory
  disclosure and `render`'s exemption from all of it are exactly as ADR-0021, ADR-0046,
  ADR-0050, ADR-0065 and ADR-0067 left them.

## Not settled here

**A caller-specified proxy resolution.** ADR-0067 names it as one of the two live cases the
360p guard exists for, and nothing in `preview`'s surface offers it. Reading 9's risk note is
where the argument would start, not a decision that it should exist.

**Whether `attempts` should carry the cap it was judged against.** The trace names each rung
and its frame; a caller wanting to know *why* a rung was that size reads `tier.long_edge_cap`
for the landing rung only. No one has needed the others, and inventing the field before
someone does is the speculative generality ADR-0046 rejected a second tier for.
