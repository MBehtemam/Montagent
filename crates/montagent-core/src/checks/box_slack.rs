//! `R-BOX-SLACK` — *"does a declared text box exceed the block it actually bounds, by
//! more than a copy-pasted container would explain away?"*
//!
//! ADR-0014 required every text element to declare `width`/`height`, then recorded the
//! gap it left open: a box much bigger than the text it bounds disables the overflow
//! check exactly as effectively as omitting the box would, and nothing in that design
//! told the two apart. ADR-0058 closes the gap for the axis that needs no font: `height`
//! is arithmetically derivable from fields already in the document (`size`,
//! `line_height`, the author's own `\n` count), so a box that overshoots it beyond
//! `max(2px, 10% of computed height)` is reported as `note` — *"a fact you may want and
//! will not act on today"*, since an oversized box has zero rendering effect and the harm
//! it names (leftover slack silently absorbing a future edit that grows the text past its
//! true bound) is deferred rather than present.
//!
//! **Height only.** `width`'s expected value needs the font binary and shaper, not
//! arithmetic on the element — ADR-0014 already parked it `UNCHECKED` pending `measure`,
//! and this check stays exactly there.
//!
//! **No font, no I/O.** ADR-0028's block-height formula —
//! `(size × line_height×10 × line_count + 9) // 10`, [`crate::exact::text_block_height`]
//! — reads only fields the document already carries, evaluated in exact integer
//! arithmetic so a `line_height` like `1.1` never reaches the boundary through `f64`
//! (ADR-0028 measured that divergence on 6.35% of sampled cases).
//!
//! **The sibling census is not scoped to `text`.** ADR-0058's worked example matches a
//! caption's declared `height` against a `rect` card's — the fact that tells the two
//! populations apart is a coincidence of declared extent, not of element type, so the
//! census here reads every element in the document that carries a `height`.

use serde_json::{Value, json};

use crate::checks::subject_of;
use crate::exact::{Decimal, text_block_height};
use crate::finding::{Census, Finding};
use crate::permissive::Loose;
use crate::report::Report;

/// ADR-0028: `line_height` defaults to `1.2` when the element omits it.
const DEFAULT_LINE_HEIGHT_TENTHS: i64 = 12;

/// ADR-0058's floor for small text — a trivial-padding floor, not a noise floor; ADR-0028
/// already eliminated the float-rounding residual ADR-0014 would have cited for it.
const MIN_SLACK_PX: i64 = 2;

/// `R-BOX-SLACK`, over every `text` element the document declares a `height` for.
pub fn check(document: &Loose, report: &mut Report) {
    let file = document.path();
    let heights = declared_heights(document);

    for (track, element) in document.elements_in_tracks() {
        // prototype(#764): the derived-height checks do not apply to a text on a path
        // (ADR-0161 §7).
        if element.get("type").and_then(Value::as_str) != Some("text")
            || crate::text_path::has_path(element)
        {
            continue;
        }
        let Some(finding) = candidate(element, &heights) else {
            continue;
        };

        let mut finding = finding.at_file(file);
        if let Some(track) = track {
            finding = finding.at_track(track);
        }
        report.push(finding);
    }
}

/// Every element's declared `height`, by subject, in document order — the pool the
/// sibling census draws from. Any element type: ADR-0058's own example matches a caption
/// against a `rect` card.
fn declared_heights(document: &Loose) -> Vec<(String, i64)> {
    document
        .elements_in_tracks()
        .filter_map(|(_, element)| {
            let subject = subject_of(element.get("id").and_then(Value::as_str));
            let height = element.get("height").and_then(Value::as_i64)?;
            Some((subject, height))
        })
        .collect()
}

/// This text element's finding, if its declared `height` clears ADR-0058's threshold —
/// `None` if any field this check needs is absent or malformed (another check's business
/// to name) or if the box is not oversized.
fn candidate(element: &Value, heights: &[(String, i64)]) -> Option<Finding> {
    let declared_height = element.get("height").and_then(Value::as_i64)?;
    let size = element.get("size").and_then(Value::as_i64)?;
    let line_height_tenths = match element.get("line_height") {
        None => DEFAULT_LINE_HEIGHT_TENTHS,
        Some(value) => Decimal::of(value.as_number()?)?.tenths()?,
    };
    let line_count = 1 + concatenated_runs(element).matches('\n').count() as i64;

    let computed_height = text_block_height(size, line_height_tenths, line_count)?;
    // `i128` here, not `i64`: `declared_height` is read straight off the document with no
    // bound of its own, and `slack * 10` is exactly the multiplication that overflowed an
    // `i64` comparison on a maliciously large `height` before this widened.
    let slack = i128::from(declared_height) - i128::from(computed_height);
    if slack <= i128::from(MIN_SLACK_PX) || slack * 10 <= i128::from(computed_height) {
        return None;
    }
    let slack = i64::try_from(slack).ok()?;

    let subject = subject_of(element.get("id").and_then(Value::as_str));
    let mut finding = Finding::new("R-BOX-SLACK")
        .at_element(subject.clone())
        .field("element", json!(subject))
        .field("declared_height", json!(declared_height))
        .field("computed_height", json!(computed_height))
        .field("slack", json!(slack))
        .field("slack_percent", json!(percent(slack, computed_height)))
        .field(
            "derivation",
            json!(derivation(size, line_height_tenths, line_count)),
        );

    if let Some(census) = sibling_census(&subject, declared_height, heights) {
        finding = finding.census(census);
    }
    Some(finding)
}

/// An element's `runs`, concatenated. `\n` inside a run is the format's only line break
/// (ADR-0008's no-auto-wrap rule), so counting it is counting the author's own lines —
/// NFC normalization is [`crate::checks::caption`]'s concern, not this count's, since it
/// cannot move where a `\n` falls.
fn concatenated_runs(element: &Value) -> String {
    element
        .get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|run| run.get("text").and_then(Value::as_str))
        .collect()
}

/// The measured rate, to the nearest whole percent — presentational only, matching
/// [`crate::checks::caption::cps`]'s posture: the comparison above is made on the exact
/// integers, so this number is never the reason a finding fired or did not.
fn percent(slack: i64, computed_height: i64) -> i64 {
    ((slack as f64 * 100.0 / computed_height as f64).round()) as i64
}

/// ADR-0058's worked prose: `"size 55 × line_height 1.1 × 1 line"`.
fn derivation(size: i64, line_height_tenths: i64, line_count: i64) -> String {
    let whole = line_height_tenths / 10;
    let tenth = line_height_tenths % 10;
    let noun = if line_count == 1 { "line" } else { "lines" };
    format!("size {size} × line_height {whole}.{tenth} × {line_count} {noun}")
}

/// ADR-0058's sibling census: every *other* element whose declared `height` exactly
/// equals this one's, stated as a bare coincidence of extent — never "copied from",
/// never "should be". Omitted, not replaced with "no match found", when nothing matches
/// (ADR-0058: stating the negative would itself imply the copy hypothesis was tested).
fn sibling_census(
    subject: &str,
    declared_height: i64,
    heights: &[(String, i64)],
) -> Option<Census> {
    let members: Vec<String> = heights
        .iter()
        .filter(|(other_subject, height)| *height == declared_height && other_subject != subject)
        .map(|(other_subject, _)| other_subject.clone())
        .collect();

    (!members.is_empty()).then(|| Census::on("height").group(json!(declared_height), members))
}
