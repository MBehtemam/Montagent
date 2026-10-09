//! **The whole-video falsification** (#219): the committed fixture rendered end to end and
//! compared against `reference/en-halloween-decorating.mp4`, the already-published short.
//!
//! # What this reaches that `tests/reference_frames.rs` structurally cannot
//!
//! That file is the *frame* falsification (#213), and a frame is a picture: it can say the
//! navy card is in the right place and it cannot say anything at all about how long the
//! video is, how many frames it has, or when a narration line begins. Spec #168 names this
//! file's comparison as the only test of #216 — the four narration elements at
//! `speed: 0.645` are audible facts, and no still frame carries one.
//!
//! So nothing here is gated on pixels. The axes are time and sound.
//!
//! # The container is the wrong axis, and on this very file the two answers differ
//!
//! ADR-0011's probe quad exists because a single scalar named `duration` is a lie on this
//! MP4: the **video stream** runs 65.216016 s and the **container** declares 65.258667 s,
//! 42 ms apart — more than one frame at the project's 25 fps. The project declares
//! `duration: 65216`, which is the stream to the millisecond and the container to nothing.
//! [`the_container_is_the_wrong_axis_and_reading_it_would_move_the_answer`] asserts that
//! distinction directly, so the comparison below cannot quietly start reading the other
//! number.
//!
//! The quad's fourth member earns its keep too, in a place nobody planned: the published
//! narration sits **43 ms later** than the document says, and that is the reference's own
//! video-stream `start_time` of 42.031 ms. Remove it and all 39 narration boundaries agree
//! with this render to inside one frame; leave it in and the median disagreement is wider
//! than a frame. See [`the_narration_lands_where_the_published_video_puts_it`].
//!
//! # Every divergence found is a number this file prints, not a threshold it widens
//!
//! Three real divergences came out of this comparison and none is absorbed:
//!
//! - **Two frames.** The reference carries 1629 where the project's 25 fps grid asks for
//!   1631. The reference is not on that grid at all — `r_frame_rate` is `50/1` and
//!   `avg_frame_rate` is `1390080/55651` ≈ 24.9785 — so the counts are derived separately
//!   and asserted exactly, with no tolerance between them.
//! - **24 ms.** This render's video stream is 65.240 s against the reference's 65.216 s,
//!   which is exactly the last of 1631 frames being displayed for its own 1/25 s.
//! - **≈3 dB.** The published mix is mono at 24 kHz and this one is stereo at 48 kHz, so
//!   peak levels differ by a few dB throughout. Placement is what is asserted; level is
//!   what is reported.
//!
//! # The picture is not compared here at all, and that is the point
//!
//! Known causes put a difference into the frames of this fixture and none of them is a
//! defect in this build: the fixture is typeset in **Open Runde** and the published video
//! in **SF Pro Rounded** ([#143](https://github.com/MBehtemam/Montagent/issues/143),
//! ADR-0057 — *"no font claims formal metric compatibility"*, quantified by
//! [#186](https://github.com/MBehtemam/Montagent/issues/186), settled by ADR-0085 as a
//! permanent divergence); and the photograph carries an
//! unexplained displacement that grows with the Ken Burns ramp, 8 × 4 px by 14 000 ms
//! ([#299](https://github.com/MBehtemam/Montagent/issues/299)). A whole-frame threshold
//! loose enough to absorb either would be loose enough to absorb anything, which is the
//! *"completed, looked plausible, was wrong"* failure ADR-0010 records this project having
//! had twice.
//!
//! The larger of the two causes this file was written against is gone:
//! [#276](https://github.com/MBehtemam/Montagent/issues/276) corrected the fixture's Ken
//! Burns pivot from `top-left` to the centre the published move actually uses, which was
//! 32 × 56 px at that same instant. The argument is unchanged — it never rested on the
//! size of any one divergence — and #299 is what is left of it.
//!
//! So the picture belongs to `reference_frames.rs`, which gates it where it can be
//! believed — region-masked, at two instants, with the substitution measured beside the
//! gate — and #219 says in as many words that this file is *"not redundant with ticket
//! 26"*. A second, looser picture comparison here would be exactly that redundancy, and a
//! whole-frame number asserted against the two open defects would turn *fixing* them into
//! a test failure.
//!
//! Run with `--nocapture` to see every number this file measured.

