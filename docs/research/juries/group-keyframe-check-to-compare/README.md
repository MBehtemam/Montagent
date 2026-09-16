# Jury: the group keyframe-time check (ticket #71)

[ADR-0039](../../adr/0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md)
claimed this question was "resolved by a jury of three independent models (Opus, Sonnet,
Haiku), unanimous 3/3," but no individual ballots were committed when the ADR was written —
only the synthesized disposition. This directory is that gap closed: a fresh three-model
jury (Opus, Sonnet, Haiku), run in isolation from each other and from ADR-0039 itself, on
[BRIEF.md](./BRIEF.md).

**The re-run is not unanimous.** Opus and Haiku land on ADR-0039's disposition — retract the
check from `validate`, reassign the drift hazard to `compare` — independently re-deriving
most of the ADR's own reasoning. **Sonnet dissents**, arguing for an opt-in, per-property
"motion unit" declaration that stays in `validate` — precisely the shape ADR-0039 lines
58–61 pre-rejects ("an opt-in marker... protects documents whose author already knew the
coupling mattered... precisely the population that doesn't have the bug"). Sonnet's ballot
was written without seeing that counter-argument (the brief withholds ADR-0039's own
reasoning to avoid contaminating the jury) and does not independently arrive at it or rebut
it — so it stands as live, unaddressed dissent, not a ballot that considered and rejected
ADR-0039's point.

Separately, Haiku's ballot — despite agreeing with the disposition — recommends the project
add opt-in schema syntax for declaring motion coupling, which ADR-0039 explicitly declines
("nobody is proposing here"). Opus argues against ever adding such a field, for the same
reason Sonnet's dissent fails: the authors who'd use it are the ones least likely to need it.

## Files

- [`BRIEF.md`](./BRIEF.md) — the self-contained brief given to all three jurors, withholding
  ADR-0039's own conclusion and reasoning.
- [`opus.md`](./opus.md), [`sonnet.md`](./sonnet.md), [`haiku.md`](./haiku.md) — each
  juror's full ballot, verbatim.

## Verdict tally

| | Q1: unit | Q4: cross-property hand-off | Verdict |
|---|---|---|---|
| Opus | A relationship between two keyframe times, evaluable only with a before/after — not `group`, not a narrower opt-in unit | In scope for the same relocated mechanism; falls out for free | Retract from `validate`; property-agnostic "relationship broken" check in `compare` |
| Sonnet | An explicit, opt-in "motion unit" field naming coupled elements/properties | Out of scope; would need its own opt-in mechanism | **Keep a reworked check in `validate`**, gated on the opt-in declaration |
| Haiku | `group` fails; no syntax currently exists to express coupling at all | Belongs to `compare` entirely | Retract from `validate`; split into a `compare` check plus new opt-in schema syntax |

2/3 (Opus, Haiku) agree with ADR-0039's disposition. 1/3 (Sonnet) does not. 1/3 (Haiku), while
agreeing with the disposition, adds a recommendation ADR-0039 declined to make.
