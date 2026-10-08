# Echo and reverb: PROTOTYPE (map #795, tail rule #843). Throwaway.

Not product code, and nothing here is accepted. `prototype_echo_reverb.py` builds, by string, the graphs a
renderer would build for two candidate `audio_effects` members and renders them through the pipeline's
settings (48 kHz stereo, AAC 160k) as loudness-matched blind A/B clips on the shared narration-over-bed
fixture. The owner's ear decides; the numbers below are recorded next to it
([ADR-0173](../../../adr/0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md)).

Run (needs ffmpeg and numpy): `python3 -I prototype_echo_reverb.py ab` writes `ab/`, then
`python3 -I prototype_echo_reverb.py check [--ffmpeg2 OTHER]` writes `measurements.json`. The committed
files were made with ffmpeg 7.0.2 (the nearest build to the 7.1 floor available here); see "Builds".

## What the clips are

The narration element's source ends where the last word ends, 19 425 ms (`silencedetect`), and the
element's `end` is that instant. So an echo or reverb has nothing after `end` to ring into and the render
truncates it (the latency and tail rule of ticket #843; the repo's ADR-0143 file is a different, render-speed ADR, so cite by ticket). The bed runs to 20 s underneath, unchanged, at the fixture's own level
(the voice is the thing under test, and the bed is 15 LU below it). Every clip is brought to -20 LUFS by one
gain and encoded AAC 160k, so loudness is not a clue (decoded: -20.0 to -20.1 LUFS, all eight).

Settings were chosen to be clearly audible, not to be good defaults:
echo `delay_ms 300, feedback 0.5, mix 0.5`; reverb `decay_ms 1200, mix 0.3`.

| Pair | X vs Y | Question for the ear |
|---|---|---|
| `ab/echo-X.m4a`, `echo-Y.m4a` | the bypassed source vs the echo (order hidden) | Is the echo audible and usable on speech? Does the last word sound cut off? |
| `ab/reverb-X.m4a`, `reverb-Y.m4a` | the bypassed source vs the reverb | Does it sound like a room, or like noise smeared on the voice? Is the end abrupt? |
| `ab/echo-tail-X.m4a`, `echo-tail-Y.m4a` | the echo cut at `end` vs the same echo with 1500 ms of tail kept | Is the cut tail audible at all? Which would you want? |
| `ab/reverb-tail-X.m4a`, `reverb-tail-Y.m4a` | the reverb cut at `end` vs the same with 1500 ms of tail kept | Same question. |

The two tail pairs are 21 s long (both clips of a pair, so length is no clue); the kept-tail clips are the
named departure of a literal `tail_ms`, built here only so the owner can hear what the rule costs
(`apad` before the effect, cut `tail_ms` later). **Do not open `ab/KEY` or the `what` fields of
`measurements-ab.json` until you have judged.** KEY sha256 `88dc2f56...30d6`
(full: `88dc2f560f7a31efb505e608d0e5e919786505a5cf912c6b18613b94fc8530d6`). The owner's words go in
`VERDICT.md`, verbatim, with date, build and the KEY's hash.

## The two candidates

**Echo** renders with ONE `aecho in_gain=1 out_gain=1`. Taps at k x `delay_ms`; tap k has gain
`mix x feedback^(k-1)`; taps stop below -60 dB or at 8. The dry path stays at unity, so
`x + mix * (taps)` is exactly a dry/wet crossfade of `x` against `x + taps`, and `mix 0` is the bypass
(byte-identical, measured). For the clips: 300, 600, 900, 1200, 1500, 1800, 2100, 2400 ms at 0.5, 0.25, ...,
0.0039. `aecho` is a tap line (no recursion), accepts no runtime commands, and clips nothing here
(mix peak 0.74).

**Reverb** is a generated impulse response, no file: two decorrelated seeded pink-noise channels
(`anoisesrc c=pink seed=11` and `seed=12`), an exponential envelope reaching -60 dB at `decay_ms`
(`aeval`, per sample), convolved by `afir`, in an `asplit`/`amix normalize=0` dry/wet:
`(1-mix) * dry + mix * wet`. All literal. Three traps found, each of which the renderer must carry:

1. **`afir`'s `dry` and `wet` are an input gain and an output gain, not a mix.** `dry=0` is silence and
   `dry=1:wet=0` is silence. The dry path is a separate branch.
2. **`afir` normalises the IR itself, and the builds disagree.** 6.1.1 honours `gtype`; 7.x added `irnorm`
   (default 1), and with it `gtype=gn`, `rms`, `peak` and `none` all rendered the same level (-55 dB RMS on
   the narration) on 7.0.2, against -14.6 dB for `gn` on 6.1.1. So the graph switches normalisation off
   (`irnorm=-1:gtype=none` on 7.x, `gtype=none` on 6.1) and scales the IR itself:
   `K = C / (sigma * sqrt(rate * T / (6 ln 10)))`, `T = decay_ms / 1000`, `sigma = 0.0970` (the measured RMS
   of `anoisesrc c=pink a=0.5`, seeds 3, 11 and 12 within 0.7 %). With `C = 1` the wet-only RMS is +7.0 dB
   over the narration's (+7.07, +6.98, +6.92 dB at 600, 1200, 2400 ms), so `C = 0.447` makes wet equal dry.
   The level is **spectrum-dependent**: pink noise has a low-frequency-heavy response, so the same IR gives a different wet level for a
   different input spectrum (a white-noise burst came out tens of dB under what speech does, in a
   preliminary run with the 6.1 `gn` normalisation). The calibration is for speech, and is a constant, not a law.
3. **The IR's energy is analytic, not measured per render.** Both builds produce identical IR bytes.

## Measured (ffmpeg 7.0.2 unless said; `measurements-ffmpeg7.0.2.json`, 36 of 36 claims; `measurements-ffmpeg6.1.1.json`, 34 of 36)

| Check | Echo | Reverb |
|---|---|---|
| Onset latency, lavfi impulse at sample 24000 | **0 samples** (`mix` 0.5 and 1.0); length exact | **0 samples** (`mix` 0.3 and 1.0); length exact. Round-off before the onset: up to 2.8e-9 (-171 dB) |
| Length when cut at `end` | exact | exact |
| Length with `tail_ms` 1500 kept | exact (end + 1500 ms) | exact |
| Energy past `end`, tail kept (220 Hz burst in the last 100 ms) | -30.0 dBFS RMS | -34.9 dBFS RMS |
| Cut render is the kept render's prefix | bit for bit | within 4.5e-8 |
| No NaN/Inf, no clipping (mix, float) | peak 0.740 | peak 0.625 |
| Loudness vs bypass, before matching | -16.8 vs -18.0 LUFS (+1.2) | -20.6 vs -18.0 LUFS (-2.6) |
| Not a no-op: difference RMS re bypass RMS | -5.0 dB | -7.4 dB |
| `mix 0` vs bypass | byte-identical | byte-identical (7.0.2) |
| Same bytes on a rerun, one build | yes | yes |
| Scalar (`-cpuflags 0`) vs SIMD | identical | **differs**, max 6e-8 (-144 dBFS) |

The `afir` pre-onset round-off means a latency assertion for reverb needs a threshold (the check uses
1e-5, -100 dB re the impulse), where the others can assert exact zero.

### What the cut costs on this narration (the question the map left open)

On the real narration element, 19 425 ms end, with 1500 ms kept (`cut_cost.py`; `measurements-ab.json` holds the clip levels):

- **Echo:** the first 100 ms after `end` runs at -29.9 dBFS, **9.5 dB below the element's own RMS**
  (-20.4 dBFS), then -28.9, -36.6, -35.9, -34.9 dB per 100 ms. The cut removes the whole repeat of the last
  words. Energy share only 0.19 %, but it is the part that tells the ear an echo exists. At the cut the
  last 10 ms are -32.9 dBFS (last sample 1.1e-3): a step, not a fade.
- **Reverb:** the first 100 ms after `end` is -51.2 dBFS, **26.9 dB below** the element's RMS, 0.0016 % of
  the energy. At the cut -38.1 dBFS. Far less to lose at these settings; a longer decay or a higher mix loses more.

So the cost is real for echo and small for this reverb. Whether it is audible is the owner's call (pairs 3 and 4).

### Builds (drift)

Two ffmpeg builds were available: Ubuntu 6.1.1 and a static 7.0.2 (`imageio-ffmpeg` 0.6.0's binary). The
7.1 floor and the CI legs were not available; the three-leg table belongs to the capability's build slice.

- **Echo:** byte-identical between 6.1.1 and 7.0.2 over the whole 20 s mix, and scalar vs SIMD.
- **Reverb:** the generated IR is byte-identical across builds. The convolved element is byte-identical up
  to sample 925 696 (= 113 x 8192, the last whole `afir` partition) and, between builds, **the mixed PCM is
  not**: on 6.1.1 the dry branch of `asplit -> amix` loses its last 4 656 samples (97 ms) when the source
  ends, which is exactly the final-word region here (max difference 0.13, -31 dB re signal). That also
  breaks `mix 0` = bypass and the cut = prefix claim on 6.1.1 (the two failures). Standalone `afir` keeps
  the length, so this is an `asplit`/`amix` end-of-stream defect in 6.1.1, absent in 7.0.2. It is a
  reason the bypass-identity and the end-of-source cases belong in the reverb member's check.
- Hashes are not portable: reverb PCM differs by SIMD path even on one build (6e-8), so the check is a
  tolerance, as ADR-0173 already says.

## Proposed member shapes (for the echo/reverb ADR; nothing decided)

Per ADR-0169 (`{name, ...params}`, closed union, `enabled: false` bypass), ADR-0170 (ms may be
fractional, mix and feedback 0..1, no unit suffix on a single-scale quantity) and ADR-0180's "every key
required, no defaults":

```json
{ "name": "echo",   "delay_ms": 300, "feedback": 0.5, "mix": 0.5 }
{ "name": "reverb", "decay_ms": 1200, "mix": 0.3 }
```

| Member | Key | Unit | Proposed range | Note |
|---|---|---|---|---|
| `echo` | `delay_ms` | ms | 1..2000 | Premiere's Delay tops at 2 s (recalled). `aecho` allows 90 000. |
| `echo` | `feedback` | 0..1 | 0..0.95 | The map's draft called it `decay`; ADR-0170 and Premiere say feedback. Renders as up to 8 taps (gain `feedback^(k-1)`), so the ring is bounded at 8 x `delay_ms`, an honest departure from a true recursive feedback. |
| `echo` | `mix` | 0..1 | 0..1 | `mix 0` byte-identical to bypass. |
| `reverb` | `decay_ms` | ms | 100..5000 | Time to -60 dB (RT60). Premiere's Studio Reverb "Decay" is in ms (recalled). |
| `reverb` | `mix` | 0..1 | 0..1 | A true crossfade. |

No level key in dB, so ADR-0170's suffix rule adds nothing. Left out on purpose: reverb pre-delay,
damping, size, width, diffusion (Premiere has all; the generated IR has no early reflections and a fixed
pink tilt, see owner calls). `echo` stackable; `reverb` singular is suggested (one convolution per element,
a second reverb is never the point), with the 8-per-element-style cap an ADR call. Neither takes a tail
key: the tail is cut at `end` unless an owner-accepted prototype admits `tail_ms`.
No parameter is keyframable (ticket #842): `aecho` has no commands, and `afir`'s `wet`/`ir` commands do not touch
the generated IR.

## The cut-tail review, suggested wording

Structural, so `validate` can raise it without hearing anything: an enabled `echo` or `reverb` on an
`audio`/`video` element whose `end` reaches the end of its source with no `loop` (and no later
sound-carrying media past `end`, which `validate` cannot know, so it stays a review, never an error).
Suggested id `R-AUDIO-TAIL-CUT`:

> `audio_effects[1] (echo)`: this element ends where its source ends, so the echo's tail (about 2.4 s at
> `delay_ms` 300, `feedback` 0.5) is cut at `end`. Extend `end` past the last sound, or end the source
> with silence the effect can ring into. There is no tail key.

Tail length for the message: echo about taps x `delay_ms` (to -60 dB, at most 8 x), reverb `decay_ms`.
`R-AUDIO-TAIL-CUT` should name the effect index in ADR-0146's form (`audio_effects[1].delay_ms (echo)`).
Check before it ships that it does not fire on every TTS clip: those always end at their source's end, so
the review would be near-universal on narration, which is the owner's first call below.

## Owner calls / owner must listen

Listen, in this order: `echo-X/Y`, `reverb-X/Y`, then `echo-tail-X/Y`, `reverb-tail-X/Y`.

1. **Is the echo wanted on narration at all, and does the cut tail bother you?** (`echo-tail`.) If the cut
   is acceptable, nothing changes in the format. If not, a literal `tail_ms` per member is the departure
   to admit; this prototype is the evidence the rule asks for, it does not accept it.
2. **Does the generated reverb sound like a reverb?** (`reverb-X/Y`.) It is pink noise with an exponential
   decay: no early reflections, no pre-delay, no damping control, a fixed dark tilt. If it sounds like noise,
   the options are `aecho` multi-tap only (no reverb), more parameters (a pre-delay, a damping low-pass),
   or "no, and why" for reverb. Also say whether 1200 ms / 0.3 is a sensible scale.
3. **Is the reverb worth `afir`'s costs?** A partition-sized (8192 sample, 171 ms) block structure, the
   normalisation difference between 6.1 and 7.x, a SIMD-dependent last bit, and the end-of-source dry-path
   defect seen on 6.1.1. The floor is 7.1, untested here.
4. **`mix` meaning for echo.** Dry stays at unity and `mix` scales the repeats (identical to a dry/wet of
   `x` vs `x+taps`). Premiere's Delay mix is a dry/wet where the dry fades. Keep mine, or fade the dry too?
5. **Feedback as 8 taps, not recursion.** Fine for a speech echo, a departure from a real feedback delay.
6. **Names:** `echo` (the map's, and CapCut's preset name) or `delay` (Premiere's)? `feedback` or `decay`?
7. **Reverb singular?** And the per-element cap of echo/reverb members.
8. **Calibration constant** `C = 0.447` (wet RMS = dry RMS on this narration, within 0.15 dB over
   0.6..2.4 s) is for speech; a different source spectrum changes it by tens of dB. Accept as a stated
   approximation, or make the reverb's level an exact measured property (it would need a per-element
   measurement, like `normalize_loudness`)?
9. **One listener, one voice, one fixture.** Same limits as every prior listen; the narration has no long
   pause, so a tail ringing into silence is only heard in the `-tail` pairs.
10. **Not exercised:** a source with a `loop`, a `speed` other than 1, a source not ending at `end`
    (the common case for music), the 7.1 floor build, and macOS and Windows legs.

## Files

- `prototype_echo_reverb.py`: the builder and the checks (`ab`, `check`).
- `cut_cost.py FFMPEG`: prints the cut-tail numbers above (imports the prototype).
- `ab/echo-X|Y.m4a`, `ab/reverb-X|Y.m4a`, `ab/echo-tail-X|Y.m4a`, `ab/reverb-tail-X|Y.m4a`, `ab/KEY`.
- `measurements-ab.json` (levels per clip, graphs' parameters, tap list; has the labels, so open after judging).
- `measurements-ffmpeg7.0.2.json` (36 of 36), `measurements-ffmpeg6.1.1.json` (34 of 36, the two failures are the defect above).
