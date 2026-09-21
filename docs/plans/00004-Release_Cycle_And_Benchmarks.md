---
title: "Delivery Plan 00004: Release Cycle And Benchmarks"
aliases:
  - "Plan 00004"
tags:
  - delivery-plan
  - implementation
  - claude-code
type: delivery-plan
plan_id: "PLAN-00004"
plan_status: approved              # draft | approved | cancelled
plan_kind: initial                 # initial | superseding
created_at: "2026-09-21T20:31:02Z"
approved_at: "2026-09-21T23:00:34Z"
planner_agent: "Claude Code"
planner_model: "anthropic/claude-opus-5"
triggered_by: user                 # user | agent:<agent-name>
request_kind: idea-and-review      # idea | review | idea-and-review | direct | unplanned-query
repository: "joelee/cli-bot"
baseline_branch: "feature/00004-release-cycle-and-benchmarks"
baseline_commit: "ffca51b2d389202a70c83108ac3aac1873b02717"
source_ideas: []
source_reviews:
  - "docs/reviews/00001-Main_Current_Code_State.md"
previous_plan: "docs/plans/00003-Session_Integrity_And_Reliability.md"
requirements_count: 15
steps_count: 8
acceptance_criteria_count: 20
blocking_decisions: 0
build_ready: true
web_research_used: false
confidence: medium                # high | medium | low

# Builder-maintained front matter. Builder may update only these keys after
# explicit user approval; Delivery Planner initializes them.
implementation_status: not-started # not-started | in-progress | blocked | completed | abandoned
builder_agent: null
builder_model: null
execution_branch: null
execution_started_at: null
execution_updated_at: null
execution_completed_at: null
current_step: null
---

# Delivery Plan 00004: Release Cycle And Benchmarks

> [!abstract] Plan status: `draft`
> v0.5.0 cuts a release from ten user actions to four: the release records
> travel in the feature pull request, and merging it makes the workflow tag,
> build, publish, and open the Homebrew pull request itself. It also reports
> tokens per second wherever cli-bot reports timings, so a model comparison
> stops being dominated by how long the model took to load. It closes the
> last eight findings of Review 00001. Ten decisions are resolved
> (section 7); none blocks, and the plan waits for user approval.

## 1. Objective and outcome

Two threads meet here.

The first is the release cycle. Releasing v0.4.1 took ten user actions and
three pull requests: push, pull request, merge, then a separate release
commit with its own push, pull request and merge, then pull, tag, push tag,
approve, and finally a third pull request in the Homebrew tap. The user
asked for this to be shorter.

The second is measurement. `docs/benchmarks/` ranks models by total
milliseconds, which on this hardware is dominated by model loading: a
measured call spent 8.6 seconds of 8.96 loading and 0.26 generating. Ollama
reports its own timings, and using them makes the comparison mean something.

Alongside both, this plan closes the eight findings of Review 00001 that
PLAN-00002 and PLAN-00003 left: `MED-07`, `LOW-03` to `LOW-07`, `INFO-01`
and `INFO-02`.

After this plan:

- Merging the feature pull request releases the version, when and only when
  the release records in it are final.
- `--benchmark` and the models benchmark report tokens per second, token
  counts, and model load time.
- The Homebrew formula pull request is opened by one command.
- Review 00001 has no open findings.

## 2. Source traceability

| Requirement | Source | Source location | Interpretation |
|---|---|---|---|
| PLAN-00004-REQ-01 | User; measured | User request 2026-09-21 "add tokens per second"; live `/api/generate` and `/api/chat` responses on Ollama 0.32.13 | Ollama's timings are captured |
| PLAN-00004-REQ-02 | User | Same request | `--benchmark` reports them |
| PLAN-00004-REQ-03 | User | Same request, "and models-benchmark" | The report and its ranking use them |
| PLAN-00004-REQ-04 | User | User request 2026-09-21: two pushes, two pull requests, pull and tag are "unnecessary" | Merge to `main` releases |
| PLAN-00004-REQ-05 | Review | REV-00001-LOW-05 | The release workflow is hardened |
| PLAN-00004-REQ-06 | User | Same request | The release records travel in the feature pull request |
| PLAN-00004-REQ-07 | User | Same request, on the Homebrew step | One command opens the tap pull request |
| PLAN-00004-REQ-08 | Review | REV-00001-MED-07 | The allocation script is in the repository again |
| PLAN-00004-REQ-09 | Review | REV-00001-LOW-03 | The shipped default config is neutral |
| PLAN-00004-REQ-10 | Review | REV-00001-LOW-04 | `--check` accepts an untagged model name |
| PLAN-00004-REQ-11 | Review | REV-00001-LOW-07 | Tests clean up their temporary folders |
| PLAN-00004-REQ-12 | Review | REV-00001-INFO-01 | The crate carries only what it needs |
| PLAN-00004-REQ-13 | Review | REV-00001-INFO-02 | Unified-memory GPUs are reported honestly |
| PLAN-00004-REQ-14 | Review | REV-00001-LOW-06 | Documents match the code; reports are gathered |
| PLAN-00004-REQ-15 | Repository | `AGENTS.md` § Non-negotiables, § Release workflow step 1 | Version, release draft, gate, no unrelated change |

## 3. Repository baseline

| Field | Value |
|---|---|
| Repository | `joelee/cli-bot` |
| Branch | `feature/00004-release-cycle-and-benchmarks`, created from `main` |
| HEAD | `ffca51b2d389202a70c83108ac3aac1873b02717` |
| Working tree at publication | Clean |
| Applicable instructions | `AGENTS.md`; `docs/plans/AGENTS.md` |

`main` is at `fc27809`, carrying v0.4.1, published to crates.io on
2026-09-21. One commit precedes this plan on the branch: the changelog entry
`ffca51b`.

Measured on `main` while planning:

