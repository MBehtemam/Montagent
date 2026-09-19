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
    in_form(&report.to_json(), form)
}

/// Render a `probe` answer in one wire form.
///
/// `probe`'s canonical JSON is a report's plus the facts only it establishes, and its
/// prose is that JSON's — so it renders through the same function, under the same rule,
/// rather than through a second one in an adapter. ADR-0006's *"never both in one
/// invocation"* is a property of [`Wire`]; it stays one only while every form goes through
/// here.
pub fn render_answer(answer: &crate::verbs::probe::Answer, form: Wire) -> String {
    let json = answer.to_json();
    match form {
        Wire::Json => canonical(&json),
        // `probe`'s whole answer is the media facts, so they print at any verbosity. For
        // `validate` the same block is `--verbose`-gated; the canonical JSON carries them
        // either way.
        Wire::Text { verbose } => prose(&json, text::Options::with_media(verbose)),
    }
}

/// Render a `timeline` answer in one wire form.
///
/// Through the same function and the same rule as everything else: the view is a block on
/// the canonical JSON, and the prose is that block generated from it (ADR-0006).
pub fn render_timeline(answer: &crate::verbs::timeline::Answer, form: Wire) -> String {
    in_form(&answer.to_json(), form)
}

/// Render a `query` answer in one wire form.
///
/// Through the same function and the same rule as everything else. ADR-0011 transplants
/// ADR-0006's wire decision onto this verb by name and gives its own reason for it: an
/// answer from `query` is *"an explanation of one moment, and prose is the denser encoding
/// of an explanation."*
pub fn render_query(answer: &crate::verbs::query::Answer, form: Wire) -> String {
    in_form(&answer.to_json(), form)
}

/// One canonical JSON, one prose generator, one rule about which of them prints.
fn in_form(json: &serde_json::Value, form: Wire) -> String {
    match form {
        Wire::Json => canonical(json),
        Wire::Text { verbose } => prose(
            json,
            text::Options {
                verbose,
                media: false,
            },
        ),
    }
}

/// The prose form, or — where Montaget cannot render its own report — the one form that
/// cannot itself fail, with the reason above it.
fn prose(json: &serde_json::Value, options: text::Options) -> String {
    match text::render(json, options) {
        Ok(rendered) => rendered,
        Err(e) => format!(
            "montaget could not render its own report: {e}\n{}\n",
            canonical(json)
        ),
    }
}

/// The canonical JSON as bytes — the one form that cannot itself fail.
fn canonical(json: &serde_json::Value) -> String {
    serde_json::to_string_pretty(json).unwrap_or_else(|_| json.to_string())
}
