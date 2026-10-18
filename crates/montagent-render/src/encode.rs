//! The encode path: raw frames in, one MP4 out, through an `ffmpeg` Montagent spawns.
//!
//! **Spawned, never linked** (ADR-0009): `libx264` is GPL and `libavcodec` linked into the
//! binary would make it a GPL combined work, so every frame crosses a pipe to a separate
//! program. The path to that program is handed in, resolved once by
//! `montagent_core::media::tools` for every verb that spawns one — this crate resolves
//! nothing.
//!
//! **The deliverable is written to a sibling temp path and renamed into place** (ADR-0011:
//! *"a truncated MP4 at the deliverable path reads as finished"*). [`Deliverable`] is that
//! rule as a type: the declared path is never opened for writing, the bytes land in a
//! dotted sibling, and one `rename` publishes them — atomic on every filesystem the six
//! targets run on, and a sibling rather than the system temp directory because `rename`
//! across filesystems fails and a project on an external disk is the normal case for video
//! work. A failure anywhere before the rename takes the temp file with it and leaves the
//! declared path exactly as it was.
//!
//! **Nothing here reads the document.** The frame size, the rate, the span and the audio
//! graph arrive already decided; which elements are audible, at what volume, through what
//! `atempo` chain — every one of those is a rule about the format and lives in
//! `montagent-core` (spec #168's crate split).
//!
//! ## Three encoder choices, ratified by ADR-0077 (#287)
//!
//! Each shipped with #215 and no ADR behind it. ADR-0077 ratifies all three as they are —
//! the settings amending ADR-0009, the padding amending ADR-0021 — and notes that CRF 20
//! at preset `medium` is a default rather than a measured optimum.
//!
//! - **`libx264`, `yuv420p`, CRF 20, `medium`.** H.264 in an MP4 with `+faststart` is what
//!   every player and every upload form reads. An `ffmpeg` built without `--enable-gpl` has
//!   no `libx264`, and that surfaces as exit 70 carrying `ffmpeg`'s own sentence rather than
//!   a silent fallback to a lesser encoder (ADR-0009 names `libopenh264` as the escape route
//!   and does not take it).
//! - **An odd frame dimension is padded to even, on the right and bottom, in the project's
//!   background colour, and the answer says so.** `yuv420p` cannot carry an odd width or
//!   height at all, so the choice is between refusing a legal project and adding one row or
//!   column of background — and ADR-0021's rule is that `render` never *silently* changes
//!   what was rendered. Padded, and disclosed as [`Finished::encoded`] beside the declared
//!   frame.
//! - **AAC at 160 kb/s, 48 kHz stereo** for the mix bus, which is the rate the mix graph is
//!   built at so that every sample count in it is exact without a probe.
//!
//! ## One thread count, pinned (ADR-0143, #628)
//!
//! ADR-0143 keeps the three choices above, now measured, and adds a fourth: libx264 is
//! always given **`-threads` [`THREADS`]**, never `ffmpeg`'s automatic count (1.5x the
//! CPUs). x264 is deterministic for one input, settings and thread count but not across
//! thread counts, so the automatic count made the same project write different bytes on
//! machines with different core counts, and left the encoder's share of the cores unbounded.
//! [`Settings`] is the whole set, [`Settings::PRODUCTION`] is the only one `render` and
//! `preview` use, and the answer discloses it. There is no project field and no CLI or MCP
//! option for any of it; a test that needs another thread count builds its own `Settings`.

use std::io::Write;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

use crate::canvas::Rgba;

/// The libx264 thread count every deliverable is encoded with (ADR-0143).
///
/// The smallest count for which the encode stayed off the critical path of the benchmark
/// project on the dev's M1 Pro. Changing it changes the bytes of every MP4 Montagent
/// writes, so it moves only with a superseding ADR. Non-zero by type, because `-threads 0`
/// is how `ffmpeg` spells *"pick for me"*, which is what this constant exists to rule out.
pub const THREADS: NonZeroU32 = match NonZeroU32::new(5) {
    Some(n) => n,
    None => panic!("a thread count of zero is ffmpeg's automatic count"),
};

/// The video encoder's settings: what the encode is asked for, and what the answer says
/// was used (ADR-0077, ADR-0143).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub encoder: &'static str,
    pub preset: Option<&'static str>,
    pub crf: Option<u8>,
    pub threads: NonZeroU32,
    pub profile: Option<String>,
    pub pix_fmt: Option<String>,
}

impl Settings {
    /// The one set `render` and `preview` encode with. Not reachable from a project, the
    /// CLI or MCP (ADR-0143 §5).
    pub const PRODUCTION: Settings = Settings {
        encoder: crate::floor::VIDEO_ENCODER,
        preset: Some("medium"),
        crf: Some(20),
        threads: THREADS,
        profile: None,
        pix_fmt: None,
    };

    /// The output-side video arguments these settings become, in the order `ffmpeg` is
    /// given them. `-threads` follows `-c:v`, so it is the encoder's thread count and not
    /// the raw input's.
    pub fn args(&self) -> Vec<String> {
        let mut args = vec![
            "-c:v".to_string(),
            self.encoder.to_string(),
        ];
        if let Some(preset) = self.preset {
            args.extend(["-preset".to_string(), preset.to_string()]);
        }
        if let Some(crf) = self.crf {
            args.extend(["-crf".to_string(), crf.to_string()]);
        }
        args.extend(["-threads".to_string(), self.threads.to_string()]);
        args
    }
}

