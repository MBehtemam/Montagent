//! The probe sidecar: the half of ADR-0006's cache that survives the process.
//!
//! ADR-0006 settled the local probe cache as `(path, size, mtime) → duration` and said in
//! the same breath that *"the probe cache is a gitignored sidecar"*. #190 built the cache
//! and not the sidecar, and ADR-0011 had by then made that omission expensive: the
//! document records no source duration, so `max(source_end)` is only a lower bound, and
//! ADR-0006's cache-miss line is *"not a performance optimisation with a pleasant side
//! effect — it is the sole mechanism"* catching a source that **grew** on disk. In process
//! memory that mechanism can only fire twice within one process, and `probe` is CLI-only
//! (ADR-0011), so over the CLI it could not fire at all. ADR-0069 settles the design
//! `#231` found ownerless; this module is it.
//!
//! Four decisions are worth reading off the code rather than inferring from it:
//!
//! - **It lives under the per-user cache directory, never beside the project.** ADR-0006
//!   asks for a sidecar that cannot be committed; the surest form of that is one that is
//!   not in anybody's repository. A project stays exactly ADR-0053's *"movable unit"* —
//!   the `.montagent.json` file plus its relative assets, and nothing else.
//! - **It is JSON, and human-readable**, because every other thing Montagent writes is, and
//!   a cache a person cannot read is a cache a person cannot disbelieve. ADR-0017's closed
//!   schema does not reach here: that is the *project* format, and its sidecar carve-out is
//!   about third-party annotation.
//! - **It holds the font chains too** ([`ProjectFonts`], #206). ADR-0007 asks for font
//!   files in *"ADR-0006's `(path, size, mtime)` probe cache — a font swapped in place is a
//!   silent whole-project render change that no census sees"*, and that is this cache, not
//!   a second one beside it: one file, one cache directory, one `MONTAGENT_CACHE_DIR` that
//!   turns both halves off. What is recorded is a *chain's* identity rather than a bare
//!   file's, because the two changes ADR-0007 and spec #168 story 54 name — the table
//!   edited, and the file rewritten under an unchanged table — are then one comparison. The
//!   triple is the key and [`FontEntry::sha256`] is the value, which is the same split the
//!   probe half already has.
//! - **It holds the whole [`Probe`]** — ADR-0011's quad, ADR-0023's rotation-resolved,
//!   PAR-applied dimensions, alpha, and the audio facts. This answers ADR-0023's parked
//!   sub-question in the affirmative: they are already computed by the probe that fills
//!   this entry, so storing them is the zero-extra-I/O case that ADR wondered about.
//! - **Nothing that is not a [`Probe`] is stored, and nothing remote is stored at all.**
//!   A missing, unreadable or existence-only source has no content facts to cache, and
//!   ADR-0056's no-persistent-cache decision for remote sources is untouched: the remote
//!   half of [`super::session::Session`] never reaches this module.
//!
//! **Every failure here is silence.** A sidecar that is missing, corrupt, of an unknown
//! version, or unwritable is a cache miss and nothing else. It is a fact about a cache
//! directory, and `validate` answers questions about the project; inventing a finding out
//! of one would be exactly the manufactured claim ADR-0006 exists to prevent.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::probe::Probe;

/// The sidecar's own schema version. There is no migration: a version this binary does not
/// recognise is read as an empty cache, which costs one re-probe and cannot be wrong.
///
/// **Unchanged by #206's font half**, deliberately: the new section is `#[serde(default)]`,
/// so a file written before it existed reads as a file with no font chains recorded — which
/// is exactly what it is, and which costs one silent first run rather than throwing away
/// every media probe on the machine. The cost runs the other way too, and is the same size:
/// an *older* binary reading a file this one wrote drops the font section on its next write,
/// so alternating between versions costs one silent run each time it swaps back.
///
/// **Bumped to 2 by ADR-0089**, which is the other kind of change and needs the other
/// answer. `Probe::alpha` went from a bool to a reading with its own source, and every
/// entry written at version 1 holds the old shape — including, for a VP9-in-WebM source,
/// an `alpha` of `false` that ADR-0089 establishes is wrong. A `#[serde(default)]` would
/// keep those entries and keep serving that answer, so the version carries it instead:
/// every machine re-probes once, and no cached wrong answer outlives the fix.
const VERSION: u64 = 2;

/// The file's name inside the cache directory.
const FILE: &str = "probe-cache.json";

