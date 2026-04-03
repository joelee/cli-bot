# Architecture

## Overview

`cli-bot` has a small pipeline:

1. Parse CLI arguments.
2. Resolve the config path from `--config`, `${HOME}/.config/cli-bot/cli-bot.toml`, or `/etc/cli-bot.toml`.
3. If `--check` is used, validate config, Ollama reachability, model availability, and editor resolution.
4. Otherwise send the user request to Ollama.
5. Parse the structured response into a command plan.
6. Ask the user to select a command when multiple options are returned.
7. Ask for confirmation when the command is potentially destructive.
8. Execute the final command through the configured shell.

## Modules

- `src/main.rs`: process entrypoint and top-level error handling.
- `src/lib.rs`: application orchestration.
- `src/config.rs`: strongly typed configuration parsing.
- `src/llm.rs`: Ollama HTTP client and planner prompt generation.
- `src/planner.rs`: planner response model and destructive-command detection.
- `src/shell.rs`: command execution wrapper.

## Design Notes

- The LLM is asked to return JSON so downstream code stays deterministic.
- Destructive-command detection combines an LLM-provided `potentially_destructive` boolean and configured substring matching.
- The planner can mark one command as `recommended`, which the CLI may auto-select when configured to do so.
- The planner prompt includes the resolved preferred editor from config or `$EDITOR` so edit requests can use the user's normal tool.
- The configured Ollama model can be overridden per invocation with `--model` before health checks or planning run.
- Human-facing status output is styled through a small output module with `--color auto|always|never` control.
- Verbose mode surfaces request/response details from the Ollama boundary to debug malformed model output.
- Interactive prompts use `dialoguer` for simple terminal UX.
- The shell command is executed through configurable shell settings so the runtime stays portable.
- Benchmark mode measures planner, execution, and total elapsed time for model comparison.

## Follow-up Enhancements

- Add a non-interactive mode for CI usage.
- Add integration tests against a mocked Ollama endpoint.
- Support sequential command plans separately from alternative command choices.
