//! `probe` — the one authority on what a media file's numbers are.
//!
//! ADR-0011: *"`probe` therefore returns the quad and forces the caller to pick: video
//! duration, container duration, `start_time`, and both frame rates — plus pixel
//! dimensions, alpha, sample rate and channels. **A single scalar named `duration` is the
//! failure mode**, because it invites arithmetic against the wrong axis. It returns
//! **integer milliseconds**; the float-seconds conversion is not the agent's to fumble."*
//!
//! So there is no `duration` field on [`Probe`], no method that produces one, and
//! `tests/probe.rs` walks the serialised form asserting the key does not appear anywhere
//! in it. The caller picks the axis its arithmetic is against, and the pick is visible in
//! the caller's code.
//!
//! ## The network
//!
//! This module holds **the only code in Montagent that can reach the network**, and it can
//! only do so for a [`Source::Remote`]. A local probe passes `ffprobe` a protocol
//! whitelist of `file,crypto,data`, so a local file that names a remote resource inside
//! itself — a playlist, a concat script — cannot turn a local probe into a fetch. That is
//! ADR-0056's *"no unsolicited network call"* as an argument to the subprocess rather than
//! as a promise in a doc comment.
//!
//! The remote fetch is `ffprobe`'s own ranged read rather than an HTTP client of
//! Montagent's. ADR-0056 asks for *"an HTTP range request feeding the same
//! duration/dimension extraction the local probe already does"*, and the surest way to
//! feed the *same* extraction is to run the same extractor: one code path, one parser, and
//! no second opinion about what a container says. The degradation ADR-0056 requires is
//! kept explicit in [`Outcome`] — an existence-only result never occupies the slot a
//! confirmed duration does.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::dimensions::{self, Decoded, Rotation, RotationSource, SourceDimensions};
use super::tools::{Missing, Tools};
use super::{Rational, milliseconds};
use crate::finding::{Finding, UncheckedReason};

/// ADR-0011's quad, which is the whole reason `probe` exists.
///
/// On the fixture's reference MP4 these disagree — 65.216016 s on the stream, 65.258667 s
/// on the container, a 42 ms `start_time`, and two frame rates that differ by 2× — and
/// the project declares numbers matching the stream and neither frame rate. Every field
/// is optional because a still image has none of them and an MP3 has only some.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Quad {
    /// The video stream's own length, in integer milliseconds.
    pub video_stream_ms: Option<i64>,
    /// The container's declared length, in integer milliseconds.
    pub container_ms: Option<i64>,
    /// The first presentation timestamp, in integer milliseconds.
    pub start_time_ms: Option<i64>,
    /// `r_frame_rate` — the base rate the container ticks at.
    pub r_frame_rate: Option<Rational>,
    /// `avg_frame_rate` — frames divided by duration. On the reference MP4 this is
    /// `1390080/55651`, which is why it is kept as a ratio and not as `24.9785`.
    pub avg_frame_rate: Option<Rational>,
}

impl Quad {
    /// Whether the two durations disagree — the condition that makes a single scalar
    /// named `duration` a lie rather than a shorthand.
    pub fn durations_disagree(&self) -> bool {
        match (self.video_stream_ms, self.container_ms) {
            (Some(stream), Some(container)) => stream != container,
            _ => false,
        }
    }
}

/// What an audio stream says about itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Audio {
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    /// The audio stream's own length. Named for its axis, like every other duration here.
    pub audio_stream_ms: Option<i64>,
}

/// Everything `probe` establishes about one source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Probe {
    /// The source as the document spells it, or as it resolved on disk.
    pub source: String,
    pub quad: Quad,
    /// ADR-0023's resolved source dimensions, with the inputs that produced them.
    pub dimensions: Option<SourceDimensions>,
    /// Whether the decoded pixel format carries an alpha channel.
    pub alpha: Option<bool>,
    pub audio: Option<Audio>,
}

