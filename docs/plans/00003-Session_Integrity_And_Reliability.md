---
title: "Delivery Plan 00003: Session Integrity And Reliability"
aliases:
  - "Plan 00003"
tags:
  - delivery-plan
  - implementation
  - claude-code
type: delivery-plan
plan_id: "PLAN-00003"
plan_status: approved              # draft | approved | cancelled
plan_kind: initial                 # initial | superseding
created_at: "2026-09-20T22:53:45Z"
approved_at: "2026-09-20T23:17:42Z"
planner_agent: "Claude Code"
planner_model: "anthropic/claude-opus-5"
triggered_by: user                 # user | agent:<agent-name>
request_kind: review               # idea | review | idea-and-review | direct | unplanned-query
repository: "joelee/cli-bot"
baseline_branch: "feature/00003-session-integrity-and-reliability"
baseline_commit: "aff0b8e5976a53f8afc78a5d0f7136e59eeb928f"
source_ideas: []
source_reviews:
  - "docs/reviews/00001-Main_Current_Code_State.md"
previous_plan: "docs/plans/00002-Safe_Command_Confirmation.md"
requirements_count: 11
steps_count: 7
acceptance_criteria_count: 17
blocking_decisions: 0
build_ready: true
web_research_used: false
confidence: high                  # high | medium | low

# Builder-maintained front matter. Builder may update only these keys after
# explicit user approval; Delivery Planner initializes them.
implementation_status: in-progress # not-started | in-progress | blocked | completed | abandoned
builder_agent: "Claude Code"
builder_model: "anthropic/claude-opus-5"
execution_branch: "feature/00003-session-integrity-and-reliability"
execution_started_at: "2026-09-20T23:18:41Z"
execution_updated_at: "2026-09-20T23:39:23Z"
execution_completed_at: null
current_step: "PLAN-00003-STEP-02"
---

# Delivery Plan 00003: Session Integrity And Reliability

> [!abstract] Plan status: `draft`
> v0.4.1 fixes the seven runtime findings of Review 00001: one unreadable
> session file no longer stops every invocation, session files are written
> atomically and owner-only, the planner timeout becomes configurable
> instead of failing at a fixed 30 seconds, interactive mode survives a
> planner error, a failed command keeps its exit status and is remembered,
> captured output no longer breaks editors and pagers, and session file
> names stop depending on a hash the standard library may change. Nine
> decisions are resolved (section 7); none blocks, and the plan waits for
> user approval.

## 1. Objective and outcome

[Review 00001](../reviews/00001-Main_Current_Code_State.md) recorded eight
Medium findings. PLAN-00002 fixed one of them (MED-06) together with the
Major finding. This plan takes the five that affect a user at runtime, plus
the two Low findings that belong with them, leaving `MED-07` and `MED-08`
to PLAN-00004 and the backlog.

After this plan:

- A corrupt or hand-edited session file is reported and skipped, never a
  reason for `--check`, `--no-session`, or `--session-clear` to fail.
- Session files are written through a temporary file and a rename, and on
  Unix are readable only by their owner.
- `[ollama] request_timeout_seconds` and `connect_timeout_seconds` replace
  reqwest's fixed 30-second default, so loading a large model is no longer
  reported as a timeout.
- Error kinds are told apart by type. Interactive mode prints a planner
  error and asks again instead of ending, and end of input exits cleanly.
- A command that exits non-zero keeps its status, is saved in the session,
  and is reported without the doubled word.
- With `capture_command_output = true`, a command keeps the terminal's
  standard input and its output streams while being captured; editors and
  pagers are run without capture rather than broken.
- Working-directory session names use a fixed hash, and a file written
  under the old name is found once and renamed.
- The crate is at 0.4.1 with a `docs/release/v0.4.1.md` draft.

## 2. Source traceability

| Requirement | Source | Source location | Interpretation |
|---|---|---|---|
| PLAN-00003-REQ-01 | Review | REV-00001-MED-02 | Scanning tolerates unreadable files and does not run when unused |
| PLAN-00003-REQ-02 | Review | REV-00001-MED-02 "Suggested fix" (atomic write) | A session file is never half-written |
| PLAN-00003-REQ-03 | Review | REV-00001-MED-05 | Owner-only permissions on Unix |
| PLAN-00003-REQ-04 | Review | REV-00001-MED-01 | Configurable request and connect timeouts |
| PLAN-00003-REQ-05 | Review | REV-00001-MED-03 | Typed errors in place of message matching |
| PLAN-00003-REQ-06 | Review | REV-00001-MED-03 "Suggested fix" | Interactive mode survives a planner error |
| PLAN-00003-REQ-07 | Review | REV-00001-LOW-01 | Exit status kept, turn saved, message corrected |
| PLAN-00003-REQ-08 | Review | REV-00001-MED-04 | Captured output keeps stdin and streams |
| PLAN-00003-REQ-09 | Review | REV-00001-LOW-02 | A fixed hash for session file names |
| PLAN-00003-REQ-10 | Repository | `AGENTS.md` § Docs to maintain, § Release workflow step 1 | Documents, version, release draft |
| PLAN-00003-REQ-11 | Repository | `AGENTS.md` § Non-negotiables; § Product rules | The gate holds and nothing unrelated changes |

## 3. Repository baseline

| Field | Value |
|---|---|
| Repository | `joelee/cli-bot` |
| Branch | `feature/00003-session-integrity-and-reliability`, created from `main` |
| HEAD | `aff0b8e5976a53f8afc78a5d0f7136e59eeb928f` |
| Working tree at publication | Clean |
| Applicable instructions | `AGENTS.md`; `docs/plans/AGENTS.md` |

