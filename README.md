# cli-bot

`cli-bot` is a Rust CLI that turns natural-language requests into shell commands using Ollama, with awareness of the user's OS, Linux distro, and package manager. Shell command generation remains the priority, with a text-response fallback only when the planner marks a request as unresolved.

Instead of remembering exact flags, command variants, and editor invocations, you can describe what you want in plain English and let `cli-bot` translate that intent into a shell command you can inspect, benchmark, approve, and run.

## Quick Start

```bash
ollama pull lfm2:latest
cargo install cli-bot
cli-bot "Ping google five times"
```

You can also skip the quotes for simple requests:

```bash
cli-bot ping google five times
```

On macOS, you can also install with Homebrew:

```bash
brew install joelee/oss/cli-bot
cli-bot "Ping google five times"
```

This assumes:

- Ollama is installed
- the Ollama service is running locally
- `cli-bot` can create a default config file on first run if needed

See [Install Ollama](docs/install-ollama.md) and [Configuration](docs/configuration.md) for the full setup.

If you encounter any issues, run `cli-bot --check` to identify the issues. 

## Why This Exists

Terminal users often know what they want to do, but not always the exact command shape.

Examples:

- You remember `find` or `du` can do the job, but not the right flags.
- You know there is a one-liner for archive extraction, file inspection, or network diagnostics, but you do not want to search the web again.
- You want a local LLM-assisted shell helper without sending requests to a hosted service.

`cli-bot` is built for that gap between intent and syntax.

## Who This Is For

- developers who live in the terminal and want a faster way to recall commands
- DevOps and platform engineers who need quick shell help without leaving the terminal
- Linux and macOS users who know the shell but do not memorize every flag combination
- newcomers who want a safer, more guided path into command-line usage
- teams that want a local, Ollama-backed CLI instead of a cloud dependency

## Inspiration

The inspiration for `cli-bot` is simple: many shell tasks are easy once you know the exact command, but the friction of remembering the exact syntax breaks flow.

This project was created to give users a practical local assistant for terminal work:

- natural language in
- real shell commands out
- confirmation for risky operations
- choice when multiple commands are plausible
- benchmark data to compare local Ollama models in real usage

It is meant to feel less like a chatbot and more like a sharp command-line copilot.

## What It Does

- translates plain-English requests into shell commands with Ollama
- supports configurable local models through `cli-bot.toml`
- understands the user's OS and preferred package manager for package-related requests
- asks for approval before running potentially destructive commands
- lets the user pick between multiple command choices
- can optionally auto-select the LLM-recommended best command
- can fall back to a direct text response when a request cannot be resolved into a shell command safely
- supports benchmarking to compare models by latency
- supports verbose debugging to inspect full Ollama responses
- respects a preferred editor from config or `$EDITOR`
- includes colorized output, checks, tests, and local pre-commit hooks

## Real-Life Examples

### Networking

```bash
cli-bot "Ping google five times"
```

Unquoted arguments also work:

```bash
cli-bot ping google five times
```

You can also run `cli-bot` with no request string and it will prompt you interactively.

Expected command:

```bash
ping -c 5 google.com
```

### Text Response

```bash
cli-bot "spell mantainence"
```

Expected response:

```text
maintenance
```

### Disk Usage

```bash
cli-bot "Show the ten largest files in the current directory"
```

Possible output plan could include commands using `find`, `du`, and `sort`.

### Package Management

On macOS with Homebrew:

```bash
cli-bot "Install btop"
```

Expected command:

```bash
brew install btop
```

On Arch Linux with `paru` available:

```bash
cli-bot "List all my installed packages"
```

Expected command could be:

```bash
paru -Q
```

### Archives

```bash
cli-bot "Extract archive.tar.gz here"
```

Expected command:

```bash
tar -xzf archive.tar.gz
```

### Search in Logs

```bash
cli-bot "Show me lines containing timeout in server.log"
```

Expected command:

