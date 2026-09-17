# Reviews Agent Guide

`docs/reviews/` holds immutable, evidence-led code-review reports. This guide is
the contract for **any** agent that writes to this directory — not only the
`CodeReview` agent. Read it fully before creating, editing, or renumbering a
report.

The authoritative, detailed specification lives in `.opencode/agents/CodeReview.md`.
This file is the distilled directory contract: the rules every writer must honour,
and the format every report must follow. Where the two differ, the CodeReview
agent definition is normative.

## What lives here

Two kinds of records share this directory, and they are treated differently:

1. **Agent-authored reviews** — written by `CodeReview` (or another agent granted
   write access) against a pull request, commit, branch, or working tree. These
   follow the canonical front-matter schema and body structure below.
2. **External review records** — verbatim imports of a review written by an
   external reviewer or model (e.g. `00003`, `00004`). These keep the external
   author's own front matter and formatting, exactly as authored. They are
   evidence, not agent output: never reformat, renumber, or "normalise" them.

Both kinds are **immutable**. A report is written once and never edited.

## Who may write here

- `CodeReview` is the primary author and the only agent whose definition grants
  `docs/reviews/` write access by default.
- Other agents may write here only when their permission rules allow it. When
  they do, they follow this guide exactly as `CodeReview` does: same numbering,
  same front matter, same body, same immutability rules.
- An agent that cannot write here must not attempt to; it should hand the review
  request to `CodeReview` instead.

## File naming and numbering

Every report uses this pattern:

```text
docs/reviews/00001-Review_Description.md
```

Allocate the number safely, in this order:

1. Derive `Review_Description` from the review target: `PR_<number>_<short-title>`,
   `<branch>_<short-purpose>`, `<short-commit>_<commit-subject>`, or
   `Current_Code_State`. Keep it concise (≤ 6 words), `Title_Case_With_Underscores`,
   ASCII only, no path separators, `..`, shell metacharacters, or repeated
   underscores; maximum 80 characters before `.md`.
2. Allocate the number and create the empty report file with the
   `allocating-report-numbers` skill:

   ```bash
   .agents/skills/allocating-report-numbers/allocate-report.sh docs/reviews <Review_Description>.md
   ```

   The script atomically allocates the next unused five-digit number (highest
   existing plus one, starting at `00001`, never filling a gap or reusing a
   number), creates the empty file, and prints the number and full path. If it
   fails because the lock is held, do not bypass it and do not publish; report
   that another allocation may be active or that a stale lock requires human
   inspection.
3. Write exactly one new report into the file the script created. Never
   overwrite, rename, delete, or edit another.

## Front matter

Agent-authored reviews use this YAML schema. Replace example values with evidence
from the review; use `null` for non-applicable or genuinely unavailable values.

```yaml
---
title: "Code Review 00001: Review Description"
aliases:
  - "Review 00001"
tags:
  - code-review
  - software-quality
  - opencode
type: code-review
status: open                 # open | addressed | superseded
review_id: "00001"
reviewed_at: "2026-08-22T07:43:00Z"
reviewer_agent: CodeReview      # the writing agent's name (CodeReview, DeliveryBuilder, general, ...)
review_model: "anthropic/claude-sonnet-4-5"
triggered_by: "user"         # user | agent:<agent-name>
review_kind: initial         # initial | re-review
previous_review: null
repository: "owner/repository"
branch: "feature/example"
review_mode: pull-request    # pull-request | branch | commit | range | working-tree | repository
pr_reference: "owner/repository#123"   # null when not a PR
commit: null                 # reviewed HEAD for non-PR reviews
base_ref: "main"
base_commit: "full-commit-hash"
head_ref: "feature/example"
head_commit: "full-commit-hash"
scope: "merge-base(base, head)..head"
related_plan: null
files_changed: 0
files_reviewed: 0
diff_additions: 0
diff_deletions: 0
blocking_issues: 0
issues:
  critical: 0
  major: 0
  medium: 0
  low: 0
  info: 0
  total: 0
categories:                  # counts by theme; drop keys that are zero
  security: 0
  correctness: 0
  performance: 0
  tests: 0
  style: 0
verdict: approve             # approve | approve-with-comments | request-changes | incomplete
review_complete: true
web_research_used: false
confidence: high             # high | medium | low
sources:                     # any URL relied on
  - https://cwe.mitre.org/data/definitions/613.html
---
```

Front-matter rules:

- `reviewed_at` is the UTC completion timestamp in ISO 8601, obtained with
  `date -u +%Y-%m-%dT%H:%M:%SZ`. Never guess the time.
