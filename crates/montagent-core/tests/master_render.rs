//! The master stage on the mix bus `render` writes (ADR-0172, ADR-0174), and what `render`
//! and `verify` report about it.
//!
//! **Conforms to ADR-0173.** The capability checks measure the PCM of the same graph `render`
//! hands the encoder ([`mix_audio`]), with the encoder swapped out (§3). The meter is
//! `ebur128=peak=true` through Montagent's own pinned parse
//! ([`montagent_core::media::loudness`]), so a parse that breaks fails loudly (§2). Every
//! signal is a lavfi generator built from literal parameters in the test (§3).
//!
//! | check | expected value, and where it comes from | sides | tolerance |
//! | --- | --- | --- | --- |
//! | target reached | `target_lufs`, the stage's definition | two | 0.1 LU, the meter's resolution |
//! | ceiling held (PCM) | `ceiling_dbtp`, the stage's definition | one (≤) | +0.2 dB, ADR-0174's measured 4× gap |
//! | no latency | the impulse's own sample | exact | none |
//! | no length change | the bypass's sample count | exact | none |
//! | bypass identity | `master` absent ≡ `master: {}` PCM | bytes | none, within one build |
//! | run-to-run | the same document twice | bytes | none, within one build |
//!
//! **The three-leg table is not here.** ADR-0173 §4 makes a tolerance
//! `max(2 × the spread across the CI legs, the meter's resolution)`; these use the meter's
//! resolution and ADR-0174's committed figure until a three-leg run is committed.

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::verbs::render::{Ask, mix_audio, render};
use serde_json::{Value, json};

mod common;
use common::media::ebur128;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// The meter's resolution (ADR-0173 §4): `ebur128` prints LUFS and dBTP to 0.1.
const METER_DB: f64 = 0.1;

/// ADR-0174's measured gap between the ceiling and the 4×-limited PCM's true peak.
const LIMITED_PCM_DB: f64 = 0.2;

fn ffmpeg() -> PathBuf {
    montagent_core::media::tools::resolve()
        .expect("an ffmpeg")
        .ffmpeg
}

/// A 48 kHz `f32` WAV from a lavfi source expression — the generator command is the input.
fn source(dir: &Path, name: &str, lavfi: &str) -> String {
    let path = dir.join(name);
    let made = Command::new(ffmpeg())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(lavfi)
        .args(["-c:a", "pcm_f32le"])
        .arg(&path)
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(made, "{lavfi}");
    path.display().to_string().replace('\\', "/")
}

/// One `audio` element playing `source` over `[0, ms)`, with `master` where it is given.
fn project(dir: &Path, name: &str, source: &str, ms: i64, master: Option<Value>) -> PathBuf {
    let mut body = json!({
        "frame": {"width": 64, "height": 64},
        "fps": 25,
        "duration": ms,
        "output": format!("{name}.mp4"),
        "tracks": [{"name": "sound", "layer": 0, "elements": [
            {"id": "a", "type": "audio", "start": 0, "end": ms, "source": source,
             "source_start": 0, "source_end": ms}]}],
    });
    if let Some(master) = master {
        body["master"] = master;
    }
    write_project(
        dir,
        &format!("{name}.montagent.json"),
        &canonical(&body.to_string()),
    )
}

/// The graph `render` would hand the encoder for `[from, to)`.
fn graph(path: &Path, from: i64, to: i64) -> String {
    mix_audio(path, from, to)
        .expect("the mix is built")
        .expect("something is audible")
        .1
}

/// The PCM of the mix graph over `[from, to)`, written to a WAV beside the project.
fn pcm(path: &Path, from: i64, to: i64) -> PathBuf {
    let (inputs, graph) = mix_audio(path, from, to)
        .expect("the mix is built")
        .expect("something is audible");
    let script = path.with_extension(format!("{from}-{to}.graph.txt"));
    std::fs::write(&script, graph).expect("write the graph");
    let out = path.with_extension(format!("{from}-{to}.wav"));
    let mut command = Command::new(ffmpeg());
    command
        .args(["-hide_banner", "-nostdin", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "anullsrc=r=48000:cl=stereo:d=0.01"]);
    for input in &inputs {
        command.arg("-i").arg(input);
    }
    let made = command
        .arg("-/filter_complex")
        .arg(&script)
        .args(["-map", "[mix]", "-c:a", "pcm_f32le"])
        .arg(&out)
        .output()
        .expect("ffmpeg runs");
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    out
}

