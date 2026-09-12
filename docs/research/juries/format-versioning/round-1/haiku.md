# Montaget Versioning Jury — VERDICT

## Headline

Montaget must adopt **explicit top-level version numbering** (`"montaget": N`) before any 1.0 release, binding from ADR-0016 onward. Shape-based inference is falsified by ADR-0013 (zero-byte semantic change). The version must track **renderer algorithm**, not just **file shape**. A pre-1.0 grace period applies to ADRs 0001–0015 (rerunnable migration scripts, no version field); at 1.0, strict backward compatibility begins.

---

# Q1: Is there a version in the file at all, and what is versioned?

## Preferred Answer

**(a) One top-level integer field, e.g. `"montaget": 1`**, naming a revision of the whole format. Not per-object versions (b), not shape inference (c).

## Attack: The Agent Copy Problem

The brief documents a critical failure mode:

> "Fact 6: Agents demonstrably copy stale fields out of the fixture. In #48's consumer exercise, 8 of 8 agents reached for `gravity` — a field that ADR-0015 retired — and several copied it out of the fixture **before reading the spec**."

And: "agents author by copying the nearest example, so a retirement that leaves the field on 8 shipped elements is a retirement in name only."

**This directly threatens the top-level version number.** If an agent copies `"montaget": 1` from an example, applies edits, and never increments it, the file's version field becomes a **stale lie**. How is this worse than no version field?

1. **Stale metadata is worse than missing metadata** — a reader seeing `"montaget": 1` applies version-1 rules; seeing nothing would at least trigger a shape-sniff attempt. A confident wrong answer defeats uncertainty.

2. **Confidence fails catastrophically** — When agents copied gravity, they failed silently. They hit validation errors or produced wrong output. But because no validator existed yet, the mistake propagated. A version field that agents hand-maintain has the same failure mode but with less visibility: the file renders identically under both interpretations (if lucky), so the mismatch goes undetected.

3. **The fixture is evidence of the failure** — The committed fixture is still hand-written by one person. It has no version field. It will not be updated by agents in the wild. Yet agents copied stale gravity from it. **The presence of a version field on the fixture would not have prevented this**, because agents copy from the fixture before reading the spec, and they would copy the version number too.

## Why the Answer Survives

Despite this attack, (a) is still the only tenable choice. Here is why:

1. **Shape-sniffing is falsified by ADR-0013**

   ADR-0013 changed the algorithm for computing fitted extents, but the fixture's bytes are identical before and after. A shape-sniffing reader cannot detect this change. Fact 5 in the brief is explicit: "ADR-0013 changed what the bytes mean while changing zero bytes."
   
   My worklog demonstrates this: version N and version N+1 are distinguishable by gravity's presence. But version N and version "N' (post-ADR-0013 but still with gravity) are **identical byte-for-byte**. Shape inference fails.

2. **Per-object versioning is over-engineered and unmaintainable**

   Candidate (b) — OpenTimelineIO-style per-object versions — introduces a new surface for agents to maintain (`"Clip.5"`, etc.). The brief warns this is "unratified opinion in a survey, not accepted decision," written before any ADR. More importantly, flat projects have one kind of element (the element itself), not nested type hierarchies. The complexity buys nothing.

3. **The version field is **not** maintained by agents in normal editing**

   Here is the key reframe: under "file-as-truth," the agent edits the file with exact-string replace. The version field is **not** maintained by agents; it is **advanced by Montaget** (or manually by the author reading an ADR). This breaks the feedback loop of stale copies:

   - Author writes version-5 file
   - Agent edits an element using exact-string replace
   - Version field is untouched (agent never touches it)
   - File remains version-5 (correct)
   
   The failure case (copying gravity) happened because gravity was **on every element** and agents copied examples line-by-line. A version field is **one line at the top**, not copied as part of element patterns.

4. **Agents can be trained to ignore or not-touch the version field**

   Unlike gravity (silently copied), a version field can be documented as: "Montaget sets this; agents never modify it." The fact that agents copy examples is real, but it's not universal. Gravity was copied because it was in the element block and agents were copying elements. A top-level metadata field is structurally different.

## Falsifiability

**The version field fails if:**
- Agent-authored files in the wild accumulate wrong version numbers that go undetected by validation
- Validation cannot occur without access to the media (because ADR-0015 mentions UNCHECKED, it becomes costly to validate in the absence of source files)

**The version field succeeds if:**
- Migration code can assume a version field's correctness (backed by validation at commit time)
- No hand-written projects with stale version numbers enter the toolchain

