//! `render` — *"files in, video out"* (ADR-0011), and the enforcement point for every
//! check in the product.
//!
//! ## The rule this verb exists to make structural
//!
//! ADR-0006 gives `validate` no defence of its own — *"nothing, while it is a discrete
//! thing you invoke"* — and adopts two mechanisms instead, of which this is the first:
//! **`render` runs the identical check engine and refuses on any `error`**, so the check
//! cannot be skipped by not running it. *"Identical"* is not a claim this module makes by
//! copying `validate`'s call list carefully; it is [`crate::verbs::validate::checked`],
//! which is the one function both verbs call — `validate` throws the document away
//! afterwards and `render` keeps it to paint from. `tests/render.rs` asserts the two
//! reports carry the same findings on the same file.
//!
//! Three consequences of that rule, each ADR-stated:
//!
//! - **A `LAYOUT` finding never gates a render** (ADR-0041). It is a category, not a
//!   severity, and a file written outside the convention is *"unsafe to edit, not unsafe
//!   to render"*. The refusal is on [`crate::report::Report::exit_code`] — which counts
//!   `error` alone — rather than on a re-derived notion of "anything wrong".
//! - **The `review` findings the render did not refuse on print after it succeeds, and so
//!   does the `NOT CHECKED` footer** (ADR-0011: *"the failure mode that matters is not a
//!   crash — it is success"*). The report the answer carries *is* the check engine's
//!   report, rendered through the same [`crate::text`] every other verb uses, with the
//!   render's own block above it — so exit 0 never reads as *"the video is right"*.
//! - **Proxy degradation is never applied** (ADR-0021, ADR-0067: *"nothing about `render`
//!   changes; proxy degradation has never applied to it"*). [`render`] builds
//!   [`Surface::declared`] and there is no other surface it can be given: the scale is a
//!   parameter of [`encode_span`] — which `preview` shares, so that a proxy frame is this
//!   verb's picture on a smaller device rather than a second renderer — and this verb
//!   states it once, at one call site, as the declared frame at true pixels. It passes no
//!   [`Span::deadline`] either: a deliverable is never abandoned part-written on a clock.
//!
//! ## The deliverable
//!
//! Written via a temp path and one atomic rename (ADR-0011, story 55), through
//! [`montagent_render::encode`], so a truncated MP4 never reads as finished. A full render
//! writes the project's declared `output`; `--from`/`--to` derives
//! `out/<name>.<from>-<to>.mp4` and refuses an explicit `--output` equal to the project's
//! while a range is set (story 56) — *"a partial render must never be able to land on the
//! deliverable"*. The range is half-open like every other range in the format (ADR-0005),
//! and the answer says so.
//!
//! **The machine-readable result goes to stdout and coarse progress to stderr** (story
//! 59). The verb prints nothing — no verb does — so the split is made in the CLI: the
//! [`Answer`] is what stdout gets, and the [`Progress`] callback is what stderr gets.
//!
//! ## What is painted, and by whom
//!
//! Every frame goes through [`crate::verbs::frame::Painter`] — the same pass, the same
//! order, the same rules — over the presence set [`crate::verbs::query::at::presence`]
//! returns for the frame's instant. The picture `frame` shows an agent is the picture
//! `render` produces because there is one painter, and ADR-0021's whole reason for keeping
//! `frame` at true pixels is that this stays true.
//!
//! **Frame *n* is painted at `⌊n × 1000 / fps⌋` ms.** ADR-0035's grid samples frame *n*
//! at `n × 1000/fps` exactly, which at 30 fps is not a whole millisecond. The painter takes
//! whole milliseconds — every resolution in [`crate::verbs::query::geometry`] does — so the
//! instant is floored. Which elements are present is unaffected, since every `start` and
//! `end` is an integer: `⌊t⌋ ≥ start ⇔ t ≥ start` and `⌊t⌋ < end ⇔ t < end`. What can
//! differ is an interpolated value, by less than one millisecond of travel, at rates where
//! the grid is not millisecond-exact. At 25, 50, 100, 200, 500 and 1000 fps it is exact.
//! Ratified by ADR-0077 (#287), which amends ADR-0035 with it; the unit test below is the
//! re-executable check behind both halves of that sentence.
//!
//! ## Audio (spec #168, stories 93–99)
//!
//! Every `audio` and `video` element's audio is mixed into the output through an `ffmpeg`
//! filter graph this module writes: each element's source range is cut (`atrim`), played
//! at its `speed` through `atempo` (story 97 — chained, since one `atempo` covers 0.5×–2×),
//! looped past its as-played duration where `overrun: "loop"` says so (ADR-0020, on an
//! exact sample count at the mix bus's own 48 kHz), scaled by its `volume` (ADR-0055 — a
//! scalar directly, a keyframe list as the resolved value on every sampled frame of the
//! element, sent to the `volume` filter as timed commands), placed at its `start`
//! (`adelay`), and summed without normalisation (`amix`) so two narration lines at `1.0`
//! are each still at `1.0`. Clipping past the sum is the renderer's documented behaviour
//! rather than a ceiling (ADR-0055). An element that cannot be mixed is listed in the
//! answer with the reason, on the same rule the picture lists what it did not paint.
//!
//! ## The readings ADR-0077 ratifies (#287)
//!
//! Each of these was decided here to ship #215, argued at its site, and had no ADR behind
//! it. ADR-0077 is where they became spec — it amends ADR-0035, ADR-0011, ADR-0021,
//! ADR-0009 and ADR-0055 rather than restating them, and it changed no behaviour.
//!
//! - The floored frame instant above (ADR-0035).
//! - **A project with no `duration` renders to its last boundary**, which is the same
//!   derived `duration` [`crate::slack`] uses — one derivation, not two (ADR-0011).
//! - **A project with no `output` and no `--output` is refused with exit 3**, naming the
//!   field: there is nowhere to put the video, and the command is what says where
//!   (ADR-0011).
//! - **`--to` past the project's end is legal** — the frames past it are background — for
//!   ADR-0011's reason that *"every instant is a legal question"*. `--from` before 0 is
//!   not: the clock begins at 0.
//! - **The derived partial name's `<name>` is the declared `output`'s stem**, or the
//!   project file's own where it declares none (ADR-0011 fixes the shape, not `<name>`).
//! - An odd frame dimension is padded to even and disclosed (ADR-0021), argued at
//!   [`montagent_render::encode`]; so are the encoder settings (ADR-0009).
//! - **The mix bus is 48 kHz stereo and `amix` runs with `normalize=0`**, so every `aloop`
//!   sample count is exact from the document alone, no element is mixed at a rate a probe
//!   had to supply, and two narration lines at `1.0` are each still at `1.0` (ADR-0055).
//! - **A keyframed `volume` is the value `resolve` computes on every sampled frame**, sent
//!   as timed commands rather than re-expressed in `ffmpeg`'s expression language
//!   (ADR-0055).
//!
//! **Not a reading, and deliberately unratified** — ADR-0077 records it as a cost:
//!
//! - **A `video` element is decoded through one `ffmpeg` seek per frame**, the same call
//!   `frame` makes. It is correct and it is slow — a spawn per frame — and a streaming
//!   decode is an optimisation this ticket does not take. The only render wall clock this
//!   project has measured is over the committed fixture, which has no `video` element
//!   (`montagent_render::budget::RENDER_REFERENCES`), so what a spawn per frame costs a
//!   project that does is unmeasured and the saving would be invented.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path as FilePath, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{Value, json};

use montagent_render::canvas::{Canvas, Rgba};
use montagent_render::encode::{self, Encoder, Spec};

