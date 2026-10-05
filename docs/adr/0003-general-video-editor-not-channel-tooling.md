---
status: accepted
---

# Montagent is a general video editor; the channel is a fixture

> **Amended by [ADR-0145](0145-capcut-and-premiere-are-where-precedent-is-looked-for-first-and-a-capability-with-none-enters-by-the-entry-test.md)**: "After Effects is out of scope for now" is replaced. The
> reference class is where precedent is looked for first, not the edge of scope. Every
> capability keeps the four invariants, and one with no precedent in CapCut or Premiere
> enters only by the entry test. The transform-only keyframe rule below stays in force until
> its own ADR decides it.

Montagent is a **general-purpose, agent-first video editor**. It will be open
source and run by people other than its author. Its primitives are shaped by what
a video editor must be able to express — never by what any one project happens to
produce today.

The `youtube_language_learning` channel, whose halloween short is checked in under
`fixtures/`, is **test data and a regression guard**. It is evidence that the
primitives are sufficient for real published work. It has no authority over what
Montagent must do.

The reference class is **CapCut and Premiere**: a video editor that edits video
and audio, places images, scales and transforms them, and draws shapes, text,
effects and text effects. **After Effects is out of scope for now** — see
*Consequences*.

## Why

**The project was documented as two different products.** The map's destination
said "a specification for Montagent, an agent-first video editor"; two paragraphs
later its domain note said "video composition tooling for an AI-driven YouTube
channel (vertical 9:16 language-teaching shorts)". Those are not the same scope,
and every working session loaded the second one.

**The narrow framing was the one that won, and it produced wrong answers.** The
ticket asking whether right-to-left script support was needed was written to ask
whether RTL was "plausibly in *this channel's* future" — so it was answered by
reading the channel's configuration and finding six Latin-script channels. Under
the general reading that evidence is close to irrelevant: a video editor that
cannot set Arabic, Hebrew or Persian text is Latin-only by construction, whatever
any one channel publishes. Same ticket, same facts, opposite answer, purely from
framing.

**Open source removes the last defence of the narrow reading.** If Montagent had
one operator, "we do not need it" would at least be a coherent scope argument.
Strangers will run this, feed it their own assets, and drive it with their own
agents. What one channel currently publishes says nothing about what the tool
must accept.

**The drift recurred because the artifact carried it, not because any one session
was careless.** Correcting a session fixes one session; correcting the document
fixes every session after it. Hence the guard below.

## The guard

> When a design question is answered by consulting the channel's configuration,
> content or output rather than the format's requirements, the drift has recurred.

The asymmetry is the load-bearing half, and it is deliberate:

**The channel is evidence that a capability is _needed_. It is never evidence
that one is _unneeded_.**

That the fixture uses Latin text at 9:16 with still images proves those must work.
It proves nothing whatever about Arabic, 16:9, or video clips.

## Consequences

**General obliges primitives, not features.** Version 1 implements what it
implements. But no primitive may be *shaped* such that a general need becomes hard
to add later — a text element must not be structurally single-script, a project
must not assume 9:16 — even where v1 only tests Latin at 9:16. This is the same
constraint already carried for templating.

**The fixture is a regression target, compared structurally.** The target MP4 came
back through YouTube's re-encoder, so pixel-exact or byte-exact comparison is
unsatisfiable by construction. "Matches" means duration, frame size, which element
is on screen when, and audio alignment.

**"Bilingual subtitle" is not a concept in the format.** A target/bridge string
pair is language-teaching vocabulary. Montagent has *text*; that two text elements
form a translation pair is meaning the agent holds and a `group` may label. It is
not a field, and it does not appear in the schema that every unrelated user reads.

**Effects are a closed, named vocabulary — not an effect stack or a plugin
system.** This follows from CapCut/Premiere rather than After Effects. A closed
list published in the schema is discoverable by an agent from the schema alone; an
open plugin architecture never is.

**Animation is keyframes on transform properties, not on everything.** After
Effects' model — every property a function of time, plus an expression language —
would put real pressure on the inert-data principle, because reading the file
would stop being enough to know what is on screen. A keyframe list is still data
you can read. An expression is not. Deferring After Effects defers that fight.

**Nesting stays rejected.** After Effects' central structure is the precomp, a
composition inside a composition. With After Effects out of scope, ADR-0001's
rejection of `scene` is not under pressure: CapCut and Premiere are flat-timeline
tools.

**The core model is incomplete, not wrong.** ADR-0001 and ADR-0002 settled a
*sequencing* model: what is on screen, and when. Nothing in them is contradicted
here. But `CONTEXT.md` and both ADRs contain no mention of transform, scale,
rotation, opacity, effects, transitions or keyframes — so the model cannot yet say
*where*, *how big*, or *how it changes*. A transform model, an effect model, an
animation model, a text primitive and a shape primitive are all still open.

**Two research documents were scored against the narrow criteria.** The renderer
survey wrote the channel into its evaluation criteria ("Text and typography —
bilingual subtitle text"; "Target format throughout: 1080x1920, 30fps"), and the
MCP SDK host survey inherited it, concluding that full-bidi support is "only
load-bearing if the channel goes RTL or CJK". Under this ADR that conditional
resolves: it **is** load-bearing. Their *facts* remain good; their *weighting* does
not, and the host decision must be re-scored against general criteria.
