# Jury Ballot: ADR-0040 mask contradiction

**Verdict: ACCEPT WITH MODIFICATION on candidate answer (A)**

The contradiction in ADR-0040 is real and requires amendment. The bare `mask: "circle"` key must migrate to `effects: [{"name": "mask", "shape": "circle"}]`, and ADR-0040's Consequences sentence ("no migration needed") must be retired by a new amendment ADR.

---

## The contradiction is genuine

Verified against the repository:

1. **The fixture contains the bare key.** Line extracted from `fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`:
   ```json
   {"id":"handle-logo","type":"image","group":"header","start":0,"end":65216,
    "source":"brand/logo-en.png","x":478,"y":96,"origin":"top-left","width":68,
    "height":68,"fit":"cover","clip":[478,96,68,68],"mask":"circle"}
   ```

2. **ADR-0040 schema clause (verified, lines 169-171):**
   > "The schema gains a discriminated union `effects: [{name, ...params}]` with exactly three members in v1: `blur{radius}`, `shadow{dx, dy, radius, color, opacity}`, `mask{shape: "circle"|"rect"|"ellipse", ...shape params}`."

3. **ADR-0040 Consequences clause (verified, lines 165-168):**
   > "`mask:"circle"` on `handle-logo` in the committed fixture becomes a valid declaration under this ADR rather than an inert stray field — no migration needed, since the value was already legal shape-vocabulary syntax; `validate` should stop treating it as an unknown key once the schema lands."

4. **ADR-0017 (verified) establishes closed schemas** at every object level: "The schema is closed at every object level, including `run` and `keyframe`. No `x-` prefix, no in-band escape hatch." Confirmed in Consequences: "Every nested object shape (`run`, `keyframe`, font-table entry) must be fully and correctly enumerated in the schema, with `additionalProperties: false` (or the schema language's equivalent) at every level."

5. **ADR-0016 (verified) requires unknown-key errors as the migration mechanism**, naming the bare-unknown-key error in its example. But ADR-0016 requires the schema to be closed (stated as a consequence that was unmet until ADR-0017).

**The fork in ADR-0040 itself.** Lines 59 of the `mask` paragraph (verified):
> "Shipping it converts that stray field into either a valid declaration or a validation error."

ADR-0040 acknowledges the fork and then resolves it (Consequences) without reconciling with the schema clause.

The contradiction is **unambiguously real**: if masks live in `effects: [...]`, then a bare `mask` key is an unknown key under ADR-0017. Under ADR-0017, unknown keys are closed-schema errors. No amount of reframing the brief makes both statements simultaneously true.

---

## Why answer (A) is correct

**The repair is fully determined — advise-class under ADR-0043.**

ADR-0043's uniformity rule states: if any instance a check can match could be load-bearing, refuse uniformly. But the evidence shows all instances are safe:

- **Fixture README (verified, lines 143-144):** "The badge's roundness is **baked into the asset** — `logo-en.png` is 800×800 RGBA with corner alpha 0 — not produced by the compositor."
- The PNG carries its own alpha channel, making the mask field inert in this element.
- The repair (moving the value into effects array) is mechanical and does not depend on author intent.
- Therefore the finding is **advise-class, not refuse-class**: the fix is printable and universally safe.

**Precedent from CONTEXT.md's Rejected terms section (verified):**

This is the established pattern for retired spellings:
- `gravity` → retired by ADR-0015
- `box` → retired, reason: "A field that is sometimes a rect and sometimes a reference cannot even produce a good error message"
- `align` on images → retired in favor of `x`/`y`/`origin`/`clip`
- `center-center` → canonical error naming `center`
- `#RRGGBBFF` → canonical error naming six-digit form
- `bold` → rejected, naming font-file alternative
- `none`/`fill` as `fit` values → retired

The pattern is consistent: a spelling that became inert or redundant is retired, the document is migrated, and the old spelling becomes a retired-key error that names the replacement. This is the treatment `mask` should receive.

**ADR-0040's own stated purpose supports this.**

Lines 55-66 state that the fixture's `mask: "circle"` is *evidence the capability is needed* (per ADR-0003's asymmetry). Migrating it to the effects mechanism does not refute this evidence — it fulfills it. The capability remains needed; the spelling changes.

**Mechanical reliability.**

ADR-0016 demonstrated (via the gravity-fork measurement) that agents reliably find and fix intent-dependent migrations only 1 of 6 times. This migration has no intent component — the value and its replacement are mechanically determined. The same measurement showed agents achieving 6 of 6 on mechanical deletions. This is the safest class of migration.

