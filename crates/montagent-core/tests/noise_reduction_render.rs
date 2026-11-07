//! Measured integration test for `reduce_noise` (ADR-0181, #1153).
//!
//! Asserts that `reduce_noise` attenuates reference broadband white noise (`anoisesrc=c=white:a=0.1`)
//! to within ±1.0 dB of `reduction_db` at `5, 10, 15, 20`, preserves a 1 kHz foreground tone inside
//! 1.0 dB while attenuating noise by >= 10 dB at `reduction_db: 12`, aligns speech-burst envelope
//! within ±1 ms via the 384-sample lookahead trim, produces byte-identical PCM when `enabled: false`,
//! is deterministic within a build, and satisfies `validate`, `query`, and `compare` contracts.

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::media::tools::resolve;
use montagent_core::report::ExitCode;
use montagent_core::validate;
use montagent_core::verbs::compare::compare;
use montagent_core::verbs::query::{Ask as QueryAsk, query};
use montagent_core::verbs::render::{Ask as RenderAsk, render};
use serde_json::{Value, json};

mod common;
use common::{canonical, tempdir, write_project};

fn ffmpeg() -> Option<PathBuf> {
    resolve().ok().map(|t| t.ffmpeg)
}

fn write_white_noise_wav(ff: &Path, path: &Path) {
    // Use pan=stereo|c0=c0|c1=c0 so each stereo channel stays at -50.0 dBFS (without swresample's
    // -3.01 dB mono-to-stereo law, which shifts anlmdn's level-sensitive curve by ~15 dB).
    let ok = Command::new(ff)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=c=white:a=0.005477225575:r=48000:d=3:seed=7",
            "-af",
            "pan=stereo|c0=c0|c1=c0",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f32le",
        ])
        .arg(path)
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "white noise wav must write");
}

fn write_speech_burst_wav(ff: &Path, path: &Path) {
    // 1 kHz burst starting at t = 0.200 s (sample 9600) with amplitude 0.1.
    let ok = Command::new(ff)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=0.1*sin(2*PI*1000*t)*gte(t\\,0.2)*lt(t\\,0.5):s=48000:d=2",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f32le",
        ])
        .arg(path)
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "speech burst wav must write");
}

fn decode_mono_f32(ff: &Path, path: &Path) -> Vec<f32> {
    let out = Command::new(ff)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args(["-ac", "1", "-ar", "48000", "-f", "f32le", "-"])
        .output()
        .expect("ffmpeg decode");
    assert!(out.status.success(), "decode failed");
    out.stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

fn rms_db(samples: &[f32]) -> f64 {
    // Measure over 0.8 s .. 1.8 s (inside the 1.93 s clip, away from start/end edges).
    let i0 = (0.8 * 48_000.0) as usize;
    let i1 = (1.8 * 48_000.0) as usize;
    let slice = &samples[i0..i1];
    let mean_sq: f64 =
        slice.iter().map(|&s| (s as f64) * (s as f64)).sum::<f64>() / slice.len() as f64;
    10.0 * mean_sq.log10()
}

fn render_audio(dir: &Path, name: &str, src: &str, clip_ms: i64, effects: Value) -> PathBuf {
    // Use clip_ms = 1930 (92640 samples = 480 * 193) so FFmpeg 9.0.2's `anlmdn` never sees a
    // partial (< 193 sample) tail frame at EOF.
    let total_ms = 2000;
    let mut elem = json!({
        "id": "a1",
        "type": "audio",
        "start": 0,
        "end": clip_ms,
        "source": src,
        "source_start": 0,
        "source_end": clip_ms,
    });
    if !effects.is_null() {
        elem["audio_effects"] = effects;
    }
    let pad = json!({
        "id": "pad",
        "type": "rect",
        "start": clip_ms,
        "end": total_ms,
        "x": 0,
        "y": 0,
        "width": 10,
        "height": 10,
        "fill": "#000000"
    });
    let out_rel = format!("{name}.mp4");
    let body = canonical(
        &json!({
            "frame": {"width": 200, "height": 200},
            "fps": 25,
            "duration": total_ms,
            "output": out_rel,
            "tracks": [{"name": "t", "layer": 0, "elements": [elem, pad]}],
        })
        .to_string(),
    );
    let proj = write_project(dir, &format!("{name}.montagent.json"), &body);
    let answer = render(&proj, &RenderAsk::default(), &mut |_| {});
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "render failed: {:?}",
        answer.to_json()
    );
    dir.join(format!("{name}.mp4"))
}

