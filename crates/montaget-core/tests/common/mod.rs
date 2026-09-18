//! Scratch-directory helpers shared by the integration tests.

use std::path::{Path, PathBuf};

/// A scratch directory unique to one call site, created fresh on every run.
pub fn tempdir(discriminator: u32) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "montaget-core-tests/{}-{discriminator}",
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
