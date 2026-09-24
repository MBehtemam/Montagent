# Inline values vs. declared-and-referenced values, for an LLM

Gathered while resolving [Name the core domain model](https://github.com/MBehtemam/Montagent/issues/4),
to decide whether an element names its file inline or refers to a declared asset
table. Feeds [ADR 0002](../adr/0002-inline-source-no-asset-table.md).

**Question.** When an LLM must read, author and edit a structured document, is it
more reliable with values written inline at the point of use, or declared once and
referenced by id?

```
A (inline)       {"type":"image","source":"images/05.png","start":3.0}
B (indirection)  "assets": {"img05":"images/05.png"}
                 {"type":"image","source":"@img05","start":3.0}
```

## Verdict

**Inline is better supported — at moderate confidence (~70% for reading and
editing, ~60% for generation).**

**Read the gaps section before relying on this.** No study tests this question
directly. Everything below is inference from adjacent results.

## For inline

**Anthropic, *Writing effective tools for AI agents*** — the most on-point
guidance found:

> "tool implementations should … eschew low-level technical identifiers (for
> example: `uuid`, `256px_image_url`, `mime_type`)"
> "Agents also tend to grapple with natural language names, terms, or identifiers
> significantly more successfully than they do with cryptic identifiers"
> "merely resolving arbitrary alphanumeric UUIDs to more semantically meaningful
> and interpretable language … significantly improves Claude's precision in
> retrieval tasks by reducing hallucinations"

([source](https://www.anthropic.com/engineering/writing-tools-for-agents)) — but
published with no numbers, dataset or methodology. Strongest citation on this
side, and unquantified.

**RULER's Variable Tracking task** ([arXiv 2404.06654](https://arxiv.org/abs/2404.06654))
is literally this problem: chains of `X2 = X1` bindings the model must follow.
Performance degrades with chain and hop count; models "make errors due to
incorrectly returning empty strings or variables from other chains." *Caveat:* VT
uses multi-hop chains with semantically empty names. A single hop with a
meaningful key is the easiest version of what it shows models are bad at, so the
real penalty here is likely far smaller than the headline.

**Parameter hallucination.** Agents assert invented ids, names and URLs that pass
schema validation but do not exist — a distinct, common failure class
([MIRAGE-Bench, arXiv 2507.21017](https://arxiv.org/abs/2507.21017)). A dangling
reference is exactly this, and JSON Schema cannot catch it; a wrong file path
fails loudly at render.

**Identifier meaningfulness measurably matters.** Stripping meaningful identifiers
drops Pass@1 from 85.7→76.1 and 85.4→71.2 on two benchmarks
([arXiv 2510.03178](https://arxiv.org/abs/2510.03178)). An opaque alias is
strictly worse than a descriptive one, and a file path at the point of use carries
more signal than either.

**Context rot** ([Chroma](https://www.trychroma.com/research/context-rot)) —
lengthening the distance between question and answer degrades performance, and
distractors hurt non-uniformly even at one. A table of similar-looking aliases is
a distractor farm.

**Compositionality gap** ([arXiv 2210.03350](https://arxiv.org/abs/2210.03350)) —
single-hop accuracy improves faster with scale than multi-hop.

*Do not cite the Two-Hop Curse here.* Its finding concerns facts learned in
separate documents; it explicitly notes models can compose when facts appear
together in the prompt ([arXiv 2411.16353](https://arxiv.org/abs/2411.16353)).

## Against inline

**Incomplete refactoring is a dominant agent failure mode.** Models "frequently
struggle to identify and update all necessary occurrences"
([RefactorBench, arXiv 2503.07832](https://arxiv.org/abs/2503.07832)). If a value
appears at a dozen sites, indirection needs one edit and inline needs twelve
correct ones. **For the editing task specifically this is stronger evidence than
anything on the inline side.**

**Repeated identical strings are not free.** Models degrade at reproducing long
repeated sequences — under-generating, stopping early, inventing tokens
([Chroma](https://www.trychroma.com/research/context-rot)).

**Length is a cost**, so deduplication has some value. Magnitude: roughly
(occurrences − 1) × path length. Negligible for ~30 files used once or twice;
material for a file used fifty times.

**The table as a table-of-contents** that helps generation is plausible and
structurally similar to why [llms.txt](https://llmstxt.org/) exists — but no
direct evidence was found. Untested intuition.

## The reconciliation

Anthropic's *context engineering* guidance recommends the opposite pattern —
lightweight identifiers, loaded just in time
([source](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)).
The two are reconciled by scope:

> **References are for data not in context** (avoid paying for it).
> **Inline is for data already in context** (indirection buys nothing, costs a hop).

An asset table sits in the document the model is already reading, so the
tool-writing guidance governs.

## What could not be found

1. **Any direct experiment on this question** — inline vs. aliased values in a
   structured document, on reading, generation or editing. The verdict is an
   inference, not a finding.
2. **Any benchmark measuring referential integrity in LLM-generated documents.**
   JSONSchemaBench ([arXiv 2501.10868](https://arxiv.org/abs/2501.10868)) and
   StructEval ([arXiv 2505.20139](https://arxiv.org/abs/2505.20139)) both measure
   shape, not whether an id declared in one place is used consistently in another.
   A genuine hole in the benchmark landscape.
3. **Any measurement of single-hop, meaningful-key alias resolution** — the actual
   case here. The penalty could be 0.5% or 15%; nobody has isolated it.
4. **Any controlled study of YAML anchors/aliases vs. expanded YAML.** Opinion
   blogs only.
5. **Evidence for the table-as-schema-hint hypothesis.**
6. **Provenance for Anthropic's UUID claim** — no numbers or methodology.

## How to close the gap cheaply

Build one fixture project in both encodings and run a matched eval: N factual
questions ("what image is on screen at t=3.2?"), N generation prompts scored on
validity and referential integrity, N targeted edits scored on completeness. That
would produce better evidence than anything cited here.
