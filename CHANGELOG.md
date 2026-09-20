# Changelog

All notable changes to `cli-bot` will be documented in this file.

The format is based on Keep a Changelog.

## [Unreleased]

### Changed

- **Commands now need approval unless they are recognised as read-only** (PLAN-00002, `REV-00001-MAJ-01`). A command is classified by parsing it into programs: read-only programs run as before, known-destructive ones get a strong prompt, and everything else gets an ordinary prompt. Previously a command ran without a prompt unless the model flagged it or its text contained a configured substring, which let `rm -fr`, `rm -r -f`, `shred`, `curl | sh`, and redirections through
- Changed the default model to `ornith-1.5:9b` in the default config template; docs now point to `docs/models-benchmark-report-v0.3.2.md` as the report behind that choice
- Adopted the release lifecycle of the `passalong` project: releases are now tag-driven through the `Release` GitHub workflow, which checks the tag and records with `scripts/check-release-tag.sh`, builds binaries, publishes to crates.io after the `release` environment is approved, and creates the GitHub release from `docs/release/vX.Y.Z.md`; changelog version headings are `vX.Y.Z - <UTC timestamp>`; the Homebrew formula is updated by `scripts/update-homebrew-formula.sh` on a tap branch; `scripts/release.sh` and `just release` are gone
- `just check` now runs the Markdown link check (`scripts/check-links.sh`) and no longer runs `cargo package`; `just ci` adds the supply-chain audit (`cargo deny` with `deny.toml`), a `publish-dry-run`, and the workflow lint
- The crate README links are absolute GitHub URLs, because crates.io resolves relative links against the crate's folder
- The changelog entry for new work is added at branch creation, before the plan, matching `passalong`

### Added

- `--yes` / `-y` and `[safety] assume_yes` to pre-approve ordinary prompts for non-interactive use, and `--i-approve-destructive-commands` to pre-approve the destructive tier as well (command line only, never config)
- `[safety] read_only_commands` and `[safety] destructive_commands` to extend the built-in classification

### Testing

- The confirmation, selection, and request prompts are behind a `Prompter` trait (`REV-00001-MED-06`), so the approve, decline, and select paths are covered by integration tests through the real application flow

## [0.3.2] - 2026-09-17

The first published release since 0.3.0; it includes everything listed under 0.3.1.

### Changed

- Adopted the working rules of the `passalong` project in `AGENTS.md`: test-first development, mocked external interfaces, numbered delivery plans, and a backlog in `docs/backlog.md` (PLAN-00001)
- Added a `justfile` as the single task runner; `just check` runs the format check, lint, script syntax check, tests, coverage gate, locked build, and package verification, and the Git hook and CI call the same recipes
- Replaced `scripts/verify.sh`, `scripts/install-hooks.sh`, and `scripts/coverage-unit.sh` with `just` recipes; coverage reports now come from `cargo llvm-cov`
- Enforced a line-coverage gate of 80% locally and in CI
- Pinned the Rust toolchain to 1.98.1 in `rust-toolchain.toml`

### Added

- `.env.sample` documenting `HOMEBREW_FORMULA_FILE` for `scripts/release.sh`
- `docs/developer-guide.md`, and `docs/backlog.md` in place of `docs/roadmap.md`

### Testing

- Added mocked-Ollama integration tests for command execution, session memory, verbose output, `--check` failures, and `--models-benchmark`, raising line coverage from 76% to 89%

## [0.3.1] - 2026-04-19

Not published to crates.io and never tagged; shipped as part of 0.3.2.

### Changed

- Increased unit test coverage across CLI orchestration, session storage, shell execution, and prompt helpers
- Added Codecov unit test result uploads using `codecov/test-results-action@v1`
- Expanded the unit coverage workflow to produce and upload JUnit XML test results

### Added

- `--interactive` / `-i` mode to keep prompting for requests until `Ctrl-C` or `/quit`
- interactive prompt styling with cyan `cli-bot`, gray `>`, red `Error:` output on command failure, and a red `cli-bot` prompt after the failure
- interactive startup hint showing `/quit` and `Ctrl-C` exit options

### Testing

- Added targeted unit tests for benchmark rendering, session command helpers, shell execution branches, and session text/output helpers
- Added CI test result export through `cargo2junit` for Codecov ingestion

## [0.3.0] - 2026-04-18

### Added

- Multi-turn session memory with local JSON-backed session storage and bounded prompt context
- `--session [name]`, `--session-list`, `--session-show`, `--session-clear`, and `--no-session`
- `session_memory` configuration for enabling, scoping, storing, and truncating local session data
- Optional command stdout/stderr capture in session history, with prompt inclusion disabled by default
- `session_memory.retention_days` for automatic session pruning
- `ollama.use_chat_api` to switch between Ollama `/api/chat` and `/api/generate`

### Changed

- Session memory is enabled by default and scoped to the current working directory by default to avoid cross-project leakage
- Planner and unresolved text fallback prompts now receive structured session context when available
- Documentation now reflects the shipped session-store behavior in `0.3.0`

### Testing

- Added mocked integration coverage for planner transport, unresolved fallback, health checks, and session commands
- Added `scripts/coverage-unit.sh` for repeatable LLVM-based unit coverage generation
- Added GitHub Actions workflows for unit coverage artifacts and GitHub Pages coverage publishing
- Added Codecov upload support from the unit coverage workflow
- Added documentation for enabling GitHub Pages and browsing published coverage reports

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
- Roadmap in `docs/roadmap.md` for future features beyond the current release scope

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