use crate::exact::{self, Decimal};
use crate::finding::{Class, Finding};
use crate::media::established::Established;
use crate::media::sidecar::Sidecar;
use crate::media::tools::Missing;
use crate::media::{Source, attest, display_local, probe, tools};
use crate::model::{Animatable, Keyframe, Volume};
use crate::permissive::Loose;
use crate::report::{ExitCode, Report};
use crate::resolve;
use crate::verbs::frame::{Declined, NotPainted, Painter};
use crate::verbs::query::at;

const TOOL: &str = "render";

/// The mix bus's sample rate. Every input is resampled to it first, which is what makes a
/// loop's sample count computable from the document alone.
const MIX_RATE: i64 = 48_000;

/// What one `render` invocation is asking.
#[derive(Debug, Clone, Default)]
pub struct Ask {
    /// The start of a partial render, in absolute milliseconds. Asked for with `to`.
    pub from: Option<i64>,
    /// The end of a partial render, exclusive: the range is half-open `[from, to)`.
    pub to: Option<i64>,
    /// Where to write the video instead of the project's `output`, as the caller spelled
    /// it. Refused when a range is set and it names the project's own `output`.
    pub output: Option<PathBuf>,
    /// ADR-0104: escalate `R-OUTPUT-UNATTESTED` from `review` to a refusal.
    ///
    /// It only ever *tightens*. There is deliberately no flag in the other direction:
    /// ADR-0093 condition 2 forbids one that would let an unwaived `error` reach the output
    /// path, and `E-OUTPUT-FOREIGN` is such an error. A caller who genuinely means to
    /// replace another project's deliverable changes `output`, or moves the file.
    pub no_clobber: bool,
}

/// One coarse progress step, handed to the caller as frames are encoded.
///
/// Coarse on purpose — ADR-0011: *"a spinner is worth nothing to an agent"*. It is
/// reported when the run starts, each time another tenth of the frames is done, and when
/// the last frame is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Frames encoded so far.
    pub done: u64,
    /// Frames in the whole run.
    pub of: u64,
    pub elapsed: Duration,
}

