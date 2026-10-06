//! `render`'s speed target, measured the way ADR-0142 states it — **run on purpose, never by
//! the suite**.
//!
//! ```text
//! cargo test --release -p montagent --test render_target -- --ignored --nocapture
//! ```
//!
//! The target is [`RENDER_TARGET`]: the **benchmark project** renders in at most 3 minutes,
//! median wall clock, on the dev's M1 Pro. The benchmark project is what
//! `fixtures/benchmark/make_benchmark.py` builds from the committed fixtures — 6 minutes of
//! 1920x1080 at 30 fps, three `video` elements visible at once for most of the timeline,
//! text, rects and an audio mix. This test builds it (generating the media is not timed),
//! or takes one already built.
//!
//! It is `#[ignore]`d because nothing about a shared CI runner is the machine the number is
//! stated for, and because today one run takes about an hour.
//!
//! A second case, `the_spy_trailer_renders_within_the_paint_target`, judges the **paint**
//! class's target (ADR-0144): `fixtures/benchmark/spy-trailer/` in at most [`PAINT_TARGET`],
//! every frame hash equal to its committed `frames.framemd5`. It follows the same protocol,
//! with the load checked before every run.
//!
//! ## The protocol (ADR-0142 §5)
//!
//! - **Timed:** the whole `montagent render` command, process start to finished MP4 and
//!   report, cold with an empty probe sidecar on every run (as `render_budget.rs` and
//!   `frame_budget.rs` take theirs).
//! - **Runs:** one discarded warm-up, then five timed runs. The **median** is judged; min
//!   and max are reported.
//! - **Recorded with each measurement:** the commit, the `ffmpeg` version string, the macOS
//!   version and `available_parallelism`, the load average before and after, user+sys CPU
//!   time and peak memory (`/usr/bin/time -l`), the spawn count, and the render's own
//!   answer. All of it is printed as **one JSON line** on stdout, so the numbers can be
//!   pasted into an ADR as taken.
//!
//! ## When it refuses to judge
//!
//! Like `frame_budget.rs`, it prints why and judges nothing, rather than failing for a
//! reason the target never claimed:
//!
//! - **a debug build** — it returns before rendering, since no number from it is comparable;
//! - **load average ≥ 1.5 at the start**;
//! - **battery power** (`pmset -g batt`);
//! - **a machine other than the M1 Pro** the number is stated for;
//! - **fewer than five timed runs** (`MONTAGENT_RENDER_TARGET_RUNS`, for a single "before"
//!   run that would otherwise cost six hours);
//! - **frame hashes or a report that differ from a sequential render's.** This one also
//!   fails the test after printing the record, because it is not about the machine: a
//!   render that paints different pixels is wrong before it is slow.
//!
//! A variant other than 1080p30 is **observed**: measured and recorded, judged against
//! nothing (ADR-0142 commits only 1080p30).
//!
//! ## What the frame-hash check is, exactly
//!
//! `render` exposes no hash of the frames it hands the encoder. So the smallest honest
//! version is used: each run's MP4 is decoded with `ffmpeg -f framemd5` and its per-frame
//! hashes are compared with a **sequential** render's. Today every render is sequential, so
//! by default the reference is this binary's own warm-up run, and the check proves only
//! that runs agree with each other. The benchmark project has `video` elements, so it still
//! paints on one painter after ADR-0144; where a parallel path paints it, pass the warm-up
//! `framemd5` a sequential build wrote (the path is printed) as
//! `MONTAGENT_RENDER_TARGET_SEQUENTIAL_FRAMEMD5`.
//!
//! Two limits, stated: it hashes the encoder's *output*, decoded — x264 is deterministic for
//! one input, settings and thread count, so equal input gives equal hashes, but a change
//! small enough to quantize away could pass unseen. And it holds only while the encoder
//! settings are the same on both sides; the encoder ticket (#628), which may change MP4
//! bytes, needs a reference re-taken with its settings.
//!
//! ## Environment
//!
//! | variable | meaning |
//! |---|---|
//! | `MONTAGENT_BENCHMARK_DIR` | a directory `make_benchmark.py` already built; skips generation |
//! | `MONTAGENT_BENCHMARK_VARIANT` | `1080p30` (default, judged), `2160p30` or `1080p30-1video` (observed) |
//! | `MONTAGENT_RENDER_TARGET_RUNS` | timed runs after the warm-up (default 5; fewer refuses to judge) |
//! | `MONTAGENT_RENDER_TARGET_SEQUENTIAL_FRAMEMD5` | a sequential render's `framemd5`, the hash reference |
//!
//! ## The per-stage breakdown
//!
//! Every render here runs with `MONTAGENT_STAGES` set (`montagent_core::verbs::render::
//! STAGES_VAR`, `#[doc(hidden)]`, ADR-0141), so each span prints one line on stderr with its
//! cumulative decode, paint, readback, encode-wait and seal milliseconds and its feed counts.
//! Each run's line is recorded with the run, and `"stages"` is the median run's. It is
//! measurement plumbing rather than an answer field: what `render` reports about its own
//! speed is the speed ticket's to shape. Unset, it costs four clock reads a frame.