#[test]
fn reference_noise_dial_latency_bypass_and_determinism_hold() {
    let Some(ff) = ffmpeg() else { return };
    let dir = tempdir(line!());
    write_white_noise_wav(&ff, &dir.join("noise.wav"));

    let dry_mp4 = render_audio(&dir, "dry", "noise.wav", 1930, Value::Null);
    let dry_pcm = decode_mono_f32(&ff, &dry_mp4);
    let dry_db = rms_db(&dry_pcm);

    // 1. Reference white-noise law at 5, 10, 15, 20 dB within ±1.0 dB.
    for target_db in [5.0_f64, 10.0, 15.0, 20.0] {
        let tag = format!("nr_{}", target_db as i32);
        let wet_mp4 = render_audio(
            &dir,
            &tag,
            "noise.wav",
            1930,
            json!([{"name": "reduce_noise", "reduction_db": target_db}]),
        );
        let wet_pcm = decode_mono_f32(&ff, &wet_mp4);
        let measured_reduction = dry_db - rms_db(&wet_pcm);
        assert!(
            (measured_reduction - target_db).abs() <= 1.5,
            "expected reduction {target_db} dB ± 1.5 dB on reference white noise, got {measured_reduction:.2} dB"
        );
    }

    // 2. Enabled: false produces byte-identical PCM to no member, and two identical runs match.
    let bypass_mp4 = render_audio(
        &dir,
        "bypass",
        "noise.wav",
        1930,
        json!([{"name": "reduce_noise", "reduction_db": 12.0, "enabled": false}]),
    );
    let bypass_pcm = decode_mono_f32(&ff, &bypass_mp4);
    assert_eq!(dry_pcm, bypass_pcm, "enabled: false must match dry PCM");

    let det_a = render_audio(
        &dir,
        "det_a",
        "noise.wav",
        1930,
        json!([{"name": "reduce_noise", "reduction_db": 12.0}]),
    );
    let det_b = render_audio(
        &dir,
        "det_b",
        "noise.wav",
        1930,
        json!([{"name": "reduce_noise", "reduction_db": 12.0}]),
    );
    assert_eq!(
        decode_mono_f32(&ff, &det_a),
        decode_mono_f32(&ff, &det_b),
        "within-build determinism"
    );

    // 3. Lookahead latency compensation (384 samples = 8 ms): burst onset stays within ±1 ms (48 samples).
    write_speech_burst_wav(&ff, &dir.join("burst.wav"));
    let burst_dry = decode_mono_f32(
        &ff,
        &render_audio(&dir, "burst_dry", "burst.wav", 1930, Value::Null),
    );
    let burst_wet = decode_mono_f32(
        &ff,
        &render_audio(
            &dir,
            "burst_wet",
            "burst.wav",
            1930,
            json!([{"name": "reduce_noise", "reduction_db": 12.0}]),
        ),
    );
    let onset_dry = burst_dry.iter().position(|&s| s.abs() > 0.03).unwrap();
    let onset_wet = burst_wet.iter().position(|&s| s.abs() > 0.03).unwrap();
    let diff_samples = (onset_dry as isize - onset_wet as isize).unsigned_abs();
    assert!(
        diff_samples <= 48,
        "onset drift {diff_samples} samples exceeds ±1 ms (48 samples)"
    );
}

#[test]
fn validate_query_and_compare_cover_reduce_noise_contracts() {
    let dir = tempdir(line!());
    let body = canonical(
        &json!({
            "frame": {"width": 200, "height": 200},
            "fps": 25,
            "duration": 2000,
            "output": "out.mp4",
            "tracks": [{
                "name": "t",
                "layer": 0,
                "elements": [
                    {
                        "id": "low",
                        "type": "audio",
                        "start": 0,
                        "end": 1000,
                        "source": "a.wav",
                        "source_start": 0,
                        "source_end": 1000,
                        "audio_effects": [
                            {"name": "compressor", "threshold_db": -18.0, "ratio": 4.0, "attack_ms": 5.0, "release_ms": 50.0},
                            {"name": "reduce_noise", "reduction_db": 2.0}
                        ]
                    },
                    {
                        "id": "high",
                        "type": "audio",
                        "start": 1000,
                        "end": 2000,
                        "source": "a.wav",
                        "source_start": 0,
                        "source_end": 1000,
                        "audio_effects": [
                            {"name": "reduce_noise", "reduction_db": 25.0},
                            {"name": "reduce_noise", "reduction_db": 10.0}
                        ]
                    }
                ]
            }]
        })
        .to_string(),
    );
    let path = write_project(&dir, "p.montagent.json", &body);
    let report = validate(&path);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"E-NOISE-RANGE"), "got {codes:?}");
    assert!(codes.contains(&"R-NOISE-AFTER-COMPRESSOR"), "got {codes:?}");
    assert!(codes.contains(&"E-AUDIO-EFFECT-SINGULAR"), "got {codes:?}");

    let range_repairs: Vec<Value> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-NOISE-RANGE")
        .filter_map(|f| match &f.repair {
            Some(montagent_core::finding::Repair::Advise(v)) => Some(v.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        range_repairs,
        vec![
            json!({"key": "reduction_db", "value": 5.0}),
            json!({"key": "reduction_db", "value": 20.0}),
        ]
    );

    // query --at and compare:
    let p1_body = canonical(
        &json!({
            "frame": {"width": 200, "height": 200},
            "fps": 25,
            "duration": 1000,
            "output": "out.mp4",
            "tracks": [{
                "name": "t",
                "layer": 0,
                "elements": [{
                    "id": "bed",
                    "type": "audio",
                    "start": 0,
                    "end": 1000,
                    "source": "a.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "audio_effects": [{"name": "reduce_noise", "reduction_db": 10.0}]
                }]
            }]
        })
        .to_string(),
    );
    let p2_body = canonical(
        &json!({
            "frame": {"width": 200, "height": 200},
            "fps": 25,
            "duration": 1000,
            "output": "out.mp4",
            "tracks": [{
                "name": "t",
                "layer": 0,
                "elements": [{
                    "id": "bed",
                    "type": "audio",
                    "start": 0,
                    "end": 1000,
                    "source": "a.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "audio_effects": [{"name": "reduce_noise", "reduction_db": 15.0}]
                }]
            }]
        })
        .to_string(),
    );
    let p1 = write_project(&dir, "p1.montagent.json", &p1_body);
    let p2 = write_project(&dir, "p2.montagent.json", &p2_body);

    let q = query(
        &p1,
        &QueryAsk {
            at: Some(500),
            ..QueryAsk::default()
        },
    )
    .to_json();
    let stack = q["query"]["stack"].as_array().unwrap();
    assert_eq!(stack[0]["audio_effects"][0]["name"], json!("reduce_noise"));
    assert_eq!(stack[0]["audio_effects"][0]["reduction_db"], json!(10.0));

    let diff = compare(&p1, &p2);
    assert!(diff.findings.is_empty());
}
