# Fixture: `en-halloween-decorating`

The assets of one already-published short, collected so that every prototype and
design ticket on [the map](https://github.com/MBehtemam/Montaget/issues/2) can be
judged against real input instead of plausible-looking invented input.

Resolves [Gather a real sample video as the project's test fixture](https://github.com/MBehtemam/Montaget/issues/3).

**Source:** `~/projects/github/youtube_language_learning/episodes/halloween`,
short 2 of 3 (`ids: 05,06,07,08`), English edition. Copied verbatim — no
re-encoding, no downscaling.

> The map's ticket named the *pumpkin* short (items 01–04). Its final MP4 does not
> exist on disk: `publish.yaml` declares `out/shorts/{da,de,es,fr,pt,en}-halloween-pumpkin.mp4`
> for all six languages, but `out/shorts/` holds only the *decorating* and *costume*
> finals. Without a final render there is nothing to pin down what correct output looks
> like, so the decorating short stands in.

## Nothing here is a design

The reference material shows **what the output must contain**, not how to express it.
The source project's JSON schema, its `.ass` subtitle files and its Python/FFmpeg
driver are the problem Montaget exists to replace — read them as evidence of the
required result, never as a format to carry over.

## Layout

```
en-halloween-decorating.montaget.json           — the project file that composes all of it
images/         05.png 06.png 07.png 08.png     — the source stills, one per item
audio/          11 mp3 files                    — see below
brand/          logo-en.png                     — the channel badge in the header
transcript.json                                 — the text, and how it maps to the media
reference/
  en-halloween-decorating.mp4                   — the published output, the target
  beats.json                                    — the timeline the old pipeline produced
  frame-intro.png, frame-05-at-11s.png          — two frames, to see the layout at a glance
  kenburns/     05.mp4 … 08.mp4                 — the pan/zoom move applied to each still
  subtitles/    7 .ass files                    — every text and shape drawn, with timings
```

## Counts and probe data

### Images — 4 files

All four identical in shape: **1536 × 2720**, PNG, `rgb24`, ~6.0 MB each (24.0 MB total).

Note the size: 1536 × 2720 is 1.42 × the 1080 × 1920 output frame. The excess is the
headroom the Ken Burns move pans and zooms within.

### Audio — 11 files

Every file is **mp3, 24 kHz, mono, 160 kbps**.

| File | Duration | Role |
| --- | --- | --- |
| `intro-2.mp3` | 2.568 s | spoken intro line |
| `hook-2.mp3` | 1.848 s | spoken hook question |
| `05-cobweb.mp3` | 1.776 s | word, item 05 |
| `06-spider.mp3` | 1.776 s | word, item 06 |
| `07-skeleton.mp3` | 1.776 s | word, item 07 |
| `08-lights.mp3` | 1.488 s | word, item 08 |
| `sentence-05-cobweb.mp3` | 2.184 s | sentence, item 05 |
| `sentence-06-spider.mp3` | 2.568 s | sentence, item 06 |
| `sentence-07-skeleton.mp3` | 2.184 s | sentence, item 07 |
| `sentence-08-lights.mp3` | 1.992 s | sentence, item 08 |
| `quiz-2.mp3` | 2.256 s | spoken quiz question |

Total narration: **22.4 s** of speech inside a **65.2 s** video. The rest is silence
held for the viewer to repeat — the gaps are load-bearing content, not padding.

### Target output — `reference/en-halloween-decorating.mp4`

| | |
| --- | --- |
| Container | MP4, 10,980,788 bytes |
| Duration | **65.216 s** on the video stream; **65.259 s** on the container |
| Video | H.264 High, **1080 × 1920**, 1629 frames → **≈24.98 fps**, 1.28 Mb/s, first PTS 42 ms |
| Audio | AAC LC, 24 kHz, **mono**, 62 kb/s |
| Overall bitrate | 1.35 Mb/s |

**The two durations are 42 ms apart and only one of them is the project's.** The
document declares `duration: 65216`, which is the **video stream** to the millisecond
and the container to nothing — more than one frame at 25 fps away. This is the
distinction [ADR-0011](../../docs/adr/0011-tool-surface-reads-checks-renders.md)'s probe
quad exists for, and `crates/montaget-core/tests/reference_video.rs` asserts that reading
the other number would move the answer before it compares anything against it.

The frame rate is the awkward number `1390080/55651`, not a clean 25 — an artifact of
the old pipeline concatenating separately-encoded segments. Montaget should produce a
clean constant rate; treat ≈25 fps as the intent. A render on the project's grid
therefore carries **1631** frames against this file's 1629, and its stream runs 65.240 s
because the last of those 1631 is held for its own 1/25 s. Both divergences are reported
as that arithmetic rather than absorbed by a tolerance.

> **The published narration sits 42 ms later than the document says, and that is this
> file's own first presentation timestamp.** Matched boundary by boundary with
> `silencedetect`, the reference's 39 speech/silence transitions run a median of 43.0 ms
> behind a Montaget render of the same project; the video stream's `start_time` is
> 42.031 ms. Take it off and every boundary agrees to inside one frame. It is a property
> of the published encode rather than a disagreement about placement — which is why the
> comparison removes it explicitly and asserts that the raw offset exceeds a frame, so the
> correction can never become unfalsifiable padding.

### Ken Burns reference — `reference/kenburns/`

Four clips, each **1080 × 1920, 25 fps, 375 frames, exactly 15.000 s**, ~4.3 MB.

These are *derived*: each is the matching still with a slow pan/zoom baked in. They are
here as the target motion, not as input — Montaget should perform this move itself from
the PNG. Each is a fixed 15 s regardless of how long its item actually runs on screen
(11.1 s to 14.5 s), so the old pipeline trimmed rather than fitted.

## Structure of the video

Seven segments, concatenated. Durations probed from the old pipeline's intermediates;
they sum to 65.216 s, matching the final.

| # | Segment | Duration | Content |
| --- | --- | --- | --- |
| 1 | intro | 3.018 s | title card over item 05's visual |
| 2 | 05-cobweb | 14.454 s | hook → word → sentence |
| 3 | 06-spider | 13.131 s | word → sentence |
| 4 | 07-skeleton | 12.160 s | word → sentence |
| 5 | 08-lights | 11.093 s | word → sentence |
| 6 | quiz | 10.160 s | question → silence → answer |
| 7 | loop-tail | 1.200 s | tail for the loop back to frame 0 |

`reference/beats.json` gives the same structure as absolute marks on one timeline:

```
intro 0.0 · hold_cobweb 3.02 · reveal_cobweb 5.32 · word_cobweb 5.32
sentence_cobweb 10.47 · word_spider 17.47 · sentence_spider 22.62
word_skeleton 30.6 · sentence_skeleton 35.75 · word_lights 42.76
sentence_lights 47.33 · quiz_question 53.85 · quiz_gap 56.11
quiz_answer 61.11 · loop_tail 63.99
```

Two timing facts worth carrying into the design: the hook holds for `hook_delay`
2.298 s before the word is revealed, and the quiz leaves a **5 s** silent gap between
question and answer.

## What is drawn on screen

Two frames are included — `reference/frame-intro.png` and `reference/frame-05-at-11s.png`
— because the layout is faster to see than to read.

> **Which frame of the MP4 each one is, and one trap in the second's name.** Neither PNG
> carried its instant, and #213 needed both in order to compare `frame` against them.
> Recovered by decoding the video and finding the frame each PNG *is* — an exact match,
> mean absolute difference 0.000, with the neighbouring frames at 0.31:
>
> | file | MP4 frame | instant on the project's timeline |
> | --- | --- | --- |
> | `frame-intro.png` | 10 | **400 ms** |
> | `frame-05-at-11s.png` | 350 | **14 000 ms** |
>
> The second file's name counts **11 s from item 05's own start** at 3 018 ms, not from
> the beginning of the video: 14 000 − 3 018 = 10 982. Reading it as 11 000 ms compares a
> render against a frame three seconds of Ken Burns away, which looks exactly like a
> renderer defect. `ci/reference_frame_instants.py` re-derives both numbers and fails if
> either stops holding.
>
> Both PNGs are 270 × 480 — a quarter of the 1080 × 1920 output — so a comparison against
> them happens at that size.

> **The committed project's Ken Burns pivoted about a different point from the published
> video's, and the file was the one that was wrong** —
> [#276](https://github.com/MBehtemam/Montaget/issues/276), now fixed. Every photo element
> declared `"origin": "top-left"` at `(0, 0)`; the published move is a **centre** pivot.
> `docs/research/sample-project-migration/README.md` D3 measured that from
> `reference/kenburns/06.mp4` when the file was written — *"the move is a centre-pivot
> zoom (centre beats top at every sample; the joint fit returns `dy = 0`)"* — and the
> migration then wrote `top-left` anyway. Nothing could catch it until a renderer existed;
> #213's first render against the published video did.
>
> It was invisible at the start of a ramp and grew with it. At 400 ms the two pivots are
> 1 × 2 px apart — smaller than the residual noted below, which is why that frame scores the
> *rejected* spelling higher and is given no vote on the question; at 14 000 ms, with the
> ramp at 1.0586, the recovered offset is (24, 52) px
> against the (32, 57) a centre pivot predicts and the (0, 0) the file used to declare —
> and the photograph plainly did not match. The recovered **scale**, 1.060, lands on the
> declared ramp, so the amplitude and the timing in the file were right and only the pivot
> was wrong.
>
> **The seven photo elements now declare `"origin": "center"` at `(540, 956)`** — the same
> rectangle at scale 1.0, and the published one at every other scale. `handle-logo` carries
> no transform and so has no observable pivot; it is unchanged. The correction is in
> `migrate.py` rather than in this file by hand, because the file is regenerated by it and
> byte-diffed by `verify.py`. `crates/montaget-core/tests/reference_frames.rs` now gates
> the photograph at both reference frames instead of masking it out of the later one, and
> `ci/reference_frame_instants.py` re-derives the finding against whatever pivot the file
> declares.
>
> **It did not close the divergence completely.** With the centre pivot and the declared
> ramp in place, the photograph still sits about 2 px from the published one at 400 ms and
> 8 × 4 px at 14 000 ms — a quarter of the horizontal error the old pivot carried and a
> fourteenth of the vertical one. D3 flagged the same thing from the other side (*"the pure
> zoom model's fit quality falls with `t`"*). It is measured and reported on every run by
> `reference_frames.rs`, and owned by
> [#299](https://github.com/MBehtemam/Montaget/issues/299).

The 1080 × 1920 frame is split. The **image occupies the top ~1300 px** (`card_h: 1300`
in `beats.json`), cropped to the frame's width; below it is a flat **cream card**
(`#FBF3E3`) that carries the text. Text is never drawn over the photograph.

A header sits over the top of the image, present for every segment:

- a cream panel at (48, 88)–(420, 172) holding a **drawn flag** — three filled
  rectangles, a blue field with a white cross — and the language chip `English` at 52 px
- a second cream panel at (438, 88)–(1032, 172) holding the handle
  `@FluencyInActionEnglish` at 34 px, with `brand/logo-en.png` as a circular badge

Both panels and the lower card have **square corners** — verified against the published
MP4 and against the ASS drawing commands, which are literal four-point rectangles
(`m 48 88 l 420 88 420 172 48 172`). An earlier revision of this file called the panels
"rounded"; they are not. The badge's roundness is **baked into the asset** — `logo-en.png`
is 800×800 RGBA with corner alpha 0 — not produced by the compositor.

Nothing in this fixture is stroked. All 35 ASS style definitions set `Outline: 0,
Shadow: 0`, and there is no inline `\bord` or `\shad` override anywhere, so
`ScaledBorderAndShadow: yes` in the headers is an inert default.

The flag drawn for the English edition is a Nordic cross, not a Union Jack. That is what
shipped; recorded as fact, not as a thing to reproduce.

In the lower card:

- the **word** line, centred at (540, 1373) at 88 px — shown as `target  -  bridge`,
  so `cobweb  -  cobweb` here
- before the word is revealed, the **hook question** occupies the same slot, centred
  slightly lower at (540, 1470), two lines broken on an explicit `\N`
- the **sentence** on a dark navy card (`#1E344C`) spanning (48, 1453)–(1032, 1622),
  its text centred at (540, 1537) at 55 px

Typography is `SF Pro Rounded`, bold throughout — **in the published video only**.
[#143](https://github.com/MBehtemam/Montaget/issues/143) re-vendored the project to Open
Runde, because SF Pro Rounded is not in this repository and is not redistributable
([ADR-0057](../../docs/adr/0057-font-vendoring-licence-gate-and-path-keyed-attestation.md)),
so every text-bearing region of a render differs from the reference frames above for a
reason that is not a defect. [#186](https://github.com/MBehtemam/Montaget/issues/186) owns
quantifying it; the falsification suite masks text out of its gate and measures it beside
it. Colours, converted from the ASS
`&HBBGGRR` form: blue `#245C8C` for the word, chip and handle; cream `#FBF3E3` for the
panels and the lower card; near-white `#FFF8E8` for the sentence on navy `#1E344C`.

Three things this fixture proves the primitives must cover, all flagged on the map:
**multi-line text** (the intro and hook break on an explicit `\N`), **filled rectangles
used as backgrounds** sized to the text they sit behind, and a **raster image overlay**
composited into the header.

## How the pieces map to one another

Per item *n* ∈ {05, 06, 07, 08}:

```
images/n.png  ──pan/zoom──▶  the item's visual for its whole segment
audio/n-<slug>.mp3           the word, spoken at the segment's word beat
audio/sentence-n-<slug>.mp3  the sentence, spoken at the sentence beat
transcript.json items[]      target · bridge · sentence · sentence_bridge
```

The intro runs over item 05's image (confirmed by comparing `reference/frame-intro.png`
with the first item's frames), and the quiz asks about item 05's sentence
(`answers_item`). So item 05's still is on screen for the intro, the hook, its own
segment and the quiz — one asset spanning four beats. `transcript.json` carries the text
of every spoken line alongside the audio that speaks it.

## The degenerate bilingual case

This edition is English taught to English speakers, so `target` and `bridge` are the
same string for all four items (`cobweb` / `cobweb`). The map calls for **bilingual**
subtitle text, and this fixture does not exercise it. The German edition of the same
short does — `der Kürbis` / `pumpkin`, `Wir haben einen großen Kürbis gekauft.` /
`We've bought a big pumpkin.` — and its assets sit beside these in the source project
under `lang/de/`, with a final MP4 for the decorating and costume shorts. Pull it in as
a second fixture if a ticket needs the pairs to actually differ.
