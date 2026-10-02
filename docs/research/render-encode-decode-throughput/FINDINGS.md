# Encoder and decoder throughput for `render` on Apple silicon

Research for [#624](https://github.com/MBehtemam/Montagent/issues/624), part of the map [#622](https://github.com/MBehtemam/Montagent/issues/622) (render a 6-minute video in minutes, not an hour). Measured 2026-10-02. `bench.sh` beside this file re-runs every measurement (`bench.sh encode`, `bench.sh decode`, `bench.sh exact`; `FFMPEG_DIR=` picks another build).

Every number is labelled:
- **documented**: stated by the source cited next to it.
- **measured**: on this machine, an Apple M1 Pro (10 CPU cores: 8 performance, 2 efficiency; one video encode engine), 16 GB, macOS 27.0, with Homebrew's ffmpeg **9.0.2**, unless it says **7.1.5** (Homebrew's `ffmpeg@7`, the ADR-0115 floor).

The machine was not idle: other sessions were running and memory was under pressure. Single throughput numbers carry roughly ±15% run-to-run noise. Encode speeds are the best of three runs. Read them as orders of magnitude and ratios, not as a specification.

## How the measurements mirror `render`

- **Encode** reproduces `crates/montagent-render/src/encode.rs`:
  - raw `rgb24` 1920x1080 frames on `pipe:0`, `-r 30`, `-c:v libx264 -preset P -crf 20 -pix_fmt yuv420p`, mp4 output;
  - the frames come from a second ffmpeg decoding the clip to `rgb24` on a pipe (feeding a raw file of several GB made the disk the bottleneck under memory pressure);
  - each clip is 150 frames (5 s at 30 fps);
  - the "feed only" row is that same pipe converted to `yuv420p` and thrown away, which is the ceiling the input side sets.
- **Clips:**
  - `cam`: presenter take, `docs/research/skills-eval/assets/presenter/take-1.mp4`;
  - `screen`: screen recording, `.../screen/session.mp4`, mostly static UI;
  - `testsrc2`: ffmpeg's synthetic pattern, high-entropy, a worst case for size.
- **Quality** is VMAF (libvmaf, default model) of each encode against the same `rgb24` frames converted to `yuv420p`.
- **Decode** reproduces `decode::frames_from` (`crates/montagent-render/src/decode.rs`):
  - `-ss <window> -copyts -i src`, the `settb,setpts,fps=30:round=up,scale` chain, `-fps_mode passthrough`, then `-f rawvideo -pix_fmt rgba -` read through a pipe;
  - sources are 20 s at 1080p30 (600 frames):
    - `cam-crf20`: H.264, 0.84 Mbit/s;
    - `cam-20M`: H.264, 18.4 Mbit/s;
    - `testsrc2`: H.264, 8.5 Mbit/s;
    - `cam-hevc`: HEVC, 0.44 Mbit/s.

## 1. libx264 at 1080p30, CRF 20, by preset

**Measured (9.0.2).** Each cell is fps / kbit/s / VMAF.

| clip | feed only | medium | fast | veryfast | ultrafast |
|---|---|---|---|---|---|
| cam | 363 | **204** / 805 / 95.4 | 224 / 789 / 95.2 | 145\* / 594 / 94.1 | 381 / 5949 / 96.8 |
| screen | 436 | **289** / 93 / 97.2 | 283 / 86 / 97.2 | 348 / 75 / 96.0 | 389 / 182 / 97.3 |
| testsrc2 | 452 | **159** / 8157 / 97.6 | 175 / 8087 / 97.2 | 281 / 7203 / 96.2 | 360 / 17327 / 99.2 |

\* Outlier under background load. Two other `cam` runs (240 frames) gave veryfast 290 and 308 fps, medium 197 and 217, fast 225, and ultrafast 369.

