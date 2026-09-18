//! ADR-0010's two-arm oracle, as a test that runs.
//!
//! The ADR relies on this harness twice and calls the second use **not
//! optional**:
//!
//! 1. **The correctness oracle** that keeps the `tiny-skia` exit real rather than
//!    aspirational. One shared ops list, two rasterizers, arms that must agree to
//!    antialiasing noise. This is the half that can actually falsify something,
//!    because the two arms are independent implementations: a bug that moves an
//!    element moves it in one arm only.
//! 2. **The golden-frame guard.** A `skia-safe` bump can quietly shift resampling
//!    and antialiasing with nothing erroring, and this project has twice shipped a
//!    render that completed, looked plausible and was wrong — the FFmpeg `zoompan`
//!    trap, and the headless-Chrome pipeline blank behind its text.
//!
//! **What the golden frames can and cannot do, stated so nobody reads more into a
//! green run than is there.** These frames were rendered by this harness and
//! committed, so they are self-confirming: they catch a *regression* and can never
//! falsify the format. Only
//! `fixtures/en-halloween-decorating/reference/frame-*.png`, extracted from the
//! published video, can do that — and doing it is `frame`'s ticket, not this one
//! (#168 names the distinction and warns that blurring it ships a suite that looks
//! thorough and tests nothing).

use std::path::{Path, PathBuf};
use std::process::Command;

/// The instants the oracle checks, chosen by #34: `t=0` is the intro card, and
/// `t=40` is mid-timeline with a different Ken Burns still and different
/// subtitles up. #6 found two renders that were wrong at one of these and right
/// at the other.
const INSTANTS: [u32; 2] = [0, 40];

const ARMS: [&str; 2] = ["skia", "tiny"];

/// How far the two arms may disagree, as a mean per-pixel channel delta out of
/// 255.
///
/// ADR-0010 records the arms agreeing "to antialiasing noise (mean Δ 0.05–0.07 of
/// 255)" and this harness still reproduces exactly that: 0.054 at `t=0` and 0.070
/// at `t=40`. The ceiling is set at roughly twice the worse of the two, so it
/// does not fire on noise and does fire on a disagreement about what is drawn.
const ARM_AGREEMENT_MEAN: f64 = 0.15;

/// And how much of the frame may differ visibly between the arms. Measured at
/// 0.22% (`t=0`) and 0.28% (`t=40`) — glyph and image edges, which is what two
/// different antialiasers produce.
const ARM_AGREEMENT_VISIBLE_FRACTION: f64 = 0.005;

/// How far an arm may drift from its committed golden frame.
///
/// Byte equality is deliberately not asserted (#168): the same rasterizer on a
/// different tier-1 target can take a different SIMD path, and a last-bit
/// difference is not a regression. The number is calibrated against a change that
/// *is* one — re-typesetting the scene in a different font moved the mean to
/// **2.88** and the visibly-different fraction to **1.70%** — so this ceiling
/// sits ~6x below a real visual change while absorbing last-bit noise. On the
/// machine the goldens were rendered on, the measured delta is 0.
///
/// The test prints its measurements on every run. If a tier-1 target exceeds
/// this, the fix is to record that target's measured number and widen to it
/// deliberately, not to loosen the threshold until it passes.
const GOLDEN_MEAN: f64 = 0.5;
const GOLDEN_VISIBLE_FRACTION: f64 = 0.001;

/// A channel delta above this counts as visibly different. #34's `compare.py`
/// used the same 8, so the two numbers are comparable.
const VISIBLE_DELTA: u8 = 8;

fn harness_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn scratch() -> PathBuf {
    let dir = harness_dir().join("out/oracle");
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// Render one still through one arm.
fn render(arm: &str, scene: &str, instant: u32, out: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_rast-bench"))
        .arg(format!("--backend={arm}"))
        .arg(format!("--scene={scene}"))
        .arg("--still")
        .arg(format!("--from={instant}"))
        .arg(format!("--out={}", out.display()))
        .status()
        .unwrap_or_else(|e| panic!("running the {arm} arm: {e}"));
    assert!(
        status.success(),
        "the {arm} arm failed to render {scene} at t={instant}"
    );
}

struct Delta {
    mean: f64,
    max: u8,
    visible_fraction: f64,
}

impl std::fmt::Display for Delta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mean={:.3} max={} visible={:.3}%",
            self.mean,
            self.max,
            self.visible_fraction * 100.0
        )
    }
}

/// Per-pixel delta between two frames, over the largest channel difference —
/// the same statistic `compare.py` reports, so #34's numbers stay comparable.
fn delta(a: &Path, b: &Path) -> Delta {
    let a = image::open(a)
        .unwrap_or_else(|e| panic!("{}: {e}", a.display()))
        .to_rgba8();
    let b = image::open(b)
        .unwrap_or_else(|e| panic!("{}: {e}", b.display()))
        .to_rgba8();
    assert_eq!(
        a.dimensions(),
        b.dimensions(),
        "frames differ in size, which is a defect rather than a tolerance question"
    );

    let mut total = 0u64;
    let mut max = 0u8;
    let mut visible = 0u64;
    let pixels = (a.width() as u64) * (a.height() as u64);

    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let d = (0..3).map(|c| pa.0[c].abs_diff(pb.0[c])).max().unwrap_or(0);
        total += d as u64;
        max = max.max(d);
        if d > VISIBLE_DELTA {
            visible += 1;
        }
    }

    Delta {
        mean: total as f64 / pixels as f64,
        max,
        visible_fraction: visible as f64 / pixels as f64,
    }
}