/// A caller's request that a running encode stop, and publish nothing (ADR-0109).
///
/// A flag rather than a channel or a token type from an async runtime, because the core is
/// synchronous and the one question it asks is *"has anyone asked me to stop?"* — once per
/// frame, and once more immediately before the rename that would publish. The MCP adapter
/// sets it when the client sends `notifications/cancelled`; the CLI never does, because a
/// CLI user who wants a render stopped sends a signal and the process goes with it.
#[derive(Debug, Clone, Default)]
pub struct Cancel(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Cancel {
    pub fn new() -> Cancel {
        Cancel::default()
    }

    /// Ask the encode to stop. Idempotent, and callable from any thread.
    pub fn cancel(&self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// `E-CANCELLED`'s sentence for an encode stopped part-way: how far it got, and what that
/// means for the disk, which is the one thing ADR-0109 decides.
pub fn cancelled_after(tool: &str, done: u64, of: u64) -> String {
    format!(
        "`{tool}` was cancelled by the caller after {done} of {of} frames, and published \
         nothing: whatever was at the output path before this call is still there, untouched"
    )
}

/// One `render` invocation's answer: the video, where one was written, and the report
/// every verb answers with — here, the check engine's own.
pub struct Answer {
    video: Option<Video>,
    report: Report,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// The video, where the run produced one.
    pub fn video(&self) -> Option<&Video> {
        self.video.as_ref()
    }

    /// The canonical JSON: the report's own object plus the `render` block — `null` where
    /// the render was refused or failed, so a consumer reads the absence off a key that is
    /// always there.
    pub fn to_json(&self) -> Value {
        self.report.to_json_with(
            "render",
            match &self.video {
                Some(video) => serde_json::to_value(video).unwrap_or(Value::Null),
                None => Value::Null,
            },
        )
    }
}

/// The machine-readable result (ADR-0011): path, duration, frame count, wall time,
/// realtime factor — and everything about the run an agent needs in order to trust it.
#[derive(Debug, Clone, Serialize)]
pub struct Video {
    /// Where the file is, as the run resolved it.
    pub path: String,
    /// The range rendered, half-open: `to` is not in it.
    pub from: i64,
    pub to: i64,
    /// Whether this was a `--from`/`--to` render rather than the whole project.
    pub partial: bool,
    /// `to - from`.
    pub duration_ms: i64,
    pub frames: u64,
    pub fps: i64,
    /// The frame that was rasterized. For `render` this is always the project's declared
    /// frame at true pixels (ADR-0021); for `preview` it is the tier's, which that verb's
    /// block discloses beside this one.
    pub width: i64,
    pub height: i64,
    /// The frame the file actually carries, where padding to even made it differ.
    pub encoded: Option<Encoded>,
    pub bytes: u64,
    pub wall_ms: u64,
    /// `duration_ms / wall_ms` — how many seconds of video each second of wall clock
    /// produced, so the caller can budget the next call.
    pub realtime: f64,
    /// Every audible element mixed into the output, in document order.
    pub mixed: Vec<String>,
    /// Every audible element that is *not* in the output, and why.
    pub not_mixed: Vec<NotPainted>,
    /// Every element painted on at least one frame, in the order first painted.
    pub painted: Vec<String>,
    /// Every element present on some frame that no frame painted, with the reason — the
    /// union over the run of what `frame` reports per instant.
    pub not_painted: Vec<NotPainted>,
    /// Every element painted without something it asked for, likewise.
    pub painted_partially: Vec<NotPainted>,
    /// Every media file opened, in the order first opened.
    pub sources: Vec<String>,
    /// Every font file opened (ADR-0007).
    pub fonts: Vec<String>,
}

/// The padded frame, where there was one.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Encoded {
    pub width: i64,
    pub height: i64,
}

/// Render the project at `path`.
///
/// `progress` is called as frames are encoded; the CLI prints it to stderr. The verb
/// itself prints nothing.
pub fn render(path: &FilePath, ask: &Ask, progress: &mut dyn FnMut(Progress)) -> Answer {
    render_cancellable(path, ask, progress, None)
}

/// [`render`], stopping where it stands and publishing nothing once `cancel` is set
/// (ADR-0109). A cancel that arrives after the publish is too late to matter: the render had
/// already succeeded, and the file at `output` is exactly the one it made.
pub fn render_cancellable(
    path: &FilePath,
    ask: &Ask,
    progress: &mut dyn FnMut(Progress),
    cancel: Option<&Cancel>,
) -> Answer {
    let started = Instant::now();
    let project = Some(path.display().to_string());

    // The invocation is settled before the file is opened, as `frame` does: `--from 5
    // --to 2` is wrong whatever the document says, and ADR-0011 keeps exit 3 apart from
    // exit 1 so that "fix the command" is never read as "fix the project".
    let range = match request(ask) {
        Ok(range) => range,
        Err(reason) => {
            return refused(Report::rejected(TOOL, project, reason));
        }
    };

    // The identical check engine, and the refusal (ADR-0006). `checked` is `validate`.
    let (document, mut report) =
        match crate::verbs::validate::checked(TOOL, path, None, Sidecar::default_path()) {
            Ok(checked) => checked,
            Err(report) => return refused(*report),
        };
    if report.exit_code() != ExitCode::Ok {
        return refused(report);
    }

    // A document every check passed parses strictly; one that does not is a defect in
    // Montagent's own schema check, and says so with exit 70 rather than guessing.
    let header = match document.strict() {
        Ok(project) => project,
        Err(e) => {
            report.fail_internally(format!(
                "the project passed every check and still does not fit the model: {e}"
            ));
            return refused(report);
        }
    };
    let (width, height) = (header.frame.width, header.frame.height);
    let fps = header.fps;

    let (from, to, partial) = match range {
        Some((from, to)) => (from, to, true),
        None => match extent(&document) {
            Some(end) => (0, end, false),
            None => {
                return refused(Report::rejected(
                    TOOL,
                    Some(document.path().to_string()),
                    "there is nothing to render: the project declares no `duration` and \
                     no element states a range",
                ));
            }
        },
    };

    // The frames in `[from, to)` on ADR-0035's grid.
    let (Some(first), Some(last)) = (
        exact::frame_at_or_after(from, fps),
        exact::frame_before(to, fps),
    ) else {
        return refused(Report::rejected(
            TOOL,
            Some(document.path().to_string()),
            format!("no frame at {fps} fps falls inside {from}..{to} ms"),
        ));
    };
    if last.frame < first.frame {
        return refused(Report::rejected(
            TOOL,
            Some(document.path().to_string()),
            format!("no frame at {fps} fps falls inside {from}..{to} ms"),
        ));
    }
    let frames = (last.frame - first.frame + 1) as u64;

    let project_dir = crate::checks::project_dir(&document);
    let output = match destination(
        &document,
        &project_dir,
        header.output.as_deref(),
        ask,
        range,
    ) {
        Ok(output) => output,
        Err(reason) => {
            return refused(Report::rejected(
                TOOL,
                Some(document.path().to_string()),
                reason,
            ));
        }
    };

    // `ffmpeg`, resolved once for the encoder and for every video element's decode
    // (ADR-0011's exit 70 when there is none).
    let resolved = match tools::resolve() {
        Ok(tools) => tools,
        Err(missing) => {
            missing.fail(&mut report);
            return refused(report);
        }
    };
    let ffmpeg = resolved.ffmpeg.clone();

    // ADR-0104. What is already at the output path, before a single frame is painted.
    //
    // Sited here and not after the encode for ADR-0093 condition 1's reason: everything
    // pre-flightable is pre-flighted, so the render that must not happen costs no wall
    // clock. The injury this prevents is measured in ninety-minute encodes, and discovering
    // it at promotion time would prevent the destruction while still spending the clock.
    let stamp = attest::stamp(FilePath::new(document.path()));
    match attest::of(&probe::ProcessRunner, &resolved, &output, &stamp) {
        // Nothing there, or this project's own last answer. Re-rendering over yourself is
        // the normal loop — four attempts at one cut is what the source incident describes —
        // and is never a finding.
        attest::Attestation::Vacant | attest::Attestation::Mine => {}
        attest::Attestation::Foreign { project } => {
            report.push(
                Finding::new("E-OUTPUT-FOREIGN")
                    .at_file(document.path().to_string())
                    .field("output", Value::String(output.display().to_string()))
                    // No `repair` value: ADR-0043 fixes this code at refuse, and the repair
                    // is genuinely not derivable — which of the two projects is meant to own
                    // this path is in neither document. The template says the two moves
                    // available; it does not pick one.
                    .field("project", Value::String(project)),
            );
            return refused(report);
        }
        attest::Attestation::Absent => {
            // `review` by default (the class the jury split on) and a refusal only when the
            // caller asked for one. The tool has no evidence about this file; `--no-clobber`
            // is how a batch script says "treat no evidence as reason enough".
            if ask.no_clobber {
                report.push(
                    Finding::at_class("R-OUTPUT-UNATTESTED", Class::Error)
                        .at_file(document.path().to_string())
                        .field("output", Value::String(output.display().to_string())),
                );
                return refused(report);
            }
            report.push(
                Finding::new("R-OUTPUT-UNATTESTED")
                    .at_file(document.path().to_string())
                    .field("output", Value::String(output.display().to_string())),
            );
        }
    }

    let background = header
        .background
        .as_ref()
        .and_then(crate::verbs::frame::rgba_of)
        .unwrap_or(Rgba::BLACK);

    // The declared frame, and nothing else: `render` has no surface of its own to choose
    // (ADR-0021). The one call site that passes anything smaller is `preview`.
    let span = Span {
        document: &document,
        established: Established::of(&report),
        project_dir: &project_dir,
        ffmpeg: &ffmpeg,
        background,
        declared: (width, height),
        surface: Surface::declared(width, height),
        fps,
        from,
        to,
        first: first.frame,
        last: last.frame,
        frames,
        output: &output,
        stamp: Some(stamp),
        deadline: None,
        cancel,
    };
    let painted = match encode_span(&span, started, progress) {
        Ok(painted) => painted,
        Err(Stop::Internal(reason)) => {
            report.fail_internally(reason);
            return refused(report);
        }
        // ADR-0093 ruling 6, condition 1: the mix is decided before the encoder is spawned,
        // so this refusal costs no wall clock and there is no temp file to withhold. Nothing
        // has been written at any path.
        Err(Stop::Refused(findings)) => {
            for finding in findings {
                report.push(finding);
            }
            return refused(report);
        }
        // ADR-0091: an unconfigured environment, not a broken project.
        Err(Stop::ToolMissing(missing)) => {
            missing.fail(&mut report);
            return refused(report);
        }
        // ADR-0109: stopped where it stood; nothing was written at any path.
        Err(Stop::Cancelled { done }) => {
            report.fail_cancelled(cancelled_after(TOOL, done, frames));
            return refused(report);
        }
        // `render` passes no deadline, so there is no clock for a span of its to run past.
        Err(Stop::Missed { .. }) => {
            report.fail_internally(
                "`render` was stopped on a wall clock, and it has none: the deliverable is \
                 never degraded and never abandoned on time (ADR-0021)"
                    .to_string(),
            );
            return refused(report);
        }
    };
    let wall_ms = started.elapsed().as_millis() as u64;

    // ADR-0093 ruling 6. Every world-effect the frame loop discovered goes onto the report
    // first, and *then* the report decides whether the file may exist.
    //
    // The invariant this buys is the one MONTAGENT-1 broke: **a file at the output path is a
    // render with zero errors.** What shipped the mute cut was not an unread report — it was
    // a plausible file existing, which is what a human uploads and what an MCP agent `stat`s.
    // A counted finding is necessary and not sufficient.
    //
    // Named plainly because it is a behaviour change beyond the reporting fix: `render` can
    // now spend wall clock and produce nothing where it used to produce a file.
    for finding in painted.declined().to_vec() {
        report.push(finding);
    }
    if report.exit_code() != ExitCode::Ok {
        painted.withhold();
        return refused(report);
    }
    // ADR-0109: the last moment a cancel can change what is on disk. Past this line the
    // rename happens, and a cancel that arrives after it finds a render that succeeded.
    if span.cancelled() {
        painted.withhold();
        report.fail_cancelled(cancelled_after(TOOL, frames, frames));
        return refused(report);
    }

    match painted.into_video(&span, partial, wall_ms) {
        Ok(video) => Answer {
            video: Some(video),
            report,
        },
        // The encode was whole and the rename was not. Nothing is at the declared path.
        Err(reason) => {
            report.fail_internally(reason);
            refused(report)
        }
    }
}

/// One span of frames to paint and encode, once the invocation and the document are
/// settled — everything [`encode_span`] needs and nothing about which verb is asking.
///
/// It exists so that `preview` is `render`'s picture at a smaller surface rather than a
/// second renderer that has to be kept in agreement with the first. Two fields carry the
/// whole of the difference between the verbs, and both are stated at the call site rather
/// than derived in here: [`Span::surface`] and [`Span::deadline`].
pub(crate) struct Span<'a> {
    pub document: &'a Loose,
    /// What the check engine established about each media file (ADR-0093) — the one
    /// structure both verbs read, taken by value so the span holds no borrow of the report
    /// it came from. The caller has to be able to push this span's own findings onto that
    /// report while still holding the span.
    pub established: Established,
    pub project_dir: &'a FilePath,
    pub ffmpeg: &'a FilePath,
    pub background: Rgba,
    /// The project's declared frame. What is painted is always in these coordinates.
    pub declared: (i64, i64),
    /// The device the painting lands on.
    pub surface: Surface,
    pub fps: i64,
    pub from: i64,
    pub to: i64,
    /// The first and last frame numbers on ADR-0035's grid, inclusive.
    pub first: i64,
    pub last: i64,
    pub frames: u64,
    pub output: &'a FilePath,
    /// ADR-0104's attestation, stamped into the container the encoder writes.
    ///
    /// `Some` for `render`, whose output is a deliverable a later run must be able to
    /// recognise as its own. `None` for `preview`: a proxy is not a deliverable, and a
    /// preview that stamped itself would hand the next `render` a forged licence to clobber.
    pub stamp: Option<String>,
    /// The wall clock this span must land inside, checked between frames.
    ///
    /// `None` for `render`, which has no ceiling to encode (ADR-0072) and must never
    /// abandon a deliverable part-written on time. `Some` for `preview`, where the budget
    /// is the whole mechanism: a span that runs past it is stopped where it stands so the
    /// next rung down starts with the budget in hand, rather than the ladder costing the
    /// sum of every attempt run to completion.
    pub deadline: Option<Duration>,
    /// ADR-0109: the caller's request to stop, checked before the encoder starts, before
    /// every frame, and after the seal. `None` where nobody can ask.
    pub cancel: Option<&'a Cancel>,
}

impl Span<'_> {
    fn cancelled(&self) -> bool {
        self.cancel.is_some_and(Cancel::is_cancelled)
    }
}

