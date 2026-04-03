# Testing

## Local Checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Pre-commit Hook

The repository includes `.githooks/pre-commit` to run the standard local checks.

To enable it for your local clone:

```bash
./scripts/install-hooks.sh
```

## Current Test Coverage

- Config parsing
- Config lookup order
- Preferred editor resolution
- Known-editor detection
- JSON extraction from Ollama responses
- ANSI color styling
- Verbose request serialization coverage
- Recommended-command selection
- Destructive-command confirmation logic
- Benchmark duration conversion
