//! One video frame, out of an `ffmpeg` Montagent **spawns** rather than links.
//!
//! ADR-0009 ships Montagent as *"a binary, plus an `ffmpeg` the user supplies"* — `libx264`
//! is GPL and linking `libavcodec` would make the shipped binary a GPL combined work — so
//! every decoded frame crosses a subprocess boundary. That is the cost the prototype
//! measured and #6 never did, *"because its fixture had no video in it"*, and it is why
//! [#212](https://github.com/MBehtemam/Montagent/issues/212) requires the decode path its
//! own fixture: the committed project has **zero** `video` elements, and under ADR-0003's
//! asymmetry that silence is not evidence the path is unneeded.
//!
//! **Where this crate's responsibility starts.** The path to `ffmpeg` is handed in, never
//! resolved here: `montagent_core::media::tools` owns resolution *"at the first tool that
//! spawns a subprocess"*, so `probe`, `frame`, `render` and `preview` share one answer to
//! "is there an `ffmpeg` on this machine, and where". A second resolution in this crate
//! would be the second authority ADR-0011 spent itself removing.
//!
//! **Two normalisations, and only one of them is here — which is a decision no ADR has
//! ratified.** Spec #168 says *"`frame` and `render` must decode video through ADR-0023's
//! rotation-and-PAR pipeline"*, and this decode applies the rotation half and not the PAR
//! half:
//!
//! - **Rotation is applied.** The container's track-level display transform is what every
//!   mainstream player reads, `ffmpeg` applies it by default, and so a portrait phone clip
//!   arrives upright — the geometry ADR-0023 says the author was looking at. It is
//!   inherited from `ffmpeg`'s default rather than asked for, and **no fixture exercises
//!   it**: ADR-0023 records the same gap for its own rule, which is *"argued and reasoned
//!   about, not measured against real footage"*.
//! - **PAR is not applied**, and the argument is that there is nothing for it to do here:
//!   ADR-0013 settled that a source is resampled to exactly the declared `width`×`height`,
//!   so the whole source maps onto the whole box whatever its pixel aspect ratio is. PAR
//!   changes the *nominal* aspect of source pixels, never which of them survive, so it is
//!   an input to `fit`'s arithmetic — which `validate` evaluates — and not to this blit.
//!
//! That argument may well be right, and it is still a spec sentence being read narrowly by
//! a module comment. It is raised as
//! [#274](https://github.com/MBehtemam/Montagent/issues/274), because a decision that a
//! whole stage of a named pipeline is a no-op belongs in an ADR amendment rather than
//! here.

use std::path::Path;
use std::process::{Command, Stdio};

/// Which decoder `ffmpeg` is told to use, where the default one gets it wrong.
///
/// ADR-0089. `ffmpeg` picks a decoder per codec and is right almost always; the exception
/// is VP9-in-WebM carrying alpha, where the native `vp9` decoder does not surface the
/// alpha side stream and every pixel decodes opaque — silently, which is the class
/// ADR-0006 exists to prevent.
///
/// **The caller decides, and that is a dependency direction rather than a preference.**
/// Answering "does this source need `libvpx-vp9`" means knowing its codec and whether its
/// alpha is a side stream, which is `montagent_core::media::probe`'s reading — and this
/// crate cannot ask, because core depends on render and not the reverse. A second codec
/// authority spawning its own `ffprobe` here is the thing ADR-0011 spent itself removing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Decoder {
    /// Whatever `ffmpeg` selects for the codec. Right for everything but the case below.
    #[default]
    Auto,
    /// `libvpx-vp9`, which surfaces VP9's alpha side stream where the native decoder
    /// drops it.
    ///
    /// Asked for **only** where the source is VP9 *and* carries alpha, never for VP9 at
    /// large: ADR-0009 ships Montagent against *"an `ffmpeg` the user supplies"*, and an
    /// `ffmpeg` built without libvpx has no such decoder. Forcing it on every VP9 source
    /// would turn renders that work today into hard failures for those users. Narrowed to
    /// the alpha case it can only fail where the alternative was a silently wrong picture,
    /// and ADR-0006 prefers the loud error to that.
    LibVpxVp9,
}