/// The device a span lands on: its pixel dimensions, and the scale from the project's
/// coordinates to them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Surface {
    pub width: i64,
    pub height: i64,
    pub scale: (f64, f64),
}

impl Surface {
    /// The project's own frame, at true pixels and no scale — `render`'s only surface.
    pub fn declared(width: i64, height: i64) -> Surface {
        Surface {
            width,
            height,
            scale: (1.0, 1.0),
        }
    }

    fn canvas(self) -> Option<Canvas> {
        if self.scale == (1.0, 1.0) {
            Canvas::new(self.width, self.height)
        } else {
            Canvas::scaled(self.width, self.height, self.scale)
        }
    }
}

/// Why a span stopped before it had a file.
pub(crate) enum Stop {
    /// Montagent's own failure, as the sentence the report fails internally with.
    Internal(String),
    /// ADR-0093 ruling 6, condition 1: *"everything pre-flightable is pre-flighted."* The
    /// mix is a pure function of the document, the report and the range, so every reason an
    /// audible element is not in the output is known before the encoder is spawned. Where
    /// one of them is an `error`, the span stops here — no subprocess, no temp file, no wall
    /// clock — and the findings are the caller's to report.
    ///
    /// The set is closed at what a span can decide *before* it starts. A reason discovered
    /// mid-loop cannot come back through here, and does not: it travels on
    /// [`Painted::declined`] and is caught by the non-promotion rule instead.
    Refused(Vec<Finding>),
    /// ADR-0091: `ffmpeg` is not on `PATH` (or, ADR-0115, cannot do what the floor asks), so
    /// exit 70 and `Missing::fail`'s code rather than a finding about the project.
    ToolMissing(Box<Missing>),
    /// The span ran past [`Span::deadline`] and was abandoned where it stood. The temp
    /// file goes with the dropped encoder; nothing was written.
    Missed {
        elapsed: Duration,
        /// How far it got, so a refusal can say how much of the span the clock bought.
        done: u64,
    },
    /// ADR-0109: the caller asked the span to stop. Walked away exactly as [`Stop::Missed`]
    /// is — the temp file goes with the encoder, and nothing was written at any path.
    Cancelled {
        /// Frames encoded before the request was seen.
        done: u64,
    },
}

/// What one span produced: the file, and the record of what of the document reached it.
pub(crate) struct Painted {
    /// The whole span, encoded and **not published**. ADR-0093 ruling 6: whether this
    /// becomes the deliverable is decided by the caller that holds the report, because the
    /// invariant being bought is *a file at the output path is a render with zero errors*.
    sealed: encode::Sealed,
    mixed: Vec<String>,
    not_mixed: Vec<NotPainted>,
    painted: Vec<String>,
    not_painted: Vec<NotPainted>,
    painted_partially: Vec<NotPainted>,
    /// ADR-0093: every world-effect the span discovered, as findings the caller pushes onto
    /// its report. The mid-loop half of the set — the pre-flightable half never gets this
    /// far, having stopped the span at [`Stop::Refused`].
    declined: Vec<Finding>,
    sources: Vec<String>,
    fonts: Vec<String>,
}

impl Painted {
    /// The machine-readable block, which is the same shape for both verbs: `preview`
    /// adds its tier beside this rather than a second spelling of it.
    /// Publish the sealed file and describe it.
    ///
    /// ADR-0093 ruling 6 is the reason this is one step and not two: a caller that has
    /// decided to publish gets the numbers *about the published file*, and a caller that has
    /// decided not to never reaches a `Video` at all — see [`Painted::withhold`].
    pub(crate) fn into_video(
        self,
        span: &Span<'_>,
        partial: bool,
        wall_ms: u64,
    ) -> Result<Video, String> {
        let duration_ms = span.to - span.from;
        let finished = self.sealed.publish()?;
        Ok(Video {
            path: finished.path.display().to_string(),
            from: span.from,
            to: span.to,
            partial,
            duration_ms,
            frames: finished.frames,
            fps: span.fps,
            width: span.surface.width,
            height: span.surface.height,
            encoded: finished.encoded.map(|e| Encoded {
                width: i64::from(e.width),
                height: i64::from(e.height),
            }),
            bytes: finished.bytes,
            wall_ms,
            realtime: if wall_ms == 0 {
                f64::INFINITY
            } else {
                duration_ms as f64 / wall_ms as f64
            },
            mixed: self.mixed,
            not_mixed: self.not_mixed,
            painted: self.painted,
            not_painted: self.not_painted,
            painted_partially: self.painted_partially,
            sources: self.sources,
            fonts: self.fonts,
        })
    }

    /// Every world-effect the span discovered, for a caller deciding whether to publish.
    pub(crate) fn declined(&self) -> &[Finding] {
        &self.declined
    }

    /// ADR-0093 ruling 6: walk away. The temp file goes with the sealed encode and the
    /// declared path was never touched — *"declining to promote is the absence of a
    /// promotion, not destruction."*
    pub(crate) fn withhold(self) {
        self.sealed.withhold();
    }
}

