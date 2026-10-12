# Premiere Pro audio, set against Montagent v0.2.0

Research for [#889](https://github.com/MBehtemam/Montagent/issues/889), re-checked for
[#911](https://github.com/MBehtemam/Montagent/issues/911), part of the map
[#882](https://github.com/MBehtemam/Montagent/issues/882) ("every Premiere Pro feature, set against
Montagent v0.2.0"). Area: **audio**.

- **Date:** 2026-10-10. Nothing was built or run. **This note decides nothing.** It lists features and
  states what v0.2.0 ships against each one.
- **Premiere side:** Adobe's own Premiere user guide (helpx.adobe.com/premiere/desktop, Premiere 26.5), read
  in full from the census fetch ([#883](https://github.com/MBehtemam/Montagent/issues/883)): the 72 pages of
  the census area `audio` (listed in **Pages covered** below), the three What's-new pages (What's new, release
  notes, beta features), and 26 pages from other chapters that hold an audio mechanism (preferences, speed,
  sample rates, recording, VR audio, export settings, plug-ins, Audition round trip, Generative Extend,
  keyframe interpolation, shortcuts). One page outside the Premiere guide, Media Encoder's loudness
  correction, was re-checked live.
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, which is `origin/main`). Read: `CONTEXT.md`,
  `schema/montagent.schema.json`, ADR-0001, 0004, 0011, 0020, 0055, 0077, 0145, 0157, 0166, 0169–0185, the
  audio map [#795](https://github.com/MBehtemam/Montagent/issues/795) (its Out of scope), the MCP tool surface
  (`crates/montagent/src/mcp.rs`: `validate`, `create_project`, `query`, `frame`, `render`, `verify`,
  `preview`, `measure`, `shift`, `compare`), the render graph (`crates/montagent-core/src/verbs/render.rs`,
  `render/master.rs`, `crates/montagent-render/src/encode.rs`), `checks/source.rs`, and `skills/`.
- **Precedent reused, not redone:** [`docs/research/audio-effects/PRECEDENT.md`](../audio-effects/PRECEDENT.md)
  (§1–§14) settled loudness normalisation, pan, channel operations, crossfades, EQ, dynamics, noise reduction,
  de-ess, reverb, echo, pitch shift, voice changers, pitch preservation and auto-ducking. Rows that lean on it
  say **PRECEDENT §n**.

## How this was read

The first pass (#889) ran while helpx.adobe.com was blocked and rested on search extracts, with **(R)** marking
recalled names. This pass (#911) re-sourced every row against the fetched pages:

- **(F)** after a link: the mechanism in the row is stated on that fetched page. Every row now carries one.
- **(F, live)**: the page sits outside the Premiere guide and was fetched live from helpx.adobe.com on
  2026-10-10.
- **unconfirmed**: no fetched page supports the feature. The row is kept, as #911 asks, and its status still
  says what v0.2.0 would ship against it. There are **9** such rows.
- **‡** after a feature name: the fetched page contradicted or corrected the first pass's mechanism, and the
  row was rewritten. **(added)**: the feature was missing from the first pass.
- Montagent evidence was read from the repo at the tag, not recalled. A status was re-checked only where the
  corrected mechanism changed it.

## What "shipped" means at v0.2.0

The audio map's ADRs are in three states, and the status column follows the **schema**, not the ADR:

- **Built** (in the schema and the render graph): `volume` (ADR-0055, 0175), `audio_effects` with seven
  members, `normalize_loudness`, `highpass`, `lowpass`, `shelf`, `bell`, `compressor`, `limiter`
  (ADR-0169, 0178–0180), each with an `enabled` flag, the `master` stage (ADR-0172, 0174), the transition
  `audio` field and the `audio_crossfade` kind (ADR-0176), and `duck.py` (ADR-0177). ADR-0176–0180 are still
  `proposed` until their three-leg measured tables are committed (`CONTEXT.md`, **Audio effect**), but they
  are built.
- **Proposed, not built:** `reduce_noise` (ADR-0181), `noise_gate` (ADR-0183), `pan` and `channels`
  (ADR-0184), `echo` (ADR-0185). None is in `schema/montagent.schema.json` at v0.2.0. Rows that depend on
  them are **missing** and name the ADR.
- **Fixed by the renderer:** every source is resampled to a **48 kHz stereo** bus (ADR-0077;
  `aformat=sample_rates=48000:channel_layouts=stereo` in `render.rs`), and the deliverable's audio is
  **AAC 160k** in an MP4 (`encode.rs`). ADR-0172 records that output sample rate, codec and channel layout
  "stay hard-coded".

Two refusals recur below:

- **Track-level audio** (track effects, track volume/pan/mute automation, submixes and buses) is ruled out by
  the audio map's court ([#795](https://github.com/MBehtemam/Montagent/issues/795) Out of scope, unanimous)
  and recorded in ADR-0172 ("Effects attach to elements and to this stage, never to tracks (ADR-0004)") and
  `CONTEXT.md` **Master stage** ("not a track and not a bus"). A track owns nothing (ADR-0004), so an
  element's sound may not depend on its lane.
- **Model- or detector-driven audio** (AI enhance voice, de-reverb, speaker separation, generated sound) and
  **convolution from an impulse-response file** fail ADR-0145's literal-values invariant, as #795's Out of
  scope records. This refuses them *as format members*; a skill that runs a detector and writes literals (as
  `beat_grid.py` and `duck.py` do) is a separate door.

## Counts

| Status | Rows (#889) | Rows (#911) |
|---|---|---|
| have | 23 | 28 |
| have (skill) | 2 | 2 |
| partial | 34 | 39 |
| missing | 67 | 87 |
| refused | 22 | 28 |
| **Total** | **148** | **184** |

36 rows added, 33 rows with a corrected mechanism (‡), 3 statuses changed (Surround Reverb missing → refused;
5.1 Mixdown Type partial → missing; Properties panel audio controls have → partial). No row was dropped. Every
(R) row was re-sourced; 9 rows are unconfirmed.

## The table

Abbreviations in the evidence column: **S** = `schema/montagent.schema.json`, **C** = `CONTEXT.md`.

### A. Audio effects: Amplitude and Compression

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Amplify | Clip/track effect: per-channel Gain sliders that boost or attenuate each channel, and Link Sliders. | [lib][lib] (F) | partial | `volume` (ADR-0055): one linear multiplier, `>1` amplifies, after `audio_effects` (ADR-0169). | A dB gain, per-channel gain, and a position among the effects: `volume` is linear, whole-element, and runs after `audio_effects`. |
| Channel Mixer | Effect: one tab per output channel; input sliders set the percentage of each channel mixed into it; per-channel Invert. | [lib][lib] (F) | missing | None. ADR-0184's `channels` (proposed, not in S) is five fixed words, not a matrix. | Any channel-mix matrix; per-channel polarity. |
| Channel Volume | Effect: each channel's level in dB, on a stereo or 5.1 clip or track. | [lib][lib] (F) | missing | ADR-0184 §8 defers it: "a later separate member" (per-channel `_db`, ADR-0170). | The member. |
| DeEsser | Effect: Broadband or Multiband mode, Threshold, Center Frequency, Bandwidth, Output Sibilance Only, a Gain Reduction meter; mono, stereo or 5.1. | [lib][lib] (F) | missing | No member in S. `compressor` (ADR-0180) is full-band, with no side-chain filter. | A band-limited (side-chain filtered) compressor or a `de_ess` member. |
| Dynamics | Four sections, each switchable: Auto Gate, Compressor (ratio, attack, release, gain), Expander (ratio), Limiter, with LED and gain-reduction meters. | [lib][lib] (F) | partial | `compressor` and `limiter` members (ADR-0180, S `audio_effects`). | The gate (`noise_gate`, ADR-0183, proposed, not in S) and the downward expander (no ADR). |
| Dynamics Processing | Effect: an editable input/output graph (add/delete point, invert, spline curves, make-up gain); look-ahead, noise gating below 50:1, input gain, attack/release, Peak or RMS detection, link channels, band limiting with low/high cutoff. | [lib][lib] (F) | partial | `compressor` (ADR-0180): one knee-less threshold/ratio, RMS detection only. | A drawn multi-point transfer curve, peak detection (ADR-0180 refuses `detection: peak` for its member), look-ahead, band limiting. |
| Hard Limiter | Effect: Maximum Amplitude, Input Boost, Look Ahead Time, Release Time (default about 100 ms), Link Channels. | [lib][lib] (F) | partial | `limiter` member: `ceiling_db`, `release_ms` (ADR-0180). `master.ceiling_dbtp` limits the mix at 4× rate (ADR-0174). | Input boost (place a `volume` or `makeup_db` before it instead), a look-ahead parameter, channel-link choice. |
| Multiband Compressor | Effect: four bands set by Low/Mid/High crossovers, each with Solo, Bypass, Threshold (−60..0 dB), Gain, Ratio (1..30), Attack (0..500 ms), Release (0..5000 ms); Output Gain; a final Limiter with Margin; Brickwall Limiter; Link Band Controls. | [lib][lib] (F) | missing | No member. | The member, and band splitting. |
| Single-band Compressor | Effect: Threshold, Ratio 1..30, Attack (default 10 ms), Release (default 100 ms), Output Gain −30..+30 dB. | [lib][lib] (F) | have | `compressor`: `threshold_db`, `ratio`, `attack_ms`, `release_ms`, `makeup_db`, all required (ADR-0180, S). | (Departure: no defaults, RMS threshold; ADR-0180.) |
| Tube-modeled Compressor | Effect: Threshold, Output Gain, Ratio, Attack, Release, with subtle distortion that "simulates the warmth of vintage hardware". | [lib][lib] (F) | partial | `compressor` (ADR-0180). | The tube/saturation colouring. |

### B. Audio effects: Delay and Echo

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Delay ‡ | Effect: a single echo; 35 ms or more gives discrete echoes, 15–34 ms a chorus or flange. The page lists no parameters (PRECEDENT §10 records delay, feedback, mix). | [lib][lib] (F) | missing | `echo` member (`delay_ms`, `feedback`, `mix`) is ADR-0185, **proposed, not in S**. | The built member. |
| Analog Delay | Effect: Mode (Tape, Tube, Analog), Dry Out, Wet Out, Delay, Feedback (up to 200 %), Trash (distortion and low boost), Spread (stereo width). | [lib][lib] (F) | missing | ADR-0185 (proposed) covers a clean echo only. | Built echo, plus saturation and spread. |
| Multitap Delay | Effect: up to four echoes of the original; mono, stereo or 5.1. | [lib][lib] (F) | missing | ADR-0185 (proposed) is one tap; two stacked `echo` members would feed each other in series, not in parallel. | Parallel taps. |

### C. Audio effects: Filter and EQ

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Bandpass ‡ | Effect: removes frequencies outside the specified band; mono, stereo or 5.1. (The first pass's centre/Q parameters are not on the page.) | [lib][lib] (F) | partial | A `highpass` plus a `lowpass` member in series (ADR-0179). | A single band member; slopes are only 12/24/48 dB/oct. |
| Bass | Effect: Boost in dB for 200 Hz and below. | [lib][lib] (F) | have | `shelf` with `side: "low"`, `frequency_hz`, `gain_db` (ADR-0179). | — |
| FFT Filter | Effect: a drawn curve or notches; log or linear scale, spline curves, FFT Size and Window. | [lib][lib] (F) | missing | Up to 8 EQ members (`E-EQ-STACK-CAP`, ADR-0179) can approximate a curve. | An arbitrary drawn response. |
| Graphic Equalizer (10, 20, 30 bands) | Effect: fixed bands at 1, ½ or ⅓ octave; per-band gain, Range 1.5..120 dB, Accuracy, Master Gain. | [lib][lib] (F) | partial | `bell` members (ADR-0179). | 10/20/30 bands: `E-EQ-STACK-CAP` refuses more than 8 enabled EQ members on one element. |
| Highpass | Effect: removes frequencies below Cutoff. | [lib][lib] (F) | have | `highpass`: `frequency_hz` 20..20000, `slope_db_per_oct` 12/24/48 (ADR-0179). | — |
| Lowpass | Effect: removes frequencies above Cutoff. | [lib][lib] (F) | have | `lowpass` (ADR-0179). | — |
| Notch Filter | Effect: up to six notches, each with frequency, gain and enable; Notch Width Narrow (2nd order) to Super Narrow (6th order); Ultra Quiet; Fix Gain. | [lib][lib] (F) | partial | `bell` with negative `gain_db` (ADR-0179). | Very narrow notches: `q` is capped at 10 and `gain_db` at −24; no notch member. |
| Notch Filter presets (added) | Named presets on the Notch Filter: Sibilance Softener, and DTMF presets for telephone tones. | [lib][lib] (F) | missing | No preset vocabulary; the agent writes the `bell` values itself. No ADR refuses effect presets as such (ADR-0178 §2 refuses only Essential Sound's per-type presets). | Named presets. |
| Parametric Equalizer ‡ | Effect: five bands plus high-pass, low-pass and two shelves; Frequency, Gain, Q/Width with Q up to about 100; Constant Q or constant width in Hz; shelf slope 6 or 12 dB/oct; Range 30 or 96 dB; Ultra-Quiet. | [lib][lib] (F) | partial | `highpass`, `lowpass`, `shelf`, `bell` (ADR-0179). | Nine sections against a cap of 8; Q only 0.1..10 (the page's narrow notches need about 100); no constant-width mode. |
| Scientific Filter | Effect: Bessel, Butterworth, Chebychev or Elliptical; LowPass, HighPass, BandPass or BandStop; Order, Transition Bandwidth, Stop Attn, ripple, Master Gain. | [lib][lib] (F) | partial | The EQ members are Butterworth biquads (ADR-0179). | The other three families, arbitrary order, and band-stop as one filter. |
| Simple Notch Filter | Effect: one notch frequency. | unconfirmed: not in the fetched library or any census page ([lib][lib] lists no such effect) | partial | `bell` with negative gain (ADR-0179). | Narrow notches beyond `q` 10. |
| Simple Parametric EQ | Effect: one bell band. | unconfirmed: not in the fetched library or any census page | have | `bell` (ADR-0179). | — |
| Treble | Effect: Boost in dB for 4000 Hz and above. | [lib][lib] (F) | have | `shelf` with `side: "high"` (ADR-0179). | — |

### D. Audio effects: Modulation

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Chorus/Flanger | Effect: Chorus or Flanger mode; Speed, Width (maximum delay), Intensity (dry/wet), Transience. | [lib][lib] (F) | missing | No member; no ADR. | The member. |
| Flanger | Effect: Initial and Final Delay Time, Stereo Phasing (degrees), Feedback, Modulation Rate (Hz or beats), Inverted / Special Effects / Sinusoidal modes, Mix. | [lib][lib] (F) | missing | No member; no ADR. | The member. |
| Phaser | Effect: Stages, Intensity, Depth, Mod Rate, Phase Diff, Upper Freq, Feedback, Mix, Output Gain. | [lib][lib] (F) | missing | No member; no ADR. | The member. |

### E. Audio effects: Noise Reduction / Restoration

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Adaptive Noise Reduction ‡ | An obsolete audio effect: still opens in old projects and has a Clip FX Editor UI (the troubleshooting page names it); not in the current library. | [obsolete-ts][obsolete-ts] (F) | refused | Adaptive (detector-driven) processing: ADR-0145 literal values, per #795 Out of scope. | — |
| Automatic Click Remover | Effect: Threshold 1..100 (default 30) and Complexity 1..100 (default 16) for clicks, pops and vinyl crackle. | [lib][lib] (F) | missing | No member; no ADR. | The member. |
| DeHummer | Effect: root Frequency, Q, Gain, Number of Harmonics, Harmonic Slope, Output Hum Only. | [lib][lib] (F) | partial | Stacked negative `bell` members at the fundamental and harmonics (ADR-0179). | A harmonic-series member; narrow enough notches (`q` ≤ 10); the 8-member cap limits harmonics. |
| DeNoise | Effect: one amount slider 0..100 % for hum, hiss, fans and background noise (PRECEDENT §7). | [lib][lib] (F) | missing | `reduce_noise` (`reduction_db`, over `anlmdn`) is ADR-0181, **proposed, not in S**; its dial's transfer to real hiss is open (ADR-0181 §4). | The built member. |
| DeReverb | Effect: "estimates the reverberation profile"; amount 0..100 %. | [lib][lib] (F) | refused | Model/detector-driven: #795 Out of scope, ADR-0145 literal values. | — |

### F. Audio effects: Reverb

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Convolution Reverb | Effect: Impulse (a WAV or AIFF file), Mix, Room Size, Damping LF/HF, Pre-Delay, Width, Gain. | [lib][lib] (F) | refused | #795 Out of scope: "an external reference, not a literal parameter set" (ADR-0145 literal values). | — |
| Studio Reverb | Effect: algorithmic (not convolution); Room Size, Decay, Early Reflections, Width, High/Low Frequency Cut, Damping, Diffusion, Dry, Wet. | [lib][lib] (F) | missing | ADR-0185 owner call 10: reverb is "out of this ADR … to get its own decision or none". | The member and its ADR. |
| Surround Reverb ‡ | Effect for 5.1: Input Center and LFE; a six-channel **impulse file** (WAV or AIFF), Room Size, Damping, Pre-Delay, Front and Surround Width; C Wet Level, L/R and F/B balance, Mix, Gain. | [lib][lib] (F) | refused (was missing) | Impulse-file convolution: ADR-0145 literal values, as Convolution Reverb (#795 Out of scope). The 5.1 bus is also missing (ADR-0077). | — |

### G. Audio effects: Special, Stereo Imagery, Time and Pitch, fixed effects, plug-ins

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Balance (effect) | A standalone stereo balance effect (PRECEDENT §2). The fetched library has none; balance is a Panner property (see Panner below). | unconfirmed: no such effect on [lib][lib] | missing | `pan` (−1..+1 linear balance) is ADR-0184, **proposed, not in S**. | The built field. |
| Binauralizer – Ambisonics | Effect on the Mix track: renders ambisonic audio for headphones, with Pan, Tilt and Roll (−180..+180°) to set the listening direction. | [ambi][ambi] (F) | missing | Stereo bus only (ADR-0077). | Ambisonic channel formats and the decode. |
| Distortion | Effect: separate Positive and Negative input/output curves, Curve Smoothing, Time Smoothing, dB Range, Linear Scale. | [lib][lib] (F) | missing | No member; no ADR. | The member. |
| Fill Left with Right ‡ | Effect: one channel copied over the other. The page's own text says it "duplicates the left channel … and places it in the right channel", the reverse of its name (PRECEDENT §3 left the direction to a test). | [lib][lib] (F) | missing | ADR-0184's `channels: "left"/"right"` (proposed, not in S). | The built field. |
| Fill Right with Left ‡ | As above; the page says it duplicates the right channel into the left; stereo clips only. | [lib][lib] (F) | missing | ADR-0184 (proposed, not in S). | The built field. |
| Guitar Suite | Effect: Compressor, Filter (resonator to talk box; Lowpass/Highpass/Bandpass, Frequency, Resonance), Distort (type menu), Amplifier (amp and speaker models), Mix. | [lib][lib] (F) | missing | `compressor` and EQ exist; no distortion, resonance or amp modelling. | Distortion and amp/cabinet modelling. |
| Invert | Effect: inverts the phase of all channels; mono, stereo or 5.1. | [lib][lib] (F) | missing | ADR-0184 §8: "No Invert … a later `audio_effects` member if wanted". | The member. |
| Loudness Meter | Track/Mix effect (Special > Loudness Meter): ITU-based meter with EBU and ATSC presets and Spotify, Apple Podcasts, Netflix and YouTube presets; readings Input (inter-sample peaks), Momentary (400 ms), Short-Term (3 s), Integrated, Loudness Range; does not change audio. | [lmeter][lmeter], [lm-apply][lm-apply], [lm-measure][lm-measure] (F) | partial | `verify` measures the deliverable's integrated LUFS and true peak and judges them against `master` (ADR-0172, 0173; `verify.rs`). `render` reports the measured mix loudness when `target_lufs` is set. | Momentary, short-term and range readings over time, per-track metering, platform presets (a target is a literal number). |
| Loudness Meter settings (added) | Target Loudness, Low Level Below, Maximum True Peak, Scale (EBU +9 or +18), Unit (LKFS, LUFS, LU), presets; a channel map. | [lm-settings][lm-settings] (F) | partial | `master.target_lufs` and `master.ceiling_dbtp` are the literal target and ceiling `verify` judges against (ADR-0172, 0173). | A noise-floor threshold, display scale and units, named presets. |
| Loudness Radar ‡ | Effect: measures the level of clips, tracks or sequences. (The first pass's "radar display over time" is not on the page.) | [lib][lib] (F) | partial | `verify` (as above). | A per-clip or per-track loudness reading. |
| Mastering | Effect: Equalizer (low shelf, peaking, high shelf; Hz, dB, Q), Reverb amount, Exciter (Retro, Tape, Tube; amount), Widener, Loudness Maximizer 0..100 %, Output Gain. | [lib][lib] (F) | partial | On an element: EQ members, `compressor`, `limiter`. On the mix: `master` loudness + true-peak ceiling (ADR-0172). | Exciter, widener, reverb; and any effect on the mix itself, which ADR-0172 refuses (see section K). |
| Panner – Ambisonics | Effect on an ambisonic audio clip: Pan, Tilt and Roll to reorient the audio sphere, set to the VR Projection effect's values. | [ambi-restage][ambi-restage] (F) | missing | Stereo bus only. | Ambisonics. |
| Pitch Shifter ‡ | Effect: Semi-tones, Cents (±100), Ratio (0.5..2.0), Precision (Low/Medium/High), Splicing Frequency, Overlapping (0..50 %), default-settings switch. (The first pass's "formant preserve" is not on the page.) | [lib][lib] (F) | missing | ADR-0170 names the unit (semitones, fractional) but no member exists. | The member and its ADR. |
| Stereo Expander ‡ | Stereo Imagery effect: Center Channel Pan (−100 %..+100 %) and Stereo Expand 0 (normal) to 300 (wide). | [lib][lib] (F) | missing | No member; ADR-0184 says pan is balance only, a stereo-image pan is "a later" call. | The member. |
| Swap Channels | Effect: exchanges left and right; stereo clips only. | [lib][lib] (F) | missing | ADR-0184 `channels: "swap"` (proposed, not in S). | The built field. |
| Vocal Enhancer | Effect: Male and Female modes reduce sibilance, plosives and handling rumble and add microphone modelling and compression; Music mode compresses and EQs music under a voice. | [lib][lib] (F) | missing | `highpass` covers rumble only. | The member (and de-ess, plosive handling, mic modelling). |
| Voice-changer presets | Named Pitch Shifter / Essential Sound presets (PRECEDENT §12). | unconfirmed: no voice-changer preset on [lib][lib] or any census page | missing | No pitch member; no preset vocabulary. | Pitch Shifter first. |
| Volume (fixed effect) | Every clip's built-in Volume under Audio Effects in Effect Controls; Level keyframable; clip and channel volume drive the dynamic waveform. | [ecp][ecp], [trackvol][trackvol], [dynwave][dynwave] (F) | have | `volume` scalar or `{t,v,ease}` keyframes (ADR-0055, 0175). | (Departure: linear, not dB; ADR-0055, 0170.) |
| Panner (fixed effect) | Clip Pan slider for mono clips in the Properties panel, keyframable; the Audio Clip Mixer pan knob writes clip keyframes; Panner > Balance or Pan on tracks (PRECEDENT §2). | [props][props], [acm-auto][acm-auto], [pan-track][pan-track] (F) | missing | `pan` is ADR-0184, proposed, not in S, and static (keyframes deferred under #842). | The built field and keyframes. |
| Effect bypass (added) | Each audio effect has a bypass (the fx toggle), which keyframes can switch; track effects have an Effects Bypass icon. | [fx-paste][fx-paste], [atm-fx][atm-fx] (F) | have | Every `audio_effects` member takes `enabled` (S); a disabled member is skipped by the render graph (ADR-0179 §2). | Keyframed bypass (members are static, #842). |
| Effect channel layout (Channelize effects) (added) | Effect Controls > Channel Map > Re-map > Request Layout: a custom channel layout per effect, with channel labels added or removed, saved as a preset, so an effect runs on a subset of channels. | [channelize][channelize] (F) | missing | Every member processes the element's stereo signal; no per-channel routing. | Channel maps on effects. |
| Obsolete audio effects ‡ | A project holding an obsolete audio effect prompts on open to replace it with the new version. (The first pass's list of legacy names is not on any fetched page.) | [lib][lib] (F) | missing | The format has no "obsolete" tier: a member is in the closed union or is an unknown key (ADR-0169). | A replace-on-open path (not a format feature). |
| Third-party audio plug-ins (VST, VST3, AU) | VST, VST3 and Audio Units (macOS) effects work as audio effects; Preferences > Audio > Manage Audio Plug-ins opens the Audio Plug-in Manager. | [plugins][plugins], [prefs][prefs] (F) | refused | ADR-0145 closed-vocabulary and checkable-by-`validate` invariants; `audio_effects` is a closed union keyed on `name` (ADR-0169). | — |

### H. Audio transitions and fades

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Constant Power crossfade | Audio transition: like a video dissolve; the first clip falls slowly then fast, the second rises fast then slowly. | [xfade][xfade] (F) | have | Transition `audio: "constant_power"`, or `kind: "audio_crossfade"` (ADR-0176, S). | (Mechanism differs: the two elements sit on two tracks and overlap; Premiere's sits on one track.) |
| Constant Gain crossfade | Audio transition at a constant rate; can dip at the midpoint. | [xfade][xfade] (F) | have | `audio: "constant_gain"` (ADR-0176). | — |
| Exponential Fade | Audio transition over smooth logarithmic curves, more gradual than Constant Power, with an Alignment control. | [xfade][xfade] (F) | refused | ADR-0176 §1, Rejected: "A third curve, `exponential` … no rule an agent can pick by"; the door is kept open. | — |
| Audio follows a video transition | Shift+D, "Apply Default Transitions to Selection", applies the default video and audio transitions together. | [shortcuts][shortcuts] (F) | have | One `audio` field on `crossfade`, `wipe`, `slide`, `push` carries the sound across the same window (ADR-0176). | — |
| Fade in / fade out a clip | Drag the fade handles at a clip's edge; or a transition at one end; or the volume keyframe graph for a custom rate. | [fade][fade], [trans-default][trans-default] (F) | have | Two `volume` keyframes (ADR-0055), heard on the sample their instant names (ADR-0175). | — |
| Fade handles | Draggable handles on audio clip edges make fades and, dragged across a neighbour, crossfades; Shift changes one side, Option changes curve or duration on one axis; 26.5 adds show/hide transition handles. | [fade][fade], [xfade-handles][xfade-handles], [relnotes][relnotes] (F) | missing | No GUI; the outcome is written as keyframes or a transition. | The interactive editor (Montagent's tools read, check and render; ADR-0011). |
| Default audio transition and its duration | Right-click Constant Gain or Constant Power > Set Selected As Default Transition; Preferences > Timeline > Audio Transition Default Duration; Ctrl/Cmd+Shift+D applies it. | [trans-default][trans-default], [tl-prefs][tl-prefs], [shortcuts][shortcuts] (F) | missing | Every transition is written with literal `start`/`end`/`audio`; the format holds no editor defaults. | An apply-default command (an editor convenience, not a format feature). |
| Edit an audio transition (added) | Double-click the transition and adjust it in Effect Controls (duration, alignment). | [trans-default][trans-default] (F) | have | A transition's window and curve are its literal `start`, `end` and `audio` (ADR-0176, S). | — |
| Multi Transitions (26.0) | Apply, edit, trim and delete transitions and fades across many audio and video clips at once, including linked and grouped clips; Option/Alt adjusts one side of a linked pair. | [multitrans][multitrans], [whatsnew][whatsnew] (F) | missing | The agent writes each `transition` element; no batch write tool (ADR-0011's write-tool invariant). | A batch command or skill script. |
| J-cut / L-cut | Disable Linked Selection, then trim audio and video independently so one leads or trails the other. | [jl][jl] (F) | have | Separate elements with their own `start`/`end`; ADR-0176 §2: a J/L cut whose window is not the overlap "stays hand-written as `volume` keyframes". | — |

### I. Essential Sound panel

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio types: Dialogue, Music, SFX, Ambience | Each clip is assigned one type, which opens that type's parameter groups. | [esp][esp], [concepts][concepts] (F) | refused | ADR-0178 §2 rejects "a `role` or `"bed"` token … a second vocabulary and a hidden default". | — |
| Audio Category Tagging (Auto Tag) | AI recognises Dialogue, Music, SFX or Ambience on selected clips and adds an interactive badge that opens the matching controls. | [tag][tag] (F) | refused | No type vocabulary (ADR-0178 §2); a model-driven classifier fails ADR-0145 literal values (#795 Out of scope). | — |
| Essential Sound presets | Preset dropdown; customise clarity, repair, loudness or reverb and save as a preset. | [presets][presets], [esp][esp] (F) | refused | ADR-0178 §2: "per-type presets, which #816 already refused"; every member parameter is required, nothing defaults (ADR-0179, 0180). | — |
| Loudness > Auto-Match ‡ | With the clip tagged, Auto-Match brings clips to a consistent loudness; the clip's LUFS shows under the button. (The first pass's per-type target is not on the page.) | [automatch][automatch] (F) | have | `normalize_loudness` with an authored `target_lufs` −40..−6, one measured gain, singular (ADR-0178, S). | (The target is a literal per element.) |
| Repair > Reduce Noise | Checkbox and 0–10 slider for background noise and clicks (PRECEDENT §7). | [repair][repair] (F) | missing | ADR-0181 (`reduce_noise`) proposed, not in S. | The built member. |
| Repair > Reduce Rumble | Checkbox and 0–10 slider for rumble below 80 Hz. | [repair][repair] (F) | have | `highpass` (ADR-0179). | (A literal cutoff and slope, not a 0–10 slider.) |
| Repair > DeHum | Checkbox and 0–10 slider for 50 Hz or 60 Hz hum. | [repair][repair] (F) | partial | Stacked negative `bell` members (ADR-0179). | As DeHummer, section E. |
| Repair > DeEss | Checkbox and 0–10 slider reducing sibilance. | [repair][repair] (F) | missing | As DeEsser, section A. | A de-ess member. |
| Repair > Reduce Reverb | Checkbox and 0–10 slider reducing room reverb. | [repair][repair] (F) | refused | As DeReverb, section E (#795, ADR-0145). | — |
| Clarity > Dynamics | 0–10 slider from Natural to Focused: compression and levelling. | [clarity][clarity] (F) | partial | `compressor` (ADR-0180). | A one-dial macro; every compressor key must be written. |
| Clarity > EQ | Preset dropdown and Amount slider. | [clarity][clarity] (F) | partial | EQ members (ADR-0179). | Named presets and an amount dial. |
| Clarity > Enhance Speech | AI analysis in the background, then a Mix Amount slider between enhanced and original; mono and stereo only (a stereo clip comes back as a mono downmix), no multichannel or nested sequences. | [enh][enh], [enh-req][enh-req] (F) | refused | "AI enhance voice" is #795 Out of scope (ADR-0145 literal values). | — |
| Clarity > Vocal Enhancer ‡ | Checkbox and slider with High Tone or Low Tone. (The first pass said Male/Female; that is the library effect, not this control.) | [clarity][clarity] (F) | missing | As Vocal Enhancer, section G. | The member. |
| Creative > Reverb ‡ | Reverb toggle, a Preset dropdown, and Amount and Clip Volume sliders. (The page does not name Studio Reverb.) | [reverb][reverb] (F) | missing | No reverb member (ADR-0185 leaves it undecided). | The member. |
| Music/SFX > Stereo Width | Width slider. | unconfirmed: no census page names this control ([esp][esp] lists no Stereo Width) | missing | No stereo-width member. | The member. |
| SFX > Pan | Pan slider for a sound effect. | unconfirmed: no census page names this control | missing | ADR-0184 `pan`, proposed, not in S. | The built field. |
| Music > Duration (Remix) | Remix tool (in the Ripple Edit tool group) or Essential Sound > Duration: AI beat analysis cuts and crossfades the middle of a song to a Target Duration (within 5 s, typically 1 s), keeping intro and outro; Segments and Variations (melodic or harmonic); zigzag marks on the clip at each edit. | [remix][remix], [remix-about][remix-about], [remix-tips][remix-tips] (F) | missing | Montagent has the outputs (elements, `overrun: "loop"`, `audio_crossfade`), but nothing computes them. No ADR refuses a skill that would; the analysis itself could not be a format member (ADR-0145). | A skill script that analyses the track and writes cuts and crossfades, as `beat_grid.py` writes a placement. |
| Ducking (Auto Duck) ‡ | On Music and Ambience clips: Duck Against (Dialogue, Music, SFX, Ambience, Un-tagged), Sensitivity, Duck Amount, Fade Duration, Fade Position (outside, inside, middle of the dialogue); Generate Keyframes adds an **Amplify** effect and keyframes its gain, overwriting hand edits on regeneration. | [duck][duck] (F) | have (skill) | `skills/montagent-footage/scripts/duck.py` writes ordinary `volume` keyframes from voice word timings or explicit spans: `under_db`, `over_db`, `end_db`, `ramp_ms`, `lead_ms`, `join_ms`, `fade_ms` (ADR-0177; C **Duck**). | Signal-based Sensitivity: ADR-0177 §2 rejects silence detection, so the voice's timings must be supplied. |
| Clip Volume (Level, Mute) | The panel's Clip Volume slider and Mute. | [reverb][reverb], [dynwave][dynwave] (F) | have | `volume`; `volume: 0` is mute, with no `mute` field (ADR-0055). | (No mute that keeps the level: ADR-0055 refuses a second field.) |

### J. Clip level, gain, normalise, keyframes, speed

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio Gain: Set Gain to / Adjust Gain by ‡ | Right-click > Audio Gain on one or more clips: set the gain to a dB value, or adjust it by ± dB. (The first pass's "pre-effect, combined with track level" is not on the page.) | [gain][gain] (F) | partial | `volume` (linear, after effects, ADR-0169); `compressor.makeup_db` or a `normalize_loudness` gain sit before effects that follow them. | A clip gain in dB separate from `volume`. |
| Peak Amplitude readout (added) | The Audio Gain dialog computes and shows the selected clips' peak amplitude, stored for the selection. | [gain][gain] (F) | missing | `verify` reads the deliverable's true peak, not a source's; `probe` reads rate and channels only (`media/probe.rs`). | A per-source or per-element peak reading. |
| Normalize Max Peak to ‡ | Adjusts the maximum peak amplitude in the selected clips to a dB value at or below 0. | [gain][gain] (F) | missing | No peak normalisation; `normalize_loudness` is loudness, not peak (ADR-0178). | A peak target. |
| Normalize All Peaks to ‡ | Adjusts the peak amplitude in the selected clips to a dB value at or below 0. | [gain][gain] (F) | missing | As above. | A peak target. |
| Normalize Mix Track | Sequence > Normalize Mix Track: the mix fader to a peak target. | unconfirmed: no census page has it | partial | `master.target_lufs` brings the whole mix to a loudness by one measured gain (ADR-0172). | A peak (not loudness) target on the mix. |
| Clip volume keyframes | Show Keyframes > Clip Keyframes; Ctrl/Cmd-click on the clip's volume line during playback. | [trackvol][trackvol], [concepts][concepts] (F) | have | `volume` `{t,v,ease}` records (ADR-0055), heard on their sample (ADR-0175). | — |
| Clip effect-parameter keyframes | Clip Keyframes animate a clip's audio effects, not only Volume. | [trackvol][trackvol], [ecp][ecp] (F) | missing | No `audio_effects` parameter is animatable: first-wave parameters are static, deferred under #842 (ADR-0178 §1, 0179, 0180, 0183 §5, 0185 §6). | Keyframable effect parameters. |
| Clip pan keyframes | Pan keyframes from the Properties panel or the Audio Clip Mixer (PRECEDENT §2). | [props][props], [acm-auto][acm-auto] (F) | missing | `pan` not built; ADR-0184 §5 makes it static and defers keyframes under #842. | Built and animatable `pan`. |
| Track keyframes (Volume, Mute, Panner, track-effect parameters) | Show Track Keyframes; the menu lists Track: Volume, Panner, and pre-/post-fader track effects by position; Pen tool edits them. | [trackvol][trackvol], [trackfx-tl][trackfx-tl] (F) | refused | Track-level audio: #795 court, ADR-0172, ADR-0004 (tracks own nothing). | — |
| Keyframe interpolation on audio | Linear, Bezier, Auto Bezier, Continuous Bezier and Hold on any keyframe. | [kfinterp][kfinterp] (F) | have | `ease` on every non-first record: a name or four bezier control points, `step` for hold (ADR-0012, 0038, S `Ease`). | (No automatic smoothing through a keyframe, as Auto/Continuous Bezier do.) |
| Automation keyframe thinning | Preferences > Audio > Automation Keyframe Optimization: Linear keyframe thinning, Minimum time interval thinning (1..2000 ms), for keyframes written by mixer automation. | [kfopt][kfopt], [prefs][prefs] (F) | missing | No recorded automation exists to thin; keyframes are written literally. | (Only meaningful with live automation recording.) |
| Large Volume Adjustment ‡ | Audio preference: the dB step of the Increase Clip Volume Many command. | [prefs][prefs] (F) | missing | No GUI or editing commands (ADR-0011). | The interactive editor. |
| Speed with Maintain Audio Pitch | Clip > Speed/Duration: "Maintain audio pitch" checkbox. | [speed][speed] (F) | partial | `speed` runs through a chained `atempo` (`render.rs`, ADR-0020), which always preserves pitch. | The "off" state: a speed change that also shifts pitch. |
| Reverse speed with audio ‡ | Speed/Duration > Reverse speed plays the clip backwards. (The page does not say what the audio does.) | [speed][speed] (F) | refused | Reverse is only a `source_time` remap, and a remapped `video` must carry `volume: 0` (`E-REMAP-AUDIBLE`, ADR-0157). | — |
| Time remapping and audio ‡ | Time Remapping is a built-in **video** effect; Volume is the clip's audio effect. | [ecp][ecp] (F) | have | Same outcome: `source_time` requires a silenced `volume` (ADR-0157). | — |
| Mute (clip) | Properties panel Mute silences the clip without changing its volume. | [props][props] (F) | have | `volume: 0`; ADR-0055 adds no `mute` field. | (No mute that keeps the level; ADR-0055.) |

### K. Mixers, tracks, submixes, buses

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio Track Mixer: track fader | One channel strip per timeline audio track; volume fader moves affect the whole track; tracks renamed in place. | [atm][atm] (F) | refused | Track-level audio: #795 court; ADR-0172; C **Master stage** ("not a track and not a bus"). | — |
| Audio Track Mixer: track pan/balance | Pan knob −100..+100 on a mono or stereo track feeding a stereo output (pan for mono, balance for stereo). | [pan][pan], [panctl][panctl] (F) | refused | Track-level audio: #795 court; ADR-0172; ADR-0004. | — |
| 5.1 Panner tray (added) | A mono or stereo track feeding a 5.1 output gets a square tray with a puck, centre-channel percentage and LFE level. | [panctl][panctl] (F) | missing | Stereo bus only (ADR-0077); and the control is per track (refused above). | A 5.1 bus. |
| Audio Track Mixer: effect slots (inserts) ‡ | A set of slots per track for effects or sends; an fx toggle mutes an effect; None removes it. (The first pass's "up to five" is not on the page.) | [atm][atm], [atm-fx][atm-fx] (F) | refused | "Track-level audio effects" are #795 Out of scope; ADR-0172. | — |
| Pre-fader / post-fader effect placement ‡ | Right-click a track effect > Pre-Fader or Post-Fader; effects are **pre-fader by default**. | [prepost][prepost] (F) | partial | On an element the order is fixed: `audio_effects` always before `volume` (ADR-0169), which is Premiere's default. | A post-fader member position (ADR-0169 chose post-fader `volume` deliberately). |
| Copy and move track effects (added) | Drag to move or Ctrl/Cmd-drag to copy an effect within or between tracks; Copy/Paste Track Effects copies a whole rack. | [atm-copy][atm-copy] (F) | refused | Track-level audio effects (#795; ADR-0172). | — |
| Copy and paste track-effect keyframes (added) | Copy track keyframes and paste them at the playhead on the same or another track. | [fx-paste][fx-paste] (F) | refused | Track keyframes are track-level audio (#795; ADR-0172). | — |
| Audio Track Mixer: automation modes | Off, Read, Write, Latch, Touch per track; Safe During Write protects a property; Switch To Touch After Write; moving controls during playback writes track keyframes. | [atm-auto][atm-auto], [atm][atm], [rec-mix][rec-mix] (F) | refused | Track keyframes are track-level audio (#795; ADR-0172). | — |
| Automatch Time (added) | Audio preference: how fast a control released in Touch or Read mode returns to its prior value. | [automatch-time][automatch-time], [prefs][prefs] (F) | refused | It times track automation, which is track-level audio (#795; ADR-0172). | — |
| Audio Clip Mixer | Per-clip volume faders and pan knobs for the clips under the playhead, with meters. | [concepts][concepts], [atm][atm] (F) | partial | Per-element `volume` (ADR-0055). | Per-element `pan` (ADR-0184 not built); the live mixing surface. |
| Audio Clip Mixer: automation (Latch, Touch) | Keyframe Mode Latch or Touch from the panel menu; moving a fader or pan knob during playback writes clip keyframes. | [acm-auto][acm-auto] (F) | missing | The output, clip `volume` keyframes, is writable by hand or script; nothing records it live. | Live recording of keyframes (no playback surface). |
| Mute / Solo track buttons ‡ | Track Solo (and Mute) buttons in the mixer and timeline; keyboard commands toggle mute and solo on targeted tracks. | [rec-atm][rec-atm], [relnotes][relnotes] (F) | refused | A track carries no audio state (ADR-0004; #795; ADR-0172) and the format has no `mute` field (ADR-0055). | — |
| Global Mute (26.2) (added) | A menu-bar Audio button mutes all playback, or only the Source or Program Monitor; not saved with the project; does not change clip or track settings. | [gmute][gmute], [whatsnew][whatsnew] (F) | missing | No playback surface: `render` and `preview` write files (ADR-0011). | A monitoring control (not a format feature). |
| Submix tracks | Mono, stereo or 5.1 tracks that sum tracks routed to them and take effects and a fader; no clips or recording; added with Sequence > Add Tracks. | [submix][submix], [atm][atm] (F) | refused | ADR-0172: the master is "not a general bus with an effect chain"; C **Master stage**: "not a track and not a bus"; track-level audio refused (#795). | — |
| Sends (pre/post fader) to submixes | Track slots hold send assignments; a send is pre- or post-fader; sends cannot go to the Master in a multichannel sequence. | [atm][atm], [prepost][prepost], [trackout][trackout] (F) | refused | As submixes: ADR-0172 (master "not a general bus"); #795 track-level audio. | — |
| Mix (master) track effects | Effects in the Mix track's slots, e.g. Loudness Meter in the first slot, Binauralizer for ambisonic monitoring. | [lm-apply][lm-apply], [ambi][ambi] (F) | refused | ADR-0172 Rejected: "An `audio_effects` list on the master … the stage must be a closed set". | — |
| Mix (master) track volume and loudness | The Mix track's fader and meters in the Audio Track Mixer. | [atm][atm] (F) | partial | `master.target_lufs` and `master.ceiling_dbtp` (ADR-0172, 0174). | A static or keyframed mix gain that is not a loudness target. |
| Pan a nested sequence as one element (added) | A sequence used as a track in another sequence can be panned or balanced as a single audio element. | [panctl][panctl] (F) | missing | `pan` is not built (ADR-0184); nesting is accepted in ADR-0166 but holds no audio control. | Pan, and pan on a nested unit. |
| Automatic downmix to fewer output channels (added) | When the output track has fewer channels than the source track, Premiere downmixes. | [panctl][panctl] (F) | have | Every source is folded to stereo at input (`aformat=…channel_layouts=stereo`, `render.rs`; ADR-0077). | (The downmix law is ffmpeg's default, not stated.) |
| Audio track types: Standard, Mono, Adaptive, 5.1 | A track's channel format: Standard (mono and stereo), Mono, Adaptive (varying formats), 5.1. | [concepts][concepts] (F) | missing | Tracks carry no channel format (ADR-0004); every source becomes stereo on the bus (ADR-0077). | Channel formats (the per-track part would also meet the track-level refusal). |
| Direct output / track output channel assignment | With an adaptive or multichannel master: Direct Output Assignment opens a matrix routing each track channel to master channels (default 1, 1-2 or 1-6). | [trackout][trackout] (F) | missing | Stereo bus only. | Multichannel master and routing. |
| Multiple stereo assignments to a multichannel master | Direct Output Assignment > Assign Stereo Pairs: one track to several pairs (1+2, 3+4) for separate delivery mixes. | [multistereo][multistereo] (F) | missing | Stereo bus only. | Multichannel master. |
| Record audio mix changes | Pick a mode other than Off or Read, play (or loop, or In to Out), move controls, stop; track properties only, not clip properties. | [rec-mix][rec-mix] (F) | refused | Track automation is track-level audio: #795; ADR-0172; ADR-0004. | — |

### L. Channels, sample rate, sync, merge, link

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Modify > Audio Channels (source channel mapping) | Clip > Modify > Audio Channels: a matrix of clip channels against source channels; enable or disable source channels; preview a source channel; Clip Channel Format (PRECEDENT §3). | [chmap][chmap], [chmap-concept][chmap-concept] (F) | missing | ADR-0184 `channels` (proposed, not in S: `stereo`, `mono`, `left`, `right`, `swap`) picks among two channels only. | Built field; mapping of more than two source channels. |
| Use a mono clip as stereo (added) | Modify > Audio Channels > Clip Channel Format: Stereo puts a mono clip on both left and right. | [mono-stereo][mono-stereo] (F) | have | A mono source is upmixed to both channels of the stereo bus at input (`aformat=…channel_layouts=stereo`, `render.rs`; ADR-0077). | — |
| Default Audio Tracks on import (added) | Preferences > Timeline > Default Audio Tracks: how a clip's channels land on tracks (Use File, Mono, Stereo, 5.1, Multichannel Mono). | [map-import][map-import], [tl-prefs][tl-prefs] (F) | missing | No track channel formats (ADR-0004, 0077). | Channel formats. |
| Hide audio channel labels (added) | Timeline wrench menu > Show Audio Channel Labels toggles the labels on clips (26.2). | [chmap][chmap], [relnotes][relnotes] (F) | missing | No GUI (ADR-0011). | The interactive editor. |
| Multichannel master and 5.1 mixing ‡ | Adaptive or multichannel master tracks with a channel-assignment matrix; 5.1 panning on the tray; downmix when the output has fewer channels. (The first pass's "up to 32 channels" is not on the page.) | [trackout][trackout], [panctl][panctl] (F) | missing | Bus is 48 kHz stereo (ADR-0077); ADR-0172: "output channel layout … belongs to the channel-operations capability". | Any non-stereo bus or output. |
| 5.1 Mixdown Type preference ‡ | Audio preference: "how Premiere mixes source channels to 5.1 audio tracks". (The first pass read it as a 5.1-to-stereo fold; the page says it is a mix **into** 5.1.) | [prefs][prefs] (F) | missing (was partial) | No 5.1 track or bus exists to mix into (ADR-0077). | A 5.1 bus. |
| Breakout to Mono ‡ | Clip > Audio Options > Breakout to Mono on a Project-panel item: new mono source clips per channel (stereo → 2, 5.1 → 6), named "Left"/"Right" etc.; no new files. | [breakout][breakout] (F) | missing | ADR-0184 `channels: "left"`/`"right"` (proposed) would give the two stereo channels on two elements; nothing reaches 5.1 channels. | The built field; per-channel access beyond stereo. |
| Extract Audio (added) | Clip > Audio Options > Extract Audio: a new audio clip from a video clip's audio, carrying its channel mapping, gain, speed and interpretation; named "Audio Extracted". | [extract][extract] (F) | have | An `audio` element whose `source` is a video file plays that file's audio stream, measured against the audio stream (`checks/source.rs`). | (No new file is written; none is needed.) |
| Sequence sample rate | Sequence Settings > Audio > Sample Rate; resampling costs time and quality; display in audio samples or milliseconds. | [seqset][seqset] (F) | partial | Fixed 48 kHz bus (ADR-0077). | A chosen rate (44.1, 96 kHz …); sample-unit time display. |
| Natively supported sample rates (added) | 8, 11.025, 22.05, 32, 44.1, 48 and 96 kHz play without conforming in a matching sequence. | [sr][sr], [digaudio][digaudio] (F) | have | Any rate ffmpeg decodes is resampled to the 48 kHz bus at render (`aformat=sample_rates=48000`); `probe` reads `sample_rate` (`media/probe.rs`). | (Everything is resampled; no rate is "native".) |
| 32-bit float processing (added) | All audio is processed as 32-bit floating point at the sequence rate. | [sr][sr], [conform][conform] (F) | partial | An `audio_effects` chain runs in float (`aformat=sample_fmts=fltp` ahead of the first enabled member, `render.rs`, ADR-0179 §2). | A stated sample format for the rest of the graph and a chosen rate. |
| Conform of unsupported sample rates (CFA files) | Uncompressed audio at a non-native rate is upsampled on import; compressed audio conforms at its source rate; CFA files cached in Audio Previews and reused across sequences. | [conform][conform], [sr][sr], [digaudio][digaudio] (F) | have | Every source is resampled to 48 kHz in the mix graph at render; `probe` reads each source's `sample_rate` and `channels` (`media/probe.rs`). | (No cache file; done per render.) |
| Match Source audio (26.5) | Match Source sets the sequence's audio channels and sample rate from the source, with automatic downmixing and a warning when a format cannot hold the layout. | [whatsnew][whatsnew], [relnotes][relnotes] (F) | missing | The bus does not follow the source (fixed 48 kHz stereo). | A source-matched bus. |
| Synchronize clips (added) | Clip > Synchronize: align by Clip Start, Clip End, Timecode, numbered Clip Marker, choosing an audio track channel as reference. | [sync-tl][sync-tl] (F) | partial | Aligning starts or ends is arithmetic on literal `start`/`source_start` values the agent writes. | Source timecode and clip markers (see the editing area); a sync command. |
| Synchronize by audio waveform | Audio-based sync of separately recorded clips (an audio track channel as reference); not available inside merged clips. | [sync-tl][sync-tl], [merge-limits][merge-limits] (F) | missing | No sync tool; detector-driven, so it could only enter as a skill that writes `start` values. | A skill script. |
| Merge Clips | One video (or none) with up to 16 audio channels (mono 1, stereo 2, 5.1 6) as one merged clip; sync by In, Out, Timecode or Clip Marker; from the Project panel or the Timeline (Timeline merges sync from Clip Start); limits: no Replace Footage, no full channel mapping, no FCP XML, no auto-sync, no re-merge. | [merge][merge], [merge-tl][merge-tl], [sync][sync], [merge-limits][merge-limits] (F) | partial | An `audio` element placed at the video's `start` with its own `source_start` gives the same playback (ADR-0001: flat elements). | A linked unit that moves as one (no linking concept, by ADR-0001). |
| Trim merged clips (added) | A trim applies to all component clips, keeping offsets; Alt/Option breaks sync to trim one; gaps play black or silence. | [merge-edit][merge-edit] (F) | missing | Each element is trimmed by its own literals; no unit ties them (ADR-0001). | A linked trim. |
| Merged clip metadata (added) | Metadata panel per component clip or All Files, written into each component's XMP. | [merge-meta][merge-meta] (F) | missing | No clip metadata is read or written. | Metadata. |
| Audio timecode from a primary clip; Remove Audio from AV Clip (added) | Merge Clips > Use Audio Timecode From Clip syncs on the recorder's timecode; Remove Audio from AV Clip drops the camera audio. | [merge-tc][merge-tc] (F) | partial | Dropping camera audio is `volume: 0` on the `video` (ADR-0055). | Source timecode. |
| Link / unlink audio and video ‡ | Clip > Link joins one video to several audio clips (same track format, each on its own track) so they move, trim and take effects together; Clip > Unlink. | [link][link] (F) | partial | A `video`'s own sound moves with it (ADR-0055); a separate `audio` element is independent. | Linking of two elements (a `group` shares no timing; ADR-0001). |
| Edit a multi-clip link in the Source Monitor (added) | Double-click a linked clip, pick a track to view, set In/Out; the same trim applies to the other linked tracks. | [mclink][mclink] (F) | missing | No linking (ADR-0001) and no monitor (ADR-0011). | Linking. |
| Separate Crosstalk (added) | Clip > Separate Crosstalk: a cloud AI model splits 1 s–10 min of overlapping speech into one track per speaker plus a background track and disables the original; costs one Generative Credit per second; needs consent to voice embeddings. | [crosstalk][crosstalk], [crosstalk-ai][crosstalk-ai] (F) | refused | Model-driven audio: ADR-0145 literal values (#795 Out of scope). | — |
| Bulk mute or bleep (added) | Transcribe the sequence, keep a Censored word list (import/export), filter Censored words in the Text panel, and apply a mute or a chosen bleep tone to one or all instances. | [bleep][bleep], [whatsnew][whatsnew] (F) | partial | Muting a span is ordinary `volume` keyframes, by hand or from supplied spans (`duck.py --spans`, ADR-0177). | Word detection (transcription is model-driven; see the captions area), a censored-word list, and any tone source for a bleep. |

### M. Waveforms, scrubbing, monitoring, hardware

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Timeline audio waveforms (peak files) | Waveforms drawn from PEK files in the media cache; Timeline Display Settings > Show Audio Waveform; "Generate waveforms automatically during import". | [conform][conform], [src-mon][src-mon], [prefs][prefs] (F) | missing | `frame` contact sheets draw picture only; ADR-0031: the overview "is not required to be spatial". | A waveform reading (image or numbers). |
| Dynamic Audio Waveforms (26.0) | View > Dynamic Audio Waveforms (on by default): waveforms reflect clip and channel volume, Amplify, Essential Sound volume, mute and ducking, Properties panel and Audio Gain. | [dynwave][dynwave], [whatsnew][whatsnew] (F) | missing | As above. | A post-volume waveform reading. |
| Live waveform editing (26.0) (added) | Waveforms update while moving, copying, trimming, rolling, rate-stretching, rippling, slipping and sliding clips; keyframes and markers show while dragging. | [livewave][livewave], [whatsnew][whatsnew] (F) | missing | No GUI (ADR-0011); no waveform reading. | The interactive editor. |
| Live track resize (added) | Dragging a track's resize handle reflows timeline content continuously. | [livewave][livewave], [relnotes][relnotes] (F) | missing | No GUI (ADR-0011). | The interactive editor. |
| Rectified waveforms | A display option for half-wave waveforms. | unconfirmed: no census page has it | missing | No waveform display. | — |
| Source Monitor audio and video waveforms (26.2) | Settings > Video and Audio Waveform Split (or the Drag Audio and Video icon) shows the waveform under the picture while playing or scrubbing. | [src-wave][src-wave], [whatsnew][whatsnew] (F) | missing | No waveform display. | — |
| Source Monitor audio inspection (added) | Audio clips open as waveforms; horizontal zoom bar; vertical zoom on one channel by the dB ruler (Shift for all); scrub to hear. | [src-mon][src-mon] (F) | missing | No monitor (ADR-0011); `query` reads values, not sound. | A waveform or level reading per source. |
| Sum multichannel outputs to mono in Source Monitor (added) | Audio preference mixing a source's channels to mono for monitoring. | [prefs][prefs] (F) | missing | No monitor. | — |
| Audio scrubbing | "Play audio while scrubbing in Source and Program Monitors"; "Maintain pitch while shuttling" (J, K, L). | [prefs][prefs], [src-mon][src-mon] (F) | partial | `preview` renders any span to an MP4 with its mix (`mcp.rs` `preview`); the master gain is the whole programme's (ADR-0172). | Interactive scrubbing and shuttling. |
| Audio Meters ‡ | Window > Audio Meters during playback; watch for peaks in the red zone. (The first pass's "dynamic peaks, valleys" is not on the page.) | [meters][meters] (F) | partial | `verify` reports integrated loudness, true peak, and per-span silence (EBU R128 momentary against −70 LUFS) of the deliverable (ADR-0172, 0173; `verify.rs`). | A level-over-time reading per element or track. |
| Audio hardware preferences | Device Class (ASIO, MME, CoreAudio), Default Input and Output, Clock, Latency, Sample Rate, I/O Buffer Size, force hardware to the document rate. | [hw][hw] (F) | missing | No playback or capture device layer; `render` writes a file. | — |
| Map output to hardware channels (added) | Preferences > Audio Hardware > Output Mapping: drag sequence channels onto device channels. | [hwmap][hwmap], [hw][hw] (F) | missing | No device layer. | — |
| Monitor Ambisonics audio | A VR ambisonics sequence, Binauralizer – Ambisonics on the Mix track, headphones; match its Pan/Tilt to the VR Viewer. | [ambi][ambi] (F) | missing | Stereo bus only. | Ambisonics. |
| Properties panel: audio controls ‡ | Select one or more audio clips: Volume slider, Mute, Pan (mono clips), keyframe button, links to Essential Sound and Adobe Stock audio. | [props][props] (F) | partial (was have) | `query --at` prints an element's values at an instant (`mcp.rs` `query`); the agent edits `volume` in the JSON. | Pan (ADR-0184 not built). |
| Properties panel: multi-clip editing (added) | With several clips selected, a shared value sets all; a dash means mixed, and dragging it adjusts each clip relatively. | [props][props] (F) | missing | No batch write tool (ADR-0011's write-tool invariant); the agent edits each element. | A batch command or skill script. |

### N. Voice-over recording

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Voice-over Record button on a track ‡ | Customize Audio Header adds the microphone button; marking an In/Out range starts the workflow; a pre-roll countdown set in Voice-Over Record Settings; the recording becomes a project item. | [vo][vo] (F) | missing | No capture; a recording enters as an ordinary `audio` element's `source`. | Capture (outside a file-in, file-out editor). |
| Record through the Audio Track Mixer ‡ | Enable Track For Recording (R), Track Input Channel, Meter Input(s) Only, Record then Play; Solo the track; the clip lands on the track and in the Project panel. | [rec-atm][rec-atm] (F) | missing | As above. | As above. |
| Recording setup (input device, mono/stereo jacks) | Audio Hardware preferences > Default Input; ASIO or Core Audio devices; match mono or stereo microphones to the jack. | [rec][rec], [hw][hw] (F) | missing | As above. | As above. |
| Mute input during timeline recording (added) | Audio preference that stops monitoring the input while recording, to avoid feedback. | [mute-input][mute-input], [prefs][prefs] (F) | missing | No capture. | — |

### O. Export and round trips

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio codec choice | Export Settings > Audio: Audio Format (for H.264, HEVC, MPEG2) and Audio Codec. | [export-ref][export-ref] (F) | partial | AAC is always written (`encode.rs`: `-c:a aac`). | Any other codec. |
| Audio bitrate | Bitrate [kbps] for the audio output. | [export-ref][export-ref] (F) | partial | Fixed 160 kbps (`encode.rs`; ADR-0173 §3). | A chosen bitrate. |
| Export sample rate | Export at the recorded rate; resampling costs time. | [export-ref][export-ref], [export][export] (F) | partial | Fixed 48 kHz (ADR-0077). | A chosen rate. |
| Export channels and downmix | Mono, Stereo or 5.1; fewer channels than the Mix track downmixes, only 5.1 → stereo or stereo → mono. | [export-ref][export-ref] (F) | partial | Always stereo. | Mono and 5.1 output. |
| Audio-only export | Export a finished sequence as an audio file, choosing layout, rate and bitrate. | [export][export] (F) | missing | `render` writes an MP4 with picture (`encode.rs`). | An audio-only deliverable. |
| Export loudness normalisation | Export Settings > Effects > Loudness Normalization: ATSC A/85, EBU R128 or ITU BS.1770-3 (only the last adjustable), true-peak limit (PRECEDENT §1); a Process checkbox per stream (26.x fix). | [ame-loud][ame-loud] (F, live), [relnotes][relnotes] (F) | have | `master.target_lufs` (one measured gain) and `master.ceiling_dbtp` (4× oversampled limiter) (ADR-0172, 0174). | (A standard is a literal number, not a dropdown.) |
| Loudness report | A loudness report generated with each export. | [ame-loud][ame-loud] (F, live) | have | `render` reports measured and applied gain; `verify` reports the deliverable's LUFS and dBTP as findings (ADR-0172, 0173). | (Not a separate report file.) |
| Separate audio tracks / stems in export | Export multiple audio tracks or channels to one container. Only Audition's Export to Adobe Premiere offers "each track as stem" ([remix-au][remix-au]). | unconfirmed for Premiere's own export | missing | One mixed stereo track. | Stems or multitrack audio output. |
| Edit in Adobe Audition | Edit > Edit in Adobe Audition > Clip renders a copy to a new WAV and opens it; saved edits replace the clip in the timeline; effects and markers kept. | [audition][audition] (F) | missing | No round trip; an externally processed file can be referenced as a new `source`. | A round-trip command. |
| Remix in Audition to match duration (added) | Edit in Adobe Audition > Sequence, Enable Remix (or Stretch) to a Target Duration, then Multitrack > Export to Adobe Premiere as mono, stereo, 5.1 or stems. | [remix-au][remix-au] (F) | missing | As above; the in-Premiere Remix row (section I) is the same gap. | A round trip, or the Remix skill. |

### P. Generative, AI and stock audio not covered above

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Generative Extend (audio) ‡ | Drag a clip's start or end to generate more ambient sound (cloud Firefly model); cannot extend dialogue (muted) or music; mono and stereo only; saved as a .wav. (The first pass's "at least 3 s" is not on the page.) | [genext][genext] (F) | refused | Model-driven audio: ADR-0145 literal values (#795 Out of scope). | — |
| Generative Media Tool: sound effects (26.5) (added) | Generate sound effects from a text prompt in the timeline, added as editable clips. | [whatsnew][whatsnew], [relnotes][relnotes] (F) | refused | Model-driven audio: ADR-0145 literal values (#795 Out of scope). A file generated elsewhere can still be a `source`. | — |
| Beat / tempo analysis (music) | Remix measures qualities of each beat and compares them across the song. No separate beat-marker feature is on any page. | [remix-about][remix-about] (F) | have (skill) | `skills/montagent-craft/scripts/beat_grid.py` detects tempo and downbeats and places the track (marked a workaround for #547). | Tempo and downbeat from `probe` (#547). |
| Adobe Stock audio in the Essential Sound panel (added) | Essential Sound > Browse: search by mood or genre, preview with Timeline sync, drag in a low-bitrate M4A preview, License to swap every instance for a 44.1 or 48 kHz WAV. | [stock][stock], [props][props] (F) | missing | No media service; a licensed file is referenced as any `source`. | A stock browser (outside the format). |

Rows that sit in other area tickets and are not counted here: Speech to Text and transcript-based editing (the
26.5 Voice Activity Detection model), caption audio checks (ADR-0054), media-intelligence search for sounds
(`organize-media/file-organization/search-for-audio-using-media-intelligence`, 26.0), bars and tone, 32-bit
float Hi-Res audio from Sony FX5 media (import), and Edit Clip in Audition's companion pages
(`use-premiere-with-other-apps`).

## Notable gaps

- **Half the rack is proposed but not built.** Noise reduction (ADR-0181), the noise gate (ADR-0183), pan
  and channel operations (ADR-0184) and echo (ADR-0185) have ADRs but are absent from the v0.2.0 schema. They
  account for more than a dozen `missing` rows, including the clip Panner, Channel Volume, and the Essential
  Sound Repair sliders; the Properties panel's Pan slider drops that row to partial.
- **No effect parameter is animatable.** Premiere keyframes every effect parameter and the clip Panner. In
  Montagent only `volume` moves, and every member is static until #842 decides otherwise.
- **The bus is 48 kHz stereo AAC 160k, fixed.** There is no 5.1, adaptive or multichannel mixing, no channel
  matrix or per-effect channel map, no chosen sample rate or codec, no audio-only export, and no stems
  (ADR-0077, ADR-0172). The 26.5 Match Source audio settings have nothing to match.
- **No reverb, pitch shift, modulation, de-ess, distortion or stereo width.** Reverb is explicitly left
  undecided (ADR-0185 owner call 10); the pitch unit is fixed (ADR-0170) but there is no member. Surround
  Reverb, like Convolution Reverb, is impulse-file based and so refused.
- **EQ is capped at 8 members and Q ≤ 10**, while Premiere's Parametric EQ reaches Q about 100, so Graphic EQ
  (10/20/30 bands), the full 9-section Parametric EQ, narrow notches and DeHum cannot be written exactly
  (ADR-0179).
- **Peak readings and peak normalisation are missing.** Montagent normalises loudness (per element and on the
  master), but has no peak readout, Normalize Max Peak, Normalize All Peaks or dB clip gain.
- **No waveform reading of any kind.** Four 26.x features (dynamic, live, Source Monitor split and Source
  Monitor inspection) are waveform displays; `verify` gives whole-file loudness, peak and silence spans only.
- **No linking or merged clips.** Merge, linked trims, multi-clip links and audio timecode all rest on a
  unit Montagent's flat element list does not have (ADR-0001); placement by literal `start` covers playback.
- **New AI audio is refused as format members:** Separate Crosstalk, the Generative Media Tool's sound
  effects, Enhance Speech, Auto Tag, Generative Extend and DeReverb. Bulk bleep is half there: a muted span is
  ordinary keyframes, but word detection and a tone source are not.
- **Music Remix and audio-waveform sync have no tool**, though neither is refused: each could be a skill
  script that writes literal cuts, crossfades or `start` values, as `beat_grid.py` and `duck.py` already do.
- **Refused by design:** track-level audio (faders, track effects and their copy/paste, submixes, sends,
  automation, Automatch Time), Essential Sound types, tags and presets, AI audio, impulse-file reverbs,
  third-party plug-ins and the exponential crossfade curve.

<!-- Premiere guide pages, census area `audio` (all fetched) -->
[gain]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/adjust-gain-in-audio.html
[trackvol]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/adjust-track-volume.html
[tag]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-category-tagging.html
[esp]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-editing-with-essential-sound-panel.html
[automatch]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/auto-match-audio-loudness.html
[duck]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/automatically-duck-audio.html
[presets]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/create-audio-presets.html
[reverb]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/create-reverb-effect.html
[dynwave]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/dynamic-audio-waveforms.html
[enh-req]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/enhance-speech-technical-requirements.html
[enh]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/enhance-speech.html
[gmute]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/global-mute.html
[clarity]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/improve-dialogue-clarity.html
[livewave]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/live-waveform-editing.html
[meters]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/monitor-volume-levels.html
[repair]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/repair-dialogue.html
[atm]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/about-audio-track-mixer.html
[multistereo]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/assign-multiple-audio-channels.html
[chmap-concept]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/audio-channel-mapping.html
[breakout]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/break-a-stereo-track-into-mono-tracks.html
[chmap]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/change-source-audio-channel-mapping.html
[bleep]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/clean-audio-with-bulk-mute-or-bleep.html
[submix]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/create-a-submix.html
[mclink]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/edit-a-multi-clip-link-in-the-source-monitor.html
[map-import]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/map-source-audio-channels-on-import.html
[hwmap]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/map-to-audio-output-device-hardware-channels.html
[remix-au]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/remix-audio-in-audition-to-match-video-duration.html
[mono-stereo]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/use-mono-clips-as-stereo-clips.html
[lmeter]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/about-loudness-meter.html
[trackfx-tl]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/adjust-audio-track-effects-in-timeline.html
[atm-fx]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/apply-effects-using-audio-track-mixer.html
[lm-apply]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/apply-loudness-meter-to-a-track.html
[lib]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-effects-library.html
[panctl]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-panning-and-balancing-controls-in-audio-track-mixer.html
[atm-auto]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-track-mixer-automation-modes.html
[channelize]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/channelize-effects.html
[atm-copy]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/copy-and-move-effects-in-audio-track-mixer.html
[fx-paste]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/copy-and-paste-audio-effects-in-the-timeline.html
[prepost]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/designate-a-track-effect-as-pre-fade-or-post-fade.html
[lm-settings]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/edit-loudness-meter-settings.html
[lm-measure]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/measure-the-loudness-of-your-track.html
[pan]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-stereo-track.html
[pan-track]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-track-in-timeline.html
[trackout]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-an-adaptive-track.html
[rec-mix]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/record-changes-to-sound-tracks.html
[remix]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-audio-in-premiere.html
[remix-about]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-in-premiere.html
[remix-tips]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-tool-considerations-in-premiere.html
[automatch-time]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/set-automatch-time-for-touch-mode-and-read-mode.html
[acm-auto]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/set-automation-modes-for-audio-clip-mixer.html
[kfopt]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/specify-automated-keyframe-creation.html
[xfade]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/audio-crossfade-transitions.html
[xfade-handles]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/crossfade-between-audio-clips.html
[fade]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/fade-in-or-fade-out-clip-audio.html
[trans-default]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/specify-default-audio-transitions.html
[concepts]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/audio-editing-concepts.html
[src-mon]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-audio-clips-in-the-source-monitor.html
[props]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-audio-using-the-properties-panel.html
[merge-edit]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-merged-clips.html
[merge-meta]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-metadata-of-merged-clips.html
[extract]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/extract-audio-from-clips.html
[merge-limits]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/limitations-of-merged-clips.html
[link]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/link-audio-and-video-clips.html
[merge]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-project-panel.html
[merge-tl]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-timeline-panel.html
[crosstalk]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/separate-overlapping-speakers.html
[sync]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/synchronize-audio-and-video.html
[sync-tl]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/synchronize-clips-in-the-timeline-panel.html
[merge-tc]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/use-the-timecode-from-an-audio-primary-clip-to-create-a-merged-clip.html
[src-wave]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/view-audio-and-video-waveforms-in-the-source-monitor.html
[stock]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/use-adobe-stock-audio/use-adobe-stock-audio-in-your-project.html
<!-- What's new pages (fetched) -->
[whatsnew]: https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html
[relnotes]: https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html
<!-- Premiere guide pages from other census areas (fetched) -->
[prefs]: https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html
[hw]: https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-hardware-preferences.html
[tl-prefs]: https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/timeline-preferences.html
[shortcuts]: https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html
[ambi]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/monitor-ambisonics-audio.html
[ambi-restage]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/restage-correctly-aligned-video-and-audio.html
[speed]: https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-using-the-speedduration-option.html
[jl]: https://helpx.adobe.com/premiere/desktop/edit-projects/trim-clips/perform-j-cuts-and-l-cuts.html
[multitrans]: https://helpx.adobe.com/premiere/desktop/edit-projects/intro-to-editing/apply-multi-transitions-across-audio-and-video-clips.html
[seqset]: https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/sequence-settings-reference.html
[genext]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-extend-overview.html
[crosstalk-ai]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/about-cloud-based-ai-voice-features.html
[ecp]: https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/about-effect-controls-panel.html
[kfinterp]: https://helpx.adobe.com/premiere/desktop/add-video-effects/control-effects-and-transitions-using-keyframes/change-keyframe-interpolation-method.html
[sr]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/audio-sample-rates-in-premiere.html
[conform]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/audio-conforming-specifications.html
[digaudio]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/importing-digital-audio.html
[vo]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-a-voice-over-on-an-audio-track-from-the-timeline.html
[rec-atm]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-audio-using-the-audio-track-mixer.html
[rec]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-audio.html
[mute-input]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/mute-input-during-recording.html
[export-ref]: https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/overview-of-export-settings.html
[export]: https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video-and-audio-files.html
[plugins]: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html
[audition]: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/edit-premiere-audio-clips-in-audition.html
[obsolete-ts]: https://helpx.adobe.com/premiere/desktop/troubleshooting/limitations-and-known-issues/ui-errors-when-using-obsolete-effects-in-on-macos-dark-mode.html
<!-- Outside the Premiere guide (fetched live, 2026-10-10) -->
[ame-loud]: https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/automatic-loudness-correction.html

## Pages covered

Every page of the census area `audio` (72), each mapped to the rows above that cover it. Section letters
refer to the tables. The three What's-new pages were also read in full; their audio items are rows here
(Global Mute, Source Monitor waveforms, live and dynamic waveforms, Multi Transitions, bulk bleep, Match
Source audio, Generative Media Tool sound effects, transition handles) and the beta page has no audio feature.

| Page URL | Rows / "no feature: <reason>" |
|---|---|
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/adjust-gain-in-audio.html | J: Audio Gain: Set Gain to / Adjust Gain by; Peak Amplitude readout; Normalize Max Peak to; Normalize All Peaks to |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/adjust-track-volume.html | G: Volume (fixed effect); J: Clip volume keyframes; Clip effect-parameter keyframes; Track keyframes |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-category-tagging.html | I: Audio Category Tagging (Auto Tag) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-editing-with-essential-sound-panel.html | I: Audio types; Essential Sound presets |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/auto-match-audio-loudness.html | I: Loudness > Auto-Match |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/automatically-duck-audio.html | I: Ducking (Auto Duck) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/create-audio-presets.html | I: Essential Sound presets |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/create-reverb-effect.html | I: Creative > Reverb; Clip Volume (Level, Mute) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/dynamic-audio-waveforms.html | M: Dynamic Audio Waveforms; G: Volume (fixed effect); I: Clip Volume (Level, Mute) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/enhance-speech-technical-requirements.html | I: Clarity > Enhance Speech (its format limits) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/enhance-speech.html | I: Clarity > Enhance Speech |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/global-mute.html | K: Global Mute (26.2) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/improve-dialogue-clarity.html | I: Clarity > Dynamics; Clarity > EQ; Clarity > Vocal Enhancer |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/live-waveform-editing.html | M: Live waveform editing; Live track resize |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/monitor-volume-levels.html | M: Audio Meters |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/repair-dialogue.html | I: Repair > Reduce Noise; Reduce Rumble; DeHum; DeEss; Reduce Reverb |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/about-audio-track-mixer.html | K: Audio Track Mixer: track fader; effect slots; automation modes; Audio Clip Mixer; Submix tracks; Sends; Mix (master) track volume and loudness |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/assign-multiple-audio-channels.html | K: Multiple stereo assignments to a multichannel master |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/audio-channel-mapping.html | L: Modify > Audio Channels (source channel mapping) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/break-a-stereo-track-into-mono-tracks.html | L: Breakout to Mono |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/change-source-audio-channel-mapping.html | L: Modify > Audio Channels (source channel mapping); Hide audio channel labels |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/clean-audio-with-bulk-mute-or-bleep.html | L: Bulk mute or bleep |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/create-a-submix.html | K: Submix tracks |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/edit-a-multi-clip-link-in-the-source-monitor.html | L: Edit a multi-clip link in the Source Monitor |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/map-source-audio-channels-on-import.html | L: Default Audio Tracks on import |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/map-to-audio-output-device-hardware-channels.html | M: Map output to hardware channels |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/remix-audio-in-audition-to-match-video-duration.html | O: Remix in Audition to match duration; Separate audio tracks / stems in export (note) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/use-mono-clips-as-stereo-clips.html | L: Use a mono clip as stereo |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/about-loudness-meter.html | G: Loudness Meter |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/adjust-audio-track-effects-in-timeline.html | J: Track keyframes |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/apply-effects-using-audio-track-mixer.html | K: Audio Track Mixer: effect slots (inserts); G: Effect bypass |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/apply-loudness-meter-to-a-track.html | G: Loudness Meter; K: Mix (master) track effects |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-effects-library.html | Sections A–G: every library effect, one row each (Amplify … Pitch Shifter, Notch Filter presets, Obsolete audio effects); 46 rows |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-panning-and-balancing-controls-in-audio-track-mixer.html | K: track pan/balance; 5.1 Panner tray; Pan a nested sequence as one element; Automatic downmix to fewer output channels; L: Multichannel master and 5.1 mixing |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-track-mixer-automation-modes.html | K: Audio Track Mixer: automation modes |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/channelize-effects.html | G: Effect channel layout (Channelize effects) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/copy-and-move-effects-in-audio-track-mixer.html | K: Copy and move track effects |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/copy-and-paste-audio-effects-in-the-timeline.html | K: Copy and paste track-effect keyframes; G: Effect bypass |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/designate-a-track-effect-as-pre-fade-or-post-fade.html | K: Pre-fader / post-fader effect placement; Sends |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/edit-loudness-meter-settings.html | G: Loudness Meter settings |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/measure-the-loudness-of-your-track.html | G: Loudness Meter |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-stereo-track.html | K: Audio Track Mixer: track pan/balance |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-track-in-timeline.html | G: Panner (fixed effect); J: Track keyframes |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-an-adaptive-track.html | K: Direct output / track output channel assignment; Sends; L: Multichannel master and 5.1 mixing |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/record-changes-to-sound-tracks.html | K: Record audio mix changes; Audio Track Mixer: automation modes |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-audio-in-premiere.html | I: Music > Duration (Remix) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-in-premiere.html | I: Music > Duration (Remix); P: Beat / tempo analysis |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-tool-considerations-in-premiere.html | I: Music > Duration (Remix) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/set-automatch-time-for-touch-mode-and-read-mode.html | K: Automatch Time |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/set-automation-modes-for-audio-clip-mixer.html | K: Audio Clip Mixer: automation (Latch, Touch); G: Panner (fixed effect); J: Clip pan keyframes |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/specify-automated-keyframe-creation.html | J: Automation keyframe thinning |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/audio-crossfade-transitions.html | H: Constant Power crossfade; Constant Gain crossfade; Exponential Fade |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/crossfade-between-audio-clips.html | H: Fade handles |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/fade-in-or-fade-out-clip-audio.html | H: Fade in / fade out a clip; Fade handles |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/specify-default-audio-transitions.html | H: Default audio transition and its duration; Edit an audio transition; Fade in / fade out a clip |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/audio-editing-concepts.html | K: Audio track types; Audio Clip Mixer; I: Audio types; J: Clip volume keyframes |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-audio-clips-in-the-source-monitor.html | M: Source Monitor audio inspection; Timeline audio waveforms; Audio scrubbing |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-audio-using-the-properties-panel.html | M: Properties panel: audio controls; Properties panel: multi-clip editing; J: Mute (clip); Clip pan keyframes; G: Panner (fixed effect) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-merged-clips.html | L: Trim merged clips |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/edit-metadata-of-merged-clips.html | L: Merged clip metadata |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/extract-audio-from-clips.html | L: Extract Audio |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/limitations-of-merged-clips.html | L: Merge Clips (its limits); Synchronize by audio waveform |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/link-audio-and-video-clips.html | L: Link / unlink audio and video |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-project-panel.html | L: Merge Clips |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-timeline-panel.html | L: Merge Clips |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/separate-overlapping-speakers.html | L: Separate Crosstalk |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/synchronize-audio-and-video.html | L: Merge Clips |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/synchronize-clips-in-the-timeline-panel.html | L: Synchronize clips; Synchronize by audio waveform |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/use-the-timecode-from-an-audio-primary-clip-to-create-a-merged-clip.html | L: Audio timecode from a primary clip; Remove Audio from AV Clip |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/view-audio-and-video-waveforms-in-the-source-monitor.html | M: Source Monitor audio and video waveforms (26.2) |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/use-adobe-stock-audio/about-adobe-stock-audio-licensing-usage-rights-and-audio-quality.html | no feature: licence terms for Adobe Stock audio (permitted uses, extended licence, YouTube claims), not an editing mechanism |
| https://helpx.adobe.com/premiere/desktop/add-audio-effects/use-adobe-stock-audio/use-adobe-stock-audio-in-your-project.html | P: Adobe Stock audio in the Essential Sound panel |