impl Decoder {
    /// The input-side arguments this choice adds, which is nothing at all for [`Auto`].
    ///
    /// [`Auto`]: Decoder::Auto
    fn args(self) -> &'static [&'static str] {
        match self {
            Decoder::Auto => &[],
            Decoder::LibVpxVp9 => &["-c:v", "libvpx-vp9"],
        }
    }
}

/// One decoded frame, as RGBA8 at the size it was asked for.
#[derive(Clone, PartialEq, Eq)]
pub struct DecodedFrame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl std::fmt::Debug for DecodedFrame {
    /// The dimensions and the byte count, never the bytes.
    ///
    /// A frame is megabytes of RGBA, and the derived form would put all of it into any
    /// panic message that mentions one — including this module's own tests, whose whole
    /// subject is the case where the bytes are *wrong*.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DecodedFrame {{ {}x{}, {} bytes }}",
            self.width,
            self.height,
            self.rgba.len()
        )
    }
}

/// How far back the at-or-before window reaches, in milliseconds.
///
/// The window has to span the largest gap between two consecutive frames in the source,
/// because the frame visible at `at_ms` is the last one starting at or before it and the
/// window is where it gets looked for. 200 ms spans any gap down to 5 fps, which is below
/// every rate a composited source plausibly carries — and the repository's own reference
/// MP4 is the reason the bound is stated as a gap rather than as a frame period: it is
/// nominally 25 fps and carries four gaps of 51–58 ms where frames were dropped.
///
/// It is not a budget to be minimised. The window is bounded on the *read* side too (see
/// `-t` below), so widening it costs the decode of the extra frames it spans and nothing
/// else; narrowing it past a real gap turns a correct frame into a missing one.
const SEEK_WINDOW_MS: i64 = 200;

