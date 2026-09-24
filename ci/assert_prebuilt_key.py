#!/usr/bin/env python3
"""Assert that a cold Skia build resolved the pinned prebuilt, and compiled nothing.

ADR-0010 accepted a C++ dependency on a premise verified once: the required
prebuilt key -- CPU-only, no `ganesh`, no `gl` -- is published for every desktop
tier-1 target. The premise decays. A version bump, an added decode feature or an
upstream change to the published asset matrix can move or drop the key, and a
miss falls back to compiling Skia from source (LLVM/Clang, Python, Ninja), which
strangles onboarding for a project whose contributors build cold (#36).

This script is the assertion the scheduled canary rests on. It reads a
`cargo build -vv` log and is deliberately stricter than "the build succeeded":

  * the key's feature segment must equal the pinned set exactly, so a feature the
    manifest did not ask for cannot pass by sorting before the ones it did;
  * the target segment of the key must be the target the job claims to check, so
    a mis-wired matrix entry that silently checked the host is caught;
  * the download must have SUCCEEDED, not merely been attempted;
  * no line may say a source build started, or was refused.

The key is read from `[workspace.metadata.skia] prebuilt-key` in the root
`Cargo.toml` -- the one place the feature set is pinned. `skia_pin.rs` proves
that value is what the pinned feature list actually resolves to, so between the
two there is no second copy of the key anywhere.

Usage:
    assert_prebuilt_key.py --log build.log --target aarch64-apple-darwin
                           [--manifest Cargo.toml]

Exits 0 when the prebuilt resolved, 1 with a named defect otherwise.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

# `skia-bindings`' build script prints these. They are the observable surface of
# a decision it otherwise makes silently.
TRYING = re.compile(r"TRYING TO DOWNLOAD AND INSTALL SKIA BINARIES:\s*(?P<tag>[^/\s]+)/(?P<key>\S+)")
SUCCEEDED = "DOWNLOAD AND INSTALL SUCCEEDED"
FAILED = "DOWNLOAD AND INSTALL FAILED"

# Every way the log can say "the prebuilt was not used", with what each one
# means. The two `Refusing` lines come from `skia-safe`'s `no-compile` feature,
# which `montagent-render` turns on by default: they are a miss that was stopped
# before it became a forty-minute compile, which is a better outcome than the
# `STARTING` lines and is still a canary failure.
SOURCE_BUILD_MARKERS = {
    "STARTING A FULL BUILD": (
        "Skia is being compiled from source rather than fetched"
    ),
    "STARTING OFFLINE BUILD": (
        "Skia is being compiled from a vendored source tree rather than fetched"
    ),
    "STARTING BIND AGAINST SYSTEM SKIA": (
        "the bindings are being generated against a system Skia rather than the prebuilt"
    ),
    "Refusing to full-build skia": (
        "the prebuilt was missed and the `no-skia-source-build` feature stopped the "
        "from-source fallback before it started"
    ),
    "Refusing to offline-build skia": (
        "a vendored source build was attempted and `no-skia-source-build` stopped it"
    ),
}


def pinned_key(manifest: Path) -> str:
    with manifest.open("rb") as f:
        data = tomllib.load(f)
    try:
        return data["workspace"]["metadata"]["skia"]["prebuilt-key"]
    except KeyError as e:
        raise SystemExit(
            f"{manifest}: no [workspace.metadata.skia] prebuilt-key. That table is the "
            f"one place the Skia pin lives (#36); missing key {e}."
        )


def check(log_text: str, expected_key: str, target: str) -> list[str]:
    """Return a list of defects. Empty means the prebuilt resolved cleanly."""
    defects: list[str] = []

    for marker, meaning in SOURCE_BUILD_MARKERS.items():
        if marker in log_text:
            defects.append(
                f"the build log says {marker!r}: {meaning}. This is the revisit trigger "
                f"ADR-0010 states -- a sustained prebuilt miss on a tier-1 target points "
                f"at the `tiny-skia` exit."
            )

    if FAILED in log_text:
        for line in log_text.splitlines():
            if FAILED in line:
                defects.append(f"the prebuilt download failed: {line.strip()}")

    attempts = TRYING.findall(log_text)
    if not attempts:
        defects.append(
            "the build log contains no 'TRYING TO DOWNLOAD AND INSTALL SKIA BINARIES' line. "
            "Either the build was cached (the canary must run cold, with an empty CARGO_HOME "
            "and an empty target directory) or `skia-safe` was never built at all."
        )
        return defects

    keys = {key for _tag, key in attempts}
    if len(keys) > 1:
        defects.append(f"more than one prebuilt key was resolved in one build: {sorted(keys)}")

    for _tag, key in attempts:
        # key == "<rust-skia short hash>-<target>-<sorted feature set>". Split on
        # the target rather than matching a suffix: features sort alphabetically,
        # so an *added* feature can land before the pinned ones and a suffix test
        # would wave it through.
        marker = f"-{target}-"
        if marker not in key:
            defects.append(
                f"the resolved key {key!r} is not for {target!r}. The canary checks one "
                f"target per job; a key for another target means the job built for the host."
            )
            continue
        resolved = key.split(marker, 1)[1]
        if resolved != expected_key:
            defects.append(
                f"the resolved feature set is {resolved!r}, not the pinned {expected_key!r} "
                f"(full key {key!r}). The feature set is pinned in exactly one place and a "
                f"change to it is its own ticket that re-verifies all six targets (#189)."
            )

    if SUCCEEDED not in log_text:
        defects.append(
            "a download was attempted but the log never says DOWNLOAD AND INSTALL SUCCEEDED"
        )

    return defects


# --- The script's own check, asserting both directions ----------------------
#
# `docs/agents/domain.md` asks a claim-bearing script to fail loudly the moment
# it stops reproducing. This one's claim is that it can tell a resolved prebuilt
# from every way the resolution can go wrong, so it carries the negative cases as
# well as the positive one and CI runs `--self-test` on every PR.

TARGET = "aarch64-apple-darwin"


def clean_log(key: str, target: str = TARGET) -> str:
    """A build log for a cold build that resolved `key` for `target`."""
    full = f"1a2b3c4-{target}-{key}"
    return (
        "   Compiling skia-bindings v0.153.2\n"
        f"TRYING TO DOWNLOAD AND INSTALL SKIA BINARIES: 0.153.2/{full}\n"
        "  FROM: https://github.com/rust-skia/skia-binaries/releases/download/"
        f"0.153.2/skia-binaries-{full}.tar.gz\n"
        "UNPACKING ARCHIVE INTO: /tmp/target/release/build/skia-bindings-xxxx/out/skia\n"
        f"{SUCCEEDED}\n"
    )


def self_test(manifest: Path) -> int:
    # The fixtures are built from the pin rather than from a literal, so this
    # script has no second copy of the key to drift from.
    pinned = pinned_key(manifest)
    clean = clean_log(pinned)

    cases: list[tuple[str, str, bool]] = [
        ("a cold build that resolved the pinned prebuilt", clean, True),
        (
            "a source build, which is the failure #36 exists to catch",
            "STARTING A FULL BUILD\nHOST: aarch64-apple-darwin\n",
            False,
        ),
        ("an offline build against a vendored Skia checkout", "STARTING OFFLINE BUILD\n", False),
        (
            "a miss that `no-skia-source-build` stopped before it became a compile",
            "thread 'main' panicked: Refusing to full-build skia with no-compile feature\n",
            False,
        ),
        (
            "a key that gained a feature the manifest did not ask for",
            clean_log(pinned + "-textlayout"),
            False,
        ),
        (
            "a key that gained a feature sorting BEFORE the pinned ones",
            clean_log("ftembed-" + pinned),
            False,
        ),
        ("a key that lost a feature", clean_log(pinned.rsplit("-", 1)[0]), False),
        (
            "a GPU key, which is published but is not the one ADR-0010 pins",
            clean_log("ganesh-gl-" + pinned),
            False,
        ),
        (
            "a key resolved for another target, i.e. a mis-wired matrix entry",
            clean_log(pinned, "x86_64-apple-darwin"),
            False,
        ),
        (
            "a warm build, where nothing was downloaded because nothing was built",
            "   Compiling montagent-render v0.1.0\n    Finished `dev` profile\n",
            False,
        ),
        (
            "a download that was attempted and failed",
            clean.replace(SUCCEEDED, "DOWNLOAD AND INSTALL FAILED: 404 Not Found"),
            False,
        ),
        ("a download attempted but never confirmed", clean.replace(SUCCEEDED + "\n", ""), False),
    ]

    failures = 0
    for label, log, should_pass in cases:
        defects = check(log, pinned, TARGET)
        passed = not defects
        if passed != should_pass:
            failures += 1
            verdict = "accepted" if passed else "rejected"
            wanted = "accept" if should_pass else "reject"
            print(f"SELF-TEST FAILED: {verdict} {label}; expected it to {wanted}", file=sys.stderr)
            for d in defects:
                print(f"    {d}", file=sys.stderr)
        else:
            print(f"  ok  {'accepts' if should_pass else 'rejects'}: {label}")

    if failures:
        print(f"{failures} of {len(cases)} self-test cases failed", file=sys.stderr)
        return 1
    print(f"all {len(cases)} self-test cases hold, against the pinned key {pinned!r}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log", type=Path, help="a `cargo build -vv` log")
    parser.add_argument("--target", help="the Rust target triple this job built for")
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="check this script against known-good and known-bad logs, and exit",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path(__file__).resolve().parent.parent / "Cargo.toml",
        help="the workspace manifest holding the pin",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test(args.manifest)
    if not args.log or not args.target:
        parser.error("--log and --target are required unless --self-test is given")

    expected = pinned_key(args.manifest)
    text = args.log.read_text(encoding="utf-8", errors="replace")
    defects = check(text, expected, args.target)

    if defects:
        print(f"SKIA PREBUILT CANARY FAILED for {args.target}", file=sys.stderr)
        for d in defects:
            print(f"  - {d}", file=sys.stderr)
        return 1

    match = TRYING.search(text)
    print(f"skia prebuilt resolved for {args.target}: {match.group('tag')}/{match.group('key')}")
    print(f"feature set: {expected} (CPU-only, no ganesh, no gl -- ADR-0010)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
