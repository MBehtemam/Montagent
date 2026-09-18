//! The text report, generated from the canonical JSON.
//!
//! ADR-0006: *"JSON is canonical; text is generated from it; text prints by default;
//! `--json` prints JSON instead, never both in one invocation."*
//!
//! [`render`] takes a [`serde_json::Value`] and nothing else. That signature is the
//! mechanism, not a convenience: there is no path by which the prose can state something
//! the JSON does not carry, because the prose renderer has never seen the `Report`.
//! Every sentence comes from the registered template for a finding's code, filled from
//! that finding's own field set — *"one code, one field set, one template."*

use serde_json::Value;
use std::fmt;

use crate::finding::Class;
use crate::registry;

/// The reserved field rendered as an indented block rather than interpolated: the
/// offending line and its caret, which ADR-0011 requires of every tool's parse failure.
const EXCERPT: &str = "excerpt";

/// What the text renderer may print.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Expand the informational classes that otherwise collapse to one counted line.
    pub verbose: bool,
}

impl Options {
    pub fn verbose() -> Self {
        Options { verbose: true }
    }
}

/// The report JSON did not carry what a template needed.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderError(String);

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RenderError {}

/// Render a canonical report JSON as the prose form.
pub fn render(report: &Value, options: Options) -> Result<String, RenderError> {
    let mut out = String::new();

    out.push_str(&summary_line(report)?);
    out.push('\n');

    let findings = report["findings"]
        .as_array()
        .ok_or_else(|| RenderError("report has no `findings` array".into()))?;

    for class in [
        Class::Error,
        Class::Review,
        Class::Note,
        Class::Unchecked,
        Class::Layout,
    ] {
        let of_class: Vec<&Value> = findings
            .iter()
            .filter(|f| f["class"] == class.as_str())
            .collect();
        if of_class.is_empty() {
            continue;
        }
        out.push('\n');
        if class.prints_in_full() || options.verbose {
            for finding in of_class {
                out.push_str(&full(finding)?);
            }
        } else {
            out.push_str(&collapsed(&of_class));
        }
    }

    // ADR-0006: "the report ends with its own scope, unconditionally." Without it a
    // clean run reads as "the file is right", which is ADR-0004's `sequence` label
    // wearing a `validate` label instead.
    let boundary = report["not_checked"]
        .as_str()
        .ok_or_else(|| RenderError("report has no `not_checked` block".into()))?;
    out.push_str("\nNOT CHECKED\n");
    for line in wrap(boundary, 76) {
        out.push_str("  ");
        out.push_str(&line);
        out.push('\n');
    }

    Ok(out)
}

fn summary_line(report: &Value) -> Result<String, RenderError> {
    let summary = &report["summary"];
    let count = |key: &str| summary[key].as_u64().unwrap_or(0);
    let mut line = format!(
        "{}, {}, {}, {} unchecked, {} layout",
        plural(count("error"), "error"),
        plural(count("review"), "review"),
        plural(count("note"), "note"),
        count("unchecked"),
        count("layout"),
    );
    if let Some(project) = report["project"].as_str() {
        line.push_str(" — ");
        line.push_str(project);
    }
    Ok(line)
}

