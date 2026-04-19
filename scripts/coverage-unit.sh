#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
REPO_ROOT="$(readlink -f "$SCRIPT_DIR/..")"

cd "$REPO_ROOT"

require_command() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'Error: required command not found: %s\n' "$1" >&2
    exit 1
  }
}

require_command cargo
require_command rustc

resolve_llvm_tool() {
  local tool_name="$1"

  if command -v "$tool_name" >/dev/null 2>&1; then
    command -v "$tool_name"
    return
  fi

  local host_triple
  host_triple="$(rustc -vV | grep '^host: ' | cut -d ' ' -f2)"
  local candidate
  candidate="$(rustc --print sysroot)/lib/rustlib/$host_triple/bin/$tool_name"

  if [[ -x "$candidate" ]]; then
    printf '%s\n' "$candidate"
    return
  fi

  printf 'Error: required LLVM tool not found: %s\n' "$tool_name" >&2
  printf 'Install it on PATH or add the Rust component: rustup component add llvm-tools-preview\n' >&2
  exit 1
}

LLVM_PROFDATA_BIN="$(resolve_llvm_tool llvm-profdata)"
LLVM_COV_BIN="$(resolve_llvm_tool llvm-cov)"

mkdir -p target/coverage
rm -f target/coverage/unit-*.profraw target/coverage/unit-tests.profdata
rm -rf target/coverage/html
rm -f target/coverage/unit-report.txt
rm -f target/coverage/lcov.info
rm -f target/coverage/lib-binaries.json target/coverage/mock-ollama-binaries.json

export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/unit-%p-%m.profraw"

printf 'Capturing exact test binaries for coverage export\n'
cargo test --lib --no-run --message-format=json > target/coverage/lib-binaries.json
cargo test --test mock_ollama --no-run --message-format=json > target/coverage/mock-ollama-binaries.json

printf 'Running unit and mocked integration tests with coverage instrumentation\n'
cargo test --lib
cargo test --test mock_ollama

printf 'Merging profile data\n'
"$LLVM_PROFDATA_BIN" merge -sparse target/coverage/unit-*.profraw -o target/coverage/unit-tests.profdata

BIN_PATHS=()
while IFS= read -r candidate; do
  BIN_PATHS+=("$candidate")
done < <(
  grep -h -oE '"executable":"[^"]+"' \
    target/coverage/lib-binaries.json \
    target/coverage/mock-ollama-binaries.json \
    | cut -d'"' -f4
)

if [[ ${#BIN_PATHS[@]} -eq 0 ]]; then
  printf 'Error: could not locate compiled test binaries from Cargo JSON output\n' >&2
  exit 1
fi

printf 'Generating text coverage report\n'
"$LLVM_COV_BIN" report "${BIN_PATHS[@]}" \
  --instr-profile=target/coverage/unit-tests.profdata \
  --ignore-filename-regex='/(\.cargo|rustc)/' \
  | tee target/coverage/unit-report.txt

printf 'Generating HTML coverage report\n'
"$LLVM_COV_BIN" show "${BIN_PATHS[@]}" \
  --instr-profile=target/coverage/unit-tests.profdata \
  --format=html \
  --output-dir=target/coverage/html \
  --ignore-filename-regex='/(\.cargo|rustc)/'

printf 'Generating LCOV coverage report\n'
"$LLVM_COV_BIN" export "${BIN_PATHS[@]}" \
  --instr-profile=target/coverage/unit-tests.profdata \
  --format=lcov \
  --ignore-filename-regex='/(\.cargo|rustc)/' \
  > target/coverage/lcov.info

printf 'Coverage HTML written to %s\n' "$REPO_ROOT/target/coverage/html/index.html"
