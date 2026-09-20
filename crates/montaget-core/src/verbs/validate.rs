//! `validate` — *"is this project file internally legal, and does it agree with the
//! media on disk?"* (ADR-0006).
//!
//! Both halves now run. The document is read, or one `E-PARSE`/`E-READ` finding comes back
//! and nothing is partially processed; the file is confirmed to be a project at all
//! (ADR-0042), or one `E-NOT-A-PROJECT` finding comes back instead; then every registered
//! check runs over the whole project, and every source it references is probed.
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

use crate::finding::Finding;
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

    if let Err(not_a_project) = document.shape() {
        // ADR-0042's precondition, which `fmt`, `timeline` and `query` already hold and
        // `validate` did not — because until #244 nothing downstream noticed. The gap the
        // ADR found was message quality: a `validate` pointed at a transcript export should
        // *"name the likely mismatch, not dump a raw schema error"*, and a schema check that
        // has just learned to speak would do exactly that — "the project does not fit the
        // published schema: missing field `frame`" — about a file that was never a project.
        //
        // What a verb does about the failure is the verb's own business, and this is
        // `validate`'s answer: the three other verbs' answer, for the three other verbs'
        // reason. It does not narrow what is analysed on a *project*, which is what ADR-0006
        // forbids; it declines to analyse something that is not one.
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point validate at the project file",
        ));
        return report;
    }

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
    // ADR-0017's closed schema, turned into findings (#244). Not gating: every other check
    // still runs on a document that does not fit the types, because every other check reads
    // the permissive tree and each of them has something true to say about a file mid-edit.
    // What changes is that the file no longer validates *clean* while carrying a key the
    // format does not publish — which is ADR-0016's whole migration mechanism, and was
    // unfired until this call existed.
    crate::checks::schema::check(document, report);
    crate::checks::retired::check(document, report);
    crate::checks::anchor::check(document, report);
    // ADR-0060's layer tie (#209): two elements resolving to one layer whose boxes
    // actually overlap, in both time and space. It reads the same `crate::stack`
    // resolution the anchor check above does — one rule, asked twice, never implemented
    // twice — and samples geometry across the pair's shared time range, which is why it is
    // its own check and not a branch of that one.
    crate::checks::tie::check(document, report);
    // The checks that read the clock (#197). Three questions and one traversal each: two
    // elements of one track sharing an instant is the rule tracks exist to enforce
    // (ADR-0004); a gap is reported apart from an overlap and is never an error
    // (ADR-0006); and the source/timeline invariant is evaluated in exact rational
    // arithmetic, never `f64` (ADR-0045).
    crate::checks::track::check(document, report);
    crate::checks::speed::check(document, report);
    // The cross-track coverage question the track check cannot see (ADR-0018, #200):
    // group-scoped pairing, not a frame-wide union — see `crate::checks::coverage`.
    crate::checks::coverage::check(document, report);
    // The four caption checks (#199). Every one is a pure document read — ADR-0054 says so
    // of the one that looks like it needs the disk — which is why this call takes no
    // session and can sit anywhere in this list.
    crate::checks::caption::check(document, report);
    // `R-BOX-SLACK` (#201, ADR-0058): a declared text-box `height` that overshoots the
    // computed block height beyond `max(2px, 10%)`. Height-only arithmetic on fields the
    // document already carries — no font, no I/O — so this too can sit anywhere in the
    // list.
    crate::checks::box_slack::check(document, report);
    // `E-HIGHLIGHT-RANGE`/`E-HIGHLIGHT-OVERLAP` (#202, ADR-0051): a per-word `highlight`
    // window out of its parent run's own range, or overlapping a sibling's. And
    // `E-TRANSITION-RANGE` (#202, ADR-0059): a transition's derived range drifted from
    // its two bridged elements. All three read only the document, so — like the checks
    // above — they can sit anywhere in this list.
    crate::checks::highlight::check(document, report);
    crate::checks::transition::check(document, report);
    // Not "which boundaries are off the grid" — which the fixture answers 109 times — but
    // what the grid actually changes, which on a correct project is nothing (ADR-0006).
    crate::checks::quantization::check(document, report);
    // ADR-0041: checked here **unconditionally**, and `fmt --check`-only was rejected
    // outright — the agent that pretty-printed the fixture from 154 lines to 1595 was not
    // running a formatter and had no reason to invoke one, while `validate` runs on files
    // nobody ever ran `fmt` over. Its findings are `LAYOUT`, which is not a severity: they
    // never gate a render, because the video is byte-identical either way.
    crate::checks::layout::check(document, report);
    // ADR-0057's two attestation checks (#207): every `fonts`-table path resolves to a
    // `fontVendor` entry whose hash matches the bytes on disk, and every entry is still
    // referenced. Reads the disk — the font files — but needs no subprocess, so it sits
    // here rather than behind the session below, and it runs whether or not the project
    // references any media.
    crate::checks::fonts::check(document, report);

    // The two checks that need a subprocess, and the only ones that can fail rather than
    // find. Whether one needs to be opened at all is decided once, here, rather than per
    // check: `crate::checks::fit` needs the identical session `crate::checks::source`
    // does, and a project referencing no media has nothing to ask either of them.
    if !document.elements().any(|e| e["source"].is_string()) {
        return Ok(());
    }
    match session {
        Some(session) => run_disk_checks(document, session, report),
        None => {
            let mut session = Session::open().map_err(Box::new)?;
            session.begin_run();
            run_disk_checks(document, &mut session, report)
        }
    }
}

/// The checks that read the disk, over the one session both share.
///
/// `source` first: it is what populates `report.misses`, and `fit` re-probing the same
/// sources afterwards is a cache hit that adds none — running them in the other order
/// would leave `fit`'s partial view overwriting `source`'s complete one.
fn run_disk_checks(
    document: &Loose,
    session: &mut Session,
    report: &mut Report,
) -> Result<(), Box<Missing>> {
    crate::checks::source::check(document, session, report)?;
    crate::checks::fit::check(document, session, report)
}
