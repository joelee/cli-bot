#!/usr/bin/env bash
# Fails unless a release tag is exactly `v` followed by the crate version
# in Cargo.toml and the release records for it are final: CHANGELOG.md has a
# `## vX.Y.Z - <UTC timestamp>` section, docs/release/vX.Y.Z.md exists, is
# not marked as a draft, and has only absolute links (the GitHub release
# page cannot resolve relative ones), and README.md has no pre-release
# wording. The release workflow runs it before building or publishing.
#
# Usage: scripts/check-release-tag.sh <tag> [path/to/Cargo.toml]
# The records are looked up next to the given Cargo.toml.
set -euo pipefail

tag="${1:-}"
manifest="${2:-Cargo.toml}"
if [ -z "$tag" ]; then
    echo "usage: check-release-tag.sh <tag> [path/to/Cargo.toml]" >&2
    exit 2
fi
root="$(cd "$(dirname "$manifest")" && pwd)"

# The first `version = "..."` inside [package].
version="$(awk '
    /^\[package\]/ { inside = 1; next }
    /^\[/ { inside = 0 }
    inside && /^version[ \t]*=/ {
        sub(/^version[ \t]*=[ \t]*"/, ""); sub(/".*$/, ""); print; exit
    }' "$manifest")"
if [ -z "$version" ]; then
    echo "error: no [package] version in $manifest" >&2
    exit 1
fi

if ! printf '%s\n' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "error: tag '$tag' is not of the form vMAJOR.MINOR.PATCH" >&2
    exit 1
fi
if [ "$tag" != "v$version" ]; then
    echo "error: tag $tag does not match the crate version $version" >&2
    exit 1
fi

# Every unfinished record is reported before giving up.
problems=0
fail() {
    echo "error: $*" >&2
    problems=1
}
if ! grep -Eq "^## ${tag//./\\.} - " "$root/CHANGELOG.md" 2>/dev/null; then
    fail "CHANGELOG.md has no \"## $tag - <UTC timestamp>\" section; rename Unreleased when finalising the release"
fi
notes="$root/docs/release/$tag.md"
if [ ! -f "$notes" ]; then
    fail "docs/release/$tag.md is missing"
else
    if grep -Eq '^Draft( |$)' "$notes"; then
        fail "docs/release/$tag.md is still marked as a draft; remove the draft line when finalising the release"
    fi
    relative="$(grep -oE '\]\([^)]+\)' "$notes" | grep -vE '^\]\((https?://|#|mailto:)' || true)"
    if [ -n "$relative" ]; then
        fail "docs/release/$tag.md has relative links, which the GitHub release page cannot resolve; use absolute URLs: ${relative//$'\n'/ }"
    fi
fi
for phrase in "being prepared" "not yet released"; do
    if grep -iq "$phrase" "$root/README.md" 2>/dev/null; then
        fail "README.md still says \"$phrase\"; remove pre-release wording when finalising the release"
    fi
done
if [ "$problems" -ne 0 ]; then
    exit 1
fi
echo "tag $tag matches the crate version and the release records are final"
