//! End-to-end validation, `query --at`, and `render` (`mix_audio` PCM `f64le`) tests for `echo`
//! (`ADR-0185`, `#1155`).

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

fn mix_f64_stereo(
    dir: &Path,
    name: &str,
    duration_ms: i64,
    body: &str,
) -> (String, Vec<(f64, f64)>) {
    let project = common::write_project(dir, name, &common::canonical(body));
    let (inputs, graph) = mix_audio(&project, 0, duration_ms)
        .expect("mix_audio succeeds")
        .expect("audible mix");
    let mut command = Command::new(ffmpeg());
    command.args([
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "anullsrc=r=48000:cl=mono:d=0.1",
    ]);
    for input in &inputs {
        command.arg("-i").arg(input);
    }
    let out = command
        .arg("-filter_complex")
        .arg(&graph)
        .args([
            "-map", "[mix]", "-ar", "48000", "-ac", "2", "-f", "f64le", "-",
        ])
        .stderr(Stdio::piped())
        .output()
        .expect("ffmpeg runs");
    assert!(
        out.status.success(),
        "{graph}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let samples = out
        .stdout
        .chunks_exact(16)
        .map(|chunk| {
            let l = f64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let r = f64::from_le_bytes(chunk[8..16].try_into().unwrap());
            (l, r)
        })
        .collect();
    (graph, samples)
}

fn make_impulse_wav(dir: &Path, name: &str, duration_s: u32, impulse_sample: u64) -> PathBuf {
    let out = dir.join(name);
    let expr = format!(
        "aevalsrc='if(eq(n\\,{impulse_sample})\\,0.25\\,0)|if(eq(n\\,{impulse_sample})\\,0.25\\,0)':s=48000:d={duration_s}"
    );
    let status = Command::new(ffmpeg())
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &expr,
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f64le",
            &out.to_string_lossy(),
        ])
        .status()
        .expect("spawn ffmpeg for impulse wav");
    assert!(status.success(), "ffmpeg impulse wav failed");
    out
}

fn make_late_burst_wav(dir: &Path, name: &str) -> PathBuf {
    let out = dir.join(name);
    // 6.0 s source with a 440 Hz burst in [3.9 s, 4.0 s) (samples 187,200..192,000).
    let status = Command::new(ffmpeg())
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc='if(between(n\\,187200\\,191999)\\,0.4*sin(2*PI*440*t)\\,0)|if(between(n\\,187200\\,191999)\\,0.4*sin(2*PI*440*t)\\,0)':s=48000:d=6",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f64le",
            &out.to_string_lossy(),
        ])
        .status()
        .expect("spawn ffmpeg for late burst wav");
    assert!(status.success(), "ffmpeg late burst wav failed");
    out
}

