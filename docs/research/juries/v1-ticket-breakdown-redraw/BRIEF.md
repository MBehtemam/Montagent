# The v1 ticket breakdown, redrawn

Second draft. The first ([`docs/research/juries/v1-ticket-breakdown/`](../../docs/research/juries/v1-ticket-breakdown/)) was refuted by two of three jurors on the blocking graph and on granularity. This one is drawn from `docs/adr/README.md` rather than from #168's story list, which was the court's unanimous direction.

## What changed, and why

**The graph is sequenced by the data each ticket consumes, not by the verb it ships.** That was Fable's headline finding and it accounts for most of the first draft's defects. The layers, in order, are: *nothing* → *the document* → *media facts* → *resolved layers* → *laid-out text* → *resolved keyframes* → *rasterized frame* → *encoded video*. A ticket is blocked by whatever produces the data it reads, and by nothing else.

The visible consequences: `probe` moves to the spine and owns FFmpeg resolution (the first draft had it blocked by the document model and had FFmpeg resolution four tickets downstream of its first consumer); the caption checks no longer wait on the text engine; the cut list no longer waits on the keyframe resolver; and `preview` sits behind `render` rather than behind `frame`, because `preview` is a span of video in every preview ADR, not a still.

**Coverage is checked against the ADR index.** #168's story list has been amended to 113 stories, but the index remains the authority — every ticket below names the ADRs it implements, and the check is that every row of the index has a ticket, not that every story has one.

**Three tickets are new because the ADRs demand them and no story did**: audio mixing (ADR-0055/0020/0062), the `fonts` tool surface (ADR-0057), and slack as a shared primitive (ADR-0032). Two more are prerequisites already filed: #185 (`mask` parameters) and #186 (the fixture's text metrics under Open Runde).

---

## Spine — nothing depends on anything yet

**1. Workspace, the finding type, the report, and the check registry**
*Blocked by: none.* Implements ADR-0006, ADR-0011, ADR-0043, ADR-0061.
The four crates; `Finding` with code, severity, location, inline numbers, sibling census, `repair`, and `UNCHECKED` reason; the four non-severity categories (`NOT CHECKED`, `UNCHECKED`, `LAYOUT`); JSON canonical with the text form generated from it; exit codes 0/1/2/3/70; `E-PARSE` with line, column, byte offset, offending line and caret; the CLI and MCP adapters, both thin.
Plus the **check registry** — the first draft left this ownerless and two tickets would each have invented it. It carries ADR-0043's per-check refuse-class declaration and a completeness test that every registered `error` code has one.
*Demo:* `montagent validate` over both adapters on a header-only project — and the negative arm, which is what makes it a tracer bullet rather than a happy path: a malformed file giving `E-PARSE` with a caret and **exit 2**, a bad invocation giving **exit 3**.
*Note:* the `Finding` type is being designed against one finding that uses almost none of its fields. Construct a census-carrying, a refuse-class and a citation-carrying finding as test fixtures here, or the type gets reshaped at the first check ticket that needs one.

**2. CI: six tier-1 targets, and the Skia prebuilt canary**
*Blocked by: 1.* Implements ADR-0010, #36.
Two jobs, not one. The test suite per-PR; the canary **scheduled, with an empty `CARGO_HOME` and empty target dir**, because #36's point is that the failure arrives from upstream rather than from a commit and a cached per-PR job structurally cannot see it.
The acceptance criterion is **the resolved key `jpegd-jpege-pdf`**, not "the build succeeded" — pinned in exactly one place. `montagent-render` declares `skia-safe` from here even though nothing calls it yet; cargo builds a declared dependency, so the canary is live from day one rather than from the rasterizer ticket thirteen tickets later.

**3. FFmpeg resolution and `probe`**
*Blocked by: 1.* Implements ADR-0023, ADR-0056, ADR-0011's `probe` quad.
Resolving `ffmpeg`/`ffprobe` from `PATH` and reporting absence as exit 70 naming what was looked for — **owned here, at the first tool that spawns a subprocess**, not in the packaging ticket four layers downstream. The quad (video stream duration, container duration, `start_time`, both frame rates) in integer milliseconds, refusing to collapse to a scalar; dimensions, alpha, sample rate, channels; the type-generic decode → rotation → PAR → round-once pipeline; the `(path, size, mtime)` cache with the miss always reported; remote URLs deduplicated per session with no persistent cache and failures as `UNCHECKED`.
*Note:* this introduces the only network code in the product. The "no unsolicited network calls" invariant needs a test from here onward, not from the packaging ticket.

