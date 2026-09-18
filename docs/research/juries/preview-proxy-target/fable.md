**Juror verdict — #158 (proxy target, ladder, floor) — Fable 5.1**

**Q1. Target and ladder — ACCEPTED WITH MODIFICATION**

The 720p target survives attack. On the load-bearing backend (skia-safe, per ADR-0009/#34) 720p clears at both sizes with real margin: 2.68 s (46% under) at 4K, 3.78 s (24% under) at 8K. 1080p is correctly rejected: 5.79 s at 8K and 7.46/7.58 s on tiny-skia at both sizes. A size-dependent target (1080p for ≤4K) would be better quality at 4K but contradicts ADR-0021's "fixed target" shape and rests on a single-run 14% margin that tiny-skia already fails; fixed 720p is the defensible choice.

Two modifications:

1. **Record that tiny-skia is *not* certified at 8K/720p.** 4.91 s, 1.8% margin, single run, no variance — FINDINGS says outright "too thin to certify as safe." The ADR must state that the `&lt;5s` guarantee at 720p is a skia-safe guarantee; tiny-skia is not covered.
2. **State what one degrade step can actually rescue.** 720p→540p buys 1.05 s at 4K and only 0.71 s at 8K (3.78→3.07), because decode at 8K/540p is 1.20 s and cannot shrink with the target. So a 720p miss of more than ~0.7 s at 8K cannot be recovered by the ladder at all — it lands on the hard-fail floor. That is fine (the floor exists precisely for this), but the ADR should say so rather than imply degradation is a general safety net. Dropping 360p is right: 540p→360p buys 0.47 s / 0.35 s.

**Q2. Legibility pass — ACCEPTED WITH MODIFICATION**

The brief contains an arithmetic error that must not reach the ADR: it says 540p text is "roughly double 360p's, since 540p is 1.5x the linear scale." It is 1.5×, not double. From FINDINGS' own figures (57–80 px at 1080p-equivalent → 19–27 px at 360p), 540p yields ~28–40 px and 720p ~38–53 px. Both are unmeasured for legibility; "not the configuration FINDINGS flagged" is absence of evidence.

Shipping without a pass is still acceptable, but on a different argument than the brief's: ADR-0021 assigns pixel-accurate checks (text legibility, edge alignment) to `frame`, which always renders at true resolution, and gives `preview` a full-resolution escape hatch. Degraded preview legibility is therefore lower-stakes by design. The ADR must (a) place the 540p floor explicitly as a *wall-clock* floor with the visual floor unmeasured — not assert that 540p "is legible"; (b) cite the 28–40 px figure so the claim is falsifiable; (c) keep the brief's rule that any tier below 540p requires a legibility pass first. Asserting 540p legibility as fact would be exactly the ADR-0005 pattern.

**Q3. Heavy composite stack gap — ACCEPTED AS PROPOSED**

Recording it as a non-blocking limitation, GPU-style, is correct. Note the scope precisely: the ablations show that at small targets fixed per-frame overhead plus decode dominate, so a heavy stack's extra raster cost scales with output pixels — 540p is 0.56× 720p's pixels — and would degrade somewhat better than the decode-bound single-clip case. But the harness measured neither, and the one-step ladder's rescue capacity at 8K (~0.7 s) is small. A follow-up should be opened only when a real heavy stack is shown to miss at 720p; until then, the "one clip, one still, one overlay" scope of the measured numbers must be stated in the ADR.

**Overall:** the proposed numbers stand (720p default, 540p single degrade tier, hard-fail below). The write-up needs the tiny-skia carve-out, the corrected 1.5× text arithmetic, and the floor framed as time-only.
