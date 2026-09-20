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

use serde_json::{Value, json};
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
    /// Print the media facts the run established.
    ///
    /// Always for `probe`, whose whole answer they are; for `validate` only under
    /// `--verbose`, because a clean run over the fixture establishes sixteen of them and
    /// ADR-0006's noise budget is explicit that *"a check that is free to run and expensive
    /// to report is still expensive"*. The canonical JSON carries them either way — this
    /// filters what prints, never what was analysed.
    pub media: bool,
}

impl Options {
    pub fn verbose() -> Self {
        Options {
            verbose: true,
            media: false,
        }
    }

    /// The form `probe` answers in: the facts are the point, at any verbosity.
    pub fn with_media(verbose: bool) -> Self {
        Options {
            verbose,
            media: true,
        }
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

    // ADR-0006: "Then report the cache miss, unprompted, at the top." Not behind
    // `--verbose`, not collapsed into a count, and not omitted when it is the only thing
    // that changed — ADR-0011 calls this line the sole mechanism announcing a source that
    // grew on disk, so it prints before the findings do.
    if let Some(misses) = report["cache_misses"].as_array().filter(|m| !m.is_empty()) {
        out.push_str("\nCACHE\n");
        for miss in misses {
            out.push_str("  ");
            out.push_str(&miss_line(miss));
            out.push('\n');
        }
    }

    if let Some(media) = report["media"]
        .as_array()
        .filter(|m| !m.is_empty() && (options.media || options.verbose))
    {
        out.push_str("\nMEDIA\n");
        for probed in media {
            out.push_str(&source_block(probed));
        }
    }

    // `timeline`'s whole answer, and the only block on the wire that is a view rather than
    // a finding. It prints wherever it is present, at any verbosity: a verb whose output is
    // the view has nothing left to say if the view is filtered out.
    if let Some(overview) = report.get("timeline").filter(|view| !view.is_null()) {
        out.push_str(&timeline_block(overview));
    }

    // `query`'s whole answer, on the same rule as `timeline`'s view: it prints wherever it
    // is present and at any verbosity, because a verb whose output is the answer has nothing
    // left to say if the answer is filtered out.
    if let Some(query) = report.get("query").filter(|view| !view.is_null()) {
        out.push_str(&query_block(query));
    }

    // `measure`'s whole answer, on the same rule again.
    if let Some(measure) = report.get("measure").filter(|view| !view.is_null()) {
        out.push_str(&measure_block(measure));
    }

    // `fonts list`'s whole answer, on the same rule again.
    if let Some(fonts) = report.get("fonts").filter(|view| !view.is_null()) {
        out.push_str(&fonts_block(fonts));
    }

    // `fonts vendor`'s block: what was written, and the chain entry to write next. It
    // prints above the findings because the findings are about the file it just changed.
    if let Some(vendor) = report.get("vendor").filter(|view| !view.is_null()) {
        out.push_str(&vendor_block(vendor));
    }

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

/// One probe that ran, in the words ADR-0006 asks for: what changed, not that a cache was
/// consulted.
fn miss_line(miss: &Value) -> String {
    let source = miss["source"].as_str().unwrap_or("?");
    match miss["kind"].as_str() {
        Some("first") => format!("{source} — probed (not yet in the probe cache)"),
        // Both halves of ADR-0006's key, both sides of the change. The size alone is the
        // number a reader acts on, and the mtime is what makes the claim checkable against
        // a `stat` — a file rewritten to the same length is a change only the mtime shows.
        Some("changed") => format!(
            "{source} — CHANGED ON DISK since it was last probed: {} bytes → {} bytes, mtime {} → {}",
            miss["previous_size"],
            miss["size"],
            mtime(&miss["previous_mtime_ns"]),
            mtime(&miss["mtime_ns"])
        ),
        Some("remote") => {
            format!("{source} — fetched (remote sources are never cached across runs)")
        }
        _ => format!("{source} — probed"),
    }
}

/// A modification time as the report prints it, or an em dash where the filesystem would
/// not say one.
fn mtime(value: &Value) -> String {
    match value.as_i64() {
        Some(ns) => format!("{ns} ns"),
        None => "—".to_string(),
    }
}

/// One probed source, as ADR-0011's quad and the ADR-0023 dimensions.
fn source_block(probed: &Value) -> String {
    // Only sources that answered reach this block: a source that did not is a finding, and
    // the report prints it in its own section. Repeating it here would put one fact in two
    // voices.
    let mut out = format!("  {}\n", probed["source"].as_str().unwrap_or("?"));
    let mut row = |label: &str, value: String| {
        out.push_str(&format!("    {label:<16}{value}\n"));
    };

    let quad = &probed["quad"];
    let ms = |key: &str| match quad[key].as_i64() {
        Some(ms) => format!("{ms} ms"),
        None => "—".to_string(),
    };
    let rate = |key: &str| match (quad[key]["num"].as_i64(), quad[key]["den"].as_i64()) {
        (Some(num), Some(den)) => format!("{num}/{den}"),
        _ => "—".to_string(),
    };

    // The quad, spelled out. ADR-0011: the caller picks, so the caller must see all four.
    row("video stream", ms("video_stream_ms"));
    row("container", ms("container_ms"));
    row("start_time", ms("start_time_ms"));
    row("r_frame_rate", rate("r_frame_rate"));
    row("avg_frame_rate", rate("avg_frame_rate"));

    let dimensions = &probed["dimensions"];
    if !dimensions.is_null() {
        row(
            "dimensions",
            format!(
                // ADR-0023 (extending ADR-0015): print the dimensions used *and* which
                // rotation source was applied.
                "{}×{} (decoded {}×{}, rotation {}° from {}, par {}:{})",
                dimensions["width"],
                dimensions["height"],
                dimensions["decoded"]["width"],
                dimensions["decoded"]["height"],
                dimensions["rotation"]["degrees"],
                dimensions["rotation"]["source"].as_str().unwrap_or("?"),
                dimensions["par"]["num"],
                dimensions["par"]["den"],
            ),
        );
    }
    if let Some(alpha) = probed["alpha"].as_bool() {
        row("alpha", if alpha { "yes" } else { "no" }.to_string());
    }
    let audio = &probed["audio"];
    if !audio.is_null() {
        // A field the stream did not state prints as the same dash the quad's rows use.
        // Printing JSON `null` at a reader would be a third spelling of "we do not know",
        // next to the dash and the report's own NOT CHECKED.
        let stated = |value: &Value| match value {
            Value::Null => "—".to_string(),
            other => other.to_string(),
        };
        row(
            "audio",
            format!(
                "{} Hz, {} channel{}, {} ms",
                stated(&audio["sample_rate"]),
                stated(&audio["channels"]),
                if audio["channels"].as_u64() == Some(1) {
                    ""
                } else {
                    "s"
                },
                stated(&audio["audio_stream_ms"])
            ),
        );
    }

    out
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

// ---- `timeline`'s wide view (#195) ---------------------------------------------------

/// The whole view, generated from the `timeline` block of the canonical JSON.
///
/// Column widths are measured from the rows themselves rather than fixed, so a project of
/// short ids does not print a field of blanks and one of long ids does not have its detail
/// column knocked out of alignment. No column is measured against the *terminal*: ADR-0031
/// names terminal-width sensitivity as one of the costs a spatial view would have imposed,
/// and a view with no axis has no reason to take it on.
fn timeline_block(overview: &Value) -> String {
    let mut out = String::from("\nTIMELINE\n");
    let mut row = |label: &str, value: String| {
        out.push_str(&format!("  {label:<12}{value}\n"));
    };

    row("frame", frame_words(&overview["frame"], &overview["fps"]));
    row("duration", milliseconds(&overview["duration_ms"]));
    // Keyed on presence, not on type. ADR-0030 makes omission and a stated value two
    // different declarations, and this view is wanted on half-written documents — so a
    // `background` of `12` prints as `12` rather than disappearing from the prose while the
    // canonical JSON still carries it.
    for (label, key) in [("background", "background"), ("output", "output")] {
        if !overview[key].is_null() {
            row(label, compact(&overview[key]));
        }
    }
    // ADR-0062: `loop` asserts that `duration` connects back to 0, so it belongs next to the
    // length it talks about — and only when the document states it, since omission and
    // explicit `false` are different declarations (ADR-0030).
    if !overview["loop"].is_null() {
        row("loop", compact(&overview["loop"]));
    }

    let counts = &overview["counts"];
    out.push_str(&format!(
        "  {}, {}, {}\n",
        counted(&counts["elements"], "element"),
        counted(&counts["tracks"], "track"),
        counted(&counts["groups"], "group"),
    ));

    out.push_str(&tracks_block(&overview["tracks"]));
    out.push_str(&groups_block(&overview["groups"]));
    out
}

/// `1080×1920 at 25 fps`, or whatever of it the document actually states.
fn frame_words(frame: &Value, fps: &Value) -> String {
    let size = match (&frame["width"], &frame["height"]) {
        (Value::Null, Value::Null) => compact(frame),
        (width, height) => format!("{width}×{height}"),
    };
    match fps {
        Value::Null => size,
        fps => format!("{size} at {} fps", compact(fps)),
    }
}

/// Every track, back to front — the one thing a track supplies (ADR-0004).
fn tracks_block(tracks: &Value) -> String {
    let Some(tracks) = tracks.as_array().filter(|tracks| !tracks.is_empty()) else {
        return String::new();
    };

    let layers: Vec<String> = tracks
        .iter()
        .map(|track| layer_words(&track["layer"]))
        .collect();
    let names: Vec<String> = tracks.iter().map(|track| compact(&track["name"])).collect();
    let layer_width = width_of(layers.iter().map(String::as_str));
    let name_width = width_of(names.iter().map(String::as_str));

    let mut out = String::from("\nTRACKS\n");
    for ((track, layer), name) in tracks.iter().zip(&layers).zip(&names) {
        out.push_str(&format!(
            "  {layer:<layer_width$}  {name:<name_width$}  {}\n",
            counted(&track["elements"], "element"),
        ));
    }
    out
}

/// One block per group, each holding its elements in the order the clock reaches them.
fn groups_block(groups: &Value) -> String {
    let Some(groups) = groups.as_array().filter(|groups| !groups.is_empty()) else {
        return String::new();
    };

    // Each group's rows, gathered per group and kept that way. The rows were never
    // flattened, so nothing has to pair them back to a heading afterwards — the structure
    // the document has is the structure this walks.
    let blocks: Vec<(&Value, Vec<Row>)> = groups
        .iter()
        .map(|group| {
            let rows = group["elements"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .map(Row::of)
                .collect();
            (group, rows)
        })
        .collect();

    // Measured across every block at once, so the columns line up down the whole view
    // rather than restarting at each heading.
    let width = Widths::over(blocks.iter().flat_map(|(_, rows)| rows));

    // The unit is stated on the heading rather than on every row: ADR-0004 asks that
    // absolute times be unmissable, and a reader working out what `0..3018` counts in has
    // missed them.
    let mut out = String::from("\nGROUPS  (start..end in absolute milliseconds)\n");
    for (group, rows) in &blocks {
        out.push_str(&format!("  {}\n", group_heading(group)));
        for row in rows {
            out.push_str(row.line(&width).trim_end());
            out.push('\n');
        }
    }
    out
}

/// One element's cells, in the order they print.
struct Row {
    range: String,
    span: String,
    layer: String,
    kind: String,
    id: String,
    track: String,
    detail: String,
}

impl Row {
    fn of(element: &Value) -> Row {
        Row {
            range: range_words(element),
            span: milliseconds(&element["duration_ms"]),
            layer: layer_words(&element["layer"]),
            kind: named(&element["type"]),
            id: named(&element["id"]),
            track: named(&element["track"]),
            detail: element["detail"].as_str().unwrap_or_default().to_string(),
        }
    }

    fn line(&self, width: &Widths) -> String {
        format!(
            "    {:<range$}  {:>span$}  {:<layer$}  {:<kind$}  {:<id$}  {:<track$}  {}",
            self.range,
            self.span,
            self.layer,
            self.kind,
            self.id,
            self.track,
            self.detail,
            range = width.range,
            span = width.span,
            layer = width.layer,
            kind = width.kind,
            id = width.id,
            track = width.track,
        )
    }
}

/// The widest cell in each column. No column is measured against the *terminal*: ADR-0031
/// names terminal-width sensitivity among the costs a spatial view would have imposed, and
/// a view with no axis has no reason to take it on.
#[derive(Default)]
struct Widths {
    range: usize,
    span: usize,
    layer: usize,
    kind: usize,
    id: usize,
    track: usize,
}

impl Widths {
    fn over<'a>(rows: impl Iterator<Item = &'a Row> + Clone) -> Widths {
        let widest =
            |cell: fn(&Row) -> &String| width_of(rows.clone().map(|row| cell(row).as_str()));
        Widths {
            range: widest(|row| &row.range),
            span: widest(|row| &row.span),
            layer: widest(|row| &row.layer),
            kind: widest(|row| &row.kind),
            id: widest(|row| &row.id),
            track: widest(|row| &row.track),
        }
    }
}

/// `item-05  0..17472  17472 ms, 11 elements`, or as much of it as the group determines.
fn group_heading(group: &Value) -> String {
    // `CONTEXT.md` makes `group` optional, so the residue needs a name of its own rather
    // than a blank heading that reads as a group whose label went missing.
    let label = match group["group"].as_str() {
        Some(label) => label.to_string(),
        None => "(no group)".to_string(),
    };
    let count = group["elements"].as_array().map_or(0, Vec::len);
    let elements = plural(count as u64, "element");
    match (group["start"].as_i64(), group["end"].as_i64()) {
        (Some(start), Some(end)) => format!(
            "{label}  {start}..{end}  {}, {elements}",
            milliseconds(&group["duration_ms"])
        ),
        _ => format!("{label}  {elements}"),
    }
}

/// `0..3018`, with a question mark for either half the document does not state — never a
/// guess, and never a row silently dropped for being half-written.
fn range_words(row: &Value) -> String {
    match (&row["start"], &row["end"]) {
        (Value::Null, Value::Null) => "—".to_string(),
        (start, end) => format!("{}..{}", stated_number(start), stated_number(end)),
    }
}

/// The widest cell in a column, counted in **characters** rather than bytes: an id or a
/// track name may be any UTF-8 at all, and padding by byte length misaligns the whole
/// column under one non-ASCII character.
///
/// Generic over what a cell is held as, because some columns are borrowed out of the
/// canonical JSON and some are computed on the way past.
fn width_of(cells: impl Iterator<Item = impl AsRef<str>>) -> usize {
    cells
        .map(|cell| cell.as_ref().chars().count())
        .max()
        .unwrap_or(0)
}

/// A name the document writes, or a question mark where it does not. Never `null`: the
/// reader is looking at a view, and the word would read as a value rather than as a gap.
fn named(value: &Value) -> String {
    match value.as_str() {
        Some(name) => name.to_string(),
        None => "?".to_string(),
    }
}

fn stated_number(value: &Value) -> String {
    match value {
        Value::Null => "?".to_string(),
        other => compact(other),
    }
}

fn layer_words(layer: &Value) -> String {
    format!("L{}", stated_number(layer))
}

fn milliseconds(value: &Value) -> String {
    match value {
        Value::Null => "—".to_string(),
        other => format!("{} ms", compact(other)),
    }
}

/// The same pluraliser the summary line uses, over a count the canonical JSON carries.
/// One spelling of "1 element" across the whole report, rather than one per block.
fn counted(count: &Value, noun: &str) -> String {
    plural(count.as_u64().unwrap_or(0), noun)
}

// ---- `measure`'s answer (#205) ------------------------------------------------------

/// What the text occupies, as prose.
///
/// The block's numbers first, then one row per line, then the break opportunities. The
/// order is the order an author reads them in: the block height is the integer they are
/// about to type into `height`, and the opportunities are what they act on only once the
/// extent tells them a break is needed at all.
///
/// Every number the canonical JSON carries prints here, and nothing else: ADR-0006 makes
/// the prose a rendering *of* that JSON, so a sentence stating something the JSON does not
/// is unreachable by construction.
fn measure_block(measure: &Value) -> String {
    let asked = &measure["asked"];
    let lines = measure["lines"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let extent = &measure["extent"];

    let mut out = format!(
        "\nMEASURE  {} in `{}` at size {}, line_height {}, origin {}\n",
        match asked["id"].as_str() {
            Some(id) => format!("`{id}`"),
            // ADR-0024: `measure` must work identically for an element being authored for
            // the first time, and such an element has no `id` to print.
            None => "an element carrying no `id`".to_string(),
        },
        named(&asked["font"]),
        stated_number(&asked["size"]),
        tenths(&asked["line_height_tenths"]),
        named(&asked["origin"]),
    );

    let mut field = |label: &str, value: String| {
        out.push_str(&format!("    {label:<18}{value}\n"));
    };
    field("lines", stated_number(&measure["line_count"]));
    // ADR-0014's consequence, spelled out where it is read rather than left to be
    // inferred: the extent is the stroked one, and the advance beside it is not.
    field(
        "extent (stroked)",
        format!(
            "{} x {} px  (stroke_width {})",
            pixels(&extent["width"]),
            pixels(&extent["height"]),
            stated_number(&extent["stroke_width"])
        ),
    );
    field(
        "advance width",
        format!(
            "{} px (typographic, before the stroke)",
            pixels(&measure["advance_width"])
        ),
    );
    field(
        "ascent / descent",
        format!(
            "{} / {} px  (the maximum across every run on a line — ADR-0029)",
            pixels(&measure["ascent"]),
            pixels(&measure["descent"])
        ),
    );
    field(
        "block height",
        format!(
            "{} px  — the integer `height` takes",
            stated_number(&measure["block_height"])
        ),
    );
    field(
        "block",
        format!(
            "y {} .. {} px",
            pixels(&measure["block_top"]),
            pixels(&measure["block_bottom"])
        ),
    );

    out.push_str("\n  LINES\n");
    for line in lines {
        out.push_str(&row(format!(
            "  {:>3}  baseline_y {:>10}  advance {:>10}  size {:<4} slot {} .. {}  {:?}",
            stated_number(&line["index"]),
            pixels(&line["baseline_y"]),
            pixels(&line["advance_width"]),
            stated_number(&line["size"]),
            pixels(&line["slot_top"]),
            pixels(&slot_bottom(line)),
            line["text"].as_str().unwrap_or_default(),
        )));
    }

    // ADR-0008: Montaget never places a line break itself, so these are offered and never
    // applied. The sentence says so, because a column of offsets does not.
    out.push_str(&format!(
        "\n  BREAK OPPORTUNITIES  byte offsets into the element's text, from {} {} ({})\n  \
         places a `\\n` may legally go — never one Montaget would place itself\n",
        named(&measure["segmenter"]["name"]),
        named(&measure["segmenter"]["version"]),
        named(&measure["segmenter"]["model"]),
    ));
    for line in lines {
        let offsets = line["break_opportunities"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let offsets = if offsets.is_empty() {
            "none".to_string()
        } else {
            offsets
                .iter()
                .map(stated_number)
                .collect::<Vec<_>>()
                .join(" ")
        };
        out.push_str(&row(format!(
            "  {:>3}  {offsets}",
            stated_number(&line["index"])
        )));
    }
    out
}

/// Where one line's slot ends — its top plus its height, so the two edges read as a range
/// rather than as an offset and a length the reader adds in their head.
fn slot_bottom(line: &Value) -> Value {
    match (line["slot_top"].as_f64(), line["slot_height"].as_f64()) {
        (Some(top), Some(height)) => json!(top + height),
        _ => Value::Null,
    }
}

/// A `line_height` held as tenths, printed as the decimal the document writes.
///
/// The conversion is exact and one-way: tenths is what the arithmetic ran on (ADR-0028),
/// and this is the only place it becomes a decimal again — for a human to recognise `1.1`.
fn tenths(value: &Value) -> String {
    match value.as_i64() {
        Some(tenths) => format!("{}.{}", tenths / 10, tenths % 10),
        None => "?".to_string(),
    }
}

/// A pixel measurement, printed without a trailing `.0` on a whole number.
fn pixels(value: &Value) -> String {
    match value.as_f64() {
        Some(px) => {
            let rounded = (px * 1000.0).round() / 1000.0;
            let mut text = format!("{rounded}");
            if let Some(stripped) = text.strip_suffix(".0") {
                text = stripped.to_string();
            }
            text
        }
        None => "?".to_string(),
    }
}

// ---- `query`'s two reading modes (#196) ----------------------------------------------

/// The answer, generated from the `query` block of the canonical JSON.
///
/// Both modes share one heading so that a reader — or a grep — finds the answer in the same
/// place whichever question was asked, and each states its own question underneath it: a
/// cut list read without knowing the range it was taken over is a column of numbers.
fn query_block(query: &Value) -> String {
    match query["mode"].as_str() {
        Some("cuts") => cuts_block(query),
        Some("matches") => matches_block(query),
        // The two modes are an internally-tagged enum, so a third spelling means this
        // renderer is older than the verb it is rendering. Say that, rather than print
        // nothing and let the answer look empty.
        other => format!(
            "\nQUERY\n  this build cannot render a `{}` answer\n",
            other.unwrap_or("(unnamed)")
        ),
    }
}

/// The cut list, with the boundary immediately outside the range named on each side.
///
/// The two outside boundaries are rows of their own rather than extra intervals, and they
/// say which side they are on in words. ADR-0011 asks for them so that *"the caller never
/// has to guess a window"* — a row that looked like an interval would leave the caller
/// working out whether it was inside the range they asked about.
fn cuts_block(cuts: &Value) -> String {
    let intervals = cuts["intervals"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    // The unit and the convention on the heading rather than on every row: ADR-0004 asks
    // that absolute times be unmissable, and ADR-0005 makes the half-open bracket the
    // difference between an element being in an interval and not.
    let mut out = format!(
        "\nQUERY  cut list over [{}, {}) — {} (absolute milliseconds, half-open)\n",
        stated_number(&cuts["from"]),
        stated_number(&cuts["to"]),
        plural(intervals.len() as u64, "interval"),
    );

    out.push_str(&outside_row("before", &cuts["previous"]));

    let ranges: Vec<String> = intervals
        .iter()
        .map(|interval| {
            format!(
                "{}..{}",
                stated_number(&interval["start"]),
                stated_number(&interval["end"])
            )
        })
        .collect();
    let spans: Vec<String> = intervals
        .iter()
        .map(|interval| milliseconds(&interval["duration_ms"]))
        .collect();
    let range_width = width_of(ranges.iter().map(String::as_str));
    let span_width = width_of(spans.iter().map(String::as_str));

    for ((interval, range), span) in intervals.iter().zip(&ranges).zip(&spans) {
        let present = interval["present"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let names: Vec<String> = present.iter().map(|member| named(&member["id"])).collect();
        out.push_str(&row(format!(
            "{range:<range_width$}  {span:>span_width$}  {}",
            // Not a blank cell: an interval nothing is present over is a fact about the
            // project — the frame is the background — and a reader scanning a column of ids
            // would read a blank as a rendering slip.
            match names.is_empty() {
                true => "(nothing present)".to_string(),
                false => names.join(", "),
            }
        )));
    }

    out.push_str(&outside_row("after", &cuts["next"]));

    // Named rather than dropped, and stated where the count is, so a cut list taken over
    // fewer elements than the project has says so next to the answer rather than in JSON
    // the prose reader never sees.
    if let Some(unplaced) = cuts["unplaced"].as_array().filter(|ids| !ids.is_empty()) {
        let names: Vec<String> = unplaced.iter().map(named).collect();
        out.push_str(&row(format!(
            "{} state no integer range and are not in the cut list: {}",
            plural(names.len() as u64, "element"),
            names.join(", "),
        )));
    }
    out
}

/// One row of a block: two-space indent, and never any trailing padding — a column measured
/// to its widest cell would otherwise leave every shorter row with a tail of spaces.
fn row(cells: String) -> String {
    format!("  {}\n", cells.trim_end())
}

/// One of the two boundaries immediately outside the range, or a line saying the document
/// has none on that side.
fn outside_row(side: &str, boundary: &Value) -> String {
    if boundary.is_null() {
        return row(format!(
            "{side}  no boundary outside the range on this side"
        ));
    }
    let names = |key: &str| -> Option<String> {
        let ids: Vec<String> = boundary[key].as_array()?.iter().map(named).collect();
        match ids.is_empty() {
            true => None,
            false => Some(format!("{key} {}", ids.join(", "))),
        }
    };
    let what: Vec<String> = ["entering", "leaving"]
        .iter()
        .filter_map(|key| names(key))
        .collect();
    row(format!(
        "{side}  {}  {}",
        stated_number(&boundary["at"]),
        what.join("; "),
    ))
}

/// The matched set, and the census under it where one was asked for.
fn matches_block(matches: &Value) -> String {
    let matched = matches["matched"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut out = format!(
        "\nQUERY  where {} — {}\n",
        matches["predicate"].as_str().unwrap_or("?"),
        plural(matched.len() as u64, "matched element"),
    );

    let cell = |element: &Value, key: &str| named(&element[key]);
    let widest = |key: &'static str| width_of(matched.iter().map(move |e| cell(e, key)));
    let (id, kind, track) = (widest("id"), widest("type"), widest("track"));

    for element in matched {
        out.push_str(&row(format!(
            "{:<id$}  {:<kind$}  {:<track$}  {}",
            cell(element, "id"),
            cell(element, "type"),
            cell(element, "track"),
            range_words(element),
        )));
    }

    if let Some(census) = matches.get("census").filter(|c| !c.is_null()) {
        out.push_str(&census_block(census));
    }
    out
}

/// *"Four of five are 1597, one is 1537"* — in the order the groups were first seen, and
/// never sorted by size (ADR-0043).
fn census_block(census: &Value) -> String {
    let field = census["field"].as_str().unwrap_or("?");
    let groups: Vec<String> = census["groups"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .map(|group| {
            let members = group["members"].as_array().map_or(0, Vec::len);
            format!("{members} at {}", compact(&group["value"]))
        })
        .collect();

    let mut out = match groups.is_empty() {
        true => format!("  census {field}: no matched element states it\n"),
        false => format!("  census {field}: {}\n", groups.join(", ")),
    };
    // ADR-0030: omitted and explicit-at-default are different declarations, so "states no
    // `y`" is its own line and never a group whose value is `null`.
    if let Some(absent) = census["absent"].as_array().filter(|ids| !ids.is_empty()) {
        let names: Vec<String> = absent.iter().map(named).collect();
        out.push_str(&format!(
            "  {} state no {field}: {}\n",
            plural(names.len() as u64, "matched element"),
            names.join(", "),
        ));
    }
    out
}

// ---- `fonts list` and `fonts vendor` --------------------------------------------------

/// The inventory: one line per face, the licence status first because it is the verdict a
/// vendor attempt would get and the reason the listing exists (ADR-0057).
fn fonts_block(fonts: &Value) -> String {
    let roots = fonts["roots"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let faces = fonts["fonts"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let unreadable = fonts["unreadable"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut out = format!(
        "\nFONTS  {} under {}\n",
        counted(&json!(faces.len()), "face"),
        if roots.is_empty() {
            "no directory that exists".to_string()
        } else {
            roots.iter().map(named).collect::<Vec<_>>().join(", ")
        }
    );
    let status_width = width_of(faces.iter().map(|f| f["status"].as_str().unwrap_or("")));
    for face in faces {
        let status = face["status"].as_str().unwrap_or("?");
        let matched = match face["matched"].as_str() {
            Some(matched) => format!(" ({matched})"),
            None => String::new(),
        };
        out.push_str(&format!(
            "  {status:<status_width$}{matched:<14}  {}#{}  {}{}\n",
            face["path"].as_str().unwrap_or("?"),
            face["index"],
            named(&face["family"]),
            match face["postscript"].as_str() {
                Some(postscript) => format!(" [{postscript}]"),
                None => String::new(),
            },
        ));
    }
    if !unreadable.is_empty() {
        out.push_str("  unreadable:\n");
        for file in unreadable {
            out.push_str(&format!(
                "    {}: {}\n",
                file["path"].as_str().unwrap_or("?"),
                file["reason"].as_str().unwrap_or("?")
            ));
        }
    }
    out
}

/// What `fonts vendor` wrote: the attestation, and the chain entry to write next.
fn vendor_block(vendor: &Value) -> String {
    let attestation = &vendor["attestation"];
    let mut out = format!(
        "\nVENDORED  {}{}\n",
        vendor["file"].as_str().unwrap_or("?"),
        if vendor["already_present"].as_bool().unwrap_or(false) {
            "  (already present with these exact bytes; attestation rewritten)"
        } else {
            ""
        }
    );
    let mut field = |label: &str, value: String| {
        out.push_str(&format!("    {label:<12}{value}\n"));
    };
    field(
        "licence",
        format!(
            "{} ({})",
            attestation["licence"].as_str().unwrap_or("?"),
            vendor["licence_basis"].as_str().unwrap_or("?")
        ),
    );
    field(
        "source",
        attestation["source"].as_str().unwrap_or("?").to_string(),
    );
    field(
        "sha256",
        attestation["sha256"].as_str().unwrap_or("?").to_string(),
    );
    field("chain entry", compact(&vendor["chain_entry"]));
    out
}
