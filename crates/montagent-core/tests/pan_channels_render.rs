//! End-to-end validation, `query --at`, and `render` (`mix_audio` PCM `f64le`) tests for `pan` and
//! `channels` (`ADR-0184`, `#1154`).

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use montagent_core::media::probe::ProcessRunner;
use montagent_core::media::session::Session;
use montagent_core::media::tools::Tools;
use montagent_core::verbs::render::mix_audio;
use serde_json::json;

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

fn channel_rms_db(samples: &[(f64, f64)]) -> (f64, f64) {
    let n = samples.len() as f64;
    let sum_l: f64 = samples.iter().map(|(l, _)| l * l).sum();
    let sum_r: f64 = samples.iter().map(|(_, r)| r * r).sum();
    let db = |mean_sq: f64| 10.0 * mean_sq.max(1e-30).log10();
    (db(sum_l / n), db(sum_r / n))
}

fn make_lavfi_wav(dir: &Path, name: &str, filter: &str, channels: u32) -> PathBuf {
    let out = dir.join(name);
    let ch_str = channels.to_string();
    let status = Command::new(ffmpeg())
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            filter,
            "-ar",
            "48000",
            "-ac",
            &ch_str,
            "-c:a",
            "pcm_f64le",
            &out.to_string_lossy(),
        ])
        .status()
        .expect("spawn ffmpeg for lavfi wav");
    assert!(status.success(), "ffmpeg lavfi wav failed: {filter}");
    out
}

fn make_uncorrelated_noise_wav(dir: &Path, name: &str) -> PathBuf {
    let out = dir.join(name);
    let status = Command::new(ffmpeg())
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=d=2:c=white:r=48000:a=0.5:seed=1",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=d=2:c=white:r=48000:a=0.5:seed=2",
            "-filter_complex",
            "[0:a][1:a]amerge=inputs=2",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f64le",
            &out.to_string_lossy(),
        ])
        .status()
        .expect("spawn ffmpeg for uncorrelated noise wav");
    assert!(status.success());
    out
}

