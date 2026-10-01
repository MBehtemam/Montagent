---
status: accepted
amends: 0054 (its scope section: the four caption checks run on every `text` element except one carrying `caption: false`, which answers the question that section handed to #135)
---

# A text element opts out of the caption checks with `caption: false`

[#458](https://github.com/MBehtemam/Montagent/issues/458).
[ADR-0054](0054-caption-audio-backing-and-minimum-duration-checks.md)'s scope section runs all
four caption checks (`R-CAPTION-PACE`, `R-CAPTION-REPEAT-DURATION`, `R-CAPTION-NO-AUDIO` and
`R-CAPTION-MIN-DURATION`) on every `text` element, project-wide. It did that on purpose. It
left the caption-vs-decorative question to
[#135](https://github.com/MBehtemam/Montagent/issues/135) and said that whatever line #135
drew *"only narrows scope forward"*. #135 closed on a different question
([ADR-0061](0061-validate-judgment-boundary-threshold-provenance-and-a-fenced-exception.md),
threshold provenance) and never drew that line. So titles, lower-thirds, logos and kinetic
words all get caption findings: a motion piece of three short titles over no audio gets
`R-CAPTION-NO-AUDIO` under every one of them (`motion_piece` in
`crates/montagent-core/tests/caption.rs` reproduces it). The format gave an author no way to
say "this text is not a caption".

This ADR draws the line ADR-0054 handed to #135.

## The decision

**A `text` element accepts an optional boolean field, `caption`.** When it is missing it means
`true`, so every project that exists today validates exactly as it did. Writing `caption: true`
out behaves the same as leaving it out.

**`caption: false` takes the element out of all four caption checks.** None of them reports
it. That includes `R-CAPTION-REPEAT-DURATION`'s grouping of identical text: an opted-out
element is neither a member of a repeat group nor the reason one forms. The element is
removed before any of the four checks reads the document, not filtered out of each check's
output.

**It changes nothing else.** No pixel moves, and no check other than the four reads it.
`R-CAPTION-NO-AUDIO` still counts `audio` and `video` elements as backing whatever the text
around them says.

**A wrong value gets the finding any wrong field already gets.** A non-boolean `caption` on a
`text` element is `E-SCHEMA` at that element. `caption` on any other element type is
`E-SCHEMA-UNKNOWN-KEY`, because it is a field of `text` and of nothing else. No new code. A
mistyped value does not opt out: the check reads only the literal `false`, so a typo leaves
the element in scope rather than silencing four checks.

**Its key position is last** in `text`'s canonical order, after `effects`.
[ADR-0041](0041-canonical-key-order-is-schema-order-validate-checks-it-fmt-splits.md) gives a
new field's position to the ADR that introduces it. `caption` draws nothing, so it follows
every field that does.

## Why this shape

1. **An explicit field, not an inference.** Two inferences were available and both were
   rejected as fragile. A `highlight` window says a word lights up, and that is as true of a
   kinetic title as of a karaoke caption. Audio overlap is what `R-CAPTION-NO-AUDIO` measures,
   so inferring "not a caption" from it would make that check unable to fire on the defect it
   exists for. Track names were already ruled out by ADR-0054: a track name is a free-text
   label with no semantics anywhere else in this domain model. Only the author knows what a
   text element is for, so the author says it.
2. **Opt-out, not opt-in.** `caption: true` as the thing an author must write would quietly
   stop checking every existing project. ADR-0006's posture is that a missing signal must not
   read as a clean result, and opt-in is exactly that failure: a project that never heard of
   the field would be reported as having no caption problems.
3. **One switch for all four checks.** A per-check switch (`pace: false`, `audio: false`, …)
   was rejected as more surface than the problem needs. The question an author answers is
   "is this a caption?", and the four checks are four consequences of the same answer. Turning
   off one finding at a time is a general suppression mechanism, which this format does not
   have and this ADR does not add.
4. **It answers the question ADR-0054 handed to #135.** ADR-0054's scope section left the
   caption-vs-decorative line to [#135](https://github.com/MBehtemam/Montagent/issues/135) and
   promised that whatever discriminator it chose would *"only narrow scope forward"*.
   [ADR-0054](0054-caption-audio-backing-and-minimum-duration-checks.md) is honoured to the
   letter: `caption: false` only removes elements from the checks, and nothing about any of
   the four needs to be unwound.

## Consequences

- **A Caption is now a glossary term** in `CONTEXT.md`: a `text` element the four caption
  checks run on, which is any `text` element without `caption: false`.
- **`montagent://schema.json` and `montagent://format.md` describe the field**, and the schema
  description names what it silences.
- **Existing projects are unchanged.** No fixture moves, and no report changes by a byte,
  because no project written before this ADR carries the field.
- **Not covered.** Checks of caption timing against the audio on disk
  ([#565](https://github.com/MBehtemam/Montagent/issues/565)) decide their own scope. If they
  land, honouring `caption: false` is the obvious reading, but that is #565's call. Teaching
  the field to the Job skills is a follow-up.
- **A gap this ADR inherits rather than fixes.** Every optional field in the format reads an
  explicit `null` as if the key were omitted (`radius: null` parses), and `caption` does the
  same. A `null` therefore leaves the element in scope. Refusing `null` is a format-wide
  question, not this field's.
