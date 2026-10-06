//! `render` on K painters over chunks of C frames gives the same frames, the same report
//! and the same file as one painter (#653, #627 §8).
//!
//! The primary oracle is a hash of every frame at the encoder's input
//! (`render::tap_frames`). The integration oracle is the MP4 itself, byte for byte: libx264's
//! thread count is pinned (ADR-0143), and x264 is deterministic for one thread count. The
//! report must be equal too, all but the wall clock and the `painting` block that says how
//! the frames were painted.
//!
//! Every render here runs on a thread of its own under a timeout, so a painter that hangs
//! fails the test instead of the suite. K, C and W are forced through the `#[doc(hidden)]`
//! override, which is per thread, so it is set on that thread.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use montagent_core::media::tools;
use montagent_core::report::ExitCode;
use montagent_core::verbs::preview;
use montagent_core::verbs::render::{
    Ask, Cancel, Forced, Progress, fail_frames, force_painting, render, render_cancellable,
    tap_frames,
};
use montagent_render::canvas::bound_filter_layers;
use serde_json::{Value, json};

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// How long one small render may take before it counts as hung.
const HUNG: Duration = Duration::from_secs(60);

fn chunks(painters: usize, chunk: u64) -> Forced {
    Forced::Chunks {
        painters,
        chunk,
        window_bytes: None,
    }
}

/// What one render handed the encoder and answered.
struct Rendered {
    /// One hash per frame pushed, in order.
    frames: Vec<u64>,
    answer: Value,
    exit: ExitCode,
    mp4: Option<Vec<u8>>,
}

impl Rendered {
    /// The answer, less what may differ between two renders of the same frames.
    fn comparable(&self) -> Value {
        let mut answer = self.answer.clone();
        // The probe sidecar's state: the first render of a project misses, the next hits.
        answer
            .as_object_mut()
            .map(|answer| answer.remove("cache_misses"));
        if let Some(render) = answer.get_mut("render").and_then(Value::as_object_mut) {
            for volatile in ["wall_ms", "realtime", "painting"] {
                render.remove(volatile);
            }
        }
        answer
    }
}

/// `f` on a thread of its own, failing the test if it takes longer than [`HUNG`].
fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(HUNG)
        .expect("the render finished inside the timeout rather than hanging")
}

/// Render the project at `path` with `forced`, and `faults` injected.
fn rendered_with(path: &Path, forced: Forced, faults: &[(i64, Duration)]) -> Rendered {
    let path = path.to_path_buf();
    let faults = faults.to_vec();
    within(move || {
        let _forced = force_painting(forced);
        let _faults = fail_frames(&faults);
        let tap = tap_frames();
        let answer = render(&path, &Ask::default(), &mut |_: Progress| {});
        let mp4 = answer
            .video()
            .map(|video| std::fs::read(&video.path).expect("the published file"));
        Rendered {
            frames: tap.hashes(),
            answer: answer.to_json(),
            exit: answer.report().exit_code(),
            mp4,
        }
    })
}

fn rendered(path: &Path, forced: Forced) -> Rendered {
    rendered_with(path, forced, &[])
}

#[track_caller]
fn assert_same(one: &Rendered, other: &Rendered, what: &str) {
    assert_eq!(one.exit, ExitCode::Ok, "{}", one.answer);
    assert_eq!(other.exit, ExitCode::Ok, "{what}: {}", other.answer);
    assert_eq!(
        one.frames.len(),
        other.frames.len(),
        "{what}: frames pushed"
    );
    let differ: Vec<usize> = (0..one.frames.len())
        .filter(|&i| one.frames[i] != other.frames[i])
        .collect();
    assert!(differ.is_empty(), "{what}: frames {differ:?} differ");
    assert_eq!(one.comparable(), other.comparable(), "{what}: the answer");
    assert!(
        one.mp4 == other.mp4,
        "{what}: the MP4s differ while every frame at the encoder's input is equal"
    );
}

/// The trailer's directory, for its title face and a photo.
fn trailer() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/benchmark/spy-trailer")
}

