//! `fmt` — *"rewrite the file in the canonical convention"* (ADR-0011), and the
//! non-destructive `--check` that reports what such a rewrite would change.
//!
//! CLI-only, by ADR-0011's cost model. It exists because the convention is normative and
//! nothing enforced it: ADR-0005 required *"one element per line, stable key order"* and
//! said it is load-bearing **because agents edit by exact-string replace**, and then an
//! agent asked for a routine edit rewrote the 154-line fixture into 1595 lines and reported
//! no schema errors. By the letter of every rule then in force it was correct.
//!
//! Four boundaries, each from an ADR, and each a thing this verb deliberately does *not*
//! do:
//!
//! - **It never adds or removes a defaulted field** (ADR-0030). Omission and
//!   explicit-at-default are two spellings of different declarations.
//! - **It never rewrites a value** (ADR-0026). At an exact aspect match both `cover` and
//!   `contain` are true, and a formatter that picked one would be deciding what the author
//!   meant.
//! - **It refuses only on a document that is not a project at all** (ADR-0042) — wholesale
//!   missing `tracks`/`fps`/`frame`. It stays usable on a file carrying `error` findings,
//!   which is exactly when it is most wanted: mid-authoring.
//! - **It writes atomically or not at all** (ADR-0011). The file is the one the agent is
//!   mid-edit on; a half-written project is worse than an unformatted one.
//!
//! What it *does* is one function's worth of decision — [`crate::layout::canonicalise`] —
//! and one writer, [`crate::write::canonical`]. Both are shared with `validate`'s `LAYOUT`
//! check rather than reimplemented, which is ADR-0041's *"exactly one place the rule lives,
//! not two that can disagree."*

use std::path::Path;

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::layout::{self, Shape};
use crate::parse;
use crate::permissive::Loose;
use crate::report::Report;
use crate::write;

const TOOL: &str = "fmt";

/// Which half of ADR-0041's split this invocation is.
///
/// A type rather than a `bool`, because the two modes differ in whether they touch the
/// file — the one distinction a reader of a call site must not have to guess at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Report what a rewrite would change. Writes nothing, ever.
    Check,
    /// Rewrite the file in the canonical convention.
    Write,
}

/// Format the project file at `path`, or report what formatting it would change.
///
/// The report's findings are the same `LAYOUT` findings `validate` produces, from the same
/// predicate — `fmt --check` is a second place to *ask*, never a second rule.
pub fn fmt(path: &Path, mode: Mode) -> Report {
    let project = Some(path.display().to_string());

    let document = match parse::read(path) {
        Ok(document) => document,
        // ADR-0011: nothing may partially process a malformed file. A formatter especially
        // — the bytes it would write are derived from a parse that did not happen.
        Err(finding) => return Report::unparseable(TOOL, project, *finding),
    };

    let mut report = Report::new(TOOL, project);

    if let Err(not_a_project) = document.shape() {
        // ADR-0042's precondition, and the whole of it: structural shape, never severity.
        report.push(
            Finding::new("E-NOT-A-PROJECT")
                .at_file(document.path())
                .field(
                    "missing",
                    Value::String(
                        not_a_project
                            .missing
                            .iter()
                            .map(|key| format!("`{key}`"))
                            .collect::<Vec<_>>()
                            .join("/"),
                    ),
                )
                .repair_value(json!({"value": "point fmt at the project file"})),
        );
        return report;
    }

    let written = match std::fs::read_to_string(path) {
        Ok(written) => written,
        // The file parsed a moment ago, so this is a race or a filesystem failure rather
        // than a fact about the project.
        Err(e) => {
            report.fail_internally(format!("{} could not be re-read: {e}", document.path()));
            return report;
        }
    };
    let canonical = write::canonical(&layout::canonicalise(document.value()));

    if written == canonical {
        return report;
    }

    for finding in findings(&document, &written, &canonical) {
        report.push(finding);
    }

    match mode {
        // Story 5: "tell me the file is in canonical convention without rewriting it, so
        // that I can verify before I commit". A `--check` that exited 0 on a file it had
        // just reported as non-canonical could not be used to verify anything.
        Mode::Check => report.needs_fixing(),
        Mode::Write => {
            if let Err(e) = write::atomically(path, &canonical) {
                report.fail_internally(format!("{} could not be written: {e}", document.path()));
            }
            // On success the findings stay in the report — they are what the run changed,
            // and an agent that asked for a rewrite is owed the list — but the file is now
            // canonical, so there is nothing left for the caller to fix.
        }
    }

    report
}

