//! The compressor's and the limiter's measured acceptance checks (ADR-0180 section 6,
//! ADR-0173): a port of `docs/research/audio-effects/dynamics/check_dynamics_curves.py` onto
//! the real mix graph.
//!
//! **Side of the encoder: PCM.** The document is rendered through `montagent`'s own mix graph
//! ([`montagent_core::verbs::render::mix_audio`]) with the encoder swapped for raw `f32`
//! samples on stdout, so AAC never mixes into a tolerance (ADR-0173 section 3). Levels are
//! computed from those samples here.
//!
//! **The bus is stereo.** A mono source is upmixed at -3 dB a channel before any member runs, so
//! a compressor on a mono file reads a tone 3 dB under its own level. These fixtures are stereo
//! with equal channels, which the bus carries unchanged, and levels are read off the left channel.
//!
//! **Fixtures** are lavfi tones and bursts, never speech: the compressor's transfer is exact on
//! tones and was not told apart on the narration (ADR-0180 section 6, "Not on TTS").
//!
//! **Tolerances are provisional.** [`CURVE_DB`], [`CEILING_DB`] and [`PASSES_DB`] are the
//! figures ADR-0180 section 6 states before any three-leg run; ADR-0173 section 4 replaces them
//! with `max(2 x the spread across the three CI legs, the meter's resolution)`. That table is
//! produced by CI, not by this file, and is not committed here.
//!
//! Every test skips where the pinned `ffmpeg` is not available (`has_ffprobe`), like the other
//! measured checks.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use montagent_core::verbs::render::mix_audio;
use serde_json::{Value, json};

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

const RATE: usize = 48_000;
/// Static curve at the reference timing (provisional).
const CURVE_DB: f64 = 0.3;
/// The timing sweep's band around the formula: deeper by up to 2.5 dB, shallower by 0.3.
const BAND_DB: (f64, f64) = (-2.5, 0.3);
/// A limiter's sample peak against its ceiling (provisional).
const CEILING_DB: f64 = 0.0002;
/// A tone 6 dB under the ceiling passes within this (provisional).
const PASSES_DB: f64 = 0.01;

fn ffmpeg() -> PathBuf {
    montagent_core::media::tools::resolve()
        .expect("an ffmpeg")
        .ffmpeg
}

fn lin(db: f64) -> f64 {
    10f64.powf(db / 20.0)
}

fn db(x: f64) -> f64 {
    20.0 * x.log10()
}

/// A mono WAV of `f32` samples from a lavfi source expression.
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

/// A 997 Hz sine of exactly this peak (lavfi's own `sine` is fixed at 1/8 amplitude).
///
/// 997 Hz, not 1 kHz: at 48 kHz a 1 kHz tone repeats every 48 samples, and ffmpeg 9.0's
/// `mpegts` probe reads the opening 2 KiB of some of those `f32` WAVs (-10 dB, -18 dB, ...) as a
/// transport stream at score 100, above the WAV demuxer's 99, so the source is unprobeable and
/// never reaches the mix. 997 Hz shares no period with the rate. Each tone level read here is
/// over a whole second (997 whole cycles), a limited peak, or a ratio of two like windows, so
/// the frequency moves no reading.
fn tone(dir: &Path, peak_db: f64, seconds: u32) -> String {
    source(
        dir,
        &format!("tone{peak_db}-{seconds}.wav"),
        &format!(
            "aevalsrc='{:.9}*sin(2*PI*997*t)':s=48000:d={seconds}:c=stereo",
            lin(peak_db)
        ),
    )
}

/// One `audio` element over the whole of `seconds`, carrying `members` (none when `Null`).
fn project(dir: &Path, name: &str, source: &str, seconds: u32, members: Value) -> PathBuf {
    let ms = seconds * 1000;
    let list = if members.is_null() {
        String::new()
    } else {
        format!(r#","audio_effects":{members}"#)
    };
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":25,"duration":{ms},
            "tracks":[{{"name":"t","layer":0,"elements":[
              {{"id":"a","type":"audio","start":0,"end":{ms},"source":"{source}",
                "source_start":0,"source_end":{ms}{list}}}]}}]}}"##
    ));
    write_project(dir, &format!("{name}.montagent.json"), &body)
}