- **`fast`** buys almost nothing over `medium`: 0 to +10% speed, about the same size and VMAF.
- **`veryfast`** is the real speed lever:
  - 1.2–1.8× `medium`'s speed;
  - at the same CRF, a smaller file (74–88% of the size);
  - 1.1–1.4 VMAF lower.
- **`ultrafast`**:
  - up to 2.3× `medium`'s speed;
  - 2–7× the size at CRF 20;
  - it turns off CABAC, B-frames, AQ, mb-tree and deblocking (documented: x264 `common/base.c`, `param_apply_preset`).
- **A CRF value is not comparable across presets.** Documented: "for constant quality encoding, you will simply save bitrate by choosing a slower preset" (FFmpeg wiki, Encode/H.264). Judge a preset change at matched quality, not at matched CRF.

**Encode cost of a 6-minute render** (10,800 frames at the measured rates): medium 37–68 s, veryfast 31–39 s, ultrafast 28–30 s.

**`medium`'s real cost is cores** (measured on `cam`, 240 frames):

| encoder | fps | cores busy |
|---|---|---|
| medium, threads auto | 197 | 6.0 |
| veryfast | 308 | 4.8 |
| medium, -threads 4 | 119 | 3.2 |
| medium, -threads 2 | 89 | 2.3 |
| h264_videotoolbox | 174 | 0.9 |
| hevc_videotoolbox | 157 | 0.9 |

- Documented: libx264's automatic thread count is 1.5× the CPU count (`x264_cpu_num_processors() * 3/2`, x264 `encoder/encoder.c`).
- Measured: feeding `yuv420p` instead of `rgb24` takes medium from 217 to 264 fps. The in-ffmpeg conversion is about a fifth of medium's wall time.

## 2. h264_videotoolbox and hevc_videotoolbox

**Throughput (measured, 9.0.2):**
- H.264 runs at 160–175 fps and HEVC at 147–157 fps.
- That rate is the same for all three clips, and unchanged by `-prio_speed 1`, by `nv12` output, or by feeding `yuv420p`. The hardware engine is the limit.
- On this M1 Pro that is no faster than libx264 medium, at 0.9 core instead of 6.
- Documented: the M1 Pro has one "Video encode engine" (Apple, MacBook Pro 14-inch 2021 tech specs, support.apple.com/en-us/111902). Apple says "M1 Max ... delivering up to 2x faster video encoding than M1 Pro" (Apple Newsroom, 2021-10-18). Other chips were not measured.

**Rate control (documented, ffmpeg `libavcodec/videotoolboxenc.c`, identical in `release/7.1` and master):**
- **`-b:v`** maps to `kVTCompressionPropertyKey_AverageBitRate`, which Apple describes as "the long-term desired average bit rate ... not a hard limit".
- **`-constant_bit_rate 1`** asks for CBR (macOS 13+).
- **No CRF.** The nearest control is `-q:v N`:
  - it sets `kVTCompressionPropertyKey_Quality` = N/100;
  - ffmpeg accepts it only on macOS on Apple silicon (`vtenc_qscale_enabled`: `!TARGET_OS_IPHONE && TARGET_CPU_ARM64`) and errors elsewhere;
  - Apple documents the property only as "0.0 to 1.0, where low = 0.25, normal = 0.50, high = 0.75", so it is opaque.
- **Default:** with no rate set, `vt_defaults` sets `b` = 0, and a bit rate of zero means "the video encoder should determine the size" (Apple). Measured, that gave 4.1 Mbit/s on cam (5× x264), 650 kbit/s on screen, and 10.6 Mbit/s on testsrc2.

**Quality at matched bitrate (measured).** VideoToolbox's `-b:v` is set to the rate x264 medium/CRF 20 chose. Each cell is kbit/s, VMAF.

| clip | x264 medium | h264_vt | hevc_vt | h264_vt -q:v 65 |
|---|---|---|---|---|
| cam | 805, 95.4 | 867, 93.2 | 857, 94.0 | 714, 94.0 |
| screen | 93, 97.2 | 206, 83.1 | 296, 92.3 | 396, 95.8 |
| testsrc2 | 8157, 97.6 | 8172, 96.9 | 8288, 97.2 | 8157, 97.6 |

