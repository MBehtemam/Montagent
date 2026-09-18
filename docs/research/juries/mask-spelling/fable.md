# Ballot — the bare `mask: "circle"` key (juror: Fable)

## Verdict

**A — ACCEPT WITH MODIFICATION.** The contradiction is real and is exactly where the brief
says it is. Migrate the fixture to the `effects` list and retire the bare key as an
**advise-class** retired spelling. But the brief's version of (A) is not implementable as
written, because ADR-0040 never fixed what a `mask` effect's parameters are, so the migration
target `{"name":"mask","shape":"circle"}` is a guess at a schema nobody has published. The
amendment has to settle that first. (B) and (C-delete) are refuted below; (C-deeper) is
partly right — the defect is a class, not an instance — but the class is about deferral
bookkeeping, not about where masks live.

## What I verified, and where the brief is wrong

**Verified true.**

- Both ADR-0040 clauses are verbatim (`docs/adr/0040-effect-model-attachment-and-v1-vocabulary.md`,
  Consequences bullets 1 and 2), and the "either a valid declaration or a validation error"
  fork is in the `mask` paragraph. The ADR landed in a single commit (`aa141f88`), so the
  Consequences bullet was not a later edit.
- ADR-0017 closes every object level; #168 line 173 enforces it with `deny_unknown_fields`.
  Under those two, a bare `mask` on an `image` is an unknown key the moment `effects` exists.
  The Consequences bullet's reasoning — *"the value was already legal shape-vocabulary
  syntax"* — is the precise error: it argues from the **value** (`"circle"` is in the enum)
  to the **key** (`mask` at element level), and the schema clause puts the key somewhere else.
- The PNG is 800×800 RGBA (colour type 6). I decoded it: all four corner alpha are 0, the
  centre is 255, and on a 4-px sample grid there are **zero** opaque pixels outside the
  inscribed circle and zero transparent pixels inside it. The badge is already a circle in
  the asset. With `clip` equal to the element rect and a square 68×68 slot, a circle mask is
  pixel-inert under every plausible default geometry.
- The fixture's `mask` key has been carried, deliberately and byte-unchanged, through every
  migration since the sample landed: `migrate.py` line 108 (`# still #22, unchanged`),
  ADR-0012 ("shape and soft masks … to #22"), ADR-0014 ("`mask` … stays #22's"),
  ADR-0041 ("`mask` on one image appending after their type's core fields"). Three ADRs
  passed the key forward to #22; #22's ADR then declared it needed nothing.

**Brief errors and over-statements.**

1. *"Roughly a third of the series amends another."* 52 of 67 ADRs carry an `amends:` line.
   It is closer to four-fifths. (README's reading-order paragraph also still says "66 ADRs"
   with 67 on disk — stale by one.)
2. *"This contradiction survived a three-juror court."* Misleading. The effect-model
   packet (`docs/research/juries/effect-model/BALLOTS.md`) asked Q1–Q4 — attachment,
   vocabulary, text effects, absent-list. **No question asked what happens to the fixture
   key.** The packet itself told the jurors the key was "a no-op there, since the source
   image already has alpha-transparent rounded corners." The "no migration needed" sentence
   was authored after the court, on a question the court was never put. Nothing survived
   scrutiny; nothing was scrutinised.
3. *"`gravity`, `box` and `center-center` received the same treatment."* They did not.
   `gravity` is refuse-class under ADR-0043 (`repair: "none"`, uniform, non-bypassable).
   `center-center` is a pure spelling substitution naming `center` (ADR-0013). `box` was
   migrated by arithmetic script (`migrate.py`), the ADR-0016 "arithmetic retirement"
   path. ADR-0043 classifies **only `gravity`**; grep it for `center-center`, `box`,
   `align`, `anchor`, `bold`, `RRGGBBFF` — none appear. "Retired spelling naming a
   replacement" is a message-text property (ADR-0016), not a repair class. The brief
   conflates the two, and an implementer copying its phrasing will too.
4. The brief's (A) writes the target as `effects: [{"name": "mask", "shape": "circle"}]`.
   ADR-0040's schema clause says `mask{shape: …, ...shape params}` and CONTEXT.md says
   "numeric parameters only" — and neither names a single parameter, a default, or what
   a param-less circle means (inscribed in the element rect? centred at `origin`?
   diameter = min side?). Contrast ADR-0049, which spelled every colour member's fields
   and an example literal. The migration target is underdetermined **by the ADR being
   amended**, and under ADR-0030 whether the fixture writes a default explicitly or omits
   it is itself content. That is the modification.

## Attacking the candidates

**(B) Admit both spellings — REFUTED.** Two spellings of one value is the exact thing
ADR-0013 (`center-center`), ADR-0012 (`scale` union) and CONTEXT.md (`#RRGGBBFF`) reject,
for a mechanical reason: `fmt` normalises on write and the agent's next exact-string replace
gets zero hits. It also breaks ADR-0040's own ordering argument — a bare `mask` has no
position in the `effects` list, so `blur` then `mask` versus `mask` then `blur` becomes
inexpressible for the sugar form. And it would be the first field in the format that is
"sugar for" another field; nothing else is.

**(C-delete) Remove the key from the fixture — REFUTED, narrowly.** It is pixel-identical
(verified above) and it is the smallest edit. It loses because: (i) ADR-0015 measured that
agents author by copying the shipped fixture *before* reading the spec, which is why that
ADR migrated the fixture *in the same change* as the retirement; a fixture with zero
`effects` on it teaches nobody the canonical spelling. (ii) #168's ticket 15a wants `frame`
exercised on "what the fixture uses (image, rect, text, `mask`, keyframed `scale`)" — delete
the key and v1's `mask` effect has no regression coverage at all. (iii) ADR-0003's asymmetry
does not forbid deletion (absence proves nothing), but ADR-0040 leaned on this key as "the
strongest evidenced candidate"; deleting the evidence the moment it becomes expressible is
perverse. Deletion is defensible only if the amendment *cannot* settle the mask parameters —
then an inert key you cannot spell correctly is worse than none.