/// The mix of `[0, seconds)` as `f32` PCM at 48 kHz, from the graph `montagent` writes: the left
/// channel of the stereo bus, so no downmix matrix touches a level. Sources are stereo with equal
/// channels for the same reason: a mono source is upmixed at -3 dB a channel, which the
/// compressor would then read (see the module doc).
fn pcm(path: &Path, seconds: u32) -> Vec<f32> {
    let (inputs, graph) = mix_audio(path, 0, i64::from(seconds) * 1000)
        .expect("the mix is built")
        .expect("something is audible");
    let mut command = Command::new(ffmpeg());
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
        ])
        // Input 0 is the video pipe in a render; the meter has no video, so a stand-in.
        .arg("anullsrc=r=48000:cl=mono:d=0.1");
    for input in &inputs {
        command.arg("-i").arg(input);
    }
    let out = command
        .arg("-filter_complex")
        .arg(format!("{graph};[mix]pan=mono|c0=c0[left]"))
        .args(["-map", "[left]", "-ar", "48000", "-f", "f32le", "-"])
        .stderr(Stdio::piped())
        .output()
        .expect("ffmpeg runs");
    assert!(
        out.status.success(),
        "{graph}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

fn rms_db(x: &[f32]) -> f64 {
    let mean: f64 = x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / x.len() as f64;
    10.0 * (mean + 1e-300).log10()
}

fn peak_db(x: &[f32]) -> f64 {
    db(x.iter().fold(0.0f64, |m, v| m.max(f64::from(*v).abs())))
}

fn compressor(threshold: f64, ratio: f64, attack: f64, release: f64, makeup: f64) -> Value {
    json!({"name": "compressor", "threshold_db": threshold, "ratio": ratio,
           "attack_ms": attack, "release_ms": release, "makeup_db": makeup})
}

fn limiter(ceiling: f64, release: f64) -> Value {
    json!({"name": "limiter", "ceiling_db": ceiling, "release_ms": release})
}

/// The formula of ADR-0180 section 2, every level an RMS dBFS.
fn formula(in_rms: f64, threshold: f64, ratio: f64, makeup: f64) -> f64 {
    if in_rms <= threshold {
        in_rms + makeup
    } else {
        threshold + (in_rms - threshold) / ratio + makeup
    }
}

/// The last second of a 3 s render, after the envelope has settled.
fn settled(x: &[f32]) -> &[f32] {
    &x[x.len() * 2 / 3..]
}

#[test]
fn the_static_curve_holds_at_the_reference_timing() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    for peak in [-50.0, -40.0, -30.0, -20.0, -12.0, -6.0, 0.0] {
        let src = tone(&dir, peak, 3);
        let reference = pcm(&project(&dir, "ref", &src, 3, Value::Null), 3);
        let in_rms = rms_db(settled(&reference));
        for (threshold, ratio, makeup) in [
            (-20.0, 4.0, 0.0),
            (-20.0, 4.0, 6.0),
            (-30.0, 8.0, 0.0),
            (-20.0, 1.0, 0.0),
            (-12.0, 2.0, 3.0),
        ] {
            let members = json!([compressor(threshold, ratio, 100.0, 100.0, makeup)]);
            let out = pcm(&project(&dir, "c", &src, 3, members), 3);
            let got = rms_db(settled(&out));
            let want = formula(in_rms, threshold, ratio, makeup);
            eprintln!(
                "static thr {threshold} ratio {ratio} makeup {makeup} in {in_rms:.2}: {:+.4} dB",
                got - want
            );
            assert!(
                (got - want).abs() <= CURVE_DB,
                "thr {threshold} ratio {ratio} makeup {makeup} in {in_rms:.2} dB: got {got:.3}, formula {want:.3}"
            );
        }
    }
}

