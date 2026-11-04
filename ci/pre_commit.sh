#!/usr/bin/env bash
# Fast deterministic checks from `.github/workflows/ci.yml` (`committed checks` + Clippy).
# Wired into `.git/hooks/pre-commit` so regressions cannot land while the 5-OS matrix
# in `ci.yml` is set to `workflow_dispatch`.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

echo "==> [1/5] cargo fmt --all --check"
cargo fmt --all --check

echo "==> [2/5] docs/adr/check_amendment_banners.py"
python3 docs/adr/check_amendment_banners.py

echo "==> [3/5] docs/adr/predicate_reserved_names_scan.py"
python3 docs/adr/predicate_reserved_names_scan.py

echo "==> [4/5] ci/skills_workarounds.py --lint"
python3 ci/skills_workarounds.py --lint

echo "==> [5/5] cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings
