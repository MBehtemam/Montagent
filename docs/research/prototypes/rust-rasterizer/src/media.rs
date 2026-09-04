// FFmpeg is a subprocess in both arms (ADR-0009: the licence, not the speed).
// Neither rasterizer decodes video, so every frame crosses this boundary — the
// cost #6 never measured, because its fixture had no video in it.
use std::io::{Read, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

pub struct Decoder {
    child: Child,
    out: ChildStdout,
    frame: Vec<u8>,
    pub frames_read: u64,
    pub bytes_read: u64,
}

impl Decoder {
    /// Open `src` at `at` seconds and stream RGBA frames at `fps`, scaled to w x h.
    pub fn open(src: &str, at: f64, fps: f64, w: u32, h: u32) -> Self {
        let mut child = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-ss",
                &format!("{at:.6}"),
                "-i",
                src,
                "-vf",
                &format!("fps={fps},scale={w}:{h}"),
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("ffmpeg decode");
        let out = child.stdout.take().unwrap();
        Self {
            child,
            out,
            frame: vec![0u8; (w as usize) * (h as usize) * 4],
            frames_read: 0,
            bytes_read: 0,
        }
    }

    /// Next decoded frame as RGBA, or None at end of stream.
    pub fn next(&mut self) -> Option<&[u8]> {
        match self.out.read_exact(&mut self.frame) {
            Ok(()) => {
                self.frames_read += 1;
                self.bytes_read += self.frame.len() as u64;
                Some(&self.frame)
            }
            Err(_) => None,
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Encoder {
    child: Child,
}

impl Encoder {
    pub fn open(w: u32, h: u32, fps: f64, out: &str, audio: Vec<String>) -> Self {
        let mut args: Vec<String> = vec![
            "-y".into(),
            "-hide_banner".into(),
            "-loglevel".into(),
            "error".into(),
            "-f".into(),
            "rawvideo".into(),
            "-pix_fmt".into(),
            "rgba".into(),
            "-s".into(),
            format!("{w}x{h}"),
            "-r".into(),
            format!("{fps}"),
            "-i".into(),
            "pipe:0".into(),
        ];
        args.extend(audio);
        args.extend([
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            "medium".into(),
            "-crf".into(),
            "23".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
        ]);
        args.push(out.into());
        let child = Command::new("ffmpeg")
            .args(&args)
            .stdin(Stdio::piped())
            .spawn()
            .expect("ffmpeg encode");
        Self { child }
    }

    pub fn write(&mut self, pixels: &[u8]) {
        self.child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(pixels)
            .expect("pipe to ffmpeg");
    }

    pub fn finish(mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

/// Load a still as RGBA8. Shared by both backends so decode cost is common.
pub fn load_rgba(path: &str) -> (Vec<u8>, u32, u32) {
    let img = image::open(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .to_rgba8();
    let (w, h) = (img.width(), img.height());
    (img.into_raw(), w, h)
}