/// What a probe attempt established. ADR-0056 requires these to stay distinct: *"an
/// existence-only result is a strictly weaker claim than a local probe's, and reporting it
/// identically would be exactly the false confidence ADR-0006 was written to prevent."*
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Outcome {
    /// Real content facts.
    Probed(Probe),
    /// The source is there and nothing about its content was established. ADR-0056:
    /// *"existence confirmed (HEAD 200), duration NOT CHECKED"*.
    ExistenceOnly { source: String, detail: String },
    /// A *confirmed* absence — a missing local file, or a server that explicitly refuses
    /// the object. ADR-0053: a plain `error`, and never to be confused with the one below.
    Missing { source: String, detail: String },
    /// Nothing was learned. ADR-0056: *"request timeout, DNS failure, host unreachable:
    /// none of these is evidence the file is missing — each is evidence that nothing was
    /// learned."*
    ///
    /// `reason` is present exactly when the failure was a network one, which is how a
    /// network-flavoured unknown stays distinguishable from an unattempted one without
    /// either being promoted into its own severity.
    Unchecked {
        source: String,
        reason: Option<UncheckedReason>,
        detail: String,
    },
}

impl Outcome {
    /// The facts, where the attempt established any.
    pub fn probe(&self) -> Option<&Probe> {
        match self {
            Outcome::Probed(probe) => Some(probe),
            _ => None,
        }
    }
}

/// One `ffprobe` run, as values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    pub success: bool,
    /// The process's exit status, or `None` where a signal killed it.
    ///
    /// Carried rather than collapsed into `success` because the *value* discriminates:
    /// `ffprobe` answers 1 for a file it cannot decode, and a shell answers 126/127 for a
    /// thing it could not execute at all. One is a fact about the media and the other is
    /// ADR-0011's exit 70, and a bool cannot tell them apart.
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// How a probe reaches `ffprobe`.
///
/// A seam, and the reason for one is specific: the behaviours this ticket must prove —
/// a 404 that is an `error`, a DNS failure that is `UNCHECKED`, a local probe that cannot
/// reach the network — are each a property of *what Montagent does with what `ffprobe`
/// said*, and reproducing them against real servers would make the test suite depend on a
/// network it is not allowed to touch.
pub trait Runner {
    fn run(&self, program: &Path, args: &[String]) -> std::io::Result<Execution>;
}

/// The real one: spawn the resolved binary.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessRunner;