/// The CI guard's project: a `blur` and a zero-offset `shadow` glow over text, shapes and a
/// still, on a 320x180 frame at 30 fps for 1100 ms — 33 frames, so C=2 leaves a ragged last
/// chunk — with every element moving across the whole span, so each chunk boundary falls
/// mid-animation. Two text elements share the face and the still is painted twice, so the
/// `fonts` and `sources` records see a file used again by a painter new to it.
fn glow_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let moving =
        |from: i64, to: i64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let blur = json!({"name": "blur", "radius": 6});
    let glow = json!({"name": "shadow", "dx": 0, "dy": 0, "radius": 12, "color": "#FF9F2E", "opacity": 0.8});
    let elements = [
        json!({"id": "boat", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 1100,
               "x": moving(150, 170), "y": 90, "origin": "center", "width": 320, "height": 181,
               "fit": "cover", "effects": [blur]}),
        json!({"id": "boat-inset", "type": "image", "source": "img/boat.jpg", "start": 400, "end": 1100,
               "x": moving(40, 60), "y": 40, "origin": "center", "width": 64, "height": 36,
               "fit": "cover", "effects": [glow]}),
        json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 40, "color": "#E3C067",
               "align": "center", "runs": [{"text": "SPY"}], "width": 200, "height": 60,
               "start": 0, "end": 1100, "x": moving(130, 190), "y": 90, "origin": "center",
               "effects": [glow]}),
        json!({"id": "subtitle", "type": "text", "font": "cinzel-bold", "size": 16, "color": "#FFFFFF",
               "align": "center", "runs": [{"text": "SILENT PROTOCOL"}], "width": 200, "height": 24,
               "start": 500, "end": 1100, "x": 160, "y": moving(150, 140), "origin": "center",
               "effects": [blur]}),
        json!({"id": "bar", "type": "rect", "start": 0, "end": 1100, "x": moving(10, 250), "y": 160,
               "origin": "top-left", "width": 60, "height": 8, "fill": "#F2F2F2",
               "effects": [{"name": "blur", "radius": 3}]}),
        json!({"id": "dot", "type": "ellipse", "start": 0, "end": 1100, "x": moving(290, 230),
               "y": 30, "origin": "center", "width": 16, "height": 16, "fill": "#FFFFFF",
               "effects": [glow]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/glow.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "glow.montagent.json",
        &canonical(&project.to_string()),
    )
}