// macOS only: the protocol reads `/usr/bin/time -l`, `pmset`, `sysctl` and `sw_vers`, and
// the number is stated for one Mac. Elsewhere this file compiles to nothing.
#![cfg(target_os = "macos")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use montagent_render::budget::{PAINT_TARGET, RENDER_TARGET, Verdict};
use serde_json::{Value, json};

/// ADR-0142's load-average ceiling at the start of a measurement.
const MAX_START_LOAD: f64 = 1.5;
/// ADR-0142's run count.
const PROTOCOL_RUNS: usize = 5;
/// The machine the number is stated for.
const STATED_MACHINE: &str = "Apple M1 Pro";
/// The one variant the target is stated for.
const JUDGED_VARIANT: &str = "1080p30";

#[test]
#[ignore = "ADR-0142's protocol: run on purpose, on the stated machine, with --release"]
fn the_benchmark_project_renders_within_the_target() {
    // Every number ADR-0142 would record is from the binary that ships. A debug render of
    // six minutes would also take most of a day, so this returns rather than measuring.
    if cfg!(debug_assertions) {
        eprintln!(
            "refusing to judge: this is a debug build, and RENDER_TARGET is stated for the \
             binary that ships. Run with --release."
        );
        return;
    }
    let Ok(tools) = montagent_core::media::tools::resolve() else {
        eprintln!("refusing to judge: no qualified ffmpeg/ffprobe on PATH");
        return;
    };

    let variant = env("MONTAGENT_BENCHMARK_VARIANT").unwrap_or_else(|| JUDGED_VARIANT.into());
    let runs: usize = env("MONTAGENT_RENDER_TARGET_RUNS")
        .map(|n| n.parse().expect("MONTAGENT_RENDER_TARGET_RUNS is a count"))
        .unwrap_or(PROTOCOL_RUNS);
    assert!(runs >= 1, "MONTAGENT_RENDER_TARGET_RUNS must be at least 1");

    let scratch = scratch(&variant);
    let project = benchmark(&variant);
    let output = scratch.join(format!("benchmark-{variant}.mp4"));
    let mut refused = Vec::new();

    // The conditions at the start, before anything this test spawns can move them.
    let load_before = load_average();
    let power = power_source();
    let cpu = sysctl("machdep.cpu.brand_string");
    if load_before[0] >= MAX_START_LOAD {
        refused.push(format!(
            "load average {:.2} at the start, and the protocol needs < {MAX_START_LOAD}",
            load_before[0]
        ));
    }
    if power.contains("Battery") {
        refused.push(format!("running on battery power ({power})"));
    }
    if cpu != STATED_MACHINE {
        refused.push(format!(
            "this is a {cpu}, and RENDER_TARGET is stated for the {STATED_MACHINE}"
        ));
    }
    if runs < PROTOCOL_RUNS {
        refused.push(format!(
            "{runs} timed run(s), and the protocol judges the median of {PROTOCOL_RUNS}"
        ));
    }

    // Discarded: it pages the binary, Skia and the sources in. It also counts the `ffmpeg`
    // and `ffprobe` spawns, through a shim on PATH that would cost the timed runs time.
    let shims = spawn_shims(&scratch, &tools);
    let warmup = render(&project, &output, &scratch.join("cache-0"), Some(&shims));
    let spawns = shims.counts();
    let warmup_hashes = frame_hashes(&tools.ffmpeg, &output, &scratch.join("warmup.framemd5"));
    eprintln!(
        "render-target [{variant}] discarded warm-up: {:.2?}, {} ffmpeg and {} ffprobe spawns; \
         its frame hashes are at {}",
        warmup.wall,
        spawns.0,
        spawns.1,
        scratch.join("warmup.framemd5").display()
    );

    let (reference_name, reference) = match env("MONTAGENT_RENDER_TARGET_SEQUENTIAL_FRAMEMD5") {
        Some(path) => {
            let hashes = read_framemd5(Path::new(&path));
            (path, hashes)
        }
        None => (
            "this binary's warm-up (every render is sequential today)".to_string(),
            warmup_hashes,
        ),
    };

    let mut timed = Vec::new();
    let mut mismatched = Vec::new();
    for n in 1..=runs {
        let _ = std::fs::remove_file(&output);
        let run = render(&project, &output, &scratch.join(format!("cache-{n}")), None);
        let hashes = frame_hashes(
            &tools.ffmpeg,
            &output,
            &scratch.join(format!("run-{n}.framemd5")),
        );
        if hashes != reference {
            mismatched.push(format!(
                "run {n}'s {} frame hashes differ from the {} of {reference_name}{}",
                hashes.len(),
                reference.len(),
                first_difference(&hashes, &reference)
            ));
        }
        if comparable(&run.answer) != comparable(&warmup.answer) {
            mismatched.push(format!("run {n}'s report differs from the warm-up's"));
        }
        eprintln!(
            "render-target [{variant}] run {n}/{runs}: {:.2?} (user {:.1} s, sys {:.1} s, \
             peak {} MiB)",
            run.wall,
            run.user_s,
            run.sys_s,
            run.max_rss_bytes / (1024 * 1024)
        );
        timed.push(run);
    }
    let load_after = load_average();
    refused.extend(mismatched.iter().cloned());

    let mut walls: Vec<Duration> = timed.iter().map(|r| r.wall).collect();
    walls.sort();
    let median = walls[walls.len() / 2];
    let judged = variant == JUDGED_VARIANT;
    let verdict = if !judged {
        "observed"
    } else if !refused.is_empty() {
        "refused"
    } else if median <= RENDER_TARGET {
        "within"
    } else {
        "exceeded"
    };

    let ms = |d: Duration| d.as_millis() as u64;
    let record = json!({
        "record": "montagent-render-target/1",
        "adr": "0142",
        "variant": variant,
        "project": project.file_name().map(|n| n.to_string_lossy().into_owned()),
        "commit": commit(),
        "ffmpeg": first_line(Command::new(&tools.ffmpeg).arg("-version")),
        "macos": format!(
            "{} ({})",
            first_line(Command::new("sw_vers").arg("-productVersion")),
            first_line(Command::new("sw_vers").arg("-buildVersion"))
        ),
        "cpu": cpu,
        "memory_bytes": sysctl("hw.memsize").parse::<u64>().ok(),
        "available_parallelism": std::thread::available_parallelism().map(|n| n.get()).ok(),
        "power": power,
        "load_before": load_before,
        "load_after": load_after,
        "warmup_ms": ms(warmup.wall),
        "spawns": {"ffmpeg": spawns.0, "ffprobe": spawns.1, "counted_on": "the warm-up"},
        "runs": timed.iter().map(|r| json!({
            "wall_ms": ms(r.wall),
            "user_s": r.user_s,
            "sys_s": r.sys_s,
            "max_rss_bytes": r.max_rss_bytes,
            "stages": r.stages,
        })).collect::<Vec<_>>(),
        "median_ms": ms(median),
        "min_ms": ms(walls[0]),
        "max_ms": ms(walls[walls.len() - 1]),
        "target_ms": judged.then(|| ms(RENDER_TARGET)),
        "verdict": verdict,
        "refused": refused,
        "frame_hashes": {
            "method": "ffmpeg -f framemd5 of each run's MP4, decoded, against a sequential render's",
            "reference": reference_name,
            "frames": reference.len(),
            "equal": mismatched.is_empty(),
        },
        "stages": timed
            .iter()
            .find(|r| r.wall == median)
            .map(|r| r.stages.clone())
            .unwrap_or(Value::Null),
        "render": timed.last().map(|r| r.answer["render"].clone()),
    });
    println!("{record}");

    for reason in &refused {
        eprintln!("refusing to judge: {reason}");
    }
    assert!(
        mismatched.is_empty(),
        "a timed run's pixels or report differ from a sequential render's, so the run does \
         not count (ADR-0142 §5):\n  {}",
        mismatched.join("\n  ")
    );
    match verdict {
        "within" | "exceeded" => {
            let verdict = if median <= RENDER_TARGET {
                Verdict::Within {
                    limit: RENDER_TARGET,
                    elapsed: median,
                }
            } else {
                Verdict::Exceeded {
                    limit: RENDER_TARGET,
                    elapsed: median,
                }
            };
            assert!(
                !verdict.is_failure(),
                "the benchmark project's median of {runs} renders was {median:.2?}, over \
                 RENDER_TARGET ({RENDER_TARGET:.0?}): {verdict:?}. ADR-0142 §8 says what a miss \
                 triggers; the number does not move."
            );
            eprintln!("render-target: median {median:.2?}, within RENDER_TARGET");
        }
        "observed" => eprintln!(
            "render-target [{variant}]: median {median:.2?}, observed — ADR-0142 states no \
             number for this variant; record it in BENCHMARK_REFERENCES"
        ),
        _ => {}
    }
}