/// Paint every frame of `span` through [`Painter`] and push it to the encoder.
///
/// **One painter, one pass, one order, for every verb that paints a span.** ADR-0021's
/// reason for keeping `frame` at true pixels is that the picture an agent checks is the
/// picture the deliverable carries; the same argument is why `preview` and `render` cannot
/// be two loops. What differs between them is the device and the clock, and both arrive as
/// fields on the span.
pub(crate) fn encode_span(
    span: &Span<'_>,
    started: Instant,
    progress: &mut dyn FnMut(Progress),
) -> Result<Painted, Stop> {
    let (width, height) = span.declared;
    let mix = Mix::of(
        span.document,
        span.project_dir,
        &span.established,
        span.fps,
        span.from,
        span.to,
    );

    // ADR-0093 ruling 6, condition 1. The mix is decided before the encoder is spawned, so
    // the span refuses here at no wall-clock cost and with no temp file to withhold. The
    // invariant violation goes first: it is not a fact about the project, so it must not be
    // reported as one alongside findings that are.
    if let Some(reason) = mix.internal {
        return Err(Stop::Internal(reason));
    }
    if mix.refuses() {
        return Err(Stop::Refused(mix.declined));
    }
    // Read out before the encoder starts, so the pre-flight's `mix` is not still alive
    // across the frame loop.
    let not_mixed = mix.not_mixed();
    let mixed = mix.mixed;
    let mix_declined = mix.declined;

    // ADR-0109: a call cancelled during pre-flight never spawns an encoder at all.
    if span.cancelled() {
        return Err(Stop::Cancelled { done: 0 });
    }

    let mut encoder = match Encoder::start(
        span.ffmpeg,
        span.output,
        &Spec {
            width: span.surface.width as u32,
            height: span.surface.height as u32,
            fps: span.fps,
            background: span.background,
            audio: mix.audio,
            stamp: span.stamp.clone(),
        },
    ) {
        Ok(encoder) => encoder,
        Err(reason) => {
            return Err(Stop::Internal(format!(
                "the encoder could not start: {reason}"
            )));
        }
    };

    let Some(mut canvas) = span.surface.canvas() else {
        return Err(Stop::Internal(format!(
            "no raster surface could be made at {}x{}",
            span.surface.width, span.surface.height
        )));
    };
    let mut painter = Painter::for_a_deliverable(span.document, span.from, (width, height));
    let mut painted: Vec<String> = Vec::new();
    let mut not_painted: BTreeSet<(String, String)> = BTreeSet::new();
    let mut painted_partially: BTreeSet<(String, String)> = BTreeSet::new();
    // ADR-0093: one finding per `(element, code)` over the whole span, not one per frame.
    // The same element declining for the same reason on 1631 consecutive frames is one fact
    // about the project, and the two lists above already dedupe on exactly this key — so
    // they and the findings cannot come out naming different sets.
    let mut declined: BTreeMap<(String, String), Finding> = BTreeMap::new();

    progress(Progress {
        done: 0,
        of: span.frames,
        elapsed: started.elapsed(),
    });
    let mut reported_tenth = 0;
    for (done, n) in (span.first..=span.last).enumerate() {
        // ADR-0109: checked before each frame, so a cancelled call frees the server within
        // one frame's time. The encoder is dropped on the way out, and its temp file with it.
        if span.cancelled() {
            return Err(Stop::Cancelled { done: done as u64 });
        }
        let instant = instant_of(n, span.fps);
        let view = at::presence(span.document, instant);
        painter.begin(instant);
        painter.paint(&mut canvas, &view);
        for name in &painter.painted {
            if !painted.contains(name) {
                painted.push(name.clone());
            }
        }
        for entry in &painter.not_painted {
            not_painted.insert((entry.element.clone(), entry.code.clone()));
        }
        for entry in &painter.painted_partially {
            painted_partially.insert((entry.element.clone(), entry.code.clone()));
        }
        for finding in &painter.declined {
            let key = (
                finding.location.element.clone().unwrap_or_default(),
                finding.code.clone(),
            );
            declined.entry(key).or_insert_with(|| finding.clone());
        }
        if let Some(reason) = painter.internal.take() {
            return Err(Stop::Internal(reason));
        }
        if let Some(missing) = painter.tool_missing.take() {
            return Err(Stop::ToolMissing(missing));
        }

        let Some(rgb) = canvas.rgb() else {
            return Err(Stop::Internal(format!(
                "frame {n} could not be read back off the canvas"
            )));
        };
        if let Err(reason) = encoder.push(&rgb) {
            // The encoder is dropped on the way out, and the temp file with it: the
            // declared path is untouched.
            return Err(Stop::Internal(format!("frame {n}: {reason}")));
        }

        let done = done as u64 + 1;
        let tenth = done * 10 / span.frames;
        if tenth > reported_tenth {
            reported_tenth = tenth;
            progress(Progress {
                done,
                of: span.frames,
                elapsed: started.elapsed(),
            });
        }
        // Checked after the frame rather than before it, so a span always encodes at least
        // one frame and a miss is a measurement rather than a refusal to start.
        if let Some(deadline) = span.deadline {
            let elapsed = started.elapsed();
            if elapsed > deadline && done < span.frames {
                return Err(Stop::Missed { elapsed, done });
            }
        }
    }

    // Sealed, not published: the whole span is in the temp file and whether it becomes the
    // deliverable is ADR-0093 ruling 6's question, which only the caller's report can answer.
    let sealed = match encoder.seal() {
        Ok(sealed) => sealed,
        Err(reason) => {
            return Err(Stop::Internal(format!(
                "the encoder did not finish: {reason}"
            )));
        }
    };
    // ADR-0109: a request that arrived during the seal still wins — the file is whole and
    // sitting in its temp path, and the caller has been told the call failed.
    if span.cancelled() {
        let done = sealed.frames();
        sealed.withhold();
        return Err(Stop::Cancelled { done });
    }
    // The whole span is in the file, so the clock only decides whether the file is worth
    // keeping — which is `preview`'s call, at the rung it is standing on, not this
    // function's.
    if let Some(deadline) = span.deadline {
        let elapsed = started.elapsed();
        if elapsed > deadline {
            // Unpublished, so there is no file at the declared path to remove — walking
            // away takes the temp file with it (ADR-0093 ruling 6).
            let done = sealed.frames();
            sealed.withhold();
            return Err(Stop::Missed { elapsed, done });
        }
    }

    let entries = |set: BTreeSet<(String, String)>| -> Vec<NotPainted> {
        set.into_iter()
            .map(|(element, code)| NotPainted { element, code })
            .collect()
    };
    Ok(Painted {
        sealed,
        mixed,
        not_mixed,
        painted,
        not_painted: entries(not_painted),
        painted_partially: entries(painted_partially),
        declined: mix_declined
            .into_iter()
            .chain(declined.into_values())
            .collect(),
        sources: painter.sources,
        fonts: painter.fonts,
    })
}

/// An answer with no video: the report says why.
fn refused(report: Report) -> Answer {
    Answer {
        video: None,
        report,
    }
}

/// The range asked for, or `None` for the whole project — or the one sentence saying why
/// the flags ask for no render.
pub(crate) fn request(ask: &Ask) -> Result<Option<(i64, i64)>, String> {
    match (ask.from, ask.to) {
        (None, None) => Ok(None),
        (Some(from), Some(to)) => {
            if from < 0 {
                return Err(format!(
                    "`--from {from}` is before the project starts; the clock begins at 0"
                ));
            }
            if to <= from {
                return Err(format!(
                    "`--from {from} --to {to}` is no range: `--to` is exclusive and must be \
                     after `--from`"
                ));
            }
            Ok(Some((from, to)))
        }
        (Some(_), None) => Err("`--from` needs a `--to`: a range is half-open `[from, to)`".into()),
        (None, Some(_)) => Err("`--to` needs a `--from`: a range is half-open `[from, to)`".into()),
    }
}

/// The instant the whole render ends at: the declared `duration`, or the last boundary
/// any element states — the same derived `duration` [`crate::slack`] uses.
pub(crate) fn extent(document: &Loose) -> Option<i64> {
    document
        .value()
        .get("duration")
        .and_then(Value::as_i64)
        .or_else(|| {
            document
                .elements()
                .filter_map(|element| element.get("end").and_then(Value::as_i64))
                .max()
        })
        .filter(|end| *end > 0)
}

