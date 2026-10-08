# Question: the ducking write tool

Ticket [#804](https://github.com/MBehtemam/Montagent/issues/804) on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)). Sent blind and identical to three
jurors dispatched in parallel (Opus 5.5, Sonnet 5.5, Fable 5.1), with no tools and no
recommendation in the packet. One ballot per sub-question.

## Context given to every juror (facts, not opinions)

Montagent is a Rust video editor operated by AI agents (over MCP and a CLI). A project is a closed JSON document (unknown keys are schema errors). Agents edit it by exact-string replace and read it through tools: `validate` (findings with a code, a class of error or review, and one repair field per code), `query --at`, `timeline`, `compare`, `shift`, `render`, `verify`, `frame`. Rules every capability keeps (ADR-0145): literal values (no expressions, no references to other values), closed vocabulary, checkable by `validate` from the file alone, editable by exact-string replace (a derived thing is changed through the literal that generates it). Times are absolute integer milliseconds. Elements are a flat list.

**The decision's history (ADR-0055).** Automatic ducking (a music bed dropping in level while narration plays) was deferred because it would make one element's level a live function of other elements' timing — a relationship read off the document's structure, which the format forbids. The author hand-writes the dip as ordinary `volume` keyframes (`{t,v,ease}` records; `volume` is a linear multiplier, 1 = source level). ADR-0055 named one future door, "an MCP write tool that computes and writes the keyframes into the document — never a live relational field", and held it back for lack of evidence that hand-authoring was costly. Its output must be ordinary `volume` keyframes; there is no live ducking field.

**What exists today.** A shipped skill script, `skills/montagent-footage/scripts/captions.py` (Python, stdlib only, not part of the binary), already ducks: its spec takes `"duck": {"id": "bed", "under": 0.18, "over": 0.5, "end": 0.85, "ramp": 200, "lead": 100, "join": 600, "fade": 1000}` and rewrites the named audio element's whole `volume` into keyframes: `under` while the voice speaks, `over` in pauses of at least `join` ms, `end` after the last word, ramps of `ramp` ms starting `lead` ms before the voice, and a fade to 0 over the last `fade` ms ending on the element's last drawn frame. It reads a words file (`[{"word","start","end"}]` in ms) that comes from an external aligner (word alignment is external to Montagent; `validate` and `compare` catch drift). It writes each held level as two equal keyframes, which `validate` reports as the review `R-EASE-INERT`. Running it again replaces what it wrote. The footage skill's guidance recommends levels (voice at 1.0; music 0.15–0.2 while speaking, 0.4–0.6 in pauses; pauses under 600 ms stay down so the music does not pump; ramps 150–250 ms starting 100 ms early; back near full after the last word).

**Tool-surface rules (ADR-0011, ADR-0083).** The MCP surface has eleven tools: compare, create_project, frame, measure, preview, query, render, shift, validate, verify, where. Only `create_project` and `shift` write. The write-tool invariant binds the MCP surface: "A tool that writes may only take a complete element, as a schema-shaped object. No tool takes a field name or an element id." Every write tool re-emits the whole file in canonical form and returns the new state's findings, never `ok`. CLI-only verbs exist for rarely used operations; the tool counts are asserted in tests, so adding a verb is a deliberate act. A skill script is outside both and may take ids freely.

**Audio decisions already merged.** The per-element chain is `atrim → atempo → aloop → [audio_effects] → volume → [pan] → [transition gain] → adelay → amix`; the effects list runs before `volume` so a later compressor cannot push a duck back up (ADR-0169). New level keys are in dB with the unit in the key name (`gain_db`, `ceiling_dbtp`), and `volume` stays the one linear level (ADR-0170). A keyframed `volume` is heard on the sample its instant names (ADR-0175). A transition may carry an audio crossfade over its window, and `validate` raises the review `R-TRANSITION-VOLUME-STACK` when a bridged element's `volume` changes level inside the window (ADR-0176). Every audio capability's ADR names a measured acceptance check run by a repo test at a stated tolerance, never a byte hash across builds (ADR-0173). A derived value the file cannot show the origin of is a provenance gap; ADR-0037 and ADR-0086 chose no computing tool for derived instants, and recorded-intent follows one pattern. The file records only the keyframe residue of a duck; it never records that a duck was applied.

**The map's workflow.** The lead workflow is narrated video over a music bed with TTS narration; the ducking tool is in the first wave. The decision is whether the tool enters now and in what shape. It builds nothing.

## How to judge

Judge every question by how an **AI agent would actually work with it**: authoring a new project, editing by exact-string replace, reading it back through `validate`/`query`/`timeline`/`compare`/`shift`, and recovering from an error. Prefer: discoverable, a small edit footprint, hard to misuse, loud with a repair rather than silent, no silent change to existing projects, and a result that survives the voice being retimed.

## Sub-questions

**Q-1 (does it enter now, and as what).** Options: (a) it stays a skill script, generalised out of `captions.py` into its own script with the same shape, and the format is untouched; (b) a CLI-only verb in the binary (e.g. `montagent duck`); (c) a new MCP write tool; (d) nothing new now: keep `captions.py`'s duck as it is, document it, and defer the tool again until there is more evidence. If (c), say how it satisfies the write-tool invariant.

**Q-2 (what it reads).** Where the voice's timing comes from: (a) an external word-timings file, as today; (b) the voice element's own `start`/`end` only (coarse, no pauses); (c) silence detection on the voice's audio source (no external file); (d) an explicit list of spans the author writes into the call. Say which, or which mix, and what the tool does when the voice has no timings.

**Q-3 (names and units).** How it names the bed and the voice (element ids; one bed under one voice, or several), and how depth, timing and levels are spelled: linear `volume` levels as today (`under: 0.18`), or dB (`under_db`, `over_db`) converted to linear when written, with `_ms` keys for ramp, lead and join. Say what is written into the file (always linear `volume` keyframes).

**Q-4 (merging with what is already on the bed).** The bed may already have `volume` keyframes (a manual fade-in, a fade-out). Options: (a) replace the whole `volume`, as `captions.py` does; (b) merge: write only inside the ducked stretches and keep everything outside; (c) refuse when the bed already has a keyframed `volume`, unless told to replace. Also say whether the final fade-out to 0 belongs to the duck tool or to a separate concern.

**Q-5 (what the file keeps, and the noise it makes).** Three parts: (i) provenance — the file keeps only keyframes: nothing more, or some record that a duck was applied and from what (and if a record, in what literal form); (ii) the equal-keyframe pairs the script writes trigger `R-EASE-INERT` — should the tool avoid emitting them, and how; (iii) a duck near a transition triggers `R-TRANSITION-VOLUME-STACK` — leave it, or have the tool avoid ducking inside a transition window. Also: when the voice is retimed, how does an agent know the duck is stale and refresh it?

**Q-6 (the measured check, ADR-0173).** What number the capability's ADR names and what it is measured on: the written keyframes (`query --at` reading under / over / end levels at named instants), the rendered PCM (the bed's level against its unducked level during speech and in pauses, ramp timing to the sample), or both; where the test runs (the repo's Rust render tests, or the skill's CI like `ci/test_prerender_still.py`); and the tolerance's origin.

## Ballot format required of every juror

For each of Q-1 … Q-6, exactly this block (a juror may reject a question's framing as their vote):

```
🗳️ **Juror <n>** (<the model backing you>) — **Q-<x> VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <what it costs, or why not the other options>
```
