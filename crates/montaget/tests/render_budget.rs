//! `render`'s budget, measured the way it was stated: *"a 60 s video renders in under two
//! minutes"* (ADR-0021's original, read linearly as
//! [`montaget_render::budget::RENDER_MS_PER_OUTPUT_SECOND`]).
//!
//! In the binary's crate and against the built executable, like `frame_budget.rs`, so the
//! number includes process launch, the `ffmpeg` spawns for the encoder and the probes, and
//! Skia's first touch. One cold run rather than three: the fixture is 65 s of 1080×1920,
//! and the ceiling it is judged against is 130 s — a measurement that lands near it is
//! worth a human's attention whichever side it falls on, and three of them would be a
//! long wait to learn what one already says.
//!
//! **The number it came in at, on the machine this was written on: 17.3 s, 3.8× realtime**
//! — every frame through the same painter `frame` uses, all 20 narration elements mixed.
//! Recorded so a later reader can see how much headroom the ceiling had, and because
//! ADR-0021 is explicit that a measured reference is a better baseline than a guessed one.
//!
//! **What "cold" includes.** An empty probe sidecar (ADR-0069), so the run pays for its
//! `ffprobe` spawns as `validate` would on a first turn.

use std::path::PathBuf;
use std::process::Command;

use montaget_render::budget::{Budget, Work};

#[test]
fn the_whole_fixture_renders_inside_the_stated_budget() {
    // ADR-0021's numbers were measured on optimised builds, and CI runs the suite with
    // `--release`. A debug run reports its number and judges nothing.
    let optimised = cfg!(not(debug_assertions));
    if !optimised {
        eprintln!(
            "skipping: this is a debug build, and the render budget is stated for the \
             binary that ships. CI runs the suite with --release."
        );
        return;
    }
    if montaget_core::media::tools::resolve().is_err() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }

    let dir = scratch("render-budget");
    let output = dir.join("fixture.mp4");
    let project = fixture();
    let started = std::time::Instant::now();
    let out = Command::new(binary())
        .args([
            "render",
            project.to_str().expect("a fixture path"),
            "--output",
            output.to_str().expect("a scratch path"),
            "--json",
        ])
        .env(
            montaget_core::media::sidecar::CACHE_DIR_VAR,
            dir.join("cache"),
        )
        .output()
        .expect("run montaget render");
    let elapsed = started.elapsed();
    assert!(
        out.status.success(),
        "render did not answer:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("the machine-readable result on stdout");
    let duration_ms = json["render"]["duration_ms"].as_i64().expect("a span");
    assert_eq!(duration_ms, 65216);

    let verdict = Budget::Render.judge(Work::span(duration_ms), elapsed);
    eprintln!(
        "render: {elapsed:.2?} cold for {duration_ms} ms of output ({:.2}x realtime), {verdict:?}",
        json["render"]["realtime"].as_f64().unwrap_or_default()
    );
    assert!(!verdict.is_failure(), "{verdict:?}");
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montaget"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montaget-render-budget/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}
