# Usage

## Run

```bash
cli-bot "Ping google five times"
```

You can also omit the quotes and let `cli-bot` join the free arguments into one request:

```bash
cli-bot ping google five times
```

Package-related requests use the resolved OS and package manager context.

```bash
cli-bot "Install btop"
```

If the planner cannot safely resolve a request into a shell command, `cli-bot` can fall back to a plain text response:

```bash
cli-bot "spell mantainence"
```

If you run `cli-bot` without a request string, it prompts you to type one interactively.

```bash
cli-bot
```

## Interactive Mode

Use `--interactive` or `-i` to keep prompting for new requests until you press `Ctrl-C` or type `/quit`.

```bash
cli-bot -i
```

You can also start interactive mode with an initial request:

```bash
cli-bot --interactive "find my git config file"
```

In this mode, `cli-bot` runs each request, prints the result, and then prompts again instead of exiting after one response.

When interactive mode starts, `cli-bot` prints a short exit hint telling you to use `/quit` or `Ctrl-C`.

The interactive prompt is shown as cyan `cli-bot` with a gray `>` suffix. If the previous command exits with an error, `cli-bot` prints a red `Error:` line and the next prompt shows `cli-bot` in red.

Session memory continues to work in interactive mode, so follow-up requests can still use the active session context.

Example with an explicit session:

```bash
cli-bot -i --session release
cli-bot> find the changelog entry for 0.3.1
cli-bot> open it in nvim
cli-bot> /quit
```

## Session Memory

Session memory is enabled by default in `0.3.1`.

By default, `cli-bot` uses the configured `session_memory.default_name` and scopes it to the current working directory, so the same session name does not leak across unrelated folders.

```bash
cli-bot "find my git config file"
cli-bot "open it in nvim"
```

Use an explicit session name when you want to keep a separate thread of follow-up requests:

```bash
cli-bot --session release "find the changelog entry for 0.3.1"
cli-bot --session release "open it again"
```

Inspect the current session:

```bash
cli-bot --session release --session-show
```

Clear the current session:

```bash
cli-bot --session release --session-clear
```

List all stored sessions:

```bash
cli-bot --session-list
```

Disable session memory for one invocation:

```bash
cli-bot --no-session "Ping google five times"
```

When `session_memory.capture_command_output = true`, command stdout and stderr are stored separately as command output. They are not included in later prompts unless `session_memory.include_command_output_in_prompt = true`.

If `session_memory.retention_days` is set, old session files are pruned automatically on startup.

## Dry Run

```bash
cli-bot --dry-run "Show disk usage for the current directory"
```

## Quiet Output

```bash
cli-bot --quiet "Show disk usage for the current directory"
```

`--quiet` hides cli-bot informational output such as:

- `Selected command:`
- `Why:`
- benchmark summaries
- successful `--check` output

The selected shell command still writes directly to the terminal.

Plain text responses, such as spelling corrections, are still shown because they are the primary result.

## Print Structured Plan

```bash
cli-bot --print-plan "List the ten largest files here"
```

## Benchmark

```bash
cli-bot --benchmark --dry-run "Ping google five times"
```

This prints:

- `model`: the Ollama model used for this invocation
- `planning_ms`: time spent waiting for Ollama to return a plan
- `execution_ms`: time spent running the selected shell command, or `skipped` in `--dry-run`
- `total_ms`: end-to-end elapsed time for the CLI invocation

## Models Benchmark

```bash
cli-bot --models-benchmark
```

Or write the Markdown report directly to a file:

```bash
cli-bot --models-benchmark docs/models-benchmark-report.md
```

You can also use:

```bash
cli-bot --models-benchmark=docs/models-benchmark-report.md
```

This mode:

- reads `[models_benchmark].models` and `[models_benchmark].queries` from `cli-bot.toml`
- runs every query against every configured model
- captures planner time and text-fallback time when unresolved
- includes host system details at the top of the report, including GPU VRAM when it can be detected
- prints a Markdown report with a query-vs-model summary table, per-model summary, ranking, and detailed responses
- writes to stdout by default, or to the provided Markdown filename when one is passed to `--models-benchmark`

Commands are never executed in this mode. It is for comparing model behavior and response quality.

The current published benchmark report is available at [Models Benchmark Report](models-benchmark-report.md), and it documents why `lfm2:latest` is the default model.

## Color Output

```bash
cli-bot --color always --check
```

Available values:

- `auto`: enable ANSI colors only when the output stream is a terminal
- `always`: always emit ANSI colors
- `never`: disable ANSI colors

If the `NO_COLOR` environment variable is set, color output is disabled.

## Model Override

```bash
cli-bot --model lfm2:latest --benchmark --dry-run "Ping google five times"
```

`--model` overrides `ollama.model` from the config file for that single invocation. It also affects `--check`.

## Session Flags

```bash
cli-bot --session work "open it again"
```

- `--session [name]`: use the configured default session or the named session
- `--session-list`: list stored sessions and exit
- `--session-show`: print stored turns for the active session and exit
- `--session-clear`: clear the active session and exit
- `--no-session`: disable session memory for the current invocation

## Ollama Backend

`cli-bot` now supports both Ollama request styles:

- `/api/chat` when `ollama.use_chat_api = true`
- `/api/generate` when `ollama.use_chat_api = false`

The planner contract stays the same in both modes.

## LLM Best Choice

```bash
cli-bot --auto-select-best "Ping google five times"
```

When multiple commands are returned, `--auto-select-best` uses the command marked by the LLM as `recommended: true`.
The same behavior can be enabled by default with `ui.auto_select_recommended = true` in `cli-bot.toml`.

## Verbose Debugging

```bash
cli-bot --model lfm2.5-thinking:latest --verbose --benchmark "Ping google five times"
```

`--verbose` prints:

- resolved config path, model, and preferred editor
- whether the planner marked the request as unresolved before the text fallback
- the natural-language request
- the rendered session context when available
- the full Ollama request body
- the full raw Ollama HTTP response body
- the generated text extracted from the Ollama response
- the extracted planner JSON before deserialization

This is intended for debugging model-specific formatting errors.

## Environment Check

```bash
cli-bot --check
```

This verifies:

- the config file can be found and parsed
- the Ollama service is reachable
- the configured model is available in Ollama
- the resolved OS and distro
- the detected and effective package manager
- the preferred editor resolves from config or `$EDITOR` and is available
- the terminal environment is suitable for interactive prompts and command selection

## Installed Binary

After installing the release binary, you can run:

```bash
cli-bot "Ping google five times"
```

Without `--config`, the installed CLI checks `${HOME}/.config/cli-bot/cli-bot.toml` first and `/etc/cli-bot.toml` second.

## Editor-Aware Requests

If `preferred_editor = "nvim"` or `$EDITOR=nvim`, a request like this should guide the planner toward `nvim`:

```bash
cli-bot "Edit my git config file"
```

## Behavior

- If the planner returns one command, `cli-bot` selects it automatically.
- If the planner returns multiple commands, `cli-bot` presents an interactive selector unless `--auto-select-best` or `ui.auto_select_recommended = true` is enabled.
- If no request string is provided, `cli-bot` prompts for one interactively.
- Session memory is enabled by default unless `session_memory.enabled = false` or `--no-session` is used.
- The default `working_directory` session scope keeps follow-up memory local to the current folder.
- If the planner marks a request as unresolved, `cli-bot` makes a second LLM request for a plain text response instead of guessing a command.
- Package-related requests use the resolved environment and effective package manager.
- The LLM is required to return `potentially_destructive: true | false` for each command.
- The LLM is also asked to mark the best command with `recommended: true`.
- If the selected command is marked or detected as destructive, `cli-bot` requests explicit approval.
