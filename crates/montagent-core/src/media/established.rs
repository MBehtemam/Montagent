//! What the check engine established about the media on disk, as **one** structure every
//! verb reads.
//!
//! ADR-0093, ruling 3. The MONTAGENT-1 defect was not a missing severity class — it was
//! that `validate` and `render` answered one question from two data structures.
//! `validate` mapped a probe [`Outcome`](crate::media::probe::Outcome) onto findings;
//! `render` reached into [`Report::media`] and compared canonical paths itself. So
//! `validate` reported `0 unchecked` on a file `render` then declined to mix 134 elements
//! of, in the same session, and neither verb was wrong about its own data.
//!
//! This module is that one data path. The rule it makes structural:
//!
//! > **`validate`'s unchecked set contains every source `render` declines to use.**
//!
//! It is a containment and not an equality on purpose. `render` mixes local sources only,
//! so it declines a remote one that `validate` probed perfectly well — a fact about the
//! verb, not about the file. What must never happen is the other direction: a source
//! `render` cannot use that `validate` called a clean pass. `tests/cross_verb.rs` asserts
//! it as a property over both verbs rather than as a unit test on either, because a unit
//! test on one path is exactly what let the two drift apart.
//!
//! The containment is the reason [`Established::about`] is the only way to ask. A consumer
//! that re-derives usability from [`Report::media`] has forked the data path again.

use std::path::{Path, PathBuf};

use crate::report::Report;

/// Everything the engine established about one file, for a consumer that has to decide
/// whether it may use it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facts {
    /// Whether the file carries an audio stream. An ordinary video with none is not a
    /// defect — it is simply not in the mix, and ADR-0093 makes that a `note`.
    pub audio: bool,
}

/// The engine's answer about every file it observed, read by `validate` and by `render`.
///
/// Built from a [`Report`] rather than carried alongside one: the report *is* what the
/// check engine established, and a second copy could disagree with it.
#[derive(Debug, Clone, Default)]
pub struct Established {
    /// One entry per probe that observed a file on this disk, under ADR-0092's observed
    /// identity — never under `Probe::source`, which is a label.
    observed: Vec<(PathBuf, Facts)>,
}

impl Established {
    /// Read what the engine established off the report it established it on.
    pub fn of(report: &Report) -> Established {
        Established {
            observed: report
                .media
                .iter()
                .filter_map(|probe| {
                    Some((
                        probe.identity.clone()?,
                        Facts {
                            audio: probe.audio.is_some(),
                        },
                    ))
                })
                .collect(),
        }
    }

    /// What the engine established about the file at `identity`, or `None` where it
    /// established nothing a consumer may match.
    ///
    /// `identity` must be a canonical path — the caller's own observation of the file it
    /// is about to use. ADR-0092: comparing anything else is comparing spellings, which is
    /// how the working directory became a third input to `render`.
    pub fn about(&self, identity: &Path) -> Option<Facts> {
        self.observed
            .iter()
            .find(|(known, _)| known == identity)
            .map(|(_, facts)| *facts)
    }
}
