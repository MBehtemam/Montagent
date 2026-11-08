//! End-to-end validation, `query --at`, and `render` (`mix_audio` PCM `f64le`) tests for
//! `reduce_noise` (`ADR-0181`, `#1153`).

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use montagent_core::verbs::render::mix_audio;
use serde_json::{Value, json};

fn ffmpeg() -> PathBuf {
    montagent_core::media::tools::resolve()
        .expect("an ffmpeg")
        .ffmpeg
}

fn mix_f64_stereo(dir: &Path, name: &str, duration_ms: i64, body: &str) -> Vec<(f64, f64)> {
    let project = common::write_project(dir, name, &common::canonical(body));
    let (inputs, graph) = mix_audio(&project, 0, duration_ms)
        .expect("mix_audio succeeds")
        .expect("audible mix");
    let mut out = None;
    for _ in 0..4 {
        let mut command = Command::new(ffmpeg());
        command.args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-filter_threads",
            "1",
            "-filter_complex_threads",
            "1",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=48000:cl=mono:d=0.1",
        ]);
        for input in &inputs {
            command.arg("-i").arg(input);
        }
        let candidate = command
            .arg("-filter_complex")
            .arg(&graph)
            .args([
                "-map", "[mix]", "-ar", "48000", "-ac", "2", "-f", "f32le", "-",
            ])
            .stderr(Stdio::piped())
            .output()
            .expect("ffmpeg runs");
        if candidate.status.success() {
            out = Some(candidate);
            break;
        }
    }
    let out = out.unwrap_or_else(|| panic!("ffmpeg failed on graph:\n{graph}"));
    out.stdout
        .chunks_exact(8)
        .map(|chunk| {
            let l = f32::from_le_bytes(chunk[0..4].try_into().unwrap()) as f64;
            let r = f32::from_le_bytes(chunk[4..8].try_into().unwrap()) as f64;
            (l, r)
        })
        .collect()
}

fn rms_dbfs_window(samples: &[(f64, f64)], start_frame: usize, end_frame: usize) -> f64 {
    let slice = &samples[start_frame..end_frame.min(samples.len())];
    assert!(!slice.is_empty());
    let sum_sq: f64 = slice.iter().map(|(l, r)| l * l + r * r).sum();
    let mean_sq = sum_sq / ((slice.len() * 2) as f64);
    10.0 * mean_sq.max(1e-30).log10()
}

fn make_stationary_noise_wav(dir: &Path, name: &str) -> PathBuf {
    let out = dir.join(name);
    // Seeded white noise at a = 0.005477225575 -> RMS = -50.0 dBFS on each stereo channel (ADR-0181 §3).
    // Equal channels via `pan=stereo|c0=c0|c1=c0` avoid ffmpeg's default -3 dB mono->stereo upmix.
    // Duration 2.895 s = 15 * 193 ms aligns to anlmdn's 193 ms block size (avoiding FFmpeg 9.0.2
    // partial-block EOF crash/hang noted in ADR-0181 §7) while covering the 1.0..2.5 s window.
    let status = Command::new(ffmpeg())
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=d=2.895:c=white:r=48000:a=0.005477225575:seed=7",
            "-af",
            "pan=stereo|c0=c0|c1=c0",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f32le",
            &out.to_string_lossy(),
        ])
        .status()
        .expect("spawn ffmpeg for stationary noise wav");
    assert!(status.success(), "ffmpeg stationary noise wav failed");
    out
}

fn make_tone_over_noise_wav(dir: &Path, name: &str) -> PathBuf {
    let out = dir.join(name);
    // 1 kHz sine at -23 dBFS RMS (amplitude 0.1) over the same -50 dBFS white noise (ADR-0181 §7).
    let status = Command::new(ffmpeg())
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=d=2.895:c=white:r=48000:a=0.005477225575:seed=7",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=0.1*sin(2*PI*997*t):s=48000:d=2.895",
            "-filter_complex",
            "[0:a][1:a]amix=inputs=2:duration=first:normalize=0,pan=stereo|c0=c0|c1=c0",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f32le",
            &out.to_string_lossy(),
        ])
        .status()
        .expect("spawn ffmpeg for tone over noise wav");
    assert!(status.success(), "ffmpeg tone over noise wav failed");
    out
}

fn make_onset_burst_wav(dir: &Path, name: &str) -> PathBuf {
    let out = dir.join(name);
    // 1.93 s (10 * 193 ms = 92,640 samples) at 48 kHz: silence in [0, 0.5 s), then 997 Hz sine burst starting at sample 24,000.
    let status = Command::new(ffmpeg())
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc='if(gte(n\\,24000)\\,0.5*sin(2*PI*997*t)\\,0)':s=48000:d=1.93",
            "-af",
            "pan=stereo|c0=c0|c1=c0",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f32le",
            &out.to_string_lossy(),
        ])
        .status()
        .expect("spawn ffmpeg for onset burst wav");
    assert!(status.success(), "ffmpeg onset burst wav failed");
    out
}

