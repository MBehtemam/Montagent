# Ballot — the bare `mask` key in ADR-0040

**Verdict: A on the spelling, C on the fixture instance. ACCEPT WITH MODIFICATION against the
brief's framing.**

The contradiction is real, but the brief mis-locates it, gets one of its supporting claims
backwards, and candidate (A) as written is **not implementable today** — the migration target
does not exist in any accepted document.

---

## 1. The contradiction is real, and verified

Both quotes are verbatim and both are on `main`.
`docs/adr/0040-effect-model-attachment-and-v1-vocabulary.md:165-171` carries, four lines apart:

> - `mask:"circle"` on `handle-logo` in the committed fixture becomes a valid declaration
>   under this ADR rather than an inert stray field — **no migration needed**, since the value
>   was already legal shape-vocabulary syntax; `validate` should stop treating it as an
>   unknown key once the schema lands.
> - The schema gains a discriminated union `effects: [{name, ...params}]` with exactly three
>   members in v1 …

The fixture is as described —
`fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json:118` ends
`…,"clip":[478,96,68,68],"mask":"circle"}`.

ADR-0017 is as described: closed **at every object level**, at `error` severity, no `x-` prefix,
no in-band escape hatch, sidecar only. #168:180 confirms `serde`'s `deny_unknown_fields` as the
implementation. With `effects: [...]` as the sole attachment point, a bare element-level `mask`
key is an unknown key, which is an `error` that blocks `render`. "No migration needed" is false.

### The brief's diagnosis is right by accident; the real defect is one clause

The brief frames this as "the Consequences section picked one branch of an unresolved fork."
That reading is wrong, and the correction matters for who has authority to fix it.

The body sentence the brief cites — *"Shipping it converts that stray field into either a valid
declaration or a validation error"* (0040:59-61) — is not an unresolved fork left dangling for
Consequences to settle. Read against the **Decision** section that follows it, the fork is
already closed: attachment is `effects: [...]`, therefore a bare key is the validation-error
branch. The Consequences bullet did not pick an open branch; it picked the branch its own
ADR's Decision section had already excluded. **Decision beats Consequences**, so nothing about
the effect model is actually in doubt — only one sentence of Consequences is wrong.

The whole defect is legible in the bullet's own stated reason:

> *"since the value was already legal shape-vocabulary syntax"*

`"circle"` is indeed a legal member of the shape vocabulary. The **key** `mask` is not a legal
element-level key. The sentence establishes value legality and concludes key legality. That is
the entire error, and it is the sentence to quote in the amendment.

## 2. Three independent documents already resolve this to `effects`-only

None of these are in the brief, and together they mean the project has *already* decided
against a bare `mask` field everywhere except in that one Consequences bullet.

**`CONTEXT.md`, the `Mask` entry (lines ~197-205):**

> **Mask**: An `effects` vocabulary member: a closed shape (`circle`, `rect`, `ellipse`) …

The glossary defines `mask` only as an `effects` member. There is no bare-field spelling
anywhere in `CONTEXT.md`, and `docs/agents/domain.md` makes the glossary binding on output
("use the term as defined in `CONTEXT.md`"). The `Effect` entry likewise: *"A list, not a map
or a single field."* A bare `mask` key is precisely a single field.

**ADR-0041** (accepted *after* 0040), `docs/adr/0041-…:65-80`. Its canonical key-order table
gives `image` as:

> `source, x, y, origin, width, height, fit, clip, scale`

`mask` **is not in it**. The same ADR then says any effect property "this ADR doesn't
enumerate — `rotation`, `opacity`, `layer`, `stroke`, `effects` — gets its position from the ADR
that introduces it." So the accepted key-order ADR already models an `image` schema that has
`effects` and has no bare `mask`.

**ADR-0049**, which amends 0040, describes the vocabulary throughout as "the existing
`blur`/`shadow`/`mask`" *members*, never as a field.

## 3. Attacking each candidate

### (B) Admit both spellings — refuted outright

Three independent standing rules kill it:

- **`CONTEXT.md`, `Colour`**: *"`#RRGGBBFF` is an error naming the six-digit form — **two
  spellings of one value break the write-read round trip, as `center-center` does**."* That is
  the project's explicit general rule against sugar, already applied twice.
- **ADR-0040's own attachment argument.** The list field was chosen over per-type baked-in
  properties because "baking effects into element types scatters the vocabulary across every
  type's schema and forces N×M duplication." A bare `mask` field *is* a baked-in per-type
  property. (B) reintroduces exactly what the Decision section rejected, for one member only.
