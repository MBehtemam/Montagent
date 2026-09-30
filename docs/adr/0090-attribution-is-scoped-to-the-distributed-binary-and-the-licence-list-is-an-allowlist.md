---
status: accepted
---

> **Amended by [ADR-0122](0122-the-chrome-face-is-an-embedded-ligature-free-monospace-and-ofl-joins-the-allowlist.md).** The allowlist gains
> `OFL-1.1`. Its reciprocal clause binds the font software alone, never the program it is
> bundled with, so it cannot reach ADR-0009's invariant. Decision 3's derive-never-assert rule
> extends to the chrome face the binary embeds: `bundled.json` attests its hash, and the script
> checks presence, digest, notice and allowlist.

# Attribution is scoped to the distributed binary, and the licence list is an allowlist

[ADR-0064](./0064-packaging-cargo-and-releases-all-six-targets-passive-updates.md) settled
what Montagent distributes: a `montagent` binary per desktop tier-1 target in a GitHub
Release, plus four crates on crates.io. It did not say what notices ride along with them, and
[#362](https://github.com/MBehtemam/Montagent/issues/362) measured the answer — *nothing*. The
archive was the binary and a directory entry, carrying neither Montagent's own MIT text nor a
word about the other people's code linked into it.

[ADR-0009](./0009-rust-host.md) separately turns on an invariant that nothing enforced:
FFmpeg is spawned from the user's `PATH` and never linked, because `libx264` is GPL and
linking `libavcodec` would make the shipped binary a GPL combined work. A copyleft dependency
arriving by any other route would break the same invariant just as thoroughly, and would do it
quietly.

This ADR settles three things, all of them standing rules rather than one-off work:
**what attribution is scoped to**, **how the permitted licence set is expressed**, and **where
the Skia prebuilt's contents come from**.

## What was measured

`ci/third-party-notices/` — `bundled.json` and the notice texts beside it — with
`ci/assert_third_party_attribution.py` as the re-executable check. It asserts both directions
and was confirmed to fail in each: removing a vendored library from the manifest fails, and
attributing one that is not in the archive fails too. `deny.toml` is the licence gate,
`THIRD-PARTY.md` the generated result.

The measurements the decisions below rest on, taken against all six targets' prebuilts at
`skia-binaries` tag `0.153.2`, key suffix `jpegd-jpege-pdf`:

| Claim | How it was derived |
| --- | --- |
| `skia-bindings`' `license = "MIT"` is not the licence of what ships | The prebuilt archive contains `LICENSE_SKIA`, which is BSD-3-Clause, Copyright 2011 Google Inc. |
| The prebuilt vendors **five** third-party C/C++ libraries | Expat, libjpeg-turbo, libpng, Wuffs, zlib — the `third_party/externals/<name>/` path components of the MSVC `.lib` members, cross-checked against the gn target names of the Unix `.a` members |
| No ICU, HarfBuzz or FreeType is vendored | Absent from all six archives. The Skia pin leaves `textlayout` off, so `skia_use_icu` and `skia_use_harfbuzz` are false and `embed-icudtl` is inert — the prebuilt key has no icu component in it |
| FreeType and fontconfig on Linux are *system* libraries | Only Skia's own ports (`typeface_freetype`, `fontmgr_fontconfig`) are in the archive; neither library's code is |
| The distributed binary links **128** third-party crates under **4** licences | `MIT`, `Unicode-3.0`, `Apache-2.0`, `Zlib` — `cargo about` over `crates/montagent`'s normal dependency graph on ADR-0064's six targets |
| `r-efi` is LGPL-2.1-or-later and reaches nothing | It exists only for UEFI. `cargo deny list -f json` with `[graph] targets` commented out reports two copies of it under that licence; with the six targets in place it is not in the graph at all, and `cargo deny check licenses` is clean |

## The decisions

**1. Attribution is owed by what is distributed, and scoped to it.** `THIRD-PARTY.md` covers
`crates/montagent`'s *normal* dependency graph over ADR-0064's six targets — not this
workspace's. The difference is not cosmetic. [ADR-0010](./0010-skia-safe-rasterizer-text-beside-it.md)
requires the two-arm harness to stay in-tree, and `image`, `tiny-skia`, `exr`, `ravif` and
`libfuzzer-sys` come with it; a workspace-scoped listing would attribute all of them to a
binary that links none. The scoping lives in `about.toml` and in `deny.toml`'s `exclude-dev`,
and the two agreeing is load-bearing, not tidiness.

**2. The permitted licence set is an allowlist, and nothing copyleft is on it.** A licence
nobody has considered *fails*, rather than passing because no rule happened to name it. This is
the direction ADR-0009's invariant needs: a denylist of the GPL family would pass the first
copyleft licence somebody forgot to enumerate. `deny.toml` carries no `exceptions`, and adding
one is a decision that belongs in an ADR amending this one rather than in a config file.
Advisory exceptions are narrower and may be taken inline, but each must name the path by which
the crate reaches the tree and why that path cannot reach a release archive.

**3. What the Skia prebuilt contains is derived from the archive, never asserted in prose.**
This is the rule the other two depend on, because it is the one a manifest-reading tool cannot
supply. `cargo about` reads `skia-bindings`' manifest and reports MIT; the artifact reports
BSD-3-Clause plus five vendored libraries. So `ci/assert_third_party_attribution.py` reads the
member names out of whichever prebuilt is in `target/` and checks them against the manifest in
both directions — nothing in the archive unattributed, nothing attributed absent. A Skia bump
that vendors a sixth library fails a build rather than being discovered by a stranger.

The Unix half of that derivation needs a committed list of Skia's own gn targets
(`skia_own_gn_targets`), because an MSVC `.lib` member names its full source path and announces
a new `third_party/externals/` directory, while a Unix `.a` member is a bare gn target name that
says nothing about where it came from. A Skia bump that adds a target will fail there too. That
is intended: deciding whether a new target is Skia's own or somebody else's is a judgment, and
the failure is the prompt to make it.

## What this obliges, concretely

The IJG License's clause (2) is the one condition here that is a sentence someone has to write
rather than a text to reproduce: a distribution of executable code only must state in its
accompanying documentation that *"this software is based in part on the work of the Independent
JPEG Group"*. `THIRD-PARTY.md` is that documentation and carries the sentence, and the release
archive carries `THIRD-PARTY.md`.

`LICENSE-MEDIA.md` deliberately does **not** ride along in the archive. It is
[#358](https://github.com/MBehtemam/Montagent/issues/358)'s canonical list of the
rights-reserved fixture media, and not one of those paths is in a release tarball; including it
would put ninety filenames in front of a reader whose archive contains none of them. The
pointer at the foot of `LICENSE` names it, and the repository is where it lives.

## What this does not decide

- **The crate tarballs.** The four published crates still carry `license = "MIT"` with no
  licence text in the tarball and no `THIRD-PARTY.md`.
  [#366](https://github.com/MBehtemam/Montagent/issues/366) owns the manifests and the publish
  step; this ADR's rules apply there when it lands, and the same file discharges them.
- **A `credits` MCP verb.** [#356](https://github.com/MBehtemam/Montagent/issues/356) rules a
  tenth verb out of scope against
  [ADR-0078](./0078-preview-is-the-ninth-mcp-verb-and-its-unstated-readings-are-ratified.md)'s
  ratified nine. A static `THIRD-PARTY.md` discharges the obligation on its own.
- **Whether the licence set should ever widen.** It may; that is an amendment to this ADR, with
  the reasoning written down, not a config edit.
