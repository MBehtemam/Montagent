//! `probe` — *"the one authority on what a media file's numbers are"* (ADR-0011).
//!
//! CLI-only, by ADR-0011's cost model: *"every MCP tool schema occupies the agent's
//! context and degrades tool selection on every turn, including turns with nothing to do
//! with video. A CLI subcommand costs nothing until invoked."*
//!
//! The answer has three parts, in the order they print:
//!
//! 1. **The cache misses**, unprompted and at the top. ADR-0006 asks for exactly that, and
//!    ADR-0011 upgrades it from a side effect to *"the sole mechanism"* catching a source
//!    that grew on disk. It is never conditional on a flag.
//! 2. **The media facts** — ADR-0011's quad, the ADR-0023 dimensions, alpha, sample rate
//!    and channels. No field is named `duration`.
//! 3. **The findings**, through the same report surface every other verb answers with, so
//!    an absent source and an unreachable one stay the two different things ADR-0053 and
//!    ADR-0056 insist they are.

use std::path::Path;

use serde_json::{Value, json};

use crate::media::probe::Outcome;
use crate::media::session::{CacheMiss, Session};
use crate::media::{Source, tools};
use crate::report::Report;
use crate::text;

const TOOL: &str = "probe";

/// One `probe` invocation's answer.
pub struct Answer {
    /// The facts, one per source asked about, in the order they were asked about.
    pub sources: Vec<Outcome>,
    /// Every probe that actually ran, rather than being answered from the cache.
    pub misses: Vec<CacheMiss>,
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
        let mut json = self.report.to_json();
        let object = json
            .as_object_mut()
            .expect("a report serialises as an object");
        object.insert("sources".into(), json!(self.sources));
        object.insert("cache_misses".into(), json!(self.misses));
        object.insert("network_attempts".into(), json!(self.network_attempts));
        json
    }
}

/// Probe every source named, resolving a relative one against the working directory.
///
/// The error is ADR-0011's exit 70 and nothing else: there is no `ffprobe`, so there are
/// no media numbers to be right or wrong about, and saying anything about the project
/// would be inventing it.
pub fn probe(sources: &[String]) -> Result<Answer, Report> {
    let mut session = Session::open().map_err(|missing| missing.into_report())?;
    session.begin_run();
    // The base a bare `probe` resolves against is the working directory, and the adapter
    // does not get to decide that: what a relative `source` resolves against is ADR-0053's
    // question, and it is answered in the core (ADR-0011 — an adapter contains no rule).
    Ok(probe_with(&mut session, Path::new("."), sources))
}

/// The same, against a session and a base directory the caller owns — an MCP server's warm
/// cache, a project file's own directory (ADR-0053), or a test's recorded `ffprobe`.
pub fn probe_with(session: &mut Session, base: &Path, sources: &[String]) -> Answer {
    let mut report = Report::new(TOOL, None);
    let mut outcomes = Vec::new();

    for source in sources {
        let resolved = Source::resolve(source, base);
        match session.probe(&resolved) {
            Ok(outcome) => {
                if let Some(finding) = finding_for(&outcome) {
                    report.push(finding);
                }
                outcomes.push(outcome);
            }
            // `ffprobe` resolved and would not run. Nothing after this point can be
            // established, and a partial answer would be a report that had quietly
            // stopped looking.
            Err(missing) => return failed(*missing, outcomes, session),
        }
    }

    Answer {
        sources: outcomes,
        misses: session.misses().to_vec(),
        network_attempts: session.network_attempts(),
        report,
    }
}

fn failed(missing: tools::Missing, outcomes: Vec<Outcome>, session: &Session) -> Answer {
    Answer {
        sources: outcomes,
        misses: session.misses().to_vec(),
        network_attempts: session.network_attempts(),
        report: missing.into_report(),
    }
}

/// The finding an outcome carries, where it carries one.
///
/// This is the boundary ADR-0053 and ADR-0056 draw, in one function so it cannot be drawn
/// differently by two callers: a **confirmed** absence is a plain `error`; a probe that
/// could not complete is `UNCHECKED` with a structured reason; and a reach that confirmed
/// existence and learned nothing else is `UNCHECKED` under its own code, never in the slot
/// a confirmed duration occupies.
pub fn finding_for(outcome: &Outcome) -> Option<crate::finding::Finding> {
    use crate::finding::Finding;

    match outcome {
        Outcome::Probed(_) => None,
        Outcome::Missing { source, detail } => Some(
            Finding::new("E-SOURCE-MISSING")
                .field("source", json!(source))
                .field("detail", json!(detail))
                .repair_value(
                    json!({"value": "correct the source, or put the file where it says"}),
                ),
        ),
        Outcome::ExistenceOnly { source, detail } => Some(
            Finding::new("U-SOURCE-EXISTENCE-ONLY")
                .field("source", json!(source))
                .field("detail", json!(detail)),
        ),
        Outcome::Unchecked {
            source,
            reason,
            detail,
        } => {
            let finding = Finding::new("U-SOURCE-UNPROBEABLE")
                .field("source", json!(source))
                .field("detail", json!(detail));
            Some(match reason {
                Some(reason) => finding.unchecked_because(reason.clone()),
                None => finding,
            })
        }
    }
}

/// The prose form, generated from the canonical JSON and nothing else — the facts block
/// included, so the two forms cannot drift apart.
///
/// Called through [`crate::wire::render_answer`], never directly by an adapter: the rule
/// about which form prints belongs to [`crate::wire::Wire`].
pub fn render_text(json: &Value, options: text::Options) -> Result<String, text::RenderError> {
    let mut out = String::new();

    // ADR-0006: "Then report the cache miss, unprompted, at the top." Not behind
    // `--verbose`, and not omitted when it is the only thing that changed.
    if let Some(misses) = json["cache_misses"].as_array().filter(|m| !m.is_empty()) {
        out.push_str("CACHE\n");
        for miss in misses {
            out.push_str("  ");
            out.push_str(&miss_line(miss));
            out.push('\n');
        }
        out.push('\n');
    }

    if let Some(sources) = json["sources"].as_array().filter(|s| !s.is_empty()) {
        out.push_str("MEDIA\n");
        for source in sources {
            out.push_str(&source_block(source));
        }
        out.push('\n');
    }

    out.push_str(&text::render(json, options)?);
    Ok(out)
}

fn miss_line(miss: &Value) -> String {
    let source = miss["source"].as_str().unwrap_or("?");
    match miss["kind"].as_str() {
        Some("first") => format!("{source} — probed (not yet in this session's cache)"),
        Some("changed") => format!(
            "{source} — CHANGED ON DISK since it was last probed: {} bytes → {} bytes",
            miss["previous_size"], miss["size"]
        ),
        Some("remote") => {
            format!("{source} — fetched (remote sources are never cached across runs)")
        }
        _ => format!("{source} — probed"),
    }
}

fn source_block(source: &Value) -> String {
    if source["state"] != "probed" {
        // Every other outcome has a finding under it, which the report prints in its own
        // section. Repeating the prose here would put one fact in two voices.
        return String::new();
    }
    let probed = source;

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