/// Where the video goes (ADR-0011, stories 55 and 56).
fn destination(
    document: &Loose,
    project_dir: &FilePath,
    declared: Option<&str>,
    ask: &Ask,
    range: Option<(i64, i64)>,
) -> Result<PathBuf, String> {
    let declared = declared.map(|output| project_dir.join(output));
    match range {
        None => ask.output.clone().or(declared).ok_or_else(|| {
            "the project declares no `output` and no `--output` was given, so there is \
                 nowhere to write the video"
                .to_string()
        }),
        Some((from, to)) => match &ask.output {
            Some(explicit) => {
                if let Some(declared) = &declared
                    && same_path(explicit, declared)
                {
                    return Err(format!(
                        "`--output {}` is the project's own `output`, and a partial render \
                         (`--from {from} --to {to}`) may never land on the deliverable",
                        explicit.display()
                    ));
                }
                Ok(explicit.clone())
            }
            None => {
                let name = declared
                    .as_deref()
                    .and_then(FilePath::file_stem)
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_else(|| {
                        let file = FilePath::new(document.path())
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "project".to_string());
                        file.trim_end_matches(".json")
                            .trim_end_matches(".montagent")
                            .to_string()
                    });
                Ok(project_dir
                    .join("out")
                    .join(format!("{name}.{from}-{to}.mp4")))
            }
        },
    }
}

/// Do two spellings name one file?
///
/// Neither need exist yet — the first partial render happens before any deliverable does
/// — so the deepest ancestor that *does* exist is canonicalised (which is what resolves a
/// symlinked `out/` to the directory it really is) and the rest of the path is appended
/// and normalised lexically. A path with no existing ancestor at all is made absolute
/// against the working directory and normalised the same way.
pub(crate) fn same_path(a: &FilePath, b: &FilePath) -> bool {
    fn normal(path: &FilePath) -> PathBuf {
        let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        // Split at the deepest existing ancestor.
        let mut existing = absolute.as_path();
        let mut rest: Vec<std::ffi::OsString> = Vec::new();
        while !existing.exists() {
            // `components`, not `file_name`: a path ending in `..` has no file name, and
            // that is exactly the spelling this function exists to see through.
            let Some(last) = existing.components().next_back() else {
                break;
            };
            rest.push(last.as_os_str().to_owned());
            let Some(parent) = existing.parent() else {
                break;
            };
            existing = parent;
        }
        let mut out = std::fs::canonicalize(existing).unwrap_or_else(|_| existing.to_path_buf());
        for name in rest.into_iter().rev() {
            match name.to_str() {
                Some(".") => {}
                Some("..") => {
                    out.pop();
                }
                _ => out.push(name),
            }
        }
        out
    }
    normal(a) == normal(b)
}

/// The audio mix, as a filter graph, and the record of what is and is not in it.
struct Mix {
    audio: Option<encode::Audio>,
    mixed: Vec<String>,
    /// ADR-0093: every audible element not in the mix, as its finding — already located on
    /// the element that declared it, ready to be pushed onto the report.
    declined: Vec<Finding>,
    /// Montagent contradicting itself. The first one is enough: the render is over.
    internal: Option<String>,
}

impl Mix {
    /// The answer block's own list, derived from the findings rather than written beside
    /// them, so the two can never name different sets.
    fn not_mixed(&self) -> Vec<NotPainted> {
        self.declined
            .iter()
            .map(|finding| NotPainted {
                element: finding.location.element.clone().unwrap_or_default(),
                code: finding.code.clone(),
            })
            .collect()
    }

    /// Did anything the render cannot deliver over happen? ADR-0093 ruling 6's test, asked
    /// of the mix alone, before a frame is encoded.
    fn refuses(&self) -> bool {
        self.internal.is_some()
            || self
                .declined
                .iter()
                .any(|finding| finding.class == Class::Error)
    }
}

impl Mix {
    /// Read every audible element inside `[from, to)` and write its chain.
    fn of(
        document: &Loose,
        project_dir: &FilePath,
        established: &Established,
        fps: i64,
        from: i64,
        to: i64,
    ) -> Mix {
        let mut inputs: Vec<PathBuf> = Vec::new();
        let mut chains: Vec<String> = Vec::new();
        let mut mixed = Vec::new();
        let mut declined: Vec<Finding> = Vec::new();
        let mut internal: Option<String> = None;

        // What `validate` established about each file, under ADR-0092's observed identity
        // — never under `Probe::source`, which is a label.
        //
        // ADR-0093, ruling 3: read through `Established`, the one structure both verbs ask.
        // This block used to derive the same answer inline from `report.media`, which is
        // how `validate` and `render` came to answer one question from two data structures
        // and disagree about one file in one session. A consumer that re-derives usability
        // here has forked the data path again.
        //
        // (#385, for the record on why the identity is the thing compared: this once read
        // `canonicalize(&probe.source)`. `probe.source` holds whichever spelling first
        // cached the probe, and a *relative* spelling resolves against the calling
        // process's working directory — so a sidecar written by a CLI run inside the
        // project directory was unreadable to an MCP render started anywhere else.
        // `canonicalize` failed, `.ok()?` dropped the entry silently, every audible element
        // fell through to the "established nothing" arm, `chains` came out empty, and the
        // encoder took its `-an` branch: a complete, entirely silent video at exit 0. The
        // working directory had become a third input to a render that `CONTEXT.md` promises
        // is a function of the project and its files.)

        for (index, element) in document.elements().enumerate() {
            let kind = element.get("type").and_then(Value::as_str);
            if !matches!(kind, Some("audio") | Some("video")) {
                continue;
            }
            let name = element
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("(element {index} with no id)"));
            match chain(
                element,
                &name,
                kind,
                project_dir,
                established,
                fps,
                from,
                to,
                inputs.len() + 1,
            ) {
                Ok(Some((path, filter))) => {
                    inputs.push(path);
                    chains.push(filter);
                    mixed.push(name);
                }
                // Outside the range, or silent by declaration: not a thing the output is
                // missing.
                Ok(None) => {}
                // ADR-0093: located here rather than at the emission site, because the
                // element that declared the source is what an agent has to edit, and the
                // arms deep in `chain` have no business knowing which document they are in.
                Err(Declined::Finding(finding)) => {
                    declined.push(finding.at_file(document.path()).at_element(name.clone()))
                }
                Err(Declined::Internal(reason)) => internal = internal.or(Some(reason)),
                // The mix spawns nothing — `Encoder::start` is the caller's — so no arm of
                // `chain` can reach for a tool. Folded into the invariant channel rather
                // than given one of its own, because reaching it would itself be the
                // invariant violation.
                Err(Declined::Tool(missing)) => internal = internal.or(Some(missing.reason())),
            }
        }

        if chains.is_empty() {
            return Mix {
                audio: None,
                mixed,
                declined,
                internal,
            };
        }
        let mut graph = String::new();
        for (k, filter) in chains.iter().enumerate() {
            graph.push_str(&format!("{filter}[a{k}];\n"));
        }
        for k in 0..chains.len() {
            graph.push_str(&format!("[a{k}]"));
        }
        // Summed, padded with silence, and cut to exactly the span the frames cover — the
        // graph's own length is the audio stream's length, since the encoder applies no
        // `-t` (it would drop the last video frame).
        if chains.len() > 1 {
            graph.push_str(&format!("amix=inputs={}:normalize=0,", chains.len()));
        }
        graph.push_str(&format!("apad,atrim=end={}[mix]\n", seconds(to - from)));
        Mix {
            audio: Some(encode::Audio { inputs, graph }),
            mixed,
            declined,
            internal,
        }
    }
}

