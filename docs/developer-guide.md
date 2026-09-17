# Developer Guide

How to build, check, and change cli-bot. The rules themselves are in
[`AGENTS.md`](../AGENTS.md); this guide is the how-to.

## Setup

You need [rustup](https://rustup.rs) and [`just`](https://just.systems)
(`cargo install --locked just`, or your package manager).

```bash
just setup          # llvm-tools-preview, cargo-llvm-cov, cargo-nextest
just install-hooks  # run `just check` before every commit
```

`rust-toolchain.toml` pins the toolchain (1.98.1 with `rustfmt`, `clippy`,
and `llvm-tools-preview`); rustup installs it on first use. When you change
the version, change it in the three files under `.github/workflows/` too.

For `just release`, copy `.env.sample` to `.env` and set
`HOMEBREW_FORMULA_FILE`. Git ignores `.env`.

## Recipes

`just` with no arguments lists them.

| Recipe | What it does |
|---|---|
| `just default` | Lists the recipes. |
| `just setup` | Installs `llvm-tools-preview`, `cargo-llvm-cov`, and `cargo-nextest` when missing. |
| `just fmt` | Formats all code in place. |
| `just fmt-check` | Fails if any file is not formatted. |
| `just lint` | Runs clippy on all targets; a warning is an error. |
| `just scripts-check` | Checks the syntax of `scripts/*.sh` and `.githooks/*` with `bash -n`. |
| `just test` | Runs the unit tests and the mocked-Ollama integration tests. |
| `just test-junit` | Runs the tests with nextest and writes `target/test-results/unit-tests.xml`. |
| `just coverage` | Runs the tests under `cargo llvm-cov` and fails below 80% line coverage. |
| `just coverage-html` | Writes the HTML coverage report to `target/llvm-cov/html/index.html`. |
| `just coverage-lcov` | Writes `target/coverage/lcov.info` for Codecov. |
| `just build` | Builds from the lockfile (`--locked`). |
| `just package` | Packages the crate and verifies that the package builds. |
| `just package-list` | Lists the files that go into the crate. |
| `just lint-workflows` | Runs `actionlint` on the GitHub workflows, from its Docker image when it is not installed. |
| `just check` | `fmt-check`, `lint`, `scripts-check`, `test`, `coverage`, `build`, `package`. Run this before you call work complete. |
| `just ci` | `check`, then `lint-workflows`. |
| `just install-hooks` | Points Git at `.githooks`, so each commit runs `just check`. |
| `just run <args>` | Runs the CLI, for example `just run -n list files`. |
| `just release vX.Y.Z` | Runs `scripts/release.sh`: checks, publishes to crates.io, updates the Homebrew formula. The project owner runs this, never an agent. |

`just check` takes about 30 seconds once the build is warm, because
`coverage` runs the tests a second time with instrumentation. Do not skip
the hook with `--no-verify`.

## Tests

- Unit tests live in `#[cfg(test)]` modules under `src/`. They must not
  reach the network, and they pass paths, environment values, and clocks in
  as arguments rather than reading the real ones.
- Integration tests live in `tests/mock_ollama.rs`. `MockOllamaServer`
  answers on a local port with the replies you queue, in order, and records
  each request. Queue exactly as many replies as the test makes requests;
  the server thread fails the test otherwise. `ConfigOptions` writes a
  `cli-bot.toml` into a temporary folder; use
  `ConfigOptions::with_session(&dir)` so session files never land in your
  real state folder.
- `sample_cli` is quiet, dry-run, and without session memory.
  `session_cli` prints normally, keeps session memory on, and executes the
  selected command.
- A test must never reach a `dialoguer` prompt. `cargo test` started from a
  terminal has a TTY, so the prompt would open and the test would hang. Use
  one command per plan, or `auto_select_best`, and never execute a command
  that needs confirmation.
- The only commands a test may execute are harmless ones such as
  `printf ok`.

See [Testing](testing.md) for coverage reports and CI.

## Workflow

Work follows `AGENTS.md` § New feature workflow:

1. Start from a clean tree on `develop` and create
   `feature/<NNNNN>-<name>`.
2. Write the plan in `docs/plans/` (see `docs/plans/AGENTS.md`); allocate
   its number with
   `.agents/skills/allocating-report-numbers/allocate-report.sh docs/plans <Name>.md`.
3. The owner approves the plan.
4. Add the `Unreleased` changelog entry, then work test-first, one commit
   per plan step, keeping the plan's Builder Work Log current.
5. Finish with `just ci`, update `docs/backlog.md`, and hand over. The
   owner pushes and opens the pull request into `develop`.

Ideas (`docs/ideas/`) and code reviews (`docs/reviews/`) are numbered,
immutable records with their own `AGENTS.md`.

## CI

| Workflow | Runs on | Does |
|---|---|---|
| `release-checks.yml` | Pull requests; pushes to `main` and `feature/**`; `v*` tags | `just check`, `just package-list`; on tags, uploads the `.crate` file |
| `unit-coverage.yml` | Pull requests; pushes to `main` | `just coverage-lcov`, `just coverage-html`, `just test-junit`; uploads to Codecov |
| `coverage-pages.yml` | Pushes to `main` | `just coverage-html`; publishes the report to GitHub Pages |
