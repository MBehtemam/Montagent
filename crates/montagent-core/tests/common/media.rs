//! Reading a written file back with a decoder that is not the encoder that wrote it.
//!
//! Every claim in the suite about an *output file* — its streams, its frame count, how
//! loud it is at an instant, where its speech starts — is a claim about bytes Montagent
//! produced, so it is answered by `ffprobe`/`ffmpeg` rather than by Montagent's own report.
//! A render that mis-stated its own frame count would otherwise agree with itself.
//!
//! Shared rather than written twice: `render.rs` asserts these numbers against the
//! *document*, and `reference_video.rs` asserts them against the *published video*. Two
//! copies of "what does `ffprobe` say" is two places for that question to get a different
//! answer.

#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

use serde_json::Value;

/// One stream of a written file, as `ffprobe` reports it.
#[derive(Debug)]
pub struct Stream {
    pub kind: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub frames: Option<i64>,
    pub duration_ms: Option<i64>,
    pub sample_rate: Option<i64>,
}

pub fn streams(path: &Path) -> Vec<Stream> {
    let tools = montagent_core::media::tools::resolve().expect("ffprobe");
    let out = Command::new(tools.ffprobe)
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(path)
        .output()
        .expect("run ffprobe");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: Value = serde_json::from_slice(&out.stdout).expect("ffprobe json");
    json["streams"]
        .as_array()
        .expect("streams")
        .iter()
        .map(|s| Stream {
            kind: s["codec_type"].as_str().unwrap_or("?").to_string(),
            width: s["width"].as_i64(),
            height: s["height"].as_i64(),
            frames: s["nb_frames"].as_str().and_then(|n| n.parse().ok()),
            duration_ms: s["duration"]
                .as_str()
                .and_then(|d| d.parse::<f64>().ok())
                .map(|d| (d * 1000.0).round() as i64),
            sample_rate: s["sample_rate"].as_str().and_then(|n| n.parse().ok()),
        })
        .collect()
}

pub fn video_stream(path: &Path) -> Stream {
    streams(path)
        .into_iter()
        .find(|s| s.kind == "video")
        .expect("a video stream")
}

pub fn audio_stream(path: &Path) -> Option<Stream> {
    streams(path).into_iter().find(|s| s.kind == "audio")
}

/// The peak level of `[from, to)` seconds of the file's audio, in dBFS — `-inf` for
/// digital silence.
pub fn peak_db(path: &Path, from: f64, to: f64) -> f64 {
    let tools = montagent_core::media::tools::resolve().expect("ffmpeg");
    let out = Command::new(tools.ffmpeg)
        .args([
            "-v",
            "info",
            "-ss",
            &from.to_string(),
            "-t",
            &(to - from).to_string(),
            "-i",
        ])
        .arg(path)
        .args(["-vn", "-af", "volumedetect", "-f", "null", "-"])
        .output()
        .expect("run ffmpeg");
    let said = String::from_utf8_lossy(&out.stderr);
    let line = said
        .lines()
        .find(|l| l.contains("max_volume"))
        .unwrap_or_else(|| panic!("no volumedetect line in:\n{said}"));
    let value = line.split("max_volume:").nth(1).unwrap().trim();
    value
        .trim_end_matches(" dB")
        .parse()
        .unwrap_or(f64::NEG_INFINITY)
}

