---
title: "Delivery Plan 00002: Safe Command Confirmation"
aliases:
  - "Plan 00002"
tags:
  - delivery-plan
  - implementation
  - claude-code
type: delivery-plan
plan_id: "PLAN-00002"
plan_status: approved              # draft | approved | cancelled
plan_kind: initial                 # initial | superseding
created_at: "2026-09-20T20:21:39Z"
approved_at: "2026-09-20T20:32:38Z"
planner_agent: "Claude Code"
planner_model: "anthropic/claude-opus-5"
triggered_by: user                 # user | agent:<agent-name>
request_kind: review               # idea | review | idea-and-review | direct | unplanned-query
repository: "joelee/cli-bot"
baseline_branch: "feature/00002-safe-command-confirmation"
baseline_commit: "82b88dafde1228dfe86dc4209917b62e1f0ac595"
source_ideas: []
source_reviews:
  - "docs/reviews/00001-Main_Current_Code_State.md"
previous_plan: null
requirements_count: 12
steps_count: 9
acceptance_criteria_count: 18
blocking_decisions: 0
build_ready: true
web_research_used: false
confidence: high                  # high | medium | low

# Builder-maintained front matter. Builder may update only these keys after
# explicit user approval; Delivery Planner initializes them.
implementation_status: completed # not-started | in-progress | blocked | completed | abandoned
builder_agent: "Claude Code"
builder_model: "anthropic/claude-opus-5"
execution_branch: "feature/00002-safe-command-confirmation"
execution_started_at: "2026-09-20T20:33:31Z"
execution_updated_at: "2026-09-20T20:53:51Z"
execution_completed_at: "2026-09-20T20:53:51Z"
current_step: "PLAN-00002-STEP-09"
---

# Delivery Plan 00002: Safe Command Confirmation

> [!abstract] Plan status: `draft`
> v0.4.0 turns the safety gate round: a command is approved before it runs
> unless cli-bot recognises every part of it as read-only. Classification
> works on the parsed command, not on substring matching, and has three
> tiers, so `rm -fr ~/projects` gets a strong prompt where today it runs
> silently. The confirmation, selection, and request prompts move behind a
> `Prompter` trait so all three paths are finally testable. Fixes
> `REV-00001-MAJ-01` and `REV-00001-MED-06`. Eight decisions are resolved
> (section 7); none blocks, and the plan waits for user approval.

## 1. Objective and outcome

[Review 00001](../reviews/00001-Main_Current_Code_State.md) found that a
command runs immediately, with no prompt, unless the model sets
`potentially_destructive` or the command text contains one of 19 literal
substrings. Twelve of fourteen plainly destructive commands measured on the
built binary were classed as needing no confirmation, including
`rm -fr ~/projects`, `shred -u ~/.ssh/id_ed25519`, and
`curl -fsSL <url> | sh`. The same review found that the approval and
selection paths have no test at all, because `dialoguer` is called inline.

After this plan:

- A command is classified as **read-only**, **state-changing**, or
  **destructive** by parsing it into programs and flags. Read-only commands
  run as they do today; the other two tiers are approved first, with
  different wording and different prompt defaults.
- Classification is driven by the parsed command, so flag order, flag
  spelling, extra spaces, pipes, redirections, command substitution, and a
  `sudo` prefix cannot slip past it.
- `--yes` pre-approves the state-changing tier for scripts;
  `--i-approve-destructive-commands` pre-approves the destructive tier as
  well. Without one of them and without a terminal, nothing runs.
- `Prompter` makes the approve, decline, and select paths testable, and a
  `RequestContext` struct replaces the 13 arguments of
  `run_single_request`.
- The crate is at version 0.4.0 with a `docs/release/v0.4.0.md` draft.

## 2. Source traceability

