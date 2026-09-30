//! The report surface: a canonical JSON finding set, its summary, its exit code, and
//! the boundary it prints on every run.

use serde_json::{Value, json};

use crate::finding::{Class, Finding};
use crate::media::probe::Probe;
use crate::media::session::CacheMiss;
use crate::registry::{self, CheckSet, RepairClass};

/// ADR-0006's `NOT CHECKED` block, verbatim.
///
/// *"Without it, a clean run is read as 'the file is right' — and 'run this and the
/// file is fine' is the `sequence` label again, wearing a `validate` label instead."*
pub const NOT_CHECKED: &str = "This file was not compared against any prior version or instruction. validate \
verifies that the file is internally legal; it cannot tell you whether it says what you \
meant it to say.";

/// Add a verb's own limits to a report's boundary, under `not_checked_also` (ADR-0117).
///
/// ADR-0006's block is the same sentence on every report, and stays so: this does not edit
/// it. A verb whose answer has limits of its own — `verify`'s one mixed track, `render`'s
/// deliverable that nothing has yet measured — states them beside it, as a list that is
/// absent rather than empty on every other verb, so no report grows a key it has nothing to
/// put in.
pub fn extend_boundary(json: &mut Value, lines: &[&str]) {
    let Some(object) = json.as_object_mut() else {
        return;
    };
    let list = object
        .entry("not_checked_also")
        .or_insert_with(|| Value::Array(Vec::new()));
    if let Value::Array(list) = list {
        list.extend(lines.iter().map(|line| Value::String(line.to_string())));
    }
}

/// ADR-0011's five exit codes, distinguished by what the caller does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    /// No `error` findings — proceed.
    Ok = 0,
    /// `error` findings — fix the project.
    Errors = 1,
    /// The file could not be read or parsed — fix the *file*.
    Unparseable = 2,
    /// The invocation was wrong — fix the command.
    BadInvocation = 3,
    /// Internal failure — retry or report.
    Internal = 70,
}

impl ExitCode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// How a run ended, where that is not simply "it ran".
///
/// Held separately from the finding list so the exit code is a property of the run
/// rather than something re-derived by sniffing codes: `E-PARSE` and `E-SOURCE-OVERRUN`
/// are both `error`-class findings, and 1 and 2 must not collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminal {
    Unparseable,
    BadInvocation,
    Internal,
}

/// How many findings of each class the run produced.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Summary {
    pub error: usize,
    pub review: usize,
    pub note: usize,
    pub unchecked: usize,
    pub layout: usize,
    pub drift: usize,
}

/// One verb's answer.
#[derive(Debug, Clone)]
pub struct Report {
    pub tool: String,
    pub project: Option<String>,
    pub findings: Vec<Finding>,
    /// Every probe that actually ran, rather than being answered from the cache.
    ///
    /// On the report rather than on any one verb's answer, because ADR-0006 puts it there:
    /// *"then report the cache miss, unprompted, at the top"* — of `validate`'s report,
    /// which is where it was specified before `probe` existed to share it. ADR-0011 then
    /// makes it load-bearing rather than incidental: the document records no source
    /// duration, so this line is *"the sole mechanism"* announcing a source that grew on
    /// disk. It is never behind a flag and never collapses into a count.
    pub misses: Vec<CacheMiss>,
    /// What the disk said about each source the run probed, one entry per distinct source.
    ///
    /// The facts themselves, not a judgement about them: ADR-0006's *"`validate` reports
    /// facts"*, and #203's *"every referenced source is probed and its real duration and
    /// dimensions reported"*. They are carried whether or not any check fired, because a
    /// clean run that established the numbers and then printed none of them would leave
    /// the reader to re-derive them — and because the fit checks downstream consume exactly
    /// these dimensions rather than probing a second time.
    pub media: Vec<Probe>,
    /// The check sets that ran to completion, in the order they completed. ADR-0112.
    ///
    /// Private, and only [`Report::record`] adds to it: the field is marked by the code
    /// that runs a set, when the set completes, and never declared per verb. A new report
    /// starts at `[]`, which is the safe direction — a verb that forgets to record reads
    /// *"no checks run"* rather than claiming a run it never made.
    check_sets: Vec<CheckSet>,
    terminal: Option<Terminal>,
}

impl Report {
    pub fn new(tool: impl Into<String>, project: Option<String>) -> Self {
        Report {
            tool: tool.into(),
            project,
            findings: Vec::new(),
            misses: Vec::new(),
            media: Vec::new(),
            check_sets: Vec::new(),
            terminal: None,
        }
    }

