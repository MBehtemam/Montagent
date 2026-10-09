---
status: accepted
amends: 0143 (the claim that the same project, settings and ffmpeg build give the same MP4 bytes holds on the tested targets other than x86_64 Windows; there the guarantee is the frames at the encoder's input, not the file's bytes)
---

# x64 Windows guarantees the frames at the encoder's input, not the MP4's bytes

The owner asked for a softer test on this leg, after being told what the flake costs a user.

[ADR-0143](0143-render-keeps-libx264-medium-crf-20-pins-five-encoder-threads-and-says-so.md)
pins libx264 to five threads so that *"the same project, settings and ffmpeg build now give
the same bytes on any machine"*. CI checks it by rendering one project several ways (one
painter, several painters, other chunk sizes) and comparing the MP4s byte for byte, beside the
hash of every frame handed to the encoder.

## What was found

On the x86_64 Windows leg, and only there, `painters` and `letter_spacing` failed in four of
five runs. The hash of every frame at the encoder's input was identical in all of them. The
files were not: they differed by about 0.5 % in size (18,980 against 19,111 bytes), and the
decoded frames of the two files differed too, because H.264 at CRF 20 is lossy and the two
files are two slightly different encodings of the same input.

A diagnostic run (#869, 24 to 48 renders per variant of one project on the same runner) tried:
one filter thread, bit-exact scaler flags, x264 single-threaded and deterministic, passthrough
frame rate, all four together, and one encoder thread instead of five. Every one still wrote
more than one distinct file. The only variant that wrote one file in all 72 runs was
`-cpuflags 0` with `-x264-params asm=0`, which turns off SIMD. So the cause is not the thread
count, the pinned arguments or Montagent: it is x264's assembly path in the x64 Windows ffmpeg 9
build CI pins (ADR-0187). Which of the two switches matters was not separated.

## Decision

- **On x86_64 Windows the guarantee is the frames at the encoder's input.** The tests still
  assert that every frame handed to the encoder is identical across painter counts, chunk
  sizes and hints. The comparison of the MP4's bytes, and of its decoded frames, is not made
  on that leg.
- **On every other target the claim is unchanged.** aarch64 and x86_64 Linux, aarch64 macOS and
  aarch64 Windows still compare the bytes. Intel macOS (ADR-0188) is built, not tested.
- **The encoder arguments are not changed.** ADR-0143's `-threads 5` stands. x264's assembly
  stays on, so no Windows render gets slower.

## Why this is the right size

The video a user gets is not worse: same input frames, same settings, a file that differs in
the last half percent of its size. What is lost is that the same project may write a different
file on x64 Windows from one run to the next, which matters only to someone who caches or
compares renders by hash. Turning off SIMD would restore it at the cost of every x64 Windows
render. That trade is not taken here.

## What would change this

A cause found and fixed upstream, a different pinned ffmpeg build on that leg that writes one
file, or a user who needs byte-stable output on Windows x64, would reopen the choice between
turning x264's assembly off there and pinning another build.
