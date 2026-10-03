//! **Feeds paint the pixels `frame` paints** (ADR-0141), and they cost what they should.
//!
//! `render` and `preview` read each `video` element through one long-lived `ffmpeg` — a
//! **feed** — instead of one `frame_at` spawn per frame. The painter is the same one `frame`
//! uses; only the supplier of its pixels differs. So the standing evidence is a byte
//! comparison of the two suppliers through the one painter: every case below paints a short
//! span through the feed supplier and through `frame`'s per-frame supplier and compares the
//! **rasters before encoding**, byte for byte, so no encoder can quantize a difference away
//! (`render::paint_span`).
//!
//! The cases are #626 §5's: a hold, a loop wrap, leaving and re-entering, a partial range that
//! starts partway through an element, `speed` ≥ 2 and < 1, an off-grid `fps`, two elements on
//! one source at different offsets, holding past the source's end, an offset exactly at the
//! probed end, and a VP9-with-alpha source (`Decoder::LibVpxVp9`, the decoder the gate check
//! did not cover). Every case also states how many `ffmpeg`s it may spawn, through the
//! `#[doc(hidden)]` per-thread counts: **feeds opened plus reopens, never one per frame.**
//!
//! Then #636 §7's seven feed-budget cases, through the `#[doc(hidden)]` budget override:
//! an element past the budget is painted per frame with the same pixels, and the render
//! answer names it.
//!
//! **Codecs covered**, at [`decode::FEED_THREADS`] decoder threads: H.264 (`libx264`, long GOP
//! with B-frames), FFV1 in Matroska (a container with no stream duration), and VP9 with an
//! alpha side stream through `libvpx-vp9`. The fixtures are generated, a few kilobytes each,
//! on `seek_clamp.rs`'s reasoning: a committed clip is one nobody can re-derive.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::media::tools;
use montagent_core::verbs::frame::supply::{self, Counts};
use montagent_core::verbs::render::{Rasters, Supplying, paint_span};
use montagent_render::decode;

use common::{canonical, has_ffprobe, tempdir, write_project};

const SOURCE_FRAMES: i64 = 82;

fn ffmpeg() -> PathBuf {
    tools::resolve().expect("an ffmpeg").ffmpeg
}

/// `SOURCE_FRAMES` frames of a moving test pattern at 25 fps, `size`, encoded with `args`,
/// or `None` where this `ffmpeg` cannot. Every frame differs from its neighbours, so a
/// supplier one frame off cannot pass.
fn source(dir: &Path, name: &str, size: &str, args: &[&str]) -> Option<PathBuf> {
    let path = dir.join(name);
    let ok = Command::new(ffmpeg())
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi"])
        .args(["-i", &format!("testsrc2=s={size}:r=25")])
        .args(["-frames:v", &SOURCE_FRAMES.to_string()])
        .args(args)
        .arg(&path)
        .output()
        .ok()?
        .status
        .success();
    ok.then_some(path)
}

/// H.264 with a two-second GOP and B-frames, the shape a real source has.
fn h264(dir: &Path, name: &str, size: &str) -> PathBuf {
    source(
        dir,
        name,
        size,
        &[
            "-c:v", "libx264", "-pix_fmt", "yuv420p", "-g", "50", "-bf", "2",
        ],
    )
    .expect("libx264 is ADR-0115's floor")
}

/// A project of `fps` on a 160x120 frame, with `tracks` as written.
fn project(dir: &Path, fps: i64, tracks: &str) -> PathBuf {
    write_project(
        dir,
        "feeds.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":160,"height":120}},"fps":{fps},"background":"#000000",
                "duration":4000,"output":"out/feeds.mp4","tracks":[{tracks}]}}"##
        )),
    )
}

/// One track holding `elements`.
fn track(name: &str, layer: i64, elements: &[String]) -> String {
    format!(
        r#"{{"name":"{name}","layer":{layer},"elements":[{}]}}"#,
        elements.join(",")
    )
}

