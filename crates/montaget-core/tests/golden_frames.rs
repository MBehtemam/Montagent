//! **Golden frames — regression only.** Frames Montaget rendered and committed (#213).
//!
//! # What these can and cannot do, said before anything else
//!
//! Every picture under `tests/golden/` was produced by the code under test. They are
//! **self-confirming**: they can catch a change nobody meant to make, and they can never
//! say the format is wrong, because a wrong renderer would have written a wrong golden
//! with the same confidence. Spec #168 names the distinction and warns that blurring it
//! ships a suite that looks thorough and tests nothing, so the two kinds live in two files
//! with two names: `tests/reference_frames.rs` holds the frames extracted from the
//! published video, and those are the only ones that can falsify anything.
//!
//! What they *are* for is the thing ADR-0010 calls **not optional**: a `skia-safe` bump
//! can quietly shift resampling and antialiasing with nothing erroring, and a change to
//! the text stack can move a baseline by a pixel across the whole project. #34's two-arm
//! oracle already guards the prototype scene that way; this guards the real verb on the
//! real fixture.
//!
//! # Byte equality is deliberately not asserted
//!
//! #168 again, and #34 measured why: the same rasterizer on a different tier-1 target
//! takes a different SIMD path, and `skia-safe` differs by one or two least-significant
//! bits off Apple targets. So the comparison is SSIM at a threshold, plus a mean channel
//! delta — the same pair of numbers the oracle prints — and the thresholds are set where a
//! real change lands well outside them: #34 measured a re-typesetting of its scene at a
//! mean delta of 2.88 against ~0.004 for a platform difference.
//!
//! # Regenerating
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p montaget-core --test golden_frames
//! ```
//!
//! Then **look at the pictures** and put the diff in the commit. A golden updated without
//! being looked at is a golden that has stopped being a test.

use std::path::{Path, PathBuf};

mod common;
use common::compare::{Plane, Scope, rendered, ssim};
use common::{canonical, fixture_dir, fixture_project, tempdir, write_project};

/// How far a render may drift from its committed golden, as mean SSIM over the whole
/// frame.
///
/// 0.999 rather than 1.0: a last-bit difference is not a regression, and #34's matrix run
/// puts the cross-platform disagreement at a mean channel delta of 0.003–0.004 out of 255
/// — four orders of magnitude below a change anybody would see. A moved baseline or a
/// re-typeset line is nowhere near it.
const GOLDEN_SSIM: f64 = 0.999;

/// And the same drift as a mean per-pixel channel delta, out of 255.
///
/// Both, because they fail on different things: SSIM is local and structural and barely
/// notices a uniform shift in level, while a mean delta barely notices a small feature
/// moving. #34 kept the same pair for the same reason.
const GOLDEN_MEAN_DELTA: f64 = 0.5;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn updating() -> bool {
    std::env::var_os("UPDATE_GOLDEN").is_some()
}

/// Compare one render against its committed golden, or write the golden where the caller
/// asked for that.
#[track_caller]
fn against_golden(name: &str, ours: &image::RgbaImage) {
    let path = golden_dir().join(format!("{name}.png"));
    if updating() {
        std::fs::create_dir_all(golden_dir()).expect("the golden directory");
        ours.save(&path).expect("write the golden");
        println!("wrote {}", path.display());
        return;
    }
    let golden = image::open(&path)
        .unwrap_or_else(|e| {
            panic!(
                "reading the committed golden {}: {e}\nRun with UPDATE_GOLDEN=1 to create \
                 it, then look at the picture before committing it.",
                path.display()
            )
        })
        .to_rgba8();
    assert_eq!(
        (golden.width(), golden.height()),
        (ours.width(), ours.height()),
        "{name}: the render is {}x{} and the golden is {}x{}",
        ours.width(),
        ours.height(),
        golden.width(),
        golden.height()
    );

    let score = ssim(&Plane::of(&golden), &Plane::of(ours), &Scope::whole());
    let delta = mean_delta(&golden, ours);
    println!("GOLDEN  {name}: SSIM {score:.6}, mean channel delta {delta:.4}");
    assert!(
        score >= GOLDEN_SSIM && delta <= GOLDEN_MEAN_DELTA,
        "{name} has drifted from its committed golden: SSIM {score:.6} (floor \
         {GOLDEN_SSIM}), mean channel delta {delta:.4} (ceiling {GOLDEN_MEAN_DELTA}). If \
         the change is intended, look at both pictures, regenerate with UPDATE_GOLDEN=1, \
         and put the diff in the commit."
    );
}