/// What a rewrite would change, as findings.
///
/// Two codes, because the two conditions are independently reachable: the incident agent
/// pretty-printed the fixture without disturbing a single key's position, and an agent
/// scripting an edit with `jq` reorders keys without touching a line break.
fn findings(document: &Loose, written: &str, canonical: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (track, element) in elements(document) {
        let Some(object) = element.as_object() else {
            continue;
        };
        let shape = Shape::of_element(element);
        if layout::is_canonical(shape, object) {
            continue;
        }
        let Some(expected) = layout::canonical_order(shape) else {
            // An element whose `type` the schema does not publish has no canonical order,
            // so there is nothing to be out of. `is_canonical` already said so; this is
            // the unreachable half of the same fact, kept rather than unwrapped.
            continue;
        };

        let id = element.get("id").and_then(Value::as_str).unwrap_or("?");
        let mut finding = Finding::new("L-KEY-ORDER")
            .at_file(document.path())
            .at_element(id)
            .field("type", element["type"].clone())
            // Only the keys the element actually carries. ADR-0030: naming the omitted
            // ones would read as a list of fields to add, which is the one thing `fmt` is
            // forbidden to do.
            .field(
                "expected",
                Value::String(
                    expected
                        .iter()
                        .filter(|key| object.contains_key(*key))
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(","),
                ),
            );
        if let Some(track) = track {
            finding = finding.at_track(track);
        }
        if let Some(line) = line_of_element(written, id) {
            finding = finding.at_line(line);
        }
        findings.push(finding);
    }

    // Everything else `fmt` would change — line breaks, indentation, the header's and the
    // tracks' own key order, the trailing newline — as one finding about the file, because
    // that is the granularity it is true at. The 154 → 1595 incident is one fact, not 1441
    // of them.
    //
    // "Everything else" is measured, not assumed: the comparison is against the document
    // canonicalised *except* for each element's own key order, so what is left is exactly
    // the part of a rewrite no `L-KEY-ORDER` finding names. A file whose only fault is an
    // element's keys produces those findings and nothing more; told both, a reader would be
    // reading one fact twice, and ADR-0006's noise budget is explicit that a check free to
    // run and expensive to report is still expensive.
    let explained = write::canonical(&layout::canonicalise_except_elements(document.value()));
    if written != explained {
        findings.push(
            Finding::new("L-LAYOUT")
                .at_file(document.path())
                .field("written_lines", json!(written.lines().count()))
                .field("canonical_lines", json!(canonical.lines().count()))
                .at_line(first_difference(written, &explained)),
        );
    }

    findings
}

/// Every element in the document, paired with the name of the track it sits in.
///
/// [`Loose::elements`] already walks them, and flattens the track away. A `LAYOUT` finding
/// wants the track for its location, so this walks the same two levels and keeps it.
fn elements(document: &Loose) -> Vec<(Option<&str>, &Value)> {
    document.value()["tracks"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .flat_map(|track| {
            let name = track.get("name").and_then(Value::as_str);
            track["elements"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .map(move |element| (name, element))
        })
        .collect()
}

/// The first line the two forms differ on, 1-based, or the line after the shorter of them
/// where one is a prefix of the other.
///
/// The number a reader opens their editor at. It is a line number in the file **as
/// written**, which is the only one that means anything to someone looking at it.
fn first_difference(written: &str, canonical: &str) -> u32 {
    let mut theirs = canonical.lines();
    for (i, line) in written.lines().enumerate() {
        if theirs.next() != Some(line) {
            return i as u32 + 1;
        }
    }
    written.lines().count() as u32 + 1
}

/// The line an element's `id` is written on, if it can be found unambiguously.
///
/// Located by text rather than by a span the parser kept, because `serde_json` keeps none.
/// The search is deliberately narrow: the line must carry the `"id"` key and the id's own
/// quoted spelling, and the id must appear on exactly one line — the same
/// uniquely-matchable-substring property the whole convention exists to protect
/// (ADR-0041). Where it is not unique the finding goes out without a line rather than with
/// a guessed one, because ADR-0006's whole posture is that a stated number is a measured
/// one.
fn line_of_element(written: &str, id: &str) -> Option<u32> {
    let quoted = format!("\"{id}\"");
    let mut found = None;
    for (i, line) in written.lines().enumerate() {
        if line.contains("\"id\"") && line.contains(&quoted) {
            if found.is_some() {
                return None;
            }
            found = Some(i as u32 + 1);
        }
    }
    found
}
