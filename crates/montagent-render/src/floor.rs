//! The `ffmpeg` Montagent supports, and the arguments that decide it — each defined once.
//!
//! ADR-0115 (#477, #479). **The floor is ffmpeg 7.1 or newer, built with `libx264`**, and it
//! is stated as what a binary can *do* rather than what it says its version is: a git build
//! prints `N-xxxxx-g<hash>`, a distribution adds a suffix, and no version string reveals a
//! missing `libx264`. So the floor is the three arguments below, and an `ffmpeg` qualifies
//! when one null encode through all three succeeds ([`qualify`]).
//!
//! **Defined once, because the qualification is only worth what its link to the call sites
//! is worth.** A qualification that exercised `-fps_mode passthrough` while `decode.rs`
//! spawned `-vsync 0` would pass on ffmpeg 9 and let the seek fail anyway — which is #471
//! exactly: ffmpeg 9 removed `-vsync` and `-filter_complex_script`, and the seek painted a
//! frame up to 200 ms early with `0 errors`. So [`crate::decode`], [`crate::encode`] and
//! [`qualify`] all build from these constants, and a test below fails if either call site
//! spells one of them out by hand.
//!
//! **This is not the primary guard.** ADR-0113's rule that a failed spawn is never an empty
//! answer is, because it also covers the breakage nobody has met yet. The qualification
//! covers the ones already known, early and before any wall clock is spent — and it lets
//! `validate` say *"this `ffmpeg` cannot render"* without rendering anything.

use std::path::Path;
use std::process::{Command, Stdio};

/// The floor, as the sentence a refusal names.
pub const FLOOR: &str = "ffmpeg 7.1 or newer, built with libx264";

/// A decode that hands on every frame it selected, with its own timestamp — no frame
/// duplicated or dropped to meet an output rate.
///
/// `-fps_mode` exists since ffmpeg 5.1; `-vsync`, which it replaced, was removed in 9.0.
pub const FPS_PASSTHROUGH: [&str; 2] = ["-fps_mode", "passthrough"];

/// The option that reads a filter graph from a file, where a graph too long for a command
/// line lives.
///
/// The `-/` prefix — *"read this option's value from the named file"* — exists since ffmpeg
/// 7.0; `-filter_complex_script`, which it replaced, was removed in 9.0. Inlining the graph
/// instead is not an alternative: a large mix breaks Windows' ~32K command-line limit.
pub const FILTER_COMPLEX_FILE: &str = "-/filter_complex";

/// The video encoder every deliverable is written with (ADR-0077).
pub const VIDEO_ENCODER: &str = "libx264";

/// How much of `ffmpeg`'s own stderr a refusal carries: its last lines, where it names the
/// option or encoder it rejected. Bounded, because a report is read by an agent with a token
/// budget and `ffmpeg` can say a great deal before it says the one line that matters.
const STDERR_TAIL_LINES: usize = 4;
const STDERR_TAIL_CHARS: usize = 600;

/// An `ffmpeg` that is present and cannot do what Montagent asks of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unqualified {
    /// Which of the floor's capabilities failed, as far as `ffmpeg`'s own words say — or
    /// the whole null encode, where they name none of them.
    pub capability: String,
    /// The tail of what `ffmpeg` said, or how the spawn itself failed.
    pub said: String,
}

