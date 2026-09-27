//! The probe caches, and the miss that is not an optimisation.
//!
//! Two caches, because ADR-0056 is emphatic that the two halves have not earned the same
//! trust:
//!
//! - **Local**, keyed on `(path, size, mtime)` (ADR-0006). Every component is a direct
//!   filesystem observation Montagent makes itself.
//! - **Remote**, keyed on the URL and **scoped to this process**. ADR-0056 resolved a 1–2
//!   split for no persistent cache: *"a cache keyed on an asserted header is a declared
//!   fact wearing an observed fact's clothes."* So a distinct URL is probed once per run
//!   — a project referencing one remote clip from five elements pays one round trip, not
//!   five — and nothing survives the process.
//!
//! The local cache is **also a sidecar on disk** ([`super::sidecar`], ADR-0069), because
//! in-process alone was not enough. The cost of that gap, while it stood, was the whole of
//! [`MissKind::Changed`]: the line ADR-0011 calls the **sole** mechanism catching a source
//! that grew on disk could only fire twice within one process, and `probe` is CLI-only, so
//! over the CLI every run started cold and the mechanism was unreachable. The sidecar is
//! read when the session opens and written when it ends, so a `Changed` miss survives a
//! process boundary. Nothing about a **remote** source goes into it: ADR-0056's
//! no-persistent-cache decision is untouched, and the remote half below never reaches
//! [`super::sidecar`] at all.
//!
//! **The miss is load-bearing, not incidental.** ADR-0011 corrects ADR-0006 on exactly
//! this point: the document records no source duration, so `max(source_end)` is only a
//! lower bound and a source that *grew* is invisible to every check constructible from
//! the document plus the disk. *"ADR-0006's `(path, size, mtime)` cache-miss line is
//! therefore **not a performance optimisation with a pleasant side effect — it is the sole
//! mechanism** catching that defect class, and must be specified as load-bearing."* So
//! every miss is recorded in [`Session::misses`] and reported unprompted; there is no
//! quiet path.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use montagent_render::decode::Decoder;
use serde::{Deserialize, Serialize};

use super::Source;
use super::probe::{self, LocalKey, Outcome, Probe, ProcessRunner, Runner};
use super::sidecar::{self, Entry, Sidecar};
use super::tools::{self, Missing, Tools};

/// Why a probe ran rather than being answered from the cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MissKind {
    /// Nothing had been probed under this identity yet.
    First,
    /// The path had been probed before — this run, or any earlier one the sidecar
    /// remembers — and its `(size, mtime)` has changed since.
    /// **This is the one that catches the file that got longer on disk.**
    Changed {
        previous_size: u64,
        size: u64,
        previous_mtime_ns: Option<i128>,
        mtime_ns: Option<i128>,
    },
    /// The key matched an entry — same canonical path, same size, same mtime — and the
    /// **bytes did not**. #385: a renumbering shuffle that moves one take's content onto
    /// another take's name changes nothing `(path, size, mtime)` can see, because `mv`
    /// preserves mtime and two dialogue lines can be the same length. The content guard
    /// caught it.
    ///
    /// Its own kind rather than a [`MissKind::Changed`], because `Changed` exists to print
    /// both sides of the key and here both sides are identical: a line reading
    /// `1024 bytes → 1024 bytes, mtime 17… → 17…` states a change while showing none, which
    /// is a worse report than no line at all.
    Rewritten { size: u64, mtime_ns: Option<i128> },
    /// A remote URL, which has no persistent cache to miss (ADR-0056). Recorded so the
    /// round trip is visible rather than implied.
    Remote,
}

/// One probe that actually ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheMiss {
    pub source: String,
    /// Flattened, so the kind and its numbers read as one object: `{"source": …,
    /// "kind": "changed", "previous_size": …}`.
    #[serde(flatten)]
    pub kind: MissKind,
}

