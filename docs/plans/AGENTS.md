# Plans Agent Guide

`docs/plans/` holds numbered, traceable, Builder-ready delivery plans. This
guide is the contract for **any** agent that writes to this directory — not only
the `DeliveryPlanner` agent. Read it fully before creating, editing, or
renumbering a plan.

The authoritative, detailed specification lives in
`.opencode/agents/DeliveryPlanner.md`. This file is the distilled directory
contract: the rules every writer must honour, and the format every plan must
follow. Where the two differ, the DeliveryPlanner agent definition is normative.

## What lives here

One kind of record: **delivery plans**, each a numbered plan with a fixed
planning body and a delimited Builder Work Log. A plan is written once; its
planning content is frozen at approval, and only the Builder-maintained fields
and work log change afterwards.

## Who may write here

- `DeliveryPlanner` is the primary author and the only agent whose definition
  grants `docs/plans/` write access by default.
- Other agents may write here only when their permission rules allow it. When
  they do, they follow this guide exactly as `DeliveryPlanner` does: same
  numbering, same front matter, same body, same clean-state and immutability
  rules.
- An agent that cannot write here must not attempt to; it should hand the
  planning request to `DeliveryPlanner` instead.

## The clean-repository gate

A plan may be written or amended **only when the Git worktree is completely
clean**. This is the single most important rule in this directory.

Run this gate before substantive planning for any request that could write or
amend a plan:

1. Confirm a Git worktree with `git rev-parse --is-inside-work-tree`.
2. Resolve the repository root with `git rev-parse --show-toplevel`.
3. Confirm the current working directory belongs to that worktree.
4. Run exactly:

   ```bash
   git status --porcelain=v1 --untracked-files=all
   ```

5. If the command fails, do not publish a plan.
6. If it returns any line, stop before analysing the planning target, publish
   nothing, make no repository change, and report those paths.
7. If it returns no output, record the baseline branch and full `HEAD` commit.

Run the same status command again immediately before every plan create or edit.
If the second check is not empty, stop without writing.

A dirty state includes modified, added, deleted, renamed, copied, conflicted,
ignored-in-index, and untracked paths. Do not exempt existing plan files. A pure
read-only `unplanned-query` may return an inventory from a dirty repository, but
it must prominently state that no plan can be published until the repository is
clean.

## File naming and numbering

New planning targets use:

```text
docs/plans/00001-Plan_Description.md
```

Allocate the path as follows:

1. Complete the initial clean-state gate before creating a directory or file.
2. Derive `Plan_Description` from the requested outcome. Keep it concise, no more
   than six words, and use `Title_Case_With_Underscores`. Convert the description
   to ASCII words separated by underscores. Remove path separators, `..`, shell
   metacharacters, control characters, and repeated underscores. Limit the
   complete filename to 100 characters.
3. Allocate the number and create the empty plan file with the
   `allocating-report-numbers` skill:

   ```bash
   .agents/skills/allocating-report-numbers/allocate-report.sh docs/plans <Plan_Description>.md
   ```

   The script atomically allocates the next unused five-digit number (highest
   existing plus one, starting at `00001`, never filling a gap or reusing a
   number), creates the empty file, and prints the number and full path. If it
   fails because the lock is held, do not bypass it and do not publish; report
   that another allocation may be active or that a stale lock requires human
   inspection.
4. Immediately before writing, re-run the strict clean-state command. If it is
   not empty, stop without writing.
5. Write exactly one new Markdown plan into the file the script created. Never
   overwrite, rename, delete, or edit another plan.

Draft amendments and approval updates retain the same path and do not allocate a
new number. They still require both clean-state checks.

## Front matter

Use valid YAML. Replace example values with verified current values; use `null`
for fields that genuinely do not apply, never guess.

