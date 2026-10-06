//! `measure`'s keyed-alpha coverage derivation (ADR-0088) — *"did this key, and did it
//! **stay** keyed?"*
//!
//! ADR-0040 refused the chroma keyer on the grounds that its result was *"not readable from
//! the schema alone"*. ADR-0088 overturns that and names this reading as the mechanism that
//! makes it readable: an agent that writes `tolerance: 0.01` and asks `measure` is told
//! `opaque 100%`, which is the whole diagnosis, without looking at a picture.
//!
//! # A series, not a frame, and that is the whole point
//!
//! Footage whose screen drifts mid-take keys well at one instant and badly at another. A
//! single sample answers *"did this key at all"*. The series answers *"did this key stay"* —
//! and a coverage series that steps mid-element is exactly the signal telling an author
//! where the drift is, so where to key `tolerance` (ADR-0146) or cut. Without it, they would
//! be keying blind, which is why ADR-0088 calls the reading load-bearing rather than
//! decorative. Each sample is painted with every keyed parameter resolved at its own instant.
//!
//! # No verdict, structurally
//!
//! ADR-0024 fixes `measure`'s shape: *"the bare derived extent… no verdict, no diff against
//! the declared value"*, because ADR-0006 gives `validate` sole authority to compare a
//! document against itself. So there is no threshold in this module, nothing is compared to
//! anything, and the three fractions are stated and left. What a *stepping* series means is
//! the author's to decide.
//!
//! # One keyer, reached through the renderer
//!
//! The coverage is read off the alpha the **rasterizer** produced, by painting each decoded
//! frame through the element's own ordered `effects` list and reading the surface back.
//! Nothing here knows what a key is. A second implementation — an arithmetic copy of the
//! shader, in Rust, to count pixels with — would be a second answer to *"what does this
//! document key"*, and the reading whose entire job is to tell an author what the render
//! will do would be the one thing in the codebase that could disagree with it.
//!
//! So `chroma` is not singled out below: the **whole** list is applied, in order. A `blur`
//! ahead of the key softens the pixels the key is measured against, and the coverage
//! reported is the coverage that element actually has.

use serde::Serialize;
use serde_json::Value;

use montagent_render::canvas::{Canvas, Extent, Raster, Rgba, Transform};
use montagent_render::decode;

use crate::exact::{self, Decimal};
use crate::media::session::Session;
use crate::media::tools::Missing;
use crate::media::{Source, tools};
use crate::model;
use crate::permissive::Loose;

/// What an element's key actually does to its own pixels, frame by frame.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Coverage {
    /// The element measured, as the argument named it — so the answer states its own
    /// question, [`crate::verbs::measure::Asked`]'s rule for the text half.
    pub asked: Asked,
    /// The project's own sampling rate, read off the document. The series is meaningless
    /// without it, so it travels with the numbers it produced.
    pub fps: i64,
    /// The rate the source was sampled at, in **source** frames per second of source time:
    /// `fps ÷ speed`, because a frame of timeline moves `speed / fps` seconds through the
    /// source. Stated because it is not `fps` whenever the element is retimed, and a reader
    /// comparing two elements' series needs to know they are on different clocks. Until
    /// ADR-0127 this said `fps × speed` and the run was sampled at it.
    pub source_fps: f64,
    /// One entry per sampled frame, in time order.
    pub frames: Vec<Sample>,
}

/// The element the coverage was read from, as the call named it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Asked {
    /// `null` for an element being authored for the first time, which ADR-0024 requires
    /// this verb to measure as readily as a finished one.
    pub id: Option<String>,
    pub r#type: String,
    pub source: String,
    /// The element's declared box, which is the space the key is computed in (ADR-0084's
    /// rule for `mask`, which ADR-0088 takes for `chroma`): the source is resampled to
    /// exactly this, and the effects run on the result.
    pub width: i64,
    pub height: i64,
}

/// One frame's alpha, as three fractions of the element's own box.
///
/// They sum to 1 by construction — every pixel is in exactly one of the three — so the
/// third is not redundant with the other two for a reader, only for an arithmetician.
/// ADR-0088 names all three, and the partial band is the one that says whether `softness`
/// did anything.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sample {
    /// The frame's index in the series, from zero at the element's own `start`.
    pub frame: usize,
    /// The timeline instant this frame is shown at, in absolute milliseconds (ADR-0005).
    pub at: i64,
    /// Fully opaque — the matte kept this pixel outright.
    pub opaque: f64,
    /// Between the two: the blend band `softness` opens, and antialiased mask edges.
    pub partial: f64,
    /// Fully transparent — the matte took this pixel out.
    pub transparent: f64,
}

