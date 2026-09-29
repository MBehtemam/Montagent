//! Media facts: the one authority on what a media file's numbers are.
//!
//! ADR-0011: *"There must be exactly one authority on what a media file's numbers are,
//! and that turns out to be genuinely ambiguous."* On the fixture's own reference MP4 the
//! container says 65.258667 s, the video stream says 65.216016 s, `r_frame_rate` says
//! 50/1 and `avg_frame_rate` says 24.9785 — and the project declares `duration: 65216`,
//! `fps: 25`, matching the **stream** and **neither** frame rate. An agent that computes
//! `frames = duration × fps` from the wrong pairing is out by 2×. So [`probe`] returns
//! the quad and forces the caller to pick; a single scalar named `duration` is the
//! failure mode, not the convenience.
//!
//! The pieces:
//!
//! - [`attest`] reads back the stamp `render` writes into a deliverable, so *"did this
//!   project produce the file already at the output path?"* is observed rather than guessed
//!   (ADR-0104).
//! - [`tools`] resolves `ffmpeg`/`ffprobe` from `PATH`. This is owned here, at the first
//!   tool that spawns a subprocess, rather than downstream of every consumer.
//! - [`probe`] runs `ffprobe` and turns its output into [`probe::Probe`] — the quad, the
//!   dimensions, alpha, sample rate and channels, in integer milliseconds and exact
//!   rationals.
//! - [`established`] is the one structure every verb reads to ask *"what did the engine
//!   establish about this file?"* — ADR-0093, and the fix for `validate` and `render`
//!   answering that question from two data structures that could disagree.
//! - [`dimensions`] is ADR-0023's one type-generic pipeline: decode, resolve rotation,
//!   apply PAR as an exact rational, round once. Images are the degenerate case.
//! - [`session`] holds the caches — `(path, size, mtime)` for local files, URL-keyed
//!   deduplication for remote ones — and reports every miss unprompted (ADR-0006,
//!   ADR-0056).
//! - [`sidecar`] is where the local half persists: a per-user JSON cache, read when a
//!   session opens and written when it ends, so the cache-miss line survives a process
//!   boundary (ADR-0069). The remote half never reaches it.

pub mod attest;
pub mod dimensions;
pub mod established;
pub mod probe;
pub mod session;
pub mod sidecar;
pub mod tools;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Where a `source` points. ADR-0053: *"`source` may be a relative path, an absolute
/// local path, or a URL."*
///
/// The two halves are different in kind, not merely in spelling: a local file has a
/// `(size, mtime)` the tool observes itself, and a remote URL has neither and costs a
/// round trip (ADR-0056). Keeping them apart in the type is what lets
/// [`session::Session`] give each the cache it has actually earned, and what makes *"no
/// unsolicited network call"* a property of the code rather than a promise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Local(PathBuf),
    Remote(String),
}

impl Source {
    /// Classify one `source` value, resolving a relative path against the directory the
    /// project file lives in.
    ///
    /// ADR-0053 rejected `assetRoot` and left the project file's own directory as the
    /// one thing a relative path resolves against — *"a project is a movable unit: the
    /// `.montagent.json` file plus its relative"* assets.
    pub fn resolve(source: &str, project_dir: &Path) -> Source {
        match url_scheme(source) {
            // `file:` is a URL spelling of a local path, not a remote source: reading it
            // costs no round trip and has a `(size, mtime)`. Its path is absolute by
            // construction — `file:///clips/a.mov` — so it replaces the base rather than
            // being joined onto it, and joining it was a bug that produced `…/file:/clips`
            // and never resolved. Percent-escapes are not decoded: no ADR asks for it, and
            // a path decoded on the way in is a path `validate` would report back in a
            // spelling the document does not contain.
            Some(scheme) if scheme == "file" => {
                Source::Local(PathBuf::from(&source["file://".len()..]))
            }
            Some(_) => Source::Remote(source.to_string()),
            None => Source::Local(project_dir.join(source)),
        }
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, Source::Remote(_))
    }
}

