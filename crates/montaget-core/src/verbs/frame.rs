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
//! **Glyphs are not painted yet** — that is
//! [#213](https://github.com/MBehtemam/Montaget/issues/213), *"text drawing, and the first
//! falsification"* — and neither are effects, colour filters, transitions or highlight
//! ([#214](https://github.com/MBehtemam/Montaget/issues/214)). Every element this build
//! cannot paint is listed in the answer with the reason, because an agent that cannot tell
//! "not there" from "not drawn yet" will chase the wrong defect.
//!
//! ## The surface, and what of it no ADR states
//!
//! ADR-0011 fixes the default encoding, the default scale, `--crop x,y,w,h` and the
//! unconditional caption. It does not say where the CLI puts the bytes, what JPEG quality
//! is, or what a project that declares no `background` is painted on. Those are this
//! ticket's, recorded here and raised for ratification rather than left to be discovered
//! from the code: the CLI takes a required `--out` rather than inventing a filename in
//! somebody's project directory; quality is
//! [`montaget_render::canvas::JPEG_QUALITY`]; and an absent `background` is opaque black
//! (argued at [`montaget_render::canvas::Rgba::BLACK`]).

use std::path::{Path as FilePath, PathBuf};

use serde::Serialize;
use serde_json::Value;

use montaget_render::canvas::{
    Canvas, Encoded, Encoding, Extent, Fill, Raster, Region, Rgba, Scale, Shape, Transform,
};

use crate::finding::Finding;
use crate::media::{Source, tools};
use crate::model::{Colour, Origin};
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
    /// The requested region, clipped to the frame, or `null` for the whole frame.
    ///
    /// The same [`Rect`] every other frame-space rectangle in the surface is written as —
    /// `--crop` asks the same kind of question a crop rectangle and `NOT COVERED` answer,
    /// and a second rectangle shape would be a second thing to parse.
    pub crop: Option<Rect>,
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

/// One present element the picture does not show, and the reason.
#[derive(Debug, Clone, Serialize)]
pub struct NotPainted {
    pub element: String,
    pub reason: String,
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

