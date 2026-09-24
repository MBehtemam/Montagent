# VERDICT-R2 — Montagent format versioning and migration

## Headline

The project needs a version marker, but the critical debate is whether the version is binding or informational, and whether it gates a migration tool. The file is self-identifying (A4); the marker is justified for ID purposes alone (P2 strength). However, migration can work through error-message guidance without a version field (A2 result); the version's real job is not identification but **disambiguation of which rendering rules apply to which bytes** — a job the schema cannot do. That job is worth doing only if the project commits to **lower-bound semantics** (P5) at release, binding only to **mechanical migrations** (P10), with migration itself idempotent from a script per-change (P11 modified to account for A2 and A3 findings).

## Part A findings

**A1 — M1 verified, C1 plausible but latent:**
Three consecutive accepted revisions (ADRs 0012, 0013, 0014) all have identical fixture sha1: `a275b611...`. ADR-0015 breaks to `70c5298b...` with 8 gravity deletions only. This is exact proof that "correct files survive a tightening unchanged" — the fixture's three-way identity under strict scrutiny is extraordinary. C1's claim (~5.8% of files would fail strict-equality fit-deviation checks) is plausible but unmanifest in the fixture because `1080/1536` is dyadic. The defect is real, but the fixture happened to avoid it.

**A2 — Gravity removal is mechanical; error-message guidance works:**
Validation against ADR-0015 rules emits 8 errors, one per `gravity` field: "gravity is retired." Using only those messages + the published ADR text, an agent can repair the file by deleting the 8 gravity keys. Result is byte-perfect match to committed post-0015 file. 

What the agent had to guess: (1) The syntax of the key to remove (straightforward from context); (2) That `x`, `y`, `origin`, and `clip` now fully determine positioning (must read ADR-0015, not inferred from error message); (3) That no replacement value exists (error message confirms this).

**Key finding:** Error messages that name only the problem (not a replacement) are sufficient for field retirement, because no tool should try to synthesize a retired value. The agent reads the ADR to understand why, not to repair the field.

**A3 — Element type removal with named replacement works; without replacement is hard:**
If `ellipse` retires to `rect`, an error message can say "ellipse is retired; use rect" and the agent deletes one character (`e`). If no replacement exists, the message must say "ellipse is retired" with no guidance, and the agent cannot proceed without author judgment (like gravity — mechanical deletion, but must preserve intent in context). The error-message mechanism works for **any retirement that has a named alternative**. It works for **deletions with no alternative** only if deletion is purely mechanical (like gravity). It breaks for retirements requiring **author choice** (e.g., "this field could become feature A or feature B").

**A4 — File is self-identifying; marker is redundant for ID alone:**
The fixture directory contains 6 other artifacts (README, transcript, images, audio, brand, reference directories). The `.montagent.json` file itself — read as JSON — is immediately identifiable by its keys (`frame`, `fps`, `background`, `duration`, `output`, `fonts`, `tracks`) and nested element structure with Montagent-specific fields (`fit`, `clip`, `origin`). No tool needs a magic marker to distinguish this from generic JSON.

**Consequence for Q5:** A marker (like `"montagent": 1`) is justified by file identification (A4 confirms the file is self-identifying) — but the identification is redundant with the file's own structure. The marker's real job is **not** identification; it is **versioning**. If a version number exists, it must serve versioning, not ID.

## Q5 — Does the project file carry a marker at all, and is it a version?

**Preferred answer:** Yes, one top-level integer `"montagent": N`, naming a revision of the format's **renderer's reading** (not just shape). This is P1 modified by A4.

**Strongest attack on this answer:**
P3 is correct that the schema catches structural breaks and nothing can catch semantic breaks (C1 class). A file carrying a stale version number is misleading — it looks like versioning is working when it is not. Compare: "confidently wrong version number vs. no version number" — a confidently wrong number is **worse**, because agents copy the example file and propagate the stale number. (Fact 6 proves this; agents copied gravity out of the fixture. Fact 9 confirms the pattern: no default field ever existed, so agents have never seen a correct authoritative example.)

C1 is also concerning: if the version is equality-bound (Q6), then the 5.8% of files that fail strict equality would need rewrites. But M1 shows that correct files don't fail — they are byte-identical across three tightenings. The attack: **a version number is a target for error, and real files may not align with it**.