impl Runner for ProcessRunner {
    fn run(&self, program: &Path, args: &[String]) -> std::io::Result<Execution> {
        let output = std::process::Command::new(program).args(args).output()?;
        Ok(Execution {
            success: output.status.success(),
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// The arguments every probe shares. `-v error` because `ffprobe`'s banner is not a fact
/// about the media.
fn base_args() -> Vec<String> {
    [
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// The protocols a **local** probe may use. Not a defensive nicety: without it, a local
/// file that names a remote resource inside itself would turn a local probe into a fetch,
/// and ADR-0056's *"no unsolicited network call"* would hold only for sources the document
/// spells as URLs.
pub const LOCAL_PROTOCOLS: &str = "file,crypto,data";

/// The protocols a **remote** probe may use. Reached only from [`probe_remote`], which is
/// reached only for a [`Source::Remote`].
pub const REMOTE_PROTOCOLS: &str = "file,crypto,data,http,https,tcp,tls";

/// A bounded round trip, in microseconds. An unbounded one would hang `validate` on a
/// black-holed host rather than reporting ADR-0056's `timeout`.
const REMOTE_TIMEOUT_US: &str = "15000000";

/// Probe a local file.
///
/// The missing-file case is answered by `stat` before any subprocess exists: ADR-0053
/// makes a confirmed-missing source *"a plain `error`"*, and that is a fact about the
/// filesystem, not something to infer from a decoder's prose.
pub fn probe_local(
    runner: &dyn Runner,
    tools: &Tools,
    path: &Path,
) -> Result<Outcome, Box<Missing>> {
    // What actually gets passed to `ffprobe` — the platform's own spelling, extended-length
    // prefix and all, since that prefix is what lets Windows open a path past `MAX_PATH`.
    let opened = path.display().to_string();
    // What the *report* names: the same path, with that prefix stripped (`display_local`).
    let display = super::display_local(path);

    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            return Ok(Outcome::Missing {
                source: display,
                detail: "that is a directory, not a media file".to_string(),
            });
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Outcome::Missing {
                source: display,
                detail: e.to_string(),
            });
        }
        // Present but unreadable: something is there, so it is not a confirmed absence,
        // and nothing about its content was learned. That is `UNCHECKED` with no network
        // reason — the "unattempted" half of ADR-0056's distinction.
        Err(e) => {
            return Ok(Outcome::Unchecked {
                source: display,
                reason: None,
                detail: e.to_string(),
            });
        }
        Ok(_) => {}
    }

    let mut args = base_args();
    args.push("-protocol_whitelist".to_string());
    args.push(LOCAL_PROTOCOLS.to_string());
    args.push(opened);

    let execution = execute(runner, tools, &args)?;
    interpret(tools, &display, execution, false)
}

/// Probe a remote URL. **The only function in Montagent that can cause a network call.**
pub fn probe_remote(
    runner: &dyn Runner,
    tools: &Tools,
    url: &str,
) -> Result<Outcome, Box<Missing>> {
    let mut args = base_args();
    args.push("-protocol_whitelist".to_string());
    args.push(REMOTE_PROTOCOLS.to_string());
    args.push("-rw_timeout".to_string());
    args.push(REMOTE_TIMEOUT_US.to_string());
    args.push(url.to_string());

    let execution = execute(runner, tools, &args)?;
    interpret(tools, url, execution, true)
}

/// Run `ffprobe`, turning a spawn failure into ADR-0011's exit 70 rather than into a
/// finding about the project. A binary that will not start says nothing about the media.
fn execute(runner: &dyn Runner, tools: &Tools, args: &[String]) -> Result<Execution, Box<Missing>> {
    runner.run(&tools.ffprobe, args).map_err(|e| {
        Box::new(Missing {
            program: "ffprobe",
            searched: Vec::new(),
            resolved: Some(tools.ffprobe.clone()),
            failure: e.to_string(),
        })
    })
}

/// Turn one `ffprobe` run into an outcome.
fn interpret(
    tools: &Tools,
    source: &str,
    execution: Execution,
    remote: bool,
) -> Result<Outcome, Box<Missing>> {
    // Before anything is read as a fact about the media, ask whether the thing that
    // answered was an `ffprobe` doing its job at all.
    if let Some(failure) = tool_failure(&execution) {
        return Err(Box::new(Missing {
            program: "ffprobe",
            searched: Vec::new(),
            resolved: Some(tools.ffprobe.clone()),
            failure,
        }));
    }

    if !execution.success {
        return Ok(failure(source, &execution.stderr, remote));
    }

    let value = serde_json::from_str::<Value>(&execution.stdout)
        .expect("tool_failure rejects stdout that is not JSON");

    Ok(match read(source, &value) {
        Some(probe) => Outcome::Probed(probe),
        // `ffprobe` answered and the answer carries no stream at all. Something is there
        // — the reach succeeded — and nothing about its content was established.
        None => Outcome::ExistenceOnly {
            source: source.to_string(),
            detail: "ffprobe reported no streams, so duration and dimensions are NOT CHECKED"
                .to_string(),
        },
    })
}

/// Whether this run failed because the **tool** would not do the job, rather than because
/// the **media** would not answer — and, if so, what to tell exit 70.
///
/// ADR-0011 gives exit 70 to *"internal failure (ffmpeg died, font stack failed)"* and its
/// next move is *"retry or report"*, against exit 1's *"fix the project"*. ADR-0009 makes
/// this a live distinction rather than a pedantic one: Montagent ships as *"a binary, plus
/// an `ffmpeg` the user supplies"*, so the supplied one may be too old to know a flag, or
/// not be an `ffprobe` at all. Reporting that as `U-SOURCE-UNPROBEABLE` would blame the
/// media for a broken tool and exit 0 while doing it.
///
/// Every signal here was measured against a real `ffprobe` 8.0.1 and against three broken
/// stubs; a file that genuinely will not decode exits 1 and diagnoses the file, and is
/// deliberately not caught by any of them.
fn tool_failure(execution: &Execution) -> Option<String> {
    // The shell's own codes for "cannot execute" and "not found", and a death by signal —
    // none of which `ffprobe` produces for any media.
    match execution.code {
        Some(126 | 127) => {
            return Some(format!(
                "the shell could not execute it ({}): {}",
                execution.code.unwrap_or_default(),
                first_line(&execution.stderr)
            ));
        }
        None => {
            return Some(format!(
                "it was killed by a signal before it answered: {}",
                first_line(&execution.stderr)
            ));
        }
        _ => {}
    }

    // An option Montagent passes that this build does not know. The network fence is one of
    // those options, so this must never be swallowed.
    const REJECTED: &[&str] = &[
        "Unrecognized option",
        "Unknown option",
        "Option not found",
        "Error splitting the argument list",
    ];
    if !execution.success {
        if let Some(needle) = REJECTED
            .iter()
            .find(|needle| execution.stderr.contains(*needle))
        {
            return Some(format!(
                "it rejected an option Montagent passes ({needle}): {}",
                first_line(&execution.stderr)
            ));
        }
        return None;
    }

    // It exited 0 and did not answer in the format that was asked for, so `-print_format
    // json` was not honoured and whatever ran is not an `ffprobe`. "No streams" would be a
    // claim about media nobody looked at.
    if serde_json::from_str::<Value>(&execution.stdout).is_err() {
        return Some(format!(
            "it exited 0 but its output is not the JSON `-print_format json` asks for: {}",
            if execution.stdout.trim().is_empty() {
                "it printed nothing at all".to_string()
            } else {
                first_line(&execution.stdout)
            }
        ));
    }

    None
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_string()
}

/// Classify a failed run. ADR-0053 and ADR-0056 draw the line this function implements:
/// a *confirmed* absence is an `error`, and every other failure is `UNCHECKED`.
fn failure(source: &str, stderr: &str, remote: bool) -> Outcome {
    let detail = stderr.trim().to_string();
    let detail = if detail.is_empty() {
        "ffprobe failed and said nothing".to_string()
    } else {
        detail
    };

    if !remote {
        // A local file that exists and will not decode. Not a network unknown, and not an
        // absence — the file is right there.
        return Outcome::Unchecked {
            source: source.to_string(),
            reason: None,
            detail,
        };
    }

    match http_status(&detail) {
        // ADR-0056: *"a confirmed absence (404, a resolved host that explicitly refuses
        // the object) is the plain `error` that ADR-0053 fixed"*.
        Some(status @ (404 | 410)) => Outcome::Missing {
            source: source.to_string(),
            detail: format!("the server refused the object: HTTP {status}. {detail}"),
        },
        // ADR-0056's degradation: *"before falling back to an existence-only check … for a
        // URL or protocol that won't support partial reads"*. Each of these is a server
        // answering **about this object** — the range was refused (416), the method or
        // feature is not implemented (405, 501) — which establishes that the object is
        // there and establishes nothing about its content. A status that says the object
        // exists is not a status that says nothing was learned, so it is neither the error
        // above nor the unknown below.
        Some(status @ (405 | 416 | 501)) => Outcome::ExistenceOnly {
            source: source.to_string(),
            detail: format!(
                "the server answered about the object (HTTP {status}) but would not serve a \
                 partial read, so duration and dimensions are NOT CHECKED. {detail}"
            ),
        },
        Some(status) => Outcome::Unchecked {
            source: source.to_string(),
            reason: Some(UncheckedReason::Http { status }),
            detail,
        },
        None => Outcome::Unchecked {
            source: source.to_string(),
            reason: Some(network_reason(&detail)),
            detail,
        },
    }
}

/// The HTTP status in an `ffprobe` error, where it names one.
///
/// `ffprobe` prints *"Server returned 404 Not Found"* and *"Server returned 5XX Server
/// Error reply"*; the second spells the class with an `X` rather than naming the code, so
/// that spelling maps to the class's first code rather than being read as a number.
fn http_status(stderr: &str) -> Option<u16> {
    let after = stderr.find("Server returned ")? + "Server returned ".len();
    let token: String = stderr[after..]
        .chars()
        .take_while(|c| !c.is_whitespace())
        .collect();

    if let Ok(status) = token.parse::<u16>() {
        return (100..600).contains(&status).then_some(status);
    }
    // `4XX`/`5XX`: the class is known and the code is not.
    let mut chars = token.chars();
    let class = chars.next()?.to_digit(10)?;
    (chars.all(|c| c.eq_ignore_ascii_case(&'x')) && (1..6).contains(&class))
        .then_some((class * 100) as u16)
}

/// ADR-0056's structured reason, from what the transport said.
///
/// The enumeration is exactly the ADR's — `timeout` / `dns` / `unreachable` / an HTTP
/// status — so anything unrecognised lands on `unreachable`: it is the reason that claims
/// the least, and inventing a fifth would be the severity-fragmentation the ADR refuses.
fn network_reason(stderr: &str) -> UncheckedReason {
    let text = stderr.to_ascii_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|needle| text.contains(needle));

    if has(&[
        "name or service not known",
        "temporary failure in name resolution",
        "failed to resolve hostname",
        "no address associated with hostname",
        "nodename nor servname",
        "name does not resolve",
    ]) {
        UncheckedReason::Dns
    } else if has(&["timed out", "timeout", "operation now in progress"]) {
        UncheckedReason::Timeout
    } else {
        UncheckedReason::Unreachable
    }
}

/// Read `ffprobe`'s JSON into the quad, the dimensions and the rest.
///
/// `None` when the file carries no stream at all — the existence-only case.
fn read(source: &str, value: &Value) -> Option<Probe> {
    let streams = value.get("streams")?.as_array()?;
    if streams.is_empty() {
        return None;
    }
    let format = value.get("format");

    // Album art on an MP3 is a video stream by codec type and is not what anyone means by
    // "the picture", so a stream disposed as an attached picture is not the video stream.
    let video = streams.iter().find(|stream| {
        stream.get("codec_type").and_then(Value::as_str) == Some("video")
            && stream
                .pointer("/disposition/attached_pic")
                .and_then(Value::as_i64)
                != Some(1)
    });
    let audio = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(Value::as_str) == Some("audio"));

    // The `start_time` an author cares about is the picture's where there is one, and the
    // sound's otherwise. The container's is the fallback of last resort.
    let start_source = video.or(audio);
    let quad = Quad {
        video_stream_ms: video.and_then(|stream| ms_at(stream, "duration")),
        container_ms: format.and_then(|format| ms_at(format, "duration")),
        start_time_ms: start_source
            .and_then(|stream| ms_at(stream, "start_time"))
            .or_else(|| format.and_then(|format| ms_at(format, "start_time"))),
        r_frame_rate: video.and_then(|stream| rational_at(stream, "r_frame_rate")),
        avg_frame_rate: video.and_then(|stream| rational_at(stream, "avg_frame_rate")),
    };

    let dimensions = video.and_then(source_dimensions);
    let alpha = video.map(|stream| {
        stream
            .get("pix_fmt")
            .and_then(Value::as_str)
            .is_some_and(pix_fmt_has_alpha)
    });
    let audio = audio.map(|stream| Audio {
        sample_rate: stream
            .get("sample_rate")
            .and_then(Value::as_str)
            .and_then(|rate| rate.parse().ok()),
        channels: stream
            .get("channels")
            .and_then(Value::as_u64)
            .map(|channels| channels as u32),
        audio_stream_ms: ms_at(stream, "duration"),
    });

    Some(Probe {
        source: source.to_string(),
        quad,
        dimensions,
        alpha,
        audio,
    })
}

/// ADR-0023's pipeline, fed from one stream. Everything type-specific happens here — in
/// *reading* the three inputs — and nothing does in resolving them.
fn source_dimensions(stream: &Value) -> Option<SourceDimensions> {
    let width = stream.get("width").and_then(Value::as_u64)? as u32;
    let height = stream.get("height").and_then(Value::as_u64)? as u32;
    let par = stream
        .get("sample_aspect_ratio")
        .and_then(Value::as_str)
        .and_then(Rational::parse)
        // A file that states no PAR is square-pixel — the image case, unchanged.
        .unwrap_or(Rational::ONE);

    Some(dimensions::resolve(
        Decoded { width, height },
        container_rotation(stream),
        par,
    ))
}

/// The container's track-level display transform, and nothing else.
///
/// ADR-0023: *"apply the container's track-level display transform (e.g. an MP4 `tkhd`
/// matrix); codec-level orientation metadata (SEI / VUI display orientation) is not
/// consulted for this purpose."* `ffprobe` surfaces the matrix as Display Matrix side
/// data, and older builds surface the same `tkhd` value as a `rotate` tag; both are read,
/// and no `Display Orientation` side data is.
fn container_rotation(stream: &Value) -> Rotation {
    let from_matrix = stream
        .get("side_data_list")
        .and_then(Value::as_array)
        .and_then(|list| {
            list.iter()
                .find(|entry| {
                    entry.get("side_data_type").and_then(Value::as_str) == Some("Display Matrix")
                })
                .and_then(|entry| entry.get("rotation"))
                .and_then(Value::as_f64)
        })
        .map(|degrees| (degrees, RotationSource::DisplayMatrix));

    let from_tag = || {
        stream
            .pointer("/tags/rotate")
            .and_then(|tag| match tag {
                Value::String(text) => text.parse().ok(),
                other => other.as_f64(),
            })
            .map(|degrees| (degrees, RotationSource::RotateTag))
    };

    match from_matrix.or_else(from_tag) {
        // `ffprobe` reports the display matrix counter-clockwise-negative; the sign is
        // preserved into the normalisation rather than flipped here, so the reported
        // quarter turn is the one the matrix states. Which of the two signals was read
        // travels with it, because ADR-0023 requires the report to say which was applied.
        Some((degrees, source)) => Rotation::from_degrees(degrees, source),
        None => Rotation::NONE,
    }
}

/// A decimal-seconds field on a stream or on the format, as integer milliseconds.
fn ms_at(value: &Value, key: &str) -> Option<i64> {
    match value.get(key)? {
        Value::String(text) => milliseconds(text),
        // Some builds emit these as numbers. Print it back through the same exact parser
        // rather than multiplying a float by 1000.
        other => milliseconds(&other.as_f64()?.to_string()),
    }
}

fn rational_at(value: &Value, key: &str) -> Option<Rational> {
    Rational::parse(value.get(key)?.as_str()?)
}

/// Whether a pixel format carries an alpha channel.
///
/// A name test, because the alternative is a table of every format FFmpeg has ever
/// defined and a silent wrong answer the first time it gains another. `ya`/`yuva`/`gbra`
/// and the packed RGB spellings are the families that carry alpha; `pal8` can and the
/// palette decides, which is why it is not claimed either way here.
fn pix_fmt_has_alpha(pix_fmt: &str) -> bool {
    const PACKED_WITH_ALPHA: &[&str] = &["rgba", "bgra", "argb", "abgr", "rgba64", "bgra64"];

    let name = pix_fmt.to_ascii_lowercase();
    PACKED_WITH_ALPHA
        .iter()
        .any(|packed| name.starts_with(packed))
        || name.starts_with("yuva")
        // `ya8`/`ya16` are grey-plus-alpha; `gray` on its own is not.
        || name.starts_with("ya")
        || name.starts_with("gbrap")
}

/// The finding an outcome carries, where it carries one.
///
/// One function, in the layer both callers share: `validate`'s disk check and the `probe`
/// verb report the same four outcomes, and a boundary drawn twice is a boundary that can be
/// drawn differently.
pub fn finding_for(outcome: &Outcome, declared: &str) -> Option<Finding> {
    match outcome {
        Outcome::Probed(_) => None,
        Outcome::Missing { source, detail } => Some(
            Finding::new("E-SOURCE-MISSING")
                // The spelling the document used, so an agent can find the string to fix;
                // the resolved path is what was actually looked for, and both matter.
                .field("source", json!(declared))
                .field("resolved", json!(source))
                .field("detail", json!(detail))
                .repair_value(json!({
                    "value": "correct the source, or put the file where it says"
                })),
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

/// Record what one probe established into a report: a finding where something is wrong, the
/// facts where something is right.
///
/// `declared` is the spelling the document used, which is the string an agent has to edit;
/// every finding carries it, and the resolved path travels beside it rather than in its
/// place.
pub fn record(outcome: &Outcome, declared: &str, report: &mut crate::report::Report) {
    match finding_for(outcome, declared) {
        Some(finding) => report.push(finding),
        None => {
            if let Some(probe) = outcome.probe() {
                // One entry per distinct source: the same file referenced by four elements
                // is one fact about the disk, not four.
                if !report.media.iter().any(|seen| seen.source == probe.source) {
                    report.media.push(probe.clone());
                }
            }
        }
    }
}

/// The identity a local probe is cached under. ADR-0006: *"cache on `(path, size, mtime)`
/// → duration"*, where every component is a direct filesystem observation the tool makes
/// itself — which is exactly what ADR-0056 says an HTTP validator is not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalKey {
    /// The file's canonical path. Canonical rather than as-written because the cache now
    /// outlives the process (ADR-0069): `./take3.mov` and `/clips/take3.mov` are one file,
    /// and a sidecar keyed on the spelling would remember them as two and notice a change
    /// in neither. What the *report* names is untouched — that is the caller's spelling.
    pub path: PathBuf,
    pub size: u64,
    /// Modification time as nanoseconds since the Unix epoch, or `None` on a filesystem
    /// that will not say.
    ///
    /// `None` equals `None`, so such a key matches on `(path, size)` alone and cannot
    /// notice a file rewritten to the same length. Within one process that is #190's
    /// accepted blind spot. It is **not** allowed to outlive one: the sidecar refuses to
    /// persist an entry keyed this way (ADR-0069), because a key Montagent only partly
    /// observed would otherwise make that blind spot permanent — and it is exactly
    /// `MissKind::Changed`, ADR-0011's sole mechanism, that would go silent.
    pub mtime_ns: Option<i128>,
}

impl LocalKey {
    pub fn of(path: &Path) -> std::io::Result<LocalKey> {
        let metadata = std::fs::metadata(path)?;
        Ok(LocalKey {
            // A path that will not canonicalise still has a `(size, mtime)` worth caching
            // under; it just caches under the spelling it arrived in.
            path: std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()),
            size: metadata.len(),
            mtime_ns: metadata.modified().ok().and_then(|time| {
                time.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_nanos() as i128)
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_confirmed_absence_and_a_network_unknown_are_different_outcomes() {
        let missing = failure(
            "https://cdn.example/take3.mov",
            "[https @ 0x1] HTTP error 404 Not Found\nServer returned 404 Not Found",
            true,
        );
        assert!(matches!(missing, Outcome::Missing { .. }));

        let unknown = failure(
            "https://cdn.example/take3.mov",
            "[tcp @ 0x1] Failed to resolve hostname cdn.example: Name or service not known",
            true,
        );
        assert!(matches!(
            unknown,
            Outcome::Unchecked {
                reason: Some(UncheckedReason::Dns),
                ..
            }
        ));
    }

    #[test]
    fn a_status_that_is_not_a_refusal_keeps_the_code_on_the_reason() {
        let outcome = failure(
            "https://x/y.mov",
            "Server returned 503 Service Unavailable",
            true,
        );
        assert!(matches!(
            outcome,
            Outcome::Unchecked {
                reason: Some(UncheckedReason::Http { status: 503 }),
                ..
            }
        ));

        // The class-only spelling, which is not a number.
        assert_eq!(
            http_status("Server returned 5XX Server Error reply"),
            Some(500)
        );
        assert_eq!(http_status("Server returned 404 Not Found"), Some(404));
        assert_eq!(http_status("Connection refused"), None);
    }

    #[test]
    fn an_unrecognised_transport_failure_claims_the_least() {
        assert_eq!(
            network_reason("[tcp @ 0x1] Connection to tcp://x:443 failed: Connection refused"),
            UncheckedReason::Unreachable
        );
        assert_eq!(
            network_reason("Connection to tcp://x:443 failed: Operation timed out"),
            UncheckedReason::Timeout
        );
    }

    #[test]
    fn a_local_failure_is_unchecked_with_no_network_reason() {
        // ADR-0056's distinction runs the other way too: a reason that is present means
        // the network was the thing that failed.
        let outcome = failure(
            "/p/06.png",
            "Invalid data found when processing input",
            false,
        );
        match outcome {
            Outcome::Unchecked { reason, .. } => assert_eq!(reason, None),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn alpha_is_read_off_the_pixel_format() {
        assert!(pix_fmt_has_alpha("rgba"));
        assert!(pix_fmt_has_alpha("yuva420p"));
        assert!(pix_fmt_has_alpha("ya8"));
        assert!(pix_fmt_has_alpha("gbrapf32le"));
        assert!(!pix_fmt_has_alpha("rgb24"));
        assert!(!pix_fmt_has_alpha("yuv420p"));
        assert!(!pix_fmt_has_alpha("gray"));
    }
}
