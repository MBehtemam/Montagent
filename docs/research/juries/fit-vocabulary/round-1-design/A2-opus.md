# Juror A2 — #48, the `fit` vocabulary

Every number below was computed against
`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json` and the PNG
headers on disk (`images/0{5,6,7,8}.png` = 1536x2720, `brand/logo-en.png` = 800x800),
not recalled from the ADRs.

## Q1

**Decision:** `fit` is a **derivation claim** — an assertion about how `width`/`height`
were computed from the source — and it is **inert at render time**. Its consumers are
`validate`, `render`'s printed findings, and any tool that *rewrites* an extent
(`measure`, a future re-fit command). It is not a layout mode.

**Why:**

Run the render pipeline for `photo-06` and ask which fields are read. ADR-0013 settled
that the declared rect is authoritative: the source is resampled to exactly
1080x1912, placed at `x`,`y` under `origin`, and painted through `clip`. That is
`x`, `y`, `origin`, `width`, `height`, `clip`, plus the transform channels. **`fit` is
not in that list, and nothing it could say would change a pixel** — if it disagreed with
the rect, the rect wins by ADR-0013's own decision. A field the renderer reads and cannot
act on is not a render instruction; it is a comment that happens to be machine-checkable.

That is not a demotion. It makes `fit` the only field in the format that binds the
document to the *media on disk* in the geometry axis, and ADR-0006's organising quote is
exactly this: *"Only the checks that compare it to something outside it — the media, the
previous version, a rendered frame — find anything."* `width`/`height` alone are
mutually consistent with any source; `fit` is what makes them falsifiable. It is also the
input a repair tool needs: after a source is swapped, `fit` is the only thing in the file
that says what the new extent should be, so an author can be handed `1546` rather than
having to re-derive it from a dimension the document does not contain.

It has a **second consumer that needs no media at all**, which I argue in Q4: it selects
the direction of the rect-versus-`clip` inequality that ADR-0013 ships as an unconditional
`error`. Today that error is hard-coded to the cover direction because `cover` is the only
value; the moment a second value exists, `fit` is its parameter.

So the honest one-line gloss for the schema is: *`fit` states the rule by which
`width` and `height` were derived from the source's dimensions. The renderer does not
consult it; `validate` checks it.*

**Strongest counter:** if `fit` changes no pixel, ADR-0007's twice-ruled *"a field the
renderer cannot honour is worse than no field"* points straight at deleting it — the same
blade I take to `gravity` in Q5, and I should explain why it does not cut here. The answer
is that the two fields fail differently: `gravity` is *contradicted* by fields that do
render (it is a second, silent home for the crop offset that `x`/`y` already own), whereas
`fit` is *unrepresented* in any rendering field — no other field in the document asserts
the source relationship, so removing it removes a fact rather than a duplicate. But the
counter still lands on the naming: a reader coming from CSS `object-fit` will read `fit`
as a render instruction, and the fixture's `"fit":"cover"` sitting next to `"clip"` invites
exactly that. A more honest spelling (`fitted_by`, `extent_rule`) was rejected only because
renaming rewrites 8 of 8 committed elements for a gloss the schema can carry in prose —
which is a cost argument, not a correctness one.

## Q2

**Decision:** the box is **`clip`'s `w`,`h`** — the element's aperture. When `clip` is
absent the box falls back to the element's own declared `width`,`height`, under which the
rule degenerates from a derivation into a **fixed-point aspect assertion**: *the declared
rect is the source's aspect ratio, floored*. `fit` is therefore always well-defined and
never needs a presence check.

**Why:**

ADR-0012 states it in prose while settling `clip`: *"`photo-06` is a 1536x2720 source in
a **1080x1300** box under `fit:"cover"`"*, and its 612.5 px spill figure is
`1912.5 - 1300` — arithmetic that is meaningless unless the box is the clip. But prose is
not proof, so I checked the numbers, and the two elements the brief points at do
discriminate — just not in the way I first expected.

Both readings *pass* on both elements, which is the trap:

| element | clip w,h | declared w,h | box=clip gives | box=rect gives |
| --- | --- | --- | --- | --- |
| `photo-06` | 1080x1300 | 1080x1912 | 1080x1912 | 1080x1912 |
| `handle-logo` | 68x68 | 68x68 | 68x68 | 68x68 |