use std::path::Path;

use montagent_core::media::probe::Quad;
use montagent_core::media::session::Session;
use montagent_core::media::tools;
use montagent_core::media::{Rational, Source};
use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{Ask, render};
use serde_json::Value;

mod common;
use common::media::{Edge, Span, audio_stream, peak_db, silence_marks, speech_spans, video_stream};
use common::{
    canonical, document, elements, fixture_dir, fixture_project, has_ffprobe, tempdir,
    write_project,
};

// ---------------------------------------------------------------------------
// What the document declares
// ---------------------------------------------------------------------------

const DECLARED_DURATION_MS: i64 = 65_216;
const FPS: i64 = 25;

/// One frame at the project's rate — the natural unit for every time tolerance here,
/// because a video cannot express a disagreement smaller than one.
const FRAME_MS: i64 = 1000 / FPS;

/// The fixture's four stretched narration elements (ADR-0020, #216).
const SPEED: f64 = 0.645;

/// How far a measured speech span may sit from the span its source and `speed` predict.
///
/// Not a fitted number: the prediction is `source speech span ÷ 0.645`, and the worst of
/// the eight measurements below (four in the published video, four in this render) is
/// **0.94 %** off it. Three percent is that with room for `silencedetect` to place an edge
/// a millisecond differently on another `ffmpeg` build — and it is nowhere near loose
/// enough to stop being a test of `speed`: an element rendered at `speed: 1` would measure
/// **35 %** short of the prediction, which is eleven times the band — demonstrated by
/// [`the_span_and_placement_predicates_reject_what_they_are_meant_to`] rather than asserted
/// here.
const SPAN_TOLERANCE: f64 = 0.03;

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

fn reference() -> std::path::PathBuf {
    fixture_dir().join("reference/en-halloween-decorating.mp4")
}

/// The committed fixture, as JSON.
fn fixture() -> Value {
    document(&fixture_project())
}

/// That the two constants above still restate the committed document.
///
/// They are constants because most of what follows reads better as arithmetic on literals
/// than on a parsed value — `ceil(65216 x 25 / 1000)` says what it is. But a literal that
/// quietly stopped matching the fixture would turn every one of those assertions into a
/// comparison against a number nothing declares, and the published video would then be
/// being held to a duration no project asks for. So the document is read and the two are
/// held equal before anything else runs.
#[track_caller]
fn the_constants_still_restate_the_document() {
    let document = fixture();
    assert_eq!(
        document["duration"].as_i64(),
        Some(DECLARED_DURATION_MS),
        "the fixture's declared duration has moved and this file's constant has not"
    );
    assert_eq!(
        document["fps"].as_i64(),
        Some(FPS),
        "the fixture's frame rate has moved and this file's constant has not"
    );
}

/// The published video's quad, through the one authority on media numbers (ADR-0011).
///
/// Built with no sidecar (ADR-0069): the numbers must come from a real `ffprobe` on every
/// run, not from a cache that outlived the last one.
fn reference_quad() -> Quad {
    let tools = tools::resolve().expect("ffprobe");
    let mut session = Session::with(tools, Box::new(montagent_core::media::probe::ProcessRunner));
    let outcome = session
        .probe(&Source::resolve(
            "reference/en-halloween-decorating.mp4",
            &fixture_dir(),
        ))
        .expect("ffprobe runs");
    outcome.probe().expect("the published MP4 probes").quad
}

/// The fixture's `audio` elements whose `speed` is not 1, with everything needed to
/// predict how long each should sound for.
struct Stretched {
    id: String,
    start: i64,
    end: i64,
    source: String,
    source_start: i64,
    source_end: i64,
    speed: f64,
}

fn stretched_narration() -> Vec<Stretched> {
    let document = fixture();
    let mut found: Vec<Stretched> = elements(&document)
        .filter(|e| e["type"] == "audio")
        .filter(|e| e["speed"].as_f64().is_some_and(|speed| speed != 1.0))
        .map(|e| Stretched {
            id: e["id"].as_str().expect("an id").to_string(),
            start: e["start"].as_i64().expect("a start"),
            end: e["end"].as_i64().expect("an end"),
            source: e["source"].as_str().expect("a source").to_string(),
            source_start: e["source_start"].as_i64().expect("a source_start"),
            source_end: e["source_end"].as_i64().expect("a source_end"),
            speed: e["speed"].as_f64().expect("a speed"),
        })
        .collect();
    found.sort_by_key(|e| e.start);
    found
}

