# Research: what does ITU-R BT.1359 say, and is there a caption-timing source?

Research for [#566](https://github.com/MBehtemam/Montagent/issues/566), part of map [#560](https://github.com/MBehtemam/Montagent/issues/560). It verifies the citation that [#564](https://github.com/MBehtemam/Montagent/issues/564) used to set the audio-timing cut-off at 90 ms, against the measured gaps in [#561](https://github.com/MBehtemam/Montagent/issues/561).

**Date of research:** 2026-10-01.

**Sourcing rule applied:** every figure and every quotation below was read in the issuing body's own document: ITU, EBU, ATSC, the US eCFR, Ofcom, the BBC and W3C. Three of those sites refused automated download, so the copy read is named each time:

- **itu.int** rejected curl. The PDF was fetched through a browser-style fetch from the ITU's own `dms_pubrec` URL.
- **atsc.org** no longer hosts IS-191. The text is ATSC's own PDF, as the Internet Archive captured it at `atsc.org/standards/is_191.pdf` on 2005-10-04.
- **ofcom.org.uk** returned 403. The text is Ofcom's own PDF, as the Internet Archive captured it on 2024-11-20.

No secondary summary is cited for any figure.

---

## 1. Answers in brief

1. **BT.1359-1 (1998) is quoted correctly.**
   - Detectability is **+45 ms to −125 ms** and acceptability is **+90 ms to −185 ms**. Both are qualified "about … on the average".
   - **A positive value means sound is advanced with respect to vision**, so sound leads.
   - Recommends 2 makes +90 / −185 ms the normative overall tolerance.
2. **90 ms stands.**
   - A caption highlight that opens after its sound is the "sound leading" case, the positive side of BT.1359, and that side's acceptability figure is +90 ms.
   - The margins are unchanged (§3).
3. **Caption-timing sources exist, but none gives a figure near 90 ms.**
   - **FCC, 47 CFR 79.1(j)(2)(ii)**, a binding US rule: "Captions shall begin to appear at the time that the corresponding speech or sounds begin". It states no number.
   - **BBC Subtitle Guidelines v1.2.5 (March 2026):** subtitle appearance "should coincide with speech onset", and subtitles "should never appear more than 2 seconds after the words were spoken".
   - **Ofcom (2024):** "synchronised with the audio" for subtitles in general. Its only figure is a 4.5 s *mean* latency, and that is for live subtitles.
   - **WCAG 2.2:** says "synchronized" and gives no figure.
   - **EBU-TT-D (Tech 3380):** has no tolerance.
   - Under #564's rule ("a published source that speaks to caption or text timing against audio"), FCC and BBC meet the condition. The class moves to **`review`**, with the caveat in §4.4.
4. **Other figures for sound leading picture:**
   - **EBU R37-2007:** at most 40 ms before picture and 60 ms after, end to end.
   - **ATSC IS-191 (2003):** sound never more than 15 ms ahead or 45 ms behind, at encoder input. IS-191 also says BT.1359-1 was "found inadequate" for DTV.
   - Both limits are far tighter than BT.1359's +90 ms acceptability figure, because they are budgets for the broadcast equipment chain, not perceptual limits (§5).

---

## 2. ITU-R BT.1359-1

- **Document:** Recommendation ITU-R BT.1359-1, *Relative timing of sound and vision for broadcasting* (Question ITU-R 35/11), dated **(1998)** on its title page.
- **Status:** the ITU's page lists BT.1359-1 (11/98) as **in force** and BT.1359-0 (02/98) as superseded. No later edition exists.
- **Source:** <https://www.itu.int/rec/R-REC-BT.1359/en>, PDF <https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.1359-1-199811-I!!PDF-E.pdf>.

### 2.1 The thresholds: exact wording

Considering g) reads:

> g) that subjective evaluations show that detectability thresholds are about +45 ms to –125 ms and acceptability thresholds are about +90 ms to –185 ms on the average, a positive value indicates that sound is advanced with respect to vision,

Recommends 2, the normative clause, reads:

> 2 that the overall tolerance in sound/picture timing (between points 1' and 6') shall not exceed +90 ms or –185 ms;

Appendix 1 §3 adds where the figures come from and how wide the bands are:

> Subjective evaluations undertaken in Japan, Switzerland and Australia show a high degree of similarity in the sensitivity of viewers to errors in sound/vision timing in television material for NTSC and PAL systems. Tests conducted have shown that the thresholds of detectability are about + 45 ms to –125 ms and thresholds of acceptability are about +90 ms to –185 ms on the average. […] Each case also shows a clearly defined and rather consistent range of values for the difference (1 grade) between detectable and acceptable limits of about 45 ms for sound leading and about 60 ms for sound delayed as shown in Figure 2.

