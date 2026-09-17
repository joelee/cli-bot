# Ideas Agent Guide

`docs/ideas/` holds immutable, evidence-led idea and feature-brief reports. This
guide is the contract for **any** agent that writes to this directory — not only
the `IdeaArchitect` agent. Read it fully before creating, editing, or renumbering
a report.

The authoritative, detailed specification lives in
`.opencode/agents/IdeaArchitect.md`. This file is the distilled directory
contract: the rules every writer must honour, and the format every report must
follow. Where the two differ, the IdeaArchitect agent definition is normative.

## What lives here

One kind of record: **idea reports**, each a numbered idea with one or more
immutable revisions. A report is written once and never edited; every change —
feedback, challenge, finalisation, or a decision to park or reject — is a new
revision file, not an edit to an earlier one.

## Who may write here

- `IdeaArchitect` is the primary author and the only agent whose definition grants
  `docs/ideas/` write access by default.
- Other agents may write here only when their permission rules allow it. When
  they do, they follow this guide exactly as `IdeaArchitect` does: same numbering,
  same front matter, same body, same immutability rules.
- An agent that cannot write here must not attempt to; it should hand the idea
  request to `IdeaArchitect` instead.

## File naming and numbering

Every report uses this pattern:

```text
docs/ideas/00001-Idea_Description-r01.md
```

The five-digit number is the stable **idea ID**. The two-digit `rNN` suffix is the
**revision number** for that idea.

Allocate the number safely, in this order:

1. Derive `Idea_Description` from the concise idea title. Use
   `Title_Case_With_Underscores`, no more than six words. Convert the description
   to ASCII words separated by underscores. Remove path separators, `..`, shell
   metacharacters, control characters, and repeated underscores. Limit it to 80
   characters.
2. For a **new idea**, allocate the number and create the empty report file with
   the `allocating-report-numbers` skill:

   ```bash
   .agents/skills/allocating-report-numbers/allocate-report.sh docs/ideas <Idea_Description>-r01.md
   ```

   The script atomically allocates the next unused five-digit idea ID (highest
   existing plus one, starting at `00001`, never filling a gap or reusing an ID),
   creates the empty file, and prints the number and full path. If it fails
   because the lock is held, do not bypass it and do not publish; report that
   another run may be allocating a number or that a stale lock needs inspection.
3. For a **revision**, do not use the skill. Preserve the five-digit idea ID and
   canonical description from the latest report, select the highest valid revision
   for that idea plus one, and create the file directly. Start at `r01` only for
   a new idea.
4. Write exactly one new Markdown report into the file the script created. Never
   overwrite, edit, rename, or delete another report.

Use Obsidian wikilinks for report relationships:

```yaml
previous_revision: "[[00001-Idea_Description-r01]]"
root_revision: "[[00001-Idea_Description-r01]]"
```

## Front matter

Use this YAML schema. Replace examples with values from the current engagement;
use `null` when a value is not applicable or genuinely unavailable.

```yaml
---
title: "Idea 00001 r01: Idea Description"
aliases:
  - "Idea 00001"
tags:
  - idea
  - discovery
  - IdeaArchitect
  - opencode
type: idea-report
idea_id: "IDEA-00001"
revision: 1
revision_kind: initial       # initial | feedback | challenge | final | decision
status: draft                # draft | revised | accepted | parked | rejected
created: 2026-09-04
updated: 2026-09-04
analysed_at: "2026-09-04T12:00:00Z"
agent: IdeaArchitect
model: "provider/model-id"
triggered_by: user           # user | agent:<agent-name>
previous_revision: null
root_revision: "[[00001-Idea_Description-r01]]"
related:
  - "path/to/related-document.md"
idea_kind: feature           # idea | feature | product | process | architecture | experiment | other
maturity: discovery          # seed | discovery | experiment-ready | decision-ready
recommendation: proceed-to-experiment  # proceed-to-experiment | revise | park | reject | incomplete
confidence: medium           # high | medium | low
fact_check_status: partial   # not-started | partial | complete
web_research_used: false
actionable_risks: 0
risks:
  critical: 0
  major: 0
  medium: 0
  low: 0
  info: 0
  total: 0
open_questions:
  blocking: 0
  non_blocking: 0
sources: []
---
```

Front-matter rules:

- Obtain `analysed_at` with `date -u +%Y-%m-%dT%H:%M:%SZ`. Never guess the
  completion timestamp.
- `created` is the date of `r01`; preserve it in later revisions. `updated` is
  the date of the current revision. Both use `YYYY-MM-DD`.
