# `frame --crop` and true scale — the evidence behind ADR-0101

Resolves [#402](https://github.com/MBehtemam/Montagent/issues/402), and carries
[#405](https://github.com/MBehtemam/Montagent/issues/405)'s numbers to the code sites that
publish them.

Everything below is re-derivable:

```
cargo build --bin montagent
python3 docs/research/crop-true-scale/check_crop_true_scale.py
```

Twelve claims, four groups, exit non-zero when one stops holding. Groups A–C need the debug
binary and Pillow; group D's arithmetic runs in a bare checkout.

## What #402 filed, and what reproduced

`frame --at 50000 --crop 48,1300,984,340` on `fixtures/en-halloween-decorating` came back
**492×170** for a 984×340 region. That reproduced exactly. `--full` returned 984×340, also as
filed. The doc string at `crates/montagent/src/mcp.rs` promised the region *"at true scale"*,
and the coordinates were true scale while the picture was not.

## It was specified this way, and tested

`crates/montagent-core/tests/frame.rs` already held the old behaviour under
`crop_returns_the_requested_region`, with a comment explaining it: *"the region is stated in
frame space at true pixels, and the default half scale applies to it like any other extent — so
an 800×600 crop comes back 400×300."* So #402 is a **decision reversal**, not a slipped
predicate. The mistake in that sentence is reading ADR-0011's default as a property of every
extent, when its own justification makes it a token discipline over the whole canvas. The test
is reversed under its own name with that history kept in the comment.

## Two things the issue got wrong, and they pull in opposite directions

**It is not silent.** The issue's headline — *"silently produced plausible-looking garbage
rather than erroring, which is the worst failure mode for a QA tool"* — does not hold.
`crates/montagent-core/src/text.rs` prints, unconditionally, in the plain-text form the
reporting agent was reading:

```
FRAME  at 50000 — png, half scale, 492x170 from a 1080x1920 frame
  region      48,1300 984x340
```

Both numbers, adjacent, in one block: the served `492x170` and the asked-for `984x340`, plus
the word `half`. `Picture` carries `scale`, `width`, `height`, `rasterized` and `region`, and
all five reach the caption. So the failure was not a missing disclosure. **A promise in the
tool description outranked a fact in the answer**, because the agent formed its belief about
the output before the call and did not revise it after.

That is worth naming because it is the exact inverse of the failure map #395 was built around,
where an agent's *silence* was read as *coverage*. Here *disclosure* was read as *noise*. It
is recorded in ADR-0101 as the discovery mechanism, not as the thing repaired — a remedy aimed
at "descriptions outrank answers" has no testable acceptance criterion, and the defect below
survives a perfectly attentive reader.

**Its mechanism is wrong, and the correction matters more than the first point.** The issue
attributed a 573 → 10514 distinct-colour blowup to *"downscale resampling multiplies
colours"*. That comparison changed two variables at once — a PNG true-scale crop against a
JPEG half-scale one. Separated over the same region:

| | dimensions | distinct colours |
| --- | --- | --- |
| PNG, true scale | 984×340 | **523** |
| PNG, half scale (pre-fix, committed in `before/`) | 492×170 | **878** |
| JPEG, true scale | 984×340 | **14,090** |

Resampling does fabricate colour — 878 distinct values on a quarter of the pixels that held
523 — but it is the **small** term, +355. **Encoding is the large one, +13,567 at identical
dimensions**, and JPEG at *true* scale is sixteen times worse than resampling at half scale.

So **true scale alone does not make `--crop` fit for pixel analysis.** The flag's own doc
string must say `--png` too, or it repeats #402's defect in a new place: a promise the output
does not keep.

### One scope limit on the resampling claim

It is about **flat ink**, not images. On the photographic whole frame the direction inverts —
1080×1920 holds **210,827** distinct colours and 540×960 holds **91,177** — because averaging
a photograph destroys more colours than edge-blending invents. The check asserts this
inversion so the evidence can never be cited as *"downscaling always adds colours"*. It
fabricates on regions of flat ink under text, which is exactly the region class an agent
inspects to judge a card's fill — and exactly the planted `sentence-08` defect that made map
#395's trial agents reach for `--crop` in the first place.

## What changed

One predicate in `crates/montagent-core/src/verbs/frame.rs`: a region is served at true scale
whether or not `--full` was written. Measured after:

- `--crop` alone → `full scale, 984x340`.
- No crop, no `--full` → `half scale, 540x960`. ADR-0011's default is untouched.
- `--crop --full` → **byte-identical** to `--crop`. Redundant, and left legal: it is the only
  spelling that reached true scale before. How long the double-downscale has been known, and
  how reflexive `--full` already is, shows in this repo's own
  `docs/research/contact-sheet-legibility/make_sheets.py` — *"at true scale (see FINDINGS.md on
  #402: `--full` is required, or every tile is downscaled twice)"*. That script passes `--full`
  on whole frames rather than with a region, so it evidences the habit, not the pair.
- `--crop --png` → **pixel-identical** to cropping a `--full --png` whole frame externally.
  That is precisely the workaround #402's agent fell back to after abandoning the flag, so the
  fix is measured against the thing it replaces rather than against itself.

## The whole-frame-crop consequence, answered rather than waved at

If a region implies true scale, `--crop 0,0,1080,1920` reaches true scale without `--full`.
Measured: it is **byte-identical to `--full`**, so it costs exactly what `--full` costs — 2691
visual tokens on the high-resolution tier. It is `--full` spelled longer, not a discount, and
the caption still says `full scale`. The half-scale default was a **default, not a quota**, and
nothing is concealed. No refusal was added.

## The token figures these doc strings now carry (#405)

ADR-0097 corrected ADR-0011's flat `2691` to a two-tier fact; #405 owned the four code sites
still publishing the flat version. They are corrected in the same pass, because they are the
same two sentences a `--crop` amendment has to edit:

| | standard tier | high-resolution tier |
| --- | --- | --- |
| 540×960 | served as-is, **700** | served as-is, **700** |
| 1080×1920 | downscaled to 819×1456, **1560** | **2691** |

`819×1456 → 1560` is **cited, not recomputed.** The standard tier's downscale is not a
published long-edge rule — a long-edge cap at 1568 would give 881×1568 and 1792 tokens — so
`check_crop_true_scale.py` deliberately declines to reimplement it and leans on
`docs/research/visual-token-cost/`, which measured it. Re-deriving a rule nobody published
would be a number this repo does not stand behind.

## What this does not decide

[#406](https://github.com/MBehtemam/Montagent/issues/406) — whether the contact sheet crops
every tile, and what a cropped sheet must disclose — is map #395's, and is only *unblocked*
here. ADR-0101 gives it one sentence to spend: the rule is about a **single-frame** region, and
ADR-0097 already parked `--crop` with a range as not-yet-legal, so nothing is left
invocable-but-undefined by declining to answer it.

## Files

- `check_crop_true_scale.py` — the twelve claims above, re-executable.
- `before/region-png-half-scale.png` — the 492×170 answer as filed, from the pre-fix binary.
  Committed because the fix makes it unreachable, and group C measures against it.
- `before/region-png-true-scale.png` — the 984×340 arm, for the same comparison.
