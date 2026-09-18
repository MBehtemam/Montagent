# ADR index

**The ADR series is the specification.** There is no separately assembled spec document, and there will not be one — [ADR-0031's ticket (#131)](https://github.com/MBehtemam/Montaget/issues/131) rejected that shape because a second artifact drifts from its source with no rule for which wins. This file is navigation only: it adds no decision, and where it summarises an ADR the ADR is what governs.

**ADRs are amended, never rewritten.** A later ADR that corrects an earlier one says so in its own text; the *Amended by* column here is the reverse view, so you can tell at a glance whether what you are about to read is still current. **An ADR with entries in that column has been touched by later decisions — read them before acting on it.** The two densest are ADR-0006 (16 amendments) and ADR-0011/ADR-0012 (14 and 12), which is expected: they are the validate report, the tool surface and the transform model, and nearly every later decision lands on one of them.

ADR-0050 and ADR-0065 appeared to contradict each other on `preview`'s floor (360p vs 540p). **Resolved by [ADR-0067](0067-two-floors-a-wall-clock-give-up-point-and-a-legibility-refusal.md)** ([#178](https://github.com/MBehtemam/Montaget/issues/178)): they are two different refusals sharing one word — a wall-clock give-up point at 540p and a legibility threshold at 360p. Both stand. Read ADR-0067 before implementing preview degradation.

---

## Scope

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0003](0003-general-video-editor-not-channel-tooling.md) | Montaget is a general video editor in the CapCut/Premiere class; After Effects is out. The channel is a fixture and a regression guard, **never a scope boundary** — evidence a capability is needed, never evidence one is unneeded | — |

## Host, renderer and distribution

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0009](0009-rust-host.md) | The host is Rust, on the stdio MCP binding — one subprocess per session, startup paid once | — |
| [0010](0010-skia-safe-rasterizer-text-beside-it.md) | `skia-safe` rasterizes; text lays out beside it; FFmpeg stays a subprocess the user supplies. `tiny-skia` is the named exit, never a second backend | 0040 |
| [0064](0064-packaging-cargo-and-releases-all-six-targets-passive-updates.md) | `cargo install` + GitHub Release binaries, all six desktop tier-1 targets, no Homebrew, passive updates, no phone-home ever | — |
| [0021](0021-preview-budget-and-graceful-degradation.md) | The budget splits in two: `frame` under 500 ms cold is primary; render/preview is secondary. Proxy-resolution preview adopted, its numbers deferred | 0046, 0050, 0065 |
| [0046](0046-proxy-preview-target-is-720p-long-edge-capped.md) | The proxy target is a single 720p tier, long edge capped at 1280 px | 0050, 0067 |
| [0050](0050-preview-hard-refuses-below-360p.md) | `preview` refuses below **360p** — from a rendered legibility pass on the real fixture | 0067 |
| [0065](0065-preview-proxy-target-720p-540p-floor-disclosed-not-certified.md) | The ladder is `720p → 540p → hard fail`, **no 360p tier**. Its "legibility is unmeasured" clause is retired by 0067 | 0067 |
| [0067](0067-two-floors-a-wall-clock-give-up-point-and-a-legibility-refusal.md) | **Two floors, not one**: 540p is where the ladder gives up on *time*; 360p is where a frame stops being *readable*. Both stand; the ladder is unchanged | — |

## The document model

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0001](0001-flat-element-list.md) | A project is one flat list of uniform elements on one absolute clock. No scene, no per-kind collections, no clip-owned audio. **Its track rejection is superseded by 0004**; the rest stands | 0004 (partially) |
| [0004](0004-tracks-as-constrained-lanes.md) | Elements live in **tracks**: named containers with an integer `layer`, supplying stacking and never timing. Children keep absolute times, array order means nothing, and children of one track may not overlap | 0019, 0031, 0059, 0060 |
| [0002](0002-inline-source-no-asset-table.md) | An element names its file inline. There is no asset table — the one carve-out is the `fonts` table | 0007, 0053 |
| [0053](0053-asset-path-resolution-no-assetroot.md) | Paths resolve against the project file's own directory. No `assetRoot`; absolute paths permitted; a missing source is a plain error | 0056 |
| [0056](0056-remote-source-probe-session-scoped-no-persistent-cache.md) | A URL source is probed once per session, deduplicated by URL, with no persistent cache. A network failure is `UNCHECKED`, never a confirmed defect | — |
| [0016](0016-no-format-version-the-unknown-key-error-is-the-mechanism.md) | The file carries **no version number**. The unknown-key error is the migration mechanism, and there is no `montaget migrate` | 0017, 0041, 0042 |
| [0017](0017-closed-schema-no-escape-hatch.md) | The schema is closed at every object level, including `run` and `keyframe`. No `x-` prefix, no in-band escape hatch | — |
| [0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) | Canonical key order **is** schema property-declaration order. `validate` checks it under a `LAYOUT` category; `fmt` splits into `--check` and write | — |
| [0042](0042-montaget-json-is-a-convention-fmt-gets-a-shape-check.md) | `.montaget.json` is a documented convention, never enforced, with no in-document marker. `fmt` refuses only when `tracks`/`fps`/`frame` are wholesale missing | — |
| [0030](0030-defaultable-field-presence-is-content.md) | A defaultable field's **presence is content**. `fmt` never adds or removes one; omission and explicit-at-default are two different declarations | — |

