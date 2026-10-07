# Ballots: what unit an audio level is written in

Two courts on [Are new audio levels written in dB or as linear multipliers?](https://github.com/MBehtemam/Montagent/issues/800).
Each had three jurors (Opus, Sonnet, Fable), dispatched in parallel. Each juror got the same
packet blind, with none of the Judge's recommendations in it. The packets are
[`QUESTION-ROUND-1.md`](QUESTION-ROUND-1.md) and [`QUESTION-ROUND-2.md`](QUESTION-ROUND-2.md).
Round 2's packet stated the bounding rule inside option (b), which favoured it in framing. Each
juror's reasoning below is condensed from their full ballot, which was reported verbatim to the
owner in session. Votes are exact. Both times the owner ruled with the Judge's read. The decision
is [ADR-0170](../../../adr/0170-audio-levels-are-written-in-db-and-their-keys-say-so-volume-stays-the-one-linear-level.md).

## Round 1 (Q1 to Q3)

| | Q1 level unit | Q2 pan | Q3 other units | Q3 durations |
|---|---|---|---|---|
| Juror 1 (Opus) | (b) dB; `volume` linear | (b) −1..+1 | accept | fractional |
| Juror 2 (Sonnet) | (b) dB; `volume` linear | (b) −1..+1 | accept | fractional |
| Juror 3 (Fable) | (b) dB; `volume` linear | (b) −1..+1 | accept | fractional |

### Juror 1 (Opus)

**Q1: (b).** ADR-0055's argument is about a keyframable fader that fades and ducking write to; it
doesn't carry over to thresholds, ceilings, EQ boosts or targets, which are native to dB. A linear
threshold of 0.0625 or an EQ boost of 1.995 is the "log arithmetic for the reader" ADR-0055 wanted
to avoid, moved to the other side. Within an effect, 0 dB means "no change", so the clash stays
local and reads correctly.
*Trade-offs:* two level scales, so the rule must be written down. (a) makes loudness targets
meaningless; (c) reopens a settled ADR and breaks fades and ducking.

**Q2: (b).** Pan is a unitless position and belongs with the normalised scalars; ±100 is the
percentage ADR-0055 rejected.
*Trade-offs:* a named departure; an agent writing `50` fails `validate` loudly.

**Q3: accept; fractional.** ADR-0005 governs timeline instants; attack and release are durations
inside a processor at audio rate, and 0.1–1 ms attacks need decimals.
*Trade-offs:* non-integer ms beside integer `start`/`end` must be documented.

### Juror 2 (Sonnet)

**Q1: (b).** A level uses the unit its domain and Premiere use. A threshold of 0.125 is much harder
to read than −18 dB, and linear is badly non-uniform near 0 dBFS. For EQ gain, 0 dB as "no change"
is the natural identity; a threshold has none. `volume` is the only keyframed, fade-driven level.
Key names should make clear these are not `volume`-style multipliers.
*Trade-offs:* agents must learn the split. (a) throws away precedent; (c) reopens a reasoned ADR
and breaks fades and ducking; a dB `volume` sibling is two spellings.

**Q2: (b).** Matches the format's normalised scalars and ADR-0055's stance on percentages; ±100 is
a UI slider convention, not a unit. Sign convention kept.
*Trade-offs:* rescale by 100 from Premiere.

**Q3: accept; fractional.** Native units where precedent exists, normalised scalars for
dimensionless quantities; fractional semitones remove a second cents field. Integer ms would force
fast attacks to 0 or 1.
*Trade-offs:* two duration conventions need a one-line doc rule.

### Juror 3 (Fable)

**Q1: (b).** ADR-0055's argument was specific to `volume`. The rule: *a level parameter takes the
unit its precedent uses, unless it is a keyframable gain whose identity must read as "on", in
which case it is linear — and `volume` is the only such field.* A linear loudness target has no
precedent and would make `validate` ranges unreadable.
*Trade-offs:* two level scales, stated next to `volume`. (Judge's note: this ballot's premise that
these parameters "are not keyframed" is undecided on the map; the rule stands without it.)

**Q2: (b).** The same argument ADR-0055 used against 0–100 rejects ±100; the sign convention is the
only meaningful part of the precedent.
*Trade-offs:* `pan: 50` fails `validate` immediately.

**Q3: accept; fractional.** A compressor attack is a filter coefficient, not a timeline instant;
nothing shifts or snaps it. Integer ms would silently clamp fast attacks.
*Trade-offs:* document so an agent doesn't try `start: 12.5`; pan and mix departures share one
justification and should be recorded together.

## Round 2 (Q4)

| | Q4 key names |
|---|---|
| Juror 1 (Opus) | (b) suffix only log levels |
| Juror 2 (Sonnet) | (b) suffix only log levels |
| Juror 3 (Fable) | (b) suffix only log levels |

### Juror 1 (Opus)

**(b).** `"gain": 0.5` meant as "half" is a legal +0.5 dB; a range check can't fix an in-range
value. The suffix is the only defence where the agent is looking when it writes. `_dbtp` vs `_db`
names different measurements. And a closed vocabulary rejects a bare `gain` where `gain_db` is
expected, so the mistake fails loudly.
*Trade-offs:* (a) leaves the silent failure; (c) adds a second naming dialect and dilutes what a
suffix signals.

### Juror 2 (Sonnet)

**(b).** Only where two scales coexist does a unit-blind key invite a real mistake. Docs and range
checks catch it after the fact, or not at all for small positive dB gains. The rule is principled
and checkable, and keeps bare names wherever the unit is unambiguous.
*Trade-offs:* (a) relies on docs an exact-string editor may never read; (c) adds noise without
preventing any further error.

### Juror 3 (Fable)

**(b).** The bare-name convention works because every bare key had one scale; audio levels break
that on purpose. The rule predicts future cases and matches Premiere's labels.
*Trade-offs:* (a)'s "rely on docs" is the weakest defence for agents that learn from examples;
(c) moves the inconsistency rather than removing it. The rule must be written down so future
authors don't suffix on impulse.
