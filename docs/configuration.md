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

## Example

```toml
[ollama]
base_url = "http://127.0.0.1:11434"
model = "lfm2:latest"
temperature = 0.0
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
```
