//! Integration tests for semantic audio roles and decoupled Broadcast WAV stem export (ADR-0204).

use std::path::Path;
use std::process::Command;

use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Ask, render};
use serde_json::Value;

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

fn make_tone(path: &Path, freq: u32, duration_s: u32) {
    let tools = montagent_core::media::tools::resolve().expect("ffmpeg");
    let status = Command::new(tools.ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency={freq}:sample_rate=48000:duration={duration_s}"),
            "-c:a",
            "pcm_s16le",
        ])
        .arg(path)
        .status()
        .expect("ffmpeg runs");
    assert!(status.success(), "make_tone failed");
}

fn probe_audio(path: &Path) -> (String, u32, u32) {
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
    let audio = json["streams"]
        .as_array()
        .expect("streams array")
        .iter()
        .find(|s| s["codec_type"].as_str() == Some("audio"))
        .expect("audio stream");

    let codec = audio["codec_name"].as_str().unwrap_or("").to_string();
    let sample_rate: u32 = audio["sample_rate"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let channels: u32 = audio["channels"].as_u64().unwrap_or(0) as u32;
    (codec, sample_rate, channels)
}

#[test]
fn multitrack_render_exports_all_stems() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe");
        return;
    }

    let dir = tempdir(line!());
    let dia_src = dir.join("dia.wav");
    let mus_src = dir.join("mus.wav");
    make_tone(&dia_src, 440, 2);
    make_tone(&mus_src, 880, 2);

    let doc = format!(
        r##"{{
            "frame": {{"width": 640, "height": 360}},
            "fps": 25,
            "duration": 2000,
            "output": {{
                "path": "out/master.mp4",
                "stems": [
                    {{"role": "dialogue", "path": "stems/dialogue.wav"}},
                    {{"role": "music", "path": "stems/music.wav"}},
                    {{"role": "m_and_e", "path": "stems/m_and_e.wav"}}
                ]
            }},
            "tracks": [
                {{
                    "name": "dialogue_track",
                    "role": "dialogue",
                    "layer": 0,
                    "elements": [
                        {{
                            "id": "dia_elem",
                            "type": "audio",
                            "start": 0,
                            "end": 2000,
                            "source": "{dia_str}",
                            "source_start": 0,
                            "source_end": 2000
                        }}
                    ]
                }},
                {{
                    "name": "music_track",
                    "role": "music",
                    "layer": 1,
                    "elements": [
                        {{
                            "id": "mus_elem",
                            "type": "audio",
                            "start": 0,
                            "end": 2000,
                            "source": "{mus_str}",
                            "source_start": 0,
                            "source_end": 2000
                        }}
                    ]
                }}
            ]
        }}"##,
        dia_str = dia_src.display().to_string().replace('\\', "/"),
        mus_str = mus_src.display().to_string().replace('\\', "/")
    );

    let project_file = write_project(&dir, "project.json", &canonical(&doc));
    let answer = render(&project_file, &Ask::default(), &mut |_| ());

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "render failed: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );

    assert!(
        dir.join("out/master.mp4").exists(),
        "master deliverable must exist"
    );

    let dia_stem = dir.join("stems/dialogue.wav");
    let mus_stem = dir.join("stems/music.wav");
    let me_stem = dir.join("stems/m_and_e.wav");

    assert!(dia_stem.exists(), "dialogue stem must exist");
    assert!(mus_stem.exists(), "music stem must exist");
    assert!(me_stem.exists(), "m_and_e stem must exist");

    for stem_path in [&dia_stem, &mus_stem, &me_stem] {
        let (codec, rate, channels) = probe_audio(stem_path);
        assert_eq!(codec, "pcm_s24le", "stem must be 24-bit PCM WAV");
        assert_eq!(rate, 48000, "stem must be 48 kHz");
        assert_eq!(channels, 2, "stem must be stereo");
    }

    let json = answer.to_json();
    let stems = json["render"]["stems"]
        .as_array()
        .expect("stems array in render report");
    assert_eq!(stems.len(), 3, "must record 3 stems in report");
    for stem in stems {
        assert!(
            stem["bytes"].as_u64().unwrap_or(0) > 0,
            "stem bytes must be non-zero"
        );
    }
}

#[test]
fn stems_only_bypasses_video_rasterization() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe");
        return;
    }

    let dir = tempdir(line!());
    let dia_src = dir.join("dia.wav");
    make_tone(&dia_src, 440, 2);

    let doc = format!(
        r##"{{
            "frame": {{"width": 640, "height": 360}},
            "fps": 25,
            "duration": 2000,
            "output": {{
                "path": "out/master.mp4",
                "stems": [
                    {{"role": "dialogue", "path": "stems/dialogue.wav"}}
                ]
            }},
            "tracks": [
                {{
                    "name": "dialogue_track",
                    "role": "dialogue",
                    "layer": 0,
                    "elements": [
                        {{
                            "id": "dia_elem",
                            "type": "audio",
                            "start": 0,
                            "end": 2000,
                            "source": "{dia_str}",
                            "source_start": 0,
                            "source_end": 2000
                        }}
                    ]
                }}
            ]
        }}"##,
        dia_str = dia_src.display().to_string().replace('\\', "/")
    );

    let project_file = write_project(&dir, "project.json", &canonical(&doc));
    let ask = Ask {
        stems_only: true,
        ..Default::default()
    };
    let answer = render(&project_file, &ask, &mut |_| ());

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "stems-only render failed: {}",
        serde_json::to_string_pretty(&answer.to_json()).unwrap_or_default()
    );

    // Master video file was NOT created
    assert!(
        !dir.join("out/master.mp4").exists(),
        "master mp4 must not exist in stems-only mode"
    );

    // But stem WAV was created
    let dia_stem = dir.join("stems/dialogue.wav");
    assert!(dia_stem.exists(), "dialogue stem must exist");
    let (codec, rate, channels) = probe_audio(&dia_stem);
    assert_eq!(codec, "pcm_s24le");
    assert_eq!(rate, 48000);
    assert_eq!(channels, 2);

    let json = answer.to_json();
    assert_eq!(json["render"]["frames"], 0, "frames rasterized must be 0");
    assert_eq!(
        json["render"]["bytes"], 0,
        "video deliverable bytes must be 0"
    );
    let stems = json["render"]["stems"].as_array().expect("stems in json");
    assert_eq!(stems.len(), 1);
    assert_eq!(stems[0]["role"], "dialogue");
}