- `just ci`: exit 0. 167 tests; line coverage **91.82%** (4940 lines, 404
  missed).
- `src/error.rs` 100%, `src/shell.rs` 98.07%, `src/safety.rs` 94.02%,
  `src/llm.rs` 95.46%, `src/session.rs` 90.95%, `src/lib.rs` 92.38%,
  `src/prompt.rs` 71.65%, `src/main.rs` 0%.
- The v0.4.1 Release run concluded `success` with four assets attached, so
  the fix made after v0.4.0 is proven.
- `main` has no branch protection: the API returns `Branch not protected`.
- The `release` environment has `joelee` as a required reviewer and
  `deployment_branch_policy: null`, so the comment in `release.yml` claiming
  it "accepts only `v*` tags" is untrue.
- On Ollama 0.32.13 both `/api/generate` and `/api/chat` return
  `eval_count`, `eval_duration`, `prompt_eval_count`,
  `prompt_eval_duration`, `load_duration`, and `total_duration`; the
  durations are nanoseconds.

## 4. Scope

### In scope

- `src/llm.rs`: capturing Ollama's timings and returning them with each
  answer; the `--check` model-name comparison.
- `src/lib.rs`: the benchmark report, the models benchmark report and its
  ranking, and GPU memory reporting.
- `src/config.rs`, `cli-bot.toml`, and a new shipped default template.
- `.github/workflows/release.yml`: the trigger, the tag, permissions,
  pinning, and the comment.
- `scripts/check-release-tag.sh`, `scripts/update-homebrew-formula.sh`.
- `.agents/skills/allocating-report-numbers/`, tracked again.
- `Cargo.toml`: an `include` list; the version.
- `tests/mock_ollama.rs`: a temporary-folder guard, and the new coverage.
- `AGENTS.md`, `README.md`, `docs/configuration.md`, `docs/usage.md`,
  `docs/testing.md`, `docs/developer-guide.md`, `docs/publishing.md`,
  `docs/crates-release.md`, `docs/session-memory.md`, `docs/backlog.md`.
- `docs/benchmarks/`, holding the reports moved from `docs/`.
- New `docs/release/v0.5.0.md` draft.

### Out of scope

- **Measuring whether a command is correct.** The ranking gains a fairer
  speed measure, not a quality one. Choosing a default model on speed alone
  would repeat the mistake Review 00001 found in the v0.3.2 report; a
  correctness measure needs its own idea and plan.
- **Changing the default model.** Out of scope for the same reason.
- **Branch protection on `main`** and the `release` environment's reviewer
  and branch policy: GitHub settings, not code. Section 16 hands them over.
- **A cross-repository token** for the Homebrew tap (D-04).
- Regenerating any benchmark report: it needs the owner's hardware.
- Structured logging, the config lookup order, rustdoc coverage, Windows
  support, and a pseudo-terminal harness for `src/prompt.rs`.

## 5. Constraints and preserved decisions

- The safety model of v0.4.0 and the reliability work of v0.4.1 are not
  weakened.
- An existing `cli-bot.toml` and existing session files keep working.
- No new runtime dependency. A dev-dependency is allowed only if a test
  cannot be written without one, and each is recorded.
- Tests never reach a real Ollama service or the network, never open a
  terminal prompt, and execute only harmless commands.
- The agent does not push, tag, or publish. That does not change: what
  changes is how much the user has to do after merging.
- A release must still be impossible by accident: the workflow releases only
  when the records in the merge are final.

## 6. Assumptions

None. Unresolved matters are recorded as decisions and block approval when
material.

## 7. Decisions and blockers

| ID | Decision or blocker | Resolution | Owner | Status |
|---|---|---|---|---|
| D-01 | How a release is triggered | One workflow on `push: branches: [main]`. It reads the version from `Cargo.toml`, exits quietly when the tag exists or `check-release-tag.sh` fails, and otherwise creates and pushes the tag itself before building. A separate auto-tag workflow cannot work: a tag pushed with `GITHUB_TOKEN` does not trigger another workflow run | Planner | Resolved |
| D-02 | The crates.io approval gate | **Kept**, confirmed by the user at approval on 2026-09-21: "Keep crates.io approval gate." Under D-01 merging the pull request becomes the only other human decision before an irreversible publish, so the gate is the one deliberate confirmation that the builds are green | User | Resolved |
| D-03 | The changelog version heading | A date, `## v0.5.0 - 2026-09-21`, not a UTC timestamp. Once the heading is written in the feature branch, a timestamp's precision is false. `check-release-tag.sh` accepts either and keeps requiring the heading | Planner | Resolved |
| D-04 | Automating the Homebrew pull request | From the user's machine, by extending `scripts/update-homebrew-formula.sh` to branch, commit, push and open the pull request. Doing it from CI needs a token with write access to another repository, living in this one, for a step that runs a few times a year | Planner | Resolved |
| D-05 | Branch protection | A GitHub setting the agent cannot make. It matters more under D-01, because a direct push to `main` carrying final records would publish with no pull request at all. Handed over in section 16 | Planner | Resolved |
| D-06 | How tokens per second is computed | `eval_count / (eval_duration / 1e9)`, from the response. `load_duration` is reported separately rather than folded in, because it measures the model arriving, not the model thinking. All fields are optional: a server that omits them leaves the figures blank rather than failing | Planner | Resolved |
| D-07 | What the ranking sorts on | Success rate first, then tokens per second, then average total milliseconds. Tokens per second is fair between a short answer and a long one, which total milliseconds is not. The report states plainly that none of this measures whether a command was correct | Planner | Resolved |
| D-08 | Version | v0.5.0. `--benchmark` gains output, the shipped default configuration changes, and `--check` accepts names it used to reject; a minor bump says so | Planner | Resolved |
| D-09 | MED-07 | The allocation script and its `SKILL.md` are tracked in this repository again. They are 153 lines with no dependencies, five tracked documents require them, and a symlink into `../opencode-template` works on one machine only | Planner | Resolved |
| D-10 | The benchmark reports | All four move to `docs/benchmarks/`, renamed by the version they measured; the two unnamed legacy ones become `v0.2.1` and `v0.3.0` by their content. Nothing is deleted: they are evidence, and one of them is cited by a released version's notes | Planner | Resolved |