`photo-06` agrees because width drives under either box (`1080*2720 = 2937600` versus
`1300*1536 = 1996800` and versus `1912*1536 = 2936832` — width wins both). So *numeric
agreement decides nothing*, and any argument that stops at "both give 1912" has not
looked.

What discriminates is the **relationship between `clip` and the rect across the two
elements**:

- `handle-logo`: `clip` is `[478,96,68,68]` and the rect is 68x68 at (478,96) — the
  aperture is **exactly** the rect. 800x800 into 68x68 is an exact aspect match, so cover
  overflows by nothing.
- `photo-06`: `clip` is 1080x1300 and the rect is 1080x1912 — the aperture is **strictly
  smaller on one axis only**, by exactly the 612 px that cover overflows by.

That is the signature of clip-as-box and of nothing else: *the clip equals the rect
precisely when the source aspect equals the box aspect, and is smaller on exactly the
slack axis otherwise.* Under box-equals-rect that correspondence is a coincidence with no
generator. Under box-equals-clip it is forced.

The second, independent argument is that box-equals-rect makes the rule **circular**: its
input is its own output. ADR-0013's framing is that a fitted extent *"is typed by a human
or an agent before any renderer runs"* — which requires an input the author already holds
before typing the rect. The author holds the design slot (1080x1300, the photo area above
the cream band). They do not hold 1912; that is what they are computing.

**On absence.** `clip` is on 8 of 8 images but no ADR requires it, and ADR-0012 notes text
and rect elements need none. Three options were live:

1. **`fit` with no `clip` is a schema error naming `clip`.** Rejected: it makes `clip`
   required on every image by implication — the creation-by-implication move ADR-0013
   refused — and it outlaws the ordinary full-bleed uncropped image, which needs no
   aperture at all.
2. **`fit` with no `clip` is inert.** Rejected as the worst option available: it is
   ADR-0006's *"an omitted field is indistinguishable from a decision not to check"*
   reached by deleting a *different* field. Drop `clip` and the fit check silently stops
   running.
3. **Fall back to the declared rect** — adopted. One sentence, no new required field, and
   the rule keeps an input in every case.

Under (3) the check is not vacuous even though it is self-referential. It asserts
`height == (sh * width) // sw` (or the transpose, per the driving axis), which **fails**
for a 1536x2720 source in a 1080x1080 rect and **passes** for 1080x1912. It says exactly:
*this rect does not distort the source*. Under cover and contain alike, because with no
aperture there is no crop and the two collapse. Worth writing in the ADR as its own
sentence, because a reader will otherwise assume a self-referential rule checks nothing.

**Strongest counter:** the fallback quietly gives one field two meanings — with `clip`,
`fit` is a claim about a crop; without it, a claim about distortion. That is a mild case
of the `box` disease CONTEXT.md retired a word over, and an agent that deletes a `clip`
during an edit silently changes what the surviving `fit` asserts, with no diff hunk on the
`fit` line. The clean alternative is option (1), and its cost — `clip` required on every
image — is one line in the schema and zero bytes in the fixture. I chose (3) because
forbidding a full-bleed image is a real expressiveness loss and the format has no other
way to spell "the whole picture, uncropped"; but this is the closest call in my six
answers.

## Q3

**Decision:** `fit` is **required on every element carrying a `source` and a declared
rect** — today `image`, and the future `video` element inherits it. An omitted `fit` is a
schema error naming the value set. `fit` on a `text`, `rect` or `ellipse` element is a
schema error (no source), naming nothing — those types have no fit concept at all.

**Why:**

This is ADR-0014's text-box argument transposed one field over, and the transposition is
tighter than the original, because `fit`'s *only* job is to enable a check.

> An omitted `height` is indistinguishable from a decision not to check.

For `height` that was an inference about intent. For `fit` it is a tautology: `fit`
produces no pixels (Q1), so the only thing an author loses by omitting it is the check. An
optional `fit` is therefore **precisely** the `sequence` label ADR-0004 rejected and
ADR-0006 spent itself defeating — an opt-in check, tagged per element, where silence means
nothing. ADR-0006's one structural defence is that `validate` checks *uniformly*, so a
clean run means something; an optional `fit` spends it.

