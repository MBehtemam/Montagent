//! Per-element loudness normalisation in the graph `render` writes (ADR-0178 §3, §5, §6),
//! and what `render` reports about it.
//!
//! **Conforms to ADR-0173.** The capability checks measure the PCM of the same graph `render`
//! hands the encoder ([`mix_audio`]), with the encoder swapped out (§3). The meter is
//! `ebur128=peak=true` through Montagent's own pinned parse
//! ([`montagent_core::media::loudness`]). Every signal is a lavfi generator built from
//! literal parameters in the test.
//!
//! | check | expected value, and where it comes from | sides | tolerance |
//! | --- | --- | --- | --- |
//! | level | `target_lufs` (-23, -38, -14, -6), the member's definition | two | 0.1 LU, provisional |
//! | window | `target_lufs`, measured on the placed window (trim, `speed: 2`, `loop`) | two | 0.1 LU |
//! | position | an EQ ahead of the member is measured; one after it is not | two | 0.1 LU |
//! | undefined | silence and 399 ms: no `volume=` for the member; 400 ms: one | exact | none |
//! | the 15 LU gap | voice at -23, bed at -38: 15 LU apart | two | 0.2 LU (two readings) |
//! | bypass identity | `enabled: false` ≡ absent, graph string and PCM bytes | bytes | none |
//! | partial | a `--from/--to` span applies the whole window's gain | exact | none |
//!
//! **The three-leg table is not here** (ADR-0178 §6, N4): the tolerances are the meter's
//! resolution until a three-leg run is committed.

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::verbs::render::{Ask, mix_audio, render};
use serde_json::{Value, json};

mod common;
use common::media::ebur128;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// The meter's resolution (ADR-0173 §4): `ebur128` prints LUFS to 0.1.
const METER_DB: f64 = 0.1;

fn ffmpeg() -> PathBuf {
    montagent_core::media::tools::resolve()
        .expect("an ffmpeg")
        .ffmpeg
}

/// A 48 kHz stereo `f32` WAV from a lavfi source expression.
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
        .args(["-ac", "2", "-c:a", "pcm_f32le"])
        .arg(&path)
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(made, "{lavfi}");
    path.display().to_string().replace('\\', "/")
}

/// Seeded pink noise, `ms` long.
fn pink(dir: &Path, ms: i64) -> String {
    source(
        dir,
        &format!("pink{ms}.wav"),
        &format!(
            "anoisesrc=color=pink:sample_rate=48000:amplitude=0.1:seed=7:duration={}",
            ms as f64 / 1000.0
        ),
    )
}

/// One `audio` element over `[0, ms)` of `source`, with `extra` keys merged in.
fn element(id: &str, source: &str, start: i64, ms: i64, extra: Value) -> Value {
    let mut element = json!({"id": id, "type": "audio", "start": start, "end": start + ms,
                             "source": source, "source_start": 0, "source_end": ms});
    for (key, value) in extra.as_object().expect("an object") {
        element[key] = value.clone();
    }
    element
}

fn project(dir: &Path, name: &str, duration: i64, elements: Vec<Value>) -> PathBuf {
    let body = json!({
        "frame": {"width": 64, "height": 64},
        "fps": 25,
        "duration": duration,
        "output": format!("{name}.mp4"),
        "tracks": [{"name": "sound", "layer": 0, "elements": elements}],
    });
    write_project(
        dir,
        &format!("{name}.montagent.json"),
        &canonical(&body.to_string()),
    )
}

/// One element playing `source` for `ms`, with `audio_effects` where given.
fn single(dir: &Path, name: &str, source: &str, ms: i64, effects: Option<Value>) -> PathBuf {
    let extra = match effects {
        Some(list) => json!({ "audio_effects": list }),
        None => json!({}),
    };
    project(dir, name, ms, vec![element("a", source, 0, ms, extra)])
}

