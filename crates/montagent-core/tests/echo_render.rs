//! Measured integration test for `echo` (ADR-0185, #1155).
//!
//! Asserts discrete tap timing (`k * delay_ms`), geometric tap decay (`mix * feedback^(k-1)`),
//! the 8-tap / −60 dB cutoff, tail truncation at `end`, `enabled: false` and `mix: 0` bypass
//! identity, within-build determinism, and `validate`/`query`/`compare` contracts.

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

const RATE: usize = 48_000;

fn ffmpeg() -> Option<PathBuf> {
    resolve().ok().map(|t| t.ffmpeg)
}

fn write_burst_wav(ff: &Path, path: &Path) {
    // 50 ms 1 kHz burst at -6.02 dBFS peak (0.5) inside a 3.0 s file.
    let ok = Command::new(ff)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=0.5*sin(2*PI*1000*t)*lt(t\\,0.05):s=48000:d=3",
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
    assert!(ok, "burst wav must write");
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

fn window_rms(samples: &[f32], start_s: f64, end_s: f64) -> f64 {
    let i0 = ((start_s * RATE as f64).round() as usize).min(samples.len());
    let i1 = ((end_s * RATE as f64).round() as usize).min(samples.len());
    let slice = &samples[i0..i1];
    let sum_sq: f64 = slice.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / slice.len() as f64).sqrt()
}

fn render_clip(dir: &Path, name: &str, clip_end_ms: i64, total_ms: i64, effects: Value) -> PathBuf {
    let mut elem = json!({
        "id": "a1",
        "type": "audio",
        "start": 0,
        "end": clip_end_ms,
        "source": "burst.wav",
        "source_start": 0,
        "source_end": clip_end_ms,
    });
    if !effects.is_null() {
        elem["audio_effects"] = effects;
    }
    let mut elements = vec![elem];
    if total_ms > clip_end_ms {
        elements.push(json!({
            "id": "pad_rect",
            "type": "rect",
            "start": clip_end_ms,
            "end": total_ms,
            "x": 0,
            "y": 0,
            "width": 10,
            "height": 10,
            "fill": "#000000"
        }));
    }
    let out_rel = format!("{name}.mp4");
    let body = canonical(
        &json!({
            "frame": {"width": 200, "height": 200},
            "fps": 25,
            "duration": total_ms,
            "output": out_rel,
            "tracks": [{"name": "t", "layer": 0, "elements": elements}],
        })
        .to_string(),
    );
    let file = write_project(dir, &format!("{name}.montagent.json"), &body);
    let answer = render(&file, &RenderAsk::default(), &mut |_| {});
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "render failed: {:?}",
        answer.to_json()
    );
    dir.join(format!("{name}.mp4"))
}

#[test]
fn echo_taps_decay_tail_cut_and_bypass_hold_in_pcm() {
    let Some(ff) = ffmpeg() else { return };
    let dir = tempdir(line!());
    write_burst_wav(&ff, &dir.join("burst.wav"));

    // 1. Single tap (feedback: 0, delay_ms: 250, mix: 0.5):
    let single_mp4 = render_clip(
        &dir,
        "single",
        2000,
        2000,
        json!([{"name": "echo", "delay_ms": 250.0, "feedback": 0.0, "mix": 0.5}]),
    );
    let single_pcm = decode_mono_f32(&ff, &single_mp4);
    let dry_rms = window_rms(&single_pcm, 0.005, 0.045);
    let tap1_rms = window_rms(&single_pcm, 0.255, 0.295);
    let tap2_rms = window_rms(&single_pcm, 0.505, 0.545);
    assert!(dry_rms > 0.2, "dry rms {dry_rms}");
    assert!(
        (tap1_rms / dry_rms - 0.5).abs() < 0.05,
        "tap 1 ratio {}",
        tap1_rms / dry_rms
    );
    assert!(
        tap2_rms < 1e-3,
        "feedback 0 must produce no 2nd tap, got {tap2_rms}"
    );

    // 2. Multi-tap geometric decay (delay_ms: 200, feedback: 0.5, mix: 0.5) & 8-tap cap:
    let multi_mp4 = render_clip(
        &dir,
        "multi",
        2500,
        2500,
        json!([{"name": "echo", "delay_ms": 200.0, "feedback": 0.5, "mix": 0.5}]),
    );
    let multi_pcm = decode_mono_f32(&ff, &multi_mp4);
    let d0 = window_rms(&multi_pcm, 0.005, 0.045);
    for k in 1..=5_usize {
        let t_start = (k as f64) * 0.2 + 0.005;
        let t_end = (k as f64) * 0.2 + 0.045;
        let rk = window_rms(&multi_pcm, t_start, t_end);
        let expected_ratio = 0.5 * 0.5_f64.powi((k - 1) as i32);
        let ratio_db_err = (20.0 * (rk / d0 / expected_ratio).log10()).abs();
        assert!(
            ratio_db_err <= 0.8,
            "tap {k} ratio error {ratio_db_err:.2} dB (got {}, expected {})",
            rk / d0,
            expected_ratio
        );
    }
    // 9th tap window at 1.8 s must be silent because taps are capped at 8:
    let tap9_rms = window_rms(&multi_pcm, 1.805, 1.845);
    assert!(
        tap9_rms < 1e-3,
        "9th tap window must be silent, got {tap9_rms}"
    );

    // 3. Tail cut at element end: clip ends at 0.30 s inside a 1.0 s timeline with delay_ms: 500.
    let tail_cut_mp4 = render_clip(
        &dir,
        "tail_cut",
        300,
        1000,
        json!([{"name": "echo", "delay_ms": 500.0, "feedback": 0.5, "mix": 0.8}]),
    );
    let tail_cut_pcm = decode_mono_f32(&ff, &tail_cut_mp4);
    let after_end_rms = window_rms(&tail_cut_pcm, 0.35, 0.95);
    assert!(
        after_end_rms < 1e-4,
        "echo tail after element end (300 ms) must be cut, got {after_end_rms}"
    );

    // 4. Bypass identity (enabled: false and mix: 0) & determinism:
    let dry_mp4 = render_clip(&dir, "dry", 1000, 1000, Value::Null);
    let disabled_mp4 = render_clip(
        &dir,
        "disabled",
        1000,
        1000,
        json!([{"name": "echo", "delay_ms": 250.0, "feedback": 0.5, "mix": 0.5, "enabled": false}]),
    );
    let mix_zero_mp4 = render_clip(
        &dir,
        "mix_zero",
        1000,
        1000,
        json!([{"name": "echo", "delay_ms": 250.0, "feedback": 0.5, "mix": 0.0}]),
    );
    let dry_samples = decode_mono_f32(&ff, &dry_mp4);
    assert_eq!(dry_samples, decode_mono_f32(&ff, &disabled_mp4));
    assert_eq!(dry_samples, decode_mono_f32(&ff, &mix_zero_mp4));
}

