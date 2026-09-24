# Round 1 — the question

Sent verbatim to all three jurors. Each was instructed to answer and nothing
else: no tools, no file reads, no recommendations about process. The context
block is reproduced here in full because the jurors had no repository access —
everything they reasoned from is below.

> **Note added on resolution.** The exemplar table below carries ADR-0037's
> `4 of 52` figure and its `8000/16800/25700/35300` values as fact. They do not
> reproduce against the committed fixture. See ADR-0086's erratum. No ballot in
> this directory weighed the real census, because none of them saw it.

---

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
| time | two timestamps meant to stay in sync; a fade meant to land exactly on a cut | `shift` prints a preamble covering only the case where the shift lands exactly on the shared instant |
| time | hand-typed derived timestamps carrying no signature | **4 of 52** absolute timestamps in the one real fixture are `8000/16800/25700/35300` ms, each equal to `end − 300` for its element, with nothing recording that relationship |
| audio | a karaoke `highlight` window's link to the audio it was calibrated against | **11 of 22** text elements overlap 2+ audio sources, so inferring the partner from timeline overlap is provably ambiguous |

## Constraints already decided, carried in

- **ADR-0036 rejected a live, persisted time-anchor reference 3/3**: a reference the *renderer* reads breaks read-by-reading, degrades to a stale literal on the first shift, and gives one element's rendered geometry a silent second author. Not reopened.
- **ADR-0037 killed an emit-only "derive" helper tool** on the discriminator *"does the tool have an input the agent lacks?"* — `end − 300` is computable from data already in the file, and the helper's output is a dead literal the instant it is pasted, so the gap survives the tool. It re-diagnosed the problem as **provenance, not computation**.
- **ADR-0036 also named the surviving candidate and its weakness**: a **renderer-ignored declaration**, structurally ADR-0015's `fit` transplanted. It called this a **half-binding** — it can record a start instant but not whether the intended relationship is same-rate or same-endpoint.
- **ADR-0063** scoped `compare`'s drift predicate to **exact equality only**, deliberately excluding the fixed-offset case ("keyframe B is always 300ms after A"), because a preserved offset is *a hypothesis about intent inferred from arithmetic*: on a 60-element file every pair of instants that both happened not to move trivially "preserves its offset", so the candidate population explodes and the signal drowns.
- **ADR-0012 retired `box:"<id>"`**, which removed the format's **last element-to-element reference**. The format currently has **zero** such references.

## The `fit` precedent (ADR-0015) — the structural model on the table

`fit` is a field on every element carrying a raster source. Its closed value set is `cover` / `contain` / `literal`. **No renderer reads it** — the declared `width`/`height` rect is authoritative at render. `fit` is *a claim about how the author computed those two integers*. Its only consumer is `validate`, which re-derives the rule's value and compares. Deviation is an **`error`** with an **advise-class repair** (it states the correct integer), because the rule determines exactly one legal integer. `literal` is the escape value meaning *no rule*.

## `validate`'s finding vocabulary

Five **classes**. Three are severities, named for what the reader does: `error` (render refused or guaranteed wrong), `review` (legal, renders, and you must look at a frame to know if it was meant), `note` (a fact you will not act on today). Two are not severities: `UNCHECKED`, `LAYOUT`.

Every `error`-class finding carries a **repair** in exactly one of two forms: **advise-class** states a value, when the correct fix is fully determined by the document, the media on disk and published rendering semantics; **refuse-class** states `"none"`, when the fix depends on knowing what the author meant — and that refusal is a guarantee no flag or write tool may lift.

## Explicit non-goals for any answer

- Not a live reference (rejected 3/3, not reopened).
- Not an escape hatch (closed schema holds).
- Not evaluation (the file stays inert data).
- Not four bespoke fields added one per exemplar without deciding the first question.

# The Question

Answer all four parts.

**Q1 — One field, one pattern, or four unrelated gaps?** Three candidates:
(a) **One generic field** — a single `derived`-style construct usable on a timestamp, a transform property and a highlight window alike, expressing "value X was derived from value Y by rule R" where R ranges over arithmetic.
(b) **One pattern, several named fields** — the gap is one gap and the *rule for filling it* is one rule (a renderer-ignored declaration consumed by `validate`, structurally `fit`), but each axis instantiates it as its own named field with its own **closed vocabulary of derivation rules**, the way `fit` is a finite enum and not an arithmetic expression.
(c) **Four unrelated gaps**, each graduating with its own evidence bar and no shared constraint.

**Q2 — May a declaration name another element's `id`?** The format currently has zero element-to-element references (ADR-0012 removed the last one). ADR-0036 rejected a *live* reference the renderer reads. Is a **recorded, renderer-ignored** id reference that only `validate` resolves a different object that the rejection does not reach — or does reintroducing any id reference bring back rename fragility, dangling references, and a second place a reader must look? Note the audio exemplar measured that the partner **cannot** be inferred (11 of 22 ambiguous), so on that axis naming the partner is the entire content of the declaration.

**Q3 — What does a violated declaration cost?** `review` with a refuse-class repair (`"none"`, on the grounds that when a declared relationship breaks you must look to know which of the two numbers is stale), or `error` with an advise-class repair (on the grounds that a declaration is **directional** — the author has already said which value is the source and which is derived, so the repair is determined: rewrite the derived value)? Does the answer differ for a declaration that cannot name a direction (a symmetric "these two stay equal")?

**Q4 — Which axes instantiate now, and which graduate as separate tickets?** Evidence is uneven: the time-derivation axis is measured 4-of-52, the audio-link axis is measured 11-of-22, the timestamp-coincidence axis is partly covered by `shift`'s preamble, and the motion axis has no measurement and no proposed shape. Should the deciding ADR ship the pattern **plus one or more concrete fields** for the best-measured axes, or ship **the pattern alone** and graduate every axis as its own ticket against it?
