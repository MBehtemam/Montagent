---
status: accepted
amends: 0021 (states the deferred floor), 0046 (adopts a floor below the 720p target)
---

# `preview`'s hard-refuse floor is 360p, and the refusal names it

> **Amended by [ADR-0078](0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)**,
> which bounds the Consequences bullet reading *"any resolution request below
> 360×640-equivalent (long-edge capped, aspect preserved) is a refusal"*: it governs a
> **proxy** resolution, never the author's declared frame. A project declaring a 320×180
> frame is previewed at its own pixels — nothing downscaled it, so there is no proxy for
> this floor to judge, and `preview` is not the verb that tells an author their project is
> too small. The measurement, the number and the refusal text are untouched.

> **Amended by [ADR-0067](0067-two-floors-a-wall-clock-give-up-point-and-a-legibility-refusal.md).**
> The 360p threshold and the legibility pass behind it stand, and that pass is what retires
> [ADR-0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md)'s
> "legibility is unmeasured" hedge. What ADR-0067 adds is *where it binds*: under ADR-0065's
> `720p → 540p → hard fail` ladder, `preview` never degrades below 540p, so this threshold is
> not reached by degradation today. It is kept as a guard on any future extension of the
> ladder, not as a live branch.

[ADR-0021](0021-preview-budget-and-graceful-degradation.md) required that a floor exist
below which `preview` hard-refuses rather than return something "too degraded to make a
decision from," and deferred the number to measurement.
[ADR-0046](0046-proxy-preview-target-is-720p-long-edge-capped.md) fixed the proxy-preview
*target* at 720p and graduated the floor to [#117](https://github.com/MBehtemam/Montagent/issues/117)
as a `/prototype` ticket, on the same discipline: this is a visual-legibility judgment, not
one a wall-clock benchmark can answer.

## The prototype

The committed fixture (`fixtures/en-halloween-decorating/`) was rendered from its published
reference video at four candidate floor tiers — 720p, 540p, 360p, 240p, all long-edge capped
per ADR-0046's rule — at four timestamps chosen from the fixture's own declared text sizes to
stress its smallest and densest text: the channel handle/chip badge (34–52px, on screen the
full 65 s runtime), a two-line caption pair carrying the fixture's single smallest run (35px),
and two ordinary 55px subtitle cards for baseline comparison. Each tier was downscaled
bilinear from the native frame and scaled back to native size for equal-size viewing,
matching how a preview player displays a proxy frame in a fixed-size viewport. Assets:
`docs/research/prototypes/preview-floor-resolution/` on branch
`prototype/preview-floor-resolution`, `run.sh` regenerating every frame from the fixture's
reference render.

**The call was made by a three-juror court (Opus, Haiku, Fable), each shown the same four
comparison strips with no visibility into the others' ballots — unanimous 3/3 on both
questions.**

## The floor: 360p, not 240p

**360p (360×640 for this fixture's 1080×1920) is the hard-refuse floor.** Below it, `preview`
fails outright. All three jurors independently located the same break point: the 55px
standard subtitle cards stay legible at every tier tested, including 240p — they do not force
the floor. What forces it is the handle/chip badge and the smallest caption line (34–35px),
which read cleanly through 360p and degrade at 240p from "soft but distinguishable" to glyphs
losing their contour — a smear an agent can only "read" by already knowing the string, which
is exactly the position an agent checking its own unfamiliar render is not in. 240p was
rejected as the floor on this evidence, not on the large-text cases, which would have
tolerated it.

**Content-dependence is a recorded finding, not the shape of the answer.** Legibility does
track the smallest text on screen at a given instant, and a fixture with no persistent small
text could in principle tolerate 240p. But this project's own fixture carries small text
(the handle/chip) through nearly its entire runtime, so a per-instant floor would not buy real
headroom here — only a threshold an agent cannot predict before calling `preview`, discovered
by trial rather than stated as a contract. A fixed number taken from the worst-case text
actually observed is conservative, shippable now, and does not preclude a future adaptive
rule if a project's small-text profile ever makes the fixed floor's cost worth reopening.

## The refusal names the floor and the reason

A bare failure below 360p is indistinguishable from a render error, a bad timestamp, or a
missing asset — the caller's only lever is the resolution it requested, and with no
explanation it will retry the identical call or abandon `preview` entirely. The refusal
**must state the floor and why**, e.g.:

> `preview` refused: requested resolution is below the 360p floor; below this, on-screen
> text is not legible enough to judge a cut from.

This is one string, kept in sync with this ADR's number the same way ADR-0046's target is
kept in sync with its own.

## Consequences

- `preview`'s degradation ladder (ADR-0021) is now fully specified: 720p target
  ([ADR-0046](0046-proxy-preview-target-is-720p-long-edge-capped.md)), 360p hard floor (this
  ADR), nothing between them — ADR-0021's "small, finite, strictly ordered" shape resolves to
  exactly two named tiers.
- Any resolution request below 360×640-equivalent (long-edge capped, aspect preserved) is a
  refusal, never a silently-returned degraded frame. `render` is unaffected — this floor
  applies only to `preview`.
- The refusal result carries a message naming the floor and the legibility reason, not a bare
  failure.
- Content-dependence of the floor (legibility tracks smallest on-screen text, not resolution
  alone) is recorded as an open observation, not a mechanism — no per-project or per-instant
  floor is adopted. A future ticket may reopen this if a project's text profile ever makes the
  fixed 360p floor demonstrably too conservative or too loose.