/// The decoded output's per-frame hashes, as `ffmpeg -f framemd5` writes them.
fn framemd5(mp4: &[u8], dir: &Path) -> Vec<String> {
    let file = dir.join("framemd5.mp4");
    std::fs::write(&file, mp4).expect("write the MP4 to hash");
    let out = Command::new(tools::resolve().expect("an ffmpeg").ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(&file)
        .args(["-map", "0:v", "-f", "framemd5", "-"])
        .output()
        .expect("ffmpeg runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

#[test]
fn three_painters_over_two_frame_chunks_give_the_blur_and_glow_framemd5_of_one() {
    if !has_ffprobe() {
        return;
    }
    let path = glow_project(line!());
    let one = rendered(&path, chunks(1, 2));
    let three = rendered(&path, chunks(3, 2));
    assert_eq!(one.frames.len(), 33, "a ragged last chunk at C=2");
    assert_eq!(three.answer["render"]["painting"]["painters"], 3);
    assert_eq!(three.answer["render"]["painting"]["chunk"], 2);
    assert_same(&one, &three, "K=3, C=2");

    let scratch = path.parent().expect("the project's directory");
    let (one, three) = (
        framemd5(one.mp4.as_deref().expect("a file"), scratch),
        framemd5(three.mp4.as_deref().expect("a file"), scratch),
    );
    assert_eq!(one.len(), 33);
    assert_eq!(one, three, "the decoded framemd5");
}

#[test]
fn any_k_and_c_paint_what_one_painter_on_the_verbs_thread_paints() {
    if !has_ffprobe() {
        return;
    }
    let path = glow_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.answer["render"]["painting"]["painters"], 1);
    let fonts = &sequential.answer["render"]["fonts"];
    let sources = &sequential.answer["render"]["sources"];
    assert_eq!(fonts.as_array().map(Vec::len), Some(1), "{fonts}");
    assert_eq!(sources.as_array().map(Vec::len), Some(1), "{sources}");
    // K = 1 with chunking on; K = 2, 3 and 4; C = 1; and C that does not divide 33.
    for (painters, chunk) in [(1, 2), (2, 1), (3, 2), (4, 3), (2, 7), (4, 1), (3, 40)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
}

/// H.264 frames of a moving test pattern at 25 fps, so a frame off by one cannot pass.
fn clip(dir: &Path) -> PathBuf {
    let path = dir.join("clip.mp4");
    let ok = Command::new(tools::resolve().expect("an ffmpeg").ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi"])
        .args(["-i", "testsrc2=s=64x48:r=25", "-frames:v", "82"])
        .args([
            "-c:v", "libx264", "-pix_fmt", "yuv420p", "-g", "50", "-bf", "2",
        ])
        .arg(&path)
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(ok, "libx264 is ADR-0115's floor");
    path
}

#[test]
fn chunk_boundaries_inside_a_feeds_seek_and_its_loop_paint_the_same_frames() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    // Wraps at 1000 and 2000 ms, mid-chunk at C=4; every chunk start is a seek.
    let looped = json!({"id": "looped", "type": "video", "start": 0, "end": 2600, "overrun": "loop",
        "source": clip, "source_start": 200, "source_end": 1200, "x": 0, "y": 0,
        "origin": "top-left", "width": 64, "height": 48, "fit": "literal", "volume": 0});
    let played = json!({"id": "played", "type": "video", "start": 300, "end": 2600,
        "source": clip, "source_start": 0, "source_end": 2300, "x": 80, "y": 40,
        "origin": "top-left", "width": 64, "height": 48, "fit": "literal", "volume": 0});
    let project = json!({
        "frame": {"width": 160, "height": 120}, "fps": 25, "background": "#000000",
        "duration": 2600, "output": "out/feeds.mp4",
        "tracks": [{"name": "a", "layer": 0, "elements": [looped]},
                   {"name": "b", "layer": 1, "elements": [played]}],
    });
    let path = write_project(
        &dir,
        "feeds.montagent.json",
        &canonical(&project.to_string()),
    );

    let sequential = rendered(&path, Forced::OnePainter);
    for (painters, chunk) in [(3, 4), (2, 7)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
}

/// ADR-0147 §4's fixture, one `blend` mode on every element above the ground: sub-pixel
/// motion, rotation, non-uniform scale, keyed `opacity`, `blur` with `shadow`, a `mask`, a
/// `clip`, stroked text, an image and a video, 18 frames at 30 fps.
fn blended_project(mode: &str, line: u32) -> PathBuf {
    let dir = tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    // Over 600 ms an `x` that travels 7 px moves a fraction of a pixel per frame.
    let keyed = |from: Value, to: Value| json!([{"t": 0, "v": from}, {"t": 600, "v": to, "ease": "linear"}]);
    let glow = json!({"name": "shadow", "dx": 3, "dy": 2, "radius": 6, "color": "#FF9F2E", "opacity": 0.8});
    let elements = [
        json!({"id": "ground", "type": "rect", "start": 0, "end": 600, "x": 0, "y": 0,
               "origin": "top-left", "width": 160, "height": 90, "fill": "#5A6E80"}),
        json!({"id": "photo", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 600,
               "x": keyed(json!(80), json!(87)), "y": 45, "origin": "center", "width": 160,
               "height": 90, "fit": "literal", "clip": [8, 6, 144, 78], "blend": mode}),
        json!({"id": "footage", "type": "video", "start": 0, "end": 600, "source": clip,
               "source_start": 0, "source_end": 600, "x": 40, "y": 40, "origin": "center",
               "width": 64, "height": 48, "fit": "literal", "volume": 0,
               "rotation": keyed(json!(0.0), json!(17.5)),
               "opacity": keyed(json!(1.0), json!(0.35)), "blend": mode}),
        json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 28, "color": "#E3C067",
               "stroke": "#A02010", "stroke_width": 2, "align": "center",
               "runs": [{"text": "SPY"}], "width": 120, "height": 40, "start": 0, "end": 600,
               "x": keyed(json!(76), json!(83)), "y": 45, "origin": "center",
               "scale": [1.3, 0.8], "effects": [{"name": "blur", "radius": 1.5}, glow],
               "blend": mode}),
        json!({"id": "lens", "type": "ellipse", "start": 0, "end": 600, "x": 120, "y": 30,
               "origin": "center", "width": 40, "height": 30, "fill": "#FFFFFF",
               "rotation": 20, "scale": keyed(json!([1.5, 0.7]), json!([1.1, 0.9])),
               "effects": [{"name": "mask", "shape": "circle"}, {"name": "blur", "radius": 3}],
               "blend": mode}),
        json!({"id": "bar", "type": "rect", "start": 0, "end": 600,
               "x": keyed(json!(10), json!(17)), "y": 70, "origin": "top-left", "width": 60,
               "height": 10, "fill": "#20C0F0", "rotation": -8, "opacity": 0.7,
               "effects": [glow], "blend": mode}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 160, "height": 90}, "fps": 30, "background": "#101418",
        "duration": 600, "output": "out/blend.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "blend.montagent.json",
        &canonical(&project.to_string()),
    )
}

/// [`rendered`], with the `blur`/`shadow` bounds hint (#652) off on the asking thread,
/// which every painter inherits.
fn rendered_unbounded(path: &Path, forced: Forced) -> Rendered {
    let path = path.to_path_buf();
    within(move || {
        bound_filter_layers(false);
        let _forced = force_painting(forced);
        let tap = tap_frames();
        let answer = render(&path, &Ask::default(), &mut |_: Progress| {});
        let mp4 = answer
            .video()
            .map(|video| std::fs::read(&video.path).expect("the published file"));
        Rendered {
            frames: tap.hashes(),
            answer: answer.to_json(),
            exit: answer.report().exit_code(),
            mp4,
        }
    })
}

#[test]
fn every_blend_mode_paints_the_same_frames_on_any_number_of_painters() {
    // ADR-0147 §4, the gating test: a mode that fails it is withdrawn, not excused.
    if !has_ffprobe() {
        return;
    }
    for mode in ["normal", "multiply", "screen", "overlay", "add"] {
        let path = blended_project(mode, line!() * 10 + mode.len() as u32);
        let sequential = rendered(&path, Forced::OnePainter);
        assert_eq!(sequential.frames.len(), 18, "{mode}");
        for (painters, chunk) in [(3, 2), (2, 5), (4, 1)] {
            let chunked = rendered(&path, chunks(painters, chunk));
            assert_same(
                &sequential,
                &chunked,
                &format!("{mode}: K={painters}, C={chunk}"),
            );
        }
        for forced in [Forced::OnePainter, chunks(3, 2)] {
            let unbounded = rendered_unbounded(&path, forced);
            assert_same(&sequential, &unbounded, &format!("{mode}: the hint off"));
        }
    }
}

#[test]
fn a_video_element_keeps_the_render_on_one_painter_unless_forced() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    let element = json!({"id": "v", "type": "video", "start": 0, "end": 400, "source": clip,
        "source_start": 0, "source_end": 400, "x": 0, "y": 0, "origin": "top-left",
        "width": 64, "height": 48, "fit": "literal", "volume": 0});
    let project = json!({
        "frame": {"width": 160, "height": 120}, "fps": 25, "background": "#000000",
        "duration": 400, "output": "out/v.mp4",
        "tracks": [{"name": "a", "layer": 0, "elements": [element]}],
    });
    let path = write_project(&dir, "v.montagent.json", &canonical(&project.to_string()));
    let answer = within(move || render(&path, &Ask::default(), &mut |_: Progress| {}).to_json());
    assert_eq!(
        answer["render"]["painting"],
        json!({"painters": 1, "chunk": 10, "window": 1, "window_floor": false})
    );
}

#[test]
fn the_earliest_failure_is_reported_when_a_later_chunk_fails_first() {
    if !has_ffprobe() {
        return;
    }
    let path = glow_project(line!());
    // Frame 2 is chunk 1's; frame 9 is chunk 4's, which a painter reaches once chunks 0
    // and 2 are done — long before frame 2's failure lands, 1.5 s after it was reached.
    let failed = rendered_with(
        &path,
        chunks(3, 2),
        &[(2, Duration::from_millis(1500)), (9, Duration::ZERO)],
    );
    assert_eq!(failed.exit, ExitCode::Internal, "{}", failed.answer);
    assert!(
        failed.answer["render"].is_null(),
        "a failed render published"
    );
    let said = failed.answer.to_string();
    assert!(said.contains("frame 2: an injected failure"), "{said}");
    assert!(!said.contains("frame 9"), "{said}");
    assert!(
        failed.frames.len() <= 2,
        "frames past the failure reached the encoder"
    );
    let out = path.parent().expect("dir").join("out");
    let left: Vec<_> = std::fs::read_dir(&out)
        .map(|entries| entries.filter_map(Result::ok).collect())
        .unwrap_or_default();
    assert!(left.is_empty(), "a failed render left {left:?}");

    // The other way round: frame 2's failure lands first, and frame 9's, already being
    // worked on, lands after it. The later arrival must not displace the earlier frame.
    let failed = rendered_with(
        &path,
        chunks(3, 2),
        &[
            (2, Duration::from_millis(300)),
            (9, Duration::from_millis(1500)),
        ],
    );
    let said = failed.answer.to_string();
    assert!(said.contains("frame 2: an injected failure"), "{said}");
    assert!(!said.contains("frame 9"), "{said}");

    // A failure alone at the last frame is the one reported, after every frame before it.
    let last = rendered_with(&path, chunks(3, 2), &[(32, Duration::ZERO)]);
    assert!(
        last.answer
            .to_string()
            .contains("frame 32: an injected failure"),
        "{}",
        last.answer
    );
    assert_eq!(last.frames.len(), 32);
}

#[test]
fn a_window_budget_below_k_times_c_still_renders_on_the_floor_and_says_so() {
    if !has_ffprobe() {
        return;
    }
    let path = glow_project(line!());
    let floored = rendered(
        &path,
        Forced::Chunks {
            painters: 3,
            chunk: 4,
            window_bytes: Some(1),
        },
    );
    assert_eq!(
        floored.answer["render"]["painting"],
        json!({"painters": 3, "chunk": 4, "window": 12, "window_floor": true})
    );
    assert_same(&rendered(&path, Forced::OnePainter), &floored, "the floor");
    let text = montagent_core::text::render(&floored.answer, Default::default())
        .expect("the answer renders as text");
    assert!(
        text.contains("so painting led the encoder by up to the floor of 12 frames"),
        "{text}"
    );
}

#[test]
fn a_chunked_render_cancelled_mid_encode_publishes_nothing() {
    if !has_ffprobe() {
        return;
    }
    let path = glow_project(line!());
    let (answer, reports) = {
        let path = path.clone();
        within(move || {
            let _forced = force_painting(chunks(3, 2));
            let cancel = Cancel::new();
            let mut reports = Vec::new();
            let answer = render_cancellable(
                &path,
                &Ask::default(),
                &mut |p: Progress| {
                    reports.push(p.done);
                    if p.done > 0 {
                        cancel.cancel();
                    }
                },
                Some(&cancel),
            );
            (answer.to_json(), reports)
        })
    };
    assert!(answer["render"].is_null(), "a cancelled render published");
    assert!(answer.to_string().contains("E-CANCELLED"), "{answer}");
    // Progress counts frames the encoder took, in tenths, and stopped at the first after 0.
    assert_eq!(reports.len(), 2, "{reports:?}");
    assert!(reports[1] > 0 && reports[1] < 33, "{reports:?}");
    let out = path.parent().expect("dir").join("out");
    let left: Vec<_> = std::fs::read_dir(&out)
        .map(|entries| entries.filter_map(Result::ok).collect())
        .unwrap_or_default();
    assert!(left.is_empty(), "a cancelled render left {left:?}");
}

#[test]
fn preview_paints_the_same_frames_on_k_painters() {
    if !has_ffprobe() {
        return;
    }
    let path = glow_project(line!());
    let previewed = |forced: Forced| {
        let path = path.clone();
        within(move || {
            let _forced = force_painting(forced);
            let tap = tap_frames();
            let answer = preview::preview(&path, &preview::Ask::default(), &mut |_| {});
            let json = answer.to_json();
            assert_eq!(answer.report().exit_code(), ExitCode::Ok, "{json}");
            (tap.hashes(), json["preview"]["painting"].clone())
        })
    };
    let (one, painting) = previewed(Forced::OnePainter);
    assert_eq!(painting["painters"], 1, "{painting}");
    let (three, painting) = previewed(chunks(3, 2));
    assert_eq!(painting["painters"], 3, "{painting}");
    assert!(!one.is_empty());
    assert_eq!(one, three, "preview's frames at the encoder's input");
}

/// Inverted masks (ADR-0152) with a blur, a glow and a ring, each moving with sub-pixel
/// steps, on a 320x180 frame at 30 fps for 1100 ms (33 frames).
fn inverted_mask_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let moving =
        |from: i64, to: i64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let inverted = |shape: &str| json!({"name": "mask", "shape": shape, "invert": true});
    let glow = json!({"name": "shadow", "dx": 4, "dy": 3, "radius": 12, "color": "#FF9F2E", "opacity": 0.8});
    let elements = [
        json!({"id": "disc", "type": "rect", "start": 0, "end": 1100, "x": moving(90, 130),
               "y": 90, "origin": "center", "width": 120, "height": 120, "fill": "#F2F2F2",
               "rotation": moving(0, 37), "scale": [1.3, 0.8],
               "effects": [inverted("circle"), {"name": "blur", "radius": 5}]}),
        json!({"id": "pane", "type": "rect", "start": 0, "end": 1100, "x": 230, "y": moving(60, 110),
               "origin": "center", "width": 90, "height": 70, "fill": "#3BA0FF",
               "effects": [{"name": "mask", "shape": "rect", "x": 10, "y": 10, "width": 50,
                            "height": 40, "radius": 8, "invert": true}, glow]}),
        json!({"id": "ring", "type": "ellipse", "start": 200, "end": 1100, "x": moving(160, 200),
               "y": 50, "origin": "center", "width": 80, "height": 80, "fill": "#FF3B30",
               "effects": [{"name": "mask", "shape": "ellipse"},
                           {"name": "mask", "shape": "ellipse", "x": 25, "y": 25, "width": 30,
                            "height": 30, "invert": true},
                           {"name": "blur", "radius": 3}]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/inverted.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "inverted.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn inverted_masks_with_blur_glow_and_a_ring_give_the_framemd5_of_one_painter_across_chunks() {
    if !has_ffprobe() {
        return;
    }
    let path = inverted_mask_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.frames.len(), 33);
    for (painters, chunk) in [(2, 1), (4, 3)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }

    let scratch = path.parent().expect("the project's directory");
    let three = rendered(&path, chunks(3, 2));
    assert_same(&sequential, &three, "K=3, C=2");
    let (one, three) = (
        framemd5(sequential.mp4.as_deref().expect("a file"), scratch),
        framemd5(three.mp4.as_deref().expect("a file"), scratch),
    );
    assert_eq!(one.len(), 33);
    assert_eq!(one, three, "the decoded framemd5");
}

/// The stagger's gating fixture (ADR-0144, ADR-0151's consequences): a letter stagger with
/// unit rotation and scale on a stroked, fading title under element rotation and `blur`,
/// with one letter singled out; a word stagger in `reverse`; and an Arabic letter stagger
/// whose joined pieces move as one. 33 frames at 30 fps, every cascade crossing the chunk
/// boundaries.
fn stagger_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let naskh = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/letter-spacing/fonts/NotoNaskhArabic-Regular.ttf");
    for (from, to) in [
        (
            trailer().join("fonts/Cinzel-Bold.ttf"),
            "fonts/Cinzel-Bold.ttf",
        ),
        (naskh, "fonts/Naskh.ttf"),
    ] {
        let to = dir.join(to);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("font dir");
        std::fs::copy(from, &to).expect("copy the fixture font");
    }
    let ramp = |from: Value, to: Value, until: i64| json!([{"t": 0, "v": from}, {"t": until, "v": to, "ease": "ease-out"}]);
    let elements = [
        json!({"id": "letters", "type": "text", "font": "titles", "size": 40, "color": "#E3C067",
               "stroke": "#7A1F1F", "stroke_width": 2,
               "align": "center", "width": 260, "height": 60, "start": 0, "end": 1100,
               "x": 160, "y": 60, "origin": "center", "rotation": 10.0,
               "effects": [{"name": "blur", "radius": 2}],
               "runs": [{"text": "SPY "}, {"text": "G", "unit": {"delay": 150,
                         "y": [{"t": 300, "v": -30}, {"t": 900, "v": 0, "ease": "ease-in-out"}]}},
                        {"text": "AME"}],
               "units": {"by": "letter", "every": 70, "origin": "bottom-center",
                         "y": ramp(json!(24), json!(0), 500),
                         "rotation": ramp(json!(-40.0), json!(0.0), 500),
                         "scale": ramp(json!([0.3, 0.3]), json!([1.0, 1.0]), 500),
                         "opacity": ramp(json!(0.0), json!(1.0), 400)},
               "caption": false}),
        json!({"id": "words", "type": "text", "font": "titles", "size": 18, "color": "#FFFFFF",
               "align": "center", "width": 300, "height": 30, "start": 0, "end": 1100,
               "x": 160, "y": 120, "origin": "center", "effects": [{"name": "blur", "radius": 1}],
               "runs": [{"text": "SILENT PROTOCOL NOW"}],
               "units": {"by": "word", "every": 250, "order": "reverse",
                         "x": ramp(json!(-40), json!(0), 400),
                         "opacity": ramp(json!(0.0), json!(1.0), 400)},
               "caption": false}),
        json!({"id": "arabic", "type": "text", "font": "titles", "size": 28, "color": "#9FE3FF",
               "align": "center", "width": 200, "height": 44, "start": 0, "end": 1100,
               "x": 160, "y": 155, "origin": "center",
               "runs": [{"text": "السلام عليكم"}],
               "units": {"by": "letter", "every": 60,
                         "scale": ramp(json!([0.2, 0.2]), json!([1.0, 1.0]), 450),
                         "opacity": ramp(json!(0.0), json!(1.0), 450)},
               "caption": false}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/stagger.mp4",
        "fonts": {"titles": [{"file": "fonts/Cinzel-Bold.ttf"}, {"file": "fonts/Naskh.ttf"}]},
        "fontVendor": {
            "fonts/Cinzel-Bold.ttf": {
                "licence": "OFL-1.1",
                "source": "google/fonts ofl/cinzel, instanced wght=700",
                "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"},
            "fonts/Naskh.ttf": {
                "licence": "OFL-1.1",
                "source": "notofonts/arabic NotoNaskhArabic-v2.019",
                "sha256": "eb5cde7fecba8c6a481039257fe02d5fd69b7b0e36f8afc56ee22f4b1d7e8c21"},
        },
        "tracks": tracks,
    });
    write_project(
        &dir,
        "stagger.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn a_staggered_title_crossing_chunk_boundaries_gives_the_framemd5_of_one_painter() {
    if !has_ffprobe() {
        return;
    }
    let path = stagger_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.frames.len(), 33);
    assert_ne!(
        sequential.frames[3], sequential.frames[30],
        "the units move"
    );
    for (painters, chunk) in [(2, 1), (4, 3), (3, 7)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }

    let scratch = path.parent().expect("the project's directory");
    let three = rendered(&path, chunks(3, 2));
    assert_same(&sequential, &three, "K=3, C=2");
    let (one, three) = (
        framemd5(sequential.mp4.as_deref().expect("a file"), scratch),
        framemd5(three.mp4.as_deref().expect("a file"), scratch),
    );
    assert_eq!(one.len(), 33);
    assert_eq!(one, three, "the decoded framemd5");
}
