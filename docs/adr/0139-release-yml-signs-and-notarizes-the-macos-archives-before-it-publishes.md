---
status: accepted
amends: 0138 (signing moves from the dev's Mac into `release.yml`'s `sign-macos` job, before the release is published; the local script stays as the fallback), 0064 (a pre-release tag is published as a draft and never reaches crates.io; `cargo publish` runs after the release, not beside it)
---

# `release.yml` signs and notarizes the macOS archives before it publishes

Asked for by the dev on 2026-10-02, right after the repository went public
([#364](https://github.com/MBehtemam/Montagent/issues/364)).

## The gap

ADR-0138 kept signing on the dev's Mac, for three reasons:
- the credentials stayed where they were made;
- CI was off for billing;
- notarization time is unbounded.

Two of those changed on 2026-10-02:
- **The repository is public**, so standard Actions runners cost nothing, and `release.yml`
  ran green on all six legs for v0.1.0 and v0.1.1.
- **Notarization is fast now.** After the team's first submission (about 2 h), every
  submission since has been accepted in about a minute (#605, v0.1.1).

ADR-0138's cost was a manual step on every release, and a window in which a fresh release
carried unsigned macOS archives until the dev ran the script.

## Decision

### 1. A `sign-macos` job signs before anything is published

The build legs upload their archives as artifacts. The macOS ones are named
`unsigned-<target>`; the other four are `dist-<target>`.

A new `sign-macos` job:
- runs on `macos-15`;
- downloads the two `unsigned-*` archives;
- runs `ci/sign_macos_release.py <tag> --archives unsigned --out dist`, the same procedure
  as ADR-0138's, with a new mode that signs archives on disk instead of a release's assets;
- uploads the results as `dist-<target>`.

The release job publishes **only** `dist-*`. So in this repository an unsigned macOS build
can't become a release asset. If the signing credentials are missing, `sign-macos` fails
and nothing is published. A fork without credentials passes its archives through unsigned,
with a warning.

### 2. The credentials live in a protected `release` environment

| Name | Kind | What |
|---|---|---|
| `MACOS_CERT_P12_BASE64` | secret | the Developer ID Application identity, exported as a password-protected `.p12`, base64 |
| `MACOS_CERT_P12_PASSWORD` | secret | that export's password |
| `NOTARY_API_KEY_P8_BASE64` | secret | the team API key `.p8` (#585), base64 |
| `NOTARY_API_KEY_ID` | variable | the key's ID, not secret |
| `NOTARY_API_ISSUER` | variable | the issuer ID, not secret |
| `CARGO_REGISTRY_TOKEN` | secret | crates.io, used only by `publish-crates` |

- **Who can use it.** Only `v*` tags and `main` can deploy to `release`. Only the admin can
  create `v*` tags (ruleset "Release tags") or merge into `main` (ruleset "Main"). Fork PR
  workflows never see environment secrets.
- **Throwaway keychain.** The `.p12` is imported into a keychain with a random password,
  and an `if: always()` step deletes that keychain and the `.p8` at the end of the job.
- **The trade ADR-0138 refused, made on purpose.** The signing identity now exists outside
  the dev's Mac. The risk is a workflow change that exfiltrates it. It's bounded by who can
  change workflows (the admin only), by the environment's ref policy, and by revocation:
  the Account Holder can revoke the certificate and issue a new one.

### 3. Pre-release tags rehearse the pipeline

A tag containing `-` (for example `v0.2.0-rc1`) publishes a **draft, pre-release** GitHub
Release and skips `publish-crates`. That runs every job, signing included, without a public
release or an irreversible crates.io version.

### 4. crates.io publishes after the release

`publish-crates` used to run beside the release, after `build`. It now needs `release`. A
crates.io version can only be yanked, never replaced, so it waits until every archive is
built, signed and published.

### 5. The local script stays

`ci/sign_macos_release.py <tag> --upload` from ADR-0138 still works. Use it when a release
was cut without signing, or when CI signing is unavailable. It refuses an
already-Developer-ID-signed binary, so it can't undo CI's work.

## Consequences

- **No manual step and no unsigned window.** A `v*` tag push ends with signed archives on
  the release.
- **Certificate expiry and key revocation break releases loudly.** `sign-macos` fails rather
  than publishing unsigned. Renewing means re-exporting the `.p12` and replacing two secrets.
- **The `.p12` export is a new artefact.** It should exist only in the environment secret and
  in the dev's password manager, never on disk afterwards and never in the repository.
- **A first notarization can still be slow.** Notarizing for a new team or a new key can
  take hours again. The job allows 240 minutes.
