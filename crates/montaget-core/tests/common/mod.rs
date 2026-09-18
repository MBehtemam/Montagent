//! Scratch-directory helpers shared by the integration tests.

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
