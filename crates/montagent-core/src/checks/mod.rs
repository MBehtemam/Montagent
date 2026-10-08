//! The checks themselves — one module per question asked of a whole project.
//!
//! A check reads [`crate::permissive::Loose`] and appends findings to a
//! [`crate::report::Report`]. It never decides its own repair class, its own template or
//! its own ADR provenance: those are declared once in [`crate::registry`], and a check
//! that wanted to vary any of them per instance is the failure ADR-0043 forbids.
//!
//! Everything here is called from [`crate::verbs::validate`], which runs every check on
//! the whole project every time — there is no fast mode and no way to narrow what is
//! analysed (ADR-0006).
//!
//! A check may also need something no document carries. [`source`] needs the disk, so it
//! takes a probe session and can fail rather than find: *"there is no `ffprobe`"* is not a
//! fact about the project and never becomes a finding about one (ADR-0011's exit 70).

pub mod anchor;
pub mod blend;
pub mod box_slack;
pub mod canvas;
pub mod caption;
pub mod chroma;
pub mod coverage;
pub mod cut;
pub mod derived;
pub mod ease;
pub mod extent;
pub mod fit;
pub mod fonts;
pub mod gradient;
pub mod grain;
pub mod highlight;
pub mod ink;
pub mod layout;
pub mod mask;
pub mod motion_blur;
pub mod overridden;
pub mod path;
pub mod projection;
pub mod quantization;
pub mod range;
pub mod remap;
pub mod retired;
pub mod runs;
pub mod schema;
pub mod source;
pub mod spacing;
pub mod speed;
pub mod stroke;
pub mod text_path;
pub mod tie;
pub mod track;
pub mod transition;
pub mod units;
pub mod unreached;

/// What a finding's prose calls the project itself, where the subject is not an element.
pub(crate) const PROJECT: &str = "the project";

/// What a finding's prose calls one element.
///
/// Here rather than in either check because two now name one, and ADR-0006's *"one code,
/// one field set, one template"* is undone at the report if two checks call the same
/// element two things — an element with no readable `id` is exactly the case where the
/// spellings would diverge without anyone noticing, since it is the rare one.
pub(crate) fn subject_of(id: Option<&str>) -> String {
    id.unwrap_or("an element carrying no `id`").to_string()
}

/// The directory a relative `source` resolves against: the project file's own (ADR-0053).
///
/// Shared by every check that resolves a `source` off disk — [`source`] and [`fit`] both
/// need it, and a second copy is a second place for ADR-0053's rule to drift.
pub(crate) fn project_dir(document: &crate::permissive::Loose) -> std::path::PathBuf {
    std::path::Path::new(document.path())
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf()
}

/// One run of a `text` element: its own text, and the `font` delta it states over the
/// element's base style (ADR-0007).
pub(crate) struct StyledRun {
    pub(crate) font: Option<String>,
    pub(crate) text: String,
}

/// One `text` element, as every check that reads its `runs` sees it.
///
/// Shared rather than walked twice: ADR-0007's font census and its three text-byte checks
/// (`crate::checks::fonts`, `crate::checks::runs`) all want the same four facts off the same
/// traversal, and two copies of "which element, which track, which font, which runs" is two
/// places for the permissive reading of a half-written element to drift.
///
/// **[`caption`]'s own `TextElement` is deliberately not this**, and its module doc says
/// why: those checks group by time and need `start`/`end`, and they are scoped *not* to read
/// the style fields this carries.
pub(crate) struct StyledText {
    pub(crate) subject: String,
    pub(crate) track: Option<String>,
    /// The element's base `font`, where it states one readably. A run with no delta of its
    /// own is set in this.
    pub(crate) font: Option<String>,
    pub(crate) runs: Vec<StyledRun>,
}

impl StyledText {
    /// Every run's text, end to end — the element's content as one string, which is what a
    /// question about *characters* rather than about styles is asked of.
    pub(crate) fn joined(&self) -> String {
        self.runs.iter().map(|run| run.text.as_str()).collect()
    }
}

