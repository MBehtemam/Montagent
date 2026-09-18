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
//!   the `.montaget.json` file plus its relative assets, and nothing else.
//! - **It is JSON, and human-readable**, because every other thing Montaget writes is, and
//!   a cache a person cannot read is a cache a person cannot disbelieve. ADR-0017's closed
//!   schema does not reach here: that is the *project* format, and its sidecar carve-out is
//!   about third-party annotation.
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
const VERSION: u64 = 1;

/// The file's name inside the cache directory.
const FILE: &str = "probe-cache.json";

/// How many sources the sidecar remembers. A global cache with no ceiling grows for the
/// life of the machine; past this, the least recently used entries are dropped, which
/// costs a re-probe and never a wrong answer.
const MAX_ENTRIES: usize = 4096;

/// What the environment variable spells. An empty value turns the sidecar off entirely,
/// which is how a run that must not persist anything says so.
pub const CACHE_DIR_VAR: &str = "MONTAGET_CACHE_DIR";

/// One cached probe, under the identity Montaget observed when it ran.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub size: u64,
    /// Modification time in nanoseconds since the epoch, or absent on a filesystem that
    /// will not say — in which case the entry can never match again and is re-probed.
    pub mtime_ns: Option<i128>,
    /// When a run last read or wrote this entry, for the eviction order. Not a fact about
    /// the media, and nothing reads it but [`Sidecar::save`].
    pub last_used_ns: i128,
    pub probe: Probe,
}

/// The file's whole contents.
#[derive(Debug, Serialize, Deserialize)]
struct Document {
    version: u64,
    /// Keyed by the canonical path, so the file reads as a list of sources and the order
    /// is stable across writes rather than churning.
    entries: BTreeMap<String, Entry>,
}

/// One sidecar file, and what it held when it was read.
#[derive(Debug, Clone)]
pub struct Sidecar {
    path: PathBuf,
    entries: BTreeMap<PathBuf, Entry>,
}

impl Sidecar {
    /// Where the sidecar lives, or `None` when this machine will not say where a cache
    /// belongs — in which case the run is in-process only and nothing is written.
    pub fn default_path() -> Option<PathBuf> {
        match std::env::var_os(CACHE_DIR_VAR) {
            // Explicitly empty: the off switch, stated rather than inferred.
            Some(dir) if dir.is_empty() => None,
            Some(dir) => Some(PathBuf::from(dir).join(FILE)),
            None => cache_dir().map(|dir| dir.join("montaget").join(FILE)),
        }
    }

    /// Read the sidecar at `path`. Never fails: everything that could go wrong here is a
    /// cache miss.
    pub fn load(path: PathBuf) -> Sidecar {
        let entries = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Document>(&text).ok())
            .filter(|document| document.version == VERSION)
            .map(|document| {
                document
                    .entries
                    .into_iter()
                    .map(|(path, entry)| (PathBuf::from(path), entry))
                    .collect()
            })
            .unwrap_or_default();
        Sidecar { path, entries }
    }

    /// What the file held. The caller seeds its caches from this.
    pub fn entries(&self) -> &BTreeMap<PathBuf, Entry> {
        &self.entries
    }

    /// Write `entries` back, pruned and capped. Never fails, for the same reason
    /// [`Sidecar::load`] cannot: a cache that could not be written is a cache that will
    /// miss next time, which is the behaviour Montaget already has to be correct under.
    pub fn save(&self, mut entries: BTreeMap<PathBuf, Entry>) {
        // An entry whose file is gone answers no future question — its key can never match
        // again, because there is nothing to stat. Dropping it here is the whole of
        // invalidation-by-deletion: there is no separate sweep and no expiry clock.
        entries.retain(|path, _| path.exists());
        evict_oldest_beyond(&mut entries, MAX_ENTRIES);

        let document = Document {
            version: VERSION,
            entries: entries
                .into_iter()
                // A path this platform will not spell as UTF-8 is dropped rather than
                // written through `display()`, which would mangle it into a key that can
                // never match what a later run stats. One permanent miss, stated here,
                // beats an entry that looks like a hit and is not.
                .filter_map(|(path, entry)| {
                    Some((path.into_os_string().into_string().ok()?, entry))
                })
                .collect(),
        };
        let Ok(text) = serde_json::to_string_pretty(&document) else {
            return;
        };
        let _ = write_atomically(&self.path, &text);
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

/// Write via a temporary file and one rename, so a second Montaget running at the same
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
/// join, and ADR-0064 ships Montaget as a binary whose dependency list is something a
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
            last_used_ns,
            probe: Probe {
                source: "x".into(),
                quad: Default::default(),
                dimensions: None,
                alpha: None,
                audio: None,
            },
        }
    }

    #[test]
    fn a_corrupt_or_unknown_sidecar_reads_as_an_empty_cache() {
        let dir = std::env::temp_dir().join("montaget-sidecar-unit");
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