/// How many projects' font chains the sidecar remembers, on [`MAX_ENTRIES`]'s reasoning and
/// for its cost: past this, the least recently seen project is dropped, which costs one
/// silent run on that project and never a wrong answer.
const MAX_PROJECTS: usize = 512;

/// How many sources the sidecar remembers. A global cache with no ceiling grows for the
/// life of the machine; past this, the least recently used entries are dropped, which
/// costs a re-probe and never a wrong answer.
const MAX_ENTRIES: usize = 4096;

/// What the environment variable spells. An empty value turns the sidecar off entirely,
/// which is how a run that must not persist anything says so.
pub const CACHE_DIR_VAR: &str = "MONTAGENT_CACHE_DIR";

/// One cached probe, under the identity Montagent observed when it ran.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub size: u64,
    /// Modification time in nanoseconds since the epoch, or absent on a filesystem that
    /// will not say — in which case the entry can never match again and is re-probed.
    pub mtime_ns: Option<i128>,
    /// A cheap content fingerprint of the bytes this entry's [`Probe`] was read from, and
    /// the guard that catches the one change `(path, size, mtime)` cannot see.
    ///
    /// **A value-side check, not part of the key** — the same split [`FontEntry::sha256`]
    /// already uses, and #385 chose it here for the same reason: the key decides whether
    /// the bytes are worth looking at, and the fingerprint decides whether anything
    /// actually changed.
    ///
    /// The hole it closes is a **renumbering shuffle**, which is how #385's eleven-minute
    /// silent cut happened. A plain rename moves the canonical path and so changes the key
    /// outright — that case was never the bug, whatever the ticket said. But renaming
    /// `line-01.wav`…`line-19.wav` so each file's content lands on a *neighbour's* name
    /// leaves every path in the cache still present, and `mv` preserves mtime, so the only
    /// thing left discriminating two different takes is `size`. Two dialogue lines of the
    /// same length then serve each other's probe with a straight face.
    ///
    /// Absent in an entry written before #385, which reads as "unguarded" and is trusted on
    /// its key alone — one silent run per pre-existing entry, on the same reasoning that
    /// keeps the font section `serde(default)` rather than bumping [`VERSION`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// When a run last read or wrote this entry, for the eviction order. Not a fact about
    /// the media, and nothing reads it but [`Sidecar::save`].
    pub last_used_ns: i128,
    pub probe: Probe,
}

/// One entry of one declared chain, under the identity Montagent observed.
///
/// `file` is the path **as the document spells it**, not the resolved one: a table edit
/// that repoints `brand` at a different relative path is the change spec #168's story 54 is
/// about, and it is invisible in a resolved path that happens to land on the same bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontEntry {
    pub file: String,
    pub size: u64,
    /// Modification time in nanoseconds since the epoch, or absent on a filesystem that
    /// will not say.
    pub mtime_ns: Option<i128>,
    /// The file's content hash — the *fact* this entry caches, which `(file, size,
    /// mtime_ns)` above is merely the key to.
    ///
    /// The split matters, and it is ADR-0069's own shape: there the triple keys a cache
    /// whose value is a [`Probe`], and a miss costs a re-probe rather than producing a
    /// finding. A font's mtime moves for reasons that are not edits — a fresh clone, a
    /// branch switch, a restore from backup — so a check that fired on the *key* would
    /// announce a font swap on a file whose bytes nobody touched. The key decides whether
    /// the bytes must be read again; the hash decides whether anything actually changed.
    pub sha256: String,
}

/// One project's declared chains, as they were when Montagent last looked.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectFonts {
    /// When a run last read or wrote this project's chains, for the eviction order.
    pub last_used_ns: i128,
    /// `fonts`-table key → its chain, in the order the author declared it.
    pub keys: BTreeMap<String, Vec<FontEntry>>,
}

/// The file's whole contents.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Document {
    version: u64,
    /// Keyed by the canonical path, so the file reads as a list of sources and the order
    /// is stable across writes rather than churning.
    entries: BTreeMap<String, Entry>,
    /// Keyed by the project file's canonical path. Absent in a file written before #206,
    /// which reads as "no project's chains are known yet".
    #[serde(default)]
    fonts: BTreeMap<String, ProjectFonts>,
}

/// One sidecar file, and what it held when it was read.
#[derive(Debug, Clone)]
pub struct Sidecar {
    path: PathBuf,
    entries: BTreeMap<PathBuf, Entry>,
    fonts: BTreeMap<String, ProjectFonts>,
}

