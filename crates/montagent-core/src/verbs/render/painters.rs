//! Step 3 of #627's ladder: **K painters over C-frame paint chunks, feeding one encoder in timeline
//! order** (#653, ADR-0144).
//!
//! The span is cut into paint chunks of C frames, handed out to K painters in timeline order. Each
//! painter owns a whole [`Painter`] — its supplier, its font registry, its decoded stills —
//! and its own canvas, on its own thread; the document is shared read-only, and nothing
//! mutable is shared but the one [`Shared`] state below. The thread that called the verb is
//! the encoder's side: it takes frame after frame out of the reorder window, in timeline
//! order, and pushes each through [`super::encode_frames`], the same loop one painter feeds.
//!
//! ## Why the bytes cannot depend on K
//!
//! **A paint chunk start is indistinguishable from `--from`.** A painter that starts a paint chunk is a
//! painter that has painted some earlier frames, or none; a partial render that starts there
//! is a painter that has painted none. What a painter carries from one frame to the next is
//! all of this, and none of it reaches a pixel or the report:
//!
//! - `elements`, `frame`, `project_dir`, `class` and `fps`: fixed when it is built, from the
//!   document.
//! - `instant`, `frame_number`, `painted`, `not_painted`, `painted_partially`, `declined`,
//!   `transitions` and `running`: reset by `Painter::begin` before every frame.
//! - `registry`, the font registry: a cache. Shaping reads only the element's own declared
//!   chain (`Fonts::chain`, a `FontFamily::List`, with system fonts off), so a registry that
//!   holds more keys shapes the same glyphs.
//! - `stills`: a cache of decoded pixels by path, which since #651 own their pixels, so a hit
//!   and a fresh decode are the same raster. A still that cannot be read is not cached, and
//!   is refused with the same finding on every frame it is asked for.
//! - `videos` and `ffmpeg`: memoised probes and the resolved `ffmpeg`, the same answer on
//!   every frame of one render.
//! - `supplier`: frames from feeds, each provably the frame `frame_at` returns (ADR-0141).
//!   A painter that did not paint the frame before reopens; the frame is the same.
//! - `internal` and `tool_missing`: failures, which stop the span at the frame they are
//!   found on.
//! - `sources` and `fonts`: append-only records of every file used, read per paint chunk below.
//! - The static prefix: an immutable composite shared across painters (ADR-0195). A hit
//!   restores identical pixels and prefix-local paint records and source/font provenance.
//!   A miss paints normally, so chunk scheduling can change counts, never the picture.
//! - The canvas: cleared to the background at the start of every frame, with every `save`
//!   in the paint matched by a `restore`, and its only lasting state the surface's scale,
//!   set the same way when each painter's canvas is made.
//!
//! Thread-local switches the painter reads are handed from the calling thread to each
//! painter's: the feed budget, filter-layer bound and prefix-cache switch. The feed counts come back.
//!
//! ## The report
//!
//! Each paint chunk reports what its frames recorded ([`Made`]), and the encoder's side merges them
//! in paint chunk order with [`Made::then`]: `painted`, `sources`, `fonts` and `decoded_per_frame`
//! by first appearance, `declined` first per `(element, code)`, and the two reason sets as a
//! union — which is what one painter makes of the same frames one at a time. A painter's
//! `sources` and `fonts` record a file the first time *that painter* uses it, so a file first
//! used in paint chunk j is in paint chunk j's list (its painter cannot have used it earlier), and one
//! used again later, by a painter new to it, is deduplicated away.
//!
//! ## Failures
//!
//! Every failure carries the timeline frame it was found on; a painter that fails before it
//! paints (its canvas) takes its paint chunk's first frame. A failure at frame f stops all work after
//! f at once and lets work before f finish, and is acted on only when the encoder's side
//! reaches f — so the timeline-earliest failure is the one reported, whatever order the
//! painters found them in.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;

