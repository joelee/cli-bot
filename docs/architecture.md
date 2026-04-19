# Architecture

## Overview

`cli-bot` has a small pipeline:

1. Parse CLI arguments.
2. Resolve the config path from `--config`, `${HOME}/.config/cli-bot/cli-bot.toml`, or `/etc/cli-bot.toml`.
3. Resolve the runtime environment profile, including OS, distro, and effective package manager.
4. Initialize session memory unless disabled for the current invocation.
5. If `--check` is used, validate config, Ollama reachability, environment resolution, model availability, and editor resolution.
6. Otherwise load recent session turns and render a bounded prompt context.
7. Send the user request to Ollama.
8. Parse the structured response into a command plan with an `unresolved` fallback flag.
9. If unresolved, make a second LLM request for a plain text response.
10. If a command plan is returned, ask the user to select a command when multiple options are returned.
11. Ask for confirmation when the command is potentially destructive.
12. Execute the final command through the configured shell.
13. Save a summarized turn back to session storage.

## Modules

- `src/main.rs`: process entrypoint and top-level error handling.
- `src/lib.rs`: application orchestration.
- `src/config.rs`: strongly typed configuration parsing.
- `src/llm.rs`: Ollama HTTP client and planner prompt generation.
- `src/planner.rs`: planner response model and destructive-command detection.
- `src/session.rs`: local session storage, scoping, naming, and prompt context rendering.
- `src/shell.rs`: command execution wrapper.

## Design Notes

- The LLM is asked to return JSON so downstream code stays deterministic.
- The primary planner returns command plans only, plus an `unresolved` flag when it should not guess a command.
- A second LLM pass produces a plain text response only when the command planner returns `unresolved = true`.
- Session memory is enabled by default, but can be disabled in config or per invocation with `--no-session`.
- The default session scope is `working_directory`, which keeps the same session name isolated per folder to avoid cross-project leakage.
- Session state is stored locally as JSON and only a bounded structured summary is added to prompts.
- Session startup can prune expired session files when `session_memory.retention_days` is configured.
- Captured command output is optional and stays out of prompts unless `session_memory.include_command_output_in_prompt = true`.
- Ollama transport is selectable: `use_chat_api = true` uses `/api/chat`, otherwise `cli-bot` uses `/api/generate`.
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
