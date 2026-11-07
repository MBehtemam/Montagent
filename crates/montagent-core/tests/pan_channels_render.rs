//! Measured integration test for `pan` and `channels` (ADR-0184, #1154).
//!
//! Asserts linear balance law (`pan = -1.0, -0.5, 0.0, 0.5, 1.0`), mono centering via
//! `aformat=channel_layouts=stereo`, the four active `channels` routing words (`swap`,
//! `mono`, `left`, `right`) on asymmetric stereo, `[channels] -> [audio_effects] -> [volume] -> [pan]`
//! stage order with `compressor`, and `validate`/`query`/`compare`/`fmt` contracts.

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

fn write_asymmetric_stereo_wav(ff: &Path, path: &Path) {
    // L = 440 Hz at 0.5 (-6.02 dBFS), R = 1000 Hz at 0.25 (-12.04 dBFS), 1.0 s.
    let ok = Command::new(ff)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=0.5*sin(2*PI*440*t)|0.25*sin(2*PI*1000*t):s=48000:d=1",
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
    assert!(ok, "asymmetric stereo wav must write");
}

fn write_symmetric_stereo_wav(ff: &Path, path: &Path) {
    // L = R = 1000 Hz at 0.5 (-6.02 dBFS), 1.0 s.
    let ok = Command::new(ff)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=0.5*sin(2*PI*1000*t)|0.5*sin(2*PI*1000*t):s=48000:d=1",
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
    assert!(ok, "symmetric stereo wav must write");
}

fn write_mono_wav(ff: &Path, path: &Path) {
    let ok = Command::new(ff)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=1000:duration=1:sample_rate=48000",
            "-af",
            "volume=0.5",
            "-ar",
            "48000",
            "-ac",
            "1",
            "-c:a",
            "pcm_f32le",
        ])
        .arg(path)
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "mono wav must write");
}

fn decode_stereo_f32(ff: &Path, path: &Path) -> (Vec<f32>, Vec<f32>) {
    let out = Command::new(ff)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args(["-ac", "2", "-ar", "48000", "-f", "f32le", "-"])
        .output()
        .expect("ffmpeg decode stereo");
    assert!(out.status.success(), "decode failed");
    let samples: Vec<f32> = out
        .stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let mut left = Vec::with_capacity(samples.len() / 2);
    let mut right = Vec::with_capacity(samples.len() / 2);
    for pair in samples.chunks_exact(2) {
        left.push(pair[0]);
        right.push(pair[1]);
    }
    (left, right)
}

fn rms(samples: &[f32]) -> f64 {
    let n = samples.len();
    let slice = &samples[n / 10..(n * 9) / 10];
    let sum_sq: f64 = slice.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / slice.len() as f64).sqrt()
}

