# Juror 2 — Sonnet 5.5 (round 2, BRIEF-2.md)

Recorded verbatim, unedited.

---

🗳️ **Juror 2** (Claude Sonnet 5.5) — **Q5 VOTE: B (check sets that ran)**

**Reasoning:** Q3 already decided the field names what ran (all / named subset / none). Check sets are the thing that actually ran; classes are a derived view. Fact 3 shows refusals are findings without being checks, so a class-based field cannot express "error raised, no checks run" cleanly. Fact 1 shows drift is a class only compare can raise, so class-level "checked" claims are easy to get wrong. B keeps summary as "findings raised, per class" and the field says who produced them.
**Trade-offs:** A makes the header trivially per-class, but it couples the wire format to the class taxonomy and turns "layout looked for" into a claim about classes rather than checks. It also hides that fmt's layout check is one of validate's. B costs a small mapping (check set to classes it can raise) to drive Q8's rule.

🗳️ **Juror 2** (Claude Sonnet 5.5) — **Q6 VOTE: B (recorded as checks execute)**

**Reasoning:** Fact 2 shows validate can return before any check runs, so a declared "validate" would lie on E-NOT-A-PROJECT, and a missing-ffprobe run would lie too. Recording at the point of execution means the field is true by construction, and a new verb or early return defaults to the safe `[]`. This is the same failure mode as ADR-0006: the false claim of coverage is the bug to prevent.
**Trade-offs:** A is simpler and greppable in one place, but it drifts from reality on every early exit. B needs the check runners to take or return the report, so it adds a little plumbing. It also needs a decision on partial runs. I'd mark a set when it completes, or record it as started and complete only when it finishes, so that a crash never claims full coverage.

🗳️ **Juror 2** (Claude Sonnet 5.5) — **Q7 VOTE: A (three names)**

**Reasoning:** Fact 4: no verb runs any subset of validate's checks other than fmt's layout check. Granularity nobody produces or consumes is speculative. Three names cover every existing run, and the field can grow later without breaking consumers if a verb ever runs a different subset.
**Trade-offs:** B (~25 names) would give precision, but it makes every new check a wire-format change. It also invites a stale registry that must be kept in sync with the checks, and it does nothing for the incident. A costs the ability to say "validate minus X"; nothing needs that today.

🗳️ **Juror 2** (Claude Sonnet 5.5) — **Q8 VOTE: A (adopt the rule, dropping 0 drift from validate's line)**

**Reasoning:** Fact 1 means validate's `0 drift` is a zero for a check it never runs, which is exactly the manufactured-confidence failure ADR-0006/0013/0041 exist to prevent. Keeping it would leave one false zero in the very line the incident was about. One rule with no special case is also easier to explain and to generate from the field. Snapshot churn is a one-time cost. The JSON `summary` keeps drift: 0, so parsers lose nothing.
**Trade-offs:** B preserves stable snapshots and a familiar six-slot line, but it keeps a lie and makes the rule "print zeros unless validate". A costs a changed line for the five validate-set verbs, and a reader who expects six slots must learn that a missing slot means "not looked for". I'd add that to the doc. Also note that on an early exit such as E-NOT-A-PROJECT under validate, the recorded set is `[]`, so the header should read "no checks run", not zeros. That falls out of Q6-B plus this rule.

🗳️ **Juror 2** (Claude Sonnet 5.5) — **Q9 VOTE: (a) yes; (b) agree with "Check set"**

**Reasoning:** (a) It meets all three tests. It is hard to reverse, because it is a wire-format change to every verb. It is surprising without context, because `summary` still has six keys while the text prints fewer zeros. It is a real trade-off, because it drops `0 drift` and the classes-versus-check-sets choice is made. It also amends three ADRs, so a new ADR should supersede or annotate the affected parts of 0006/0011/0013/0041 and link back to them, and those ADRs should get a pointer. (b) "Check set" matches the Q2/Q3 wording and the field's contents. "Engine" is vague and implies a component, and "scoreboard" misdescribes the summary. Define it as the named group of checks a run executed, and distinguish it from finding class.
**Trade-offs:** Not writing an ADR would leave the amendments scattered across four older ADRs, and the reasons would live only in a panel transcript. The cost of writing one is small. For the term, an alternative such as "coverage" would collide with test-coverage meaning, so I stay with Check set.