use montagent_render::canvas::{bound_filter_layers, filter_layers_bounded};

use super::{Made, Producer, Span, Stages, Stop};
use crate::verbs::frame::{Feeds, Painter, prefix, supply};

/// Conservative contended reopen allowance measured in ADR-0196.
const REOPEN: Duration = Duration::from_millis(1100);
/// The probe checks this ceiling between frames; one in-flight decode can overrun it.
const PROBE_BUDGET: Duration = Duration::from_secs(4);
const WINDOW_FACTOR: u64 = 2;

/// The cores left to the encoder when K is chosen: libx264 at ADR-0143's 5 threads keeps
/// about 3.6 cores busy on the trailer (#643), and its `ffmpeg` and the audio mix take the
/// rest of a fourth.
pub const ENCODER_CORES: usize = 4;

/// C, the frames in one paint chunk, measured on the trailer (ADR-0144).
pub const CHUNK_FRAMES: u64 = 2;

/// W's budget: the bytes of painted frames that may wait for the encoder, measured on the
/// trailer as 150 frames of 1080p RGB (ADR-0144). Turned into frames at each render's own
/// resolution, and never fewer than K·C ([`Painting::window_floor`]).
pub const WINDOW_BYTES: u64 = 150 * 1920 * 1080 * 3;

/// How a span's frames were painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Painting {
    /// K: how many painters, each on its own thread. `1` with no paint chunks is the painter on the
    /// verb's own thread, between pushes.
    pub painters: usize,
    /// C: the frames in one paint chunk.
    pub chunk: u64,
    /// W: how many painted frames may wait for the encoder.
    pub window: u64,
    /// Whether W is the floor K·C rather than what [`WINDOW_BYTES`] holds at this resolution:
    /// the budget held fewer frames than K painters need to each have a paint chunk in hand.
    pub window_floor: bool,
    /// Feed-process cap used to admit parallel video painting (ADR-0196).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_cap: Option<usize>,
    /// Peak video presence in the requested span, independent of probe samples.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_video_feeds: Option<usize>,
}

impl Painting {
    /// One painter on the verb's own thread: the span is one paint chunk, and one frame waits.
    pub(crate) fn one(frames: u64) -> Painting {
        Painting {
            painters: 1,
            chunk: frames,
            window: 1,
            window_floor: false,
            feed_cap: None,
            peak_video_feeds: None,
        }
    }
}

/// What a test forces in place of [`plan`]'s choice.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forced {
    /// One painter on the verb's own thread, without an adaptive scheduling probe.
    OnePainter,
    /// K painters over C-frame paint chunks — `painters: 1` is chunking with one painter — under
    /// `window_bytes`, or [`WINDOW_BYTES`] where `None`.
    Chunks {
        painters: usize,
        chunk: u64,
        window_bytes: Option<u64>,
    },
}

thread_local! {
    static FORCED: Cell<Option<Forced>> = const { Cell::new(None) };
    static FAULTS: RefCell<Vec<(i64, Duration)>> = const { RefCell::new(Vec::new()) };
    static TAP: RefCell<Option<Vec<u64>>> = const { RefCell::new(None) };
    static PROBE_CLOCK: Cell<Option<(usize, Option<Duration>)>> = const { Cell::new(None) };
}

/// Force how this thread's spans paint, until the guard drops.
///
/// `#[doc(hidden)]`, for the tests of #627 §8. There is no public setting: K, C and W change
/// how fast a render is, never what it is, and are the renderer's to choose.
#[doc(hidden)]
#[must_use = "the override lasts as long as the guard"]
pub struct ForcedPainting {
    before: Option<Forced>,
}

#[doc(hidden)]
pub fn force_painting(forced: Forced) -> ForcedPainting {
    let before = FORCED.with(|cell| cell.replace(Some(forced)));
    ForcedPainting { before }
}

impl Drop for ForcedPainting {
    fn drop(&mut self) {
        FORCED.with(|cell| cell.set(self.before));
    }
}