**(C-deeper) "masks live in the wrong place" — REFUTED.** Nothing in the record suggests
the `effects` list is wrong; the list-with-order decision was 3/3 and is used by ADR-0049
and ADR-0055 since. The contradiction is a bookkeeping failure, not a model failure.

**(A) — ACCEPTED WITH MODIFICATION.** Migrate; retire the bare key; but the amending ADR
must first publish the `mask` member's parameter set and the meaning of the param-less
form, because the fixture cannot be written against a shape that does not exist.

## The four rulings

**1. ADR amendment, not a ticket decision.** `docs/agents/domain.md`: ADRs are amended,
never rewritten, and a later ADR that corrects an earlier one says so in its own text while
the earlier one gets a pointer under its title. Two accepted sentences in ADR-0040 cannot
both be implemented; picking one in a ticket would make #168 override an ADR, which #168
itself forbids (*"Where this spec and an ADR disagree, the ADR wins"*). The amendment
carries: the retirement of the "no migration needed" bullet; the `mask` parameter set and
its defaults (or an explicit statement that the bare `shape` form means the inscribed
shape); the fixture diff; the `migrate.py`/`verify.py` change; a pointer under ADR-0040's
title; and an `Amended by` entry in the README. Note ADR-0041's `image` key-order table
omits `mask` and its prose says the key "appends after the core fields" — the amendment
must also say where `effects` sits in image order, or ADR-0041 goes stale the same way.

**2. Advise-class, with one condition.** ADR-0043's test is whether *any* instance the check
can match could be load-bearing in a way the document cannot resolve. The bare key never
had defined semantics in any accepted document — it was "undeclared" from the day it was
committed — so there is no old meaning for a repair to misread; the repair is a spelling
transposition of a value that is *already* a member of the new enum. The `gravity` fork
existed because the old field's inputs (source pixel dimensions) were absent from the
document; here the only input is the string itself. Condition: the amendment must define
the param-less form's geometry. If it instead makes centre/radius **required** with no
default, then `mask:"circle"` on a non-square element has no determinable target and the
check flips to refuse-class uniformly — including on the fixture's square badge, where it
would be a false refusal ADR-0043 says to accept. Recommend defining the default so the
check stays advise. The population of real instances outside this repo is, as far as the
record shows, zero; the fixture's own instance is pixel-inert both ways.

**3. The amending ADR's PR migrates the fixture, in the same change as the decision.** This
is ADR-0015's precedent exactly (*"the migration lands in the same change as the decision.
That is not tidiness."*), and ADR-0016's rule that an arithmetic retirement ships its script
beside the ADR. #168 ticket 3's "the fixture round-trips" demo then has a fixture that
round-trips; ticket 3 does not get to decide format bytes. The edit is one line (line 118),
one key becomes `"effects":[{"name":"mask","shape":"circle"<defaults per amendment>}]`,
and `verify.py` (which today never mentions `mask` at all) gains an assertion that the bare
key is absent and the effect present. #168's rule — *a check that fires on the fixture is
wrong unless an ADR says otherwise* — is satisfied by construction, because the ADR that
says otherwise is the one landing the bytes.

**4. Yes, a class — three overlapping ones, with addresses.**
   - *Deferrals passed forward that the successor never discharged.* Grep `docs/adr` for
     `stays #`, `still #`, `to #NN`, `deferred to`, then check whether the successor ADR's
     Consequences state what happens to any fixture bytes carrying the deferred thing.
     `mask` went ADR-0012 → ADR-0014 → ADR-0041 → ADR-0040 and the last link dropped it.
     ADR-0040's own deferrals (colour filter → 0049, highlighting → 0048, transitions →
     0059) are the next places to look; none of those touch fixture bytes, so they are
     probably clean, but that is a check, not an assumption.
   - *Missing `Amended by` pointers.* README lists 28 ADRs as amended; **16 of them carry
     no "Amended by" pointer in their own text** (0001, 0002, 0010, 0011, 0014, 0016,
     0020, 0021, 0032, 0034, 0036, 0039, 0040, 0046, 0048, 0063). `domain.md` makes the
     pointer mandatory. ADR-0040 is one of the 16 — a reader of it today cannot see that
     0048/0049/0055/0059 touched it. This is the mechanism by which a self-contradiction
     goes unnoticed: nobody re-reads an ADR that looks unamended.
   - *Retired spellings with no ADR-0043 class.* #168 story 14 lists eight retired
     spellings; ADR-0043 classifies one. The other seven (and now `mask`) will be classified
     by whoever writes the check, at ticket level, which ADR-0043 does permit ("decided once,
     by whoever authors the check") — but `box` and `align`-on-image have `gravity`-shaped
     intent forks and nobody has said so in an ADR.

## The single thing most likely to be got wrong

An implementer reads "no migration needed" and adds `mask` as an **optional alias field on
the image struct** beside `effects` — option (B), silently, in Rust — because that is the
only way to make both accepted sentences true at once, and `deny_unknown_fields` will make
the fixture fail loudly until they do. The ADR text invites exactly that. The amendment must
land, and the fixture must change, *before* ticket 3 is started, or (B) ships by accident and
the write-read round trip breaks on the first `fmt`.
