# AGENTS.md - cli-bot (Rust/cargo/just)

## Scope
Applies to this repo unless a deeper `AGENTS.md` overrides it. Follow explicit user instructions first. `docs/ideas/`, `docs/plans/`, and `docs/reviews/` each have their own `AGENTS.md`; read it before writing there. These rules are adapted from `../passalong/AGENTS.md`.

## Purpose
`cli-bot` is a Rust CLI that turns natural-language requests into shell commands through Ollama, asks before running a risky command, and runs the selected command in the user's shell.

## Stack
- Single Rust crate managed by `cargo`; `Cargo.lock` is committed.
- Toolchain pinned in `rust-toolchain.toml`. Keep the version in the GitHub workflows in step with it.
- Format with `rustfmt`; lint with `clippy`; run every task through the `justfile`.
- cli-bot runs commands in the user's own shell, so there is no container build or runtime.

## Product rules
- Keep behaviour config-driven through `cli-bot.toml`.
- Preserve the safety model: risky commands require explicit approval before execution.
- When multiple commands are returned, keep the interactive selection flow intact.
- Keep shell command generation as the primary product behaviour.
- Prefer small, local changes over broad refactors.

## Non-negotiables
- TDD: write or update a failing unit test before production code; then implement; then refactor.
- Unit tests must mock external interfaces: HTTP, filesystem edges, clocks, randomness, subprocesses.
- Every feature includes integration-test coverage planned up front and implemented before completion. Exercise Ollama- and IO-driven behaviour through the real application flow with the mock server in `tests/mock_ollama.rs`; never call a real Ollama service or the network from a test.
- A test may run only a harmless command through the shell, such as `printf`. A test must never open a terminal prompt: `cargo test` run from a terminal has a TTY.
- Coverage gate: line coverage >= 80%; report coverage in completion notes.
- Add comments/rustdoc for public APIs, unsafe code, complex blocks, and non-obvious logic; avoid comments that restate code.
- Avoid `unsafe`; if unavoidable, justify with a safety comment and test coverage.
- No secrets in code, tests, docs, logs, or VCS.

## Commands
Use the `just` recipes. Each wraps the cargo command shown:
- Format check: `just fmt-check` (`cargo fmt --all -- --check`)
- Lint: `just lint` (`cargo clippy --workspace --all-targets --all-features -- -D warnings`)
- Script syntax: `just scripts-check` (`bash -n` on `scripts/*.sh` and `.githooks/*`)
- Markdown links: `just links` (`scripts/check-links.sh`)
- Tests: `just test` (`cargo test --workspace --all-targets --all-features`)
- Coverage: `just coverage` (`cargo llvm-cov --workspace --all-features --fail-under-lines 80 --summary-only`)
- Build: `just build` (`cargo build --workspace --all-features --locked`)
- Supply-chain audit: `just audit` (`cargo deny check` with `deny.toml`)
- Publish dry run: `just publish-dry-run` (`cargo publish --dry-run --locked` after clearing same-version builds)
- All checks: `just check` (format, lint, scripts, links, tests, coverage, build). `just ci` adds the audit, the publish dry run, and `just lint-workflows`.
- The Git hook (`.githooks/pre-commit`, enabled with `just install-hooks`), `.pre-commit-config.yaml`, and CI all run `just check`. Never bypass the hook; verify it with `pre-commit run --all-files`.
- See `docs/developer-guide.md` for every recipe.

## Config and secrets
- Secrets live only in `.env`; `.env` is in `.gitignore`.
- Maintain `.env.sample` with supported variable names and safe example values.
- Non-secret config lives in the TOML config file.
- Config lookup order for new work:
  1. `--config` argument
  2. `CLI_BOT_CONFIG_FILE`
  3. `$XDG_CONFIG_HOME/cli-bot/config.toml`
  4. `$HOME/.config/cli-bot/config.toml`
  5. `/etc/cli-bot/config.toml`
  6. `./config.toml`
- Document all config in `docs/configuration.md` and keep defaults deterministic.

## Observability
- Logging is mandatory in core flows. Do not use `println!`/`eprintln!` for app logs except CLI output explicitly meant for users.
- Use syslog-compatible output and levels exposed as: Error, Warning, Info, Verbose, Debug.
- Map levels consistently: Error=err, Warning=warning, Info=info, Verbose=notice, Debug=debug.
- Logs must include timestamp, level, target/component, event/message, and correlation/request id when available.
- Never log secrets or full credentials.

## Docs to maintain
Update when behavior, commands, config, architecture, or user workflow changes:
- `README.md`
- `CHANGELOG.md`
- `docs/architecture.md`
- `docs/configuration.md`
- `docs/usage.md`
- `docs/session-memory.md`
- `docs/testing.md`
- `docs/developer-guide.md`
- `docs/backlog.md`

## Backlog rules
- Track future work in `docs/backlog.md`.
- On completion, remove completed items from backlog.
- Add obvious follow-ups under `Agent suggested next steps`.

