---
status: accepted
amends: 0003 ("After Effects is out of scope for now" is replaced: the reference class is where precedent is looked for first, every capability keeps the four invariants, and a capability with no precedent enters only by the entry test)
---

# CapCut and Premiere are where precedent is looked for first, not the edge of scope; a capability with no precedent in either enters by the entry test

[#667](https://github.com/MBehtemam/Montagent/issues/667), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663).
[ADR-0003](0003-general-video-editor-not-channel-tooling.md) named CapCut and Premiere as
the reference class and said *"After Effects is out of scope for now"*. That sentence was
then used as a scope boundary: a capability neither tool ships was refused for that reason
alone. The owner's direction has changed: *"we are pushing the bar and we are going to edit
the videos as well as generating animation."* Montagent stays a video editor and also
becomes a tool agents generate animation with. Some of what that needs (trim-path, a repeat
construct, spring easing) is in neither tool.

## The decision

"After Effects is out of scope for now" is replaced by the three rules below. Everything
else in ADR-0003 stands.

1. **The reference class stays CapCut and Premiere, read by mechanism, not by name.** It is
   the first place a design question looks for precedent. A mechanism confirmed in either
   tool's row of the precedent research
   ([#664](https://github.com/MBehtemam/Montagent/issues/664)) is a precedent. An
   unconfirmed CapCut row is not one, and does not put a capability outside the class when
   Premiere has the mechanism.
2. **Every capability keeps the four invariants,** wherever it comes from: literal values,
   closed vocabulary, checkable by `validate`, editable by exact-string replace. Precedent
   shows a mechanism exists; it does not show that our shape keeps them. The four tests
   below are normative, and a capability's ADR shows each one passing. A capability ADR
   cannot waive an invariant. A shape that fails one is reshaped until it passes, or ends as
   a recorded no. Changing an invariant or its test is its own ADR, argued for the whole
   format.
3. **A capability with no precedent in either tool enters only by the entry test:** the four
   invariants, which bind it like every capability, and in addition a rendered prototype the
   owner accepts. Acceptance is recorded as the prototype's commit pin, the linked render
   and the owner's quoted words. It is necessary, not sufficient: the capability's own ADR
   decides the shape and may still say no, recorded with its reason. A capability with a
   precedent needs no accepted prototype under this ADR.

**Precedent first.** Every capability ticket cites the capability's row in the precedent
research, starts from the precedent's shape where one exists, and names each departure.
Other tools (After Effects, Lottie, CSS) may be cited for the shape of a capability, never
as the reason it enters.

**This is a floor.** A map or ADR may demand more, and those that already do
([ADR-0086](0086-recorded-intent-is-one-pattern-and-the-time-axis-instantiates-it.md)'s
measured proof, the transform map's A/B evidence bar on
[#591](https://github.com/MBehtemam/Montagent/issues/591)) are unchanged.

## The four tests

| Invariant | Test | Passes | Fails |
| --- | --- | --- | --- |
| Literal values | Every value is a number, string, boolean, list or object written in the file. The renderer may apply fixed, schema-documented arithmetic (as keyframe interpolation already is), never an author-written expression or a reference to another value. | A keyframe list on `radius` | `"delay": "index * 80"` |
| Closed vocabulary | Every key and enum value is listed in the schema. A path may name media to decode, never a program to run. Lottie expressions and SVG scripts are programs. | `"blend": "multiply"` | `"shader": "glow.sksl"` |
| Checkable by `validate` | A malformed or out-of-range value is a finding from the file and its probed assets alone, without painting a frame. | A point list checked against bounds | A parameter whose legal range depends on rendered pixels |
| Exact-string replace | Every authored choice has its own string. A derived thing is changed through the literal that generates it, and the schema documents how to single one out. | An in-between value is singled out by adding a keyframe; a letter by splitting its run | A repeat with no documented way to change one copy |

The examples are illustrations of the tests, not rulings: no capability on the map is
decided by this table.

The fourth test is worded for what the format already does. An interpolated in-between value
and the third letter of a run have no string of their own, and both are still editable,
because the schema says how to single one out. A construct such as
`{"count": 5, "offset_ms": 80}` therefore passes literal values (its copies come from fixed
arithmetic, as in-betweens do) and is argued under the fourth test: its ADR must supply the
way to change one copy.

## What this does not decide

- **Transform-only keyframes:** ADR-0003's reason was After Effects' model. That reason no
  longer rules anything out by itself, so the rule is open to decision, and stays in force
  until the keyframable-properties ADR
  ([#669](https://github.com/MBehtemam/Montagent/issues/669)) decides it.
- **Expressions:** still refused, now because they fail the literal-values invariant.
- **The closed effect vocabulary:** unchanged.
  [ADR-0017](0017-closed-schema-no-escape-hatch.md)'s refusal of any open script, shader
  file or escape hatch stands.
- **Nesting:** still rejected by ADR-0003. Reopening it belongs to the transform map.
- **"CapCut and Premiere are flat-timeline tools":** not decided here. The group transform's
  ADR carries the erratum to this sentence.
- **Earlier rulings:** none is reversed. A ruling whose only reason was the reference class
  may be re-read by whoever owns it. Until it is re-read, it stands.
- **Shipped features:** not re-examined. The test applies to capabilities decided from now
  on, including a later change to a shipped feature.

## Why

**The reference class was doing two jobs.** It was a source of shapes agents already know,
and it was a fence. The first job is worth keeping: an agent that has seen Premiere's blend
modes writes `"blend": "multiply"` correctly the first time. The second job refused
capabilities for where they came from, not for what they would do to the file.

**The invariants are what the fence was protecting.** ADR-0003 refused After Effects' model
because *"reading the file would stop being enough to know what is on screen"*. That is a
claim about the file, and the four invariants state it directly. An agent reads the schema,
writes literals, edits by exact-string replace and trusts `validate`; a capability that keeps
all four costs the agent nothing new, whichever tool it came from.

**The prototype is the price of leaving the class.** Inside the class, two shipping editors
are evidence that a mechanism is wanted and usable. Outside it there is no such evidence, so
the capability shows a rendered result first.

## Considered and refused

- **A capability ADR may waive an invariant if it names which and why.** Refused. Each
  invariant would become "true except where an ADR says otherwise", which an agent would
  have to remember on every edit. With twelve capabilities queued, each with an appealing
  prototype, the waiver would become the normal route.
- **After Effects, Lottie or CSS as a reason to enter.** Refused: it widens the reference
  class by the side door and leaves the entry test with nothing to do.
- **An accepted prototype is sufficient.** Refused. A prototype shows the picture is wanted.
  It does not show the schema shape is right, that `validate` can check it, or that an agent
  can edit it.
- **Requiring a precedent in both tools.** Refused: CapCut publishes no reference manual, so
  most of its rows are unconfirmed, and ordinary editor features would fall outside.
- **Lifting the transform-only keyframe rule here.** Refused: it would decide the
  keyframable-properties ticket without the evidence that ticket is gathering.
- **Folding the "flat-timeline tools" erratum in here.** Refused: the transform map owns
  that sentence and holds the evidence on nesting.

## Consequences

- `CONTEXT.md` gains **Reference class** and **Entry test**.
- The transform map's parent-link exclusion rested on the reference class alone. It stands,
  and that map may re-read it.
- Every capability ticket on [#663](https://github.com/MBehtemam/Montagent/issues/663) cites
  its precedent row and shows the four tests passing in its ADR.