/// Does this element key pixels that came off a file?
///
/// The dispatch question `measure` asks before it decides which of its answers this element
/// has. **Both halves are load-bearing.** A key on a `text` element is legal — ADR-0088
/// makes `R-CHROMA-ON-AUTHORED-ELEMENT` a `review` rather than an error — and such an
/// element still has text occupying a block, which is the answer ADR-0024 gives it and
/// which this verb would otherwise have taken away. So the coverage answer belongs to the
/// elements whose pixels this document did not author, and those are the ones naming a
/// `source`.
///
/// Read permissively — an `effects` that is not an array is the schema check's to report,
/// not a reason to refuse a measurement.
pub(crate) fn is_keyed(element: &Value) -> bool {
    element.get("source").and_then(Value::as_str).is_some()
        && element
            .get("effects")
            .and_then(Value::as_array)
            .is_some_and(|effects| {
                effects.iter().any(|effect| {
                    effect.get("name").and_then(Value::as_str) == Some(model::effects::CHROMA)
                })
            })
}

/// Why [`coverage`] produced no series.
///
/// Most refusals below are about the element the caller passed, and `measure` turns those
/// into the same `E-INVOCATION` its text half already answers a malformed element with
/// (ADR-0011's exit 3). `ffmpeg`/`ffprobe` being unresolvable is not one of those — it is a
/// fact about the machine, not the element — so it carries the [`Missing`] `render`,
/// `preview` and `validate` already turn into ADR-0091's `E-TOOL-MISSING`/`E-INTERNAL` at
/// exit 70, rather than collapsing into the same string as an `E-INVOCATION` (#377).
pub(crate) enum CoverageError {
    Invocation(String),
    Missing(Missing),
}

impl From<String> for CoverageError {
    fn from(reason: String) -> Self {
        CoverageError::Invocation(reason)
    }
}

impl From<&str> for CoverageError {
    fn from(reason: &str) -> Self {
        CoverageError::Invocation(reason.to_string())
    }
}

/// The series, or the reason there is none.
pub(crate) fn coverage(document: &Loose, element: &Value) -> Result<Coverage, CoverageError> {
    let fps = match document.value().get("fps").and_then(Value::as_i64) {
        Some(fps) if fps > 0 => fps,
        _ => {
            return Err(
                "the project has no positive `fps`, and a per-frame series has no grid to sit on"
                    .into(),
            );
        }
    };

    let declared = element
        .get("type")
        .and_then(Value::as_str)
        .ok_or("the element states no `type`, so nothing knows where its pixels come from")?;
    // [`is_keyed`] has already established this, and the message is for the one other
    // caller there could be rather than for a path `measure` can reach.
    let source = element.get("source").and_then(Value::as_str).ok_or(
        "the element states no `source`: a key reads pixels off a file, and an element \
         that names none has none to read",
    )?;
    let (width, height) = match (
        element.get("width").and_then(Value::as_i64),
        element.get("height").and_then(Value::as_i64),
    ) {
        (Some(width), Some(height)) if width > 0 && height > 0 => (width, height),
        _ => {
            return Err(
                "the element states no positive `width`/`height`, which is the box the key \
                 is computed in"
                    .into(),
            );
        }
    };

    let effects = effects_of(element)?;
    let path = match Source::resolve(source, &crate::checks::project_dir(document)) {
        Source::Local(path) => path,
        // ADR-0056's rule, which `frame` already holds: the remote half belongs to `probe`'s
        // session, which fetches ranges rather than whole files. Decoding one here would be
        // the unsolicited network call this surface promises not to make.
        Source::Remote(url) => {
            return Err(
                format!("`{url}` is remote; `measure` reads coverage off local sources").into(),
            );
        }
    };
    let ffmpeg = tools::resolve()
        .map(|tools| tools.ffmpeg)
        .map_err(CoverageError::Missing)?;

    // ADR-0089: which decoder this source needs is the probe's reading, not this
    // reading's. A keyed-alpha coverage measurement over a VP9 cutout decoded by the
    // native `vp9` decoder would measure a fully opaque frame and report 0.0 coverage —
    // a number, confidently wrong, about a source that is keyed perfectly well.
    let decoder = {
        let mut session = Session::open().map_err(CoverageError::Missing)?;
        session
            .decoder_for(&Source::Local(path.clone()))
            .map_err(|missing| CoverageError::Missing(*missing))?
    };

    let run = run_of(element, declared, fps)?;
    let mut frames = decode::frames_from(
        &ffmpeg,
        &path.to_string_lossy(),
        decoder,
        run.from_ms,
        decode::Pace {
            fps,
            speed: run.speed,
        },
        width as u32,
        height as u32,
    )?;

    // The element's declared box, which is what a source is resampled to (ADR-0013) and so
    // what every pixel below is a fraction of.
    let box_ = Extent {
        width: width as f64,
        height: height as f64,
    };
    // One surface for the whole series, cleared between frames. A 1080p surface per sample
    // is 140 allocations of 8 MB on ADR-0088's own forcing case, and the reading is a
    // fraction of the same box every time — there is nothing for a second surface to be.
    let mut canvas =
        Canvas::new(width, height).ok_or("no surface could be made at the element's own box")?;
    let mut samples = Vec::with_capacity(run.count);
    for frame in 0..run.count {
        let Some(decoded) = frames.next_frame()? else {
            // The source ran out before the document said it would. That disagreement is
            // `validate`'s finding (`E-SOURCE-OVERRUN`) and not this reading's to re-derive,
            // so the series is the length it got and says so by being that length.
            break;
        };
        let at = run.at(frame, fps);
        samples.push(sample(
            &mut canvas,
            frame,
            at,
            &decoded,
            &effects_at(element, &effects, at),
            box_,
        )?);
    }

    Ok(Coverage {
        asked: Asked {
            id: element
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string),
            r#type: declared.to_string(),
            source: source.to_string(),
            width,
            height,
        },
        fps,
        source_fps: run.source_fps,
        frames: samples,
    })
}

