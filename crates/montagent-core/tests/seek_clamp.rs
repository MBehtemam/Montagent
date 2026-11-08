//! The frame a source is **showing** at an instant, and the two ways the old seek got it
//! wrong.
//!
//! ADR-0096, and the defect it closes: [#387](https://github.com/MBehtemam/Montagent/issues/387)
//! (MONTAGENT-2). `ffmpeg`'s input seek returns the first frame whose timestamp is `>= t`,
//! so an instant that did not land exactly on a source frame's start painted the *next*
//! frame — one frame early for the whole of an element whose start is off the source's
//! grid — and at the last frame there was no next one, so the element silently did not
//! draw at all.
//!
//! **The fixtures are generated rather than committed**, on `source_alpha.rs`'s reasoning:
//! they are a few kilobytes `ffmpeg` re-derives in under a second, and a committed clip is
//! one no later reader can check the *frame numbering* of without decoding it anyway. Each
//! frame carries its own index in its red channel, so a test can say which frame came back
//! rather than that *a* frame came back — the distinction the whole file is about. FFV1 in
//! `gbrp` is the encode, because it is lossless, RGB, all-native to `ffmpeg` and so costs
//! none of ADR-0009's *"an `ffmpeg` the user supplies"* optionality.
//!
//! The generator is asserted before anything else, for `source_alpha.rs`'s reason: a
//! fixture that stops numbering its frames must fail as a fixture error and never as a
//! finding about the seek.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::media::tools;
use montagent_render::decode::{self, Decoder};

const SIZE: u32 = 64;
/// 82 frames, which is the source length MONTAGENT-2 was reported on.
const FRAMES: i64 = 82;

/// The frame index in the red channel, times three so a single off-by-one in the encode
/// cannot read as the neighbouring frame.
fn generator(rate: &str) -> String {
    format!("color=c=black:s={SIZE}x{SIZE}:r={rate},format=rgba,geq=r='N*3':g='0':b='0':a='255'")
}

fn ffmpeg() -> PathBuf {
    tools::resolve().expect("an ffmpeg to decode with").ffmpeg
}

/// Encode `FRAMES` numbered frames at `rate`, or `None` where this `ffmpeg` cannot.
///
/// **The container is part of the fixture, not a detail of it.** Matroska's default
/// timecode scale is one millisecond, so it *rounds* every frame timestamp to the
/// millisecond: in a `.mkv` at 30000/1001 frame 4 starts at exactly 133 ms rather than at
/// 133.467 ms. That is not a flaw in the container and it is the whole reason this file
/// exists — the grid a source is on is a property of the timestamps it carries, never of
/// the frame rate it declares. QuickTime keeps the rational grid, so the fractional-rate
/// test asks for `.mov` and the millisecond-grid tests are content with `.mkv`.
fn encode(dir: &Path, name: &str, rate: &str) -> Option<PathBuf> {
    encode_from(dir, name, rate, &[])
}

/// [`encode`], with `extra` arguments in front of the output — `-output_ts_offset` is the one
/// this file needs, to make a source whose own frames start after zero.
fn encode_from(dir: &Path, name: &str, rate: &str, extra: &[&str]) -> Option<PathBuf> {
    let path = dir.join(name);
    let ok = Command::new(ffmpeg())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &generator(rate),
            "-frames:v",
            &FRAMES.to_string(),
            "-c:v",
            "ffv1",
            "-pix_fmt",
            "gbrp",
        ])
        .args(extra)
        .arg(&path)
        .output()
        .ok()?
        .status
        .success();
    ok.then_some(path)
}

/// Which frame this is, read back out of the red channel.
fn index_of(frame: &decode::DecodedFrame) -> i64 {
    let red = frame.rgba[0] as i64;
    assert_eq!(
        red % 3,
        0,
        "red {red} is not a frame index this fixture wrote"
    );
    red / 3
}

/// The frame the source is showing at `at_ms`, as its index.
fn frame_index_at(path: &Path, at_ms: i64) -> Result<i64, String> {
    decode::frame_at(
        &ffmpeg(),
        &path.to_string_lossy(),
        Decoder::Auto,
        at_ms,
        SIZE,
        SIZE,
    )
    .map(|frame| index_of(&frame))
}

