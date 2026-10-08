# Noise reduction: evidence for ADR-0181 (#853, map #795)

- `check_noise_reduction.py`: the measured check. Stdlib only. Run with `FFMPEG=<ffmpeg 7.1.x> python3 check_noise_reduction.py`.
  It renders seeded synthetic signals and the fixture through the graph the renderer writes, asserts the gated numbers, and
  writes `measurements.json`. It exits non-zero when a gate fails. Unset `FFMPEG`, it uses whatever `ffmpeg` is on the PATH.
- `measurements.json`: the last run, on Homebrew ffmpeg 7.1.5 (one Mac, arm64). Not yet the BtbN floor pin, and not the
  three-leg table (ADR-0181 §7).
- `ab/`: the owner's blind listen. `X.m4a` and `Y.m4a` are the fixture bypassed and with `reduction_db` 20, AAC 160k,
  loudness-matched to −19.2 LUFS. `KEY` names which is which; do not open it before the verdict. No verdict is recorded yet.

Input: `../fixtures/recorded-voice/recorded-voice.flac` (#848, provenance there).
