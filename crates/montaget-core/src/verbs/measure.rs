//! `measure` — *"what text actually occupies"* (ADR-0011).
//!
//! The tool that makes ADR-0007's literal `size` affordable. That ADR states the cost in
//! as many words — *"this makes `frame`/`measure` mandatory in the text authoring loop"*,
//! and *"the fitted sizes in the fixture (88, 80, 73, 55, 49, 35…) are unknowable from the
//! document"* — and ADR-0011 supplies the number that makes it non-optional: nominal
//! `size × line_height` overstates real rendered ink by **1.25×–1.48×**, so a text-overflow
//! check run on nominal metrics is a false-positive generator, three out of five on a legal
//! restyle of the fixture.
//!
//! # What it takes: a whole element, never a field name and never an id
//!
//! One argument, a **complete schema-shaped text element** — the same JSON the agent is
//! about to write into the file. It is the shape ADR-0011's write-tool invariant already
//! fixes for the write side, and it is the right shape here for a reason of this verb's
//! own: ADR-0024 requires that `measure` *"must work identically for an element being
//! authored for the first time"*, and an element that does not exist yet has no `id` to
//! name.
//!
//! Only the fields measurement can see are read — `runs`, `font`, `size`, `line_height`,
//! `y`, `origin`, `stroke_width`. The rest are ignored rather than rejected, so an element
//! mid-authorship measures as readily as a finished one.
//!
//! # No verdict, and the structural form of that rule
//!
//! ADR-0024: *"`measure` returns the bare derived extent … no verdict, no diff against the
//! declared value"*, because ADR-0006 gives `validate` sole authority to compare the
//! document against itself — *"a second tool that also compares would create two paths to
//! the same judgment that can silently disagree, most obviously across an edit made between
//! the two calls."*
//!
//! So `width` and `height` are **not read**, even when the element carries them. That is
//! the rule as code rather than as discipline: there is no declared value anywhere in this
//! module to compare against.
//!
//! # The half this ticket does not build
//!
//! ADR-0024 also gives `measure` the **fitted extent** of a raster-source element and its
//! driving axis. The arithmetic for it exists and is shared ([`crate::exact::fitted_extent`],
//! #204); what is missing is this verb's second input mode. An element whose `type` is not
//! `text` is refused by name rather than measured as if it were text.
//!
//! ADR-0011 also names a **per-line ink box** beside the advance width. It is not here, and
//! the reason is that it is not an extra field: an ink box is an absolute rect, so it needs
//! the block's *horizontal* placement — `x`, `origin`'s horizontal component, and how
//! `align`'s `start`/`end` resolve against a line's base direction under bidi. No ADR
//! settles the last of those, and the same geometry is what `query --at`'s crop rectangle
//! is blocked on. It belongs with that, decided once, rather than invented twice.
//!
//! ADR-0035 gives `measure` a third answer that has nothing to do with text — *"the nearest
//! sampled instant at-or-before a given time, for the project's own `fps`"*, so an author
//! targeting an exact rendered value never derives the grid arithmetic by hand. Its
//! arithmetic is already exact and shared ([`crate::exact::frame_at_or_before`]); it is a
//! second input mode on the verb — `--at`, taking a time instead of an element — because
//! ADR-0035 is explicit that the tool call and the published formula do not substitute for
//! each other, and a mode of its own is what lets an answer state which question it is
//! ([`View::At`]).
//!
//! All three are named here rather than left to be discovered, because a verb that answers
//! one of its questions and is silent about the others reads as finished.
//!
//! # Batch mode: `--elements` and `--all` (#317)
//!
//! An authoring agent measuring dozens of elements should not pay one subprocess round-trip
//! per element. Two additive entry points feed the same engine, so there is one code path
//! and no duplicated logic between them:
//!
//! - `elements` — the primitive. An array of specs, each in the same permissive shape a
//!   single `element` already accepts, including a mid-authorship element with no `id`
//!   (ADR-0024 again — this is the same rule, applied per slot rather than to the one
//!   argument).
//! - `all` — convenience. Sourced from every element [`crate::permissive::Loose::elements`]
//!   already walks for `query`, filtered to the ones the document itself declares
//!   `"type": "text"`, then fed through the identical batch engine `elements` uses.
//!
//! **Partial failure, not whole-call rejection.** ADR-0011's *"nothing may partially process
//! a malformed file"* is about the project file reaching disk half-written; it says nothing
//! about a read-only batch over independent elements, and ADR-0024 already requires this verb
//! to tolerate an element mid-authorship. So one bad slot — an undeclared font key, a
//! non-positive `size` — does not cost the batch its N-1 good answers: each slot carries its
//! own [`Text`] or its own [`SlotError`], keyed by position and, where the element names one,
//! by `id`. A malformed *request* — `elements` that is not a JSON array at all — is still a
//! whole-call refusal, the same as every other invocation error (ADR-0011's exit 3): that is
//! a defect in the call, not in one element within it.

