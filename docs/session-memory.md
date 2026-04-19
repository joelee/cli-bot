# Session Memory

This document describes the shipped Multi-turn Session Memory feature in `cli-bot` `0.3.0`, plus the design choices behind it.

## Problem

`cli-bot` is currently stateless across invocations. That works well for one-shot requests, but it breaks follow-up prompts such as:

- `install btop`
- `now remove it`
- `show the config file again`
- `open that in my editor`

The planner has no local history to resolve `it`, `that`, `again`, or `the first one` safely.

## Goals

- Keep short, local context across follow-up requests in an explicitly selected session.
- Improve command planning for references to the previous request, selected command, and recent text response.
- Keep shell-command generation as the primary behavior.
- Preserve the current safety model, including approval for risky commands and interactive selection for multiple commands.
- Keep the feature small, inspectable, and config-driven.

## Non-goals

- Do not turn `cli-bot` into a general chatbot with open-ended long-term memory.
- Do not silently reuse old command output as if it were still true.
- Do not sync history across machines or services.
- Do not infer hidden shell state beyond what `cli-bot` explicitly recorded.
- Do not lower confirmation requirements for risky commands.

## Current Scope

`cli-bot` now ships a named local session model that persists a short turn history across invocations.

Example:

```bash
cli-bot --session work "find my git config file"
cli-bot --session work "open it in nvim"
cli-bot --session work "show it again"
```

This is a better first step than a full REPL mode because it fits the current one-request-per-invocation architecture.

## Current UX

Current flags:

- `--session [name]`: load and update a named local session, or use the configured default name when no name is passed
- `--session-list`: list stored sessions and exit
- `--session-show`: print the stored turns for the selected or default session and exit
- `--session-clear`: delete the stored turns for the selected or default session and exit
- `--no-session`: disable session memory for the current invocation

Current behavior:

- Session memory is enabled by default when `session_memory.enabled = true`.
- If `--session` is omitted, `cli-bot` uses `session_memory.default_name`.
- `--session-show` and `--session-clear` operate on the selected or default session.
- Session names are restricted to simple filesystem-safe identifiers.
- `working_directory` scope is the default to avoid cross-project leakage from a single global `default` thread.

Examples:

```bash
cli-bot --session release "find the changelog entry for 0.2.2"
cli-bot --session release "open it in my editor"
cli-bot --session-list
cli-bot --session release --session-show
cli-bot --session release --session-clear
```

## Current Config

Keep the config small and bounded.

```toml
[session_memory]
enabled = true
default_name = "default"
scope = "working_directory"
storage_dir = "auto"
max_turns = 6
include_working_directory = true
save_text_responses = true
save_selected_commands = true
capture_command_output = false
include_command_output_in_prompt = false
max_output_bytes = 8192
retention_days = 14
```

Current meanings:

- `enabled`: global feature gate
- `default_name`: default session name used when `--session` is omitted
- `scope`: `working_directory` or `global`
- `storage_dir`: platform-appropriate local state directory when `auto`
- `max_turns`: only include the most recent turns in prompts
- `save_text_responses`: keep unresolved fallback answers for later references like `show that again`
- `save_selected_commands`: keep the final chosen command and rationale
- `include_working_directory`: record the cwd for each turn because references are often path-sensitive
- `capture_command_output`: capture command stdout and stderr after execution
- `include_command_output_in_prompt`: replay captured stdout and stderr into later prompts
- `max_output_bytes`: hard limit for stored or prompt-included command output
- `retention_days`: prune old session files automatically on startup

Default recommendation remains conservative: command output capture is off by default, and prompt inclusion is also off by default.

## Data To Store

Each turn should be summarized into a small record instead of saving raw prompts and raw model output forever.

Suggested fields:

- timestamp
- original request text
- working directory
- planner summary
- unresolved flag
- returned command choices
- selected command
- selected command rationale
- whether confirmation was required
- execution status code, when executed
- text response, when unresolved fallback was used

Recommendation: do not store full Ollama request or response payloads in session memory.

## Storage Model

Current module:

- `src/session.rs`

Suggested types:

- `SessionMemoryConfig`
- `SessionStore`
- `SessionRecord`
- `SessionTurn`
- `SessionTurnKind`

Current on-disk format:

- one JSON file per session
- human-inspectable and easy to prune

Example path shape:

- Linux: `${XDG_STATE_HOME}/cli-bot/sessions/<name>.json`
- macOS: `${HOME}/Library/Application Support/cli-bot/sessions/<name>.json`

If a platform-specific state directory helper is already introduced elsewhere later, reuse it instead of duplicating path logic.