    let Some((frame_width, frame_height)) = frame_dimensions(&document) else {
        report.fail_internally(
            "the project states no legal `frame`, so there is no surface to draw on — \
             `validate` names the field",
        );
        return Answer {
            picture: None,
            view: Some(view),
            image: None,
            report,
        };
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
    let Some(encoded) = canvas.encode(crop, scale, encoding) else {
        report.fail_internally(match crop {
            Some(crop) => format!(
                "`--crop {},{},{},{}` does not overlap a {frame_width}x{frame_height} frame",
                crop.x, crop.y, crop.width, crop.height
            ),
            None => format!("the frame could not be encoded as {}", encoding.name()),
        });
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
        crop: crop.map(|crop| clamp(crop, frame_width, frame_height)),
        path: written,
        bytes: encoded.bytes.len(),
        painted: painter.painted,
        not_painted: painter.not_painted,
        painted_partially: painter.painted_partially,
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

/// The requested region as the answer reports it: what was actually returned, which is the
/// asked-for region intersected with the frame.
fn clamp(crop: Region, frame_width: i64, frame_height: i64) -> Rect {
    let x = crop.x.clamp(0, frame_width);
    let y = crop.y.clamp(0, frame_height);
    Rect {
        x,
        y,
        width: (crop.x + crop.width).clamp(0, frame_width) - x,
        height: (crop.y + crop.height).clamp(0, frame_height) - y,
    }
}

/// The project's own `frame`, or `None` where it is missing or malformed.
fn frame_dimensions(document: &Loose) -> Option<(i64, i64)> {
    let frame = document.value().get("frame")?;
    Some((
        frame.get("width")?.as_i64()?,
        frame.get("height")?.as_i64()?,
    ))
}

/// One frame's painting pass, and the record of what it did.
struct Painter<'a> {
    document: &'a Loose,
    /// Every element in the document, with the identity the caption names it by — read
    /// once, because the picture walks the caption's list and would otherwise re-traverse
    /// the whole document per row.
    elements: Vec<(Named, &'a Value)>,
    instant: i64,
    frame: (i64, i64),
    project_dir: PathBuf,
    painted: Vec<String>,
    not_painted: Vec<NotPainted>,
    painted_partially: Vec<NotPainted>,
    sources: Vec<String>,
    fonts: Vec<String>,
    /// Resolved on demand, once: an all-image project must not need an `ffmpeg` on `PATH`
    /// to look at itself.
    ffmpeg: Option<Result<PathBuf, String>>,
}

impl<'a> Painter<'a> {
    fn new(document: &'a Loose, instant: i64, frame: (i64, i64)) -> Painter<'a> {
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
            sources: Vec::new(),
            fonts: Vec::new(),
            ffmpeg: None,
        }
    }

    /// Paint the frame, in the caption's own order.
    ///
    /// **The order is the view's, not a second sort.** The `query --at` block beside the
    /// picture is painter's order — ascending resolved layer, back to front, document order
    /// within a tie, with an element whose layer did not resolve last (ADR-0060) — and the
    /// picture is built by walking that same list. A second ordering here, however
    /// carefully written, would be a second place draw order could be decided, and the one
    /// failure the caption exists to prevent is a defect attributed to the wrong element.
    fn paint(&mut self, canvas: &mut Canvas, view: &At) {
        canvas.background(self.background());

        for present in &view.stack {
            let name = present
                .named
                .id
                .clone()
                .unwrap_or_else(|| "(element with no id)".to_string());
            let Some(element) = self.element_named(&present.named).copied() else {
                // A caption row with no element behind it means this pass and the view
                // disagree about the document, which is a bug in Montaget rather than a
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
            Some("text") => {
                // The chain is registered even though no glyph is drawn yet, because this
                // is where ADR-0007's "the renderer opens nothing outside the declared
                // chain" is enforced (#212) — and #213 paints through the registry this
                // call has already filled.
                self.register_fonts(name, element);
                self.defer(name, "text drawing is #213");
            }
            Some("transition") => self.defer(name, "transitions are #214"),
            Some("rect") | Some("ellipse") => self.shape(canvas, name, element, kind),
            Some("image") => self.image(canvas, name, element),
            Some("video") => self.video(canvas, name, element, source_offset),
            Some(other) => self.defer(name, format!("this build draws no `{other}` element")),
            None => self.defer(name, "the element states no `type`"),
        }
        // Drawn, and then said: an element carrying effects is painted without them rather
        // than not painted at all, and the agent is told which half it is looking at.
        if element
            .get("effects")
            .and_then(Value::as_array)
            .is_some_and(|effects| !effects.is_empty())
            && self.painted.iter().any(|painted| painted == name)
        {
            self.painted_partially.push(NotPainted {
                element: name.to_string(),
                reason: "its `effects` are #214, and it is painted without them".to_string(),
            });
        }
    }

    fn defer(&mut self, name: &str, reason: impl Into<String>) {
        self.not_painted.push(NotPainted {
            element: name.to_string(),
            reason: reason.into(),
        });
    }

    /// `rect` and `ellipse` — fill, inside stroke and `radius` (ADR-0014).
    fn shape(&mut self, canvas: &mut Canvas, name: &str, element: &Value, kind: Option<&str>) {
        let Some(extent) = self.extent(element) else {
            self.defer(name, "it states no positive integer `width`/`height`");
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
        canvas.shape(shape, extent, &self.transform(element), &paint, None);
        self.painted.push(name.to_string());
    }

    /// A still, resampled through the declared `width`/`height` and cropped by `clip`.
    fn image(&mut self, canvas: &mut Canvas, name: &str, element: &Value) {
        let Some(extent) = self.extent(element) else {
            self.defer(name, "it states no positive integer `width`/`height`");
            return;
        };
        let Some(source) = element.get("source").and_then(Value::as_str) else {
            self.defer(name, "it states no `source`");
            return;
        };
        let bytes = match Source::resolve(source, &self.project_dir) {
            Source::Local(path) => match std::fs::read(&path) {
                Ok(bytes) => {
                    self.sources.push(path.display().to_string());
                    bytes
                }
                Err(e) => {
                    self.defer(name, format!("{} could not be read: {e}", path.display()));
                    return;
                }
            },
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
        let Some(raster) = Raster::decode(&bytes) else {
            self.defer(name, format!("`{source}` did not decode as an image"));
            return;
        };
        canvas.raster(
            &raster,
            extent,
            &self.transform(element),
            self.clip(element),
        );
        self.painted.push(name.to_string());
    }

    /// One decoded video frame, at the offset the caption already resolved.
    fn video(
        &mut self,
        canvas: &mut Canvas,
        name: &str,
        element: &Value,
        source_offset: Option<i64>,
    ) {
        let Some(extent) = self.extent(element) else {
            self.defer(name, "it states no positive integer `width`/`height`");
            return;
        };
        let Some(source) = element.get("source").and_then(Value::as_str) else {
            self.defer(name, "it states no `source`");
            return;
        };
        let Some(offset) = source_offset else {
            // The caption already said why — `source_offset_unresolved` carries the
            // sentence — so this one names the consequence rather than repeating it.
            self.defer(
                name,
                "its offset into the source did not resolve; the caption says why",
            );
            return;
        };
        let path = match Source::resolve(source, &self.project_dir) {
            Source::Local(path) => path,
            Source::Remote(url) => {
                self.defer(
                    name,
                    format!("`{url}` is remote; `frame` draws local sources"),
                );
                return;
            }
        };
        let ffmpeg = match self.ffmpeg() {
            Ok(ffmpeg) => ffmpeg,
            Err(reason) => {
                self.defer(name, reason);
                return;
            }
        };
        // Decoded straight to the declared box: ADR-0013 settled that a source is resampled
        // to exactly `width`x`height`, so asking `ffmpeg` for that size is the resample
        // rather than a second one on top of it.
        let decoded = match montaget_render::decode::frame_at(
            &ffmpeg,
            &path.to_string_lossy(),
            offset,
            extent.width as u32,
            extent.height as u32,
        ) {
            Ok(decoded) => decoded,
            Err(reason) => {
                self.defer(name, reason);
                return;
            }
        };
        self.sources.push(path.display().to_string());
        let Some(raster) = Raster::from_rgba(&decoded.rgba, decoded.width, decoded.height) else {
            self.defer(
                name,
                format!("`{source}`'s decoded frame was not the size asked for"),
            );
            return;
        };
        canvas.raster(
            &raster,
            extent,
            &self.transform(element),
            self.clip(element),
        );
        self.painted.push(name.to_string());
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

    /// Register one text element's declared chains, and record every file that was opened.
    fn register_fonts(&mut self, name: &str, element: &Value) {
        let mut fonts = montaget_text::Fonts::new();
        let declared = element
            .get("font")
            .and_then(Value::as_str)
            .map(str::to_string);
        for key in declared
            .into_iter()
            .chain(crate::verbs::measure::Measurable::keys(element))
        {
            if let Err(e) = crate::verbs::measure::register(self.document, &key, &mut fonts) {
                self.defer(name, format!("its font chain did not resolve: {e}"));
            }
        }
        for path in fonts.opened() {
            let path = path.display().to_string();
            if !self.fonts.contains(&path) {
                self.fonts.push(path);
            }
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
            opacity: geometry::number::<f64>(element, "opacity", self.instant, 1.0),
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

/// One `#RRGGBB`/`#RRGGBBAA` colour, through the format's own spelling rule.
///
/// [`Colour`]'s deserializer is the one authority on what a colour is — uppercase hex, no
/// shorthand, no `#RRGGBBFF`, no CSS names (ADR-0014) — so a string it refuses paints
/// nothing here and is `validate`'s to name. Re-parsing the spelling loosely would be a
/// second, more permissive answer to *"is this a colour"*, and the renderer would then draw
/// things the document's own rules say are not there.
fn rgba(value: &Value) -> Option<Rgba> {
    let colour: Colour = serde_json::from_value(value.clone()).ok()?;
    let body = colour.as_str().strip_prefix('#')?;
    let byte = |at: usize| u8::from_str_radix(body.get(at..at + 2)?, 16).ok();
    Some(Rgba([
        byte(0)?,
        byte(2)?,
        byte(4)?,
        if body.len() == 8 { byte(6)? } else { 0xFF },
    ]))
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
            Rect {
                x: 900,
                y: 1800,
                width: 180,
                height: 120
            }
        );
    }
}