The census is one-sided in the same way ADR-0014's was. The stale-source defect class —
swap the file, every number still looks right — is caught on 8 of 8 elements if `fit` is
required and 8 of 8 *today* if it is optional, but the moment a ninth element is written
by an agent that did not know the field existed, required catches it and optional does
not. And the defect is silent by construction: ADR-0012's *"it fails quietly because a
centre-cropped photo looks plausible."* A default is only safe where its wrong answer is
loud; this one's wrong answer is a plausible crop.

There is no defensible default value either. Defaulting to `cover` would make the check
fire on files whose author never made the claim — manufacturing the ADR-0006 alarm fatigue
that killed the *"emit a finding when `height` equals the derived value"* third option in
all eight of ADR-0014's sessions. Defaulting to the escape value (`declared`) is worse: it
is a default that means *do not check*, which is the opt-in failure with an extra step.
Required is the only option with no silent branch.

**Cost: zero bytes.** `fit` is present on 8 of 8 image elements, so requiring it changes
nothing committed. `verify.py:54` already asserts `gravity` and `clip` on images and gains
a `fit` clause.

**Strongest counter:** required-and-computable is only affordable if a tool computes it —
ADR-0014's own caveat about the text height. For `height` the inputs are on the same line;
for `fit` the author must *choose a semantic*, and the honest failure is an agent that
writes `"fit":"cover"` because it is the value in every example, on an element where
`declared` was true. Then the field is required, always present, and systematically
wrong — worse than absent, because `validate` now reports confident nonsense and the
error I ship in Q6 refuses the render. The mitigation is thin: the error message names
`declared` and the note names the ratio, so the first wrong `cover` is loud. But
"required fields get cargo-culted" is a real prediction and nothing here measures it.

## Q4

**Decision:** three values, closed: **`cover`**, **`contain`**, **`declared`**.

| value | inequality (rect vs box `bw x bh`) | driving axis | rounding |
| --- | --- | --- | --- |
| `cover` | `w >= bw` **and** `h >= bh` | width drives iff `bw*sh >= bh*sw` | driving axis verbatim; slack axis **floor** |
| `contain` | `w <= bw` **and** `h <= bh` | width drives iff `bw*sh <= bh*sw` | driving axis verbatim; slack axis **floor** |
| `declared` | *none* | n/a | n/a — no extent is derived |

All comparisons are exact integer cross-multiplication, never a float ratio, per
ADR-0013's 4.466% disagreement measurement.

**Why:**

**`cover` is ADR-0013 verbatim** and is restated here only so the vocabulary is readable
in one place. The slack axis satisfies `s*f >= b` exactly, and flooring a real number that
is `>= b` for integer `b` leaves it `>= b`, so floor preserves the inequality. `1912.5 →
1912 >= 1300`. Driving axis takes the box dimension exactly, on which floor is the
identity.

**`contain` ships in v1.** ADR-0003's asymmetry makes "the fixture does not use it" no
argument at all, and the positive case is the reference class: fitting a whole picture
inside a slot with the background showing at the edges is the single most ordinary image
operation in CapCut and Premiere after cover. ADR-0014 admitted `ellipse` and `radius` on
exactly this reasoning — *"admitting it now costs one clause where admitting it later is a
schema change"* — and ADR-0013 explicitly chose floor partly to keep **one rounding
operation across all fit values** the day contain lands. Deferring collects that debt
without spending the credit.

Its inequality inverts, and so does the axis selection: under contain the *smaller* ratio
wins, so width drives when `bw*sh <= bh*sw`. Floor is now doing real work rather than
being a tiebreak — `s*f <= b`, and flooring keeps it `<= b`, where ceil would break it.
This is ADR-0013's *"direction-preservation selects floor uniquely only for `contain`"*,
now cashed.

**The tie is benign in both.** When `bw*sh == bh*sw` the source and box aspects match
exactly, both axes yield the identical rect, and the `>=`/`<=` difference between the two
selection rules is invisible. `handle-logo` is exactly this case (`68*800 == 68*800`), so
the fixture cannot exercise the selection rule at all — worth saying out loud, since a
reader will otherwise take `handle-logo` as evidence for the tiebreak direction.

