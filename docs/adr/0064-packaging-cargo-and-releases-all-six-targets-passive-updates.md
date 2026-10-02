# ADR-0064: Distribution is cargo + GitHub Releases, all six desktop tier-1 targets, no update checks

> **Amended by [ADR-0115](./0115-ffmpeg-7-1-with-libx264-is-the-floor-and-a-tool-qualification-finds-out.md)**: CI's Linux legs and the reference-frame job test the `ffmpeg`
> **floor** — a pinned BtbN 7.1.5 static build verified by checksum, in place of apt's 6.1 —
> while macOS and Windows keep installing the newest release, so both ends of the supported
> range are tested.

> **Amended by [ADR-0138](./0138-macos-release-archives-are-signed-and-notarized-on-the-devs-mac-after-ci-builds-them.md)**: the release pipeline still builds all six
> archives, but the two macOS ones are signed and notarized afterwards on the dev's Mac by
> `ci/sign_macos_release.py`, not in CI, and stay a bare binary in the same `.tar.gz`.

## Status

Accepted

## Context

[ADR-0009](0009-rust-host.md) settled the host as Rust, shipping as a single compiled binary plus an `ffmpeg` the user supplies themselves — no bundled FFmpeg (GPL/patent duties rule that out), no interpreter, no `node_modules`-style runtime. [ADR-0010](0010-skia-safe-rasterizer-text-beside-it.md) settled the renderer as `skia-safe`, and a separate ticket (#7) verified that its required prebuilt feature-key (`jpegd-jpege-pdf`, CPU-only) publishes cleanly — no silent fallback to a from-source build — on all six of Rust's desktop tier-1 targets (x86_64/aarch64 across macOS, Linux glibc, Windows MSVC).

Left open by ADR-0009: the distribution **channel** (cargo / homebrew / raw release binaries / container), the binary's own **update policy**, and the **platform target matrix** for v1. Separately, [ADR-0016](0016-no-format-version-the-unknown-key-error-is-the-mechanism.md) already closed the adjacent question of whether the *project file format* needs a version field — it doesn't; a stale binary's unknown-key error is the compatibility mechanism. This ADR does not reopen that; it is scoped only to the binary's own distribution and update story.

Settled by a three-juror court (Opus, Sonnet, Haiku), each blind to the others, put to the same question text.

## Decision

**Channel: `cargo install montagent` (crates.io) plus signed per-platform GitHub Release binaries. No Homebrew for v1.** Unanimous 3/3.

The audience splits into two, and one of them isn't human: the person wiring up an MCP server already has a Rust toolchain (ADR-0009 already accepts "you supply your own `ffmpeg`" as a comparable bar), so `cargo install` is close to free — it's `cargo publish` plus zero packaging work, and it gives Montagent a canonical registry identity for anyone who wants to depend on it as a library. But an agent bootstrapping a session non-interactively, or a user without a toolchain, needs a plain, versioned, checksummable tarball it can fetch without trusting a third-party registry's build machinery — which GitHub Releases is and Homebrew isn't. The two channels are one release pipeline, not two: `cargo binstall` reads GitHub Releases, so shipping releases makes `cargo install` toolchain-optional as a side effect. Homebrew is deferred, not rejected — a formula's content is a pointer at a release tarball, so releases are a strict prerequisite for a tap, and a tap adds a third-party gatekeeper and a recurring per-release maintenance stream Montagent cannot productively carry pre-v1.

**Update policy: passive.** Unanimous 3/3. Montagent never checks for or mentions updates; whatever channel was used (`cargo install --force`, re-downloading a release tarball, eventually `brew upgrade`) is the entire update story, with zero code in Montagent devoted to it. This is the consistent continuation of two decisions already made, not a fresh one: Montagent is a per-session subprocess spawned by an MCP client, never a resident server ([ADR-0010](0010-skia-safe-rasterizer-text-beside-it.md)), so there is no process lifetime in which an update notice is actionable, and [ADR-0056](0056-remote-source-probe-session-scoped-no-persistent-cache.md) already established that Montagent does not make network calls on its own initiative — an update check is exactly the class of unprompted outbound request that principle exists to rule out. The cost — a user can run a stale build indefinitely with no nudge — is accepted; the version string remains reportable over MCP on request (pull, not push), and ADR-0016's unknown-key error already produces an actionable message in the one case where staleness actually breaks something.

**Platform matrix: all six desktop tier-1 targets** — macOS, Linux, Windows, each x86_64 and aarch64. 2/3, with a recorded dissent.

Majority reasoning: the one concrete risk to including a target — Skia prebuilt availability — was already retired for all six by #7, and this is a headless, no-GUI binary (ADR-0009's shape), so the usual reasons to drop Windows (window management, native dialogs, installer/notarization UX, HiDPI) don't apply; what's left is cross-platform process spawning and path handling, which Rust's std covers. Platform assumptions calcify fast once implementation starts, so the cheap moment to include a target is now, before any code exists — dropping it now and adding it later is a port, not a CI matrix entry.

Dissent (recorded, not adopted): macOS + Linux only, on the grounds that `ffmpeg`-on-PATH ergonomics (dylib/DLL loading, shell quoting, path separators, process-spawn semantics) are less consistent on Windows and untested by #7, which verified only the Skia prebuilt, and that the realistic early-adopter base skews Unix. The majority's rebuttal — that #7's verification is the only *measured* risk on the table, that the FFmpeg-on-PATH friction is a documentation problem rather than an architectural one, and that "Claude Code's installed base is not Unix-only" cuts the audience argument the other way — is accepted, but the dissent is worth revisiting if Windows FFmpeg/process-spawn integration turns out rougher in practice than assumed.

## Consequences

- Release CI is six `cargo build --release` jobs (one per target), not a from-source Skia matrix — cheap, because #7 already proved the prebuilt covers all six.
- macOS codesigning and notarization (an Apple Developer account, `notarytool`) becomes necessary release-engineering work; without it, Gatekeeper quarantines the release tarball on download.
- Montagent publishes to crates.io (`cargo publish`) and cuts GitHub Releases with per-target tarballs and checksums (`SHA256SUMS`), kept in lockstep by release tooling — no separate versioning scheme beyond what release tags already provide.
- No update-check code, no telemetry, no hardcoded "latest version" endpoint, ever, in v1. `montagent --version` (or the MCP equivalent) reports the running binary's version on request; nothing pushes.
- Homebrew is explicitly open for a later effort once release cadence is proven — not ruled out, just not v1's problem. A formula's existence is contingent on GitHub Releases existing first.
- If Windows FFmpeg/process-spawn integration proves measurably rougher during implementation, the recorded dissent is the pointer to revisit — this ADR does not treat that risk as closed, only as untested.

## Evidence

Three-juror court (Claude Opus 5, Claude Sonnet 5, Claude Haiku 4.5), each answering the same question text independently, blind to the others' ballots. Full ballots and the judge's read are recorded in the resolution comment on [#153](https://github.com/MBehtemam/Montagent/issues/153).
