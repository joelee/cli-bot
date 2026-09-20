# cli-bot task runner. Every quality gate here wraps the exact commands
# mandated by AGENTS.md; the Git hook, pre-commit, and CI call these recipes.

set shell := ["bash", "-euo", "pipefail", "-c"]

# List available recipes
default:
    @just --list

# Install developer tooling: llvm-tools-preview, cargo-llvm-cov,
# cargo-nextest, cargo-deny, actionlint (needs Go)
setup:
    rustup component add llvm-tools-preview
    command -v cargo-llvm-cov >/dev/null || cargo install --locked cargo-llvm-cov
    command -v cargo-nextest >/dev/null || cargo install --locked cargo-nextest
    command -v cargo-deny >/dev/null || cargo install --locked cargo-deny
    command -v actionlint >/dev/null || ! command -v go >/dev/null || GOBIN="$HOME/.local/bin" go install github.com/rhysd/actionlint/cmd/actionlint@v1.7.12

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

# Check Markdown links: relative targets, headings, paths on main, and
# absolute links in the crate README (scripts/check-links.sh)
links:
    scripts/check-links.sh

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

# Supply-chain audit: advisories, licences, bans, and sources (deny.toml)
audit:
    cargo deny check

# Package and verify the crate for crates.io without uploading
publish-dry-run *ARGS:
    # Verification compiles the packaged crate as a registry dependency,
    # and cargo never rebuilds or re-unpacks a registry crate whose version
    # is unchanged, so clear earlier builds of the crate and the sources
    # unpacked from cargo's temporary registries (the `-<hash>` folders;
    # the crates.io cache is left alone) first.
    cargo clean -p cli-bot
    rm -rf "${CARGO_HOME:-$HOME/.cargo}"/registry/src/-*/cli-bot-*
    cargo publish --dry-run --locked {{ARGS}}
    # The unpacked packages in target/package are only needed during
    # verification, and rust-cache's cleanup fails on their tests/ folders.
    rm -rf target/package

# Lint the GitHub Actions workflows, with a local actionlint or its image
lint-workflows:
    if command -v actionlint >/dev/null; then actionlint; else docker run --rm -v "$PWD:/repo" -w /repo rhysd/actionlint:1.7.12 -color; fi

# All mandated checks: format, lint, scripts, links, tests, coverage, build
check: fmt-check lint scripts-check links test coverage build

# Full CI pipeline: all checks, the supply-chain audit, the publish dry
# run, and the workflow lint
ci: check audit publish-dry-run lint-workflows

# Run the checks before every commit: point Git at .githooks
install-hooks:
    git config core.hooksPath .githooks
    @echo "Git hooks installed at $PWD/.githooks"

# Run the CLI, e.g. `just run --check`
run *ARGS:
    cargo run -- {{ARGS}}
