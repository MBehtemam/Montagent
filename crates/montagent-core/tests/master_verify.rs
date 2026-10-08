//! `verify`'s loudness and true peak (ADR-0172, ADR-0173 §6, ADR-0174 §3): always measured
//! on the deliverable's decoded AAC, and judged only against what `master` declares — a miss
//! is a `review`, never an error.

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::finding::Class;
use montagent_core::verbs::render::{Ask, render};
use montagent_core::verbs::verify::verify;
use serde_json::{Value, json};

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

fn ffmpeg() -> PathBuf {
    montagent_core::media::tools::resolve()
        .expect("an ffmpeg")
        .ffmpeg
}

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

fn project(dir: &Path, name: &str, source: &str, master: Option<Value>) -> PathBuf {
    let mut body = json!({
        "frame": {"width": 64, "height": 64},
        "fps": 25,
        "duration": 2000,
        "output": format!("{name}.mp4"),
        "tracks": [{"name": "sound", "layer": 0, "elements": [
            {"id": "a", "type": "audio", "start": 0, "end": 2000, "source": source,
             "source_start": 0, "source_end": 2000}]}],
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

fn rendered(path: &Path) {
    let answer = render(path, &Ask::default(), &mut |_| {});
    assert!(answer.video().is_some(), "{:#}", answer.to_json());
}

fn music(dir: &Path) -> String {
    source(dir, "music.wav", "aevalsrc=0.3*sin(2*PI*440*t):s=48000:d=2")
}

fn ours(report: &montagent_core::report::Report) -> Vec<(&str, Class)> {
    report
        .findings
        .iter()
        .filter(|f| f.code == "R-VERIFY-LOUDNESS" || f.code == "R-VERIFY-TRUE-PEAK")
        .map(|f| (f.code.as_str(), f.class))
        .collect()
}

#[test]
fn verify_always_reports_loudness_and_true_peak() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = project(&dir, "plain", &music(&dir), None);
    rendered(&path);
    let answer = verify(&path);
    let block = answer.to_json()["verify"].clone();
    let lufs = block["integrated_lufs"]
        .as_f64()
        .unwrap_or_else(|| panic!("{block:#}"));
    let peak = block["true_peak_dbtp"].as_f64().expect("a true peak");
    assert!((-40.0..0.0).contains(&lufs), "{lufs}");
    assert!((-30.0..0.0).contains(&peak), "{peak}");
    assert_eq!(
        ours(answer.report()),
        [],
        "nothing is declared, nothing judged"
    );
}

#[test]
fn a_master_the_deliverable_meets_raises_nothing() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = project(
        &dir,
        "met",
        &music(&dir),
        Some(json!({"target_lufs": -16, "ceiling_dbtp": -1})),
    );
    rendered(&path);
    let answer = verify(&path);
    let block = answer.to_json()["verify"].clone();
    let lufs = block["integrated_lufs"].as_f64().expect("loudness");
    assert!((lufs + 16.0).abs() <= 1.0, "{lufs}");
    assert!(
        block["true_peak_dbtp"].as_f64().expect("peak") <= -1.0 + 1.0,
        "{block}"
    );
    assert_eq!(ours(answer.report()), []);
}

#[test]
fn a_loudness_shortfall_from_heavy_limiting_is_a_review_naming_the_cause() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    // A target 4 dB under the ceiling's headroom: the limiter takes the loudness away.
    let path = project(
        &dir,
        "squashed",
        &music(&dir),
        Some(json!({"target_lufs": -6, "ceiling_dbtp": -10})),
    );
    rendered(&path);
    let answer = verify(&path);
    assert_eq!(
        ours(answer.report()),
        [("R-VERIFY-LOUDNESS", Class::Review)],
        "{:#}",
        answer.to_json()
    );
    let finding = answer
        .report()
        .findings
        .iter()
        .find(|f| f.code == "R-VERIFY-LOUDNESS")
        .unwrap();
    assert_eq!(finding.fields["target_lufs"], json!(-6.0));
    assert_eq!(finding.fields["tolerance_lu"], json!(1.0));
    assert!(finding.citation.is_some());
    let text =
        montagent_core::text::render(&answer.to_json(), montagent_core::text::Options::verbose())
            .expect("renders");
    assert!(text.contains("R-MASTER-HEADROOM"), "{text}");
}

#[test]
fn a_true_peak_over_the_allowance_is_a_review_never_an_error() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let path = project(&dir, "hot", &music(&dir), Some(json!({"ceiling_dbtp": -6})));
    // A file the engine did not write: hot, unattested, at the declared `output`, so the
    // measurement runs (`R-VERIFY-UNATTESTED`) on audio that ignores the ceiling.
    let made = Command::new(ffmpeg())
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "color=c=black:s=64x64:r=25:d=2"])
        .args([
            "-f",
            "lavfi",
            "-i",
            "aevalsrc=0.99*sin(2*PI*440*t):s=48000:d=2",
        ])
        .args([
            "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "160k",
        ])
        .arg(dir.join("hot.mp4"))
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(made);
    let answer = verify(&path);
    assert_eq!(
        ours(answer.report()),
        [("R-VERIFY-TRUE-PEAK", Class::Review)],
        "{:#}",
        answer.to_json()
    );
    let finding = answer
        .report()
        .findings
        .iter()
        .find(|f| f.code == "R-VERIFY-TRUE-PEAK")
        .unwrap();
    assert_eq!(finding.fields["ceiling_dbtp"], json!(-6.0));
    assert_eq!(finding.fields["tolerance_db"], json!(1.0));
    assert!(finding.fields["overshoot_db"].as_f64().unwrap() > 1.0);
    assert!(finding.citation.is_some());
}
