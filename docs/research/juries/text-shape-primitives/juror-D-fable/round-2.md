# Answer R2-D

Verification done before arguing: I read CONTEXT.md and ADRs 0003/0006/0007/0008/0012/0013, and re-derived the brief's facts against the repo. Two additions the brief does not state:

- Every ASS style in `fixtures/en-halloween-decorating/reference/subtitles/` carries `Outline=0, Shadow=0`, and there is **not one** inline `\bord`, `\shad`, `\3c`, `\1a`, `\3a` or `\alpha` tag in any of the seven files. `ScaledBorderAndShadow: yes` is inert header boilerplate — there is no border or shadow for it to scale.
- All **180** eight-digit ASS colours across the seven files have alpha nibble `00` (ASS-opaque). The old pipeline used zero strokes and zero colour transparency.

Per the standing rule, neither fact argues *against* stroke or alpha. But they must not be cited *for* them either — brief fact 6 reads as if `ScaledBorderAndShadow` evidences stroke usage, and it evidences nothing. Flagged again at the end.

---

## Q7 — the paint package

**First attraction: Position A** (stroke on shapes and text, run-level on text; `#RRGGBB` or `#RRGGBBAA`).

### Strongest attack I could mount

Three prongs, in descending order of force.

**1. The stroke-geometry finding is a live violation of ADR-0013.** If `stroke_width: 8` centres on `card-05`'s path, the drawn card is 992x177 at (44,1449) — "what is drawn is a number not in the file", which is the exact defect ADR-0013 spent a section killing under the name *declared-rect-authoritative* ("that would make what is actually drawn a number not in the file, derivable only from … not in the file"). If Position A entails centred strokes, Position A imports a defect the repo has already named and rejected.

**2. Two spellings of one colour.** `#245C8C` and `#245C8CFF` are the same value in two shapes — the alias pattern ADR-0013 rejected for `center-center` and ADR-0012 rejected for bare-scalar `scale`, both via the fmt-normalisation / exact-string-replace trap.

**3. Renderer rule growth.** Stroke must answer: does `stroke_width` scale with `scale`? What are the join semantics? A field that "reads declarative and behaves surprisingly" is the failure class ADR-0007 twice names.

### Whether it survived

**Yes, but only because each prong has a specific answer, and prong 1 forces the geometry sub-answer.**

