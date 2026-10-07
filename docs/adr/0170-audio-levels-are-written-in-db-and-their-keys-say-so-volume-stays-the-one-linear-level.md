---
status: accepted
amends: 0005 (scopes its integer rule to timeline instants; effect-internal durations are fractional ms), 0055 (scopes its rejection of dB to `volume`; every other level is dB)
---

# Audio levels are written in dB and their keys say so; `volume` stays the one linear level

[Are new audio levels written in dB or as linear multipliers?](https://github.com/MBehtemam/Montagent/issues/800)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)) asked what unit
every audio capability's parameters use, and how an agent is kept from writing the wrong one.
This ADR fixes the units and the naming rule. It adds no members: each capability ADR names its
own keys and ranges within these rules.

## Decisions

### A level takes the unit its precedent uses; `volume` is the one linear level

A **level** parameter (EQ band gain, compressor threshold and make-up gain, limiter ceiling,
loudness target) is written in **dB**. A loudness target is in **LUFS**, and a true-peak ceiling
is in **dBTP**.

The rule: *a level takes the unit its precedent uses, unless it is a keyframable gain whose
identity must read as "on" at `1`.* `volume` is the only such field. It stays the linear
multiplier ADR-0055 made it, and it gets no dB sibling: `volume_db` beside `volume` would be two
spellings of one setting.

ADR-0055's case against dB was about `volume`, and it does not carry over:

- It objected that dB's identity value is `0`, which means "off" everywhere else in the format.
  For an EQ band or a make-up gain, `0` dB *is* "does nothing", so the reading agrees. A
  threshold, ceiling or target has no identity value at all.
- It objected that a reader would need log arithmetic to read a level. For these parameters it is
  the linear spelling that needs it: a threshold of `0.0631` or a ceiling of `0.891` is the
  arithmetic, and a loudness target has no linear spelling anyone writes.
- Premiere shows every one of these in dB, LUFS or dBTP, and so does every broadcast spec. An
  agent knows "−18 dB, 4:1, −1 dBTP, −16 LUFS"; it does not know their linear values.

The rule does not depend on these parameters never being keyframed. Whether they are animatable
is still open on the map. An EQ gain keyed through `0` dB still passes through "no change".

ffmpeg is mixed: `equalizer` takes dB and `loudnorm` takes LUFS and dBTP, but `acompressor`'s
threshold and make-up and `alimiter`'s limit are linear. The renderer converts. The document's
unit never follows a filter's.

Rejected:
- **Everything linear.** A loudness target has no linear form, and linear thresholds and gains
  are unreadable and match no precedent.
- **Reopen `volume` to dB.** That reverses ADR-0055, breaks every existing document, and turns a
  fade to silence into a fade to `-inf`.

### A key carries its unit exactly where the format has two scales for one quantity

Today no key names its unit: `start` is ms, `rotation` degrees, `stroke_width` px. The rule above
gives *level* two scales, linear `volume` and dB everything else. So every logarithmic level key
carries a suffix naming its measurement:

| Suffix | Measurement | Example keys |
|---|---|---|
| `_db` | a gain, or a level against sample peak (dBFS) | `gain_db`, `threshold_db`, `makeup_db` |
| `_lufs` | integrated loudness (ITU-R BS.1770) | `target_lufs` |
| `_dbtp` | true peak | `ceiling_dbtp` |

The examples show the pattern; each capability ADR names its own keys.

**The rule is a test, not a style.** A key carries a unit suffix if and only if the format has two
scales for its kind of quantity. Level qualifies. Frequency, effect durations, ratio and fractions
each have one scale format-wide, so they stay bare. `volume` stays bare as the linear default. A
later capability whose level is in dB (a per-channel Channel Volume, say) takes `_db` under this
rule.

The suffix closes the one silent mistake `validate`'s ranges leave open. A threshold, ceiling or
target is at or below `0`, so a linear-minded `0.125` there is out of range and fails. A gain is
not: EQ band gain and make-up gain take small positive dB values, so `"gain": 0.5` meant as "half"
would pass as +0.5 dB. With the suffix, the agent reads the unit in the key it is typing, and a
bare `gain` is an unknown key in a closed vocabulary, which `validate` rejects.

The suffix also says *which* measurement: a sample-peak ceiling and a true-peak ceiling are
different numbers for the same signal, and `ceiling_db` and `ceiling_dbtp` cannot be confused.

This copies Premiere's labels, which carry the unit in the same place: "Gain (dB)", "Threshold
(dBFS)", "Target Loudness (LUFS)". It is the format's first unit suffix, and this ADR names that
departure with the rule that bounds it.

Rejected:
- **Bare names everywhere.** It keeps the convention and leaves the gain hole open, guarded only
  by docs an agent editing by exact-string replace may never read.
- **Suffix every audio unit** (`frequency_hz`, `attack_ms`). It adds suffixes where nothing is
  ambiguous, sets audio against every timeline field (`attack_ms` beside `start`), and dilutes
  what a suffix signals.

### Pan is −1..+1

Pan is a bare number from `-1` to `1`: `0` is centre and negative is left. Premiere writes
−100..+100. ADR-0055 rejected a 0–100 scale for `volume` as "the same freedom with a different
decimal point, clashing with every other scalar in the format", and the same reason applies here.
The sign convention is Premiere's. An agent that writes Premiere's `50` is out of range and fails
`validate`. The pan law is the pan ADR's to decide.

### The other units

| Quantity | Unit | Note |
|---|---|---|
| Frequency | Hz, a plain number | Never kHz. A capability's range (e.g. 20–20000) rejects a stray `2.5` meant as kHz. |
| Effect-internal duration (attack, release, hold, look-ahead, echo delay) | ms, **may be fractional** | See below. |
| Ratio | a bare `n` meaning n:1, `n ≥ 1` | `4` is 4:1. |
| Mix, dry/wet, feedback | a 0..1 fraction | Not Premiere's %, for the reason pan gives. |
| Pitch | semitones, fractional allowed | No separate cents field: +30 cents is `0.3`. The pitch ADR decides the rest. |

**Effect-internal durations may be fractional ms.** ADR-0005's integer rule governs *instants on
the timeline*: the values `shift` moves and the frame grid samples. An attack time is neither. It
is a coefficient inside a filter, applied at the audio rate, and nothing shifts or snaps it. Fast
compressor and limiter attacks run 0.1–1 ms, so integer ms would make them unwritable. They stay
in ms, never µs or s, so the format keeps one time unit. `start`, `end` and keyframe `t` remain
integers.

### Departures from Premiere, recorded together

Pan at ±1 instead of ±100, mix and feedback at 0..1 instead of %, and pitch with no cents field.
All three follow the format's own scalars rather than a slider's display scale.

## Consequences

- Every audio capability ADR names its level keys with `_db`, `_lufs` or `_dbtp`, and its other
  parameters bare in the units above.
- `CONTEXT.md`'s **Volume** and **Audio effect** entries state the split.
- `docs/agents` format rules gain one line per suffix when the first member lands.
- A future quantity that gains a second scale takes a suffix under the test above. One that keeps
  one scale does not.

## Evidence

Two three-juror courts (Opus, Sonnet, Fable), run blind with no recommendation in the packet,
with the owner ruling with the Judge on both. Packets and ballots are in
[`docs/research/juries/audio-units/`](../research/juries/audio-units/BALLOTS.md).

- **Round 1:** dB for every new level with `volume` the one linear exception: 3/3. Pan −1..+1:
  3/3. The other-units package with fractional effect durations: 3/3. The Judge noted one
  juror's premise that these parameters are "never keyframed" is not decided; the rule above
  stands without it.
- **Round 2:** suffix only the logarithmic levels: 3/3. The round's packet stated the bounding
  rule as part of option (b), which favoured it in framing.

Precedent rows: [`docs/research/audio-effects/PRECEDENT.md`](../research/audio-effects/PRECEDENT.md)
§1 (loudness), §2 (pan), §5 (EQ), §6 (dynamics), §9–11 (reverb, delay, pitch).
