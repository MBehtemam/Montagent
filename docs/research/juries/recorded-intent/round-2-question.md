# Context

Montagent is an agent-first declarative video editor. A single JSON project file in git is the source of truth. Standing principles (settled, not reopenable):

- **File-as-truth.** The project file is the source of truth.
- **Inert data, no evaluation.** The file must be fully understandable by *reading* it. The moment it becomes code, the agent has to execute it in its head to know what is on screen at 6s — the original disease.
- **Expressions must stay out** (explicit Out-of-scope ruling): a property computed by evaluating code ends file-as-truth.
- **Closed schema, no escape hatch** (ADR-0017): any field is named and published; no free-form blobs.
- The tool surface is nine verbs, including `validate` (stateless, one document, no I/O beyond probing sources), `compare` (takes a before/after pair), `render`, `shift`, `measure`.

## The gap

**The format has no way to record what an author meant.** Every value is a literal, deliberately. A relationship between two values exists only in the author's head. An edit that breaks it is indistinguishable from an edit that never had it. `validate` cannot flag such a break without asserting intent no document carries. `compare` needs a before/after, and for something *born* broken there is no earlier correct state to diff against.

## The four exemplars (each found independently, on a different axis)

| axis | exemplar | evidence |
| --- | --- | --- |
| motion | a coupling between two elements that was never correct from the moment of authoring | none — no census, no measurement, no proposed shape |
| time (coincidence) | two timestamps meant to stay in sync; a fade meant to land exactly on a cut | `shift` prints a preamble covering only the case where the shift lands exactly on the shared instant |
| time (derivation) | hand-typed derived timestamps carrying no signature | **4 of 52** absolute timestamps in the one real fixture are `8000/16800/25700/35300` ms, each equal to `end - 300` for its element, with nothing recording that relationship |
| audio | a karaoke `highlight` window's link to the audio it was calibrated against | **11 of 22** text elements overlap 2+ audio sources, so inferring the partner from timeline overlap is provably ambiguous |

## Constraints already decided, carried in

- **ADR-0036 rejected a live, persisted time-anchor reference 3/3**: a reference the *renderer* reads breaks read-by-reading, degrades to a stale literal on the first shift, and gives one element's rendered geometry a silent second author. Not reopened.
- **ADR-0037 killed an emit-only "derive" helper tool** on the discriminator *"does the tool have an input the agent lacks?"* — `end - 300` is computable from data already in the file, and the helper's output is a dead literal the instant it is pasted, so the gap survives the tool. It re-diagnosed the problem as **provenance, not computation**.
- **ADR-0063** scoped `compare`'s drift predicate to **exact equality only**, deliberately excluding the fixed-offset case ("keyframe B is always 300ms after A"), because a preserved offset is *a hypothesis about intent inferred from arithmetic*: on a 60-element file every pair of instants that both happened not to move trivially "preserves its offset", so the candidate population explodes and the signal drowns.
- **ADR-0012 retired `box:"<id>"`**, which removed the format's **last element-to-element reference**. The format currently has **zero** such references.
- **ADR-0041**: canonical key order is *schema order*, checked by `validate`, split by `fmt`.

## The `fit` precedent (ADR-0015) — the structural model

`fit` is a field on every element carrying a raster source. Its closed value set is `cover` / `contain` / `literal`. **No renderer reads it** — the declared `width`/`height` rect is authoritative at render. `fit` is *a claim about how the author computed those two integers*. Its only consumer is `validate`, which re-derives the rule's value and compares. Deviation is an **`error`** with an **advise-class repair** (it states the correct integer), because the rule determines exactly one legal integer. `literal` is the escape value meaning *no rule*. `fit` is **required** on every raster element, on the reasoning that *"an omitted field is indistinguishable from a decision not to check"* — and it was free, because 8 of 8 elements already carried it.

## `validate`'s finding vocabulary

Five **classes**. Three are severities, named for what the reader does: `error` (render refused or guaranteed wrong), `review` (legal, renders, and **you must look at a frame** to know if it was meant), `note` (a fact you will not act on today). Two are not severities: `UNCHECKED`, `LAYOUT`.

Every `error`-class finding carries a **repair** in exactly one of two forms: **advise-class** states a value, when the correct fix is fully determined by the document, the media on disk and published rendering semantics; **refuse-class** states `"none"`, when the fix depends on knowing what the author meant — and that refusal is a guarantee no flag or write tool may lift.

## Explicit non-goals

- Not a live reference (rejected 3/3, not reopened).
- Not an escape hatch (closed schema holds).
- Not evaluation (the file stays inert data).
- Not four bespoke fields added one per exemplar without deciding the first question.

# The Question

