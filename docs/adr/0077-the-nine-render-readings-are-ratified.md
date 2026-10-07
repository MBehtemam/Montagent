---
status: accepted
amends: 0035 (states which whole millisecond of its grid a frame is painted at), 0011 (ratifies `render`'s four command-surface readings: the derived extent, the missing-`output` refusal, `--to` past the end, and what `<name>` is in the derived partial name), 0021 (states what happens to an odd frame dimension, which its *"never silently"* rule requires but does not decide), 0009 (ratifies the encoder settings its spawned-`ffmpeg` decision leaves open, and confirms it does not take the `libopenh264` escape route), 0055 (ratifies the mix bus, `normalize=0`, and that a keyframed `volume` is applied as the resolved value on every sampled frame)
---

# The nine `render` readings are ratified

> **Amended by [ADR-0143](0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md)**,
> which keeps reading 7's libx264, CRF 20 and preset `medium` as a measured choice rather than
> a default, pins libx264 to `-threads 5` instead of ffmpeg's automatic count, and makes the
> `render` and `preview` answers disclose the encoder, preset, CRF, threads and ffmpeg
> version. VideoToolbox is rejected; none of it is exposed as an option.

> **Amended by [ADR-0155](0155-motion-blur-is-a-per-element-field-that-accumulates-the-element-over-a-centred-shutter.md)**:
> a moving element carrying `motion_blur` is also resolved at exact rational sample instants
> around reading 1's floored frame instant. The frame instant still decides presence and a
> video's source frame.

> **Amended by [ADR-0172](0172-a-keyframed-volume-is-heard-on-the-sample-its-instant-names.md)**:
> reading 9's commands are heard on the sample their instant names, on 1 ms frames and in
> pieces of at most 256 commands. Its claim that a per-frame step is "below what the filter
> could have resolved anyway" no longer holds; that interval was the decoder's frame, and it
> made the command up to 21 ms late.

**Ticket:** [#287](https://github.com/MBehtemam/Montagent/issues/287), from
[#215](https://github.com/MBehtemam/Montagent/issues/215).

## The gap

[#215](https://github.com/MBehtemam/Montagent/issues/215) built `render`. Nine places where
the ADR series names a rule but not its edge — or names nothing at all — had to be decided
in the code to ship it. Each was argued at its site, in `crates/montagent-core/src/verbs/render.rs`
and `crates/montagent-render/src/encode.rs`, and each ended *"raised for ratification"*.

This ADR is that ratification, on the precedent of
[ADR-0076](0076-the-four-structural-time-finding-codes-are-ratified.md) and the tickets it
names: ADR-0031 makes the ADR series the specification, so behaviour an author can observe
— a refusal, a derived file name, a pixel that moved — existing only as a source comment is
a gap in the spec and not only in the docs.

**All nine are ratified as shipped.** Nothing in the code changes; what changes is that the
argument is now consultable from the ADR that each reading extends.

## Decision

### 1. Frame *n* is painted at `⌊n × 1000 / fps⌋` ms

Amends [ADR-0035](0035-keyframe-grid-alignment-is-a-review-check-not-a-schema-rule.md),
which fixes the grid — frame *n* samples at `n × 1000/fps` — and says nothing about an
instant that is not a whole number. At 30 fps it is not: the grid is `n × 100/3` ms. Every
resolution in `crate::verbs::query::geometry` takes whole milliseconds, so the instant is
floored, at one place (`render::instant_of`) that both the frame loop and the `volume`
commands read — or a fade would be heard on a different clock from the one it is seen on.

Two consequences, and the second is why flooring is admissible at all:

- **Presence is unaffected.** Every `start` and `end` is an integer (ADR-0005), so
  `⌊t⌋ ≥ start ⇔ t ≥ start` and `⌊t⌋ < end ⇔ t < end`. No element enters or leaves a frame
  because of the floor.
- **An interpolated value can differ**, by less than one millisecond of travel, at rates
  where the grid is not millisecond-exact. At 25, 50, 100, 200, 500 and 1000 fps it is
  exact and the floor takes nothing.

The alternative — threading a rational instant through the painter — is a larger change
than #215 and buys under a millisecond of a continuous property. It is not taken.

The claim is re-executable:
`verbs::render::tests::the_painted_instant_is_the_grid_instant_floored_and_presence_is_unaffected`
asserts the floor at 30 fps, exactness at all six rates named above, inexactness at 24, 30
and 60, and the presence equivalence by scan, in integer arithmetic. One scan settles both
halves: `⌊t⌋ < end ⇔ t < end` is the negation of `⌊t⌋ ≥ end ⇔ t ≥ end`.

### 2. A project with no `duration` renders to its last boundary

Amends [ADR-0011](0011-tool-surface-reads-checks-renders.md). `render::extent` takes the
declared `duration`, or the greatest `end` any element states — which is the derived
`duration` `CONTEXT.md` names and `crate::slack` already computes. **One derivation, not
two**: a second spelling of the project's own last boundary is exactly the "two spellings
of one freedom" this project's decision history keeps refusing, and here it would let
`render` and `shift` disagree about where the project ends.

A project with neither a `duration` nor a single element boundary is refused with exit 3.
There is nothing to render, and the refusal says so.

### 3. A project with no `output` and no `--output` is exit 3, naming the field

Amends ADR-0011. There is nowhere to put the video. This is `BadInvocation` and not a
document error: the project is legal, and ADR-0011 makes the command the thing that says
where output goes, so the missing piece is in the invocation. The refusal names both
`output` and `--output`, since either supplies it.

### 4. `--to` past the project's end is legal; `--from` before 0 is not

Amends ADR-0011, on its own reading that **every instant is a legal question**. The frames
past the last boundary are the background, which is what every uncovered instant inside the
project is too — there is no boundary at which asking becomes an error. `--from` before 0
*is* refused, because the clock begins at 0 (ADR-0005) and there is no instant there to
ask about.

The asymmetry is the point: the project's end is derived, and the clock's start is not.

### 5. The derived partial name is `out/<name>.<from>-<to>.mp4`, and `<name>` is the output's stem

Amends ADR-0011, which fixes the shape (story 55) and never says what `<name>` is. It is
the stem of the project's declared `output` — `en-halloween-decorating` from
`out/en-halloween-decorating.mp4` — or the project file's own stem where it declares no
`output`. The path is under the project's own directory, which is where ADR-0053 resolves
every other path.

The deliverable's own name is the one thing an author already associates with the project,
and a partial render is a slice of that deliverable. The fallback exists so that the
`--from`/`--to` pair keeps working on a project that names no `output` at all — the case
reading 3 refuses for a *full* render, because there a file name would have to be invented
for the deliverable itself.

This never lands on the deliverable: an explicit `--output` equal to the project's own is
refused while a range is set (story 56), by path identity rather than by spelling.

### 6. An odd frame dimension is padded to even, right and bottom, in the background colour, and disclosed

Amends [ADR-0021](0021-preview-budget-and-graceful-degradation.md), whose rule is that
`render` never *silently* changes what was rendered — a rule that forbids one of the three
answers here without choosing among the other two.

`yuv420p` cannot carry an odd width or height at all, so the choices were: refuse a legal
project, crop a declared pixel, or add one row or column. Padding is taken, and **disclosed**
— `Finished::encoded` beside the declared frame, in the answer's `encoded` field and in the
`encoded` prose row — which is what keeps it inside ADR-0021 rather than against it. The
pad is the project's own `background`, on the right and bottom, so the declared frame keeps
its origin and every painted pixel keeps its coordinates.

Refusing was rejected because nothing in the format makes an odd dimension illegal, and
cropping was rejected because it silently discards a pixel the document declared.

### 7. The encoder settings: `libx264`, `yuv420p`, CRF 20, `medium`, `+faststart`; AAC at 160 kb/s

Amends [ADR-0009](0009-rust-host.md), which decides that `ffmpeg` is **spawned, never
linked** — `libx264` is GPL and linking `libavcodec` would make the binary a GPL combined
work — and names `libopenh264` as an escape route without taking it. This ADR confirms it
is still not taken.

H.264 in an MP4 with `+faststart` is what every player and every upload form reads. An
`ffmpeg` built without `--enable-gpl` has no `libx264`, and **that surfaces as exit 70
carrying `ffmpeg`'s own sentence** rather than as a silent fallback to a lesser encoder: a
deliverable that quietly changed codec is the failure ADR-0011 calls *"success"* — the one
that matters.

CRF 20 at preset `medium` is a quality-per-byte default, not a measured optimum, and is
recorded as such. A ticket that wants to move it should measure first; nothing downstream
reads either number.

### 8. The mix bus is 48 kHz stereo, and `amix` runs with `normalize=0`

Amends [ADR-0055](0055-audio-mixing-model-volume-fades-ducking-deferred.md).

**Every input is resampled to the bus rate first** (`aformat=sample_rates=48000:channel_layouts=stereo`).
That is what makes an `aloop` sample count exact from the document alone: the loop length is
the as-played duration in samples at a rate the mix path knows without probing anything.
A bus that adopted each source's rate would need a probe inside the mix path to compute the
same number, and would compute a different one per file.

**`normalize=0` is what keeps two narration lines at `volume: 1.0` each at `1.0`.** ADR-0055
makes `1` mean source level; `amix`'s default normalisation would divide by the input count,
so adding a second narration line would quieten the first — a level the document does not
state. Clipping past the sum is *"the renderer's documented behaviour"* in ADR-0055's own
words, and is preferred to a silent, count-dependent attenuation.

`normalize` needs `ffmpeg` ≥ 4.4. An older one fails with its own sentence at exit 70, on
the same rule as reading 7.

### 9. A keyframed `volume` is applied as the value resolved on every sampled frame

Amends ADR-0055, which makes `volume` keyframable and does not say how a keyframe list
reaches the encoder. It reaches it as timed commands (`asendcmd`) against a `volume` filter
at `eval=frame`, one command per sampled frame whose resolved value moved — **so the ease
curve is the one `resolve` computes**, not one re-expressed in `ffmpeg`'s expression
language.

That is the decision: a second implementation of the ease vocabulary, in a different
language, with different arithmetic, would be a second spelling of ADR-0012's interpolation
that could drift from the first without anything noticing. The level therefore steps per
frame rather than gliding; at 25 fps that is 40 ms, finer than the `volume` filter's own
evaluation interval at `eval=frame`, so the step is below what the filter could have
resolved anyway.

The instants are `render::instant_of`'s — reading 1's floor — for the reason stated there:
the fade must be heard on the clock it is seen on.

## Not a reading: one `ffmpeg` seek per `video` frame

Recorded, deliberately unratified. A `video` element is decoded through the same
`decode::frame_at` call `frame` makes, which is one spawn per frame. It is correct and it is
slow.

It is not a reading because no ADR is being extended and no choice is being defended: it is
a cost nobody has measured. The only render wall clock this project has taken is over the
committed fixture (`montagent_render::budget::RENDER_REFERENCES`), which has **no `video`
element** — spec [#168](https://github.com/MBehtemam/Montagent/issues/168) names this as the
case most likely to be missed — so the saving a streaming decoder would deliver is
currently invented rather than measured.

This is [ADR-0072](0072-the-render-budget-is-retired-not-replaced.md)'s rule applied one
level down: a number read off nothing is a larger claim than no number. A later ticket
measures a `video`-carrying project before it optimises one.

## Not ratified here: the fixture wall clock

#287's closing section quotes #215's 17.3 s whole-fixture render against *"the 130 s ceiling
`RENDER_MS_PER_OUTPUT_SECOND` implies"*. That ceiling no longer exists.
[#217](https://github.com/MBehtemam/Montagent/issues/217) deleted the constant and ADR-0072
retired the budget: `render` is observational, `Budget::Render.limit()` is `None`, and
`crates/montagent/tests/render_budget.rs` asserts that the harness reached `Verdict::Observed`
rather than passing against a limit. The 17.3 s reading itself survives, as the first row of
`RENDER_REFERENCES` and of ADR-0072's table.

Nothing is ratified from that section. It is named here so a reader following #287 to this
ADR does not conclude the ceiling was quietly kept.

## Consequences

- **ADR-0035, ADR-0011, ADR-0021, ADR-0009 and ADR-0055** each gain an "Amended by" banner
  pointing here, naming the reading that extends them.
- The *"raised for ratification"* passages in `crates/montagent-core/src/verbs/render.rs`
  and `crates/montagent-render/src/encode.rs` are replaced with citations to this ADR. **No
  behaviour changes** — this ADR ratifies what #215 shipped.
- Four readings, or halves of readings, that shipped with no test of their own now have
  one, so that a later change to any of them is a failing suite rather than a silent
  contradiction of this ADR:
  - reading 1, by the unit test named above;
  - reading 2's second half — nothing to render at all — by
    `a_project_with_no_duration_and_no_boundary_is_exit_3_with_nothing_to_render`;
  - reading 4's `--to` half, by
    `a_to_past_the_projects_end_is_legal_and_renders_the_whole_range`, which renders
    `[0, 2000)` of a project that ends at 1000 and reads 50 frames and 2000 ms back out of
    the file through `ffprobe`. Its `--from` half was already asserted, with the other
    invocation refusals, by `a_range_is_both_flags_or_neither_and_is_half_open` and its
    unit-test counterpart;
  - reading 5's fallback — the project file's own stem — by
    `a_partial_render_of_a_project_with_no_output_is_named_from_the_project_file`. The
    declared-`output` branch was already asserted by
    `a_partial_render_derives_its_name_and_can_never_land_on_the_deliverable`.
- The rest were already asserted: reading 2's derived extent by
  `a_project_with_no_duration_renders_to_its_last_boundary` and reading 3 by
  `a_project_with_no_output_and_no_flag_is_exit_3_naming_the_field`, reading 6 by `the_output_is_the_declared_frame_never_the_proxy_target`
  (a frame odd on both axes), reading 8 by the mixed stream's `sample_rate` of 48000, and
  reading 9 by `volume_speed_and_loop_go_through_the_mix`, which reads the keyframed fade's
  level back out of the written file at two instants.
- **Reading 7 is asserted only by the file being decodable.** No test names `libx264`, the
  CRF or the bitrate; `tests/reference_video.rs` decodes the rendered fixture and compares
  it against the published video, which would fail on a codec no decoder reads but not on a
  different quality setting. That is proportionate — the settings are an argument list with
  no reader downstream of `ffmpeg` — and it is stated here so the gap is a known one.
- **CRF 20 and preset `medium` are ratified as defaults, not as measured optima.** They are
  the one pair in this ADR a later ticket should expect to revisit with numbers.