#[test]
fn all_seven_reduction_db_values_attenuate_noise_within_1db_and_preserve_1khz_tone_within_half_db()
{
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let noise_wav = make_stationary_noise_wav(&dir, "noise.wav");
    let tone_wav = make_tone_over_noise_wav(&dir, "tone_plus_noise.wav");
    let noise_src = noise_wav.file_name().unwrap().to_str().unwrap();
    let tone_src = tone_wav.file_name().unwrap().to_str().unwrap();

    // Window 1.0..2.5 s (samples 48,000..120,000) per ADR-0181 §3 & §7.
    let w0 = 48_000;
    let w1 = 120_000;

    let base_noise = mix_f64_stereo(
        &dir,
        "noise_base.montagent.json",
        2895,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":2895,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":2895,"source":"{noise_src}","source_start":0,"source_end":2895}}
                ]}}
            ]}}"#
        ),
    );
    let base_noise_db = rms_dbfs_window(&base_noise, w0, w1);

    let base_tone = mix_f64_stereo(
        &dir,
        "tone_base.montagent.json",
        2895,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":2895,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":2895,"source":"{tone_src}","source_start":0,"source_end":2895}}
                ]}}
            ]}}"#
        ),
    );
    let base_tone_db = rms_dbfs_window(&base_tone, w0, w1);

    for reduction_db in [5.0_f64, 8.0, 10.0, 12.0, 15.0, 18.0, 20.0] {
        let tag = reduction_db as u32;
        let denoised = mix_f64_stereo(
            &dir,
            &format!("noise_{tag}.montagent.json"),
            2895,
            &format!(
                r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":2895,"output":"o.mp4","tracks":[
                    {{"name":"a","layer":1,"elements":[
                        {{"id":"bed","type":"audio","start":0,"end":2895,"source":"{noise_src}","source_start":0,"source_end":2895,
                          "audio_effects":[{{"name":"reduce_noise","reduction_db":{reduction_db}}}]}}
                    ]}}
                ]}}"#
            ),
        );
        let denoised_db = rms_dbfs_window(&denoised, w0, w1);
        let measured_atten = base_noise_db - denoised_db;
        let err = (measured_atten - reduction_db).abs();
        assert!(
            err <= 1.0,
            "reduction_db={reduction_db}: measured attenuation {measured_atten:.2} dB (error {err:.2} dB > 1.0 dB)"
        );

        let tone_denoised = mix_f64_stereo(
            &dir,
            &format!("tone_{tag}.montagent.json"),
            2895,
            &format!(
                r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":2895,"output":"o.mp4","tracks":[
                    {{"name":"a","layer":1,"elements":[
                        {{"id":"bed","type":"audio","start":0,"end":2895,"source":"{tone_src}","source_start":0,"source_end":2895,
                          "audio_effects":[{{"name":"reduce_noise","reduction_db":{reduction_db}}}]}}
                    ]}}
                ]}}"#
            ),
        );
        let tone_db = rms_dbfs_window(&tone_denoised, w0, w1);
        let tone_delta = (tone_db - base_tone_db).abs();
        assert!(
            tone_delta <= 0.5,
            "reduction_db={reduction_db}: 1 kHz tone at -23 dBFS changed by {tone_delta:.3} dB (> 0.5 dB)"
        );
    }
}

#[test]
fn compensated_onset_latency_is_zero_samples_on_pcm_and_disabled_is_bit_identical_to_bypass() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let burst_wav = make_onset_burst_wav(&dir, "burst.wav");
    let burst_src = burst_wav.file_name().unwrap().to_str().unwrap();

    let bypass = mix_f64_stereo(
        &dir,
        "burst_bypass.montagent.json",
        1930,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1930,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1930,"source":"{burst_src}","source_start":0,"source_end":1930}}
                ]}}
            ]}}"#
        ),
    );
    let disabled = mix_f64_stereo(
        &dir,
        "burst_disabled.montagent.json",
        1930,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1930,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1930,"source":"{burst_src}","source_start":0,"source_end":1930,
                      "audio_effects":[{{"name":"reduce_noise","reduction_db":12,"enabled":false}}]}}
                ]}}
            ]}}"#
        ),
    );
    let active = mix_f64_stereo(
        &dir,
        "burst_active.montagent.json",
        1930,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1930,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1930,"source":"{burst_src}","source_start":0,"source_end":1930,
                      "audio_effects":[{{"name":"reduce_noise","reduction_db":12}}]}}
                ]}}
            ]}}"#
        ),
    );

    assert_eq!(bypass.len(), 92_640, "exact 1.93 s length at 48 kHz");
    assert_eq!(active.len(), 92_640, "exact 1.93 s length at 48 kHz");
    assert_eq!(
        bypass, disabled,
        "disabled reduce_noise must be bit-identical in f64le to bypass"
    );

    let bypass_onset = bypass
        .iter()
        .position(|(l, _)| l.abs() > 0.01)
        .expect("bypass onset");
    let active_onset = active
        .iter()
        .position(|(l, _)| l.abs() > 0.01)
        .expect("active onset");
    assert_eq!(
        active_onset, bypass_onset,
        "compensated onset latency must be 0 samples (active {active_onset} vs bypass {bypass_onset})"
    );
}