fn plural(n: u64, noun: &str) -> String {
    if n == 1 {
        format!("{n} {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

/// One counted line carrying the code, per class-and-code, for the informational
/// classes. ADR-0006's noise budget: `0 errors, 47 notes` must not read as a pass, and
/// 47 printed alignment lines are how a reader learns to skip the output.
fn collapsed(findings: &[&Value]) -> String {
    // Counted in first-appearance order rather than by size: a report that reordered its
    // own classes by how many of each there are would be ranking them, and ADR-0006's
    // whole point about the informational classes is that they are inert.
    let mut counted: Vec<(&str, usize)> = Vec::new();
    for finding in findings {
        let code = finding["code"].as_str().unwrap_or("?");
        match counted.iter_mut().find(|(c, _)| *c == code) {
            Some((_, count)) => *count += 1,
            None => counted.push((code, 1)),
        }
    }

    // Every finding here shares one class — the caller groups by it before collapsing.
    let class = findings[0]["class"].as_str().unwrap_or("?");
    counted
        .into_iter()
        .map(|(code, count)| format!("{class}  {code}  {count} — expand with --verbose\n"))
        .collect()
}

fn full(finding: &Value) -> Result<String, RenderError> {
    let code = finding["code"]
        .as_str()
        .ok_or_else(|| RenderError("finding has no `code`".into()))?;
    let spec = registry::spec(code).ok_or_else(|| {
        RenderError(format!(
            "{code} is not a registered check code, so it has no template"
        ))
    })?;

    let class = finding["class"]
        .as_str()
        .unwrap_or(spec.default_class().as_str());
    let locus = finding["location"]["element"]
        .as_str()
        .or_else(|| finding["location"]["track"].as_str())
        .map(|l| format!("  {l}"))
        .unwrap_or_default();

    let mut out = format!("{class}  {code}{locus}\n");
    // A template may render several lines — a bad invocation carries a usage block —
    // so indent every one of them rather than only the first.
    for line in interpolate(spec.template, finding, code)?.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }

    if let Some(excerpt) = finding["fields"][EXCERPT].as_str() {
        out.push('\n');
        for line in excerpt.lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }

    if let Some(reason) = finding.get("reason").filter(|r| !r.is_null()) {
        out.push_str(&format!("  Could not look: {}\n", reason_words(reason)));
    }

    // ADR-0043: the field is not visible in a bare code quoted in text, so the prose
    // renderer states the class in words on every `error`-class finding.
    match finding.get("repair") {
        Some(Value::String(s)) if s == "none" => {
            // Class and guarantee only. ADR-0043's further instruction — stop, and
            // surface the finding verbatim to whoever is operating Montaget — is carried
            // by the templates of the checks it was written for, because it is advice
            // about a document an agent might otherwise plausibly repair. Told to an
            // agent that has just broken its own JSON with `sed`, it sends a typo to a
            // human instead of to the caret two lines above.
            out.push_str(
                "  This finding is refuse-class: Montaget will not state a repair, because none\n  \
                 is determined by the document, the media on disk or the published rendering\n  \
                 semantics — and no flag, force mode or write tool can lift it. Do not guess one.\n",
            );
        }
        Some(repair) if !repair.is_null() => {
            // `{"value": …}` is ADR-0043's shape for an advise-class repair; unwrap the
            // one key so the sentence reads as a sentence.
            let stated = repair.get("value").unwrap_or(repair);
            out.push_str(&format!(
                "  This finding is advise-class: the fix is fully determined — {}.\n",
                compact(stated)
            ));
        }
        _ => {}
    }

    if let Some(census) = finding.get("census").filter(|c| !c.is_null()) {
        let field = census["field"].as_str().unwrap_or("?");
        let groups: Vec<String> = census["groups"]
            .as_array()
            .map(|groups| {
                groups
                    .iter()
                    .map(|g| {
                        let members = g["members"].as_array().map_or(0, |m| m.len());
                        format!("{members} at {}", compact(&g["value"]))
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.push_str(&format!("  census {field}: {}\n", groups.join(", ")));
    }

    // ADR-0061: the threshold's source is cited inline **in the finding**. There is
    // deliberately no fallback to the registry's own `ThresholdProvenance::External`
    // here: reaching for it would have the prose state a fact the canonical JSON does
    // not carry, which is the one thing the wire rule exists to prevent. A finding whose
    // check borrows a threshold and which arrives without a citation is a defect in that
    // check, and it should read as one rather than be papered over at render time.
    if let Some(citation) = finding.get("citation").filter(|c| !c.is_null()) {
        out.push_str(&format!(
            "  Threshold {} is external, cited from {} ({}).\n",
            compact(&citation["threshold"]),
            citation["source"].as_str().unwrap_or("?"),
            citation["adr"].as_str().unwrap_or("?"),
        ));
    }

    Ok(out)
}

fn reason_words(reason: &Value) -> String {
    match reason["kind"].as_str() {
        Some("http") => format!("http {}", reason["status"]),
        Some(kind) => kind.replace('_', " "),
        None => compact(reason),
    }
}

/// Fill `{name}` from the finding's field set, falling back to its location.
fn interpolate(template: &str, finding: &Value, code: &str) -> Result<String, RenderError> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = rest[open..]
            .find('}')
            .ok_or_else(|| RenderError(format!("{code}: unterminated `{{` in its template")))?
            + open;
        let name = &rest[open + 1..close];
        let value = finding["fields"]
            .get(name)
            .or_else(|| finding["location"].get(name))
            .ok_or_else(|| {
                RenderError(format!(
                    "{code}: its template names `{name}`, which the finding does not carry"
                ))
            })?;
        out.push_str(&compact(value));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// A value as it reads in prose: a string bare, everything else as compact JSON.
fn compact(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
