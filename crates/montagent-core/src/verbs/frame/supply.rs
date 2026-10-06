//! Where a `video` element's pixels come from: the **frame supplier** the painter owns
//! (ADR-0141).
//!
//! ADR-0021's *"one painter"* is one place that decides what is painted — visibility, the
//! offset into the source, the extent and the findings. Where the decoded pixels come from
//! is not part of that decision, and it is the one thing `frame` and `render` need to do
//! differently: `frame` paints one instant and wants one frame, `render` paints thousands
//! of consecutive instants and was spawning an `ffmpeg` for every one of them (#532, about
//! an hour for six minutes of 1080p30 with three videos on screen).
//!
//! So the [`Painter`](super::Painter) is handed a [`FrameSupplier`] when it is built:
//!
//! - [`PerFrame`], for `frame` (and its sheet): one [`decode::frame_at`] per request, which is
//!   exactly what the painter did before this module existed.
//! - [`Feeds`], for `render` and `preview` (through `encode_span`): one long-lived `ffmpeg`
//!   per visible `video` element — a **feed** — read a frame at a time, capped by a memory
//!   budget.
//!
//! Nothing in the painter branches on which verb it serves, and nothing here decides
//! anything the picture shows: a request carries the offset the painter already computed,
//! and both suppliers answer with the frame [`decode::frame_at`] would return at it.
//! [`Feeds`] serves a request from a feed only where the feed's next frame is *provably* that
//! frame — [`decode::offset_at`] for the feed's next timeline frame equals the requested
//! offset — and otherwise holds, reopens or decodes the one frame.
//!
//! **A supplier never makes a finding.** A failure comes back as a [`Failure`], and the
//! painter turns it into `E-NOT-PAINTED-UNDECODABLE`, so ADR-0093's classes — `review` for
//! `frame`, `error` for `render` — stay in the one place that knows which verb is asking.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use montagent_render::decode::{self, DecodedFrame, Decoder, Frames, Origin, Pace};

/// The bytes of feed memory one painter may hold at once: 2 GiB (ADR-0141).
///
/// Estimated, never read off the machine: [`feed_bytes`] prices each feed from the probe and
/// the element's declared size, so a document splits between feeds and per-frame decodes the
/// same way on every machine, and the spawn count — which CI asserts — does not depend on
/// how much memory the runner has. It covers the feed processes and their prefetch only, not
/// the painter's canvas or the encoder. Raising it takes a superseding ADR; tests shrink it
/// through [`override_budget`].
pub const FEED_BUDGET: u64 = 2 * 1024 * 1024 * 1024;

/// A feed's fixed cost: the `ffmpeg` process before it holds any picture.
///
/// The three per-feed constants are fitted to ADR-0141's measured peak resident memory of a
/// feed-shaped `ffmpeg` at `-threads 1` (ffmpeg 9.0.2, M1 Pro, a reader pausing between
/// frames the way a painter does), over 1080x1920 and 2160x3840 H.264 sources painted at four
/// sizes, and sit above every reading — including a 10-bit H.264, a ProRes 422 10-bit and a
/// 16-reference H.264 sample. The fit's intercept is about 33 MB.
pub const FEED_FIXED_BYTES: u64 = 48 * 1024 * 1024;
/// A feed's cost per **source** pixel at [`decode::FEED_THREADS`]: the decoder's reference
/// frames and working buffers, which scale with what is decoded, not with what is painted.
/// Measured at about 14 bytes on the 8-bit benchmark sources and up to about 40 on the
/// 16-reference sample, which is what this is sized against.
pub const FEED_BYTES_PER_SOURCE_PIXEL: u64 = 40;
/// A feed's cost per **declared** pixel inside `ffmpeg`: the RGBA frames its pipeline queues
/// between the scaler and the pipe while the painter is busy. Measured at 62–74 bytes — about
/// sixteen frames' worth — and charged at twenty frames'.
pub const FEED_BYTES_PER_DECLARED_PIXEL: u64 = 80;
/// The frames a reader may hold ahead of the painter, at the declared size: ADR-0141's
/// two-frame prefetch, charged now so the estimate already covers #627's reader thread.
pub const FEED_PREFETCH_FRAMES: u64 = 2;

