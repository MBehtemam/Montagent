---
status: proposed (DRAFT; becomes accepted once the render-test table in section 7 is committed with its three-leg spread on the 7.1 floor, the 6.1.1 dry-branch risk in section 7 is resolved, and the owner has ruled the calls at the end; until then the tolerances there are provisional)
---

# Echo is one stackable audio effect with delay, feedback and mix, and its tail is cut at the element's end

[Capability ADR and hand-off spec: echo](https://github.com/MBehtemam/Montagent/issues/795) on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)), modelled on
[ADR-0183](0183-the-noise-gate-is-one-stackable-audio-effect-with-a-range-no-hold-and-an-rms-threshold.md),
[ADR-0180](0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md)
and [ADR-0179](0179-eq-is-four-stackable-audio-effects-built-from-butterworth-biquad-sections.md). No ticket
ruled echo's shape; this draft proposes one. **Reverb is explicitly out of this ADR**: the same prototype built
one, the owner could not tell it from the bypass, and it carries `afir` costs echo does not (see "Not decided
here"). This ADR fixes the name, the three keys and their ranges, what the 8-tap rendering is and is not, that
the tail is cut at `end` and why, the finding list, and the measured check. It conforms to
[ADR-0169](0169-audio-effects-are-an-ordered-list-before-volume-and-gain-and-routing-stay-flat.md),
[ADR-0170](0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md),
[ADR-0173](0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md),
the latency and tail rule of [#843](https://github.com/MBehtemam/Montagent/issues/843) (its resolution
comment; the repo's ADR-0143 is the render encoder, and the prototype's header cites it by mistake) and, for
the float list, ADR-0179 section 2. Evidence:
[`docs/research/audio-effects/echo-reverb/`](../research/audio-effects/echo-reverb/README.md)
(`prototype_echo_reverb.py`, `cut_cost.py`, `measurements-ffmpeg7.0.2.json` 36 of 36 and
`measurements-ffmpeg6.1.1.json` 34 of 36, the two failures being reverb's), the owner's
[`VERDICT.md`](../research/audio-effects/echo-reverb/VERDICT.md) (2026-10-08, ffmpeg 7.0.2), and a small probe
described in section 2.

## Decisions

### 1. One member, stackable; every key is required, nothing defaults, all static

```json
{ "name": "echo", "delay_ms": 300, "feedback": 0.5, "mix": 0.5 }
```

| Key | Range | Meaning |
| --- | --- | --- |
| `delay_ms` | 1..2000 (may be fractional) | the gap between the voice and the first repeat, and between repeats |
| `feedback` | 0..0.95 | the gain of each repeat relative to the one before it: repeat k is `feedback^(k-1)` times the first |
| `mix` | 0..1 | the level of the first repeat against the dry signal; `0` adds nothing |

- **No key defaults.** ADR-0145's rule, #816's, and ADR-0180's section 1; a missing key is the clean error
  naming the keys, and the documentation carries the worked example above. This is the minority call of
  ADR-0180 and stays the owner's to reopen for every dynamics-and-effects member at once.
- **Stackable (non-singular), ADR-0169.** A short slap and a long repeat on one voice is ordinary practice, and
  two copies are two independent tap lines. Unlike EQ there is **no stack cap**: an `aecho` is one cheap
  tap line with no recursion and no per-sample state beyond its buffer, so the cap's reason (a graph that grows
  without a sound reason) does not apply. There is no `N-ECHO-STACKED` note either (ADR-0183 has one for the
  gate): two echoes are not a likely mistake and the note would have no repair.
- **Ranges.** `delay_ms` 1..2000 is the README's proposal and Premiere's Delay ceiling (recalled, PRECEDENT
  section 10); `aecho` itself allows 90 000. 1 ms is 48 samples at the mix rate and reads as a comb filter
  rather than an echo, which is why it is the floor and not 0. `feedback` stops at 0.95 because the name
  promises a repeat that keeps coming; the 8-tap bound (section 2) means 1.0 would be eight equal repeats and
  then silence, a thing the key's name must not suggest. `mix` 0..1 is a fraction with no unit suffix
  (ADR-0170: the suffix is for logarithmic levels; a dB wet level was not what the precedent shows). Widening a
  range later is compatible; tightening is not, which is why it starts here.
- **A keyframe list on any key is a schema error** (section 6, #842).
- **Stereo is fixed.** Both channels take the same delays and gains; there is no ping-pong, no cross-feed and no
  filter on the repeats (Premiere's Analog Delay adds Trash and Spread, recalled; not here).

Rejected: a `tail_ms` key (section 3); a `decay` key (name collision, section 5); a `dry` key (the dry stays at
unity, section 2); a `taps` key (an implementation detail the author would have to reason about); a singular
member (a second echo is a legitimate second delay time).

### 2. How it is rendered, and what the numbers mean

```
aecho=in_gain=1:out_gain=1:delays=<d1>|<d2>|...|<dn>:decays=<g1>|<g2>|...|<gn>,atrim=end=<element duration>
```

with tap k at `d_k = k x delay_ms` and gain `g_k = mix x feedback^(k-1)`, k = 1.. up to **8**, stopping at the
first tap whose gain is under **0.001 (-60 dB)**. The output is

```
y(t) = x(t) + sum over k of  mix x feedback^(k-1) x x(t - k x delay_ms)
```

- **The dry path stays at unity and `mix` scales the repeats.** That is exactly a dry/wet crossfade of `x`
  against `x` plus the full tap line, but it is **not** Premiere's Delay, whose dry/wet fades the dry as the
  wet rises (recalled). With `mix` 1 the voice is still at full level, the first repeat equals it, and the
  result is up to 6 dB hotter before the later repeats are added. This is owner call 2. It is chosen here because
  the alternative (fade the dry) needs an `asplit`/`amix` dry branch, and that branch is exactly where ffmpeg
  6.1.1 loses the last 97 ms of the dry signal (section 7, README "Builds"). A dry-at-unity echo is one filter
  in line and has no such branch.
- **`in_gain` and `out_gain` must be written, both 1.** `aecho`'s own defaults are 0.6 and 0.3
  (`ffmpeg -h filter=aecho`), which would silently scale the dry path to 0.18 of its level (-14.9 dB). The prototype and this
  rendering write them explicitly.
- **It is a tap line, not recursion.** `aecho` has no feedback loop, so what the document calls `feedback` is the
  *ratio between successive repeats* of a bounded series. This is a named departure (section 8) and it has
  three honest limits, which the documentation states:
  1. **The series ends at the eighth repeat** (or sooner, at -60 dB). The ring is therefore at most
     `8 x delay_ms` (16 s at the cap) and does not lengthen with more `feedback`. A high `feedback` does not
     produce a longer tail, it produces a louder eighth repeat followed by silence: the last repeat sits at
     `20 log10(mix x feedback^7)` dB, which is -48 dB at the README's 0.5 / 0.5 but -17.5 dB at `feedback` 0.75
     with `mix` 1 and -12.4 dB at `feedback` 0.9 with `mix` 0.5. `N-ECHO-TAPS-CAPPED` (section 4) says so.
  2. **Gain is bounded but not 1.** The worst case, a signal coherent with all its repeats, is
     `1 + mix x sum of feedback^(k-1)`: +6.0 dB at the README's settings (measured mix peak 0.74 on the
     narration), +17.8 dB at `feedback` 0.95 and `mix` 1. The list runs in float (ADR-0179 section 2), so
     nothing clips inside the list; the master (ADR-0172) is what holds the file. `R-ECHO-HOT` (section 4).
  3. **The delay is truncated to a whole sample, not rounded.** Probed on a lavfi impulse through `aecho` on
     ffmpeg 6.1.1: `delays=300.4|600.6` put the repeats at +14 419 and +28 828 samples (14 419.2 and 28 828.8
     exact), and 0.5 ms landed at +24. The probe is not a committed script; section 7 makes it a test case.
- **`aecho` extends the stream by its longest delay.** The same probe, with no cut, returned a 4 s impulse file
  as 220 800 samples (192 000 plus the last delay, 28 800). So the rendering **must** cut the element back to
  its placed duration after the member (`atrim=end=...` in the graph above, the prototype's cut); the length
  must not depend on what `aecho` flushes at end of stream, which is exactly the reason #843 gave for refusing
  free tails ("`aecho` and `afir` flush differently at EOF").
- **Latency: 0 samples, nothing to compensate (#843).** An impulse at sample 24 000 stays at 24 000 with its
  value unchanged, at `mix` 0.5 and 1.0, with nothing before it (max |x| before the onset 0.0e+00), and the
  length is exact, on ffmpeg 6.1.1 and 7.0.2. The dry path is the input, so the onset is exact by
  construction. The renderer adds **no** `apad ... atrim` latency cancellation for this member; its only
  length-fixing is the end cut above. A bypassed member adds neither.
- **Position.** In the author's order in `audio_effects`, before `volume` (ADR-0169): after `aloop`, so the
  repeats run continuously across a loop seam and are cut at `end` (#843, "follows from 2"), and after `atempo`,
  so `delay_ms` is playback milliseconds and is **not** scaled by `speed`. The element starts with an empty
  delay line: the repeats of material before the element's `start` are not heard. The transition's gain
  (ADR-0176) sits after the list and sees the truncated signal.
- **The rate** is the mix bus's fixed 48 kHz; the tap offsets are `floor(k x delay_ms x 48)` samples (above).
- **Determinism.** Byte-identical between ffmpeg 6.1.1 and 7.0.2 over the whole 20 s mix, between scalar
  (`-cpuflags 0`) and SIMD, and on a rerun. Tolerances are still the convention (ADR-0173 section 1): the
  three-leg table commits the answer for the floor build and the other two legs.

### 3. The tail is cut at the element's `end`; `tail_ms` is a named departure, not admitted

#843 ruled that an effect's tail is truncated at `end`, that a literal `tail_ms` may be admitted later only
one member at a time and only after a rendered prototype the owner accepts, and that this ADR's prototype must
exercise the flush-with-the-source case on the shared narration. It did, and the evidence is:

- **What the cut loses (measured, `cut_cost.py`, narration element, `end` 19 425 ms).** With the tail kept, the
  first 100 ms after `end` runs at -29.9 dBFS, **9.5 dB under the element's own RMS** (-20.4 dBFS), then
  -28.9, -36.6, -35.9 and -34.9 dBFS per 100 ms. The repeat of the last words is wholly removed: 0.19 % of the
  energy but the part that tells an ear an echo exists. At the cut the last 10 ms is -32.9 dBFS, a step, not a
  fade. The cut render is the kept render's prefix bit for bit.
- **What the owner heard (VERDICT, one listener, one fixture, blind).** `echo-Y` was identified as the echo;
  and for `echo-tail-X` and `echo-tail-Y` (the echo cut at `end` against the same echo with 1500 ms kept) the
  owner wrote "echo-tail-x has echo same as echo-tail-y". The cut was **not heard**. The record's own reading is
  that "on this one fixture a literal `tail_ms` was not shown to be worth admitting".

So the rule stands: **the tail is cut, there is no `tail_ms`**, and `tail_ms` is recorded here as a **named
departure** (from a free tail; Premiere and CapCut also clip tails at the clip edge, as #843 records) that
**was prototyped, not admitted**. This is a result about one narration that ends on its last word, with the cut listener hearing a
step at about -33 dBFS; it does not say the tail never matters. It is reopened by the owner hearing a cut on a
harsher fixture (a percussive source, a long `delay_ms`, a quiet room), not by an agent preferring a key. The
author's fix, as #843 states it, stays: extend the element's source range past the last sound, or end the source
with silence the effect can ring into.

### 4. Findings

| Code | Class | Fires when | Repair |
| --- | --- | --- | --- |
| *(schema)* | error | a key missing or unknown, a keyframe list on any parameter (#842) | the existing schema errors |
| `E-ECHO-RANGE` | error | `delay_ms`, `feedback` or `mix` outside its range, or not finite | a value: the nearest bound |
| `R-ECHO-HOT` | review | the product, over the element's enabled `echo` members, of `1 + mix x sum of feedback^(k-1)` (the section 2 taps) exceeds **3** (about +9.5 dB) | none |
| `N-ECHO-NO-OP` | note | no tap survives: `mix` 0, or under 0.001 | none |
| `N-ECHO-TAPS-CAPPED` | note | the eighth tap is still above -40 dB: `mix x feedback^7 > 0.01` | none |

- **`R-ECHO-HOT`** is a static estimate for a signal coherent with its repeats and can be wrong either way (a
  voice rarely is), so it is a review, as `R-COMPRESSOR-MAKEUP-CLIP` is. The README's 0.5 / 0.5 sits at 2.0 and
  does not fire; `mix` 1 with `feedback` 0.5 is 2.99 and does not; `mix` 0.5 with `feedback` 0.8 is 3.08 and
  does. The 3 is a judgement (owner call 7), not a borrowed threshold, and a test never reads it.
- **`N-ECHO-TAPS-CAPPED`** is the honest signal of the section 2 departure: it fires when the series is being
  chopped rather than dying away.
- **`N-ECHO-NO-OP`** is the bypass-identical case (section 5) seen from the author's side.
- **There is no `R-AUDIO-TAIL-CUT`.** See below.
- Only definite breakage is an error; the other three are judgement or a restatement, none has a repair
  (ADR-0043, ADR-0061).

#### Should `R-AUDIO-TAIL-CUT` exist? Recommendation: no, document it instead

#843 asked for "one review, not an error" when a tailed member sits on an element whose `end` reaches its
source's end with no loop, and left the id and wording to this ADR, and the README added that it must be checked
not to fire on every TTS clip. It would. An element whose `end` is the end of its source is the normal case for
a narration clip, and the shared fixture is exactly that. So the finding would be present on nearly every
document that puts an echo on a voice, and:

- **It would be noise the owner's own ear says is not worth acting on.** The cut was not heard on the fixture
  (section 3). A review that always fires and whose fix (extend the range, add silence) the listener gave no
  reason to make trains agents to dismiss reviews, which spends the credibility of the other reviews.
- **It cannot be targeted.** What it ought to say is "the tail you are cutting carries energy". `validate`
  cannot see that, nor sound-carrying media past `end` (README), so it has only the always-true structural
  condition.
- **It has no repair and no threshold.** It names a cost, not a defect.

So: **no finding.** The cost is stated where an author or agent reads about the member: `format.md`, and the
`query`/`compare` description of the member, carry "the repeats stop at the element's `end`; the last word's
echo is lost when the element ends on it; extend the source range or end the source with silence". If the owner
wants a signal anyway, the smallest honest form is a **note**, `N-ECHO-TAIL-CUT`, with the cut length computed
from literals (`min(8, taps) x delay_ms`, about 2.4 s at 300 / 0.5 on the README's settings, from the section 2
tap list) and **named effect index in ADR-0146's form** (`audio_effects[1].delay_ms (echo)`); it would not be a
review. This overrules the #843 follow-on that expected a review, which is why it is owner call 1. The
condition is recorded so the finding can be added without a design if a listener ever hears the cut.

### 5. Names: `echo` and `feedback`

- **`echo`, not `delay`.** Premiere names the effect Delay (with Analog Delay and Multitap Delay beside it);
  CapCut offers only an opaque "Echo" voice-filter preset, no parametric delay
  ([`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) section 10). The map's draft said `echo`. The
  reasons for `echo`: it is what an author or agent asks for ("add an echo to the narration"); `delay` is
  already a word in this format's audio path, where the per-element graph ends `... adelay -> amix` and an
  element is placed by start time, so a `delay` member invites an agent to reach for it to shift a clip; and
  the rendering is a bounded series of repeats, which `echo` describes and `delay` (one time-shift) does not.
  `delay_ms` stays as the *key* because inside `echo` there is no ambiguity and it is Premiere's parameter name.
  A single non-repeating time-shift, if ever wanted, is not this member.
- **`feedback`, not `decay`.** Premiere's parameter is Feedback (recalled, a percentage there); ADR-0170 says
  `feedback` 0..1 with no unit suffix for a single-scale quantity, and the README says the same. `decay` is
  worse in this ADR's own scope: `aecho`'s `decays` mean a tap *gain*, while the reverb candidate's `decay_ms` is
  a *time* to -60 dB (and Premiere's Studio Reverb Decay is a time too, recalled), so one word would name a
  gain in one member and a duration in the other. The cost of `feedback` is that it overstates the rendering
  (section 2); the documentation says "the ratio of each repeat to the one before it".

### 6. Not animatable

`aecho` accepts no runtime commands (README; the harness of FFMPEG-FILTERS section 4.2 is the one ADR-0183
section 5 used). Under #842 a filter that drops or ignores commands is never admitted as keyframable, so
**every key is a static literal and a keyframe list on one is a schema error.** This is a named departure from
Premiere, whose Delay parameters are keyframable.

### 7. The measured check (conforms to ADR-0173)

- [ ] **Conforms to ADR-0173.**
- [ ] **Impulse taps (the anchor; the three-leg measured test).** Metric: sample index and value of every
  non-zero sample of a rendered PCM impulse (the real mix graph with the encoder swapped for PCM, ADR-0173
  section 3), against the same impulse with the member `enabled: false` **in the same run** (the delta against
  bypass, ADR-0173 section 4; never against a stored render). Fixture: a lavfi impulse at sample 24 000 of a
  *stereo* source (`aevalsrc` mono up-mixed to stereo scales an impulse by 0.7071; the test measures ratios to
  the dry impulse so it does not matter, but it must be known). Settings: `(delay_ms, feedback, mix)` =
  (300, 0.5, 0.5), (300, 0.5, 1), (100, 0.9, 0.5), (1, 0.95, 1), (2000, 0.95, 1), (300.4, 0.5, 0.5) and
  (250, 0, 1). Expected, from the filter's definition: tap k at `floor(k x delay_ms x 48)` samples after the dry
  impulse, **exact**, with value `mix x feedback^(k-1)` times the dry impulse's, for k up to the first tap under
  0.001 or 8; no other non-zero sample. Tolerance: position **0 samples, exact**; amplitude **2e-6 absolute on a
  unit impulse, provisional** (the probe agreed to float precision, 0.35355338 against 0.5 x 0.70710677);
  ADR-0173 section 4 replaces it with max(2 x the three-leg spread, the meter's resolution) once the table is
  committed. Run on **all three CI legs** (Linux at the 7.1 floor, macOS on Homebrew, Windows on Chocolatey).
- [ ] **Onset is 0 samples (#843: 0 samples or stay out).** On the same impulse: the first non-zero sample is
  at 24 000 and its peak is at 24 000, at `mix` 0.5 and 1.0, **exact**, with nothing before it (the evidence
  runs read max |x| before the onset as 0.0e+00, and it is asserted at exactly 0, not at a threshold, since
  the dry path is the input). Evidence: `onset_latency` in both measurement files.
- [ ] **Length is exact.** The rendered element is exactly its placed duration, with the tail cut: asserted at
  `delay_ms` 2000 (where `aecho` alone would add 16 s) and on an element that ends exactly where its source
  ends. This is the case where `aecho`'s unconditioned flush (section 2) would show.
- [ ] **The cut render is the kept render's prefix.** Rendering the same element with a longer `end` and cutting
  it at the shorter equals the cut render, bit for bit within one run (measured 0.0 on both builds). It keeps
  the guarantee that the tail rule cuts and does not alter.
- [ ] **Smoke, as the spec requires for echo's tail (ADR-0173 section 5), on the shared narration.** 48 kHz
  stereo, exact length, no NaN/Inf, no clipping (peak 0.740 at 300 / 0.5 / 0.5), above the -70 LUFS gate
  (-16.8), not a no-op (difference -5.0 dB re bypass, floor -30), integrated loudness within 3 LU of bypass
  (+1.2). This is a smoke check *in addition to* the taps test, which gives echo an honest number; the
  narration supplies the tail energy figure above.
- [ ] **Range edges.** Every bound of every range (delay 1 and 2000, feedback 0 and 0.95, mix 0 and 1) renders
  without an ffmpeg error, so a document that passes `validate` never fails at render. The evidence has not
  yet exercised 1 ms or 2000 ms; the build does. A `delays` list longer than 8 is never produced (assert).
- [ ] **Bypass identity.** `enabled: false` PCM equals member-absent PCM, **and `mix` 0 PCM equals member-absent
  PCM** (the renderer emits no filter when no tap survives); the no-member graph is unchanged
  (ADR-0173 section 5). Measured: `mix` 0 byte-identical to the bypass render on both builds.
- [ ] **Same-build determinism** (identical bytes across runs and `-cpuflags 0`; measured identical).
- [ ] **Fractional delay.** `(300.4, 0.5, 0.5)` lands the repeats at +14 419 and +28 828 samples (truncation);
  if a future build rounds, this fails and section 2 is reopened.
- [ ] **Floor-build risk, resolved on 7.1 before acceptance.** On ffmpeg 6.1.1 the dry branch of reverb's
  `asplit -> amix` loses its last 4 656 samples (97 ms) when the source ends (README "Builds"; max difference
  0.13, -31 dB re signal, exactly the final-word region), and the same defect breaks `mix` 0 = bypass and
  cut = prefix for reverb. **Echo has no such branch, and its own 6.1.1 run passes all of its claims** (byte-
  identical to 7.0.2 across the whole mix). The risk for this ADR is therefore indirect and two-fold: (a) the
  per-element mix path (`amix`) at the floor ends the element at its source's end, which the length and
  flush-with-the-source case above must exercise on the 7.1 leg; and (b) if the owner takes call 2 and fades the
  dry, the rendering gains a dry branch and inherits the defect. The 7.1 floor was not available to the
  prototype (6.1.1 and 7.0.2 only), so nothing here is claimed for 7.1 until the three-leg table is committed.
- [ ] **Not exercised.** A looped element (the repeats should run across the seam, #843), a `speed` other than 1,
  a source with a `start` later than 0, a stack of two echoes, a mono or 5.1 source, and a window other than the
  README's. The build adds them; none can change a decision above except by failing.
- [ ] **Evidence.** The prototype and `cut_cost.py` are a throwaway starting point and their numbers are
  cited; the build ports the impulse-tap case into the repo test. The ad-hoc probe of fractional delays and
  `aecho`'s flush (ffmpeg 6.1.1) is not committed; the test cases above replace it.
- [x] **A/B.** [`VERDICT.md`](../research/audio-effects/echo-reverb/VERDICT.md), the clips and `ab/KEY` (sha256
  `88dc2f56...30d6`, 2026-10-08, ffmpeg 7.0.2): "echo Y has a echo". The owner heard the echo. The cut-tail
  pair was not told apart.

### 8. Departures from the Premiere precedent

[`PRECEDENT.md`](../research/audio-effects/PRECEDENT.md) section 10: Premiere's Delay (delay time up to 2 s,
Feedback %, Mix %; units recalled); CapCut only an "Echo" preset.

- **A bounded tap line, not recursive feedback** (section 2).
- **`mix` leaves the dry at unity**, where Premiere's dry/wet fades it (owner call 2).
- **The tail is cut at `end`; no `tail_ms`** (section 3). Premiere also clips at the clip edge (#843), so this is
  a departure only from a free tail.
- **Static parameters** (section 6), against keyframable ones.
- **No analog modelling, no spread, no multitap** (Analog Delay and Multitap Delay are not offered).

## Consequences

- The schema gains one `audio_effects` member and a `validate` pass for it (one error, one review, two
  notes). The tools that read a document describe it, the cut tail and the 8-repeat limit. `render` places the
  member's end cut after `aecho`; it adds nothing to a graph with no echo.
- `CONTEXT.md`'s **Audio effect** entry names the member.
- No new finding id depends on the source's length, so `validate` stays free of a probe for this member.
- Not decided here (the map's fog): **reverb** (the prototype's reverb was not told apart from the bypass at
  `mix` 0.3 and 1200 ms, one described "noise" under the voice, and it costs `afir`'s partition delay, a
  6.1 against 7.x normalisation split and a SIMD-dependent last bit; it needs its own ADR or none), a `tail_ms`
  key, a multitap or analog delay, a ping-pong or filtered repeat, an `echo` that crossfades its dry, a
  pre-delay, and animating any parameter (#842).

## Evidence

No court has been convened for this draft. The numbers are the prototype's and `cut_cost.py`'s on ffmpeg 7.0.2
and 6.1.1, one ad-hoc probe on 6.1.1 (section 2), and the owner's VERDICT. Premiere's parameter names and ranges
are recalled, not quoted. One listener, one voice, one fixture, as every prior listen. The #843 ruling is its
resolution comment; its text is quoted in section 3.

## Owner calls to confirm

1. **No `R-AUDIO-TAIL-CUT`** (section 4): documentation only, overruling the #843 follow-on that expected a
   review; or the smaller `N-ECHO-TAIL-CUT` note; or the review as ruled, accepting that it fires on nearly
   every TTS echo.
2. **`mix` leaves the dry at unity** (section 2), or fades the dry like Premiere (needs a dry branch, and
   inherits the 6.1.1 end-of-source defect until 7.1 clears it).
3. **The tail stays cut, `tail_ms` stays out** (section 3), on one listener's "same echo" on one narration.
4. **Stackable, no cap, no stack note** (section 1).
5. **`echo` and `feedback`** (section 5), against `delay` and `decay`.
6. **Ranges** (section 1): `delay_ms` 1..2000, `feedback` 0..0.95, `mix` 0..1.
7. **`R-ECHO-HOT` at 3 and `N-ECHO-TAPS-CAPPED` at -40 dB** (section 4): the thresholds are judgements, and
   either finding can be dropped.
8. **Eight repeats, truncated to the sample** (section 2), as the stated honest departure from feedback.
9. **No defaults** (section 1), the minority call of ADR-0180, for the member as for the others.
10. **Reverb is out of this ADR** (the opening and "Not decided here"), to get its own decision or none.