#[test]
fn the_generator_numbers_its_frames_which_is_what_every_other_test_here_reads() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // Decoded as a whole run, in order, so the numbering is checked against every frame
    // rather than against the ones the later tests happen to ask for.
    let mut frames = decode::frames_from(
        &ffmpeg(),
        &path.to_string_lossy(),
        Decoder::Auto,
        0,
        decode::Pace {
            fps: 25,
            speed: (1, 1),
        },
        SIZE,
        SIZE,
    )
    .expect("a run of frames");
    let mut seen = 0;
    while let Some(frame) = frames.next_frame().expect("a frame or an end") {
        assert_eq!(index_of(&frame), seen, "frame {seen} is misnumbered");
        seen += 1;
    }
    assert_eq!(seen, FRAMES, "the fixture is not {FRAMES} frames long");
}

#[test]
fn an_off_grid_instant_paints_the_frame_it_is_inside_and_not_the_next_one() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // 1234 ms is inside frame 30, which covers [1200, 1240). The old seek returned 31: the
    // first frame at or after the instant. This is the one-frame-early shift that ran for
    // the whole of an element whose start was off the source's grid.
    assert_eq!(frame_index_at(&path, 1234), Ok(30));
    assert_eq!(frame_index_at(&path, 1200), Ok(30), "its own start");
    assert_eq!(frame_index_at(&path, 1239), Ok(30), "its last millisecond");
    assert_eq!(
        frame_index_at(&path, 1240),
        Ok(31),
        "the next frame's start"
    );
}

#[test]
fn an_instant_exactly_on_a_frames_start_paints_that_frame() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // `select` compares `t` as a float, so a frame whose start *is* the instant compares
    // marginally above it: without ADR-0096's microsecond of slack this returned 34, and
    // the whole-second boundaries are where that showed up.
    assert_eq!(frame_index_at(&path, 1400), Ok(35));
    assert_eq!(frame_index_at(&path, 1000), Ok(25));
    assert_eq!(frame_index_at(&path, 2000), Ok(50));
    assert_eq!(frame_index_at(&path, 0), Ok(0));
}

#[test]
fn the_last_frame_is_painted_rather_than_dropped_which_is_montagent_2() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // The reported case, to the millisecond: an 82-frame 25 fps source is 3280 ms long, and
    // an element whose start was off the grid asked for 3276 ms — inside frame 81, which
    // covers [3240, 3280). `ffmpeg` had no frame at or after it, exited cleanly having
    // written nothing, and the element did not draw.
    assert_eq!(frame_index_at(&path, 3276), Ok(81));
    assert_eq!(frame_index_at(&path, 3240), Ok(81), "its own start");
    assert_eq!(frame_index_at(&path, 3279), Ok(81), "its last millisecond");
    // Past the source's end entirely. ADR-0020 sends `overrun: "hold"` here, and what it
    // must get is the frame the source ends on rather than nothing.
    assert_eq!(frame_index_at(&path, 3280), Ok(81));
    assert_eq!(frame_index_at(&path, 3400), Ok(81));
}

#[test]
fn a_fractional_frame_rate_is_clamped_on_its_own_grid() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mov", "30000/1001") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // 30000/1001 puts frame starts off the millisecond grid, which is why the clamp cannot
    // be arithmetic on an integer frame period. Frame 30 starts at 30*1001/30000 s = 1001
    // ms exactly; frame 4 starts at 133.467 ms, so 133 ms is still inside frame 3.
    assert_eq!(frame_index_at(&path, 1001), Ok(30), "exactly a frame start");
    assert_eq!(frame_index_at(&path, 1000), Ok(29), "one ms before it");
    assert_eq!(
        frame_index_at(&path, 133),
        Ok(3),
        "inside frame 3, which ends at 133.467"
    );
    assert_eq!(
        frame_index_at(&path, 134),
        Ok(4),
        "the millisecond after it starts"
    );
}

