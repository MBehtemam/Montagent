//! **A diagnostic, not a gate**: where do the x64 Windows runner's differing MP4 bytes come
//! from?
//!
//! On `x86_64-pc-windows-msvc` (windows-2022, ffmpeg 9.0.2 from BtbN or gyan), `painters.rs`
//! and `letter_spacing.rs` intermittently see two renders hand the encoder byte-identical
//! frames and get MP4s a few hundred bytes apart. Linux x64 (the 7.1 floor), macOS and both
//! arm64 legs never do, and one libx264 thread (PR #868) gives the same differing pairs as
//! five. This test renders `painters.rs`'s glow project many times per **encoder variant**,
//! each variant pinning one more part of `ffmpeg` through the `#[doc(hidden)]`, per-thread,
//! test-only `encode::force_extra_args` (and `render::force_encoder`), and prints how many
//! distinct MP4s each variant wrote. The variant whose count drops to one names the part.
//!
//! | variant | what it adds to the production (ADR-0143) command line |
//! |---|---|
//! | `V0` | nothing: the production encode |
//! | `V1` | `-filter_threads 1 -filter_complex_threads 1` (global), `-threads 1` on the raw input |
//! | `V2` | `-sws_flags +bitexact+accurate_rnd+full_chroma_int` (the auto-inserted rgb24 to yuv420p) |
//! | `V3` | `-x264-params threads=1:lookahead-threads=1:sliced-threads=0:deterministic=1` |
//! | `V4` | `-fps_mode passthrough`, with the output `-r` left out (`ffmpeg` refuses both) |
//! | `V5` | V1 + V2 + V3 + V4 |
//! | `V6` | `-cpuflags 0` (no ffmpeg SIMD) and `-x264-params asm=0` (no x264 asm) |
//! | `V7` | `encode::Settings { threads: 1, ..PRODUCTION }`: PR #868's one libx264 thread |
//!
//! Environment, read by this test only — nothing in the product reads any of it:
//!
//! - `MONTAGENT_TEST_ENCODER_VARIANT`: `V0`…`V7`, a comma list, or `all`. Unset: `V0`.
//! - `MONTAGENT_DIAG_RUNS`: renders per variant, cycling one painter, K=3 C=2, K=2 C=1 and
//!   K=4 C=5. Unset: 4, so the suite's own run of this test on every leg stays cheap.
//! - `MONTAGENT_DIAG_PARALLEL`: renders in flight at once, each on its own copy of the project,
//!   to stand in for the load of the full suite. Unset: 1.
//! - `MONTAGENT_DIAG_STRICT=1`: fail when a variant wrote more than one distinct MP4.
//!   Unset, the test only prints: it is gathering data, and a differing variant is an answer.
//! - `MONTAGENT_KEEP_FAILED_MP4_DIR`: where to write the first differing pair of each variant.
//!
//! Read the output (`-- --nocapture`) by its `DIAG-SUMMARY` lines, one per variant and
//! parallelism. The MP4's ADR-0117 stamp names the project's path, so renders are compared
//! only within one project copy; `distinct_mp4` is the most distinct files any copy wrote.
//! `distinct_mp4=1` is deterministic over that many runs; more than one prints,
//! below it, each distinct file's size, count and x264 options SEI, then the first differing
//! pair's first differing byte, packet table and decoded `framemd5`, which say whether the
//! encoder decided differently (decoded frames differ) or only the container did (they do not).

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use montagent_core::media::tools;
use montagent_core::report::ExitCode;
use montagent_core::verbs::render::{
    Ask, Forced, Progress, force_encoder, force_painting, render, tap_frames,
};
use montagent_render::encode::{ExtraArgs, Settings, force_extra_args};
use serde_json::{Value, json};

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// How long one small render may take before it counts as hung.
const HUNG: Duration = Duration::from_secs(120);

const V1_THREADS_GLOBAL: &[&str] = &["-filter_threads", "1", "-filter_complex_threads", "1"];
const V1_THREADS_INPUT: &[&str] = &["-threads", "1"];
const V2_SWS: &[&str] = &["-sws_flags", "+bitexact+accurate_rnd+full_chroma_int"];
const V3_X264: &[&str] = &[
    "-x264-params",
    "threads=1:lookahead-threads=1:sliced-threads=0:deterministic=1",
];
const V4_FPS: &[&str] = &["-fps_mode", "passthrough"];
const V5_OUTPUT: &[&str] = &[
    "-sws_flags",
    "+bitexact+accurate_rnd+full_chroma_int",
    "-x264-params",
    "threads=1:lookahead-threads=1:sliced-threads=0:deterministic=1",
    "-fps_mode",
    "passthrough",
];
const V6_GLOBAL: &[&str] = &["-cpuflags", "0"];
const V6_OUTPUT: &[&str] = &["-x264-params", "asm=0"];