fn normalize(target: f64) -> Value {
    json!({"name": "normalize_loudness", "target_lufs": target})
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

/// The integrated loudness of `[from, to)` of a project's mix PCM.
fn loudness(path: &Path, from: i64, to: i64) -> f64 {
    ebur128(&pcm(path, from, to))
        .0
        .unwrap_or_else(|| panic!("{} [{from}, {to}) is undefined", path.display()))
}

/// The `normalize_loudness` gain the graph applies, if any: the `volume=…dB` stage.
fn gain_of(graph: &str) -> Option<String> {
    graph
        .split(",volume=")
        .nth(1)
        .and_then(|rest| rest.split("dB").next())
        .filter(|gain| !gain.contains(','))
        .map(str::to_string)
}

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

#[track_caller]
fn near(measured: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (measured - expected).abs() <= tolerance + 1e-9,
        "{what}: measured {measured}, expected {expected} ± {tolerance}"
    );
}

// ---- The graph. -------------------------------------------------------------------------

#[test]
fn no_member_and_a_bypassed_member_leave_the_graph_and_the_pcm_unchanged() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let noise = pink(&dir, 2000);
    let absent = single(&dir, "absent", &noise, 2000, None);
    let today = graph(&absent, 0, 2000);
    assert!(
        !today.contains("volume=") && !today.contains("fltp"),
        "{today}"
    );
    let mut off = normalize(-23.0);
    off["enabled"] = json!(false);
    let bypassed = single(&dir, "bypassed", &noise, 2000, Some(json!([off])));
    assert_eq!(graph(&bypassed, 0, 2000), today);
    let empty = single(&dir, "empty", &noise, 2000, Some(json!([])));
    assert_eq!(graph(&empty, 0, 2000), today);
    // ADR-0173 §5: bypass identity by bytes, within one build.
    assert_eq!(
        std::fs::read(pcm(&absent, 0, 2000)).unwrap(),
        std::fs::read(pcm(&bypassed, 0, 2000)).unwrap()
    );
}

#[test]
fn the_member_is_one_fixed_volume_in_db_in_float_before_the_elements_volume() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let noise = pink(&dir, 2000);
    let path = project(
        &dir,
        "wired",
        2000,
        vec![element(
            "a",
            &noise,
            0,
            2000,
            json!({"audio_effects": [normalize(-23.0)], "volume": 0.5}),
        )],
    );
    let wired = graph(&path, 0, 2000);
    let gain = gain_of(&wired).unwrap_or_else(|| panic!("no gain in {wired}"));
    assert!(
        wired.contains(&format!(
            ",aformat=sample_fmts=fltp,volume={gain}dB,volume=0.5"
        )),
        "{wired}"
    );
    assert!(!wired.contains("loudnorm"), "{wired}");
}

// ---- The measured checks (PCM). ---------------------------------------------------------

#[test]
fn the_level_reaches_each_target() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let noise = pink(&dir, 5000);
    for target in [-23.0, -38.0, -14.0, -6.0] {
        let path = single(
            &dir,
            &format!("t{}", -target as i64),
            &noise,
            5000,
            Some(json!([normalize(target)])),
        );
        near(
            loudness(&path, 0, 5000),
            target,
            METER_DB,
            &format!("target {target}"),
        );
    }
}

/// Loud for the first 2 s, then 27 LU quieter: the whole file and the quiet part measure
/// far apart, so a whole-file measurement fails every leg here.
fn stepped(dir: &Path) -> String {
    source(
        dir,
        "stepped.wav",
        "aevalsrc=if(lt(t\\,2)\\,0.5\\,0.0224)*sin(2*PI*997*t):s=48000:d=6",
    )
}

