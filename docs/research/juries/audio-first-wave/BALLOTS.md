# Ballots: the audio map's six frontier tickets

Packet: [`QUESTION.md`](QUESTION.md), sent blind and identical to three jurors dispatched in
parallel. Map: [#795](https://github.com/MBehtemam/Montagent/issues/795). Ballots are verbatim,
grouped by juror; the Judge's read is separate, in [`JUDGES-READ.md`](JUDGES-READ.md).

**Only Q-F is live.** Q-A to Q-E were decided by other sessions before this record landed
(ADR-0169, ADR-0170, ADR-0172, ADR-0173), so their ballots are kept for the record.

## Juror 1 (Claude Opus 5.5)

🗳️ **Juror 1** (Claude Opus 5.5) — **Q-A VOTE: (a) Confirm the narration-over-music assumption and proceed. Ask the owner one non-blocking question in the ticket.**

**Reasoning:** The owner's sentence names tools (Premiere, After Effects, CapCut), not a workflow. The pieces the narration-over-music assumption puts first are loudness normalisation, ducking-as-volume-keyframes, voice EQ and compression. The other two workflows need those same pieces. Dialogue clean-up needs EQ, a gate and the master stage. CapCut short-form needs normalisation (CapCut's one confirmed feature is a fixed -23 LUFS normalise). Building those first also settles the structural decisions in Q-B to Q-E: the chain list, the units, the master stage and the measured check. Every later capability depends on those. Starting with creative effects such as voice changers would mean building on the one category with no DSP precedent in either reference editor. The question for the owner: "Is your typical project a voiceover on music, recorded on-camera speech, or short-form clips with sound effects? Which one sounds wrong today?" The answer reorders the second wave. It does not change the first.
**Trade-offs:** If the owner really wants CapCut-style voice changers or pitch effects first, they wait one wave. Option (d) would stall the whole map on a reply. Most of the first wave is the same under every answer, so that wait buys almost nothing. Options (b) and (c) are each narrower than (a) and depend on (a)'s pieces anyway.

🗳️ **Juror 1** (Claude Opus 5.5) — **Q-B VOTE: (c) A flat keyframeable `pan` peer of `volume`, plus an ordered `audio_effects` list for the processing chain.**

**Reasoning:** This mirrors Premiere. Every clip there has fixed Volume and Panner stages after a reorderable effect rack. Order matters for processing (EQ before compressor sounds different from compressor before EQ), so the chain must be a list. Volume and pan are order-free mixing controls, so they stay as fields.
- **Key name:** `audio_effects`. This follows ADR-0040, which says audio effects are their own vocabulary and not a branch of `effects`.
- **Pipeline order:** atrim → atempo → aloop → `audio_effects` in list order → `volume` → `pan` → atrim/asetpts/adelay. Putting `volume` after the chain means fades and ducking keyframes are not undone by a compressor or limiter, and the compressor sees the source level the agent reasons about. Pan can be built as per-channel volume, which accepts runtime commands, so it can be keyframed even though ffmpeg's `pan` filter ignores them.
- **Duplicates:** The same kind may appear twice, for example two EQ stages or a de-esser on each side of a compressor. Premiere's rack allows this.
- **Singling out one entry:** Each entry carries a required `id` string that is unique within the element. An exact-string replace includes `"id": "..."` as its anchor, so an edit is unambiguous even when two entries have identical parameters.
- **Video elements:** A `video` element's embedded audio carries the same `audio_effects` and `pan` fields, just as it already carries `volume`.
- **Channel ops:** Operations such as mono from left or swap are list entries, normally placed first.

**Trade-offs:** Pan as a flat field makes it impossible to pan before an effect, which is rarely wanted and could be added later as a list kind. A required `id` adds one authored string per entry. Without it, two identical entries cannot be told apart for exact-string replace. Option (b) cannot express order or repeats. Option (a) would push volume and pan into a list where their position would be meaningless and could hide a misplaced fade.

🗳️ **Juror 1** (Claude Opus 5.5) — **Q-C VOTE: (a) dB for all new level parameters. `volume` stays the only linear level. Units show as key suffixes.**

**Reasoning:** ADR-0055 rejected dB for `volume` because its identity value would be 0, which means "off" elsewhere in the format. That objection does not apply to the new parameters:
- An EQ gain of `gain_db: 0` means "this band is flat", which is close to "off".
- Thresholds and ceilings are naturally negative dBFS values. A linear threshold of 0.0316 cannot be read by a human or an agent.
- A loudness target is LUFS by definition, and every precedent (Premiere's -23 LUFS Auto-Match and R128 export, CapCut's fixed -23 LUFS) states it in LUFS.

Linear-everywhere (b) would force the log arithmetic onto the reader that ADR-0055 was trying to spare them. A per-parameter choice (c) without one rule would drift over time.

