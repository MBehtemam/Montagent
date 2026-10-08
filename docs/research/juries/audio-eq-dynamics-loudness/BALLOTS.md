# Ballots: loudness normalisation, EQ, and the compressor and limiter

Packet: [`QUESTION.md`](QUESTION.md), sent blind and identical to three jurors dispatched in parallel.
Tickets: [#844](https://github.com/MBehtemam/Montagent/issues/844),
[#845](https://github.com/MBehtemam/Montagent/issues/845),
[#846](https://github.com/MBehtemam/Montagent/issues/846). Ballots are verbatim, grouped by juror (each
juror answered Q1, Q2 and Q3 in turn). The owner ruled "go with the Judge" on the Judge's read below the
ballots. The ADRs are
[ADR-0178](../../../adr/0178-normalize-loudness-is-a-singular-audio-effect-with-one-authored-target-lufs-and-one-fixed-gain.md),
[ADR-0179](../../../adr/0179-eq-is-four-stackable-audio-effects-built-from-butterworth-biquad-sections.md) and
[ADR-0180](../../../adr/0180-the-compressor-and-the-limiter-are-two-audio-effects-with-no-defaults-and-an-rms-threshold.md).

## Juror 1 (Opus)

🗳️ **Juror 1** (Opus) — **VOTE: Q1 — `normalize_loudness {target_lufs}`, required, a literal from −50.0 to −10.0 LUFS (0.1 steps). The bed level is an authored absolute value (bed −38 / voice −23), with "bed ≈ 15 LU under voice" written only as a documented convention. Findings: errors for missing/out-of-range/keyframed/no-audio; review for target above master and for a big gain lift; note for undefined loudness and volume≠1. The measured check uses a lavfi pink-noise level test, a trim-window test, a silence test and a 15 LU voice/bed gap test.**

**Reasoning:** The ADR should fix:
- **Name:** keep `normalize_loudness` (singular member, per the accepted shape).
- **`target_lufs` range:** −50.0..−10.0 inclusive. This holds −38 for beds with headroom for quieter ambience. It stops above −10 because louder targets just fight the master limiter.
- **Bed target:** authored as an absolute value on each element, never as a relative offset or a per-type preset. This keeps the schema literal and every element self-contained. The ADR and the agent docs state the convention: dialogue/voice −23, music bed −38 under voice (15 LU below), and a bed alone at the master target.
- **Validate findings, error (each carries a repair):**
  - `target_lufs` missing
  - out of range or non-numeric
  - given as a keyframe list
  - member placed on an element with no audio stream
  - member given twice on one element (it is singular)
- **Validate findings, review:**
  - `target_lufs` above master `target_lufs` (one element louder than the whole mix is almost always a mistake)
  - measured gain above +20 dB at render/probe time (it would lift noise)
- **Validate findings, note:**
  - loudness undefined (silence, or a placed window under 400 ms so no gating block) → no gain applied
  - `volume` ≠ 1.0 alongside normalisation (the final level is `target_lufs` + 20·log10(volume))
- **Measured check (repo test):**
  1. Level: lavfi pink noise at −30 LUFS, normalised to −23. The rendered element's integrated loudness (ebur128) must be −23 ± tol.
  2. Window: a clip whose loud head is trimmed off. The gain must come from the placed window only, also checked under speed 2.0 and loop.
  3. Silence: anullsrc gives gain 0 dB, the "undefined" finding, and no error.
  4. Mix: voice −23 plus bed −38. The measured gap must be 15 LU ± tol.
  - Tolerance is max(2× three-leg spread, 0.1 LU).

**Trade-offs:** An absolute bed value costs the agent one subtraction, and two places can drift apart. I rejected a relative `relative_to`/offset key because it breaks per-element literalness and adds a reference graph. I rejected per-type presets because the accepted shape already ruled them out. The +20 dB review threshold is a judgement call.

🗳️ **Juror 1** (Opus) — **VOTE: Q2 — Members `highpass`/`lowpass {frequency_hz, slope_db_per_oct ∈ 12|24|48}` (Butterworth cascades), `shelf {side: low|high, frequency_hz, gain_db}`, and `bell {frequency_hz, gain_db, q}`. Ranges: frequency 20–20000 Hz and below Nyquist, gain ±24 dB, q 0.1–18. Findings: errors for range, Nyquist and keyframes; review for gain over 15 dB, a crossed HP/LP pair and more than 8 EQ members; note for a 0 dB no-op. The measured check uses lavfi tones.**

**Reasoning:** The ADR should fix:
- **Members and keys:**
  - `highpass` and `lowpass`: `frequency_hz`, `slope_db_per_oct`.
  - `shelf`: `side` ("low"|"high"), `frequency_hz`, `gain_db`.
  - `bell`: `frequency_hz`, `gain_db`, `q`.
  - All are non-singular and stackable.
- **Slope key:** the key is `slope_db_per_oct`, so the unit is in the key name as ruled. The set is {12, 24, 48}:
  - Each slope is built from 1, 2 or 4 biquads with Butterworth section Qs: 0.7071; 0.5412/1.3066; and the 8th-order set.
  - That way every slope measures −3.01 dB at the cutoff. Cascading identical Q 0.707 sections would read −6 dB or −12 dB there instead.
  - The shelf Q is fixed at 0.7071 (S=1) and is not authorable.
- **Ranges:**
  - `frequency_hz`: 20..20000, and also below 0.45 × the project sample rate.
  - `gain_db`: −24..+24.
  - `q`: 0.1..18.
  - No defaults; every key is required.
- **Validate findings, error:**
  - a key missing or unknown
  - a value out of range
  - a slope outside the set
  - `frequency_hz` at or above Nyquist (or above 0.45·sr) for the project rate
  - a keyframe list
- **Validate findings, review:**
  - |`gain_db`| > 15 (extreme)
  - `highpass.frequency_hz` ≥ `lowpass.frequency_hz` on the same element (the band is removed)
  - more than 8 EQ members on one element (stack cap)
- **Validate findings, note:**
  - `gain_db` = 0 on a shelf or bell (a no-op)
  - identical duplicate members
- **Measured check:**
  - Highpass/lowpass at 1 kHz, each slope: −3.01 dB at fc. At two octaves into the stopband: about −24, −48 and −96 dB for 12/24/48 (asserted as ≤ the theoretical value + tol). Passband (4·fc) within tol of 0.
  - Bell at 1 kHz with +6 / −6 dB: the centre tone hits `gain_db`, and a tone 3 octaves away reads about 0.
  - Shelf at 200 Hz, +6 dB: a 40 Hz tone reads +6, a 5 kHz tone reads about 0.
  - Tones come from lavfi `sine`; levels are measured as astats RMS.
  - Tolerance is max(2× spread, 0.05 dB).

**Trade-offs:** Butterworth-designed cascades take a few more lines than "repeat the biquad", but they keep the −3.01 dB contract honest for every slope. I rejected an authorable shelf Q and a `slope` key without its unit. I also rejected 6 dB/oct, because the first-order biquad behaves differently and buys little. The stack cap of 8 is a guess at what is sane.

🗳️ **Juror 1** (Opus) — **VOTE: Q3 — `compressor {threshold_db −60..0, ratio 1..20, attack_ms 0.1..2000, release_ms 1..9000, makeup_db 0..24}` (RMS detection, hard knee fixed) and `limiter {ceiling_db −30..0, release_ms 1..1000}` (5 ms lookahead fixed), all keys required with no defaults. Findings: errors for range and keyframes; review for compressor after limiter, risky makeup and a limiter ceiling above the master ceiling; notes for ratio 1 and stacked limiters. The measured check uses a stepped-tone transfer curve, a burst test and limiter tone/impulse tests.**

**Reasoning:** The ADR should fix:
- **Compressor:** `threshold_db`, `ratio`, `attack_ms`, `release_ms`, `makeup_db`.
  - Mapped to `acompressor` with `detection=rms`, `link=average` and `knee=1` (hard knee, not authorable), so the static curve has an exact formula to check against.
  - Ranges: `threshold_db` −60..0; `ratio` 1..20; `attack_ms` 0.1..2000; `release_ms` 1..9000 (inside acompressor's limits); `makeup_db` 0..+24.
  - No defaults. This matches the "required, no default" rule from #844, and an agent can't leave out timing it never chose.
- **Limiter:** `ceiling_db` (sample peak dBFS), range −30..0; `release_ms`, range 1..1000.
  - Mapped to `alimiter level=0 latency=1 attack=5`; the attack/lookahead is fixed and not authorable.
- **Validate findings, error:**
  - a value missing or out of range
  - an unknown key
  - a keyframe list
- **Validate findings, review:**
  - a compressor placed after a limiter in the same `audio_effects` list (wrong order)
  - `makeup_db` > −`threshold_db`·(1−1/ratio) + 3 with no limiter after it (static estimate that full-scale input exceeds 0 dBFS)
  - limiter `ceiling_db` above master `ceiling_dbtp` (the master overrides it, so the element ceiling is meaningless)
- **Validate findings, note:**
  - `ratio` = 1 (no-op; makeup is applied as gain only)
  - more than one limiter on one element
  - `attack_ms` > `release_ms` (unusual)
- **Measured check, compressor static curve:** lavfi sine steps from −50 to 0 dBFS in 5 dB steps, 1 s each. Steady-state RMS must match:
  - in + makeup below the threshold
  - thr + (in−thr)/ratio + makeup above it
- **Measured check, compressor burst:** the input jumps from −30 to −6 dBFS and back. Gain reduction must reach 63% of its final value within `attack_ms` ± tol, and recover 63% within `release_ms` ± tol.
- **Measured check, limiter:**
  - A sine 6 dB over the ceiling and an impulse train: output sample peak ≤ `ceiling_db` + 0.00001 dB.
  - Cross-correlation onset shift = 0 samples.
  - A sine 6 dB below the ceiling passes with gain within 0.01 dB.
- **Tolerance:** max(2× spread, meter resolution). No TTS anywhere in the check, as ruled.

**Trade-offs:** Requiring attack and release makes authoring more verbose than Premiere-style defaults. I accept that, because the schema prefers explicit literals and a missing value is a clean error with a repair. A fixed hard knee sounds less smooth than acompressor's default 2.83 knee, but it makes the transfer curve exactly checkable. I rejected an authorable knee and lookahead because both bring in variable latency, or a curve with no closed form. The makeup-clips check is a static estimate and can be wrong in either direction, which is why it is review, not error.

## Juror 2 (Sonnet)

🗳️ **Juror 2** (Sonnet) — **VOTE (Q1): `normalize_loudness {target_lufs}`, target_lufs required, range -40.0..-5.0 LUFS (finite, 0.1 resolution); bed target is an authored literal, with no convention baked into the schema. Findings: error = target_lufs missing / non-finite / outside range; review = resulting gain > +20 dB or < -20 dB; note = undefined loudness (silent/too short), so no gain applied. Measured check: lavfi sine/pink-noise at known level, render, measure integrated LUFS of the placed window, expect target within max(2x three-leg spread, 0.1 LU).**

**Reasoning:** The accepted shape has no default and no presets, so the schema should stay literal and closed. A convention such as "bed = voice - 15" would be a hidden default and would fail the exact-string-replace and validate-checkable rules. -40..-5 spans the prototype's -38 bed and -23 voice, plus loud delivery targets around -14, and rejects nonsense. Documenting "voice about -23, bed about 15 LU under" in the ADR and in the MCP tool description as guidance only keeps the value visible in the JSON. Only range violations are errors because only those have a repair (set a value in range). Silence is a runtime condition, not an authoring mistake, so it is a note rather than a review. The review for extreme gain catches a wrong target or a wrong element. The measured check uses synthetic lavfi signals with a known LUFS, not TTS, matching the lesson from #845. It also asserts that a trimmed, sped-up or looped element is measured on its placed window. A second assertion shows that a silent element produces gain 0 and the note.
**Trade-offs:** An authored bed target means the agent must compute -38 itself, and a wrong value is only caught by the extreme-gain review. I rejected a `relative_to_voice_lu` form and a `role` preset because they add a cross-element dependency and a default, and make the document no longer self-evident. I also rejected making the extreme gain an error: it can be intentional, and an error must carry a repair. The ±20 dB threshold is a judgement call that should be tuned.

🗳️ **Juror 2** (Sonnet) — **VOTE (Q2): members `highpass`, `lowpass` {`frequency_hz`, `slope_db_per_oct` in 12|24|48}, `shelf` {`side` low|high, `frequency_hz`, `gain_db`}, `bell` {`frequency_hz`, `gain_db`, `q`}. Ranges: frequency_hz 20..20000 and strictly below 0.45 x sample rate; gain_db -18..+18; q 0.1..10; max 8 EQ members per element. Findings: error = unknown slope, out of range, frequency at or above Nyquist (0.5 x sample rate), count over cap; review = |gain_db| > 12, or a highpass above 1000 Hz, or a lowpass below 2000 Hz; note = gain_db 0 (no-op) and duplicate identical stage. Measured check: lavfi sine tones through each member, expect -3.01 dB at cutoff (within max(2x spread, meter resolution)), bell/shelf reaching gain_db at centre/plateau, and slope per octave.**

**Reasoning:** The facts fix the shape, so the ADR only needs to close names and numbers. `slope_db_per_oct` carries its unit in the key name, as the ruled units convention requires, and the set 12|24|48 maps onto 2, 4 and 8 pole cascades of biquads. 20..20000 Hz covers the audible band. The frequency check is vs. the actual sample rate (Nyquist 0.5 x rate, or 24000 Hz at 48 kHz), so the schema bound is looser than the validate bound. A zero-gain stage is harmless, so it is a note. Extreme gain and band-limiting that would gut speech are intent-dependent, so they are review, not error. Only definite breakage is an error. The check uses tones because the owner could not hear a gentle high-pass on TTS; a tone gives an unambiguous level at the cutoff (-3.01 dB) and the bell gain. I pinned the cap at 8 to bound the filter-graph length and accumulated numerical error.
**Trade-offs:** The 8-stage cap and ±18 dB range are arbitrary but cheap to loosen later. Loosening is compatible, tightening is not, which is the reason to start strict. I rejected a single `eq` member with a bands array, since that is non-stackable by exact-string-replace and harder to validate. I rejected a free-form slope number (it would admit non-integer pole counts), and rejected `q` on the shelf (the facts give a shelf no q). Review thresholds (12 dB, 1000 Hz, 2000 Hz) are judgement calls.

🗳️ **Juror 2** (Sonnet) — **VOTE (Q3): `compressor` {`threshold_db` -60..0, `ratio` 1..20, `attack_ms` 0.01..2000 default 10, `release_ms` 0.01..9000 default 100, `makeup_db` 0..24 default 0}; `limiter` {`ceiling_db` -20..0, `release_ms` 1..8000 default 50}. Required: threshold_db, ratio, ceiling_db; attack_ms, release_ms, makeup_db optional with the defaults shown, written into the ADR. Findings: error = out of range, ratio < 1, ceiling above the master ceiling_dbtp when master is set; review = compressor after a limiter in the chain, makeup that raises the peak above 0 dBFS (clip risk), ratio above 10; note = ratio 1 (no-op), threshold at 0 (no-op). Measured check: static transfer curve using lavfi sine steps at several levels (output = threshold + (in - threshold)/ratio, RMS-referenced), plus a burst signal for attack/release time constants; limiter: ceiling held at 0.00001 dB sample peak on a hot burst, no onset shift, lookahead 5 ms.**

**Reasoning:** The ranges come from acompressor's native limits (threshold, ratio up to 20, attack 0.01..2000 ms, release 0.01..9000 ms), so no parameter can be accepted that ffmpeg would reject at render time. Defaults are justified for attack/release/makeup because those are the parameters an agent has no basis to choose and the owner could not hear differences on TTS; the required ones, threshold, ratio and ceiling, are exactly the ones that define the effect, so a default would hide the intent (same logic as `target_lufs` having none). Limiter lookahead is fixed at 5 ms (latency=1) so it is not a key. The master ceiling vs. element ceiling check is an error because limiting an element above the master's ceiling cannot ever bind and is a definite authoring contradiction with a repair (lower it). Ordering and clipping makeup are intent-dependent, so review. The compressor check cannot use TTS (already even), so it reads a static curve from steady tones against the RMS detector, and the burst signal exercises the attack and release time constants.
**Trade-offs:** Defaults add a documented implicit value, which weakens "every value visible in the JSON"; I accepted this only for the three non-defining parameters, and the validate output should state the default in use. I rejected defaults for threshold and ratio. I also rejected a `mode: peak|rms` key (peak detection read about 3 dB off, so it is fixed to RMS). The numeric ranges (e.g. makeup 0..24) are conservative choices beyond ffmpeg's hard limits. The ratio > 10 review is a judgement call.

## Juror 3 (Fable)

🗳️ **Juror 3** (Fable) — **VOTE: `normalize_loudness { target_lufs: -40..-6 }` (closed range, integer or one-decimal float), bed target is the authored value with a documented convention (voice −23, music bed −38, SFX −30 LUFS) in the member's doc string, five validate findings (2 errors, 2 reviews, 1 note), measured check = lavfi pink-noise element asserted to within ±0.5 LU of target by ebur128 on the rendered file.**

**Reasoning:**
- *Range −40..−6 LUFS.* Below −40 the gain pushes a bed into the dither floor and the integrated meter becomes unreliable; above −6 any real signal clips before `volume`, so the master limiter would be doing the loudness work the ADR says it must not. The range covers both prototype values (−23, −38) and EBU R128 / streaming targets (−14, −16, −18, −23). `validate` rejects out-of-range as **error** with the repair "clamp to nearest bound".
- *Bed target: authored value, not a convention token.* The ruled schema is closed vocabulary with literal values that are exact-string-replace editable. A convention token (`"bed"`) would be a second vocabulary, and would hide the LU offset an agent should see and reason about. So `target_lufs` is always a number; the ADR documents the convention table (voice −23, bed −38 = 15 LU under, SFX −30) in the schema description so agents author sane numbers without a preset mechanism.
- *Finding list:* 
  1. **error** `target_lufs` outside −40..−6 → repair: clamp.
  2. **error** `normalize_loudness` appears twice in `audio_effects` (singular member) → repair: delete the later one.
  3. **review** measured gain > +20 dB (source extremely quiet; likely wrong asset or wrong target) — no repair, needs a human/agent decision.
  4. **review** `normalize_loudness` followed by `compressor`/`limiter` whose `threshold_db`/`ceiling_db` is above the normalised peak estimate (i.e. the dynamics member will be a no-op after normalisation) — review, not error, because it may be intentional.
  5. **note** loudness undefined (element placed window < 400 ms or integrated below −70 LUFS gate) → no gain applied; this is the already-ruled "no gain + a finding", and it is a note because the render is well-defined.
- *Measured check:* repo test renders a project with one element whose source is a lavfi `anoisesrc=color=pink` (3 s, −30 LUFS raw), with `normalize_loudness { target_lufs: -23 }` and a second element targeting −38; runs `ebur128` over each element's placed window of the rendered file; asserts integrated loudness within ±0.5 LU of target. 0.5 LU = max(2× three-leg spread observed in the prototype ≈ 0.3 LU, ebur128 resolution 0.1 LU) rounded up to the EBU meter's own tolerance. A third element of 200 ms silence asserts "no gain + note" by checking the renderer's filter graph contains no `volume=` for that element and `validate` emits the note.

**Trade-offs:**
- Rejected per-type presets and a `"bed"` token: cheaper to author but a second vocabulary, and the owner's "15 LU under voice" is a relationship between two authored numbers, not a property of one element.
- Rejected a wider range (−60..0): it would let `validate` pass projects whose master limiter is doing all the work, contradicting the one-measured-gain rule.
- Rejected making the "gain > +20 dB" finding an error: an agent cannot repair it mechanically (the right fix is a different asset or target), and errors must carry a repair.
- Cost: the ±0.5 LU tolerance is coarser than the meter can read, so a regression of 0.3 LU would pass. Accepted because the three-leg spread is the ruled basis.

---

🗳️ **Juror 3** (Fable) — **VOTE: members `highpass`, `lowpass`, `low_shelf`, `high_shelf`, `bell`; key `slope_db_oct` ∈ {12, 24, 48}; `frequency_hz` 20..20000 (integer), `gain_db` −24..+24 (one decimal), `q` 0.1..10 (one decimal); stack cap 8 EQ members per element; six findings (3 errors, 2 reviews, 1 note); measured check = lavfi sine sweep at the named frequency, asserting −3.0 ± 0.2 dB at cutoff, shelf/bell gain ± 0.2 dB at centre, and 0 ± 0.1 dB far from the band.**

**Reasoning:**
- *Names.* I reject the `shelf { side }` framing from the accepted shape. `side: "low"|"high"` is a second enum inside a member when the closed vocabulary already has room for two members; `low_shelf` / `high_shelf` are exact-string-replace editable, grep-able, and match Premiere's and CapCut's labels (Low Shelf / High Shelf). `highpass`/`lowpass` match ffmpeg filter names one-to-one so the renderer mapping is trivially auditable. `bell` stays (Premiere "Peak", CapCut "Bell"; ffmpeg `equalizer`).
- *Slope key.* `slope_db_oct` carries the unit in the key name like `_db`/`_hz`/`_ms`, and the ruled unit rule says units live in the key. Set {12, 24, 48} maps to `poles=2` ×1, ×2, ×4 cascaded biquads (ffmpeg `highpass`/`lowpass` are 2-pole = 12 dB/oct; 24 and 48 are two and four instances chained). All zero latency, identical across builds, as ruled.
- *Ranges.* `frequency_hz` 20..20000: integer, so agents cannot author 19999.5; 20000 is below Nyquist for 44.1 k and 48 k; `validate` additionally errors if frequency ≥ project sample rate / 2 (matters if a project is 32 kHz). `gain_db` −24..+24: covers Premiere's ±20 and CapCut's ±15 with headroom; beyond ±24 a biquad shelf starts producing audible instability in float and nobody mixes there. `q` 0.1..10: ffmpeg `equalizer` uses `width_type=q`; below 0.1 the bell is a shelf, above 10 it is a notch and belongs to a future `notch` member.
- *Finding list:*
  1. **error** `frequency_hz` ≥ sample_rate/2 or outside 20..20000 → repair: clamp to nearest bound.
  2. **error** `slope_db_oct` not in {12,24,48} → repair: snap to nearest member of the set.
  3. **error** more than 8 EQ members in one element's `audio_effects` → repair: drop the trailing ones past the eighth. Eight is Premiere's Parametric EQ band count; past it the stack is a sign of generated noise, not intent.
  4. **review** `gain_db` is 0 on `bell`/`low_shelf`/`high_shelf` (no-op member) — review not error, because an agent may be scaffolding and the render is correct.
  5. **review** |`gain_db`| > 12 (exaggerated EQ; the owner could tell it apart blind so it is a creative decision, flagged for a human ear) — no repair.
  6. **note** two `highpass` members where the lower cutoff is more than two octaves below the higher (the lower one is almost certainly a no-op) — note because stacking is allowed and the render is well-defined.
- *Measured check.* The ruled fact is that a gentle high-pass on TTS was inaudible, so the check uses lavfi tones. For each member type one project element whose source is `sine=frequency=F` at three frequencies (F = cutoff, F = cutoff × 4 or ÷ 4 depending on filter sense, F = 1 kHz reference). Render, measure RMS with `astats`, compute dB relative to the unfiltered render. Assertions: at cutoff −3.0 ± 0.2 dB for all three slopes (biquad Butterworth is −3.01 dB independent of pole count); two octaves away ≤ −(slope × 2) + 1 dB (so 12 → ≤ −23, 24 → ≤ −47, 48 → ≤ −95, with the 48 case asserting ≤ −80 because astats noise floor); bell/shelf at centre reaches `gain_db` ± 0.2 dB; reference tone two decades away 0 ± 0.1 dB. 0.2 dB = max(2× three-leg spread ~0.05 dB, astats 0.01 dB) padded to the lavfi sine's own non-integer-period leakage; the measured −3.01 lands inside it.

**Trade-offs:**
- Rejected `shelf { side }`: one fewer member but an in-member enum, and `side` is a word no editor uses in its UI.
- Rejected a `slope` key without unit: breaks the ruled unit-in-key rule.
- Rejected ±48 dB or unbounded gain: an agent can still achieve any curve by stacking; a tighter range catches more typos.
- Rejected a cap of 4 or no cap: 4 would reject a realistic highpass + 3 bells; no cap lets a runaway agent author a hundred filters and ffmpeg would happily render them.
- Cost: two more member names than the accepted shape; `low_shelf`/`high_shelf` doubles the doc-string surface. Cost: the 48 dB/oct two-octave assertion is loosened to −80 dB because `astats` on a 16-bit intermediate cannot resolve −95.

---

🗳️ **Juror 3** (Fable) — **VOTE: `compressor { threshold_db −60..0, ratio 1.0..20.0, attack_ms 1..200 (default 20), release_ms 10..1000 (default 250), makeup_db 0..+24 (default 0) }`; `limiter { ceiling_db −20..0 (required, no default), release_ms 10..1000 (default 50) }`; seven findings (3 errors, 3 reviews, 1 note); measured checks = static transfer curve on a stepped lavfi sine (compressor) and a 0 dBFS burst (limiter), tolerance 0.3 dB on the compressor curve and 0.01 dB on the limiter ceiling.**

**Reasoning:**
- *Key names.* Keep the accepted names; every one carries its unit in the key (`_db`, `_ms`, `ratio`) as ruled. `ceiling_db` is sample-peak as ruled; the ADR says so in the doc string because the master `ceiling_dbtp` is true-peak and agents will otherwise conflate them.
- *Compressor ranges and defaults.* `threshold_db` −60..0 (acompressor's own range is −60..0 dB; RMS detection as ruled, and the doc string says "reads against RMS, roughly 3 dB above the equivalent peak threshold"). `ratio` 1.0..20.0 one decimal (acompressor caps at 20; 1.0 is a legal no-op that validate flags). `attack_ms` 1..200 default 20, `release_ms` 10..1000 default 250: these are acompressor's defaults and Premiere's Tube-Modeled Compressor defaults are in the same decade, so an agent that writes only `threshold_db` and `ratio` gets a sensible compressor. Defaults are allowed here and not on `normalize_loudness` because attack/release are timing parameters with an industry-standard idle value, whereas a loudness target without a value is a content decision. `makeup_db` 0..+24 default 0; negative makeup is `volume`'s job.
- *Limiter ranges and defaults.* `ceiling_db` −20..0, required, no default: a limiter without a ceiling is meaningless, and the ruled master `ceiling_dbtp` already provides the delivery ceiling, so this one is per-element intent. `release_ms` 10..1000 default 50 (alimiter default). Lookahead fixed 5 ms and `level=0 latency=1` are render semantics already ruled, not schema.
- *Finding list:*
  1. **error** any parameter outside its range → repair: clamp.
  2. **error** `limiter.ceiling_db` above master `ceiling_dbtp` − 1 dB (the per-element limiter would be undone by the master, and the sample-peak/true-peak gap eats the 1 dB) → repair: set to master `ceiling_dbtp` − 1.
  3. **error** `compressor` appears after `limiter` in the same `audio_effects` list → repair: swap them. A compressor after a limiter re-exposes peaks the limiter just caught; no precedent editor allows it (Premiere's Essential Sound and CapCut both fix the order).
  4. **review** `ratio` = 1.0 (no-op compressor) — no repair, may be scaffolding.
  5. **review** `threshold_db` + `makeup_db` > 0 with no later `limiter` in the chain and no master ceiling (makeup can clip) — no repair, needs a decision between adding a limiter and reducing makeup.
  6. **review** `attack_ms` < 5 with `ratio` ≥ 10 (brick-wall compressor; an agent probably wanted `limiter`) — no repair.
  7. **note** `limiter` present on an element and master `ceiling_dbtp` is also set (double limiting is legal and common, but the doc string tells the agent which one governs delivery).
- *Measured checks.* The ruled fact is that the compressor could not be told apart on even TTS, so the test is a transfer curve, not TTS. Compressor: a lavfi `sine=1000` stepped at −40, −30, −20, −12, −6, 0 dBFS RMS (each step 2 s, so attack/release settle) through `compressor { threshold_db: -20, ratio: 4.0, attack_ms: 5, release_ms: 50, makeup_db: 0 }`; measure RMS of the last second of each step with `astats`; assert output = input below −20, and output = −20 + (input + 20)/4 above it, within ±0.3 dB (max of 2× observed spread ~0.1 dB and acompressor's RMS window settling; the ruled ~3 dB peak-vs-RMS discrepancy is why the test uses RMS on both sides). Burst half: a 50 ms 0 dBFS burst in silence asserts attack reaches 90% of gain reduction within `attack_ms` ± 1 ms by sampling the envelope. Limiter: 0 dBFS lavfi sine burst through `limiter { ceiling_db: -3.0, release_ms: 50 }`; assert sample peak ≤ −3.0 dBFS within 0.01 dB (the ruled 0.00001 dB hold makes this near-exact; 0.01 is the `astats` print resolution), and assert first non-zero output sample is at the same index as the input's (no onset shift, as ruled).

**Trade-offs:**
- Rejected no defaults on attack/release: it would make the compressor five required keys for an agent who only knows "squash it a bit", and the ruled first-wave values are static so defaults are safe to document.
- Rejected a default `ceiling_db`: it would silently duplicate the master and make the "ceiling above master" error fire by default.
- Rejected making compressor-after-limiter a review: it has a mechanical repair (swap) and no legitimate use in this schema, so it is an error.
- Rejected a `knee_db` key: acompressor has one but Premiere/CapCut basic compressors hide it; it can come in a later wave without breaking this one.
- Cost: the ratio ceiling of 20 means "infinity:1" is unauthorable; the answer is `limiter`, which the review finding 6 points at. Cost: the limiter check cannot verify true-peak, only sample-peak, which is why the master handles delivery.

## The Judge's read

(The assistant's, after the ballots; the owner ruled "go with the Judge".)

- **Agreed.** Q1: the bed target is an authored literal (3/3), out-of-range is an error, the +20 dB lift is a
  review, undefined loudness is a note. Q2: `highpass`/`lowpass`, `frequency_hz` 20..20000, a unit-named slope
  key, a stack cap of 8, a lavfi-tone check. Q3: RMS detection, a threshold in RMS dBFS, a fixed 5 ms
  look-ahead, `ceiling_db` as sample peak, makeup 0..24, a static curve plus a burst, no TTS.
- **Q1 range:** −50..−10, −40..−5 and −40..−6; taken −40..−6 (holds −38 and −14; two of three land near it).
  Juror 1's "gain is only known at render time" is right: the +20 dB review belongs to `render`, not `validate`.
- **Q2 shelf:** Juror 3 rejects #817's `shelf { side }` for `low_shelf`/`high_shelf`; that reopens a ruling, so
  `shelf { side }` stays. `gain_db` ±24 and `q` 0.1..10 are the 2/3 figures.
- **Q2 slope contract:** only Juror 1 saw that cascading identical Q 0.707 biquads reads about −6 dB (24) or
  −12 dB (48) at the cutoff, not −3.01. His Butterworth-section design is the only one that makes #817's
  number true for every slope, so it is taken. (Confirmed on ffmpeg 7.1.5: −6.02 and −12.04.)
- **Q3 defaults:** Juror 1 required every key; Jurors 2 and 3 defaulted attack, release and makeup, with
  figures that disagree (10/100 against 20/250). Leaned to no defaults, which follows ADR-0145 and #816; flagged
  as the owner-reopenable call.
- **Q3 severities:** the limiter ceiling against the master is an error in Fable's `− 1 dB` form; compressor
  after limiter is a review.

## Ruling

The owner said "go with the Judge", which took the Judge's carry list as above. Three things the ballots could
not have known, found by the evidence run while the ADRs were written, were added: the compressor's curve is
exact only at attack = release = 100 ms; `alimiter` rejects a ceiling below −24.08 dB, so the limiter range is
−24..0, not −30..0; and the effect list must run in float or a 48 dB/oct stopband quantises to zero. All three
are in the ADRs. The evidence scripts are `check_eq_biquads.py`, `check_dynamics_curves.py` and
`check_normalize_loudness.py` under `docs/research/audio-effects/`.
