---
status: accepted
amends: 0011 (`fmt` gains an explicit exception), 0015 (states the fixed-point behaviour of
  `fit` at exact-aspect match)
---

# Exact-aspect `fit`: both spellings stand, `fmt` leaves them alone

When a source's aspect ratio exactly matches its aperture's (`clip`'s), `cover` and
`contain` (ADR-0015) compute the identical rect, so two `fit` values describe one element
with no difference in output. Live in the committed fixture: `handle-logo` is an 800x800
source into a 68x68 aperture, giving 68x68 under either rule. [#56](https://github.com/MBehtemam/Montagent/issues/56)
asked whether `fmt` (ADR-0011), which normalizes the file on write, should canonicalize one
spelling, or whether both should stand as permanently legal.

## Decision

**Both stand. `fmt` must not rewrite a `fit` value that already agrees with what it derives,
even when a different legal value would derive the identical rect.**

`cover`/`contain` at exact-aspect agreement is not the hazard [ADR-0013](./0013-fitted-extents-floor-and-the-nine-origin-keywords.md)
already killed with `center-center` — that was one meaning with two spellings, redundant
under every future edit. This is **two different functions that happen to coincide at one
point in their domain**, a property of the *current* source asset's aspect ratio, not of the
field. The moment the source is swapped for one of a different aspect, `cover` and `contain`
diverge in opposite directions — `cover` crops past the aperture, `contain` letterboxes
inside it — so which one the author wrote is the entire behavioural specification for that
future edit, invisible only because no swap has happened yet. Canonicalizing would have
`fmt`, a tool whose charter (ADR-0011) is to be semantically inert, silently decide how a
future asset swap behaves.

It also reproduces exactly the failure ADR-0013 was written to prevent, on a *conditional*
trigger that makes it worse: an agent that writes `"fit":"contain"` and then edits by
exact-string replace gets zero hits the moment `fmt` has silently rewritten it to `cover` —
unpredictably, only on the inputs where the two rules happen to agree.

**This is consistent with `fit`'s own definition (ADR-0015): `validate` checks that a
derivation claim is *true*, never that it is the *unique* true claim.** At the fixed point
both `cover` and `contain` are true, so `validate` must accept either, and `fmt` has no
basis to prefer one — whichever it picked would misrepresent the actual claim for
whichever half of exact-aspect elements the author didn't intend.

## Evidence

Three-model court (Claude Opus, Claude Haiku 4.5, Claude Fable 5.1), each blind to the
others' ballots. **Unanimous 3/3**, independently converging on the same distinction: this is
not an alias (one meaning, two spellings) but two distinct, individually-true derivation
claims that coincide only for the current asset. All three separately named the same
consequence for `fmt`'s design charter — a formatter that rewrites this value is making a
design decision on the author's behalf, not normalizing a spelling.

## Consequences

- `fmt` (ADR-0011) gains a documented exception: it must not rewrite a `fit` value when both
  the declared value and at least one other legal value derive the identical rect from the
  current source and `clip`. The declared value is left exactly as written.
- `validate` (ADR-0006, ADR-0015) is unaffected in mechanism — it already only checks that
  the declared extent equals *the* value the declared `fit` rule derives, which both `cover`
  and `contain` satisfy independently at the fixed point. No new check is added.
- Two project files can be textually different (`cover` vs `contain` on the same element) and
  render byte-identically today. Diff-based and byte-equality tooling must treat this as
  meaningful divergence in intent, not noise — documented here so a future check does not
  "fix" it by canonicalizing after all.
- `handle-logo`'s existing `fit` value in the committed fixture is unaffected; this decision
  makes no fixture change.

## Not settled here

- If canonical-form purity is ever wanted for its own sake, the correct place to seek it is
  upstream of `fmt` — for example, requiring `fit` only when `cover` and `contain` disagree —
  not a `fmt` rule that guesses at author intent it cannot observe. No such change is proposed
  here.
