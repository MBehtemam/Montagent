# Thai vertical metrics: `OS/2` typo metrics vs a `line_height` floor vs font selection

Research for [#325](https://github.com/MBehtemam/Montagent/issues/325). Read
[`../../thai-vertical-metrics.md`](../../thai-vertical-metrics.md) first — it
is the findings; this file is only how to re-run them.

```sh
./run.sh     # ~20s on a warm cargo cache
```

`run.sh` fetches the two OFL-1.1 Thai faces from their upstream repos, **refuses
to continue if either sha256 differs from the pinned one**, rebuilds the probe,
regenerates `results.txt` and `frames/`, and then re-derives every headline
number the write-up states, exiting non-zero the moment one stops reproducing
(`docs/agents/domain.md`'s evidence rule).

Needs a Rust toolchain and network access on first run. The font files are
fetched into `fonts/`, which is git-ignored — the pins in `run.sh` are the
committed record of exactly which bytes were measured.

## Layout

| Path | What it is |
| --- | --- |
| `run.sh` | Fetch (hash-pinned), build, run, and re-assert every headline number. |
| `probe/src/main.rs` | The probe. Part 1 dumps each face's raw `hhea` / `OS/2` sTypo* / `OS/2` usWin* / fsSelection bit 7, alongside what `skrifa::metrics::Metrics::new` resolves. Part 2 lays real Thai out through parley (`complex-scripts` on), places baselines with [ADR-0029](../../../adr/0029-line-baseline-half-leading.md)'s half-leading formula, extracts every glyph's real ink from its skrifa outline, and sweeps `line_height` over [ADR-0028](../../../adr/0028-text-block-arithmetic-is-exact-tenths.md)'s tenths looking for the first one with no ink-to-ink collision. |
| `results.txt` | Full numeric output, committed. |
| `frames/` | Rendered PNGs, committed. Blue lines mark each line's nominal slot boundary. |

## Relationship to #130's probe

This is [#130](https://github.com/MBehtemam/Montagent/issues/130)'s probe
(branch `prototype/thai-vertical-metrics`) extended along the three axes that
ticket recorded as unsettled: a **vendorable** face instead of the macOS system
font Ayuthaya, the **`OS/2` vs `hhea` question** answered from the binaries, and
a **sweep** over the tenths instead of two spot values. The Thai strings and the
measurement method are unchanged, so the numbers are comparable.

## Method notes

- Each `\n`-delimited line gets its **own single-line parley layout**, because
  [ADR-0008](../../../adr/0008-line-breaks-belong-to-the-agent.md) makes
  Montagent the owner of the line partition: the renderer hands the shaper one
  line at a time. #130's probe recorded the same choice and the same reason.
- "Ink" is the glyph **outline** bounding box, not the advance box — the same
  "measure the real ink, don't trust nominal metrics" methodology ADR-0011 used,
  done from vectors rather than by pixel-scanning, which is exact rather than
  sampled.
- The probe registers the font bytes into parley's collection **by path** and
  never consults the system font set, per
  [ADR-0007](../../../adr/0007-text-runs-literal-size-declared-fonts.md).