## Time

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0005](0005-absolute-integer-milliseconds.md) | Absolute integer milliseconds, half-open `[start, end)`, `start`+`end` and no stored `duration`. Structural edits belong to a tool | 0012, 0020, 0035, 0036, 0041 |
| [0020](0020-speed-overrun-hold-loop.md) | `speed` is a strictly-positive rate multiplier; `fill` is renamed `overrun` (`"hold"`/`"loop"`); the two compose rather than exclude | 0045, 0055 |
| [0045](0045-speed-invariant-is-evaluated-in-exact-arithmetic.md) | `speed`'s rounding invariant is evaluated in **exact arithmetic**, never IEEE double. No schema change | — |
| [0062](0062-loop-declares-a-boolean-wrap-r-source-cut-pop-extends-mechanically.md) | `loop` is a project-level boolean, purely `validate`-facing; the wrap seam reuses `R-SOURCE-CUT-POP` unchanged | — |
| [0032](0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md) | Every slack is invariant by default. `shift` refuses rather than silently absorbing; `compare` is the backstop for raw edits | 0039, 0047, 0051, 0063, 0066 |
| [0047](0047-shift-releases-slack-by-boundary-instant-pairs.md) | `shift --release` takes a list of **boundary-instant pairs**, enumerated individually. No bulk release | — |
| [0036](0036-shift-preambles-coincident-instants-validate-and-compare-stay-out.md) | `shift` unconditionally prints what its rules will do at a coincident `at`. No flag. `validate` gets no coincidence census | 0039, 0063 |
| [0035](0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md) | Off-grid keyframe `t` stays legal; `validate` gains `R-KEYFRAME-UNREACHED`; the renderer needs no keyframe rounding rule | — |
| [0037](0037-derived-time-signature-is-a-provenance-gap-not-a-tool.md) | A derived time carries no signature — and that is a provenance gap, not a tool gap. **No authoring tool ships** | — |

## Transform and animation

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0012](0012-flat-transform-keyframes-carried-by-their-element.md) | One flat transform per visual element (`x`, `y`, `origin`, `scale`, `rotation`, `opacity`) in absolute integer pixels. Keyframes are `{t,v,ease}` on absolute times, **carried by their element**. Skew out; `box` retired; `clip` is the aperture | 0013, 0015, 0022, 0025, 0030, 0036, 0037, 0038, 0039, 0040, 0048, 0055 |
| [0013](0013-fitted-extents-floor-and-the-nine-origin-keywords.md) | Fitted extents **floor**, in exact integer arithmetic; the nine `origin` keywords are spelled, and `center-center` is an error naming `center` | 0014, 0015 |
| [0038](0038-ease-is-required-on-every-non-first-keyframe-record.md) | `ease` is required on every non-first keyframe record and a schema error on the first. Presence is a pure function of position | 0052 |
| [0022](0022-easing-example-is-hypothetical-not-measured.md) | ADR-0012's `photo-06` easing example is **hypothetical, not measured** — relabelled rather than replaced. No decision changes | — |
| [0025](0025-clip-stays-static.md) | `clip` stays static and is not keyframable. Wipes and reveals belong to the effect model | — |

## Fit and source dimensions

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0015](0015-fit-is-a-derivation-claim-and-gravity-retires.md) | `fit` is a **derivation claim, not a layout mode** — it never executes, and its only consumer is `validate`. The set is `cover`/`contain`/`literal`; `gravity` retires | 0017, 0023, 0024, 0026, 0027 |
| [0023](0023-video-source-dimensions-par-and-rotation.md) | One type-generic pipeline: decode, resolve rotation, apply PAR as an exact rational, round once. Images are the degenerate case | — |
| [0024](0024-measure-writes-the-fit-repair-not-the-verdict.md) | `measure` returns the fitted extent as a **bare derivation** — no verdict, no diff. `width`/`height` stay required and author-written | — |
| [0026](0026-exact-aspect-fit-both-spellings-stand.md) | At an exact aspect match both `cover` and `contain` are true and both stand; `fmt` must not canonicalize either | — |
| [0027](0027-vector-sources-out-of-scope.md) | Vector sources are out of scope for v1 — a scope boundary, not a permanent rejection | — |