/// Decode the frame the source is **showing** at `at_ms`, resampled to `width`×`height`.
///
/// `at_ms` is an offset **into the source file**, not a timeline instant: the arithmetic
/// that turns one into the other — `source_start`, `speed`, `overrun`'s hold and loop —
/// is ADR-0020's and lives in the core, which is also what answers `query --at`'s *offset
/// into the source* with the same number. Two implementations of that arithmetic would be
/// a picture and a caption that disagree.
///
/// **The frame at an instant is the last frame starting at or before it, and `ffmpeg`'s
/// own seek answers a different question.** ADR-0096. A plain `-ss t` returns the first
/// frame whose timestamp is `>= t` — measured, across ProRes, long-GOP H.264 and a
/// fractional 30000/1001 rate alike — so every `at_ms` that does not land exactly on a
/// source frame's start used to paint the *next* frame. That is one frame early for the
/// whole of an element whose start is off the source's grid, and at the last frame there is
/// no next one, so `ffmpeg` exited cleanly having written nothing and the element silently
/// did not draw (#387, MONTAGENT-2).
///
/// **The grid is not computable from what `probe` establishes, which is why this is a
/// filter and not arithmetic.** Clamping `at_ms` down onto a frame grid needs that grid,
/// and [`Quad`](montagent_core::media::probe::Quad)'s two frame rates are not it: on the
/// repository's own reference MP4 the real frame timestamps start at 42.031 ms and step
/// 40 ms — 25 fps — while `r_frame_rate` says 50 and `avg_frame_rate` says 24.9785, and
/// four dropped frames mean no single rate describes the file at all. So the question is
/// put to the one authority that holds the real timestamps:
///
/// - `-ss` lands [`SEEK_WINDOW_MS`] **before** `at_ms`, still the input seek — `ffmpeg`
///   jumps to the nearest preceding keyframe and decodes forward rather than reading the
///   file from zero, which at the 500 ms budget is the whole budget.
/// - `-copyts` keeps the source's own timestamps, so the filter compares against source
///   time rather than against time rebased onto the seek point.
/// - `select='lte(t,<at_ms>)'` keeps every frame starting at or before the instant, and the
///   **last** one of those is the answer. A streaming filter cannot know which frame is
///   last, so the run is read to its end and the final whole frame is taken.
/// - `-t` bounds the read to the window, and it is load-bearing rather than tidy: `select`
///   places no limit on the output, so without it `ffmpeg` reads to end of file decoding
///   and discarding everything after the instant. Measured at 1080p that is the difference
///   between a 20% cost over the old single-frame seek and a 2x one, and the unbounded form
///   gets worse the longer the source is.
///
/// **The clamp is symmetric.** A source whose first frame starts *after* the instant has no
/// frame at or before it — this repository's reference MP4 begins at 42.031 ms, so an element
/// with `source_start: 0` asks for instants no frame covers — and the answer there is the
/// window's **earliest** frame. A container's own origin is not a statement by the author, and
/// refusing it would have moved the silent failure from the end of a source to the start of
/// one. The fallback searches the same window and no further, so an instant past the end still
/// finds nothing and stays an error.
///
/// **The threshold carries one microsecond of slack, and the slack is exact.** `select`
/// evaluates `t` as a float, so a frame whose start *is* the instant compares marginally
/// above it and `lte` drops it — measured on a 25 fps source at 1400 ms and a 30000/1001
/// one at 1001 ms, both of which returned the frame before. A microsecond clears that
/// margin by three orders of magnitude while staying three below the closest a real
/// source's frames come to each other, and it is written as integer digits —
/// `1.001` ms becomes `1.001001` s — so ADR-0005's integer millisecond is still never
/// routed through a float. Coarser slack is not safe: at 100 microseconds a 30000/1001
/// source over-includes the *next* frame wherever its start falls that close above a
/// whole millisecond.
///
/// The error is one sentence naming the source and what `ffmpeg` said, because the caller
/// is a report and ADR-0011's exit 70 says *"retry or report"* — neither of which is
/// possible from *"ffmpeg failed"*.
pub fn frame_at(
    ffmpeg: &Path,
    source: &str,
    decoder: Decoder,
    at_ms: i64,
    width: u32,
    height: u32,
) -> Result<DecodedFrame, String> {
    if width == 0 || height == 0 {
        return Err(format!(
            "{source}: a frame was asked for at {width}x{height}, which is no frame at all"
        ));
    }
    let at_ms = at_ms.max(0);
    let from_ms = (at_ms - SEEK_WINDOW_MS).max(0);
    // The read has to reach the instant itself, and the window is shorter than
    // `SEEK_WINDOW_MS` for any instant inside the first window's worth of the source.
    let read_ms = at_ms - from_ms + 1;

    // Milliseconds to `ffmpeg`'s decimal seconds, in the string rather than through a
    // float: ADR-0005 stores every time as an integer millisecond because float seconds
    // failed in practice, and re-introducing one at the boundary would put the error back
    // in the one place the whole surface treats as authoritative.
    let seconds = format!("{}.{:03}", at_ms / 1000, at_ms % 1000);
    let from = format!("{}.{:03}", from_ms / 1000, from_ms % 1000);
    let read = format!("{}.{:03}", read_ms / 1000, read_ms % 1000);
    // The instant plus one microsecond, in the same integer arithmetic — the six decimal
    // places *are* the slack, so there is no float here either.
    let threshold = format!("{}.{:03}001", at_ms / 1000, at_ms % 1000);

    let expected = (width as usize) * (height as usize) * 4;
    let scale = format!("scale={width}:{height}");

    // One spawn of `ffmpeg` over the window, with whichever output-side arguments pick the
    // frame wanted out of it.
    let over_window = |output_args: &[&str]| -> Result<std::process::Output, String> {
        Command::new(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error"])
            // Before `-i`, because a decoder choice is an option about the *input* — after
            // it, `ffmpeg` reads it as an encoder for the output and the decode is
            // unchanged.
            .args(decoder.args())
            .args(["-ss", &from, "-copyts", "-t", &read, "-i", source])
            .args(output_args)
            .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("{source}: {} could not be run: {e}", ffmpeg.display()))
    };

    // **A spawn that failed is a refusal, never an empty answer.** Its stdout is empty for the
    // same reason an instant with no frame at or before it is empty, and reading the one as
    // the other is how an `ffmpeg` that rejected an argument painted a frame up to a window
    // early with `0 errors` — ffmpeg 9 removed `-vsync`, the `select` run exited 8, and the
    // fallback below answered every instant. ADR-0113.
    let succeeded = |output: std::process::Output| -> Result<std::process::Output, String> {
        if output.status.success() {
            return Ok(output);
        }
        let said = String::from_utf8_lossy(&output.stderr);
        let said = said.trim();
        Err(format!(
            "{source}: {} could not read the frame at {seconds}s: it exited with {}{}",
            ffmpeg.display(),
            output.status,
            if said.is_empty() {
                String::new()
            } else {
                format!(" — {said}")
            }
        ))
    };

    let filter = format!("select='lte(t\\,{threshold})',{scale}");
    // `-fps_mode passthrough`, from the one place the floor's arguments are spelled
    // (ADR-0115): every selected frame, none duplicated to meet an output rate.
    let at_or_before = succeeded(over_window(&[
        "-vf",
        &filter,
        crate::floor::FPS_PASSTHROUGH[0],
        crate::floor::FPS_PASSTHROUGH[1],
    ])?)?;

    // **The frame at or before the instant, and where there is none, the source's first.**
    // A source whose first frame starts *after* the instant has no frame at or before it,
    // and the clamp is symmetric for the same reason it exists at the other end: a
    // container's own origin is not the author's mistake. This repository's reference MP4
    // begins at 42.031 ms, so an element with `source_start: 0` asks for instants no frame
    // covers — and before this fallback existed those instants failed, which is the defect
    // the clamp was written to remove rather than to move to the other end of the file.
    //
    // `-frames:v 1` over the same window, so the answer is the source's earliest frame
    // *near the instant* and never a search of the whole file: an instant past the end of
    // the source still finds nothing here and stays an error.
    //
    // Only a run that *succeeded* and wrote no frame reaches the fallback: that is the one
    // outcome that means *"no frame at or before"*.
    let (output, at_or_after) = if at_or_before.stdout.len() >= expected {
        (at_or_before, false)
    } else {
        (
            succeeded(over_window(&["-frames:v", "1", "-vf", &scale])?)?,
            true,
        )
    };

    let whole = output.stdout.len() / expected;
    if whole == 0 || output.stdout.len() % expected != 0 {
        let said = String::from_utf8_lossy(&output.stderr);
        let said = said.trim();
        // The empty-output case is the one an author can act on, so it says what it means
        // rather than reporting a byte count. After ADR-0096's clamp it no longer means
        // *"the instant is past the last frame"* — that instant now paints the last frame —
        // but *"there is no frame within a window of it in either direction"*, which is a
        // source whose frames stop more than `SEEK_WINDOW_MS` before the instant the
        // document asked for.
        return Err(if output.stdout.is_empty() {
            format!(
                "{source}: no frame at or before {seconds}s{}",
                if said.is_empty() {
                    format!(
                        " — the source carries no frame within {SEEK_WINDOW_MS} ms of it in \
                         either direction, so it does not cover the range the document \
                         declares"
                    )
                } else {
                    format!(" — {said}")
                }
            )
        } else {
            format!(
                "{source}: ffmpeg wrote {} bytes for {width}x{height} RGBA frames, which is \
                 not a whole number of the {expected} each one needs{}",
                output.stdout.len(),
                if said.is_empty() {
                    String::new()
                } else {
                    format!(" — {said}")
                }
            )
        });
    }

    // The last whole frame of the `select` run, because it kept every frame starting at or
    // before the instant in decode order — or the only frame of the fallback, which asked
    // for one.
    let taken = if at_or_after {
        output.stdout[..expected].to_vec()
    } else {
        output.stdout[(whole - 1) * expected..].to_vec()
    };
    Ok(DecodedFrame {
        rgba: taken,
        width,
        height,
    })
}