`main` is at `6daa385`, which carries v0.4.0 (published to crates.io on
2026-09-20) and the Release-workflow fix. One commit precedes this plan on
the branch: the changelog entry `aff0b8e`.

Measured during PLAN-00003 planning, on the merged `main`:

- `just ci`: exit 0. 143 tests (104 unit, 39 integration); line coverage
  **90.62%** (4479 lines, 420 missed).
- `src/prompt.rs` is 71.65%, for the reason recorded in PLAN-00003 STEP-08
  of the previous plan: its three `dialoguer` bodies need a terminal.
- `src/safety.rs` 93.85%, `src/lib.rs` 90.62%, `src/session.rs` 88.38%,
  `src/shell.rs` 88.24%.

STEP-01 re-measures and stops if the figures differ.

## 4. Scope

### In scope

- `src/session.rs`: scanning, pruning, atomic writes, permissions, hashing.
- `src/llm.rs`: the HTTP client's timeouts. This file was out of scope in
  PLAN-00002; REQ-04 brings it in.
- `src/error.rs` (new): the error kinds the request flow distinguishes.
- `src/shell.rs`: standard-input inheritance, streamed capture, exit status.
- `src/lib.rs`: interactive error handling, exit-code resolution, wiring.
- `src/main.rs`: exit with the command's status.
- `src/safety.rs`: expose the program names it already parses, so the
  capture skip list does not need a second parser.
- `src/config.rs` and `cli-bot.toml`: the two new `[ollama]` keys.
- `README.md`, `docs/configuration.md`, `docs/usage.md`,
  `docs/session-memory.md`, `docs/architecture.md`, `docs/testing.md`,
  `docs/backlog.md`.
- `Cargo.toml`, `Cargo.lock` to 0.4.1; new `docs/release/v0.4.1.md` draft.

### Out of scope

- `REV-00001-MED-07` (the missing allocation script) and every Low and Info
  finding other than LOW-01 and LOW-02: PLAN-00004.
- `REV-00001-MED-08` (Windows): a candidate feature in `docs/backlog.md`.
  This plan must not make Windows support harder, but does not add it.
- Regenerating `docs/models-benchmark-report-v0.3.2.md`. REQ-04 makes the
  regeneration worthwhile, but it needs the user's hardware and a long run;
  it stays a backlog item.
- Structured logging, the config lookup order, rustdoc coverage.
- A pseudo-terminal test harness for `src/prompt.rs`.
- The release commit, the tag, and publishing.

## 5. Constraints and preserved decisions

- The safety model of v0.4.0 is not weakened: no change may let a command
  run with less approval than it needs today.
- Session files written by v0.3.x and v0.4.0 keep loading, including files
  named with the old hash.
- A `cli-bot.toml` written for any earlier version keeps working; the new
  keys have defaults.
- No new runtime dependency. The hash is written in this crate.
- Tests never reach a real Ollama service or the network, never open a
  terminal prompt, and execute only harmless commands.
- The agent does not push, tag, or publish.

## 6. Assumptions

None. Unresolved matters are recorded as decisions and block approval when
material.

## 7. Decisions and blockers

All resolved by the planner from the review and the code. The user may
overturn any of them at approval; none blocks.

| ID | Decision or blocker | Resolution | Owner | Status |
|---|---|---|---|---|
| D-01 | Timeout defaults | `request_timeout_seconds = 300`, `connect_timeout_seconds = 10`, both in `[ollama]`; `0` means no limit. 300 s covers the 120B model in the v0.3.2 report; a separate short connect timeout keeps an unreachable server failing fast | Planner | Resolved |
| D-02 | What to do with an unreadable session file | Skip it and report it in verbose output. Not moved aside, not deleted: a file the user hand-edited is theirs, and renaming it would hide the mistake | Planner | Resolved |
| D-03 | How pruning decides age | By file modification time, not by parsing the last turn. Retention means "not updated in N days", which is exactly what mtime records, and it removes the parse that MED-02 is about | Planner | Resolved |
| D-04 | When pruning runs | Only when the invocation uses session memory. `--no-session` and `--check` no longer touch the sessions folder at all | Planner | Resolved |
| D-05 | Permissions | Unix only: the folder `0700`, files `0600`, applied when each is next written. Other platforms keep the system default, and the code must still compile there | Planner | Resolved |
| D-06 | Error kinds | A `CliBotError` enum in a new `src/error.rs`, carried through `anyhow` and recognised with `downcast_ref`. Three kinds: `Cancelled`, `CommandFailed { status }`, `Planner`. Everything else stays an ordinary error and remains fatal | Planner | Resolved |
| D-07 | End of input | At an interactive prompt it is `Cancelled` and exits 0. On the single-request path it stays an error, so `printf '' \| cli-bot` still fails rather than silently succeeding | Planner | Resolved |
| D-08 | Exit status | `cli-bot` exits with the command's own status; its own failures stay 1. `run` keeps its signature, and the status travels in `CommandFailed`, so `src/main.rs` only asks a tested helper for the code | Planner | Resolved |
| D-09 | Captured output | Standard input is always inherited. When capture is on, output is copied to the terminal as it arrives and collected at the same time. Programs that need the terminal itself, the configured editor plus a built-in list, run with no capture rather than broken. The program names come from `src/safety.rs`, which already parses commands | Planner | Resolved |

## 8. Affected architecture and components

