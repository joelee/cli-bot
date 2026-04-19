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

export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/unit-%p-%m.profraw"

printf 'Running unit tests with coverage instrumentation\n'
cargo test --lib

printf 'Merging profile data\n'
"$LLVM_PROFDATA_BIN" merge -sparse target/coverage/unit-*.profraw -o target/coverage/unit-tests.profdata

BIN_PATH=''
while IFS= read -r candidate; do
  case "$candidate" in
    *.d|*.rlib|*.rmeta)
      continue
      ;;
  esac
  BIN_PATH="$candidate"
  break
done < <(ls -1t target/debug/deps/cli_bot-*)

if [[ -z "$BIN_PATH" ]]; then
  printf 'Error: could not locate compiled unit test binary under target/debug/deps\n' >&2
  exit 1
fi

printf 'Generating text coverage report\n'
"$LLVM_COV_BIN" report "$BIN_PATH" \
  --instr-profile=target/coverage/unit-tests.profdata \
  --ignore-filename-regex='/(\.cargo|rustc)/' \
  | tee target/coverage/unit-report.txt

printf 'Generating HTML coverage report\n'
"$LLVM_COV_BIN" show "$BIN_PATH" \
  --instr-profile=target/coverage/unit-tests.profdata \
  --format=html \
  --output-dir=target/coverage/html \
  --ignore-filename-regex='/(\.cargo|rustc)/'

printf 'Generating LCOV coverage report\n'
"$LLVM_COV_BIN" export "$BIN_PATH" \
  --instr-profile=target/coverage/unit-tests.profdata \
  --format=lcov \
  --ignore-filename-regex='/(\.cargo|rustc)/' \
  > target/coverage/lcov.info

printf 'Coverage HTML written to %s\n' "$REPO_ROOT/target/coverage/html/index.html"
