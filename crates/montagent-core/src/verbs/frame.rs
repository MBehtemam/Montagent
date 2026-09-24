//! `frame` — *"what does it look like right now"* (ADR-0011).
//!
//! The first pixels, and the loop an agent runs every turn — which is why its budget is
//! the primary one (ADR-0021: under 500 ms **cold**, at true pixel dimensions, and
//! resolution-independent).
//!
//! ## Three decisions this verb is built on, all ADR-0011's
//!
//! **JPEG at half the project's frame size by default, full scale and PNG behind flags.**
//! Not because full scale is redundant — it is not, and the reasoning that said so was
//! wrong in both directions — but because an image costs an agent
//! `⌈width/28⌉ × ⌈height/28⌉` visual tokens whatever it is encoded as: 2691 at 1080×1920
//! against 700 at 540×960, every single time it looks. The encoding is a latency-and-disk
//! choice and nothing more.
//!
//! **The `query --at` block prints alongside the image, unconditionally.** *"Looking at a
//! picture without knowing which elements produced it is how a defect gets attributed to
//! the wrong element."* It is the picture's caption rather than an alternative to it, so
//! there is no flag that suppresses it — and it is the *same* block, from the same
//! [`crate::verbs::query::at`] code, rather than a second rendering of the same idea.
//!
//! **And `frame` is demoted from a job it keeps being assigned.** It is how an agent
//! *believes* a layout; it is not how one *measures* one. An agent asked to settle a 26 px
//! overflow from a rendered frame could not, and fell back to pixel-scanning raw RGB —
//! where its first scan was wrong because the text colour `#FFF8E8` and the background
//! `#FBF3E3` differ by only (4, 5, 5). `measure` is what measures.
//!
//! ## What it does not do, and why that is not an omission
//!
//! **It runs no checks.** ADR-0006 gives `render` the enforcement — *"`render` runs the
//! identical check engine and refuses on any `error`"* — and that rule is about the
//! deliverable. `frame` is the verb an agent reaches for precisely *when the document is
//! wrong*, so refusing to show a broken project would remove the tool at the moment it is
//! needed. It renders what it can and names, beside the picture, everything it could not.
//!
//! **It reads `fit` nowhere.** ADR-0015's load-bearing sentence is that the declared rect
//! is authoritative at render, so `fit` *"never executes"* and no renderer reads it. That
//! is asserted rather than asserted-about: `tests/frame.rs` renders the same project with
//! the field present and absent and compares the bytes.
//!
//! **It opens no font outside the declared chain** (ADR-0007), and this is where that is
//! enforced — at the renderer, rather than at `measure`. Every present text element's
//! chain is registered through the same [`crate::verbs::measure::register`] `measure`
//! uses, and the files that were actually opened come back in the answer, so a decoy font
//! installed on the machine is not merely unused but demonstrably unopened.
//!
//! **Glyphs are painted through the engine `measure` answers with** (#213). The line
//! partition, the slot heights and every baseline come back from
//! [`montagent_text::place`], which reads them off the same shaping pass
//! [`montagent_text::measure`] uses — so what `measure` tells an author and what the
//! picture shows are one derivation rather than two that agree.
//!
//! **The rest of the paint vocabulary is
//! [#214](https://github.com/MBehtemam/Montagent/issues/214)**, and it arrives in this verb
//! split three ways, because the three things are three different kinds of rule. The
//! ordered `effects` list is *painting*, so it is read here ([`Painter::effects_of`]) and
//! applied there ([`montagent_render::canvas::Effect`]). A `crossfade` is *resolution* —
//! ADR-0059 makes a transition "purely descriptive", and what it descriptively says is
//! what the two bridged elements' opacities are — so it is settled before a pixel is drawn
//! ([`Painter::resolve_crossfades`]) and the rasterizer never hears of it. A run's
//! `highlight` is the same shape of thing one level down: which of the run's two declared
//! styles applies at this instant ([`highlight_at`]).
//!
//! Every element this build cannot paint is still listed in the answer with the reason,
//! because an agent that cannot tell "not there" from "not drawn yet" will chase the wrong
//! defect — and so is every element painted without something it asked for.
//!
//! ## The surface, and what of it no ADR states
//!
//! ADR-0011 fixes the default encoding, the default scale, `--crop x,y,w,h` and the
//! unconditional caption. It does not say where the CLI puts the bytes, what JPEG quality
//! is, or what a project that declares no `background` is painted on. Those are this
//! ticket's, recorded here and raised as
//! [#274](https://github.com/MBehtemam/Montagent/issues/274) rather than left to be
//! discovered from the code: the CLI takes a required `--out` rather than inventing a
//! filename in somebody's project directory; quality is
//! [`montagent_render::canvas::JPEG_QUALITY`]; and an absent `background` is opaque black
//! (argued at [`montagent_render::canvas::Rgba::BLACK`]).
//!
//! #214 adds six more again, collected at
//! [#280](https://github.com/MBehtemam/Montagent/issues/280) and argued at their sites: a
//! blur or shadow `radius` is `2σ` rather than `σ`, a crossfade's ramp is linear,
//! `opacity` composites outside the effects, a shadow's `opacity` multiplies its colour's
//! alpha, a `highlight` is paint only, and the answer gains a `crossfades` block.
//!
//! #213 adds six more of the same kind, collected at
//! [#277](https://github.com/MBehtemam/Montagent/issues/277) and argued at their sites: an
//! absent `color` is opaque black ([`DEFAULT_INK`]), an absent `align` is `start`
//! ([`align_of`]), and `align`'s box is the block the lines make rather than the
//! element's declared `width` (argued in [`montagent_text::place`], from ADR-0007's *"how
//! lines align to each other"* and ADR-0014's *"a container claim, not painted
//! geometry"*).

use std::path::{Path as FilePath, PathBuf};

use serde::Serialize;
use serde_json::Value;

use montagent_render::canvas::{
    Canvas, Effect, Encoded, Encoding, Extent, Fill, Glyph, MaskRect, MaskShape, PathEl, Raster,
    Region, Rgba, Scale, Shape, Transform,
};

use crate::finding::Finding;
use crate::media::{Source, tools};
use crate::model::{self, Colour, Origin};
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;
use crate::verbs::query::Named;
use crate::verbs::query::at::{self, At};
use crate::verbs::query::geometry::{self, Rect};

const TOOL: &str = "frame";

/// What one `frame` invocation is asking.
///
/// The flags as they arrive from either adapter, and no resolved mode: which combinations
/// are legal is a rule about the verb, and both adapters are thin by construction
/// (ADR-0011).
#[derive(Debug, Clone, Default)]
pub struct Ask {
    /// The instant to draw, in absolute milliseconds. Required — there is no default
    /// instant, and a verb that chose one would be choosing what the agent is looking at.
    pub at: Option<i64>,
    /// `x,y,w,h` in frame space at true pixels, as the caller wrote it. Parsed here rather
    /// than in an adapter, so the MCP surface and argv reject the same strings.
    pub crop: Option<String>,
    /// True pixels instead of half scale. ADR-0011's escape hatch: the caller asked for
    /// 2691 tokens and is paying for them.
    pub full: bool,
    /// PNG instead of JPEG.
    pub png: bool,
    /// Where to write the bytes. The CLI's; the MCP surface carries the image itself.
    pub out: Option<PathBuf>,
}

/// One `frame` invocation's answer: the picture, its caption, and the report every verb
/// answers with.
pub struct Answer {
    picture: Option<Picture>,
    /// The `query --at` view, printed unconditionally beside the image (ADR-0011).
    view: Option<At>,
    /// The encoded bytes. Off the canonical JSON deliberately — a base64 frame in a report
    /// would be megabytes of a form whose whole purpose is to be read.
    image: Option<Encoded>,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The encoded picture, where one was drawn. What the MCP adapter turns into an image
    /// content block, and what the CLI has already written to `--out`.
    pub fn image(&self) -> Option<&Encoded> {
        self.image.as_ref()
    }

