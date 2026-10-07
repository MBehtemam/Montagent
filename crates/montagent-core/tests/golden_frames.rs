//! **Golden frames — regression only.** Frames Montagent rendered and committed (#213).
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
//! UPDATE_GOLDEN=1 cargo test -p montagent-core --test golden_frames
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
        "text-features.montagent.json",
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
        "paint-vocabulary.montagent.json",
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

/// One masked `image` on a 640×640 frame, carrying whatever element body is handed to it.
///
/// **Commissioned, not asserted.** Two of ADR-0084's three jurors independently warned that
/// agreeing with the shipped renderer is weak corroboration given ADR-0075, and asked for
/// measurement instead — ADR-0075 exists because ADR-0068 asserted a rendering fact that
/// measurement later refuted. The committed fixture can produce none of the frames below:
/// its one mask sits on an unrotated, unscaled, `top-left` 68×68 badge and carries no
/// geometry at all.
///
/// A golden is regression-only (see this file's own opening), so what these do is hold the
/// readings fixed: the day somebody moves the mask into frame space, or centres a `circle`
/// on the element instead of on its rect, or stops the rect riding the transform, the
/// picture changes and a reader can look at it.
fn masked(dir: &Path, name: &str, element: &str) -> PathBuf {
    write_project(
        dir,
        &format!("{name}.montagent.json"),
        &canonical(&format!(
            r##"{{"frame":{{"width":640,"height":640}},"fps":25,"background":"#1E344C",
                "tracks":[{{"name":"masked","layer":0,"elements":[{element}]}}]}}"##
        )),
    )
}

/// The `brand/logo-en.png` the fixture already carries, as a `source` a temp project
/// resolves.
fn logo() -> String {
    common::with_forward_slashes(
        &fixture_dir()
            .join("brand/logo-en.png")
            .display()
            .to_string(),
    )
}

#[test]
fn a_rotated_scaled_off_anchor_mask_rides_the_transform() {
    // The first frame ADR-0084 commissions, and the one that "distinguishes every candidate
    // answer to the coordinate-space and transform questions": rotated 32°, scaled 1.6×,
    // anchored `bottom-right`. A `rect` mask here paints a **rotated** rounded rectangle; a
    // frame-space mask would paint an upright one, an `origin`-relative one would put it
    // somewhere else entirely, and a mask that refused the transform would paint it at the
    // unscaled size.
    let dir = tempdir(line!());
    let project = masked(
        &dir,
        "mask-under-transform",
        &format!(
            r##"{{"id":"turned","type":"image","start":0,"end":2000,"x":420,"y":400,
                "origin":"bottom-right","width":200,"height":120,"scale":[1.6,1.6],
                "rotation":32,"source":"{logo}","fit":"cover",
                "effects":[{{"name":"mask","shape":"rect","x":20,"y":20,"width":120,
                            "height":80,"radius":16}}]}}"##,
            logo = logo()
        ),
    );
    against_golden(
        "mask-under-transform",
        &rendered(&project, 500, /* full */ false),
    );
}

#[test]
fn an_explicit_mask_rect_that_is_not_the_elements_own_is_its_own_frame() {
    // The second frame ADR-0084 commissions — "an explicit mask rect that differs from its
    // element's rect" — with its own golden rather than a second element in the frame
    // above, so a change to one reading cannot be read off a picture the other also moves.
    //
    // The `circle` sits in a 100×100 rect at the *right-hand* end of a 260×160 element, so
    // it is nowhere near the 160 px circle the bare form would have inscribed in the
    // element's own rect — which is the whole of what "the rect the shape is inscribed in"
    // buys over ADR-0068's form.
    let dir = tempdir(line!());
    let project = masked(
        &dir,
        "mask-explicit-rect",
        &format!(
            r##"{{"id":"off-centre","type":"image","start":0,"end":2000,"x":190,"y":240,
                "origin":"top-left","width":260,"height":160,
                "source":"{logo}","fit":"cover",
                "effects":[{{"name":"mask","shape":"circle","x":140,"y":20,
                            "width":100,"height":100}}]}}"##,
            logo = logo()
        ),
    );
    against_golden(
        "mask-explicit-rect",
        &rendered(&project, 500, /* full */ false),
    );
}

