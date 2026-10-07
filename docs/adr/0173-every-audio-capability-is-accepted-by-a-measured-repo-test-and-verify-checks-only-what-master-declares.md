---
status: proposed (becomes accepted once §7's true-peak measurement is committed and runs clean; until then the 0.5 dB allowance is provisional)
amends: 0172 (sets `verify`'s tolerances for misses against `target_lufs` and `ceiling_dbtp`, which ADR-0172 left to this ticket), 0117 (`verify` gains no per-capability measurement: it checks only what `master` declares)
---

# Every audio capability is accepted by a measured repo test, and `verify` checks only what `master` declares

[What measured check must every audio capability's ADR name, and what runs it?](https://github.com/MBehtemam/Montagent/issues/802)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)). The map's standing
rule says every capability ADR names a measured acceptance check next to the owner's ear. This
ADR is that rule's contract. A capability ADR cites it in one line ("conforms to ADR-0173") and
fills in §8's checklist.

## Decisions

### 1. Where the check runs

- **Every capability ADR names a repo integration test.** It renders a fixed fixture document
  through the real mix graph and asserts the number on every CI leg: Linux at the floor, macOS on
  Homebrew, and Windows on Chocolatey.
- **The test is a port of the prototype's evidence script**, with the same metric and the same
  fixture. The ADR's number and the build's assertion therefore cannot drift apart. A capability
  may add cases, but the ported case stays the anchor.
- **Assertions use a stated tolerance (§4), never a byte hash across builds.** Several filters
  change bytes across builds and CPU SIMD paths
  ([`FFMPEG-FILTERS.md`](../research/audio-effects/FFMPEG-FILTERS.md) §3.2). This is the same
  stance as the picture's golden frames, which use SSIM at a threshold.
- **`verify` checks only what `master` declares** (§6). It measures one mixed track, so it cannot
  attribute a number to one element's effect, as ADR-0117's own module docs say. A per-capability
  check there would be confident nonsense.

### 2. What measures

- **ffmpeg's own meters:**
  - `ebur128=peak=true` for LUFS and dBTP;
  - `astats` for peak, RMS, NaN/Inf and DC;
  - a filter plus `astats`, or `aspectralstats`, for band energy.
- They are the same instrument `verify` already uses, so a number means the same thing in the
  ADR, in CI and in `verify`.
- The ADR names the meter. The test pins how it parses the meter's output and fails loudly if the
  parse breaks.
- **One carve-out:** a Rust FFT inside the test is allowed for band energy only, and only if an
  ffmpeg-filtered measurement proves noisy across legs.

### 3. Fixtures, and which side of the encoder

**Numbers come from synthetic signals.** lavfi sines, sweeps and impulses, plus `anoisesrc` with a
pinned `seed`, are built in the test from literal parameters. The generator command is the
committed input. No binary is involved, and the expected value follows from the signal's
definition.

**The ear gets one shared committed fixture:** about 20 s of TTS narration over a music bed (the
lead workflow), from a licence-clean TTS voice and a CC0 bed.
- It sits in `docs/research/audio-effects/fixtures/narration-over-bed/` with a `PROVENANCE.md`.
  That file records the source, the TTS engine and voice, the licence with any vendor terms
  quoted, the command that produced it, and its hash.
- A content-dependent check (a gate on speech pauses, compressor pumping) may measure this fixture
  against bypass, but only where a synthetic burst cannot model the behaviour.
- Restoration's recorded-voice fixture is a separate fixture, added later under the same rules.

**Which side of the encoder depends on the claim:**
- **Capability tests** measure the PCM of the same graph with the encoder swapped out. AAC's
  inter-sample peaks and its high-band trim would otherwise mix into every tolerance.
- **Delivery claims** measure the decoded AAC: `verify`, and §7's allowance.
- **A/B clips** go through AAC 160k, because that is what ships. An ADR may also commit a PCM pair
  where AAC could mask the effect, but the verdict it quotes is the AAC listen.

### 4. The tolerance convention

Every check states:
- its metric, meter and pinned parse;
- the side of the encoder;
- its fixture;
- its **expected value and where that comes from**: the filter's definition where one exists
  (gain at a band centre, a pitch ratio, a ceiling, an echo delay), otherwise a delta against
  bypass computed in the same run. Never a stored previous render, which is a byte hash under
  another name;
- whether it is **one- or two-sided** (a ceiling is ≤, a reduction is ≥);
- its **tolerance and where that comes from**: max(2 × the spread measured across the three legs,
  the meter's resolution: 0.1 LU for `ebur128`, the print precision for `astats`). The per-leg
  table is committed with the evidence.

**A borrowed external figure never sets a test's gate.** ADR-0061 makes a borrowed threshold a
`review`, and a test has no `review` outcome. Borrowed figures belong to `verify` and `review`
findings.

When a future build drifts outside a tolerance, that is the signal to re-measure. The tolerance is
never loosened silently.

### 5. Bypass identity, and the smoke check

**Every capability also carries two exact assertions.** Neither compares across builds:
- Within one run, a document with the member set to `"enabled": false` produces PCM byte-identical
  to the same document without the member. Every filter is byte-identical within one build
  (`FFMPEG-FILTERS.md` §3.1). A tolerance here would let a leaked resampler or a one-sample delay
  pass.
- A document that uses no new feature produces the committed expected filtergraph string. This
  extends ADR-0172's promise that an existing document renders byte-identically. The string is the
  engine's own deterministic output, so a golden file is legitimate here. Updating it is a
  deliberate act the capability's ADR states.

**A smoke check** replaces the number where no honest number exists: reverb and voice changers.
It asserts that:
- the render succeeds;
- the length is exact, or within a bound the ADR states;
- the output is 48 kHz stereo;
- the output is not silent, meaning above the −70 LUFS gate;
- there is no NaN/Inf and no clipping;
- the output is not a no-op: its difference from bypass is above a stated floor;
- integrated loudness stays within a generous stated window of bypass.

Echo and reverb also assert their tail. **Pitch shift is not smoke:** it is measured as the
fundamental's shift on a sine, within a tolerance.

### 6. `verify`'s tolerances (amends ADR-0172)

| `master` key | `review` when | source |
| --- | --- | --- |
| `target_lufs` | \|integrated − target\| > 1.0 LU, either direction | EBU R 128 (2020) short-form allowance, ATSC A/85 |
| `ceiling_dbtp` | true peak > ceiling + 0.5 dB, on the decoded AAC | codec inter-sample allowance, **provisional until §7** |

- The below-target miss is **not suppressed** when ADR-0172's headroom review already fired.
  Checking that would mean reading the document's other findings, and `verify` stays independent
  of them (ADR-0117). Instead, its message names heavy limiting and the headroom review as the
  likely cause.
- A finding prints the measured value, the declared value and the tolerance.
- These are `review`, never `error`, as ADR-0172 already ruled.

### 7. The true-peak allowance is measured before this ADR is accepted

A committed script under `docs/research/audio-effects/true-peak-allowance/` does the following:
1. It drives each of these signals several dB over the ceiling:
   - a sine near fs/4 with a 45° phase offset;
   - clipped-sine bursts;
   - seeded pink noise;
   - the shared fixture.
2. It runs them through gain, `alimiter` at −1 dBTP, the native `aac` encoder (pinned by name) at
   160k, and decode.
3. It measures the overshoot with `ebur128=peak=true` and asserts it.

The script runs on all three legs, because the deliverable is encoded by the user's build. The
allowance in §6 becomes the worst case across legs plus a margin, rounded up to 0.1 dB, if that
differs from 0.5 dB.

### 8. The A/B record, the folder layout, and the checklist

A capability's prototype A/B works as follows:
- **What is compared:** the processed document against the same document with the member set to
  `enabled: false`. Both are rendered through the real pipeline and brought to the same
  integrated loudness by one gain.
- **How it is heard:** blind, as X and Y.
- **What is committed:** the clips the owner judged, as a record of what was heard, not a
  reference to compare against.

```
docs/research/audio-effects/
  fixtures/narration-over-bed/    the shared fixture + PROVENANCE.md
  <capability>/
    check_<capability>.py         the evidence script (exits non-zero when a number stops holding)
    measurements.json             its numbers, with the per-leg table
    ab/X.m4a, ab/Y.m4a, ab/KEY    the judged clips and which is which
    VERDICT.md                    the owner's verdict verbatim, with date, build and the KEY's hash
```

The capability ADR quotes `VERDICT.md` and fills in this checklist:

- [ ] Conforms to ADR-0173.
- [ ] Each check: metric · meter and pinned parse · encoder side · fixture · expected value and
  where it comes from · one- or two-sided · tolerance and where it comes from (per-leg table
  committed).
- [ ] Bypass identity: `enabled: false` PCM equals member-absent PCM. The no-new-feature graph
  string is unchanged, or its change is stated.
- [ ] Smoke check instead of a number, if and only if no honest number exists, and the ADR says
  why.
- [ ] A/B: clips, `KEY` and `VERDICT.md` committed. The ADR quotes the verdict.

## Consequences

- No capability ADR on the audio map is accepted before this one.
- The test harness needs a PCM-output path through the same mix graph.
- `verify` gains the two tolerances in §6 and nothing per-capability.
- `docs/agents/domain.md` points audio ADRs here.

## Evidence

Two three-juror courts (Opus, Sonnet, Fable), run blind with no recommendation in the packet.
The owner ruled with the Judge on both. Packets, ballots and rulings are in
[`docs/research/juries/audio-measured-check/`](../research/juries/audio-measured-check/BALLOTS.md).

- Round 1: (c) for where the check runs, ffmpeg's meters, ±1 LU in both directions, and a smoke
  check for reverb and voice changers: all 3/3. The +0.5 dB true-peak allowance: 2/3 (Juror 2
  said +0.3). Committing the judged clips: 2/3 (Juror 3 against). A blind listen: 2/3 (Juror 2
  against).
- Round 2: the fixture split, the per-claim encoder side, bypass identity by bytes, measuring
  the allowance in this effort, and one ADR: all 3/3. That a borrowed figure never gates a test
  is Juror 3's, taken in the Judge's read. Measuring on all legs: 2/3 (Juror 2 said floor only).

The determinism facts are in [`FFMPEG-FILTERS.md`](../research/audio-effects/FFMPEG-FILTERS.md)
§3, backed by its re-runnable check. §7's script is this ADR's own outstanding evidence.