/// The largest term of a `speed` ratio [`frames_from`] samples exactly — see its bound.
const EXACT_SPEED_TERM: i128 = 1_000_000;

/// The decoder threads one **feed** runs with: the `-threads` every run of frames passes.
///
/// ADR-0141. A feed is one `ffmpeg` per visible video element, so its memory is paid once
/// per element on screen, and `ffmpeg`'s default — about ten decoder threads on the M1 Pro
/// — roughly doubles a feed's peak resident memory for a rate the painter cannot use: at
/// one thread a feed still decodes several hundred frames a second, against the 30 the
/// timeline asks for. [`frame_at`] keeps the default, because there one process runs alone.
///
/// TODO(measure): pinned at 1 from the memory measurements in ADR-0141. The rate half of
/// "the lowest that leaves clear headroom over the rate the painter needs" is a wall-clock
/// measurement, taken with the before/after timing; raise this to 2 only if it says so.
pub const FEED_THREADS: u32 = 1;

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// How a run moves through its source: one frame per timeline frame at the project's `fps`,
/// each `speed / fps` seconds of source further on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    /// The project's frame rate, which is an integer (ADR-0005).
    pub fps: i64,
    /// The element's `speed` as the exact rational `(numerator, denominator)` the document's
    /// decimal is — never a float, because the render's offset into the source is not one.
    pub speed: (i128, i128),
}

