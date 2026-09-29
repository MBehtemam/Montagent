---
status: accepted
amends: 0006 (closes the gap between `validate`'s classes and `render`'s prose: a world-effect is now an `error` like any other finding, and the two verbs read one structure rather than two — the `validate`/`render` split this ADR designed held for the *document* and not for the disk), 0043 (adds sixteen codes whose repair form is fixed per code, and records that per-reason coding is forced by that rule rather than chosen), 0056 (amends the `UncheckedReason` enumeration with its first non-network member, `Unidentified`), 0021 (reads *"never silently"* at the level of the deliverable: an `error`-class finding withholds the file, so a file at the output path is a render with zero errors)
---

# `render`'s world-effects are findings, and an `error` withholds the deliverable

> **Amended by [ADR-0109](0109-a-cancelled-encode-publishes-nothing.md).** Ruling 6's
> walk-away has a third trigger: the MCP caller cancelling. A cancelled `render` or `preview`
> stops before its next frame and publishes nothing, so a file at the output path is a
> zero-error render that nobody cancelled.

> **Amended by [ADR-0096](0096-the-frame-at-an-instant-is-the-last-one-starting-at-or-before-it.md).**
> Ruling 6's condition 1 — *"the seek predicate is computable before the frame loop"* — is
> **not true of the fine predicate**: whether a given seek lands needs the source's real frame
> timestamps, and neither frame rate `probe` reports is the source's frame grid (ADR-0096 §2).
> The ruling survives in a weaker form, because ADR-0096's clamp removes the case it was about;
> what is left pre-flightable is the coarse question *"does this source end more than a window
> before the declared range?"*, raised as
> [#413](https://github.com/MBehtemam/Montagent/issues/413). `E-NOT-PAINTED-UNDECODABLE` keeps
> the case either way, and this ADR still withholds the deliverable for it.
>
> **Amended by [ADR-0107](0107-an-empty-range-is-validates-error-under-renders-code.md).**
> The empty range is no longer a `render`-only finding: `validate` states `E-EMPTY-RANGE` under
> the same code, so the check engine refuses first and ruling 2's two arms are `E-INTERNAL`.

> **Amended by [ADR-0104](0104-the-output-path-is-checked-for-a-foreign-deliverable-before-the-encoder-runs.md)**,
> which extends ruling 6 outward. This ADR made *a file at the output path a render with zero
> errors* — a statement about the file the render **produces**, silent about the file it
> **replaces**. The invariant now also carries *and the promotion destroyed nothing this
> project did not write*: `render` stamps what it publishes and reads that stamp back before
> writing again, refusing at `E-OUTPUT-FOREIGN` before the encoder is spawned. It also gives
> this ADR's `preview` exemption a boundary — exempt from *withholding*, never licensed to
> clobber, which `preview --output` had left open.

[#386](https://github.com/MBehtemam/Montagent/issues/386) applies
[#384](https://github.com/MBehtemam/Montagent/issues/384)'s six rulings to the case they were
decided for. Both arrive from the same place ADR-0092 did: the first real end-to-end build
through this tool, which shipped an eleven-minute cut with 134 of 217 audio elements missing,
at `0 errors`, twice.

ADR-0092 fixed the mechanism that lost the probes. It says so in its own Scope section, and
it says what it left: *"this ADR stops the mix bus losing probes it was handed, and #384's
ticket stops an empty mix bus from reaching the output path at all. Neither subsumes the
other, and a fix to either alone would still have shipped #385's cut — this one silently,
that one loudly."*

This is that second half. The signature it is aimed at is one sentence: **something legal
happened, it was wrong, and Montagent said `0 errors` anyway.**

## What was actually there

`validate` and `review` are scrupulous about the *document's* internal legality. Nothing in
the severity ladder covered what happens when the render acts on the world and something
legal-but-wrong occurs. Those events were reported — and reported as **prose in a list**:

```rust
// render.rs, before this ADR
Err(reason) => not_mixed.push(NotPainted { element: name, reason }),
```

with `reason` a `String` assembled at the arm that noticed. Twenty-five such arms across
`render` and `frame`. A `NotPainted` carried no code, so no class, so nothing counted it, so
the one-line summary every report starts with said `0 errors` — and the deliverable was
written anyway.

The prose was the whole defect surface, in three distinct ways:

1. **Unassertable.** No test could check that one reason was worded the same way twice,
   because nothing named the reason.
2. **Uncounted.** A class is what makes a reader do something. A sentence in a list is what a
   reader scrolls past.
3. **Non-blocking.** The file landed at the declared path regardless. That file is what a
   human uploads and what an MCP agent `stat`s.

## The decision

### 1. No new severity tier — these are `error`

`CONTEXT.md` already defines `error` as *"the render is refused **or is guaranteed wrong**"*.
A legal element the render declined to draw or mix is the second clause with nothing left
over. So is a field parsed and discarded.

A sixth class would encode only *where in the pipeline the fact surfaced*, and ADR-0006 fixes
class as *"computed from the consequence at an instant"* — pipeline position is not a
consequence. Classes are named for what the reader does, and the reader does nothing
different here. Exit 1 already means *"error findings — fix the project"* and needs no
extension.

**But the class is `error` from `render` and `review` from `frame`, and #384 did not see
this.** The implementation found it, in the form of a committed test failing for the right
reason. `effects.rs` asserts, in a comment that cites ADR-0006 as its authority:

> *"`frame` runs no checks (ADR-0006 gives `render` the enforcement), so a document carrying a
> member the format does not have **still gets a picture**. What it must not get is silence."*

That is a contract, and it is the one an agent inspecting a half-written document depends on.
Making every painter finding an `error` would have raised `frame`'s exit code — a behaviour
change to a verb this effort is not about, smuggled in as a side effect of a reporting fix.

ADR-0006's own rule resolves it without a new tier, and resolves it *better* than a uniform
`error` would: the consequence genuinely differs. From `render`, an element the painter
declined means **the deliverable is guaranteed wrong** — `error`, and ruling 6 withholds the
file. From `frame`, it means **look at this frame: the element you asked about is not in it** —
which is `review`'s definition almost verbatim. One code, two consequences, computed from the
consequence rather than from the check. The registry's `classes` set exists for exactly this
and had until now only been used across *instants*; this is the first use across *verbs*.

**Repair form does not vary with it**, and that asymmetry is the point of ADR-0043: class is
the axis a check may compute, repair form is the axis it may not.

### 2. One code per reason, and the reason set is closed

`not_mixed` / `not_painted` / `painted_partially` do not become one code whose class is
computed per instance. ADR-0043 forces the split: repair form is decided **once, when the
check is written, and holds for every instance it matches.** One code cannot be a `note` for
*"its source carries no audio stream"* and refuse-class for *"the engine established nothing
about its source"*. ADR-0006's per-instance freedom governs **class**, not repair form.

Sixteen codes, in `registry.rs`. The ticket expected roughly twenty-five, and where the other
nine went is the part worth recording, because **the argument that removes them holds for one
half of the render and not the other.**

> `render` and `preview` refuse on any `error` and then call `document.strict()`. So no
> document that reaches `Mix::of` can carry a value the model cannot represent.

For the **mix**, that closes eight arms outright. `source_start` is a required `i64`. `Speed`'s
`Deserialize` refuses `<= 0`. `Volume`'s refuses negatives. `AudioOverrun` has no `hold`
variant at all. Reaching one of those arms does not mean the project is wrong — it means the
check engine and the renderer disagree about one document, which is ADR-0073's `E-INTERNAL`
and exit 70. Reporting it as a finding would send an agent to edit a file that is not wrong.

#384 ruling 2 guessed exactly this for `overrun: "hold"`, hedging at *"probably not a
world-effect finding at all"*, and left the call here. **The type system settles it:** the
model cannot represent the document that would reach that arm.

**For the painter it is false, and this ADR got it wrong once before getting it right.** The
same collapse was applied to the painter's type-level arms and two committed tests failed —
`effects.rs`'s *"a transition that cannot be read is named rather than silently inert"* and
*"an effect the vocabulary does not admit is named beside the picture"*. They were right and
the reasoning was wrong: **`frame` never calls `strict()`.** It reads the document
*permissively* and draws it, which is its job — an agent asks `frame` what a picture looks
like precisely when the document is not yet right. One painter, two callers, and only one of
them has done the refusing.

So `E-NOT-PAINTED-UNDRAWABLE` exists, and it is one code rather than six: the *condition* is
one condition — the element cannot be drawn as declared — and which key carries the undrawable
value is a field. A `type` the format does not have, a `transition` with no `kind`, a source
offset that would not resolve: all reachable, all facts about the project, none of them
reachable from `render`. `E-EFFECT-UNKNOWN` is separate only because its consequence is
different in the way the report is organised around — the element *was* drawn, minus one thing
it asked for, which is the `painted_partially` list rather than `not_painted`.

Two further riders on the boundary:

- **A reason's open-ended sub-prose is a `{detail}` field, not its own code.** An `ffmpeg`
  message and an `io::Error` are not a closed set and never will be. What is closed is the
  *condition* — "this source did not decode" — and the condition is the code.
- **Two reasons were reclassified on the way through**, as ruling 2 required the implementing
  ticket to decide. `"its source carries no audio stream"` is an ordinary video, not a defect:
  `N-NO-AUDIO-STREAM`, a `note`. The element is still not in the mix and the report still says
  so, because a reader who cannot tell *"silent by nature"* from *"dropped"* chases the wrong
  thing.

**Two arms turned out to be reachable that nobody had claimed**, and finding them is the
concrete return on closing the set:

- **An empty range.** `E-EMPTY-RANGE`. `start`/`end` is the one cross-field fact the schema
  cannot express, and no `validate` check states it: a project with `"start": 0, "end": 0`
  validates at **zero errors** today, verified against a built binary. The render genuinely
  reaches it. That `validate` should state it too is a real gap and is left as one, ticketed
  separately rather than smuggled in here as a new check.
- **A transition's dangling reference.** `E-NOT-PAINTED-UNRESOLVED-REF`.
  `checks::transition` declines the question in as many words — *"a `from`/`to` naming an
  element that is not in the project at all is a dangling reference, not a drifted one … a
  different question this check declines to answer"* — and hands it to *"whichever check owns
  that question"*. No check ever claimed it. The crossfade simply did not happen.

### 3. `validate` needs no new outcome class — it needs one data path

`UNCHECKED` already exists: *"the question was unanswerable — an unprobeable source"*. It
reported `0 unchecked` on the file `render` then declined to mix 134 elements of because
**the two verbs answered one question from two data structures.** `validate` mapped a probe
outcome onto findings; `render` reached into `Report::media` and compared canonical paths
itself. Neither verb was wrong about its own data. That is the defect.

Three parts, and the third is the one that keeps it fixed:

- **One shared function**, `media::established`, read by both verbs. A consumer that
  re-derives usability from `Report::media` has forked the data path again, and the module
  says so at the top.
- **One new `UncheckedReason` variant.** ADR-0056's enumeration is amended with
  `Unidentified`, which is not a network reason and is the enum's first such member. Its
  inhabitant is precise, and worth stating because it is what remained after ADR-0092: a
  **local** source `ffprobe` answered for, whose canonical path the run could not observe.
  The content facts are real, but ADR-0092 makes the observed identity the only admissible
  answer to *"is this the same file?"*, so every consumer that must tie a probe to a file has
  to decline it — `render` included. Before this it was a clean pass to `validate` and a
  silent drop in `render`: the last remaining route to the MONTAGENT-1 silence.

  ADR-0056's deliberately absent `missing` variant is **not** violated. The file exists; what
  is unknown is which file it is.
- **A cross-verb invariant, as a property over both verbs:**

  > **`validate`'s unchecked set contains every source `render` declines to use.**

  A containment and not an equality, on purpose: `render` mixes local sources only, so it
  declines a remote one `validate` probed perfectly well — a fact about the verb, not the
  file. The direction that must never hold is the other one. Asserted across both verbs
  rather than as a unit test on either, because a unit test on one path is exactly what let
  the two drift apart.

### 4. `render` does not promote the deliverable when an `error` fired

Any `error`-class finding means the temp file is **not promoted**: exit 1, and no file at the
output path. `render` already writes through a temp path and one atomic rename, so declining
to promote is the absence of a promotion, not destruction.

The invariant this buys is the one that failed:

> **A file at the output path is a render with zero errors.**

What shipped the mute cut was not an unread report — it was **a plausible file existing**. A
counted finding is necessary and not sufficient. This is ADR-0021's *"never silently"* read
as ADR-0077 read it: about the deliverable, not the prose.

**Named plainly, because it is a behaviour change beyond the reporting fix:** `render` can
now spend wall clock and produce nothing where today it produces a file.

Mechanically, `Encoder::finish` splits into `seal` and `publish`. A `Sealed` is a complete
encode that has not been published; the caller holding the report decides. `preview` is
untouched by the rule and publishes as it always has — ADR-0093 is about *the deliverable*,
and ADR-0065 discloses a proxy as a proxy while ADR-0021 keeps `render` the only verb that
writes the declared `output`.

The rejected alternative was refusing pre-flight while keeping mid-loop output. It failed on
arithmetic rather than doctrine, and #384's own three-juror court was unanimous against it.

**Condition 1: everything pre-flightable is pre-flighted.** Under the no-promotion rule, wall
clock is only ever wasted on a genuinely mid-loop error, so minimising that set is part of
this decision and not a later optimisation. What that came to here:

- **The mix is wholly pre-flighted.** `Mix::of` is a pure function of the document, the
  established facts and the range, and it already ran before `Encoder::start`. So every reason
  an audible element is not in the output is known before a subprocess exists: the span stops
  at `Stop::Refused`, with no encoder, no temp file and no wall clock. **This is
  MONTAGENT-1's own case**, and it costs nothing.
- **The painter's reasons are discovered in the frame loop**, and are what the non-promotion
  rule is actually for. Pre-flighting them fully would mean re-deriving the presence set over
  the whole timeline — a second traversal that could disagree with the first, which is the
  defect ruling 3 exists to kill. Not taken.
- **MONTAGENT-2's failed seek** surfaces at `E-NOT-PAINTED-UNDECODABLE` in the meantime.
  [#387](https://github.com/MBehtemam/Montagent/issues/387) removes the case outright by
  clamping the seek, which is this condition discharged rather than deferred: #384 established
  that the predicate is computable before the frame loop.

**Condition 2: no bypass in this effort.** A waiver, if one is ever wanted, is explicit,
per-code and author-typed, and *downgrades the finding* — never a flag that lets an unwaived
error reach the output path. Keeping the unpromoted temp file at a clearly non-deliverable
path for inspection is a compatible and cheaper alternative. Either way it is a separate,
later ADR.

## Scope

**`render`'s world-effects only.** The six findings share one signature — legal, wrong,
`0 errors` — and `review`-class findings never have it: they are classed, counted and
exit-visible already. Reaching into them re-litigates ADR-0006's class computation for every
existing check, which is past this effort's destination. Recorded as out of scope in #384 §5.

Two neighbours this ADR deliberately does not settle:

- **General policy on collapsing repeated findings, severity floors and summary modes** stays
  with [#388](https://github.com/MBehtemam/Montagent/issues/388). #384 ruling 4 ruled only
  that `census` is the mechanism; `E-TRACK-OVERLAP`'s per-track census is
  [#389](https://github.com/MBehtemam/Montagent/issues/389)'s.
- **`E-FIELD-UNHONOURED`** is registered here, with the field name as a *field*, and
  [#390](https://github.com/MBehtemam/Montagent/issues/390) reuses it for a font chain's
  `index`. Its first instance is `runs[].dir`. One code and not one per key: the *reason* is
  one reason — "this build parsed a declared field, validated it, and drew without it" — and
  ruling 2's per-reason rule is about the condition, not about how many document keys can meet
  it.

## Consequences

- **`NotPainted` carries a `code` where it carried a `reason`. This is a breaking change to
  machine-readable output.** The sentence has not moved far — it is the registered template for
  that code, rendered into the report's findings list, where it now arrives with a class and is
  counted. The prose row in the render block becomes the index into that finding, so one reason
  can no longer be worded two ways in one report.
- **`render` can now exit 1 having produced no file.** The behaviour change above, stated where
  a reader looking for it will be.
- **`frame`'s exit code is unchanged.** Its world-effect findings are `review`, so a document
  it cannot fully draw still answers with a picture at exit 0 — and now says, at a stable code
  and with the offending value inline, what is missing from it. `frame` had to start pushing the
  painter's findings onto its report for that to be true: before this the painter's list was the
  only record, and moving the sentence into the finding without moving the finding into the
  report would have left `frame` stating a condition and never an instance.
- **An empty audio mix can no longer reach the output path**, which is the MONTAGENT-1
  criterion.
- **A dropped element now names its own reason at a stable code**, so `compare` can diff on it
  and an agent can suppress a class.
- **A missing `ffmpeg` keeps its own door.** `Declined::Tool` is a third channel beside the
  finding and the contradiction, so ADR-0091's `E-TOOL-MISSING` and exit 70 survive the
  conversion instead of being flattened into a claim about the source.
- **Eight of the mix's arms became `E-INTERNAL`.** Montagent contradicting itself is now
  reported as such rather than as a defect in the user's project. The painter's equivalents did
  **not**, because `frame` paints a permissively-parsed document — the asymmetry is documented
  at `E-NOT-PAINTED-UNDRAWABLE` for the next reader tempted by the same collapse.
- **No sidecar version bump and no re-probe.** Nothing about the probe format changed.

## Evidence

Argued from committed source — `CONTEXT.md`'s class definitions, ADR-0043's uniformity rule,
the model's own `Deserialize` impls, `checks/transition.rs`'s declined question — and pinned by
re-executable checks rather than by the lost session the ticket describes.

- `crates/montagent-core/tests/world_effects.rs` — an unmixable audible element exits 1, is a
  counted `error` at its own code, and leaves nothing at the output path *or beside it*; a clean
  render still publishes; an empty range is `E-EMPTY-RANGE`; an ordinary silent video is a `note`
  that does **not** withhold the file. The last of these is the other direction, without which
  every assertion above is satisfied by a `render` that has simply stopped working.
- `crates/montagent-core/tests/cross_verb.rs` — ruling 3's containment, over both verbs. It
  asserts that `render` declined *something* before asserting the containment over it, so the
  property cannot pass vacuously on an empty set.
- `crates/montagent-core/tests/effects.rs`, `tests/frame.rs` — the six *"named rather than
  silent"* tests, rewritten to read the code from the row and the offending value from the
  finding. They are the reason the painter's class is `review` rather than `error`: two of them
  failed against a uniform `error` and were right to.

- **`docs/adr/world_effects_withhold_the_deliverable_check.sh`** — this ADR's claims asserted end
  to end against a built binary, exiting non-zero the moment one stops holding. A test at the
  verb seam can assert the report; only a run of the real binary can assert what is on disk
  afterwards, and what is on disk is the thing that actually misled someone. Checked in **both**
  directions before `status: accepted` was written:

  | run against | result |
  | --- | --- |
  | this branch | `ADR-0093 holds` — 11 assertions, exit 0 |
  | `bb26c062` (the commit before it) | **7 failures**, exit 1 |

  The base-commit run is the reproduction of MONTAGENT-1 that the ticket could not supply — the
  original session is gone — and it is reproducible rather than recounted. On an audible element
  whose source is present and unopenable, `bb26c062` reports:

  ```
  0 errors, 0 reviews, 0 notes, 1 unchecked, 1 layout, 0 drift
  RENDER  …/out/p.mp4 — 1000 ms, 25 frames at 25 fps, 200x200
  ```

  exit 0 — and `ffprobe` on the published file answers `video`, with no audio stream. That is
  the eleven-minute silent cut in miniature: **`0 errors`, a plausible file, and nothing in it.**
  Note the `1 unchecked`: `validate` had already said it could not read that source, in the same
  session, and `render` published over the top of it anyway. A counted finding is necessary and
  not sufficient, which is ruling 6's whole argument.