- **ADR-0041 / `fmt`.** Two spellings means either `fmt` normalises between them (it may not —
  #168 story 7: `fmt` never adds or removes a field) or the type has two canonical key orders.

(B) is dead. It is also the candidate most likely to be reached for, because it looks like the
cheap fix that makes the fixture validate unchanged.

### (A) Migrate — correct on the spelling, but **not executable today**

This is my strongest finding and the brief does not contain it.

ADR-0040's union member is written:

> `mask{shape: "circle"|"rect"|"ellipse", ...shape params}`

The `...shape params` is an **ellipsis standing in for a decision that was never taken**. No
accepted ADR states a circle mask's centre or radius, whether they default to the element's
declared rect, whether they are element-space or frame-space, or whether they are optional.
`docs/adr/` contains no shape-parameter set for masks; `CONTEXT.md`'s `Mask` entry says only
"numeric parameters."

Therefore `effects: [{"name": "mask", "shape": "circle"}]` is **not a known-legal document**.
(A)'s migration has no target. Any implementer who writes that rewrite is inventing the
target's shape, and under ADR-0017 an invented field set is either an unknown key or a
silently-underspecified one.

(A) is still the right answer *on the spelling* — the bare key retires and names `effects` —
but the amending ADR must fix the shape-parameter set first, or it ships a retirement that
points at nothing.

### (C) Delete from the fixture — the strongest reading of the evidence, and the one that
cannot be taken by the check

I verified the brief's claim and it holds, and goes further than the brief says:

- `fixtures/en-halloween-decorating/README.md:143-144`: *"The badge's roundness is **baked into
  the asset** — `logo-en.png` is 800×800 RGBA with corner alpha 0 — not produced by the
  compositor."* Confirmed on disk: `PNG image data, 800 x 800, 8-bit/color RGBA`.
- Stronger, and not in the brief: **there is no masking operation anywhere in the published
  pipeline.** `grep -in "logo|png|clip"` across all seven
  `fixtures/en-halloween-decorating/reference/subtitles/*.ass` returns **nothing** — no `\clip`,
  no `\iclip`, no logo reference. The badge was composited outside ASS from a PNG that already
  carried its alpha. The same README notes the panels' square corners were verified against the
  published MP4 and the literal ASS rectangles.

So the bare `mask:"circle"` is best read as a **hand-transcription annotation describing the
asset**, not a compositor instruction that was ever executed. Under (A), migrating it turns an
inert annotation into a live render operation — a redundant clip, with geometry nobody has
specified, applied to the one artifact #168:246 names as the regression guard and #168:296
names as *"the only test in this spec capable of falsifying the format itself."*

But (C) may **not** be chosen by the implementer on that reasoning, because that is the gravity
failure verbatim: ADR-0043 records that 4 of 6 agents found the fork and **3 of those 4 shipped
the delete-only repair anyway**. Deletion here is an act of authorship about the published
video, and it belongs in an ADR made by a human with a render in hand — not in a migration
script and not in a finding's `repair` value.

Note also ADR-0003's asymmetry, which the brief correctly flags: deleting the key must be
recorded as *not* retracting `mask` from v1. ADR-0040 calls the fixture's `mask` "the strongest
evidenced candidate of any considered"; once the key is gone that becomes a historical claim
about a file that no longer says it, and the amending ADR must say so, or a later reader
re-opens the mask decision on an absence.

---

## Rulings on the four questions

### 1. ADR amendment, not a ticket-level decision

`docs/agents/domain.md` is explicit: *"ADRs are amended, never rewritten. A later ADR that
corrects an earlier one says so in its own text, and the earlier one gets a short pointer under
its title."* #168 is equally explicit that it has no standing here: *"Where this spec and an ADR
disagree, the ADR wins and this spec is wrong."* A ticket deciding this would be the
implementation spec overriding the format spec, which is the one move #168 forbids itself.

A retired spelling is additionally a schema-error class under ADR-0016/0017 — only an ADR
creates one. #168 story 14 enumerates the retired spellings (`gravity`, `box`, `align` on a
non-text element, `anchor` as a string, `center-center`, `#RRGGBBFF`, `bold`, `none`/`fill`) and
**`mask` is not among them**; adding it to that list is an ADR consequence, not a spec edit.

Concretely: open a GitHub issue first (I searched all issues — **none exists**; nothing in the
repo tracks this), then a new ADR-0068 from a `domain/` branch, amending **0040** (retires the
"no migration needed" bullet, fixes the shape-parameter set) and **0041** (its `image` key-order
table and its `mask` sentence). #178 → ADR-0067 is the exact worked precedent: two accepted,
contradictory ADRs, resolved by a third, neither overturned.

### 2. Refuse-class, `repair: "none"` — unambiguously

ADR-0043's uniformity test: *"If any instance a check can match is capable of being
load-bearing, the check emits `repair: "none"` for every instance."* A bare `mask` clears that
bar twice over:

- **The target's parameters are not in the document.** Even granting a settled shape-parameter
  set, a bare `mask:"circle"` does not say what circle. The migrated member's numeric fields
  would have to be inferred from the element rect — a semantic assumption the file does not
  carry. That is ADR-0043's definition of refuse-class: *"the fix depends on knowing what the
  author meant."*
- **Two plausible repairs, discriminated only by facts outside the document.** In this fixture
  the render-preserving repair is deletion (the PNG carries its own alpha); in a hypothetical
  file over an opaque source it is migration to a live clip. The discriminating fact — the
  source's alpha channel and the author's intent — is not in the project file. Structurally
  identical to `gravity`'s inert-`top` versus load-bearing-`bottom` fork.