| Component | Today | After |
|---|---|---|
| `SessionStore::prune_expired` | Parses every file; first bad file aborts the run; runs on every invocation | Uses modification time; skips unreadable entries; runs only when sessions are used |
| `SessionStore::list` | First unparsable file aborts | Skips and reports it, and lists the rest |
| `SessionStore::save` | `fs::write` truncates in place | Writes `<name>.json.tmp` with mode `0600`, then renames |
| Sessions folder | Created with the process umask | Created `0700` on Unix |
| `stable_path_hash` | `DefaultHasher` | FNV-1a, with a one-time lookup and rename of the old name |
| `OllamaClient::new` | `Client::new()`, fixed 30 s | Client built from the configured timeouts |
| Error handling | `error.to_string().contains(...)` in two places | `CliBotError` and `downcast_ref` |
| Interactive loop | Any non-command error ends the session | Planner errors print and the prompt returns |
| `shell::execute` | `output()` when capturing: no stdin, no streaming | Always inherits stdin; streams and captures together; skips capture for terminal programs |
| Exit code | Always 1 on failure | The command's own status when a command failed |

### New module `src/error.rs`

```rust
pub enum CliBotError {
    /// The user ended the prompt, with Ctrl-C or end of input.
    Cancelled,
    /// The command ran and exited non-zero.
    CommandFailed { status: Option<i32> },
    /// The planner could not produce a usable plan: transport, timeout,
    /// decoding, or an empty plan.
    Planner,
}
```

Implements `std::error::Error` and `Display`, so an `anyhow::Error` carries
it and `downcast_ref::<CliBotError>()` recognises it. `Display` keeps the
present wording where a user sees it, with `CommandFailed` reading
`command exited with status 3`.

### Streamed capture

`shell::execute` spawns the child with `Stdio::inherit()` for standard
input in every mode. When capture is wanted and the command is not on the
skip list, standard output and standard error are piped, and one thread per
stream copies each chunk to the real stream while appending to a buffer
bounded by `max_output_bytes`. Both threads are joined before the status is
read, so nothing is lost. When capture is not wanted, or the command is on
the skip list, all three streams are inherited, which is today's behaviour.

The skip list is the first word of the configured `preferred_editor` plus
the built-ins `vi`, `vim`, `nvim`, `emacs`, `nano`, `helix`, `hx`, `less`,
`more`, `most`, `top`, `htop`, `btop`, `man`, `ssh`, `watch`, `tmux`,
`screen`, and `fzf`, matched against the program names
`safety::program_names` returns for the command.

## 9. Requirement catalogue

### PLAN-00003-REQ-01 — Session scanning tolerates bad files and stays out of the way

- **Requirement:** `prune_expired` decides age from file modification time
  (D-03), skips any entry it cannot read, and returns the number removed.
  `list` skips a file it cannot read or parse and reports it through the
  verbose channel, listing the rest. Neither runs when the invocation does
  not use session memory (D-04).
- **Rationale:** REV-00001-MED-02: one bad file stops `--check`,
  `--no-session`, and `--session-clear`, the commands a user would reach
  for to recover.
- **Source:** REV-00001-MED-02.
- **Acceptance evidence:** AC-01, AC-02.

### PLAN-00003-REQ-02 — Session files are written atomically

- **Requirement:** `save` writes to a temporary file beside the target and
  renames it over the target. A failed or interrupted write leaves the
  previous file intact, and never a truncated one.
- **Rationale:** the same finding: `fs::write` truncates first, which is how
  a half-written file appears.
- **Source:** REV-00001-MED-02.
- **Acceptance evidence:** AC-03.

### PLAN-00003-REQ-03 — Session data is owner-only on Unix

- **Requirement:** on Unix the sessions folder is created with mode `0700`
  and each session file written with `0600`, including files that already
  exist with wider permissions. On other platforms the code compiles and
  behaves as before.
- **Rationale:** REV-00001-MED-05: requests, commands, working directories,
  and captured output are world-readable at `644` today.
- **Source:** REV-00001-MED-05.
- **Acceptance evidence:** AC-04.

### PLAN-00003-REQ-04 — Planner timeouts are configurable

- **Requirement:** `[ollama] request_timeout_seconds` (default `300`) and
  `connect_timeout_seconds` (default `10`), both `u64`, `0` meaning no
  limit, are applied when the client is built. `--verbose` prints both.
  Every client the process builds, including the one per model in
  `--models-benchmark`, uses them.
- **Rationale:** REV-00001-MED-01: 11 of 35 calls in the project's own
  benchmark report failed on reqwest's fixed 30 seconds.
- **Source:** REV-00001-MED-01.
- **Acceptance evidence:** AC-05, AC-06.

### PLAN-00003-REQ-05 — Error kinds are types, not text

- **Requirement:** `src/error.rs` defines `CliBotError` as in section 8.
  The planner path returns `Planner`, `shell::execute` returns
  `CommandFailed`, and a prompt the user ended returns `Cancelled`. No code
  outside `Display` decides what an error is by matching its message; the
  two existing text matches are removed with their tests.
- **Rationale:** REV-00001-MED-03: rewording a message silently changes
  behaviour today, and a unit test pins the text in place.
- **Source:** REV-00001-MED-03.
- **Acceptance evidence:** AC-07, AC-11.

### PLAN-00003-REQ-06 — Interactive mode survives a planner error

- **Requirement:** in interactive mode a `Planner` or `CommandFailed` error
  is printed and the prompt returns, with the next prompt in its error
  colour. `Cancelled` ends the session with status 0. Any other error is
  fatal, as today. On the single-request path `Cancelled` is an error
  (D-07).
- **Rationale:** the same finding: the errors that end the session today are
  the common ones with small local models.
- **Source:** REV-00001-MED-03.
- **Acceptance evidence:** AC-08, AC-09.

### PLAN-00003-REQ-07 — A failed command keeps its status and is remembered