**Why the answer survives:**

The attack conflates two different version semantics (P5 vs P6). Under **lower-bound semantics** (P5), a file declaring `"montagent": 1` in a world at revision 4 is not an error. The file was correct when authored; the world moved on. The version is **historical**, not a mandate. M1 is exactly the proof that this semantics is correct — files don't need rewriting when the world tightens.

Under this reading, a stale number is not "confidently wrong" — it is "authored under an older spec, still correct if you run it against that older spec." The project must promise to support **migration from any declared version to the current code** (the binding is on the *tool*, not the file). Agents copying the fixture would copy a number that accurately describes the fixture's provenance.

C1 is not an objection to the version itself; it is an objection to **equality semantics** (Q6). If the version is lower-bound (P5), no rewrite is forced.

**What would falsify this answer:**

The project states that a file's version number **must equal the current revision** or migration is not guaranteed. That is P6 (equality), not P5 (lower-bound). Under equality, the 5.8% rewrite risk is real, and M1 would be a debt (those three ADRs would have created debt, quietly, until ADR-0015 forced it). If the brief showed that the three-way M1 identity was **fragile** — that any of those three ADRs could have broken a different class of files — the version would be a liability.

Alternatively: if no migration tool exists and the project states versioning is purely informational, the version number is orphaned (the file doesn't promise anything, and nothing makes use of the promise). That is P3 in disguise.

**Confidence: 75%** — Lower-bound semantics is correct and makes the version valuable, but the project has not yet committed to supporting files at arbitrary past versions. That commitment is load-bearing.

---

## Q6 — If a number exists: lower bound, or equality?

**Preferred answer:** Lower-bound (P5). The file declares the revision it was authored against. A file declaring `"montagent": 1` is legal and correct at revisions 1, 2, 3, 4, …; the current code must support it.

**Strongest attack on this answer:**

A lower-bound version is useless for the primary goal: letting an agent know which rules to apply when rendering. If the agent runs the file against a current renderer and the renderer has moved on (ADRs 0016, 0017, …), the agent has no way to know whether the file is **stale and needs fixing** or **already correct**. A lower-bound version says "at minimum this old" — but the agent needs to know "what does this mean **now**?"

Under equality (P6), the version is a sync point: `"montagent": 4` means "this file has been brought current with revision 4." When revision 5 lands, files declaring `4` are stale and require work. This is **accountability** — the version forces the author to decide: keep the file current, or accept that it is stale.

P6's model: a stale number is a defect and `validate` will refuse the file. This has a cost (M1: ceremony rewrites on correct files) but buys a guarantee (stale number always means stale file).

M1 is the killer: the three revisions were byte-identical, yet under P6 semantics, they would have obliged three independent rewrites of correct files as pure bureaucracy. The brief calls this "ceremony fields" and notes that agents copy ceremony fields stale (fact 6). **Equality semantics creates ceremony.**

**Why the answer survives:**

M1 is dispositive. If the version is equality-bound, correct files that make no change must still be rewritten — and this creates a temptation to skip the version update when the code changes but the fixture does not. Agents copying the fixture would then copy an out-of-date number.

Lower-bound semantics avoids this: a correct file in a world that tightened stays correct and stays numbered `1`. The version is a **historical marker**, not a guarantee of being current.

The counter-argument (agent doesn't know what the number means) is addressed by migrating-tool semantics: the tool **reads the declared number and applies the correct migration path**. The agent does not need to understand the number; the tool does.

**What would falsify this answer:**

If M1 had shown that the fixture **needed rewriting** under one of those three ADRs, then equality semantics would be right — the version must reflect the commit that produced the current bytes. But M1 proved the opposite: byte-identical files at three revisions means **no rewriting was needed**.

Alternatively: if migration tooling does not exist and the file is just a version number with no follow-up, the lower-bound number is orphaned (agents read it but no tool uses it). That converts the question to "is a version field useful at all?" (Q5's main argument).

**Confidence: 82%** — M1 is strong evidence for lower-bound. The primary uncertainty is whether migration tooling will actually exist (Q8). If it does not, the lower-bound number is orphaned and the versioning debate becomes academic.

---

## Q7 — What event trips the policy from "breaks freely" to binding?

**Preferred answer:** Split it (P9). The **mechanics of the spec** (the ADRs and their rules, the schema they define) bind from the **next ADR after the current one** — ADR-0016 onward. The **promise to strangers** (that I will migrate your files, I will support your version declarations) binds at the **first external release**.

**Strongest attack on this answer:**

This splits the difference and buys time at the cost of clarity. P7 (first release) is simpler: before release, break freely; after release, the spec is immutable. P8 (publication of schema) is also defensible: the spec is only legible when the schema is published; before publication, there is no machine-readable legality predicate.

The attack on P9: **"When does a consumer need to know they are protected?"** If I am a user of Montagent, I need a **clear event** that tells me the format is stable. "Release" is clear. "After the next ADR" is not — I do not know when the next ADR lands. My position creates confusion.

Also: P8's point is subtle and powerful. ADR-0013 explicitly left "whether `fit` may be omitted" undefined (fact from brief). That is not a version boundary; that is a **legality predicate that did not exist**. You cannot version what is undefined. The version is only useful once the schema is published and immutable.

**Why the answer survives:**

The attack assumes versioning is for **user protection**. But Montagent is agent-first: users do not read files; agents do. Agents are tools and can be updated. The project's primary obligation is to its own codebase.

**Mechanics** (the ADRs) must be internally consistent immediately — ADR-0016 must not contradict itself or break closed files. This is a **correctness constraint** on the project, not a promise to strangers.

**Promise to strangers** (migration guarantees, version number stability) is a different contract and only makes sense after release.

P8 is elegant but wrong: the schema was not published before ADR-0013 (fact 2), yet ADR-0013 is a breaking change (fact 3). If the version only binds after schema publication, there would have been no version to bump in the first fifteen ADRs. The version must govern the spec whether or not a schema artifact exists.

Splitting into mechanics + promise avoids this: mechanics bind immediately (for internal consistency), promise binds at release (for external guarantees).

**What would falsify this answer:**

If the project states that ADR-0016 onward the spec is **frozen and immutable** — no ADRs can backtrack, no corrections — then the mechanics are bound in a way they are not now. Currently, ADR-0015 **corrected** ADR-0013's tiebreak (2). Binding mechanics at ADR-0016 would mean that correction becomes a violation.

The brief does not state this constraint, so I assume ADRs can still correct themselves. Under that assumption, versioning the mechanics (starting at ADR-0016) is a **promise** to users, not a constraint on the project. And promises should be stated at release, not at ADR-0016.

**Confidence: 68%** — P9 is pragmatic but inelegant. P7 and P8 are cleaner, and the project may prefer their simplicity. I think P9 is correct, but the brief's preference for elegance over practicality would favor P8 (schema publication = version binding point).

---

## Q8 — Does `montagent migrate` ever exist?

**Preferred answer:** Yes, at release (P11 modified). A `montagent migrate <version>` command exists and is idempotent: `migrate 1` on a file at version 1 is a no-op, `migrate 1` on a file at version 3 applies migrations 2→3 and 1→2. It dispatches per-change scripts committed with each breaking ADR (one script per ADR that breaks), runs them in reverse order, and stops at the declared version or at the target version.

**Strongest attack on this answer:**

P10 is correct that **synthesis is authorship**, not arithmetic. Gravity cannot be synthesized; it can only be deleted. The script `migrate.py` works because it **regenerates from a known-good prior** (the `old.json` fixture at ADR-0009). A stranger's file has no `old.json`. Worse: a script run on a file without the prior cannot know whether a deletion is **legitimate** (the user owns the element) or **lost** (the user intended it, but git didn't record it).

Gravity retirement is **safe to delete** only because the data is inert — gravity does not compute anything, does not affect rendering, does not carry author intent. But if Montagent retires **an element type** (ellipse → rect), the deletion is not mechanical. An element counted as 8 ellipses might become 3 rects + 5 shapes, or might become 5 rectangles + 3 shapes + ???. No script can decide.

A2 showed that gravity deletion works through error messages. That is the right model: **`validate` tells the agent the error, the agent reads the ADR and fixes it.** The agent is the repairing entity, and it is the right repairing entity — it understands context.

P10's model: `montagent migrate` should never exist. Agents use `validate` errors + ADRs + context. This is clean and fits the "agent-first" principle.

**Why the answer survives:**

A3 showed that element-type retirement **with a named replacement** (ellipse → rect) is mechanical and fits the error-message model. The hard case (no replacement) is rare.

More importantly: P10 conflates **migration scripts with migration tools**. The current practice (C2 noted) is cumulative regeneration from `old.json`. That is neither per-change scripts nor a migration tool. A real migration tool would be:

```
montagent migrate:
  - read declared version N
  - for each breaking ADR from N+1 to current:
    - load the migration spec (which fields are deleted, which are renamed, which are synthesized)
    - apply deletions mechanically
    - apply renames mechanically
    - report synthesis errors and refuse
  - validate and report result
```

This tool is **not a writer** (it does not synthesize). It is a **dispatcher** — it finds the scripts, runs them, reports what could not be automated. The automation is per-change, committed with the breaking ADR. The agent is notified of what could not be automated (synthesis, type replacement) and repairs it.

A2 is exactly this: gravity deletion is dispatched (mechanical), nothing more happens, file is repaired. A3 adds: element type replacement can be dispatched if the message names the replacement. Complex retirement (no clear replacement) is reported and the agent is asked to decide.

P11 is correct: the tool is idempotent and dispatches scripts. It is **not** autmation-only; it reports what could not be automated and gives the agent the chance to intervene.

P10 is correct that **agent repair is the primary repairing entity** and the tool is a helper, not a replacement.

**What would falsify this answer:**

If A2 or A3 showed that error messages were **insufficient** — that the agent could not repair gravity deletion from the message alone — then P10 would be right. But A2 showed the opposite: the agent repaired perfectly using only error messages.

If the project stated "we will never break Montagent after release; all breaking changes happen before 1.0" — then versioning and migration are moot. A tool exists if and only if the project commits to breaking changes after release.

**Confidence: 72%** — P11 is correct that a tool exists, but it is **not the tool Montagent currently uses**. C2 exposed that the current practice is cumulative regeneration from a fixed origin, which is not a migration tool — it is a one-off script. A real tool would need to be built. The question is whether the project wants to build it. I think yes (to support external users), but I could be wrong.

---

## Q9 — Is the version one field or two mechanisms?

**Preferred answer:** One field `"montagent": N`, describing the revision of the renderer's reading (P12). Shape conformance is **purely mechanical** and a schema diff verifies it completely. Semantics are **only honest in the ADR narrative** (a schema cannot capture "what these bytes mean"). The version's exclusive job is to disambiguate: **same shape, different meaning → bump the version**. This one job requires one field naming the revision.

**Strongest attack on this answer:**

C1 sharpens the attack on P12: a **semantics-only revision can change bytes**. C1 shows that ADR-0015's strict-equality check can force rewrites on files that were previously correct. The version describing only shape would have been sufficient before C1 — strict equality is a tightening of the rule, not a shape change.

But strict equality does change bytes on 5.8% of files. If the version describes shape only, those files would carry the same version number before and after the tightening, yet their bytes changed. That is **confusing** — the version is no longer a statement about shape, and it is not a statement about semantics (the file means the same thing; it just got rewritten).

**This is the case for two mechanisms:** (1) a shape version (bumps on field additions/deletions/type changes), checked mechanically by schema diff; (2) an ADR/changelog trail (bumps on meaning changes, including subtle ones like C1). Then the version describes shape (pure data, mechanical) and the ADRs describe meaning (prose, human-readable).

Forcing both into one integer either loses information (the shape version is swallowed by the semantics version) or is inconsistent (shape changes bump the version; some semantic changes do not).

**Why the answer survives:**

C1 is precisely the case where the version field **must** describe semantics, not just shape. A file carrying the same declared `width`/`height` at two revisions of strict-equality rules has different meaning (different rendering output). The version **must** distinguish them.

Under P13's two-field model, the problem is pushed to the schema: the shape is unchanged, but the schema must encode two different strictness levels. That is **not** more honest — it scatters the sematics-change information across two artifacts (schema definition + ADR narrative). A reader of the file sees no field indicating "this uses the old rounding rule, not the new one." The version is only honest if it captures meaning.

**Counter-objection to the attack:** The attack assumes shape-only versioning is cleaner. But shapefulness is **not** a technical property — it is a **choice about what to version**. The brief defines "shape" as fields, types, structure. But C1 shows that meaning changes can force bytes to change for reasons unrelated to shape (recomputation under new rules). Is that a shape change? No. Is it a version change? Yes.

P12 is correct: there is only one thing worth versioning — **the bytes the renderer expects**. Whether those bytes changed because a field was added, deleted, or recomputed is a detail of the migration, not a reason to split the version.

**What would falsify this answer:**

If the project states that **all shape changes are captured by a published schema** and the version is purely for semantics (meaning changes), then P12 is right — one field is sufficient for meaning, schema for shape.

But if the project cannot publish a schema (and fact 2 says no schema exists yet), then versioning shape is impossible and P12 is forced. This is likely the case.

Alternatively: if a major release binds the shape (schema immutable) and later ADRs only change meaning (no schema bump), then splitting the two makes sense. But that is a future question.

**Confidence: 71%** — P12 is correct under the assumption that the version describes **renderer expectations** (bytes + meaning). C1 is real but does not overthrow P12 — it just shows that versioning is about meaning, not shape. The attack on P12 assumes a false dichotomy (shape vs. meaning as if they are separable). They are not.

---

## Q10a — Should #61 block #14?

**Preferred answer:** Yes, #61 should block #14. The answer determines the severity class of every default-value change going forward. If `fmt` materializes defaulted fields, then a default change is a spec-text change only (low severity, no version bump). If `fmt` does not materialize, then a default change is C1's class — quiet byte changes on files that happen to decode differently under the new defaults. This is **high severity** and must bump the version.

The project cannot decide versioning semantics without knowing which class defaults fall into.

**Strongest attack:**

#61 is an implementation detail. Montagent is agent-first; agents write the file, and agents do not write defaults (fact 6: "no evaluated properties"). If `fmt` is a tool that the agent runs **after writing**, it is agent-controlled, not a contract. The format's semantics are unaffected by whether `fmt` emits defaults or not.

Versioning should not wait on an implementation detail. #14 is about the format contract; #61 is about tooling. Decouple them.

**Why it survives:**

The counter-argument misses the contract that matters: **between Montagent and a file written by someone else's tool**. If `fmt` materializes defaults, a file written by an older tool (without defaults) and a file written by a newer tool (with defaults) are **indistinguishable in bytes**. The version number on each cannot tell them apart. No problem: both versions are legal at the current code.

If `fmt` **does not** materialize defaults, then a file without defaults carries information: "I was written by an old tool, before defaults were added." If `fmt` later changes the default for some field, the old file decodes differently. This is **silent byte semantics**, exactly C1's danger. The version must distinguish them.

The severity class of default changes is **determined** by whether `fmt` materializes. Versioning must account for it.

**Falsifier:**

If the project states "the version only describes what was **explicitly in the file**, not what `fmt` adds" — then #61 is irrelevant. The version distinguishes files by their declared bytes, not by rendering behavior. That is P5 (lower-bound) taken to its logical conclusion.

Under that reading, #61 does not block #14. But the counter-argument is that rendering behavior is what matters for users, and it *is* affected by whether `fmt` materializes.

**Confidence: 65%** — This is load-bearing on the specific question. #61 must be resolved first.

---

## Q10b — Unknown-key policy: new ticket, clause in #14, or neither?

**Preferred answer:** New ticket. C4 found that nothing in the codebase states a policy on unknown keys. This is **load-bearing** for Q4's missing row: "adding an optional field." 

If the schema rejects unknown keys, adding an optional field is breaking (old readers reject the new file).

If the schema permits unknown keys, adding an optional field is not breaking (old readers ignore the new field).

The question of whether optional additions are breaking **depends entirely on the unknown-key policy**. The policy is a precondition for versioning decisions, not a clause in the versioning ADR. It should be a separate decision, stated clearly.

**Strongest attack:**

A separate ticket fragments the decision space. Adding the policy as a clause in #14's ADR keeps the versioning contract together in one place. The reader of the version ADR will see "optional fields are not breaking because additionalProperties: true" and understand the full scope.

Also: the policy is not "new" — it is a clarification. Most modern formats (OpenAPI, JSON Schema) default to `additionalProperties: true`. Montagent may intend this as default and just never stated it. Stating it as a clause in #14 is faster than opening a new ticket.

**Why a separate ticket survives:**

Cramming the policy into #14 conflates two decisions: (1) how to version, (2) what unknown keys mean. They can have independent answers. A future ADR might tighten the policy (prohibit unknown keys) without changing versioning semantics.

More importantly: **the policy is not about versioning at all**. It is about schema semantics. The brief's outline of "breaking changes" (Q4) lists it as a dependency, but it is not a versioning property — it is a **schema property**. The schema should state it. If a new ADR states it, the versioning ADR can reference it and be done.

C4 says the codebase already rejected one (likely false) claim about the policy. Stating it as a new ticket makes that debate visible and falsifiable, rather than hidden in a versioning ADR's fine print.

**Falsifier:**

If the project states "versioning ADRs must be self-contained and list all preconditions" — then the policy clause should be in #14. But versioning can reference other ADRs (e.g., ADR-0006 on schema validation), so that is not a constraint.

**Confidence: 73%** — New ticket is cleanest, but is higher friction. The project might prefer a clause. Either works if stated clearly.

---

## What I could not settle

1. **Whether a published JSON Schema will exist before or after release.** Fact 2 says no schema artifact exists yet; fact 10 says the schema and format docs will ship as MCP resources. The timing determines whether P8's "schema publication = binding point" applies. If the schema is published at 1.0 release, P8 and my P9 answer converge.

2. **Whether the project will commit to supporting files at arbitrary past versions.** This is the load-bearing assumption for P5 (lower-bound) to be useful. If the project only supports files at the current version, versioning degenerates to a changelog (informational only) and P3 becomes right.

3. **Whether migration tool development is in scope for this ADR or is a future decision.** I assumed "yes" (P11), but the brief does not commit to it. If no tool is built, the version number is orphaned (agents use `validate` errors, not the version).

4. **What severity class default-value changes fall into** — this is #61, and it is a hard dependency for versioning.

5. **The unknown-key policy is entirely absent from the codebase.** C4 is correct. This must be stated before versioning is complete.

6. **Whether the fixture's byte-identity (M1) is evidence of high code quality or luck.** I assumed the former (correct files survive untouched), but it could be the latter (the fixture happened to use ratios that avoid rounding issues). If it is luck, the version is less useful as a stability marker.

---

## Confidence, per question

- **Q5** (marker at all, is it a version?): **75%** for yes + P1 (integer naming renderer revision). The main uncertainty is whether the file is self-identifying enough that a marker is unnecessary.

- **Q6** (lower-bound or equality?): **82%** for P5 (lower-bound). M1 is strong evidence. The uncertainty is whether migration tooling will exist to make lower-bound useful.

- **Q7** (what event binds?): **68%** for P9 (split mechanics/promise). This is pragmatic but inelegant. P7 (release) is simpler; P8 (schema publication) is more principled.

- **Q8** (montagent migrate?): **72%** for yes, P11 (at release, idempotent per-change scripts, tool dispatches + reports synthesis errors). Uncertainty is whether the project will build this.

- **Q9** (one field or two?): **71%** for P12 (one field, renderer's reading). C1 is interesting but does not topple the argument.

- **Q10a** (does #61 block?): **65%** Yes, with caveats. This depends on what #61 discovers. The logic is sound, but the prerequisite is open.

- **Q10b** (unknown-key policy?): **73%** New ticket. Cleanest separation of concerns. C4's note that the policy does not exist makes it load-bearing.

---

## Summary

The project needs a version marker describing the renderer's reading (P1, P12). The semantics must be **lower-bound** (P5) to avoid ceremony rewrites on correct files (M1 proves this is real). The binding point is **pragmatically split** (P9) between mechanics (at next ADR) and promise (at release). A migration tool is justified (P11) **only if** the project commits to supporting older files at release. Gravity removal works perfectly through error messages (A2), which is the right model for agent-first repair. The tool dispatches per-change scripts for mechanical migrations; synthesis is left to the agent.

**Critical blockers:** #61 (what do defaults materialize to?) and the unknown-key policy (C4) must be resolved before this ADR is final.