- H.264 trails x264 by 0.7–2.2 VMAF on camera and synthetic content.
- On screen content it can't reach x264's rate: it overshoots 2–3× and still scores far lower.
- HEVC narrows the gap but doesn't close it.

**Availability:**
- Documented: ffmpeg `configure` (`release/7.1`) marks `--disable-videotoolbox` as `[autodetect]`, so a default macOS build includes it.
- Documented: Homebrew's formula passes `--enable-videotoolbox --enable-audiotoolbox` when `OS.mac?`.
- Measured: Homebrew 9.0.2 and ffmpeg@7 7.1.5 both list `h264_videotoolbox`, `hevc_videotoolbox` and the `videotoolbox` hwaccel.
- macOS ships no ffmpeg. Linux and Windows builds, and builds made with `--disable-videotoolbox`, lack it. It can only ever be an optional macOS path.

## 3. Software decode, one streaming process

**Measured (9.0.2).** fps through `frames_from`'s exact command, rgba on a pipe, by input `-threads`. Brackets show the ffmpeg process's busy cores.

| source | threads 1 | 2 | 4 | auto (default) | -hwaccel videotoolbox |
|---|---|---|---|---|---|
| cam-crf20 (H.264, 0.84 Mbit/s) | 499 [1.7] | 434 | 447 | 432 [1.6] | 253 [0.4] |
| cam-20M (H.264, 18.4 Mbit/s) | 151 [1.2] | 246 | 345 | 361 [3.1] | 203 [0.4] |
| testsrc2 (H.264, 8.5 Mbit/s) | 148 [0.9] | 204 | 350 | 419 [2.5] | 231 [0.4] |
| cam-hevc (HEVC, 0.44 Mbit/s) | 389 [1.5] | 495 | 463 | 478 [2.0] | 261 [0.5] |

Repeat runs of cam-20M with threads auto gave 285–361 fps on 9.0.2 and 240 fps on 7.1.5.

**Where the ceiling is (cam-crf20):**

| stage | fps |
|---|---|
| decode only, -f null | 3,279–3,409 |
| decode + rgba conversion, no pipe | 2,690 |
| yuv420p to a pipe | 1,081 |
| **rgba to a pipe** | **477** |
| `frames_from`'s full chain | 432–499 |

cam-20M decodes alone at 979 fps.

**The rgba pipe is the ceiling, not the decoder.**
- 8.3 MB per frame at ~480 fps is ~4 GB/s.
- Documented: xnu's pipe buffer is 16 KB, growing to 64 KB (`PIPE_SIZE` and `BIG_PIPE_SIZE` in `bsd/sys/pipe.h`).

**What `-threads` does:**
- Documented: `threads` sets the codec's thread count; `auto`/0 is the default (`doc/codecs.texi`). For the H.264 decoder that means frame threading, which "will increase decoding delay by one frame per thread" — harmless to a streaming reader.
- Measured: on heavy sources, auto is 2.4–2.8× faster than threads 1. On light sources it changes nothing or costs ~10%, because the pipe binds. Leave it at the default.

**Many streams at once (measured):** four concurrent decoders of cam-20M gave 485 fps in aggregate (~121 each), against 285–361 for one alone. They share the ceiling rather than multiply it.

## 4. -hwaccel videotoolbox, rgba to a pipe