use std::path::Path as FilePath;

use serde::Serialize;
use serde_json::{Value, json};

use montaget_text::{
    Extent, Fonts, MeasuredLine, Measurement, Run, Segmenter, Spec, VerticalOrigin,
};

use crate::exact::{Decimal, block_height_of_tenths};
use crate::finding::Finding;
use crate::model::Origin;
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;

const TOOL: &str = "measure";

/// `line_height` defaults to 1.2 when omitted (ADR-0007), as tenths (ADR-0028).
const DEFAULT_LINE_HEIGHT_TENTHS: i64 = 12;

/// What one `measure` invocation is asking.
///
/// The element as JSON, unparsed. Which fields are required and what a malformed one means
/// is a rule about the verb, so it is settled here rather than twice in the two adapters
/// (ADR-0011).
#[derive(Debug, Clone, Default)]
pub struct Ask {
    /// A complete schema-shaped text element.
    pub element: Option<Value>,
    /// A time instead: the nearest sampled instant at-or-before it, for the project's own
    /// `fps` (ADR-0035). Mutually exclusive with `element` — one call asks one question.
    pub at: Option<i64>,
    /// An explicit batch: an array of element specs, each in `element`'s own permissive
    /// shape (#317). Mutually exclusive with `element`, `at` and `all` — one call asks one
    /// question, batch or not.
    pub elements: Option<Vec<Value>>,
    /// The convenience batch: every element the project already declares `"type": "text"`
    /// on, fed through the same engine `elements` uses (#317). Mutually exclusive with
    /// `element`, `at` and `elements`.
    pub all: bool,
}

/// One `measure` invocation's answer: what the text occupies, and the report every verb
/// answers with.
pub struct Answer {
    view: Option<View>,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The canonical JSON: the report's own object, plus the answer under `measure`.
    ///
    /// Present and `null` where no answer could be built, rather than absent — `query`'s
    /// rule, for its reason: an absent key makes *"there is no answer"* indistinguishable
    /// from a version of Montaget that did not have this verb.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "measure",
            match &self.view {
                Some(view) => serde_json::to_value(view).unwrap_or(Value::Null),
                None => Value::Null,
            },
        )
    }
}

/// Which question was asked, and its answer.
///
/// Internally tagged, `query`'s pattern: a consumer reads `mode` and knows which other keys
/// are there, rather than sniffing for the presence of `lines`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum View {
    /// An element, as JSON.
    Element(Text),
    /// A time instead (ADR-0035).
    At(Instant),
    /// A batch — `elements` or `all` (#317).
    Batch(Batch),
}

/// The ordered answer to a batch call: one slot per input element, in input order.
///
/// Ordered rather than keyed solely by `id`, because a batch element need not have one
/// (ADR-0024) and two id-less elements are still two distinct slots — `query`'s
/// [`crate::verbs::query::Named::called`] names the same problem for the same reason.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Batch {
    pub results: Vec<Slot>,
}

