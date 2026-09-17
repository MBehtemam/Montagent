---
status: accepted
amends: 0034 (settles the audio-backing and minimum-duration deferrals)
---

# `validate` gains two more caption checks: no audio backing, and a minimum display duration

[ADR-0034](./0034-caption-pace-and-repeat-duration-checks.md) named two defect
classes it deliberately did not build — audio backing and an absolute duration
floor — and parked both as map fog, graduated to
[#124](https://github.com/MBehtemam/Montaget/issues/124). This ADR designs both,
resolves a wrong premise in each, and gives `validate` two more `R-CAPTION-*`
checks.

## Check 3: `R-CAPTION-NO-AUDIO`

**The premise that this needs media analysis was wrong.** ADR-0034 recorded audio
backing as needing "media analysis, and narration-vs-ambience discrimination to
say more than raw presence." On the fixture, `hook-loop`
(`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json`,
64016–65216ms) is the actual defect this check exists for — and its
`narration` track's last element ends at 63300ms. Whether any audio element
overlaps a text element's time range is answerable from declared `start`/`end`
values alone. **No file is read; this is a pure document-level check**, the same
"no I/O" shape as `R-CAPTION-PACE` and `R-CAPTION-REPEAT-DURATION`.

**Narration-vs-ambience discrimination stays out of reach, categorically.**
CONTEXT.md states Montaget "is deterministic and contains no model" — telling a
spoken voice apart from music or ambience in an audio file is a classification
problem with no crisp deterministic boundary. This is a constitutional
exclusion, not a cost or scheduling one: nothing about staging a lighter
heuristic (energy thresholds, zero-crossing rate) escapes it, since a tuned
heuristic aimed at a classification task behaves like a model in every way that
matters here. Recorded as its own permanent fog entry, not a "future work" item
implying it merely awaits engineering time.

**Trigger.** A text element is flagged if **zero elements of any kind whose
`type` is a time-based audio source** overlap any part of its
`[start, end)` window — a project-wide interval query, no track-name
involvement.

**What "any audio" deliberately does not mean.** This is presence, not
narration coverage. A text element sitting entirely under a continuous music
bed, with no spoken line anywhere near it, passes this check — the same
"under-flagging is the acceptable error direction" posture ADR-0034 already
adopted for its CJK cps gap. **The named trigger for revisiting this:** the
first real project that pairs a persistent non-narration audio track (music,
ambience) with captions, where this check's silence would read as false
reassurance rather than a narrow gap. Closing that gap requires the project
format to declare an audio element's role (narration vs. music vs. sfx) —
a schema addition with its own cost, not proposed here, and not worth building
against a hypothetical the current single-track fixture cannot exercise.

**A second, stronger check is a different thing and stays in fog.** Whether a
*declared* overlapping audio element's actual file content is silent (wrong
file, corrupted encode, the same "truth changed on disk" class of defect
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) already
established `validate` must catch elsewhere) requires reading the waveform —
deterministic signal analysis (e.g. `ffmpeg silencedetect`, already used as a
non-model technique in [#9](https://github.com/MBehtemam/Montaget/issues/9)'s
research), not a document query. It has different inputs, a different failure
mode (a forgotten element vs. a correct element pointing at a bad file), and a
different operational cost (I/O and a threshold constant of its own). Recorded
as a future check, not folded into this one.

**Severity: `review`.** Silence under a caption can be deliberate — ADR-0005:
"silence in an audio track is information" — so this is not `error`. But a
caption with nothing spoken under it is usually a forgotten narration line, not
a stylistic choice, so it is not something a reader would skim past either;
`note` undersells it. `review` — legal, renders, you must look at (or listen
to) the moment to know if it was meant — matches ADR-0006's definition exactly
and keeps this check at the same severity as its three siblings.

## Check 4: `R-CAPTION-MIN-DURATION`

**The ticket's premise — "5-6 frames minimum" — does not survive its source.**
Netflix's Timed Text Style Guide *General Requirements* page (script-agnostic,
sitting above the per-language cps tables already surveyed for
[#123](https://github.com/MBehtemam/Montaget/issues/123)) states a universal
minimum caption duration of **5/6 second**, not a frame count. "5-6 frames" was
a downstream, fps-specific approximation of that duration read back as if it
were the primitive; at the fixture's 25fps, 5/6s is 20.83 frames — not an
integer, unlike the repeat-duration check's genuinely frame-native one-frame
tolerance (which exists to absorb frame-snapping float noise, a fact about the
rendering grid). A human-reading-time constant is not a grid fact.

**Threshold.** A fixed **834ms** — 833.33ms (5/6 second) rounded up to the
stricter integer millisecond, so a caption at exactly 833ms is correctly
flagged rather than passed by a rounding artifact — independent of the
project's `fps`. This sits in the same register as `R-CAPTION-PACE`'s 20 cps: an
externally documented constant about human reading capacity, not a property of
the render.

**Trigger.** A text element's `end - start` is less than 834ms.

**Severity: `review`.** A sub-floor caption renders and is not guaranteed
wrong — a deliberate rapid flash-word effect is a real, if unusual, device, and
nothing in the document says which this is. Matches `R-CAPTION-NO-AUDIO` and
both of ADR-0034's checks.

## Scope: both checks apply to every `text` element, project-wide

ADR-0034's two existing checks carry no track-name restriction anywhere in
their mechanics — they operate on any element's `runs`, `start`, `end` — even
though their worked examples happen to come from a track named `caption`. Both
new checks inherit that same scope rather than introduce a second, inconsistent
rule keyed on track name, which is a free-text label with no semantics
elsewhere in this domain model (an author could as easily name the track
`subs` or `lower-third`).

**The motivating case for [#135](https://github.com/MBehtemam/Montaget/issues/135):**
a persistent decorative element — a looping video's static logo overlay, on
screen for the whole runtime — in a hypothetical project with no audio
anywhere produces a spurious `R-CAPTION-NO-AUDIO` finding, because nothing in
the document distinguishes "this text is a spoken-line caption" from "this text
is decorative UI." That line is explicitly not drawn here; it is #135's
question. Scoping today's checks to "all text" adds no new category of false
positive — an all-text-scoped `R-CAPTION-PACE` already carries the identical
exposure — and whatever discriminator #135 eventually settles on only narrows
scope forward. Nothing about `R-CAPTION-NO-AUDIO` or `R-CAPTION-MIN-DURATION`
needs to be unwound once it lands.

## Consequences

- **`validate` gains `R-CAPTION-NO-AUDIO`** (no audio element of any kind
  overlaps a text element's window, `review`) and **`R-CAPTION-MIN-DURATION`**
  (`end - start` < 834ms, `review`), both scoped to every `text` element
  project-wide, matching ADR-0034's existing checks.
- **`R-CAPTION-NO-AUDIO` is a zero-I/O document check**, correcting ADR-0034's
  assumption that audio backing requires media analysis; a genuinely stronger,
  waveform-reading version (declared-but-silent audio) is recorded as a
  separate future check, not built here.
- **Narration-vs-ambience discrimination is recorded as permanently out of
  reach** under CONTEXT.md's no-model constraint, not merely deferred.
- **No schema change.** Both checks read data the format already carries.
- **One fog entry graduates into a named future ticket trigger**: an audio
  element role field, motivated specifically by the first project that pairs
  captions with a persistent non-narration audio track.
- **[#135](https://github.com/MBehtemam/Montaget/issues/135) gets a concrete
  motivating case** (the looping-logo scenario) rather than an abstract
  boundary question.