/// A local path, spelled the way a person or an agent would type it.
///
/// [`Path::display`] is otherwise the right call at this boundary — `validate`'s `CACHE`
/// and `MEDIA` blocks, and every finding naming a local source, print whatever the caller
/// resolved. On Windows that can be `std::fs::canonicalize`'s own extended-length spelling —
/// a real, correct path, and also a Win32 API detail nobody writes by hand. Stripping it
/// here, at render rather than at resolution, keeps every canonicalisation upstream free to
/// keep doing its job — the prefix is still exactly what a symlink or a long path resolves
/// to, only never what gets printed.
///
/// Two spellings, not one: a drive path's is `\\?\C:\...`, where the four characters are
/// simply cut. A UNC share's is `\\?\UNC\server\share\...`, where cutting the same four
/// characters would leave `UNC\server\share\...` — not a path at all, since the leading
/// `\\` a UNC path needs is gone. That case has its own rule: drop `\\?\UNC` and keep the
/// slashes, so `\\?\UNC\server\share\...` becomes `\\server\share\...`.
pub fn display_local(path: &Path) -> String {
    let text = path.display().to_string();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
}

/// The scheme of a URL, or `None` if the string is a path.
///
/// Deliberately strict: a scheme is a letter followed by letters, digits, `+`, `-` or
/// `.`, then `://`. A bare `C:\clips\take3.mov` is a path (`C` is a drive, and there is
/// no `//`), and so is anything with a colon in a directory name.
fn url_scheme(source: &str) -> Option<String> {
    let (scheme, _) = source.split_once("://")?;
    let mut chars = scheme.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')) {
        return None;
    }
    Some(scheme.to_ascii_lowercase())
}

/// An exact integer ratio — a frame rate, or a pixel aspect ratio.
///
/// ADR-0023: PAR is *"an exact-integer-pair rational"*, and ADR-0011 keeps both frame
/// rates rather than a float, because `avg_frame_rate` printed as `24.9785` is already a
/// lossy rendering of `18871/755`. Nothing here divides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rational {
    pub num: i64,
    pub den: i64,
}

impl Rational {
    pub const ONE: Rational = Rational { num: 1, den: 1 };

    pub fn new(num: i64, den: i64) -> Rational {
        Rational { num, den }
    }

    /// Parse `ffprobe`'s `N/D` spelling, or `N:D` for a sample aspect ratio.
    ///
    /// `0/0` and `0:1` are `ffprobe`'s two spellings of *"the file does not say"*, and
    /// both come back `None` rather than as a ratio that would divide by zero or
    /// annihilate a dimension.
    pub fn parse(text: &str) -> Option<Rational> {
        let (num, den) = text.split_once(['/', ':'])?;
        let num: i64 = num.trim().parse().ok()?;
        let den: i64 = den.trim().parse().ok()?;
        if num <= 0 || den <= 0 {
            return None;
        }
        Some(Rational { num, den })
    }
}