#[test]
fn an_instant_far_past_the_end_still_says_so_rather_than_inventing_a_frame() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // The clamp reaches back one window (200 ms), which is what makes it a clamp and not a
    // search of the whole file. An instant further past the end than that is a source that
    // ends well before the range the document declares — ADR-0006 prefers the loud failure,
    // and ADR-0093 makes it an `error` that withholds the deliverable.
    let e = frame_index_at(&path, 3280 + 500).expect_err("no frame within a window");
    assert!(e.contains("numbered.mkv"), "{e}");
    assert!(e.contains("no frame at or before"), "{e}");
}

#[test]
fn a_source_whose_frames_start_after_zero_paints_its_first_frame_and_does_not_refuse() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    // The origin this repository's own reference MP4 carries: its frames start at 42.031 ms,
    // not at zero, which `tests/reference_video.rs` records as the reason ADR-0011 needs a
    // quad at all. An element with `source_start: 0` on such a source asks for instants no
    // frame covers.
    let Some(path) = encode_from(&dir, "offset.mov", "25", &["-output_ts_offset", "0.042"]) else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // The clamp is symmetric: a container's own origin is not the author's mistake, so these
    // paint the source's first frame rather than refusing. Asking for the frame *at or
    // before* alone would have moved MONTAGENT-2's silent failure from the end of a source
    // to the beginning of one.
    assert_eq!(
        frame_index_at(&path, 0),
        Ok(0),
        "before the first frame starts"
    );
    assert_eq!(
        frame_index_at(&path, 42),
        Ok(0),
        "the millisecond it starts in"
    );
    assert_eq!(frame_index_at(&path, 43), Ok(0), "just after it starts");
    // Frame 1 starts at 82.031 ms, so 82 ms is still inside frame 0 — the offset origin
    // shifts the whole grid, which is exactly why the grid cannot be computed from a rate.
    assert_eq!(frame_index_at(&path, 82), Ok(0), "still inside frame 0");
    assert_eq!(
        frame_index_at(&path, 83),
        Ok(1),
        "and the grid carries on from there"
    );
}

/// An `ffmpeg` that refuses any run whose arguments include `refused`, the way ffmpeg 9
/// refuses `-vsync`, and hands every other run to the real one.
///
/// A stub rather than a real old or new `ffmpeg`, because the claim under test is about
/// what `frame_at` does with a *failed* spawn, and the only way to hold that on every
/// machine is to make the failure rather than hope the installed build has one.
#[cfg(unix)]
fn refusing(dir: &Path, refused: &str) -> PathBuf {
    let path = dir.join("ffmpeg");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do case \"$a\" in *{refused}*) \
             echo \"Unrecognized option '{refused}'.\" >&2; exit 8;; esac; done\n\
             exec \"{}\" \"$@\"\n",
            ffmpeg().display()
        ),
    )
    .expect("write the stub");
    let mut mode = std::fs::metadata(&path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o755);
    std::fs::set_permissions(&path, mode).expect("make the stub executable");
    path
}

#[cfg(unix)]
#[test]
fn a_seek_whose_ffmpeg_failed_is_refused_and_never_painted_from_the_fallback() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // ADR-0113, and the shape ffmpeg 9 actually had: the `select` run exits 8 on an argument
    // it no longer knows, and the fallback run — which does not pass it — succeeds. Reading
    // the failure as *"no frame at or before"* painted frame 26 here, the earliest in the
    // window, for an instant inside frame 30, and said nothing.
    let stub = refusing(&dir, "select=");
    let refused = decode::frame_at(
        &stub,
        &path.to_string_lossy(),
        Decoder::Auto,
        1234,
        SIZE,
        SIZE,
    )
    .map(|frame| index_of(&frame));
    let e = refused.expect_err("a failed seek is a refusal, not frame 26");
    assert!(e.contains("numbered.mkv"), "it names the source: {e}");
    assert!(
        e.contains("Unrecognized option"),
        "and what ffmpeg said, because exit 70 is retry-or-report: {e}"
    );
}

