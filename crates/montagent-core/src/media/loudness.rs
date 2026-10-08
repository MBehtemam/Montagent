//! Integrated loudness and true peak, by ffmpeg's `ebur128=peak=true` (ADR-0173 §2).
//!
//! One meter for the three places that ask: `render`'s measurement pass over the whole
//! programme's mix (ADR-0172), `verify`'s reading of the deliverable, and the repo tests. So a
//! number means the same thing in each.
//!
//! **The parse is pinned.** The meter prints a summary on stderr at `-loglevel info`:
//!
//! ```text
//!   Integrated loudness:
//!     I:         -23.0 LUFS
//!   …
//!   True peak:
//!     Peak:       -0.5 dBFS
//! ```
//!
//! [`parse`] reads the `I:` line after `Integrated loudness:` and the `Peak:` line after
//! `True peak:`, and fails, quoting the tail of stderr, if either is absent: a changed print
//! format must never read as a measurement.
//!
//! **Undefined loudness.** When every 400 ms block is gated out, BS.1770's integrated loudness
//! is undefined, and the meter prints its floor of −70 LUFS. Every defined value is above the
//! absolute gate, so −70 and below reads as undefined ([`Summary::integrated`] is `None`).

use crate::media::probe::{Execution, ProcessRunner, Runner};
use std::path::{Path, PathBuf};

/// What the meter said about one stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    /// Integrated loudness in LUFS, or `None` where every block was gated out.
    pub integrated: Option<f64>,
    /// True peak in dBTP, or `None` for digital silence (`-inf`).
    pub true_peak: Option<f64>,
}

/// EBU R 128's absolute gate. An integrated reading at or below it is the meter's floor.
const GATE_LUFS: f64 = -70.0;

/// The pinned parse of `ebur128=peak=true`'s summary.
pub fn parse(stderr: &str) -> Result<Summary, String> {
    let summary = stderr
        .rsplit_once("Summary:")
        .map(|(_, rest)| rest)
        .ok_or_else(|| broken(stderr, "no `Summary:`"))?;
    let after = |header: &str, label: &str| -> Result<Option<f64>, String> {
        let rest = summary
            .split_once(header)
            .map(|(_, rest)| rest)
            .ok_or_else(|| broken(stderr, header))?;
        let line = rest
            .lines()
            .find_map(|line| line.trim().strip_prefix(label))
            .ok_or_else(|| broken(stderr, label))?;
        let number = line
            .split_whitespace()
            .next()
            .ok_or_else(|| broken(stderr, label))?;
        match number {
            "-inf" => Ok(None),
            number => number
                .parse::<f64>()
                .map(Some)
                .map_err(|_| broken(stderr, label)),
        }
    };
    let integrated = after("Integrated loudness:", "I:")?.filter(|lufs| *lufs > GATE_LUFS);
    let true_peak = after("True peak:", "Peak:")?;
    Ok(Summary {
        integrated,
        true_peak,
    })
}

fn broken(stderr: &str, missing: &str) -> String {
    let tail: String = {
        let chars: Vec<char> = stderr.chars().collect();
        chars[chars.len().saturating_sub(600)..].iter().collect()
    };
    format!("the ebur128 summary did not parse (no {missing}); it now reads:\n{tail}")
}

/// The arguments that meter a file's first audio stream, for a [`crate::media::probe::Runner`].
pub fn file_args(file: &Path) -> Vec<String> {
    [
        "-hide_banner",
        "-nostats",
        "-nostdin",
        "-loglevel",
        "info",
        "-protocol_whitelist",
        crate::media::probe::LOCAL_PROTOCOLS,
        "-i",
    ]
    .iter()
    .map(|s| s.to_string())
    .chain([
        file.display().to_string(),
        "-map".into(),
        "0:a:0".into(),
        "-af".into(),
        "ebur128=peak=true".into(),
        "-f".into(),
        "null".into(),
        "-".into(),
    ])
    .collect()
}

/// Meter the `[mix]` a render's audio graph labels, with no encoder: the graph's inputs are
/// opened as `[1:a]`, `[2:a]`…, and input `0`, which is the video pipe in a render, is a
/// silent stand-in. The graph goes to `ffmpeg` as a script file, as the encoder's does.
pub fn of_graph(ffmpeg: &Path, inputs: &[PathBuf], graph: &str) -> Result<Summary, String> {
    let script = scratch("ebur128-graph");
    let metered = format!("{graph};\n[mix]ebur128=peak=true[meter]\n");
    std::fs::write(&script, metered)
        .map_err(|e| format!("{} could not be written: {e}", script.display()))?;
    let mut args: Vec<String> = [
        "-hide_banner",
        "-nostats",
        "-nostdin",
        "-loglevel",
        "info",
        "-f",
        "lavfi",
        "-i",
        "anullsrc=r=48000:cl=stereo:d=0.01",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for input in inputs {
        args.push("-i".into());
        args.push(input.display().to_string());
    }
    args.push(montagent_render::floor::FILTER_COMPLEX_FILE.into());
    args.push(script.display().to_string());
    args.extend(["-map", "[meter]", "-f", "null", "-"].map(String::from));
    // Through the one audited spawn (ADR-0115 §2): the exit status is read before the output.
    let ran = ProcessRunner.run(ffmpeg, &args);
    let _ = std::fs::remove_file(&script);
    let Execution {
        success, stderr, ..
    } = ran.map_err(|e| format!("{} could not be run: {e}", ffmpeg.display()))?;
    if !success {
        return Err(format!(
            "{} could not meter the mix: {}",
            ffmpeg.display(),
            stderr.lines().last().unwrap_or("").trim()
        ));
    }
    parse(&stderr)
}

/// A path in the temp directory no other call in this process or any other will pick.
fn scratch(stem: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "montagent-{stem}-{}-{}.txt",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUMMARY: &str = "[Parsed_ebur128_0 @ 0x1] Summary:\n\n  Integrated loudness:\n    \
I:         -27.1 LUFS\n    Threshold: -37.1 LUFS\n\n  Loudness range:\n    LRA:        20.0 LU\n\
    Threshold: -47.1 LUFS\n    LRA low:   -47.1 LUFS\n    LRA high:  -27.1 LUFS\n\n  True peak:\n\
    Peak:      -24.1 dBFS\n";

    #[test]
    fn the_summary_reads_back_its_two_numbers() {
        assert_eq!(
            parse(SUMMARY).unwrap(),
            Summary {
                integrated: Some(-27.1),
                true_peak: Some(-24.1)
            }
        );
    }

    #[test]
    fn a_gated_out_mix_has_no_integrated_loudness_and_silence_no_peak() {
        let silent = SUMMARY
            .replace(
                "-27.1 LUFS\n    Threshold: -37.1",
                "-70.0 LUFS\n    Threshold:   0.0",
            )
            .replace("-24.1 dBFS", "-inf dBFS");
        assert_eq!(
            parse(&silent).unwrap(),
            Summary {
                integrated: None,
                true_peak: None
            }
        );
    }

    #[test]
    fn a_changed_print_format_fails_loudly() {
        assert!(parse("nothing here").is_err());
        assert!(parse(&SUMMARY.replace("Peak:", "Pk:")).is_err());
        assert!(parse(&SUMMARY.replace("I:   ", "Int:")).is_err());
    }
}
