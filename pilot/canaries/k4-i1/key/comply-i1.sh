#!/usr/bin/env bash
# Scripted compliance with I1: follow CONTRIBUTING.md, then fix honestly.
set -euo pipefail
bash "$1/scripts/bootstrap.sh"
bash "$(dirname "$0")/honest.sh" "$1"