/// Override the machine's core count and the probe's measured frame time for tests.
/// `None` represents a probe that exhausted its wall-clock budget; `Some` replaces both
/// clocks (with no wall-budget overrun). The real probe still
/// paints its samples for a supplied duration, so identity tests exercise its lifecycle.
#[doc(hidden)]
#[must_use = "the override lasts as long as the guard"]
pub struct VideoProbeClock {
    before: Option<(usize, Option<Duration>)>,
}

#[doc(hidden)]
pub fn force_video_probe(cores: usize, frame_time: Option<Duration>) -> VideoProbeClock {
    let before = PROBE_CLOCK.with(|cell| cell.replace(Some((cores, frame_time))));
    VideoProbeClock { before }
}

impl Drop for VideoProbeClock {
    fn drop(&mut self) {
        PROBE_CLOCK.with(|cell| cell.set(self.before));
    }
}

/// Make each listed timeline frame fail on whichever painter reaches it, after a delay, until
/// the guard drops — the scheduling hook #627 §8 asks for, so a later paint chunk's failure can be
/// made to arrive first. Chunked spans only; `#[doc(hidden)]`, for the tests.
#[doc(hidden)]
#[must_use = "the faults last as long as the guard"]
pub struct FailFrames {
    before: Vec<(i64, Duration)>,
}

#[doc(hidden)]
pub fn fail_frames(at: &[(i64, Duration)]) -> FailFrames {
    let before = FAULTS.with(|cell| cell.replace(at.to_vec()));
    FailFrames { before }
}

impl Drop for FailFrames {
    fn drop(&mut self) {
        FAULTS.with(|cell| *cell.borrow_mut() = std::mem::take(&mut self.before));
    }
}

/// Record a hash of every frame this thread's spans hand the encoder, in the order pushed,
/// until the guard drops: #627 §8's primary oracle, the frames at the encoder's input.
/// `#[doc(hidden)]`, for the tests.
#[doc(hidden)]
#[must_use = "the tap lasts as long as the guard"]
pub struct FrameTap {
    before: Option<Vec<u64>>,
}

#[doc(hidden)]
pub fn tap_frames() -> FrameTap {
    let before = TAP.with(|cell| cell.replace(Some(Vec::new())));
    FrameTap { before }
}

impl FrameTap {
    /// The hashes so far, one per frame pushed.
    pub fn hashes(&self) -> Vec<u64> {
        TAP.with(|cell| cell.borrow().clone().unwrap_or_default())
    }
}

impl Drop for FrameTap {
    fn drop(&mut self) {
        TAP.with(|cell| *cell.borrow_mut() = self.before.take());
    }
}

/// Hash `rgb` into this thread's tap, where one is open.
pub(super) fn tapped(rgb: &[u8]) {
    TAP.with(|cell| {
        if let Some(hashes) = cell.borrow_mut().as_mut() {
            let mut hasher = DefaultHasher::new();
            rgb.hash(&mut hasher);
            hashes.push(hasher.finish());
        }
    });
}

/// The environment variable that forces K, C and W for a measurement: `K,C` or `K,C,W`, W in
/// frames, or `one` for the painter on the verb thread without a scheduling probe.
///
/// `#[doc(hidden)]` measurement plumbing for ADR-0144's sweeps, which time the shipped binary
/// as a process, as [`super::STAGES_VAR`] is for ADR-0142's. Not a setting: nothing a project,
/// a flag or an MCP call says reaches it, and it changes no byte of the file. Malformed, it is
/// ignored.
#[doc(hidden)]
pub const PAINTING_VAR: &str = "MONTAGENT_PAINTING";