/// Every sample of a WAV as written, channels interleaved.
fn samples(wav: &Path) -> Vec<f32> {
    let out = Command::new(ffmpeg())
        .args(["-v", "error", "-i"])
        .arg(wav)
        .args(["-f", "f32le", "-"])
        .output()
        .expect("ffmpeg runs");
    assert!(out.status.success());
    out.stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

/// A quiet 997 Hz sine, `seconds` long.
fn quiet_sine(dir: &Path, seconds: u32) -> String {
    source(
        dir,
        &format!("quiet{seconds}.wav"),
        &format!("sine=frequency=997:sample_rate=48000:duration={seconds},volume=0.2"),
    )
}

// ---- The graph. -------------------------------------------------------------------------

#[test]
fn without_master_the_graph_is_todays_and_an_empty_master_is_the_same_bytes() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let sine = quiet_sine(&dir, 6);
    let absent = project(&dir, "absent", &sine, 6000, None);
    let empty = project(&dir, "empty", &sine, 6000, Some(json!({})));
    let today = graph(&absent, 0, 6000);
    assert!(
        !today.contains("volume=") && !today.contains("alimiter"),
        "{today}"
    );
    assert!(
        today.ends_with("[a0]apad=whole_dur=6.000,atrim=end=6.000[mix]\n"),
        "{today}"
    );
    assert_eq!(graph(&empty, 0, 6000), today);
    // ADR-0173 §5: bypass identity by bytes, within one build.
    assert_eq!(
        std::fs::read(pcm(&absent, 0, 6000)).unwrap(),
        std::fs::read(pcm(&empty, 0, 6000)).unwrap()
    );
}

/// ADR-0174 §2's stage as a committed golden: the ceiling alone is the 4× limiter, spliced
/// between the sum and the pad, and nothing else in the graph moves.
#[test]
fn a_ceiling_alone_splices_the_4x_limiter_before_the_pad() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let sine = quiet_sine(&dir, 6);
    let today = graph(&project(&dir, "absent", &sine, 6000, None), 0, 6000);
    let limited = graph(
        &project(
            &dir,
            "limited",
            &sine,
            6000,
            Some(json!({"ceiling_dbtp": -1})),
        ),
        0,
        6000,
    );
    assert_eq!(
        limited,
        today.replace(
            "[a0]apad=",
            "[a0]aresample=192000,alimiter=limit=0.89125094:latency=1:level=0,\
             aresample=48000,apad="
        )
    );
}

// ---- The measured checks (PCM). ---------------------------------------------------------

#[test]
fn a_target_is_reached_by_one_measured_gain() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let sine = quiet_sine(&dir, 6);
    let before = ebur128(&pcm(&project(&dir, "absent", &sine, 6000, None), 0, 6000))
        .0
        .expect("defined");
    for target in [-23.0, -16.0] {
        let path = project(
            &dir,
            &format!("t{}", -target as i64),
            &sine,
            6000,
            Some(json!({ "target_lufs": target })),
        );
        let wired = graph(&path, 0, 6000);
        let gain = ((target - before) * 100.0_f64).round() / 100.0;
        assert!(
            wired.contains(&format!("volume={gain}dB,")),
            "{gain}: {wired}"
        );
        let after = ebur128(&pcm(&path, 0, 6000)).0.expect("defined");
        assert!(
            (after - target).abs() <= METER_DB + 1e-9,
            "target {target}: measured {after}"
        );
    }
}

#[test]
fn the_4x_limiter_holds_the_pcm_true_peak_to_the_ceiling() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // ADR-0173 §7's first signal: a sine at fs/4 with a 45° phase offset, whose sample peaks
    // sit 3 dB under its true peak — the case a sample-peak limiter misses.
    // At 0.99 its true peak is about 0 dBTP, so the ceilings below drive it 3 and 6 dB over.
    let fs4 = source(
        &dir,
        "fs4.wav",
        "aevalsrc=0.99*sin(2*PI*12000*t+PI/4):s=48000:d=4:c=stereo",
    );
    for ceiling in [-3.0, -6.0] {
        let path = project(
            &dir,
            &format!("fs4-{}", -ceiling as i64),
            &fs4,
            4000,
            Some(json!({ "ceiling_dbtp": ceiling })),
        );
        let peak = ebur128(&pcm(&path, 0, 4000)).1.expect("not silent");
        assert!(
            peak <= ceiling + LIMITED_PCM_DB + 1e-9,
            "ceiling {ceiling}: true peak {peak}"
        );
    }
    // And the stage whole: a target that drives a quiet sine into the ceiling.
    let sine = quiet_sine(&dir, 4);
    let path = project(
        &dir,
        "driven",
        &sine,
        4000,
        Some(json!({"target_lufs": -6, "ceiling_dbtp": -1})),
    );
    let peak = ebur128(&pcm(&path, 0, 4000)).1.expect("not silent");
    assert!(peak <= -1.0 + LIMITED_PCM_DB + 1e-9, "true peak {peak}");
}