/// One feed's estimated peak memory, in bytes (ADR-0141): a fixed `ffmpeg` overhead, plus a
/// cost per probed **source** pixel, plus a cost per **declared** pixel for the frames
/// `ffmpeg` queues, plus the two-frame prefetch at the declared size.
///
/// A pure function of the probe and the document, so the budget decision is the same on
/// every machine. A source whose decoded size the probe did not establish is priced as 4K,
/// the largest size ADR-0003 puts in scope, rather than as nothing.
pub fn feed_bytes(source: Option<(u32, u32)>, declared: (u32, u32)) -> u64 {
    let (sw, sh) = source.unwrap_or((3840, 2160));
    let source_pixels = u64::from(sw) * u64::from(sh);
    let declared_pixels = u64::from(declared.0) * u64::from(declared.1);
    FEED_FIXED_BYTES
        + FEED_BYTES_PER_SOURCE_PIXEL * source_pixels
        + FEED_BYTES_PER_DECLARED_PIXEL * declared_pixels
        + FEED_PREFETCH_FRAMES * 4 * declared_pixels
}

/// Whether a feed of `bytes` fits beside the `held` bytes already admitted, under `budget`.
///
/// The whole admission test, kept allocation-free because it runs for every element without
/// a feed on every frame it is painted.
pub fn fits(budget: u64, held: u64, bytes: u64) -> bool {
    held.checked_add(bytes).is_some_and(|total| total <= budget)
}

/// First-fit over one frame's askers, in paint order: which of `asks` (estimated bytes, in
/// the order the painter asks) get a feed beside `held`, under `budget`.
///
/// [`Feeds`] makes the same decision one request at a time through [`fits`]; this is that
/// decision over a whole frame, for the pure test of the estimate and the rule. A later,
/// smaller ask can take room an earlier, larger one could not.
pub fn first_fit(budget: u64, held: u64, asks: &[u64]) -> Vec<bool> {
    let mut held = held;
    asks.iter()
        .map(|&bytes| {
            let admitted = fits(budget, held, bytes);
            if admitted {
                held += bytes;
            }
            admitted
        })
        .collect()
}

/// How many decodes this thread's painters have spawned, by kind.
///
/// `#[doc(hidden)]` evidence for the spawn-count tests (ADR-0141, ADR-0142 §4), and never part
/// of an answer: the public shape of what `render` reports about its speed is the speed
/// ticket's to set. Per thread, because a painter runs on the thread that calls the verb and
/// `cargo test` runs tests on many threads at once.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Feeds opened for an element that had none.
    pub opened: u64,
    /// Feeds reopened for an element that already held one: a loop wrap, a skip, an offset
    /// the feed's next frame is not.
    pub reopened: u64,
    /// Single frames decoded through [`decode::frame_at`]: every request of [`PerFrame`],
    /// every frame of an element past the feed budget, and each terminal frame.
    pub frame_at: u64,
    /// Feeds alive right now — a gauge, not a count. Zero after every span, however it ended.
    pub open: u64,
}

thread_local! {
    static COUNTS: Cell<Counts> = const {
        Cell::new(Counts {
            opened: 0,
            reopened: 0,
            frame_at: 0,
            open: 0,
        })
    };
    static BUDGET: Cell<Option<u64>> = const { Cell::new(None) };
}

fn count(change: impl FnOnce(&mut Counts)) {
    COUNTS.with(|cell| {
        let mut counts = cell.get();
        change(&mut counts);
        cell.set(counts);
    });
}

/// This thread's decode counts.
#[doc(hidden)]
pub fn counts() -> Counts {
    COUNTS.with(Cell::get)
}