    /// The canonical JSON: the report's own object, the `frame` block, and the `query`
    /// block the caption is generated from.
    ///
    /// Two keys rather than one nested inside the other, because the caption *is* a
    /// `query --at` answer: it renders through [`crate::text`]'s existing `at_block`, so
    /// the block an agent reads under `frame` is byte-for-byte the block it reads under
    /// `query --at`, and neither can drift from the other.
    pub fn to_json(&self) -> Value {
        let mut json = self.report.to_json();
        let object = json
            .as_object_mut()
            .expect("a report serialises as an object");
        object.insert(
            "frame".to_string(),
            match &self.picture {
                Some(picture) => serde_json::to_value(picture).unwrap_or(Value::Null),
                None => Value::Null,
            },
        );
        object.insert(
            "query".to_string(),
            match &self.view {
                Some(view) => serde_json::json!({
                    "mode": "at",
                    "at": view.at,
                    "stack": view.stack,
                    "unplaced": view.unplaced,
                    "not_covered": view.not_covered,
                    "not_covered_unresolved": view.not_covered_unresolved,
                }),
                None => Value::Null,
            },
        );
        json
    }
}

/// The picture, and everything about it an agent needs in order to trust what it is
/// looking at.
#[derive(Debug, Clone, Serialize)]
pub struct Picture {
    pub at: i64,
    /// `jpeg` or `png`.
    pub encoding: String,
    /// `half` or `full`.
    pub scale: String,
    /// The returned image's own pixel dimensions — which is what it costs to look at
    /// (ADR-0011: visual tokens are a pure function of decoded pixel dimensions).
    pub width: i64,
    pub height: i64,
    /// The project's true pixels, which is what was rasterized whatever the answer's scale
    /// is (ADR-0021: `frame` is never proxy-scaled).
    pub rasterized: Rasterized,
    /// The part of the frame this picture is, clipped to the frame — `null` for all of it.
    ///
    /// **`region`, not `crop`, and the difference is the glossary's.** `CONTEXT.md` lists
    /// *crop* under Clip's avoided words, and already spends *crop rectangle* on a
    /// different quantity: which part of a **source file's** pixels survive onto the
    /// screen, which is what the `query --at` block prints two lines below this one. One
    /// word meaning two things, adjacent in one answer, is how a reader ends up looking at
    /// the wrong number. The flag keeps the spelling ADR-0011 fixes for it; the field does
    /// not.
    ///
    /// The same [`Rect`] every other frame-space rectangle in the surface is written as: a
    /// second rectangle shape would be a second thing to parse.
    pub region: Option<Rect>,
    /// Where the bytes were written, where the caller asked for that.
    pub path: Option<String>,
    pub bytes: usize,
    /// Every element painted, in painter's order — the order the frame was built in, back
    /// to front, so the last name is the one on top.
    pub painted: Vec<String>,
    /// Every present element this build did not paint at all, and why.
    ///
    /// Its own list rather than silence: an agent that cannot tell *"the element is not
    /// there"* from *"this build does not draw that yet"* will go looking for a defect in
    /// the document, which is the failure the unconditional caption exists to prevent.
    pub not_painted: Vec<NotPainted>,
    /// Every element that *was* painted but not in full — one the document asks something
    /// of that this build cannot yet do.
    ///
    /// Kept apart from [`Picture::not_painted`] rather than folded into it, because the two
    /// send an agent to different places: an element missing from the picture and an
    /// element in the picture missing its shadow are not the same report, and one list
    /// naming both would have every entry read as the worse of the two.
    pub painted_partially: Vec<NotPainted>,
    /// Every `crossfade` running at this instant, with the window it runs over and how far
    /// through it this frame is.
    ///
    /// Its own list, because a transition draws nothing of its own and would otherwise be
    /// invisible in the answer while being the reason two elements are half-strength. The
    /// `query --at` block beside the picture prints each element's *declared* `opacity`,
    /// which is what the document says and not what the frame shows — and an agent reading
    /// `opacity 1` under a half-faded element goes looking for a defect in the wrong
    /// place. This is the sentence that closes that gap — new answer surface, raised for
    /// ratification at [#280](https://github.com/MBehtemam/Montagent/issues/280) on the
    /// precedent [#274](https://github.com/MBehtemam/Montagent/issues/274) set.
    pub crossfades: Vec<Crossfade>,
    /// Every media file opened, in the order it was opened.
    pub sources: Vec<String>,
    /// Every font file opened (ADR-0007). A file from outside the declared chain would
    /// appear here if one were ever opened, which is what makes the decoy test a test.
    pub fonts: Vec<String>,
}

/// The surface the frame was rasterized on, before any crop or scale.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Rasterized {
    pub width: i64,
    pub height: i64,
}

/// The one reason two element types give in the same words, so they give it in one place.
const NO_EXTENT: &str = "it states no positive integer `width`/`height`";

/// One present element the picture does not show, and the reason.
#[derive(Debug, Clone, Serialize)]
pub struct NotPainted {
    pub element: String,
    pub reason: String,
}

/// One `crossfade` in effect at the instant drawn (ADR-0059).
#[derive(Debug, Clone, Serialize)]
pub struct Crossfade {
    /// The transition element's own id.
    pub element: String,
    pub from: String,
    pub to: String,
    /// **The derived window** — the intersection of the two bridged elements' ranges —
    /// rather than whatever the transition declares. ADR-0059 requires the two to be equal
    /// and gives `validate` the drift; where they disagree this is the one the picture was
    /// drawn over.
    pub start: i64,
    pub end: i64,
    /// `0` at the window's start, approaching `1` at its end — the fraction of the way
    /// across, which is the outgoing element's lost opacity and the incoming one's gained.
    pub progress: f64,
}

