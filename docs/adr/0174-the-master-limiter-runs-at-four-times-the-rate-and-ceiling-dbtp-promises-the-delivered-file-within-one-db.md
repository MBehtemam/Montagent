---
status: accepted
amends: 0172 (the closing limiter's recipe, and what `ceiling_dbtp` promises), 0173 (§6's `ceiling_dbtp` tolerance becomes +1.0 dB, and §7 is discharged)
---

# The master limiter runs at four times the rate, and `ceiling_dbtp` promises the delivered file within 1 dB

[How does the master stage hold a true-peak ceiling on the AAC delivery, and what is the
allowance?](https://github.com/MBehtemam/Montagent/issues/824) on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)). ADR-0173 §7 required the true-peak
measurement before it could be accepted. [#815](https://github.com/MBehtemam/Montagent/issues/815)
made it, and it contradicted ADR-0172's stage.

## Evidence

[`FINDINGS.md`](../research/audio-effects/true-peak-allowance/FINDINGS.md) holds the tables.
Signals 3, 6 and 10 dB over a −1 dBTP ceiling, through gain, the limiter, the native `aac`
encoder at 160k and a decode, on Linux (floor n7.1.5), macOS (Homebrew) and Windows
(Chocolatey):

- **The stage as ADR-0172 worded it** (`alimiter`, `latency=1`): the decoded AAC lands up to
  **+3.3 to +3.8 dB** over the ceiling. `alimiter` limits sample peaks, so an fs/4 sine at 45°
  carries +3 dB of inter-sample peak before the encoder. Its `level` option also defaults on and
  re-raises the output.
- **The same limiter at 4× the rate with `level=0`:** the limited PCM is within +0.2 dB, with no
  added delay, the same length and identical bytes across runs on one build. After AAC the
  worst case is **+1.8 dB** (clipped-sine bursts, pink noise at +10 dB), identical on all
  three legs. The shared fixture is at most +0.5 dB.
- **Other factors:** 2× is worse (+2.1 dB), 8× no better (+2.0 dB). 4× is the measured choice.
- **Across builds** the limited PCM is *not* byte-identical (the legs' hashes differ), so no
  test compares bytes across builds, as ADR-0173 §1 already said.

## Decisions

### 1. `ceiling_dbtp` promises the delivered file

It means the decoded AAC stays at or under the ceiling plus the allowance below. It does not
mean the limiter's own output. The key's format-page text says so, with the figure. An agent that
sees a `review` finding lowers `ceiling_dbtp` by about the reported overshoot and re-renders.

### 2. The closing limiter runs at 4× the rate

ADR-0172's second pass is `volume=<target − measured>dB`, then, if `ceiling_dbtp` is set:

```
aresample=192000, alimiter=limit=<ceiling as linear>:latency=1:level=0, aresample=48000
```

`level=0` is required under any variant. With only `ceiling_dbtp` set, the stage is the same
chain on the summed mix.

### 3. The allowance is +1.0 dB, and the worst case is written down

`verify` raises a `review` when the decoded AAC's true peak is more than **1.0 dB** above
`ceiling_dbtp` (ADR-0173 §6). The fixture's worst is +0.5 dB; the margin covers encoder drift
across builds. The figure is a `verify` threshold and gates no test.

The adversarial worst case is **+1.8 dB**: clipped-sine bursts and noise-like material driven
far over the ceiling. A project with such content can legitimately trip the finding. The
finding's text says a miss on clipped or noisy material is advisory and names the one edit that
helps. The script asserts the fixture at +1.0 dB and the synthetic signals at +1.9 dB (worst
plus 0.1), on every leg.

## Consequences

- The renderer's master stage implements §2. Its filtergraph string is a committed golden, and
  the byte-identical-within-one-build assertion of ADR-0173 §5 applies to it.
- Latency: the chain adds none (impulse stays at its sample), and a test pins that.
- The resampler's cross-build behaviour is measured only as "not byte-identical", which the
  tolerance convention already absorbs.
- ADR-0173 is accepted. No capability ADR on the audio map waits on it any longer.

## Court

Three jurors (Opus, Sonnet, Fable) answered the three questions as an authoring agent would live
with them. Records are in
[`docs/research/juries/audio-true-peak/`](../research/juries/audio-true-peak/BALLOTS.md).
Q1 (a) and Q2 (ii) were 3/3. Q3 was 2/1 on the number: (gamma) at +0.5 dB twice, and Juror 3's
(delta) at about +1.0 dB, because the fixture itself measured +0.5 dB and +0.5 leaves no margin.
The owner ruled with the Judge's read, which took the shape of (gamma) with Juror 3's figure.