/// Zero this thread's counts, leaving the [`Counts::open`] gauge alone.
#[doc(hidden)]
pub fn reset_counts() {
    count(|counts| {
        *counts = Counts {
            open: counts.open,
            ..Counts::default()
        }
    });
}

/// Add the decodes another thread's painter spawned to this thread's counts.
///
/// A render that paints on more than one thread (#653) hands each painter's counts back to
/// the thread that called the verb, which is the thread the spawn-count tests read.
pub(crate) fn absorb(other: Counts) {
    count(|counts| {
        counts.opened += other.opened;
        counts.reopened += other.reopened;
        counts.frame_at += other.frame_at;
        counts.open += other.open;
    });
}

/// This thread's feed budget: [`FEED_BUDGET`], or the override in force.
pub(crate) fn budget() -> u64 {
    BUDGET.with(Cell::get).unwrap_or(FEED_BUDGET)
}

/// A shrunken feed budget for this thread's painters, until the guard drops.
///
/// `#[doc(hidden)]`, for the feed-budget tests. There is no public setting (ADR-0141): a
/// fraction of the machine's memory would make the split, the answer and the spawn count
/// depend on the machine.
#[doc(hidden)]
#[must_use = "the override lasts as long as the guard"]
pub struct BudgetOverride {
    before: Option<u64>,
}

#[doc(hidden)]
pub fn override_budget(bytes: u64) -> BudgetOverride {
    let before = BUDGET.with(|cell| cell.replace(Some(bytes)));
    BudgetOverride { before }
}

impl Drop for BudgetOverride {
    fn drop(&mut self) {
        BUDGET.with(|cell| cell.set(self.before));
    }
}

/// What the painter's probe says about one video source, for a supplier.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Video {
    /// ADR-0089's decoder choice.
    pub decoder: Decoder,
    /// Where the video stream ends in source time: `start_time + video_stream_ms`
    /// (`media/probe.rs`). Never the container's duration where the stream states its own.
    pub end_ms: Option<i64>,
    /// One source frame, in whole milliseconds rounded up — the tolerance on that end.
    pub frame_ms: Option<i64>,
    /// The decoded size, which a feed's decoder works at whatever size is painted.
    pub pixels: Option<(u32, u32)>,
}

/// One element's pixels, asked for at one timeline frame.
pub(crate) struct Request<'r> {
    /// The element's index in the painter's element table — stable for the painter's life.
    pub key: usize,
    /// The timeline frame being painted, where the painter is painting one (`render`,
    /// `preview`); `None` for `frame`, which paints an instant.
    pub frame: Option<i64>,
    /// The offset into the source the painter computed (ADR-0020), from the caption.
    pub offset: i64,
    /// Where the current pass over the source began (`query::at::Present::source_origin`).
    pub origin: Option<(i64, i64)>,
    /// The declared size: fixed per element, since `width`/`height` are not animatable.
    pub width: u32,
    pub height: u32,
    pub ffmpeg: &'r Path,
    pub source: &'r Path,
    pub video: Video,
    /// The project's `fps` and the element's `speed`. `1` on a remapped element.
    pub pace: Pace,
    /// Set on an element carrying `source_time` (ADR-0157), whose offset is decided frame by
    /// frame by its curve rather than by a pass at one `speed`.
    pub remap: Option<Remap>,
}

/// What a supplier knows about a remapped element's curve beyond this frame (ADR-0157).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Remap {
    /// The offset the curve resolves at the next timeline frame, where it resolves.
    pub next: Option<i64>,
}

impl Request<'_> {
    fn source_name(&self) -> String {
        self.source.to_string_lossy().into_owned()
    }

    /// [`decode::frame_at`] at this request's offset, counted.
    fn frame_at(&self) -> Result<DecodedFrame, Failure> {
        count(|counts| counts.frame_at += 1);
        decode::frame_at(
            self.ffmpeg,
            &self.source_name(),
            self.video.decoder,
            self.offset,
            self.width,
            self.height,
        )
        .map_err(Failure::Seek)
    }
}