---

## Ruling on the four follow-up questions

### 1. Is this an ADR amendment or ticket-level decision?

**This is an ADR amendment, not ticket-level.**

Per `docs/agents/domain.md` (verified):
> "**ADRs are amended, never rewritten.** A later ADR that corrects an earlier one says so in its own text, and the earlier one gets a short pointer under its title naming the amendment and what in it no longer holds."

This contradiction exists *inside* an accepted ADR's own text. It is not a new design question (ticket-level), but a correction to an existing decision. A new ADR should:
- Cite ADR-0040 as the source
- State in its own title that it resolves the contradiction
- Amend ADR-0040's Consequences to retire the "no migration needed" sentence
- Commit the migration of the fixture as evidence

The amendment should live on ADR-0040's "Amended by" pointer.

### 2. Refuse class if a spelling is retired?

**Advise-class, not refuse-class.**

Per ADR-0043 (verified):
- Advise-class: "repair depends on document geometry or media that determines it fully"
- Refuse-class: "repair depends on knowing what the author meant, which the document does not and cannot carry"

The bare `mask` key falls in the advise-class:
- The fixture README states the PNG carries corner alpha 0
- The geometric replacement (move to effects array) is fully determined
- No element's rendering changes under the move, because the PNG alpha is what produced the rounded appearance, not a mask effect
- The sibling census would show all instances grouped by the same pattern: a bare mask key on an image element with no other rendering path that requires the mask

The uniformity rule (line 27-30 of ADR-0043) applies: "If any instance a check can match is capable of being load-bearing, the check emits `repair: "none"` for **every** instance it matches". But this is the rare case where *no* instance is load-bearing; they are all safe.

### 3. Who migrates the fixture, and when?

**The amendment-ADR ticket migrates the fixture, before acceptance.**

Per `docs/agents/domain.md` (verified):
> "**Commit the evidence an ADR rests on — checked before `status: accepted` is written, not after.**"

And per #168's stated rule (verified in the issue body):
> "a check that fires on it is wrong unless an ADR says otherwise."

The fixture is "the one artifact everything is judged against." If the fixture must change, that is a commitment an ADR must make. The migration must land in the amendment ADR's branch and be verified before the amendment reaches `status: accepted`. It cannot be deferred to implementation.

The migration is mechanical:
```json
// before
{"id":"handle-logo","type":"image",...,"mask":"circle"}

// after
{"id":"handle-logo","type":"image",...,"effects":[{"name":"mask","shape":"circle"}]}
```

### 4. Does this reveal a class of defects?

**Probably not a class; but a process defect should be addressed.**

The contradiction exists only in ADR-0040. No other ADR in the series exhibits this pattern — a decision body that explicitly names two incompatible paths and then proceeds with one silently chosen in the Consequences section, leaving the fork unresolved.

**However, the jury process failure is broader.** ADR-0040's Evidence section (verified, lines 177-192) notes:
> "A three-juror court (Opus, Haiku, Fable — via `/court`), each given the same question packet cold, blocked from each other's ballots..."

The court was unanimous on key points but split on others. The jury's ballots are cited as committed. **The fact that the Consequences section contains an explicit fork** ("Shipping it converts that stray field into either a valid declaration or a validation error") **and the jury did not catch that the Consequences silently picked one branch without resolving the contradiction** suggests jury checklists should flag:

- Explicit forks or binary choices named in an ADR's body
- Verify that the Consequences section resolves every named fork
- Verify that Consequences do not contradict the schema/decision clauses

The court reviewed ADR-0040 and did not catch this. No evidence of this defect in other ADRs.

---

## Single most important finding

**The contradiction survived because ADR-0040's Consequences paragraph names the resolution of a fork without explicitly retiring the alternate path.** The fork is visible in the body ("either a valid declaration or a validation error"), but the Consequences casually selects one ("becomes a valid declaration... no migration needed") without stating it is rejecting the alternative or reconciling with the schema clause that requires the other path.

This is not a disagreement about design direction — all three jury models should have caught and flagged this as an unresolved fork during court. The lesson: jury procedures should require explicit reconciliation of any binary choice or fork stated anywhere in an ADR's body text before the decision moves to `status: accepted`.

---

**Amendments needed:**
1. New ADR amending ADR-0040: retire "no migration needed" sentence, migrate fixture, define retired-key handling as advise-class
2. Update jury checklist per #78 / domain.md to flag unresolved forks in ADR text
