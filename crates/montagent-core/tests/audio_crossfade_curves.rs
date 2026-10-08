//! The audio crossfade's measured acceptance check (ADR-0176 §7, ADR-0173): a port of
//! `docs/research/audio-effects/audio-crossfade/check_audio_crossfade_curves.py` onto the
//! real mix graph.
//!
//! **Side of the encoder: PCM.** The document is rendered through `montagent`'s own mix
//! graph ([`montagent_core::verbs::render::mix_audio`]), with the encoder swapped for ffmpeg's
//! `astats` meter, so AAC's high-band trim never mixes into a tolerance (ADR-0173 §3).
//!
//! **The meter and its parse (ADR-0173 §2).** `astats` prints `RMS level dB: <f>` and
//! `Peak level dB: <f>` under its `Overall` block on stderr at `-loglevel info`. [`measure`]
//! reads the *last* such pair and panics, naming the stderr, if a line is absent or
//! unparseable — so a changed print format fails loudly rather than reading as 0 dB.
//!
//! **Tolerance.** [`TOLERANCE_DB`] is `astats`'s print precision rounded up to a figure set
//! before any three-leg run (ADR-0176 §7). It is **provisional**: ADR-0173 §4 makes it
//! `max(2 × the spread across the three CI legs, the meter's resolution)`, and that table is
//! produced by CI, not by this test. Until it is committed ADR-0176 stays `proposed`.

use std::path::{Path, PathBuf};
use std::process::Command;

use montagent_core::verbs::render::mix_audio;

mod common;
use common::{canonical, has_ffprobe, tempdir, write_project};

/// Provisional (see the module doc): the midpoint levels are asserted two-sided to this.
const TOLERANCE_DB: f64 = 0.10;

fn ffmpeg() -> PathBuf {
    montagent_core::media::tools::resolve()
        .expect("an ffmpeg")
        .ffmpeg
}

/// A 6 s mono WAV from a lavfi source expression, as `f32` PCM.
fn source(dir: &Path, name: &str, lavfi: &str) -> String {
    let path = dir.join(name);
    let made = Command::new(ffmpeg())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(lavfi)
        .args(["-c:a", "pcm_f32le"])
        .arg(&path)
        .status()
        .expect("ffmpeg runs")
        .success();
    assert!(made, "{lavfi}");
    path.display().to_string().replace('\\', "/")
}

/// `a` over 0..4000 and `b` over 2000..6000 (the window is 2000..4000, 2 s as in the
/// script), bridged by an `audio_crossfade` of `curve` — or by nothing, for the reference.
fn project(dir: &Path, name: &str, a: &str, b: &str, curve: Option<&str>) -> PathBuf {
    let bridge = match curve {
        Some(curve) => format!(
            r##",{{"name":"bridge","layer":2,"elements":[{{"id":"t","type":"transition",
              "start":2000,"end":4000,"kind":"audio_crossfade","from":"a","to":"b",
              "audio":"{curve}"}}]}}"##
        ),
        None => String::new(),
    };
    let body = canonical(&format!(
        r##"{{"frame":{{"width":200,"height":200}},"fps":25,"duration":6000,
            "tracks":[
              {{"name":"first","layer":0,"elements":[
                {{"id":"a","type":"audio","start":0,"end":4000,"source":"{a}",
                  "source_start":0,"source_end":4000}}]}},
              {{"name":"second","layer":1,"elements":[
                {{"id":"b","type":"audio","start":2000,"end":6000,"source":"{b}",
                  "source_start":0,"source_end":4000}}]}}{bridge}]}}"##
    ));
    write_project(dir, &format!("{name}.montagent.json"), &body)
}