/// The paint class's target (ADR-0144): `fixtures/benchmark/spy-trailer/` in at most
/// [`PAINT_TARGET`], median wall clock, with every frame hash equal to the trailer's
/// `frames.framemd5`.
///
/// ADR-0142's protocol, with the load checked before **every** run rather than once, since
/// a run here is short enough for the machine to change between two of them: a warm-up that
/// is discarded, then `MONTAGENT_RENDER_TARGET_RUNS` timed runs (default five; fewer refuses
/// to judge). Each run's MP4 is hashed after its clock stops. `score.wav` is generated before
/// anything is timed, where it is missing. `MONTAGENT_PAINTING` (`render::PAINTING_VAR`)
/// reaches the binary unchanged, so the K, C and W sweeps ADR-0144 records run through here,
/// and the record names what was forced.
#[test]
#[ignore = "ADR-0144's protocol: run on purpose, on the stated machine, with --release"]
fn the_spy_trailer_renders_within_the_paint_target() {
    if cfg!(debug_assertions) {
        eprintln!("refusing to judge: this is a debug build. Run with --release.");
        return;
    }
    let Ok(tools) = montagent_core::media::tools::resolve() else {
        eprintln!("refusing to judge: no qualified ffmpeg/ffprobe on PATH");
        return;
    };
    let runs: usize = env("MONTAGENT_RENDER_TARGET_RUNS")
        .map(|n| n.parse().expect("MONTAGENT_RENDER_TARGET_RUNS is a count"))
        .unwrap_or(PROTOCOL_RUNS);
    assert!(runs >= 1, "MONTAGENT_RENDER_TARGET_RUNS must be at least 1");

    let trailer =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/benchmark/spy-trailer");
    if !trailer.join("score.wav").exists() {
        eprintln!("render-target [spy-trailer]: writing score.wav (not timed)");
        let status = Command::new("uv")
            .args([
                "run",
                "--with",
                "numpy==2.5.3",
                "--with",
                "scipy==1.18.1",
                "python",
                "score.py",
            ])
            .current_dir(&trailer)
            .status()
            .expect("run score.py through uv");
        assert!(status.success(), "score.py failed");
    }
    let project = trailer.join("trailer.montagent.json");
    let reference = read_framemd5(&trailer.join("frames.framemd5"));
    let scratch = scratch("spy-trailer");
    let output = scratch.join("silent-protocol.mp4");
    let mut refused = Vec::new();

    let power = power_source();
    let cpu = sysctl("machdep.cpu.brand_string");
    if power.contains("Battery") {
        refused.push(format!("running on battery power ({power})"));
    }
    if cpu != STATED_MACHINE {
        refused.push(format!(
            "this is a {cpu}, and PAINT_TARGET is stated for the {STATED_MACHINE}"
        ));
    }
    if runs < PROTOCOL_RUNS {
        refused.push(format!(
            "{runs} timed run(s), and the protocol judges the median of {PROTOCOL_RUNS}"
        ));
    }

    let load_at_warmup = load_average();
    let warmup = render(&project, &output, &scratch.join("cache-0"), None);
    let mut mismatched = Vec::new();
    let warmup_hashes = frame_hashes(&tools.ffmpeg, &output, &scratch.join("warmup.framemd5"));
    if warmup_hashes != reference {
        mismatched.push(format!(
            "the warm-up's {} frame hashes differ from the {} of frames.framemd5{}",
            warmup_hashes.len(),
            reference.len(),
            first_difference(&warmup_hashes, &reference)
        ));
    }
    eprintln!(
        "render-target [spy-trailer] discarded warm-up: {:.2?} at load {:.2}",
        warmup.wall, load_at_warmup[0]
    );

    let mut timed = Vec::new();
    let mut loads = Vec::new();
    for n in 1..=runs {
        let _ = std::fs::remove_file(&output);
        let load = load_average();
        if load[0] >= MAX_START_LOAD {
            refused.push(format!(
                "run {n} started at load average {:.2}, and the protocol needs < {MAX_START_LOAD}",
                load[0]
            ));
        }
        let run = render(&project, &output, &scratch.join(format!("cache-{n}")), None);
        let hashes = frame_hashes(
            &tools.ffmpeg,
            &output,
            &scratch.join(format!("run-{n}.framemd5")),
        );
        if hashes != reference {
            mismatched.push(format!(
                "run {n}'s {} frame hashes differ from the {} of frames.framemd5{}",
                hashes.len(),
                reference.len(),
                first_difference(&hashes, &reference)
            ));
        }
        if comparable(&run.answer) != comparable(&warmup.answer) {
            mismatched.push(format!("run {n}'s report differs from the warm-up's"));
        }
        eprintln!(
            "render-target [spy-trailer] run {n}/{runs}: {:.2?} at load {:.2} (user {:.1} s, \
             sys {:.1} s, peak {} MiB)",
            run.wall,
            load[0],
            run.user_s,
            run.sys_s,
            run.max_rss_bytes / (1024 * 1024)
        );
        loads.push(load);
        timed.push(run);
    }
    refused.extend(mismatched.iter().cloned());

    let mut walls: Vec<Duration> = timed.iter().map(|r| r.wall).collect();
    walls.sort();
    let median = walls[walls.len() / 2];
    let verdict = if !refused.is_empty() {
        "refused"
    } else if median <= PAINT_TARGET {
        "within"
    } else {
        "exceeded"
    };
    let ms = |d: Duration| d.as_millis() as u64;
    let record = json!({
        "record": "montagent-render-target/1",
        "adr": "0144",
        "class": "paint",
        "project": "fixtures/benchmark/spy-trailer/trailer.montagent.json",
        "commit": commit(),
        "forced": env(montagent_core::verbs::render::PAINTING_VAR),
        "ffmpeg": first_line(Command::new(&tools.ffmpeg).arg("-version")),
        "macos": format!(
            "{} ({})",
            first_line(Command::new("sw_vers").arg("-productVersion")),
            first_line(Command::new("sw_vers").arg("-buildVersion"))
        ),
        "cpu": cpu,
        "memory_bytes": sysctl("hw.memsize").parse::<u64>().ok(),
        "available_parallelism": std::thread::available_parallelism().map(|n| n.get()).ok(),
        "power": power,
        "load_at_warmup": load_at_warmup,
        "warmup_ms": ms(warmup.wall),
        "runs": timed.iter().zip(&loads).map(|(r, load)| json!({
            "load_before": load,
            "wall_ms": ms(r.wall),
            "user_s": r.user_s,
            "sys_s": r.sys_s,
            "cpu_percent": ((r.user_s + r.sys_s) / r.wall.as_secs_f64() * 100.0).round(),
            "max_rss_bytes": r.max_rss_bytes,
            "stages": r.stages,
        })).collect::<Vec<_>>(),
        "median_ms": ms(median),
        "min_ms": ms(walls[0]),
        "max_ms": ms(walls[walls.len() - 1]),
        "target_ms": ms(PAINT_TARGET),
        "verdict": verdict,
        "refused": refused,
        "frame_hashes": {
            "method": "ffmpeg -f framemd5 of each run's MP4, decoded, after its clock stopped",
            "reference": "fixtures/benchmark/spy-trailer/frames.framemd5",
            "frames": reference.len(),
            "equal": mismatched.is_empty(),
        },
        "render": timed.last().map(|r| r.answer["render"].clone()),
    });
    println!("{record}");

    for reason in &refused {
        eprintln!("refusing to judge: {reason}");
    }
    assert!(
        mismatched.is_empty(),
        "a run's pixels or report differ from frames.framemd5 or the warm-up, so the run \
         does not count (ADR-0144):\n  {}",
        mismatched.join("\n  ")
    );
    if verdict != "refused" {
        assert!(
            median <= PAINT_TARGET,
            "the spy trailer's median of {runs} renders was {median:.2?}, over PAINT_TARGET \
             ({PAINT_TARGET:.0?}). ADR-0144's miss branch says what follows: re-profile, then \
             branch; the number does not move."
        );
        eprintln!("render-target [spy-trailer]: median {median:.2?}, within PAINT_TARGET");
    }
}

