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
//! spellings are 1 × 2 px apart, which is smaller than the residual below and so tells you
//! nothing — the intro frame actually scores the *rejected* spelling higher, for the reason
//! [`the_corrected_pivot_beats_the_one_the_migration_first_wrote`] sets out. At 14.0 s, with
//! the ramp at 1.0586, they are 32 px apart horizontally and 56 px vertically, and the
//! photograph plainly did not match. `ci/reference_frame_instants.py` re-derives the scale
//! and offset from the pictures and shows the centre pivot fitting where the declared one
//! did not.
//!
//! That was a defect in the **fixture**, not in this build: the renderer paints the pivot
//! the document names, and #212's tests fix that reading against all nine keywords. So the
//! repair is in `migrate.py`, which regenerates the committed file, and it is checked from
//! this side by [`the_corrected_pivot_beats_the_one_the_migration_first_wrote`] — which
//! renders both spellings against the published frame rather than trusting either.
//!
//! **What it was not, at any point, is absorbed into a looser threshold.** While it stood
//! it was quantified, masked out of the gate at the frame where it bit, and reported as a
//! number on every run — the same treatment ADR-0085 makes permanent for the typeface — and
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
//! numbers and what is known about them, and
//! [#299](https://github.com/MBehtemam/Montaget/issues/299) owns it.
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
//! defect. [#186](https://github.com/MBehtemam/Montaget/issues/186) measured it and
//! [ADR-0085](../../../docs/adr/0085-the-font-swap-census-holds-and-the-text-mask-is-permanent.md)
//! settled it: the fixture's declared layouts all still hold, and the text mask is
//! nevertheless **permanent**, because its cause is a licence rather than a bug. Text is
//! masked out of every gate and measured beside it — and stays that way.
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
//! Between them they are the whole frame, less the text —
//! [`the_two_halves_leave_no_pixel_ungated`] asserts it per pixel, after an earlier draft
//! left a band around the photograph's own boundary in neither half.
//!
//! **One gate would be one number over two pictures that diverge for different reasons.**
//! The photograph is 60% of the frame, arrives through H.264 at 1.28 Mb/s, and matches
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

use std::path::Path;

mod common;
use common::compare::{Plane, RADIUS, Region, Scope, rendered, resized, ssim};
use common::{Scratch, fixture_dir, fixture_project};

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
        // **Measured, not chosen: 0.8618.** Low for a gate in this file, though not the
        // lowest — the same frame's later sibling sits at 0.59.
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
        // **Measured, not chosen: 0.6142**, and this is the number #276 exists to produce.
        // The photograph was masked out of this frame entirely until the fixture's pivot
        // was corrected; with the centre pivot in the file it can be compared, and the ramp
        // here is at 1.0586 — far enough up it that the Ken Burns move is finally gated
        // somewhere, which is the one thing #213 asked for and could not have.
        //
        // Low, and every part of the distance from the intro frame's 0.8618 is accounted
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
    ///
    /// **The two halves partition the frame, and
    /// [`the_two_halves_leave_no_pixel_ungated`] holds them to it.** Both use the *same*
    /// [`photograph`] rectangle — one subtracting it, the other admitting it — so there is
    /// no band that belongs to neither. Growing one and shrinking the other by [`MARGIN`],
    /// which is right when a region is *excluded* for a known divergence, would leave the
    /// photograph's own boundary gated by nothing at all: the seam between the photograph
    /// and the cream card below it, which is the edge a `clip`-height defect moves.
    fn scope(&self, half: Half) -> Scope {
        // Text comes out of both halves, so the typeface substitution (#186) cannot reach
        // either number. It is the one thing this file measures and never gates.
        match half {
            Half::Drawn => Scope::whole()
                .less(vec![photograph()])
                .plus(scaled(&PANELS))
                .less(scaled(self.text)),
            Half::Photograph => Scope::inside(vec![photograph()])
                .less(scaled(&PANELS))
                .less(scaled(self.text)),
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
    // Two halves per frame, each with its own number: see [`Reference::photograph`]
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
    // ADR-0085's quantification, at the two instants the suite has references for. There is
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
             redistributable (ADR-0057). ADR-0085 settles it as permanent.\n\
             \x20           Elements: {}",
            reference.file,
            reference.at,
            text.coverage(theirs.width, theirs.height) * 100.0,
            named(reference.text),
        );
        assert!(
            in_text < gated,
            "{}: the text scored {in_text:.4} against {gated:.4} in the drawn region. \
             ADR-0085 holds that the substitution costs something at every instant, \
             because SF Pro Rounded cannot be vendored and Open Runde's advances run \
             2.0-9.4% wider. If that has stopped being true, the mask is overstating its \
             case and ADR-0085 is what needs revisiting.",
            reference.file
        );
    }
}