Key suffixes, which suit a closed vocabulary that `validate` checks:
- Levels: `gain_db`, `threshold_db`, `makeup_db`, `ceiling_dbtp`, `target_lufs`.
- Frequency: `_hz`.
- Time: integer `_ms`, matching ADR-0005.
- Ratio: plain `ratio`, a number ≥ 1 meaning n:1.
- Q: plain `q`.
- Pitch: `semitones` (cents allowed as a fractional value).
- Dry/wet: `mix`, 0..1.
- Pan: plain `pan`, -1..+1 with 0 at centre, on the same normalised scale as `volume`.

**Trade-offs:** Two level conventions now coexist: linear `volume` and dB everywhere else. The suffix makes the unit visible at every use, which limits confusion. Pan of -1..+1 differs from Premiere's -100..+100. It was chosen to match the format's normalised-number style.

🗳️ **Juror 1** (Claude Opus 5.5) — **Q-D VOTE: A top-level `master` object holding a loudness target and a true-peak limiter (option (i) two-pass, measured by the renderer with the `ebur128` crate rather than `loudnorm`).**

**Reasoning:**
- **Location and vocabulary:** `master` sits at the top level, as a peer of the element list. Its closed vocabulary is `loudness: {target_lufs, ceiling_dbtp}`, optionally plus a master `gain_db`. Nothing else goes in it for now.
- **Why not (ii):** If the agent writes a literal gain after `verify` measures the mix, that gain is a derived value. The next element edit makes it stale, which contradicts ADR-0145's rule that derived things are changed through the literal that generates them. The target is that literal.
- **How (i) works:**
  - The renderer mixes the pre-master audio and measures its integrated loudness in-process with the `ebur128` crate, which is deterministic Rust and does not vary with the ffmpeg build.
  - It applies one constant gain equal to the target minus the measurement.
  - It then runs `alimiter` with `latency=1` at the ceiling.
  - It never uses dynamic `loudnorm`, which outputs 192 kHz, is not byte-stable across builds and ignores runtime commands.
- **How `verify` reads it:** `verify`'s R128 gate reads `master.loudness.target_lufs` and asserts that integrated loudness is within ±0.5 LU (the EBU R128 programme tolerance) and true peak ≤ `ceiling_dbtp`. Without a `master`, it falls back to today's behaviour.
- **Partial renders and `preview`:** Both run the whole-timeline audio-only measurement pass, which is cheap compared with video, and apply the whole-programme gain. An excerpt is then exactly as loud as it will be in the deliverable. On a partial render, `verify` skips the integrated-loudness assertion, because integrated loudness of an excerpt is meaningless, and checks only true peak.

**Trade-offs:** Every render pays one extra audio-only mix pass. The limiter can change the integrated level slightly on very peaky mixes, so the tolerance must absorb that or the measurement must be iterated once. `validate` can check the target's range from the file but cannot check that the target was reached; `verify` does that.

🗳️ **Juror 1** (Claude Opus 5.5) — **Q-E VOTE: Both — a repo test fixture is mandatory for every capability, plus a `verify` check for capabilities that make a deliverable-level promise. Measurement is in-process Rust on decoded PCM.**

**Reasoning:**
- **Fixtures:** Each capability's ADR names its number. The fixture uses deterministic synthetic inputs (sines, sweeps, pink noise, impulses, a short licensed speech clip), renders through the real pipeline and measures the result. Examples:
  - EQ: band energy changes by `gain_db` at `freq_hz`, ±0.5 dB.
  - Limiter: true peak ≤ ceiling.
  - Compressor: steady-state level change matches the ratio, ±0.5 dB.
  - Crossfade: sample-exact boundaries and a constant-power sum within ±0.5 dB.
  - Latency-cancelled filters: impulse position exact to the sample.
  - Pitch shift: fundamental within ±1 cent.
- **`verify` on the deliverable:** This is reserved for loudness and true peak, the deliverable-level promises.
- **Tooling:** The `ebur128` crate measures LUFS and true peak. A Goertzel or FFT measurement in the test code measures band energy and pitch. No ffmpeg analysis filters are used, so the numbers do not drift with the build.
- **Tolerance convention:** Every ADR states each check as `metric = expected ± tolerance unit`. Defaults are ±0.5 LU or ±0.5 dB for level and exact for time. Audio byte-hashes are only ever pinned on the CI floor build, never compared across builds.
- **A/B evidence:**
  - A repo script renders the prototype project twice through the real pipeline, once with the effect and once with it bypassed.
  - It measures both renders with `ebur128` and applies a constant gain to the bypass render so both match within 0.1 LU.
  - It writes a short report of the numbers.
  - The commit includes the two project JSONs, the report and the two short (≤ 20 s) clips under a prototypes evidence directory. The ADR records the owner's verdict next to the numbers.

