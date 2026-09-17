# Testing

## Local Checks

```bash
just check
```

This runs, in order:

- `just fmt-check`: `cargo fmt --all -- --check`
- `just lint`: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `just scripts-check`: `bash -n` against repository shell scripts and hooks
- `just test`: `cargo test --workspace --all-targets --all-features`
- `just coverage`: the 80% line-coverage gate
- `just build`: `cargo build --workspace --all-features --locked`
- `just package`: `cargo package --allow-dirty`

See the [Developer Guide](developer-guide.md) for setup and every recipe.

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
just test
```

## Coverage

Coverage comes from [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov); `just setup` installs it. Line coverage must stay at or above 80%.

```bash
just coverage       # summary table; fails below 80% line coverage
just coverage-html  # target/llvm-cov/html/index.html
just coverage-lcov  # target/coverage/lcov.info
```

All three run the unit tests and the mocked-Ollama integration tests with coverage instrumentation. Files under `tests/` are not counted.

Code that needs a terminal is not covered: the `dialoguer` selection and confirmation prompts, the interactive loop, and `src/main.rs`.

## Coverage In CI

`.github/workflows/release-checks.yml` runs `just check` on every pull request, so the coverage gate is enforced there.

`.github/workflows/unit-coverage.yml` publishes the reports. It:

- installs the pinned Rust toolchain with `llvm-tools-preview`, plus `just`, `cargo-llvm-cov`, and `cargo-nextest`
- runs `just coverage-lcov` and `just coverage-html`
- runs `just test-junit`, which writes JUnit XML to the path configured in `.config/nextest.toml`
- uploads the HTML coverage output as a workflow artifact
- uploads `target/coverage/lcov.info` to Codecov
- uploads test results to Codecov with `codecov/test-results-action@v1`

The uploaded artifact contains:

- `target/llvm-cov/html/`
- `target/coverage/lcov.info`
- `target/test-results/unit-tests.xml`

`.github/workflows/coverage-pages.yml` publishes the HTML coverage site to GitHub Pages on pushes to `main`.

Before the first successful Pages deploy, enable GitHub Pages in the repository settings:

1. Open `Settings -> Pages`
2. Under the build source, choose `GitHub Actions`
3. Keep the workflow-based deployment enabled for the repository

Expected published URL:

```text
https://joelee.github.io/cli-bot/
```

## Pre-commit Hook

The repository includes `.githooks/pre-commit`, which runs `just check` before each commit.

To enable it for your local clone:

```bash
just install-hooks
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
just check
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
- Mocked session command coverage for `--session-list`, `--session-show`, `--session-clear`, named sessions, and retention pruning
- Mocked command execution, dry runs, and unresolved text responses saved as session turns
- Mocked follow-up requests that carry earlier turns, exit status, and captured output as session context
- Mocked `--auto-select-best`, `--model`, `--verbose`, `--benchmark`, and `--print-plan` flows
- Mocked planner failures: no JSON, invalid plan JSON, empty command list, HTTP error status, undecodable body
- Mocked `--check` failures: missing model, failing service, missing editor
- Mocked `--models-benchmark` report with a failing model, a fallback, and a trailing report path