/// Draw one frame.
pub fn frame(path: &FilePath, ask: &Ask) -> Answer {
    let project = Some(path.display().to_string());

    // The invocation is settled before the file is opened: `--crop 0,0,0,0` is wrong
    // whatever the document says, and ADR-0011 keeps exit 3 apart from exit 1 so that
    // "fix the command" is never read as "fix the project".
    let (instant, crop) = match request(ask) {
        Ok(request) => request,
        Err(reason) => {
            return Answer {
                picture: None,
                view: None,
                image: None,
                report: Report::rejected(TOOL, project, reason),
            };
        }
    };

    let document = match parse::read(path) {
        Ok(document) => document,
        // ADR-0011: nothing may partially process a malformed file.
        Err(finding) => {
            return Answer {
                picture: None,
                view: None,
                image: None,
                report: Report::unparseable(TOOL, project, *finding),
            };
        }
    };

    let mut report = Report::new(TOOL, project);

    if let Err(not_a_project) = document.shape() {
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point frame at the project file",
        ));
        return Answer {
            picture: None,
            view: None,
            image: None,
            report,
        };
    }

    // The caption first, and from the same code `query --at` answers with. It is also
    // where the offset into each video source comes from, so the picture and its caption
    // cannot disagree about which frame of a clip is on screen.
    let view = match at::answer(&document, instant) {
        Ok(view) => view,
        // No `ffmpeg`/`ffprobe` where the project needs one. ADR-0011's exit 70.
        Err(reason) => {
            report.fail_internally(reason);
            return Answer {
                picture: None,
                view: None,
                image: None,
                report,
            };
        }
    };

    let Some((frame_width, frame_height)) = at::frame_dimensions(&document) else {
        // Exit 1, not 70. `document.shape()` above has already confirmed a `frame` key is
        // there (ADR-0042's shared predicate), so reaching here means the key is present
        // and holds something that is not a frame — a defect in the *document*, and
        // ADR-0011 reserves 70 for "Montagent could not run" precisely so it is not
        // mistaken for one.
        //
        // ADR-0042's own code, with the same reading extended one level down: a file whose
        // `frame` carries no integer `width`/`height` does not have the shape of a project
        // any more than one carrying no `frame` at all. Which *way* it is malformed is
        // `validate`'s schema check to name, and `frame` runs no checks (#212 raises this
        // reading for ratification rather than leaving it to be found here).
        report.push(
            Finding::new("E-NOT-A-PROJECT")
                .at_file(document.path())
                .field(
                    "missing",
                    Value::String("integer `frame.width`/`frame.height`".to_string()),
                )
                .repair_value(serde_json::json!({
                    "value": "give `frame` an integer `width` and `height`, then run validate"
                })),
        );
        return Answer {
            picture: None,
            view: Some(view),
            image: None,
            report,
        };
    };

    // The crop is settled here and nowhere else: `request` checked its *spelling* without
    // the document, and this is the first point at which "does it reach the frame" can be
    // asked at all. A region that misses entirely is a wrong command rather than a broken
    // Montagent, so it is ADR-0011's exit 3 — the same exit an unspelled `--crop` gets, for
    // the same reason.
    let region = match crop {
        Some(crop) => match clamp(crop, frame_width, frame_height) {
            Some(region) => Some(region),
            None => {
                report = Report::rejected(
                    TOOL,
                    Some(document.path().to_string()),
                    format!(
                        "`--crop {},{},{},{}` does not overlap a {frame_width}x{frame_height} \
                         frame; a crop is a region of the frame",
                        crop.x, crop.y, crop.width, crop.height
                    ),
                );
                return Answer {
                    picture: None,
                    view: Some(view),
                    image: None,
                    report,
                };
            }
        },
        None => None,
    };

    let Some(mut canvas) = Canvas::new(frame_width, frame_height) else {
        report.fail_internally(format!(
            "no raster surface could be made at {frame_width}x{frame_height}"
        ));
        return Answer {
            picture: None,
            view: Some(view),
            image: None,
            report,
        };
    };

    let mut painter = Painter::new(&document, instant, (frame_width, frame_height));
    painter.paint(&mut canvas, &view);

    let encoding = if ask.png {
        Encoding::Png
    } else {
        Encoding::Jpeg
    };
    let scale = if ask.full { Scale::Full } else { Scale::Half };
    let Some(encoded) = canvas.encode(region.map(region_of), scale, encoding) else {
        // The region is already known to overlap the frame, so the only failure left is the
        // encoder's — which is Montagent failing, and is exit 70.
        report.fail_internally(format!(
            "the frame could not be encoded as {}",
            encoding.name()
        ));
        return Answer {
            picture: None,
            view: Some(view),
            image: None,
            report,
        };
    };

    let written = match &ask.out {
        Some(out) => match std::fs::write(out, &encoded.bytes) {
            Ok(()) => Some(out.display().to_string()),
            Err(e) => {
                report.could_not_write(out.display(), &e);
                None
            }
        },
        None => None,
    };

    let picture = Picture {
        at: instant,
        encoding: encoded.encoding.name().to_string(),
        scale: encoded.scale.name().to_string(),
        width: encoded.width,
        height: encoded.height,
        rasterized: Rasterized {
            width: frame_width,
            height: frame_height,
        },
        region,
        path: written,
        bytes: encoded.bytes.len(),
        painted: painter.painted,
        not_painted: painter.not_painted,
        painted_partially: painter.painted_partially,
        crossfades: painter.crossfades,
        sources: painter.sources,
        fonts: painter.fonts,
    };

    Answer {
        picture: Some(picture),
        view: Some(view),
        image: Some(encoded),
        report,
    }
}

/// The instant and the crop, or the one sentence saying why the flags ask for no frame.
fn request(ask: &Ask) -> Result<(i64, Option<Region>), String> {
    // Every instant is a legal question, including one before the project starts and one
    // after it ends — the answer there is the background and an empty caption, which is a
    // fact about the document rather than a malformed call. `query --at` takes the same
    // view, and the two must agree about what an instant is.
    let Some(instant) = ask.at else {
        return Err(
            "`frame` needs the instant to draw: `--at <t>`, in absolute milliseconds".into(),
        );
    };
    let crop = match &ask.crop {
        Some(spelling) => Some(region(spelling)?),
        None => None,
    };
    Ok((instant, crop))
}

/// `x,y,w,h` — four whole pixels, in frame space.
fn region(spelling: &str) -> Result<Region, String> {
    let numbers: Vec<&str> = spelling.split(',').map(str::trim).collect();
    let [x, y, width, height] = numbers.as_slice() else {
        return Err(format!(
            "`--crop {spelling}` is not a region: write `x,y,w,h` in whole frame-space pixels"
        ));
    };
    let parse = |text: &str, name: &str| -> Result<i64, String> {
        text.parse::<i64>().map_err(|_| {
            format!("`--crop {spelling}`: `{name}` is `{text}`, not a whole number of pixels")
        })
    };
    let (x, y) = (parse(x, "x")?, parse(y, "y")?);
    let (width, height) = (parse(width, "w")?, parse(height, "h")?);
    if width <= 0 || height <= 0 {
        return Err(format!(
            "`--crop {spelling}`: a region is {width}x{height}, which is no region at all"
        ));
    }
    Ok(Region {
        x,
        y,
        width,
        height,
    })
}

/// The region a `--crop` actually names: the asked-for rectangle intersected with the
/// frame, or `None` where it misses the frame entirely.
///
/// A region that reaches past an edge comes back as the part that is inside it, because
/// that is a picture and the caller can see what they got. A region with no overlap at all
/// is not a smaller picture, it is no picture, and the two are told apart here rather than
/// by a zero-size rectangle that would read like a measurement.
///
/// **`saturating_add`, and that is not defensiveness.** These are the only numbers in the
/// verb that come from the *caller* rather than from the document, so they are the only
/// ones nothing upstream has bounded: `--crop 9223372036854775807,0,10,10` overflows a
/// plain `+`. That is worse than it looks, because the two builds disagree — a debug build
/// panics, which [`crate::verbs`]' caller turns into ADR-0011's exit 70 ("Montagent broke"),
/// while a release build wraps to a negative edge and answers. One command, two behaviours,
/// neither of them the exit 3 the caller has earned. Saturating makes an edge past
/// `i64::MAX` mean what the caller wrote: past the frame.
fn clamp(crop: Region, frame_width: i64, frame_height: i64) -> Option<Rect> {
    let x = crop.x.clamp(0, frame_width);
    let y = crop.y.clamp(0, frame_height);
    let width = crop.x.saturating_add(crop.width).clamp(0, frame_width) - x;
    let height = crop.y.saturating_add(crop.height).clamp(0, frame_height) - y;
    (width > 0 && height > 0).then_some(Rect {
        x,
        y,
        width,
        height,
    })
}

