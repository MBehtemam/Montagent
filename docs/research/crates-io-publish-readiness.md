# What this workspace needs before `cargo publish` will succeed

Research for [#361](https://github.com/MBehtemam/Montagent/issues/361), part of the map [#356](https://github.com/MBehtemam/Montagent/issues/356).

**Date of research:** 2026-09-25. Name availability, registry limits and toolchain behaviour are as of that date.

**Nothing was published.** Every registry interaction here is either a read of the crates.io API or `cargo publish --dry-run`, which ends in `warning: aborting upload due to dry run`. `cargo publish` is irreversible — a published version can be yanked but never replaced — so this ticket deliberately stops at the last check before upload.

**Sourcing rule applied:** every claim is either a command run against this checkout (transcript quoted), a read of a manifest in this repo, or a primary source (the Cargo Book, the crates.io API, docs.rs). Where a fact could only be reached through a search-engine summary, it is marked **[secondary]**.

**Local toolchain used for the dry-runs:** `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` / `rustc 1.95.0 (59807616e 2026-04-14)` on `aarch64-apple-darwin`. `release.yml` pins `toolchain: "1.90"`; where that difference matters it is called out.

---

## The headline

**All four crates package and verify cleanly today — but only as a workspace.** There is no blocking manifest defect and no size problem. The one real blocker is that `release.yml`'s publish loop uses the wrong command shape: it publishes the four crates one at a time with `sleep 30` between them, and `cargo publish -p montagent-core` **fails outright** on a first publish because `montagent-render` is not yet in the crates.io index. `cargo publish --workspace` — stabilised in Cargo 1.90, exactly the pinned toolchain — resolves the interdependencies through a temporary local registry and succeeds.

Everything else on the list is a quality gap (no README anywhere, no `keywords`, no `categories`, no per-crate `LICENSE`), not a gate.

---

## 1. Name availability — all four are free

Queried the crates.io API directly:

```
$ for n in montagent montagent-core montagent-text montagent-render; do
    curl -s "https://crates.io/api/v1/crates/$n"; done
{"errors":[{"detail":"crate `montagent` does not exist"}]}
{"errors":[{"detail":"crate `montagent-core` does not exist"}]}
{"errors":[{"detail":"crate `montagent-text` does not exist"}]}
{"errors":[{"detail":"crate `montagent-render` does not exist"}]}
```

All four names are unregistered. Source: `https://crates.io/api/v1/crates/{name}`.

> [!WARNING]
> Name availability is a fact with a shelf life, and crates.io has no reservation mechanism. This is the one finding in this document that can be falsified by a stranger between now and the first tag.

---

## 2. Manifest metadata — what is present, what is missing

Read from `crates/*/Cargo.toml` and the `[workspace.package]` table in the root `Cargo.toml`.

| Field | `montagent-core` | `montagent-text` | `montagent-render` | `montagent` |
|---|---|---|---|---|
| `description` | present (own) | present (own) | present (own) | present (own) |
| `license` | `MIT` (inherited) | `MIT` (inherited) | `MIT` (inherited) | `MIT` (inherited) |
| `repository` | inherited | inherited | inherited | inherited |
| `readme` | **missing** | **missing** | **missing** | **missing** |
| `keywords` | **missing** | **missing** | **missing** | **missing** |
| `categories` | **missing** | **missing** | **missing** | **missing** |
| `homepage` | missing | missing | missing | missing |

**Only `description` and `license`-or-`license-file` are enforced by crates.io.** The Cargo Book's publishing chapter lists `license` or `license-file`, `description`, `homepage`, `repository` and `readme` as fields to "fill out … before publishing", and says of the other two: *"It would also be a good idea to include some `keywords` and `categories`, though they are not required."* Source: <https://doc.rust-lang.org/cargo/reference/publishing.html>.

So **nothing in this table blocks a publish.** Both enforced fields are set on all four crates, and the dry-runs below confirm it.

Three things are nonetheless worth fixing before the first irreversible upload, because a published version cannot be amended:

- **There is no `README.md` anywhere in this repository** — not at the root, not in any crate. `cargo package --list` confirms none is packaged. The `readme` key defaults to `README.md` in the package directory, so with no file there is nothing to default to, and every crates.io page would render with only the one-line `description` and an empty body.
- **`LICENSE` exists only at the repo root** (`/LICENSE`, 1080 bytes, MIT). `cargo package` only walks the package directory, so **no crate ships its licence text.** Verified: `grep -iE "license|readme" ` over all four `cargo package --list` outputs returns nothing. `license = "MIT"` satisfies crates.io's SPDX requirement, so this is not a gate — but the published tarballs carry no copy of the licence they claim.
- **`keywords` and `categories` are the only discovery surface crates.io has.** Limits, from <https://doc.rust-lang.org/cargo/reference/manifest.html>: *"crates.io allows a maximum of 5 keywords"* and *"crates.io has a maximum of 5 categories"*, each keyword *"a maximum of 64 characters of length"*, and categories must match crates.io's fixed slug list (<https://crates.io/category_slugs>).

---

## 3. The test-only crate is correctly fenced

`docs/research/prototypes/rust-rasterizer/Cargo.toml`:

```toml
[package]
name = "rast-bench"
version = "0.0.0"
edition = "2021"
publish = false
```

**It carries `publish = false`.** This is not a latent hazard. Confirmed behaviourally as well as by inspection: `cargo publish --workspace --dry-run` packaged exactly four crates (`montagent-render`, `montagent-text`, `montagent-core`, `montagent`) and never mentioned `rast-bench`. ADR-0010's in-tree oracle is safe from `--workspace`.

Note the crate is also named `rast-bench`, not `montagent-*`, so even a manual `-p` typo cannot reach it by tab-completion of the family prefix.

---

## 4. Path dependencies pair `path` with `version` — correct shape

Root `Cargo.toml`:

```toml
[workspace.dependencies]
montagent-core = { path = "crates/montagent-core", version = "0.1.0" }
montagent-text = { path = "crates/montagent-text", version = "0.1.0" }
montagent-render = { path = "crates/montagent-render", version = "0.1.0" }
```

All three internal dependencies carry both keys, and every consuming crate reaches them through `.workspace = true` (`montagent-core` → `montagent-render`, `montagent-text`; `montagent` → `montagent-core`, plus `montagent-render` as a dev-dependency). This is the shape that lets Cargo strip `path` and emit `version` into the published manifest. **No change needed.**

---

## 5. The dry-runs — and the one real blocker

### 5a. Dependency order in `release.yml` is correct

`release.yml` publishes in the order `montagent-text`, `montagent-render`, `montagent-core`, `montagent`. Checked against the real graph:

- `montagent-text` → no internal deps
- `montagent-render` → no internal deps
- `montagent-core` → `montagent-render`, `montagent-text`
- `montagent` → `montagent-core` (and `montagent-render` as a dev-dep)

The two leaves are order-independent between themselves, so `text, render, core, montagent` is a valid topological order. **The order is not the bug.** (Cargo's own `--workspace` run chose `render, text, core, montagent` — equally valid.)

### 5b. The two leaves pass

```
$ cargo publish --dry-run --locked -p montagent-text
    Packaged 15 files, 110.7KiB (36.4KiB compressed)
   Verifying montagent-text v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.22s
warning: aborting upload due to dry run

$ cargo publish --dry-run --locked -p montagent-render
    Packaged 12 files, 207.3KiB (63.3KiB compressed)
   Verifying montagent-render v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.76s
warning: aborting upload due to dry run
```

Both clean. Note for #36's benefit: **the `skia-safe` prebuilt resolved locally on `aarch64-apple-darwin`** — `montagent-render`'s verification build finished in 3.76 s, which is a cache hit, not a Skia source build. macOS needs no `libfontconfig`/`libfreetype`; those are the Linux link line only, and `release.yml` already installs them on the Linux legs. **No dry-run had to be abandoned**; every one of the four completed.

### 5c. The two dependants fail — this is the blocker

```
$ cargo publish --dry-run --locked -p montagent-core
error: failed to prepare local package for uploading
Caused by:
  no matching package named `montagent-render` found
  location searched: crates.io index
  required by package `montagent-core v0.1.0`

$ cargo publish --dry-run --locked -p montagent
error: failed to prepare local package for uploading
Caused by:
  no matching package named `montagent-core` found
  location searched: crates.io index
  required by package `montagent v0.1.0`
```

This is the ordinary first-publish chicken-and-egg: single-package `cargo publish` rewrites the path dependency to a registry dependency and then insists that registry dependency already exist. `release.yml`'s `sleep 30` is an attempt to paper over exactly this by hoping the index has caught up — which is a race, and in the dry-run case (and in any re-run, or any fork without the earlier crates) it is not a race but a certainty of failure.

### 5d. `cargo publish --workspace` passes, all four

```
$ cargo publish --workspace --dry-run --locked
    Packaged montagent-render v0.1.0 — 12 files, 207.3KiB (63.3KiB compressed)
    Packaged montagent-text   v0.1.0 — 15 files, 110.7KiB (36.4KiB compressed)
    Packaged montagent-core   v0.1.0 — 146 files, 3.6MiB (1.8MiB compressed)
    Packaged montagent        v0.1.0 — 12 files, 243.8KiB (58.9KiB compressed)
   Verifying montagent-render … Finished in 12.27s
   Verifying montagent-text   … Finished in 14.39s
   Verifying montagent-core   …
   Unpacking montagent-render v0.1.0 (registry `target/package/tmp-registry`)
   Unpacking montagent-text   v0.1.0 (registry `target/package/tmp-registry`)
                               Finished in 38.50s
   Verifying montagent        …
   Unpacking montagent-core   v0.1.0 (registry `target/package/tmp-registry`)
                               Finished in 1m 29s
warning: aborting upload due to dry run   (×4)
```

The `tmp-registry` lines are the mechanism: Cargo stands up a temporary local registry, publishes each packaged crate into it, and resolves the next crate's registry dependency against that instead of against crates.io. This removes both the ordering burden and the propagation race.

`cargo publish --workspace` is documented — *"Publish all members in the workspace"* — at <https://doc.rust-lang.org/cargo/commands/cargo-publish.html>, alongside `--exclude SPEC…` (*"Must be used in conjunction with the `--workspace` flag"*).

**Availability on the pinned toolchain:** multi-package publishing was stabilised in Cargo **1.90**, which is exactly the version `release.yml` pins, so the workflow needs no toolchain bump. **[secondary]** — this attribution comes from the Inside Rust development-cycle post for 1.90 (<https://blog.rust-lang.org/inside-rust/2025/10/01/this-development-cycle-in-cargo-1.90/>) and the long-running tracking issue <https://github.com/rust-lang/cargo/issues/1169>; the per-version Cargo changelog could not be fetched directly during this session, so 1.90 is not confirmed against the changelog itself. It is confirmed present in local `cargo 1.95`. **If the 1.90 attribution is wrong, the fix is a one-line toolchain bump in `release.yml`, not a redesign.**

> [!IMPORTANT]
> `cargo publish --workspace` is **not atomic.** A server-side error partway through leaves the workspace partially published — and those versions can only be yanked, never replaced. **[secondary]**, same sources. This makes `--workspace` strictly better than the current loop (it cannot fail on ordering) but does not make the first publish risk-free.

---

## 6. Package size — no crate is close to the limit

**The limit is 10 MB on the `.crate` file:** *"crates.io currently has a 10MB size limit on the `.crate` file."* Source: <https://doc.rust-lang.org/cargo/reference/publishing.html>.

Measured from the `--workspace` dry-run:

| Crate | Files | Uncompressed | **Compressed (`.crate`)** | Headroom vs 10 MB |
|---|---:|---:|---:|---|
| `montagent-text` | 15 | 110.7 KiB | 36.4 KiB | ~99.6 % |
| `montagent-render` | 12 | 207.3 KiB | 63.3 KiB | ~99.4 % |
| `montagent-core` | 146 | 3.6 MiB | **1.8 MiB** | ~82 % |
| `montagent` | 12 | 243.8 KiB | 58.9 KiB | ~99.4 % |

**Nothing exceeds or approaches the limit. `exclude` is not required.**

### Why the 24 MB of fixtures never reaches the tarball

`cargo package` walks only the package directory, and the bulk media lives outside every crate. Breakdown of tracked binary media by directory (`git ls-files | grep -Ei '\.(png|mp3|mp4|jpg|jpeg|webm|wav|ttf|otf)$'`):

```
  62  docs/research/prototypes/thai-line-breaking/frames
  36  docs/research/prototypes/preview-floor-resolution/frames
  24  docs/research/prototypes/preview-floor-resolution/crops
  21  docs/research/prototypes/thai-vertical-metrics/frames
  11  fixtures/en-halloween-decorating/audio
   7  docs/research/prototypes/rust-rasterizer/frames
   6  crates/montagent-core/tests/golden        <- the only ones inside a published crate
   …
```

Total tracked media: **129 MB** — and all but the six golden PNGs sits under `fixtures/` or `docs/research/prototypes/`, i.e. outside `crates/`. `rast-bench` owns its own prototype frames and never publishes.

The complete set of non-`.rs` files in `montagent-core`'s tarball:

```
.cargo_vcs_info.json
Cargo.lock
Cargo.toml
Cargo.toml.orig
docs/format.md
tests/golden/fixture-intro.png
tests/golden/mask-explicit-rect.png
tests/golden/mask-under-transform.png
tests/golden/paint-vocabulary.png
tests/golden/text-features.png
tests/golden/fixture-sentence.png
```

Six golden PNGs (~1.2 MB on disk for the whole `tests/golden` directory) plus one Markdown file. **The goldens do not push `montagent-core` over the limit and are nowhere near doing so.**

### One quality issue the size check surfaced

`montagent-core` ships its whole `tests/` tree (≈140 `.rs` files), and many of those tests reach **outside** the package for their inputs — `crates/montagent-core/tests/{fixture,resolve,query,time,disk,timeline,fit,derived_t,retired_spellings,frame_environment}.rs` all reference paths above `CARGO_MANIFEST_DIR`. Those files are not in the tarball, so a downstream `cargo test` on the vendored crate cannot pass.

This is **not a publish blocker** — `cargo publish`'s verification step builds the package, it does not run its tests, which is why all four dry-runs succeeded. It is a choice worth making deliberately:

- **Leave as is** — 1.8 MiB, tests that a consumer cannot run.
- **`exclude = ["tests/**"]` on `montagent-core`** — drops the tarball to roughly 600 KiB and removes tests that were never runnable downstream. Costs nothing in-tree; `cargo test --workspace` reads the source tree, not the tarball.

Recommend the second, as an `exclude` chosen for coherence rather than for size.

---

## 7. Other blockers checked

| Concern | Finding |
|---|---|
| **`--locked`** | Not a problem. Every dry-run above was run with `--locked` and none reported lockfile drift. The `Cargo.lock` is committed and current. |
| **`edition = "2024"` + `rust-version = "1.90"`** | Not a problem. Edition 2024 requires Rust ≥ 1.85, so 1.90 satisfies it. crates.io accepts both keys; `skia-safe` 0.153.2 itself publishes with `edition = "2024"` and `rust_version = "1.85"` (crates.io API). |
| **Exact-pinned (`=`) dependencies** | Not a publish blocker. crates.io imposes no constraint on requirement operators, and `=` requirements resolve normally. They are a downstream-composability cost (a consumer who also depends on, say, `image` at a different version gets a hard conflict), which is ADR territory, not a publish gate. |
| **`docs.rs` building `montagent-render`** | Very likely fine, and **no `[package.metadata.docs.rs]` is required.** `skia-safe` 0.153.2 itself builds successfully on docs.rs — the builds page shows *"All builds succeeded"*, 2026-09-03, 1 m 33 s, rustc 1.100.0-nightly, 1.3 GB of documentation (<https://docs.rs/crate/skia-safe/0.153.2/builds>). Its own manifest carries **no** `[package.metadata.docs.rs]` section (checked by unpacking `skia-safe-0.153.2.crate` from static.crates.io), so the prebuilt download works in docs.rs's build environment unaided. `montagent-render`'s `default = ["no-skia-source-build"]` is the right default for docs.rs: a prebuilt miss becomes a fast, legible build failure rather than a 40-minute source compile that would hit docs.rs's build timeout. |
| **The `sleep 30` in `release.yml`** | Becomes dead code once `--workspace` replaces the loop. Its purpose — waiting for index propagation — is exactly what the temporary local registry makes unnecessary. |

---

## The checklist — exactly what to change

Nothing below is required to make the dry-run pass except **A1**. Everything else is "do it before the version becomes permanent".

### A. Blocking — the publish will fail without this

- [ ] **A1. Replace `release.yml`'s per-crate publish loop with a single workspace publish.** In `.github/workflows/release.yml`, the `publish-crates` job's `Publish in dependency order` step, replace the `for crate in … ; do cargo publish -p "$crate" --locked; sleep 30; done` body with:
  ```yaml
  run: cargo publish --workspace --locked
  ```
  Drop the `sleep 30` and the dependency-order comment above the job with it (the order is now Cargo's problem, and it solves it through a temporary local registry). `rast-bench`'s `publish = false` keeps it out; no `--exclude` is needed. Verified working: `cargo publish --workspace --dry-run --locked` packages and verifies all four.
  - [ ] Confirm `cargo publish --workspace` exists on the pinned `toolchain: "1.90"`. If it does not, bump the pin in that job to `"1.91"` or `stable`. (One-line change; see the **[secondary]** caveat in §5d.)
  - [ ] Consider adding a `cargo publish --workspace --dry-run --locked` step to `ci.yml`, so the publish path stops being a code path that has never executed.

### B. Permanent-record — fix before the first tag, cannot be amended after

- [ ] **B1. Write a root `README.md`.** There is currently none in the repository at all.
- [ ] **B2. Give each of the four crates a `readme`.** Either add a per-crate `crates/<name>/README.md` and let the default pick it up, or point at the root one explicitly. Note that `readme = "../../README.md"` does **not** package the file — Cargo only walks the package directory — so a per-crate file (or a build-time copy) is the shape that actually ships.
- [ ] **B3. Copy `LICENSE` into each crate directory** (`crates/montagent-core/LICENSE`, `-text`, `-render`, `montagent`), so each tarball carries the MIT text it claims. Confirmed absent from all four packages today.
- [ ] **B4. Add `keywords` to each crate** — max 5 each, max 64 chars each. Suggested:
  - `montagent`: `["video", "video-editing", "mcp", "agent", "cli"]`
  - `montagent-core`: `["video", "video-editing", "validation", "timeline", "agent"]`
  - `montagent-text`: `["text", "shaping", "typography", "line-breaking", "video"]`
  - `montagent-render`: `["video", "rendering", "rasterizer", "skia", "graphics"]`
- [ ] **B5. Add `categories` to each crate** — max 5 each, and each must be an exact slug from <https://crates.io/category_slugs>. Suggested, all four of which are real slugs:
  - `montagent`: `["multimedia::video", "command-line-utilities"]`
  - `montagent-core`: `["multimedia::video", "multimedia"]`
  - `montagent-text`: `["text-processing", "internationalization"]`
  - `montagent-render`: `["multimedia::video", "rendering", "graphics"]`
  - **All slugs above were validated** against `https://crates.io/api/v1/category_slugs` (104 slugs returned, 2026-09-25): `multimedia::video`, `command-line-utilities`, `multimedia`, `text-processing`, `internationalization`, `rendering`, `graphics` all exist. An unknown slug is rejected at upload time.
- [ ] **B6. Add `homepage` to `[workspace.package]`** (optional; the Cargo Book lists it among the fields to fill out). `homepage = "https://github.com/MBehtemam/Montagent"` is honest until there is a site.

### C. Recommended, not required

- [ ] **C1. `exclude = ["tests/**"]` on `montagent-core`.** Drops ≈1.2 MB of golden PNGs and ~140 test files that reference fixtures outside the package and therefore cannot run downstream. Takes the tarball from 1.8 MiB to roughly 600 KiB. Does not affect `cargo test --workspace` in-tree.
- [ ] **C2. Do a real dry-run of the exact release command before tagging**: `cargo publish --workspace --dry-run --locked`, on Linux as well as macOS, after B1–B6 land. B5 in particular is only validated by a real upload attempt or a careful slug check.

### D. Confirmed — no action needed

- [x] All four names free on crates.io (2026-09-25).
- [x] `description` present on all four (own, not inherited).
- [x] `license = "MIT"` and `repository` present on all four (inherited from `[workspace.package]`).
- [x] `rast-bench` carries `publish = false`; `--workspace` skips it.
- [x] Path dependencies pair `path` with `version` in `[workspace.dependencies]`.
- [x] `release.yml`'s stated dependency order is a valid topological order.
- [x] No crate approaches the 10 MB limit; largest is `montagent-core` at 1.8 MiB.
- [x] The 129 MB of tracked media lives outside `crates/` and is never packaged.
- [x] `--locked` clean; edition 2024 / rust-version 1.90 consistent; `=` pins harmless to publishing.
- [x] `skia-safe` builds on docs.rs unaided; no `[package.metadata.docs.rs]` needed.

---

## Could not verify

1. **That `cargo publish --workspace` is available in Cargo 1.90 specifically.** Confirmed present in local `cargo 1.95`; the 1.90 attribution is **[secondary]** (Inside Rust post, cargo#1169). The per-version Cargo changelog and the tagged `RELEASES.md` could not be fetched in this session. **Check this before relying on the pinned toolchain** — it is a one-line fix either way.
2. **Whether crates.io applies any limit beyond the 10 MB `.crate` size** (e.g. an uncompressed-size or file-count ceiling). The Cargo Book states only the 10 MB figure. With the largest crate at 1.8 MiB compressed / 3.6 MiB uncompressed, no plausible secondary limit is in reach.
3. **`montagent-render`'s own docs.rs build.** Only `skia-safe`'s build was checked (it succeeds). `montagent-render` cannot be built on docs.rs until it is published, so this is unverifiable before the fact by construction. The evidence that it will work is that its only non-trivial dependency already does, with the same prebuilt mechanism and a feature set (`no-compile`) that is strictly safer.
4. **Linux dry-runs.** All four dry-runs here ran on `aarch64-apple-darwin`. The Linux leg additionally needs `libfontconfig1-dev`/`libfreetype6-dev`, which `release.yml` already installs in the `publish-crates` job.
