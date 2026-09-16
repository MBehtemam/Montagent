# Juror: Haiku

Given [BRIEF.md](./BRIEF.md) in isolation — no access to ADR-0039, no access to the other
jurors' ballots.

---

## Question 1: What unit should the check range over?

The check should not range over `group` as currently defined. `group` is established as a "purely a vocabulary/organizational label" — a signal that elements are "conceptually one item," not that they should move together. Evidence #1 is determinative: the check fires on "the overwhelming majority of correct, intentional documents" when measured against a real, shipped project. The photo cards, captions, and quiz groups all show keyframe-time disagreement among members because vocabulary grouping and motion coupling are separate concerns. A narrower "motion unit" could theoretically work, but the project's governing principle states: "an opt-in check is worth nothing" — and any narrower opt-in mechanism would meet that fate immediately. The real problem is not the unit; it is that **there is no syntax in the project file to express "these elements should move together."** Until that syntax exists, any validator check will fire on files whose author never claimed the relationship, violating the principle that "findings must never assert intent the document doesn't carry."

## Question 2: Should it be scoped per-property, and if so how is that expressed/discovered?

The check *is* already per-property (Evidence #2 documents this), but nothing says so. An agent discovered by trial-and-error that mixing `scale`-animated and `opacity`-only elements in one group is allowed. This hidden scoping is worse than no scoping: it means authors have no way to know what the check does without hitting false positives. If the check were to stay, per-property scoping would need to be: (a) explicitly documented in error messages, and (b) paired with a way to express cross-property coupling intent. Without both, authors cannot distinguish "the check is scoped per-property, so mismatched opacities are fine" from "I accidentally left a property out of my stagger definition." **Currently, there is no discovered or expressible per-property syntax.**

## Question 3: How would a deliberate stagger be expressed?

Evidence #3 shows the check cannot distinguish a correct 35600 / 36300 / 36700 ms stagger from a bug. Deliberate staggers need either: (a) a keyframe-level or group-level annotation (e.g., `"staggered": true` on a group, or a `+stagger` ease type), or (b) an entirely different system for motion grouping that separates "these move together" from "these move in order." Without such syntax, **any per-group check will produce false positives on staggers, violating the noise-budget principle.**

## Question 4: Is the cross-property hand-off case in scope, or does it belong elsewhere?

**It belongs elsewhere entirely.** Cross-property hand-offs ("property A's animation ends exactly where property B's begins") are a *before/after* relationship — the hazard is that an edit *broke* a coupling that *used to hold.* The `validate` tool sees only one document and **cannot report that something used to be true and no longer is.** This is the exact case `compare` was designed for: take two versions of a project and report what changed. The check, as written, fires identically before and after such an edit, discriminating nothing. This is not a flaw in the check's implementation; it is a flaw in its placement. `validate` is the wrong tool.

---

**Verdict: The group keyframe-time check should be retracted from `validate` and split into two separate mechanisms.**

**Reasoning:**

The check violates the project's core governing principle: "a noisy check is worse than no check." Evidence #1 proves it fires on the overwhelming majority of correct files in real projects, meaning it would be disabled on first contact by any author using it. The project further states that findings "must never assert intent the document doesn't carry," but this check fires on vocabulary groups where no motion coupling was ever claimed.

The check conflates two separate problems:
1. **De-syncing a multi-element entrance that once moved together** (a change over time — belongs in `compare`)
2. **Expressing intentional motion patterns in a single document** (staggers, disparate motion, per-property coupling — belongs in new optional project-file syntax)

**Replace with:**

- **In `compare`** (not `validate`): A check that reports "elements in group G had identical keyframe `t` values in the previous version but now differ on property X." This catches the real hazard — an edit that broke an existing coupling. It works because `compare` sees both versions and can report that a relationship held and then stopped.

- **In the project file schema** (not validation): A new optional `motionGroup` field or keyframe-level `stagger` metadata to let authors express intentional non-matching timing. Until this syntax exists, staggers and disparate motion in vocabulary groups cannot be distinguished from bugs by any single-document validator.

- **Cross-property hand-offs** are only detectable in `compare` (detecting loss of the relationship), and require the same before/after semantics.

The correct tool (`compare`) already exists. The missing pieces are: project-file syntax to express motion intent, and a `compare` check to detect when that intent breaks. A noisy single-document check solves neither.