## 8. Affected architecture and components

### The release cycle

| Stage | Now | After |
|---|---|---|
| Release records | A second commit, on its own branch, with its own pull request | Written in the feature branch, in the pull request that does the work |
| Tag | User pulls, tags, pushes | The Release workflow creates it |
| Trigger | `push: tags: ["v*"]` | `push: branches: [main]`, gated on `check-release-tag.sh` |
| Publish | After the `release` environment is approved | Unchanged (D-02) |
| Homebrew | Agent edits the formula; user branches, commits, pushes, opens a pull request | `scripts/update-homebrew-formula.sh vX.Y.Z --open-pr` does all of it |
| User actions | 10 | 4: push, open the pull request, merge, approve |

The workflow's first job becomes:

```bash
version="$(...Cargo.toml...)"
tag="v$version"
if git ls-remote --exit-code --tags origin "$tag" >/dev/null 2>&1; then
    echo "::notice::$tag is already released"; echo "release=false" >> "$GITHUB_OUTPUT"; exit 0
fi
if ! scripts/check-release-tag.sh "$tag"; then
    echo "::notice::release records for $tag are not final; nothing to release"
    echo "release=false" >> "$GITHUB_OUTPUT"; exit 0
fi
echo "release=true" >> "$GITHUB_OUTPUT"
```

Every later job is conditional on that output. The tag is created in the
same job, after the check and before the build.

### Ollama timings

`GenerateResponse` and `ChatResponse` gain the six optional fields. The
client returns them beside the generated text, in a `Timings` struct:

```rust
pub struct Timings {
    pub eval_count: Option<u64>,
    pub eval_duration_ns: Option<u64>,
    pub prompt_eval_count: Option<u64>,
    pub prompt_eval_duration_ns: Option<u64>,
    pub load_duration_ns: Option<u64>,
    pub total_duration_ns: Option<u64>,
}

impl Timings {
    pub fn tokens_per_second(&self) -> Option<f64>;
    pub fn load_ms(&self) -> Option<u128>;
}
```

`--benchmark` gains `tokens_per_second`, `generated_tokens`,
`prompt_tokens`, and `model_load_ms`, each printed only when the server
supplied it. The models benchmark report gains a tokens-per-second column in
its model summary and ranking, and the per-result timings in its detailed
section.

## 9. Requirement catalogue

### PLAN-00004-REQ-01 — Ollama's timings are captured

- **Requirement:** both response types deserialize `eval_count`,
  `eval_duration`, `prompt_eval_count`, `prompt_eval_duration`,
  `load_duration`, and `total_duration` as optional, and the planner and
  the text fallback return a `Timings` alongside their answer. A response
  without them parses and yields `None`.
- **Rationale:** the figures exist already and are thrown away.
- **Source:** user request; live responses recorded in section 3.
- **Acceptance evidence:** AC-01.

### PLAN-00004-REQ-02 — `--benchmark` reports them

- **Requirement:** `--benchmark` prints `tokens_per_second` to one decimal,
  `generated_tokens`, `prompt_tokens`, and `model_load_ms`, each omitted
  when the server did not supply it. The existing `planning_ms`,
  `execution_ms`, and `total_ms` lines keep their wording.
- **Rationale:** the user asked for it where cli-bot reports timings.
- **Source:** user request.
- **Acceptance evidence:** AC-02.

### PLAN-00004-REQ-03 — The models benchmark reports and ranks on them

- **Requirement:** the model summary and ranking tables gain an average
  tokens-per-second column; the ranking sorts by success rate, then tokens
  per second, then average total milliseconds (D-07). The detailed section
  records each call's tokens per second and load time. The report states
  that it measures speed and not whether a command was correct.
- **Rationale:** total milliseconds is dominated by model loading.
- **Source:** user request; the measurement in section 1.
- **Acceptance evidence:** AC-03, AC-04.

### PLAN-00004-REQ-04 — Merging releases

- **Requirement:** `release.yml` triggers on `push: branches: [main]` and
  implements the gate of section 8. When the gate passes it creates and
  pushes `vX.Y.Z`, then builds, publishes, and creates the GitHub release
  exactly as it does today. When it does not, the run succeeds having done
  nothing. `workflow_dispatch` is added so a release can be retried.
- **Rationale:** removes pull, tag and push tag, and the second pull
  request, without letting a release happen by accident.
- **Source:** user request.
- **Acceptance evidence:** AC-05, AC-06, AC-07.

### PLAN-00004-REQ-05 — The release workflow is hardened

- **Requirement:** `permissions: contents: read` at the top, with
  `contents: write` only on the jobs that create the tag and the release.
  Third-party actions in `release.yml` are pinned to commit hashes with the
  version in a comment. The `crates` job, which holds the token, does not
  restore a build cache. The false comment about the environment accepting
  only `v*` tags is replaced with what is actually true.
- **Rationale:** REV-00001-LOW-05, and a claim in the code that is untrue.
- **Source:** REV-00001-LOW-05; the API result in section 3.
- **Acceptance evidence:** AC-08.

### PLAN-00004-REQ-06 — Release records travel in the feature pull request

- **Requirement:** `AGENTS.md` § Release workflow is rewritten: the plan's
  documentation step writes the final changelog heading and the release
  notes with no draft line, in the feature branch. `check-release-tag.sh`
  accepts `## vX.Y.Z - <date>` as well as a timestamp (D-03), and keeps
  rejecting a missing heading, a draft line, relative links, and
  pre-release wording.