### 2.2 The sign convention

- **NOTE 1:** "A positive value indicates that sound is advanced with respect to vision."
- **Figure 2** ("Detectability and acceptability thresholds") labels the axes the same way. "Sound delay wrt vision" is on the negative side and "Sound advanced wrt vision" is on the positive side. The x-axis is "Delay time (ms)", running from −200 to +100.

**What this means for Montagent:** a highlight window that opens *s − b* ms after the sound resumes is a case of sound leading the picture, so it is a positive value. The figure that applies is therefore **+90 ms**. The −185 ms figure covers sound arriving late, which would correspond to text that appears *before* its word.

### 2.3 What the thresholds measured

The definitions and test conditions are in Appendix 2:

- **What the two thresholds are:** "Acceptable thresholds (i.e. rating 3.5 on DSIS)". "Detectability" is "rating 4.5 on DSIS".
- **How they were measured:** double-stimulus impairment scale, under BT.500 and BT.1128 viewing conditions.
- **Test material:** a female newsreader. "Assessors should be able to easily see lip movements of announcers."
- **Assessors:** at least 15, expert and non-expert.

So the figures are lip-sync judgements on a talking head. They say nothing about text. #564 point 6 already requires the ADR to call the borrowing an analogy, and this confirms it.

### 2.4 Other figures in the Recommendation (not thresholds)

These figures split the +90 / −185 budget across the broadcast chain. They are not perceptual figures, and they should not be cited as cut-offs:

- **Recommends 3:** image source to the zero reference point, "+25 ms and -100 ms". This is the producer's zone.
- **Recommends 4:** source-selection output to the transmitter input, "+22.5 ms and –30 ms".
- **Recommends 5:** each downstream segment the broadcaster does not control, "±2 ms".

---

## 3. Does 90 ms stand? The margins redone

**Yes.** The figure, the sign and the wording all match what #564 quoted from memory. Recommends 2 ("shall not exceed +90 ms") is the clause to cite, because it is normative and Considering g) is not. It also settles the edge case: a gap of exactly 90 ms is *within* tolerance, so the check fires only on **s − b > 90 ms**.

The research numbers from #561 / #564, against +90 ms:

| Word | Measured gap | Against 90 ms | Outcome |
|---|---|---|---|
| "Now" (late words file) | +110 ms | 20 ms over | fires; the margin exceeds the ~12 ms spread across detector settings |
| "Montagent." | +88 to +90 ms | 0 to 2 ms under | does not fire (90 is not over 90); on the line, as #564 accepted |
| "This" (worst repaired gap) | +44 to +47 ms | 43 to 46 ms under | does not fire |

One new observation follows from the verified text. "This", at +44 to +47 ms, sits **exactly on BT.1359's detectability threshold (+45 ms)**. That is the cited reason a 45 ms cut-off has no margin, and it is the reason the cut-off has to be the acceptability figure rather than the detectability one.

---

## 4. Is there a source on caption or text timing?

### 4.1 FCC: 47 CFR § 79.1(j)(2)(ii), Synchronicity (binding, United States)

- **Edition:** eCFR text current as of the 2026-03-13 issue. The section was last amended 2022-09-07.
- **Source:** <https://www.ecfr.gov/current/title-47/chapter-I/subchapter-C/part-79/subpart-A/section-79.1>

> (ii) Synchronicity. Captioning shall coincide with the corresponding spoken words and sounds to the greatest extent possible, given the type of the programming. Captions shall begin to appear at the time that the corresponding speech or sounds begin and end approximately when the speech or sounds end. Captions shall be displayed on the screen at a speed that permits them to be read by viewers.

The offline captioning best practices at § 79.1(k)(4)(iv) add: "Ensure offline captions are synchronized with the audio of the program."

**No figure.** The rule says onset should coincide with onset and leaves out a tolerance in ms. Errors are judged as "de minimis" case by case under § 79.1(j)(3).

### 4.2 BBC Subtitle Guidelines, Version 1.2.5, March 2026

