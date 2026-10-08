---
status: accepted
amends: 0077 (reading 9: a keyframed `volume`'s commands are now heard on the sample their instant names, on 1 ms frames and in pieces of at most 256 commands; the reading's claim that a per-frame step is "below what the filter could have resolved anyway" no longer holds)
---

# A keyframed `volume` is heard on the sample its instant names

The research for [#797](https://github.com/MBehtemam/Montagent/issues/797)
([`docs/research/audio-effects/FFMPEG-FILTERS.md`](../research/audio-effects/FFMPEG-FILTERS.md)
§4.3, side finding 1) measured that `asendcmd` fires a command on the first frame that
*starts* at or after the command's time, not on the command's sample. `render` sends a
keyframed `volume` as `asendcmd` commands straight after `atrim,asetpts`, so frame sizes are
the decoder's: 1024 samples for AAC, 1152 for MP3. A change was heard up to one source frame
late, which is 21 ms at 48 kHz.

ADR-0077 reading 9 says the fade "must be heard on the clock it is seen on", and that was
not true. `a_keyframed_volume_changes_on_the_sample_its_instant_names` reproduced it: a step at
frame 8 of 30 fps (266 ms, sample 12768) from an AAC source was heard at sample 13312, 544
samples late.

## Decision

### 1. Commands run on 1 ms frames

A keyframed `volume` that moves has `asetnsamples=n=48:p=0` in front of its commands. Every
command time is a whole millisecond (`render::instant_of` floors), so a 1 ms frame starts
under every command, at every fps. `p=0` leaves the last frame unpadded, so the element keeps
its exact length.

Nothing coarser works in general. The instants at 30 fps are 0, 33, 66, 100 ms, so any frame
size that divides them all is 48 samples. `48000 / fps` samples (1600 at 30 fps) is exact only
where the frame grid is whole milliseconds; at 30 fps it is 32 samples late on frame 8. Both
tests reject it.

### 2. At most 256 commands to an `asendcmd`

`asendcmd` checks every command it holds on every frame it passes, so one instance costs
frames × commands. On 1 ms frames a fade across a 6-minute element at 30 fps (10,799 commands)
took **125–147 s** of `ffmpeg` time where today's graph took **8–9 s**. That is the worst case,
since the fade sends a command on every frame.

So past 256 commands the stream is split with `asegment` at the time of each 256th command.
Each piece has its own `asendcmd` and `volume@v<input>_<k>`, starting at the level the piece
before ended on. `asetpts=PTS-STARTPTS` runs after the commands, because `concat` needs each
piece to start at 0. `concat` then joins the pieces. The cost is then linear in the element's
length. 256 was the fastest of 64, 256 and 1024 on that fade.

### 3. Frames go back to 1024 samples after the `volume`

`asetnsamples=n=1024:p=0` follows, so `amix`, `apad`, the trim and the encoder never see the
small frames. Without it, 64-command pieces cost 13.4 s against 7.9 s.

### 4. Nothing else changes

A static `volume`, and a keyframe list whose value never moves, write exactly the graph they
wrote before. The benchmark project (ADR-0142) has only static volumes, so its graph is
unchanged.

## Measured cost

On the dev's M1 Pro, `ffmpeg` alone, the mix chain encoded to AAC 160k:

| Case | Today | 1 ms frames, one `asendcmd` | This ADR |
| --- | --- | --- | --- |
| 6-min element, fade on every 30 fps frame (10,799 commands) | 8.2 s | ≈125 s | 5.4 s |
| 6-min element, 1 s fade in and out (61 commands) | 5.4–5.7 s | — | 5.2–5.9 s |

These runs mixed the element with a second input through `amix`, as `render` does. The check
script runs the element alone and measured the worst case at 9.4 s today, 9.5 s with pieces
and 123.6 s with one `asendcmd`. The machine was loaded (1-minute load 4–11), so these are
paired runs, not ADR-0142 protocol timings. They show the shape. The pieces graph is no
slower than today's on the worst case, and the common case is within noise.

## Evidence

- `crates/montagent-core/tests/render.rs`:
  `a_keyframed_volume_changes_on_the_sample_its_instant_names` (one piece) and
  `a_keyframed_volume_past_one_command_piece_still_changes_on_its_sample` (a step in the
  second piece, at 9266 ms after 270 earlier commands). Each passes within 24 samples of the
  step, read back through the AAC encode. Each fails with 1024-sample frames (+544, +678)
  and with `48000 / fps`-sample frames (+32).
- `crates/montagent-core/src/verbs/render.rs`:
  `keyed_volume_cuts_its_commands_into_pieces_on_one_millisecond_frames` holds the graph's
  shape.
- [`keyed_volume_pieces_check.py`](./keyed_volume_pieces_check.py) re-derives the claims
  about `ffmpeg` on the 6-minute fade, and exits non-zero if any of them stops holding:
  - today's graph is late;
  - 1 ms frames under one `asendcmd` cost over 5× today's graph;
  - the pieces graph matches source × envelope on all 17,280,000 samples (worst 1.9e-8);
  - the pieces graph is no slower than today's.

## Not chosen

- **One `asendcmd` on 1 ms frames.** It is exact, but it is quadratic in the element's length
  (§2).
- **An envelope multiplied in (`amultiply`).** Montagent would write the per-sample gain as a
  second input. That is exact and linear, but it adds a generated file per element: 138 MB of
  stereo `f32` for six minutes. It also puts a second input path into the mix.
- **`afade`.** It is sample-exact, but it uses `ffmpeg`'s own curves. Reading 9 rejects a
  second implementation of the ease vocabulary, and that still stands.

## What no longer holds in ADR-0077

Reading 9 says the per-frame step is "finer than the `volume` filter's own evaluation
interval at `eval=frame`, so the step is below what the filter could have resolved anyway".
That interval was the decoder's frame, and it is what made the command late. The level now
changes on the sample of each frame instant, and it still steps once per frame. The rest of
reading 9 stands: commands, not an expression; one per sampled frame whose value moved;
`instant_of`'s instants.