#[test]
fn the_gain_comes_from_the_placed_window_after_trim_speed_and_loop() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let file = stepped(&dir);
    let target = -23.0;
    // Trimmed to the quiet part.
    let trimmed = project(
        &dir,
        "trimmed",
        4000,
        vec![
            json!({"id": "a", "type": "audio", "start": 0, "end": 4000, "source": file,
                    "source_start": 2000, "source_end": 6000,
                    "audio_effects": [normalize(target)]}),
        ],
    );
    near(loudness(&trimmed, 0, 4000), target, METER_DB, "trimmed");
    // The same quiet part at `speed: 2`.
    let fast = project(
        &dir,
        "fast",
        2000,
        vec![
            json!({"id": "a", "type": "audio", "start": 0, "end": 2000, "source": file,
                    "source_start": 2000, "source_end": 6000, "speed": 2,
                    "audio_effects": [normalize(target)]}),
        ],
    );
    near(loudness(&fast, 0, 2000), target, METER_DB, "speed 2");
    // A 1 s quiet slice looped to fill 5 s.
    let looped = project(
        &dir,
        "looped",
        5000,
        vec![
            json!({"id": "a", "type": "audio", "start": 0, "end": 5000, "source": file,
                    "source_start": 3000, "source_end": 4000, "overrun": "loop",
                    "audio_effects": [normalize(target)]}),
        ],
    );
    near(loudness(&looped, 0, 5000), target, METER_DB, "loop");
}

#[test]
fn the_member_measures_what_is_ahead_of_it_in_the_list_and_not_what_follows() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let sine = source(
        &dir,
        "sine.wav",
        "sine=frequency=1000:sample_rate=48000:duration=4,volume=0.1",
    );
    let bell = json!({"name": "bell", "frequency_hz": 1000, "gain_db": 12, "q": 1});
    let after = single(
        &dir,
        "eq-first",
        &sine,
        4000,
        Some(json!([bell, normalize(-23.0)])),
    );
    near(loudness(&after, 0, 4000), -23.0, METER_DB, "EQ ahead");
    let before = single(
        &dir,
        "eq-after",
        &sine,
        4000,
        Some(json!([normalize(-23.0), bell])),
    );
    near(loudness(&before, 0, 4000), -11.0, 0.2, "EQ after");
}

#[test]
fn undefined_loudness_gets_no_gain_and_a_note_and_400_ms_is_defined() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let silence = source(&dir, "silence.wav", "anullsrc=r=48000:cl=stereo:d=2");
    let silent = single(
        &dir,
        "silent",
        &silence,
        2000,
        Some(json!([normalize(-23.0)])),
    );
    assert_eq!(gain_of(&graph(&silent, 0, 2000)), None);
    let json = rendered(&silent, None, None);
    assert!(
        codes(&json).contains(&"N-NORMALIZE-UNDEFINED".to_string()),
        "{json:#}"
    );
    let block = &json["render"]["normalized"][0];
    assert_eq!(block["element"], "a", "{json:#}");
    assert_eq!(block["measured_lufs"], Value::Null);
    assert_eq!(block["applied_gain_db"], json!(0.0));

    // One 400 ms gating block is the shortest defined window.
    let short = pink(&dir, 399);
    let path = single(&dir, "short", &short, 399, Some(json!([normalize(-23.0)])));
    assert_eq!(gain_of(&graph(&path, 0, 399)), None);
    let enough = pink(&dir, 400);
    let path = single(
        &dir,
        "enough",
        &enough,
        400,
        Some(json!([normalize(-23.0)])),
    );
    assert!(gain_of(&graph(&path, 0, 400)).is_some());
}

