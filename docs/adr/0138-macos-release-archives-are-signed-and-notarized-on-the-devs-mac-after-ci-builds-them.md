---
status: accepted
amends: 0064 (the release pipeline still builds all six archives, but the two macOS ones are signed and notarized afterwards on the dev's Mac by `ci/sign_macos_release.py`, not by CI; the macOS asset stays a bare binary in the same `.tar.gz`)
---

# macOS release archives are signed and notarized on the dev's Mac after CI builds them

> **Amended by [ADR-0139](./0139-release-yml-signs-and-notarizes-the-macos-archives-before-it-publishes.md)**: signing now runs in `release.yml`'s `sign-macos`
> job before the release is published; this ADR's local script stays as the fallback.

[#604](https://github.com/MBehtemam/Montagent/issues/604), on the map
[#583](https://github.com/MBehtemam/Montagent/issues/583). Decisions from
[#587](https://github.com/MBehtemam/Montagent/issues/587) and
[#588](https://github.com/MBehtemam/Montagent/issues/588), resting on the facts in
[#584](https://github.com/MBehtemam/Montagent/issues/584),
[#585](https://github.com/MBehtemam/Montagent/issues/585) and
[#586](https://github.com/MBehtemam/Montagent/issues/586).

## The gap

ADR-0064 chose "signed per-platform GitHub Release binaries" and named macOS codesigning
and notarization as necessary release work. It also said "Release CI is six
`cargo build --release` jobs". It didn't say where signing runs or what the signed
macOS asset is.

Unsigned, the gap is real. [#360](https://github.com/MBehtemam/Montagent/issues/360)
measured it on macOS 27.0: quarantine survives `tar xzf`, and an MCP client spawning the
binary gets a pid that blocks on a dialog nobody sees, then dies with SIGKILL and empty
stderr. Every later run is killed silently in about 2 s, because the denial is cached.

## Decision

### 1. CI builds; the dev's Mac signs

`release.yml` still builds and publishes all six archives and `SHA256SUMS`, unchanged.
Afterwards, on the dev's Mac, `ci/sign_macos_release.py <tag> --upload` replaces the two
macOS archives and `SHA256SUMS` on that release with signed and notarized ones.

Signing doesn't move into CI because:
- **The credentials stay where they were created.** The Developer ID Application identity
  is in the dev's login keychain. The notary credential is a team App Store Connect API
  key (Developer role) stored as the keychain profile `montagent-notary` (#585). Signing
  in CI would mean exporting that identity as a `.p12` into a secret store, and it is
  the most sensitive thing in this release process.
- **CI can't run anyway.** GitHub Actions has been off for billing since 2026-09-19.
- **Notarization time is unbounded.** A team's first submission took about 2 h (#586);
  later ones took about a minute. The script submits with `--no-wait` and polls, so no
  job timeout applies.

The build stays in CI because nothing about signing needs it to move, and keeping all six
targets on one builder is what ADR-0064 chose.

### 2. The signed asset is the bare binary, in the same `.tar.gz`

No `.pkg`. The archive's name and layout are exactly what `release.yml` produced, so
`cargo binstall` and anyone scripting the download see no change.

- A notarized binary in a quarantined, `tar`-extracted archive **starts silently on its
  first MCP spawn** when online (#586: 0.28 s to the `initialize` reply). The log shows
  `syspolicyd` fetching the ticket inline (`Inserting ticket`, then
  `GK eval - was allowed: 1, show prompt: 0`).
- A bare binary **can't be stapled**, so an offline first launch is expected to be
  blocked (#584). That was **not** measured: #586 skipped the offline run. The README
  carries the fallback (`xattr -d com.apple.quarantine`) for that case.
- A `.pkg` would close the offline gap. It would cost an admin password at install, a
  different install location and story, and a second signing step. It can be added later
  as an extra asset if anyone actually hits the offline case.

### 3. The signature

```
codesign -s "Developer ID Application: … (8XTSMPCT3L)" -f --timestamp -o runtime \
         -i io.github.mbehtemam.montagent montagent
```

- **The identifier `io.github.mbehtemam.montagent` is fixed forever once shipped.** A
  tool with no `Info.plist` otherwise takes its filename plus a hash as its identifier
  (#584).
- **Hardened runtime, and no entitlements.** #586 rendered `examples/hello-text`, Skia's
  CPU path plus the spawned `ffmpeg`, under `-o runtime` with none.
- **The notary service takes a zip of the binary.** It rejects `.tar.gz` (#584). A
  ticket is a list of code hashes, so the same signed bytes repacked into the `.tar.gz`
  are covered.
- **Nothing touches the binary after `codesign`.** `SHA256SUMS` is recomputed over the
  repacked archives.

### 4. What the script guarantees, and what it refuses

`ci/sign_macos_release.py`:
- checks each downloaded archive against the release's `SHA256SUMS` before signing it;
- refuses a binary that is already Developer ID signed, so a second run can't replace
  notarized bytes with an un-notarized signature;
- checks that Apple's notarization log names the exact code hash it signed;
- checks that the repacked archive extracts to a tree identical to the signed one;
- checks that `codesign --verify --check-notarization -R=notarized` passes on the repacked
  binary. A signed but never-notarized control **fails** this check, so it tells the two
  apart, and it works for an x86_64 binary that an Apple-silicon Mac without Rosetta
  can't run;
- rewrites only the two macOS lines of `SHA256SUMS`;
- with `--upload`, reads the published assets back and compares bytes.

### 5. Two checks that look right and aren't

Both reject **every** bare binary, notarized or not:
- `spctl --assess -t exec` says "the code is valid but does not seem to be an app";
- `syspolicy_check distribution` says "Notary Ticket Missing" because nothing is stapled.

Neither goes in the README or the script as a health check.

## Consequences

- **A release has a manual step.** Until the dev runs the script, the macOS assets on a
  fresh release are unsigned. Run it before announcing the release. `--clobber` deletes
  each asset before re-uploading it, so for a moment the asset is missing.
- **Releases depend on one Mac and one keychain.** Losing them means re-creating the
  certificate (the Account Holder can) and generating a new API key. Neither can be
  recovered from the repository, by design.
- **v0.1.0 ships unsigned**, as decided before this ADR. The first signed release is the
  next tag.
- **The certificate expires.** A Developer ID Application certificate lasts five years.
  The secure timestamp keeps binaries signed before expiry valid afterwards; new releases
  need a renewed certificate.
- **Signing in CI stays possible later.** The script is the whole procedure. Running it
  on a macOS runner with an imported `.p12` and the API key as secrets is a change of
  where it runs, not of what it does.
