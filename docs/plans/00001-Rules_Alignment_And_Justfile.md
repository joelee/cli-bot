---
title: "Delivery Plan 00001: Rules Alignment And Justfile"
aliases:
  - "Plan 00001"
tags:
  - delivery-plan
  - implementation
  - claude-code
type: delivery-plan
plan_id: "PLAN-00001"
plan_status: approved              # draft | approved | cancelled
plan_kind: initial                 # initial | superseding
created_at: "2026-09-17T16:22:20Z"
approved_at: "2026-09-17T17:27:35Z"
planner_agent: "Claude Code"
planner_model: "anthropic/claude-fable-5-1"
triggered_by: user                 # user | agent:<agent-name>
request_kind: direct               # idea | review | idea-and-review | direct | unplanned-query
repository: "joelee/cli-bot"
baseline_branch: "feature/00001-rules-alignment"
baseline_commit: "fa272c8b2d09b0e28a493804fb4e29d895a6d1cc"
source_ideas: []
source_reviews: []
previous_plan: null
requirements_count: 10
steps_count: 8
acceptance_criteria_count: 15
blocking_decisions: 0
build_ready: true
web_research_used: false
confidence: high                  # high | medium | low

# Builder-maintained front matter. Builder may update only these keys after
# explicit user approval; Delivery Planner initializes them.
implementation_status: completed # not-started | in-progress | blocked | completed | abandoned
builder_agent: "Claude Code"
builder_model: "anthropic/claude-fable-5-1"
execution_branch: "feature/00001-rules-alignment"
execution_started_at: "2026-09-17T17:27:54Z"
execution_updated_at: "2026-09-17T17:47:31Z"
execution_completed_at: "2026-09-17T17:47:31Z"
current_step: "PLAN-00001-STEP-08"
---

# Delivery Plan 00001: Rules Alignment And Justfile

> [!abstract] Plan status: `draft`
> cli-bot adopts the working rules of `../passalong`: an adapted root
> `AGENTS.md`, a `justfile` as the only task runner for local checks, Git
> hooks, and CI, a pinned toolchain, and an enforced line-coverage gate of
> 80% (today 76.00%). The `cli-bot` binary does not change. Seven decisions
> were resolved by the planner (section 7); none blocks, and the plan waits
> for user approval.

## 1. Objective and outcome

The user handed cli-bot to the agent on 2026-09-17 and asked that the rules
of `../passalong` apply here, and that the project move to a `justfile`.
`docs/ideas/AGENTS.md`, `docs/plans/AGENTS.md`, `docs/reviews/AGENTS.md`, and
`.agents/skills/allocating-report-numbers/` are already byte-identical to
passalong's. The root `AGENTS.md`, the tooling, and the quality gates are
not.

After this plan:

- `AGENTS.md` carries passalong's rules, adapted to a single-crate CLI, and
  keeps cli-bot's own product rules.
- `just check` is the one command that local work, the pre-commit hook, and
  CI all run: format check, lint, script syntax check, tests, coverage gate,
  locked build, and package verification.
- Line coverage is at least 80% and a lower figure fails `just check` and CI.
- The toolchain is pinned, `.env.sample` exists, and `docs/backlog.md` and
  `docs/developer-guide.md` exist.
- Every rule cli-bot does not yet meet is named in `AGENTS.md` and tracked in
  `docs/backlog.md`, so no rule is silently broken.

## 2. Source traceability

| Requirement | Source | Source location | Interpretation |
|---|---|---|---|
| PLAN-00001-REQ-01 | User; external repository | User instruction 2026-09-17 "adopt the rules in `../passalong`"; `../passalong/AGENTS.md` (all sections) | Root `AGENTS.md` is rewritten from passalong's, adapted |
| PLAN-00001-REQ-02 | User; external repository | User instruction 2026-09-17 "migrate the project to use `justfile`"; `../passalong/justfile` | A `justfile` is the single task runner |
| PLAN-00001-REQ-03 | External repository; repository | `../passalong/AGENTS.md` § Commands ("Pre-commit must enforce the same checks"); `.githooks/pre-commit`, `.pre-commit-config.yaml`, `scripts/verify.sh` | Hooks call `just check`; scripts that `just` replaces are removed |
| PLAN-00001-REQ-04 | External repository; repository | `../passalong/.github/workflows/ci.yml`; `.github/workflows/*.yml` | CI runs `just` recipes and enforces the coverage gate |
| PLAN-00001-REQ-05 | External repository | `../passalong/rust-toolchain.toml` | The toolchain is pinned |
| PLAN-00001-REQ-06 | External repository | `../passalong/AGENTS.md` § Non-negotiables ("line coverage >= 80%") | Coverage rises from 76.00% to at least 80% |
| PLAN-00001-REQ-07 | External repository; repository | `../passalong/AGENTS.md` § Config and secrets; `scripts/release.sh` lines 6-11, 92 | `.env.sample` documents `HOMEBREW_FORMULA_FILE` |
| PLAN-00001-REQ-08 | External repository; repository | `../passalong/AGENTS.md` § Docs to maintain, § Backlog rules; `docs/roadmap.md` | `docs/backlog.md` and `docs/developer-guide.md` exist; references are current |
| PLAN-00001-REQ-09 | External repository | `../passalong/AGENTS.md` § New feature workflow, step 3 | `CHANGELOG.md` has an `Unreleased` entry |
| PLAN-00001-REQ-10 | User; repository | Handover conversation 2026-09-17 (behaviour changes are follow-up features); `AGENTS.md` § Working Rules ("Prefer small, local changes") | No user-visible behaviour change |

## 3. Repository baseline

| Field | Value |
|---|---|
| Repository | `joelee/cli-bot` (`git@github.com:joelee/cli-bot.git`) |
| Branch | `feature/00001-rules-alignment`, created from `develop` for this plan |
| HEAD | `fa272c8b2d09b0e28a493804fb4e29d895a6d1cc` (same as `develop` and `origin/develop`) |
| Working tree at publication | Clean, apart from this plan's newly allocated file |
| Applicable instructions | `AGENTS.md`; `docs/plans/AGENTS.md`; `../passalong/AGENTS.md` (by user instruction) |

Measured on this commit on 2026-09-17 with cargo 1.98.1 and cargo-llvm-cov
0.9.1:

- `cargo fmt --all --check`: pass.
- `cargo clippy --all-targets --all-features -- -D warnings`: pass.
- `cargo llvm-cov --workspace --all-features --summary-only`: all tests
  pass; lines 3813, missed 915, **76.00%**. Per file: `lib.rs` 67.32%,
  `llm.rs` 77.14%, `output.rs` 76.47%, `session.rs` 82.79%,
  `environment.rs` 82.90%, `shell.rs` 88.24%, `config.rs` 93.09%,
  `planner.rs` 94.83%, `main.rs` 0.00% (8 lines).
- `Cargo.toml` version is `0.3.1`. crates.io has `0.3.0`; the newest tag is
  `v0.3.0`; `main` is at v0.3.0. Version 0.3.1 is therefore unreleased.