- **Rationale:** the separate release commit exists only because the rules
  order it that way.
- **Source:** user request.
- **Acceptance evidence:** AC-09, AC-10.

### PLAN-00004-REQ-07 — One command opens the Homebrew pull request

- **Requirement:** `scripts/update-homebrew-formula.sh vX.Y.Z` gains
  `--open-pr`, which creates a branch from the tap's default branch, commits
  the formula change, pushes it, and opens a pull request with `gh`. Without
  the flag it behaves as it does now. It refuses to run on the tap's default
  branch, and reports what it did.
- **Rationale:** five manual steps for a mechanical change.
- **Source:** user request.
- **Acceptance evidence:** AC-11.

### PLAN-00004-REQ-08 — The allocation script is in the repository

- **Requirement:** `.agents/skills/allocating-report-numbers/SKILL.md` and
  `allocate-report.sh` are tracked again, byte-identical to the copies in
  `../opencode-template`, and `.agents` is removed from
  `.git/info/exclude`'s effect by being tracked. `just check` fails when a
  script path named in `AGENTS.md` or a directory guide does not exist.
- **Rationale:** REV-00001-MED-07: five tracked documents tell the writer to
  run a script the repository does not contain.
- **Source:** REV-00001-MED-07; D-09.
- **Acceptance evidence:** AC-12.

### PLAN-00004-REQ-09 — The shipped default config is neutral

- **Requirement:** the template written to `~/.config` on first run is a
  separate file from the repository's own `cli-bot.toml`: no
  `preferred_editor`, so `$EDITOR` applies, and a benchmark list of only the
  default model. `is_known_editor` matches the first word of a value such as
  `code --wait`. A missing editor is a warning in `--check`, not a failure.
- **Rationale:** REV-00001-LOW-03: a fresh install fails `--check` without
  Neovim and would try to load about 250 GB of models.
- **Source:** REV-00001-LOW-03.
- **Acceptance evidence:** AC-13, AC-14.

### PLAN-00004-REQ-10 — `--check` accepts an untagged model name

- **Requirement:** when the configured model name contains no `:`,
  `check_service` also accepts `<name>:latest`.
- **Rationale:** REV-00001-LOW-04: `model = "gemma4"` works for requests but
  is reported missing.
- **Source:** REV-00001-LOW-04.
- **Acceptance evidence:** AC-15.

### PLAN-00004-REQ-11 — Tests clean up after themselves

- **Requirement:** `unique_temp_dir` returns a guard that removes the folder
  on `Drop`, and every test uses it. After a full `just check`, the system
  temporary directory holds no `cli-bot-*` folder created by the run.
- **Rationale:** REV-00001-LOW-07: about 60 folders per `just check`.
- **Source:** REV-00001-LOW-07.
- **Acceptance evidence:** AC-16.

### PLAN-00004-REQ-12 — The crate carries only what it needs

- **Requirement:** `Cargo.toml` gains an `include` list covering `src/**`,
  `cli-bot.toml`, the default template, `README.md`, `LICENSE`,
  `CHANGELOG.md`, and the Cargo files. `cargo package --list` shows no file
  under `docs/`, `.github/`, `scripts/`, `.agents/`, or `tests/`.
- **Rationale:** REV-00001-INFO-01: 26 of 53 packaged files are repository
  records.
- **Source:** REV-00001-INFO-01.
- **Acceptance evidence:** AC-17.

### PLAN-00004-REQ-13 — Unified-memory GPUs are reported honestly

- **Requirement:** the benchmark host section reports
  `mem_info_vram_total` labelled as dedicated VRAM, and, when
  `mem_info_gtt_total` exists, the shared total as well.
- **Rationale:** REV-00001-INFO-02: the reports say `GPU VRAM: 0.5 GiB` on a
  125 GiB machine.
- **Source:** REV-00001-INFO-02.
- **Acceptance evidence:** AC-18.

### PLAN-00004-REQ-14 — Documents match, reports are gathered

- **Requirement:** `docs/usage.md` and `docs/session-memory.md` stop
  crediting session memory to the unpublished `0.3.1`. The four benchmark
  reports move to `docs/benchmarks/` with version names (D-10), and every
  link to them is updated, including the one in `docs/release/v0.4.0.md`,
  which is pinned to a tag and must keep resolving.
- **Rationale:** REV-00001-LOW-06, and four reports in one folder.
- **Source:** REV-00001-LOW-06.
- **Acceptance evidence:** AC-19.

### PLAN-00004-REQ-15 — Version, release draft, gate, nothing unrelated

- **Requirement:** `Cargo.toml` and `Cargo.lock` move to 0.5.0, and
  `docs/release/v0.5.0.md` is written **final**, not as a draft, together
  with the changelog heading (REQ-06). `just ci` exits 0 with line coverage
  at or above 91.82%. The classification, approval, and session behaviour of
  v0.4.0 and v0.4.1 are unchanged, and the `--help` diff shows only what
  this plan adds.
- **Rationale:** the gate, and this plan is the first to use its own new
  release flow.
- **Source:** `AGENTS.md`; D-08.
- **Acceptance evidence:** AC-20.

## 10. Delivery strategy

The tokens-per-second work comes first: it is self-contained, it is what the
user asked for most recently, and it does not touch the release machinery.

The release-cycle work follows, in the order that keeps the repository
releasable at every commit: the workflow gate before the rules change, so
that if the plan stops halfway a release still works the old way.

The small findings are gathered into one step, because each is a few lines
and splitting them would cost more in ceremony than the changes themselves.

This plan is the first to write its own release records final in the feature
branch (REQ-06, REQ-15). That makes its own merge the first live test of
REQ-04. Section 16 says what to watch.

Each step is test-first, ends in one commit
`build: complete PLAN-00004-STEP-NN - <title>` once its verification passes,
and keeps the work log current. As in PLAN-00003, `just ci` is run after the
step commit, because its publish dry run needs a clean tree.