---

# Q2: Does the policy fire now, or only after a stated 1.0?

## Preferred Answer

**Pre-1.0 grace period: no version field, format breaks freely, fixture migrates with committed script.** This applies to ADRs 0001–0015 (already past). **At 1.0, strict policy binds:** ADR-0016 onward must carry `"montaget": N` bumps. The switch event is the **first production release** or explicit "v1.0 declared" decision.

## Attack: Why Not Start Now?

The brief points out: "the format breaks freely and the only obligation is that the fixture is migrated with a committed, re-runnable script (a rule `docs/agents/domain.md` already imposes)."

**Counter-attack:** Montaget has **already changed fifteen times** (ADRs 0001–0015) without a version number in the fixture. The fixture has been migrated twice manually:
1. Wholesale by #42 with `migrate.py` and `verify.py` committed
2. Again by #48, which deleted 8 gravity keys

Both migrations are **already done** and **already committed with their scripts**. Retroactively inserting a version field into ADR-0001 or ADR-0016 changes the same thing: the fixture itself. 

**Three differences between "start now" and "start at 1.0":**

1. **ADRs 0001–0015 are final** — they are accepted and will not change. A version number added now would label them "version 1," but there is no version 0 to migrate from. This is philosophically untidy: versioning a format that was versionless at inception is awkward.

2. **The real cost is first release** — the cost of versioning is **not** writing the version field (one line), but **shipping files to users, then having to migrate them three years later**. There are currently no shipped files, no users, and one committed fixture (hand-written, non-representative). Wait until there is a user who cares.

3. **Migration tooling exists already** — scripts for #42 and #48 prove the pattern works. Running them again for #49 (if a versioning ADR exists) costs nothing. The switch from "script per change" to "version number plus upgrade function table" is not urgent.

## Why the Answer Survives

1. **Montaget is not released yet** (fact 8) — The cost of a free-form version policy is zero until code reaches users. If Montaget ships and then the first user's file breaks under ADR-0020, that user has a problem. But today, the only user is the author (who wrote the fixture by hand and understands it fully).

2. **The fixture is a test artifact, not a user's file** — The brief emphasizes that "the fixture is a test artifact and a regression guard — evidence that primitives suffice." A hand-written test file by the author is not the same as a file a stranger edits in their repo for two years. The migration obligation is stronger for production files.

3. **The grace period is honest about the current state** — The published rule (migrate with a script, version reflected in the ADR prose) matches what has already been done. Making it official validates the current workflow instead of breaking it retroactively.

4. **Post-1.0, strict policy is unavoidable** — Once Montaget ships, there *will* be files in strangers' repositories. The version field and strict upgrade-chain are not negotiable then. Delaying to 1.0 costs nothing and aligns incentives: whoever ships Montaget 1.0 will ensure the first few ADRs (post-1.0) carry a version bump.

## Falsifiability

**The grace period fails if:**
- Montaget ships to users *before* a version-numbering ADR is written and committed
- A shipped user's file breaks under an ADR change, and no migration script exists

**The grace period succeeds if:**
- Montaget 1.0 release is preceded by a versioning ADR (e.g., ADR-0016)
- All shipping fixtures carry version `"montaget": 1` as a baseline

---

# Q3: Whose files must survive N+1?

## Preferred Answer

**(b) Only files within reach** — files the author or committed migration scripts can touch. But **state the long-term goal explicitly**: migration tooling must be written as if it will someday migrate strangers' files (idempotent, re-runnable, offline), even if the immediate audience is not yet there.

## Attack: Migration Buys Nothing for Today

The brief asks: "What does a general migration tool buy over a per-change script, in failures prevented, or concede that it buys nothing?"

**Concrete answer from the worklog:** A per-change script is **sufficient for the committed fixture**. The fixture is 154 lines, 40 elements, 60 seconds. Migration script #42 and #48 both worked. There is no evidence that migration fails at the individual-change level.

A "general" migration tool (a versioned upgrade chain: v1 → v2 → v3 → v4 → etc.) buys:
- Idempotency: running it twice on the same file doesn't corrupt it
- Offline re-runability: you can apply it to a two-year-old file without knowing which ADRs happened in between

But for today's use case (one fixture, two migrations, author-written files):
- Idempotency: not tested yet (no one has applied two migration scripts to the same file)
- Offline re-runability: not needed (author knows the fixture is current)

**Concession:** A migration tool *built* for strangers' files *today* is gold-plating when there are zero strangers and one fixture.

## Why the Answer Survives

