//! The static-prefix cache (ADR-0195, #932): the bottom run of time-invariant layers is
//! composited once and restored, and nothing a reader can see differs.
//!
//! The oracle is the one `tests/painters.rs` uses: the hash of every frame at the encoder's
//! input (`render::tap_frames`), the MP4's bytes, and the answer with what describes how the
//! file was made removed — each with the cache on and off, on one painter and on three. Every
//! case also asserts that the cache **fired** (`frames_reused` in the profile), so a dead
//! cache cannot pass for an identical one.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use montagent_core::verbs::render::{
    Ask, Forced, Progress, force_painting, force_prefix_cache, render, tap_frames,
};
use serde_json::{Value, json};

mod common;
use common::{canonical, has_ffprobe, mp4_bytes_are_guaranteed, tempdir, write_project};

const HUNG: Duration = Duration::from_secs(120);

struct Rendered {
    frames: Vec<u64>,
    answer: Value,
    mp4: Option<Vec<u8>>,
}

impl Rendered {
    /// The answer minus what describes how the file was made (ADR-0144 §7).
    fn comparable(&self) -> Value {
        let mut answer = self.answer.clone();
        // Probe-sidecar misses describe previous calls, independently of painting
        // (the same exclusion as tests/painters.rs).
        answer.as_object_mut().unwrap().remove("cache_misses");
        if let Some(render) = answer.get_mut("render").and_then(Value::as_object_mut) {
            for volatile in ["wall_ms", "realtime", "painting", "profile"] {
                render.remove(volatile);
            }
            if !mp4_bytes_are_guaranteed() {
                render.remove("bytes");
            }
        }
        answer
    }

    /// `frames_reused` summed over the profile's element rows.
    fn reused(&self) -> u64 {
        self.answer["render"]["profile"]["elements"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .map(|row| row["frames_reused"].as_u64().unwrap_or(0))
                    .sum()
            })
            .unwrap_or(0)
    }

    fn row(&self, id: &str) -> Value {
        self.answer["render"]["profile"]["elements"]
            .as_array()
            .and_then(|rows| rows.iter().find(|row| row["id"] == id))
            .cloned()
            .unwrap_or_else(|| panic!("no profile row for {id}: {}", self.answer))
    }
}

fn rendered(path: &Path, forced: Forced, cache: bool, profile: bool) -> Rendered {
    rendered_with_missing_still(path, forced, cache, profile, false)
}

fn rendered_with_missing_still(
    path: &Path,
    forced: Forced,
    cache: bool,
    profile: bool,
    missing: bool,
) -> Rendered {
    let path = path.to_path_buf();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _forced = force_painting(forced);
        let _cache = force_prefix_cache(cache);
        let tap = tap_frames();
        let still = path.parent().unwrap().join("plate.png");
        let mut removed = None;
        let answer = render(
            &path,
            &Ask {
                profile,
                ..Ask::default()
            },
            &mut |progress: Progress| {
                // Lose a preflight-checked file before painting starts: the public answer
                // must still carry the deferred painter finding on cache hits.
                if missing && progress.of > 0 && progress.done == 0 && removed.is_none() {
                    removed = Some(std::fs::read(&still).unwrap());
                    std::fs::remove_file(&still).unwrap();
                }
            },
        );
        if let Some(bytes) = removed {
            std::fs::write(&still, bytes).unwrap();
        }
        let mp4 = answer
            .video()
            .map(|video| std::fs::read(&video.path).expect("the published file"));
        let _ = tx.send(Rendered {
            frames: tap.hashes(),
            answer: answer.to_json(),
            mp4,
        });
    });
    rx.recv_timeout(HUNG).expect("the render finished")
}

fn forced(painters: usize) -> Forced {
    if painters == 0 {
        return Forced::OnePainter;
    }
    Forced::Chunks {
        painters,
        chunk: 2,
        window_bytes: None,
    }
}

fn plate_png(dir: &Path) {
    let picture = image::RgbaImage::from_fn(64, 36, |x, y| {
        image::Rgba([(x * 4) as u8, (y * 7) as u8, ((x + y) * 2) as u8, 255])
    });
    picture
        .save(dir.join("plate.png"))
        .expect("write the plate");
}

fn write(line: u32, tracks: Value) -> PathBuf {
    let dir = tempdir(line);
    plate_png(&dir);
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 600, "output": "out/prefix.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "prefix.montagent.json",
        &canonical(&project.to_string()),
    )
}