/// A run's probe state: the two caches, the misses, and the ledger of network attempts.
///
/// One `Session` per `validate` invocation — or, over MCP, one per long-lived server,
/// which is ADR-0011's *"a warm probe cache: a long-lived server holds `(path, size,
/// mtime) →` duration in memory across calls"*. The remote half is deliberately **not**
/// warmed that way; [`Session::begin_run`] clears it, because *"probe every distinct
/// remote URL once per `validate` invocation, not once ever"*.
pub struct Session {
    tools: Tools,
    runner: Box<dyn Runner>,
    local: BTreeMap<LocalKey, Outcome>,
    /// What each path was last probed as, so a changed file is a *reported* miss rather
    /// than merely a different key.
    seen: BTreeMap<PathBuf, (u64, Option<i128>)>,
    remote: BTreeMap<String, Outcome>,
    misses: Vec<CacheMiss>,
    network_attempts: usize,
    /// The content fingerprint recorded for each cached path — #385's guard against a key
    /// that matches bytes it was not read from. Seeded from the sidecar and updated for
    /// every path this session probed; a path absent here is unguarded and is trusted on
    /// its key alone.
    content: BTreeMap<PathBuf, String>,
    /// Where the local half persists, or `None` for an in-process-only session.
    sidecar: Option<Sidecar>,
    /// When each cached path was last used, which is what the sidecar evicts by. Seeded
    /// from the sidecar and stamped with *now* for every path this session actually
    /// probed or answered from cache — so a run that merely rewrote the file does not
    /// refresh entries it never looked at.
    recency: BTreeMap<PathBuf, i128>,
}

impl Session {
    /// A session over the `ffmpeg`/`ffprobe` on this machine's `PATH`.
    ///
    /// The resolution failure is exit 70 and is returned here, at the first tool that
    /// spawns a subprocess, rather than at each of the four verbs that need it.
    ///
    /// This is the production path, and it is the one that carries the sidecar: where
    /// the machine says a per-user cache belongs, the local cache is read from there and
    /// written back when the session ends.
    pub fn open() -> Result<Session, Missing> {
        Session::open_at(Sidecar::default_path())
    }

    /// The same, over a named sidecar file — or none, for a run that must persist nothing.
    ///
    /// Taken as an argument rather than resolved here so that one `validate` has **one**
    /// cache: its font half (#206) and its probe half write the same file, and a caller
    /// that owns one owns both.
    pub fn open_at(sidecar: Option<PathBuf>) -> Result<Session, Missing> {
        let mut session = Session::with(tools::resolve()?, Box::new(ProcessRunner));
        if let Some(path) = sidecar {
            session.attach(Sidecar::load(path));
        }
        Ok(session)
    }

    /// A session with **no** sidecar: the local cache lives and dies with the value.
    pub fn with(tools: Tools, runner: Box<dyn Runner>) -> Session {
        Session {
            tools,
            runner,
            local: BTreeMap::new(),
            seen: BTreeMap::new(),
            remote: BTreeMap::new(),
            misses: Vec::new(),
            network_attempts: 0,
            content: BTreeMap::new(),
            sidecar: None,
            recency: BTreeMap::new(),
        }
    }

    /// The same, over a named sidecar file.
    pub fn with_sidecar(tools: Tools, runner: Box<dyn Runner>, sidecar: PathBuf) -> Session {
        let mut session = Session::with(tools, runner);
        session.attach(Sidecar::load(sidecar));
        session
    }

    /// Seed the local cache from a sidecar, and keep it to write back to.
    ///
    /// Both maps are seeded, and the second is the point: `local` alone would make an
    /// unchanged file free, which is the optimisation. `seen` is what makes a *changed*
    /// one announce itself with the numbers it had before — the mechanism ADR-0011 calls
    /// the only one there is for a source that grew.
    ///
    /// An entry with no mtime is skipped rather than seeded. `None` equals `None`, so it
    /// would match on `(path, size)` alone and answer for a file rewritten to the same
    /// length — a stale fact served with a straight face, and `MissKind::Changed` silent.
    /// Within one process that blind spot is #190's; across processes it is refused.
    fn attach(&mut self, sidecar: Sidecar) {
        for (path, entry) in sidecar
            .entries()
            .iter()
            .filter(|(_, e)| e.mtime_ns.is_some())
        {
            let key = LocalKey {
                path: path.clone(),
                size: entry.size,
                mtime_ns: entry.mtime_ns,
            };
            // Re-stamped from the key, which is where ADR-0069 already keeps the canonical
            // path. `Probe::identity` is `serde(skip)`, so a probe read back from the
            // sidecar arrives with none — and a consumer that had to fall back to
            // re-resolving `Probe::source` is #385's silent audio drop.
            let mut probe = entry.probe.clone();
            probe.identity = Some(path.clone());
            self.local.insert(key, Outcome::Probed(probe));
            self.seen.insert(path.clone(), (entry.size, entry.mtime_ns));
            self.recency.insert(path.clone(), entry.last_used_ns);
            if let Some(content) = &entry.content {
                self.content.insert(path.clone(), content.clone());
            }
        }
        self.sidecar = Some(sidecar);
    }

