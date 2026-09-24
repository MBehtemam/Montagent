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

/// Decode the frame at `at_ms` into the source, resampled to `width`×`height`.
///
/// `at_ms` is an offset **into the source file**, not a timeline instant: the arithmetic
/// that turns one into the other — `source_start`, `speed`, `overrun`'s hold and loop —
/// is ADR-0020's and lives in the core, which is also what answers `query --at`'s *offset
/// into the source* with the same number. Two implementations of that arithmetic would be
/// a picture and a caption that disagree.
///
/// The seek is `-ss` **before** `-i`, which is the input seek: `ffmpeg` jumps to the
/// nearest preceding keyframe and decodes forward to the requested instant rather than
/// decoding the file from zero. At the 500 ms budget the difference is the whole budget.
///
/// The error is one sentence naming the source and what `ffmpeg` said, because the caller
/// is a report and ADR-0011's exit 70 says *"retry or report"* — neither of which is
/// possible from *"ffmpeg failed"*.
pub fn frame_at(
    ffmpeg: &Path,
    source: &str,
    at_ms: i64,
    width: u32,
    height: u32,
) -> Result<DecodedFrame, String> {
    if width == 0 || height == 0 {
        return Err(format!(
            "{source}: a frame was asked for at {width}x{height}, which is no frame at all"
        ));
    }
    // Milliseconds to `ffmpeg`'s decimal seconds, in the string rather than through a
    // float: ADR-0005 stores every time as an integer millisecond because float seconds
    // failed in practice, and re-introducing one at the boundary would put the error back
    // in the one place the whole surface treats as authoritative.
    let seconds = format!("{}.{:03}", at_ms.max(0) / 1000, at_ms.max(0) % 1000);

    let output = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &seconds,
            "-i",
            source,
            "-frames:v",
            "1",
            "-vf",
            &format!("scale={width}:{height}"),
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("{source}: {} could not be run: {e}", ffmpeg.display()))?;

    let expected = (width as usize) * (height as usize) * 4;
    if output.stdout.len() != expected {
        let said = String::from_utf8_lossy(&output.stderr);
        let said = said.trim();
        // The empty-output case is the one an author can act on, so it says what it means
        // rather than reporting a byte count: seeking past the end of a source produces no
        // frame, and `ffmpeg` exits cleanly having written nothing.
        return Err(if output.stdout.is_empty() {
            format!(
                "{source}: no frame at {seconds}s{}",
                if said.is_empty() {
                    " — the seek is past the end of the source".to_string()
                } else {
                    format!(" — {said}")
                }
            )
        } else {
            format!(
                "{source}: ffmpeg wrote {} bytes for a {width}x{height} RGBA frame, which \
                 needs {expected}{}",
                output.stdout.len(),
                if said.is_empty() {
                    String::new()
                } else {
                    format!(" — {said}")
                }
            )
        });
    }

    Ok(DecodedFrame {
        rgba: output.stdout,
        width,
        height,
    })
}

/// A **run** of frames out of one `ffmpeg`, starting at `from_ms` into the source and
/// sampled at `per_second` source frames per second of source time.
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
/// `per_second` is the rate in **source** time, so an element played at `speed: 2` asks for
/// twice the project's `fps` and lands on the frames the timeline actually shows. A float,
/// because that product is one: `fps` is an integer and `speed` is not.
pub fn frames_from(
    ffmpeg: &Path,
    source: &str,
    from_ms: i64,
    per_second: f64,
    width: u32,
    height: u32,
) -> Result<Frames, String> {
    if width == 0 || height == 0 {
        return Err(format!(
            "{source}: frames were asked for at {width}x{height}, which is no frame at all"
        ));
    }
    if !per_second.is_finite() || per_second <= 0.0 {
        return Err(format!(
            "{source}: a run of frames needs a positive sampling rate, not {per_second}"
        ));
    }

    // [`frame_at`]'s conversion, for [`frame_at`]'s reason: integer milliseconds to
    // `ffmpeg`'s decimal seconds through the string, never through a float.
    let seconds = format!("{}.{:03}", from_ms.max(0) / 1000, from_ms.max(0) % 1000);

    let mut child = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &seconds,
            "-i",
            source,
            "-vf",
            &format!("fps={per_second},scale={width}:{height}"),
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{source}: {} could not be run: {e}", ffmpeg.display()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("{source}: ffmpeg's output could not be read"))?;

    Ok(Frames {
        child,
        stdout,
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
    /// re-derive. A run that ends mid-frame is the one genuine failure, because those bytes
    /// are not a picture.
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
        let e = frame_at(Path::new("/nowhere/ffmpeg"), "clip.mov", 0, 0, 100)
            .expect_err("no frame has zero width");
        assert!(e.contains("clip.mov"), "{e}");
    }

    #[test]
    fn an_unrunnable_ffmpeg_names_the_path_it_tried() {
        let e = frame_at(Path::new("/nowhere/ffmpeg"), "clip.mov", 0, 16, 16)
            .expect_err("nothing to run");
        assert!(e.contains("/nowhere/ffmpeg"), "{e}");
        assert!(e.contains("clip.mov"), "{e}");
    }
}