## 11. Detailed implementation steps

### PLAN-00004-STEP-01 — Baseline

- **Objective:** confirm the starting state and lock the references.
- **Requirements:** `PLAN-00004-REQ-15`
- **Depends on:** None
- **Affected components:** this plan's Builder fields only
- **Preconditions:** plan approved; worktree clean; on the plan's branch.
- **Test or evidence first:** run `just ci`; save `cargo run -- --help`,
  `cargo package --list`, and a `--benchmark` run against the mock server
  outside the repository as the AC-20, AC-17 and AC-02 references.
- **Implementation tasks:**
  1. Set the Builder front-matter fields.
  2. Record the baseline in the work log.
- **Documentation/configuration/operations:** none.
- **Verification:** `just ci` exits 0; coverage within 0.5 points of 91.82%.
- **Completion criteria:** the baseline is recorded and committed.
- **Rollback or recovery:** none needed.
- **Builder stop conditions:** the baseline differs from section 3.

### PLAN-00004-STEP-02 — Capture Ollama's timings

- **Objective:** keep the numbers the server already sends.
- **Requirements:** `PLAN-00004-REQ-01`
- **Depends on:** `PLAN-00004-STEP-01`
- **Affected components:** `src/llm.rs`, `src/lib.rs` (call sites only),
  `tests/mock_ollama.rs`
- **Preconditions:** STEP-01 committed.
- **Test or evidence first:** unit tests that a response carrying the six
  fields yields them and the derived figures, that a response without them
  yields `None` throughout, and that `tokens_per_second` is `None` rather
  than infinite when `eval_duration` is zero.
- **Implementation tasks:**
  1. Add the optional fields to both response types.
  2. Add `Timings` with `tokens_per_second` and `load_ms`.
  3. Return it from `plan_commands` and `answer_unresolved` without
     changing what they already return.
- **Documentation/configuration/operations:** STEP-08 documents.
- **Verification:** `just check` exits 0; a mock response with no timing
  fields still parses.
- **Completion criteria:** verification passes; no output changes yet.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** the two response shapes cannot share one
  `Timings` without duplicating the fields; record it and use a shared
  `#[serde(flatten)]` struct.

### PLAN-00004-STEP-03 — Report tokens per second

- **Objective:** put the numbers where a user and a report can see them.
- **Requirements:** `PLAN-00004-REQ-02`, `PLAN-00004-REQ-03`,
  `PLAN-00004-REQ-13`
- **Depends on:** `PLAN-00004-STEP-02`
- **Affected components:** `src/lib.rs`, `tests/mock_ollama.rs`
- **Preconditions:** STEP-02 committed.
- **Test or evidence first:** an integration test that `--benchmark` against
  a mock server returning timings prints tokens per second and the counts,
  and that it omits them when the server does not; unit tests of the
  ranking order for models that differ only in tokens per second, and of
  the GPU memory labels.
- **Implementation tasks:**
  1. Extend the benchmark report with the four figures.
  2. Extend the models benchmark summary, ranking and detailed sections,
     and add the sentence about what is not measured.
  3. Report dedicated VRAM and, when present, shared memory (REQ-13).
- **Documentation/configuration/operations:** STEP-08 documents.
- **Verification:** `just check` exits 0; the rendered report contains a
  tokens-per-second column and the caveat sentence.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** none expected.

### PLAN-00004-STEP-04 — Release on merge

- **Objective:** the workflow decides, tags, and releases.
- **Requirements:** `PLAN-00004-REQ-04`, `PLAN-00004-REQ-05`
- **Depends on:** `PLAN-00004-STEP-03`
- **Affected components:** `.github/workflows/release.yml`
- **Preconditions:** STEP-03 committed.
- **Test or evidence first:** extract the gate's shell into a file and run
  it under `bash -n` and against three cases with a stubbed
  `git ls-remote`: tag already present, records not final, records final.
  `just lint-workflows` before and after.
- **Implementation tasks:**
  1. Change the trigger, add `workflow_dispatch`, and add the gate job with
     its `release` output.
  2. Make every later job conditional on it; create and push the tag in the
     gate job after the check.
  3. Apply REQ-05: scoped permissions, pinned actions, no cache on the
     `crates` job, and a true comment about the environment.
- **Documentation/configuration/operations:** STEP-08 documents.
- **Verification:** `just lint-workflows` exits 0; the extracted gate
  behaves correctly in all three cases; `grep` shows no unpinned
  third-party action in `release.yml`.
- **Completion criteria:** verification passes. The live run is this plan's
  own merge; section 16 covers it.
- **Rollback or recovery:** `git revert` the step commit. Until it is
  merged, releases still work the old way.
- **Builder stop conditions:** the gate cannot be expressed without a
  third-party action; record it and keep the manual tag.

### PLAN-00004-STEP-05 — Records in the feature branch, and the tap command

- **Objective:** remove the second pull request and the tap ceremony.
- **Requirements:** `PLAN-00004-REQ-06`, `PLAN-00004-REQ-07`
- **Depends on:** `PLAN-00004-STEP-04`
- **Affected components:** `AGENTS.md`, `scripts/check-release-tag.sh`,
  `scripts/update-homebrew-formula.sh`
- **Preconditions:** STEP-04 committed.
- **Test or evidence first:** run `check-release-tag.sh` against a changelog
  heading with a date, with a timestamp, and with neither, expecting pass,
  pass, fail. Run `update-homebrew-formula.sh --open-pr` with `gh` stubbed
  on `PATH`, asserting the branch, the commit, and the `gh pr create`
  arguments, and that it refuses on the default branch.
- **Implementation tasks:**
  1. Accept a date in the changelog heading.
  2. Rewrite `AGENTS.md` § Release workflow for the new cycle, and
     § New feature workflow where it refers to the release records.
  3. Add `--open-pr` to the Homebrew script.