/// One frame, painted through the element's effects and counted.
fn sample(
    canvas: &mut Canvas,
    frame: usize,
    at: i64,
    decoded: &decode::DecodedFrame,
    effects: &[montagent_render::canvas::Effect],
    box_: Extent,
) -> Result<Sample, String> {
    let raster = Raster::from_rgba(&decoded.rgba, decoded.width, decoded.height)
        .ok_or("a decoded frame was not the size it was asked for")?;
    // Wiped to nothing rather than to a colour: the reading is about transparency, and a
    // surface carrying the previous frame's matte underneath this one would report a
    // coverage no frame has.
    canvas.background(Rgba([0, 0, 0, 0]));
    // The element's own box, at the origin, untransformed: ADR-0084 puts `effects` in
    // **element space**, so the coverage is a fraction of the element rather than of a
    // frame it has not been placed on yet. An element's `x`, `scale` and `rotation` move
    // where that box lands and change no pixel inside it.
    canvas.raster(
        &raster,
        box_,
        &Transform {
            x: 0.0,
            y: 0.0,
            scale: (1.0, 1.0),
            rotation: 0.0,
            opacity: 1.0,
            blend: montagent_render::canvas::Blend::Normal,
            origin: (0.0, 0.0),
        },
        None,
        effects,
    );
    let rgba = canvas
        .rgba()
        .ok_or("the keyed surface could not be read back")?;

    let total = rgba.len() / 4;
    if total == 0 {
        return Err("the keyed surface held no pixels".to_string());
    }
    let (mut opaque, mut transparent) = (0usize, 0usize);
    for pixel in rgba.chunks_exact(4) {
        match pixel[3] {
            0 => transparent += 1,
            0xFF => opaque += 1,
            _ => {}
        }
    }

    // Counted in two buckets and the third derived, so the three provably sum to 1 rather
    // than nearly doing so.
    let fraction = |count: usize| count as f64 / total as f64;
    Ok(Sample {
        frame,
        at,
        opaque: fraction(opaque),
        partial: fraction(total - opaque - transparent),
        transparent: fraction(transparent),
    })
}

/// What one run of frames asks `ffmpeg` for.
struct Run {
    /// Where the run starts, in **source** milliseconds.
    from_ms: i64,
    /// Source frames per second of source time — `fps ÷ speed` — as the answer states it.
    source_fps: f64,
    /// `speed` as the exact rational `(numerator, denominator)` the run is sampled at.
    speed: (i128, i128),
    /// How many frames the element's own range shows.
    count: usize,
    /// The element's `start`, so a sample can name the instant it is shown at.
    start: i64,
}

impl Run {
    /// The timeline instant frame `n` of this run is shown at (ADR-0005: absolute integer
    /// milliseconds, and the grid is the project's own).
    fn at(&self, n: usize, fps: i64) -> i64 {
        self.start + (n as i64) * 1000 / fps
    }
}