#[cfg(unix)]
#[test]
fn a_run_whose_ffmpeg_failed_is_an_error_and_not_an_empty_series() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // `frames_from`'s end of run is how `measure`'s coverage series finds a source's end, so a
    // failed `ffmpeg` read as an end is a series of no frames about a source that has 82.
    let stub = refusing(&dir, "fps=");
    let mut frames = decode::frames_from(
        &stub,
        &path.to_string_lossy(),
        Decoder::Auto,
        0,
        decode::Pace {
            fps: 25,
            speed: (1, 1),
        },
        SIZE,
        SIZE,
    )
    .expect("the stub runs");
    let e = frames
        .next_frame()
        .expect_err("a failed run is an error, not an end");
    assert!(e.contains("numbered.mkv"), "it names the source: {e}");
    assert!(e.contains("Unrecognized option"), "{e}");
}

// ---------------------------------------------------------------------------------------
// ADR-0127: a *run* of frames is the renderer's frames too.
//
// `frames_from` is `measure`'s coverage series, and a series is only worth reading if frame
// `n` of it is the frame the render paints at frame `n` of the element. The render's rule is
// integer arithmetic ending in `frame_at` — instant `⌊n × 1000 / fps⌋`, offset
// `source_start + source_advance(instant, speed)`, then the last frame starting at or before
// it — so that is the oracle here, and `frame_at` itself is held to the pixels above.
// ---------------------------------------------------------------------------------------

/// The first `count` frames of a run, as indices.
fn run_indices(path: &Path, from_ms: i64, fps: i64, speed: (i128, i128), count: usize) -> Vec<i64> {
    let mut frames = decode::frames_from(
        &ffmpeg(),
        &path.to_string_lossy(),
        Decoder::Auto,
        from_ms,
        decode::Pace { fps, speed },
        SIZE,
        SIZE,
    )
    .expect("a run of frames");
    let mut seen = Vec::new();
    while seen.len() < count {
        match frames.next_frame().expect("a frame or an end") {
            Some(frame) => seen.push(index_of(&frame)),
            None => break,
        }
    }
    seen
}

/// What the render paints at timeline frame `n` of an element starting `from_ms` into this
/// source at `speed` — `render`'s own arithmetic, then `frame_at_us` (ADR-0251).
fn painted(path: &Path, from_ms: i64, fps: i64, speed: (i128, i128), n: i64) -> i64 {
    let offset_us = decode::offset_us(
        decode::Origin {
            timeline_ms: 0,
            source_ms: from_ms,
        },
        decode::Pace { fps, speed },
        n,
    );
    decode::frame_at_us(
        &ffmpeg(),
        &path.to_string_lossy(),
        Decoder::Auto,
        offset_us,
        SIZE,
        SIZE,
    )
    .map(|frame| index_of(&frame))
    .expect("a frame")
}

#[test]
fn a_run_paints_what_the_render_paints_on_and_off_the_grid_at_any_rate_and_speed() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    // The three grids a source can be on that matter here: the project's own millisecond
    // grid, a fractional one that never lands on a whole millisecond, and one whose frames
    // start after zero — this repository's reference MP4 begins at 42.031 ms.
    let (Some(cfr), Some(ntsc), Some(late)) = (
        encode(&dir, "numbered.mkv", "25"),
        encode(&dir, "numbered.mov", "30000/1001"),
        encode_from(&dir, "late.mkv", "25", &["-output_ts_offset", "0.042"]),
    ) else {
        eprintln!("skipped: this ffmpeg cannot encode the fixtures");
        return;
    };
    const SAMPLES: i64 = 12;
    for path in [&cfr, &ntsc, &late] {
        // (from_ms, fps, speed as the document writes it, speed as a ratio)
        for (from_ms, fps, literal, ratio) in [
            (0, 25, "1", (1, 1)),
            (1234, 25, "1", (1, 1)),
            (0, 30, "1", (1, 1)),
            (1210, 24, "1", (1, 1)),
            (1239, 25, "2", (2, 1)),
            (20, 30, "0.5", (1, 2)),
            (1001, 30, "1.5", (3, 2)),
            (367, 60, "0.645", (645, 1000)),
        ] {
            let got = run_indices(path, from_ms, fps, ratio, SAMPLES as usize);
            let want: Vec<i64> = (0..SAMPLES)
                .map(|n| painted(path, from_ms, fps, ratio, n))
                .collect();
            assert_eq!(
                got,
                want,
                "{} from {from_ms} ms at {fps} fps, speed {literal}: the run is not the frames \
                 the render paints",
                path.display()
            );
        }
    }
}

