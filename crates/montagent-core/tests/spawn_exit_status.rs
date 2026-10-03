//! ADR-0115 §2: **a failed spawn is never an empty answer**, as an invariant over every spawn.
//!
//! ADR-0113 made the rule for the seek, after ffmpeg 9 removed `-vsync` and a failed `select`
//! run was read as *"no frame at or before"* — painting frames early with `0 errors`. The
//! tool qualification (ADR-0115) catches the breakages already known; this rule is the one
//! that catches the next one, so it has to hold at every spawn and not only at the sites
//! that have been bitten.
//!
//! A test cannot read whether a call site honours an exit status. It can make sure nobody
//! adds a spawn without saying how it does: every `Command::new(` in the workspace's sources
//! is listed below with the reason it complies, and a site that is not listed fails here.

use std::path::{Path, PathBuf};

/// Each spawning file, how many spawns it holds, and how each honours the exit status.
const AUDITED: &[(&str, usize, &str)] = &[
    (
        "montagent-render/src/decode.rs",
        2,
        "frame_at checks both runs' status before reading stdout (ADR-0113 §1); \
         frames_from waits for the process at end of stdout and errors on non-zero (§2)",
    ),
    (
        "montagent-render/src/encode.rs",
        1,
        "Encoder::seal waits and checks the status before anything is published \
         (ADR-0093, ADR-0109)",
    ),
    (
        "montagent-render/src/floor.rs",
        2,
        "the tool qualification: a non-zero exit *is* the verdict (ADR-0115); `version` \
         reads stdout only after a zero exit and answers `None` otherwise (ADR-0143 §6)",
    ),
    (
        "montagent-core/src/media/probe.rs",
        1,
        "ProcessRunner records success and the exit code; interpret reads tool_failure \
         before any fact about the media (ADR-0091)",
    ),
];

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .to_path_buf()
}

fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

#[test]
fn every_spawn_in_the_workspace_is_one_whose_exit_status_was_audited() {
    let crates = crates_dir();
    let mut files = Vec::new();
    for krate in std::fs::read_dir(&crates).expect("read crates/").flatten() {
        rust_files(&krate.path().join("src"), &mut files);
    }
    assert!(
        !files.is_empty(),
        "no sources found under {}",
        crates.display()
    );

    let mut found: Vec<(String, usize)> = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).expect("read a source file");
        let spawns = source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .filter(|line| line.contains("Command::new("))
            .count();
        if spawns > 0 {
            let relative = file
                .strip_prefix(&crates)
                .expect("under crates/")
                .to_string_lossy()
                .replace('\\', "/");
            found.push((relative, spawns));
        }
    }
    found.sort();

    let mut audited: Vec<(String, usize)> = AUDITED
        .iter()
        .map(|(file, count, _)| (file.to_string(), *count))
        .collect();
    audited.sort();

    assert_eq!(
        found, audited,
        "a spawn was added, moved or removed. ADR-0115 §2: a non-zero exit from any \
         ffmpeg/ffprobe spawn is a failure, never an empty answer — check the new site reads \
         the exit status before its output, then record it in AUDITED with how it does"
    );
}