/// The one span of sound inside `[start, end)`, refusing to continue if the window holds
/// anything other than exactly one — two would mean the line broke in the middle and the
/// arithmetic below would be measuring half of it.
#[track_caller]
fn the_one_span_within(spans: &[Span], start: i64, end: i64, what: &str) -> Span {
    let (start, end) = (start as f64, end as f64);
    let hits: Vec<Span> = spans
        .iter()
        .copied()
        .filter(|span| span.from_ms < end && span.to_ms > start)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "{what} over {start}..{end} ms holds {} spans of sound, not one: {hits:?}",
        hits.len()
    );
    hits[0]
}

// ---------------------------------------------------------------------------
// The axis, asserted before anything is compared along it
// ---------------------------------------------------------------------------

/// ADR-0011's distinction, on the one file the ADR derived it from — and the reason
/// [`the_whole_fixture_is_compared_against_the_published_video`] may read only one of the
/// two numbers.
///
/// Cheap enough to run in a debug build, and deliberately separate from the render: if the
/// axis is wrong, every comparison downstream of it is wrong, and finding that out should
/// not cost half a minute of encoding.
#[test]
fn the_container_is_the_wrong_axis_and_reading_it_would_move_the_answer() {
    if !has_ffprobe() {
        return;
    }
    the_constants_still_restate_the_document();
    let quad = reference_quad();

    // The document's own number is the video stream's, exactly. No tolerance: these are
    // two committed files and neither can drift.
    assert_eq!(quad.video_stream_ms, Some(DECLARED_DURATION_MS));
    assert!(
        quad.durations_disagree(),
        "the whole reason this test exists is that these two numbers differ on this file"
    );

    let container = quad.container_ms.expect("a container duration");
    let error_if_read = (container - DECLARED_DURATION_MS).abs();
    println!(
        "AXIS  video stream {} ms == the declared duration; container {container} ms, \
         {error_if_read} ms away — {} frame(s) at {FPS} fps.",
        DECLARED_DURATION_MS,
        error_if_read / FRAME_MS,
    );
    assert!(
        error_if_read > FRAME_MS,
        "reading the container instead of the stream would cost {error_if_read} ms, which \
         is inside one frame — the distinction this file rests on would be unobservable \
         and every assertion below would pass on the wrong number"
    );
}

// ---------------------------------------------------------------------------
// The whole video
// ---------------------------------------------------------------------------

/// #219, end to end. One render, then every comparison a still frame cannot make.
///
/// **Release builds only, and that is not the same as rarely.** 1631 frames of 1080×1920
/// through a debug build is minutes, and unlike `render.rs`'s fixture test there is no
/// useful partial: a prefix of the video cannot be compared against the published one's
/// duration, frame count, or last narration line. A debug run says what it skipped rather
/// than pretending to a weaker version of it.
///
/// Spec #168 calls this the only test of #216, so "skips under `cargo test`" would be a
/// real hole — `.github/workflows/ci.yml` runs `cargo test --workspace --all-targets
/// --release` on the five tier-1 test targets (ADR-0188), which is where this one fires.
#[test]
fn the_whole_fixture_is_compared_against_the_published_video() {
    if !has_ffprobe() {
        return;
    }
    if cfg!(debug_assertions) {
        eprintln!(
            "skipping: the whole-video falsification renders all 1631 frames of the fixture \
             and needs an optimised build — `cargo test --release`. A partial render has no \
             duration, no frame count and no last narration line to compare."
        );
        return;
    }

    the_constants_still_restate_the_document();
    let dir = tempdir(line!());
    let output = dir.join("fixture.mp4");
    let answer = render(
        &fixture_project(),
        &Ask {
            output: Some(output.clone()),
            ..Default::default()
        },
        &mut |_| {},
    );
    let json = answer.to_json();
    assert_eq!(
        answer.report().exit_code(),
        ExitCode::Ok,
        "the fixture must render end to end before anything can be compared: {json}"
    );

    let published = reference_quad();
    duration_is_compared_against_the_video_stream(&json["render"], &output, &published);
    the_frame_counts_are_compared_and_their_difference_is_accounted_for(&output, &published);
    the_narration_lands_where_the_published_video_puts_it(&output, &published);
    the_four_stretched_lines_are_stretched_in_the_published_video_too(&output, &published);
}

