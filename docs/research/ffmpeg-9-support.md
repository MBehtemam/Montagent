# Research: ffmpeg 9, the MONTAGENT-2 seek clamp, and the ffmpeg range Montagent supports

Research for [#471](https://github.com/MBehtemam/Montagent/issues/471), part of the map [#383](https://github.com/MBehtemam/Montagent/issues/383).

**Date of research:** 2026-09-29. Montagent at `origin/main` `d819e9a2`. ffmpeg binaries measured: **9.0.2** (Homebrew, `/opt/homebrew/bin/ffmpeg`), **8.1** and **7.1.1** (osxexperts.net arm64 static builds, unpacked into a scratch directory and put first on `PATH`, with Homebrew's 9.0.2 `ffprobe` beside them). Homebrew's leftover 8.0.1 cellar would not start (`libvpx.11.dylib` missing).

**Sourcing rule applied:** version facts come from FFmpeg's own tree at release tags (`Changelog`, `fftools/ffmpeg.h`, `fftools/ffmpeg_opt.c`, `fftools/cmdutils.c`, `doc/ffmpeg.texi`, `doc/filters.texi`, fetched from `raw.githubusercontent.com/FFmpeg/FFmpeg/<tag>/…`) and from FFmpeg commits (GitHub mirror). Behaviour claims come from runs on the three binaries. Each finding is tagged **[verified]** (read in a primary source or measured) or **[inference]** (reasoned from verified facts, not observed).

---

## 1. Answer

- **Group 2 (seek_clamp) is an engine regression, not a test-harness bug.** ffmpeg 9 removed `-vsync`. `decode::frame_at` passes `-vsync 0` to its `select` spawn, so that spawn exits 8 with no output. `frame_at` does not check the exit status. It takes the empty output to mean "no frame at or before" and runs its symmetric-clamp fallback, which returns the **earliest frame in the 200 ms window**. That is up to five frames early at 25 fps, with `Ok(..)` and no finding. MONTAGENT-2's silent wrong picture is back, and it is worse than the original one-frame shift. The fixture and the pixel readback behave the same on 9.0.2, 8.1 and 7.1.1. [verified, §2]
- **Group 1 (audio mix) has the same cause class.** `-filter_complex_script` was deprecated in **7.0** in favour of `-/filter_complex <file>`, and removed in **9.0**. [verified, §3]
- **One argument form covers 7.0 through 9.x:** `-fps_mode passthrough` (since 5.1) plus `-/filter_complex <file>` (since 7.0). With those two, every other option and filter Montagent uses works unchanged from 7.0 up. **Recommended supported range: ffmpeg ≥ 7.0**, 9.x included. [verified for 7.1.1 / 8.1 / 9.0.2 by measurement; 7.0 by source, §4]
- **Detect vs support both:** use the single form, not a dual path. A version floor is worth enforcing, but by **probing for capability** and not by parsing the `ffmpeg version` string, which is not reliably parseable (§5). Whatever else is chosen, `frame_at` should **treat a non-zero exit from the `select` spawn as an error** and not as "no frame". That missing check is why a removed CLI option became a silently wrong picture rather than a loud failure. [inference, §5]

---

## 2. Group 2: which frame the engine paints on ffmpeg 9, measured

### 2.1 The failing tests, on a clean tree [verified]

`cargo test -p montagent-core --test seek_clamp` on 9.0.2: 5 failed, 2 passed. The failures are `Ok(77)` for `Ok(81)` (at 3276 ms), `Ok(25)` for `Ok(30)` (30000/1001, at 1001 ms), `Ok(26)` for `Ok(30)` (at 1234 ms), `Ok(30)` for `Ok(35)` (at 1400 ms), and `Ok(0)` for `Ok(1)` (offset source, at 83 ms). The generator self-check (`the_generator_numbers_its_frames…`, which decodes through `frames_from` without `-vsync`) **passes**, so the fixture numbers its frames correctly on 9.

Every wrong index is the first frame at or after `at − 200 ms`. For example, 3276 − 200 = 3076 ms, and the first frame at or after that is 77 (3080 ms).

### 2.2 The `select` spawn, run by hand [verified]

This is the exact argument list from `crates/montagent-render/src/decode.rs:226–237` for `at = 1234 ms`, on the test's own generated fixture (FFV1/gbrp `.mkv`, 25 fps, 82 frames, index × 3 in red):

```
ffmpeg -hide_banner -loglevel error -ss 1.034 -copyts -t 0.201 -i numbered.mkv \
  -vf "select='lte(t\,1.234001)',scale=64:64" -vsync 0 -f rawvideo -pix_fmt rgba -
```

| ffmpeg | `-vsync 0` | same with `-fps_mode passthrough` | fallback (`-frames:v 1`) |
| --- | --- | --- | --- |
| 9.0.2 | **exit 8**, `Unrecognized option 'vsync'.`, 0 frames | exit 0, frames `[26,27,28,29,30]` → last = **30** | `[26]` |
| 8.1 | exit 0, `[26,27,28,29,30]` → **30** | same | `[26]` |
| 7.1.1 | exit 0, `[26,27,28,29,30]` → **30** | same | `[26]` |

`-copyts`, the input seek and `select` return exactly the same frames on all three versions. The only difference is that 9.0.2 refuses to parse the command line at all.

### 2.3 Why a refused command line becomes `Ok(wrong frame)` [verified, `decode.rs:237–254`]

```rust
let at_or_before = over_window(&["-vf", &filter, "-vsync", "0"])?;   // Err only if the spawn fails
let (output, at_or_after) = if at_or_before.stdout.len() >= expected {
    (at_or_before, false)
} else {
    (over_window(&["-frames:v", "1", "-vf", &scale])?, true)          // ← taken on exit 8
};
```

`over_window` returns `Err` only when the process cannot be spawned. The exit status and stderr of the first run are never inspected. So "ffmpeg rejected the arguments" cannot be told apart from "the window holds no frame at or before the instant" (ADR-0096 §5's legitimate case). The fallback spawn has no `-vsync`, so it succeeds and paints the window's earliest frame.

### 2.4 The whole test matrix, emulated at the CLI on 9.0.2 [verified]

This is a Python emulation of `frame_at` (same window, `-t`, threshold and fallback), run over all 23 instants `seek_clamp.rs` asserts:

- with `-vsync 0`: **17 of 23 wrong**, all via the fallback, with the same values the tests report (77, 25, 26, 30, 0 for 1). The 6 that happen to be right are instants where the earliest frame in the window is the correct answer (t = 0, the offset source before frame 1, and the past-the-end error).
- with `-fps_mode passthrough`: **0 of 23 wrong**. The offset source's `at ≤ 42` uses the fallback legitimately (ADR-0096 §5), and everything else goes through `select`.

### 2.5 Other versions run the whole suite green [verified]

`cargo test -p montagent-core --test seek_clamp --test render --test reference_video --test world_effects --no-fail-fast`, with each ffmpeg first on `PATH`:

| ffmpeg | seek_clamp | render | reference_video | world_effects |
| --- | --- | --- | --- | --- |
| 9.0.2 | 2 ok / **5 failed** | 15 ok / **5 failed** | 2 ok / **1 failed** | 3 ok / **1 failed** |
| 8.1 | 7 ok | 20 ok | 3 ok | 4 ok |
| 7.1.1 | 7 ok | 20 ok | 3 ok | 4 ok |

That accounts for exactly the 12 failures in the ticket, and all of them disappear on 8.1 and 7.1.1.

### 2.6 Severity [verified + inference]

- **Engine, not harness.** The tests call `montagent_render::decode::frame_at`, which is the single-frame decode that paints video elements (ADR-0096, "Scope"). The fixture generator and the red-channel readback give the same results on all three versions.
- **Silent.** There is no error, no finding, and the result is `Ok`. On ffmpeg 9, **every video instant is painted from the earliest frame in `[at − 200 ms, at]`**, so 200 ms (5 frames at 25 fps) early once `at ≥ 200 ms`, and frame 0 below that. The only instants that come out right are those inside a source's first frame (and the offset-origin case ADR-0096 §5 already sends to the fallback). This happens whether or not the instant is on the grid. This is broader than MONTAGENT-2, which was one frame early and only off-grid. [inference from §2.4; not measured through a full `render`]
- **ADR-0096 was right that seek_clamp.rs is the only guard.** Without it, nothing in the repository would have caught this.
- I did not observe the render-side symptom end to end, because production code could not be patched in this session. What is measured is the engine function the render path calls.

### 2.7 The ffmpeg 8/9 changelog items the ticket asked about [verified]

The `Changelog` sections for 8.0, 8.1 and 9.0 (at `n9.0.2`) list no change to `select`, `-copyts`, input `-ss`, `-accurate_seek`, timestamp handling or `-fps_mode` defaults. 9.0's user-visible CLI change here is the removal of long-deprecated fftools options, done in commits and not listed in the Changelog (§3). Measured, the frames `select` returns are identical across 7.1.1, 8.1 and 9.0.2 (§2.2), so none of those areas contributes to this failure.

---

## 3. Version facts: `-vsync`, `-fps_mode`, `-filter_complex_script`, `-/filter_complex`

| Option | Added | Deprecated | Removed | Source |
| --- | --- | --- | --- | --- |
| `-fps_mode` | **5.1** (commit authored 2022-06-11) | — | — | [09c53a04](https://github.com/FFmpeg/FFmpeg/commit/09c53a04c5892baee88872fbce3df17a00472faa) "ffmpeg: add option fps_mode". Absent from `ffmpeg_opt.c` at `n5.0` (0 hits), present at `n5.1`. |
| `-vsync` | ancient | **5.1** in docs ("vsync is deprecated and will be removed in the future", `doc/ffmpeg.texi` @ `n5.1`). **7.0** guarded by `FFMPEG_OPT_VSYNC` ([7f982065](https://github.com/FFmpeg/FFmpeg/commit/7f982065a8025a65ef0e3e719b0fb1a59b2a0d77) "mark -vsync for future removal"). The help text in 7.0 reads "deprecated, use -fps_mode". | **9.0** | [927ffd09](https://github.com/FFmpeg/FFmpeg/commit/927ffd0930fa3f7a948d4ed30c67bd518f9549ce) "fftools/ffmpeg: Remove deprecated -vsync option" (2026-06-23). `FFMPEG_OPT_VSYNC 1` is present in `fftools/ffmpeg.h` at `n7.0`, `n8.0` and `n8.1`, and absent at `n9.0`. |
| `-filter_complex_script` | ancient | **7.0**: [c316c4c7](https://github.com/FFmpeg/FFmpeg/commit/c316c4c77b1bac8e9a77000294d3e8cf0ca45dc7) "fftools/ffmpeg: deprecate -filter_complex_script" (2024-01-20). The option table at `n7.0` has "deprecated, use -/filter_complex instead" under `#if FFMPEG_OPT_FILTER_SCRIPT`. | **9.0** | [07407fff](https://github.com/FFmpeg/FFmpeg/commit/07407fff6142f14dcb21b8a06d0d15db0e31135e) "fftools/ffmpeg: Remove deprecated -filter_complex_script option" (2026-06-23). `FFMPEG_OPT_FILTER_SCRIPT 1` is present at `n7.0`, `n8.0` and `n8.1`, and absent at `n9.0`. |
| `-/<opt> <file>` (hence `-/filter_complex`) | **7.0** | — | — | [6d17991b](https://github.com/FFmpeg/FFmpeg/commit/6d17991b7e1bf1a5d104c8a6261709f7e6640d97) "fftools/cmdutils: add option syntax for loading arbitrary arguments from a file" (committed 2024-01-20). `Changelog` @ `n7.0`: *"ffmpeg CLI options may now be used as -/opt <path>, which is equivalent to -opt <contents of file <path>>"*. `cmdutils.c` has `if (*opt == '/')` from `n7.0` on and not at `n6.1`. |

Measured with `-filter_complex_script fc.txt` against `-/filter_complex fc.txt`, where the file holds a newline-terminated graph like Montagent's: both exit 0 on 7.1.1 and 8.1. On 9.0.2 the first gives `Unrecognized option 'filter_complex_script'` (exit 8) and the second exits 0. `ffmpeg -h full` lists `-vsync` and `-filter_complex_script` on 7.1.1 and 8.1, and on 9.0.2 lists neither. It lists `-fps_mode` on all three. [verified]

So the **overlap window for one form is 7.0 through 9.x**. The old form works up to 8.x, and the new form works from 7.0.

---

## 4. Option and filter inventory and supported range

This is every ffmpeg/ffprobe argument in `crates/**/src` (production, not tests).

### 4.1 `ffmpeg`, decode (`crates/montagent-render/src/decode.rs`)

| Argument | Used as | Works |
| --- | --- | --- |
| `-hide_banner`, `-loglevel error` | global | all modern versions |
| `-c:v libvpx-vp9` (before `-i`) | decoder choice for VP9-with-alpha | all modern versions, **but only if built `--enable-libvpx`** (a build flag, not a version) |
| `-ss <s>` before `-i` | input seek | all modern versions |
| `-copyts` | keep source timestamps | all modern versions |
| `-t <s>` before `-i` | bound the read | all modern versions |
| `-vf select='lte(t\,…)',scale=W:H` | frame_at | all modern versions |
| `-vf fps=N,scale=W:H` | frames_from | all modern versions |
| **`-vsync 0`** | frame_at | **≤ 8.x**. Replace with **`-fps_mode passthrough`** (≥ 5.1) |
| `-frames:v 1` | fallback | all modern versions |
| `-f rawvideo -pix_fmt rgba -` | output | all modern versions |

### 4.2 `ffmpeg`, encode (`crates/montagent-render/src/encode.rs`)

| Argument | Works |
| --- | --- |
| `-y`, `-f rawvideo -pix_fmt rgb24 -s WxH -r N -i pipe:0` | all modern versions |
| **`-filter_complex_script <file>`** | **≤ 8.x**. Replace with **`-/filter_complex <file>`** (≥ 7.0) |
| `-map 0:v`, `-map [mix]` | all |
| `-vf pad=W:H:0:0:color=0xRRGGBB` | all |
| `-c:v libx264 -preset medium -crf 20 -pix_fmt yuv420p -r N` | all versions, **but only if built with `--enable-gpl --enable-libx264`** (a build flag) |
| `-c:a aac -b:a 160k` | native AAC encoder, all modern versions |
| `-an`, `-metadata comment=…`, `-movflags +faststart`, `-f mp4` | all |

### 4.3 Audio filtergraph (`crates/montagent-core/src/verbs/render.rs:1215–1475`)

`aformat=sample_rates=…:channel_layouts=stereo`, `atrim=start:end`, `asetpts=PTS-STARTPTS`, `atempo=<f>` (Montagent chains factors, and 7.0 docs give the per-instance range as [0.5, 100.0]), `aloop=loop=-1:size=N`, `asendcmd=c='…'`, `volume@label=volume=V:eval=frame`, `adelay=delays=N:all=1`, `amix=inputs=N:normalize=0` and `apad`. Every one of these filters and options is documented in `doc/filters.texi` at both `n7.0` and `n9.0.2`. `asendcmd` and `asetpts` are documented under the shared `sendcmd, asendcmd` and `setpts, asetpts` sections. [verified by doc presence; the full render suite passes on 7.1.1, 8.1 and on 9.0.2 once the script option is fixed, §2.5 and §3]

### 4.4 `ffprobe` (`crates/montagent-core/src/media/probe.rs`)

`-v error -print_format json -show_format -show_streams -protocol_whitelist <list> [-rw_timeout 15000000] <path|url>`. Nothing here changed in 7 → 9. The ffprobe 9.0.2 used in every run above is Homebrew's.

### 4.5 Recommended range [inference from §3 and §4]

**ffmpeg ≥ 7.0**, 9.x included, with `-fps_mode passthrough` and `-/filter_complex`. The floor is set by `-/filter_complex`. Everything else Montagent uses is older.

- Two build requirements sit apart from the version: libx264 for any render, and libvpx for VP9-with-alpha sources. Both are already failure modes today, and the version floor does not cover them.
- Keeping ffmpeg < 7.0 working would need the dual path this ticket asks about. Current distros ship ≥ 7 for the most part, but Ubuntu 24.04 LTS ships 6.1. [inference, not researched here beyond the 6.1 option table: `n6.1` lacks `-/`]

---

## 5. Detect vs support both: the evidence

### 5.1 How parseable the `ffmpeg version` string is [verified for the three binaries; inference for the rest]

- Release builds print `ffmpeg version 9.0.2 Copyright …` (Homebrew), `ffmpeg version 8.1 …` and `ffmpeg version 7.1.1 …`.
- FFmpeg's own versioning for **git builds** prints `N-<commits>-g<hash>` with no release number. Distros append suffixes (`6.1.1-3ubuntu5`, `4.4.2-0ubuntu0.22.04.1`), and some packagers prefix `n`. [inference: FFmpeg's `ffbuild/version.sh` behaviour, not re-read for this note]
- The lines that are machine-regular even on git builds are the library versions (`libavutil 61. 1.102` on 9.0.2, `60.26.100` on 8.1, `59.39.100` on 7.1.1). But `-vsync` and `-filter_complex_script` are **fftools** options, not library API. Their removal is tied to the major bump only by FFmpeg's practice, not by any rule, so a libavutil major is a proxy and not the fact.

### 5.2 What each option would cost

- **Refuse an out-of-range ffmpeg by version string.** This is brittle on git and distro builds. It still misses build-flag gaps (no libx264, no libvpx). It also repeats ADR-0096's lesson: ask the authority, don't compute the answer from a declared number.
- **Support both argument forms.** This doubles the argument shape for exactly two options, needs a capability decision per run anyway (which form?), and buys ffmpeg 5.1–6.x (for `-/`) at the price of a second code path the test suite only covers on whichever ffmpeg CI happens to have.
- **Single form plus a capability probe.** `-fps_mode` and `-/filter_complex` both exist on 7.0 through 9.x. A cheap probe tests the capability directly and is independent of version strings and build suffixes. One option is a single `ffmpeg -hide_banner -f lavfi -i nullsrc=d=0.04 -/filter_complex <tmpfile> -fps_mode passthrough -f null -` at resolve time. Another is a match on `ffmpeg -hide_banner -h full` for `^-fps_mode` and a check that `-/` parses. The probe would name the missing capability in a finding (ADR-0091's `E-TOOL-*` family is the natural home).

### 5.3 Recommendation [inference]

1. Switch to the single 7.0+ form: `-fps_mode passthrough` in `decode.rs:237` (and in `docs/adr/frame_at_or_before_check.sh:96,174,179`, which also uses `-vsync 0`), and `-/filter_complex` in `encode.rs:219`. Record **ffmpeg ≥ 7.0** in an ADR amending ADR-0009 and ADR-0064.
2. **Independently of the range:** make `frame_at` fail loudly when its `select` spawn exits non-zero. Take the fallback only on a *successful* run that produced no frame. This turns any future CLI break into `E-INTERNAL`/a named finding instead of a wrong picture. It is the defect that made this silent.
3. If a floor is enforced, **probe capability, don't parse the version**, and report a named finding saying which capability is missing. The version string can go into the message as a courtesy.
4. Keep `seek_clamp.rs` as the guard. It caught this exactly as ADR-0096 intended. Running it in CI against a pinned 7.x and the latest ffmpeg would catch the next removal before users do.

---

## 6. Reproduction

The scripts used live only in the session scratchpad and are not committed. The essential commands:

```sh
# fixture, as seek_clamp.rs generates it
ffmpeg -hide_banner -loglevel error -y -f lavfi \
  -i "color=c=black:s=64x64:r=25,format=rgba,geq=r='N*3':g='0':b='0':a='255'" \
  -frames:v 82 -c:v ffv1 -pix_fmt gbrp numbered.mkv
# frame_at's select spawn at 1234 ms; red/3 of each 16384-byte frame is its index
ffmpeg -hide_banner -loglevel error -ss 1.034 -copyts -t 0.201 -i numbered.mkv \
  -vf "select='lte(t\,1.234001)',scale=64:64" -vsync 0 -f rawvideo -pix_fmt rgba - | wc -c
# 9.0.2: 0 bytes, exit 8. 7.1.1 / 8.1: 81920 bytes (5 frames, last = 30)
```