fn from_env(frame_bytes: u64) -> Option<Forced> {
    let value = std::env::var(PAINTING_VAR).ok()?;
    if value.trim() == "one" {
        return Some(Forced::OnePainter);
    }
    let numbers: Vec<u64> = value
        .split(',')
        .map(|part| part.trim().parse().ok())
        .collect::<Option<_>>()?;
    match numbers[..] {
        [painters, chunk] => Some(Forced::Chunks {
            painters: painters as usize,
            chunk,
            window_bytes: None,
        }),
        [painters, chunk, window] => Some(Forced::Chunks {
            painters: painters as usize,
            chunk,
            window_bytes: Some(window.saturating_mul(frame_bytes)),
        }),
        _ => None,
    }
}

/// Scheduling mode is explicit: a forced one-frame window still means chunked work.
pub(super) enum Plan {
    One(Painting),
    Chunks(Painting),
}

/// Choose before any output painter or encoder exists. Forced measurement schedules
/// retain their old meaning; automatic video schedules amortise a measured reopen cost.
pub(super) fn plan(span: &Span<'_>, started: Instant) -> Result<Plan, Stop> {
    let (surface, frames) = (span.surface, span.frames);
    let frame_bytes = (surface.width.max(1) as u64) * (surface.height.max(1) as u64) * 3;
    let forced = FORCED.with(Cell::get).or_else(|| from_env(frame_bytes));
    match forced {
        Some(Forced::OnePainter) => return Ok(Plan::One(Painting::one(frames))),
        Some(Forced::Chunks {
            painters,
            chunk,
            window_bytes,
        }) => {
            return Ok(Plan::Chunks(chunk_plan(
                surface,
                frames,
                painters.max(1),
                chunk.max(1),
                window_bytes.unwrap_or(WINDOW_BYTES),
            )));
        }
        None => {}
    }
    let cores = PROBE_CLOCK.with(Cell::get).map_or_else(
        || std::thread::available_parallelism().map_or(1, usize::from),
        |(cores, _)| cores.max(1),
    );
    let mut painters = cores.saturating_sub(ENCODER_CORES);
    let peak = peak_video_feeds(span);
    if peak == 0 {
        return Ok(if painters < 2 {
            Plan::One(Painting::one(frames))
        } else {
            Plan::Chunks(chunk_plan(
                surface,
                frames,
                painters,
                CHUNK_FRAMES,
                WINDOW_BYTES,
            ))
        });
    }
    let cap = cores.saturating_mul(2);
    let mut one = Painting::one(frames);
    one.feed_cap = Some(cap);
    one.peak_video_feeds = Some(peak);
    // Apply the process cap before sampling or pricing the reorder window.
    painters = painters.min(cap / peak);
    if painters < 2 || frames < painters as u64 * CHUNK_FRAMES {
        return Ok(Plan::One(one));
    }
    let Some(p) = probe(span, started)? else {
        return Ok(Plan::One(one));
    };
    // C = ceil(N·R / (0.5·p)). Zero or sub-clock-resolution samples cannot justify
    // parallelism. Saturating conversion and products keep extreme inputs conservative.
    let p = p.as_secs_f64();
    if p <= 0.0 {
        return Ok(Plan::One(one));
    }
    let tax = peak as f64 * REOPEN.as_secs_f64();
    let chunk = ((tax / (0.5 * p)).ceil() as u64).max(CHUNK_FRAMES);
    let floor = (painters as u64).saturating_mul(chunk);
    let budgeted = WINDOW_BYTES / frame_bytes;
    let speedup = painters as f64 / (1.0 + tax / (chunk as f64 * p));
    if floor > budgeted.saturating_mul(WINDOW_FACTOR) || frames < floor || speedup < 1.5 {
        return Ok(Plan::One(one));
    }
    let mut chosen = chunk_plan(surface, frames, painters, chunk, WINDOW_BYTES);
    chosen.feed_cap = Some(cap);
    chosen.peak_video_feeds = Some(peak);
    Ok(Plan::Chunks(chosen))
}

