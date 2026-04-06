# Architecture

## Overview

`cli-bot` has a small pipeline:

1. Parse CLI arguments.
2. Resolve the config path from `--config`, `${HOME}/.config/cli-bot/cli-bot.toml`, or `/etc/cli-bot.toml`.
3. Resolve the runtime environment profile, including OS, distro, and effective package manager.
4. If `--check` is used, validate config, Ollama reachability, environment resolution, model availability, and editor resolution.
5. Otherwise send the user request to Ollama.
6. Parse the structured response into a command plan with an `unresolved` fallback flag.
7. If unresolved, make a second LLM request for a plain text response.
8. If a command plan is returned, ask the user to select a command when multiple options are returned.
9. Ask for confirmation when the command is potentially destructive.
10. Execute the final command through the configured shell.

## Modules

- `src/main.rs`: process entrypoint and top-level error handling.
- `src/lib.rs`: application orchestration.
- `src/config.rs`: strongly typed configuration parsing.
- `src/llm.rs`: Ollama HTTP client and planner prompt generation.
- `src/planner.rs`: planner response model and destructive-command detection.
- `src/shell.rs`: command execution wrapper.

## Design Notes

- The LLM is asked to return JSON so downstream code stays deterministic.
- The primary planner returns command plans only, plus an `unresolved` flag when it should not guess a command.
- A second LLM pass produces a plain text response only when the command planner returns `unresolved = true`.
- Destructive-command detection combines an LLM-provided `potentially_destructive` boolean and configured substring matching.
- The planner can mark one command as `recommended`, which the CLI may auto-select when configured to do so.
- The planner prompt includes a resolved environment profile so package-related requests use commands appropriate to the user's platform and package manager.
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