**Trade-offs:** Committed audio clips grow the repo. Capping clip length keeps this small; moving clips to issue attachments would make the evidence non-durable. Rubberband capabilities cannot run on CI's macOS ffmpeg, so those fixtures must be skipped there or gated to the floor build. Fixtures prove the mechanism, not the taste, which is why the owner's ear stays beside the numbers.

🗳️ **Juror 1** (Claude Opus 5.5) — **Q-F VOTE: (b) Audio crossfades are their own mechanism, as in Premiere.**

**Reasoning:**
- **Why separate:** Premiere treats audio crossfades as objects separate from video transitions. Under option (a) or (c), a visual transition would silently change audio. That is an implicit, derived behaviour, and it breaks "each authored choice has its own string". Audio-only cuts, such as music bed to music bed, need a crossfade with no picture anyway. With (b), one mechanism covers both audio and video-embedded audio.
- **Shape:** It is a new element kind, `audio_transition`. It names two audio-bearing elements (`audio`, or `video` with embedded audio) and follows the same exact-window rule as visual transitions, checked by `validate`.
- **Curve:** A required, selectable `curve` from a closed vocabulary: `constant_power`, `constant_gain`, `exponential`. These match Premiere's three crossfade types.
- **Rendering:** It is sample-exact `afade` out on one chain and `afade` in on the other, summed by the existing `amix normalize=0`. It avoids `acrossfade`, which ignores runtime commands and is replaceable by this method anyway.
- **Volume keyframes inside the window:** The crossfade gain multiplies with them. It is applied as a separate stage after `volume`, so ducking or a fade inside the window still composes predictably. `validate` may warn when a fade keyframe to 0 overlaps a crossfade.

**Trade-offs:** A typical video cut needs two authored objects, one visual and one audio, where (a) would need one. A write tool that emits both together can absorb this without hiding the choice in the format. Option (c) would carry both mechanisms plus an opt-out flag, which is more vocabulary for little gain. Option (a) cannot express audio-only cuts and leaves no way to cut picture while letting the sound overlap.

## Juror 2 (Sonnet 5.5)

🗳️ **Juror 2** (Sonnet 5.5) — **Q-A VOTE: (d) variant — keep (a) as the working assumption but ask the owner one question**

**Reasoning:** The owner's sentence names Premiere, After Effects and CapCut, which span all three workflows, so nothing in it confirms (a). The research shows Premiere has the most confirmed precedent, and the settled scope (loudness, EQ, compressor, noise reduction, ducking) is mostly narration and dialogue tooling. That scope already builds the shared foundation for (a) and (c). Ask the owner once: "Which finished video do you most want to make sound better first: a narrated video over music, recorded footage with noisy speech, or a short-form clip with fun voices and effects?" Until they answer, proceed with (a), because its capabilities are a subset of what (c) needs.

**Trade-offs:** Confirming (a) outright would record a guess as a decision. Choosing (b) fails the ADR-0145 precedent rule, because voice changers have no DSP precedent in Premiere or CapCut. Choosing (c) alone leaves out ducking, which is already in settled scope. Asking costs one round trip, but work on loudness, EQ and the compressor is not blocked, since those serve every workflow.

🗳️ **Juror 2** (Sonnet 5.5) — **Q-B VOTE: (a) an ordered list of tagged audio effects**

**Reasoning:**
- **Key name:** `audio_effects`. It sits beside `effects` (visual) and ADR-0040 already says audio is its own vocabulary.
- **Where `volume` sits:** `volume` stays a flat top-level field and is applied after the chain, as clip gain at the end. This matches Premiere's separation of clip gain from effects, and it keeps existing projects valid.
- **Duplicate kinds:** the same kind may appear twice, as with an EQ before and after a compressor. The order is the authored choice.
- **Exact-string replace:** each entry carries a required unique `id` string, so one entry is a single line that can be replaced. The reference conventions for flat elements can be reused. Entries are tagged `{"id":"eq1","type":"eq",...}`.
- **Pan:** this is order-sensitive in practice (before or after a reverb), so it goes in the list as a tagged entry. That keeps one rule, with no split between order-free and ordered properties.
- **Embedded `video` audio:** `video` elements carry the same `audio_effects` and `volume`, applied to their embedded audio. A video with no audio stream is a validate error if `audio_effects` is non-empty.

**Trade-offs:** (b) cannot express the order of the chain, such as an EQ before or after a compressor, and it would bloat the closed element schema with flat fields. (c) creates two places to look for a property, and agents would have to learn which one holds which. The cost of (a) is a longer line per entry and the need for `id` uniqueness checks in `validate`.

