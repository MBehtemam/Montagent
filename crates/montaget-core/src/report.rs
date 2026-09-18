//! The report surface: a canonical JSON finding set, its summary, its exit code, and
//! the boundary it prints on every run.

use serde_json::{Value, json};

use crate::finding::{Class, Finding};

/// ADR-0006's `NOT CHECKED` block, verbatim.
///
/// *"Without it, a clean run is read as 'the file is right' — and 'run this and the
/// file is fine' is the `sequence` label again, wearing a `validate` label instead."*
pub const NOT_CHECKED: &str = "This file was not compared against any prior version or instruction. validate \
verifies that the file is internally legal; it cannot tell you whether it says what you \
meant it to say.";

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
}

/// One verb's answer.
#[derive(Debug, Clone)]
pub struct Report {
    pub tool: String,
    pub project: Option<String>,
    pub findings: Vec<Finding>,
    terminal: Option<Terminal>,
}

impl Report {
    pub fn new(tool: impl Into<String>, project: Option<String>) -> Self {
        Report {
            tool: tool.into(),
            project,
            findings: Vec::new(),
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
            terminal: Some(Terminal::Unparseable),
        }
    }

    /// The invocation was wrong. An invocation error is a finding like any other, so
    /// there is exactly one thing to parse across the surface (ADR-0011).
    pub fn bad_invocation(reason: impl Into<String>) -> Self {
        Report {
            tool: "montaget".into(),
            project: None,
            findings: vec![
                Finding::new("E-INVOCATION")
                    .field("reason", Value::String(reason.into()))
                    .advise_class(json!({"value": "fix the command"})),
            ],
            terminal: Some(Terminal::BadInvocation),
        }
    }

    /// Montaget itself failed — ffmpeg died, the font stack failed.
    pub fn internal_failure(reason: impl Into<String>) -> Self {
        Report {
            tool: "montaget".into(),
            project: None,
            findings: vec![
                Finding::new("E-INTERNAL")
                    .field("reason", Value::String(reason.into()))
                    .refuse_class(),
            ],
            terminal: Some(Terminal::Internal),
        }
    }

    pub fn push(&mut self, finding: Finding) {
        self.findings.push(finding);
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

    /// The canonical JSON. Everything the text form prints is derivable from this and
    /// from the registry; nothing is assembled twice.
    pub fn to_json(&self) -> Value {
        let summary = self.summary();
        json!({
            "tool": self.tool,
            "project": self.project,
            "summary": {
                "error": summary.error,
                "review": summary.review,
                "note": summary.note,
                "unchecked": summary.unchecked,
                "layout": summary.layout,
            },
            "exit_code": self.exit_code().as_u8(),
            "findings": self.findings,
            // Unconditional. ADR-0006: "the report ends with its own scope,
            // unconditionally" — an exception for the reports that never reached a
            // project reads as reasonable and is exactly the erosion the ADR is written
            // against, since the next exception argues from this one.
            "not_checked": NOT_CHECKED,
        })
    }
}
