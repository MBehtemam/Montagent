---
status: accepted
amends: 0093 (extends the promotion rule outward: it made a file at the output path mean *a render with zero errors*, and said nothing about the file that was already there. The invariant now also carries *and the promotion destroyed nothing this project did not write*. Adds the encoder's first identifying metadata, and gives `preview`'s exemption a boundary — exempt from withholding, not licensed to clobber), 0021 (`render` is still the only verb that writes the declared `output`; this states what happens when something else already has)
---

# The output path is checked for a foreign deliverable before the encoder runs

> **Numbered 0104 and not 0103.** A concurrent session on
> [#406](https://github.com/MBehtemam/Montagent/issues/406) took 0103 while this one was
> being written — the second time this effort has hit it (ADR-0099 records the first, on
> #400). The map's parallelism meets a shared resource with no allocator; the later session
> yields, as it did then.

[#391](https://github.com/MBehtemam/Montagent/issues/391), the seventh of the nine findings
from the first real end-to-end build through this tool
([#383](https://github.com/MBehtemam/Montagent/issues/383)).

The incident, in the reporting agent's own words:

> `build_episode_project.py` hardcoded `"output": "../../out/da-episode.mp4"`. The German
> project inherited it, so rendering German would have written **over the finished Danish
> deliverable** — an eleven-minute cut that had taken four attempts and about ninety minutes
> of render time to get right. Nothing would have reported it: the file is legal, the path
> resolves, `validate` is clean, and `render` would have said `0 errors` while destroying the
> previous answer.
>
> I caught it by grepping the project before rendering. No tool was involved.

It is the map's signature exactly — **something legal happened, it was wrong, and Montagent
said `0 errors` anyway** — with one difference from its six siblings that decides this ADR:
the thing that was wrong is not in the document at all. Every other finding in this effort is
about what the render did to *its own output*. This one is about what it did to **a file that
was already there**, which no amount of reading the project can discover.

## What was actually there

ADR-0093 had just bought the invariant *a file at the output path is a render with zero
errors*, by making promotion conditional on the report. It is a statement about the file the
render **produces** and it is silent about the file the render **replaces**. `Deliverable`'s
one atomic rename is scrupulous about never leaving a half-written MP4 that reads as finished
— and renames straight over whatever occupied the path, because nothing had ever looked.

So the care was real and it was pointed one step too late. The ticket says so: *"`render`
already writes through a temp path and an atomic rename so a truncated MP4 never reads as
finished — the same care taken one step earlier would close this."*

## The decision

### 1. The predicate is an attestation, and this is the whole of the ADR

The ticket proposed a cheaper test, and it is wrong in a way worth recording because it is the
obvious thing to reach for:

> an existing file at that path whose **duration or frame count** disagrees with the document
> is worth one line.

**Its two failure modes invert the defect.** Two language cuts of one timeline agree on
duration, frame count and frame size to the frame — so on the incident this ADR exists for, a
duration comparison is *silent*. And a project that has been edited at all disagrees with its
own last render — so on attempt four of a cut that took four attempts, it *fires*. Loud on the
safe case, silent on the dangerous one.

It is also, structurally, the thing this repo has already ruled against twice. ADR-0092 settled
that a **re-derived label is not an identity**, at the cost of an eleven-minute silent video;
duration-and-frame-count is a re-derived label. So:

> **`render` stamps what it writes, and reads that stamp back before it writes again.**

The encoder gains its first identifying metadata — `-metadata comment="montagent/1
project=<canonical project path>"` — and the question *"did this project produce the file
already at the output path?"* becomes one Montagent **observes** rather than one it infers.
The value is the project file's observed identity in ADR-0092's sense, and deliberately not a
label re-derived from the working directory.

This is the move `fontVendor` already makes for font bytes (ADR-0057): an attestation recorded
at the moment of the act, read back later. It is also the move `E-NOT-A-PROJECT` already makes
for `fmt`, and that entry's own comment states this ADR's argument two years early —
*"the reported hazard is wrong-file destruction … so the proportionate fix is an identity
check"*.

`comment` rather than a custom key, for a mechanical reason: the MP4 muxer maps a fixed set of
tags and silently drops anything else unless `-movflags use_metadata_tags` is set, which would
change the container's shape for every consumer.

### 2. Two codes, because ADR-0043 forces the split

The three-juror court empanelled on this ticket was unanimous on the predicate and **split 2–1
on the class**, which is the sharpest thing it produced. Both sides are right about different
conditions, and ADR-0043 is what makes that a split rather than a compromise: repair form is
fixed per code, so one code cannot be refuse-class for one reason and `review` for another.

- **`E-OUTPUT-FOREIGN`** — the file carries another project's stamp. Montagent has *observed*
  that this is somebody else's deliverable. `error`, refuse-class, and it withholds under
  ADR-0093 ruling 6. The repair is genuinely not derivable and the finding therefore carries
  no `repair` value: which of the two projects is meant to own the path is in neither
  document.
- **`R-OUTPUT-UNATTESTED`** — a file is there and carries no Montagent stamp at all.
  `review`. This is the dissenting juror's case and the argument is decisive: `error` is
  defined as *"the render is refused **or is guaranteed wrong**"*, and a guarantee is exactly
  what Montagent cannot offer about a file it has no evidence about. Classing it `error`
  would refuse the first render of **every project that existed before this check**, charging
  the migration cost to the wrong party for a file that is, in the overwhelming case, last
  week's scratch.

  Under `--no-clobber` the same code is emitted at `error`. That is ADR-0006's per-instance
  class rule doing precisely its job — the *condition* is identical and the *consequence* is
  not, because the caller has said no evidence is reason enough — and it is one code rather
  than a second borrowing `E-OUTPUT-FOREIGN`, which would name a foreign project for a file
  that has none. **The first draft did borrow it**, and the report would have been wrong
  about what it had observed. ADR-0043 is satisfied because repair *form* is still declared
  once for the code and does not vary with the class.

A file that exists and cannot be probed at all is `E-OUTPUT-FOREIGN` with an unnamed project,
not `R-OUTPUT-UNATTESTED`: bytes are there, the render is about to destroy them, and *"I could
not read it"* is the least safe moment to assume it is nobody's.

### 3. Pre-flight, because the injury is measured in wall clock

The check runs after `destination()` and **before the encoder is spawned** — before `ffmpeg`
resolves, before a frame is painted. ADR-0093 condition 1 requires it (*everything
pre-flightable is pre-flighted*), and here the requirement has teeth beyond tidiness: a
refusal that arrived at promotion time would save the file and still spend the ninety minutes.
It is externally checkable, because `Deliverable` writes to a dotted sibling: **a temp file
beside the output means the encoder ran before anyone looked at the path.**

### 4. `--no-clobber` survives, and only ever tightens

The ticket offered the flag as an *alternative* to the finding. It is neither that nor
redundant: it escalates `R-OUTPUT-UNATTESTED` from `review` to a refusal, which is how a batch
script says *"treat no evidence as reason enough"*. It covers the case attestation cannot
reach — a file a human deliberately placed at that path, which Montagent has never touched and
therefore cannot distinguish from an empty slot.

**There is deliberately no flag in the other direction.** ADR-0093 condition 2 forbids one
that would let an unwaived `error` reach the output path, and `E-OUTPUT-FOREIGN` is such an
error. *"I know, do it anyway"* is that flag by another name. A caller who genuinely means to
replace another project's deliverable changes `output`, or moves the file — and if a waiver is
ever wanted it is ADR-0093's: explicit, per-code, author-typed, recorded in the document, and
leaving evidence in the artifact that the clobber was chosen.

CLI-only, for ADR-0011's reason that kept `probe` off the MCP surface — a tool schema costs
context on every turn, and the case the flag exists for is a batch script. The MCP caller
still gets the entire default rule.

### 5. `preview`'s exemption gets a boundary

ADR-0093 exempts `preview` from the no-promotion rule because a proxy is not a deliverable.
Checking the implementation turned that exemption into a real hole, and it is the one finding
here that no ticket had named: **`preview` honours `--output`.** It already refuses the
*previewing project's own* declared `output` (ADR-0021), so it cannot destroy its own
deliverable — but nothing stopped it renaming a proxy over **another project's** finished cut,
which is MONTAGENT-7 one verb across. The exemption was one step from being the bypass.

So `preview` runs the same predicate, on a deliberately narrower rule: **`Foreign` refuses,
`Absent` does not.** A preview writes no stamp — it must not hand a later `render` a forged
licence — so the previous preview at any path is unattested by construction, and refusing that
would refuse the second preview of every project.

## Scope

**The promotion, not the field that named it.** Every path `render` promotes carries the rule —
the declared `output`, `--output`, and the derived partial-render name — because the hazard is
the rename onto a real path and a caller who passes `--output` over an irreplaceable file has
the identical accident. `frame`'s PNGs are excluded: one instant, seconds to regenerate, not a
deliverable.

`validate` is silent, and that is a decision rather than an omission. MONTAGENT-7's signature
is literally *"`validate` is clean"*, and the cure is a `render` that refuses rather than a
`validate` that mutters about an intent it does not hold. Three further reasons, each
sufficient: ADR-0006 gives world-effects to `render`; `validate` cannot see `--output`, so any
verdict it gave would be about a path the render may never touch; and a file-at-output fact is
true at validate time and false at render time, so a clean `validate` would read as a licence
it cannot issue. Two verbs answering one question from two data paths is the defect ADR-0093
ruling 3 exists to kill, and it is not being reintroduced here.

## Consequences

- **A deliverable's container now carries a `comment` tag.** This is the encoder's first
  identifying metadata and it is visible to anyone who opens the file's properties.
- **The bytes of a render now depend on where the project file lives.** Determinism in
  `CONTEXT.md`'s sense is unharmed — *the same project and the same files* still produce the
  same video — but two copies of one document at two paths no longer produce byte-identical
  MP4s. This surfaced immediately as a committed test asserting byte-identity across two
  projects; it was restated to blank each file's own attestation rather than weakened, so
  every other byte is still compared.
- **A project tree that moves reads its own earlier deliverables as `Foreign`.** Stated as a
  cost rather than hidden: it is the price of ADR-0092's rule that observed identity beats
  convenience, and the move is to render again or to move the file.
- **Every deliverable that predates this ADR is unattested**, so the first render over one
  reports `R-OUTPUT-UNATTESTED` and proceeds. One `review` per file, once.
- **`render` can now refuse before doing any work at all** — a third refusal shape beside
  ADR-0093's *spend the clock and produce nothing* and the ordinary pre-flight refusal.
- **`preview` can now be refused**, which it could not be before except on its own project's
  `output`.
- **No sidecar version bump and no re-probe.** Nothing about the probe format changed;
  `media::attest` reads a file directly and caches nothing.

## Evidence

- `crates/montagent-core/tests/output_clobber.rs` — six tests over the verb seam. The suite's
  *shape* is the decision: it renders one project twice (which must be silent) before
  rendering its sibling at the same path (which must refuse), because a suite that only
  asserted the refusal would be passed by a `render` that had simply stopped overwriting
  anything.
- `crates/montagent-core/tests/render.rs` — the `loop`-flag byte-identity test, restated as
  described above. It now also proves the stamp reaches the container, since it fails if the
  attestation is not found in the file.
- **`docs/adr/output_attestation_check.sh`** — this ADR's claims asserted end to end against a
  built binary, exiting non-zero the moment one stops holding. A test at the verb seam can
  assert the report; only a run of the real binary can assert that the earlier cut's bytes are
  still on disk afterwards, and the bytes are what the incident would have lost. Checked in
  **both** directions before `status: accepted` was written:

  | run against | result |
  | --- | --- |
  | this branch | `ADR-0104 holds` — 14 assertions, exit 0 |
  | `9974ba40` (the commit before it) | **9 failures**, exit 1 |

  The base-commit run is the reproduction the ticket could not supply — the near-miss was
  caught by hand and never happened — and it is reproducible rather than recounted. Against
  `9974ba40` it reports, in as many words:

  ```
  FAIL: the earlier deliverable was destroyed
  FAIL: a preview destroyed a deliverable
  ```

  with the sibling render exiting 0 at `0 errors, 0 reviews, 0 notes, 0 unchecked`. That is
  MONTAGENT-7 in miniature: **a legal document, a resolved path, `0 errors`, and the previous
  answer gone.**

  **Two of its assertions were weakened by their own fixture and had to be repaired**, which is
  worth recording because both passed against the base commit while proving nothing. The
  `--no-clobber` refusal was asserted as *"exit non-zero"*, which a commit lacking the flag
  satisfies via `clap`'s own argument error — now pinned to exit 1 exactly, the findings code.
  And the two sibling projects were initially identical apart from their filenames, so the
  deterministic render replaced the deliverable with a **byte-identical** file and the
  destruction was invisible to a hash comparison; they now differ in background colour, which
  is also the better fixture for the argument, since duration, frame count and frame size stay
  identical and are exactly what the rejected heuristic compares.