/// #219: *"Duration is compared against the **video stream**, not the container."*
///
/// Three numbers: Montagent's answer, the published video's stream, and the file this
/// render wrote.
///
/// **The first two are held equal exactly** — that is the falsification, and it carries no
/// tolerance at all. The document declares 65 216 ms and the published video's stream runs
/// 65 216 ms.
///
/// The third is 24 ms longer, and the band it is checked inside is a *proof's conclusion*
/// rather than a chosen slack. The written stream holds `ceil(65216 × 25 / 1000) = 1631`
/// frames each displayed for 1/25 s, so it occupies `1631 × 40 = 65 240` ms; in general
/// `ceil(d × fps / 1000) × 1000 / fps - d` lies in `[0, one frame)` for every `d`, because
/// that is what a ceiling to a grid does. So `0..FRAME_MS` is the whole range the
/// arithmetic permits and there is nothing to widen: a render 40 ms long would need a
/// different frame count, which the check below asserts exactly.
fn duration_is_compared_against_the_video_stream(video: &Value, output: &Path, published: &Quad) {
    assert_eq!(
        video["duration_ms"], DECLARED_DURATION_MS,
        "render's own answer"
    );
    assert_eq!(
        published.video_stream_ms,
        Some(DECLARED_DURATION_MS),
        "the published video's stream is the document's duration, to the millisecond"
    );

    let ours = video_stream(output);
    let frames = ours.frames.expect("the written file's frame count");
    let occupied = frames * 1000 / FPS;
    // Both sides of this come off the same file, so it cannot catch a wrong render — what
    // it establishes is that the muxer agrees with its own frame count, which is what makes
    // the 24 ms below a display-period fact rather than a truncated encode.
    assert_eq!(
        ours.duration_ms,
        Some(occupied),
        "{frames} frames at {FPS} fps occupy {occupied} ms and the written stream says so"
    );

    let divergence = occupied - DECLARED_DURATION_MS;
    println!(
        "DURATION  published stream {} ms; this render's stream {occupied} ms; \
         Montagent's answer {} ms. Divergence {divergence} ms = the last of {frames} frames \
         held for its own 1/{FPS} s. Container, unread: published {} ms, ours {} ms.",
        DECLARED_DURATION_MS,
        video["duration_ms"],
        published.container_ms.expect("a container duration"),
        ours.duration_ms.unwrap_or_default(),
    );
    assert!(
        (0..FRAME_MS).contains(&divergence),
        "a whole video's stream cannot be off by a frame or more and still be the same cut: \
         {divergence} ms"
    );
}

