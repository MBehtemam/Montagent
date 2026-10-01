# Craft sources: numbers with URLs and quotes

Legend: [V] = fetched this session and quote read from the page/file. [S] = seen only in a search-result summary (secondary). [U] = could not verify.

## 1. Safe areas

### 1.1 SMPTE ST 2046-1 (action 93% / title 90%) [V via NAB reproduction; standard itself is paywalled]
- https://www.nab.org/xert/scitech/pdfs/tv031510.pdf (NAB, extracts reproduced with SMPTE permission)
- "SMPTE ST 2046-1 defines the Safe Action Area as a rectangle that is 93% of the width and 93% of the height of the Production Aperture ... The Safe Title Area is defined as a rectangle that is 90% of the width and 90% of the height of the Production Aperture ... and concentric with it."
- Legacy for context: "a safe action area 90% of the width and 90% of the height ... and a safe title area 80% of the width and 80% of the height" (pre-2009, CRT).
- "SMPTE RP 2046-2 defines only 90%, 90% safe areas" (16:9 images shown on 4:3 displays).
- Standard landing page (abstract not readable via fetch): https://ieeexplore.ieee.org/document/7291650/ [U]

### 1.2 EBU R 95 v1.1 (June 2017) [V, read the PDF]
- https://tech.ebu.ch/docs/r/r095.pdf
- Note 5: "The action safe area is 3.5% and the graphics safe area is 5%, at the top, bottom and lateral parts of the image." => action safe = 93% x 93%, graphics safe = 90% x 90%. Same numbers as SMPTE ST 2046-1.
- "all essential action takes place inside the Action Safe Area, and all graphics are framed in the Graphics Safe Area".
- On 1920x1080: 3.5% = 67 px per edge (per the PDF's erratum note "3.5% of 16:9 ... correct, 67 pixels"); 5% = 96 px horizontally, 54 px vertically (computed).
- CAUTION: a web-search summary claimed "EBU R95 defines 93% (action) and 88% (title)". The primary PDF says 5% margin (= 90%), so the 88% is wrong/obsolete.

### 1.3 BBC subtitle safe area (a broadcaster's practical use) [V]
- https://www.bbc.co.uk/accessibility/forproducts/guides/subtitles (section 10 Positioning)
- "For 16:9 video in landscape mode, subtitles should not be placed outside the central 90% vertically and the central 75% horizontally."
- "For 9:16 video in portrait or vertical mode, this is reversed: subtitles should not be placed outside the central 75% vertically and the central 90% horizontally."
- "For 4:3 and 1:1 (square) video, ... the central 90% vertically and the central 90% horizontally."
- "In vertical (9:16) videos, it is common to position subtitles a little higher up, though still generally in the lower third of the screen."
- This is the only primary, non-platform-specific source found for 9:16 text placement.

### 1.4 Vertical 9:16 platform overlay regions (context only, NOT primary) [S/U]
Nothing official was fetched. All figures below are from third-party aggregator pages in one web search (e.g. https://adaptlypost.com/en/blog/social-media-safe-zones-2026-complete-guide, https://reloop.so/blog/article/social-media-safe-zones/, https://postplanify.com/blog/social-media-safe-zones-2026-complete-guide). They disagree with each other, which itself shows the range:
- Margins on 1080x1920 (top / bottom / left / right px), one aggregator: TikTok 240/660/120/120; Reels 269/672/65/65; Shorts 288/672/48/192.
- Another: Reels dead zone top 210 / bottom 310; Shorts top 120 / bottom 300; TikTok top 108 / bottom 320 / left 60 / right 120.
- A "universal" intersection quoted: 900x1400 centred (=> about 90 px sides, 260 px top and bottom, i.e. roughly 83% width x 73% height).
- Range of bottom exclusion across sources: about 310 to 672 px (16% to 35% of height); top: about 108 to 288 px (6% to 15%); right: 48 to 192 px. Treat as unverified; the platform UI changes often. Consistent with BBC's "central 75% vertically" for vertical text.

## 2. Motion durations and easing

### 2.1 Material Design 3 tokens [V from code; the m3.material.io page is JS-rendered and returned no content]
Sources: https://github.com/material-components/material-web/blob/main/tokens/versions/v0_192/_md-sys-motion.scss and https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md (docs page links to https://m3.material.io/styles/motion/easing-and-duration/tokens-specs).

Durations (`'duration-short1': ... 50ms` etc.):
| token | ms | token | ms |
|---|---|---|---|
| short1 | 50 | long1 | 450 |
| short2 | 100 | long2 | 500 |
| short3 | 150 | long3 | 550 |
| short4 | 200 | long4 | 600 |
| medium1 | 250 | extra-long1 | 700 |
| medium2 | 300 | extra-long2 | 800 |
| medium3 | 350 | extra-long3 | 900 |
| medium4 | 400 | extra-long4 | 1000 |

Easing (material-web tokens):
- standard: cubic-bezier(0.2, 0, 0, 1)
- standard-decelerate: cubic-bezier(0, 0, 0, 1)
- standard-accelerate: cubic-bezier(0.3, 0, 1, 1)
- emphasized (web token): cubic-bezier(0.2, 0, 0, 1). Note: Android doc says emphasized is a two-segment path "M 0,0 C 0.05, 0, 0.133333, 0.06, 0.166666, 0.4 C 0.208333, 0.82, 0.25, 1, 1, 1", which a single cubic-bezier only approximates.
- emphasized-decelerate: cubic-bezier(0.05, 0.7, 0.1, 1)
- emphasized-accelerate: cubic-bezier(0.3, 0, 0.8, 0.15)
- linear: cubic-bezier(0, 0, 1, 1)
- legacy: cubic-bezier(0.4, 0, 0.2, 1) (decelerate 0,0,0.2,1; accelerate 0.4,0,1,1)
- Android doc usage: "Easing used for common, M3-styled animations that enter the screen" (emphasized decelerate) / "...exit the screen" (emphasized accelerate).
- Pairing guidance (which duration for which size) lives on the JS page and was NOT retrieved [U].

### 2.2 IBM Carbon [V]
- https://carbondesignsystem.com/elements/motion/overview/
- Productive: "creates a sense of efficiency and responsiveness, while remaining subtle and out of the way." Expressive: "delivers enthusiastic, vibrant, and highly visible movement".
- Curves (productive / expressive):
  - standard: cubic-bezier(0.2, 0, 0.38, 0.9) / cubic-bezier(0.4, 0.14, 0.3, 1)
  - entrance: cubic-bezier(0, 0, 0.38, 0.9) / cubic-bezier(0, 0, 0.3, 1)
  - exit: cubic-bezier(0.2, 0, 1, 0.9) / cubic-bezier(0.4, 0.14, 1, 1)
- Durations: fast-01 70 ms (micro-interactions), fast-02 110 ms (fades), moderate-01 150 ms (small expansions), moderate-02 240 ms (system communication), slow-01 400 ms (large expansions), slow-02 700 ms (background dimming).
- Caveat: the fetch tool summarises pages, so these came via a model summary of the page; the curve values match Carbon's published tokens as I know them, but re-check on the page if a quote is needed.

### 2.3 Apple HIG [U]
- https://developer.apple.com/design/human-interface-guidelines/motion returned only a title (JS-rendered). No numbers verified. Do not cite Apple numbers.

### 2.4 After Effects MCP skill `ae-motion-principles` [V, read the raw file]
- https://github.com/solomondivyananth/aftereffects-mcp/blob/main/skills/ae-motion-principles/SKILL.md
- "Motion is judged in frames, not seconds ... At 30 fps:" then:
  - a hit, a stomp, a cut-on-beat: 2-4 frames ("anything shorter than 2 frames reads as a glitch")
  - a small UI move, a pop: 6-10
  - a standard move across the frame: 12-18
  - a big, heavy or premium move: 20-30 ("weight comes from duration and ease together")
  - settle after an overshoot: 6-12
  - "a hold long enough to read text: ~ words ÷ 3 × 30 frames, and at least 24 ... viewers read about 3 words a second"
  - "At 24 or 25 fps, keep the same *seconds*: scale the frame counts."
- Ease table (AE influence %, "out" on key A, "in" on key B): linear "almost never right"; gentle `"easy"` = 33%; snappy arrival A out 15 / B in 85; heavy premium A out 60 / B in 90; launch A out 85 / B in 15; hard stop A out 10 then linear arrival + overshoot key. "Most 'make it feel better' notes come down to raising the landing influence to 75-90 and shortening the move."
- Overshoot example: scale 0 -> 108 at 8f -> 97 at 14f -> 100 at 19f. Slam: "Scale 300% -> 92% in 3 frames, then -> 100% in 3 more".
- Stagger: "Offset each element by 2-4 frames. Keep a group's total stagger under about 12 frames". Secondary parts settle "2-4 frames after the main body". Anticipation: "2-6 frames, 5-10% of the distance".

### 2.5 HyperFrames `hyperframes-creative/references` [V, read raw files]
Base: https://github.com/heygen-com/hyperframes/tree/main/skills/hyperframes-creative/references
- motion-principles.md:
  - "Fast (0.15-0.3s) - energy, urgency, confidence; Medium (0.3-0.5s) - professional, most content; Slow (0.5-0.8s) - gravity, luxury, contemplation; Very slow (0.8-2.0s) - cinematic, emotional, atmospheric"
  - "The slowest scene should be 3x slower than the fastest."
  - "no more than 2 independent tweens with the same ease in a scene."
  - "Don't start at t=0. Offset the first animation 0.1-0.3s."
  - "Total stagger sequence under 500ms regardless of item count."
  - "Entrances need longer than exits. A card takes 0.4s to appear but 0.25s to disappear." (exit about 60% of entrance)
  - Build/breathe/resolve = 0-30% / 30-70% / 70-100% of a scene; "ONE ambient motion" in breathe.
  - ".out for elements entering ... .in for elements leaving ... .inOut for elements moving between positions."
  - "Hero text: 60-80% of width." "Three layers minimum per scene." "Two focal points minimum per scene."
  - Ken Burns: scale 1 -> 1.04 over the beat. Perspective tilt rotationY -8 at perspective 1200.
- house-style.md: "Motion ... Quick: 0.3-0.6s, vary eases, combine transforms on entrances, overlap entries."
- beat-direction.md: shader transitions 0.5-0.8s; exits use accelerating ease, entries decelerating, "Match exit velocity to entry velocity within ~5% tolerance"; a 5-7 beat reel "usually wants 1-2 shader transitions".

## 3. Reading time / hold length

### 3.1 BBC Subtitle Guidelines [V]
https://www.bbc.co.uk/accessibility/forproducts/guides/subtitles (section 4 Timing)
- "The recommended subtitle speed is 160-180 words-per-minute (WPM) or 0.33 to 0.375 second per word." (= 2.67-3.0 words/s)
- "you should aim to leave a subtitle on screen for a minimum period of around 0.3 seconds per word (e.g. 1.2 seconds for a 4-word subtitle)."
- Changelog: "the word rate for live subtitles has been adjusted to 160-180wpm from 130-150wpm."
- Lines: broadcast 37 chars; online 68% of width (16:9) or 90% (9:16); "25 characters in a 90% width region of a 9:16 (vertical) video"; max 2 lines landscape, 3 lines vertical.

### 3.2 Netflix Timed Text Style Guide [V]
- General requirements: https://partnerhelp.netflixstudios.com/hc/en-us/articles/215758617-Timed-Text-Style-Guide-General-Requirements
  - "Minimum duration: 5/6 (five-sixths) of a second per subtitle event (e.g. 20 frames for 24fps)"
  - "Maximum duration: 7 seconds per subtitle event"; "2 lines maximum".
  - Note: the "frame gap" section has been removed in the current version.
- English: https://partnerhelp.netflixstudios.com/hc/en-us/articles/217350977-English-Timed-Text-Style-Guide
  - "Adult programs: Up to 20 characters per second"; "Children's programs: Up to 17 characters per second"; "42 characters per line"; "Maximum two lines".
  - "Font size: Relative to video resolution and ability to fit 42 characters across screen".
- Rule-of-thumb check: 20 cps / ~6 chars per word incl. space = about 3.3 words/s.

### 3.3 "hold = words / 3 s"
- Appears in the AE skill (2.4): words / 3 x 30 frames, min 24 frames. Consistent with BBC upper bound (180 wpm = 3 w/s). Also HyperFrames typography.md: "Fixed reading time. 3 seconds on screen = must be readable in 2. Fewer words, larger type." (i.e. design for reading at roughly 1.5x speed margin).
- Derived: minimum hold = max(5/6 s, words x 0.33 s) for subtitles; for motion-graphics text add the entrance time.

## 4. Type scale for video

- BBC subtitle line height [V] (https://www.bbc.co.uk/accessibility/forproducts/guides/subtitles, 9.2.1): "Font size should be set to fit within a line height in the range 7% to 8% of the active video height for 16:9, 4:3 and 1:1 aspect ratio videos" and "3.9% to 4.5% of the active video height for 9:16 aspect ratio videos". Rationale in doc: 8% originates in Teletext; 9:16 value is 8% x 9/16 = 4.5%. Line height is not font size; font size is roughly line height / 1.2-1.3 (derived) => about 5.5-6.7% of height in 16:9 (about 60-72 px on 1080).
- Netflix [V]: no % of frame; "tts:fontSize shall be defined as 100%", size "relative to video resolution and ability to fit 42 characters across screen".
- BBC default viewing angle [V]: "a default size of 0.5 degrees subtended at the eye may be used to derive the default line height".
- HyperFrames typography.md [V] (https://github.com/heygen-com/hyperframes/blob/main/skills/hyperframes-creative/references/typography.md): full-screen viewing "body 20px minimum, headlines 60px+, data labels 16px"; in-feed viewing "body >=32px, headlines >=90px, data labels >=24px (first-pass values; calibrate against real renders)". Units are px on a 1080p-class canvas (canvas size not stated in that passage; as % of 1080: body 1.9%, headline 5.6%, label 1.5%; in-feed body 3.0%, headline 8.3%, label 2.2%). These are the author's own first-pass numbers, not a standard.
- Same file: "Weight contrast must be extreme ... Video needs 300 vs 900"; "Tracking tighter than web. -0.03em to -0.05em on display sizes"; light-on-dark "Use 350 instead of 400 for body text"; line-height +0.05-0.1 on dark.
- Modular scale ratios (1.25 / 1.333 / 1.5): no primary source for video found [U]. The ratios are the standard named musical intervals (major third 5:4, perfect fourth 4:3, perfect fifth 3:2) per Tim Brown's modularscale; not fetched.
- EBU/SMPTE give no minimum text height; none found for broadcast [U].

## 5. Beat grid

ms per beat = 60000 / BPM. frames per beat = ms/beat x fps / 1000. Whole = integer (computed here, exact).

| BPM | ms/beat | 24 fps | 25 fps | 30 fps | 60 fps | 4-beat bar (s) |
|---|---|---|---|---|---|---|
| 90 | 666.667 | 16 (whole) | 16.667 | 20 (whole) | 40 (whole) | 2.667 |
| 100 | 600.000 | 14.4 | 15 (whole) | 18 (whole) | 36 (whole) | 2.400 |
| 110 | 545.455 | 13.091 | 13.636 | 16.364 | 32.727 | 2.182 |
| 120 | 500.000 | 12 (whole) | 12.5 | 15 (whole) | 30 (whole) | 2.000 |
| 128 | 468.750 | 11.25 | 11.719 | 14.0625 | 28.125 | 1.875 |
| 140 | 428.571 | 10.286 | 10.714 | 12.857 | 25.714 | 1.714 |

Notes: 90, 100, 120 are whole at 30 and 60 fps; 110, 128, 140 are never whole at any of the four rates. Non-whole beats are fine if each cut is rounded from the absolute time (round(n x frames_per_beat)), which keeps drift at 0 and error under 0.5 frame; do not add a rounded per-beat length repeatedly. A 4/4 bar at 120 BPM is exactly 2 s; at 128 BPM, 4 bars = 7.5 s = 180 frames at 24 fps (whole), 8 bars = 15 s.
Primary sources on cutting to the beat: none found with numbers. HyperFrames beat-direction.md [V] only says shader/hard transitions suit "Any moment the music/VO punctuates with a downbeat or SFX hit" and "percussive edits on the beat". The AE skill [V]: "Put a comp marker on each beat ... Everything is timed to markers, not guessed", and a cut-on-beat "hit" is 2-4 frames. "Phrase = 4 or 8 bars" [U]: no primary source fetched.

## 6. Restraint

### 6.1 WCAG contrast [V]
- https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html (1.4.3 AA): "contrast ratio of at least 4.5:1" for text; "Large-scale text and images of large-scale text have a contrast ratio of at least 3:1". Large = "at least 18 point or 14 point bold"; "1pt = 1.333px, therefore 14pt and 18pt are equivalent to approximately 18.5px and 24px." (the 18.5px figure is the bold threshold.)
- https://www.w3.org/WAI/WCAG22/Understanding/contrast-enhanced.html (1.4.6 AAA): 7:1 for text, 4.5:1 for large-scale text.
- Applies to video text burned in as images of text; WCAG itself has no video-frame carve-out beyond logotypes/incidental.

### 6.2 Flashing [V]
- https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html (2.3.2): "do not contain anything that flashes more than three times in any one second period". Flash = "a pair of opposing changes in relative luminance of 10% or more of the maximum relative luminance (1.0) where the relative luminance of the darker image is below 0.80"; area threshold ".006 steradians within any 10 degree visual field ... (25% of any 10 degree visual field)". Relevant to strobe/flash-frame effects (3 flashes/s max => no luminance flip faster than every ~167 ms repeated).

### 6.3 HyperFrames restraint numbers [V] (https://github.com/heygen-com/hyperframes/blob/main/skills/hyperframes-creative/references/house-style.md and typography.md)
- "One accent hue. Same background across all scenes." "Declare one background, one foreground, one accent before writing HTML." Contrast "enforced by `hyperframes check` (WCAG AA)".
- Pure #000 / #fff discouraged: "tint toward your accent hue instead".
- Typefaces: "One expressive font per scene"; "Don't pair two sans-serifs" (serif + sans, or sans + mono); effectively 2 families.
- Decoratives: "2-5 per scene" (decorative elements), ghost text "3-8% opacity"; "ONE ambient motion" per scene (a shared breath/drift/pulse applied to decoratives).
- Motion: "no more than 2 independent tweens with the same ease in a scene"; stagger total under 500 ms.
- Max simultaneous moving elements: no published number found [U].

## 7. Speed limits against strobing and judder

### 7.1 Pan speed: the "7 second rule" [V]
- RED: https://www.reddigitalcinema.com/red-101/camera-panning-speed
  - "The rule of thumb is to pan no faster than a full image width every seven seconds, otherwise judder will become too detrimental. This rule is especially simple and powerful because it applies regardless of camera lens, model or sensor size."
  - "The seven second rule of thumb is based on traditional theatrical viewing at 24 fps with a 180 degree shutter angle."
  - "images will not immediately become unwatchable faster than seven seconds, nor will they become fully artifact-free when panning slower than this limit. It's just a good starting point."
  - "larger angles cause panning to appear smoother but more smeared, whereas smaller angles cause panning to appear crisper but choppier"; higher playback rate "can reduce the appearance of judder ... without having to decrease the panning speed" (no scaling factor given).
  - "Objects or backgrounds may appear to flash across the screen in discrete jumps ... whenever the on-screen displacement is too great compared to the duration between frames."
- Converted (computed; 7 s = 1 frame width; per-frame figure keeps SECONDS constant, which is the RED statement):
  - 24 fps: 168 frames => 0.595% of frame width per frame (11.4 px/frame on 1920)
  - 25 fps: 175 frames => 0.571% (11.0 px)
  - 30 fps: 210 frames => 0.476% (9.1 px)
  - 60 fps: 420 frames => 0.238% (4.6 px)
  - In per-second terms: 14.3% of frame width per second at any rate. A 10%-of-width move in 1 s is thus 0.7x the "7 s" speed; a 100% crossing in 1 s is 7x over it.
- This is a LIVE-ACTION rule for a camera with natural motion blur. Rendered graphics with no/low motion blur judder at lower speeds; there is no published CG figure [U]. Using the RED statement that shutter matters, add motion blur (180 degree equivalent) before pushing faster moves.

### 7.2 ASC Manual panning table [S/partly U]
- Thread referencing it: http://www.cinematography.com/index.php?showtopic=61301 (table itself not reproduced; fetch returned none). Also https://cinematography.com/index.php?showtopic=44555.
- Search-summary claims (NOT verified against the book): "no faster than 5 degrees per second for a 35mm lens at 24 fps"; "50mm lens at 24fps ... 90-degree pan will take 23 seconds"; table is in American Cinematographer Manual 9th ed. pp. 815-817. Sanity check (computed): 50 mm lens on Super-35-ish gate has ~26 degree horizontal FOV; 90 degrees / 23 s = 3.9 deg/s => 15% of the frame width per second, roughly consistent with RED's 14.3%/s. The ASC figure scales with lens FOV, RED's rule does not.

### 7.3 Distance per frame before it reads as stepping [U]
- No primary source gives a px/frame or %-width/frame threshold. Best available derivation is the RED rule above: about 0.6% of frame width per frame at 24 fps (about 11 px on 1920) for legible, unblurred travel; faster travel needs motion blur or is intended as a "whip".
- RED's phrasing is the closest primary statement: displacement "too great compared to the duration between frames".

### 7.4 Rotation and the wagon-wheel effect [S; math derived]
- Search summary (https://optical-illusions.fandom.com/wiki/Wagon-Wheel_Effect and others, secondary): "The backward appearance happens when the wheel rotates more than 180 degrees per frame at 24 fps." Wikipedia (https://en.wikipedia.org/wiki/Wagon-wheel_effect) describes the mechanism with spokes but gives no numeric threshold [V].
- Derivation (sampling theorem): for an object with N-fold rotational symmetry (N identical spokes), apparent direction reverses once rotation per frame exceeds half the symmetry period, i.e. 180/N degrees per frame; the object appears stationary at exactly 360/N degrees per frame.
  - Asymmetric single-arm hand/needle (N=1): reversal above 180 deg/frame = 4320 deg/s at 24 fps (12 rev/s), 5400 deg/s at 30 fps.
  - 2-fold (bar/propeller): 90 deg/frame; 4 spokes: 45 deg/frame; 12 ticks (a dial): 15 deg/frame; 60 ticks: 3 deg/frame; 360 ticks: 0.5 deg/frame.
  - Comfortable limit is far lower than the reversal point: judder/blur already appears well before. Practical conversion: rev/s limit = (180/N) x fps / 360 = fps/(2N) rev/s (24 fps, N=1: 12 rev/s; N=4: 3 rev/s).
- Primary source with numbers (SMPTE/ASC) not found [U]. Treat the 180 deg/frame as sampling math, not a published craft rule.

## Unverified / not retrieved summary
Apple HIG numbers; M3 page pairing guidance (JS page); platform overlay regions from any official source; ASC panning table; modular scale for video; phrase = 4 or 8 bars; max simultaneous moving elements; any px/frame or %/frame threshold; a SMPTE ST 2046-1 document read directly (NAB reproduction used).
