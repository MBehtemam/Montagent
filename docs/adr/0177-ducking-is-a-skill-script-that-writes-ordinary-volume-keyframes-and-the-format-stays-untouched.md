---
status: proposed (becomes accepted once the three-leg table for the rendered-audio check in §6 is committed; until then the ±0.5 dB tolerance is provisional. The owner's blind listen is committed and recorded in §6)
amends: 0055 (discharges its deferral: the door it left open for "a write tool that computes and writes the keyframes" is a skill script, not an MCP verb, and the format gains nothing)
---

# Ducking is a skill script that writes ordinary `volume` keyframes, and the format stays untouched

[Does the ducking write tool enter now, and what does it read and write?](https://github.com/MBehtemam/Montagent/issues/804)
on the audio map ([#795](https://github.com/MBehtemam/Montagent/issues/795)).
[ADR-0055](0055-audio-mixing-model-volume-fades-ducking-deferred.md) refused automatic ducking as a
live relationship between two elements and left one door open: a tool that computes the dip and
writes it as ordinary `volume` keyframes, built if hand-authoring proved costly. That evidence has
arrived in a narrow form. A shipped skill script, `captions.py`, already ducks, and agents use it.
This ADR makes the duck its own script and settles what it reads, how it is spelled, what it does to a
`volume` already on the bed, and how it is measured. It conforms to
[ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md).

## Decisions

### 1. Ducking enters now as a standalone skill script; the format and the binary do not change

- The script is `skills/montagent-footage/scripts/duck.py`, generalised out of `captions.py`
  (Python, stdlib only, as that file is). `captions.py`'s `duck` entry delegates to it.
- **It is not an MCP tool.** The write-tool invariant ([ADR-0011](0011-tool-surface-reads-checks-renders.md))
  says a tool that writes "may only take a complete element… no tool takes a field name or an element
  id". A duck is two ids and some parameters. The only invariant-compliant shape would have the agent
  pass the bed element already carrying the keyframes it wants, which is the agent computing the duck
  itself. A script is outside the MCP surface and may take ids.
- **It is not a CLI verb yet.** The evidence for a script is in hand and the evidence for a binary
  verb is not; the verb count is asserted in tests, so adding one is a deliberate act. The door stays
  open for `montagent duck` until a second script needs the same keyframe maths. The schema-drift risk
  of a script that writes JSON is answered in §5: it ends by running `montagent validate`.
- The output is only ordinary `volume` keyframes. ADR-0055's rule stands: there is no live ducking
  field, and the file never records that a duck was applied.

Rejected: an MCP write tool (the invariant, above); a CLI verb now (court: Juror 1 only); and leaving
the duck buried in `captions.py` (a voice retime without a caption change has no refresh path, and an
agent doing narration over a bed does not find it).

### 2. It reads one span list, from a words file or from spans written in the call

- The input is a list of `{start, end}` in milliseconds, in the voice source's own time, mapped onto
  the timeline exactly as `captions.py` maps it today. A words file (`[{word, start, end}]`) is the
  same data with an extra key the script ignores. Explicit spans serve a voice with no aligner, such
  as per-sentence TTS clips whose boundaries the author already knows.
- **No silence detection.** It would put a signal-analysis judgement, with thresholds the file never
  records, inside the write path.
- **No coarse fallback to the voice element's `start` and `end`.** That is a different edit (a flat
  dip with no pauses), so it is not a default.
- With neither input, the script exits non-zero and its message names both inputs and the shape of
  each. It does not guess.

### 3. It is spelled in dB and milliseconds and writes linear `volume`

- `bed` and `voice` are element ids. One bed under one voice per call. Several beds are several calls;
  several voices are merged into one span list by the caller.
- Levels follow ADR-0170, where a key carries its unit exactly where the format has two scales for one
  quantity: `under_db`, `over_db`, `end_db`. Times are `ramp_ms`, `lead_ms`, `join_ms` and, for the
  optional fade-out, `fade_ms`.
- **Defaults:** `under_db` −15, `over_db` −6, `end_db` −1.5, `ramp_ms` 200, `lead_ms` 100,
  `join_ms` 600; `fade_ms` is off. The dB defaults reproduce today's linear 0.18, 0.5 and 0.85 to
  within 0.15 dB and each sits inside the range the footage skill quotes
  ([`check_duck_levels.py`](../research/audio-effects/ducking/check_duck_levels.py)).
- **What is written is always linear `volume`**, rounded to four decimals so exact-string replace and
  `compare` see stable strings. Rounding costs at most 0.01 dB for every level above −27 dB, which
  includes every default. The script prints the dB to linear mapping it wrote, so the agent sees both.
- `captions.py` accepts its old linear keys (`under`, `over`, `end`, `ramp`, `lead`, `join`, `fade`)
  for one release, with a deprecation warning. It passes its current default `fade` through when it
  delegates, so its behaviour does not change.

Rejected: linear levels in the spec (court: Juror 1). They are arguments to a call, not file keys,
and read-back is trivially equal; but the spec is where an agent thinks in dB ("drop 15 dB under
speech"), and the script prints the linear value it wrote.

### 4. It refuses to overwrite a keyframed `volume`, unless told to

- If the bed already has a keyframed `volume`, the script refuses. Its message says so and names the
  repair: pass `--replace`. With `--replace` it owns the bed's whole curve.
- **Its own earlier output needs `--replace` too.** The script does not recognise its own shape;
  that would be a guess. `--check` (§5) is the read-only way to ask whether a refresh is needed.
- **No merge mode.** A hand-written fade-in that overlaps the first ducked stretch has no unique right
  answer (multiply, take the lower, clip), and whichever one the script picked would be a derived value
  the file cannot show the origin of.
- **The fade-out is an optional argument of the same pass** (`fade_ms`, off by default). A replace
  rewrites the single `volume` list, so a separate fade writer would be wiped on every refresh. It lands
  on the bed's last drawn frame, as `captions.py`'s does. A fade-in is not an argument; a hand-written
  one is re-added after `--replace`.

### 5. The file keeps only the keyframes, and the script says the rest

- **No provenance record.** A note that a duck was applied, from which voice, at which depth, is the
  live relational field ADR-0055 refused in another form: every reader would have to decide whether it
  and the keyframes agree, and `validate` cannot check it because the words are not in the file. This
  is the pattern of ADR-0037 and ADR-0086: keep the residue, keep the recipe outside.
- **Staleness is found by `--check`.** It computes the keyframes the arguments would write, compares
  them with the bed's current `volume`, writes nothing, and exits non-zero naming the first instant
  that differs. The footage skill's retime checklist runs it whenever the voice element or its words
  file changes. If the words file is gone, the honest answer is to regenerate it from the aligner.
- **The script ends by running `montagent validate` and printing its findings.** That is how an agent
  sees the binary's own verdict, and it is the answer to the drift risk of a script that writes JSON.
- **`R-TRANSITION-VOLUME-STACK` is left alone.** A duck that crosses a transition's audio crossfade
  does stack two level changes, and that is the author's call. The script prints the transition windows
  its keyframes overlap. It does not duck less inside them, which would leave the bed loud under
  speech, the one outcome a duck exists to prevent.
- **`R-EASE-INERT` stays, and is not amended here.** The court asked the script to avoid the
  equal-keyframe pairs that trigger it. That cannot be done for a held level: holding a level between
  two ramps needs two keyframes of the same `v`; [ADR-0038](0038-ease-is-required-on-every-non-first-keyframe-record.md)
  requires an `ease` on the second; and [ADR-0052](0052-review-check-for-inert-ease-on-held-keyframes.md)
  fires on any two consecutive keyframes with an identical literal `v`, whatever the `ease` is.
  So a hold always trips the review. The script writes holds as it does today, and the skill
  documents the review as expected on a duck. This was argued from those ADRs' text and is now confirmed against the
  engine by [`tests/ease.rs`](../../crates/montagent-core/tests/ease.rs) (each hold of a duck's
  curve is one finding, under every `ease` in the vocabulary). Whether ADR-0052 should spare a hold a tool writes is a separate decision and
  is left to the map's fog.

### 6. The measured check (conforms to ADR-0173)

The capability is in two parts, so its check is too:

- [ ] **Conforms to ADR-0173.**
- [ ] **The script's maths, in the skill's CI.** Metric: the bed's `volume` read back through
  `montagent query --at`. Fixture: a project and a words file in the skill's fixtures. Instants:
  inside speech, inside a pause of at least `join_ms`, inside a pause shorter than `join_ms`, after
  the last word, the ramp's start (`voice start − lead_ms`) and its end. Expected value: the linear
  value of the dB argument, rounded to four decimals, so the tolerance is the rounding step (0.0001);
  timings are exact integer milliseconds. This is where a script bug that writes the wrong instants is
  caught.
- [ ] **What the engine does with it, in the repo's Rust render tests.** Metric: RMS of the rendered PCM
  (before the encoder), in a window well inside speech and in one inside a pause, against the same
  bed unducked; the ratio must equal the level in dB. Fixture: a steady-tone bed under a project that
  already carries a known duck curve. Two-sided, **±0.5 dB, provisional**. Ramp edges are exact to
  ±1 sample, which is [ADR-0175](0175-a-keyframed-volume-is-heard-on-the-sample-its-instant-names.md)'s
  promise, cited and not re-measured. The tolerance's origin is the jurors' figure; ADR-0173 §4 makes
  it max(2 × the spread across the three CI legs, the meter's resolution) once that table exists.
  This also catches a chain-order regression, an effect placed after `volume` pushing a dip back up
  (ADR-0169).
- [ ] **Bypass identity.** A document with no duck is unchanged, and `captions.py` with no `duck` entry
  produces the same project as before.
- [ ] **The level arithmetic.** [`check_duck_levels.py`](../research/audio-effects/ducking/check_duck_levels.py)
  (stdlib only; exits non-zero when a number stops holding): the dB defaults are within 0.15 dB of
  today's, each is inside the guidance's range, and four-decimal rounding costs at most 0.01 dB above
  −27 dB (scanned worst case 0.0099 dB).
- [x] **A/B.** The owner's blind listen is committed in
  [`VERDICT.md`](../research/audio-effects/ducking/VERDICT.md), with the clips and `KEY`, as ADR-0173 §8 asks
  (2026-10-08, ffmpeg 6.1.1). The bed unducked against the bed ducked by the defaults, on the shared fixture
  with the bed raised 12 dB: the owner heard the music as louder in the unducked clip and "mostly like a
  background" in the ducked one. Not exercised: the return in a long pause (this narration has none over
  600 ms), pumping, and the narration's clarity were not remarked on.

## Consequences

- A new skill script, `duck.py`, and a footage-skill section that points to it. `captions.py` delegates.
  No schema, `validate`, tool or verb change, and no change to the counts asserted in tests.
- `CONTEXT.md` gains **Duck**.
- ADR-0055 carries an *Amended by* banner and the README's column says so.
- **Not decided here** (the map's fog): whether `montagent duck` becomes a CLI verb; whether ADR-0052
  should spare a hold a tool writes; a fade-in argument; ducking under several voices in one call;
  and a ducking write tool over MCP, which the invariant rules out unless it takes complete elements.

## Evidence

One three-juror court (Opus, Sonnet, Fable), run blind with no recommendation in the packet. The owner
ruled with the Judge. The packet and ballots are in
[`docs/research/juries/audio-ducking/`](../research/juries/audio-ducking/BALLOTS.md).

- Ruling out an MCP write tool, one input shape from words or spans with no silence detection,
  refusing over a keyframed `volume`, no provenance record, `--check` for staleness, leaving
  `R-TRANSITION-VOLUME-STACK` alone, and measuring both the keyframes and the rendered audio: all 3/3.
- A skill script over a CLI verb: 2/3 (Juror 1 for the verb). dB in the spec: 2/3 (Juror 1 for
  linear). The fade-out as an optional argument of the same pass: 2/3 (Juror 3 would keep it out).
- The jurors' answers to avoiding `R-EASE-INERT` could not be taken (§5); the court did not weigh
  amending ADR-0052. Juror 3's claim that ADR-0176 gives an outro fade a native home is not taken:
  a fade to silence is not a crossfade between two elements.

Precedent: Premiere's auto-ducking (Duck Against, Sensitivity, Duck Amount in dB, Fades in ms) writes
ordinary volume keyframes, so a mechanism exists by ADR-0145's rule, and ours has the same shape
([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md)).
