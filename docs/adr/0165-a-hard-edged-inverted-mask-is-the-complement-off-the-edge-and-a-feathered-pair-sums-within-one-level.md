---
status: accepted
amends: 0152 (§2 and its Why: an inverted feathered mask keeps 1 − what the plain one keeps within one level of 255, not exactly; a hard pair is the complement only where either keeps a pixel whole or erases it whole), 0163 (§5: `invert` keeps the complement in the same sense, not exactly)
---

# A hard-edged inverted mask is the complement off the edge, and a feathered pair sums within one level

[#773](https://github.com/MBehtemam/Montagent/issues/773), on the map
[#663](https://github.com/MBehtemam/Montagent/issues/663), deferred from the path-mask slice
([#768](https://github.com/MBehtemam/Montagent/issues/768),
[PR #772](https://github.com/MBehtemam/Montagent/pull/772)) by a court ruling.
[ADR-0152](0152-a-mask-gains-invert-and-feather-and-takes-no-text-shape-or-image-source.md) §2
says an inverted feathered mask keeps "exactly 1 − what the uninverted one keeps", and its Why
says a centred edge keeps the inverted mask "the exact complement of the plain one".
[ADR-0163](0163-a-mask-takes-a-closed-path-inline-measured-from-its-own-rect.md) §5 says
"`invert` keeps exactly the complement". The painter does not do that on a hard edge:

- a hard plain mask is one antialiased `Clear` draw of the shape's inverse fill, and a hard
  inverted mask is one antialiased `Clear` draw of the shape. Skia antialiases the two
  separately, so on the 1–2 px edge they differ: measured up to 25 levels of 255 for an
  `ellipse` and about 64 for a `path`;
- a feathered mask draws the shape once into a blurred layer, composited `DstIn` (plain) or
  `DstOut` (inverted), so the pair sums to the whole element within one level of 255 on every
  pixel, not exactly.

Settled by two `/court` rounds of three jurors (Fable, Opus, Sonnet), judged by how agents would
work with each choice, with the owner ruling with the Judge's read. The first round was
unanimous for amending the wording over changing the eraser; the second was unanimous on the
text below and split two to one on the hand-off form.

## The decision

### 1. The wording, not the eraser, changes

**A plain and an inverted mask of one shape are complements on every pixel either keeps whole
or erases whole.** This holds for every mask shape. On pixels the edge only partly covers, a
hard edge (`feather` 0 or omitted) antialiases the shape and its inverse separately, so the two
may differ there. That was measured at up to 25 levels of 255 for an `ellipse` and about 64 for
a `path`, on a 1–2 px edge. These are two measurements, not a bound. A feathered pair
(`feather` 1 or more) keeps one blurred coverage two ways, so the two sum to the whole element
within one level of 255 on every pixel. An author who splits one source across a plain and an
inverted mask and needs the two to rebuild it (an `add`-blended pair, say) writes a `feather`.

- The eraser is unchanged, so no byte of any render changes, and
  [ADR-0075](0075-the-badges-mask-changes-its-own-rim-and-nothing-else.md)'s measured badge
  numbers stand.
- Whether a hard-edged `rect` or `circle` pair differs at all on its edge is left to the test
  (§3): an axis-aligned rect on whole pixels may have no partly covered pixel. "May differ"
  covers both outcomes.

### 2. Exact complements do not make a seamless split

Two elements stacked with coverages c and 1 − c still show the backdrop by c·(1 − c) on the
edge, whatever the masks are. With everything opaque and c = 0.5, the lower element keeps alpha
0.5, the upper adds 0.5 over it, and the total is 0.75: a quarter of the backdrop shows. This is
the conflation of stacked antialiased edges, and
[ADR-0150](0150-wipe-slide-and-push-enter-as-transition-kinds-and-a-matte-from-another-element-is-refused.md)'s
wipes avoid it by cutting on whole pixels. Whether a split outside a transition gets an
equivalent is a separate question, recorded in the map's "Not yet specified".

### 3. What the format page and the test say

- The format page's `invert` and `feather` rules (`compositing.md`) state §1: the hard-edge
  sentence, "within one level" for a feathered pair, and the pointer to `feather`. The
  capability map does not change, since no capability is added or retired.
- The test that today asserts the pair on a `path` only is renamed to what it asserts and runs
  over all four shapes: exact where both are whole, a thin edge, and within one level with a
  `feather`.

## The four tests

| Invariant | How this keeps it |
| --- | --- |
| Literal values | No value changes. |
| Closed vocabulary | No field or value is added. |
| Checkable by `validate` | Nothing new to check; the promise is now one a test asserts over every shape. |
| Exact-string replace | Unchanged: `"invert": true` and `"feather": <n>` are still one string each. |

## Considered and refused

- **Change the plain eraser only**, to the opaque shape in a bounded layer composited `DstIn`,
  so a hard pair sums within one level everywhere and the hard edge becomes the `feather → 0`
  limit. Refused: the case agents would reach for, a stacked hard split, still shows a quarter
  of the backdrop at c = 0.5 (§2), so the change buys the sentence and an `add`-blended hard
  pair. Its price is every plain hard-mask golden changing on its edge, ADR-0075's numbers
  re-measured, a bounded layer per hard mask with its own outset, and a fresh byte-identity
  run with an owner-accepted render. `Clear` at coverage c and `DstOut` at alpha c may also
  round apart, so the inverted side was unmeasured.
- **Route both through the layer.** Refused for the same reasons, with every inverted golden
  changing too.
- **Edit ADR-0152 and ADR-0163 in place.** Refused: it would erase the record that both once
  promised an exact complement and why that was withdrawn.
- **"Exact except where antialiasing touches the edge."** Refused as untestable: the edge
  differs by shape, and the wording above names what a test checks.

## Consequences

- ADR-0152 and ADR-0163 carry amendment banners. ADR-0152 §2's "exactly 1 −" sentence, its Why
  paragraph's "exact complement", and ADR-0163 §5's "`invert` keeps exactly the complement" are
  read through this ADR.
- No new code in the painter and no prototype gate.
- A hand-off spec on the map carries §3: the format page edit and the four-shape test.
