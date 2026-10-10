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
use common::{canonical, has_ffprobe, mp4_bytes_are_guaranteed, tempdir, write_project};

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
            // ADR-0190: the file's size is its bytes, which x86_64 Windows does not guarantee.
            if !mp4_bytes_are_guaranteed() {
                render.remove("bytes");
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
    // ADR-0190: x86_64 Windows guarantees the frames above, not the file's bytes.
    assert!(
        !mp4_bytes_are_guaranteed() || one.mp4 == other.mp4,
        "{what}: the MP4s differ while every frame at the encoder's input is equal"
    );
}

/// The two files' decoded frames agree (ADR-0190: not asserted where the bytes are not
/// guaranteed, since two encodings of the same frames decode to different pixels).
#[track_caller]
fn assert_same_decoded(one: &Rendered, other: &Rendered, scratch: &Path) {
    if !mp4_bytes_are_guaranteed() {
        return;
    }
    let (one, other) = (
        framemd5(one.mp4.as_deref().expect("a file"), scratch),
        framemd5(other.mp4.as_deref().expect("a file"), scratch),
    );
    assert_eq!(one.len(), 33);
    assert_eq!(one, other, "the decoded framemd5");
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
    assert_same_decoded(&one, &three, scratch);
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

/// Key every parameter of one gradient over 0..600 ms.
fn key_gradient(gradient: &mut Value) {
    let list = |from: Value, to: Value, ease: Value| json!([{"t": 0, "v": from}, {"t": 600, "v": to, "ease": ease}]);
    if gradient["gradient"] == "linear" {
        let angle = gradient["angle"].as_f64().expect("an angle");
        gradient["angle"] = list(json!(angle), json!(angle + 140.0), json!("linear"));
    } else {
        gradient["center"] = list(json!([0.3, 0.25]), json!([0.8, 0.7]), json!("ease-in-out"));
        gradient["radius"] = list(json!(0.4), json!(1.3), json!("linear"));
    }
    let stops = |a: f64, b: f64, c: &str, d: &str| {
        json!([{"offset": 0, "color": c}, {"offset": a, "color": d}, {"offset": b, "color": c},
               {"offset": 1, "color": d}])
    };
    gradient["stops"] = list(
        stops(0.2, 0.6, "#FF3366", "#3366FF00"),
        stops(0.5, 0.6, "#FFCC00", "#20C0F0"),
        json!([0.34, 1.56, 0.64, 1]),
    );
}

/// ADR-0149 §8's gate, carried into the build: gradient paint on every paint field, under
/// `blur` and `shadow`, with sub-pixel motion, rotation and non-uniform scale, 18 frames at
/// 30 fps.
///
/// With `keyed_gradients`, every parameter of every gradient is a keyframe list over the 600
/// ms, the stops on a bezier that overshoots (ADR-0149 §3, §4), so the paint differs on every
/// frame.
fn gradient_project(line: u32, keyed_gradients: bool) -> PathBuf {
    let dir = tempdir(line);
    let to = dir.join("fonts/Cinzel-Bold.ttf");
    std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
    std::fs::copy(trailer().join("fonts/Cinzel-Bold.ttf"), &to).expect("copy the face");
    // Over 600 ms an `x` that travels 7 px moves a fraction of a pixel per frame.
    let keyed = |from: Value, to: Value| json!([{"t": 0, "v": from}, {"t": 600, "v": to, "ease": "linear"}]);
    let stops =
        |from: &str, to: &str| json!([{"offset": 0, "color": from}, {"offset": 1, "color": to}]);
    let linear = |angle: f64, from: &str, to: &str| json!({"gradient": "linear", "angle": angle, "stops": stops(from, to)});
    let radial = |center: [f64; 2], radius: f64, from: &str, to: &str| json!({"gradient": "radial", "center": center, "radius": radius, "stops": stops(from, to)});
    let glow = json!({"name": "shadow", "dx": 3, "dy": 2, "radius": 6, "color": "#FF9F2E", "opacity": 0.8});
    let mut elements = [
        json!({"id": "sky", "type": "rect", "start": 0, "end": 600, "x": 0, "y": 0,
               "origin": "top-left", "width": 160, "height": 90,
               "fill": linear(160.0, "#101830", "#5A6E80")}),
        json!({"id": "card", "type": "rect", "start": 0, "end": 600,
               "x": keyed(json!(60), json!(67)), "y": 40, "origin": "center", "width": 70,
               "height": 36, "radius": 6, "rotation": keyed(json!(0.0), json!(17.5)),
               "scale": [1.3, 0.8], "fill": linear(30.0, "#FF3366", "#3366FF00"),
               "stroke": radial([0.5, 0.5], 1.0, "#FFFFFF", "#000000"), "stroke_width": 3,
               "effects": [{"name": "blur", "radius": 1.5}, glow]}),
        json!({"id": "lens", "type": "ellipse", "start": 0, "end": 600,
               "x": keyed(json!(120), json!(113)), "y": 30, "origin": "center", "width": 40,
               "height": 26, "rotation": 20, "scale": keyed(json!([1.5, 0.7]), json!([1.1, 0.9])),
               "fill": radial([0.3, 0.25], 0.9, "#FFCC00", "#FF330000"),
               "effects": [glow, {"name": "blur", "radius": 3}]}),
        json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 28,
               "color": linear(90.0, "#E3C067", "#20C0F0"),
               "stroke": radial([0.5, 0.5], 1.0, "#A02010", "#102040"), "stroke_width": 2,
               "align": "center", "runs": [{"text": "SPY"}], "width": 120, "height": 40,
               "start": 0, "end": 600, "x": keyed(json!(76), json!(83)), "y": 65,
               "origin": "center", "scale": [1.2, 0.9], "rotation": -6,
               "effects": [{"name": "blur", "radius": 1.0}, glow]}),
    ];
    if keyed_gradients {
        for element in &mut elements {
            for paint in ["fill", "stroke", "color"] {
                if element[paint].is_object() {
                    key_gradient(&mut element[paint]);
                }
            }
        }
    }
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 160, "height": 90}, "fps": 30, "background": "#101418",
        "duration": 600, "output": "out/gradient.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "gradient.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn gradients_paint_the_same_frames_on_any_number_of_painters_with_the_hint_on_or_off() {
    // ADR-0149 §8 and ADR-0144: a gradient kind that fails this is withdrawn, not excused.
    if !has_ffprobe() {
        return;
    }
    // Static paint (slice 1) and every nested parameter keyed (slice 2, #687).
    for keyed in [false, true] {
        let path = gradient_project(line!() + u32::from(keyed), keyed);
        let sequential = rendered(&path, Forced::OnePainter);
        assert_eq!(sequential.frames.len(), 18);
        for (painters, chunk) in [(3, 2), (2, 5), (4, 1)] {
            let chunked = rendered(&path, chunks(painters, chunk));
            assert_same(
                &sequential,
                &chunked,
                &format!("keyed={keyed}, K={painters}, C={chunk}"),
            );
        }
        for forced in [Forced::OnePainter, chunks(3, 2)] {
            let unbounded = rendered_unbounded(&path, forced);
            assert_same(
                &sequential,
                &unbounded,
                &format!("keyed={keyed}, the hint off"),
            );
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

/// #698's gating fixture: every mask shape feathered, plain and inverted, a keyed
/// `feather`, rotation, non-uniform scale, sub-pixel motion, `[mask, blur]`, `[blur, mask]`,
/// a `shadow` and a video, on a 320x180 frame at 30 fps for 1100 ms (33 frames).
fn feathered_mask_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    // Over 1100 ms an `x` that travels 9 px moves a fraction of a pixel per frame.
    let moving = |from: Value, to: Value| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let blur = |radius: f64| json!({"name": "blur", "radius": radius});
    let glow = json!({"name": "shadow", "dx": 4, "dy": 3, "radius": 12, "color": "#FF9F2E", "opacity": 0.8});
    let elements = [
        // `[mask, blur]`: a feathered circle, rotating, squashed and drifting.
        json!({"id": "disc", "type": "rect", "start": 0, "end": 1100, "x": moving(json!(60), json!(69)),
               "y": 60, "origin": "center", "width": 100, "height": 100, "fill": "#F2F2F2",
               "rotation": moving(json!(0), json!(37)), "scale": [1.3, 0.8],
               "effects": [{"name": "mask", "shape": "circle", "feather": 18}, blur(4.0)]}),
        // `[blur, mask]`: an inverted feathered ellipse after the blur.
        json!({"id": "lens", "type": "ellipse", "start": 0, "end": 1100, "x": 160,
               "y": moving(json!(50), json!(58)), "origin": "center", "width": 110, "height": 80,
               "fill": "#3BA0FF", "effects": [blur(3.0),
                   {"name": "mask", "shape": "ellipse", "x": 25, "y": 15, "width": 60,
                    "height": 50, "invert": true, "feather": 14}]}),
        // A feathered rounded rect with a shadow after it under a keyed feather — 0 at the
        // start (the hard path), fractional between the keys — and an inverted rect whose
        // keyed feather falls.
        json!({"id": "pane", "type": "rect", "start": 0, "end": 1100,
               "x": moving(json!(250), json!(257)), "y": 60, "origin": "center", "width": 90,
               "height": 70, "fill": "#FF3B30", "scale": moving(json!([1.0, 1.0]), json!([1.4, 0.9])),
               "effects": [{"name": "mask", "shape": "rect", "x": 10, "y": 10, "width": 70,
                            "height": 50, "radius": 12,
                            "feather": [{"t": 0, "v": 0}, {"t": 600, "v": 25, "ease": "ease-in-out"},
                                        {"t": 1100, "v": 9, "ease": "linear"}]}, glow]}),
        json!({"id": "pane-hole", "type": "rect", "start": 100, "end": 1100,
               "x": moving(json!(250), json!(243)), "y": 140, "origin": "center", "width": 90,
               "height": 60, "fill": "#20C0F0", "rotation": -12,
               "effects": [{"name": "mask", "shape": "rect", "x": 15, "y": 10, "width": 60,
                            "height": 40, "invert": true,
                            "feather": [{"t": 100, "v": 30}, {"t": 1100, "v": 3, "ease": "linear"}]}]}),
        // A feathered vignette on video, and a feathered hole in it.
        json!({"id": "footage", "type": "video", "start": 0, "end": 1100, "source": clip,
               "source_start": 0, "source_end": 1100, "x": moving(json!(80), json!(86)), "y": 135,
               "origin": "center", "width": 128, "height": 72, "fit": "literal", "volume": 0,
               "rotation": 6,
               "effects": [{"name": "mask", "shape": "ellipse", "feather": 20},
                           {"name": "mask", "shape": "circle", "x": 44, "y": 16, "width": 40,
                            "height": 40, "invert": true, "feather": 10}]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/feathered.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "feathered.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn feathered_masks_paint_the_same_frames_on_any_painters_with_the_hint_on_or_off() {
    // #698's gating test (ADR-0152 §4, ADR-0144): a feather that fails it does not ship.
    if !has_ffprobe() {
        return;
    }
    let path = feathered_mask_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    for (painters, chunk) in [(2, 1), (3, 2), (4, 3), (6, 9)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// #676's gating fixture (ADR-0146 §3, ADR-0144): every effect member with its parameters
/// keyed — a keyed `blur` and a keyed `shadow` under the bounds hint, a mask reveal, the
/// colour filters and a keyed `chroma` on video — with sub-pixel motion, on a 320x180 frame
/// at 30 fps for 1100 ms (33 frames).
fn keyed_effects_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    let keyed = |a: Value, b: Value, c: Value| {
        json!([{"t": 0, "v": a}, {"t": 600, "v": b, "ease": "ease-in-out"},
               {"t": 1100, "v": c, "ease": [0.3, 1.4, 0.6, 1.2]}])
    };
    let moving = |from: Value, to: Value| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let elements = [
        // A keyed blur, then a shadow with every parameter keyed.
        json!({"id": "disc", "type": "ellipse", "start": 0, "end": 1100,
               "x": moving(json!(60), json!(69)), "y": 60, "origin": "center", "width": 90,
               "height": 70, "fill": "#F2F2F2", "rotation": moving(json!(0), json!(23)),
               "effects": [{"name": "blur", "radius": keyed(json!(0), json!(9), json!(2))},
                           {"name": "shadow", "dx": keyed(json!(-6), json!(11), json!(3)),
                            "dy": keyed(json!(4), json!(-5), json!(0)),
                            "radius": keyed(json!(0), json!(17), json!(5)),
                            "color": keyed(json!("#FF9F2E"), json!("#3BA0FF80"), json!("#000000")),
                            "opacity": keyed(json!(0.2), json!(1), json!(0.6))}]}),
        // A reveal: a rect mask keyed from width 0, rounded by a keyed radius, under a blur.
        json!({"id": "pane", "type": "rect", "start": 0, "end": 1100,
               "x": moving(json!(160), json!(166)), "y": 60, "origin": "center", "width": 110,
               "height": 70, "fill": "#FF3B30", "scale": [1.2, 0.9],
               "effects": [{"name": "mask", "shape": "rect", "x": keyed(json!(0), json!(6), json!(2)),
                            "y": 0, "width": keyed(json!(0), json!(80), json!(110)),
                            "height": 70, "radius": keyed(json!(0), json!(30), json!(12)),
                            "feather": 4},
                           {"name": "blur", "radius": 2}]}),
        // The colour filters, keyed.
        json!({"id": "graded", "type": "rect", "start": 0, "end": 1100,
               "x": moving(json!(260), json!(255)), "y": 60, "origin": "center", "width": 90,
               "height": 80, "fill": "#20C0F0",
               "effects": [{"name": "tint", "color": keyed(json!("#FF0000"), json!("#00FF00"), json!("#0000FF")),
                            "amount": keyed(json!(0), json!(0.7), json!(0.3))},
                           {"name": "saturation", "amount": keyed(json!(1), json!(0), json!(2))},
                           {"name": "brightness", "amount": keyed(json!(0), json!(0.3), json!(-0.2))},
                           {"name": "contrast", "amount": keyed(json!(0), json!(-0.4), json!(0.5))}]}),
        // A keyed key on footage.
        json!({"id": "footage", "type": "video", "start": 0, "end": 1100, "source": clip,
               "source_start": 0, "source_end": 1100, "x": moving(json!(80), json!(86)), "y": 135,
               "origin": "center", "width": 128, "height": 72, "fit": "literal", "volume": 0,
               "effects": [{"name": "chroma", "color": keyed(json!("#00FF00"), json!("#20C020"), json!("#00CD00")),
                            "tolerance": keyed(json!(0), json!(0.5), json!(0.2)),
                            "softness": keyed(json!(0), json!(0.3), json!(0.1)),
                            "spill": keyed(json!(0), json!(0.6), json!(0.2))}]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/keyed.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "keyed.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn keyed_effect_parameters_paint_the_same_frames_on_any_painters_with_the_hint_on_or_off() {
    // #676's gating test (ADR-0146 Consequences, ADR-0144, #652).
    if !has_ffprobe() {
        return;
    }
    let path = keyed_effects_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    for (painters, chunk) in [(2, 1), (3, 2), (4, 3), (6, 9)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
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
    assert_same_decoded(&sequential, &three, scratch);
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
    assert_same_decoded(&sequential, &three, scratch);
}

/// ADR-0154's gating fixture (#710): paths with sub-pixel motion, rotation, non-uniform
/// scale, keyed `points` with an overshoot, `blur` with `shadow`, and a `mask`, over a
/// still seen through a `clip`, on a 320x180 frame at 30 fps for 1100 ms (33 frames).
fn path_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let to = dir.join("img/boat.jpg");
    std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
    std::fs::copy(trailer().join("img/boat.jpg"), &to).expect("copy the trailer's photo");
    let moving =
        |from: i64, to: i64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let glow = json!({"name": "shadow", "dx": 4, "dy": 3, "radius": 10, "color": "#FF9F2E", "opacity": 0.8});
    let wave = |rise: i64, reach: i64| {
        json!([{"at": [3, 40 + rise], "out": [reach, -35 - rise]},
               {"at": [137, 40 - rise], "in": [-reach, 35 + rise]}])
    };
    let elements = [
        json!({"id": "photo", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 1100,
               "x": 160, "y": 90, "origin": "center", "width": 320, "height": 180,
               "fit": "literal", "clip": [16, 12, 288, 156]}),
        json!({"id": "line", "type": "path", "start": 0, "end": 1100, "x": moving(40, 47),
               "y": 30, "origin": "center", "width": 120, "height": 12, "closed": false,
               "stroke": "#F2F2F2", "stroke_width": 4, "rotation": moving(0, 23),
               "points": [{"at": [2, 6]}, {"at": [118, 6]}],
               "effects": [{"name": "blur", "radius": 2}, glow]}),
        json!({"id": "wave", "type": "path", "start": 0, "end": 1100, "x": moving(90, 96),
               "y": moving(120, 127), "origin": "center", "width": 140, "height": 80,
               "closed": false, "stroke": "#3BA0FF", "stroke_width": 5, "scale": [1.4, 0.7],
               "points": [
                   {"t": 0, "v": wave(0, 30)},
                   {"t": 700, "v": wave(20, 40), "ease": [0.34, 1.56, 0.64, 1]},
                   {"t": 1100, "v": wave(0, 30), "ease": "ease-in-out"}],
               "effects": [glow, {"name": "blur", "radius": 1.5}]}),
        json!({"id": "star", "type": "path", "start": 0, "end": 1100, "x": moving(230, 236),
               "y": 90, "origin": "center", "width": 120, "height": 112, "closed": true,
               "fill": "#FF3B30", "stroke": "#FFFFFF", "stroke_width": 3,
               "rotation": moving(0, -31),
               "scale": [{"t": 0, "v": [1.2, 0.8]}, {"t": 1100, "v": [0.9, 1.15], "ease": "linear"}],
               "points": [{"at": [60, 6]}, {"at": [90, 106]}, {"at": [8, 44]}, {"at": [112, 44]},
                          {"at": [30, 106]}],
               "effects": [{"name": "mask", "shape": "ellipse"}, {"name": "blur", "radius": 3}]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/paths.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "paths.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn paths_paint_the_same_frames_on_any_number_of_painters_and_with_the_bound_hint_off() {
    // ADR-0154's gating test, under ADR-0144.
    if !has_ffprobe() {
        return;
    }
    let path = path_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.frames.len(), 33);
    for (painters, chunk) in [(3, 2), (2, 5), (4, 1)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// ADR-0163 §7's run (#768): path masks with keyed `points` (one easing past its box, so
/// the clamp is painted), `feather` keyed and static, `invert`, a keyed mask `x`, and
/// `[mask, blur]` and `[blur, mask]` so the bounds hint is in play, on elements rotating,
/// non-uniformly scaled and moving by sub-pixels, over a still. 320×180 at 30 fps for
/// 1100 ms: 33 frames.
fn path_mask_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let to = dir.join("img/boat.jpg");
    std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
    std::fs::copy(trailer().join("img/boat.jpg"), &to).expect("copy the trailer's photo");
    let moving =
        |from: i64, to: i64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let blur = |radius: f64| json!({"name": "blur", "radius": radius});
    let shadow = json!({"name": "shadow", "dx": 4, "dy": 3, "radius": 8, "color": "#FF9F2E", "opacity": 0.8});
    let blob = |apex: i64, reach: i64| {
        json!([{"at": [6, 70], "out": [10, -30]}, {"at": [50, apex], "in": [-reach, 0], "out": [reach, 0]},
               {"at": [94, 70], "in": [-10, -30]}, {"at": [50, 76]}])
    };
    let elements = [
        json!({"id": "photo", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 1100,
               "x": 160, "y": 90, "origin": "center", "width": 320, "height": 180,
               "fit": "literal"}),
        // `[mask, blur]`: a keyed, feathered blob on a turning, squashed, drifting plate,
        // its last key overshooting the top of the box.
        json!({"id": "blob", "type": "rect", "start": 0, "end": 1100, "x": moving(60, 69),
               "y": 60, "origin": "center", "width": 100, "height": 80, "fill": "#F2F2F2",
               "rotation": moving(0, 37), "scale": [1.3, 0.8],
               "effects": [
                   {"name": "mask", "shape": "path", "feather": 9, "points": [
                       {"t": 0, "v": blob(30, 20)},
                       {"t": 600, "v": blob(6, 34), "ease": [0.34, 1.8, 0.64, 1.0]},
                       {"t": 1100, "v": blob(24, 12), "ease": "ease-in-out"}]},
                   blur(3.0)]}),
        // `[blur, mask]`: an inverted, feathered star (nonzero, so its centre is kept by
        // the plain mask and cut by this one) in a written rect whose `x` is keyed.
        json!({"id": "star", "type": "ellipse", "start": 0, "end": 1100, "x": 160,
               "y": moving(60, 67), "origin": "center", "width": 120, "height": 90,
               "fill": "#3BA0FF", "rotation": -14,
               "scale": [{"t": 0, "v": [1.1, 0.9]}, {"t": 1100, "v": [0.8, 1.2], "ease": "linear"}],
               "effects": [blur(2.0),
                   {"name": "mask", "shape": "path", "x": moving(10, 25), "y": 8,
                    "width": 80, "height": 76, "invert": true,
                    "feather": [{"t": 0, "v": 0}, {"t": 700, "v": 15, "ease": "ease-out"}],
                    "points": [{"at": [40, 2]}, {"at": [64, 74]}, {"at": [2, 28]}, {"at": [78, 28]},
                               {"at": [16, 74]}]},
                   shadow]}),
        // A hard path mask and a hard inverted one in one list, cutting a ring-like band,
        // on a turning plate with a shadow after them.
        json!({"id": "band", "type": "rect", "start": 100, "end": 1100,
               "x": moving(250, 243), "y": 120, "origin": "center", "width": 110, "height": 90,
               "fill": "#FF3B30", "rotation": moving(8, -20), "scale": [0.9, 1.25],
               "effects": [
                   {"name": "mask", "shape": "path", "points": [
                       {"t": 100, "v": [{"at": [5, 45], "out": [0, -40]}, {"at": [105, 45], "in": [0, -40], "out": [0, 40]}, {"at": [5, 45], "in": [0, 40]}]},
                       {"t": 1100, "v": [{"at": [15, 45], "out": [0, -30]}, {"at": [95, 45], "in": [0, -30], "out": [0, 30]}, {"at": [15, 45], "in": [0, 30]}], "ease": "linear"}]},
                   {"name": "mask", "shape": "path", "x": 30, "y": 25, "width": 50, "height": 40,
                    "invert": true,
                    "points": [{"at": [0, 20]}, {"at": [25, 0]}, {"at": [50, 20]}, {"at": [25, 40]}]},
                   shadow]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/path-masks.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "path-masks.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn path_masks_paint_the_same_frames_on_one_to_ten_painters_with_the_hint_on_or_off() {
    // ADR-0163 §7: no prototype gate, so this run is the gate. If it fails, the slice fails.
    if !has_ffprobe() {
        return;
    }
    let path = path_mask_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    assert!(
        sequential.frames.windows(2).all(|pair| pair[0] != pair[1]),
        "the masks move on every frame"
    );
    for (painters, chunk) in [(2, 1), (3, 2), (4, 3), (5, 7), (7, 2), (10, 1), (10, 3)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2), chunks(10, 1)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// #751's gating fixture (ADR-0158 §8, ADR-0144): all three joins and all three caps, a
/// miter at limits 4 and 10, and a keyed miter whose in-between corner sharpens past its
/// limit and snaps to a bevel, under motion, rotation and scale, with `shadow` and `blur`
/// so the bounds hint is in play. 320×180 at 30 fps for 1100 ms: 33 frames.
fn stroke_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let moving =
        |from: i64, to: i64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let shadow = json!({"name": "shadow", "dx": 4, "dy": 3, "radius": 6, "color": "#FF9F2E", "opacity": 0.8});
    let zigzag = json!([{"at": [12, 60]}, {"at": [40, 12]}, {"at": [60, 60]}, {"at": [80, 12]},
                        {"at": [108, 60]}]);
    let join = |id: &str, x: i64, join: Value, limit: Option<i64>, effects: Value| {
        let mut element = json!({"id": id, "type": "path", "start": 0, "end": 1100,
            "x": moving(x, x + 5), "y": 45, "origin": "center", "width": 120, "height": 72,
            "closed": false, "stroke": "#F2F2F2", "stroke_width": 6, "stroke_join": join,
            "rotation": moving(0, 17), "points": zigzag.clone(), "effects": effects});
        if let Some(limit) = limit {
            element["stroke_miter_limit"] = json!(limit);
        }
        element
    };
    let cap = |id: &str, y: i64, cap: &str, effects: Value| {
        json!({"id": id, "type": "path", "start": 0, "end": 1100, "x": moving(60, 66),
               "y": y, "origin": "center", "width": 100, "height": 40, "closed": false,
               "stroke": "#3BA0FF", "stroke_width": 12, "stroke_cap": cap,
               "rotation": moving(-30, 25), "scale": [1.2, 0.9],
               "points": [{"at": [10, 30], "out": [20, -15]}, {"at": [90, 10]}],
               "effects": effects})
    };
    let elements = [
        join("round", 70, json!("round"), None, json!([shadow])),
        join(
            "bevel",
            160,
            json!("bevel"),
            None,
            json!([{"name": "blur", "radius": 1.5}]),
        ),
        join(
            "miter-4",
            250,
            json!("miter"),
            Some(4),
            json!([shadow, {"name": "blur", "radius": 2}]),
        ),
        cap("butt", 110, "butt", json!([shadow])),
        cap(
            "round-cap",
            140,
            "round",
            json!([{"name": "blur", "radius": 2}]),
        ),
        cap("square", 168, "square", json!([shadow])),
        json!({"id": "keyed", "type": "path", "start": 0, "end": 1100, "x": 230, "y": 130,
               "origin": "center", "width": 120, "height": 80, "closed": false,
               "stroke": "#FF3B30", "stroke_width": 3, "stroke_join": "miter",
               "stroke_miter_limit": 10, "stroke_cap": "square",
               "points": [
                   {"t": 0, "v": [{"at": [15, 20]}, {"at": [90, 40]}, {"at": [15, 60]}]},
                   {"t": 1100, "v": [{"at": [15, 60]}, {"at": [90, 40]}, {"at": [15, 20]}],
                    "ease": "linear"}],
               "effects": [shadow, {"name": "blur", "radius": 1}]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/strokes.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "strokes.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn joins_and_caps_paint_the_same_frames_on_any_number_of_painters_with_the_hint_on_or_off() {
    // ADR-0158 §8's gating test, under ADR-0144.
    if !has_ffprobe() {
        return;
    }
    let path = stroke_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    for (painters, chunk) in [(3, 2), (2, 5), (4, 1), (10, 3)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// #752's gating fixture (ADR-0158 §8, ADR-0144): marching ants on all three shapes — a
/// `rect`, a rounded `rect`, an `ellipse` and a closed `path` — and a dotted open path,
/// each `stroke_dash_offset` keyed through whole and fractional values in both signs, under
/// motion, rotation and scale, with `shadow` and `blur` so the bounds hint is in play.
/// 320×180 at 30 fps for 1100 ms: 33 frames.
fn dash_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let moving =
        |from: i64, to: i64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let shadow = json!({"name": "shadow", "dx": 4, "dy": 3, "radius": 6, "color": "#FF9F2E", "opacity": 0.8});
    let blur = json!({"name": "blur", "radius": 1.5});
    let ants = |id: &str, kind: &str, x: i64, y: i64, offset: i64, effects: Value| {
        json!({"id": id, "type": kind, "start": 0, "end": 1100, "x": moving(x, x + 4), "y": y,
               "origin": "center", "width": 90, "height": 60, "stroke": "#F2F2F2",
               "stroke_width": 4, "stroke_dash": [10, 6], "stroke_dash_offset": moving(0, offset),
               "rotation": moving(0, 13), "effects": effects})
    };
    let mut rounded = ants("rounded", "rect", 160, 45, 48, json!([blur]));
    rounded["radius"] = json!(18);
    rounded["scale"] = json!([1.1, 0.9]);
    rounded["stroke_dash"] = json!([14, 4, 2, 4]);
    let elements = [
        ants("rect", "rect", 60, 45, -48, json!([shadow])),
        rounded,
        ants(
            "ellipse",
            "ellipse",
            260,
            45,
            -77,
            json!([shadow, {"name": "blur", "radius": 2}]),
        ),
        json!({"id": "closed", "type": "path", "start": 0, "end": 1100, "x": moving(80, 86),
               "y": 125, "origin": "center", "width": 120, "height": 80, "closed": true,
               "stroke": "#3BA0FF", "stroke_width": 5, "stroke_join": "miter",
               "stroke_miter_limit": 4, "stroke_cap": "square", "stroke_dash": [18, 7],
               "stroke_dash_offset": moving(0, -100), "rotation": moving(-10, 12),
               "points": [{"at": [20, 20]}, {"at": [100, 20], "out": [0, 20]},
                          {"at": [100, 60]}, {"at": [20, 60]}],
               "effects": [shadow, {"name": "blur", "radius": 1}]}),
        json!({"id": "dots", "type": "path", "start": 0, "end": 1100, "x": 230, "y": 130,
               "origin": "center", "width": 140, "height": 40, "closed": false,
               "stroke": "#FF3B30", "stroke_width": 6, "stroke_cap": "round",
               "stroke_dash": [0, 12], "stroke_dash_offset": moving(0, 36),
               "scale": [1.3, 0.8],
               "points": [{"at": [10, 20], "out": [40, -15]}, {"at": [130, 20]}],
               "effects": [shadow]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/dashes.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "dashes.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn marching_ants_paint_the_same_frames_on_any_number_of_painters_with_the_hint_on_or_off() {
    // ADR-0158 §8's gating test for dashes, under ADR-0144.
    if !has_ffprobe() {
        return;
    }
    let path = dash_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    assert!(
        sequential.frames.windows(2).all(|pair| pair[0] != pair[1]),
        "the ants march on every frame"
    );
    for (painters, chunk) in [(3, 2), (2, 5), (4, 1), (10, 3)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// #761's gating fixture (ADR-0160 §8, ADR-0144): a round-capped draw-on whose first frame
/// is empty, a motion-blurred ring loader on a `rect` whose window crosses its start point
/// (a corner) and one on an `ellipse`, a dashed closed path whose window crosses the start point through
/// the dash on there, a dashed open path drawing on, and a square-capped closed path whose
/// keyed start and end cross, under motion, rotation and scale, with `shadow` and `blur` so
/// the bounds hint is in play. 320×180 at 30 fps for 1100 ms: 33 frames.
fn trim_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let moving =
        |from: f64, to: f64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let shadow = json!({"name": "shadow", "dx": 4, "dy": 3, "radius": 6, "color": "#FF9F2E", "opacity": 0.8});
    let blur = json!({"name": "blur", "radius": 1.5});
    let loader = |id: &str, kind: &str, x: i64, effects: Value| {
        let x = json!([{"t": 0, "v": x}, {"t": 1100, "v": x + 4, "ease": "linear"}]);
        json!({"id": id, "type": kind, "start": 0, "end": 1100, "x": x,
               "y": 45, "origin": "center", "width": 80, "height": 60, "stroke": "#F2F2F2",
               "stroke_width": 6, "trim_start": 0, "trim_end": 0.3,
               "trim_offset": moving(0.6, 2.9), "rotation": moving(0.0, 13.0), "effects": effects})
    };
    let mut rounded = loader("rounded", "rect", 160, json!([blur]));
    rounded["radius"] = json!(16);
    rounded["scale"] = json!([1.1, 0.9]);
    // Motion blur samples the keyed window with every other keyed value (ADR-0155).
    let mut rect = loader("rect", "rect", 60, json!([shadow]));
    rect["motion_blur"] = json!({"shutter": 180, "samples": 4});
    let elements = [
        json!({"id": "draw-on", "type": "path", "start": 0, "end": 1100, "x": 50, "y": 130,
               "origin": "center", "width": 90, "height": 60, "closed": false,
               "stroke": "#3BA0FF", "stroke_width": 8, "stroke_cap": "round",
               "trim_end": [{"t": 0, "v": 0}, {"t": 1000, "v": 1, "ease": [0.65, 0, 0.35, 1]}],
               "points": [{"at": [10, 50], "out": [30, -44]}, {"at": [80, 10]}],
               "effects": [shadow]}),
        rect,
        rounded,
        loader(
            "ellipse",
            "ellipse",
            260,
            json!([shadow, {"name": "blur", "radius": 2}]),
        ),
        json!({"id": "dashed-ring", "type": "path", "start": 0, "end": 1100, "x": 140,
               "y": 130, "origin": "center", "width": 80, "height": 70, "closed": true,
               "stroke": "#FF3B30", "stroke_width": 5, "stroke_join": "miter",
               "stroke_miter_limit": 4, "stroke_cap": "butt", "stroke_dash": [18, 6],
               "stroke_dash_offset": 9, "trim_start": 0.1, "trim_end": 0.6,
               "trim_offset": moving(0.0, 1.4), "scale": [1.2, 0.9],
               "points": [{"at": [10, 10]}, {"at": [70, 10], "out": [0, 20]}, {"at": [70, 60]},
                          {"at": [10, 60]}],
               "effects": [shadow, {"name": "blur", "radius": 1}]}),
        json!({"id": "dotted", "type": "path", "start": 0, "end": 1100, "x": 225, "y": 125,
               "origin": "center", "width": 90, "height": 40, "closed": false,
               "stroke": "#FFD60A", "stroke_width": 6, "stroke_cap": "round",
               "stroke_dash": [0, 12], "trim_end": moving(0.0, 1.0),
               "points": [{"at": [10, 20], "out": [30, -15]}, {"at": [80, 20]}],
               "effects": [shadow]}),
        json!({"id": "crossing", "type": "path", "start": 0, "end": 1100, "x": 290,
               "y": 140, "origin": "center", "width": 50, "height": 50, "closed": true,
               "stroke": "#34C759", "stroke_width": 6, "stroke_cap": "square",
               "stroke_join": "bevel", "trim_start": moving(0.0, 0.8),
               "trim_end": moving(1.0, 0.2), "rotation": moving(0.0, -20.0),
               "points": [{"at": [25, 5]}, {"at": [45, 25]}, {"at": [25, 45]}, {"at": [5, 25]}],
               "effects": [blur]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/trim.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "trim.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn trimmed_strokes_paint_the_same_frames_on_one_to_ten_painters_with_the_hint_on_or_off() {
    // ADR-0160 §8's byte-identity requirement, under ADR-0144: the slice's gating test.
    if !has_ffprobe() {
        return;
    }
    let path = trim_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    assert!(
        sequential.frames.windows(2).all(|pair| pair[0] != pair[1]),
        "the windows move on every frame"
    );
    for (painters, chunk) in [
        (2, 1),
        (3, 2),
        (4, 3),
        (5, 7),
        (6, 4),
        (7, 2),
        (8, 5),
        (9, 1),
        (10, 1),
        (10, 3),
    ] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2), chunks(10, 1)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// #724's gating fixture (ADR-0156 §6, ADR-0144): `posterize`, `glow` and
/// `directional_blur` together with `blur`, a `mask` and a non-`normal` `blend`, under
/// sub-pixel motion, rotation, non-uniform scale and a flip, with keyed parameters and one
/// element under `motion_blur`, over a still. 160x90 at 30 fps for 600 ms: 18 frames.
fn named_effects_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let keyed = |from: Value, to: Value| json!([{"t": 0, "v": from}, {"t": 600, "v": to, "ease": "linear"}]);
    let elements = [
        json!({"id": "ground", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 600,
               "x": 80, "y": 45, "origin": "center", "width": 160, "height": 90,
               "fit": "literal",
               "effects": [{"name": "posterize", "levels": keyed(json!(3), json!(12))}]}),
        json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 28,
               "color": "#F4E2A0", "align": "center", "runs": [{"text": "SPY"}],
               "width": 120, "height": 40, "start": 0, "end": 600,
               "x": keyed(json!(76), json!(83)), "y": 40, "origin": "center",
               "rotation": keyed(json!(-6.0), json!(9.5)), "scale": [1.3, 0.8],
               "effects": [{"name": "blur", "radius": 1.5},
                           {"name": "glow", "threshold": keyed(json!(0.3), json!(0.7)),
                            "radius": 9, "intensity": 1.6},
                           {"name": "posterize", "levels": 6}],
               "blend": "screen"}),
        json!({"id": "streak", "type": "rect", "start": 0, "end": 600,
               "x": keyed(json!(30), json!(37)), "y": 70, "origin": "center", "width": 40,
               "height": 10, "fill": "#20C0F0", "rotation": 17, "scale": [-1.2, 1.0],
               "effects": [{"name": "mask", "shape": "ellipse", "feather": 3},
                           {"name": "directional_blur",
                            "angle": keyed(json!(350.0), json!(10.0)),
                            "length": keyed(json!(4.5), json!(22))},
                           {"name": "blur", "radius": 2}],
               "blend": "add"}),
        json!({"id": "all", "type": "ellipse", "start": 0, "end": 600,
               "x": keyed(json!(110), json!(126)), "y": 62, "origin": "center", "width": 30,
               "height": 22, "fill": "#FF9F2E", "rotation": keyed(json!(0.0), json!(40.0)),
               "effects": [{"name": "posterize", "levels": 4},
                           {"name": "glow", "threshold": 0.4, "radius": 6, "intensity": 2},
                           {"name": "directional_blur", "angle": 45, "length": 9.5},
                           {"name": "mask", "shape": "rect", "x": 2, "y": 2, "width": 26,
                            "height": 18},
                           {"name": "blur", "radius": 1}],
               "motion_blur": {"shutter": 180, "samples": 4},
               "blend": "overlay"}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 160, "height": 90}, "fps": 30, "background": "#101418",
        "duration": 600, "output": "out/named-effects.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "named-effects.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn named_effects_paint_the_same_frames_on_any_number_of_painters_with_the_hint_on_or_off() {
    // ADR-0156 §6, the gating test (#724): a member that fails it is withdrawn, not excused.
    if !has_ffprobe() {
        return;
    }
    let path = named_effects_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.frames.len(), 18);
    assert_ne!(
        sequential.frames[2], sequential.frames[9],
        "the scene moves"
    );
    for (painters, chunk) in [(3, 2), (2, 5), (4, 1), (10, 1)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2), chunks(10, 1)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// ADR-0155's gating fixture (#719): every element carries `motion_blur`. A fast slide on
/// `ease-out`, a spin, a rect whose size, radius and colour are keyed, a title staggering in
/// through `units`, a moving `video`, a still element, and one with `effects`, a `mask` and
/// `blend: screen`, over a still seen through a `clip`. 160x90 at 30 fps for 600 ms: 18
/// frames, the motion crossing every chunk boundary, and settling before the end so the
/// moving-to-still seam is painted too.
fn motion_blur_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    let keyed = |from: Value, to: Value, until: i64, ease: &str| json!([{"t": 0, "v": from}, {"t": until, "v": to, "ease": ease}]);
    let blur = |shutter: i64, samples: i64| json!({"shutter": shutter, "samples": samples});
    let elements = [
        json!({"id": "ground", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 600,
               "x": 80, "y": 45, "origin": "center", "width": 160, "height": 90,
               "fit": "literal", "clip": [4, 4, 152, 82]}),
        json!({"id": "slide", "type": "rect", "start": 0, "end": 600,
               "x": keyed(json!(-20), json!(140), 400, "ease-out"), "y": 14,
               "origin": "center", "width": 24, "height": 12, "fill": "#F04020",
               "motion_blur": blur(180, 8)}),
        json!({"id": "spin", "type": "rect", "start": 0, "end": 600, "x": 30, "y": 60,
               "origin": "center", "width": 36, "height": 4, "fill": "#FFFFFF",
               "rotation": keyed(json!(0.0), json!(720.0), 600, "linear"),
               "motion_blur": blur(360, 12)}),
        json!({"id": "grow", "type": "rect", "start": 0, "end": 600, "x": 120, "y": 60,
               "origin": "center", "width": keyed(json!(6), json!(34), 300, "ease-in-out"),
               "height": keyed(json!(6), json!(24), 300, "ease-in-out"),
               "radius": keyed(json!(0), json!(10), 300, "linear"),
               "fill": keyed(json!("#2060FF"), json!("#FFC020"), 300, "linear"),
               "motion_blur": blur(180, 6)}),
        json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 18,
               "color": "#E3C067", "align": "center", "width": 120, "height": 26,
               "start": 0, "end": 600, "x": 80, "y": 34, "origin": "center",
               "runs": [{"text": "SPY"}], "caption": false,
               "units": {"by": "letter", "every": 60,
                         "x": keyed(json!(-30), json!(0), 250, "ease-out"),
                         "opacity": keyed(json!(0.0), json!(1.0), 200, "linear")},
               "motion_blur": blur(180, 8)}),
        json!({"id": "footage", "type": "video", "start": 0, "end": 600, "source": clip,
               "source_start": 0, "source_end": 600, "y": 74, "origin": "center",
               "x": keyed(json!(20), json!(70), 500, "ease-in"),
               "width": 32, "height": 24, "fit": "literal", "volume": 0,
               "motion_blur": blur(180, 4)}),
        json!({"id": "still", "type": "ellipse", "start": 0, "end": 600, "x": 140, "y": 20,
               "origin": "center", "width": 16, "height": 16, "fill": "#20C0F0",
               "rotation": 15, "motion_blur": blur(360, 32)}),
        json!({"id": "lens", "type": "rect", "start": 0, "end": 600, "y": 46,
               "x": keyed(json!(60), json!(110), 450, "ease-in-out"),
               "rotation": keyed(json!(-10.0), json!(25.0), 450, "linear"),
               "origin": "center", "width": 40, "height": 26, "fill": "#FF9F2E",
               "blend": "screen",
               "effects": [{"name": "mask", "shape": "ellipse", "feather": 2},
                           {"name": "shadow", "dx": 0, "dy": 0, "radius": 5,
                            "color": "#FF4020", "opacity": 0.9},
                           {"name": "blur", "radius": 1.5}],
               "motion_blur": blur(270, 8)}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 160, "height": 90}, "fps": 30, "background": "#101418",
        "duration": 600, "output": "out/motion-blur.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "motion-blur.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn motion_blur_paints_the_same_frames_on_any_number_of_painters_with_the_hint_on_or_off() {
    // ADR-0155 §6, the gating test: if it fails, motion blur is withdrawn, not excused.
    if !has_ffprobe() {
        return;
    }
    let path = motion_blur_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.frames.len(), 18);
    assert_ne!(
        sequential.frames[2], sequential.frames[9],
        "the scene moves"
    );
    for (painters, chunk) in [(3, 2), (2, 5), (4, 1), (10, 1)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// ADR-0156's gating fixture for `grain` (#725): grain on a moving `video`, on a rect turned
/// and scaled while it moves, and on a grey rect blended `overlay` over a still, the texture
/// idiom. Beside them, colour grain under a blur and a shadow, a keyed `amount`, two members
/// in one list, and grain under `motion_blur`. 160x90 at 30 fps for 600 ms: 18 frames, the
/// grain re-rolling on every one, across every chunk boundary.
fn grain_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    let keyed = |from: Value, to: Value, until: i64, ease: &str| json!([{"t": 0, "v": from}, {"t": until, "v": to, "ease": ease}]);
    let grain = |seed: i64, amount: Value, size: i64, mono: bool| json!({"name": "grain", "seed": seed, "amount": amount, "size": size, "mono": mono});
    let elements = [
        json!({"id": "ground", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 600,
               "x": 80, "y": 45, "origin": "center", "width": 160, "height": 90,
               "fit": "literal"}),
        json!({"id": "footage", "type": "video", "start": 0, "end": 600, "source": clip,
               "source_start": 0, "source_end": 600, "y": 30, "origin": "center",
               "x": keyed(json!(30), json!(60), 500, "ease-in"),
               "width": 48, "height": 36, "fit": "literal", "volume": 0,
               "effects": [grain(7, json!(0.15), 1, true)]}),
        json!({"id": "texture", "type": "rect", "start": 0, "end": 600, "x": 120, "y": 45,
               "origin": "center", "width": 80, "height": 90, "fill": "#808080",
               "blend": "overlay", "effects": [grain(11, json!(0.2), 2, true)]}),
        json!({"id": "turned", "type": "rect", "start": 33, "end": 600, "x": 40, "y": 70,
               "origin": "center", "width": 30, "height": 14, "fill": "#E0A030",
               "rotation": keyed(json!(0.0), json!(45.0), 600, "linear"),
               "scale": keyed(json!([1.0, 1.0]), json!([2.5, 1.5]), 600, "ease-in-out"),
               "effects": [grain(3, json!(0.3), 3, false)]}),
        json!({"id": "glowing", "type": "text", "font": "cinzel-bold", "size": 18,
               "color": "#E3C067", "align": "center", "width": 120, "height": 26,
               "start": 0, "end": 600, "x": 80, "y": 76, "origin": "center",
               "runs": [{"text": "GRAIN"}], "caption": false,
               "effects": [grain(5, keyed(json!(0.0), json!(0.5), 400, "linear"), 1, false),
                           {"name": "blur", "radius": 1.5},
                           {"name": "shadow", "dx": 2, "dy": 1, "radius": 4,
                            "color": "#2050FF", "opacity": 0.8},
                           grain(6, json!(0.1), 2, true)]}),
        json!({"id": "smeared", "type": "ellipse", "start": 0, "end": 600, "y": 14,
               "x": keyed(json!(10), json!(150), 400, "ease-out"), "origin": "center",
               "width": 20, "height": 12, "fill": "#20C0F0",
               "effects": [grain(9, json!(0.4), 1, false)],
               "motion_blur": {"shutter": 180, "samples": 6}}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 160, "height": 90}, "fps": 30, "background": "#101418",
        "duration": 600, "output": "out/grain.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "grain.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn grain_paints_the_same_frames_on_any_number_of_painters_with_the_hint_on_or_off() {
    // ADR-0156 §6, the gating test: if it fails, `grain` is withdrawn, not excused.
    if !has_ffprobe() {
        return;
    }
    let path = grain_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.frames.len(), 18);
    for pair in sequential.frames.windows(2) {
        assert_ne!(pair[0], pair[1], "the grain re-rolls on every frame");
    }
    for (painters, chunk) in [(3, 2), (2, 5), (4, 1), (10, 1)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// ADR-0157's acceptance fixture: five `video` elements carrying `source_time` on one clip —
/// a ramp (1× → 0.3× → 1×), an eased segment, a reverse, a flat freeze between two 1×
/// stretches, and a literal freeze — on a 192x96 frame at 25 fps for 2000 ms (50 frames).
/// The reverse, the freeze and the slow stretch cross every chunk boundary, so a chunk start
/// reopens a feed (or decodes a frame alone) in the middle of each.
fn remap_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    let linear = |keys: &[(i64, i64)]| {
        let records: Vec<Value> = keys
            .iter()
            .enumerate()
            .map(|(i, &(t, v))| match i {
                0 => json!({"t": t, "v": v}),
                _ => json!({"t": t, "v": v, "ease": "linear"}),
            })
            .collect();
        json!(records)
    };
    let curves = [
        (
            "ramp",
            linear(&[(0, 0), (400, 400), (1200, 640), (1960, 1400)]),
        ),
        (
            "eased",
            json!([{"t": 0, "v": 200}, {"t": 1960, "v": 3000, "ease": "ease-in-out"}]),
        ),
        ("reverse", linear(&[(0, 3000), (1960, 1040)])),
        (
            "frozen",
            linear(&[(0, 100), (600, 700), (1400, 700), (1960, 1260)]),
        ),
        ("literal", json!(1500)),
    ];
    let tracks: Vec<Value> = curves
        .into_iter()
        .enumerate()
        .map(|(i, (id, curve))| {
            let element = json!({"id": id, "type": "video", "start": 0, "end": 2000,
                "source": clip, "source_time": curve, "x": (i as i64 % 3) * 64,
                "y": (i as i64 / 3) * 48, "origin": "top-left", "width": 64, "height": 48,
                "fit": "literal", "volume": 0});
            json!({"name": format!("t{i}"), "layer": i, "elements": [element]})
        })
        .collect();
    let project = json!({
        "frame": {"width": 192, "height": 96}, "fps": 25, "background": "#000000",
        "duration": 2000, "output": "out/remap.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "remap.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn time_remapped_video_paints_the_same_frames_on_any_number_of_painters() {
    // ADR-0157's byte-identity acceptance (ADR-0144): every chunk start reopens each feed, or
    // decodes a frame alone, partway through a reverse, a freeze and a slow stretch.
    if !has_ffprobe() {
        return;
    }
    let path = remap_project(line!());
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.frames.len(), 50);
    assert_ne!(
        sequential.frames[5], sequential.frames[30],
        "the clips move"
    );
    for (painters, chunk) in [(3, 4), (2, 7), (4, 1), (10, 1), (3, 13)] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
}

/// ADR-0161 §9's run (#765): texts bent along their own curves, on a 320×180 frame at 30 fps
/// for 1100 ms (33 frames): a centred arc turning under a `shadow` (so the bounds hint is in
/// play), a title sliding on along a wave, a circle badge wrapping under a `blur`, a fading
/// letter stagger with a `shadow`, a stroked banner with keyed points, and a mixed Arabic
/// line sliding under motion blur.
fn text_path_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    for (from, to) in [
        (
            fixtures.join("benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf"),
            "fonts/Oswald-SemiBold.ttf",
        ),
        (
            fixtures.join("letter-spacing/fonts/NotoNaskhArabic-Regular.ttf"),
            "fonts/Naskh.ttf",
        ),
    ] {
        let to = dir.join(to);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("font dir");
        std::fs::copy(from, &to).expect("copy the fixture font");
    }
    let ramp = |from: Value, to: Value, until: i64| json!([{"t": 0, "v": from}, {"t": until, "v": to, "ease": "ease-in-out"}]);
    let arc = json!([{"at": [24, 70], "out": [40, -40]}, {"at": [136, 70], "in": [-40, -40]}]);
    let wave = |amp: i64| {
        json!([{"at": [20, 40], "out": [25, -amp]},
               {"at": [80, 40], "in": [-25, amp], "out": [25, -amp]},
               {"at": [140, 40], "in": [-25, amp]}])
    };
    let circle = json!([
        {"at": [50, 15], "in": [-19, 0], "out": [19, 0]},
        {"at": [85, 50], "in": [0, -19], "out": [0, 19]},
        {"at": [50, 85], "in": [19, 0], "out": [-19, 0]},
        {"at": [15, 50], "in": [0, 19], "out": [0, -19]}]);
    let text = |id: &str, x: i64, y: i64, size: i64, words: &str, extra: Value| {
        let mut element = json!({"id": id, "type": "text", "start": 0, "end": 1100, "x": x,
            "y": y, "origin": "top-left", "width": 160, "height": 90, "font": "titles",
            "size": size, "color": "#F2E6C9", "runs": [{"text": words}], "caption": false});
        for (key, value) in extra.as_object().expect("fields") {
            element[key] = value.clone();
        }
        element
    };
    let elements = [
        text(
            "arc",
            0,
            0,
            18,
            "OVER THE ARC",
            json!({"align": "center",
            "path": {"closed": false, "points": arc}, "path_offset": 0.5,
            "rotation": ramp(json!(0.0), json!(12.0), 1100),
            "effects": [{"name": "shadow", "dx": 2, "dy": 3, "radius": 4, "color": "#000000",
                         "opacity": 0.7}]}),
        ),
        text(
            "slide",
            160,
            0,
            14,
            "SLIDING ON",
            json!({"align": "end",
            "path": {"closed": false, "points": wave(14)},
            "path_offset": ramp(json!(0.0), json!(1.0), 1000)}),
        ),
        text(
            "badge",
            0,
            85,
            12,
            "ROUND AND ROUND IT GOES",
            json!({"width": 100,
            "height": 100, "path": {"closed": true, "points": circle},
            "path_offset": ramp(json!(0.0), json!(1.0), 1000),
            "effects": [{"name": "blur", "radius": 1}]}),
        ),
        text(
            "stagger",
            100,
            80,
            18,
            "DROP IN",
            json!({"align": "center",
            "path": {"closed": false, "points": arc}, "path_offset": 0.5,
            "units": {"by": "letter", "every": 70, "y": ramp(json!(-40), json!(0), 500),
                      "rotation": ramp(json!(-30.0), json!(0.0), 500),
                      "opacity": ramp(json!(0.0), json!(1.0), 400)},
            "effects": [{"name": "shadow", "dx": 2, "dy": 2, "radius": 3, "color": "#000000",
                         "opacity": 0.6}]}),
        ),
        text(
            "banner",
            160,
            60,
            14,
            "WAVING BANNER",
            json!({"align": "center",
            "path": {"closed": false, "points": [
                {"t": 0, "v": wave(14)}, {"t": 550, "v": wave(-14), "ease": "ease-in-out"},
                {"t": 1100, "v": wave(14), "ease": "ease-in-out"}]},
            "path_offset": 0.5, "stroke": "#7A1F1F", "stroke_width": 2}),
        ),
        text(
            "arabic",
            160,
            95,
            16,
            "بسم الله 2026",
            json!({
            "path": {"closed": false, "points": [{"at": [20, 60], "out": [40, -30]},
                                                  {"at": [140, 60], "in": [-40, -30]}]},
            "path_offset": ramp(json!(0.0), json!(0.2), 1100),
            "motion_blur": {"shutter": 180, "samples": 4}}),
        ),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/text-path.mp4",
        "fonts": {"titles": [{"file": "fonts/Oswald-SemiBold.ttf"}, {"file": "fonts/Naskh.ttf"}]},
        "fontVendor": {
            "fonts/Oswald-SemiBold.ttf": {
                "licence": "OFL-1.1",
                "source": "google/fonts ofl/oswald, instanced wght=600",
                "sha256": "442420449b66e3f8a49025fbb229a8b4b1efa5f7be9458a7c3244498d34f8de9"},
            "fonts/Naskh.ttf": {
                "licence": "OFL-1.1",
                "source": "notofonts/arabic NotoNaskhArabic-v2.019",
                "sha256": "eb5cde7fecba8c6a481039257fe02d5fd69b7b0e36f8afc56ee22f4b1d7e8c21"},
        },
        "tracks": tracks,
    });
    write_project(
        &dir,
        "text-path.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn text_on_a_path_paints_the_same_frames_on_one_to_ten_painters_with_the_hint_on_or_off() {
    // ADR-0161 §9's byte identity, in the build (#765).
    if !has_ffprobe() {
        return;
    }
    let path = text_path_project(line!());
    let validated = montagent_core::validate(&path);
    assert_eq!(
        validated.exit_code(),
        ExitCode::Ok,
        "{:?}",
        validated.findings
    );
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    assert!(
        sequential.frames.windows(2).all(|pair| pair[0] != pair[1]),
        "the texts move on every frame"
    );
    for (painters, chunk) in [
        (2, 1),
        (3, 2),
        (4, 3),
        (5, 7),
        (6, 4),
        (7, 2),
        (8, 5),
        (9, 1),
        (10, 1),
        (10, 3),
    ] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
    }
    for forced in [Forced::OnePainter, chunks(3, 2), chunks(10, 1)] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
    }
}

/// #787's gating fixture (ADR-0167 §3, ADR-0168, ADR-0144): projected elements on a 320×180
/// frame at 30 fps for 1100 ms (33 frames). A card flip (a front 0 → 180 and a back −180 → 0,
/// each drawing nothing past 90°), a door swing about `center-left`, a projected `video`
/// under a rounded mask, projected text with a shadow, a projected stroked `path`, a still
/// with `blur`, `shadow`, `mask`, `grain` and `directional_blur` drawn flat and projected, a
/// card under `motion_blur` passing 90°, and a projected card that is also rotated and
/// scaled.
fn projection_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let clip = clip(&dir).display().to_string().replace('\\', "/");
    let moving = |from: Value, to: Value| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let shadow = json!({"name": "shadow", "dx": 3, "dy": 4, "radius": 6, "color": "#000000", "opacity": 0.6});
    let elements = [
        json!({"id": "front", "type": "rect", "start": 0, "end": 1100, "x": 45, "y": 45,
               "origin": "center", "width": 70, "height": 50, "fill": "#E8D9B0", "radius": 6,
               "swivel": moving(json!(0), json!(180)), "perspective": 400,
               "effects": [shadow]}),
        json!({"id": "back", "type": "rect", "start": 0, "end": 1100, "x": 45, "y": 45,
               "origin": "center", "width": 70, "height": 50, "fill": "#3B6EA8", "radius": 6,
               "swivel": moving(json!(-180), json!(0)), "perspective": 400,
               "effects": [shadow]}),
        json!({"id": "door", "type": "rect", "start": 0, "end": 1100, "x": 90, "y": 45,
               "origin": "center-left", "width": 60, "height": 70, "fill": "#A0522D",
               "stroke": "#F2F2F2", "stroke_width": 3,
               "swivel": [{"t": 0, "v": 0}, {"t": 550, "v": -75, "ease": "ease-in-out"},
                          {"t": 1100, "v": 0, "ease": "ease-in-out"}],
               "perspective": 400}),
        json!({"id": "screen", "type": "video", "start": 0, "end": 1100, "source": clip,
               "source_start": 0, "source_end": 1100, "x": 200, "y": 45, "origin": "center",
               "width": 64, "height": 48, "fit": "literal", "volume": 0,
               "swivel": 20, "tilt": moving(json!(-30), json!(30)), "perspective": 300,
               "effects": [{"name": "mask", "shape": "rect", "radius": 8}]}),
        json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 28,
               "color": "#E3C067", "align": "center", "runs": [{"text": "SPY"}],
               "width": 120, "height": 50, "start": 0, "end": 1100, "x": 275, "y": 45,
               "origin": "center", "caption": false,
               "swivel": moving(json!(-50), json!(50)), "perspective": 300,
               "effects": [shadow]}),
        json!({"id": "star", "type": "path", "start": 0, "end": 1100, "x": 40, "y": 130,
               "origin": "center", "width": 60, "height": 60, "closed": true,
               "stroke": "#FFD60A", "stroke_width": 3, "stroke_join": "round",
               "tilt": moving(json!(0), json!(60)), "perspective": 300,
               "points": [{"at": [30, 4]}, {"at": [38, 24]}, {"at": [56, 24]}, {"at": [42, 36]},
                          {"at": [48, 56]}, {"at": [30, 44]}, {"at": [12, 56]}, {"at": [18, 36]},
                          {"at": [4, 24]}, {"at": [22, 24]}]}),
        json!({"id": "still", "type": "image", "source": "img/boat.jpg", "start": 0,
               "end": 1100, "x": 120, "y": 130, "origin": "center", "width": 80, "height": 45,
               "fit": "literal", "swivel": 35, "tilt": moving(json!(12), json!(-12)),
               "perspective": 400,
               "effects": [{"name": "blur", "radius": 1.5}, shadow,
                           {"name": "mask", "shape": "rect", "radius": 6},
                           {"name": "grain", "seed": 7, "amount": 0.15, "size": 2, "mono": true},
                           {"name": "directional_blur", "angle": 30, "length": 8}]}),
        json!({"id": "blurred", "type": "rect", "start": 0, "end": 1100, "x": 200, "y": 130,
               "origin": "center", "width": 50, "height": 40, "fill": "#34C759",
               "swivel": moving(json!(45), json!(135)), "perspective": 300,
               "motion_blur": {"shutter": 360, "samples": 8}}),
        json!({"id": "turned", "type": "rect", "start": 0, "end": 1100, "x": 275, "y": 130,
               "origin": "center", "width": 50, "height": 30, "fill": "#FF3B30",
               "swivel": 50, "perspective": 300, "rotation": moving(json!(0.0), json!(90.0)),
               "scale": moving(json!([1.0, 1.0]), json!([1.4, 0.8])),
               "effects": [{"name": "blur", "radius": 1}]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/projection.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "projection.montagent.json",
        &canonical(&project.to_string()),
    )
}

#[test]
fn projected_elements_paint_the_same_frames_on_one_to_ten_painters_with_the_hint_on_or_off() {
    // ADR-0167 §11's byte identity, in the build (#787), under ADR-0144: the slice's gating
    // test. The video keeps an unforced render on one painter, so every run is forced.
    if !has_ffprobe() {
        return;
    }
    let path = projection_project(line!());
    let validated = montagent_core::validate(&path);
    assert_eq!(
        validated.exit_code(),
        ExitCode::Ok,
        "{:?}",
        validated.findings
    );
    let sequential = rendered(&path, Forced::OnePainter);
    assert_eq!(sequential.exit, ExitCode::Ok, "{}", sequential.answer);
    assert_eq!(sequential.frames.len(), 33);
    assert!(
        sequential.frames.windows(2).all(|pair| pair[0] != pair[1]),
        "the projections move on every frame"
    );
    let mut runs = 1;
    for (painters, chunk) in [
        (1, 2),
        (2, 1),
        (3, 2),
        (4, 3),
        (5, 7),
        (6, 4),
        (7, 2),
        (8, 5),
        (9, 1),
        (10, 1),
        (10, 3),
        (3, 40),
    ] {
        let chunked = rendered(&path, chunks(painters, chunk));
        assert_same(&sequential, &chunked, &format!("K={painters}, C={chunk}"));
        runs += 1;
    }
    for forced in [
        Forced::OnePainter,
        chunks(2, 1),
        chunks(3, 2),
        chunks(5, 7),
        chunks(10, 1),
        chunks(10, 3),
    ] {
        let unbounded = rendered_unbounded(&path, forced);
        assert_same(&sequential, &unbounded, "the hint off");
        runs += 1;
    }
    println!("{runs} renders of 33 frames, every frame byte-identical to one painter's");
}
