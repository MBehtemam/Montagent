# Media in this repository is not covered by the MIT licence

**The MIT licence in [`LICENSE`](LICENSE) covers the source code and the documentation.**
It does **not** cover the images, audio and video committed here as test fixtures and
research evidence. Those files fall into the two categories below, and this file is the
canonical list of them. Where a media directory carries its own `LICENSE` or a licence
note, that note repeats what is written here; if the two ever disagree, this file wins.

Nothing here restricts the code. Clone the repository, read it, build it, fork it, vendor
it — MIT, unchanged. The carve-out exists because a bare MIT grant over the whole tree
would hand anyone the right to take a language channel's voice, photography and logo and
sell them, which is not a right the maintainer is offering and, for the third-party file
in category 2, not a right the maintainer has.

---

## 1. Rights reserved — the maintainer's own media

**© 2026 Mohammad Bagher Ehtemam. All rights reserved.**

These files are the assets of an already-published short from the maintainer's own
language channel, and renders derived from them. They are committed **so that this
repository's tests and research can be judged against real input instead of plausible
invented input**, and that is the only use granted:

> Permission is granted to use these files **only** to build, test, benchmark and study
> this repository — including in a fork of it, and including running the repository's
> test suite and research harnesses. Any other use, and in particular any commercial
> use, any redistribution apart from this repository, any use of the channel's logo,
> photography or recorded voice in another work, and any use to train a generative
> model, requires the copyright holder's written permission.

### The fixture

`fixtures/en-halloween-decorating/` — every file in the directory **except**
`fonts/OpenRunde-Bold.otf` and `fonts/OpenRunde-LICENSE.txt`, which are category 2.
That is:

| path | what it is |
| --- | --- |
| `images/05.png` `06.png` `07.png` `08.png` | the four source stills |
| `audio/*.mp3` (11 files) | the recorded voice-over |
| `brand/logo-en.png` | the channel badge |
| `reference/en-halloween-decorating.mp4` | the published short |
| `reference/kenburns/05.mp4` … `08.mp4` | the pan/zoom clips derived from the stills |
| `reference/frame-intro.png` `frame-05-at-11s.png` | two frames of the published short |
| `reference/subtitles/*.ass`, `reference/beats.json`, `transcript.json`, `en-halloween-decorating.montagent.json` | the script, the timing and the composition |

`fixtures/video-decode/` adds **no media of its own** — its `source` is the published
MP4 above, reached by a relative path — so its rendered output is derived from this
category even though the directory holds only JSON.

### Renders derived from the fixture

Each of these is a committed picture that **composites the channel's photography, its
logo, or both**. A render of copyrighted media is a derivative of it, so the same terms
apply to the render:

| path | files |
| --- | --- |
| `crates/montagent-core/tests/golden/` | `fixture-intro.png`, `fixture-sentence.png`, `paint-vocabulary.png`, `mask-under-transform.png`, `mask-explicit-rect.png` — **five of the six**. `text-features.png` draws only text and is MIT with the rest of the code. |
| `docs/research/prototypes/rust-rasterizer/frames/` | all 11, including `frames/oracle/` |
| `docs/research/prototypes/preview-floor-resolution/` | all 64 under `frames/`, `crops/` and `thumbs/` |
| `docs/research/prototypes/proxy-preview-savings/frames/` | all 4 |
| `docs/research/prototypes/heavy-composite-stack/frames/` | all 4 |

The last two composite **synthetic** `testsrc2` media rather than the channel's
photography, but they still draw `brand/logo-en.png` and the channel handle, so they are
listed here rather than treated as clean.

**Not in this category**, and MIT along with the code: the frames under
`docs/research/prototypes/thai-line-breaking/` and
`docs/research/prototypes/thai-vertical-metrics/`. Those are text typeset on a blank
ground, with no channel media in them at all. The **fonts** that typeset them are a
different matter and are category 2 below — the frames are ours, the faces are not.

### The skills-eval asset pack

`docs/research/skills-eval/assets/`: the fixed pack every run of the skills eval
([#465](https://github.com/MBehtemam/Montagent/issues/465)) starts from. The pack is
also listed, file by file, in its own `README.md`. Everything in it is in this category
**except** `fonts/` (category 2 below) and `music/bed-120bpm.wav` (dedicated CC0 1.0;
see the note after the table).

| path | what it is |
| --- | --- |
| `brand/*.png` | the Montagent mark, wordmark and lockup, drawn by `pack-src/make_brand.py` |
| `voiceover/voiceover.wav`, `voiceover.words.json`, `script.txt` | a synthetic voiceover spoken by Kokoro-82M (Apache-2.0 weights and voice; the output is ours) |
| `presenter/*.txt`, `presenter/*.words.json` | the presenter takes' scripts, and their word timings made by `pack-src/align_takes.py` |
| `screen/*`, `stills/*` | a screen recording of a real Montagent session, and stills from it |
| `character/*` | Hoot, the owl mascot: a cut-out rig, its voice lines with visemes, a set and a prop (see below) |

**The presenter takes are Azure avatar output.** `presenter/take-{1,2,3}.mp4` were
generated with Azure AI Speech's text-to-speech avatar (the prebuilt avatar "Harry",
casual style) on the maintainer's subscription, and are committed as delivered. The
person on screen is synthetic. Microsoft's Product Terms and the Azure AI Speech code of
conduct govern them, including its requirement to disclose that the presenter is
synthetic, which the pack's `README.md` does. They are in this category because the
maintainer generated them, not because the maintainer is on camera.

**The character is FLUX and Azure Speech output.** The drawings in `character/` (and
their sources in `pack-src/character/`) were generated with FLUX.2-pro on Azure AI
Foundry, and `character/voice/*.wav` with Azure AI Speech (`en-US-AvaNeural`), on the
maintainer's subscription and under Microsoft's Product Terms. The voice is synthetic.
The rig's parts, its five mouths, the set and the prop were cut and drawn from them by
`pack-src/character/cut_rig.py` and `make_props.py`. The drawings in `pack-src/character/`
are in this category too, like the pack made from them.

**The music bed is CC0.** `music/bed-120bpm.wav` is synthesised from scratch by
`pack-src/make_music_bed.py`, which writes the same bytes on every run. The maintainer
dedicates it to the public domain under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). It is the one media file
in the repository with no restriction on it at all.

