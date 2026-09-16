# The Question

You are a juror on an independent panel. Answer from the material given here only — do not
search the internet, do not assume context you weren't given, do not look at any repository.
You will not see the other jurors' answers and they will not see yours.

## Background

Montaget is a declarative, agent-authored video editor. A project file is a flat list of
`elements` (image, video, audio, text, rect, ellipse) placed on named `tracks`. Every agent
that has edited this file edits it by **exact-string replace** — find a unique substring,
replace it. That convention only works if the file's layout is stable: a previously accepted
decision requires elements to be written one per line, sorted by `start`, in a stable key
order, specifically *because* it makes a unique matchable substring exist. Nothing currently
published says a writer (a tool, or an agent editing by hand) may not reorder keys or
reformat the file, as long as the result is still legal JSON that means the same thing.

**The incident.** One agent predicted the failure: *"if a tool reorders my keys, my next
exact-string replace gets zero hits."* Then a different agent demonstrated it by accident —
asked to make a routine edit, it rewrote a real 154-line project file into 1595 lines,
silently pretty-printing the whole thing and destroying the one-element-per-line convention.
It reported "no schema errors," and by the letter of every existing rule it was correct:
**nothing forbids what it did.** The file was restored; production data was untouched, but the
gap is real.

Separately, a related open question (#61) found that the project's planned `fmt` tool might
also *materialize defaulted fields* (writing out six fields that are currently allowed to be
omitted) when it runs — which, if `fmt` also normalizes key order, means a single `fmt`
invocation could produce a large, hard-to-review diff for two independent reasons at once.

`validate` is a separate, already-accepted tool. It answers exactly one question — "is this
project file internally legal, and does it agree with the media on disk?" — and never asserts
intent the document doesn't carry. It runs unconditionally on every file, no fast mode,
nothing scoped out. Its own governing principle: an opt-in check is worth nothing, and a
noisy check that fires on the majority of correct files is worse than no check, because it
gets disabled on first contact and teaches false confidence.

## The four questions

**Q1 — Should the writing convention (one element per line, sorted by `start`, a stable key
order) become an enforced requirement, or stay a documented-but-unchecked convention?**
Options: (a) leave it as folklore a formatter happens to produce, with no rule anywhere; (b) a
documented `SHOULD` with no tooling that checks it; (c) a `MUST` that some tool actually
checks compliance against.

**Q2 — Should the canonical key order be one single global fixed sequence applied to whatever
keys happen to be present on an element, or a separate canonical order per element `type`
(image/video/audio/text/rect/ellipse each having their own)?**

**Q3 — Where should compliance be checked: only in a `fmt --check` mode (a tool you have to
remember to invoke), in `validate` (which already runs unconditionally on every file,
including ones nobody ran `fmt` on), or both?**

**Q4 — Given the `fmt`-also-materializes-defaults finding (#61), should `fmt` be split into a
non-destructive check mode and a separate rewriting mode, so an author can see what a `fmt`
run would change before it silently lands both a key-reorder and a default-materialization in
one large diff — or is that unnecessary caution?**

## How to answer

For each of the four questions, cast one vote in exactly this format:

```
🗳️ **Juror** (<the model backing you>) — **Q1 VOTE: <your chosen option>**

**Reasoning:** <why>
**Trade-offs:** <what it costs, or why not the other options>
```

...and the same block shape for Q2, Q3, Q4. If you think a question's framing is wrong or its
options are all bad, say so as your vote for that question rather than picking one you don't
believe. Do not recommend anything about topics outside these four questions. Do not add any
commentary before or after the four vote blocks.