const NONE: ExtraArgs = ExtraArgs {
    global: &[],
    input: &[],
    output: &[],
    omit_output_rate: false,
};

struct Variant {
    name: &'static str,
    extra: ExtraArgs,
    settings: Settings,
}

const VARIANTS: &[Variant] = &[
    Variant {
        name: "V0",
        extra: NONE,
        settings: Settings::PRODUCTION,
    },
    Variant {
        name: "V1",
        extra: ExtraArgs {
            global: V1_THREADS_GLOBAL,
            input: V1_THREADS_INPUT,
            ..NONE
        },
        settings: Settings::PRODUCTION,
    },
    Variant {
        name: "V2",
        extra: ExtraArgs {
            output: V2_SWS,
            ..NONE
        },
        settings: Settings::PRODUCTION,
    },
    Variant {
        name: "V3",
        extra: ExtraArgs {
            output: V3_X264,
            ..NONE
        },
        settings: Settings::PRODUCTION,
    },
    Variant {
        name: "V4",
        extra: ExtraArgs {
            output: V4_FPS,
            omit_output_rate: true,
            ..NONE
        },
        settings: Settings::PRODUCTION,
    },
    Variant {
        name: "V5",
        extra: ExtraArgs {
            global: V1_THREADS_GLOBAL,
            input: V1_THREADS_INPUT,
            output: V5_OUTPUT,
            omit_output_rate: true,
        },
        settings: Settings::PRODUCTION,
    },
    Variant {
        name: "V6",
        extra: ExtraArgs {
            global: V6_GLOBAL,
            output: V6_OUTPUT,
            ..NONE
        },
        settings: Settings::PRODUCTION,
    },
    Variant {
        name: "V7",
        extra: NONE,
        settings: Settings {
            threads: std::num::NonZeroU32::MIN,
            ..Settings::PRODUCTION
        },
    },
];

/// The painting configurations the runs cycle through, as `painters.rs` compares them.
const PAINTING: [Forced; 4] = [
    Forced::OnePainter,
    Forced::Chunks {
        painters: 3,
        chunk: 2,
        window_bytes: None,
    },
    Forced::Chunks {
        painters: 2,
        chunk: 1,
        window_bytes: None,
    },
    Forced::Chunks {
        painters: 4,
        chunk: 5,
        window_bytes: None,
    },
];

fn env_number(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .filter(|&n: &usize| n > 0)
        .unwrap_or(default)
}

fn selected() -> Vec<&'static Variant> {
    let asked = std::env::var("MONTAGENT_TEST_ENCODER_VARIANT").unwrap_or_default();
    let asked = asked.trim();
    if asked.is_empty() {
        return vec![&VARIANTS[0]];
    }
    if asked.eq_ignore_ascii_case("all") {
        return VARIANTS.iter().collect();
    }
    asked
        .split(',')
        .map(|name| {
            let name = name.trim();
            VARIANTS
                .iter()
                .find(|variant| variant.name.eq_ignore_ascii_case(name))
                .unwrap_or_else(|| panic!("no encoder variant {name:?}: V0..V7 or all"))
        })
        .collect()
}

/// One render's outcome.
struct Run {
    /// Which project copy it rendered. The MP4's ADR-0117 stamp carries the project's path,
    /// so only renders of one copy can be compared byte for byte.
    worker: usize,
    painting: Forced,
    /// One hash of every frame pushed, in order.
    frames: u64,
    frame_count: usize,
    mp4: Vec<u8>,
}

/// One distinct MP4 among a project copy's runs.
#[derive(Clone)]
struct Distinct {
    hash: u64,
    size: usize,
    count: usize,
    /// The painting of each run that wrote it.
    paintings: Vec<String>,
    /// The first run that wrote it.
    first: usize,
}

fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Render `path` once with `variant` and `painting`, on a thread of its own.
fn render_once(path: &Path, worker: usize, variant: &'static Variant, painting: Forced) -> Run {
    let path = path.to_path_buf();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _painting = force_painting(painting);
        let _settings = force_encoder(variant.settings);
        let _extra = force_extra_args(variant.extra);
        let tap = tap_frames();
        let answer = render(&path, &Ask::default(), &mut |_: Progress| {});
        assert_eq!(
            answer.report().exit_code(),
            ExitCode::Ok,
            "{}: {}",
            variant.name,
            answer.to_json()
        );
        let mp4 = answer
            .video()
            .map(|video| std::fs::read(&video.path).expect("the published file"))
            .expect("a published MP4");
        let frames = tap.hashes();
        let _ = tx.send(Run {
            worker,
            painting,
            frames: hash_of(&frames),
            frame_count: frames.len(),
            mp4,
        });
    });
    rx.recv_timeout(HUNG)
        .expect("the render finished inside the timeout rather than hanging or failing")
}

/// `runs` renders of `variant`, `parallel` at a time, each worker on its own project copy so
/// no two renders share an output path.
fn runs_of(variant: &'static Variant, runs: usize, parallel: usize, line: u32) -> Vec<Run> {
    let projects: Vec<PathBuf> = (0..parallel)
        .map(|worker| glow_project(line * 100 + worker as u32))
        .collect();
    let mut done = Vec::with_capacity(runs);
    let mut next = 0;
    while next < runs {
        let batch: Vec<_> = (0..parallel.min(runs - next))
            .map(|worker| {
                let path = projects[worker].clone();
                let painting = PAINTING[(next + worker) % PAINTING.len()];
                std::thread::spawn(move || render_once(&path, worker, variant, painting))
            })
            .collect();
        next += batch.len();
        done.extend(
            batch
                .into_iter()
                .map(|h| h.join().expect("a render worker")),
        );
    }
    done
}

/// The x264 options SEI libx264 writes into every stream it encodes, as text.
fn x264_options(mp4: &[u8]) -> String {
    let needle = b"x264 - core";
    let Some(at) = mp4.windows(needle.len()).position(|w| w == needle) else {
        return "(no x264 SEI)".to_string();
    };
    let end = mp4[at..]
        .iter()
        .position(|&b| b == 0)
        .map_or(mp4.len(), |n| at + n);
    String::from_utf8_lossy(&mp4[at..end]).into_owned()
}

/// `ffprobe`'s packet table and `ffmpeg`'s decoded `framemd5` of one MP4, one line each.
fn packets_and_frames(mp4: &[u8], file: &Path) -> (Vec<String>, Vec<String>) {
    std::fs::write(file, mp4).expect("write the MP4 to read back");
    let Ok(tools) = tools::resolve() else {
        return (Vec::new(), Vec::new());
    };
    let lines = |out: std::io::Result<std::process::Output>| -> Vec<String> {
        match out {
            Ok(out) => String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|line| !line.starts_with('#'))
                .map(str::to_string)
                .collect(),
            Err(e) => vec![format!("(could not run: {e})")],
        }
    };
    let packets = lines(
        Command::new(&tools.ffprobe)
            .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
            .args(["packet=pts,dts,duration,size,flags", "-of", "csv=p=0"])
            .arg(file)
            .output(),
    );
    let frames = lines(
        Command::new(&tools.ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-i"])
            .arg(file)
            .args(["-map", "0:v", "-f", "framemd5", "-"])
            .output(),
    );
    (packets, frames)
}

