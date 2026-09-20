//! One video frame, out of an `ffmpeg` Montaget **spawns** rather than links.
//!
//! ADR-0009 ships Montaget as *"a binary, plus an `ffmpeg` the user supplies"* — `libx264`
//! is GPL and linking `libavcodec` would make the shipped binary a GPL combined work — so
//! every decoded frame crosses a subprocess boundary. That is the cost the prototype
//! measured and #6 never did, *"because its fixture had no video in it"*, and it is why
//! [#212](https://github.com/MBehtemam/Montaget/issues/212) requires the decode path its
//! own fixture: the committed project has **zero** `video` elements, and under ADR-0003's
//! asymmetry that silence is not evidence the path is unneeded.
//!
//! **Where this crate's responsibility starts.** The path to `ffmpeg` is handed in, never
//! resolved here: `montaget_core::media::tools` owns resolution *"at the first tool that
//! spawns a subprocess"*, so `probe`, `frame`, `render` and `preview` share one answer to
//! "is there an `ffmpeg` on this machine, and where". A second resolution in this crate
//! would be the second authority ADR-0011 spent itself removing.
//!
//! **Two normalisations, and why only one of them is here.** Rotation is applied: the
//! container's track-level display transform is what every mainstream player reads, and
//! `ffmpeg` applies it by default, so a portrait phone clip arrives upright — which is the
//! geometry ADR-0023 says the author was looking at. PAR is **not** applied, and its
//! absence is not an oversight: ADR-0013 settled that a source is resampled to exactly the
//! declared `width`×`height`, so the whole source maps onto the whole box whatever its
//! pixel aspect ratio is. PAR changes the *nominal* aspect of source pixels, never which
//! of them survive, so it is an input to `fit`'s arithmetic — which `validate` evaluates —
//! and not to this blit.

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
