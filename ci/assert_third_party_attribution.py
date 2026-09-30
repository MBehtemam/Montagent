#!/usr/bin/env python3
"""Generates and re-checks `THIRD-PARTY.md` — the attribution Montagent owes for the
code it ships inside the `montagent` binary (#359), in **two** places: the repository
root, and `crates/montagent/THIRD-PARTY.md`, which rides along in the crates.io tarball
(#366) because `include` cannot reach a file outside its own package the way `license-file`
and `readme` can — confirmed by testing, not assumed.

    python3 ci/assert_third_party_attribution.py --write          # regenerate both copies
    python3 ci/assert_third_party_attribution.py --skip-prebuilt  # drift check, no Skia needed
    python3 ci/assert_third_party_attribution.py --prebuilt-only  # archive check, no cargo-about

Attribution is owed by the thing that is **distributed**, so the scope is the `montagent`
binary's normal dependency graph over ADR-0064's six targets — see `about.toml`, which is
where that scoping lives. The workspace is a wider set and the wrong one: `image`,
`tiny-skia`, `exr`, `ravif` and the `rust-rasterizer` harness are test-only and reach no
release archive.

Two halves, because two different things can rot:

  1. **Drift.** The Rust half of `THIRD-PARTY.md` is `cargo about`'s output. Regenerating it
     and diffing against the committed file is what makes a new dependency — or a
     dependency that changes licence — fail a build instead of silently shipping
     unattributed. Needs `cargo about` (`cargo install --locked cargo-about --features cli`).

  2. **The prebuilt.** `cargo about` reads manifests, and `skia-bindings`' manifest says
     `license = "MIT"` — the licence of the Rust binding code. What actually reaches the
     binary is a prebuilt static library holding Skia under BSD-3-Clause *plus five vendored
     C/C++ libraries*, and no manifest mentions any of them. So this half reads the member
     names straight out of whichever prebuilt is sitting in `target/` and checks them against
     `ci/third-party-notices/bundled.json` in **both** directions: nothing in the archive is
     unattributed, and nothing attributed is absent. A Skia bump that vendors a sixth library
     fails here rather than being noticed by a stranger.

     The MSVC `.lib` members carry their full source paths
     (`obj/third_party/externals/<name>/...`), which is what makes this a measurement rather
     than a restatement of the gn args; the Unix `.a` members carry bare gn target names, so
     the manifest's `gn_targets` is the mapping back.

  3. **The fonts.** The binary carries one font of its own — the chrome face `frame` draws its
     labels in (#421) — and no manifest-reading tool sees a file `include_bytes!` put there.
     So `bundled.json`'s `fonts` are checked with the drift half: each file is present and
     hashes to what is attested, its notice is present, and its licence is on ADR-0090's
     allowlist in both `about.toml` and `deny.toml`. A licence that is not there fails here
     exactly as it would for a crate.
"""

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, ".."))
NOTICES = os.path.join(HERE, "third-party-notices")
MANIFEST = os.path.join(NOTICES, "bundled.json")
OUTPUT = os.path.join(ROOT, "THIRD-PARTY.md")
# `montagent`'s own copy, physically in-tree rather than referenced. `include`/`exclude`
# only see inside a package directory — confirmed by testing (#366): an `include` entry
# pointing at `../../THIRD-PARTY.md` is silently dropped, unlike the special-cased
# `license-file`/`readme` path fields, which Cargo resolves and copies in from outside.
# So the only way this reaches the crates.io tarball — which ADR-0090 says it must,
# since `cargo install montagent` is a distribution channel with the same linked code as
# the GitHub Release binary — is an ordinary tracked file inside the package, generated
# and checked exactly like the root copy.
CRATE_COPY = os.path.join(ROOT, "crates", "montagent", "THIRD-PARTY.md")
BINARY_MANIFEST = os.path.join(ROOT, "crates", "montagent", "Cargo.toml")

# The four crates in this workspace. They are the thing being licensed, not a third party
# to it, and the repository's own `LICENSE` covers them. `cargo about`'s `private.ignore`
# only drops crates marked `publish = false`, and these are published, so they are dropped
# here instead.
OWN_CRATES = {"montagent", "montagent-core", "montagent-render", "montagent-text"}