**`contain` costs one amendment to ADR-0013, which must be paid explicitly.** ADR-0013
ships *"the declared rect must contain `clip`"* as an unconditional `error`. Under contain
the rect is *inside* the box, so with box = clip that error fires on every correct contain
element. The fix is not to weaken the error but to **parameterise it by `fit`**, which is
what the field is for:

> The declared rect and the `clip` rect must satisfy the inequality `fit` names.
> Under `cover` and `declared`, the rect must contain `clip`. Under `contain`, `clip`
> must contain the rect.

This keeps ADR-0013's load-bearing structural property intact — *the error-severity check
needs no media, so an unprobeable source can never suppress an error* — because `fit` is
in the document. It passes 8 of 8 today unchanged. And it gives `fit` a consumer that runs
even under `UNCHECKED`.

**The escape value is `declared`, and the name matters more than it looks.**

- **`fill` is spent twice over.** ADR-0014 made `fill` the *colour* field on shapes, so
  one word would be a paint on a rect and a geometry mode on an image. And ADR-0005 killed
  *implicit `fill`* by name. Independently, CSS `object-fit: fill` means *stretch to the
  box ignoring aspect* — a derivation instruction, the opposite of declining to derive —
  so an agent reaching for CSS habit would write it meaning "distort to my rect" and get a
  value that in Montaget asserts nothing. Worst available candidate.
- **`none` is the trap.** CSS `object-fit: none` means *use the source's intrinsic size*.
  That is not "use my rect"; it is the natural-source-size default that ADR-0012 forbade
  outright and ADR-0005 killed for `fill`. A name that an agent decodes into the one
  reading the format has banned twice is disqualifying, and `none` also reads as "no
  value" — a spelling of absence, in a field where absence is now an error (Q3).
- **`stretch`** overclaims: the escape is also correct for an aspect-exact rect with a
  cropped source, where nothing is stretched.
- **`declared`** collides with nothing, and it is the format's own established word:
  ADR-0013 says *declared rect*, *declared extent*, *declared-authoritative*; ADR-0014 says
  *declared rect*, *declared geometry*. `"fit":"declared"` reads, correctly and without a
  glossary, as *the extents are the ones I declared and no rule produced them*. It also
  has no CSS homonym at all, which is a feature: there is no habit to mis-fire.

Under `declared`, the media-backed deviation check does not run — there is no claim to
contradict — while the document-only rect-versus-`clip` check still does. That split is
the point: the escape declines the claim about the source, not the claim about the
aperture.

**Excluded, deliberately:** `scale-down` (CSS; a conditional on top of contain, and a
conditional is not an inequality), `fill`/`stretch`/`none` (above), and any per-axis or
percentage form (an open syntax, which is what ADR-0014 rejected gradients and `path` for).

**Strongest counter:** `contain` is unevidenced in the only real project file, and ADR-0014
put `gradients` out of v1 as *"unevidenced rather than rejected"* on a very similar record —
so a consistent juror could park `contain` the same way. The cost of admitting it is not
one clause but **an amendment to an accepted ADR's only image-geometry `error`**, and
amending an error that currently passes 8 of 8, in order to serve zero elements, is a real
risk taken for a hypothetical. If the amendment is got subtly wrong, the damage lands on
the 8 elements that exist and not on the 0 that do not. The counter to the counter is that
the amendment is required by *any* second value, including the escape, so it is owed today
regardless — but that is weaker than it sounds, since `declared` alone could keep the
cover direction and need no inversion.

## Q5

**Decision:** **retire `gravity`.** `gravity` on any element is a schema error. On an image
the message names **`x`/`y` (with `origin`)**; on text and shapes it names `origin`, which
is the clause ADR-0014 already owed. The 8 occurrences come out of the committed fixture,
and this is the first ticket in this run that changes bytes.

**Why:**

ADR-0013's *"inert on 8 of 8"* is the weaker half of the case, and on its own it would not
move me — ADR-0003 forbids reading a fixture census as evidence against a capability, and
ADR-0014's round-one juror who argued *"all 10 shapes are rectangles, ship rect only"* is
the named failure I would be repeating.