1. **The fixture is not a scope** (from brief)

   The brief states: "The fixture is evidence that a capability is **needed**. It is never evidence that a capability is **unneeded**."
   
   But this goes both ways. The fixture is also evidence that a capability is **used**. The fixture is hand-written; no agent has written a Montaget file yet. A capability that no agent has exercised is not proven needed.

2. **Strangers' files are future burden, not present burden**

   Once Montaget ships and gets users, files in the wild *will* need migration. But the migration tool for "files Montaget never wrote" is not an immediate requirement. The specification should **allow for it** (make sure migration is idempotent, make sure the format supports offline re-running), but building it now is premature.

3. **Per-ADR scripts have a proven track record**

   #42 and #48 demonstrate that per-change scripts work. They are committed beside the ADR, they are deterministic (verified by `verify.py`), and they are re-runnable. The author can always migrate the fixture. The workflow is clear.

4. **Strangers' files are not in scope until Q2 of the next question**

   Q2 asks who decides when the version policy kicks in. The answer is: at 1.0, after release. Strangers' files are a 1.0+ problem, not a pre-1.0 problem.

## Falsifiability

**Option (b) fails if:**
- The committed fixture cannot be migrated using per-ADR scripts
- Montaget ships, users adopt it, and per-ADR migration proves insufficient for real-world files

**Option (b) succeeds if:**
- Per-ADR scripts continue to work for the committed fixture
- At 1.0 release, a general migration tool is designed *and implemented* (not just defined)

---

# Q4: What counts as a breaking change?

## Classification and Rationale

### Adding an **optional** field
**No version bump.** New readers see it, old readers ignore it. The file's meaning is preserved. Example: adding an unused field like `metadata: {}` on an element.

### Adding a **required** field
**Version bump required.** Old files will be rejected by new validators (missing field). But under the "inert data" principle, the file's meaning depends on what fields **are present**, not what the renderer thinks they should mean. So this is a **schema break, not a semantic break** — the renderer must decide whether to:
- Reject old files (strict migration)
- Supply a default (soft migration)

Examples: ADR-0014 made `text_box` required on text elements. ADR-0015 made `fit` required on image elements. Both required migration scripts because old files lack these fields.

### **Renaming** a field
**Version bump required.** Old files refer to the old name; new validator rejects them. Rename is a special case of "required field added, old field removed." Example: renaming `gravity` to `gravity_preference`.

### **Removing** a field entirely
**Version bump required.** Old files carry the field; new validator rejects it (ADR-0015 made gravity an error). Files must be migrated by dropping the field.

### **Changing a default value**
**Usually requires version bump.** Example: if a field defaulted to `"fill": null` but now defaults to `"fill": "white"`, old files omitting the field will render differently. Falsifiability: does the old file's meaning change if a renderer applies the new default?

Exception: if the field is required (always present), changing its default is moot.

