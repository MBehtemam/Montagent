#!/usr/bin/env bash
# ci/check_release_honesty.sh
# Verifies tag version alignment, non-empty release notes/tag message,
# and checks that schema-affecting marketing claims in release notes
# correspond to actual engine code diffs.
set -euo pipefail

TAG="${1:-${GITHUB_REF_NAME:-}}"
if [ -z "$TAG" ]; then
    echo "Usage: ci/check_release_honesty.sh <tag_name>"
    exit 1
fi

echo "=== Release Honesty Gate: Checking $TAG ==="

# 1. Tag Version Alignment
CARGO_VERSION=$(grep -m1 '^version = ' Cargo.toml | tr -d '"' | awk '{print $3}')
EXPECTED_TAG="v${CARGO_VERSION}"
if [ "$TAG" != "$EXPECTED_TAG" ]; then
    echo "::error::Tag mismatch: git tag is '$TAG', but workspace version in Cargo.toml is '$CARGO_VERSION' (expected '$EXPECTED_TAG')"
    exit 1
fi
echo "✓ Tag matches Cargo.toml workspace version: $TAG"

# 2. Release Notes / Tag Annotation Presence
# Get tag annotation or commit message
TAG_BODY=$(git tag -l --format='%(contents)' "$TAG" 2>/dev/null || git log -1 --format=%B "$TAG" 2>/dev/null || echo "")
if [ ${#TAG_BODY} -lt 40 ]; then
    echo "::error::Release notes for $TAG are empty or too brief (${#TAG_BODY} chars). Expected detailed notes with highlights."
    exit 1
fi
echo "✓ Release notes presence verified (${#TAG_BODY} chars)"

# 3. Claim-to-Diff Verification
# Find previous tag to compute diff
PREV_TAG=$(git describe --tags --abbrev=0 "${TAG}^" 2>/dev/null || echo "")
if [ -n "$PREV_TAG" ]; then
    echo "Comparing against previous tag: $PREV_TAG"
    DIFF_STAT=$(git diff "${PREV_TAG}..${TAG}" -- crates/montagent-core/src/schema.rs crates/montagent-render/ || echo "")

    # Check for known deferred keywords:
    # If the release notes claim motion paths, new transition kinds, or Rec.709,
    # verify that actual schema/renderer additions exist in the diff.
    LOWER_BODY=$(echo "$TAG_BODY" | tr '[:upper:]' '[:lower:]')
    
    # Keyword checks (unless explicitly mentioned in an erratum or "not in" section)
    if echo "$LOWER_BODY" | grep -qiE "(motion path|path motion)" && ! echo "$LOWER_BODY" | grep -qiE "(not in|deferred|errat)"; then
        if ! echo "$DIFF_STAT" | grep -qiE "motion.*path"; then
            echo "::error::Release notes claim 'motion path', but no code diff detected in engine crates between $PREV_TAG and $TAG"
            exit 1
        fi
    fi

    if echo "$LOWER_BODY" | grep -qiE "(iris|radial_wipe|blinds)" && ! echo "$LOWER_BODY" | grep -qiE "(not in|deferred|errat)"; then
        if ! echo "$DIFF_STAT" | grep -qiE "(iris|radial_wipe|blinds)"; then
            echo "::error::Release notes claim unreleased transition kinds, but no code diff detected in engine crates between $PREV_TAG and $TAG"
            exit 1
        fi
    fi

    if echo "$LOWER_BODY" | grep -qiE "(rec\.?709|bt\.?709)" && ! echo "$LOWER_BODY" | grep -qiE "(not in|deferred|errat)"; then
        if ! echo "$DIFF_STAT" | grep -qiE "(rec709|bt709|rec_709)"; then
            echo "::error::Release notes claim 'Rec.709' colour management, but no code diff detected in engine crates between $PREV_TAG and $TAG"
            exit 1
        fi
    fi
    echo "✓ Claim-to-diff honesty verified against $PREV_TAG"
else
    echo "No previous tag found; skipping relative claim-to-diff check."
fi

echo "=== Release Honesty Gate Passed Successfully ==="
