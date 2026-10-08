---
status: accepted
amends: 0061 (its fenced exception speaks of an "externally-sourced" threshold and admits a number only with a cited source; a number the project measured on its own committed fixture is admitted too, on four conditions, and `source` names the fixture and the script)
---

# ADR-0061's fence admits the project's own measurement as a source, on four conditions

[#854](https://github.com/MBehtemam/Montagent/issues/854), on the map
[#560](https://github.com/MBehtemam/Montagent/issues/560) (does `validate` check word timings
against the audio on disk?). That map chose a silence level and a minimum pause for a speech-timing
check ([#567](https://github.com/MBehtemam/Montagent/issues/567)): −40 dB and 100 ms. No
standard gives that pair. It is the pair at which the brief B music bed reads as one unbroken
sound ([#561](https://github.com/MBehtemam/Montagent/issues/561)), and, as a positive control,
the pair at which the check fires on the late words file and stays silent on the repaired
projects ([#855](https://github.com/MBehtemam/Montagent/issues/855)). It is the project's own
measurement, and [ADR-0061](0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md)
did not say whether such a number may decide a finding.

Decided by `/court` (three jurors: Opus, Sonnet, Haiku), and the human ruled to go with the
Judge's read.

## The decision

### 1. Amend ADR-0061; do not read it as already admitting this. Split 2–1

ADR-0061 admits a deciding number that is not derivable from the document only when its
source is cited, at `review` or `note`, with the raw measurement as the finding's substance.
Its examples are published guidance (Netflix, BBC). Whether a number the project measured
itself is "cited" is a reading the text does not settle. The fence exists to stop judgment
passing as fact. A threshold chosen by the party that writes the check meets the third
condition in form only, so it is not read in. Opus and Sonnet for amending; Haiku held that
it was already admitted.

### 2. The −40 dB / 100 ms pair is a fenced threshold. Unanimous

It is not a fixed parameter of a deterministic reader. [ADR-0051](0051-word-alignment-is-external-validate-and-compare-catch-drift.md)
governs who may measure, not whether a number that decides a finding is fenced. The pair
decides whether a pause exists, so it decides whether the finding fires. It is fenced:
`review` or `note` only, the raw measured fact as the substance, the source stated inline.

### 3. A project's own measurement is admitted on four conditions

A check may compare a document-derived fact against a number that comes from the project's own
measurement, if and only if it satisfies ADR-0061's three conditions **and**:

1. **The fixture is committed.** The audio, project or other input the number was measured on
   is in the repository, at a path the registry names.
2. **The measurement re-runs as committed.** A script in the repository re-derives the number
   from the fixture with no input but the fixture and a stated tool version, and asserts it.
3. **The finding says what the number is.** The inline source states that the number is the
   project's own measurement on that fixture, not a standard and not published guidance.
4. **The check's own ADR records what the fixture stands for, and where that stops being
   true.** Which take, which voice, which platform and tool versions, and what the fixture
   does not cover (a real microphone, two voices at once, another platform).

A number the project measured on a fixture that is not committed, or that no script re-derives,
is not admitted.

### 4. The registry's `source` names the fixture path and the script. Unanimous

For a check admitted under this rule, `source` is the fixture path and the script that
re-derives the number. The ticket that chose the number belongs in the check's ADR as history,
not in `source`. `ThresholdProvenance::External`'s `source` field already carries a string;
a variant that marks the provenance as the project's own measurement is the implementing
check's change, not this ADR's, and it keeps the registry test (never `error`, always cites)
unchanged.

## What this does not change

ADR-0061's test, its three conditions and its binding citation rule stand for every check that
borrows from outside the project. This ADR adds one admission route and takes none away: a
published number still needs a published source. `R-CAPTION-PACE` and
`R-CAPTION-MIN-DURATION` are unaffected. No check is admitted by this ADR; the speech-timing
check's own ADR (the end of #560) is the first to use the route, and it must meet conditions 1
to 4 itself.