# --- Reading a static library -------------------------------------------------------


def archive_member_names(path):
    """Member names out of a static library, for all three dialects the six targets
    produce: BSD (macOS — `#1/<len>` with the name inline in the body), SysV/GNU (Linux —
    a `//` long-name table referenced as `/<offset>`), and MSVC (`.lib` — the SysV layout,
    with full `obj/...` source paths as the names)."""
    with open(path, "rb") as f:
        data = f.read()
    if not data.startswith(b"!<arch>\n"):
        raise ValueError(f"{path} is not an `ar` archive")

    pos, longnames, names = 8, b"", []
    while pos + 60 <= len(data):
        header = data[pos : pos + 60]
        if header[58:60] != b"`\n":
            break  # not a header where one is due: stop rather than guess
        raw = header[0:16].rstrip()
        size = int(header[48:58].strip() or 0)
        body = pos + 60

        if raw == b"//":
            longnames = data[body : body + size]
        elif raw.startswith(b"#1/"):
            length = int(raw[3:])
            names.append(data[body : body + length].rstrip(b"\0").decode("utf-8", "replace"))
        elif raw.startswith(b"/") and raw[1:].isdigit():
            offset = int(raw[1:])
            end = longnames.find(b"/\n", offset)
            if end == -1:
                end = longnames.find(b"\n", offset)
            names.append(longnames[offset:end].rstrip(b"/").decode("utf-8", "replace"))
        elif raw not in (b"/", b"/SYM64/"):
            names.append(raw.rstrip(b"/").decode("utf-8", "replace"))

        pos = body + size + (size & 1)

    # `__.SYMDEF` / `__.SYMDEF SORTED` is the BSD symbol table, not a member.
    return [n for n in names if not n.startswith("__.SYMDEF")]


def find_prebuilts():
    """Wherever `skia-bindings`' build script has unpacked a prebuilt under `target/`.

    Found by shape rather than by path: a directory holding `key.txt` beside a `libskia.a`
    or `skia.lib`. The unpack flattens the archive into `<out>/skia/` and leaves an empty
    `skia-binaries/` behind it, so the archive's own directory name is not where to look.

    A tree built more than once has many identical copies — one per `skia-bindings-<hash>`
    — so the results are deduplicated on the prebuilt key. What varies between keys is the
    target and the feature set, which is exactly what changes what is vendored in."""
    by_key, target_dir = {}, os.path.join(ROOT, "target")
    for dirpath, _dirnames, filenames in os.walk(target_dir):
        names = set(filenames)
        if "key.txt" not in names or not (names & {"libskia.a", "skia.lib"}):
            continue
        with open(os.path.join(dirpath, "key.txt")) as f:
            by_key.setdefault(f.read().strip(), dirpath)
    return [by_key[key] for key in sorted(by_key)]


# --- The prebuilt half --------------------------------------------------------------


def check_prebuilt(manifest, prebuilt_dir, all_failures):
    """The archive in `prebuilt_dir` carries exactly what the manifest says it carries."""
    where = os.path.relpath(prebuilt_dir, ROOT)
    # This archive's own failures, so the summary line below reports on *this* prebuilt
    # rather than on whatever the drift half happened to find first.
    failures = []
    try:
        _check_prebuilt(manifest, prebuilt_dir, where, failures)
    finally:
        all_failures.extend(failures)