#[test]
fn the_bare_form_and_the_elements_own_rect_written_out_are_the_same_frame() {
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
    let render = |name: &str, mask: &str| {
        let project = masked(
            &dir,
            name,
            &format!(
                r##"{{"id":"turned","type":"image","start":0,"end":2000,"x":420,"y":400,
                    "origin":"bottom-right","width":200,"height":120,"scale":[1.6,1.6],
                    "rotation":32,"source":"{logo}","fit":"cover",
                    "effects":[{mask}]}}"##,
                logo = logo()
            ),
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

/// `frame --from --to`'s sheet of `[from, to)`, decoded.
fn sheet_of(project: &Path, from: i64, to: i64) -> image::RgbaImage {
    use montagent_core::report::ExitCode;
    use montagent_core::verbs::frame::{Ask, frame};
    let answer = frame(
        project,
        &Ask {
            from: Some(from),
            to: Some(to),
            png: true,
            ..Ask::default()
        },
    );
    assert_eq!(answer.report().exit_code(), ExitCode::Ok, "no sheet");
    image::load_from_memory(&answer.image().expect("a sheet").bytes)
        .expect("the bytes decode as a picture")
        .to_rgba8()
}

#[test]
fn a_labelled_sheet_draws_its_labels_beneath_its_tiles_the_same_way_it_did_before() {
    // The chrome face's first pixels (#490): four labels in the strips beneath four tiles —
    // one with no change to name, one naming an entry, one a departure, one the highest of
    // two layers. `tests/frame_range.rs` asserts the strings; this is what they look like,
    // in the face ADR-0122 embeds, at the size the sizing chose.
    let dir = tempdir(line!());
    let rect = |id: &str, x: i64, start: i64, end: i64, fill: &str| {
        format!(
            r##"{{"id":"{id}","type":"rect","start":{start},"end":{end},"x":{x},"y":60,
                "origin":"top-left","width":40,"height":80,"fill":"{fill}"}}"##
        )
    };
    let project = write_project(
        &dir,
        "labelled.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":200,"height":200}},"fps":25,"background":"#000000",
                "duration":4000,"tracks":[
                  {{"name":"bg","layer":0,"elements":[{{"id":"bg","type":"rect","start":0,
                    "end":4000,"x":0,"y":0,"origin":"top-left","width":200,"height":200,
                    "fill":"#203040"}}]}},
                  {{"name":"a","layer":5,"elements":[{}]}},
                  {{"name":"b","layer":5,"elements":[{}]}},
                  {{"name":"hi","layer":9,"elements":[{}]}}
                ]}}"##,
            rect("card-a", 20, 1000, 4000, "#E0A030"),
            rect("card-b", 80, 1000, 2000, "#30A0E0"),
            rect("title-hi", 140, 3000, 4000, "#E03050"),
        )),
    );
    against_golden("sheet-labelled", &sheet_of(&project, 0, 4000));
}

#[test]
fn an_elided_sheet_draws_every_label_without_its_id_the_same_way_it_did_before() {
    // Thirty portrait states serve at 141 px, where labels carrying a 17-character id no longer fit
    // at the 8 px floor: every strip carries its numeric core alone (ADR-0098 §5).
    let dir = tempdir(line!());
    let elements: Vec<String> = (0..30)
        .map(|i| {
            format!(
                r##"{{"id":"a-long-card-id-{i:02}","type":"rect","start":{},"end":{},"x":0,
                    "y":0,"origin":"top-left","width":1080,"height":1920,
                    "fill":"#{:02X}{:02X}60"}}"##,
                i * 200,
                (i + 1) * 200,
                40 + i * 6,
                220 - i * 6
            )
        })
        .collect();
    let project = write_project(
        &dir,
        "elided.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":1080,"height":1920}},"fps":25,"background":"#000000",
                "tracks":[{{"name":"cards","layer":0,"elements":[{}]}}]}}"##,
            elements.join(",")
        )),
    );
    against_golden("sheet-elided", &sheet_of(&project, 0, 6000));
}

// ---------------------------------------------------------------------------
// The sampling rule (ADR-0132, #500)
// ---------------------------------------------------------------------------
//
// The goldens above cannot see which filter reads an image: under the change from
// bilinear to ADR-0132's rule, five of them moved by at most 0.14 mean delta and all of
// them passed. So these two are built for the one input where the filter shows — a hard
// alpha edge filling the frame — and are held to a tighter pair of thresholds, set from
// what was measured when they were committed: the rule against itself across platforms
// is #34's ~0.004, and the same frames painted with plain bilinear (no mipmaps, the
// sampling ADR-0132 replaced) land at the deltas quoted on each test.