/// Why a supplier has no frame, for the painter to turn into a finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Failure {
    /// [`decode::frame_at`] refused, with its own sentence (ADR-0096, ADR-0113).
    Seek(String),
    /// A feed's `ffmpeg` failed, or ended cleanly before the source does (ADR-0141). Never
    /// retried through [`decode::frame_at`]: ADR-0113's rule, which ADR-0115 made an
    /// invariant over every spawn.
    Feed {
        source: String,
        /// Where the feed opened, as an offset into the source.
        from_ms: i64,
        /// The frames it delivered before this.
        delivered: u64,
        /// What went wrong, with `ffmpeg`'s stderr in it.
        said: String,
    },
}

impl Failure {
    /// The finding's `detail`.
    pub(crate) fn detail(&self) -> String {
        match self {
            Failure::Seek(said) => said.clone(),
            Failure::Feed {
                source,
                from_ms,
                delivered,
                said,
            } => format!(
                "{source}: the feed opened at {from_ms} ms into the source stopped after \
                 {delivered} frame(s): {said}"
            ),
        }
    }
}

/// Where a painter gets a `video` element's pixels.
pub(crate) trait FrameSupplier {
    /// A new timeline frame starts. A supplier holding feeds closes every one that was not
    /// asked for during the frame before, so an early return from a paint cannot leak a
    /// child process past the next frame, and the bytes it held are free for this one.
    fn begin_frame(&mut self, n: i64);

    /// The frame [`decode::frame_at`] would return at `request.offset`, at the declared size.
    fn supply(&mut self, request: &Request<'_>) -> Result<DecodedFrame, Failure>;

    /// The element keys painted per frame because the feed budget was full, in the order
    /// first so decoded.
    fn decoded_per_frame(&self) -> &[usize] {
        &[]
    }

    /// The wall clock spent decoding so far, for the `#[doc(hidden)]` stage breakdown.
    fn decoding(&self) -> Duration;
}

/// `frame`'s supplier: one [`decode::frame_at`] per request, as the painter always did.
#[derive(Default)]
pub(crate) struct PerFrame {
    decoding: Duration,
}

impl FrameSupplier for PerFrame {
    fn begin_frame(&mut self, _n: i64) {}

    fn supply(&mut self, request: &Request<'_>) -> Result<DecodedFrame, Failure> {
        let started = Instant::now();
        let frame = request.frame_at();
        self.decoding += started.elapsed();
        frame
    }

    fn decoding(&self) -> Duration {
        self.decoding
    }
}

/// One live feed: the `ffmpeg`, and the arithmetic its next frame is predicted with.
struct Feed {
    frames: Frames,
    origin: Origin,
    pace: Pace,
    /// The timeline frame its next frame is for.
    next: i64,
    /// Where it opened, as an offset into the source.
    from_ms: i64,
    delivered: u64,
}

impl Drop for Feed {
    /// The `ffmpeg` goes with [`Frames`]' own `Drop`, which kills and reaps it; this keeps the
    /// gauge honest about it.
    fn drop(&mut self) {
        count(|counts| counts.open = counts.open.saturating_sub(1));
    }
}

/// One element's claim on the budget, held from its first admitted frame until the first
/// frame it is not painted on.
struct Slot {
    /// The estimate it was admitted at. A reopen keeps it; only leaving frees it.
    bytes: u64,
    feed: Option<Feed>,
    /// The last frame served, at its offset: a hold is served from here.
    last: Option<(i64, DecodedFrame)>,
    /// The frame at or past the source's end, once a feed has reached it: every later
    /// offset gets it without another spawn.
    terminal: Option<(i64, DecodedFrame)>,
}

/// `render`'s supplier: a feed per visible `video` element, inside [`FEED_BUDGET`].
///
/// A request is served by the first of these that applies, in this order (the order is the
/// rule, ADR-0141):
///
/// 1. **The feed's predicted next frame** — the request is for the feed's next timeline frame
///    at [`decode::offset_at`] of it — read off the feed.
/// 2. **A hold** — the same offset as the last frame served — served from the cache. After
///    the prediction, because at a slow `speed` the predicted next offset can equal the last
///    one, and checking the hold first would leave the feed a frame behind for good.
/// 3. **Anything else** — a loop wrap, a re-entry, a skip — the feed is dropped and reopened
///    at this frame.
///
/// An element without a feed asks for one when it is painted; it gets one if its
/// [`feed_bytes`] [`fits`] beside what is held, and is otherwise decoded per frame through
/// [`decode::frame_at`] for that frame. That is a capacity decision taken before any feed
/// opens, never a reaction to a failure: a feed that fails is refused, not retried.
pub(crate) struct Feeds {
    budget: u64,
    held: u64,
    slots: HashMap<usize, Slot>,
    /// The keys asked for during the current frame.
    asked: Vec<usize>,
    per_frame: Vec<usize>,
    decoding: Duration,
}

impl Feeds {
    pub(crate) fn new() -> Feeds {
        Feeds {
            budget: budget(),
            held: 0,
            slots: HashMap::new(),
            asked: Vec::new(),
            per_frame: Vec::new(),
            decoding: Duration::ZERO,
        }
    }