/// Print what differs between the first two distinct MP4s of a variant.
fn explain(name: &str, one: &[u8], other: &[u8], scratch: &Path) {
    let first_byte = one
        .iter()
        .zip(other)
        .position(|(a, b)| a != b)
        .unwrap_or(one.len().min(other.len()));
    eprintln!(
        "  {name}: first differing pair {} vs {} bytes, first differing byte at {first_byte}",
        one.len(),
        other.len()
    );
    // The bytes around it, printable ASCII as is and the rest as `.`, to tell an atom name
    // or a metadata string from the coded stream at a glance.
    let around = |bytes: &[u8]| -> String {
        let from = first_byte.saturating_sub(16);
        let to = (first_byte + 32).min(bytes.len());
        bytes
            .get(from..to)
            .unwrap_or_default()
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect()
    };
    eprintln!("    a: {:?}", around(one));
    eprintln!("    b: {:?}", around(other));
    let (packets_a, frames_a) = packets_and_frames(one, &scratch.join("a.mp4"));
    let (packets_b, frames_b) = packets_and_frames(other, &scratch.join("b.mp4"));
    let differing = |a: &[String], b: &[String]| -> Vec<usize> {
        (0..a.len().max(b.len()))
            .filter(|&i| a.get(i) != b.get(i))
            .collect()
    };
    let packets = differing(&packets_a, &packets_b);
    eprintln!(
        "  {name}: packets {} vs {}; {} differ (pts,dts,duration,size,flags), first at {:?}",
        packets_a.len(),
        packets_b.len(),
        packets.len(),
        packets.first()
    );
    for &i in packets.iter().take(8) {
        eprintln!(
            "    packet {i}: {} | {}",
            packets_a.get(i).map_or("-", String::as_str),
            packets_b.get(i).map_or("-", String::as_str)
        );
    }
    let frames = differing(&frames_a, &frames_b);
    eprintln!(
        "  {name}: decoded frames {} vs {}; {} differ (framemd5), indices {:?}",
        frames_a.len(),
        frames_b.len(),
        frames.len(),
        frames.iter().take(16).collect::<Vec<_>>()
    );
}

