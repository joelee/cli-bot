#!/usr/bin/env bash
# Decides whether the commit now on `main` should be released, and says why
# when it should not. The Release workflow runs this on every push to
# `main`; almost every push is an ordinary merge and releases nothing.
#
# A release happens only when both are true:
#   - the crate version has no tag yet, on the remote or locally;
#   - scripts/check-release-tag.sh passes, meaning CHANGELOG.md carries the
#     version heading, docs/release/<tag>.md exists without a draft line and
#     with only absolute links, and README.md has no pre-release wording.
#
# Prints `release=<true|false>` and `tag=<vX.Y.Z>`, and appends the same to
# $GITHUB_OUTPUT when that is set. Exits 0 either way: "nothing to release"
# is a normal outcome, not a failure.
#
# Usage: scripts/release-gate.sh [path/to/Cargo.toml]
set -euo pipefail

manifest="${1:-Cargo.toml}"
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
tag="v$version"

decide() {
    local release="$1" reason="$2"
    if [ "$release" = false ]; then
        # A GitHub notice, so the run says plainly why it did nothing.
        echo "::notice::no release: $reason"
    else
        echo "releasing $tag: $reason"
    fi
    printf 'release=%s\ntag=%s\n' "$release" "$tag"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then
        printf 'release=%s\ntag=%s\n' "$release" "$tag" >> "$GITHUB_OUTPUT"
    fi
    exit 0
}

# Asked of the repository this runs in, not of wherever the manifest sits.
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null 2>&1 ||
    git ls-remote --exit-code --tags origin "$tag" >/dev/null 2>&1; then
    decide false "$tag already exists"
fi

if ! "$root/scripts/check-release-tag.sh" "$tag" "$manifest" >/dev/null 2>&1; then
    decide false "the release records for $tag are not final"
fi

decide true "the release records are final and the tag is free"
