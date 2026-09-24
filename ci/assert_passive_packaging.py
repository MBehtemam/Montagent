#!/usr/bin/env python3
"""Re-derives #222's packaging invariants against the repository state itself, rather
than trusting prose. Exits non-zero if any of them stop holding.

    python3 ci/assert_passive_packaging.py

ADR-0064 settled the distribution story: `cargo install montagent` plus signed GitHub
Release tarballs, no Homebrew tap for v1, and *no update-check code, no telemetry, no
hardcoded "latest version" endpoint, ever*. ADR-0009 separately settled that FFmpeg is
user-supplied, never bundled, because linking it would make the shipped binary a GPL
combined work.

Three checks, each a fact about the committed tree that would silently stop being true
the moment someone adds the wrong dependency or file, not a design intent that erodes
without anyone noticing:

  1. no Homebrew formula lives in the repo (a formula is a pointer at a release tarball,
     and ADR-0064 defers the tap itself past v1);
  2. `Cargo.lock` names no crate from the update-check/telemetry/HTTP-client families —
     `montagent_core::media::probe::probe_remote`'s own doc comment is "the only function
     in Montagent that can cause a network call", and that claim is false the moment an
     HTTP client crate enters the dependency graph, self-reported network_attempts
     notwithstanding;
  3. `Cargo.lock` names no FFmpeg-linking crate (`ffmpeg-next`, `ffmpeg-sys*`,
     `rusty_ffmpeg`), which would make the *build* pull in the GPL library ADR-0009
     requires stay strictly on the user's own PATH.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..")
LOCKFILE = os.path.join(ROOT, "Cargo.lock")

# Crates whose mere presence in the dependency graph would mean Montagent itself is
# capable of reaching the network — the invariant `probe_remote`'s doc comment states as
# a fact about the source, checked here as a fact about the resolved graph.
NETWORK_CLIENT_CRATES = [
    "reqwest",
    "ureq",
    "hyper",
    "curl",
    "isahc",
    "surf",
    "attohttpc",
]

# Crates whose job is specifically an update check or usage telemetry — the "no update
# check of any kind, ever" half of ADR-0064.
UPDATE_OR_TELEMETRY_CRATES = [
    "self_update",
    "self-update",
    "update-informer",
    "cargo-dist",
    "sentry",
    "posthog",
    "segment",
    "mixpanel",
]

# Crates that link FFmpeg's own libraries into the binary, which is exactly the
# distribution ADR-0009 ruled out: FFmpeg is resolved from `PATH` at runtime, never
# linked or vendored.
FFMPEG_LINKING_CRATES = [
    "ffmpeg-next",
    "ffmpeg-sys",
    "ffmpeg-sys-next",
    "rusty_ffmpeg",
]

failures = []

# --- 1. No Homebrew tap -------------------------------------------------------------
# A real formula is named after the package, not after the word "formula" — Homebrew's
# own convention is `Formula/<name>.rb` (or a bare `<name>.rb` in a tap root) — so the
# thing that actually identifies one is its Ruby shape: `class Name < Formula`. Matching
# on that rather than on a filename catches the tap ADR-0064 calls "deferred, not
# rejected" wherever it lands, not only one it happens to be named.
FORMULA_CLASS = re.compile(r"(?m)^\s*class\s+\w+\s*<\s*Formula\b")
for dirpath, dirnames, filenames in os.walk(ROOT):
    dirnames[:] = [d for d in dirnames if d not in (".git", "target")]
    for filename in filenames:
        if not filename.endswith(".rb"):
            continue
        path = os.path.join(dirpath, filename)
        with open(path, encoding="utf-8", errors="replace") as f:
            if FORMULA_CLASS.search(f.read()):
                failures.append(f"a Homebrew formula exists: {os.path.relpath(path, ROOT)}")

# --- 2 & 3. The dependency graph names nothing that could phone home or link FFmpeg --
if not os.path.exists(LOCKFILE):
    failures.append(f"no Cargo.lock at {LOCKFILE} — run `cargo generate-lockfile` first")
else:
    with open(LOCKFILE, encoding="utf-8") as f:
        lock = f.read()
    package_names = set(re.findall(r'(?m)^name = "([^"]+)"', lock))

    def named(candidates):
        return sorted(name for name in package_names if any(name == c or name.startswith(c) for c in candidates))

    for name in named(NETWORK_CLIENT_CRATES):
        failures.append(f"Cargo.lock names an HTTP client crate ({name!r}); probe_remote's 'only function that can cause a network call' claim no longer holds")
    for name in named(UPDATE_OR_TELEMETRY_CRATES):
        failures.append(f"Cargo.lock names an update-check/telemetry crate ({name!r}); ADR-0064 requires none, ever")
    for name in named(FFMPEG_LINKING_CRATES):
        failures.append(f"Cargo.lock names an FFmpeg-linking crate ({name!r}); ADR-0009 requires FFmpeg stay resolved from PATH, never linked")

if failures:
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    sys.exit(1)

print("passive packaging holds: no Homebrew tap, no network/telemetry client, no linked FFmpeg")