#[test]
fn adr_0184_t1_to_t11_pcm_pan_law_channel_routing_impulse_and_bypass_identity() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());

    // Sources:
    // 1. Correlated stereo 997 Hz sine (`L = R`).
    let corr_wav = make_lavfi_wav(
        &dir,
        "corr.wav",
        "aevalsrc='0.5*sin(2*PI*997*t)|0.5*sin(2*PI*997*t)':s=48000:d=1",
        2,
    );
    let corr_src = corr_wav.file_name().unwrap().to_str().unwrap();

    // 2. Stereo `L = 440 Hz, R = 880 Hz`.
    let lr_wav = make_lavfi_wav(
        &dir,
        "lr_tones.wav",
        "aevalsrc='0.5*sin(2*PI*440*t)|0.5*sin(2*PI*880*t)':s=48000:d=1",
        2,
    );
    let lr_src = lr_wav.file_name().unwrap().to_str().unwrap();

    // 3. Anti-phase stereo `L = -R`.
    let anti_wav = make_lavfi_wav(
        &dir,
        "anti.wav",
        "aevalsrc='0.5*sin(2*PI*997*t)|-0.5*sin(2*PI*997*t)':s=48000:d=1",
        2,
    );
    let anti_src = anti_wav.file_name().unwrap().to_str().unwrap();

    // 4. Left-only stereo (`L = 997 Hz sine, R = 0`).
    let left_only_wav = make_lavfi_wav(
        &dir,
        "left_only.wav",
        "aevalsrc='0.5*sin(2*PI*997*t)|0':s=48000:d=1",
        2,
    );
    let left_only_src = left_only_wav.file_name().unwrap().to_str().unwrap();

    // 5. Impulse at sample 100 (`d = 1 s`, 48,000 samples).
    let imp_wav = make_lavfi_wav(
        &dir,
        "impulse100.wav",
        "aevalsrc='if(eq(n\\,100)\\,0.8\\,0)|if(eq(n\\,100)\\,0.4\\,0)':s=48000:d=1",
        2,
    );
    let imp_src = imp_wav.file_name().unwrap().to_str().unwrap();

    // 6. Seeded uncorrelated white noise (`seed=1` L, `seed=2` R, `d = 2 s`).
    let noise_wav = make_uncorrelated_noise_wav(&dir, "uncorr_noise.wav");
    let noise_src = noise_wav.file_name().unwrap().to_str().unwrap();

    // Baseline correlated render (`pan` and `channels` omitted).
    let (corr_base_graph, corr_base) = mix_f64_stereo(
        &dir,
        "corr_base.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{corr_src}","source_start":0,"source_end":1000}}
                ]}}
            ]}}"#
        ),
    );
    assert!(
        !corr_base_graph.contains("pan=stereo"),
        "T10: omitted pan and channels must emit no pan=stereo filter: {corr_base_graph}"
    );
    let (base_l_db, base_r_db) = channel_rms_db(&corr_base);

    // T1, T2, T9, T11: Balance pan law at p in {-1, -0.5, 0, 0.5, 1} within 0.01 dB, exact 0.0 on
    // the far channel at p = ±1, bypass identity at p = 0, and no `<` in any `pan=` filter.
    for (idx, p) in [-1.0_f64, -0.5, 0.0, 0.5, 1.0].into_iter().enumerate() {
        let (panned_graph, panned) = mix_f64_stereo(
            &dir,
            &format!("pan_{idx}.montagent.json"),
            1000,
            &format!(
                r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                    {{"name":"a","layer":1,"elements":[
                        {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{corr_src}","source_start":0,"source_end":1000,"pan":{p}}}
                    ]}}
                ]}}"#
            ),
        );
        assert!(
            !panned_graph.contains('<'),
            "T11: rendered graph must never contain `<`: {panned_graph}"
        );
        if p == -1.0 {
            let (l_db, _) = channel_rms_db(&panned);
            assert!(
                (l_db - base_l_db).abs() <= 0.01,
                "T1 p=-1 L gain error: {}",
                (l_db - base_l_db).abs()
            );
            assert!(
                panned.iter().all(|(_, r)| *r == 0.0),
                "T2: pan=-1 must leave R at exact 0.0 on every sample"
            );
        } else if p == 1.0 {
            let (_, r_db) = channel_rms_db(&panned);
            assert!(
                (r_db - base_r_db).abs() <= 0.01,
                "T1 p=1 R gain error: {}",
                (r_db - base_r_db).abs()
            );
            assert!(
                panned.iter().all(|(l, _)| *l == 0.0),
                "T2: pan=1 must leave L at exact 0.0 on every sample"
            );
        } else {
            let exp_l_db = 20.0 * (1.0 - p).min(1.0).log10();
            let exp_r_db = 20.0 * (1.0 + p).min(1.0).log10();
            let (l_db, r_db) = channel_rms_db(&panned);
            let dl = (l_db - base_l_db) - exp_l_db;
            let dr = (r_db - base_r_db) - exp_r_db;
            assert!(
                dl.abs() <= 0.01 && dr.abs() <= 0.01,
                "T1 p={p}: dl={dl:.5} dB, dr={dr:.5} dB (> 0.01 dB)"
            );
        }

        // T9: `pan: 0` is bit-identical and graph-identical to omitted `pan`.
        if p == 0.0 {
            assert_eq!(panned_graph, corr_base_graph);
            assert_eq!(
                panned, corr_base,
                "T9: pan=0 must be bit-identical to omitted pan"
            );
        }
    }

    // T9 & T10: `channels: "stereo"` is bit-identical and graph-identical to omitted `channels`.
    let (corr_stereo_graph, corr_stereo) = mix_f64_stereo(
        &dir,
        "corr_stereo.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{corr_src}","source_start":0,"source_end":1000,"channels":"stereo"}}
                ]}}
            ]}}"#
        ),
    );
    assert_eq!(corr_stereo_graph, corr_base_graph);
    assert_eq!(
        corr_stereo, corr_base,
        "T9/T10: channels=\"stereo\" must be bit-identical to omitted channels"
    );

    // T3 & T4: On `L = 440 Hz, R = 880 Hz`, `pan: 1` leaves R bit-identical to source R (no L
    // bleed), and `channels: "swap" | "left" | "right"` are bit-identical to `(R, L)`, `(L, L)`,
    // `(R, R)` of the bypass render.
    let (_, lr_base) = mix_f64_stereo(
        &dir,
        "lr_base.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{lr_src}","source_start":0,"source_end":1000}}
                ]}}
            ]}}"#
        ),
    );
    let (_, lr_pan1) = mix_f64_stereo(
        &dir,
        "lr_pan1.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{lr_src}","source_start":0,"source_end":1000,"pan":1}}
                ]}}
            ]}}"#
        ),
    );
    assert!(
        lr_pan1
            .iter()
            .zip(lr_base.iter())
            .all(|((l, r), (_, base_r))| *l == 0.0 && r.to_bits() == base_r.to_bits()),
        "T3: pan=1 on L=440 Hz, R=880 Hz must leave L=0 and R bit-identical to source R"
    );

    for mode in ["swap", "left", "right"] {
        let (routed_graph, routed) = mix_f64_stereo(
            &dir,
            &format!("lr_{mode}.montagent.json"),
            1000,
            &format!(
                r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                    {{"name":"a","layer":1,"elements":[
                        {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{lr_src}","source_start":0,"source_end":1000,"channels":"{mode}"}}
                    ]}}
                ]}}"#
            ),
        );
        assert!(
            !routed_graph.contains('<'),
            "T11: rendered graph must never contain `<`: {routed_graph}"
        );
        for (i, ((out_l, out_r), (src_l, src_r))) in routed.iter().zip(lr_base.iter()).enumerate() {
            let (exp_l, exp_r) = match mode {
                "swap" => (*src_r, *src_l),
                "left" => (*src_l, *src_l),
                "right" => (*src_r, *src_r),
                _ => unreachable!(),
            };
            assert!(
                out_l.to_bits() == exp_l.to_bits() && out_r.to_bits() == exp_r.to_bits(),
                "T4 mode={mode} sample {i}: got ({out_l}, {out_r}), expected ({exp_l}, {exp_r})"
            );
        }
    }

    // T5: `channels: "mono"` on correlated `L = R` (`0.00 ± 0.01 dB`), on seeded uncorrelated
    // noise (`-3.0 ± 0.15 dB`), and on anti-phase `L = -R` (exact 0.0).
    let (mono_graph, corr_mono) = mix_f64_stereo(
        &dir,
        "corr_mono.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{corr_src}","source_start":0,"source_end":1000,"channels":"mono"}}
                ]}}
            ]}}"#
        ),
    );
    assert!(
        !mono_graph.contains('<'),
        "T11: rendered graph must never contain `<`: {mono_graph}"
    );
    let (mono_l_db, mono_r_db) = channel_rms_db(&corr_mono);
    assert!(
        (mono_l_db - base_l_db).abs() <= 0.01 && (mono_r_db - base_r_db).abs() <= 0.01,
        "T5 correlated mono: dl={}, dr={}",
        mono_l_db - base_l_db,
        mono_r_db - base_r_db
    );

    let (_, noise_base) = mix_f64_stereo(
        &dir,
        "noise_base.montagent.json",
        2000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":2000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":2000,"source":"{noise_src}","source_start":0,"source_end":2000}}
                ]}}
            ]}}"#
        ),
    );
    let (_, noise_mono) = mix_f64_stereo(
        &dir,
        "noise_mono.montagent.json",
        2000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":2000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":2000,"source":"{noise_src}","source_start":0,"source_end":2000,"channels":"mono"}}
                ]}}
            ]}}"#
        ),
    );
    let (nb_l_db, _) = channel_rms_db(&noise_base);
    let (nm_l_db, nm_r_db) = channel_rms_db(&noise_mono);
    assert!(
        ((nm_l_db - nb_l_db) - (-3.0103)).abs() <= 0.15
            && ((nm_r_db - nb_l_db) - (-3.0103)).abs() <= 0.15,
        "T5 uncorrelated noise mono: delta_l={:.3} dB, delta_r={:.3} dB",
        nm_l_db - nb_l_db,
        nm_r_db - nb_l_db
    );

    let (_, anti_mono) = mix_f64_stereo(
        &dir,
        "anti_mono.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{anti_src}","source_start":0,"source_end":1000,"channels":"mono"}}
                ]}}
            ]}}"#
        ),
    );
    assert!(
        anti_mono.iter().all(|(l, r)| *l == 0.0 && *r == 0.0),
        "T5 anti-phase L=-R mono must cancel to exact zero"
    );

    // T6 & T7: On left-only stereo (`R = 0`), `channels: "mono"` drops L by `-6.0206 ± 0.01 dB`,
    // while `channels: "left"` preserves L at `0.00 ± 0.01 dB` on both channels.
    let (_, lo_base) = mix_f64_stereo(
        &dir,
        "lo_base.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{left_only_src}","source_start":0,"source_end":1000}}
                ]}}
            ]}}"#
        ),
    );
    let (lo_base_l_db, _) = channel_rms_db(&lo_base);

    let (_, lo_mono) = mix_f64_stereo(
        &dir,
        "lo_mono.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{left_only_src}","source_start":0,"source_end":1000,"channels":"mono"}}
                ]}}
            ]}}"#
        ),
    );
    let (lo_mono_l_db, lo_mono_r_db) = channel_rms_db(&lo_mono);
    assert!(
        ((lo_mono_l_db - lo_base_l_db) - (-6.0206)).abs() <= 0.01
            && ((lo_mono_r_db - lo_base_l_db) - (-6.0206)).abs() <= 0.01,
        "T6 left-only mono: dl={:.4}, dr={:.4}",
        lo_mono_l_db - lo_base_l_db,
        lo_mono_r_db - lo_base_l_db
    );

    let (_, lo_left) = mix_f64_stereo(
        &dir,
        "lo_left.montagent.json",
        1000,
        &format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{left_only_src}","source_start":0,"source_end":1000,"channels":"left"}}
                ]}}
            ]}}"#
        ),
    );
    let (lo_left_l_db, lo_left_r_db) = channel_rms_db(&lo_left);
    assert!(
        (lo_left_l_db - lo_base_l_db).abs() <= 0.01 && (lo_left_r_db - lo_base_l_db).abs() <= 0.01,
        "T7 left-only channels=\"left\": dl={:.4}, dr={:.4}",
        lo_left_l_db - lo_base_l_db,
        lo_left_r_db - lo_base_l_db
    );

    // T8: Impulse at sample 100 (`d = 1 s`, 48,000 samples) through `pan: 0.5`, `channels: "swap"`,
    // and `channels: "mono"` lands on sample 100 (`0 samples` latency) and has exact length 48,000.
    for (idx, mode) in ["swap", "mono", "left", "right"].into_iter().enumerate() {
        let (_, imp_routed) = mix_f64_stereo(
            &dir,
            &format!("imp_routed_{idx}.montagent.json"),
            1000,
            &format!(
                r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                    {{"name":"a","layer":1,"elements":[
                        {{"id":"bed","type":"audio","start":0,"end":1000,"source":"{imp_src}","source_start":0,"source_end":1000,"pan":0.5,"channels":"{mode}"}}
                    ]}}
                ]}}"#
            ),
        );
        assert_eq!(
            imp_routed.len(),
            48_000,
            "T8 mode={mode}: exact 48,000 sample length"
        );
        let peak_idx = imp_routed
            .iter()
            .position(|(l, r)| l.abs() > 0.01 || r.abs() > 0.01)
            .expect("impulse peak");
        assert_eq!(
            peak_idx, 100,
            "T8 mode={mode}: impulse must land on exact sample 100"
        );
        assert!(
            imp_routed
                .iter()
                .enumerate()
                .all(|(i, (l, r))| i == 100 || (*l == 0.0 && *r == 0.0)),
            "T8 mode={mode}: all samples other than 100 must be exact 0.0"
        );
    }
}

