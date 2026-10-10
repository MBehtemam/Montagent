//! What a running `render` says about itself (ADR-0192, #872).
//!
//! The principle is the ADR's: **Montagent states the status, and the agent never infers
//! it.** A run has a *phase*, and every surface names it. A number that is not there says why
//! it is not there. An agent never reads meaning into silence, a missing field or an
//! unchanged file without a rule written down.
//!
//! One [`Watch`] per `render` call holds the run's counters. It does three things:
//!
//! - **Enriches** each coarse [`Progress`] the core reports with the timeline position and an
//!   ETA, so the stderr line, the MCP stream and the progress file all read one source.
//! - **Writes the progress file**, if one was asked for, through a temporary name and a
//!   rename, so a reader never sees half a file. The first record is written before the
//!   project is opened, so a file left by an earlier run at the same path is replaced at
//!   once and an agent polling right after launch cannot read a stale `done` as this run's.
//! - **Runs a ticker thread** that rewrites the file about once a second and, when about
//!   [`LINE_GAP`] has passed with no line, hands the adapter a snapshot to print. The ticker
//!   exists because pre-flight (checks, probing, the mix) is one thread that reports nothing
//!   until the first frame: a line there cannot wait for the core to reach a step.
//!
//! The ETA is an **estimate**, from the rate over the last [`WINDOW`], because a rate read
//! off few points overclaims (ADR-0072: the reported render drifted 0.208 to 0.238 s/frame).

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use super::Progress;

/// How often the progress file is rewritten, and how often the ticker looks.
const WRITE_INTERVAL: Duration = Duration::from_secs(1);

/// How long a run goes without a stderr line before one is printed anyway.
const LINE_GAP: Duration = Duration::from_secs(5);

/// [`WRITE_INTERVAL`], which a test may shorten with `MONTAGENT_PROGRESS_WRITE_MS`.
///
/// Read, not advertised, like the MCP heartbeat's `MONTAGENT_MCP_HEARTBEAT_MS`: proving a
/// line *"whenever five seconds pass without one"* at five seconds would need a render
/// longer than the test suite.
fn write_interval() -> Duration {
    millis_from_env("MONTAGENT_PROGRESS_WRITE_MS").unwrap_or(WRITE_INTERVAL)
}

/// [`LINE_GAP`], which a test may shorten with `MONTAGENT_PROGRESS_LINE_GAP_MS`.
fn line_gap() -> Duration {
    millis_from_env("MONTAGENT_PROGRESS_LINE_GAP_MS").unwrap_or(LINE_GAP)
}

fn millis_from_env(name: &str) -> Option<Duration> {
    std::env::var(name)
        .ok()
        .and_then(|ms| ms.parse::<u64>().ok())
        .filter(|&ms| ms > 0)
        .map(Duration::from_millis)
}

/// Whole seconds as a bare integer, anything else with its fraction.
fn seconds(d: Duration) -> String {
    if d.subsec_nanos() == 0 {
        d.as_secs().to_string()
    } else {
        format!("{}", d.as_secs_f64())
    }
}

/// The span of recent frames the ETA's rate is read from.
const WINDOW: Duration = Duration::from_secs(30);

/// A rate needs at least this much wall clock behind it before it is called a rate.
const MIN_SPAN: Duration = Duration::from_secs(2);

/// Which part of a run it is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Checks, probing, the audio mix and starting the encoder: everything before frame 0.
    Preparing,
    /// Frames are being drawn and encoded.
    Rendering,
    /// Every frame is in; the encoder is closing the file and it is being renamed into place.
    Finishing,
}

impl Phase {
    /// The word every surface uses.
    pub fn word(self) -> &'static str {
        match self {
            Phase::Preparing => "preparing",
            Phase::Rendering => "rendering",
            Phase::Finishing => "finishing",
        }
    }

    fn code(self) -> u8 {
        match self {
            Phase::Preparing => 0,
            Phase::Rendering => 1,
            Phase::Finishing => 2,
        }
    }

    fn of(code: u8) -> Phase {
        match code {
            0 => Phase::Preparing,
            1 => Phase::Rendering,
            _ => Phase::Finishing,
        }
    }
}

/// How a run ended, or `Running` while it has not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Running,
    Done,
    Failed,
    Cancelled,
}

