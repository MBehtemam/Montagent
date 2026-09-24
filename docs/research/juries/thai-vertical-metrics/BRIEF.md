# Brief: does Thai's `line_height` collision have a repair inside the format?

You are one of three jurors, each a different model, each blind to the others and to the
author's conclusion. **Do not read** `docs/research/thai-vertical-metrics.md`, `docs/adr/0087-*`,
`crates/montagent-text/src/ink.rs`, `crates/montagent-core/src/checks/ink.rs` or
`crates/montagent-core/tests/ink.rs`. If you open one by accident, say so in your ballot.

You **may** read: the ticket, any ADR numbered 0086 or below, the rest of the source tree, the
font binaries, and the OpenType spec.

## The situation

[ADR-0007](../../../adr/0007-text-runs-literal-size-declared-fonts.md) makes a line's height
*"the largest `size` among the runs on that line × `line_height`"* — a function of two numbers
the document declares, **never of the font's ink**. On a script whose marks stack (Thai: base +
upper vowel + tone mark + lower vowel) a `line_height` tuned on Latin can put one line's ink
through the next line's with every field individually valid.
[#130](https://github.com/MBehtemam/Montagent/issues/130) measured that the assumption breaks
and stopped there by design. [#325](https://github.com/MBehtemam/Montagent/issues/325) named
three candidate repairs and evaluated none:

1. **Read the font's `OS/2` typo ascent/descent** instead of whatever `hhea`-derived value
   `skrifa`/parley currently reports.
2. **A script-aware `line_height` floor** — which the format has no way to express today, so it
   implies a schema question as well as a computation one.
3. **Accept it as a font-selection problem**, pushed onto whoever vendors a Thai-capable face
   under [ADR-0057](../../../adr/0057-font-vendoring-licence-gate-and-path-keyed-attestation.md)'s
   licence gate.

## Your materials

Two OFL Thai faces are fetched (not committed) by
`docs/research/prototypes/thai-vertical-metrics/run.sh`. Run it, or fetch the faces yourself:

- **Noto Sans Thai Regular 2.002**, from `notofonts/notofonts.github.io`
- **Sarabun Regular 1.000**, from `google/fonts`

The workspace pins `skrifa = "=0.46.2"` and `parley = 0.11.1`. Both crates' sources are in
`~/.cargo/registry/src/`.

## Answer these four, each with the evidence you gathered yourself

1. **Candidate 1.** Which table does this repo's stack actually read for a line's ascent and
   descent — `hhea` or `OS/2` typo? Name the function and file in the dependency source. Then:
   on each of the two faces, what are `hhea` ascender/descender/lineGap, `OS/2`
   sTypoAscender/sTypoDescender/sTypoLineGap, `usWinAscent`/`usWinDescent`, and is `fsSelection`
   bit 7 set? **Would candidate 1 change any rendered pixel?** Show the numbers.

2. **Candidate 2.** For each face, at size 55, find the lowest `line_height` tenth at which two
   consecutive lines of full-stack Thai do **not** overlap in real ink. Is that number a
   property of the *script* or of the *face*? What does your answer imply about a schema floor?

3. **Candidate 3.** If the format declines to repair this, what — if anything — should the tools
   do instead? Be specific about verb, class and whether anything should be refused. If you
   think the correct answer is "nothing", say so.

4. **The question none of the three asks.** Is there a repair, or a defect, that #325's framing
   misses entirely? Look at the **single-line** case as well as the multi-line one.

## Form

Your ballot is one Markdown file. Lead with a one-paragraph verdict, then the evidence per
question. Tag every claim **MEASURED** (you ran it), **READ** (you read it in a spec or source)
or **INFERRED**. Never present an inference as a measurement. If you could not measure
something, say what blocked you rather than estimating.

You are being asked to attack, not to ratify. A ballot that finds nothing is a real outcome;
a ballot that invents something to find is worse than useless.
