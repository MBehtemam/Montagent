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
pub mod box_slack;
pub mod caption;
pub mod coverage;
pub mod fit;
pub mod fonts;
pub mod highlight;
pub mod layout;
pub mod quantization;
pub mod retired;
pub mod runs;
pub mod schema;
pub mod source;
pub mod speed;
pub mod tie;
pub mod track;
pub mod transition;

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

/// `1 element` / `3 elements` — a count and its noun, for a template that cannot inflect.
pub(crate) fn pluralised(count: usize, noun: &str) -> String {
    match count {
        1 => format!("1 {noun}"),
        count => format!("{count} {noun}s"),
    }
}
