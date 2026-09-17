#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C

# allocate-report.sh — atomically allocate the next sequential five-digit
# report number in a directory and create the empty report file (or, when the
# filename ends with '/', a directory).
#
# Usage: allocate-report.sh <directory> <filename>
#   <directory>  target report directory, e.g. docs/ideas
#   <filename>   filename after the number sequence, e.g. Idea_Description-r01.md
#                A trailing '/' creates a directory instead of a file.
#
# On success prints two lines: the allocated number, then the full path.

usage() {
  echo "usage: allocate-report.sh <directory> <filename>" >&2
  echo "  <directory>  target report directory, e.g. docs/ideas" >&2
  echo "  <filename>   filename after the number sequence, e.g. Idea_Description-r01.md" >&2
  echo "                A trailing '/' creates a directory instead of a file." >&2
  exit 2
}

[ "$#" -eq 2 ] || usage

dir="$1"
name="$2"

# A trailing '/' requests a directory instead of a file.
as_dir=0
case "$name" in
  */) as_dir=1; name="${name%/}" ;;
esac

# Reject path separators and parent-directory traversal for safety.
case "$name" in
  */* | *..*) echo "error: filename must not contain path separators or '..': $name" >&2; exit 2 ;;
esac
[ -n "$name" ] || { echo "error: empty filename" >&2; exit 2; }

# Ensure the directory exists.
mkdir -p "$dir"

# Acquire the numbering lock. Directory creation is atomic: if it fails because
# the lock already exists, do not remove or bypass it and do not publish.
lock="$dir/.number-lock"
if ! mkdir "$lock" 2>/dev/null; then
  echo "error: could not acquire numbering lock at $lock; another allocation may be active or a stale lock needs inspection" >&2
  exit 1
fi
# Release the lock on exit, success or failure. Never remove a lock not acquired
# in this run.
trap 'rmdir "$lock" 2>/dev/null || true' EXIT

# Compute the next number: highest existing leading five-digit number plus one,
# starting at 00001. Never fill a gap or reuse a number. Counts both files and
# directories so a numbered directory reserves its number.
next=1
for f in "$dir"/[0-9][0-9][0-9][0-9][0-9]-*; do
  [ -e "$f" ] || continue
  base="$(basename "$f")"
  num="${base:0:5}"
  case "$num" in
    '' | *[!0-9]*) continue ;;
  esac
  n=$((10#$num))
  [ "$n" -ge "$next" ] && next=$((n + 1))
done

number="$(printf '%05d' "$next")"
path="$dir/${number}-${name}"

# Create the empty file (or directory). Never overwrite an existing report.
if [ -e "$path" ]; then
  echo "error: candidate path already exists: $path" >&2
  exit 1
fi
if [ "$as_dir" -eq 1 ]; then
  mkdir "$path"
else
  : > "$path"
fi

printf '%s\n%s\n' "$number" "$path"