/// The instant a run's arithmetic is measured from: timeline millisecond `timeline_ms`
/// plays source millisecond `source_ms`.
///
/// For a `video` element that is its own `start` and `source_start` — or, past a loop
/// wrap, the instant the current pass began and `source_start` again. Measuring from
/// there rather than from wherever a run happens to open is what keeps a run's offsets
/// the render's: `round_half_up` does not distribute over a sum, so `o0 + advance(b)` is
/// not `advance(a + b)` at a `speed` like 0.5, and a run measured from its own first frame
/// would disagree with the render by a millisecond on about half its frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Origin {
    pub timeline_ms: i64,
    pub source_ms: i64,
}

/// Whether a run can sample `pace` at all, and exactly: a positive rate and a positive
/// `speed` whose terms, in lowest terms, are at most a million (ADR-0127 §2). The same test
/// [`frames_at`] refuses on, for a caller that would rather not open a run it will refuse.
pub fn exact_pace(pace: Pace) -> bool {
    let (num, den) = pace.speed;
    if pace.fps <= 0 || num <= 0 || den <= 0 {
        return false;
    }
    let common = gcd(num, den);
    num / common <= EXACT_SPEED_TERM && den / common <= EXACT_SPEED_TERM
}

/// `⌊n × 1000 / fps⌋`: the millisecond the render paints timeline frame `n` at (ADR-0035,
/// ADR-0077).
fn instant(n: i64, fps: i64) -> i64 {
    ((i128::from(n) * 1000) / i128::from(fps)) as i64
}

/// `source_ms + round_half_up((instant(n) − timeline_ms) × speed)`: the offset into the
/// source a run anchored at `origin` is showing at timeline frame `n`.
///
/// **The one function a run's frames are defined by**, and public so that a caller deciding
/// whether a run's next frame is the one it wants asks this rather than a copy of it: the
/// filter [`frames_at`] builds is this function's inverse, frame for frame. It is
/// `montagent_core::exact::source_advance`'s arithmetic on the render's instant, which is
/// what ADR-0127 holds the run to.
pub fn offset_at(origin: Origin, pace: Pace, n: i64) -> i64 {
    let (num, den) = pace.speed;
    let elapsed = i128::from(instant(n, pace.fps) - origin.timeline_ms);
    // `floor((2 × elapsed × num + den) / 2den)`, the half-up tie-break folded into one floored
    // division as `exact::round_half_up` folds it.
    let doubled = 2 * elapsed * num + den;
    let divisor = 2 * den;
    let quotient = doubled / divisor;
    let advance = if doubled % divisor != 0 && doubled < 0 {
        quotient - 1
    } else {
        quotient
    };
    origin.source_ms + advance as i64
}

