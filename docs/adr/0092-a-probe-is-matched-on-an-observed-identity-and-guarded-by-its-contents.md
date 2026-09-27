---
status: accepted
amends: 0069 (adds `Probe::identity` and the value-side content guard to the sidecar entry it designed, and carves the one exception to its "every failure here is silence" rule for the `cache clear` that deleting the file *is* the request), 0011 (adds one CLI-only non-verb, `cache clear`, on that ADR's own cost model)
---

# A probe is matched on an identity it observed, guarded by its contents — and the cache has a clear command

[#385](https://github.com/MBehtemam/Montagent/issues/385) arrives from the first real
end-to-end build through this tool: an eleven-minute cut shipped with 134 of 217 audio
elements silently missing, at `0 errors`, twice, on the same project. It blames the probe
sidecar and names two mechanisms. **Neither survives contact with the source**, and saying
so is the first thing this ADR is for — a fix aimed at either one would have left the defect
running.

## The two reported mechanisms, and what was actually there

**"Rename survives the cache key" — no.** The key is `(canonical path, size, mtime_ns)`
(`probe.rs`, `LocalKey`), so a plain rename moves the path, changes the key outright, and is
a `First` miss. Renaming cannot serve a stale entry.

What *can*, and what matches "renaming 19 files during a dialogue edit" exactly, is a
**renumbering shuffle**: renaming `line-01.wav`…`line-19.wav` so each file's content lands
on a *neighbour's* existing name. Every path in the cache is still present, `mv` preserves
mtime, and so the only thing left discriminating two different takes is `size` — and two
dialogue lines can be the same length. The entry then serves another take's probe with a
straight face. The ticket had the symptom and the wrong cause; the hole is narrower than it
claimed and nastier.

**"The cache degrades under its own concurrent use" — it cannot.** ADR-0069 already
specified, and `sidecar.rs` already implements, a write through a PID-suffixed temporary and
one rename, with a re-read of the other half immediately before serialising. There is no
window in which a reader sees half a file. What ~250 concurrent `probe` subprocesses
actually cost is **last-writer-wins**: what one run learned is dropped, which is a re-probe
on some later run and never a corrupt file or a wrong answer. #385's fix item 2 was already
shipped, and this ADR closes it as such rather than re-implementing it behind a lock —
ADR-0069's reasoning that a lock file would make a cache able to block a `validate` stands
untouched.

**What was actually there** is neither. It is already re-derived from committed source in
`docs/research/first-real-use/SILENT-AUDIO-DROP-TRACE.md`, and the cache entry was innocent:

```rust
// render.rs, Mix::of, before this ADR
let path = std::fs::canonicalize(&probe.source).ok()?;   // silently drops on failure
```

`Probe::source` is a **label** — it holds whichever spelling the run that first cached the
probe happened to use. Where that spelling is relative, `canonicalize` resolves it against
**the calling process's working directory**. So a sidecar written by a CLI run started
inside the project directory was unreadable to a render started anywhere else: the entry
matched its key perfectly, held valid audio facts, and was then dropped by `.ok()?` one
layer downstream. Every audible element fell through to the `None` arm, `chains` came out
empty, and the encoder took its `-an` branch. A complete, entirely silent video at exit 0.

The cache entry matched. **The identity comparison downstream of it is what failed** — which
is precisely why the tool could not state a reason for declining the entry: it did not have
one. `"the check engine established nothing about its source"` was not an unhelpfully worded
true statement. It was false.

That also makes the working directory a third input to a render, against `CONTEXT.md`'s
*"given the same project and the same files it produces the same video every time."*

## The decision

### 1. A probe carries the identity it observed, and consumers match on that

`Probe` gains `identity: Option<PathBuf>` — the canonical path as observed at the moment of
probing, stamped in `probe::probe_local`, the one place a local probe and a real path are
both in scope. `Mix::of` compares identities and never re-resolves a string. `None` means
*"do not know"* — a remote source, or a probe no local observation stands behind — and can
never match a file on this disk.

**It is `serde(skip)`, so [ADR-0069](./0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md)'s
`VERSION` does not move.** The canonical path is *already* the sidecar's key, so writing it
again inside the value would store one fact twice and let the two copies disagree — the same
class of defect as the one being fixed. It is re-stamped from the key when a session attaches
the sidecar. The cost of the alternative was concrete and avoidable: a version bump discards
every media probe on every machine, and this field needs no migration because it is derived.

### 2. The declined case says why

#385's first ask. Reaching the `None` arm now genuinely means the check engine recorded no
probe for that file, so the message can name the file it looked for and point at the finding
that holds the reason, instead of asserting an absence of knowledge that was not the case.

Fixing the identity **had to come first**. Adding a reason to a verdict that was wrong would
have made a false line more talkative, which is worse than terse.

### 3. The content guard is a value, not a key

`Entry` gains `content: Option<String>` — sha256 over `size`, the first 64 KiB and the last
64 KiB — verified on every cache **hit**. A mismatch is a new `MissKind::Rewritten`.

This is the split [`FontEntry::sha256`] already uses, and #385 chose it here for the same
reason: **the key decides whether the bytes are worth looking at; the fingerprint decides
whether anything actually changed.** The rejected alternative was content *as* the key, which
reads bytes from every source on every run including the unchanged case ADR-0069 promises is
free.

Three sizings, each load-bearing:

- **Head and tail, not the whole file.** The guard runs on the hit, so hashing a 2 GB ProRes
  master to confirm it is unchanged would spend more than the probe it saves. The head
  carries the container header, codec configuration and stream metadata; the tail carries the
  trailing index (`moov` in a faststart MP4, cues in a WebM). Two different takes of equal
  byte length agreeing on both is the residual blind spot, and it is smaller than the one
  this closes by the size of the guard.
- **`size` is hashed too**, so a head and tail that coincide across two different lengths
  still fingerprint differently.
- **An unestablished guard never invalidates.** A `None` from either side — an entry written
  before this ADR, or a file that would not read this instant — falls back to the key. The
  alternative turns one unreadable moment into a re-probe storm.

`Rewritten` is its own kind rather than a `Changed`, because `Changed` exists to print both
sides of the key and here both sides are identical: `1024 bytes → 1024 bytes, mtime 17… →
17…` states a change while showing none.

Absent `content` reads as unguarded rather than bumping `VERSION` — one silent run per
pre-existing entry, on the same reasoning ADR-0069 applied to the font section.

### 4. `montagent cache clear`, CLI-only, and the one failure that is not silence

#385's third ask offered a `cache clear` subcommand **or** a `--no-cache` flag. The
subcommand, and not the flag:

- `MONTAGENT_CACHE_DIR=""` **is already** the off switch, stated in ADR-0069 and implemented
  in `Sidecar::default_path`. A `--no-cache` flag would be a second spelling of a mechanism
  that exists, on two verbs, and a flag lets a user bypass a bad cache but never reclaim the
  disk or reset it.
- ADR-0011's cost model settles the surface: *"A CLI subcommand costs nothing until
  invoked"*, which is why `probe`, `fmt` and `timeline` are CLI-only. An agent has no reason
  to carry a schema slot for a recovery step taken once. It is **not a verb** either — it
  reads no project and makes no claim about one — so it does not touch ADR-0011's nine.

It states the path it removed, because the path is platform-specific and the user reaching
for this command is exactly the user who does not know it. It sweeps temporaries beside the
sidecar under the same name-plus-suffix rule `write_atomically` writes them under, so a run
killed between the write and the rename does not survive a clear.

**This is the one operation on the cache that reports a failure**, which is a deliberate
carve-out from ADR-0069's *"every failure here is silence"*. That rule is about a cache
consulted **in passing**, where manufacturing a finding would be a claim `validate` cannot
substantiate. Here deleting the file **is** the entire request: a clear that could not delete
and said nothing would report success for work it did not do, and send a user on to debug
against a cache they believe is gone. It exits 74 and writes to stderr; it never produces a
`Finding`, so the rule holds everywhere it was actually about.

## Scope

This is the **cache-mechanics half** of MONTAGENT-1. The severity half — making a dropped
element an `error` and giving `validate` an `UNCHECKED` class for it — is
[#384](https://github.com/MBehtemam/Montagent/issues/384)'s decision and a separate ticket.
The two are independent and both are needed: this ADR stops the mix bus losing probes it
was handed, and #384's ticket stops an empty mix bus from reaching the output path at all.
Neither subsumes the other, and a fix to either alone would still have shipped #385's cut —
this one silently, that one loudly.

## Consequences

- **A render no longer depends on the working directory it was started from.** `CONTEXT.md`'s
  determinism claim holds against the case that broke it, and a sidecar written by any run
  is usable by any other.
- **`Probe::source` is documented as a label, at the field.** The next consumer tempted to
  re-resolve it finds the reason not to in the place they will be looking.
- **A renumbering shuffle is announced.** The one invalidation `(path, size, mtime)` cannot
  see now has a miss kind and a report line.
- **A cache hit now reads up to 128 KiB.** ADR-0069's *"spawns no `ffprobe` at all"` is
  unchanged — that is the cost being saved, and it is orders of magnitude above this.
- **No `VERSION` bump, so no machine re-probes.** Both new fields are derived or optional,
  which is what earned that.
- **#385's fix item 2 closes as already shipped.** ADR-0069's atomic write was correct; the
  ticket's concurrency theory was not.

## Evidence

Argued from committed source — the trace document, `LocalKey`, `write_atomically`, ADR-0011's
cost model — and pinned by re-executable tests rather than by the lost session the ticket
describes:

- `crates/montagent-core/tests/sidecar.rs` — a probe cached under a relative spelling is
  still matched by a run with a different working directory (the silent-audio-drop
  criterion, at the seam that can state it); a renumbering shuffle that preserves size and
  mtime is a `Rewritten` miss; an entry with no recorded `content` is still trusted on its
  key; a file that cannot be fingerprinted does not invalidate its entry.
- `crates/montagent/tests/adapters.rs` — `cache clear` removes the sidecar and its
  temporaries and states the path; clearing an absent cache succeeds.

- **`docs/adr/probe_identity_and_content_guard_check.sh`** — this ADR's two behavioural
  claims, asserted end to end against a built binary and exiting non-zero the moment either
  stops holding. It is committed because it is the one artifact that distinguishes a fix
  from a description of one, and it was checked in **both** directions before
  `status: accepted` was written:

  | run against | result |
  | --- | --- |
  | this branch | `ADR-0092 holds` — 5 assertions, exit 0 |
  | `f344da08` (the commit before it) | **3 failures**, exit 1: a render from a different working directory delivered an MP4 with **no audio stream** while reporting `0 errors` at exit 0, narrating *"not mixed vo — the check engine established nothing about its source"*; and the `mv` shuffle went unannounced |

  The base-commit run is the reproduction of #385 the ticket could not supply — the original
  session is gone — and it is reproducible rather than recounted. The script asserts its own
  setup too (that the cache really stored a relative spelling, and that the shuffle really
  moved no size or mtime), so a later change that dismantles the trap turns it into a visible
  failure rather than a vacuous pass.

The numeric claim in §3 is the guard size, and it is a sizing choice rather than a measured
threshold — there is no number here to re-derive, so nothing is committed under
`docs/adr/*_scan.py` for it. The claim that ADR-0069's atomic write already holds is
checkable by reading `write_atomically`, cited above.
