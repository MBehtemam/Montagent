# Repair report — en-halloween-decorating.montagent.json

## What I changed, element by element, and why

The validator reported the same error on 8 elements: `gravity` is not a field
of this format, with the note "Which part of the source survives is
determined by `x`, `y`, `origin` and `clip`." The glossary excerpt
(`graveyard.txt`) confirms `gravity` was retired by ADR-0015: it named "which
part of a source survives a crop," but that quantity no longer exists once
the declared rect is authoritative at render — `x`/`y`/`origin`/`clip` already
fully determine it, leaving `gravity` nothing to do. The glossary also notes
that these fields typically enter a file by an agent copying a neighbouring
element, and that all eight occurrences were introduced (and later retired)
together — i.e. they are leftover copy-paste debris, not intentional data.

I deleted the `"gravity":"<value>",` key/value pair from each of the following
8 elements, leaving every other key, value, and the surrounding formatting
(one element per line, key order, indentation) untouched:

- `photo-05-intro` (had `"gravity":"top"`)
- `photo-05` (had `"gravity":"top"`)
- `photo-06` (had `"gravity":"bottom"`)
- `photo-07` (had `"gravity":"bottom"`)
- `photo-08` (had `"gravity":"top"`)
- `photo-05-quiz` (had `"gravity":"top"`)
- `photo-05-loop` (had `"gravity":"top"`)
- `handle-logo` (had `"gravity":"center"`)

For every one of these, I verified the accompanying `clip` value is a static,
literal frame-space rectangle already fully specifying what's visible:

- The seven `photo-*` elements all share `x:0, y:0, origin:"top-left",
  width:1080, height:1912` and `clip:[0,0,1080,1300]` — the clip window is
  identical regardless of what `gravity` value was attached (some say
  `"top"`, some say `"bottom"`), which is itself evidence the field was never
  read by the renderer: if it had been live, elements with different gravity
  values couldn't share byte-identical clip windows and still be correct.
- `handle-logo` has `clip:[478,96,68,68]` exactly equal to its
  `x,y,width,height` — the aperture exactly covers the rect, so there is no
  cropping happening at all, and `gravity:"center"` had literally nothing to
  bite on.

Because `gravity` never executed and its value is redundant with the already
fully-specified `clip`, this repair is a pure field deletion: no `x`, `y`,
`origin`, `width`, `height`, `clip`, `scale`, or any other field was touched,
so the rendered video is identical to what the file already produced (`fit`
and `gravity` both being non-executing, declarative-metadata-only fields per
the task's stated facts).

I re-validated the result is well-formed JSON and confirmed every track's
elements remain sorted by `start` and that no line other than the 8 listed
above changed.

## What I was unsure about

- **Whether to touch `clip` or leave it as-is.** In principle a `gravity`
  value that disagreed with the clip window (e.g. `"bottom"` on `photo-06`/
  `photo-07`, when the clip keeps the *top* 1300px of a 1912px-tall image)
  could be read as evidence that the *clip* was the thing wrong, and that the
  intended crop was actually the bottom of the image for those two elements.
  I did not "fix" clip, for two reasons: (1) the task's stated facts and the
  graveyard note are explicit and unconditional that `gravity` never executes
  and the renderer never reads it — the declared rect (here, `clip`) is
  solely authoritative, so there is no rendering-behavior reading under which
  `gravity`'s value should override or correct `clip`; and (2) the glossary
  says these fields are copy-paste debris common to eight-of-eight agents,
  which fits the pattern here (four different photo elements, three distinct
  gravity values, one shared clip window) — i.e. the mismatched gravity
  values are the artifact of sloppy copying, not a signal about intended
  framing. I could not fully rule out the alternative reading (that gravity
  encoded a real intended crop that was never migrated into `clip` when the
  field was retired elsewhere in the project), since I have no other source
  describing what each photo *should* show.
- **Whether `mask` on `handle-logo` needed any adjustment now that `clip`
  exactly equals the element's rect.** A `clip` equal to the full rect makes
  the aperture a no-op, and the visible shape is then governed entirely by
  `mask:"circle"`. I left it alone — nothing in the validator output flagged
  `mask`, and the task rules said to repair only what's broken and not
  reformat/reorder/adjust anything else.
- **Trailing comma mechanics.** Deleting `"gravity":"<value>",` (value plus
  trailing comma) rather than `,"gravity":"<value>"` (leading comma) matters
  for JSON validity depending on position; since `gravity` was always
  followed by `clip` in all 8 occurrences (never last in the object), either
  convention produces valid JSON here, but I removed the trailing-comma form
  consistently and confirmed the result parses and matches the original
  minus exactly that substring.

## What I wanted and did not have

- The actual source images (`images/05.png` … `images/08.png`,
  `brand/logo-en.png`) to visually confirm what the intended crop/framing
  looks like, which would have let me confirm with certainty that the
  now-sole-authoritative `clip` values already produce the framing the author
  wanted (rather than merely being self-consistent JSON).
- Any changelog, migration note, or prior version of this exact file from
  before `gravity` was introduced, which would show whether `clip` was ever
  adjusted to compensate when `gravity` was added, or whether `clip` has been
  frame-space-correct for these elements all along. I was told not to consult
  the Montagent repository or ADRs beyond the provided `graveyard.txt`
  excerpt, so I relied solely on that excerpt and the task's stated facts.

## Confidence

High confidence that the file now validates and that the JSON is
byte-identical to the original except for the 8 deletions described above —
this was checked mechanically (JSON parse, sort-order check, and a
line-by-line diff limited to the 8 edited lines).

High-but-not-total confidence that the *video* is unchanged: the task's
stated facts assert unconditionally that `gravity` (like `fit`) never
executes and the renderer never reads it, so removing it cannot change any
rendered frame by definition. What I could not check is whether the file was
already producing the intended video *before* my edit — e.g., whether the
`clip:[0,0,1080,1300]` windows correctly show the top of each 1912px-tall
source image the way the author intended, especially for `photo-06` and
`photo-07`, whose retired `gravity` value ("bottom") disagreed with the
shared clip window. I did not have the source images to check this, and per
the task rules did not treat the mismatch as license to change `clip`, since
doing so would be changing the video rather than repairing the format error.
That question — is the crop itself correct — is outside what `montagent
validate` can tell you and outside what I could verify from the materials
provided.
