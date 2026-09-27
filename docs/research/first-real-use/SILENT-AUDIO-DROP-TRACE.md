# The silent audio drop, re-derived from source

> **Resolved by [ADR-0092](../../adr/0092-a-probe-is-matched-on-an-observed-identity-and-guarded-by-its-contents.md)**
> (#385). The mechanism below is no longer reachable: `Mix::of` matches on
> `Probe::identity` — the canonical path observed at probe time — instead of
> canonicalising `Probe::source`, so the working directory is no longer an input
> to the mix bus. The `None` arm now names the file it looked for and points at
> the finding holding the reason. **The two other routes named below are
> untouched** and remain open: an `Unchecked`/`ExistenceOnly` probe still drops
> its audio at exit 0, which is the severity half of MONTAGENT-1 and belongs to
> [#384](https://github.com/MBehtemam/Montagent/issues/384)'s decision, not to
> this one. Read the trace as the diagnosis it was; the citations are to the
> commit it landed on.

The [field report](FIELD-REPORT.md) describes a render that emitted a complete
video with every audio element missing, at exit 0, over MCP — while the CLI
rendered the same file correctly. That session is gone, so the report grounds
nothing on its own.

This trace re-derives the mechanism **from committed source**. Every claim below
is a file:line citation into this repository at the commit this document lands
on, so a later reader can check it without the lost session.

## MCP and the CLI cannot diverge by code path

Both surfaces are thin adapters over one core verb,
`montagent_core::verbs::render::render(path, &Ask, &mut progress)`:

- CLI — `crates/montagent/src/cli.rs:732-770`
- MCP — `crates/montagent/src/mcp.rs:562-601`

The only differences are argv parsing vs. `serde_json` deserialization, the
progress sink (stderr in both) and result formatting (`wire::render_video` in
both). Neither passes a session, a cache path, a surface, or any audio setting.
The MCP server holds no warm session.

Core entry is `crates/montagent-core/src/verbs/render.rs:253`; both go through
the shared span encoder `encode_span` at
`crates/montagent-core/src/verbs/render.rs:537`.

**They diverge by process state, not by code path.**

## The mechanism

`Mix::of` (`crates/montagent-core/src/verbs/render.rs:820-908`) does not resolve
audio from the document. It matches each element against `report.media` — the
probes the check engine recorded — by canonicalizing *the probe's stored source
string*:

```rust
// crates/montagent-core/src/verbs/render.rs:838-846
let probed: Vec<(PathBuf, bool)> = report.media.iter().filter_map(|probe| {
        let path = std::fs::canonicalize(&probe.source).ok()?;   // silently drops on failure
        Some((path, probe.audio.is_some()))
    }).collect();
```

`probe.source` holds whatever spelling the run that first cached it used
(`crates/montagent-core/src/media/probe.rs:299`, `:633` — `source:
display_local(path)`, where `path` is `project_dir.join(source)` and
`project_dir` comes from the project path as given). **If the project path was
given relative, `probe.source` is relative** — and `std::fs::canonicalize` on a
relative string resolves against the *calling process's* current directory.

The probe cache is a cross-process sidecar storing the whole `Probe`, original
spelling included (`crates/montagent-core/src/media/sidecar.rs:90-99`;
`crates/montagent-core/src/media/session.rs:158-171`). `Mix::of`'s own comment
already names the hazard at `render.rs:835-838`: *"the report names a probed
source as it was first cached — which may be an earlier run's spelling of the
same path."*

So: a CLI run started in the project directory caches `assets/vo.wav`. A later
MCP render, whose working directory is whatever the host launched the server
with, reuses that cached probe and fails to canonicalize the relative spelling.
Every audio element falls into the `None` arm:

```rust
// crates/montagent-core/src/verbs/render.rs:955-962
None => return Err("the check engine established nothing about its source, so it is not mixed"),
```

An `Err` there becomes a `not_mixed` entry, not a failure
(`render.rs:876-880`). When every element lands there, `chains.is_empty()` →
`Mix { audio: None, .. }` (`render.rs:885-889`) → the encoder takes the `-an`
branch:

```rust
// crates/montagent-render/src/encode.rs:187-191
if spec.audio.is_some() { command.args(["-map","[mix]","-c:a","aac","-b:a","160k"]); }
else { command.arg("-an"); }
```

A complete, entirely silent MP4. Exit 0.

## Two other routes to the same silence

- **`Unchecked` / `ExistenceOnly` probes.** `record()` only pushes into
  `report.media` for `Outcome::Probed`
  (`crates/montagent-core/src/media/probe.rs:827-840`). The other outcomes raise
  `U-SOURCE-UNPROBEABLE` / `U-SOURCE-EXISTENCE-ONLY` (`probe.rs:786-818`), which
  are UNCHECKED severity and so leave `exit_code()` at `Ok`
  (`crates/montagent-core/src/report.rs:287-295`). A present-but-unreadable file
  (`probe.rs:314-320`) therefore drops its audio at exit 0.
- **A differing cache directory.** `Sidecar::default_path()` reads
  `MONTAGENT_CACHE_DIR`, else the per-user cache dir
  (`crates/montagent-core/src/media/sidecar.rs:158-165`), and `render` always
  uses it (`render.rs:271`).

Two environment differences that *do* fail loudly, for contrast: a missing
`ffmpeg`/`ffprobe` on `PATH` becomes `report.fail_internally(...)` → exit 70
(`render.rs:346-352`), and an unwritable filter-script temp file becomes
`Stop::Internal` (`crates/montagent-render/src/encode.rs:135-143`).

## Why it is invisible

The **video** path never consults `report.media` at all — the painter probes and
decodes on its own (`crates/montagent-core/src/verbs/frame.rs:1112`). The same
broken lookup that mutes a video element still paints its pixels. The output
looks right.

## Montagent did narrate it

The reading exists and is explicit, produced in
`crates/montagent-core/src/text.rs:1309-1358`:

```
audio       none mixed — N not mixed, named below; the file carries no audio stream
not mixed   <element> — <reason>
```

backed by the machine-readable `Video::mixed` / `Video::not_mixed`
(`crates/montagent-core/src/verbs/render.rs:225-229`, filled at `:500-501`).

**Nothing treats it as an error.** `not_mixed` is a plain reporting vector: it
never becomes a `Finding`, never touches `Report::summary()`, and so never
reaches `exit_code()` or MCP `isError`. There is no registry code for "audible
elements present, none mixed" — `crates/montagent-core/src/registry.rs:757`
carries only `R-CAPTION-NO-AUDIO`. The render tests assert the happy path only
(`crates/montagent-core/tests/render.rs:624-625`, `:903`).

## What this means for the spec

The field report asked for *"it should refuse, not narrate."* The machinery to
notice already ran and already narrated. What is missing is the step that turns
that observation into an `error`-class finding — which is precisely the shape
[ADR-0006](../../adr/0006-validate-reports-facts-and-render-enforces.md)
names: the check engine stated the fact, and `render` declined to enforce it.

Three further consequences for the ADR series, each a live question rather than a
conclusion:

- `CONTEXT.md` says Montagent is deterministic — *"given the same project and the
  same files it produces the same video every time."* Here the working directory
  is a third input, and it is not in the document.
- [ADR-0069](../../adr/0069-probe-sidecar-is-a-per-user-json-cache-keyed-on-what-montagent-observed.md)
  keys the sidecar on a canonicalised path but stores the probe's original
  spelling. That spelling is what breaks.
- [ADR-0077](../../adr/0077-the-nine-render-readings-are-ratified.md) ratified the
  mix bus without stating what happens when the bus resolves to empty while
  audible elements are present.