/// Convert `ffprobe`'s decimal-seconds spelling to integer milliseconds, exactly.
///
/// ADR-0005 stores every time as an integer millisecond because float seconds failed in
/// practice — two agents produced `4.400000000000002` and `14.832999999999998` in their
/// own runs. Parsing `"65.258667"` into an `f64` and multiplying by 1000 would reintroduce
/// exactly that error inside the one tool the whole surface treats as authoritative, so
/// this walks the decimal string instead and never constructs a float.
///
/// **It truncates toward zero**, which is a decision and not a rounding accident: a
/// duration reported one millisecond long is a claim that media exists which does not,
/// and `E-SOURCE-OVERRUN` is built on comparing a declared span against this number.
/// Under-claiming is the safe direction. On the fixture's reference MP4 the two spellings
/// agree anyway — 65.216016 s truncates to the 65216 the project declares.
pub fn milliseconds(decimal_seconds: &str) -> Option<i64> {
    let text = decimal_seconds.trim();
    if text.is_empty() || text.eq_ignore_ascii_case("N/A") {
        return None;
    }

    let (negative, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };

    let (whole, fraction) = rest.split_once('.').unwrap_or((rest, ""));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    if !whole.bytes().all(|b| b.is_ascii_digit()) || !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let seconds: i64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    // Three digits, padded or truncated — truncation toward zero, in the string.
    let mut millis = 0i64;
    for i in 0..3 {
        let digit = fraction
            .as_bytes()
            .get(i)
            .map_or(0, |b| i64::from(b - b'0'));
        millis = millis * 10 + digit;
    }

    let total = seconds.checked_mul(1000)?.checked_add(millis)?;
    Some(if negative { -total } else { total })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_url_is_remote_and_a_path_is_not() {
        let dir = Path::new("/p");
        assert_eq!(
            Source::resolve("https://cdn.example/take3.mov", dir),
            Source::Remote("https://cdn.example/take3.mov".into())
        );
        assert_eq!(
            Source::resolve("images/06.png", dir),
            Source::Local("/p/images/06.png".into())
        );
        // ADR-0053 permits an absolute local path, and `join` keeps it whole.
        assert_eq!(
            Source::resolve("/Volumes/footage/take3.mov", dir),
            Source::Local("/Volumes/footage/take3.mov".into())
        );
        // A colon in a directory name is not a scheme.
        assert!(!Source::resolve("odd:name/06.png", dir).is_remote());

        // `file:` is a spelling of a local path, and its path is absolute — joining it
        // onto the project directory would produce `/p/file:/clips/take3.mov`.
        assert_eq!(
            Source::resolve("file:///clips/take3.mov", dir),
            Source::Local("/clips/take3.mov".into())
        );
    }

    #[test]
    fn a_windows_verbatim_prefix_is_stripped_from_the_displayed_path() {
        // #239: `canonicalize()` on Windows returns `\\?\C:\...`, a correct path and also
        // a Win32 API detail nobody types. It never appears in a report.
        assert_eq!(
            display_local(Path::new(r"\\?\C:\a\images\06.png")),
            r"C:\a\images\06.png"
        );
        // Any other path — including one with no drive letter at all — is untouched.
        assert_eq!(
            display_local(Path::new("/p/images/06.png")),
            "/p/images/06.png"
        );
        // The UNC form of the same prefix carries its own `UNC\` segment, which is not
        // part of the share's own spelling: cutting only `\\?\` would leave `UNC\server\...`,
        // a string with no leading `\\` and so not a path a person could open.
        assert_eq!(
            display_local(Path::new(r"\\?\UNC\fileserver\media\clip.mp4")),
            r"\\fileserver\media\clip.mp4"
        );
    }

    #[test]
    fn decimal_seconds_become_milliseconds_without_a_float() {
        // ADR-0011's own table, which is where these three numbers come from.
        assert_eq!(milliseconds("65.216016"), Some(65216));
        assert_eq!(milliseconds("65.258667"), Some(65258));
        assert_eq!(milliseconds("0.042031"), Some(42));

        assert_eq!(milliseconds("0"), Some(0));
        assert_eq!(milliseconds("1.5"), Some(1500));
        assert_eq!(milliseconds("-0.5"), Some(-500));
        assert_eq!(milliseconds("N/A"), None);
        assert_eq!(milliseconds(""), None);
        assert_eq!(milliseconds("1e3"), None);
    }

    #[test]
    fn a_ratio_the_file_does_not_state_is_not_a_ratio() {
        assert_eq!(Rational::parse("50/1"), Some(Rational::new(50, 1)));
        assert_eq!(Rational::parse("40:33"), Some(Rational::new(40, 33)));
        assert_eq!(Rational::parse("0/0"), None);
        assert_eq!(Rational::parse("0:1"), None);
        assert_eq!(Rational::parse("nonsense"), None);
    }
}