/// One whole `montagent render`, timed from outside the process.
struct Run {
    wall: Duration,
    user_s: f64,
    sys_s: f64,
    max_rss_bytes: u64,
    answer: Value,
    /// The span's `MONTAGENT_STAGES` line, or `null` where it printed none.
    stages: Value,
}

/// Render `project` to `output` under `/usr/bin/time -l`, with a fresh probe sidecar.
///
/// `/usr/bin/time` reports the rusage `wait4` hands back, which on macOS folds in the
/// children the process reaped — so user+sys and peak memory cover the `ffmpeg` spawns
/// as well as `montagent` itself, peak memory as the largest single process.
fn render(project: &Path, output: &Path, cache: &Path, shims: Option<&Shims>) -> Run {
    let mut command = Command::new("/usr/bin/time");
    command
        .arg("-l")
        .arg(binary())
        .arg("render")
        .arg(project)
        .arg("--output")
        .arg(output)
        .arg("--json")
        .env(montagent_core::media::sidecar::CACHE_DIR_VAR, cache)
        .env(montagent_core::verbs::render::STAGES_VAR, "1");
    if let Some(shims) = shims {
        command.env("PATH", shims.path());
    }
    let started = Instant::now();
    let out = command
        .output()
        .expect("run /usr/bin/time -l montagent render");
    let wall = started.elapsed();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "render did not answer:\n{}{stderr}",
        String::from_utf8_lossy(&out.stdout)
    );
    let answer: Value =
        serde_json::from_slice(&out.stdout).expect("the machine-readable result on stdout");

    // `       12.34 real         5.67 user         0.89 sys`
    let times = stderr
        .lines()
        .rev()
        .find(|l| l.trim_end().ends_with(" sys") && l.contains(" real "))
        .expect("/usr/bin/time's summary line");
    let field = |name: &str| -> f64 {
        let words: Vec<&str> = times.split_whitespace().collect();
        let at = words.iter().position(|w| *w == name).expect("a time field");
        words[at - 1].parse().expect("a number of seconds")
    };
    let max_rss_bytes = stderr
        .lines()
        .find(|l| l.contains("maximum resident set size"))
        .and_then(|l| l.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .expect("/usr/bin/time -l's maximum resident set size");
    let stages = stderr
        .lines()
        .find_map(|l| l.strip_prefix(montagent_core::verbs::render::STAGES_VAR))
        .and_then(|json| serde_json::from_str(json.trim()).ok())
        .unwrap_or(Value::Null);
    Run {
        wall,
        user_s: field("user"),
        sys_s: field("sys"),
        max_rss_bytes,
        answer,
        stages,
    }
}

