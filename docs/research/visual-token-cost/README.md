# Is the token-cost argument for a contact sheet actually true?

Research for [#397](https://github.com/MBehtemam/Montagent/issues/397), part of
map [#395](https://github.com/MBehtemam/Montagent/issues/395).

**Short answer: the formula is right, one of `frame`'s two numbers is wrong, the
"16 instants for the price of one" framing is arithmetically true but describes
the wrong saving, and the caption is a second cost of comparable size that no
pixel-denominated budget can see.**

Re-executable check: `python3 docs/research/visual-token-cost/check_visual_token_cost.py`
(exits non-zero the moment any number below stops reproducing). Data it runs
against: `captions/*.txt`, the 18 real `frame` answers, one per visual state of
`fixtures/en-halloween-decorating`.

---

## Primary sources

Both pages are Anthropic's own documentation for the Messages API, which is the
surface an MCP tool result lands on.

- **Vision — Resolution and token cost**:
  <https://platform.claude.com/docs/en/build-with-claude/vision#evaluate-image-size>
- **Coordinates and bounding boxes — How Claude resizes and pads images**:
  <https://platform.claude.com/docs/en/build-with-claude/vision-coordinates#how-claude-resizes-and-pads-images>

The formula, quoted verbatim from the first:

> Claude views images in patches instead of pixels. Each patch is a 28×28-pixel
> block of the image, referred to as a visual token. An image, therefore, costs
> `⌈width / 28⌉ × ⌈height / 28⌉` visual tokens.

And the sentence `frame`'s doc string does not know about, from the same section:

> Each model has a maximum native image resolution, expressed as a long-edge
> limit and a visual-token limit. Images larger than either limit are downscaled
> before processing.

| Resolution tier | Models | Max long edge | Max visual tokens |
| --- | --- | --- | --- |
| High-resolution | Claude 4.7 and later models | 2576 px | 4784 |
| Standard | All other models | 1568 px | 1568 |

The resize rule, quoted from the second page:

> Claude finds the largest aspect-preserving size that satisfies both of the
> model's image limits: **Edge limit:** neither side exceeds the maximum edge
> length […]. **Visual token limit:** the image's token cost
> `⌈width / 28⌉ × ⌈height / 28⌉` does not exceed the model's visual token budget
> […].
>
> Claude then pads every image, resized or not, up to the next multiple of 28
> pixels on the bottom and right edges.

`check_visual_token_cost.py` transcribes that page's reference implementation and
reproduces every row of the published cost table and its A4 worked example
(group A). That is the anchor: if group A ever fails, the transcription or the
published table has moved and everything below needs re-reading.

---

## 1. The formula

`⌈w/28⌉ × ⌈h/28⌉` is correct **as the formula**, but it is applied to the
dimensions the model is *served*, not the dimensions you sent. Those differ
whenever either tier limit is exceeded, and they differ by tier.

The doc string's claim, checked (group B):

| Claim | Standard tier | High-resolution tier (Claude 4.7+) |
| --- | --- | --- |
| `frame` default, 540×960 → **700** | served 540×960, **700** ✅ | served 540×960, **700** ✅ |
| `frame --full`, 1080×1920 → **2691** | downscaled to 819×1456, **1560** ❌ | served 1080×1920, **2691** ✅ |

**700 is right everywhere. 2691 is right only on Claude 4.7 and later.** On a
standard-tier model the long edge (1920 px) exceeds the 1568 px limit, so the
frame is served at 819×1456 and costs 1560. That makes `--full` a 3.84×
multiplier on a high-resolution model and only a 2.23× multiplier on a standard
one — the flag's own help text ("You are asking for 2691 tokens rather than 700,
and that is the whole of what this flag does") is wrong for half the model range,
and wrong in the direction of overstating the penalty.

This also means `frame --full`'s pixels are *not always delivered*. On a
standard-tier model, a 1080×1920 frame is silently resampled to 819×1456 before
the model sees it, which is a legibility fact, not just a cost one, and it lands
squarely on the map's tile-legibility question.

## 2. Encoding

**The doc string is right: encoding does not enter the token count.** The formula
takes only width and height, and the docs list JPEG, PNG, GIF and WebP as
equally supported inputs with no cost distinction.

Measured against the real binary (group C), at the same instant and scale:

| | Served size | Visual tokens | Bytes |
| --- | --- | --- | --- |
| `frame` (JPEG, half) | 540×960 | 700 | 69,681 |
| `frame --png` (half) | 540×960 | 700 | 527,379 |
| `frame --full` (JPEG) | 1080×1920 | 2,691 | 200,354 |
| `frame --png --full` | 1080×1920 | 2,691 | 2,035,990 |

Same tokens, **7.6× the bytes**. The `--png` help text ("Costs the same tokens
(they are a function of decoded pixels), and buys lossless pixels for more bytes
and more latency") is exactly right.

Bytes are not free, though — they are just charged somewhere else. Eighteen
full-scale PNGs base64-encode to **46.6 MB**, over the API's 32 MB request limit;
eighteen full-scale JPEGs come to **4.6 MB** and fit. So on the status-quo
N-calls path, `--png --full` over a whole video is not expensive, it is
*impossible*, at identical token cost. Worth a line in whatever the sheet's
disclosure says.

One incidental confirmation of the map's **Out of scope** section, from the same
docs: "Animations are unsupported, and only the first frame is used." An animated
GIF is not a cheap motion channel; it is one frame at full price.

## 3. One large sheet versus N small images

This is where the map's framing is loosest. The ticket asked whether "per-image
overhead makes N images strictly worse". **It does not.** The only per-image
overhead in the cost model is each image's own `ceil()` to a 28 px multiple, and
it is small enough to be outweighed by grid geometry:

| Layout | Per tile | N tiles loose | One 540×960 sheet |
| --- | --- | --- | --- |
| 4×4, 135×240 tiles | 45 | **720** | **700** |
| 4×5, 135×192 tiles (the fixture's 18 states) | 35 | **630** | **700** |

At 16 tiles the sheet wins by 20 tokens — that is the padding overhead, and it is
the entire per-image penalty. At 18 tiles in a 4×5 grid the sheet *loses* by 70,
because it pays for the two empty cells. **Tiling does not save visual tokens.
Shrinking does.**

The real arithmetic, and it is large:

- 18 instants the way an agent gets them today: 18 × 700 = **12,600** visual tokens.
- 18 instants as one 540×960 sheet: **700** visual tokens. Saving: **11,900**.

So "16 (or 18) instants for the price of one" is *true*, and the check asserts it.
But the mechanism is not tiling; it is that a sheet caps total pixels. Each
instant arrives at 35 visual tokens instead of 700 — **20× less resolution per
instant**. The cost model is not giving anything away. It is offering an exchange
rate, and the thing being spent is exactly the tile legibility the map already
names as its constraint.

There is one genuine, non-linear per-image penalty, and it is not in the formula:

> If a single API request contains more than 20 images, a stricter per-image
> dimension limit applies to every image in that request. All `image` blocks in
> the request count toward this threshold, including images from earlier
> conversation turns that you resend and images nested inside `tool_result`
> content […] To stay under the limit on all platforms, either resize each image
> so that neither dimension exceeds 2000 px, or keep the request to 20 or fewer
> image and document blocks.

Eighteen `frame` results plus two of anything else crosses that line, and then
*every* image in the conversation — including ones from earlier turns the agent
is still relying on — gets clamped. A sheet is one image block and cannot trigger
it. This is a better argument for the sheet than the token arithmetic is, and
nobody on the map has made it yet.

## 4. The status quo's other half: the caption

Every `frame` call returns the resolved stack as text, unconditionally and by
design. Measured over the fixture's 18 visual states (group D — the captions are
committed verbatim in `captions/`, and the check regenerates and byte-compares
them):

| | Characters | Words | Lines |
| --- | --- | --- | --- |
| One caption (at 30603 ms) | 1,530 | ~200 | 26 |
| **All 18 captions** | **28,410** | **3,588** | **456** |
| One equivalent sheet answer | 1,413 | 183 | 30 |

**Ratio: 20.1×.** The "equivalent sheet answer" is not a guess — the script builds
it from the same captions' own data: the validate scoreboard, a `SHEET` header
with the selection rule, the tile scale and what was skipped, one label per tile
naming the instant and the elements that produced it, and the `NOT CHECKED`
block. It is written to the scratch directory on every run so it can be read and
argued with. Most of what the 18 captions repeat is genuinely redundant: the
eight persistent chrome elements (`chip-panel`, `handle-*`, `flag-*`,
`chip-text`) appear with full coordinates in all 18, and the scoreboard,
`sources`, `fonts` and `NOT CHECKED` blocks appear 18 times each.

The trial agents' complaint — "a lot of text tokens for information I already had
from the cut list" — is real and is most of the repetition.

**What this document does not establish: the caption's cost in tokens.** There is
no offline Claude tokenizer (`tiktoken` is OpenAI's and is wrong for Claude), and
this session had no `ANTHROPIC_API_KEY` or `ant` credential to call
`POST /v1/messages/count_tokens`. Rather than guess a chars-per-token ratio from
memory — which this repo's rules forbid, and rightly, since it is exactly the
kind of number that would then get cited — the check reports characters, words
and lines, which are exact and tokenizer-independent, and **calls
`count_tokens` and asserts the token ratio automatically the moment a key is
present**. Whoever runs it with credentials closes this gap in one command; the
gap is in the absolute figure, not in the 20.1× ratio, since both texts are the
same register and the same tokenizer sees both.

What can be said without a tokenizer: 3,588 words of English prose and dotted
identifiers is thousands of input tokens, on the same bill and in the same
currency as the 12,600 visual tokens the same 18 calls spend on pixels. The
caption is not a rounding error on the image cost. It is the same order of
magnitude.

---

## Two errors found in the sources

Recorded so nobody re-derives them:

1. **`frame`'s doc string is wrong about 2691** (three copies: `frame.rs:12`,
   `cli.rs:176`, `mcp.rs:495`, plus the `--full` flag help). The number is
   tier-dependent and the doc string asserts it unconditionally. See §1.
2. **Anthropic's published table is one pixel off its own reference
   implementation.** The table gives 1269×952 as the standard-tier downsize of a
   2000×1500 image; the reference implementation on the coordinates page returns
   1270×952. It does not move the token count (`ceil(1269/28) == ceil(1270/28)`),
   so the cost model is unaffected; the check asserts the reference
   implementation's answer and says why in a comment. Reported here only so a
   later reader does not treat it as a transcription bug in the check.

## What this means for the budget question in #399

1. **Denominate the budget in input tokens, not tiles and not pixels.** Visual
   tokens and caption tokens are the same currency on the same bill. A policy
   that counts tiles governs the cheaper half of what a sheet call costs today.
2. **The sheet's saving on pixels is real and large (12,600 → 700), but it is a
   resolution trade, not a free lunch.** Any budget policy is therefore a
   legibility policy wearing a cost hat, and the honest knob is tile size, with
   the disclosure naming it. `--full`'s tier-dependence (§1) means the sheet's
   tile scale must be disclosed in served pixels, not in a scale factor.
3. **Tiling per se saves nothing** — a 4×5 grid with empty cells is *more*
   expensive in visual tokens than the loose tiles it holds. Do not write a
   policy whose justification is "tiling amortizes per-image overhead"; the
   overhead is 20 tokens at 16 tiles.
4. **The >20-images rule is the strongest structural argument for the sheet**
   and is not a budget question at all: it is a correctness cliff that silently
   degrades images from *earlier turns*. It belongs in the map's notes.
5. **The caption is the other budget, and it is the one nobody has costed.**
   18 captions are 20.1× one sheet answer's text. If the sheet repeats the
   full resolved stack per tile it gives the pixel saving straight back. The
   sheet's label design is a cost decision, not a formatting one — which makes
   "one label per tile, chrome elided" a *specification*, not a nicety.
6. **`frame`'s doc string needs correcting before ADR-0011 is amended**, since
   the amendment will inherit its numbers.

Not done here, deliberately: no ticket closed, nothing merged, no doc string
edited. §1 and the doc-string fix are a separate change on a separate branch.
