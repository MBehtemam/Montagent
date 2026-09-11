# Design Questions: Text and Shape Primitives — Jury 13, Answer C

## Q1: Is a shape one element type or many?

**Answer:** Multiple types (`type:"rect"`, `type:"ellipse"`, etc.) are better than a single `type:"shape"` with a discriminator.

**Confidence:** Medium-high (75%).

**Reason:**  
Montaget's core editing model is exact-string replacement. The fixture currently uses `type:"rect"` directly, and all coordinates and properties appear flat on the element itself. ADR-0012 explicitly rejects nested objects for transform fields to avoid "adding a path segment to every edit, every `jq` and every `query` output." The same argument applies here: a `shape:"rect"` nested under `type:"shape"` would require replacing `"type":"shape","shape":"rect"` as a compound, defeating the atomic-edit model when I want to change just the shape type. Alternatively, if I flatten it to sibling fields and try exact-string replace, I'd write `"shape":"rect"` but the document format has already normalized to `type:"rect"` on write (`fmt`), so my replacement finds nothing. Every concrete element type in the fixture (text, image, audio) is a direct `type` value, not a discriminator inside. One schema convention per element kind.

## Q2: Which shapes exist in v1?

**Answer:** `rect` only, in v1. `ellipse`, `line`, `polygon`, and `path` are deferred.

**Confidence:** High (85%).

**Reason:**  
Fact 3 states all 10 rect elements carry exactly `fill`, no other paint field. The fixture contains 10 rects and zero other shapes. The README documents the layout—a cream panel, navy card, white flag bars—all implemented as separate rectangles. The README's phrase "drawn flag — three filled rectangles" shows rects are sufficient for the real project. No ellipse, circle (the logo badge is a PNG with baked-in alpha), line, or polygon appears anywhere. Circles and curves were not needed to ship the fixture. `ellipse` adds complexity (bounding box vs. natural circle size, radii vs. dimensions) and text on circles (ADR-0012's `origin` makes placement explicit everywhere, but rotation + text on a curve is different). Lines and polygons invite stroke styling (open Q5), which is not yet designed. Defer them; don't preclude them. No schema error for an unknown `shape` value; an `UNCHECKED` finding is enough if a future ticket lands one.

## Q3: How is a colour spelled?

**Answer:** `#RRGGBB` hex only, in v1. No alpha, no shorthand, no CSS names.

**Confidence:** High (88%).

**Reason:**  
Every color in the fixture is `#RRGGBB` format: `#FBF3E3`, `#245C8C`, `#1E344C`, `#FFF8E8`. ADR-0012 already gives every element an `opacity` field (defaulting to 1), which handles alpha separately and decouples it from color. Allowing `#RRGGBBAA` creates two ways to express transparency in one file (color alpha vs. element opacity), triggering ADR-0007's named failure: "a field that is sometimes X and sometimes Y cannot even produce a good error message." Three-digit shorthand (`#RGB`) looks succinct but requires a parser to expand to six digits; exact-string replace then finds nothing (`#FFF` → `#FFFFFF`). CSS names like `red` are human-readable but not deterministic (is it sRGB? does rendering engine agree?), and CONTEXT.md's rejection of system font names—*"a table you cannot read, cannot commit, and that differs per machine"*—applies identically to color names. The fixture's proven grammar is simple: `#` followed by six hex digits, always. One spelling, no ambiguity.

## Q4: Does a text element still declare a `height`?

**Answer:** No. `height` should become optional in the schema, and when omitted, `validate` emits a `note` if the declared value matches the derived value.

**Confidence:** Medium (60%).

**Reason:**  
Fact 4 is the crux: 15 of 22 elements have `height == ceil(size * line_height * line_count)` and are derivable from the element itself; 7 have `height` sourced externally (container dimensions). All 22 have declared `width` for reasons ADR-0007 explains—horizontal overflow is real and the box may not be an element. But if `height` is derivable for 68% of elements, enforcing it everywhere creates three problems:

1. **The overflow check becomes useless on most files.** With both `width` and `height` required, validation never catches the common case where I correct `size` or `line_height` but forget to recalculate `height`. ADR-0006 frames overflow as "the format has no representation of the box the text has to fit in"; keeping both required papers over this by making the field do two jobs: real containment (on 7 elements) and derived validation (on 15).

2. **I have to maintain a redundant value.** When I measure text using `measure`, it tells me the exact `height` needed. If I paste that value into the file, I've done the job—but if `height` is still required and I then edit `line_height`, the overflow check is now stale. I can't see in the file whether `height` is real (external) or derived; I have to remember or recompute. Option (b), making `height` optional and letting omission mean "no external container is claimed", splits this: when present, it's authoritatively external; when absent, it's understood to be derived from typography and the overflow check uses the computed value. The redundancy disappears.