impl Sidecar {
    /// Where the sidecar lives, or `None` when this machine will not say where a cache
    /// belongs — in which case the run is in-process only and nothing is written.
    pub fn default_path() -> Option<PathBuf> {
        match std::env::var_os(CACHE_DIR_VAR) {
            // Explicitly empty: the off switch, stated rather than inferred.
            Some(dir) if dir.is_empty() => None,
            Some(dir) => Some(PathBuf::from(dir).join(FILE)),
            None => cache_dir().map(|dir| dir.join("montagent").join(FILE)),
        }
    }

    /// Read the sidecar at `path`. Never fails: everything that could go wrong here is a
    /// cache miss.
    pub fn load(path: PathBuf) -> Sidecar {
        let document = read(&path);
        Sidecar {
            path,
            entries: document
                .entries
                .into_iter()
                .map(|(path, entry)| (PathBuf::from(path), entry))
                .collect(),
            fonts: document.fonts,
        }
    }

    /// The file this sidecar reads and writes.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// What the file held. The caller seeds its caches from this.
    pub fn entries(&self) -> &BTreeMap<PathBuf, Entry> {
        &self.entries
    }

    /// The font chains the file held, by project path.
    pub fn fonts(&self) -> &BTreeMap<String, ProjectFonts> {
        &self.fonts
    }

    /// Write `entries` back, pruned and capped. Never fails, for the same reason
    /// [`Sidecar::load`] cannot: a cache that could not be written is a cache that will
    /// miss next time, which is the behaviour Montagent already has to be correct under.
    pub fn save(&self, mut entries: BTreeMap<PathBuf, Entry>) {
        // An entry whose file is gone answers no future question — its key can never match
        // again, because there is nothing to stat. Dropping it here is the whole of
        // invalidation-by-deletion: there is no separate sweep and no expiry clock.
        entries.retain(|path, _| path.exists());
        evict_oldest_beyond(&mut entries, MAX_ENTRIES);

        self.write(
            Some(
                entries
                    .into_iter()
                    // A path this platform will not spell as UTF-8 is dropped rather than
                    // written through `display()`, which would mangle it into a key that can
                    // never match what a later run stats. One permanent miss, stated here,
                    // beats an entry that looks like a hit and is not.
                    .filter_map(|(path, entry)| {
                        Some((path.into_os_string().into_string().ok()?, entry))
                    })
                    .collect(),
            ),
            None,
        );
    }

    /// Record what one project's chains were observed to be, and write the font half back.
    ///
    /// **Merged into what was recorded, never replacing it.** A key whose files could not
    /// all be read is absent from `keys`, and forgetting what was recorded for it would turn
    /// one unreadable run into a permanently silent one — the next run would have nothing to
    /// compare against and would call the change a first sighting.
    ///
    /// Here rather than in the check that calls it: which map is merged, when `last_used_ns`
    /// is stamped and what the eviction order is are facts about this cache, and a check
    /// that assembled them itself would be a second place for them to drift.
    pub fn record_fonts(&self, project: String, keys: BTreeMap<String, Vec<FontEntry>>) {
        let mut fonts = self.fonts.clone();
        let entry = fonts.entry(project).or_default();
        entry.last_used_ns = now_ns();
        entry.keys.extend(keys);
        self.save_fonts(fonts);
    }

    /// Write the font half back, leaving the probe half exactly as it is on disk.
    fn save_fonts(&self, mut fonts: BTreeMap<String, ProjectFonts>) {
        // A project file that is gone answers no future question, on `save`'s reasoning.
        fonts.retain(|project, _| Path::new(project).exists());
        evict_projects_beyond(&mut fonts, MAX_PROJECTS);
        self.write(None, Some(fonts));
    }

    /// Replace one section and keep the other, reading the file again first.
    ///
    /// **The re-read is the point.** The two halves are written by different parts of one
    /// `validate` — the font check when it runs, the probe session when it is dropped — and
    /// a writer that serialised its own stale copy of the other half would silently undo
    /// whatever the first writer had just learned. Between two *processes* it is still last
    /// writer wins, which [`write_atomically`] already explains and which costs a re-probe.
    fn write(
        &self,
        entries: Option<BTreeMap<String, Entry>>,
        fonts: Option<BTreeMap<String, ProjectFonts>>,
    ) {
        let current = read(&self.path);
        let document = Document {
            version: VERSION,
            entries: entries.unwrap_or(current.entries),
            fonts: fonts.unwrap_or(current.fonts),
        };
        let Ok(text) = serde_json::to_string_pretty(&document) else {
            return;
        };
        let _ = write_atomically(&self.path, &text);
    }
}

