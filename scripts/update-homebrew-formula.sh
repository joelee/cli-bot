#!/usr/bin/env bash
# Points the Homebrew tap's cli-bot formula at a published release: sets
# its `url` to that version's crate on crates.io and its `sha256` to the
# checksum crates.io records for it, and changes nothing else. Fails when
# the version is not on crates.io yet. Review, commit, and push the change
# in the tap afterwards.
#
# Usage: scripts/update-homebrew-formula.sh [--sha256 <hex>] vX.Y.Z [TAP_DIR]
# TAP_DIR defaults to ../homebrew-oss next to this repository. --sha256 uses
# the given checksum instead of asking crates.io, for tests.
set -euo pipefail

usage() {
    echo "usage: update-homebrew-formula.sh [--sha256 <hex>] vX.Y.Z [TAP_DIR]" >&2
    exit 2
}

sha256=""
if [ "${1:-}" = "--sha256" ]; then
    [ $# -ge 2 ] || usage
    sha256="$2"
    shift 2
fi
tag="${1:-}"
[ -n "$tag" ] || usage
repo="$(cd "$(dirname "$0")/.." && pwd)"
tap="${2:-$repo/../homebrew-oss}"

if ! printf '%s\n' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "error: version '$tag' is not of the form vMAJOR.MINOR.PATCH" >&2
    exit 1
fi
version="${tag#v}"
if [ ! -d "$tap" ]; then
    echo "error: the tap directory $tap does not exist" >&2
    exit 1
fi
formula="$tap/Formula/cli-bot.rb"
if [ ! -f "$formula" ]; then
    echo "error: $formula does not exist" >&2
    exit 1
fi
if [ "$(grep -cE '^  url "' "$formula")" -ne 1 ] || [ "$(grep -cE '^  sha256 "' "$formula")" -ne 1 ]; then
    echo "error: $formula needs exactly one top-level url line and one sha256 line" >&2
    exit 1
fi

if [ -z "$sha256" ]; then
    api="https://crates.io/api/v1/crates/cli-bot/$version"
    if ! meta="$(curl -fsS -A 'cli-bot update-homebrew-formula.sh (https://github.com/joelee/cli-bot)' "$api")"; then
        echo "error: cli-bot $version is not published on crates.io yet ($api)" >&2
        exit 1
    fi
    sha256="$(printf '%s' "$meta" | grep -oE '"checksum": *"[0-9a-f]{64}"' | grep -oE '[0-9a-f]{64}' | head -n 1 || true)"
fi
if ! printf '%s\n' "$sha256" | grep -Eq '^[0-9a-f]{64}$'; then
    echo "error: '$sha256' is not a SHA-256 checksum" >&2
    exit 1
fi

url="https://static.crates.io/crates/cli-bot/cli-bot-$version.crate"
updated="$(sed -E \
    -e "s|^  url \".*\"$|  url \"$url\"|" \
    -e "s|^  sha256 \".*\"$|  sha256 \"$sha256\"|" \
    "$formula")"
# Written in place, so the file keeps its mode.
printf '%s\n' "$updated" > "$formula"
echo "Formula/cli-bot.rb now installs cli-bot $version (sha256 $sha256)"
