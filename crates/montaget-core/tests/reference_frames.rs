//! **The falsification.** `frame` against two frames of the published video (#213).
//!
//! # Reference, not golden — the distinction the whole file turns on
//!
//! A *golden* frame is one Montaget rendered and committed. It is **self-confirming**: it
//! catches a regression and can never falsify the format, because the thing under test
//! produced it. `tests/golden_frames.rs` holds those, and says so in its name.
//!
//! A *reference* frame was extracted from `reference/en-halloween-decorating.mp4`, the
//! already-published short — made by a Python/FFmpeg/ASS pipeline that knows nothing about
//! Montaget. Nothing Montaget does can change it. It is the only thing in this repository
//! capable of saying *this is wrong*, and spec #168 calls the comparison against it *"the
//! only test in this spec capable of falsifying the format itself"*. #168 also warns that
//! blurring the two words ships a suite that looks thorough and tests nothing, which is
//! why they are two files with two names.
//!
//! # It falsified two things on its first run, and both are recorded rather than tuned away
//!
//! **The reference frames' instants were not what their names say.** Neither PNG carried
//! one. `frame-05-at-11s.png` is frame **350** of the MP4 — 14.0 s on the project's one
//! timeline — because the name counts 11 s from *item 05's* start at 3 018 ms, not from
//! the beginning. Taking the name at face value compares this render against a frame three
//! seconds of Ken Burns away, and the 0.60 that comes back reads like a rasterizer
//! defect.
//! Both instants are re-derived by `ci/reference_frame_instants.py`.
//!
//! **The fixture's Ken Burns pivots about the wrong point.** The photo elements declare
//! `origin: "top-left"` at `(0, 0)`; the published move is a **centre** pivot, which
//! `docs/research/sample-project-migration/README.md` D3 had already measured — *"the move
//! is a centre-pivot zoom (centre beats top at every sample)"* — and which the migration
//! then did not write into the file. At 400 ms the two differ by a fifth of a pixel and
//! nothing can tell them apart; at 14.0 s, with the ramp at 1.0588, they differ by 32 px
//! horizontally and 56 px vertically, and the photograph plainly does not match.
//! `ci/reference_frame_instants.py` re-derives the scale and offset from the pictures and
//! shows the centre pivot fitting where the declared one does not.
//!
//! That is a defect in the **fixture**, not in this build: the renderer paints the pivot
//! the document names, and #212's tests fix that reading against all nine keywords. It is
//! quantified, masked out of the gate at the frame where it bites, reported as a number on
//! every run, and raised as [#276](https://github.com/MBehtemam/Montaget/issues/276) —
//! the same treatment #186 already prescribes for the typeface. What it is not is absorbed into a looser threshold: a gate slack
//! enough to pass a 56 px displacement is slack enough to pass anything, which is the
//! *"completed, looked plausible, was wrong"* failure ADR-0010 records this project having
//! had twice.
//!
//! # The typeface is not the same either
//!
//! The reference was typeset in **SF Pro Rounded**;
//! [#143](https://github.com/MBehtemam/Montaget/issues/143) re-vendored the fixture to
//! **Open Runde** because SF Pro Rounded was never in the repository and cannot legally be
//! redistributed (ADR-0057, which also records that *"no font claims formal metric
//! compatibility"*). So every text-bearing region differs for a reason that is not a
//! defect, and [#186](https://github.com/MBehtemam/Montaget/issues/186) — still open — is
//! where that gets quantified. Text is masked out of every gate and measured beside it.
//!
//! # So each reference states its own gated region
//!
//! Between the two frames the whole picture is gated, and each half is gated where it can
//! be believed:
//!
//! - **`frame-intro.png`, 400 ms** — the photograph and its Ken Burns at the start of the
//!   ramp, the cream ground, the header panels, the drawn flag and the circular badge:
//!   everything but the text.
//! - **`frame-05-at-11s.png`, 14 000 ms** — the navy sentence card, the cream ground, both
//!   header panels, the flag and the badge, with the photograph behind them masked for the
//!   pivot divergence above. The panels are cut back *in* over the photograph they are
//!   drawn on, which is why [`Scope`] is layered.
//!
//! **One thing #213 asked to be gated is not, and saying so is the point of this
//! paragraph.** The ticket's masked-in list names *"the Ken Burns move"*. The move is
//! gated only at 400 ms, where the ramp is 1.0021 and the displacement it would catch is
//! a single pixel — so in practice the *move* is gated nowhere, and only the
//! photograph's framing and content are. That is a direct consequence of #276: the
//! fixture's pivot is wrong, so the one frame far enough up the ramp to test the move is
//! the one frame where the photograph cannot be compared at all. #276's "done when"
//! carries the obligation to unmask it. Until then
//! [`the_fixtures_ken_burns_pivot_divergence_is_measured_and_reported`] measures the
//! photograph at both instants and asserts the divergence grows with the ramp, which
//! identifies the defect but does not gate the move.
//!
//! Run with `--nocapture` to see every number this file measured.

