---
status: accepted
amends: 0093 (a partial render's world-effects are established only inside `[from, to)`, so a partial render may publish its file over a world-effect `error` that would refuse the full render; ruling 6's withholding protects the deliverable, which a partial render never is), 0117 (a partial render adds one line to `not_checked_also`, naming its range and stating that a clean partial render says nothing about whether the full render will pass)
---

# A partial render's world effects stop at its range, and its report says so

**Ticket:** [#501](https://github.com/MBehtemam/Montagent/issues/501), building
[#494](https://github.com/MBehtemam/Montagent/issues/494)'s rulings, on the map
[#383](https://github.com/MBehtemam/Montagent/issues/383).

## The question, and the two premises the code corrected

`render --from 0 --to 1000` printed every `review` finding in the project, 98 of them about
instants out to 269 s. #494 asked whether a partial render's findings should be scoped to its
range. It framed the answer as a semantics change that amends ADR-0006, on the condition that
*"`error` refusal must stay whole-project, or a partial render could succeed on a project the
full render refuses."* Both premises were wrong.

1. **Scoping the printout needs no amendment.** ADR-0006 already says *"Output may be filtered;
   analysis may not."* Only scoping *what is checked* would amend it, and nobody proposed that.
2. **The condition was already false.** Every `validate` check does run on the whole project
   first, and any `error` refuses (`render.rs`, before the range is read). But `render`'s own
   world-effects are read off the span. `Mix::of` reads only the audible elements inside
   `[from, to)`, and the painter paints only those frames. So an audio element that cannot be
   mixed at 200 s refuses the full render, and `render --from 0 --to 1000` publishes its file.
   **Nothing in the report said so.** That silence was the actual defect.

## Decision

### 1. A partial render publishes over a world-effect outside its range

A partial render is written to a derived path and refused if that path is the declared
`output` (story 56), so it is never the deliverable. Its file is correct for its range: every
element inside it was mixed and painted, or the render refused. ADR-0093 ruling 6's
withholding exists to keep a wrong file off the output path, and the full render still
pre-flights everything and refuses.

The alternative was to pre-flight the whole project's mix and painter refusals on every
partial render. That would make #494's condition true, but it would probe media the iteration
never touches on every scene render, and refuse a correct one-second file over an unrelated
defect at 200 s. The problem was that the limit went unstated, not that it existed.

### 2. The report states the limit

A partial render adds one line to `not_checked_also` (ADR-0117), after `render`'s existing
line about `verify`:

> Whether anything outside `{from}..{to}` ms would stop the full render. This was a partial
> render: every validate check ran on the whole project, but elements not mixed, not painted
> or painted without a field (ADR-0093) are looked for only inside that range, so a clean
> partial render says nothing about whether the full render will pass.

It states both halves in one line, so the report does not contradict itself. It appears
wherever the invocation settled a range, **refused or published**, because the scope of what
`render` looked for is the same either way. It never appears on a full render or on an
invocation whose flags were rejected. `render` reads the range from its flags a second time
to attach it, and that is safe because the reading is a pure function of the flags.

### 3. The printout is not scoped

Every finding prints as it does on a full render. After [#419](https://github.com/MBehtemam/Montagent/issues/419)'s
collapse (ADR-0099), the 98 out-of-range findings cost a few lines, so there is no volume
left to recover. A default that hides the project's `review` findings from its most frequent
command would be a quiet partial silence, which is the shape this map exists to remove.
**ADR-0006 is not amended.**

**The known cost.** When a code has more than ADR-0099's three instances, the one instance
inside the range, which a scene author most wants, is folded into a count together with all
the others. Finding it takes `--verbose` and a search. **The remedy if that cost shows up in a
real session is an opt-in filter** (e.g. `--findings-in-range`). It is out of scope here, and
making it the default was rejected as a silence.

This was put to a three-juror court and split 2–1. The dissent voted to scope by default,
hiding only `review`/`note` findings tied to an out-of-range instant and printing every
`error` and every finding with no instant. Its strongest argument rested on a factual error:
that the collapse could print three out-of-range instances while folding the in-range one into
a count. It cannot. A code past the bound prints **no instance at all** (`text.rs`, `bounded`),
precisely so the printed set is never an arbitrary sample. The corrected form of the argument
is the cost stated above.

Not scoping makes one of #494's sub-questions moot: how a finding that spans the range
boundary counts.

## Consequences

- **`render --from/--to`'s report gains one `not_checked_also` line**, in the JSON and under
  `NOT CHECKED` in the text. The finding set and the printout are otherwise unchanged.
- **A clean partial render is not evidence about the full render.** Only the full render, or
  `validate` for the checks it owns, answers that.
- **`preview` is untouched.** It is not a deliverable, has its own budget (ADR-0021), and #494
  asked only about `render`.
- **`CONTEXT.md`** gains **Partial render**.

## Evidence

`tests/world_effects.rs`'s
`a_partial_render_publishes_past_a_world_effect_outside_its_range_and_says_so` places an
unreadable audio source at 1000..2000 ms. The full render refuses, and its report carries no
partial-scope line. `--from 0 --to 1000` publishes, and its JSON and text both carry the line
naming `0..1000 ms`. Against the previous `render.rs` the test fails at the line assertion
**after** the publish assertions pass, which reproduces the silence on `main`. Court ballots
(Claude Opus 5.5, Sonnet 5.5, Fable 5.1) are in
[#494](https://github.com/MBehtemam/Montagent/issues/494).
