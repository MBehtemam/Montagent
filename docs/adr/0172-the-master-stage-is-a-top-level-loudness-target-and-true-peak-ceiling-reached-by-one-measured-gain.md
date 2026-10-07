---
status: accepted
amends: 0055 (clipping past the summed mix is no longer only "the renderer's documented behaviour" when the project sets a ceiling), 0117 (`verify` gains loudness and true-peak measurements)
---

# The master stage is a top-level loudness target and true-peak ceiling, reached by one measured gain

> **Amended by [ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md)**: sets `verify`'s tolerances. A miss against
> `target_lufs` is a `review` beyond ±1.0 LU, and a miss against `ceiling_dbtp` beyond +0.5 dB on the
> decoded AAC (provisional until ADR-0173 §7's measurement).

[What is the master stage, where does it live, and what does it hold?](https://github.com/MBehtemam/Montagent/issues/801)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)) asked for the one
processing stage the map allows on the final mix. Effects attach to elements and to this stage,
never to tracks (ADR-0004; the map's Out of scope). This ADR fixes where the stage lives, what it
holds, and how its loudness target is reached. It decides no tolerance: the measured-check ticket
([#802](https://github.com/MBehtemam/Montagent/issues/802)) does.

## Decisions

### `master` is an optional top-level object with two keys

```json
"master": { "target_lufs": -16, "ceiling_dbtp": -1 }
```

- `master` is a top-level key of the project, placed after `output` in canonical key order
  (ADR-0041). It is the format's first audio setting outside an element.
- It holds a closed set of two keys. Both are optional and independent of each other:
  - `target_lufs`: the integrated loudness (ITU-R BS.1770, gated) the deliverable's mix is
    brought to.
  - `ceiling_dbtp`: the true-peak ceiling of a limiter on the mix.
- The keys are named under ADR-0170's suffix rule.
- Neither key is animatable. Each describes the whole programme, so a value that changes over
  time has no meaning.
- An empty `master: {}` is legal and behaves as if `master` were absent.

Rejected:
- **`mix: {…}`.** "Mix" already names the summing (`amix`) and reads as a bus.
- **Turning `output` into an object** (`{path, audio}`). It breaks every existing document, and
  it mixes a destination path with audio processing.
- **An `audio_effects` list on the master.** A court already ruled the stage must be a closed
  set, not a general bus with an effect chain. EQ or compression of a narration-over-bed mix
  belongs on the elements, where the agent can reason about each one.
- **Output channel layout.** It is an output-format property, like sample rate and codec, which
  stay hard-coded. It belongs to the channel-operations capability, not to master processing.

### Absent `master` leaves the mix exactly as summed

With no `master`, or with neither key set, the graph is today's graph and every existing
document renders byte-identically. No ceiling applies by default. A default ceiling would be
processing the file does not state, which breaks the literal-values invariant (ADR-0145). It
would also change every existing render. Clipping on a mix without a ceiling remains the
documented behaviour ADR-0055 describes, and `verify` now reports true peak, so it is visible.

### The renderer reaches `target_lufs` with one measured gain, never with `loudnorm`

The stage runs on the bus after `amix` and before the closing `apad`/`atrim`:

1. **Pass 1, measure.** The renderer renders the whole programme's mix, audio only, and measures
   its integrated loudness.
2. **Pass 2, apply.** It applies one fixed gain, `volume=<target − measured>dB`. If
   `ceiling_dbtp` is set, `alimiter` follows with `latency=1`, its `limit` converted from dBTP to
   linear.

With only `ceiling_dbtp` set, there is no measurement pass, and the limiter runs on the summed
mix.

`loudnorm` is never the processing filter. It silently falls back to dynamic mode when its linear
conditions fail, and dynamic-mode output varies with the CPU's SIMD features
([`FFMPEG-FILTERS.md`](../research/audio-effects/FFMPEG-FILTERS.md) §3.2). A plain gain and
`alimiter` are deterministic and SIMD-independent, and `latency=1` cancels `alimiter`'s 5 ms delay
(same research). Measuring by any BS.1770 meter is an implementation choice.

**Partial renders and `preview` use the whole-programme gain.** A `--from/--to` render
(ADR-0121) and a `preview` mix only `[from, to)`, but they apply the gain measured on the whole
programme. A span therefore sounds the same as it does in the deliverable. The measurement may be
cached on the audio-relevant inputs. That is an implementation choice, as long as a cached value
equals a fresh one.

**One gain, no compensation.** The gain is computed before the limiter. Heavy limiting can leave
the delivered loudness below target, for example −14.6 against −14. The renderer does not
measure again or iterate. A miss means this target is too hot for this mix's peaks, and the fix
belongs in the document: a lower target, or peaks reduced on the elements.

**Undefined loudness gets no gain.** If every block of the mix is gated out (silent or
near-silent), integrated loudness is undefined. No gain is applied, and the run reports a
finding. Any defined loudness gets its full gain, however large. There is no cap: a cap would
quietly miss a literal target, with no value in the file saying why.

Rejected:
- **An agent-written `gain_db`** (one juror's vote). It is fully literal, but every edit silently
  makes it wrong, and agents work in edit-render loops. The applied gain is instead reported, as
  below.
- **A write tool that measures and writes `gain_db`.** It writes the same stale number
  automatically, and adds a tool that changes the file.
- **Two-pass linear `loudnorm`.** It is exact when its conditions hold, and silently
  non-deterministic across CPUs when they do not.
- **Compensating for limiter loss** with a third pass, or iterating. These change the gain after
  partial renders have used it, and iteration is unbounded.

### `render` and `verify` report the measurement and the gain

- `render`'s result carries the pre-gain integrated loudness (`measured_lufs`) and the gain
  applied (`applied_gain_db`) whenever `target_lufs` is set. The gain does not appear in the
  file, so this report is how it stays visible.
- `verify` always reports the deliverable's integrated loudness and true peak as measurements,
  with or without `master`. An agent then has the numbers it needs to decide whether to add a
  master stage.
- `verify` raises findings only against values `master` sets: integrated loudness against
  `target_lufs`, and true peak against `ceiling_dbtp`. A miss is a `review`-class finding, never
  an error. The deliverable exists, the miss can be legitimate (limiter loss, or AAC adding peaks
  between samples after a correctly applied limiter), and the tolerance is set in #802.

### Checks

`validate` (errors, from the file alone):
- `target_lufs` ∈ [−40, −5].
- `ceiling_dbtp` ∈ [−12, 0].

`review` (document-level, borrowed thresholds, ADR-0061):
- `target_lufs` without `ceiling_dbtp`. A positive gain without a limiter can clip, and the
  finding names `ceiling_dbtp: -1` as the fix. It is not an error: `validate` cannot know the
  gain's sign before pass 1, and a target on a hot mix is pure attenuation.
- `target_lufs` louder than −9 or quieter than −31.
- `ceiling_dbtp` above −1. The deliverable is lossy AAC, and lossy-delivery guidance recommends
  −1 dBTP or lower.
- Headroom (`ceiling_dbtp − target_lufs`) under 6 dB. Heavy limiting and a loudness miss are
  likely; this predicts the shortfall described above before anything renders.

Measurement-time findings (from `render` and `verify`, since they depend on pass 1):
- Integrated loudness undefined, so no gain was applied.
- Applied gain above +20 dB, which raises the noise floor. The fix is element `volume` or a lower
  target.
- `verify`'s misses against target and ceiling, as above.

Rejected:
- **A hard error when the target is at or above the ceiling.** The headroom finding covers it,
  and the proposed physical-impossibility premise was shaky.
- **A `−20` floor for the ceiling.** A ceiling that low is a mistake, not a choice.

## Consequences

- The schema gains `master`. `CONTEXT.md` gains **Master stage**.
- The mix graph gains a bus stage (gain, then `alimiter`), and the renderer gains an audio-only
  measurement pass when `target_lufs` is set.
- `verify` gains integrated-loudness and true-peak measurement on every run. The R128 silence
  gate stays as it is.
- The candidate per-element limiter finding under `volume > 1` (ADR-0169) now has a master
  ceiling to point to.
- Per-element loudness normalisation (an `audio_effects` member) remains its own capability ADR.
  If that member is made singular, its loudness target is the element's own, and the master
  target applies on top.

## Evidence

Two three-juror courts (Opus, Sonnet, Fable), run blind with no recommendation in the packet.
The owner ruled with the Judge on both. Packets and ballots are in
[`docs/research/juries/audio-master-stage/`](../research/juries/audio-master-stage/BALLOTS.md).

- **Round 1:** `master`, with absent meaning today's graph: 3/3. A target reached by the
  renderer: 2/3; Juror 2 voted for an agent-written `gain_db`. `target_lufs` + `ceiling_dbtp`:
  2/3, with Juror 2's `gain_db` following from their Q1 vote.
- **Round 2:**
  - A review finding for a target without a ceiling, one gain without compensation, and `verify`
    always measuring with a miss never an error: 3/3.
  - Undefined loudness gets no gain and a finding, with no cap: 3/3. The +20 dB threshold: 2/3.
  - The ranges and review thresholds are the Judge's read of a three-way split.

Precedent: Premiere's export loudness normalisation applies a target with an optional true-peak
limiter. CapCut documents no mix-level processing.
([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) §1). The ffmpeg facts are in
[`FFMPEG-FILTERS.md`](../research/audio-effects/FFMPEG-FILTERS.md), backed by its re-runnable
check.