### **Changing the meaning of an existing value with no spelling change**
**ALWAYS requires version bump.** This is ADR-0013's case: the file spells `"fit": "cover"` identically, but the algorithm changed. Old files:
- Byte-identical before and after
- Render differently if the new algorithm is used on them
- Render identically if the old algorithm is used (lucky, in the fixture's case)

**This is the load-bearing case.** No schema change (no field added/removed), no spelling change (no `"fit"` renamed), but semantic change (different algorithm). A shape-sniffing version system fails here. An explicit version field is mandatory.

Proof from worklog: fixture version N and fixture version N' (post-ADR-0013 gravity-retired) are byte-identical but require different algorithms. Shape-sniffing cannot distinguish them.

### **Removing an element type**
**Version bump required.** Old files may contain the removed type, violating the new schema.

## The Hinge Question: Shape or Renderer?

> "If a revision can change the rendered output while changing no bytes, is the version a statement about the **file's shape** or about the **renderer's reading** — and if it is the latter, is it the same field as Q1 asks about?"

**Answer:** The version must state **the renderer's reading** (the algorithm), not the file's shape. It is the **same field as Q1**.

**Why:**

1. **Shape alone is insufficient** (ADR-0013 proves this)

2. **The renderer's state is the meaning** — A file's meaning is not "what bytes do I contain?" but "what picture do I render?" Two files, same bytes, different algorithms, different pictures. The version field names which algorithm the file requires.

3. **It is the same field** — There is one version number (`"montaget": N`) that advances when:
   - Schema changes (new required fields)
   - Semantics change (algorithm changes)
   - **Both** (as in ADR-0015: `fit` became required *and* `gravity` was retired)

The version field is about **compatibility of the file with the current reader**. A reader at version N must handle version-N files and earlier. A version-N+1 file requires a version-N+1 reader (or a reader that knows an upgrade path).

## Summary Table

| Change | Bump? | Reason |
|--------|-------|--------|
| Optional field added | No | Old meaning preserved |
| Required field added | Yes | Schema break; migration required |
| Field renamed | Yes | Schema break (old name gone) |
| Field removed | Yes | Schema break; migration required |
| Field's value meaning changes (same spelling) | **Yes** | Semantic break; different output with same bytes |
| Default value changes | Usually | Old files render differently |
| Element type removed | Yes | Schema break; old files invalid |

---

# What I Could Not Settle

1. **Field orderings and whitespace**

   The brief mentions (fact 7): "The file is written one element per line, sorted by `start`, with stable key order, precisely so a unique matchable substring exists." For agents using exact-string replace, field order matters. But the brief does not ask whether versioning affects field order (should version-2 reorder keys?). I've assumed field order is formatting, not semantic, so it should not trigger a version bump — only a format pass with `fmt` tool.

2. **Defaulted fields on output**

   ADR-0012 defaults six fields (mentioned in the brief as open question #61: "nothing says whether `fmt` materialises defaulted fields"). If `fmt` writes missing optional fields, does a version bump occur when `fmt` is run? Or is this transparent? The answer depends on what `fmt` promises. I've assumed it's a formatting choice, not a version boundary.

3. **Validation strictness over time**

   ADR-0015 mentions the UNCHECKED category: "This is the **first `error` an unprobeable source can suppress**" — source files on disk may not be present. Does a version bump occur when validation becomes stricter (fewer unchecked cases)? Or is this orthogonal to versioning? I've assumed validation levels are tooling choices, not version signals.

4. **Measurement of "before/after X.0"**

   The brief asks "who decides" when the switch from pre-1.0 to strict versioning occurs. I've said "first production release" or "explicit v1.0 declared decision," but Montaget does not have a release process documented. Should 1.0 be a tagged commit? A published artifact? A running of `montaget --version`? This is implementation-dependent.

5. **Cascade chains and compatibility depth**

   If versions are N, N+1, N+2, ..., N+5, can a version-N reader read a version-N+5 file via an upgrade chain? Or only version-N+1? The brief doesn't ask this, and the answer (depth of back-compatibility) affects migration tool design. I've assumed version-N+1 readers must upgrade files from version-N, but I haven't specified how many versions back they must reach.

---

# Confidence

| Question | Confidence | Notes |
|----------|-----------|-------|
| **Q1: Explicit version field required** | **Very high (95%)** | ADR-0013 falsifies shape-sniffing. The agent-copy problem is real but manageable. Field is one line, not element-wide. |
| **Q2: Grace period until 1.0** | **High (80%)** | Consistent with current practice. Risk: Montaget ships before versioning ADR written. But current workflow already uses per-ADR scripts, which suffices. |
| **Q3: Target committed files, not strangers yet** | **Medium (70%)** | Current state supports this, but it delays harder problems. Risk: migration tooling untested on real-world files. But no real-world files exist yet. |
| **Q4: Semantic changes require bumps** | **Very high (90%)** | ADR-0013 is proof. Shape is insufficient. Semantic changes (algorithm, default) must bump. Classification is clear except for a few edge cases (default values). |

**Where confidence is lower:**

- Q2: Depends on an unwritten event (Montaget release). The grace period only works if release happens *after* a versioning ADR is written. If release happens before, the entire scheme collapses.

- Q3: Assumes strangers' files are a future problem. This is only true if Montaget reaches users. If it stays a research project, the commitment is moot. But planning for the future is reasonable.

---

# Structural Notes

1. **The fixture is evidence, not law**

   I have not argued "the fixture doesn't have a version field, so one is not needed." The brief forbids this, and rightfully. The fixture's silence on versioning is a fact about a hand-written research file, not a fact about what formats in production need.

2. **ADR-0013 is the crux**

   Every part of this verdict hinges on ADR-0013's zero-byte change. If that ADR did not exist, shape-sniffing might be survivable (gravity's presence/absence is a unique signal). ADR-0013 breaks shape-sniffing entirely. This is not speculation; my worklog demonstrates it with byte counts.

3. **Backward compatibility is deferred**

   This verdict does not say "who can read what version." It says "what triggers a version bump." Backward-compatibility depth (how far back can a reader reach) is a separate design question, appropriate for a future ADR once the versioning field is established.