/// One element's outcome within a batch: a measured block, or a reason it could not be
/// measured — never neither, and never both.
///
/// `ok`/`error` rather than a `Result`-shaped tag, so a consumer reading the canonical JSON
/// sees the same "present and possibly null" shape [`Answer::to_json`] already uses for the
/// single-element case, rather than learning a second convention for the batch one.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Slot {
    /// Position in the input array — the one name every slot has, `id` or not.
    pub index: usize,
    /// The element's own `id`, where it names one. `null` for a mid-authorship element,
    /// same as [`Asked::id`] — the same fact, read the same way, one level up.
    pub id: Option<String>,
    pub ok: Option<Text>,
    pub error: Option<SlotError>,
}

/// Why one slot in a batch could not be measured.
///
/// `code` reuses the single-element path's own codes rather than inventing batch-specific
/// ones: `E-INVOCATION` for a malformed or unresolvable-by-name element (a bad shape, a
/// non-positive `size`, a font key the project does not declare), `E-READ` for a declared
/// font whose file will not open — [`unresolvable`]'s own split, applied per slot instead of
/// to the one call. Not a [`crate::finding::Finding`]: a batch's own findings would need to
/// name their slot to mean anything, which is exactly what this struct already does more
/// simply, and `measure` reaches no verdict for `push`'s repair invariant to apply to in the
/// first place (ADR-0024).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SlotError {
    pub code: &'static str,
    pub reason: String,
}

/// What the text occupies.
///
/// The typographic numbers and the stroked extent both, named apart. ADR-0014 requires the
/// extent this verb reports to be the **stroked** one — *"if it returns stroke-naive
/// numbers, every author adds `2 × stroke_width` by hand and they diverge"* — and naming
/// the advance beside it costs one field and keeps the derivation inspectable rather than
/// mysterious, which is the same posture ADR-0024 takes for the driving axis.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Text {
    /// The element measured, as the arguments named it — so the answer states its own
    /// question, and two calls in a transcript are told apart by reading them.
    pub asked: Asked,
    /// `mandatory breaks + 1` (ADR-0008).
    pub line_count: usize,
    /// The widest line's typographic advance, before the stroke.
    pub advance_width: f64,
    /// The largest ascent and descent on any line, read across **every run** and not the
    /// largest one (ADR-0029).
    pub ascent: f64,
    pub descent: f64,
    /// `ceil(Σ (largest size on line × line_height))` — the integer a text element's
    /// required `height` field takes, so an author can fill it without hand-arithmetic
    /// (ADR-0014), in ADR-0028's exact tenths.
    ///
    /// `null` only where the exact height does not fit in an integer, which needs a `size`
    /// beyond any length in a frame. A number there would be a wrapped one, and a wrapped
    /// height an author transcribed into `height` is the plausible-and-wrong failure this
    /// whole verb exists to remove.
    pub block_height: Option<i64>,
    /// Where the block sits, from `y` through `origin`.
    pub block_top: f64,
    pub block_bottom: f64,
    /// The **stroked** extent (ADR-0014) — what a box question is asked about.
    pub extent: Extent,
    pub lines: Vec<MeasuredLine>,
    /// Which segmenter produced every line's `break_opportunities` (ADR-0008). Two
    /// versions of this data legitimately disagree, so a set of offsets that did not name
    /// its source would be unfalsifiable.
    pub segmenter: Segmenter,
}

/// The style the measurement was taken in, read back off the element.
///
/// `line_height` as tenths rather than as the decimal the document writes: it is the
/// integer the arithmetic actually ran on (ADR-0028), and printing the decimal instead
/// would put the one value this verb is careful never to reach through `f64` back on the
/// wire as an `f64`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Asked {
    pub id: Option<String>,
    pub font: String,
    pub size: i64,
    pub line_height_tenths: i64,
    pub stroke_width: i64,
    pub y: i64,
    pub origin: String,
}