#[test]
fn adr_0184_t12_channels_left_runs_before_normalize_loudness_landing_within_0_1_lu() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    // Left-only stereo source (`L = 997 Hz sine, R = 0`, 3.0 s).
    let left_only_wav = make_lavfi_wav(
        &dir,
        "left_only_3s.wav",
        "aevalsrc='0.25*sin(2*PI*997*t)|0':s=48000:d=3",
        2,
    );
    let src = left_only_wav.file_name().unwrap().to_str().unwrap();

    let project = common::write_project(
        &dir,
        "t12_norm.montagent.json",
        &common::canonical(&format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":3000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"bed","type":"audio","start":0,"end":3000,"source":"{src}","source_start":0,"source_end":3000,
                      "channels":"left",
                      "audio_effects":[{{"name":"normalize_loudness","target_lufs":-18}}]}}
                ]}}
            ]}}"#
        )),
    );
    let (inputs, graph) = mix_audio(&project, 0, 3000)
        .expect("mix_audio succeeds")
        .expect("audible mix");
    let wav = dir.join("t12_out.wav");
    let mut command = Command::new(ffmpeg());
    command.args([
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "lavfi",
        "-i",
        "anullsrc=r=48000:cl=mono:d=0.1",
    ]);
    for input in &inputs {
        command.arg("-i").arg(input);
    }
    let status = command
        .arg("-filter_complex")
        .arg(&graph)
        .args([
            "-map",
            "[mix]",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-c:a",
            "pcm_f64le",
        ])
        .arg(&wav)
        .status()
        .expect("ffmpeg wav render");
    assert!(status.success());

    let integrated = common::media::ebur128(&wav)
        .0
        .expect("finite integrated_lufs");
    assert!(
        (integrated - (-18.0)).abs() <= 0.1,
        "T12: channels=\"left\" before normalize_loudness(-18) must land within 0.1 LU of -18.0 LUFS, got {integrated:.3} LUFS"
    );
}