- **Requirement:** a non-zero exit is reported as
  `command exited with status <code>`; the session turn is saved with
  `executed: true` and the status; and `cli-bot` exits with that status.
  `cli-bot`'s own failures still exit 1. The mapping lives in a tested
  function in `src/lib.rs`; `src/main.rs` only calls it (D-08).
- **Rationale:** REV-00001-LOW-01: the status is lost, the message reads
  `command exited with status exit status: 3`, and a follow-up such as
  "why did that fail" has no context.
- **Source:** REV-00001-LOW-01.
- **Acceptance evidence:** AC-10, AC-11.

### PLAN-00003-REQ-08 — Captured output keeps the terminal usable

- **Requirement:** `shell::execute` always inherits standard input. With
  capture on, output is streamed to the terminal as it arrives and captured
  at the same time, bounded by `max_output_bytes`. A command whose program
  is on the skip list of section 8 runs with all three streams inherited and
  is recorded with no captured output. `safety::program_names` supplies the
  program names.
- **Rationale:** REV-00001-MED-04: `Command::output()` gives the child no
  terminal, so `nvim`, `sudo`, `less`, and `ssh` cannot work and nothing
  appears until the command ends.
- **Source:** REV-00001-MED-04.
- **Acceptance evidence:** AC-12, AC-13.

### PLAN-00003-REQ-09 — Session file names are stable across toolchains

- **Requirement:** `stable_path_hash` uses FNV-1a written in this crate. On
  load, when no file exists under the new name but one exists under the
  `DefaultHasher` name this toolchain would produce, it is renamed to the
  new name once and used.
- **Rationale:** REV-00001-LOW-02: `DefaultHasher`'s algorithm may change
  between Rust releases and then orphans working-directory sessions.
- **Source:** REV-00001-LOW-02.
- **Acceptance evidence:** AC-14.

### PLAN-00003-REQ-10 — Documentation, version, release notes

- **Requirement:** `docs/configuration.md` documents the two new keys and
  the owner-only permissions; `docs/session-memory.md` describes atomic
  writes, tolerant scanning, mtime pruning, the file-name change, and the
  capture limits; `docs/usage.md` shows a failing command's exit status;
  `docs/architecture.md` lists `src/error.rs`; `docs/testing.md` records
  the new coverage; `README.md` mentions the configurable timeout;
  `docs/backlog.md` loses the seven findings this plan closes.
  `Cargo.toml` and `Cargo.lock` move to 0.4.1, and
  `docs/release/v0.4.1.md` is created as a draft, absolute links only.
- **Rationale:** `AGENTS.md` § Docs to maintain and § Release workflow.
- **Source:** `AGENTS.md`.
- **Acceptance evidence:** AC-15, AC-16.

### PLAN-00003-REQ-11 — Gate held, nothing unrelated changed

- **Requirement:** `just ci` exits 0 with line coverage at or above 90.62%,
  the figure at the baseline. `src/session.rs`, `src/shell.rs`, and
  `src/error.rs` are each at or above 90%. The classification and approval
  behaviour of v0.4.0 is unchanged, and the `--help` output differs from
  the baseline only where this plan adds something.
- **Rationale:** the coverage gate, and the safety work must not regress.
- **Source:** `AGENTS.md` § Non-negotiables; section 5.
- **Acceptance evidence:** AC-17.

## 10. Delivery strategy

The order follows the review's recommendation, and each step is independent
enough to revert alone.

The session work comes first, because MED-02, MED-05, and LOW-02 all change
how a session file is written or found, and doing them together avoids
writing the same function three times. Timeouts follow, as a contained
change to one constructor. The error work is next, because REQ-07 rides on
the enum REQ-05 introduces. Captured output is last of the code steps: it
is the largest change and the one with the least test reach, so it benefits
from everything else being settled.

Each step is test-first, ends in one commit
`build: complete PLAN-00003-STEP-NN - <title>` once its verification
passes, and keeps the work log current. Two lessons from PLAN-00002 are
built into the steps: a lint that denies dead code means a module and the
code that uses it land together, and `just ci` is run after the step commit
because its publish dry run needs a clean tree.

## 11. Detailed implementation steps

### PLAN-00003-STEP-01 — Baseline

- **Objective:** confirm the starting state and lock the references.
- **Requirements:** `PLAN-00003-REQ-11`
- **Depends on:** None
- **Affected components:** this plan's Builder fields only
- **Preconditions:** plan approved; worktree clean; on the plan's branch.
- **Test or evidence first:** run `just ci`; save `cargo run -- --help` and
  a v0.4.0-format session file outside the repository as the AC-17 and
  AC-14 references.
- **Implementation tasks:**
  1. Set the Builder front-matter fields.
  2. Record the baseline in the work log.
- **Documentation/configuration/operations:** none.
- **Verification:** `just ci` exits 0; coverage within 0.5 points of 90.62%.
- **Completion criteria:** the baseline is recorded and committed.
- **Rollback or recovery:** none needed.
- **Builder stop conditions:** the baseline differs from section 3.

### PLAN-00003-STEP-02 — Session scanning, atomic writes, permissions, hashing

- **Objective:** a session store that cannot be stopped by one bad file, and
  does not leak its contents.
- **Requirements:** `PLAN-00003-REQ-01`, `PLAN-00003-REQ-02`,
  `PLAN-00003-REQ-03`, `PLAN-00003-REQ-09`
- **Depends on:** `PLAN-00003-STEP-01`
- **Affected components:** `src/session.rs`, `src/lib.rs` (pruning is moved
  behind the session-enabled check), `tests/mock_ollama.rs`
