---
status: accepted
amends: 0053 (discharges the four remote-specific questions ADR-0053 deferred: probe cadence, what a remote probe fetches, cache key, and what a probe failure means)
---

# Remote sources are probed fresh every session, deduplicated by URL, with no persistent cache; a network failure is `UNCHECKED`, never a confirmed defect, and `render` verifies independently rather than trusting `validate`

> **Amended by [ADR-0093](./0093-renders-world-effects-are-findings-and-an-error-withholds-the-deliverable.md)**,
> which amends this ADR's `UncheckedReason` enumeration with its **first non-network member**,
> `Unidentified`: a local source `ffprobe` answered for whose canonical path the run could not
> observe, so ADR-0092 leaves every consumer with no admissible answer to *"is this the same
> file?"* and `render` must decline it. It was the last route by which a source could be a
> clean pass to `validate` and a silent drop in `render`. This ADR's deliberately absent
> `missing` variant is not violated — the file exists; what is unknown is which file it is.

[ADR-0002](./0002-inline-source-no-asset-table.md) permits `source` to be a URL.
[ADR-0006](./0006-validate-reports-facts-and-render-enforces.md) settled that
`validate` probes every local media file on every run — no fast mode, no
`--no-probe` flag, no scoping — because the measured cost was ~0.04s per file,
cached on `(path, size, mtime) → duration`. [ADR-0053](./0053-asset-path-resolution-no-assetroot.md)
settled that a *confirmed*-missing `source`, local or remote, is a plain
`error`, and explicitly deferred everything remote-specific — network cost,
caching, and what a probe *failure* (as opposed to a probe *finding a defect*)
means — to this ADR.

A remote probe has none of the properties the local case relied on: no mtime,
no cheap size, and a round-trip that can be slow, flaky, or simply
unreachable rather than ~0.04s and deterministic. The one committed fixture
has zero remote sources, so nothing here is measured; it is reasoned from the
project's standing invariants — inert data, `validate` reports facts and
`render` enforces, and no silent false confidence.

## Decision

### Probe every distinct remote URL once per `validate` invocation, not once ever

The **intent** of "always probe, no fast mode" transfers to remote sources
unmodified: there is no flag that skips a remote probe, and every `validate`
run contacts every remote host it needs to say anything about. What does
*not* transfer is the **measurement** that justified running it unconditionally
at zero perceived cost. The resolution is deduplication, not a fast mode: a
project referencing one remote clip from five elements probes that URL once
per run, not five times — the same shape as ADR-0006's local cache, just
scoped to the run rather than persisted across runs. A URL that changes
mid-run is not noticed; this is the same accepted blind spot as local
`mtime` caching mid-run, and it buys a run whose duration is bounded by
distinct URLs rather than by element count.

Decided **unanimous 3/3** (Opus, Haiku, Fable).

### A probe attempts real content verification (duration/dimensions), and degrades to existence-only only when it must, saying so in the finding