impl State {
    fn word(self) -> &'static str {
        match self {
            State::Running => "running",
            State::Done => "done",
            State::Failed => "failed",
            State::Cancelled => "cancelled",
        }
    }
}

/// The ETA, or the reason there is none. Never silently absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eta {
    /// Whole seconds left at the recent rate, where there is a rate.
    pub seconds: Option<u64>,
    /// Why there is no ETA. `None` exactly when `seconds` is `Some`.
    pub note: Option<&'static str>,
}

impl Eta {
    /// No ETA yet, and the sentence saying why.
    pub const fn none(note: &'static str) -> Eta {
        Eta {
            seconds: None,
            note: Some(note),
        }
    }
}

impl Progress {
    /// The one stderr line, for `tool` (`render`).
    ///
    /// Every line carries the phase, the timeline position and the ETA — labelled an
    /// estimate — or the words saying why there is none (ADR-0192 §2). The CLI and the MCP
    /// server print this same string, so an agent learns one shape.
    pub fn line(&self, tool: &str) -> String {
        let elapsed = self.elapsed.as_secs_f64();
        let eta = match (self.eta.seconds, self.eta.note) {
            (Some(seconds), _) => format!("ETA ~{seconds} s (an estimate)"),
            (None, Some(note)) => format!("no ETA yet: {note}"),
            (None, None) => "no ETA".to_string(),
        };
        match self.phase {
            Phase::Preparing => format!(
                "{tool}  preparing  checks, probing and the mix, before the first frame  \
                 {elapsed:.1} s  {eta}"
            ),
            Phase::Rendering => format!(
                "{tool}  rendering  {}/{} frames  timeline {:.1} s  {elapsed:.1} s  {eta}",
                self.done,
                self.of,
                self.timeline_ms as f64 / 1000.0,
            ),
            Phase::Finishing => format!(
                "{tool}  finishing  all {} frames drawn, closing the file  {elapsed:.1} s",
                self.of
            ),
        }
    }
}

/// The line printed when a run starts: its phase, where the progress file is (or how to ask
/// for one), and when the next line is due. `flag` is the surface's own spelling.
pub fn opening_line(tool: &str, progress_file: Option<&Path>, flag: &str) -> String {
    let next = format!("a line at least every {} s", seconds(line_gap()));
    match progress_file {
        Some(path) => format!(
            "{tool}  started  preparing  progress file {}, rewritten about every {} s  {next}",
            path.display(),
            seconds(write_interval())
        ),
        None => format!(
            "{tool}  started  preparing  tip: {flag} <path> writes a JSON status file to poll \
             (state, phase, frames, ETA)  {next}"
        ),
    }
}

/// One run's counters, its progress file and its ticker. See the module docs.
pub(crate) struct Watch {
    file: Option<PathBuf>,
    started: Instant,
    started_at: String,
    phase: AtomicU8,
    done: AtomicU64,
    of: AtomicU64,
    /// The first frame's instant and the project's frame rate, once the span is known.
    first_ms: AtomicI64,
    fps: AtomicI64,
    cancelled: AtomicBool,
    samples: Mutex<VecDeque<(Duration, u64)>>,
    last_line: Mutex<Instant>,
    terminal: Mutex<Option<State>>,
    stop: (Mutex<bool>, Condvar),
}

impl Watch {
    pub(crate) fn new(file: Option<PathBuf>) -> Watch {
        let now = Instant::now();
        Watch {
            file,
            started: now,
            started_at: rfc3339(SystemTime::now()),
            phase: AtomicU8::new(Phase::Preparing.code()),
            done: AtomicU64::new(0),
            of: AtomicU64::new(0),
            first_ms: AtomicI64::new(0),
            fps: AtomicI64::new(0),
            cancelled: AtomicBool::new(false),
            samples: Mutex::new(VecDeque::new()),
            last_line: Mutex::new(now),
            terminal: Mutex::new(None),
            stop: (Mutex::new(false), Condvar::new()),
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// Write the first record, `running` in `preparing`, before anything else is done.
    ///
    /// An `Err` is an invocation the caller can fix (`--progress-file` names somewhere that
    /// cannot be written), so the run refuses it as one rather than render for an agent that
    /// will poll a file that never appears. Every later write is best effort.
    pub(crate) fn begin(&self) -> Result<(), String> {
        let Some(path) = &self.file else {
            return Ok(());
        };
        if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| {
                format!(
                    "the progress file `{}` cannot be written: {e}",
                    path.display()
                )
            })?;
        }
        write_atomic(path, &self.record(State::Running)).map_err(|e| {
            format!(
                "the progress file `{}` cannot be written: {e}",
                path.display()
            )
        })
    }

