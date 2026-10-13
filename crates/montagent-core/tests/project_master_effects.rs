//! Project-level master effects (ADR-0205).
//!
//! - Visual effects applied to the full composite at the device boundary.
//! - Static prefix cache invariance: master effects run post-composite without invalidating prefix cache.
//! - Static validation: masks and keyframes refused at project level.
//! - Query introspection: `query --at` reports project-level master effects.

use std::path::Path;

use montagent_core::report::ExitCode;
use montagent_core::verbs::frame::{Ask, frame};
use montagent_core::verbs::query::{self, Ask as QueryAsk};
use montagent_core::verbs::validate::validate;
use serde_json::json;

mod common;
use common::{canonical, tempdir, write_project};

fn drawn(project: &Path, ask: &Ask) -> (serde_json::Value, Vec<u8>) {
    let answer = frame(project, ask);
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "frame did not answer: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    let bytes = answer
        .image()
        .expect("an answer with exit 0 carries a picture")
        .bytes
        .clone();
    (answer.to_json(), bytes)
}

fn pixels(bytes: &[u8]) -> image::RgbaImage {
    image::load_from_memory(bytes)
        .expect("the bytes decode as a picture")
        .to_rgba8()
}

#[test]
fn project_master_effects_grades_full_frame() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "background": "#FFFFFF",
        "effects": [
            {"name": "brightness", "amount": -0.5}
        ],
        "tracks": []
    });
    let project = write_project(
        &dir,
        "master_effect.montagent.json",
        &canonical(&doc.to_string()),
    );

    let (_, bytes) = drawn(
        &project,
        &Ask {
            full: true,
            at: Some(0),
            ..Default::default()
        },
    );
    let pic = pixels(&bytes);
    let p = pic.get_pixel(50, 50).0;
    // Brightness at -0.5 dims white (255) to ~128
    assert_eq!(p[0], 128);
    assert_eq!(p[1], 128);
    assert_eq!(p[2], 128);
    assert_eq!(p[3], 255);
}

#[test]
fn project_master_effects_refuses_mask() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "effects": [
            {"name": "mask", "shape": "rect"}
        ],
        "tracks": []
    });
    let project = write_project(
        &dir,
        "mask_refused.montagent.json",
        &canonical(&doc.to_string()),
    );

    let report = validate(&project);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-PROJECT-EFFECT-MASK"),
        "expected E-PROJECT-EFFECT-MASK in report: {:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}

#[test]
fn project_master_effects_refuses_keyframes() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "effects": [
            {
                "name": "brightness",
                "amount": [
                    {"t": 0, "v": 0.0},
                    {"t": 1000, "v": -0.5, "ease": "linear"}
                ]
            }
        ],
        "tracks": []
    });
    let project = write_project(
        &dir,
        "keyframe_refused.montagent.json",
        &canonical(&doc.to_string()),
    );

    let report = validate(&project);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-PROJECT-EFFECT-KEYFRAME"),
        "expected E-PROJECT-EFFECT-KEYFRAME in report: {:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}

#[test]
fn project_master_effects_refuses_audio_effects() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "effects": [
            {
                "name": "compressor",
                "threshold_db": -20.0,
                "ratio": 4.0,
                "attack_ms": 20.0,
                "release_ms": 250.0,
                "makeup_db": 0.0
            }
        ],
        "tracks": []
    });
    let project = write_project(
        &dir,
        "audio_in_effects.montagent.json",
        &canonical(&doc.to_string()),
    );

    let report = validate(&project);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-AUDIO-EFFECT-WRONG-LIST"),
        "expected E-AUDIO-EFFECT-WRONG-LIST in report: {:?}",
        report.findings.iter().map(|f| &f.code).collect::<Vec<_>>()
    );
}

#[test]
fn query_at_reports_project_master_effects() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 100, "height": 100},
        "fps": 25,
        "effects": [
            {"name": "saturation", "amount": 1.5}
        ],
        "tracks": []
    });
    let project = write_project(
        &dir,
        "query_effects.montagent.json",
        &canonical(&doc.to_string()),
    );

    let answer = query::query(
        &project,
        &QueryAsk {
            at: Some(0),
            ..Default::default()
        },
    );
    let at_val = answer.to_json();
    let effects = at_val["query"]["effects"]
        .as_array()
        .expect("effects array in At");
    assert_eq!(effects.len(), 1);
    assert_eq!(
        effects[0].get("name").and_then(|v| v.as_str()),
        Some("saturation")
    );
}

#[test]
fn project_master_effects_preserves_prefix_cache() {
    if !common::has_ffprobe() {
        return;
    }
    use montagent_core::verbs::render::{Ask as RenderAsk, render};

    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 160, "height": 90},
        "fps": 30,
        "background": "#000000",
        "duration": 300,
        "output": "out.mp4",
        "effects": [
            {"name": "brightness", "amount": 0.2}
        ],
        "tracks": [
            {
                "name": "bg",
                "layer": 0,
                "elements": [
                    {
                        "id": "bg_plate",
                        "type": "rect",
                        "start": 0,
                        "end": 300,
                        "x": 0,
                        "y": 0,
                        "width": 160,
                        "height": 90,
                        "fill": "#334455"
                    }
                ]
            },
            {
                "name": "fg",
                "layer": 1,
                "elements": [
                    {
                        "id": "fg_box",
                        "type": "rect",
                        "start": 0,
                        "end": 300,
                        "x": [
                            {"t": 0, "v": 0},
                            {"t": 300, "v": 50, "ease": "linear"}
                        ],
                        "y": 10,
                        "width": 20,
                        "height": 20,
                        "fill": "#FFFFFF"
                    }
                ]
            }
        ]
    });
    let project = write_project(
        &dir,
        "prefix_master.montagent.json",
        &canonical(&doc.to_string()),
    );

    let answer = render(
        &project,
        &RenderAsk {
            profile: true,
            ..RenderAsk::default()
        },
        &mut |_| {},
    );

    let val = answer.to_json();
    assert_eq!(
        val["exit_code"],
        0,
        "render did not exit 0: {}",
        serde_json::to_string_pretty(&val).unwrap_or_default()
    );

    let elements = val["render"]["profile"]["elements"]
        .as_array()
        .expect("profile elements");
    let bg_row = elements
        .iter()
        .find(|row| row["id"] == "bg_plate")
        .expect("bg_plate in profile");
    let reused = bg_row["frames_reused"].as_u64().unwrap_or(0);
    assert!(
        reused > 0,
        "expected bg_plate to have reused frames under master effects, got {}",
        reused
    );
}
