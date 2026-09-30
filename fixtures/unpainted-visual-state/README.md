# `unpainted-visual-state` — a visual state the frame grid never paints

This project exists because **the committed `en-halloween-decorating` fixture has zero
unpainted visual states** ([ADR-0105](../../docs/adr/0105-the-sheets-refusals-are-invocation-errors-its-blind-spots-are-not-findings-and-an-unpainted-state-is-quantization.md) §6),
so a code that fires on one would otherwise have an untested emission path. ADR-0105 made a
constructed instance a shipping condition, and [#437](https://github.com/MBehtemam/Montagent/issues/437)
commits it. `frame`'s range mode ([#488](https://github.com/MBehtemam/Montagent/issues/488))
uses it too.

At 25 fps, over a whole-span `bg`:

| element | track | range | what it is for |
| --- | --- | --- | --- |
| `bg` | `bg` | 0..2000 | Present throughout, so the unpainted state is not empty. |
| `a` | `a` | 0..1010 | Leaves at 1010 ms. |
| `b` | `b` | 1030..2000 | Enters at 1030 ms, on a different track from `a`. |

The visual state `1010..1030 {bg}` is declared and never shown: frames paint at 1000 and
1040 ms, and neither falls inside it. No element rounds out of existence and no track gains
a gap, so neither of ADR-0006's first two `N-QUANTIZATION` conditions reaches it. It is the
third condition, and `validate` reports it as one `N-QUANTIZATION` at `review`. That is the
fixture's only finding.

It uses no media, so `validate` needs no `ffprobe` to read it.
