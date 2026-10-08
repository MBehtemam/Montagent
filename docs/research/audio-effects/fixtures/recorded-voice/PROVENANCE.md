# recorded-voice: provenance

A real recorded human voice for the restoration capabilities (noise reduction and de-ess), which
are inert on clean TTS. Made for [#848](https://github.com/MBehtemam/Montagent/issues/848), which
is a child of the audio-effects map (see its "Not yet specified" section).

About 20 s of an 1888 Edison wax-cylinder recording of an after-dinner speech by the composer Arthur
Sullivan, 48 kHz stereo, lossless. The cylinder is steady hiss under band-limited speech.

| file | what | sha256 |
| --- | --- | --- |
| `recorded-voice.flac` | the first 20 s of the source, 48 kHz stereo | `52566f8bb10213a567c631e4e207710ff3801461d2d85fb1777eadfbe19bb5bc` |
| `build.sh` | the rebuild script | `fdcbfe7b9eab3574709d4951942ba35d4239c6c9f800f45bc3cbc413b6664e0b` |

The source itself is not committed (as with `narration-over-bed/`). Its identity is pinned below.

## Source

- **File:** *Arthur Sullivan - wax cylinder recording.ogg*, Wikimedia Commons —
  <https://commons.wikimedia.org/wiki/File:Arthur_Sullivan_-_wax_cylinder_recording.ogg>
- **Source file:** 94.17 s, mono, 44.1 kHz Vorbis in Ogg, 668,459 bytes, sha1
  `d67606e25f111826e88778a8e11931d21e6958c6` (the sha1 Commons reports, and the one measured after
  download), sha256 `6409cb50832f3c156ad417980e282c68535c7ac41b22e3ada552f3a3a68a6883`.
- **Speaker:** Arthur Sullivan (1842–1900), giving an after-dinner speech at "Little Menlo", London.
- **Recorded:** 1888-10-05, on an Edison yellow paraffin cylinder, by George Gouraud during the
  autumn-1888 phonograph demonstrations.
- **Recording credit:** the Commons "Artist" field names George Gouraud. Its credit points to the
  US National Park Service Edison collection (EDIS), <https://www.nps.gov/edis/learn/photosmultimedia/upload/EDIS-SRP-0155-14.mp3>.
- **Licence:** Commons "Public domain" (`pd`). The recording is from 1888 and its speaker died in
  1900, so it is in the public domain by age. This is not a CC0 dedication, but it grants the same
  permissions. No restrictions are listed on the file page.

## Cut

- **Window:** the first 20 s of the source (0.0–20.0 s), with no gain change. The first second is a
  quiet lead-in; speech runs from about 1 s to the end of the window.
- **Conversion:** resampled 44.1 → 48 kHz, mono duplicated to both channels (`aformat`), written as
  FLAC. The FLAC is 24-bit (the bit depth ffmpeg chose), 1.45 MB.
- **Command:** `VOICE_SRC=<the source .ogg> ./build.sh` (see `build.sh`).

## Measurements

Made with ffmpeg 9.0.2 (Homebrew, arm64), 2026-10-08.

- **Integrated loudness** (`ebur128`): **−19.2 LUFS**, LRA 10.5 LU.
- **True peak** (`ebur128=peak=true`): **−3.1 dBTP**. No clipping in the cut.
- **RMS level** (`astats`, whole file): **−24.4 dBFS**.
- **Noise-floor estimate:** 100 ms RMS frames over the whole cut (199 frames) give p05 **−41.6 dBFS**,
  p10 −40.0, median −30.1, p90 −17.1. The quietest frames are the closest thing to a noise floor
  here; with speech on nearly all of the window they are an estimate, not a measured silence.
- **Band balance** (`astats` RMS):

  | band | RMS |
  | --- | --- |
  | full band | −24.4 dBFS |
  | below 4 kHz | −24.5 dBFS |
  | above 4 kHz (`highpass=f=4000`) | −41.9 dBFS |

  Above 4 kHz sits about 17 dB under the in-band level.

## What this fixture can and cannot prove for restoration

**Can:**

- It is a real voice with a real, steady background hiss at a level well above what a TTS render
  carries. It is the input a noise-reduction filter (`afftdn`/`anlmdn`) can act on, and a
  before/after loudness-matched A/B would show whether the filter changes the noise floor at all.
- It is licensed for reuse and its provenance is pinned to a source file hash.

**Cannot:**

- **Sibilance is weak here.** The recording is band-limited (dull, with little energy above 4 kHz),
  and the high band sits about 17 dB under the speech. A de-esser's threshold will rarely engage on
  this clip, so a null result from de-ess on this fixture says little about de-ess on sibilant
  speech. It is not a sibilance test. Restoration's de-ess check needs a separate, brighter
  recording.
- **The noise-floor number is an estimate**, not a measured silence. There is no speech-free gap of
  length to measure a true noise floor against.
- **It is not a modern microphone recording.** Its noise is cylinder surface hiss and mechanical
  noise with a limited bandwidth. Filter behaviour tuned here may not carry to room tone or mains
  hum in a modern recording.
- **No transcript** was made, so the fixture cannot be checked against a text the way
  `narration.txt` can.