/// A **run** of frames out of one `ffmpeg`, starting at `from_ms` into the source: frame `n`
/// of the run is the frame the renderer paints at timeline frame `n` of an element that
/// starts there and plays at `pace`.
///
/// [`frame_at`]'s sibling, and the reason it exists is a measurement rather than a
/// preference: on ADR-0088's 140-frame forcing case, one spawn per frame costs ~13 s and
/// one spawn for the whole run costs ~0.4 s at full 1080p. `measure`'s keyed-alpha
/// coverage reading is defined as a *series* — ADR-0088 is explicit that a single frame
/// answers *"did this key at all"* while the series answers *"did this key **stay**"* — so
/// a series costing 30x what it needs to would be a reading nobody runs, which is the same
/// as not having one.
///
/// **Streamed, one frame at a time, and that is not an optimisation either.** The same run
/// materialised as a `Vec` is 1.16 GB of RGBA at 1080p, which is not a buffer a reading
/// about *fractions* may demand of the machine it runs on.
///
/// **The frames are the renderer's, to the millisecond, and a rate could not say which those
/// are.** ADR-0127. The render paints timeline frame `n` at `⌊n × 1000 / fps⌋` ms, moves
/// `round_half_up(elapsed × speed)` ms into the source, and asks [`frame_at`] for the last
/// frame starting at or before that. This run used to be `-ss from -vf fps=fps×speed`, and
/// measured against that rule it returned a frame the render does not paint on 501 of 504
/// runs, always later in the source, by three separate mechanisms:
///
/// - `-ss` without `-copyts` is ADR-0096's seek, so a `from_ms` off the source's grid
///   started the whole run on the *next* frame, and it rebased the source onto zero, so a
///   file whose frames start at 42.031 ms — this repository's reference MP4 — was read a
///   frame ahead from its very first instant;
/// - `fps=` rounds to the **nearest** tick by default, so wherever the two rates differ it
///   took the frame about to start rather than the one showing;
/// - and the rate was `fps × speed` where a retimed element moves `speed / fps` seconds
///   through its source per frame, so a `speed: 2` series sampled four times too densely
///   and covered a quarter of the range it named.
///
/// So the run asks the renderer's question in the renderer's arithmetic, with no rate in it.
/// [`frames_at`] says how; this is its case for a run anchored at timeline frame 0.
///
/// The run may end a frame later than the element does; the caller counts the frames its
/// range shows and stops there.
pub fn frames_from(
    ffmpeg: &Path,
    source: &str,
    decoder: Decoder,
    from_ms: i64,
    pace: Pace,
    width: u32,
    height: u32,
) -> Result<Frames, String> {
    frames_at(
        ffmpeg,
        source,
        decoder,
        Origin {
            timeline_ms: 0,
            source_ms: from_ms.max(0),
        },
        0,
        pace,
        width,
        height,
    )
}

