#!/usr/bin/env python3
"""Re-derives that every published crate's tarball ships the licence text it claims (#366).

    python3 ci/assert_crate_licenses.py --write   # copy the root LICENSE into each crate
    python3 ci/assert_crate_licenses.py           # check the copies are still current

`license.workspace = true` puts `license = "MIT"` in all four manifests, and crates.io
enforces that field — but the text it names lived only at the repository root
(`/LICENSE`), outside every package directory `cargo package` walks. #361 found the four
tarballs shipped none of it: `license = "MIT"` was a claim with nothing behind it.

`license-file` was tried first, since it is the field built for exactly this, and pointing
it outside the crate directory does resolve and copy the text in (confirmed: `cargo package
--list` includes it, and the packaged file is byte-identical to the root). It was dropped
because setting `license` and `license-file` together prints `warning: only one of
"license" or "license-file" is necessary` on every build once the licence is a standard SPDX
expression — which MIT is — and that warning would recur on every `cargo package`, publish
dry-run, and CI run forever. `license-file` is for a licence that is *not* expressible as an
SPDX identifier; MIT already is one, so this is the situation Cargo's own message is warning
against, not a false positive to silence.

So the text ships as an ordinary in-tree file instead: `crates/<name>/LICENSE`, a physical
copy, checked against the root rather than generated from anything (there is no dependency
graph to derive it from — it is just Montagent's own licence, repeated). This script is that
check, run the same way `ci/assert_third_party_attribution.py` checks its own copies.
"""

import argparse
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, ".."))
ROOT_LICENSE = os.path.join(ROOT, "LICENSE")

CRATES = ["montagent", "montagent-core", "montagent-render", "montagent-text"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="copy the root LICENSE into each crate")
    args = parser.parse_args()

    with open(ROOT_LICENSE, "rb") as f:
        canonical = f.read()

    failures = []
    for crate in CRATES:
        path = os.path.join(ROOT, "crates", crate, "LICENSE")
        where = os.path.relpath(path, ROOT)
        if args.write:
            with open(path, "wb") as f:
                f.write(canonical)
            print(f"wrote {where}")
            continue

        if not os.path.exists(path):
            failures.append(f"{where} does not exist — run with --write")
            continue

        with open(path, "rb") as f:
            got = f.read()
        if got != canonical:
            failures.append(f"{where} is not byte-identical to the root LICENSE — run with --write")
        else:
            print(f"{where} matches the root LICENSE")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        sys.exit(1)

    if not args.write:
        print("every crate ships the licence text it claims")


if __name__ == "__main__":
    main()
