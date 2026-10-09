//! `preview`'s wall clock: **the one span budget that is enforced** (#218, story 61).
//!
//! `render`'s number was retired and not replaced (#217, ADR-0072), and the
//! full-resolution arm is observational by design — the caller asked for true pixels and
//! accepted the cost. The scrub preview is the arm that keeps a ceiling: `<5 s`, at the
//! 720p proxy target, on `skia-safe` (ADR-0021, ADR-0065). This is where that ceiling is
//! asserted, through [`Budget::measure`] and [`Measured::assert_within_budget`] — #189's
//! own entry points, so the number is the one in
//! [`montagent_render::budget::SCRUB_PREVIEW_LIMIT`] rather than a literal repeated here.
//!
//! **Ten seconds of the committed fixture**, which is the span ADR-0021 states the budget
//! for — *"a 10 s preview under 5 seconds"* — taken over the fixture's own opening rather
//! than a synthetic project, so what is measured includes real decode, real fonts and real
//! text.
//!
//! **What a failure here means.** Not "the machine is slow": the budget sits against
//! measurements of 2.68 s (4K) and 3.78 s (8K) at this target, and the fixture is 1080×1920
//! — comfortably inside. Those two were measured with bilinear sampling and no mipmaps; the
//! proxy now reads a shrunk image through one mip level, which ADR-0186 adopted after #552's
//! trilinear read took this test past the budget on every CI leg. A miss is either a real
//! regression in the paint path or a claim in ADR-0065 that no longer holds, and both are
//! worth stopping for. Note also that the verb
//! itself would have degraded to 540p before it returned; this test asserts the tier it
//! actually came back at, so a pass that was bought by degrading is not read as a pass.
//!
//! Release only, as every other budget test: a debug number is comparable to none of the
//! recorded ones. The guarantee is `skia-safe`-specific and `tiny-skia` is explicitly not
//! certified at 8K/720p (ADR-0065), which is why this asserts nothing about that backend.

use std::path::PathBuf;
use std::process::Command;

use montagent_render::budget::{Budget, Work};

/// The span ADR-0021's `<5 s` is written for.
const SCRUB_MS: i64 = 10_000;

#[test]
fn a_ten_second_scrub_preview_of_the_fixture_stays_inside_the_budget() {
    if cfg!(debug_assertions) {
        eprintln!(
            "skipping: this is a debug build, and the scrub budget is stated for the binary \
             that ships. CI runs the suite with --release."
        );
        return;
    }
    if montagent_core::media::tools::resolve().is_err() {
        eprintln!("skipping: no ffmpeg/ffprobe on PATH");
        return;
    }

    let dir = scratch("preview-budget");
    let output = dir.join("scrub.mp4");
    let project = fixture();
    let work = Work::span(SCRUB_MS);
    let (out, measured) = Budget::ScrubPreview.measure(work, || {
        Command::new(binary())
            .args([
                "preview",
                project.to_str().expect("a fixture path"),
                "--from",
                "0",
                "--to",
                &SCRUB_MS.to_string(),
                "--output",
                output.to_str().expect("a scratch path"),
                "--json",
            ])
            .env(
                montagent_core::media::sidecar::CACHE_DIR_VAR,
                dir.join("cache"),
            )
            .output()
            .expect("run montagent preview")
    });
    assert!(
        out.status.success(),
        "preview did not answer:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("the machine-readable result on stdout");
    assert_eq!(
        json["preview"]["duration_ms"].as_i64(),
        Some(SCRUB_MS),
        "the budget is stated for a 10 s scrub; a number over another span is not it"
    );
    // A pass bought by degrading is not a pass against this budget: the `<5 s` is stated
    // for the 720p target, and the tier is on the answer precisely so a reader never has to
    // assume which one it got (ADR-0021).
    assert_eq!(
        json["preview"]["tier"]["name"], "720p",
        "the scrub budget is the 720p target's; this run came back at another tier: {}",
        json["preview"]["tier"]["disclosure"]
    );
    assert_eq!(json["preview"]["tier"]["degraded"], false);

    eprintln!(
        "scrub preview: {:.2?} cold for {SCRUB_MS} ms of output at {}x{} (the 720p target, \
         from the fixture's 1080x1920)",
        measured.elapsed, json["preview"]["width"], json["preview"]["height"],
    );
    measured.assert_within_budget();
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montagent"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montagent.json")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montagent-preview-budget/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}
