//! **`render` spawns an `ffmpeg` per feed, never one per frame** — ADR-0142 §4's deterministic
//! spawn-count test, which stands in CI where a wall-clock gate cannot.
//!
//! The hour a six-minute render used to take (#532) was structural: one `frame_at` spawn per
//! frame per visible `video` element. A wall clock on a shared runner cannot see that come
//! back; a count can. So this test runs the shipped binary with `ffmpeg` and `ffprobe`
//! shimmed on `PATH` to log every invocation, classifies each one by its command line, and
//! asserts:
//!
//! - the decode spawns are exactly the **feeds opened plus their reopens** (ADR-0141) — one
//!   per element, one more per loop wrap — and no `frame_at` at all;
//! - **doubling the frames does not add a single spawn**, decode or otherwise.
//!
//! It catches a reopen storm too: a feed reopened on every frame is a spawn per frame again.
//! It does not catch a slowdown that keeps the same structure; that shows as drift against
//! the benchmark readings (`BENCHMARK_REFERENCES`), re-measured whenever render's hot path changes (ADR-0142).

// The shims are shell scripts.
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// What one `render` spawned, by kind.
#[derive(Debug, Default, PartialEq, Eq)]
struct Spawned {
    /// Feeds: `frames_at`'s run, recognisable by its `setpts` onto the timeline.
    feeds: usize,
    /// Single-frame seeks: `frame_at`'s `select`.
    seeks: usize,
    /// Every other `ffmpeg`: the encoder, the tool qualification, the mux.
    other_ffmpeg: usize,
    ffprobe: usize,
}

#[test]
fn a_render_spawns_its_feeds_and_their_reopens_and_nothing_per_frame() {
    let Ok(tools) = montagent_core::media::tools::resolve() else {
        eprintln!("skipping: no qualified ffmpeg/ffprobe on PATH");
        return;
    };
    let dir = scratch("render-spawns");
    let clip = dir.join("clip.mp4");
    let made = Command::new(&tools.ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi"])
        .args(["-i", "testsrc2=s=64x48:r=25", "-frames:v", "200"])
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
        .arg(&clip)
        .status()
        .expect("ffmpeg runs");
    assert!(made.success(), "the source could not be generated");

    // One playing element, and one looping over a 1 s source range.
    let short = project(&dir, "short", &clip, 2000);
    let long = project(&dir, "long", &clip, 4000);
    let short = render(&dir, &tools, &short, "short");
    let long = render(&dir, &tools, &long, "long");

    // `played` plays 0..2000 or 0..4000 once; `looped` wraps once a second.
    assert_eq!(
        short,
        Spawned {
            feeds: 2 + 1,
            seeks: 0,
            ..short
        },
        "2 s: two feeds and one reopen at the loop's wrap"
    );
    assert_eq!(
        long,
        Spawned {
            feeds: 2 + 3,
            seeks: 0,
            ..long
        },
        "4 s: two feeds and three reopens"
    );
    // Twice the frames, and nothing but the wraps added: no spawn is per frame.
    assert_eq!(long.other_ffmpeg, short.other_ffmpeg);
    assert_eq!(long.ffprobe, short.ffprobe);
}

/// A 25 fps project of `duration` ms holding one playing and one looping `video` element.
fn project(dir: &Path, name: &str, clip: &Path, duration: i64) -> PathBuf {
    let source = clip.display().to_string().replace('\\', "/");
    let body = format!(
        r##"{{"frame":{{"width":160,"height":120}},"fps":25,"background":"#000000",
            "duration":{duration},"output":"out/{name}.mp4","tracks":[
            {{"name":"a","layer":0,"elements":[
              {{"id":"played","type":"video","start":0,"end":{duration},"source":"{source}",
                "source_start":0,"source_end":{duration},"x":0,"y":0,"origin":"top-left",
                "width":64,"height":48,"fit":"literal","volume":0}}]}},
            {{"name":"b","layer":1,"elements":[
              {{"id":"looped","type":"video","start":0,"end":{duration},"source":"{source}",
                "overrun":"loop","source_start":0,"source_end":1000,"x":80,"y":0,
                "origin":"top-left","width":64,"height":48,"fit":"literal","volume":0}}]}}]}}"##
    );
    let value: serde_json::Value = serde_json::from_str(&body).expect("a valid test project");
    let path = dir.join(format!("{name}.montagent.json"));
    std::fs::write(
        &path,
        montagent_core::write::canonical(&montagent_core::layout::canonicalise(&value)),
    )
    .expect("write the project");
    path
}

/// Render `project` with every `ffmpeg` and `ffprobe` logged, and classify what ran.
fn render(
    dir: &Path,
    tools: &montagent_core::media::tools::Tools,
    project: &Path,
    name: &str,
) -> Spawned {
    let shims = dir.join(format!("shims-{name}"));
    std::fs::create_dir_all(&shims).expect("create the shim dir");
    for (tool, real) in [("ffmpeg", &tools.ffmpeg), ("ffprobe", &tools.ffprobe)] {
        let shim = shims.join(tool);
        std::fs::write(
            &shim,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec '{}' \"$@\"\n",
                shims.join(format!("{tool}.log")).display(),
                real.display()
            ),
        )
        .expect("write a shim");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755))
            .expect("make the shim executable");
    }
    let mut path = shims.clone().into_os_string();
    path.push(":");
    path.push(std::env::var_os("PATH").unwrap_or_default());

    let out = Command::new(env!("CARGO_BIN_EXE_montagent"))
        .arg("render")
        .arg(project)
        .arg("--json")
        .env("PATH", path)
        // A cold probe cache, so both renders pay the same `ffprobe`s.
        .env(
            montagent_core::media::sidecar::CACHE_DIR_VAR,
            dir.join(format!("cache-{name}")),
        )
        .output()
        .expect("run montagent render");
    assert!(
        out.status.success(),
        "render failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let lines = |tool: &str| -> Vec<String> {
        std::fs::read_to_string(shims.join(format!("{tool}.log")))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    };
    let mut spawned = Spawned {
        ffprobe: lines("ffprobe").len(),
        ..Spawned::default()
    };
    for line in lines("ffmpeg") {
        if line.contains("setpts=") && line.contains("rawvideo") {
            spawned.feeds += 1;
        } else if line.contains("select=") {
            spawned.seeks += 1;
        } else {
            spawned.other_ffmpeg += 1;
        }
    }
    spawned
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "montagent-render-spawns/{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}
