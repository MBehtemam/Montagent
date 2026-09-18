//! `validate` — *"is this project file internally legal, and does it agree with the
//! media on disk?"* (ADR-0006).
//!
//! **What runs today is the parse layer and nothing else** (#188). The document is read,
//! or one `E-PARSE`/`E-READ` finding comes back and nothing is partially processed. The
//! checks the ADR series specifies are later tickets, and each one appends findings to
//! the report this function builds.
//!
//! Two properties of `validate` are structural and are settled here rather than by any
//! individual check (ADR-0006): it **always runs every check on the whole project** —
//! there is no fast mode, no `--no-probe` and no way to narrow what is analysed — and its
//! report always ends with its own boundary. Output may be filtered; analysis may not.

use std::path::Path;

use crate::parse;
use crate::report::Report;

/// The verb's own name, as it travels in the report. One spelling, so the two exits
/// from the function below cannot disagree about which tool answered.
const TOOL: &str = "validate";

/// Validate the project file at `path`.
pub fn validate(path: &Path) -> Report {
    let project = Some(path.display().to_string());

    let document = match parse::read(path) {
        Ok(document) => document,
        Err(finding) => return Report::unparseable(TOOL, project, *finding),
    };

    let mut report = Report::new(TOOL, project);
    run_checks(&document, &mut report);
    report
}

/// Every registered check, over the whole document.
///
/// One call per check, in no significant order: a report's findings are a set of facts
/// about the project, and nothing downstream may depend on which check spoke first.
fn run_checks(document: &crate::permissive::Loose, report: &mut Report) {
    crate::checks::retired::check(document, report);
}