/// One audible element's filter chain, from `[input:a]` up to (not including) its output
/// label — or `None` where it has nothing to contribute to this range.
#[allow(clippy::too_many_arguments)]
fn chain(
    element: &Value,
    name: &str,
    kind: Option<&str>,
    project_dir: &FilePath,
    established: &Established,
    fps: i64,
    from: i64,
    to: i64,
    input: usize,
) -> Result<Option<(PathBuf, String)>, Declined> {
    let (Some(start), Some(end)) = (
        element.get("start").and_then(Value::as_i64),
        element.get("end").and_then(Value::as_i64),
    ) else {
        return Err(Declined::internal(name, "no integer `start`/`end`"));
    };
    // ADR-0093 gave this arm `E-EMPTY-RANGE` because no `validate` check stated it and the
    // render genuinely reached it. ADR-0107 moved the finding to `crate::checks::range`, so
    // the check engine refuses first and reaching here is the two halves disagreeing.
    if end <= start {
        return Err(Declined::internal(name, "an empty `start`..`end`"));
    }
    // The part of the element inside the render's range, in the element's own time.
    let window_start = from.max(start) - start;
    let window_end = to.min(end) - start;
    if window_end <= window_start {
        return Ok(None);
    }

    let Some(source) = element.get("source").and_then(Value::as_str) else {
        return Err(Declined::internal(name, "no `source`"));
    };
    let path = match Source::resolve(source, project_dir) {
        Source::Local(path) => path,
        Source::Remote(url) => {
            return Err(Declined::Finding(
                Finding::new("E-NOT-MIXED-REMOTE").field("source", json!(url)),
            ));
        }
    };
    // What `validate` established about the file, on the report this run carries. A
    // video with no audio stream is an ordinary video, not a defect — but it is still not
    // in the mix, and the answer says so.
    let identity = std::fs::canonicalize(&path).map_err(|e| {
        Declined::Finding(
            Finding::new("E-NOT-MIXED-UNREADABLE")
                .field("resolved", json!(display_local(&path)))
                .field("detail", json!(e.to_string())),
        )
    })?;
    match established.about(&identity) {
        Some(facts) if !facts.audio => {
            return Err(Declined::Finding(
                Finding::at_class("N-NO-AUDIO-STREAM", Class::Note)
                    .field("resolved", json!(display_local(&identity))),
            ));
        }
        Some(_) => {}
        // #385's first ask: an entry the engine declines to use must say *why*.
        // "established nothing" about a file that is sitting right there, readable, is the
        // least actionable sentence in the tool — so name the file that was looked for and
        // point at the line that holds the reason. Reaching this arm now means the check
        // engine genuinely recorded no probe for this file, rather than meaning it recorded
        // one under a spelling this process could not resolve.
        // ADR-0093's own finding, and the MONTAGENT-1 defect: this was one line of prose
        // in the render block, with no class attached and nothing counting it, while the
        // deliverable came out silent at exit 0. Ruling 3 is what makes the sentence below
        // true — `validate` now reports the same file `UNCHECKED` in the same session, so
        // *"the findings above say why"* is a pointer at something that is actually there.
        None => {
            return Err(Declined::Finding(
                Finding::new("E-NOT-MIXED-UNESTABLISHED")
                    .field("resolved", json!(display_local(&identity))),
            ));
        }
    }

    let (Some(source_start), Some(source_end)) = (
        element.get("source_start").and_then(Value::as_i64),
        element.get("source_end").and_then(Value::as_i64),
    ) else {
        return Err(Declined::internal(
            name,
            "no integer `source_start`/`source_end`",
        ));
    };
    let source_span = source_end - source_start;
    // The source half of the same fact, and unreachable for the same reason (ADR-0107).
    if source_span <= 0 {
        return Err(Declined::internal(
            name,
            "an empty `source_start`..`source_end`",
        ));
    }
    let speed = match element.get("speed") {
        None | Some(Value::Null) => Decimal::of(&serde_json::Number::from(1)),
        Some(value) => value.as_number().and_then(Decimal::of),
    }
    .filter(|speed| speed.is_positive())
    .ok_or_else(|| Declined::internal(name, "a `speed` that is not a positive number"))?;
    let played = exact::played_ms(source_span, speed).ok_or_else(|| {
        Declined::internal(name, "a source span and `speed` whose product overflowed")
    })?;

    let mut filter = format!(
        "[{input}:a]aformat=sample_rates={MIX_RATE}:channel_layouts=stereo,\
         atrim=start={}:end={},asetpts=PTS-STARTPTS",
        seconds(source_start),
        seconds(source_end)
    );
    for factor in atempo_chain(speed.as_f64()) {
        filter.push_str(&format!(",atempo={factor}"));
    }
    match element.get("overrun").and_then(Value::as_str) {
        Some("loop") => {
            // ADR-0020: the source restarts with a hard cut, and the last iteration
            // truncates at `end`. The loop is the as-played span, in samples at the bus
            // rate, and the truncation is the `atrim` below.
            let samples = played * MIX_RATE / 1000;
            filter.push_str(&format!(",aloop=loop=-1:size={samples}"));
        }
        // ADR-0020: on a video, `hold` freezes the last frame — and there is "no
        // non-arbitrary meaning for holding the last sample", so the audio simply ends
        // where the source does and the mix's padding is silence from there. On an
        // `audio` element the same field is a schema error, which the check engine has
        // already refused the render for; it is named here only so a document that
        // reaches this point malformed says why it was not mixed.
        // ADR-0093 ruling 2 called this `E-INTERNAL`-adjacent and left the call to this
        // ticket. It is not adjacent: `AudioOverrun` has no `hold` variant, so
        // `document.strict()` cannot produce a document that reaches here.
        Some("hold") if kind == Some("audio") => {
            return Err(Declined::internal(
                name,
                "`overrun: \"hold\"` on an audio element (ADR-0020)",
            ));
        }
        _ => {}
    }
    filter.push_str(&format!(
        ",atrim=end={},asetpts=PTS-STARTPTS",
        seconds(end - start)
    ));

    // `volume` (ADR-0055): a scalar as it is, a keyframe list as the value resolved on
    // every sampled frame of the element, applied as timed commands to the same filter.
    let label = format!("v{input}");
    match element.get("volume") {
        None | Some(Value::Null) => {}
        Some(written) => {
            // Read as [`Volume`], not as a bare `f64`: ADR-0055's *"negative is a schema
            // error"* is one bound, stated once, on the type — and reading the raw value
            // through it is what makes that true of the keyframed spelling too, which a
            // guard on the scalar arm alone would leave to reach `ffmpeg` and invert the
            // waveform at full level. The check engine has already refused the render for
            // it; this is the same belt-and-braces the `overrun: "hold"` arm above is.
            let volume: Animatable<Volume> =
                serde_json::from_value(written.clone()).map_err(|e| {
                    Declined::internal(
                        name,
                        &format!("a `volume` that does not fit the schema: {e}"),
                    )
                })?;
            let volume = match volume {
                Animatable::Static(Volume(v)) => Animatable::Static(v),
                Animatable::Keyed(records) => Animatable::Keyed(
                    records
                        .into_iter()
                        .map(|record| Keyframe {
                            t: record.t,
                            // Dropped, not carried: ADR-0086's declaration is
                            // renderer-ignored by construction, and this is the renderer.
                            t_from: None,
                            v: record.v.0,
                            ease: record.ease,
                        })
                        .collect(),
                ),
            };
            match volume {
                Animatable::Static(v) => {
                    if v != 1.0 {
                        filter.push_str(&format!(",volume={}", ratio(v)));
                    }
                }
                keyed => {
                    let at = |t: i64| {
                        resolve::at(&keyed, t).map_err(|_| {
                            Declined::internal(name, "`volume` keyframes that do not resolve")
                        })
                    };
                    let initial = at(start)?;
                    let mut commands = String::new();
                    let mut last = initial;
                    let Some(first) = exact::frame_at_or_after(start, fps) else {
                        return Err(Declined::Internal(format!(
                            "the mix was asked for {fps} fps, which is not a rate (ADR-0093)"
                        )));
                    };
                    let mut n = first.frame;
                    loop {
                        let instant = instant_of(n, fps);
                        if instant >= end {
                            break;
                        }
                        let v = at(instant)?;
                        if (v - last).abs() > 1e-6 {
                            commands.push_str(&format!(
                                "{} volume@{label} volume {};",
                                seconds(instant - start),
                                ratio(v)
                            ));
                            last = v;
                        }
                        n += 1;
                    }
                    if !commands.is_empty() {
                        filter.push_str(&format!(",asendcmd=c='{commands}'"));
                    }
                    filter.push_str(&format!(
                        ",volume@{label}=volume={}:eval=frame",
                        ratio(initial)
                    ));
                }
            }
        }
    }

    // The window inside the range, then the placement on the clock.
    filter.push_str(&format!(
        ",atrim=start={}:end={},asetpts=PTS-STARTPTS,adelay=delays={}:all=1",
        seconds(window_start),
        seconds(window_end),
        from.max(start) - from
    ));
    Ok(Some((path, filter)))
}

