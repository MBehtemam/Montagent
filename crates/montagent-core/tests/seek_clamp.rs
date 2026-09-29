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
        25.0,
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
        25.0,
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
