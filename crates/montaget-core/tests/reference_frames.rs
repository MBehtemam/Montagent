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
//! **The fixture's Ken Burns pivoted about the wrong point** — [#276], now fixed in the
//! fixture. The photo elements declared `origin: "top-left"` at `(0, 0)`; the published
//! move is a **centre** pivot, which `docs/research/sample-project-migration/README.md` D3
//! had already measured — *"the move is a centre-pivot zoom (centre beats top at every
//! sample)"* — and which the migration then did not write into the file. At 400 ms the two
//! spellings are a pixel apart and nothing can tell them apart; at 14.0 s, with the ramp at
//! 1.0586, they are 32 px apart horizontally and 56 px vertically, and the photograph
//! plainly did not match. `ci/reference_frame_instants.py` re-derives the scale and offset
//! from the pictures and shows the centre pivot fitting where the declared one did not.
//!
//! That was a defect in the **fixture**, not in this build: the renderer paints the pivot
//! the document names, and #212's tests fix that reading against all nine keywords. So the
//! repair is in `migrate.py`, which regenerates the committed file, and it is checked from
//! this side by [`the_corrected_pivot_beats_the_one_the_migration_first_wrote`] — which
//! renders both spellings against the published frame rather than trusting either.
//!
//! **What it was not, at any point, is absorbed into a looser threshold.** While it stood
//! it was quantified, masked out of the gate at the frame where it bit, and reported as a
//! number on every run — the same treatment #186 still prescribes for the typeface — and
//! the fix did not loosen anything either: the photograph became its own gated half rather
//! than being folded into one whole-frame number that a 40 px displacement could pass. A
//! gate slack enough to admit a 56 px displacement is slack enough to admit anything, which
//! is the *"completed, looked plausible, was wrong"* failure ADR-0010 records this project
//! having had twice.
//!
//! **The correction did not close the divergence, and the remainder is measured rather than
//! assumed away.** With the centre pivot and the declared ramp both in place, the
//! photograph still sits about 2 px from the published one at 400 ms and 8 px at 14 000 ms.
//! [`what_the_corrected_pivot_does_not_explain_is_measured_and_reported`] carries the
//! numbers and what is known about them; nothing owns it yet.
//!
//! [#276]: https://github.com/MBehtemam/Montaget/issues/276
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
//! # Every frame is gated over two regions, never one
//!
//! Each reference is compared twice, against its own threshold each time ([`Half`]):
//!
//! - **[`Half::Drawn`]** — everything Montaget draws from the document's own geometry: the
//!   cream ground, the navy sentence card, the flag's three rectangles, the circular badge
//!   and both header panels, less the text. The panels are cut back *in* over the
//!   photograph they are painted on, which is why [`Scope`] is layered.
//! - **[`Half::Photograph`]** — the photograph, and with it the Ken Burns move, less those
//!   same two panels.
//!
//! **One gate would be one number over two pictures that diverge for different reasons.**
//! The photograph is 53% of the frame, arrives through H.264 at 1.28 Mb/s, and matches
//! worst; the drawn geometry is flat brand colour and matches at 0.9874. A single
//! threshold would have to clear the worse of them, and at that height a navy card 40 px
//! out of place — 0.034 on a combined number — would pass. Split, the same break costs
//! 0.131 against a gate 0.012 below the unbroken score, and every gate in this file is set
//! one notch under a measurement printed on every run.
//!
//! **This is what #213 asked for and could not have.** Its masked-in list names *"the Ken
//! Burns move"*, and until #276 the move was gated nowhere: at 400 ms the ramp is 1.0021
//! and there is nothing to catch, and at 14 000 ms the photograph had to be masked out
//! entirely because the fixture's pivot put it 32 × 56 px away. With the pivot corrected,
//! the 14 000 ms frame gates the photograph at a ramp of 1.0586.
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
    /// [`Half::Drawn`]: everything Montaget draws from the document's own geometry.
    drawn: Gated,
    /// [`Half::Photograph`]: the photograph, and so the Ken Burns move that carries it.
    ///
    /// **Its own number, and that is the point.** Until #276 the photograph was masked out
    /// of the later frame entirely, because the fixture's pivot put it 32 × 56 px from
    /// where the published move puts it, and a single gate loose enough to admit that was
    /// loose enough to admit anything. Folding it back into one whole-frame number instead
    /// would have had the same effect by a quieter route: the photograph is 53% of the
    /// frame and matches worst, so one combined gate would have to sit near 0.69 — and a
    /// navy card 40 px out of place, which moves that number by 0.034, would sail through
    /// it. Two regions, two numbers, each set where its own picture can be believed.
    photograph: Gated,
}

