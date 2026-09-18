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
//!
//! One of the checks answers a question that is not about the video at all. ADR-0041's
//! `LAYOUT` is a fourth report category rather than a severity: a file written outside the
//! canonical convention is *"unsafe to edit, not unsafe to render"*, so it is reported on
//! every run and gates nothing.

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
///
/// The session — and with it the probe cache and its sidecar — is this call's own, which
/// is the CLI's shape: one process, one run, and the sidecar is what carries the cache
/// across to the next one (ADR-0069).
pub fn validate(path: &Path) -> Report {
    run(path, None)
}

/// The same, against a session the caller owns — an MCP server's warm cache (ADR-0011), or
/// a test's recorded `ffprobe`. The remote half is cleared here, because this is one run.
pub fn validate_with(path: &Path, session: &mut Session) -> Report {
    session.begin_run();
    run(path, Some(session))
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
    crate::checks::anchor::check(document, report);
    // ADR-0041: checked here **unconditionally**, and `fmt --check`-only was rejected
    // outright — the agent that pretty-printed the fixture from 154 lines to 1595 was not
    // running a formatter and had no reason to invoke one, while `validate` runs on files
    // nobody ever ran `fmt` over. Its findings are `LAYOUT`, which is not a severity: they
    // never gate a render, because the video is byte-identical either way.
    crate::checks::layout::check(document, report);
    // The one check that needs a subprocess, and the only one that can fail rather than
    // find. Whether it needs to open one at all is its own business — it is the thing that
    // knows whether the document references any media.
    crate::checks::source::check(document, session, report)
}
