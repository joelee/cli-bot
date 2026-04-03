# AGENTS.md

## Purpose

This repository contains `cli-bot`, a Rust CLI that converts natural-language requests into shell commands through Ollama.

## Working Rules

- Keep the implementation config-driven through `cli-bot.toml`.
- Preserve the safety model: risky commands require explicit approval before execution.
- When multiple commands are returned, keep the interactive selection flow intact.
- Prefer small, local changes over broad refactors.
- Update docs in `docs/` when behavior, architecture, or configuration changes.

## Verification

Run these before considering work complete:

```bash
./scripts/verify.sh
```

## Project Layout

- `src/config.rs`: TOML config loading
- `src/llm.rs`: Ollama request/response handling
- `src/planner.rs`: structured command plan types and safety checks
- `src/shell.rs`: shell execution
- `docs/`: project documentation
- `.githooks/pre-commit`: local pre-commit verification script
- `scripts/verify.sh`: shared verification entrypoint for local checks and hooks