/// A run of frames anchored at `origin`, starting at timeline frame `first`: the run's
/// `k`-th frame is the last source frame starting at or before
/// [`offset_at`]`(origin, pace, first + k)`. This is a **feed** (ADR-0141) when `render`
/// opens it for a `video` element at the first frame that paints it.
///
/// [`frames_from`] is the case `origin = (0, from_ms)`, `first = 0`. The general case is what
/// lets a feed open partway through an element — a partial render, a re-entry, the pass
/// after a loop wrap — and still deliver exactly the frames the render's own arithmetic
/// names, rather than frames measured from wherever it opened (see [`Origin`]).
///
/// - `-ss` lands [`SEEK_WINDOW_MS`] before the first frame's offset with `-copyts`,
///   [`frame_at`]'s window, so the frame already showing there is decoded and compared in
///   source time.
/// - `settb` puts every timestamp on the microsecond, and `setpts` rewrites each frame's to
///   the first **timeline** millisecond the render shows it at — `ceil(p − 1 µs)` in source
///   milliseconds is [`frame_at`]'s at-or-before with its slack, and the inverse of
///   `source_ms + round_half_up((t − timeline_ms) × speed)` is
///   `timeline_ms + ceil((2(q − source_ms) − 1) × den / 2 × num)` — every operand an integer,
///   so the float the expression evaluator works in is exact.
/// - A second `settb` moves that millisecond onto a clock of `1 / (1000 × fps)` seconds, where
///   it is `t × fps` exactly, and a second `setpts` subtracts `first × 1000`. Timeline frame
///   `first + k` is then tick `k` of the output, with no fractional start time for `fps=` to
///   round: frame `n`'s tick, `n × 1000 / fps` ms, is not a whole millisecond or microsecond
///   at most rates, and a `start_time` spelled in either would put a frame that starts
///   exactly on a tick onto the next one.
/// - `fps=<fps>:round=up:start_time=0` then paints each tick with the last frame whose
///   timeline start is at or before it, which on integer milliseconds is exactly *"at or
///   before `⌊n × 1000 / fps⌋`"*. Before the first frame, `start_time` holds the earliest
///   one, [`frame_at`]'s symmetric clamp.
/// - `-threads` is [`FEED_THREADS`] (ADR-0141): a feed runs beside others, one per visible
///   element, and its memory is what the feed budget counts.
#[allow(clippy::too_many_arguments)]
pub fn frames_at(
    ffmpeg: &Path,
    source: &str,
    decoder: Decoder,
    origin: Origin,
    first: i64,
    pace: Pace,
    width: u32,
    height: u32,
) -> Result<Frames, String> {
    let Pace { fps, speed } = pace;
    if width == 0 || height == 0 {
        return Err(format!(
            "{source}: frames were asked for at {width}x{height}, which is no frame at all"
        ));
    }
    if fps <= 0 {
        return Err(format!(
            "{source}: a run of frames needs a positive frame rate, not {fps}"
        ));
    }
    let (num, den) = speed;
    if num <= 0 || den <= 0 {
        return Err(format!(
            "{source}: a run of frames needs a positive speed, not {num}/{den}"
        ));
    }
    // In lowest terms, and small enough that the filter's arithmetic stays exact: the
    // expression evaluator is a double, so its largest product, `(2q − 1) × den`, must stay
    // under 2^53. With each term of the ratio at most a million — six decimal places of
    // `speed` — that holds for any millisecond `q` under 4.5 × 10^9, a source of 52 days.
    let common = gcd(num, den);
    let (num, den) = (num / common, den / common);
    if num > EXACT_SPEED_TERM || den > EXACT_SPEED_TERM {
        return Err(format!(
            "{source}: a speed of {num}/{den} carries more precision than a run of frames can \
             sample exactly; six decimal places is the most it reads"
        ));
    }
    let pace = Pace {
        fps,
        speed: (num, den),
    };
    let first = first.max(0);

    let from_ms = offset_at(origin, pace, first).max(0);
    let window_ms = (from_ms - SEEK_WINDOW_MS).max(0);
    // [`frame_at`]'s conversion, for [`frame_at`]'s reason: integer milliseconds to
    // `ffmpeg`'s decimal seconds through the string, never through a float.
    let window = format!("{}.{:03}", window_ms / 1000, window_ms % 1000);

    let Origin {
        timeline_ms,
        source_ms,
    } = origin;
    let filter = format!(
        "settb=1/1000000,\
         setpts='(ceil((2*(ceil((PTS-1)/1000)-{source_ms})-1)*{den}/(2*{num}))+{timeline_ms})*1000',\
         settb=1/{tick_base},\
         setpts='PTS-{first_tick}',\
         fps={fps}:round=up:start_time=0,\
         scale={width}:{height}",
        tick_base = 1000 * i128::from(fps),
        first_tick = 1000 * i128::from(first),
    );
    let threads = FEED_THREADS.to_string();
    let mut child = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error"])
        // [`frame_at`]'s placement, for [`frame_at`]'s reason: an input option goes before
        // the input it is about. `-threads` too, which before `-i` is the decoder's.
        .args(["-threads", &threads])
        .args(decoder.args())
        .args(["-ss", &window, "-copyts", "-i", source, "-vf", &filter])
        // Every frame the filter paints, none duplicated or dropped again by the muxer
        // (ADR-0115's spelling): the filter's output *is* the timeline's grid.
        .args(crate::floor::FPS_PASSTHROUGH)
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{source}: {} could not be run: {e}", ffmpeg.display()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("{source}: ffmpeg's output could not be read"))?;
    // Drained on its own thread, because a run is read a frame at a time and `ffmpeg`
    // blocked on a full stderr pipe would stop writing the frames the reader is waiting for.
    let stderr = child.stderr.take().map(|mut stderr| {
        std::thread::spawn(move || {
            use std::io::Read;
            let mut said = String::new();
            let _ = stderr.read_to_string(&mut said);
            said
        })
    });

    Ok(Frames {
        child,
        stdout,
        stderr,
        said: None,
        source: source.to_string(),
        width,
        height,
    })
}

/// A run of decoded frames, still arriving.
///
/// Not an `Iterator`, because a frame that could not be read is a sentence rather than an
/// end of stream, and `Option<Result<_>>` puts the two states a caller must tell apart into
/// the shape most likely to collapse them. [`Frames::next_frame`] answers with all three:
/// a frame, the end of the run, or what went wrong.
pub struct Frames {
    child: std::process::Child,
    stdout: std::process::ChildStdout,
    stderr: Option<std::thread::JoinHandle<String>>,
    /// What `ffmpeg` said on stderr, once the run has ended and the drain was joined.
    said: Option<String>,
    source: String,
    width: u32,
    height: u32,
}