/// An exact rational frame rate for video encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rate {
    pub num: i64,
    pub den: i64,
}

impl Rate {
    pub const fn integer(fps: i64) -> Self {
        Self { num: fps, den: 1 }
    }

    pub const fn new(num: i64, den: i64) -> Self {
        Self { num, den }
    }
}

impl From<i64> for Rate {
    fn from(fps: i64) -> Self {
        Self::integer(fps)
    }
}

impl From<(i64, i64)> for Rate {
    fn from((num, den): (i64, i64)) -> Self {
        Self::new(num, den)
    }
}

impl From<[i64; 2]> for Rate {
    fn from([num, den]: [i64; 2]) -> Self {
        Self::new(num, den)
    }
}

impl std::fmt::Display for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// What one encode is asked to produce.
#[derive(Debug, Clone)]
pub struct Spec {
    /// The project's declared frame, at true pixels. Never proxy-scaled (ADR-0021).
    pub width: u32,
    pub height: u32,
    pub fps: Rate,
    /// The colour an odd dimension is padded with.
    pub background: Rgba,
    /// The audio mix, where the project has anything audible.
    pub audio: Option<Audio>,
    /// What to stamp the container with, so a later run can tell whose file this is
    /// (ADR-0104). `None` writes no stamp: a **preview** is not a deliverable and must not
    /// leave an attestation a later `render` would read as its own.
    pub stamp: Option<String>,
    /// The video encoder's settings: [`Settings::PRODUCTION`] everywhere but a test.
    pub settings: Settings,
    /// Target container override: "mp4", "mov", "mxf". If None, inferred from output file extension.
    pub container: Option<String>,
    /// Video codec name: "h264", "prores", "dnxhr". Default is "h264".
    pub video_codec: Option<String>,
    /// Video profile name (e.g. "4444", "standard", "dnxhr_hq").
    pub video_profile: Option<String>,
    /// Audio codec name: "aac", "pcm".
    pub audio_codec: Option<String>,
    /// Audio sample rate in Hz (e.g. 48000).
    pub audio_sample_rate: Option<u32>,
    /// Audio bitrate (e.g. "160k").
    pub audio_bitrate: Option<String>,
    /// Whether alpha transparency is preserved in the video stream (ADR-0203).
    pub alpha: bool,
}

impl Spec {
    /// A new basic Spec with production settings, defaulting to H.264 MP4 without alpha.
    pub fn new(width: u32, height: u32, fps: Rate, background: Rgba) -> Self {
        Self {
            width,
            height,
            fps,
            background,
            audio: None,
            stamp: None,
            settings: Settings::PRODUCTION,
            container: None,
            video_codec: None,
            video_profile: None,
            audio_codec: None,
            audio_sample_rate: None,
            audio_bitrate: None,
            alpha: false,
        }
    }