🗳️ **Juror 2** (Sonnet 5.5) — **Q-C VOTE: (c) with a firm default — dB for every new level parameter except where a linear ratio is the natural unit. This is (a) with exceptions.**

**Reasoning:**
- **Why dB:** ADR-0055's objection was about the identity value for `volume`, where 0 would mean "off". New EQ gain, threshold, makeup, ceiling and loudness-target parameters have no such problem, because 0 dB is a natural neutral for gain and negative numbers are normal for thresholds. Premiere and every audio tool use dB, and a reader can read -18 dB directly.
- **Key naming:** the unit goes in the key name (`gain_db`, `threshold_db`, `ceiling_db`, `target_lufs`). The renderer converts to linear internally.
- **Other units:** frequency in `_hz`, time in `_ms` (integer, in line with ADR-0005), `ratio` as a plain number such as 4 for 4:1, Q unitless, pan as an integer -100 to +100 matching Premiere, pitch in `semitones` and `cents`.
- **Which keys stay unitless:** `volume` remains linear and is the only linear level.
- **Closed unit set:** the list of units is closed, so `validate` can check each key against its range.

**Trade-offs:** (b) would force agents into log arithmetic, which ADR-0055 explicitly wanted to avoid, and a threshold of 0.125 is unreadable. Pure (a) is nearly the same as my vote, but the fixed rule of unit in the key name is what makes it safe. The cost is two level scales in one document, which is bounded because `volume` is the only one that is linear.

🗳️ **Juror 2** (Sonnet 5.5) — **Q-D VOTE: (ii) a literal gain the agent writes after `verify` measures it, with a small closed master stage holding a literal limiter and a literal gain**

**Reasoning:**
- **Where it lives:** a single top-level `master` object, one per project, which the renderer applies after `amix`. Tracks own nothing (ADR-0004), so a project-level key is consistent.
- **Closed vocabulary:** `gain_db` (literal), `limiter` (`ceiling_db`, `release_ms`), and optionally a fixed high-pass and `pan`/`channels` mode. It is not a list, because the stage is small and closed.
- **Loudness:** option (i) breaks ADR-0145. A target is not a literal value, and the two-pass `loudnorm` is not byte-stable across builds, and linear mode needs four measured values. Instead, `verify` measures the integrated LUFS, and the agent writes `master.gain_db` to bring it to the target. Premiere's Auto-Match and ducking likewise write ordinary values. A write tool such as `normalize` can compute and write the literal, so the derived number is changed by changing the literal that generates it. The `limiter` ceiling catches the true peak.
- **`verify` reading:** `verify` measures the rendered deliverable as it does today and reports the gap to the R128 target. It does not read `master` itself, and the check stays "from the file and probed assets alone", because the gain is just a number in the file.
- **Partial render / `preview`:** the master gain applies as written, with no measurement. A partial render is not loudness-gated. The gain is a fixed literal, so a time range sounds the same as in the full render. `verify` on a partial render reports loudness for that range only and labels it as such, never as a pass for the whole.

**Trade-offs:** (ii) means the agent must re-measure after any edit that changes the mix, and the number goes stale. That is acceptable because staleness is visible, as `verify` would report the miss. (i) gives a hands-off result, which is what CapCut offers with a fixed -23 LUFS, but it makes the renderer non-deterministic across builds and breaks partial renders, since a range cannot be measured as a whole mix.

🗳️ **Juror 2** (Sonnet 5.5) — **Q-E VOTE: both — `verify` for the deliverable, and a repo test fixture for the capability**

**Reasoning:**
- **What each does:** `verify` is the production check, using the existing `ebur128` gate on a real project (integrated LUFS, true peak, loudness range). The repo fixture is the regression check: a small committed project and a synthetic input (a sine sweep or pink noise) rendered through the real pipeline, with asserts in a render test. The repo has no audio-content tests today, so this closes a real gap.
- **Tooling:** use `ebur128` for loudness and true peak, the same crate as `verify`, so there is one measurement authority. For spectra and band energy, add a small Rust FFT (`rustfft`) in a dev-only test helper, which keeps the check off ffmpeg and in-repo. Do not use ffmpeg's `astats` or `ebur128` filters as the authority, since they can differ across builds.
- **Tolerance convention:** state each number with its tolerance in the ADR, such as ±0.5 LU or ±0.5 dB at a named frequency, ±1 dB for true peak, and a tighter ±0.1 dB for a deterministic gain. Where ffmpeg is bit-stable across reruns on one build but not across builds, assert on measured numbers with tolerances, not hashes. Pin only the cases the research marks as byte-identical to the CI floor build.
- **A/B evidence:** a script renders the effect and a bypass, then applies a literal gain to the processed render so integrated loudness matches the bypass, within ±0.1 LU. It commits the two short WAV or AAC files, the project JSON, and a small text report with the measured numbers under `docs/adr/evidence/<adr>/`. Keep clips short (under 10 s) to bound repo size. The owner listens to the A/B beside the measured numbers.

