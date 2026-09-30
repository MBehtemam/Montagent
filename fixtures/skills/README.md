# Skills fixture set

The files a skill's recipes are checked against (#450, #513). `crates/montagent/tests/skills.rs`
copies this directory to a scratch location, writes each fenced `json` snippet from `skills/`
into it, and runs `validate`. Errors fail and warnings don't. It also runs each skill script
there. So a snippet names these paths, relative to its own project file, and the
media-dependent checks run for real: probed durations, overrun, font resolution and the
licence attestation.

| Path | What it is |
|---|---|
| `fonts/Inter-Bold.ttf` | Inter 4.1 Bold, the eval pack's copy. Its `fontVendor` attestation is `{"licence": "OFL-1.1", "source": "https://github.com/rsms/inter", "sha256": "288316099b1e0a47a4716d159098005eef7c0066921f34e3200393dbdb01947f"}`. |
| `media/still.png` | 360×360 test pattern. |
| `media/bed.wav` | 6 s mono two-tone bed. |
| `media/bed.words.json` | Word timings over the bed, in the pack's `voiceover.words.json` shape. |
| `media/bed.visemes.json` | Azure viseme ids over the bed, in the pack's `character/voice/line-1.json` shape. |
| `media/clip.mp4` | 30 s, 320×180, 25 fps green screen with a moving subject and a tone. |
| `rig/rig.json`, `rig/parts/` | A three-level rig (torso → head → mouth and eyes; torso → arm) in the pack's `character/rig.json` shape: parts padded so each pivot is the canvas centre, parents, and a viseme → mouth table. |

Everything except the font is synthetic, and `make_fixtures.py` regenerates it with ffmpeg.

## A skill script's header

A script in `skills/<skill>/scripts/` states two lines in its leading docstring or comments.
The guard runs it on exactly that Python (through `uv`), from this directory, with those
arguments, and then validates what it prints:

```
Requires Python >= 3.9
drift-guard: rig/rig.json media/bed.visemes.json
```

`--help` has to state the same minimum version.
