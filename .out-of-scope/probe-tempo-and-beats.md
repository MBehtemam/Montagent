# `probe` reading audio tempo (BPM), downbeats, and beat grids

This project does not add tempo (BPM), downbeat, or onset beat-grid estimation
to `montagent probe`. Audio beat-grid extraction lives permanently in
`skills/montagent-craft/scripts/beat_grid.py`, which analyses the audio track
offline and emits frame-snapped beat grids and project/element `markers`
(`"kind": "beat"`, [ADR-0230](../docs/adr/0230-project-and-element-markers-enter-the-format-and-chapter-markers-export-to-the-container.md)).

## Why this is out of scope

- **`probe` is fast, deterministic, and metadata-focused.**
  [ADR-0011](../docs/adr/0011-tool-surface-reads-checks-renders.md) and
  [ADR-0092](../docs/adr/0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md)
  (together with [`.out-of-scope/probe-frame-timestamps.md`](probe-frame-timestamps.md))
  keep `probe` focused on container/stream facts (dimensions, frame rate, sample
  rate, channels, duration, loudness) rather than heuristic DSP guesses.
- **Tempo and downbeat detection are heuristic estimates with confidence bounds.**
  Onset comb filtering and bar-phase estimation can land on a half/double-time
  octave or tie across beats (`tempo_confidence`, `downbeat_confidence`,
  `downbeat_by`). That interactive inspection and override loop (`bpm`,
  `downbeat`, `range` in `beat_grid.spec.json`) belongs in the authoring skill
  script `skills/montagent-craft/scripts/beat_grid.py` (alongside
  `keyframes.py` and `duck.py`), not in `validate`/`probe`'s cached sidecar.

## Prior requests

- #547 — "Flag: Montagent can't read a track's tempo or beats, so agents analyse the music themselves"