- **Preconditions:** STEP-01 committed.
- **Test or evidence first:** write these and see them fail: a truncated
  file beside a valid one, after which `--check`, `--no-session`,
  `--session-clear`, and `--session-list` all still succeed and the valid
  session is still listed; a save that leaves no `.tmp` file behind and
  leaves the previous content intact when serialisation fails; on Unix, a
  folder at `0700` and a file at `0600`, including a file that existed at
  `0644`; pruning that removes a file with an old modification time and
  keeps a fresh one; a session written under the `DefaultHasher` name being
  found once and renamed.
- **Implementation tasks:**
  1. Replace `stable_path_hash` with FNV-1a and add the one-time fallback
     and rename.
  2. Rewrite `save` as write-temp-then-rename, with `0600` on Unix and
     `0700` for the folder.
  3. Rewrite `prune_expired` to use modification times and skip unreadable
     entries; make `list` skip and report bad files.
  4. In `src/lib.rs`, prune only when the invocation uses sessions.
- **Documentation/configuration/operations:** documented in STEP-07.
- **Verification:** `just check` exits 0; the new tests pass; a v0.4.0
  session file still loads; `src/session.rs` at or above 90% lines.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a rename cannot be made atomic on the target
  platform without a new dependency.

### PLAN-00003-STEP-03 — Configurable timeouts

- **Objective:** a slow model load is no longer a timeout.
- **Requirements:** `PLAN-00003-REQ-04`
- **Depends on:** `PLAN-00003-STEP-02`
- **Affected components:** `src/llm.rs`, `src/config.rs`, `cli-bot.toml`
- **Preconditions:** STEP-02 committed.
- **Test or evidence first:** a unit test that the client is built from the
  configured values and that `0` means no limit; an integration test
  against a mock server that sleeps past a very short configured timeout,
  asserting the error names the timeout rather than a decoding failure; a
  test that a v0.4.0 config parses and yields the defaults.
- **Implementation tasks:**
  1. Add both keys to `OllamaConfig` with serde defaults and to
     `cli-bot.toml` with comments.
  2. Build the client with `reqwest::blocking::Client::builder`, mapping
     `0` to no limit.
  3. Print both in verbose output.
- **Documentation/configuration/operations:** documented in STEP-07.
- **Verification:** `just check` exits 0; the timeout test is deterministic
  in three consecutive runs.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** the timeout test proves flaky on this
  machine; record it and use a unit test of the builder instead.

### PLAN-00003-STEP-04 — Typed errors and interactive survival

- **Objective:** stop deciding what an error is by reading its text.
- **Requirements:** `PLAN-00003-REQ-05`, `PLAN-00003-REQ-06`
- **Depends on:** `PLAN-00003-STEP-03`
- **Affected components:** `src/error.rs` (new), `src/lib.rs`,
  `src/llm.rs`, `src/prompt.rs`, `tests/mock_ollama.rs`
- **Preconditions:** STEP-03 committed.
- **Test or evidence first:** integration tests that, in interactive mode
  with a scripted prompter, a planner failure on the first of two requests
  prints an error and the second request is still planned and run; that
  end of input ends the session with `Ok`; that a fatal error, such as an
  unreadable config, still ends it. A unit test that each kind survives a
  round trip through `anyhow` and is recognised by `downcast_ref`.
- **Implementation tasks:**
  1. Add `src/error.rs` with `CliBotError`, its `Display`, and its
     `std::error::Error` implementation.
  2. Return `Planner` from the planner path, `Cancelled` from a prompt the
     user ended, and `CommandFailed` from `shell::execute`.
  3. Replace `interactive_prompt_cancelled` and
     `handle_interactive_request_error` with `downcast_ref`, and delete
     their text-matching tests.
- **Documentation/configuration/operations:** documented in STEP-07.
- **Verification:** `just check` exits 0;
  `git grep -n 'to_string().contains' src/` finds nothing;
  `src/error.rs` at or above 90% lines.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a `Cancelled` from `dialoguer` cannot be told
  apart from a real I/O failure without matching text; record it and keep
  the narrowest possible match, documented in the module.

### PLAN-00003-STEP-05 — Exit status and the failed turn

- **Objective:** a failed command reports and is remembered properly.
- **Requirements:** `PLAN-00003-REQ-07`
- **Depends on:** `PLAN-00003-STEP-04`
- **Affected components:** `src/shell.rs`, `src/lib.rs`, `src/main.rs`,
  `tests/mock_ollama.rs`
- **Preconditions:** STEP-04 committed.
- **Test or evidence first:** an integration test that an approved `exit 3`
  saves a turn with `executed: true` and `exit_status: 3`, and that the
  error message is exactly `command exited with status 3`; a unit test of
  the exit-code helper for a command failure, another error, and success.
- **Implementation tasks:**
  1. Have `shell::execute` return `CommandFailed` carrying the status.
  2. Save the turn before returning the error.
  3. Add the exit-code helper in `src/lib.rs` and call it from
     `src/main.rs`.
- **Documentation/configuration/operations:** documented in STEP-07.
- **Verification:** `just check` exits 0; the message has no doubled word;
  running the built binary on a failing command exits with its status.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** none expected.

### PLAN-00003-STEP-06 — Captured output keeps the terminal usable

- **Objective:** capture stops breaking editors, pagers, and `sudo`.
- **Requirements:** `PLAN-00003-REQ-08`
- **Depends on:** `PLAN-00003-STEP-05`
- **Affected components:** `src/shell.rs`, `src/safety.rs`,
  `src/config.rs`, `tests/mock_ollama.rs`
- **Preconditions:** STEP-05 committed.
- **Test or evidence first:** tests that a captured command's output is
  both printed and stored; that output larger than `max_output_bytes` is
  truncated in the session but printed in full; that a command whose
  program is on the skip list is executed with no captured output; that
  standard input reaches the child, by running a command that reads it.
  A unit test of `safety::program_names` for a pipeline and a wrapper.
