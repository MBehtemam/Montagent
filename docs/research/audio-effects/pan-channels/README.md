# Pan/balance and channel operations: measurements and a recommended shape

Research for the later-wave capability on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)). **This note decides nothing and is not an
ADR.** It gives the capability ADR its numbers, a recommended shape and the calls that are the owner's.
No product code changed. Read against ADR-0169 ("Where routing sits"), ADR-0170 ("Pan is -1..+1"),
ADR-0173 (measured check), ADR-0077 (48 kHz stereo bus), ADR-0055, `PRECEDENT.md` rows 2-3 and
`FFMPEG-FILTERS.md`.

Date: 2026-10-08. Run on the CI floor build `n7.1.5-12-g1fdbca85aa` (via `ci/install_ffmpeg_floor.sh`) and
on distro `6.1.1-3ubuntu5`. Both pass every claim below.

## Files

- `measure_pan_channels.py`: every number in sections 1-5 (`python3 -I measure_pan_channels.py [ffmpeg] [--json out] [--dump-pcm dir]`). Exits 1 naming any claim that stops holding. Needs numpy.
- `measure_slot_loudness.py`: the loudness shift each routing op causes, which decides the slot (section 6). Stdlib.
- `measurements.json`: the floor build's output of the first script.

All measurements are f64le PCM straight off the graph (no encoder), from lavfi tones and seeded noise, per
ADR-0173 section 3. The bus is stereo, so every source reaching `pan` is L/R.

## What the bus does today (a fact to carry into the ADR)

`aformat=channel_layouts=stereo` on a **mono** source puts it on L and R at **-3.0103 dB each** (swresample
uses 0.7071 for the centre). That is already a constant-power centre. It is existing behaviour, not something
this capability changes. ffmpeg's own stereo-to-mono (`aformat=channel_layouts=mono`) is the same 0.7071(L+R):
**+3.01 dB on a centred source**, which can clip. So `mono` must be an explicit matrix, never `aformat`.

## 1. Pan law choices (stereo bus, centred L=R source, level re the unpanned source)

| law | gains at p=0 | p=+-0.5 (near / far) | p=+-1 (near / far) | total power at 0 / 0.5 / 1 |
|---|---|---|---|---|
| balance, linear far channel `1-|p|` (0 dB centre) | 0 / 0 dB | 0 / -6.02 dB | 0 / silent | 0 / -2.0 / -3.0 dB |
| balance, quarter-cosine far channel (0 dB centre) | 0 / 0 dB | 0 / -3.01 dB | 0 / silent | 0 / -1.2 / -3.0 dB |
| constant power `cos/sin((p+1)pi/4)` (-3 dB centre) | -3.01 / -3.01 | -0.69 / -8.34 dB | 0 / silent | -3.0 at every p |
| linear `(1-+p)/2` (-6 dB centre) | -6.02 / -6.02 | -2.50 / -12.04 dB | 0 / silent | -6.0 / -5.1 / -3.0 dB |

Reading it:

- **Only a 0 dB centre keeps `pan: 0` identical to no pan.** The -3 and -6 dB centres change the level of a
  clip the moment a pan key is written, even at 0. That breaks "an absent field is today's render" and shifts
  `normalize_loudness` outcomes by 3-6 dB. A cap of 1.0 on every gain means a 0 dB-centre law cannot also be
  constant-power: it dips by up to 3.0 dB at the edges (both balance tapers; the cosine dips less in the middle).
- Constant power is the textbook law for a *mono source panned in a stereo field*. On this bus the source is
  already L/R, and Premiere's Panner on a stereo clip is a balance (section 7), so balance is the match.
- A balance **never moves content between channels**: with L=440 Hz and R=880 Hz, balance +1 leaves R
  bit-identical and L exactly zero (440 Hz in R < -100 dB). A mixing pan matrix does put 440 Hz in R. That is
  the difference between "balance" and "pan a stereo image", and the recommendation is balance only.