impl Frames {
    /// The next frame, or `None` where the run has ended.
    ///
    /// **A short run is an end, not an error.** Asking past the end of a source is how a
    /// caller finds out where the end is; whether the source is shorter than the document
    /// says it is is `validate`'s finding (`E-SOURCE-OVERRUN`) and not this type's to
    /// re-derive. A run that ends mid-frame is a genuine failure, because those bytes are not
    /// a picture.
    ///
    /// **So is a run whose `ffmpeg` failed**, however many frames it wrote first. An `ffmpeg`
    /// that rejected an argument writes nothing and exits non-zero, and reading that as an end
    /// is a series of no frames about a source that has them — ADR-0113.
    pub fn next_frame(&mut self) -> Result<Option<DecodedFrame>, String> {
        use std::io::Read;

        let stride = (self.width as usize) * (self.height as usize) * 4;
        let mut rgba = vec![0u8; stride];
        let mut filled = 0;
        while filled < stride {
            match self.stdout.read(&mut rgba[filled..]) {
                Ok(0) => break,
                Ok(read) => filled += read,
                Err(e) => return Err(format!("{}: its frames stopped arriving: {e}", self.source)),
            }
        }
        if filled < stride {
            self.ended()?;
        }
        if filled == 0 {
            return Ok(None);
        }
        if filled < stride {
            return Err(format!(
                "{}: ffmpeg wrote {filled} bytes for a {}x{} RGBA frame, which needs {stride}",
                self.source, self.width, self.height
            ));
        }
        Ok(Some(DecodedFrame {
            rgba,
            width: self.width,
            height: self.height,
        }))
    }
}

impl Frames {
    /// The run's output has ended: wait for `ffmpeg` and refuse if it failed.
    fn ended(&mut self) -> Result<(), String> {
        let status = self
            .child
            .wait()
            .map_err(|e| format!("{}: ffmpeg could not be waited for: {e}", self.source))?;
        let said = self.stderr();
        if status.success() {
            return Ok(());
        }
        let said = said.trim();
        Err(format!(
            "{}: ffmpeg stopped reading its frames: it exited with {status}{}",
            self.source,
            if said.is_empty() {
                String::new()
            } else {
                format!(" — {said}")
            }
        ))
    }
}

impl Frames {
    /// What `ffmpeg` wrote on stderr. Complete only once the run has ended — before that it
    /// waits for the drain thread, which finishes when `ffmpeg` closes the pipe.
    ///
    /// For a caller that must say *why* a run that ended cleanly ended where it did
    /// (ADR-0141's early-end refusal): a clean exit is not a reason, and what `ffmpeg` said
    /// on the way out is the nearest thing to one.
    pub fn stderr(&mut self) -> String {
        if self.said.is_none() {
            self.said = Some(
                self.stderr
                    .take()
                    .and_then(|thread| thread.join().ok())
                    .unwrap_or_default(),
            );
        }
        self.said.clone().unwrap_or_default()
    }
}

impl Drop for Frames {
    /// The child is killed rather than waited on, because a caller that stopped reading
    /// stopped for a reason and the rest of a 1080p run is gigabytes it no longer wants.
    /// `ffmpeg` would otherwise sit blocked on a pipe nobody is draining.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zero_sized_frame_is_refused_before_anything_is_spawned() {
        let e = frame_at(
            Path::new("/nowhere/ffmpeg"),
            "clip.mov",
            Decoder::Auto,
            0,
            0,
            100,
        )
        .expect_err("no frame has zero width");
        assert!(e.contains("clip.mov"), "{e}");
    }

    #[test]
    fn the_default_decoder_adds_no_arguments_at_all() {
        // The whole safety of the conditional rests on this: every source that is not
        // VP9-with-alpha must reach `ffmpeg` with exactly the command line it reached
        // before ADR-0089, or the fix has broken the cases that already worked.
        assert!(Decoder::Auto.args().is_empty());
        assert_eq!(Decoder::default(), Decoder::Auto);
        assert_eq!(Decoder::LibVpxVp9.args(), &["-c:v", "libvpx-vp9"]);
    }

    #[test]
    fn an_unrunnable_ffmpeg_names_the_path_it_tried() {
        let e = frame_at(
            Path::new("/nowhere/ffmpeg"),
            "clip.mov",
            Decoder::Auto,
            0,
            16,
            16,
        )
        .expect_err("nothing to run");
        assert!(e.contains("/nowhere/ffmpeg"), "{e}");
        assert!(e.contains("clip.mov"), "{e}");
    }
}