#[test]
fn faster_timings_are_deeper_but_inside_the_stated_band() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let src = tone(&dir, -6.0, 4);
    let in_rms = -6.0 - 3.0103;
    let want = formula(in_rms, -20.0, 4.0, 0.0);
    for (attack, release) in [
        (1.0, 10.0),
        (5.0, 50.0),
        (5.0, 120.0),
        (20.0, 250.0),
        (50.0, 120.0),
        (100.0, 100.0),
        (200.0, 1000.0),
        (5.0, 1000.0),
    ] {
        let members = json!([compressor(-20.0, 4.0, attack, release, 0.0)]);
        let out = pcm(&project(&dir, "s", &src, 4, members), 4);
        let diff = rms_db(&out[out.len() * 3 / 4..]) - want;
        eprintln!("sweep {attack}/{release}: {diff:+.3} dB against the formula");
        assert!(
            (BAND_DB.0..=BAND_DB.1).contains(&diff),
            "attack {attack} release {release}: {diff:+.3} dB is outside {BAND_DB:?}"
        );
    }
}

/// 2 ms RMS windows (two whole cycles of 1 kHz) of a 2 s render, as `(seconds, dB)`.
fn envelope(x: &[f32]) -> Vec<(f64, f64)> {
    let win = RATE / 500;
    x.chunks_exact(win)
        .enumerate()
        .map(|(i, w)| ((i * win) as f64 / RATE as f64, rms_db(w)))
        .collect()
}

/// A level step at t = 1 s: `from` to `to` dB peak, through one compressor.
fn step(dir: &Path, name: &str, from: f64, to: f64, member: Value) -> Vec<(f64, f64)> {
    let src = source(
        dir,
        &format!("{name}.wav"),
        &format!(
            "aevalsrc='if(lt(t,1),{:.9},{:.9})*sin(2*PI*1000*t)':s=48000:d=2:c=stereo",
            lin(from),
            lin(to)
        ),
    );
    envelope(&pcm(
        &project(dir, &format!("{name}-doc"), &src, 2, json!([member])),
        2,
    ))
}

/// Seconds after the step until the output has covered 63% of its way from its level in the
/// step's first window to its settled level. Not `attack_ms`: ADR-0180 section 2 says the
/// key is not a time constant, and the check asserts the ordering only.
fn t63(env: &[(f64, f64)]) -> f64 {
    let after: Vec<_> = env.iter().filter(|(t, _)| *t >= 1.0).collect();
    let (start, end) = (after[0].1, after[after.len() - 1].1);
    let target = start + 0.63 * (end - start);
    let reached = |level: f64| {
        if end < start {
            level <= target
        } else {
            level >= target
        }
    };
    after
        .iter()
        .find(|(_, level)| reached(*level))
        .expect("it settles")
        .0
        - 1.0
}

#[test]
fn a_longer_attack_is_slower_by_more_than_a_factor_of_two() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let quick = t63(&step(
        &dir,
        "a5",
        -30.0,
        -6.0,
        compressor(-24.0, 8.0, 5.0, 250.0, 0.0),
    ));
    let slow = t63(&step(
        &dir,
        "a50",
        -30.0,
        -6.0,
        compressor(-24.0, 8.0, 50.0, 250.0, 0.0),
    ));
    eprintln!("attack: 5 ms reaches 63% in {quick:.4} s, 50 ms in {slow:.4} s");
    assert!(
        slow > 2.0 * quick,
        "attack 50: {slow} s, attack 5: {quick} s"
    );
}

#[test]
fn a_longer_release_is_slower_by_more_than_a_factor_of_two() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let quick = t63(&step(
        &dir,
        "r50",
        -6.0,
        -30.0,
        compressor(-24.0, 8.0, 5.0, 50.0, 0.0),
    ));
    let slow = t63(&step(
        &dir,
        "r500",
        -6.0,
        -30.0,
        compressor(-24.0, 8.0, 5.0, 500.0, 0.0),
    ));
    eprintln!("release: 50 ms recovers 63% in {quick:.4} s, 500 ms in {slow:.4} s");
    assert!(
        slow > 2.0 * quick,
        "release 500: {slow} s, release 50: {quick} s"
    );
}