### On crates.io

Five of the six goldens above are excluded from the published `montagent-core` package
(`exclude` in `crates/montagent-core/Cargo.toml`), because a crate's `license = "MIT"`
field is a machine-readable claim over every byte in its tarball and that claim would be
false for them. The excluded files are test data for tests that already cannot run from a
published tarball — they read the fixture through `../../fixtures/`, which is outside the
crate. Nothing else in this file reaches crates.io.

---

## 2. Third-party media, under its own licence

Not the maintainer's, so **not the maintainer's to place under MIT and not the
maintainer's to reserve rights over either**. Each is governed by the licence named here.

### `fixtures/en-halloween-decorating/fonts/OpenRunde-Bold.otf`

**Open Runde**, derived from Inter. © 2016 The Inter Project Authors, under the **SIL
Open Font License, Version 1.1**. The full licence ships beside the file as
`fonts/OpenRunde-LICENSE.txt` and that attribution is already correct; this entry only
records that the font is not covered by either the MIT grant or category 1 above.

### `docs/research/skills-eval/assets/fonts/`

**Inter 4.1**, Regular and Bold, © 2016 The Inter Project Authors
(<https://github.com/rsms/inter>), under the **SIL Open Font License, Version 1.1**. They
are copied unmodified from the v4.1 release's `extras/ttf/`, with the full licence beside
them as `Inter-LICENSE.txt`.

### `docs/research/prototypes/thai-vertical-metrics/fonts/`

Two Thai text faces, each under the **SIL Open Font License, Version 1.1**, with the full
licence text committed beside it as the OFL itself requires:

| file | font | copyright | licence beside it |
| --- | --- | --- | --- |
| `NotoSansThai-Regular.ttf` | Noto Sans Thai | © 2022 The Noto Project Authors (<https://github.com/notofonts/thai>) | `NotoSansThai-OFL.txt` |
| `Sarabun-Regular.ttf` | Sarabun | © 2018 The Sarabun Project Authors (<https://github.com/cadsondemak/Sarabun>) | `Sarabun-OFL.txt` |

They were downloaded for [ADR-0087](docs/adr/0087-thai-line-height-collision-is-a-font-selection-problem.md)'s
line-height measurements and are **also the fixture ADR-0102's `.ttc` tests are built
from** — which is why they are committed rather than downloaded on demand. A test that
reads a font the repository does not carry passes only on the machine that fetched it,
and that is the precise failure ADR-0102 exists to prevent
([#432](https://github.com/MBehtemam/Montagent/issues/432)).

Like Open Runde above, neither font is the maintainer's to place under MIT or to reserve
rights over; this entry records that they are governed by the OFL and nothing else.

### `docs/research/chroma-key/green-screen-trex.mp4`

Downloaded from Pixabay, by **Exceptional_3D**, under the **Pixabay Content License**:

- source: <https://pixabay.com/videos/dinosaur-trex-reptile-roar-extinct-170831/>
- licence: <https://pixabay.com/service/license-summary/>

The Pixabay Content License grants free commercial and non-commercial use and requires no
attribution; the attribution above is given anyway, because a reader of this repository
needs to know the file is not ours. It is committed as the forcing case
[ADR-0088](docs/adr/0088-chroma-is-a-matte-operation-and-color-stays-literal.md) measures
its keyer against, and `chroma_key_scan.sh` beside it re-derives every number from it.

> **An unresolved question, recorded rather than glossed.** The file is committed
> **verbatim as downloaded** — 1920×1080, 24 fps, 140 frames, no edit. The Pixabay terms
> forbid distributing Content *"on a Standalone basis"*, which they define as *"where no
> creative effort has been applied to the Content and it remains in substantially the same
> form as it exists on the Service"*. Every example the terms give of that prohibition is
> a resale — prints, wallpapers, posters, NFTs, merchandise — and a test fixture inside a
> source repository is plainly not one. But it is equally plainly an unedited copy being
> redistributed, and this repository is about to become public. The question is open and
> owned by its own issue; it is **not** settled by this file.

---

## 3. Everything else

MIT, per [`LICENSE`](LICENSE). That includes every Rust source file, every markdown
document, the JSON schemas, the scripts, and the two clean picture sets named above.

Third-party **code** the binary links is a separate matter with a separate document; see
`THIRD-PARTY.md` when it lands.
