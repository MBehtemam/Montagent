# The Judge's read

Written by the session that convened the court, after the ballots in [`BALLOTS.md`](BALLOTS.md). It is the
Judge's own view, not a juror's, and it decided nothing. Only Q-F was still open when this record
landed; the rest were settled by ADR-0169, ADR-0170, ADR-0172 and ADR-0173.

- **Q-A:** all three agree in substance. Carry on with "narrated video over a music bed" as the working
  assumption, because the first decisions (chain shape, units, master stage) apply to every workflow,
  and ask the owner one question.
- **Q-B:** all three want an ordered `audio_effects` list, with `volume` applied after the chain, a kind
  allowed to appear twice, and `video` elements carrying the same fields. They split on pan (two want a
  flat field, one a list entry) and on an `id` per entry (two want one, one doesn't). The Judge sided
  with the flat `pan` and leaned towards no `id`; it was a close call. ADR-0169 decided it.
- **Q-C:** all three effectively say dB, with the unit in every key name and `volume` staying the only
  linear level. They split only on the pan range (−1..+1 against −100..+100); the Judge took −1..+1.
  ADR-0170 decided it.
- **Q-D:** the real split. Juror 1: the renderer measures the mix and applies the gain. Juror 2: the agent
  writes a literal gain. Juror 3: a literal gain plus a declared target `verify` checks. The Judge sided
  with Juror 3, with the master stage limited to a gain, a limiter and the loudness target. ADR-0172
  decided it.
- **Q-E:** all three agree on both checks, a repo test per capability plus `verify` on the deliverable,
  measured in Rust. They split on whether A/B audio clips are committed; the Judge favoured committing
  short clips, per `docs/agents/domain.md`. ADR-0173 decided it.
- **Q-F (live):** the second real split.
  - Juror 1: a separate audio transition, as in Premiere.
  - Juror 2: visual transitions carry the audio by default, which changes existing renders.
  - Juror 3: an `audio` field on the transition that defaults to `cut`, plus an audio-only transition
    kind.

  The Judge sided with **Juror 3**: it avoids writing the same window twice (a separate mechanism makes the
  agent copy the window by hand onto a second object), keeps every existing project rendering
  byte-identically, and reuses the existing `transition` element rather than inventing a second one. This
  is a recommendation to whoever works [#803](https://github.com/MBehtemam/Montagent/issues/803), not a
  decision.
