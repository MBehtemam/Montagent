//! Integration tests for ProRes and DNxHR mezzanine encoders with alpha transparency (ADR-0202, ADR-0203).

use std::path::Path;
use std::process::Command;

use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Ask, render};
use serde_json::Value;

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

fn probe_stream(path: &Path) -> (String, String, String) {
    let tools = montagent_core::media::tools::resolve().expect("ffprobe");
    let out = Command::new(tools.ffprobe)
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(path)
        .output()
        .expect("run ffprobe");
    assert!(
        out.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: Value = serde_json::from_slice(&out.stdout).expect("parse ffprobe json");
    let video = json["streams"]
        .as_array()
        .expect("streams array")
        .iter()
        .find(|s| s["codec_type"].as_str() == Some("video"))
        .expect("video stream");

    let codec = video["codec_name"].as_str().unwrap_or("").to_string();
    let profile = video["profile"].as_str().unwrap_or("").to_string();
    let pix_fmt = video["pix_fmt"].as_str().unwrap_or("").to_string();
    (codec, profile, pix_fmt)
}

#[test]
fn prores_4444_with_alpha_deliverable() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe");
        return;
    }

    let dir = tempdir(line!());
    let out_path = dir.join("prores_alpha.mov");
    let out_path_str = out_path.display().to_string();

    let doc = format!(
        r##"{{
            "frame": {{"width": 1280, "height": 720}},
            "fps": 25,
            "duration": 400,
            "output": {{
                "path": "{out_path_str}",
                "container": "mov",
                "video": {{
                    "codec": "prores",
                    "profile": "4444",
                    "alpha": true
                }}
            }},
            "tracks": [
                {{
                    "name": "overlay",
                    "layer": 0,
                    "elements": [
                        {{
                            "id": "box",
                            "type": "rect",
                            "start": 0,
                            "end": 400,
                            "x": 100,
                            "y": 100,
                            "width": 200,
                            "height": 200,
                            "fill": "#FF0000"
                        }}
                    ]
                }}
            ]
        }}"##
    );

    let project_file = write_project(&dir, "project.json", &canonical(&doc));
    let answer = render(&project_file, &Ask::default(), &mut |_| ());

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "render failed: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    assert!(out_path.exists(), "deliverable must exist at target path");

    let (codec, profile, pix_fmt) = probe_stream(&out_path);
    assert_eq!(codec, "prores", "video codec must be prores");
    assert!(
        profile.contains("4444"),
        "profile must be 4444, got: {profile}"
    );
    assert!(
        pix_fmt.starts_with("yuva444p"),
        "pixel format must carry alpha (yuva444p*), got: {pix_fmt}"
    );
}

#[test]
fn prores_standard_deliverable() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe");
        return;
    }

    let dir = tempdir(line!());
    let out_path = dir.join("prores_std.mov");
    let out_path_str = out_path.display().to_string();

    let doc = format!(
        r##"{{
            "frame": {{"width": 1280, "height": 720}},
            "fps": 25,
            "duration": 400,
            "output": {{
                "path": "{out_path_str}",
                "container": "mov",
                "video": {{
                    "codec": "prores",
                    "profile": "standard"
                }}
            }},
            "tracks": [
                {{
                    "name": "base",
                    "layer": 0,
                    "elements": [
                        {{
                            "id": "box",
                            "type": "rect",
                            "start": 0,
                            "end": 400,
                            "x": 50,
                            "y": 50,
                            "width": 100,
                            "height": 100,
                            "fill": "#00FF00"
                        }}
                    ]
                }}
            ]
        }}"##
    );

    let project_file = write_project(&dir, "project.json", &canonical(&doc));
    let answer = render(&project_file, &Ask::default(), &mut |_| ());

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "render failed: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    assert!(out_path.exists());

    let (codec, profile, pix_fmt) = probe_stream(&out_path);
    assert_eq!(codec, "prores");
    assert!(
        profile.contains("Standard") || profile.contains("standard") || profile == "2",
        "profile should be standard, got: {profile}"
    );
    assert_eq!(pix_fmt, "yuv422p10le");
}

#[test]
fn dnxhr_hq_mxf_deliverable() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe");
        return;
    }

    let dir = tempdir(line!());
    let out_path = dir.join("dnxhr.mxf");
    let out_path_str = out_path.display().to_string();

    let doc = format!(
        r##"{{
            "frame": {{"width": 1280, "height": 720}},
            "fps": 25,
            "duration": 400,
            "output": {{
                "path": "{out_path_str}",
                "container": "mxf",
                "video": {{
                    "codec": "dnxhr",
                    "profile": "hq"
                }}
            }},
            "tracks": [
                {{
                    "name": "base",
                    "layer": 0,
                    "elements": [
                        {{
                            "id": "box",
                            "type": "rect",
                            "start": 0,
                            "end": 400,
                            "x": 0,
                            "y": 0,
                            "width": 200,
                            "height": 200,
                            "fill": "#0000FF"
                        }}
                    ]
                }}
            ]
        }}"##
    );

    let project_file = write_project(&dir, "project.json", &canonical(&doc));
    let answer = render(&project_file, &Ask::default(), &mut |_| ());

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "render failed: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );
    assert!(out_path.exists());

    let (codec, profile, pix_fmt) = probe_stream(&out_path);
    assert_eq!(codec, "dnxhd");
    assert_eq!(pix_fmt, "yuv422p");
    let lower_profile = profile.to_ascii_lowercase();
    assert!(
        lower_profile.contains("dnxhr") && lower_profile.contains("hq"),
        "DNxHR profile mismatch: {profile}"
    );
}