/// #219: *"Frame count and audio placement are compared."* The counts, here.
///
/// **Asserted exactly, with no tolerance in either direction.** Both files are committed
/// and deterministic, so a band between them would only hide a change. The two counts
/// differ by two, and the two are separately derived: Montagent puts `ceil(65216 × 25 /
/// 1000) = 1631` frames on the project's grid, and the published encode is not on that grid
/// at all — `r_frame_rate 50/1`, `avg_frame_rate 1390080/55651` ≈ 24.9785 fps — carrying
/// 1629 over the same stream duration. The divergence is reported as that arithmetic
/// rather than waved past.
fn the_frame_counts_are_compared_and_their_difference_is_accounted_for(
    output: &Path,
    published: &Quad,
) {
    let ours = video_stream(output);
    let theirs = video_stream(&reference());

    let on_the_grid = (DECLARED_DURATION_MS * FPS + 999) / 1000;
    assert_eq!(
        ours.frames,
        Some(on_the_grid),
        "ceil({DECLARED_DURATION_MS} x {FPS} / 1000)"
    );
    assert_eq!((ours.width, ours.height), (Some(1080), Some(1920)));
    assert_eq!(
        (theirs.width, theirs.height),
        (ours.width, ours.height),
        "the two are the same frame size or nothing below compares like with like"
    );

    let published_frames = theirs.frames.expect("the published video's frame count");
    // What a 25 fps grid would have put in the published stream's own duration, in tenths
    // of a frame — 16304, i.e. 1630.4 — which is neither count and is why there are two.
    let tenths = DECLARED_DURATION_MS * FPS / 100;
    println!(
        "FRAMES  published {published_frames}; this render {on_the_grid}; \
         divergence {}. A {FPS} fps grid over the published stream's {DECLARED_DURATION_MS} ms \
         holds {}.{} frames: Montagent takes the ceiling, and the published encode is not on \
         that grid — r_frame_rate {:?}, avg_frame_rate {:?}.",
        on_the_grid - published_frames,
        tenths / 10,
        tenths % 10,
        published.r_frame_rate.expect("a base rate"),
        published.avg_frame_rate.expect("an average rate"),
    );

    assert_eq!(
        published_frames, 1629,
        "the published video's frame count, which nothing Montagent does can change"
    );
    assert_ne!(
        published.r_frame_rate,
        Some(Rational::new(FPS, 1)),
        "if the published encode were on the project's grid the two counts would have to \
         agree, and the divergence above would be a defect rather than an explanation"
    );
    assert_ne!(published.avg_frame_rate, Some(Rational::new(FPS, 1)));
}

/// #219: *"audio placement"* — and the sharpest thing in this file, because it compares two
/// independent mixes of the same narration boundary by boundary.
///
/// `silencedetect` finds every crossing between speech and silence in each file. The two
/// must produce the same number of crossings in the same order — a mix that summed
/// everything at 0, or dropped a line, cannot — and then each pair must agree in time.
///
/// **They agree only once the published video's own `start_time` is taken off**, which is
/// the finding this comparison produced. The reference's narration sits ≈43 ms later than
/// the document says, and its video stream's first presentation timestamp is 42.031 ms:
/// the fourth member of ADR-0011's quad, and a quantity no container-level duration
/// comparison ever sees. Both halves are asserted — that the raw disagreement exceeds a
/// frame, so the correction is load-bearing rather than cosmetic, and that removing it
/// brings every boundary inside one.
fn the_narration_lands_where_the_published_video_puts_it(output: &Path, published: &Quad) {
    let theirs = silence_marks(&reference());
    let ours = silence_marks(output);
    assert_eq!(
        theirs.len(),
        ours.len(),
        "the two mixes cross between speech and silence a different number of times — \
         published {:?}, ours {:?}",
        theirs.len(),
        ours.len()
    );
    // Each of the fixture's `audio` elements is one run of sound with silence on at least
    // one side, so it contributes at least one crossing. Read off the document rather than
    // written down, so the floor tracks the fixture instead of restating it.
    let narration = elements(&fixture())
        .filter(|e| e["type"] == "audio")
        .count();
    assert!(
        theirs.len() >= narration,
        "{narration} narration elements against {} crossings — the published mix has lost \
         lines",
        theirs.len()
    );

    // The last crossing in each file is `silencedetect` closing its final run at
    // end-of-file, so it measures the two files' durations — already compared above — and
    // not where a narration line is. Every other crossing is a narration boundary.
    let boundaries = theirs.len() - 1;
    assert_eq!(
        theirs.last().map(|m| m.edge),
        Some(Edge::SpeechBegan),
        "the trailing mark is the end-of-file one this excludes"
    );

    let mut offsets: Vec<f64> = Vec::new();
    for (index, (published, ours)) in theirs.iter().zip(ours.iter()).take(boundaries).enumerate() {
        assert_eq!(
            published.edge, ours.edge,
            "crossing {index} is a {:?} in the published video and a {:?} here — the two \
             mixes do not hold the same narration",
            published.edge, ours.edge
        );
        offsets.push(published.at_ms - ours.at_ms);
    }

    let mut sorted = offsets.clone();
    sorted.sort_by(f64::total_cmp);
    let median = sorted[sorted.len() / 2];
    let start_time = published
        .start_time_ms
        .expect("a first presentation timestamp") as f64;
    let worst = offsets
        .iter()
        .map(|o| (o - start_time).abs())
        .fold(0.0, f64::max);
    println!(
        "NARRATION  {boundaries} speech/silence boundaries, matched one to one. The \
         published mix is later by a median of {median:.1} ms; the published video stream's \
         start_time is {start_time:.0} ms. Residual after removing it: worst {worst:.1} ms, \
         against one frame of {FRAME_MS} ms.",
    );

    assert!(
        median > FRAME_MS as f64,
        "the raw disagreement is {median:.1} ms, inside one frame — the start_time \
         correction below would then be unfalsifiable padding rather than the explanation"
    );
    assert!(
        (median - start_time).abs() < FRAME_MS as f64,
        "the published mix is {median:.1} ms late and its stream's start_time is \
         {start_time:.0} ms — more than one frame apart, so the start_time is not what the \
         offset is and this test is naming the wrong cause. They matched to within a \
         millisecond when #219 measured them; the printed number above is the live one."
    );
    assert!(
        worst < FRAME_MS as f64,
        "once the published video's own {start_time:.0} ms start_time is removed, some \
         narration boundary is still {worst:.1} ms from where this render puts it — more \
         than one frame at {FPS} fps. The mix has moved."
    );

    // Level, measured and reported, gating nothing: the published mix is mono at 24 kHz
    // and this one stereo at 48 kHz, so the two peak at different numbers throughout for a
    // reason that is a format difference rather than a defect. Placement is the claim.
    let audio = audio_stream(output).expect("the narration");
    let published_audio = audio_stream(&reference()).expect("the published narration");
    let (from, to) = (10.5, 12.6);
    println!(
        "LEVEL  {}..{} s peaks at {:.1} dBFS in the published mix ({} Hz) and {:.1} dBFS \
         here ({} Hz) — reported, not gated.",
        from,
        to,
        peak_db(&reference(), from, to),
        published_audio.sample_rate.unwrap_or_default(),
        peak_db(output, from, to),
        audio.sample_rate.unwrap_or_default(),
    );
}