fn plate(id: &str, start: i64, end: i64, fill: &str) -> Value {
    json!({"id": id, "type": "rect", "start": start, "end": end, "x": 0, "y": 0,
           "origin": "top-left", "width": 320, "height": 180, "fill": fill})
}

fn feathered_image(id: &str) -> Value {
    json!({"id": id, "type": "image", "start": 0, "end": 600, "source": "plate.png",
           "x": 40, "y": 30, "origin": "top-left", "width": 240, "height": 120, "fit": "literal",
           "effects": [{"name": "mask", "shape": "rect", "feather": 20},
                       {"name": "shadow", "dx": 4, "dy": 4, "radius": 6,
                        "color": "#000000", "opacity": 0.6}]})
}

fn mover(id: &str) -> Value {
    json!({"id": id, "type": "rect", "start": 0, "end": 600,
           "x": [{"t": 0, "v": 10}, {"t": 600, "v": 200, "ease": "linear"}],
           "y": 100, "origin": "top-left", "width": 40, "height": 30, "fill": "#F2F2F2"})
}

fn keyed_opacity(id: &str) -> Value {
    json!({"id": id, "type": "ellipse", "start": 0, "end": 600, "x": 100, "y": 60,
           "origin": "top-left", "width": 120, "height": 60, "fill": "#FF9F2E",
           "opacity": [{"t": 0, "v": 1}, {"t": 600, "v": 0.2, "ease": "linear"}]})
}

/// A sine tone, `ms` long, written as `tone.wav` beside the project at `path`.
fn tone(path: &Path, ms: i64) {
    let ffmpeg = montagent_core::media::tools::resolve()
        .expect("an ffmpeg")
        .ffmpeg;
    let made = std::process::Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(format!(
            "sine=frequency=440:duration={}",
            ms as f64 / 1000.0
        ))
        .args(["-ac", "2"])
        .arg(path.parent().unwrap().join("tone.wav"))
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(made, "write the tone");
}

/// An `audio` element playing [`tone`] over `[start, end)`. It draws nothing.
fn audio(id: &str, start: i64, end: i64) -> Value {
    json!({"id": id, "type": "audio", "start": start, "end": end, "source": "tone.wav",
           "source_start": 0, "source_end": end - start})
}

/// ADR-0198 §3: an element that draws nothing is still visited with the cache on, so its
/// profile row (time aside) is the one it has with the cache off, and nothing of it is reused.
#[track_caller]
fn listed_alike(path: &Path, on: &Rendered, id: &str) {
    let off = rendered(path, forced(1), false, true);
    let counts = |row: Value| {
        (
            row["type"].clone(),
            row["frames_painted"].clone(),
            row["frames_reused"].clone(),
        )
    };
    assert_eq!(counts(on.row(id)), counts(off.row(id)), "{id}");
    assert_eq!(on.row(id)["frames_reused"], 0, "{id}");
}

/// Plates, then (in layer order) the project's own elements, one track each.
fn stacked(elements: Vec<Value>) -> Value {
    Value::Array(
        elements
            .into_iter()
            .enumerate()
            .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
            .collect(),
    )
}

/// Render with the cache on and off, on 0, 1 and 3 painters, and assert nothing differs but
/// the cache's own counters. Returns the profiled render with the cache on (one painter).
#[track_caller]
fn identical_with_and_without(path: &Path) -> Rendered {
    // The first render of a source misses the probe cache, which the answer reports; every
    // render after it hits.
    rendered(path, forced(1), false, false);
    let baseline = rendered(path, forced(1), false, false);
    assert_eq!(baseline.frames.len(), 18);
    assert!(baseline.mp4.is_some(), "{}", baseline.answer);
    for painters in [0, 1, 3] {
        let cached = rendered(path, forced(painters), true, true);
        assert!(
            cached.reused() > 0,
            "{painters} painters: {}",
            cached.answer
        );
        assert_eq!(
            cached.frames, baseline.frames,
            "{painters} painters: frames"
        );
        let (one, other) = (cached.comparable(), baseline.comparable());
        if one != other {
            let (a, b) = (one.as_object().unwrap(), other.as_object().unwrap());
            let differing: Vec<String> = a
                .iter()
                .filter(|(k, v)| b.get(*k) != Some(v))
                .map(|(k, v)| format!("{k}: {v} != {:?}", b.get(k)))
                .collect();
            panic!("{painters} painters: the answer differs: {differing:?}");
        }
        assert!(
            !mp4_bytes_are_guaranteed() || cached.mp4 == baseline.mp4,
            "{painters} painters: the MP4s differ"
        );
    }
    let off = rendered(path, forced(1), false, true);
    assert_eq!(off.reused(), 0, "the cache is off: {}", off.answer);
    let on = rendered(path, forced(1), true, true);
    assert_eq!(on.frames, baseline.frames);
    on
}