    /// The sidecar file this session persists to, where it has one.
    ///
    /// Read by `validate` so that a caller who owns a session owns the *whole* run's cache:
    /// the font half (#206) writes the same file the probe half does, which is the invariant
    /// [`Session::open_at`] exists for.
    pub fn cache_path(&self) -> Option<&Path> {
        self.sidecar.as_ref().map(|sidecar| sidecar.path())
    }

    /// Start one verb invocation.
    ///
    /// The local cache survives — its key is an observation, and a file that has not
    /// changed has not changed. The remote cache does not: ADR-0056 scopes remote
    /// deduplication to the run, *"the same shape as ADR-0006's local cache, just scoped
    /// to the run rather than persisted across runs"*.
    pub fn begin_run(&mut self) {
        self.remote.clear();
        self.misses.clear();
    }

    /// Probe one source, from the cache where the cache is entitled to answer.
    pub fn probe(&mut self, source: &Source) -> Result<Outcome, Box<Missing>> {
        match source {
            Source::Local(path) => self.probe_local(path),
            Source::Remote(url) => self.probe_remote(url),
        }
    }

    /// The decoder a source needs, off the same probe every other reading comes from.
    ///
    /// ADR-0089 puts the decode decision behind the probe, and this is where the two meet:
    /// the answer is [`Probe::decoder`]'s, and going through the session means the
    /// `ffprobe` it rests on is the one this run already paid for rather than a second one
    /// at the spawn.
    ///
    /// A source that could not be probed decodes with [`Decoder::Auto`] — the behaviour
    /// every source had before ADR-0089. An unprobeable source is `validate`'s finding to
    /// report, and refusing to decode here would be this reading inventing a second one.
    pub fn decoder_for(&mut self, source: &Source) -> Result<Decoder, Box<Missing>> {
        Ok(self
            .probe(source)?
            .probe()
            .map(Probe::decoder)
            .unwrap_or_default())
    }

    fn probe_local(&mut self, path: &Path) -> Result<Outcome, Box<Missing>> {
        // A file that is not there has no `(size, mtime)` to key on. It is not a cache
        // miss either — there is nothing to cache — so it goes straight to the prober,
        // which answers it from `stat` without spawning anything.
        let Ok(key) = LocalKey::of(path) else {
            return probe::probe_local(self.runner.as_ref(), &self.tools, path);
        };

        // The key's path is canonical, so two spellings of one file are one entry and a
        // sidecar written by a run started in one directory is readable by a run started in
        // another. The *report* still names the source the way the caller spelled it.
        self.recency.insert(key.path.clone(), sidecar::now_ns());

        // #385's content guard, and the only place it runs: a key that matches still has to
        // agree about the bytes. `None` from either side means the guard established
        // nothing — an unguarded entry written before #385, or a file that would not read
        // this instant — and an unestablished guard never invalidates anything.
        let rewritten = self.local.contains_key(&key)
            && match (self.content.get(&key.path), sidecar::fingerprint(&key.path)) {
                (Some(recorded), Some(actual)) => recorded != &actual,
                _ => false,
            };

        if let Some(cached) = self.local.get(&key).filter(|_| !rewritten) {
            return Ok(cached.clone());
        }

        self.misses.push(CacheMiss {
            source: super::display_local(path),
            kind: match (rewritten, self.seen.get(&key.path)) {
                (true, _) => MissKind::Rewritten {
                    size: key.size,
                    mtime_ns: key.mtime_ns,
                },
                (false, Some(&(previous_size, previous_mtime_ns))) => MissKind::Changed {
                    previous_size,
                    size: key.size,
                    previous_mtime_ns,
                    mtime_ns: key.mtime_ns,
                },
                (false, None) => MissKind::First,
            },
        });

        let outcome = probe::probe_local(self.runner.as_ref(), &self.tools, path)?;
        self.seen.insert(key.path.clone(), (key.size, key.mtime_ns));
        // Recorded after the probe, so what is persisted is a fingerprint of the bytes this
        // probe actually read rather than of whatever was there before it ran.
        match sidecar::fingerprint(&key.path) {
            Some(content) => {
                self.content.insert(key.path.clone(), content);
            }
            // A file that would not read leaves no guard rather than a stale one: the entry
            // falls back to its key, which is exactly where it was before #385.
            None => {
                self.content.remove(&key.path);
            }
        }
        self.local.insert(key, outcome.clone());
        Ok(outcome)
    }