/// What `speed` predicts of one element, measured from the file it plays.
///
/// `silencedetect` measures how long the source itself speaks for — its own leading and
/// trailing room tone taken off, which is why this is measured rather than read off
/// `source_end - source_start` — and ADR-0020's convention says the element must speak for
/// that ÷ `speed`, beginning `lead-in ÷ speed` after its own `start`.
///
/// Returned as a pair because both halves are claims: the length tests `atempo`, and the
/// onset tests placement, and #219 asks for both — *"land at the right instants"*.
struct Predicted {
    /// How long the source speaks for, unstretched.
    source_speech_ms: f64,
    /// How long the element must speak for.
    length_ms: f64,
    /// Where on the project's timeline it must begin.
    onset_ms: f64,
}

#[track_caller]
fn predicted(element: &Stretched) -> Predicted {
    let source = fixture_dir().join(&element.source);
    let duration = audio_stream(&source)
        .and_then(|s| s.duration_ms)
        .expect("the source's own length") as f64;

    // The prediction is the *whole file's* speech, so the element must play the whole
    // file. Asserted rather than assumed: a trimmed source window would make every number
    // below a prediction about audio this element never reaches.
    assert_eq!(element.source_start, 0, "`{}`", element.id);
    assert_eq!(
        element.source_end, duration as i64,
        "`{}` plays {}..{} ms of a {duration:.0} ms file, so the whole-file speech span is \
         not what it sounds",
        element.id, element.source_start, element.source_end
    );

    let spans = speech_spans(&source, duration);
    assert_eq!(
        spans.len(),
        1,
        "`{}` reads {} spans of speech in {}, not one",
        element.id,
        spans.len(),
        element.source
    );
    Predicted {
        source_speech_ms: spans[0].length_ms(),
        length_ms: spans[0].length_ms() / element.speed,
        onset_ms: element.start as f64 + spans[0].from_ms / element.speed,
    }
}

