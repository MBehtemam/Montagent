# Prototype: a cut-out character telling a short story in Montagent

Throwaway prototype for [#487](https://github.com/MBehtemam/Montagent/issues/487) on the skills map (#441). A ~10.5 s scene: a FLUX fox inventor waves ("Oh! Hi there! I'm Pip."), presents a machine ("I built a machine that turns big ideas into…"), toast pops out, cut to a close-up ("…toast.").

**Watch:** `out/side-by-side.mp4`. Montagent is on the left; on the right is the reference, the same scene graph rendered without Montagent (PIL + ffmpeg, sub-pixel affine, real parenting).

## Pipeline

| Step | Script | What it does |
|---|---|---|
| Character | `flux.py` | Azure FLUX.2-pro. `master.png` is the one full-body drawing. `v_mouth{A,O,S,E,M}.png` / `v_blink.png` are *edits* of it ("keep everything identical, change only the mouth"). They register to the master at shift (0,0). |
| Voice | `tts.py` | Azure Speech (`en-US-AnaNeural`): `voice/line-N.wav` + viseme ids and word boundaries in `voice/line-N.json`. |
| Rig | `parts.py` | Keys the white out, splits outline-bounded fills into head / arm / body groups, and gives every pixel to the nearest group. Cuts the arm at the elbow, paints round joint caps, and pads each PNG so **its joint is the canvas centre** (so `origin: "center"` is the pivot). Face overlays share the head's canvas, so they copy its transform verbatim. |
| Props | `props.py` | Keys the machine and toast; crops the background. |
| Scene | `scene.py` | A scene graph (camera > fox > head / upper arm > forearm), animated in Python, sampled per frame, simplified, and flattened to `fox.montagent.json` (48 elements, 643 x/y/rotation keyframes). |
| Reference | `ref_render.py` | The same `pose(t)` drawn with PIL, straight to ffmpeg. |
| Check | `compare.py` | PSNR per frame between the two renders, plus `stills/compare.png`. |

Keys come from a git-ignored `.env` at the repo root (`AZURE_IMAGE_*`, `AZURE_SPEECH_*`) and are not in this branch.

## Findings

See the resolution comment on #487.