```bash
grep -n "timeout" server.log
```

### Git Help

```bash
cli-bot "Show branches merged into main"
```

Expected command could be:

```bash
git branch --merged main
```

### Editor-Aware Requests

If `preferred_editor = "nvim"` or `$EDITOR=nvim`:

```bash
cli-bot "Edit my git config file"
```

Expected command:

```bash
nvim ~/.gitconfig
```

### Safety for Risky Commands

```bash
cli-bot "Delete the target directory"
```

If the selected command is destructive, `cli-bot` asks for explicit approval before execution.

### Unresolved Fallback

If a request cannot be safely resolved into a shell command, `cli-bot` can fall back to a direct text response instead of guessing an incorrect command.

## Installation

`cli-bot` requires a running Ollama endpoint. Before using the CLI, install Ollama, start the local service, and pull the model you want to use.

See [Install Ollama](docs/install-ollama.md) for a step-by-step setup guide.

### From Homebrew on macOS

This install path has been tested on macOS:

```bash
brew install joelee/oss/cli-bot
```

Then install the config into the default user location:

```bash
mkdir -p "$HOME/.config/cli-bot"
install -m 0644 cli-bot.toml "$HOME/.config/cli-bot/cli-bot.toml"
```

If you skip that step, `cli-bot` will create a default config automatically at `~/.config/cli-bot/cli-bot.toml` when no config file is found.

Before first use, make sure Ollama is running and the default model is available:

```bash
ollama pull lfm2:latest
cli-bot --check
```

`--check` now also reports whether the current terminal environment supports interactive prompts and command selection.

### From crates.io

After publishing, users can install with:

```bash
cargo install cli-bot
```

This builds the executable from source on the user's machine.

If `cargo` is not installed yet, install the Rust toolchain first.

Common options:

- Debian/Ubuntu:
  ```bash
  sudo apt install cargo
  ```
- Fedora:
  ```bash
  sudo dnf install cargo
  ```
- Arch Linux:
  ```bash
  sudo pacman -S rust
  ```
- macOS with Homebrew:
  ```bash
  brew install rust
  ```

If you want the latest official Rust toolchain instead of a distro package, use `rustup`:

```bash
curl https://sh.rustup.rs -sSf | sh
```

### From Source

```bash
git clone https://github.com/joelee/cli-bot
cd cli-bot
cargo build --release
sudo install -m 0755 target/release/cli-bot /usr/local/bin/cli-bot
```

### Optional Shell Alias

If you want a shorter command, you can alias `cli-bot` to `cb` in your shell profile:

```bash
alias cb="cli-bot"
```

Then you can run commands like:

```bash
cb "Ping google five times"
```

## Ollama Setup

At minimum, make sure:

1. Ollama is installed
2. The Ollama service is running locally
3. The default model is pulled

Example:

```bash
ollama pull lfm2:latest
```

Detailed instructions are in [Install Ollama](docs/install-ollama.md).

## Configuration

If no config file is provided and none is found in the normal lookup locations, `cli-bot` creates a default config automatically at:

```text
$HOME/.config/cli-bot/cli-bot.toml
```

You can also configure the environment profile used for package-related requests, including `preferred_package_manager`.

Install the config into one of the default lookup locations.

User-local:

```bash
mkdir -p "$HOME/.config/cli-bot"
install -m 0644 cli-bot.toml "$HOME/.config/cli-bot/cli-bot.toml"
```

System-wide:

```bash
sudo install -m 0644 cli-bot.toml /etc/cli-bot.toml
```

When `--config` is not provided, `cli-bot` looks for configuration in this order:

1. `${HOME}/.config/cli-bot/cli-bot.toml`
2. `/etc/cli-bot.toml`

For local development from the repository root:

```bash
cargo run -- --config ./cli-bot.toml "Ping google five times"
```

## Good First Commands