## New feature workflow
1. Run `git status --short`. If non-empty, stop and report dirty files; do not edit.
2. Create branch from `main`: `git switch -c feature/<NNNNN>-<feature_name>`, where `NNNNN` is the plan number.
3. Add a `CHANGELOG.md` entry under `Unreleased`.
4. Create the plan in `docs/plans/` as `docs/plans/AGENTS.md` directs: a numbered file allocated with `.agents/skills/allocating-report-numbers/allocate-report.sh`. Plans are never renamed or moved; their status lives in the front matter.
5. Plan must include scope, TDD unit tests, integration tests, config/secrets impact, observability, docs, coverage target, risks, and the `docs/release/vX.Y.Z.md` draft when the work releases a version.
6. The user approves the plan. Work starts only after approval.
7. Write failing unit tests first; mock external interfaces.
8. Implement minimal code; add integration tests; update docs/config/backlog.
9. Commit each plan step as `build: complete PLAN-<NNNNN>-STEP-<NN> - <title>` after its verification passes, and keep the plan's Builder Work Log current.
10. Run `just ci` and ensure coverage >= 80%.
11. Report changed files, tests run, coverage result, docs updated, and backlog updates.

## Release workflow
1. Use SemVer `vMAJOR.MINOR.PATCH`. If no version is given, increment PATCH. The plan's documentation step bumps `Cargo.toml` and `Cargo.lock` and writes the `docs/release/vX.Y.Z.md` draft.
2. Agent completes the plan: all checks pass, coverage >= 80%, branch CI passes. Agent hands off with the evidence.
3. User reviews and approves the work.
4. Agent finalises the release in one commit, `release: vX.Y.Z - <top feature>`:
   - `CHANGELOG.md`: rename `Unreleased` to `vX.Y.Z - <UTC timestamp of this commit>` and add a fresh `Unreleased` above it.
   - `docs/release/vX.Y.Z.md`: remove the draft line; name the date, plan, and PR, not a commit hash. Use absolute links pinned to the tag (`https://github.com/joelee/cli-bot/blob/vX.Y.Z/...`), because the GitHub release page cannot resolve relative ones.
   - `README.md` and other docs: remove pre-release wording such as "being prepared".
   - Run `scripts/check-release-tag.sh vX.Y.Z`, and suggest the PR title and description.
5. User verifies, pushes the branch, and opens a PR to `main`; there is no `develop` branch. `main` accepts only PRs whose checks pass. Agent debugs PR CI failures on the branch. User gets the PR approved and merged.
6. User pulls `main`, tags the merge commit, and pushes the tag. The Release workflow checks the tag and release records and builds binaries. After the user approves the pending `release` deployment, it publishes to crates.io and then creates the GitHub release from `docs/release/vX.Y.Z.md`. Do not create the release by hand. The agent never pushes, tags, or publishes. A published crate version can only be yanked, never replaced.
7. User checks the release page and crates.io. Agent helps debug a failed release run.
8. Once crates.io has the version, agent runs `scripts/update-homebrew-formula.sh vX.Y.Z` (tap at `../homebrew-oss`) and commits the formula change on a branch in the tap, never on its `main`. User pushes it and merges once the tap's macOS formula test passes.

## Known deviations
Rules above that the code does not meet yet. Do not make them worse; each is tracked in `docs/backlog.md` under `Agent suggested next steps`.
- **Config lookup order:** only `--config`, `~/.config/cli-bot/cli-bot.toml`, and `/etc/cli-bot.toml` are read, and the file is named `cli-bot.toml`.
- **Logging:** there is no logging framework; `--verbose` prints `[verbose]` lines with `eprintln!`.
- **Rustdoc:** most public items have no rustdoc.

## Project layout
- `src/lib.rs`: CLI definition and request flow (`run`), `--check`, `--models-benchmark`, interactive mode
- `src/config.rs`: TOML config loading and lookup
- `src/environment.rs`: OS, distro, and package-manager detection
- `src/llm.rs`: Ollama request/response handling and prompts
- `src/planner.rs`: structured command plan types and safety checks
- `src/session.rs`: session memory storage and prompt context
- `src/shell.rs`: shell execution
- `src/output.rs`: ANSI styling
- `tests/mock_ollama.rs`: integration tests against a mock Ollama server
- `justfile`: task runner for every check
- `.githooks/pre-commit`: runs `just check`
- `deny.toml`: `cargo deny` policy behind `just audit`
- `scripts/check-links.sh`: Markdown link check behind `just links`
- `scripts/check-release-tag.sh`: tag and release-record check run by the Release workflow
- `scripts/update-homebrew-formula.sh`: points the Homebrew tap's formula at a published release
- `.github/workflows/release.yml`: tag-driven pipeline that builds binaries, publishes to crates.io after the `release` environment is approved, and creates the GitHub release
- `docs/`: project documentation; `docs/plans/`, `docs/ideas/`, `docs/reviews/`, and `docs/release/` hold numbered records and release notes
