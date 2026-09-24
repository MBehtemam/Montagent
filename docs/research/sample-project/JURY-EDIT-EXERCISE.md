# What it is like to work in this format, as an agent

> **Historical record.** This describes the sample project **as it was written for #9**,
> before ADR-0007 and ADR-0012 landed. Its field names are the pre-migration ones — `box`,
> `align` on images, `"text"` instead of `runs`, positional `scale` pairs. The current file
> lives at
> [`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json`](../../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json);
> what changed and why is in
> [`docs/research/sample-project-migration/`](../sample-project-migration/README.md), which
> also records three findings that land on ADR-0012 itself. Read this document for the
> *evidence and the juries*, not for the schema.


Five agents — Opus, two Sonnets, Haiku and Fable — were each given a **private copy** of
[`en-halloween-decorating.montagent.json`](./pre-migration.montagent.json),
`CONTEXT.md`, the five ADRs, and the fixture media. Each did the same three tasks and then
reported what the experience was like. Their edited files were scored by script, against
the invariants, independently of what they claimed.

Isolation was deliberate: ADR-0005 records that the #8 exercise had agents overwriting each
other's working directory. No agent could see another's copy, this document, `FINDINGS.md`,
the repo, or the issue tracker. The fixture README had its Corrections appendix stripped, so
nothing this project has already concluded could leak in.

**The tasks.** (1) Read: what is on screen and audible at 6.2 s and 47.5 s? (2) Edit:
`sentence-06-spider.mp3` has been re-recorded 800 ms longer — make the project correct and
legal. (3) Edit: move the sentence card and its text down 60 px, everywhere.

Task 2 is the edit ADR-0005 records **0 of 3 agents surviving**. It is nastier than it
reads: the re-recorded file has *two* placements, and the second plays it at `speed: 0.645`,
so 800 ms of source becomes ~1241 ms of timeline. The total is +2041 ms, which collides with
item 07, which five tracks agree starts at 30603.

---

## Scores

| | Task 1 read | Task 2 legal | Task 2 correct | Task 3 | Self-reported |
| --- | --- | --- | --- | --- | --- |
| **Opus** | ✅ | ✅ | ✅ | ✅ | "moderate-to-high" |
| **Sonnet A** | ✅ | ✅ | ✅ | ✅ | "fairly high" |
| **Sonnet B** *(told to work fast)* | ✅ | ✅ | ✅ | ✅ | "medium" |
| **Fable** | ✅ | ✅ | ✅ | ✅ | "high / moderately high" |
| **Haiku** | ✅ | ❌ | ❌ | ❌ | **"very high"** |

**5/5 on the read.** Every agent answered both instants correctly by comparison, no
arithmetic, in about a minute. This property is not in doubt any more.

**4/5 on the edit that 0/3 failed in #8** — and the four converged to the *millisecond* on
28 of 29 numbers: +2041 delta, `vo-sentence-06-b` at 26510–31732, `round(3368/0.645) = 5222`,
duration 67257, the 520 ms repeat gap preserved, the straddling header elements stretched
rather than moved.

## Why 4/5 is not evidence the format got safer

Every one of the four named the reason unprompted. **They were handed ADR-0005, which
describes this exact trap by name.** Sonnet B was blunt about it: *"I only avoided it because
I'd just read the ADR that describes the trap before doing the edit — that's not a fair test
of ordinary conditions."* Opus agreed: *"That is a documentation win, not a format win: the
format is safe here only for an agent who read a 181-line ADR first."*

Three further advantages over #8: the file already follows ADR-0005's one-element-per-line
convention; the new duration was given rather than needing a probe; and the sandboxes were
isolated.

So the finding is narrower and still worth having: **documenting a trap and formatting the
file well demonstrably prevents it.** Both are cheap. Neither is a property of the data model.

## The failure, and what it proves

Haiku shipped three defects and rated itself *"very high"* confidence on two of the three
tasks, having "checked by inspection of ranges":

1. **1241 ms of the re-recorded line is silently cut off.** It set `source_end: 3368` but
   left the timeline range at 3981 — the `speed` element's duration is derived from the
   source, and it missed the derivation. It also miscomputed `3368/0.645` as 5220.155
   (it is 5221.7).
2. **An 800 ms hole in the `photo` track and the `caption` track** — `photo-06` ends at
   30603, `photo-07` now starts at 31403. That is 800 ms of background colour with a
   caption missing, in the middle of the video. Exactly the *"three-second black hole"*
   ADR-0005 records two of three agents punching.
3. **Task 3 moved four of the five sentence texts.** `sentence-quiz` still sits at
   `y: 1537` — it is the one furthest from the others in the file.

**None of the three is an overlap.** Every one is a class ADR-0005 already identifies as
invisible to the obvious check: the first needs the source/speed invariant, the second is a
*legal gap* that is only near-error because of what the track renders, and the third no
validator can catch at all — it is an intent question.

This is the strongest available argument for ADR-0005's severity rule (gap severity follows
whether the track renders picture) and for a cross-track coverage report. A plain
overlap-checker reports Haiku's file **clean**.

## Where all five agreed

**A CRUD API would be noise.** Unanimous, and forcefully. Opus: *"They wrap `Edit` in a worse
`Edit` … A CRUD API over a file I can already read is strictly negative value. Do not build
them."* Fable: *"The one-element-per-line convention is a better API than an API."* The
standing write-tool invariant is confirmed from the consumer side, for the second time.

