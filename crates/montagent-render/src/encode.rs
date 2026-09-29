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

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

use crate::canvas::Rgba;

/// What one encode is asked to produce.
#[derive(Debug, Clone)]
pub struct Spec {
    /// The project's declared frame, at true pixels. Never proxy-scaled (ADR-0021).
    pub width: u32,
    pub height: u32,
    pub fps: i64,
    /// The colour an odd dimension is padded with.
    pub background: Rgba,
    /// The audio mix, where the project has anything audible.
    pub audio: Option<Audio>,
    /// What to stamp the container with, so a later run can tell whose file this is
    /// (ADR-0104). `None` writes no stamp: a **preview** is not a deliverable and must not
    /// leave an attestation a later `render` would read as its own.
    pub stamp: Option<String>,
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
        if spec.fps <= 0 {
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
        command.args(["-hide_banner", "-loglevel", "error", "-y"]);
        command.args([
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            &format!("{}x{}", spec.width, spec.height),
            "-r",
            &spec.fps.to_string(),
            "-i",
            "pipe:0",
        ]);
        if let Some(audio) = &spec.audio {
            for input in &audio.inputs {
                command.arg("-i").arg(input);
            }
        }
        if let Some(script) = &script {
            // ADR-0115: the graph is read from a file, in the spelling the floor fixes.
            command.arg(crate::floor::FILTER_COMPLEX_FILE).arg(script);
        }
        command.args(["-map", "0:v"]);
        if let Some(Encoded { width, height }) = encoded {
            let Rgba([r, g, b, _]) = spec.background;
            command.args([
                "-vf",
                &format!("pad={width}:{height}:0:0:color=0x{r:02X}{g:02X}{b:02X}"),
            ]);
        }
        command.args([
            "-c:v",
            crate::floor::VIDEO_ENCODER,
            "-preset",
            "medium",
            "-crf",
            "20",
            "-pix_fmt",
            "yuv420p",
            "-r",
            &spec.fps.to_string(),
        ]);
        if spec.audio.is_some() {
            command.args(["-map", "[mix]", "-c:a", "aac", "-b:a", "160k"]);
        } else {
            command.arg("-an");
        }
        // ADR-0104: the attestation, written at the moment of the act. It is what makes
        // *"did this project produce the file already at the output path?"* a question a
        // later run reads an answer to instead of guessing one from duration or frame count.
        if let Some(stamp) = &spec.stamp {
            command.arg("-metadata").arg(format!("comment={stamp}"));
        }
        command.args(["-movflags", "+faststart", "-f", "mp4"]);
        command.arg(deliverable.temp());
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

        Ok(Encoder {
            child,
            stdin,
            stderr,
            deliverable: Some(deliverable),
            script,
            frame_bytes: spec.width as usize * spec.height as usize * 3,
            frames: 0,
            encoded,
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
            &Spec {
                width: 16,
                height: 16,
                fps: 25,
                background: Rgba::BLACK,
                audio: None,
                stamp: None,
            },
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
        let mut encoder = Encoder::start(
            &ffmpeg,
            &target,
            &Spec {
                width: 16,
                height: 16,
                fps: 25,
                background: Rgba::BLACK,
                audio: None,
                stamp: None,
            },
        )
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
        let mut encoder = Encoder::start(
            &ffmpeg,
            &target,
            &Spec {
                width: 16,
                height: 16,
                fps: 25,
                background: Rgba::BLACK,
                audio: None,
                stamp: None,
            },
        )
        .expect("spawned");
        let e = encoder
            .push(&[0u8; 7])
            .expect_err("seven bytes is not a frame");
        assert!(e.contains("7 bytes"), "{e}");
        drop(encoder);
        assert!(!target.exists(), "an unfinished encode publishes nothing");
        assert_eq!(entries(&dir), Vec::<String>::new());
    }
}
