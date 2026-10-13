---
status: accepted
amends: 0077 (render readings), 0172 (master stage), 0184 (audio routing)
---

# Parametric EQ, dynamics processors, and perceptual scopes provide mixing and inspection

[#981](https://github.com/MBehtemam/Montagent/issues/981), [#1018](https://github.com/MBehtemam/Montagent/issues/1018), [#1021](https://github.com/MBehtemam/Montagent/issues/1021).

## Context

In Premiere Pro, professional post-production relies on two complementary systems:
1. **Audio Mixing & Processing**: The Essential Sound panel and Audio Track Mixer provide parametric equalization, multiband compression, brickwall limiting, and noise gating (`docs/research/premiere-gap/audio.md`).
2. **Diagnostic Scopes**: The Lumetri Scopes and Loudness Radar provide real-time perceptual telemetry (Histograms, Vectorscopes, RGB Parades, Integrated LUFS, and True Peak dBFS meters).

In Montagent, human GUI panels are replaced by declarative project schema and MCP/CLI verbs. To achieve Premiere parity:
- Audio elements and tracks need a closed, deterministic vocabulary for parametric EQ and dynamics processing that compiles reliably to FFmpeg's native audio filtergraph without third-party plugin hosts (VST/AU).
- AI agents need structured perceptual telemetry emitted as JSON via `montagent measure` so they can evaluate visual contrast, color balance, and audio loudness without rendering video files.

## Decision

1. **Closed Parametric Equalizer Vocabulary**:  
   Audio elements and tracks admit `audio_effects` containing equalizer filters:
   ```json
   {
     "name": "equalizer",
     "filter": "peaking",
     "frequency": 2500,
     "gain_db": -3.5,
     "q": 1.4
   }
   ```
   - `filter`: Closed enum (`"lowshelf"`, `"highshelf"`, `"peaking"`, `"bandpass"`).
   - `frequency`: Center or cutoff frequency in integer Hz, bounded to $[20, 20000]$.
   - `gain_db`: Gain adjustment in float decibels, bounded to $[-48.0, 24.0]$.
   - `q`: Quality factor (bandwidth sharpness), bounded to $[0.1, 10.0]$ (defaults to `1.0`).

2. **Closed Audio Dynamics Processors**:  
   Admitted to `audio_effects`:
   - **Compressor (`"name": "compressor"`)**:
     ```json
     {
       "name": "compressor",
       "threshold_db": -18.0,
       "ratio": 4.0,
       "attack_ms": 15,
       "release_ms": 120,
       "makeup_gain_db": 3.0
     }
     ```
   - **Limiter (`"name": "limiter"`)**:
     ```json
     {
       "name": "limiter",
       "ceiling_db": -0.1,
       "release_ms": 50
     }
     ```
   - **Noise Gate (`"name": "gate"`)**:
     ```json
     {
       "name": "gate",
       "threshold_db": -45.0,
       "attack_ms": 5,
       "release_ms": 80
     }
     ```

3. **Deterministic FFmpeg Filtergraph Compilation**:  
   Montagent's audio pipeline compiles these declarative effects directly into FFmpeg filtergraph strings:
   - `equalizer` (peaking) $\rightarrow$ `equalizer=f={freq}:t=q:w={q}:g={gain_db}`
   - `lowshelf` $\rightarrow$ `lowshelf=f={freq}:t=q:w={q}:g={gain_db}`
   - `highshelf` $\rightarrow$ `highshelf=f={freq}:t=q:w={q}:g={gain_db}`
   - `compressor` $\rightarrow$ `acompressor=threshold={th}dB:ratio={r}:attack={att}:release={rel}:makeup={mg}dB`
   - `limiter` $\rightarrow$ `alimiter=limit={ceil}dB:release={rel}`
   - `gate` $\rightarrow$ `agate=threshold={th}dB:attack={att}:release={rel}`

4. **Perceptual Diagnostic Scopes in `montagent measure`**:  
   The `measure` verb is extended to emit structured machine-readable scope data when requested:
   - **Visual Scopes (`visual_scopes`)**:
     - `histogram`: Array of 256 bin counts for `luma`, `red`, `green`, `blue`.
     - `vectorscope`: $64 \times 64$ grid of normalized Cb/Cr spatial density distributions.
   - **Audio Scopes (`audio_scopes`)**:
     - `integrated_lufs`: Float integrated loudness according to ITU-R BS.1770-4.
     - `true_peak_db`: Float peak amplitude in dBFS.
     - `loudness_range_lu`: Loudness range (LRA) across the measured timeline window.