    /// The file could not be read or parsed. ADR-0011: *"Nothing may partially process a
    /// malformed file"* — so this report carries the one `E-PARSE` finding and nothing
    /// that a partial parse might have produced.
    pub fn unparseable(tool: impl Into<String>, project: Option<String>, finding: Finding) -> Self {
        Report {
            tool: tool.into(),
            project,
            findings: vec![finding],
            misses: Vec::new(),
            media: Vec::new(),
            check_sets: Vec::new(),
            terminal: Some(Terminal::Unparseable),
        }
    }

    /// The invocation was wrong. An invocation error is a finding like any other, so
    /// there is exactly one thing to parse across the surface (ADR-0011).
    pub fn bad_invocation(reason: impl Into<String>) -> Self {
        Report {
            tool: "montagent".into(),
            project: None,
            findings: vec![
                // ADR-0073: not about a document — no structured repair. The usage text
                // in `reason` already states the fix ("fix the command"); nothing here
                // asks the registry's `NotAboutDocument` declaration for one.
                Finding::new("E-INVOCATION").field("reason", Value::String(reason.into())),
            ],
            misses: Vec::new(),
            media: Vec::new(),
            check_sets: Vec::new(),
            terminal: Some(Terminal::BadInvocation),
        }
    }

    /// One verb rejected its own arguments.
    ///
    /// [`Report::bad_invocation`] is argv's shape — no tool, no project — because that is
    /// where the CLI raises it. A verb that names a project and gets an argument wrong is
    /// the same finding about a known file, with the same code, class and exit: ADR-0011's
    /// *"an error is a finding... so there is exactly one thing to parse across the
    /// surface."* Which combinations of a verb's flags are legal is a rule about the verb,
    /// so it is enforced in the verb rather than twice in the two adapters, and this is how
    /// that answer gets back out with exit 3 intact.
    pub fn rejected(
        tool: impl Into<String>,
        project: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        // Built from the one above rather than beside it: a second spelling of the finding
        // is a second place its code, class and repair could drift.
        let mut report = Report::bad_invocation(reason);
        report.tool = tool.into();
        report.project = project;
        report
    }

    /// The invocation was wrong, and the verb has a registered code of its own for why.
    ///
    /// [`Report::rejected`] is the general case and its finding is always `E-INVOCATION`:
    /// the verb got an argument wrong, and the usage text is the whole of the answer. This
    /// is the narrower one — a condition the verb names with its own code, carrying the
    /// fields that code's template reads. The two share exit 3 because they share a next
    /// move: *"fix the command"* (ADR-0011). `E-PROJECT-EXISTS` is its only inhabitant
    /// (ADR-0080).
    ///
    /// Named `refused_invocation` rather than `refused` because `verbs::render` and
    /// `verbs::preview` each already have a free `refused(Report) -> Answer`, which wraps a
    /// report rather than building one. Two different meanings under one word, one `use`
    /// away from each other, is a name that has to be read twice.
    ///
    /// A code reaching exit 3 this way must be declared `NotAboutDocument` in the
    /// registry, and that is asserted here rather than left to [`Report::push`].
    /// `push` holds every finding to ADR-0043's repair invariant as narrowed by
    /// ADR-0073, but it is satisfied by *either* an exempt declaration or a repair
    /// value — so an `Advise` code that states its repair would pass it and still
    /// arrive at exit 3 carrying a `repair` field, which is what ADR-0073 forbids
    /// there. The declaration is the thing this constructor depends on, so the
    /// declaration is what it checks.
    #[track_caller]
    pub fn refused_invocation(
        tool: impl Into<String>,
        project: Option<String>,
        finding: Finding,
    ) -> Self {
        assert!(
            matches!(
                registry::spec(&finding.code).and_then(|spec| spec.repair),
                Some(RepairClass::NotAboutDocument)
            ),
            "{} reaches exit 3 but is not declared `NotAboutDocument` (ADR-0073/ADR-0080)",
            finding.code
        );
        let mut report = Report::new(tool, project);
        report.terminal = Some(Terminal::BadInvocation);
        report.push(finding);
        report
    }