/// The photograph's rectangle at the comparison's size — **the one boundary both halves
/// use**, so that what [`Half::Drawn`] subtracts is exactly what [`Half::Photograph`]
/// admits and no band belongs to neither.
fn photograph() -> Region {
    PHOTOGRAPH.scaled(FRAME_WIDTH, REFERENCE_WIDTH, MARGIN)
}

/// The photograph's *interior* — its rectangle shrunk by [`MARGIN`], less the panels and
/// the text — for the two tests that **measure the photograph's content** rather than gate
/// the frame.
///
/// **Deliberately not [`photograph`], and the difference is the point.** A gate must cover
/// the photograph's boundary, because the seam where it meets the cream card is real
/// geometry a defect can move. A measurement of *where the photograph sits* must exclude
/// that boundary: the seam is drawn at the aperture and stays put however the picture
/// behind it slides, so a search for the picture's displacement that could see the seam
/// would be pulled toward reporting no displacement at all.
fn photograph_interior() -> Scope {
    Scope::inside(vec![PHOTOGRAPH.scaled(
        FRAME_WIDTH,
        REFERENCE_WIDTH,
        -MARGIN,
    )])
    .less(scaled(&PANELS))
}

#[test]
fn the_two_halves_leave_no_pixel_ungated() {
    // The property that makes "two gates" honest rather than two gates with a hole between
    // them. Every pixel of every reference is in exactly one of three places: the drawn
    // half, the photograph half, or the text that is measured and never gated.
    //
    // **This is a regression test for a real hole.** When the photograph was first split
    // out, `Drawn` subtracted it grown by `MARGIN` while `Photograph` admitted it *shrunk*
    // by `MARGIN` — the two conventions that are each correct on their own, for excluding
    // and for measuring. Together they left a 16-pixel band around the photograph's
    // boundary in neither gate, including the seam where the photograph meets the cream
    // card, which is precisely the edge a `clip`-height defect moves. The gates both
    // passed and 8.5% of the frame was checked by nothing.
    for reference in &REFERENCES {
        let theirs = published(reference.file);
        let (width, height) = (theirs.width() as usize, theirs.height() as usize);
        let drawn = reference.scope(Half::Drawn);
        let photo = reference.scope(Half::Photograph);
        let text = reference.text_only();

        let total = drawn.coverage(width, height)
            + photo.coverage(width, height)
            + text.coverage(width, height);
        println!(
            "COVERAGE  {} — drawn {:.1}% + photograph {:.1}% + text {:.1}% = {:.1}%",
            reference.file,
            drawn.coverage(width, height) * 100.0,
            photo.coverage(width, height) * 100.0,
            text.coverage(width, height) * 100.0,
            total * 100.0,
        );

        // Per pixel rather than on the sum alone: three coverages can add to 1.0 with a
        // pixel double-counted in one place and missing in another.
        for y in 0..height {
            for x in 0..width {
                let places = [&drawn, &photo, &text]
                    .iter()
                    .filter(|scope| scope.admits(x, y))
                    .count();
                assert_eq!(
                    places, 1,
                    "{}: pixel ({x}, {y}) is in {places} of the three regions, not 1. \
                     The halves have stopped partitioning the frame — a pixel in none is \
                     gated by nothing, and one in two is gated twice under two thresholds.",
                    reference.file
                );
            }
        }
    }
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

    let photograph = photograph_interior();
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
        // **Only at the later frame, and the intro frame's own number says why rather than
        // hiding it.** At 400 ms the intro frame scores the *rejected* spelling higher —
        // 0.9188 about the top-left against 0.8578 about the centre — and that is not
        // evidence for `top-left`. At a ramp of 1.0021 the two spellings put the photograph
        // about 1 px apart horizontally and 2 px vertically, while the unexplained residual
        // `what_the_corrected_pivot_does_not_explain_is_measured_and_reported` measures is
        // (+2, +2) px in almost exactly the direction `top-left` displaces. So at this
        // instant the comparison is reading the residual (#299) and not the pivot at all,
        // and an assertion either way would be reading that residual as evidence about
        // something else. At 14 000 ms the two spellings are 32 × 56 px apart — an order of
        // magnitude above the residual — and the frame can speak.
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
    // **The displacement is searched here, not quoted here.** An earlier draft of this test
    // printed those numbers as string literals and asserted nothing, which is a test that
    // cannot stop agreeing with itself: the residual could move to 20 px and the line would
    // still read "2 px". The search below re-derives it every run, and what is asserted is
    // the *shape* — that it grows with the ramp — because that is the part which says this
    // is a systematic divergence rather than noise. The magnitude stays un-thresholded, for
    // the same reason as the typeface: freezing it would make explaining it a test failure.
    let mut best = Vec::new();
    for reference in &REFERENCES {
        let photograph = photograph_interior();
        let theirs = published(reference.file);
        let ours = resized(
            &rendered(&fixture_project(), reference.at, /* full */ true),
            theirs.width(),
            theirs.height(),
        );
        let theirs = Plane::of(&theirs);

        // Whole reference pixels — four of the project's — because that is the finest
        // shift this comparison can actually resolve. The offsets recorded in #299 were
        // measured in project pixels by a separate sweep; these are the same quantity at
        // this suite's own resolution.
        let (shift, score) = (0..=RESIDUAL_REACH)
            .flat_map(|dy| (0..=RESIDUAL_REACH).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| {
                let shifted = Plane::of(&translated(&ours, dx, dy));
                ((dx, dy), ssim(&theirs, &shifted, &photograph))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).expect("SSIM is never NaN"))
            .expect("the sweep is not empty");

        let at_rest = ssim(&theirs, &Plane::of(&ours), &photograph);
        println!(
            "NON-GATING  {} at {} ms — photograph SSIM {at_rest:.4} as written; the best \
             match is {:+}, {:+} reference px away at {score:.4}.\n\
             \x20           That displacement is what the centre pivot #276 wrote does not \
             explain. It is owned by #299.",
            reference.file, reference.at, shift.0, shift.1,
        );
        best.push((reference.at, shift));
    }

    let (early_at, early) = best[0];
    let (late_at, late) = best[1];
    let magnitude = |(dx, dy): (i64, i64)| dx.abs() + dy.abs();
    assert!(
        magnitude(late) > magnitude(early),
        "the residual displacement at {late_at} ms ({late:?}) is not larger than at \
         {early_at} ms ({early:?}). It growing with the ramp is what makes it a systematic \
         divergence rather than noise, and it is the one thing this test asserts — see \
         #299, which is written on the assumption."
    );
}