- Literal rendering: `pan=stereo|c0=<gl>*c0|c1=<gr>*c1`, the gains written as decimals. Identity
  `pan=stereo|c0=c0|c1=c1` is bit-identical in float, but the renderer should emit no `pan` at all at 0.
- `pan` syntax trap, measured: `=` is literal; **`<` normalises the gains** (divides by their sum when it
  exceeds 1). `c0=c0+c1` on L=R gives +6.02 dB, `c0<c0+c1` gives 0 dB. Use `=` only.

## 2. Mono downmix level

| fold | correlated L=R | uncorrelated noise | left-only | anti-phase L=-R |
|---|---|---|---|---|
| `0.5(L+R)` | 0.00 dB | -2.98 dB | -6.02 dB | exact silence |
| `0.7071(L+R)` (= ffmpeg `aformat` mono) | +3.01 | +0.03 | -3.01 | silent |
| `(L+R)` | +6.02 | +3.04 | 0.00 | silent |

`0.5(L+R)` is the only fold that cannot raise a clip: unity on a dual-mono file (the TTS case), -3 dB on
uncorrelated material, never clips. Its cost is -6 dB on a one-channel recording, which is what `left` / `right`
are for. Note a mono fold of a polarity-inverted pair is silence: a candidate review.

## 3. Swap, left only, right only

All bit-exact in float on both builds: swap puts R on L and L on R; `left` copies L onto both channels
(`pan=stereo|c0=c0|c1=c0`); `right` likewise. The "mute the other channel" form (`c1=0*c0`) is exact zero and
is just balance at -1, so the vocabulary needs only the **fill** forms (Premiere's Fill Left with Right /
Fill Right with Left). On a lav-mic file (left live, right empty), `left` returns a centred signal at the live
channel's level. Two-channel loudness moves with each op (section 6).

## 4. Latency, length, drift, keyframes

- **Latency 0, length exact.** An impulse at sample 100 lands at sample 100 through balance, swap and mono; the
  output length is the input's (48000/48000). Nothing for `apad...atrim` to cancel.
- **Build drift: none observed.** SHA-256 of the f64 PCM of six renders (balance, constant-power, mono fold of
  tones, swap, fill, mono fold of seeded noise) is identical between 6.1.1 and 7.1.5, and the noise fold is
  identical with `-cpuflags 0` (SIMD off). Repeat runs are identical. `pan` is already in
  `FFMPEG-FILTERS.md` as zero-latency and keyframe-dropping; this adds the cross-build equality.
