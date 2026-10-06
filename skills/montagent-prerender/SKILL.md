---
name: montagent-prerender
description: "Bring a look drawn in code into a Montagent video as lossless footage with alpha, plus a recipe that rebuilds it. Load when the look is not an `effects` member (distortions, mosaic, lens flare, a particle or shader piece) or the brief asks for generative or code-drawn visuals."
---

# Pre-render: a code-drawn piece as footage

Montagent has no shader or script file ([ADR-0017](https://github.com/MBehtemam/Montagent/blob/main/docs/adr/0017-closed-schema-no-escape-hatch.md)). A look outside the `effects` list is drawn in your own code, encoded once as footage, and placed as an ordinary `video`. The format, the schema and the painter stay as they are ([ADR-0156](https://github.com/MBehtemam/Montagent/blob/main/docs/adr/0156-four-named-effects-join-the-effects-list-and-a-code-drawn-piece-enters-as-pre-rendered-footage.md) §7).

## When to reach for it

Take the named route first. Read the `effects` list in `montagent://schema/index.json`: when a member makes the look (blur, shadow, tint, a `blend`, a keyed `mask`), use it. Pre-render only when no member, paint or blend composes the look: distortions, mosaic, lens flare, particles, a procedural pattern.

A pre-rendered piece is fixed footage. Its size, fps and length are set when you render it. Changing the look means rendering again, so settle the brief's size, fps and duration first, and match the project's `fps`.

## The steps

Run every step, in order. The piece is **placed** when step 5 passes.

1. **Draw.** Write code, in any tool, that produces RGBA frames of a fixed `width`, `height` and `frames` count: transparent where nothing is drawn. It writes raw RGBA to stdout, or PNG files `f00000.png`, `f00001.png` and so on into the folder named by `PRERENDER_FRAMES_DIR`. Read its size from `PRERENDER_WIDTH`, `PRERENDER_HEIGHT`, `PRERENDER_FPS` and `PRERENDER_FRAMES`. Use integer arithmetic or a fixed seed, so a second run draws the same bytes. [The sample piece](references/sample_piece.py) is the smallest working one. <!-- guard-ok: PRERENDER_FRAMES_DIR PRERENDER_WIDTH PRERENDER_HEIGHT PRERENDER_FPS PRERENDER_FRAMES frames -->
2. **Spec.** Write a build spec next to the code: `name`, `out` (the project's media folder), `width`, `height`, `fps`, `frames`, `render` (the argv that runs your code), `files` (every file the code needs), and `input` of `"png"` when it writes PNGs. Run `python3 scripts/prerender.py --help` for every key. Add `versions` for any library whose version changes the pixels (a canvas library, a headless browser). <!-- guard-ok: versions name out render files input -->
3. **Build.** `python3 scripts/prerender.py build <spec>`. It runs the code, encodes **one** `<name>.mov`, decodes it, and fails on the first frame that differs from what your code drew. Read its last line: `N frames, decoded = drawn`.
4. **Place.** Add the footage to the project as a plain `video` element (below), or run `python3 scripts/prerender.py place <project> <placing>`, where `<placing>` is a JSON file `{"source": "media/<name>.mov"}` plus any of `id`, `at`, `x`, `y`, `origin`, `width`, `height`, `fit`, `track`, `layer`. It prints the project with the element added, so redirect that to a new file. <!-- guard-ok: at origin track layer fit -->
5. **Check.** `validate` the project and fix every error. Then `montagent probe` the footage and confirm it reports alpha (`carries: true`). Only then `frame` a tile where the piece is up and check the transparent parts show what is behind.

```json
{
  "frame": {"width": 640, "height": 360}, "fps": 25, "background": "#101418", "duration": 480, "output": "out/pulse.mp4",
  "tracks": [
    {"name": "prerender", "layer": 10, "elements": [
      {"id": "pulse", "type": "video", "start": 0, "end": 480, "source": "media/pulse.mov", "source_start": 0, "source_end": 480, "volume": 0, "x": 320, "y": 180, "origin": "center", "width": 320, "height": 240, "fit": "contain"}
    ]}
  ]
}
```

The element is an ordinary `video`: key `opacity`, `scale` and `position`, add `effects` and a `blend`, and lay it on a track like any clip. The footage has no sound, so `volume` is 0. Its `end` minus `start` is the footage's length; `source_end` is the same length. <!-- guard-ok: position -->

## The codec

Footage is **PNG in a MOV**, `-c:v png -pix_fmt rgba`, and `build` writes exactly that. It is the codec checked to return every byte of RGBA, alpha included, through Montagent's decoder. Keep `-pix_fmt rgba` explicit: left to itself, ffmpeg can pick `rgb24` and drop the alpha. <!-- guard-ok: build rgb24 -->

Any `yuva` pixel format is not exact from RGBA: FFV1 `yuva444p` and VP9 lossless both change pixels, and ProRes 4444 is lossy. A lossy or `yuva` encode makes the recipe's hashes meaningless, so do not swap one in. FFV1 with `bgra` also round-trips and is smaller on noisy frames, but this skill uses PNG. <!-- guard-ok: yuva yuva444p rgb24 bgra -->

If `build` stops on a missing `png` encoder, the installed ffmpeg is unusual: install a standard build. The encoder is built in by default. <!-- guard-ok: build png -->

## The recipe

`build` writes `<name>.recipe/` beside the footage. It holds a copy of the code, and `recipe.json`: the commands, the tool versions, the size, fps and frame count, and the hash of every **decoded** frame, read with `-f rawvideo -pix_fmt rgba`. The project never names the recipe or the code. <!-- guard-ok: build -->

To rebuild, or to check the footage still matches what its recipe made:

```sh
python3 scripts/prerender.py rebuild <name>.recipe
```

It re-runs the code from the recipe's own copy, encodes again, and compares decoded-frame hashes, never file bytes (container metadata changes the bytes and not the pixels). It prints `rebuild: match` and `footage: match`, or the first frame that differs and exits 1. After you edit the code, copy it into the recipe by running `build` again. A mismatch with the code unchanged means a tool version changed: compare `versions` in `recipe.json` against the machine. <!-- guard-ok: build versions -->
