# Swaps: the evidence ADR-0140 rests on

[ADR-0140](../../../adr/0140-an-image-element-changes-its-file-over-time-through-timed-swaps.md)
admits an image's `swaps`. This directory holds what its numbers come from, copied from the
throwaway branch `prototype/swaps` (`7fd08e26`), which also holds the prototype renderer,
the owl fixtures and the pixel checks
([#614](https://github.com/MBehtemam/Montagent/issues/614)).

- `P-edit-error.md`: the change request the agents were given
  ([#597](https://github.com/MBehtemam/Montagent/issues/597)).
- `seeds/`: the project each spelling started from. `group.hypothetical.json` is the split
  inside a container that carries the push-in. It is a shape
  [#632](https://github.com/MBehtemam/Montagent/issues/632) has not decided, and no build
  reads it.
- `runs/`: the final project of each of the nine runs, and `scores.txt`, `pip_score.py`'s
  verdicts verbatim. Landing was checked through the prototype binary's `query --at` at every
  painted frame, so it can't be re-derived on `main`.
- `edit_size_check.py`: re-derives every changed-line count in ADR-0140, for both the runs
  and the minimum edit by hand, and exits non-zero if one stops reproducing.

The screenshots aren't copied. No count reads them.