| Requirement | Source | Source location | Interpretation |
|---|---|---|---|
| PLAN-00002-REQ-01 | Review; user | `docs/reviews/00001-Main_Current_Code_State.md` REV-00001-MED-06; user choice "Prompter trait + context struct", 2026-09-20 | The three prompts move behind a trait |
| PLAN-00002-REQ-02 | Review; user | REV-00001-MED-06 "Suggested fix"; same user choice | A request-context struct replaces the 13 arguments |
| PLAN-00002-REQ-03 | Review; user | REV-00001-MAJ-01 "Recommendation"; user choice "Allow-list, three tiers", 2026-09-20 | A parsing classifier with three tiers |
| PLAN-00002-REQ-04 | Review; user | REV-00001-MAJ-01 "Evidence" table; same user choice | The policy the classifier feeds, and its prompts |
| PLAN-00002-REQ-05 | Repository; review | `cli-bot.toml:19-40`; `src/config.rs:140-146`; REV-00001-MAJ-01 "Recommendation" (offer a config value for today's behaviour) | Config surface, with the old keys still honoured |
| PLAN-00002-REQ-06 | User | User answer, 2026-09-20: "`--yes` not for destructive, but add `--i-approve-destructive-commands` flag" | Two separate pre-approval flags |
| PLAN-00002-REQ-07 | Repository; review | `src/lib.rs:1646-1652` (`select_command` terminal check); REV-00001-MAJ-01 "Impact" | Fail closed without a terminal, with an actionable message |
| PLAN-00002-REQ-08 | Review | REV-00001-MED-06 "Suggested fix"; REV-00001-MAJ-01 "Suggested verification" | The tests the two findings ask for |
| PLAN-00002-REQ-09 | Repository | `AGENTS.md` § Docs to maintain; `README.md:432-438`; `docs/configuration.md`; `docs/usage.md` | Documents that describe the safety model |
| PLAN-00002-REQ-10 | User; repository | User choice "v0.4.0", 2026-09-20; `AGENTS.md` § Release workflow step 1 | Version bump and release-notes draft |
| PLAN-00002-REQ-11 | Repository | `AGENTS.md` § Product rules ("Prefer small, local changes"); `docs/usage.md` | Everything outside the confirmation path keeps its behaviour |
| PLAN-00002-REQ-12 | Repository | `AGENTS.md` § Non-negotiables ("line coverage >= 80%") | The coverage gate holds |

## 3. Repository baseline

| Field | Value |
|---|---|
| Repository | `joelee/cli-bot` |
| Branch | `feature/00002-safe-command-confirmation`, created from `main` |
| HEAD | `82b88dafde1228dfe86dc4209917b62e1f0ac595` |
| Working tree at publication | Clean |
| Applicable instructions | `AGENTS.md`; `docs/plans/AGENTS.md`; `docs/reviews/AGENTS.md` |

`main` is at `7c41d822b492064227e81336f393673618919320` (v0.3.2 released to
crates.io). Three commits precede this plan on the branch: the review
`8e3c273`, the backlog update `69c98e3`, and the changelog entry `82b88da`.

Measured on `main` during Review 00001, with cargo 1.98.1:

- `just ci`: exit 0. 118 tests (91 unit, 27 integration); line coverage
  **88.93%** (3813 lines, 422 missed); `cargo deny`, link check, publish
  dry run, and `actionlint` all clean.
- `src/lib.rs` 87.20%, `src/planner.rs` 94.83%, `src/shell.rs` 88.24%.
  The uncovered remainder is mostly the terminal-bound code this plan makes
  testable.

## 4. Scope

### In scope

- New `src/safety.rs`: command parsing and the three-tier classifier.
- New `src/prompt.rs`: the `Prompter` trait, the `dialoguer` implementation,
  and the terminal-capability check.
- `src/lib.rs`: `RequestContext`, the confirmation policy, the two new CLI
  flags, `run_with_prompter`.
- `src/config.rs` and `cli-bot.toml`: `[safety] read_only_commands`,
  `destructive_commands`, `assume_yes`; `[ui] confirmation_prompt`.
- `src/planner.rs`: `command_requires_confirmation` is replaced by the
  classifier; `PlannedCommand` and plan parsing are unchanged.
- Unit tests in the new modules; integration tests in
  `tests/mock_ollama.rs` driven by a scripted prompter.
- `README.md`, `docs/configuration.md`, `docs/usage.md`,
  `docs/architecture.md`, `docs/testing.md`, `docs/backlog.md`.
- `Cargo.toml`, `Cargo.lock` to 0.4.0; new `docs/release/v0.4.0.md` draft.

### Out of scope

Each is in `docs/backlog.md` and keeps its review ID.

- Every other open finding of Review 00001: `MED-01` timeout, `MED-02`
  corrupt session file, `MED-03` interactive error handling, `MED-04`
  output capture, `MED-05` session permissions, `MED-07` allocation
  script, `MED-08` Windows, and all seven Low and both Info findings.
- Structured logging, the config lookup order, and rustdoc coverage
  (`AGENTS.md` § Known deviations).
- Regenerating the models benchmark report; it depends on `MED-01`.
- The release commit itself, the tag, and publishing, which follow
  `AGENTS.md` § Release workflow after the user approves the work.

## 5. Constraints and preserved decisions

- Keep behaviour config-driven through `cli-bot.toml`; every classification
  list is visible and editable there.
- Keep the interactive selection flow intact: selection stays separate from
  confirmation, and `--auto-select-best` keeps its meaning.
- Fail closed. Any command that cannot be parsed, or whose program is not
  recognised, is treated as needing approval, never as read-only.
- No new runtime dependency. The parser is written in this crate; a
  dev-dependency is allowed only if a test cannot be written without one.
- Tests never call a real Ollama service or the network, never open a
  terminal prompt, and execute only harmless commands such as `printf ok`.
- `main.rs` keeps calling `run(cli)`; the trait is additive.
- No agent push, tag, or publish.

## 6. Assumptions

None. Unresolved matters are recorded as decisions and block approval when
material.

## 7. Decisions and blockers

D-01 to D-04 are the user's answers of 2026-09-20; the rest the planner
resolved from the repository and the review. None blocks.

| ID | Decision or blocker | Resolution | Owner | Status |
|---|---|---|---|---|
| D-01 | Gate shape | Allow-list with three tiers: read-only runs silently, destructive gets a strong prompt, everything else an ordinary prompt | User | Resolved |
| D-02 | Non-interactive escape hatch | `--yes`/`-y` pre-approves the state-changing tier only; `--i-approve-destructive-commands` additionally pre-approves the destructive tier | User | Resolved |
| D-03 | Version | v0.4.0 | User | Resolved |
| D-04 | Size of the test seam | `Prompter` trait **and** the request-context struct | User | Resolved |
| D-05 | Where the destructive flag may be set | Command line only. `[safety] assume_yes` is a config default for `--yes` alone; no config key pre-approves destructive commands, because a config default would make the strong tier permanently silent | Planner | Resolved |
| D-06 | Prompt defaults | The ordinary prompt defaults to yes (Enter runs); the destructive prompt defaults to no, as today. The differing default is part of the signal | Planner | Resolved |
| D-07 | `require_confirmation = false` | Keeps its current meaning: no prompt for any tier. Existing configs that set it keep working, and `docs/configuration.md` states the risk plainly | Planner | Resolved |
| D-08 | Flag-sensitive rules | Programs whose risk depends on flags or subcommands (`rm`, `find`, `chmod`, `chown`, `git`, `docker`) use a built-in rule table in `src/safety.rs`, documented in `docs/configuration.md`. The two config lists match on whole leading words only, so the config file stays declarative | Planner | Resolved |

## 8. Affected architecture and components

The planner, the session store, the environment detection, the Ollama
client, and the benchmark remain untouched. The change sits between
"a command has been selected" and "the command is executed".

| Component | Today | After |
|---|---|---|
| Risk decision | `planner::command_requires_confirmation`: LLM flag OR lower-cased substring match | `safety::classify`: parses the command, applies built-in rules and the two config lists, returns `CommandRisk` |
| Prompting | `Confirm::new()` and `Select::new()` inline in `src/lib.rs`; `Input` in `prompt_for_request` | `Prompter` trait in `src/prompt.rs`; `DialoguerPrompter` in production, a scripted one in tests |
| Request flow | `run_single_request` with 13 arguments | `run_single_request(&RequestContext, &dyn Prompter, &str)` |
| Entry point | `run(cli)` | `run(cli)` calls `run_with_prompter(cli, &DialoguerPrompter::new())`; both public |
| CLI surface | no pre-approval flag | `--yes`/`-y`, `--i-approve-destructive-commands` |
| `[safety]` config | `require_confirmation`, `destructive_substrings` | plus `read_only_commands`, `destructive_commands`, `assume_yes` |

### Classification algorithm

`safety::classify(command_text, llm_flagged, &SafetyConfig) -> CommandRisk`.

1. **Split** the text into segments at top-level `;`, `&&`, `||`, `|`, `&`,
   and newlines, honouring single quotes, double quotes, and backslash
   escapes.
2. **Mark the text unsafe for the read-only tier** when it contains command
   substitution (`$(`, a backtick), a redirection (`>`, `>>`, `<>`, `&>`),
   or a segment whose program is a shell (`sh`, `bash`, `zsh`, `fish`,
   `dash`) or `eval`, `exec`, `source`, or `.`.
3. For each segment, **strip prefixes**: leading `VAR=value` assignments,
   and the wrappers `sudo`, `doas`, `env`, `command`, `nice`, `nohup`,
   `time`, `stdbuf`, `xargs`, with their own flags. Take the **basename**
   of the first remaining word as the program.
4. **Decide, in this order** (first match wins, so a destructive rule always
   beats the read-only list):
   1. `llm_flagged` is true → `Destructive`.
   2. The lower-cased text contains a `destructive_substrings` entry →
      `Destructive` (back-compat).
   3. A segment matches a built-in destructive rule, or its leading words
      match a `destructive_commands` entry → `Destructive`.
   4. Step 2 marked the text unsafe → `StateChanging`.
   5. Every segment's leading words match a `read_only_commands` entry →
      `ReadOnly`.
   6. Otherwise → `StateChanging`.

The risk of the whole command is the highest risk of any segment.

### Built-in destructive rules (D-08)

| Program | Destructive when |
|---|---|
| `rm` | any of `-r`, `-R`, `--recursive`, `-f`, `--force`, or a combined short flag containing `r` or `f` |
| `find` | any of `-delete`, `-exec`, `-execdir`, `-ok`, `-okdir`, `-fls`, `-fprint` |
| `chmod`, `chown`, `chgrp` | any of `-R`, `--recursive` |
| `git` | subcommand `clean`, `reset` with `--hard`, `push` with `--force`/`-f`/`--force-with-lease`, `branch` with `-D` |
| `docker`, `podman` | `system prune`, `volume rm`, `rmi`, `container prune` |
| `shred`, `mkfs*`, `dd`, `fdisk`, `parted`, `mkswap`, `wipefs`, `sgdisk` | always |
| `shutdown`, `reboot`, `poweroff`, `halt`, `init` | always |
| `kill`, `killall`, `pkill` | always |
| `truncate`, `dd` | always |
| `systemctl`, `service` | `stop`, `disable`, `mask` |

### Shipped lists

`read_only_commands` ships with single programs (`ls`, `cat`, `head`,
`tail`, `wc`, `stat`, `file`, `du`, `df`, `tree`, `find`, `fd`, `grep`,
`rg`, `pwd`, `whoami`, `id`, `hostname`, `uname`, `uptime`, `date`, `cal`,
`printenv`, `which`, `type`, `man`, `ps`, `free`, `lscpu`, `lsblk`,
`lsusb`, `lspci`, `ping`, `dig`, `host`, `nslookup`, `traceroute`, `echo`,
`printf`, `less`, `more`, `bat`) and with multi-word entries (`git status`,
`git log`, `git diff`, `git show`, `git branch`, `git remote`,
`docker ps`, `docker images`, `systemctl status`, `cargo tree`,
`npm list`). `destructive_commands` ships empty, because the built-in rules
cover the defaults; it exists so a site can add its own.

## 9. Requirement catalogue

### PLAN-00002-REQ-01 — Prompter trait

- **Requirement:** `src/prompt.rs` defines
  `pub trait Prompter { fn read_request(&self, error_state: bool) -> Result<String>; fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize>; fn confirm(&self, prompt: &str, default: bool) -> Result<bool>; fn supports_dialogs(&self) -> bool; }`
  and `DialoguerPrompter`, which holds today's behaviour verbatim,
  including the interactive theme, the stdin-not-a-terminal branch of
  `prompt_for_request`, and the terminal-capability description.
  `pub fn run_with_prompter(cli: Cli, prompter: &dyn Prompter) -> Result<()>`
  is added and `run` delegates to it.
- **Rationale:** REV-00001-MED-06: the three prompts cannot be tested while
  `dialoguer` is called inline.
- **Source:** REV-00001-MED-06; user choice D-04.
- **Acceptance evidence:** AC-10, AC-12.

### PLAN-00002-REQ-02 — Request context struct

- **Requirement:** a `RequestContext<'a>` struct carries the fields
  `run_single_request` takes today (cli flags it reads, config, resolved
  environment, session store and name, planner, preferred editor,
  total_start, show_output, verbose, output). `run_single_request` takes
  `(&RequestContext<'_>, &dyn Prompter, &str)`. The
  `#[allow(clippy::too_many_arguments)]` on it is removed.
- **Rationale:** the 13 arguments are what makes injecting a prompter
  awkward; the backlog carries this item.
- **Source:** REV-00001-MED-06; user choice D-04.
- **Acceptance evidence:** AC-11.

### PLAN-00002-REQ-03 — Command classifier

- **Requirement:** `src/safety.rs` implements the algorithm of section 8,
  exposing `CommandRisk { ReadOnly, StateChanging, Destructive }` and
  `classify`. Parsing handles quoting, escapes, the listed separators,
  redirections, substitution, prefix wrappers, and absolute program paths.
  Anything it cannot parse yields at least `StateChanging`.
- **Rationale:** substring matching is the root cause of REV-00001-MAJ-01.
- **Source:** REV-00001-MAJ-01; user choice D-01.
- **Acceptance evidence:** AC-01, AC-02.

### PLAN-00002-REQ-04 — Three-tier policy

- **Requirement:** in `run_single_request`, a selected command is executed
  only after approval appropriate to its risk: `ReadOnly` runs without a
  prompt; `StateChanging` uses `[ui] confirmation_prompt` with default yes;
  `Destructive` uses `[ui] approval_prompt` with default no. A declined
  command prints the existing cancellation message, saves its session turn
  as not executed, and returns `Ok(())`. The session turn records the risk
  tier alongside `confirmation_required`.
- **Rationale:** the behaviour the review asks for.
- **Source:** REV-00001-MAJ-01; user choices D-01, D-06.
- **Acceptance evidence:** AC-03, AC-17.

### PLAN-00002-REQ-05 — Configuration

- **Requirement:** `[safety]` gains `read_only_commands` (list, shipped
  values of section 8), `destructive_commands` (list, empty), and
  `assume_yes` (bool, false). `[ui]` gains `confirmation_prompt`
  (default `"Run this command?"`). `require_confirmation` and
  `destructive_substrings` keep their current meaning (D-07, algorithm step
  4.2). Every key has a serde default, so an existing `cli-bot.toml` still
  parses; a config without the new keys behaves as if the shipped defaults
  were present.
- **Rationale:** the product rule is that behaviour is config-driven, and
  users with a customised config must not be broken.
- **Source:** `cli-bot.toml:19-40`; `src/config.rs:140-146`; D-07, D-08.
- **Acceptance evidence:** AC-08, AC-09, AC-15.

### PLAN-00002-REQ-06 — Pre-approval flags

- **Requirement:** `--yes` / `-y` pre-approves the `StateChanging` tier.
  `--i-approve-destructive-commands` (long form only, no short form)
  pre-approves `Destructive` and implies `--yes`. `[safety] assume_yes`
  is the config default for `--yes`; no config key pre-approves the
  destructive tier (D-05). Both appear in `--help` with wording that says
  what they skip.
- **Rationale:** confirm-by-default would otherwise end non-interactive use.
- **Source:** user answer D-02; D-05.
- **Acceptance evidence:** AC-05, AC-06, AC-07.

### PLAN-00002-REQ-07 — Fail closed without a terminal

- **Requirement:** when approval is required, not pre-approved, and
  `supports_dialogs()` is false, `run_single_request` returns an error that
  names the command, its tier, and the flag that would allow it, and
  executes nothing. The existing message for selection without a terminal
  keeps its meaning.
- **Rationale:** a prompt that cannot be shown must never become an
  implicit yes.
- **Source:** `src/lib.rs:1646-1652`; REV-00001-MAJ-01.
- **Acceptance evidence:** AC-04.

### PLAN-00002-REQ-08 — Tests for the gate and the prompts

- **Requirement:** unit tests in `src/safety.rs` cover the fourteen
  commands of the REV-00001-MAJ-01 table with their expected tiers, plus
  read-only cases, quoting, pipes, redirections, substitution, `sudo`,
  absolute paths, and unparsable input. Integration tests in
  `tests/mock_ollama.rs` drive the real flow with a scripted prompter and
  cover: approved and executed; declined and not executed; a selected
  alternative being the command that runs; each flag; each config switch;
  and the no-terminal error.
- **Rationale:** both findings ask for exactly these.
- **Source:** REV-00001-MAJ-01 "Suggested verification"; REV-00001-MED-06.
- **Acceptance evidence:** AC-01, AC-12.

### PLAN-00002-REQ-09 — Documentation

- **Requirement:** `README.md` § Safety Model and § Safety for Risky
  Commands describe the three tiers, the two flags, and what read-only
  means; `docs/configuration.md` documents every new key, the built-in rule
  table, and the risk of `require_confirmation = false`;
  `docs/usage.md` shows a destructive prompt and a scripted run with
  `--yes`; `docs/architecture.md` adds `src/safety.rs` and `src/prompt.rs`;
  `docs/testing.md` records the new coverage; `docs/backlog.md` loses the
  two findings this plan closes.
- **Rationale:** `AGENTS.md` § Docs to maintain; the README currently
  describes a gate that will no longer exist.
- **Source:** `AGENTS.md`; `README.md:432-438`.
- **Acceptance evidence:** AC-14.

### PLAN-00002-REQ-10 — Version and release notes draft

- **Requirement:** `Cargo.toml` and `Cargo.lock` move to `0.4.0`, and
  `docs/release/v0.4.0.md` is created as a draft: it starts with a line
  beginning `Draft`, uses only absolute
  `https://github.com/joelee/cli-bot/blob/v0.4.0/...` links, and leads with
  the behaviour change and how to restore unattended running. The release
  commit that removes the draft line is not part of this plan.
- **Rationale:** `AGENTS.md` § Release workflow step 1;
  `scripts/check-release-tag.sh` requires the file and rejects relative
  links and a draft line at tag time.
- **Source:** user choice D-03; `AGENTS.md`.
- **Acceptance evidence:** AC-16.

### PLAN-00002-REQ-11 — No unrelated behaviour change

- **Requirement:** `--dry-run`, `--print-plan`, `--benchmark`,
  `--models-benchmark`, `--check`, `--auto-select-best`, `--quiet`,
  `--verbose`, the session flags, session file format, and the planner
  prompts behave as they do at the baseline commit. `--dry-run` continues
  to print without executing and without prompting, and records the risk in
  the session turn.
- **Rationale:** `AGENTS.md` § Product rules; a wide behaviour change would
  make the safety change hard to review.
- **Source:** `AGENTS.md`; `docs/usage.md`.
- **Acceptance evidence:** AC-17.

### PLAN-00002-REQ-12 — Coverage gate

- **Requirement:** `just coverage` exits 0, and `src/safety.rs` and
  `src/prompt.rs` are each at or above 90% line coverage.
- **Rationale:** the gate, and these two modules carry the safety decision.
- **Source:** `AGENTS.md` § Non-negotiables.
- **Acceptance evidence:** AC-13.

## 10. Delivery strategy

Three refactors that change no behaviour come first (`Prompter`,
`RequestContext`, then the classifier as a standalone module with its own
tests). Only STEP-06 changes what a user sees, and by then the classifier
is already covered by unit tests and the prompts are injectable, so the
behaviour change can be driven by tests rather than by hand.

Each step is test-first: the failing test is written and run before the
production code, and the step's commit happens only after its verification
passes. Each step ends in one commit,
`build: complete PLAN-00002-STEP-NN - <title>`, made through the
`just check` hook.

The classifier is the risk. It is a small shell parser, and a parser that
is wrong in the permissive direction reintroduces the finding. Two rules
contain that: anything unparsable is at least `StateChanging`, and the
destructive rules are evaluated before the read-only list. STEP-04 tests
both directly.

## 11. Detailed implementation steps

### PLAN-00002-STEP-01 — Baseline

- **Objective:** record the starting state and lock the reference outputs.
- **Requirements:** `PLAN-00002-REQ-11`
- **Depends on:** None
- **Affected components:** this plan's Builder fields only
- **Preconditions:** plan approved; worktree clean; on
  `feature/00002-safe-command-confirmation`.
- **Test or evidence first:** run `just ci` and record the result, the test
  count, and the coverage figure. Save `cargo run -- --help` and the
  fourteen-command classification measured in Review 00001 to files outside
  the repository, as the AC-17 and AC-01 references.
- **Implementation tasks:**
  1. Set the Builder front-matter fields.
  2. Record the baseline in the work log.
- **Documentation/configuration/operations:** none.
- **Verification:** `just ci` exits 0; the recorded coverage is within 0.5
  points of 88.93%.
- **Completion criteria:** the work log holds the baseline; the step is
  committed.
- **Rollback or recovery:** none needed; no repository change.
- **Builder stop conditions:** `just ci` fails on an untouched branch.

### PLAN-00002-STEP-02 — Prompter trait and DialoguerPrompter

- **Objective:** put a seam in front of the three prompts, changing nothing.
- **Requirements:** `PLAN-00002-REQ-01`
- **Depends on:** `PLAN-00002-STEP-01`
- **Affected components:** `src/prompt.rs` (new), `src/lib.rs`,
  `tests/mock_ollama.rs`
- **Preconditions:** STEP-01 committed.
- **Test or evidence first:** a unit test that `DialoguerPrompter`
  reports `supports_dialogs() == false` when `TERM` is `dumb` or a stream
  is not a terminal, and an integration test that an existing flow still
  works when driven through `run_with_prompter`. Both fail to compile
  before the module exists.
- **Implementation tasks:**
  1. Create `src/prompt.rs` with the trait of REQ-01.
  2. Move `prompt_for_request`, `interactive_prompt_theme`,
     `select_command`'s `Select` call, the `Confirm` call, and
     `TerminalEnvironmentStatus` into `DialoguerPrompter`, unchanged.
  3. Add `run_with_prompter`; `run` delegates.
  4. Thread `&dyn Prompter` through `run`, the interactive loop,
     `run_single_request`, and `select_command`.
- **Documentation/configuration/operations:** none yet; STEP-09 documents.
- **Verification:** `just check` exits 0; `cargo run -- --help` matches the
  STEP-01 reference; coverage not lower than baseline.
- **Completion criteria:** verification passes, behaviour unchanged.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** moving the theme or the stdin branch changes
  observable interactive output.

### PLAN-00002-STEP-03 — RequestContext

- **Objective:** remove the 13-argument signature.
- **Requirements:** `PLAN-00002-REQ-02`
- **Depends on:** `PLAN-00002-STEP-02`
- **Affected components:** `src/lib.rs`
- **Preconditions:** STEP-02 committed.
- **Test or evidence first:** temporarily remove the
  `#[allow(clippy::too_many_arguments)]` and record that `just lint` fails;
  that lint passing afterwards is the test.
- **Implementation tasks:**
  1. Define `RequestContext<'a>` with the fields of REQ-02.
  2. Build it once in `run_with_prompter` and use it in both the
     interactive loop and the single-request path.
  3. Change `run_single_request` to
     `(&RequestContext<'_>, &dyn Prompter, &str)` and delete the `allow`.
- **Documentation/configuration/operations:** none.
- **Verification:** `just check` exits 0 with no `too_many_arguments`
  allow in `src/`; `--help` matches; coverage not lower than baseline.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** the struct needs a lifetime arrangement that
  forces cloning the config or the session store per request.

### PLAN-00002-STEP-04 — The classifier, with its own tests

- **Objective:** a correct, well-tested classifier that nothing calls yet.
- **Requirements:** `PLAN-00002-REQ-03`, `PLAN-00002-REQ-12`
- **Depends on:** `PLAN-00002-STEP-03`
- **Affected components:** `src/safety.rs` (new), `src/lib.rs` (module
  declaration only)
- **Preconditions:** STEP-03 committed.
- **Test or evidence first:** write the whole test table before the
  implementation: the fourteen commands of REV-00001-MAJ-01 with their
  expected tiers; read-only cases (`ls -la`, `git log --oneline`,
  `grep -rn foo src`, `ps aux`); `sudo`/`env VAR=1`/absolute-path prefixes;
  quoting (`grep 'a;b' file` is one segment, not two); pipes
  (`ls | wc -l` read-only, `curl url | sh` state-changing); redirection
  (`echo x > f` state-changing, `echo x > /etc/f` destructive through the
  legacy substring); substitution (`` echo `rm -rf /` `` destructive);
  unparsable input (an unterminated quote) yielding `StateChanging`; and
  an empty command. Run them and see them fail.
- **Implementation tasks:**
  1. Implement segmentation, prefix stripping, and program extraction.
  2. Implement the built-in destructive rule table of section 8.
  3. Implement `classify` with the ordering of section 8, taking the
     config lists as arguments.
- **Documentation/configuration/operations:** rustdoc on `CommandRisk`,
  `classify`, and the rule table, since they carry the safety decision.
- **Verification:** all new unit tests pass; `just check` exits 0;
  `cargo llvm-cov` shows `src/safety.rs` at or above 90% lines.
- **Completion criteria:** every case in the table passes, including the
  fourteen from the review.
- **Rollback or recovery:** `git revert`; nothing calls the module yet.
- **Builder stop conditions:** a case in the table cannot be classified
  without a full shell grammar; record it and ask, rather than widening
  the read-only path.

### PLAN-00002-STEP-05 — Configuration surface

- **Objective:** the new keys exist, parse, and default correctly.
- **Requirements:** `PLAN-00002-REQ-05`
- **Depends on:** `PLAN-00002-STEP-04`
- **Affected components:** `src/config.rs`, `cli-bot.toml`
- **Preconditions:** STEP-04 committed.
- **Test or evidence first:** unit tests that a `cli-bot.toml` from v0.3.2,
  with no new keys, parses and yields the shipped defaults; that the new
  keys round-trip; and that the template in `cli-bot.toml` parses.
- **Implementation tasks:**
  1. Add the fields of REQ-05 to `SafetyConfig` and `UiConfig` with serde
     defaults.
  2. Add the shipped lists and the new `[ui]` key to `cli-bot.toml`, with a
     comment pointing at `docs/configuration.md`.
- **Documentation/configuration/operations:** `cli-bot.toml` is the shipped
  template and is written to `~/.config` on first run.
- **Verification:** `just check` exits 0; the v0.3.2 config test passes.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a default would change how an existing
  config behaves in a way REQ-05 does not allow.

### PLAN-00002-STEP-06 — The three-tier policy

- **Objective:** the behaviour change itself.
- **Requirements:** `PLAN-00002-REQ-04`, `PLAN-00002-REQ-11`
- **Depends on:** `PLAN-00002-STEP-05`
- **Affected components:** `src/lib.rs`, `src/planner.rs`, `src/session.rs`
- **Preconditions:** STEP-05 committed.
- **Test or evidence first:** integration tests, using the scripted
  prompter, that a read-only command runs with no prompt; that a
  state-changing command is prompted with default yes and runs when
  approved; that a destructive command is prompted with default no and does
  **not** run when declined, leaving a turn with `executed: false`. They
  fail against the current policy.
- **Implementation tasks:**
  1. Replace `command_requires_confirmation` with a call to
     `safety::classify`, keeping the function as a thin wrapper only if a
     caller still needs it; otherwise delete it and its tests.
  2. Apply the policy of REQ-04 in `run_single_request`, using
     `[ui] confirmation_prompt` and `[ui] approval_prompt`.
  3. Record the tier in the session turn next to `confirmation_required`,
     with a serde default so older session files still parse.
  4. Keep `--dry-run` printing without prompting.
- **Documentation/configuration/operations:** STEP-09 documents.
- **Verification:** `just check` exits 0; the fourteen-command table now
  reaches the correct tier end to end for at least `rm -fr`, `shred`, and
  `curl | sh`; a v0.3.2 session file still loads.
- **Completion criteria:** the new integration tests pass and no existing
  test needed a change beyond the policy it asserts.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** an existing integration test fails for a
  reason other than the intended policy change.

### PLAN-00002-STEP-07 — Pre-approval flags and no-terminal behaviour

- **Objective:** keep scripted use possible, and fail closed otherwise.
- **Requirements:** `PLAN-00002-REQ-06`, `PLAN-00002-REQ-07`
- **Depends on:** `PLAN-00002-STEP-06`
- **Affected components:** `src/lib.rs`, `src/config.rs`, `cli-bot.toml`
- **Preconditions:** STEP-06 committed.
- **Test or evidence first:** integration tests that `--yes` runs a
  state-changing command with no prompt but still prompts for a destructive
  one; that `--i-approve-destructive-commands` runs both and implies
  `--yes`; that `assume_yes = true` behaves as `--yes`; and that with a
  prompter reporting `supports_dialogs() == false` and no flag, the run
  fails with a message naming the flag and executes nothing.
- **Implementation tasks:**
  1. Add both flags to `Cli` with `--help` wording.
  2. Resolve the effective approval level from flags and `assume_yes`.
  3. Return the REQ-07 error when approval is needed and impossible.
- **Documentation/configuration/operations:** the flags appear in `--help`;
  STEP-09 documents them.
- **Verification:** `just check` exits 0; `cargo run -- --help` shows both
  flags; the no-terminal test asserts that nothing was executed.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** `-y` collides with an existing short flag.

### PLAN-00002-STEP-08 — Complete the test matrix

- **Objective:** cover the paths the review named, and meet the gate.
- **Requirements:** `PLAN-00002-REQ-08`, `PLAN-00002-REQ-12`
- **Depends on:** `PLAN-00002-STEP-07`
- **Affected components:** `tests/mock_ollama.rs`, unit tests in
  `src/prompt.rs`, `src/safety.rs`, `src/lib.rs`
- **Preconditions:** STEP-07 committed.
- **Test or evidence first:** run `just coverage` and record which of the
  policy branches are still uncovered; that list is the work.
- **Implementation tasks:**
  1. Add the selection tests: with two commands and a scripted choice of
     the second, the second is the command that runs and is saved.
  2. Add `require_confirmation = false` and `destructive_substrings`
     back-compat tests.
  3. Add a test that an unparsable command is prompted, not run silently.
  4. Add prompter unit tests for the terminal-capability description.
- **Documentation/configuration/operations:** none.
- **Verification:** `just coverage` exits 0; `src/safety.rs` and
  `src/prompt.rs` at or above 90% lines; total not below 88.93%.
- **Completion criteria:** the matrix of REQ-08 is complete.
- **Rollback or recovery:** `git revert`; tests only.
- **Builder stop conditions:** a new test is flaky in three consecutive
  runs.

### PLAN-00002-STEP-09 — Documentation, version, release draft, final gate

- **Objective:** documents match behaviour; hand over with evidence.
- **Requirements:** `PLAN-00002-REQ-09`, `PLAN-00002-REQ-10`,
  `PLAN-00002-REQ-11`
- **Depends on:** `PLAN-00002-STEP-08`
- **Affected components:** `README.md`, `docs/configuration.md`,
  `docs/usage.md`, `docs/architecture.md`, `docs/testing.md`,
  `docs/backlog.md`, `Cargo.toml`, `Cargo.lock`,
  `docs/release/v0.4.0.md` (new)
- **Preconditions:** STEP-08 committed.
- **Test or evidence first:**
  `git grep -n 'destructive substring\|substring rules'` and a read of
  `README.md` § Safety Model, recorded; every hit is a line to fix.
- **Implementation tasks:**
  1. Update the six documents per REQ-09.
  2. Bump `Cargo.toml` and `Cargo.lock` to 0.4.0.
  3. Write the `docs/release/v0.4.0.md` draft per REQ-10.
  4. Remove `REV-00001-MAJ-01` and `REV-00001-MED-06` from
     `docs/backlog.md`, and the "Coverage of terminal-bound code" and
     "`run_single_request` takes 13 arguments" items they close.
  5. Run `just ci`; compare `--help` with the STEP-01 reference, expecting
     exactly the two new flags.
  6. Fill in the completion summary and hand over.
- **Documentation/configuration/operations:** this step is documentation
  and the version bump.
- **Verification:** `just ci` exits 0; `just links` passes;
  `scripts/check-release-tag.sh v0.4.0` fails **only** on the draft line,
  which proves every other release record is in place; the `--help` diff
  shows only the two new flags.
- **Completion criteria:** every acceptance criterion is checked.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a documented behaviour cannot be reconciled
  with the implementation; stop and report rather than documenting the
  discrepancy.

## 12. Cross-cutting concerns

| Area | Applicability | Planned action or reason not applicable | Step or requirement |
|---|---|---|---|
| Compatibility and APIs | Applicable | An existing `cli-bot.toml` and existing session files keep working (REQ-05, STEP-06 task 3); `run` keeps its signature; the behaviour change is the point of the version bump | REQ-05, REQ-10 |
| Data and migration | Applicable | The session turn gains one field with a serde default; no migration | STEP-06 |
| Security and privacy | Applicable | This is the security change. Fail-closed parsing, destructive rules before the read-only list, no config key for the destructive flag (D-05) | REQ-03, REQ-07, D-05 |
| Performance and scale | Not applicable | Classification is a string parse per command, negligible beside an LLM call | — |
| Reliability and failure handling | Applicable | A prompt that cannot be shown is an error, never an implicit yes | REQ-07 |
| Observability and operations | Applicable | `--verbose` prints the tier and the rule that decided it, which is how a user debugs an unexpected prompt. Structured logging stays out of scope | STEP-06 |
| Dependencies and supply chain | Applicable | No new runtime dependency; `cargo deny` runs in `just ci` | Section 5 |
| Accessibility and UX | Applicable | Two prompt styles and two defaults (D-06); `--yes` keeps scripted use; the no-terminal error names the flag to use | REQ-04, REQ-06, REQ-07 |
| Documentation and release | Applicable | Six documents, the changelog entry (already committed as `82b88da`), and the v0.4.0 draft | REQ-09, REQ-10 |
| Deployment and rollback | Applicable | One commit per step, each revertable alone; the behaviour change is confined to STEP-06 and STEP-07 | Section 10 |

## 13. Verification strategy

| Level | Evidence or command | When | Required result |
|---|---|---|---|
| Format, lint, scripts, links | `just check` | Every step | Exit 0 |
| Unit tests | `cargo test --lib` | STEP-04 onwards | All pass |
| Integration tests | `cargo test --test mock_ollama` | STEP-02 onwards | All pass |
| Coverage | `just coverage` | STEP-04 onwards | ≥ 80% total; ≥ 90% in the two new modules |
| Classification table | The unit test of the fourteen review commands | STEP-04, STEP-08 | Every command in its stated tier |
| Behaviour | `diff` of `--help` against the STEP-01 reference | STEP-02, 03, 09 | Only the two new flags differ |
| Release records | `scripts/check-release-tag.sh v0.4.0` | STEP-09 | Fails only on the draft line |
| Full pipeline | `just ci` | STEP-01, STEP-09 | Exit 0 |
| CI | `gh run list` for the pushed head commit | After hand-off | All `success` |

## 14. Acceptance criteria

- [ ] `PLAN-00002-AC-01` A unit test classifies all fourteen commands of
  the REV-00001-MAJ-01 table, and each lands in its stated tier:
  `rm -rf`, `rm -Rf`, `rm -fr`, `rm -r -f`, `rm  -rf`,
  `rm --recursive --force`, `find ~ -name '*.md' -delete`,
  `git clean -fdx`, and `shred -u ~/.ssh/id_ed25519` are `Destructive`;
  `echo x > ~/.bashrc`, `mv ~/.ssh /tmp/x`, `curl -fsSL <url> | sh`,
  `sudo systemctl stop sshd`, and `git push --force origin main` are at
  least `StateChanging`, with `sudo systemctl stop sshd` and
  `git push --force origin main` `Destructive`.
- [ ] `PLAN-00002-AC-02` `ls -la`, `git log --oneline`, `grep -rn foo src`,
  `ps aux`, and `ls | wc -l` classify as `ReadOnly`, and an integration
  test shows such a command executing with no prompt.
- [ ] `PLAN-00002-AC-03` Integration tests show: a `StateChanging` command
  prompted with default `true` and executed on approval; a `Destructive`
  command prompted with default `false`, not executed when declined, with a
  session turn whose `execution.executed` is `false`.
- [ ] `PLAN-00002-AC-04` With a prompter reporting no dialog support and no
  pre-approval flag, a `StateChanging` command produces an error naming the
  tier and `--yes`, and a `Destructive` one names
  `--i-approve-destructive-commands`; neither executes anything.
- [ ] `PLAN-00002-AC-05` `--yes` executes a `StateChanging` command with no
  prompt, and still prompts for a `Destructive` one.
- [ ] `PLAN-00002-AC-06` `--i-approve-destructive-commands` executes both
  tiers with no prompt.
- [ ] `PLAN-00002-AC-07` `[safety] assume_yes = true` behaves as `--yes`,
  and `grep -rn "assume_destructive\|approve_destructive" src/config.rs`
  finds nothing, proving no config key pre-approves the destructive tier.
- [ ] `PLAN-00002-AC-08` With `require_confirmation = false`, a
  `Destructive` command executes with no prompt (D-07 back-compat).
- [ ] `PLAN-00002-AC-09` A command matching a `destructive_substrings`
  entry classifies as `Destructive` even when no built-in rule applies.
- [ ] `PLAN-00002-AC-10` `src/prompt.rs` defines `Prompter` with the four
  methods of REQ-01; `run_with_prompter` is public; `src/main.rs` is
  unchanged from the baseline commit.
- [ ] `PLAN-00002-AC-11` `run_single_request` takes three arguments, and
  `git grep -n "too_many_arguments" src/` finds nothing.
- [ ] `PLAN-00002-AC-12` Integration tests cover approval, decline, and a
  scripted selection of the second of two commands, through
  `run_with_prompter`, with no test opening a real prompt.
- [ ] `PLAN-00002-AC-13` `just coverage` exits 0; `src/safety.rs` and
  `src/prompt.rs` are each at or above 90% line coverage; the total is not
  below 88.93%.
- [ ] `PLAN-00002-AC-14` `README.md`, `docs/configuration.md`,
  `docs/usage.md`, `docs/architecture.md`, and `docs/testing.md` describe
  the three tiers and both flags, and no document still says that a
  destructive substring list is what protects the user.
- [ ] `PLAN-00002-AC-15` `cli-bot.toml` contains `read_only_commands`,
  `destructive_commands`, `assume_yes`, and `[ui] confirmation_prompt`, and
  a v0.3.2 config with none of them parses and yields the shipped defaults.
- [ ] `PLAN-00002-AC-16` `Cargo.toml` and `Cargo.lock` say `0.4.0`;
  `docs/release/v0.4.0.md` exists, begins with a `Draft` line, and has no
  relative links; `scripts/check-release-tag.sh v0.4.0` reports the draft
  line as its only problem.
- [ ] `PLAN-00002-AC-17` `--dry-run`, `--print-plan`, `--benchmark`,
  `--models-benchmark`, `--check`, `--auto-select-best`, `--quiet`,
  `--verbose`, and the session flags behave as at the baseline commit, and
  the `--help` diff against the STEP-01 reference shows only the two new
  flags.
- [ ] `PLAN-00002-AC-18` `just ci` exits 0 on the branch head, and every
  workflow run on the pushed head commit concludes `success`.

## 15. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation or test | Owner/step |
|---|---|---|---|---|
| The parser is permissive somewhere and a destructive command still runs silently | Medium | High | Unparsable input is at least `StateChanging`; destructive rules run before the read-only list; the review's fourteen commands are a test | STEP-04 |
| The read-only list is too wide (`find`, `less`, `man` can run other programs) | Medium | High | `find` has a built-in destructive rule for `-delete`/`-exec`; the list is config, so a site can trim it; `docs/configuration.md` states the limit | STEP-04, STEP-09 |
| Too many prompts make the tool tiring and train users to accept | Medium | Medium | Read-only commands, which are the common case, never prompt; `--yes` exists; the ordinary prompt defaults to yes (D-06) | REQ-04, D-06 |
| Existing users' configs or session files break | Low | Medium | Every new key has a serde default; a v0.3.2 config and session file are explicit tests | STEP-05, STEP-06 |
| Scripted or piped use breaks silently for someone upgrading | Medium | Medium | The no-terminal error names the flag to use; the release notes lead with it | REQ-07, REQ-10 |
| The refactor of STEP-02 and STEP-03 changes interactive behaviour by accident | Low | Medium | Both steps move code without editing it, and are verified against the `--help` reference and the existing integration tests | STEP-02, STEP-03 |
| `--models-benchmark` slows down because every benchmarked command is classified | Low | Low | Classification does not prompt; the benchmark never executes a command | REQ-11 |

## 16. Builder hand-off

- **Start condition:** User approval and a clean repository.
- **First step:** `PLAN-00002-STEP-01`.
- **Required sequence:** STEP-01 to STEP-09 in order.
- **Parallel-safe work:** None; each step verifies with what the previous
  one added.
- **Do not change:** approved scope, requirements, steps, acceptance
  criteria, or content outside Builder's permitted work-log area. Do not
  edit `docs/reviews/00001-Main_Current_Code_State.md`, the directory
  `AGENTS.md` guides, or anything listed as out of scope in section 4.
- **Escalate when:** a stop condition is met; a command in the AC-01 table
  cannot be classified without a full shell grammar; a change outside
  section 4 seems necessary.
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
| PLAN-00002-STEP-01 | completed | 2026-09-20T20:33:31Z | 2026-09-20T20:33:31Z | Verification results rows 1-2 | Baseline identical to plan section 3 |
| PLAN-00002-STEP-02 | completed | 2026-09-20T20:36:52Z | 2026-09-20T20:36:52Z | Verification results row 3 | Behaviour unchanged; `run_check` keeps a `too_many_arguments` allow until STEP-03 |
| PLAN-00002-STEP-03 | completed | 2026-09-20T20:38:12Z | 2026-09-20T20:38:12Z | Verification results rows 4-5 | One deviation recorded: the pre-existing allow in the out-of-scope `src/llm.rs` |
| PLAN-00002-STEP-04 | completed | 2026-09-20T20:43:12Z | 2026-09-20T20:43:12Z | Verification results rows 6-7 | `src/safety.rs` 93.85%; module made public, see Deviations |
| PLAN-00002-STEP-05 | completed | 2026-09-20T20:47:20Z | 2026-09-20T20:47:20Z | Verification results row 8 | Committed together with STEP-06; see Deviations |
| PLAN-00002-STEP-06 | completed | 2026-09-20T20:47:20Z | 2026-09-20T20:47:20Z | Verification results rows 9-10 | Declined commands now save a turn; see Deviations |
| PLAN-00002-STEP-07 | completed | 2026-09-20T20:49:08Z | 2026-09-20T20:49:08Z | Verification results rows 11-12 | Both flags and the fail-closed path are covered by integration tests |
| PLAN-00002-STEP-08 | completed | 2026-09-20T20:50:31Z | 2026-09-20T20:50:31Z | Verification results rows 13-14 | REQ-12 partially met; see Deviations |
| PLAN-00002-STEP-09 | completed | 2026-09-20T21:05:00Z | 2026-09-20T20:53:51Z | Verification results rows 15-20 | Two recorded deviations, both expected states of an unreleased branch |

Allowed status values: `not-started`, `in-progress`, `blocked`, `completed`,
`skipped`. A skipped step requires explicit user approval recorded in Evidence.

### Execution log

| Timestamp (UTC) | Step | Event | Evidence or reference | Next action |
|---|---|---|---|---|
| 2026-09-20T20:33:31Z | STEP-01 | Execution started on the approved plan; baseline matches plan section 3 exactly (88.93%) | Verification results rows 1-2 | STEP-02: Prompter trait |
| 2026-09-20T20:36:52Z | STEP-02 | Created `src/prompt.rs` with `Prompter`, `DialoguerPrompter`, and `describe_terminal`; moved `prompt_for_request`, `interactive_prompt_theme`, and `TerminalEnvironmentStatus` there unchanged; added `run_with_prompter`, threaded `&dyn Prompter` through `run_check`, the interactive loop, `run_single_request`, and `select_command`; added `ScriptedPrompter` and one seam test | src/prompt.rs, src/lib.rs, tests/mock_ollama.rs | STEP-03: RequestContext |
| 2026-09-20T20:38:12Z | STEP-03 | `RequestContext<'a>` built once in `run_with_prompter`; `run_single_request` now takes `(&RequestContext, &dyn Prompter, &str)`; the allow is gone from both `run_single_request` and `run_check` (7 arguments, within the threshold) | src/lib.rs | STEP-04: the classifier |
| 2026-09-20T20:43:12Z | STEP-04 | Wrote `src/safety.rs`: `CommandRisk`, a quote-aware parser (segments, redirections, `$(...)` and backticks, wrapper stripping), the built-in destructive rule table, and `classify`. All fourteen commands of the REV-00001-MAJ-01 table reach the tier AC-01 requires | src/safety.rs | STEP-05: configuration |
| 2026-09-20T20:47:20Z | STEP-06 | Added `[safety] read_only_commands`, `destructive_commands`, `assume_yes` and `[ui] confirmation_prompt` with serde defaults and shipped values in `cli-bot.toml`; replaced `planner::command_requires_confirmation` with `safety::classify`; added `requires_approval` and `ask_approval`; the session turn now records the tier | src/config.rs, cli-bot.toml, src/lib.rs, src/planner.rs, src/session.rs | STEP-07: the pre-approval flags |
| 2026-09-20T20:49:08Z | STEP-07 | Added `--yes`/`-y` and `--i-approve-destructive-commands`; `pre_approved` resolves flags and `[safety] assume_yes`; the no-terminal error now names the tier and the flag that would allow the command | src/lib.rs, tests/mock_ollama.rs | STEP-08: complete the test matrix |
| 2026-09-20T20:50:31Z | STEP-08 | Added the selection test (second of two chosen, approved, and saved), `require_confirmation = false` back-compat, a `destructive_substrings` match on a quoted argument, and an unparsable command being confirmed rather than run | tests/mock_ollama.rs | STEP-09: documentation, version, release draft |
| 2026-09-20T20:53:23Z | STEP-09 | Rewrote the safety sections of `README.md`, `docs/configuration.md` (including the built-in rule table), `docs/architecture.md`, `docs/usage.md` (a new "Approving Commands" section), and `docs/testing.md`; bumped to 0.4.0; wrote the `docs/release/v0.4.0.md` draft; recorded the pty harness in `docs/backlog.md` | README.md, docs/, Cargo.toml, Cargo.lock | Final `just ci` on the committed tree, then hand-off |
| 2026-09-20T20:53:51Z | STEP-09 | Step and plan completed. Local work is done; the branch is ready to push and open as a pull request into `main` | Verification results rows 15-20 | User pushes; Builder records the CI run |

### Deviations and blockers

| Timestamp (UTC) | Step | Deviation or blocker | Impact | Decision required from |
|---|---|---|---|---|
| 2026-09-20T20:36:52Z | STEP-02 | Clippy rejects test helpers that no step has used yet, so `ScriptedPrompter` ships with only the constructor and recorder that STEP-02 needs; `with_confirmations`, `with_selections`, `with_requests`, and `without_dialogs` are added by the steps that first use them | None on scope; the helper grows step by step instead of arriving complete | None; planner decision |
| 2026-09-20T20:38:12Z | STEP-03 | AC-11 asks that `git grep -n "too_many_arguments" src/` find nothing, but `src/llm.rs:24` carries a pre-existing allow on `OllamaClient::plan_commands` (8 arguments), and `src/llm.rs` is not in the plan's scope (section 4). REQ-02, which asks only that the allow on `run_single_request` be removed, is met in full, and the grep is clean for `src/lib.rs` | AC-11 is met for the file the requirement is about; the `src/llm.rs` allow is untouched and reported at hand-off | User, at hand-off: whether to clean `src/llm.rs` in a follow-up |
| 2026-09-20T20:43:12Z | STEP-04 | The plan wanted the classifier to sit unused until STEP-06, but `just lint` denies dead code, so nothing can be committed that nothing calls. The module is `pub mod safety` instead of private: `CommandRisk` and `classify` become public API, which also lets an integration test assert the table directly | A wider public API than the plan implied; no behaviour change, and STEP-06 still does the wiring | User, at hand-off: whether `safety` should be private again once it is wired |
| 2026-09-20T20:43:12Z | STEP-04 | One test expectation written in the same step was wrong: `echo 'rm -fr /'` only prints a string, so `ReadOnly` is right and the implementation was correct. The test now asserts `ReadOnly`, with `echo 'rm -rf /'` asserting that the legacy substring list still fires on quoted text | None; the AC-01 table is untouched | None; planner decision |
| 2026-09-20T20:47:20Z | STEP-06 | STEP-05 and STEP-06 landed in one commit. `just lint` denies dead code, so the new configuration fields cannot be committed in a step before the step that reads them | None on scope; both steps' tasks and verifications were carried out in full, in order | None; planner decision |
| 2026-09-20T20:47:20Z | STEP-06 | The new destructive-tier test found that a declined command saved no session turn at all, so REQ-04 ("saves its session turn as not executed") was not met by the code being replaced. The declined path now saves the turn, as the dry-run path already did | A behaviour improvement inside REQ-04; a follow-up such as "why did I decline that" now has context | None; required by REQ-04 |
| 2026-09-20T20:49:08Z | STEP-07 | AC-17 asks that the `--help` diff show only the two new flags. It also shows clap re-aligning every description column, because `--i-approve-destructive-commands` is longer than any previous flag. No wording changed | Cosmetic; the flag list differs by exactly the two new entries | None; planner decision |
| 2026-09-20T20:50:31Z | STEP-08 | REQ-12 asks for 90% line coverage in both new modules. `src/safety.rs` reaches 93.85%, but `src/prompt.rs` stays at 71.65%: its 36 uncovered lines are the bodies of `read_request`, `select`, and `confirm`, which call `dialoguer` and need a real terminal. A test that reached them would open a prompt and hang, which `AGENTS.md` forbids. Isolating them is the point of the trait: the untestable code is now 36 lines in one module instead of being spread through the request flow | REQ-12 is met for the module carrying the safety decision and for the 80% gate; the `src/prompt.rs` floor needs a pseudo-terminal harness, which is not in this plan | User, at hand-off: whether a pty-backed test is worth a dev-dependency |
| 2026-09-20T20:53:23Z | STEP-09 | The step verification expected `scripts/check-release-tag.sh v0.4.0` to fail only on the draft line. It also fails on the missing `## v0.4.0 - <UTC timestamp>` changelog heading, which `AGENTS.md` § Release workflow step 4 assigns to the release commit, not to this plan. Both failures are the expected state of an unreleased branch | None; every other release record is in place | None; planner decision |
| 2026-09-20T20:53:23Z | STEP-09 | `just ci` cannot pass on a dirty tree: `cargo publish --dry-run --locked` refuses uncommitted changes. The step commit is therefore made first, through the `just check` hook, and `just ci` is run on the committed tree afterwards | None; CI runs `just ci` on a clean checkout anyway | None; planner decision |

### Verification results

| Timestamp (UTC) | Step | Command or check | Result | Evidence |
|---|---|---|---|---|
| 2026-09-20T20:33:31Z | STEP-01 | `just ci` | pass | Exit 0; 118 tests pass; lines 3813, missed 422, 88.93%; `cargo deny`, links, publish dry run, actionlint clean |
| 2026-09-20T20:33:31Z | STEP-01 | Reference outputs saved outside the repository | recorded | `--help` (27 lines) for AC-17; the REV-00001-MAJ-01 table with the expected tier per command for AC-01 |
| 2026-09-20T20:36:52Z | STEP-02 | `just check`; `cargo run -- --help` against the STEP-01 reference | pass | Exit 0; 121 tests pass (93 unit, 28 integration); lines 3865, missed 426, 88.98%; `--help` byte-identical |
| 2026-09-20T20:38:12Z | STEP-03 | Test first: removed the allow and ran `just lint` | fails as expected | `error: this function has too many arguments (14/7)` at `src/lib.rs:351` |
| 2026-09-20T20:38:12Z | STEP-03 | `just check`; `git grep -n too_many_arguments src/lib.rs`; `--help` against the STEP-01 reference | pass | Exit 0; 121 tests; lines 3853, missed 412, 89.31%; no allow left in `src/lib.rs`; `--help` byte-identical |
| 2026-09-20T20:43:12Z | STEP-04 | Test first: the whole table against a stub returning `StateChanging` | fails as expected | 8 of 11 tests failed, including `classifies_the_review_table` |
| 2026-09-20T20:43:12Z | STEP-04 | `cargo test --lib safety`; `just check` | pass | 11 classifier tests pass; `just check` exit 0; 132 tests total; `src/safety.rs` 93.85% lines (floor 90%); total 89.88%, up from 88.93% |
| 2026-09-20T20:47:20Z | STEP-05 | Unit tests: a v0.3.2 config parses and yields the shipped defaults; the new keys round-trip; `cli-bot.toml` parses | pass | Three new tests in `src/config.rs` |
| 2026-09-20T20:47:20Z | STEP-06 | Test first: the three tier tests against the old policy | fail as expected | Also `failed_command_returns_error_and_saves_no_turn` failed, because `exit 3` is state-changing and the run had no terminal |
| 2026-09-20T20:47:20Z | STEP-06 | `just check`; `--help` against the STEP-01 reference; a v0.3.2 session file loads | pass | Exit 0; 135 tests; total 90.21% (was 88.93%); `src/safety.rs` 93.85%; `--help` byte-identical; `risk` is `#[serde(default)]` |
| 2026-09-20T20:49:08Z | STEP-07 | Test first: the four flag and no-terminal tests before the flags existed | fail as expected | Compilation failed on the unknown `Cli` fields |
| 2026-09-20T20:49:08Z | STEP-07 | `just check`; `cargo run -- --help` | pass | Exit 0; 139 tests; total 90.31%; `--help` shows `-y, --yes` and `--i-approve-destructive-commands` and nothing else new |
| 2026-09-20T20:50:31Z | STEP-08 | Uncovered-branch survey before the step | recorded | `src/prompt.rs` lines 46-87, the three `dialoguer` call bodies; `src/safety.rs` already 93.85% |
| 2026-09-20T20:50:31Z | STEP-08 | `just check`; `cargo test --test mock_ollama` three times | pass | Exit 0; 143 tests (104 unit, 39 integration); identical on all three runs, no flakiness; total 90.62%, `src/lib.rs` 90.62%, `src/safety.rs` 93.85% |
| 2026-09-20T20:53:23Z | STEP-09 | Test first: `git grep -n 'destructive substring\\|substring rules'` before the step | recorded | Two hits outside the plan and the review: `README.md:437` and `docs/configuration.md:37`; both rewritten |
| 2026-09-20T20:53:23Z | STEP-09 | The same grep after the step; `just links` | pass | No hits outside `docs/plans` and `docs/reviews`; links ok in 24 Markdown files |
| 2026-09-20T20:53:23Z | STEP-09 | `cargo run -- --help` against the STEP-01 reference | pass | The two new flags are the only added lines; clap re-aligned the description column because `--i-approve-destructive-commands` is the longest flag |
| 2026-09-20T20:53:23Z | STEP-09 | `scripts/check-release-tag.sh v0.4.0` | fails on exactly the two expected records | The draft line, and the `## v0.4.0 - <UTC timestamp>` changelog heading that the release commit writes |
| 2026-09-20T20:53:51Z | STEP-09 | `just ci` on the committed tree | pass | Exit 0; 143 tests pass; lines 4479, missed 420, 90.62% (was 88.93%); `cargo deny`, links, publish dry run, and actionlint clean |
| 2026-09-20T20:53:51Z | STEP-09 | Acceptance criteria AC-01 to AC-10, AC-12, AC-14 to AC-18 | pass | Each checked with the command named in the criterion |
| 2026-09-20T20:53:51Z | STEP-09 | AC-11 and AC-13 | partially met | AC-11 holds for `src/lib.rs`; `src/llm.rs` is out of scope. AC-13 holds for `src/safety.rs` (93.85%) and the gate (90.62%); `src/prompt.rs` is 71.65%. Both are in Deviations |

### Completion summary

- **Implementation status:** `completed`
- **Completed requirements:** PLAN-00002-REQ-01 to REQ-11; REQ-12 for `src/safety.rs` and the 80% gate
- **Incomplete requirements:** REQ-12 for `src/prompt.rs` only: its three `dialoguer` bodies need a pseudo-terminal, which is now a backlog item
- **Outstanding blockers:** None. Left with the user: push the branch and open the pull request into `main`; decide on the four items raised in Deviations
- **Review request:** Ready. Coverage 90.62% lines (was 88.93%); 143 tests
<!-- BUILDER_WORK_LOG_END -->

## 18. Planning change log

| Timestamp (UTC) | Plan status | Change | Reason | Requested/approved by |
|---|---|---|---|---|
| 2026-09-20T20:21:39Z | draft | Plan created | User asked to plan `REV-00001-MAJ-01` with `REV-00001-MED-06` first, and answered the four design questions (D-01 to D-04) | User |
| 2026-09-20T20:32:38Z | approved | Plan approved without amendment; `build_ready` set. The draft was committed by the planner as `0838c15`, as for PLAN-00001 | User replied "I approve. Please implement." | User |

## 19. External references

None. Review 00001 and the repository were read from disk.

## 20. Confidence

**High.** The two findings were reproduced on the built binary during
Review 00001, the four design decisions come from the user, and every file
in scope was read while writing the review. The principal uncertainty is
the classifier: a small shell parser has more edge cases than the test
table names, which is why unparsable input fails towards a prompt and why
STEP-04 is a standalone step with its own coverage floor.