```bash
cli-bot "List all listening TCP ports"
cli-bot "Show disk usage for this folder"
cli-bot "Find all .rs files containing TODO"
cli-bot "Show the last 50 lines of app.log"
cli-bot "Create a gzipped tarball of the dist folder"
```

You can also start with no arguments and type the request when prompted:

```bash
cli-bot
```

## Safety Model

`cli-bot` does not blindly trust model output.

- commands can be marked `potentially_destructive` by the LLM
- destructive substring rules in config provide an additional safety net
- risky commands require explicit user approval before execution
- if multiple commands are returned, the user can choose the right one

This keeps the tool useful without pretending shell execution is risk-free.

## Model Comparison and Benchmarking

One of the practical uses of `cli-bot` is comparing local Ollama models for real CLI tasks.

```bash
cli-bot --config ./cli-bot.toml --model lfm2:latest --benchmark --dry-run "Ping google five times"
```

The current default model selection is documented in the published benchmark report:

- [Models Benchmark Report](docs/models-benchmark-report.md)

That report captures comparative results across multiple models and is the basis for choosing `lfm2:latest` as the default model.

The benchmark report includes:

- the model used
- planner latency in milliseconds
- execution latency in milliseconds, or `skipped` for dry runs
- total end-to-end runtime

This makes it easy to compare models for speed and response quality on realistic terminal prompts.

## Debugging Model Output

Some models follow structured output instructions better than others.

Use verbose mode when you need to inspect the full Ollama exchange:

```bash
cli-bot --config ./cli-bot.toml --model lfm2.5-thinking:latest --verbose --benchmark "Ping google five times"
```

Verbose mode shows:

- resolved config and model
- preferred editor
- full Ollama request body
- full raw Ollama response body
- extracted planner JSON before deserialization

This is especially useful when a model adds extra commentary, malformed JSON, or the wrong schema.

## Health Check

Use `--check` to verify the local environment:

```bash
cli-bot --config ./cli-bot.toml --check
```

This validates:

- the config file can be found and parsed
- the Ollama service is reachable
- the configured model exists in Ollama
- the preferred editor resolves and is available

## Best Choice Selection

By default, `cli-bot` shows an interactive selector when multiple commands are returned.

If you want the LLM to choose the best option automatically:

- set `ui.auto_select_recommended = true` in `cli-bot.toml`
- or pass `--auto-select-best`

## Color Output

`cli-bot` supports ANSI color output with:

```bash
cli-bot --color auto "Ping google five times"
```

Available values:

- `auto`
- `always`
- `never`

If `NO_COLOR` is set, color output is disabled.

## Main Flags

- `--config <path>`: use a specific config file
- `--model <name>`: override the configured Ollama model for this invocation
- `--auto-select-best`: automatically use the LLM-recommended command when multiple choices are returned
- `--dry-run`: show the selected command without executing it
- `--print-plan`: print the structured planner response
- `--benchmark`: print planning, execution, and total elapsed time in milliseconds
- `--models-benchmark`: run configured model/query benchmarks and print a Markdown report
- `--models-benchmark [file]`: run configured model/query benchmarks and print Markdown to stdout or the given file
- `--check`: verify config, Ollama connectivity, model availability, and editor resolution
- `--color <auto|always|never>`: control ANSI color output
- `--quiet`: hide cli-bot informational output and only show the selected command output
- `--verbose`: print detailed actions and full Ollama responses for debugging

## Documentation

- [Changelog](CHANGELOG.md)
- [Installation](docs/installation.md)
- [Install Ollama](docs/install-ollama.md)
- [Homebrew](docs/homebrew.md)
- [crates.io Release](docs/crates-release.md)
- [Publishing](docs/publishing.md)
- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Usage](docs/usage.md)
- [Testing](docs/testing.md)
- [Roadmap](docs/roadmap.md)

## Developer Setup

If you use the `pre-commit` framework, install the repo hooks with:

```bash
pre-commit install
```

The repository includes `.pre-commit-config.yaml`, which runs `./scripts/verify.sh` before commits.