/// The run this element's declared playback asks for.
///
/// **Sampled over the element's own source range, never over its timeline range.** The two
/// differ only where `overrun` extends the element past the material it has, and there the
/// extra instants show frames the run has already sampled — held or looped (ADR-0020). So
/// this is every *distinct* frame the element puts on screen, which is the population the
/// drift question is asked of.
fn run_of(element: &Value, declared: &str, fps: i64) -> Result<Run, String> {
    let start = element
        .get("start")
        .and_then(Value::as_i64)
        .ok_or("the element states no `start`, so its frames sit at no instant")?;
    let end = element
        .get("end")
        .and_then(Value::as_i64)
        .filter(|end| *end > start)
        .ok_or("the element's `end` does not follow its `start`, so it shows no frames")?;

    // A still has one picture, whatever range it occupies: its series is one sample, and a
    // hundred identical ones would state the same fact a hundred times.
    if declared != "video" {
        return Ok(Run {
            from_ms: 0,
            source_fps: fps as f64,
            speed: (1, 1),
            count: 1,
            start,
        });
    }

    let (Some(source_start), Some(source_end)) = (
        element.get("source_start").and_then(Value::as_i64),
        element.get("source_end").and_then(Value::as_i64),
    ) else {
        return Err(
            "the element carries no integer `source_start`/`source_end`, so there is no \
             source range to sample"
                .to_string(),
        );
    };
    let source_span = source_end - source_start;
    if source_span <= 0 {
        return Err("`source_end` does not exceed `source_start`".to_string());
    }

    // `crate::verbs::query::at::source_offset`'s own reading of the same field, including
    // its refusal: a `speed` that is not a positive number is `validate`'s finding to make,
    // not a number this reading may pick for itself.
    let speed = match element.get("speed") {
        None | Some(Value::Null) => Decimal::of(&serde_json::Number::from(1)),
        Some(value) => value.as_number().and_then(Decimal::of),
    };
    let speed = speed
        .filter(|speed| speed.is_positive())
        .ok_or("`speed` is not a positive number, which is `validate`'s finding to make")?;
    let played = exact::played_ms(source_span, speed)
        .ok_or("the as-played duration of the source range could not be computed")?;

    // Every instant the element is on screen *and* has material for. `overrun` covers the
    // remainder with frames already in this run.
    let shown = played.min(end - start);
    // Ceiling, because the last partial frame interval is still a frame that is shown —
    // integer arithmetic throughout, as ADR-0045 requires of anything reading the clock.
    let ticks = shown.saturating_mul(fps);
    let count = usize::try_from(ticks / 1000 + i64::from(ticks % 1000 != 0))
        .map_err(|_| "the element shows more frames than can be counted".to_string())?;

    // **The run is sampled on the exact ratio**, the same one `source_advance` above and the
    // renderer's own offset-into-source are computed on (ADR-0127); `frames_from` takes it as
    // integers so no float stands between the two. `source_fps` is the one float, and it is
    // the answer's statement of the clock rather than an input to anything.
    let speed_ratio = speed
        .as_ratio()
        .ok_or("`speed` could not be read as an exact ratio")?;
    let source_fps = fps as f64 / speed.as_f64();

    Ok(Run {
        from_ms: source_start,
        source_fps,
        speed: speed_ratio,
        count,
        start,
    })
}

/// The element's ordered `effects`, in the rasterizer's spelling.
///
/// Through the model's own enum, never a looser reading: `crate::model::Effect` is the one
/// authority on what an effect is, and a coverage reading that keyed by a rule the renderer
/// does not follow would be the second answer this whole module exists not to be.
fn effects_of(element: &Value) -> Result<Vec<model::Effect>, String> {
    match element.get("effects") {
        None => Ok(Vec::new()),
        Some(effects) => serde_json::from_value(effects.clone())
            .map_err(|e| format!("the element's `effects` are not this format's: {e}")),
    }
}

/// The element's `effects` as the rasterizer paints them at `instant`, every keyed parameter
/// resolved there (ADR-0146) — so a keyed `tolerance` is measured at the value each frame
/// keys with.
fn effects_at(
    element: &Value,
    declared: &[model::Effect],
    instant: i64,
) -> Vec<montagent_render::canvas::Effect> {
    (0..declared.len())
        .filter_map(|index| {
            crate::verbs::frame::effect_of(element, index, (i128::from(instant), 1))
        })
        .collect()
}
