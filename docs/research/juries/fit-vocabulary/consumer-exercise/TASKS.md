# Authoring tasks

You are an agent authoring a Montaget project file. Work ONLY from `SPEC-<arm>.md`, given to
you. **Do not read anything under `docs/adr/`** — those are internal decision records, not
published format documentation, and reading them invalidates this exercise.

You may read the existing project file at
`fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json` for context, and you
may use `python3` to compute. Do not edit that file — write your answers in your worklog.

The project frame is 1080x1920.

For **each** task: write the complete JSON element (or the changed fields), and show the
arithmetic you used.

1. **New photo.** Add a photo element. Source `images/09.png` is **1536x2720**. It must fill
   the same aperture the other photos use: `clip` `[0,0,1080,1300]`, at `x:0, y:0`,
   `origin:"top-left"`. Times 30000-36000.

2. **A source was re-exported.** `images/06.png` was 1536x2720; it is now **1536x2200**. The
   element currently says `"width":1080,"height":1912,"fit":"cover"` with clip `[0,0,1080,1300]`.
   `validate` now reports an error. Fix the element so validate passes AND the aperture is
   still completely covered.

3. **A badge that must not be cropped.** Place `brand/badge.png`, **1200x400**, entirely
   visible inside the aperture `[840,1700,200,160]`. Nothing may be cut off.

4. **Deliberate distortion.** The art director wants `images/07.png` (**1536x2720**) squashed
   to exactly 1080x1600 in the aperture `[0,0,1080,1300]` — the wrong aspect ratio, on
   purpose. Note: this element also animates `scale` with keyframes, and that animation must
   stay readable and untouched.

5. **A rotated phone photo.** `images/10.jpg` has stored pixel dimensions **3024x4032** and
   carries an EXIF orientation flag of **6** (rotate 90 degrees clockwise for display). It
   must cover the aperture `[0,0,1080,1300]`. What `width`/`height` do you write?

6. **Crop to the top of a tall photo.** `images/11.png` is **1080x3000**. The author wants
   only the **top third** of the photo visible, in the aperture `[0,0,1080,1000]`. Write it.

## Worklog — write this, it is the point of the exercise

For every task record:
- **Wrote:** the element/fields.
- **First reached for:** the very first field name or value you considered, BEFORE checking
  the spec. Be honest — if you first thought of a field or value that turned out not to exist
  or not to be legal, say exactly what it was. This is the most valuable thing you can report.
- **Hesitated:** anything you had to re-read the spec to resolve, or were unsure about.
- **Wanted an error message saying:** if you got something wrong, what message would have
  fixed you fastest.

End with **## Verdict**: is this vocabulary usable as published? Name the single worst thing
about it, and one concrete change you would make.