## Text

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0007](0007-text-runs-literal-size-declared-fonts.md) | Text is **styled runs at a literal size**, always an array, in fonts the project declares by path. No fit-to-box, no auto-wrap, no `weight`/`bold`, no variable axes. NFC, raw UTF-8 | 0012, 0014, 0028, 0029, 0040, 0048, 0057 |
| [0008](0008-line-breaks-belong-to-the-agent.md) | The renderer never needs a break opportunity; the agent always does. `\n` is the only mechanism, `measure` gains a break-opportunity output, and **Montaget owns the line partition** via UAX #14 | — |
| [0028](0028-text-block-arithmetic-is-exact-tenths.md) | `line_height` is restricted to tenths; block height is `ceil`, in exact integer arithmetic, never IEEE double. States the general rule for any such field | — |
| [0029](0029-line-baseline-half-leading.md) | The baseline is half-leading — `slot_centre_y + (ascent − descent) / 2` — read across **every** run on the line, not the largest | — |
| [0048](0048-per-word-highlighting-is-a-timed-window-on-the-run.md) | Per-word highlighting is a `highlight` object **on the run**: a timed window with its own style delta. Word times are frozen literals; one highlighted word is one run | 0051 |
| [0051](0051-word-alignment-is-external-validate-and-compare-catch-drift.md) | Forced alignment stays **entirely external** — a forced aligner is a model, and Montaget contains none. No ingestion tool; `validate` and `compare` catch the drift | — |
| [0057](0057-font-vendoring-licence-gate-and-path-keyed-attestation.md) | `fonts vendor` is a local-only copy behind a three-bucket licence gate with a hard refuse and no override. Attestation is keyed by **file path** | — |

## Shapes, effects and audio

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0014](0014-stroke-is-paint-the-text-box-is-required.md) | `rect` and `ellipse` are sibling element types; `line`/`polygon`/`path` are rejected because a point list has no declared extent. **Stroke is paint**, not an effect. The text box stays required | 0015, 0028, 0040, 0048, 0058 |
| [0040](0040-effect-model-attachment-and-v1-vocabulary.md) | Effects attach as an ordered `effects: [...]` list — order is semantically real. v1 ships `blur`, `shadow`, shape-only `mask`. Effects attach to elements, never to a run | 0048, 0049, 0055, 0059 |
| [0049](0049-v1-colour-filter-vocabulary-four-scalar-members.md) | Four flat scalar colour effects — `tint`, `saturation`, `brightness`, `contrast` — no `mode` discriminator, plus a mechanical stopping rule that excludes curves and LUTs | — |
| [0059](0059-transitions-element-type-crossfade-only-exact-window.md) | A transition is **its own element type**, id-targeting two elements over an exact window. v1 `kind` is `crossfade` only | — |
| [0055](0055-audio-mixing-model-volume-fades-ducking-deferred.md) | `volume` is a keyframable `0..1..>1` linear multiplier, flat on `audio`/`video`. No `mute`. Automatic ducking deferred — hand-authored as ordinary keyframes | — |

## The tool surface

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0011](0011-tool-surface-reads-checks-renders.md) | Nine MCP verbs + three CLI-only, split deliberately unequally. **One binary, one core library; the MCP server wraps the library, never the CLI.** The schema and format docs are resources. Every write tool returns findings, never `ok` | 0012, 0016, 0019, 0024, 0026, 0030, 0035, 0036, 0037, 0039, 0041, 0042, 0051, 0060 |
| [0031](0031-timeline-overview-is-not-required-to-be-spatial.md) | An agent-facing overview is **not required to be spatial** — measured, not assumed. `timeline` as the human's wide view is unaffected | — |

