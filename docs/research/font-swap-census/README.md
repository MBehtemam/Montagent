# Font-swap census: Open Runde against SF Pro Rounded

Evidence for [ADR-0085](../../adr/0085-the-font-swap-census-holds-and-the-text-mask-is-permanent.md),
resolving [#186](https://github.com/MBehtemam/Montagent/issues/186) and discharging the
ADR-0007 census obligation [#143](https://github.com/MBehtemam/Montagent/issues/143) left open.

**Re-executable:** `cargo test -p montagent-core --test font_swap_census -- --nocapture`.
Every number below is asserted there, so the census fails the suite the moment it stops
reproducing. The SF Pro Rounded column needs that face installed at
`/Library/Fonts/SF-Pro-Rounded-Bold.otf` (a stock macOS has it); where it is absent the
delta half skips with a printed note and the Open Runde half still asserts in full. The
face is deliberately **not** vendored — that it cannot be is the whole reason #143 happened
(ADR-0057).

## Why the measurement goes through `measure`

The other committed evidence here is stdlib-only Python, so it runs on a bare interpreter.
That is right for arithmetic over a PNG and wrong for this: a hand-rolled OTF parser would
measure *a* number, not the number this build shapes and draws. The census goes through the
`measure` verb, whose output *is* the text engine's output (spec #168) — the same choice
`tests/effects.rs` makes when it measures the mask by rendering it.

## The two questions, which are independent

#186 was written around a conflation, and the measurement separates it:

1. **Do the fixture's declared layouts still hold?** **Yes, with margin.** Nothing
   overflows, no line partition moves, no block height changes.
2. **Can the reference-frame gate therefore include text?** **No — and no census could ever
   make it so.** A declared box is a *layout* claim; the gate makes a *pixel* claim.

## 1. The layout census — all 22 elements hold

Sorted by how much of its declared `width` each element actually uses, tightest first.

| element | size | declared `width` | Open Runde | SF Pro Rounded | OR wider by | uses | lines | block h / declared |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `quiz-question` | 73 | 984 | 903.66 | 839.75 | 7.6% | 91.8% | 2 | 161 / 161 |
| `handle-text` | 34 | 472 | 428.66 | 394.20 | 8.7% | 90.8% | 1 | 38 / 84 |
| `word-05` | 88 | 984 | 807.94 | 750.02 | 7.7% | 82.1% | 1 | 97 / 97 |
| `word-quiz` | 88 | 984 | 807.94 | 750.02 | 7.7% | 82.1% | 1 | 97 / 97 |
| `sentence-05` | 55 | 984 | 805.84 | 744.25 | 8.3% | 81.9% | 1 | 61 / 169 |
| `sentence-quiz` | 55 | 984 | 805.84 | 744.25 | 8.3% | 81.9% | 1 | 61 / 169 |
| `hook-05` | 88 | 984 | 788.56 | 723.64 | 9.0% | 80.1% | 2 | 194 / 194 |
| `hook-loop` | 88 | 984 | 788.56 | 723.64 | 9.0% | 80.1% | 2 | 194 / 194 |
| `word-07` | 80 | 984 | 784.72 | 719.18 | 9.1% | 79.7% | 1 | 88 / 88 |
| `chip-text` | 52 | 238 | 187.08 | 172.86 | 8.2% | 78.6% | 1 | 58 / 84 |
| `sentence-08` | 55 | 984 | 709.12 | 651.73 | 8.8% | 72.1% | 1 | 61 / 169 |
| `word-06` | 88 | 984 | 669.88 | 612.35 | 9.4% | 68.1% | 1 | 97 / 97 |
| `intro-title` | 58 | 984 | 604.82 | 558.33 | 8.3% | 61.5% | 2 | 128 / 128 |
| `sentence-07` | 57 | 984 | 528.04 | 486.36 | 8.6% | 53.7% | 2 | 126 / 169 |
| `sentence-06` | 57 | 984 | 509.28 | 470.67 | 8.2% | 51.8% | 2 | 126 / 169 |
| `word-08-target` | 49 | 984 | 347.58 | 318.40 | 9.2% | 35.3% | 1 | 54 / 54 |
| `word-08-bridge` | 35 | 984 | 248.27 | 227.43 | 9.2% | 25.2% | 1 | 39 / 39 |
| `count-4` | 249 | 984 | 168.80 | 161.22 | 4.7% | 17.2% | 1 | 274 / 274 |
| `count-3` | 249 | 984 | 164.20 | 156.84 | 4.7% | 16.7% | 1 | 274 / 274 |
| `count-5` | 249 | 984 | 160.49 | 155.62 | 3.1% | 16.3% | 1 | 274 / 274 |
| `count-2` | 249 | 984 | 156.86 | 150.15 | 4.5% | 15.9% | 1 | 274 / 274 |
| `count-1` | 249 | 984 | 121.85 | 119.52 | 2.0% | 12.4% | 1 | 274 / 274 |
**Nothing overflows.** The tightest element is `quiz-question` at **91.8%** of its declared
width — a face **8.9% wider than Open Runde** would be the first to overflow anything here,
and Open Runde is itself only 7.6% wider than SF Pro Rounded on that element.

**No line partition moved** under the swap, and none could: ADR-0008 forbids auto-wrap, so
`line_count` is the author's own `\n` count and a wider face overflows rather than
re-wrapping. Asserted rather than assumed, because it is the premise the census rests on.

**No block height moved**, and none could: ADR-0028's `ceil(size × line_height ×
line_count)` carries no font term. Also asserted rather than assumed.

So the only font-dependent axis is the width term — the one
[ADR-0014](../../adr/0014-stroke-is-paint-the-text-box-is-required.md) parks as `UNCHECKED`
pending measurement. For this fixture it is now checked, and it passes under both faces.

## 2. The pixel divergence — why text still cannot gate

Open Runde's advances run **2.0%–9.4% wider** than SF Pro Rounded's across the 22 elements
(the narrow end is the large tabular digits of the countdown; the body text sits at
7.6%–9.4%). Every line in this fixture is centred or centre-left, so a line 8% narrower is
not merely shorter — every glyph on it lands somewhere else.

`crates/montagent-core/tests/reference_frames.rs` already measures the consequence on every
run, and reports it without gating it:

| frame | text region SSIM | drawn region beside it | text share of frame |
| --- | ---: | ---: | ---: |
| `frame-intro.png` at 400 ms | **0.6743** | 0.9874 (gate 0.975) | 14.2% |
| `frame-05-at-11s.png` at 14 000 ms | **0.6597** | 0.9646 (gate 0.953) | 18.8% |

A region at 0.66 cannot join a gate set at 0.95+. No threshold admits it and stays
sensitive to anything — which is exactly the *"completed, looked plausible, was wrong"*
failure [ADR-0010](../../adr/0010-skia-safe-rasterizer-text-beside-it.md) records this
project having had twice.

## What #186 predicted, and where it was wrong

The ticket already struck two of its own three original justifications, both traced to one
mistake — *"assuming 'text metrics' means 'font metrics'"*. The measurement strikes a third
claim, from the same family. #186 offered two outcomes:

> **They still hold** (the deltas are inside every threshold). Record it, cite the script,
> and the falsification test can treat text regions as gating.

The first half is right and the second does not follow. The deltas *are* inside every
declared threshold, and text still cannot gate, because fitting a declared box and matching
a reference pixel are different claims about different things. A face could fit every box
perfectly and still draw every glyph in a different place.

`CONTEXT.md`'s Reference frame entry says a divergence is *"masked only while its cause is
unfixed."* The cause here is a licence (ADR-0057), and a licence does not get fixed. The
text mask is **permanent**, not provisional — which is what ADR-0085 records, and the
substantive correction this ticket produced.
