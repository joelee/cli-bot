#!/usr/bin/env bash
set -euo pipefail

git config core.hooksPath .githooks
printf 'Git hooks installed at %s\n' "$(pwd)/.githooks"
