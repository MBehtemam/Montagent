//! `verify` — *is X in the file?* (ADR-0117, [#436](https://github.com/MBehtemam/Montagent/issues/436)).
//!
//! `render` asks *"did I intend to put X in?"*, and after ADR-0093 it withholds the
//! deliverable whenever the answer is no. What that cannot catch is the class in which **the
//! engine believed its own output** — a mix it built and the encoder truncated, a frame count
//! it pushed and the container disagrees with. The only witness for that class is one that
//! does not consult the engine, so this verb measures the file with the decoder (`ffprobe`,
//! `ffmpeg`) and compares it to the document, and never reads what the engine established.
//!
//! **This is not a second data path in ADR-0093 ruling 3's sense.** It is one file and two
//! different questions, not one question asked down two data paths. `render` states intent;
//! `verify` observes the artifact.
//!
//! ## Two phases: identity, then measurement
//!
//! Measuring a file against a document it was not rendered from produces confident nonsense,
//! so the stamp is read first ([`attest`]):
//!
//! | what is at `output` | finding | then |
//! | --- | --- | --- |
//! | nothing | `E-VERIFY-NO-OUTPUT` | stop |
//! | another project's stamp | `E-OUTPUT-FOREIGN` | stop |
//! | this project's stamp, digest differs | `E-VERIFY-STALE` | stop |
//! | this project's stamp, digest matches | — | measure |
//! | no stamp | `R-VERIFY-UNATTESTED` | measure |
//! | this project's stamp, staleness unknown | `R-VERIFY-STALENESS-UNKNOWN` | measure |
//!
//! A stale file is **one** finding and measurement stops, because after an edit every
//! duration, frame-count and energy mismatch descends from one fact — *the document changed* —
//! and the census cannot carry it instead: a census collapses siblings of one code, and the
//! stale cascade spans several. The flip side is the point: once the gate passes, every
//! mismatch below is attributable to the engine or the encoder.
//!
//! ## What is measured
//!
//! Frame size, frame timing from decoded PTS (never `r_frame_rate`/`avg_frame_rate` —
//! ADR-0011, ADR-0096), frame count and stream duration (one finding when both fail from one
//! cause), audio stream presence, audio extent against video extent, and per-span energy.
//!
//! **Energy is per span, not per element.** The file carries one mixed track, so a silent
//! voice-over under music is invisible in it. The honest question is where the mix is silent
//! inside the union of spans in which something *should be heard*. It is measured with EBU
//! R128 momentary loudness against the −70 LUFS absolute gate, both borrowed and so `review`
//! only (ADR-0061), cited and reported raw. Each source is measured too, so the census splits
//! into *the source is silent here* (the author's material) and *the source is audible and
//! the mix is not* (the engine) — without that split a genuinely silent recording reads as an
//! engine bug.
//!
//! ## On demand only
//!
//! `render` never runs this. A `verify` `error` inside `render`'s report would sit beside a
//! file already published, which breaks ADR-0093's *"a file at the output path is a
//! zero-error render"*; and sharing `render`'s process and parsed model would compromise the
//! independence that is the verb's only value. `render`'s `NOT CHECKED` names it instead.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};

use crate::exact::{self, Decimal};
use crate::finding::{Census, Citation, Finding};
use crate::media::attest::{self, Attestation};
use crate::media::digest::{self, Digest};
use crate::media::probe::{self, Execution, LOCAL_PROTOCOLS, Runner};
use crate::media::tools::{self, Tools};
use crate::media::{Source, display_local};
use crate::model::{Animatable, Keyframe, Volume};
use crate::parse;
use crate::permissive::Loose;
use crate::registry::CheckSet;
use crate::report::Report;
use crate::resolve;

const TOOL: &str = "verify";

/// EBU R 128's absolute gate, in LUFS. Borrowed (ADR-0061), so it decides `review` only.
pub const GATE_LUFS: f64 = -70.0;

/// ITU-R BS.1770's momentary block, in milliseconds — the shortest stretch the loudness
/// measurement can call silent, and so the minimum silent length. Borrowed rather than an
/// unsourced one second, for the same reason as the gate.
pub const BLOCK_MS: i64 = 400;

/// The citation every energy finding carries inline (ADR-0061).
const R128: &str = "EBU R 128 (2020): the absolute gate of −70 LUFS, applied to ITU-R BS.1770-4 \
momentary loudness over 400 ms blocks";

