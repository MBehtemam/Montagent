# Jury: which repair class do `box`, `align`-on-image and `bold`/`weight` take?

Convened while implementing [#192](https://github.com/MBehtemam/Montaget/issues/192),
which had to place three retired spellings that no ADR classifies. The question is
[#228](https://github.com/MBehtemam/Montaget/issues/228). This directory is the evidence
that ticket rests on.

**Four jurors, four models — Sonnet, Haiku, Fable, and an adversarial Opus — each answering
the same six questions, blind to each other.** Each was pinned to commit `aef1cdcf` and
told not to read `crates/montaget-core/src/checks/`, issues #192/#228 or PR #229, so that
none could review the implementing agent's conclusion instead of the artifact. All four
reported the contamination check clean. The Opus juror was briefed to **default to
refuted**: predict the consensus, then argue against it.

The ballots are in [`ballots.md`](ballots.md), verbatim.

## The verdicts

| Question | Opus (adversarial) | Haiku | Fable | Sonnet | Verdict |
| --- | --- | --- | --- | --- | --- |
| `box: [x,y,w,h]` | advise | advise | advise | advise | **advise, 4–0** |
| `box: "card-05"` | refuse | refuse | advise | refuse | refuse, 3–1 |
| `align` on an image | refuse | refuse | refuse | refuse | **refuse, 4–0** |
| `bold` / `weight` | refuse | refuse | refuse | refuse | **refuse, 4–0** |
| Does ADR-0043's binary have a gap? | yes | no | yes | yes | gap, 3–1 |
| Is ADR-0068's "migrated by arithmetic script" a classification? | no | no | no | no | **historical, 4–0** |

## What the unanimity concealed

**The 4–0 on `box: [x,y,w,h]` splits 2–2 the moment you ask for the repair value**, and
two of the four are wrong in a way `validate` exists to prevent.

`migrate.py` is the historical migration and settles it
([lines 97–108](../../sample-project-migration/migrate.py)):

```python
x, y, w, h = e.pop("box")
out["x"], out["y"], out["origin"] = x, y, "top-left"
if t == "image":
    sw, sh = SRC_DIMS[out["source"]]
    dw, dh = cover(sw, sh, w, h)          # the drawn rect, ADR-0012
    out["width"], out["height"] = dw, dh
    ...
    out["clip"] = [x, y, w, h]            # the aperture the old box was doing silently
```

On an **image**, the old `box` was the *aperture*, not the drawn rect: `width`/`height`
come from `cover()` against the probed source, and the box itself becomes `clip`. Opus and
Fable both derived this. Haiku and Sonnet both answered `{"width": box[2], "height":
box[3]}` — which on the fixture's own `photo-06` writes a height of 1300 where the drawn
rect is 1912, drops `clip`, and silently changes which band of the source is on screen.
That is the gravity failure mode exactly: the right class, then a confident wrong repair.

On a **rect**, the same key needs no probe — `out["width"], out["height"] = w, h`. One
spelling, two repairs, one of which reads the media on disk.

**Consequence for the implementation:** the advise repair for `box` on an image is not
computable by `validate` today, because probing is [#190](https://github.com/MBehtemam/Montaget/issues/190)
and has not landed. A check that advises here before then would have to invent the source
dimensions or omit `clip`.

## The premise defect the adversarial juror found

**ADR-0043's stated ground for refusing `gravity` fails ADR-0043's own test.** The advise
arm admits three inputs — *"fully determined by the document, the media on disk, and
published rendering semantics"* — while the Why section justifies the refusal on the
ground that the deciding fact *"is **not in the document**"*. Source pixel dimensions are
on the media on disk, and `migrate.py`'s `SRC_DIMS` table is those dimensions, read by
`ffprobe`. Applied literally, ADR-0043 classifies its own founding example as advise.
[ADR-0068](../../../adr/0068-the-bare-mask-key-retires-masks-are-effects-members.md)
repeats the reasoning verbatim.

The refusal may still be right on a different ground — that the fork is *which of two
inconsistent statements the author meant*, not a missing measurement — but that ground is
not the one written down, and every classification downstream inherits the confusion.
Found by one juror of four, unprompted, and verified here by script.

## The gaps the jurors located

Three jurors said the binary has a gap; no two put it in the same place, which is stronger
evidence of a real defect than agreement on a verdict would be.

- **Opus and Fable — `bold` is determined but not writable.** The fix ("a different weight
  is a different file") is fully nameable without knowing intent, yet it is not a value
  that can be written into the document, so `"none"` is its only legal encoding — and
  `"none"` instructs the agent to *stop and surface to a human*, which is wrong counsel
  for a fix the agent is the right actor to perform. ADR-0016 already names the category:
  *"authoring an asset is something an agent can do and a program categorically cannot."*
- **Fable — the `contain` edge.** ADR-0043's Consequences bullet (*"an arithmetic
  retirement … is advise-class and states the fix"*) and its per-check uniformity rule
  contradict each other where the old rect's position inside the box was never published.
- **Sonnet — granularity is defined per *check*, but `box` is one spelling with two
  repairs** of different classes, so "per check, never per instance" needs a definition of
  what counts as one check.
- **Sonnet — transitive retirement.** `align` → `gravity` → `x`/`y`/`origin`/`clip`.
  ADR-0016 requires the error to name its replacement but never says what happens when the
  named replacement is itself later retired. `CONTEXT.md` names the final live replacement
  de facto; no ADR states the rule.

## Corrections to the brief itself

The convening brief said `bold: true` appears *on a text run*. ADR-0007 says the real
project file carries `"weight": "bold"` **on all 22 text elements**, and `migrate.py`
asserts it at element level (`e.pop("weight") == "bold"`). One juror caught the slip. The
check must match both positions, and its fixtures should exercise the element-level
spelling the only real corpus actually used.