**Trade-offs:** `verify` alone leaves effects untested in CI, and the fixture alone does not guard real projects. The cost is two places to maintain, and `rustfft` as a dev-dependency. Tolerances rather than hashes means slight drift can go unnoticed within the band, but that is the honest choice given the cross-build differences.

🗳️ **Juror 2** (Sonnet 5.5) — **Q-F VOTE: (c) with a twist — transitions carry audio by default with an opt-out, and a separate audio-only crossfade handles audio-to-audio cuts**

**Reasoning:**
- **Why default-on:** today a visual crossfade or wipe between two clips cuts the sound hard, which is what a user would call a bug, and CapCut applies its transition sound with the picture. A user who adds a transition expects the audio to follow its exact window, and the window is already authored on the `transition` element, so no new time literal is needed.
- **Mechanism:** the renderer applies `afade` out on the outgoing element chain and `afade` in on the incoming one over that window, summed in `amix`. This is the replacement for `acrossfade` that the research recommends, and `afade` is sample-exact. It works with `normalize=0`.
- **Opt-out:** a literal field on the `transition` element, `audio: "none"` (closed vocabulary `"crossfade"`, the default, and `"none"`), so each authored choice has its own string.
- **Separate audio-only crossfade:** needed for audio-to-audio cuts such as music tracks, where no visual transition exists, as Premiere has separate objects. It is an `audio_transition` element with an exact window, the same shape as the visual one.
- **Curves:** a closed `curve` enum with `constant_power` as the default (Premiere's default, and the right choice for uncorrelated material such as narration to music) and `constant_gain`. Skip Exponential Fade at first. `afade` supports the corresponding curves natively.
- **Interaction with `volume` keyframes:** the fade multiplies with the keyframed volume, since the fade is applied as an `afade` in the chain and the keyframes as timed commands. That makes it deterministic: effective gain = keyframed volume × fade. Keyframes inside the window still apply, and `validate` warns when a keyframe inside the window changes the level by a large amount, because it is probably a duplicate fade.

**Trade-offs:** (a) alone leaves audio-to-audio cuts with no way to crossfade, and (b) alone makes every transition silent unless the agent authors a second element, which is easy to forget, and the visual cut and audio cut then drift. The cost of (c) is two mechanisms and a default that changes the audio of existing projects that already use transitions, which means a migration note or an ADR-recorded behaviour change. The one-frame lateness of keyframed volume is a separate known issue and does not affect `afade`.

## Juror 3 (Fable 5.1)

🗳️ **Juror 3** (Fable 5.1) — **Q-A VOTE: (a) confirm narration-over-music as the first workflow — but sequence the first ADR so the answer does not matter, and ask the owner one bounded question in parallel.**

**Reasoning:** The owner's sentence is a feature-parity complaint ("like Premiere / CapCut, we have less"), not a workflow statement, so no option can be *derived* from it. But the settled scope already encodes a workflow: ducking-as-write-tool, loudness normalisation, voice EQ + compressor are the narration-over-music kit, and Montagent is agent-driven — the videos agents actually produce today are narrated explainers over a bed. (b) short-form creative effects (voice changers, reverb, echo) have no DSP precedent in either reference app and would each need an owner-accepted prototype before any ADR can land; starting there blocks the map on the slowest path. (c) dialogue clean-up (afftdn/anlmdn/de-ess) is a strict subset of (a) once the chain and master stage exist. Crucially, the first two ADRs — the element audio chain shape (Q-B) and the master stage with loudness (Q-D) — are workflow-agnostic; every one of (a)/(b)/(c) needs them. So confirm (a), start those two, and ask the owner exactly one thing: "Paste two project JSONs (or describe two videos) you want produced in the next month, and name the platform they ship to." The platform answers the loudness target; the projects answer the workflow. If the answer contradicts (a), only the ordering of the per-effect ADRs changes, not the infrastructure ADRs.

**Trade-offs:** Voting (d) alone would stall the map on an owner who has shown they communicate in one sentence; voting (a) without the question risks building a dialogue toolkit for someone who wanted chipmunk voices. The hybrid costs nothing but the one question. Cost of (a) itself: compressor/EQ/normalisation ADRs are the heaviest on measured acceptance (Q-E), so the early ADRs are slow — but that tooling is reused by every later capability.

🗳️ **Juror 3** (Fable 5.1) — **Q-B VOTE: (c) — flat fields for order-free properties, plus one ordered tagged list `audio_effects` for the processing chain.**

**Reasoning:** Pan, volume and the speed/pitch choice are *positions* and *faders*, not processors: they commute with each other, every agent and every reference app treats them as always-present clip properties, and `volume` is already a flat peer of `speed`. Forcing them into a list would make "where is the pan?" a search, and would let an agent author two pans. The processing chain (EQ, compressor, gate, NR, de-ess, reverb, echo, pitch shift) is genuinely order-dependent — compressor-before-EQ and EQ-before-compressor are different sounds, Premiere's effect rack is ordered, and the visual `effects` list already establishes the tagged-ordered-list idiom in this format. Specifics:
- **Key name:** `audio_effects`. Not `effects` (ADR-0040 forbids branching the visual chain); not `audio` (collides with the element kind name); the `audio_` prefix lets a `video` element carry both `effects` and `audio_effects` unambiguously.
- **Volume's place:** post-chain, as a fader, then pan (same as Premiere's fixed effects, which render after standard clip effects). ffmpeg order per element: atrim → atempo/pitch → aloop → [audio_effects in list order] → volume → pan → atrim/asetpts/adelay. This is what makes a compressor's threshold meaningful (it sees source level, not the fade) and keeps today's fades/ducking keyframes working untouched.
- **Same kind twice:** allowed. Two EQs or an EQ-compressor-EQ sandwich is ordinary practice; forbidding it buys nothing because each entry is its own literal.
- **Singling out an entry:** by its own text. Each entry is a self-contained object with its own parameter strings; an agent replaces the entry's full object text, or a parameter inside it with enough surrounding context to be unique — exactly as it must already do for two identical `{t, v, ease}` keyframes in different elements. No `id` field: it would be a new concept with no reader value, and a duplicate-ambiguity error from the replace tool is the correct behaviour, not a defect.
- **`video` element's embedded audio:** carries the identical fields — `volume`, `pan`, `audio_effects`, and the speed-pitch field — with identical semantics. Today `volume` is already on both; splitting the vocabularies would double every ADR.
- **Pitch under speed:** a flat field beside `speed`, closed vocabulary `speed_pitch: "preserve" | "follow"`. Note the default must stay today's behaviour, which is *preserve* (the pipeline already uses `atempo`); "follow" (chipmunk) is the new option, rendered by asetrate+aresample. A boolean named "maintain pitch" would read as if preserve were new.