/// The whole millisecond frame `n` is painted at: `⌊n × 1000 / fps⌋`.
///
/// The one place the module doc's floor is spelled, for the frame loop and for the
/// `volume` commands alike — the two must sample the same instants, or a fade would be
/// heard on a different clock from the one it is seen on.
pub(crate) fn instant_of(n: i64, fps: i64) -> i64 {
    ((i128::from(n) * 1000) / i128::from(fps)) as i64
}

/// Milliseconds as `ffmpeg`'s decimal seconds — in the string, never through a float
/// (ADR-0005).
fn seconds(ms: i64) -> String {
    format!("{}.{:03}", ms.max(0) / 1000, ms.max(0) % 1000)
}

/// A ratio for a filter argument. Six places, ADR-0012's own precision for the continuous
/// properties, and enough that `0.645` survives.
fn ratio(v: f64) -> String {
    let text = format!("{v:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The `atempo` factors that multiply to `speed`, each inside one instance's `0.5..=2.0`.
///
/// Chained rather than trusting a newer `ffmpeg`'s wider range, so the same graph plays
/// on every `ffmpeg` an author is likely to have. An unchanged rate is no filter at all.
fn atempo_chain(speed: f64) -> Vec<String> {
    let mut factors = Vec::new();
    let mut remaining = speed;
    while remaining < 0.5 {
        factors.push("0.5".to_string());
        remaining /= 0.5;
    }
    while remaining > 2.0 {
        factors.push("2".to_string());
        remaining /= 2.0;
    }
    if (remaining - 1.0).abs() > 1e-9 {
        factors.push(ratio(remaining));
    }
    factors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_is_both_flags_or_neither_and_half_open() {
        assert_eq!(request(&Ask::default()), Ok(None));
        assert_eq!(
            request(&Ask {
                from: Some(1000),
                to: Some(2000),
                output: None,
                no_clobber: false
            }),
            Ok(Some((1000, 2000)))
        );
        for (from, to) in [
            (Some(1000), None),
            (None, Some(2000)),
            (Some(2000), Some(2000)),
            (Some(-1), Some(5)),
        ] {
            assert!(
                request(&Ask {
                    from,
                    to,
                    output: None,
                    no_clobber: false
                })
                .is_err(),
                "{from:?}..{to:?}"
            );
        }
    }

    /// ADR-0077, reading 1: the re-executable check behind *"at 25, 50, 100, 200, 500 and
    /// 1000 fps it is exact"* and behind the claim that flooring never moves an element in
    /// or out of a frame.
    #[test]
    fn the_painted_instant_is_the_grid_instant_floored_and_presence_is_unaffected() {
        // 30 fps: the grid is n × 100/3 ms, and the painter takes the whole millisecond
        // below it.
        assert_eq!(
            (0..7).map(|n| instant_of(n, 30)).collect::<Vec<_>>(),
            vec![0, 33, 66, 100, 133, 166, 200]
        );

        // Exact at the six rates that divide 1000, and only at those: one second of
        // frames at each rate, floor against the exact product.
        for fps in [25, 50, 100, 200, 500, 1000] {
            for n in 0..fps {
                assert_eq!(
                    i128::from(instant_of(n, fps)) * i128::from(fps),
                    i128::from(n) * 1000,
                    "{fps} fps is millisecond-exact, so the floor takes nothing at frame {n}"
                );
            }
        }
        for fps in [24, 30, 60] {
            assert!(
                (0..fps)
                    .any(|n| i128::from(instant_of(n, fps)) * i128::from(fps)
                        != i128::from(n) * 1000),
                "{fps} fps has a grid instant the floor moves"
            );
        }

        // Presence is unaffected, since every `start` and `end` is an integer:
        // `⌊t⌋ ≥ start ⇔ t ≥ start`, and the `end` half — `⌊t⌋ < end ⇔ t < end` — is the
        // negation of the same equivalence, so one scan settles both. Over a second of
        // frames at the awkward rates, against every whole-millisecond boundary in range.
        for fps in [24, 30, 60] {
            for n in 0..fps {
                let floored = i128::from(instant_of(n, fps));
                let exact = i128::from(n) * 1000; // t × fps, to stay in integers
                for boundary in 0..1000i128 {
                    assert_eq!(
                        floored >= boundary,
                        exact >= boundary * i128::from(fps),
                        "{fps} fps, frame {n}, boundary {boundary}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_atempo_chain_multiplies_to_the_speed_and_stays_inside_one_instances_range() {
        assert_eq!(atempo_chain(1.0), Vec::<String>::new());
        assert_eq!(atempo_chain(0.645), vec!["0.645"]);
        assert_eq!(atempo_chain(0.25), vec!["0.5", "0.5"]);
        assert_eq!(atempo_chain(0.2), vec!["0.5", "0.5", "0.8"]);
        assert_eq!(atempo_chain(3.0), vec!["2", "1.5"]);
    }

    #[test]
    fn seconds_are_spelled_from_integer_milliseconds() {
        assert_eq!(seconds(13172), "13.172");
        assert_eq!(seconds(5), "0.005");
        assert_eq!(seconds(0), "0.000");
    }

    #[test]
    fn two_spellings_of_one_path_are_one_path() {
        let dir =
            std::env::temp_dir().join(format!("montagent-render-same-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(dir.join("out"));
        assert!(same_path(
            &dir.join("out/./video.mp4"),
            &dir.join("out/../out/video.mp4")
        ));
        assert!(!same_path(
            &dir.join("out/video.mp4"),
            &dir.join("out/video.0-1000.mp4")
        ));
        // Nothing under `nowhere` exists; the comparison is still one path.
        assert!(same_path(
            &dir.join("nowhere/a/../video.mp4"),
            &dir.join("nowhere/video.mp4")
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_directory_cannot_smuggle_a_partial_render_onto_the_deliverable() {
        // `alias/` is a symlink to `out/`, and no deliverable exists yet: the two
        // spellings must still resolve to one file, before anything is written.
        let dir =
            std::env::temp_dir().join(format!("montagent-render-alias-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("out")).unwrap();
        std::os::unix::fs::symlink(dir.join("out"), dir.join("alias")).unwrap();
        assert!(same_path(
            &dir.join("alias/video.mp4"),
            &dir.join("out/video.mp4")
        ));
    }
}
