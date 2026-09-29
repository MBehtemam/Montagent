//! What wrote the file at the output path, and whether it was this project.
//!
//! [#391](https://github.com/MBehtemam/Montagent/issues/391), ADR-0104. `render`'s promotion
//! is one rename onto a path the document named, and until now nothing looked at what was
//! already there. A sibling project that inherited a hardcoded `output` would have renamed
//! an eleven-minute finished cut out of existence at `0 errors`.
//!
//! **The predicate is an observed fact, never a re-derived one.** The obvious cheap test —
//! does the existing file's duration or frame count disagree with the document? — is exactly
//! the re-derived label ADR-0092 ruled is not an identity, and its two failure modes invert
//! the defect: two language cuts of one timeline match to the frame, and a project matches
//! nothing after its own first edit. So the encoder **stamps** what it writes, and this
//! module reads that stamp back. A file either carries this project's attestation or it does
//! not, and the answer is read rather than guessed.
//!
//! The same move `fontVendor` already makes for font bytes (ADR-0057) and `E-NOT-A-PROJECT`
//! already makes for `fmt`'s wrong-file destruction (ADR-0042): where the hazard is
//! destroying the wrong file, the proportionate check is an identity check.

use std::path::Path;

use super::probe::{Execution, LOCAL_PROTOCOLS, Runner};
use super::tools::Tools;

/// The stamp's version prefix. It is in the value rather than implied so that a later
/// grammar can be told from an unreadable one — an unparseable stamp is [`Attestation::Foreign`],
/// not [`Attestation::Absent`], because something wrote it and it was not this run.
pub const MARK: &str = "montagent/1";

/// The container tag the stamp rides in.
///
/// `comment` and not a custom key: the MP4 muxer maps a fixed set of tags to their `©`-prefixed
/// atoms and silently drops anything else unless `-movflags use_metadata_tags` is set, which
/// would change the container's shape for every consumer. `comment` is mapped, round-trips
/// spaces, and is the one tag a human reading the file's properties would expect to find a
/// provenance note in.
pub const TAG: &str = "comment";

/// What the file at a path says about who wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attestation {
    /// Nothing is at that path. The render is free.
    Vacant,
    /// This project's own stamp. Re-rendering over your own last answer is the normal loop
    /// and is never a finding.
    Mine,
    /// A file written by some other project, naming it.
    Foreign { project: String },
    /// A file with no Montagent stamp at all — a human's, another tool's, or a deliverable
    /// from before this check existed. Montagent has no evidence either way, which is a
    /// weaker statement than [`Attestation::Foreign`] and is classed weaker.
    Absent,
}

/// The stamp this project writes, and matches against.
///
/// The value is the project file's **observed identity** in ADR-0092's sense — the canonical
/// path observed at the moment the render ran — and deliberately not a label re-derived from
/// the working directory, which is the bug ADR-0092 exists to have fixed. The cost is stated
/// rather than hidden: a project tree that moves reads its own earlier deliverables as
/// `Foreign`.
pub fn stamp(project: &Path) -> String {
    let identity = std::fs::canonicalize(project)
        .unwrap_or_else(|_| project.to_path_buf())
        .display()
        .to_string();
    format!("{MARK} project={identity}")
}

/// Read what is at `output` and say whose it is.
///
/// A file that cannot be probed at all is `Foreign` with an unnamed project rather than
/// `Absent`: bytes exist at the path, the render is about to destroy them, and "I could not
/// read it" is the least safe moment to assume it is nobody's.
pub fn of(runner: &dyn Runner, tools: &Tools, output: &Path, mine: &str) -> Attestation {
    if !output.exists() {
        return Attestation::Vacant;
    }
    match read_tag(runner, tools, output) {
        Some(value) if value == mine => Attestation::Mine,
        Some(value) => match value.strip_prefix(&format!("{MARK} project=")) {
            Some(project) => Attestation::Foreign {
                project: project.to_string(),
            },
            // Something wrote a comment that is not a stamp of ours. The file is not this
            // project's and that is all this module claims about it.
            None => Attestation::Absent,
        },
        None => Attestation::Absent,
    }
}

/// The `format.tags.comment` of one local file, or `None` when there is not one.
fn read_tag(runner: &dyn Runner, tools: &Tools, file: &Path) -> Option<String> {
    let args: Vec<String> = [
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_format",
        "-protocol_whitelist",
        LOCAL_PROTOCOLS,
    ]
    .iter()
    .map(|s| s.to_string())
    .chain(std::iter::once(file.display().to_string()))
    .collect();

    let Execution {
        success, stdout, ..
    } = runner.run(&tools.ffprobe, &args).ok()?;
    if !success {
        return None;
    }
    let parsed: serde_json::Value = serde_json::from_str(&stdout).ok()?;
    parsed
        .get("format")?
        .get("tags")?
        .get(TAG)?
        .as_str()
        .map(str::to_string)
}
