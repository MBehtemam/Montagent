# Per-element loudness normalisation: PROTOTYPE (#816). Throwaway.

Not production code. `prototype_ab.py` hand-builds the mix graph of
`crates/montagent-core/src/verbs/render.rs` with the proposed member spliced in, and renders a blind
A/B on the shared fixture. Run: `python3 -I prototype_ab.py` (ffmpeg only).

## The shape under test

```json
{ "type": "audio", "audio_effects": [ { "name": "normalize_loudness", "target_lufs": -23 } ] }
```

- Singular (a second one would only compose with the first), `target_lufs` required (no default
  and no per-type presets), range as ADR-0172's `target_lufs`.
- Render: the element's chain up to the member's slot (trim, `atempo`, `aloop`, the members before
  it) is measured with BS.1770 over the element's own placed window; one fixed
  `volume=<target - measured>dB` follows. Never `loudnorm`. Silent, or shorter than one 400 ms
  block: no gain, and a finding.
- The master target (ADR-0172) applies on top, to the summed mix.

## What the clips are

Three elements, 20 s: voice line 1 (fixture, -17.6 LUFS), voice line 2 (the rest of the
narration, 9 dB quieter, as a second TTS call arrives: -28.5 LUFS), and the bed (-33.0 LUFS).
`X.m4a` and `Y.m4a` are the same document, one with the member `enabled: false`, the other with
it on every element (voice lines to -23, bed to -38: 15 LU under the voice, as the fixture is).
Both are brought to -20 LUFS by one gain and encoded AAC 160k. Which is which is in `ab/KEY`:
**do not open it until you have judged.**

Listen for: does the narration hold one level from the first word to the last, and does the bed
sit where you would put it?