use std::path::{Path, PathBuf};

mod common;
use common::compare::{Plane, RADIUS, Region, Scope, rendered, resized, ssim};
use common::{fixture_dir, fixture_project};

// ---------------------------------------------------------------------------
// The two reference frames
// ---------------------------------------------------------------------------

/// The project's true frame width, which is what `frame` rasterizes (ADR-0021).
const FRAME_WIDTH: i64 = 1080;

/// The committed reference frames are 270×480 — a quarter of the project's frame, which is
/// the size they were extracted at. The comparison therefore happens at 270×480 and the
/// render comes down to meet it.
const REFERENCE_WIDTH: i64 = 270;

/// How far outside an excluded box the mask reaches, in reference pixels.
///
/// **At least [`RADIUS`], and this is the whole reason the number is 8 rather than 3.**
/// [`ssim`] admits or rejects a *window centre*, and every window reaches five pixels in
/// each direction — so a centre admitted three pixels outside a text box still averages
/// that box's glyphs into its own mean and covariance, and the exclusion leaks straight
/// back into the number it exists to keep out. `Region::scaled` asserts the bound rather
/// than trusting this comment.
///
/// The remaining three are the ink's own slop: antialiasing, the H.264 decoder's ringing
/// around high-contrast edges, and the two fonts' differing overshoot all put a
/// difference just outside the letter. Eight at quarter scale is 32 of the project's own
/// pixels, which is wide — the cost is paid in coverage, which every run prints.
const MARGIN: i64 = RADIUS as i64 + 3;

/// One frame of the published video, and everything needed to compare against it.
struct Reference {
    file: &'static str,
    /// The instant on the project's one timeline, re-derived by
    /// `ci/reference_frame_instants.py` — **not** read off the file name.
    at: i64,
    provenance: &'static str,
    /// The text on screen, as the rectangle its ink can occupy: the element's **declared
    /// `width`** horizontally, and its **computed block** vertically — `size ×
    /// line_height × lines` centred on `y` through `origin`, which is ADR-0028's
    /// arithmetic and carries no font term.
    ///
    /// The horizontal half is the declared box rather than the measured advance, because
    /// the advance is exactly the quantity the substitution changes: a mask sized to Open
    /// Runde's line would leave SF Pro Rounded's wider one half inside the gate, and the
    /// gate would then be measuring the typeface. Vertically the two agree by
    /// construction, so the block is tight and the navy card's own top and bottom bands
    /// stay gated.
    text: &'static [(&'static str, Region)],
    /// Regions excluded for a reason of this frame's own, each carrying the reason.
    excluded: &'static [(&'static str, Region)],
    /// Regions put back in over an exclusion — the header panels, which are drawn *on* the
    /// photograph and are Montaget's own geometry rather than the photograph's.
    readmitted: &'static [(&'static str, Region)],
    /// What the gated comparison must clear.
    gate: f64,
    /// The least of the frame the gate may cover before it stops being a gate.
    least_coverage: f64,
}

