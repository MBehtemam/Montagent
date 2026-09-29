You are a juror on a design question for Montagent, a tool that reads, checks and renders a declarative video project for AI agents. Answer from the brief below alone. Do not use any tools, do not read files, do not edit anything, and do not address recommendations to anyone. Answer and nothing else, in the exact ballot format at the end.

## Background

Montagent's `frame` verb renders one frame of a video to an image an agent can look at. A new range mode (`frame --from A --to B`) returns ONE labelled contact sheet: a grid of tiles, one per visual state in the range, each tile sampled at the first frame the grid paints in that state. The effort's destination: an agent can see a span of the video in one call, "honest enough that 'I checked the whole thing' is an artifact rather than an argument."

The effort was founded on an experiment. A trial video was doctored with two defects that the checker (`validate`) passes: a wrong photo (a clip shows the previous clip's image, visible only as a relation between neighbouring tiles) and an invisible sentence (text coloured identically to its background, visible only in pixels). One of three trial agents hand-rolled a sampler that was structurally blind to colour, then claimed full coverage. Its silence read as coverage. That is the effort's founding failure, and its settled defence is **disclosure**: every range answer carries, unconditionally and in both plain text and JSON:

- a per-tile **provenance list** in text: each tile's sampled instant, its run boundary, and its full presence set (the element ids visible in that state, e.g. `sentence-08` present on tile 13);
- a fitted one-line **label** drawn in a gutter on each tile, in pixels: `<index> <sigil> <instant>ms <±offset> <±id>`, type floored at 8 px as served;
- **`blind_to`**: a fixed list of five tokens (`inside-run`, `between-keyframes`, `below-tile-width`, `across-sheets`, `audio`), each bound to one fixed sentence, printed on every answer, never a finding. It is defined as the sheet's "NOT CHECKED" block: statements about the *selection rule's* structural blindness, constant and learnable, never varying with the document, judging nothing. Two of its sentences already name a follow-up call (`frame --at <instant>` for change inside a state; `frame --crop --at <instant>` for detail below tile width).

The sheet's budget is served tile width: a 180 px target, a 140 px floor below which the call refuses. Both the 140 px width floor and the 8 px type floor were set from one primed observer, then re-measured with cold observers.

Standing rules of the effort: no verdict layer (Montagent never says "tile 7 looks wrong", never scores, never ranks instants by interest). Every behaviour must be ratified in an ADR before it is spec. A prior ADR bound a spec constant to "the tier the API serves" (the image resolution the host API serves), a host property that is documented and stable.

A prior incident: an agent read a *correct* disclosure in a `frame` answer and still filed the behaviour as silent, because the tool's description had promised otherwise — "a promise in the tool description outranked a fact in the answer." One incident, one agent.

## The new evidence

42 cold readings, each a fresh subagent shown exactly one image with no text and no defect list, told only "the video's author says something about it looks wrong." Readers: Haiku 4.5, Sonnet, Opus.

- Picture sheets (one 18-tile sheet, downscaled so only width varies): Sonnet found all three planted defects from 184 px down to 120 px; Opus down to 92 px. **Haiku 4.5 found none at any width, including the 184 px target** (0 of 21, one mis-indexed partial). It confidently invented defects that are not there (a duplicate "5", an umbrella scene, a dreamcatcher). On the clean control it reported only invented problems.
- Label sheets (transcribe the gutter labels): Sonnet and Opus transcribed exactly at 8 px, first breaking near 5 px. **Haiku never found the label strip at any size, including 10 px**; it transcribed the captions inside the video frames instead.
- So both floors hold for a capable reader and mean nothing for this weak one, and it fails confidently: the founding failure relocated from the sampler into the reader. Montagent cannot know which model is calling. One reading per model per cell, but Haiku's failure is not a threshold effect.
- Untested: whether a weak reader given the sheet **plus** its plain-text provenance list and `blind_to` does any better, or notices it cannot read the sheet. The plain-text answer is not built yet; testing it would mean mocking it from the ADRs — a separate prototype ticket.

## The Question — answer all three parts

**Part 1. Is the weak reader this effort's problem at all?**
- (a) Out of scope: reader capability is the caller's choice, like choosing a model for any other task. Record why and close.
- (b) Spec fact only: the ADRs record that both floors are reader-conditional (notes already drafted for the two floor ADRs). Nothing `frame` prints changes.
- (c) The answer owes something: every range answer carries something addressed to the reader's own capability. One candidate form: a fixed sentence exploiting the label/provenance redundancy as a self-check, e.g. "Tile 1's label reads `<exact text>`; if you cannot read it on the image, this sheet is below what you can see, and `frame --at` per instant is the call to make." If you choose (c), say whether it belongs in `blind_to` (whose definition is "about the rule", not the reader) or somewhere else, and whether the self-check is the right form or something else is.

**Part 2. Does anything `frame` prints — answer text or tool description — ever name a model or model class** (e.g. "floors measured on Sonnet/Opus-class readers")? Yes / No / only in the tool description. Consider that model names rot, and that the prior host-tier binding was a documented host property rather than a model capability.

**Part 3. Decide Part 1 on the argument now, or first measure** whether a weak reader given the full plain-text answer finds the defects or notices its own failure? (i) Decide now, measurement optional as a later prototype ticket; (ii) measure first, Part 1 waits; (iii) decide now and make the measurement a condition of shipping whatever Part 1 decided.

You may reject any part's framing if its options are all wrong; say so in that part's ballot.

## Ballot format — exactly this, once per part

🗳️ **Juror <n>** (<the model backing you>) — **Part <k> VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <what it costs, and why not the other options>
