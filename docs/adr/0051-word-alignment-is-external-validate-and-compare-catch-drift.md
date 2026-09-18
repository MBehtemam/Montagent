---
status: accepted
amends: 0048 (fulfills the deferred authoring-time-tool obligation), 0011 (confirms the tool surface is unchanged), 0006/0032 (extends validate's and compare's fact vocabularies)
---

# Word alignment is a documented external workflow; `validate` and `compare` catch the failure modes it can leave behind

[ADR-0048](0048-per-word-highlighting-is-a-timed-window-on-the-run.md) settled that
per-word (karaoke) highlight timings are literal `start`/`end` millisecond integers on a
run, frozen at authoring time, and named an obligation it did not discharge: *"the project
now owes an authoring-time tool to produce word times. None exists."* The fixture's own
source materials confirm the gap is real — per-sentence audio and a plain transcript, no
word-level timestamps anywhere, and no `\k` karaoke tags in the old pipeline's `.ass`
subtitle files either.

## Decision

**Producing word-level timestamps is a forced-alignment step the agent runs entirely
outside Montaget.** Montaget does not invoke, ship, or shell out to a forced aligner.
**Montaget also provides no dedicated ingestion tool.** The agent normalizes whatever its
chosen aligner emits, merges it into matching runs itself using its own general-purpose
tools, and submits the result as ordinary complete-element writes under the existing
write-tool invariant. **`validate` gains two `error`-level checks on `highlight` windows,
and `compare` gains one new reported fact** — both closing the specific failure modes a
hand-scripted merge can produce, now that nothing else in the pipeline catches them.

Decided by three rounds of `/court` (Opus, Haiku, Fable, 3 jurors per question, blind to
each other) — **unanimous on every one of six questions across two rounds.**

### Alignment stays outside Montaget entirely

A forced aligner (Whisper-based, Montreal Forced Aligner, Gentle) is itself a model, with
weights, versioning drift, and non-deterministic output across hardware and library
versions. Shipping or invoking one inside Montaget's binary puts a model inside the tool
CONTEXT.md says never contains one — *"if it did, renders would stop being reproducible
and file-as-truth would die with them."* FFmpeg/ffprobe are not a counter-precedent:
`ffprobe` is a deterministic reader whose output is a pure function of its input file, not
an ML model producing an estimate. ADR-0048's freeze pattern already resolves this
architecturally — the timestamp is authored once, by whatever agency is willing to own the
estimate, and becomes a durable literal the renderer merely obeys. Where the literal came
from is authoring provenance, the same status a hand-scrubbed clip in-point already has;
letting Montaget regenerate it on demand would make the freeze decorative.

### No ingestion tool: the agent scripts the merge itself

The write-tool invariant ("a tool that writes may only take a complete element; no tool
takes a field name or an element id") governs *shape*, not who performs the computation.
An agent that has already run an external aligner can trivially script the merge of its
output into matching runs and submit the result as complete-element writes — this is a
small, deterministic transform the agent is already equipped for. What is genuinely
open-ended is the *matching* step (aligner tokens vs. run text: punctuation, contractions,
multi-word spans, repeated words), which is judgment, not transcription — exactly the kind
of problem this project keeps refusing to freeze into a rigid tool ahead of evidence. A
fixed tool would need to grow flags for tokenizer policy and fuzzy matching, which is the
CRUD-surface creep the invariant exists to prevent. Montaget's leverage is validation, not
ingestion (see below). If practice later shows agents systematically mis-merging, the
narrower fallback is a pure compute verb — text element and alignment data in, a
proposed element out, nothing written — which stays inside the invariant; that door is
left open, not adopted here.

**Consequence for [ADR-0011](0011-tool-surface-reads-checks-renders.md): the tool
surface is unchanged.** No verb is added for alignment or ingestion; the nine-verb, two-
resource surface stands.

### A documented (unenforced) intermediate schema

Different aligners emit different raw shapes and units (commonly seconds, as floats).
Montaget's documentation states one canonical intermediate convention agents should
normalize to before merging: **a list of `{word, start, end}` objects, `start`/`end` in
absolute integer milliseconds** — the same unit and time-base every other literal time in
the document already uses. Nothing parses or enforces this; it exists purely so the
sharpest edge (a silent unit mixup that would validate clean and render wrong) has a
stated target, and so bug reports, examples, and future ADRs share a vocabulary for "the
word-timing list." Marked explicitly in the docs as convention, not a contract.

### `validate`: containment and non-overlap

Two new `error`-level checks, both intra-document — no external reference needed, so this
is squarely `validate`'s territory (per [ADR-0006](0006-validate-reports-facts-and-render-enforces.md)),
not `compare`'s:

1. **Containment.** Every run's `highlight.start`/`highlight.end` must fall within its
   parent element's own `[start, end)` range (half-open, matching every other interval in
   this format). Catches the highest-frequency merge-script failure: a whole-document unit
   or offset mixup that would otherwise validate clean and render every word wrong or
   invisible.