def _check_prebuilt(manifest, prebuilt_dir, where, failures):

    library = next(
        (
            os.path.join(prebuilt_dir, n)
            for n in ("libskia.a", "skia.lib")
            if os.path.exists(os.path.join(prebuilt_dir, n))
        ),
        None,
    )
    if library is None:
        failures.append(f"{where}: no libskia.a or skia.lib")
        return

    key_file = os.path.join(prebuilt_dir, "key.txt")
    key = open(key_file).read().strip() if os.path.exists(key_file) else "<no key.txt>"
    expected_suffix = manifest["prebuilt"]["key_suffix"]
    if not key.endswith(expected_suffix):
        failures.append(
            f"{where}: prebuilt key {key!r} does not end in {expected_suffix!r} — the Skia "
            f"feature set has moved, so what is vendored into it may have moved too "
            f"(see the Skia pin in the root Cargo.toml)"
        )

    # Skia's own licence, from the artifact rather than from a manifest. The Windows
    # archives ship the same text with CRLF, so the comparison is over normalised newlines.
    license_in_archive = os.path.join(prebuilt_dir, manifest["prebuilt"]["license_file_in_archive"])
    if not os.path.exists(license_in_archive):
        failures.append(f"{where}: no {manifest['prebuilt']['license_file_in_archive']} in the prebuilt")
    else:
        with open(license_in_archive, "rb") as f:
            got = hashlib.sha256(f.read().replace(b"\r\n", b"\n")).hexdigest()
        want = manifest["prebuilt"]["license_file_sha256"]
        if got != want:
            failures.append(
                f"{where}: the prebuilt's Skia licence hashes to {got}, not {want} — "
                f"ci/third-party-notices/skia.LICENSE is a stale copy of it"
            )
        committed = os.path.join(NOTICES, manifest["skia"]["notices"][0])
        with open(committed, "rb") as f:
            committed_hash = hashlib.sha256(f.read().replace(b"\r\n", b"\n")).hexdigest()
        if committed_hash != got:
            failures.append(
                f"{where}: ci/third-party-notices/{manifest['skia']['notices'][0]} is not the "
                f"text the prebuilt actually ships"
            )

    # What is vendored in, derived. `third_party/externals/<name>/` where the member names
    # carry paths (MSVC), the gn target name where they do not (Unix).
    declared = {entry["externals_dir"]: entry for entry in manifest["bundled"]}
    gn_to_dir = {
        target: entry["externals_dir"]
        for entry in manifest["bundled"]
        for target in entry["gn_targets"]
    }

    members = archive_member_names(library)
    if not members:
        failures.append(f"{where}: {os.path.basename(library)} has no members — unreadable archive?")
        return

    skia_own = set(manifest["skia_own_gn_targets"])
    seen, unattributed, unknown = set(), set(), set()
    for name in members:
        if "third_party/externals/" in name:
            external = name.split("third_party/externals/", 1)[1].split("/", 1)[0]
            if external in declared:
                seen.add(external)
            else:
                unattributed.add(external)
        elif "/" not in name:
            # A bare gn target name, which is all a Unix `.a` member gives. It is either a
            # vendored library's, Skia's own, or something nobody has classified yet — and
            # the third case is the one that would otherwise ship unattributed.
            gn_target = name.split(".", 1)[0]
            if gn_target in gn_to_dir:
                seen.add(gn_to_dir[gn_target])
            elif gn_target not in skia_own:
                unknown.add(gn_target)

    for external in sorted(unattributed):
        failures.append(
            f"{where}: the prebuilt vendors third_party/externals/{external}, which "
            f"ci/third-party-notices/bundled.json does not attribute — add it, with its "
            f"notice text, before shipping this binary"
        )

    for gn_target in sorted(unknown):
        failures.append(
            f"{where}: the archive has members from a gn target {gn_target!r} that is in "
            f"neither bundled.json's gn_targets nor its skia_own_gn_targets. Find out which "
            f"it is: if it is a newly vendored library its notice is owed, and if it is "
            f"Skia's own it belongs in skia_own_gn_targets."
        )

    for external in sorted(set(declared) - seen):
        failures.append(
            f"{where}: bundled.json attributes {declared[external]['name']} "
            f"(third_party/externals/{external}) but no member of "
            f"{os.path.basename(library)} comes from it — over-attribution, or the "
            f"gn_targets mapping is stale"
        )

    # Absent on purpose, and worth failing over if they ever arrive: these carry their own
    # notice obligations, and `embed-icudtl` plus a `textlayout` flip would bring them in.
    for external, why in (
        ("icu", "`embed-icudtl` is inert only while `textlayout` is off"),
        ("harfbuzz", "`skia_use_harfbuzz` follows `textlayout`"),
        ("freetype", "FreeType is meant to stay a *system* library on Linux"),
    ):
        # Either spelling: the externals path on MSVC, the `lib`-prefixed gn target on Unix.
        if external in seen or external in unattributed or f"lib{external}" in unknown:
            failures.append(
                f"{where}: the prebuilt now vendors {external} — {why}. It needs its own "
                f"notice in bundled.json, and ADR-0010's feature pin has moved."
            )

    if not failures:
        print(f"  {where}: key {key}, {len(members)} members, vendors {', '.join(sorted(seen))}")