fn render_clip(dir: &Path, name: &str, element_extra: Value) -> PathBuf {
    let mut elem = json!({
        "id": "a1",
        "type": "audio",
        "start": 0,
        "end": 1000,
        "source": "in.wav",
        "source_start": 0,
        "source_end": 1000,
    });
    if let (Some(dst), Some(src)) = (elem.as_object_mut(), element_extra.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    let out_rel = format!("{name}.mp4");
    let body = canonical(
        &json!({
            "frame": {"width": 200, "height": 200},
            "fps": 25,
            "duration": 1000,
            "output": out_rel,
            "tracks": [{"name": "t", "layer": 0, "elements": [elem]}],
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
fn pan_linear_balance_and_channels_routing_hold_in_pcm() {
    let Some(ff) = ffmpeg() else { return };
    let dir = tempdir(line!());
    write_symmetric_stereo_wav(&ff, &dir.join("in.wav"));

    // 1. Pan law on symmetric stereo input (L_in == R_in):
    let out_absent = render_clip(&dir, "absent", json!({}));
    let out_zero = render_clip(&dir, "zero", json!({ "pan": 0.0 }));
    let (l_abs, r_abs) = decode_stereo_f32(&ff, &out_absent);
    let (l_zero, r_zero) = decode_stereo_f32(&ff, &out_zero);
    assert_eq!(l_abs, l_zero, "pan: 0 must be byte-identical to pan absent");
    assert_eq!(r_abs, r_zero, "pan: 0 must be byte-identical to pan absent");
    let base_rms = rms(&l_abs);

    // pan = -1.0 -> L = 1.0, R = 0.0
    let (l_m1, r_m1) = decode_stereo_f32(&ff, &render_clip(&dir, "m1", json!({ "pan": -1.0 })));
    assert!((rms(&l_m1) - base_rms).abs() / base_rms < 0.02);
    assert!(rms(&r_m1) < 1e-4, "R channel must be silent at pan = -1.0");

    // pan = -0.5 -> L = 1.0, R = 0.5
    let (l_mh, r_mh) = decode_stereo_f32(&ff, &render_clip(&dir, "mh", json!({ "pan": -0.5 })));
    assert!((rms(&l_mh) - base_rms).abs() / base_rms < 0.02);
    assert!((rms(&r_mh) / rms(&l_mh) - 0.5).abs() < 0.02);

    // pan = 0.5 -> L = 0.5, R = 1.0
    let (l_ph, r_ph) = decode_stereo_f32(&ff, &render_clip(&dir, "ph", json!({ "pan": 0.5 })));
    assert!((rms(&r_ph) - base_rms).abs() / base_rms < 0.02);
    assert!((rms(&l_ph) / rms(&r_ph) - 0.5).abs() < 0.02);

    // pan = 1.0 -> L = 0.0, R = 1.0
    let (l_p1, r_p1) = decode_stereo_f32(&ff, &render_clip(&dir, "p1", json!({ "pan": 1.0 })));
    assert!((rms(&r_p1) - base_rms).abs() / base_rms < 0.02);
    assert!(rms(&l_p1) < 1e-4, "L channel must be silent at pan = 1.0");

    // 2. Channels routing on asymmetric stereo input (L = 0.5 @ 440Hz, R = 0.25 @ 1000Hz):
    write_asymmetric_stereo_wav(&ff, &dir.join("in.wav"));
    let (l_st, r_st) = decode_stereo_f32(&ff, &render_clip(&dir, "st", json!({})));
    let (l_st_explicit, r_st_explicit) = decode_stereo_f32(
        &ff,
        &render_clip(&dir, "st_exp", json!({ "channels": "stereo" })),
    );
    assert_eq!(l_st, l_st_explicit, "channels: stereo must equal omitted");
    assert_eq!(r_st, r_st_explicit);

    let rms_l = rms(&l_st);
    let rms_r = rms(&r_st);
    assert!(
        (rms_l / rms_r - 2.0).abs() < 0.05,
        "L is 2x R in asymmetric fixture"
    );

    // swap: L_out = R_in, R_out = L_in
    let (l_sw, r_sw) = decode_stereo_f32(
        &ff,
        &render_clip(&dir, "swap", json!({ "channels": "swap" })),
    );
    assert!((rms(&l_sw) - rms_r).abs() / rms_r < 0.02);
    assert!((rms(&r_sw) - rms_l).abs() / rms_l < 0.02);

    // left: L_out = L_in, R_out = L_in
    let (l_le, r_le) = decode_stereo_f32(
        &ff,
        &render_clip(&dir, "left", json!({ "channels": "left" })),
    );
    assert!((rms(&l_le) - rms_l).abs() / rms_l < 0.02);
    assert!((rms(&r_le) - rms_l).abs() / rms_l < 0.02);

    // right: L_out = R_in, R_out = R_in
    let (l_ri, r_ri) = decode_stereo_f32(
        &ff,
        &render_clip(&dir, "right", json!({ "channels": "right" })),
    );
    assert!((rms(&l_ri) - rms_r).abs() / rms_r < 0.02);
    assert!((rms(&r_ri) - rms_r).abs() / rms_r < 0.02);

    // mono: L_out = R_out = 0.5*L_in + 0.5*R_in
    let (l_mo, r_mo) = decode_stereo_f32(
        &ff,
        &render_clip(&dir, "mono", json!({ "channels": "mono" })),
    );
    let expected_mono = 0.5 * (rms_l * rms_l + rms_r * rms_r).sqrt();
    assert!((rms(&l_mo) - expected_mono).abs() / expected_mono < 0.03);
    assert!((rms(&r_mo) - expected_mono).abs() / expected_mono < 0.03);

    // 3. Stage order: channels: "left" feeds compressor, pan: 0.5 attenuates L after compressor.
    let (l_ord, r_ord) = decode_stereo_f32(
        &ff,
        &render_clip(
            &dir,
            "order",
            json!({
                "channels": "left",
                "pan": 0.5,
                "audio_effects": [{
                    "name": "compressor",
                    "threshold_db": -18.0,
                    "ratio": 4.0,
                    "attack_ms": 5.0,
                    "release_ms": 50.0,
                    "makeup_db": 0.0
                }]
            }),
        ),
    );
    assert!((rms(&l_ord) / rms(&r_ord) - 0.5).abs() < 0.02);

    // 4. Mono source with pan: 1.0 routes to R because aformat=channel_layouts=stereo runs before pan.
    write_mono_wav(&ff, &dir.join("in.wav"));
    let (l_m_pan, r_m_pan) = decode_stereo_f32(
        &ff,
        &render_clip(&dir, "mono_pan_right", json!({ "pan": 1.0 })),
    );
    assert!(
        rms(&l_m_pan) < 1e-4,
        "mono source panned hard right must have silent L"
    );
    assert!(
        rms(&r_m_pan) > 0.02,
        "mono source panned hard right must have signal on R"
    );
}

#[test]
fn validate_query_compare_and_fmt_preserve_pan_and_channels_contracts() {
    let Some(ff) = ffmpeg() else { return };
    let dir = tempdir(line!());
    write_mono_wav(&ff, &dir.join("mono.wav"));
    write_symmetric_stereo_wav(&ff, &dir.join("stereo.wav"));

    let doc = json!({
        "frame": {"width": 200, "height": 200},
        "fps": 25,
        "duration": 2000,
        "output": "out.mp4",
        "tracks": [{
            "name": "t",
            "layer": 0,
            "elements": [
                {
                    "id": "inert_and_mono",
                    "type": "audio",
                    "start": 0,
                    "end": 1000,
                    "source": "mono.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "pan": 0.0,
                    "channels": "left"
                },
                {
                    "id": "range_and_inert_ch",
                    "type": "audio",
                    "start": 1000,
                    "end": 2000,
                    "source": "stereo.wav",
                    "source_start": 0,
                    "source_end": 1000,
                    "pan": 1.5,
                    "channels": "stereo"
                }
            ]
        }]
    });
    let file = write_project(&dir, "p.montagent.json", &canonical(&doc.to_string()));
    let report = validate(&file);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"E-PAN-RANGE"), "got {codes:?}");
    assert!(codes.contains(&"R-PAN-INERT"), "got {codes:?}");
    assert!(codes.contains(&"R-CHANNELS-INERT"), "got {codes:?}");
    assert!(
        codes.contains(&"R-CHANNELS-ON-MONO-SOURCE"),
        "got {codes:?}"
    );

    // fmt preserves explicit pan: 0 and channels: "stereo".
    let fmt_doc = json!({
        "frame": {"width": 200, "height": 200},
        "fps": 25,
        "duration": 1000,
        "output": "out.mp4",
        "tracks": [{
            "name": "t",
            "layer": 0,
            "elements": [{
                "id": "a1",
                "type": "audio",
                "start": 0,
                "end": 1000,
                "source": "stereo.wav",
                "source_start": 0,
                "source_end": 1000,
                "pan": 0.0,
                "channels": "stereo"
            }]
        }]
    });
    let text = canonical(&fmt_doc.to_string());
    assert!(
        text.contains("\"pan\":0.0"),
        "fmt must preserve explicit pan: 0.0: {text}"
    );
    assert!(
        text.contains("\"channels\":\"stereo\""),
        "fmt must preserve explicit channels: stereo: {text}"
    );

    // query --at and compare:
    let before = write_project(&dir, "before.montagent.json", &text);
    let after_doc = json!({
        "frame": {"width": 200, "height": 200},
        "fps": 25,
        "duration": 1000,
        "output": "out.mp4",
        "tracks": [{
            "name": "t",
            "layer": 0,
            "elements": [{
                "id": "a1",
                "type": "audio",
                "start": 0,
                "end": 1000,
                "source": "stereo.wav",
                "source_start": 0,
                "source_end": 1000,
                "pan": -0.5,
                "channels": "swap"
            }]
        }]
    });
    let after = write_project(
        &dir,
        "after.montagent.json",
        &canonical(&after_doc.to_string()),
    );

    let at_json = query(
        &after,
        &QueryAsk {
            at: Some(500),
            ..QueryAsk::default()
        },
    )
    .to_json();
    let present = &at_json["query"]["stack"][0];
    assert_eq!(present["pan"], json!(-0.5));
    assert_eq!(present["channels"], json!("swap"));

    let diff = compare(&before, &after);
    assert!(diff.findings.is_empty());
}