#[test]
fn pan_and_channels_validation_and_disk_skip_honesty() {
    if !common::has_ffprobe() {
        return;
    }
    let dir = common::tempdir(line!());
    let mono_wav = make_lavfi_wav(&dir, "mono.wav", "sine=f=440:r=48000:d=1", 1);
    let stereo_wav = make_lavfi_wav(
        &dir,
        "stereo.wav",
        "aevalsrc='0.5*sin(2*PI*440*t)|0.5*sin(2*PI*880*t)':s=48000:d=1",
        2,
    );
    let mono_src = mono_wav.file_name().unwrap().to_str().unwrap();
    let stereo_src = stereo_wav.file_name().unwrap().to_str().unwrap();

    // 1. R-CHANNELS-ON-MONO-SOURCE fires on 1-channel source with channels != "stereo" and not on 2-channel source.
    let mono_project = common::write_project(
        &dir,
        "mono_check.montagent.json",
        &common::canonical(&format!(
            r#"{{"frame":{{"width":320,"height":180}},"fps":25,"duration":1000,"output":"o.mp4","tracks":[
                {{"name":"a","layer":1,"elements":[
                    {{"id":"m","type":"audio","start":0,"end":1000,"source":"{mono_src}","source_start":0,"source_end":1000,"channels":"left"}},
                    {{"id":"s","type":"audio","start":0,"end":1000,"source":"{stereo_src}","source_start":0,"source_end":1000,"channels":"left"}}
                ]}}
            ]}}"#
        )),
    );
    let report = montagent_core::verbs::validate::validate(&mono_project);
    let mono_findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "R-CHANNELS-ON-MONO-SOURCE")
        .collect();
    assert_eq!(mono_findings.len(), 1, "{:?}", report.findings);
    assert_eq!(mono_findings[0].fields["element"], json!("m"));

    // 2. Check-set honesty (ADR-0112): when the probe fails to spawn ffprobe,
    // `validate` stops before the disk set, records only `["document"]`, and emits `E-INTERNAL` (exit 70).
    let mut bad_session = Session::with(
        Tools {
            ffmpeg: PathBuf::from("/nonexistent/ffmpeg"),
            ffprobe: PathBuf::from("/nonexistent/ffprobe"),
        },
        Box::new(ProcessRunner),
    );
    let skipped_report =
        montagent_core::verbs::validate::validate_with(&mono_project, &mut bad_session);
    let skipped_json = skipped_report.to_json();
    assert_eq!(
        skipped_json["check_sets"],
        json!(["document"]),
        "failed ffprobe must not claim the disk check set ran: {skipped_json}"
    );
    assert!(
        skipped_report
            .findings
            .iter()
            .any(|f| f.code == "E-INTERNAL"),
        "failed ffprobe spawn must emit E-INTERNAL: {:?}",
        skipped_report.findings
    );

    // 3. E-PAN-RANGE, R-PAN-INERT, R-CHANNELS-INERT, and L-KEY-ORDER.
    let canonical_pan = common::canonical(
        r#"{"frame":{"width":1080,"height":1920},"fps":25,"tracks":[{"name":"a","layer":0,"elements":[
            {"id":"e1","type":"audio","start":0,"end":1000,"source":"mono.wav","source_start":0,"source_end":1000,"pan":-1.5,"channels":"stereo"},
            {"id":"e2","type":"audio","start":1000,"end":2000,"source":"mono.wav","source_start":0,"source_end":1000,"pan":0.0,"channels":"left"}
        ]}]}"#,
    );
    let reordered_pan = canonical_pan.replace(
        r#""pan":0.0,"channels":"left""#,
        r#""channels":"left","pan":0.0"#,
    );
    assert_ne!(reordered_pan, canonical_pan);
    let doc_path = common::write_project(&dir, "pan_doc_checks.montagent.json", &reordered_pan);
    let doc_report = montagent_core::verbs::validate::validate(&doc_path);

    let codes: Vec<&str> = doc_report
        .findings
        .iter()
        .map(|f| f.code.as_str())
        .collect();
    assert!(codes.contains(&"E-PAN-RANGE"), "{codes:?}");
    assert!(codes.contains(&"R-PAN-INERT"), "{codes:?}");
    assert!(codes.contains(&"R-CHANNELS-INERT"), "{codes:?}");
    assert!(codes.contains(&"L-KEY-ORDER"), "{codes:?}");

    let pan_range = doc_report
        .findings
        .iter()
        .find(|f| f.code == "E-PAN-RANGE")
        .unwrap();
    assert_eq!(
        serde_json::to_value(&pan_range.repair).unwrap(),
        json!({"key": "pan", "value": -1.0})
    );
}
