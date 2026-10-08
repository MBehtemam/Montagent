# EQ as an audio effect: PROTOTYPE (#817). Throwaway.

`prototype_eq.py` builds the EQ chains with ffmpeg biquads and renders a blind A/B on the shared
fixture. Run: `python3 -I prototype_eq.py` (ffmpeg only). Numbers: `measurements.json`
(ffmpeg 6.1.1, Linux only; the three-leg run belongs to the repo test).

## Shape under test: several small members, stacked (no band list)

```json
{ "name": "highpass", "frequency_hz": 100, "slope_db_per_oct": 24 }
{ "name": "lowpass",  "frequency_hz": 8000, "slope_db_per_oct": 12 }
{ "name": "shelf",    "side": "low" | "high", "frequency_hz": 200, "gain_db": 3 }
{ "name": "bell",     "frequency_hz": 1500, "gain_db": -6, "q": 0.8 }
```

- None singular (ADR-0169: any kind repeats; stacked members are the band list).
- Slopes a closed set 12 / 24 / 48 dB/oct = 1, 2 or 4 cascaded Butterworth biquad sections.
- Shelf is a 2-pole biquad at ffmpeg's slope S=1 (fixed, not a parameter). Bell width is `q`.
- Render: `highpass`/`lowpass`/`bass`/`treble`/`equalizer` only: zero latency, identical on 6.1
  and 7.1. Never `firequalizer`/`superequalizer`/`anequalizer`.
- Premiere's Highpass, Lowpass, Bass, Treble and Parametric are separate effects too; departures:
  Premiere's Parametric holds a band list (here: stack bells), and its Treble/Bass are named by side
  (here: one `shelf` with `side`).

## The A/B

Document: voice `highpass 100 Hz 24 dB/oct`; bed `bell 1500 Hz −6 dB q 0.8` + `shelf high 9 kHz −4 dB`
(carves the bed where the voice lives). Both mixes are matched to −20 LUFS, AAC 160k. Mix loudness
barely moves (−18.0 either way). Which of X/Y is the EQ is in `ab/KEY`: **do not open until you
have judged.**

Listen for: does the voice sit more clearly on top of the bed in one of them? Is the low end of
the narration cleaner? Is anything made worse (thin, harsh)?

## Numbers that hold (tone vs bypass)

Cutoff −3.01 dB at `frequency_hz` for all three slopes; one octave below ≈ −12 / −24 dB (12/24);
passband ≤ 0.3 dB; bell lands on its `gain_db` at centre (−6.00); low shelf +6.12 at 40 Hz;
a member with `enabled: false` is byte-identical to no member. 48 dB/oct is at the 16-bit-float
noise floor past about 36 dB, so its check stops at the first octave.
