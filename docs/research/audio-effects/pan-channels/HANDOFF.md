# Hand-off spec: pan/balance and channel operations (ADR-0183, draft)

Written as the bodies of build tickets, sliced like G1-G5 for the gate. Build nothing until the owner has ruled
the calls in [ADR-0183 "Owner calls to confirm"](../../../adr/0183-pan-is-a-linear-balance-and-channels-is-a-five-word-routing-field-both-flat-on-audio-and-video.md)
and moved it out of `proposed`. Both fields are flat, so these slices do **not** depend on **F0** (the
`audio_effects` foundation, defined on #845): `channels` sits after `aloop`, and F0 later inserts the list after
it. The ids T1-T12 below are ADR-0183 section 7's assertions (not the slice ids). Nothing here changes the tool
or verb counts. `channels` and `pan` can ship together or `channels` first (owner call 11): P2's two halves
are written to be separable.

- **P1. Schema.** On `audio` and `video` (ADR-0183 section 1): optional `pan`, a finite number in `-1..1`
  (absent = `0`), and optional `channels`, one of `"stereo"`, `"swap"`, `"mono"`, `"left"`, `"right"` (absent =
  `"stereo"`). Flat: not `audio_effects` members, no `enabled`. A keyframe list on either is a schema error
  (section 5, #842); an unknown `channels` word or a non-number `pan` is a schema error naming the five words /
  the range. `fmt` keeps an explicit default (the reviews in P3 read it) and puts `channels` immediately before
  `audio_effects` and `pan` immediately after `volume`, mirroring the graph order (confirm against the existing
  key order). Test: every word parses, every just-outside `pan` value (`-1.0001`, `1.0001`, `50`) and a non-word
  is refused, a keyframe list is refused.
- **P2. Render.** The `pan=` expressions of ADR-0183 sections 2 and 3, string for string, **with `=` only and
  never `<`** (`<` normalises the gains: `c0=c0+c1` on L = R is +6.02 dB, `c0<c0+c1` is 0 dB; the matrix would
  be silently re-levelled). Balance: `pan=stereo|c0=<gl>*c0|c1=<gr>*c1`, a gain of 1 written without its factor,
  the far gain `1 - |pan|` as the shortest decimal to 6 places (`pan: -0.3` is `pan=stereo|c0=c0|c1=0.7*c1`).
  Channels: `swap` `c0=c1|c1=c0`; `mono` both outputs `0.5*c0+0.5*c1`; `left` `c0=c0|c1=c0`; `right`
  `c0=c1|c1=c1`; **never `aformat=channel_layouts=mono`** (it folds at 0.7071 and can clip, section 3). Slot
  (section 4): `atrim → atempo → aloop → [channels] → [audio_effects] → volume → [pan] → [transition gain]
  → adelay → amix`. Confirm where the engine's `aformat` sits; if it follows `aloop`, add
  `aformat=channel_layouts=stereo` ahead of `channels` only when `channels` is not `stereo`. No `apad...atrim`
  (latency 0, #843). Emit nothing for `pan: 0`, `channels: "stereo"` or either absent: a document with neither
  field renders the graph it rendered before (the committed string, ADR-0173 section 5).
- **P3. `validate`.** The findings in ADR-0183 section 6: `E-PAN-RANGE` (error; repair: the nearest bound),
  `R-PAN-INERT` (`pan: 0`; repair: delete the field), `R-CHANNELS-INERT` (`channels: "stereo"`; repair: delete
  the field), `R-CHANNELS-ON-MONO-SOURCE` (`swap`, `mono`, `left` or `right` on a source the probe reports as
  mono; no repair; needs the probe). `pan` together with `channels: "mono"` produces no finding. The
  silent-kept-channel review and the polarity-inverted mono review are **not** built (section 6).
- **P4. Reading tools.** `query` and `compare` describe `channels` and `pan` on an element (the word, the
  number, the slot); `shift` and the contact sheet need no change beyond not dropping the fields. No tool
  animates either (section 5).
- **P5. Repo tests, three-leg table, accept.** Port
  [`measure_pan_channels.py`](measure_pan_channels.py) to an integration test through the real mix graph with
  the encoder swapped for PCM (ADR-0183 section 7; ADR-0173 sections 1 and 3): T1 balance gains at p in {-1,
  -0.5, 0, 0.5, 1}, T2 exact-zero far channel, T3 balance never mixes (L = 440 Hz, R = 880 Hz), T4 swap / `left` /
  `right` equal the specified source channels, T5-T6 the `mono` fold (L = R, seeded noise, L = -R, left-only),
  T7 `left` on a left-only source, T8 the 0-sample impulse at sample 100 and exact length for `pan` and every
  `channels` word, T9 bypass identity (`pan: 0`, `channels: "stereo"`, absent: same PCM bytes and same graph
  string), T10 the no-feature committed graph string, T11 no `<` in any rendered `pan=`, and T12 (once
  `normalize_loudness` is built, ADR-0178) the slot case within 0.1 LU. Keep the `asendcmd`-ignored control
  and same-build determinism (identical bytes across runs and `-cpuflags 0`). Fixtures are lavfi tones and
  `anoisesrc` with a pinned `seed`, built from literal parameters. Run on all three CI legs, **commit the
  per-leg table (macOS Homebrew and Windows Chocolatey legs were not available to the evidence run)**, and
  replace the provisional 0.01 dB / exact with max(2 x spread, print precision). **Do not flip to `accepted`**
  until the owner's listening test confirms the name of `channels: "left"` / `"right"` as measured (ADR-0183
  section 3: `left` keeps the left channel and copies it onto the right; `right` the mirror) and the owner has
  confirmed the calls; the blind A/B for `pan: -0.5` on the shared narration fixture is recorded in
  `VERDICT.md` (or the owner accepts the numbers alone).
- **P6. Docs.** `format.md`: the two fields and their defaults; `pan` is a **balance** (near channel unity, far
  channel `1 - |pan|`, never moves content between channels, a centred source loses 2.04 dB at 0.5 and
  3.01 dB at 1); `mono` is `0.5 (L + R)` (unity on a dual-mono file, -6.02 dB on a one-channel recording; use
  `left` or `right` for that); `left` / `right` keep that channel and copy it onto the other, **and the
  direction as measured and confirmed**, with no claim of a mapping to Premiere's Fill Left with Right / Fill
  Right with Left (Adobe's text for the first describes this ADR's `left`, opposite to its name); `channels`
  runs before the effects and `pan` after `volume`, so `normalize_loudness` does not see a pan; "to silence
  one side use `pan: -1` or `1`"; no keyframed pan (#842); and the worked example
  `{ "type": "audio", "src": "narration.wav", "channels": "left", "pan": -0.3, "volume": 1 }`.
  `CONTEXT.md`'s **Audio effect** entry notes that routing is flat. The capability-map line moves pan and
  channel operations out of "later".
