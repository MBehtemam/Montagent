---
status: accepted
amends: 0015 (discharges the deferred "PAR and video source dimensions" clause; "source dimensions" becomes one type-generic definition instead of an image-only one)
---

# Source dimensions generalise to video: rotation resolves, then PAR, then one integer

> **Amended by [ADR-0069](./0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md)**,
> which answers the *"PAR provenance for the probe sidecar"* sub-question parked
> under **Not settled here** below: **yes** — the resolved rotation-applied
> dimensions and the file's own probed `par` are both stored in the sidecar. The
> probe that fills an entry has already computed them, so the choice was never
> between storing them and not paying for them, but between storing them and
> throwing them away.

[ADR-0015](./0015-fit-is-a-derivation-claim-and-gravity-retires.md) defined "source
dimensions" — the input to `fit`'s `cover`/`contain` arithmetic — as *decoded,
orientation-applied integer pixel dimensions*, and explicitly deferred non-square pixel
aspect ratio (PAR): "Raster images are square-pixel; a PAR rule written now would be
untestable speculation with no element to exercise it." [ADR-0003](./0003-general-video-editor-not-channel-tooling.md)
commits Montagent to video clips, so that deferral's own stated condition has lapsed —
[#53](https://github.com/MBehtemam/Montagent/issues/53) asks the question the deferral
postponed.

Decided by a three-question, three-model independent court (Opus, Haiku, Fable — one
juror per model per question, no persona, no assigned stance), following an interview
round that produced the same recommendations the court then cross-examined. Full ballots
in [#53](https://github.com/MBehtemam/Montagent/issues/53).

## Source dimensions are one definition, not two

**"Source dimensions" is a single, type-generic pipeline: decode, resolve rotation,
apply PAR, round to one integer pair.** Images are the degenerate case — PAR is `1:1`
and there is exactly one rotation signal (EXIF) — so ADR-0015's rule is not replaced, it
is the general rule with the video-only stages collapsed to no-ops. Nothing about images
changes; zero fixture bytes move.

Two rules dispatched by source type were rejected 3–0. `fit` is a provenance claim about
*what the author was looking at when they computed the rect*, and that is one concept
regardless of container format. Splitting it would mean the same `fit: cover` on the same
declared rect means different arithmetic depending on a sibling field — the "same
defect wearing a new costume" pattern ADR-0015 already diagnosed in `speed`, and it would
force every future raster source (an image sequence, a generated frame) to add a new
branch and a new ADR rather than an identity stage in an existing pipeline.

### Rotation resolves first, and it is the container's track-level transform — not a "container vs. stream" vote

Video can carry a rotation/orientation signal in more than one place. The rule is not
"container wins a conflict" stated as an abstract precedence — real demuxers don't
surface it as a conflict to referee. The rule is narrower and matches what every
mainstream player actually does: **apply the container's track-level display transform
(e.g. an MP4 `tkhd` matrix); codec-level orientation metadata (SEI / VUI display
orientation) is not consulted for this purpose.** A portrait phone clip is the motivating
case — every player reads the container transform to show it upright, so that is the
dimension an author is looking at when they write `width`/`height`. Picking the
codec-level signal instead would make ordinary portrait phone footage the failing case,
which is the most common video this tool will see.

`validate` must print the dimensions it used, same as ADR-0015 already requires for EXIF
orientation. Where the container carries no display transform, no rotation is applied.

### Then PAR, as an exact rational, carried explicitly on the element

**PAR is applied, not ignored.** The magnitude argument from ADR-0015 carries: PAR
divergence between a reader that applies it and one that doesn't was measured at 33.4% on
a 4:3 worked case — three orders of magnitude past the 0.038% that got `speed` legislated,
and comparable in kind (not leaving silence) to the 60.5% that got EXIF orientation
legislated. Ignoring PAR is not the absence of a rule; it is a rule that silently produces
a rect stretched relative to what every player shows, and it would also contradict the
*renderer*, which decodes and must therefore make the identical choice — a `validate` that
ignores PAR while a render's decode path anywhere depends on the same normalization would
mean the format disagrees with itself. `fit` is only a provenance claim, but its author's
mental model of "what covers the box" is always the displayed geometry.

PAR is not silently re-probed from whichever container/bitstream layer an implementation
happens to read (H.264 VUI `sar`, MP4 `pasp`, and the container's DAR field can disagree
with each other) — that would just relocate the ADR-0015 divergence problem to "which PAR."
Instead:

- The element declares `par: [num, den]`, an exact-integer-pair rational, populated by an
  ingest/authoring tool from whichever layer it reads (`validate` may `note` when the
  file's own probed PAR disagrees with the declared value, mirroring ADR-0006's
  report-facts posture — it does not decide which is "right").
- Absent `par` defaults to `[1, 1]` — the image case, unchanged.
- `par` is a schema error on any element whose source is not video.

### Ordering and rounding

**Rotation resolves before PAR is applied**, not the reverse — applying PAR along the
wrong (pre-rotation) axis stretches the wrong dimension. The corrected dimension flows
through `cover`/`contain` as an exact rational and rounds exactly once, at the final rect,
by ADR-0013's existing floor rule. Two implementations that rounded at different pipeline
stages would diverge by a pixel under `validate`'s strict-equality check; this ADR forbids
that by naming the single rounding point.

## Mid-stream dimension changes: `UNCHECKED`, not an error, and not a new top-level category

A source whose stream dimensions change partway through has no single "source
dimensions" value to re-derive from at all — the fit-deviation check's precondition
fails, not its result. **This is `UNCHECKED`, at the same severity ADR-0013 already
defined for an unprobeable source** (missing file, permission denied, an unfetchable URL),
not a new top-level category and not a hard `error`.

The court was 3–0 against a hard error: `validate` answers "does this file agree with the
media on disk," and a stream with variable dimensions is not a defect in the *document* —
the declared rect may be exactly what the author intended, and `validate` has no basis to
demand a destructive re-encode of source media over a property the format never promised
uniform. Manufacturing an error `validate` cannot substantiate is the same failure ADR-0006
was written against, on the opposite side.

The panel split 2–1 on whether "unprobed" and "dimensions indeterminate" need distinct
finding codes (one juror argued the two have different remediations — one is fixable by
mounting a drive, the other never resolves for that source, and collapsing them tells an
author to keep retrying something permanent). This ADR does not grow the `UNCHECKED`
category to accommodate that: findings already carry a stable code and inline values
(#26), so the distinction — permanent versus transient unverifiability — belongs in that
finding's existing code and message, not in a new severity or category. Keeping the
top-level vocabulary small was itself a decided principle (the `sequence`-label and
`kind`-field rejections), and this defers to it rather than reopening it.

`fit: literal` is unaffected either way — it makes no derivation claim, so there is nothing
to suppress.

## Not settled here

- **PAR provenance for the probe sidecar.** Whether `par` (and the resolved
  rotation-applied dimensions) belong in the gitignored probe sidecar at zero extra I/O is
  a mechanical fact ([#4](https://github.com/MBehtemam/Montagent/issues/4)'s sidecar
  design), not decided here — this ADR only fixes what the *element* declares and what
  `fit`'s arithmetic consumes.
- **A real PAR/rotation-bearing video fixture.** As ADR-0015 recorded for EXIF
  orientation, this rule is argued and reasoned about, not measured against real footage —
  the fixture remains all-PNG. A synthetic case (a scan script, or a declared `par` on a
  video-backed element) is sufficient to exercise the arithmetic; it does not test real
  container/codec disagreement in the wild.
- **Non-integer PAR-corrected rounding edge cases beyond the stated single rounding
  point** — this ADR fixes *where* rounding happens, not a full re-derivation of
  ADR-0013's tiebreak reasoning for the video case, which is assumed to transfer
  unchanged.

## Consequences

- **ADR-0015's "source dimensions" definition is generalised**, not replaced: "decoded,
  orientation-applied integer pixel dimensions" becomes the image-only instance of
  "decoded, rotation-resolved, PAR-applied integer pixel dimensions." Zero fixture bytes
  change.
- **The schema gains `par` on video elements** — `[num, den]`, exact integers, defaulting
  to `[1, 1]`, and a schema error on any non-video element.
- **`validate` must print both the dimensions it used and which rotation source it
  applied** whenever the fit-deviation check fires on a video element, extending
  ADR-0015's existing print-what-you-used requirement.
- **The `UNCHECKED` category gains a new firing condition** (mid-stream dimension
  changes) at its existing severity, with no new top-level category.
