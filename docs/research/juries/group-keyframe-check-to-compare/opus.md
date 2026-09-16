# Juror: Opus

Given [BRIEF.md](./BRIEF.md) in isolation — no access to ADR-0039, no access to the other
jurors' ballots.

---

I've read the brief in full. Here is my ballot.

---

# Juror ballot: the group keyframe-time check

## 1. What unit should the check range over?

**None of the offered ones — because the correct unit is not a set of elements at all.**

`group` fails on measurement: five of five groups in a real, correct, shipped file disagree on keyframe times (evidence 1). That is not a tuning problem, it is a category error — `group` answers "what is this conceptually?" and the check asks "what moves together?" Those are different partitions of the same file, and the fixture proves they are different in practice, not just in theory.

A narrower opt-in "motion unit" (a new `motionGroup` field, or `group` plus a `coupled: true` flag) is excluded by the project's own governing principle: **an opt-in check is worth nothing.** And here the principle bites harder than usual, because of *who* breaks these couplings. The author who cares enough to declare "these two elements move as one" is precisely the author least likely to desync them; the edit that desyncs them is a later, narrower edit — often by an agent — that touches one element's keyframes and never reads, let alone maintains, a declaration attached to the other. An opt-in motion unit would therefore be absent from exactly the files where the hazard lands, and present in the files that were already safe. It also buys that nothing at the cost of adding a second render-inert label to the format whose only consumer is a validator — new vocabulary that every reader and every authoring agent must now learn and keep correct, paying for a check that fires on the population that doesn't need it.

The unit that actually matches the hazard is **a relationship between two specific keyframe times** — "these two `t` values were equal," "these three `t` values were 700 ms apart" — and a single document contains no evidence about which numeric coincidences are relationships and which are accidents. The only available witness to that distinction is the previous version of the file. So the unit is a *relationship*, and its scope is `compare`, not `validate`. `group` stays exactly what it is — vocabulary — and at most appears in a finding's wording as a hint ("both in group `lower-third`"), never as a gate.

## 2. Should it be scoped per-property, and if so how is that expressed/discovered by an author?

Per-property scoping is **necessary and nowhere near sufficient**, and the fact that it had to be discovered by an implementer hitting a bug (evidence 2) is itself the finding. Any comparison of keyframe times must be between the same animation channel — a `scale` ramp and an `opacity` fade have no reason to share a `t`, and the literal "all `t`s in a group must match" reading is simply wrong. But applying that fix does not rescue the check: the fixture's groups disagree *within* the same property too (a photo card's members animate the same channel on different schedules by design), and the deliberate stagger of evidence 3 is a same-property disagreement. Per-property scoping removes a class of absurd findings and leaves the noise floor above the ceiling.

As for how an author would express or discover the scoping: **they can't, and that is the decisive objection.** Nothing in the format says which channels are coupled. Any per-property rule `validate` applies is a rule the validator invented and the document never stated — so every finding it produces is an assertion of intent the document does not carry, which the project forbids outright. A rule that cannot be read off the file is a rule an author cannot predict, cannot satisfy deliberately, and can only learn about by being yelled at.

Under the `compare` relocation the question dissolves: comparison is per-channel by construction (a keyframe is identified by element id + property + ordinal), and the author discovers nothing, because there is no rule to discover. The before-version *is* the specification.

## 3. How would a deliberate stagger be expressed so it does not fire as a false positive?

**It would be expressed the way it already is — as three numbers — and it would not fire, because nothing would be asking.**