# --- The fonts ----------------------------------------------------------------------


def allowlists():
    """ADR-0090's allowlist as the two files that carry it spell it: `about.toml`'s
    `accepted` and `deny.toml`'s `[licenses] allow`. Both, because a font's licence has to
    be permitted for the distributed binary and for the workspace alike."""
    with open(os.path.join(ROOT, "about.toml"), "rb") as f:
        about = set(tomllib.load(f)["accepted"])
    with open(os.path.join(ROOT, "deny.toml"), "rb") as f:
        deny = set(tomllib.load(f)["licenses"]["allow"])
    return {"about.toml": about, "deny.toml": deny}


def check_fonts(manifest, failures):
    """Each font the binary carries is present, is the attested file, has its notice beside
    it, and is under a licence ADR-0090's allowlist permits."""
    lists, before = allowlists(), len(failures)
    for font in manifest["fonts"]:
        path = os.path.join(ROOT, font["path"])
        if not os.path.exists(path):
            failures.append(
                f"{font['path']} is missing — the binary embeds it with `include_bytes!`, so "
                f"this checkout does not build either"
            )
        else:
            with open(path, "rb") as f:
                got = hashlib.sha256(f.read()).hexdigest()
            if got != font["sha256"]:
                failures.append(
                    f"{font['path']} hashes to {got}, not the attested {font['sha256']} — the "
                    f"notice in bundled.json was written for a different file"
                )
        if not os.path.exists(os.path.join(ROOT, font["notice"])):
            failures.append(f"{font['notice']}, {font['name']}'s licence text, is missing")
        for where, allowed in lists.items():
            if font["spdx"] not in allowed:
                failures.append(
                    f"{font['name']} is {font['spdx']}, which {where}'s allowlist does not "
                    f"permit (ADR-0090: widening it is an ADR amendment, not a config edit)"
                )
    if len(failures) == before:
        print(f"fonts: {', '.join(f['name'] for f in manifest['fonts'])} attested and allowed")


# --- The generated document ---------------------------------------------------------


def notice(filename):
    return read_notice(os.path.join(NOTICES, filename))


def read_notice(path):
    # `newline=""` throughout this script, on every read and every write of the generated
    # document: several upstream licence texts are CRLF (`generic-array`'s is), and Python's
    # default universal-newline translation would rewrite them on the way in but not on the
    # way out, so a file that had just been written would not compare equal to itself.
    with open(path, encoding="utf-8", newline="") as f:
        return f.read().rstrip("\n")


