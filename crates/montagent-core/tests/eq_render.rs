//! What the engine delivers for an EQ list on a lavfi tone (ADR-0179 §6, ADR-0173; #845 E2).
//!
//! The filters' own accuracy is measured on the graph strings in `verbs::render`'s unit tests
//! (they run on any ffmpeg). This is the other half: the real `render` of a project whose
//! audio element carries `audio_effects`, read back through the file's AAC, against the same
//! tone rendered with no list. Like `duck_render.rs` it skips where the engine's ffmpeg
//! qualification (7.1+) fails, and its tolerance is provisional until the three-leg table
//! exists (ADR-0173 §4). The deltas print on every run
//! (`cargo test -p montagent-core --test eq_render -- --nocapture`).

use montagent_core::verbs::render::{Ask, render};

mod common;
use common::media::rms_db;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// Two-sided, in dB, widened past the unit tests' 0.15 for the AAC read-back. Provisional.
const TOLERANCE_DB: f64 = 0.5;

/// Render a 1 kHz bed with `audio_effects` (a JSON list, or none) and return the file.
fn render_bed(line: u32, list: Option<&str>) -> std::path::PathBuf {
    let dir = tempdir(line);
    let bed = dir.join("bed.wav");
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
        "sine=frequency=1000:sample_rate=48000:duration=3",
    ])
    .arg(&bed)
    .status()
    .expect("ffmpeg runs")
    .success();
    assert!(made);
    let list = list.map_or(String::new(), |l| format!(r#","audio_effects":{l}"#));
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":30,"background":"#000000",
            "duration":3000,"output":"out/eq.mp4",
            "tracks":[{{"name":"music","layer":0,"elements":[
              {{"id":"bed","type":"audio","start":0,"end":3000,"source":"{}",
                "source_start":0,"source_end":3000{list}}}]}}]}}"##,
        bed.display().to_string().replace('\\', "/")
    ));
    let path = write_project(&dir, "p.montagent.json", &body);
    let answer = render(&path, &Ask::default(), &mut |_| {});
    let json = answer.to_json();
    assert!(!json["render"].is_null(), "the render answered: {json}");
    dir.join("out/eq.mp4")
}

fn delta(line: u32, list: &str) -> f64 {
    let bare = render_bed(line, None);
    let eq = render_bed(line + 1, Some(list));
    rms_db(&eq, 1.0, 2.5) - rms_db(&bare, 1.0, 2.5)
}

#[test]
fn a_pass_filter_reads_minus_3_01_db_at_its_cutoff_through_the_render() {
    if !has_ffprobe() {
        return;
    }
    for (name, slope) in [("highpass", 12), ("highpass", 48), ("lowpass", 24)] {
        let got = delta(
            line!(),
            &format!(r#"[{{"name":"{name}","frequency_hz":1000,"slope_db_per_oct":{slope}}}]"#),
        );
        eprintln!("EQ-RENDER-DELTAS {name} {slope}: {got:.3}");
        assert!((got + 3.01).abs() <= TOLERANCE_DB, "{name} {slope}: {got}");
    }
}

#[test]
fn a_bell_reads_its_gain_at_its_centre_through_the_render() {
    if !has_ffprobe() {
        return;
    }
    for gain in [-6, 6] {
        let got = delta(
            line!(),
            &format!(r#"[{{"name":"bell","frequency_hz":1000,"gain_db":{gain},"q":1}}]"#),
        );
        eprintln!("EQ-RENDER-DELTAS bell {gain}: {got:.3}");
        assert!((got - gain as f64).abs() <= TOLERANCE_DB, "{gain}: {got}");
    }
}

#[test]
fn a_bypassed_member_renders_as_the_member_absent() {
    if !has_ffprobe() {
        return;
    }
    let got = delta(
        line!(),
        r#"[{"name":"bell","frequency_hz":1000,"gain_db":12,"q":1,"enabled":false}]"#,
    );
    assert!(got.abs() <= 0.05, "{got}");
}