/// A `video` element. `extra` is spliced in as written: `"speed":2,` or `"overrun":"loop",`.
#[allow(clippy::too_many_arguments)]
fn video(
    id: &str,
    source: &Path,
    (start, end): (i64, i64),
    (source_start, source_end): (i64, i64),
    (x, width, height): (i64, i64, i64),
    extra: &str,
) -> String {
    let source = source.display().to_string().replace('\\', "/");
    format!(
        r#"{{"id":"{id}","type":"video","start":{start},"end":{end},{extra}"source":"{source}",
            "source_start":{source_start},"source_end":{source_end},"x":{x},"y":0,
            "origin":"top-left","width":{width},"height":{height},"fit":"literal","volume":0}}"#
    )
}

/// Paint `[from, to)` through `supplying`, counting what it spawned.
fn painted(path: &Path, from: i64, to: i64, supplying: Supplying) -> (Rasters, Counts) {
    supply::reset_counts();
    let rasters = paint_span(path, from, to, supplying).expect("the span paints");
    let counts = supply::counts();
    assert!(
        rasters.declined.is_empty(),
        "{supplying:?} declined to paint: {:?}",
        rasters.declined
    );
    assert_eq!(
        counts.open, 0,
        "every feed is closed once the painter is gone"
    );
    (rasters, counts)
}

/// The two suppliers' rasters for `[from, to)`, asserted byte for byte equal, and the feed
/// supplier's counts.
#[track_caller]
fn same_pixels(path: &Path, from: i64, to: i64) -> Counts {
    let (per_frame, _) = painted(path, from, to, Supplying::PerFrame);
    let (feeds, counts) = painted(path, from, to, Supplying::Feeds);
    assert_eq!(per_frame.frames.len(), feeds.frames.len());
    for (n, (want, got)) in per_frame.frames.iter().zip(&feeds.frames).enumerate() {
        assert!(
            want == got,
            "frame {n} of {from}..{to}: the feed's raster differs from frame_at's"
        );
    }
    assert_eq!(per_frame.painted, feeds.painted);
    // The fixture must be moving, or equal rasters would prove nothing about which frame.
    assert!(
        feeds.frames.windows(2).any(|pair| pair[0] != pair[1]),
        "the span shows a still picture, which cannot tell one frame from the next"
    );
    counts
}

/// What a case may spawn: feeds opened, reopened, and single `frame_at`s.
#[track_caller]
fn spawned(counts: Counts, opened: u64, reopened: u64, frame_at: u64) {
    assert_eq!(
        (counts.opened, counts.reopened, counts.frame_at),
        (opened, reopened, frame_at),
        "feeds opened, reopened and frame_at spawns"
    );
}

#[test]
fn a_feed_plays_the_frames_frame_at_paints_and_one_element_opens_one_feed() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    // Downscaled, and upscaled to an odd aspect, from the same source.
    let path = project(
        &dir,
        25,
        &[
            track(
                "a",
                0,
                &[video("down", &clip, (0, 2400), (0, 2400), (0, 40, 30), "")],
            ),
            track(
                "b",
                1,
                &[video("up", &clip, (0, 2400), (400, 2800), (60, 90, 50), "")],
            ),
        ]
        .join(","),
    );
    // 60 frames, two elements: two feeds, never one spawn per frame.
    spawned(same_pixels(&path, 0, 2400), 2, 0, 0);
}

#[test]
fn a_hold_and_an_offset_exactly_at_the_probed_end_take_one_terminal_frame() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    // 82 frames at 25 fps end at 3280 ms. `hold` resolves to `source_end`, which is exactly
    // the probed end: the feed ends cleanly there, one `frame_at` takes the terminal frame,
    // and every held frame after it is served without another spawn.
    let path = project(
        &dir,
        25,
        &track(
            "a",
            0,
            &[video(
                "held",
                &clip,
                (0, 4000),
                (0, 3280),
                (0, 64, 48),
                r#""overrun":"hold","#,
            )],
        ),
    );
    let counts = same_pixels(&path, 0, 4000);
    assert_eq!((counts.opened, counts.reopened), (1, 0));
    assert!(counts.frame_at <= 1, "{counts:?}");
}

