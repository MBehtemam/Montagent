---
status: accepted
amends: 0011 (redefines the `alpha` the probe returns — it is the file's reading, not the pixel format's — and adds `codec_name` to the returned facts, because the decode path cannot probe), 0069 (bumps the sidecar to version 2 rather than defaulting the new shape, since a version-1 entry holds an `alpha` this ADR establishes is wrong), 0088 (`R-CHROMA-ON-ALPHA-SOURCE` now reaches sources whose alpha is a side stream; its reading was the pixel format's and silently skipped every one of them)
---

# Source alpha is a file-level reading, and VP9-in-WebM needs its own decoder

[ADR-0040](./0040-effect-model-attachment-and-v1-vocabulary.md) put chroma key out of
scope, and although [ADR-0088](./0088-chroma-is-a-matte-operation-and-color-stays-literal.md)
has since admitted it, the pre-keyed source — a subject arriving with its own alpha, declared
as an ordinary `video` element — remains the way a cutout composites without a key at all.
[#338](https://github.com/MBehtemam/Montagent/issues/338) and
[#339](https://github.com/MBehtemam/Montagent/issues/339) found that path broken in two
places for one of the two formats anyone ships it in, and broken **silently** in both.

## What was measured

`docs/research/alpha-decode/` — `FINDINGS.md` and `alpha_decode_scan.sh` beside it, which
re-derives every claim below against `ffmpeg` and exits non-zero the moment one stops
reproducing. It asserts **both directions**: the ProRes case that already worked fails the
script as loudly as the VP9 case that did not.

| Source | `ffprobe` says | Probe answered | Default decode |
| --- | --- | --- | --- |
| ProRes 4444 | `pix_fmt=yuva444p12le` | `alpha: true` | alpha survives, pixel-exact |
| VP9-in-WebM, alpha | `pix_fmt=yuv420p`, `codec_name=vp9`, tag `alpha_mode=1` | **`alpha: false`** | **every pixel opaque** |
| VP9-in-WebM, no alpha | `pix_fmt=yuv420p`, `codec_name=vp9`, no tag | `alpha: false` | correct |

Neither failure produced an error. An agent compositing a VP9 cutout got an opaque
rectangle over whatever was beneath it and no finding said why — and an agent that did the
right thing and *asked* the probe first was told `false`, with no hedge. That is the class
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) exists to make impossible.

## The decision

### 1. `alpha` is the file's reading, and it carries the signal that settled it

The pixel format provably cannot answer *"does this source carry alpha"* for VP9, which
carries it in a side stream. The container can and does: Matroska's `AlphaMode` element,
which `ffprobe` surfaces as an `alpha_mode` stream tag, present on the alpha file and
absent on the flat one.

So `Probe::alpha` answers the question every consumer was already asking, from the pixel
format **or** the container declaration — and it reports which:

```rust
pub struct SourceAlpha { pub carries: bool, pub source: AlphaSource }
pub enum AlphaSource { None, PixelFormat, ContainerDeclaration }
```

The pair rather than a bare `bool`, for
[ADR-0023](./0023-video-source-dimensions-par-and-rotation.md)'s reason about rotation:
*"the answer travels with the number rather than being reconstructed by whoever reports
it."* Two signals of unequal strength answer this question — a pixel format is a property
of the decoded stream, a container declaration is the muxer's word about a side stream —
and this reading is cached to disk under a version number, where a bare `true` is a claim a
later binary cannot check. `SourceDimensions` already travels with the inputs that produced
it and `Rotation` already names whether a display matrix or a rotate tag settled it; this is
that idiom, not a new one.

**The pixel format is consulted first** and the declaration is the fallback, so a muxer's
tag can never overrule the stream itself.

**The rejected alternative was a second field** — keeping `alpha` strictly about the pixel
format and adding the file-level answer beside it. It preserves a reading that has no
consumer, and makes two adjacent alpha-ish fields an agent picks wrong from at a glance;
`R-CHROMA-ON-ALPHA-SOURCE` reaching for the obvious name is the evidence that it would. The
datum survives as provenance instead.

### 2. `codec_name` travels out of the probe, because the decode cannot ask

`montagent-core` depends on `montagent-render`, not the reverse, so `decode.rs` cannot
probe. A second codec authority spawning its own `ffprobe` at the decode site is the thing
[ADR-0011](./0011-tool-surface-reads-checks-renders.md) spent itself removing. The codec
therefore joins the facts the one authority returns, and the decode takes its answer as an
argument.

### 3. The decoder is forced only for VP9 *with* alpha

`-c:v libvpx-vp9` recovers the alpha completely. It is asked for **only** where the source
is VP9 and its alpha is a side stream — never for VP9 at large.

That narrowness is the decision, not an implementation detail.
[ADR-0009](./0009-binary-plus-user-supplied-ffmpeg.md) ships Montagent as *"a binary, plus
an `ffmpeg` the user supplies"*, and libvpx is exactly the optional piece that argument is
about: an `ffmpeg` built without it has no `libvpx-vp9` decoder. Forcing it on every VP9
source would turn renders that work today into hard failures on those machines. Narrowed to
the alpha case it can only fail where the alternative was a silently wrong picture — and
ADR-0006 prefers the loud error to that.

### 4. The sidecar goes to version 2

Every entry written at version 1 holds the old shape, including — for a VP9-alpha source —
an `alpha` of `false` that this ADR establishes is wrong. A `#[serde(default)]` would keep
those entries and keep serving that answer.
[ADR-0069](./0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md)
already says an unrecognised version reads as an empty cache with no migration, so the
version carries it: every machine re-probes once, and no cached wrong answer outlives the
fix. This is the opposite call from #206's font half, which defaulted, and for the opposite
reason — that section's absence was *true* of a file written before it existed, where this
one's contents are false.

## Consequences

- `R-CHROMA-ON-ALPHA-SOURCE` now fires on pre-keyed VP9 sources. It never did, so an agent
  keying a source that was already keyed was told nothing. This is a check reaching further,
  not a new one.
- The `probe` verb's JSON prints `alpha` as `{"carries": …, "source": …}` where it printed a
  bare boolean, and gains `codec_name`. The text report is unchanged; it never printed alpha.
- Every machine re-probes once on first run after this lands.
- **Not decided here:** whether any other codec hides alpha where the pixel format cannot
  see it. `AlphaSource::ContainerDeclaration` is spelled generally and the `alpha_mode`
  lookup is not VP9-specific, but VP9 is the only case measured, and ADR-0003's asymmetry
  says that silence is not evidence there are no others.

## Evidence

- `docs/research/alpha-decode/FINDINGS.md` — the measurement, with its method note.
- `docs/research/alpha-decode/alpha_decode_scan.sh` — re-derives every claim against
  `ffmpeg`, both directions, exits non-zero when one stops reproducing.
- `crates/montagent-core/tests/source_alpha.rs` — the same claims through *Montagent's* own
  probe and decode rather than through raw `ffmpeg`, including the defect itself pinned, so
  that a future `ffmpeg` whose native `vp9` decoder gains the capability fails loudly and
  sends the next reader back to this ADR rather than to the conditional.