This sub-question is a trap worth naming explicitly. Every answer of the form "the author annotates the stagger" (`stagger: true`, an explicit offset field, a `motionGroup` the staggered elements deliberately don't join) violates the brief's own constraint — it requires every author who wants a stagger to already know about the check and opt out of it — and violates the noise principle from the other direction: a check you must disarm on correct files is the same check that gets disabled on first contact, just with extra ceremony. The end-card at 35600 / 36300 / 36700 is *correct*. A validator whose price of admission is that correct files must be edited to placate it has inverted the relationship between the tool and the work.

In `compare`, the stagger is handled for free and with zero author burden:

- A stagger that was always a stagger: both versions show 35600 / 36300 / 36700. Nothing changed. **Silent.**
- A stagger whose internal structure is destroyed by an edit — one element moved to 35000 while the others stayed: the 700 ms offsets that held in the before version no longer hold. **Reported.**

Note what that buys: `compare` doesn't merely stop false-positiving on staggers, it *catches the desync bug inside a stagger* — the exact coupled-delta failure the original check existed for, in a case the group check could never have handled, since a staggered group looks identical to a broken one when you only have one document.

## 4. Is the cross-property hand-off case in scope?

**Not for a same-property, same-group check — it is provably outside it (evidence 4: the check fires identically before and after an edit that destroys the hand-off, discriminating nothing).** But it does not belong to a *different* mechanism either; it belongs to the *same* mechanism, once that mechanism is relocated.

A hand-off ("property A's animation starts exactly where property B's ends") is, on a single document, an equality between two integers. `validate` cannot tell that equality from arithmetic luck, and declaring it a coupling would be pure intent-assertion. Across two versions it is a first-class relationship: it held, an edit moved one side, it no longer holds. The relationship-based `compare` check is **property-agnostic by design** — it compares keyframe times across channels and across elements alike — so the hand-off, the shared entrance time, and the fixed stagger offset are all one feature rather than three. That the hand-off falls out for free is evidence the relationship framing is the right one; that it was invisible to the group framing is evidence the group framing was wrong.

---

## Verdict

**Retract the group keyframe-time check from `validate` entirely and replace it in `compare` with a property-agnostic "keyframe relationship broken" check that reports, at `review` severity, any exact time equality or fixed time offset between keyframes that held in the before version and does not hold in the after version.**

---

## Reasoning

### Why retraction, not repair

Four independent agents hit four distinct failures. That pattern matters more than any single failure: they are not four bugs in one implementation, they are four symptoms of one root cause, which the brief states plainly and then almost lets slide past — **nothing in the file says these two elements were ever supposed to move together.** Every difficulty follows from that single fact. The check ranges over `group` because `group` is the only element-set the format has, not because it is the right one. It needs per-property scoping nobody can discover, because the coupling it is looking for was never written down. It cannot distinguish a stagger from a desync, because both are "times that differ" and only intent separates them. It misses the hand-off, because it was built around the one proxy available rather than around the relationship.

A check whose premise is absent from its input cannot be repaired by narrowing its scope. Narrowing trades false positives for false negatives along a dimension where the document carries no signal at all — and the fixture measurement (5/5 groups firing) says the starting point is not "slightly too loud." It says the check is anti-correlated with the thing it claims to detect: it fires on every correct file measured and, per evidence 4, fails to fire differentially on the coupling breakage closest to its target. Under this project's stated principle a check like that is strictly worse than no check, because a clean run would teach a false confidence `validate` is explicitly forbidden from implying — and a check that fires on 100% of correct groups will be disabled on first contact, at which point it protects nothing while having spent format vocabulary, author attention, and the credibility of every other finding `validate` emits.

### Why `compare` is not a consolation prize

The brief defines `compare` as the project's designated home for anything requiring a before/after — "detecting that a relationship held and then stopped holding." The commissioned hazard, read carefully, *is exactly that sentence*: the bar and the text were in sync, an edit changed one and not the other, and now they are not. The hazard was never a property of a document. It is a property of an edit. Filing it under `validate` was a scope error from the start, and the four agents' findings are what that error looks like when you try to implement it.

Relocation is also the only formulation that satisfies all three project constraints at once:

- **Noise budget.** The check is silent on every correct file that is not being edited in the relevant way, including all five fixture groups, because nothing about them changed. It speaks only about relationships this specific edit disturbed — a set that is small by construction and already the user's focus.
- **No intent-assertion.** The finding is "in the previous version these two times were equal; they no longer are." That is a verifiable fact about two documents. It never claims the coupling was intended — it reports that a numeric relationship the author's own prior file exhibited has been dropped, and lets the author judge.
- **Single-document scope.** `validate` keeps its promise of answering one question about one file, and stops being asked to infer authorial intent it has no basis to infer.

### Exact shape of the replacement (in `compare`)

**Candidate relationships** are extracted from the *before* document only, over transform-property keyframes (`x`, `y`, `origin`, `scale`, `rotation`, `opacity`), between keyframes belonging to elements whose time ranges overlap. Two kinds:

- **Coincidence:** two keyframe times are exactly equal. Both same-property/different-element (the lower third's bar and text entering together) and different-property (the cross-property hand-off, A's first `t` equal to B's last `t`) qualify — same-element cross-property pairs included.
- **Offset:** three or more same-property keyframe times across different elements form a constant non-zero spacing (the 35600 / 36300 / 36700 stagger, spacing 700).

**Keyframe identity across versions** is element id + property + ordinal within that property's keyframe list. A relationship is evaluated only if every keyframe in it still exists in the after document under that identity; otherwise it is dropped silently (an element or animation that was removed is already `compare`'s ordinary business and needs no relationship finding).

**Trigger condition:** the relationship held in before, all its keyframes survive in after, and it does not hold in after. Equivalently: if every participating keyframe moved by the same delta, the relationship survives and nothing is reported — which is precisely the "moved the whole unit together" edit, the correct one.

**What it reports**, at `review` (never `error` — nothing here is illegal, and `note` understates a likely-unintended regression):

> `review` — Keyframe relationship changed. `lower-third-bar.y[0]` and `lower-third-text.y[0]` were both at 12400 ms in the previous version; `lower-third-text.y[0]` is now 12750 ms (+350). Both elements are in group `lower-third`.

and for the stagger:

> `review` — Keyframe spacing changed. `card-a.opacity[0]`, `card-b.opacity[0]`, `card-c.opacity[0]` were evenly spaced 700 ms apart (35600 / 36300 / 36700 — note: 700/400); `card-a.opacity[0]` is now 35000, so the spacing is no longer even.

Group membership appears as context in the message, never in the trigger. The wording states what changed and stops — no "these elements are supposed to move together," no "this is probably a bug."

**Noise control inside `compare`:** report each broken relationship once, attributed to the keyframe(s) that moved; when one keyframe move breaks several overlapping relationships, collapse them into a single finding about that keyframe. If an edit moves many keyframes wholesale (a retimed sequence), the same-delta rule already suppresses the bulk of it.

### What remains in `validate`

Nothing group-ranged, and nothing about cross-element coupling. `validate` should still make the purely intrinsic keyframe claims it can make from one document without inferring anything:

- keyframe `t` outside its own element's time range (`error` — internally illegal),
- duplicate or non-monotonic `t` within one property's keyframe list (`error`),
- a single-keyframe list on a property, which animates nothing (`review` — likely a truncated edit, and a fact about the document, not about intent),
- unknown `ease` value (`error`).

These are legality facts. They fire on broken files and stay quiet on correct ones, which is the only bargain `validate` is allowed to make.

### What is honestly lost

An author who writes a coupled entrance and desyncs it **in the very first version of the file** gets no warning from anyone. That gap is real and I would not paper over it. But it is unclosable given the format as specified: with no declaration of coupling and no prior version, a desynced lower third and a deliberate stagger are the same document. The original check did not close that gap either — it "closed" it by firing on both, which is not detection, and by its own commissioning language was only ever "the mitigation, not a cure." The correct response to a hazard you cannot detect is to say so in the format's documentation (note that multi-element entrances are hand-maintained and that `compare` guards them across edits), not to ship a detector that cannot discriminate and then spend the validator's credibility pretending otherwise.