#[test]
fn an_off_grid_run_starts_on_the_frame_showing_and_not_the_next_one() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // 1234 ms is inside frame 30. The old `-ss` without `-copyts` started on 31 and stayed
    // one frame ahead for the whole run.
    assert_eq!(run_indices(&path, 1234, 25, (1, 1), 4), [30, 31, 32, 33]);
}

#[test]
fn a_run_at_another_rate_holds_the_frame_showing_rather_than_rounding_to_the_nearest() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // At 30 fps over a 25 fps source the render paints at 0, 33, 66, 100, 133, 166 and 200 ms,
    // which are inside frames 0, 0, 1, 2, 3, 4 and 5. `fps=`'s default nearest-tick rounding
    // took frame 1 at 33 ms — the frame about to start, not the one showing.
    assert_eq!(run_indices(&path, 0, 30, (1, 1), 7), [0, 0, 1, 2, 3, 4, 5]);
}

#[test]
fn a_retimed_run_moves_speed_over_fps_through_its_source_per_frame() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode(&dir, "numbered.mkv", "25") else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // `speed: 2` at 25 fps moves 80 ms per frame, so every other source frame. The old run
    // sampled at `fps × speed` = 50 per source second, which is every *half* frame: 0, 0, 1,
    // 1, … — a series covering a quarter of the range it named.
    assert_eq!(run_indices(&path, 0, 25, (2, 1), 5), [0, 2, 4, 6, 8]);
    assert_eq!(run_indices(&path, 0, 25, (1, 2), 5), [0, 0, 1, 1, 2]);
}

#[test]
fn a_run_over_a_source_that_starts_late_holds_its_first_frame_until_the_second_starts() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(path) = encode_from(&dir, "late.mkv", "25", &["-output_ts_offset", "0.042"]) else {
        eprintln!("skipped: this ffmpeg cannot encode the fixture");
        return;
    };
    // Frame k starts at 42 + 40k ms. At 0 and 40 ms no frame has started, and `frame_at`'s
    // symmetric clamp paints frame 0; at 80 ms frame 0 is still showing. The old run rebased
    // the file onto zero and was a frame ahead from its first instant.
    assert_eq!(run_indices(&path, 0, 25, (1, 1), 5), [0, 0, 0, 1, 2]);
}

#[test]
fn a_speed_finer_than_a_run_can_sample_exactly_is_refused_and_not_approximated() {
    let e = decode::frames_from(
        Path::new("ffmpeg"),
        "clip.mov",
        Decoder::Auto,
        0,
        decode::Pace {
            fps: 25,
            speed: (1_000_001, 1_000_000),
        },
        SIZE,
        SIZE,
    )
    .err()
    .expect("a speed of seven significant decimals is refused");
    assert!(e.contains("clip.mov") && e.contains("exactly"), "{e}");
    // In lowest terms first: 2_000_000/1_000_000 is 2, which is exact.
    assert!(
        decode::frames_from(
            Path::new("/nonexistent/ffmpeg"),
            "clip.mov",
            Decoder::Auto,
            0,
            decode::Pace {
                fps: 25,
                speed: (2_000_000, 1_000_000)
            },
            SIZE,
            SIZE,
        )
        .err()
        .is_some_and(|e| e.contains("could not be run"))
    );
}