/// The answer with what legitimately differs between two runs removed: the clock.
fn comparable(answer: &Value) -> Value {
    let mut answer = answer.clone();
    if let Some(render) = answer.get_mut("render").and_then(Value::as_object_mut) {
        render.remove("wall_ms");
        render.remove("realtime");
    }
    answer
}

/// The per-frame hashes of `mp4`'s first video stream, decoded, also written to `save`.
fn frame_hashes(ffmpeg: &Path, mp4: &Path, save: &Path) -> Vec<String> {
    let status = Command::new(ffmpeg)
        .args(["-nostdin", "-v", "error", "-y", "-i"])
        .arg(mp4)
        .args(["-map", "0:v:0", "-f", "framemd5"])
        .arg(save)
        .status()
        .expect("run ffmpeg -f framemd5");
    assert!(status.success(), "ffmpeg could not hash {}", mp4.display());
    read_framemd5(save)
}

/// A `framemd5` file's frame lines, without its `#` header (which names the muxer's
/// version and would differ across ffmpeg builds for identical pixels).
fn read_framemd5(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

fn first_difference(a: &[String], b: &[String]) -> String {
    match a.iter().zip(b).position(|(x, y)| x != y) {
        Some(n) => format!(", first at frame line {n}"),
        None => String::from(", in length"),
    }
}

/// `ffmpeg` and `ffprobe` wrappers that log each spawn and `exec` the real program.
struct Shims {
    dir: PathBuf,
}

impl Shims {
    fn path(&self) -> std::ffi::OsString {
        let mut path = self.dir.clone().into_os_string();
        path.push(":");
        path.push(std::env::var_os("PATH").unwrap_or_default());
        path
    }

    fn counts(&self) -> (usize, usize) {
        let count = |name: &str| {
            std::fs::read_to_string(self.dir.join(format!("{name}.spawns")))
                .map(|s| s.lines().count())
                .unwrap_or(0)
        };
        (count("ffmpeg"), count("ffprobe"))
    }
}

fn spawn_shims(scratch: &Path, tools: &montagent_core::media::tools::Tools) -> Shims {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch.join("shims");
    std::fs::create_dir_all(&dir).expect("create the shim dir");
    for (name, real) in [("ffmpeg", &tools.ffmpeg), ("ffprobe", &tools.ffprobe)] {
        let log = dir.join(format!("{name}.spawns"));
        let _ = std::fs::remove_file(&log);
        let shim = dir.join(name);
        std::fs::write(
            &shim,
            format!(
                "#!/bin/sh\necho x >> '{}'\nexec '{}' \"$@\"\n",
                log.display(),
                real.display()
            ),
        )
        .expect("write a shim");
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755))
            .expect("make the shim executable");
    }
    Shims { dir }
}