/// What `verify` adds to the report's own boundary: the stated limits of an independent
/// witness that sees one mixed track and the container's structure.
pub const NOT_CHECKED: &[&str] = &[
    "What each frame shows. verify measures the picture's size, count and timing; it does not \
compare a frame's pixels against the document, so a frame the engine drew wrong but on time \
passes.",
    "An element masked by another audible one. The file carries one mixed track, so an \
element's own energy cannot be measured from it; where one element is alone in a silent span \
the finding says its attribution is exact.",
    "Audio present but far too quiet. The energy check fires below the −70 LUFS absolute gate \
only, so a mix at −45 LUFS where it should be at −16 passes.",
    "A remote source's bytes. The staleness digest covers its URL as the document spells it, \
not what the URL serves now (ADR-0056).",
];

/// One `verify` invocation's answer: what was measured, and the report.
pub struct Answer {
    report: Report,
    measured: Option<Measured>,
}

impl Answer {
    pub fn report(&self) -> &Report {
        &self.report
    }

    pub fn measured(&self) -> Option<&Measured> {
        self.measured.as_ref()
    }

    /// The canonical JSON: the report's object plus the `verify` block — `null` where the
    /// identity gate stopped the run before anything was measured.
    pub fn to_json(&self) -> Value {
        let mut json = self.report.to_json_with(
            "verify",
            match &self.measured {
                Some(measured) => serde_json::to_value(measured).unwrap_or(Value::Null),
                None => Value::Null,
            },
        );
        crate::report::extend_boundary(&mut json, NOT_CHECKED);
        json
    }
}

/// What the decoder said about the file, raw — the substance every finding is judged from.
#[derive(Debug, Clone, Serialize)]
pub struct Measured {
    pub path: String,
    /// `mine`, `unattested` or `staleness_unknown`: how far the identity gate got.
    pub attestation: &'static str,
    /// The engine the stamp names, where it names one.
    pub engine: Option<String>,
    pub width: i64,
    pub height: i64,
    pub frames: u64,
    /// First PTS to last PTS plus one frame, in milliseconds.
    pub video_ms: f64,
    /// Audio stream duration in milliseconds, or `None` where there is no audio stream.
    pub audio_ms: Option<f64>,
    /// The spans in which something should be heard, in milliseconds, half-open.
    pub should_be_heard: Vec<(i64, i64)>,
}