/// **ADR-0035's grid arithmetic**, `measure`'s second answer: the nearest sampled instant
/// at-or-before `at`, for the project's own `fps`.
///
/// So an author targeting an exact rendered value — retargeting a fade so it reaches
/// exactly 0, say, past `R-KEYFRAME-UNREACHED`'s finding — never derives `floor(t × fps /
/// 1000) × 1000 / fps` by hand.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Instant {
    /// The time asked about, so the answer states its own question.
    pub at: i64,
    /// The project's own sampling rate, read off the document rather than echoed from the
    /// call — the answer is meaningless without it, so it travels with the number it
    /// produced.
    pub fps: i64,
    /// The frame index this instant is, counted from zero at the project's own `t = 0`.
    pub frame: i64,
    /// The instant in milliseconds — `frame × 1000 / fps`, as a float for prose. The one
    /// division of the exact ratio in this verb, and it exists to be read; nothing here
    /// derives further from it.
    pub nearest: f64,
}

/// Measure one text element against the fonts a project declares, or resolve a time
/// against the project's grid — `measure`'s two input modes (ADR-0035).
pub fn measure(path: &FilePath, ask: &Ask) -> Answer {
    let project = Some(path.display().to_string());

    let document = match parse::read(path) {
        Ok(document) => document,
        // ADR-0011: nothing may partially process a malformed file.
        Err(finding) => {
            return Answer {
                view: None,
                report: Report::unparseable(TOOL, project, *finding),
            };
        }
    };

    // Which one question this call is asking — batch or not, `element` or not, `at` or not.
    // Mutually exclusive with each other, `element`/`at`'s own existing rule extended to the
    // two new modes rather than replaced (#317).
    let named = usize::from(ask.element.is_some())
        + usize::from(ask.at.is_some())
        + usize::from(ask.elements.is_some())
        + usize::from(ask.all);
    if named > 1 {
        return rejected(
            project,
            "`measure` takes exactly one of an element, `--at`, `--elements` or `--all`: \
             they are different questions and a call naming more than one does not say \
             which it is asking",
        );
    }

    if let Some(elements) = &ask.elements {
        return batch(&document, project, elements.iter().cloned());
    }
    if ask.all {
        // #317: sourced from the same walk `query` already uses over every element in the
        // document, filtered to the ones the project itself declares text — not the
        // permissive "absent `type` measures as text" rule `Measurable::of` applies to a
        // single mid-authorship element, because these elements are already written and
        // already carry the field the schema requires.
        let elements: Vec<Value> = document
            .elements()
            .filter(|element| element.get("type").and_then(Value::as_str) == Some("text"))
            .cloned()
            .collect();
        return batch(&document, project, elements);
    }

    let element = match (&ask.element, ask.at) {
        (Some(_), Some(_)) => unreachable!("named > 1 above already refused this"),
        (None, Some(at)) => return by_instant(&document, project, at),
        (Some(element), None) => element,
        (None, None) => {
            return rejected(
                project,
                "`measure` needs an element, `--at`, `--elements` or `--all`: pass the text \
                 element as JSON, the same shape you are about to write into the file, an \
                 array of them, a time to resolve against the project's frame grid \
                 (ADR-0035), or ask for every text element the project already has",
            );
        }
    };

    match try_measure_element(&document, element) {
        Ok(text) => Answer {
            view: Some(View::Element(text)),
            report: Report::new(TOOL, project),
        },
        Err(ElementError::Invocation(reason)) => rejected(project, reason),
        Err(ElementError::Font(e)) => unresolvable(project, &e),
    }
}

/// One element's failure to measure, on its way to becoming either a whole-call refusal (the
/// single-element path) or one [`SlotError`] (the batch path) — the same two causes
/// [`unresolvable`] already distinguishes, held here so both callers read them once.
enum ElementError {
    /// The element itself is malformed, or names a font key the project does not declare —
    /// [`Measurable::of`]'s and [`register`]'s own invocation-shaped refusals.
    Invocation(String),
    /// A declared font whose file will not open — a fact about the project and its disk.
    Font(montaget_text::FontError),
}