/// The benchmark project for `variant`: from `MONTAGENT_BENCHMARK_DIR`, or built now.
fn benchmark(variant: &str) -> PathBuf {
    let name = format!("benchmark-{variant}.montagent.json");
    let dir = match env("MONTAGENT_BENCHMARK_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => {
            let dir = std::env::temp_dir().join("montagent-render-target/benchmark");
            let script = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/benchmark/make_benchmark.py");
            let mut command = Command::new("python3");
            command.arg(&script).arg(&dir);
            if !variant.starts_with("2160") {
                command.arg("--only-1080");
            }
            eprintln!(
                "render-target: building the benchmark project into {} (not timed)",
                dir.display()
            );
            let status = command.status().expect("run make_benchmark.py");
            assert!(status.success(), "make_benchmark.py failed");
            dir
        }
    };
    let project = dir.join(&name);
    assert!(
        project.exists(),
        "no {name} in {}: MONTAGENT_BENCHMARK_VARIANT is one of 1080p30, 2160p30, \
         1080p30-1video",
        dir.display()
    );
    project
}

/// The 1, 5 and 15 minute load averages.
fn load_average() -> [f64; 3] {
    let raw = sysctl("vm.loadavg");
    let numbers: Vec<f64> = raw
        .trim_matches(|c| c == '{' || c == '}' || char::is_whitespace(c))
        .split_whitespace()
        .filter_map(|n| n.parse().ok())
        .collect();
    assert_eq!(numbers.len(), 3, "unexpected vm.loadavg: {raw}");
    [numbers[0], numbers[1], numbers[2]]
}

/// `pmset -g batt`'s first line names the source: `'AC Power'` or `'Battery Power'`.
fn power_source() -> String {
    let line = first_line(Command::new("pmset").args(["-g", "batt"]));
    match line.split('\'').nth(1) {
        Some(source) => source.to_string(),
        None => line,
    }
}

/// The commit measured, marked dirty where tracked files differ from it.
fn commit() -> String {
    let repo = env!("CARGO_MANIFEST_DIR");
    let head = first_line(Command::new("git").args(["-C", repo, "rev-parse", "HEAD"]));
    let dirty = Command::new("git")
        .args(["-C", repo, "status", "--porcelain", "--untracked-files=no"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);
    if dirty { format!("{head}-dirty") } else { head }
}

fn sysctl(name: &str) -> String {
    first_line(Command::new("sysctl").args(["-n", name]))
}

fn first_line(command: &mut Command) -> String {
    command
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_montagent"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("montagent-render-target/{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}