#[test]
fn a_30_fps_source_in_a_30_fps_project_steps_one_to_one_across_render_frame_and_measure() {
    // #522 / ADR-0251: at 30 fps, timeline frame 1 is assigned integer millisecond 33 ms
    // (`⌊1000 / 30⌋`), while a 30 fps QuickTime source has frame 1 at `1/30 s = 33.333... ms`.
    // Mapping timeline frame `n` through its exact rational time `n * 1000 / fps` ms (and
    // sub-millisecond source microseconds) means timeline frames `0..10` select source frames
    // `0..10` in `frames_from` (`measure`), `render` (both `Feeds` and `PerFrame`), and
    // `frame --at` at each frame's floored instant (`0, 33, 66, 100, ...`).
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let Some(clip) = encode(&dir, "thirty.mov", "30") else {
        eprintln!("skipped: this ffmpeg cannot encode the 30 fps fixture");
        return;
    };

    // 1. `frames_from` (used by `measure`'s coverage series and `render`'s feeds at origin 0)
    // steps 0, 1, 2, ..., 9 without holding 0 at frame 1 or skipping 2 at frame 3.
    let expected: Vec<i64> = (0..10).collect();
    assert_eq!(run_indices(&clip, 0, 30, (1, 1), 10), expected);

    // 2. Build a 30 fps project playing `thirty.mov` from 0..333 ms (10 frames: 0..10) and
    // verify `render` (`Feeds` and `PerFrame`), `frame --at`, and `measure` all paint the
    // exact 1-to-1 sequence 0..10.
    let source = clip.display().to_string().replace('\\', "/");
    let body = common::canonical(&format!(
        r##"{{"frame":{{"width":{SIZE},"height":{SIZE}}},"fps":30,"background":"#000000",
            "duration":333,"output":"out/thirty.mp4","tracks":[
            {{"name":"v","layer":0,"elements":[
              {{"id":"clip","type":"video","start":0,"end":333,"source":"{source}",
                "source_start":0,"source_end":333,"x":0,"y":0,"origin":"top-left",
                "width":{SIZE},"height":{SIZE},"fit":"literal","volume":0,
                "effects":[{{"name":"chroma","color":"#00FF00","tolerance":0.2,
                             "softness":0.0,"spill":0.0}}]}}]}}]}}"##
    ));
    let project_path = common::write_project(&dir, "thirty.montagent.json", &body);

    let feeds = montagent_core::verbs::render::paint_span(
        &project_path,
        0,
        333,
        montagent_core::verbs::render::Supplying::Feeds,
    )
    .expect("feeds paint");
    let per_frame = montagent_core::verbs::render::paint_span(
        &project_path,
        0,
        333,
        montagent_core::verbs::render::Supplying::PerFrame,
    )
    .expect("per-frame paint");
    assert_eq!(feeds.frames.len(), 10);
    assert_eq!(feeds.frames, per_frame.frames);

    for n in 0..10i64 {
        let rgb = &feeds.frames[n as usize];
        assert_eq!(
            i64::from(rgb[0]) / 3,
            n,
            "render timeline frame {n} should paint source frame {n}"
        );

        let instant = (n * 1000) / 30;
        let single = montagent_core::verbs::frame::frame(
            &project_path,
            &montagent_core::verbs::frame::Ask {
                at: Some(instant),
                full: true,
                png: true,
                ..Default::default()
            },
        );
        let decoded = image::load_from_memory(&single.image().expect("frame image").bytes)
            .expect("valid PNG")
            .to_rgba8();
        assert_eq!(
            i64::from(decoded.as_raw()[0]) / 3,
            n,
            "frame --at {instant} (timeline frame {n}) should paint source frame {n}"
        );
    }

    let element = serde_json::from_str::<serde_json::Value>(&body).expect("project JSON")["tracks"]
        [0]["elements"][0]
        .clone();
    let measured = montagent_core::verbs::measure::measure(
        &project_path,
        &montagent_core::verbs::measure::Ask {
            element: Some(element),
            ..Default::default()
        },
    )
    .to_json();
    assert_eq!(
        measured["measure"]["frames"].as_array().map(Vec::len),
        Some(10),
        "{measured}"
    );

    // 3. Off-grid `start` (`start: 10, end: 110, source_start: 0, source_end: 100` at 30 fps):
    // the first sampled timeline frame is frame 1 (`33 ms`, exact time `33.333 ms`, which is
    // `23.333 ms` into the source — still inside source frame 0 `[0, 33.333 ms)`), and the
    // second sampled timeline frame is frame 2 (`66 ms`, exact time `66.667 ms`, which is
    // `56.667 ms` into the source — inside source frame 1 `[33.333, 66.667 ms)`).
    let off_grid_body = common::canonical(&format!(
        r##"{{"frame":{{"width":{SIZE},"height":{SIZE}}},"fps":30,"background":"#000000",
            "duration":110,"output":"out/off_grid.mp4","tracks":[
            {{"name":"v","layer":0,"elements":[
              {{"id":"clip","type":"video","start":10,"end":110,"source":"{source}",
                "source_start":0,"source_end":100,"x":0,"y":0,"origin":"top-left",
                "width":{SIZE},"height":{SIZE},"fit":"literal","volume":0,
                "effects":[{{"name":"chroma","color":"#000000","tolerance":0.002,
                             "softness":0.0,"spill":0.0}}]}}]}}]}}"##
    ));
    let off_grid_path = common::write_project(&dir, "off_grid.montagent.json", &off_grid_body);
    let off_grid_element = serde_json::from_str::<serde_json::Value>(&off_grid_body)
        .expect("project JSON")["tracks"][0]["elements"][0]
        .clone();
    let off_grid_measured = montagent_core::verbs::measure::measure(
        &off_grid_path,
        &montagent_core::verbs::measure::Ask {
            element: Some(off_grid_element),
            ..Default::default()
        },
    )
    .to_json();
    let off_grid_frames = off_grid_measured["measure"]["frames"]
        .as_array()
        .expect("frames array");
    assert_eq!(off_grid_frames.len(), 3, "{off_grid_measured}");
    // Source frame 1 has R = 3 (`#030000`, chroma distance ~0.0044 from `#000000` > 0.002),
    // while source frame 0 has R = 0 (`#000000`, chroma distance 0 <= 0.002).
    assert!(
        off_grid_frames[0]["transparent"].as_f64().unwrap() > 0.9,
        "first sampled frame at 33.333 ms (23.333 ms in source) is source frame 0: {off_grid_frames:?}"
    );
    assert!(
        off_grid_frames[1]["opaque"].as_f64().unwrap() > 0.9,
        "second sampled frame at 66.667 ms (56.667 ms in source) is source frame 1: {off_grid_frames:?}"
    );
    assert!(
        off_grid_frames[2]["opaque"].as_f64().unwrap() > 0.9,
        "third sampled frame at 100.0 ms (90.0 ms in source) is source frame 2: {off_grid_frames:?}"
    );

    // 4. Rational `fps: [24000, 1001]` in `frame --at`: at 24000/1001 fps, timeline frame 1
    // has painted instant `floor(1001 / 24) = 41 ms`, while its exact time is
    // `1001000 / 24000 = 41.708333 ms` (`41_709 us`). On a `24000/1001` source whose frame 1
    // starts at `41.708333 ms`, `frame --at 41` must recognise `41` as timeline frame 1 at
    // `[24000, 1001]` and seek to `41_709 us` (source frame 1), not `41_000 us` (source frame 0).
    if let Some(ntsc_clip) = encode(&dir, "ntsc.mov", "24000/1001") {
        let ntsc_source = ntsc_clip.display().to_string().replace('\\', "/");
        let ntsc_body = common::canonical(&format!(
            r##"{{"frame":{{"width":{SIZE},"height":{SIZE}}},"fps":[24000,1001],"background":"#000000",
                "duration":200,"output":"out/ntsc.mp4","tracks":[
                {{"name":"v","layer":0,"elements":[
                  {{"id":"clip","type":"video","start":0,"end":200,"source":"{ntsc_source}",
                    "source_start":0,"source_end":200,"x":0,"y":0,"origin":"top-left",
                    "width":{SIZE},"height":{SIZE},"fit":"literal","volume":0}}]}}]}}"##
        ));
        let ntsc_path = common::write_project(&dir, "ntsc.montagent.json", &ntsc_body);
        let single = montagent_core::verbs::frame::frame(
            &ntsc_path,
            &montagent_core::verbs::frame::Ask {
                at: Some(41),
                full: true,
                png: true,
                ..Default::default()
            },
        );
        let decoded = image::load_from_memory(&single.image().expect("frame image").bytes)
            .expect("valid PNG")
            .to_rgba8();
        assert_eq!(
            i64::from(decoded.as_raw()[0]) / 3,
            1,
            "frame --at 41 on [24000, 1001] timeline must paint source frame 1"
        );
    }
}
