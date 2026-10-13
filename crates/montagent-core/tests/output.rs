//! Integration tests for Output deliverable configuration and validation (ADR-0202, ADR-0203, ADR-0204).

use montagent_core::model::{Project, infer_container};
use montagent_core::report::ExitCode;
use montagent_core::validate;
use serde_json::json;

mod common;
use common::{tempdir, write_project};

fn scaffold_json(output: serde_json::Value) -> String {
    json!({
        "frame": { "width": 1920, "height": 1080 },
        "fps": 30,
        "duration": 5000,
        "output": output,
        "tracks": []
    })
    .to_string()
}

#[test]
fn container_inference_resolves_known_extensions() {
    assert_eq!(infer_container("renders/final.mp4"), Some("mp4"));
    assert_eq!(infer_container("renders/FINAL.MP4"), Some("mp4"));
    assert_eq!(infer_container("exports/master.mov"), Some("mov"));
    assert_eq!(infer_container("exports/broadcast.mxf"), Some("mxf"));
    assert_eq!(infer_container("exports/unknown.avi"), None);
    assert_eq!(infer_container("exports/no_ext"), None);
}

#[test]
fn output_string_shorthand_deserializes_and_preserves_compatibility() {
    let json_str = scaffold_json(json!("deliverables/final.mp4"));
    let project: Project = serde_json::from_str(&json_str).expect("deserializes shorthand string");

    let output = project.output.expect("output is present");
    assert_eq!(output.path(), "deliverables/final.mp4");
    assert_eq!(output.container(), Some("mp4"));
    assert!(output.video().is_none());
    assert!(output.audio().is_none());
    assert!(output.stems().is_empty());
}

#[test]
fn output_structured_config_deserializes_all_fields() {
    let json_str = scaffold_json(json!({
        "path": "deliverables/master.mov",
        "container": "mov",
        "video": {
            "codec": "prores",
            "profile": "4444",
            "alpha": true
        },
        "audio": {
            "codec": "pcm",
            "sample_rate": 48000,
            "bitrate": "1536k"
        },
        "stems": [
            { "role": "dialogue", "path": "stems/dx.wav" },
            { "role": "music", "path": "stems/mx.wav" }
        ]
    }));

    let project: Project = serde_json::from_str(&json_str).expect("deserializes structured output");
    let output = project.output.expect("output is present");
    assert_eq!(output.path(), "deliverables/master.mov");
    assert_eq!(output.container(), Some("mov"));

    let video = output.video().expect("video config present");
    assert_eq!(video.codec.as_deref(), Some("prores"));
    assert_eq!(video.profile.as_deref(), Some("4444"));
    assert_eq!(video.alpha, Some(true));

    let audio = output.audio().expect("audio config present");
    assert_eq!(audio.codec.as_deref(), Some("pcm"));
    assert_eq!(audio.sample_rate, Some(48000));
    assert_eq!(audio.bitrate.as_deref(), Some("1536k"));

    let stems = output.stems();
    assert_eq!(stems.len(), 2);
    assert_eq!(stems[0].role, "dialogue");
    assert_eq!(stems[0].path, "stems/dx.wav");
    assert_eq!(stems[1].role, "music");
    assert_eq!(stems[1].path, "stems/mx.wav");
}

#[test]
fn validate_accepts_valid_shorthand_and_structured_output() {
    let dir = tempdir(line!());

    let path_str = write_project(
        &dir,
        "shorthand.montagent.json",
        &scaffold_json(json!("out.mp4")),
    );
    let report = validate(&path_str);
    assert_eq!(report.summary().error, 0, "findings: {:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);

    let fixture_wav = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures/skills/media/beat.wav");
    let _ = std::fs::copy(&fixture_wav, dir.join("audio.wav"));

    let path_cfg = write_project(
        &dir,
        "structured.montagent.json",
        &json!({
            "frame": { "width": 1920, "height": 1080 },
            "fps": 30,
            "duration": 5000,
            "output": {
                "path": "out.mov",
                "video": { "codec": "prores", "profile": "4444", "alpha": true },
                "audio": { "codec": "pcm" },
                "stems": [{ "role": "dialogue", "path": "dx.wav" }]
            },
            "tracks": [
                {
                    "name": "VO",
                    "layer": 1,
                    "role": "dialogue",
                    "elements": [
                        {
                            "id": "vo1",
                            "type": "audio",
                            "start": 0,
                            "end": 2000,
                            "source": "audio.wav",
                            "source_start": 0,
                            "source_end": 2000
                        }
                    ]
                }
            ]
        })
        .to_string(),
    );
    let report = validate(&path_cfg);
    assert_eq!(report.summary().error, 0, "findings: {:?}", report.findings);
    assert_eq!(report.exit_code(), ExitCode::Ok);
}

#[test]
fn validate_refuses_empty_output_path() {
    let dir = tempdir(line!());

    let path = write_project(&dir, "empty_str.montagent.json", &scaffold_json(json!("")));
    let report = validate(&path);
    assert!(
        report.findings.iter().any(|f| f.code == "E-OUTPUT-EMPTY"),
        "expected E-OUTPUT-EMPTY for empty string"
    );

    let path_ws = write_project(
        &dir,
        "whitespace.montagent.json",
        &scaffold_json(json!({ "path": "  " })),
    );
    let report = validate(&path_ws);
    assert!(
        report.findings.iter().any(|f| f.code == "E-OUTPUT-EMPTY"),
        "expected E-OUTPUT-EMPTY for whitespace path"
    );
}

#[test]
fn validate_refuses_extension_container_mismatch() {
    let dir = tempdir(line!());
    let path = write_project(
        &dir,
        "mismatch.montagent.json",
        &scaffold_json(json!({
            "path": "video.mp4",
            "container": "mov"
        })),
    );
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-OUTPUT-EXTENSION-MISMATCH"),
        "expected E-OUTPUT-EXTENSION-MISMATCH"
    );
}