#[test]
fn element_role_overrides_track_role() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe");
        return;
    }

    let dir = tempdir(line!());
    let src = dir.join("clip.wav");
    make_tone(&src, 500, 2);

    let doc = format!(
        r##"{{
            "frame": {{"width": 640, "height": 360}},
            "fps": 25,
            "duration": 2000,
            "output": {{
                "path": "out/master.mp4",
                "stems": [
                    {{"role": "voiceover", "path": "stems/vo.wav"}},
                    {{"role": "dialogue", "path": "stems/dia.wav"}}
                ]
            }},
            "tracks": [
                {{
                    "name": "dia_track",
                    "role": "dialogue",
                    "layer": 0,
                    "elements": [
                        {{
                            "id": "vo_override",
                            "role": "voiceover",
                            "type": "audio",
                            "start": 0,
                            "end": 1000,
                            "source": "{src_str}",
                            "source_start": 0,
                            "source_end": 1000
                        }},
                        {{
                            "id": "dia_inherited",
                            "type": "audio",
                            "start": 1000,
                            "end": 2000,
                            "source": "{src_str}",
                            "source_start": 0,
                            "source_end": 1000
                        }}
                    ]
                }}
            ]
        }}"##,
        src_str = src.display().to_string().replace('\\', "/")
    );

    let project_file = write_project(&dir, "project.json", &canonical(&doc));
    let ask = Ask {
        stems_only: true,
        ..Default::default()
    };
    let answer = render(&project_file, &ask, &mut |_| ());

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);

    let vo_stem = dir.join("stems/vo.wav");
    let dia_stem = dir.join("stems/dia.wav");
    assert!(vo_stem.exists());
    assert!(dia_stem.exists());

    // Both files exist and are valid 24-bit 48kHz Broadcast WAV
    let (vo_codec, vo_rate, _) = probe_audio(&vo_stem);
    assert_eq!(vo_codec, "pcm_s24le");
    assert_eq!(vo_rate, 48000);

    let (dia_codec, dia_rate, _) = probe_audio(&dia_stem);
    assert_eq!(dia_codec, "pcm_s24le");
    assert_eq!(dia_rate, 48000);
}

#[test]
fn empty_stem_generates_valid_silent_broadcast_wav() {
    if !has_ffprobe() {
        eprintln!("skipped: no ffprobe");
        return;
    }

    let dir = tempdir(line!());
    let src = dir.join("foley_src.wav");
    make_tone(&src, 300, 1);

    let doc = format!(
        r##"{{
        "frame": {{"width": 640, "height": 360}},
        "fps": 25,
        "duration": 4000,
        "output": {{
            "path": "out/master.mp4",
            "stems": [
                {{"role": "foley", "path": "stems/foley.wav"}}
            ]
        }},
        "tracks": [
            {{
                "name": "foley_track",
                "role": "foley",
                "layer": 0,
                "elements": [
                    {{
                        "id": "late_foley",
                        "type": "audio",
                        "start": 3000,
                        "end": 4000,
                        "source": "{src_str}",
                        "source_start": 0,
                        "source_end": 1000
                    }}
                ]
            }}
        ]
    }}"##,
        src_str = src.display().to_string().replace('\\', "/")
    );

    let project_file = write_project(&dir, "project.json", &canonical(&doc));
    let ask = Ask {
        stems_only: true,
        from: Some(0),
        to: Some(2000),
        ..Default::default()
    };
    let answer = render(&project_file, &ask, &mut |_| ());

    assert_eq!(answer.report().exit_code(), ExitCode::Ok);

    let foley_stem = dir.join("stems/foley.wav");
    assert!(foley_stem.exists(), "silent foley stem must exist");
    let (codec, rate, channels) = probe_audio(&foley_stem);
    assert_eq!(codec, "pcm_s24le");
    assert_eq!(rate, 48000);
    assert_eq!(channels, 2);
}

#[test]
fn stems_only_refuses_when_no_stems_declared() {
    let dir = tempdir(line!());
    let doc = r##"{
        "frame": {"width": 640, "height": 360},
        "fps": 25,
        "duration": 1000,
        "output": "out/master.mp4",
        "tracks": [
            {
                "name": "bg",
                "layer": 0,
                "elements": [
                    {
                        "id": "box",
                        "type": "rect",
                        "start": 0,
                        "end": 1000,
                        "x": 0,
                        "y": 0,
                        "width": 640,
                        "height": 360,
                        "fill": "#000000"
                    }
                ]
            }
        ]
    }"##;

    let project_file = write_project(&dir, "project.json", &canonical(doc));
    let ask = Ask {
        stems_only: true,
        ..Default::default()
    };
    let answer = render(&project_file, &ask, &mut |_| ());

    assert_eq!(
        answer.report().exit_code(),
        ExitCode::BadInvocation,
        "stems-only must reject if output.stems is empty"
    );
}
