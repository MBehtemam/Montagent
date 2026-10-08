# Hand-off spec: noise gate (ADR-0182, draft)

Written as the bodies of build tickets, sliced like C1-C5 on #846. Build nothing until the owner has ruled the
calls in [ADR-0182 "Owner calls"](../../../adr/0182-the-noise-gate-is-one-stackable-audio-effect-with-a-range-no-hold-and-an-rms-threshold.md)
and moved it out of `proposed`. Depends on **F0** (the `audio_effects` foundation, defined on #845): the
field, its tagged union, `enabled: false`, the singular error, the render hook and the float format. Nothing here
changes the tool or verb counts.

- **G1. Schema.** The `noise_gate` member with five required keys, no defaults, the ranges of ADR-0182 section 1
  (`threshold_db` -80..0, `ratio` 1..100, `attack_ms` 0.1..2000, `release_ms` 1..9000, `range_db` 0..80);
  non-singular (section 1); a keyframe list on any key is a schema error (section 5, #842); `enabled: false`
  honoured. Missing or unknown key: the clean error naming the keys. Test: every bound parses, every
  just-outside value is refused.
- **G2. Render.** The `agate` filter of ADR-0182 section 2, string for string: dB to linear for `threshold` and
  `range`, **`ratio` mapped to (ratio + 1) / 2**, `knee=1:detection=rms:link=average:makeup=1:mode=downward`;
  no `apad...atrim` (the gate has 0 samples of latency, section 2 and #843); placed in the author's order
  before `volume`, in float (ADR-0179 section 2). A document with no gate member renders the graph it rendered
  before (committed string, ADR-0173 section 5); `enabled: false` emits nothing (section 4).
- **G3. `validate`.** The findings in ADR-0182 section 3: `E-GATE-RANGE` (error), `R-GATE-ATTACK-SLOWER`,
  `R-GATE-AFTER-COMPRESSOR` (reviews), `N-GATE-RATIO-1`, `N-GATE-RANGE-0`, `N-GATE-STACKED` (notes). Counts
  consider enabled members only. If the owner takes the note for a threshold above -3 dBFS, it lands here.
- **G4. Repo tests, three-leg table, accept.** Port `check_gate.py` to an integration test through the real mix
  graph with the encoder swapped for PCM (ADR-0182 section 6; ADR-0173 sections 1 and 3): static curve at
  100/100 ms, the timing-bias band for `attack_ms` <= `release_ms`, the timing order, open-is-identity,
  the **0-sample impulse onset and exact length**, the five-command no-op with the `volume` control, range
  edges and the three negative controls, bypass identity and same-build determinism, **and the stereo-link
  case the evidence run lacks**. Run on all three CI legs, commit the per-leg table, replace the provisional
  +/-0.3 dB with max(2 x spread, resolution). Do not flip to `accepted` until the owner's blind listen to
  `gate/ab/` is in `VERDICT.md` (or the owner accepts the numbers alone).
- **G5. Reading tools and docs.** `query` and `compare` describe the member; `format.md`: the RMS convention,
  the n:1 `ratio`, "the threshold reads up to 3 dB high when attack is faster than release", "put the
  threshold at least 12 dB over the noise floor", "a slow attack softens an onset", no hold, and the worked
  example `{ "name": "noise_gate", "threshold_db": -34, "ratio": 10, "attack_ms": 5, "release_ms": 150, "range_db": 30 }`.
  `CONTEXT.md`'s **Audio effect** entry names the member.