- **Documentation/configuration/operations:** `AGENTS.md` is the rules file;
  `docs/developer-guide.md` follows in STEP-08.
- **Verification:** `just check` exits 0; the three
  `check-release-tag.sh` cases behave as stated; the stubbed `--open-pr`
  run produces the expected calls.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** `--open-pr` cannot be tested without a real
  remote; record it and cover only the argument construction.

### PLAN-00004-STEP-06 — The remaining review findings

- **Objective:** close `MED-07`, `LOW-03`, `LOW-04`, `LOW-07`, `INFO-01`.
- **Requirements:** `PLAN-00004-REQ-08`, `PLAN-00004-REQ-09`,
  `PLAN-00004-REQ-10`, `PLAN-00004-REQ-11`, `PLAN-00004-REQ-12`
- **Depends on:** `PLAN-00004-STEP-05`
- **Affected components:** `.agents/skills/allocating-report-numbers/`,
  `src/config.rs`, a new default template, `src/llm.rs`, `Cargo.toml`,
  `tests/mock_ollama.rs`, `justfile`
- **Preconditions:** STEP-05 committed.
- **Test or evidence first:** for each finding, the check that fails today:
  the script path missing; a fresh-install config that sets
  `preferred_editor`; `is_known_editor("code --wait")` false;
  `check_service` rejecting `gemma4`; the temporary folders left behind;
  `cargo package --list` showing `docs/`.
- **Implementation tasks:**
  1. Track the skill again, byte-compared with `../opencode-template`, and
     add the script-path check to `just check`.
  2. Ship a separate neutral default template; match an editor's first
     word; make a missing editor a warning in `--check`.
  3. Accept `<name>:latest` for an untagged model.
  4. Return a removing guard from `unique_temp_dir`.
  5. Add the `include` list to `Cargo.toml`.
- **Documentation/configuration/operations:** STEP-08 documents.
- **Verification:** `just check` exits 0; `cargo package --list` shows no
  `docs/`, `.github/`, `scripts/`, `.agents/` or `tests/` entry; no
  `cli-bot-*` folder remains in the system temporary directory after a run;
  the tracked script is byte-identical to the template copy.
- **Completion criteria:** all five findings verified closed.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** the `include` list excludes something
  `include_str!` needs; the build fails loudly, so fix and continue.

### PLAN-00004-STEP-07 — Documents and the benchmark reports

- **Objective:** documents match, and the reports live together.
- **Requirements:** `PLAN-00004-REQ-14`
- **Depends on:** `PLAN-00004-STEP-06`
- **Affected components:** `docs/benchmarks/` (new), `docs/usage.md`,
  `docs/session-memory.md`, `README.md`, and every file linking a report
- **Preconditions:** STEP-06 committed.
- **Test or evidence first:**
  `git grep -n 'models-benchmark-report\|0\.3\.1'` recorded; every hit is a
  line to fix. `just links` before the move.
- **Implementation tasks:**
  1. `git mv` the four reports into `docs/benchmarks/` with version names.
  2. Update every link, including the tag-pinned one in
     `docs/release/v0.4.0.md`.
  3. Correct the `0.3.1` attributions.
- **Documentation/configuration/operations:** documentation only.
- **Verification:** `just links` exits 0; the grep shows only intended
  hits; `docs/` has no `models-benchmark-report*.md` left.
- **Completion criteria:** verification passes.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a link in a published release's notes cannot
  be kept working; record it and leave that file alone.

### PLAN-00004-STEP-08 — Version, release records, final gate

- **Objective:** ship this plan through its own new cycle.
- **Requirements:** `PLAN-00004-REQ-15`
- **Depends on:** `PLAN-00004-STEP-07`
- **Affected components:** `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`,
  `docs/release/v0.5.0.md` (new), `docs/developer-guide.md`,
  `docs/publishing.md`, `docs/crates-release.md`, `docs/configuration.md`,
  `docs/testing.md`, `docs/backlog.md`, `README.md`
- **Preconditions:** STEP-07 committed.
- **Test or evidence first:** `scripts/check-release-tag.sh v0.5.0` before
  the records are written, expecting it to fail on both.
- **Implementation tasks:**
  1. Document the new release cycle, the tokens-per-second output, the new
     `--open-pr` flag, and the neutral default template.
  2. Remove the eight closed findings from `docs/backlog.md`, leaving
     Review 00001 with none open, and record the two settings of D-05 as
     items for the user.
  3. Bump to 0.5.0; rename the changelog heading to `## v0.5.0 - <date>`;
     write `docs/release/v0.5.0.md` **final**, with absolute links.
  4. Commit, then run `just ci` on the clean tree; compare `--help` and
     `cargo package --list` with the STEP-01 references.
  5. Fill in the completion summary and hand over.
- **Documentation/configuration/operations:** this step is documentation and
  the release records.
- **Verification:** `just ci` exits 0; `scripts/check-release-tag.sh v0.5.0`
  **passes**; `just links` passes; the `--help` diff shows only this plan's
  additions.
- **Completion criteria:** every acceptance criterion is checked.
- **Rollback or recovery:** `git revert` the step commit.
- **Builder stop conditions:** a documented behaviour cannot be reconciled
  with the implementation.

## 12. Cross-cutting concerns

