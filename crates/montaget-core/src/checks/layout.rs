//! The `LAYOUT` check — *"is this file written in the canonical convention?"* (ADR-0041).
//!
//! The one category that is not about what the video looks like. A key-order violation
//! renders identically; what it costs is that the file *"stops being silently unsafe to
//! edit"*, because the whole exact-string-replace surface leans on the convention. ADR-0041
//! gave it a fourth report category rather than forcing it into the `error`/`review`/`note`
//! ladder, and three properties follow from that and are asserted rather than assumed:
//!
//! - **It runs unconditionally.** `fmt --check`-only was rejected outright: the incident
//!   agent *"was not running a formatter, it believed it was making a routine edit and had
//!   no reason to invoke one"*. `validate` runs on files nobody ever ran `fmt` over, which
//!   is exactly the file the incident produced.
//! - **It never gates `render`.** `LAYOUT` is not a severity, refusal stays keyed to
//!   `error` alone, and a non-canonical file renders — identically.
//! - **It is one implementation, shared.** ADR-0041: `fmt` and this check *"share one
//!   implementation of 'what does canonical form look like' … so there is exactly one place
//!   the rule lives, not two that can disagree."* This module is that place at the level of
//!   the findings, and [`crate::layout`] is it at the level of the rule: `fmt` prints
//!   exactly what this function returns, having decided nothing of its own.
//!
//! Two codes, because the two conditions are independently reachable: the incident agent
//! pretty-printed the fixture from 154 lines to 1595 **without disturbing a single key's
//! position**, and an agent scripting an edit with `jq` reorders keys without touching a
//! line break.

use serde_json::{Value, json};

use crate::finding::Finding;
use crate::layout::{self, Published};
use crate::permissive::Loose;
use crate::report::Report;
use crate::write;

/// Every way this file departs from the canonical convention, as findings.
pub fn check(document: &Loose, report: &mut Report) {
    for finding in findings(document) {
        report.push(finding);
    }
}

/// The same findings, as a list — what `fmt` reports and, in write mode, what its run
/// changed.
///
/// A file with no bytes behind it produces the key-order findings and no `L-LAYOUT`: that
/// code's whole claim is a comparison against the file **as written**, and there is nothing
/// to compare. Only a document built from a [`Value`] in memory is in that position;
/// [`crate::parse::read`] always carries the bytes.
pub fn findings(document: &Loose) -> Vec<Finding> {
    let mut findings = Vec::new();
    let written = document.source();

    for (track, element) in document.elements_in_tracks() {
        let Some(object) = element.as_object() else {
            continue;
        };
        let published = Published::of_element(element);
        if layout::is_canonical(published, object) {
            continue;
        }
        let Some(expected) = layout::canonical_order(published) else {
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
        if let Some(line) = document.line_of_element(id) {
            finding = finding.at_line(line);
        }
        findings.push(finding);
    }

    // Everything else a rewrite would change — line breaks, indentation, the header's and
    // the tracks' own key order, the trailing newline — as one finding about the file,
    // because that is the granularity it is true at. The 154 → 1595 incident is one fact,
    // not 1441 of them.
    //
    // "Everything else" is measured, not assumed: the comparison is against the document
    // canonicalised *except* for each element's own key order, so what is left is exactly
    // the part of a rewrite no `L-KEY-ORDER` finding names. A file whose only fault is an
    // element's keys produces those findings and nothing more; told both, a reader would be
    // reading one fact twice, and ADR-0006's noise budget is explicit that a check free to
    // run and expensive to report is still expensive.
    let Some(written) = written else {
        return findings;
    };
    let beyond_key_order =
        write::canonical(&layout::canonicalise_except_elements(document.value()));
    if written != beyond_key_order {
        findings.push(
            Finding::new("L-LAYOUT")
                .at_file(document.path())
                // The two line counts are the file the reader is looking at and the file
                // `montaget fmt` would hand them — both about the real output, not about
                // the intermediate this comparison is made against.
                .field("written_lines", json!(written.lines().count()))
                .field(
                    "canonical_lines",
                    json!(canonical(document).lines().count()),
                )
                // The line, though, is the first difference this finding is *about*: a
                // difference on an element's own line already has an `L-KEY-ORDER` finding
                // naming it, and sending the reader there twice is the duplicate-voice
                // problem this comparison exists to avoid.
                .at_line(first_difference(written, &beyond_key_order)),
        );
    }

    findings
}

/// The whole document in canonical form — the bytes `fmt` would write.
///
/// Here rather than in `fmt` so that the file the findings describe and the file the rewrite
/// produces are the same string, derived once.
pub fn canonical(document: &Loose) -> String {
    write::canonical(&layout::canonicalise(document.value()))
}

/// The first line on which `written` differs from `target`, 1-based, or the line after the
/// shorter of them where one is a prefix of the other.
///
/// The number a reader opens their editor at. It is a line number in the file **as
/// written**, which is the only one that means anything to someone looking at it; `target`
/// is whichever canonical form the finding is about.
///
/// Split on `\n` rather than with `lines()`, which strips `\r\n` and `\n` alike. The
/// convention is a *byte* convention (`.gitattributes`, #189), so a file checked out with
/// CRLF differs from canonical on its very first line — and `lines()` would report every
/// line equal and send the reader to the end of the file.
fn first_difference(written: &str, target: &str) -> u32 {
    let mut theirs = target.split('\n');
    for (i, line) in written.split('\n').enumerate() {
        if theirs.next() != Some(line) {
            return i as u32 + 1;
        }
    }
    written.split('\n').count() as u32 + 1
}