**Trade-offs:** Three homes for audio state (flat fields, chain, master) is more surface than one list. Pan as a flat field can never be placed mid-chain (a "stereo reverb after pan" sound) — accepted; neither CapCut nor Premiere's clip panner offers that either. The frame-late keyframed volume defect (ADR-0077) is inherited by every keyframed chain parameter until the asendcmd granularity is fixed; the chain ADR should say so.

🗳️ **Juror 3** (Fable 5.1) — **Q-C VOTE: (a) dB for all new level parameters, unit in every key name; `volume` stays the single linear multiplier.**

**Reasoning:** ADR-0055's two objections were specific to a *multiplier*: "0 means off everywhere else" and "a reader can't read a level without log arithmetic." Neither applies to the new parameters. For an EQ band or makeup gain, 0 dB *is* the natural identity ("no change"), and `gain_db: 3` is read directly as "+3 dB" by every human and every manual; `gain: 1.41` requires the log arithmetic the ADR wanted to avoid. For thresholds, ceilings and loudness targets there is no readable linear form at all — `threshold: 0.1` versus `threshold_db: -20`; `target_lufs: -16` has no non-logarithmic spelling. Premiere, CapCut's loudness page, every platform spec and every ffmpeg filter speak dB here, so precedent, ffmpeg mapping and readability all agree. (b) would make Montagent the only tool whose compressor threshold is a fraction. (c) invites the same debate per ADR; decide once.
- **Key naming:** unit suffix on every quantity-bearing key: `gain_db`, `threshold_db`, `makeup_db`, `ceiling_dbtp` (true peak, distinct from sample peak), `target_lufs`, `freq_hz`, `attack_ms`, `release_ms`, `delay_ms`, `semitones`/`cents` for pitch. `volume` and `pan` keep bare names because they are dimensionless and already/by-convention unitless.
- **Frequency:** integer Hz (`freq_hz: 3200`), 20..20000 — no kHz floats.
- **Time:** integer ms, per ADR-0005 (`attack_ms`, `release_ms`, `delay_ms`, `decay_ms`). Minimum 1 ms; a limiter wanting 0.1 ms attack is covered by `latency` lookahead, not fractional ms.
- **Ratio:** bare `ratio` as a number meaning N:1 (`ratio: 4`); limiters have no ratio, they have `ceiling_dbtp`.
- **Q:** bare `q`, dimensionless (Premiere/biquad convention); alternative `width_oct` rejected to keep one vocabulary.
- **Pan:** `pan` -1.0..1.0, 0 centre, left negative. Identity 0 like every other new parameter; the float style matches `volume` and keyframe `v`; Premiere's -100..+100 maps by ×100.