ADR-0016's carve-out does not rescue it: that exempts retirements whose *"repair is arithmetic"*
and ships a script. There is no arithmetic here.

Ship it with the sibling census ADR-0043 requires (grouped by observable geometry; n = 1 today),
and with `render` non-bypassable.

**A trap inside this ruling.** `gravity`, `box`, `align`, `anchor` all have entries under
`CONTEXT.md`'s **Rejected terms**, so the obvious move is to add a `mask` entry there explaining
what the bare key used to mean. **Do not.** ADR-0043 records the measurement: agents given a
glossary entry describing a retired field's old semantics solved the analogous fork **0 of 3**,
versus **2 of 3** given the bare error — *"measurably harmful,"* and #76 is the ticket where
`CONTEXT.md`'s `Gravity` entry was found to be a defect generator in its own right. The existing
`Mask` entry is safe precisely because it describes the *present* spelling and never the retired
one. Keep it that way.

### 3. Who migrates the fixture, and when

A human, inside the amending ADR's own PR, and **after** `render` exists and the fixture has
been rendered against `reference/en-halloween-decorating.mp4`.

#168:246 — *"It is the regression guard: it is a real published video, and a check that fires on
it is wrong unless an ADR says otherwise."* The amending ADR is exactly the "otherwise," and it
has to say so in words, or the first `validate` run legitimately reports an error on the
project's own reference artifact.

There is a genuine ordering tension the implementer will hit:

- The schema with `deny_unknown_fields` **must not** land before the fixture edit, or the
  regression guard fails its own validator on day one.
- The fixture edit **must not** land before a renderer exists, or nobody can prove the edit did
  not change the picture — and that comparison is #168:296's only falsification test.

Resolve it by choosing deletion in the ADR (byte-safe and render-preserving on the ASS/alpha
evidence above, decided by a human, and reversible) and sequencing the render comparison as the
confirmation rather than the precondition. Do **not** resolve it by letting the fixture validate
dirty "for now."

One knock-on to fix in the same PR: ADR-0041's *"Measured against the committed fixture, this
costs zero bytes"* passage names *"`mask` on one image appending after their type's core fields,
still in a stable relative position."* That sentence stops being true whichever way this goes.

### 4. Yes — a class, and here is where to look

Three sweeps, in descending yield:

1. **Missing "Amended by" pointers.** `docs/adr/README.md:96` lists ADR-0040 as amended by
   **0048, 0049, 0055, 0059**. ADR-0040's file carries **no pointer block at all** — compare
   ADR-0006, which carries three, and which `domain.md` names as the worked example precisely so
   that *"a reader landing on an ADR must be able to see it has been superseded without having
   read the one that superseded it."* ADR-0040 is therefore among the ADRs most likely to be read
   stale, which is a plausible reason this survived its court and three subsequent amendments.
   This is a mechanical audit: for every README row with a non-empty *Amended by* column, assert
   the target file contains a matching pointer. Highest-yield check available and it costs a grep.

2. **Consequences bullets that restate a Decision and drift from it.** The defect class here is
   *intra*-ADR: a Consequences bullet asserting what its own Decision section excludes. I found a
   second confirmed instance while ruling — ADR-0041's `image` key-order table omits `mask` while
   its own prose two paragraphs earlier names `mask` as an image key in stable position. Sweep the
   Consequences of every ADR that introduces a schema field, specifically for claims about the
   **committed fixture**; fixture claims are where a Consequences section reaches for a concrete
   detail it did not re-derive.

3. **Ellipses inside schema sketches.** `mask{shape: …, ...shape params}` is a placeholder that
   reads as a settled union. Grep the series for `...` inside braces; each hit is a spec gap that
   an implementer will fill in by invention.

Precedent for the class being real rather than a one-off: #178 / ADR-0067 — two accepted ADRs,
both using *floor*, different numbers, neither amending the other. That is this defect at
inter-ADR scope; the present one is the same defect at intra-ADR scope.

---

## The single thing most likely to be got wrong

The implementer will write the mechanical rewrite
`"mask":"circle"` → `"effects":[{"name":"mask","shape":"circle"}]`, ship the check as
**advise**-class with that object printed as its `repair` value, and apply the same edit to the
fixture.

All three moves are wrong, for one shared reason: **the target member's parameter set was never
specified** (`...shape params` is an unmade decision, not a shorthand), so the printed "repair"
is an invention dressed as a fact — which is exactly the authoritative-sounding prose ADR-0043
measured at 0 of 3 versus a bare error's 2 of 3. And on this specific element the invention is
probably also wrong in the other direction: the published video never masked anything, the
roundness is alpha baked into `logo-en.png`, and the render-preserving edit is removal.

Fix the shape-parameter set first. Then retire the key as **refuse**-class. Then let a human,
holding a render diffed against the reference MP4, decide what `handle-logo` should say.
