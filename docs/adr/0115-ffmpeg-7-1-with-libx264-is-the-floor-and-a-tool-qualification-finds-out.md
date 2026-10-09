---
status: accepted
amends: 0009 (the `ffmpeg` the user supplies has a floor, ffmpeg 7.1 or newer built with libx264, stated as three arguments rather than a version string), 0064 (CI's Linux legs test the floor on a pinned, checksum-verified static build; macOS and Windows test the newest release), 0091 (`Missing`'s one dispatch point chooses among three codes, adding `E-TOOL-UNSUPPORTED` for a found `ffmpeg` that fails the tool qualification), 0113 (a failed spawn is never an empty answer, as an invariant over every spawn rather than a fix at two)
---

# ffmpeg 7.1 with libx264 is the floor, and a tool qualification finds out

> **Amended by [ADR-0187](0187-windows-ci-pins-a-checksummed-ffmpeg-and-macos-alone-floats.md).**
> §7's "Windows (`choco`) keep[s] installing the newest release" no longer holds. Both Windows
> legs install a pinned, checksum-verified BtbN build of the newest release series. macOS
> (`brew`) alone still floats and catches the next ffmpeg that removes an option.

[#479](https://github.com/MBehtemam/Montagent/issues/479), which ships what
[#477](https://github.com/MBehtemam/Montagent/issues/477) ruled after
[#471](https://github.com/MBehtemam/Montagent/issues/471) found ffmpeg 9 breaking two of
Montagent's arguments. #477 was settled in a grilling session with two three-juror courts; its
resolution comment holds the rulings and the splits. This ADR records what shipped.

## What was there

ADR-0009 ships Montagent as *"a binary, plus an `ffmpeg` the user supplies"*, and said nothing
about *which* `ffmpeg`. Two of the arguments Montagent passed were removed in ffmpeg 9:

- `-vsync 0` in the seek (`decode.rs`). The `select` run exited 8, the fallback answered every
  instant, and frames were painted up to 200 ms early with `0 errors`. ADR-0113 made that
  failure loud; it did not make the seek work.
- `-filter_complex_script <file>` in the encoder (`encode.rs`), so no project with audio could
  render on 9.

CI installed apt's ffmpeg 6.1 on Linux and the newest release on macOS and Windows, so the
suite ran on two versions without saying which it supported.

## The decision

### 1. The floor is ffmpeg 7.1 or newer, built with libx264

Stated as the three arguments that decide it, defined once in `montagent_render::floor`:

| Constant | Spelling | Available since | What it replaced |
| --- | --- | --- | --- |
| `FPS_PASSTHROUGH` | `-fps_mode passthrough` | 5.1 | `-vsync 0`, removed in 9.0 |
| `FILTER_COMPLEX_FILE` | `-/filter_complex <file>` | 7.0 | `-filter_complex_script`, removed in 9.0 |
| `VIDEO_ENCODER` | `libx264` | needs `--enable-gpl` | — (ADR-0077) |

**One argument form, no second code path.** 6.1 would need the graph inlined on the command
line, which breaks Windows' ~32K limit on a large mix and would only ever run on the Linux legs.
**7.1 and not 7.0**, although the arguments alone would allow 7.0, because no pinned,
checksum-verifiable 7.0 static Linux build survives, and **the floor stated is the floor CI
tests**. `libvpx` is not part of the floor: only sources carrying alpha need it, and a missing
one stays a failure where such a source is decoded.

`decode.rs` and `encode.rs` build from these constants. `floor.rs`'s
`the_call_sites_build_their_version_sensitive_arguments_from_this_module` fails if either file
spells one of them, or a retired spelling, by hand. That is the link #477 §7 asked a test to
pin: a qualification that exercised `-fps_mode` while the seek still sent `-vsync` would pass on
ffmpeg 9 and let the seek fail anyway.

### 2. A failed spawn is never an empty answer, at every spawn

ADR-0113 made the rule for `frame_at` and `frames_from`. **It is now an invariant over every
spawn in the workspace, including the next one**, because a capability test only covers the
breakages already known and this rule also covers ffmpeg 10's. There are five spawns, each
listed with how it honours the exit status in `tests/spawn_exit_status.rs`:

| Site | How the exit status is honoured |
| --- | --- |
| `decode.rs` ×2 | `frame_at` checks both runs before reading stdout; `frames_from` waits at end of stdout (ADR-0113) |
| `encode.rs` | `Encoder::seal` checks before anything is published (ADR-0093, ADR-0109) |
| `floor.rs` | the qualification: a non-zero exit *is* the verdict |
| `probe.rs` | `ProcessRunner` records it; `tool_failure` is read before any media fact (ADR-0091) |

A spawn added, moved or removed fails that test until its author records how it complies. The
list is an audit, not a proof: the test counts `Command::new(`, and reading the exit status is
still the author's job.

### 3. The tool qualification: one null encode per `ffmpeg`, per process

`floor::qualify` runs one null encode through all three arguments: 16×16 black RGB on stdin, the
graph `[0:v]null[v]` read through `-/filter_complex` from a scratch file, `-fps_mode
passthrough`, `libx264`, one frame, the null muxer. **Not a version parse**: a git build prints
`N-xxxxx-g<hash>`, distributions add suffixes, and no version string can reveal a missing
libx264. **Not the real call sites on synthetic input**, which would make a failure ambiguous
between the tool and the fixture.

It runs **the first time `tools::resolve` finds a given `ffmpeg` in this process**. The verdict
is kept in memory, keyed by the resolved path, and **never on disk**: a cached verdict is a new
way for an answer to outlive an upgrade, ADR-0092's lesson. The lock is held across the spawn,
so concurrent MCP calls (ADR-0108) wait for one qualification rather than each running its own.

It is called the **tool qualification**. Not a *probe*, which is the verb and ADR-0092's cache,
and not a *check*, which asks a question of the project.

**`tools::resolve` now returns only a qualified `ffmpeg`**, so no verb that spawns one can reach
an unqualified one by forgetting to ask: `render`, `preview`, `frame`'s painter and `measure`'s
coverage all resolve through it. The paths that need only `ffprobe` (`validate`'s disk half,
`probe`, the decoder choice) resolve through `tools::resolve_found`, which returns the found
tools *beside* the verdict, and the session carries it.

### 4. `E-TOOL-UNSUPPORTED`, a sibling of `E-TOOL-MISSING`

`Error`, exit 70, `NotAboutDocument`, `Internal`, template `"{reason}"`, the same row shape as
ADR-0091's code. Its own code because **the remedy is to upgrade**: `-MISSING` would be a false
name, and `E-INTERNAL` would blame Montagent.

The sentence names the floor, the capability that failed, the resolved path, and the tail of
`ffmpeg`'s stderr. Verbatim from `validate`, with the stand-in `ffmpeg` of the Evidence section
(its directory shortened here):

```
error  E-TOOL-UNSUPPORTED
  Montagent needs ffmpeg 7.1 or newer, built with libx264; ffmpeg at <stand-in>/ffmpeg failed the tool qualification on reading a filter graph from a file (-/filter_complex, ffmpeg 7.0+): it exited with exit status: 8 — Unrecognized option '/filter_complex'. / Error splitting the argument list: Option not found
```

The capability is read from `ffmpeg`'s own prose, so it only ever narrows the sentence. Where
stderr names none of the three, the sentence says the null encode failed and carries the stderr,
which is still all that is known. The tail is its last four lines, capped at 600 characters.

**`Missing::fail` chooses among the three codes, and nothing else does.** Not on `PATH` is
`E-TOOL-MISSING`; found and qualification-failed is `E-TOOL-UNSUPPORTED`; found and unable to
run is `E-INTERNAL`. `into_report` now goes through `fail` rather than repeating the choice.
The painter's and the encode loop's tool channel (`Declined::Tool`, `Stop::ToolMissing`) carry
the `Missing` whole, where they carried its sentence, so they no longer name `E-TOOL-MISSING`
for a binary that was found. That was already wrong for a found binary that would not run,
which ADR-0091 classes `E-INTERNAL`.

### 5. A spawn that fails after qualification keeps its existing code

There is no catch-all `E-TOOL-FAILED`. Once the tool has qualified, a non-zero exit is about
*this input*, a corrupt or truncated source, and the call site that knows the element keeps its
error path, which ADR-0113 already makes carry the exit status and stderr. **That answers
ADR-0113's open note:** `E-NOT-PAINTED-UNDECODABLE` blaming the source is correct once
qualification has passed, and on ffmpeg 9 before this change users now meet
`E-TOOL-UNSUPPORTED` first.

### 6. `validate` reports it at `error`, and still reads the disk

`render` is guaranteed to refuse, so `validate` says so: ADR-0006's *"refused or guaranteed
wrong"*. It is raised **after** the disk half, not instead of it, because that half needs only
`ffprobe`, and it belongs to no check set: under ADR-0112 it is refusal-shaped, not a check. The
document half's findings and the disk half's both stand beside it, and the run exits 70.

**Stated cost: `validate` is no longer ffprobe-only.** It spawns `ffmpeg` once per process, so a
machine with a working `ffprobe` and an unusable `ffmpeg` now gets an `error` from `validate`.

**Stated limit: a project with no media never resolves the tools**, so its `validate` says
nothing about `ffmpeg`, which is exactly what it says today about a missing one. Its `render`
refuses with `E-TOOL-UNSUPPORTED` at once and before any frame, so no wall clock is lost; but a
`validate` that is clean on such a machine is not a promise that it can render. Qualifying on
every `validate` would make a text-only project need an `ffmpeg` to be checked, which it does not
need today and which #477 §6 did not ask for: its rule is *every verb that resolves the tools*.

### 7. CI tests both ends of the range

- **The floor.** Both Linux legs and the reference-frame job install BtbN's
  `ffmpeg-n7.1.5-12-g1fdbca85aa` static GPL build from the month-end release
  `autobuild-2026-07-31-14-10`, verified by SHA-256 in `ci/install_ffmpeg_floor.sh`. A month-end
  release because those survive where BtbN's dailies are pruned. The checksums are BtbN's own
  `checksums.sha256`, cross-checked against GitHub's asset digests when the pin was set.
- **The top.** macOS (`brew`) and Windows (`choco`) keep installing the newest release, so the
  next ffmpeg that removes an option Montagent uses turns CI red without anyone bumping a pin.
- **Accepted gaps.** The newest release is never tested on Linux, and a failure that happens only
  at the floor on macOS or Windows goes unseen. Both are accepted because the arguments are
  identical on every OS.

## Evidence

Run on macOS aarch64 against both ends of the range, with each `ffmpeg` first on `PATH`:

| | ffmpeg 9.0.2 (Homebrew) | ffmpeg 7.1.5 (Homebrew `ffmpeg@7`) |
| --- | --- | --- |
| `cargo test --workspace --all-targets` | 82 of 82 test binaries pass | 82 of 82 test binaries pass |
| #479's 12 (`seek_clamp` ×5, `render` ×5, `reference_video`, `world_effects`) | pass | pass |

On 9.0.2 those 12 had failed before this change, with ADR-0113's refusal naming `-vsync`.

`tests/adapters.rs` drives the qualification failure end to end with a stand-in `ffmpeg` that
rejects `-/filter_complex` the way 6.1 does, beside the real `ffprobe`:

- `cli_validate_on_an_ffmpeg_below_the_floor_is_an_error_and_still_reads_the_disk`: exit 70,
  `E-TOOL-UNSUPPORTED` naming the floor, the capability, the path and ffmpeg's words, **and**
  `E-RETIRED-KEY` from the document half **and** `E-SOURCE-MISSING` from the disk half.
- `cli_render_on_an_ffmpeg_below_the_floor_refuses_before_the_encoder`: exit 70,
  `E-TOOL-UNSUPPORTED`, and nothing at the output path.

`docs/adr/frame_at_or_before_check.sh` (ADR-0096's evidence) and
`ci/reference_frame_instants.py` move to `-fps_mode passthrough` as well; the second was not in
#479's list and would have broken the reference-frame job on the new pin. The first reports
*"found no defects"* on both 9.0.2 and 7.1.5; the second's `--instants-only` half reproduces
every claim on 9.0.2, and was not run locally on 7.1.5 — CI's reference-frame job is that run.

## Scope

It states no floor for `ffprobe`, which is resolved and spawned as before; no `ffprobe`
argument has been removed by 9, and ADR-0113's rule already covers one that is. It does not
change what `probe` reports: `probe` resolves through `resolve_found`, so it runs the qualification, and its report
is unchanged. #477 §6 names `validate` as the verb that states the verdict.