#[test]
fn temporal_grain_ends_the_prefix_despite_literal_parameters() {
    if !has_ffprobe() {
        return;
    }
    let mut grain = plate("grain", 0, 600, "#8090A0");
    grain["effects"] = json!([{"name": "grain", "seed": 7, "amount": 0.4,
        "size": 1, "mono": true}]);
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            grain,
            plate("cap", 0, 600, "#FFFFFF20"),
        ]),
    );
    let on = identical_with_and_without(&path);
    assert_eq!(on.row("table")["frames_reused"], 17);
    assert_eq!(on.row("grain")["frames_reused"], 0);
    assert_eq!(on.row("grain")["frames_painted"], 18);
    assert_eq!(on.row("cap")["frames_reused"], 0);
}

#[test]
fn shared_prefixes_keep_source_and_font_first_use_order_across_intervals() {
    if !has_ffprobe() {
        return;
    }
    let mut first = feathered_image("first");
    first["end"] = json!(200);
    first["source"] = json!("first.png");
    first["x"] = mover("m")["x"].clone();
    let mut second = first.clone();
    second["id"] = json!("second");
    second["start"] = json!(200);
    second["end"] = json!(600);
    second["source"] = json!("second.png");
    let text = json!({"id": "title", "type": "text", "start": 0, "end": 600,
        "font": "cinzel", "size": 24, "color": "#FFFFFF", "runs": [{"text": "PLATE"}],
        "x": 10, "y": 10, "origin": "top-left", "width": 200, "height": 40});
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            feathered_image("book"),
            text,
            plate("late", 200, 600, "#FFFFFF20"),
            first,
            second,
        ]),
    );
    let dir = path.parent().unwrap();
    for name in ["first.png", "second.png"] {
        std::fs::copy(dir.join("plate.png"), dir.join(name)).unwrap();
    }
    let font = "Cinzel-Bold.ttf";
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/benchmark/spy-trailer/fonts")
            .join(font),
        dir.join(font),
    )
    .unwrap();
    let mut project: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    project["fonts"] = json!({"cinzel": [{"file": font}]});
    project["fontVendor"] = json!({font: {"licence": "OFL-1.1",
        "source": "google/fonts ofl/cinzel, instanced wght=700",
        "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}});
    std::fs::write(&path, canonical(&project.to_string())).unwrap();
    let on = identical_with_and_without(&path);
    assert!(on.row("title")["frames_reused"].as_u64().unwrap() > 0);
}

#[test]
fn a_prefix_decline_keeps_the_deferred_finding_and_stops_the_render() {
    if !has_ffprobe() {
        return;
    }
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            feathered_image("book"),
            mover("mover"),
        ]),
    );
    rendered(&path, forced(1), false, false);
    let off = rendered_with_missing_still(&path, forced(1), false, true, true);
    for painters in [0, 1, 3] {
        let on = rendered_with_missing_still(&path, forced(painters), true, true, true);
        assert_eq!(on.frames, off.frames);
        assert_eq!(on.comparable(), off.comparable());
        assert!(on.answer["render"].is_null(), "{}", on.answer);
        assert!(
            on.answer.to_string().contains("E-NOT-PAINTED-UNREADABLE"),
            "{}",
            on.answer
        );
    }
}

#[test]
fn a_video_above_the_prefix_keeps_its_feed_bookkeeping_and_frames() {
    if !has_ffprobe() {
        return;
    }
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/skills/media/clip.mp4");
    let video = json!({"id": "video", "type": "video", "start": 0, "end": 600,
        "source": source, "source_start": 0, "source_end": 600, "volume": 0,
        "x": 100, "y": 50, "origin": "top-left", "width": 64, "height": 48,
        "fit": "literal"});
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            feathered_image("book"),
            video,
        ]),
    );
    let on = identical_with_and_without(&path);
    assert_eq!(on.row("table")["frames_reused"], 17);
    assert_eq!(on.row("book")["frames_reused"], 17);
    assert_eq!(on.row("video")["frames_reused"], 0);
    assert_eq!(on.row("video")["frames_painted"], 18);
}

