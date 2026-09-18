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
/// Empty by design: #188 builds the spine, and each later check ticket adds its own call
/// here. The function exists now so that "where does a check go?" has one answer before
/// the first check is written.
fn run_checks(_document: &crate::document::Document, _report: &mut Report) {}