**Trade-offs:** Two level conventions coexist in one element (`volume: 0.5` next to `makeup_db: 6`); the unit suffix is what makes that survivable, and the ADR must state plainly "volume is the only linear level, by ADR-0055". Integer ms rules out sub-millisecond attacks; accepted for consistency with ADR-0005. `ceiling_dbtp` requires true-peak measurement (4× oversampled), which the `ebur128` crate already provides.

🗳️ **Juror 3** (Fable 5.1) — **Q-D VOTE: (iii) — a top-level `master` object; loudness is a *declared target that `verify` asserts*, while the gain that reaches it is an ordinary literal the agent writes (optionally via a write tool), never a two-pass measurement inside the renderer.**

**Reasoning:** (i) breaks three of the four ADR-0145 invariants at once: the gain is derived from the whole mix (not a literal), `validate` cannot check it from the file, and it is not byte-stable (`loudnorm` dynamic drifts across builds and resamples to 192 kHz; linear mode needs four measured_* values — i.e. the two-pass result *is* a hidden literal). It also makes partial renders lie: the same project renders a different gain for 0–10 s than for the whole. (ii) is correct in mechanism but, standing alone, leaves the format unable to say what the author *wanted*, so `verify` has nothing to gate against and the next agent cannot tell a deliberate -14 from an accident. The hybrid keeps both: the document carries the intent as a checkable assertion and the mechanism as a literal.
- **Where:** `master` at the project top level, peer of the elements list — it is a property of the one output, not of any element or track (tracks own nothing, ADR-0004). All fields optional; an absent `master` renders byte-identically to today.
- **Closed vocabulary:** `master: { volume, audio_effects: [eq | compressor | limiter], loudness: { target_lufs, ceiling_dbtp } }`. Three chain kinds only — the "small" stage. No pan, no NR, no reverb at master; those are element-level. `master.volume` is the literal gain the agent writes (linear, same type as element `volume`, keyframeable so a global dip is possible). `limiter` with `ceiling_dbtp` is deterministic (`alimiter`, `latency=1`, apad/atrim cancel) and is the only thing the renderer *does* with the true-peak number.
- **Loudness semantics:** `loudness.target_lufs` and `loudness.ceiling_dbtp` are *assertions*. The renderer ignores `target_lufs` entirely. `verify` measures integrated loudness (R128 gated, `ebur128`) and true peak on the full deliverable and fails if |measured − target| > 1 LU or TP > ceiling. A write tool `normalise` (like ducking) renders, measures, and writes `master.volume` as the literal that closes the gap — same pattern, same invariants.
- **Partial render / preview:** apply `master.volume` and the master chain literally, identically to a full render (they are literals, so a 10 s preview sounds like that 10 s of the final). `verify` on a partial render reports measured LUFS/TP but marks the loudness gate "not evaluated: partial range" — a whole-mix integrated target cannot be judged on a slice, and pretending otherwise would produce false failures.

**Trade-offs:** The agent must do a render-measure-write loop (or call the tool) rather than declare a target and be done — that is the price of literal values, and it is exactly how Premiere's Auto-Match and ducking already behave (they write gains). Nothing stops an agent writing `target_lufs: -16` and `volume: 1` and shipping without `verify`; the standing rule that `verify` gates the deliverable is what catches it. A keyframed `master.volume` reintroduces the frame-late asendcmd issue at master; acceptable, documented.

🗳️ **Juror 3** (Fable 5.1) — **Q-E VOTE: Both — `verify` checks whole-mix numbers on the deliverable; a repo test fixture checks each capability's own number through the real pipeline against the CI-floor ffmpeg. Every ADR names one number, one signal, one tolerance.**