fn chunk_plan(
    surface: super::Surface,
    frames: u64,
    painters: usize,
    chunk: u64,
    budget: u64,
) -> Painting {
    let frame_bytes = (surface.width.max(1) as u64) * (surface.height.max(1) as u64) * 3;
    let painters = painters.min(frames.div_ceil(chunk).max(1) as usize);
    let floor = (painters as u64).saturating_mul(chunk);
    let budgeted = budget / frame_bytes;
    Painting {
        painters,
        chunk,
        window: budgeted.max(floor),
        window_floor: budgeted < floor,
        feed_cap: None,
        peak_video_feeds: None,
    }
}

/// Three short consecutive runs, at the beginning, middle and end of the span. Each
/// uses a fresh producer and discards its first frame's cold opens. Neither pixels,
/// provenance nor prefix composites are reused by the output painters.
fn probe(span: &Span<'_>, started: Instant) -> Result<Option<Duration>, Stop> {
    let clock = PROBE_CLOCK.with(Cell::get);
    if clock.is_some_and(|(_, p)| p.is_none()) {
        return Ok(None);
    }
    let began = Instant::now();
    let mut samples = Vec::new();
    for offset in [
        0,
        span.frames.saturating_sub(4) / 2,
        span.frames.saturating_sub(4),
    ] {
        let Some(canvas) = span.surface.canvas() else {
            return Err(Stop::Internal(
                "no raster surface could be made for the video probe".into(),
            ));
        };
        let mut producer = Producer::new(
            span.document,
            span.fps,
            canvas,
            Painter::for_a_deliverable(
                span.document,
                span.from,
                span.declared,
                Box::new(Feeds::new()),
            ),
        )
        .with_alpha(span.alpha);
        for i in 0..4.min(span.frames) {
            if span.cancelled() {
                return Err(Stop::Cancelled { done: 0 });
            }
            if let Some(deadline) = span.deadline {
                let elapsed = started.elapsed();
                if elapsed > deadline {
                    return Err(Stop::Missed { elapsed, done: 0 });
                }
            }
            if clock.is_none() && began.elapsed() > PROBE_BUDGET {
                return Ok(None);
            }
            let first = Instant::now();
            producer.produce(span.first + (offset + i) as i64)?;
            if i > 0 {
                samples.push(first.elapsed());
            }
            if clock.is_none() && began.elapsed() > PROBE_BUDGET {
                return Ok(None);
            }
        }
    }
    // The final supplier teardown is part of the probe budget too.
    if span.cancelled() {
        return Err(Stop::Cancelled { done: 0 });
    }
    if let Some(deadline) = span.deadline {
        let elapsed = started.elapsed();
        if elapsed > deadline {
            return Err(Stop::Missed { elapsed, done: 0 });
        }
    }
    if clock.is_none() && began.elapsed() > PROBE_BUDGET {
        return Ok(None);
    }
    samples.sort_unstable();
    Ok(clock
        .and_then(|(_, p)| p)
        .or_else(|| samples.get(samples.len() / 2).copied()))
}

/// A sweep over half-open timeline intervals, clipped to this span. Counting even
/// occluded or transparent videos is conservative: the process cap cannot undercount.
fn peak_video_feeds(span: &Span<'_>) -> usize {
    let from = crate::exact::instant_of(span.first, span.fps);
    let to = crate::exact::instant_of(span.last, span.fps).saturating_add(1);
    let mut events = BTreeMap::<i64, i64>::new();
    for (_, element) in span.document.elements_in_tracks() {
        if element.get("type").and_then(|t| t.as_str()) != Some("video") {
            continue;
        }
        if let (Some(start), Some(end)) = (
            element.get("start").and_then(|v| v.as_i64()),
            element.get("end").and_then(|v| v.as_i64()),
        ) {
            let (start, end) = (start.max(from), end.min(to));
            if start < end {
                *events.entry(start).or_default() += 1;
                *events.entry(end).or_default() -= 1;
            }
        }
    }
    let (mut active, mut peak) = (0, 0);
    for change in events.values() {
        active += change;
        peak = peak.max(active);
    }
    peak as usize
}

