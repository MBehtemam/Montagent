---
status: accepted
amends: 0011 (a `--crop` is exempted from the half-scale default, so that default governs the whole canvas and nothing else; and the four code sites still publishing its flat single-tier token count are corrected to the two-tier fact already established for it)
---

# A `--crop` is served at true scale, and the half-scale default governs the whole canvas

[#402](https://github.com/MBehtemam/Montagent/issues/402), filed as a bug rather than as a map
ticket. It also discharges [#405](https://github.com/MBehtemam/Montagent/issues/405) at the
code sites, because both edit the same two sentences and #405 instructed that its numbers be
corrected before an amendment inherited them.

Every number below is re-derived by
[`docs/research/crop-true-scale/check_crop_true_scale.py`](../research/crop-true-scale/check_crop_true_scale.py)
— **twelve claims in four groups**, all passing as committed. The write-up is
[`docs/research/crop-true-scale/README.md`](../research/crop-true-scale/README.md).

## Decision

**A `--crop` comes back at the project's true pixel dimensions, whether or not `--full` was
written.** `--full` with a region is **redundant and stays legal**. The half-scale default is
unchanged for every uncropped call.

## Why

[ADR-0011](0011-tool-surface-reads-checks-renders.md) adopted half scale because *"full scale
genuinely costs 2691 tokens every time the agent looks"* — a token discipline over **the whole
canvas**. A region has already paid that discipline by asking for less of the frame. Halving it
again applies one budget twice, and it left the flag unable to do the single thing its own doc
string offers it for: *"look closely at one card without paying for the whole canvas."*

`frame --at 50000 --crop 48,1300,984,340` answered **492×170** for a 984×340 region, exactly as
filed. The coordinates were true scale; the picture was not.

**This is a reversal, not an oversight.** The halving was specified and asserted: the test
`crop_returns_the_requested_region` read *"the region is stated in frame space at true pixels,
and the default half scale applies to it like any other extent — so an 800×600 crop comes back
400×300."* Someone wrote that sentence deliberately, and it is wrong about what ADR-0011's
default is *for* — it reads the default as a property of every extent rather than a discipline
over the canvas. The test is reversed here under its own name, with that history in a comment,
because a suite that silently changes its mind is how the next reader loses this argument.

## What this ADR does not repair, and why that was the harder call

#402's headline was that the behaviour was **silent** — *"silently produced plausible-looking
garbage rather than erroring, which is the worst failure mode for a QA tool."* **That does not
hold.** The plain-text answer the reporting agent was reading prints, unconditionally:

```
FRAME  at 50000 — png, half scale, 492x170 from a 1080x1920 frame
  region      48,1300 984x340
```

The served dimensions, the asked-for region and the word `half`, adjacent, in one block.
`Picture` carries `scale`, `width`, `height`, `rasterized` and `region` and all five reach the
caption. **A promise in the tool description outranked a fact in the answer**, because an agent
forms its belief about a call's output before making it and does not necessarily revise it
after.

That is the exact inverse of the failure [#395](https://github.com/MBehtemam/Montagent/issues/395)
was built around, where an agent's *silence* was read as *coverage*. Here *disclosure* was read
as *noise*, and it is worth having on the record precisely because this project has just bet
heavily on plain-text disclosure —
[ADR-0097](0097-the-range-is-from-to-on-both-surfaces-and-the-caption-becomes-an-attribution-obligation.md)
§6 requires the no-`--json` path to carry the disclosure in full.

**It is recorded as the discovery mechanism and not as the thing repaired.** Two independent
arguments decided that:

1. **The defect survives a perfectly attentive reader.** Ask whether `--crop` is still wrong
   once the caption is read correctly. It is: the region is not the pixels the flag exists to
   deliver. A fault that stands after the reader is assumed attentive is a fault in the
   behaviour, not in the disclosure's rank.
2. **A remedy aimed at "descriptions outrank answers" has no testable acceptance criterion.**
   No change scoped to `frame`'s surface makes an agent re-read a caption it has already
   decided to skip. The durable cure is to not publish a promise the answer contradicts, which
   is this decision plus honest doc strings.

## The correction #402 needs, which is larger than its headline

The issue attributed a 573 → 10514 distinct-colour blowup to *"downscale resampling multiplies
colours"*, and concluded that `--crop` output is *"unusable for any pixel analysis"*. **The
mechanism is wrong.** That comparison moved two variables at once — a PNG true-scale crop
against a JPEG half-scale one. Separated over the same region:

| | dimensions | distinct colours |
| --- | --- | --- |
| PNG, true scale | 984×340 | **523** |
| PNG, half scale (the answer as filed) | 492×170 | **878** |
| JPEG, true scale | 984×340 | **14,090** |

Resampling does fabricate colour — 878 values on a quarter of the pixels that held 523 — but at
**+355** it is the small term. **Encoding is the large one at +13,567**, one variable changed,
and JPEG at *true* scale is sixteen times worse than resampling at half scale.

**So this decision alone does not make `--crop` fit for pixel analysis, and the doc strings
must not imply that it does.** They now name `--png` at the same time, because a flag that
promises analysable pixels and returns JPEG would be #402's defect rebuilt one flag over.

**The resampling claim is scoped to flat ink, not stated generally.** On the photographic whole
frame the direction inverts — 210,827 distinct colours at 1080×1920 against 91,177 at 540×960 —
and the check asserts that inversion so this evidence can never be cited as *"downscaling
always adds colours"*. It fabricates on flat ink under text: the region class an agent inspects
to judge a card's fill, and the one that made map #395's trial agents reach for `--crop`.

## `--full` with a region is redundant, not refused

ADR-0097 §4 refuses `--full` with a *range* because it has no referent. The parallel does not
carry to a region: `--full --crop` is today the **only** spelling that reaches what the doc
string promised, so refusing it would turn every correct workaround into an error on upgrade,
to buy tidiness. Measured: `--crop --full` is **byte-identical** to `--crop`.

That `--full` is already a reflex for anyone who wants honest pixels is on the record in this
repo's own `docs/research/contact-sheet-legibility/make_sheets.py`, whose comment cites this
very issue — *"at true scale (see FINDINGS.md on #402: `--full` is required, or every tile is
downscaled twice)"*. It passes `--full` on whole frames rather than with a region, so it is
evidence of the habit and of how long the double-downscale has been known, not an instance of
the pair.

## The whole-frame crop opens no discount

If a region implies true scale then `--crop 0,0,1080,1920` reaches true scale without `--full`.
Measured, it is **byte-identical to `--full`**, so it costs what `--full` costs — 2691 tokens on
the high-resolution tier — and the caption still reads `full scale`. It is `--full` spelled
longer. **The half-scale default was a default, not a quota**, and nothing is concealed, so no
refusal is added. A redundancy refusal was considered and declined: it would spend a new error
path defending a budget that was never enforced as one.

## ADR-0011's token figures, corrected at the code sites (#405)

ADR-0097 corrected ADR-0011's flat `2691` to a two-tier fact. #405 owned the four sites still
publishing the flat version, and they are the same sentences this amendment edits:

| | standard tier | high-resolution tier |
| --- | --- | --- |
| 540×960 | served as-is, **700** | served as-is, **700** |
| 1080×1920 | downscaled to 819×1456, **1560** | **2691** |

So `--full` is 3.84× on one tier and 2.23× on the other, and a caller paying for it to inspect
a thin stroke may not be served those pixels at all. `819×1456 → 1560` is **cited, not
recomputed**: the standard tier's downscale is not a published long-edge rule — a long-edge cap
at 1568 would give 881×1568 and 1792 tokens — so the check declines to reimplement it and rests
on `docs/research/visual-token-cost/`, which measured it.

Corrected at `crates/montagent-core/src/verbs/frame.rs`, `crates/montagent/src/cli.rs` (the
verb's help and the `--full` flag's), and `crates/montagent/src/mcp.rs` (the tool description
and the `full` parameter).

## What this leaves to others

**[#406](https://github.com/MBehtemam/Montagent/issues/406) is unblocked, not answered.**
Whether the contact sheet crops every tile, and what a cropped sheet must disclose, is map
#395's decision with its own evidence — cropping every tile was measured there to buy 2.33×
scale and total blindness outside the band, losing a planted wrong-photo defect from a sheet
that looked clean. The sentence it may spend: **this rule is about a single-frame region.**
ADR-0097 §4 already parks `--crop` with a range as not-yet-legal, so declining to answer leaves
nothing invocable-but-undefined.

**The precedence finding is left as a recorded observation**, not a ticket. It is an argument
about agents rather than about `frame`, and the only lever this repo holds is the one used
here: do not publish a promise the answer contradicts.

## Costs, recorded honestly

- **One predicate now carries two meanings.** `--full` means "true scale" and, with a region,
  means nothing. That is a surface that reads slightly worse than it behaves, bought
  deliberately for upgrade safety.
- **The 8× JPEG finding argues for a default this ADR does not change.** A crop is for looking
  closely, and its default encoding fabricates 14,090 colours where the frame held 523. Making
  `--crop` imply `--png` was not decided here — encoding is orthogonal to scale under ADR-0011,
  and latency is a real cost — but the measurement is on the record and a future ticket may
  reach for it.
- **The half-scale arm is unreachable after this change**, so its measurement is a committed
  artifact (`docs/research/crop-true-scale/before/`) rather than something the check re-renders.
  A future change to the renderer will not move that number, and the check cannot notice.