    /// Assemble the output-side and input-side arguments for FFmpeg (ADR-0077, ADR-0143, ADR-0203).
    pub fn ffmpeg_args(
        &self,
        target_path: &Path,
        output_temp: &Path,
        encoded: Option<Encoded>,
        script: Option<&Path>,
    ) -> Vec<String> {
        let mut args = vec![
            "-hide_banner".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
            "-y".to_string(),
        ];
        let raw_pix_fmt = if self.alpha { "rgba" } else { "rgb24" };
        args.extend([
            "-f".to_string(),
            "rawvideo".to_string(),
            "-pix_fmt".to_string(),
            raw_pix_fmt.to_string(),
            "-s".to_string(),
            format!("{}x{}", self.width, self.height),
            "-r".to_string(),
            self.fps.to_string(),
            "-i".to_string(),
            "pipe:0".to_string(),
        ]);
        if let Some(audio) = &self.audio {
            for input in &audio.inputs {
                args.push("-i".to_string());
                args.push(input.display().to_string());
            }
        }
        if let Some(script) = script {
            args.push(crate::floor::FILTER_COMPLEX_FILE.to_string());
            args.push(script.display().to_string());
        }
        args.extend(["-map".to_string(), "0:v".to_string()]);
        if let Some(Encoded { width, height }) = encoded {
            let pad_color = if self.alpha {
                "0x00000000".to_string()
            } else {
                let Rgba([r, g, b, _]) = self.background;
                format!("0x{r:02X}{g:02X}{b:02X}")
            };
            args.extend([
                "-vf".to_string(),
                format!("pad={width}:{height}:0:0:color={pad_color}"),
            ]);
        }

        let video_codec = self.video_codec.as_deref().unwrap_or("h264");
        match video_codec {
            "prores" => {
                let profile = self.video_profile.as_deref().unwrap_or("standard");
                args.extend([
                    "-c:v".to_string(),
                    "prores_ks".to_string(),
                    "-profile:v".to_string(),
                    profile.to_string(),
                    "-vendor".to_string(),
                    "apl0".to_string(),
                ]);
                let out_pix_fmt = if self.alpha {
                    "yuva444p10le"
                } else if profile == "4444" || profile == "4444xq" {
                    "yuv444p10le"
                } else {
                    "yuv422p10le"
                };
                args.extend([
                    "-pix_fmt".to_string(),
                    out_pix_fmt.to_string(),
                    "-r".to_string(),
                    self.fps.to_string(),
                ]);
            }
            "dnxhr" => {
                let profile = self.video_profile.as_deref().unwrap_or("dnxhr_hq");
                let lower = profile.to_ascii_lowercase();
                let normalized_profile = match lower.as_str() {
                    "dnxhr_lb" | "lb" => "dnxhr_lb",
                    "dnxhr_sq" | "sq" => "dnxhr_sq",
                    "dnxhr_hq" | "hq" => "dnxhr_hq",
                    "dnxhr_hqx" | "hqx" => "dnxhr_hqx",
                    "dnxhr_444" | "444" => "dnxhr_444",
                    _ => profile,
                };
                args.extend([
                    "-c:v".to_string(),
                    "dnxhd".to_string(),
                    "-profile:v".to_string(),
                    normalized_profile.to_string(),
                ]);
                let out_pix_fmt = match normalized_profile {
                    "dnxhr_444" => "yuv444p10le",
                    "dnxhr_hqx" => "yuv422p10le",
                    _ => "yuv422p",
                };
                args.extend([
                    "-pix_fmt".to_string(),
                    out_pix_fmt.to_string(),
                    "-r".to_string(),
                    self.fps.to_string(),
                ]);
            }
            _ => {
                args.extend(self.settings.args());
                if let Some(profile) = self.video_profile.as_deref() {
                    args.extend(["-profile:v".to_string(), profile.to_string()]);
                }
                args.extend([
                    "-pix_fmt".to_string(),
                    "yuv420p".to_string(),
                    "-r".to_string(),
                    self.fps.to_string(),
                ]);
            }
        }

        if self.audio.is_some() {
            args.extend(["-map".to_string(), "[mix]".to_string()]);
            let audio_codec = self.audio_codec.as_deref().unwrap_or(
                if video_codec == "prores" || video_codec == "dnxhr" {
                    "pcm"
                } else {
                    "aac"
                },
            );
            if audio_codec == "pcm" {
                args.extend(["-c:a".to_string(), "pcm_s16le".to_string()]);
            } else {
                args.extend(["-c:a".to_string(), "aac".to_string()]);
                let bitrate = self.audio_bitrate.as_deref().unwrap_or("160k");
                args.extend(["-b:a".to_string(), bitrate.to_string()]);
            }
            if let Some(sr) = self.audio_sample_rate {
                args.extend(["-ar".to_string(), sr.to_string()]);
            }
        } else {
            args.push("-an".to_string());
        }

        if let Some(stamp) = &self.stamp {
            args.extend(["-metadata".to_string(), format!("comment={stamp}")]);
        }

        let container = self.container.clone().unwrap_or_else(|| {
            target_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| match s.to_ascii_lowercase().as_str() {
                    "mov" => "mov".to_string(),
                    "mxf" => "mxf".to_string(),
                    _ => "mp4".to_string(),
                })
                .unwrap_or_else(|| "mp4".to_string())
        });

        match container.as_str() {
            "mov" => {
                args.extend([
                    "-movflags".to_string(),
                    "+faststart".to_string(),
                    "-f".to_string(),
                    "mov".to_string(),
                ]);
            }
            "mxf" => {
                args.extend(["-f".to_string(), "mxf".to_string()]);
            }
            _ => {
                args.extend([
                    "-movflags".to_string(),
                    "+faststart".to_string(),
                    "-f".to_string(),
                    "mp4".to_string(),
                ]);
            }
        }

        args.push(output_temp.display().to_string());
        args
    }
}

/// One audio mix: the files to open, in input order, and the filter graph that turns
/// their streams into one `[mix]`.
///
/// The graph's input labels are `[1:a]`, `[2:a]`, … — input `0` is the video on stdin —
/// and it must end by labelling one stream `[mix]`, **already cut to the span the video
/// covers**: the encoder applies no `-t` of its own, because `ffmpeg` drops a video frame
/// whose display interval crosses the limit and the last frame of every render would go
/// with it. The video's length is the frames pushed; the audio's is the graph's. It goes
/// to `ffmpeg` as a script file rather than an argument, so a project with a keyframed
/// `volume` on twenty narration elements cannot run into an argv limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Audio {
    pub inputs: Vec<PathBuf>,
    pub graph: String,
}

/// The frame the encoder actually wrote, where padding made it differ from the declared
/// one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Encoded {
    pub width: u32,
    pub height: u32,
}

/// What an encode came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finished {
    pub path: PathBuf,
    pub frames: u64,
    pub bytes: u64,
    /// `Some` only where the declared frame was padded to even.
    pub encoded: Option<Encoded>,
    /// The settings the file was encoded with.
    pub settings: Settings,
}

/// A complete encode that has not been published.
///
/// Every number the answer needs is already known — the whole span is in the temp file —
/// and the one thing left undecided is whether that file becomes the deliverable. ADR-0093
/// ruling 6 puts that decision with the caller that holds the report.
///
/// Dropping a `Sealed` without publishing removes the temp file and leaves the declared
/// path untouched, which is [`Deliverable`]'s own `Drop` and not a second implementation of
/// it: *declining to promote is the absence of a promotion, not destruction.*
pub struct Sealed {
    deliverable: Deliverable,
    frames: u64,
    bytes: u64,
    encoded: Option<Encoded>,
    settings: Settings,
}

impl Sealed {
    /// How many frames are in the file, for a caller reporting on a span it will not publish.
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// The path the file *would* land at. It does not exist yet.
    pub fn target(&self) -> &Path {
        self.deliverable.target()
    }

    pub fn encoded(&self) -> Option<Encoded> {
        self.encoded
    }