    /// The span, once it is known: where it starts on the timeline, the project's frame rate
    /// and how many frames there are.
    pub(crate) fn plan(&self, first_ms: i64, fps: i64, frames: u64) {
        self.first_ms.store(first_ms, Ordering::Relaxed);
        self.fps.store(fps, Ordering::Relaxed);
        self.of.store(frames, Ordering::Relaxed);
        self.flush();
    }

    /// A frame is in. Called once per frame, so it only touches atomics.
    pub(crate) fn frame(&self, done: u64) {
        self.done.store(done, Ordering::Relaxed);
        self.phase.store(Phase::Rendering.code(), Ordering::Relaxed);
    }

    /// The run was cancelled where it stood.
    pub(crate) fn cancelled(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Take the coarse report the core made, add what only the watch knows, and write the
    /// file. What the adapter prints and sends is the returned value.
    pub(crate) fn report(&self, mut p: Progress) -> Progress {
        self.done.store(p.done, Ordering::Relaxed);
        self.of.store(p.of, Ordering::Relaxed);
        let phase = if p.of > 0 && p.done >= p.of {
            Phase::Finishing
        } else {
            Phase::Rendering
        };
        self.phase.store(phase.code(), Ordering::Relaxed);
        p.phase = phase;
        let snapshot = self.snapshot();
        p.timeline_ms = snapshot.timeline_ms;
        p.eta = snapshot.eta;
        *self.last_line.lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
        self.flush();
        p
    }

    /// The run as it stands, for an adapter to print.
    fn snapshot(&self) -> Progress {
        let elapsed = self.started.elapsed();
        let done = self.done.load(Ordering::Relaxed);
        let of = self.of.load(Ordering::Relaxed);
        let phase = Phase::of(self.phase.load(Ordering::Relaxed));
        let fps = self.fps.load(Ordering::Relaxed);
        let timeline_ms = if fps > 0 {
            self.first_ms.load(Ordering::Relaxed) + (done as i64 * 1000) / fps
        } else {
            self.first_ms.load(Ordering::Relaxed)
        };
        Progress {
            phase,
            done,
            of,
            elapsed,
            timeline_ms,
            eta: self.eta(elapsed, done, of, phase),
        }
    }

    /// The ETA from the rate over the recent window, or the reason there is none.
    fn eta(&self, elapsed: Duration, done: u64, of: u64, phase: Phase) -> Eta {
        let mut samples = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if samples.back().is_none_or(|&(at, _)| at < elapsed) {
            samples.push_back((elapsed, done));
        }
        while samples
            .front()
            .is_some_and(|&(at, _)| elapsed.saturating_sub(at) > WINDOW)
        {
            samples.pop_front();
        }
        match phase {
            Phase::Preparing => return Eta::none("no frames rendered yet"),
            Phase::Finishing => return Eta::none("every frame is drawn; the file is closing"),
            Phase::Rendering => {}
        }
        if of == 0 {
            return Eta::none("the number of frames is not known yet");
        }
        let Some(&(from_at, from_done)) = samples.front() else {
            return Eta::none("no frames rendered yet");
        };
        let span = elapsed.saturating_sub(from_at);
        if done == 0 {
            return Eta::none("no frames rendered yet");
        }
        if span < MIN_SPAN || done <= from_done {
            return Eta::none("too few frames so far to read a rate");
        }
        let rate = (done - from_done) as f64 / span.as_secs_f64();
        let left = of.saturating_sub(done) as f64;
        Eta {
            seconds: Some((left / rate).ceil() as u64),
            note: None,
        }
    }

    /// The frames per second drawn over the recent window, where there is a rate.
    fn throughput(&self) -> Option<f64> {
        let samples = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        let &(from_at, from_done) = samples.front()?;
        let &(to_at, to_done) = samples.back()?;
        let span = to_at.saturating_sub(from_at);
        (span >= MIN_SPAN && to_done > from_done)
            .then(|| (to_done - from_done) as f64 / span.as_secs_f64())
    }

    /// The JSON record for `state`.
    fn record(&self, state: State) -> Value {
        let p = self.snapshot();
        let planned = self.fps.load(Ordering::Relaxed) > 0;
        json!({
            "state": state.word(),
            "phase": p.phase.word(),
            "started_at": self.started_at,
            "updated_at": rfc3339(SystemTime::now()),
            "write_interval_s": write_interval().as_secs_f64(),
            "pid": std::process::id(),
            "elapsed_s": (p.elapsed.as_secs_f64() * 10.0).round() / 10.0,
            "frames_done": p.done,
            "frames_total": (p.of > 0).then_some(p.of),
            // Frames drawn per second over the recent window, not the project's frame rate.
            "fps": self.throughput().map(|rate| (rate * 100.0).round() / 100.0),
            "timeline_ms": planned.then_some(p.timeline_ms),
            "eta_s": p.eta.seconds,
            "eta_note": p.eta.note,
        })
    }

    /// Rewrite the file as the run stands. Best effort: a failed write never fails a render.
    fn flush(&self) {
        let Some(path) = &self.file else {
            return;
        };
        if self.terminal.lock().map(|t| t.is_some()).unwrap_or(true) {
            return;
        }
        let _ = write_atomic(path, &self.record(State::Running));
    }

    /// The last write, once: how the run ended. Later calls do nothing, so a normal finish
    /// and the guard's fallback cannot disagree.
    pub(crate) fn finish(&self, state: State) {
        {
            let mut terminal = self.terminal.lock().unwrap_or_else(|e| e.into_inner());
            if terminal.is_some() {
                return;
            }
            *terminal = Some(state);
        }
        if state == State::Done {
            self.phase.store(Phase::Finishing.code(), Ordering::Relaxed);
            let of = self.of.load(Ordering::Relaxed);
            self.done.store(of, Ordering::Relaxed);
        }
        if let Some(path) = &self.file {
            let _ = write_atomic(path, &self.record(state));
        }
    }

    /// How a run that returned an answer ended.
    pub(crate) fn finish_as(&self, produced_a_file: bool) {
        let state = if produced_a_file {
            State::Done
        } else if self.cancelled.load(Ordering::Relaxed) {
            State::Cancelled
        } else {
            State::Failed
        };
        self.finish(state);
    }

    /// Wake the ticker and tell it to leave.
    pub(crate) fn stop(&self) {
        let (flag, wake) = &self.stop;
        *flag.lock().unwrap_or_else(|e| e.into_inner()) = true;
        wake.notify_all();
    }

    /// The ticker: rewrite the file about once a second and, when about five seconds have
    /// passed with no line from the core, hand `line` a snapshot to print.
    pub(crate) fn tick(&self, line: Option<&(dyn Fn(Progress) + Sync)>) {
        let (flag, wake) = &self.stop;
        let mut stopped = flag.lock().unwrap_or_else(|e| e.into_inner());
        while !*stopped {
            let (guard, _) = wake
                .wait_timeout(stopped, write_interval())
                .unwrap_or_else(|e| e.into_inner());
            stopped = guard;
            if *stopped {
                break;
            }
            // Sampling and writing take other locks; the stop flag is not held across them.
            drop(stopped);
            self.flush();
            if let Some(line) = line {
                let quiet = self
                    .last_line
                    .lock()
                    .map(|at| at.elapsed() >= line_gap())
                    .unwrap_or(false);
                if quiet {
                    *self.last_line.lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
                    line(self.snapshot());
                }
            }
            stopped = flag.lock().unwrap_or_else(|e| e.into_inner());
        }
    }
}

/// Writes `Failed` if the run left without saying how it ended — a panic, say — so a reader
/// never finds a file stuck at `running` by a run that Montagent itself could have reported.
pub(crate) struct Guard<'a>(pub(crate) &'a Watch);

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.0.finish(State::Failed);
        self.0.stop();
    }
}