/// The tighter pair for the two sampling goldens. The SSIM floor is the file's own; the
/// mean delta is what separates the filters here, and sits well under what bilinear
/// measured on either frame.
const SAMPLING_MEAN_DELTA: f64 = 0.25;

/// [`against_golden`], with the sampling goldens' tighter mean delta.
#[track_caller]
fn against_sampling_golden(name: &str, ours: &image::RgbaImage) {
    against_golden(name, ours);
    if updating() {
        return;
    }
    let golden = image::open(golden_dir().join(format!("{name}.png")))
        .expect("the golden just compared")
        .to_rgba8();
    let delta = mean_delta(&golden, ours);
    assert!(
        delta <= SAMPLING_MEAN_DELTA,
        "{name}: mean channel delta {delta:.4} is over the sampling goldens' ceiling of \
         {SAMPLING_MEAN_DELTA} — the filter that reads an image has changed (ADR-0132). If \
         that is intended, look at both pictures, regenerate with UPDATE_GOLDEN=1, and \
         put the diff in the commit."
    );
}

/// A cut-out drawn the way #500's character parts are: a light fill inside a dark
/// outline, on transparency — the edge a magnifying filter stair-steps and a cubic can
/// ring on. Antialiased by 8×8 supersampling, so the source is the honest picture of the
/// shape and not itself aliased.
fn cut_out(size: u32, paint: impl Fn(f64, f64) -> Option<[u8; 3]>) -> image::RgbaImage {
    const SUB: u32 = 8;
    image::RgbaImage::from_fn(size, size, |px, py| {
        let (mut sum, mut covered) = ([0u32; 3], 0u32);
        for sy in 0..SUB {
            for sx in 0..SUB {
                let x = px as f64 + (sx as f64 + 0.5) / SUB as f64;
                let y = py as f64 + (sy as f64 + 0.5) / SUB as f64;
                if let Some(rgb) = paint(x, y) {
                    covered += 1;
                    for c in 0..3 {
                        sum[c] += rgb[c] as u32;
                    }
                }
            }
        }
        if covered == 0 {
            return image::Rgba([0, 0, 0, 0]);
        }
        let alpha = (covered * 255 + SUB * SUB / 2) / (SUB * SUB);
        image::Rgba([
            (sum[0] / covered) as u8,
            (sum[1] / covered) as u8,
            (sum[2] / covered) as u8,
            alpha as u8,
        ])
    })
}

const OUTLINE: [u8; 3] = [0x1E, 0x34, 0x4C];
const FILL: [u8; 3] = [0xFB, 0xF3, 0xE3];

/// One image element filling a square frame, its source written beside the project.
fn sampled(dir: &Path, name: &str, frame: u32, source: &image::RgbaImage, scale: f64) -> PathBuf {
    let png = dir.join(format!("{name}.png"));
    source.save(&png).expect("write the source");
    let size = source.width();
    write_project(
        dir,
        &format!("{name}.montagent.json"),
        &canonical(&format!(
            r##"{{"frame":{{"width":{frame},"height":{frame}}},"fps":25,"background":"#F2A65A",
                "tracks":[{{"name":"image","layer":0,"elements":[
                  {{"id":"part","type":"image","start":0,"end":1000,"x":{half},"y":{half},
                    "origin":"center","width":{size},"height":{size},
                    "scale":[{scale},{scale}],"source":"{source}","fit":"cover"}}
                ]}}]}}"##,
            half = frame / 2,
            source = common::with_forward_slashes(&png.display().to_string()),
        )),
    )
}

#[test]
fn a_cut_out_enlarged_reads_through_the_cubic() {
    // ×2.3, the close-up #500 measured. A 96 px part: a filled head with a 3 px outline,
    // an eye, and a thin diagonal stroke — dark on light, the contrast where Catmull-Rom's
    // overshoot would show as a halo if it were going to. Looked at when committed: no
    // halo, no stair-steps. Painted with plain bilinear this frame measured a mean delta
    // of 0.79 against the golden (SSIM 0.9965), and Mitchell 0.73.
    let dir = tempdir(line!());
    let source = cut_out(96, |x, y| {
        let r = ((x - 48.0).powi(2) + (y - 50.0).powi(2)).sqrt();
        let eye = ((x - 60.0) / 7.0).powi(2) + ((y - 42.0) / 9.0).powi(2);
        let stroke = ((x - y) / std::f64::consts::SQRT_2).abs();
        if eye <= 1.0 || (stroke <= 1.0 && r < 41.0 && x < 40.0) {
            Some(OUTLINE)
        } else if r <= 38.0 {
            Some(FILL)
        } else if r <= 41.0 {
            Some(OUTLINE)
        } else {
            None
        }
    });
    let project = sampled(&dir, "sampling-enlarged", 240, &source, 2.3);
    against_sampling_golden(
        "sampling-enlarged",
        &rendered(&project, 500, /* full */ true),
    );
}