/// One gated region's two numbers.
struct Gated {
    /// What the comparison over that region must clear.
    gate: f64,
    /// The least of the frame it may cover before it stops being a gate. A threshold over
    /// a region the masks have eaten passes for the wrong reason, so both are asserted.
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
        // **Measured, not chosen: 0.9874.** The gate sits one notch under it. 1.0 is not
        // reachable and a gate near it would be measuring the codec rather than the
        // render — the reference side is a whole H.264 frame of cobweb at 1.28 Mb/s,
        // resampled with a different filter from the one that encoded it.
        //
        // The headroom is 0.012, which is thin only if the render varies across targets.
        // It does not meaningfully: #34's oracle measured `skia-safe` differing by at
        // most 2/255 on a handful of pixels between the six tier-1 targets, and the
        // resample and the SSIM here are both pure deterministic Rust. Every run prints
        // its score, so tightening this is a matter of reading a log.
        //
        // It was 0.9362 over 85.8% before #276 split the photograph out into its own
        // half. The number rose because what is left is flat brand geometry rather than a
        // recompressed photograph — the same render, measured over a region where a
        // difference means something.
        drawn: Gated {
            gate: 0.975,
            least_coverage: 0.2,
        },
        // **Measured, not chosen: 0.8578**, and the lowest gate in this file by a distance.
        // The photograph is the one thing in the frame that arrives through H.264 at
        // 1.28 Mb/s, and cobweb at that bitrate is where a codec spends its errors. The
        // residual `what_the_corrected_pivot_does_not_explain_is_measured_and_reported`
        // measures is in here too.
        photograph: Gated {
            gate: 0.845,
            least_coverage: 0.5,
        },
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
        // **Measured, not chosen: 0.9646.** The region is the flag's three rectangles, the
        // badge, two cream panels, the navy card's own top and bottom bands and the cream
        // ground. Unchanged by #276 — this half never contained the photograph.
        drawn: Gated {
            gate: 0.953,
            least_coverage: 0.2,
        },
        // **Measured, not chosen: 0.6045**, and this is the number #276 exists to produce.
        // The photograph was masked out of this frame entirely until the fixture's pivot
        // was corrected; with the centre pivot in the file it can be compared, and the ramp
        // here is at 1.0586 — far enough up it that the Ken Burns move is finally gated
        // somewhere, which is the one thing #213 asked for and could not have.
        //
        // Low, and every part of the distance from the intro frame's 0.8578 is accounted
        // for: the ramp has magnified the photograph by 5.9%, so the render and the
        // reference disagree about every high-frequency pixel in it, and the residual
        // displacement measured below is 8 × 4 px here against 2 × 2 px there. What it is
        // not is slack: `a_deliberately_broken_render_fails_the_gated_comparison` puts two
        // photograph breaks under it at the intro frame, and the top-left spelling this
        // ticket replaced scores 0.3919 here.
        photograph: Gated {
            gate: 0.59,
            least_coverage: 0.5,
        },
    },
];

/// The photograph, which is every photo element's own `clip`: `[0, 0, 1080, 1300]`.
const PHOTOGRAPH: Region = Region {
    x: 0,
    y: 0,
    width: 1080,
    height: 1300,
};

