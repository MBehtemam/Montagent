# Juror A1 — #48, the `fit` vocabulary

---

## Q1

**Decision:** `fit` is a **derivation claim** — an assertion about how the author computed
`width`/`height` from the source and the aperture. It is **not** a layout mode and it has
**no render-time effect whatsoever**. Its only consumers are `validate` (and `render`'s
printed notes, which are the same checks), plus the human or agent reading the file. In
consequence `fit` should be understood as a *stated premise*, of the same species as
ADR-0007's literal `size` and ADR-0014's required text `height`: a frozen number's
provenance, written down so that a later divergence is catchable.

**Why:** Follow ADR-0013's "the declared rect is authoritative at render" to its end. At
render the pipeline is: decode the source; resample it to exactly `width` x `height`; place
it at `x`,`y` under `origin`; apply `scale`/`rotation`/`opacity`; clip to the frame-space
`clip` rect. Walk that list and ask where a renderer would *read* `fit`. Nowhere. The extent
it would have computed is already in the file, and ADR-0013 forbids the renderer from
preferring its own computation ("the renderer never silently samples at the exact isotropic
factor and treats the declared integers as an approximate bound"). A field with no consumer
in the operation that produces the artifact is, by definition, not an instruction.

What remains is a comparison. `validate` can recompute the rule value from the media on
disk and the numbers in the document, and report disagreement. `fit` is the field that says
*which* computation to perform — it selects the predicate. That is exactly the shape of an
assertion, not of a mode.

This reframing is doing the work in every question below, so state its three corollaries
plainly:

1. **Omission is "make no claim", i.e. "do not check"** — ADR-0006's named disease (Q3).
2. **"Do not fit this, use my rect" is not a render behaviour at all.** The renderer already
   uses your rect, unconditionally, for every value of `fit`. The escape value is a
   *declination to assert*, not a layout mode (Q4). This is why the naming candidates
   borrowed from CSS `object-fit` all mislead: every one of them names a resampling
   behaviour that Montagent does not have.
3. **The check can be promoted to `error` without touching the renderer** (Q6). Nothing
   about the drawn pixels changes; what changes is whether a document that contradicts its
   own stated premise is allowed to become a video.

**Strongest counter:** A renderer *could* be written to consume `fit`, and one day might
want to — ADR-0012 left "whether `clip` is keyframable" open, and a keyframed aperture makes
a static derived extent visibly wrong over time; the obvious fix there is to let `fit`
re-derive the extent per frame, which would make it a render instruction retroactively and
change what every existing file draws. More sharply: calling `fit` an assertion means the
format carries a field that a correct, minimal renderer may ignore entirely — and ADR-0007
has twice ruled that "a field the renderer cannot honour is worse than no field." I answer
that `render` *does* honour it, by running ADR-0006's checks and refusing on `error`, which
is precisely the mechanism ADR-0006 built so that checks are not optional. But the
resemblance to a decorative field is real, and it is the reason Q3 and Q6 must both go the
strict way: an assertion nobody is obliged to make and nothing refuses on *is* decoration.

---

## Q2

**Decision:** The box is **the `clip` rectangle's extent — `clip[2]`, `clip[3]`** — when
`clip` is present. When `clip` is absent, the box is **the element's own declared
`width`/`height`**. One formula, one substitution rule, written into the schema so it is not
inferred:

> `(bw, bh) = (clip[2], clip[3])` if `clip` is present, else `(width, height)`.

**Why:** ADR-0012 names it outright and never gets quoted: *"`photo-06` is a 1536x2720
source in a **1080x1300 box** under `fit:"cover"`."* 1080x1300 is `clip[2..3]`. The arithmetic
confirms it: cross-multiply `1080*2720 = 2,937,600 >= 1300*1536 = 1,996,800`, so width
drives; width takes 1080 verbatim; `(2720*1080) // 1536 = 1912`, which is the declared
height. And the 612 px of spill (1912 − 1300) is exactly the quantity ADR-0012 introduced
`clip` to bound. The box is the aperture.

**The instructive difference, since it does not fall out the way it first appears.** I
tested the rival reading — box = the declared rect — and it *also* reproduces both elements:
`(1536,2720) -> (1080,1912)` under cover yields `1080x1912`, and `(800,800) -> (68,68)`
yields `68x68`. Box = rect is a **fixed point** whenever the rect already carries the
source's aspect, which it does here *because it was derived from the clip in the first
place*. So the fixture cannot separate the two readings by equality alone. Three things do:

- **Information flow.** Under box = clip, 1912 is a function of 1300 and the source aspect —
  it carries information from the aperture. Under box = rect, 1912 is a function of itself.
  The reading that makes the number derivable is the reading under which `fit` says anything.
- **The driving-axis term goes vacuous.** Under box = clip, "the driving axis takes the box
  dimension verbatim" means `width == 1080 == clip[2]` — a real constraint pinning the rect
  to the aperture. Under box = rect it reads `width == width`. Half of ADR-0013's rule
  evaporates into a tautology, which is proof the substitution is wrong.
- **`handle-logo` is the element you cannot learn from, and that is its lesson.** Its `clip`
  is `[478,96,68,68]` — the aperture *equals* the rect, and its source is square. Both
  readings coincide, `cover` and `contain` coincide, and the exact value needs no rounding
  at all. It is the degenerate case that makes box = rect look plausible, sitting in the
  same file as the one geometry that discriminates. This is ADR-0013's own "the fixture
  escapes the bug entirely" hazard recurring one field over: a lucky data point that
  structurally cannot demonstrate the thing it is being cited for.

**What a declared `fit` means when `clip` is absent.** Not an error, and not a no-op: the
box falls back to the declared rect and the rule keeps running, whereupon `cover` and
`contain` each reduce to a genuine, checkable claim — *the rect carries the source's aspect
ratio, within one floor step* — differing only in which axis is taken verbatim. It has
teeth: a 1536x2720 source declared at `1080x1920` with no `clip` fails under both
(`contain` computes `1080x1912`, `cover` computes `1084x1920`), catching an 8 px
hand-typed stretch that nothing else in the tool surface sees. I prefer this to making
`fit`-without-`clip` a schema error because it is the *same formula* with a substituted
argument rather than a second rule, and because an image placed whole with no aperture is
the commonest thing a general editor does (ADR-0003: the fixture's universal `clip` is not
evidence that the no-clip case is rare).

**Strongest counter:** Two boxes for one keyword is a union, and ADR-0012 rejected unions
for `scale` on the ground that "two possible shapes put a shape test in every consumer, and
one of them will get the rarer branch wrong." Here the rarer branch — no `clip` — is
exactly the one the fixture never exercises, so the wrong branch is also the untested one.
A reader who has only ever seen the fixture will assume box = rect (the fixed point above
positively invites it) and will be right for the wrong reason on every element until the
day they omit a `clip`. The cleaner alternative is genuinely available: make `clip`
**required on image elements** — `verify.py` already asserts it on 8 of 8 — at which point
the box is unconditionally the aperture and the substitution rule disappears. I do not take
it because requiring an aperture on every image is a real expressive cost imposed to protect
a one-line rule, and because it is a decision about `clip`, which is ADR-0012's field and
not this ticket's to tighten.

---

## Q3

**Decision:** **`fit` is required on every image element** (and on any future `video`
element), with no default. A missing `fit` is a schema error naming the value set. `fit` on
a text, shape, or audio element is a schema error — for text and shape the message names
`width`/`height`, since those elements declare their extent and derive nothing.

**Why:** This is ADR-0014's text-box argument with the nouns changed, and the substitution
is exact. ADR-0006's rule:

> An omitted `height` is indistinguishable from a decision not to check.

An omitted `fit` is indistinguishable from a decision not to check, and it is *worse* than
the height case in one respect: `height` at least has a value the checker can compare
against something. An absent `fit` leaves `validate` with a rect, a source, and no stated
relationship between them — it cannot even tell whether 1912 was meant to be derived. That
spends `validate`'s one structural defence, the thing ADR-0006 says rescues it from being
the `sequence` label: *it checks uniformly, so a clean run means something.* With `fit`
optional, `0 errors` over a file of fit-less images means nothing at all, and the author who
most needs the stale-source check is exactly the author who did not type the field.

Three supports:

- **Required is affordable only if the tool computes it** (ADR-0014's rule for `height`).
  It does: the rule value is one integer division from the probed source and the clip, so
  `validate`'s finding must print the computed extents, and `query`'s crop rectangle —
  unblocked by ADR-0013 — already exposes them. An author never hand-derives anything.
- **The census is one-sided.** `fit` is on 8 of 8 image elements in the only real project
  file, so required costs the corpus zero bytes and zero edits. Optional catches the
  replaced-source defect on whatever subset happened to type it; required catches it on all
  of them. There is no class of edit that optional catches and required misses.
- **A default is forbidden twice over.** Defaulting to `cover` would make the document's
  most consequential premise the one nobody wrote, and ADR-0012's rule ("a default is only
  safe where its wrong answer is loud") disqualifies it precisely: a wrongly-assumed `cover`
  fires a stale-source error on a file whose author never claimed anything, which is a loud
  wrong answer in the worst direction — a false positive on a legal file, ADR-0006's alarm
  fatigue. Defaulting to the escape value is worse: it is optionality with extra steps.

**Corollary that must ship with this:** **`fmt` may neither rewrite nor insert `fit`.**
ADR-0013 forbids `fmt` from rewriting a declared extent because under
declared-authoritative the integer is content; `fit` is the *premise* of that integer, so it
is content by the same argument, and a `fmt` that synthesised `"fit":"cover"` from numbers
that happen to satisfy the rule would manufacture an assertion the author never made — and
the agent's next exact-string replace on the line it just wrote gets zero hits.

**Strongest counter:** ADR-0014 kept the text box required partly because the height term is
"armed and unfired" — it catches the *next* edit. `fit` is weaker there: the edit it catches
is a change on disk, not in the document, and the author who replaces a source is not
typically editing the element at all. Meanwhile required-with-an-escape-value has a known
degenerate outcome: an agent that does not want to think writes `"fit":"declared"` on
everything, and the field becomes a required piece of ceremony that asserts nothing on a
majority of elements — the same "optional in practice" state, now costing 8 lines of
boilerplate and a schema clause. That failure is real and nothing in this design prevents
it; the honest defence is only that a *written* declination is greppable and censusable
(`validate` can count them) where an absence is not.

---

## Q4

**Decision:** Exactly three values, published closed:

| value | inequality it asserts | driving axis | rounding |
| --- | --- | --- | --- |
| `cover` | drawn extent **contains** the box: `W >= bw` **and** `H >= bh`, and is the *minimal* aspect-preserving extent doing so | the axis with the larger required ratio: width drives iff `bw*sh >= bh*sw` | driving axis verbatim; slack axis **floors** |
| `contain` | drawn extent is **contained by** the box: `W <= bw` **and** `H <= bh`, and is the *maximal* aspect-preserving extent doing so | the axis with the smaller required ratio: width drives iff `bw*sh <= bh*sw` | driving axis verbatim; slack axis **floors** |
| `declared` | **nothing.** The extents are declared and are not derived from the source. | n/a | n/a |

All comparisons are integer cross-multiplications; the slack axis is
`(s_slack * b_driving) // s_driving`, per ADR-0013, in both fitting values.

**Why:**

*The inequality for `cover` is stronger than coverage alone, and this matters.* Read
literally, `W >= bw and H >= bh` is satisfied by any oversized rect — including ADR-0013's
stale-source case (declared 1912 where the rule gives 1546), which is why that case covers
the aperture and no error fires today. So `fit:"cover"` asserts *identity with the rule
value*, not mere coverage. Mere coverage is a different check — ADR-0013's media-free
aperture-containment error — and the two are complementary, not redundant (see the note on
`declared` below).

*`contain` belongs in v1, and its rounding is forced rather than chosen.* Under `<=`, ceil
breaks the bound and floor preserves it — ADR-0013 says so itself in tiebreak 2: *"If
`contain` is ever defined with the obvious semantics, only floor is safe there."* So
admitting `contain` introduces **no new rounding rule**; it consumes a rule already
published and already paid for, and it keeps one rounding operation across the whole
vocabulary, which was floor's second-strongest justification. The affirmative case is
ADR-0003: `contain` is the letterbox, and the letterbox is the reference class's single most
common fit — CapCut and Premiere both do it by default when a clip's aspect does not match
the sequence. ADR-0012's own retarget census (1080x1920 → 1920x1080 leaves 32 of 60 elements
outside the frame) is a `contain` scenario described without the word. ADR-0013 declined it
only to avoid **creating a value by implication**; #48 is the ticket chartered to create it
by statement, so the refusal does not carry over. And the cost of deferring is asymmetric:
without `contain`, an author who letterboxes must write `declared` and hand-compute, and
`validate` cannot check the commonest fit in the medium.

*The escape value is `declared`, and the name matters more than it looks.* Per Q1 it is not
a layout mode — the renderer already uses your rect for every value of `fit` — so the name
must read as a **declination to assert**, not as a resampling behaviour. `declared` does,
and it echoes the exact phrase the format already uses for this concept (ADR-0012's "a size
is required, not defaulted"; ADR-0013's "the declared rect is authoritative"). One word, one
concept, per ADR-0013's `center-left` reasoning.

The rejected names, and why each is a trap — **every one of them becomes a schema error
naming `declared`**, on ADR-0013's `center-center` and ADR-0012's `anchor` pattern:

- **`fill`** — spent twice inside this format. ADR-0014 makes `fill` the **paint colour on
  a shape**, and ADR-0005 argued about an implicit `fill` meaning background. A value that
  is a sibling field's name, on adjacent element types, is the `box` collision recurring.
- **`none`** — the CSS `object-fit` collision that would actually mislead an agent. In CSS,
  `none` means *use the source's intrinsic dimensions*, which is a genuine constraint on the
  drawn size. An agent writing `fit:"none"` from CSS habit means "don't touch my rect" and
  gets, in CSS terms, something else entirely — and because Montagent honours the declared
  rect regardless, the file *appears* to work while its stated premise is wrong. Silent
  plausibility is this format's named failure class.
- **`stretch`** / **`fill`** (CSS sense) — CSS `fill` means stretch-to-box ignoring aspect,
  which under declared-rect-authoritative is what the renderer *always* does, for `cover`
  too. So the name describes the universal behaviour and cannot distinguish anything.
- **`scale-down`** — CSS's `min(contain, none)`; a derivation with an intrinsic-size term
  Montagent does not have.
- **`crop`** — ADR-0012 rejected a source-space `crop` by name when it chose `clip`.
  Reviving the word as a `fit` value re-opens a closed decision by vocabulary.

**Note that `declared` opens no hole.** ADR-0013's aperture-coverage error — *the declared
rect must contain `clip`* — is computed from the document alone and applies to every image
regardless of `fit`. So `fit:"declared"` declines the *derivation* claim and cannot decline
the *visible* failure: an author who writes a rect too small to cover their own aperture
still gets an error, with no media probe required. This is what makes promoting Q6 to
`error` safe: the escape hatch releases the check that needs the disk, and keeps the check
that does not.

**Strongest counter:** Three values where ADR-0013 deliberately shipped one is the exact
expansion it refused, and my `contain` rests on reference-class reasoning plus an ADR-0013
sentence written as a *conditional* ("if `contain` is ever defined"), which is weaker
evidence than it reads as. There is a real cost: `contain` is the value most likely to be
written by an agent that then expects **letterbox bars**, which this format cannot produce —
`contain` yields a rect smaller than the aperture, the background shows through, and
ADR-0013's aperture-coverage **error fires on every correct `contain` element**. That is a
direct contradiction between two checks I am shipping together, and its resolution — that
the coverage error must be scoped to `cover`/`declared`, or restated as "the rect must
contain `clip` *unless* `fit` is `contain`" — is an amendment to ADR-0013 that I am making
by implication in a sentence, which is the sin this whole chain is organised against. If a
juror wanted to cut `contain` from v1, that is the argument I would find hardest to answer,
and it is one I generated against myself only after writing the table.

---

## Q5

**Decision:** **Retire `gravity`.** `gravity` on **any** element is a schema error; on an
image the message names **`clip`**, and on text and shape it names `origin` (ADR-0014's
clause, subsumed and generalised). The committed fixture loses the field from 8 elements —
~139 bytes — and this is the first decision in the 0012→0013→0014 chain that changes the
project file's bytes.

**Why:** The argument is **structural, and the 8-of-8 measurement is demoted to
corroboration** — because "the fixture does not use it" is forbidden as an argument by
ADR-0003, and ADR-0014's jury demoted the author's favourite fixture fact for exactly this
reason. The structural argument:

`gravity` means *which part of the source survives the crop.* For it to carry information,
some part of that crop must be **underdetermined** by the document. Under the two settled
decisions it never is:

- ADR-0013: the source is resampled to **exactly** the declared rect. The fit crops nothing
  — the whole source lands in `width` x `height`. There is no fit-crop for a gravity to
  bias.
- ADR-0012: `clip` is a **static frame-space rect**, absolutely positioned. The only crop is
  the aperture, and its position is four declared integers.

So "which part of the source survives" is `clip` ∩ (the rect at `x`,`y`,`origin`), mapped
back through an exact affine — fully determined, for **every** document, not merely for
these eight. `gravity` has no free parameter left to name. The fixture then corroborates:
`photo-06`'s `gravity:"top"` is a restatement of `clip:[0,0,1080,1300]` sitting at the top
of a rect at `y=0`; `handle-logo`'s `gravity:"center"` restates a clip identical to its rect,
where every gravity value is the same picture.

And a redundant field is not free. It is a **second fact that drifts from the first** —
ADR-0006's `sequence`-label mechanism and ADR-0005's stale-half objection. Move the clip
down and `gravity:"top"` is now a lie that renders correctly, so the document says two
things and the renderer honours one. That is ADR-0007's "a field the renderer cannot honour
is worse than no field," twice quoted in ADR-0012 and dispositive here.

**"Give it a new job" was tested and failed.** The two candidates: (a) position the drawn
extent inside the box when they differ — but the extent's position is `x`,`y`,`origin`, so
it is inert again, and it would be doubly inert under `cover` where by construction the
extent meets the box on the driving axis; (b) bias the *slack axis's* half-pixel/one-pixel
residual under floor — real but absurd, a nine-value keyword for at most 1 px, which
ADR-0013 already spent its whole rounding section making unnecessary by publishing one
integer. There is no third.

**The cost, stated honestly.** ADR-0013 and ADR-0014 each advertised "changes zero bytes of
the committed fixture" as evidence of additivity; this breaks that streak. Concretely:
8 elements edited, `verify.py:54` (`assert "gravity" in e and "clip" in e`) loses a
conjunct, `migrate.py:61`'s key order and `migrate.py:89`'s `align → gravity` mapping both
change, `CONTEXT.md`'s `gravity` glossary entry inverts from "defined with the `fit`
vocabulary" to "retired", and the migration README's row 8 is rewritten. Also worth
recording: `migrate.py:89` maps the ASS `\an` alignment into `gravity`, so retiring it means
the migration **discards** that provenance — which is correct, because the information was
re-expressed in `clip` and keeping two copies is the drift above, but it should be written
down rather than discovered.

**Keeping it costs more.** Under "keep", every author must decide a field that changes
nothing, `validate` must either check it against the geometry (a check that can only ever
fire on a file that renders correctly — pure alarm fatigue) or not check it (a field that
can be wrong forever, silently), and the `align`/`gravity`/`origin` triple keeps three
near-synonyms alive in one schema with only two jobs between them.

**Strongest counter:** ADR-0003's asymmetry bites harder than I have allowed. My structural
argument is conditional on two decisions that are *themselves* flagged as reopenable:
ADR-0013 explicitly couples its floor tiebreak to declared-rect-authoritative ("if that
decision is ever reopened, floor becomes geometrically forced again"), and ADR-0012 leaves
**"whether `clip` is keyframable"** open. Reopen either — sample isotropically, or animate
the aperture — and a real fit-crop reappears with a genuinely free parameter, and `gravity`
is instantly the field that names it. Retiring is cheap now (8 fields) and expensive to
undo (a schema value, a migration, and every file written in between). The cautious answer
— keep it, forbid it on text and shape per ADR-0014, and record the inertness — costs
nothing today and preserves the option. I reject it because a field that is inert *by
construction* is not an option being preserved, it is a stale second copy accruing drift
for the duration of the option; but this is the question on which I would most want a
second measurement, and I would accept "retire it, and name the reopening condition
explicitly — *a crop whose position is not fully declared*" as a friendly amendment.

---

## Q6

**Decision:** Promote to **`error`**, with this predicate. For an image element whose `fit`
is `cover` or `contain`, with source `sw x sh` probed from disk and box `(bw, bh)` per Q2:

1. Choose the driving axis by integer cross-multiplication, per the `fit` value (Q4).
2. **Driving axis: zero grace.** The declared extent on that axis must equal the box
   dimension **exactly**. Otherwise **`error`**.
3. **Slack axis: membership in the bracketing pair.** With `exact = s_slack * b_driving /
   s_driving` as an exact rational, the declared extent must be in
   `{ floor(exact), ceil(exact) }` — computed as `(a)//(b)` and `-((-a)//b)` in integers,
   which collapse to one value when `exact` is integral. Outside that pair: **`error`**.
4. **Inside the pair but not the rule value** (i.e. declared `== ceil(exact) != floor`):
   **`note`**, naming the canonical floor value. Not an error, and `fmt` may not rewrite it.
5. `fit:"declared"`: **not checked** at all, by construction.
6. Source unprobeable: **`UNCHECKED`**, counted in the summary line, never an error.

**On the committed fixture this breaks 0 of 8 elements — computed, against the four PNGs on
disk, not asserted.** All seven photos: source 1536x2720, box 1080x1300, width drives,
declared width 1080 == box 1080 (rule 2 passes), band `{1912, 1913}`, declared 1912 (rule 3
passes, rule 4 silent). `handle-logo`: source 800x800, box 68x68, band is the **singleton
`{68}`**, declared 68 — passes. Four sources, all probed, zero UNCHECKED.

**Why:**

*Severity.* ADR-0013 named the one blocker — *"it cannot ship while there is no way to
decline the fit"* — and Q3/Q4 supply `fit:"declared"`, a legal, cheap, one-token
declination that does **not** collide with the `scale` animation channel on any element.
That was the entire objection ("an error whose named replacement is wrong for 87.5% of the
corpus"), and it is now answered: the named replacement is `declared`, wrong for 0% of the
corpus. With the blocker gone, ADR-0006's charter applies directly — a rect that contradicts
its own stated premise given the media on disk **is the document contradicting itself** —
and ADR-0013's own framing of the defect ("swap the file and the declared rect silently
means a different crop forever, with every number in the file still looking right; nothing
else in the tool surface catches it") describes a `note` doing nothing about a defect
nothing else catches. A 23.6% anisotropic stretch that `render` proceeds over is the
strongest possible argument that `note` is the wrong level.

*Why the second shape, sharpened.* ADR-0013 preferred it and was right: a step count is not
scale-free (0.105% on 1912 vs 2.941% on 68, a 28x spread), and a *percentage* tolerance is
no better — it would have to be tuned, and any tuned constant is the "self-consistent guess
no validator can falsify" that ADR-0005's `speed` bequeathed. `{floor, ceil}` needs **no
constant at all**: it is derived per element from the exact rational, and it
auto-tightens where it should. `handle-logo` demonstrates this for free — its exact value is
integral, so the band is a singleton and the tolerance is **zero**, which is exactly right
for the element where 1 px is 1.5%. The 28x spread is not mitigated by the rule; it is
dissolved by it.

*Why zero grace on the driving axis.* It is exact by construction — ADR-0013 says so, and
its whole integer-arithmetic section exists because a ULP on that axis is a real bug. There
is no legitimate rounding that lands a driving axis off the box dimension, so any deviation
is either a hand edit or a float implementation, both of which the file should refuse. This
term is also the one that has teeth only under Q2's box = clip reading; under box = rect it
is a tautology, which is the coupling between the two answers.

*Why ±1 on the slack axis is `note` and not `error`.* ADR-0013 is explicit that **both floor
and ceil cover**, and that floor won on three non-geometric tiebreaks. A file at 1913 is
therefore wrong-by-convention, not self-contradicting; erroring on it would exceed ADR-0006's
charter and would punish an implementer for a coin-flip the ADR itself admits was a
tiebreak. And `fmt` may not fix it (ADR-0013), so a `note` naming the canonical value is the
only available instrument. It fires 0 times on the fixture, so it costs nothing per run.

*The structural property that is lost, stated rather than glossed.* ADR-0013 observes: *"the
error-severity check needs no media, so an unprobeable source can never suppress an error."*
This error **does** need media, so that property no longer holds of the whole error class.
The mitigation is the asymmetry ADR-0013 itself published: `render` must decode the source
to draw it, so at render time the check is always answerable — and `render` is the only true
chokepoint ADR-0006 was able to identify. `validate` may return UNCHECKED; `render` never
can. The summary-line UNCHECKED count is what stops a validate-only `0 errors` reading as
proof.

**Strongest counter:** Two distinct failures, both real.

First, **the escape is load-bearing and untested by anything.** `fit:"declared"` exists in
zero committed elements, so the day this error starts firing is the first day anyone writes
the value that silences it — and the predictable agent response to any error is to silence
it, not to investigate. Under that behaviour the promotion converts a stale-source defect
into a `"fit":"declared"` edit that renders the identical wrong crop with the check now
formally declined, and `validate` reports `0 errors` on a file that is worse than before
because the premise has been retracted rather than fixed. Nothing in this design prevents
that; at most, `validate` can census the `declared` count so it is visible.

Second, **the media-dependence inverts an author's expectations at the worst moment.** An
error that fires because a file on disk changed — with no edit to the project, and possibly
in a shared or CI checkout where the image is a symlink, an LFS pointer, or a re-export
someone else made — refuses a render that worked yesterday, on a document nobody touched.
ADR-0006's `review` level exists for precisely "legal, renders, and you must look at a frame
to know if it was meant," and a re-exported source is arguably its textbook case. A juror
who ruled **`review`, not `error`** would be reading ADR-0006's own table more faithfully
than I am, and would keep ADR-0013's no-media-needed-for-errors property intact. My answer
is that a 23.6% stretch is not "you must look at a frame" but "this is guaranteed wrong
against its own stated premise" — but the gap between those two readings is one sentence
wide, and the fixture, which breaks 0 of 8 under either, cannot arbitrate it.

---

## Anything the questions missed

**1. A keyframed `clip` destroys the box.** ADR-0012 leaves "whether `clip` is keyframable"
open, and #22's wipes and reveals are exactly a keyframed aperture. If the box is
`clip[2..3]` (Q2) and `clip` animates, `fit` names a derivation with **no single box**, and
the Q6 predicate has nothing to evaluate against. This must be settled in the same breath:
either the fit claim is evaluated against the clip at the element's `start` (stated, not
inferred), or a keyframed `clip` with `fit` other than `declared` is a schema error. Left
open, two implementers will pick differently, which is ADR-0011's two-different-videos
failure.

**2. "The source's dimensions" is not yet defined, and it is an ADR-0005-`speed`-shaped
hole.** The Q6 predicate consumes `sw x sh` and the format has never said what those are.
JPEG EXIF orientation (a 90° rotation flag swaps them), non-square pixel aspect ratio in
video, and SVG (no intrinsic pixel size at all) each give two defensible answers, and two
implementations that disagree produce a spurious `error` on a correct file — the very
outcome the integer-arithmetic section was written to prevent. It must be pinned:
**decoded, orientation-applied, square-pixel dimensions**, and `validate` must print them
(ADR-0013's example already does: *"(1536x2200)"*), so the disagreement is visible in the
finding rather than silent.

**3. `mask` must be declared inert with respect to the box.** `handle-logo` carries
`mask:"circle"` (#22's). Nothing says whether a mask participates in the fit box. It must
not — otherwise the box becomes shape-dependent and the one-line rule forks per mask type.
One clause, cheap now.

**4. `fit` on future `video` elements, and the dimension-stability assumption.** ADR-0003
obliges primitives not to be shaped so a general need becomes hard to add, and a `video`
element is the most certain future addition. The fit rule transfers unchanged *provided*
the source's dimensions are constant over its duration — which is not true of every
container. Either the rule is stated against the first frame's dimensions, or a
varying-dimension source is UNCHECKED. Deciding it while `fit` has one element type is
cheap; deciding it after is a schema change.

**5. `validate` must print the rule value, and that obligation should be normative here.**
ADR-0014 established that required-and-computable is only affordable if the tool computes
it (`measure` returns the derived height). Making `fit` required **and** an error without
the corresponding obligation — the finding names the declared extents, the rule extents, the
probed source dimensions, and the distortion ratio, per ADR-0013's exemplar line — leaves the
author doing integer arithmetic by hand, which is where divergence starts.