**What it returns:**
- Measured (verbose log): ffmpeg keeps the software h264/hevc decoder with the hwaccel attached ("Selecting decoder 'h264' because of requested hwaccel method videotoolbox").
- Measured: frames are downloaded as `nv12` for 8-bit 4:2:0 (`p210le` for a 10-bit 4:2:2 H.264 source, which VideoToolbox did decode on this machine), and ffmpeg auto-inserts an `nv12 -> rgba` scale.
- Documented: the default output is NV12 (`videotoolbox.c`: "return AV_PIX_FMT_NV12; // same as av_videotoolbox_alloc_context()").
- Documented: `-hwaccel` "has no effect if the selected hwaccel is not available or not supported", and "most acceleration methods ... will not be faster than software decoding on modern CPUs ... ffmpeg will usually need to copy the decoded frames from the GPU memory into the system memory" (`doc/ffmpeg.texi`).

**Speed (measured):** 200–260 fps, slower than software every time, but at 0.4 core instead of 1.6–3.1.

**Timestamps (ADR-0096), measured on 9.0.2 and 7.1.5:**
- 90 frames from each of 2 starts (0 s, and an off-keyframe 7.3 s seek) × 3 sources (H.264, HEVC, testsrc2), compared with framemd5.
- **0 of 540** frames differed in pts or duration.

**Pixels (measured):**

| variant vs software frames_from | frames whose rgba bytes differ |
|---|---|
| hwaccel, chain unchanged | 540/540 |
| hwaccel, chain prefixed with `format=yuv420p` | **0/540** |
| software decode, `format=nv12` prefix (control) | 540/540 |

- The decoded planes are identical: nv12 and yuv420p framemd5s match 30/30 for H.264 and HEVC.
- The difference is swscale's `nv12->rgba` path rounding differently from its `yuv420p->rgba` path; the software control reproduces it without VideoToolbox.
- On sampled frames, ~1% of bytes differed, some by up to 163 levels, clustered in a few regions.
- Prefixing the chain with the source's software pix_fmt (`format=yuv420p` for 8-bit 4:2:0) restores byte-identity. 10-bit sources arrive as p010/p210, so use the pix_fmt ffprobe reports.

## Conclusions for the map

1. **The encoder doesn't stand between render and "minutes".** medium/CRF 20 (ADR-0077) does 160–290 fps, 37–68 s for 6 min. Keep it. If the encode stage ever binds, use veryfast, judged at matched quality. fast buys nothing; ultrafast multiplies the file size 2–7×.
2. **medium costs ~6 of 10 cores.** Parallel paint workers must budget for it, or cap it (`-threads 4` → 119 fps, still 4× real-time).
3. **VideoToolbox encode isn't a win on the M1 Pro.** Same speed, lower VMAF at matched rate (much lower on screen content), no CRF, macOS only. At most an opt-in for chips with more encode engines.
4. **One streaming software decoder suffices.** 150–500 fps of rgba, capped by the pipe (~480 fps). Default threads. Concurrent decoders share the ceiling.
5. **-hwaccel videotoolbox keeps timing exact but breaks frame/render pixel parity** unless prefixed with the source's software pix_fmt. It is slower and only worth it to free CPU.

## Sources
- FFmpeg `libavcodec/videotoolboxenc.c` (master and release/7.1): `vtenc_qscale_enabled`, the `-q:v` → Quality mapping, `vt_defaults`.
- FFmpeg `libavcodec/videotoolbox.c`; `configure` (release/7.1); `doc/ffmpeg.texi` (`-hwaccel`); `doc/codecs.texi` (`threads`, `thread_type`).
- FFmpeg wiki, Encode/H.264.
- x264 `common/base.c` (`param_apply_preset`) and `encoder/encoder.c` (github.com/mirror/x264).
- Apple VideoToolbox docs: `kVTCompressionPropertyKey_Quality`, `AverageBitRate`, `PrioritizeEncodingSpeedOverQuality`.
- Apple MacBook Pro 14-inch 2021 tech specs (support.apple.com/en-us/111902); Apple Newsroom, 2021-10-18.
- Homebrew `Formula/f/ffmpeg.rb`.
- xnu `bsd/sys/pipe.h`.
- Montagent `encode.rs`, `decode.rs` (`frames_from`), `floor.rs`.
