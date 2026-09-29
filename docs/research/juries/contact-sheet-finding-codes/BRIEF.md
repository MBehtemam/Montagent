You are a juror. Answer the three questions below from the facts given. Do not use tools, do not edit anything, do not make recommendations to the person who convened you beyond your ballot. Reply ONLY with the ballot block specified at the end.

# Background

Montagent is a tool for agents that edit a declarative video project (a JSON file of tracks and timed elements: images, text, rects, audio). Its verbs include `validate` (reports facts about the document as findings; reaches no verdicts), `query` (a "cut list": the half-open intervals over which the set of present elements is constant), `render`, `preview`, and `frame` (rasterizes a still at one instant).

Findings have a stable code whose letter prefix is conventional: E- error, R- review ("legal, renders, and you must look at a frame to judge it"), N- note, U- unchecked ("the tool could not establish this"), L- layout, D- drift. The code is "the contract keeping the JSON and the prose in step, the handle for suppressing a class, and the identity a future `compare` diffs on" (ADR-0006). Severity is computed per instance, so one code may be declared at more than one class. Separately, every answer carries a fixed prose "NOT CHECKED" block stating what the tool cannot know (e.g. "validate verifies that the file is internally legal; it cannot tell you whether it says what you meant it to say") — a statement about the tool, not about the document, printed unconditionally on every answer. It is not a finding.

`frame` is gaining a range mode (`--from`/`--to`) that returns ONE contact sheet: one tile per *visual state*. Already ratified (ADR-0094):
- Take `query`'s cut list over the range, drop audio elements from each interval's presence set, then re-merge adjacent intervals whose visual presence sets are equal. Each resulting run is a visual state.
- A tile is sampled at the first frame the project's frame grid actually paints inside the run: least n with floor(n*1000/fps) in [run start, run end).
- "A run containing no painted frame gets no tile and is named in `skipped` with reason `no-grid-frame`. Brevity alone never skips a run."
- Optional uniform "infill" tiles (behind a flag) are "first evicted under budget".
- "Every answer carries an unconditional structured disclosure — the rule, the per-tile provenance, what was skipped and why, the audio-only boundaries dropped, and a fixed `blind_to` enumeration — plus one sentence of prose." A later ADR (0097) requires the plain-text answer to carry that disclosure in full, not only the JSON form, so everything must render as prose, not only as enum values.
- ADR-0094's line on codes: "no finding code for a blind spot — a finding says something about *this document*, and the blindness is a property of the *rule*. A skipped run does earn one, because it is a fact about this document. Nothing here judges anything: `blind_to` describes the rule, `skipped` describes the document."
- `blind_to` currently enumerates, in ADR prose: change inside a run (a source clip's own cut, content motion); between keyframes and at them by default (easing); below the served tile width (fine detail dies ~140 px); across sheets (a relation between two tiles on different sheets); audio entirely; motion.
- A separate ADR (0103) permanently refuses crop-with-a-range and requires its refusal to name the two-step loop "sheet to locate, then `frame --crop --at <instant>` to look closely". One juror there asked that the sheet itself also point at `frame --crop --at`; that was left to be decided with the disclosure content.

An existing validate note/review code, `N-QUANTIZATION` (classes review and note, template "quantization at {fps} fps changes {changed} boundaries: {detail}."), reports what snapping the document's millisecond times to the frame grid *changes*. ADR-0006 escalates it to `review` for exactly two conditions: (1) an element that rounds out of existence, (2) a rounding that manufactures an overlap or gap — "both mean the rendered frames do not show what the document declares". It does NOT currently fire on a visual *state* that lies between two boundaries belonging to *different* elements on different tracks which both round onto the same frame: no element vanishes and no same-track overlap/gap is created, yet the combination the document declares for that span is never on screen. `validate` is therefore silent on the no-grid-frame condition today.

A fact measured in this session on the repo's only real project (a 65,216 ms, 25 fps fixture): `query` gives 46 intervals; after the audio filter and re-merge there are 18 visual runs, and **every one of the 18 contains a painted frame** — zero `no-grid-frame` instances. ADR-0094 §4 justified the rule with an example: "The fixture's 4 ms run at 56112–56116 contains no painted frame at 25 fps." But the boundary at 56112 is an audio element (`vo-quiz`) ending; the visual presence set is identical on both sides, so the re-merge absorbs it and the 4 ms run does not exist as a visual state. The example was computed before the re-merge that the same ADR mandates. The condition remains possible on other projects (two visual boundaries on different elements less than one frame period apart). A precedent: a sibling ticket recently found the fixture has zero instances of the keyframe population ADR-0094 §3 reasoned about, recorded that as a correction to §3 in its own ADR with a re-executable check script, and the correction was accepted.

# The questions

**Q4 — What code does a `no-grid-frame` skipped run raise?**
(a) Reuse `N-QUANTIZATION`, broadening its scope from elements to visual states, emitted at `review` (same escalation reasoning as "rounds out of existence"). `validate` would share the code; since validate does not detect this today, that gap is filed as separate work and `frame` states the fact before `validate` does until it lands.
(b) A new code of its own (e.g. `R-STATE-UNPAINTED` or a note-class equivalent), emitted only by `frame`'s range mode.
(c) Something else (say what), including "no finding at all, disclosure only" if you think ADR-0094 was wrong to give it one.
Also state the class (review / note) you would emit it at.

**Q5 — Where does `blind_to` live, and in what form?**
(a) As the sheet's analogue of the NOT CHECKED block: a fixed list of stable lowercase kebab tokens without a letter prefix (e.g. `inside-run`, `between-keyframes`, `below-tile-width`, `across-sheets`, `audio`, `motion`), each rendered by one fixed prose sentence, printed on every answer; not findings. The `below-tile-width` sentence names `frame --crop --at <instant>`.
(b) As `U-` (unchecked) class findings, one per blind spot, on every answer.
(c) Free prose only, no stable identifiers.
(d) Something else (say what).
Also say whether the other disclosure items — the count/list of dropped audio-only boundaries, the count of untiled keyframe change points, the served tile width and which budget rung produced it — are findings or disclosure, and whether the sheet should point at `frame --crop --at` (and if so, where).

**Q6 — The ADR-0094 §4 example.**
(a) Record the correction in the ADR this ticket writes (amending ADR-0094 §4's example: zero instances on the fixture, the 4 ms run is a pre-merge artifact), and commit a small re-executable script that recomputes the 46 → 18 runs and the zero count.
(b) Record it in the ticket's resolution comment only; don't amend the ADR or commit a check.
(c) Something else (say what) — including whether zero instances on the only fixture should change your answer to Q4, or should require a constructed test document with an instance before the code ships.

# Ballot format

Reply with exactly this block, and nothing outside it:

🗳️ **Juror <n>** (<the model backing you>) — **VOTE: Q4 <option + class>; Q5 <option>; Q6 <option>**

**Reasoning:** <why, per question>
**Trade-offs:** <what each choice costs, and why not the other options>

A juror may reject the framing outright: if the options are all bad or a question is wrong, saying so is a valid ballot.
