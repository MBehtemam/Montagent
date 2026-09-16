# Jury: canonical key order, `validate` enforcement, `fmt --check` (ticket #73)

Three independent models (Opus, Sonnet, Haiku), run in isolation from each other, from a
brief ([QUESTION.md](./QUESTION.md)) that supplies the incident evidence and this project's
governing principles without steering toward an answer.

## Verdict tally

| | Q1: enforce? | Q2: order shape | Q3: where checked | Q4: split `fmt`? |
|---|---|---|---|---|
| [Opus](./opus.md) | (c) `MUST`, checked | Per-type, with a shared universal prefix then a type-specific tail | Both — `validate` is the real enforcement point, `fmt --check` is convenience, one shared implementation | Split — and separately argues `fmt` shouldn't materialize defaults at all |
| [Sonnet](./sonnet.md) | (c) `MUST`, checked | Per-type | Both | Split |
| [Haiku](./haiku.md) | (c) `MUST`, checked | **Single global order** (dissent) | Both | Split |

Q1, Q3, Q4: unanimous 3/3. Q2: 2–1 for a per-type order, with Opus's ballot the most
developed version of that side — a shared prefix plus per-type tail, with the explicit
argument that a bare global order is "defined over a set the schema does not close," so
adding one field to one type silently reshuffles every other type's lines.

[ADR-0041](../../adr/0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md)
adopts the 2/3 position on Q2 (Opus's version specifically — tying the per-type order to
schema property-declaration order rather than a second hand-maintained list), and both
unanimous positions on Q1/Q3/Q4, per the human's decision after reviewing all three ballots
in full.
