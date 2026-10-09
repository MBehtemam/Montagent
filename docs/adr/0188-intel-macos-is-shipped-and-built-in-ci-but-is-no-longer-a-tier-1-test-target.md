---
status: accepted
amends: 0064 (Intel macOS, `x86_64-apple-darwin`, is no longer a tier-1 test target: the suite runs on five targets, Intel macOS gets a build-only CI leg that runs no tests, and a release still ships all six archives)
---

# Intel macOS is shipped and built in CI, but is no longer a tier-1 test target

[ADR-0064](0064-packaging-cargo-and-releases-all-six-targets-passive-updates.md) committed to
all six desktop tier-1 targets: `{aarch64, x86_64}` × `{apple-darwin, unknown-linux-gnu,
pc-windows-msvc}`. `ci.yml` ran the whole suite on each of them on a native runner, and
`x86_64-apple-darwin` ran on GitHub's `macos-15-intel`. The owner approved this decision.

## Why Intel macOS leaves the suite

- **The platform is ending.** Apple has announced that macOS 26 (Tahoe) is the last macOS
  release that will run on Intel Macs. This ADR records that as Apple's stated position. It
  does not predict when Intel Macs stop being used.
- **The runner is ending too.** GitHub has said `macos-15-intel` is the last Intel macOS
  runner image it will offer. It is also the slowest leg in the matrix.
- **It misses the scrub-preview budget on every run.** `preview_budget` enforces ADR-0021's
  `<5 s` for a 10 s scrub preview of the fixture, with ADR-0065's 720p proxy and 540p floor.
  After [ADR-0186](0186-a-proxy-tier-reads-a-shrunk-image-through-one-mip-level-and-the-deliverable-keeps-adr-0132s-rule.md)
  shipped, the Intel leg's 720p attempt still ran to the 5.0 s deadline and degraded to 540p.
  Rasterizing took about 148 ms per frame there, against 61–69 ms on the Linux legs. ADR-0186
  also records `frame_budget` misses on this leg only (586 ms and 612 ms against 500 ms).
- **So a tier-1 Intel leg is a check that is always red.** A check that is always red hides
  real regressions, because nobody can tell a new failure from the usual one. ADR-0186 already
  shipped the fix for the real regression, the paint cost of #552's trilinear minifier. What
  the Intel leg still reports is that its runner rasterizes at less than half the Linux legs'
  speed.

Two other options were rejected:

- **Raising `SCRUB_PREVIEW_LIMIT`.** The `<5 s` is ADR-0021's budget for everyone. Moving it
  because of one runner would weaken it on the other five legs too.
- **Exempting the Intel leg from the budget tests.** That keeps the slowest leg in the suite
  while removing the checks that made it fail. The leg would still cost a full suite run, and
  a test list that differs per target is a second matrix to keep in sync.

## The decision

**The tier-1 test targets are five:** `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`,
`x86_64-unknown-linux-gnu`, `aarch64-pc-windows-msvc` and `x86_64-pc-windows-msvc`. `ci.yml`'s
`suite` job runs the full suite on these five and on no others: Clippy, `cargo test
--workspace --all-targets --release`, the oracle, the doctests, the skill scripts and the
attribution check.

**Intel macOS keeps a build-only leg.** `ci.yml` has a second job, `build-only`, whose check
is named `build only x86_64-apple-darwin`. It runs on `macos-15-intel`, the same runner
`release.yml` and the canary use for that target. It does three things:

1. `cargo clippy --workspace --all-targets --target x86_64-apple-darwin -- -D warnings`. This
   type-checks the test code too, so a `cfg`-gated break on this target is still caught.
2. `cargo build --release --locked --target x86_64-apple-darwin -p montagent`, the same
   command `release.yml` runs, so a tag is not the first place the shipped binary fails to
   link.
3. `ci/assert_third_party_attribution.py --prebuilt-only`. What the prebuilt vendors is a
   fact about each target, and this archive still ships.