`validate` traffics in facts, and the fact the document depends on is
duration and dimensions (an element's `source_end` implies a real length),
not "some server answered 200 for this URL." The probe attempts a partial
content fetch — an HTTP range request feeding the same duration/dimension
extraction the local probe already does — before falling back to an
existence-only check (HTTP HEAD / status code) for a URL or protocol that
won't support partial reads. **The degradation is never silent**: a finding
that only confirmed existence says exactly that — *"existence confirmed
(HEAD 200), duration NOT CHECKED"* — never occupying the same report slot as
a confirmed duration the way a real probe result does. An existence-only
result is a strictly weaker claim than a local probe's, and reporting it
identically would be exactly the false confidence ADR-0006 was written to
prevent.

Decided **unanimous 3/3**.

### No persistent cache for a remote source. Session-scoped only

The tempting cache key — ETag or `Last-Modified` — is a *server assertion*,
not an observation the way local `(size, mtime)` is. A server may omit
either header, rotate an ETag on unchanged bytes, or hold one fixed across
changed bytes; CDNs do all three routinely. Persisting a value keyed on a
header nobody controls plants network-derived, unverifiable state into the
tool's durable footprint — the same hidden-state failure the inert-data
principle forbids at the document level, now recurring one layer down in the
tool that reads it. **No persistent cache keeps every claim `validate` makes
about a remote source honest on its own terms**: it was learned this run, or
it is marked as not learned at all. Between runs there is no cache to go
stale, so there is nothing to silently trust.

This is the one question the panel split on. Two of three jurors (Haiku,
Fable) voted for an ETag → `Last-Modified` fallback chain, reasoning it
mirrors the local `(path, size, mtime)` cache closely enough to earn the same
trust and saves re-probing a large, stable, unchanging asset on every run.
The map author resolves the split for **no persistent cache** (the
one-juror, Opus, position): the local cache is trustworthy because its key
(`size`, `mtime`) is a direct filesystem observation the tool itself makes;
an HTTP cache validator is trustworthy only to the degree the remote server
is trustworthy, and this project's entire probe apparatus exists precisely
because ADR-0005 and ADR-0006 do not extend that kind of trust to declared
or asserted facts — they re-derive them. A cache keyed on an asserted header
is a declared fact wearing an observed fact's clothes. If remote re-probing
cost is ever measured as a real burden (which it cannot be against a fixture
with zero remote sources), the honest fix is a separately-evidenced,
separately-decided mechanism — not a header-keyed cache adopted now on
inference alone.

### A network failure that learns nothing is `UNCHECKED`, never a new severity tier, and never laundered into `error`

Request timeout, DNS failure, host unreachable: none of these is evidence
the file is missing — each is evidence that *nothing was learned*. That is
exactly the shape ADR-0006's `UNCHECKED`/`NOT CHECKED` category already
exists to hold, and it holds without strain: reusing it keeps the report's
whole value — one glance separates confirmed-good from not-confirmed —
intact, rather than fragmenting it into unknowing-by-timeout versus
unknowing-by-not-looking, a distinction nobody can act on differently.
ADR-0053's boundary already does the discriminating: a *confirmed* absence
(404, a resolved host that explicitly refuses the object) is the plain
`error` that ADR fixed; a probe that simply couldn't complete stays
`UNCHECKED` and must never be reported as if it had confirmed absence.

`UNCHECKED` gains a mandatory structured reason (`timeout` / `dns` /
`unreachable` / an HTTP status code) rather than a free-text note, so a
network-flavoured unknown is distinguishable from an unattempted one without
promoting either into its own severity.

Decided **unanimous 3/3** on reusing `UNCHECKED`; the reason sub-field is
adopted from one juror's (Fable's) ballot and does not conflict with either
Q3 position.

### `render` never gates on `validate`'s `UNCHECKED` finding — it fetches independently and fails on its own attempt

`render` enforces on facts; `UNCHECKED` is the explicit absence of a fact,
and enforcing on an absence is enforcing on ignorance. It is worse than
inert here: gating on a `validate`-time network sample makes `render`'s
outcome depend on a condition observed at an arbitrary earlier moment — a
thirty-second blip during `validate` would block a render attempted an hour
later over a healthy connection, and the reverse (clean at `validate`, dead
at `render`) would pass a gate that then fails anyway. `render` needs the
actual bytes regardless of what `validate` reported, so it is the only place
with an authoritative, correctly-timed observation. It resolves every remote
source up front, before encoding begins, and fails loudly and specifically —
naming the URL and the cause — if that resolution fails. `validate`'s
`UNCHECKED` finding still does its job: it warns a careful author before
they start a render that is likely to fail, without being trusted as a gate
that decides whether the attempt happens.

Decided **unanimous 3/3**.

## Consequences

- A `validate` run's latency is no longer bounded by local-file count alone;
  it grows with the number of *distinct* remote URLs a project references,
  each probed once per run.
- A remote probe result is one of three report states: a real duration/
  dimension fact, an existence-only fact with duration explicitly `NOT
  CHECKED`, or `UNCHECKED` with a reason (`timeout`/`dns`/`unreachable`/HTTP
  status) — never silently promoted to a stronger or weaker claim than what
  was actually established.
- No cache entry for a remote source survives past the `validate` process
  that created it. A project with many stable remote assets pays a fresh
  probe every invocation; that cost is accepted rather than trusted away.
- `render` performs its own remote-source resolution before encoding starts,
  independent of anything `validate` reported, and its failure messages are
  the authoritative account of a remote-source problem at render time.
- ADR-0053's "confirmed-missing is a plain `error`" is unchanged and now has
  its remote-specific complement: *unconfirmed* is `UNCHECKED`, and the two
  must never be conflated in either direction.

## Evidence

One round, three jurors (Claude Opus 5, Claude Haiku 4.5, Claude Fable 5.1),
independent, blind to each other's ballots, given the same five-part question
and no assigned stance. Full ballots and the verbatim question text at
[`docs/research/juries/remote-source-probe/BALLOTS.md`](../research/juries/remote-source-probe/BALLOTS.md).
Unanimous 3/3 on probe cadence (session-scoped, deduplicated by URL), what a
probe fetches (attempt real content verification, degrade to existence-only
explicitly), reuse of `UNCHECKED` for network failure, and `render` verifying
independently rather than trusting `validate`. Split 1-2 on whether the
remote cache persists across runs (ETag/Last-Modified chain) or not at all;
resolved by the map author for no persistent cache, reasoning recorded above
and in the ballots file's Verdict section.
