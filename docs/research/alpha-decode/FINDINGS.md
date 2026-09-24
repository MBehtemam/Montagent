# Source alpha through the decode path: one capability that works, two that fail silently

Measured 2026-09-24 against `crates/montagent-render/src/decode.rs` and
`crates/montagent-core/src/media/probe.rs` on `ffmpeg` 8.0.1 (Homebrew, arm64).
`alpha_decode_scan.sh` beside this file re-derives every claim below and exits
non-zero the moment any one of them stops reproducing. It asserts both
directions — the working case *and* the two failures — so a fix that quietly
breaks the working case fails the script just as loudly as a regression.

## Why this was measured

[ADR-0040](../../adr/0040-effect-model-attachment-and-v1-vocabulary.md) rules
chroma key out of scope, so the question *"how does a subject shot on a green
background composite over other elements"* has only one standing answer: the
source arrives already keyed, carrying its own alpha channel, and the document
declares it as an ordinary `video` element. Nothing had ever tested whether that
answer actually works.

The test reproduces `decode.rs`'s ffmpeg invocation verbatim rather than
approximating it, because the defect turned out to live in an argument that
invocation does **not** pass.

## What reproduces

### 1. ProRes 4444 alpha survives, pixel-exact — the capability already works

A `yuva444p10le` ProRes 4444 source decoded through `decode.rs`'s exact command
returns its alpha intact: 57 879 fully transparent pixels and 18 921 fully
opaque, matching the generated source exactly. It survives the `-vf scale`
filter too, with correctly antialiased edge alpha (17 distinct values at
160x120, where the unscaled frame has 2).

**A pre-keyed ProRes 4444 asset composites correctly today, with no format
change and no schema change.**

### 2. VP9-in-WebM alpha is silently dropped

`decode.rs` passes no `-c:v`, so ffmpeg auto-selects its native `vp9` decoder.
VP9 carries alpha as a separate side stream that the native decoder does not
surface, and every pixel returns opaque — corner alpha 255 where the source is
0.

Passing `-c:v libvpx-vp9` recovers the alpha completely (corner 0, centre 255).
The fix must be **conditional on the source codec**: forcing `libvpx-vp9`
unconditionally would break every non-VP9 source. The probe already runs before
decode, so the codec is knowable at the point the decision must be made — but
see finding 3, which is why it is not knowable *yet*.

### 3. `probe.rs` reports `alpha: false` for a file that carries alpha

This is a separate defect from finding 2, and fixing the decoder leaves it
standing.

`probe.rs:536` derives `Probe::alpha` from the pixel format alone, via
`pix_fmt_has_alpha()`. `ffprobe` reports `pix_fmt=yuv420p` for a VP9-alpha
WebM — the alpha lives in the side stream, not in the pixel format — so
`pix_fmt_has_alpha("yuv420p")` correctly answers `false` about the pixel format
while answering the wrong question about the file.

The pixel format provably cannot answer "does this source carry alpha" for VP9.
`codec_name` has to travel beside it.

## Why both failures matter more than their size suggests

Neither failure produces an error. An agent that composites a VP9-alpha cutout
gets an opaque rectangle covering whatever is beneath it, and no finding
anywhere says why — the exact class of silent wrongness
[ADR-0006](../../adr/0006-validate-reports-facts-and-render-enforces.md) exists
to make impossible. Finding 3 makes it worse than silent: an agent that
correctly asks the probe whether its source carries alpha is told `false`, and
told it with no hedge.

## Method note, recorded rather than glossed

The first fixture was wrong and produced a false negative. It built its alpha
with `drawbox ... color=red@1.0:t=fill`, which writes no alpha at all — the
generated clip was uniformly transparent, and the ProRes case therefore read as
"alpha lost" when the pipeline was fine. It was caught by sanity-checking the
generator's own output before the encode step, not by the test.

`alpha_decode_scan.sh` now asserts the generator first, so a fixture that stops
carrying alpha fails as a fixture error rather than as a finding about the
decoder.