const REFERENCES: [Reference; 2] = [
    Reference {
        file: "frame-intro.png",
        at: 400,
        provenance: "frame 10 of the published MP4 (ci/reference_frame_instants.py: mean \
                     absolute difference 0.000 here, 0.31 at the frame either side)",
        text: &[
            // intro-title: x 540, y 1470, origin center, width 984; 2 lines × 58 × 1.1.
            (
                "intro-title",
                Region {
                    x: 48,
                    y: 1406,
                    width: 984,
                    height: 128,
                },
            ),
            CHIP_TEXT,
            HANDLE_TEXT,
        ],
        excluded: &[],
        readmitted: &[],
        // **Measured, not chosen: 0.9362.** The gate sits one notch under it. 1.0 is not
        // reachable and a gate near it would be measuring the codec rather than the
        // render — the reference side is a whole H.264 frame of cobweb at 1.28 Mb/s,
        // resampled with a different filter from the one that encoded it.
        //
        // The headroom is 0.011, which is thin only if the render varies across targets.
        // It does not meaningfully: #34's oracle measured `skia-safe` differing by at
        // most 2/255 on a handful of pixels between the six tier-1 targets, and the
        // resample and the SSIM here are both pure deterministic Rust. Every run prints
        // its score, so tightening this is a matter of reading a log.
        gate: 0.925,
        least_coverage: 0.8,
    },
    Reference {
        file: "frame-05-at-11s.png",
        at: 14000,
        provenance: "frame 350 of the published MP4 — 11 s into item 05, which starts at \
                     3 018 ms, not 11 s into the video (ci/reference_frame_instants.py)",
        text: &[
            // word-05: x 540, y 1373, origin center, width 984; 1 line × 88 × 1.1 = 96.8.
            (
                "word-05",
                Region {
                    x: 48,
                    y: 1324,
                    width: 984,
                    height: 98,
                },
            ),
            // sentence-05: x 540, y 1537, origin center, width 984; 1 line × 55 × 1.1.
            (
                "sentence-05",
                Region {
                    x: 48,
                    y: 1506,
                    width: 984,
                    height: 61,
                },
            ),
            CHIP_TEXT,
            HANDLE_TEXT,
        ],
        excluded: &[(
            // The photograph, which is `photo-05`'s own `clip`: [0, 0, 1080, 1300].
            // Excluded for the Ken Burns pivot divergence this file's header documents —
            // at 14 000 ms the ramp is at 1.0586, and a top-left pivot puts the photograph
            // 32 px right and 56 px down of where the published move puts it.
            "the photograph (the fixture's Ken Burns pivot — #276, measured below)",
            Region {
                x: 0,
                y: 0,
                width: 1080,
                height: 1300,
            },
        )],
        readmitted: &[
            // Both header panels are Montaget's own rectangles drawn *over* the
            // photograph, so excluding the photograph must not take them with it. Their
            // own text goes back out afterwards, which is what the layering is for.
            (
                "chip-panel",
                Region {
                    x: 48,
                    y: 88,
                    width: 372,
                    height: 84,
                },
            ),
            (
                "handle-panel",
                Region {
                    x: 438,
                    y: 88,
                    width: 594,
                    height: 84,
                },
            ),
        ],
        // **Measured at 0.9515.** The region is the flag's three rectangles, the badge,
        // two cream panels, the navy card's own top and bottom bands and the cream
        // ground, with the photograph's detail masked away.
        gate: 0.94,
        least_coverage: 0.18,
    },
];

/// `chip-text`: x 182, y 130, origin center-left, width 238; 1 line × 52 × 1.1 = 57.2.
const CHIP_TEXT: (&str, Region) = (
    "chip-text",
    Region {
        x: 182,
        y: 101,
        width: 238,
        height: 58,
    },
);

/// `handle-text`: x 560, y 130, origin center-left, width 472; 1 line × 34 × 1.1 = 37.4.
const HANDLE_TEXT: (&str, Region) = (
    "handle-text",
    Region {
        x: 560,
        y: 111,
        width: 472,
        height: 38,
    },
);