3. **The override case is expressible but confusing.** If an author wants deliberate padding (text element inside a larger box, leaves whitespace), they need to declare `height > computed`. With both required, that intent is silent—any `height` could be either computed or intentional. With optional, I write `height` explicitly because the element itself does not determine it; the gap is visible. (This is option c's silent finding, but better located: optional presence says "this is external", not a validation error on every element.)

**Cost:** This collides mildly with ADR-0012, which says "An element's own `width`/`height` must be in the document … a default is only safe where its wrong answer is loud." But the wrong answer *is* loud here: `validate` can compute the expected value and name the difference. And `height` is not truly defaulted—it defaults to the computed value only when absent, and `measure` tells me what to write if I want to override. Option (b) does not violate the principle; it just moves the enforcement of *derivability* to the tool layer rather than the schema layer. Montaget already requires `measure` for text authoring (ADR-0007); this makes that tool slightly more load-bearing.

## Q5: Is `stroke` a field on the primitive, or does it belong to a separate effects vocabulary?

**Answer:** Deferred. Stroke belongs in a separate effects system to be designed in a different ticket.

**Confidence:** High (82%).

**Reason:**  
The fixture contains no stroke on any shape or text. ADR-0007 lists full run-style deltas as "open and belonging elsewhere," including "outline — with the ASS BGR-with-alpha-nibble trap and `ScaledBorderAndShadow` written down." This signals that stroke is complex: it has width, color, alpha, dash patterns, line caps, joins. Bundling it into the primitive schema now before the effects vocabulary is designed creates a false choice.

If I add a `stroke` field to rect now, I have to decide: is it one object with color and width? Does it live alongside `fill`, or does it bundle them both into one paint object? Does text stroke share the same shape? The fixture shows no stroke anywhere, so I have no evidence to pick the right shape. Worse, it would be easier to overshoot—shipping paint fields I do not need and that constrain later design. ADR-0007 explicitly warns: "a field the renderer cannot honour is worse than no field."

The line between primitive and effect is: **a primitive's paint fields only** (`fill`, maybe color on text). **Stroke, shadow, and other affectations are effects.** This is clean because `fill` determines whether a shape is drawn at all, and `color` on text is the fundamental style; stroke is decorative and orthogonal to the thing's existence. Given that text elements already carry `color` but no stroke, and rects carry `fill` but no stroke, and nothing in the fixture uses stroke, the right call is: **leave stroke out of the primitive design, defer it to the effects ticket (#22 or equivalent), and do not add it to the schema until the effects vocabulary is named.**

## Q6: Should `gravity` be decided together with text and shape primitives, or does it belong with `fit`?

**Answer:** Defer `gravity` with the `fit` vocabulary to ticket #21/#48. Do not ship it in this ticket.

**Confidence:** High (87%).

**Reason:**  
ADR-0013 supplies the measured fact: `gravity` is **inert on 8 of 8 image elements** in the committed fixture. The declared rect and `clip` together already determine which part of the source survives—the aperture is static. This is not aesthetic preference; it is structural fact: when you have explicit `width`/`height`, `fit`, and `clip`, the geometry is fully determined, and no separate `gravity` field adds information.

However, the question is not whether `gravity` is used *now*, but whether it belongs in *this ticket*. The brief warns that "the fixture is test data and a regression guard, never a scope boundary … 'The fixture does not use it' is not an argument against a feature." So the architectural question stands: should `gravity` ship alongside rect and text primitives, or with the `fit` vocabulary that modifies it?

**It belongs with fit.** Here is why:

1. **Gravity modifies fit behavior.** On `fit:"cover"`, `gravity:"top"` means "show the top of the source". On a hypothetical `fit:"contain"`, it would mean something different (pad top? preserve top?). Gravity's meaning is entangled with fit's value. Separating them now splits a single design problem and invites inconsistency—one ticket says gravity is for images, another says fit handles scaling, and nobody names the coupling.

2. **The fixture uses only one value.** All 8 images have `gravity:"top"`. If this ticket ships gravity, I have to explain why only `top` appears and what the other 8 directions mean. I have to decide whether gravity applies to text (no, only images) or future shapes (unclear). I have to define its interaction with `clip` when clip itself is just now settled. That is overcommitting to one part of a larger question.

3. **Fit vocabulary is still open.** ADR-0013 records fit values other than `cover` as undefined (#48). Gravity's semantics depend on that vocabulary. Deferring gravity with it keeps them coupled and makes the later ticket complete.

**The cost:** Gravity in the fixture is syntax-inert anyway (all elements use `top`), so removing the requirement does not break the file—only validation that expects it. But I'd remove it from the schema entirely for v1, or mark it `UNCHECKED` for unprobeable sources. Do not ship an unused field; it teaches the wrong lesson.

---

## What would bite me first?

**Text height redundancy and overflow invisibility.** If I ship with both `width` and `height` required on every text element, and both are supposed to contain the text, then the overflow check is my only guard against mismatched typography. But I can derive height from four values (size, line_height, line_count, runs), and fifteen of the fixture's elements do. So on most edits—swap a font, change size, add a line—I recalculate `height` manually. I will eventually forget on one of them. The file looks valid; `validate` passes; the video renders with text spilling into the cream band, and I ship it believing it was fine. The data was there to see (size and line_height are declared), the check was supposed to protect me (height is present), but the structure made the check a no-op. This is ADR-0007's worst-case state: *a field that determines whether the output is usable is the one property the file cannot state [clearly]*.

---

## Where I disagree with the question itself

**Q2 is slightly malformed.** It asks "which are in, which are out, and which are merely unevidenced rather than rejected." But "unevidenced" needs a referent. Unevidenced by what? The fixture is evidence that rect works; nothing in the repo is evidence that ellipse works. So every shape except rect is currently unevidenced. The question might be trying to ask "which are ruled out by design (like After Effects corner pin), which have design space (open for later), which are proved needed." If so, clearer: `rect` is proved; `ellipse`, `line`, `polygon`, `path` are open; skew and corner pin are After Effects (ADR-0003) and out of scope. Distinguishing "deferred" from "ruled out" is the real work.

**Q3 presupposes something true but worth naming.** The question asks how colour is spelled, and all the options (hex length, shorthand, CSS names, alpha) are about the *serialization* of color. It does not ask whether color can be per-run (inline in a run's style delta on text) or only per-element (uniform on a rect). Fact 3 and the fixture show per-element color on rects (one `fill` per rect). ADR-0007 discusses per-run color deltas as "open and belonging elsewhere." So there are two questions silently here: "what is the serialization" (answered above) and "what is the scope: per-element or per-run." The latter is design for text styling, which is deferred to the effects ticket. Worth separating them.

