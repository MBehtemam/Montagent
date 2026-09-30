# Montagent eval asset pack

Every run of the skills eval, in every arm, starts in an empty directory containing this
pack and nothing else. Every brief, development and held-out, is written against it.
Treat it as a client's delivery folder: these files are what you have.

## Brand

Montagent is a video editor that AI agents drive. Its identity is a mark of four tiles
and the name set in Inter Bold.

| Colour | Hex | Use |
|---|---|---|
| Ink | `#101418` | text and tiles on light grounds; the dark ground |
| Paper | `#F5F0E6` | the light ground; text and tiles on dark grounds |
| Signal | `#FF5A36` | the accent: one word, one shape, never a whole field of text |

**The mark** sits in a 1024×1024 box: four tiles on a 2×2 grid, each 460×460, 40 apart,
32 in from every edge. So the tiles' top-left corners are at (32, 32), (532, 32),
(32, 532) and (532, 532). The top-right tile is a **circle** in Signal. The other three
are **rounded squares**, corner radius 96, in Ink (on light grounds) or Paper (on dark).

| File | Size | What it is |
|---|---|---|
| `brand/mark.png` | 1024×1024, alpha | the mark, Ink tiles |
| `brand/mark-on-dark.png` | 1024×1024, alpha | the mark, Paper tiles |
| `brand/wordmark.png` | 1998×419, alpha | "Montagent" in Inter Bold, Ink |
| `brand/wordmark-on-dark.png` | 1998×419, alpha | the same, Paper |
| `brand/lockup.png` | 2694×623, alpha | mark and wordmark side by side, Ink |
| `brand/lockup-on-dark.png` | 2694×623, alpha | the same, Paper |

## Fonts

| File | Face |
|---|---|
| `fonts/Inter-Regular.ttf` | Inter 4.1, Regular |
| `fonts/Inter-Bold.ttf` | Inter 4.1, Bold |

## Music

| File | What it is |
|---|---|
| `music/bed-120bpm.wav` | 16.0 s, 48 kHz stereo. **120 BPM**: a beat every 0.5 s, four beats to a bar, the first downbeat at 0.000 s. The drums stop at 14 s and the last bar is a held chord, so a fade over any final second has something to fade. |

## Voice

| File | What it is |
|---|---|
| `voiceover/voiceover.wav` | 11.8 s, 24 kHz mono, a synthetic voice reading `script.txt` |
| `voiceover/script.txt` | the words |
| `voiceover/voiceover.words.json` | each word with its `start` and `end` in milliseconds from the start of the file |

## Presenter

A presenter speaking to camera against a green screen, in three takes. The presenter is a
synthetic avatar, not a real person. Each take is 1920×1080 at 25 fps, with the presenter
centred, and has its script and word timings beside it, in the same form as the
voiceover's.

| File | What it is |
|---|---|
| `presenter/take-1.mp4` · `.txt` · `.words.json` | 9.8 s, 25 words: "I stopped dragging clips around a timeline…" |
| `presenter/take-2.mp4` · `.txt` · `.words.json` | 9.9 s, 26 words: "Here's the trick…" |
| `presenter/take-3.mp4` · `.txt` · `.words.json` | 9.0 s, 25 words: "Most editors hide the video behind a mouse…" |

## Product

| File | What it is |
|---|---|
| `screen/session.mp4` | 55.0 s, 1920×1080, 30 fps, no sound. A real terminal session: Claude Code, with Montagent connected, is asked to add a subtitle that fades in. It reads the format, edits the project file, checks frames and renders. |
| `stills/session-01.png` | 1920×1080: the terminal at the moment of the edit, with the diff of the project file on screen |
| `stills/session-02.png` | 960×540: a frame of the video that session rendered, "Hello, Montagent" with the new subtitle beneath |

## Character

**Hoot**, Montagent's mascot: a cartoon owl in a coral cardigan, drawn flat with dark
outlines. It comes as a **cut-out rig**, a set of parts to pose and animate, with a voice
and a set to stand in.

### The rig

`character/rig.json` describes it. Every coordinate is in pixels of the original drawing,
which is 1024×1536 (`drawing`); `character/rest.png` is that drawing reassembled from the
parts, cropped to the owl.

- **Each part is a PNG whose pivot is the exact centre of its canvas.** A part's `pivot` is
  where that centre sits in the drawing. So a part centred on its pivot, unrotated, is at
  rest, and it rotates about its joint.
- **At rest the owl stands as drawn:** facing front, feet on the ground, arms held out and
  down at about 25° below horizontal, hands open. A lowered arm is a rotation from there.
- **`parent`** gives the hierarchy: `torso` (legs and feet included, pivoting at the feet)
  > `head` (at the neck), `upper_arm_left` / `upper_arm_right` (at the shoulders) >
  `forearm_left` / `forearm_right` (at the elbows, hands included). Left and right are the
  viewer's. A child follows its parent: when the upper arm turns, the elbow moves with it.
- **`draw_order`** is back to front. Arms go over the body; the upper arm goes over the
  forearm, so the rounded elbow shows when it bends. The shoulders and elbows are
  overlapping round joints, so a part turned about its pivot never shows a gap.
