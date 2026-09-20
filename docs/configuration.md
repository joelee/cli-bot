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

- `require_confirmation`: the master switch. Setting it to `false` runs every command without asking, including destructive ones
- `assume_yes`: pre-approve the state-changing tier, as `--yes` does. There is deliberately no key for the destructive tier; that needs `--i-approve-destructive-commands` on the command line
- `read_only_commands`: commands that run without asking. Each entry matches whole leading words, so `git log` covers `git log --oneline` but not `git logs`
- `destructive_commands`: extra commands for the destructive tier, matched the same way
- `destructive_substrings`: text that forces the destructive tier wherever it appears, matched case-insensitively. Kept from earlier versions, and it can fire on a quoted argument

#### How a command is classified

`cli-bot` parses the command into the programs it runs, following `;`, `&&`,
`||`, and pipes, honouring quotes, and stripping wrappers such as `sudo`,
`env`, `nice`, and `xargs`. It then decides, in this order:

1. the planner marked the command `potentially_destructive` → destructive
2. the text matches a `destructive_substrings` entry → destructive
3. a built-in rule or a `destructive_commands` entry matches → destructive
4. the command redirects output or uses `$(...)` → state-changing
5. every program matches `read_only_commands` → read-only
6. otherwise → state-changing

A command that cannot be parsed is never read-only.

#### Built-in destructive rules

Some programs are only dangerous with certain flags or subcommands, which a
list of names cannot express, so these rules live in the code:

| Program | Destructive when |
|---|---|
| `rm` | `-r`, `-R`, `-f`, `--recursive`, or `--force` |
| `find` | `-delete`, `-exec`, `-execdir`, `-ok`, `-okdir`, `-fls`, `-fprint` |
| `chmod`, `chown`, `chgrp` | `-R` or `--recursive` |
| `git` | `clean`; `reset --hard`; `push --force`; `branch -D` |
| `docker`, `podman` | any `prune`; `rmi`; `volume`/`container`/`image`/`network rm` |
| `systemctl`, `service` | `stop`, `disable`, `mask` |
| `shred`, `dd`, `mkfs*`, `fdisk`, `parted`, `mkswap`, `wipefs`, `sgdisk`, `truncate` | always |
| `shutdown`, `reboot`, `poweroff`, `halt`, `init` | always |
| `kill`, `killall`, `pkill`, `userdel`, `groupdel` | always |

### `[ui]`

- `selection_prompt`: prompt shown when multiple command choices are available
- `approval_prompt`: prompt shown before a destructive command
- `confirmation_prompt`: prompt shown before a state-changing command
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
model = "ornith-1.5:9b"
temperature = 0.0
use_chat_api = true
system_prompt = "Return JSON only"

[environment]
os = "auto"
distro = "auto"
preferred_package_manager = "auto"

[safety]
require_confirmation = true
assume_yes = false
read_only_commands = ["ls", "cat", "git status", "git log"]
destructive_commands = []
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
models = ["ornith-1.5:9b", "qwen3.8:27b", "nemotron-3-super:120b"]
queries = [
  "Ping google five times",
  "Print the last git log message",
  "What is my external IP address?"
]
```
