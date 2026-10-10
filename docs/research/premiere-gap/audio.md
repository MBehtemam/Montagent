# Premiere Pro audio, set against Montagent v0.2.0

Research for [#889](https://github.com/MBehtemam/Montagent/issues/889), part of the map
[#882](https://github.com/MBehtemam/Montagent/issues/882) ("every Premiere Pro feature, set against
Montagent v0.2.0"). Area: **audio**.

- **Date:** 2026-10-10. Nothing was built or run. **This note decides nothing.** It lists features and
  states what v0.2.0 ships against each one.
- **Premiere side:** Adobe's own pages only: the Premiere user guide on helpx.adobe.com, its What's new and
  release-notes pages (through 26.5, September 2026), the Media Encoder page that Premiere's export uses, and
  Adobe's Audition pages for the effects Adobe says are shared with Premiere ([shared effects][au-shared]).
- **Montagent side:** tag `v0.2.0` (`51aa4b5`, which is `origin/main`). Read: `CONTEXT.md`,
  `schema/montagent.schema.json`, ADR-0004, 0020, 0055, 0077, 0145, 0157, 0169–0185, the audio map
  [#795](https://github.com/MBehtemam/Montagent/issues/795) (its Out of scope), the MCP tool surface
  (`crates/montagent/src/mcp.rs`: `validate`, `create_project`, `query`, `frame`, `render`, `verify`,
  `preview`, `measure`, `shift`, `compare`), the render graph (`crates/montagent-core/src/verbs/render.rs`,
  `render/master.rs`, `crates/montagent-render/src/encode.rs`) and `skills/`.
- **Precedent reused, not redone:** [`docs/research/audio-effects/PRECEDENT.md`](../audio-effects/PRECEDENT.md)
  (§1–§14) settled loudness normalisation, pan, channel operations, crossfades, EQ, dynamics, noise reduction,
  de-ess, reverb, echo, pitch shift, voice changers, pitch preservation and auto-ducking. Its rows are listed
  below and marked **PRECEDENT §n**; the mechanism cell then summarises, it does not re-argue.

## How this was read

The sandbox could not open helpx.adobe.com directly (the egress proxy refuses the host, the same finding
PRECEDENT.md records). Every Adobe page was reached through a web-search engine restricted to
`helpx.adobe.com`, which returns an extract of the indexed page. The search budget for the session ran out
before every page was reached, so:

- A mechanism with no marker is stated in the extract of the cited Adobe page.
- **(R)** marks a mechanism or an effect name that is the author's knowledge of the product and was not seen
  in an extract this session. The URL is then the Adobe page that should hold it (in most cases the effects
  library page, whose category headings were confirmed but whose full list was not returned). Re-open the
  URL before quoting it.
- Montagent evidence was read from the repo at the tag, not recalled.

## What "shipped" means at v0.2.0

The audio map's ADRs are in three states, and the status column follows the **schema**, not the ADR:

- **Built** (in the schema and the render graph): `volume` (ADR-0055, 0175), `audio_effects` with seven
  members, `normalize_loudness`, `highpass`, `lowpass`, `shelf`, `bell`, `compressor`, `limiter`
  (ADR-0169, 0178–0180), the `master` stage (ADR-0172, 0174), the transition `audio` field and the
  `audio_crossfade` kind (ADR-0176), and `duck.py` (ADR-0177). ADR-0176–0180 are still `proposed` until
  their three-leg measured tables are committed (`CONTEXT.md`, **Audio effect**), but they are built.
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
- **Model- or detector-driven audio** (AI enhance voice, de-reverb, beat detection) and **convolution from an
  impulse-response file** fail ADR-0145's literal-values invariant, as #795's Out of scope records. Note that
  this refuses them *as format members*; a skill that runs a detector and writes literals (as `beat_grid.py`
  and `duck.py` do) is a separate door.

## Counts

| Status | Rows |
|---|---|
| have | 23 |
| have (skill) | 2 |
| partial | 34 |
| missing | 67 |
| refused | 22 |
| **Total** | **148** |

## The table

Abbreviations in the evidence column: **S** = `schema/montagent.schema.json`, **C** = `CONTEXT.md`.

### A. Audio effects: Amplitude and Compression

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Amplify | Clip/track effect: per-channel gain sliders in dB, with a link toggle. | [lib][lib] | partial | `volume` (ADR-0055): one linear multiplier, `>1` amplifies, post-fader (ADR-0169). | A dB gain, per-channel gain, and a pre-effect position: `volume` is linear, whole-element, and runs after `audio_effects`. |
| Channel Mixer | Effect: a matrix setting how much of each input channel feeds each output channel, with per-channel invert. | [lib][lib] | missing | None. ADR-0184's `channels` (proposed, not in S) is five fixed words, not a matrix. | Any channel-mix matrix; per-channel polarity. |
| Channel Volume | Fixed clip effect: per-channel level in dB (PRECEDENT §3). | [lib][lib] | missing | ADR-0184 §8 defers it: "a later separate member" (per-channel `_db`, ADR-0170). | The member. |
| DeEsser | Effect: Broadband/Multiband mode, threshold, centre frequency, bandwidth, sibilance-only monitor (PRECEDENT §8). | [lib][lib] | missing | No member in S. `compressor` (ADR-0180) is full-band, with no side-chain filter. | A band-limited (side-chain filtered) compressor or a `de_ess` member. |
| Dynamics | Four-section effect: AutoGate, Compressor, Expander, Limiter (PRECEDENT §6). | [lib][lib] | partial | `compressor` and `limiter` members (ADR-0180, S `audio_effects`). | The gate (`noise_gate`, ADR-0183, proposed, not in S) and the downward expander (no ADR). |
| Dynamics Processing | Effect: an editable input/output transfer graph, peak or RMS detection, band limiting with low/high cutoff (PRECEDENT §6). | [lib][lib] | partial | `compressor` (ADR-0180): one knee-less threshold/ratio, RMS detection only. | An arbitrary multi-segment transfer curve, peak detection (ADR-0180 refuses `detection: peak` for its member), band limiting. |
| Hard Limiter | Effect: Maximum Amplitude, Input Boost, Look-ahead, Release, Link Channels (Audition-shared; values RECALLED in PRECEDENT §6). | [au-amp][au-amp] | partial | `limiter` member: `ceiling_db`, `release_ms` (ADR-0180). `master.ceiling_dbtp` limits the mix at 4× rate (ADR-0174). | Input boost (place a `volume` or `makeup_db` before it instead), a look-ahead parameter, channel-link choice. |
| Multiband Compressor | Effect: four bands with crossover frequencies, each band its own threshold/ratio/attack/release (Audition-shared). | [au-shared][au-shared] | missing | No member. | The member, and band splitting. |
| Single-band Compressor | Effect: threshold, ratio, attack, release, output gain (Audition-shared). | [au-amp][au-amp] | have | `compressor`: `threshold_db`, `ratio`, `attack_ms`, `release_ms`, `makeup_db`, all required (ADR-0180, S). | (Departure: no defaults, RMS threshold; ADR-0180.) |
| Tube-modeled Compressor | Effect: compressor with tube-saturation colouring (Audition-shared). | [au-shared][au-shared] | partial | `compressor` (ADR-0180). | The tube/saturation modelling. |

### B. Audio effects: Delay and Echo

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Delay | Effect: delay time, feedback, mix (PRECEDENT §10). | [lib][lib] | missing | `echo` member (`delay_ms`, `feedback`, `mix`) is ADR-0185, **proposed, not in S**. | The built member. |
| Analog Delay | Effect: delay with saturation ("Trash") and spread, modelling hardware units (PRECEDENT §10). | [lib][lib] | missing | ADR-0185 (proposed) covers a clean echo only. | Built echo, plus saturation/spread modelling. |
| Multitap Delay | Effect: several independent delay taps (PRECEDENT §10). | [lib][lib] | missing | ADR-0185 (proposed) is one tap; two stacked `echo` members would feed each other in series, not in parallel. | Parallel taps. |

### C. Audio effects: Filter and EQ

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Bandpass | Effect: centre frequency and Q; passes one band (R for the parameters). | [lib][lib] | partial | A `highpass` plus a `lowpass` member in series (ADR-0179). | A centre/Q spelling; slopes are only 12/24/48 dB/oct. |
| Bass | Effect: boost/cut below about 200 Hz. | [fx-old][fx-old] | have | `shelf` with `side: "low"`, `frequency_hz`, `gain_db` (ADR-0179). | — |
| FFT Filter | Effect: a drawn frequency curve, FFT size and window shape. | [fx-old][fx-old] | missing | Up to 8 EQ members (`E-EQ-STACK-CAP`, ADR-0179) can approximate a curve. | An arbitrary drawn response. |
| Graphic Equalizer (10, 20, 30 bands) | Effect: fixed bands at 1, ½ or ⅓ octave spacing (PRECEDENT §5). | [lib][lib] | partial | `bell` members (ADR-0179). | 10/20/30 bands: `E-EQ-STACK-CAP` refuses more than 8 enabled EQ members on one element. |
| Highpass | Effect: cutoff frequency. | [lib][lib] | have | `highpass`: `frequency_hz` 20..20000, `slope_db_per_oct` 12/24/48 (ADR-0179). | — |
| Lowpass | Effect: cutoff frequency. | [lib][lib] | have | `lowpass` (ADR-0179). | — |
| Notch Filter | Effect: up to six notches, widths Narrow to Super Narrow, gain per notch (PRECEDENT §5). | [fx-old][fx-old] | partial | `bell` with negative `gain_db` (ADR-0179). | Very narrow notches: `q` is capped at 10 and `gain_db` at −24; no notch member. |
| Parametric Equalizer | Effect: HP, LP, two shelves and five bell bands, each with frequency, gain, Q (PRECEDENT §5). | [lib][lib] | partial | `highpass`, `lowpass`, `shelf`, `bell` (ADR-0179). | Premiere's full set is 9 sections and the cap is 8; Q range 0.1..10; slopes 12/24/48 only. |
| Scientific Filter | Effect: Bessel, Butterworth, Chebyshev, Elliptical; low/high/band-pass/band-stop; master gain. | [fx-old][fx-old] | partial | The EQ members are Butterworth biquads (ADR-0179). | The other three families and band-stop as one filter. |
| Simple Notch Filter | Effect: one notch frequency (R in Premiere; named on the Premiere Elements page). | [lib][lib] | partial | `bell` with negative gain (ADR-0179). | Narrow notches beyond `q` 10. |
| Simple Parametric EQ | Effect: one bell band (R). | [lib][lib] | have | `bell` (ADR-0179). | — |
| Treble | Effect: boost/cut above about 4 kHz. | [fx-old][fx-old] | have | `shelf` with `side: "high"` (ADR-0179). | — |

### D. Audio effects: Modulation

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Chorus/Flanger | Effect: multiple short modulated delays with feedback. | [lib][lib] | missing | No member; no ADR. | The member. |
| Flanger | Effect: initial/final delay, feedback, inverted or sinusoidal modes. | [lib][lib] | missing | No member; no ADR. | The member. |
| Phaser | Effect: swept phase-shifting filters. | [au-mod][au-mod] | missing | No member; no ADR. | The member. |

### E. Audio effects: Noise Reduction / Restoration

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Adaptive Noise Reduction | Effect (Audition-shared): learns and removes broadband noise. | [au-shared][au-shared] | refused | Adaptive (detector-driven) processing: ADR-0145 literal values, per #795 Out of scope. | — |
| Automatic Click Remover | Effect: removes clicks and crackle (vinyl). | [lib][lib] | missing | No member; no ADR. | The member. |
| DeHummer | Effect: removes 50/60 Hz hum and its harmonics. | [lib][lib] | partial | Stacked negative `bell` members at the fundamental and harmonics (ADR-0179). | A harmonic-series member; narrow enough notches (`q` ≤ 10); the 8-member cap limits harmonics. |
| DeNoise | Effect: broadband noise reduction with an Amount (PRECEDENT §7). | [lib][lib] | missing | `reduce_noise` (`reduction_db`, over `anlmdn`) is ADR-0181, **proposed, not in S**; its dial's transfer to real hiss is open (ADR-0181 §4). | The built member. |
| DeReverb | Effect: reduces room reverberation (PRECEDENT §7). | [noise-adv][noise-adv] | refused | "de-reverb" is listed as model/detector-driven: #795 Out of scope, ADR-0145 literal values. | — |

### F. Audio effects: Reverb

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Convolution Reverb | Effect: impulse-response file, mix, room size, damping, pre-delay (PRECEDENT §9). | [lib][lib] | refused | #795 Out of scope: "an external reference, not a literal parameter set" (ADR-0145 literal values). | — |
| Studio Reverb | Effect: algorithmic room size, decay, early reflections, width, HF/LF cut, damping, diffusion, dry/wet (PRECEDENT §9). | [reverb][reverb] | missing | ADR-0185 owner call 10: reverb is "out of this ADR … to get its own decision or none". | The member and its ADR. |
| Surround Reverb | Effect: reverb for 5.1 material. | [lib][lib] | missing | No member; the bus is stereo only (ADR-0077). | The member and a 5.1 bus. |

### G. Audio effects: Special, Stereo Imagery, Time and Pitch

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Balance | Effect: stereo balance −100..+100 (PRECEDENT §2). | [lib][lib] | missing | `pan` (−1..+1 linear balance) is ADR-0184, **proposed, not in S**. | The built field. |
| Binauralizer – Ambisonics | Effect: renders ambisonic audio to binaural for headphones. | [ambi][ambi] | missing | Stereo bus only (ADR-0077). | Ambisonic channel formats and the decode. |
| Distortion | Effect: editable positive/negative transfer curves, time smoothing, dB range. | [fx-old][fx-old] | missing | No member; no ADR. | The member. |
| Fill Left with Right | Effect: one channel copied to both (PRECEDENT §3; direction to be settled by test). | [lib][lib] | missing | ADR-0184's `channels: "left"/"right"` (proposed, not in S). | The built field. |
| Fill Right with Left | As above, the other side. | [lib][lib] | missing | ADR-0184 (proposed, not in S). | The built field. |
| Guitar Suite | Effect: compressor, filter, distortion and box modeller in series. | [lib][lib] | missing | `compressor` and EQ exist; no distortion or modeller. | Distortion and cabinet modelling. |
| Invert | Effect: polarity inversion (PRECEDENT §3). | [lib][lib] | missing | ADR-0184 §8: "No Invert … a later `audio_effects` member if wanted". | The member. |
| Loudness Meter | Track/mix effect: ITU BS.1770 meter with EBU, ATSC, Spotify, Apple Podcasts, Netflix, YouTube presets; does not change audio. | [lmeter][lmeter] | partial | `verify` measures the deliverable's integrated LUFS and true peak and judges them against `master` (ADR-0172, 0173; `verify.rs`). `render` reports the measured mix loudness when `target_lufs` is set. | Momentary/short-term/range readings over time, per-track metering, platform presets (a target is a literal number). |
| Loudness Radar | Effect (Audition-shared): radar display of loudness over time. | [au-shared][au-shared] | partial | `verify` (as above). | A loudness-over-time reading. |
| Mastering | Effect: EQ, reverb, exciter, widener and loudness maximiser in one rack (Audition-shared). | [au-shared][au-shared] | partial | On an element: EQ members, `compressor`, `limiter`. On the mix: `master` loudness + true-peak ceiling (ADR-0172). | Exciter, widener, reverb; and any effect on the mix itself, which ADR-0172 refuses (see section M). |
| Panner – Ambisonics | Effect: rotates the ambisonic sound field to match the picture. | [ambi][ambi] | missing | Stereo bus only. | Ambisonics. |
| Pitch Shifter | Effect: semitones, cents, ratio, formant preserve (PRECEDENT §11). | [lib][lib] | missing | ADR-0170 names the unit (semitones, fractional) but no member exists. | The member and its ADR. |
| Stereo Expander | Stereo Imagery effect: width 0 (narrow/normal) to 300 (wide). | [lib][lib] | missing | No member; ADR-0184 says pan is balance only, a stereo-image pan is "a later" call. | The member. |
| Swap Channels | Effect: left and right exchanged (PRECEDENT §3). | [lib][lib] | missing | ADR-0184 `channels: "swap"` (proposed, not in S). | The built field. |
| Vocal Enhancer | Effect: Male/Female/Music modes; reduces sibilance, plosives, handling rumble. | [lib][lib] | missing | `highpass` covers rumble only. | The member (and de-ess, plosive handling). |
| Voice-changer presets | Named Pitch Shifter / Essential Sound presets (PRECEDENT §12). | [lib][lib] | missing | No pitch member; effect presets are not part of the format (see section I). | Pitch Shifter first. |
| Volume (fixed effect) | Every clip's fixed Volume: Level, keyframable, Bypass. | [gainvol][gainvol] | have | `volume` scalar or `{t,v,ease}` keyframes (ADR-0055, 0175). | (Departure: linear, not dB; ADR-0055, 0170.) |
| Panner (fixed effect) | Every clip's fixed Panner: Balance (stereo) or Pan (mono), keyframable (PRECEDENT §2). | [pan][pan] | missing | `pan` is ADR-0184, proposed, not in S, and static (keyframes deferred under #842). | The built field and keyframes. |
| Obsolete audio effects | A library section: an obsolete effect in an opened project prompts to be replaced by its current version; obsolete items are hidden but still render (26.0 page). (R) names include the legacy Chorus, DeClicker, DeCrackler, DeEsser, DeHummer, DeNoiser, Dynamics, EQ, Flanger, Multiband Compressor, Pitch Shifter, Reverb, Spectral NoiseReduction. | [lib][lib], [fx26][fx26] | missing | Each legacy effect maps to its current row above; none of them is shipped beyond what that row says. The format has no "obsolete" tier: a member is in the closed union or is an unknown key (ADR-0169). | Whatever its current replacement's row lists. |
| Third-party audio plug-ins (VST, VST3, AU) | Audio Plug-in Manager: scan, rescan, enable/disable; plug-ins appear as effects. | [plugins][plugins] | refused | ADR-0145 closed-vocabulary and checkable-by-`validate` invariants; `audio_effects` is a closed union keyed on `name` (ADR-0169). | — |

### H. Audio transitions and fades

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Constant Power crossfade | Audio transition on an edit point; default; like a video dissolve (PRECEDENT §4). | [xfade][xfade] | have | Transition `audio: "constant_power"`, or `kind: "audio_crossfade"` (ADR-0176, S). | (Mechanism differs: the two elements sit on two tracks and overlap; Premiere's sits on one track.) |
| Constant Gain crossfade | Audio transition with a constant rate; can dip at the midpoint (PRECEDENT §4). | [xfade][xfade] | have | `audio: "constant_gain"` (ADR-0176). | — |
| Exponential Fade | Audio transition, a gradual logarithmic fade (PRECEDENT §4). | [xfade][xfade] | refused | ADR-0176 §1, Rejected: "A third curve, `exponential` … no rule an agent can pick by"; the door is kept open. | — |
| Audio follows a video transition | Shift+D applies the default video and audio transitions together; a video transition alone does not crossfade audio (PRECEDENT §4). | [trans][trans] | have | One `audio` field on `crossfade`, `wipe`, `slide`, `push` carries the sound across the same window (ADR-0176). | — |
| Fade in / fade out a clip | A transition at one clip end, or volume keyframes, or fade handles. | [fade][fade] | have | Two `volume` keyframes (ADR-0055), heard on the sample their instant names (ADR-0175). | — |
| Fade handles (26.x) | Draggable handles on audio clip edges shape fades and crossfades; Option constrains to one axis; 26.5 adds show/hide transition handles. | [whatsnew][whatsnew] | missing | No GUI; the outcome is written as keyframes or a transition. | The interactive editor (Montagent ships none, ADR-0011). |
| Default audio transition and its duration | Preferences > Timeline > Audio Transition Default Duration; Ctrl/Cmd+Shift+D applies it (PRECEDENT §4). | [trans-default][trans-default] | missing | Every transition is written with literal `start`/`end`/`audio`; the format holds no editor defaults. | An apply-default command (an editor convenience, not a format feature). |
| Multi Transitions (26.0) | Apply, edit and manage transitions across many audio and video clips at once. | [whatsnew][whatsnew] | missing | The agent writes each `transition` element; no batch write tool (ADR-0011's write-tool invariant). | A batch command or skill script. |
| J-cut / L-cut | Linked audio and video trimmed independently. | [concepts][concepts] | have | Separate elements with their own `start`/`end`; ADR-0176 §2: a J/L cut whose window is not the overlap "stays hand-written as `volume` keyframes". | — |

### I. Essential Sound panel

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio types: Dialogue, Music, SFX, Ambience | Each clip is tagged with one type, which opens that type's controls. | [esp][esp] | refused | ADR-0178 §2 rejects "a `role` or `"bed"` token … a second vocabulary and a hidden default". | — |
| Audio Category Tagging (Auto Tag) | AI classifier tags selected clips Dialogue/Music/SFX/Ambience; badge on the clip; Edit Audio Type overrides. | [tag][tag] | refused | No type vocabulary (ADR-0178 §2); a model-driven classifier fails ADR-0145 literal values (#795 Out of scope). | — |
| Essential Sound presets (per type, saved) | Presets per type that store the panel's settings. | [esp][esp] | refused | ADR-0178 §2: "per-type presets, which #816 already refused"; every member parameter is required, nothing defaults (ADR-0179, 0180). | — |
| Loudness > Auto-Match | Measures a clip and sets its gain to a per-type LUFS target (PRECEDENT §1). | [automatch][automatch] | have | `normalize_loudness` with an authored `target_lufs` −40..−6, one measured gain, singular (ADR-0178, S). | (The target is a literal per element, not per type.) |
| Repair > Reduce Noise | Slider 0–10 applying DeNoise underneath (PRECEDENT §7). | [repair][repair] | missing | ADR-0181 (`reduce_noise`) proposed, not in S. | The built member. |
| Repair > Reduce Rumble | Slider removing low-frequency rumble. | [repair][repair] | have | `highpass` (ADR-0179). | (A literal cutoff and slope, not a 0–10 slider.) |
| Repair > DeHum | Slider with 50/60 Hz choice. | [repair][repair] | partial | Stacked negative `bell` members (ADR-0179). | As DeHummer, section E. |
| Repair > DeEss | Slider reducing sibilance. | [repair][repair] | missing | As DeEsser, section A. | A de-ess member. |
| Repair > Reduce Reverb | Slider applying DeReverb. | [repair][repair] | refused | As DeReverb, section E (#795, ADR-0145). | — |
| Clarity > Dynamics | Natural-to-Focused slider: compression and levelling. | [clarity][clarity] | partial | `compressor` (ADR-0180). | A one-dial macro; every compressor key must be written. |
| Clarity > EQ (presets + amount) | Preset dropdown of Parametric EQ settings and an amount slider. | [clarity][clarity] | partial | EQ members (ADR-0179). | Named presets (refused for the format, see above) and an amount dial. |
| Clarity > Enhance Speech | AI speech enhancement with a Mix Amount slider; has technical requirements. | [enh][enh] | refused | "AI enhance voice" is #795 Out of scope (ADR-0145 literal values). | — |
| Clarity > Vocal Enhancer | Male/Female voice enhancement toggle. | [clarity][clarity] | missing | As Vocal Enhancer, section G. | The member. |
| Creative > Reverb | Toggle and preset dropdown of Studio Reverb settings (PRECEDENT §9). | [esp][esp] | missing | No reverb member (ADR-0185 leaves it undecided). | The member. |
| Music/SFX > Stereo Width | Width slider. | [esp][esp] | missing | No stereo-width member. | The member. |
| SFX > Pan | Pan slider for a sound effect. | [esp][esp] | missing | ADR-0184 `pan`, proposed, not in S. | The built field. |
| Music > Duration (Remix) | Retimes a music clip to a Target Duration by cutting and crossfading at detected seams; Segments and Variations controls; Remix tool in the toolbar; usually within 1 s. | [remix][remix] | missing | Montagent has the outputs (elements, `overrun: "loop"`, `audio_crossfade`), but nothing computes them. No ADR refuses a skill that would; the analysis itself could not be a format member (ADR-0145). | A skill script that analyses the track and writes cuts and crossfades, as `beat_grid.py` writes a placement. |
| Ducking (Auto Duck) | On Music/SFX/Ambience: Duck Against types, Sensitivity, Duck Amount (dB), Fades (ms), Generate Keyframes; writes ordinary keyframes (PRECEDENT §14). | [duck][duck] | have (skill) | `skills/montagent-footage/scripts/duck.py` writes ordinary `volume` keyframes from voice word timings or explicit spans: `under_db`, `over_db`, `end_db`, `ramp_ms`, `lead_ms`, `join_ms`, `fade_ms` (ADR-0177; C **Duck**). | Signal-based Sensitivity: ADR-0177 §2 rejects silence detection, so the voice's timings must be supplied. |
| Clip Volume (Level, Mute) | Panel's own volume slider and mute. | [esp][esp] | have | `volume`; `volume: 0` is mute, with no `mute` field (ADR-0055). | — |

### J. Clip and track level, gain, normalise, keyframes

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio Gain: Set Gain to / Adjust Gain by | Clip > Audio Options > Audio Gain (G): a non-keyframed pre-effect clip gain in dB, combined with track level. | [gain][gain] | partial | `volume` (linear, post-effects, ADR-0169); `compressor.makeup_db` or a `normalize_loudness` gain sit before effects that follow them. | A plain pre-effect gain in dB on the element. |
| Normalize Max Peak to | One gain applied to all selected clips so the loudest peak meets a dB target (PRECEDENT §1). | [gain][gain] | missing | No peak normalisation; `normalize_loudness` is loudness, not peak (ADR-0178). | A peak target. |
| Normalize All Peaks to | Each selected clip gained separately to a peak target (PRECEDENT §1). | [gain][gain] | missing | As above. | A peak target. |
| Normalize Mix Track | Sequence > Normalize Mix Track: adjusts the mix fader to a peak target, scaling mix keyframes. | [gainvol][gainvol] | partial | `master.target_lufs` brings the whole mix to a loudness by one measured gain (ADR-0172). | A peak (not loudness) target on the mix. |
| Clip volume keyframes | Clip Keyframes > Volume > Level in the timeline or Effect Controls. | [gainvol][gainvol] | have | `volume` `{t,v,ease}` records (ADR-0055), heard on their sample (ADR-0175). | — |
| Clip effect-parameter keyframes | Every effect parameter with a stopwatch is keyframable (PRECEDENT, "Where Premiere attaches"). | [keyframes][keyframes] | missing | No `audio_effects` parameter is animatable: first-wave parameters are static, deferred under #842 (ADR-0178 §1, 0179, 0180, 0183 §5, 0185 §6). | Keyframable effect parameters. |
| Clip pan keyframes | Panner > Balance/Pan keyframes on the clip (PRECEDENT §2). | [pan-track][pan-track] | missing | `pan` not built; ADR-0184 §5 makes it static and defers keyframes under #842. | Built and animatable `pan`. |
| Track keyframes (Volume, Mute, Pan, effect params) | Show Keyframes > Track Keyframes; apply to the whole track. | [gainvol][gainvol] | refused | Track-level audio: #795 court, ADR-0172, ADR-0004 (tracks own nothing). | — |
| Keyframe interpolation on audio | Linear, Bezier, hold etc. on audio keyframes (R). | [keyframes][keyframes] | have | `ease` on every non-first record (ADR-0038), the same vocabulary as picture keyframes. | — |
| Audio keyframe optimisation (thinning) | Preferences: Linear Keyframe Thinning and Minimum Time Interval Thinning for keyframes written by mixer automation. | [kfopt][kfopt] | missing | No recorded automation exists to thin; keyframes are written literally. | (Only meaningful with live automation recording.) |
| Large volume adjustment step | Audio preference for the size of a keyboard volume step (R). | [prefs][prefs] | missing | No GUI. | The interactive editor. |
| Speed with Maintain Audio Pitch | Speed/Duration checkbox: pitch preserved or not (PRECEDENT §13). | [speed][speed] | partial | `speed` runs through a chained `atempo` (`render.rs`, ADR-0020), which always preserves pitch. | The "off" state: a speed change that also shifts pitch. |
| Reverse speed with audio | Speed/Duration > Reverse Speed plays audio backwards (R). | [speed][speed] | refused | Reverse is only a `source_time` remap, and a remapped `video` must carry `volume: 0` (`E-REMAP-AUDIBLE`, ADR-0157). | — |
| Time remapping and audio | Variable speed applies to video; audio is not remapped (PRECEDENT §13, R). | [speed][speed] | have | Same outcome: `source_time` requires a silenced `volume` (ADR-0157). | — |
| Mute (clip) | Clip enable/mute. | [gainvol][gainvol] | have | `volume: 0`; ADR-0055 adds no `mute` field. | — |

### K. Mixers, tracks, submixes, buses

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio Track Mixer: track fader | Per-track volume fader. | [atm][atm] | refused | Track-level audio: #795 court; ADR-0172; C **Master stage** ("not a track and not a bus"). | — |
| Audio Track Mixer: track pan/balance | Per-track pan knob −100..+100. | [mixer-pan][mixer-pan] | refused | Track-level audio (as above). | — |
| Audio Track Mixer: effect slots (inserts) | Up to five effects per track, pre- or post-fader (PRECEDENT, slot count R). | [atm-fx][atm-fx] | refused | "Track-level audio effects" are #795 Out of scope; ADR-0172. | — |
| Pre-fader / post-fader effect placement | An effect slot or send switched pre or post fader. | [atm-fx][atm-fx] | partial | On an element the order is fixed: `audio_effects` always before `volume` (ADR-0169). | A post-fader member position (ADR-0169 chose post-fader `volume` deliberately). |
| Audio Track Mixer: automation modes (Off, Read, Write, Latch, Touch; Safe During Write; Automatch Time) | Moving a control during playback writes track keyframes. | [atm-auto][atm-auto] | refused | Track keyframes are track-level audio (#795; ADR-0172). | — |
| Audio Clip Mixer | Per-clip volume faders and pan knobs at the playhead, with meters. | [acm-auto][acm-auto] | partial | Per-element `volume` (ADR-0055). | Per-element `pan` (ADR-0184 not built); the live mixing surface. |
| Audio Clip Mixer: automation (Latch, Touch) | Moving a clip fader during playback writes clip keyframes. | [acm-auto][acm-auto] | missing | The output, clip `volume` keyframes, is writable by hand or script; nothing records it live. | Live recording of keyframes (no playback surface). |
| Mute / Solo track buttons | Track M and S buttons in the timeline and mixer; Global Mute (26.2). | [atm][atm] | refused | A track carries no audio state (ADR-0004; #795; ADR-0172) and the format has no `mute` field (ADR-0055). | — |
| Submix tracks | Tracks that sum other tracks' output and take their own effects and fader. | [adv-mix][adv-mix] | refused | ADR-0172: the master is "not a general bus with an effect chain"; C **Master stage**: "not a track and not a bus"; track-level audio refused (#795). | — |
| Sends (pre/post fader) to submixes | Per-track sends with their own level and pan. | [adv-mix][adv-mix] | refused | As submixes. | — |
| Mix (master) track effects | Effects on the Mix track in the Audio Track Mixer. | [atm-fx][atm-fx] | refused | ADR-0172 Rejected: "An `audio_effects` list on the master … the stage must be a closed set". | — |
| Mix (master) track volume and loudness | Mix fader; loudness measured or normalised. | [atm][atm] | partial | `master.target_lufs` and `master.ceiling_dbtp` (ADR-0172, 0174). | A static or keyframed mix gain that is not a loudness target. |
| Audio track types: Standard, Mono, 5.1, Adaptive | A track's channel format, set when the track is added (R). | [concepts][concepts] | missing | Tracks carry no channel format (ADR-0004); every source becomes stereo on the bus (ADR-0077). | Channel formats (the per-track part would also meet the track-level refusal). |
| Direct output / track output channel assignment | Audio Track Mixer matrix assigning track channels to master channels (adaptive tracks). | [trackout][trackout] | missing | Stereo bus only. | Multichannel master and routing. |
| Multiple stereo assignments to a multichannel master | One track feeding several stereo pairs (1+2, 3+4). | [multistereo][multistereo] | missing | Stereo bus only. | Multichannel master. |
| Record audio mix changes | Recording mixer moves as automation during playback. | [rec-mix][rec-mix] | refused | Track automation (as above). | — |

### L. Channels, multichannel, sample rate, sync

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Modify > Audio Channels (source channel mapping) | Matrix mapping source channels to clip channels; clip channel format mono/stereo/5.1/adaptive (PRECEDENT §3). | [chmap][chmap] | missing | ADR-0184 `channels` (proposed, not in S: `stereo`, `mono`, `left`, `right`, `swap`) picks among two channels only. | Built field; mapping of more than two source channels. |
| Multichannel master (5.1, adaptive up to 32 channels) and 5.1 mixing | Sequence master track format; 5.1 panning; 5.1-to-stereo mixdown. | [multich][multich] | missing | Bus is 48 kHz stereo (ADR-0077); ADR-0172: "output channel layout … belongs to the channel-operations capability". | Any non-stereo bus or output. |
| 5.1 mixdown type preference | Audio preference choosing how 5.1 folds to stereo (R). | [prefs][prefs] | partial | A 5.1 source is folded to stereo by ffmpeg's default downmix at input (`aformat=…channel_layouts=stereo`, `render.rs`). | A chosen or stated downmix law. |
| Breakout to Mono | Splits a stereo/5.1 project item into per-channel mono items (PRECEDENT §3). | [breakout][breakout] | missing | ADR-0184 `channels: "left"`/`"right"` (proposed) would give the same per-channel use on two elements. | The built field. |
| Sequence sample rate | Sequence audio sample rate chosen in sequence settings (R). | [sr][sr] | partial | Fixed 48 kHz bus (ADR-0077). | A chosen rate (44.1, 96 kHz …). |
| Conform of unsupported sample rates (CFA files) | Uncompressed audio at an unsupported rate is upsampled; compressed formats conform on import; conformed audio cached as CFA. | [conform][conform] | have | Every source is resampled to 48 kHz in the mix graph at render (`aformat=sample_rates=48000`); `probe` reads each source's `sample_rate` and `channels` (`media/probe.rs`). | (No cache file; done per render.) |
| Match Source audio (26.5) | Match Source sets the sequence's audio channels and sample rate from the source, with automatic downmixing. | [relnotes][relnotes] | missing | The bus does not follow the source (fixed 48 kHz stereo). | A source-matched bus. |
| Synchronize by audio waveform | Clip > Synchronize: align clips on their audio waveforms (also In/Out, timecode, markers). | [sync][sync] | missing | No sync tool; detector-driven, so it could only enter as a skill that writes `start` values. | A skill script. |
| Merge Clips | One video synced to up to 16 audio channels into a merged clip (In, Out, timecode, markers). | [merge][merge] | partial | An `audio` element placed at the video's `start` with its own `source_start` gives the same playback (ADR-0001: flat elements). | A linked unit that moves as one (no linking concept, by ADR-0001). |
| Link / unlink audio and video | Linked clips trim and move together; unlink to edit separately (R). | [concepts][concepts] | partial | A `video`'s own sound moves with it (ADR-0055); a separate `audio` element is independent. | Linking of two elements (a `group` shares no timing; ADR-0001). |

### M. Waveforms, scrubbing, monitoring, hardware

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Timeline audio waveforms (peak files) | Waveforms drawn from PEK files in the media cache; "Generate waveforms automatically during import". | [conform][conform] | missing | `frame` contact sheets draw picture only; ADR-0031: the overview "is not required to be spatial". | A waveform reading (image or numbers). |
| Dynamic / live waveforms (26.0) | Waveforms update live as volume, keyframes, ripple and roll change; on by default. | [wave][wave] | missing | As above. | A post-volume waveform reading. |
| Rectified waveforms | Display option for half-wave waveforms (R). | [wave][wave] | missing | No waveform display. | — |
| Source Monitor waveforms (26.2) | Source Monitor shows audio waveforms alongside video. | [whatsnew][whatsnew] | missing | No waveform display. | — |
| Audio scrubbing | Preference "Play audio while scrubbing"; "Maintain pitch while shuttling" (J, K, L). | [prefs][prefs] | partial | `preview` renders any span to an MP4 with its mix (`mcp.rs` `preview`); the master gain is the whole programme's (ADR-0172). | Interactive scrubbing and shuttling. |
| Audio meters | Audio Meters panel and mixer meters: peak, dynamic peaks, valleys (R). | [atm][atm] | partial | `verify` reports integrated loudness, true peak, and per-span silence (EBU R128 momentary against −70 LUFS) of the deliverable (ADR-0172, 0173; `verify.rs`). | A level-over-time reading per element or track. |
| Audio hardware preferences | Default input/output device, ASIO/Core Audio, output mapping, latency. | [hw][hw] | missing | No playback or capture device layer; `render` writes a file. | — |
| Monitor Ambisonics audio | Monitor ambisonic audio binaurally while editing VR. | [ambi][ambi] | missing | Stereo bus only. | Ambisonics. |
| Properties panel: audio controls | The Properties panel shows a selected clip's audio settings to edit. | [props][props] | have | `query --at` prints an element's values at an instant (`mcp.rs` `query`); the agent edits the JSON. | — |

### N. Voice-over recording

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Voice-over Record button on a track | Customize Audio Header adds the microphone button; recording at the playhead or between In/Out with a pre-roll countdown. | [vo][vo] | missing | No capture; a recording enters as an ordinary `audio` element's `source`. | Capture (outside a file-in, file-out editor). |
| Record through the Audio Track Mixer | Track record-enable (R), input channel, Record button; solo the track. | [rec-atm][rec-atm] | missing | As above. | As above. |
| Recording setup (input device, mono/stereo jacks) | Audio Hardware preferences > Default Input. | [rec][rec] | missing | As above. | As above. |

### O. Export

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Audio codec choice (AAC, MPEG, PCM, …) | Export Settings > Audio: format and codec. | [export-ref][export-ref] | partial | AAC is always written (`encode.rs`: `-c:a aac`). | Any other codec. |
| Audio bitrate | kbps for AAC, MPEG and some WMA. | [export-ref][export-ref] | partial | Fixed 160 kbps (`encode.rs`; ADR-0173 §3). | A chosen bitrate. |
| Export sample rate | Export at the recorded rate; 48 kHz recommended for delivery. | [export-ref][export-ref] | partial | Fixed 48 kHz (ADR-0077). | A chosen rate. |
| Export channels (mono, stereo, 5.1) and downmix | Fewer channels than the mix downmixes (5.1→stereo, stereo→mono). | [export-ref][export-ref] | partial | Always stereo. | Mono and 5.1 output. |
| Audio-only export (WAV, AIFF, MP3, AAC) | Export the sequence's audio as an audio file. | [export][export] | missing | `render` writes an MP4 with picture (`encode.rs`). | An audio-only deliverable. |
| Export loudness normalisation | Effects tab: Loudness Normalization to ATSC A/85, EBU R128 or ITU BS.1770-3, optional true-peak limit (PRECEDENT §1). | [ame-loud][ame-loud] | have | `master.target_lufs` (one measured gain) and `master.ceiling_dbtp` (4× oversampled limiter) (ADR-0172, 0174). | (A standard is a literal number, not a dropdown.) |
| Loudness report | Optional XML report of loudness and true peak written with the export. | [ame-loud][ame-loud] | have | `render` reports measured and applied gain; `verify` reports the deliverable's LUFS and dBTP as findings (ADR-0172, 0173). | (Not XML.) |
| Separate audio tracks / stems in export (R) | Export multiple audio tracks or channels to one container. | [export-ref][export-ref] | missing | One mixed stereo track. | Stems or multitrack audio output. |
| Render and replace / Edit in Adobe Audition | Edit Clip in Audition renders a copy to WAV and opens it; round-trip replaces the clip. | [audition][audition] | missing | No round-trip; an externally processed file can be referenced as a new `source`. | A round-trip command. |

### P. Generative and AI audio not covered above

| Premiere feature | Premiere mechanism | Adobe doc URL | Montagent status | Montagent evidence | What's missing |
|---|---|---|---|---|---|
| Generative Extend (audio) | Drag a clip's start or end to generate up to a few seconds more audio (ambience, room tone); audio clips at least 3 s. | [genext][genext] | refused | Model-driven audio: ADR-0145 literal values (#795 Out of scope). | — |
| Beat / tempo detection (music) | Premiere: Remix's song analysis (R: no separate beat-marker feature confirmed). | [remix][remix] | have (skill) | `skills/montagent-craft/scripts/beat_grid.py` detects tempo and downbeats and places the track (marked a workaround for #547). | Tempo and downbeat from `probe` (#547). |

Rows that sit in other area tickets and are not counted here: Speech to Text and transcript-based editing
([auto-transcribe][stt]; the 26.5 Voice Activity Detection model), caption audio checks (ADR-0054), and
media-intelligence search for sound effects (26.2.2 release notes).

## Notable gaps

- **Half the rack is proposed but not built.** Noise reduction (ADR-0181), the noise gate (ADR-0183), pan
  and channel operations (ADR-0184) and echo (ADR-0185) have ADRs but are absent from the v0.2.0 schema. They
  account for more than a dozen `missing` rows, including Premiere's two fixed clip controls, Panner and Channel
  Volume, and the Essential Sound Repair sliders.
- **No effect parameter is animatable.** Premiere keyframes every effect scalar and the clip Panner. In
  Montagent only `volume` moves, and every member is static until #842 decides otherwise.
- **The bus is 48 kHz stereo AAC 160k, fixed.** There is no 5.1, adaptive or multichannel mixing, no channel
  matrix, no chosen sample rate or codec, no audio-only export, and no stems (ADR-0077, ADR-0172).
- **No reverb, pitch shift, modulation, de-ess, distortion or stereo width.** Reverb is explicitly left
  undecided (ADR-0185 owner call 10); the pitch unit is fixed (ADR-0170) but there is no member.
- **EQ is capped at 8 members and Q ≤ 10**, so Graphic EQ (10/20/30 bands), Premiere's full 9-section
  Parametric EQ and narrow notches or DeHum cannot be written exactly (ADR-0179).
- **Peak normalisation is missing.** Montagent normalises loudness (per element and on the master), but has no
  Normalize Max Peak, Normalize All Peaks or pre-effect dB clip gain.
- **No waveform reading of any kind.** The agent cannot see audio the way `frame` lets it see the picture;
  `verify` gives whole-file loudness and peak, and silence spans, only.
- **Music Remix and audio-waveform sync have no tool**, though neither is refused: each could be a skill
  script that writes literal cuts, crossfades or `start` values, as `beat_grid.py` and `duck.py` already do.
- **Refused by design:** track-level audio (faders, track effects, submixes, sends, automation), Essential
  Sound types, tags and presets, AI audio (Enhance Speech, Auto Tag, Generative Extend, DeReverb, adaptive
  noise reduction), convolution reverb, third-party plug-ins and the exponential crossfade curve.

<!-- Premiere / Adobe -->
[lib]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-effects-library.html
[fx-old]: https://helpx.adobe.com/premiere-pro/using/audio-effects-transitions.html
[fx26]: https://helpx.adobe.com/premiere/desktop/add-video-effects/effects-and-transitions-library/list-of-effects-and-transitions.html
[au-shared]: https://helpx.adobe.com/uk/audition/kb/shared-audio-effects.html
[au-amp]: https://helpx.adobe.com/audition/using/amplitude-compression-effects.html
[au-mod]: https://helpx.adobe.com/audition/desktop/effects-reference/modulation-effects.html
[reverb]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/create-reverb-effect.html
[noise-adv]: https://helpx.adobe.com/in_hi/premiere-pro/how-to/advanced-noise-reverberation-reduction.html
[ambi]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-vr-content/monitor-ambisonics-audio.html
[lmeter]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/about-loudness-meter.html
[plugins]: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/plug-ins.html
[pan]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-stereo-track.html
[pan-track]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-track-in-timeline.html
[mixer-pan]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-panning-and-balancing-controls-in-audio-track-mixer.html
[xfade]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/audio-crossfade-transitions.html
[trans]: https://helpx.adobe.com/premiere-pro/using/audio-transitions.html
[trans-default]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/specify-default-audio-transitions.html
[fade]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/fade-in-or-fade-out-clip-audio.html
[whatsnew]: https://helpx.adobe.com/premiere/desktop/whats-new/whats-new.html
[relnotes]: https://helpx.adobe.com/premiere/desktop/whats-new/release-notes.html
[concepts]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/audio-editing-concepts.html
[esp]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-editing-with-essential-sound-panel.html
[tag]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/audio-category-tagging.html
[automatch]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/auto-match-audio-loudness.html
[repair]: https://helpx.adobe.com/uk/premiere/desktop/add-audio-effects/adjust-volume-and-levels/repair-dialogue.html
[clarity]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/improve-dialogue-clarity.html
[enh]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/enhance-speech.html
[remix]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/remix-in-premiere.html
[duck]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/automatically-duck-audio.html
[gain]: https://helpx.adobe.com/premiere-pro/desktop/add-audio-effects/adjust-volume-and-levels/adjust-gain-in-audio.html
[gainvol]: https://helpx.adobe.com/ca/premiere-pro/using/adjusting-volume-levels.html
[keyframes]: https://helpx.adobe.com/premiere-pro-next/add-video-effects/apply-video-effects/keyframes-in-premiere-pro.html
[kfopt]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/specify-automated-keyframe-creation.html
[prefs]: https://helpx.adobe.com/premiere/desktop/get-started/preferences-and-settings/audio-preferences.html
[speed]: https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-using-the-speedduration-option.html
[atm]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/about-audio-track-mixer.html
[atm-fx]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/apply-effects-using-audio-track-mixer.html
[atm-auto]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-track-mixer-automation-modes.html
[acm-auto]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/set-automation-modes-for-audio-clip-mixer.html
[adv-mix]: https://helpx.adobe.com/premiere-pro/using/advanced-mixing.html
[rec-mix]: https://helpx.adobe.com/uk/premiere-pro/using/recording-audio-mixes.html
[trackout]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-an-adaptive-track.html
[multistereo]: https://helpx.adobe.com/sg/premiere/desktop/add-audio-effects/advanced-audio-techniques/assign-multiple-audio-channels.html
[chmap]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/advanced-audio-techniques/change-source-audio-channel-mapping.html
[multich]: https://helpx.adobe.com/ph_fil/premiere-pro/how-to/mix-multichannel-audio.html
[breakout]: https://helpx.adobe.com/premiere-pro-next/add-audio-effects/edit-audio/break-stereo-track-to-mono.html
[sr]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/audio-sample-rates-in-premiere.html
[conform]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/audio-conforming-specifications.html
[sync]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/synchronize-audio-and-video.html
[merge]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/basic-audio-editing/merge-clips-in-the-project-panel.html
[wave]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/dynamic-audio-waveforms.html
[hw]: https://helpx.adobe.com/uk/premiere/desktop/get-started/preferences-and-settings/audio-hardware-preferences.html
[props]: https://helpx.adobe.com/ie/premiere/desktop/add-text-images/stylize-text/about-properties-panel.html
[vo]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-a-voice-over-on-an-audio-track-from-the-timeline.html
[rec-atm]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-audio-using-the-audio-track-mixer.html
[rec]: https://helpx.adobe.com/premiere/desktop/organize-media/import-files/record-audio.html
[export-ref]: https://helpx.adobe.com/premiere-pro/using/export-settings-reference-premiere-pro.html
[export]: https://helpx.adobe.com/premiere/desktop/render-and-export/export-files/export-video-and-audio-files.html
[ame-loud]: https://helpx.adobe.com/media-encoder/desktop/encoding-and-exporting/automatic-loudness-correction.html
[audition]: https://helpx.adobe.com/premiere/desktop/use-premiere-with-other-apps/working-with-other-adobe-applications/edit-premiere-audio-clips-in-audition.html
[genext]: https://helpx.adobe.com/premiere/desktop/edit-projects/edit-with-generative-ai/generative-extend-overview.html
[stt]: https://helpx.adobe.com/premiere/desktop/add-text-images/insert-captions/auto-transcribe-video-using-speech-to-text.html