    /// One rename, and the declared path exists for the first time — complete.
    pub fn publish(mut self) -> Result<Finished, String> {
        let path = self.deliverable.commit()?;
        Ok(Finished {
            path,
            frames: self.frames,
            bytes: self.bytes,
            encoded: self.encoded,
            settings: self.settings,
        })
    }

    /// Walk away: the temp file goes, and the declared path was never touched.
    pub fn withhold(self) {
        // `Deliverable`'s `Drop` does the work, so there is exactly one implementation.
    }
}

/// A running `ffmpeg`, fed one raw RGB8 frame at a time.
pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    stderr: Option<std::thread::JoinHandle<String>>,
    /// `None` once [`Encoder::seal`] has handed it to a [`Sealed`], which is then the
    /// thing that decides whether it is published.
    deliverable: Option<Deliverable>,
    script: Option<PathBuf>,
    frame_bytes: usize,
    frames: u64,
    encoded: Option<Encoded>,
    settings: Settings,
}

impl Encoder {
    /// Spawn the encoder, writing to a temp sibling of `output`.
    ///
    /// The parent directory is created if it is missing: the project's `output` is
    /// `out/<name>.mp4` by convention, and `out/` not existing yet is the first run rather
    /// than a mistake.
    pub fn start(ffmpeg: &Path, output: &Path, spec: &Spec) -> Result<Encoder, String> {
        if spec.width == 0 || spec.height == 0 {
            return Err(format!(
                "a {}x{} frame is no frame at all",
                spec.width, spec.height
            ));
        }
        if spec.fps.num <= 0 || spec.fps.den <= 0 {
            return Err(format!("{} frames per second is not a rate", spec.fps));
        }
        let deliverable = Deliverable::begin(output)?;

        // Even dimensions, or the encoder refuses the frame. Padded rather than cropped so
        // no pixel the project declared is lost, and disclosed in the answer.
        let even = |n: u32| n + (n % 2);
        let (out_width, out_height) = (even(spec.width), even(spec.height));
        let encoded = (out_width != spec.width || out_height != spec.height).then_some(Encoded {
            width: out_width,
            height: out_height,
        });

        let script = match &spec.audio {
            Some(audio) => {
                let path = deliverable.sibling("filters");
                std::fs::write(&path, &audio.graph)
                    .map_err(|e| format!("{} could not be written: {e}", path.display()))?;
                Some(path)
            }
            None => None,
        };

        let mut command = Command::new(ffmpeg);
        command.args(spec.ffmpeg_args(output, deliverable.temp(), encoded, script.as_deref()));
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                deliverable.abandon();
                if let Some(script) = &script {
                    let _ = std::fs::remove_file(script);
                }
                return Err(format!("{} could not be run: {e}", ffmpeg.display()));
            }
        };
        let stdin = child.stdin.take();
        // Drained on its own thread: an `ffmpeg` that has something to say and nobody
        // reading would block on a full pipe while this process blocks writing frames to
        // it, and the two would wait for each other until the caller gave up.
        let stderr = child.stderr.take().map(|mut pipe| {
            std::thread::spawn(move || {
                let mut said = String::new();
                let _ = std::io::Read::read_to_string(&mut pipe, &mut said);
                said
            })
        });

        let bytes_per_pixel = if spec.alpha { 4 } else { 3 };

        let video_codec = spec.video_codec.as_deref().unwrap_or("h264");
        let settings = match video_codec {
            "prores" => {
                let profile = spec.video_profile.as_deref().unwrap_or("standard");
                let pix_fmt = if spec.alpha {
                    "yuva444p10le"
                } else if profile == "4444" || profile == "4444xq" {
                    "yuv444p10le"
                } else {
                    "yuv422p10le"
                };
                Settings {
                    encoder: "prores_ks",
                    preset: None,
                    crf: None,
                    threads: spec.settings.threads,
                    profile: Some(profile.to_string()),
                    pix_fmt: Some(pix_fmt.to_string()),
                }
            }
            "dnxhr" => {
                let profile = spec.video_profile.as_deref().unwrap_or("dnxhr_hq");
                let lower = profile.to_ascii_lowercase();
                let normalized_profile = match lower.as_str() {
                    "dnxhr_lb" | "lb" => "dnxhr_lb",
                    "dnxhr_sq" | "sq" => "dnxhr_sq",
                    "dnxhr_hq" | "hq" => "dnxhr_hq",
                    "dnxhr_hqx" | "hqx" => "dnxhr_hqx",
                    "dnxhr_444" | "444" => "dnxhr_444",
                    _ => profile,
                };
                let out_pix_fmt = match normalized_profile {
                    "dnxhr_444" => "yuv444p10le",
                    "dnxhr_hqx" => "yuv422p10le",
                    _ => "yuv422p",
                };
                Settings {
                    encoder: "dnxhd",
                    preset: None,
                    crf: None,
                    threads: spec.settings.threads,
                    profile: Some(normalized_profile.to_string()),
                    pix_fmt: Some(out_pix_fmt.to_string()),
                }
            }
            _ => {
                let mut s = spec.settings.clone();
                if s.profile.is_none() {
                    s.profile = spec.video_profile.clone();
                }
                if s.pix_fmt.is_none() {
                    s.pix_fmt = Some(if spec.alpha { "yuva420p" } else { "yuv420p" }.to_string());
                }
                s
            }
        };

        Ok(Encoder {
            child,
            stdin,
            stderr,
            deliverable: Some(deliverable),
            script,
            frame_bytes: spec.width as usize * spec.height as usize * bytes_per_pixel,
            frames: 0,
            encoded,
            settings,
        })
    }

    /// Hand over one frame, as packed RGB8 at the declared size.
    pub fn push(&mut self, rgb: &[u8]) -> Result<(), String> {
        if rgb.len() != self.frame_bytes {
            return Err(format!(
                "a frame of {} bytes was handed to an encoder expecting {}",
                rgb.len(),
                self.frame_bytes
            ));
        }
        let Some(stdin) = self.stdin.as_mut() else {
            return Err("the encoder's input is already closed".to_string());
        };
        if let Err(e) = stdin.write_all(rgb) {
            // A write that fails is `ffmpeg` having exited, and what it said on the way
            // out is the sentence the caller can act on.
            let said = self.said();
            return Err(if said.is_empty() {
                format!("ffmpeg stopped accepting frames after {}: {e}", self.frames)
            } else {
                format!(
                    "ffmpeg stopped accepting frames after {}: {said}",
                    self.frames
                )
            });
        }
        self.frames += 1;
        Ok(())
    }

    /// Close the input, wait for the encoder, and publish the file.
    ///
    /// [`Encoder::seal`] then [`Sealed::publish`], so that a caller who has to decide
    /// *whether* to publish and a caller who does not share one implementation of the
    /// encode's ending.
    pub fn finish(self) -> Result<Finished, String> {
        self.seal()?.publish()
    }

    /// Close the input and wait for the encoder, stopping **short of publishing**.
    ///
    /// ADR-0093 ruling 6: `render` does not promote the deliverable when any `error`-class
    /// finding fired, and the invariant it buys is that *a file at the output path is a
    /// render with zero errors.* That decision is the caller's — it is the one holding the
    /// report — so the encode's ending is two steps, and the temp file is still
    /// [`Deliverable`]'s to remove if the caller walks away from it.
    pub fn seal(mut self) -> Result<Sealed, String> {
        drop(self.stdin.take());
        let status = self.child.wait();
        let said = self.said();
        if let Some(script) = self.script.take() {
            let _ = std::fs::remove_file(script);
        }
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => {
                return Err(format!(
                    "ffmpeg exited with {status}{}",
                    if said.is_empty() {
                        String::new()
                    } else {
                        format!(": {said}")
                    }
                ));
            }
            Err(e) => return Err(format!("ffmpeg could not be waited for: {e}")),
        }
        let deliverable = self
            .deliverable
            .take()
            .ok_or("the encode was already sealed")?;
        let bytes = std::fs::metadata(deliverable.temp())
            .map(|m| m.len())
            .unwrap_or(0);
        Ok(Sealed {
            deliverable,
            frames: self.frames,
            bytes,
            encoded: self.encoded,
            settings: self.settings.clone(),
        })
    }

    /// Everything `ffmpeg` wrote to stderr so far, trimmed. Kills the process first if it is
    /// still running, since a still-running encoder is one that will never close the pipe.
    fn said(&mut self) -> String {
        drop(self.stdin.take());
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        self.stderr
            .take()
            .and_then(|thread| thread.join().ok())
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

impl Drop for Encoder {
    /// An encoder dropped before [`Encoder::finish`] is a render that did not complete, and
    /// the temp file goes with it. The declared path was never touched.
    fn drop(&mut self) {
        drop(self.stdin.take());
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(script) = self.script.take() {
            let _ = std::fs::remove_file(script);
        }
        // The temp file is `Deliverable`'s to remove, and its own `Drop` does so next.
    }
}

/// A file that will exist at its declared path only once it is complete.
pub struct Deliverable {
    target: PathBuf,
    temp: PathBuf,
    committed: bool,
}

impl Deliverable {
    /// Reserve a temp sibling of `target`, creating the parent directory if it is missing.
    pub fn begin(target: &Path) -> Result<Deliverable, String> {
        let directory = target.parent().filter(|p| !p.as_os_str().is_empty());
        if let Some(directory) = directory {
            std::fs::create_dir_all(directory)
                .map_err(|e| format!("{} could not be created: {e}", directory.display()))?;
        }
        let stem = target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| format!("{} names no file", target.display()))?;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp = directory.unwrap_or_else(|| Path::new("")).join(format!(
            ".{stem}.montagent-partial-{}-{nanos}",
            std::process::id()
        ));
        Ok(Deliverable {
            target: target.to_path_buf(),
            temp,
            committed: false,
        })
    }

    /// Where the bytes are going for now.
    /// Where the file lands once it is published.
    pub fn target(&self) -> &Path {
        &self.target
    }

    pub fn temp(&self) -> &Path {
        &self.temp
    }

    /// Another scratch file beside the temp one, for the encoder's own use.
    fn sibling(&self, suffix: &str) -> PathBuf {
        let mut name = self.temp.as_os_str().to_owned();
        name.push(".");
        name.push(suffix);
        PathBuf::from(name)
    }

    /// Publish: one rename, and the declared path exists for the first time — complete.
    ///
    /// `&mut self` rather than `self` because [`Encoder`] owns one and implements `Drop`;
    /// after a commit there is nothing left for either drop to remove.
    pub fn commit(&mut self) -> Result<PathBuf, String> {
        std::fs::rename(&self.temp, &self.target).map_err(|e| {
            let _ = std::fs::remove_file(&self.temp);
            format!(
                "{} could not be moved into place at {}: {e}",
                self.temp.display(),
                self.target.display()
            )
        })?;
        self.committed = true;
        Ok(self.target.clone())
    }

    /// Give up: the temp file is removed and the declared path is untouched.
    pub fn abandon(self) {
        // `Drop` does the work, so there is exactly one implementation of it.
    }

    fn remove_temp(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.temp);
        }
    }
}

