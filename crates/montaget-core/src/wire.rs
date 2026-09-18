//! The one place a report becomes bytes.
//!
//! ADR-0006 settles the rule once for the whole surface: *"JSON is canonical; text is
//! generated from it; text prints by default; `--json` prints JSON instead, never both
//! in one invocation."* [`Wire`] is that rule as a type — the two forms are alternatives
//! by construction, so no adapter can grow a third combination and none of them can
//! print both.
//!
//! Both adapters call [`render`]. Neither decides anything about the wire format, which
//! is what keeps "thin by construction" true rather than aspirational.

use crate::report::Report;
use crate::text;

/// Which form one invocation answers in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wire {
    /// The prose form, generated from the canonical JSON.
    Text {
        /// Expand the informational classes that otherwise collapse to one counted line.
        verbose: bool,
    },
    /// The canonical JSON, pretty-printed.
    Json,
}

impl Wire {
    /// The form a `--json` / `--verbose` pair of flags asks for. `--verbose` is a
    /// property of the prose, so it has no effect on the JSON: the canonical form is
    /// already complete and is filtered by its consumer, not by Montaget.
    pub fn from_flags(json: bool, verbose: bool) -> Self {
        if json {
            Wire::Json
        } else {
            Wire::Text { verbose }
        }
    }
}

/// Render a report in one wire form.
///
/// Infallible on purpose. Both failures it could have — a finding whose code has no
/// registered template, or JSON that will not serialise — are bugs in Montaget rather
/// than facts about the project, and a caller cannot act on either. Rather than hand
/// every adapter an error to mishandle, say so in the output and print the canonical
/// JSON, which is the one form that cannot itself fail.
pub fn render(report: &Report, form: Wire) -> String {
    let json = report.to_json();
    let canonical = || serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string());

    match form {
        Wire::Json => canonical(),
        Wire::Text { verbose } => {
            let options = text::Options { verbose };
            match text::render(&json, options) {
                Ok(rendered) => rendered,
                Err(e) => format!(
                    "montaget could not render its own report: {e}\n{}\n",
                    canonical()
                ),
            }
        }
    }
}