- **Implementation tasks:**
  1. Expose `safety::program_names`, reusing the existing parser.
  2. Rewrite the capturing branch of `shell::execute` to spawn with an
     inherited standard input and piped output, with one copying thread per
     stream, joined before the status is read.
  3. Add the skip list and consult the configured editor.
- **Documentation/configuration/operations:** documented in STEP-07.
- **Verification:** `just check` exits 0; `src/shell.rs` at or above 90%
  lines; the tests pass three times in a row.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** the streaming threads cannot be joined
  deterministically, or a test proves flaky; record it and fall back to
  inheriting all three streams whenever capture is requested for a
  non-read-only command, documenting the limit.

### PLAN-00003-STEP-07 — Documentation, version, release draft, final gate

- **Objective:** documents match behaviour; hand over with evidence.
- **Requirements:** `PLAN-00003-REQ-10`, `PLAN-00003-REQ-11`
- **Depends on:** `PLAN-00003-STEP-06`
- **Affected components:** `README.md`, `docs/configuration.md`,
  `docs/usage.md`, `docs/session-memory.md`, `docs/architecture.md`,
  `docs/testing.md`, `docs/backlog.md`, `Cargo.toml`, `Cargo.lock`,
  `docs/release/v0.4.1.md` (new)
- **Preconditions:** STEP-06 committed.
- **Test or evidence first:**
  `git grep -n '30-second\|30 seconds\|fs::write' -- '*.md' src/session.rs`
  and a read of `docs/session-memory.md`, recorded; every hit is a line to
  check.
- **Implementation tasks:**
  1. Update the seven documents per REQ-10.
  2. Bump to 0.4.1 and write the release draft.
  3. Remove MED-01, MED-02, MED-03, MED-04, MED-05, LOW-01, and LOW-02 from
     `docs/backlog.md`.
  4. Commit, then run `just ci` on the clean tree, and compare `--help`
     with the STEP-01 reference.
  5. Fill in the completion summary and hand over.
- **Documentation/configuration/operations:** this step is documentation
  and the version bump.
- **Verification:** `just ci` exits 0; `just links` passes;
  `scripts/check-release-tag.sh v0.4.1` reports only the draft line and the
  changelog heading, both of which the release commit writes.
- **Completion criteria:** every acceptance criterion is checked.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a documented behaviour cannot be reconciled
  with the implementation.

## 12. Cross-cutting concerns

| Area | Applicability | Planned action or reason not applicable | Step or requirement |
|---|---|---|---|
| Compatibility and APIs | Applicable | Old configs and old session files keep working; the old file name is found once and renamed; `run` keeps its signature | REQ-04, REQ-09, D-08 |
| Data and migration | Applicable | The rename in REQ-09 is the only migration, and it is one file at a time, on load | STEP-02 |
| Security and privacy | Applicable | Owner-only session files close a local disclosure; the v0.4.0 approval model is explicitly unchanged | REQ-03, section 5 |
| Performance and scale | Applicable | Pruning stops parsing every file; streaming adds two threads per captured command, which is negligible beside an LLM call | REQ-01, REQ-08 |
| Reliability and failure handling | Applicable | This is the theme: tolerant scanning, atomic writes, configurable timeouts, typed errors, a prompt that survives | REQ-01 to REQ-06 |
| Observability and operations | Applicable | Skipped session files and both timeouts are reported in verbose output; structured logging stays out of scope | REQ-01, REQ-04 |
| Dependencies and supply chain | Applicable | No new runtime dependency; the hash is written here; `cargo deny` runs in `just ci` | Section 5 |
| Accessibility and UX | Applicable | Output appears as it is produced rather than at the end; a failed command's status reaches the shell | REQ-07, REQ-08 |
| Documentation and release | Applicable | Seven documents, the changelog entry already committed as `aff0b8e`, and the v0.4.1 draft | REQ-10 |
| Deployment and rollback | Applicable | One commit per step, each revertable alone | Section 10 |

## 13. Verification strategy

| Level | Evidence or command | When | Required result |
|---|---|---|---|
| Format, lint, scripts, links | `just check` | Every step | Exit 0 |
| Unit tests | `cargo test --lib` | Every step from STEP-02 | All pass |
| Integration tests | `cargo test --test mock_ollama` | Every step from STEP-02 | All pass |
| Coverage | `just coverage` | STEP-02 onwards | ≥ 90.62% total; ≥ 90% in `src/session.rs`, `src/shell.rs`, `src/error.rs` |
| Backward compatibility | A v0.4.0 session file and config | STEP-02, STEP-03 | Both load unchanged |
| Flakiness | The timeout and streaming tests, three runs | STEP-03, STEP-06 | Identical each time |
| Behaviour | `diff` of `--help` against the STEP-01 reference | STEP-07 | Only this plan's additions |
| Release records | `scripts/check-release-tag.sh v0.4.1` | STEP-07 | Only the draft line and the changelog heading |
| Full pipeline | `just ci` | STEP-01, STEP-07 | Exit 0 |
| CI | `gh run list` for the pushed head commit | After hand-off | All `success` |

## 14. Acceptance criteria

- [ ] `PLAN-00003-AC-01` With an unparsable file in the sessions folder,
  `--check`, `--no-session`, `--session-clear`, and `--session-list` all
  succeed, the valid sessions are still listed, and `--verbose` names the
  skipped file.
- [ ] `PLAN-00003-AC-02` With `--no-session`, the sessions folder is not
  read at all, shown by a test whose storage directory does not exist.