    fn probe_remote(&mut self, url: &str) -> Result<Outcome, Box<Missing>> {
        if let Some(cached) = self.remote.get(url) {
            return Ok(cached.clone());
        }

        self.misses.push(CacheMiss {
            source: url.to_string(),
            kind: MissKind::Remote,
        });
        // The one line in Montagent after which a packet may leave the machine. It is
        // counted, so "no unsolicited network call" is a number a test can read rather
        // than a claim a reviewer has to take on trust.
        self.network_attempts += 1;

        let outcome = probe::probe_remote(self.runner.as_ref(), &self.tools, url)?;
        self.remote.insert(url.to_string(), outcome.clone());
        Ok(outcome)
    }

    /// Every probe that actually ran this run, in the order they ran.
    pub fn misses(&self) -> &[CacheMiss] {
        &self.misses
    }

    /// How many times this session has attempted to reach the network.
    pub fn network_attempts(&self) -> usize {
        self.network_attempts
    }

    /// Write the local cache back to the sidecar. A no-op for a session that has none.
    ///
    /// Called by [`Drop`], which is the only *"on finish"* that also covers the paths that
    /// leave early — a `validate` that discovers there is no `ffprobe` after probing four
    /// files has still learned four things, and dropping them on the floor would cost the
    /// next run the `Changed` line for all four.
    pub fn save(&self) {
        let Some(sidecar) = &self.sidecar else {
            return;
        };
        sidecar.save(self.entries());
    }

    /// The local cache as sidecar entries.
    ///
    /// Built from `seen` rather than from `local`, because `local` may hold two keys for
    /// one path — the identity it had before it changed, and the one it has now — and only
    /// the current one is worth remembering. **Only [`Outcome::Probed`] survives**: a
    /// source that was missing, unreadable or existence-only established no content facts,
    /// and persisting the absence of a fact would let a transient condition outlive the
    /// process that saw it.
    ///
    /// **And only a fully observed key survives.** A file whose filesystem would not state
    /// an mtime is cached under `(path, size)` alone, which cannot notice a rewrite to the
    /// same length; in one process that is #190's accepted blind spot, and persisting it
    /// would make it permanent and silence [`MissKind::Changed`] for that file forever.
    fn entries(&self) -> BTreeMap<PathBuf, Entry> {
        let now = sidecar::now_ns();
        self.seen
            .iter()
            .filter(|&(_, &(_, mtime_ns))| mtime_ns.is_some())
            .filter_map(|(path, &(size, mtime_ns))| {
                let key = LocalKey {
                    path: path.clone(),
                    size,
                    mtime_ns,
                };
                let Some(Outcome::Probed(probe)) = self.local.get(&key) else {
                    return None;
                };
                Some((
                    path.clone(),
                    Entry {
                        size,
                        mtime_ns,
                        content: self.content.get(path).cloned(),
                        last_used_ns: self.recency.get(path).copied().unwrap_or(now),
                        probe: probe.clone(),
                    },
                ))
            })
            .collect()
    }
}

/// The sidecar is written when the session ends, whichever way it ends.
impl Drop for Session {
    fn drop(&mut self) {
        self.save();
    }
}
