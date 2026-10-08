You are a juror on a court. Answer the Question below and nothing else: do not use any tools, do not edit anything, do not make recommendations to whoever convened you. Judge it from the standpoint of how AI agents actually work with Montagent: they author and edit the JSON document by hand (often by exact-string replace), run `validate`, `render`, partial renders and `preview`, read `verify` and `review` findings, and iterate.

## Context (facts and prior rulings, not opinions)

Montagent is a video document format plus renderer. An AI agent writes a JSON project file; the renderer turns it into video with ffmpeg. Standing invariants (ADR-0145): literal values, closed vocabulary, checkable by `validate` from the file and probed assets alone without painting a frame, editable by exact-string replace. Findings come in two kinds: `validate` errors (the document is wrong; render refuses) and `review` findings (flagged for judgement, e.g. where a threshold is borrowed rather than certain; ADR-0061). `verify` inspects the rendered deliverable after `render`.

Audio: elements carry `volume` (linear, >1 allowed) and an `audio_effects` list. Every input is mixed at 48 kHz stereo with `amix normalize=0`, and the deliverable's audio is lossy AAC 160k. New levels are in dB / LUFS / dBTP.

Already ruled for the master stage (do not relitigate):
- An optional top-level `master: { target_lufs?, ceiling_dbtp? }`, both keys independently optional. With no `master`, the mix renders exactly as today (no normalisation, no limiter).
- `target_lufs` is a target the renderer reaches: pass 1 renders the whole programme's mix (audio only) and measures integrated loudness (EBU R128 / BS.1770, gated; blocks below −70 LUFS are gated out). Pass 2 applies one fixed `volume=<x>dB`, then `alimiter` (latency-compensated) at `ceiling_dbtp` when set. Never `loudnorm`. Partial renders and `preview` take the gain from the whole-programme measurement, so a span sounds the same as in the final render.
- `render` and `verify` report the measured pre-gain loudness and the gain applied.
- The tolerance for "hit the target" is set by a separate ticket on measured checks, not here.

Precedent: Premiere's export loudness normalisation (BS.1770 with editable target; R128 fixed −23; A/85 −24) applies with an optional true-peak limiter. Broadcast targets are −23/−24 LUFS, streaming platforms roughly −14/−16 LUFS. Streaming and R128 guidance recommend a true-peak ceiling of −1 dBTP or lower for lossy delivery, because lossy encoding adds inter-sample peaks.

## The Question (five parts; answer each)

**Q5 — `target_lufs` set without `ceiling_dbtp`.** A positive gain without a limiter can clip.
(a) `validate` error: a target requires a ceiling.
(b) `review` finding: allowed, flagged.
(c) Allowed silently.
(d) An implicit ceiling turns on whenever a target is set.

**Q6 — Does the renderer compensate the loudness the limiter removes?** The gain is computed before the limiter; heavy limiting can leave the delivered loudness below target (e.g. −14.6 vs −14).
(a) No: one gain, stop; `verify` reports the miss.
(b) Yes: re-measure after the limiter and correct once (a third audio pass).
(c) Iterate until within tolerance.

**Q7 — What does `verify` read and report about loudness, and how severe is a miss?**
(a) Only when `master` is present: integrated LUFS vs `target_lufs`, true peak vs `ceiling_dbtp`.
(b) Always reports the deliverable's integrated LUFS and true peak as measurements; findings only against values `master` sets.
And separately: is a miss against the target/ceiling a `review` finding or an error?

**Q8 — A target on a silent or near-silent mix.** If every block is gated out, integrated loudness is undefined; a mix at −50 LUFS would need +36 dB to reach −14 (raising the noise floor).
(a) Undefined loudness → apply no gain, `review` finding; any defined loudness gets its full gain however large.
(b) As (a), but cap the applied gain (e.g. +20 dB).
(c) Something else (say what).

**Q9 — What ranges does `validate` allow for `target_lufs` and `ceiling_dbtp`, and should any in-range value still raise a `review` finding?** Open-ended: propose the ranges and any review thresholds.

You may reject any framing if you think the options are wrong.

## Answer format — exactly this block, nothing else

🗳️ **Juror <n>** (<the model backing you>) — **VOTE: Q5 <choice>; Q6 <choice>; Q7 <choice, severity>; Q8 <choice>; Q9 <ranges / thresholds>**

**Reasoning:** <why, per part>
**Trade-offs:** <what it costs, and why not the other options, per part>
