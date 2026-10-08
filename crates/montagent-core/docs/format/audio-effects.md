# The Montagent project format: audio effects

A page of `montagent://format.md`, which holds the rules every element shares and lists the
other pages; read it first. This page holds `audio_effects`, the ordered list on an `audio` or
`video` element that shapes its sound. Every rule here is an accepted decision in the ADR
series, cited inline by number, and the ADR is right where the two disagree.

## The list

- **`audio_effects` is an ordered list of members, each `{"name": ..., ...}`** (ADR-0169), the
  last key of the element. A `video` carries it too and it applies to the embedded sound.
  Members run in list order, after `speed` and any loop and before `volume`, so a `volume`
  duck is never undone by a member. `speed` does not scale a frequency.
- **Its vocabulary is its own.** A visual member in `audio_effects`, or an audio member in
  `effects`, is `E-AUDIO-EFFECT-WRONG-LIST`, naming the right list.
- **Every key is required and none defaults**, and a level is in dB with the unit in the key.
  Numbers are literals: a keyframe list is a schema error.
- **`"enabled": false` bypasses a member** without losing its values: it is still checked for
  range, does not render, and is reviewed (`R-AUDIO-EFFECT-DISABLED`) as a leftover.
- The list runs in 32-bit float from its first enabled member, so a steep filter keeps its
  stopband. A project with no members renders as it always did.

## EQ: `highpass`, `lowpass`, `shelf`, `bell`

All four are stackable: a high-pass and three bells is ordinary.

```json
{ "name": "highpass", "frequency_hz": 100,  "slope_db_per_oct": 24 }
{ "name": "lowpass",  "frequency_hz": 8000, "slope_db_per_oct": 12 }
{ "name": "shelf",    "side": "high", "frequency_hz": 4000, "gain_db": 3 }
{ "name": "bell",     "frequency_hz": 1500, "gain_db": -6, "q": 1.4 }
```

- **`frequency_hz`**, 20..20000: a pass filter's cutoff, where it reads -3.01 dB at every
  slope; a shelf's corner, where it has half its gain; a bell's centre.
- **`slope_db_per_oct`** on a pass filter is `12`, `24` or `48`. **`side`** on a shelf is
  `"low"` or `"high"`. **`gain_db`** on a shelf or bell is -24..24. **`q`** on a bell is
  0.1..10 (higher is narrower). A shelf's Q is fixed.
- A bell or shelf at `gain_db` 0 changes nothing (`N-EQ-NO-OP`).

| Finding | Class | When |
| --- | --- | --- |
| `E-EQ-RANGE` | error | a value outside its range, or a slope not 12, 24 or 48; the repair is the nearest legal value |
| `E-EQ-STACK-CAP` | error | more than 8 enabled EQ members on one element; merge bands or delete one |
| `R-EQ-GAIN-EXTREME` | review | `abs(gain_db)` above 12 |
| `R-EQ-BAND-CROSSED` | review | an enabled `highpass` at or above an enabled `lowpass` |
| `N-EQ-NO-OP` | note | a `shelf` or `bell` with `gain_db` 0 |

**Worked example**: take the rumble off a voice and tame a boxy 1.5 kHz.

```json
{"id": "voice", "type": "audio", "start": 0, "end": 4000, "source": "audio/voice.wav", "source_start": 0, "source_end": 4000, "audio_effects": [{"name": "highpass", "frequency_hz": 100, "slope_db_per_oct": 24}, {"name": "bell", "frequency_hz": 1500, "gain_db": -6, "q": 1.4}]}
```

`query --at` prints the list on the element's row: `audio_effects[0] highpass 100 Hz
24 dB/oct`, then `audio_effects[1] bell 1500 Hz -6 dB q 1.4`, and `(bypassed)` after a
disabled member. Change one number and `query` again; a gentle EQ is hard to hear on speech,
so check a change against the numbers, not only by ear.