    fn serve(&mut self, request: &Request<'_>, n: i64) -> Result<DecodedFrame, Failure> {
        if !self.asked.contains(&request.key) {
            self.asked.push(request.key);
        }
        // A speed `ffmpeg`'s filter cannot sample exactly (ADR-0127 §2) has no feed to read:
        // it is painted per frame, as `frame_at` always painted it.
        if !decode::exact_pace(request.pace) {
            return request.frame_at();
        }

        if !self.slots.contains_key(&request.key) {
            let bytes = feed_bytes(request.video.pixels, (request.width, request.height));
            if !fits(self.budget, self.held, bytes) {
                if !self.per_frame.contains(&request.key) {
                    self.per_frame.push(request.key);
                }
                return request.frame_at();
            }
            self.held += bytes;
            self.slots.insert(
                request.key,
                Slot {
                    bytes,
                    feed: None,
                    last: None,
                    terminal: None,
                },
            );
        }
        let slot = self.slots.get_mut(&request.key).expect("just admitted");

        if let Some((from, frame)) = &slot.terminal
            && request.offset >= *from
        {
            return Ok(frame.clone());
        }

        // 1. The feed's predicted next frame.
        if let Some(feed) = &slot.feed
            && feed.next == n
            && decode::offset_at(feed.origin, feed.pace, n) == request.offset
        {
            return read(slot, request);
        }
        // 2. A hold.
        if let Some((offset, frame)) = &slot.last
            && *offset == request.offset
        {
            return Ok(frame.clone());
        }
        let pace = request.pace;
        let instant = crate::exact::instant_of(n, pace.fps);
        // ADR-0157: a remapped element rides a feed only on a rising 1× stretch, where the
        // feed's next frame is provably the one the curve picks there too. A slower, faster,
        // flat or falling stretch is decoded a frame at a time: a feed opened for it would be
        // reopened on the next frame.
        if let Some(remap) = request.remap {
            let here = Origin {
                timeline_ms: instant,
                source_ms: request.offset,
            };
            let rising = Pace {
                fps: pace.fps,
                speed: (1, 1),
            };
            if remap.next != Some(decode::offset_at(here, rising, n + 1)) {
                slot.feed = None;
                let frame = request.frame_at()?;
                slot.last = Some((request.offset, frame.clone()));
                return Ok(frame);
            }
        }
        // 3. Anything else: open, or reopen, at this frame.
        let reopening = slot.feed.is_some() || slot.last.is_some() || slot.terminal.is_some();
        slot.feed = None;
        let mut origin = match request.origin {
            Some((timeline_ms, source_ms)) => Origin {
                timeline_ms,
                source_ms,
            },
            None => Origin {
                timeline_ms: instant,
                source_ms: request.offset,
            },
        };
        // The pass's own origin reproduces the painter's offset by construction; where it
        // somehow does not, the feed is measured from here instead, which is still exact for
        // this frame and only costs a reopen later.
        if decode::offset_at(origin, pace, n) != request.offset {
            origin = Origin {
                timeline_ms: instant,
                source_ms: request.offset,
            };
        }
        let frames = decode::frames_at(
            request.ffmpeg,
            &request.source_name(),
            request.video.decoder,
            origin,
            n,
            pace,
            request.width,
            request.height,
        )
        .map_err(|said| Failure::Feed {
            source: request.source_name(),
            from_ms: request.offset,
            delivered: 0,
            said,
        })?;
        count(|counts| {
            if reopening {
                counts.reopened += 1;
            } else {
                counts.opened += 1;
            }
            counts.open += 1;
        });
        slot.feed = Some(Feed {
            frames,
            origin,
            pace,
            next: n,
            from_ms: request.offset,
            delivered: 0,
        });
        read(slot, request)
    }
}

/// Read the slot's feed's next frame, which the caller has established is the one asked for.
///
/// **A clean end is accepted only at the source's end** (ADR-0141): at or past the probed
/// video-stream end less one source frame. There one [`decode::frame_at`] at the asked offset
/// becomes the element's terminal frame — ADR-0096's at-or-before rule stays `frame_at`'s at
/// the edge — and every later offset gets it. A feed that ends cleanly anywhere earlier is
/// refused: a feed that stops early because of a bug must fail loudly, never freeze on a
/// frame.
fn read(slot: &mut Slot, request: &Request<'_>) -> Result<DecodedFrame, Failure> {
    let feed = slot.feed.as_mut().expect("the caller checked the feed");
    let failed = |feed: &Feed, said: String| Failure::Feed {
        source: request.source_name(),
        from_ms: feed.from_ms,
        delivered: feed.delivered,
        said,
    };
    match feed.frames.next_frame() {
        Ok(Some(frame)) => {
            feed.next += 1;
            feed.delivered += 1;
            if frame.width != request.width || frame.height != request.height {
                return Err(failed(
                    feed,
                    format!(
                        "it delivered a {}x{} frame for a {}x{} element",
                        frame.width, frame.height, request.width, request.height
                    ),
                ));
            }
            slot.last = Some((request.offset, frame.clone()));
            Ok(frame)
        }
        Ok(None) => {
            let at_the_end = match (request.video.end_ms, request.video.frame_ms) {
                (Some(end), frame_ms) => request.offset >= end - frame_ms.unwrap_or(0),
                // A source whose probe states no end at all: `frame_at` is the authority on
                // whether there is a frame here, and it refuses where there is none.
                (None, _) => true,
            };
            if !at_the_end {
                let said = feed.frames.stderr();
                let said = said.trim();
                return Err(failed(
                    feed,
                    format!(
                        "it ended cleanly at {} ms, before the video stream ends at {} ms{}",
                        request.offset,
                        request.video.end_ms.unwrap_or_default(),
                        if said.is_empty() {
                            String::new()
                        } else {
                            format!(" — {said}")
                        }
                    ),
                ));
            }
            slot.feed = None;
            let frame = request.frame_at()?;
            slot.terminal = Some((request.offset, frame.clone()));
            slot.last = Some((request.offset, frame.clone()));
            Ok(frame)
        }
        Err(said) => Err(failed(feed, said)),
    }
}

impl FrameSupplier for Feeds {
    fn begin_frame(&mut self, _n: i64) {
        let asked = std::mem::take(&mut self.asked);
        let mut freed = 0;
        self.slots.retain(|key, slot| {
            let keep = asked.contains(key);
            if !keep {
                freed += slot.bytes;
            }
            keep
        });
        self.held -= freed;
        self.asked = asked;
        self.asked.clear();
    }