#[test]
fn validate_refuses_unknown_extension_without_container() {
    let dir = tempdir(line!());
    let path = write_project(
        &dir,
        "unknown_ext.montagent.json",
        &scaffold_json(json!("video.avi")),
    );
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-OUTPUT-EXTENSION-UNKNOWN"),
        "expected E-OUTPUT-EXTENSION-UNKNOWN"
    );
}

#[test]
fn validate_refuses_incompatible_container_and_codec() {
    let dir = tempdir(line!());
    let path = write_project(
        &dir,
        "mp4_prores.montagent.json",
        &scaffold_json(json!({
            "path": "out.mp4",
            "video": { "codec": "prores" }
        })),
    );
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-CONTAINER-CODEC-MISMATCH"),
        "expected E-CONTAINER-CODEC-MISMATCH for MP4 with ProRes"
    );
}

#[test]
fn validate_refuses_invalid_codec_profile() {
    let dir = tempdir(line!());
    let path = write_project(
        &dir,
        "invalid_prof.montagent.json",
        &scaffold_json(json!({
            "path": "out.mov",
            "video": { "codec": "prores", "profile": "invalid_profile" }
        })),
    );
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-CODEC-PROFILE-INVALID"),
        "expected E-CODEC-PROFILE-INVALID"
    );
}

#[test]
fn validate_refuses_alpha_on_unsupported_codecs() {
    let dir = tempdir(line!());

    let path_h264 = write_project(
        &dir,
        "h264_alpha.montagent.json",
        &scaffold_json(json!({
            "path": "out.mp4",
            "video": { "codec": "h264", "alpha": true }
        })),
    );
    let report = validate(&path_h264);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-CODEC-ALPHA-UNSUPPORTED"),
        "expected E-CODEC-ALPHA-UNSUPPORTED for h264"
    );

    let path_proxy = write_project(
        &dir,
        "proxy_alpha.montagent.json",
        &scaffold_json(json!({
            "path": "out.mov",
            "video": { "codec": "prores", "profile": "proxy", "alpha": true }
        })),
    );
    let report = validate(&path_proxy);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-CODEC-ALPHA-UNSUPPORTED"),
        "expected E-CODEC-ALPHA-UNSUPPORTED for prores proxy"
    );
}

#[test]
fn validate_refuses_non_wav_stem_format() {
    let dir = tempdir(line!());
    let path = write_project(
        &dir,
        "bad_stem.montagent.json",
        &scaffold_json(json!({
            "path": "out.mov",
            "stems": [{ "role": "dialogue", "path": "stems/dx.mp3" }]
        })),
    );
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-STEM-FORMAT-INVALID"),
        "expected E-STEM-FORMAT-INVALID"
    );
}

#[test]
fn validate_refuses_colliding_stem_paths() {
    let dir = tempdir(line!());

    let path_main = write_project(
        &dir,
        "main_collision.montagent.json",
        &scaffold_json(json!({
            "path": "out.wav",
            "stems": [{ "role": "dialogue", "path": "out.wav" }]
        })),
    );
    let report = validate(&path_main);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-STEM-PATH-COLLISION"),
        "expected E-STEM-PATH-COLLISION with main output"
    );

    let path_between = write_project(
        &dir,
        "stem_collision.montagent.json",
        &scaffold_json(json!({
            "path": "out.mov",
            "stems": [
                { "role": "dialogue", "path": "stems/audio.wav" },
                { "role": "music", "path": "stems/audio.wav" }
            ]
        })),
    );
    let report = validate(&path_between);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-STEM-PATH-COLLISION"),
        "expected E-STEM-PATH-COLLISION between stems"
    );
}

#[test]
fn validate_refuses_empty_stem_role() {
    let dir = tempdir(line!());

    let fixture_wav = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures/skills/media/beat.wav");
    let _ = std::fs::copy(&fixture_wav, dir.join("audio.wav"));

    let path = write_project(
        &dir,
        "empty_stem_role.montagent.json",
        &json!({
            "frame": { "width": 1920, "height": 1080 },
            "fps": 30,
            "duration": 5000,
            "output": {
                "path": "out.mov",
                "stems": [{ "role": "dialogue", "path": "dx.wav" }]
            },
            "tracks": [
                {
                    "name": "SFX",
                    "layer": 1,
                    "role": "sfx",
                    "elements": [
                        {
                            "id": "sfx1",
                            "type": "audio",
                            "start": 0,
                            "end": 2000,
                            "source": "audio.wav",
                            "source_start": 0,
                            "source_end": 2000
                        }
                    ]
                }
            ]
        })
        .to_string(),
    );
    let report = validate(&path);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == "E-STEM-ROLE-EMPTY"),
        "expected E-STEM-ROLE-EMPTY when dialogue stem has no dialogue elements: {:?}",
        report.findings
    );
}
