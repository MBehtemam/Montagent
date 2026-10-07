# Which audio filters the ffmpeg floor ships, and which render deterministically in the mix graph

Research for [#797](https://github.com/MBehtemam/Montagent/issues/797), part of the map
[#795](https://github.com/MBehtemam/Montagent/issues/795). The sibling note
[`PRECEDENT.md`](./PRECEDENT.md) (#796) records what CapCut and Premiere do. This note records
what ffmpeg can do.

**Date of research:** 2026-10-07. **This note decides nothing.** It measures and cites, so that
the audio tickets on #795 can argue from it.

**What was run.** The floor itself: BtbN's `ffmpeg-n7.1.5-12-g1fdbca85aa` linux64 GPL static
build from `autobuild-2026-07-31-14-10`. This is the pin in `ci/install_ffmpeg_floor.sh`
(ADR-0115 §7), downloaded and SHA-256-verified against that script's checksum. A second build,
Ubuntu 24.04's `6.1.1-3ubuntu5` (below the floor), was used only as a cross-build comparison.
Both run on x86_64 Linux. **No aarch64, macOS or Windows binary was executed.** Claims about
those targets come from build recipes and from shipped binaries' configure strings, as labelled
below.

**Sourcing.** Every filter fact cites the ffmpeg 7.1 documentation. The online page
[ffmpeg.org/ffmpeg-filters.html](https://ffmpeg.org/ffmpeg-filters.html) tracks git master, so
each claim is pinned to the 7.1.5 tag's `doc/filters.texi` (abbreviated **texi**) or to
`libavfilter/` source at the same tag. The egress proxy refused ffmpeg.org, gyan.dev,
evermeet.cx, formulae.brew.sh and the distro package sites. GitHub (tag sources, Homebrew
formulae, BtbN's build scripts, Gyan's release binaries) was reachable. Tags:
**[measured]** means the check script below asserts it. **[source]** means read in a primary
source at a pinned revision. **[unverified]** means a lead that no primary source reached here
confirms.

**Re-runnable check:** [`check_ffmpeg_audio_filters.py`](./check_ffmpeg_audio_filters.py).
`python3 -I check_ffmpeg_audio_filters.py <ffmpeg>` runs 142 assertions in about 30 s. It exits
1 and names each claim that no longer holds. It passes on the 7.1.5 floor build, and also on
6.1.1, so the numbers below are not specific to one release.

---

## 1. Answer in one table

Mix-graph context: each element's chain is
`aformat → atrim → [atempo…] → [aloop] → atrim,asetpts → [asendcmd,] volume → atrim,asetpts,adelay`,
then `amix=inputs=N:normalize=0 → apad → atrim`, then AAC 160k (`render.rs` `chain`/`mix_graph`,
`encode.rs:300`).

| Filter | In every floor build? | Same bytes run to run (one build) | Bytes depend on SIMD path | Runtime commands (`asendcmd`) | Delay / tail at 48 kHz |
| --- | --- | --- | --- | --- | --- |
| `loudnorm` | yes, built in | yes | **yes** in dynamic mode | **none** | 0 delay. Dynamic mode **resamples to 192 kHz** |
| `alimiter` | yes | yes | no | all options (`limit`, `level_in/out`, `attack`, `release`, `asc*`, `level`, `latency`) | **delay = `attack`** (5 ms → 239 samples), unless `latency=1` → 0 |
| `acompressor` | yes | yes | no | every option (`threshold`, `ratio`, `attack`, `release`, `makeup`, `knee`, `mix`, …); `threshold` verified | 0 |
| `agate` | yes | yes | no | options marked T, but **`threshold` command is a silent no-op** | 0 |
| `equalizer`, `highpass`, `lowpass`, `bass`, `treble` | yes | yes | no | `f`, `t`, `w`, `g`, `m`, `c`, `n` | 0 (IIR phase shift only) |
| `anequalizer` | yes | yes | no | one command, `change` (`fN\|f=\|w=\|g=`) | 0 |
| `superequalizer` | yes | yes | **yes** | **none** | **4095 samples (85 ms)**, length kept, so the last 85 ms is lost |
| `firequalizer` | yes | yes | **yes** (and differs between 6.1 and 7.1) | `gain`, `gain_entry` | 480 samples (10 ms). +960 samples at EOF |
| `pan` | yes | yes | no | **none** | 0 |
| `acrossfade` (24 curves) | yes | yes, all curves | not measured | **none** | exact: `len1 + len2 − d` |
| `afade` (same 24 curves) | yes | yes | — | `type`, `start_sample`, `nb_samples`, `start_time`, `duration`, `curve`, `silence`, `unity` | 0, sample-exact |
| `afftdn` | yes | yes | **yes** | 12 options (`nr`, `nf`, `rf`, `tn`, `tr`, `om`, `ad`, `fo`, `nl`, `sn`, `gs`) | **1200 samples (25 ms)**, last 25 ms lost |
| `anlmdn` | yes | yes | no | `s`, `p`, `r`, `o`, `m` | **384 samples (8 ms)** at default `p`/`r` |
| `deesser` | yes | yes | no | **none** | 0. **Its default `i=0` is a pass-through** |
| `aecho` | yes | yes | no | **none** | 0 delay. Adds a tail of max(delays) |
| `afir` + in-graph IR (reverb with no IR file) | yes | yes, **if `anoisesrc` has a `seed`** | not measured | `dry`, `wet`, `ir` | 0 delay. No tail past input end |
| `rubberband` | **no**. Needs `--enable-librubberband`, which Homebrew's default `ffmpeg` lacks | yes | no | `tempo`, `pitch` | not sample-exact (≈ −320 samples for pitch, ≈ +240 for tempo, measured) |
| `asetrate,aresample,atempo` (pitch) | yes | yes | **yes** | `atempo` `tempo` only | not sample-exact (≈ −740 samples), ~1550 samples short |
| `atempo` (speed, pitch kept) | yes | yes | no | `tempo` | not sample-exact (≈ −640 to −830 samples at onset), length off by tens of ms |

"Same bytes run to run" means byte-identical f32 PCM across two runs and across
`-filter_threads 1` vs `8`. It also means byte-identical AAC (`-bitexact`) when the filter
is inserted into a Montagent-shaped two-element mix chain and run twice **[measured, §B]**.
Determinism held for every candidate. The risks are elsewhere: latency, silent command no-ops,
build-to-build drift, and one optional library.

---

## 2. Availability on the six targets

### 2.1 Everything but `rubberband` is unconditionally built in [source]

In 7.1's `configure`, the only candidates with a `_deps` line are `pan_filter_deps="swresample"`
(always present in an ffmpeg CLI build), `rubberband_filter_deps="librubberband"`,
`ladspa_filter_deps="ladspa libdl"` and `lv2_filter_deps="lv2"`
([configure@n7.1, lines 3890–3946](https://github.com/FFmpeg/FFmpeg/blob/n7.1/configure#L3890)).
`loudnorm`, `alimiter`, `acompressor`, `agate`, the biquads, `anequalizer`, `superequalizer`,
`firequalizer`, `acrossfade`/`afade`, `afftdn`, `anlmdn`, `deesser`, `aecho`, `afir`,
`asetrate`, `atempo`, `asendcmd` and `asetnsamples` have no external dependency. They are present
in any build that does not explicitly `--disable-filter` them. All are present in the floor
build **[measured, §A]**.

`librubberband` sits in `EXTERNAL_LIBRARY_GPL_LIST` beside `libx264`
([configure@n7.1 L1861–1875](https://github.com/FFmpeg/FFmpeg/blob/n7.1/configure#L1861)). So
the floor's `--enable-gpl` requirement makes it licence-compatible, but nothing makes it present.

### 2.2 Which common builds carry `librubberband`

| Build | Targets | `rubberband`? | How established |
| --- | --- | --- | --- |
| BtbN GPL (the CI floor pin), `linux64` | Linux x86_64 | **yes** | [measured] `-filters` lists it; configure string has `--enable-librubberband` |
| BtbN GPL `linuxarm64`, `win64`, `winarm64` | Linux aarch64, Windows x86_64/aarch64 | **yes** | [source] [`scripts.d/50-rubberband.sh`](https://github.com/BtbN/FFmpeg-Builds/blob/master/scripts.d/50-rubberband.sh): `ffbuild_enabled` returns false **only** for `lgpl*` variants |
| BtbN LGPL variants | same | no (and no libx264, so below the floor anyway) | same script |
| Gyan 7.1 *essentials* and *full* | Windows x86_64 | **yes** (both) | [measured] configure strings of `ffmpeg.exe` in [GyanD/codexffmpeg 7.1](https://github.com/GyanD/codexffmpeg/releases/tag/7.1) release zips. *full* also has `--enable-ladspa` |
| Homebrew **`ffmpeg`** (default formula, now 9.0.2) | macOS x86_64/arm64 | **no** | [source] [Formula/f/ffmpeg.rb](https://github.com/Homebrew/homebrew-core/blob/main/Formula/f/ffmpeg.rb) (main, read 2026-10-07): its 11 runtime deps and `--enable-*` list do not include rubberband |
| Homebrew `ffmpeg@7` (7.1.5), `ffmpeg-full` (9.0.2) | macOS | **yes** | [source] [ffmpeg@7.rb](https://github.com/Homebrew/homebrew-core/blob/main/Formula/f/ffmpeg@7.rb) L55/L116, [ffmpeg-full.rb](https://github.com/Homebrew/homebrew-core/blob/main/Formula/f/ffmpeg-full.rb) L55/L119 |
| evermeet.cx | macOS **x86_64 only** (no Apple Silicon build) | [unverified] | site blocked. A search extract of [evermeet.cx/ffmpeg/apple-silicon-arm](https://evermeet.cx/ffmpeg/apple-silicon-arm) states that no arm64 build is planned |
| Debian / Ubuntu `ffmpeg` | Linux | yes on 6.1.1 (below floor) | [measured] Ubuntu 24.04's configure line has `--enable-librubberband`. Debian 13's 7.1 [unverified] |
| Fedora `ffmpeg-free` / RPM Fusion | Linux | conditional | [unverified] search extracts show a `_with_rubberband` spec conditional, switched off in some RPM Fusion configurations |
| Chocolatey `ffmpeg` | Windows | [unverified] which Gyan flavour it repackages. Both Gyan flavours have it |

**Consequence.** `rubberband` is **not** on the floor as ADR-0115 states it. The most common
macOS install, `brew install ffmpeg`, lacks it. That is also what CI's macOS leg installs (ADR-0115
§7, "the top"). So a graph that names `rubberband` would fail on that leg. Shipping it would
mean either adding `rubberband` to the tool qualification (a fourth argument, failing
`E-TOOL-UNSUPPORTED` on stock Homebrew) or making it optional with a declared fallback.

`ladspa`/`lv2` (the route to Freeverb-style plugins) need plugin files on disk. Even where
they are compiled in (BtbN has `lv2`, Gyan *full* has `ladspa`), they are not a stock reverb.

---

## 3. Determinism

### 3.1 Within one build: every candidate is byte-identical [measured, §B]

On the floor build every candidate in §1 produced identical f32 PCM across two runs and across
`-filter_threads 1` and `8`. Through the mix chain it also produced identical AAC 160k
(`-fflags +bitexact -flags:a +bitexact`, ADTS). The filters with slice threading (`S` in
`ffmpeg -filters`: the biquads, `afftdn`, `anlmdn`, `afir`, `dynaudnorm`) split work per
channel, so the thread count does not change the bytes. All 24 `acrossfade` curves are
deterministic. (`qsin2` and `hsin` give identical output because sin²(πt/2) = (1−cos πt)/2.)

**One trap:** `anoisesrc`'s `seed` defaults to −1, meaning random. Two runs of an unseeded
`anoisesrc` differ **[measured]**. A synthetic reverb IR built from it must pin `seed`.

### 3.2 Across builds and CPUs: not guaranteed [measured]

Running each filter with `-cpuflags 0` (SIMD off) changes the output bytes of `loudnorm`
(dynamic), `superequalizer`, `firequalizer`, `afftdn` and the `asetrate`/`aresample`/`atempo`
pitch chain. Those filters' bytes therefore depend on which SIMD path the CPU and build take.
x86_64 (SSE/AVX) and aarch64 (NEON) builds should be expected to differ in the last bits for
them **[inference from the measurement; no aarch64 binary was run]**. Between 6.1.1 and
7.1.5 on the same machine, `firequalizer`'s PCM differs, and so does `afftdn`'s when it runs
inside the mix chain. With `rubberband` in the chain, the AAC differs while the PCM does not.
Every other candidate matched bit for bit across the two versions.

So "deterministic" holds per (ffmpeg build, CPU feature set). That is the same footing the
picture already stands on with libx264 across targets. A reference-hash test of a mix that
uses these filters should be pinned to the CI floor build, not compared across OSes.

---

## 4. Runtime commands

### 4.1 The mechanism [source]

*"Some options can be changed during the operation of the filter using a command. These
options are marked 'T' on the output of `ffmpeg -h filter=<name>`."* ([texi L352–358](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L352),
[online: #commands](https://ffmpeg.org/ffmpeg-filters.html#commands)). `asendcmd` sends them on a
time interval ([texi L31200](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L31200),
[online: #sendcmd_002c-asendcmd](https://ffmpeg.org/ffmpeg-filters.html#sendcmd_002c-asendcmd)).
This is how `render.rs` drives keyframed `volume` (ADR-0077 reading 9).

The T-flagged options on the floor build, from `ffmpeg -h filter=<name>`:

- `alimiter`: every option.
- `acompressor`: `level_in mode threshold ratio attack release makeup knee link detection level_sc mix`.
- `agate`: the same set minus `mix`.
- biquads: `f t w g m c n`.
- `afftdn`: `nr nf rf tn tr om ad fo nl sn gs`.
- `anlmdn`: `s p r o m`.
- `afir`: `dry wet ir`.
- `rubberband`: `tempo pitch`.
- `atempo`: `tempo`.
- `adelay`: `delays`.
- `firequalizer`: `gain gain_entry`.
- `anequalizer`: its own `change` command
  ([af_anequalizer.c L616](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_anequalizer.c#L616)).
- **None at all** on `loudnorm`, `pan`, `aecho`, `deesser`, `superequalizer`, `acrossfade`, `asetrate`.

### 4.2 Measured behaviour [measured, §D]

- A command changes the output, deterministically, for `volume`, `equalizer g`, `highpass f`,
  `bass g`, `treble g`, `anequalizer change`, `firequalizer gain`, `acompressor threshold`,
  `alimiter limit`, `afftdn nr`, `anlmdn s`, `atempo tempo`, `dynaudnorm p` and
  `rubberband pitch`.
- **`agate`'s commands are accepted and do nothing.** `agate` registers
  `process_command = ff_filter_process_command`
  ([af_agate.c L239](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_agate.c#L239)),
  which writes the option field. But the derived values (`thres`, knee start/stop) are
  computed only in `agate_config_input` ([L93–107](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_agate.c#L93)),
  and agate never calls that again. (`sidechaingate` does, at L350.) The texi still says
  *"This filter supports the all above options as commands"*. A keyframed gate threshold
  through commands would render as its first value.
- **A command to a filter that does not support it is silently dropped.** `asendcmd` logs
  `Command reply … ret:Function not implemented` only at `-v verbose`, and the run exits 0
  with unchanged output (`pan`, `aecho`, `deesser`, `superequalizer`, `loudnorm`).
  Montagent's spawn rule (ADR-0113/0115 §2) watches exit status, so it would not see this.

### 4.3 Commands land on frame boundaries, not on their timestamp [measured, §E]

`asendcmd` compares the **frame's** start pts with the interval
([f_sendcmd.c L497–500](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/f_sendcmd.c#L497)).
So a command at 1.000 s fires on the first frame that **starts** at or after 1.000 s. With
1024-sample frames, `volume … 0.25` at 1.000 s first takes effect at sample **48128**, not
48000. That is 128 samples (2.7 ms) late, and in general up to one upstream frame late
(21.3 ms for AAC sources, 24 ms for MP3's 1152). With `asetnsamples=n=480` (10 ms frames)
or `n=1` in front of `asendcmd`, it lands at exactly 48000.

**This already applies to today's keyframed `volume`.** `render.rs` puts `asendcmd` straight
after `atrim,asetpts`, so frame sizes come from the decoder and the `atempo`/`aloop` before it.
Whether that matters is the tickets' call. `asetnsamples=n=48000/fps` would put the grid on
Montagent's frame instants, at some per-frame filter overhead.

`afade` is the sample-exact alternative for fades: `afade=t=in:st=1:d=0.5` starts at sample
48000 and reaches full level at 72000, and the fade-out ends at 120000 **[measured]**
([texi L1430](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L1430),
[online: #afade](https://ffmpeg.org/ffmpeg-filters.html#afade)).

---

## 5. Latency, lookahead and tails, and what they do to `atrim`/`adelay`

`render.rs` places an element by `atrim=…,asetpts=PTS-STARTPTS,adelay=delays=<ms>:all=1`.
Every `asetpts=PTS-STARTPTS` throws away timestamp-level compensation, so **only the sample
position matters**. Any filter that emits its first sample late shifts the element's audio
later than the document says. If it also keeps the sample count (most do), the element's last
*delay* samples fall off the end.

Impulse-peak delay (an impulse at sample 24000 of a 4 s, 48 kHz input) and length change
**[measured, §C]**:

| Filter | Delay (samples) | Length change | Why [source] |
| --- | --- | --- | --- |
| `alimiter` (default) | **239** (≈ `attack` 5 ms) | 0 | lookahead buffer `sample_rate × attack` ([af_alimiter.c L376](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_alimiter.c#L376)) |
| `alimiter attack=20` | **959** | 0 | same |
| `alimiter latency=1` | **0** | 0 | *"Compensate the delay introduced by using the lookahead buffer … Also flush … at EOF"* ([texi L2259](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L2259)) |
| `superequalizer` | **4095** | 0 | `winlen = 2^(wb−1) − 1` ([af_superequalizer.c L149](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_superequalizer.c#L149)) |
| `firequalizer` | **480** | **+960** | default `delay=0.01` s ([af_firequalizer.c L134](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_firequalizer.c#L134)) |
| `firequalizer zero_phase=1` | 480 in samples | +960 | compensates by **shifting pts** ([L865](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_firequalizer.c#L865)), which the next `asetpts=PTS-STARTPTS` undoes. Followed by `atrim=start=0,asetpts=PTS-STARTPTS`, the delay is 0 (length +480) |
| `afftdn` | **1200** | 0 | `sample_advance = rate/80`, `window_length = 3 × advance` ([af_afftdn.c L664–665](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_afftdn.c#L664)) |
| `anlmdn` | **384** | 0 | `K + S` = patch 2 ms + research 6 ms ([af_anlmdn.c L130–134](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_anlmdn.c#L130)) |
| `loudnorm` (any mode) | 0 | 0 | the 3 s lookahead is internal. Output pts are re-stamped from input ([af_loudnorm.c L689–716](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_loudnorm.c#L689)) |
| `acompressor`, `agate`, biquads, `anequalizer`, `pan`, `deesser i=0.5` | 0 | 0 | no lookahead |
| `aecho 0.8:0.9:60:0.4` | 0 | **+2880** (= 60 ms delay) | flushes the echo tail at EOF |
| `aecho` 4 taps, max 71 ms | 0 | **+3408** | same |
| `afir` + in-graph IR | 0 | 0 | the reverb tail is **cut** at the input's end |
| `acrossfade d=0.5`, two 4 s inputs | n/a | exactly 7.5 s | |

**A generic, measured compensation.** `apad=pad_len=L,<filter>,atrim=start_sample=L,asetpts=PTS-STARTPTS`
removes a fixed delay *L* and keeps the full length. It gives 0 delay and 0 length change for
`afftdn` (L=1200), `superequalizer` (4095) and `anlmdn` (384) **[measured]**. The *L* values
are functions of the 48 kHz bus rate and the filter's options, so Montagent could compute
them from the document, as it already does for `aloop`'s sample count.

**Tails.** Effects with a tail (`aecho`, a reverb) inserted *before* the element's
`atrim=end=…` are cut at the element's end. If a tail should ring past `end`, the effect has
to sit after that cut, with explicit `apad`. That is a modelling choice for the tickets.

### 5.1 Pitch and tempo are not sample-exact [measured, §F]

A 440 Hz burst starts at 0.5 s. This is where its onset lands, relative to an exact
time-stretch of the input:

| Chain | Onset error (samples) | Length (4 s in) |
| --- | --- | --- |
| `atempo=1.25` | −829 | 153797 vs 153600 exact (file input). 152832 with lavfi input: depends on frame sizes |
| `atempo=0.8` | −643 | short by ~1500 |
| `asetrate=60476.21,aresample=48000,atempo=0.793701` (+4 st) | −737 | −1550 |
| `rubberband=tempo=1.25` | +236 | **exactly 153600** |
| `rubberband=pitch=1.259921` (+4 st) | −319 | 192000 |

Neither route preserves sample placement to better than ~5–17 ms. `rubberband` is built in
**real-time mode** (`RubberBandOptionProcessRealTime`,
[af_rubberband.c L135](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_rubberband.c#L135)).
The filter never queries the library's start delay, and at EOF it sets the status without
draining the stretcher (L170–175). ffmpeg 7.1 exposes no R3 "finer" engine option
([texi L6184](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L6184),
[online: #rubberband](https://ffmpeg.org/ffmpeg-filters.html#rubberband)). The existing
`atempo` speed path already carries this error today (story 97). This is an observation, not a
regression.

On quality, which ffmpeg's docs do not grade: `rubberband` is a phase vocoder with formant
options (`formant=preserved`, `pitchq`). `asetrate+atempo` changes formants with pitch
(the "chipmunk" effect) **[unverified as a listening claim; standard DSP knowledge]**.

---

## 6. Per-filter notes

- **`loudnorm`** ([texi L5818](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L5818), [online: #loudnorm](https://ffmpeg.org/ffmpeg-filters.html#loudnorm)).
  - **Dynamic mode.** Dynamic mode forces the filter's I/O to 192 kHz double
    ([af_loudnorm.c L736–750](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_loudnorm.c#L736)).
    Without a following `aresample=48000` the stream leaves at 192000 Hz **[measured]**.
  - **When linear mode applies.** Linear mode is used only when all four `measured_*` are
    given, `measured_tp + (I − measured_I) ≤ TP` and `measured_LRA ≤ LRA` (L806–815).
    Otherwise it silently falls back to dynamic. It also becomes linear by itself when the
    whole input is shorter than 3 s (L446–460).
  - **Two-pass linear vs one-pass on the test clip.** Two-pass linear on the test clip hit
    −16.0 LUFS exactly. One-pass dynamic reached −15.4 **[measured, `print_format=summary`]**.
  - **Two-pass means two ffmpeg runs.** For the mix bus, the first run is a full render of the
    mix. For an element, it is a measurement of that element's chain.
  - No commands. Deterministic. Its dynamic-mode bytes are SIMD-dependent.
- **`alimiter`** ([texi L2259](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L2259)). Use `latency=1` in the mix, or every limited element moves 5 ms late.
- **`acompressor`** ([texi L474](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L474)) and **`agate`** ([texi L2066](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L2066)). No lookahead. The agate command defect is in §4.2.
- **Biquads** `equalizer`/`highpass`/`lowpass`/`bass`/`treble` ([texi L5008](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L5008), [L5522](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L5522), [L5882](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L5882), [L4074](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L4074), [L7324](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L7324)). These are the cleanest candidates: 0 delay, SIMD-independent, identical on 6.1 and 7.1, and every parameter is a command.
- **`anequalizer`** ([texi L2560](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L2560)). A parametric EQ with per-channel bands. Its output is `A->N` only when `curves=1`. Its command is `change`, not the option names.
- **`superequalizer`** ([texi L7024](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L7024)). 18 fixed bands, 85 ms latency, no commands, SIMD-dependent. The weakest EQ for this graph.
- **`firequalizer`** ([texi L5153](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L5153)). Linear-phase FIR. 10 ms latency. Its commands exist although the texi lists none.
- **`pan`** ([texi L6073](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L6073)). Static matrix, no commands. A keyframed pan would need per-channel `volume` commands or `stereotools`, whose `balance_in`/`balance_out`/`mpan` are T-flagged on the floor build (their effect under commands not measured).
- **`acrossfade`** ([texi L572](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L572), [online: #acrossfade](https://ffmpeg.org/ffmpeg-filters.html#acrossfade)). Takes exactly two inputs and fades the **end of the first** into the start of the second. It shortens the result by `d`. Its curves are `afade`'s 24 ([af_afade.c L460–486](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/libavfilter/af_afade.c#L460)). Montagent's graph is one chain per element summed by `amix`. The same crossfade is `afade=t=out` on one chain plus `afade=t=in` on the other, overlapped by `adelay`. That form is sample-exact, keeps the one-chain-per-element shape and has runtime-changeable parameters.
- **`afftdn`** ([texi L1552](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L1552)) and **`anlmdn`** ([texi L2654](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L2654)). Both have fixed latency (25 ms / 8 ms), compensable by §5's pad-and-cut. `afftdn` bytes are SIMD- and version-dependent.
- **`deesser`** ([texi L4711](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L4711)). The default `i=0` is a pass-through **[measured]**, so an effect "deesser" with no parameters does nothing. No commands.
- **`aecho`** ([texi L1199](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L1199)). Multi-tap delays (`delays`/`decays` lists) are the only built-in reverb-like effect that needs no IR. No commands. Its tail extends the stream by the longest delay.
- **Reverb without an IR file.** ffmpeg 7.1 has no algorithmic reverb filter: no `freeverb` or `areverb` in the floor build's `-filters`. The two file-free routes are:
  - `aecho` with several taps;
  - `afir` with an IR synthesised in the graph, such as
    `anoisesrc=d=0.6:c=pink:seed=1,afade=t=out:…:curve=exp[ir];[x][ir]afir`. This is
    deterministic, has 0 delay and supports `dry`/`wet` commands. Its tail is cut at the
    input's end ([texi L1820](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L1820), [anoisesrc texi L8086](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L8086)).
- **`rubberband` vs `asetrate`+`atempo`** ([texi L6184](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L6184), [L3171](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L3171), [L3700](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/doc/filters.texi#L3700)). Covered in §2.2 and §5.1. `rubberband` gives exact output *length* for tempo and has runtime `tempo`/`pitch` commands, but it is not on the floor. `asetrate`+`atempo` is always available, has no runtime pitch, and is SIMD-dependent through `aresample`.

## 7. Side findings for the map (not this ticket's to fix)

1. **Keyframed `volume` lands up to one source frame late** (§4.3). This is today's behaviour,
   and `asetnsamples` fixes it.
2. **`agate`'s commands are no-ops in ffmpeg 7.1.5**, contrary to its documentation (§4.2).
   This matters if a gate is ever keyframed.
3. **Commands to filters without command support fail silently** with exit 0 (§4.2). A
   generator must know each filter's T-flagged set rather than trusting ffmpeg to complain.

## Files

- [`check_ffmpeg_audio_filters.py`](./check_ffmpeg_audio_filters.py). Re-derives §§1–5's
  measured claims (142 assertions). Run `python3 -I check_ffmpeg_audio_filters.py <ffmpeg>`.
  It needs numpy. It passed on BtbN 7.1.5 (floor) and Ubuntu 6.1.1, both x86_64. Its last
  line reports the SIMD-sensitive set (§3.2).
