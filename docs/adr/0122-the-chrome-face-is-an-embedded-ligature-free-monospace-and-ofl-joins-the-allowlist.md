---
status: accepted
amends: 0090 (widens the permitted licence set by one identifier, `OFL-1.1`, with the reasoning its "what this does not decide" asked for; and extends its derived-not-asserted rule to a font the binary embeds, which no manifest reports), 0098 (discharges decision 9's open question — which face, its licence bucket under the vendoring gate, and what "the build fails loudly" is mechanically)
---

# The chrome face is an embedded, ligature-free monospace, and OFL-1.1 joins the allowlist

**Ticket:** [#421](https://github.com/MBehtemam/Montagent/issues/421), under the spec
[#486](https://github.com/MBehtemam/Montagent/issues/486); it blocks
[#490](https://github.com/MBehtemam/Montagent/issues/490).

## What was open

[ADR-0098](0098-the-tile-label-is-a-floored-fitted-line-naming-the-change-at-its-own-boundary.md)
§9 ruled that a contact-sheet tile label is drawn in a face Montagent vendors for its own
chrome. It is never the project's declared font and never a system font. The ADR left four
requirements to whoever chose the face:

- a permissive licence passing [ADR-0057](0057-font-vendoring-licence-gate-and-path-keyed-attestation.md)'s
  gate and added to [ADR-0090](0090-attribution-is-scoped-to-the-distributed-binary-and-the-licence-list-is-an-allowlist.md)'s
  allowlist;
- tabular figures;
- ASCII coverage with a visible replacement glyph;
- a build that fails loudly without it.

## Decisions

### 1. The face is JetBrains Mono NL Regular, v2.304

Taken unmodified from the project's own release archive,
`JetBrainsMono-2.304.zip` → `fonts/ttf/JetBrainsMonoNL-Regular.ttf`, and committed at
`crates/montagent-core/fonts/` beside its `OFL.txt`.

**Monospace, rather than a proportional face with a `tnum` feature.** Tabular figures are
what ADR-0098 asks for, and a monospaced face has them with no feature switched on. The
decisive reason is the width law, though. ADR-0098's 8 px floor and its measured
`295/chars` law come from a proportional face, and hold to ±9 %. In a face where every
glyph advances 600/1000 em, a label of *n* characters at size *s* is exactly
`n × 600 × s / 1000` px wide. The law stops being a fit and becomes arithmetic: 180 px ÷ 0.6
gives 300/chars, inside the measured band. The label sizing (#490) can then decide
sheet-wide id elision in integers, without shaping a label to find out.

**The `NL` cut, because ligatures would break both of the things above.** The standard
JetBrains Mono joins `->`, `--`, `==` and `!=` into single glyphs, and label ids are free
text. A ligature redraws a label's characters as something the id does not spell. It also
makes an unshaped width disagree with what the shaper lays out, so a label judged to fit
would not.

**JetBrains Mono rather than another OFL monospace.** It has a large x-height and
distinguishes `0`/`O` and `1`/`l`/`I`, which matters at an 8 px floor where the payload is
digits next to ids. It has a no-ligature cut published by its own project, and its licence
names no Reserved Font Name. No Apache-2.0 monospace was found that is still maintained:
Google Fonts' `apache/` directory holds none, and Roboto Mono and Cousine have both moved
to OFL.

### 2. Absence is a compile error: `include_bytes!`

`montagent_core::fonts::chrome` embeds the file with `include_bytes!`. A checkout without
it does not build, and no code path could fall back, because no second face is reachable
from the chrome registry. The motivating accident was #396's prototype drawing its labels
in whatever font happened to sit in the fixture directory, and that is now structurally
impossible rather than checked for. CI's presence check is the build itself. The attribution
script also fails on a missing file, naming the reason.

`montagent-text` gains `Fonts::register_bytes`. It registers a face from memory under the
same rules as a chain entry — cut to its `index` (ADR-0102), a synthetic family name, the
same refusal sentence — and adds nothing to `Fonts::opened`, which lists files read. The
chrome face is registered only in a registry that holds nothing else, so project text can
never shape in it.

### 3. The metrics the sizing needs are exposed as integers, and tested against the shaper

`montagent_text::FaceMetrics` reads a face's em, ascender, descender, line gap and unshaped
advances as the integers the file stores. `chrome` also carries them as constants:
`UNITS_PER_EM = 1000`, `ADVANCE = 600`, `ASCENDER = 1020`, `DESCENDER = -300`.

An unshaped advance equals the shaped width only for a face with no kerning, ligatures or
contextual alternates. That is a property of *this* face, not of the reader, so
`tests/chrome_face.rs` asserts it against the shaper at one unit per pixel and at the 8 px
floor. The test strings hold the pairs a ligature font would join.

### 4. The face passes ADR-0057's gate as it stands

It is held to the same gate `fonts vendor` applies to a project's font, not to a softer
one. Asked of the embedded bytes, the gate returns **bucket 2, recognised open, `OFL-1.1`**,
read off the face's own `name` table.

### 5. `OFL-1.1` joins the allowlist

ADR-0090's allowlist holds nothing copyleft because of ADR-0009's invariant: the
distributed binary must never become a copyleft combined work. OFL-1.1 has a reciprocal
clause, but it cannot reach the binary. Its condition 5 binds only *"the Font Software,
modified or unmodified, in part or in whole"*. Its condition 2 expressly permits the font
to be *"bundled, redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license"*. Its closing sentence says the
requirement to remain under the licence *"does not apply to any document created using the
Font Software"*.

So the obligations are the notice, which `THIRD-PARTY.md` carries in every release archive
and `crates/montagent-core/fonts/OFL.txt` carries in the crate tarball, and not selling the
font on its own, which Montagent does not do. The licence of Montagent's own code is
untouched.

The identifier is added to `about.toml`'s `accepted` and `deny.toml`'s `allow`. Both lists
now say the same thing, as ADR-0090 requires. It is admitted for any component rather than
for fonts alone: the argument above is about what the licence binds, not about the file
extension, and a crate that ships a font under it is owed the same answer.

### 6. The font is attributed from `bundled.json` and checked in full

No manifest reports a file `include_bytes!` put in the binary, so ADR-0090 §3's rule —
derive, never assert — applies here as it does to the Skia prebuilt.
`ci/third-party-notices/bundled.json` gains a `fonts` list: name, SPDX identifier, source
archive, path, SHA-256 and notice. `ci/assert_third_party_attribution.py` checks that each
file:

- is present;
- hashes to the recorded digest;
- has its notice present;
- has its identifier on both allowlists.

It then renders a *Fonts* section into `THIRD-PARTY.md` with the licence text in full.
`montagent-core`'s test checks the same digest against the embedded bytes, so the attested
file and the one in the binary cannot be two different files.

`THIRD-PARTY.md`'s old *"No font ships in the binary"* is withdrawn. A project's vendored
fonts still belong to the project that vendors them.

## Consequences

- The binary grows by 208,576 bytes. That is small against the Skia prebuilt, and it is the
  price of a label that looks the same on all six targets.
- A future chrome face, or a second weight, is a new `bundled.json` entry, and the same
  script checks it.
- ADR-0098's `295/chars` law remains the measured record for the prototype's face. For the
  shipped face it is replaced by exact arithmetic, and #490's table tests are written
  against `chrome::ADVANCE` rather than against the law.