    fn supply(&mut self, request: &Request<'_>) -> Result<DecodedFrame, Failure> {
        let started = Instant::now();
        let frame = match request.frame {
            Some(n) => self.serve(request, n),
            // A painter that paints instants rather than frames has nothing for a feed to
            // predict; it is `PerFrame`'s caller, and this arm only keeps the answer right.
            None => request.frame_at(),
        };
        self.decoding += started.elapsed();
        frame
    }

    fn decoded_per_frame(&self) -> &[usize] {
        &self.per_frame
    }

    fn decoding(&self) -> Duration {
        self.decoding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1024 * 1024;

    #[test]
    fn the_estimate_prices_source_and_declared_pixels_and_admission_is_first_fit() {
        // The benchmark project's shape: a 1080x1920 source painted at 576x1024, and the 4K
        // variant's 2160x3840 source painted at 1152x2048.
        let hd = feed_bytes(Some((1080, 1920)), (576, 1024));
        let uhd = feed_bytes(Some((2160, 3840)), (1152, 2048));
        let hd_full = feed_bytes(Some((1080, 1920)), (1080, 1920));
        assert!(uhd > hd, "a 4K feed costs more than a 1080p one");
        assert!(
            hd_full > hd,
            "a larger declared size costs more from the same source"
        );
        // Every estimate sits above the peak ADR-0141 measured for its shape: (source,
        // declared, peak resident bytes of the `ffmpeg` at `-threads 1`).
        for (source, declared, peak) in [
            ((1080, 1920), (288, 512), 70_778_880),
            ((1080, 1920), (576, 1024), 93_585_408),
            ((1080, 1920), (1080, 1920), 192_315_392),
            ((1080, 1920), (2160, 3840), 655_261_696),
            ((2160, 3840), (288, 512), 157_204_480),
            ((2160, 3840), (576, 1024), 180_322_304),
            ((2160, 3840), (1080, 1920), 276_037_632),
            ((2160, 3840), (2160, 3840), 702_398_464),
            // The 16-reference H.264, the 10-bit H.264 and the ProRes 422 10-bit samples.
            ((1080, 1920), (576, 1024), 147_505_152),
            ((1080, 1920), (1080, 1920), 246_185_984),
            ((1080, 1920), (1080, 1920), 215_875_584),
            ((1080, 1920), (1080, 1920), 214_876_160),
        ] {
            let estimate = feed_bytes(Some(source), declared);
            assert!(
                estimate > peak,
                "{source:?} painted at {declared:?}: estimated {estimate}, measured {peak}"
            );
        }
        // An unprobed source is priced as 4K, never as nothing.
        assert_eq!(
            feed_bytes(None, (576, 1024)),
            feed_bytes(Some((3840, 2160)), (576, 1024))
        );
        // The benchmark project's three lanes take about a quarter of the 2 GiB budget.
        assert!(3 * hd < FEED_BUDGET / 3, "{hd}");

        // First-fit: a budget with room for the small feed and not the large one admits the
        // small one even when the large one asks first.
        let budget = hd + MIB;
        assert_eq!(first_fit(budget, 0, &[uhd, hd]), [false, true]);
        assert_eq!(first_fit(budget, 0, &[hd, hd]), [true, false]);
        // What is already held counts.
        assert_eq!(first_fit(budget, 2 * MIB, &[hd]), [false]);
        // A budget the size of two feeds is two feeds, whatever order they ask in.
        assert_eq!(
            first_fit(2 * hd, 0, &[hd, hd, hd]),
            [true, true, false],
            "it is bytes, so a third equal feed does not fit"
        );
        assert!(fits(FEED_BUDGET, FEED_BUDGET - hd, hd));
        assert!(!fits(FEED_BUDGET, FEED_BUDGET - hd + 1, hd));
        assert!(!fits(u64::MAX, u64::MAX, 1), "overflow is not a fit");
    }
}