- `reviewer_agent` is the name of the agent that wrote the report
  (`CodeReview` for
  CodeReview; the agent's own name otherwise). `review_model` is the exact
  active
  provider/model identifier when available.
- `triggered_by` is `user` or `agent:<agent-name>`.
- `review_kind` is `initial` or `re-review`; `previous_review` is the prior report
  path or wikilink for a re-review, otherwise `null`.
- `review_mode` is one of `pull-request`, `branch`, `commit`, `range`,
  `working-tree`, or `repository`.
- For a PR, populate `pr_reference`; `commit` may be `null` while `head_commit`
  records the PR head. For any non-PR review, set `pr_reference: null` and
  populate `commit` with the reviewed `HEAD` or single commit hash.
- `files_reviewed` counts files whose diff and relevant context were actually
  inspected; do not copy `files_changed` blindly.
- Issue counts and `issues.total` must exactly match the findings in the body.
  `blocking_issues` is the count of Critical plus Major findings.
- `review_complete` is `false` when inaccessible, truncated, binary, generated, or
  missing evidence materially limits coverage.
- `confidence` is `high`, `medium`, or `low` based on evidence quality and
  coverage, not model self-confidence.
- `status` is `open` when written; a review becomes `addressed` once its Critical
  and Major findings are fixed, and `superseded` when fully replaced by a later one.

## Body structure

Use this structure. Omit an empty severity section only when the issue-summary
table clearly records zero, but preserve all other headings.

```markdown
# Code Review 00001: Review Description

> [!abstract] Verdict: `approve`
> One-sentence conclusion describing merge or handoff readiness.

## Review target

| Field | Value |
|---|---|
| Review mode | Pull request / branch / commit / range / working tree / repository |
| PR or commit | ... |
| Base | ... |
| Head | ... |
| Branch | ... |
| Triggered by | ... |
| Related plan | ... |

## Executive summary

Concise description of the change, the most important risks, and the verdict.

## Issue summary

| Severity | Count | Merge impact |
|---|---:|---|
| Critical | 0 | Blocks merge/release |
| Major | 0 | Blocks merge/release |
| Medium | 0 | Changes requested |
| Low | 0 | Non-blocking |
| **Total** | **0** | |

## Findings

### Critical

#### REV-00001-CRIT-01 — Finding title

> [!danger] Blocking
> - **Confidence:** High / Medium / Low
> - **Category:** Correctness / Security / Data integrity / Reliability / Performance / Testing / Maintainability / Style / Documentation
> - **Location:** `path/to/file.ext:line-line` — `symbol`
> - **Evidence:** Minimal, specific evidence from the change and surrounding code.
> - **Failure or attack scenario:** Concrete sequence that triggers the problem.
> - **Impact:** User, security, data, operational, or compatibility consequence.
> - **Recommendation:** Smallest safe correction, without editing the code.
> - **Suggested verification:** Regression test, analysis, or operational check that proves the fix.
> - **References:** Official external source when used, otherwise `Repository evidence`.

### Major

> [!warning] Blocking
> ... (same shape as Critical)

### Medium

#### REV-00001-MED-01 — Finding title

- **Location:** ...
- **What:** the defect
- **Why it matters:** concrete impact
- **Evidence:** short fenced snippet or diff hunk, ≤ 15 lines
- **Suggested fix:** described, not applied

### Low

#### REV-00001-LOW-01 — Finding title

- **Location:** ...
- **What / Why / Suggested fix**

## Open questions

Questions that require evidence but are not counted as findings. State what
would resolve each question.

## Review coverage

### Files and areas reviewed

- Changed files and relevant surrounding components actually inspected

### Checks performed

- Diff and target-resolution checks
- Static code and contract reasoning
- LSP or repository searches used
- External research used, when applicable

### Checks not performed

- Tests, builds, linters, formatters, scanners, hooks, migrations, and runtime
  execution were not run by this read-only agent
- Any additional target-specific limitation

## Positive notes

Briefly record safeguards, tests, or design choices that materially reduce risk.
Do not manufacture or pad this.

## External references

List only sources actually used, with title, publisher, publication/update date
when known, access date, and URL. Write `None` when no external research was used.

## Recommended next actions

Order required fixes and verification steps by severity and dependency.

## Handoff

State what the user or calling agent should do next. For a re-review, identify
which prior findings were resolved, remain open, or were superseded.

## Confidence

**High / Medium / Low.** Explain coverage quality and the principal uncertainty
in two or three sentences.
```

Use `[[wikilinks]]` for other review files, backtick-wrapped `path:line`
references for code, and Obsidian callouts for severity. Keep the file
self-contained: someone reading it in Obsidian months later, without the diff,
should understand each finding.

### No-finding report

When no actionable findings exist, include this callout under `## Findings`:

```markdown
> [!success] No actionable findings
> No Critical, Major, Medium, or Low issues were identified within the reviewed scope. This does not prove the change is defect-free; see review coverage and limitations.
```

### Incomplete review report

When evidence is materially incomplete, set `verdict: incomplete` and
`review_complete: false`. Do not convert unknowns into findings merely to avoid
an empty report. State exactly what was inaccessible, why it matters, what was
still reviewed, and what evidence is required for a dependable re-review.

## Findings, severity, and verdict

### Severity model

Assign exactly one level per finding. Be strict: inflated severity destroys the
signal the calling agent relies on.

- **Critical** — readily exploitable or highly probable catastrophic impact (RCE,
  broad auth bypass, major secret compromise, irreversible widespread data loss,
  severe outage). Blocks merge or release.
- **Major** — likely production correctness, security, privacy, data-integrity,
  availability, or compatibility failure with substantial impact. Normally
  blocks merge or release.
- **Medium** — credible edge-case defect, reliability problem, performance
  regression, inadequate validation, or maintainability issue likely to affect
  users or operations. Requires changes but may not justify an emergency stop.
- **Low** — localized, low-impact issue (minor test gap, documentation mismatch,
  clarity, explicit style-rule violation). Non-blocking unless project policy says
  otherwise.
- **Info** — observation, praise, or suggestion carrying no obligation. Not
  counted toward `blocking_issues` or the verdict.

Do not use `High`, `Warning`, or other levels in issue counts.

### Finding identifiers

Finding identifiers include the review number and severity:

```text
REV-00001-CRIT-01
REV-00001-MAJ-01
REV-00001-MED-01
REV-00001-LOW-01
```

Number findings independently within each severity, starting at `01`.

### Verdict rules

Choose exactly one verdict:

- `request-changes` — one or more Critical or Major findings.
- `approve-with-comments` — no Critical or Major findings, but one or more Medium findings.
- `approve` — only Low and Info findings.
- `incomplete` — missing or inaccessible evidence prevents a dependable verdict.

An `approve` verdict means no actionable issue was found in the reviewed scope. It
is not proof that the code is defect-free.

## Immutability and re-review

- Never overwrite, rename, delete, or edit a prior report. A re-review creates the
  next numbered report and links to the earlier one.
- A re-review sets `review_kind: re-review` and `previous_review` to the earlier
  report, re-tests each prior finding through static reasoning, and classifies
  every prior finding in `Handoff` as `resolved`, `still-open`,
  `partially-resolved`, `not-in-scope`, or `superseded`.
- New findings get identifiers based on the new review number; preserve old
  identifiers only when referring to the previous report.

## External review records

External reviews (e.g. `00003`, `00004`) are verbatim records of a review written
by an external reviewer or model. They are imported as authored and are **not**
rewritten into the agent schema:

- Keep the external author's own front matter and body formatting.
- Do not reformat, renumber, or "normalise" them — the same principle as NIST
  fixtures, which are never regenerated.
- Do not edit them to add findings, change verdicts, or "fix" their prose.
- If an external review must be superseded, write a new agent-authored review that
  links to it; do not modify the original.

## Relationship to plans, ideas, and backlog

- Review findings are the primary input to delivery plans (`docs/plans/`); a plan
  cites its source reviews by path or ID.
- A re-review supersedes an earlier result only when it explicitly links to that
  review and resolves its findings.
- When a review's Critical/Major findings are fixed, its `status` moves to
  `addressed`; when fully replaced, `superseded`. Record the outcome in
  `docs/backlog.md` where the change is a completed or dropped item.

## Rules summary

- Write only new Markdown reports below `docs/reviews/`; never modify anything else.
- Never overwrite or revise a prior report; re-review creates the next number.
- Never fill a numbering gap or reuse a number; always allocate through the
  `allocating-report-numbers` skill.
- Never regenerate or reformat external review records.
- Never reproduce secrets; describe type and location, redact the value.
- Never claim tests passed unless trustworthy results were supplied; state that
  tests were not executed by a read-only agent.
- Keep front-matter counts equal to body findings and sum to `issues.total`.
- Keep the verdict deterministic per the verdict rules.
