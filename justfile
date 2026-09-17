# cli-bot task runner. Every quality gate here wraps the exact command
# mandated by AGENTS.md; the Git hook, pre-commit, and CI call these recipes.

set shell := ["bash", "-euo", "pipefail", "-c"]

# List available recipes
default:
    @just --list

# Install developer tooling: llvm-tools-preview, cargo-llvm-cov, cargo-nextest
setup:
    rustup component add llvm-tools-preview
    command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
    command -v cargo-nextest >/dev/null || cargo install --locked cargo-nextest

# Format all code in place
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Lint with clippy, warnings are errors
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check the syntax of every shell script and Git hook
scripts-check:
    for script in scripts/*.sh .githooks/*; do bash -n "$script"; done

# Run unit tests and the mocked-Ollama integration tests
test:
    cargo test --workspace --all-targets --all-features

# Run the tests with nextest and write target/test-results/unit-tests.xml
test-junit:
    cargo nextest run --workspace --all-targets --all-features

# Line coverage gate (>= 80%)
coverage:
    cargo llvm-cov --workspace --all-features --fail-under-lines 80 --summary-only

# Write the HTML coverage report to target/llvm-cov/html
coverage-html:
    cargo llvm-cov --workspace --all-features --html

# Write the LCOV coverage report to target/coverage/lcov.info
coverage-lcov:
    mkdir -p target/coverage
    cargo llvm-cov --workspace --all-features --lcov --output-path target/coverage/lcov.info

# Build from the lockfile
build:
    cargo build --workspace --all-features --locked

# Package and verify the crate for crates.io without uploading
package:
    cargo package --allow-dirty

# List the files that go into the crate
package-list:
    cargo package --list --allow-dirty

# Lint the GitHub Actions workflows, with a local actionlint or its image
lint-workflows:
    if command -v actionlint >/dev/null; then actionlint; else docker run --rm -v "$PWD:/repo" -w /repo rhysd/actionlint:1.7.12 -color; fi

# All mandated checks: format, lint, scripts, tests, coverage, build, package
check: fmt-check lint scripts-check test coverage build package

# Full CI pipeline: all checks, then the workflow lint
ci: check lint-workflows

# Run the checks before every commit: point Git at .githooks
install-hooks:
    git config core.hooksPath .githooks
    @echo "Git hooks installed at $PWD/.githooks"

# Run the CLI, e.g. `just run -n list files`
run *ARGS:
    cargo run -- {{ARGS}}

# Publish TAG (vX.Y.Z) to crates.io and update the Homebrew formula
release TAG:
    scripts/release.sh {{TAG}}
