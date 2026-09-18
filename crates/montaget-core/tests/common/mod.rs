//! Scratch-directory helpers shared by the integration tests.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// A scratch directory created fresh on every run, unique to the line that asked for it
/// so that tests running in parallel never share one.
pub fn tempdir(caller_line: u32) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "montaget-core-tests/{}-line-{caller_line}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Write `body` to `name` inside `dir` and hand back the path.
pub fn write_project(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write project file");
    path
}

/// Whether this machine has the `ffmpeg`/`ffprobe` the disk half of `validate` needs.
///
/// ADR-0009 ships Montaget as *"a binary, plus an `ffmpeg` the user supplies"*, so not
/// having one is a legitimate state rather than a broken checkout — and one the tool
/// answers with exit 70 rather than with a verdict. Tests that need real media say so
/// through this, which asks the same question `validate` asks; inferring it from an exit
/// code afterwards would green-light every *other* internal failure too.
pub fn has_ffprobe() -> bool {
    match montaget_core::media::tools::resolve() {
        Ok(_) => true,
        Err(missing) => {
            eprintln!("skipping: {}", missing.reason());
            false
        }
    }
}

/// A path rendered with `/` separators, so a tail can be compared to one.
///
/// The project format writes `audio/05-cobweb.mp3`. A *resolved* path is the
/// platform's, and on Windows that is backslashes plus, after `canonicalize`, a
/// `\\?\` verbatim prefix — so `ends_with("audio/05-cobweb.mp3")` is false for
/// a path that is entirely correct. Normalise before comparing a tail; the
/// separator is the platform's, never the document's.
pub fn with_forward_slashes(path: &str) -> String {
    path.replace('\\', "/")
}
