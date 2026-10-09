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
//!   git is tracking. An existing path is `E-PROJECT-EXISTS` and nothing is written, at
//!   **exit 3**: the project is not what needs fixing, the path argument is (ADR-0080).
//! - **It never invents a value the agent did not state.** ADR-0030: omission and
//!   explicit-at-default are two spellings of *different declarations*, so Montagent writing
//!   `"background": "#000000"` on the agent's behalf would be authoring a declaration
//!   nobody made. `background`, `duration` and `output` are offered as arguments — which is
//!   what story 1 of spec #168 is asking for, a file whose shape the agent does not have to
//!   invent — and appear in the file exactly when they were asked for. This departs from
//!   #194's enumeration of five keys, and ADR-0080 is what settles it: the question
//!   ADR-0030 left open is closed in favour of the three keys staying optional, and the
//!   ticket's five-key sentence is read as naming the header's shape rather than the
//!   scaffold's output.
//! - **It decides nothing about canonical form.** The bytes come from
//!   [`crate::write::canonical`], and their key order comes from serialising
//!   [`crate::model::Project`], whose field order *is* canonical key order (ADR-0041).
//!   There is no second ordering here to fall out of step with the schema.
//!
//! The write itself is ticket 6's machinery unchanged: [`crate::write::atomically`], which
//! replaces the file's bytes or leaves it exactly as it was.

use std::path::Path;

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::model::{Colour, Frame, Project};
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
    // Exit 3 and not 1 (ADR-0080): the file that is there is intact and the project being
    // scaffolded does not exist, so there is no document to fix — what needs changing is
    // the path this call was given, which is exit 3's *"fix the command"* exactly. The
    // remedy is the registry template's own sentence rather than a `repair` field, because
    // the code is `NotAboutDocument`.
    //
    // A window stays open between here and the rename below, which replaces
    // unconditionally. Closing it would mean reserving the destination first, which trades
    // the race for a worse failure — an empty file left in place when the write then fails.
    // Ratified as the narrower risk by ADR-0080.
    if path.exists() {
        return Report::refused_invocation(
            TOOL,
            project,
            Finding::new("E-PROJECT-EXISTS").at_file(path.display().to_string()),
        );
    }

    let header = match header(scaffold) {
        Ok(header) => header,
        Err(reason) => return Report::rejected(TOOL, project, reason),
    };

    // A missing parent is the agent's path argument being wrong, not Montagent breaking, and
    // exit 70's *"retry or report"* is the wrong next move for a typo. Every other write
    // failure goes the way `fmt`'s does, below.
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.is_dir()
    {
        return Report::rejected(
            TOOL,
            project,
            format!(
                "`create_project`: {} is not a directory that exists",
                parent.display()
            ),
        );
    }

    if let Err(e) = write::atomically(path, &write::canonical(&header)) {
        let mut report = Report::new(TOOL, project);
        report.could_not_write(path.display(), &e);
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
/// [`Project`] is built as a value rather than as a map of key strings, so the format's
/// field names are spelled once — in the model — rather than again here. Serialising it is
/// what produces canonical key order: struct field order *is* canonical key order
/// (ADR-0041), so a scaffold that assembled its own ordering would be the second place the
/// rule lives.
///
/// `background` is the one field that cannot be built by assignment: [`Colour`] admits
/// `#RRGGBB` and `#RRGGBBAA` uppercase and nothing else, and its check lives in its
/// `Deserialize`. Going through that is the point — the alternative is a second colour
/// predicate here, disagreeing with the first the day one of them is revised.
fn header(scaffold: &Scaffold) -> Result<Value, String> {
    let background = scaffold
        .background
        .as_deref()
        .map(|colour| serde_json::from_value::<Colour>(json!(colour)))
        .transpose()
        .map_err(|e| format!("`create_project`: {e}"))?;

    let project = Project {
        frame: Frame {
            width: scaffold.width,
            height: scaffold.height,
        },
        fps: scaffold.fps,
        background,
        duration: scaffold.duration,
        // ADR-0062's `loop` is not one of the five #194 enumerates, and ADR-0030 means
        // writing it at its default would be a declaration rather than a scaffold.
        looping: None,
        output: scaffold.output.clone(),
        master: None,
        fonts: None,
        font_vendor: None,
        tracks: Vec::new(),
    };
    serde_json::to_value(&project).map_err(|e| format!("`create_project`: {e}"))
}