/// Every `text` element, in document order.
///
/// Read permissively throughout: a run that is not an object, or a `text`/`font` that is not
/// a string, is the schema check's to report and is simply not read here. A missing `text` is
/// an empty run rather than a dropped one, so that the *boundaries* between runs stay where
/// the document puts them.
pub(crate) fn styled_text(document: &crate::permissive::Loose) -> Vec<StyledText> {
    use serde_json::Value;
    document
        .elements_in_tracks()
        .filter(|(_, element)| element.get("type").and_then(Value::as_str) == Some("text"))
        .map(|(track, element)| StyledText {
            subject: subject_of(element.get("id").and_then(Value::as_str)),
            track: track.map(str::to_string),
            font: element
                .get("font")
                .and_then(Value::as_str)
                .map(str::to_string),
            runs: element
                .get("runs")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .map(|run| StyledRun {
                    font: run.get("font").and_then(Value::as_str).map(str::to_string),
                    text: run
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect(),
        })
        .collect()
}

/// **Every keyframe list on one element that the record checks walk**, as
/// `(name, container, property)`: every animatable property the element writes — the one
/// list, [`crate::animatable::declared`], read off the schema (ADR-0146): its own, then each
/// `effects` member's parameters, named `effects[1].radius (blur)` — then a text element's
/// stagger lists: the `units` block's (`units.y`) and each run override's
/// (`runs[3].unit.y`), ADR-0151 §5. The unit lists are nested inside the element, so they are
/// their own walk rather than members of the element-level list. `name` is what a finding
/// calls the list, so an agent finds it in the file.
pub(crate) fn keyframe_lists(
    element: &serde_json::Value,
) -> Vec<(String, &serde_json::Value, &str)> {
    let mut out: Vec<(String, &serde_json::Value, &str)> = crate::animatable::declared(element)
        .into_iter()
        .map(|declared| (declared.path.clone(), declared.owner, declared.key()))
        .collect();
    if let Some(block) = element.get("units") {
        for property in crate::units::LISTS {
            out.push((format!("units.{property}"), block, property));
        }
    }
    for (r, run) in crate::verbs::measure::runs_array(element)
        .iter()
        .enumerate()
    {
        if let Some(over) = run.get("unit") {
            for property in crate::units::LISTS {
                out.push((format!("runs[{r}].unit.{property}"), over, property));
            }
        }
    }
    out
}

/// One property's keyframe records, or `None` where the property is absent or static.
///
/// [`crate::animatable::records`], which holds ADR-0012's shape test. The properties a check
/// walks are [`crate::animatable::declared`] — the one list, read off the schema (ADR-0146)
/// — and no check keeps a copy of its own.
pub(crate) fn keyframe_records<'a>(
    element: &'a serde_json::Value,
    property: &str,
) -> Option<&'a Vec<serde_json::Value>> {
    crate::animatable::records(element, property)
}

/// The properties whose keyframes move an element's box, and so whose record times are
/// boundaries a geometric check must sample at.
///
/// The animatable list ([`crate::animatable::names`]), narrowed to what changes a rectangle:
/// the placement, the transform, and a shape's keyed `width`/`height` (ADR-0146 §6 — the
/// off-canvas check resolves a keyed box as it resolves keyed `x` and `y`). Paint, `opacity`
/// and `volume` are dropped because none of them moves a rectangle. `opacity` in particular
/// is deliberate — it changes what a collision *looks like* and not whether there is one
/// (ADR-0060), and an element faded to nothing is still somewhere (ADR-0044). A test holds
/// this list inside the animatable one. The projection's three fields move a projected
/// element's quadrilateral as `rotation` moves its box (ADR-0167 §6).
pub const MOVES_THE_BOX: [&str; 9] = [
    "x",
    "y",
    "width",
    "height",
    "scale",
    "rotation",
    "swivel",
    "tilt",
    "perspective",
];

/// **ADR-0060's sample set**: every keyframe boundary inside `window`, its own two ends,
/// and the midpoint of every consecutive pair — over every element in `elements`.
///
/// Shared by [`tie`] and [`canvas`], which ask two different questions of the same
/// arithmetic: *do these two boxes ever meet* and *does this box ever meet the frame*.
/// ADR-0060 wrote the set down and ADR-0044 names no set of its own, so a second copy
/// would be a second answer to *"which instants is a rectangle worth looking at"* — the
/// drift [`crate::stack`] and [`crate::track`] are each owned centrally to prevent.
///
/// The window's last *instant* is `end - 1` and not `end`: the range is half-open, so at
/// `end` the element is already off screen (ADR-0005).
///
/// Between two boundaries each box travels monotonically along a single eased segment, so
/// one interior sample is what catches a crossing that begins and ends elsewhere. It is a
/// sample set and not a proof, which is the trade ADR-0060 made when it chose sampling
/// over a static check.
///
/// Off-grid keyframe times need nothing here (ADR-0035): a `t` is an ordinary integer
/// input to an ordinary sample set, and there is no branch below that could round one.
pub(crate) fn box_samples(
    elements: &[&serde_json::Value],
    window: crate::stack::TimelineRange,
) -> Vec<i64> {
    use serde_json::Value;

    let last = window.end - 1;
    let mut boundaries = vec![window.start, last];
    for element in elements {
        for key in MOVES_THE_BOX {
            let Some(records) = element.get(key).and_then(Value::as_array) else {
                continue;
            };
            for t in records
                .iter()
                .filter_map(|record| record.get("t")?.as_i64())
            {
                // A keyframe outside the element's own range is legal and ordinary — it is
                // how a trimmed move is spelled (`CONTEXT.md`) — and outside the window it
                // is an instant at which the question being asked cannot matter.
                if window.start < t && t <= last {
                    boundaries.push(t);
                }
            }
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    // The interval sample the ADR asks for, between each consecutive pair. Floored, so it
    // is a real instant on the clock; where two boundaries are adjacent it lands on the
    // earlier one and `dedup` drops it, which is correct — there is no instant between
    // them to look at.
    let mut out = Vec::with_capacity(boundaries.len() * 2);
    for pair in boundaries.windows(2) {
        out.push(pair[0]);
        out.push(pair[0] + (pair[1] - pair[0]) / 2);
    }
    out.push(*boundaries.last().expect("the window has at least one end"));
    out.dedup();
    out
}

/// `1 element` / `3 elements` — a count and its noun, for a template that cannot inflect.
pub(crate) fn pluralised(count: usize, noun: &str) -> String {
    match count {
        1 => format!("1 {noun}"),
        count => format!("{count} {noun}s"),
    }
}
