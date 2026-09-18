//! The probe caches, and the miss that is not an optimisation.
//!
//! Two caches, because ADR-0056 is emphatic that the two halves have not earned the same
//! trust:
//!
//! - **Local**, keyed on `(path, size, mtime)` (ADR-0006). Every component is a direct
//!   filesystem observation Montaget makes itself.
//! - **Remote**, keyed on the URL and **scoped to this process**. ADR-0056 resolved a 1–2
//!   split for no persistent cache: *"a cache keyed on an asserted header is a declared
//!   fact wearing an observed fact's clothes."* So a distinct URL is probed once per run
//!   — a project referencing one remote clip from five elements pays one round trip, not
//!   five — and nothing survives the process.
//!
//! **Known gap: the local cache is in-process only, and ADR-0006 says it should be a
//! sidecar.** ADR-0006's consequences include *"the probe cache is a gitignored sidecar,
//! consistent with #4's rule that probe results never live in the source of truth where
//! they could go stale"*, and #190 specifies the local cache as in-process. Those
//! disagree, and the ADR wins — but the sidecar's design (its path, its format, its
//! invalidation) has no live owner: ADR-0023 defers it to #4, which is closed under a
//! different title, and calls it *"not decided here"*. Inventing one is therefore a
//! decision this ticket is not entitled to make. The cost, stated
//! rather than hidden: [`MissKind::Changed`] — the line ADR-0011 calls the **sole**
//! mechanism catching a source that grew on disk — can only fire twice within one process.
//! Over MCP that is a long-lived server and the mechanism works as specified; over the
//! CLI, each run starts cold, so every probe is a `First` miss and a growth between two
//! runs is not announced. **The sidecar is the missing half, and it is a ticket.**
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

use serde::{Deserialize, Serialize};

use super::Source;
use super::probe::{self, LocalKey, Outcome, ProcessRunner, Runner};
use super::tools::{self, Missing, Tools};

/// Why a probe ran rather than being answered from the cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MissKind {
    /// Nothing had been probed under this identity yet.
    First,
    /// The path had been probed this run, and its `(size, mtime)` has changed since.
    /// **This is the one that catches the file that got longer on disk.**
    Changed {
        previous_size: u64,
        size: u64,
        previous_mtime_ns: Option<i128>,
        mtime_ns: Option<i128>,
    },
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
}

impl Session {
    /// A session over the `ffmpeg`/`ffprobe` on this machine's `PATH`.
    ///
    /// The resolution failure is exit 70 and is returned here, at the first tool that
    /// spawns a subprocess, rather than at each of the four verbs that need it.
    pub fn open() -> Result<Session, Missing> {
        Ok(Session::with(tools::resolve()?, Box::new(ProcessRunner)))
    }

    pub fn with(tools: Tools, runner: Box<dyn Runner>) -> Session {
        Session {
            tools,
            runner,
            local: BTreeMap::new(),
            seen: BTreeMap::new(),
            remote: BTreeMap::new(),
            misses: Vec::new(),
            network_attempts: 0,
        }
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

    fn probe_local(&mut self, path: &Path) -> Result<Outcome, Box<Missing>> {
        // A file that is not there has no `(size, mtime)` to key on. It is not a cache
        // miss either — there is nothing to cache — so it goes straight to the prober,
        // which answers it from `stat` without spawning anything.
        let Ok(key) = LocalKey::of(path) else {
            return probe::probe_local(self.runner.as_ref(), &self.tools, path);
        };

        if let Some(cached) = self.local.get(&key) {
            return Ok(cached.clone());
        }

        self.misses.push(CacheMiss {
            source: path.display().to_string(),
            kind: match self.seen.get(path) {
                Some(&(previous_size, previous_mtime_ns)) => MissKind::Changed {
                    previous_size,
                    size: key.size,
                    previous_mtime_ns,
                    mtime_ns: key.mtime_ns,
                },
                None => MissKind::First,
            },
        });

        let outcome = probe::probe_local(self.runner.as_ref(), &self.tools, path)?;
        self.seen
            .insert(path.to_path_buf(), (key.size, key.mtime_ns));
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
        // The one line in Montaget after which a packet may leave the machine. It is
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
}
