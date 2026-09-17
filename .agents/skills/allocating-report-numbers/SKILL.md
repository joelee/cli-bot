---
name: allocating-report-numbers
description: Use when creating a new numbered report file in any report directory and needing the next sequential five-digit number without gaps or reuse.
---

# Allocating Report Numbers

## Overview

Numbered report files share one allocation rule: the next five-digit number is
the highest existing number plus one, starting at `00001`, never filling a gap
or reusing a number. This skill's script performs that allocation atomically and
creates the empty report file (or, when the filename ends with `/`, a
directory). It is directory-agnostic: any agent may point it at any report
directory (for example `docs/ideas/`, `docs/plans/`, or `docs/reviews/`, or a
new directory introduced later).

## When to use

Use when an agent must create a new numbered report file in any report
directory. Do not use for revisions of an existing idea (IdeaArchitect handles
`-rNN` revisions itself) or for amending an existing plan or review in place.

## How to allocate

Run the script with the target directory and the filename that follows the
number sequence:

```bash
.agents/skills/allocating-report-numbers/allocate-report.sh <directory> <filename>
```

- `<directory>` — any report directory, e.g. `docs/ideas`, `docs/plans`, or
  `docs/reviews`.
- `<filename>` — the filename after the number sequence, including its extension,
  e.g. `Idea_Description-r01.md`, `Plan_Description.md`, or `Review_Description.md`.
  A trailing `/` creates a directory instead of a file, e.g. `My_Report/`.

The script:

1. Creates the directory if absent.
2. Acquires an atomic lock (`<directory>/.number-lock`). If the lock is held, it
   fails without writing — report that another allocation may be active or that a
   stale lock needs inspection.
3. Scans `<directory>/[0-9][0-9][0-9][0-9][0-9]-*` (files and directories), takes
   the highest leading five-digit number plus one (starting at `00001`), never
   filling a gap.
4. Creates the empty file `<directory>/NNNNN-<filename>`, or the directory
   `<directory>/NNNNN-<filename>` when `<filename>` ends with `/`.
5. Releases the lock.

On success it prints two lines: the allocated number, then the full path.

```text
00009
docs/reviews/00009-Review_Description.md
```

## After allocation

Write the report content into the file the script created, using the agent's own
front-matter and body schema. Never overwrite, rename, or delete an existing
report.

## Common mistakes

- Reusing a number or filling a gap — the script never does this; do not bypass it.
- Passing a filename with path separators or `..` — the script rejects these.
- Removing a lock you did not acquire — the script releases only its own lock.