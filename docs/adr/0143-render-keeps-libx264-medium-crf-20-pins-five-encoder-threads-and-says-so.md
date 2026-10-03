---
status: accepted
amends: 0077 (reading 7's CRF 20 and preset `medium` are now a measured choice rather than a default; libx264 is given a pinned `-threads 5`, never ffmpeg's automatic count; the `render` answer discloses the encoder, preset, CRF, threads and ffmpeg version)
---

# `render` keeps libx264 `medium` at CRF 20, pins five encoder threads, and says so

[#628](https://github.com/MBehtemam/Montagent/issues/628), from
[#532](https://github.com/MBehtemam/Montagent/issues/532) and
[#625](https://github.com/MBehtemam/Montagent/issues/625). The decision was put to a court of
three independent jurors. All three agreed on everything except threads: two wanted a pin in
production, one wanted auto threads with a pin only in tests. The decisions below are the
Judge's read, which the dev adopted. This ADR adds the measured N and the evidence.

## The gap

ADR-0077 ratified libx264, `yuv420p`, CRF 20, preset `medium` as *"a quality-per-byte
default, not a measured optimum"*, and said a ticket that wanted to move them should measure
first. ADR-0142 then gave `render` a target (the benchmark project in ≤ 3 min) and said MP4
bytes may change only through #628, with quality evidence.

Once decode is streamed (ADR-0141), the question is whether the encoder is now the
bottleneck. It is not. There was also a gap nobody had decided: libx264's thread count. It was
left to `ffmpeg`, whose automatic count is 1.5 × the CPUs (15 on the M1 Pro). x264 is
deterministic for one input, one set of settings and one thread count, but not across thread
counts, so the same project wrote different bytes on machines with different core counts. It
also left the encoder's share of the cores unbounded, which #627 §5 needs to know.

## Decision

### 1. Encoder, preset and CRF: unchanged, now measured

**libx264, CRF 20, preset `medium`**, as ADR-0077 ratified. From *Research: encoder and
decoder throughput for render on Apple silicon*
([#624](https://github.com/MBehtemam/Montagent/issues/624)), each stage of the benchmark
project once decode is streamed:

- **decode:** 3 × 10,800 frames against a ~485 fps aggregate pipe ceiling, so ~67 s;
- **paint and readback:** ~5 ms per frame on one painter, so ~54 s;
- **encode:** libx264 `medium` at 159–289 fps, so 37–68 s.

The stages overlap, so the slowest one sets the wall time, and all are under three minutes.
A faster preset would save time nobody is waiting on, and costs quality: `veryfast` loses
1.1–1.4 VMAF at CRF 20 (#624).

### 2. No VideoToolbox

`h264_videotoolbox` is not a default and not an option:

- it has no CRF;
- it loses 0.7–2.2 VMAF on camera and synthetic content at x264 `medium`'s bitrate, and scores
  83 against 97 on screen content even at twice the bitrate (#624);
- it exists only on macOS;
- its byte determinism is unproven;
- all it saves is CPU, which this render does not need saved.

Any future hardware-encode option needs its own ADR with quality evidence.

### 3. A pinned thread count: `-threads 5`

- `render` and `preview` always pass **`-threads 5`** to libx264, as an output option after
  `-c:v`. They never use ffmpeg's automatic count.
- **N = 5 is the smallest count for which the encode stays off the critical path** of the
  benchmark project on the dev's M1 Pro (see *The measurements*). At 4, the producer waited on
  the encoder 2.8 × as long as at auto and the render took 26 s longer. At 5 and 6 the wait is
  within 2 s of auto's.
- What this buys:
  - the byte-for-byte MP4 test for time chunks (*How does render paint on more than one
    core?*, [#627](https://github.com/MBehtemam/Montagent/issues/627) §8) runs the production
    path, not a special mode;
  - the same input no longer writes different bytes on machines with different core counts
    (it still can across ffmpeg and x264 builds);
  - the encoder's share of the cores is bounded, which #627 §5 needs to work out K.
- In code: `montagent_render::encode::THREADS` (a `NonZeroU32`, so `0`, which is ffmpeg's
  spelling of *"pick for me"*, cannot be written) and `Settings::PRODUCTION`, the only
  settings `render` and `preview` encode with.
- **Changing N needs a superseding ADR.** It changes the bytes of every MP4 Montagent writes.

### 4. Evidence for the one-time byte change

Pinning changed MP4 bytes once. It ships with VMAF and file size compared against auto
threads, and with the benchmark wall time still within ADR-0142's three minutes. Both are
below.

### 5. Nothing is exposed

No project field, and no CLI or MCP option, for the encoder, the preset, the CRF or the thread
count. An encoder setting is not a property of the composition, and a public knob would have
to be supported forever with no measured need behind it. Tests that need another thread count
build their own `encode::Settings` through the internal encoder settings; nothing outside the
crate's callers can reach them.

### 6. Disclosure

The `render` answer carries, beside `bytes`:

- `encoder`: `"libx264"`;
- `preset`: `"medium"`;
- `crf`: `20`;
- `threads`: `5`;
- `ffmpeg_version`: the word the encoding `ffmpeg` prints after `ffmpeg version` (`"9.0.2"`,
  or a git build's `N-…-g<hash>`), or `null` where it prints something else. It is a label
  for the answer, read from the `ffmpeg` that encoded the file, and never a capability test:
  ADR-0115's qualification is that.

`preview` answers through the same block, flattened under its tier, so it discloses the same
five fields. The text form prints one line:
`encoder     libx264, preset medium, CRF 20, 5 threads, ffmpeg 9.0.2`.

A test reads the file's own libx264 options SEI (`threads=5`, `crf=20.0`, `medium`'s
`bframes=3 subme=7 rc_lookahead=40`) and checks it agrees with the answer. That also closes
ADR-0077's stated gap that *"no test names `libx264`, the CRF or the bitrate"*, for the first
two.

### 7. If the benchmark misses and encode is the cause

Try these in order, each compared **at matched quality, never at matched CRF** (#624):

1. retune N (a superseding ADR);
2. switch to preset `fast`;
3. switch to preset `veryfast`.

## The measurements

**Observed, not the ADR-0142 protocol.** By the dev's standing preference, these are short
runs at whatever load the machine had, one run per setting, no warm-up, and no wait for load
< 1.5. The load average was 8.5–14 throughout, with other sessions running. An observed
number under load is an upper bound on a quiet one; it can confirm a target it is inside of.

Conditions: ffmpeg 9.0.2 (x264 core 165), macOS 27.0, the dev's M1 Pro (8P+2E, 16 GB),
`available_parallelism` 10, release build from a private `CARGO_TARGET_DIR`, nothing compiled
during the runs. Each setting was run from **the same shipped binary** through an `ffmpeg`
wrapper on `PATH` that rewrote only the encode's `-threads` argument (removing it for auto),
so no code differs between the rows. Each output's SEI was read back and carries the thread
count asked for (`threads=4/5/6`, and `threads=15` for auto).

### The whole benchmark project, once per setting

6 min of 1080p30, 10,800 frames, `MONTAGENT_STAGES=1`, `/usr/bin/time -l`. Encode wait is
the time the paint loop spent blocked handing frames to the encoder: when it grows, the
encoder is on the critical path.

| threads | wall | encode wait | decode | paint | user + sys | size | peak RSS | load before → after |
|---|---|---|---|---|---|---|---|---|
| 4 | 155.1 s | 50.1 s | 38.0 s | 52.8 s | 730.9 + 48.4 s | 87,164,955 B | 579 MiB | 14.0 → 10.4 |
| **5** | **130.1 s** | **19.8 s** | 41.9 s | 54.1 s | 746.5 + 50.5 s | 87,154,159 B | 560 MiB | 10.4 → 10.0 |
| 6 | 133.5 s | 19.1 s | 44.4 s | 55.7 s | 742.9 + 53.0 s | 87,165,612 B | 575 MiB | 10.0 → 10.3 |
| auto (15) | 129.4 s | 18.1 s | 41.9 s | 55.3 s | 745.8 + 53.3 s | 87,160,708 B | 740 MiB | 10.3 → 8.5 |

- At 4 the encode is on the critical path: encode wait is 2.8 × auto's and the wall 25.7 s
  longer. At 5 the wait is within 1.7 s of auto's and the wall within 0.7 s, inside the noise
  of one run at this load. So N = 5.
- All four are inside ADR-0142's three minutes; N = 5 at **130.1 s**, against ADR-0142's
  observed 135.4 s.
- File sizes differ by under 0.02 % across all four.
- Readback was 12.9–13.1 s and the seal 0.15–0.34 s in every row.

### Quality: VMAF on a 60 s span

`render --from 60000 --to 120000` (1,800 frames, three videos visible) was encoded at each
setting and once **losslessly** (the same wrapper replaced `-crf 20` with `-qp 0`, so the
reference is the same painted frames, through the same `yuv420p` conversion, 270,711,210 B).
Each was scored against the reference with ffmpeg's `libvmaf` filter (default model).

| threads | VMAF mean | VMAF min | harmonic mean | size |
|---|---|---|---|---|
| 4 | 96.683 | 95.800 | 96.683 | 15,473,780 B |
| **5** | **96.684** | **95.800** | **96.684** | **15,471,158 B** |
| 6 | 96.683 | 95.800 | 96.682 | 15,467,882 B |
| auto (15) | 96.684 | 95.778 | 96.683 | 15,472,103 B |

The pin costs nothing measurable: VMAF is equal to three decimals and the size 0.006 % smaller
than auto's.

**Verdict:** `-threads 5`. The benchmark project renders in 130.1 s observed, within
`RENDER_TARGET`, at the same quality and size as auto threads. `BENCHMARK_REFERENCES` records
the reading, labelled observed.

## Consequences

- Every MP4 `render` writes changed bytes once, with this ADR. A `render_target.rs` frame-hash
  reference taken before it needs re-taking, as that test's header already says.
- The same project, settings and ffmpeg build now give the same bytes on any machine, which
  #627 §8's byte test relies on.
- The encoder uses a bounded share of the cores: at most about 5 of 10, not 15 threads
  competing with the painter.
- Peak memory fell from 740 to 560 MiB in these runs, because 15 encoder threads keep more
  frames in flight than 5.
- An agent can read from any answer which encoder settings and ffmpeg wrote the file, without
  probing it.
- Nothing new is exposed, so nothing new has to be supported.