#[test]
fn the_stage_adds_no_latency_no_length_and_is_byte_identical_run_to_run() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // One near-full-scale sample at 1 s, in silence.
    let impulse = source(
        &dir,
        "impulse.wav",
        "aevalsrc=if(eq(n\\,48000)\\,0.99\\,0):s=48000:d=2:c=stereo",
    );
    let peak_at = |wav: &Path| {
        let samples = samples(wav);
        let (at, _) = samples
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .expect("samples");
        (at, samples.len())
    };
    let bypass = peak_at(&pcm(
        &project(&dir, "bypass", &impulse, 2000, None),
        0,
        2000,
    ));
    let mastered = project(
        &dir,
        "mastered",
        &impulse,
        2000,
        Some(json!({"ceiling_dbtp": -6})),
    );
    let once = pcm(&mastered, 0, 2000);
    assert_eq!(peak_at(&once), bypass, "the impulse stays on its sample");
    let first = std::fs::read(&once).unwrap();
    let again = std::fs::read(pcm(&mastered, 0, 2000)).unwrap();
    assert_eq!(first, again, "byte-identical within one build");
}

#[test]
fn a_partial_span_applies_the_whole_programmes_gain() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // Loud for 3 s, then quiet: the second half alone would measure far lower.
    let stepped = source(
        &dir,
        "stepped.wav",
        "aevalsrc=if(lt(t\\,3)\\,0.5\\,0.02)*sin(2*PI*997*t):s=48000:d=6",
    );
    let path = project(
        &dir,
        "stepped",
        &stepped,
        6000,
        Some(json!({"target_lufs": -16, "ceiling_dbtp": -1})),
    );
    let gain_of = |graph: &str| {
        graph
            .split("volume=")
            .nth(1)
            .and_then(|rest| rest.split("dB,").next())
            .map(str::to_string)
            .unwrap_or_else(|| panic!("no gain in {graph}"))
    };
    let whole = gain_of(&graph(&path, 0, 6000));
    assert_eq!(gain_of(&graph(&path, 3000, 6000)), whole);
    assert_eq!(gain_of(&graph(&path, 0, 1000)), whole);
}

// ---- What `render` reports. -------------------------------------------------------------

fn rendered(path: &Path, from: Option<i64>, to: Option<i64>) -> Value {
    render(
        path,
        &Ask {
            from,
            to,
            ..Ask::default()
        },
        &mut |_| {},
    )
    .to_json()
}

fn codes(json: &Value) -> Vec<String> {
    json["findings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|f| f["code"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn render_reports_the_measurement_and_the_gain_and_a_partial_render_the_same() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let sine = quiet_sine(&dir, 2);
    let path = project(
        &dir,
        "reported",
        &sine,
        2000,
        Some(json!({"target_lufs": -16, "ceiling_dbtp": -1})),
    );
    let full = rendered(&path, None, None);
    let block = &full["render"];
    let measured = block["measured_lufs"]
        .as_f64()
        .unwrap_or_else(|| panic!("{full:#}"));
    let gain = block["applied_gain_db"].as_f64().expect("a gain");
    assert!(((-16.0 - measured) - gain).abs() < 0.006, "{block}");
    let partial = rendered(&path, Some(1000), Some(2000));
    assert_eq!(partial["render"]["measured_lufs"], block["measured_lufs"]);
    assert_eq!(
        partial["render"]["applied_gain_db"],
        block["applied_gain_db"]
    );

    // Without `target_lufs`, the keys are absent.
    let plain = project(
        &dir,
        "plain",
        &sine,
        2000,
        Some(json!({"ceiling_dbtp": -1})),
    );
    let block = rendered(&plain, None, None)["render"].clone();
    assert!(block.is_object(), "{block}");
    assert!(block.get("measured_lufs").is_none() && block.get("applied_gain_db").is_none());
}

#[test]
fn undefined_loudness_gets_no_gain_and_a_finding() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let silence = source(&dir, "silence.wav", "anullsrc=r=48000:cl=stereo:d=2");
    let path = project(
        &dir,
        "silent",
        &silence,
        2000,
        Some(json!({"target_lufs": -16, "ceiling_dbtp": -1})),
    );
    assert!(!graph(&path, 0, 2000).contains("volume="));
    let json = rendered(&path, None, None);
    assert!(
        codes(&json).contains(&"R-MASTER-LOUDNESS-UNDEFINED".to_string()),
        "{json:#}"
    );
    assert_eq!(json["render"]["measured_lufs"], Value::Null, "{json:#}");
    assert_eq!(json["render"]["applied_gain_db"], json!(0.0));
}

#[test]
fn a_gain_above_20_db_is_applied_in_full_with_a_finding() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let faint = source(
        &dir,
        "faint.wav",
        "sine=frequency=997:sample_rate=48000:duration=2,volume=0.05",
    );
    let path = project(
        &dir,
        "faint",
        &faint,
        2000,
        Some(json!({"target_lufs": -16, "ceiling_dbtp": -1})),
    );
    let json = rendered(&path, None, None);
    assert!(
        codes(&json).contains(&"R-MASTER-GAIN-HIGH".to_string()),
        "{json:#}"
    );
    let gain = json["render"]["applied_gain_db"].as_f64().expect("a gain");
    assert!(gain > 20.0, "{gain}");
}