/// Write `value` to `path` through a temporary sibling and a rename.
fn write_atomic(path: &Path, value: &Value) -> std::io::Result<()> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "progress".to_string());
    let temp = path.with_file_name(format!(".{name}.tmp-{}", std::process::id()));
    let mut text = serde_json::to_string_pretty(value).unwrap_or_default();
    text.push('\n');
    std::fs::write(&temp, text)?;
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}

/// `YYYY-MM-DDTHH:MM:SS.mmmZ`, in UTC.
fn rfc3339(at: SystemTime) -> String {
    let since = at.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (secs, millis) = (since.as_secs() as i64, since.subsec_millis());
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_timestamp_is_rfc3339_in_utc() {
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
        let at = UNIX_EPOCH + Duration::from_millis(1_791_636_541_007);
        assert_eq!(rfc3339(at), "2026-10-10T12:49:01.007Z");
        // A leap day, which the civil-date arithmetic has to get right.
        let leap = UNIX_EPOCH + Duration::from_secs(1_709_164_800);
        assert_eq!(rfc3339(leap), "2024-02-29T00:00:00.000Z");
    }

    #[test]
    fn before_the_first_frame_there_is_no_eta_and_the_file_says_why() {
        let watch = Watch::new(None);
        let record = watch.record(State::Running);
        assert_eq!(record["state"], "running");
        assert_eq!(record["phase"], "preparing");
        assert_eq!(record["eta_s"], Value::Null);
        assert_eq!(record["eta_note"], "no frames rendered yet");
        assert_eq!(record["frames_total"], Value::Null);
        assert_eq!(record["timeline_ms"], Value::Null);
    }

    #[test]
    fn an_eta_needs_a_rate_and_is_a_number_only_then() {
        let watch = Watch::new(None);
        let eta = |elapsed_s, done, of| {
            watch.eta(Duration::from_secs(elapsed_s), done, of, Phase::Rendering)
        };
        assert_eq!(eta(0, 0, 100).seconds, None);
        assert_eq!(
            eta(1, 10, 100).note,
            Some("too few frames so far to read a rate")
        );
        // 10 frames at 0 s, 60 at 10 s: 5 frames a second, 40 left, 8 s.
        {
            let mut samples = watch.samples.lock().unwrap();
            samples.clear();
            samples.push_back((Duration::from_secs(0), 10));
        }
        assert_eq!(eta(10, 60, 100).seconds, Some(8));
        assert_eq!(eta(10, 60, 100).note, None);
    }

    #[test]
    fn a_line_always_names_its_phase_and_its_eta_or_why_not() {
        let preparing = Progress {
            phase: Phase::Preparing,
            done: 0,
            of: 0,
            elapsed: Duration::from_secs(6),
            timeline_ms: 0,
            eta: Eta::none("no frames rendered yet"),
        };
        let line = preparing.line("render");
        assert!(
            line.contains("preparing") && line.contains("no ETA yet: no frames"),
            "{line}"
        );
        let rendering = Progress {
            phase: Phase::Rendering,
            done: 4140,
            of: 13767,
            elapsed: Duration::from_secs(62),
            timeline_ms: 138_000,
            eta: Eta {
                seconds: Some(140),
                note: None,
            },
        };
        let line = rendering.line("render");
        assert!(
            line.contains("rendering")
                && line.contains("4140/13767 frames")
                && line.contains("timeline 138.0 s")
                && line.contains("ETA ~140 s (an estimate)"),
            "{line}"
        );
    }

    #[test]
    fn the_file_is_replaced_by_a_fresh_running_record_before_anything_else() {
        let dir = std::env::temp_dir().join(format!("montagent-watch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("progress.json");
        std::fs::write(&path, r#"{"state":"done"}"#).unwrap();
        let watch = Watch::new(Some(path.clone()));
        watch.begin().unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let record: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(record["state"], "running");
        assert_eq!(record["phase"], "preparing");
        assert!(record["started_at"].is_string() && record["pid"].is_number());
        // The terminal write is made once, and nothing after it changes the file.
        watch.finish(State::Failed);
        watch.finish(State::Done);
        watch.flush();
        let record: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(record["state"], "failed");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_that_cannot_be_written_is_refused_up_front() {
        let dir = std::env::temp_dir().join(format!("montagent-watch-no-{}", std::process::id()));
        std::fs::write(&dir, "a file where a folder is needed").unwrap();
        let watch = Watch::new(Some(dir.join("progress.json")));
        assert!(watch.begin().unwrap_err().contains("cannot be written"));
        std::fs::remove_file(&dir).unwrap();
    }
}