```yaml
---
title: "Delivery Plan 00001: Plan Description"
aliases:
  - "Plan 00001"
tags:
  - delivery-plan
  - implementation
  - opencode
type: delivery-plan
plan_id: "PLAN-00001"
plan_status: draft                 # draft | approved | cancelled
plan_kind: initial                 # initial | superseding
created_at: "2026-09-04T12:00:00Z"
approved_at: null
planner_agent: DeliveryPlanner
planner_model: "provider/model-id"
triggered_by: user                 # user | agent:<agent-name>
request_kind: direct               # idea | review | idea-and-review | direct | unplanned-query
repository: "owner/repository"
baseline_branch: "main"
baseline_commit: "full-commit-hash"
source_ideas: []
source_reviews: []
previous_plan: null
requirements_count: 0
steps_count: 0
acceptance_criteria_count: 0
blocking_decisions: 0
build_ready: false
web_research_used: false
confidence: high                  # high | medium | low

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
```

Front-matter rules:

- Obtain timestamps with `date -u +%Y-%m-%dT%H:%M:%SZ`; never estimate them.
- `planner_agent` is the name of the writing agent (`DeliveryPlanner` for
  DeliveryPlanner; the agent's own name otherwise). `planner_model` is the
  exact active
  provider/model identifier when available; use `null` and explain the limitation
  in Confidence otherwise.
- Record full repository-relative paths in `source_ideas` and `source_reviews`.
- `previous_plan` is an exact path or Obsidian wikilink only for a superseding
  plan.
- Counts must exactly match the body.
- `blocking_decisions` must be zero before approval.
- `build_ready` is true only after explicit user approval and successful quality
  control.
- `confidence` reflects repository coverage and evidence quality, not model
  self-confidence.
- Builder may update only the explicitly labelled Builder-maintained keys.

## Body structure

Use this structure. Keep every heading, writing `Not applicable` with a concise
reason where a section genuinely does not apply.

```markdown
# Delivery Plan 00001: Plan Description

> [!abstract] Plan status: `draft`
> One-sentence outcome, readiness statement, and principal blocker if any.

## 1. Objective and outcome

Describe the observable outcome and why it is needed.

## 2. Source traceability

| Requirement | Source | Source location | Interpretation |
|---|---|---|---|
| PLAN-00001-REQ-01 | User / Idea / Review / Repository | Exact path, section, finding, or symbol | Atomic requirement |

## 3. Repository baseline

| Field | Value |
|---|---|
| Repository | ... |
| Branch | ... |
| HEAD | ... |
| Working tree at publication | Clean |
| Applicable instructions | ... |

## 4. Scope

### In scope

- ...

### Out of scope

- ...

## 5. Constraints and preserved decisions

- ...

## 6. Assumptions

None. Unresolved matters are recorded as decisions and block approval when
material.

## 7. Decisions and blockers

| ID | Decision or blocker | Resolution | Owner | Status |
|---|---|---|---|---|

Write `None` when all material decisions are resolved.

## 8. Affected architecture and components

Explain affected boundaries and list confirmed paths, symbols, interfaces,
schemas, services, data stores, configuration, and documentation. Include a
compact Mermaid diagram only when relationships or ordering would be materially
clearer than prose or a table.

## 9. Requirement catalogue

### PLAN-00001-REQ-01 — Requirement title

- **Requirement:** ...
- **Rationale:** ...
- **Source:** ...
- **Acceptance evidence:** ...

## 10. Delivery strategy

Explain sequencing, dependency boundaries, test approach, checkpoints, and why
the proposed increments are safe.

## 11. Detailed implementation steps

### PLAN-00001-STEP-01 — Step title

- **Objective:** ...
- **Requirements:** `PLAN-00001-REQ-01`
- **Depends on:** None / step IDs
- **Affected components:** exact paths or symbols
- **Preconditions:** ...
- **Test or evidence first:** ...
- **Implementation tasks:**
  1. ...
  2. ...
- **Documentation/configuration/operations:** ...
- **Verification:** exact command or observable check
- **Completion criteria:** ...
- **Rollback or recovery:** ...
- **Builder stop conditions:** ...

Repeat for every ordered step.

## 12. Cross-cutting concerns

| Area | Applicability | Planned action or reason not applicable | Step or requirement |
|---|---|---|---|
| Compatibility and APIs | Applicable / Not applicable | ... | ... |
| Data and migration | ... | ... | ... |
| Security and privacy | ... | ... | ... |
| Performance and scale | ... | ... | ... |
| Reliability and failure handling | ... | ... | ... |
| Observability and operations | ... | ... | ... |
| Dependencies and supply chain | ... | ... | ... |
| Accessibility and UX | ... | ... | ... |
| Documentation and release | ... | ... | ... |
| Deployment and rollback | ... | ... | ... |

## 13. Verification strategy

| Level | Evidence or command | When | Required result |
|---|---|---|---|

## 14. Acceptance criteria

- [ ] `PLAN-00001-AC-01` ...
- [ ] `PLAN-00001-AC-02` ...

## 15. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation or test | Owner/step |
|---|---|---|---|---|

## 16. Builder hand-off

- **Start condition:** User approval and a clean repository.
- **First step:** ...
- **Required sequence:** ...
- **Parallel-safe work:** ...
- **Do not change:** approved scope, requirements, steps, acceptance criteria,
  or content outside Builder's permitted work-log area.
- **Escalate when:** ...
- **Completion hand-off:** ...

<!-- BUILDER_WORK_LOG_START -->
## 17. Builder Work Log

> [!warning] Builder-maintained section
> Delivery Planner creates this section. After approval, Builder may update only
> this delimited section and the Builder-maintained front-matter fields. Builder
> must preserve prior entries and use UTC timestamps.

### Step status

| Step | Status | Started (UTC) | Completed (UTC) | Evidence | Builder notes |
|---|---|---|---|---|---|
| PLAN-00001-STEP-01 | not-started | — | — | — | — |

Allowed status values: `not-started`, `in-progress`, `blocked`, `completed`,
`skipped`. A skipped step requires explicit user approval recorded in Evidence.

### Execution log

| Timestamp (UTC) | Step | Event | Evidence or reference | Next action |
|---|---|---|---|---|

### Deviations and blockers

| Timestamp (UTC) | Step | Deviation or blocker | Impact | Decision required from |
|---|---|---|---|---|

Write `None` until an entry is required.

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

## 19. External references

List only sources actually used, with title, publisher or author,
publication/update date when known, access date, and URL. Write `None` when no
external research was used.

## 20. Confidence

**High / Medium / Low.** Explain repository coverage, source quality, and the
principal residual uncertainty in two or three sentences.
```

## Requirements, steps, and acceptance criteria

- **Requirements** use IDs `PLAN-00001-REQ-01`, `PLAN-00001-REQ-02`, and so on.
  Every requirement must cite at least one of: an explicit current user
  instruction, an exact Idea report path and section, an exact Review report path
  and finding ID, an applicable repository instruction path and section, a
  verified repository path and symbol, or an authoritative external source (only
  when necessary).
- **Steps** use IDs `PLAN-00001-STEP-01`, and so on. Each step defines its
  objective, requirement IDs, dependencies, affected components, preconditions,
  test-or-evidence-first, implementation tasks, documentation/configuration/
  operations work, verification, completion criteria, rollback/recovery, and
  Builder stop conditions.
- **Acceptance criteria** use IDs `PLAN-00001-AC-01`, and so on. They must be
  observable by Builder and a later Review or Validate agent. Avoid vague terms
  such as `works`, `robust`, `clean`, `appropriate`, `properly`, or `as needed`
  unless a measurable definition immediately follows.
- Do not convert an unresolved question into a requirement, and do not add
  attractive scope unsupported by the request or sources.

## Plan lifecycle and ownership

### Draft

Create the first plan with `plan_status: draft`. A draft may contain resolved
decisions and clearly identified non-blocking uncertainties, but it must not
claim to be Builder-ready while a user decision remains.

The user may request corrections to a draft. Amend that same numbered file only
when: the user clearly identifies the draft; the repository is clean before the
amendment; the current file still has `plan_status: draft`; and no Builder
execution entry has started. Record every material draft amendment in
`Planning change log`; never erase the history of a changed decision.

### Approval

Only the user can approve a plan. On explicit approval:

1. Recheck the repository clean-state gate.
2. Re-read the current draft and confirm that no blocking decision remains.
3. Set `plan_status: approved`, `build_ready: true`, and `approved_at` to the
   current UTC timestamp.
4. Add an approval entry to `Planning change log`.
5. Validate that the approval update changed only the exact plan file.
6. Stage only that file with `git add -- <exact-plan-path>`.
7. Verify with `git diff --cached --name-only` that the staged set contains
   exactly that one plan path. If it does not, do not commit and report the
   unexpected staged paths.
8. Commit that file with the exact message:

   ```text
   docs(plan): approve PLAN-<ID> - <title>
   ```

   `<ID>` is the five-digit numeric part of `plan_id`; `<title>` is the sanitised
   filename description with underscores replaced by spaces.
9. Do not bypass Git hooks. If staging, a hook, or the commit fails, stop and
   report the failure and current Git status without attempting recovery.
10. Verify the commit exists, capture its full hash, and check the worktree is
    clean. If a hook or another process left changes, report them rather than
    modifying or committing them.
11. Do not alter the approved scope, requirements, steps, or acceptance criteria
    afterwards.

Because the strict clean-state gate includes plan files, the user must manually
commit or otherwise clean every newly written draft and draft revision before
asking for another revision or approval. DeliveryPlanner never commits a draft or
revision; it commits only the final approval update after explicit user approval.

### Builder execution

Builder owns only the front-matter fields listed under `Builder-maintained front
matter` and the content between `<!-- BUILDER_WORK_LOG_START -->` and
`<!-- BUILDER_WORK_LOG_END -->`. Builder must not alter the approved planning
content. If implementation reveals a material omission or scope change, Builder
records the blocker and returns to the user; DeliveryPlanner then creates a new
plan after the repository becomes clean and the user resolves the change.

### Superseding plan

Create a new numbered plan when an approved plan needs a material change. Set
`plan_kind: superseding`, link `previous_plan`, and state exactly what changed
and why. Do not edit the earlier approved plan merely to mark it superseded.

## Invocation types

Classify the request as exactly one of: `idea` (Idea reports are the primary
source), `review` (Review reports are the primary source), `idea-and-review`
(both apply to one outcome), `direct` (the user's instruction is sufficient), or
`unplanned-query` (which Ideas or Reviews lack a plan). Do not require an Idea or
Review report for a simple direct task, and do not invent a report merely to
satisfy a workflow shape.

## Relationship to ideas, reviews, and backlog

- A plan's `source_ideas` and `source_reviews` link it to the exact reports it
  implements; the Source traceability section maps each requirement to its
  source.
- An Idea source must be `accepted` (or the user must explicitly authorise
  planning from a non-accepted revision). A Review source is read for its
  verdict, blocking findings, scope, and recommended actions; a later re-review
  supersedes an earlier result only when it explicitly links to that review and
  resolves its findings.
- When a plan is approved, executed, or cancelled, record the outcome in
  `docs/backlog.md` where the change is a completed, active, or dropped item.

## Rules summary

- Write only new Markdown plans below `docs/plans/`; never modify anything else.
- Never write or amend a plan while the worktree is dirty; run the clean-state
  gate before and immediately before every create or edit.
- Never fill a numbering gap or reuse a number; always allocate through the
  `allocating-report-numbers` skill.
- Never overwrite an existing numbered plan with a new planning target.
- Never alter the planning sections of an approved plan; a material post-approval
  change requires a new superseding plan.
- Never stage or commit anything except the exact approved plan file during the
  approval transaction; never use `git add -A`, `git add .`, `git commit -a`,
  `--amend`, `--no-verify`, or a broader pathspec.
- Never reproduce secrets; redact the value and report only its type and safe
  location.
- Never claim implementation, verification, approval, or a clean repository
  without current evidence.
- Keep front-matter counts equal to the body and `blocking_decisions` at zero
  before approval.
