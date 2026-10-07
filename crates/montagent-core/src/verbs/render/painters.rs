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
//! - The canvas: cleared to the background at the start of every frame, with every `save`
//!   in the paint matched by a `restore`, and its only lasting state the surface's scale,
//!   set the same way when each painter's canvas is made.
//!
//! Thread-local switches the painter reads are handed from the calling thread to each
//! painter's: the feed budget and the filter-layer bound. The feed counts come back.
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
use crate::permissive::Loose;
use crate::verbs::frame::{Feeds, Painter, supply};

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
}

impl Painting {
    /// One painter on the verb's own thread: the span is one paint chunk, and one frame waits.
    pub(crate) fn one(frames: u64) -> Painting {
        Painting {
            painters: 1,
            chunk: frames,
            window: 1,
            window_floor: false,
        }
    }
}

/// What a test forces in place of [`plan`]'s choice.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forced {
    /// One painter on the verb's own thread, as a span with a `video` element is painted.
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
    // prototype(#750): `MONTAGENT_PROTO_DUMP=<dir>` writes every pushed frame's raw RGB as
    // `<dir>/<n>.rgb`; `MONTAGENT_PROTO_HASHES=<file>` appends one hash per frame, in order.
    static PUSHED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let index = PUSHED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if let Some(dir) = std::env::var_os("MONTAGENT_PROTO_DUMP") {
        let _ = std::fs::write(std::path::Path::new(&dir).join(format!("{index}.rgb")), rgb);
    }
    if let Some(path) = std::env::var_os("MONTAGENT_PROTO_HASHES") {
        use std::io::Write as _;
        let mut hasher = DefaultHasher::new();
        rgb.hash(&mut hasher);
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{:016x}", hasher.finish());
        }
    }
    TAP.with(|cell| {
        if let Some(hashes) = cell.borrow_mut().as_mut() {
            let mut hasher = DefaultHasher::new();
            rgb.hash(&mut hasher);
            hashes.push(hasher.finish());
        }
    });
}

/// The environment variable that forces K, C and W for a measurement: `K,C` or `K,C,W`, W in
/// frames.
///
/// `#[doc(hidden)]` measurement plumbing for ADR-0144's sweeps, which time the shipped binary
/// as a process, as [`super::STAGES_VAR`] is for ADR-0142's. Not a setting: nothing a project,
/// a flag or an MCP call says reaches it, and it changes no byte of the file. Malformed, it is
/// ignored.
#[doc(hidden)]
pub const PAINTING_VAR: &str = "MONTAGENT_PAINTING";

fn from_env(frame_bytes: u64) -> Option<Forced> {
    let value = std::env::var(PAINTING_VAR).ok()?;
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

/// How a span paints: `None` for one painter on the verb's own thread, or K painters over
/// paint chunks.
///
/// A document with a `video` element keeps one painter. Step 3 was opened for the paint class
/// (#647), and a paint chunk boundary reopens every visible feed: the ladder's reopen floor puts C in
/// the hundreds there, which at 1080p puts the window K·C at gigabytes, while the decode class
/// already meets its own target on one painter (ADR-0142). Otherwise K is the cores the
/// encoder leaves, C is [`CHUNK_FRAMES`], and W is [`WINDOW_BYTES`] at this resolution, or
/// K·C where that is more.
pub(super) fn plan(document: &Loose, surface: super::Surface, frames: u64) -> Option<Painting> {
    let frame_bytes = (surface.width.max(1) as u64) * (surface.height.max(1) as u64) * 3;
    let forced = FORCED.with(Cell::get).or_else(|| from_env(frame_bytes));
    let (painters, chunk, budget) = match forced {
        Some(Forced::OnePainter) => return None,
        Some(Forced::Chunks {
            painters,
            chunk,
            window_bytes,
        }) => (
            painters.max(1),
            chunk.max(1),
            window_bytes.unwrap_or(WINDOW_BYTES),
        ),
        None => {
            if has_video(document) {
                return None;
            }
            let cores = std::thread::available_parallelism().map_or(1, usize::from);
            let painters = cores.saturating_sub(ENCODER_CORES);
            if painters < 2 {
                return None;
            }
            (painters, CHUNK_FRAMES, WINDOW_BYTES)
        }
    };
    // A painter with no paint chunk to take is a thread for nothing.
    let painters = painters.min(frames.div_ceil(chunk).max(1) as usize);
    let floor = painters as u64 * chunk;
    let budgeted = budget / frame_bytes;
    Some(Painting {
        painters,
        chunk,
        window: budgeted.max(floor),
        window_floor: budgeted < floor,
    })
}

fn has_video(document: &Loose) -> bool {
    document
        .elements_in_tracks()
        .any(|(_, element)| element.get("type").and_then(|t| t.as_str()) == Some("video"))
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
    };

    let (encoded, painters) = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..painting.painters)
            .map(|_| {
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
#[derive(Clone, Copy)]
struct Inherited {
    feed_budget: u64,
    bounded: bool,
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
                    producer.insert(Producer::new(
                        span.document,
                        span.fps,
                        canvas,
                        Painter::for_a_deliverable(
                            span.document,
                            span.from,
                            (width, height),
                            Box::new(Feeds::new()),
                        ),
                    ))
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
        }
    }

    #[test]
    fn the_window_is_the_budget_at_this_resolution_and_never_below_k_times_c() {
        let document = Loose::new(
            "test.montagent.json",
            serde_json::json!({
                "fps": 30, "frame": {"width": 1920, "height": 1080}, "tracks": []
            }),
        );
        let hd = super::super::Surface::declared(1920, 1080);
        let _forced = force_painting(Forced::Chunks {
            painters: 6,
            chunk: 2,
            window_bytes: None,
        });
        let chosen = plan(&document, hd, 1080).expect("chunks");
        assert_eq!((chosen.window, chosen.window_floor), (150, false));

        let small = 10 * 1920 * 1080 * 3;
        let _forced = force_painting(Forced::Chunks {
            painters: 6,
            chunk: 4,
            window_bytes: Some(small),
        });
        let chosen = plan(&document, hd, 1080).expect("chunks");
        assert_eq!((chosen.window, chosen.window_floor), (24, true));

        // At a quarter of the pixels the same budget holds four times the frames.
        let quarter = super::super::Surface::declared(960, 540);
        let _forced = force_painting(Forced::Chunks {
            painters: 6,
            chunk: 4,
            window_bytes: Some(small),
        });
        assert_eq!(plan(&document, quarter, 1080).expect("chunks").window, 40);

        // No more painters than paint chunks.
        assert_eq!(plan(&document, hd, 5).expect("chunks").painters, 2);
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
