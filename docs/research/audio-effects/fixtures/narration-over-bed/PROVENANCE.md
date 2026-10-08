# narration-over-bed: provenance

The shared fixture of [ADR-0173](../../../../adr/0173-every-audio-capability-is-accepted-by-a-measured-repo-test-and-verify-checks-only-what-master-declares.md)
§3, made for [#815](https://github.com/MBehtemam/Montagent/issues/815). About 20 s of TTS narration
over a music bed, 48 kHz stereo, lossless.

| file | what | sha256 |
| --- | --- | --- |
| `narration.txt` | the script that was spoken | `36317372054900285638d2584c7fb5266609b5862b0feb3b1f27dad745576a06` |
| `narration.flac` | the voice alone, 0.8 s lead-in, padded to 20 s | `b179e07336b096af89896d4e5b8b1a377f6c213d87a58f10bdcf18c9ab44f691` |
| `bed.flac` | the music alone, 20 s, fades, −20 dB | `0a4665637bbf779c446d4f161fed40fb6d5384254cf13014118fa315dad99765` |
| `narration-over-bed.flac` | the two summed | `05465e071e773c8f66b6e44b01266818c7be98dc4a597a0383009e7b1622bacb` |

The stems are kept because capability prototypes need them as separate elements (the ducking
tool reads the voice and writes the bed's `volume`).

Measured with `ebur128=peak=true`: narration −18.1 LUFS, peak −2.9 dBFS; bed −33.0 LUFS (15 LU
under the voice); mix −18.0 LUFS, peak −2.8 dBFS.

## Narration

- **Engine:** [Piper](https://github.com/OHF-Voice/piper1-gpl) as `piper-tts` 1.8.0 from PyPI
  (GPL-3.0; only its *output* is used, and the engine is not part of this repository).
- **Voice:** `en_US-ljspeech-medium`, from `rhasspy/piper-voices` at revision
  `c10ece1aade47bb51c153c893d14e5bf8e5b7117`. The repository's card says `license: mit`.
  - `.onnx` sha256 `6f52a751e2349abe7a76735eb09dc1875298c77ea2342ffd2fef79ff81b87f22`
  - `.onnx.json` sha256 `141d612cc0a95ed7efc1ca936b845c2364967f2e9217c5dbfcf69fc4d6c65860`
- **The voice's own terms**, from its `MODEL_CARD`, verbatim:

  > Dataset: URL: https://keithito.com/LJ-Speech-Dataset/ · License: public domain
  > … US English female voice. Single speaker. Trained from scratch for 1000 epochs on medium
  > quality settings using the LJ Speech dataset.

  No vendor terms beyond that were found. Generated speech is a derivative of a public-domain
  corpus through an MIT-licensed model.
- **Command as run** (macOS arm64, 2026-10-08):

  ```
  piper -m en_US-ljspeech-medium.onnx -i narration.txt -f narr.wav
  ```

  The PyPI wheel's bundled espeak data was not found where the engine looked (it printed
  `…/site-packages//phontab: No such file`), so each file in `piper/espeak-ng-data/` was
  symlinked into `site-packages/`. That is a workaround for the install, not a change to the
  output.
- **Not reproducible bit for bit.** Two runs of the same command gave different WAVs
  (sha256 `6664f09c…` and `c1f3462e…`): the voice is a VITS model with sampled noise. The
  committed `narration.flac` pins the file; `build.sh` does not regenerate it.

## Bed

- **Source:** *Ambient music test, Yamaha CK61.flac* by Wilfredor, own work, 2024-02-27 —
  <https://commons.wikimedia.org/wiki/File:Ambient_music_test,_Yamaha_CK61.flac>
- **Licence:** CC0 1.0 (Commons shows "Creative Commons Zero, Public Domain Dedication",
  <https://creativecommons.org/publicdomain/zero/1.0/>).
- **Source file:** 197.75 s, 48 kHz stereo FLAC, sha1 `fbe9e5fbe4b727fac1608dec68c94a956d9cbd65`
  (the sha1 Commons reports, and the one measured after download).
- **Cut:** 20 s starting 60 s in, −20 dB, 1.5 s fade in, 2 s fade out.

## Rebuild

`build.sh` turns a Piper WAV and the Commons FLAC into the three FLAC files, with ffmpeg
9.0.2 (Homebrew, arm64). FLAC is lossless, but the *decoded* samples can differ between
ffmpeg builds only through resampling (`aresample` 22.05 → 48 kHz), so the hashes above are
for the committed files, not a promise about a rebuild.
