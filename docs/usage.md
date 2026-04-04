# Usage

## Run

```bash
cargo run -- --config ./cli-bot.toml "Ping google five times"
```

Package-related requests use the resolved OS and package manager context.

```bash
cargo run -- --config ./cli-bot.toml "Install btop"
```

If you run `cli-bot` without a request string, it prompts you to type one interactively.

```bash
cargo run -- --config ./cli-bot.toml
```

## Dry Run

```bash
cargo run -- --config ./cli-bot.toml --dry-run "Show disk usage for the current directory"
```

## Quiet Output

```bash
cargo run -- --config ./cli-bot.toml --quiet "Show disk usage for the current directory"
```

`--quiet` hides cli-bot informational output such as:

- `Selected command:`
- `Why:`
- benchmark summaries
- successful `--check` output

The selected shell command still writes directly to the terminal.

## Print Structured Plan

```bash
cargo run -- --config ./cli-bot.toml --print-plan "List the ten largest files here"
```

## Benchmark

```bash
cargo run -- --config ./cli-bot.toml --benchmark --dry-run "Ping google five times"
```

This prints:

- `model`: the Ollama model used for this invocation
- `planning_ms`: time spent waiting for Ollama to return a plan
- `execution_ms`: time spent running the selected shell command, or `skipped` in `--dry-run`
- `total_ms`: end-to-end elapsed time for the CLI invocation

## Color Output

```bash
cargo run -- --config ./cli-bot.toml --color always --check
```

Available values:

- `auto`: enable ANSI colors only when the output stream is a terminal
- `always`: always emit ANSI colors
- `never`: disable ANSI colors

If the `NO_COLOR` environment variable is set, color output is disabled.

## Model Override

```bash
cargo run -- --config ./cli-bot.toml --model lfm2:latest --benchmark --dry-run "Ping google five times"
```

`--model` overrides `ollama.model` from the config file for that single invocation. It also affects `--check`.

## LLM Best Choice

```bash
cargo run -- --config ./cli-bot.toml --auto-select-best "Ping google five times"
```

When multiple commands are returned, `--auto-select-best` uses the command marked by the LLM as `recommended: true`.
The same behavior can be enabled by default with `ui.auto_select_recommended = true` in `cli-bot.toml`.

## Verbose Debugging

```bash
cargo run -- --config ./cli-bot.toml --model lfm2.5-thinking:latest --verbose --benchmark "Ping google five times"
```

`--verbose` prints:

- resolved config path, model, and preferred editor
- the natural-language request
- the full Ollama generate request body
- the full raw Ollama HTTP response body
- the generated text extracted from the Ollama response
- the extracted planner JSON before deserialization

This is intended for debugging model-specific formatting errors.

## Environment Check

```bash
cargo run -- --config ./cli-bot.toml --check
```

This verifies:

- the config file can be found and parsed
- the Ollama service is reachable
- the configured model is available in Ollama
- the resolved OS and distro
- the detected and effective package manager
- the preferred editor resolves from config or `$EDITOR` and is available

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
- Package-related requests use the resolved environment and effective package manager.
- The LLM is required to return `potentially_destructive: true | false` for each command.
- The LLM is also asked to mark the best command with `recommended: true`.
- If the selected command is marked or detected as destructive, `cli-bot` requests explicit approval.