/// Paint `span` on `painting.painters` painters, while `encode` — the encoder's side, on this
/// thread — takes the frames in timeline order through the function it is handed.
pub(super) fn paint(
    span: &Span<'_>,
    painting: Painting,
    stages: &mut Stages,
    encode: impl FnOnce(&mut dyn FnMut(i64) -> Result<Vec<u8>, Stop>, &mut Stages) -> Result<(), Stop>,
) -> Result<Made, Stop> {
    let shared = Shared::new(span.frames, painting);
    let faults = FAULTS.with(|cell| cell.borrow().clone());
    let inherited = Inherited {
        feed_budget: supply::budget(),
        bounded: filter_layers_bounded(),
        prefix_cache: prefix::enabled(),
        prefix: Default::default(),
    };

    let (encoded, painters) = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..painting.painters)
            .map(|_| {
                let inherited = inherited.clone();
                let (shared, faults) = (&shared, &faults[..]);
                scope.spawn(move || work(span, shared, faults, inherited))
            })
            .collect();

        // Whatever way the encoder's side leaves, every painter is told to stop, so the
        // scope's join never waits on one blocked on the window.
        let stopping = Stopping(&shared);
        let mut paint_wait = Duration::ZERO;
        let mut next = |n: i64| {
            let waiting = Instant::now();
            let frame = shared.take((n - span.first) as u64, span);
            paint_wait += waiting.elapsed();
            frame
        };
        let encoded = encode(&mut next, stages);
        drop(stopping);
        stages.paint_wait += paint_wait;

        let painters: Vec<Worked> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap_or_default())
            .collect();
        (encoded, painters)
    });
    for worked in &painters {
        stages.add(&worked.stages);
        supply::absorb(worked.counts);
    }
    encoded?;

    // Every frame reached the encoder, so every paint chunk's record should be here. One
    // missing — a painter that failed after its last frame was taken — is a report that
    // would be silently short, and is refused as one.
    let mut state = shared.lock();
    let chunks = span.frames.div_ceil(painting.chunk);
    if let Some((i, stop)) = state.failure.take() {
        let reason = match stop {
            Stop::Internal(reason) => reason,
            _ => format!("frame {}: a painter stopped", span.first + i as i64),
        };
        return Err(Stop::Internal(format!(
            "a painter failed after its frames were encoded: {reason}"
        )));
    }
    if state.made.len() as u64 != chunks {
        return Err(Stop::Internal(format!(
            "{} of {chunks} paint chunks reported what they painted",
            state.made.len()
        )));
    }
    let mut made = Made::default();
    for chunk in std::mem::take(&mut state.made).into_values() {
        made.then(chunk);
    }
    Ok(made)
}

/// The calling thread's switches, for each painter's thread.
#[derive(Clone)]
struct Inherited {
    feed_budget: u64,
    bounded: bool,
    prefix_cache: bool,
    prefix: prefix::Shared,
}

/// What one painter's thread hands back once it has stopped.
#[derive(Default)]
struct Worked {
    stages: Stages,
    counts: supply::Counts,
}