#[test]
fn a_blended_layer_ends_the_prefix() {
    if !has_ffprobe() {
        return;
    }
    let mut blended = plate("blended", 0, 600, "#2E6A9F80");
    blended["blend"] = json!("multiply");
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            blended,
            plate("cap", 0, 600, "#FFFFFF20"),
        ]),
    );
    let on = identical_with_and_without(&path);
    assert_eq!(on.row("table")["frames_reused"], 17);
    for id in ["blended", "cap"] {
        assert_eq!(on.row(id)["frames_reused"], 0);
        assert_eq!(on.row(id)["frames_painted"], 18);
    }
}

#[test]
fn a_group_member_ends_the_prefix_even_when_its_pixels_are_static() {
    if !has_ffprobe() {
        return;
    }
    let mut grouped = plate("grouped", 0, 600, "#2E6A9F");
    grouped["group"] = json!("plates");
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            grouped,
            mover("mover"),
        ]),
    );
    let on = identical_with_and_without(&path);
    assert_eq!(on.row("table")["frames_reused"], 17);
    assert_eq!(on.row("grouped")["frames_reused"], 0);
    assert_eq!(on.row("grouped")["frames_painted"], 18);
}

#[test]
fn static_plates_under_a_mover_are_restored_and_every_frame_is_the_same() {
    if !has_ffprobe() {
        return;
    }
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            feathered_image("book"),
            mover("mover"),
            plate("cap", 0, 600, "#00000040"),
        ]),
    );
    let on = identical_with_and_without(&path);
    // Painted once (the miss), reused on the other 17 frames, for each plate.
    for id in ["table", "book"] {
        let row = on.row(id);
        assert_eq!(row["frames_painted"], 1, "{row}");
        assert_eq!(row["frames_reused"], 17, "{row}");
    }
    // The mover ends the prefix, and the layer above it is not in it.
    for id in ["mover", "cap"] {
        let row = on.row(id);
        assert_eq!(row["frames_painted"], 18, "{row}");
        assert_eq!(row["frames_reused"], 0, "{row}");
    }
}

/// #974: an audio element on a layer below static plates draws nothing, so it neither
/// joins nor ends the static prefix; the plates are restored as if it were not there.
#[test]
fn audio_below_static_plates_does_not_end_the_prefix() {
    if !has_ffprobe() {
        return;
    }
    let path = write(
        line!(),
        stacked(vec![
            audio("voice", 0, 600),
            plate("table", 0, 600, "#3A2A1E"),
            feathered_image("book"),
            mover("mover"),
        ]),
    );
    tone(&path, 600);
    let on = identical_with_and_without(&path);
    for id in ["table", "book"] {
        let row = on.row(id);
        assert_eq!(row["frames_painted"], 1, "{row}");
        assert_eq!(row["frames_reused"], 17, "{row}");
    }
    listed_alike(&path, &on, "voice");
}

/// #974: an audio clip that starts and stops mid-span is not part of the prefix's key, so its
/// entry and exit cost the plates nothing — unlike a drawn element entering
/// (`an_element_that_enters_mid_span_starts_a_new_prefix`).
#[test]
fn audio_entering_and_leaving_mid_span_does_not_restart_the_prefix() {
    if !has_ffprobe() {
        return;
    }
    let path = write(
        line!(),
        stacked(vec![
            audio("voice", 200, 400),
            plate("table", 0, 600, "#3A2A1E"),
            feathered_image("book"),
            mover("mover"),
        ]),
    );
    tone(&path, 200);
    let on = identical_with_and_without(&path);
    for id in ["table", "book"] {
        let row = on.row(id);
        assert_eq!(row["frames_painted"], 1, "{row}");
        assert_eq!(row["frames_reused"], 17, "{row}");
    }
    listed_alike(&path, &on, "voice");
}

