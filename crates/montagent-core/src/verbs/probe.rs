//! `probe` — *"the one authority on what a media file's numbers are"* (ADR-0011).
//!
//! CLI-only, by ADR-0011's cost model: *"every MCP tool schema occupies the agent's
//! context and degrades tool selection on every turn, including turns with nothing to do
//! with video. A CLI subcommand costs nothing until invoked."*
//!
//! It answers with **a report and nothing of its own but one number**. The cache misses,
//! the media facts and the findings all live on [`crate::report::Report`], which is where
//! ADR-0006 specified the first of them and where `validate` reads the other two from — so
//! there is one wire shape across the surface rather than a verb-shaped variant, and one
//! renderer rather than two that can disagree about the order they print in.
//!
//! The number is `network_attempts`: how many times the run reached for the network, which
//! is `probe`'s alone because it is the only verb a user points at a URL directly.

use std::path::Path;

use serde_json::{Value, json};

use crate::media::Source;
use crate::media::session::Session;
use crate::report::Report;

const TOOL: &str = "probe";

/// One `probe` invocation's answer.
pub struct Answer {
    /// How many times the run attempted to reach the network. Zero for a project of
    /// local sources, and the number is in the output so that is checkable rather than
    /// promised (ADR-0056).
    pub network_attempts: usize,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The canonical JSON: the report's own object, plus the two things only `probe`
    /// produces. One object per invocation, like every other verb.
    pub fn to_json(&self) -> Value {
        self.report
            .to_json_with("network_attempts", json!(self.network_attempts))
    }
}

/// Probe every source named, resolving a relative one against the working directory.
///
/// The error is ADR-0011's exit 70 and nothing else: there is no `ffprobe`, so there are
/// no media numbers to be right or wrong about, and saying anything about the project
/// would be inventing it.
pub fn probe(sources: &[String]) -> Result<Answer, Box<Report>> {
    let mut session = Session::open().map_err(|missing| Box::new(missing.into_report()))?;
    session.begin_run();
    // The base a bare `probe` resolves against is the working directory, and the adapter
    // does not get to decide that: what a relative `source` resolves against is ADR-0053's
    // question, and it is answered in the core (ADR-0011 — an adapter contains no rule).
    probe_with(&mut session, Path::new("."), sources)
}

/// The same, against a session and a base directory the caller owns — an MCP server's warm
/// cache, a project file's own directory (ADR-0053), or a test's recorded `ffprobe`.
pub fn probe_with(
    session: &mut Session,
    base: &Path,
    sources: &[String],
) -> Result<Answer, Box<Report>> {
    let mut report = Report::new(TOOL, None);

    for source in sources {
        let resolved = Source::resolve(source, base);
        match session.probe(&resolved) {
            Ok(outcome) => crate::media::probe::record(&outcome, source, &mut report),
            // The tool is broken, so nothing after this point can be established — and
            // the facts gathered before it are not an answer to what was asked. This
            // leaves by the same door a bad invocation does: one report, exit 70, and no
            // half-answer printed beside it (ADR-0011 — "nothing may partially process").
            Err(missing) => return Err(Box::new(missing.into_report())),
        }
    }

    report.misses = session.misses().to_vec();
    Ok(Answer {
        network_attempts: session.network_attempts(),
        report,
    })
}