#[test]
fn holding_past_the_end_of_the_source_is_served_from_the_terminal_frame() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // FFV1 in Matroska: a container that states no stream duration, so the end is the
    // container's.
    let clip = source(
        &dir,
        "clip.mkv",
        "64x48",
        &["-c:v", "ffv1", "-pix_fmt", "yuv420p"],
    )
    .expect("ffv1 is native");
    // `source_end` 120 ms past the last frame's end, inside `frame_at`'s window: the render
    // asks for offsets the source has no frame starting at, and gets its last.
    let path = project(
        &dir,
        25,
        &track(
            "a",
            0,
            &[video(
                "held",
                &clip,
                (0, 4000),
                (0, 3400),
                (0, 64, 48),
                r#""overrun":"hold","#,
            )],
        ),
    );
    let counts = same_pixels(&path, 2000, 4000);
    assert_eq!((counts.opened, counts.reopened), (1, 0), "{counts:?}");
    assert_eq!(counts.frame_at, 1, "one terminal frame, then the cache");
}

#[test]
fn a_loop_wrap_reopens_the_feed_once_per_wrap() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    let path = project(
        &dir,
        25,
        &track(
            "a",
            0,
            &[video(
                "looped",
                &clip,
                (0, 2600),
                (200, 1200),
                (0, 64, 48),
                r#""overrun":"loop","#,
            )],
        ),
    );
    // Wraps at 1000 and 2000 ms.
    spawned(same_pixels(&path, 0, 2600), 1, 2, 0);
}

#[test]
fn leaving_and_re_entering_closes_the_feed_and_opens_another() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    let path = project(
        &dir,
        25,
        &track(
            "a",
            0,
            &[
                video("first", &clip, (0, 800), (0, 800), (0, 64, 48), ""),
                video("again", &clip, (1200, 2000), (400, 1200), (0, 64, 48), ""),
            ],
        ),
    );
    spawned(same_pixels(&path, 0, 2400), 2, 0, 0);
}

#[test]
fn a_partial_range_opens_its_feed_partway_through_the_element() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    let path = project(
        &dir,
        30,
        &track(
            "a",
            0,
            &[video(
                "clip",
                &clip,
                (100, 3100),
                (120, 3120),
                (0, 64, 48),
                "",
            )],
        ),
    );
    // From an off-grid instant, at a rate whose instants are not whole milliseconds.
    spawned(same_pixels(&path, 1234, 2500), 1, 0, 0);
}

#[test]
fn a_retimed_element_keeps_one_feed_at_any_speed() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    // (speed, source range, timeline end): each pair satisfies ADR-0020's invariant.
    for (speed, (source_start, source_end), end) in [
        ("2", (0, 3200), 1600),
        ("2.5", (100, 3100), 1200),
        ("0.5", (40, 1240), 2400),
        ("0.645", (0, 1290), 2000),
        // Slow enough that consecutive frames share an offset: the prediction must be asked
        // before the hold, or the feed falls a frame behind and reopens.
        ("0.02", (400, 480), 4000),
    ] {
        let path = project(
            &dir,
            25,
            &track(
                "a",
                0,
                &[video(
                    "retimed",
                    &clip,
                    (0, end),
                    (source_start, source_end),
                    (0, 64, 48),
                    &format!(r#""speed":{speed},"#),
                )],
            ),
        );
        // Measured from the element's own origin, a feed's offsets are the render's at every
        // speed: no rounding disagreement, so no reopen (ADR-0141).
        let counts = same_pixels(&path, 0, end);
        assert_eq!(
            (counts.opened, counts.reopened, counts.frame_at),
            (1, 0, 0),
            "speed {speed}"
        );
        // And from partway through, where a feed measured from its own first frame would
        // round differently from the render.
        let counts = same_pixels(&path, 280, end);
        assert_eq!(
            (counts.opened, counts.reopened, counts.frame_at),
            (1, 0, 0),
            "speed {speed} from 280 ms"
        );
    }
}

#[test]
fn an_off_grid_rate_over_a_late_starting_source_paints_the_same_frames() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // Frames from 42 ms, like this repository's reference MP4.
    let clip = source(
        &dir,
        "late.mp4",
        "64x48",
        &[
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-output_ts_offset",
            "0.042",
        ],
    )
    .expect("libx264");
    for fps in [24, 30, 60] {
        let path = project(
            &dir,
            fps,
            &track(
                "a",
                0,
                &[video("late", &clip, (0, 3000), (0, 3000), (0, 64, 48), "")],
            ),
        );
        spawned(same_pixels(&path, 0, 3000), 1, 0, 0);
    }
}