    /// Montagent itself failed — ffmpeg died, the font stack failed.
    pub fn internal_failure(reason: impl Into<String>) -> Self {
        Report {
            tool: "montagent".into(),
            project: None,
            findings: vec![
                // ADR-0073: `NotAboutDocument` comes from the registry; nothing here
                // asks for a repair.
                Finding::new("E-INTERNAL").field("reason", Value::String(reason.into())),
            ],
            misses: Vec::new(),
            media: Vec::new(),
            check_sets: Vec::new(),
            terminal: Some(Terminal::Internal),
        }
    }

    /// `ffmpeg` or `ffprobe` was never on `PATH`. ADR-0009 makes supplying one the user's
    /// own job, so this is an unconfigured environment, not Montagent breaking — distinct
    /// from [`Report::internal_failure`], which stays for a resolved binary that would
    /// not run (ADR-0091).
    pub fn tool_missing(reason: impl Into<String>) -> Self {
        Report {
            tool: "montagent".into(),
            project: None,
            findings: vec![
                // ADR-0073: `NotAboutDocument` comes from the registry; nothing here
                // asks for a repair.
                Finding::new("E-TOOL-MISSING").field("reason", Value::String(reason.into())),
            ],
            misses: Vec::new(),
            media: Vec::new(),
            check_sets: Vec::new(),
            // Same exit code as `E-INTERNAL` (ADR-0011, unchanged by ADR-0091): the run
            // did not finish either way.
            terminal: Some(Terminal::Internal),
        }
    }

    /// Montagent failed part-way through a run that had already established facts.
    ///
    /// The findings already in the report stay in it. A run that read the document, found
    /// a retired spelling, and *then* discovered there is no `ffprobe` has learned two
    /// things, and replacing the report with the second would throw away the first —
    /// leaving an agent to fix its `PATH`, re-run, and only then be told about the key it
    /// could have fixed in the same turn. The exit code still says 70, because the run did
    /// not finish (ADR-0011).
    pub fn fail_internally(&mut self, reason: impl Into<String>) {
        self.findings
            .push(Finding::new("E-INTERNAL").field("reason", Value::String(reason.into())));
        self.terminal = Some(Terminal::Internal);
    }

    /// The same, for a run that discovers a missing `ffmpeg`/`ffprobe` after it has
    /// already established facts. See [`Report::tool_missing`] for why this is not
    /// `E-INTERNAL` (ADR-0091).
    pub fn fail_tool_missing(&mut self, reason: impl Into<String>) {
        self.findings
            .push(Finding::new("E-TOOL-MISSING").field("reason", Value::String(reason.into())));
        self.terminal = Some(Terminal::Internal);
    }

    /// `ffmpeg` was found and ran, and cannot do what Montagent asks of it: it failed the
    /// tool qualification (ADR-0115). A sibling of [`Report::fail_tool_missing`] with its own
    /// code, because the next move is to upgrade rather than to install.
    pub fn fail_tool_unsupported(&mut self, reason: impl Into<String>) {
        self.findings
            .push(Finding::new("E-TOOL-UNSUPPORTED").field("reason", Value::String(reason.into())));
        self.terminal = Some(Terminal::Internal);
    }

    /// ADR-0109: the caller cancelled the call, and nothing was published.
    ///
    /// `NotAboutDocument`, on ADR-0091's reasoning for `E-TOOL-MISSING`: the project is not
    /// what stopped the run, so no finding about it may be implied. The sentence names what
    /// the disk holds, because that is the whole of what ADR-0109 decides.
    pub fn fail_cancelled(&mut self, reason: impl Into<String>) {
        self.findings
            .push(Finding::new("E-CANCELLED").field("reason", Value::String(reason.into())));
        self.terminal = Some(Terminal::Internal);
    }

    /// The atomic write did not land. One sentence, so every write tool says it the same
    /// way.
    ///
    /// ADR-0011's exit 70 rather than an `error` finding, at both call sites and for the
    /// same reason: the file is untouched, so there is nothing about the *project* to
    /// report, and *"Montagent could not run"* has its own exit code precisely so it is not
    /// mistaken for a defect in the document.
    pub fn could_not_write(&mut self, path: impl std::fmt::Display, e: &std::io::Error) {
        self.fail_internally(format!("{path} could not be written: {e}"));
    }

