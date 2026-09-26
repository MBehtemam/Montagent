//! Resolving `ffmpeg` and `ffprobe` from `PATH`, and the one failure that is exit 70.
//!
//! ADR-0009 ships Montagent as *"a binary, plus an `ffmpeg` the user supplies"* — no
//! bundled FFmpeg, because the GPL and patent duties rule it out. That makes "is there an
//! `ffmpeg` on this machine, and where" a question the tool must answer out loud rather
//! than discover halfway through a render.
//!
//! **Resolution lives here, at the first tool that spawns a subprocess**, and not in the
//! packaging ticket downstream of every consumer: `probe`, `render`, `frame` and
//! `preview` all need the same answer, and three of them would otherwise each invent it.
//!
//! An absent binary is ADR-0011's **exit 70** — *"internal failure (ffmpeg died, font
//! stack failed) → retry or report"*. It is not exit 1: nothing is wrong with the
//! project, and telling an author to fix their timings because their `PATH` is short is
//! the mis-signalling the five exit codes exist to prevent.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::report::Report;

/// The two programs Montagent spawns, resolved to absolute paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

/// A program that was looked for and not found, carrying enough to act on.
///
/// ADR-0011's exit 70 says *"retry or report"*, and neither is possible from *"ffmpeg
/// failed"*. So the finding names the program, every directory that was searched, and —
/// where resolution succeeded and the spawn did not — the path that was resolved and what
/// the OS said about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    /// The program's own name, as it was looked for.
    pub program: &'static str,
    /// The `PATH` entries searched, in order.
    pub searched: Vec<PathBuf>,
    /// The path that was resolved, where one was, and the failure it then produced.
    pub resolved: Option<PathBuf>,
    pub failure: String,
}

impl Missing {
    /// The sentence exit 70 carries. It names the resolved path when there is one and the
    /// search when there is not, because those are two different next moves: install it,
    /// versus look at the thing that is already there.
    pub fn reason(&self) -> String {
        match &self.resolved {
            Some(path) => format!(
                "{} resolved to {} and could not be run: {}",
                self.program,
                path.display(),
                self.failure
            ),
            None => format!(
                "{} was not found on PATH: {}. Looked in: {}",
                self.program,
                self.failure,
                if self.searched.is_empty() {
                    "nothing — PATH is unset or empty".to_string()
                } else {
                    self.searched
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ),
        }
    }

    /// ADR-0011's exit 70, as the report every adapter already knows how to print.
    ///
    /// ADR-0091: never found on `PATH` at all is `E-TOOL-MISSING` — the shape ADR-0009
    /// chose ("a binary, plus an `ffmpeg` the user supplies"), not a break in Montagent.
    /// Resolved and then unable to run — a corrupt or incompatible binary — stays
    /// `E-INTERNAL`, which is the narrower thing that name should mean.
    pub fn into_report(self) -> Report {
        if self.resolved.is_none() {
            Report::tool_missing(self.reason())
        } else {
            Report::internal_failure(self.reason())
        }
    }

    /// The same distinction, for a run that already has a report open and has found more
    /// to say than this one failure (`render`, `preview`, `validate`'s probe half).
    pub fn fail(&self, report: &mut Report) {
        if self.resolved.is_none() {
            report.fail_tool_missing(self.reason());
        } else {
            report.fail_internally(self.reason());
        }
    }
}

/// Resolve both programs from the process's own `PATH`.
pub fn resolve() -> Result<Tools, Missing> {
    resolve_in(std::env::var_os("PATH").as_deref())
}

/// Resolve both programs from one `PATH` value.
///
/// Split out so the absent case is testable without mutating the process environment —
/// the failure this module exists for is the one hardest to reproduce by accident.
pub fn resolve_in(path_var: Option<&std::ffi::OsStr>) -> Result<Tools, Missing> {
    let entries: Vec<PathBuf> = match path_var {
        Some(value) => std::env::split_paths(value).collect(),
        None => Vec::new(),
    };

    Ok(Tools {
        ffmpeg: find("ffmpeg", &entries)?,
        ffprobe: find("ffprobe", &entries)?,
    })
}

/// The first entry in `entries` holding an executable named `program`.
fn find(program: &'static str, entries: &[PathBuf]) -> Result<PathBuf, Missing> {
    for entry in entries {
        for name in candidate_names(program) {
            let candidate = entry.join(&name);
            if is_executable(&candidate) {
                return Ok(candidate);
            }
        }
    }

    Err(Missing {
        program,
        searched: entries.to_vec(),
        resolved: None,
        failure: "no executable of that name in any PATH entry".to_string(),
    })
}

/// The filenames one program can wear. On Windows an executable carries an extension from
/// `PATHEXT`; everywhere else the bare name is the only spelling.
fn candidate_names(program: &str) -> Vec<OsString> {
    if !cfg!(windows) {
        return vec![OsString::from(program)];
    }

    let extensions = std::env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".EXE;.COM"));
    let extensions = extensions.to_string_lossy().to_string();
    extensions
        .split(';')
        .filter(|ext| !ext.is_empty())
        .map(|ext| OsString::from(format!("{program}{}", ext.to_ascii_lowercase())))
        .collect()
}

/// Whether a path names a file this process could execute.
///
/// On Unix that is the mode bits; elsewhere, being a file is as much as the filesystem
/// will say, and the spawn itself is the remaining test — which [`Missing::resolved`]
/// exists to report.
fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ExitCode;

    #[test]
    fn an_empty_path_names_the_program_and_says_nothing_was_searched() {
        let missing = resolve_in(None).expect_err("an unset PATH resolves nothing");

        assert_eq!(missing.program, "ffmpeg");
        assert!(missing.searched.is_empty());
        let reason = missing.reason();
        assert!(reason.contains("ffmpeg"), "{reason}");
        assert!(reason.contains("PATH is unset or empty"), "{reason}");
    }

    #[test]
    fn a_path_that_holds_neither_program_is_exit_70_naming_what_was_looked_for() {
        let empty = std::env::temp_dir().join("montagent-no-ffmpeg-here");
        std::fs::create_dir_all(&empty).expect("a directory with no ffmpeg in it");

        let missing = resolve_in(Some(empty.as_os_str())).expect_err("nothing to find");
        let report = missing.clone().into_report();

        assert_eq!(report.exit_code(), ExitCode::Internal);
        let rendered = crate::wire::render(&report, crate::Wire::Text { verbose: false });
        assert!(rendered.contains("ffmpeg"), "{rendered}");
        assert!(
            rendered.contains(&empty.display().to_string()),
            "the directories searched are the actionable half: {rendered}"
        );
    }

    #[test]
    fn a_resolved_path_that_will_not_run_reports_the_path_rather_than_the_search() {
        let missing = Missing {
            program: "ffprobe",
            searched: vec![PathBuf::from("/usr/bin")],
            resolved: Some(PathBuf::from("/usr/bin/ffprobe")),
            failure: "Exec format error (os error 8)".to_string(),
        };

        let reason = missing.reason();
        assert!(reason.contains("/usr/bin/ffprobe"), "{reason}");
        assert!(reason.contains("Exec format error"), "{reason}");
        assert!(
            !reason.contains("Looked in"),
            "a program that was found is not a search failure: {reason}"
        );
    }
}
