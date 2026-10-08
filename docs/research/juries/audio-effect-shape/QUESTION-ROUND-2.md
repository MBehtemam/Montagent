You are a juror. Answer as an AI agent that authors and edits Montagent projects: you read the JSON document, edit it by exact-string replace, run `validate`, and render. Judge each option by how well you, as that agent, would work with it. Do not use any tools. Do not edit anything. Answer only in the ballot format at the end.

## Background (facts, not recommendations)

Montagent is a video-editing format: a JSON document an agent writes, which a Rust renderer turns into video. Its standing rules: every value is literal, the vocabulary is closed, every value is checkable by `validate`, and every value is editable by exact-string replace. Keys are snake_case (`source_start`, `stroke_width`). `fmt` rewrites the document into a canonical form when it is written, so the same value is always spelled the same way. Precedent is looked for first in CapCut and Premiere.

- **Visual effects** (ADR-0040): `"effects": [{"name": "blur", "radius": 4}, {"name": "shadow", ...}]` is an ordered list on the element. `name` discriminates a closed union. Order is meaningful. Two members of the same name are ordinary; the schema does not forbid them, and `review` may flag a duplicate. Members carry no id, so an agent singles one out by its literal text within the element's block. The glossary already has a term **Colour filter** for four visual `effects` members.
- **Audio today** (ADR-0055): `volume`, a flat, keyframable linear multiplier on `audio` and `video` elements.
- **Already decided for audio effects (this effort):**
  - An `audio`/`video` element gets an **ordered list** for signal-shaping audio effects: EQ, dynamics (compressor, limiter, gate), loudness normalisation, restoration (noise reduction, de-ess), and creative effects (reverb/echo, pitch, voice changers). The list is separate from the visual `effects` list and is not a branch of it.
  - **Gain and routing controls are flat fields**, not list members: `volume`, and later pan/balance and channel operations (mono, swap, one channel only).
  - The per-element graph is `atrim → atempo (speed) → aloop (loop) → [audio effect list] → volume → adelay (placement) → amix`, so the list runs after `aloop` and before `volume`.
  - A `video` element carries the same audio list as an `audio` element, alongside its visual `effects`.
  - A separate master stage on the final mix is decided elsewhere. Whether parameters are keyframable is decided elsewhere.
- **Premiere precedent:** every effect in a clip's audio rack has an on/off (bypass) toggle. Clip-intrinsic controls are Volume, Channel Volume and Panner, with Panner applied after Volume. EQ, compressors and the like may each appear more than once in a rack.
- **Evidence bar for this effort:** each audio capability's prototype is rendered as a loudness-matched A/B against the bypassed source.

## The questions

**Q4. What is the list field called?**
- (a) `audio_effects`
- (b) `filters`
- (c) `sound` (or `audio`)
- (d) `fx`
- (e) something else (name it)

**Q5. What does one list member look like?**
- (a) The same tagged shape as `effects`: `{"name": "highpass", "frequency": 80}`, with `name` as the discriminator and parameters flat beside it.
- (b) A nested shape: `{"highpass": {"frequency": 80}}`.
- (c) `{"type": "highpass", "params": {"frequency": 80}}`.

**Q6. Is there a per-member bypass?**
- (a) No bypass field. To bypass an effect, remove it.
- (b) An optional `"enabled": false` on any member.

**Q7. What happens when the same kind appears twice on one element** (two `highpass` members, or two loudness normalisations)?
- (a) Always allowed, as with visual `effects`; `review` may flag duplicates.
- (b) Always a `validate` error.
- (c) Allowed by default, but each capability's own ADR may declare its member singular, which makes a second copy a `validate` error (for example, loudness normalisation, where a second target can only override the first).

**Q8. How is one member singled out for an exact-string replace edit?**
- (a) By its literal text within the element's block, as visual effects are today (canonical `fmt` key order makes that text stable).
- (b) An optional `id` field on each member.
- (c) By index, through a write tool.

**Q9. Where do the flat routing controls (pan/balance, channel operations), when they arrive, sit in the per-element graph?**
- (a) After `volume`: `[list] → volume → [routing] → adelay`.
- (b) Before the list: `aloop → [routing] → [list] → volume`.
- (c) Between the list and `volume`: `[list] → [routing] → volume`.

A juror may reject the framing of any question outright; that is a valid ballot.

## Ballot format

Give one block per question (Q4 to Q9), exactly in this form:

🗳️ **Juror <n>** (<the model backing you>) — **Q<k> VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <why not the others, and what your choice costs>
