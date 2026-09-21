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
//! arithmetic is already exact and shared ([`crate::exact`]); like the fitted extent, what
//! is missing is the input mode.
//!
//! All three are named here rather than left to be discovered, because a verb that answers
//! one of its questions and is silent about the others reads as finished.

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

/// What the text occupies.
///
/// The typographic numbers and the stroked extent both, named apart. ADR-0014 requires the
/// extent this verb reports to be the **stroked** one — *"if it returns stroke-naive
/// numbers, every author adds `2 × stroke_width` by hand and they diverge"* — and naming
/// the advance beside it costs one field and keeps the derivation inspectable rather than
/// mysterious, which is the same posture ADR-0024 takes for the driving axis.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct View {
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

/// Measure one text element against the fonts a project declares.
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

    let element = match &ask.element {
        Some(element) => element,
        None => {
            return rejected(
                project,
                "`measure` needs an element: pass the text element as JSON, the same shape you are about to write into the file",
            );
        }
    };

    let spec = match Measurable::of(element) {
        Ok(spec) => spec,
        Err(reason) => return rejected(project, reason),
    };

    // Every key the element names, the base and each run's override (ADR-0007), resolved
    // before anything is laid out: a key that resolved for the element but not for a run
    // would fail half-way through the layout, leaving a partial answer to throw away.
    let mut fonts = Fonts::new();
    for key in std::iter::once(spec.asked.font.clone()).chain(Measurable::keys(element)) {
        if let Err(e) = register(&document, &key, &mut fonts) {
            return unresolvable(project, &e);
        }
    }

    let runs = runs_of(element);
    let measured = match montaget_text::measure(
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
    ) {
        Ok(measured) => measured,
        // Unreachable while every key above registered, and reported rather than
        // `expect`ed because "unreachable" is a claim about this function's own control
        // flow that a later edit can falsify silently.
        Err(e) => return unresolvable(project, &e),
    };

    Answer {
        view: Some(View::of(spec.asked, measured)),
        report: Report::new(TOOL, project),
    }
}

impl View {
    fn of(asked: Asked, measured: Measurement) -> View {
        View {
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
