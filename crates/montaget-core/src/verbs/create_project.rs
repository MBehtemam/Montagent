//! `create_project` — *"scaffold a legal file so the agent never starts blank"* (ADR-0011).
//!
//! The first write tool, and the place the write-tool invariant stops being a sentence in
//! an ADR and becomes something a test can observe: **every write tool returns the new
//! state's findings, never `ok`** — ADR-0006's second structural mechanism, restated by
//! ADR-0011 as a constraint on every write tool it defines. *"Then I don't run `validate`;
//! `validate` runs me."* So the report this hands back is not a receipt for the write; it
//! is `validate`'s answer about the file that now exists, under this verb's name.
//!
//! Three things it deliberately does not do:
//!
//! - **It never overwrites.** A scaffold that clobbers is a scaffold that deletes a
//!   project, and the file it would land on is the one the agent is mid-edit on and the one
//!   git is tracking. An existing path is `E-PROJECT-EXISTS` and nothing is written.
//! - **It never invents a value the agent did not state.** ADR-0030: omission and
//!   explicit-at-default are two spellings of *different declarations*, so Montaget writing
//!   `"background": "#000000"` on the agent's behalf would be authoring a declaration
//!   nobody made. `background`, `duration` and `output` are offered as arguments — which is
//!   what story 1 of spec #168 is asking for, a file whose shape the agent does not have to
//!   invent — and appear in the file exactly when they were asked for. ADR-0030 leaves this
//!   one open in as many words, and #194 reads the other way; raised as #246 rather than
//!   left to be discovered from the struct.
//! - **It decides nothing about canonical form.** The bytes come from
//!   [`crate::write::canonical`], and their key order comes from serialising
//!   [`crate::model::Project`], whose field order *is* canonical key order (ADR-0041).
//!   There is no second ordering here to fall out of step with the schema.
//!
//! The write itself is ticket 6's machinery unchanged: [`crate::write::atomically`], which
//! replaces the file's bytes or leaves it exactly as it was.

use std::path::Path;

use serde_json::{Map, Value, json};

use crate::finding::Finding;
use crate::model::Project;
use crate::report::Report;
use crate::write;

const TOOL: &str = "create_project";

/// What the agent is asking for, as the header of the project it wants.
///
/// `frame` and `fps` are the two the format requires and are therefore not optional here
/// either. The other three are optional for ADR-0030's reason, not for convenience: a
/// scaffold that wrote them unasked would be stating an authorial declaration — *"I have
/// pinned this"* — on behalf of an author who said nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct Scaffold {
    pub width: i64,
    pub height: i64,
    pub fps: i64,
    /// `#RRGGBB` or `#RRGGBBAA`, uppercase. Checked by [`crate::model::Colour`] before
    /// anything is written, so a bad colour is a bad invocation rather than a file that
    /// has to be repaired after the fact.
    pub background: Option<String>,
    pub duration: Option<i64>,
    pub output: Option<String>,
}

/// Scaffold a project file at `path`, and report what `validate` then says about it.
///
/// The returned report's exit code is the new file's, not the write's: a scaffold that
/// landed cleanly is exit 0 because the project is clean, and the two are the same
/// statement by construction.
pub fn create_project(path: &Path, scaffold: &Scaffold) -> Report {
    let project = Some(path.display().to_string());

    // Checked before the header is even assembled: the one thing worth knowing about a
    // path that already exists is that nothing should happen to it.
    //
    // A window stays open between here and the rename below, which replaces
    // unconditionally. Closing it would mean reserving the destination first, which trades
    // the race for a worse failure — an empty file left in place when the write then fails.
    // Recorded in #246 rather than papered over.
    if path.exists() {
        let mut report = Report::new(TOOL, project);
        report.push(
            Finding::new("E-PROJECT-EXISTS")
                .at_file(path.display().to_string())
                .repair_value(json!({"value": "scaffold at a path that does not exist yet"})),
        );
        return report;
    }

    let header = match header(scaffold) {
        Ok(header) => header,
        Err(reason) => return bad_invocation(reason, project),
    };

    // A missing parent is the agent's path argument being wrong, not Montaget breaking, and
    // exit 70's *"retry or report"* is the wrong next move for a typo. Every other write
    // failure goes the way `fmt`'s does, below.
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.is_dir()
    {
        return bad_invocation(
            format!(
                "`create_project`: {} is not a directory that exists",
                parent.display()
            ),
            project,
        );
    }

    if let Err(e) = write::atomically(path, &write::canonical(&header)) {
        // `fmt`'s choice, for `fmt`'s reason: the file is untouched, so there is nothing
        // about the *project* to report, and ADR-0011 gives "Montaget could not run" its
        // own exit code precisely so it is not mistaken for a defect in the document.
        let mut report = Report::new(TOOL, project);
        report.fail_internally(format!("{} could not be written: {e}", path.display()));
        return report;
    }

    // The write-tool invariant, and the whole reason this verb answers with a report at
    // all: the new state's findings, never `ok`.
    let mut report = crate::verbs::validate::validate(path);
    report.tool = TOOL.to_string();
    report
}

/// The scaffolded header, in canonical key order, or why it could not be built.
///
/// Order is not stated here. The map below is assembled in whatever order reads well, then
/// round-tripped through [`Project`] — which both rejects a value the format does not admit
/// (a lowercase colour, a CSS name) and re-emits the keys in struct-field order, and struct
/// field order *is* canonical key order (ADR-0041). A scaffold that wrote its own ordering
/// would be the second place the rule lives.
fn header(scaffold: &Scaffold) -> Result<Value, String> {
    let mut header = Map::new();
    header.insert(
        "frame".to_string(),
        json!({"width": scaffold.width, "height": scaffold.height}),
    );
    header.insert("fps".to_string(), json!(scaffold.fps));
    if let Some(background) = &scaffold.background {
        header.insert("background".to_string(), json!(background));
    }
    if let Some(duration) = scaffold.duration {
        header.insert("duration".to_string(), json!(duration));
    }
    if let Some(output) = &scaffold.output {
        header.insert("output".to_string(), json!(output));
    }
    header.insert("tracks".to_string(), json!([]));

    let project: Project = serde_json::from_value(Value::Object(header))
        .map_err(|e| format!("`create_project`: {e}"))?;
    serde_json::to_value(&project).map_err(|e| format!("`create_project`: {e}"))
}

/// ADR-0011's exit 3, wearing this verb's name.
///
/// [`Report::bad_invocation`] is argv's shape — no tool, no project — because that is where
/// the CLI raises it. A tool call that names a project and gets an argument wrong is the
/// same finding about a known file, and saying so costs nothing.
fn bad_invocation(reason: String, project: Option<String>) -> Report {
    let mut report = Report::bad_invocation(reason);
    report.tool = TOOL.to_string();
    report.project = project;
    report
}
