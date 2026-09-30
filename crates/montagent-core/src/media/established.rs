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
//! *Unchecked* here means *not a clean pass*: an `error` about the same source is a strictly
//! louder statement of the same thing, and it is the class ADR-0131 gives a remote one.
//!
//! It is a containment and not an equality: `validate` may say more than `render` declines,
//! never less. What must never happen is a source `render` cannot use that `validate` called
//! a clean pass. `tests/cross_verb.rs` asserts it as a property over both verbs rather than
//! as a unit test on either, because a unit test on one path is exactly what let the two
//! drift apart.
//!
//! There is no exception. ADR-0093 carved one for a remote source — `render` mixes and draws
//! local sources only, so it declines a URL `validate` probed perfectly well — and that
//! carve-out was the same *clean pass, then refused* the invariant exists to prevent.
//! ADR-0131 took it out: [`local`] is the one place a consumer that mixes or draws turns a
//! [`Source`] into a file it may open, and `validate` asks it the same question `render`
//! and `frame` do.
//!
//! The containment is the reason [`Established::about`] and [`local`] are the only ways to
//! ask. A consumer that re-derives usability from [`Report::media`], or its own reading of
//! a `source`, has forked the data path again.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::finding::Finding;
use crate::media::Source;
use crate::report::Report;

/// What a verb does with a sourced element's file: `render` mixes `audio` and `video`, and
/// the painter draws `image` and `video`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Use {
    Mix,
    Paint,
}

impl Use {
    /// Every use a verb makes of an element of this `type` — both, for a `video`.
    pub fn of(kind: Option<&str>) -> &'static [Use] {
        match kind {
            Some("audio") => &[Use::Mix],
            Some("image") => &[Use::Paint],
            Some("video") => &[Use::Mix, Use::Paint],
            _ => &[],
        }
    }

    /// The code a remote source is declined at, for this use.
    fn remote(self) -> &'static str {
        match self {
            Use::Mix => "E-NOT-MIXED-REMOTE",
            Use::Paint => "E-NOT-PAINTED-REMOTE",
        }
    }
}

/// The file a consumer that mixes or draws may open for `source`, or the finding that says
/// why it may not.
///
/// ADR-0131: `render` and `frame` use **local** sources only. A URL stays legal and
/// `validate` still probes it (ADR-0056), and whatever that probe established, nothing here
/// fetches it — so the answer for a remote source is the refusal, at the code for `used`.
/// `file:` URLs are local ([`Source::resolve`]), and reach here as paths.
pub fn local(source: Source, used: Use) -> Result<PathBuf, Box<Finding>> {
    match source {
        Source::Local(path) => Ok(path),
        Source::Remote(url) => Err(Box::new(
            Finding::new(used.remote()).field("source", json!(url)),
        )),
    }
}

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
