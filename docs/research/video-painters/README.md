# Video-painter evidence (#933)

Measured on 2026-10-11, Apple M1 Pro, ten cores, ffmpeg 9.0.2.
[environment.json](environment.json) records the platform, exact measured release binary
hash and runtime-source hashes. That supplement was captured after the long comparison;
subsequent gate runs print environment metadata in every measurement row. The owner explicitly
permitted measurements under load instead of waiting for load <1.5. Every render row
records load; these are interleaved observations, not a quiet-machine verdict. Timing
includes validation, any probe, painting, encoding and publication. Generated sources
are outside the repository; scripts and raw measurements are committed here.

`reopen.py` is the #926 spawn/seek/first-frame harness, corrected to fail on an incomplete
read or nonexistent input rather than timing EOF as a frame. The command was:

```sh
python3 docs/research/video-painters/reopen.py /private/tmp/w926/bench/media/lane-a-1080.mp4 576 1024 300 3 6
```

[The result](reopen.txt) covers five GOP phases, 18 observations per phase (three runs,
six concurrent opens). Phase medians are 162, 369, 621, 811 and 1,004 ms; the largest
sample is 1,018 ms. R=1,100 ms rounds above those observed costs. This is a conservative
constant for the measured sources; unusual sources or greater contention can cost more.

`measure.py BINARY DISPOSABLE_KIT BENCHMARK_DIR OUTPUT_JSONL` runs interleaved 240-frame
kit slices and short decode slices; `--cap-only` clones the kit's video to four simultaneous
videos and sweeps K=1/5/6; `--after` compares the actual sequential `one` schedule against
automatic scheduling after implementation. `--decode-only` measures only the short decode
slice. It records all ffmpeg processes and their total RSS at 100 ms intervals (the host
must allow `ps`), verifies the exact frame count, and asserts identical decoded-frame and
MP4 SHA-256 digests across schedules. Output filenames are distinct from old research
outputs to respect the renderer's deliverable attestation.

Inputs:

- The kit used for #932, `/private/tmp/932-measure/kit`, with its existing 14-second
  H.264 stand-in. See [the prefix-cache evidence](../static-prefix-cache/README.md).
  Original assets/project are on `origin/claude/confident-clarke-r5iqm5` under
  `docs/research/long-form-feedback/montagent-repro/`; media is not duplicated here.
- The benchmark built by `python3 fixtures/benchmark/make_benchmark.py DIR --only-1080`.
  This session reused `/private/tmp/w926/bench`, whose `manifest.json` records the pinned
  source settings. The full benchmark has 20 video elements, peak simultaneous presence
  three; the old design's N=20 example counted total elements rather than peak presence.

[before.jsonl](before.jsonl) uses current HEAD before the painter lift. The kit's median
wall seconds are 4.524 (one painter over one paint chunk), 4.874 (K=2,C=15), 3.633
(K=6,C=15) and 8.500 (K=6,C=2). The forced-six C=15 speedup is 1.25×, observed only.
The newer prefix cache has removed much of the original kit's paint cost. Two-frame
paint chunks still lose to larger chunks.

[cap.jsonl](cap.jsonl) compares four videos: medians 5.240, 7.658 and 6.351 seconds for
K=1/5/6 respectively at C=15 for the parallel cases. The process count is 5/21/25
(including the encoder); ffmpeg RSS at K=5 is 1,250–1,329 MiB, at K=6 1,436–1,466 MiB.
This justifies keeping the cap conservative, not claiming its factor is optimal. The
per-painter feed budget and the separate reorder-window check still matter.

The first [uncached diagnostic](uncached.jsonl) used a two-second budget and kept one
painter: 31.65 seconds sequential versus 33.73 seconds automatic. Its 2.08-second added
work demonstrated that the original budget could expire on the very paint class this
change must recognise. The final policy increases that budget to four seconds, retaining
the conservative window/span/speedup exits. [uncached-final.jsonl](uncached-final.jsonl)
checks the real probe on this original paint-heavy case with the prefix cache disabled;
speedup is observed, never gated. Three interleaved pairs gave medians **33.388 seconds
sequential and 14.871 seconds automatic (2.25×)**, selecting K=6 with C=17 or C=18;
all six frame and MP4 digests agree. `--uncached --kit-only --after` reproduces it.
The cached kit's final medians are 8.173 seconds sequential and 7.792 automatic; the
short decode slice is 3.153 versus 3.659 seconds. These short comparisons overlapped
the suite/full benchmark, as their load and global ffmpeg process/RSS samples disclose.
The short decode slice pays fixed probe overhead; the full-span gate is the no-regression
criterion. Earlier cached observations with the two-second pilot are retained in
[after-2s.jsonl](after-2s.jsonl).

The probe policies are 3×4 consecutive frames, a 4-second wall budget, minimum predicted
speedup 1.5 and maximum window floor twice the ordinary byte budget. These bound work and
memory; they are not measured universal thresholds. A decoded-frame identity check guards
all observed render comparisons. The four-second budget can be overrun by an in-flight
frame or feed teardown, as documented in ADR-0196.

The full 10,800-frame, six-minute comparison is the ignored Rust test:

```sh
MONTAGENT_BENCHMARK_DIR=/path/to/benchmark \
  cargo test --release -p montagent --test render_target \
  video_painters_do_not_regress -- --ignored --nocapture
```

It checks a quiet M1 Pro on AC before every run, discards one pair and alternates order
across five timed pairs, using fresh probe sidecars. Frame hashes must agree with the
sequential reference. To reproduce the owner's load exception, additionally set
`MONTAGENT_VIDEO_PAINTERS_ALLOW_LOAD=1`; every JSON row records that exception.
The full comparison passed: sequential median **139.556 seconds**, automatic median
**143.344 seconds**, ratio **1.027143** (about **2.7% overhead**, below 10%). All 12
renders, including the two discarded warm-ups, had identical 10,800-frame hashes.
Automatic selection kept one painter, with peak N=3 and feed cap=20. The first timed
pair overlapped the final suite; later pairs were steadier. These remain observations
under the explicitly authorised load exception.

[decode-gate.jsonl](decode-gate.jsonl) holds those rows; [after.jsonl](after.jsonl) records
the short kit/decode comparisons after implementation. `python3 check.py` re-derives the
recorded scheduling allowance, timing medians, exact digest agreement and the 10% relative
decode comparison, failing if any recorded requirement stops holding. A quiet-machine
comparison remains owed; the relative comparison under load is the accepted exception.