- [ ] `PLAN-00003-AC-03` After a save, no `.tmp` file remains; a save that
  fails part way leaves the previous file byte-identical.
- [ ] `PLAN-00003-AC-04` On Unix the sessions folder is mode `0700` and
  each session file `0600`, including a file that existed at `0644`.
- [ ] `PLAN-00003-AC-05` `[ollama] request_timeout_seconds` and
  `connect_timeout_seconds` exist with defaults `300` and `10`; a v0.4.0
  config parses and gets them; `0` means no limit.
- [ ] `PLAN-00003-AC-06` Against a mock server that sleeps past a short
  configured timeout, the error names the timeout, and `--verbose` prints
  both values.
- [ ] `PLAN-00003-AC-07` `git grep -n "to_string().contains" src/` finds
  nothing, and each `CliBotError` kind is recognised through `anyhow` by
  `downcast_ref`.
- [ ] `PLAN-00003-AC-08` In interactive mode, a planner failure on the
  first of two requests prints an error and the second request is still
  planned and run.
- [ ] `PLAN-00003-AC-09` End of input at an interactive prompt returns
  `Ok`; on the single-request path it is still an error.
- [ ] `PLAN-00003-AC-10` An approved `exit 3` saves a turn with
  `executed: true` and `exit_status: 3`, and the built binary exits 3.
- [ ] `PLAN-00003-AC-11` The failure message is exactly
  `command exited with status 3`.
- [ ] `PLAN-00003-AC-12` A captured command's output is both printed and
  stored, truncated in the session at `max_output_bytes` but printed in
  full; a command that reads standard input receives it.
- [ ] `PLAN-00003-AC-13` A command whose program is on the skip list, or
  the configured editor, runs with no captured output, and
  `safety::program_names` returns the programs of a pipeline with wrappers
  stripped.
- [ ] `PLAN-00003-AC-14` A session file written under the `DefaultHasher`
  name is found once, renamed to the FNV-1a name, and its turns preserved.
- [ ] `PLAN-00003-AC-15` `docs/configuration.md`, `docs/session-memory.md`,
  `docs/usage.md`, `docs/architecture.md`, `docs/testing.md`, and
  `README.md` describe the new behaviour, and `docs/backlog.md` no longer
  lists MED-01 to MED-05, LOW-01, or LOW-02.
- [ ] `PLAN-00003-AC-16` `Cargo.toml` and `Cargo.lock` say `0.4.1`;
  `docs/release/v0.4.1.md` exists, begins with a `Draft` line, and has no
  relative links.
- [ ] `PLAN-00003-AC-17` `just ci` exits 0 with total line coverage at or
  above 90.62%; `src/session.rs`, `src/shell.rs`, and `src/error.rs` are
  each at or above 90%; the v0.4.0 classification and approval tests pass
  unchanged; the `--help` diff shows only this plan's additions.

## 15. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation or test | Owner/step |
|---|---|---|---|---|
| Streaming and capturing at once deadlocks or loses output | Medium | High | One thread per stream, both joined before the status is read; three consecutive runs; a documented fallback to inheriting all streams | STEP-06 |
| The skip list misses a program that needs the terminal | Medium | Medium | Standard input is inherited in every mode, so `sudo` works regardless; the list is documented and the limit stated | REQ-08, STEP-07 |
| Exiting with the command's status changes what a script sees | Medium | Medium | It is the point of LOW-01; the release notes lead with it, and cli-bot's own failures still exit 1 | REQ-07, D-08 |
| The one-time rename picks up a file belonging to another directory | Low | High | The old name is recomputed with `DefaultHasher` from the same path, so it can only match that directory's own file; the rename happens only when no new-name file exists | REQ-09, STEP-02 |
| A longer default timeout makes a genuinely unreachable server feel hung | Medium | Low | A separate 10-second connect timeout keeps that case fast | D-01 |
| mtime-based pruning behaves oddly on a copied or restored folder | Low | Low | Retention is a convenience, not a guarantee; documented in `docs/session-memory.md` | D-03, STEP-07 |
| The timeout test is timing-dependent | Medium | Low | A very short configured timeout against a server that sleeps far longer; a unit test of the builder as the fallback | STEP-03 |

## 16. Builder hand-off

- **Start condition:** User approval and a clean repository.
- **First step:** `PLAN-00003-STEP-01`.
- **Required sequence:** STEP-01 to STEP-07 in order.
- **Parallel-safe work:** None.
- **Do not change:** approved scope, requirements, steps, acceptance
  criteria, or content outside Builder's permitted work-log area. Do not
  weaken the v0.4.0 approval model, edit the review, or touch anything
  listed as out of scope in section 4.
- **Escalate when:** a stop condition is met; a change outside section 4
  seems necessary; a fix would require a new runtime dependency.
- **Completion hand-off:** report changed files, tests run, the coverage
  figure, documents updated, and backlog changes. Suggest the pull-request
  title and description for a pull request into `main`. The user pushes.

<!-- BUILDER_WORK_LOG_START -->
## 17. Builder Work Log

> [!warning] Builder-maintained section
> Delivery Planner creates this section. After approval, Builder may update only
> this delimited section and the Builder-maintained front-matter fields. Builder
> must preserve prior entries and use UTC timestamps.

### Step status