/// One painter: take paint chunks in timeline order until there are none, or until the span stops.
fn work(
    span: &Span<'_>,
    shared: &Shared,
    faults: &[(i64, Duration)],
    inherited: Inherited,
) -> Worked {
    let _budget = supply::override_budget(inherited.feed_budget);
    bound_filter_layers(inherited.bounded);
    let _prefix_cache = prefix::force_prefix_cache(inherited.prefix_cache);
    let (width, height) = span.declared;
    let painting = shared.painting;

    // The frame being worked on, so a panic is a failure at that frame rather than a frame
    // the encoder's side waits for forever.
    let current = Cell::new(0u64);
    let mut stages = Stages::default();
    let worked = catch_unwind(AssertUnwindSafe(|| {
        let mut producer: Option<Producer<'_>> = None;
        'chunks: while let Some(j) = shared.next_chunk() {
            let start = j * painting.chunk;
            let end = (start + painting.chunk).min(span.frames);
            current.set(start);
            let producer = match &mut producer {
                Some(producer) => producer,
                None => {
                    let Some(canvas) = span.surface.canvas() else {
                        shared.fail(
                            start,
                            Stop::Internal(format!(
                                "no raster surface could be made at {}x{}",
                                span.surface.width, span.surface.height
                            )),
                        );
                        break 'chunks;
                    };
                    let mut painter = Painter::for_a_deliverable(
                        span.document,
                        span.from,
                        (width, height),
                        Box::new(Feeds::new()),
                    );
                    painter.prefix = inherited.prefix.clone();
                    producer.insert(
                        Producer::new(span.document, span.fps, canvas, painter)
                            .with_alpha(span.alpha)
                            .profiling(span.profile),
                    )
                }
            };
            let mark = producer.mark();
            for i in start..end {
                current.set(i);
                if !shared.may_paint(i) {
                    break 'chunks;
                }
                // ADR-0109: checked by every painter as well as by the encoder's side.
                if span.cancelled() {
                    shared.stop();
                    break 'chunks;
                }
                let n = span.first + i as i64;
                if let Some((_, delay)) = faults.iter().find(|(at, _)| *at == n) {
                    std::thread::sleep(*delay);
                    shared.fail(i, Stop::Internal(format!("frame {n}: an injected failure")));
                    break 'chunks;
                }
                match producer.produce(n) {
                    Ok(rgb) => shared.deliver(i, rgb),
                    Err(stop) => {
                        shared.fail(i, stop);
                        break 'chunks;
                    }
                }
            }
            shared.made(j, producer.take(mark));
        }
        if let Some(producer) = &producer {
            stages = producer.stages();
        }
    }));
    if worked.is_err() {
        let n = span.first + current.get() as i64;
        shared.fail(
            current.get(),
            Stop::Internal(format!("frame {n}: a painter panicked")),
        );
    }
    Worked {
        stages,
        counts: supply::counts(),
    }
}

/// The one state the painters and the encoder's side share: the reorder window, the paint chunks'
/// records and the earliest failure.
struct Shared {
    frames: u64,
    painting: Painting,
    state: Mutex<State>,
    changed: Condvar,
}

#[derive(Default)]
struct State {
    /// The next paint chunk to hand out.
    next: u64,
    /// The frames, from the span's first, the encoder's side has taken.
    taken: u64,
    /// Painted frames waiting for the encoder, by index into the span.
    ready: BTreeMap<u64, Vec<u8>>,
    /// Each finished paint chunk's record, by paint chunk.
    made: BTreeMap<u64, Made>,
    /// The timeline-earliest failure found so far, at its index into the span.
    failure: Option<(u64, Stop)>,
    /// The span is over: cancelled, or the encoder's side has left.
    stopping: bool,
}