def rust_licenses():
    """`cargo about`'s view of the distributed binary's dependency graph, normalised into
    (spdx, text) -> crates. Ordering is imposed here rather than taken from the tool, so
    the generated file is byte-stable across runs and machines."""
    out = subprocess.run(
        [
            "cargo", "about", "generate",
            "--format", "json",
            "--manifest-path", BINARY_MANIFEST,
            "--config", os.path.join(ROOT, "about.toml"),
            "--fail",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if out.returncode != 0:
        sys.exit(
            "FAIL: `cargo about generate` failed. Install it with\n"
            "    cargo install --locked cargo-about --features cli\n\n" + out.stderr
        )
    data = json.loads(out.stdout)

    grouped = {}
    for entry in data["licenses"]:
        crates = sorted(
            {
                f"{u['crate']['name']} {u['crate']['version']}"
                for u in entry["used_by"]
                if u["crate"]["name"] not in OWN_CRATES
            }
        )
        if not crates:
            continue
        key = (entry["id"], entry["text"].rstrip("\n"))
        grouped.setdefault(key, set()).update(crates)

    by_id = {}
    for (spdx, text), crates in grouped.items():
        by_id.setdefault(spdx, []).append((sorted(crates), text))
    for spdx in by_id:
        by_id[spdx].sort(key=lambda pair: (pair[0][0], pair[1]))
    return dict(sorted(by_id.items()))


def render(manifest):
    L = []
    w = L.append

    w("<!-- Generated by `python3 ci/assert_third_party_attribution.py --write`. Do not edit by hand:")
    w("     CI regenerates this file and fails on any difference. To change what it says, change")
    w("     `about.toml`, `ci/third-party-notices/bundled.json`, or the notice texts beside that")
    w("     file. #359 is the ticket this came from. -->")
    w("")
    w("# Third-party notices")
    w("")
    w("Montagent is MIT-licensed (see `LICENSE`; the fixture media is carved out separately in")
    w("`LICENSE-MEDIA.md`). This file is the attribution owed for **other people's code that is")
    w("inside the `montagent` binary** — the one file in each release archive.")
    w("")
    w("Scope is deliberate. It is the binary's dependency graph over the six desktop targets")
    w("ADR-0064 commits to, not this workspace's: the golden-frame harness, the second rasterizer")
    w("arm and the independent decoders the tests measure against are all test-only, and no")
    w("release archive contains a line of them.")
    w("")
    w("## What is not in the binary")
    w("")
    w("- **FFmpeg.** Montagent spawns an `ffmpeg` the user already has, and never links or bundles")
    w("  one — ADR-0009, because `libx264` is GPL and linking `libavcodec` would make this binary a")
    w("  GPL combined work. Nothing in this file covers FFmpeg; its licence is between the user and")
    w("  whoever packaged their copy.")
    w("- **A project's fonts.** A vendored font belongs to the project that vendors it, gated and")
    w("  attested per ADR-0057. The one font that *is* in the binary is Montagent's own, below.")
    w("- **System libraries**, linked dynamically from the platform rather than distributed:")
    for lib in manifest["system_libraries"]:
        w(f"  - **{lib['name']}** — {lib['note']}")
    w("")
    w("---")
    w("")
    w("## The Skia prebuilt")
    w("")
    w("`skia-safe`'s manifest says `license = \"MIT\"`, and that is true of the Rust binding code")
    w("and true of nothing else. ADR-0010 accepted a C++ dependency on the strength of a")
    w("*prebuilt*, and what the build downloads is a static library containing Skia itself plus")
    w(f"{len(manifest['bundled'])} third-party C/C++ libraries Skia vendors. No manifest-reading tool reports any of")
    w("them, so this section is maintained against the archive: `bundled.json` records what is in")
    w("there, and `ci/assert_third_party_attribution.py` re-derives it from the member names.")
    w("")
    p = manifest["prebuilt"]
    w(f"Prebuilt: `{p['crate']} {p['crate_version']}`, tag `{p['tag']}`, key suffix `{p['key_suffix']}`,")
    w(f"built from Skia `{p['skia_revision']}`.")
    w("")
    w("### Skia")
    w("")
    skia = manifest["skia"]
    w(f"**{skia['name']}** — {skia['spdx']} — <{skia['url']}> at `{skia['revision']}`.")
    w("")
    for line in skia["covers"]:
        w(f"{line}")
    w("")
    w("```")
    w(notice(skia["notices"][0]))
    w("```")
    w("")
    w("### Vendored into the Skia prebuilt")
    w("")
    for entry in manifest["bundled"]:
        w(f"#### {entry['name']} — {entry['spdx']}")
        w("")
        w(f"<{entry['url']}> at `{entry['revision']}`.")
        w("")
        w(f"Why it is in the binary: {entry['why']}")
        w("")
        if "obligation" in entry:
            w(f"**Obligation:** {entry['obligation']}")
            w("")
        for filename in entry["notices"]:
            w(f"<details><summary><code>{filename}</code></summary>")
            w("")
            w("```")
            w(notice(filename))
            w("```")
            w("")
            w("</details>")
            w("")

    # The one condition in this file that is a sentence someone has to actually write down,
    # rather than a text to reproduce. IJG clause (2) asks for it in so many words.
    w("### Independent JPEG Group")
    w("")
    w("This software is based in part on the work of the Independent JPEG Group.")
    w("")
    w("---")
    w("")
    w("## Fonts")
    w("")
    w("Embedded in the binary with `include_bytes!`, so no manifest reports them. Each is checked")
    w("by `ci/assert_third_party_attribution.py` against the hash recorded in `bundled.json`.")
    w("")
    for font in manifest["fonts"]:
        w(f"### {font['name']} — {font['spdx']}")
        w("")
        w(f"<{font['url']}> at `{font['revision']}`, `{font['path']}`.")
        w("")
        w(f"Why it is in the binary: {font['why']}")
        w("")
        w("```")
        w(read_notice(os.path.join(ROOT, font["notice"])))
        w("```")
        w("")
    w("---")
    w("")
    w("## Rust crates")
    w("")
    by_id = rust_licenses()
    total = len({c for texts in by_id.values() for crates, _ in texts for c in crates})
    w(f"{total} crates, under {len(by_id)} licences: " + ", ".join(f"`{i}`" for i in by_id) + ".")
    w("")
    w("Generated by `cargo about` from `about.toml`. Where a crate ships no licence file in its")
    w("published tarball, `cargo about` falls back to the canonical text for the licence its")
    w("manifest declares; those are grouped together and marked below, because the fallback is a")
    w("template and not that crate's own notice.")
    w("")
    for spdx, texts in by_id.items():
        w(f"### {spdx}")
        w("")
        for crates, text in texts:
            fallback = "<year> <copyright holders>" in text or "[yyyy] [name of copyright owner]" in text
            w(", ".join(f"`{c}`" for c in crates))
            w("")
            if fallback:
                w("*No licence file in these crates' published tarballs; the canonical")
                w(f"`{spdx}` text follows.*")
                w("")
            w("```")
            w(text)
            w("```")
            w("")

    # LF, unconditionally. `.gitattributes` in this repository is `* text=auto eol=lf`, and
    # for a reason ADR-0041 makes load-bearing rather than stylistic — so a generated file
    # that contained CRLF would be normalised by git on its way in and would then never again
    # equal what this script produces, failing the drift check permanently instead of when a
    # dependency changes. Some upstream licence texts are CRLF (`generic-array`'s is), so this
    # is not hypothetical. Reproducing the notice is the obligation; its line endings are not.
    document = "\n".join(L).replace("\r\n", "\n").replace("\r", "\n")
    return document.rstrip("\n") + "\n"


# --- Entry point --------------------------------------------------------------------


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="regenerate THIRD-PARTY.md")
    parser.add_argument("--skip-prebuilt", action="store_true", help="do not check any Skia prebuilt")
    parser.add_argument("--prebuilt-only", action="store_true", help="only check the Skia prebuilt")
    parser.add_argument("--prebuilt", help="a specific unpacked skia-binaries directory")
    args = parser.parse_args()

    with open(MANIFEST, encoding="utf-8") as f:
        manifest = json.load(f)

    failures = []

    if not args.prebuilt_only:
        check_fonts(manifest, failures)
        generated = render(manifest)
        # Two copies of the same generated text: the root, for a reader of the repository or
        # the GitHub Release archive, and `crates/montagent/`'s, so the same attribution rides
        # along in the crates.io tarball `cargo install montagent` builds from (#366). One
        # `render()` call, checked against both paths, so they cannot drift from each other —
        # only from the dependency graph, the same way the root copy always has.
        for output in (OUTPUT, CRATE_COPY):
            where = os.path.relpath(output, ROOT)
            if args.write:
                with open(output, "w", encoding="utf-8", newline="") as f:
                    f.write(generated)
                print(f"wrote {where} ({len(generated.splitlines())} lines)")
            elif not os.path.exists(output):
                failures.append(f"{where} does not exist — run with --write")
            else:
                with open(output, encoding="utf-8", newline="") as f:
                    committed = f.read()
                if committed != generated:
                    failures.append(
                        f"{where} is not what the dependency graph generates. A dependency was "
                        f"added, removed, or changed licence. Re-run with --write and commit the "
                        f"result."
                    )
                else:
                    print(f"{where} matches the dependency graph")

    if not args.skip_prebuilt:
        prebuilts = [args.prebuilt] if args.prebuilt else find_prebuilts()
        if not prebuilts:
            failures.append(
                "no unpacked skia-binaries directory under target/ — build the workspace first, "
                "or pass --skip-prebuilt if this run is only meant to check for drift"
            )
        else:
            print(f"prebuilts found: {len(prebuilts)}")
            for prebuilt in prebuilts:
                check_prebuilt(manifest, prebuilt, failures)

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        sys.exit(1)

    print("third-party attribution holds")


if __name__ == "__main__":
    main()