#[test]
fn validate_query_and_compare_cover_echo_contracts() {
    let dir = tempdir(line!());
    let doc = json!({
        "frame": {"width": 200, "height": 200},
        "fps": 25,
        "duration": 3000,
        "output": "out.mp4",
        "tracks": [{
            "name": "t",
            "layer": 0,
            "elements": [
                {
                    "id": "ranges",
                    "type": "audio",
                    "start": 0,
                    "end": 1000,
                    "source": "a.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "audio_effects": [
                        {"name": "echo", "delay_ms": 0.5, "feedback": 0.99, "mix": 1.5}
                    ]
                },
                {
                    "id": "hot_and_capped",
                    "type": "audio",
                    "start": 1000,
                    "end": 2000,
                    "source": "a.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "audio_effects": [
                        {"name": "echo", "delay_ms": 250.0, "feedback": 0.85, "mix": 0.6}
                    ]
                },
                {
                    "id": "noop",
                    "type": "audio",
                    "start": 2000,
                    "end": 3000,
                    "source": "a.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "audio_effects": [
                        {"name": "echo", "delay_ms": 250.0, "feedback": 0.3, "mix": 0.0}
                    ]
                }
            ]
        }]
    });
    let file = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    let report = validate(&file);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"E-ECHO-RANGE"), "got {codes:?}");
    assert!(codes.contains(&"R-ECHO-HOT"), "got {codes:?}");
    assert!(codes.contains(&"N-ECHO-TAPS-CAPPED"), "got {codes:?}");
    assert!(codes.contains(&"N-ECHO-NO-OP"), "got {codes:?}");

    let range_repairs: Vec<Value> = report
        .findings
        .iter()
        .filter(|f| f.code == "E-ECHO-RANGE")
        .filter_map(|f| match &f.repair {
            Some(montagent_core::finding::Repair::Advise(v)) => Some(v.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        range_repairs,
        vec![
            json!({"key": "delay_ms", "value": 1.0}),
            json!({"key": "feedback", "value": 0.95}),
            json!({"key": "mix", "value": 1.0}),
        ]
    );

    // query --at and compare:
    let p1_doc = json!({
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
                "audio_effects": [{"name": "echo", "delay_ms": 250.0, "feedback": 0.35, "mix": 0.25}]
            }]
        }]
    });
    let p2_doc = json!({
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
                "audio_effects": [{"name": "echo", "delay_ms": 300.0, "feedback": 0.35, "mix": 0.25}]
            }]
        }]
    });
    let p1 = write_project(&dir, "p1.montagent.json", &canonical(&p1_doc.to_string()));
    let p2 = write_project(&dir, "p2.montagent.json", &canonical(&p2_doc.to_string()));

    let q = query(
        &p1,
        &QueryAsk {
            at: Some(500),
            ..QueryAsk::default()
        },
    )
    .to_json();
    let eff = &q["query"]["stack"][0]["audio_effects"][0];
    assert_eq!(eff["name"], json!("echo"));
    assert_eq!(eff["delay_ms"], json!(250.0));
    assert_eq!(eff["feedback"], json!(0.35));
    assert_eq!(eff["mix"], json!(0.25));

    let diff = compare(&p1, &p2);
    assert!(diff.findings.is_empty());
}
