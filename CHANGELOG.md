# Changelog

All notable changes to `cli-bot` will be documented in this file.

The format is based on Keep a Changelog.

## [0.2.2] - 2026-04-17

### Added

- Unquoted free-argument request parsing so `cli-bot ping google five times` works without surrounding quotes ([#3](https://github.com/joelee/cli-bot/issues/3))

### Changed

- `docs/usage.md` now uses the installed `cli-bot` command in examples instead of `cargo run --`

## [0.2.1] - 2026-04-04

### Added

- Command-plan `unresolved` fallback for requests that should not be forced into guessed commands
- Second-pass text response generation when the command planner marks a request as unresolved
- `--models-benchmark` to benchmark configured models across configured queries and print a Markdown report
- Host system details at the top of the models benchmark report, including OS, kernel, CPU, GPU, and memory
- Best-effort GPU VRAM reporting in the models benchmark host section
- Per-model benchmark summary, success rate, and ranking sections in the Markdown benchmark report
- Optional file output for `--models-benchmark`, with detailed results grouped by query first for easier model comparison
- [Roadmap](docs/roadmap.md) for future features beyond the current release scope

### Changed

- CLI command generation remains the priority, with direct text responses only after the command planner explicitly marks a request as unresolved
- Package-manager instructions are now restricted to clearly package-related requests to reduce false positives
- Documentation now links to the published benchmark report that justifies `lfm2:latest` as the default model
- Unquoted free arguments are now joined into a single natural-language request, so `cli-bot ping google five times` works without extra quotes

## [0.2.0] - 2026-04-04

### Added

- Environment-aware planning with OS, distro, and package manager context
- `preferred_package_manager` configuration override
- Automatic package manager detection for common environments including Homebrew, apt, dnf, pacman, paru, and yay
- Arch package manager priority of `paru -> yay -> pacman` in auto-detection mode
- `--quiet` / `-q` to suppress cli-bot informational output while leaving the selected command output visible

### Changed

- `--check` now reports resolved OS, distro, detected package manager, and effective package manager
- The LLM prompt now includes the resolved environment so package-related requests use platform-appropriate commands

## [0.1.2] - 2026-04-03

### Added

- Interactive request prompting when `cli-bot` is run without a request argument
- Stdin fallback for non-interactive request input when no request argument is provided
- `-a` short flag for `--auto-select-best`
- `.pre-commit-config.yaml` for developers using the `pre-commit` framework
- `scripts/verify.sh` as the shared local verification entrypoint
- `scripts/release.sh` for local crates.io release publishing and Homebrew formula updates
- [crates.io Release](docs/crates-release.md) for crates.io release instructions
- [Homebrew](docs/homebrew.md) for Homebrew update instructions
- Automatic bootstrap of a default config file at `$HOME/.config/cli-bot/cli-bot.toml` when no config exists

### Changed

- Normal output no longer prints the planner summary or request text unless `--verbose` is enabled
- Pre-commit verification now runs formatting, linting, tests, shell script syntax checks, and `cargo package`
- Release automation now uses the checksum of the published crates.io artifact for Homebrew updates

### Fixed

- Corrected the Homebrew checksum update flow to avoid using the local packaged crate hash

## [0.1.1] - 2026-04-03

### Added

- Automatic creation of a default user config file when no config file is found
- macOS Homebrew installation documentation
- [Install Ollama](docs/install-ollama.md) for Ollama installation and model setup

### Changed

- Default model changed to `lfm2:latest`
- Installation docs now treat manual config installation as optional

## [0.1.0] - 2026-04-02

### Added

- Initial Rust CLI scaffold for translating natural-language requests into shell commands with Ollama
- Config-driven model, execution, UI, and safety settings through `cli-bot.toml`
- Destructive-command confirmation flow
- Multi-command interactive selection flow
- Optional LLM-recommended best-command auto-selection
- Benchmark output for planner and execution timing
- Verbose debugging mode with raw Ollama output
- Editor preference support through config and `$EDITOR`
- Environment validation with `--check`
- ANSI color output control with `--color auto|always|never`
- Unit tests, local hooks, release checks workflow, and project documentation