/// #974: a `transition` element draws nothing of its own, so on a layer below plates it does
/// not end their prefix. (The elements it bridges are still excluded while it runs:
/// `plates_bridged_by_a_transition_are_not_cached_while_it_runs`.)
#[test]
fn a_transition_element_below_static_plates_does_not_end_the_prefix() {
    if !has_ffprobe() {
        return;
    }
    let transition = json!({"id": "x", "type": "transition", "start": 200, "end": 400,
                            "kind": "crossfade", "from": "one", "to": "two"});
    let path = write(
        line!(),
        stacked(vec![
            transition,
            plate("table", 0, 600, "#3A2A1E"),
            feathered_image("book"),
            mover("mover"),
            plate("one", 0, 400, "#2E6A9F"),
            plate("two", 200, 600, "#9F2E6A"),
        ]),
    );
    let on = identical_with_and_without(&path);
    for id in ["table", "book"] {
        let row = on.row(id);
        assert_eq!(row["frames_painted"], 1, "{row}");
        assert_eq!(row["frames_reused"], 17, "{row}");
    }
    listed_alike(&path, &on, "x");
}

#[test]
fn a_layer_with_an_opacity_keyframe_is_never_wrongly_cached() {
    if !has_ffprobe() {
        return;
    }
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            keyed_opacity("fading"),
            // Static, but above a keyed layer: not part of the bottom prefix.
            plate("cap", 0, 600, "#FFFFFF20"),
        ]),
    );
    let on = identical_with_and_without(&path);
    assert_eq!(on.row("table")["frames_reused"], 17);
    for id in ["fading", "cap"] {
        let row = on.row(id);
        assert_eq!(row["frames_painted"], 18, "{row}");
        assert_eq!(row["frames_reused"], 0, "{row}");
    }
}

#[test]
fn an_element_that_enters_mid_span_starts_a_new_prefix() {
    if !has_ffprobe() {
        return;
    }
    // `late` enters at frame 6 (200 ms) and sits above `table`: two prefix intervals.
    let path = write(
        line!(),
        stacked(vec![
            plate("table", 0, 600, "#3A2A1E"),
            plate("late", 200, 600, "#2E6A9F"),
            mover("mover"),
        ]),
    );
    let on = identical_with_and_without(&path);
    // table: frames 0..6 (the first prefix interval), restored 5; then 6..18 with `late`.
    let table = on.row("table");
    assert_eq!(table["frames_painted"], 2, "{table}");
    assert_eq!(table["frames_reused"], 16, "{table}");
    let late = on.row("late");
    assert_eq!(late["frames_painted"], 1, "{late}");
    assert_eq!(late["frames_reused"], 11, "{late}");
}

#[test]
fn plates_bridged_by_a_transition_are_not_cached_while_it_runs() {
    if !has_ffprobe() {
        return;
    }
    let transition = json!({"id": "x", "type": "transition", "start": 200, "end": 400,
                            "kind": "crossfade", "from": "one", "to": "two"});
    let path = write(
        line!(),
        stacked(vec![
            plate("one", 0, 400, "#3A2A1E"),
            plate("two", 200, 600, "#2E6A9F"),
            transition,
            mover("mover"),
        ]),
    );
    let on = identical_with_and_without(&path);
    assert!(on.reused() > 0, "{}", on.answer);
    // `one` is bridged for frames 6..12 and so cannot be restored there.
    let one = on.row("one");
    let painted = one["frames_painted"].as_u64().unwrap();
    assert!(painted >= 6, "{one}");
}

#[test]
fn a_scaled_preview_surface_restores_the_cache_at_its_own_scale() {
    // `preview` paints on a smaller surface under a base scale; a restore that drew the
    // cached pixels through that matrix would scale them again.
    use montagent_core::verbs::preview::{Ask, preview};
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    plate_png(&dir);
    let tracks = stacked(vec![
        plate("table", 0, 200, "#3A2A1E"),
        feathered_image("book"),
        mover("mover"),
    ]);
    let project = json!({
        "frame": {"width": 2000, "height": 1000}, "fps": 30, "background": "#101418",
        "duration": 200, "output": "out/prefix.mp4", "tracks": tracks,
    });
    // The plate fills 320x180 of a 2000x1000 frame; the image and mover sit inside it.
    let path = write_project(
        &dir,
        "prefix.montagent.json",
        &canonical(&project.to_string()),
    );
    let previewed = |cache: bool| {
        let _cache = force_prefix_cache(cache);
        let tap = tap_frames();
        let answer = preview(&path, &Ask::default(), &mut |_| {});
        (tap.hashes(), answer.to_json())
    };
    let (on, on_answer) = previewed(true);
    let (off, _) = previewed(false);
    assert!(!on.is_empty(), "{on_answer}");
    assert_eq!(on, off, "{on_answer}");
}
