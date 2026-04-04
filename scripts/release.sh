#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
REPO_ROOT="$(readlink -f "$SCRIPT_DIR/..")"
ENV_FILE="$REPO_ROOT/.env"

if [[ -f "$ENV_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
fi

usage() {
  printf 'Usage: %s vX.Y.Z\n' "$0"
  printf 'Example: %s v0.1.2\n' "$0"
}

fail() {
  printf 'Error: %s\n' "$1" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

extract_cargo_version() {
  local version_line
  version_line="$(grep -m1 '^version = ' "$REPO_ROOT/Cargo.toml" || true)"
  [[ -n "$version_line" ]] || fail 'failed to read version from Cargo.toml'
  printf '%s\n' "$version_line" | cut -d '"' -f2
}

compute_sha256() {
  local file="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | cut -d ' ' -f1
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | cut -d ' ' -f1
    return
  fi

  fail 'required command not found: sha256sum or shasum'
}

compute_sha256_from_url() {
  local url="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    curl -fsSL "$url" | sha256sum | cut -d ' ' -f1
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    curl -fsSL "$url" | shasum -a 256 | cut -d ' ' -f1
    return
  fi

  fail 'required command not found: sha256sum or shasum'
}

wait_for_crate_url() {
  local url="$1"
  local attempt

  for attempt in $(seq 1 20); do
    if curl -fsI "$url" >/dev/null 2>&1; then
      return 0
    fi

    sleep 3
  done

  return 1
}

if [[ $# -ne 1 ]]; then
  usage
  exit 1
fi

TAG="$1"
[[ "$TAG" == v* ]] || fail 'release tag must start with `v`, for example `v0.1.2`'
VERSION="${TAG#v}"
[[ -n "$VERSION" ]] || fail 'release version must not be empty'

[[ -n "${HOMEBREW_FORMULA_FILE:-}" ]] || fail 'HOMEBREW_FORMULA_FILE is not set; export it or define it in .env'
[[ -f "$HOMEBREW_FORMULA_FILE" ]] || fail "Homebrew formula file does not exist: $HOMEBREW_FORMULA_FILE"

require_command cargo
require_command curl
require_command perl

CARGO_VERSION="$(extract_cargo_version)"
[[ "$CARGO_VERSION" == "$VERSION" ]] || fail "Cargo.toml version ($CARGO_VERSION) does not match requested release tag ($TAG)"

cd "$REPO_ROOT"

printf 'Running release checks for cli-bot %s\n' "$VERSION"
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo package

CRATE_FILE="$REPO_ROOT/target/package/cli-bot-$VERSION.crate"
[[ -f "$CRATE_FILE" ]] || fail "expected packaged crate not found: $CRATE_FILE"

LOCAL_CRATE_SHA256="$(compute_sha256 "$CRATE_FILE")"
CRATE_URL="https://static.crates.io/crates/cli-bot/cli-bot-$VERSION.crate"

printf 'Publishing cli-bot %s to crates.io\n' "$VERSION"
cargo publish

printf 'Waiting for published crate to become available: %s\n' "$CRATE_URL"
wait_for_crate_url "$CRATE_URL" || fail "published crate did not become available in time: $CRATE_URL"

printf 'Computing checksum from published crate artifact\n'
CRATE_SHA256="$(compute_sha256_from_url "$CRATE_URL")"

printf 'Updating Homebrew formula: %s\n' "$HOMEBREW_FORMULA_FILE"
NEW_URL="url \"$CRATE_URL\"" \
NEW_SHA="sha256 \"$CRATE_SHA256\"" \
perl -0pi -e 's{url "https://static\.crates\.io/crates/cli-bot/cli-bot-[^"]+\.crate"}{$ENV{NEW_URL}}g; s{sha256 "[0-9a-f]+"}{$ENV{NEW_SHA}}g' "$HOMEBREW_FORMULA_FILE"

printf '\nRelease complete.\n'
printf 'Published version: %s\n' "$VERSION"
printf 'Crate URL: %s\n' "$CRATE_URL"
printf 'Crate SHA256: %s\n' "$CRATE_SHA256"
printf 'Local package SHA256: %s\n' "$LOCAL_CRATE_SHA256"
printf 'Updated formula: %s\n' "$HOMEBREW_FORMULA_FILE"
printf '\nNext steps:\n'
printf '1. Review and commit the Homebrew formula update.\n'
printf '2. Push the Homebrew tap repository changes.\n'
