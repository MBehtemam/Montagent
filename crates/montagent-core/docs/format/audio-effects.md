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

## Loudness: `normalize_loudness`

```json
{ "name": "normalize_loudness", "target_lufs": -23 }
```

- **One fixed gain** that brings the element's own integrated loudness to `target_lufs`. It
  is measured on the element's placed window: its whole `start`..`end` after the trim,
  `speed` and any loop, at the member's position in the list, so an EQ ahead of it is part
  of what it measures and one after it is not. It never rides the level.
- **`target_lufs`** is required, -40..-6, and has no default. Write the number: dialogue at
  -23, and a music bed 15 LU under the voice at -38.
- **One per element**: a second enabled copy is `E-AUDIO-EFFECT-SINGULAR`.
- `volume` applies after it, so the element ends at `target_lufs` plus `volume` in dB, and a
  `volume` duck moves it on purpose. `master` applies to the sum afterwards.
- A `--from`/`--to` render and `preview` apply the gain the whole window measured.
- `render` reports each member's `measured_lufs` and `applied_gain_db` under `normalized`.

| Finding | Class | When |
| --- | --- | --- |
| `E-NORMALIZE-TARGET-RANGE` | error | `target_lufs` outside -40..-6; the repair is the nearest bound |
| `E-NORMALIZE-NO-AUDIO` | error | the source has no audio stream; drop the member or change the source |
| `R-NORMALIZE-ABOVE-MASTER` | review | `target_lufs` above `master.target_lufs` |
| `R-NORMALIZE-LIFT` | review | at render, the gain applied is above +20 dB: check the source and the target |
| `N-NORMALIZE-UNDEFINED` | note | at render, the window is silent or under 400 ms, so no gain was applied |

**Worked example**: a voice and the music bed under it.

```json
{"id": "voice", "type": "audio", "start": 0, "end": 4000, "source": "audio/voice.wav", "source_start": 0, "source_end": 4000, "audio_effects": [{"name": "highpass", "frequency_hz": 100, "slope_db_per_oct": 24}, {"name": "normalize_loudness", "target_lufs": -23}]}
{"id": "music", "type": "audio", "start": 0, "end": 4000, "source": "audio/music.wav", "source_start": 0, "source_end": 4000, "audio_effects": [{"name": "normalize_loudness", "target_lufs": -38}]}
```

`query --at` prints `audio_effects[1] normalize_loudness -23 LUFS` on the voice's row.

## Noise reduction (ADR-0181)

`reduce_noise` is a **singular** member over `anlmdn` that reduces steady hiss:

- **`reduce_noise`**: `{"name": "reduce_noise", "reduction_db": <5..20>}`. `reduction_db` is
  required, static and has no default (`E-NOISE-RANGE` repairs to the nearest bound in 5..20 dB).
  It maps to `anlmdn` strength `s = 10^(-2 + (reduction_db - 5) / 32)` (6 significant digits)
  with fixed `p=0.002:r=0.006:o=o` and `apad=pad_len=384,atrim=start_sample=384` to cancel the
  384-sample internal latency.
- **What `reduction_db` promises**: the dial is calibrated on white noise at −50 dBFS RMS, where
  it realises within ±1.0 dB across 5..20 dB while preserving speech within ±0.5 dB. Because
  `anlmdn`'s strength is a level threshold, the realised dB drop on real hiss varies with the
  hiss level.
- **Order**: place `reduce_noise` before `compressor`. Putting `reduce_noise` after an enabled
  `compressor` raises `R-NOISE-AFTER-COMPRESSOR`, because the compressor lifts and modulates the
  noise floor ahead of `anlmdn`.

## Echo (ADR-0185)

`echo` is a **stackable** discrete-tap delay over one `aecho` stage whose tail is truncated at the
element's `end`:

- **`echo`**: `{"name": "echo", "delay_ms": <1..2000>, "feedback": <0..0.95>, "mix": <0..1>}`.
  All three keys are required, static and have no defaults (`E-ECHO-RANGE` repairs to the nearest
  bound).
- **Tap law and tail**: taps `k = 1..=8` sit at `k * delay_ms` with gain `g_k = mix * feedback^(k-1)`,
  stopping at the first tap below `0.001` (−60 dB). The dry path stays at unity (`in_gain=1:out_gain=1`),
  and the output is cut at `end - start` (`atrim=end=<duration_s>`). When `mix < 0.001`, no tap
  survives, no filter is emitted (byte-identical to omitting the member), and `N-ECHO-NO-OP` notes it.
  When the 8th tap exceeds `0.01` (−40 dB), `N-ECHO-TAPS-CAPPED` notes that later audible taps are
  dropped. When the product of `1 + sum(g_k)` across enabled `echo` members on an element exceeds
  `3.0` (+9.54 dB), `R-ECHO-HOT` advises lowering `mix`/`feedback` or following with a `limiter`.

## Routing and stereo balance: `channels` and `pan` (ADR-0184)

`channels` and `pan` are flat, optional, static fields on `audio` and `video` elements (never
inside `audio_effects`, and never keyframable):

- **`channels`**: `"stereo" | "swap" | "mono" | "left" | "right"` (default `"stereo"`), applied
  after `aloop` and before `audio_effects` so `normalize_loudness` measures the routed signal.
  `"swap"` swaps L and R; `"mono"` sums `0.5 * (L + R)` into both channels; `"left"` fills both
  channels from L; `"right"` fills both channels from R. Explicit `"stereo"` emits no filter and
  fires `R-CHANNELS-INERT`; a non-stereo mode on a probed 1-channel source fires
  `R-CHANNELS-ON-MONO-SOURCE`.
- **`pan`**: `-1..1` (default `0`), applied after `volume` and before transition gain as a linear
  0 dB-centre balance — the near channel stays at unity (`1.0`) and the far channel scales by
  `1 - |pan|` (`E-PAN-RANGE` outside `-1..1`; explicit `0` emits no filter and fires
  `R-PAN-INERT`).
