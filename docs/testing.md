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
