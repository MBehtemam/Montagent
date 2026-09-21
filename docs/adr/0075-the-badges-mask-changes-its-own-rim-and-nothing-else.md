---
status: accepted
amends: 0068 (retires its "the rendered frame is unchanged" Consequences bullet and the
  "pixel-inert" sentence it rests on — a param-less `mask` selects every pixel of the
  badge and changes only the antialiasing of its own rim, measured at 112 pixels of a
  540×960 frame and a mean channel delta four orders below visible)
---

# The badge's `mask` changes the antialiasing of its own rim — "the rendered frame is unchanged" retires

**Ticket:** [#279](https://github.com/MBehtemam/Montaget/issues/279). Evidence:
`crates/montaget-core/tests/effects.rs::the_badges_mask_changes_only_the_antialiasing_of_its_own_rim`,
which re-derives every number below by rendering the committed fixture twice — with the
badge's `mask` and with it removed — at the instant and scale the committed golden is
taken at, and fails the suite the moment one stops reproducing.

[ADR-0068](0068-the-bare-mask-key-retires-masks-are-effects-members.md) migrated the
fixture's bare `mask` key to an `effects` member, and closed the argument for doing so
with a claim about the pixels:

> **The change is pixel-inert.** The asset's own alpha already produces the circle, `clip`
> equals the element rect, and the slot is square, so the mask selects every pixel the
> asset already shows.

— and, in its Consequences: *"The committed fixture, `migrate.py` and `verify.py` are
updated in this change. **The rendered frame is unchanged.**"*

[#214](https://github.com/MBehtemam/Montaget/issues/214) implemented the `mask` member.
The rendered frame is not unchanged.

## What the ADR measured, and what it did not

ADR-0068's evidence is `docs/research/juries/mask-spelling/decode_logo_alpha.py`, and it
is correct about what it measures. The **stored** 800×800 `brand/logo-en.png` has zero
opaque pixels outside its inscribed circle and zero transparent pixels inside it. The
badge is the inscribed circle, drawn into the asset.

The renderer does not paint the stored asset. `handle-logo` is a 68×68 slot, so
`montaget-render` resamples 800×800 down into it — an 11.8× bilinear minification with no
mipmaps, kept that way in `canvas::sampling` because the golden frames were measured with
it. A minification that steep carries a little of each boundary texel into the destination
pixels immediately *outside* the ideal circle. So the **drawn** badge reaches marginally
past the circle the **stored** badge does not, and a mask that is doing its job trims that
rim.

The inference the ADR made — the asset's alpha is already the circle, therefore the mask
selects every pixel the asset shows, therefore nothing moves — is sound in its first two
steps and false in the third. *Selects every pixel the asset shows* is a claim about the
source; *nothing moves* is a claim about the destination, and a resampler sits between
them.

## Decision

**ADR-0068's Consequences bullet *"The rendered frame is unchanged"* is retired, together
with the two sentences in its body that say the same thing — *"The change is pixel-inert"*
and, four lines later, *"Nothing about the rendered frame changes — which is the point"*.**
What replaces all three, and is what ADR-0068's argument actually needed:

> A param-less `mask` on `handle-logo` selects every pixel of the badge and changes only
> the antialiasing of its own rim. On the committed fixture at 400 ms, half scale: **112
> pixels** of 540×960 change, mean channel delta **0.0041**, SSIM **0.999982** — four
> orders below anything a reader sees, and inside the golden suite's own budget for a
> picture it calls unchanged.

Nothing else in ADR-0068 moves. The retirement of the bare key, the param-less form's
geometry, `effects`' position in key order, the advise-class classification and the
fixture migration itself all stand — none of them depends on the frame being byte-identical,
only on the mask selecting the whole badge, which it does.

## Why these numbers say what they say

**Every changed pixel is on the circle's own edge.** The two-band assertion in
`the_fixtures_badge_keeps_every_pixel_the_mask_selects` is the one that carries the
surviving half of the ADR's claim: nothing further than a pixel from the circle's edge
moves by more than the two-bit rounding of the extra premultiplied layer the effect
pipeline composites through, and six sampled interior points are identical bit for bit.
This ADR's own test adds the whole-frame view and asserts the complement — that no pixel
outside the badge's 34×34 half-scale slot changes at all, which a mask on one element
reaching another would violate.

**The magnitude is the same order as the noise the suite already budgets for.**
`tests/golden_frames.rs` admits SSIM ≥ 0.999 and a mean channel delta ≤ 0.5, on the ground
that the last bit differs across platforms —
[#34](https://github.com/MBehtemam/Montaget/issues/34) measured that drift at 0.003–0.004,
which brackets this change's 0.0041. The pre-#214 golden would have gone on passing
against the masked render — 0.0041 is inside the 0.5 ceiling and 0.999982 inside the 0.999
floor — which is the fact the next section turns on. The delta is real and it is not
visible.

**Looked at, the masked badge is better.** Its rim is an antialiased circle rather than
the aliased staircase the raw minification leaves. This is not why the decision goes the
way it does — it is worth recording because a reader who assumes a retired *"unchanged"*
means a regression would have it backwards.

## The goldens were regenerated, and that was disputed

#214 regenerated both fixture goldens and the diff was looked at before they were
committed. A reviewer argued they should not have been: the delta is far inside the
harness's own budget, both goldens pass untouched, and regenerating discards the pre-#214
pixel record for no measured gain.

**Recorded as a real disagreement rather than settled silently.** The counter-argument,
and the reason the regenerated goldens stand, is `golden_frames.rs`'s own statement of
what a golden is — *"the picture an agent actually receives"*. A committed golden that is
knowingly not the picture the code produces has stopped being that, and the next reader to
compare them would be reading a 112-pixel difference as drift to investigate rather than
as a decision already taken. The budget is there to absorb *unmeasured* platform noise,
not to hold a known difference out of the record.

## How this happened

**The same way ADR-0068's own diagnosis of ADR-0040 said it happened.** This is the second
clause of ADR-0068 to be falsified by a claim about the fixture that was never put to its
own court. The first was ADR-0040's *"no migration needed"*, which ADR-0068 diagnosed as
*"a deferral passed forward that the last holder dropped"* and whose Consequences bullet
*"was authored after the court, on a question the court was never asked"*.

The mask-spelling court was asked about the **spelling**. Its packet did not ask what a
renderer does with the pixels, and could not have answered it: at the time there was no
`mask` implementation to render through. The pixel claim was authored after the court, on
a question the court was never asked — the identical shape, in the identical document,
one section further down.

The generalisable part is narrower than "ADRs make claims they cannot check". It is that
**a claim about the rendered frame is not entailed by a claim about the asset**, and this
series has a standing way to tell them apart: `docs/agents/domain.md` requires a numeric
claim to carry a re-executable check. `decode_logo_alpha.py` is one — for the claim it
makes, about the stored PNG. No artifact was ever committed for the claim about the frame,
because none could be written without rendering, and the sentence went in anyway. The test
this ADR cites is the artifact that was missing.

## Consequences

- **ADR-0068 keeps its original words.** An ADR is amended, never rewritten
  (`docs/agents/domain.md`), so the replacement lives in ADR-0068's banner, which names
  this ADR and quotes all three sentences it retires — the body's two as well as the
  Consequences bullet, since a reader who passes the first and stops at the second would
  otherwise read the retired claim as current.
- A param-less `mask` whose element is resampled is **not** pixel-inert in general. The
  badge's case — every changed pixel on the shape's own edge, the interior bit-identical —
  is what the measurement supports, and it is a property of the mask selecting the whole
  subject, not of masks at large.
- The two fixture goldens stand as regenerated in #214, over a recorded objection.
- `the_badges_mask_changes_only_the_antialiasing_of_its_own_rim` is this ADR's
  re-executable check. It runs in the ordinary suite rather than by hand, unlike the
  `docs/adr/*.py` scans, because it needs the renderer. A failure means this ADR's prose
  cites numbers the code no longer produces, and the ADR needs an amendment rather than
  the test a fix.

## Not settled here

**The sampling filter.** `canvas::sampling`'s no-mipmap bilinear minification is what
produces the rim in the first place, and a better filter would shrink the delta this ADR
measures. Whether to change it is a rendering-quality question with the golden frames
downstream of it, and nothing here decides it.

**The `mask` member's full parameter set** remains where ADR-0068 graduated it — its own
ticket, undecided. This ADR measures the param-less form only.