/// What a `cache clear` did, so the caller can report it rather than guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cleared {
    /// The sidecar file, whether or not it was there.
    pub path: PathBuf,
    /// Whether a sidecar was actually removed. `false` means there was nothing to remove,
    /// which is a successful clear and not a failure.
    pub removed: bool,
    /// Temporaries removed beside it — one is left behind by a run killed between
    /// [`write_atomically`]'s write and its rename, and a `clear` that left them would
    /// leave the directory it claims to have emptied non-empty.
    pub temporaries: usize,
}

/// Delete the sidecar at `path`, and any temporary left beside it.
///
/// #385's third ask. Every other way out of a bad cache requires knowing where the file
/// lives — `~/Library/Caches/montagent/probe-cache.json` on macOS, somewhere else on every
/// other platform — which is a path a user has no reason to have memorised and that the
/// tool was the only one in a position to state.
///
/// **This is the one operation on the cache that reports a failure.** ADR-0069's
/// every-failure-is-silence rule is about a cache consulted *in passing*, where inventing a
/// finding would be a claim `validate` cannot substantiate. Here deleting the file is the
/// entire request: a `clear` that could not delete and said nothing would report success
/// for work it did not do, and the user would go on believing a cache they are debugging
/// against is gone.
pub fn clear(path: &Path) -> std::io::Result<Cleared> {
    let removed = match std::fs::remove_file(path) {
        Ok(()) => true,
        // Nothing to remove is the state the caller asked for, so it is not an error.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => return Err(e),
    };

    // Swept by the same name-plus-suffix rule `write_atomically` writes them under, so the
    // two stay in step: anything starting with the sidecar's own file name and not equal to
    // it is one of ours.
    let mut temporaries = 0;
    if let (Some(parent), Some(name)) = (path.parent(), path.file_name()) {
        // A directory that will not list is nothing to sweep, which is the silence rule.
        for entry in std::fs::read_dir(parent).into_iter().flatten().flatten() {
            let found = entry.file_name();
            if found != *name
                && found
                    .to_string_lossy()
                    .starts_with(&*name.to_string_lossy())
                && std::fs::remove_file(entry.path()).is_ok()
            {
                temporaries += 1;
            }
        }
    }

    Ok(Cleared {
        path: path.to_path_buf(),
        removed,
        temporaries,
    })
}

/// The file as it is on disk right now, or an empty document.
///
/// Everything that could go wrong here is a cache miss: missing, corrupt, or of a version
/// this binary does not recognise all read the same way.
fn read(path: &Path) -> Document {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Document>(&text).ok())
        .filter(|document| document.version == VERSION)
        .unwrap_or_default()
}

/// Drop the least recently seen projects until at most `limit` remain.
fn evict_projects_beyond(fonts: &mut BTreeMap<String, ProjectFonts>, limit: usize) {
    if fonts.len() <= limit {
        return;
    }
    let mut recency: Vec<(i128, String)> = fonts
        .iter()
        .map(|(project, entry)| (entry.last_used_ns, project.clone()))
        .collect();
    recency.sort();
    for (_, project) in recency.into_iter().take(fonts.len() - limit) {
        fonts.remove(&project);
    }
}

/// Drop the least recently used entries until at most `limit` remain.
fn evict_oldest_beyond(entries: &mut BTreeMap<PathBuf, Entry>, limit: usize) {
    if entries.len() <= limit {
        return;
    }
    let mut recency: Vec<(i128, PathBuf)> = entries
        .iter()
        .map(|(path, entry)| (entry.last_used_ns, path.clone()))
        .collect();
    recency.sort();
    for (_, path) in recency.into_iter().take(entries.len() - limit) {
        entries.remove(&path);
    }
}

/// Write via a temporary file and one rename, so a second Montagent running at the same
/// time reads either the old file or the new one and never half of either.
///
/// Between two concurrent runs the last writer wins, and what the other one learned is
/// lost. That is a re-probe on some later run, which is the one consequence every code
/// path here is already correct under; the alternative — a lock file — would make a cache
/// able to block a `validate`, which is a worse trade than paying for one probe twice.
fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Appended rather than substituted for the extension, so the one ignore rule this
    // project states (`probe-cache.json*`) covers the temporary too — a run killed between
    // the write and the rename must not leave an un-ignored file behind.
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    std::fs::write(&temporary, text)?;
    match std::fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&temporary);
            Err(e)
        }
    }
}

