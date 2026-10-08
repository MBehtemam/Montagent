# Hand-off spec: echo (ADR-0184, draft)

Written as the bodies of build tickets, sliced like C1-C5 on #846 and G1-G5 for the gate. Build nothing until the
owner has ruled the calls in [ADR-0184 "Owner calls to confirm"](../../../adr/0184-echo-is-one-stackable-audio-effect-with-delay-feedback-and-mix-and-its-tail-is-cut-at-end.md)
and moved it out of `proposed`. Every slice depends on **F0** (the `audio_effects` foundation, defined on #845):
the field, its tagged union, `enabled: false`, the singular error, the render hook and the float format. Nothing
here changes the tool or verb counts. **Reverb is not in these slices** (ADR-0184, "Not decided here"); the
prototype's reverb code and numbers are not a starting point for any slice below.

- **EC1. Schema.** The `echo` member with three required keys, no defaults, the ranges of ADR-0184 section 1
  (`delay_ms` 1..2000 and may be fractional, `feedback` 0..0.95, `mix` 0..1); **non-singular, no stack cap**
  (section 1); a keyframe list on any key is a schema error (section 6, #842); `enabled: false` honoured.
  Missing or unknown key: the clean error naming the keys. Test: every bound parses, every just-outside value is
  refused, a document with two `echo` members is valid.
- **EC2. Render.** The graph of ADR-0184 section 2, string for string:
  `aecho=in_gain=1:out_gain=1:delays=...:decays=...` followed by the cut to the element's placed duration.
  - Taps: k = 1..8, `d_k = k x delay_ms`, `g_k = mix x feedback^(k-1)`, stopping at the first `g_k` under 0.001.
    **`in_gain` and `out_gain` are written, both 1** (aecho's defaults are 0.6 and 0.3).
  - **The end cut is mandatory** (`atrim=end=<duration>` after the filter): `aecho` extends its stream by its
    longest delay, so the length must not rest on its flush (section 2; #843).
  - **No `apad ... atrim` latency cancellation** (the latency is 0 samples, #843 and section 2); a bypassed
    member, a `mix` 0 member or one with no surviving tap emits no filter at all.
  - Placed in the author's order before `volume`, after `aloop` and `atempo`, in float (ADR-0179 section 2);
    `delay_ms` is not scaled by `speed`. A document with no echo renders the graph it rendered before
    (committed string, ADR-0173 section 5).
- **EC3. `validate`.** The findings of ADR-0184 section 4: `E-ECHO-RANGE` (error, repair: the nearest bound),
  `R-ECHO-HOT` (review: product over the element's enabled `echo` members of `1 + mix x sum of feedback^(k-1)`
  over the section 2 taps, above 3), `N-ECHO-NO-OP` (note: no tap survives), `N-ECHO-TAPS-CAPPED` (note:
  `mix x feedback^7 > 0.01`). Counts consider enabled members only. **No `R-AUDIO-TAIL-CUT`** unless the owner
  rules call 1 the other way; if it rules the note form, `N-ECHO-TAIL-CUT` lands here with the effect named in
  ADR-0146's form (`audio_effects[1].delay_ms (echo)`) and the cut length computed from literals.
- **EC4. Repo tests, three-leg table, accept.** Port the prototype's impulse and smoke cases into an
  integration test through the real mix graph with the encoder swapped for PCM (ADR-0184 section 7; ADR-0173
  sections 1 and 3), not by importing `prototype_echo_reverb.py`:
  1. **The 0-sample onset assertion on a lavfi impulse (#843: 0 samples or stay out).** A stereo lavfi impulse
     at sample 24 000 through `{echo, 300, 0.5, 0.5}` and `mix` 1.0: first non-zero sample and peak both at
     24 000, **exact**, nothing before it asserted at exactly 0; the length exact.
  2. **The three-leg measured test.** The impulse-tap case: for `(delay_ms, feedback, mix)` = (300, 0.5, 0.5),
     (300, 0.5, 1), (100, 0.9, 0.5), (1, 0.95, 1), (2000, 0.95, 1), (300.4, 0.5, 0.5), (250, 0, 1), tap k lands
     at `floor(k x delay_ms x 48)` samples after the dry impulse, **exact**, with value `mix x feedback^(k-1)` x the
     dry impulse's (measure against the member-disabled render in the same run, never a stored render);
     no other non-zero sample; at most 8 taps. Provisional amplitude tolerance 2e-6 absolute. Run on **all three
     CI legs** (Linux at the 7.1 floor, macOS on Homebrew, Windows on Chocolatey), commit the per-leg table, and
     replace the provisional tolerance with max(2 x spread, resolution). Position stays exact.
  3. **Length and tail.** Exact length at `delay_ms` 2000 and on an element ending exactly at its source's end;
     the cut render equals the longer-`end` render's prefix.
  4. **Smoke on the shared narration** (ADR-0173 section 5): 48 kHz stereo, exact length, no NaN/Inf, no clip,
     above -70 LUFS, difference from bypass above -30 dB re bypass RMS, loudness within 3 LU of bypass.
  5. Range edges (every bound renders), bypass identity (`enabled: false` PCM equals member-absent PCM, and
     `mix` 0 PCM equals it), the no-member graph string, same-build determinism (`-cpuflags 0`), and the cases
     the evidence lacks: a looped element (repeats continuous across the seam), `speed` other than 1, a `start`
     later than 0, two stacked echoes, a mono source.
  6. Do not flip the ADR to `accepted` until the table is committed, EC5 is resolved and the owner has ruled
     the calls (the A/B is already in `VERDICT.md`).
- **EC5. Floor-build risk: the 6.1.1 dry-branch loss, resolved on 7.1.** README "Builds": on 6.1.1 the dry
  branch of an `asplit -> amix` loses its last 4 656 samples (97 ms) when its source ends (max difference
  0.13, -31 dB re signal), which broke `mix` 0 = bypass and cut = prefix for reverb. Echo has no such branch
  and passed its own claims on 6.1.1, but the 7.1 floor was never run. Run the EC4 length and
  flush-with-the-source cases on the 7.1 floor leg first, and record whether the defect is present on 7.1 or
  absent. If the owner rules ADR-0184 call 2 (fade the dry), the dry branch becomes part of the graph and this
  slice becomes a blocker for it; otherwise it is a confirmation. Output: a line in the per-leg table, and the
  ADR's section 7 box ticked or the member held back.
- **EC6. Reading tools and docs.** `query` and `compare` describe the member; `format.md`: the section 2
  formula, "the repeats are a series of at most eight, so a high `feedback` makes a louder last repeat, not a
  longer tail", "the repeats stop at the element's `end` and the echo of the last word is lost when the
  element ends on it: extend the source range or end the source with silence" (this sentence is the whole
  tail-cut warning if call 1 is taken as recommended), the `mix` meaning (dry at unity), the sample truncation
  of `delay_ms`, and the worked example `{ "name": "echo", "delay_ms": 300, "feedback": 0.5, "mix": 0.5 }`.
  `CONTEXT.md`'s **Audio effect** entry names the member.

## Not in these slices

Reverb; a `tail_ms` key (a named departure, prototyped and not admitted; reopened only by the owner hearing a
cut on a harsher fixture); a multitap or analog delay; ping-pong or a filter on the repeats; animating any key.

## Owner calls to confirm

The ten calls in [ADR-0184](../../../adr/0184-echo-is-one-stackable-audio-effect-with-delay-feedback-and-mix-and-its-tail-is-cut-at-end.md)
stand; the ones that change a slice:

1. **No `R-AUDIO-TAIL-CUT`** (EC3, EC6): none (recommended), the `N-ECHO-TAIL-CUT` note, or the review #843 expected.
2. **`mix` leaves the dry at unity** (EC2, EC5): fading the dry adds a branch and makes EC5 a blocker.
3. **Tail cut, no `tail_ms`** (EC2): on one narration and one listener.
4. **Stackable, no cap** (EC1).
5. **Names `echo` and `feedback`** (EC1, EC6).
6. **Ranges** (EC1), and **`R-ECHO-HOT` at 3, `N-ECHO-TAPS-CAPPED` at -40 dB** (EC3).
7. **No defaults** (EC1), the standing minority call of ADR-0180.
8. **Reverb is out** of this ADR and these slices.
