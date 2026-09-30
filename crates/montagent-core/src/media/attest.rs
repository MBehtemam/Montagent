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
//!
//! ## `montagent/2`: the stamp says *what* was rendered, not only *who* rendered it
//!
//! ADR-0117 (#436) adds a **staleness digest** so `verify` can ask *"is this file the
//! rendering of this document as it stands now?"* — a question the project name alone cannot
//! answer, because a project that renders and is then edited is still the same project.
//! [`crate::media::digest`] computes it. The engine version rides beside it for diagnosis and
//! is deliberately **not** in it: every upgrade would otherwise make every deliverable stale,
//! which is *loud on the safe case*, the failure ADR-0104 §1 rejected.
//!
//! Ownership is still the project identity and nothing else. `render`'s pre-flight reads a
//! `montagent/1` stamp and a `montagent/2` stamp alike as [`Attestation::Mine`], so the first
//! render after an upgrade does not read its own last deliverable as somebody else's.

use std::path::Path;

use super::probe::{Execution, LOCAL_PROTOCOLS, Runner};
use super::tools::Tools;

/// The stamp's version prefix, as this binary writes it. It is in the value rather than
/// implied so that a later grammar can be told from an unreadable one.
///
/// `montagent/2` since ADR-0117: `montagent/2 engine=<version> digest=<hex|none> project=<path>`.
/// `project=` is last because it is the one field that may contain spaces.
pub const MARK: &str = "montagent/2";

/// ADR-0104's first grammar, `montagent/1 project=<path>`. Still read, never written: a
/// deliverable rendered before ADR-0117 is this project's, with its staleness unknown.
pub const MARK_V1: &str = "montagent/1";

/// The container tag the stamp rides in.
///
/// `comment` and not a custom key: the MP4 muxer maps a fixed set of tags to their `©`-prefixed
/// atoms and silently drops anything else unless `-movflags use_metadata_tags` is set, which
/// would change the container's shape for every consumer. `comment` is mapped, round-trips
/// spaces, and is the one tag a human reading the file's properties would expect to find a
/// provenance note in.
pub const TAG: &str = "comment";

/// The engine version a `montagent/2` stamp records. For diagnosis only — see the module doc.
pub const ENGINE: &str = env!("CARGO_PKG_VERSION");

/// One stamp, read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    /// `1` or `2`: which grammar wrote it.
    pub version: u8,
    /// The engine that rendered the file, where the grammar records one.
    pub engine: Option<String>,
    /// ADR-0117's staleness digest, or `None` where the stamp carries none: a `montagent/1`
    /// stamp, or a render that could not fingerprint one of its inputs and so wrote `none`
    /// rather than a digest of less than it rendered.
    pub digest: Option<String>,
    /// The canonical path of the project file that rendered it (ADR-0092's observed identity).
    pub project: String,
}

impl Stamp {
    /// Read one `comment` value as a stamp of ours, or `None` where it is not one.
    pub fn parse(value: &str) -> Option<Stamp> {
        if let Some(project) = value.strip_prefix(&format!("{MARK_V1} project=")) {
            return Some(Stamp {
                version: 1,
                engine: None,
                digest: None,
                project: project.to_string(),
            });
        }
        let rest = value.strip_prefix(&format!("{MARK} "))?;
        let rest = rest.strip_prefix("engine=")?;
        let (engine, rest) = rest.split_once(' ')?;
        let rest = rest.strip_prefix("digest=")?;
        let (digest, rest) = rest.split_once(' ')?;
        let project = rest.strip_prefix("project=")?;
        Some(Stamp {
            version: 2,
            engine: Some(engine.to_string()),
            digest: (digest != "none").then(|| digest.to_string()),
            project: project.to_string(),
        })
    }
}

/// What the file at a path says about who wrote it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attestation {
    /// Nothing is at that path. The render is free.
    Vacant,
    /// This project's own stamp, of either grammar. Re-rendering over your own last answer is
    /// the normal loop and is never a finding; `verify` reads the digest off it.
    Mine(Stamp),
    /// A file written by some other project, naming it.
    Foreign { project: String },
    /// A file with no Montagent stamp at all — a human's, another tool's, or a deliverable
    /// from before this check existed. Montagent has no evidence either way, which is a
    /// weaker statement than [`Attestation::Foreign`] and is classed weaker.
    Absent,
}

/// The project file's **observed identity** in ADR-0092's sense — the canonical path observed
/// now — and deliberately not a label re-derived from the working directory, which is the bug
/// ADR-0092 exists to have fixed. The cost is stated rather than hidden: a project tree that
/// moves reads its own earlier deliverables as `Foreign`.
pub fn identity(project: &Path) -> String {
    std::fs::canonicalize(project)
        .unwrap_or_else(|_| project.to_path_buf())
        .display()
        .to_string()
}

/// The stamp this project writes: its identity, this engine, and the digest of what it is
/// about to render — or `none` where the digest could not be computed.
pub fn stamp(project: &Path, digest: Option<&str>) -> String {
    format!(
        "{MARK} engine={ENGINE} digest={} project={}",
        digest.unwrap_or("none"),
        identity(project)
    )
}

/// Read what is at `output` and say whose it is, against the project at `project`.
///
/// A file that cannot be probed at all is `Foreign` with an unnamed project rather than
/// `Absent`: bytes exist at the path, the render is about to destroy them, and "I could not
/// read it" is the least safe moment to assume it is nobody's.
pub fn of(runner: &dyn Runner, tools: &Tools, output: &Path, project: &Path) -> Attestation {
    if !output.exists() {
        return Attestation::Vacant;
    }
    let Some(value) = read_tag(runner, tools, output) else {
        return Attestation::Absent;
    };
    match Stamp::parse(&value) {
        Some(stamp) if stamp.project == identity(project) => Attestation::Mine(stamp),
        Some(stamp) => Attestation::Foreign {
            project: stamp.project,
        },
        // Something wrote a comment that is not a stamp of ours. The file is not this
        // project's and that is all this module claims about it.
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
