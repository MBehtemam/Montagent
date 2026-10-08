---
status: accepted
amends: 0003 ("Nesting stays rejected": the reason given for it is wrong by mechanism)
---

# CapCut and Premiere nest with a transform, and ADR-0001 refuses the clock, not the shared space

[#632](https://github.com/MBehtemam/Montagent/issues/632), on the map
[#591](https://github.com/MBehtemam/Montagent/issues/591). The map's Destination owes two
records whatever the group transform's ruling is: an erratum to ADR-0003, and a statement of
what ADR-0001's rejection of `scene` still covers.
[ADR-0140](0140-an-image-changes-over-time-by-a-list-of-timed-swaps-over-its-base-source.md)
left both to this ticket. A court on #632 put them in their own ADR, so that a correction to the
reference class does not depend on a ruling that its prototype and score can still overturn.

## 1. The erratum

[ADR-0003](0003-general-video-editor-not-channel-tooling.md) says:

> **Nesting stays rejected.** After Effects' central structure is the precomp, a composition
> inside a composition. With After Effects out of scope, ADR-0001's rejection of `scene` is not
> under pressure: CapCut and Premiere are flat-timeline tools.

The last clause is wrong by mechanism. A Premiere nested sequence is a sequence placed as a clip
in another sequence, and it carries that clip's own Motion transform. A CapCut compound clip
does the same for the clips it holds. Both group children under a parent transform.
[Is a rigged cut-out character a CapCut/Premiere job?](https://github.com/MBehtemam/Montagent/issues/594)
read them this way and found the container in the reference class. It found the parent link
between siblings outside it, because only After Effects has one.

So nesting is not outside the reference class. Whether a nest enters the format is a separate
decision with its own evidence. This ADR does not make it.

## 2. What ADR-0001's rejection of `scene` still covers

[ADR-0001](0001-flat-element-list.md) splits a grouping noun into two jobs: "giving related
elements a shared identity, and giving them a shared clock." It keeps the identity as the
render-inert `group` string. It refuses the clock, because of a failure it observed: "an
element's `start` is relative to its scene, a model computes it against the movie instead, and
the render succeeds and is silently wrong."

- **The clock refusal stands, unchanged.** No element's time is relative to another element or
  to a container. Every `start`, `end` and keyframe `t` is timeline milliseconds.
- **ADR-0001 never argued against a shared space.** It refused the container only because
  "nesting implies a local clock whether or not one exists". That is a claim about how readers
  read a container. It is not a claim about transforms. ADR-0003's "Nesting stays rejected"
  stretched it to cover space, and did so on the flat-timeline premise that §1 corrects.
- **A container is not ruled out by this ADR, and not admitted either.** A proposal for one has
  to answer ADR-0001's inference: a reader will treat its children's times as relative. "We will
  tell readers not to" does not count as an answer. The answer has to be a rule that `validate`
  can enforce.

## Consequences

- ADR-0003 carries a banner pointing here. Its After Effects reasoning still stands as amended by
  [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md).
- ADR-0001 is unchanged, and so is its banner, which already says "no scene, no local clock …
  stand unchanged".
- The group transform's own ruling (#632) cites this ADR. That ruling is the space-only
  container, provisionally a `nest`, and its ADR follows its prototype and A/B score. If the score
  ends with "the format stays flat", this ADR still stands, because it records a fact about the
  reference class, not a design.