#[test]
fn a_voice_at_minus_23_and_a_bed_at_minus_38_are_15_lu_apart() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let voice = pink(&dir, 4000);
    let bed = source(
        &dir,
        "bed.wav",
        "anoisesrc=color=brown:sample_rate=48000:amplitude=0.3:seed=11:duration=4",
    );
    // Back to back, so each element's own loudness is read off its own span of the mix.
    let path = project(
        &dir,
        "gap",
        8000,
        vec![
            element(
                "voice",
                &voice,
                0,
                4000,
                json!({"audio_effects": [normalize(-23.0)]}),
            ),
            element(
                "bed",
                &bed,
                4000,
                4000,
                json!({"audio_effects": [normalize(-38.0)]}),
            ),
        ],
    );
    let whole = pcm(&path, 0, 8000);
    let span = |from: f64, to: f64| {
        let cut = whole.with_extension(format!("{from}.wav"));
        let ok = Command::new(ffmpeg())
            .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
            .arg(&whole)
            .args([
                "-af",
                &format!("atrim=start={from}:end={to}"),
                "-c:a",
                "pcm_f32le",
            ])
            .arg(&cut)
            .status()
            .expect("ffmpeg runs")
            .success();
        assert!(ok);
        ebur128(&cut).0.expect("defined")
    };
    let gap = span(0.0, 4.0) - span(4.0, 8.0);
    near(gap, 15.0, 2.0 * METER_DB, "the voice-over-bed gap");
}

#[test]
fn a_partial_span_applies_the_whole_windows_gain() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let file = stepped(&dir);
    let path = single(
        &dir,
        "partial",
        &file,
        6000,
        Some(json!([normalize(-23.0)])),
    );
    let whole = gain_of(&graph(&path, 0, 6000)).expect("a gain");
    assert_eq!(gain_of(&graph(&path, 3000, 6000)), Some(whole.clone()));
    assert_eq!(gain_of(&graph(&path, 0, 1000)), Some(whole));
}

// ---- What `render` reports. -------------------------------------------------------------

#[test]
fn render_reports_each_members_measurement_and_gain_and_a_partial_render_the_same() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let noise = pink(&dir, 2000);
    let path = single(
        &dir,
        "reported",
        &noise,
        2000,
        Some(json!([normalize(-23.0)])),
    );
    let full = rendered(&path, None, None);
    let block = &full["render"]["normalized"];
    assert_eq!(block.as_array().map(Vec::len), Some(1), "{full:#}");
    assert_eq!(block[0]["element"], "a");
    assert_eq!(block[0]["index"], 0);
    assert_eq!(block[0]["target_lufs"], json!(-23.0));
    let measured = block[0]["measured_lufs"].as_f64().expect("defined");
    let gain = block[0]["applied_gain_db"].as_f64().expect("a gain");
    assert!(((-23.0 - measured) - gain).abs() < 0.006, "{block}");
    let partial = rendered(&path, Some(1000), Some(2000));
    assert_eq!(&partial["render"]["normalized"], block);

    // Without the member, the key is absent.
    let plain = single(&dir, "plain", &noise, 2000, None);
    let json = rendered(&plain, None, None);
    assert!(json["render"].is_object(), "{json:#}");
    assert!(json["render"].get("normalized").is_none(), "{json:#}");
}

#[test]
fn a_lift_above_20_db_is_applied_in_full_with_a_review() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let faint = source(
        &dir,
        "faint.wav",
        "sine=frequency=997:sample_rate=48000:duration=2,volume=0.03",
    );
    let path = single(&dir, "faint", &faint, 2000, Some(json!([normalize(-14.0)])));
    let json = rendered(&path, None, None);
    assert!(
        codes(&json).contains(&"R-NORMALIZE-LIFT".to_string()),
        "{json:#}"
    );
    let gain = json["render"]["normalized"][0]["applied_gain_db"]
        .as_f64()
        .expect("a gain");
    assert!(gain > 20.0, "{gain}");
}

#[test]
fn preview_reports_the_measurement_findings_too() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let silence = source(&dir, "silence.wav", "anullsrc=r=48000:cl=stereo:d=1");
    let path = single(
        &dir,
        "previewed",
        &silence,
        1000,
        Some(json!([normalize(-23.0)])),
    );
    let answer = montagent_core::verbs::preview::preview(
        &path,
        &montagent_core::verbs::preview::Ask {
            full: true,
            ..Default::default()
        },
        &mut |_| {},
    );
    let json = answer.to_json();
    assert!(
        codes(&json).contains(&"N-NORMALIZE-UNDEFINED".to_string()),
        "{json:#}"
    );
}
