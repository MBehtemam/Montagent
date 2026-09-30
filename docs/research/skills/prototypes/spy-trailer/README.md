# Prototype: a VFX-heavy spy-thriller trailer in Montagent

Throwaway prototype for [#508](https://github.com/MBehtemam/Montagent/issues/508) on the skills map (#441). **SILENT PROTOCOL** is a 36 s, 1920×1080,
30 fps trailer in the style of a 007 spot. The agent (Elias Vale), his villain and the title
are all original. It has a gun-barrel-style cold open, a story told in six shots and three
voice lines, text cards, a HUD insert, a montage and a title reveal.

**Watch:** `out/silent-protocol.mp4` (re-encoded at CRF 29 for the repository; the render
itself was 59 MB). **Glance:** `contact-sheet.jpg`, 20 frames across the trailer.

## Pipeline

| Step | Script | What it does |
|---|---|---|
| Shots | `gen_images.sh`, `gen_images2.sh`, `flux.py` | FLUX.2-pro on Azure: six 1920×1088 cinematic stills in `img/`. The filter blocked five prompts (casino ×2, control room ×2, helicopter); those shots were dropped or rebuilt from shapes. |
| Voices | `vo.py` | Azure Speech: narrator `en-US-DavisNeural`, villain `en-GB-ThomasNeural`, agent `en-GB-RyanNeural`, with word timings in `vo/`. |
| Score | `score.py` | The music and sound design, synthesised from scratch in numpy (ticks, whooshes, booms, braams, a pad, toms, a riser), cut to `plan.py`'s timeline. It is seeded, so `score.wav` is not committed; run the script before rendering. |
| Plates | `plates.py` | Still PNGs in `fx/`: rifling spiral, vignette, grain, anamorphic flare, light sweep, embers, HUD map. |
| Fonts | `fonts/` | Cinzel and Oswald (OFL 1.1, from google/fonts), instanced to static weights with fontTools and vendored with `montagent fonts vendor`. |
| Scene | `plan.py`, `scene.py` | The timeline, then every element and keyframe, written to `trailer.montagent.json`: 181 elements on 114 tracks. |

Render: `montagent validate` then `montagent render trailer.montagent.json` (0 errors). Keys
come from a git-ignored `.env` at the repository root (`AZURE_IMAGE_*`, `AZURE_SPEECH_*`).

## What is Montagent and what is not

Every asset handed to Montagent is still: a photo, a PNG plate, a WAV. **All motion is
keyframes in the project file,** rendered by Montagent: dots, iris, spin, Ken Burns, shake,
glitch, orbits, typing, per-letter animation, flare, sweep, embers, letterbox, fades and the
music ducking. A script computed those keyframes; several are per-frame lists (shake
offsets, orbit positions, grain jitter) that nobody would type.

## Findings

See the resolution comment on #508.