**4a. The document model, and the fixture round-trips**
*Blocked by: 1.* Implements ADR-0001, 0004, 0005, 0012, 0013, 0014, 0017, 0019, 0025, 0030, 0038, 0040, 0048, 0049, 0055, 0057, 0059, 0062, 0068.
Every struct and every field, **field order frozen** — ADR-0041 makes Rust struct field order the canonical key order, so reordering a field is a format change visible in `fmt` and `LAYOUT` on every file. `deny_unknown_fields` at every level including `run` and `keyframe`; `i64` ms and half-open ranges; required unique `id`; the `fonts` and `fontVendor` tables; `effects` last in key order per ADR-0068; JSON Schema generated from the types with a committed-vs-generated drift test.
*Demo:* the committed fixture round-trips byte-for-byte.
*Note:* ADR-0030 makes a defaultable field's **presence** content, which forces `Option<T>` rather than `#[serde(default)]` throughout — otherwise omitted and explicit-at-default are indistinguishable after a round trip. Make "a lossless round-trip of a file containing an unknown key and a retired spelling" an acceptance criterion **here**, not at the `fmt` ticket, or ADR-0042's rule that `fmt` proceeds on a file with error findings sends an edit back through every type.

**4b. Retired spellings**
*Blocked by: 4a.* Implements ADR-0016, ADR-0043, ADR-0068.
Nine spellings, each a fire/must-not-fire pair, each naming its replacement, each with its ADR-0043 class. `gravity` is refuse-class with a geometry-grouped census; the bare `mask` key is advise-class per ADR-0068. This is where stories 38–39 are actually demonstrable — ticket 1 builds the mechanism but has no refuse-class instance to exercise it on.
*Note:* ADR-0043 classifies only `gravity`. `box` and `align`-on-image have `gravity`-shaped intent forks that no ADR has ruled on; record rather than decide.

---

## Reads over the document alone

**5. `fmt`, the canonical convention, and atomic whole-file write** — *blocked by: 4a.* ADR-0041, 0042, 0030. Includes the permissive, order-and-presence-preserving parse path ADR-0042 forces, and the atomic write every later write tool reuses.

**6. `create_project` and the MCP resources** — *blocked by: 5.* ADR-0011. First write tool; proves findings-as-result. The schema and format-docs resources are static the moment 4a generates the schema.

**7. `timeline`** — *blocked by: 4a.* ADR-0031. A leaf with no dependents.

**8. `query --from --to` and `--where --census`** — *blocked by: 4a.* ADR-0011. The cut list is the set of intervals over which the *presence* set is constant — element boundaries only, nothing resolved. Parallel with the whole text-and-raster arm.

**9. Structural time checks, and slack** — *blocked by: 4a.* ADR-0004, 0005, 0020, 0032, 0045, 0062.
Overlap within a track (error), gaps reported separately and never as errors, the `speed` invariant in exact arithmetic, `N-QUANTIZATION` reporting what quantization changes rather than what is unaligned.
**Owns the `slack` primitive** (ADR-0032) — the distance from a boundary to its nearest neighbour in any track, or to `duration`. `shift` and `compare` both consume it and the first draft had neither defining it, so two tickets would have implemented it twice and disagreed.

**10. Layer and anchor resolution, and the checks over it** — *blocked by: 4a.* ADR-0019, 0041, 0060.
The **resolution function** — anchor → integer layer in one hop — owned here, because the keyframe resolver and the rasterizer both need painter's order and neither should own it. Plus the checks: missing/self/chained anchor (error), never-overlapping target (review), geometry-overlapping layer tie (error), and `LAYOUT` key order checked unconditionally and never gating `render`.

**11. Caption checks** — *blocked by: 4a, 9.* ADR-0034, 0054, 0061.
`R-CAPTION-PACE` over grapheme clusters with its raw measurement and binding citation, `R-CAPTION-REPEAT-DURATION` symmetric, `R-CAPTION-NO-AUDIO`, `R-CAPTION-MIN-DURATION` at 834 ms.
**No dependency on the text engine.** ADR-0034/0054 compute these with no I/O; grapheme segmentation is Unicode, not font layout. The first draft serialized all four behind `measure` on the strength of one unrelated check.