- **`pan` ignores `asendcmd`** (measured: output equals a render with no command). A keyed pan has exactly one
  path that works: `channelsplit` + per-channel `volume` + `asendcmd` + `join`, measured to silence L at 0.5 s while R
  stays up. That is a five-filter graph per element, which fails the cost-cap criterion in the animatable-parameters
  ruling (#842). **Static in the first slice; a keyframe list on `pan` is a schema error** (same rule as #842).

## 5. Build drift summary and tolerance

Because gains are exact in float and the three-leg spread is zero in everything measured, ADR-0173 section 4's tolerance
rule (max of 2 x spread, meter resolution) lands on the print precision: **0.01 dB** for gains, **exact** for
swap/fill/zero. macOS Homebrew and Windows legs were not available here; the table must be committed with
those legs before the ADR flips to accepted.

## 6. Where channel operations go: before `audio_effects` (measured support)

Integrated loudness change from each op (ebur128, 5 s, 1 kHz):

| source | `left` fill | `mono` 0.5(L+R) | `swap` | balance +1 | balance +0.5 linear |
|---|---|---|---|---|---|
| left-only (lav mic) | **+3.00 LU** | **-3.00 LU** | 0 | silent | -6.00 LU |
| centred L=R | 0 | 0 | 0 | -3.00 LU | -2.00 LU |

A routing op can move loudness by several LU. `normalize_loudness`, the compressor threshold and the limiter
all read level, so they must see the signal after the op, or the author's `target_lufs` is met on a signal
that is then re-levelled. That supports ADR-0169's leading candidate: **channel ops run after `aloop`, before
the `audio_effects` list.** The reverse holds for pan: it is a placement control that should come after the
list and `volume`, so a duck and a normalise are never re-levelled by it (ADR-0169). Full chain:

`atrim -> atempo -> aloop -> [channels] -> [audio_effects] -> volume -> [pan] -> [transition gain, ADR-0176] -> adelay -> amix`

Ordering caveat: `channels: "left"` then `pan: -1` gives left audio on L only; that composition is well defined
and needs no rule.

## 7. Premiere precedent: confirmed vs recalled

This sandbox's proxy blocks helpx.adobe.com (WebFetch returned EGRESS_BLOCKED), so **nothing here was re-opened
against Adobe's pages**; `PRECEDENT.md`'s labels stand and are not upgraded.

| claim | status in `PRECEDENT.md` |
|---|---|
| Panner/Balance is a clip fixed effect and a track mixer control, keyframable | CONFIRMED (search extract of the page) |
| Range -100..+100, 0 centre, negative left | CONFIRMED (extract) |
| Balance rescales L/R levels in place | CONFIRMED (Effects library extract) |
| **Pan law** | **not documented** anywhere found |
| Fill Left with Right, Fill Right with Left, Swap Channels (stereo only), Invert, Channel Volume | CONFIRMED (extract) |
| Fill/Swap/Invert have no parameters | CONFIRMED |
| Channel Volume is in dB | CONFIRMED |
| Audio Channels source-to-clip mapping matrix | RECALLED |
| Which direction "Fill Left with Right" copies | **Adobe's own text contradicts the name; resolve by testing, not by quoting.** This note's `left`/`right` are defined by this repo's own measurement (section 3), not by Adobe's wording |
| CapCut | nothing first-party found for either row |

Departures from Premiere to name in the ADR: -1..+1 instead of -100..+100 (ADR-0170, already ruled); a flat
`channels` word instead of separate effects; no Invert and no Channel Volume (neither is requested; Channel Volume
would be per-channel `_db`, a later separate member); no keyframes in the first slice; no track-level pan (ADR-0004).
Precedent is Premiere only, so the pan law is a choice, not a copy.

## 8. Recommended schema shape

**Flat fields on `audio` and `video` elements, not `audio_effects` members**, per ADR-0169's test (routing is
flat; one correct position; a list would invite meaningless orders and duplicates).

```json
{ "type": "audio", "src": "narration.wav", "channels": "left", "pan": -0.3, "volume": 1 }
```

- **`pan`**: number, `-1..1`, `0` centre, negative left (ADR-0170). Literal. Static (no keyframes) in the first slice.
  Meaning: **balance on the stereo bus**; far channel gain `1 - |pan|`, near channel unity. Law: owner call 1.
  Absent and `0` both mean no `pan` filter in the graph (byte-identical to today).
- **`channels`**: closed string vocabulary `"stereo" | "swap" | "mono" | "left" | "right"`. `"stereo"` is the default
  and means no filter. Meanings (exact matrices):
  - `swap`: `c0=c1, c1=c0`
  - `mono`: both outputs `0.5*c0+0.5*c1`
  - `left`: both outputs `c0` (right is dropped)
  - `right`: both outputs `c1` (left is dropped)
- Rendered with `pan=` with `=` (never `<`), gains as shortest decimals, after `aformat` has put every source on the
  stereo bus, so the matrix is always 2-in, 2-out.
- Slot: `channels` after `aloop`, before `audio_effects`; `pan` after `volume` (section 6).
- A `video` carries both too (ADR-0169: the embedded track).
- Not enabled/bypass flagged: flat fields have no `enabled`; omitting is the bypass.

### `validate` checks