/// The file's audio as 48 kHz mono `f32` samples, from sample 0 of the stream — what a
/// claim about *which sample* something happens at is read against.
pub fn samples(path: &Path) -> Vec<f32> {
    let tools = montagent_core::media::tools::resolve().expect("ffmpeg");
    let out = Command::new(tools.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args(["-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "-"])
        .output()
        .expect("run ffmpeg");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

/// Where the audio crosses between speech and silence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// Sound stopped here.
    SpeechEnded,
    /// Sound resumed here.
    SpeechBegan,
}

/// One boundary between speech and silence, in milliseconds on the audio stream's own
/// clock.
#[derive(Debug, Clone, Copy)]
pub struct Mark {
    pub edge: Edge,
    pub at_ms: f64,
}

/// What counts as silence, in dBFS.
///
/// The fixture's narration peaks between -3 and -19 dBFS and the silence between its lines
/// is digital, so -45 sits in the empty middle of the two populations rather than near
/// either — a separator, not a tuned parameter. Shared by [`silence_marks`] and
/// [`speech_spans`] so the two cannot come to different conclusions about the same sample.
pub const SILENCE_FLOOR_DB: f64 = -45.0;

/// Every speech/silence boundary in a file's audio, in order.
///
/// The 150 ms hold is what keeps a glottal stop inside a word from reading as a gap
/// between two words.
pub fn silence_marks(path: &Path) -> Vec<Mark> {
    let tools = montagent_core::media::tools::resolve().expect("ffmpeg");
    let out = Command::new(tools.ffmpeg)
        .args(["-v", "info", "-i"])
        .arg(path)
        .args([
            "-vn",
            "-af",
            &format!("silencedetect=noise={SILENCE_FLOOR_DB}dB:d=0.15"),
            "-f",
            "null",
            "-",
        ])
        .output()
        .expect("run ffmpeg");
    let said = String::from_utf8_lossy(&out.stderr);
    let mut marks = Vec::new();
    for line in said.lines() {
        // `silence_start` is where speech ended; `silence_end` is where it resumed. Named
        // for the speech rather than for the silence, because every caller below is asking
        // where a narration line begins and ends.
        let (edge, key) = if line.contains("silence_start:") {
            (Edge::SpeechEnded, "silence_start:")
        } else if line.contains("silence_end:") {
            (Edge::SpeechBegan, "silence_end:")
        } else {
            continue;
        };
        let rest = line.split(key).nth(1).expect("the key is in the line");
        let number = rest.split('|').next().unwrap_or(rest).trim();
        if let Ok(seconds) = number.parse::<f64>() {
            marks.push(Mark {
                edge,
                at_ms: seconds * 1000.0,
            });
        }
    }
    marks
}

/// One stretch of a file's audio that carries sound, in milliseconds on the audio
/// stream's own clock.
///
/// A type rather than a `(f64, f64)` because every caller wants one of three things from
/// it — where the line starts, where it ends, how long it ran — and a bare pair makes the
/// third arithmetic the caller has to get right each time.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub from_ms: f64,
    pub to_ms: f64,
}

impl Span {
    pub fn length_ms(&self) -> f64 {
        self.to_ms - self.from_ms
    }
}

/// The spans of a file's audio that carry sound, derived from [`silence_marks`].
///
/// A file that opens with speech has no `SpeechBegan` before it, so the first span starts
/// at 0; one that ends in speech is closed at `until_ms`.
///
/// **Whether the file opens in speech is measured, not inferred from the marks.**
/// `silencedetect` reports a *transition*, and it is not consistent about the one at the
/// head: a narration MP3 that begins with room tone emits `silence_start: 0`, while an
/// encoded render that begins with the same silence can emit its first `silence_start`
/// part-way in, with nothing before it. Assuming speech until told otherwise turns that
/// second case into a phantom span at the front of the file and reports one narration line
/// as two. So the first 50 ms is levelled against `silencedetect`'s own threshold and the
/// answer comes from the audio rather than from the absence of a log line.
pub fn speech_spans(path: &Path, until_ms: f64) -> Vec<Span> {
    let marks = silence_marks(path);
    let mut spans = Vec::new();
    let mut open = (peak_db(path, 0.0, 0.05) > SILENCE_FLOOR_DB).then_some(0.0);
    for mark in &marks {
        match mark.edge {
            Edge::SpeechEnded => {
                if let Some(from_ms) = open.take() {
                    spans.push(Span {
                        from_ms,
                        to_ms: mark.at_ms,
                    });
                }
            }
            Edge::SpeechBegan => open = Some(mark.at_ms),
        }
    }
    if let Some(from_ms) = open
        && from_ms < until_ms
    {
        spans.push(Span {
            from_ms,
            to_ms: until_ms,
        });
    }
    // A zero-length span is two transitions at the same instant, which is an artefact of
    // where the file starts rather than a thing that was heard. One millisecond rather than
    // zero because `silencedetect` reports its edges in samples.
    spans.retain(|span| span.length_ms() > 1.0);
    spans
}
