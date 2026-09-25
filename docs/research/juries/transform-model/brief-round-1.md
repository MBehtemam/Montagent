# Brief: you are an agent that authors and edits Montagent projects

Montagent is a video editor whose project format is a single declarative JSON file,
designed to be authored and edited by an AI agent rather than dragged around in a GUI.
**You are that agent.** You are not a design reviewer, not a consultant, and not being
asked for your opinion of an architecture. You are the consumer who will live in this
format every day. Answer from what it is like to *use*, and back every claim with
something you actually did below.

Repo: `MBehtemam/Montagent` (read-only — see Rules).

## Read first

- `CONTEXT.md` — the domain glossary. Note the entries for Element, Track, Layer,
  Anchor, **Origin**, Run, Timeline range, Shift, and the Rejected terms (Scene, Clip,
  Asset, Anchor-for-text-positioning).
- `docs/adr/0001-flat-element-list.md` — why there are no scenes and no nesting.
- `docs/adr/0003-general-video-editor-not-channel-tooling.md` — the scope. **Read the
  guard**: the fixture is evidence a capability is *needed*, never evidence one is
  *unneeded*. "The fixture doesn't use rotation" is not an argument.
- `docs/adr/0005-absolute-integer-milliseconds.md` — the one absolute clock, and `shift`.
- `docs/adr/0006-...md` — especially "Checks that must not fire" and the two new checks.
- `docs/adr/0007-text-runs-literal-size-declared-fonts.md` — already ships `x`, `y`,
  `origin`, `align`, literal `size`, and a `box` an element must fit inside.
- `docs/adr/0011-tool-surface-reads-checks-renders.md` — the tool surface, and the
  section on `shift` and keyframes.
- The ticket: `gh issue view 21`
- The only project file anyone has ever written, hand-authored by an agent for issue #9:
  `git show prototype/sample-project-file:docs/research/prototypes/sample-project/en-halloween-decorating.montagent.json`
  and its `FINDINGS.md` and `JURY-EDIT-EXERCISE.md` beside it. Fields in it marked
  INVENTED are guesses at decisions that have never been made. It is a prototype, not a
  spec, and you may contradict it freely.

## What is already settled and is NOT yours to reopen

Absolute integer milliseconds on one project clock. Flat element list, no nesting, no
scenes. Tracks supply stacking, never timing. Keyframes exist and are on **transform
properties only** — never "every property is a function of time". **No expression
language, ever**: a keyframe list is inert data, code is not. Reference class is
CapCut/Premiere; After Effects is out of scope. Text size is a literal number.

## Do the work first (this is the part that matters)

Write your attempts into your own scratch file. Do not skip to the questions — the
questions are answered *by* what happens here.

1. **Author.** A lower-third title card that slides in from the left over 400 ms and
   fades up while it moves, sitting over a photo that is itself slowly zooming
   (Ken Burns). Write the actual JSON for both elements.
2. **Author.** A badge in the corner that rotates continuously for 3 s and then holds.
3. **Edit.** Insert 2000 ms of time at t=20000 in the #9 sample project — the `shift`
   operation. Produce the corrected JSON for `photo-06`, which carries
   `"scale": [[17472, 1.0], [32472, 1.08]]` and runs from 17472 to 30603. State exactly
   what the right answer is and why, and note where a plausible agent gets it wrong.
   (Two agents previously produced two different legal files here, both of which
   validate clean, and they are different videos.)
4. **Edit.** The project is `frame: {width: 1080, height: 1920}`. Retarget it to
   1920x1080. Say concretely what you would have to touch, and what the format could
   have done to make that better or whether it is inherently a re-layout.
5. **Read.** Without running anything, answer by eye from your own JSON: where is the
   title card, and how opaque is it, at t=6000? Note how hard that was.

## Then answer these five, each with a decision and the reason

1. **One positioning model or two?** Today images are placed with
   `"box": [0,0,1080,1300]` + `"fit": "cover"` + `"align": "top"`; text is placed with
   `"x": 540, "y": 1537` + `origin` + `align`. Separately, `box` currently names two
   different things: a literal `[x,y,w,h]` rect (#9's file) and *the id of another
   element you must fit inside* (ADR-0007). Do all visual elements share one placement
   model? What do you do about `box`?
2. **Units.** Absolute pixels in the project's frame space, or resolution-independent
   fractions (0..1)? Note `size: 88` and every rect are already literal pixels.
3. **The property set.** Which of `x`, `y`, `origin`, `scale`, `rotation`, `opacity`
   exist? Is `scale` one number or `[sx, sy]`? Is skew in or out?
4. **Shape and scope.** Flat fields on the element, or a nested `transform` object? Does
   an audio element carry the fields and ignore them, or is `"x"` on audio an error?
5. **Keyframe times: absolute, or relative to the element's start?** Absolute matches the
   one clock; relative survives `shift` without rewriting anything. Whichever you pick,
   state what `shift` must then do to keyframes. Also propose the **keyframe record
   shape** — the file currently uses positional pairs `[[3018, 1.0], [18018, 1.08]]` with
   no easing field at all — and say what easing should be (a closed named vocabulary, or
   cubic-bezier control points, or something else).

## Rules

- **Do not modify the repository.** No commits, no branches, no edits to tracked files,
  no `gh` writes. Read with `git show`. Write only inside your own scratch directory.
- Ground claims in the repo's own text and cite the file. Where you are guessing, say so.
- **Disagreeing with the settled ADRs where they genuinely bite is in scope** — say so
  loudly and separately, rather than quietly designing around them.
- Do not soften. If a choice is close, say it is close and name the tiebreaker. If a
  question is malformed, say that instead of answering it.

## Return

Your final message is the whole deliverable. Structure it exactly:

- **VERDICT** — a five-line table: Q1..Q5, your answer in under 12 words each.
- **What the work showed** — the findings from exercises 1-5 that actually drove the
  verdict. Concrete. Include the JSON where it makes the point.
- **Where I changed my mind** — anything you believed before exercise 3 and not after.
- **Closest call** — the one question that could have gone the other way, and the
  tiebreaker.
- **What I would need to overturn this** — per question where relevant.