**They wanted `validate` more than `shift`.** All five hand-wrote a checker before trusting
their own work. Fable named the problem exactly: *"the auditor and the defendant are the same
person."* Opus: *"An agent that has to build its own correctness oracle out of a design
document will build a subtly wrong one, and will then be confidently wrong. Ship the oracle."*

**`shift` is wanted but insufficient, in a specific way.** Every agent that discussed it made
the same point: `at` and `delta` are *outputs* of the hard part, not inputs to it.

> *"`shift` cleans up after the decision; it doesn't help make it."* — Opus
>
> *"It would have made the edit I'd already decided on safe to execute; it would not have told
> me an edit was needed at item-07 at all."* — Sonnet B

Opus adds a live bug in the planned spec: **`at` sitting exactly on a boundary is the normal
case, and an off-by-one there is silent.** Guess 30604 instead of 30603 and `shift` obediently
leaves `photo-06.end` behind, punching a two-second hole. The refusal-with-boundaries message
should fire on *ambiguity*, not only on straddling.

## Three new findings

### 1. `speed` has no stated definition, and a self-consistent wrong guess is unfalsifiable

All five reverse-engineered "divisor, not multiplier" from a single data point,
`2568 / 0.645 = 3981`. Opus called this **the scariest thing it did**:

> *"If `speed` is actually a multiplier, my `vo-sentence-06-b` is wrong by 2.4 seconds and my
> validator — which learned the same convention from the same data — would cheerfully agree
> with me. A self-consistent wrong convention passes every check I can write from inside the
> file."*

The rounding rule is equally unstated: `3368/0.645 = 5221.70`. Four agents rounded to 5222;
Haiku's arithmetic gave 5220. Under an integer type whose whole point is exact adjacency, a
1 ms error is a real overlap. ADR-0005 chose integers *precisely* so adjacency would be
decided by the type rather than by everyone remembering to round — and then left unstated the
one place where rounding is unavoidable.

### 2. The stretched straddler's keyframes are underspecified, and the divergence is invisible

`photo-06` is stretched by the insert: its `end` moves, its `start` does not. ADR-0005 says
stretch it. It says nothing about its animation.

| | `photo-06` | zoom ramp | scale at its end |
| --- | --- | --- | --- |
| Opus, Sonnet A, Sonnet B | 17472–32644 | held at 32472 | **1.0800**, then frozen 172 ms |
| Fable | 17472–32644 | moved to 34513 | **1.0712**, still zooming |

**Both are legal. Both pass every check. They render differently.** Fable chose mechanical
shift and flagged it as its own most likely error, correctly noting the format cannot
arbitrate. Opus reasoned the opposite way — *"it got longer without moving, so its ramp must
not move"* — and preserved the `[start, start+15000]` invariant.

Two agents also hit the plain version of this live: Sonnet A's first pass shifted `start`/`end`
correctly and left three images' keyframes stale, caught *"only because I happened to eyeball
the diff."* Opus: **"a naive implementation of the planned `shift` tool would get this wrong."**

### 3. The segment boundary is the most important fact in the file and does not exist in it

30603 is where item 06 ends. It is not a field. It is five tracks coincidentally holding the
same integer, and every agent had to induce it by comparing tracks. Opus:

> *"If `photo-06`, `card-06`, `sentence-06` and `word-06` had ended at four slightly different
> times — which is entirely legal — I'd have had to guess which one was the real segment
> boundary."*

Both Opus and Sonnet B independently asked for the same missing thing: a way to see which
times multiple tracks agree on. Sonnet B also wanted derived relationships marked, so the next
reader can tell a rule from a coincidence.

## Smaller things worth keeping

- **`group` was the most-praised field in the file** — no rendering effect, and the only thing
  that expresses cross-track coupling at all. Opus used it to find item 06's blast radius in
  one grep across seven tracks.
- **`id` on every element** was called underrated: it makes edits addressable and validator
  messages meaningful.
- **Three agents independently flagged `word-08-bridge`** as unexplained — *"I can state that
  it is on screen but not why it exists — the file answers what, not why."* The two-element
  split for item 08's word slot is illegible to a fresh reader.
- **`box` is an unlabelled 4-array.** Two agents noted they had to get `[x,y,w,h]` from the
  fixture README, and that without it the guess was 50/50 with no way to check. Text uses named
  `x`/`y`; rects use positional. Haiku asked for named fields.
- **`duration` is a fourth copy of a fact stated three other ways** (last photo `end`, header
  `end`, loop-tail `end`) — Opus flagged it as the dual-representation defect ADR-0005 rejects,
  sitting in the file unremarked.
- **Comparisons:** better than FCPXML/OTIO (order-derived position makes the read a fold),
  better than JSON2Video (scene-local clocks), better than `.prproj` (not agent-authorable at
  all), **worse than Remotion on writes** — "item 06 got 800 ms longer" is one constant there
  and 29 numbers here. Opus's closest analogue: **Kubernetes manifests** — flat, greppable,
  wonderful to read, with cross-resource invariants no single file expresses. *"K8s is livable
  because `kubectl apply --dry-run=server` exists and is fast. That is exactly the piece this
  format is missing, and it is missing nothing else important."*

## The verdict, in one line

Sonnet B: *"a format that is safe to read carelessly and unsafe to write carelessly."*
Fable: *"a very good format with a hard hat required."*
