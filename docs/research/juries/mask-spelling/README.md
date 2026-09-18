# Jury: the bare `mask` key

Evidence for [ADR-0068](../../../adr/0068-the-bare-mask-key-retires-masks-are-effects-members.md).

Three independent jurors (Opus, Haiku, Fable), each given the same brief ([`BRIEF.md`](BRIEF.md))
cold, blind to each other, instructed to default to "refuted" and verify against the repository
rather than the brief. Full ballots: [`opus.md`](opus.md), [`haiku.md`](haiku.md),
[`fable.md`](fable.md).

The question: [ADR-0040](../../../adr/0040-effect-model-attachment-and-v1-vocabulary.md) puts
masks inside `effects: [{name, ...params}]` in its schema clause, and says the committed
fixture's bare `mask: "circle"` key needs *"no migration"* in its Consequences. Under
ADR-0017's closed schema those cannot both hold. Surfaced while breaking
[#168](https://github.com/MBehtemam/Montaget/issues/168) into tickets — it is the first
ticket's demo that fails.

## Verdicts

| | Opus | Haiku | Fable |
| --- | --- | --- | --- |
| the spelling | **A** (migrate) | **A** (migrate) | **A** (migrate) |
| admit both spellings (B) | refuted | refuted | refuted |
| amendment or ticket? | amendment | amendment | amendment |
| refuse class | **refuse** | advise | advise *if a default is defined* |
| instance or class? | class | class | class |

**Unanimous on the spelling.** Masks live in `effects`; the bare key retires. (B) was refuted
by all three on the same mechanical ground, which `CONTEXT.md` states directly about
`#RRGGBBFF`: two spellings of one value break the write-read round trip, *"as `center-center`
does"* — `fmt` normalises on write and the agent's next exact-string replace gets zero hits.

## The split, and why it resolves the way it does

The court divided on whether the retired spelling is advise-class or refuse-class, and the
division turns out to rest on a false premise rather than a judgment difference.

**Opus ruled refuse because the migration target does not exist.** ADR-0040's union member is
written `mask{shape: "circle"|"rect"|"ellipse", ...shape params}`, and that ellipsis is an
unmade decision, not shorthand — no accepted document names a single parameter, a default, or
what a param-less circle means. So `{"name":"mask","shape":"circle"}` is not a known-legal
document, and printing it as an advise-class repair is *inventing the target's shape* — the
authoritative-sounding prose ADR-0043 measured at 0-of-3 jurors against a bare error's 2-of-3.

**Fable found the same gap independently** and made its vote conditional: advise if the
amendment defines the param-less form's geometry, refuse uniformly if it makes centre/radius
required with no default — since then `mask:"circle"` on a non-square element has no
determinable target. It also noted the contrast with ADR-0049, which spelled out every colour
member's fields and gave an example literal.

**Haiku ruled advise on the ground that the repair is mechanical** — correct, but resting on
the assumption that the brief's migration target was already legal. It is not.

**Resolution:** the disagreement is downstream of a gap all three agree exists. ADR-0068
closes it by stating the param-less form — the inscribed shape of the element's own rect —
which satisfies Fable's condition and makes the repair a pure spelling transposition of a
value already in the target enum. The classification is therefore **advise**, and Opus's
reasoning is why it could not have been advise without the amendment writing that sentence
first. The full parameter surface is graduated rather than guessed at.

## What the court established that the brief did not contain

**The badge is already the inscribed circle.** Fable decoded `logo-en.png` rather than trusting
the fixture README: 800×800 RGBA, all four corner alphas 0, centre 255, and on a sampled grid
zero opaque pixels outside the inscribed circle and zero transparent pixels inside it.
Independently reproduced by the author at 39,403 sampled pixels with zero violations either
way — [`decode_logo_alpha.py`](decode_logo_alpha.py), re-executable. The key is pixel-inert in
both directions, so this was never a question about pixels; it is about the fixture's role as
the copied example and about regression coverage of a v1 effect.

**The contradiction never survived a court, because it was never put to one.** The effect-model
packet asked four questions — attachment, vocabulary, text effects, the absent-list — and none
asked what happens to the fixture's key. The packet itself *told* the jurors the key was
inert. The "no migration needed" sentence was authored after the court, on a question the court
was never asked.

**The published pipeline never masked anything.** Opus grepped all seven reference `.ass`
files: no `\clip`, no `\iclip`, no logo reference at all. The badge was composited by the old
Python/FFmpeg driver from a PNG with baked alpha, so the key is a transcription annotation
that was never executed by anything.

**ADR-0041's `image` key-order table omits `mask`** — while its surrounding prose describes the
key as *"appending after their type's core fields"*. Both jurors who found this drew the same
conclusion: the amendment must say where `effects` sits, or ADR-0041 goes stale the same way
ADR-0040's Consequences did.

## The class defect

All three ruled this a class rather than an instance, and named the same mechanism from
different angles.

Fable traced the deferral chain: the bare key was carried forward deliberately and
byte-unchanged through ADR-0012 (*shape masks to #22*), ADR-0014 (*"`mask` … stays #22's"*),
ADR-0041 (its key position) and `migrate.py` (`# still #22, unchanged`). Three ADRs handed it
on correctly. **ADR-0040 is #22's own ADR — the end of the chain — and that is where the
hand-off was supposed to be caught.**

Both Opus and Fable independently flagged that **ADR-0040 carries no *Amended by* pointer**
despite four later ADRs amending it, and named that as the mechanism by which a
self-contradiction goes unnoticed: nobody re-reads an ADR that looks unamended. Fable counted
16 of 28 amended ADRs with no pointer; the author's count was 15 of 27, plus 6 more whose
banners named only some of their amenders while reading as exhaustive. Discharged separately
in [#183](https://github.com/MBehtemam/Montaget/pull/183), which also adds
`docs/adr/check_amendment_banners.py` so the rule is enforced rather than merely written.

## Method note

The brief was written by the author and carried three errors the jurors caught:

- *"Roughly a third of the series amends another"* — it is **49 of 67**, closer to
  three-quarters. The same wrong figure had been written into #168's body earlier the same day.
- *"This contradiction survived a three-juror court"* — it did not; see above.
- *"`gravity`, `box` and `center-center` received the same treatment"* — they did not. ADR-0043
  classifies **only `gravity`**. `center-center` is a pure spelling substitution (ADR-0013) and
  `box` was migrated by arithmetic script. *"A retired spelling that names its replacement"* is
  a message-text property under ADR-0016, **not a repair class**, and conflating the two is
  exactly how an implementer would mis-classify the next one.

Fable also caught that the brief's own proposed migration target was a guess at an unpublished
schema — which is the finding that reshaped the amendment.

## The thing most likely to be got wrong

Fable and Opus converged here from opposite directions. An implementer reads *"no migration
needed"*, hits `deny_unknown_fields` failing loudly on the fixture, and satisfies **both**
accepted sentences the only way they can both be true: by adding `mask` as an optional alias
field on the image struct beside `effects`. That is option (B), shipped silently in Rust, and
the write-read round trip breaks on the first `fmt`.

The amendment and the fixture migration must therefore land **before** #168's document-model
ticket starts — which is why ADR-0068 carries the fixture diff rather than delegating it.
