# Static-prefix cache measurements (#932)

Measured 2026-10-11 on the developer's Apple M1 Pro, release build, ffmpeg/libx264 supplied
by the environment. The disposable kit copy lives at `/private/tmp/932-measure/kit`; it
contains the reporter's project, image/font assets and the same stand-in video used by
the #923 probe. Media is not duplicated in this repository.

Run `python3 measure.py /absolute/path/to/montagent /absolute/path/to/disposable-kit`.
The script creates a baked background and renders `[1500000,1503000)` (90 frames,
1920×1080, 30 fps), interleaving cache off, cache on, and baked three times. It forces one
painter over one 90-frame chunk to isolate prefix reuse from the separate painter-count
work. Profiling is off. Times include process startup, validation and finalization.

| Variant | Median seconds/frame |
| --- | ---: |
| Unchanged project, cache off | 0.135482 |
| Unchanged project, cache on | 0.025700 |
| Two plates baked into one PNG, cache on | 0.023482 |

**Observed speedup: 5.27×. Observed cost above baked: 9.45%.** Neither is a gate.
The first repetition overlapped workspace compilation/testing and was slower; the raw
[results](results.jsonl) retain that sample. Three samples are an observation, not a
stable performance bound. An earlier sweep before shared publication measured 5.10×
and 11.53% above baked, illustrating why the 10% aim is observed rather than enforced.

All six unchanged-project runs (cache on/off) have the same decoded-frame digest:
`5aafebb29a1c626f57e387174459cfc37f64c52da923ad52143c004ea97cfefa`.
The digest is SHA-256 over ffmpeg's non-comment `framemd5` rows. The baked variant's
digest differs, as expected: it changes the author's layers and is only a timing
comparison. Baked identity is not an acceptance criterion.

A separate `[1500000,1501000)` run, forced one painter over 30 frames, compared decoded
frame MD5 values from final cache-on, final cache-off, and the retained original
`/private/tmp/p923_base.mp4` from `probe/923-prefix-identity`: **all 30 equal in all three**.
Unlike comparing the first 30 frames of a 90-frame encode, this holds the encoder's
lookahead range constant. The original probe's n=1–10,12,14,16 and forced three-painter
identity results are recorded in [#923](https://github.com/MBehtemam/Montagent/issues/923#issuecomment-6108717242).

The deterministic integration guards are in `crates/montagent-core/tests/prefix_cache.rs`.
They compare encoder-input hashes and public reports, so encoder lookahead cannot hide
a pixel regression; every successful cache-on comparison checks actual reuse. They also
exercise multiple painters, opacity keyframes, temporal grain, masks/shadows, transitions,
groups/blends, video feeds, changing presence, source/font order and scaled previews.

Run `python3 docs/research/static-prefix-cache/check.py` from the repository to re-derive
the reported medians and ratios from committed samples, and check consistency of the
recorded identity evidence. It exits nonzero on disagreement. This reproduces the
arithmetic without media; rerunning the performance sweep itself needs the reporter kit.
The retained probe frame MD5 values are committed in [frames.framemd5](frames.framemd5)
and the final 30-frame comparison record in [identity.json](identity.json).