/// Verify the deliverable at the project's declared `output`.
pub fn verify(path: &Path) -> Answer {
    let project = Some(path.display().to_string());
    let refused = |report: Report| Answer {
        report,
        measured: None,
    };

    let document = match parse::read(path) {
        Ok(document) => document,
        Err(finding) => return refused(Report::unparseable(TOOL, project, *finding)),
    };
    let mut report = Report::new(TOOL, project.clone());
    if let Err(not_a_project) = document.shape() {
        report.push(Finding::not_a_project(
            document.path(),
            &not_a_project,
            "point verify at the project file",
        ));
        return refused(report);
    }
    let header = match document.strict() {
        Ok(header) => header,
        Err(e) => {
            return refused(Report::rejected(
                TOOL,
                project,
                format!(
                    "the project does not fit the model ({e}), so there is no document to \
                     measure the file against: run `montagent validate` first"
                ),
            ));
        }
    };
    let project_dir = crate::checks::project_dir(&document);
    let Some(declared) = header.output.as_deref() else {
        return refused(Report::rejected(
            TOOL,
            project,
            "the project declares no `output`, so there is no deliverable to verify: verify \
             reads only the declared `output`, the one file `render` stamps",
        ));
    };
    let output = project_dir.join(declared);
    let Some(extent) = crate::exact::extent(&document) else {
        return refused(Report::rejected(
            TOOL,
            project,
            "the project declares no `duration` and no element states a range, so it renders \
             nothing and there is nothing to verify",
        ));
    };
    let resolved = match tools::resolve() {
        Ok(tools) => tools,
        Err(missing) => {
            missing.fail(&mut report);
            return refused(report);
        }
    };
    let runner = probe::ProcessRunner;
    let at = |code: &str| {
        Finding::new(code)
            .at_file(document.path().to_string())
            .field("output", json!(display_local(&output)))
    };

    // ---- Phase 1: identity ----------------------------------------------------------------
    let render_command = json!({ "run": format!("montagent render {}", document.path()) });
    let (attestation, engine) =
        match attest::of(&runner, &resolved, &output, Path::new(document.path())) {
            Attestation::Vacant => {
                report.push(at("E-VERIFY-NO-OUTPUT").repair_value(render_command));
                return refused(report);
            }
            Attestation::Foreign { project } => {
                report.push(at("E-OUTPUT-FOREIGN").field("project", json!(project)));
                return refused(report);
            }
            Attestation::Absent => {
                report.push(at("R-VERIFY-UNATTESTED"));
                ("unattested", None)
            }
            Attestation::Mine(stamp) => match (&stamp.digest, digest::of(&document)) {
                (Some(stamped), Digest::Known(now)) if *stamped == now => ("mine", stamp.engine),
                (Some(_), Digest::Known(_)) => {
                    report.push(at("E-VERIFY-STALE").repair_value(render_command));
                    return refused(report);
                }
                (None, _) => {
                    report.push(at("R-VERIFY-STALENESS-UNKNOWN").field(
                        "reason",
                        json!(match stamp.version {
                            1 => "its stamp predates the staleness digest (`montagent/1`)"
                                .to_string(),
                            _ => "the render that wrote it could not fingerprint one of its \
                                  inputs, so it stamped no digest"
                                .to_string(),
                        }),
                    ));
                    ("staleness_unknown", stamp.engine)
                }
                (Some(_), Digest::Unread(unread)) => {
                    report.push(at("R-VERIFY-STALENESS-UNKNOWN").field(
                        "reason",
                        json!(format!(
                            "{} could not be fingerprinted now, and an input that cannot be \
                             read is no evidence of a change",
                            unread.join(", ")
                        )),
                    ));
                    ("staleness_unknown", stamp.engine)
                }
            },
        };

    // ---- Phase 2: measurement -------------------------------------------------------------
    let fps = header.fps;
    let (width, height) = (header.frame.width, header.frame.height);
    let file = match Structure::of(&runner, &resolved, &output) {
        Ok(file) => file,
        Err(reason) => {
            report.fail_internally(format!(
                "{} carries this project's attestation and could not be measured: {reason}",
                display_local(&output)
            ));
            return refused(report);
        }
    };

    // Frame size, against the frame `render` writes: the declared one, padded to even
    // (ADR-0021's disclosed padding), which is a rule of the format and not of the engine.
    let even = |n: i64| n + (n % 2);
    if (file.width, file.height) != (even(width), even(height)) {
        report.push(
            at("E-VERIFY-FRAME-SIZE")
                .field(
                    "expected",
                    json!(format!("{}x{}", even(width), even(height))),
                )
                .field("measured", json!(format!("{}x{}", file.width, file.height))),
        );
    }

    // Frame timing, from decoded PTS spacing. One tick either way is the rounding a
    // non-integral `timebase / fps` forces on the muxer; anything past it is a different rate.
    let step = file.time_base.1 as f64 / (fps as f64 * file.time_base.0 as f64);
    let gaps: Vec<(usize, i64)> = file
        .pts
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| ((pair[1] - pair[0]) as f64 - step).abs() > 1.0)
        .map(|(n, pair)| (n + 1, pair[1] - pair[0]))
        .collect();
    if let Some(&(first, ticks)) = gaps.first() {
        report.push(
            at("E-VERIFY-FRAME-TIMING")
                .field("fps", json!(fps))
                .field("irregular", json!(gaps.len()))
                .field("frame", json!(first))
                .field("interval_ms", json!(round3(file.ms(ticks as f64))))
                .field("expected_ms", json!(round3(1000.0 / fps as f64))),
        );
    }

    // Frame count and duration on the engine's own grid rule — ADR-0035's frames in
    // `[0, extent)` — and the video *stream*'s duration, never the container's.
    let expected_frames = match (
        exact::frame_at_or_after(0, fps),
        exact::frame_before(extent, fps),
    ) {
        (Some(first), Some(last)) if last.frame >= first.frame => {
            (last.frame - first.frame + 1) as u64
        }
        _ => 0,
    };
    let expected_ms = expected_frames as f64 * 1000.0 / fps as f64;
    let video_ms = file.video_ms(step);
    let tick_ms = file.ms(1.0);
    if file.pts.len() as u64 != expected_frames || (video_ms - expected_ms).abs() > tick_ms {
        report.push(
            at("E-VERIFY-EXTENT")
                .field("expected_frames", json!(expected_frames))
                .field("measured_frames", json!(file.pts.len()))
                .field("expected_ms", json!(round3(expected_ms)))
                .field("measured_ms", json!(round3(video_ms))),
        );
    }

    // What should be heard, from the document and the sources' own streams.
    let heard = Heard::of(&document, &project_dir, &runner, &resolved, fps, extent);
    let spans = heard.union();
    match (&file.audio, spans.is_empty()) {
        (None, false) => report.push(
            at("E-VERIFY-NO-AUDIO")
                .field("from", json!(spans[0].0))
                .field("to", json!(spans[0].1)),
        ),
        (Some(_), true) => report.push(at("N-VERIFY-UNEXPECTED-AUDIO")),
        _ => {}
    }

    // Audio extent against video extent: within one video frame plus one audio frame, the
    // granularity each stream is cut at. A truncated mixed track is at least as likely a
    // shape for "silent at 0:30" as a quiet one, and this catches it with no borrowed number.
    if let Some(audio) = &file.audio {
        let slack = 1000.0 / fps as f64 + audio.frame_ms;
        if (audio.ms - video_ms).abs() > slack {
            report.push(
                at("E-VERIFY-AUDIO-EXTENT")
                    .field("audio_ms", json!(round3(audio.ms)))
                    .field("video_ms", json!(round3(video_ms)))
                    .field("slack_ms", json!(round3(slack))),
            );
        }
    }

    // Per-span energy, where there is a track to measure and something should be in it.
    if file.audio.is_some() && !spans.is_empty() {
        match loudness(&runner, &resolved, &output) {
            Ok(mix) => {
                for finding in silent_spans(&mix, &spans, &heard, &runner, &resolved) {
                    report.push(
                        finding
                            .at_file(document.path().to_string())
                            .field("output", json!(display_local(&output))),
                    );
                }
            }
            Err(reason) => {
                report.fail_internally(format!(
                    "the loudness of {} could not be measured: {reason}",
                    display_local(&output)
                ));
                return refused(report);
            }
        }
    }
    // ADR-0112, as this ticket extends it: the measurement has completed. A refusal above
    // returned before it and records nothing.
    report.record(CheckSet::Deliverable);

    Answer {
        report,
        measured: Some(Measured {
            path: display_local(&output),
            attestation,
            engine,
            width: file.width,
            height: file.height,
            frames: file.pts.len() as u64,
            video_ms: round3(video_ms),
            audio_ms: file.audio.as_ref().map(|audio| round3(audio.ms)),
            should_be_heard: spans,
        }),
    }
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

