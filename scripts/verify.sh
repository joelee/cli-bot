#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
REPO_ROOT="$(readlink -f "$SCRIPT_DIR/..")"

cd "$REPO_ROOT"

cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash -n ".githooks/pre-commit" "scripts/install-hooks.sh" "scripts/release.sh" "scripts/verify.sh"
cargo package --allow-dirty
