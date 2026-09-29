# Skills eval

Evidence for the eval designed in
[Eval design: measuring a Montagent ad built with and without skills](https://github.com/MBehtemam/Montagent/issues/446).
It asks whether Opus 5.5 using Montagent with the end-user skills makes motion graphics
comparable to what it makes without Montagent, and better than Montagent without the skills.

| Path | What it holds |
|---|---|
| `briefs/dev/` | The three open development briefs: A (Montagent launch spot), B (talking-head social cut), E (logo reveal loop). The baseline runs these, and the skills may be written with them in view. |
| `assets/` | The fixed asset pack. Every run starts from a copy of this directory and nothing else. Its `README.md` is the manifest, and is part of the pack. |
| `pack-src/` | The scripts that made the pack's generated files (brand, music bed, voiceover). They are not part of the pack. |
| `RECORDING.md` | The maintainer's checklist and scripts for the footage only they can record. |

The held-out briefs are not here. Only their SHA-256 hashes will be, once they are written
([Held-out eval briefs: written blind and sealed by hash](https://github.com/MBehtemam/Montagent/issues/466)).