**12. `R-VISUAL-GAP`** — *blocked by: 4a, 10.* ADR-0018. Group-scoped pairing, never a frame-wide union — the union rule fires on zero of the fixture's 11 real gaps under all 2¹³ bases, which is why it is a check about `group` and declared rects, with no motion in it.

---

## Media facts

**13. `validate` against the disk** — *blocked by: 3, 4a.* ADR-0002, 0053, 0056, 0006.
Every referenced source probed, real duration and dimensions reported; `E-SOURCE-OVERRUN`; a missing local source as a plain error; a network failure as `UNCHECKED` with a structured reason, never confirmed-missing. No fast mode and no scoped check, ever — the defect that changed on disk rather than in the project is the one this exists to catch.

**14. Fitted extents and the `fit` checks** — *blocked by: 3, 4a.* ADR-0013, 0015, 0023, 0024, 0026.
Fitted extents by integer cross-multiplication, box dimension assigned verbatim, slack axis in integer division — never `floor(sw * f)`, and the legal `0` case is not clamped. `measure`'s fitted-extent output as a bare derivation with no verdict. Both spellings stand at an exact aspect match and `fmt` canonicalizes neither.
**This is a `fit` ticket, not a text ticket** — the first draft filed it under the text engine, where it has no font in it and no access to the source dimensions it needs.

---

## Text

**15. `measure` and the text engine** — *blocked by: 4a.* ADR-0007, 0008, 0028, 0029.
`parley`/`skrifa` with `complex-scripts`; Montagent's own UAX #14 line partition, since ADR-0008 takes the partition away from `split('\n')`; advance width, ascent, descent, line count; per-line `baseline_y` as half-leading read across **every** run on the line, not the largest; block height as `ceil` in exact tenths; break opportunities with the segmenter version named; the **stroked** extent rather than the typographic one (ADR-0014); and the renderer opening nothing outside the declared font chain.

**16. ADR-0007's text checks** — *blocked by: 15, 3.* Glyph coverage across the chain (error), the font census, the grapheme-cluster check that no run boundary splits a base from its combining mark, the invisible-character census (ZWJ/ZWNJ, RLM/LRM, variation selectors), the mixed-normalization finding, and font files joining the `(path, size, mtime)` probe cache — a font swapped in place is a silent whole-project render change that no census sees.

**17. `R-BOX-SLACK`** — *blocked by: 15, **#186**.* ADR-0058. `note`, height-only, `slack > max(2px, 10%)`, carrying a sibling census.
**Blocked on #186 deliberately.** The fixture's 22 text elements were tuned against SF Pro Rounded and the repo now vendors Open Runde, so this check may fire on the regression guard — and #168's rule would then send the implementer hunting a bug in a correct check.

**18. The `fonts` tool surface** — *blocked by: 4a, 15.* ADR-0057.
`fonts list` with per-font detected licence status; `fonts vendor` running its blocklist and heuristics **before** copying bytes, writing the `fontVendor` attestation; the font-attestation `error` on a hash mismatch and the orphaned-attestation `note`, never auto-pruned.
**Had no story at all in #168's original 83** — an entire CLI verb pair, invisible to a story-list audit.

---

## Resolved keyframes

**19. The keyframe resolver, and `query --at` over the document** — *blocked by: 4a, 10.* ADR-0012, 0035, 0038.
Resolution of `{t,v,ease}` on absolute times, keyframes carried by their element, `ease` presence a pure function of position. `query --at` returning presence, painter's order and resolved transform values.

**20. `query --at`, the full block** — *blocked by: 19, 14, 15, 3.* ADR-0011.
Offset into source (needs the probed duration under `overrun: hold|loop`), the crop rectangle (source-dimension arithmetic under `fit`), the ink box (needs text layout), and the `NOT COVERED` region. These are the four things `jq` cannot compute, and each one reaches a different layer — which is why they are separated from the resolver.