- Prong 1 is an attack on *centred* strokes, not on strokes. It is resolved by the geometry rule below (inside, for shapes), under which every number in the file stays true.
- Prong 2 dissolves on inspection of the mechanism. The scale-union was rejected because `shift` must *interpolate* the value, putting a shape test in every keyframe consumer; `center-center` was rejected because `fmt` is *required* to normalise conventions. Colour is not keyframable (it is not in ADR-0012's property set), so no interpolation consumer exists; and a colour is *content*, which ADR-0013 already rules `fmt` must never rewrite ("a formatting tool that changes the output video" is disqualifying). So the one rule needed is: **`fmt` never rewrites a colour; `#RRGGBBFF` is legal as written and never canonicalised to `#RRGGBB` or vice versa.** No round-trip trap remains.
- Prong 3 is answered by publishing the rule rather than not shipping the field (the ADR-0013 pattern: the renderer publishes a sampling rule instead of sampling privately): `stroke_width` is in element space and scales with the element, exactly like everything drawn and unlike frame-space `clip`.

Two positive arguments then decide it, and I could not break either:

- **The run-inexpressibility argument is structural, not aesthetic.** The effect vocabulary attaches to elements; a run lives inside an element. If text stroke is an effect, "outline one word" — the ordinary reference-class edit, the same class as ADR-0007's "emphasise one word mid-line" — is inexpressible at any price. Position B has no answer to this; it can only deny the need, and CapCut and Premiere both put text stroke in the text styling panel, not in an effects rack.
- **Position B misreads the settled record twice.** ADR-0007's open item places "outline" *inside* "the full run style-delta set (colour, outline — …)" — it frames outline as run-level primitive styling awaiting specification, not as effect-vocabulary territory. And the "ASS BGR-with-alpha-nibble trap" it names is ASS's *BGR byte order plus inverted alpha* (`00` = opaque) — a transcription hazard to be **written down**, which is literally what the open item asks for. `#RRGGBBAA` is CSS Color 4: RGB order, `FF` = opaque. Citing the trap against any alpha nibble treats a documentation instruction as a prohibition.
- Finally, alpha is not redundant with `opacity` even for a single paint, because **`opacity` is the animation channel**. A 50% scrim that also fades in must bake the static 0.5 into every opacity keyframe (0 → 0.5) — the exact channel-collision ADR-0013 records for static stretch in `scale` ("base geometry and animation share one channel and neither stays readable"). `fill:"#0000007F"` plus opacity keyframes 0 → 1 compose cleanly. The repo has already ruled on this shape of problem.

**Verdict: Position A. Confidence: high (~0.8).** Shapes carry `stroke` + `stroke_width`; text carries stroke as a run-level style delta (element-level base, per-run override, like every other run delta); colour is `#RRGGBB` or `#RRGGBBAA`, RGB order, `FF` opaque, with the ASS trap documented in the ADR and `fmt` forbidden to rewrite colours. The effect ticket must be told stroke's home is here — one concept, one home, per the `anchor` precedent.

### The stroke-geometry sub-question

**Shapes: inside the declared rect. Text: outside the glyph contour. The rule is deliberately not the same, and the difference tracks an already-settled difference in what the rect means.**

- **Shapes — inside.** ADR-0013's declared-rect-authoritative principle makes the declared `width`/`height` *the drawn geometry* for a shape. Inside is the only one of the three options under which every number in the file remains true when a stroke is added: `card-05` with `stroke_width: 8` still occupies exactly 984x169 at (48,1453), the 48px margin the layout rests on is untouched, and finding 1's exactly-square corners at exactly (48,88)/(48,1453) stay at exactly those coordinates. Centred eats 4px of margin and outside eats 8; both make the drawn rect "a number not in the file". The CSS prior agrees: modern practice is `box-sizing: border-box`, border inside the declared box.
- **Text — outside, expanding the ink.** The sub-question's "inside/outside/centred *the declared rect*" is a category error for text: per ADR-0007/0012, a text element's `width`/`height` are a **fit-claim the overflow check tests**, not drawn geometry — "the block of lines is the sum, and the block is placed according to `origin`". The meaningful choice is relative to the *glyph contour*. Inside-the-glyph erases thin stems (the notorious `-webkit-text-stroke` centred failure is the half-way version of this); outside never thins the fill, and it is the ASS `\bord` model this project's only real corpus descends from. Consequence to record: a stroked run's ink extents grow by `stroke_width` per side, `measure` must report stroked extents, and the overflow check's width and height terms consume them — the stroke participates in the fit-claim rather than silently escaping it.
- The unifying invariant, stated once: **a stroke never moves declared geometry and never thins a fill.** For a shape the declared rect *is* the geometry, so the stroke goes inward; for text the glyph is the geometry and the rect is a claim, so the stroke goes outward and the claim is checked against it.

---

## Q8 — does text still declare a `height`?

**First attraction: Option (b)** — `width` required, `height` optional, absence meaning no vertical container is claimed.

### Strongest attack I could mount

**Option (a)'s frozen-number argument, pressed on the edit the brief distinguishes.** Add a `\n` to an uncontained caption under (b) with `height` absent: the block grows, nothing fires, and the caption can grow into the photo below it with `validate` green. Under (a) the frozen height fires correctly — the finding means "this element is now taller than it was when the layout around it was built", which is a real regression signal, not a vacuous one. And (b) apparently breaks ADR-0012's flat statement "A size is required, not defaulted" — a uniformity argument with a settled ADR behind it.

### Whether it survived

**The attack lands a real hit but does not carry, and the uniformity half of it is false.**

- **The uniformity claim fails on ADR-0012's own reasoning.** The size-required rule exists because the tempting default (natural source size) has inputs *not in the document* and "fails quietly because a centre-cropped photo looks plausible". An absent text height's meaning — the typographic extent — is computed from `size`, `line_height`, and the `\n` count, all on the same line. It is understandable by reading, which is the actual principle the rule serves. ADR-0012 itself defaults `x`, `y`, `origin`, `scale`, `opacity`; "declared, never defaulted" was never a uniform surface rule, it is a rule about off-document inputs. `height` stays *legal* everywhere and *required* nowhere — the 7 container-copied heights keep working unchanged.
- **The frozen-guard is worth less than it claims, by its own maintenance dynamics.** The guard only stays meaningful if the author does *not* update the number on deliberate edits — but the fix `validate` teaches for every deliberate line-add is "recompute `ceil(size*line_height*lines)` and paste it in", i.e. re-deriving the check input from the thing being checked. After the first such edit the guard is a rubber stamp. A check whose maintenance procedure is "make it pass by construction" checks nothing on exactly the elements that have no container — which is 15 of 22 *because most captions have nothing behind them to overflow*.
- **Presence becomes signal under (b).** Under (a), 15 vacuous numbers are byte-indistinguishable from the 7 meaningful ones (the brief's own "byte-indistinguishable interim rects" problem, recurring in a new costume). Under (b), a written `height` *means* "there is a container here" — strictly more information in the same document.
- **The edit census, explicitly, since the brief demands it.** *Lengthening an existing line*: caught identically under (a) and (b) — that is the width term, `width` is required in both, and it is the axis where all 22 fixture values are real external constraints. *Adding a line to a contained caption* (the 7): caught identically — those elements keep their heights under (b). *Adding a line to an uncontained caption* (the 15): caught by (a) once, until the rubber-stamp dynamic sets in; missed by (b) by design. That residual miss is real and is my "bites first" below — but it is a *collision* problem (the block grows toward a neighbour), and none of the three options checks collision; (a) merely proxies it badly through a number that decays.
- **Option (c)** is dead on arrival: 15 findings on the only correct project file that exists violates ADR-0006's noise budget and ADR-0013's "a correct file should produce no findings" — the manufactured-false-confidence failure by name.

**Verdict: Option (b). Confidence: moderately high (~0.75).** `width` required; `height` optional; absent height = no vertical container claimed, extent is the typography; present height feeds the overflow check exactly as now. The ADR should also record what this does *not* fix: the 7 container-copied heights are still not bindings (resize the card, stale caption height, green validate) — that is ADR-0012's open placement-drift ticket, and neither (a) nor (b) touches it.

---

## Q9 — `line` and `polygon`: rejected or unevidenced?

**First attraction: Position A** — rejected, on placement grammar.

### Strongest attack I could mount

Two prongs. **First**: Position B is factually right about the reference class — Premiere's Essential Graphics panel does offer polygon shapes, so "ordinary primitives in the CapCut/Premiere class" is a true claim, and "rejected" then looks like it overstates, dangerously close to the forbidden juror move ("the fixture shipped only rectangles, so reject"). Fact 3 (a bezier-capable pipeline shipping six four-point axis-aligned drawings) is *corroborating colour at best* and must not be load-bearing — the standing rule forbids it. **Second**: the grammar claim is soft for `line` specifically — SVG places point-lists under transforms every day, so "no transform, origin or clip rule covers it" could be read as "we haven't written one", not "one cannot exist".

### Whether it survived

**Yes, because Position A's stated reason never touches the fixture, and the second prong concedes A's actual point.**

- A's argument is structural: every visual element is placed by `x`, `y`, `origin`, `width`, `height`, and a point-list defines its own extent — so `origin` has no box to name a ninth of, `scale` has no declared rect to be authoritative over, and ADR-0013's aperture-coverage check ("the declared rect must contain `clip`", computable with no probe) has no declared rect. That is not "unevidenced"; that is *incompatible with the settled placement grammar as it stands*. SVG solving it proves a second grammar is writable — which is exactly what A says: the day it is needed, that is **a new ADR about placement**, not a schema addition. The falsifiability is the point; recording it routes the future request to the right ticket instead of letting polygon arrive as "a schema value by implication", which ADR-0013 refuses by name.
- The already-settled "path is out of v1" makes B's position unstable: polygon is path minus curves, and line is polygon with two points. Holding path out while calling line/polygon merely "unevidenced" leaves no recorded reason a future contributor couldn't cite for adding polygon casually — reproducing the exact relitigation the CONTEXT.md **Rejected terms** section exists to prevent.
- `line`'s practical case is covered: a thin rotated `rect` draws any segment (rotation is float degrees; the sub-pixel length residual is under the 0.5px SPLIT tolerance ADR-0012 already accepts). The endpoint-pair-to-rect trigonometry is authoring friction, and the repo has a recorded home for that: *"wherever this format refuses a convenience, the convenience belongs in an authoring-time tool whose output is inert"* (ADR-0007). A segment helper that emits a rect is that tool.
- Discipline the record must keep: the ADR states the placement-grammar reason **only**, cites the reopening condition, and does not cite fixture non-use — if fact 3 appears at all, it appears as colour explicitly marked non-load-bearing.

**Verdict: Position A, rejected on placement grammar with the reopening condition recorded. Confidence: moderate (~0.7)** — the Premiere-has-polygon fact keeps this from being higher, but "rejected from this placement grammar, reopenable by a placement ADR" is a *decision with a falsifiable reason*, and B's "merely unevidenced" is a non-decision that invites the implication-shaped schema addition.

---

## What would bite me first

**The uncontained caption that grows a line.** Under Q8(b), an agent edits a two-line caption into three lines, `validate` stays green (correctly — no container is claimed), and the block's bottom third now sits on top of the photo below it. Nothing in any option here checks inter-element collision; (a) only proxied it through a number that decays into a rubber stamp. Second in line: under Q7-A, `stroke_width` under an animated `scale` — a pixel-matched 8px inside stroke on a card that Ken-Burnses to 1.08 draws 8.64px at peak, and an agent who pixel-matched an adjacent element at scale 1.0 will file that as a renderer bug. The element-space rule must be stated in the ADR with a worked number, ADR-0012 style. Third: run-level stroke extents — a stroked run mid-line widens the measured line, so every stroke edit re-enters the mandatory `measure` loop; authors who stroke a word and skip re-measuring get a silently stale width claim, the brief's own "goes stale silently" axis.

## Malformed or false presuppositions

1. **Brief fact 6 insinuates evidence that does not exist.** "All seven fixture ASS files carry `ScaledBorderAndShadow: yes`" is true and inert: every style has `Outline=0, Shadow=0` and there is not one inline `\bord`/`\shad`/`\3c`/`\alpha` tag in any file, and all 180 eight-digit colours are alpha-nibble-opaque. The old pipeline used no strokes and no colour alpha. Per the standing rule this argues nothing against Q7-A — but it must not be cited *for* it, and any ADR quoting fact 6 as stroke evidence would be quoting boilerplate.
2. **Q7's "these two resolve together" is only half true.** The scrim/fade channel-collision argument for `#RRGGBBAA` (static alpha in paint, animation in `opacity`) stands with no stroke anywhere in the schema. A jury could coherently ship alpha without stroke; bundling them hides that.
3. **The stroke sub-question's "same rule for shapes and for text" presupposes text has a declared rect in the same sense shapes do.** It does not — for text the rect is a fit-claim the overflow check tests, not drawn geometry (ADR-0007/0012). "Inside/outside/centred *the declared rect*" is a category error for text; the meaningful frame is the glyph contour, which is why my answer is legitimately non-uniform rather than inconsistent.
