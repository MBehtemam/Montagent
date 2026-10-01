---
status: accepted
amends: 0007 (`dir` is drawn, as the isolate this ADR always said it was, and it never sets the line's base direction, which is the one the author's own characters give; `align`'s `start`/`end` resolve against that direction, line by line), 0093 (`E-FIELD-UNHONOURED` loses its first instance, `runs[].dir`, and has none: it stays registered as the code for the next field a build parses and does not draw)
---

# A run's `dir` is an isolate, and `start`/`end` follow the line's own direction

[#457](https://github.com/MBehtemam/Montagent/issues/457), which closes §2 and §6 of
[#277](https://github.com/MBehtemam/Montagent/issues/277). Reproduced on `fef18b78`: copy
`fixtures/en-halloween-decorating`, set `"dir": "rtl"` on the first run of `sentence-05`, and

```
$ montagent validate p.montagent.json        # exit 0
$ montagent render p.montagent.json --from 11000 --to 12000 --output out.mp4
error  E-FIELD-UNHONOURED  sentence-05
  `sentence-05`: `runs[].dir` was parsed and validated, and this build drew without it
exit 1                                        # no file written
```

ADR-0007 puts bidi in v1 and defines `dir` as a per-run `"ltr"`/`"rtl"` override with
**isolate semantics only**, *"not a new capability"*. The schema took it and `validate`
passed it, but no run's direction reached `measure`, the layout or the painter. ADR-0093 made
that honest: `render` refused and `frame` said so. Honest is still not drawn. And `query`'s
ink box refused any element with a `dir:"rtl"` run, because nothing said what `start` and
`end` mean on a right-to-left line.

## The decision

### 1. `dir` is laid out as an isolate, and as nothing else

A run carrying `dir` is laid out as though its text were wrapped in the matching isolate:
`"ltr"` → LRI (U+2066) … PDI (U+2069), `"rtl"` → RLI (U+2067) … PDI. Inside the isolate, the
run's own characters are resolved at that direction, by UAX #9 as `parley` implements it.
Outside, the isolate is one neutral: it reorders nothing around it, and nothing around it
reorders into it. **Embeddings and overrides (LRE/RLE/LRO/RLO) are never used**, as ADR-0007
rules: there is no spelling in the format that reaches one.

So `"x "` + `"abc!"`(`rtl`) + `" y"` draws as `x !abc y`. The letters stay a left-to-right
span (UAX #9 I2). The trailing `!` is a neutral between that span and the isolate's own
right-to-left edge, so it takes the isolate's direction and moves to the far side. An RLO
would have drawn `x !cba y`. `crates/montagent-text/tests/run_direction.rs` asserts both,
and that every glyph of the run stays between its neighbours.

**The marks exist only in the string handed to the shaper.** The document is unchanged, and
`measure` publishes nothing derived from the marked string. A line's `text`, its `start`, its
`break_opportunities`, the character counts the caption-pace checks read and the `highlight`
windows are all offsets into what the author wrote. The marks are default-ignorables: shaping
hides them, they advance by nothing, and a run of nothing but marks sets no line's ascent or
descent (ADR-0029's maximum is over the runs the author wrote).

A line that sets no `dir` is laid out from exactly the string it always was. No frame of a
project that does not use the field can move, and none of the committed goldens does.

### 2. A line's base direction is its characters', and `dir` never changes it

Each line is laid out as its own paragraph (ADR-0008's partition). Its **base direction** is
UAX #9's P2/P3 over **the author's characters on that line**: the first strong character
decides, and a line with none is left-to-right. **A run's `dir` takes no part in it.**

This is a higher-level protocol in UAX #9's own sense (HL1), and it departs from P2 applied to
the marked string on purpose. P2 skips an isolate's contents when it looks for the first
strong character. A line whose only run is a Hebrew run marked `dir:"ltr"` would then have
no strong character outside the isolate, fall back to left-to-right, and send `start` to the
other edge. That would mean adding a `dir` that changes nothing inside the run moves the
whole line. The rule here gives the invariant an author can rely on: **adding or removing a
`dir` never moves `start` or `end`.**

The implementation reads the base direction off the line laid out without its marks. It then
pins the marked layout to that direction with one leading LRM or RLM, so the shaper, the
painter and `measure` all see the same base. `rtl` on a measured line carries it to every
caller that aligns, so none of them guesses it from the characters.

A paragraph- or element-level direction field is **not** added. That is a schema change, and
#457 rules it out of scope. An author who wants a right-to-left line writes right-to-left
text, which is what the line's characters then say.

### 3. `start` and `end` resolve against each line's own base direction

`start` is the left edge of a left-to-right line and the right edge of a right-to-left one;
`end` is the opposite. `center` ignores direction. This holds **per line**, not per element,
so one element can hold a Latin line flush left and a Hebrew line flush right, each at its
own `start`. That matches how lines are laid out, each as its own paragraph.

This is the reading #277 §2 recorded, and it is now the rule. It applies everywhere a line is
aligned: the painter (`montagent_text::place`), the ink seam `measure` reports (ADR-0087),
and `query --at`'s ink box.

### 4. Shaping does not cross an isolate's edge

The text on either side of an isolate's edge is shaped separately. A kern, a ligature or an
Arabic join across the run boundary is lost. `cob` + `web`(`rtl`) is 0.8 px wider than
`cobweb` in the fixture's face at size 60. That is what an isolate is. The advance `measure` publishes is the isolated one, and it is the one the
painter draws, because both come from one layout pass.

### 5. `E-FIELD-UNHONOURED` keeps its registration and loses its instance

The painter no longer raises it for `runs[].dir`. That was its only raise site: the font
chain `index` that #390 was to reuse it for was honoured instead (ADR-0102). The code stays
registered, because ADR-0093 makes it the code for *any* field a build parses, validates and
draws without, and the next one is raised under it rather than under a new code. Its registry
entry is otherwise unchanged; only the comment that named `runs[].dir` is rewritten.

### 6. `query`'s ink box answers for right-to-left lines

The guard that refused any element with a `dir:"rtl"` run is removed, because nothing it
waited for is still undecided. The ink box now resolves `start`/`end` against each line's
base direction, the same direction the painter uses. That also fixes a case the guard never
covered: an all-Hebrew or all-Arabic line **with no `dir`** was given its `start` at the left
edge.

**Recorded, not settled here:** the ink box still aligns lines inside the element's declared
`width`, and places that box by `origin`. The painter aligns them inside the block the lines
make (#277 §1). The two agree on direction and disagree whenever a line's advance differs
from the declared `width`. That difference predates this ADR and is #277 §1's question, not
this one.

## Consequences

- A project that sets `dir` now renders. `frame` no longer lists such an element under
  `painted_partially`, and `render` no longer refuses it.
- `query --at`'s ink box changes for right-to-left lines aligned `start` or `end`, with or
  without `dir`: it moves to the edge the painter treats as `start` or `end`.
- `measure`'s JSON is unchanged. The line's direction is carried as `rtl` on
  `montagent_text::MeasuredLine` for the callers that align, and is not serialized. Publishing
  it would be a wire change #457 did not ask for.
- `crates/montagent-core/docs/format.md` (`montagent://format.md`) gains one bullet under
  *Text* stating rules 1–4.

## Not decided here

- LRO/RLO-style direction **overrides** (ADR-0007 rules them out), vertical writing modes and
  UAX #14 `SA` segmentation (ADR-0007 defers them).
- #277's §1, §3, §4 and §5, and §2's recorded residual: trailing whitespace is not hung.