**21. Motion and geometry checks** — *blocked by: 19, 14.* ADR-0033, 0035, 0044, 0052, 0062.
`R-SOURCE-CUT-POP` keyed on same track + canonicalized source + adjacency and never `group`, extending across the wrap when `loop: true`; `R-KEYFRAME-UNREACHED`; `R-OFF-CANVAS` as a standing check on an element's rect over its whole active range; `R-EASE-INERT` on literal exact equality of author-written `v`, one finding per run of holds.
*`R-EASE-INERT` lives here, not with the caption checks* — the first draft misfiled it, and all three jurors moved it.

---

## Raster

**22. The rasterizer core and the `frame` surface** — *blocked by: 19, 3.* ADR-0009, 0010, 0012, 0014, 0023.
Image resample through `clip`/`fit`, video decode and seek, `rect`/`ellipse` with fill and inside-stroke and `radius`, the transform pipeline (`x`/`y`/`origin`/`scale`/`rotation`/`opacity`). `frame` returning JPEG at half the project's frame size by default with full scale and PNG behind flags, `--crop`, the `query --at` block printed unconditionally, and the **under-500 ms cold at true pixel dimensions** budget.
*Note:* the fixture has **zero `video` elements**. The decode path will not be exercised by it, and per ADR-0003 that is not evidence it is unneeded.

**23. Text drawing, and the first falsification** — *blocked by: 22, 15, **#186**.* ADR-0007, 0014, 0029.
Text as `skrifa` outlines with outside-stroke, drawn from `measure`'s partition and baselines.
Then the comparison this whole spec exists to make possible: `frame` at fixed timestamps against `reference/frame-intro.png` and `reference/frame-05-at-11s.png`, **both already committed and both extracted from the published MP4**.
**Two distinctions the ticket must state or the suite tests nothing.** A golden frame we render and commit is *self-confirming* — it catches regressions and can never falsify the format. Only the reference frames falsify. And until #186 lands, the text regions are compared in a **different typeface than the reference**, so the gating comparison is region-masked to the image card, the Ken Burns move, the drawn flag, the panels and the badge, with text reported as an explicitly non-gating measurement and the font substitution named.

**24. Effects, colour filters, transitions and highlight** — *blocked by: 22, 23, **#185**.* ADR-0040, 0048, 0049, 0059, 0068.
The ordered `effects` list with `blur`, `shadow`, shape `mask`; the four colour scalars; `crossfade` transitions as their own element type over an exact window, plus the check that a transition's derived range still matches the two elements it bridges; `highlight` windows on runs.
**Blocked on #185** because the `mask` member's explicit parameters do not exist yet; ADR-0068 fixed only the param-less form.
*Note:* the feature set pinned in ticket 2 must not change here. If an effect needs a `skia-safe` feature, that is its own ticket and it re-verifies all six targets.

---

## Encoded video

**25. `render`, video only** — *blocked by: 22, 9, 10, 13.* ADR-0006, 0011.
The frame loop and FFmpeg encode; the declared `output` written via temp path and atomic rename; **the identical check engine `validate` runs**, refusing on any `error` — this is the enforcement, and it is why all three check tickets block it; surviving `review` findings and the `NOT CHECKED` footer printed after success; the machine-readable result on stdout and coarse progress on stderr.

**26. The audio mix** — *blocked by: 25, 3.* ADR-0055, 0020, 0062.
Mixing every `audio` and `video` element; `volume` as scalar or keyframed, defaulting to 1, negative a schema error, no `mute`; `speed` through the `atempo` convention — the fixture has four narration elements at `speed: 0.645`; `overrun: loop` on audio with `hold` staying a schema error; the `loop` wrap.
**Had no story at all in #168's original 83**, against a fixture of 20 audio elements and 22.4 s of narration.

**27. `render --from --to`, and the budgets** — *blocked by: 25.* ADR-0011, 0021. Derived output name, refusal of an explicit `--output` equal to the project's, and the 60 s-under-two-minutes budget.

**28. `preview` and the proxy ladder** — *blocked by: 27.* ADR-0021, 0046, 0050, 0065, 0067.
720p target with the long edge capped at 1280 px; exactly one degrade to 540p with mandatory disclosure; hard fail rather than a second degrade; the 360p legibility refusal as a guard that is unreachable under the ladder; never applied to `render`.
**Two floors, two constants — do not collapse them.** 540p is a wall-clock give-up point, 360p is a legibility threshold.
*Blocked by `render`, not `frame`* — `preview` is a span of video in every preview ADR and needs the encode path.

