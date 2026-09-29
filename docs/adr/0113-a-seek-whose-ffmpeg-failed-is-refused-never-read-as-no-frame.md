---
status: accepted
amends: 0096 (§5's fallback is taken only after a `select` run that *succeeded* and wrote no frame; a run that exited non-zero is a refusal carrying `ffmpeg`'s own words, and so is a failed `frames_from` run, which §Scope left outside this ADR's reach)
---

# A seek whose `ffmpeg` failed is refused, never read as "no frame"

> **Amended by [ADR-0115](./0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md)**: this ADR's rule — a failed spawn is never an empty answer —
> is an **invariant over every spawn**, held by `tests/spawn_exit_status.rs`'s audit of each
> one. The seek now sends `-fps_mode passthrough`, and the open note below is answered:
> once the tool has qualified, `E-NOT-PAINTED-UNDECODABLE` blaming the source is correct, and an
> `ffmpeg` that cannot seek is `E-TOOL-UNSUPPORTED` before any seek runs.

[#478](https://github.com/MBehtemam/Montagent/issues/478), found by
[#471](https://github.com/MBehtemam/Montagent/issues/471) while bringing
[#424](https://github.com/MBehtemam/Montagent/pull/424) up to date on a machine with
`ffmpeg 9.0.2`.

## What was there

ADR-0096 §1 asks `ffmpeg` for every frame starting at or before the instant, and §5 falls back
to the window's **earliest** frame where that run returns none. `frame_at` decided *"returned
none"* by the length of stdout, and **never read the exit status**. `over_window` returned `Err`
only when the process could not be spawned at all.

ffmpeg 9 removed `-vsync`, which §1's run passes. On 9 that run exits 8 having written nothing,
so every instant took the fallback and painted the earliest frame in `[at − 200 ms, at]`:

| asked for | frame it covers | painted on 9.0.2 |
| --- | --- | --- |
| 1.234 s | 30 | **26** |
| 1.400 s | 35 | **30** |
| 3.276 s | 81 | **77** |
| 0.030 s | 0 | 0 (the window is clipped at zero) |

**Up to five frames early at 25 fps, on the grid or off it, with `0 errors`.** That is
MONTAGENT-2's silent defect back, and wider than the original: ADR-0096 §Scope names
`tests/seek_clamp.rs` as the only thing holding the behaviour, and it was the only thing that
noticed. The measurements are #471's, on `research/ffmpeg-9-support`.

The removed option is the trigger. The defect is that **an empty answer and a failed one were the
same value.** Any `ffmpeg` failure takes that path: an argument some build rejects, a codec a
build lacks, or a source the decoder chokes on. So this ADR does not depend on which `ffmpeg`
Montagent supports. That is [#477](https://github.com/MBehtemam/Montagent/issues/477)'s
question.

## The decision

### 1. A non-zero exit from either run is a refusal

Both of `frame_at`'s spawns, the §1 `select` run and the §5 fallback, have their exit status
checked before their output is read. A run that failed is an `Err`, one sentence naming:
- the source;
- the resolved `ffmpeg` path, which is the thing to go and look at;
- the exit status;
- what `ffmpeg` wrote to stderr.

This is ADR-0096's own rule for its error sentence, for ADR-0011's reason: exit 70 says
*"retry or report"*, and neither is possible from *"ffmpeg failed"*.

**§5's fallback is reached only by a run that succeeded and wrote no frame.** That is the one
outcome that means *"no frame at or before"*, and it is the only case §5 was ever written for.

### 2. `frames_from` gets the same rule

`frames_from` read an end of stdout as the end of the run, and it sent `ffmpeg`'s stderr to
null. `measure`'s keyed-alpha coverage series (ADR-0088) reads *"the run ended"* as *"the
source ran out"* and reports a shorter series. So a failed `ffmpeg` produced **a series of no
frames about a source that has them.** That is the same silent class, and the same fix:
- Stderr is drained on its own thread, so `ffmpeg` never blocks on a full pipe mid-run.
- An end of stdout, whether clean or mid-frame, waits for the process.
- A non-zero exit is an error carrying what `ffmpeg` said.

A caller that stops reading early still kills the child, as before. Its exit status is not
consulted, because the caller asked for no more.

### 3. What `render` and `frame` do with it is unchanged

A `frame_at` error already reaches `frame.rs`'s `undecodable` arm, which raises
`E-NOT-PAINTED-UNDECODABLE`. ADR-0093 makes that an `error` from `render`, which withholds the
deliverable, and a `review` from `frame`, which names the element missing from the picture. So on
ffmpeg 9 today:
- a project with a video element now **refuses to render, and names `-vsync`**, where it used to
  publish a picture that was up to 200 ms early;
- `frame` paints without the element and says why, where it used to paint the wrong frame of it.

That finding blames the *source*, and here the fault is the *tool*. The two cannot be told
apart from an exit code: `ffmpeg` exits non-zero for a corrupt input as readily as for an
unknown option. **A named finding for an unsupported `ffmpeg`** is left to
[#477](https://github.com/MBehtemam/Montagent/issues/477). It will be a check that runs once
up front, not a reading of a failed seek. This ADR makes the failure loud. It does not claim
to name it correctly.

## Scope

**It does not move to ffmpeg 9's argument forms.** `-vsync 0` and `-filter_complex_script` stay
until [#479](https://github.com/MBehtemam/Montagent/issues/479) ships what #477 rules. On 9, the
seek-clamp tests now fail with this ADR's refusal rather than a wrong frame index, which is the
point: a failing test and a failing render now say the same true thing.

**The other spawns were audited and already comply:**
- `encode.rs` waits and checks the encoder's status before publishing (ADR-0093, ADR-0109).
- `probe`'s `Runner` records `success` and the exit code, and an `ffprobe` that rejects its
  flags is exit 70 (`adapters.rs`, `cli_probe_with_an_ffprobe_too_old_for_our_flags_…`).

## Evidence

`crates/montagent-core/tests/seek_clamp.rs` holds this with a stub `ffmpeg` that refuses any
run carrying a given argument and hands every other run to the real one. That is the shape
ffmpeg 9 had: the `select` run refused, the fallback allowed. A stub rather than a particular
build, because the claim is about a failed spawn, and every machine has to be able to make
one.
- `a_seek_whose_ffmpeg_failed_is_refused_and_never_painted_from_the_fallback`: before this
  ADR, `Ok(26)` at 1234 ms; now an `Err` naming the source and `ffmpeg`'s message.
- `a_run_whose_ffmpeg_failed_is_an_error_and_not_an_empty_series`: before this ADR,
  `Ok(None)` on the first read; now an `Err`.

Both were run red against the code before this change and green after it.