#[test]
fn the_limiter_holds_its_ceiling_passes_a_quiet_tone_and_moves_no_onset() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    for ceiling in [-3.0, -12.0, -20.0, -24.0] {
        let members = json!([limiter(ceiling, 50.0)]);

        let hot = tone(&dir, 6.0, 2);
        let out = pcm(&project(&dir, "hot", &hot, 2, members.clone()), 2);
        let peak = peak_db(&out[RATE / 2..]);
        eprintln!("limiter {ceiling}: sample peak {peak:.6}");
        assert!(
            (peak - ceiling).abs() <= CEILING_DB,
            "ceiling {ceiling}: peak {peak:.6}"
        );

        let quiet = tone(&dir, ceiling - 6.0, 2);
        let reference = pcm(&project(&dir, "q-ref", &quiet, 2, Value::Null), 2);
        let passed = pcm(&project(&dir, "q", &quiet, 2, members.clone()), 2);
        let gain = rms_db(&passed[RATE / 2..]) - rms_db(&reference[RATE / 2..]);
        assert!(
            gain.abs() <= PASSES_DB,
            "ceiling {ceiling}: a quiet tone moved {gain:+.4} dB"
        );

        // 100 ms of silence, then a 20 ms burst at +6 dB: its first sample over 1e-4 lands on
        // the same sample with the limiter as without it (the 5 ms look-ahead is cancelled).
        let burst = source(
            &dir,
            "burst.wav",
            &format!(
                "aevalsrc='if(lt(t,0.1),0,if(lt(t,0.12),{:.9}*sin(2*PI*1000*t),0))':s=48000:d=1:c=stereo",
                lin(6.0)
            ),
        );
        let bare = pcm(&project(&dir, "b-ref", &burst, 1, Value::Null), 1);
        let limited = pcm(&project(&dir, "b", &burst, 1, members), 1);
        let onset = |x: &[f32]| x.iter().position(|v| v.abs() > 1e-4).expect("a burst");
        assert_eq!(onset(&limited), onset(&bare), "ceiling {ceiling}");
        assert_eq!(limited.len(), bare.len(), "the length is kept");
    }
}

#[test]
fn every_range_edge_renders_and_the_limiter_floor_is_alimiters_own() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let src = tone(&dir, -10.0, 1);
    let edges = json!([
        compressor(-60.0, 20.0, 0.1, 1.0, 24.0),
        compressor(0.0, 1.0, 2000.0, 9000.0, 0.0),
        compressor(-60.0, 1.0, 0.1, 9000.0, 24.0),
        limiter(-24.0, 1.0),
        limiter(-24.0, 1000.0),
        limiter(0.0, 1.0),
        limiter(0.0, 1000.0),
    ]);
    let out = pcm(&project(&dir, "edges", &src, 1, edges), 1);
    assert_eq!(out.len(), RATE);

    // The negative control: a -30 dB ceiling (0.0316) is below `alimiter`'s 0.0625 floor and
    // ffmpeg refuses it, which is why the range stops at -24.
    let refused = Command::new(ffmpeg())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "anoisesrc=d=0.2",
        ])
        .args([
            "-af",
            "alimiter=limit=0.031622777:attack=5:release=50:asc=0:level=0:latency=1",
        ])
        .args(["-f", "null", "-"])
        .output()
        .expect("ffmpeg runs");
    assert!(!refused.status.success(), "a -30 dB ceiling was accepted");
}

#[test]
fn a_bypassed_member_is_the_member_absent() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let src = tone(&dir, -6.0, 1);
    let absent = project(&dir, "absent", &src, 1, Value::Null);
    let mut off_compressor = compressor(-20.0, 4.0, 20.0, 250.0, 4.0);
    off_compressor["enabled"] = json!(false);
    let mut off_limiter = limiter(-12.0, 50.0);
    off_limiter["enabled"] = json!(false);
    let bypassed = project(
        &dir,
        "bypassed",
        &src,
        1,
        json!([off_compressor, off_limiter]),
    );

    let graph = |path: &Path| mix_audio(path, 0, 1000).unwrap().unwrap().1;
    assert_eq!(graph(&absent), graph(&bypassed), "the graph is unchanged");
    assert_eq!(pcm(&absent, 1), pcm(&bypassed, 1), "the PCM is identical");
}
