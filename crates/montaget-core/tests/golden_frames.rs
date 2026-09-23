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
use common::compare::{Plane, Scope, mean_delta, rendered, ssim};
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

#[test]
fn the_paint_vocabulary_the_fixture_does_not_use_renders_the_same_way_too() {
    // The same argument as the golden above, one ticket later. The committed fixture
    // exercises exactly one member of #214's vocabulary — `handle-logo`'s param-less
    // `mask`, whose whole point (ADR-0068) is that it changes nothing there — and carries
    // no transition and no highlighted run at all. So a `skia-safe` bump could move every
    // blur kernel, every colour matrix and every shadow in the crate and the two goldens
    // above would not notice.
    //
    // One frame with the whole of it: an ordered pair of effects, two members of the same
    // name, all four colour scalars, each mask shape, a crossfade caught mid-window, and a
    // run inside its own highlight.
    let dir = tempdir(line!());
    let font = common::with_forward_slashes(
        &fixture_dir()
            .join("fonts/OpenRunde-Bold.otf")
            .display()
            .to_string(),
    );
    let project = write_project(
        &dir,
        "paint-vocabulary.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":640,"height":640}},"fps":25,"background":"#1E344C",
                "fonts":{{"brand":[{{"file":"{font}"}}]}},
                "tracks":[
                  {{"name":"effects","layer":0,"elements":[
                    {{"id":"blurred-then-shadowed","type":"rect","start":0,"end":2000,
                      "x":110,"y":110,"origin":"center","width":140,"height":100,
                      "radius":12,"fill":"#FBF3E3","effects":[
                        {{"name":"blur","radius":10}},
                        {{"name":"shadow","dx":14,"dy":14,"radius":8,"color":"#000000",
                          "opacity":0.7}}]}},
                    {{"id":"twice-shadowed","type":"rect","start":0,"end":2000,
                      "x":320,"y":110,"origin":"center","width":140,"height":100,
                      "fill":"#FFF8E8","effects":[
                        {{"name":"shadow","dx":12,"dy":0,"radius":4,"color":"#B03A2E",
                          "opacity":1}},
                        {{"name":"shadow","dx":-12,"dy":0,"radius":4,"color":"#245C8C",
                          "opacity":1}}]}},
                    {{"id":"all-four-scalars","type":"ellipse","start":0,"end":2000,
                      "x":530,"y":110,"origin":"center","width":140,"height":100,
                      "fill":"#FF8A00","effects":[
                        {{"name":"saturation","amount":0.4}},
                        {{"name":"brightness","amount":0.1}},
                        {{"name":"contrast","amount":0.3}},
                        {{"name":"tint","color":"#245C8C","amount":0.35}}]}},
                    {{"id":"masked-circle","type":"image","start":0,"end":2000,
                      "x":110,"y":280,"origin":"center","width":140,"height":100,
                      "source":"{logo}","fit":"cover",
                      "effects":[{{"name":"mask","shape":"circle"}}]}},
                    {{"id":"masked-ellipse","type":"image","start":0,"end":2000,
                      "x":320,"y":280,"origin":"center","width":140,"height":100,
                      "source":"{logo}","fit":"cover",
                      "effects":[{{"name":"mask","shape":"ellipse"}}]}},
                    {{"id":"masked-then-blurred","type":"image","start":0,"end":2000,
                      "x":530,"y":280,"origin":"center","width":140,"height":100,
                      "source":"{logo}","fit":"cover","effects":[
                        {{"name":"mask","shape":"rect"}},
                        {{"name":"blur","radius":6}}]}}
                  ]}},
                  {{"name":"outgoing","layer":1,"elements":[
                    {{"id":"leaving","type":"rect","start":0,"end":1000,"x":210,"y":440,
                      "origin":"center","width":220,"height":100,"fill":"#B03A2E"}}
                  ]}},
                  {{"name":"incoming","layer":2,"elements":[
                    {{"id":"arriving","type":"rect","start":500,"end":2000,"x":430,"y":440,
                      "origin":"center","width":220,"height":100,"fill":"#245C8C"}}
                  ]}},
                  {{"name":"bridge","layer":3,"elements":[
                    {{"id":"fade","type":"transition","start":500,"end":1000,
                      "kind":"crossfade","from":"leaving","to":"arriving"}}
                  ]}},
                  {{"name":"karaoke","layer":4,"elements":[
                    {{"id":"line","type":"text","start":0,"end":2000,"x":320,"y":570,
                      "origin":"center","width":600,"height":100,"font":"brand","size":52,
                      "color":"#FFF8E8","align":"center","runs":[
                        {{"text":"lit "}},
                        {{"text":"now","highlight":{{"start":600,"end":900,
                          "color":"#FFD34D","stroke":"#B03A2E","stroke_width":4}}}},
                        {{"text":" then"}}
                      ]}}
                  ]}}
                ]}}"##,
            logo = common::with_forward_slashes(
                &fixture_dir()
                    .join("brand/logo-en.png")
                    .display()
                    .to_string()
            ),
        )),
    );
    // 750 ms is the midpoint of the crossfade's derived window and inside the highlight's,
    // so one frame catches both mid-flight. A golden taken at either end would agree with
    // a renderer that had stopped running them.
    against_golden(
        "paint-vocabulary",
        &rendered(&project, 750, /* full */ false),
    );
}

// ---------------------------------------------------------------------------
// The mask rect under a transform (ADR-0084, #322)
// ---------------------------------------------------------------------------