- `git config core.hooksPath` is unset and `pre-commit` is not installed on
  the development machine, so no hook runs locally today.
- `just` 1.58.0 is installed.

## 4. Scope

### In scope

- Root `AGENTS.md`, rewritten.
- New `justfile`, `rust-toolchain.toml`, `.env.sample`, `docs/backlog.md`,
  `docs/developer-guide.md`.
- `.githooks/pre-commit`, `.pre-commit-config.yaml`, the three workflows in
  `.github/workflows/`, `.config/nextest.toml`.
- Removal of `scripts/verify.sh`, `scripts/install-hooks.sh`,
  `scripts/coverage-unit.sh`, and `docs/roadmap.md`.
- `scripts/release.sh`: its check commands change to `just` recipes; nothing
  else in it changes.
- New tests in `tests/mock_ollama.rs` and in `#[cfg(test)]` modules.
- `CHANGELOG.md`, `README.md`, `docs/testing.md`, `docs/publishing.md`,
  `docs/crates-release.md`, and any other document that names a removed file.

### Out of scope

- The config file name and lookup order (`--config`, `CLI_BOT_CONFIG_FILE`,
  XDG, `/etc`, `./config.toml`). Recorded in the backlog.
- Structured logging with syslog levels. Recorded in the backlog.
- Rustdoc for the whole public API. Recorded in the backlog.
- A release workflow in CI that publishes to crates.io, and
  `docs/release/`. Recorded in the backlog.
- The code findings of the handover review (interactive-mode error handling,
  the 30-second HTTP timeout, `DefaultHasher` in session file names,
  substring-based safety matching, duplicated helpers). Recorded in the
  backlog.
- Releasing 0.3.1, a version bump, a tag, or a publish.
- macOS or Windows CI jobs.

## 5. Constraints and preserved decisions

- cli-bot's product rules stay in `AGENTS.md`: behaviour is driven by
  `cli-bot.toml`; risky commands need explicit approval; the interactive
  selection flow stays intact; prefer small, local changes.
- `docs/ideas/AGENTS.md`, `docs/plans/AGENTS.md`, `docs/reviews/AGENTS.md`,
  and `.agents/` are not edited.
- Tests must not call a real Ollama service. They use `MockOllamaServer` in
  `tests/mock_ollama.rs`.
- A test may execute only a harmless command through the shell, such as
  `printf`. No test may run a network command such as `ping`.
- No new runtime dependency. Dev-dependencies are allowed only if a test
  cannot be written without one, and each is recorded in the work log.
- The agent does not push, tag, or publish. The user does.
- Git hooks are never bypassed.

## 6. Assumptions

None. Unresolved matters are recorded as decisions and block approval when
material.

## 7. Decisions and blockers

All were resolved by the planner from the repository and the handover
conversation. The user may change any of them before approval; none blocks.

| ID | Decision or blocker | Resolution | Owner | Status |
|---|---|---|---|---|
| D-01 | passalong requires Docker-compatible build and runtime | Not adopted. cli-bot runs commands in the user's own shell and has no container use. `AGENTS.md` omits the rule | Planner | Resolved |
| D-02 | passalong's `AGENTS.md` names plans `docs/plans/<timestamp>-<name>.md` and moves finished plans to `docs/plans/done/`; `docs/plans/AGENTS.md` and passalong's actual plans use five-digit numbers and never move | `AGENTS.md` follows `docs/plans/AGENTS.md`: numbered plans, never moved; status lives in the front matter | Planner | Resolved |
| D-03 | Version | No bump. 0.3.1 is unreleased, so this work is recorded under `Unreleased` in `CHANGELOG.md`. Whether it ships as 0.3.1 or 0.3.2 is decided at release time | Planner | Resolved |
| D-04 | Rules cli-bot cannot meet without changing behaviour (config lookup order, logging) | `AGENTS.md` states them as the rule for new work, lists today's gaps under `Known deviations`, and `docs/backlog.md` tracks each gap | Planner | Resolved |
| D-05 | Release workflow | `AGENTS.md` describes cli-bot's present process (the user runs `scripts/release.sh`, which publishes and updates the Homebrew formula) with passalong's approval order. Publishing from CI goes to the backlog | Planner | Resolved |
| D-06 | Coverage tooling | `cargo llvm-cov` replaces the hand-written `scripts/coverage-unit.sh`. It produces the summary, the HTML report, and `lcov.info` from one run | Planner | Resolved |
| D-07 | Branch flow | The feature branch starts from `develop` and its pull request targets `develop`, as PR #7 did. `develop` to `main` stays the user's release step | Planner | Resolved |

## 8. Affected architecture and components

No source module changes its behaviour. The change is to the tooling around
the crate.

| Component | Today | After |
|---|---|---|
| Local checks | `scripts/verify.sh`: fmt, clippy, test, `bash -n`, `cargo package --allow-dirty` | `just check`: `fmt-check`, `lint`, `scripts-check`, `test`, `coverage`, `build`, `package` |
| Coverage | `scripts/coverage-unit.sh` (hand-merged profiles, no gate) | `just coverage` (gate), `just coverage-html`, `just coverage-lcov` |
| Git hook | `.githooks/pre-commit` calls `scripts/verify.sh`; installed by `scripts/install-hooks.sh` | Calls `just check`; installed by `just install-hooks` |
| pre-commit framework | `.pre-commit-config.yaml` calls `scripts/verify.sh` | Calls `just check` |
| `release-checks.yml` | Inline cargo commands | `just check`, `just package-list` |
| `unit-coverage.yml` | `coverage-unit.sh`, `cargo nextest run --lib` | `just coverage-lcov`, `just coverage-html`, `just test-junit` |
| `coverage-pages.yml` | `coverage-unit.sh`, publishes `target/coverage/html` | `just coverage-html`, publishes `target/llvm-cov/html` |
| Toolchain | Whatever `stable` is | `rust-toolchain.toml` pins 1.98.1 |
| Future work | `docs/roadmap.md` (two listed items already shipped) | `docs/backlog.md` |

## 9. Requirement catalogue

### PLAN-00001-REQ-01 — Adapted root AGENTS.md

- **Requirement:** `AGENTS.md` has these sections, taken from
  `../passalong/AGENTS.md` and adapted: Scope, Purpose, Stack, Product rules,
  Non-negotiables, Commands, Config and secrets, Observability, Docs to
  maintain, Backlog rules, New feature workflow, Release workflow, Known
  deviations, Project layout. Commands are given as `just` recipes with the
  cargo command each wraps. Decisions D-01, D-02, D-04, D-05, and D-07 apply.
- **Rationale:** the user asked for passalong's rules here.
- **Source:** user instruction; `../passalong/AGENTS.md`.
- **Acceptance evidence:** AC-01.

### PLAN-00001-REQ-02 — justfile as the single task runner

