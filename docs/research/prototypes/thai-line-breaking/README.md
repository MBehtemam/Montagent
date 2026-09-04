# Thai line breaking: cosmic-text vs parley

Resolves [#27](https://github.com/MBehtemam/Montaget/issues/27). Read
[FINDINGS.md](FINDINGS.md) first — this file is only how to re-run it.

```sh
./run.sh          # regenerates results/ and frames/ from scratch (~1 min)
```

macOS only, because the oracle is `CFStringTokenizer` and the fonts are the
system's. Needs a Rust toolchain and `swiftc`.

## Layout

| Path | What it is |
| --- | --- |
| `samples.rs` | **The one shared input.** Nine paragraphs, each with its font, size, box width, and a written-before-running `expect`. Neither probe defines its own text, so a difference in the output is the text stack and not the scene. |
| `render.rs` | **The one shared rasterizer.** Both probes hand it positioned glyphs plus the font blob; it draws them with `skrifa` outlines into a `tiny-skia` pixmap. Neither stack rasterizes its own output, so a difference between two PNGs is the line breaking and not the renderer. |
| `cosmic-probe/` | cosmic-text 0.19.0. `cosmic-probe <Word\|WordOrGlyph\|Glyph> [width] [png-dir]` |
| `parley-probe/` | parley 0.11.1, built twice — the `complex-scripts` Cargo feature is the *only* difference between the `parley-off-*` and `parley-on-*` outputs. `parley-probe <Normal\|BreakWord\|Anywhere> [width] [png-dir]` |
| `oracle.swift` | Independent word-boundary oracle: macOS `CFStringTokenizer`. Not parley, not cosmic-text, and not this prototype's author — needed because the author does not read Thai, Khmer or Lao. |
| `compare.py` | Diffs each stack's break-opportunity set against the oracle. |
| `bench_startup.py` | Interleaved whole-process timing. |
| `results/` | Every output, committed. |
| `frames/` | Every rendered PNG, committed. |

## How break *opportunities* are measured

The probes take a box width. At the real width (420px) they show where the
lines fell. At `width=1` every break opportunity is forced to become a break,
so the list of line starts **is** the opportunity set — measured the same way
through both stacks, with no stack-specific API for it.