// ---- The decoder's view of the file ---------------------------------------------------------

/// The file's structure, as the decoder reports it.
struct Structure {
    width: i64,
    height: i64,
    /// The video stream's time base, as `num/den`.
    time_base: (i64, i64),
    /// Every decoded video frame's PTS, in decode-output (presentation) order.
    pts: Vec<i64>,
    audio: Option<Track>,
}

/// The audio stream's extent.
struct Track {
    ms: f64,
    /// One coded audio frame, in milliseconds — the stream's own cutting granularity.
    frame_ms: f64,
}

impl Structure {
    fn of(runner: &dyn Runner, tools: &Tools, file: &Path) -> Result<Structure, String> {
        let streams: Value = serde_json::from_str(&ffprobe(
            runner,
            tools,
            file,
            &[
                "-show_entries",
                "stream=codec_type,width,height,time_base,sample_rate",
            ],
        )?)
        .map_err(|e| format!("ffprobe's stream list did not parse: {e}"))?;
        let streams = streams["streams"].as_array().cloned().unwrap_or_default();
        let video = streams
            .iter()
            .find(|s| s["codec_type"] == "video")
            .ok_or("the file has no video stream")?;
        let time_base = rational(video["time_base"].as_str().unwrap_or(""))
            .ok_or("the video stream states no time base")?;

        // A real decode: `-show_frames` runs the decoder, which is what an independent witness
        // costs. Packet counts are not frame counts on an encoder not shown to write one
        // packet per frame.
        let pts: Vec<i64> = csv(
            runner,
            tools,
            file,
            &["-select_streams", "v:0", "-show_entries", "frame=pts"],
        )?
        .iter()
        .filter_map(|row| row.first()?.parse().ok())
        .collect();

        let audio = match streams.iter().find(|s| s["codec_type"] == "audio") {
            None => None,
            Some(stream) => {
                let time_base = rational(stream["time_base"].as_str().unwrap_or(""))
                    .ok_or("the audio stream states no time base")?;
                let packets: Vec<(i64, i64)> = csv(
                    runner,
                    tools,
                    file,
                    &[
                        "-select_streams",
                        "a:0",
                        "-show_entries",
                        "packet=pts,duration",
                    ],
                )?
                .iter()
                .filter_map(|row| Some((row.first()?.parse().ok()?, row.get(1)?.parse().ok()?)))
                .collect();
                let tick = 1000.0 * time_base.0 as f64 / time_base.1 as f64;
                // Priming samples an AAC encoder writes before `t = 0` are not part of the
                // stream's extent: the edit list says so and the player never plays them.
                let first = packets
                    .iter()
                    .map(|(pts, _)| (*pts).max(0))
                    .min()
                    .unwrap_or(0);
                let end = packets.iter().map(|(pts, d)| pts + d).max().unwrap_or(0);
                let frame = packets.iter().map(|(_, d)| *d).max().unwrap_or(0);
                Some(Track {
                    ms: (end - first).max(0) as f64 * tick,
                    frame_ms: frame as f64 * tick,
                })
            }
        };

        Ok(Structure {
            width: video["width"].as_i64().unwrap_or(0),
            height: video["height"].as_i64().unwrap_or(0),
            time_base,
            pts,
            audio,
        })
    }