#[test]
fn two_elements_on_one_source_at_different_offsets_get_a_feed_each() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    let path = project(
        &dir,
        25,
        &[
            track(
                "a",
                0,
                &[video("early", &clip, (0, 2000), (0, 2000), (0, 64, 48), "")],
            ),
            track(
                "b",
                1,
                &[video(
                    "later",
                    &clip,
                    (0, 2000),
                    (1100, 3100),
                    (80, 64, 48),
                    "",
                )],
            ),
        ]
        .join(","),
    );
    spawned(same_pixels(&path, 0, 2000), 2, 0, 0);
}

#[test]
fn a_vp9_source_with_alpha_feeds_through_libvpx_and_keeps_its_alpha() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // The one decoder path the gate check did not cover (ADR-0089): VP9 whose alpha is a side
    // stream, which only `libvpx-vp9` surfaces.
    let Some(clip) = source(
        &dir,
        "alpha.webm",
        "64x48",
        &[
            "-vf",
            "format=yuva420p,geq=lum='p(X,Y)':cb='p(X,Y)':cr='p(X,Y)':a='if(lt(X,32),255,64)'",
            "-c:v",
            "libvpx-vp9",
            "-pix_fmt",
            "yuva420p",
            "-auto-alt-ref",
            "0",
        ],
    ) else {
        eprintln!("skipped: this ffmpeg has no libvpx-vp9 encoder");
        return;
    };
    let path = project(
        &dir,
        25,
        &track(
            "a",
            0,
            &[video("keyed", &clip, (0, 2000), (0, 2000), (0, 64, 48), "")],
        ),
    );
    spawned(same_pixels(&path, 0, 2000), 1, 0, 0);
}

// ---------------------------------------------------------------------------------------
// The feed budget (#636 §7, ADR-0141). Each case shrinks the budget for this thread only.
// ---------------------------------------------------------------------------------------

const MIB: u64 = 1024 * 1024;

/// One feed's estimate for a 64x48 source painted at 64x48.
fn small_feed() -> u64 {
    supply::feed_bytes(Some((64, 48)), (64, 48))
}

/// Two 64x48 elements on two tracks, `first` painted before `second`.
fn two(dir: &Path, clip: &Path, first: (i64, i64), second: (i64, i64)) -> PathBuf {
    project(
        dir,
        25,
        &[
            track(
                "a",
                0,
                &[video(
                    "first",
                    clip,
                    first,
                    (0, first.1 - first.0),
                    (0, 64, 48),
                    "",
                )],
            ),
            track(
                "b",
                1,
                &[video(
                    "second",
                    clip,
                    second,
                    (0, second.1 - second.0),
                    (80, 64, 48),
                    "",
                )],
            ),
        ]
        .join(","),
    )
}

#[test]
fn past_the_budget_an_element_is_painted_per_frame_with_the_same_pixels() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    let path = two(&dir, &clip, (0, 2000), (0, 2000));
    let _budget = supply::override_budget(small_feed() + MIB);

    // 1. Room for one feed: the mixed span's rasters equal an all-per-frame span's.
    let counts = same_pixels(&path, 0, 2000);
    // 2. Exactly one feed, plus one `frame_at` per frame for the second element.
    spawned(counts, 1, 0, 50);
    // 4. The second element is the one named.
    let (feeds, _) = painted(&path, 0, 2000, Supplying::Feeds);
    assert_eq!(feeds.decoded_per_frame, ["second"]);
    assert_eq!(
        feeds.painted,
        ["first", "second"],
        "and it is still painted"
    );
}

#[test]
fn an_element_waiting_for_room_is_promoted_once_the_holder_leaves() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    // `first` holds the only feed until 800 ms (frame 20 is its first frame absent);
    // `second` is painted from 0 to 2000 ms.
    let path = two(&dir, &clip, (0, 800), (0, 2000));
    let _budget = supply::override_budget(small_feed() + MIB);
    // 3 and 6. Frame 20 is the first `first` is not asked at, so `begin_frame(21)` closes
    // its feed before anything asks at frame 21 — and `second` gets a feed at that same
    // frame. It was decoded per frame for frames 0..=20: 21 `frame_at`s.
    spawned(same_pixels(&path, 0, 2000), 2, 0, 21);
}

