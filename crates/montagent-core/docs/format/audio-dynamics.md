# The Montagent project format: compressor and limiter

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds the two dynamics members of `audio_effects`
(ADR-0169): `compressor` and `limiter`. Both are stackable, and **every key is required**: a
missing key is a schema error and a keyframe list on any key is one too (ADR-0170).

```json
{ "name": "compressor", "threshold_db": -24, "ratio": 3, "attack_ms": 20, "release_ms": 250, "makeup_db": 4 }
{ "name": "limiter",    "ceiling_db": -3, "release_ms": 50 }
```

| Member | Key | Range |
| --- | --- | --- |
| `compressor` | `threshold_db` | -60..0 |
| | `ratio` | 1..20 (n:1) |
| | `attack_ms` | 0.1..2000 |
| | `release_ms` | 1..9000 |
| | `makeup_db` | 0..24 (a negative gain is `volume`'s job) |
| `limiter` | `ceiling_db` | -24..0 |
| | `release_ms` | 1..1000 |

## What the numbers mean

- **Levels are RMS dBFS**, so a full-scale sine reads -3.01. Think in peak and you are about
  3 dB off. Above the threshold the output is `threshold + (in - threshold) / ratio +
  makeup_db`; below it, `in + makeup_db`. The knee is hard and fixed.
- **The formula is exact at `attack_ms` = `release_ms` = 100.** A faster compressor reduces
  *deeper* than the formula says, by up to 2.5 dB (1.6 dB at 5 ms / 50 ms) and never less
  than 0.3 dB shallower. It is a property of the filter, not a defect. `attack_ms` is not a
  time constant: a longer attack is slower, that is all.
- **The list runs before `volume`**, so a compressor reads the level before the fader, and a
  `volume` above 1 pushes a limited element back over its ceiling.
- **A mono file is upmixed to the stereo bus at -3 dB a channel before the list runs**, so
  the compressor sees a mono tone 3 dB under its file level; write the threshold 3 dB lower
  for the same effect.
- **`ceiling_db` is a sample peak.** The delivered file's true-peak ceiling is the master's,
  and the two are not the same number. The limiter looks 5 ms ahead; the delay is cancelled,
  so the element's onset does not move. The floor is -24 because `alimiter` refuses lower.

| Finding | Class | When |
| --- | --- | --- |
| `E-DYNAMICS-RANGE` | error | a key outside its range; the repair is the nearest bound |
| `E-LIMITER-ABOVE-MASTER` | error | `master.ceiling_dbtp` is set and `ceiling_db` is above `ceiling_dbtp - 1`; the repair is `ceiling_dbtp - 1` |
| `R-DYNAMICS-ORDER` | review | an enabled `compressor` after an enabled `limiter` |
| `R-COMPRESSOR-MAKEUP-CLIP` | review | `makeup_db` above `max(0, (-3 - threshold_db) * (1 - 1/ratio))`, with no enabled `limiter` after it and no `master.ceiling_dbtp` |
| `N-COMPRESSOR-RATIO-1` | note | `ratio` is 1: the member is a plain gain |
| `N-LIMITER-STACKED` | note | more than one enabled `limiter` on one element |

**Worked example**: even out a voice, then hold it under -3 dBFS.

```json
{"id": "voice", "type": "audio", "start": 0, "end": 4000, "source": "audio/voice.wav", "source_start": 0, "source_end": 4000, "audio_effects": [{"name": "compressor", "threshold_db": -24, "ratio": 3, "attack_ms": 20, "release_ms": 250, "makeup_db": 4}, {"name": "limiter", "ceiling_db": -3, "release_ms": 50}]}
```

`"enabled": false` bypasses a member and renders as if it were absent. `query --at` prints
each member on the element's row.
