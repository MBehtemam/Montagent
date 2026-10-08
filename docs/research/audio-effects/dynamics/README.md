# Compressor and per-element limiter as audio effects: PROTOTYPE (#818). Throwaway.

`prototype_dynamics.py` builds both members with ffmpeg, runs the number checks and renders two blind
A/B pairs on the shared fixture. Run: `python3 -I prototype_dynamics.py` (ffmpeg only).
Numbers: `measurements.json` (ffmpeg 6.1.1, Linux only; the three-leg run belongs to the repo test).

## Shape under test: two members, neither singular

```json
{ "name": "compressor", "threshold_db": -28, "ratio": 6, "attack_ms": 5, "release_ms": 120, "makeup_db": 8 }
{ "name": "limiter",    "ceiling_db": -12, "release_ms": 50 }
```

- `compressor` -> `acompressor`, hard knee, **`detection=rms`**, dB converted to ffmpeg's linear.
  `threshold_db` is therefore read against the detector's RMS level, not sample peak: with
  `detection=peak` the transfer sat ~3 dB off the formula (ffmpeg's envelope follower reads a sine
  ~3 dB under its peak). Premiere's Compressor and Audition's are RMS-style, recalled.
- `limiter` -> `alimiter level=0 latency=1`, lookahead fixed at 5 ms, `ceiling_db` is **sample peak**
  (the true-peak ceiling is the master stage's, ADR-0172/0174). Premiere's Limiter has threshold and
  release only; here `ceiling_db` + `release_ms`. No auto-level (`level=0`), no ASC.
- Departures from Premiere: no knee or detection controls, no multiband/expander (not first wave).

## Numbers that hold

- Static transfer on a steady 1 kHz sine, hard knee: gain within 0.3 dB of
  `makeup_db - (rms_in - threshold_db) * (1 - 1/ratio)` above threshold, `makeup_db` below; ratio 1
  is identity.
- Limiter: output peak at the ceiling to 0.00001 dB on a +6 dBFS sine (ceilings -3 and -12).
- Neither member moves a 20 ms burst's onset by a single sample (limiter via `latency=1`).
- Members with `enabled: false` are byte-identical to no members.

## The A/B (two pairs, matched to -20 LUFS, AAC 160k)

- `compressor-X/Y`: narration compressed (threshold -28, ratio 6, makeup +8; strong on purpose), bed untouched.
- `limiter-X/Y`: narration at `volume` +6 dB (its mix peaks +3.1 dBFS, over full scale); one pair
  member has `limiter -12 dB`, the other none. Exaggerated: expect it to sound squashed.

Which of X/Y is the processed one is in `ab/KEY`: **do not open until you have judged.**
Listen for: does the compressed narration sit more evenly (quiet words closer to loud ones), and is
anything worse (pumping, breath noise lifted)? Does the limited one sound clean, squashed, or no different?
