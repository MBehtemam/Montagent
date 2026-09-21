//! `fmt` — *"rewrite the file in the canonical convention"* (ADR-0011), and the
//! non-destructive `--check` that reports what such a rewrite would change.
//!
//! CLI-only, by ADR-0011's cost model. It exists because the convention is normative and
//! nothing enforced it: ADR-0005 required *"one element per line, stable key order"* and
//! said it is load-bearing **because agents edit by exact-string replace**, and then an
//! agent asked for a routine edit rewrote the 154-line fixture into 1595 lines and reported
//! no schema errors. By the letter of every rule then in force it was correct.
//!
//! Four boundaries, each from an ADR, and each a thing this verb deliberately does *not*
//! do:
//!
//! - **It never adds or removes a defaulted field** (ADR-0030). Omission and
//!   explicit-at-default are two spellings of different declarations.
//! - **It never rewrites a value** (ADR-0026). At an exact aspect match both `cover` and
//!   `contain` are true, and a formatter that picked one would be deciding what the author
//!   meant.
//! - **It refuses only on a document that is not a project at all** (ADR-0042) — wholesale
//!   missing `tracks`/`fps`/`frame`. It stays usable on a file carrying `error` findings,
//!   which is exactly when it is most wanted: mid-authoring.
//! - **It writes atomically or not at all** (ADR-0011). The file is the one the agent is
//!   mid-edit on; a half-written project is worse than an unformatted one.
//!
//! What it *does* is one function's worth of decision — [`crate::layout::canonicalise`] —
//! one writer, [`crate::write::canonical`], and one set of findings,
//! [`crate::checks::layout`]. All three are `validate`'s `LAYOUT` check rather than a
//! parallel copy of it, which is ADR-0041's *"exactly one place the rule lives, not two
//! that can disagree."* This verb decides nothing about canonical form; it writes what the
//! check describes.

use std::path::Path;

use crate::checks;
use crate::finding::Finding;
use crate::parse;
use crate::report::Report;
use crate::write;

const TOOL: &str = "fmt";

/// Which half of ADR-0041's split this invocation is.
///
/// A type rather than a `bool`, because the two modes differ in whether they touch the
/// file — the one distinction a reader of a call site must not have to guess at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Report what a rewrite would change. Writes nothing, ever.
    Check,
    /// Rewrite the file in the canonical convention.
    Write,
}

/// Format the project file at `path`, or report what formatting it would change.
///
/// The report's findings are the same `LAYOUT` findings `validate` produces, from the same
/// predicate — `fmt --check` is a second place to *ask*, never a second rule.
///
/// **A non-canonical file is exit 0**, in both modes — ratified by ADR-0079. ADR-0011
/// states it flatly — *"Exit non-zero only on `error`"* — and ADR-0041 is equally flat that
/// `LAYOUT` is not one: the video renders identically either way, and refusal stays keyed
/// to `error` alone. Story 5's *"verify before I commit"* is satisfied without a sixth
/// code: a caller that wants a hard gate reads the `layout` count out of `--json`, which is
/// exact, the same shape ADR-0013 already gives `UNCHECKED`.
pub fn fmt(path: &Path, mode: Mode) -> Report {
    let project = Some(path.display().to_string());

    let document = match parse::read(path) {
        Ok(document) => document,
        // ADR-0011: nothing may partially process a malformed file. A formatter especially
        // — the bytes it would write are derived from a parse that did not happen.
        Err(finding) => return Report::unparseable(TOOL, project, *finding),
    };

    let mut report = Report::new(TOOL, project);

    if let Err(not_a_project) = document.shape() {
        // ADR-0042's precondition, and the whole of it: structural shape, never severity.
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point fmt at the project file",
        ));
        return report;
    }

    // The bytes the parse read, rather than a second read of the file: the findings below
    // are about the file that was parsed, and a file that changed in between would give
    // them line numbers into a document nobody looked at.
    let written = document.source().unwrap_or_default();
    let canonical = checks::layout::canonical(&document);

    if written == canonical {
        return report;
    }

    // The same findings `validate` produces, from the same function — `fmt --check` is a
    // second place to *ask*, never a second rule (ADR-0041).
    checks::layout::check(&document, &mut report);

    // On success the findings stay in the report — they are what the run changed, and an
    // agent that asked for a rewrite is owed the list.
    if mode == Mode::Write
        && let Err(e) = write::atomically(path, &canonical)
    {
        report.could_not_write(document.path(), &e);
    }

    report
}