    fn ms(&self, ticks: f64) -> f64 {
        ticks * 1000.0 * self.time_base.0 as f64 / self.time_base.1 as f64
    }

    /// First PTS to last PTS plus one frame.
    fn video_ms(&self, step: f64) -> f64 {
        match (self.pts.iter().min(), self.pts.iter().max()) {
            (Some(first), Some(last)) => self.ms((last - first) as f64 + step),
            _ => 0.0,
        }
    }
}

fn rational(text: &str) -> Option<(i64, i64)> {
    let (num, den) = text.split_once('/')?;
    let (num, den) = (num.parse().ok()?, den.parse().ok()?);
    (num > 0 && den > 0).then_some((num, den))
}

fn ffprobe(
    runner: &dyn Runner,
    tools: &Tools,
    file: &Path,
    entries: &[&str],
) -> Result<String, String> {
    let mut args: Vec<String> = [
        "-v",
        "error",
        "-print_format",
        "json",
        "-protocol_whitelist",
        LOCAL_PROTOCOLS,
    ]
    .iter()
    .chain(entries)
    .map(|s| s.to_string())
    .collect();
    args.push(file.display().to_string());
    run(runner, &tools.ffprobe, &args)
}

fn csv(
    runner: &dyn Runner,
    tools: &Tools,
    file: &Path,
    entries: &[&str],
) -> Result<Vec<Vec<String>>, String> {
    let mut args: Vec<String> = [
        "-v",
        "error",
        "-of",
        "csv=p=0",
        "-protocol_whitelist",
        LOCAL_PROTOCOLS,
    ]
    .iter()
    .chain(entries)
    .map(|s| s.to_string())
    .collect();
    args.push(file.display().to_string());
    Ok(run(runner, &tools.ffprobe, &args)?
        .lines()
        .map(|line| {
            line.split(',')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect())
}

fn run(runner: &dyn Runner, program: &Path, args: &[String]) -> Result<String, String> {
    let Execution {
        success,
        stdout,
        stderr,
        ..
    } = runner
        .run(program, args)
        .map_err(|e| format!("{} could not be run: {e}", program.display()))?;
    if !success {
        return Err(format!("{} failed: {}", program.display(), stderr.trim()));
    }
    Ok(stdout)
}

// ---- Loudness -------------------------------------------------------------------------------

/// One momentary-loudness reading: the 400 ms block ending at `end_ms`, in LUFS.
#[derive(Debug, Clone, Copy)]
struct Reading {
    end_ms: i64,
    lufs: f64,
}

/// Every full-block momentary reading of a file's first audio stream, from `ebur128`'s
/// per-100 ms log. Readings before the first full block are the filter's placeholder
/// (−120.7 even over a loud tone) and are dropped: they are not measurements.
fn loudness(runner: &dyn Runner, tools: &Tools, file: &Path) -> Result<Vec<Reading>, String> {
    let args: Vec<String> = [
        "-hide_banner",
        "-nostats",
        "-v",
        "verbose",
        "-protocol_whitelist",
        LOCAL_PROTOCOLS,
        "-i",
    ]
    .iter()
    .map(|s| s.to_string())
    .chain([
        file.display().to_string(),
        "-map".into(),
        "0:a:0".into(),
        "-af".into(),
        "ebur128=framelog=verbose".into(),
        "-f".into(),
        "null".into(),
        "-".into(),
    ])
    .collect();
    let Execution {
        success, stderr, ..
    } = runner
        .run(&tools.ffmpeg, &args)
        .map_err(|e| format!("{} could not be run: {e}", tools.ffmpeg.display()))?;
    if !success {
        return Err(format!(
            "{} failed: {}",
            tools.ffmpeg.display(),
            last_line(&stderr)
        ));
    }
    Ok(stderr
        .lines()
        .filter(|line| line.contains("Parsed_ebur128"))
        .filter_map(parse_reading)
        .filter(|reading| reading.end_ms >= BLOCK_MS)
        .collect())
}

fn last_line(text: &str) -> &str {
    text.lines().last().unwrap_or("").trim()
}

/// `… t: 0.399979   TARGET:-23 LUFS    M: -21.7 S:-120.7 …` → the block ending at 400 ms.
fn parse_reading(line: &str) -> Option<Reading> {
    let t: f64 = line
        .split("t:")
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let m: f64 = line
        .split(" M:")
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(Reading {
        end_ms: (t * 1000.0).round() as i64,
        // A block of exact digital silence prints `M:   nan`, and `NaN < GATE_LUFS` is false:
        // read as it is, the quietest block there is would count as heard.
        lufs: if m.is_nan() { f64::NEG_INFINITY } else { m },
    })
}

// ---- What should be heard -------------------------------------------------------------------

/// One element's audible contribution: where on the clock it should be heard, and how the
/// clock maps back onto its source.
#[derive(Debug, Clone)]
struct Contribution {
    element: String,
    source: PathBuf,
    /// The spans on the clock in which its resolved `volume` is above 0.
    spans: Vec<(i64, i64)>,
    start: i64,
    source_start: i64,
    source_end: i64,
    speed: f64,
    /// The as-played length of one pass, where it loops.
    period: Option<i64>,
    /// The largest resolved `volume` anywhere in its spans, for the source-energy reading.
    peak_volume: f64,
}

struct Heard {
    contributions: Vec<Contribution>,
}

impl Heard {
    /// Something should be heard where an `audio` or `video` element is in range, its source
    /// has an audio stream, and its resolved `volume` is above 0 — evaluated per frame, so a
    /// keyframed fade to 0 is respected.
    fn of(
        document: &Loose,
        project_dir: &Path,
        runner: &dyn Runner,
        tools: &Tools,
        fps: i64,
        extent: i64,
    ) -> Heard {
        let mut has_audio: BTreeMap<PathBuf, bool> = BTreeMap::new();
        let mut contributions = Vec::new();
        for (index, element) in document.elements().enumerate() {
            if !matches!(
                element.get("type").and_then(Value::as_str),
                Some("audio") | Some("video")
            ) {
                continue;
            }
            // ADR-0157 §4: a remapped video is silent, so nothing of it should be heard.
            if crate::remap::is_remapped(element) {
                continue;
            }
            let name = element
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("(element {index} with no id)"));
            let int = |key: &str| element.get(key).and_then(Value::as_i64);
            let (Some(start), Some(end), Some(source_start), Some(source_end)) = (
                int("start"),
                int("end"),
                int("source_start"),
                int("source_end"),
            ) else {
                continue;
            };
            let Some(Source::Local(path)) = element
                .get("source")
                .and_then(Value::as_str)
                .map(|s| Source::resolve(s, project_dir))
            else {
                continue;
            };
            let audible = *has_audio
                .entry(path.clone())
                .or_insert_with(|| source_has_audio(runner, tools, &path));
            if !audible || source_end <= source_start || end <= start {
                continue;
            }
            let speed = match element.get("speed") {
                None | Some(Value::Null) => Decimal::of(&serde_json::Number::from(1)),
                Some(value) => value.as_number().and_then(Decimal::of),
            }
            .filter(|speed| speed.is_positive());
            let Some(speed) = speed else { continue };
            let Some(played) = exact::played_ms(source_end - source_start, speed) else {
                continue;
            };
            let looping = element.get("overrun").and_then(Value::as_str) == Some("loop");
            // Without a loop the source's audio ends where the source does, whatever `end`
            // says (ADR-0020's `hold` freezes a picture, never a sample).
            let until = if looping {
                end
            } else {
                end.min(start + played)
            };
            let (from, to) = (start.max(0), until.min(extent));
            if to <= from {
                continue;
            }
            let (spans, peak_volume) = match audible_spans(element, from, to, fps) {
                Some(found) => found,
                None => continue,
            };
            if spans.is_empty() {
                continue;
            }
            contributions.push(Contribution {
                element: name,
                source: path,
                spans,
                start,
                source_start,
                source_end,
                speed: speed.as_f64(),
                period: looping.then_some(played),
                peak_volume,
            });
        }
        Heard { contributions }
    }

    /// The union of every contribution's spans, merged and sorted.
    fn union(&self) -> Vec<(i64, i64)> {
        merge(
            self.contributions
                .iter()
                .flat_map(|c| c.spans.iter().copied())
                .collect(),
        )
    }
}

/// The spans of `[from, to)` in which the element's resolved `volume` is above 0, and the
/// largest value it takes there. `None` where the volume does not resolve — the check engine
/// refuses such a document, and this verb does not guess at one.
fn audible_spans(element: &Value, from: i64, to: i64, fps: i64) -> Option<(Vec<(i64, i64)>, f64)> {
    let volume: Animatable<f64> = match element.get("volume") {
        None | Some(Value::Null) => Animatable::Static(1.0),
        Some(written) => {
            match serde_json::from_value::<Animatable<Volume>>(written.clone()).ok()? {
                Animatable::Static(Volume(v)) => Animatable::Static(v),
                Animatable::Keyed(records) => Animatable::Keyed(
                    records
                        .into_iter()
                        .map(|record| Keyframe {
                            t: record.t,
                            t_from: None,
                            v: record.v.0,
                            ease: record.ease,
                        })
                        .collect(),
                ),
            }
        }
    };
    if let Animatable::Static(v) = volume {
        return Some(if v > 0.0 {
            (vec![(from, to)], v)
        } else {
            (Vec::new(), 0.0)
        });
    }
    // Per frame on ADR-0035's grid, the same instants `render` resolves a keyframed volume at.
    let first = exact::frame_at_or_after(from, fps)?.frame;
    let mut spans = Vec::new();
    let mut peak: f64 = 0.0;
    let mut n = first;
    loop {
        let instant = crate::exact::instant_of(n, fps);
        if instant >= to {
            break;
        }
        let next = crate::exact::instant_of(n + 1, fps).min(to);
        let v = resolve::at(&volume, instant).ok()?;
        if v > 0.0 {
            peak = peak.max(v);
            spans.push((instant.max(from), next));
        }
        n += 1;
    }
    Some((merge(spans), peak))
}

fn merge(mut spans: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
    spans.sort();
    let mut out: Vec<(i64, i64)> = Vec::new();
    for (a, b) in spans {
        match out.last_mut() {
            Some(last) if a <= last.1 => last.1 = last.1.max(b),
            _ => out.push((a, b)),
        }
    }
    out
}

fn intersect(a: &[(i64, i64)], b: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    for &(a0, a1) in a {
        for &(b0, b1) in b {
            let (lo, hi) = (a0.max(b0), a1.min(b1));
            if hi > lo {
                out.push((lo, hi));
            }
        }
    }
    merge(out)
}

fn source_has_audio(runner: &dyn Runner, tools: &Tools, source: &Path) -> bool {
    csv(
        runner,
        tools,
        source,
        &["-select_streams", "a", "-show_entries", "stream=index"],
    )
    .map(|rows| rows.iter().any(|row| !row.concat().is_empty()))
    .unwrap_or(false)
}

// ---- Silent spans ---------------------------------------------------------------------------

/// Every stretch of at least one R128 block where the mix is below the absolute gate inside
/// a span that should be heard, as one `review` finding each, with the census split by each
/// source's own energy over what it contributes there.
fn silent_spans(
    mix: &[Reading],
    spans: &[(i64, i64)],
    heard: &Heard,
    runner: &dyn Runner,
    tools: &Tools,
) -> Vec<Finding> {
    let silent: Vec<(i64, i64)> = merge(
        mix.iter()
            .filter(|reading| reading.lufs < GATE_LUFS)
            .map(|reading| (reading.end_ms - BLOCK_MS, reading.end_ms))
            .collect(),
    );
    let mut sources: BTreeMap<PathBuf, Option<Vec<Reading>>> = BTreeMap::new();
    let mut findings = Vec::new();
    for (from, to) in intersect(&silent, spans) {
        if to - from < BLOCK_MS {
            continue;
        }
        let quietest = mix
            .iter()
            .filter(|r| r.end_ms - BLOCK_MS >= from && r.end_ms <= to)
            .map(|r| r.lufs)
            .fold(f64::INFINITY, f64::min);

        let mut own_silence = Vec::new();
        let mut engine = Vec::new();
        let mut unmeasured = Vec::new();
        for contribution in &heard.contributions {
            let here = intersect(&contribution.spans, &[(from, to)]);
            if here.is_empty() {
                continue;
            }
            let readings = sources
                .entry(contribution.source.clone())
                .or_insert_with(|| loudness(runner, tools, &contribution.source).ok());
            match readings {
                Some(readings) => {
                    if audible_in_source(contribution, &here, readings) {
                        engine.push(contribution.element.clone());
                    } else {
                        own_silence.push(contribution.element.clone());
                    }
                }
                None => unmeasured.push(contribution.element.clone()),
            }
        }
        let alone = own_silence.len() + engine.len() + unmeasured.len() == 1;
        let mut census = Census::on("source_energy");
        if !own_silence.is_empty() {
            census = census.group(json!("silent in its own source"), own_silence);
        }
        if !engine.is_empty() {
            census = census.group(json!("audible in its own source"), engine);
        }
        if !unmeasured.is_empty() {
            census = census.group(json!("source not measurable"), unmeasured);
        }
        findings.push(
            Finding::new("R-VERIFY-SILENT-SPAN")
                .field("from", json!(from))
                .field("to", json!(to))
                .field("quietest_lufs", json!(quietest))
                .field("gate_lufs", json!(GATE_LUFS))
                .field("block_ms", json!(BLOCK_MS))
                .field(
                    "attribution",
                    json!(if alone {
                        "exact: one element should be heard here"
                    } else {
                        "shared: more than one element should be heard here"
                    }),
                )
                .census(census)
                .citation(Citation {
                    threshold: json!(GATE_LUFS),
                    source: R128.to_string(),
                    adr: "ADR-0117".to_string(),
                }),
        );
    }
    findings
}

/// Whether the source is audible over what it contributes to `here`, at its resolved volume.
///
/// The clock maps back onto the source at the element's `speed`; a looping element that wraps
/// inside `here` is judged over its whole source span, which is the honest reading of a stretch
/// that plays every part of it.
fn audible_in_source(
    contribution: &Contribution,
    here: &[(i64, i64)],
    readings: &[Reading],
) -> bool {
    let gain = 20.0 * contribution.peak_volume.max(f64::MIN_POSITIVE).log10();
    here.iter().any(|&(from, to)| {
        let local = |t: i64| (t - contribution.start) as f64;
        let (mut s0, mut s1) = (local(from), local(to));
        if let Some(period) = contribution.period {
            let period = period as f64;
            if s1 - s0 >= period || (s0 / period).floor() != ((s1 - 1.0) / period).floor() {
                (s0, s1) = (0.0, period);
            } else {
                let base = (s0 / period).floor() * period;
                (s0, s1) = (s0 - base, s1 - base);
            }
        }
        let a = contribution.source_start as f64 + s0 * contribution.speed;
        let b = (contribution.source_start as f64 + s1 * contribution.speed)
            .min(contribution.source_end as f64);
        let centre = |r: &Reading| (r.end_ms - BLOCK_MS / 2) as f64;
        let inside: Vec<&Reading> = readings
            .iter()
            .filter(|r| centre(r) >= a && centre(r) <= b)
            .collect();
        let judged: Vec<&Reading> = if inside.is_empty() {
            // Shorter than one reading's spacing: the nearest block is the measurement.
            readings
                .iter()
                .min_by(|x, y| {
                    (centre(x) - (a + b) / 2.0)
                        .abs()
                        .total_cmp(&(centre(y) - (a + b) / 2.0).abs())
                })
                .into_iter()
                .collect()
        } else {
            inside
        };
        judged.iter().any(|r| r.lufs + gain >= GATE_LUFS)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ebur128_log_line_is_one_block_ending_at_its_timestamp() {
        let line = "[Parsed_ebur128_0 @ 0x7af6c3c900] t: 0.399979   TARGET:-23 LUFS    \
                    M: -21.7 S:-120.7     I: -21.7 LUFS       LRA:   0.0 LU";
        let reading = parse_reading(line).expect("a reading");
        assert_eq!(reading.end_ms, 400);
        assert_eq!(reading.lufs, -21.7);
        let placeholder =
            "[Parsed_ebur128_0 @ 0x1] t: 1.4999   TARGET:-23 LUFS    M:-160.7 S:-120.7";
        assert_eq!(parse_reading(placeholder).unwrap().lufs, -160.7);
    }

    /// A block of exact digital silence is `M:   nan` in the meter's log — what an AAC mix
    /// decodes to on some builds (aarch64 Linux, x86_64 macOS) where others leave a residue
    /// near −160. It is the quietest reading there is, never one above the gate.
    #[test]
    fn a_nan_reading_is_digital_silence_and_below_the_gate() {
        let line = "[Parsed_ebur128_0 @ 0x7fa43c001b80] t: 2.099979   TARGET:-23 LUFS    \
                    M:   nan S:-120.7     I: -22.2 LUFS       LRA:   0.0 LU";
        let reading = parse_reading(line).expect("a reading");
        assert_eq!(reading.end_ms, 2100);
        assert!(reading.lufs < GATE_LUFS, "{}", reading.lufs);
    }

    #[test]
    fn spans_merge_where_they_touch_and_intersect_where_they_overlap() {
        assert_eq!(
            merge(vec![(400, 800), (0, 400), (900, 1000)]),
            vec![(0, 800), (900, 1000)]
        );
        assert_eq!(
            intersect(&[(0, 1000)], &[(200, 300), (900, 1200)]),
            vec![(200, 300), (900, 1000)]
        );
    }

    #[test]
    fn a_stamp_of_either_grammar_reads_back_its_project_and_only_the_second_its_digest() {
        let v1 = attest::Stamp::parse("montagent/1 project=/a b/p.montagent.json").unwrap();
        assert_eq!((v1.version, v1.digest), (1, None));
        assert_eq!(v1.project, "/a b/p.montagent.json");
        let v2 = attest::Stamp::parse(
            "montagent/2 engine=0.1.0 digest=abc project=/a b/p.montagent.json",
        )
        .unwrap();
        assert_eq!(v2.digest.as_deref(), Some("abc"));
        assert_eq!(v2.project, "/a b/p.montagent.json");
        let none = attest::Stamp::parse("montagent/2 engine=0.1.0 digest=none project=/p").unwrap();
        assert_eq!(none.digest, None);
        assert_eq!(attest::Stamp::parse("made with another tool"), None);
    }
}
