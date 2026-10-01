# Research: which ffmpeg facts about speech timing catch a late words file?

Research for [#561](https://github.com/MBehtemam/Montagent/issues/561), on the map
[#560](https://github.com/MBehtemam/Montagent/issues/560) (does `validate` check word timings
against the audio on disk?).

**Date:** 2026-10-01.

**Method:** measurement on the files in this repo, not reading. Every number below comes from
`docs/research/speech-timing-ffmpeg-facts/measure.py`, which runs ffmpeg's own filters on the
presenter take and compares the result with the highlight windows in the eval runs' project
files. What each filter measures is taken from the official FFmpeg filter documentation
(`doc/filters.texi`, published as [ffmpeg-filters](https://ffmpeg.org/ffmpeg-filters.html)) and
checked against each filter's C source, at tags `n7.1` and `n9.0.2` of
[FFmpeg/FFmpeg](https://github.com/FFmpeg/FFmpeg). No aligner or speech model was run or read.

**ffmpeg builds used** (Homebrew, macOS 27.0, arm64, Apple clang 21.0.0):

- `ffmpeg version 9.0.2` (`/opt/homebrew/bin/ffmpeg`)
- `ffmpeg version 7.1.5` (`/opt/homebrew/opt/ffmpeg@7/bin/ffmpeg`), the project's floor series
  (ADR-0115)

An 8.0.1 install is also on this machine, but it does not start (`libbluray.3.dylib` is
missing), and Docker was not running, so no Linux or x86 build was tried.

---

## 1. Verdict

**One candidate fact separates brief B's late words file from the repaired projects. It needs a
borrowed number to do it.**

- **The fact.** After a pause that ffmpeg detects in the voice's source audio, sound resumes at
  *b*, but no highlight window is lit at the painted instants from *b* until a later window
  opens. Here it is called **unlit speech after a pause** ("head-unlit").
- **On brief B.** It fires at "Now" in all 7 projects that kept the words file's "Now" at
  2420 ms. Sound resumes at 2301.5–2313.8 ms, depending on the detector settings, and the 3
  painted instants 2333, 2367 and 2400 show speech with nothing lit. This holds with every
  sample-level and RMS detector tried: 15 `silencedetect` settings, 3 `silenceremove` settings
  and 3 `astats` settings.
- **On the repaired projects.** It also fires on both with-skills projects, once each,
  at "This". Sound resumes at 8353–8357 ms and the window opens at 8400, so 1 painted instant
  (8367) is unlit. The captions script left that one alone on purpose: it only moves a start
  more than 60 ms after the speech.
- **The split.** As a plain fact (≥ 1 unlit instant), it fires on **2 of 2** with-skills
  projects: 1 of their 5–6 inner pauses each. Only a count threshold separates the two sets:
  late "Now" is 3 instants and the worst repaired case is 1, so "≥ 2 painted instants"
  (≈ 44–67 ms) fires on 7 of 7 late projects and 0 of 2 with-skills projects. That "2", or the
  captions script's 60 ms, is a borrowed number. Under ADR-0061 it can only be reported at
  `review`/`note`, with a citation, and no source for it was looked for here.
- **The other candidates.** None separates late from correct:
  - *window opens inside a pause*: the late windows open **after** the pause, not inside it;
  - *window lit inside a pause*: fires on natural pauses within and between words;
  - *speech left unlit before a pause*: fires slightly more on the repaired projects;
  - `ebur128`: its 400 ms window puts the "Now" onset at 2400 ms.

**Read per heard source, the same fact fires on the music bed.** The bed has short gaps
before its bars. At −30 dB, or at −35 dB with a 50 ms minimum, those gaps make the bed "resume"
at 2000 and 8000 ms. That gives 8–12 unlit painted instants on **every** B project, late and
repaired alike, which is more than the voice's late "Now". At −35 dB/0.15 s and −40 dB/0.1 s,
the bed has no gaps and nothing fires (§4.1). So which source fires depends on the borrowed
level. **No project in the runs has two voices at once**, so crosstalk could not be measured.

**Deterministic?** `silencedetect`, `silenceremove`, `astats` and `aspectralstats` gave
byte-identical results on 7.1.5 and 9.0.2, and so did the decoded audio (MD5
`1278d03762ada1a1e04f3d278680760c`). `ebur128` did **not** (§6). Only two versions on one
platform were tested. FFmpeg's own AAC decoder tests allow decoded samples to differ by up to 2
LSB (§6), so bit-exactness across platforms is reasoned, not shown.

**Caveat on the sample.** The "correct" set is 2 projects, built from the same take, the same
words file and the same captions script. That script repaired the timings against
`silencedetect` itself (−35 dB, 150 ms), so measuring those projects with `silencedetect` is
partly circular. The false-fire numbers here are therefore thin.

---

## 2. The candidate signals (part 1)

All candidates read the **decoded samples** of one audio stream. Each one's output is a
function of those samples, its parameters and the stream's timestamps. Whether decoding itself
is the same across versions and platforms is the open part, covered in §6.

| Filter | What it measures (official docs; source) | Parameters | Gives a time? | Same on 7.1.5 and 9.0.2? |
|---|---|---|---|---|
| `silencedetect` | Logs `silence_start` / `silence_end` when "the input audio volume is less or equal to a noise tolerance value for a duration greater or equal to the minimum detected noise duration". The source compares each **sample** with `-noise < x < noise`: strictly less, per sample, with no window and no averaging. It counts consecutive quiet samples across all channels, unless `mono=1`. `silence_end` is the timestamp of the first sample at or above the level. | `noise`/`n` (default −60 dB), `duration`/`d` (default 2 s), `mono` | Yes, to the sample (1/16000 s here) | Identical. The 7.1 → 9.0.2 source diff only changes the logging context and the struct layout; the comparison is unchanged. |
| `silenceremove` | Trims silence. It detects silence with a moving-window statistic: `detection=avg/rms/peak/median/ptp/dev`, default `rms`, over a `window` that defaults to 20 ms. It emits **no timestamps**. With `timestamp=copy` and 1 ms input frames (`asetnsamples=n=16`), the kept frames' pts show where audio was kept. | `start_*`/`stop_*` periods, duration, threshold, silence, mode; `detection`; `window`; `timestamp` | Only indirectly (kept spans), to the input-frame size | Identical (template source unchanged) |
| `astats` | Time-domain statistics per frame, or per `reset` frames: RMS level, peak level, min/max, crest, zero crossings and more, written as `lavfi.astats.*` metadata when `metadata=1`. The block length comes from the input frames (`asetnsamples`), and `length` (default 50 ms) only sets the RMS peak and trough windows. | `metadata`, `reset`, `length`, `measure_perchannel`/`measure_overall`; block size via `asetnsamples` | Yes, per block (10 ms used here) | Identical |
| `ebur128` | EBU R128 loudness. It applies a K-weighting pre-filter and an RLB filter. *Momentary* loudness `M` is taken over 400 ms, short-term `S` over 3 s, and both are updated every 100 ms. `metadata=1` cuts the input into 100 ms frames carrying `lavfi.r128.*`. | `metadata`, `framelog`, `peak`, `dualmono`, `panlaw`, `target`, `gauge`, `scale`, `integrated`, `range` | Yes, but on a 100 ms grid, and each value looks back 400 ms | **Not identical**: 3 of the 97 `M` values differ (e.g. −155.347 against −157.766 LUFS). All are below −120 LUFS. Between 7.1 and 9.0.2 the filter's biquads were rewritten into a DSP context (with x86 assembly), and the cache layout changed. |
| `aspectralstats` | Frequency-domain statistics per frame and channel (mean, variance, centroid, spread, skewness, kurtosis, entropy, flatness, crest, **flux**, slope, decrease, rolloff), from a windowed FFT. | `win_size` (32–65536; default 2048), `win_func` (default `hann`), `overlap` (default 0.5), `measure` | Yes, per hop (16 ms used here: 512-sample window, 0.5 overlap) | Identical (only slice bookkeeping changed) |
| `volumedetect` | Whole-stream mean and peak volume and a histogram | none | **No**: one figure for the whole stream | not measured; not a timing signal |

Filters that were **excluded**:

- `asr` (PocketSphinx) and, new in 9.0.2's documentation, `whisper`. Both are speech-recognition
  models, which ADR-0051 and the map rule out. Neither is in the local 9.0.2 build's filter list.
- Video-side signals such as `scdet`. They read the picture, not the speech.

---

## 3. Brief B (part 2)

`take-1.mp4` has one AAC-LC audio stream: mono, 16 kHz, 9.792 s. The presenter is a synthetic
text-to-speech avatar (assets `README.md`), and its pauses are **digital silence**: `astats`
gives an RMS of `-inf` per 10 ms block until 2280 ms. That is why every threshold from −30 to
−50 dB finds nearly the same boundaries. A real microphone recording, with room noise, would
not be this forgiving.

### Where sound resumes before "Now" (words file: `Now` 2420–2440)

| Signal and settings | Pause before "Now" (ms) |
|---|---|
| `silencedetect` n=−30 dB (d = 0.05/0.1/0.15) | 1799.9 – **2313.8** |
| `silencedetect` n=−35 dB | 1845.5 – **2309.9** (the figure in ADR-0134) |
| `silencedetect` n=−40 dB | 1845.6 – **2303.2** |
| `silencedetect` n=−45 dB | 1848.6 – **2302.9** |
| `silencedetect` n=−50 dB | 1850.9 – **2301.5** |
| `silenceremove` rms, 20 ms window, −35/−40/−45 dB, d=0.1 | kept again from **2314 / 2313 / 2306** |
| `astats` 10 ms RMS < −35/−40 dB, ≥ 100 ms | ends **2310** |
| `astats` 10 ms RMS < −45 dB, ≥ 100 ms | ends **2300** |
| `ebur128` M < −40 / −50 LUFS, ≥ 100 ms | 2200 – **2400** |
| `ebur128` M < −60 LUFS | 2300 – 2400 |

The 10 ms RMS envelope (`astats`) over the onset, in dBFS:

```
2270 -inf | 2280 -115.6 | 2290 -78.3 | 2300 -40.5 | 2310 -27.1 | 2320 -24.5 | 2330 -22.7
2340 -19.7 | 2350 -18.3 | 2370 -17.4 | 2390 -16.6 | 2400 -16.6 | 2420 -17.3
```

By 2400 ms, one painted frame before the window opens, the voice is already at its loudest
level of the word. So the lateness is real, and it is not a quiet consonant onset.
`aspectralstats` flux is 0 until the 2304 ms hop and high from 2352 to 2528 ms; its largest hop
in 1700–2600 ms is 2416 ms (0.0365). A flux peak shows where the spectrum changes most, not
where sound starts, so it is not an onset fact by itself.

### What the issue's two examples show

- **"A pause ending at 2309.9 ms, with no window opening until 2420":** shown by every
  sample-level and RMS detector. As unlit speech after a pause, the painted instants 2333.3,
  2366.7 and 2400.0 (30 fps) fall in detected speech with no window lit: **3 instants**. With
  `astats` < −45 dB it is 4, because that run's pause ends at 2300.
- **"Windows opening inside detected pauses":** **does not** show the lateness. A late window
  opens *after* the pause. The windows that do open inside pauses are sub-millisecond
  boundary cases in both sets: in the words file, "checks" at 5621 against a pause ending at
  5621.3 and "and" at 7164 against 7164.5. In with-skills, "Now" opens at 2300 against a pause
  ending at 2303.2.

The same detectors also show the words file late at "This" (8400 against 8352–8357, +44 to
+47 ms) and "Montagent." (+88 to +90 ms). Against `silencedetect` −35 dB/0.15 s, the pause-to-start gap
for each word after a pause is: Now **+110**, checks −4, and −5, This **+44**, Montagent. **+88**.

---

## 4. False fires on the repaired projects (part 3)

**Which projects.** Every eval run whose project has `highlight` windows was listed (script,
`LATE`/`CORRECT`):

- **brief B, the presenter's words:** 7 no-skills projects and 2 with-skills projects;
- **brief A:** 7 launch-spot projects, each with one emphasis window over a music bed only.

Brief A's windows are not timed to any speech, and the document declares no link to the music,
so they are out of the map's scope (ADR-0006) and were not scored.

**Their audio is all available locally.** Every source the B projects reference
(`presenter/take-1.mp4`, `screen/session.mp4`, `music/bed-120bpm.wav`) is in
`docs/research/skills-eval/assets/`. In every B project the audible presenter element maps
document time 1:1 to source time (`start = source_start = 0`, no `speed`). The script asserts
this, so windows compare directly with the take's timestamps.

**One no-skills project is not late at "Now".** `baseline-superseded/.../no-skills-1` moved
"Now" to 2300 itself. It behaves like the with-skills projects throughout. The 6 other no-skills
projects, and the words file, keep "Now" at 2420–2440.

### Fires per project

The table shows `silencedetect` n=−40 dB, d=0.1 s (the pack aligner's setting) and, in
brackets, n=−35 dB, d=0.15 s (close to the captions script's −35 dB/150 ms). An *inner pause*
lies between the first and the last window.

| Candidate fact | Late project (words file, 6 no-skills) | with-skills-1 | with-skills-2 |
|---|---|---|---|
| inner pauses | 6 (5) | 6 (5) | 6 (5) |
| **unlit speech after a pause**, ≥ 1 painted instant | 2 pauses: Now (3), This (1) | **1**: This (1) | **1**: This (1) |
| **unlit speech after a pause**, ≥ 2 painted instants | **1**: Now | **0** | **0** |
| window opens inside a pause | 2 (2) | 1 (2) | 1 (2) |
| window wholly inside a pause | 0 | 0 | 0 |
| speech left unlit before a pause (≥ 1 instant) | 3 (3) | 3 (3) | 3 (3) |
| window lit inside a pause (≥ 1 instant) | 2 (2) | 3 (3) | 3 (3) |

**Across all 24 detector settings** (aggregates in the script's output):

- **The unlit-speech maximum is the same everywhere.** Late projects reach 3 instants (4 with
  `astats` < −45 dB) and with-skills projects reach 1, for every `silencedetect`,
  `silenceremove` and `astats` setting.
- **`ebur128` gives 1 against 0.** That is not resolution: its 400 ms momentary window delays
  the "Now" onset to 2400 and swallows the "This" onset entirely. A 30–40 ms lateness would be
  invisible to it.
- **The other three facts fire at least as often on the repaired projects as on the late
  ones.** "Speech left unlit before a pause" fires because the aligner's word ends stop before
  the sound decays, which the repair keeps. "Window lit inside a pause" fires on the 126 ms
  pause inside "every" (5979–6105 at −40 dB) and the 172 ms pause inside "is" (8778–8950).
  Those pauses are real and do not make the windows wrong.

### Other words files in the pack (supplementary, correctness unknown)

No project uses these files, and nothing establishes that they are correct. The figures are the
next word's start minus the end of the `silencedetect` pause (−40 dB, 0.1 s), in ms:

| Words file | Gaps |
|---|---|
| `take-2.words.json` | Every +28, so +0, exactly −0, prove **+99** |
| `take-3.words.json` | Montagent **+105**, and +0, the −0, frame **+235** |
| `voiceover.words.json` (TTS, WAV) | Your −78, Check −51, look −47, Same −22, same +12, every −62 |

- **Takes 2 and 3** show the same late-after-pause pattern as take 1. It looks like a property
  of the pack's aligner, which only moves word edges that fall *inside* a pause.
- **The TTS voiceover** starts its words 22–80 ms **before** the sound. On that file,
  unlit speech after a pause would stay silent, and "window opens inside a pause" would fire
  on nearly every phrase.
- **The +235 entries are not lateness to report.** The detector found a pause in the middle of
  what the words file calls one long word ("every" 5840–6340). A pause does not say which word
  boundary it belongs to.

### 4.1 The other heard sources: the music bed

Every project with windows has exactly one other heard source: `music/bed-120bpm.wav` (16 s
WAV, 48 kHz stereo). It sits under the captions in the 9 B projects and under the single
emphasis window (2000–4000 ms) in the 7 A projects. `screen/session.mp4` has no audio stream.

The script reads the bed on its own, as it is on disk: it does not apply the volume envelope
and does not classify the audio. It maps the bed's pauses onto the document through the
element's `start` and `source_start`, and states the same facts against the same windows.
These numbers are reported apart from the voice's.

The bed's own pauses (`silencedetect`, ms in the source file):

| Settings | Pauses |
|---|---|
| −30 dB, 0.05 s | 1894–2000, 3916–4000, 5858–6000, 7922–8000, 9894–10000, 11908–12000, 13886–14053, 15889–16000 |
| −30 dB, 0.1 s | 1894–2000, 5858–6000, 9894–10000, 13886–14053, 15889–16000 |
| −35 dB, 0.05 s | 1947–2000, 5903–6000, 9924–10000, 11931–12000, 13923–14031, 15934–16000 |
| −35 dB, 0.15 s | none |
| −40 dB, 0.05 s | 5941–6000, 13959–14022 |
| −40 dB, 0.1 s | none |

What fires against the bed:

| Settings | B projects (7 no-skills, 2 with-skills): bed | A projects (7): bed |
|---|---|---|
| −30 dB, 0.05 s | every project: **unlit after a pause** at 2000.5 (12 instants; 8 where "Now" is at 2300) and 8000.5 (11); speech unlit before a pause 2; lit in a pause 2; opens in a pause 1 | opens in a pause: 1 in each (the window at 2000 opens 0.5 ms before the bed resumes) |
| −30 dB, 0.1 s | every project: unlit after a pause at 2000.5 (12 or 8); unlit before a pause 1; lit in a pause 1 | opens in a pause: 1 in each |
| −35 dB, 0.05 s | every project: unlit after a pause at 2000.4 (12 or 8); unlit before a pause 1; lit in a pause 1 | opens in a pause: 1 in each |
| −35 dB, 0.15 s | nothing | nothing |
| −40 dB, 0.05 s | every project: lit in a pause, 1 (5941–6000, 2 instants) | nothing |
| −40 dB, 0.1 s | nothing | nothing |

**What this means for the check.**

- **The bed out-fires the voice.** At the settings where the bed has gaps, "unlit sound after a
  pause" fires on the bed in **9 of 9** B projects. It fires with 8–12 instants, well past the
  "≥ 2 instants" cut that separated the voice's late "Now", and the repaired projects fire
  exactly as the late ones do.
- **The check cannot pick the voice.** A check that reads every heard source the same way
  cannot tell these fires from the voice's without knowing which source carries the words.
  The document does not declare that, and classifying the audio would take a model.
- **The borrowed level decides it.** At −35 dB/0.15 s and −40 dB/0.1 s, the bed yields
  nothing, and that is only because no gap in it is quiet and long enough.

### 4.2 Crosstalk (two voices at once)

**None available.** All 37 project files in `runs/` were scanned for heard `audio`/`video`
elements whose source is a pack voice (`presenter/`, `voiceover/`, `character/voice/`).
Every project has at most one, so no two overlap in time. Brief C's projects each have one
voice element and no highlight windows. Crosstalk was therefore not measured. In principle,
two overlapping voices fill each other's pauses, so a per-source reading would still find each
voice's own pauses, while a reading of the mix would find fewer.

### 4.3 Against which audio

Measured against the **rendered mix** (`render.mp4`), the same facts are useless:

- At −40 dB, the music bed fills almost every pause. `with-skills-1` keeps only 1845–2001,
  7865–8001 and 9648–10000 ms.
- Where a pause does survive, it ends on the bed's beat at 2.000 s, not on the voice.

The facts are only meaningful against the voice element's own source audio, mapped through the
element's `source_start` (and speed, if any).

---

## 5. Facts versus thresholds (part 4)

"Plain fact" here means a statement that uses no number from outside the document and the media.

| Candidate | Plain-fact part | Part that needs a borrowed number |
|---|---|---|
| `silencedetect` | "Every sample from *a* to *b* has an absolute value below *L*" is exact and checkable. Only *L* and the minimum run *d* come from outside. Given those, the pause boundaries are facts to the sample. | *L* (noise level) and *d* (shortest pause) are both borrowed. On this take, any *L* from −30 to −50 dB moves the "Now" onset by only 12 ms, because the pauses are digital zero. On the music bed, the same choice decides whether pauses exist at all: 8 at −30 dB/0.05 s, none at −40 dB/0.1 s (§4.1). |
| unlit speech after a pause (built on any of the above) | "Sound resumes at *b*; the painted instants *t₁…tₖ* after *b* show no lit word; the next window opens at *s*". The instants come from the document's `fps`, so beyond the detector's parameters this needs no number. | Deciding to **report** it does: at k ≥ 1 it fires on both repaired projects ("This", 1 instant). Separating late from repaired needs k ≥ 2, or a gap in ms (the captions script uses 60 ms). The fact also equates "sound resumes" with "the word starts", which holds for this take but is an interpretation in general. |
| window opens / is lit inside a pause | "Window *w* opens at *s*, inside the detected pause *(a, b)*" | A tolerance is needed even to ignore sub-millisecond boundary cases. Beyond that, pauses inside and between words are normal, so a lit window inside a pause is not evidence of an error. |
| `silenceremove` | none directly; it is a trimmer, and its kept spans are a side effect | threshold, window (20 ms default), `detection` statistic, durations; and the timing resolution depends on the input frame size chosen |
| `astats` | "The RMS level of samples [*t*, *t*+Δ) is *x* dBFS". With an interval taken from the document (a window, or one frame at the document's fps), no outside number is needed. | the block length when it is not the document's frame, and the level that counts as speech |
| `ebur128` | "Momentary loudness over the 400 ms ending at *t* is *x* LUFS": an exact definition from a standard. | the level that counts as speech, plus a 400 ms look-back that blurs onsets by up to 400 ms. Not bit-stable across versions (§6). |
| `aspectralstats` (flux) | "Spectral flux between hops at *t* is *x*" | window size and function, overlap, and a level or peak rule. Flux marks spectral change, not sound onset; on "Now" its maximum is at 2416 ms, inside the word. |

---

## 6. Deterministic across ffmpeg versions?

**What was tested.** Two versions, one platform: 7.1.5 and 9.0.2, both Homebrew builds on macOS
arm64. Running the script with each build gave:

- **The same decoded audio.** `-map 0:a -c:a pcm_f32le -f md5 -` gives
  `MD5=1278d03762ada1a1e04f3d278680760c` on both.
- **The same results, apart from `ebur128`.** Every pause, onset, fire count and supplementary
  gap is identical, including the bed's (a PCM WAV, so no lossy decoder is involved), and so is the normalized raw output of `silencedetect`, `silenceremove`,
  `astats` and `aspectralstats`. `ebur128`'s raw metadata differs in 3 of 97 frames, all far
  below the −70 LUFS absolute gate, so it changed no pause.
- **One cosmetic difference.** The log prefix changed from `[Parsed_silencedetect_0 @ …]` on
  7.1 to `[silencedetect @ …]` on 9.0.2, because 9.0 logs with the filter context. A reader that
  parses the log must match only the text after the prefix.

**What was reasoned, not tested:**

- **`silencedetect` is a comparison, not arithmetic.** Given identical decoded samples, its
  output is exact on any version whose source keeps that loop. The 7.1 → 9.0.2 diff shows it
  unchanged.
- **The decoder is the weak link.** The take's audio is AAC, a floating-point decoder with
  platform-specific SIMD. FFmpeg's own AAC conformance tests do not require bit-exact output:
  `tests/fate/aac.mak` at `n7.1` sets `CMP = oneoff` and `FUZZ = 2` for every decode test. On
  another platform or a later version, a sample sitting right at the threshold could change
  side, and a boundary could move by a sample, or a pause close to *d* could appear or vanish.
  A WAV or other PCM voice has no such risk.
- **The other filters depend on the build's floating-point code.** `astats` uses plain C double
  arithmetic, and `aspectralstats` uses `av_tx` FFTs with per-architecture assembly. Both were
  bit-identical across these two versions on arm64. x86 was not tested.
- **`ebur128` is the least stable.** Its arithmetic was restructured between 7.1 and 9.0.2, and
  9.0.2 adds x86 assembly. It is not a version-stable fact.

---

## 7. Reproducing

From the repo root, with any ffmpeg on `PATH` (or `--ffmpeg PATH`); standard-library Python
only. Each run takes about 20 s:

```sh
python3 docs/research/speech-timing-ffmpeg-facts/measure.py --json /tmp/r9.json
python3 docs/research/speech-timing-ffmpeg-facts/measure.py \
    --ffmpeg /opt/homebrew/opt/ffmpeg@7/bin/ffmpeg --json /tmp/r7.json
```

The underlying ffmpeg calls (V = `docs/research/skills-eval/assets/presenter/take-1.mp4`):

```sh
ffmpeg -version
ffprobe -v error -show_entries stream=index,codec_type,codec_name,sample_rate,channels,duration -of compact "$V"
ffmpeg -hide_banner -nostats -i "$V" -map 0:a -c:a pcm_f32le -f md5 -
ffmpeg -hide_banner -nostats -i "$V" -vn -af silencedetect=n=-35dB:d=0.15 -f null -
ffmpeg -hide_banner -nostats -i "$V" -vn -af "asetnsamples=n=16:p=0,silenceremove=stop_periods=-1:stop_threshold=-40dB:stop_duration=0.1:detection=rms:window=0.02:timestamp=copy" -c:a pcm_f32le -f framecrc -
ffmpeg -hide_banner -nostats -i "$V" -vn -af "asetnsamples=n=160:p=0,astats=metadata=1:reset=1,ametadata=mode=print:file=astats.txt" -f null -
ffmpeg -hide_banner -nostats -i "$V" -vn -af "ebur128=metadata=1:framelog=quiet,ametadata=mode=print:file=ebur.txt" -f null -
ffmpeg -hide_banner -nostats -i "$V" -vn -af "asetnsamples=n=256:p=0,aspectralstats=win_size=512:overlap=0.5:measure=flux,ametadata=mode=print:file=spec.txt" -f null -
# the rendered mixes (§4, "Against which audio"):
ffmpeg -hide_banner -nostats -i docs/research/skills-eval/runs/dev/B-talking-head/with-skills-1/render.mp4 -vn -af silencedetect=n=-40dB:d=0.1 -f null -
```

## Sources

- FFmpeg filter documentation: [ffmpeg-filters](https://ffmpeg.org/ffmpeg-filters.html),
  sections `silencedetect`, `silenceremove`, `astats`, `ebur128`, `aspectralstats`,
  `volumedetect`, `asr`, `whisper`. Read from `doc/filters.texi` at tags `n7.1` and `n9.0.2`.
- FFmpeg source at `n7.1` and `n9.0.2`: `libavfilter/af_silencedetect.c`,
  `af_silenceremove.c`, `silenceremove_template.c`, `af_astats.c`, `f_ebur128.c`,
  `af_aspectralstats.c`, `af_volumedetect.c`; `tests/fate/aac.mak` (`n7.1`).
- Repo: `docs/research/skills-eval/pack-src/align_takes.py` (how the words files were made),
  `skills/montagent-footage/scripts/captions.py` (the repair: −35 dB/150 ms, 60 ms rule),
  ADR-0051, ADR-0061, ADR-0115, ADR-0134.
