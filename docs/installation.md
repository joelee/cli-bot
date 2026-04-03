# Installation

## Build Release Binary

```bash
cargo build --release
```

## Install Binary Into `/usr/local/bin`

```bash
sudo install -m 0755 target/release/cli-bot /usr/local/bin/cli-bot
```

After that, `cli-bot` is available on the terminal `PATH` for typical Unix-like environments.

## Install Configuration

User-local install:

```bash
mkdir -p "$HOME/.config/cli-bot"
install -m 0644 cli-bot.toml "$HOME/.config/cli-bot/cli-bot.toml"
```

System-wide install:

```bash
sudo install -m 0644 cli-bot.toml /etc/cli-bot.toml
```

You can also omit `preferred_editor` from the config and rely on the shell environment:

```bash
export EDITOR=nvim
```

## Config Discovery Order

If `--config` is omitted, `cli-bot` searches in this order:

1. `${HOME}/.config/cli-bot/cli-bot.toml`
2. `/etc/cli-bot.toml`

## Development Usage

When running from the project checkout, use the repository config explicitly:

```bash
cargo run -- --config ./cli-bot.toml "Ping google five times"
```
