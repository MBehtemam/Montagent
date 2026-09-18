# Brief: ADR-0040 contradicts itself on the fixture's bare `mask` key

You are ruling on a live contradiction in an accepted ADR, found while breaking the v1
implementation spec into tickets. Repository: `/Users/mohammedehtemam/projects/github/Montaget`.

## The facts

The committed fixture's `handle-logo` element carries a **bare top-level `mask` key**:

```json
{"id": "handle-logo", "type": "image", "group": "header", "start": 0, "end": 65216,
 "source": "brand/logo-en.png", "x": 478, "y": 96, "origin": "top-left",
 "width": 68, "height": 68, "fit": "cover", "clip": [478, 96, 68, 68],
 "mask": "circle"}
```

[ADR-0040](docs/adr/0040-*.md) (accepted, on `main`) adopted an effect model. Two of its
statements cannot both be implemented:

**Its schema clause** — *"The schema gains a discriminated union `effects: [{name, ...params}]`
with exactly three members in v1: `blur{radius}`, `shadow{dx, dy, radius, color, opacity}`,
`mask{shape: "circle"|"rect"|"ellipse", ...shape params}`."*

**Its Consequences clause** — *"`mask:"circle"` on `handle-logo` in the committed fixture becomes
a valid declaration under this ADR rather than an inert stray field — no migration needed, since
the value was already legal shape-vocabulary syntax; `validate` should stop treating it as an
unknown key once the schema lands."*

If masks live in `effects: [...]`, then a bare `mask` key on an element is an **unknown key**.
[ADR-0017](docs/adr/) makes the schema closed at every object level, and #168's implementation
plan enforces that with `serde`'s `deny_unknown_fields`. So "no migration needed" and the
`effects` union are in direct conflict.

Note that ADR-0040's own body already contains the unresolved fork, in the `mask` paragraph:
*"Shipping it converts that stray field into either a valid declaration or a validation error."*
The Consequences section then picked one branch without reconciling it with the schema clause.

## Context you must establish yourself

Read these rather than taking the brief's word:

- `docs/adr/0040-*.md` in full — especially the `mask` paragraph and the Consequences.
- `docs/adr/README.md` — the index of all 67 ADRs with an *Amended by* column. Roughly a third
  of the series amends another; read the index before any single ADR.
- ADR-0017 (closed schema), ADR-0016 (unknown keys), ADR-0043 (refuse-class findings and how a
  retired spelling is classified), ADR-0003 (the fixture is evidence a capability is *needed*,
  never that one is *unneeded*).
- `CONTEXT.md` — the domain glossary, including its **Rejected terms** list. Note what precedent
  exists for retiring a spelling: `gravity`, `box`, `align` on a non-text element, `anchor` as a
  string, `center-center`, `#RRGGBBFF`, `bold`, `none`/`fill` as a `fit` value.
- `docs/agents/domain.md` — the ADR conventions, including the rule that ADRs are amended,
  never rewritten, and the evidence requirement.
- The fixture itself, and `fixtures/en-halloween-decorating/README.md` — note that the badge's
  roundness is **baked into `logo-en.png`** (800×800 RGBA with corner alpha 0), which bears on
  whether the `mask` key is load-bearing at all.

## The question

**What should happen to the bare `mask: "circle"` key?** The candidate answers, which you should
attack rather than choose between if you think the real answer is elsewhere:

- **(A) Migrate.** The fixture becomes `effects: [{"name": "mask", "shape": "circle"}]`, and a
  bare `mask` key becomes a **retired spelling** naming `effects` as its replacement — the same
  treatment `gravity`, `box` and `center-center` received. ADR-0040's "no migration needed"
  sentence is retired.
- **(B) Admit both spellings.** The element schema keeps a bare `mask` field alongside `effects`,
  as sugar for the single-shape-mask case.
- **(C) Something else** — including the possibility that the key should simply be deleted from
  the fixture (the PNG already carries its own alpha), or that the contradiction reveals a
  deeper problem with where masks live.

Rule also on these, which follow from whichever answer you give:

1. **Is this an ADR amendment or a ticket-level decision?** `docs/agents/domain.md` governs.
2. **If a spelling is retired, what is its refuse class under ADR-0043** — advise (the finding
   carries a repair) or refuse (`repair: "none"`, no override)? ADR-0043's uniformity rule is
   that if *any* instance a check can match could be load-bearing, the check refuses for *every*
   instance. Is a bare `mask` ever load-bearing in a way that makes the repair ambiguous?
3. **Who migrates the fixture, and when?** The fixture is the project's regression guard, and
   #168's rule is that *"a check that fires on it is wrong unless an ADR says otherwise."* If
   the fixture must change, that is a real edit to the one artifact everything is judged against.
4. **Does this reveal a class of defect rather than an instance?** ADR-0040 is one of 67 ADRs,
   roughly a third of which amend another. This contradiction survived a three-juror court when
   ADR-0040 was accepted. If there is a reason to expect siblings, say where to look.

## How to answer

**Default to "refuted".** Attack every candidate answer including the one you end up preferring.
Do not assume the brief's framing is right — if the contradiction is not real, or is real for a
different reason than stated, say so and show the text that settles it.

Verify every claim against the repository. The brief may contain errors; finding them is part of
the exercise.

Structure your ballot as: a verdict line (`A` / `B` / `C`, with `ACCEPT` / `ACCEPT WITH
MODIFICATION` / `REFUTE` against the framing), then the reasoning with ADR citations, then your
rulings on the four numbered questions above. End with the single thing most likely to be got
wrong by whoever implements this.