| Area | Applicability | Planned action or reason not applicable | Step or requirement |
|---|---|---|---|
| Compatibility and APIs | Applicable | Old configs and sessions keep working; timing fields are optional; the shipped template changes only what a new install receives | REQ-01, REQ-09 |
| Data and migration | Applicable | Benchmark reports move; every link is updated, including a tag-pinned one in published notes | REQ-14 |
| Security and privacy | Applicable | The release workflow's permissions are scoped, its actions pinned, and the token job loses its cache. No cross-repository token is introduced | REQ-05, D-04 |
| Performance and scale | Not applicable | No hot path changes | — |
| Reliability and failure handling | Applicable | The release gate fails closed: anything short of final records releases nothing. `workflow_dispatch` allows a retry without a new commit | REQ-04 |
| Observability and operations | Applicable | The benchmark output is the observability this plan adds; the gate writes a notice saying why it did nothing | REQ-02, REQ-04 |
| Dependencies and supply chain | Applicable | No new runtime dependency; `include` narrows what is published; `cargo deny` still runs | REQ-12 |
| Accessibility and UX | Applicable | Benchmark figures are omitted rather than shown as zero when the server does not report them | REQ-02 |
| Documentation and release | Applicable | Eleven documents, the changelog entry already committed as `ffca51b`, and final v0.5.0 records | REQ-14, REQ-15 |
| Deployment and rollback | Applicable | One commit per step. Until STEP-04 merges, releases work the old way; after it, `git revert` restores the tag trigger | Section 10 |

## 13. Verification strategy

| Level | Evidence or command | When | Required result |
|---|---|---|---|
| Format, lint, scripts, links | `just check` | Every step | Exit 0 |
| Unit and integration tests | `cargo test --workspace --all-targets --all-features` | Every step from STEP-02 | All pass |
| Coverage | `just coverage` | STEP-02 onwards | ≥ 91.82% total |
| Workflow lint | `just lint-workflows` | STEP-04, STEP-08 | Exit 0 |
| Release gate | The extracted gate script, three cases | STEP-04 | Releases only on final records |
| Release records | `scripts/check-release-tag.sh v0.5.0` | STEP-08 | Passes |
| Crate contents | `cargo package --list` | STEP-06, STEP-08 | No repository records |
| Temporary folders | `ls $TMPDIR/cli-bot-*` after `just check` | STEP-06 | Nothing left |
| Behaviour | `diff` of `--help` against the STEP-01 reference | STEP-08 | Only this plan's additions |
| Full pipeline | `just ci` | STEP-01, STEP-08 | Exit 0 |
| Live release | The Release run on this plan's own merge | After hand-off | Tags, builds, publishes, opens the release |

## 14. Acceptance criteria

- [ ] `PLAN-00004-AC-01` A response carrying the six timing fields yields
  them and the derived figures; one without them yields `None` and still
  parses; a zero `eval_duration` yields `None`, not infinity.
- [ ] `PLAN-00004-AC-02` `--benchmark` against a mock server returning
  timings prints `tokens_per_second`, `generated_tokens`, `prompt_tokens`,
  and `model_load_ms`, and omits each when the server does not supply it.
- [ ] `PLAN-00004-AC-03` The models benchmark report has an average
  tokens-per-second column in its model summary and its ranking, and each
  detailed result records tokens per second and load time.
- [ ] `PLAN-00004-AC-04` The ranking sorts by success rate, then tokens per
  second, then average total milliseconds, shown by a unit test over models
  differing only in the second key; the report states that it does not
  measure whether a command was correct.
- [ ] `PLAN-00004-AC-05` `release.yml` triggers on `push` to `main` and on
  `workflow_dispatch`, and no longer on a tag.
- [ ] `PLAN-00004-AC-06` The extracted gate releases nothing when the tag
  exists or the records are not final, and reports why; it proceeds only
  when both are satisfied.
- [ ] `PLAN-00004-AC-07` The gate job creates and pushes `vX.Y.Z` itself,
  and every later job is conditional on its output.
- [ ] `PLAN-00004-AC-08` `release.yml` has `contents: read` at the top with
  `contents: write` only where needed; every third-party action is pinned to
  a commit hash; the `crates` job restores no cache; and no comment claims
  the environment restricts tags.
- [ ] `PLAN-00004-AC-09` `check-release-tag.sh` passes for
  `## v0.5.0 - 2026-09-21` and for a timestamp heading, and fails when the
  heading is missing, the notes carry a draft line, the notes hold a
  relative link, or `README.md` has pre-release wording.
- [ ] `PLAN-00004-AC-10` `AGENTS.md` § Release workflow describes one pull
  request, no manual tag, and no separate release commit.
- [ ] `PLAN-00004-AC-11` `update-homebrew-formula.sh --open-pr` with a
  stubbed `gh` creates a branch from the tap's default branch, commits only
  the formula, and calls `gh pr create`; it refuses on the default branch;
  without the flag it behaves as before.
- [ ] `PLAN-00004-AC-12` `git ls-files .agents` lists both skill files,
  each byte-identical to `../opencode-template`, and `just check` fails when
  a script path named in `AGENTS.md` does not exist.
- [ ] `PLAN-00004-AC-13` The template written on first run sets no
  `preferred_editor` and lists only the default model under
  `models_benchmark`; the repository's own `cli-bot.toml` is unchanged in
  that respect.
- [ ] `PLAN-00004-AC-14` `is_known_editor("code --wait")` is true when
  `code` is on `PATH`; `--check` reports a missing editor as a warning and
  exits 0 when nothing else failed.
- [ ] `PLAN-00004-AC-15` `--check` accepts `model = "gemma4"` when the
  server lists `gemma4:latest`.
- [ ] `PLAN-00004-AC-16` After a full `just check`, no `cli-bot-*` folder
  created by the run remains in the system temporary directory.
- [ ] `PLAN-00004-AC-17` `cargo package --list` shows no file under
  `docs/`, `.github/`, `scripts/`, `.agents/`, or `tests/`, and the build
  still succeeds, proving `include_str!` still finds what it needs.
- [ ] `PLAN-00004-AC-18` The benchmark host section labels dedicated VRAM as
  such and reports shared memory when `mem_info_gtt_total` exists.
- [ ] `PLAN-00004-AC-19` `docs/` holds no `models-benchmark-report*.md`,
  `docs/benchmarks/` holds all four under version names, `just links`
  passes, and no document credits session memory to `0.3.1`.