#[test]
fn the_estimate_is_bytes_so_a_large_source_does_not_take_room_a_small_one_fits() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let small = h264(&dir, "small.mp4", "64x48");
    let large = h264(&dir, "large.mp4", "640x480");
    // 5. The large source asks first, and does not fit; the small one, later in paint order,
    // does. A count of feeds would have admitted the first asker.
    let budget = small_feed() + MIB;
    assert!(supply::feed_bytes(Some((640, 480)), (64, 48)) > budget);
    let path = project(
        &dir,
        25,
        &[
            track(
                "a",
                0,
                &[video(
                    "large",
                    &large,
                    (0, 1000),
                    (0, 1000),
                    (0, 64, 48),
                    "",
                )],
            ),
            track(
                "b",
                1,
                &[video(
                    "small",
                    &small,
                    (0, 1000),
                    (0, 1000),
                    (80, 64, 48),
                    "",
                )],
            ),
        ]
        .join(","),
    );
    let _budget = supply::override_budget(budget);
    spawned(same_pixels(&path, 0, 1000), 1, 0, 25);
    let (feeds, _) = painted(&path, 0, 1000, Supplying::Feeds);
    assert_eq!(feeds.decoded_per_frame, ["large"]);
}

#[test]
fn a_holder_that_loops_keeps_its_slot_while_another_element_waits() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    // 7. `looped` wraps at 600, 1200 and 1800 ms; `waiting` is painted throughout and never
    // gets the room, because a reopen keeps its slot.
    let path = project(
        &dir,
        25,
        &[
            track(
                "a",
                0,
                &[video(
                    "looped",
                    &clip,
                    (0, 2000),
                    (0, 600),
                    (0, 64, 48),
                    r#""overrun":"loop","#,
                )],
            ),
            track(
                "b",
                1,
                &[video(
                    "waiting",
                    &clip,
                    (0, 2000),
                    (0, 2000),
                    (80, 64, 48),
                    "",
                )],
            ),
        ]
        .join(","),
    );
    let _budget = supply::override_budget(small_feed() + MIB);
    spawned(same_pixels(&path, 0, 2000), 1, 3, 50);
}

#[test]
fn the_render_answer_always_carries_decoded_per_frame() {
    if !has_ffprobe() {
        return;
    }
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::render::{Ask, render};

    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    let path = two(&dir, &clip, (0, 1000), (0, 1000));
    let decoded_per_frame = |path: &Path| {
        let answer = render(path, &Ask::default(), &mut |_| {});
        let json = answer.to_json();
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::Ok,
            "{}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
        json["render"]["decoded_per_frame"].clone()
    };
    // 4. `[]` when the budget is not hit — present, never omitted.
    assert_eq!(decoded_per_frame(&path), serde_json::json!([]));
    let _budget = supply::override_budget(small_feed() + MIB);
    assert_eq!(decoded_per_frame(&path), serde_json::json!(["second"]));
}

#[test]
fn a_feed_opened_partway_is_the_run_the_render_paints_from_there() {
    if !has_ffprobe() {
        return;
    }
    // `frames_at` directly, on its own arithmetic: frame `k` of a run anchored at an element's
    // origin and opened at timeline frame `first` is `frame_at` at `offset_at(first + k)`.
    let dir = tempdir(line!());
    let clip = h264(&dir, "clip.mp4", "64x48");
    let source = clip.to_string_lossy();
    for (fps, speed, origin, first) in [
        (30, (1, 1), (100, 120), 17),
        (25, (1, 2), (0, 40), 9),
        (24, (645, 1000), (37, 0), 11),
        (60, (5, 2), (0, 100), 7),
    ] {
        let pace = decode::Pace { fps, speed };
        let origin = decode::Origin {
            timeline_ms: origin.0,
            source_ms: origin.1,
        };
        let mut run = decode::frames_at(
            &ffmpeg(),
            &source,
            Default::default(),
            origin,
            first,
            pace,
            32,
            24,
        )
        .expect("a run");
        for k in 0..12 {
            let offset = decode::offset_at(origin, pace, first + k);
            let want = decode::frame_at(&ffmpeg(), &source, Default::default(), offset, 32, 24)
                .expect("a frame");
            let got = run
                .next_frame()
                .expect("a frame or an end")
                .expect("a frame");
            assert!(
                want == got,
                "fps {fps}, speed {speed:?}, frame {} at {offset} ms",
                first + k
            );
        }
    }
}