- **Requirement:** a `justfile` at the repository root with
  `set shell := ["bash", "-euo", "pipefail", "-c"]` and these recipes, each
  with a one-line comment: `default` (lists recipes), `setup`, `fmt`,
  `fmt-check`, `lint`, `scripts-check`, `test`, `test-junit`, `coverage`,
  `coverage-html`, `coverage-lcov`, `build`, `package`, `package-list`,
  `lint-workflows`, `check`, `ci`, `install-hooks`, `run *ARGS`,
  `release TAG`. `check` depends on `fmt-check lint scripts-check test
  coverage build package`. `ci` depends on `check lint-workflows`.
- **Rationale:** one entry point that local work, hooks, and CI share.
- **Source:** user instruction; `../passalong/justfile`.
- **Acceptance evidence:** AC-02, AC-03.

### PLAN-00001-REQ-03 — Hooks call just; replaced scripts are removed

- **Requirement:** `.githooks/pre-commit` and `.pre-commit-config.yaml` run
  `just check`. `just install-hooks` sets `core.hooksPath` to `.githooks`.
  `scripts/verify.sh`, `scripts/install-hooks.sh`, and
  `scripts/coverage-unit.sh` are removed with `git rm`. `scripts/release.sh`
  runs `just check` in place of its own fmt, clippy, test, and package
  commands.
- **Rationale:** two ways to run the checks would drift apart.
- **Source:** `../passalong/AGENTS.md` § Commands.
- **Acceptance evidence:** AC-05, AC-06.

### PLAN-00001-REQ-04 — CI runs just and enforces coverage

- **Requirement:** each workflow pins the toolchain with
  `dtolnay/rust-toolchain@master` and `toolchain: 1.98.1`, installs `just`
  and `cargo-llvm-cov` (and `cargo-nextest` where used) with
  `taiki-e/install-action@v2`, and runs `just` recipes. Codecov still gets
  `lcov.info` and the JUnit file; GitHub Pages still gets the HTML report.
  `release-checks.yml` also runs on pushes to `feature/**`.
- **Rationale:** the gate must hold in CI, not only on one machine.
- **Source:** `../passalong/.github/workflows/ci.yml`.
- **Acceptance evidence:** AC-07, AC-08.

### PLAN-00001-REQ-05 — Pinned toolchain

- **Requirement:** `rust-toolchain.toml` with `channel = "1.98.1"`,
  `components = ["rustfmt", "clippy", "llvm-tools-preview"]`,
  `profile = "minimal"`.
- **Rationale:** the same compiler, formatter, and lints everywhere.
- **Source:** `../passalong/rust-toolchain.toml`.
- **Acceptance evidence:** AC-09.

### PLAN-00001-REQ-06 — Line coverage of at least 80%

- **Requirement:**
  `cargo llvm-cov --workspace --all-features --fail-under-lines 80 --summary-only`
  exits 0.
- **Rationale:** passalong's coverage gate.
- **Source:** `../passalong/AGENTS.md` § Non-negotiables.
- **Acceptance evidence:** AC-04.

### PLAN-00001-REQ-07 — .env.sample

- **Requirement:** `.env.sample` names `HOMEBREW_FORMULA_FILE` with a safe
  example path and a comment saying `scripts/release.sh` reads it. `.env`
  stays in `.gitignore`.
- **Rationale:** the one variable the repository reads from `.env` is
  undocumented.
- **Source:** `../passalong/AGENTS.md` § Config and secrets;
  `scripts/release.sh`.
- **Acceptance evidence:** AC-10.

### PLAN-00001-REQ-08 — Backlog, developer guide, current references

- **Requirement:** `docs/backlog.md` replaces `docs/roadmap.md` (`git mv`,
  then rewrite). It keeps the candidate features that have not shipped,
  drops "Mocked Planner Integration Tests" and "Session Logging Or History"
  (both shipped), and adds, under `Agent suggested next steps`, every item
  listed as out of scope in section 4. `docs/developer-guide.md` documents
  setup, every `just` recipe, the hook, the coverage reports, and the
  plan-based workflow. No tracked document other than `CHANGELOG.md` and
  files under `docs/plans/` names a removed file.
- **Rationale:** passalong's documentation set and backlog rules.
- **Source:** `../passalong/AGENTS.md` § Docs to maintain, § Backlog rules.
- **Acceptance evidence:** AC-05, AC-11, AC-12.

### PLAN-00001-REQ-09 — Changelog entry

- **Requirement:** `CHANGELOG.md` has a `## [Unreleased]` section above
  `## [0.3.1]` that describes this work.
- **Rationale:** passalong's feature workflow, step 3.
- **Source:** `../passalong/AGENTS.md` § New feature workflow.
- **Acceptance evidence:** AC-13.

### PLAN-00001-REQ-10 — No user-visible behaviour change

- **Requirement:** the `cli-bot` binary behaves as it does at the baseline
  commit. `Cargo.toml` keeps version `0.3.1` and its `[dependencies]` table.
  A change to non-test code under `src/` is allowed only when a test cannot
  be written without it; it must not change behaviour, and it is recorded in
  the work log under `Deviations and blockers` before it is made.
- **Rationale:** behaviour changes are separate features with their own
  plans.
- **Source:** handover conversation; `AGENTS.md` § Working Rules.
- **Acceptance evidence:** AC-14, AC-15.

## 10. Delivery strategy

The coverage work comes before the `justfile`, because `just check` includes
the coverage gate and would fail from the moment it exists otherwise. Until
STEP-04, checks run as plain cargo commands.

Coverage is raised mostly through `tests/mock_ollama.rs`. Its `sample_cli`
helper sets `quiet: true`, `no_session: true`, and `dry_run: true`, so the six
existing tests skip most of `run_single_request`, every verbose branch in
`lib.rs` and `llm.rs`, and all of `run_models_benchmark`. New tests that turn
those flags off go through the real application flow against the mock server,
which is what cli-bot's own rules prefer. Paths that need a terminal (the
`dialoguer` prompts and the interactive loop) stay uncovered; they are not
needed to reach 80%. The gate needs about 155 more covered lines; the verbose,
session, and benchmark paths hold more than 400 uncovered lines.

Each step ends in one commit, `build: complete PLAN-00001-STEP-NN - <title>`,
made only after the step's verification passes. From STEP-05 on, the commit
hook itself runs `just check`.

## 11. Detailed implementation steps

### PLAN-00001-STEP-01 — Baseline and changelog entry

- **Objective:** start the work log and record the change.
- **Requirements:** `PLAN-00001-REQ-09`
- **Depends on:** None
- **Affected components:** `CHANGELOG.md`; this plan's Builder fields
- **Preconditions:** plan approved; worktree clean; on
  `feature/00001-rules-alignment`.
- **Test or evidence first:** run the three baseline commands of section 3
  and record the results in the work log. Save `cargo run -- --help` output
  to a file outside the repository for AC-14.
- **Implementation tasks:**
  1. Set the Builder front-matter fields.
  2. Add `## [Unreleased]` above `## [0.3.1]` with `Changed` entries for the
     rules, the `justfile`, the coverage gate, and the pinned toolchain.
