# Jury: how the range mode appears on the tool surface

Three jurors, three different models, put to
[#401](https://github.com/MBehtemam/Montagent/issues/401) on the map
[#395](https://github.com/MBehtemam/Montagent/issues/395). Resolved as
[ADR-0097](../../../adr/0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md).

Each juror received `BRIEF.md` **verbatim and identical** — no assigned stance, no
persona, no sight of each other's ballots, dispatched in parallel. Ballots are
recorded here unedited.

| juror | model | ballot |
| --- | --- | --- |
| 1 | Opus 5 | [opus-5.md](opus-5.md) |
| 2 | Sonnet 5 | [sonnet-5.md](sonnet-5.md) |
| 3 | Fable 5.1 | [fable-5-1.md](fable-5-1.md) |

Unlike the [instant-selection panel](../contact-sheet-instant-selection/README.md), this
brief was put as **seven sub-questions in one ballot** rather than one question. That is
recorded because it is a methodological difference, not a neutral one: it lets a juror
trade one answer against another (Juror 1 did, explicitly accepting Q7's (b) *"as a
friendly amendment rather than an alternative"*), and it gives less depth per question
than a dedicated panel would.

## What the panel settled without dissent — Q1 through Q4

- **`--from`/`--to`, half-open `[from, to)`, both required.** All three rejected the
  trial agents' own `--per-state` / `--each-cut` on the same reasoning, which no juror was
  handed: those spellings name a *selection rule*, and ADR-0094 has already fixed that
  rule as the only one the verb has, so the flag would advertise a sibling that does not
  exist. All three also rejected a defaulted `--to`, and two reached ADR-0095's own risk
  note to do it — an implied "to the end" would make the first call on any real project a
  refusal on a range the caller never typed.
- **The `--from`/`--to` pair *is* the mode** — no `--sheet` flag. All three cited
  `measure`'s exclusive argument groups, and all three placed the refusal in the verb
  rather than in `clap`, which the existing CLI comment already requires.
- **Both surfaces.** All three read the trial agent's CLI-looping as evidence that *both*
  status-quo paths were bad — which that agent said itself — rather than as evidence for
  cutting a surface. Two independently noted that the >20-image-block clamp is an **MCP**
  correctness cliff, so MCP is the surface that most needs the fix.
- **`--full` with a range is refused, naming why** — never silently ignored. All three
  reached the project's own recurring failure mode for this: a flag that accepts a request
  for fidelity and quietly does not deliver it is the same lie as the sampler whose silence
  read as coverage.

## The splits

**Q5 — how much of the disclosure the plain-text form carries.** Jurors 1 and 3 required
the full `rule` / `skipped` / `coverage` / `blind_to` content in prose; Juror 2 required
one prose sentence with the structured detail behind `--json`, on the ground that
ADR-0094 §6 had already settled it. Resolved in ADR-0097 **for Jurors 1 and 3**: §6 says
structured disclosure *plus* one sentence of prose, and Juror 2's reading turns *plus*
into *or*. See the ADR's *"Unconditional cannot mean behind a flag"*.

**Q6 — whether this ticket fixes `--crop`'s spelling.** Jurors 1 and 3 settled the
spelling and the not-yet-legal refusal; Juror 2 settled only the refusal and called the
rest scope creep into #406. Resolved **for Jurors 1 and 3**, on Juror 3's observation
that the combination is reachable *today* and, with #402 live, returns a silently
half-scaled sheet.

**Q7 — the caption obligation. This is what the panel was worth.** Jurors 1 and 2 said
the per-tile label discharges it. Juror 3 alone answered (d) and found what the other two
missed: **#400's label is *fitted* to tile width, so at 140 px it can carry an instant and
little else — it cannot name the presence set**, at exactly the width where the refusal is
about to fire. Juror 1 half-saw it from the other side, noting a fitted label *"cannot
carry a full resolved stack"*, then accepted a second `frame --at` call as the price
instead of following it through. Resolved **for Juror 3**. Note that Juror 1 had already
said it would take the range-level block as a friendly amendment, so 1 and 3 agree on
substance once (d) is on the table.

## What the panel did not have

The brief did not tell the jurors that
[#405](https://github.com/MBehtemam/Montagent/issues/405) exists — the filed bug for
ADR-0011's `2691` figure — though it did give them the corrected measurement. Juror 3
independently proposed folding the correction into this amendment. #405 had already
reached the same conclusion and stated the ordering: *"an amendment would inherit these
numbers. Correct them first."* No juror saw that sentence.
