# Question: the capability ADRs for loudness normalisation, EQ, and the compressor and limiter

Tickets [#844](https://github.com/MBehtemam/Montagent/issues/844),
[#845](https://github.com/MBehtemam/Montagent/issues/845) and
[#846](https://github.com/MBehtemam/Montagent/issues/846) on the audio map
([#795](https://github.com/MBehtemam/Montagent/issues/795)). Sent blind and identical to three jurors
dispatched in parallel (Opus, Sonnet, Fable), with no tools and no recommendation in the packet. One ballot
per sub-question. The jurors were told they could reject a framing.

## Facts given to every juror

Montagent: a Rust video-editing tool; a project is a closed JSON document; elements are a flat list on
tracks; times are absolute integer ms; render is an ffmpeg 7.1 filter graph; agents author the JSON through
MCP tools and a `validate` command.

**Already ruled (not reopened).** Each element may carry an ordered `audio_effects` list, rendered before
`volume`: chain `atrim → atempo → aloop → [audio_effects] → volume → [pan] → [transition gain] → adelay →
amix normalize=0`. Units: dB carried in the key name (`_db`), `_hz`, `_ms`, `ratio`, `q`, `semitones`,
`mix`; volume stays a linear multiplier; pan −1..+1. Every parameter in the first wave is a static literal
(a keyframe list on one is a schema error). The schema is a closed vocabulary with literal values, checkable
by `validate`, editable by exact-string replace. validate findings are error/review/note; only errors carry
a repair. The master stage (top-level `master {target_lufs, ceiling_dbtp}`) applies one measured gain, never
loudnorm; the limiter runs at 4× rate with the ceiling within 1 dB of the delivered file. Every audio
capability is accepted by a measured repo test; tolerance = max(2 × three-leg spread, meter resolution).
Latency is a render semantic: the renderer cancels fixed filter latency, filters enter only with
literal-parameter latency and 0-sample onset; effect tails are truncated at element end. Precedent:
Premiere/CapCut first; no precedent needs an owner-accepted prototype.

**#844 loudness normalisation (accepted shape).** Singular member `normalize_loudness` with required
`target_lufs` (no default, no per-type presets); measures the element's own placed window (after trim,
speed, loop), applies one fixed gain, never loudnorm; undefined loudness (silence, too short) → no gain and a
finding. The prototype used voice −23 LUFS and a music bed −38 LUFS (the bed sits 15 LU under the voice);
the owner preferred the normalised mix. *Open:* member name confirmation, `target_lufs` range, the bed-target
convention (authored value vs a documented convention), which validate checks and findings (error / review /
note) exist, and the measured repo check.

**#845 EQ (accepted shape).** Several small stackable non-singular members using ffmpeg biquads only (zero
latency, identical across builds): `highpass` / `lowpass` (`frequency_hz`, slope in 12|24|48 dB/oct),
`shelf` (`side`, `frequency_hz`, `gain_db`), `bell` (`frequency_hz`, `gain_db`, `q`). Measured: −3.01 dB at
the cutoff; the bell hits its gain. A gentle high-pass on clean TTS was inaudible, so the measured check must
use lavfi tones; an exaggerated EQ was told apart blind. *Open:* exact member names, the slope key name and
set, numeric ranges for each parameter, validate checks (frequency above Nyquist, zero-gain no-op, extreme
gain, a stack-count cap), and the measured check.

**#846 dynamics (accepted shape).** Two non-singular members. `compressor` (`threshold_db`, `ratio`,
`attack_ms`, `release_ms`, `makeup_db`) via `acompressor` with RMS detection (peak detection read about 3 dB
off, so the threshold reads against RMS). `limiter` (`ceiling_db` sample peak, `release_ms`) via
`alimiter level=0 latency=1`, ceiling held to 0.00001 dB, lookahead fixed at 5 ms, no onset shift. The owner
heard the limited clip as steadier; the compressor was not told apart on already-even TTS, so its check must
be a static transfer curve plus a burst signal, not TTS. *Open:* names, numeric ranges (and whether attack
and release have defaults), validate checks (ratio 1 no-op, makeup that clips, compressor after limiter,
ceiling above the master ceiling), and the measured check.

## Questions

**Q1 (#844).** What exactly does the capability ADR for `normalize_loudness` fix: the `target_lufs` range,
how the bed target is expressed, the validate finding list with severities, and the measured check?

**Q2 (#845).** What exactly does the capability ADR for EQ fix: member and key names, slope set, parameter
ranges, validate finding list with severities, and the measured check?

**Q3 (#846).** What exactly does the capability ADR for the compressor and limiter fix: key names, parameter
ranges and defaults (if any), validate finding list with severities, and the measured check?

Answer each concisely but concretely (actual numbers and names).

## Ballot format required of every juror

```
🗳️ **Juror <n>** (<the model backing them>) — **VOTE: <a concrete answer in one line>**

**Reasoning:** <why>
**Trade-offs:** <what it costs, and what was rejected>
```
