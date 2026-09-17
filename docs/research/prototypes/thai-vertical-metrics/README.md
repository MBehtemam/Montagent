# Thai vertical stacking vs `line_height`/`box`

Resolves [#130](https://github.com/MBehtemam/Montaget/issues/130). Read
[FINDINGS.md](FINDINGS.md) first — this file is only how to re-run it.

```sh
cd probe
cargo run --release -- ../frames    # regenerates results and PNGs (~5s after first build)
```

Needs a Rust toolchain and the macOS system font `Ayuthaya` (Supplemental Fonts).

## Layout

| Path | What it is |
| --- | --- |
| `probe/src/main.rs` | The whole prototype: lays real Thai and Latin-control paragraphs out through parley (`complex-scripts` on, matching #27/#28's accepted config), places every line's baseline with [ADR-0029](../../../adr/0029-line-baseline-half-leading.md)'s half-leading formula, then extracts every glyph's real ink extent from its `skrifa` outline (not its advance box) and compares it against the format's `size × line_height` slot ([ADR-0007](../../../adr/0007-text-runs-literal-size-declared-fonts.md)/[ADR-0028](../../../adr/0028-text-block-arithmetic-is-exact-tenths.md)). |
| `results.txt` | Full numeric output, committed. |
| `frames/` | Rendered PNGs, committed. Blue lines mark each line's nominal slot boundary. |

## Method note

Each `\n`-delimited line gets its **own single-line parley layout**, rather than
handing the whole paragraph to parley and letting it find the hard breaks.
That mirrors [ADR-0008](../../../adr/0008-line-breaks-belong-to-the-agent.md)
exactly: Montaget owns the line partition and hands the shaper one line at a
time, so this measures what the renderer will actually do — and it sidesteps
an oddity hit while building this: handing the whole `\n`-joined paragraph to
parley's own `break_all_lines(None)` in one call silently produced a single
un-split line for every Thai sample while correctly splitting the identical-
shape Latin control. Not investigated further (root cause unknown — could be
this prototype's own misuse of the API, could be script-dependent behaviour in
parley's hard-break handling) because this project's own design already
requires per-line shaping regardless of what the shaper's multi-line API does.