2. **Non-overlap.** Sibling runs sharing one parent element must not have overlapping
   `highlight` windows — two different words can't both be "the highlighted one" at the
   same instant, which follows directly from what a highlight window means. Catches a
   distinct, more insidious failure containment can't see: alignment data sliced against
   the wrong sentence, or a script that copies one whole-sentence window into every run —
   individually well-formed, only visible as siblings compared against each other.

Both are cheap (sort siblings by `start`, walk adjacent pairs) and forbid intentional
future extensions (a crossfade between adjacent words) at `error` level for now; ADR-0048's
"multiple windows per run" extension point, if ever adopted, is where that gets revisited —
not assumed here.

### `compare`: one new fact, not a general mechanism

**`compare` reports when a run's `text` differs between two compared versions while its
`highlight` object is unchanged** — a fact, not a judgment (a typo fix with the same audio
is legitimate; the report doesn't claim otherwise), in the same grammar `compare` already
uses ("19 of 20 elements moved +800ms; one did not"). This is the same organizing
principle [ADR-0032](0032-slack-is-invariant-shift-refuses-compare-is-the-backstop.md)
already established: *"every check that compares this document to itself, it passes —
only checks that compare it to something outside it find anything."* Both document states
are independently legal; the drift is visible only in the delta between two versions,
which only `compare` receives.

**Implemented as a one-off, hardcoded check for this specific pair — not a generic
"field X measures field Y" mechanism.** A one-row dependency table would be exactly the
failure this project's own precedent already named and rejected (ADR-0045's
division-exactness rule, refused with one instance as "asserted, not earned"). `highlight`
is richer than a bare "changed/unchanged" pair could show — the useful report may need to
name *which words* lost their timing anchors — and a generic predicate would force the one
real case down to the weakest shape both it and a hypothetical second case could share.
Two concrete checks, if a second frozen-measurement field ever needs one, make the real
common structure obvious and cheap to extract; a wrong abstraction adopted from a single
instance is neither. This ADR states only the one check; if a second instance arrives,
that is the evidence to generalize from, not this one.

Out of scope, named rather than silently dropped: detecting drift against the *audio*
itself (a re-recorded take with unchanged text/wording would drift invisibly to `compare`,
which is document-to-document, not document-to-disk) — a `validate`-against-media question
for a future ticket, not folded into this decision.

## Consequences

- The tool surface (ADR-0011) is unchanged: no alignment or ingestion verb is added.
- Montaget's docs gain one stated, unenforced convention: `{word, start, end}` in absolute
  integer milliseconds as the normalization target for any aligner's raw output.
- `validate` gains two `error` findings on `highlight`: out-of-parent-range, and
  sibling-window overlap.
- `compare` gains one new reported fact: a run's `text` changed while its `highlight`
  object did not, between the two compared versions.
- No generic "measurement depends on field" mechanism is adopted in `compare`. A second
  frozen-measurement field needing the same treatment gets its own hardcoded check first;
  generalizing from two real instances is the next ticket, not this one, if it ever comes.
- Drift between a `highlight` window and the *audio on disk* (as opposed to the run's
  text) is named as an open gap, not addressed here.

## Evidence

Two rounds of `/court`, three jurors each (Claude Opus, Claude Haiku, Claude Fable —
independent, blind to each other's ballots), six questions total. **Unanimous 18/18**
across every ballot in both rounds, with one juror (Fable) explicitly reversing a
generalization it had floated informally between rounds once the question was put to it
neutrally — evidence the panel was reasoning from the project's stated precedents rather
than anchoring on a prior framing.
