//! What the engine does with a duck (ADR-0177 §6, ADR-0173, #838 D5).
//!
//! The duck script's maths is `ci/test_duck.py`'s. This is the other half: a steady-tone bed
//! under a project that carries a known duck curve, written by hand here and not by `duck.py`
//! (this test is about the engine, not the script), and the level the render delivers against
//! the same bed rendered with no `volume`. In dB the ratio of the two RMS levels is the level
//! the curve names.
//!
//! The windows are well inside a stretch (further than `ramp_ms + lead_ms` from any ramp), so a
//! ramp's shape does not enter. Ramp edges are exact to the sample, which is ADR-0175's promise,
//! cited here and measured by `render.rs`.
//!
//! **The encoder is not swapped out yet.** ADR-0173 §3 asks for the PCM of the same graph with
//! the encoder swapped out, and the engine has no such tap: the render reads back through the
//! file's AAC, as `render.rs`'s ADR-0175 test does. The tolerance below is provisional until
//! the three-leg table exists (ADR-0173 §4); each leg prints its deltas on every run
//! (`cargo test -p montagent-core --test duck_render -- --nocapture`).
//!
//! **Chain order (ADR-0169).** The ADR asks this test to catch an effect placed after `volume`
//! pushing a dip back up. No `audio_effects` member exists to place yet, so there is no such
//! case here; the build slice of the first such member owns adding a ducked-bed case to this
//! file.

use std::path::Path;

use montagent_core::verbs::render::{Ask, render};

mod common;
use common::media::rms_db;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// The tolerance, in dB, two-sided. Provisional: the jurors' figure (ADR-0177 §6) until the
/// three-leg table sets it to max(2 × spread, astats's print precision) (ADR-0173 §4).
const TOLERANCE_DB: f64 = 0.5;

const OVER: f64 = 0.5012; // -6 dB
const UNDER: f64 = 0.1778; // -15 dB
const END: f64 = 0.8414; // -1.5 dB

/// The curve `duck.py` writes for voice speech from 1000 to 3000 ms, and a last word that is
/// followed by an end card: a hold, a ramp down 200 ms long that ends 100 ms before the voice, a
/// hold, a ramp up, a hold in the pause, and a ramp to the end level.
const CURVE: &str = r#"[
    {"t":0,"v":0.5012},
    {"t":700,"v":0.5012,"ease":"linear"},
    {"t":900,"v":0.1778,"ease":"ease-in-out"},
    {"t":3100,"v":0.1778,"ease":"linear"},
    {"t":3300,"v":0.5012,"ease":"ease-in-out"},
    {"t":4700,"v":0.5012,"ease":"linear"},
    {"t":4900,"v":0.8414,"ease":"ease-in-out"}
]"#;

/// Render the steady-tone bed with `volume` (or none) and return the rendered file.
fn render_bed(line: u32, volume: Option<&str>) -> std::path::PathBuf {
    let dir = tempdir(line);
    let bed = dir.join("bed.wav");
    // Whole cycles (1000 Hz at 48 kHz is 48 samples a period), literal parameters.
    let made = std::process::Command::new(
        montagent_core::media::tools::resolve()
            .expect("an ffmpeg")
            .ffmpeg,
    )
    .args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=1000:sample_rate=48000:duration=6",
    ])
    .arg(&bed)
    .status()
    .expect("ffmpeg runs")
    .success();
    assert!(made);
    let volume = volume.map_or(String::new(), |v| format!(r#","volume":{v}"#));
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":30,"background":"#000000",
            "duration":6000,"output":"out/duck.mp4",
            "tracks":[{{"name":"music","layer":0,"elements":[
              {{"id":"bed","type":"audio","start":0,"end":6000,"source":"{}",
                "source_start":0,"source_end":6000{volume}}}]}}]}}"##,
        bed.display().to_string().replace('\\', "/")
    ));
    let path = write_project(&dir, "p.montagent.json", &body);
    let answer = render(&path, &Ask::default(), &mut |_| {});
    let json = answer.to_json();
    assert!(!json["render"].is_null(), "the render answered: {json}");
    assert_eq!(json["render"]["mixed"], serde_json::json!(["bed"]));
    dir.join("out/duck.mp4")
}

fn db(linear: f64) -> f64 {
    20.0 * linear.log10()
}

#[test]
fn a_bed_under_a_known_duck_curve_is_delivered_at_the_level_the_curve_names() {
    if !has_ffprobe() {
        return;
    }
    let bare = render_bed(line!(), None);
    let ducked = render_bed(line!(), Some(CURVE));
    // (label, window in seconds, the level the curve names there)
    let windows: [(&str, (f64, f64), f64); 3] = [
        ("inside speech", (1.4, 2.6), db(UNDER)),
        ("inside a pause", (3.6, 4.4), db(OVER)),
        ("after the last word", (5.3, 5.9), db(END)),
    ];
    let mut deltas = Vec::new();
    for (label, (from, to), level) in windows {
        let delta = rms_db(&ducked, from, to) - rms_db(&bare, from, to);
        deltas.push(format!("\"{label}\":{:.3}", delta - level));
        assert!(
            (delta - level).abs() <= TOLERANCE_DB,
            "{label} ({from}-{to} s): the render is {delta:.3} dB against the bare bed, the curve names {level:.3} dB"
        );
    }
    // The three-leg table (ADR-0173 §4) is made from these lines, one per CI leg.
    eprintln!("DUCK-RENDER-DELTAS {{{}}}", deltas.join(","));
}

#[test]
fn the_unducked_bed_reads_flat_so_the_ratio_is_the_curves() {
    if !has_ffprobe() {
        return;
    }
    let bare = render_bed(line!(), None);
    let early = rms_db(&bare, 1.4, 2.6);
    let late = rms_db(&bare, 3.6, 4.4);
    assert!(
        (early - late).abs() <= 0.05,
        "a steady tone: {early:.3} then {late:.3} dB"
    );
    assert!(Path::new(&bare).exists());
}
