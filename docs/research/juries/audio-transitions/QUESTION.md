# Question: the transitions ticket, as an agent would work with it

Ticket [#803](https://github.com/MBehtemam/Montagent/issues/803) on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)). Sent blind and identical to three
jurors dispatched in parallel (Opus 5.5, Sonnet 5.5, Fable 5.1), with no tools and no
recommendation in the packet. One ballot per sub-question. An earlier court on the same ticket,
with fewer questions, is in [`../audio-first-wave/`](../audio-first-wave/BALLOTS.md) (its Q-F).

## Context given to every juror (facts, not opinions)

Montagent is a Rust video editor operated by AI agents (over MCP and a CLI). A project is a closed JSON document (unknown keys are schema errors). Agents edit it by exact-string replace and read it through tools: `validate` (findings with a code, a class of error or review, and one repair field per code), `query --at`, `timeline`, `compare`, `shift`, `render`, `verify`, `frame`. Rules every capability keeps (ADR-0145): literal values (no expressions, no references to other values), closed vocabulary, checkable by `validate` from the file alone, editable by exact-string replace (each authored choice has its own string; a derived thing is changed through the literal that generates it). Times are absolute integer milliseconds. Elements are a flat list; tracks are lanes that own nothing, and elements on one track may not overlap in time.

**Transitions today (ADR-0059, ADR-0150).** A transition is its own element: `{"type":"transition","id":"t1","kind":"crossfade"|"wipe"|"slide"|"push","from":"<id>","to":"<id>","start":ms,"end":ms,"direction":..,"ease":..}`. Its `start`/`end` must exactly equal the intersection of the two elements it bridges (so those two elements overlap in time and sit on separate tracks). `from`/`to` must name visual elements: `E-TRANSITION-REF-MISSING` fires on an id that names no visual element, so an `audio` element cannot be bridged today. Other codes: `E-TRANSITION-RANGE`, `E-TRANSITION-NO-OVERLAP`, `E-TRANSITION-REF-SELF`. A `video` element is visual and carries its own embedded audio. Today the audio mix ignores transitions entirely: each element's sound simply starts and ends at its own `start`/`end`, so a picture dissolve between two clips has a hard audio cut at each clip's edge, and two overlapping clips simply sum.

**Audio today.** The only audio control on `audio`/`video` elements is `volume`: a keyframeable linear multiplier (`{t,v,ease}` records; 0 silent, 1 source level). Fades are volume keyframes (no fade fields). Decided and merged: audio effects are an ordered `audio_effects` list on the element (ADR-0169) whose per-element render chain is `atrim → atempo → aloop → [audio_effects] → volume → [pan] → adelay → amix normalize=0`; the list comes before `volume` precisely so a later compressor cannot push a ducking dip back up; new level keys are in dB with the unit in the key name, `volume` stays the one linear level (ADR-0170); there is a top-level `master` stage (ADR-0172); a keyframed volume is heard on the sample its instant names (ADR-0175). A speed-ramped `video` must carry `volume: 0` (`E-REMAP-AUDIBLE`). `N-NO-AUDIO-STREAM` flags an element with no audio stream. Master loudness etc. are not in question here.

**Precedent.** Premiere has audio crossfades as objects separate from video transitions: Constant Power (the default), Constant Gain and Exponential Fade; applied at an edit point between two audio clips. CapCut: no confirmed audio-crossfade mechanism. **ffmpeg:** `afade` is sample-exact with curves `qsin` (constant power), `tri` (constant gain), `exp`; two fade chains summed by the existing `amix normalize=0` replace `acrossfade`, which ignores runtime commands. Under `normalize=0` summation, a constant-gain crossfade of two uncorrelated signals dips about 3 dB at the midpoint; constant power holds level.

Planning-map scope: this ticket decides whether a visual transition carries the audio across its cut, or audio crossfades get their own mechanism. It builds nothing.

## How to judge

Judge every question below by how an **AI agent would actually work with it**: authoring a new project, editing an existing one by exact-string replace, reading it back through `validate`/`query`/`timeline`/`compare`/`shift`, and recovering from an error. Prefer: discoverable from the schema, a small edit footprint, hard to misuse, loud with a repair rather than silent, and no silent change to the sound of projects that already exist.

## Sub-questions

**Q-1 (who owns the audio window).** For a video→video transition, the window already exists on the `transition` element. Options: (a) the transition itself carries the audio behaviour in a field; (b) a separate audio element names the two audio-bearing elements and carries its own window (the author writes the window twice, and `validate` keeps the two in step).

**Q-2 (curves).** Options: (a) a closed, author-selectable set of all three (constant power, constant gain, exponential); (b) two (constant power, constant gain); (c) one fixed curve (constant power), not selectable. If selectable, say the key name and the default.

**Q-3 (position in the chain).** Where does the crossfade gain apply relative to the element's chain: (a) before `audio_effects`; (b) after `volume` (and pan), at the end of the element's own chain; (c) elsewhere (say where).

**Q-4 (default for existing and new visual transitions).** For a visual transition whose author writes nothing about audio: (a) absent means today's behaviour (a hard audio cut), so every existing project renders byte-identically, and guidance tells agents to write the field; (b) absent means an audio crossfade (constant power), which changes the sound of every existing project that uses transitions; (c) the field is required on every transition (a schema break, so every existing project must be edited). Also say what an agent writing a brand-new transition would do under your choice.

**Q-5 (an audio-only crossfade, e.g. one music bed into another, no picture).** A transition cannot name an `audio` element today. Options: (a) widen the existing `transition` element: a new `kind` (e.g. `audio_crossfade`) whose `from`/`to` may name `audio` or audio-bearing `video` elements, with the visual kinds still refusing audio-only refs; (b) a separate new element type (e.g. `audio_transition`) with its own `from`/`to`/window; (c) nothing new: an audio-only cut is authored as volume keyframes on two overlapped elements, as today. Also say how the key that selects the curve is named, and whether one key name is used for both visual transitions and the audio-only form or two.

**Q-6 (stacking with the author's own fades).** The author may already have written volume keyframes (a manual fade or a duck) inside the transition window, so the crossfade multiplies with them. Options for what `validate` does: (a) nothing; (b) a `review` finding when a volume keyframe changes the level inside the window; (c) an `error`. Also say what should happen when a bridged element has no audio stream, or is a speed-ramped `video` carrying `volume: 0` (nothing to fade).

**Q-7 (reading it back).** What should the reading tools report so an agent can see the audio is crossfading and change it correctly? Options to weigh: `query --at` at an instant inside the window (which elements are sounding, at what resolved gain, and from which transition); `timeline` (does a transition row show its audio curve); `compare`/`shift` (do they treat the audio field as part of the transition, and does `shift` keep the window and the bridged elements consistent). Name the smallest set that stops an agent from being misled, and say what must NOT be added.

## Ballot format required of every juror

For each of Q-1 … Q-7, exactly this block (a juror may reject a question's framing as their vote):

```
🗳️ **Juror <n>** (<the model backing you>) — **Q-<x> VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <what it costs, or why not the other options>
```