- `agent` is the name of the writing agent (`IdeaArchitect` for
  IdeaArchitect; the agent's own name otherwise). `model` is the exact active
  provider/model
  identifier when available; use `unknown` and explain the limitation otherwise.
- `triggered_by` is `user` or `agent:<agent-name>`.
- `previous_revision` points to the immediately preceding report, except on `r01`,
  where it is `null`. `root_revision` always points to `r01`, including from
  `r01` itself.
- `related` contains repository-relative paths or wikilinks actually relevant to
  the analysis; do not populate it speculatively.
- Issue counts must exactly match the report body. `actionable_risks` is the sum
  of Critical, Major, Medium, and Low; Info is excluded.
- `fact_check_status` cannot be `complete` while a material factual claim is
  unverified or disputed without a documented resolution.
- Confidence reflects evidence quality and coverage, not model confidence.
- `status: accepted`, `parked`, or `rejected` requires an explicit user decision.
- Record only URLs actually relied on under `sources`.

## Body structure

Use this structure. Adapt detail to the idea and omit only genuinely irrelevant
subsections. Preserve every level-two heading so later revisions remain easy to
compare.

```markdown
# Idea 00001 r01: Idea Description

> [!abstract] Recommendation: `proceed-to-experiment`
> Concise conclusion, evidence position, principal uncertainty, and next move.

## 1. Seed Idea

### Original proposition

A faithful summary of the user's idea.

### Motivation and timing

Why it matters and why it is being considered now.

## 2. Context and Intent

| Field | Detail |
|---|---|
| Intended outcome | ... |
| Target users or beneficiaries | ... |
| Current stage | ... |
| Known constraints | ... |
| Non-negotiables | ... |
| Related project context | ... |

## 3. Problem or Opportunity

The problem, current behaviour or alternatives, urgency, and evidence that the
problem exists.

## 4. Proposed Feature or Concept

### User-visible outcome

What changes for the user or stakeholder.

### Principal use cases

- ...

### Important edge cases

- ...

## 5. Desired Outcomes and Success Measures

| Outcome | Measure | Baseline | Target | Evidence needed |
|---|---|---:|---:|---|

## 6. Scope and Non-goals

### In scope

- ...

### Out of scope

- ...

## 7. Users and Stakeholders

| Stakeholder | Need or incentive | Impact | Involvement needed |
|---|---|---|---|

## 8. Assumption Ledger

| ID | Statement | Classification | Impact if wrong | Evidence status | Confidence | Cheapest test |
|---|---|---|---|---|---|---|

## 9. Research and Fact Check

| Claim | Finding | Status | Evidence | Checked on |
|---|---|---|---|---|

### Evidence limitations

Coverage gaps, stale data, source disagreement, or methodological limitations.

## 10. Challenge Review

### Strongest version of the idea

The steelman.

### Formal findings

#### IDEA-00001-R01-MAJ-01: Finding title

> [!warning] Major
> - **Confidence:** High / Medium / Low
> - **Category:** Value / Feasibility / Architecture / Security / Privacy / Data / Operations / Cost / Adoption / Compliance / Maintainability
> - **Evidence:** Specific project or external evidence.
> - **Failure scenario:** Concrete sequence or condition.
> - **Impact:** User, delivery, security, data, operational, or strategic consequence.
> - **Mitigation or test:** Smallest credible action.
> - **References:** Project evidence or cited source.

### Failure modes and unintended consequences

- ...

### Conditions to revise, park, or reject

- ...

## 11. Options and Trade-offs

| Option | Benefits | Costs and risks | Reversibility | Evidence needed |
|---|---|---|---|---|

## 12. Recommended Concept

The current best synthesis and why it is preferred. Include a Mermaid diagram
when relationships, states, or workflow would be materially clearer.

## 13. Dependencies, Risks, and Safeguards

| Item | Type | Likelihood | Impact | Mitigation, test, or owner |
|---|---|---|---|---|

## 14. Highest-value Next Experiment

- **Hypothesis:** ...
- **Method:** ...
- **Inputs or participants:** ...
- **Success threshold:** ...
- **Failure threshold:** ...
- **Expected effort:** ...
- **Risks and safeguards:** ...
- **Evidence to capture:** ...
- **Decision enabled:** ...

## 15. Open Questions and Loose Ends

### Blocking

- [ ] ...

### Important but non-blocking

- [ ] ...

### Later considerations

- [ ] ...

## 16. Feedback Incorporated

| Feedback or prior finding | Disposition | Change in this revision | Rationale |
|---|---|---|---|

For `r01`, write `Not applicable: initial draft`.

## 17. Decision Log

| Date | Decision or change | Rationale | Owner |
|---|---|---|---|

## 18. Recommended Next Actions

1. ...
2. ...
3. ...

## 19. Revision History

| Revision | Status | Kind | Supersedes | Summary |
|---|---|---|---|---|

## References

1. Author or organisation. "Title." Publication or update date. Accessed
   YYYY-MM-DD. URL

Write `None` when no external research was used.

## Confidence

**High / Medium / Low.** Explain evidence coverage and the main uncertainty in
two or three sentences.
```

### No formal risk findings

When no Critical, Major, Medium, or Low finding exists, include:

```markdown
> [!success] No actionable risk findings
> No Critical, Major, Medium, or Low risks were established within the assessed
> scope. This is not proof that the idea is valid or implementation-ready; see
> evidence limitations and open questions.
```

### Incomplete assessment

When evidence is materially incomplete, set `recommendation: incomplete` and
reduce confidence appropriately. Do not turn unknowns into findings merely to
fill the report. State exactly what was inaccessible or undecidable, why it
matters, and what evidence is required; still publish a useful draft when a
coherent report can be produced safely.

## Findings, severity, and recommendation

### Severity model

Assign exactly one severity per finding. Be strict: inflated severity destroys
the signal the calling agent relies on.

- **Critical** — a fundamental contradiction or unacceptable safety, security,
  legal, data, operational, or viability risk that makes the idea unsafe or
  unsound unless the premise changes. Blocks recommendation to proceed.
- **Major** — a likely high-impact failure, unproven core assumption, infeasible
  dependency, or substantial value or delivery problem. Blocks implementation
  planning but may be resolved through reframing or a targeted experiment.
- **Medium** — a credible edge case, adoption barrier, reliability concern,
  integration issue, cost uncertainty, or maintainability problem that requires
  attention but does not invalidate the concept.
- **Low** — a localised, low-impact clarity gap, minor constraint, documentation
  issue, or improvement opportunity. Does not block the next discovery step.
- **Info** — an observation, strength, or optional suggestion with no
  obligation. Not counted as an actionable risk.

### Finding identifiers

Finding identifiers include the idea number and report revision:

```text
IDEA-00001-R01-CRIT-01
IDEA-00001-R01-MAJ-01
IDEA-00001-R01-MED-01
IDEA-00001-R01-LOW-01
```

Number findings independently within each severity. Preserve an earlier finding
ID only when referring to it; a finding in a new report receives a new ID and the
disposition table links it to the earlier finding.

### Recommendation rules

Choose exactly one recommendation:

- `proceed-to-experiment` — the concept is coherent enough for a bounded test.
- `revise` — material issues require the concept or brief to change first.
- `park` — the idea may have merit, but timing, priority, evidence, or dependency
  conditions do not currently justify more work.
- `reject` — evidence shows the central proposition is unsafe, infeasible,
  unnecessary, or clearly dominated by a better alternative.
- `incomplete` — missing or inaccessible evidence prevents a dependable view.

Rules:

- Any unresolved Critical finding results in `revise`, `park`, `reject`, or
  `incomplete`, never `proceed-to-experiment`.
- An unresolved Major finding normally results in `revise` unless the proposed
  experiment is explicitly designed to resolve it safely before planning.
- `proceed-to-experiment` is permission for discovery work, not implementation.
- Only the user can set the report status to `accepted`, `parked`, or `rejected`.
- A user preference does not erase contradictory evidence; record the user's
  decision and the residual risk.

## Report lifecycle and immutability

- **Initial draft** — publish `r01` after the first substantive analysis, with
  `status: draft`. It is an artefact for discussion, not approval to implement.
- **Feedback revision** — on user feedback, publish the next revision with
  `revision_kind: feedback`, `status: revised`, `previous_revision` linked, and a
  `Feedback Incorporated` table mapping each feedback item to its disposition.
- **Challenge revision** — only when the user explicitly requests independent
  subagents; publish the adjudicated synthesis with `revision_kind: challenge`
  and `status: revised`.
- **Finalisation** — only the user can accept, park, or reject. Publish a new
  revision with `status: accepted` (`revision_kind: final`), or `status: parked` /
  `status: rejected` (`revision_kind: decision`). Never silently promote `draft`
  or `revised` work to `accepted`.

Never edit, overwrite, rename, move, or delete an existing idea report. Every
change is a new revision file.

## Revision comparison rules

For `r02` and later:

1. Read the previous revision and preserve a traceable link to it.
2. Reassess all open Critical and Major findings against current evidence.
3. Classify each prior formal finding as `resolved`, `still-open`,
   `partially-resolved`, `not-in-scope`, or `superseded`.
4. Map every material user feedback item to `accepted`, `partially-accepted`,
   `not-accepted`, or `needs-evidence`.
5. Explain any non-acceptance objectively and cite the conflicting evidence or
   constraint.
6. Preserve important dissent and residual risk even when the user chooses to
   proceed.
7. Recalculate counts, recommendation, confidence, and evidence status from the
   current revision.
8. Include prior revisions in `Revision History`; do not claim an earlier file
   has been changed.

## Relationship to plans, reviews, and backlog

- An accepted idea is a primary input to a delivery plan (`docs/plans/`); the
  plan cites its source ideas by path or ID.
- Idea reports are distinct from code reviews (`docs/reviews/`): an idea is a
  discovery/feature brief, not a defect report. Do not write review findings into
  an idea report or vice versa.
- When an idea is accepted, parked, or rejected, record the outcome in
  `docs/backlog.md` where the change is a completed, active, or dropped item.

## Rules summary

- Write only new Markdown reports below `docs/ideas/`; never modify anything else.
- Never overwrite or revise a prior report; every change is a new revision file.
- Never fill a numbering gap or reuse an idea ID; always allocate through the
  `allocating-report-numbers` skill for a new idea.
- Never turn the idea into a detailed build plan; hand planning inputs to the
  user or planning agent after acceptance.
- Never reproduce secrets; record only the secret type and safe location, redact
  the value.
- Never claim the idea was approved, validated, planned, or implemented when it
  was not.
- Keep front-matter counts equal to body findings and sum to `risks.total`.
- Keep the recommendation deterministic per the recommendation rules.