- **Source:** <https://www.bbc.co.uk/accessibility/forproducts/guides/subtitles/>
- **§5.1, Match subtitle to speech onset:** "Impaired viewers make use of visual cues from the faces of television speakers. Therefore subtitle appearance should coincide with speech onset."
- **§5.2, Match subtitle to pace of speaking:** "Ideally, when the speaker is in shot, your subtitles should not anticipate speech by more than 1.5 seconds or hang up on the screen for more than 1.5 seconds after speech has stopped."
- **§5.4, Keep lag behind speech to a minimum:** "Your aim is to minimise lag between speech and the appearance of the subtitle. But sometimes, in order to meet other requirements (e.g. matching shots), you will find it difficult to avoid slipping slightly out of sync. In this case, subtitles should never appear more than 2 seconds after the words were spoken."

**Figure: 2 s as a ceiling on lag**, which the BBC frames as a last resort. That is 22 times the 90 ms cut-off and would not fire on "Now" (+110 ms).

### 4.3 Ofcom, *Guidelines on Providing Television and On-Demand Access Services*

- **Edition:** the PDF's creation date is 2024-04-12.
- **Source:** Ofcom's own PDF, read through the Internet Archive capture of `ofcom.org.uk/…/ofcoms-guidelines-on-providing-tv-and-on-demand-access-services.pdf?v=357053` taken on 2024-11-20, because ofcom.org.uk returned 403.
- **§3.1:** "Subtitling is text on screen representing speech and sound effects, synchronised as closely as possible with the sound."
- **Heading over §3.6:** "In general, subtitles should be synchronised with the audio, and reflect the speech verbatim, as closely as possible". §3.6 adds: "Subtitles should not appear before key information is relayed".
- **§3.9, under the heading "Live subtitles should have an average delay of no more than 4.5 seconds":** "providers should aim for a mean latency of no more than 4.5 seconds across their live programming taken together". Footnote 25 reads: "By latency, we mean the time delay between the speech and live subtitles."

**Figure:** for prerecorded subtitles there is none. For live subtitles the figure is 4.5 s, and it is a *mean* across a provider's whole output, not a per-subtitle limit.

### 4.4 W3C and EBU