/// The correctness oracle: two independent rasterizers, one shared ops list.
///
/// This is the assertion that keeps ADR-0010's named exit real. If `skia-safe`
/// ever has to be dropped for `tiny-skia`, this is the evidence that the swap
/// draws the same picture — and while it holds, it is a standing check that
/// neither arm has quietly started drawing something else.
#[test]
fn the_two_arms_agree_to_antialiasing_noise() {
    let out = scratch();
    for instant in INSTANTS {
        let skia = out.join(format!("arms-skia-t{instant}.png"));
        let tiny = out.join(format!("arms-tiny-t{instant}.png"));
        render("skia", "scene.json", instant, &skia);
        render("tiny", "scene.json", instant, &tiny);

        let d = delta(&skia, &tiny);
        println!("arms @ t={instant}: {d}");
        assert!(
            d.mean <= ARM_AGREEMENT_MEAN,
            "the arms disagree at t={instant} beyond antialiasing noise: {d} \
             (ADR-0010 records mean 0.05-0.07; the ceiling is {ARM_AGREEMENT_MEAN})"
        );
        assert!(
            d.visible_fraction <= ARM_AGREEMENT_VISIBLE_FRACTION,
            "too much of the frame differs between the arms at t={instant}: {d}"
        );
    }
}

/// The golden-frame guard: each arm against what it drew when the goldens were
/// committed.
#[test]
fn each_arm_still_draws_its_committed_golden_frame() {
    let out = scratch();
    for arm in ARMS {
        for instant in INSTANTS {
            let rendered = out.join(format!("golden-{arm}-t{instant}.png"));
            render(arm, "scene.json", instant, &rendered);
            let golden = harness_dir().join(format!("frames/oracle/{arm}-t{instant}.png"));

            let d = delta(&rendered, &golden);
            println!("{arm} @ t={instant} vs golden: {d}");
            assert!(
                d.mean <= GOLDEN_MEAN && d.visible_fraction <= GOLDEN_VISIBLE_FRACTION,
                "{arm} no longer draws its committed golden frame at t={instant}: {d}. \
                 A `skia-safe` bump can shift resampling and antialiasing with nothing \
                 erroring (ADR-0010), so look at the frames before adjusting anything."
            );
        }
    }
}

/// Whether `ffmpeg` and `ffprobe` are on `PATH`. Decode is a subprocess in both
/// arms (ADR-0009: the licence decided that, not the speed).
fn ffmpeg_available() -> bool {
    ["ffmpeg", "ffprobe"].iter().all(|tool| {
        Command::new(tool)
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

/// The arms agree with a decoded video frame on the timeline too.
///
/// #34 exists because #6 measured four stills and zero decoded video frames, and
/// #168 warns that the fixture has zero `video` elements, so no ticket will
/// discover the decode path from it. This is the only place in the suite where a
/// decoded frame reaches a rasterizer.
///
/// Only the arms are compared, never a golden: both arms are fed the *identical*
/// bytes from one shared FFmpeg subprocess, so their agreement is independent of
/// which `ffmpeg` build produced those bytes — whereas a committed golden would
/// be asserting that every machine's decoder agrees, which is not this harness's
/// claim to make and not ADR-0010's premise.
#[test]
fn the_arms_agree_with_a_decoded_video_frame_on_the_timeline() {
    if !ffmpeg_available() {
        println!("skipped: ffmpeg and ffprobe are not both on PATH");
        return;
    }

    let out = scratch();
    let instant = 40; // inside the clip's 30..60 s window
    let skia = out.join("video-skia-t40.png");
    let tiny = out.join("video-tiny-t40.png");
    render("skia", "scene-video.json", instant, &skia);
    render("tiny", "scene-video.json", instant, &tiny);

    let d = delta(&skia, &tiny);
    println!("arms with video @ t={instant}: {d}");
    assert!(
        d.mean <= ARM_AGREEMENT_MEAN && d.visible_fraction <= ARM_AGREEMENT_VISIBLE_FRACTION,
        "the arms disagree about a decoded video frame at t={instant}: {d}"
    );
}

/// The oracle would pass trivially if it compared a frame with itself, and it
/// would pass trivially if the scene drew nothing. Both are checked, because a
/// guard that cannot fail is the failure #168 warns about.
#[test]
fn the_comparison_can_actually_tell_two_frames_apart() {
    let out = scratch();
    let a = out.join("sanity-t0.png");
    let b = out.join("sanity-t40.png");
    render("skia", "scene.json", 0, &a);
    render("skia", "scene.json", 40, &b);

    let same = delta(&a, &a);
    assert_eq!(same.max, 0, "a frame must not differ from itself");

    // Two different instants of the same scene are a different picture: a
    // different Ken Burns still, at a different zoom, under different subtitles.
    let different = delta(&a, &b);
    println!("t=0 vs t=40 (a real difference, for scale): {different}");
    assert!(
        different.mean > GOLDEN_MEAN && different.visible_fraction > GOLDEN_VISIBLE_FRACTION,
        "two different instants came out within the golden tolerance, so the tolerance \
         is too loose to catch anything: {different}"
    );
}