/// The shared core of every measurement, batch or not: resolve the element's style, register
/// every font key it and its runs name, and lay it out. One path, so `elements`/`all` cannot
/// drift from what a single `element` call already does (#317).
fn try_measure_element(document: &Loose, element: &Value) -> Result<Text, ElementError> {
    let spec = Measurable::of(element).map_err(ElementError::Invocation)?;

    // Every key the element names, the base and each run's override (ADR-0007), resolved
    // before anything is laid out: a key that resolved for the element but not for a run
    // would fail half-way through the layout, leaving a partial answer to throw away.
    let mut fonts = Fonts::new();
    for key in std::iter::once(spec.asked.font.clone()).chain(Measurable::keys(element)) {
        if let Err(e) = register(document, &key, &mut fonts) {
            return Err(ElementError::Font(e));
        }
    }

    let runs = runs_of(element);
    let measured = montaget_text::measure(
        &mut fonts,
        &Spec {
            runs: &runs,
            font: &spec.asked.font,
            size: spec.asked.size,
            line_height_tenths: spec.asked.line_height_tenths,
            stroke_width: spec.asked.stroke_width,
            y: spec.asked.y,
            vertical_origin: spec.vertical_origin,
        },
    )
    // Unreachable while every key above registered, and reported rather than `expect`ed
    // because "unreachable" is a claim about this function's own control flow that a later
    // edit can falsify silently.
    .map_err(ElementError::Font)?;

    Ok(Text::of(spec.asked, measured))
}

/// Measure every element in `elements`, in order, never rejecting the call for one bad slot
/// (#317's partial-failure rule).
fn batch(
    document: &Loose,
    project: Option<String>,
    elements: impl IntoIterator<Item = Value>,
) -> Answer {
    let results = elements
        .into_iter()
        .enumerate()
        .map(|(index, element)| {
            let id = element
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string);
            match try_measure_element(document, &element) {
                Ok(text) => Slot {
                    index,
                    id,
                    ok: Some(text),
                    error: None,
                },
                Err(e) => Slot {
                    index,
                    id,
                    ok: None,
                    error: Some(slot_error(e)),
                },
            }
        })
        .collect();

    Answer {
        view: Some(View::Batch(Batch { results })),
        report: Report::new(TOOL, project),
    }
}

/// [`ElementError`] as the [`SlotError`] a batch slot carries — [`unresolvable`]'s own E-READ
/// / E-INVOCATION split, restated for a slot that does not get to fail the whole call.
fn slot_error(e: ElementError) -> SlotError {
    match e {
        ElementError::Invocation(reason) => SlotError {
            code: "E-INVOCATION",
            reason,
        },
        ElementError::Font(e) => match &e.path {
            Some(path) => SlotError {
                code: "E-READ",
                reason: format!("{}: {}", path.display(), e.reason),
            },
            None => SlotError {
                code: "E-INVOCATION",
                reason: e.reason,
            },
        },
    }
}

/// The nearest-sampled-instant answer, resolved against the project's declared `fps`.
///
/// **Document-only**, `R-KEYFRAME-UNREACHED`'s budget: no I/O beyond the document already
/// in hand, since the grid needs nothing else.
fn by_instant(document: &Loose, project: Option<String>, at: i64) -> Answer {
    if at < 0 {
        // The format's times are absolute integer milliseconds and never negative
        // (ADR-0005); a negative `at` names no instant on the clock at all, which is a
        // different fact from a legal instant this project's grid simply doesn't sample.
        return rejected(
            project,
            format!("`--at` is an absolute millisecond and cannot be negative, not {at}"),
        );
    }
    let fps = match document.value().get("fps").and_then(Value::as_i64) {
        Some(fps) if fps > 0 => fps,
        Some(fps) => {
            return rejected(
                project,
                format!("the project's `fps` must be positive, not {fps}"),
            );
        }
        None => {
            return rejected(
                project,
                "the project has no `fps`, and the grid ADR-0035 samples is undefined without it",
            );
        }
    };
    // `fps > 0` above is exactly `frame_at_or_before`'s only refusal condition, so this
    // always succeeds.
    let sampled = crate::exact::frame_at_or_before(at, fps).expect("fps checked positive above");

    Answer {
        view: Some(View::At(Instant {
            at,
            fps,
            frame: sampled.frame,
            nearest: sampled.ms(),
        })),
        report: Report::new(TOOL, project),
    }
}