/// The rasterizer's spelling of the same rectangle.
///
/// Two types for one concept, and the conversion is the seam rather than an oversight: the
/// answer's rectangle is [`Rect`], which is what every other frame-space rectangle in the
/// surface serialises as, and the rasterizer knows nothing of `serde` or of the format. A
/// shared type would mean one of those two facts stopped being true.
fn region_of(rect: Rect) -> Region {
    Region {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

/// One frame's painting pass, and the record of what it did.
///
/// `pub(crate)` because `render` paints every frame through this same pass — ADR-0021's
/// *"the picture `frame` shows is the picture `render` produces"* holds by construction
/// only while there is one painter. A `Painter` outlives one frame: [`Painter::begin`]
/// resets the per-frame record and keeps the font registry, the resolved `ffmpeg` and the
/// decoded stills, none of which change between two instants of one document.
pub(crate) struct Painter<'a> {
    document: &'a Loose,
    /// Every element in the document, with the identity the caption names it by — read
    /// once, because the picture walks the caption's list and would otherwise re-traverse
    /// the whole document per row.
    elements: Vec<(Named, &'a Value)>,
    instant: i64,
    frame: (i64, i64),
    project_dir: PathBuf,
    pub(crate) painted: Vec<String>,
    pub(crate) not_painted: Vec<NotPainted>,
    pub(crate) painted_partially: Vec<NotPainted>,
    /// Every crossfade running at this instant, in document order.
    pub(crate) crossfades: Vec<Crossfade>,
    /// Each bridged element's id and the factor its `opacity` is multiplied by — the one
    /// place a crossfade touches the picture.
    ///
    /// A list rather than a map, and a *multiplication* rather than a replacement, for the
    /// same reason: an element can be the `to` of one transition and the `from` of the
    /// next — a sequence of cross-fading clips, which ADR-0059 calls the "checkerboard" of
    /// tracks the model costs — and at the instant where those two windows meet it is
    /// fading in and out at once. Replacing would let whichever transition was read last
    /// win.
    fades: Vec<(String, f64)>,
    pub(crate) sources: Vec<String>,
    pub(crate) fonts: Vec<String>,
    /// Every declared chain this frame has opened, **one registry for the whole frame**.
    ///
    /// Shared rather than per element for two reasons that are the same reason: the
    /// fixture's four text elements all name `brand`, so a registry per element would read
    /// and parse the same `.otf` four times against a 500 ms cold budget (ADR-0021) — and
    /// the list of files actually opened, which is what makes #212's decoy test a test,
    /// should be the frame's list rather than the last element's.
    registry: montagent_text::Fonts,
    /// Resolved on demand, once: an all-image project must not need an `ffmpeg` on `PATH`
    /// to look at itself.
    ffmpeg: Option<Result<PathBuf, String>>,
    /// Every still decoded so far, by path. A `frame` decodes each once and gains nothing;
    /// a `render` paints the fixture's 6 MB PNGs on 1631 consecutive frames and would
    /// otherwise decode each of them 1631 times.
    stills: std::collections::HashMap<PathBuf, Raster>,
}

impl<'a> Painter<'a> {
    pub(crate) fn new(document: &'a Loose, instant: i64, frame: (i64, i64)) -> Painter<'a> {
        Painter {
            document,
            elements: document
                .elements_in_tracks()
                .map(|(track, element)| (Named::of(element, track), element))
                .collect(),
            instant,
            frame,
            project_dir: crate::checks::project_dir(document),
            painted: Vec::new(),
            not_painted: Vec::new(),
            painted_partially: Vec::new(),
            crossfades: Vec::new(),
            fades: Vec::new(),
            sources: Vec::new(),
            fonts: Vec::new(),
            registry: montagent_text::Fonts::new(),
            ffmpeg: None,
            stills: std::collections::HashMap::new(),
        }
    }

    /// Start another frame at `instant`, forgetting what the last one painted and keeping
    /// what it opened.
    ///
    /// `sources` and `fonts` are *not* cleared: they are the record of every file this
    /// painter has opened, and a still decoded on frame 0 and painted on frame 400 was
    /// opened once. A `render` reads them after its last frame as the whole run's list.
    pub(crate) fn begin(&mut self, instant: i64) {
        self.instant = instant;
        self.painted.clear();
        self.not_painted.clear();
        self.painted_partially.clear();
        self.crossfades.clear();
        self.fades.clear();
    }

    /// Paint the frame, in the caption's own order.
    ///
    /// **The order is the view's, not a second sort.** The `query --at` block beside the
    /// picture is painter's order — ascending resolved layer, back to front, document order
    /// within a tie, with an element whose layer did not resolve last (ADR-0060) — and the
    /// picture is built by walking that same list. A second ordering here, however
    /// carefully written, would be a second place draw order could be decided, and the one
    /// failure the caption exists to prevent is a defect attributed to the wrong element.
    pub(crate) fn paint(&mut self, canvas: &mut Canvas, view: &At) {
        self.resolve_crossfades();
        canvas.background(self.background());

        for present in &view.stack {
            let name = present
                .named
                .id
                .clone()
                .unwrap_or_else(|| "(element with no id)".to_string());
            let Some(element) = self.element_named(&present.named).copied() else {
                // A caption row with no element behind it means this pass and the view
                // disagree about the document, which is a bug in Montagent rather than a
                // fact about the project — but the picture still owes the row an answer.
                self.defer(
                    &name,
                    "this build could not find the element the caption names",
                );
                continue;
            };
            self.element(canvas, &name, element, present.source_offset);
        }
    }

    /// The project's `background`, or opaque black where it declares none.
    fn background(&self) -> Rgba {
        self.document
            .value()
            .get("background")
            .and_then(rgba)
            .unwrap_or(Rgba::BLACK)
    }

    /// Find the element the caption's row names.
    ///
    /// By `id` where there is one — ADR-0019 requires it to be unique — and by identity
    /// otherwise, which is what an id-less element has. An element that cannot be found is
    /// not silently skipped: the caption row exists, so the picture owes an explanation.
    fn element_named(&self, named: &Named) -> Option<&&'a Value> {
        self.elements
            .iter()
            .find(|(candidate, _)| candidate == named)
            .map(|(_, element)| element)
    }

    fn element(
        &mut self,
        canvas: &mut Canvas,
        name: &str,
        element: &Value,
        source_offset: Option<i64>,
    ) {
        let kind = element.get("type").and_then(Value::as_str);
        match kind {
            // No frame-space footprint at all. Not listed as unpainted: an audio element
            // that draws nothing is not a thing the picture is missing.
            Some("audio") => {}
            Some("text") => self.text(canvas, name, element),
            // No frame-space footprint of its own. ADR-0059 keeps a transition's job
            // "purely descriptive: name the pair, own the exact window" — what it does to
            // the picture is already in the two bridged elements' opacities, resolved in
            // `resolve_crossfades` before a single element was drawn. Listing it as
            // unpainted would say the picture is missing something it is not.
            Some("transition") => {}
            Some("rect") | Some("ellipse") => self.shape(canvas, name, element, kind),
            Some("image") | Some("video") => {
                self.raster(canvas, name, element, kind, source_offset)
            }
            Some(other) => self.defer(name, format!("this build draws no `{other}` element")),
            None => self.defer(name, "the element states no `type`"),
        }
    }

    /// The element's ordered `effects` list, in the rasterizer's spelling.
    ///
    /// **A member the format does not admit does not silently disappear.** The list is
    /// parsed through the model's own closed vocabulary, and an entry that does not parse
    /// — an invented `grayscale`, a `mask` carrying geometry parameters, a malformed
    /// colour — leaves the element painted *without that member* and puts a sentence on
    /// [`Picture::painted_partially`] saying so. `validate` names it as a schema error;
    /// this is the same fact at the surface the agent is looking at, because an agent that
    /// cannot tell "the blur is subtle" from "the blur was never applied" will chase the
    /// wrong defect.
    fn effects_of(&mut self, name: &str, element: &Value) -> Vec<Effect> {
        let Some(declared) = element.get("effects").and_then(Value::as_array) else {
            return Vec::new();
        };
        let mut effects = Vec::with_capacity(declared.len());
        for (i, value) in declared.iter().enumerate() {
            match serde_json::from_value::<model::Effect>(value.clone())
                .ok()
                .as_ref()
                .and_then(effect_of)
            {
                Some(effect) => effects.push(effect),
                None => self.partially(
                    name,
                    format!(
                        "`effects[{i}]` is not a member of the effect vocabulary, and is \
                         painted as though it were not there"
                    ),
                ),
            }
        }
        effects
    }

    /// Read every `transition` in the document and work out what each bridged element's
    /// opacity is multiplied by at this instant (ADR-0059).
    ///
    /// **The window is derived, not declared.** ADR-0059 requires a transition's
    /// `start`/`end` to equal the intersection of the two elements it bridges, and
    /// `E-TRANSITION-RANGE` is the check that says so — through the same
    /// [`crate::stack::Stack`] index this reads, rather than a second one. Where a
    /// document states a wider window the renderer cannot honour it: outside the
    /// intersection one of the two elements does not exist, so there is nothing to cross
    /// to. ADR-0007 settles what to do with a field like that — *"worse than no field"* —
    /// so the picture is drawn over the intersection and the drift stays `validate`'s to
    /// report.
    ///
    /// **The ramp is linear.** No ADR states a shape, and ADR-0059's whole argument for
    /// the element type is that *"a pair of opposite opacity ramps on two tracks"* is what
    /// a crossfade already was — those ramps being ADR-0012 keyframes, whose own default
    /// `ease` is what a document would have written by hand. Linear is also the only shape
    /// under which the two halves sum to a constant at every instant, which is what stops
    /// a crossfade dipping through the background halfway. Recorded here and raised at
    /// [#280](https://github.com/MBehtemam/Montagent/issues/280).
    fn resolve_crossfades(&mut self) {
        let stack = crate::stack::Stack::of(self.document);
        for (_, element) in self.document.elements_in_tracks() {
            if element.get("type").and_then(Value::as_str) != Some("transition") {
                continue;
            }
            match self.crossfade(&stack, element) {
                Ok(Some(fade)) => {
                    self.fades.push((fade.from.clone(), 1.0 - fade.progress));
                    self.fades.push((fade.to.clone(), fade.progress));
                    self.crossfades.push(fade);
                }
                // Not running at this instant, which is not a thing the picture is
                // missing — a transition is only ever doing something inside its own
                // window.
                Ok(None) => {}
                // Running, or claiming to be, and unusable. Named on the same rule every
                // other element in this verb is: an agent that cannot tell "the fade has
                // not started" from "the fade could not be read" will go looking for the
                // defect in the wrong place.
                Err(reason) => {
                    if self.present(element) {
                        let name = name_of(element);
                        self.defer(&name, reason);
                    }
                }
            }
        }
    }

    /// One transition's contribution at this instant: the crossfade it is running, nothing
    /// (because the instant is outside its window), or the reason it could not be read.
    fn crossfade(
        &self,
        stack: &crate::stack::Stack<'_>,
        element: &Value,
    ) -> Result<Option<Crossfade>, String> {
        // `crossfade` is the whole v1 vocabulary (ADR-0059): "wipe, slide and push are
        // deferred", and freezing a closed-vocabulary member on a guess is the trap that
        // ADR avoided. A `kind` the format does not have is `validate`'s schema error, and
        // nothing here invents a ramp for it.
        match element.get("kind").and_then(Value::as_str) {
            Some("crossfade") => {}
            Some(other) => {
                return Err(format!(
                    "`crossfade` is the whole of v1's transition vocabulary, and this one \
                     is a `{other}`"
                ));
            }
            None => return Err("it states no `kind`".to_string()),
        }
        let (Some(from), Some(to)) = (
            element.get("from").and_then(Value::as_str),
            element.get("to").and_then(Value::as_str),
        ) else {
            return Err("it does not name both the elements it bridges".to_string());
        };
        // A `from`/`to` naming nothing in the document is a dangling reference, which is
        // `validate`'s to classify — but it is also the reason this frame shows no fade,
        // so it is said here too.
        let (Some(from_range), Some(to_range)) = (
            stack.placement(from).and_then(|placement| placement.range),
            stack.placement(to).and_then(|placement| placement.range),
        ) else {
            return Err(format!(
                "`{from}` and `{to}` do not both resolve to an element with a range"
            ));
        };

        let start = from_range.start.max(to_range.start);
        let end = from_range.end.min(to_range.end);
        if end <= start {
            return Err(format!(
                "`{from}` and `{to}` share no instant, so there is no window to cross over"
            ));
        }
        if self.instant < start || self.instant >= end {
            return Ok(None);
        }
        Ok(Some(Crossfade {
            element: crate::checks::subject_of(element.get("id").and_then(Value::as_str)),
            from: from.to_string(),
            to: to.to_string(),
            start,
            end,
            progress: (self.instant - start) as f64 / (end - start) as f64,
        }))
    }

    /// Does this element's own declared range contain the instant?
    ///
    /// Asked only of a transition, and only to decide whether an unusable one is worth a
    /// sentence. Every other element reaches [`Painter::element`] through the caption's
    /// stack, which has already answered this.
    fn present(&self, element: &Value) -> bool {
        let (Some(start), Some(end)) = (
            element.get("start").and_then(Value::as_i64),
            element.get("end").and_then(Value::as_i64),
        ) else {
            // A transition with no range of its own cannot be placed in time at all, and
            // saying so is more use than silence.
            return true;
        };
        self.instant >= start && self.instant < end
    }

    /// What this element's `opacity` is multiplied by, from every crossfade it is bridged
    /// by — `1.0` for an element no transition names.
    fn fade(&self, element: &Value) -> f64 {
        let Some(id) = element.get("id").and_then(Value::as_str) else {
            return 1.0;
        };
        self.fades
            .iter()
            .filter(|(named, _)| named == id)
            .map(|(_, factor)| factor)
            .product()
    }

    fn defer(&mut self, name: &str, reason: impl Into<String>) {
        self.not_painted.push(NotPainted {
            element: name.to_string(),
            reason: reason.into(),
        });
    }

    /// Drawn, but not in full — the other list, kept apart for the reason
    /// [`Picture::painted_partially`] gives.
    fn partially(&mut self, name: &str, reason: impl Into<String>) {
        self.painted_partially.push(NotPainted {
            element: name.to_string(),
            reason: reason.into(),
        });
    }

    /// `rect` and `ellipse` — fill, inside stroke and `radius` (ADR-0014).
    fn shape(&mut self, canvas: &mut Canvas, name: &str, element: &Value, kind: Option<&str>) {
        let Some(extent) = self.extent(element) else {
            self.defer(name, NO_EXTENT);
            return;
        };
        let paint = Fill {
            fill: element.get("fill").and_then(rgba),
            stroke: element.get("stroke").and_then(rgba),
            stroke_width: element
                .get("stroke_width")
                .and_then(Value::as_i64)
                .unwrap_or(0) as f64,
        };
        if paint.fill.is_none() && (paint.stroke.is_none() || paint.stroke_width <= 0.0) {
            // ADR-0014: a shape with neither fill nor stroke is a schema error naming both,
            // "because an element that deliberately renders nothing and an element that
            // forgot its paint must not look alike". Saying so here is the same rule at the
            // surface an agent is looking at.
            self.defer(
                name,
                "it carries neither a `fill` nor a `stroke` to paint with",
            );
            return;
        }
        let shape = match kind {
            Some("ellipse") => Shape::Ellipse,
            _ => Shape::Rect {
                radius: element.get("radius").and_then(Value::as_i64).unwrap_or(0) as f64,
            },
        };
        let effects = self.effects_of(name, element);
        canvas.shape(
            shape,
            extent,
            &self.transform(element),
            &paint,
            None,
            &effects,
        );
        self.painted.push(name.to_string());
    }

    /// A raster element — `image` or `video` — resampled through the declared
    /// `width`/`height` and cropped by `clip`.
    ///
    /// **One function for both**, because they are one drawing: ADR-0013 settled that a
    /// source is resampled to exactly the declared rect, and from that point on a decoded
    /// still and a decoded video frame are the same pixels placed the same way. The only
    /// thing that differs is how the bytes are got, which is [`Painter::decoded`] — so the
    /// transform, the aperture, the remote refusal and the record of what was opened are
    /// each written once rather than twice with a chance to drift.
    fn raster(
        &mut self,
        canvas: &mut Canvas,
        name: &str,
        element: &Value,
        kind: Option<&str>,
        source_offset: Option<i64>,
    ) {
        let Some(extent) = self.extent(element) else {
            self.defer(name, NO_EXTENT);
            return;
        };
        let Some(source) = element.get("source").and_then(Value::as_str) else {
            self.defer(name, "it states no `source`");
            return;
        };
        let path = match Source::resolve(source, &self.project_dir) {
            Source::Local(path) => path,
            // ADR-0056 keeps the remote half to `probe`'s session, which fetches ranges
            // rather than whole files. Drawing one would be the unsolicited network call
            // this surface promises not to make.
            Source::Remote(url) => {
                self.defer(
                    name,
                    format!("`{url}` is remote; `frame` draws local sources"),
                );
                return;
            }
        };

        let raster = match self.decoded(&path, kind, extent, source_offset) {
            Ok(raster) => raster,
            Err(reason) => {
                self.defer(name, reason);
                return;
            }
        };
        let opened = path.display().to_string();
        if !self.sources.contains(&opened) {
            self.sources.push(opened);
        }
        let effects = self.effects_of(name, element);
        canvas.raster(
            &raster,
            extent,
            &self.transform(element),
            self.clip(element),
            &effects,
        );
        self.painted.push(name.to_string());
    }

    /// The pixels, however this element's type gets them: a still off the disk, or one
    /// frame out of the `ffmpeg` Montagent spawns.
    fn decoded(
        &mut self,
        path: &FilePath,
        kind: Option<&str>,
        extent: Extent,
        source_offset: Option<i64>,
    ) -> Result<Raster, String> {
        if kind != Some("video") {
            if let Some(still) = self.stills.get(path) {
                return Ok(still.clone());
            }
            let bytes = std::fs::read(path)
                .map_err(|e| format!("{} could not be read: {e}", path.display()))?;
            let still = Raster::decode(&bytes)
                .ok_or_else(|| format!("{} did not decode as an image", path.display()))?;
            self.stills.insert(path.to_path_buf(), still.clone());
            return Ok(still);
        }

        // The caption already said *why* an offset did not resolve —
        // `source_offset_unresolved` carries that sentence — so this one names the
        // consequence rather than repeating it.
        let offset = source_offset
            .ok_or("its offset into the source did not resolve; the caption says why")?;
        let ffmpeg = self.ffmpeg()?;
        // Decoded straight to the declared box: ADR-0013 settled that a source is resampled
        // to exactly `width`x`height`, so asking `ffmpeg` for that size is the resample
        // rather than a second one on top of it.
        let decoded = montagent_render::decode::frame_at(
            &ffmpeg,
            &path.to_string_lossy(),
            offset,
            extent.width as u32,
            extent.height as u32,
        )?;
        Raster::from_rgba(&decoded.rgba, decoded.width, decoded.height).ok_or_else(|| {
            format!(
                "{}'s decoded frame was not the size asked for",
                path.display()
            )
        })
    }

    /// `ffmpeg`, resolved once per run and only where a video element needs one.
    fn ffmpeg(&mut self) -> Result<PathBuf, String> {
        if self.ffmpeg.is_none() {
            self.ffmpeg = Some(
                tools::resolve()
                    .map(|tools| tools.ffmpeg)
                    .map_err(|missing| missing.reason()),
            );
        }
        self.ffmpeg.clone().expect("just resolved")
    }

    /// Register one text element's declared chains, and record every file that was
    /// opened.
    ///
    /// `false` where a chain did not resolve — and the element is already on
    /// [`Picture::not_painted`] with the reason by then. Every key the element names is
    /// tried before answering, so a project with two broken chains reports both rather
    /// than the first.
    fn register_fonts(&mut self, name: &str, element: &Value) -> bool {
        let declared = element
            .get("font")
            .and_then(Value::as_str)
            .map(str::to_string);
        let mut resolved = true;
        for key in declared
            .into_iter()
            .chain(crate::verbs::measure::Measurable::keys(element))
        {
            if let Err(e) = crate::verbs::measure::register(self.document, &key, &mut self.registry)
            {
                self.defer(name, format!("its font chain did not resolve: {e}"));
                resolved = false;
            }
        }
        let opened: Vec<String> = self
            .registry
            .opened()
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        for path in opened {
            if !self.fonts.contains(&path) {
                self.fonts.push(path);
            }
        }
        resolved
    }

    /// Paint one text element's glyphs (#213).
    ///
    /// **Everything about *where* the text goes comes from the same engine `measure`
    /// answers with**, through [`montagent_text::place`], which reads the glyphs off the
    /// very layouts the measurement was derived from. So there is one line partition
    /// (ADR-0008), one slot rule (ADR-0007) and one baseline formula (ADR-0029) in
    /// Montagent, and `measure`'s answer is a statement about the picture rather than a
    /// parallel derivation that happens to agree.
    ///
    /// **`y` and `origin` are deliberately *not* handed to the engine.** The placement
    /// comes back in the block's own coordinates and the canvas applies ADR-0012's
    /// transform to it, exactly as it does for a rect or an image — so the pivot rule has
    /// one implementation. Passing the element's `y` as well would place the block twice.
    fn text(&mut self, canvas: &mut Canvas, name: &str, element: &Value) {
        // The chain is registered first and unconditionally, because this is where
        // ADR-0007's "the renderer opens nothing outside the declared chain" is enforced
        // (#212) — an element that will not measure must still not be able to reach a
        // font the document does not name.
        if !self.register_fonts(name, element) {
            return;
        }
        let style = match crate::verbs::measure::Measurable::of(element) {
            Ok(style) => style,
            Err(reason) => {
                self.defer(name, reason);
                return;
            }
        };

        let runs = crate::verbs::measure::runs_of(element);
        let placement = montagent_text::place(
            &mut self.registry,
            &montagent_text::Spec {
                runs: &runs,
                font: &style.asked.font,
                size: style.asked.size,
                line_height_tenths: style.asked.line_height_tenths,
                stroke_width: style.asked.stroke_width,
                // The block's own frame: the canvas places it.
                y: 0,
                vertical_origin: montagent_text::VerticalOrigin::Top,
                align: crate::verbs::measure::align_of(element),
            },
        );
        let placement = match placement {
            Ok(placement) => placement,
            // Unreachable while every key above registered, and reported rather than
            // `expect`ed: "unreachable" is a claim about this function's control flow that
            // a later edit can falsify in silence.
            Err(e) => {
                self.defer(name, format!("its text could not be laid out: {e}"));
                return;
            }
        };

        let paints = paints_of(element, &runs, self.instant);
        let glyphs: Vec<Glyph> = placement
            .glyphs
            .iter()
            .map(|glyph| Glyph {
                x: glyph.x,
                y: glyph.y,
                outline: glyph.outline,
                // A glyph whose run index has no paint is unreachable — `paints_of` maps
                // the same array `montagent_text::place` indexed into — and is painted in
                // the default ink rather than skipped, so a future divergence shows up as
                // a black letter rather than as a hole.
                paint: paints.get(glyph.run).copied().unwrap_or(Fill {
                    fill: Some(DEFAULT_INK),
                    stroke: None,
                    stroke_width: 0.0,
                }),
            })
            .collect();
        let outlines: Vec<Vec<PathEl>> = placement
            .outlines
            .iter()
            .map(|outline| outline.iter().copied().map(path_element).collect())
            .collect();

        let effects = self.effects_of(name, element);
        canvas.text(
            &glyphs,
            &outlines,
            Extent {
                width: placement.width,
                height: placement.height,
            },
            &self.transform(element),
            None,
            &effects,
        );
        self.painted.push(name.to_string());

        // Drawn, and then said. A `dir` override reaches neither the measurement nor the
        // layout yet, which changes what the picture shows, so an agent comparing this
        // frame against the document is owed the sentence.
        if runs_with(element, "dir") {
            self.partially(
                name,
                "a run's `dir` override is not applied yet; the line's base direction is \
                 the one the text itself implies",
            );
        }
    }

    /// The declared box, before `scale`.
    fn extent(&self, element: &Value) -> Option<Extent> {
        let width = element.get("width").and_then(Value::as_i64)?;
        let height = element.get("height").and_then(Value::as_i64)?;
        (width > 0 && height > 0).then_some(Extent {
            width: width as f64,
            height: height as f64,
        })
    }

    /// The resolved transform, through the same reading the `query --at` block's geometry
    /// uses — ADR-0012's defaults included, since this is *where the element actually is*
    /// rather than *what the document declares*.
    fn transform(&self, element: &Value) -> Transform {
        let (frame_width, frame_height) = self.frame;
        let origin = match element.get("origin") {
            None | Some(Value::Null) => Origin::Center,
            Some(value) => serde_json::from_value(value.clone()).unwrap_or(Origin::Center),
        };
        Transform {
            x: geometry::number::<i64>(element, "x", self.instant, frame_width as f64 / 2.0),
            y: geometry::number::<i64>(element, "y", self.instant, frame_height as f64 / 2.0),
            origin: geometry::origin_fraction(origin),
            scale: {
                let [sx, sy] =
                    geometry::number::<[f64; 2]>(element, "scale", self.instant, [1.0, 1.0]);
                (sx, sy)
            },
            rotation: geometry::number::<f64>(element, "rotation", self.instant, 0.0),
            // The declared opacity, times whatever crossfade this element is bridged by.
            // Multiplied rather than replaced: a crossfade is a ramp *on* what the
            // document says, so an element already keyframed to 0.5 fades from 0.5 rather
            // than jumping to 1 to start.
            opacity: geometry::number::<f64>(element, "opacity", self.instant, 1.0)
                * self.fade(element),
        }
    }

    /// `clip`, the static frame-space aperture (ADR-0025) — `image` and `video` only.
    fn clip(&self, element: &Value) -> Option<Region> {
        let clip = element.get("clip")?.as_array()?;
        if clip.len() != 4 {
            return None;
        }
        Some(Region {
            x: clip[0].as_i64()?,
            y: clip[1].as_i64()?,
            width: clip[2].as_i64()?,
            height: clip[3].as_i64()?,
        })
    }
}

/// The ink an element that states no `color` is painted in.
///
/// **A recorded reading, not an ADR's.** `color` is optional on a text element, so an
/// absent one is legal and ADR-0030 makes it *"give me whatever the default is"* — which
/// means there is a default and this is it. Black, and not a colour picked to contrast
/// with the background: it is what every rasterizer in the reference class paints with
/// when nobody said, it is the same constant an absent `background` already resolves to
/// ([`Rgba::BLACK`]), and a default that read the background would make one element's
/// colour depend on another element's field. Raised at
/// [#277](https://github.com/MBehtemam/Montagent/issues/277).
const DEFAULT_INK: Rgba = Rgba::BLACK;

/// Each run's resolved paint, in the order the runs are written — the element's base with
/// the run's deltas over it (ADR-0007's base-plus-deltas model, ADR-0014's
/// run-addressable stroke).
///
/// [`Fill`] rather than a type of this module's own: it is the same three fields a shape
/// is painted with, and ADR-0014 makes them one vocabulary. What differs between the two
/// is which side of the outline the stroke falls on, and that is the rasterizer's rule
/// rather than the paint's.
fn paints_of(element: &Value, runs: &[montagent_text::Run<'_>], instant: i64) -> Vec<Fill> {
    let base = Fill {
        fill: Some(element.get("color").and_then(rgba).unwrap_or(DEFAULT_INK)),
        stroke: element.get("stroke").and_then(rgba),
        stroke_width: element
            .get("stroke_width")
            .and_then(Value::as_i64)
            .unwrap_or(0) as f64,
    };
    crate::verbs::measure::runs_array(element)
        .iter()
        .enumerate()
        .map(|(i, run)| {
            let unconditional = Fill {
                // A delta that is absent is a delta that was not made (ADR-0007), so the
                // base stands wherever the run is silent — and a `stroke_width` the run
                // *does* state is the one the engine measured with, which is why it is
                // read back out of the parsed run rather than off the JSON a second time.
                fill: run.get("color").and_then(rgba).or(base.fill),
                stroke: run.get("stroke").and_then(rgba).or(base.stroke),
                stroke_width: runs
                    .get(i)
                    .and_then(|run| run.stroke_width)
                    .map(|width| width as f64)
                    .unwrap_or(base.stroke_width),
            };
            match highlight_at(run, instant) {
                Some(window) => Fill {
                    // The window's own deltas over the run's, on exactly the rule the
                    // run's sit over the element's: absent is not a delta.
                    fill: window
                        .color
                        .as_ref()
                        .and_then(rgba_of)
                        .or(unconditional.fill),
                    stroke: window
                        .stroke
                        .as_ref()
                        .and_then(rgba_of)
                        .or(unconditional.stroke),
                    stroke_width: window
                        .stroke_width
                        .map(|width| width as f64)
                        .unwrap_or(unconditional.stroke_width),
                },
                None => unconditional,
            }
        })
        .collect()
}

/// This run's `highlight` window, if the instant is inside it (ADR-0048).
///
/// **Half-open, like every other range in the format** — `[start, end)`, ADR-0005 — so the
/// instant a word lights up is inside its window and the instant it goes out is not. A
/// closed range would make two adjacent words both lit for one instant at every boundary,
/// which is the one thing a karaoke line must never show.
///
/// **A window whose fields the format does not admit is not a window.** It is read through
/// the model's own [`crate::model::Highlight`], whose `deny_unknown_fields` is what keeps
/// the delta to the run-addressable paint fields ADR-0048 names — so a `size` or a `font`
/// smuggled into a highlight cannot restyle a word mid-line into a layout `measure` never
/// saw.
///
/// **A highlight is paint and only paint, which diverges from ADR-0048's own sentence.**
/// That ADR says the delta's *"shape matches the run's existing unconditional delta
/// fields"*, and `stroke_width` is one of them — but on an unconditional run that key
/// changes *layout*, because ADR-0014 puts a text stroke outside the contour and
/// `montagent-text` grows a line's extent by `2 × stroke_width`. Letting it through here
/// would make a word jump sideways at the instant it lit up, and would make `measure`'s
/// answer — taken once, at authoring time, and the number the author sized the box
/// against — stop describing the frame for the length of every window. So the element is
/// laid out from the runs' unconditional style and the window changes what is painted
/// into the slots, never where the slots are. Raised at
/// [#280](https://github.com/MBehtemam/Montagent/issues/280), asserted by
/// `tests/effects.rs::a_highlight_moves_no_glyph`.
fn highlight_at(run: &Value, instant: i64) -> Option<crate::model::Highlight> {
    let window: crate::model::Highlight =
        serde_json::from_value(run.get("highlight")?.clone()).ok()?;
    (instant >= window.start && instant < window.end).then_some(window)
}

/// What the answer calls this element — its `id`, or the one phrase an id-less element is
/// known by.
///
/// The same string [`Painter::paint`] takes off the caption's row, so a transition named
/// here and an element named there are named the same way. The caption's rows come from
/// [`Named`], which reads the very same field.
fn name_of(element: &Value) -> String {
    element
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| "(element with no id)".to_string())
}

/// Does any run carry this field?
fn runs_with(element: &Value, key: &str) -> bool {
    crate::verbs::measure::runs_array(element)
        .iter()
        .any(|run| run.get(key).is_some_and(|value| !value.is_null()))
}

/// The rasterizer's spelling of one outline segment.
///
/// The `Rect`/`Region` seam again, and for the same reason: ADR-0010 keeps the text stack
/// *beside* the rasterizer, so `montagent-render` may not name a `montagent-text` type. This
/// is the crate that depends on both, so the conversion belongs here.
fn path_element(element: montagent_text::PathEl) -> PathEl {
    match element {
        montagent_text::PathEl::Move(x, y) => PathEl::Move(x, y),
        montagent_text::PathEl::Line(x, y) => PathEl::Line(x, y),
        montagent_text::PathEl::Quad(cx, cy, x, y) => PathEl::Quad(cx, cy, x, y),
        montagent_text::PathEl::Cubic(ax, ay, bx, by, x, y) => PathEl::Cubic(ax, ay, bx, by, x, y),
        montagent_text::PathEl::Close => PathEl::Close,
    }
}

/// One `#RRGGBB`/`#RRGGBBAA` colour, through the format's own spelling rule.
///
/// [`Colour`]'s deserializer is the one authority on what a colour is — uppercase hex, no
/// shorthand, no `#RRGGBBFF`, no CSS names (ADR-0014) — so a string it refuses paints
/// nothing here and is `validate`'s to name. Re-parsing the spelling loosely would be a
/// second, more permissive answer to *"is this a colour"*, and the renderer would then draw
/// things the document's own rules say are not there.
fn rgba(value: &Value) -> Option<Rgba> {
    rgba_of(&serde_json::from_value::<Colour>(value.clone()).ok()?)
}

/// The same conversion, from a colour that has already been through that deserializer —
/// which is where an `effects` member's colour arrives, the whole list having been parsed
/// as the model's own type.
pub(crate) fn rgba_of(colour: &Colour) -> Option<Rgba> {
    let body = colour.as_str().strip_prefix('#')?;
    let byte = |at: usize| u8::from_str_radix(body.get(at..at + 2)?, 16).ok();
    Some(Rgba([
        byte(0)?,
        byte(2)?,
        byte(4)?,
        if body.len() == 8 { byte(6)? } else { 0xFF },
    ]))
}

/// One `effects` member in the rasterizer's spelling, or `None` for a colour this
/// document's own rules say is not a colour.
///
/// **The model's enum is the one authority on what an effect is.** The list is
/// deserialized through [`crate::model::Effect`], whose `deny_unknown_fields` and
/// lowercase `name` tag are the closed vocabulary ADR-0040 and ADR-0049 fixed — so the
/// renderer cannot paint a `grayscale`, or a `mask` with geometry parameters, that the
/// format says does not exist. A second, looser reading here would be a second answer to
/// *"what effects are there"*.
fn effect_of(declared: &model::Effect) -> Option<Effect> {
    Some(match declared {
        model::Effect::Blur { radius } => Effect::Blur { radius: *radius },
        model::Effect::Shadow {
            dx,
            dy,
            radius,
            color,
            opacity,
        } => Effect::Shadow {
            dx: *dx,
            dy: *dy,
            radius: *radius,
            colour: rgba_of(color)?,
            opacity: *opacity,
        },
        model::Effect::Mask {
            shape,
            x,
            y,
            width,
            height,
            radius,
        } => Effect::Mask {
            shape: match shape {
                model::MaskShape::Circle => MaskShape::Circle,
                model::MaskShape::Rect => MaskShape::Rect,
                model::MaskShape::Ellipse => MaskShape::Ellipse,
            },
            // ADR-0084's all-or-none rect. The model refuses a partial tuple on the way
            // in, so the only two shapes that reach here are all four and none — and
            // `None` is the identity value the rasterizer resolves to the element's own
            // rect, not a missing answer it has to guess at.
            rect: match (x, y, width, height) {
                (Some(x), Some(y), Some(width), Some(height)) => Some(MaskRect {
                    x: *x as f64,
                    y: *y as f64,
                    width: *width as f64,
                    height: *height as f64,
                }),
                _ => None,
            },
            radius: radius.unwrap_or(0) as f64,
        },
        model::Effect::Tint { color, amount } => Effect::Tint {
            colour: rgba_of(color)?,
            amount: *amount,
        },
        model::Effect::Saturation { amount } => Effect::Saturation { amount: *amount },
        model::Effect::Brightness { amount } => Effect::Brightness { amount: *amount },
        model::Effect::Contrast { amount } => Effect::Contrast { amount: *amount },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crop_is_four_whole_pixels_or_a_sentence_saying_what_is_wrong() {
        assert_eq!(
            region("10,20,30,40"),
            Ok(Region {
                x: 10,
                y: 20,
                width: 30,
                height: 40
            })
        );
        // Whitespace is transport, not content.
        assert_eq!(
            region(" 10 , 20 , 30 , 40 "),
            Ok(Region {
                x: 10,
                y: 20,
                width: 30,
                height: 40
            })
        );
        assert!(region("10,20,30").is_err());
        assert!(region("10,20,30,40,50").is_err());
        assert!(region("10,20,30,0").is_err());
        assert!(region("10,20,-30,40").is_err());
        assert!(region("a,20,30,40").is_err());
    }

    #[test]
    fn a_colour_is_read_through_the_formats_own_spelling_rule() {
        assert_eq!(
            rgba(&Value::String("#1E344C".into())),
            Some(Rgba([0x1E, 0x34, 0x4C, 0xFF]))
        );
        assert_eq!(
            rgba(&Value::String("#1E344C80".into())),
            Some(Rgba([0x1E, 0x34, 0x4C, 0x80]))
        );
        // ADR-0014's four refusals, all of which must stay refusals here.
        assert_eq!(rgba(&Value::String("#1e344c".into())), None);
        assert_eq!(rgba(&Value::String("#FFF".into())), None);
        assert_eq!(rgba(&Value::String("#1E344CFF".into())), None);
        assert_eq!(rgba(&Value::String("red".into())), None);
    }

    #[test]
    fn a_crop_reaching_outside_the_frame_is_reported_as_the_part_inside_it() {
        let asked = Region {
            x: 900,
            y: 1800,
            width: 400,
            height: 400,
        };
        assert_eq!(
            clamp(asked, 1080, 1920),
            Some(Rect {
                x: 900,
                y: 1800,
                width: 180,
                height: 120
            })
        );
    }

    #[test]
    fn an_edge_past_i64_max_means_past_the_frame_rather_than_a_panic() {
        // The only numbers in the verb that come from the caller rather than the document,
        // and so the only ones nothing upstream has bounded. A plain `+` here panics in a
        // debug build and wraps in a release one — one command, two behaviours.
        assert_eq!(
            clamp(
                Region {
                    x: 0,
                    y: 0,
                    width: i64::MAX,
                    height: i64::MAX,
                },
                1080,
                1920
            ),
            Some(Rect {
                x: 0,
                y: 0,
                width: 1080,
                height: 1920
            }),
            "a region wider than the world is the whole frame"
        );
        assert_eq!(
            clamp(
                Region {
                    x: i64::MAX,
                    y: 0,
                    width: 10,
                    height: 10,
                },
                1080,
                1920
            ),
            None,
            "and one that starts past the world misses the frame"
        );
        assert_eq!(
            clamp(
                Region {
                    x: i64::MIN,
                    y: i64::MIN,
                    width: 10,
                    height: 10,
                },
                1080,
                1920
            ),
            None,
        );
    }

    #[test]
    fn a_crop_that_misses_the_frame_names_no_region_at_all() {
        // Not a zero-size rectangle: "no picture" and "a very small picture" are two
        // different answers, and one of them reads like a measurement.
        for asked in [
            Region {
                x: 4000,
                y: 4000,
                width: 100,
                height: 100,
            },
            Region {
                x: -400,
                y: 0,
                width: 100,
                height: 100,
            },
        ] {
            assert_eq!(clamp(asked, 1080, 1920), None);
        }
    }
}
