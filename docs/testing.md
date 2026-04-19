# Testing

## Local Checks

```bash
./scripts/verify.sh
```

This runs:

- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- `bash -n` against repository shell scripts and hooks
- `cargo package --allow-dirty`

## Run Unit Tests Manually

```bash
cargo test --lib
```

This runs the unit tests embedded under `src/`.

## Run Integration Tests Manually

```bash
cargo test --test mock_ollama
```

This runs the mocked end-to-end Ollama integration tests under `tests/`.

To run all unit and integration tests together:

```bash
cargo test --all-targets --all-features
```

## Manual Coverage

This repository includes a helper script for unit-test coverage.

Run it with:

```bash
bash ./scripts/coverage-unit.sh
```

This script:

- runs `cargo test --lib` with LLVM coverage instrumentation
- merges `.profraw` files into `target/coverage/unit-tests.profdata`
- prints a text coverage summary
- writes an HTML report to `target/coverage/html/index.html`
- writes an LCOV report to `target/coverage/lcov.info`

If you want to run the coverage commands manually, use the LLVM flow below.

Example:

```bash
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/unit-%p-%m.profraw"
cargo test --lib
profdata_bin="$(which llvm-profdata)"
cov_bin="$(which llvm-cov)"
"$profdata_bin" merge -sparse target/coverage/unit-*.profraw -o target/coverage/unit-tests.profdata
"$cov_bin" report \
  target/debug/deps/cli_bot-* \
  --instr-profile=target/coverage/unit-tests.profdata \
  --ignore-filename-regex='/(\.cargo|rustc)/'
```

You can also export HTML coverage with:

```bash
"$cov_bin" show \
  target/debug/deps/cli_bot-* \
  --instr-profile=target/coverage/unit-tests.profdata \
  --format=html \
  --output-dir=target/coverage/html \
  --ignore-filename-regex='/(\.cargo|rustc)/'
```

## Coverage In CI

The repository now includes a dedicated GitHub Actions workflow at `.github/workflows/unit-coverage.yml`.

It:

- installs Rust with `llvm-tools-preview`
- installs `cargo-nextest` for stable JUnit XML export
- runs `bash ./scripts/coverage-unit.sh`
- runs `cargo nextest run --lib` with JUnit output configured in `.config/nextest.toml`
- uploads the HTML coverage output as a workflow artifact
- uploads `target/coverage/lcov.info` to Codecov
- uploads unit test results to Codecov with `codecov/test-results-action@v1`

If you want to run the same coverage step in another workflow, use:

```yaml
- name: Run unit coverage
  run: bash ./scripts/coverage-unit.sh
```

The uploaded artifact currently contains:

- `target/coverage/html/`
- `target/coverage/unit-report.txt`
- `target/coverage/unit-tests.profdata`
- `target/coverage/lcov.info`
- `target/test-results/unit-tests.xml`

The repository also includes `.github/workflows/coverage-pages.yml`, which publishes the generated HTML coverage site to GitHub Pages on pushes to `main`.

Before the first successful Pages deploy, enable GitHub Pages in the repository settings:

1. Open `Settings -> Pages`
2. Under the build source, choose `GitHub Actions`
3. Keep the workflow-based deployment enabled for the repository

Expected published URL:

```text
https://joelee.github.io/cli-bot/
```

## Future Publishing

Coverage is currently exposed through the GitHub Actions artifact, which is a good low-friction first step.

Later publishing options include:

- keep the HTML report on GitHub Pages for stable public browsing
- use Codecov for commit and PR diff summaries
- keep README badges pointing to stable hosted coverage URLs

## Pre-commit Hook

The repository includes `.githooks/pre-commit`, which runs `./scripts/verify.sh` before each commit.

To enable it for your local clone:

```bash
./scripts/install-hooks.sh
```

## pre-commit Framework

The repository also includes `.pre-commit-config.yaml` for developers who use the `pre-commit` framework.

Install and enable it with:

```bash
pre-commit install
```

Run it manually across the whole repository with:

```bash
pre-commit run --all-files
```

The configured hook runs:

```bash
./scripts/verify.sh
```

## Current Test Coverage

- Config parsing
- Config lookup order
- Environment resolution and package manager detection
- Unresolved-plan validation and text-response fallback handling
- Preferred editor resolution
- Known-editor detection
- JSON extraction from Ollama responses
- ANSI color styling
- Verbose request serialization coverage
- Recommended-command selection
- Destructive-command confirmation logic
- Benchmark duration conversion
- Session name validation and JSON session storage
- Session prompt-context rendering and output truncation
- Mocked Ollama planner coverage for both `/api/generate` and `/api/chat`
- Mocked unresolved text fallback coverage for both backends
- Mocked `--check` coverage for Ollama version and tags endpoints
- Mocked session command coverage for `--session-list`, `--session-show`, and retention pruning