impl Text {
    fn of(asked: Asked, measured: Measurement) -> Text {
        Text {
            asked,
            line_count: measured.line_count,
            advance_width: measured.advance_width,
            ascent: measured.ascent,
            descent: measured.descent,
            // ADR-0028's `ceil`, from the one place it lives. The engine hands back exact
            // tenths and never rounds, so `validate`'s `R-BOX-SLACK` and this verb cannot
            // carry two implementations of the same boundary.
            block_height: block_height_of_tenths(measured.block_height_tenths),
            block_top: measured.block_top,
            block_bottom: measured.block_bottom,
            extent: measured.extent,
            lines: measured.lines,
            segmenter: measured.segmenter,
        }
    }
}

/// An element's own measurable style, read off the JSON permissively.
pub(crate) struct Measurable {
    pub(crate) asked: Asked,
    pub(crate) vertical_origin: VerticalOrigin,
}

impl Measurable {
    /// The fields measurement needs, or the one sentence saying which is wrong.
    ///
    /// Permissive about everything else, deliberately: an element mid-authorship is exactly
    /// the element this verb exists for (ADR-0024), and refusing it for a `width` it has
    /// not been given yet would refuse the case.
    pub(crate) fn of(element: &Value) -> Result<Measurable, String> {
        let Some(object) = element.as_object() else {
            return Err("the element must be a JSON object".into());
        };
        if let Some(kind) = object.get("type").and_then(Value::as_str)
            && kind != "text"
        {
            return Err(format!(
                "`measure` measures text, and this element's `type` is `{kind}`; the \
                 fitted-extent half of `measure` (ADR-0024) is not built yet"
            ));
        }

        let font = object
            .get("font")
            .and_then(Value::as_str)
            .ok_or("the element needs a `font`: a key into the project's `fonts` table")?
            .to_string();
        let size = object.get("size").and_then(Value::as_i64).ok_or(
            "the element needs an integer `size` (ADR-0007: the renderer never chooses one)",
        )?;
        if size <= 0 {
            // A `size` is a length in the project's frame space (ADR-0012's absolute
            // integer pixels), and there is no text a non-positive one describes. Naming it
            // as a defect in the document is `validate`'s; refusing to answer about it is
            // this verb's, because the alternative is a block of zero or negative height
            // reported as if it were a measurement.
            return Err(format!(
                "`size` is a length in pixels and must be positive, not {size}"
            ));
        }
        let line_height_tenths = match object.get("line_height") {
            None | Some(Value::Null) => DEFAULT_LINE_HEIGHT_TENTHS,
            Some(value) => value
                .as_number()
                .and_then(Decimal::of)
                .and_then(Decimal::tenths)
                .ok_or("`line_height` is restricted to one decimal digit (ADR-0028)")?,
        };
        if object.get("runs").and_then(Value::as_array).is_none() {
            return Err("the element needs a `runs` array".into());
        }

        let origin = match object.get("origin") {
            None | Some(Value::Null) => Origin::Center,
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|_| format!("`origin` is not one of the nine keywords: {value}"))?,
        };

        Ok(Measurable {
            asked: Asked {
                id: object.get("id").and_then(Value::as_str).map(str::to_string),
                font,
                size,
                line_height_tenths,
                stroke_width: object
                    .get("stroke_width")
                    .and_then(Value::as_i64)
                    .unwrap_or(0),
                // An element with no `y` measures at zero rather than being refused: `y`
                // places the block and nothing else, so a caller asking only for the
                // advance width should not have to invent a coordinate first. Every
                // vertical number in the answer is then relative to zero, and says so by
                // echoing the `y` it used.
                y: object.get("y").and_then(Value::as_i64).unwrap_or(0),
                origin: serde_json::to_value(origin)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_else(|| "center".into()),
            },
            vertical_origin: vertical_origin_of(origin),
        })
    }

    /// Every distinct `font` key the runs override with — read off the array below rather
    /// than off a parsed copy, so there is one reading of it.
    pub(crate) fn keys(element: &Value) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        for run in runs_array(element) {
            if let Some(key) = run.get("font").and_then(Value::as_str)
                && !keys.iter().any(|seen| seen == key)
            {
                keys.push(key.to_string());
            }
        }
        keys
    }
}

