//! `validate` — *"is this project file internally legal, and does it agree with the
//! media on disk?"* (ADR-0006).
//!
//! Both halves now run. The document is read, or one `E-PARSE`/`E-READ` finding comes back
//! and nothing is partially processed; then every registered check runs over the whole
//! project, and every source it references is probed.
//!
//! Two properties of `validate` are structural and are settled here rather than by any
//! individual check (ADR-0006): it **always runs every check on the whole project** —
//! there is no fast mode, no `--no-probe` and no way to narrow what is analysed — and its
//! report always ends with its own boundary. Output may be filtered; analysis may not.

use std::path::Path;

use crate::media::session::Session;
use crate::media::tools::Missing;
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;

/// The verb's own name, as it travels in the report. One spelling, so the two exits
/// from the function below cannot disagree about which tool answered.
const TOOL: &str = "validate";

/// Validate the project file at `path`.
pub fn validate(path: &Path) -> Report {
    run(path, None)
}

fn run(path: &Path, session: Option<&mut Session>) -> Report {
    let project = Some(path.display().to_string());

    let document = match parse::read(path) {
        Ok(document) => document,
        Err(finding) => return Report::unparseable(TOOL, project, *finding),
    };

    let mut report = Report::new(TOOL, project);
    if let Err(missing) = run_checks(&document, &mut report, session) {
        // There is no `ffprobe`, so the disk half of the question cannot be asked. ADR-0006
        // forbids answering it with silence, and ADR-0011 gives "Montaget could not run"
        // its own exit code precisely so it is not mistaken for a defect in the project.
        //
        // What the document half already established stays in the report: a run that found
        // a retired spelling and *then* discovered there is no `ffprobe` has learned two
        // things, and an agent told only the second would fix its `PATH`, re-run, and only
        // then hear about the key it could have fixed in the same turn.
        report.fail_internally(missing.reason());
    }
    report
}

/// Every registered check, over the whole document.
///
/// One call per check, in no significant order: a report's findings are a set of facts
/// about the project, and nothing downstream may depend on which check spoke first.
fn run_checks(
    document: &Loose,
    report: &mut Report,
    session: Option<&mut Session>,
) -> Result<(), Box<Missing>> {
    crate::checks::retired::check(document, report);
    // The one check that needs a subprocess, and the only one that can fail rather than
    // find. Whether it needs to open one at all is its own business — it is the thing that
    // knows whether the document references any media.
    crate::checks::source::check(document, session, report)
}