/// Run one null encode through every argument above, and say whether it succeeded.
///
/// **The same arguments the call sites use, on input that cannot be the problem.** Sixteen
/// black pixels on stdin, one frame, the graph read from a file through
/// [`FILTER_COMPLEX_FILE`], [`FPS_PASSTHROUGH`] on the output and [`VIDEO_ENCODER`]
/// encoding it into a null muxer. Driving the real call sites on synthetic input was
/// rejected (#477 §7): a failure there would be ambiguous between the tool and the fixture.
///
/// `scratch` is the directory the one-line graph file is written into.
pub fn qualify(ffmpeg: &Path, scratch: &Path) -> Result<(), Unqualified> {
    const SIDE: u32 = 16;
    let graph = scratch.join(format!(
        "montagent-qualify-{}-{}.filters",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    std::fs::write(&graph, "[0:v]null[v]").map_err(|e| Unqualified {
        capability: "a filter graph file".to_string(),
        said: format!("{} could not be written: {e}", graph.display()),
    })?;

    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24"])
        .args(["-s", &format!("{SIDE}x{SIDE}"), "-r", "1", "-i", "pipe:0"])
        .arg(FILTER_COMPLEX_FILE)
        .arg(&graph)
        .args(["-map", "[v]"])
        .args(FPS_PASSTHROUGH)
        .args([
            "-c:v",
            VIDEO_ENCODER,
            "-pix_fmt",
            "yuv420p",
            "-frames:v",
            "1",
        ])
        .args(["-f", "null", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let outcome = command.spawn().and_then(|mut child| {
        if let Some(mut stdin) = child.stdin.take() {
            // A write error is not the verdict: an `ffmpeg` that rejected an argument exits
            // before reading, and its exit status and stderr are what say so.
            let _ = std::io::Write::write_all(&mut stdin, &[0u8; (SIDE * SIDE * 3) as usize]);
        }
        child.wait_with_output()
    });
    let _ = std::fs::remove_file(&graph);

    let output = outcome.map_err(|e| Unqualified {
        capability: "a null encode".to_string(),
        said: format!("{} could not be run: {e}", ffmpeg.display()),
    })?;
    if output.status.success() {
        return Ok(());
    }
    let said = String::from_utf8_lossy(&output.stderr);
    Err(Unqualified {
        capability: capability_named_in(&said).to_string(),
        said: format!("it exited with {} — {}", output.status, tail(&said)),
    })
}

/// Which capability `ffmpeg`'s stderr names, where it names one of the floor's.
///
/// A reading of its prose and nothing firmer, so it only ever narrows the sentence: where it
/// matches nothing, the refusal says the null encode failed and carries the stderr, which is
/// still the whole of what is known.
fn capability_named_in(said: &str) -> &'static str {
    if said.contains(FILTER_COMPLEX_FILE.trim_start_matches('-')) {
        "reading a filter graph from a file (-/filter_complex, ffmpeg 7.0+)"
    } else if said.contains(FPS_PASSTHROUGH[0].trim_start_matches('-')) {
        "-fps_mode passthrough (ffmpeg 5.1+)"
    } else if said.contains(VIDEO_ENCODER) {
        "the libx264 encoder (an ffmpeg built with --enable-gpl --enable-libx264)"
    } else {
        "a null encode through -/filter_complex, -fps_mode passthrough and libx264"
    }
}

/// The last few non-empty lines of `said`, bounded in characters.
fn tail(said: &str) -> String {
    let lines: Vec<&str> = said
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return "it said nothing".to_string();
    }
    let joined = lines[lines.len().saturating_sub(STDERR_TAIL_LINES)..].join(" / ");
    if joined.chars().count() <= STDERR_TAIL_CHARS {
        return joined;
    }
    let skip = joined.chars().count() - STDERR_TAIL_CHARS;
    format!("…{}", joined.chars().skip(skip).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The link #477 §7 asks a test to pin: neither call site spells a version-sensitive
    /// argument by hand, and each reaches it through this module.
    ///
    /// A scan of the source rather than of a built command, because the arguments are
    /// assembled inline beside the spawn, and the failure this guards against is exactly a
    /// literal written there by someone who never opened this file.
    #[test]
    fn the_call_sites_build_their_version_sensitive_arguments_from_this_module() {
        let sites = [
            (
                "decode.rs",
                include_str!("decode.rs"),
                "floor::FPS_PASSTHROUGH",
            ),
            (
                "encode.rs",
                include_str!("encode.rs"),
                "floor::FILTER_COMPLEX_FILE",
            ),
            (
                "encode.rs",
                include_str!("encode.rs"),
                "floor::VIDEO_ENCODER",
            ),
        ];
        for (file, source, reference) in sites {
            assert!(
                source.contains(reference),
                "{file} no longer uses {reference}"
            );
        }

        // The retired spellings and the current ones, as quoted arguments. Comments may name
        // them; code may not.
        let spelled = [
            "\"-vsync\"",
            "\"-filter_complex_script\"",
            "\"-filter_complex\"",
            "\"-/filter_complex\"",
            "\"-fps_mode\"",
            "\"libx264\"",
        ];
        for (file, source) in [
            ("decode.rs", include_str!("decode.rs")),
            ("encode.rs", include_str!("encode.rs")),
        ] {
            for line in source.lines().filter(|l| !l.trim_start().starts_with("//")) {
                for literal in spelled {
                    assert!(
                        !line.contains(literal),
                        "{file} spells {literal} by hand — build it from floor.rs: {line}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_rejected_option_is_named_as_the_capability_that_failed() {
        // ffmpeg 6.1, which has no `-/` syntax.
        assert!(
            capability_named_in("Unrecognized option '/filter_complex'.\nError splitting")
                .contains("-/filter_complex")
        );
        // ffmpeg 9 on the retired spelling this module replaced.
        assert!(capability_named_in("Unrecognized option 'fps_mode'.").contains("-fps_mode"));
        // An `ffmpeg` built without `--enable-gpl`.
        assert!(capability_named_in("Unknown encoder 'libx264'").contains("libx264"));
        assert!(capability_named_in("something else entirely").starts_with("a null encode"));
    }

    #[test]
    fn the_stderr_tail_is_bounded_and_keeps_the_last_lines() {
        let said = (0..50).map(|n| format!("line {n}\n")).collect::<String>();
        let tail = tail(&said);
        assert!(tail.ends_with("line 49"), "{tail}");
        assert!(!tail.contains("line 45"), "{tail}");

        let long = "x".repeat(5_000);
        assert!(super::tail(&long).chars().count() <= STDERR_TAIL_CHARS + 1);
        assert_eq!(super::tail("\n  \n"), "it said nothing");
    }

    #[test]
    fn a_program_that_is_not_ffmpeg_does_not_qualify() {
        // Anything that exits non-zero on these arguments; `false` ignores them all.
        let Some(not_ffmpeg) = ["/usr/bin/false", "/bin/false"]
            .into_iter()
            .map(Path::new)
            .find(|p| p.exists())
        else {
            return;
        };
        let unqualified =
            qualify(not_ffmpeg, &std::env::temp_dir()).expect_err("`false` is not an ffmpeg");
        assert!(unqualified.said.contains("exited with"), "{unqualified:?}");
    }
}