#[test]
fn adr_0185_seven_tuples_tap_positions_amplitudes_onset_latency_duration_and_prefix_equality() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    // 4.0 s (192,000 samples) with unit-like impulse (0.25) at sample 24,000 (0.5 s).
    let imp_wav = make_impulse_wav(&dir, "impulse_4s.wav", 4, 24_000);
    let imp_src = imp_wav.file_name().unwrap().to_str().unwrap();

    let (bypass_graph, bypass) = mix_f64_stereo(
        &dir,
        "imp_bypass.montagent.json",
        4000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":4000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":4000,"source":"{imp_src}","source_start":0,"source_end":4000}}
                ]}}
            ]}}"#
        ),
    );
    assert_eq!(bypass.len(), 192_000);
    let dry_amp = bypass[24_000].0;
    assert!((dry_amp - 0.25).abs() < 1e-12);

    // Bypass identity: `enabled: false` and `mix: 0` are bit-for-bit identical to member-absent PCM.
    let (disabled_graph, disabled) = mix_f64_stereo(
        &dir,
        "imp_disabled.montagent.json",
        4000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":4000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":4000,"source":"{imp_src}","source_start":0,"source_end":4000,
                      "audio_effects":[{{"name":"echo","delay_ms":300,"feedback":0.5,"mix":0.5,"enabled":false}}]}}
                ]}}
            ]}}"#
        ),
    );
    let (mix_zero_graph, mix_zero) = mix_f64_stereo(
        &dir,
        "imp_mix0.montagent.json",
        4000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":4000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":4000,"source":"{imp_src}","source_start":0,"source_end":4000,
                      "audio_effects":[{{"name":"echo","delay_ms":300,"feedback":0.5,"mix":0}}]}}
                ]}}
            ]}}"#
        ),
    );
    assert_eq!(disabled_graph, bypass_graph);
    assert_eq!(mix_zero_graph, bypass_graph);
    assert_eq!(
        disabled, bypass,
        "enabled=false must be bit-identical to bypass"
    );
    assert_eq!(mix_zero, bypass, "mix=0 must be bit-identical to bypass");

    // ADR-0185 §7's 7 tuples:
    let tuples: [(f64, f64, f64); 7] = [
        (300.0, 0.5, 0.5),
        (300.0, 0.5, 1.0),
        (100.0, 0.9, 0.5),
        (1.0, 0.95, 1.0),
        (2000.0, 0.95, 1.0),
        (300.4, 0.5, 0.5),
        (250.0, 0.0, 1.0),
    ];

    for (idx, (delay_ms, feedback, mix)) in tuples.into_iter().enumerate() {
        let (graph, rendered) = mix_f64_stereo(
            &dir,
            &format!("echo_tuple_{idx}.montagent.json"),
            4000,
            &format!(
                r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":4000,"output":"o.mp4","tracks":[
                    {{"name":"a","layer":1,"elements":[
                        {{"id":"bed","type":"audio","start":0,"end":4000,"source":"{imp_src}","source_start":0,"source_end":4000,
                          "audio_effects":[{{"name":"echo","delay_ms":{delay_ms},"feedback":{feedback},"mix":{mix}}}]}}
                    ]}}
                ]}}"#
            ),
        );
        // Never more than 8 taps in the `aecho` delays list.
        if let Some(aecho_part) = graph.split("aecho=").nth(1) {
            let delays_part = aecho_part.split(':').nth(2).unwrap_or("");
            assert!(
                delays_part.split('|').count() <= 8,
                "aecho delays list must have at most 8 taps: {delays_part}"
            );
        }

        // 1. Exact placed duration: 192,000 samples (including at delay_ms = 2000).
        assert_eq!(
            rendered.len(),
            192_000,
            "tuple ({delay_ms}, {feedback}, {mix}): length must be 192,000 samples"
        );

        // 2. Zero onset latency: all samples before 24,000 are exact 0.0, and sample 24,000 is the dry impulse.
        assert!(
            rendered[..24_000]
                .iter()
                .all(|(l, r)| *l == 0.0 && *r == 0.0),
            "tuple ({delay_ms}, {feedback}, {mix}): samples before 24,000 must be 0.0"
        );
        assert!(
            (rendered[24_000].0 - dry_amp).abs() <= 1e-12
                && (rendered[24_000].1 - dry_amp).abs() <= 1e-12,
            "tuple ({delay_ms}, {feedback}, {mix}): dry impulse at sample 24,000 must equal {dry_amp}"
        );

        // 3. Expected taps k = 1..=8 with g_k = round(mix * feedback^(k-1), 6) >= 0.001:
        //    sample position = 24_000 + floor(k * delay_ms * 48) (within 192,000).
        let mut expected_positions = std::collections::BTreeMap::new();
        expected_positions.insert(24_000usize, dry_amp);
        let max_k = if feedback == 0.0 { 1 } else { 8 };
        for k in 1..=max_k {
            let g_raw = mix * feedback.powi(k - 1);
            if g_raw < 0.001 {
                break;
            }
            let g_round = (g_raw * 1_000_000.0).round() / 1_000_000.0;
            let offset_samples = ((k as f64) * delay_ms * 48.0).floor() as usize;
            let pos = 24_000 + offset_samples;
            if pos < 192_000 {
                *expected_positions.entry(pos).or_insert(0.0) += dry_amp * g_round;
            }
        }
        assert!(expected_positions.len() <= 9);

        for (sample_idx, (l, r)) in rendered.iter().enumerate() {
            if let Some(&exp_val) = expected_positions.get(&sample_idx) {
                let err_l = (l - exp_val).abs();
                let err_r = (r - exp_val).abs();
                assert!(
                    err_l <= 2e-6 && err_r <= 2e-6,
                    "tuple ({delay_ms}, {feedback}, {mix}) tap at sample {sample_idx}: got ({l}, {r}), expected {exp_val} (err {err_l:.2e})"
                );
            } else {
                assert!(
                    *l == 0.0 && *r == 0.0,
                    "tuple ({delay_ms}, {feedback}, {mix}) non-tap sample {sample_idx} must be 0.0, got ({l}, {r})"
                );
            }
        }
    }

    // Cut behavior & bit-for-bit prefix equality (ADR-0185 §3 & §7):
    // A 4,000 ms element (192,000 samples) with a burst in [3,900..4,000 ms) rendered with
    // `echo` (`delay_ms: 500, feedback: 0.6, mix: 0.5`) must be bit-for-bit identical to the
    // first 192,000 samples of a 5,500 ms render of the same source and effect.
    let burst_wav = make_late_burst_wav(&dir, "late_burst.wav");
    let burst_src = burst_wav.file_name().unwrap().to_str().unwrap();

    let (_, cut_4000) = mix_f64_stereo(
        &dir,
        "cut_4000.montagent.json",
        4000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":4000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":4000,"source":"{burst_src}","source_start":0,"source_end":4000,
                      "audio_effects":[{{"name":"echo","delay_ms":500,"feedback":0.6,"mix":0.5}}]}}
                ]}}
            ]}}"#
        ),
    );
    let (_, full_5500) = mix_f64_stereo(
        &dir,
        "full_5500.montagent.json",
        5500,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":5500,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":5500,"source":"{burst_src}","source_start":0,"source_end":5500,
                      "audio_effects":[{{"name":"echo","delay_ms":500,"feedback":0.6,"mix":0.5}}]}}
                ]}}
            ]}}"#
        ),
    );
    assert_eq!(cut_4000.len(), 192_000);
    assert_eq!(full_5500.len(), 264_000);
    assert_eq!(
        &cut_4000[..],
        &full_5500[..192_000],
        "4,000 ms cut render must be bit-for-bit equal to the first 192,000 samples of the 5,500 ms render"
    );
}