- **The face overlays** (`mouth_*`, `eyes_closed`) are the head's own canvas, the same size
  and pivot, and are transparent except where they change the face. Put one exactly where
  the head is, with the head's rotation and scale, and it lines up. With no mouth overlay,
  the beak is at rest (closed).

| Mouth | Shape | For sounds like |
|---|---|---|
| `mouth_open` | beak wide open, tongue showing | "ah", "uh", "ow", "eye" |
| `mouth_round` | a small round opening | "oo", "oh", "aw", "w", "r", "sh", "ch" |
| `mouth_small` | slightly open | "eh", "ee", "er", and "t", "d", "n", "k", "g", "l", "h", "th" |
| `mouth_teeth` | beak nearly shut, a strip of white showing | "s", "z", "f", "v" |
| `mouth_closed` | beak pressed shut | "m", "b", "p" |

`visemes` in `rig.json` maps each of Azure Speech's viseme ids (0–21) to one of these
mouths; id 0 is silence, which is `null`: no overlay, the beak at rest.

| File | What it is |
|---|---|
| `character/rig.json` | the rig: parts, pivots, parents, draw order, viseme → mouth |
| `character/rest.png` | the owl at rest, assembled from the parts, alpha |
| `character/parts/torso.png` | body, cardigan, legs and feet |
| `character/parts/head.png` | head, beak at rest, eyes open |
| `character/parts/upper_arm_{left,right}.png` | shoulder to elbow |
| `character/parts/forearm_{left,right}.png` | elbow to fingertips |
| `character/parts/mouth_{open,round,small,teeth,closed}.png` | the five mouths, on the head's canvas |
| `character/parts/eyes_closed.png` | a blink, on the head's canvas |

### Voice

Three lines in the owl's voice, each with its words, and its visemes and word timings as
`{"voice", "duration_ms", "visemes": [[ms, id], …], "words": [{"word", "start", "end"}, …]}`,
all in milliseconds from the start of the file. A viseme holds until the next one.

| File | What it is |
|---|---|
| `character/voice/line-1.wav` · `.txt` · `.json` | 6.0 s, 48 kHz mono: "Hi, I'm Hoot! Your agent writes the video as a file, and Montagent turns it into a movie." |
| `character/voice/line-2.wav` · `.txt` · `.json` | 5.5 s: "Want to change one word in a video? Just edit the file, and Montagent renders it again." |
| `character/voice/line-3.wav` · `.txt` · `.json` | 6.6 s: "Hmm. That caption lands two frames late. Let me check… Fixed! Right on the beat." |

### Set and prop

| File | What it is |
|---|---|
| `character/study.png` | 1920×1080: a study in the same flat style, bookcases at the sides and open wall in the middle. The floor starts about 870 px down. |
| `character/laptop.png` | 708×566, alpha: an open laptop, front view. Its blank screen is the rectangle x 120–587, y 41–312 of this image. |

## Licences

Nothing here is covered by the repository's MIT licence unless it says so. The canonical
record is the repository's `LICENSE-MEDIA.md`.

- `fonts/`: SIL Open Font License 1.1, © The Inter Project Authors; see `fonts/Inter-LICENSE.txt`.
- `music/bed-120bpm.wav`: CC0 1.0. It is synthesised by `docs/research/skills-eval/pack-src/make_music_bed.py` in the Montagent repository (MIT), which writes the same bytes on every run.
- `voiceover/voiceover.wav`: spoken by Kokoro-82M (hexgrad, Apache-2.0) with its `af_heart` voice, by `docs/research/skills-eval/pack-src/make_voiceover.py` in the Montagent repository (MIT). © 2026 Mohammad Bagher Ehtemam, rights reserved, like the rest of the maintainer's media.
- `presenter/*.mp4`: generated with Azure AI Speech's text-to-speech avatar (the prebuilt avatar "Harry", casual style) on the maintainer's subscription, under Microsoft's Product Terms and the Azure AI Speech code of conduct. That code requires disclosing that the presenter is synthetic; this manifest does so. The scripts and word timings (made by `docs/research/skills-eval/pack-src/align_takes.py`) are the maintainer's, as below.
- `character/`: the drawings (the owl, its blink, the study and the laptop) were generated with FLUX.2-pro on Azure AI Foundry, and the voice lines with Azure AI Speech (the `en-US-AvaNeural` voice, which supplied the visemes and word timings), both on the maintainer's subscription and under Microsoft's Product Terms. The owl's voice is synthetic. The rig was cut, the five mouths drawn, and the set and prop prepared by the scripts in `docs/research/skills-eval/pack-src/character/` in the Montagent repository (MIT). © 2026 Mohammad Bagher Ehtemam, rights reserved, as below.
- `brand/`, `character/`, `presenter/*.txt`, `presenter/*.words.json`, `screen/`, `stills/`: © 2026 Mohammad Bagher Ehtemam, all rights reserved. They may be used only to build, test, benchmark and study this repository. `brand/` is drawn by `docs/research/skills-eval/pack-src/make_brand.py` in the Montagent repository (MIT).