The decisive argument is not a census. It is that **`gravity` is structurally
unimplementable under ADR-0013's render model**, and would be so on an empty fixture.

`gravity` means *which part of the source survives the crop*. Under declared-rect-
authoritative, **the source is resampled to the entire declared rect — no part of the
source is cropped at all.** Every pixel of `images/06.png` lands somewhere in the 1080x1912
rect. What is cropped is the *rect*, by `clip`, in frame space, at a position fixed by `x`,
`y` and `origin`. So the quantity `gravity` names does not exist in this model, and the
quantity it is reaching for — which part of the drawn rect the aperture admits — is already
a subtraction of two rects both fully stated in the document.

Worked, on `photo-06`: rect 1080x1912 at `y=0`, clip `y` 0..1300, so the top 1300 px of the
picture survives. That is `gravity:"top"` — *and it is already forced by `y=0`*. An author
wanting the centre writes `y = -(1912-1300)/2 = -306`. Wanting the bottom, `y = -612`.
`gravity` is `y` in a hat.

That makes it not merely inert but **the exact failure ADR-0012 legislated against twice**:
a second home for a fact that a rendering field already owns (*"it would also give position
two homes"*), and *"a field the renderer cannot honour"* — because if `gravity:"center"`
were ever honoured against `y=0`, one of the two would have to lose, silently, and ADR-0013
already ruled that the declared geometry wins. A field that can only ever lose its
disagreements is not a field.

Note the asymmetry with `fit`, which I kept in Q1 on the same test: `fit` asserts something
**no other field in the document asserts** (the relationship to the source on disk);
`gravity` asserts something **two other fields already determine**. Inert-and-unique is a
check. Inert-and-duplicated is drift.

**"Give it a new job" was considered and rejected.** The plausible new job is
aperture-positioning shorthand — `gravity:"top"` meaning "place the clip at the top of the
rect". That is a *second placement grammar* layered over `x`/`y`/`origin`, which is exactly
the class ADR-0014 closed when it rejected point lists: admitting one is *"a new ADR about
placement, not a schema addition."* It would also be a derived-position field whose value
must be kept in sync with two literal ones, which is the stale-half objection.

**The cost, stated and not minimised.** This breaks the zero-bytes streak that ADR-0013 and
ADR-0014 both asserted by script:

- **8 elements in the committed fixture change** — `"gravity":"top"` on 7 photos,
  `"gravity":"center"` on `handle-logo` (counted, not estimated, by grep on the raw file).
- **ADR-0012's published `photo-06` element and ADR-0013's are both reprinted**, since both
  carry `gravity:"top"` inline. That is the *"floor is a supplement; ceil is a correction"*
  distinction landing on the wrong side, and it must be admitted.
- **`migrate.py:89`** — `out["gravity"] = e.pop("align")` — becomes a *drop*, so the ASS
  `align` on images no longer maps to anything. That loses the only record that the source
  said `\an8`; the replacement is that the migration must emit the `y` that `align`
  implied, which for every fixture photo is the `y=0` already there.
- **`verify.py:54`** asserts `gravity` on images and inverts to assert its absence.
- **`CONTEXT.md`'s glossary sentence** *"On an image it is real, and it is defined with the
  `fit` vocabulary rather than here"* is falsified by this decision and must be rewritten to
  the retirement, alongside the existing `anchor`/`align`/`box` retirements it will sit with.

Keeping it costs less today and more forever: every agent that writes an image element
writes a field that does nothing, and the first renderer author has to discover by reading
ADR-0013 that they must ignore it.

**Strongest counter:** this is the one answer where I overturn committed prose and committed
bytes, and the honest reading of ADR-0003's asymmetry is uncomfortable — *the channel is
never evidence a capability is unneeded*, and "inert on 8 of 8" is doing more work in my
reasoning than I would like to admit even after I demoted it. A juror could hold that
`gravity` becomes live the instant `fit` is *not* a derivation claim — i.e. if
declared-rect-authoritative is ever reopened and the renderer samples isotropically, there
*is* slack inside the rect and `gravity` places it. ADR-0013 itself flags that coupling by
name for the ceil argument (*"void under declared-rect-authoritative and live under
isotropic sampling"*), and retiring `gravity` makes reopening that decision strictly more
expensive. Deprecating rather than erroring — accept and ignore for one version — would
hedge it, and I rejected that only because ADR-0011 requires `fmt` to normalise on write,
so an accepted-then-dropped field is the zero-hits exact-string trap again.

## Q6

**Decision:** **`error`**, code `E-FIT-DEVIATION`, with ADR-0013's second candidate
predicate, stated exactly:

Let the box be `(bw,bh)` per Q2 and the source `(sw,sh)` as probed. Choose the driving axis
by the integer cross-multiplication for the declared `fit` (Q4). Then the element is in
error unless **both**:

1. **Driving axis, zero grace:** the declared extent on that axis `==` the box dimension on
   that axis.
2. **Slack axis, one-step band:** the declared extent on that axis is a member of
   `{ (s_slack*b_driving) // s_driving , -((-s_slack*b_driving) // s_driving) }` —
   `{floor(exact), ceil(exact)}`, collapsing to a single value when `exact` is integral.

Under `fit:"declared"` the check **does not run**. When the source cannot be probed it is
`UNCHECKED` and counted in the summary line (`validate` only; `render` must decode, so the
check is always answerable there and `render` refuses on it).

A **`note`**, `N-FIT-CEIL`, fires when the declared slack value is in the band but is the
ceil rather than the floor — a fact, not a fault: it tells the reader that regenerating the
file will not reproduce it. It fires **0 of 8** times on the fixture.

**Why the severity can move now.** ADR-0013 blocked the error on exactly one thing: *"there
is no legal way to spell 'do not fit this, use my rect'"*, and the only escape was to put
the distortion in `scale`, which collides with the Ken Burns channel on 7 of 8 elements.
`fit:"declared"` is that spelling, and it is the *right* named replacement under ADR-0012's
own *"a schema error naming the replacement"* pattern — it touches no animation channel, it
is one token, and it is correct for 8 of 8 elements rather than wrong for 87.5% of them. The
23.6% stale-source case then has a legal terminus in both directions: fix the rect to 1546,
or declare `fit:"declared"` and mean it.

**Why this predicate and not "more than one integer step".** The step count is not
scale-free, and the scan reproduces the spread: the same 2-step deviation is **0.105%** on
`photo-06`'s 1912 and **2.941%** on `handle-logo`'s 68, a 28x range inside one file. But
the deeper reason — which ADR-0013 records as a preference without giving its principle —
is that **the grace band should be exactly the set of values the rounding tiebreak could
legitimately have produced, and nothing wider.** ADR-0013 says plainly that for `cover`
both floor and ceil are geometrically safe and that floor won on three non-geometric
tiebreaks. So a hand-written 1913 is not a defect; it is the rejected-but-safe branch of an
admittedly arbitrary choice. Erroring on it would refuse a render over a coin-flip — and
`fmt` is forbidden from repairing it (ADR-0013: `fmt` may never rewrite a declared extent),
so the author would be stuck hand-editing to satisfy a tiebreak. Meanwhile the driving axis
has **no tiebreak at all** — it takes the box dimension verbatim and is exact by
construction — so any deviation there is a genuine self-contradiction and gets zero grace.
The band is derived from the rule's own uncertainty rather than picked as a threshold, which
is why it needs no percentage and no scale.

**The fixture count, computed.** I evaluated the predicate over all 8 image elements against
the real PNG headers on disk:

| element | box | source | driving | declared driving | slack band | declared slack | result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `photo-05-intro` | 1080x1300 | 1536x2720 | w=1080 | 1080 | {1912,1913} | 1912 | pass |
| `photo-05` | 1080x1300 | 1536x2720 | w=1080 | 1080 | {1912,1913} | 1912 | pass |
| `photo-06` | 1080x1300 | 1536x2720 | w=1080 | 1080 | {1912,1913} | 1912 | pass |
| `photo-07` | 1080x1300 | 1536x2720 | w=1080 | 1080 | {1912,1913} | 1912 | pass |
| `photo-08` | 1080x1300 | 1536x2720 | w=1080 | 1080 | {1912,1913} | 1912 | pass |
| `photo-05-quiz` | 1080x1300 | 1536x2720 | w=1080 | 1080 | {1912,1913} | 1912 | pass |
| `photo-05-loop` | 1080x1300 | 1536x2720 | w=1080 | 1080 | {1912,1913} | 1912 | pass |
| `handle-logo` | 68x68 | 800x800 | w=68 | 68 | {68} | 68 | pass |

**0 of 8 elements break.** `N-FIT-CEIL` also fires 0 times, since all seven photos are the
floor and `handle-logo` is exact. Note that the corpus is still **two geometries counted
eight times**, so this zero is a safety check, not evidence the predicate is well-calibrated
— exactly as ADR-0013 said of its own fixture pass.

The finding's text keeps ADR-0013's shape unchanged, only the level moves:

```
error  E-FIT-DEVIATION  photo-06: declared 1912; cover from images/06.png (1536x2200)
       is 1546; 23.6% anisotropic stretch. Write 1546, or fit:"declared".
```

Naming the escape in the message is not decoration — it is the clause that makes the error
satisfy ADR-0012's replacement-naming pattern, and it is what the 23.6% case had no way to
say before.

**Strongest counter:** the escape hatch is what makes the error safe, and it is also what
makes it toothless. An agent that hits `E-FIT-DEVIATION` and cannot work out why has a
one-token way to silence it, and `fit:"declared"` will be written far more often as
*"make the error go away"* than as *"I mean this rect."* Then the stale-source defect —
the entire justification for the check — ships anyway, now with the file affirmatively
stating that the wrong rect was intended, which is worse than silence because the next
reader believes it. ADR-0013's `note` has no such failure mode: it cannot be silenced, so
it cannot be silenced *wrongly*, and it still prints at render. That is a real argument for
leaving the severity alone, and the only answer I have is that ADR-0006 already made this
trade — `render` refuses on `error` precisely because notes get skimmed — and that an
error whose suppression is a visible, greppable, diffable token in the source of truth is
strictly better than one suppressed by omission.

## Anything the questions missed

**1. `fit` is unowned in the one place it would do the most work: the repair.** Every
answer above treats `fit` as read-only. But the reason to keep a derivation claim in a file
is that a tool can *re-run* it — swap `images/06.png` for a 1536x2200 export and the
correct new extent, 1546, is computable from `fit` + `clip` + the new header. Nothing in
the tool surface (ADR-0011) is allowed to write it: ADR-0013 forbids `fmt` from touching a
declared extent, and `validate` states facts and never repairs. So the format now has an
`error` (Q6) whose fix is mechanical, fully determined, and performable by no tool —
leaving an agent to hand-type an integer that a one-line function knows. ADR-0014 hit the
identical shape and solved it (*"`measure` also returns the derived height, so an author can
fill the required field without hand-arithmetic — required-and-computable is only
affordable if the tool computes it"*). The image axis needs the same: **`measure` on an
image element should return the fitted extent**, which is a read-only capability and
therefore violates nothing. Without it, Q3's required field and Q6's error are
required-and-computable with nothing computing.

**2. Nothing in the six questions asks what `fit` means for a `video` element**, and
ADR-0003 makes that a real gap rather than a hypothetical — *the reference class is CapCut
and Premiere*, both of which are video editors first, and the fixture's 8 still images are
explicitly not evidence that video is unneeded. A video source has per-frame dimensions,
which are constant in practice but not in the document, and a variable-resolution source
would make the fit claim true at one instant and false at another. My answers extend
unchanged if the probe reports one dimension pair, which is the sane reading — but it is
an assumption I am making silently and it should be written down, not inferred, per
ADR-0014's own closing note about unflagged presuppositions.

**3. The `clip`-keyframability question ADR-0012 left open collides with Q2.** If `clip`
ever animates — *"a wipe or reveal is exactly a keyframed aperture"* — then the box is a
function of time and `fit` has no single input. Under my Q2 answer the fit rule would have
to name an instant, and the obvious candidate (the first keyframe's value) is the sort of
implicit convention ADR-0005's `speed` failure was made of. This ticket should either
state that `fit`'s box is `clip`'s value at the element's `start`, or record that a
keyframed `clip` and a non-`declared` `fit` are mutually exclusive. Neither ADR-0013 nor
#48 mentions it.