## Planner Integration

The planner does not receive the entire transcript. It receives a short structured context block built from the most recent turns.

Suggested prompt addition:

```text
Session context is provided below. Use it only to resolve short follow-up references such as "it", "that", "again", or "the first one". If the reference is still ambiguous, set unresolved=true instead of guessing.

Recent session turns:
1. request: find my git config file
   selected_command: fd gitconfig ~
   result: executed successfully
2. request: open it in nvim
   selected_command: nvim ~/.gitconfig
   result: not executed yet
```

Recommended prompt rules:

- session context is advisory, not authoritative
- do not invent current system state from prior turns
- do not assume a previous command succeeded unless the stored turn says so
- if a follow-up could refer to multiple prior items, mark the request unresolved

## Application Flow Changes

Today the flow is:

1. Parse CLI args.
2. Load config.
3. Resolve environment.
4. Plan commands.
5. Fallback to text response if unresolved.
6. Select a command.
7. Confirm risky commands.
8. Execute.

Current flow with session memory:

1. Parse CLI args, including session flags.
2. Load config.
3. Resolve environment.
4. Load the selected session, if any.
5. Build a bounded session-context summary.
6. Pass that summary into the planner and unresolved fallback prompt.
7. Select a command.
8. Confirm risky commands.
9. Execute.
10. Save a summarized turn back into the session.

This keeps memory outside the shell execution path and makes it easy to disable entirely.

## Safety Requirements

Session memory must not weaken safety, and the current implementation preserves the existing confirmation flow.

Required rules:

- A resolved follow-up command still goes through the same destructive-command checks.
- `--session` must not imply approval of a previously approved command.
- If the session context does not clearly resolve the reference, the planner should return `unresolved = true`.
- Stored session data should stay local on disk.
- Output capture should remain disabled by default.

Examples:

- `delete it` should still require confirmation when the resolved command is risky.
- `run that again` should not skip the selection flow if multiple commands were stored previously.
- `open the config again` should become unresolved if multiple config files were discussed.

## Privacy And Retention

Session memory can easily collect sensitive local details. The design should keep retention explicit and bounded.

Current defaults:

- memory enabled unless disabled in config or with `--no-session`
- short history only
- no command output storage in prompts by default
- easy session deletion with `--session-clear`

Possible later addition:

- `retention_days` for pruning old session files

## Testing Plan

Add focused unit tests first:

- config parsing for `[session_memory]`
- session name validation
- storage round-trip for a session file
- prompt rendering with and without session context
- pruning to `max_turns`
- safety behavior for ambiguous follow-up references

Later integration coverage:

- mocked Ollama tests for follow-up prompts that depend on prior turns
- confirmation flow remains unchanged for risky remembered commands
- `--session-show` and `--session-clear` behavior

## Shipped Rollout

Implemented in `0.3.0`:

- named local sessions with `--session`
- prompt injection from recent summarized turns
- save request, selected command, text response, cwd, and execution status
- add `--session-show` and `--session-clear`
- add `--session-list`
- add `--no-session`
- default-enable session memory with `working_directory` scope
- optional command stdout/stderr capture with prompt inclusion disabled by default
- automatic retention pruning when configured
- optional Ollama `/api/chat` backend through `ollama.use_chat_api`

Still worth considering later:

- retention and pruning controls
- optional session listing command
- better summaries for multiple-command plans

Longer-term:

- optional interactive session mode if there is still strong demand

The shipped implementation follows that recommendation: persisted named sessions first, REPL mode deferred.

## Future Decisions

1. Should `0.3.0` stay on `/api/generate` or move to Ollama's role-based `/api/chat`?
`0.3.0` now supports both. `/api/chat` can be enabled with `ollama.use_chat_api = true`.

2. Should command output stay JSON-backed if capture becomes heavily used?
Maybe not. SQLite becomes more attractive if stored output grows substantially.

3. Should automatic pruning or retention windows be added?
`0.3.0` adds `retention_days`, but future work may still add finer-grained pruning policies.

4. Should session memory feed both planner and unresolved text fallback?
Already yes, with the same ambiguity rules.

5. Should session data be editable by users?
Yes. The current JSON format keeps that property.

## Implementation Notes

The shipped code touches:

- `src/config.rs`: add `SessionMemoryConfig`
- `src/lib.rs`: parse session flags and wire load/save flow
- `src/llm.rs`: accept optional session context in both prompt builders
- `src/session.rs`: storage, validation, summarization, and pruning to `max_turns`
- docs for configuration, usage, and architecture after implementation lands

The feature remains disableable in config and per invocation with `--no-session`.