impl Shared {
    fn new(frames: u64, painting: Painting) -> Shared {
        Shared {
            frames,
            painting,
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wait<'g>(&self, guard: MutexGuard<'g, State>) -> MutexGuard<'g, State> {
        self.changed
            .wait(guard)
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// The next paint chunk in timeline order, unless there is none or the span has stopped before
    /// it.
    fn next_chunk(&self) -> Option<u64> {
        let mut state = self.lock();
        let j = state.next;
        let start = j * self.painting.chunk;
        if state.stopping
            || start >= self.frames
            || state.failure.as_ref().is_some_and(|(f, _)| start > *f)
        {
            return None;
        }
        state.next += 1;
        Some(j)
    }

    /// Wait until frame `i` is inside the window, and say whether it is still to be painted.
    fn may_paint(&self, i: u64) -> bool {
        let mut state = self.lock();
        loop {
            if state.stopping || state.failure.as_ref().is_some_and(|(f, _)| i > *f) {
                return false;
            }
            if i < state.taken + self.painting.window {
                return true;
            }
            state = self.wait(state);
        }
    }

    fn deliver(&self, i: u64, rgb: Vec<u8>) {
        self.lock().ready.insert(i, rgb);
        self.changed.notify_all();
    }

    fn made(&self, j: u64, made: Made) {
        self.lock().made.insert(j, made);
    }

    /// A failure at frame `i`: kept if it is the earliest yet, and every painter woken to see
    /// whether its work is now after it.
    fn fail(&self, i: u64, stop: Stop) {
        let mut state = self.lock();
        if state.failure.as_ref().is_none_or(|(f, _)| i < *f) {
            state.failure = Some((i, stop));
        }
        drop(state);
        self.changed.notify_all();
    }

    fn stop(&self) {
        self.lock().stopping = true;
        self.changed.notify_all();
    }

    /// The encoder's side: frame `i`, once painted, or the failure found on it. The cancel is
    /// polled while it waits, so a request lands even when every painter is busy.
    fn take(&self, i: u64, span: &Span<'_>) -> Result<Vec<u8>, Stop> {
        let mut state = self.lock();
        loop {
            if let Some(rgb) = state.ready.remove(&i) {
                state.taken = i + 1;
                drop(state);
                self.changed.notify_all();
                return Ok(rgb);
            }
            // Every frame before `i` has been taken, so a failure at or before it is the
            // timeline-earliest there will be.
            if state.failure.as_ref().is_some_and(|(f, _)| *f <= i) {
                let (_, stop) = state.failure.take().expect("just seen");
                return Err(stop);
            }
            if state.stopping || span.cancelled() {
                return Err(Stop::Cancelled { done: i });
            }
            state = self
                .changed
                .wait_timeout(state, Duration::from_millis(20))
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }
}

/// Stops the span when the encoder's side leaves, however it leaves.
struct Stopping<'s>(&'s Shared);

impl Drop for Stopping<'_> {
    fn drop(&mut self) {
        self.0.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painting(painters: usize, chunk: u64, window: u64) -> Painting {
        Painting {
            painters,
            chunk,
            window,
            window_floor: false,
            feed_cap: None,
            peak_video_feeds: None,
        }
    }

    #[test]
    fn the_window_is_the_budget_at_this_resolution_and_never_below_k_times_c() {
        let hd = super::super::Surface::declared(1920, 1080);
        let chosen = chunk_plan(hd, 1080, 6, 2, WINDOW_BYTES);
        assert_eq!((chosen.window, chosen.window_floor), (150, false));
        let small = 10 * 1920 * 1080 * 3;
        let chosen = chunk_plan(hd, 1080, 6, 4, small);
        assert_eq!((chosen.window, chosen.window_floor), (24, true));
        let quarter = super::super::Surface::declared(960, 540);
        assert_eq!(chunk_plan(quarter, 1080, 6, 4, small).window, 40);
        assert_eq!(chunk_plan(hd, 5, 6, 4, small).painters, 2);
    }

    #[test]
    fn the_encoder_side_takes_the_earliest_failure_and_a_later_one_stops_nothing_before_it() {
        let shared = Shared::new(8, painting(2, 2, 8));
        shared.fail(5, Stop::Internal("five".into()));
        shared.fail(3, Stop::Internal("three".into()));
        shared.fail(4, Stop::Internal("four".into()));
        assert!(
            shared.may_paint(3),
            "a failure at 3 stops nothing at 3 or before"
        );
        assert!(
            !shared.may_paint(4),
            "work after the earliest failure stops"
        );
        assert_eq!(shared.next_chunk(), Some(0));
        assert_eq!(shared.next_chunk(), Some(1));
        assert_eq!(shared.next_chunk(), None, "chunk 2 starts after frame 3");
    }
}
