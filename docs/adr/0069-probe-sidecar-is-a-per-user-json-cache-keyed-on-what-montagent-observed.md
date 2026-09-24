---
status: accepted
amends: 0006 (designs the "gitignored sidecar" consequence it stated and left undesigned, and places it outside every repository rather than beside the project), 0023 (answers the parked "PAR provenance for the probe sidecar" sub-question: yes, the resolved dimensions and the `par` that produced them are stored)
---

# The probe sidecar is a per-user JSON cache, keyed on what Montagent observed, and a failure in it is always silence

> **Amended by [ADR-0089](0089-source-alpha-is-a-file-level-reading-and-vp9-needs-its-own-decoder.md).**
> The sidecar is at **version 2**. `Probe::alpha` changed shape and meaning, and a
> version-1 entry holds an answer ADR-0089 establishes is wrong for VP9-alpha sources — so
> the new field is carried by a version bump rather than by a `#[serde(default)]`, which is
> the opposite call from #206's font half and for the opposite reason. This ADR's own rule
> is what makes that cheap: an unrecognised version reads as an empty cache, so the cost is
> one re-probe per machine.

[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) settled the local probe
cache as `(path, size, mtime) → duration` and stated in its consequences that *"the probe
cache is a gitignored sidecar, consistent with
[#4](https://github.com/MBehtemam/Montagent/issues/4)'s rule that probe results never live
in the source of truth where they could go stale"*. It did not say where, in what, or
what happens when one goes wrong. [ADR-0023](./0023-video-source-dimensions-par-and-rotation.md)
then parked one sub-question against it — whether `par` and the resolved
rotation-applied dimensions belong in the sidecar *"at zero extra I/O"* — calling the
sidecar's design *"a mechanical fact (#4's sidecar design), not decided here"*. #4 is
closed under a different title, so that design had no owner;
[#231](https://github.com/MBehtemam/Montagent/issues/231) is where it acquires one.

[#190](https://github.com/MBehtemam/Montagent/issues/190) shipped the cache in process
memory only, and by then the omission had stopped being a performance matter.
[ADR-0011](./0011-tool-surface-reads-checks-renders.md) had already corrected ADR-0006 on
exactly this point:

> The document records **no source duration**, so `max(source_end)` is only a *lower
> bound*. A source that **shrank** is caught with certainty; a source that **grew** is
> caught only by the accident that this fixture consumes every source to its last
> millisecond. […] ADR-0006's `(path, size, mtime)` cache-miss line is therefore **not a
> performance optimisation with a pleasant side effect — it is the sole mechanism**
> catching that defect class, and must be specified as load-bearing.

A cache that lives in one process can only announce a change twice within that process.
`probe` is CLI-only by ADR-0011, and every CLI run starts cold, so the sole mechanism
catching a source that grew was **unreachable from the CLI**. #231 reports two independent
reviewers reproducing that on the shipped binary; no transcript was committed, so read that
as the ticket's account rather than as evidence this ADR carries. What *is* checkable is
the property itself, and it is checkable by construction: a cache held only in process
memory has no entry on the first probe of a new process, so the first miss for any path is
a `First` and `Changed` cannot be reached. Over MCP, where a long-lived
server holds the cache, ADR-0011's *"warm probe cache"* worked as written. That asymmetry
is what this ADR removes.

## Where it lives: under the per-user cache directory, never beside the project

One JSON file per user, at `<cache dir>/montagent/probe-cache.json` — `~/Library/Caches`
on macOS, `$XDG_CACHE_HOME` or `~/.cache` elsewhere, `%LOCALAPPDATA%` on Windows —
overridable with `MONTAGENT_CACHE_DIR`, whose empty value turns persistence off entirely.

The alternative ADR-0006's wording suggests, a file beside the project, was rejected on
its own stated requirement. ADR-0006 wants a cache that cannot be committed; a file beside
the project is one `git add .` away from being committed, and the only defence is an
ignore rule in **every repository that ever holds a Montagent project** — a rule this
project cannot state once because it would have to be stated in repositories that do not
exist yet. Writing the rule on the user's behalf is worse: a `validate` that edits
someone's `.gitignore` has done something to their repository that they did not ask for,
and ADR-0006's whole posture is that `validate` reports rather than acts.

Put under the per-user cache directory, **the requirement is met structurally rather than
by a convention anyone has to remember**: the file is not inside any repository, so there
is no rule to forget. [ADR-0053](./0053-asset-path-resolution-no-assetroot.md)'s *"a
project is a movable unit — the `.montagent.json` file plus its relative assets"* is also
left exactly as it was, gaining nothing that travels with it and nothing that goes stale
when it moves. This repository still states one ignore rule (`probe-cache.json*`), for the
single way a cache can appear inside a checkout at all: `MONTAGENT_CACHE_DIR` pointed into
one.

The cost, stated: two users on one machine do not share a cache, and a project moved to a
new machine arrives cold. Both are re-probes, and a re-probe is the behaviour every code
path here is already correct under.

## What it is: JSON, human-readable, versioned, and capped

JSON, pretty-printed, keyed by canonical path, with a `version` integer. Everything else
Montagent writes is JSON a person can read, and a cache a person cannot read is a cache a
person cannot disbelieve — which matters precisely because this file's entries are claims
about media the reader may be arguing with.
[ADR-0017](./0017-closed-schema-no-escape-hatch.md)'s closed schema does not reach here:
that governs the *project* format, and its sidecar carve-out is about third-party
annotation, not about the tool's own cache.

There is no migration path. A `version` this binary does not recognise reads as an empty
cache — one re-probe, and it cannot be wrong. The file is capped at 4096 entries, evicting
least-recently-used, because a cache with no ceiling grows for the life of the machine.
Writes go through a temporary file and one rename, so two Montagents running at once read
either the old file or the new one and never half of either; between two concurrent runs
the last writer wins, and what the other learned costs a later re-probe rather than a lock
file able to block a `validate`. The temporary is named by **appending** to the sidecar's
own name rather than replacing its extension, so the single ignore rule this repository
states (`probe-cache.json*`) covers a run killed between the write and the rename too.

## What it stores: the whole `Probe` — which answers ADR-0023's parked question, yes

Each entry is the observed `(size, mtime)` and the complete `Probe`
(`crates/montagent-core/src/media/probe.rs`): ADR-0011's quad, ADR-0023's
resolved rotation-applied dimensions **and the `par` that produced them**, alpha, and the
audio facts.

ADR-0023 asked whether the dimensions and `par` belonged here *"at zero extra I/O"*. They
do, and the reason is that the question's premise is already satisfied: the probe that
fills an entry has computed them before the entry exists, so the choice is not between
storing them and not paying for them — it is between storing them and **throwing them
away**, to re-derive them on the next run from a file whose identity has not changed. The
`par` is the file's own probed value, which is what ADR-0023's `note` compares a declared
`par` against; storing it keeps that check answerable from a cache hit rather than
silently demoting it to a re-probe.

**Two things are never stored.** An outcome that is not a `Probe` — a missing, unreadable
or existence-only source — establishes no content facts, and persisting the *absence* of a
fact would let a transient condition outlive the process that saw it. And **nothing
remote**: [ADR-0056](./0056-remote-source-probe-session-scoped-no-persistent-cache.md)
resolved a 1–2 split for no persistent cache on a remote source, reasoning that *"a cache
keyed on an asserted header is a declared fact wearing an observed fact's clothes"*. That
decision is untouched. The session's remote half never reaches the sidecar module at all,
and a test pins that the two halves stay separate.

## The key is canonical, and it must be whole

`(path, size, mtime)` is unchanged in substance, and the path is **canonicalised**.
Within one process the spelling a caller used was as good as any other; across processes
it is not, because `./take3.mov` and `/clips/take3.mov` are one file, and a cache keyed on
the spelling would remember them as two and notice a change in neither. What the *report*
names is untouched: that is still the source as the document spells it. A path this
platform will not spell as UTF-8 is not written at all, rather than written through a
lossy rendering that could never match what a later run stats.

**An entry with no mtime is never persisted, and never loaded.** `mtime` is an `Option`,
because a filesystem may decline to state one, and `None` equals `None` — so such a key
matches on `(path, size)` alone and cannot notice a file rewritten to the same length.
Within one process that is #190's accepted blind spot, bounded by the process. Persisted,
it would be permanent, and what it would silence is precisely `MissKind::Changed`, which
is the mechanism this whole ADR exists to make reachable. A key Montagent only partly
observed does not earn a place in a cache that outlives the observing.

## Invalidation, and the rule that every failure here is silence

- An entry whose path no longer exists is **dropped when the sidecar is written**. That is
  the whole of invalidation-by-deletion: there is no sweep and no expiry clock, because an
  entry with nothing to `stat` can never match again.
- An entry whose `(size, mtime)` no longer matches is a **`Changed` miss**, which is the
  mechanism this ADR exists to make reachable. The miss line names both halves of the key
  on both sides of the change — `previous_size → size`, `previous_mtime → mtime` — because
  a file rewritten to the same length is a change only the mtime shows.
- A sidecar that is **missing, corrupt, of an unknown version, unreadable or unwritable is
  a cache miss and nothing else.** Never a finding, never an error, never a line in the report. A
  cache directory is not the project, and `validate` answers exactly one question — *"is
  this project file internally legal, and does it agree with the media on disk?"*
  (ADR-0006). A finding manufactured out of a cache failure would be a claim `validate`
  cannot substantiate, which is the failure ADR-0006 was written against, on the opposite
  side.

## Not settled here

- **A warm in-process session for the MCP server.** ADR-0011's *"warm probe cache"* is
  satisfied by the sidecar today — each MCP call reads and writes it — and whether the
  server should additionally hold one session across calls is a performance question with
  no measurement behind it yet.
- **Sharing a cache between users or machines.** Deliberately out: a shared cache is a
  cache whose entries were observed by somebody else's filesystem, which is the trust
  ADR-0056 declined to extend to a server's headers.

## Consequences

- **`MissKind::Changed` is reachable from the CLI.** A source that grows between two runs
  is announced by the second, which is [#231](https://github.com/MBehtemam/Montagent/issues/231)'s
  first acceptance criterion and ADR-0011's sole mechanism for that defect class.
- **A `validate` or `probe` run over unchanged media spawns no `ffprobe` at all.** ADR-0006's
  *"no fast mode"* is unaffected — every source is still asked about on every run; what
  changed is that the answer may come from an observation this machine already made.
- **The cache-miss line's wording changes**: *"not yet in this session's cache"* becomes
  *"not yet in the probe cache"*, because the cache is no longer the session's.
- **`montagent_core::verbs::validate::validate_with`** exists, so a caller that owns a
  session — an MCP server, or a test that must not persist anything — can hand one in, the
  way `probe_with` already allowed.
- **`Session::with` is sidecar-free and `Session::open` is not.** The default carries
  persistence; a session built from parts does not, which is what keeps a test's cache
  inside the test.

## Evidence

The behaviour is argued from committed spec text — ADR-0006's consequence, ADR-0011's
correction of it, ADR-0023's parked question, ADR-0056's remote decision — and pinned by
re-executable tests rather than by a transcript:

- `crates/montagent-core/tests/sidecar.rs` — the acceptance criteria at the `Session` and
  file seams: a source that grew between two runs, an unchanged one that costs nothing,
  the dimensions and `par` round trip, a vanished path that is dropped, an outcome without
  content facts that is not remembered, a half-observed key that is neither written nor
  trusted, and the pin that nothing remote is persisted. The never-a-finding half of the
  failure rule is asserted against a whole `Report` rather than a session — four ways for a
  sidecar to be unreadable (corrupt, an unknown version, absent, permission-denied) and one
  to be unwritable, each producing zero findings, one `First` miss, and a probe that
  answered anyway.
- `crates/montagent/tests/adapters.rs` — `cli_a_source_that_grew_between_two_runs_is_announced_by_the_second`
  states the criterion at the only seam that can: two processes. Its third run asserts the
  unchanged case reports no miss at all.
- `crates/montagent/tests/adapters.rs` — `cli_the_probe_cache_never_lands_beside_the_project`
  asserts the run leaves nothing beside the media it probed.

No jury was convened. ADR-0023 called this design *"a mechanical fact"*, and the two
questions that were genuinely open — where it lives, and whether the dimensions belong in
it — are each settled by a requirement already written down (ADR-0006's uncommittable
cache; ADR-0023's own "zero extra I/O" test) rather than by a preference a panel would
have had to weigh.