/// How much of a file's head and tail the content guard reads. 64 KiB from each end.
///
/// Not the whole file, deliberately. The guard runs on every cache *hit*, so its cost is
/// paid by the case ADR-0069 promises costs nothing but an `ffprobe` — and hashing a 2 GB
/// ProRes master to confirm it is unchanged would spend more than the probe it saves. Head
/// and tail together cover what distinguishes two media files of equal length: the head
/// carries the container header, the codec configuration and the stream metadata, and the
/// tail carries the trailing index — `moov` in a faststart MP4, the cues in a WebM. Two
/// different takes of the same byte length agreeing on both is the residual blind spot, and
/// it is smaller than the one this closes by the size of the guard.
const GUARD_BYTES: u64 = 64 * 1024;

/// A cheap content fingerprint of the file at `path`, or `None` where it cannot be read.
///
/// `None` is not a mismatch. It means the guard established nothing, and a guard that
/// established nothing must not be allowed to invalidate an entry — that would turn an
/// unreadable moment into a re-probe storm, and ADR-0069's rule is that every failure in
/// this module is silence.
pub fn fingerprint(path: &Path) -> Option<String> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).ok()?;
    let size = file.metadata().ok()?.len();

    // The size goes into the hash as well as keying the entry, so a head and tail that
    // happen to coincide across two different lengths still fingerprint differently.
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    sha2::Digest::update(&mut hasher, size.to_le_bytes());

    let mut head = vec![0u8; size.min(GUARD_BYTES) as usize];
    file.read_exact(&mut head).ok()?;
    sha2::Digest::update(&mut hasher, &head);

    // Only where the file is long enough for the tail to be bytes the head did not already
    // cover; below that the head is the whole file and reading it twice says nothing new.
    if size > GUARD_BYTES * 2 {
        file.seek(SeekFrom::End(-(GUARD_BYTES as i64))).ok()?;
        let mut tail = vec![0u8; GUARD_BYTES as usize];
        file.read_exact(&mut tail).ok()?;
        sha2::Digest::update(&mut hasher, &tail);
    }

    Some(format!("{:x}", sha2::Digest::finalize(hasher)))
}

/// Now, in nanoseconds since the epoch. Only ever compared against itself.
pub fn now_ns() -> i128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i128)
        .unwrap_or(0)
}

/// This platform's per-user cache directory, by its own convention.
///
/// Resolved here rather than by a dependency: it is three environment variables and a
/// join, and ADR-0064 ships Montagent as a binary whose dependency list is something a
/// reader can hold in their head.
fn cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home().map(|home| home.join("Library").join("Caches"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        match std::env::var_os("XDG_CACHE_HOME") {
            Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
            _ => home().map(|home| home.join(".cache")),
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(last_used_ns: i128) -> Entry {
        Entry {
            size: 1,
            mtime_ns: Some(0),
            content: None,
            last_used_ns,
            probe: Probe {
                source: "x".into(),
                identity: None,
                quad: Default::default(),
                dimensions: None,
                alpha: None,
                codec_name: None,
                audio: None,
            },
        }
    }

    #[test]
    fn a_corrupt_or_unknown_sidecar_reads_as_an_empty_cache() {
        let dir = std::env::temp_dir().join("montagent-sidecar-unit");
        std::fs::create_dir_all(&dir).unwrap();

        let corrupt = dir.join("corrupt.json");
        std::fs::write(&corrupt, b"{ this is not json").unwrap();
        assert!(Sidecar::load(corrupt).entries().is_empty());

        let future = dir.join("future.json");
        std::fs::write(&future, br#"{"version": 99, "entries": {}}"#).unwrap();
        assert!(Sidecar::load(future).entries().is_empty());

        assert!(Sidecar::load(dir.join("absent.json")).entries().is_empty());
    }

    #[test]
    fn eviction_keeps_the_most_recently_used() {
        let mut entries = BTreeMap::new();
        for i in 0..5i128 {
            entries.insert(PathBuf::from(format!("/{i}")), entry(i));
        }
        evict_oldest_beyond(&mut entries, 2);

        assert_eq!(entries.len(), 2);
        assert!(entries.contains_key(Path::new("/3")));
        assert!(entries.contains_key(Path::new("/4")));
    }
}