    /// Add a finding to the report.
    ///
    /// # Panics
    ///
    /// If an `error`-class finding carries no repair. ADR-0043: *"Every `error`-class
    /// finding carries a `repair` field."* A refuse-class check gets one for free from
    /// its declaration; an advise-class one must state its value, and forgetting to is
    /// the only way the field goes missing. This is the single place every finding
    /// passes through, so it is the place to catch it — and the CLI turns the panic into
    /// exit 70 rather than an abort.
    ///
    /// ADR-0073 narrows the requirement to findings *about a document*: a code the
    /// registry declares `RepairClass::NotAboutDocument` is exempt, and only that
    /// declaration exempts it — nothing at a call site can.
    #[track_caller]
    pub fn push(&mut self, finding: Finding) {
        let exempt = matches!(
            registry::spec(&finding.code).and_then(|spec| spec.repair),
            Some(RepairClass::NotAboutDocument)
        );
        assert!(
            finding.class != Class::Error || finding.repair.is_some() || exempt,
            "{} is `error`-class and carries no repair (ADR-0043/ADR-0073)",
            finding.code
        );
        self.findings.push(finding);
    }

    /// A check set has run to completion. Called at the four run sites and nowhere else.
    pub fn record(&mut self, set: CheckSet) {
        if !self.check_sets.contains(&set) {
            self.check_sets.push(set);
        }
    }

    /// The check sets this run completed.
    pub fn check_sets(&self) -> &[CheckSet] {
        &self.check_sets
    }

    pub fn summary(&self) -> Summary {
        let mut summary = Summary::default();
        for finding in &self.findings {
            let slot = match finding.class {
                Class::Error => &mut summary.error,
                Class::Review => &mut summary.review,
                Class::Note => &mut summary.note,
                Class::Unchecked => &mut summary.unchecked,
                Class::Layout => &mut summary.layout,
                Class::Drift => &mut summary.drift,
            };
            *slot += 1;
        }
        summary
    }

    /// ADR-0011: exit non-zero only on `error`. A `review`-level failure would break
    /// every routine run in the edit loop, which is ADR-0006's alarm fatigue by another
    /// route — and `UNCHECKED` and `LAYOUT` are not severities at all.
    pub fn exit_code(&self) -> ExitCode {
        match self.terminal {
            Some(Terminal::Unparseable) => ExitCode::Unparseable,
            Some(Terminal::BadInvocation) => ExitCode::BadInvocation,
            Some(Terminal::Internal) => ExitCode::Internal,
            None if self.summary().error > 0 => ExitCode::Errors,
            None => ExitCode::Ok,
        }
    }

    /// ADR-0083: whether this run reached a condition whose subject is not the document
    /// at all — the invocation, the raw bytes, or Montagent's own process
    /// (`RepairClass::NotAboutDocument`, ADR-0073) — rather than a fact about the
    /// project. Exactly the runs that reach exit 2, 3 or 70; `Ok` and `Errors` (exit 0/1)
    /// are about the document and answer `false` here, which is the write-tool
    /// invariant's own scope (ADR-0006/ADR-0011): a document-shaped answer is still
    /// `success` over MCP, and only a run with nothing to say about the document at all
    /// sets `isError`.
    pub fn is_not_about_document(&self) -> bool {
        self.terminal.is_some()
    }

    /// The canonical JSON plus one verb's own block under `key` — `probe`'s
    /// `network_attempts`, `timeline`'s view, `measure`'s answer. One object per
    /// invocation, and one place the shape "a report, plus this" is spelled.
    pub fn to_json_with(&self, key: &str, block: Value) -> Value {
        let mut json = self.to_json();
        json.as_object_mut()
            .expect("a report serialises as an object")
            .insert(key.to_string(), block);
        json
    }

    /// The canonical JSON. Everything the text form prints is derivable from this and
    /// from the registry; nothing is assembled twice.
    pub fn to_json(&self) -> Value {
        let summary = self.summary();
        json!({
            "tool": self.tool,
            "project": self.project,
            // ADR-0112: what ran, beside what it found. `summary` keeps all six keys and
            // its meaning, so a zero there is only a claim where a set named here could
            // have raised that class.
            "check_sets": self.check_sets.iter().map(|set| set.as_str()).collect::<Vec<_>>(),
            "summary": {
                "error": summary.error,
                "review": summary.review,
                "note": summary.note,
                "unchecked": summary.unchecked,
                "layout": summary.layout,
                "drift": summary.drift,
            },
            "exit_code": self.exit_code().as_u8(),
            "findings": self.findings,
            "cache_misses": self.misses,
            "media": self.media,
            // Unconditional. ADR-0006: "the report ends with its own scope,
            // unconditionally" — an exception for the reports that never reached a
            // project reads as reasonable and is exactly the erosion the ADR is written
            // against, since the next exception argues from this one.
            "not_checked": NOT_CHECKED,
        })
    }
}
