# What CapCut and Premiere do, by mechanism, for each audio capability in scope

Research for [#796](https://github.com/MBehtemam/Montagent/issues/796), part of the map
[#795](https://github.com/MBehtemam/Montagent/issues/795).

**Date of research:** 2026-10-07. Nothing was built or run. **This note decides nothing.** It
records precedent so the audio tickets on #795 can cite it.

It follows the method and the honesty rules of the earlier visual-effects precedent study
([#664](https://github.com/MBehtemam/Montagent/issues/664), and its follow-up
`docs/research/skia-reach/named-effects-candidates.md`), which found that CapCut publishes no
reference manual. That finding holds for audio too. See "CapCut's sources" below.

## How this was read, and what that does to the confidence labels

The sandbox this was written in **could not open helpx.adobe.com or capcut.com directly**: the
egress proxy refuses both hosts (`CONNECT tunnel failed, response 403`). Every page below was
reached through a web search engine that returns an extract of the page it indexes. So:

- **CONFIRMED** means: an official page (helpx.adobe.com for Premiere; a capcut.com page for
  CapCut) was located, and the search extract of *that page* states the claim. The URL is the
  page the extract came from. A reader with normal web access should re-open the URL before
  quoting it in an ADR.
- **CONFIRMED (Au)** means the claim is on Adobe's **Audition** page for an effect that Adobe
  states is shared between Audition and Premiere ([Shared audio effects][au-shared]). That
  Premiere exposes the identical parameter set is an inference, as "AE" was in the visual
  study.
- **RECALLED** means the claim is the author's knowledge of the product and was **not**
  re-verified against a page this session. Treat it as a lead, not a citation. Every number in
  a RECALLED cell needs checking before it is relied on.
- **UNCONFIRMED** means no first-party page reached states it. It does not mean the tool lacks
  it.
- **CapCut first-party marketing** means a capcut.com `/resource/`, `/tools/` or `/create/`
  page. These are first-party but are SEO articles, not a reference manual; they contradict
  each other in places (noted where it matters) and never give parameter units. They are
  weaker than a help page.

### CapCut's sources

CapCut's real help centre (`capcut.com/help/...`) exists but is a FAQ: its audio entries are
troubleshooting ("Why is the audio not playing", "Keep original sound", "Recognition says no
audio") ([cc-help-audio][cc-help-audio], [cc-help-keep][cc-help-keep]). No CapCut page reached
documents an audio effect's parameters, ranges or units. Pages under `capcut.com/create/` and
`capcut.com/explore/` read as generated SEO content and are **not** treated as evidence of a
feature even when they name one.

### Where Premiere attaches audio processing (applies to every table)

- **Clip:** effects dragged onto a clip, edited in Effect Controls. Each clip also carries
  fixed effects Volume, Channel Volume and Panner. Parameters in Effect Controls have
  stopwatches and are keyframable ([Pan and balance][pr-pan]; [Keyframes][pr-keyframes]).
- **Track:** up to five effect slots per track in the Audio Track Mixer (RECALLED: the count),
  with track pan/balance and volume automated by Read/Touch/Latch/Write modes
  ([Track mixer pan controls][pr-mixer-pan]).
- **Mix (master):** the Mix track in the Audio Track Mixer takes effects the same way
  (RECALLED), and **export** can loudness-normalise the whole programme
  ([Automatic loudness correction][ame-loudness]).
- **Essential Sound panel** is not a fourth place: it tags clips by type (Dialogue, Music,
  SFX, Ambience) and writes ordinary clip effects/keyframes underneath (e.g. DeNoise, DeReverb,
  the ducking keyframes) ([Advanced noise and reverb][pr-noise-adv]; [Auto ducking][pr-duck]).

### Where CapCut attaches audio processing (applies to every table)

CapCut (desktop) shows audio controls in a right-hand panel for the **selected clip**, in tabs
named **Basic** (volume, fade in/out, loudness normalisation, enhance voice, reduce noise),
**Voice changer**, and **Speed** (CapCut first-party marketing: [Voice changers][cc-vc],
[Normalize (Premiere article)][cc-norm-pr], [Voice enhancer][cc-enhance]; fade location from
third-party guides only). No track-level or master/mix audio processing is documented anywhere
first-party. Volume keyframing is widely described but only on third-party pages:
UNCONFIRMED.

---

## 1. Loudness normalisation

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes, three mechanisms. CONFIRMED. | Yes, a "Loudness normalization" option. CapCut first-party marketing. |
| Mechanisms | (a) **Essential Sound > Loudness > Auto-Match**: analyses the clip and sets its gain to a target ([Auto-match loudness][pr-automatch]). (b) **Audio Gain > Normalize Max Peak to / Normalize All Peaks to**: peak, not loudness ([Adjust gain][pr-gain]). (c) **Export loudness normalisation** to ITU BS.1770-3, EBU R128 or ATSC A/85 with an optional true-peak limiter ([Automatic loudness correction][ame-loudness]). | A checkbox in Basic that normalises the clip ([Normalize (Premiere article)][cc-norm-pr]; [Loudness Normalization tool][cc-loud-tool]). |
| Attaches to | (a) clip; (b) clip (selected clips, gain is a non-keyframed pre-effect clip property); (c) whole mix at export. | Clip. |
| Parameters | (a) Target per type: Dialogue **−23 LUFS** (CONFIRMED on the Adobe page extract); Music −25, SFX −21, Ambience −30 LUFS come from a LinkedIn Learning transcript, not Adobe: UNCONFIRMED. Measured clip loudness shown in LUFS under the button ([Auto-match loudness][pr-automatch]). (b) Target in **dB**, default 0.0 dB, must be ≤ 0 dB. *Max Peak* applies one gain to all selected clips (keeps relative levels); *All Peaks* gains each clip separately ([Adjust gain][pr-gain]). (c) Standard choice; Target Loudness in LUFS editable only for ITU BS.1770-3 (fixed for R128 = −23 LUFS, A/85 = −24 LKFS, RECALLED values); true-peak limit in dBTP (RECALLED) ([Automatic loudness correction][ame-loudness]). | Target **−23 LUFS**, fixed (stated by capcut.com articles; no user target found). No other parameter documented. |
| Keyframable | No: (a) and (b) are one-shot gain changes; (c) is an export setting. | No (it is a checkbox). UNCONFIRMED. |
| Presets | n/a | n/a |

**Verdict: precedent confirmed in Premiere (clip auto-match to a LUFS target, peak normalise in dB, and programme-level export normalisation); CapCut first-party marketing only, fixed −23 LUFS per clip.**

## 2. Pan / balance

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | No first-party page names pan or balance. UNCONFIRMED. |
| Mechanism | Clip fixed effect **Panner > Balance** (stereo) / Pan (mono); track pan/balance knob in the Audio Track Mixer; also a **Balance** effect ([Pan and balance][pr-pan]; [Pan or balance a track][pr-pan-track]; [Track mixer pan][pr-mixer-pan]; [Effects library][pr-lib]). | — |
| Attaches to | Clip (Panner, Balance effect) and track (mixer knob). | — |
| Parameters | **−100 to +100**, unitless; negative favours left, 0 centre (CONFIRMED, Adobe's Audio Track Mixer page: "type a new percentage between -100 and 100"; [Pan and balance][pr-pan]). A stereo track's balance "determines how much of each input channel is sent to the output channels" (CONFIRMED, Adobe text); Adobe does not say whether it moves content, so the repo's measurement decides (see [pan-channels README](pan-channels/README.md) §7). Pan law: **not documented** on any Adobe page read (2026-10-08). | — |
| Keyframable | Yes: Adobe's timeline page says to select Track:Volume, then Panner > Balance or Panner > Pan, and use the Add/Remove Keyframe icon (CONFIRMED, Adobe text, 2026-10-08); track pan via automation modes or track keyframes ([Pan and balance][pr-pan]; [Pan or balance a track][pr-pan-track]). | — |

**Verdict: precedent confirmed in Premiere only.**

## 3. Channel operations (mono, swap, one channel only)

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | UNCONFIRMED; nothing first-party found. |
| Mechanisms | Effects **Fill Left with Right**, **Fill Right with Left**, **Swap Channels** (stereo clips only), **Invert** (polarity) and **Channel Volume** ([Audio effects and transitions][pr-fx-old]; [Effects library][pr-lib]). **Modify > Audio Channels** remaps which source channels feed the clip and its channel format (mono/stereo/5.1/adaptive) ([Clips, channels, tracks][pr-channels]). **Breakout to Mono** splits a stereo/5.1 project item into per-channel mono clips named e.g. "Zoom Left", "Zoom Right" ([Breakout to mono][pr-breakout]). | — |
| Attaches to | Fill/Swap/Invert/Channel Volume: clip or track effect. Audio Channels: clip (or project item before editing). Breakout: project item, not timeline clip ([Breakout to mono][pr-breakout]). | — |
| Parameters | Fill/Swap/Invert: **none** (presence is the operation). Channel Volume: per-channel level in **dB** ([Effects library][pr-lib]). Audio Channels: a source-channel × clip-channel mapping matrix (RECALLED form). | — |
| Keyframable | Fill/Swap/Invert have no parameters; Channel Volume yes (it is a fixed clip effect). Channel mapping: no. | — |

Note: Adobe's own text, read on its Audio effects library page (2026-10-08, CONFIRMED), says Fill Left with
Right "duplicates the left channel information of the audio clip and places it in the right channel,
discarding the original clip's right channel information". That is the opposite of the name. Adobe's text
settles what Adobe says, not what the effect does, so the direction is still resolved by testing (the
pan-channels README's measurements), not by quoting. Swap Channels ("switches the placement of the left and
right channel information", stereo clips only), Invert and Channel Volume (per-channel dB) are quoted from the
same page.

**Verdict: precedent confirmed in Premiere only.**

## 4. Audio crossfades and audio across video transitions

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | Fades yes (third-party guides only); crossfade mechanism UNCONFIRMED. |
| Mechanism | Audio **transitions** placed on an edit between two adjacent clips on one audio track: **Constant Power** (default, "similar to a video dissolve"), **Constant Gain** (perceived dip), **Exponential Fade** ([Audio transitions][pr-trans]; [Default audio transitions][pr-trans-default]). A transition on only one clip end is a fade in/out. | Per-clip **Fade in / Fade out** durations in the Basic tab (third-party: [hollyland fade guide][x-cc-fade]). No first-party page describes overlapping audio crossfades or whether a video transition moves audio. |
| Audio vs video transitions | **Independent objects.** Shift+D applies the default video and audio transitions together; Ctrl/Cmd+D video only; Ctrl/Cmd+Shift+D audio only ([Audio transitions][pr-trans]). A video transition does not by itself crossfade the audio. | UNCONFIRMED. |
| Attaches to | The edit point on an audio track (a transition object, not a clip effect). | Clip ends. |
| Parameters | Curve = the choice of transition (no shape parameter); **duration** (default set in Preferences > Timeline > Audio Transition Default Duration; RECALLED default 1 s) ([Default audio transitions][pr-trans-default]); alignment Center/Start/End at cut (RECALLED). | Fade durations in **seconds** (third-party). |
| Keyframable | No; a transition has a duration, not keyframes. | No. |

**Verdict: precedent confirmed in Premiere (three named curves, audio transitions separate from video ones); CapCut neither confirmed beyond per-clip fades, which are third-party only.**

## 5. EQ (high/low-pass, shelf, parametric, presets)

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | **UNCONFIRMED.** Only `capcut.com/explore` pages (SEO, user-content style) and third-party blogs mention an "equalizer" with presets ("Vocal Clarity", "Bass Boost"…); no first-party feature page found. |
| Mechanisms | **Parametric Equalizer** (multi-band: per-band frequency, gain, Q/bandwidth, plus HP/LP and shelves) ([Improve audio quality][pr-improve]; [Effects library][pr-lib]); **Simple Parametric EQ**, **Highpass**, **Lowpass**, **Bandpass**, **Bass**, **Treble** (RECALLED names, in the library); **Graphic Equalizer** at 1-octave (10 bands), ½-octave (20) or ⅓-octave (30) spacing, CONFIRMED (Au) ([Filter and EQ effects][au-eq]); **Notch Filter**, up to six bands, CONFIRMED (Au) ([Filter and EQ effects][au-eq]). Essential Sound **Dialogue > Enhance > EQ** with a preset dropdown (RECALLED). | — |
| Attaches to | Clip, track or mix. | — |
| Parameters | Frequency in **Hz**, gain in **dB**, width as **Q** (CONFIRMED in kind: [Improve audio quality][pr-improve]). Ranges (RECALLED, from Audition's Parametric EQ): bands 20 Hz–20 kHz, gain ±30 dB, Q 0.1–100, HP/LP slope 6–48 dB/octave. Highpass/Lowpass: single **Cutoff (Hz)** (RECALLED). | — |
| Keyframable | Single-value effects (Highpass, Lowpass, Bass, Treble, Simple Parametric) yes. Parametric/Graphic EQ: individual band parameters exposed in Effect Controls are keyframable (RECALLED; the curve editor itself is a "Custom Setup" that is not). | — |
| Presets | Parametric Equalizer and Essential Sound EQ ship named presets that are stored settings of the same Parametric Equalizer (RECALLED; preset names not verified). | Composition unknown. |

**Verdict: precedent confirmed in Premiere only (CapCut EQ unconfirmed).**

## 6. Compressor / limiter / noise gate

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | **No first-party mention found.** UNCONFIRMED. |
| Mechanisms | **Dynamics** with four sections, **AutoGate**, **Compressor**, **Expander**, **Limiter** (brick-wall) ([Effects library][pr-lib]; legacy, still listed). **Hard Limiter**, **Dynamics Processing**, **Multiband Compressor**, **Single-band Compressor**, **Tube-modeled Compressor**, all shared with Audition ([Shared audio effects][au-shared]). Essential Sound Dialogue/Music "Dynamics" slider (RECALLED). | — |
| Attaches to | Clip, track or mix. | — |
| Parameters | Gate: threshold, attack, hold, release. Compressor: threshold, ratio, attack, release, make-up. Expander: threshold, ratio. Limiter: threshold, release ([Effects library][pr-lib], kinds only). Units (RECALLED, Audition): threshold in **dBFS**, ratio **n:1** (up to 30:1 on Single-band), attack/release in **ms**; Hard Limiter: Maximum Amplitude **dB**, Input Boost dB, Look-ahead **ms**, Release **ms** ([Amplitude and compression][au-amp], CONFIRMED (Au) for the effect, values RECALLED). | — |
| Keyframable | Scalar parameters yes in Effect Controls; Multiband/Dynamics Processing curves no (RECALLED). | — |

**Verdict: precedent confirmed in Premiere only.**

## 7. Noise reduction

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | Yes: **Reduce noise** (Basic) and **Enhance voice**. CapCut first-party marketing. |
| Mechanisms | **DeNoise** (broadband), **DeReverb**, **DeHummer**, **Adaptive Noise Reduction**, **Automatic Click Remover**; Essential Sound **Repair** sliders (Reduce Noise, Reduce Rumble, DeHum, DeEss, Reduce Reverb) which apply DeNoise/DeReverb underneath; **Enhance Speech** (AI) ([Advanced noise and reverb][pr-noise-adv]; [Effects library][pr-lib]). | **Reduce noise**: a toggle that "automatically removes hums, static, or other background noises". **Enhance voice**: checkbox plus **Intensity 0–100** that removes noise, clicks, echo, pops ([Voice enhancer][cc-enhance]; [Normalize (Premiere article)][cc-norm-pr]). |
| Attaches to | Clip (Essential Sound); clip/track/mix (effects). | Clip. |
| Parameters | DeNoise: Amount **%** (RECALLED). Essential Sound sliders 0–10 (RECALLED). DeHummer: 50/60 Hz base + harmonics (RECALLED). Enhance Speech: Mix amount (RECALLED). | Reduce noise: none (on/off). Enhance voice: Intensity, unitless 0–100. |
| Keyframable | DeNoise Amount yes (RECALLED); Enhance Speech no (RECALLED: it is a render pass). | No. UNCONFIRMED. |

**Verdict: precedent confirmed in Premiere; CapCut has it on first-party marketing only (on/off and a 0–100 intensity, AI, undisclosed internals).**

## 8. De-ess

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | UNCONFIRMED (no separate control; "Enhance voice" may cover it, not stated). |
| Mechanism | **DeEsser** effect; also Essential Sound Repair > DeEss slider (RECALLED) ([Effects library][pr-lib]). | — |
| Attaches to | Clip, track or mix. | — |
| Parameters | **Mode**: Broadband (compress all frequencies) or Multiband (only the sibilance range); **Threshold** (amplitude above which compression occurs, dB RECALLED); **Center Frequency** (Hz); **Bandwidth** (Hz); **Output Sibilance Only** (monitor toggle); Gain Reduction meter ([Effects library][pr-lib]). | — |
| Keyframable | Scalar parameters yes (RECALLED). | — |

**Verdict: precedent confirmed in Premiere only.**

## 9. Reverb

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | Only as voice-filter presets (e.g. "Echo"; reverb-like presets on third-party lists). UNCONFIRMED as a parametric reverb. |
| Mechanisms | **Studio Reverb** (algorithmic, "faster and less processor-intensive … because it isn't convolution-based"), **Convolution Reverb** (impulse-response file), **Surround Reverb** ([Effects library][pr-lib]); Essential Sound Creative > Reverb with a **preset** dropdown ([Essential Sound (Audition)][au-esp]). | Voice changer > Voice filters (see §12). |
| Attaches to | Clip, track or mix. | Clip. |
| Parameters | Studio Reverb: Room Size, Decay (**ms**), Early Reflections, Width, High/Low Frequency Cut (**Hz**), Damping, Diffusion, Dry/Wet (**%**) (from a third-party guide and RECALLED; Adobe page extract confirmed only the effect). Convolution: impulse, mix, room size %, damping, pre-delay ms, gain dB (RECALLED). | None documented. |
| Keyframable | Scalars yes (RECALLED). | UNCONFIRMED. |
| Presets | Essential Sound reverb presets are stored Studio Reverb settings (RECALLED). | Composition undisclosed. |

**Verdict: precedent confirmed in Premiere; CapCut only as opaque presets (unconfirmed).**

## 10. Echo / delay

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | "Echo" is named as a voice-filter preset on capcut.com marketing pages ([Voice filters][cc-filters]). No parametric delay. |
| Mechanisms | **Delay**, **Analog Delay** ("simulates the sonic warmth of vintage hardware delay units"), **Multitap Delay** ([Effects library][pr-lib]). | Preset only. |
| Attaches to | Clip, track or mix. | Clip. |
| Parameters | Delay time (**s/ms**; Delay effect up to 2 s, RECALLED), Feedback (**%**), Mix/Dry-Wet (**%**) (kinds from search extract; units RECALLED). Analog Delay adds Trash (saturation) and Spread (RECALLED). | None. |
| Keyframable | Scalars yes (RECALLED). | UNCONFIRMED. |

**Verdict: precedent confirmed in Premiere; CapCut only as an opaque "Echo" preset.**

## 11. Pitch shift

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | Yes, inside Voice changer: "Pitch" and "Timbre" adjustments. CapCut first-party marketing, weak ([Voice changers][cc-vc]; [Pitch changer tool][cc-pitch-tool]). |
| Mechanism | **Pitch Shifter** effect, time-preserving ([Effects library][pr-lib]; [Time and pitch (Au)][au-pitch]). | Sliders on the selected voice filter (third-party descriptions; no first-party parameter page). |
| Attaches to | Clip, track or mix. | Clip. |
| Parameters | **Semitones** (0 = original; ±12 = one octave), **Cents** −100…+100, **Ratio** 0.5…2.0 (frequency ratio), plus Formant preserve and Precision/Splicing options (latter RECALLED) ([Effects library][pr-lib]). | Units and ranges undisclosed. |
| Keyframable | Yes for the scalars (RECALLED). | UNCONFIRMED. |

**Verdict: precedent confirmed in Premiere (semitones/cents/ratio); CapCut has a pitch control of undisclosed units.**

## 12. Voice changers

| | Premiere | CapCut |
|---|---|---|
| Exists | **No named voice-changer mechanism.** Pitch Shifter ships named presets that serve that purpose (RECALLED names such as "Angry Gerbil", "Dark Ruler"); Essential Sound has creative presets. | Yes. CapCut first-party marketing ([Voice changers][cc-vc]; [Voice filters][cc-filters]; [How to change your voice][cc-change-voice]). |
| Mechanism | A preset = a saved parameter set of one ordinary effect (Pitch Shifter, Parametric EQ, Studio Reverb). Composition is fully visible: applying it fills the effect's parameters. | Right panel **Voice changer** with three families: **Voice filters** (presets such as Robot, Chipmunk, Deep, High, Elf, Echo, Gender swap…; "over 100" claimed), **Voice characters** / AI characters (male, female, child, aged, cartoon, eerie), and singing/"speech to song" voices. Some AI options need "Preview 5s" then **Generate**, i.e. they render a new audio asset server-side ([Voice filters][cc-filters]). |
| Attaches to | Clip, track or mix (as the underlying effect). | Clip. |
| Parameters | Those of the underlying effect. | Per-filter sliders described as pitch / timbre (and on some, intensity); units undisclosed. |
| Keyframable | As the underlying effect. | UNCONFIRMED (generated voices cannot be, being rendered audio). |
| What a preset is made of | Discoverable: a parameter set of one effect. | **Not discoverable.** No page states whether a filter is a DSP chain (pitch + EQ + reverb/delay) or a neural voice conversion; the "Generate" step indicates the character voices are model-generated. |

**Verdict: neither as a named DSP mechanism. Premiere's "voice changer" is presets over Pitch Shifter etc. (composition visible); CapCut's is a closed preset library (some generative) with undisclosed composition.**

## 13. Pitch preservation under speed change

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | Yes, a pitch toggle in the Speed panel. CapCut first-party marketing ([Pitch changer tool][cc-pitch-tool]; [Music speed changer][cc-speed-tool]). |
| Mechanism | **Clip > Speed/Duration > "Maintain Audio Pitch"** checkbox; also the Rate Stretch tool (RECALLED that it honours the setting) ([Speed/Duration][pr-speed]). Time Remapping (variable speed) applies to video only; audio is not remapped (RECALLED). | Speed tab, a toggle labelled differently across sources ("Pitch", "Change audio pitch", "Keep pitch"), and capcut.com pages contradict each other on which state preserves pitch. Polarity: UNCONFIRMED. Curve speed: pitch behaviour UNCONFIRMED. |
| Attaches to | Clip. | Clip. |
| Parameters | Boolean. Speed in **%** (100 = normal) ([Speed/Duration][pr-speed]). | Boolean; speed as a multiplier (0.1×–100× RECALLED). |
| Keyframable | No (a clip property). | No. |

**Verdict: precedent confirmed in Premiere (a per-clip boolean); CapCut has a per-clip toggle on first-party marketing, polarity unclear.**

## 14. Auto-ducking

| | Premiere | CapCut |
|---|---|---|
| Exists | Yes. CONFIRMED. | **UNCONFIRMED.** Only `capcut.com/create/…` SEO pages, which themselves say mobile "lists" ducking and that desktop/web equivalents "are not confirmed" ([Ducking (create page)][cc-duck-create]). |
| Mechanism | **Essential Sound > Music (or SFX/Ambience) > Ducking**: choose what to **Duck Against** (Dialogue, Music, SFX, Ambience, untagged), set parameters, press **Generate Keyframes**. The output is **ordinary volume keyframes** on the ducked clips (the Adobe extract says the keyframes are written to an added Amplify effect's gain; RECALLED versions write clip Volume keyframes, so check) ([Auto ducking][pr-duck]; [Automatic audio ducking][pr-duck-howto]). It is a one-shot generator, not a live side-chain. | — |
| Attaches to | The ducked clips (Music/SFX/Ambience type); key is the tagged clips on other tracks. | — |
| Parameters | **Sensitivity** (trigger threshold, unitless slider), **Duck Amount** in **dB**, **Fades** in **ms** ([Auto ducking][pr-duck]; [Adobe Learn: ducking][pr-duck-learn]). Ranges not documented on pages reached. | — |
| Keyframable | Its output *is* keyframes, editable afterwards; the generator settings are not. | — |

**Verdict: precedent confirmed in Premiere (generator that writes volume keyframes); CapCut neither confirmed.**

---

## Summary

| Capability | Premiere | CapCut | Verdict |
|---|---|---|---|
| Loudness normalisation | CONFIRMED: clip Auto-Match (−23 LUFS dialogue), peak normalise (dB), export R128/A85/BS.1770 | Marketing: per-clip checkbox, fixed −23 LUFS | Both (CapCut weak) |
| Pan / balance | CONFIRMED: −100…+100, clip + track, keyframable | Not found | Premiere |
| Channel ops | CONFIRMED: Fill L/R, Swap, Invert, Channel Volume dB, Audio Channels map, Breakout to Mono | Not found | Premiere |
| Crossfades | CONFIRMED: Constant Power/Gain/Exponential; audio and video transitions independent | Fades (third-party only) | Premiere |
| EQ | CONFIRMED: Parametric (Hz/dB/Q), HP/LP, Graphic 10/20/30, Notch ×6 | Not confirmed | Premiere |
| Compressor/limiter/gate | CONFIRMED: Dynamics (gate/comp/exp/limit), Hard Limiter, multiband etc. | Not found | Premiere |
| Noise reduction | CONFIRMED: DeNoise, DeHum, DeReverb, Enhance Speech | Marketing: Reduce noise (on/off), Enhance voice (0–100) | Both (CapCut weak, AI, opaque) |
| De-ess | CONFIRMED: DeEsser (mode, threshold, centre Hz, bandwidth) | Not found | Premiere |
| Reverb | CONFIRMED: Studio, Convolution, Surround | Preset-only, unconfirmed | Premiere |
| Echo / delay | CONFIRMED: Delay, Analog, Multitap | "Echo" preset only | Premiere |
| Pitch shift | CONFIRMED: semitones ±, cents ±100, ratio 0.5–2 | Marketing: pitch/timbre sliders, no units | Premiere (CapCut weak) |
| Voice changers | No named mechanism; presets over Pitch Shifter etc. | Marketing: closed preset library, some generative | Neither as a DSP mechanism |
| Pitch preservation | CONFIRMED: Speed/Duration "Maintain Audio Pitch" boolean | Marketing: Speed-panel toggle, polarity unclear | Both (CapCut weak) |
| Auto-ducking | CONFIRMED: Essential Sound generator → volume keyframes (Duck Against, Sensitivity, Amount dB, Fades ms) | Not confirmed | Premiere |

Two cross-cutting observations, stated as observations, not recommendations:

- **Premiere expresses every capability as a named effect with typed scalar parameters, at
  clip, track or mix scope**, and keyframes those scalars. Presets are saved parameter sets of
  one effect. That matches #664's finding for visual effects.
- **Premiere's "smart" audio features are generators that write ordinary state** (Auto-Match
  writes clip gain; ducking writes volume keyframes; Essential Sound sliders write DeNoise /
  DeReverb parameters). The document after the button press is plain effects and keyframes.
  CapCut's equivalents (normalise, reduce noise, enhance voice, character voices) are opaque
  per-clip switches or server-generated audio with no published internals.

## Open items a reader with web access should close

- Re-open each helpx URL and replace every RECALLED value with a quoted one (EQ ranges,
  compressor units, delay maximum, export targets, transition default duration, track slot
  count).
- Settle Adobe's Fill Left with Right wording and the ducking target (Amplify vs Volume) by
  test.
- Settle CapCut's speed/pitch toggle polarity by test in the current desktop build.

<!-- Premiere / Adobe -->
[pr-lib]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-effects-library.html
[pr-fx-old]: https://helpx.adobe.com/premiere-pro/using/audio-effects-transitions.html
[pr-automatch]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/auto-match-audio-loudness.html
[pr-gain]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/adjust-gain-in-audio.html
[ame-loudness]: https://helpx.adobe.com/media-encoder/using/automatic-loudness-correction.html
[pr-pan]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-stereo-track.html
[pr-pan-track]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/pan-or-balance-a-track-in-timeline.html
[pr-mixer-pan]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-effects/audio-panning-and-balancing-controls-in-audio-track-mixer.html
[pr-keyframes]: https://helpx.adobe.com/premiere-pro-next/add-video-effects/apply-video-effects/keyframes-in-premiere-pro.html
[pr-channels]: https://helpx.adobe.com/si/prelude/using/clips-channels-tracks.html
[pr-breakout]: https://helpx.adobe.com/premiere-pro-next/add-audio-effects/edit-audio/break-stereo-track-to-mono.html
[pr-trans]: https://helpx.adobe.com/premiere-pro/using/audio-transitions.html
[pr-trans-default]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/apply-audio-transitions/specify-default-audio-transitions.html
[pr-improve]: https://helpx.adobe.com/si/premiere-pro/how-to/improve-audio-quality.html
[pr-noise-adv]: https://helpx.adobe.com/in_hi/premiere-pro/how-to/advanced-noise-reverberation-reduction.html
[pr-speed]: https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-speed/change-clip-speed-using-the-speedduration-option.html
[pr-duck]: https://helpx.adobe.com/premiere/desktop/add-audio-effects/adjust-volume-and-levels/automatically-duck-audio.html
[pr-duck-howto]: https://helpx.adobe.com/ph_fil/premiere-pro/how-to/automatic-audio-ducking.html
[pr-duck-learn]: https://www.adobe.com/learn/premiere-pro/web/automatic-audio-ducking
[au-shared]: https://helpx.adobe.com/ro/audition/kb/shared-audio-effects.html
[au-eq]: https://helpx.adobe.com/audition/desktop/effects-reference/filter-equalizer-effects.html
[au-amp]: https://helpx.adobe.com/audition/using/amplitude-compression-effects.html
[au-pitch]: https://helpx.adobe.com/audition/using/time-pitch-manipulation-effects.html
[au-esp]: https://helpx.adobe.com/audition/desktop/editing-audio-files/essential-sound-panel.html

<!-- CapCut -->
[cc-help-audio]: https://www.capcut.com/help/audio-not-playing
[cc-help-keep]: https://www.capcut.com/help/keep-original-sound
[cc-norm-pr]: https://www.capcut.com/resource/normalize-audio-premiere
[cc-loud-tool]: https://www.capcut.com/tools/loudness-normalization
[cc-enhance]: https://www.capcut.com/tools/voice-enhancer
[cc-vc]: https://www.capcut.com/resource/audio-voice-changer
[cc-filters]: https://www.capcut.com/tools/voice-filters
[cc-change-voice]: https://www.capcut.com/resource/how-to-change-your-voice
[cc-pitch-tool]: https://www.capcut.com/tools/pitch-changer
[cc-speed-tool]: https://www.capcut.com/tools/music-speed-changer
[cc-duck-create]: https://www.capcut.com/create/audio-ducking-in-capcut-for-clear-commentary

<!-- Third party (used only where marked) -->
[x-cc-fade]: https://store.hollyland.com/blogs/creator-hub/fade-out-audio-in-capcut