/// The two header panels — Montaget's own rectangles, drawn *over* the photograph.
///
/// They are not part of the photograph, so the measurement below subtracts them: a number
/// about the Ken Burns move must not be diluted by two flat cream rectangles that match
/// whatever the move does.
const PANELS: [(&str, Region); 2] = [
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

/// One of the two regions a frame is gated over. Each is gated; neither is the whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Half {
    /// Everything Montaget draws from the document's own geometry: the cream ground, the
    /// navy card, the flag's three rectangles, the badge and both header panels.
    Drawn,
    /// The photograph, and therefore the Ken Burns move that carries it.
    Photograph,
}

impl Reference {
    /// The region for one half.
    ///
    /// **[`Half::Drawn`] is the whole frame less the photograph, plus the panels drawn over
    /// it, less the text — in that order, because the last layer to name a pixel is the one
    /// that decides.** The panels have to come back after the photograph goes out: they are
    /// Montaget's own rectangles, painted *on* the photograph, and losing them with it
    /// would take the frame's two cleanest pieces of geometry out of the gate.
    fn scope(&self, half: Half) -> Scope {
        match half {
            Half::Drawn => Scope::whole()
                .less(vec![PHOTOGRAPH.scaled(
                    FRAME_WIDTH,
                    REFERENCE_WIDTH,
                    MARGIN,
                )])
                .plus(scaled(&PANELS))
                .less(scaled(self.text)),
            Half::Photograph => photograph_only(),
        }
    }