| Step | Status | Started (UTC) | Completed (UTC) | Evidence | Builder notes |
|---|---|---|---|---|---|
| PLAN-00003-STEP-01 | completed | 2026-09-20T23:18:41Z | 2026-09-20T23:18:41Z | Verification results rows 1-2 | One stale figure in section 3; see Deviations |
| PLAN-00003-STEP-02 | completed | 2026-09-20T23:23:10Z | 2026-09-20T23:23:10Z | Verification results rows 3-5 | Permissions tighten on the next write, as D-05 specifies; a file only read keeps its mode |
| PLAN-00003-STEP-03 | not-started | — | — | — | — |
| PLAN-00003-STEP-04 | not-started | — | — | — | — |
| PLAN-00003-STEP-05 | not-started | — | — | — | — |
| PLAN-00003-STEP-06 | not-started | — | — | — | — |
| PLAN-00003-STEP-07 | not-started | — | — | — | — |

Allowed status values: `not-started`, `in-progress`, `blocked`, `completed`,
`skipped`. A skipped step requires explicit user approval recorded in Evidence.

### Execution log

| Timestamp (UTC) | Step | Event | Evidence or reference | Next action |
|---|---|---|---|---|
| 2026-09-20T23:18:41Z | STEP-01 | Execution started on the approved plan; baseline confirmed | Verification results rows 1-2 | STEP-02: the session write path |
| 2026-09-20T23:23:10Z | STEP-02 | FNV-1a replaces `DefaultHasher`, with a one-time rename from the old name; `save` writes a `.json.tmp` file with mode 0600 and renames it, and narrows the folder to 0700; `prune_expired` uses modification times and skips unreadable entries; `list` returns the paths it skipped; `src/lib.rs` prunes only when the invocation uses session memory and reports skipped files | src/session.rs, src/lib.rs, tests/mock_ollama.rs | STEP-03: configurable timeouts |
| 2026-09-20T23:39:23Z | STEP-02 | User reported four core files in the repository root. They came from `MockOllamaServer`: its `Drop` opened a connection to wake the accept loop, the server thread read an empty request and panicked, and `Drop` then panicked joining it, which aborts a process that is already unwinding a failed test. The server now stops on an atomic flag, `read_http_request` returns `None` for an empty connection, and `Drop` never panics. The four core files (290 MB) were deleted | tests/mock_ollama.rs | STEP-03: configurable timeouts |

### Deviations and blockers

| Timestamp (UTC) | Step | Deviation or blocker | Impact | Decision required from |
|---|---|---|---|---|
| 2026-09-20T23:18:41Z | STEP-01 | Plan section 3 gives `src/shell.rs` as 88.24%, which was its figure before PLAN-00002; the measured value is 93.14%. Every other figure, including the 90.62% total, matches exactly | None; the stated total and the per-file floors in REQ-11 are unaffected | None; planner error in a non-binding figure |
| 2026-09-20T23:23:10Z | STEP-02 | Two pre-existing tests asserted the parse-based pruning that D-03 replaces: `session::tests::prunes_expired_sessions` and `session_commands_list_show_and_prune` both wrote a file whose newest turn was old. Both now set the file modification time instead, which is what the approved decision prunes on | None on scope; the tests assert the approved behaviour rather than the replaced one | None; required by D-03 |
| 2026-09-20T23:39:23Z | STEP-02 | A test-harness fix outside the seven findings: every failing integration test was aborting the process and writing a 72 MB core file, which would have kept happening through the rest of this plan. Made on the STEP-02 branch state rather than as a new step | None on scope; no production code changed, and the harness is one the plan extends at every step | None; reported to the user |

### Verification results

| Timestamp (UTC) | Step | Command or check | Result | Evidence |
|---|---|---|---|---|
| 2026-09-20T23:18:41Z | STEP-01 | `just ci` | pass | Exit 0; 143 tests; lines 4479, missed 420, 90.62%, matching plan section 3 exactly |
| 2026-09-20T23:18:41Z | STEP-01 | Reference material saved outside the repository | recorded | `--help` (29 lines) for AC-17; a v0.4.0-format session file, including the `risk` field, for AC-14 |
| 2026-09-20T23:23:10Z | STEP-02 | Test first: the six new tests before the implementation | fail as expected | Compilation failed on `legacy_path_hash` and on the new `list` signature |
| 2026-09-20T23:23:10Z | STEP-02 | `just check` | pass | Exit 0; 151 tests (110 unit, 41 integration); `src/session.rs` 90.95% lines (floor 90%); total 91.15%, up from 90.62% |
| 2026-09-20T23:23:10Z | STEP-02 | A v0.4.0 session file loaded by the built binary | pass | `--session-show` printed the stored turn unchanged, including its `risk` field |
| 2026-09-20T23:39:23Z | STEP-02 | A deliberately failing test, before and after the harness fix | pass | Before: SIGABRT and a 72 MB `core.<pid>` in the repository root. After: the test reports its failure and no core file is written |

### Completion summary

- **Implementation status:** `in-progress`
- **Completed requirements:** PLAN-00003-REQ-01, REQ-02, REQ-03, REQ-09
- **Incomplete requirements:** REQ-04 to REQ-08, REQ-10, REQ-11
- **Outstanding blockers:** None
- **Review request:** Not ready
<!-- BUILDER_WORK_LOG_END -->

## 18. Planning change log

| Timestamp (UTC) | Plan status | Change | Reason | Requested/approved by |
|---|---|---|---|---|
| 2026-09-20T22:53:45Z | draft | Plan created | User chose two themed plans for the remaining Review 00001 findings, and this is the runtime half | User |
| 2026-09-20T23:17:42Z | approved | Plan approved without amendment; `build_ready` set | User replied "approved" | User |

## 19. External references

None. Review 00001 and the repository were read from disk.

## 20. Confidence

**High.** All seven findings were reproduced or read in full during Review
00001, and every file in scope was read while writing this plan. The
principal uncertainty is REQ-08: streaming and capturing a child's output
at once is the only part with real concurrency, and its step carries both a
flakiness check and a documented fallback.
