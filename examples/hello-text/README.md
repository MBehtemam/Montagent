# hello-text

The shortest honest path to a first rendered frame (#369): two commands, run from
wherever you unpacked the release archive.

```
montagent fonts vendor examples/hello-text/hello-text.montagent.json examples/hello-text/OpenRunde-Bold.otf --licence OFL-1.1
montagent render examples/hello-text/hello-text.montagent.json
```

That writes `out/hello-text.mp4` — 90 frames of white text on a dark background.

## Why the first command needs `--licence`

`hello-text.montagent.json` already declares `OpenRunde-Bold.otf` in its `fonts` table,
but the file has not been vendored into the project yet — there is no `fontVendor`
attestation for it. `fonts vendor` copies the font's bytes in, and records who verified
its licence.

Normally `fonts vendor` reads the licence straight out of the font's own name table
(OTF name IDs 13/14) and needs no flag from you. Open Runde's name table does not carry
those fields, so the licence can't be read back from the file — `--licence OFL-1.1`
is you, a human, stating what `OpenRunde-LICENSE.txt` (shipped alongside the font in
this directory) already says. This is the normal path for a font like this one, not an
exception.

`--licence` is refused only when a font's licence text is machine-recognisable (the flag
would be redundant) or when the font is on Montagent's blocklist (nothing lifts that).