## `validate`

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0006](0006-validate-reports-facts-and-render-enforces.md) | `validate` answers *"is this legal and does it agree with the disk"* and **never** *"does it say what you meant."* It prints its own boundary (`NOT CHECKED`); findings state facts, never repairs; the noise budget is a safety property; `render` runs the identical checks and is what enforces them | 0007, 0012, 0013, 0014, 0019, 0035, 0036, 0039, 0041, 0043, 0044, 0051, 0052, 0058, 0060, 0061 |
| [0061](0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md) | **Threshold provenance decides admission, not severity.** A check is fact-only if every number deciding whether it fires is derivable from the document. `R-CAPTION-PACE` is kept via a fenced exception with binding citation | — |
| [0043](0043-refuse-class-findings-a-repair-field-uniform-per-check-non-bypassable.md) | Error-class findings whose repair needs intent the document lacks are **refuse-class**: a `repair` field, decided once per check, uniform across instances, non-bypassable | — |
| [0018](0018-cross-track-coverage-is-group-scoped-not-a-union.md) | Cross-track coverage is **group-scoped pairing**, not a frame-wide union — the union rule fires on zero of the fixture's 11 real gaps under all 2¹³ bases | — |
| [0019](0019-layer-anchor-gets-an-id-a-validate-check-and-one-hop.md) | An anchor names an element `id` (now a required unique field) and resolves in **one hop**. `error` for missing/self-anchored, `review` for a target that never overlaps in time | — |
| [0060](0060-layer-tie-is-an-error-array-order-stays-meaningless.md) | A geometry-overlapping layer tie is an **error**, never a fallback order. Array order stays permanently meaningless | — |
| [0033](0033-same-source-cut-continuity-is-a-review-check.md) | `R-SOURCE-CUT-POP` at `review`, keyed on same track + canonicalized source + adjacency — **never `group`** | 0062 |
| [0034](0034-caption-pace-and-repeat-duration-checks.md) | `R-CAPTION-PACE` (cps over grapheme clusters) and `R-CAPTION-REPEAT-DURATION` (symmetric, not "shorter on repeat"), both `review` | 0054 |
| [0054](0054-caption-audio-backing-and-minimum-duration-checks.md) | `R-CAPTION-NO-AUDIO` and `R-CAPTION-MIN-DURATION` (834 ms, a script-agnostic floor), both `review` | — |
| [0044](0044-off-canvas-is-a-standing-review-check-not-a-frame-change-census.md) | `R-OFF-CANVAS` at `review`: a **standing** check, not history-triggered — fires when an element's rect never intersects the frame at any instant of its active range | — |
| [0052](0052-review-check-for-inert-ease-on-held-keyframes.md) | `R-EASE-INERT` at `review`, on literal exact equality of author-written `v`, one finding per run of consecutive holds | — |
| [0058](0058-text-box-slack-is-a-note-with-sibling-census.md) | `R-BOX-SLACK` at `note`, height-only, `slack > max(2px, 10%)`, carrying a sibling census | — |

## `compare`

| ADR | Decision | Amended by |
| --- | --- | --- |
| [0039](0039-group-keyframe-time-check-retracted-from-validate-reassigned-to-compare.md) | The group keyframe-time check is **retracted from `validate`** entirely and reassigned to `compare`: desynchronization is a temporal predicate a single document has no time axis to evaluate | 0063 |
| [0063](0063-compare-drift-checks-keyframe-instant-relationships.md) | One exact-equality predicate over three keyframe-involving populations. "Held" means exact numeric equality; fixed-offset stays deferred. No severity | 0066 |
| [0066](0066-boundary-coincidence-cluster-drift-is-its-own-predicate.md) | Boundary-coincidence-cluster drift is **its own** predicate, scoped to all N≥2 clusters, reporting cluster-level facts (moved-set vs stayed-set) rather than pairwise | — |

---

## Reading order for a newcomer

The series is 66 ADRs and mostly not worth reading front to back. To get the model:

1. **[0003](0003-general-video-editor-not-channel-tooling.md)** — what this is and what the fixture is for. Read the anti-drift rule and take it seriously.
2. **[0004](0004-tracks-as-constrained-lanes.md)** + **[0005](0005-absolute-integer-milliseconds.md)** — the shape of the document and its clock.
3. **[0012](0012-flat-transform-keyframes-carried-by-their-element.md)** — where things are and how they move. Check its amendment list first; it is long.
4. **[0006](0006-validate-reports-facts-and-render-enforces.md)** — the discipline the whole surface rests on. Also the most-amended file in the series.
5. **[0011](0011-tool-surface-reads-checks-renders.md)** — what the tool actually offers, and the write-tool invariant that bounds it.

`CONTEXT.md` at the repo root is the glossary and is shorter than any of these. Read it first.

## Conventions this series follows

- **Amended, never rewritten.** An ADR that corrects an earlier one says so; the earlier one gets a pointer under its title.
- **Evidence is committed before `status: accepted`.** A numeric claim needs a re-executable script; a qualitative one needs the artifact verbatim. See `docs/agents/domain.md`.
- **ADRs land on `main` as their ticket closes.** Nine of these did not, for a while, and [#178](https://github.com/MBehtemam/Montaget/issues/178) is what that cost.
