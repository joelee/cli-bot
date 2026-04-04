# Installation

`cli-bot` requires a running Ollama endpoint. Install Ollama, start the local service, and pull the configured model before first use.

See [Install Ollama](install-ollama.md) for the Ollama setup steps.

## macOS With Homebrew

This install path has been tested on macOS:

```bash
brew install joelee/oss/cli-bot
```

Install the config into the default user location:

```bash
mkdir -p "$HOME/.config/cli-bot"
install -m 0644 cli-bot.toml "$HOME/.config/cli-bot/cli-bot.toml"
```

If no config file is found, `cli-bot` can create a default config automatically on first run at `~/.config/cli-bot/cli-bot.toml`.

Then verify the setup:

```bash
ollama pull lfm2:latest
cli-bot --check
```

## From crates.io

```bash
cargo install cli-bot
```

## Install Cargo / Rust Toolchain

`cargo install cli-bot` requires Cargo to be available on your machine.

Common installation paths:

### Debian / Ubuntu

```bash
sudo apt install cargo
```

### Fedora

```bash
sudo dnf install cargo
```

### Arch Linux

```bash
sudo pacman -S rust
```

### macOS With Homebrew

```bash
brew install rust
```

### Official Rust Toolchain via rustup

If you want the latest upstream Rust toolchain, install via `rustup`:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Then install `cli-bot` with:

```bash
cargo install cli-bot
```

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

This step is optional.

If no config file is found and `--config` is not provided, `cli-bot` creates a default config automatically at:

```text
$HOME/.config/cli-bot/cli-bot.toml
```

If you want to preseed or customize it before first run, install the config manually.

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