**29. The whole-video falsification** — *blocked by: 26.* Render the committed fixture and compare against `reference/en-halloween-decorating.mp4`.
**Not redundant with ticket 23.** Frames cannot falsify duration (65.216 s against the stream, not the container), frame count, audio placement, or `speed: 0.645`. This is the only test of ticket 26.

---

## Edits and comparisons

**30. `shift`** — *blocked by: 5, 9.* ADR-0011, 0012, 0032, 0036, 0047.
Project-scoped by default; refusal when a time-based element straddles the insert point, naming the nearest legal boundaries; keyframes carried with their element rather than "at or after"; the coincident-instant preamble printed unconditionally with no flag; refusal of any edit that would change an existing slack; `--release` taking boundary-instant pairs individually.
*Note:* `shift` must return the new state's findings, and at this point only tickets 9–12's checks exist. It returns a knowingly partial set — say so in the ticket rather than discovering it.

**31. `compare`** — *blocked by: 4a, 9.* ADR-0039, 0051, 0063, 0066.
Grouped deltas as facts with no severity; slack drift; keyframe-instant relationships that held by exact equality and stopped holding; boundary-coincidence-cluster drift at cluster level rather than pairwise; a run whose text changed while its `highlight` timing did not.
**No dependency on the keyframe resolver.** ADR-0063's predicate is exact equality of keyframe `t` values between two files and ADR-0066's is cluster membership of instants — nothing interpolates. The first draft serialized the whole compare arm behind the resolver for nothing.

**32. Packaging and releases** — *blocked by: 25.* ADR-0064.
`cargo install` plus GitHub Release binaries on all six desktop tier-1 targets, no Homebrew, passive updates, no phone-home ever. FFmpeg resolution already landed in ticket 3.

---

## The frontier

Day one: **{1}**. Then **{2, 3, 4a}** in parallel, and once 4a lands, **{4b, 5, 7, 8, 9, 10, 15}** all open at once — against the near-linear chain the first draft implied.

Two external prerequisites gate leaves rather than the trunk: **#185** gates ticket 24 only, and **#186** gates tickets 17 and 23's text arm. Neither blocks the spine.

---

# The question put to you

A three-juror court refuted the first draft of this breakdown. This is the redraw. **One question:**

> **Which of the first court's findings did this redraw fail to absorb, and what is newly wrong that was not wrong before?**

The first court's findings are recorded at `docs/research/juries/v1-ticket-breakdown/` — read `README.md` for the verdicts and the three ballots for the reasoning. Check each finding against the redraw above and say, per finding, whether it was absorbed, partially absorbed, or missed. Be specific; "mostly absorbed" is not an answer.

Then attack the redraw on its own terms. It is a second draft by the same author who wrote the first, so the likeliest failure is a correction applied too literally, or a new defect introduced by the re-sequencing. In particular:

1. **Did splitting tickets introduce edges that are now missing?** The first draft was refuted for a graph drawn from the verb table. This one claims to be drawn from data dependencies. Test that claim — find a ticket that cannot start with its declared blockers done, and a ticket whose blockers are still ceremonial.
2. **Is coverage actually complete against `docs/adr/README.md`?** That is the standard this redraw sets itself. Walk the index — every row, all 68 ADRs — and name any that no ticket implements. Note that #168's story list has been amended to 113 stories; the index is still the authority.
3. **Is the reasoning behind any absorbed correction wrong even though the correction looks right?** Two examples the author is unsure of: ticket 11 (caption checks) is now blocked by the time-checks ticket — is that real, or was it carried over from the first draft without re-examination? And ticket 12 (`R-VISUAL-GAP`) is blocked by the layer/anchor ticket — does a group-scoped coverage check actually need resolved layers?
4. **Sizing.** Ticket 4a now implements nineteen ADRs. It is the biggest in the set by a wide margin and the author has left it whole on the argument that ADR-0041 freezes field order, so the shape must land in one piece. Is that argument right, or is it a rationalisation for a ticket that cannot fit a context window?

Default to "refuted". Verify against the repository — the redraw may misdescribe an ADR, and finding that is part of the exercise. Note that ADR-0068 is new and on `main` since the first court ruled.

End with the single most important thing still wrong.