pub(crate) fn runs_array(element: &Value) -> &[Value] {
    element
        .get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// The runs, as much of each as measurement can see, borrowed from the element itself.
pub(crate) fn runs_of(element: &Value) -> Vec<Run<'_>> {
    runs_array(element)
        .iter()
        .map(|run| Run {
            // A run carrying no `text` contributes no character rather than refusing the
            // measurement: naming a malformed run is `validate`'s schema check (ADR-0006).
            text: run.get("text").and_then(Value::as_str).unwrap_or_default(),
            font: run.get("font").and_then(Value::as_str),
            size: run.get("size").and_then(Value::as_i64),
            stroke_width: run.get("stroke_width").and_then(Value::as_i64),
        })
        .collect()
}

/// `origin`'s vertical component — the half that places the block (ADR-0013).
pub(crate) fn vertical_origin_of(origin: Origin) -> VerticalOrigin {
    match origin {
        Origin::TopLeft | Origin::TopCenter | Origin::TopRight => VerticalOrigin::Top,
        Origin::CenterLeft | Origin::Center | Origin::CenterRight => VerticalOrigin::Center,
        Origin::BottomLeft | Origin::BottomCenter | Origin::BottomRight => VerticalOrigin::Bottom,
    }
}

/// Register one key's declared chain from the project's `fonts` table.
///
/// The paths resolve against the project file's own directory (ADR-0053), like every other
/// relative path the format carries.
pub(crate) fn register(
    document: &Loose,
    key: &str,
    fonts: &mut Fonts,
) -> Result<(), montaget_text::FontError> {
    if fonts.declares(key) {
        return Ok(());
    }
    let base = crate::checks::project_dir(document);
    let chain = document
        .value()
        .get("fonts")
        .and_then(Value::as_object)
        .and_then(|table| table.get(key))
        .and_then(Value::as_array)
        .ok_or_else(|| montaget_text::FontError::undeclared(key))?;

    let files: Vec<montaget_text::FontFile> = chain
        .iter()
        .filter_map(|entry| {
            let file = entry.get("file").and_then(Value::as_str)?;
            let index = entry.get("index").and_then(Value::as_u64).map(|i| i as u32);
            Some(montaget_text::fonts::resolve(file, index, &base))
        })
        .collect();

    fonts.register(key, &files)
}

/// One unresolvable font chain, as the report that says so.
///
/// The two causes end up in different places on purpose, and the deciding question is
/// *what does the caller do next* — which is ADR-0011's own organising rule for its exit
/// codes.
///
/// - **A key the project does not declare** is exit 3: the *command* named a font the
///   document does not carry, so the next move is to fix the command, not the project.
/// - **A declared key whose file will not open** is a fact about the project and its disk,
///   and `E-READ` is the finding that already says it — the same code every other verb
///   reports an unopenable file under, named against the *font*, not the project file, so
///   the sentence points at the file the reader must go and vendor. The checks that will
///   judge a font chain as such are #206's.
fn unresolvable(project: Option<String>, e: &montaget_text::FontError) -> Answer {
    let Some(path) = &e.path else {
        return rejected(project, e.reason.clone());
    };
    let path = path.display().to_string();
    let mut report = Report::new(TOOL, project);
    report.push(
        Finding::new("E-READ")
            .at_file(&path)
            .field("file", json!(path))
            .field("reason", json!(e.reason))
            // ADR-0073: not about a document — message text in the template's
            // `{advice}`, not a structured repair.
            .field("advice", json!("vendor the declared font file")),
    );
    Answer { view: None, report }
}

fn rejected(project: Option<String>, reason: impl Into<String>) -> Answer {
    Answer {
        view: None,
        report: Report::rejected(TOOL, project, reason),
    }
}
