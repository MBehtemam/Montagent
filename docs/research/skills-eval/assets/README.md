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

The person who builds Montagent, filmed vertically against a green screen, speaking to
camera. Each take has its script and word timings beside it, in the same form as the
voiceover's.

| File | What it is |
|---|---|
| `presenter/take-1.mp4` · `.txt` · `.words.json` | *to be recorded* |
| `presenter/take-2.mp4` · `.txt` · `.words.json` | *to be recorded* |
| `presenter/take-3.mp4` · `.txt` · `.words.json` | *to be recorded* |

## Product

| File | What it is |
|---|---|
| `screen/session.mp4` | *to be recorded*: a screen recording of an agent using Montagent in a terminal |
| `stills/session-01.png`, `stills/session-02.png` | *to be taken from the screen recording* |
| `render/montagent-render.mp4` | 8.0 s, 720×1280, 25 fps, with sound: a stretch of a real short rendered by Montagent |

## Licences

Nothing here is covered by the repository's MIT licence unless it says so. The canonical
record is the repository's `LICENSE-MEDIA.md`.

- `fonts/`: SIL Open Font License 1.1, © The Inter Project Authors; see `fonts/Inter-LICENSE.txt`.
- `music/bed-120bpm.wav`: CC0 1.0. It is synthesised by `docs/research/skills-eval/pack-src/make_music_bed.py` in the Montagent repository (MIT), which writes the same bytes on every run.
- `voiceover/voiceover.wav`: spoken by Kokoro-82M (hexgrad, Apache-2.0) with its `af_heart` voice, by `docs/research/skills-eval/pack-src/make_voiceover.py` in the Montagent repository (MIT). © 2026 Mohammad Bagher Ehtemam, rights reserved, like the rest of the maintainer's media.
- `brand/`, `presenter/`, `screen/`, `stills/`, `render/`: © 2026 Mohammad Bagher Ehtemam, all rights reserved. They may be used only to build, test, benchmark and study this repository. `brand/` is drawn by `docs/research/skills-eval/pack-src/make_brand.py` in the Montagent repository (MIT).