/// How far the residual sweep reaches, in reference pixels.
///
/// Four, which is 16 of the project's — twice the largest displacement #299 records, so
/// the minimum it finds is an interior one rather than the edge of the search. One
/// direction only: every measurement so far puts the published photograph down and to the
/// right of where the fixture puts it, and a sweep that admitted the other sign would
/// spend four times the renders confirming it.
const RESIDUAL_REACH: i64 = 4;

/// The same picture, moved by whole pixels, with the vacated edge left black.
///
/// The comparison that uses this is scoped to the photograph's interior, so the vacated
/// edge never enters a window that is scored.
fn translated(image: &image::RgbaImage, dx: i64, dy: i64) -> image::RgbaImage {
    let (width, height) = (image.width(), image.height());
    let mut out = image::RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let (from_x, from_y) = (x as i64 - dx, y as i64 - dy);
            if from_x >= 0 && from_y >= 0 && (from_x as u32) < width && (from_y as u32) < height {
                out.put_pixel(x, y, *image.get_pixel(from_x as u32, from_y as u32));
            }
        }
    }
    out
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
    //
    // Computed once per reference and reused, rather than per break: the two are the same
    // number, and each costs a full 1080×1920 render of the fixture in the slowest test in
    // this file.
    let unbroken: Vec<[f64; 2]> = REFERENCES
        .iter()
        .map(|reference| {
            let (theirs, ours) = planes(reference, &fixture_project());
            [
                ssim(&theirs, &ours, &reference.scope(Half::Drawn)),
                ssim(&theirs, &ours, &reference.scope(Half::Photograph)),
            ]
        })
        .collect();

    for (which, half, what, from, to) in breaks {
        let reference = &REFERENCES[which];
        let before = unbroken[which][match half {
            Half::Drawn => 0,
            Half::Photograph => 1,
        }];
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