impl Drop for Deliverable {
    fn drop(&mut self) {
        self.remove_temp();
    }
}

/// Specifications for exporting an audio stem (ADR-0204).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioStemSpec {
    pub codec: String,
    pub sample_rate: u32,
    pub duration_ms: i64,
}

impl Default for AudioStemSpec {
    fn default() -> Self {
        Self {
            codec: "pcm_s24le".to_string(),
            sample_rate: 48000,
            duration_ms: 0,
        }
    }
}

/// What an audio stem export came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedAudio {
    pub path: PathBuf,
    pub bytes: u64,
}

/// Export an audio stem to a Broadcast WAV file using FFmpeg (ADR-0204).
pub fn export_audio(
    ffmpeg: &Path,
    output: &Path,
    audio: Option<&Audio>,
    spec: &AudioStemSpec,
) -> Result<FinishedAudio, String> {
    let mut deliverable = Deliverable::begin(output)?;

    let mut command = Command::new(ffmpeg);
    command.arg("-y");

    let script = if let Some(audio) = audio {
        if !audio.inputs.is_empty() && !audio.graph.trim().is_empty() {
            // Input 0: dummy null audio so graph input indices [1:a], [2:a] match exactly
            command.args(["-f", "lavfi", "-i", "anullsrc=r=48000:cl=stereo"]);
            for input in &audio.inputs {
                command.arg("-i").arg(input);
            }
            let script_path = deliverable.sibling("montagent-stem-filtergraph");
            if let Err(e) = std::fs::write(&script_path, &audio.graph) {
                return Err(format!(
                    "{} could not be written: {e}",
                    script_path.display()
                ));
            }
            command
                .arg(crate::floor::FILTER_COMPLEX_FILE)
                .arg(&script_path);
            command.args(["-map", "[mix]"]);
            Some(script_path)
        } else {
            let duration_sec = (spec.duration_ms.max(0) as f64) / 1000.0;
            command.args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=48000:cl=stereo",
                "-t",
                &format!("{:.3}", duration_sec),
            ]);
            None
        }
    } else {
        let duration_sec = (spec.duration_ms.max(0) as f64) / 1000.0;
        command.args([
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=48000:cl=stereo",
            "-t",
            &format!("{:.3}", duration_sec),
        ]);
        None
    };

    command.args([
        "-vn",
        "-c:a",
        &spec.codec,
        "-ar",
        &spec.sample_rate.to_string(),
        "-f",
        "wav",
    ]);
    command.arg(deliverable.temp());

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            if let Some(script) = &script {
                let _ = std::fs::remove_file(script);
            }
            return Err(format!("{} could not be run: {e}", ffmpeg.display()));
        }
    };

    let mut stderr_output = String::new();
    if let Some(mut err_pipe) = child.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut err_pipe, &mut stderr_output);
    }

    let status = match child.wait() {
        Ok(status) => status,
        Err(e) => {
            if let Some(script) = &script {
                let _ = std::fs::remove_file(script);
            }
            return Err(format!("ffmpeg process error: {e}"));
        }
    };

    if let Some(script) = &script {
        let _ = std::fs::remove_file(script);
    }

    if !status.success() {
        return Err(format!(
            "ffmpeg failed to export audio stem {}: {stderr_output}",
            output.display()
        ));
    }

    let bytes = std::fs::metadata(deliverable.temp())
        .map(|m| m.len())
        .unwrap_or(0);
    let path = deliverable.commit()?;

    Ok(FinishedAudio { path, bytes })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "montagent-render-encode/{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("readable")
            .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// ADR-0143 §3: the production encode always names its thread count, and never the
    /// automatic one. `-threads 0` is `ffmpeg`'s *"pick for me"*, and leaving the option
    /// out means the same; either would make the bytes depend on the machine's core count.
    #[test]
    fn the_production_encode_pins_libx264s_thread_count_and_never_asks_for_auto() {
        let args = Settings::PRODUCTION.args();
        let at = |flag: &str| {
            let i = args.iter().position(|a| a == flag);
            i.map(|i| args[i + 1].as_str())
        };
        assert_eq!(at("-c:v"), Some(crate::floor::VIDEO_ENCODER));
        assert_eq!(at("-preset"), Some("medium"), "ADR-0077, kept by ADR-0143");
        assert_eq!(at("-crf"), Some("20"), "ADR-0077, kept by ADR-0143");
        assert_eq!(
            at("-threads"),
            Some(THREADS.to_string().as_str()),
            "the encoder is given a thread count: {args:?}"
        );
        assert_eq!(THREADS.get(), 5, "changing N needs a superseding ADR");
        assert_eq!(args.iter().filter(|a| *a == "-threads").count(), 1);
        assert!(
            !["0", "auto"].contains(&at("-threads").unwrap_or("0")),
            "{args:?}"
        );
        assert!(
            args.iter().position(|a| a == "-threads") > args.iter().position(|a| a == "-c:v"),
            "an output option after the encoder, so it is libx264's thread count"
        );
    }

    /// A test can pin another thread count through the same settings, and the encode
    /// carries exactly what it was given (#627 §8's byte test needs this).
    #[test]
    fn a_tests_own_settings_reach_the_arguments() {
        let settings = Settings {
            threads: NonZeroU32::new(2).expect("two"),
            ..Settings::PRODUCTION
        };
        let args = settings.args();
        let i = args.iter().position(|a| a == "-threads").expect("pinned");
        assert_eq!(args[i + 1], "2");
    }

    #[test]
    fn the_temp_file_is_a_dotted_sibling_and_the_target_is_never_opened() {
        let dir = scratch("sibling");
        let target = dir.join("out").join("video.mp4");
        let deliverable = Deliverable::begin(&target).expect("reserved");

        assert_eq!(deliverable.temp().parent(), target.parent());
        let name = deliverable
            .temp()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(name.starts_with(".video.mp4.montagent-partial-"), "{name}");
        assert!(dir.join("out").is_dir(), "the parent directory is created");
        assert!(!target.exists());
    }

    #[test]
    fn an_abandoned_deliverable_leaves_nothing_behind() {
        let dir = scratch("abandon");
        let target = dir.join("video.mp4");
        let deliverable = Deliverable::begin(&target).expect("reserved");
        std::fs::write(deliverable.temp(), b"half a video").expect("partial bytes");
        deliverable.abandon();

        assert!(!target.exists(), "the declared path was never touched");
        assert_eq!(
            entries(&dir),
            Vec::<String>::new(),
            "and the temp file is gone"
        );
    }

    #[test]
    fn a_committed_deliverable_is_the_whole_file_under_its_own_name() {
        let dir = scratch("commit");
        let target = dir.join("video.mp4");
        std::fs::write(&target, b"the previous render").expect("an older deliverable");
        let mut deliverable = Deliverable::begin(&target).expect("reserved");
        std::fs::write(deliverable.temp(), b"the new render").expect("bytes");
        let published = deliverable.commit().expect("renamed");

        assert_eq!(published, target);
        assert_eq!(std::fs::read(&target).unwrap(), b"the new render");
        assert_eq!(entries(&dir), vec!["video.mp4".to_string()]);
    }

    #[test]
    fn an_unrunnable_ffmpeg_names_its_path_and_leaves_no_file() {
        let dir = scratch("no-ffmpeg");
        let target = dir.join("video.mp4");
        let e = Encoder::start(
            Path::new("/nowhere/ffmpeg"),
            &target,
            &Spec::new(16, 16, 25.into(), Rgba::BLACK),
        )
        .err()
        .expect("nothing to run");

        assert!(e.contains("/nowhere/ffmpeg"), "{e}");
        assert!(!target.exists());
        assert_eq!(entries(&dir), Vec::<String>::new());
    }

    #[test]
    fn an_encoder_dropped_mid_stream_publishes_nothing_and_leaves_nothing() {
        // ADR-0011: "a truncated MP4 at the deliverable path reads as finished". Frames
        // are pushed, the process holding them is abandoned, and the declared path must
        // not exist — not as a short file, not as an empty one.
        let Some(ffmpeg) = find_ffmpeg() else {
            eprintln!("skipping: no ffmpeg on PATH");
            return;
        };
        let dir = scratch("dropped");
        let target = dir.join("video.mp4");
        let mut encoder =
            Encoder::start(&ffmpeg, &target, &Spec::new(16, 16, 25.into(), Rgba::BLACK))
                .expect("spawned");
        for _ in 0..10 {
            encoder.push(&[0x80u8; 16 * 16 * 3]).expect("a frame");
        }
        assert!(
            !target.exists(),
            "nothing is published while frames are still arriving"
        );
        drop(encoder);
        assert!(!target.exists(), "an abandoned encode publishes nothing");
        assert_eq!(
            entries(&dir),
            Vec::<String>::new(),
            "and leaves no temp file"
        );
    }

    fn find_ffmpeg() -> Option<PathBuf> {
        std::env::var_os("PATH").and_then(|path| {
            std::env::split_paths(&path)
                .map(|dir| dir.join("ffmpeg"))
                .find(|candidate| candidate.is_file())
        })
    }

    #[test]
    fn a_frame_the_wrong_size_is_refused_before_it_reaches_the_pipe() {
        // Only where an `ffmpeg` exists to spawn; the check under test is the byte count.
        let Some(ffmpeg) = find_ffmpeg() else {
            eprintln!("skipping: no ffmpeg on PATH");
            return;
        };
        let dir = scratch("wrong-size");
        let target = dir.join("video.mp4");
        let mut encoder =
            Encoder::start(&ffmpeg, &target, &Spec::new(16, 16, 25.into(), Rgba::BLACK))
                .expect("spawned");
        let e = encoder
            .push(&[0u8; 7])
            .expect_err("seven bytes is not a frame");
        assert!(e.contains("7 bytes"), "{e}");
        drop(encoder);
        assert!(!target.exists(), "an unfinished encode publishes nothing");
        assert_eq!(entries(&dir), Vec::<String>::new());
    }

    #[test]
    fn exact_rational_rate_displays_as_fraction_and_integer_as_number() {
        assert_eq!(Rate::integer(25).to_string(), "25");
        assert_eq!(Rate::new(24000, 1001).to_string(), "24000/1001");
        assert_eq!(Rate::new(30000, 1001).to_string(), "30000/1001");
        assert_eq!(Rate::new(60000, 1001).to_string(), "60000/1001");
    }

    #[test]
    fn prores_arguments_and_alpha_pixel_format() {
        let spec_alpha = Spec {
            video_codec: Some("prores".to_string()),
            video_profile: Some("4444".to_string()),
            alpha: true,
            ..Spec::new(1920, 1080, 25.into(), Rgba::BLACK)
        };
        let args_alpha = spec_alpha.ffmpeg_args(
            Path::new("out.mov"),
            Path::new(".out.mov.tmp"),
            Some(Encoded {
                width: 1920,
                height: 1080,
            }),
            None,
        );
        let find_first = |flag: &str| -> Option<&str> {
            args_alpha
                .iter()
                .position(|a| a == flag)
                .map(|i| args_alpha[i + 1].as_str())
        };
        let find_last = |flag: &str| -> Option<&str> {
            args_alpha
                .iter()
                .rposition(|a| a == flag)
                .map(|i| args_alpha[i + 1].as_str())
        };
        assert_eq!(find_first("-pix_fmt"), Some("rgba"));
        assert_eq!(find_last("-c:v"), Some("prores_ks"));
        assert_eq!(find_last("-profile:v"), Some("4444"));
        assert_eq!(find_last("-vendor"), Some("apl0"));
        assert_eq!(find_last("-pix_fmt"), Some("yuva444p10le"));
        assert!(args_alpha.contains(&"-f".to_string()));
        assert!(args_alpha.contains(&"mov".to_string()));
        assert!(args_alpha.iter().any(|a| a.contains("color=0x00000000")));

        // Non-alpha ProRes defaults to standard (422) and yuv422p10le
        let spec_standard = Spec {
            video_codec: Some("prores".to_string()),
            ..Spec::new(1920, 1080, 25.into(), Rgba::BLACK)
        };
        let args_standard =
            spec_standard.ffmpeg_args(Path::new("out.mov"), Path::new(".out.mov.tmp"), None, None);
        let find_first_std = |flag: &str| -> Option<&str> {
            args_standard
                .iter()
                .position(|a| a == flag)
                .map(|i| args_standard[i + 1].as_str())
        };
        let find_last_std = |flag: &str| -> Option<&str> {
            args_standard
                .iter()
                .rposition(|a| a == flag)
                .map(|i| args_standard[i + 1].as_str())
        };
        assert_eq!(find_first_std("-pix_fmt"), Some("rgb24"));
        assert_eq!(find_last_std("-c:v"), Some("prores_ks"));
        assert_eq!(find_last_std("-profile:v"), Some("standard"));
        assert_eq!(find_last_std("-pix_fmt"), Some("yuv422p10le"));
    }

    #[test]
    fn dnxhr_arguments_and_mxf_container() {
        let spec = Spec {
            video_codec: Some("dnxhr".to_string()),
            video_profile: Some("dnxhr_hq".to_string()),
            ..Spec::new(1920, 1080, 25.into(), Rgba::BLACK)
        };
        let args = spec.ffmpeg_args(
            Path::new("master.mxf"),
            Path::new(".master.mxf.tmp"),
            None,
            None,
        );
        let find_first = |flag: &str| -> Option<&str> {
            args.iter()
                .position(|a| a == flag)
                .map(|i| args[i + 1].as_str())
        };
        let find_last = |flag: &str| -> Option<&str> {
            args.iter()
                .rposition(|a| a == flag)
                .map(|i| args[i + 1].as_str())
        };
        assert_eq!(find_first("-pix_fmt"), Some("rgb24"));
        assert_eq!(find_last("-c:v"), Some("dnxhd"));
        assert_eq!(find_last("-profile:v"), Some("dnxhr_hq"));
        assert_eq!(find_last("-pix_fmt"), Some("yuv422p"));
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"mxf".to_string()));
        assert!(!args.contains(&"-movflags".to_string()));
    }

    #[test]
    fn alpha_pushes_4_bytes_per_pixel() {
        let Some(ffmpeg) = find_ffmpeg() else {
            return;
        };
        let dir = scratch("alpha-frame-bytes");
        let target = dir.join("alpha.mov");
        let mut encoder = Encoder::start(
            &ffmpeg,
            &target,
            &Spec {
                video_codec: Some("prores".to_string()),
                video_profile: Some("4444".to_string()),
                alpha: true,
                ..Spec::new(16, 16, 25.into(), Rgba::TRANSPARENT)
            },
        )
        .expect("spawned");

        // 3 bytes/pixel should be rejected because alpha expects 4 bytes/pixel
        let err = encoder
            .push(&[0u8; 16 * 16 * 3])
            .expect_err("3 bytes rejected");
        assert!(err.contains("expecting 1024"), "{err}");

        // 4 bytes/pixel succeeds
        encoder
            .push(&[0u8; 16 * 16 * 4])
            .expect("valid 4-byte frame");
        drop(encoder);
    }

    #[test]
    fn audio_stem_export_writes_broadcast_wav_and_silence_fallback() {
        let Some(ffmpeg) = find_ffmpeg() else {
            return;
        };
        let dir = scratch("audio-stem");
        let target = dir.join("dx.wav");

        let res = export_audio(
            &ffmpeg,
            &target,
            None,
            &AudioStemSpec {
                codec: "pcm_s24le".to_string(),
                sample_rate: 48000,
                duration_ms: 1000,
            },
        )
        .expect("exported audio stem");

        assert_eq!(res.path, target);
        assert!(res.bytes > 0);
        assert!(target.exists());
    }
}
