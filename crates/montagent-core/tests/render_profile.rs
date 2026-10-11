//! `render`'s `profile` (ADR-0194, #931): the block says where the painters' CPU went, and
//! asking for it changes no byte.
//!
//! The oracle for "no byte" is the hash of every frame at the encoder's input
//! (`render::tap_frames`), the same one `tests/painters.rs` uses, with and without the flag,
//! on one painter and on three.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use montagent_core::verbs::render::{Ask, Forced, Progress, force_painting, render, tap_frames};
use serde_json::{Value, json};

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

const HUNG: Duration = Duration::from_secs(120);

struct Rendered {
    frames: Vec<u64>,
    answer: Value,
}

fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(HUNG).expect("the render finished")
}

fn rendered(path: &Path, forced: Forced, profile: bool) -> Rendered {
    let path = path.to_path_buf();
    within(move || {
        let _forced = force_painting(forced);
        let tap = tap_frames();
        let answer = render(
            &path,
            &Ask {
                profile,
                ..Ask::default()
            },
            &mut |_: Progress| {},
        );
        Rendered {
            frames: tap.hashes(),
            answer: answer.to_json(),
        }
    })
}

/// `rects` moving rectangles, the first of which carries a `blur` and a `shadow`, on 320x180
/// at 30 fps for 600 ms: 18 frames.
fn project(line: u32, rects: usize) -> PathBuf {
    let dir = tempdir(line);
    let tracks: Vec<Value> = (0..rects)
        .map(|i| {
            let effects = if i == 0 {
                json!([
                    {"name": "blur", "radius": 8},
                    {"name": "shadow", "dx": 0, "dy": 0, "radius": 12, "color": "#FF9F2E", "opacity": 0.8}
                ])
            } else {
                json!([])
            };
            json!({"name": format!("t{i}"), "layer": i, "elements": [{
                "id": format!("box-{i:02}"), "type": "rect", "start": 0, "end": 600,
                "x": [{"t": 0, "v": 20}, {"t": 600, "v": 200, "ease": "linear"}],
                "y": 20 + (i as i64 % 8) * 16, "origin": "top-left",
                "width": if i == 0 { 90 } else { 12 }, "height": if i == 0 { 90 } else { 8 },
                "fill": "#F2F2F2", "effects": effects,
            }]})
        })
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 600, "output": "out/profile.mp4", "tracks": tracks,
    });
    write_project(
        &dir,
        "profile.montagent.json",
        &canonical(&project.to_string()),
    )
}

/// `0` is the verb's own thread, the way a document with a `video` is painted; `n` is `n`
/// painters over three-frame chunks.
fn forced(painters: usize) -> Forced {
    if painters == 0 {
        return Forced::OnePainter;
    }
    Forced::Chunks {
        painters,
        chunk: 3,
        window_bytes: None,
    }
}

fn number(value: &Value) -> f64 {
    value.as_f64().expect("a number")
}

#[test]
fn a_render_without_profile_has_no_profile_block() {
    if !has_ffprobe() {
        return;
    }
    let plain = rendered(&project(line!(), 3), forced(1), false);
    assert!(
        plain.answer["render"].get("profile").is_none(),
        "{}",
        plain.answer
    );
}

#[test]
fn asking_for_a_profile_changes_no_frame_on_one_painter_or_three() {
    if !has_ffprobe() {
        return;
    }
    let path = project(line!(), 3);
    let baseline = rendered(&path, forced(1), false);
    assert_eq!(baseline.frames.len(), 18);
    for painters in [0, 1, 3] {
        let profiled = rendered(&path, forced(painters), true);
        assert_eq!(profiled.frames, baseline.frames, "{painters} painters");
        assert!(profiled.answer["render"]["profile"].is_object());
    }
}

#[test]
fn the_block_lists_elements_types_and_effects_and_accounts_for_the_paint_stage() {
    if !has_ffprobe() {
        return;
    }
    for painters in [0, 1, 3] {
        let profiled = rendered(&project(line!(), 3), forced(painters), true);
        let profile = &profiled.answer["render"]["profile"];

        let elements = profile["elements"].as_array().expect("element rows");
        let ids: Vec<&str> = elements.iter().map(|e| e["id"].as_str().unwrap()).collect();
        assert_eq!(ids.len(), 3, "{profile}");
        // Sorted by cost, and the blurred, shadowed box is the expensive one.
        assert_eq!(ids[0], "box-00", "{profile}");
        for row in elements {
            assert_eq!(row["type"], "rect");
            assert_eq!(row["frames_painted"], 18, "{row}");
        }
        assert!(profile.get("elements_remainder").is_none());

        let types = profile["types"].as_array().unwrap();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0]["frames_painted"], 54, "3 rects x 18 frames");

        let effects: Vec<&str> = profile["effects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["effect"].as_str().unwrap())
            .collect();
        assert!(
            effects.contains(&"blur") && effects.contains(&"shadow"),
            "{profile}"
        );
        for row in profile["effects"].as_array().unwrap() {
            assert_eq!(row["applications"], 18, "{row}");
        }

        // ADR-0194: the listed costs are at least 90% of the paint stage, and what is not
        // listed is named rather than dropped.
        assert!(number(&profile["attributed_pct"]) >= 90.0, "{profile}");
        assert!(profile["unattributed"]["label"].is_string());
        assert!(number(&profile["stages"]["paint_ms"]) > 0.0);
        for key in [
            "decode_ms",
            "readback_ms",
            "paint_wait_ms",
            "encode_wait_ms",
            "seal_ms",
        ] {
            assert!(profile["stages"][key].is_number(), "{key}");
        }
    }
}

#[test]
fn elements_past_the_top_rows_fold_into_one_remainder_row() {
    if !has_ffprobe() {
        return;
    }
    let profiled = rendered(&project(line!(), 30), forced(3), true);
    let profile = &profiled.answer["render"]["profile"];
    assert_eq!(
        profile["elements"].as_array().unwrap().len(),
        25,
        "{profile}"
    );
    assert_eq!(profile["elements_remainder"]["elements"], 5, "{profile}");
    assert_eq!(profile["elements_remainder"]["frames_painted"], 5 * 18);
}
