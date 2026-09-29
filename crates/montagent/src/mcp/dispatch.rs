//! How a tool call runs: off the runtime thread, announced while it waits, and heard from
//! while it works (ADR-0108, #439).
//!
//! **Why this module exists.** `serve()` runs one `current_thread` runtime, and every tool
//! was a synchronous `fn` that `rmcp` calls *inside the poll* — so a `render` held the only
//! thread for its whole encode. The service loop could not read the next request, write a
//! finished response, or send a notification. A call issued behind the render waited unread
//! with its client clock already running, and Claude Code's documented stdio idle window
//! (*"no response and no progress notification"* for 1800 s) aborted it. That was
//! MONTAGENT-9 (`docs/research/mcp-render-timeout.md` §4.1).
//!
//! Three things, each necessary:
//!
//! - **Every call runs in `spawn_blocking`**, so the loop keeps turning. `current_thread` is
//!   kept: nothing blocks it any more, and the verbs are CPU-bound on their own threads.
//! - **Progress goes on the wire** when the client supplied `_meta.progressToken`: each frame
//!   report the core makes, plus a **heartbeat** every [`heartbeat`] that covers the phases
//!   the core reports nothing from — pre-flight, `seal`, and waiting for the slot.
//! - **Encodes take one slot.** `render` and `preview` queue behind each other, and a queued
//!   call *says so* in its heartbeat, naming what it waits behind. One slot because two
//!   encodes race for the same cores, and a `preview` sharing them with a 4K `render` would
//!   blow ADR-0021's scrub budget on contention the project did not cause. It also closes
//!   ADR-0104's race by construction: no two publishing verbs are ever in flight in this
//!   process, so the foreign-output pre-flight and the publish cannot interleave.
//!
//! The stderr lines ADR-0011 specified stay, and gain the request id and the dispatch,
//! start and finish times, so the next incident is attributable without argument (research
//! §8.3). The dispatch line also records whether the call carried a `progressToken` —
//! the one fact about Claude Code's client its documentation does not state.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use rmcp::model::{CallToolResult, ProgressNotificationParam, ProgressToken, RequestId};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, Peer, RoleServer};
use tokio::sync::{Semaphore, mpsc};

use montagent_core::verbs::render::Progress;

/// How often a call that has said nothing else says it is alive.
///
/// **30 s**, which is what Anthropic chose for its own `claude mcp serve` (Claude Code
/// changelog 2.1.271) and a tenth of the *shortest* idle window Claude Code applies to any
/// transport (300 s for network servers; stdio's is 1800 s). A notification is one small
/// line, so the margin costs nothing.
const HEARTBEAT: Duration = Duration::from_secs(30);

/// The heartbeat interval, which a test may shorten.
///
/// `MONTAGENT_MCP_HEARTBEAT_MS` exists for the acceptance test alone — proving *"at least
/// every N s from dispatch to result"* at 30 s would need a render longer than the test
/// suite. It is read, not advertised: no tool schema carries it.
fn heartbeat() -> Duration {
    std::env::var("MONTAGENT_MCP_HEARTBEAT_MS")
        .ok()
        .and_then(|ms| ms.parse::<u64>().ok())
        .filter(|&ms| ms > 0)
        .map_or(HEARTBEAT, Duration::from_millis)
}

/// The one encode slot `render` and `preview` share (see the module docs for why one).
static ENCODE: Semaphore = Semaphore::const_new(1);

/// What holds [`ENCODE`], so a queued call can name what it waits behind.
static HOLDER: Mutex<Option<String>> = Mutex::new(None);

/// Whether a call needs the encode slot before it may start.
pub enum Slot {
    /// Reads, checks and single frames: they run as soon as they arrive.
    Free,
    /// `render` and `preview`, described for the heartbeat of whatever queues behind it.
    Encode(String),
}

/// One tool call, as this module needs to know it.
pub struct Call {
    tool: &'static str,
    id: RequestId,
    token: Option<ProgressToken>,
    peer: Peer<RoleServer>,
    dispatched: Instant,
}

impl Call {
    /// The call, and its dispatch line on stderr.
    pub fn of(tool: &'static str, context: &RequestContext<RoleServer>) -> Call {
        let token = context.meta.get_progress_token();
        let call = Call {
            tool,
            id: context.id.clone(),
            token,
            peer: context.peer.clone(),
            dispatched: Instant::now(),
        };
        eprintln!(
            "mcp  #{} {}  dispatched  progressToken: {}",
            call.id,
            call.tool,
            if call.token.is_some() {
                "present"
            } else {
                "absent"
            }
        );
        call
    }

    fn since(&self) -> f64 {
        self.dispatched.elapsed().as_secs_f64()
    }
}

/// Where the blocking work reports its frames. `render` and `preview` hand it to the core
/// as their progress callback; every other verb reports nothing and ignores it.
pub type Sink = Box<dyn FnMut(Progress) + Send>;

