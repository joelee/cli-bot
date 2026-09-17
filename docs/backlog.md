# Backlog

Future work not covered by an active plan. Completed items are removed.
Items here are candidates, not release promises. Shell command generation
stays the primary product behaviour.

## Release status

- **0.3.1 is not released.** crates.io, `main`, and the newest tag are at
  0.3.0. `develop` carries 0.3.1 (interactive mode) plus the `Unreleased`
  changelog section (PLAN-00001). The owner decides whether these ship
  together as 0.3.1 or as 0.3.1 and 0.3.2.

## Candidate features

- **External dictionary and language tools.** Use dictionary, spelling, or
  thesaurus tools when available, to improve non-command requests such as
  spelling, definitions, and synonyms.
- **Better non-command result types.** Go beyond plain text and
  clarification: concise help, explanations, or command comparisons, without
  executing anything.
- **Safer state-changing command classification.** A middle tier for
  commands that change state without being destructive, such as package
  installs, service restarts, and system updates.
- **Platform-aware package helpers.** Richer prompt examples per platform,
  and stronger handling of package search, upgrade, and uninstall.
- **Session storage.** Move from JSON files to SQLite if captured command
  output becomes heavily used; finer-grained pruning than `retention_days`.
  See [Session Memory](session-memory.md).

## Agent suggested next steps

### Rules the code does not meet yet

These are the `Known deviations` of `AGENTS.md`. Each changes behaviour or
is large, so each needs its own plan.

- **Config lookup order.** Read `--config`, then `CLI_BOT_CONFIG_FILE`,
  `$XDG_CONFIG_HOME/cli-bot/config.toml`, `~/.config/cli-bot/config.toml`,
  `/etc/cli-bot/config.toml`, and `./config.toml`. Today only `--config`,
  `~/.config/cli-bot/cli-bot.toml`, and `/etc/cli-bot.toml` are read. Keep
  reading the old `cli-bot.toml` paths so existing installs still work.
- **Structured logging.** Replace the `[verbose]` `eprintln!` lines with a
  logger that has the levels Error, Warning, Info, Verbose, and Debug,
  timestamps, and a component name. Do not log full prompts or command
  output above Debug.
- **Rustdoc for the public API.** `Cli`, `run`, `OutputStyler`, and
  `ColorMode` are public; most items have no rustdoc.
- **Release workflow in CI.** Publish to crates.io from a tag-triggered
  workflow with an approval gate, keep release notes in
  `docs/release/vX.Y.Z.md`, and reduce `scripts/release.sh` to the Homebrew
  formula update.

### Fixes found in the handover review (2026-09-17)

- **Interactive mode exits on the first planner error.**
  `handle_interactive_request_error` in `src/lib.rs` survives only errors
  whose text contains `command exited with status`; an Ollama timeout or
  malformed plan ends the session. Cancellation is also detected by matching
  error text. Use typed errors, and keep the prompt open after a planner
  error.
- **HTTP timeout.** `OllamaClient` uses reqwest's default 30-second timeout,
  which a cold load of a large model can exceed. Make it configurable under
  `[ollama]` with a longer default.
- **Session file names are not stable across toolchains.**
  `stable_path_hash` in `src/session.rs` uses `DefaultHasher`, whose output
  may change between Rust releases and then orphans working-directory
  sessions. Use a fixed hash, and keep finding files written under the old
  name.
- **Safety patterns are plain substrings.** `rm -fr`, `rm  -rf` (two
  spaces), and `rm -r -f` do not match `rm -rf`. Normalise whitespace and
  flag order, or match on parsed tokens. `chmod -r` and `chmod -R` in the
  default list are the same pattern, because matching ignores case.
- **A failed command is not saved in the session.** `shell::execute`
  returns an error on a non-zero exit, so the turn is never stored and a
  follow-up such as "why did that fail" has no context.
- **Duplicated helpers.** `is_executable_file` exists in `src/config.rs`
  and `src/environment.rs`; byte-limited truncation exists in
  `src/shell.rs` and `src/session.rs`.
- **`run_single_request` takes 13 arguments.** Group them into a request
  context struct; this also makes the confirmation and selection prompts
  injectable, so they can be tested without a terminal.

### Tooling

- **Coverage of terminal-bound code.** The `dialoguer` prompts, the
  interactive loop, and `src/main.rs` are not covered (88.93% overall at
  PLAN-00001). Injectable prompts (above) would close the gap.
- **macOS CI job.** cli-bot supports macOS, but CI runs on Linux only.
- **Supply-chain audit.** Add `cargo deny` with a `deny.toml`, as passalong
  has, and a `just audit` recipe.
- **Stale benchmark reports.** `docs/models-benchmark-report.md` and
  `docs/models-benchmark-report1.md` are two generated reports; keep one, or
  move them under `docs/benchmarks/` with dated names.