/// #219: *"The four `speed: 0.645` narration elements land at the right instants."* Spec
/// #168 calls this file the only test of #216, and this is the half of #216 nothing else
/// can reach.
///
/// Two claims per element, against **both** files, because *"land at the right instants"*
/// is two questions: the line must run for as long as `0.645` says, and it must begin
/// where `0.645` says. A length-only check passes a line stretched correctly and placed
/// 300 ms late.
///
/// The prediction comes from the **source file**, not from a sibling element and not from
/// a number written here — see [`predicted`]. Grounding it there is what makes the test
/// sensitive: an element rendered at `speed: 1` measures 35 % short of the predicted
/// length, eleven times [`SPAN_TOLERANCE`], which
/// [`the_span_and_placement_predicates_reject_what_they_are_meant_to`] demonstrates rather
/// than asserts here.
///
/// The published side has its own `start_time` taken off first, for the reason
/// [`the_narration_lands_where_the_published_video_puts_it`] establishes.
fn the_four_stretched_lines_are_stretched_in_the_published_video_too(
    output: &Path,
    published_quad: &Quad,
) {
    let stretched = stretched_narration();
    assert_eq!(stretched.len(), 4, "the fixture's four 0.645 sentences");

    let published_spans = speech_spans(&reference(), DECLARED_DURATION_MS as f64);
    let our_spans = speech_spans(output, DECLARED_DURATION_MS as f64);
    let start_time = published_quad
        .start_time_ms
        .expect("a first presentation timestamp") as f64;

    for element in &stretched {
        assert_eq!(element.speed, SPEED, "`{}`", element.id);
        let predicted = predicted(element);

        let published = the_one_span_within(
            &published_spans,
            element.start,
            element.end,
            &format!("`{}` in the published video", element.id),
        );
        let ours = the_one_span_within(
            &our_spans,
            element.start,
            element.end,
            &format!("`{}` in this render", element.id),
        );

        let sides = [
            (
                "the published video",
                published.length_ms(),
                published.from_ms - start_time,
            ),
            ("this render", ours.length_ms(), ours.from_ms),
        ];
        println!(
            "SPEED  `{}` {}..{} ms — {} speaks for {:.0} ms, so at {SPEED} it must run \
             {:.0} ms from {:.0} ms. Published: {:.0} ms from {:.0} ms ({:+.2} %, {:+.0} ms). \
             This render: {:.0} ms from {:.0} ms ({:+.2} %, {:+.0} ms).",
            element.id,
            element.start,
            element.end,
            element.source,
            predicted.source_speech_ms,
            predicted.length_ms,
            predicted.onset_ms,
            sides[0].1,
            sides[0].2,
            (sides[0].1 / predicted.length_ms - 1.0) * 100.0,
            sides[0].2 - predicted.onset_ms,
            sides[1].1,
            sides[1].2,
            (sides[1].1 / predicted.length_ms - 1.0) * 100.0,
            sides[1].2 - predicted.onset_ms,
        );

        for (what, length, onset) in sides {
            let off = (length / predicted.length_ms - 1.0).abs();
            assert!(
                off <= SPAN_TOLERANCE,
                "`{}` runs {length:.0} ms in {what}, {:.2} % from the {:.0} ms its source and \
                 `speed: {SPEED}` predict. Unstretched it would be {:.0} ms.",
                element.id,
                off * 100.0,
                predicted.length_ms,
                predicted.source_speech_ms,
            );
            let late = onset - predicted.onset_ms;
            assert!(
                late.abs() < FRAME_MS as f64,
                "`{}` begins sounding at {onset:.0} ms in {what}, {late:+.0} ms from the \
                 {:.0} ms its `start` and `speed: {SPEED}` predict — more than one frame at \
                 {FPS} fps.",
                element.id,
                predicted.onset_ms,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The predicates are demonstrated sensitive
// ---------------------------------------------------------------------------

/// A passing comparison says nothing on its own — this repository has twice shipped a
/// check that *"completed, looked plausible, was wrong"* (ADR-0010), and #213 answered the
/// same objection for the frame gate by breaking a render and requiring the comparison to
/// notice. This is that pair, for the two predicates above that carry a tolerance. The
/// others compare committed numbers for exact equality and have nothing to widen.
///
/// Both are shown against small renders rather than against the fixture, because the
/// question is whether the *predicate* can fail and a 1631-frame encode is not needed to
/// answer it. The published video is not involved: there is nothing to compare a
/// deliberately broken render against except the prediction the predicate makes.
///
/// - **`speed`.** The same element at `speed: 1` measures 35 % from what the source and
///   `0.645` predict — eleven times [`SPAN_TOLERANCE`] — so the band cannot pass an
///   unstretched render.
/// - **Placement.** Moving the element 200 ms moves its speech onset 200 ms, which is five
///   frames: the one-frame budget both the onset check above and
///   [`the_narration_lands_where_the_published_video_puts_it`] hold every boundary to
///   cannot absorb a mis-placed line.
#[test]
fn the_span_and_placement_predicates_reject_what_they_are_meant_to() {
    if !has_ffprobe() {
        return;
    }
    the_constants_still_restate_the_document();
    let dir = tempdir(line!());
    let source = fixture_dir()
        .join("audio/sentence-05-cobweb.mp3")
        .display()
        .to_string()
        .replace('\\', "/");

    // How long the source itself speaks for, and where `speed: 0.645` must carry it —
    // exactly the prediction the fixture comparison makes, recomputed from the same
    // measurement so the two cannot drift apart.
    let source_duration = audio_stream(Path::new(&source))
        .and_then(|s| s.duration_ms)
        .expect("the source's own length") as f64;
    let inside = speech_spans(Path::new(&source), source_duration);
    assert_eq!(inside.len(), 1, "one sentence in the source");
    let prediction = inside[0].length_ms() / SPEED;

    // Three renders: the declaration as the fixture makes it, the same element left
    // unstretched, and the same element moved late. `end - start` follows ADR-0005's
    // invariant in each, which is what keeps all three legal.
    let span = source_duration as i64;
    let stretched = (span as f64 / SPEED).round() as i64;
    let cases = [
        ("as declared", 500_i64, SPEED, stretched),
        ("unstretched", 500, 1.0, span),
        ("moved late", 700, SPEED, stretched),
    ];

    let mut measured = Vec::new();
    for (what, start, speed, length) in cases {
        let end = start + length;
        let body = canonical(&format!(
            r##"{{"frame":{{"width":200,"height":200}},"fps":{FPS},"duration":6000,"output":"out/{start}-{speed}.mp4",
                "tracks":[{{"name":"vo","layer":0,"elements":[
                  {{"id":"vo","type":"audio","start":{start},"end":{end},"source":"{source}",
                    "source_start":0,"source_end":{span},"speed":{speed}}}]}}]}}"##
        ));
        let path = write_project(&dir, &format!("{start}-{speed}.montagent.json"), &body);
        let answer = render(&path, &Ask::default(), &mut |_| {});
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::Ok,
            "{what}: {}",
            answer.to_json()
        );
        let written = dir.join(format!("out/{start}-{speed}.mp4"));
        let spans = speech_spans(&written, 6000.0);
        assert_eq!(spans.len(), 1, "{what}: one line of speech, not {spans:?}");
        println!(
            "SENSITIVITY  {what}: speech {:.0}..{:.0} ms, {:.0} ms long against a \
             prediction of {prediction:.0} ms ({:+.1} %).",
            spans[0].from_ms,
            spans[0].to_ms,
            spans[0].length_ms(),
            (spans[0].length_ms() / prediction - 1.0) * 100.0,
        );
        measured.push(spans[0]);
    }
    let [declared, unstretched, moved_late] = measured[..] else {
        unreachable!("three renders")
    };

    assert!(
        (declared.length_ms() / prediction - 1.0).abs() <= SPAN_TOLERANCE,
        "the band must pass the declaration it was derived from: {:.0} ms against \
         {prediction:.0} ms",
        declared.length_ms()
    );
    assert!(
        (unstretched.length_ms() / prediction - 1.0).abs() > SPAN_TOLERANCE,
        "an unstretched render measured {:.0} ms against a prediction of {prediction:.0} ms \
         and the band still passed it — the {SPEED} check is not testing `speed`",
        unstretched.length_ms()
    );

    let moved = moved_late.from_ms - declared.from_ms;
    assert!(
        moved > FRAME_MS as f64,
        "moving the element 200 ms moved its speech onset {moved:.0} ms, inside one frame — \
         a mis-placed narration line would pass the placement checks"
    );
    println!(
        "SENSITIVITY  moving the element 200 ms moved its speech onset {moved:.0} ms, \
         against a one-frame budget of {FRAME_MS} ms."
    );
}