- **Documentation/configuration/operations:** `CHANGELOG.md` only.
- **Verification:** `grep -n '^## \[' CHANGELOG.md | head -2` shows
  `Unreleased` first, then `0.3.1`.
- **Completion criteria:** the verification passes; the step is committed.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** the baseline differs from section 3 (tests
  fail, or coverage is not 76.00% ± 0.5).

### PLAN-00001-STEP-02 — Pin the toolchain

- **Objective:** one compiler version for every environment.
- **Requirements:** `PLAN-00001-REQ-05`
- **Depends on:** `PLAN-00001-STEP-01`
- **Affected components:** `rust-toolchain.toml` (new)
- **Preconditions:** toolchain 1.98.1 is installed or can be installed by
  rustup.
- **Test or evidence first:** `rustup show active-toolchain` before the
  change, recorded.
- **Implementation tasks:**
  1. Create `rust-toolchain.toml` as REQ-05 specifies.
  2. Run fmt check, clippy, and tests under the pinned toolchain.
- **Documentation/configuration/operations:** documented in STEP-08.
- **Verification:** `rustc --version` prints `rustc 1.98.1`;
  `cargo fmt --all -- --check`, clippy with `-D warnings`, and
  `cargo test --all-targets --all-features` pass.
- **Completion criteria:** all verification commands exit 0.
- **Rollback or recovery:** delete the file.
- **Builder stop conditions:** the pinned toolchain raises a lint or format
  difference that needs a change to non-test code.

### PLAN-00001-STEP-03 — Raise line coverage to at least 80%

- **Objective:** meet the gate with tests of real behaviour.
- **Requirements:** `PLAN-00001-REQ-06`, `PLAN-00001-REQ-10`
- **Depends on:** `PLAN-00001-STEP-02`
- **Affected components:** `tests/mock_ollama.rs`; `#[cfg(test)]` modules in
  `src/lib.rs`, `src/llm.rs`, `src/output.rs`, `src/environment.rs`,
  `src/session.rs`, `src/config.rs`
- **Preconditions:** STEP-02 committed.
- **Test or evidence first:** run
  `cargo llvm-cov --workspace --all-features --fail-under-lines 80 --summary-only`
  and record that it fails at about 76%. Each new test is written to assert
  behaviour (captured requests, saved session files, written report files,
  returned errors), not only to execute lines.
- **Implementation tasks:**
  1. Extend the helpers in `tests/mock_ollama.rs` so a test can set `quiet`,
     `verbose`, `dry_run`, `no_session`, `benchmark`, `print_plan`,
     `auto_select_best`, and a session storage directory in a temporary
     folder.
  2. Add integration tests for: executing a harmless command (`printf ok`)
     and saving the session turn with its exit status; a dry run that saves
     the turn as not executed; an unresolved request that saves its text
     response; a second request whose prompt carries the first turn as
     session context; auto-selecting the recommended command from two
     choices; `--verbose` on both the chat and the generate endpoint;
     `--benchmark` and `--print-plan`; `--check` when the model is missing
     and when the service returns an error; `--models-benchmark` writing a
     Markdown report to a file, including one model that fails; malformed
     planner JSON; an empty command list; `--session-clear`.
  3. Add unit tests for the remaining cheap gaps: `OutputStyler` tones and
     `ColorMode::Never`/`Always`; the `as_str` methods and the unknown-distro
     candidate list in `environment.rs`; config path resolution with a
     temporary home directory; `PlannedCommand::label`; session name
     validation and `truncate_to_bytes` edge cases.
  4. Re-measure after each group and stop adding tests once the gate passes
     with at least one percentage point to spare.
- **Documentation/configuration/operations:** `docs/testing.md` is updated in
  STEP-08.
- **Verification:**
  `cargo llvm-cov --workspace --all-features --fail-under-lines 80 --summary-only`
  exits 0; clippy with `-D warnings` passes;
  `git diff fa272c8 -- src/ | grep '^[-+]' ` shows changes only inside
  `#[cfg(test)]` modules, or every other change is recorded as a deviation.
- **Completion criteria:** TOTAL line coverage is at least 81.00%.
- **Rollback or recovery:** `git revert` the step commit; tests only.
- **Builder stop conditions:** the gate cannot be met without changing
  behaviour or adding a runtime dependency; a new test is flaky in three
  consecutive runs; a test would need a real network or Ollama call.

### PLAN-00001-STEP-04 — Add the justfile

- **Objective:** one entry point for every check.
- **Requirements:** `PLAN-00001-REQ-02`
- **Depends on:** `PLAN-00001-STEP-03`
- **Affected components:** `justfile` (new); `.config/nextest.toml`
- **Preconditions:** `just` is installed.
- **Test or evidence first:** run `scripts/verify.sh` once more and record
  that it passes, so `just check` can be compared with it.