#[test]
fn noise_reduction_validation_and_query_checks() {
    let dir = common::tempdir(line!());
    let mut seq = 0usize;
    let mut check_doc = |audio_effects: Value| -> montagent_core::report::Report {
        seq += 1;
        let doc = json!({
            "frame": {"width": 1080, "height": 1920},
            "fps": 25,
            "tracks": [{
                "name": "a",
                "layer": 0,
                "elements": [{
                    "id": "bed",
                    "type": "audio",
                    "start": 0,
                    "end": 1000,
                    "source": "a.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "audio_effects": audio_effects,
                }]
            }]
        });
        let path = common::write_project(&dir, &format!("p{seq}.montagent.json"), &doc.to_string());
        montagent_core::verbs::validate::validate(&path)
    };

    // 1. E-NOISE-RANGE below 5 and above 20, with clamped nearest repair.
    let low = check_doc(json!([{"name": "reduce_noise", "reduction_db": 3}]));
    let low_f = low
        .findings
        .iter()
        .find(|f| f.code == "E-NOISE-RANGE")
        .expect("E-NOISE-RANGE on 3");
    assert_eq!(
        serde_json::to_value(&low_f.repair).unwrap(),
        json!({"key": "reduction_db", "value": 5.0})
    );

    let high = check_doc(json!([{"name": "reduce_noise", "reduction_db": 25}]));
    let high_f = high
        .findings
        .iter()
        .find(|f| f.code == "E-NOISE-RANGE")
        .expect("E-NOISE-RANGE on 25");
    assert_eq!(
        serde_json::to_value(&high_f.repair).unwrap(),
        json!({"key": "reduction_db", "value": 20.0})
    );

    // 2. R-NOISE-AFTER-COMPRESSOR fires when enabled reduce_noise follows enabled compressor.
    let after_comp = check_doc(json!([
        {"name": "compressor", "threshold_db": -18, "ratio": 3, "attack_ms": 10, "release_ms": 100, "makeup_db": 0},
        {"name": "reduce_noise", "reduction_db": 12}
    ]));
    assert!(
        after_comp
            .findings
            .iter()
            .any(|f| f.code == "R-NOISE-AFTER-COMPRESSOR"),
        "{:?}",
        after_comp.findings
    );

    // Before compressor -> no R-NOISE-AFTER-COMPRESSOR.
    let before_comp = check_doc(json!([
        {"name": "reduce_noise", "reduction_db": 12},
        {"name": "compressor", "threshold_db": -18, "ratio": 3, "attack_ms": 10, "release_ms": 100, "makeup_db": 0}
    ]));
    assert!(
        !before_comp
            .findings
            .iter()
            .any(|f| f.code == "R-NOISE-AFTER-COMPRESSOR"),
        "{:?}",
        before_comp.findings
    );

    // 3. E-AUDIO-EFFECT-SINGULAR on two enabled reduce_noise members.
    let singular = check_doc(json!([
        {"name": "reduce_noise", "reduction_db": 10},
        {"name": "reduce_noise", "reduction_db": 15}
    ]));
    assert!(
        singular
            .findings
            .iter()
            .any(|f| f.code == "E-AUDIO-EFFECT-SINGULAR"),
        "{:?}",
        singular.findings
    );

    // 4. R-AUDIO-EFFECT-DISABLED on disabled reduce_noise.
    let disabled = check_doc(json!([
        {"name": "reduce_noise", "reduction_db": 12, "enabled": false}
    ]));
    assert!(
        disabled
            .findings
            .iter()
            .any(|f| f.code == "R-AUDIO-EFFECT-DISABLED"),
        "{:?}",
        disabled.findings
    );

    // 5. Out-of-order reduce_noise keys trigger L-LAYOUT and canonicalise to name, reduction_db.
    let canonical_noise = common::canonical(
        r#"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":0,"elements":[{"id":"bed","type":"audio","start":0,"end":1000,"source":"a.wav","source_start":0,"source_end":1000,"audio_effects":[{"name":"reduce_noise","reduction_db":12}]}]}]}"#,
    );
    let reordered_noise = canonical_noise.replace(
        r#""name":"reduce_noise","reduction_db":12"#,
        r#""reduction_db":12,"name":"reduce_noise""#,
    );
    assert_ne!(reordered_noise, canonical_noise);
    let bad_order_path =
        common::write_project(&dir, "noise_bad_order.montagent.json", &reordered_noise);
    let bad_order = montagent_core::verbs::validate::validate(&bad_order_path);
    assert!(
        bad_order.findings.iter().any(|f| f.code == "L-LAYOUT"),
        "{:?}",
        bad_order.findings
    );
}