**Reasoning:** `verify` can only see the sum: integrated LUFS, LRA, true peak, and (new) L/R balance. It cannot tell whether the EQ boosted 3 dB at 3.2 kHz or the compressor's attack was 10 ms. Those are per-capability claims and belong in fixtures that run on every PR, or ADRs become promises nobody re-checks — the repo currently has none, which is the gap the ticket names. Conversely, a fixture on a 10 s sine says nothing about the owner's deliverable, so `verify` stays the gate on the real thing.
- **Fixture method:** synthetic inputs generated in the test (sine at known Hz/dBFS, two-tone, step burst, pink noise from a seeded PRNG; no committed audio assets), written to WAV, rendered through the *real* render path to a project JSON that exercises exactly one capability vs a bypass twin, decoded back with `ffmpeg -f f32le pipe`, measured in Rust.
- **Tooling:** `ebur128` (already a dependency) for LUFS/LRA/true peak; a single-bin Goertzel (≈20 lines, no new crate) for band gain at a known frequency; RMS over windows for compressor gain-reduction and attack/release timing; cross-correlation lag against the bypass render for the "zero added latency" claim every fixed-latency filter must prove; L/R RMS ratio for pan. `rustfft` only if a capability genuinely needs a spectrum (reverb/NR), as a dev-dependency.
- **Tolerance convention:** dB-valued parameters ±0.5 dB; loudness ±0.5 LU in fixtures, ±1 LU in `verify`; true peak ≤ ceiling, no tolerance; timing claims (latency cancellation, afade/crossfade placement) exact to 0 samples; byte-hash assertions only for filters the research found byte-stable and only pinned to the CI floor build string, which the fixture asserts first and skips otherwise.
- **Prototype A/B evidence:** a script under the prototype directory renders `effect.json` and `bypass.json` through the pipeline, measures both with `ebur128`, writes the literal `master.volume` that matches the quieter to within 0.1 LU, re-renders, and emits `measurements.json` (ffmpeg version string, LUFS/TP of both, the capability's own number, pass/fail against the ADR's tolerance). Committed: the two project JSONs, the script, `measurements.json`. The two short loudness-matched renders (≤15 s AAC) are attached to the PR/issue for the owner's ear, not committed — the numbers are reproducible from what is committed, the audio is not reviewable in a diff anyway.

**Trade-offs:** CI gets slower and ffmpeg-dependent: every fixture is a real render. Rust-side measurement means reimplementing small bits of DSP (Goertzel, RMS) that could be done with `astats`/`showspectrum`, but ffmpeg-side measurement would be measuring the tool with itself and parsing text. Build-pinned hashes rot when CI's ffmpeg moves; that is the intended alarm. The owner's ear remains the acceptance for things with no precedent (voice changers, reverb character), the number only proves the mechanism did what the ADR says.

🗳️ **Juror 3** (Fable 5.1) — **Q-F VOTE: (c) — but implemented as one field and one kind, not two mechanisms: visual transitions gain a closed `audio` field whose value is the curve (or `cut`), and the existing `transition` element gains an `audio_crossfade` kind for audio↔audio cuts.**

**Reasoning:** Today a visual crossfade between two `video` elements with embedded audio produces a hard audio cut in the middle of a picture dissolve — that is a defect every viewer hears, and a format that offers transitions with "exact windows between two elements" already holds the literal that defines the audio window. So (b) alone (Premiere's separation) leaves the common case broken and makes the agent author a second object whose window must be kept identical by hand — a derived value spelled twice, against ADR-0145. (a) alone cannot express an audio-only crossfade between two music beds, and cannot opt out when the cut is intentional (a hard music cut on a wipe). Hence (c), with the smallest possible surface:
- **On visual transitions:** `audio: "cut" | "constant_power" | "constant_gain" | "exponential"`. Default `cut` — today's behaviour, so existing documents render byte-identically; agent guidance and write tools default new transitions to `constant_power`. One key carries both the opt-out and the curve choice.
- **Audio-only:** the `transition` element takes kind `audio_crossfade` with the same two-element reference and exact window, plus `curve` from the same three-value vocabulary. Reusing the element avoids a second "between two elements over a window" concept.
- **Curves:** exactly Premiere's three (constant power default — the right choice under `amix normalize=0` summation of uncorrelated material; constant gain for correlated/contiguous material; exponential for the tail-dip). Closed, with precedent, nothing invented.
- **Rendering:** `afade` out on A and `afade` in on B with `curve=` mapped (`qsin`/`tri`/`exp`), inserted *before* the element's `audio_effects` → volume stage; the two chains sum in the existing `amix`. Sample-exact, unlike keyframed volume (which can land a frame late) — the reason this is not "just emit volume keyframes": a 40 ms crossfade cannot tolerate a 21 ms slip.
- **Volume keyframes inside the window:** they multiply. The crossfade is a derived gain generated by the transition literal; the element's own `volume` keyframes (ducking, a deliberate dip) continue to apply on top. Two fades stacking is the author's doing and is visible in the file. `validate` warns, not errors, when a volume keyframe falls inside a transition window on that element.

**Trade-offs:** The audio of a visual transition is now authored on the transition, so an agent changing only audio must find the transition element — acceptable, since the window lives there anyway. Default `cut` means today's silent defect persists for documents that don't opt in; the alternative (default crossfade) silently changes every existing render and was rejected for that reason. `acrossfade` is deliberately not used (no runtime commands, lengthens/consumes streams) — afade pairs through `amix` were verified equivalent. Stacking afade before volume means the crossfade gain is not keyframe-editable; by ADR-0145 it is changed through the literal that generates it (window, curve).