    /// What that half must clear, and how much of the frame it must still cover.
    fn gated(&self, half: Half) -> &Gated {
        match half {
            Half::Drawn => &self.drawn,
            Half::Photograph => &self.photograph,
        }
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
    //
    // Two halves per frame, each with its own number: see [`Reference::photograph_gate`]
    // for why the photograph is not folded into one.
    for reference in &REFERENCES {
        let (theirs, ours) = planes(reference, &fixture_project());
        for half in [Half::Drawn, Half::Photograph] {
            let mask = reference.scope(half);
            let Gated {
                gate,
                least_coverage,
            } = *reference.gated(half);
            let coverage = mask.coverage(theirs.width, theirs.height);
            let score = ssim(&theirs, &ours, &mask);

            println!(
                "GATING  {} at {} ms, {half:?} — SSIM {score:.4} over {:.1}% of the frame \
                 (gate {gate}).\n\
                 \x20        instant: {}\n\
                 \x20        {}",
                reference.file,
                reference.at,
                coverage * 100.0,
                reference.provenance,
                match half {
                    // Each half names what it actually subtracted, rather than one line
                    // that is true of the other one: the text is masked out of the drawn
                    // geometry it is painted on, and the panels out of the photograph they
                    // are painted on. The text needs no second subtraction here — both
                    // text elements over the photograph sit inside those panels.
                    Half::Drawn => format!(
                        "the photograph is out; the panels drawn on it are back in [{}]; \
                         text is out [{}]",
                        named(&PANELS),
                        named(reference.text)
                    ),
                    Half::Photograph =>
                        format!("the panels drawn over it are out [{}]", named(&PANELS)),
                },
            );
            assert!(
                coverage >= least_coverage,
                "{} {half:?}: a gate over {:.1}% of the frame is not the gate this \
                 reference claims to be — the masks have swallowed the picture",
                reference.file,
                coverage * 100.0
            );
            assert!(
                score >= gate,
                "{} at {} ms, {half:?}: SSIM {score:.4} is below the gate {gate}. This is \
                 the comparison that can falsify the format — look at the two pictures \
                 before touching the number.",
                reference.file,
                reference.at,
            );
        }
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
    // drawn region's: if the text ever matched as well as the geometry it sits on, the mask
    // would be unnecessary and this file would be overstating its case. Against the drawn
    // half rather than against both, because the text is drawn *on* that geometry — the
    // photograph is a different picture with a different reason for diverging (#276), and
    // measuring the typeface against it would mix the two.
    for reference in &REFERENCES {
        let (theirs, ours) = planes(reference, &fixture_project());
        let text = reference.text_only();
        let in_text = ssim(&theirs, &ours, &text);
        let gated = ssim(&theirs, &ours, &reference.scope(Half::Drawn));

        println!(
            "NON-GATING  {} at {} ms — text SSIM {in_text:.4} over {:.1}% of the frame, \
             against {gated:.4} in the drawn region.\n\
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
            "{}: the text scored {in_text:.4} against {gated:.4} in the drawn region. If \
             the typeface substitution has stopped costing anything, #186 can close and \
             this mask should go.",
            reference.file
        );
    }
}

/// The photograph alone, less the two panels drawn over it — the region a statement about
/// the Ken Burns move has to be made over.
///
/// **No text is subtracted here, and that is a claim, not an omission.** Every text element
/// in either reference either sits below the photograph entirely or sits inside one of the
/// two panels that are subtracted — so the typeface substitution (#186) cannot reach this
/// number. Asserted rather than commented, for the reason `Region::scaled` gives about its
/// own margin: this is the sort of thing that silently stops being true when somebody moves
/// an element, and a photograph gate quietly measuring Open Runde against SF Pro Rounded
/// would be a gate measuring the wrong thing.
fn photograph_only() -> Scope {
    for reference in &REFERENCES {
        for (name, text) in reference.text {
            let below = text.y >= PHOTOGRAPH.y + PHOTOGRAPH.height;
            let inside_a_panel = PANELS.iter().any(|(_, panel)| {
                text.x >= panel.x
                    && text.y >= panel.y
                    && text.x + text.width <= panel.x + panel.width
                    && text.y + text.height <= panel.y + panel.height
            });
            assert!(
                below || inside_a_panel,
                "`{name}` is inside the photograph's region and inside neither header \
                 panel, so the photograph's gate is measuring the typeface as well as the \
                 Ken Burns move. Subtract it here, or say why it is admissible."
            );
        }
    }
    Scope::inside(vec![PHOTOGRAPH.scaled(
        FRAME_WIDTH,
        REFERENCE_WIDTH,
        -MARGIN,
    )])
    .less(scaled(&PANELS))
}

/// The pivot the migration first wrote, as an exact-string edit of the committed fixture.
///
/// Both halves of the rewrite are checked against the file by the test that uses them, so
/// this cannot rot into a no-op that silently proves nothing.
const AS_THE_MIGRATION_FIRST_WROTE_IT: (&str, &str) = (
    r#""x":540,"y":956,"origin":"center""#,
    r#""x":0,"y":0,"origin":"top-left""#,
);

#[test]
fn the_corrected_pivot_beats_the_one_the_migration_first_wrote() {
    // #276, from the other end. The committed fixture now declares `origin: "center"` at
    // (540, 956); until #276 it declared `origin: "top-left"` at (0, 0), which is the same
    // rectangle at scale 1.0 and a different one at every other scale. The two spellings
    // are therefore indistinguishable at the intro frame and 32 × 56 px apart at 14 000 ms,
    // and only the published video can say which is right.
    //
    // This renders both against the reference and asserts the committed one wins, which is
    // the assertion that would catch the correction being reverted — including by a
    // re-run of `migrate.py` from a copy that never got the fix. It is deliberately not a
    // frozen number: what is claimed is an ordering between two renders, and an ordering
    // survives a `skia-safe` bump that a threshold would not.
    let original = std::fs::read_to_string(fixture_project()).expect("the fixture");
    let (committed, first_written) = AS_THE_MIGRATION_FIRST_WROTE_IT;
    assert!(
        original.contains(committed),
        "the committed fixture no longer declares the centre pivot #276 corrected it to: \
         {committed}"
    );
    let top_left = Scratch::beside_the_fixture(
        "pivot-under-test",
        &original.replace(committed, first_written),
    );
    assert!(
        std::fs::read_to_string(top_left.path())
            .expect("the rewritten project")
            .contains(first_written),
        "the rewrite to the old pivot did not take: {first_written}"
    );

    let photograph = photograph_only();
    for reference in &REFERENCES {
        let theirs = published(reference.file);
        let plane = |project: &Path| {
            Plane::of(&resized(
                &rendered(project, reference.at, /* full */ true),
                theirs.width(),
                theirs.height(),
            ))
        };
        let theirs = Plane::of(&theirs);
        let centre = ssim(&theirs, &plane(&fixture_project()), &photograph);
        let top = ssim(&theirs, &plane(top_left.path()), &photograph);
        // The ramp each element declares: 1.0 at its first keyframe, 1.08 fifteen seconds
        // later. `photo-05-intro` starts at 0; `photo-05` restarts at 3 018.
        let ramp = match reference.at {
            400 => 1.0 + 0.08 * 400.0 / 15000.0,
            at => 1.0 + 0.08 * (at as f64 - 3018.0) / 15000.0,
        };
        println!(
            "PIVOT   {} at {} ms — photograph SSIM {centre:.4} about the centre, \
             {top:.4} about the top-left, ramp at {ramp:.4} ({:.0} px apart).",
            reference.file,
            reference.at,
            (ramp - 1.0) * 540.0,
        );
        // Only at the later frame. At 400 ms the ramp is 1.0021 and the two spellings put
        // the photograph one pixel apart, which is under this comparison's resolution and
        // below the residual measured beside it — so the intro frame has no opinion, and
        // asserting one here would be reading noise as evidence.
        if reference.at == 14000 {
            assert!(
                centre > top,
                "{} at {} ms: the photograph matched the published video better about the \
                 *top-left* ({top:.4}) than about the centre ({centre:.4}). #276's \
                 correction rests on the opposite, and this is the measurement it rests on.",
                reference.file,
                reference.at,
            );
        }
    }
}

#[test]
fn what_the_corrected_pivot_does_not_explain_is_measured_and_reported() {
    // #276 asked for this explicitly: *"there may be a second, smaller residual underneath
    // this one — measure it rather than assuming this fix closes it."* There is one.
    //
    // Searching the photograph's displacement against the reference, with the centre pivot
    // and the declared ramp both in place, the best match is **2 px right and 2 px down**
    // at 400 ms and **8 px right and 4 px down** at 14 000 ms (mean absolute difference
    // 6.27 → 5.10 and 11.24 → 3.63, over a ±24 px sweep in 4 px steps). So a small
    // displacement is left, and it is a quarter of the horizontal error the pivot defect
    // carried and a fourteenth of the vertical one.
    //
    // `docs/research/sample-project-migration/README.md` D3 flagged the same thing from the
    // other side — *"the pure-zoom model's fit quality falls with `t`"* — and it is
    // recorded rather than chased: the displacement implied by it puts the pivot at about
    // 0.40 of the box horizontally, which is not one of ADR-0013's nine keywords and so is
    // not a thing this format can spell. Whatever it is, it is not a pivot.
    //
    // Reported without a threshold, for the same reason as the typeface: a number here
    // would freeze a residual that a later ticket may explain and remove.
    let photograph = photograph_only();
    for reference in &REFERENCES {
        let (theirs, ours) = planes(reference, &fixture_project());
        let score = ssim(&theirs, &ours, &photograph);
        println!(
            "NON-GATING  {} at {} ms — photograph SSIM {score:.4} over {:.1}% of the \
             frame; this region is gated as [`Half::Photograph`], the residual below is \
             not.\n\
             \x20           The centre pivot #276 wrote leaves a displacement of 2 px at \
             400 ms and 8 px at 14 000 ms, growing with the ramp; it is named in this test \
             and owned by nothing yet.",
            reference.file,
            reference.at,
            photograph.coverage(theirs.width, theirs.height) * 100.0,
        );
    }
}

// ---------------------------------------------------------------------------
// The threshold is sensitive
// ---------------------------------------------------------------------------

#[test]
fn a_deliberately_broken_render_fails_the_gated_comparison() {
    // The claim a passing threshold cannot make on its own. Each break is inside the
    // region of the reference *and the half* it is checked against, and each is the size of
    // defect this project has actually shipped: a card in the wrong place, a panel the
    // wrong colour, the wrong photograph, an image off its mark.
    //
    // Naming the half is not bookkeeping. Two of these four break the photograph and two
    // break the geometry drawn over it, and since #276 split the gate in two, a break
    // checked against the other half would be a break checked against pixels it does not
    // touch — which is a test that passes for the wrong reason.
    let original = std::fs::read_to_string(fixture_project()).expect("the fixture");

    let breaks: [(usize, Half, &str, &str, &str); 4] = [
        (
            1,
            Half::Drawn,
            "the navy sentence card sits 40 px high",
            r#""id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"x":48,"y":1453"#,
            r#""id":"card-05","type":"rect","group":"item-05","start":10468,"end":17472,"x":48,"y":1413"#,
        ),
        (
            1,
            Half::Drawn,
            "the cream chip panel turns navy",
            r##""id":"chip-panel","type":"rect","group":"header","start":0,"end":65216,"x":48,"y":88,"origin":"top-left","width":372,"height":84,"fill":"#FBF3E3""##,
            r##""id":"chip-panel","type":"rect","group":"header","start":0,"end":65216,"x":48,"y":88,"origin":"top-left","width":372,"height":84,"fill":"#1E344C""##,
        ),
        (
            0,
            Half::Photograph,
            "the intro runs over the wrong photograph",
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/05.png""#,
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/06.png""#,
        ),
        (
            0,
            Half::Photograph,
            "the photograph sits 40 px down",
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/05.png","x":540,"y":956"#,
            r#""id":"photo-05-intro","type":"image","group":"item-05","start":0,"end":3018,"source":"images/05.png","x":540,"y":996"#,
        ),
    ];

    // The unbroken score for each reference *and half*, so the cost of a break is a
    // measured difference rather than a distance from a constant.
    let unbroken = |reference: &Reference, half: Half| {
        let (theirs, ours) = planes(reference, &fixture_project());
        ssim(&theirs, &ours, &reference.scope(half))
    };

    for (which, half, what, from, to) in breaks {
        let reference = &REFERENCES[which];
        let before = unbroken(reference, half);
        assert!(
            original.contains(from),
            "the break `{what}` no longer matches the fixture: {from}"
        );
        // Written **beside the fixture**, because a project's relative paths resolve
        // against its own directory (ADR-0053): moving it to a scratch directory would
        // change the photograph, the badge and the font too, and the comparison would then
        // be measuring the move rather than the break.
        let broken = Scratch::beside_the_fixture("broken-under-test", &original.replace(from, to));

        let theirs = published(reference.file);
        let ours = resized(
            &rendered(broken.path(), reference.at, /* full */ true),
            theirs.width(),
            theirs.height(),
        );
        let score = ssim(
            &Plane::of(&theirs),
            &Plane::of(&ours),
            &reference.scope(half),
        );
        let drop = before - score;
        let gate = reference.gated(half).gate;
        println!(
            "BREAK   {what} — SSIM {score:.4} against {} {half:?}, down {drop:.4} from \
             {before:.4} (gate {gate})",
            reference.file,
        );
        assert!(
            drop >= MIN_DROP,
            "`{what}` cost only {drop:.4} ({before:.4} to {score:.4}) over {} {half:?}. A \
             comparison a break does not move is a comparison that tests nothing \
             (ADR-0010).",
            reference.file,
        );
        assert!(
            score < gate,
            "`{what}` scored {score:.4} against {} {half:?}, which its gate {gate} still \
             passes.",
            reference.file,
        );
    }
}

/// A project file written into the fixture's own directory and removed again, even when
/// the assertion between the two panics.
struct Scratch(PathBuf);

impl Scratch {
    fn beside_the_fixture(name: &str, body: &str) -> Scratch {
        // One fixed name **per caller**, rather than a unique one per write: the breaks
        // above run in sequence inside a single test, so they can share one file — but the
        // harness runs the *tests* on parallel threads, and two of them writing one path
        // would race. Naming it at the call site is what keeps that visible. None of these
        // is a project any other test reads and nothing globs this directory for
        // `*.montaget.json`, so the only cost of a copy that a hard abort leaves behind is
        // a line of `git status` — and the next run overwrites it.
        let path = fixture_dir().join(format!("{name}.montaget.json"));
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
