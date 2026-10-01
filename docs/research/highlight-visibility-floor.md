# Research: is there a citable minimum on-screen time for a word highlight to be seen?

Research for [#553](https://github.com/MBehtemam/Montagent/issues/553), raised by [#542](https://github.com/MBehtemam/Montagent/issues/542).

**Date of research:** 2026-10-01.

**Sourcing rule applied:** every number below is quoted from the primary source: the paper itself (or its indexed abstract on PubMed), or the style guide or standard itself. Blog posts and vendor pages were found and **not** used. ADR-0061 requires a borrowed threshold to be cited at its source, so a number appears here only when the source states it.

**What counts:** a measurement of *seeing an emphasis change* (colour, weight or highlight) on a word that is *already on screen*. Reading time, subtitle display time and words-per-minute rates measure something else and are listed only to rule them out.

---

## 1. Verdict

**No citable source.**

No published paper, broadcast standard or official style guide that I found states a minimum time, in ms or in frames, that a highlight or colour change on an already-visible word must hold to be seen. The vision research that comes closest measures other things:

- **detection thresholds at low contrast.** These are critical durations: the window over which the eye trades intensity against time.
- **the role of attention.** Change blindness shows that attention, not duration, decides whether a change is noticed.

Neither one is a floor for a clearly visible highlight. The one rapid-presentation result that fits a "frames" framing points the other way: a single 13 ms frame can register (§2.3).

The proposed "too few frames to be seen" check therefore has no number it can cite, and should be dropped for that reason. `R-HIGHLIGHT-UNPAINTED` (a window holding **zero** painted frames, from #542) needs no perceptual threshold, so it is unaffected.

---

## 2. Candidates from vision research

### 2.1 Chromatic critical duration: about 200 ms (fovea) to about 100 ms (20 degrees)

- **Number:** "Critical duration decreased by a factor of 2 (from approximately 200 to approximately 100 ms) from the fovea to 20 degrees eccentricity." The abstract also says: "Foveal chromatic mechanisms integrate over several hundred milliseconds for pulse detection."
- **What was measured:** temporal integration, under Bloch's law, for detecting a *threshold-level* chromatic pulse. Below the critical duration, a shorter pulse needs proportionally more contrast to be detected. It is **not** a minimum time for seeing a suprathreshold colour change.
- **Conditions:** psychophysical pulse detection of isoluminant chromatic stimuli in the lab, as a function of retinal eccentricity. Not text, not video, no frame rate.
- **Source:** Swanson, Pan & Lee (2008), *Chromatic temporal integration and retinal eccentricity: psychophysics, neurometric analysis and cortical pooling*, Vision Research 48(26):2657–62. [doi:10.1016/j.visres.2008.03.002](https://doi.org/10.1016/j.visres.2008.03.002) · [PMC2613683](https://pmc.ncbi.nlm.nih.gov/articles/PMC2613683/)
- **Why it does not qualify:** a critical duration says where the trade between time and contrast stops. A highlight drawn at full contrast is far above threshold, so the figure gives no minimum for it. Using "200 ms" as a floor would misread the measurement.

### 2.2 Pulse-detection thresholds, chromatic and luminance (5 to 2560 ms pulses)

- **Number:** the abstract states no single threshold. It reports thresholds across pulse durations "from 5 to 2560 msec", fitted by a peak-detector model.
- **What was measured:** contrast thresholds for detecting chromatic and luminance pulses on a 600 nm field, plus temporal modulation sensitivity from 0.25 to 40 Hz.
- **Conditions:** 0.9 to 900 Td mean luminance; 0.5 to 8 degree fields; lab optics, not a display.
- **Source:** Swanson, Ueno, Smith & Pokorny (1987), *Temporal modulation sensitivity and pulse-detection thresholds for chromatic and luminance perturbations*, J. Opt. Soc. Am. A 4(10):1992–2005. [doi:10.1364/josaa.4.001992](https://doi.org/10.1364/josaa.4.001992) · [PubMed 3430210](https://pubmed.ncbi.nlm.nih.gov/3430210/)
- **Why it does not qualify:** same as §2.1. It describes how contrast and duration trade near threshold, not a duration floor for a visible change.

### 2.3 A single 13 ms frame can register

- **Number:** pictures "presented at between 13 and 80 ms per picture, with no interstimulus interval". Performance was "significantly above chance at all durations", including 13 ms.
- **What was measured:** detecting a named picture inside a rapid stream of pictures. This is gist recognition of whole scenes, not a change on a word.
- **Conditions:** sequences of six or 12 pictures, each followed at once by the next. Only the abstract was read (the full text is paywalled), so the display refresh rate is not recorded here.
- **Source:** Potter, Wyble, Hagmann & McCourt (2014), *Detecting meaning in RSVP at 13 ms per picture*, Attention, Perception, & Psychophysics 76(2):270–279. [doi:10.3758/s13414-013-0605-z](https://doi.org/10.3758/s13414-013-0605-z) · [PubMed 24374558](https://pubmed.ncbi.nlm.nih.gov/24374558/)
- **Why it does not qualify:** it measures something different. If anything, it shows that one frame of a high-contrast image is enough for the visual system to register it. It gives no "too few frames" floor; it argues against having one.

### 2.4 Change blindness: attention, not duration, decides whether a change is seen

- **Numbers (stimulus parameters, not thresholds):**
  - **Flicker paradigm:** "Each image was displayed for 240 ms and each blank for 80 ms." With the blanks in place, large changes took many alternations to find. With the blanks removed, "identification required only 1.4 alternations (0.9 s) on average".
  - **Earlier work cited in the paper** (Phillips 1974; Pashler 1988): "Observers were found to be poor at detecting change if old and new displays were separated by an ISI of more than 60 to 70 ms."
- **What was measured:** time to find and name a change between two versions of a photograph that alternate.
- **Conditions:** real-world photographs; changes in colour, presence or position; changes of "central" vs "marginal" interest.
- **Source:** Rensink, O'Regan & Clark (1997), *To see or not to see: The need for attention to perceive changes in scenes*, Psychological Science 8:368–373. [doi:10.1111/j.1467-9280.1997.tb00427.x](https://doi.org/10.1111/j.1467-9280.1997.tb00427.x) · [author PDF](https://www2.psych.ubc.ca/~rensink/publications/download/PsychSci97-RR.pdf)
- **What it does establish:** a change is normally seen because its local motion transient draws attention to it: "attention is normally drawn to any change in a scene". Changes are missed "when low-level transients are masked". A highlight on a static line is a local transient, so by this account it is noticed *unless* something else masks it at the same moment, such as a cut, a whole-frame change, or many words changing at once.
- **Why it does not qualify:** neither number is a minimum hold time. The 60–70 ms figure is about the **gap** between two displays, not how long the changed state lasts. The 240 ms figure is a chosen stimulus duration, not a measured threshold.

### 2.5 Transient attention peaks 70–150 ms after a cue

- **Number:** "peak discrimination performance occurred if the cue preceded the target array by 70-150 msec."
- **What was measured:** how a sudden cue improves discrimination at the cued location, as a function of the cue-to-target delay (SOA).
- **Conditions:** search arrays of oriented black and white bars; central fixation.
- **Source:** Nakayama & Mackeben (1989), *Sustained and transient components of focal visual attention*, Vision Research 29(11):1631–47. [doi:10.1016/0042-6989(89)90144-2](https://doi.org/10.1016/0042-6989(89)90144-2)
- **Why it does not qualify:** this is the time course of attention *after* a cue. It is not the time the cue itself must stay on screen to be seen.

---

## 3. Candidates from broadcast and caption standards

None of these gives a time for a highlight change. Each figure here governs a whole subtitle event or a reading rate, which the issue rules out.

| Source | Number | What it governs |
|---|---|---|
| [BBC Subtitle Guidelines v1.2.5 (March 2026)](https://www.bbc.co.uk/accessibility/forproducts/guides/subtitles/), §4.1 | "a minimum period of around 0.3 seconds per word (e.g. 1.2 seconds for a 4-word subtitle)", from "the recommended rate of 160-180 words per minute" | Reading time for a subtitle. This is the kind of figure the issue excludes. |
| BBC, §4.5 | gaps between subtitles: "a minimum of one second, preferably a second and a half" | Gaps between subtitles, chosen to avoid a "jerky effect". |
| BBC, §19.3 (cumulative subtitles, where words are added to a line already on screen) | No number: "Make sure there is sufficient time to read each segment of a cumulative" | The closest BBC case to a change on an existing line, and it is phrased as reading time with no figure. |
| BBC, §8 and §11 (colour and stress) | No timing. Colour may mark stress only for the word "I", "used sparingly"; §11.2.1 adds "there is currently little research to indicate the effectiveness of italics for emphasis in subtitles" | Emphasis by style, with no timing guidance at all. |
| [Netflix Timed Text Style Guide: General Requirements](https://partnerhelp.netflixstudios.com/hc/en-us/articles/215758617-Timed-Text-Style-Guide-General-Requirements) | "Minimum duration: 5/6 (five-sixths) of a second per subtitle event (e.g. 20 frames for 24fps)" | A whole subtitle event, not a change inside one. Nothing on highlighting or karaoke. |
| [DCMP Captioning Key: Presentation Rate](https://dcmp.org/learn/captioningkey/601) | 130 / 140 / 160 wpm ceilings by audience level | Reading rate. No minimum duration and nothing on highlighting. |
| Ofcom guidelines on TV and on-demand access services ([2024 PDF](https://www.ofcom.org.uk/siteassets/resources/documents/tv-radio-and-on-demand/broadcast-codes/other-codes/ofcoms-guidelines-on-providing-tv-and-on-demand-access-services.pdf?v=357053)) | **Not verified:** the PDF returned HTTP 403 to automated fetches. Search snippets show a 160–180 wpm subtitle rate. | Reading rate at most. Nothing about highlights was seen, but the document was not read in full. |

## 4. Karaoke and lyric-video practice

No written standard or peer-reviewed timing rule was found for karaoke or per-word highlight duration.

- The one peer-reviewed study of highlighted same-language subtitling found (Kothari et al., *Do weak readers in rural India automatically read same language subtitles on Bollywood films? An eye gaze analysis*, [PMC10615567](https://pmc.ncbi.nlm.nih.gov/articles/PMC10615567/)) compares fixations on highlighted and unhighlighted subtitles. It states no highlight duration or minimum.
- Pages describing "karaoke captions" and "word highlighting subtitles" were all vendor or blog content. One repeated a "200-300 milliseconds per change" figure with no primary source behind it. Under the sourcing rule, none of them is cited.

---

## 5. Consequences for Montagent

1. **Drop the "fewer than N frames" check.** Record the reason: no primary source measures a minimum time for seeing a highlight change on a word already on screen (§1). Any N would be picked for convenience, or borrowed from reading time (montagent-craft's 330 ms per word, BBC's 0.3 s per word), and ADR-0061 forbids both.
2. **Keep `R-HIGHLIGHT-UNPAINTED`** (#542). "Zero painted frames" is a fact about the render, not a perceptual threshold, so it needs no citation.
3. **What the research does support, as review guidance rather than a threshold:** a highlight is most at risk when it coincides with another transient that masks it, such as a cut, a whole-frame change or simultaneous changes elsewhere (Rensink et al. 1997, §2.4). It is not mainly at risk for being short. Nothing here gives a number to enforce that.
4. Out of scope and still open: checking a highlight window against the audio (ADR-0051). That was the actual cause of brief B's invisible "Now".