Error class:
- `pan` not a finite number, or outside -1..1 (so an agent writing Premiere's `50` fails, per ADR-0170).
- `channels` not one of the five strings.
- `pan` or `channels` given as a keyframe list.

Review class (ids are placeholders; the ADR names them and keeps them out of ADR-0173's verify):
- `channels` other than `"stereo"` on a source probed as mono (inert: the bus already doubled it).
- `channels: "left"|"right"` on a source whose chosen channel is silent needs a probe; probably out of scope.
- `pan` with `channels: "mono"` is allowed (mono then balance); no finding.
- Explicit `pan: 0` or `channels: "stereo"` is inert (spelled like the default). One review of the same family as
  `R-EASE-INERT`, or accept as harmless: owner call 5.

## 9. Measured acceptance check (conforms to ADR-0173)

Repo integration test, port of `measure_pan_channels.py`, PCM before encode, lavfi tones and seeded noise,
floor build on all three legs. Metric: per-channel RMS in dB (RMS over the last 3/4 of 1 s, f64) and sample
equality. One-sided where a ceiling, otherwise two-sided at 0.01 dB. Expected values come from the matrix definition.

| id | assertion | tolerance |
|---|---|---|
| P1 | pan p in {-1,-0.5,0,0.5,1}: near channel 0 dB, far channel `20log10(1-|p|)` dB re source | 0.01 dB |
| P2 | pan=+-1: far channel max abs sample == 0 | exact |
| P3 | balance never mixes: L=440, R=880, pan=+1 -> right bit-identical to source right | exact |
| C1 | swap / left / right: output channels equal the specified source channels | exact |
| C2 | mono on L=R: 0.00 dB; on seeded uncorrelated noise: -3.0 dB; on L=-R: exact zero | 0.01 / 0.15 / exact |
| C3 | mono on left-only: -6.02 dB re the live channel | 0.01 dB |
| L0 | impulse at sample 100 lands at 100, length unchanged, for pan and each channels value | exact |
| B1 | `pan: 0`, `channels: "stereo"`, both absent: same PCM bytes and same committed filtergraph string | exact |
| B2 | a document with no new field: the committed expected graph string is unchanged | exact |
| S1 (proposed, not run here; needs `normalize_loudness` built) | slot: left-only source + `left`, then `normalize_loudness`: output within 0.1 LU of target (shows the op is seen first) | 0.1 LU |

The ADR flips to accepted only with the three-leg table committed (ADR-0173). The owner's ear: a short blind
A/B on the shared narration fixture for `pan: -0.5`, which is the one claim numbers do not fully settle.
The ear is not claimed here (one measurement run, no listening).

## 10. Owner calls

1. **Taper of the balance law**: linear far channel `1-|p|` (recommended: explainable, `-6.02 dB` at 0.5, simplest
   for exact-string editing) or quarter-cosine (`-3.01 dB` at 0.5, gentler: total dip -1.2 vs -2.0 dB at 0.5, but
   not as literal). A constant-power -3 dB centre is not recommended (it re-levels by 3 dB at `pan: 0`).
2. **One `pan` (balance) only, or also a true stereo pan** that can move content across channels. Recommended: balance only; a
   mixing pan is a later separate member if wanted.
3. **`mono` fold gain**: `0.5(L+R)` (recommended, never clips, -6 dB on one-channel files) vs `0.7071(L+R)` (ffmpeg's own,
   +3 dB on dual-mono, can clip).
4. **Vocabulary and names**: `channels` with `stereo|swap|mono|left|right`; whether `left`/`right` should be fill (recommended,
   Premiere's Fill ops, and what a lav-mic fix needs) or leave the other side silent (that is `pan: -1`/`1`).
   Whether `Invert` and per-channel `Channel Volume` stay out (recommended).
5. **Inert-spelling findings** for explicit `pan: 0` / `channels: "stereo"`: review, or none.
6. **Wave**: the map puts pan and channel ops in "later". Both are one slice each with no new engine concept beyond one
   `pan` filter; they can ship together or `channels` first (it is the lav-mic enabler and the one with a loudness slot decision).
7. **Keyframed pan** (autopan, a move across the picture): deferred under #842; a five-filter graph is the only working
   path today. Say yes only with a tolerance on the cost cap.
8. **Slot confirmation**: before the list for `channels` (measured support, section 6), after `volume` for `pan`.
9. Whether the ADR may rely on the macOS and Windows legs matching (not measurable here).