#[test]
fn diagnose_mp4_bytes_against_equal_encoder_input_per_encoder_variant() {
    if !has_ffprobe() {
        return;
    }
    let runs = env_number("MONTAGENT_DIAG_RUNS", 4);
    let parallel = env_number("MONTAGENT_DIAG_PARALLEL", 1);
    let strict = std::env::var("MONTAGENT_DIAG_STRICT").is_ok_and(|v| v.trim() == "1");
    let scratch = tempdir(line!());
    let mut nondeterministic = Vec::new();

    for variant in selected() {
        let done = runs_of(variant, runs, parallel, line!());
        let frames: BTreeMap<(u64, usize), usize> =
            done.iter().fold(BTreeMap::new(), |mut seen, run| {
                *seen.entry((run.frames, run.frame_count)).or_default() += 1;
                seen
            });
        // Per project copy (the stamp names the copy), the distinct MP4s in first-seen order.
        let mut workers: Vec<Vec<Distinct>> = vec![Vec::new(); parallel];
        for (i, run) in done.iter().enumerate() {
            let hash = hash_of(&run.mp4[..]);
            let painting = match run.painting {
                Forced::OnePainter => "K1".to_string(),
                Forced::Chunks {
                    painters, chunk, ..
                } => format!("K{painters}C{chunk}"),
            };
            let mp4s = &mut workers[run.worker];
            match mp4s.iter_mut().find(|entry| entry.hash == hash) {
                Some(entry) => {
                    entry.count += 1;
                    entry.paintings.push(painting);
                }
                None => mp4s.push(Distinct {
                    hash,
                    size: run.mp4.len(),
                    count: 1,
                    paintings: vec![painting],
                    first: i,
                }),
            }
        }
        let distinct = workers.iter().map(Vec::len).max().unwrap_or(0);
        let sizes: Vec<String> = workers
            .iter()
            .map(|mp4s| {
                mp4s.iter()
                    .map(|entry| format!("{}x{}", entry.size, entry.count))
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .collect();
        eprintln!(
            "DIAG-SUMMARY {} parallel={parallel} runs={runs} distinct_mp4={distinct} distinct_frames={} sizes_per_copy=[{}] extra={:?} threads={}",
            variant.name,
            frames.len(),
            sizes.join(", "),
            variant.extra,
            variant.settings.threads,
        );
        let mut seis: Vec<String> = Vec::new();
        for (worker, mp4s) in workers.iter().enumerate() {
            for Distinct {
                hash,
                size,
                count,
                paintings,
                first,
            } in mp4s
            {
                // Each distinct SEI in full once, then by number.
                let sei = x264_options(&done[*first].mp4);
                let shown = match seis.iter().position(|seen| *seen == sei) {
                    Some(n) => format!("SEI #{n}"),
                    None => {
                        seis.push(sei.clone());
                        format!("SEI #{}: {sei}", seis.len() - 1)
                    }
                };
                eprintln!(
                    "  {} copy {worker}: mp4 {hash:016x} {size} bytes x{count} (first run {first}; {}) {shown}",
                    variant.name,
                    paintings.join(" "),
                );
            }
        }
        if let Some(mp4s) = workers.iter().find(|mp4s| mp4s.len() > 1) {
            nondeterministic.push(variant.name);
            let (one, other) = (&done[mp4s[0].first].mp4, &done[mp4s[1].first].mp4);
            explain(variant.name, one, other, &scratch);
            common::keep_differing_mp4s(
                &format!("diag-{}-parallel{parallel}", variant.name),
                Some(one),
                Some(other),
            );
        }
    }

    if strict {
        assert!(
            nondeterministic.is_empty(),
            "variants that wrote more than one MP4 from equal frames: {nondeterministic:?}"
        );
    } else if !nondeterministic.is_empty() {
        eprintln!(
            "DIAG: more than one MP4 from equal frames under {nondeterministic:?} (not failing: MONTAGENT_DIAG_STRICT is not 1)"
        );
    }
}

/// The trailer's directory, for its title face and a photo.
fn trailer() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/benchmark/spy-trailer")
}

/// `painters.rs`'s CI guard project, copied: a `blur` and a zero-offset `shadow` glow over
/// text, shapes and a still, 320x180 at 30 fps for 1100 ms (33 frames), everything moving.
fn glow_project(line: u32) -> PathBuf {
    let dir = tempdir(line);
    for asset in ["fonts/Cinzel-Bold.ttf", "img/boat.jpg"] {
        let to = dir.join(asset);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("asset dir");
        std::fs::copy(trailer().join(asset), &to).expect("copy the trailer's asset");
    }
    let moving =
        |from: i64, to: i64| json!([{"t": 0, "v": from}, {"t": 1100, "v": to, "ease": "linear"}]);
    let blur = json!({"name": "blur", "radius": 6});
    let glow = json!({"name": "shadow", "dx": 0, "dy": 0, "radius": 12, "color": "#FF9F2E", "opacity": 0.8});
    let elements = [
        json!({"id": "boat", "type": "image", "source": "img/boat.jpg", "start": 0, "end": 1100,
               "x": moving(150, 170), "y": 90, "origin": "center", "width": 320, "height": 181,
               "fit": "cover", "effects": [blur]}),
        json!({"id": "boat-inset", "type": "image", "source": "img/boat.jpg", "start": 400, "end": 1100,
               "x": moving(40, 60), "y": 40, "origin": "center", "width": 64, "height": 36,
               "fit": "cover", "effects": [glow]}),
        json!({"id": "title", "type": "text", "font": "cinzel-bold", "size": 40, "color": "#E3C067",
               "align": "center", "runs": [{"text": "SPY"}], "width": 200, "height": 60,
               "start": 0, "end": 1100, "x": moving(130, 190), "y": 90, "origin": "center",
               "effects": [glow]}),
        json!({"id": "subtitle", "type": "text", "font": "cinzel-bold", "size": 16, "color": "#FFFFFF",
               "align": "center", "runs": [{"text": "SILENT PROTOCOL"}], "width": 200, "height": 24,
               "start": 500, "end": 1100, "x": 160, "y": moving(150, 140), "origin": "center",
               "effects": [blur]}),
        json!({"id": "bar", "type": "rect", "start": 0, "end": 1100, "x": moving(10, 250), "y": 160,
               "origin": "top-left", "width": 60, "height": 8, "fill": "#F2F2F2",
               "effects": [{"name": "blur", "radius": 3}]}),
        json!({"id": "dot", "type": "ellipse", "start": 0, "end": 1100, "x": moving(290, 230),
               "y": 30, "origin": "center", "width": 16, "height": 16, "fill": "#FFFFFF",
               "effects": [glow]}),
    ];
    let tracks: Vec<Value> = elements
        .into_iter()
        .enumerate()
        .map(|(i, element)| json!({"name": format!("t{i}"), "layer": i, "elements": [element]}))
        .collect();
    let project = json!({
        "frame": {"width": 320, "height": 180}, "fps": 30, "background": "#101418",
        "duration": 1100, "output": "out/glow.mp4",
        "fonts": {"cinzel-bold": [{"file": "fonts/Cinzel-Bold.ttf"}]},
        "fontVendor": {"fonts/Cinzel-Bold.ttf": {
            "licence": "OFL-1.1",
            "source": "google/fonts ofl/cinzel, instanced wght=700",
            "sha256": "c9ac320a5f48ecb57db76bc0b942ccc39eb89994e37fb182a454ec273ee43e02"}},
        "tracks": tracks,
    });
    write_project(
        &dir,
        "glow.montagent.json",
        &canonical(&project.to_string()),
    )
}