Eight parts. **Q1-Q4 already carry provisional answers, adopted by the decision-maker after an earlier panel.** They are restated here with those answers. You are explicitly invited to **confirm or overturn** each one now that the shape questions (Q5-Q8) sit alongside them — a later question may expose that an earlier answer was wrong. Do not defer to the provisional answer.

**Q1 - One field, one pattern, or four unrelated gaps?**
(a) **One generic field** expressing "value X was derived from value Y by rule R" where R ranges over arithmetic.
(b) **One pattern, several named fields** - the gap is one gap and the fix rule is one rule (a renderer-ignored declaration consumed by `validate`, structurally `fit`), but each axis instantiates it as its own named field with its own **closed vocabulary of derivation rules**.
(c) **Four unrelated gaps**, each graduating with its own evidence bar and no shared constraint.
*Provisional answer: (b).*

**Q2 - May a declaration name another element's `id`?** The format has zero element-to-element references. ADR-0036 rejected a *live* reference the renderer reads. Is a **recorded, renderer-ignored** id reference that only `validate` resolves a different object the rejection does not reach? And what class is a **dangling** id (the named element does not exist)?
*Provisional answer: yes, permitted - inert, renderer-ignored, `validate`-only - but the permission is **per-axis and earned**, granted only where inference has been measured to fail. A dangling id is an **`error` with refuse-class repair**, on the reasoning that `review` is defined as "you must look at a frame to know if it was meant" and a frame cannot tell you which element id was intended.*

**Q3 - What does a violated declaration cost?**
*Provisional answer: **directional** declarations (the document names which value is source and which is derived) are `error` + advise-class repair, by the same logic that makes `fit` an error; **symmetric** declarations ("these two stay equal", naming no source) are `review` + refuse-class `"none"`; and a **partner-only** declaration (the audio link) has no arithmetic to re-derive, so it cannot be violated at all - its only findings are "partner missing" or nothing.*

**Q4 - Which axes instantiate now?**
*Provisional answer: ship the pattern **plus exactly one field**, the time-derivation axis (measured 4-of-52, directional, needs no id reference); graduate audio, coincidence and motion as separate tickets. Audio is better-measured (11-of-22) but it is the axis that spends the Q2 id permission, and reversing ADR-0012's zero-references invariant deserves its own record rather than a rider on a pattern ADR. The ADR would have to state plainly that holding the better-measured axis back is an architectural call, not an evidential one.*

**Q5 - Is a derivation declaration required with an escape value, or optional with absence meaning no claim?** `fit`'s precedent is **required**, on the reasoning that *"an omitted field is indistinguishable from a decision not to check"* - an argument that transplants verbatim, since an absent declaration and a deliberately-unrelated literal look identical. But the cost does not transplant: `fit` was free (8 of 8 elements already carried it), whereas here **52 timestamps would carry a declaration so that 4 can be checked**, a 13x authoring tax on a format whose authors are agents that copy the nearest example.
*Provisional answer: optional, absence means no claim - accepting as a stated limit that the mechanism is structurally blind to undeclared relationships.*

**Q6 - May the rule carry an integer parameter?** A fully closed enum (`same-as-end`, `same-as-start`) cannot express the fixture's `end - 300`. Admitting `{rule: "before-end", ms: 300}` expresses it exactly, and an integer argument is *data*, not evaluation - `fit`'s `cover` likewise takes an implicit input (the aperture) without becoming an expression. The counter: a named rule plus a free numeric argument is one generalisation away from the arithmetic language option (a) was rejected for.
*Provisional answer: yes - a closed enum of named rules plus **at most one** integer argument, with the bright line being that the rule set is finite and published, there is exactly one argument, and no rule composes with another.*

**Q7 - Where does the declaration sit?** Beside the element (one declaration per element), or beside the annotated value (so an individual keyframe record can carry its own)? Key order itself follows ADR-0041 automatically from schema placement.
*Provisional answer: beside the annotated value, because all four fixture cases annotate a specific timestamp and an element-level declaration would need to name **which** of its timestamps it describes - reintroducing an intra-element pointer to avoid an inter-element one.*

**Q8 - Does `compare` gain anything, or is `validate` the whole consumer?** ADR-0063 refused fixed-offset drift in `compare` because a preserved offset is a hypothesis inferred from arithmetic and the candidate population explodes. A **declared** offset is not inferred, and the population is exactly the declarations - so that reason stops applying.
*Provisional answer: `validate` alone; `compare` gains nothing, because both values sit in one document and a declared relationship is checkable statelessly. But ADR-0063's refusal should be recorded as **narrowed rather than reversed**.*

# Required output format

Reply with exactly eight blocks and nothing else. For Q1-Q4, state clearly whether you CONFIRM or OVERTURN the provisional answer.