- **WCAG 2.2** (W3C Recommendation, 12 December 2024, <https://www.w3.org/TR/WCAG22/>) defines captions as a "synchronized visual and/or text alternative for both speech and non-speech audio information". SC 1.2.2 and its Understanding document state no timing tolerance.
- **EBU Tech 3380, EBU-TT-D v1.0.1 (May 2018)**, <https://tech.ebu.ch/docs/tech/tech3380.pdf>. Annex E (informative) asks display devices "to render and remove subtitles as close as possible to their respective begin and end times". It gives no tolerance.

### 4.5 Verdict for the class

**Sources that speak to text timing against audio do exist.**

- **FCC:** binding, and it requires caption *onset* to coincide with speech onset.
- **BBC:** a guideline with the same onset rule.
- **Ofcom:** requires subtitles to be "synchronised with the audio".

#564 point 7 says the class becomes `review` "only if the verification turns up a published source that speaks to caption or text timing against audio". The FCC and BBC texts meet that condition, so **the class becomes `review`**.

**Caveat for the ADR:** these sources back the *rule* that text should start with its speech. They do not back the *number*. The only caption-specific figures are 2 s (BBC, a last-resort ceiling) and 4.5 s (Ofcom, a mean for live subtitles). Both are about 20 to 50 times looser than 90 ms and would not fire on "Now". The cut-off itself still rests only on BT.1359's lip-sync figure, by analogy. The ADR's citation should therefore cite FCC § 79.1(j)(2)(ii) and BBC §5.1 for the *requirement* that onsets coincide, and BT.1359-1 recommends 2 for the 90 ms *tolerance*.

A second difference: subtitles appear in blocks, while Montagent's `highlight` windows mark single words. None of these sources speaks to word-level highlighting.

If the human reads #564's rule as needing a source for the *number*, the class stays `note`. That is a call for the human.

---

## 5. Other figures for sound leading picture

| Source | Sound leading (before picture) | Sound lagging (after picture) | What it is |
|---|---|---|---|
| ITU-R BT.1359-1 (1998), Considering g / App. 1 | detectable ~+45 ms; acceptable ~+90 ms | detectable ~−125 ms; acceptable ~−185 ms | subjective thresholds (DSIS 4.5 and 3.5) |
| ITU-R BT.1359-1, recommends 2 | +90 ms | −185 ms | overall chain tolerance |
| EBU R37-2007, Table 1 | ≤ 40 ms | ≤ 60 ms | end-to-end limit at any output intended for emission |
| EBU R37-2007, per stage | 5 ms | 15 ms | accuracy at each stage |
| ATSC IS-191 (26 June 2003) | 15 ms | 45 ms | at the inputs to the DTV encoding devices |
| ATSC IS-191, receiver | ±15 ms of PTS | ±15 ms of PTS | presentation tolerance |

### 5.1 EBU R37-2007

- **Title:** *The relative timing of the sound and vision components of a television signal*.
- **Edition:** Geneva, February 2007. First issued 1986, revised 1997, 2002 and 2006, re-issued 2007.
- **Source:** <https://tech.ebu.ch/docs/r/r037.pdf>
- **Basis of the figures:** "The following recommended maximum values have been established on the basis of subjective tests of the relative delays at which failure of the synchronism between lip movements and speech becomes perceptible to 50% of observers, under the viewing conditions defined in EBU Recommendation R28".
- **Per stage:** "The accuracy of A/V synchronisation at each stage should lie within the range of Audio 5 ms early (sound before picture) to 15 ms late (sound after picture)."
- **Table 1** ("Limits of the relative timing of the sound component of a television programme relative to the corresponding picture component"): "Sound before picture ≤ 40 ms", "Sound after picture ≤ 60 ms".

### 5.2 ATSC IS-191

- **Title:** *ATSC Implementation Subcommittee Finding: Relative Timing of Sound and Vision for Broadcast Operations*, Doc. IS-191, 26 June 2003.
- **Source:** ATSC's PDF from `http://www.atsc.org/standards/is_191.pdf`, read through the Internet Archive capture of 2005-10-04. ATSC no longer hosts it.
- **Encoder input:** "IS finds that under all operational situations, at the inputs to the DTV encoding devices, the sound program should be tightly synchronized to the video program. The sound program should never lead the video program by more than 15 milliseconds, and should never lag the video program by more than 45 milliseconds."
- **Receiver:** "…present the audio-video content to the viewer with a tolerance of +/-15 milliseconds of the time indicated by PTS."
- **On BT.1359:** "Note: ITU R BT.1359-1 (1998) was carefully considered and found inadequate for purposes of audio and video synchronization for DTV broadcasting."

### 5.3 Where these disagree with BT.1359

- **Sound leading.** BT.1359 allows +90 ms overall. EBU R37 allows 40 ms end to end, and ATSC allows 15 ms at encoder input. ATSC explicitly rejects BT.1359-1 for DTV.
- **They measure different things.** R37 and IS-191 are *operational budgets*: they leave headroom so that the sum of errors along the chain stays below perception. BT.1359's +90 ms is the *acceptability* threshold, the point at which viewers rate the error as annoying, and that is the question the check asks.
- **On perception they agree.** EBU R37's basis ("perceptible to 50% of observers") is a detectability measure, and its 40 ms is close to BT.1359's ~45 ms detectability figure.
- **They would misfire here.** A cut-off at R37's 40 ms or IS-191's 15 ms would fire on "This" (+44 to +47 ms), a correct repaired project. The court ruled that kind of false fire the expensive error. Neither is a suitable cut-off for this check, but both belong in the ADR's citation paragraph as the stricter, equipment-chain view.

---

## 6. Sources

- ITU-R BT.1359-1 (1998): <https://www.itu.int/rec/R-REC-BT.1359/en>
- EBU R37-2007: <https://tech.ebu.ch/docs/r/r037.pdf>
- ATSC IS-191 (2003): `http://www.atsc.org/standards/is_191.pdf`, as captured at <https://web.archive.org/web/20051004001811/http://www.atsc.org:80/standards/is_191.pdf>
- 47 CFR § 79.1: <https://www.ecfr.gov/current/title-47/chapter-I/subchapter-C/part-79/subpart-A/section-79.1>
- BBC Subtitle Guidelines v1.2.5: <https://www.bbc.co.uk/accessibility/forproducts/guides/subtitles/>
- Ofcom, Guidelines on Providing TV and On-Demand Access Services (2024): `https://www.ofcom.org.uk/siteassets/resources/documents/tv-radio-and-on-demand/broadcast-codes/other-codes/ofcoms-guidelines-on-providing-tv-and-on-demand-access-services.pdf?v=357053`, as captured at <https://web.archive.org/web/20241120082743/https://www.ofcom.org.uk/siteassets/resources/documents/tv-radio-and-on-demand/broadcast-codes/other-codes/ofcoms-guidelines-on-providing-tv-and-on-demand-access-services.pdf?v=357053>
- WCAG 2.2: <https://www.w3.org/TR/WCAG22/>, and Understanding SC 1.2.2: <https://www.w3.org/WAI/WCAG22/Understanding/captions-prerecorded.html>
- EBU Tech 3380 (EBU-TT-D) v1.0.1: <https://tech.ebu.ch/docs/tech/tech3380.pdf>