#[test]
fn echo_validation_and_query_checks() {
    let dir = common::tempdir(line!());
    let mut counter = 0usize;
    let mut check_doc = |audio_effects: Value| -> montagent_core::report::Report {
        counter += 1;
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
        let path = common::write_project(
            &dir,
            &format!("echo_check_{counter}.montagent.json"),
            &serde_json::to_string_pretty(&doc).unwrap(),
        );
        montagent_core::verbs::validate::validate(&path)
    };

    // 1. E-ECHO-RANGE on delay_ms (< 1, > 2000), feedback (< 0, > 0.95), mix (< 0, > 1).
    let bad_range = check_doc(json!([
        {"name": "echo", "delay_ms": 0.5, "feedback": 0.99, "mix": 1.5}
    ]));
    let range_findings: Vec<_> = bad_range
        .findings
        .iter()
        .filter(|f| f.code == "E-ECHO-RANGE")
        .collect();
    assert_eq!(range_findings.len(), 3, "{:?}", bad_range.findings);

    // 2. R-ECHO-HOT when combined worst-case peak gain > 3.0x; not when <= 3.0x.
    let hot = check_doc(json!([
        {"name": "echo", "delay_ms": 250, "feedback": 0.8, "mix": 0.8}
    ]));
    assert!(
        hot.findings.iter().any(|f| f.code == "R-ECHO-HOT"),
        "{:?}",
        hot.findings
    );
    let safe = check_doc(json!([
        {"name": "echo", "delay_ms": 250, "feedback": 0.5, "mix": 0.5}
    ]));
    assert!(
        !safe.findings.iter().any(|f| f.code == "R-ECHO-HOT"),
        "{:?}",
        safe.findings
    );

    // 3. N-ECHO-NO-OP when mix < 0.001.
    let noop = check_doc(json!([
        {"name": "echo", "delay_ms": 300, "feedback": 0.5, "mix": 0.0005}
    ]));
    assert!(
        noop.findings.iter().any(|f| f.code == "N-ECHO-NO-OP"),
        "{:?}",
        noop.findings
    );

    // 4. N-ECHO-TAPS-CAPPED when mix * feedback^7 > 0.01.
    let capped = check_doc(json!([
        {"name": "echo", "delay_ms": 100, "feedback": 0.9, "mix": 0.5}
    ]));
    assert!(
        capped
            .findings
            .iter()
            .any(|f| f.code == "N-ECHO-TAPS-CAPPED"),
        "{:?}",
        capped.findings
    );

    // 5. Out-of-order echo keys trigger L-LAYOUT and canonicalise to name, delay_ms, feedback, mix.
    let canonical_echo = common::canonical(
        r#"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":0,"elements":[{"id":"bed","type":"audio","start":0,"end":1000,"source":"a.wav","source_start":0,"source_end":1000,"audio_effects":[{"name":"echo","delay_ms":300,"feedback":0.5,"mix":0.5}]}]}]}"#,
    );
    let reordered_echo = canonical_echo.replace(
        r#""delay_ms":300,"feedback":0.5"#,
        r#""feedback":0.5,"delay_ms":300"#,
    );
    assert_ne!(reordered_echo, canonical_echo);
    let bad_order_path =
        common::write_project(&dir, "echo_bad_order.montagent.json", &reordered_echo);
    let bad_order = montagent_core::verbs::validate::validate(&bad_order_path);
    assert!(
        bad_order.findings.iter().any(|f| f.code == "L-LAYOUT"),
        "{:?}",
        bad_order.findings
    );
}
