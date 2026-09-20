//! `frame`'s budget, measured the way ADR-0021 states it: **cold**.
//!
//! *"A fully cold process: launch, load the project, resolve fonts, decode the still,
//! rasterize, encode."* That sentence is why this test lives in the binary's crate and
//! spawns the built executable rather than calling the verb in-process: an in-process
//! measurement skips process launch, dynamic linking and the whole of Skia's first-touch
//! initialisation, which is most of what a cold run actually pays for. A test that timed
//! the function would be measuring the half of the budget that was never in doubt.
//!
//! The ceiling itself is not restated here. It lives in
//! [`montaget_render::budget`] — #189 put it there *"so that the verb tickets have
//! somewhere to assert rather than each inventing a number"* — and this test judges
//! through [`Budget::judge`], so a change to the number is one diff in one file rather
//! than a hunt through the suite.
//!
//! **What the number does not yet cover.** The fixture is text-heavy — 22 of its 60
//! elements carry runs — and this build paints no glyphs (#213). So the median below is
//! the cost of the raster, the decode and the encode, and it will move when text lands.
//! ADR-0021's ceiling is the same either way; what changes is how much of it is spent, and
//! #213 is where this is measured again rather than assumed to have held.
//!
//! **What "cold" does and does not include.** Every judged run gets an **empty probe
//! sidecar** (ADR-0069), so it pays for the `ffprobe` spawns `query --at`'s crop rectangle
//! needs as well as for the raster — the strictest reading of the sentence, and the one
//! worth asserting, since a budget met only by a warm cache is a budget the agent's first
//! turn does not get.
//!
//! One run before those is discarded, and it is the only thing here that is not measured:
//! the first execution of a just-linked binary pays to page itself and Skia's shared
//! libraries in, which is a property of the machine the suite was built on rather than of
//! the verb. ADR-0021's own 0.11–0.27 s cold figures were taken the same way. Its number is
//! printed rather than hidden, so a reader can see what was excluded and how much it was.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use montaget_render::budget::{Budget, Work};

#[test]
fn frame_comes_back_under_the_budget_at_true_pixel_dimensions() {
    let Some(runs) = timed("half", &[]) else {
        return;
    };
    judge("frame, at the default half-scale answer", runs);
}

#[test]
fn the_full_scale_answer_is_under_the_same_budget() {
    // The budget is about the *raster*, not the answer: ADR-0021 is explicit that `frame`
    // is never proxy-scaled, so the surface is 1080x1920 either way and `--full` changes
    // only what the encoder is handed. If these two ever diverge, the downscale has become
    // the expensive step, which is worth knowing.
    let Some(runs) = timed("full", &["--full"]) else {
        return;
    };
    judge("frame, at true pixels", runs);
}

/// Three timed cold runs, or `None` where this build is not one the budget is stated for.
fn timed(name: &str, extra: &[&str]) -> Option<Vec<Duration>> {
    // ADR-0021's numbers were measured on optimised builds, and CI runs the suite with
    // `--release` for exactly this reason. Asserting them against an unoptimised binary
    // would be measuring a build nobody ships — so a debug run reports its numbers and
    // judges nothing, rather than failing for a reason the ceiling never claimed.
    let optimised = cfg!(not(debug_assertions));

    let dir = scratch(name);
    let out = dir.join(format!("frame-{name}.jpg"));
    let project = fixture();
    let mut args: Vec<&str> = vec![
        "frame",
        project.to_str().expect("a fixture path"),
        "--at",
        "11000",
        "--out",
        out.to_str().expect("a scratch path"),
    ];
    args.extend_from_slice(extra);

    let run = |attempt: usize| {
        // A fresh sidecar per run: nothing this process learns survives into the next one.
        let cache = dir.join(format!("cache-{attempt}"));
        let started = std::time::Instant::now();
        let output = Command::new(binary())
            .args(&args)
            .env(montaget_core::media::sidecar::CACHE_DIR_VAR, &cache)
            .output()
            .expect("run montaget frame");
        let elapsed = started.elapsed();
        assert!(
            output.status.success(),
            "frame did not answer:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        elapsed
    };

    // Discarded: this one pages the binary and Skia in. See the module doc.
    let paging = run(0);
    eprintln!("frame [{name}] discarded first execution (paging in the binary): {paging:.2?}");

    // Three, and the median is what is judged: one sample on a shared CI runner is a
    // measurement of whatever else that runner was doing.
    let runs: Vec<Duration> = (1..=3).map(run).collect();
    for (n, elapsed) in runs.iter().enumerate() {
        eprintln!(
            "frame [{name}] cold run {} (empty probe sidecar): {elapsed:.2?}",
            n + 1
        );
    }

    if !optimised {
        eprintln!(
            "skipping the assertion: this is a debug build, and ADR-0021's ceiling is \
             stated for the binary that ships. CI runs the suite with --release."
        );
        return None;
    }
    Some(runs)
}

#[track_caller]
fn judge(what: &str, mut runs: Vec<Duration>) {
    runs.sort();
    let median = runs[runs.len() / 2];
    let verdict = Budget::Frame.judge(Work::Still, median);
    assert!(
        !verdict.is_failure(),
        "{what}: the median of three cold runs was {median:.2?}, and {verdict:?}"
    );
    eprintln!("{what}: median {median:.2?}, within ADR-0021's ceiling");
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montaget"))
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/en-halloween-decorating/en-halloween-decorating.montaget.json")
        .canonicalize()
        .expect("the committed fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montaget-frame-budget/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}
