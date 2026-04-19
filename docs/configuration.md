# Configuration

All runtime configuration lives in `cli-bot.toml`.

## Lookup Order

If `--config` is provided, that path is used directly.

If `--config` is not provided, `cli-bot` searches for configuration in this order:

1. `${HOME}/.config/cli-bot/cli-bot.toml`
2. `/etc/cli-bot.toml`

If neither file exists, `cli-bot` creates a default config at `${HOME}/.config/cli-bot/cli-bot.toml` automatically.

For local development from the project root, pass `--config ./cli-bot.toml` explicitly.

## Sections

### `[ollama]`

- `base_url`: Ollama server URL
- `model`: default model name to query; can be overridden at runtime with `--model`
- `temperature`: generation temperature
- `use_chat_api`: use Ollama's role-based `/api/chat` endpoint instead of `/api/generate`
- `system_prompt`: base system instruction for the planner

### `[environment]`

- `os`: operating system hint; use `auto`, `macos`, `linux`, or `unknown`
- `distro`: Linux distribution hint; use `auto`, `arch`, `debian`, `ubuntu`, `fedora`, or `unknown`
- `preferred_package_manager`: package manager override; use `auto`, `brew`, `apt`, `apt-get`, `dnf`, `pacman`, `paru`, `yay`, or `unknown`

### `[safety]`

- `require_confirmation`: require approval before risky commands run
- `destructive_substrings`: substring patterns that force confirmation

### `[ui]`

- `selection_prompt`: prompt shown when multiple command choices are available
- `approval_prompt`: prompt shown before executing risky commands
- `show_command_before_execution`: print the command before it runs
- `auto_select_recommended`: automatically use the LLM-recommended command when multiple choices are returned

### `[execution]`

- `shell`: shell executable used to run the final command
- `shell_arg`: shell flag used to pass a command string
- `preferred_editor`: preferred editor for edit-style requests; falls back to `$EDITOR` when unset

### `[session_memory]`

- `enabled`: enable session memory by default; can be disabled per invocation with `--no-session`
- `default_name`: default session name used when `--session` is omitted
- `scope`: `working_directory` or `global`; `working_directory` keeps the same session name isolated per folder
- `storage_dir`: local state directory; use `auto` for the platform default
- `max_turns`: number of recent turns kept in prompt context and on disk
- `include_working_directory`: record the cwd for each turn
- `save_text_responses`: persist unresolved text fallback responses
- `save_selected_commands`: persist selected commands and command choices
- `capture_command_output`: capture and store command stdout/stderr after execution
- `include_command_output_in_prompt`: include captured stdout/stderr in later LLM prompts
- `max_output_bytes`: truncate stored and prompt-included command output to this many bytes
- `retention_days`: automatically prune session files older than this many days; unset disables pruning

### `[models_benchmark]`

- `models`: list of Ollama model names to benchmark
- `queries`: list of natural-language queries to run against each configured model

## Example

```toml
[ollama]
base_url = "http://127.0.0.1:11434"
model = "lfm2:latest"
temperature = 0.0
use_chat_api = true
system_prompt = "Return JSON only"

[environment]
os = "auto"
distro = "auto"
preferred_package_manager = "auto"

[safety]
require_confirmation = true
destructive_substrings = ["rm -rf", "git reset --hard"]

[ui]
selection_prompt = "Choose a command"
approval_prompt = "Approve execution?"
show_command_before_execution = true
auto_select_recommended = false

[execution]
shell = "/bin/sh"
shell_arg = "-c"
preferred_editor = "nvim"

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

[models_benchmark]
models = ["lfm2:latest", "qwen3.5:latest", "gemma4:latest"]
queries = [
  "Ping google five times",
  "Print the last git log message",
  "What is my external IP address?"
]
```