- [ ] `PLAN-00004-AC-20` `just ci` exits 0 with line coverage at or above
  91.82%; `scripts/check-release-tag.sh v0.5.0` passes; `Cargo.toml` and
  `Cargo.lock` say `0.5.0`; `docs/backlog.md` lists no open Review 00001
  finding; the `--help` diff shows only this plan's additions.

## 15. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation or test | Owner/step |
|---|---|---|---|---|
| The new release trigger fires on a merge that should not release | Low | High | The gate needs final release records *and* an absent tag; ordinary merges fail it and the run does nothing. Tested with a stubbed `git ls-remote` | STEP-04 |
| The new trigger fails to fire, and a release silently does not happen | Medium | Medium | The gate writes a GitHub notice saying why; `workflow_dispatch` retries without a commit | STEP-04 |
| This plan's own merge is the first live run | High | Medium | Section 16 says what to watch and how to fall back to a manual tag | Section 16 |
| A direct push to `main` releases with no pull request | Low | High | Branch protection, which the agent cannot set; handed over in section 16 | D-05 |
| Tokens per second becomes the reason to change the default model | Medium | Medium | Out of scope in section 4, and the report states that correctness is not measured | D-07, REQ-03 |
| The `include` list omits something the build needs | Medium | Low | `cargo package` compiles the packaged crate, so an omission fails the gate | REQ-12 |
| The neutral template diverges from `cli-bot.toml` over time | Medium | Low | A test parses both and compares their key sets | STEP-06 |
| Moving the reports breaks a link in published release notes | Medium | Medium | `just links` checks tag-pinned GitHub paths; the v0.4.0 notes are updated with the rest | REQ-14 |

## 16. Builder hand-off

- **Start condition:** User approval and a clean repository.
- **First step:** `PLAN-00004-STEP-01`.
- **Required sequence:** STEP-01 to STEP-08 in order.
- **Parallel-safe work:** None.
- **Do not change:** approved scope, requirements, steps, acceptance
  criteria, or content outside Builder's permitted work-log area. Do not
  weaken the v0.4.0 safety model or the v0.4.1 session behaviour, change the
  default model, or edit the review.
- **Escalate when:** a stop condition is met; a change outside section 4
  seems necessary; the release gate cannot be made to fail closed.
- **Completion hand-off:** report changed files, tests run, the coverage
  figure, documents updated, and backlog changes. Suggest the pull-request
  title and description. The user pushes.

Two GitHub settings the agent cannot make, both from D-05:

1. **Protect `main`.** Require a pull request and passing checks. Under the
   new cycle a direct push carrying final records would publish.
2. **Decide the `release` environment's reviewer** (D-02), and set its
   deployment branch policy or accept that it has none.

**Watching this plan's own release**, which is the first live run of
REQ-04: after the merge, the Release workflow should create `v0.5.0`, build
two binaries, pause for approval, publish, and create the release. If the
gate wrongly does nothing, the run still succeeds and says why in a notice;
tag `v0.5.0` by hand and dispatch the workflow to recover.

<!-- BUILDER_WORK_LOG_START -->
## 17. Builder Work Log

> [!warning] Builder-maintained section
> Delivery Planner creates this section. After approval, Builder may update only
> this delimited section and the Builder-maintained front-matter fields. Builder
> must preserve prior entries and use UTC timestamps.

### Step status

| Step | Status | Started (UTC) | Completed (UTC) | Evidence | Builder notes |
|---|---|---|---|---|---|
| PLAN-00004-STEP-01 | not-started | — | — | — | — |
| PLAN-00004-STEP-02 | not-started | — | — | — | — |
| PLAN-00004-STEP-03 | not-started | — | — | — | — |
| PLAN-00004-STEP-04 | not-started | — | — | — | — |
| PLAN-00004-STEP-05 | not-started | — | — | — | — |
| PLAN-00004-STEP-06 | not-started | — | — | — | — |
| PLAN-00004-STEP-07 | not-started | — | — | — | — |
| PLAN-00004-STEP-08 | not-started | — | — | — | — |

Allowed status values: `not-started`, `in-progress`, `blocked`, `completed`,
`skipped`. A skipped step requires explicit user approval recorded in Evidence.

### Execution log

| Timestamp (UTC) | Step | Event | Evidence or reference | Next action |
|---|---|---|---|---|

### Deviations and blockers

| Timestamp (UTC) | Step | Deviation or blocker | Impact | Decision required from |
|---|---|---|---|---|

### Verification results

| Timestamp (UTC) | Step | Command or check | Result | Evidence |
|---|---|---|---|---|

### Completion summary

- **Implementation status:** `not-started`
- **Completed requirements:** None
- **Incomplete requirements:** All
- **Outstanding blockers:** None
- **Review request:** Not ready
<!-- BUILDER_WORK_LOG_END -->

## 18. Planning change log

| Timestamp (UTC) | Plan status | Change | Reason | Requested/approved by |
|---|---|---|---|---|
| 2026-09-21T20:31:02Z | draft | Plan created | User asked to improve the release cycle with automation, to add tokens per second to both benchmarks, and to close the remaining Review 00001 findings | User |
| 2026-09-21T23:00:34Z | approved | Plan approved; D-02 confirmed by the user, so the `release` environment keeps its required reviewer. `build_ready` set | User replied "Keep crates.io approval gate. Approved." | User |

## 19. External references

None. Review 00001, the repository, the GitHub API, and a live Ollama
0.32.13 instance were read directly.

## 20. Confidence

**Medium**, where the previous two plans were High. The code changes are
small and well understood, and the Ollama fields were confirmed against a
live server. The release cycle is the reason for the lower rating: its gate
can be tested in pieces but not end to end until this plan's own merge runs
it for real, and two of the safeguards around it, branch protection and the
environment's reviewer, are GitHub settings outside the plan's reach.
