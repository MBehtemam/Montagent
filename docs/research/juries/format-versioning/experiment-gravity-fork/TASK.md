# Task — repair a Montaget project file

You are the editing agent for **Montaget**, an agent-first video editor. A
Montaget project is a single declarative JSON file that is the source of truth
for a video: images, video clips and audio composed onto one absolute timeline.
You author and edit it with ordinary file tools.

The format has moved on since this project file was written, and the file no
longer validates. **Your job is to repair it so that it validates, without
changing the video it describes.**

## What you have

- `project.json` — the file to repair, in your own directory. Edit it in place.
- `validate-output.txt` — the current output of `montaget validate` on it.
- (in some sandboxes) `graveyard.txt` — an excerpt from the format's glossary.

## Facts about the format you will need

These are complete for this task; there is nothing else to look up.

- Every visual element is placed by `x`, `y`, `origin` and `width`/`height`,
  in absolute integer pixels on the project's frame. `origin` names which point
  of the element's own rect sits at (`x`,`y`) — `"top-left"` means the rect's
  top-left corner is at (`x`,`y`).
- `clip` is a **static rectangle in frame space**, `[x, y, w, h]`. Only the part
  of the element falling inside `clip` is visible. It is an aperture: it does not
  move or scale the element.
- `fit` is a claim about how the element's declared `width`/`height` were derived
  from the source's pixel dimensions. It never executes and the renderer never
  reads it; the declared rect is authoritative at render.
- Times are absolute integer milliseconds. `scale` keyframes are `{"t","v"}`.

## Rules

- **Do not consult any Montaget repository, ADR, spec or website.** Everything
  you need is in your own directory. If you find yourself wanting a document you
  do not have, write down what you wanted and why — that is a result.
- The repaired file must be valid JSON and must keep the format's writing
  convention: one element per line, elements sorted by `start`, stable key order.
- Do not reformat, re-indent or re-order anything you were not repairing.

## Deliverables, in your own directory

1. `project.json` — repaired, in place.
2. `REPORT.md` containing:
   - **What I changed**, element by element, and **why**.
   - **What I was unsure about** — every point where more than one repair looked
     legal, what they were, and how you chose. Be exhaustive here; this is the
     most valuable part of your report.
   - **What I wanted and did not have.**
   - **Confidence** that the video you produced is the video the author meant,
     and say plainly what you could not check.

Work alone. Do not look in any sibling directory.