fn mean_delta(a: &image::RgbaImage, b: &image::RgbaImage) -> f64 {
    let total: u64 = a
        .pixels()
        .zip(b.pixels())
        .map(|(a, b)| {
            (0..3)
                .map(|c| u64::from(a.0[c].abs_diff(b.0[c])))
                .sum::<u64>()
        })
        .sum();
    total as f64 / (a.pixels().len() * 3) as f64
}

#[test]
fn the_fixture_renders_the_same_two_frames_it_rendered_before() {
    // The same two instants `tests/reference_frames.rs` uses, so a regression and a
    // falsification can be read side by side — and so that a change which moves the render
    // *towards* the published video is visible as both a golden failure and a reference
    // improvement rather than only as the first.
    // Half scale — 540×960 — because ADR-0011 makes that the default answer, so the
    // golden is the picture an agent actually receives.
    let project = fixture_project();
    against_golden("fixture-intro", &rendered(&project, 400, /* full */ false));
    against_golden(
        "fixture-sentence",
        &rendered(&project, 14000, /* full */ false),
    );
}

#[test]
fn the_text_features_the_fixture_does_not_use_render_the_same_way_too() {
    // The fixture typesets nothing the golden above would catch a change to: its README
    // records that "nothing in this fixture is stroked", every text element is a single
    // run, and only two of them break a line. So the second golden is a project written
    // for the purpose, carrying exactly the text features the real one does not —
    // ADR-0014's outside stroke, ADR-0007's per-run colour and size deltas, a mandatory
    // break, and all three alignments.
    //
    // Its bytes are written here rather than committed as a fixture, because it is not a
    // fixture: it is a test's own input, it exercises no file on disk but the font, and a
    // second project file in `fixtures/` would read as a second real project.
    let dir = tempdir(line!());
    let font = common::with_forward_slashes(
        &fixture_dir()
            .join("fonts/OpenRunde-Bold.otf")
            .display()
            .to_string(),
    );
    let project = write_project(
        &dir,
        "text-features.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":640,"height":640}},"fps":25,"background":"#FBF3E3",
                "fonts":{{"brand":[{{"file":"{font}"}}]}},
                "tracks":[{{"name":"text","layer":0,"elements":[
                  {{"id":"stroked","type":"text","start":0,"end":1000,"x":320,"y":110,
                    "origin":"center","width":600,"height":120,"font":"brand","size":72,
                    "color":"#FFF8E8","align":"center","stroke":"#1E344C",
                    "stroke_width":8,"runs":[{{"text":"outlined"}}]}},
                  {{"id":"deltas","type":"text","start":0,"end":1000,"x":320,"y":260,
                    "origin":"center","width":600,"height":120,"font":"brand","size":48,
                    "color":"#245C8C","align":"center",
                    "runs":[{{"text":"base "}},
                            {{"text":"big","size":72}},
                            {{"text":" then "}},
                            {{"text":"red","color":"#B03A2E"}}]}},
                  {{"id":"broken","type":"text","start":0,"end":1000,"x":320,"y":420,
                    "origin":"center","width":600,"height":160,"font":"brand","size":44,
                    "line_height":1.3,"color":"#1E344C","align":"center",
                    "runs":[{{"text":"a break\nand a longer line"}}]}},
                  {{"id":"at-the-end","type":"text","start":0,"end":1000,"x":600,"y":560,
                    "origin":"center-right","width":560,"height":120,"font":"brand",
                    "size":40,"color":"#1E344C","align":"end",
                    "runs":[{{"text":"short\nthe longer line"}}]}}
                ]}}]}}"##
        )),
    );
    against_golden("text-features", &rendered(&project, 500, /* full */ false));
}