#[test]
fn a_fine_cut_out_shrunk_reads_through_the_mipmaps() {
    // ×0.3, the wide shot's toast. Rings and hatching a few source pixels wide, which a
    // filter reading four source pixels out of eleven either keeps or drops by where the
    // grid falls — the aliasing mipmaps remove and a cubic does not. Painted with plain
    // bilinear this frame measured a mean delta of 8.30 against the golden, and Catmull-Rom — a cubic, which ignores
    // mipmaps — 10.04.
    let dir = tempdir(line!());
    let source = cut_out(400, |x, y| {
        let r = ((x - 200.0).powi(2) + (y - 200.0).powi(2)).sqrt();
        if r > 190.0 {
            return None;
        }
        let ring = (r % 12.0) < 3.0;
        let hatch = ((x + y) % 10.0) < 2.0 && x < 200.0;
        Some(if ring || hatch { OUTLINE } else { FILL })
    });
    let project = sampled(&dir, "sampling-shrunk", 160, &source, 0.3);
    against_sampling_golden("sampling-shrunk", &rendered(&project, 500, /* full */ true));
}

#[test]
fn flat_text_renders_as_it_did_before_text_could_bend_along_a_path() {
    // ADR-0161 changed the text painter: a text carrying `path` draws through its curve, and
    // every other text must draw exactly as before. This golden was written by `main` before
    // that change (#765), over what the painter's flat route carries: Arabic joined pieces, a
    // mixed-direction line, letter spacing, a run's stroke, and a fading letter stagger
    // caught mid-flight under a shadow.
    let dir = tempdir(line!());
    let fixtures = fixture_dir().join("..");
    let font = |relative: &str| {
        common::with_forward_slashes(&fixtures.join(relative).display().to_string())
    };
    let (latin, naskh) = (
        font("benchmark/spy-trailer/fonts/Oswald-SemiBold.ttf"),
        font("letter-spacing/fonts/NotoNaskhArabic-Regular.ttf"),
    );
    let project = write_project(
        &dir,
        "flat-text.montagent.json",
        &canonical(&format!(
            r##"{{"frame":{{"width":640,"height":360}},"fps":25,"background":"#101418",
                "fonts":{{"title":[{{"file":"{latin}"}},{{"file":"{naskh}"}}]}},
                "tracks":[{{"name":"text","layer":0,"elements":[
                  {{"id":"arabic","type":"text","start":0,"end":1000,"x":320,"y":60,
                    "origin":"center","width":600,"height":80,"font":"title","size":44,
                    "color":"#9FE3FF","align":"center",
                    "runs":[{{"text":"بسم الله الرحمن الرحيم"}}],"caption":false}},
                  {{"id":"mixed","type":"text","start":0,"end":1000,"x":320,"y":140,
                    "origin":"center","width":600,"height":70,"font":"title","size":36,
                    "color":"#F5F0E6","align":"end","letter_spacing":60,
                    "runs":[{{"text":"عام 2026 سعيد "}},
                            {{"text":"NEW","stroke":"#FF5A36","stroke_width":3}}],
                    "caption":false}},
                  {{"id":"stagger","type":"text","start":0,"end":1000,"x":320,"y":260,
                    "origin":"center","width":600,"height":110,"font":"title","size":72,
                    "color":"#F2E6C9","align":"center","runs":[{{"text":"DROP IN fi"}}],
                    "units":{{"by":"letter","every":60,"origin":"bottom-center",
                      "y":[{{"t":0,"v":-60}},{{"t":500,"v":0,"ease":"ease-out"}}],
                      "rotation":[{{"t":0,"v":-30.0}},{{"t":500,"v":0.0,"ease":"ease-out"}}],
                      "opacity":[{{"t":0,"v":0.0}},{{"t":400,"v":1.0,"ease":"linear"}}]}},
                    "effects":[{{"name":"shadow","dx":3,"dy":4,"radius":5,"color":"#000000",
                                 "opacity":0.7}}],
                    "caption":false}}
                ]}}]}}"##
        )),
    );
    against_golden("flat-text", &rendered(&project, 280, /* full */ true));
}