/// The two frames ADR-0084's Evidence section commissions, and the identity assertion that
/// goes with them, over one project.
///
/// **Commissioned, not asserted.** Two of the three jurors independently warned that
/// agreeing with the shipped renderer is weak corroboration given ADR-0075, and asked for
/// measurement instead — ADR-0075 exists because ADR-0068 asserted a rendering fact that
/// measurement later refuted. The committed fixture cannot produce any of these frames: its
/// one mask sits on an unrotated, unscaled, `top-left` 68×68 badge and carries no geometry.
///
/// A golden is regression-only (see this file's own opening), so what these can do is hold
/// the readings fixed: the day somebody moves the mask into frame space, or centres a
/// `circle` on the element instead of on its rect, or stops the rect riding the transform,
/// all three fail at once and in a picture a reader can look at.
fn masked_under_transform(dir: &Path) -> PathBuf {
    let logo = common::with_forward_slashes(
        &fixture_dir()
            .join("brand/logo-en.png")
            .display()
            .to_string(),
    );
    write_project(
        dir,
        "mask-under-transform.montaget.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":640,"height":640}},"fps":25,"background":"#1E344C",
                "tracks":[{{"name":"masked","layer":0,"elements":[
                  {{"id":"turned","type":"image","start":0,"end":2000,"x":420,"y":330,
                    "origin":"bottom-right","width":200,"height":120,"scale":[1.6,1.6],
                    "rotation":32,"source":"{logo}","fit":"cover",
                    "effects":[{{"name":"mask","shape":"rect","x":20,"y":20,"width":120,
                                "height":80,"radius":16}}]}},
                  {{"id":"off-centre","type":"image","start":0,"end":2000,"x":60,"y":420,
                    "origin":"top-left","width":260,"height":160,
                    "source":"{logo}","fit":"cover",
                    "effects":[{{"name":"mask","shape":"circle","x":140,"y":20,
                                "width":100,"height":100}}]}}
                ]}}]}}"##
        )),
    )
}

#[test]
fn a_rotated_scaled_off_anchor_mask_rides_the_transform() {
    // Golden 1 of ADR-0084's three. `turned` is rotated 32°, scaled 1.6× and anchored
    // `bottom-right` — every one of the coordinate-space and transform questions gives a
    // *different picture* on this element, which is exactly why no committed fixture can
    // stand in for it. A `rect` mask here paints a **rotated** rectangle; a frame-space
    // mask would paint an upright one, and an `origin`-relative one would put it somewhere
    // else entirely.
    //
    // Golden 2 is in the same frame: `off-centre`'s `circle` sits in a 100×100 rect at the
    // *right-hand* end of a 260×160 element, so it is nowhere near the circle the bare form
    // would have inscribed in the element's own rect.
    let dir = tempdir(line!());
    let project = masked_under_transform(&dir);
    against_golden(
        "mask-under-transform",
        &rendered(&project, 500, /* full */ false),
    );
}

#[test]
fn the_bare_form_and_the_element_s_own_rect_written_out_are_the_same_frame() {
    // Golden 3, which is an executable assertion rather than a picture: ADR-0084's central
    // backward-compatibility claim is that the bare form is the *identity value* of the new
    // parameter set — "reached by the same arithmetic" — and not a legacy form beside it.
    // Stated as a sentence it is unfalsifiable; stated here, the day the fixture silently
    // migrates this fails.
    //
    // Asserted on the transform-laden element rather than on a plain one, because the two
    // arities could agree at rest and disagree once a rotation is in the matrix.
    //
    // **Byte equality, deliberately**, against this file's own rule for its goldens: the
    // two renders come off one rasterizer in one process, so there is no platform
    // difference for a threshold to absorb, and anything short of identical would mean the
    // identity value is a second code path.
    let dir = tempdir(line!());
    let logo = common::with_forward_slashes(
        &fixture_dir()
            .join("brand/logo-en.png")
            .display()
            .to_string(),
    );
    let render = |name: &str, mask: &str| {
        let project = write_project(
            &dir,
            &format!("{name}.montaget.json"),
            &canonical(&format!(
                r##"{{"frame":{{"width":640,"height":640}},"fps":25,"background":"#1E344C",
                    "tracks":[{{"name":"masked","layer":0,"elements":[
                      {{"id":"turned","type":"image","start":0,"end":2000,"x":400,"y":360,
                        "origin":"bottom-right","width":200,"height":120,"scale":[1.6,1.6],
                        "rotation":32,"source":"{logo}","fit":"cover",
                        "effects":[{mask}]}}
                    ]}}]}}"##
            )),
        );
        rendered(&project, 500, /* full */ false)
    };

    for shape in ["circle", "rect", "ellipse"] {
        let bare = render(
            &format!("bare-{shape}"),
            &format!(r##"{{"name":"mask","shape":"{shape}"}}"##),
        );
        let spelled_out = render(
            &format!("explicit-{shape}"),
            &format!(
                r##"{{"name":"mask","shape":"{shape}","x":0,"y":0,"width":200,"height":120}}"##
            ),
        );
        assert_eq!(
            bare.as_raw(),
            spelled_out.as_raw(),
            "a bare `{shape}` mask and one spelling the element's own rect are one \
             declaration (ADR-0084), so they must render byte-identically — and under a \
             rotation and a scale, which is where a second code path would show"
        );
    }
}