/// `(RMS dB, Peak dB)` of the mix over `[from_sample, to_sample)`, read by `astats` from the
/// PCM of the graph `montagent` writes — no encoder.
fn measure(path: &Path, from_sample: u64, to_sample: u64) -> (f64, f64) {
    let (inputs, graph) = mix_audio(path, 0, 6000)
        .expect("the mix is built")
        .expect("something is audible");
    let mut command = Command::new(ffmpeg());
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "info",
            "-f",
            "lavfi",
            "-i",
        ])
        // Input 0 is the video pipe in a render; the meter has no video, so a stand-in.
        .arg("anullsrc=r=48000:cl=mono:d=0.1");
    for input in &inputs {
        command.arg("-i").arg(input);
    }
    let filter = format!(
        "{graph};[mix]atrim=start_sample={from_sample}:end_sample={to_sample},\
         astats=measure_perchannel=none:measure_overall=RMS_level+Peak_level[meter]"
    );
    let out = command
        .arg("-filter_complex")
        .arg(filter)
        .args(["-map", "[meter]", "-f", "null", "-"])
        .output()
        .expect("ffmpeg runs");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "{stderr}");
    let overall = stderr
        .rsplit_once("Overall")
        .unwrap_or_else(|| panic!("astats printed no `Overall` block:\n{stderr}"))
        .1;
    let read = |label: &str| -> f64 {
        let line = overall
            .lines()
            .find_map(|line| line.split_once(label).map(|(_, rest)| rest))
            .unwrap_or_else(|| panic!("astats printed no `{label}`:\n{stderr}"));
        match line.trim() {
            "-inf" => f64::NEG_INFINITY,
            number => number
                .parse()
                .unwrap_or_else(|_| panic!("`{label}` is not a number: {number:?}\n{stderr}")),
        }
    };
    (read("RMS level dB:"), read("Peak level dB:"))
}

const RATE: u64 = 48_000;

/// Midpoint level against one side alone: the script's four numbers.
#[test]
fn the_midpoint_level_is_the_curves_definition() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let tone = |name: &str, hz: u32| {
        source(
            &dir,
            name,
            &format!("sine=frequency={hz}:sample_rate=48000:duration=6,volume=0.5"),
        )
    };
    let (f500, f750) = (tone("f500.wav", 500), tone("f750.wav", 750));
    // 40 ms either side of the 3 s midpoint: 20 and 30 whole cycles, so the two sines are
    // orthogonal over the measured span.
    let (mid_from, mid_to) = (2980 * RATE / 1000, 3020 * RATE / 1000);
    // One side alone: `a` before the window (1000..1040 ms).
    let alone = measure(
        &project(&dir, "alone", &f500, &f500, None),
        1000 * RATE / 1000,
        1040 * RATE / 1000,
    )
    .0;
    for (case, second, want) in [
        (
            "unrelated",
            &f750,
            [("constant_power", 0.0), ("constant_gain", -3.01)],
        ),
        (
            "same signal",
            &f500,
            [("constant_power", 3.01), ("constant_gain", 0.0)],
        ),
    ] {
        for (curve, expected) in want {
            let name = format!("{case}-{curve}").replace(' ', "-");
            let path = project(&dir, &name, &f500, second, Some(curve));
            let got = measure(&path, mid_from, mid_to).0 - alone;
            // Printed so a CI leg's number can be read off its log into the three-leg table.
            eprintln!("measured {case} {curve}: {got:+.3} dB");
            assert!(
                (got - expected).abs() <= TOLERANCE_DB,
                "{case} {curve}: {got:.3} dB against one side alone, expected {expected} ± {TOLERANCE_DB}"
            );
        }
    }
}

/// The window edges on a constant signal, one sample at a time, read as peak level against
/// the same document with no transition (the bypass computed in the same run).
#[test]
fn the_window_is_sample_exact() {
    if !has_ffprobe() {
        return;
    }
    let dir = tempdir(line!());
    let level = |name: &str, value: f64| {
        source(&dir, name, &format!("aevalsrc={value}:s=48000:d=6:c=mono"))
    };
    // 0.5 on the outgoing side, 0.25 on the incoming, so the two are told apart by level.
    let (a, b) = (level("half.wav", 0.5), level("quarter.wav", 0.25));
    let bypass = project(&dir, "bypass", &a, &b, None);
    let faded = project(&dir, "faded", &a, &b, Some("constant_power"));
    let (s0, s1) = (2000 * RATE / 1000, 4000 * RATE / 1000);
    let peak = |path: &Path, sample: u64| measure(path, sample, sample + 1).1;

    // The sample before the window: the outgoing side, untouched — identical to bypass.
    assert_eq!(peak(&faded, s0 - 1), peak(&bypass, s0 - 1));
    // The window's first sample: the outgoing side at full level and the incoming at 0, so
    // the mix is still the outgoing side alone — the level of the sample before it.
    assert_eq!(peak(&faded, s0), peak(&faded, s0 - 1));
    // The window's end: the outgoing side is gone and the incoming is full — identical to
    // bypass, where the incoming side is alone too.
    assert_eq!(peak(&faded, s1), peak(&bypass, s1));
    // Inside the window the sum is neither side alone.
    assert_ne!(peak(&faded, s0 + 24_000), peak(&faded, s0 - 1));
}