/// Run one call's work off the runtime thread, and keep the client informed until it ends.
pub async fn run<F>(call: Call, slot: Slot, work: F) -> Result<CallToolResult, ErrorData>
where
    F: FnOnce(Sink) -> CallToolResult + Send + 'static,
{
    let mut reporter = Reporter::new(call);
    let mut tick = tokio::time::interval(heartbeat());
    // The first tick of an interval is immediate; the dispatch line already said this call
    // exists, so the first heartbeat is one period in.
    tick.reset();

    // Waiting for the slot, announced. The permit lives until the work returns.
    let _permit = match &slot {
        Slot::Free => None,
        Slot::Encode(label) => {
            let acquire = ENCODE.acquire();
            tokio::pin!(acquire);
            let permit = loop {
                tokio::select! {
                    permit = &mut acquire => break permit,
                    _ = tick.tick() => {
                        let behind = HOLDER
                            .lock()
                            .map(|holder| holder.clone())
                            .unwrap_or(None)
                            .unwrap_or_else(|| "another encode".to_string());
                        reporter.send(None, format!("queued behind {behind}")).await;
                    }
                }
            };
            let permit =
                permit.map_err(|_| ErrorData::internal_error("the encode slot closed", None))?;
            if let Ok(mut holder) = HOLDER.lock() {
                *holder = Some(label.clone());
            }
            Some(Held(permit))
        }
    };
    eprintln!(
        "mcp  #{} {}  started  {:.1} s after dispatch",
        reporter.call.id,
        reporter.call.tool,
        reporter.call.since()
    );

    let (frames_tx, mut frames) = mpsc::unbounded_channel::<Progress>();
    let tool = reporter.call.tool;
    let sink: Sink = Box::new(move |p: Progress| {
        // ADR-0011's coarse stderr line, unchanged in shape.
        eprintln!(
            "{tool}  {}/{} frames  {:.1} s",
            p.done,
            p.of,
            p.elapsed.as_secs_f64()
        );
        // The receiver outlives the work, so a failed send cannot happen while it matters.
        let _ = frames_tx.send(p);
    });
    let mut handle = tokio::task::spawn_blocking(move || work(sink));

    let mut last: Option<Progress> = None;
    let joined = loop {
        tokio::select! {
            joined = &mut handle => break joined,
            Some(p) = frames.recv() => {
                reporter.send(Some(&p), frames_message(tool, &p)).await;
                last = Some(p);
                tick.reset();
            }
            _ = tick.tick() => {
                let message = match &last {
                    None => format!("{tool}: working, {:.0} s", reporter.call.since()),
                    Some(p) if p.done >= p.of => {
                        format!("{tool}: all {} frames drawn, finishing the file", p.of)
                    }
                    Some(p) => frames_message(tool, p),
                };
                reporter.send(last.as_ref(), message).await;
            }
        }
    };

    eprintln!(
        "mcp  #{} {}  finished  {:.1} s after dispatch",
        reporter.call.id,
        reporter.call.tool,
        reporter.call.since()
    );
    joined.map_err(|e| ErrorData::internal_error(format!("`{tool}` did not complete: {e}"), None))
}

fn frames_message(tool: &str, p: &Progress) -> String {
    format!(
        "{tool}: {}/{} frames, {:.0} s",
        p.done,
        p.of,
        p.elapsed.as_secs_f64()
    )
}

/// The encode permit, which also clears [`HOLDER`] when the work is over.
// The permit is held for its `Drop`, never read.
struct Held(#[allow(dead_code)] tokio::sync::SemaphorePermit<'static>);

impl Drop for Held {
    fn drop(&mut self) {
        if let Ok(mut holder) = HOLDER.lock() {
            *holder = None;
        }
    }
}

/// Sends `notifications/progress` for one call, keeping `progress` strictly increasing.
///
/// The spec: `progress` *"MUST increase with each notification, even if the total is
/// unknown"*. It counts **frames drawn** where the core has reported any, so a client's bar
/// means something; a notification that brings no new frame — a heartbeat, a queued call —
/// advances it by 1/1024 of a frame, an exact binary fraction that keeps the value honest to
/// within one frame for 1024 heartbeats (8.5 hours at 30 s). `total` is sent only while it
/// is not exceeded, so no client is told it is past 100%.
struct Reporter {
    call: Call,
    last: Option<f64>,
}

impl Reporter {
    fn new(call: Call) -> Reporter {
        Reporter { call, last: None }
    }

    async fn send(&mut self, frames: Option<&Progress>, message: String) {
        let Some(token) = self.call.token.clone() else {
            // No token, no stream: the spec lets a server notify only against a token the
            // client supplied. The stderr lines are the whole report.
            return;
        };
        let candidate = frames.map_or(0.0, |p| p.done as f64);
        let progress = match self.last {
            Some(last) if candidate <= last => last + 1.0 / 1024.0,
            _ => candidate,
        };
        self.last = Some(progress);

        let mut param = ProgressNotificationParam::new(token, progress).with_message(message);
        if let Some(p) = frames
            && progress <= p.of as f64
        {
            param = param.with_total(p.of as f64);
        }
        // A client that has gone away is not this call's failure to report: the result is
        // still produced, and `rmcp` drops it if nobody is waiting.
        let _ = self.call.peer.notify_progress(param).await;
    }
}