It runs no `cargo test`, no oracle, no ffmpeg and no budget. **Builds are checked, tests are
not run.** A green `build only x86_64-apple-darwin` says the binary compiles and links. It
says nothing about whether the binary behaves correctly on an Intel Mac.

**Why keep the build-only leg.** It is cheap. It needs no ffmpeg, it renders no pixels, and
`rust-cache` keeps it warm, so its cost is one Clippy pass and one release build. Without it,
code that stops compiling for `x86_64-apple-darwin`, for example behind a `cfg(target_arch)`
gate, would first fail on a release tag, for a binary this repository still ships. The leg
also needs nothing new to resolve. The Skia prebuilt for `x86_64-apple-darwin` (key
`jpegd-jpege-pdf`) is the one `release.yml` already builds against and the canary already
resolves cold every night.

**A release still ships six archives.** `release.yml`'s build matrix is unchanged. It builds,
signs and notarizes an `x86_64-apple-darwin` archive as before (ADR-0138, ADR-0139). Shipping
a binary and testing it are separate decisions. This ADR changes only the testing. The
consequence is that the Intel archive ships with weaker evidence than the other five: it
compiles and links, but it is not tested. The README says so.

**The canary still covers six targets.** `skia-canary.yml` resolves the prebuilt cold on all
six shipped targets, including Intel macOS on `macos-15-intel`. ADR-0010's revisit trigger, a
sustained prebuilt miss on a target, still matters for a target whose archive ships. The
canary builds one crate once a day and runs no tests, so it costs little.

## What changes where

- **`Cargo.toml`** `[workspace.metadata.skia]`: `targets` stays the six shipped targets. A new
  `build-only-targets = ["x86_64-apple-darwin"]` names the target that is shipped but not
  tested. The test targets are `targets` minus `build-only-targets`.
- **`crates/montagent-render/tests/skia_pin.rs`** holds all of this with literal sets.
  `the_target_matrix_is_adr_0064s_six` still requires the six shipped targets.
  `the_tested_targets_are_adr_0188s_five` requires exactly Intel macOS to be build-only and
  the other five to be tested. `both_workflows_cover_every_target_on_the_same_runner` now reads
  each workflow job by job. `ci.yml`'s `suite` must cover exactly the five, its `build-only`
  job exactly the build-only set, the canary and `release.yml`'s `build` job all six, and every
  job must run a given target on the same runner label. A dropped target or a drifted label
  still fails it.
- **Check names.** `every_per_target_check_name_is_unique` replaces
  `the_two_workflows_report_under_distinct_check_names`. It expands every per-target job name
  in `ci.yml` and the canary and requires each name to have exactly one producer. Only the
  suite may report a bare target triple. This matters here because branch protection may still
  list `x86_64-apple-darwin` as a required check after this lands. If the build-only leg
  reported under that name, a build would satisfy a requirement meant for the suite.

## Consequences

- **Branch protection must be updated by hand.** The required check `x86_64-apple-darwin`
  is no longer produced, so a rule that still requires it blocks every pull request. The new
  check is `build only x86_64-apple-darwin`. Whether to require it is the owner's choice.
- **ADR-0115 and ADR-0187's floating macOS ffmpeg** is now tested on one leg,
  `aarch64-apple-darwin`, rather than two. Its purpose, catching the next ffmpeg that removes
  an option Montagent uses, does not depend on the CPU architecture, because the arguments
  are the same on every OS.
- **A regression that shows up only on Intel macOS is not caught by CI.** That includes a
  rendering difference, a budget miss, or a process-spawn difference. Anyone who reports one is
  using an archive that was built and not tested, and the report is the first evidence.
- **Retiring the leg entirely** is the expected next step when GitHub removes
  `macos-15-intel`. At that point `release.yml` also has no native Intel runner, and whether
  to cross-compile the archive from `macos-15` or stop shipping it is a separate decision. This
  ADR does not make it.