- **Implementation tasks:**
  1. Write the `justfile` with the recipes of REQ-02. Recipes wrap:
     `fmt-check` → `cargo fmt --all -- --check`; `lint` →
     `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
     `test` → `cargo test --workspace --all-targets --all-features`;
     `coverage` → the gate command of REQ-06; `coverage-html` →
     `cargo llvm-cov --workspace --all-features --html`; `coverage-lcov` →
     `cargo llvm-cov --workspace --all-features --lcov --output-path target/coverage/lcov.info`;
     `build` → `cargo build --workspace --all-features --locked`; `package` →
     `cargo package --allow-dirty`; `package-list` →
     `cargo package --list --allow-dirty`; `scripts-check` → `bash -n` on
     every file in `scripts/` and `.githooks/`; `lint-workflows` →
     `actionlint`, or its Docker image when it is not installed; `setup` →
     installs `llvm-tools-preview`, `cargo-llvm-cov`, and `cargo-nextest`
     when missing; `test-junit` → `cargo nextest run --all-targets`;
     `run *ARGS` → `cargo run -- {{ARGS}}`; `release TAG` →
     `scripts/release.sh {{TAG}}`.
  2. Run `just test-junit`, find where the JUnit file is written, and set
     the path in `.config/nextest.toml` so that it is
     `target/test-results/unit-tests.xml`, the path the workflow uploads.
- **Documentation/configuration/operations:** documented in STEP-08.
- **Verification:** `just --list` names all 20 recipes; `just check` exits 0;
  `just coverage-html` writes `target/llvm-cov/html/index.html`;
  `just coverage-lcov` writes `target/coverage/lcov.info`; `just test-junit`
  writes `target/test-results/unit-tests.xml`.
- **Completion criteria:** all verification commands pass.
- **Rollback or recovery:** delete the `justfile`; `scripts/verify.sh` still
  exists at this step.
- **Builder stop conditions:** `just check` fails where `scripts/verify.sh`
  passes, for a reason other than the coverage gate.

### PLAN-00001-STEP-05 — Move the hooks to just and remove replaced scripts

- **Objective:** no second way to run the checks.
- **Requirements:** `PLAN-00001-REQ-03`
- **Depends on:** `PLAN-00001-STEP-04`
- **Affected components:** `.githooks/pre-commit`, `.pre-commit-config.yaml`,
  `scripts/verify.sh`, `scripts/install-hooks.sh`,
  `scripts/coverage-unit.sh`, `scripts/release.sh`, `justfile`
- **Preconditions:** `just check` passes.
- **Test or evidence first:** `git config core.hooksPath` before the change,
  recorded (unset at baseline).
- **Implementation tasks:**
  1. `.githooks/pre-commit` changes to the repository root and runs
     `just check`. It prints how to install `just` when it is missing.
  2. `.pre-commit-config.yaml`: hook `just-check`, `entry: just check`,
     `language: system`, `pass_filenames: false`, `always_run: true`.
  3. `install-hooks` recipe: `git config core.hooksPath .githooks`.
  4. `git rm` the three scripts.
  5. In `scripts/release.sh`, replace the four cargo check lines (fmt,
     clippy, test, package) with `just check`, and add `just` to its
     `require_command` list.
  6. Run `just install-hooks`.
- **Documentation/configuration/operations:** documented in STEP-08.
- **Verification:** `git config core.hooksPath` prints `.githooks`; the
  step's own `git commit` runs `just check` (seen in its output);
  `bash -n scripts/release.sh` passes.
- **Completion criteria:** the commit succeeds through the hook.
- **Rollback or recovery:** `git revert`; `git config --unset core.hooksPath`.
- **Builder stop conditions:** the hook fails for a reason the step cannot
  fix within its scope.

### PLAN-00001-STEP-06 — Move CI to just

- **Objective:** CI runs the same recipes and enforces the gate.
- **Requirements:** `PLAN-00001-REQ-04`
- **Depends on:** `PLAN-00001-STEP-05`
- **Affected components:** `.github/workflows/release-checks.yml`,
  `unit-coverage.yml`, `coverage-pages.yml`
- **Preconditions:** none beyond STEP-05.
- **Test or evidence first:** `just lint-workflows` on the unchanged
  workflows, recorded.
- **Implementation tasks:**
  1. In all three: pin the toolchain and install tools as REQ-04 specifies.
  2. `release-checks.yml`: the `verify` job runs `just check` and
     `just package-list`; add `feature/**` to the push branches;
     `publish-prep` runs `just package`.
  3. `unit-coverage.yml`: run `just coverage-lcov`, `just coverage-html`,
     and `just test-junit`; upload `target/llvm-cov/html/**`,
     `target/coverage/lcov.info`, and `target/test-results/unit-tests.xml`;
     keep both Codecov steps.
  4. `coverage-pages.yml`: run `just coverage-html`; publish
     `target/llvm-cov/html`.
- **Documentation/configuration/operations:** documented in STEP-08.
- **Verification:** `just lint-workflows` exits 0;
  `grep -n 'coverage-unit\|verify.sh' .github/workflows/*.yml` prints
  nothing.
- **Completion criteria:** verification passes. The GitHub run itself is
  checked in STEP-08, after the user pushes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** neither `actionlint` nor Docker is available
  to lint the workflows.

### PLAN-00001-STEP-07 — AGENTS.md and .env.sample

- **Objective:** the rules are written down in this repository.
- **Requirements:** `PLAN-00001-REQ-01`, `PLAN-00001-REQ-07`
- **Depends on:** `PLAN-00001-STEP-06`
- **Affected components:** `AGENTS.md`, `.env.sample` (new)
- **Preconditions:** re-read `../passalong/AGENTS.md`; it may have changed
  since this plan was written.
- **Test or evidence first:** `diff` of the section headings of
  `../passalong/AGENTS.md` against the list in REQ-01, recorded, so that a
  section added to passalong since planning is noticed.
- **Implementation tasks:**
  1. Rewrite `AGENTS.md` with the sections of REQ-01. Replace
     `<APP_NAME>` with `CLI_BOT` and `<app-name>` with `cli-bot`. Apply
     D-01, D-02, D-04, D-05, D-07.
  2. `Known deviations` lists: config file name and lookup order; logging;
     rustdoc coverage; release publishing outside CI; and links each to
     `docs/backlog.md`.
  3. Create `.env.sample`.
- **Documentation/configuration/operations:** this step is documentation.
- **Verification:** `grep -c '^## ' AGENTS.md` prints 14;
  `git check-ignore -q .env` exits 0; `.env.sample` contains
  `HOMEBREW_FORMULA_FILE=`.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** passalong's `AGENTS.md` now has a rule that
  conflicts with a decision in section 7.

### PLAN-00001-STEP-08 — Documentation, backlog, and final gate

- **Objective:** documents match the repository; hand over with evidence.
- **Requirements:** `PLAN-00001-REQ-08`, `PLAN-00001-REQ-10`
- **Depends on:** `PLAN-00001-STEP-07`
- **Affected components:** `docs/backlog.md`, `docs/roadmap.md`,
  `docs/developer-guide.md`, `docs/testing.md`, `docs/publishing.md`,
  `docs/crates-release.md`, `README.md`
- **Preconditions:** none beyond STEP-07.
- **Test or evidence first:**
  `git grep -n 'verify\.sh\|install-hooks\.sh\|coverage-unit\.sh\|roadmap\.md' -- ':!CHANGELOG.md' ':!docs/plans'`,
  recorded; every hit is a line to fix.
- **Implementation tasks:**
  1. `git mv docs/roadmap.md docs/backlog.md` and rewrite it as REQ-08
     specifies.
  2. Write `docs/developer-guide.md`.
  3. Fix every hit of the grep above.
  4. Run `just ci`. Compare `cargo run -- --help` with the output saved in
     STEP-01.
  5. Fill in the work log's completion summary with the coverage figure and
     hand over to the user, who pushes the branch. Then record the result of
     `gh run list --branch feature/00001-rules-alignment` for the head
     commit.
- **Documentation/configuration/operations:** this step is documentation.
- **Verification:** the grep prints nothing; `just ci` exits 0; the two
  `--help` outputs are identical; all workflow runs on the pushed head
  commit conclude `success`.
- **Completion criteria:** every acceptance criterion is checked.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a CI failure whose fix would change scope.

## 12. Cross-cutting concerns

| Area | Applicability | Planned action or reason not applicable | Step or requirement |
|---|---|---|---|
| Compatibility and APIs | Applicable | The binary, its flags, and `cli-bot.toml` do not change; `--help` is compared with the baseline | REQ-10, STEP-08 |
| Data and migration | Not applicable | No stored data or format changes | — |
| Security and privacy | Applicable | Tests run only harmless shell commands and never reach a network; `.env` stays ignored; `.env.sample` holds no secret | STEP-03, REQ-07 |
| Performance and scale | Applicable | The commit hook now also runs coverage, which adds one instrumented test run (about 30 s locally) | STEP-05 |
| Reliability and failure handling | Applicable | New tests use the in-process mock server and temporary folders; a flaky test is a stop condition | STEP-03 |
| Observability and operations | Not applicable | Logging is out of scope and tracked in the backlog | D-04 |
| Dependencies and supply chain | Applicable | No runtime dependency is added; CI tools are installed by `taiki-e/install-action` | REQ-10, REQ-04 |
| Accessibility and UX | Not applicable | No user interface changes | — |
| Documentation and release | Applicable | New and updated documents; no release | STEP-07, STEP-08 |
| Deployment and rollback | Applicable | Each step is one commit and can be reverted alone | Section 10 |

## 13. Verification strategy

| Level | Evidence or command | When | Required result |
|---|---|---|---|
| Format | `just fmt-check` | Every step from STEP-04 | Exit 0 |
| Lint | `just lint` | Every step from STEP-04 | Exit 0, no warnings |
| Unit and integration tests | `just test` | Every step from STEP-04 | All pass |
| Coverage | `just coverage` | STEP-03 onwards | Lines ≥ 80% |
| Package | `just package` | STEP-04 onwards | Exit 0 |
| Workflows | `just lint-workflows` | STEP-06, STEP-08 | Exit 0 |
| Hook | Output of `git commit` | STEP-05 onwards | Shows `just check` ran |
| CI | `gh run list` for the pushed head commit | STEP-08 | All `success` |
| Behaviour | `diff` of `--help` output | STEP-08 | No difference |

## 14. Acceptance criteria

- [ ] `PLAN-00001-AC-01` `AGENTS.md` has the 14 sections named in REQ-01;
  its Commands section names `just check`; its `Known deviations` section
  names the config lookup order, logging, rustdoc, and release publishing,
  and links to `docs/backlog.md`; the three product rules of section 5 are
  present.
- [ ] `PLAN-00001-AC-02` `just --list` shows the 20 recipes named in REQ-02.
- [ ] `PLAN-00001-AC-03` `just check` exits 0 on a clean checkout of the
  branch head.
- [ ] `PLAN-00001-AC-04` `just coverage` exits 0 and reports TOTAL line
  coverage of at least 80.00%; the recipe contains `--fail-under-lines 80`.
- [ ] `PLAN-00001-AC-05` `scripts/verify.sh`, `scripts/install-hooks.sh`,
  `scripts/coverage-unit.sh`, and `docs/roadmap.md` do not exist, and the
  grep of STEP-08 prints nothing.
- [ ] `PLAN-00001-AC-06` `.githooks/pre-commit` and
  `.pre-commit-config.yaml` both run `just check`; after
  `just install-hooks`, `git config core.hooksPath` prints `.githooks`.
- [ ] `PLAN-00001-AC-07` The three workflows contain no `cargo fmt`,
  `cargo clippy`, `cargo test`, or `cargo package` command of their own,
  and `just lint-workflows` exits 0.
- [ ] `PLAN-00001-AC-08` Every workflow run on the pushed head commit of
  `feature/00001-rules-alignment` concludes `success`.
- [ ] `PLAN-00001-AC-09` `rust-toolchain.toml` pins `1.98.1` with `rustfmt`,
  `clippy`, and `llvm-tools-preview`; `rustc --version` in the repository
  prints `1.98.1`.
- [ ] `PLAN-00001-AC-10` `.env.sample` contains `HOMEBREW_FORMULA_FILE=`
  with an example path, and `git check-ignore -q .env` exits 0.
- [ ] `PLAN-00001-AC-11` `docs/backlog.md` has an
  `Agent suggested next steps` heading and an entry for each of: config
  lookup order, structured logging, rustdoc, CI release workflow,
  interactive-mode error handling, HTTP timeout, session file-name hash,
  safety pattern matching, duplicated helpers. It has no entry for mocked
  integration tests or session history.
- [ ] `PLAN-00001-AC-12` `docs/developer-guide.md` has an entry for each of
  the 20 recipes, and `README.md` and `docs/testing.md` give `just check`
  as the verification command.
- [ ] `PLAN-00001-AC-13` `CHANGELOG.md` has `## [Unreleased]` directly above
  `## [0.3.1]`, with entries for the rules, the `justfile`, the coverage
  gate, and the toolchain.
- [ ] `PLAN-00001-AC-14` `cargo run -- --help` prints the same text as at
  the baseline commit, and every change to non-test code under `src/` is
  listed in the work log under `Deviations and blockers`.
- [ ] `PLAN-00001-AC-15` `Cargo.toml` still has `version = "0.3.1"`, and its
  `[dependencies]` table is unchanged from the baseline commit.

## 15. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation or test | Owner/step |
|---|---|---|---|---|
| 80% cannot be reached without touching terminal-bound code | Low | Medium | More than 400 uncovered lines are reachable through the mock server; stop condition if not | STEP-03 |
| `--models-benchmark` tests depend on host tools (`lspci`, `nvidia-smi`) | Medium | Low | The code already falls back to `unknown`; tests assert the report structure, not host values | STEP-03 |
| The coverage run makes each commit slower | High | Low | Accepted; it is passalong's rule. The developer guide says how long it takes | STEP-05 |
| The pinned toolchain formats or lints differently from `stable` on CI | Low | Low | CI pins the same version | STEP-02, STEP-06 |
| HTML and JUnit paths change and Codecov or Pages uploads break | Medium | Medium | Paths are checked locally in STEP-04 and in the GitHub run in STEP-08 | STEP-04, STEP-08 |
| passalong's `AGENTS.md` changes before STEP-07 | Medium | Low | The step re-reads it and diffs the headings first | STEP-07 |
| Contributors without `just` cannot commit | Medium | Low | The hook prints install instructions; `just setup` and the developer guide cover the rest | STEP-05, STEP-08 |

## 16. Builder hand-off

- **Start condition:** User approval and a clean repository.
- **First step:** `PLAN-00001-STEP-01`.
- **Required sequence:** STEP-01 to STEP-08 in order.
- **Parallel-safe work:** None; later steps verify with what earlier steps
  add.
- **Do not change:** approved scope, requirements, steps, acceptance
  criteria, or content outside Builder's permitted work-log area. Do not
  edit `docs/ideas/AGENTS.md`, `docs/plans/AGENTS.md`,
  `docs/reviews/AGENTS.md`, or `.agents/`.
- **Escalate when:** a stop condition is met; a non-test change under `src/`
  seems necessary; a CI failure needs a change outside the scope.
- **Completion hand-off:** report changed files, tests run, the coverage
  figure, documents updated, and backlog changes. Suggest the pull-request
  title and description for a pull request into `develop`. The user pushes.

<!-- BUILDER_WORK_LOG_START -->
## 17. Builder Work Log

> [!warning] Builder-maintained section
> Delivery Planner creates this section. After approval, Builder may update only
> this delimited section and the Builder-maintained front-matter fields. Builder
> must preserve prior entries and use UTC timestamps.

### Step status

| Step | Status | Started (UTC) | Completed (UTC) | Evidence | Builder notes |
|---|---|---|---|---|---|
| PLAN-00001-STEP-01 | completed | 2026-09-17T17:27:54Z | 2026-09-17T17:28:33Z | Verification results rows 1-3 | Baseline identical to plan section 3 |
| PLAN-00001-STEP-02 | completed | 2026-09-17T17:28:41Z | 2026-09-17T17:28:54Z | Verification results rows 4-5 | No format or lint difference under the pinned toolchain |
| PLAN-00001-STEP-03 | completed | 2026-09-17T17:29:30Z | 2026-09-17T17:31:52Z | Verification results rows 6-9 | 88.93% lines; tests only, `src/` untouched; task 3 unit tests not needed (task 4) |
| PLAN-00001-STEP-04 | completed | 2026-09-17T17:31:52Z | 2026-09-17T17:33:47Z | Verification results rows 10-12 | JUnit path bug in CI found and fixed |
| PLAN-00001-STEP-05 | completed | 2026-09-17T17:33:47Z | 2026-09-17T17:34:21Z | Verification results rows 13-14 | Hooks are now active in this clone |
| PLAN-00001-STEP-06 | completed | 2026-09-17T17:34:32Z | 2026-09-17T17:35:09Z | Verification results rows 15-16 | GitHub run checked in STEP-08 after the user pushes |
| PLAN-00001-STEP-07 | completed | 2026-09-17T17:35:15Z | 2026-09-17T17:36:04Z | Verification results rows 17-18 | The workflow puts the changelog entry after plan approval, because the plan gate needs a clean tree |
| PLAN-00001-STEP-08 | completed | 2026-09-17T17:36:11Z | 2026-09-17T17:47:31Z | Verification results rows 19-25 | All 15 acceptance criteria met; two user decisions recorded under Deviations |

Allowed status values: `not-started`, `in-progress`, `blocked`, `completed`,
`skipped`. A skipped step requires explicit user approval recorded in Evidence.

### Execution log

| Timestamp (UTC) | Step | Event | Evidence or reference | Next action |
|---|---|---|---|---|
| 2026-09-17T17:27:54Z | STEP-01 | Execution started; baseline recorded; `--help` output (27 lines) saved outside the repository for AC-14 | Verification results | Add changelog entry |
| 2026-09-17T17:28:33Z | STEP-01 | Added `## [Unreleased]` to `CHANGELOG.md`; step completed | Step commit | STEP-02 |
| 2026-09-17T17:28:54Z | STEP-02 | Created `rust-toolchain.toml`; step completed | Step commit | STEP-03 |
| 2026-09-17T17:29:30Z | STEP-03 | Refactored `tests/mock_ollama.rs` helpers: `ConfigOptions`, response status, `generated`/`chat` replies, collision-free temporary folders | tests/mock_ollama.rs | Add integration tests |
| 2026-09-17T17:31:52Z | STEP-03 | Added 21 integration tests (implementation task 2). Coverage reached 88.93%, so the unit tests of task 3 were not added, as task 4 directs. No test needs a terminal, a network, or Ollama; the only executed commands are `printf ok` and `exit 3` | Verification results | STEP-04 |
| 2026-09-17T17:31:52Z | STEP-04 | Wrote `justfile`. `cargo-nextest` was not installed; `just setup` installed 0.9.145 into `~/.cargo/bin` | justfile | Verify JUnit path |
| 2026-09-17T17:33:47Z | STEP-04 | With the old `.config/nextest.toml` path `../test-results/…` the JUnit file was written to `target/nextest/test-results/`, which CI never uploaded. Path changed to `../../test-results/unit-tests.xml`; step completed | .config/nextest.toml | STEP-05 |
| 2026-09-17T17:34:21Z | STEP-05 | Hook and `.pre-commit-config.yaml` now run `just check`; removed `scripts/verify.sh`, `scripts/install-hooks.sh`, `scripts/coverage-unit.sh`; `scripts/release.sh` runs `just check` and requires `just`; ran `just install-hooks`; step completed | Step commit | STEP-06 |
| 2026-09-17T17:34:32Z | STEP-06 | The STEP-05 commit `06c7029` ran `just check` through `.githooks/pre-commit` (its output showed the recipes) and succeeded | git commit output | Edit workflows |
| 2026-09-17T17:35:09Z | STEP-06 | All three workflows pin toolchain 1.98.1, install `just` and `cargo-llvm-cov` (and `cargo-nextest` for unit coverage), and run `just` recipes; `release-checks.yml` also runs on `feature/**` pushes; artifact and Pages paths moved to `target/llvm-cov/html`; step completed | Step commit | STEP-07 |
| 2026-09-17T17:36:04Z | STEP-07 | Rewrote `AGENTS.md` (14 sections, decisions D-01, D-02, D-04, D-05, D-07 applied, `Known deviations` added); created `.env.sample`; step completed | Step commit | STEP-08 |
| 2026-09-17T17:38:25Z | STEP-08 | `docs/roadmap.md` moved to `docs/backlog.md` and rewritten; `docs/developer-guide.md` written; `README.md`, `docs/testing.md`, `docs/publishing.md`, `docs/crates-release.md` now name `just` recipes. One more finding added to the backlog: a failed command is never saved as a session turn (seen while writing STEP-03 tests) | Step commit | User pushes the branch; Builder records the GitHub run (AC-08) |
| 2026-09-17T17:47:31Z | STEP-08 | User pushed the branch; AC-08 recorded; branch-flow documentation changed to `main` by user decision; step and plan completed | Verification results; Deviations | Release commit for v0.3.2, then the user opens the pull request into `main` |

### Deviations and blockers

| Timestamp (UTC) | Step | Deviation or blocker | Impact | Decision required from |
|---|---|---|---|---|
| 2026-09-17T17:47:31Z | STEP-08 | User decision after the push, replacing D-07: pull requests go to `main`, and the `develop` branch will be removed, as in passalong. `AGENTS.md` (New feature workflow step 2, Release workflow steps 5-6) and `docs/developer-guide.md` changed from `develop` to `main` | Documentation only; no requirement or acceptance criterion changes | None; decided by the user on 2026-09-17 |
| 2026-09-17T17:47:31Z | STEP-08 | User decision after the push, replacing D-03: this work and the unreleased 0.3.1 ship together as v0.3.2. The version bump and the changelog rename are made in a separate `release: v0.3.2` commit, outside this plan, so AC-15 holds for the plan commits | AC-15 is true up to this commit and stops being true at the release commit, by user instruction | None; decided by the user on 2026-09-17 |

### Verification results

| Timestamp (UTC) | Step | Command or check | Result | Evidence |
|---|---|---|---|---|
| 2026-09-17T17:27:54Z | STEP-01 | `cargo fmt --all --check`; `cargo clippy --all-targets --all-features -- -D warnings` | pass | Exit 0, no warnings |
| 2026-09-17T17:27:54Z | STEP-01 | `cargo llvm-cov --workspace --all-features --summary-only` | pass; matches baseline | 97 tests pass (91 unit, 6 integration); lines 3813, missed 915, 76.00% |
| 2026-09-17T17:28:33Z | STEP-01 | `grep -n '^## \[' CHANGELOG.md \| head -2` | pass | `Unreleased` on line 7, `0.3.1` follows |
| 2026-09-17T17:28:41Z | STEP-02 | `rustup show active-toolchain` before the change | recorded | `stable-x86_64-unknown-linux-gnu (default)` |
| 2026-09-17T17:28:54Z | STEP-02 | `rustc --version`; fmt check; clippy `-D warnings`; `cargo test --all-targets --all-features` | pass | `rustc 1.98.1 (48a229cea 2026-09-01)`, overridden by `rust-toolchain.toml`; 97 tests pass |
| 2026-09-17T17:29:30Z | STEP-03 | `cargo llvm-cov --workspace --all-features --fail-under-lines 80 --summary-only` before new tests | fails as expected | 76.00% (STEP-01 measurement on the same sources) |
| 2026-09-17T17:31:52Z | STEP-03 | `cargo llvm-cov --workspace --all-features --fail-under-lines 80 --summary-only` | pass | Exit 0; lines 3813, missed 422, 88.93%. `lib.rs` 87.20%, `llm.rs` 95.43%, `session.rs` 88.25%, `output.rs` 87.06%, `environment.rs` 84.39%, `main.rs` 0.00% |
| 2026-09-17T17:31:52Z | STEP-03 | `cargo test --test mock_ollama`, five consecutive runs; fmt check; clippy `-D warnings` | pass | 27 passed in every run; no warnings |
| 2026-09-17T17:31:52Z | STEP-03 | `git diff fa272c8 -- src/` | pass | Empty: no file under `src/` changed |
| 2026-09-17T17:31:52Z | STEP-04 | `scripts/verify.sh` before the change | pass | Exit 0 |
| 2026-09-17T17:33:47Z | STEP-04 | `just --list`; `just check` | pass | 20 recipes listed; `just check` exit 0 (fmt-check, lint, scripts-check, test, coverage 88.93%, build, package) |
| 2026-09-17T17:33:47Z | STEP-04 | `just coverage-html`; `just coverage-lcov`; `just test-junit` | pass | `target/llvm-cov/html/index.html`, `target/coverage/lcov.info` (121941 bytes), `target/test-results/unit-tests.xml`; nextest ran 118 tests, all passed |
| 2026-09-17T17:33:47Z | STEP-05 | `git config core.hooksPath` before the change | recorded | Unset (empty output) |
| 2026-09-17T17:34:21Z | STEP-05 | `just install-hooks`; `git config core.hooksPath`; `bash -n scripts/release.sh`; `.githooks/pre-commit` run directly | pass | `.githooks`; hook exit 0. That the step commit itself ran the hook is recorded in the next execution-log entry, because the log is part of that commit |
| 2026-09-17T17:34:32Z | STEP-06 | `just lint-workflows` on the unchanged workflows | pass | actionlint 1.7.12 (local), exit 0 |
| 2026-09-17T17:35:09Z | STEP-06 | `just lint-workflows`; `grep -n 'coverage-unit\\|verify.sh' .github/workflows/*.yml`; `grep -nE 'cargo (fmt\|clippy\|test\|package)' .github/workflows/*.yml` | pass | actionlint exit 0; both greps print nothing |
| 2026-09-17T17:35:15Z | STEP-07 | Section headings of `../passalong/AGENTS.md` against REQ-01 | pass | Same ten headings as at planning; last changed 2026-09-14, before this plan |
| 2026-09-17T17:36:04Z | STEP-07 | `grep -c '^## ' AGENTS.md`; `git check-ignore -q .env`; `grep -c 'HOMEBREW_FORMULA_FILE=' .env.sample` | pass | 14; exit 0; 1 |
| 2026-09-17T17:36:11Z | STEP-08 | `git grep -n 'verify\.sh\\|install-hooks\.sh\\|coverage-unit\.sh\\|roadmap\.md' -- ':!CHANGELOG.md' ':!docs/plans'` before the step | recorded | 9 lines to fix in `README.md` (2), `docs/testing.md` (7), `docs/publishing.md` (1); hits in `AGENTS.md`, hooks, and workflows were already fixed by STEP-05 to STEP-07 |
| 2026-09-17T17:38:25Z | STEP-08 | The same grep after the step | pass | No output (exit 1) |
| 2026-09-17T17:38:25Z | STEP-08 | `just ci` | pass | Exit 0; 118 tests pass; lines 3813, missed 422, 88.93%; package verified; actionlint clean |
| 2026-09-17T17:38:25Z | STEP-08 | `cargo run -q -- --help` against the STEP-01 output; `git diff fa272c8 -- src/ Cargo.toml Cargo.lock` | pass | Identical; empty diff; `version = "0.3.1"` |
| 2026-09-17T17:38:25Z | STEP-08 | Acceptance criteria AC-01 to AC-07 and AC-09 to AC-15 | pass | Each checked with the command named in the criterion; `docs/developer-guide.md` names all 20 recipes |
| 2026-09-17T17:38:25Z | STEP-08 | AC-08: workflow runs on the pushed head commit | pending | Needs the user to push `feature/00001-rules-alignment`; then `gh run list --branch feature/00001-rules-alignment` |
| 2026-09-17T17:47:31Z | STEP-08 | AC-08: `gh run list --branch feature/00001-rules-alignment` | pass | Release Checks run 35254186685 on head `ae1727b`: `success`. It is the only workflow triggered by a `feature/**` push; Unit Coverage runs on the pull request |

### Completion summary

- **Implementation status:** `completed`
- **Completed requirements:** PLAN-00001-REQ-01 to REQ-10
- **Incomplete requirements:** None
- **Outstanding blockers:** None. Left with the user: open the pull request into `main`, merge, tag `v0.3.2`, run `just release v0.3.2`, remove `develop`
- **Review request:** Ready. Coverage: 88.93% lines (was 76.00%)
<!-- BUILDER_WORK_LOG_END -->

## 18. Planning change log

| Timestamp (UTC) | Plan status | Change | Reason | Requested/approved by |
|---|---|---|---|---|
| 2026-09-17T16:22:20Z | draft | Plan created | User asked for a rules-alignment plan that includes the move to a `justfile` | User |
| 2026-09-17T17:27:35Z | approved | Plan approved without amendment; `build_ready` set. The draft was committed by the planner as `8afb408` at the user's request, a one-off waiver of the rule that the user commits drafts | User replied "approved" and chose "Commit draft for me" | User |

## 19. External references

None. `../passalong` is a local repository of the same owner and was read
from disk.

## 20. Confidence

**High.** Every source file, script, workflow, and document in scope was read,
and the baseline figures were measured on the baseline commit. The main
residual uncertainty is the exact number of tests needed to pass 80%, and
whether the Codecov and Pages uploads work with the new report paths, which
only the GitHub run in STEP-08 can show.