/// The least a deliberate break must cost, as a drop from the unbroken score.
///
/// Stated as a *drop* rather than as a distance below the gate, because that is the
/// property being demonstrated: a comparison is sensitive when breaking the picture
/// changes the number, and a test that only asserted "below the threshold" would pass
/// with the break scoring 0.9599 against a gate of 0.96 — coincidence dressed as
/// discrimination. Both are asserted: the break must cost this much *and* land under the
/// gate.
const MIN_DROP: f64 = 0.02;

impl Reference {
    /// The gated region: everything, less the photograph where this frame excludes it,
    /// plus the panels drawn over it, less the text — **in that order**, because the last
    /// layer to name a pixel is the one that decides.
    fn gated(&self) -> Scope {
        Scope::whole()
            .less(scaled(self.excluded))
            .plus(scaled(self.readmitted))
            .less(scaled(self.text))
    }

    /// The text regions alone — measured, never gated.
    fn text_only(&self) -> Scope {
        Scope::inside(scaled(self.text))
    }
}

fn scaled(boxes: &[(&str, Region)]) -> Vec<Region> {
    boxes
        .iter()
        .map(|(_, area)| area.scaled(FRAME_WIDTH, REFERENCE_WIDTH, MARGIN))
        .collect()
}

fn named(boxes: &[(&str, Region)]) -> String {
    boxes
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

#[track_caller]
fn published(file: &str) -> image::RgbaImage {
    let path = fixture_dir().join("reference").join(file);
    image::open(&path)
        .unwrap_or_else(|e| panic!("reading the committed reference {}: {e}", path.display()))
        .to_rgba8()
}

/// Both planes of one comparison, at the reference's size.
///
/// Rasterized at the project's true pixels and resampled down, never rendered at the
/// reference's size: ADR-0021 is explicit that `frame` is never proxy-scaled, so the
/// picture under test has to be the one the verb actually produces.
fn planes(reference: &Reference, project: &Path) -> (Plane, Plane) {
    let theirs = published(reference.file);
    let ours = resized(
        &rendered(project, reference.at, /* full */ true),
        theirs.width(),
        theirs.height(),
    );
    (Plane::of(&theirs), Plane::of(&ours))
}

// ---------------------------------------------------------------------------
// The gating comparison
// ---------------------------------------------------------------------------

#[test]
fn the_render_matches_the_published_video_over_each_frames_gated_region() {
    // The one test in this repository that can say the format is wrong. Everything it
    // covers was drawn by Montaget from the document and by the old pipeline from its own
    // inputs — nothing in the comparison came from the thing under test.
    for reference in &REFERENCES {
        let (theirs, ours) = planes(reference, &fixture_project());
        let mask = reference.gated();
        let coverage = mask.coverage(theirs.width, theirs.height);
        let score = ssim(&theirs, &ours, &mask);

        println!(
            "GATING  {} at {} ms — SSIM {score:.4} over {:.1}% of the frame (gate {}).\n\
             \x20        instant: {}\n\
             \x20        masked out: text [{}]{}",
            reference.file,
            reference.at,
            coverage * 100.0,
            reference.gate,
            reference.provenance,
            named(reference.text),
            if reference.excluded.is_empty() {
                String::new()
            } else {
                format!(
                    "; {}; with [{}] cut back in",
                    named(reference.excluded),
                    named(reference.readmitted)
                )
            },
        );
        assert!(
            coverage >= reference.least_coverage,
            "{}: a gate over {:.1}% of the frame is not the gate this reference claims to \
             be — the masks have swallowed the picture",
            reference.file,
            coverage * 100.0
        );
        assert!(
            score >= reference.gate,
            "{} at {} ms: SSIM {score:.4} is below the gate {}. This is the comparison \
             that can falsify the format — look at the two pictures before touching the \
             number.",
            reference.file,
            reference.at,
            reference.gate
        );
    }
}

#[test]
fn the_text_regions_are_measured_and_reported_and_gate_nothing() {
    // #186, quantified at the two instants the suite has references for. There is
    // deliberately **no threshold**: the fixture renders in Open Runde and the published
    // video was typeset in SF Pro Rounded (#143, under ADR-0057 — "no font claims formal
    // metric compatibility"), so a number here would be a measurement of the substitution
    // and asserting on one would freeze the substitution into the suite.
    //
    // What *is* asserted is that the measurement was taken, and that it is worse than the
    // gated region's: if the text ever matched as well as the rest of the frame, the mask
    // would be unnecessary and this file would be overstating its case.
    for reference in &REFERENCES {
        let (theirs, ours) = planes(reference, &fixture_project());
        let text = reference.text_only();
        let in_text = ssim(&theirs, &ours, &text);
        let gated = ssim(&theirs, &ours, &reference.gated());

        println!(
            "NON-GATING  {} at {} ms — text SSIM {in_text:.4} over {:.1}% of the frame, \
             against {gated:.4} in the gated region.\n\
             \x20           The published video is typeset in SF Pro Rounded; this render \
             is in Open Runde — #143 re-vendored it because SF Pro Rounded is not \
             redistributable (ADR-0057). #186 owns the divergence.\n\
             \x20           Elements: {}",
            reference.file,
            reference.at,
            text.coverage(theirs.width, theirs.height) * 100.0,
            named(reference.text),
        );
        assert!(
            in_text < gated,
            "{}: the text scored {in_text:.4} against {gated:.4} in the gated region. If \
             the typeface substitution has stopped costing anything, #186 can close and \
             this mask should go.",
            reference.file
        );
    }
}

#[test]
fn the_fixtures_ken_burns_pivot_divergence_is_measured_and_reported() {
    // The other non-gating measurement, and the one this suite found rather than
    // inherited. The photo elements declare `origin: "top-left"`; the published move is a
    // centre pivot, which `docs/research/sample-project-migration/README.md` D3 measured
    // and the migration did not write into the file.
    //
    // Reported rather than held to a threshold, for the same reason as the typeface: the
    // number measures a defect in the fixture, and freezing it would make *fixing* the
    // fixture a test failure. What is asserted is the finding's shape — that the
    // divergence grows with the ramp — because that is what identifies it as a pivot
    // rather than as a renderer that cannot place an image at all.
    let photograph = Scope::inside(vec![
        Region {
            x: 0,
            y: 0,
            width: 1080,
            height: 1300,
        }
        .scaled(FRAME_WIDTH, REFERENCE_WIDTH, -MARGIN),
    ])
    .less(scaled(REFERENCES[1].readmitted));

    let mut scores = Vec::new();
    for reference in &REFERENCES {
        let (theirs, ours) = planes(reference, &fixture_project());
        let score = ssim(&theirs, &ours, &photograph);
        // The ramp each element declares: 1.0 at its first keyframe, 1.08 fifteen seconds
        // later. `photo-05-intro` starts at 0; `photo-05` restarts at 3 018.
        let ramp = match reference.at {
            400 => 1.0 + 0.08 * 400.0 / 15000.0,
            at => 1.0 + 0.08 * (at as f64 - 3018.0) / 15000.0,
        };
        println!(
            "NON-GATING  {} at {} ms — photograph SSIM {score:.4}, Ken Burns ramp at \
             {ramp:.4}.\n\
             \x20           The fixture declares `origin: \"top-left\"` at (0, 0); the \
             published move pivots about the centre, so the two part company as the ramp \
             grows — {:.0} px horizontally here.",
            reference.file,
            reference.at,
            (ramp - 1.0) * 540.0,
        );
        scores.push((reference.at, ramp, score));
    }

    let (_, early_ramp, early) = scores[0];
    let (_, late_ramp, late) = scores[1];
    assert!(
        early_ramp < late_ramp,
        "the two instants order by their ramp"
    );
    assert!(
        early > late,
        "the photograph matched better at the *larger* zoom ({late:.4} at ramp \
         {late_ramp:.4}) than at the smaller ({early:.4} at ramp {early_ramp:.4}). That is \
         not a pivot divergence, and the reading recorded in this file is wrong."
    );
}

// ---------------------------------------------------------------------------
// The threshold is sensitive
// ---------------------------------------------------------------------------

#[test]
fn a_deliberately_broken_render_fails_the_gated_comparison() {
    // The claim a passing threshold cannot make on its own. Each break is inside the
    // gated region of the reference it is checked against, and each is the size of defect
    // this project has actually shipped: a card in the wrong place, a panel the wrong
    // colour, the wrong photograph, an image off its mark.
    let original = std::fs::read_to_string(fixture_project()).expect("the fixture");

    let breaks: [(usize, &str, &str, &str); 4] = [
        (
            1,
            "the navy sentence card sits 40 px high",
            r#""id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"x":48,"y":1453"#,
            r#""id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"x":48,"y":1413"#,
        ),
        (
            1,
            "the cream chip panel turns navy",
            r##""id":"chip-panel","type":"rect","group":"header","start":0,"end":65216,"x":48,"y":88,"origin":"top-left","width":372,"height":84,"fill":"#FBF3E3""##,
            r##""id":"chip-panel","type":"rect","group":"header","start":0,"end":65216,"x":48,"y":88,"origin":"top-left","width":372,"height":84,"fill":"#1E344C""##,
        ),
        (
            0,
            "the intro runs over the wrong photograph",
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/05.png""#,
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/06.png""#,
        ),
        (
            0,
            "the photograph sits 40 px down",
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/05.png","x":0,"y":0"#,
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/05.png","x":0,"y":40"#,
        ),
    ];

    // The unbroken score for each reference, so the cost of a break is a measured
    // difference rather than a distance from a constant.
    let unbroken: Vec<f64> = REFERENCES
        .iter()
        .map(|reference| {
            let (theirs, ours) = planes(reference, &fixture_project());
            ssim(&theirs, &ours, &reference.gated())
        })
        .collect();

    for (which, what, from, to) in breaks {
        let reference = &REFERENCES[which];
        assert!(
            original.contains(from),
            "the break `{what}` no longer matches the fixture: {from}"
        );
        // Written **beside the fixture**, because a project's relative paths resolve
        // against its own directory (ADR-0053): moving it to a scratch directory would
        // change the photograph, the badge and the font too, and the comparison would then
        // be measuring the move rather than the break.
        let broken = Scratch::beside_the_fixture(&original.replace(from, to));

        let theirs = published(reference.file);
        let ours = resized(
            &rendered(broken.path(), reference.at, /* full */ true),
            theirs.width(),
            theirs.height(),
        );
        let score = ssim(&Plane::of(&theirs), &Plane::of(&ours), &reference.gated());
        let drop = unbroken[which] - score;
        println!(
            "BREAK   {what} — SSIM {score:.4} against {}, down {drop:.4} from {:.4} \
             (gate {})",
            reference.file, unbroken[which], reference.gate
        );
        assert!(
            drop >= MIN_DROP,
            "`{what}` cost only {drop:.4} ({:.4} to {score:.4}). A comparison a break \
             does not move is a comparison that tests nothing (ADR-0010).",
            unbroken[which]
        );
        assert!(
            score < reference.gate,
            "`{what}` scored {score:.4} against {}, which its gate {} still passes.",
            reference.file,
            reference.gate
        );
    }
}

/// A project file written into the fixture's own directory and removed again, even when
/// the assertion between the two panics.
struct Scratch(PathBuf);

impl Scratch {
    fn beside_the_fixture(body: &str) -> Scratch {
        // One fixed name rather than a unique one: the breaks above run in sequence inside
        // a single test, and a second test writing here at the same time would be a second
        // test that also has to know the fixture directory is shared state. The name is
        // not a project any other test reads and nothing globs this directory for
        // `*.montaget.json`, so the only cost of the copy that a hard abort leaves behind
        // is a line of `git status` — and the next run overwrites it.
        let path = fixture_dir().join("broken-under-test.montaget.json");
        std::fs::write(&path, body).expect("write the broken project beside the fixture");
        Scratch(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
